//! Runtime metadata that travels with every value (RFC-0001 §4.3; `meta.schema.json`).
//!
//! [`Meta`] enforces the schema invariants **M-I1…M-I4** *by construction* — a [`Meta`] cannot be
//! built with an inconsistent guarantee/bound pairing (the honesty rule, mechanically).

use crate::bound::{Bound, BoundBasis};
use crate::id::ContentHash;
use crate::{GuaranteeStrength, WfError};

/// Provenance: an acyclic derivation DAG (RFC-0001 §4.6). Not part of code identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Provenance {
    /// A primitive/constant origin.
    Root,
    /// Derived from inputs by the operation at content hash `op`.
    Derived {
        /// Content hash of the producing operation/definition.
        op: ContentHash,
        /// Content hashes of the inputs.
        inputs: Vec<ContentHash>,
    },
}

/// Measured (dynamic) sparsity — distinct from the declared [`crate::repr::SparsityClass`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SparsityObs {
    /// Number of active components.
    pub active: u64,
    /// Density in `[0, 1]`.
    pub density: f64,
}

/// Lossless physical packing schemes (extensible registry; RFC-0001 §4.3; DN-01).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackScheme {
    /// Unpacked.
    Unpacked,
    /// Two bits per trit.
    TwoBitPerTrit,
    /// Five trits per byte.
    FiveTritPerByte,
    /// bitnet.cpp I2_S.
    I2S,
    /// bitnet.cpp TL1.
    Tl1,
    /// bitnet.cpp TL2.
    Tl2,
}

/// The recorded schedule-staged packing (RFC-0001 §4.3; RFC-0004 §5). A *record*, not the decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalLayout {
    /// Binary words.
    BinaryWords,
    /// Packed trits under `scheme`.
    TritPacked {
        /// The packing scheme used.
        scheme: PackScheme,
    },
    /// Dense scalar array.
    DenseArray,
    /// VSA storage (sparse or dense).
    VsaStore {
        /// Whether storage is sparse.
        sparse: bool,
    },
}

/// Runtime, queryable metadata (RFC-0001 §4.3). Fields are private; the only way to build a `Meta`
/// is [`Meta::new`], which enforces M-I1…M-I4 — so an inconsistent `Meta` is unrepresentable.
///
/// `reconstruction` (RFC-0003 §6) is deferred to the VSA work (M-130 / E2-5) and intentionally
/// omitted here.
#[derive(Debug, Clone, PartialEq)]
pub struct Meta {
    provenance: Provenance,
    guarantee: GuaranteeStrength,
    bound: Option<Bound>,
    sparsity: Option<SparsityObs>,
    physical: Option<PhysicalLayout>,
    policy_used: Option<ContentHash>,
}

impl Meta {
    /// Build a `Meta`, enforcing the guarantee/bound invariants:
    ///
    /// - **M-I1** `guarantee == Exact ⟺ bound == None`,
    /// - **M-I2** `Proven ⟹ basis ProvenThm`, **M-I3** `Empirical ⟹ EmpiricalFit`,
    ///   **M-I4** `Declared ⟹ UserDeclared`,
    ///
    /// plus numeric well-formedness of any bound. Returns [`WfError`] on violation.
    pub fn new(
        provenance: Provenance,
        guarantee: GuaranteeStrength,
        bound: Option<Bound>,
        sparsity: Option<SparsityObs>,
        physical: Option<PhysicalLayout>,
        policy_used: Option<ContentHash>,
    ) -> Result<Self, WfError> {
        check_guarantee_bound(guarantee, bound.as_ref())?;
        if let Some(b) = &bound {
            if !b.well_formed() {
                return Err(WfError::MalformedBound);
            }
        }
        if let Some(s) = &sparsity {
            if !(0.0..=1.0).contains(&s.density) {
                return Err(WfError::MalformedBound);
            }
        }
        Ok(Meta {
            provenance,
            guarantee,
            bound,
            sparsity,
            physical,
            policy_used,
        })
    }

    /// The common `Exact` metadata with no bound (M-I1).
    #[must_use]
    pub fn exact(provenance: Provenance) -> Self {
        Meta {
            provenance,
            guarantee: GuaranteeStrength::Exact,
            bound: None,
            sparsity: None,
            physical: None,
            policy_used: None,
        }
    }

    /// The value's provenance.
    #[must_use]
    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }
    /// The disclosed guarantee strength.
    #[must_use]
    pub fn guarantee(&self) -> GuaranteeStrength {
        self.guarantee
    }
    /// The bound, if approximate.
    #[must_use]
    pub fn bound(&self) -> Option<&Bound> {
        self.bound.as_ref()
    }
    /// Measured sparsity, if recorded.
    #[must_use]
    pub fn sparsity(&self) -> Option<SparsityObs> {
        self.sparsity
    }
    /// The recorded physical layout, if any.
    #[must_use]
    pub fn physical(&self) -> Option<PhysicalLayout> {
        self.physical
    }
    /// The policy that produced this value (set iff produced by a swap).
    #[must_use]
    pub fn policy_used(&self) -> Option<&ContentHash> {
        self.policy_used.as_ref()
    }
}

/// The M-I1…M-I4 guarantee/bound consistency check.
fn check_guarantee_bound(g: GuaranteeStrength, bound: Option<&Bound>) -> Result<(), WfError> {
    use GuaranteeStrength::{Declared, Empirical, Exact, Proven};
    let basis_ok =
        |b: Option<&Bound>, want_proven: bool, want_empirical: bool| match b.map(|b| &b.basis) {
            Some(BoundBasis::ProvenThm { .. }) => want_proven,
            Some(BoundBasis::EmpiricalFit { .. }) => want_empirical,
            Some(BoundBasis::UserDeclared) => !want_proven && !want_empirical,
            None => false,
        };
    match g {
        Exact => {
            if bound.is_none() {
                Ok(())
            } else {
                Err(WfError::GuaranteeBoundMismatch) // M-I1
            }
        }
        Proven if basis_ok(bound, true, false) => Ok(()), // M-I2
        Empirical if basis_ok(bound, false, true) => Ok(()), // M-I3
        Declared if basis_ok(bound, false, false) => Ok(()), // M-I4
        _ => Err(WfError::GuaranteeBoundMismatch),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bound::{BoundKind, NormKind};

    fn proven_capacity() -> Bound {
        Bound {
            kind: BoundKind::Capacity {
                items: 3,
                dim: 10_000,
            },
            basis: BoundBasis::ProvenThm {
                citation: "Clarkson-Ubaru-Yang 2023".to_owned(),
            },
        }
    }

    #[test]
    fn exact_without_bound_is_ok() {
        assert!(Meta::new(
            Provenance::Root,
            GuaranteeStrength::Exact,
            None,
            None,
            None,
            None
        )
        .is_ok());
    }

    #[test]
    fn exact_with_bound_violates_m_i1() {
        let m = Meta::new(
            Provenance::Root,
            GuaranteeStrength::Exact,
            Some(proven_capacity()),
            None,
            None,
            None,
        );
        assert_eq!(m.unwrap_err(), WfError::GuaranteeBoundMismatch);
    }

    #[test]
    fn proven_requires_proven_basis() {
        // Proven + ProvenThm: ok (M-I2).
        assert!(Meta::new(
            Provenance::Root,
            GuaranteeStrength::Proven,
            Some(proven_capacity()),
            None,
            None,
            None,
        )
        .is_ok());
        // Declared cannot claim a ProvenThm basis (M-I4).
        let bad = Meta::new(
            Provenance::Root,
            GuaranteeStrength::Declared,
            Some(proven_capacity()),
            None,
            None,
            None,
        );
        assert_eq!(bad.unwrap_err(), WfError::GuaranteeBoundMismatch);
    }

    #[test]
    fn non_exact_requires_a_bound() {
        let m = Meta::new(
            Provenance::Root,
            GuaranteeStrength::Proven,
            None,
            None,
            None,
            None,
        );
        assert_eq!(m.unwrap_err(), WfError::GuaranteeBoundMismatch);
    }

    #[test]
    fn out_of_range_bound_is_malformed() {
        let b = Bound {
            kind: BoundKind::Probability { delta: 1.5 },
            basis: BoundBasis::UserDeclared,
        };
        let m = Meta::new(
            Provenance::Root,
            GuaranteeStrength::Declared,
            Some(b),
            None,
            None,
            None,
        );
        assert_eq!(m.unwrap_err(), WfError::MalformedBound);
    }

    #[test]
    fn error_bound_uses_norm() {
        let b = Bound {
            kind: BoundKind::Error {
                eps: 0.004,
                norm: NormKind::L2,
            },
            basis: BoundBasis::EmpiricalFit {
                trials: 10_000,
                method: "Frady-Sommer Gaussian".to_owned(),
            },
        };
        assert!(Meta::new(
            Provenance::Root,
            GuaranteeStrength::Empirical,
            Some(b),
            None,
            None,
            None,
        )
        .is_ok());
    }
}
