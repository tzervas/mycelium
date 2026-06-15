//! The trial-validated **resonator profile** gate (RFC-0009 §5.2 / §9 Q4 / §11; M-350). This is the
//! single test that *earns* the `Empirical` δ for [`MAPI_RESONATOR_PROFILE`]: it runs **exactly**
//! `profile.trials` Monte-Carlo trials at the profile's worst covered point (max factors, max
//! codebook, min dim), scoring **exact-tuple recovery against ground truth** (not self-reported
//! convergence — §8.1 P5), and asserts the measured failure rate stays at or below `profile.delta`.
//! Mirrors the bundle pattern in `tests/empirical_profiles.rs`; no `rand` dependency (deterministic
//! LCG). The δ in the const is the conservative ceiling this run confirms, never asserted ahead of it.

use mycelium_vsa::{
    factorize, CleanupMemory, MapI, ResonatorParams, VsaModel, MAPI_RESONATOR_PROFILE,
};

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

/// One trial: build `f` fresh codebooks of `k` bipolar atoms at `dim`, pick a true tuple, bind it,
/// factorize, and return `true` iff the resonator recovers **exactly** the true tuple. A
/// wrong-fixed-point `Ok`, an oscillation/budget error, or a below-gate refusal all count as failure
/// (RFC-0009 §5.3/§6).
fn recovery_fails(model: &MapI, f: usize, k: usize, dim: u32, lcg: &mut Lcg) -> bool {
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
    let truth: Vec<usize> = (0..f)
        .map(|_| (lcg.next_u64() % k as u64) as usize)
        .collect();
    let mut s = atoms[0][truth[0]].clone();
    for slot in 1..f {
        s = model.bind(&s, &atoms[slot][truth[slot]]).unwrap();
    }
    let params = ResonatorParams::mapi_default(50, lcg.next_u64());
    match factorize(model, &s, &mems, &params) {
        Ok(out) => (0..f).any(|i| out.factors[i].index != truth[i]),
        Err(_) => true,
    }
}

#[test]
fn mapi_resonator_profile_holds_over_declared_trials() {
    let p = &MAPI_RESONATOR_PROFILE;
    // Run at the profile's worst covered point: max factors, max codebook, the floor dimension.
    let f = p.max_factors;
    let k = p.max_codebook;
    let dim = p.min_dim;
    let model = MapI::new(dim);

    let mut failures = 0u64;
    for trial in 0..p.trials {
        // Deterministic per-trial seed (mirrors empirical_profiles.rs; a resonator-specific salt).
        let mut lcg = Lcg::new(0x8E50_0000 ^ trial);
        if recovery_fails(&model, f, k, dim, &mut lcg) {
            failures += 1;
        }
    }
    let rate = failures as f64 / p.trials as f64;
    // Transparency: the measured rate is the evidence the const's δ records (run with --nocapture).
    eprintln!(
        "resonator profile: {failures}/{} failures, rate={rate} (δ={})",
        p.trials, p.delta
    );
    assert!(
        rate <= p.delta,
        "measured resonator failure rate {rate} ({failures}/{}) exceeds the profile's δ={} \
         (F={f}, k={k}, d={dim}) — the Empirical tag would outrun its evidence (VR-5)",
        p.trials,
        p.delta
    );
}
