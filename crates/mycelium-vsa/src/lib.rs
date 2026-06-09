//! `mycelium-vsa` — the **VSA submodule**: the `VsaModel` trait and its first model, **MAP-I**
//! (M-130; RFC-0003 §3–§4; ADR-008; T2.6).
//!
//! This is a **dependency-gated submodule** (ADR-008): it depends on `mycelium-core` but the kernel
//! does *not* depend on it. The kernel already type-checks hypervector *mentions* — `Repr::Vsa` and
//! `Payload::Hypervector` live in core — so programs that name VSA values stay well-typed without
//! pulling in this algebra (KC-3: the kernel stays small and auditable; VSA is opt-in).
//!
//! # Honesty of per-operation guarantees (RFC-0003 §4, normative matrix)
//! Each model declares, per operation, an [intrinsic guarantee](VsaModel::intrinsic_guarantee). For
//! **MAP-I**: `bind`/`unbind` are **self-inverse and `Exact`** (algebraic — elementwise product on
//! bipolar vectors), `permute` is **`Exact`** (a cyclic shift), and `bundle` (elementwise
//! superposition) carries a **`Proven`** capacity bound *citing Clarkson/Thomas* — but that bound's
//! derivation, checked instantiation, and ≥1e4-trial validation are **M-131**. So this module ships
//! the `bundle` *algebra* and the Value-level wrappers for the **Exact** ops; the `Proven`
//! Value-level bundle (which must carry the checked `CapacityBound`, M-I2) is added in M-131 — we do
//! not stamp `Proven` on a value without a checked bound here (VR-5).

pub mod mapi;

pub use mapi::MapI;

use mycelium_core::GuaranteeStrength;

/// The VSA operations a model supplies (RFC-0003 §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VsaOp {
    /// Binding (associate two hypervectors).
    Bind,
    /// Unbinding (recover a factor).
    Unbind,
    /// Bundling / superposition (set-like union).
    Bundle,
    /// Permutation (protect order / quoting).
    Permute,
}

/// Why a VSA operation could not be performed — always explicit, never a silent coercion (G2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VsaError {
    /// Operand dimensionalities disagree (or disagree with the model's `dim`).
    DimMismatch {
        /// Expected length.
        expected: usize,
        /// Actual length.
        got: usize,
    },
    /// A bundle was requested over zero items (no superposition is defined).
    EmptyBundle,
    /// A value handed to a Value-level adapter was not a hypervector of the expected model.
    NotThisModel {
        /// The model id the adapter expected.
        expected: &'static str,
    },
    /// A constructed result violated a Core IR invariant.
    Wf(mycelium_core::WfError),
}

impl core::fmt::Display for VsaError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            VsaError::DimMismatch { expected, got } => {
                write!(f, "dimension mismatch: expected {expected}, got {got}")
            }
            VsaError::EmptyBundle => write!(f, "bundle requires at least one item"),
            VsaError::NotThisModel { expected } => {
                write!(f, "expected a {expected} hypervector value")
            }
            VsaError::Wf(e) => write!(f, "well-formedness violation: {e}"),
        }
    }
}

impl std::error::Error for VsaError {}

/// A composition-style VSA model (RFC-0003 §3): the `bind`/`unbind` (+ self-inverse flag),
/// `bundle`, `permute`, `similarity` algebra over hypervectors (represented as `&[f64]`), plus the
/// honest per-operation guarantee tag. Concrete models (MAP-I, …) implement it; the registry that
/// resolves a `Repr::Vsa { model }` to an implementation is ADR-008 (later).
pub trait VsaModel {
    /// The registry model id (e.g. `"MAP-I"`), matching `Repr::Vsa { model }`.
    fn model_id(&self) -> &'static str;

    /// Whether `unbind` is the same operation as `bind` (true for MAP-I / BSC).
    fn self_inverse(&self) -> bool;

    /// The honest intrinsic guarantee for an operation (RFC-0003 §4). `Proven` here is a *literature*
    /// claim about the operation; a `Proven` **value** still requires a checked bound (M-131, M-I2).
    fn intrinsic_guarantee(&self, op: VsaOp) -> GuaranteeStrength;

    /// Bind two hypervectors (associate). For MAP-I this is the elementwise product.
    fn bind(&self, a: &[f64], b: &[f64]) -> Result<Vec<f64>, VsaError>;

    /// Unbind (recover a factor): the (approximate or exact) inverse of [`bind`](Self::bind).
    fn unbind(&self, a: &[f64], b: &[f64]) -> Result<Vec<f64>, VsaError>;

    /// Bundle (superpose) a non-empty set of hypervectors. The retrieval/capacity bound is supplied
    /// by the bound derivation (M-131), not here.
    fn bundle(&self, items: &[&[f64]]) -> Result<Vec<f64>, VsaError>;

    /// Permute (cyclically shift) a hypervector by `shift` positions — protects order/quotes a role.
    fn permute(&self, a: &[f64], shift: i64) -> Result<Vec<f64>, VsaError>;

    /// The inverse of [`permute`](Self::permute) by the same `shift`.
    fn unpermute(&self, a: &[f64], shift: i64) -> Result<Vec<f64>, VsaError>;

    /// Cosine similarity in `[-1, 1]` (`0` if either operand has zero norm).
    fn similarity(&self, a: &[f64], b: &[f64]) -> f64;
}
