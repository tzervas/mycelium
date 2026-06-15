//! Brute-force **differential oracle** for resonator factorization (RFC-0009 §5.3 / §10.2 / §11;
//! M-350). The certificate quantity is **exact-tuple recovery against ground truth**, not
//! self-reported convergence (§8.1 P5): we build `s` from a *known* factor tuple, run the resonator,
//! and assert it recovers exactly that tuple. A companion **identifiability** check confirms the true
//! tuple is the global arg-max over all `∏ᵢ kᵢ` combinations — so a resonator miss is a *resonator*
//! failure, not an ambiguous instance.

use mycelium_vsa::{factorize, CleanupMemory, MapI, ResonatorParams, StopReason, VsaModel};

/// A tiny deterministic LCG (no `rand` — house rule). Same constants as the in-crate one.
struct Lcg(u64);
impl Lcg {
    fn new(seed: u64) -> Self {
        Lcg(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1))
    }
    fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }
    fn bipolar(&mut self, dim: u32) -> Vec<f64> {
        (0..dim)
            .map(|_| {
                if (self.next_u64() >> 63) & 1 == 1 {
                    1.0
                } else {
                    -1.0
                }
            })
            .collect()
    }
}

/// Build `f` codebooks of `k` bipolar atoms each at dimension `dim`, returning the cleanup memories
/// and the raw atoms (for the oracle's exhaustive scoring).
fn codebooks(
    f: usize,
    k: usize,
    dim: u32,
    mut lcg: Lcg,
) -> (Vec<CleanupMemory>, Vec<Vec<Vec<f64>>>) {
    let mut mems = Vec::with_capacity(f);
    let mut atoms = Vec::with_capacity(f);
    for i in 0..f {
        let mut mem = CleanupMemory::new(dim);
        let mut slot = Vec::with_capacity(k);
        for j in 0..k {
            let a = lcg.bipolar(dim);
            mem.insert(format!("{i}:{j}"), a.clone()).unwrap();
            slot.push(a);
        }
        mems.push(mem);
        atoms.push(slot);
    }
    (mems, atoms)
}

/// Bind one chosen atom per slot into the product `s` (MAP-I elementwise product).
fn bind_tuple(model: &MapI, atoms: &[Vec<Vec<f64>>], tuple: &[usize]) -> Vec<f64> {
    let mut acc = atoms[0][tuple[0]].clone();
    for slot in 1..atoms.len() {
        acc = model.bind(&acc, &atoms[slot][tuple[slot]]).unwrap();
    }
    acc
}

/// The brute-force oracle: the true tuple is the global arg-max of `similarity(s, bind(tuple))` over
/// all `∏ᵢ kᵢ` combinations. Returns `true` iff the instance is identifiable (true tuple wins
/// uniquely), so a resonator failure on an identifiable instance is unambiguous.
fn is_identifiable(model: &MapI, s: &[f64], atoms: &[Vec<Vec<f64>>], truth: &[usize]) -> bool {
    let f = atoms.len();
    let mut best_sim = f64::NEG_INFINITY;
    let mut best: Vec<usize> = vec![0; f];
    // Enumerate the full grid (small by construction).
    let mut idx = vec![0usize; f];
    loop {
        let cand = bind_tuple(model, atoms, &idx);
        let sim = model.similarity(s, &cand);
        if sim > best_sim {
            best_sim = sim;
            best = idx.clone();
        }
        // Increment the mixed-radix counter.
        let mut carry = 0;
        idx[carry] += 1;
        while idx[carry] == atoms[carry].len() {
            idx[carry] = 0;
            carry += 1;
            if carry == f {
                return best == truth;
            }
            idx[carry] += 1;
        }
    }
}

#[test]
fn oracle_exact_recovery_on_small_instances() {
    let model = MapI::new(4096);
    // A handful of fully-enumerated F=2 instances at k ∈ {4, 8}, distinct seeds + true tuples.
    let cases = [
        (4usize, [0usize, 0usize], 1u64),
        (4, [3, 1], 2),
        (8, [5, 2], 3),
        (8, [0, 7], 4),
        (8, [6, 6], 5),
    ];
    for (k, truth, seed) in cases {
        let (mems, atoms) = codebooks(2, k, 4096, Lcg::new(seed));
        let s = bind_tuple(&model, &atoms, &truth);
        assert!(
            is_identifiable(&model, &s, &atoms, &truth),
            "instance (k={k}, seed={seed}) must be identifiable"
        );
        let params = ResonatorParams::mapi_default(50, 0x0_AC1E ^ seed);
        let out = factorize(&model, &s, &mems, &params)
            .unwrap_or_else(|e| panic!("k={k} seed={seed}: expected recovery, got {e:?}"));
        assert_eq!(out.trace.stop, StopReason::Converged);
        assert_eq!(
            [out.factors[0].index, out.factors[1].index],
            truth,
            "k={k} seed={seed}: recovered the wrong tuple"
        );
    }
}

#[test]
fn instance_is_identifiable_by_exhaustive_argmax() {
    // The oracle's premise: on a clean F=2, k=8 bipolar instance at d=4096 the true tuple is the
    // unique global arg-max. (If this ever failed, a resonator miss could be the instance's fault.)
    let model = MapI::new(4096);
    let (_, atoms) = codebooks(2, 8, 4096, Lcg::new(777));
    let truth = [4, 1];
    let s = bind_tuple(&model, &atoms, &truth);
    assert!(is_identifiable(&model, &s, &atoms, &truth));
}
