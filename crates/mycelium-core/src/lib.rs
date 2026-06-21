//! `mycelium-core` — the Core IR (M-101): `Value<Repr, Meta>`, the guarantee lattice, the bound
//! vocabulary, and the node grammar (RFC-0001 r2). The Rust types mirror the ratified
//! data-contract schemas under `docs/spec/schemas/`, and the honesty invariants (M-I1…M-I4) are
//! enforced *by construction* (see [`meta::Meta::new`]).
//!
//! Here so far: the guarantee `meet` composition + laws (M-102) and content-addressing (M-103).
//! Not yet here (own issues): (de)serialization to the schemas (M-104), the reference interpreter
//! (M-110).

pub mod binary;
pub mod bound;
pub mod content;
pub mod data;
pub mod datum;
pub mod guarantee;
pub mod id;
pub mod lower;
pub mod meta;
pub mod node;
pub mod prim;
pub mod recon;
pub mod repr;
pub mod ternary;
pub mod value;

pub use bound::{Bound, BoundBasis, BoundKind, NormKind};
pub use content::{operation_hash, Names};
pub use data::{
    CtorDecl, CtorRef, CtorSpec, DataDecl, DataRegistry, DeclSpec, FieldSpec, FieldTy,
    RegistryError,
};
pub use datum::{CoreValue, Datum};
pub use guarantee::GuaranteeStrength;
pub use id::ContentHash;
pub use meta::{Meta, PackScheme, PhysicalLayout, Provenance, SparsityObs};
pub use node::{Alt, Node, PolicyRef, Prim, VarId};
pub use prim::{PrimDecl, PrimParadigm, PrimRef, PrimSig, PrimTable, WidthRel};
pub use recon::{
    CleanupShape, DecodeProcedure, DecodeSpec, InitStrategy, Recipe, ReconInfo, ReconMode,
};
pub use repr::{Repr, ScalarKind, SparsityClass};
pub use value::{Payload, Trit, Value};

/// Well-formedness errors for Core IR construction (RFC-0001 §4.3/§4.5 invariants).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WfError {
    /// The guarantee/bound pairing violates M-I1…M-I4 (the honesty rule).
    GuaranteeBoundMismatch,
    /// A bound's numeric payload is out of range (e.g. `delta ∉ [0,1]`).
    MalformedBound,
    /// A representation has a non-positive width/dim/trits or an empty VSA model id.
    MalformedRepr,
    /// A payload does not match its representation (paradigm or length).
    PayloadReprMismatch,
    /// A reconstruction manifest violates its schema invariants (RFC-0003 §6;
    /// `reconstruction-manifest.schema.json`).
    MalformedReconstruction,
    /// A measured sparsity observation is out of range (e.g. `density ∉ [0,1]`). Distinct from
    /// [`WfError::MalformedBound`]: a [`SparsityObs`] is an observation, not a
    /// guarantee bound (A6-08).
    MalformedSparsity,
}

impl core::fmt::Display for WfError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            WfError::GuaranteeBoundMismatch => "guarantee/bound inconsistency (M-I1..M-I4)",
            WfError::MalformedBound => "bound payload out of range",
            WfError::MalformedRepr => {
                "representation has non-positive width/dim/trits or empty model"
            }
            WfError::PayloadReprMismatch => "payload does not match its representation",
            WfError::MalformedReconstruction => {
                "reconstruction manifest violates its schema invariants (RFC-0003 §6)"
            }
            WfError::MalformedSparsity => "sparsity observation out of range (density ∉ [0,1])",
        };
        f.write_str(s)
    }
}

impl std::error::Error for WfError {}

#[cfg(test)]
mod tests {
    use super::WfError;

    // Mutant-witness (lib.rs:66:9): `WfError::fmt` replaced with `Ok(Default::default())`
    // which emits an empty string. A non-empty, distinct error message for each variant is
    // required so callers can distinguish errors (G2: never silent).
    #[test]
    fn wf_error_display_is_non_empty_and_variant_specific() {
        // Each variant must produce a non-empty, distinct message.
        let variants = [
            (WfError::GuaranteeBoundMismatch, "M-I"),
            (WfError::MalformedBound, "bound"),
            (WfError::MalformedRepr, "non-positive"),
            (WfError::PayloadReprMismatch, "payload"),
            (WfError::MalformedReconstruction, "manifest"),
            (WfError::MalformedSparsity, "sparsity"),
        ];
        let mut messages = Vec::new();
        for (variant, expected_fragment) in &variants {
            let msg = format!("{variant}");
            assert!(
                !msg.is_empty(),
                "WfError::{variant:?} must not display as empty string"
            );
            assert!(
                msg.contains(expected_fragment),
                "WfError display must contain '{expected_fragment}': got {msg:?}"
            );
            messages.push(msg);
        }
        // All messages must be distinct (no constant replacement covers all variants).
        for i in 0..messages.len() {
            for j in (i + 1)..messages.len() {
                assert_ne!(
                    messages[i], messages[j],
                    "WfError messages must be distinct: [{i}]={:?} == [{j}]={:?}",
                    messages[i], messages[j]
                );
            }
        }
    }
}
