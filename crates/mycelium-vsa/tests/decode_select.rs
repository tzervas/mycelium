//! **Decode-methodology selection** tests (RFC-0010; M-350). Exercise the three arms of the
//! `reconstruct_factors_auto` selector and the honesty floor: brute-force enumeration is `Exact` and
//! identifiability-checked; the resonator is `Empirical` and in-regime only; everything else is an
//! explicit refusal — and a forced override **cannot** escape the floor (RFC-0010 §4.4/§4.5). The
//! selection always emits the mandatory EXPLAIN (RFC-0005 §2.2). No `rand` (deterministic LCG).

use mycelium_core::GuaranteeStrength;
use mycelium_vsa::{
    decode_method_policy, explain_decode_method, reconstruct_factors_auto, CleanupMemory,
    DecodeMethod, MapI, ResonatorParams, VsaError, VsaModel, DEFAULT_ENUM_BUDGET,
};

const D: u32 = 4096;

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

/// Build `f` codebooks of `k` bipolar atoms at dim `D`; return the memories and the raw atoms.
fn codebooks(f: usize, k: usize, seed: u64) -> (Vec<CleanupMemory>, Vec<Vec<Vec<f64>>>) {
    let mut lcg = Lcg::new(seed);
    let mut mems = Vec::with_capacity(f);
    let mut atoms = Vec::with_capacity(f);
    for i in 0..f {
        let mut mem = CleanupMemory::new(D);
        let mut slot = Vec::with_capacity(k);
        for j in 0..k {
            let a = lcg.bipolar(D);
            mem.insert(format!("{i}:{j}"), a.clone()).unwrap();
            slot.push(a);
        }
        mems.push(mem);
        atoms.push(slot);
    }
    (mems, atoms)
}

fn bind_tuple(model: &MapI, atoms: &[Vec<Vec<f64>>], tuple: &[usize]) -> Vec<f64> {
    let mut acc = atoms[0][tuple[0]].clone();
    for slot in 1..atoms.len() {
        acc = model.bind(&acc, &atoms[slot][tuple[slot]]).unwrap();
    }
    acc
}

fn params() -> ResonatorParams {
    ResonatorParams::mapi_default(50, 0xD0DE)
}

#[test]
fn small_instance_routes_to_brute_force_exact() {
    let model = MapI::new(D);
    let (mems, atoms) = codebooks(2, 8, 1); // ∏k = 64 ≤ DEFAULT_ENUM_BUDGET
    let truth = [5usize, 2];
    let s = bind_tuple(&model, &atoms, &truth);
    let out =
        reconstruct_factors_auto(&model, &s, &mems, &params(), DEFAULT_ENUM_BUDGET, None).unwrap();
    assert_eq!(out.method, DecodeMethod::BruteForceExact);
    assert_eq!(out.guarantee, GuaranteeStrength::Exact);
    assert!(out.resonator_trace.is_none());
    assert_eq!([out.factors[0].index, out.factors[1].index], truth);
    // Exact recovery ⇒ full confidence and a positive identifiability margin.
    assert!((out.factors[0].confidence - 1.0).abs() < 1e-9);
    assert!(out.factors[0].margin > 0.0);
    // The mandatory EXPLAIN fired the first table rule (capacity ≤ budget) and ranks all 3 arms.
    assert_eq!(out.explanation.matched_rule, Some(0));
    assert_eq!(out.explanation.costs.len(), 3);
}

#[test]
fn in_regime_but_over_budget_routes_to_resonator() {
    // ∏k = 512 (F=3,k=8) is in-regime but over a *tight* budget of 64 ⇒ the resonator arm.
    let model = MapI::new(D);
    let (mems, atoms) = codebooks(3, 8, 2);
    let truth = [1usize, 6, 3];
    let s = bind_tuple(&model, &atoms, &truth);
    let out = reconstruct_factors_auto(&model, &s, &mems, &params(), 64, None).unwrap();
    assert_eq!(out.method, DecodeMethod::Resonator);
    assert_eq!(out.guarantee, GuaranteeStrength::Empirical);
    assert!(out.resonator_trace.is_some());
    assert_eq!(
        [
            out.factors[0].index,
            out.factors[1].index,
            out.factors[2].index
        ],
        truth
    );
    assert_eq!(out.explanation.matched_rule, Some(1)); // the in-regime rule
}

#[test]
fn out_of_regime_and_over_budget_refuses() {
    // F=3,k=32 (∏=32768): out of regime AND far over a tight budget ⇒ explicit Refuse (no decode).
    let model = MapI::new(D);
    let (mems, atoms) = codebooks(3, 32, 3);
    let s = bind_tuple(&model, &atoms, &[0, 0, 0]);
    match reconstruct_factors_auto(&model, &s, &mems, &params(), 64, None) {
        Err(VsaError::DecodeRefused { detail }) => assert!(detail.contains("outside")),
        other => panic!("expected DecodeRefused, got {other:?}"),
    }
}

#[test]
fn forced_brute_force_beyond_budget_still_refuses() {
    // Honesty floor (RFC-0010 §4.5): forcing brute force past the enumeration budget refuses rather
    // than enumerating an intractable grid — it does NOT silently run.
    let model = MapI::new(D);
    let (mems, atoms) = codebooks(3, 8, 4); // ∏=512 > budget 64
    let s = bind_tuple(&model, &atoms, &[2, 2, 2]);
    match reconstruct_factors_auto(
        &model,
        &s,
        &mems,
        &params(),
        64,
        Some(DecodeMethod::BruteForceExact),
    ) {
        Err(VsaError::DecodeRefused { detail }) => assert!(detail.contains("exceeds")),
        other => panic!("expected DecodeRefused, got {other:?}"),
    }
}

#[test]
fn forced_resonator_out_of_regime_still_refuses() {
    // Honesty floor: forcing the resonator outside its validated regime refuses with the profile's
    // own explicit reason (never an unvalidated Empirical decode).
    let model = MapI::new(D);
    let (mems, atoms) = codebooks(3, 32, 5); // k=32 is out of regime
    let s = bind_tuple(&model, &atoms, &[0, 0, 0]);
    match reconstruct_factors_auto(
        &model,
        &s,
        &mems,
        &params(),
        DEFAULT_ENUM_BUDGET,
        Some(DecodeMethod::Resonator),
    ) {
        Err(VsaError::OutsideEmpiricalProfile { .. }) => {}
        other => panic!("expected OutsideEmpiricalProfile, got {other:?}"),
    }
}

#[test]
fn non_identifiable_brute_force_refuses() {
    // A slot with two *identical* atoms makes two tuples produce the same product ⇒ the true tuple is
    // not the unique arg-max. The Exact arm refuses (NonIdentifiable), never coin-flips (RFC-0010 §4.4).
    let model = MapI::new(D);
    let mut lcg = Lcg::new(7);
    let dup = lcg.bipolar(D);
    let mut c0 = CleanupMemory::new(D);
    c0.insert("0:0", dup.clone()).unwrap();
    c0.insert("0:1", dup.clone()).unwrap(); // identical ⇒ ambiguous
    let mut c1 = CleanupMemory::new(D);
    let a1: Vec<Vec<f64>> = (0..4).map(|_| lcg.bipolar(D)).collect();
    for (j, a) in a1.iter().enumerate() {
        c1.insert(format!("1:{j}"), a.clone()).unwrap();
    }
    let s = model.bind(&dup, &a1[2]).unwrap();
    match reconstruct_factors_auto(&model, &s, &[c0, c1], &params(), DEFAULT_ENUM_BUDGET, None) {
        Err(VsaError::NonIdentifiable { .. }) => {}
        other => panic!("expected NonIdentifiable, got {other:?}"),
    }
}

#[test]
fn explain_is_pure_and_matches_a_run() {
    // The standalone EXPLAIN (no execution) agrees with the explanation a real run records — the
    // selection is a deterministic function of (policy, inputs) (RFC-0005 §2.3).
    let model = MapI::new(D);
    let (mems, atoms) = codebooks(2, 8, 8);
    let s = bind_tuple(&model, &atoms, &[0, 1]);
    let pure = explain_decode_method(&model, &[8, 8], D, DEFAULT_ENUM_BUDGET);
    let out =
        reconstruct_factors_auto(&model, &s, &mems, &params(), DEFAULT_ENUM_BUDGET, None).unwrap();
    assert_eq!(pure.chosen_index, out.explanation.chosen_index);
    assert_eq!(pure.policy, out.explanation.policy);
    assert_eq!(pure.matched_rule, out.explanation.matched_rule);
    // Same policy + same regime facts ⇒ identical decision; the policy is content-addressed.
    assert_eq!(
        decode_method_policy(DEFAULT_ENUM_BUDGET).policy_ref(),
        pure.policy
    );
}

#[test]
fn determinism_same_inputs_same_selection() {
    let model = MapI::new(D);
    let (m1, atoms) = codebooks(2, 8, 9);
    let (m2, _) = codebooks(2, 8, 9);
    let s = bind_tuple(&model, &atoms, &[4, 4]);
    let a =
        reconstruct_factors_auto(&model, &s, &m1, &params(), DEFAULT_ENUM_BUDGET, None).unwrap();
    let b =
        reconstruct_factors_auto(&model, &s, &m2, &params(), DEFAULT_ENUM_BUDGET, None).unwrap();
    assert_eq!(a, b);
}

#[test]
fn empty_and_mismatched_inputs_are_explicit() {
    let model = MapI::new(D);
    let s = vec![1.0; D as usize];
    assert!(matches!(
        reconstruct_factors_auto(&model, &s, &[], &params(), DEFAULT_ENUM_BUDGET, None),
        Err(VsaError::EmptyCodebook)
    ));
    let mut wrong = CleanupMemory::new(8);
    wrong.insert("x", vec![1.0; 8]).unwrap();
    assert!(matches!(
        reconstruct_factors_auto(&model, &s, &[wrong], &params(), DEFAULT_ENUM_BUDGET, None),
        Err(VsaError::DimMismatch { .. })
    ));
}
