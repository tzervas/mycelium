//! M-131 — empirical validation of the MAP-I bundle capacity bound (SC-2).
//!
//! The `Proven` capacity bound (`mycelium_vsa::capacity`) cites Clarkson/Thomas and is issued only
//! when the checked side-condition `dim ≥ requiredDim(m, δ)` holds (M-001 pattern). This test
//! *empirically validates* that the bound is not vacuous: over **≥10⁴ independent trials** at a
//! dimension that satisfies the side-condition, the measured retrieval-failure rate stays at or
//! below the proven target `δ`. (It does not re-prove the theorem — it checks the instantiation
//! behaves as claimed, the SC-2 obligation.)

use mycelium_core::{Meta, Payload, Provenance, Repr, SparsityClass, Value};
use mycelium_vsa::{capacity, MapI, VsaError};

/// Deterministic bipolar (`±1`) atom generator (a tiny LCG — reproducible, no rand dependency).
struct Lcg(u64);
impl Lcg {
    fn new(seed: u64) -> Self {
        Lcg(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1))
    }
    fn bit(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        if (self.0 >> 63) & 1 == 1 {
            1.0
        } else {
            -1.0
        }
    }
    fn atom(&mut self, dim: usize) -> Vec<f64> {
        (0..dim).map(|_| self.bit()).collect()
    }
}

const M: u64 = 3; // items per bundle
const DELTA: f64 = 1e-2; // proven target failure probability
const N: usize = 8; // codebook size
const TRIALS: usize = 10_000; // ≥ 1e4 (SC-2)

/// SC-2: with `dim ≥ requiredDim(M, δ)`, the empirical retrieval-failure rate is `≤ δ` over ≥10⁴
/// trials — every bundled member out-scores every non-member by nearest-neighbour cleanup.
#[test]
fn bundle_capacity_holds_over_1e4_trials() {
    let dim = capacity::required_dim(M, DELTA, capacity::MARGIN_MU) as usize; // 1141
    assert!(dim >= 1141);

    let mut failures = 0usize;
    for trial in 0..TRIALS {
        let mut rng = Lcg::new(0xC0FFEE ^ trial as u64);
        // A fresh codebook of N atoms; bundle the first M of them.
        let codebook: Vec<Vec<f64>> = (0..N).map(|_| rng.atom(dim)).collect();
        let mut bundle = vec![0.0f64; dim];
        for atom in codebook.iter().take(M as usize) {
            for (b, x) in bundle.iter_mut().zip(atom) {
                *b += x;
            }
        }
        // Dot of the bundle with each codebook atom (norms are equal, so dot ranks = cosine ranks).
        let dot = |atom: &[f64]| -> f64 { bundle.iter().zip(atom).map(|(b, x)| b * x).sum() };
        let member_min = (0..M as usize)
            .map(|i| dot(&codebook[i]))
            .fold(f64::INFINITY, f64::min);
        let stranger_max = (M as usize..N)
            .map(|j| dot(&codebook[j]))
            .fold(f64::NEG_INFINITY, f64::max);
        // Failure: some non-member out-ranks some member (cleanup would mis-retrieve).
        if member_min <= stranger_max {
            failures += 1;
        }
    }

    let rate = failures as f64 / TRIALS as f64;
    assert!(
        rate <= DELTA,
        "empirical failure rate {rate} exceeded the proven δ={DELTA} (failures={failures}/{TRIALS}, dim={dim})"
    );
}

/// The certified Value-level bundle issues a `Proven` `CapacityBound` exactly when the dimension
/// meets the side-condition, and refuses (explicitly) when it does not — the honest downgrade.
#[test]
fn certified_bundle_is_proven_only_when_dimension_suffices() {
    let dim = capacity::required_dim(M, DELTA, capacity::MARGIN_MU) as u32; // 1141
    let model = MapI::new(dim);

    let mut rng = Lcg::new(42);
    let items: Vec<Value> = (0..M)
        .map(|_| {
            Value::new(
                Repr::Vsa {
                    model: "MAP-I".to_owned(),
                    dim,
                    sparsity: SparsityClass::Dense,
                },
                Payload::Hypervector(rng.atom(dim as usize)),
                Meta::exact(Provenance::Root),
            )
            .unwrap()
        })
        .collect();
    let refs: Vec<&Value> = items.iter().collect();

    // Sufficient dimension → Proven bound.
    let bundle = model.bundle_values_certified(&refs, DELTA).expect("proven");
    assert_eq!(
        bundle.meta().guarantee(),
        mycelium_core::GuaranteeStrength::Proven
    );
    match bundle.meta().bound() {
        Some(b) => {
            assert!(matches!(
                b.basis,
                mycelium_core::BoundBasis::ProvenThm { .. }
            ));
            assert!(matches!(
                b.kind,
                mycelium_core::BoundKind::Capacity { items: 3, .. }
            ));
        }
        None => panic!("a Proven bundle must carry a bound (M-I1)"),
    }

    // Undersized model → explicit InsufficientCapacity, never an unbacked Proven tag.
    let small = MapI::new(64);
    let small_items: Vec<Value> = (0..M)
        .map(|_| {
            Value::new(
                Repr::Vsa {
                    model: "MAP-I".to_owned(),
                    dim: 64,
                    sparsity: SparsityClass::Dense,
                },
                Payload::Hypervector(Lcg::new(1).atom(64)),
                Meta::exact(Provenance::Root),
            )
            .unwrap()
        })
        .collect();
    let small_refs: Vec<&Value> = small_items.iter().collect();
    assert!(matches!(
        small.bundle_values_certified(&small_refs, DELTA),
        Err(VsaError::InsufficientCapacity { .. })
    ));
}
