//! Bounds and their basis (RFC-0001 §4.3 r2; ADR-010; ADR-011; `bound.schema.json`).
//!
//! Per **ADR-011**, `basis` is a required companion of *every* [`Bound`], not just capacity bounds:
//! the guarantee strength derives from the basis for all bound kinds.

/// How a bound was obtained — this determines the honest [`crate::GuaranteeStrength`].
#[derive(Debug, Clone, PartialEq)]
pub enum BoundBasis {
    /// A cited theorem whose side-conditions are checked (e.g. "Clarkson-Ubaru-Yang 2023").
    ProvenThm {
        /// The citation.
        citation: String,
    },
    /// An empirical fit over `trials` (e.g. method "Frady-Sommer Gaussian").
    EmpiricalFit {
        /// Number of trials.
        trials: u64,
        /// Fitting method.
        method: String,
    },
    /// A user assertion, not yet validated. Tooling must surface a "declared, unverified" marker.
    UserDeclared,
}

/// Norm in which an [`BoundKind::Error`] `eps` is expressed (extensible registry; RFC-0001 §4.3 r2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormKind {
    /// ℓ¹.
    L1,
    /// ℓ².
    L2,
    /// ℓ∞.
    Linf,
    /// Relative error.
    Rel,
}

/// The bound payload, per kind (RFC-0001 §4.3).
#[derive(Debug, Clone, PartialEq)]
pub enum BoundKind {
    /// ε-magnitude bound (composes via ADR-010's affine-arithmetic kernel).
    Error {
        /// Error magnitude (`>= 0`).
        eps: f64,
        /// Norm in which `eps` is measured.
        norm: NormKind,
    },
    /// Failure-probability bound (composes via the union-bound kernel).
    Probability {
        /// Failure probability in `[0, 1]`.
        delta: f64,
    },
    /// Expected crosstalk with an optional tail.
    Crosstalk {
        /// Expected crosstalk (`>= 0`).
        expected: f64,
        /// Optional tail bound.
        tail: Option<f64>,
    },
    /// VSA superposition capacity (`items` into `dim`).
    Capacity {
        /// Number of superposed items (`>= 1`).
        items: u64,
        /// Hypervector dimension (`>= 1`).
        dim: u64,
    },
}

/// A sound bound plus the basis by which it was obtained (ADR-011: `basis` is universal).
#[derive(Debug, Clone, PartialEq)]
pub struct Bound {
    /// The kind-specific payload.
    pub kind: BoundKind,
    /// How the bound was obtained.
    pub basis: BoundBasis,
}

impl Bound {
    /// Numeric well-formedness of the payload (ranges per `bound.schema.json`). Independent of the
    /// guarantee↔basis coupling, which [`crate::meta::Meta`] enforces.
    #[must_use]
    pub fn well_formed(&self) -> bool {
        match self.kind {
            BoundKind::Error { eps, .. } => eps >= 0.0,
            BoundKind::Probability { delta } => (0.0..=1.0).contains(&delta),
            BoundKind::Crosstalk { expected, tail } => {
                expected >= 0.0 && tail.is_none_or(|t| t >= 0.0)
            }
            BoundKind::Capacity { items, dim } => items >= 1 && dim >= 1,
        }
    }
}
