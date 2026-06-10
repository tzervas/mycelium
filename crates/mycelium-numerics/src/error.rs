//! The **`ErrorBound` (ε) kernel** — affine arithmetic (M-201; ADR-010 §1; RFC-0001 §4.7).
//!
//! ε-magnitude bounds compose through **affine arithmetic** (the ADR-010 §1 choice — sound,
//! compositional, *correlation-aware*; the Daisy/Rosa + FloVer lineage). The semantic domain is the
//! [`AffineForm`] `x̂ = x₀ + Σ xᵢ·εᵢ` over noise symbols `εᵢ ∈ [−1, +1]`; its **radius** `Σ|xᵢ|` is the
//! sound ε on the deviation from the central value. Linear ops (`add`/`sub`/`neg`/`scale`) are
//! *exact* on the form (shared noise symbols cancel — the correlation awareness); `mul` is nonlinear,
//! so its second-order remainder is over-approximated onto a fresh noise symbol.
//!
//! The scalar [`ErrorBound`] `{eps, norm}` is the projection that travels in a [`mycelium_core::Bound`]
//! (`BoundKind::Error`). Its compositions are the *conservative* (worst-case, correlation-free)
//! projections used when only the magnitudes — not the affine structure — are available (the
//! interpreter's case, M-204). All three normative composition properties hold: **Soundness**
//! (the composed `eps` is a true upper bound on the deviation), **Monotonicity** (never tighter than
//! the inputs justify), **Determinism** (identical inputs → identical `eps`).

use std::collections::BTreeMap;

use mycelium_core::NormKind;

/// A noise-symbol identifier. Distinct symbols model *independent* uncertainty sources; a shared
/// symbol models a *correlated* one (the affine-arithmetic advantage over interval arithmetic).
pub type NoiseSym = u64;

/// An affine form `x₀ + Σ xᵢ·εᵢ` with noise symbols `εᵢ ∈ [−1, +1]` (ADR-010 §1). The
/// **concretization** is the interval `[x₀ − radius, x₀ + radius]` with `radius = Σ|xᵢ|`; linear
/// operations are *exact* functions of the shared assignment, so correlated uncertainty cancels.
#[derive(Debug, Clone, PartialEq)]
pub struct AffineForm {
    center: f64,
    /// Noise terms `symbol → coefficient`. Zero coefficients are never stored, so equality is the
    /// mathematical equality of forms.
    terms: BTreeMap<NoiseSym, f64>,
}

impl AffineForm {
    /// The exact constant `c` (no uncertainty; `radius == 0`).
    #[must_use]
    pub fn constant(center: f64) -> Self {
        AffineForm {
            center,
            terms: BTreeMap::new(),
        }
    }

    /// `center ± |radius|` carried on a single noise symbol `sym`. A non-positive or non-finite
    /// `radius` collapses to the [`constant`](Self::constant) (no uncertainty term stored).
    #[must_use]
    pub fn uncertain(center: f64, sym: NoiseSym, radius: f64) -> Self {
        let mut terms = BTreeMap::new();
        if radius.is_finite() && radius > 0.0 {
            terms.insert(sym, radius);
        }
        AffineForm { center, terms }
    }

    /// The central value `x₀`.
    #[must_use]
    pub fn center(&self) -> f64 {
        self.center
    }

    /// The total deviation `radius = Σ|xᵢ|` — the sound ε on `|value − center|` (ADR-010 §1).
    #[must_use]
    pub fn radius(&self) -> f64 {
        self.terms.values().map(|c| c.abs()).sum()
    }

    /// Evaluate the form at a noise assignment `ε(sym) ∈ [−1, +1]`. Used to *check* soundness: for a
    /// linear op this equals the corresponding real operation exactly (the soundness property test).
    #[must_use]
    pub fn eval(&self, assign: impl Fn(NoiseSym) -> f64) -> f64 {
        self.center
            + self
                .terms
                .iter()
                .map(|(sym, coeff)| coeff * assign(*sym))
                .sum::<f64>()
    }

    /// Insert/accumulate a term, dropping it if the coefficient lands on exactly zero (keeps forms
    /// canonical, so structurally-equal forms are `==`).
    fn accumulate(terms: &mut BTreeMap<NoiseSym, f64>, sym: NoiseSym, coeff: f64) {
        if coeff == 0.0 {
            return;
        }
        let entry = terms.entry(sym).or_insert(0.0);
        *entry += coeff;
        if *entry == 0.0 {
            terms.remove(&sym);
        }
    }

    /// Negation — exact (`−x̂`).
    #[must_use]
    pub fn neg(&self) -> Self {
        AffineForm {
            center: -self.center,
            terms: self.terms.iter().map(|(s, c)| (*s, -c)).collect(),
        }
    }

    /// Addition — *exact* on the form: shared noise symbols combine (correlated uncertainty cancels),
    /// independent ones accumulate.
    #[must_use]
    pub fn add(&self, other: &AffineForm) -> Self {
        let mut terms = self.terms.clone();
        for (sym, coeff) in &other.terms {
            Self::accumulate(&mut terms, *sym, *coeff);
        }
        AffineForm {
            center: self.center + other.center,
            terms,
        }
    }

    /// Subtraction — exact (`x̂ − ŷ`); `x̂ − x̂ == 0` with `radius 0` (the correlation advantage).
    #[must_use]
    pub fn sub(&self, other: &AffineForm) -> Self {
        self.add(&other.neg())
    }

    /// Scaling by an exact constant — exact (`c·x̂`).
    #[must_use]
    pub fn scale(&self, c: f64) -> Self {
        AffineForm {
            center: c * self.center,
            terms: self
                .terms
                .iter()
                .map(|(s, coeff)| (*s, c * coeff))
                .collect(),
        }
    }

    /// Multiplication — *nonlinear*. The linear part `x₀ŷ + y₀x̂` is kept exactly; the second-order
    /// remainder (magnitude `≤ radius(x)·radius(y)`) is over-approximated onto a **fresh** noise
    /// symbol `fresh` (standard affine multiplication, ADR-010 §1). Sound: the true product lies in
    /// `[center ± radius]` for every noise assignment. `fresh` must not already appear in either
    /// operand.
    #[must_use]
    pub fn mul(&self, other: &AffineForm, fresh: NoiseSym) -> Self {
        let mut terms: BTreeMap<NoiseSym, f64> = BTreeMap::new();
        // Linear part: x0·(other terms) + y0·(self terms).
        for (sym, coeff) in &other.terms {
            Self::accumulate(&mut terms, *sym, self.center * coeff);
        }
        for (sym, coeff) in &self.terms {
            Self::accumulate(&mut terms, *sym, other.center * coeff);
        }
        // Second-order remainder onto the fresh symbol.
        let remainder = self.radius() * other.radius();
        Self::accumulate(&mut terms, fresh, remainder);
        AffineForm {
            center: self.center * other.center,
            terms,
        }
    }
}

/// A scalar ε-magnitude bound `{eps ≥ 0, norm}` — the [`AffineForm::radius`] projection that travels
/// in a [`mycelium_core::Bound`] (`BoundKind::Error`). Compositions are the conservative (worst-case)
/// projections used when only magnitudes are available.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ErrorBound {
    /// Error magnitude (`>= 0`, finite).
    pub eps: f64,
    /// The norm `eps` is measured in.
    pub norm: NormKind,
}

impl ErrorBound {
    /// An exact bound (`eps == 0`) in `norm` — the identity of [`add`](Self::add).
    #[must_use]
    pub const fn exact(norm: NormKind) -> Self {
        ErrorBound { eps: 0.0, norm }
    }

    /// A well-formed bound, or `None` if `eps` is negative or non-finite (never a silent coercion).
    #[must_use]
    pub fn new(eps: f64, norm: NormKind) -> Option<Self> {
        (eps.is_finite() && eps >= 0.0).then_some(ErrorBound { eps, norm })
    }

    /// `eps(x ± y) = eps(x) + eps(y)` — the affine-`add` projection (1-Lipschitz; ℓ-norm triangle
    /// inequality). Sound and monotone. `None` if the norms differ (no silent norm coercion).
    #[must_use]
    pub fn add(&self, other: &ErrorBound) -> Option<Self> {
        (self.norm == other.norm).then_some(ErrorBound {
            eps: self.eps + other.eps,
            norm: self.norm,
        })
    }

    /// `eps(x − y) = eps(x) + eps(y)` — identical magnitude composition to [`add`](Self::add).
    #[must_use]
    pub fn sub(&self, other: &ErrorBound) -> Option<Self> {
        self.add(other)
    }

    /// `eps(−x) = eps(x)` — negation does not change the magnitude.
    #[must_use]
    pub fn neg(&self) -> Self {
        *self
    }

    /// `eps(c·x) = |c|·eps(x)` — exact scaling.
    #[must_use]
    pub fn scale(&self, c: f64) -> Self {
        ErrorBound {
            eps: c.abs() * self.eps,
            norm: self.norm,
        }
    }

    /// `eps(x·y) ≤ |x₀|·eps(y) + |y₀|·eps(x) + eps(x)·eps(y)` — sound first-order error propagation
    /// for a product about central magnitudes `x0_mag = |x₀|`, `y0_mag = |y₀|`. `None` if the norms
    /// differ.
    #[must_use]
    pub fn mul(&self, other: &ErrorBound, x0_mag: f64, y0_mag: f64) -> Option<Self> {
        (self.norm == other.norm).then_some(ErrorBound {
            eps: x0_mag.abs() * other.eps + y0_mag.abs() * self.eps + self.eps * other.eps,
            norm: self.norm,
        })
    }
}
