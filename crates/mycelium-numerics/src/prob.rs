//! The **`ProbBound` (δ) kernel** — union bound + apRHL sequencing (M-202; ADR-010 §2; RFC-0001 §4.7).
//!
//! Failure-probability bounds compose through a *different monoid* than ε (ADR-010/T0.1c — a settled
//! negative result): the **union bound** `P(⋃ Eᵢ) ≤ Σ P(Eᵢ)` (saturating at 1), natural for "decode
//! succeeds w.p. ≥ 1−δ" and "P(any of N retrievals fails) ≤ Σδ". For *relational*
//! reference-vs-implementation certificates the **apRHL** `[SEQ]` rule composes `⟨ε, δ⟩` judgments —
//! multiplicatively in the privacy factor `e^ε` (so `ε` adds) and additively in `δ` (ADR-010 §2).
//!
//! Both compositions are **Sound** (never under-state the true failure probability), **Monotone**
//! (each input can only raise `δ`), and **Deterministic**. `δ` is always clamped to `[0, 1]` — a
//! probability is never `> 1`, and that clamp is itself a sound over-approximation.

/// A scalar failure-probability bound `δ ∈ [0, 1]` — travels in a [`mycelium_core::Bound`]
/// (`BoundKind::Probability`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProbBound {
    /// Failure probability, always in `[0, 1]`.
    pub delta: f64,
}

impl ProbBound {
    /// The certain bound (`δ == 0`, never fails) — the identity of [`union`](Self::union).
    #[must_use]
    pub const fn certain() -> Self {
        ProbBound { delta: 0.0 }
    }

    /// A well-formed bound, or `None` if `delta ∉ [0, 1]` or is non-finite (never silent).
    #[must_use]
    pub fn new(delta: f64) -> Option<Self> {
        (delta.is_finite() && (0.0..=1.0).contains(&delta)).then_some(ProbBound { delta })
    }

    /// The **union bound**: `P(⋃ Eᵢ) ≤ min(1, Σ δᵢ)` (ADR-010 §2). Saturates at 1 (a sound
    /// over-approximation — probabilities never exceed 1). Empty input ⇒ [`certain`](Self::certain).
    #[must_use]
    pub fn union<'a, I>(bounds: I) -> Self
    where
        I: IntoIterator<Item = &'a ProbBound>,
    {
        let sum: f64 = bounds.into_iter().map(|b| b.delta).sum();
        ProbBound {
            delta: sum.min(1.0),
        }
    }

    /// Combine with another failure mode by the union bound — the binary form of [`union`](Self::union).
    #[must_use]
    pub fn or(&self, other: &ProbBound) -> Self {
        ProbBound::union([self, other])
    }
}

/// An apRHL `⟨ε, δ⟩` relational judgment (ADR-010 §2): "the implementation refines the reference up
/// to multiplicative privacy factor `e^ε` and additive slack `δ`". Used for reference-vs-implementation
/// certificates (the relational path), distinct from the scalar [`ProbBound`] union path.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ApRhlJudgment {
    /// The log privacy factor `ε ≥ 0` (the factor is `e^ε`).
    pub eps: f64,
    /// The additive slack `δ ∈ [0, 1]`.
    pub delta: f64,
}

impl ApRhlJudgment {
    /// A well-formed judgment, or `None` on a negative/non-finite `ε` or `δ ∉ [0, 1]`.
    #[must_use]
    pub fn new(eps: f64, delta: f64) -> Option<Self> {
        let ok = eps.is_finite() && eps >= 0.0 && delta.is_finite() && (0.0..=1.0).contains(&delta);
        ok.then_some(ApRhlJudgment { eps, delta })
    }

    /// The apRHL **`[SEQ]`** rule: sequencing two relational steps composes **multiplicatively in the
    /// privacy factor** `e^ε` (so `ε` adds: `e^{ε₁}·e^{ε₂} = e^{ε₁+ε₂}`) and **additively in `δ`**
    /// (clamped to 1) — ADR-010 §2 / Barthe et al. apRHL. Sound and monotone in both components.
    #[must_use]
    pub fn seq(&self, next: &ApRhlJudgment) -> Self {
        ApRhlJudgment {
            eps: self.eps + next.eps,
            delta: (self.delta + next.delta).min(1.0),
        }
    }
}
