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
pub mod guarantee;
pub mod id;
pub mod lower;
pub mod meta;
pub mod node;
pub mod repr;
pub mod ternary;
pub mod value;

pub use bound::{Bound, BoundBasis, BoundKind, NormKind};
pub use content::{operation_hash, Names};
pub use guarantee::GuaranteeStrength;
pub use id::ContentHash;
pub use meta::{Meta, PackScheme, PhysicalLayout, Provenance, SparsityObs};
pub use node::{Node, PolicyRef, Prim, VarId};
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
        };
        f.write_str(s)
    }
}

impl std::error::Error for WfError {}
