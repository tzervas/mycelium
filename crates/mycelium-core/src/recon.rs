//! The **reconstruction manifest** (`ReconInfo`) — `Meta.reconstruction` (M-260; RFC-0001 §4.3;
//! RFC-0003 §6, normative; `reconstruction-manifest.schema.json`, the ratified name).
//!
//! Explicitly and inspectably distinguishes (RFC-0003 §6):
//! - **Indexed retrieval** — codebook + similarity + threshold; returns a *stored atom*;
//!   bounded-lossy. NOT holographic reconstruction.
//! - **True compositional reconstruction** — requires the structural **recipe / role schema**
//!   (which ops combined which slots) + algebraic inverse operations; can recover *novel*
//!   combinations never stored — VSA's defining capability over a hash table.
//!
//! The kernel carries only this *data type* (RFC-0003 §2 — "its metadata fields" stay in core);
//! constructing manifests and executing decode procedures is the VSA submodule's business
//! (ADR-008). The wire form is exactly the ratified schema; `Deserialize` re-runs
//! [`ReconInfo::new`]'s invariants, so a malformed manifest is rejected, never silently trusted
//! (the M-104 discipline).
//!
//! Per the schema's ratified comment, a **resonator** decode is Phase-3 exploratory and
//! **probabilistic-only** (FR-C2): its bound basis must not exceed `Empirical` — enforced here.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::bound::{Bound, BoundBasis};
use crate::id::ContentHash;
use crate::WfError;

/// Which capability the manifest supports (RFC-0003 §6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReconMode {
    /// Codebook + similarity + threshold; returns a stored atom (bounded-lossy).
    IndexedRetrieval,
    /// Recipe + algebraic inverses; can recover novel combinations.
    CompositionalReconstruction,
}

/// The compositional recipe / role schema: which ops combined which slots. `structure` maps each
/// role name to the content hash of its role atom (an inspectable object, per the schema).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recipe {
    /// The role names.
    pub roles: Vec<String>,
    /// Role name → content hash of the role atom.
    pub structure: BTreeMap<String, ContentHash>,
}

/// The decoding procedure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecodeProcedure {
    /// Nearest-atom cleanup against the codebook(s).
    Cleanup,
    /// Resonator factorization (Phase-3 exploratory; probabilistic-only — FR-C2).
    Resonator,
}

/// Decoding procedure + parameters: a cleanup threshold (indexed/cleanup) or a resonator factor
/// structure + iteration budget (RFC-0003 §6). Optional fields are omitted from the wire form
/// when absent, matching the schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodeSpec {
    /// The procedure.
    pub procedure: DecodeProcedure,
    /// Minimum acceptable cleanup confidence in `[0, 1]` (required for `Cleanup`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cleanup_threshold: Option<f64>,
    /// Per-factor codebook references (required for `Resonator`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub factors: Option<Vec<ContentHash>>,
    /// Resonator iteration budget (required for `Resonator`; ≥ 1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iteration_budget: Option<u64>,
}

/// The reconstruction manifest. Fields are private; the only constructor, [`ReconInfo::new`],
/// enforces the schema invariants — a malformed manifest is unrepresentable.
#[derive(Debug, Clone, PartialEq)]
pub struct ReconInfo {
    mode: ReconMode,
    model: String,
    dim: u32,
    codebooks: Vec<ContentHash>,
    recipe: Option<Recipe>,
    decode: DecodeSpec,
    bound: Bound,
}

impl ReconInfo {
    /// Build a manifest, enforcing the schema invariants (RFC-0003 §6;
    /// `reconstruction-manifest.schema.json`):
    ///
    /// - `model` non-empty, `dim ≥ 1`, `codebooks` non-empty (content-addressed references);
    /// - `CompositionalReconstruction` **requires** a recipe; `IndexedRetrieval` must not carry
    ///   one (absent on the wire);
    /// - `Cleanup` requires `cleanup_threshold ∈ [0, 1]`; `Resonator` requires non-empty
    ///   `factors` + `iteration_budget ≥ 1` **and** a bound basis no stronger than
    ///   `EmpiricalFit` (probabilistic-only, FR-C2);
    /// - the attached `{ε, δ, strength}` bound must be numerically well-formed.
    pub fn new(
        mode: ReconMode,
        model: impl Into<String>,
        dim: u32,
        codebooks: Vec<ContentHash>,
        recipe: Option<Recipe>,
        decode: DecodeSpec,
        bound: Bound,
    ) -> Result<Self, WfError> {
        let model = model.into();
        if model.is_empty() || dim == 0 || codebooks.is_empty() {
            return Err(WfError::MalformedReconstruction);
        }
        match mode {
            ReconMode::CompositionalReconstruction if recipe.is_none() => {
                return Err(WfError::MalformedReconstruction)
            }
            ReconMode::IndexedRetrieval if recipe.is_some() => {
                return Err(WfError::MalformedReconstruction)
            }
            _ => {}
        }
        match decode.procedure {
            DecodeProcedure::Cleanup => match decode.cleanup_threshold {
                Some(t) if (0.0..=1.0).contains(&t) => {}
                _ => return Err(WfError::MalformedReconstruction),
            },
            DecodeProcedure::Resonator => {
                let factors_ok = decode.factors.as_ref().is_some_and(|f| !f.is_empty());
                let budget_ok = decode.iteration_budget.is_some_and(|b| b >= 1);
                if !factors_ok || !budget_ok {
                    return Err(WfError::MalformedReconstruction);
                }
                // Probabilistic-only (FR-C2): a Proven basis must not appear on a resonator path.
                if matches!(bound.basis, BoundBasis::ProvenThm { .. }) {
                    return Err(WfError::MalformedReconstruction);
                }
            }
        }
        if !bound.well_formed() {
            return Err(WfError::MalformedBound);
        }
        Ok(ReconInfo {
            mode,
            model,
            dim,
            codebooks,
            recipe,
            decode,
            bound,
        })
    }

    /// Which capability this manifest supports.
    #[must_use]
    pub fn mode(&self) -> ReconMode {
        self.mode
    }
    /// The VSA model id (matches the producing `Repr.model`).
    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }
    /// Hypervector dimensionality.
    #[must_use]
    pub fn dim(&self) -> u32 {
        self.dim
    }
    /// The content-addressed codebook references.
    #[must_use]
    pub fn codebooks(&self) -> &[ContentHash] {
        &self.codebooks
    }
    /// The compositional recipe, if this manifest is compositional.
    #[must_use]
    pub fn recipe(&self) -> Option<&Recipe> {
        self.recipe.as_ref()
    }
    /// The decode procedure + parameters.
    #[must_use]
    pub fn decode(&self) -> &DecodeSpec {
        &self.decode
    }
    /// The attached `{ε, δ, strength}` bound certificate.
    #[must_use]
    pub fn bound(&self) -> &Bound {
        &self.bound
    }
}

/// The wire projection (`reconstruction-manifest.schema.json`): `recipe` is omitted when absent
/// (the `IndexedRetrieval` form); `Deserialize` re-runs the invariants. `deny_unknown_fields`
/// enforces the schema's `additionalProperties: false` (A6-02).
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReconWire {
    mode: ReconMode,
    model: String,
    dim: u32,
    codebooks: Vec<ContentHash>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    recipe: Option<Recipe>,
    decode: DecodeSpec,
    bound: Bound,
}

impl Serialize for ReconInfo {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        ReconWire {
            mode: self.mode,
            model: self.model.clone(),
            dim: self.dim,
            codebooks: self.codebooks.clone(),
            recipe: self.recipe.clone(),
            decode: self.decode.clone(),
            bound: self.bound.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ReconInfo {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let w = ReconWire::deserialize(deserializer)?;
        // Wire data is never silently trusted (the M-104 discipline).
        ReconInfo::new(
            w.mode,
            w.model,
            w.dim,
            w.codebooks,
            w.recipe,
            w.decode,
            w.bound,
        )
        .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bound::{BoundKind, NormKind};
    use crate::content::operation_hash;

    fn empirical_bound() -> Bound {
        Bound {
            kind: BoundKind::Probability { delta: 0.01 },
            basis: BoundBasis::EmpiricalFit {
                trials: 10_000,
                method: "test".to_owned(),
            },
        }
    }

    fn cleanup_decode() -> DecodeSpec {
        DecodeSpec {
            procedure: DecodeProcedure::Cleanup,
            cleanup_threshold: Some(0.2),
            factors: None,
            iteration_budget: None,
        }
    }

    #[test]
    fn compositional_requires_a_recipe() {
        let err = ReconInfo::new(
            ReconMode::CompositionalReconstruction,
            "MAP-I",
            1024,
            vec![operation_hash("codebook")],
            None,
            cleanup_decode(),
            empirical_bound(),
        );
        assert_eq!(err.unwrap_err(), WfError::MalformedReconstruction);
    }

    #[test]
    fn indexed_must_not_carry_a_recipe() {
        let recipe = Recipe {
            roles: vec!["color".to_owned()],
            structure: BTreeMap::from([("color".to_owned(), operation_hash("role"))]),
        };
        let err = ReconInfo::new(
            ReconMode::IndexedRetrieval,
            "MAP-I",
            1024,
            vec![operation_hash("codebook")],
            Some(recipe),
            cleanup_decode(),
            empirical_bound(),
        );
        assert_eq!(err.unwrap_err(), WfError::MalformedReconstruction);
    }

    #[test]
    fn resonator_is_probabilistic_only() {
        let proven = Bound {
            kind: BoundKind::Error {
                eps: 0.1,
                norm: NormKind::L2,
            },
            basis: BoundBasis::ProvenThm {
                citation: "nope".to_owned(),
            },
        };
        let err = ReconInfo::new(
            ReconMode::IndexedRetrieval,
            "FHRR",
            1024,
            vec![operation_hash("codebook")],
            None,
            DecodeSpec {
                procedure: DecodeProcedure::Resonator,
                cleanup_threshold: None,
                factors: Some(vec![operation_hash("factor")]),
                iteration_budget: Some(100),
            },
            proven,
        );
        assert_eq!(err.unwrap_err(), WfError::MalformedReconstruction);
    }

    #[test]
    fn wire_round_trips_and_rejects_malformed() {
        let info = ReconInfo::new(
            ReconMode::IndexedRetrieval,
            "MAP-I",
            1024,
            vec![operation_hash("codebook")],
            None,
            cleanup_decode(),
            empirical_bound(),
        )
        .unwrap();
        let json = serde_json::to_value(&info).unwrap();
        // The ratified field names, exactly (reconstruction-manifest.schema.json).
        assert_eq!(json["mode"], "IndexedRetrieval");
        assert_eq!(json["model"], "MAP-I");
        assert_eq!(json["dim"], 1024);
        assert!(json["codebooks"].is_array());
        assert!(json.get("recipe").is_none(), "absent recipe is omitted");
        assert_eq!(json["decode"]["procedure"], "Cleanup");
        assert!(json["bound"]["delta"].is_number());
        let back: ReconInfo = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(back, info);
        // A tampered wire manifest (compositional without a recipe) is rejected.
        let mut bad = json;
        bad["mode"] = "CompositionalReconstruction".into();
        assert!(serde_json::from_value::<ReconInfo>(bad).is_err());
    }
}
