//! Property tests for the verified-numerics kernels (E2-4; RFC-0001 §4.7 — Soundness, Monotonicity,
//! Determinism). Following the Phase-1 house style, randomness is a tiny inline LCG (no `proptest`/
//! `rand` dependency); the trial counts make these de-facto property tests over many cases.

use mycelium_core::{Bound, BoundBasis, BoundKind, GuaranteeStrength, NormKind};
use mycelium_numerics::{
    accuracy_to_probability, check_error_claim, check_union_claim, compose_error_bound,
    recompute_error, AffineForm, ApRhlJudgment, Certificate, CheckOutcome, ErrorBound, ErrorOp,
    ProbBound,
};

/// A tiny deterministic LCG (SplitMix-ish), seeded — the Phase-1 trial-test pattern, no `rand` dep.
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
    /// Uniform in `[-1, 1]`.
    fn unit(&mut self) -> f64 {
        (self.next_u64() as f64 / u64::MAX as f64) * 2.0 - 1.0
    }
    /// Uniform in `[0, hi]`.
    fn nonneg(&mut self, hi: f64) -> f64 {
        (self.next_u64() as f64 / u64::MAX as f64) * hi
    }
}

const TRIALS: usize = 20_000;

// --- ErrorBound / AffineForm -------------------------------------------------------------------

/// **Soundness (affine, linear ops are exact).** For every noise assignment, the composed form
/// evaluates to *exactly* the corresponding real operation — `add`/`sub`/`neg`/`scale` introduce no
/// error (the affine domain is exact on linear maps; ADR-010 §1).
#[test]
fn affine_linear_ops_are_exact() {
    let mut rng = Lcg::new(1);
    for _ in 0..TRIALS {
        // Two forms over a shared symbol 0 plus private symbols 1 and 2 — exercises correlation.
        let x = AffineForm::uncertain(rng.unit() * 5.0, 0, rng.nonneg(3.0))
            .add(&AffineForm::uncertain(0.0, 1, rng.nonneg(2.0)));
        let y = AffineForm::uncertain(rng.unit() * 5.0, 0, rng.nonneg(3.0))
            .add(&AffineForm::uncertain(0.0, 2, rng.nonneg(2.0)));
        let e0 = rng.unit();
        let e1 = rng.unit();
        let e2 = rng.unit();
        let assign = |s| match s {
            0 => e0,
            1 => e1,
            2 => e2,
            _ => 0.0,
        };
        let c = rng.unit() * 4.0;

        let approx_add = x.add(&y).eval(assign);
        let exact_add = x.eval(assign) + y.eval(assign);
        assert!((approx_add - exact_add).abs() < 1e-9);

        let approx_sub = x.sub(&y).eval(assign);
        let exact_sub = x.eval(assign) - y.eval(assign);
        assert!((approx_sub - exact_sub).abs() < 1e-9);

        let approx_scale = x.scale(c).eval(assign);
        assert!((approx_scale - c * x.eval(assign)).abs() < 1e-9);

        let approx_neg = x.neg().eval(assign);
        assert!((approx_neg + x.eval(assign)).abs() < 1e-9);
    }
}

/// **Soundness (affine `mul`).** The true product lies inside `[center ± radius]` of the composed
/// form for every noise assignment — the second-order remainder is soundly over-approximated.
#[test]
fn affine_mul_is_sound() {
    let mut rng = Lcg::new(2);
    for _ in 0..TRIALS {
        let x = AffineForm::uncertain(rng.unit() * 5.0, 0, rng.nonneg(3.0))
            .add(&AffineForm::uncertain(0.0, 1, rng.nonneg(2.0)));
        let y = AffineForm::uncertain(rng.unit() * 5.0, 0, rng.nonneg(3.0))
            .add(&AffineForm::uncertain(0.0, 2, rng.nonneg(2.0)));
        // Fresh symbol 9 not used by x or y.
        let prod = x.mul(&y, 9);
        let e0 = rng.unit();
        let e1 = rng.unit();
        let e2 = rng.unit();
        let assign = |s| match s {
            0 => e0,
            1 => e1,
            2 => e2,
            _ => 0.0,
        };
        let true_product = x.eval(assign) * y.eval(assign);
        assert!(
            (true_product - prod.center()).abs() <= prod.radius() + 1e-9,
            "mul unsound: |{true_product} - {}| > radius {}",
            prod.center(),
            prod.radius()
        );
    }
}

/// **Soundness (scalar `ErrorBound`).** The composed `eps` upper-bounds the true deviation of the
/// composed *values* for `add`/`sub`/`scale`/`mul` — sampled over worst-case-aligned deviations.
#[test]
fn error_bound_scalar_is_sound() {
    let mut rng = Lcg::new(3);
    for _ in 0..TRIALS {
        let ex = rng.nonneg(4.0);
        let ey = rng.nonneg(4.0);
        let bx = ErrorBound::new(ex, NormKind::Linf).unwrap();
        let by = ErrorBound::new(ey, NormKind::Linf).unwrap();
        // True deviations within the per-input bounds.
        let dx = rng.unit() * ex;
        let dy = rng.unit() * ey;

        // add: |dx + dy| <= eps_add
        assert!(dx + dy - 1e-9 <= bx.add(&by).unwrap().eps);
        // sub: |dx - dy| <= eps_sub
        assert!((dx - dy).abs() - 1e-9 <= bx.sub(&by).unwrap().eps);
        // scale
        let c = rng.unit() * 3.0;
        assert!((c * dx).abs() - 1e-9 <= bx.scale(c).eps);
        // mul about centers x0,y0: |(x0+dx)(y0+dy) - x0 y0| <= eps_mul
        let x0 = rng.unit() * 6.0;
        let y0 = rng.unit() * 6.0;
        let true_dev = ((x0 + dx) * (y0 + dy) - x0 * y0).abs();
        assert!(true_dev - 1e-6 <= bx.mul(&by, x0, y0).unwrap().eps);
    }
}

/// **Monotonicity.** Raising any input `eps` can only raise the composed `eps`.
#[test]
fn error_bound_is_monotone() {
    let mut rng = Lcg::new(4);
    for _ in 0..TRIALS {
        let ex = rng.nonneg(4.0);
        let ey = rng.nonneg(4.0);
        let bump = rng.nonneg(2.0);
        let lo = ErrorBound::new(ex, NormKind::L2).unwrap();
        let hi = ErrorBound::new(ex + bump, NormKind::L2).unwrap();
        let y = ErrorBound::new(ey, NormKind::L2).unwrap();
        assert!(hi.add(&y).unwrap().eps >= lo.add(&y).unwrap().eps);
        assert!(hi.mul(&y, 2.0, 3.0).unwrap().eps >= lo.mul(&y, 2.0, 3.0).unwrap().eps);
    }
}

/// **Determinism.** Identical inputs → identical composed `eps` (so composed bounds are
/// content-addressable).
#[test]
fn error_bound_is_deterministic() {
    let mut rng = Lcg::new(5);
    for _ in 0..TRIALS {
        let ex = rng.nonneg(4.0);
        let ey = rng.nonneg(4.0);
        let x = ErrorBound::new(ex, NormKind::Rel).unwrap();
        let y = ErrorBound::new(ey, NormKind::Rel).unwrap();
        assert_eq!(x.add(&y), x.add(&y));
        assert_eq!(x.mul(&y, 1.5, 2.5), x.mul(&y, 1.5, 2.5));
    }
}

/// Mixing norms is refused, never silently coerced (G2).
#[test]
fn error_bound_refuses_norm_mismatch() {
    let x = ErrorBound::new(1.0, NormKind::L1).unwrap();
    let y = ErrorBound::new(1.0, NormKind::L2).unwrap();
    assert!(x.add(&y).is_none());
    assert!(x.mul(&y, 1.0, 1.0).is_none());
}

// --- ProbBound ---------------------------------------------------------------------------------

/// **Soundness (union bound).** The union δ upper-bounds the empirical failure rate of independent
/// events with the given per-event probabilities; and it never exceeds 1.
#[test]
fn union_bound_is_sound() {
    let mut rng = Lcg::new(6);
    let deltas = [0.01, 0.02, 0.05];
    let bounds: Vec<ProbBound> = deltas.iter().map(|d| ProbBound::new(*d).unwrap()).collect();
    let claimed = ProbBound::union(&bounds);
    assert!(claimed.delta <= 1.0);
    let mut failures = 0u64;
    let n = 200_000u64;
    for _ in 0..n {
        let any = deltas
            .iter()
            .any(|d| (rng.next_u64() as f64 / u64::MAX as f64) < *d);
        if any {
            failures += 1;
        }
    }
    let empirical = failures as f64 / n as f64;
    // Union bound must over-estimate the true "any fails" probability.
    assert!(claimed.delta + 0.01 >= empirical, "union {} < emp {empirical}", claimed.delta);
}

/// **Monotonicity + saturation.** Adding a failure mode never lowers δ; δ saturates at 1.
#[test]
fn union_bound_is_monotone_and_saturates() {
    let a = ProbBound::new(0.4).unwrap();
    let b = ProbBound::new(0.4).unwrap();
    let c = ProbBound::new(0.9).unwrap();
    assert!(a.or(&b).delta >= a.delta);
    assert_eq!(a.or(&b).or(&c).delta, 1.0); // 0.4+0.4+0.9 -> clamp to 1
}

/// **Determinism.** Same δ inputs → same union; empty union is `certain`.
#[test]
fn union_bound_is_deterministic() {
    let xs: Vec<ProbBound> = [0.1, 0.2].iter().map(|d| ProbBound::new(*d).unwrap()).collect();
    assert_eq!(ProbBound::union(&xs), ProbBound::union(&xs));
    assert_eq!(ProbBound::union::<&[ProbBound]>(&[]), ProbBound::certain());
}

/// apRHL `[SEQ]`: ε adds (privacy factors `e^ε` multiply), δ adds and saturates (ADR-010 §2).
#[test]
fn aprhl_seq_composes() {
    let j1 = ApRhlJudgment::new(0.5, 0.01).unwrap();
    let j2 = ApRhlJudgment::new(0.3, 0.02).unwrap();
    let seq = j1.seq(&j2);
    assert!((seq.eps - 0.8).abs() < 1e-12);
    assert!((seq.delta - 0.03).abs() < 1e-12);
    // Saturation at δ = 1.
    let big = ApRhlJudgment::new(0.0, 0.7).unwrap();
    assert_eq!(big.seq(&big).delta, 1.0);
}

// --- tier-i checker ----------------------------------------------------------------------------

/// The checker accepts a sound (≥ re-derivation) claim and **rejects a too-tight** one — never a
/// silent pass (ADR-010 "Trusted base"; RFC-0002 §2).
#[test]
fn checker_rejects_too_tight_claims() {
    let x = ErrorBound::new(2.0, NormKind::Linf).unwrap();
    let y = ErrorBound::new(3.0, NormKind::Linf).unwrap();
    let inputs = [x, y];
    // Sound re-derivation of add = 5.0.
    let recomputed = recompute_error(&inputs, ErrorOp::Add).unwrap();
    assert!((recomputed.eps - 5.0).abs() < 1e-12);

    // Exact claim: valid.
    let exact_claim = ErrorBound::new(5.0, NormKind::Linf).unwrap();
    assert_eq!(check_error_claim(&inputs, ErrorOp::Add, exact_claim), CheckOutcome::Valid);
    // Looser claim: valid (sound, allowed).
    let loose = ErrorBound::new(7.0, NormKind::Linf).unwrap();
    assert_eq!(check_error_claim(&inputs, ErrorOp::Add, loose), CheckOutcome::Valid);
    // Too-tight claim: rejected.
    let tight = ErrorBound::new(4.0, NormKind::Linf).unwrap();
    assert!(matches!(
        check_error_claim(&inputs, ErrorOp::Add, tight),
        CheckOutcome::Rejected { .. }
    ));
    // Norm mismatch: malformed.
    let wrong_norm = ErrorBound::new(5.0, NormKind::L1).unwrap();
    assert_eq!(check_error_claim(&inputs, ErrorOp::Add, wrong_norm), CheckOutcome::Malformed);
}

/// The union checker likewise rejects a δ claim below `min(1, Σδ)`.
#[test]
fn union_checker_rejects_too_tight() {
    let inputs = [ProbBound::new(0.1).unwrap(), ProbBound::new(0.2).unwrap()];
    assert_eq!(
        check_union_claim(&inputs, ProbBound::new(0.3).unwrap()),
        CheckOutcome::Valid
    );
    assert!(matches!(
        check_union_claim(&inputs, ProbBound::new(0.2).unwrap()),
        CheckOutcome::Rejected { .. }
    ));
}

// --- cross-kernel + certificate ----------------------------------------------------------------

/// The single sanctioned cross-kernel rule: within tolerance ⇒ inherits the accuracy confidence;
/// outside ⇒ honest worst case δ = 1 (ADR-010 §4).
#[test]
fn accuracy_to_probability_is_honest() {
    let acc = ErrorBound::new(0.5, NormKind::L2).unwrap();
    // Within tolerance: failure prob = the accuracy bound's own confidence slack.
    assert_eq!(
        accuracy_to_probability(acc, 1.0, 0.03).unwrap(),
        ProbBound::new(0.03).unwrap()
    );
    // Exceeds tolerance: worst case.
    assert_eq!(
        accuracy_to_probability(acc, 0.25, 0.03).unwrap(),
        ProbBound::new(1.0).unwrap()
    );
    // Malformed tolerance.
    assert!(accuracy_to_probability(acc, -1.0, 0.0).is_none());
}

/// The shared certificate round-trips through its serialized form.
#[test]
fn certificate_round_trips() {
    let cert = Certificate::new(0.25, 0.01, GuaranteeStrength::Proven).unwrap();
    let json = serde_json::to_string(&cert).unwrap();
    let back: Certificate = serde_json::from_str(&json).unwrap();
    assert_eq!(cert, back);
    assert!(json.contains("\"strength\":\"Proven\""));
    // Out-of-range δ is refused.
    assert!(Certificate::new(0.0, 1.5, GuaranteeStrength::Declared).is_none());
}

// --- compose_error_bound (the M-204 entry) -----------------------------------------------------

fn error_bound(eps: f64, basis: BoundBasis) -> Bound {
    Bound {
        kind: BoundKind::Error {
            eps,
            norm: NormKind::Linf,
        },
        basis,
    }
}

/// Composing two `Proven` error bounds via `add` stays `Proven` (affine composition is itself sound)
/// with the composition citation, and `eps` is the kernel re-derivation.
#[test]
fn compose_keeps_proven_and_sums_eps() {
    let x = error_bound(
        1.0,
        BoundBasis::ProvenThm {
            citation: "thm-x".to_owned(),
        },
    );
    let y = error_bound(
        2.0,
        BoundBasis::ProvenThm {
            citation: "thm-y".to_owned(),
        },
    );
    let composed = compose_error_bound(&[&x, &y], ErrorOp::Add).unwrap();
    assert_eq!(composed.strength, GuaranteeStrength::Proven);
    match composed.bound.kind {
        BoundKind::Error { eps, .. } => assert!((eps - 3.0).abs() < 1e-12),
        _ => panic!("expected Error"),
    }
    assert!(matches!(composed.bound.basis, BoundBasis::ProvenThm { .. }));
}

/// The meet degrades the composed strength to the weakest input (VR-5): `Proven ⊕ Empirical →
/// Empirical`, carrying the fewest trials; `… ⊕ Declared → Declared`.
#[test]
fn compose_meets_strength_down() {
    let proven = error_bound(
        1.0,
        BoundBasis::ProvenThm {
            citation: "thm".to_owned(),
        },
    );
    let empirical = error_bound(
        2.0,
        BoundBasis::EmpiricalFit {
            trials: 10_000,
            method: "frady".to_owned(),
        },
    );
    let declared = error_bound(0.5, BoundBasis::UserDeclared);

    let pe = compose_error_bound(&[&proven, &empirical], ErrorOp::Add).unwrap();
    assert_eq!(pe.strength, GuaranteeStrength::Empirical);
    assert!(matches!(
        pe.bound.basis,
        BoundBasis::EmpiricalFit { trials: 10_000, .. }
    ));

    let pd = compose_error_bound(&[&proven, &declared], ErrorOp::Add).unwrap();
    assert_eq!(pd.strength, GuaranteeStrength::Declared);
    assert_eq!(pd.bound.basis, BoundBasis::UserDeclared);
}

/// A non-`Error` input bound has no error-composition rule → `None` (the caller refuses honestly,
/// never fabricates a bound).
#[test]
fn compose_refuses_non_error_bounds() {
    let capacity = Bound {
        kind: BoundKind::Capacity {
            items: 5,
            dim: 1000,
        },
        basis: BoundBasis::ProvenThm {
            citation: "cap".to_owned(),
        },
    };
    let err = error_bound(
        1.0,
        BoundBasis::ProvenThm {
            citation: "e".to_owned(),
        },
    );
    assert!(compose_error_bound(&[&capacity, &err], ErrorOp::Add).is_none());
    assert!(compose_error_bound(&[], ErrorOp::Add).is_none());
}
