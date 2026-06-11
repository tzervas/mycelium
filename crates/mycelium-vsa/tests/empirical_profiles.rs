//! M-240/M-241 — **trial validation of the declared `Empirical` profiles** (M-I3/VR-5; SC-2).
//!
//! Each crate-declared [`EmpiricalProfile`](mycelium_vsa::EmpiricalProfile) constant promises a δ
//! over a stated regime with a stated trial count; this suite **runs exactly those trials** at
//! the profile's worst covered point (max items, min dim) and asserts the measured failure rate
//! stays at or below the declared δ. The basis recorded on values (`EmpiricalFit{trials, …}`) is
//! honest *because* this suite exists — the constants are exercised, never merely asserted.

use mycelium_vsa::{
    bsc::BSC_BUNDLE_PROFILE, fhrr::FHRR_UNBIND_PROFILE, hrr::HRR_UNBIND_PROFILE,
    mapb::MAPB_BUNDLE_PROFILE, Bsc, CleanupMemory, Fhrr, Hrr, MapB, VsaModel,
};

/// Deterministic generator (tiny LCG — house style, no rand dependency).
struct Lcg(u64);
impl Lcg {
    fn new(seed: u64) -> Self {
        Lcg(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1))
    }
    fn bit(&mut self) -> bool {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 63) & 1 == 1
    }
    fn unif(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 11) as f64 / (1u64 << 53) as f64).max(1e-12)
    }
    fn bipolar(&mut self, dim: usize) -> Vec<f64> {
        (0..dim)
            .map(|_| if self.bit() { 1.0 } else { -1.0 })
            .collect()
    }
    fn binary(&mut self, dim: usize) -> Vec<f64> {
        (0..dim)
            .map(|_| if self.bit() { 1.0 } else { 0.0 })
            .collect()
    }
    /// ~N(0, 1/d) atom (Box–Muller).
    fn gaussian(&mut self, dim: usize) -> Vec<f64> {
        let scale = 1.0 / (dim as f64).sqrt();
        (0..dim)
            .map(|_| {
                let (u1, u2) = (self.unif(), self.unif());
                scale * (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
            })
            .collect()
    }
    /// Uniform phasor atom (phases in `(−π, π]`).
    fn phasor(&mut self, dim: usize) -> Vec<f64> {
        (0..dim)
            .map(|_| {
                let t = std::f64::consts::TAU * self.unif();
                if t > std::f64::consts::PI {
                    t - std::f64::consts::TAU
                } else {
                    t
                }
            })
            .collect()
    }
}

const CODEBOOK: usize = 8;

/// Membership decode failure: some non-member out-ranks some member by the model's similarity.
fn decode_fails<M: VsaModel>(
    model: &M,
    bundle: &[f64],
    codebook: &[Vec<f64>],
    members: usize,
) -> bool {
    let member_min = codebook[..members]
        .iter()
        .map(|a| model.similarity(bundle, a))
        .fold(f64::INFINITY, f64::min);
    let stranger_max = codebook[members..]
        .iter()
        .map(|a| model.similarity(bundle, a))
        .fold(f64::NEG_INFINITY, f64::max);
    member_min <= stranger_max
}

/// MAP-B: the declared bundle profile holds at its worst covered point (m = max items,
/// d = min dim) over exactly the declared trial count.
#[test]
fn mapb_bundle_profile_holds_over_declared_trials() {
    let p = MAPB_BUNDLE_PROFILE;
    let model = MapB::new(p.min_dim);
    let m = p.max_items;
    let mut failures = 0usize;
    for trial in 0..p.trials {
        let mut rng = Lcg::new(0xB1_B0 ^ trial);
        let codebook: Vec<Vec<f64>> = (0..CODEBOOK)
            .map(|_| rng.bipolar(p.min_dim as usize))
            .collect();
        let refs: Vec<&[f64]> = codebook[..m].iter().map(Vec::as_slice).collect();
        let bundle = model.bundle(&refs).unwrap();
        if decode_fails(&model, &bundle, &codebook, m) {
            failures += 1;
        }
    }
    let rate = failures as f64 / p.trials as f64;
    assert!(
        rate <= p.delta,
        "MAP-B empirical rate {rate} exceeded the declared δ={} ({failures}/{} failures)",
        p.delta,
        p.trials
    );
}

/// BSC: the declared bundle profile holds at its worst covered point over exactly the declared
/// trial count.
#[test]
fn bsc_bundle_profile_holds_over_declared_trials() {
    let p = BSC_BUNDLE_PROFILE;
    let model = Bsc::new(p.min_dim);
    let m = p.max_items;
    let mut failures = 0usize;
    for trial in 0..p.trials {
        let mut rng = Lcg::new(0x5C_5C ^ trial);
        let codebook: Vec<Vec<f64>> = (0..CODEBOOK)
            .map(|_| rng.binary(p.min_dim as usize))
            .collect();
        let refs: Vec<&[f64]> = codebook[..m].iter().map(Vec::as_slice).collect();
        let bundle = model.bundle(&refs).unwrap();
        if decode_fails(&model, &bundle, &codebook, m) {
            failures += 1;
        }
    }
    let rate = failures as f64 / p.trials as f64;
    assert!(
        rate <= p.delta,
        "BSC empirical rate {rate} exceeded the declared δ={} ({failures}/{} failures)",
        p.delta,
        p.trials
    );
}

/// One bind→unbind→cleanup recovery trial: returns `true` on failure (wrong atom recovered).
fn unbind_recovery_fails<M: VsaModel>(
    model: &M,
    role: &[f64],
    codebook: &[(String, Vec<f64>)],
    target: usize,
    mem: &CleanupMemory,
) -> bool {
    let bound = model.bind(role, &codebook[target].1).unwrap();
    let noisy = model.unbind(&bound, role).unwrap();
    match mem.cleanup(&noisy, model) {
        Some(hit) => hit.index != target,
        None => true,
    }
}

/// HRR: the declared single-factor unbind profile holds at its worst covered point (min dim,
/// codebook 16) over exactly the declared trial count.
#[test]
fn hrr_unbind_profile_holds_over_declared_trials() {
    let p = HRR_UNBIND_PROFILE;
    let model = Hrr::new(p.min_dim);
    let dim = p.min_dim as usize;
    let mut failures = 0usize;
    for trial in 0..p.trials {
        let mut rng = Lcg::new(0x44_88 ^ trial);
        let codebook: Vec<(String, Vec<f64>)> = (0..16)
            .map(|i| (format!("atom{i}"), rng.gaussian(dim)))
            .collect();
        let mut mem = CleanupMemory::new(p.min_dim);
        for (label, atom) in &codebook {
            mem.insert(label.clone(), atom.clone()).unwrap();
        }
        let role = rng.gaussian(dim);
        let target = (trial % 16) as usize;
        if unbind_recovery_fails(&model, &role, &codebook, target, &mem) {
            failures += 1;
        }
    }
    let rate = failures as f64 / p.trials as f64;
    assert!(
        rate <= p.delta,
        "HRR unbind empirical rate {rate} exceeded the declared δ={} ({failures}/{} failures)",
        p.delta,
        p.trials
    );
}

/// FHRR: the declared single-factor unbind profile holds at its worst covered point over exactly
/// the declared trial count.
#[test]
fn fhrr_unbind_profile_holds_over_declared_trials() {
    let p = FHRR_UNBIND_PROFILE;
    let model = Fhrr::new(p.min_dim);
    let dim = p.min_dim as usize;
    let mut failures = 0usize;
    for trial in 0..p.trials {
        let mut rng = Lcg::new(0xF4_44 ^ trial);
        let codebook: Vec<(String, Vec<f64>)> = (0..16)
            .map(|i| (format!("atom{i}"), rng.phasor(dim)))
            .collect();
        let mut mem = CleanupMemory::new(p.min_dim);
        for (label, atom) in &codebook {
            mem.insert(label.clone(), atom.clone()).unwrap();
        }
        let role = rng.phasor(dim);
        let target = (trial % 16) as usize;
        if unbind_recovery_fails(&model, &role, &codebook, target, &mem) {
            failures += 1;
        }
    }
    let rate = failures as f64 / p.trials as f64;
    assert!(
        rate <= p.delta,
        "FHRR unbind empirical rate {rate} exceeded the declared δ={} ({failures}/{} failures)",
        p.delta,
        p.trials
    );
}
