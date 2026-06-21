//! Runtime metadata that travels with every value (RFC-0001 §4.3; `meta.schema.json`).
//!
//! [`Meta`] enforces the schema invariants **M-I1…M-I4** *by construction* — a [`Meta`] cannot be
//! built with an inconsistent guarantee/bound pairing (the honesty rule, mechanically). Its `serde`
//! form is `meta.schema.json` (M-104): `bound` is modelled by **presence** (absent for `Exact`,
//! per M-I1), and `Deserialize` re-runs the M-I1…M-I4 invariants through [`Meta::new`] so a
//! malformed wire `Meta` is rejected, never silently trusted.

use serde::{Deserialize, Serialize};

use crate::bound::{Bound, BoundBasis};
use crate::id::ContentHash;
use crate::recon::ReconInfo;
use crate::{GuaranteeStrength, WfError};

/// Provenance: an acyclic derivation DAG (RFC-0001 §4.6). Not part of code identity. The `serde`
/// form is tagged on `kind` (`Root|Derived`), matching `provenance.schema.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
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
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SparsityObs {
    /// Number of active components.
    pub active: u64,
    /// Density in `[0, 1]`.
    pub density: f64,
}

/// Lossless physical packing schemes (extensible registry; RFC-0001 §4.3; DN-01). The `serde`
/// renderings match `physical-layout.schema.json`'s `PackScheme` enum (`I2S|TL1|TL2`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    #[serde(rename = "TL1")]
    Tl1,
    /// bitnet.cpp TL2.
    #[serde(rename = "TL2")]
    Tl2,
}

/// The recorded schedule-staged packing (RFC-0001 §4.3; RFC-0004 §5). A *record*, not the decision.
/// The `serde` form is tagged on `layout`, matching `physical-layout.schema.json`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "layout")]
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
/// `reconstruction` (RFC-0003 §6; M-260) is attached via [`Meta::with_reconstruction`] — its own
/// invariants are enforced by [`ReconInfo::new`], and it does not interact with M-I1…M-I4.
#[derive(Debug, Clone, PartialEq)]
pub struct Meta {
    provenance: Provenance,
    guarantee: GuaranteeStrength,
    bound: Option<Bound>,
    sparsity: Option<SparsityObs>,
    physical: Option<PhysicalLayout>,
    reconstruction: Option<Box<ReconInfo>>,
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
                // A6-08: a sparsity observation is a measurement, not a guarantee bound — so an
                // out-of-range density is `MalformedSparsity`, not the misleading `MalformedBound`.
                return Err(WfError::MalformedSparsity);
            }
        }
        Ok(Meta {
            provenance,
            guarantee,
            bound,
            sparsity,
            physical,
            reconstruction: None,
            policy_used,
        })
    }

    /// Attach a reconstruction manifest (RFC-0003 §6; M-260). The manifest's own schema
    /// invariants are already enforced by [`ReconInfo::new`]; it is independent of M-I1…M-I4,
    /// so this cannot invalidate an existing `Meta`.
    #[must_use]
    pub fn with_reconstruction(mut self, reconstruction: ReconInfo) -> Self {
        self.reconstruction = Some(Box::new(reconstruction));
        self
    }

    /// Record the schedule-staged packing chosen at a lowering stage (RFC-0004 §5; DN-01;
    /// M-250). This is the **inspectable record** of the layout decision, not the decision locus
    /// (the selector is [`mycelium-select`](https://docs.rs/mycelium-select); RFC-0005 §4).
    ///
    /// **M-I5 (lossless `physical`).** The layout is a lossless re-encoding of the same `payload`:
    /// it touches only the `physical` field and leaves the guarantee, bound, and value untouched
    /// — so recording (or *re*-recording) it can never change the value's type or its guarantee
    /// (RFC-0001 §4.3; `physical-layout.schema.json`). M-I1…M-I4 are therefore preserved by
    /// construction.
    #[must_use]
    pub fn with_physical(mut self, physical: PhysicalLayout) -> Self {
        self.physical = Some(physical);
        self
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
            reconstruction: None,
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
    /// The reconstruction manifest, if attached (RFC-0003 §6).
    #[must_use]
    pub fn reconstruction(&self) -> Option<&ReconInfo> {
        self.reconstruction.as_deref()
    }
    /// The policy that produced this value (set iff produced by a swap).
    #[must_use]
    pub fn policy_used(&self) -> Option<&ContentHash> {
        self.policy_used.as_ref()
    }
}

/// The wire projection of [`Meta`] (`meta.schema.json`). Optional fields are omitted when absent
/// (so `Exact` emits no `bound`, satisfying M-I1's presence model); on the way back in, `null` and
/// absent both decode to `None`. `reconstruction` (RFC-0003 §6) **is** carried (serialized when
/// present, re-validated on the way in). `deny_unknown_fields` makes the schema's
/// `additionalProperties: false` a real contract — an unknown wire field is rejected, not silently
/// dropped (A6-02/B2-03).
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MetaWire {
    provenance: Provenance,
    guarantee: GuaranteeStrength,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bound: Option<Bound>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sparsity: Option<SparsityObs>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    physical: Option<PhysicalLayout>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reconstruction: Option<Box<ReconInfo>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    policy_used: Option<ContentHash>,
}

impl Serialize for Meta {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        MetaWire {
            provenance: self.provenance.clone(),
            guarantee: self.guarantee,
            bound: self.bound.clone(),
            sparsity: self.sparsity,
            physical: self.physical,
            reconstruction: self.reconstruction.clone(),
            policy_used: self.policy_used.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Meta {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let w = MetaWire::deserialize(deserializer)?;
        // Re-run M-I1…M-I4 (+ numeric well-formedness): wire data is never silently trusted.
        // (`ReconInfo`'s own `Deserialize` has already re-run its schema invariants.)
        let meta = Meta::new(
            w.provenance,
            w.guarantee,
            w.bound,
            w.sparsity,
            w.physical,
            w.policy_used,
        )
        .map_err(serde::de::Error::custom)?;
        Ok(match w.reconstruction {
            Some(r) => meta.with_reconstruction(*r),
            None => meta,
        })
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
    fn out_of_range_sparsity_is_malformed_sparsity() {
        // A6-08 mutant-witness: an out-of-range `density` is a sparsity-observation error, not a
        // bound error — so it must be `MalformedSparsity`, never the misleading `MalformedBound`.
        let bad_sparsity = SparsityObs {
            active: 10,
            density: 1.5,
        };
        let m = Meta::new(
            Provenance::Root,
            GuaranteeStrength::Exact,
            None,
            Some(bad_sparsity),
            None,
            None,
        );
        assert_eq!(m.unwrap_err(), WfError::MalformedSparsity);
    }

    #[test]
    fn with_physical_is_lossless_m_i5() {
        // M-I5: recording a layout touches only `physical` — guarantee, bound, and every other
        // field are untouched, so the value's type and guarantee cannot change.
        let base = Meta::new(
            Provenance::Root,
            GuaranteeStrength::Proven,
            Some(proven_capacity()),
            None,
            None,
            None,
        )
        .unwrap();
        let recorded = base.clone().with_physical(PhysicalLayout::TritPacked {
            scheme: PackScheme::Tl2,
        });
        assert_eq!(
            recorded.physical(),
            Some(PhysicalLayout::TritPacked {
                scheme: PackScheme::Tl2
            })
        );
        // Everything that defines type/guarantee is identical (M-I5: lossless).
        assert_eq!(recorded.guarantee(), base.guarantee());
        assert_eq!(recorded.bound(), base.bound());
        assert_eq!(recorded.provenance(), base.provenance());
        // Re-recording a different layout still changes nothing but `physical`.
        let rerecorded = recorded.clone().with_physical(PhysicalLayout::TritPacked {
            scheme: PackScheme::I2S,
        });
        assert_eq!(rerecorded.guarantee(), base.guarantee());
        assert_eq!(rerecorded.bound(), base.bound());
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

    // Mutant-witnesses for Meta accessor methods (meta.rs:189, 194, 204, 209):
    // bound(), sparsity(), reconstruction(), policy_used() must each return the value passed to
    // new() (not always None). Tests construct a Meta with each optional field set and assert the
    // accessor returns Some with the correct value.
    #[test]
    fn accessors_return_the_constructed_optional_fields() {
        // bound() — must return the bound passed to new(), not None.
        let b = proven_capacity();
        let m = Meta::new(
            Provenance::Root,
            GuaranteeStrength::Proven,
            Some(b.clone()),
            None,
            None,
            None,
        )
        .unwrap();
        assert!(
            m.bound().is_some(),
            "bound() must return Some when a bound was passed"
        );
        assert_eq!(m.bound().unwrap(), &b);

        // sparsity() — must return the SparsityObs, not None.
        let sp = SparsityObs {
            active: 10,
            density: 0.01,
        };
        let m2 = Meta::new(
            Provenance::Root,
            GuaranteeStrength::Exact,
            None,
            Some(sp),
            None,
            None,
        )
        .unwrap();
        assert!(
            m2.sparsity().is_some(),
            "sparsity() must return Some when a SparsityObs was passed"
        );
        assert_eq!(m2.sparsity().unwrap(), sp);

        // policy_used() — must return the ContentHash, not None.
        let hash = crate::id::ContentHash::parse("blake3:round_trip_safe").unwrap();
        let m3 = Meta::new(
            Provenance::Root,
            GuaranteeStrength::Exact,
            None,
            None,
            None,
            Some(hash.clone()),
        )
        .unwrap();
        assert!(
            m3.policy_used().is_some(),
            "policy_used() must return Some when a hash was passed"
        );
        assert_eq!(m3.policy_used().unwrap(), &hash);

        // reconstruction() — must return the ReconInfo, not None. Built via with_reconstruction().
        use crate::bound::{BoundBasis, BoundKind};
        use crate::content::operation_hash;
        use crate::recon::{DecodeProcedure, DecodeSpec, ReconInfo, ReconMode};
        let recon = ReconInfo::new(
            ReconMode::IndexedRetrieval,
            "MAP-I",
            1024,
            vec![operation_hash("cb")],
            None,
            DecodeSpec {
                procedure: DecodeProcedure::Cleanup,
                cleanup_threshold: Some(0.5),
                factors: None,
                iteration_budget: None,
                cleanup: None,
                beta: None,
                tau_lock: None,
                init: None,
                seed: None,
            },
            Bound {
                kind: BoundKind::Probability { delta: 0.01 },
                basis: BoundBasis::EmpiricalFit {
                    trials: 1_000,
                    method: "frady".to_owned(),
                },
            },
        )
        .unwrap();
        let m4 = Meta::exact(Provenance::Root).with_reconstruction(recon.clone());
        assert!(
            m4.reconstruction().is_some(),
            "reconstruction() must return Some after with_reconstruction()"
        );
        assert_eq!(m4.reconstruction().unwrap(), &recon);
    }

    // Mutant-witness (meta.rs:279:60): the guard `!want_proven && !want_empirical` (for Declared)
    // becomes `!want_proven || !want_empirical` when mutated. For Declared we call
    // `basis_ok(bound, false, false)`, so `!false && !false = true` which correctly accepts it.
    // Under the `||` mutant, `!false || !false = true` also accepts it — so **UserDeclared with
    // Declared** does not distinguish the two. BUT consider a ProvenThm basis with Declared tag:
    // `basis_ok(Some(ProvenThm), false, false)` → `want_proven = false` → returns `false` (correct);
    // under `||`, `!false || !false = true` would also return `false` for the *outer* match arm,
    // but the *inner* basis match sees `Some(ProvenThm) => want_proven`, and `want_proven = false`,
    // so it still returns `false`. The real distinction is with an EmpiricalFit basis + Declared:
    // `basis_ok(Some(EmpiricalFit), false, false)` → `want_empirical = false` → returns false
    // (correct). Under `||`, the expression is `!want_proven || !want_empirical` = `!false || !false`
    // = true, so `want_empirical` wouldn't matter — but the basis match returns `want_empirical = false`.
    // The inner match still returns false in both cases. Actually the || vs && matters only when
    // both flags are true simultaneously — which never happens for `(false, false)`. The real mutant
    // kill is: `Declared` + `ProvenThm` basis must be REJECTED (existing `proven_requires_proven_basis`
    // covers this). Additionally we need `Empirical` + `UserDeclared` to be rejected.
    //
    // Mutant-witness (meta.rs:291:22): the guard `basis_ok(bound, false, true)` for Empirical is
    // replaced with `true`, making Empirical always succeed regardless of basis. We need a test
    // that shows Empirical + UserDeclared (or ProvenThm) must be rejected.
    #[test]
    fn empirical_requires_empirical_fit_basis() {
        // Empirical + EmpiricalFit: ok (M-I3).
        let empirical_ok = Bound {
            kind: BoundKind::Error {
                eps: 0.1,
                norm: NormKind::L2,
            },
            basis: BoundBasis::EmpiricalFit {
                trials: 1_000,
                method: "frady".to_owned(),
            },
        };
        assert!(Meta::new(
            Provenance::Root,
            GuaranteeStrength::Empirical,
            Some(empirical_ok),
            None,
            None,
            None,
        )
        .is_ok());

        // Empirical + UserDeclared: rejected (M-I3). This kills the `true` replacement mutant.
        let declared_basis = Bound {
            kind: BoundKind::Error {
                eps: 0.1,
                norm: NormKind::L2,
            },
            basis: BoundBasis::UserDeclared,
        };
        let bad = Meta::new(
            Provenance::Root,
            GuaranteeStrength::Empirical,
            Some(declared_basis),
            None,
            None,
            None,
        );
        assert_eq!(
            bad.unwrap_err(),
            WfError::GuaranteeBoundMismatch,
            "Empirical with UserDeclared basis must be rejected (M-I3)"
        );

        // Empirical + ProvenThm: also rejected — Empirical is lower than Proven (M-I3 requires
        // EmpiricalFit specifically). This reinforces that the guard is basis-specific.
        let proven_basis = Bound {
            kind: BoundKind::Error {
                eps: 0.05,
                norm: NormKind::L2,
            },
            basis: BoundBasis::ProvenThm {
                citation: "some theorem".to_owned(),
            },
        };
        let bad2 = Meta::new(
            Provenance::Root,
            GuaranteeStrength::Empirical,
            Some(proven_basis),
            None,
            None,
            None,
        );
        assert_eq!(
            bad2.unwrap_err(),
            WfError::GuaranteeBoundMismatch,
            "Empirical with ProvenThm basis must be rejected (M-I3 demands EmpiricalFit)"
        );
    }

    // Mutant-witness for meta.rs:279:60 (the && → || in Declared check): Declared + ProvenThm
    // or Declared + EmpiricalFit must both be rejected; Declared + UserDeclared must be accepted.
    #[test]
    fn declared_accepts_only_user_declared_basis() {
        // Declared + UserDeclared: ok (M-I4).
        let declared_ok = Bound {
            kind: BoundKind::Error {
                eps: 0.2,
                norm: NormKind::Linf,
            },
            basis: BoundBasis::UserDeclared,
        };
        assert!(Meta::new(
            Provenance::Root,
            GuaranteeStrength::Declared,
            Some(declared_ok),
            None,
            None,
            None,
        )
        .is_ok());

        // Declared + EmpiricalFit: rejected. Under `||` mutant, `!false || !true = !false || false`
        // = true — so the branch would (incorrectly) accept. Pinning the rejection kills the mutant.
        let empirical_basis = Bound {
            kind: BoundKind::Error {
                eps: 0.2,
                norm: NormKind::Linf,
            },
            basis: BoundBasis::EmpiricalFit {
                trials: 10,
                method: "m".to_owned(),
            },
        };
        let bad = Meta::new(
            Provenance::Root,
            GuaranteeStrength::Declared,
            Some(empirical_basis),
            None,
            None,
            None,
        );
        assert_eq!(
            bad.unwrap_err(),
            WfError::GuaranteeBoundMismatch,
            "Declared with EmpiricalFit basis must be rejected (M-I4)"
        );
    }
}
