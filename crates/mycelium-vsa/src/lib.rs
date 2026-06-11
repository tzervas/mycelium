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

pub mod bsc;
pub mod capacity;
pub mod cleanup;
pub mod fhrr;
pub mod hrr;
pub mod mapb;
pub mod mapi;
pub mod matrix;
pub mod recon;
pub mod sbc;
pub(crate) mod wrap;

pub use bsc::Bsc;
pub use cleanup::{CleanupMemory, Match};
pub use fhrr::Fhrr;
pub use hrr::Hrr;
pub use mapb::MapB;
pub use mapi::MapI;
pub use matrix::{matrix_tag, RFC0003_MATRIX};
pub use recon::reconstruct_role;
pub use sbc::Sbc;

use mycelium_core::{Bound, BoundBasis, BoundKind, GuaranteeStrength};

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
#[derive(Debug, Clone, PartialEq)]
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
    /// A `Proven` bundle was requested but the dimension is below `requiredDim(items, δ)` — the
    /// cited capacity theorem's side-condition fails, so no `Proven` bound can be issued (M-131;
    /// M-I2/VR-5). Raise the dimension or lower the item count / relax `δ`.
    InsufficientCapacity {
        /// Number of items bundled.
        items: u64,
        /// The dimension supplied.
        dim: u64,
        /// The dimension the theorem requires.
        required: u64,
    },
    /// A value handed to a Value-level adapter was not a hypervector of the expected model.
    NotThisModel {
        /// The model id the adapter expected.
        expected: &'static str,
    },
    /// A component is outside the model's alphabet (e.g. not `±1` for a bipolar model, not
    /// `0/1` for BSC) — the algebra is undefined there; refused, never coerced (G2).
    NonAlphabetComponent {
        /// Index of the offending component.
        index: usize,
    },
    /// An `Empirical` Value-level op was requested outside the side-conditions its declared
    /// trial-validated profile covers — issuing the tag there would outrun the evidence (VR-5).
    OutsideEmpiricalProfile {
        /// Which side-condition failed.
        detail: String,
    },
    /// A MAP-B bundle input is itself a MAP-B bundle: reliability decays `1/2 + 1/2^r` with
    /// nesting depth `r` (RR-13; RFC-0003 §4), so nesting beyond depth 1 is refused explicitly —
    /// never a silent accuracy loss (M-242).
    NestedBundleUnsupported {
        /// The model whose bundle nesting was refused.
        model: &'static str,
    },
    /// An FHRR bundle component's phasor sum has (near-)zero magnitude — its phase is undefined;
    /// refused, never an arbitrary pick (G2).
    DegenerateBundleComponent {
        /// Index of the offending component.
        index: usize,
    },
    /// The manifest does not support compositional reconstruction with a cleanup decode — the
    /// RFC-0003 §6 indexed-vs-compositional distinction, made operational (M-260).
    NotCompositional,
    /// The requested role is not named in the manifest's recipe — reconstruction outside the
    /// recorded structure is refused, never guessed (G2).
    UnknownRole {
        /// The role that was asked for.
        role: String,
    },
    /// The cleanup confidence fell below the manifest's own threshold — an explicit refusal,
    /// never a silent low-quality retrieval (G2; FR-S4).
    BelowCleanupThreshold {
        /// The achieved confidence.
        confidence: f64,
        /// The manifest's threshold.
        threshold: f64,
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
            VsaError::InsufficientCapacity {
                items,
                dim,
                required,
            } => write!(
                f,
                "insufficient capacity for a Proven bound: bundling {items} items needs dim ≥ {required}, got {dim}"
            ),
            VsaError::NotThisModel { expected } => {
                write!(f, "expected a {expected} hypervector value")
            }
            VsaError::NonAlphabetComponent { index } => {
                write!(f, "component {index} is outside the model's alphabet")
            }
            VsaError::OutsideEmpiricalProfile { detail } => {
                write!(f, "outside the trial-validated empirical profile: {detail}")
            }
            VsaError::NestedBundleUnsupported { model } => write!(
                f,
                "{model} bundle nesting beyond depth 1 is refused (reliability decays with depth — RR-13)"
            ),
            VsaError::DegenerateBundleComponent { index } => write!(
                f,
                "bundle component {index} has a vanished phasor sum — its phase is undefined"
            ),
            VsaError::NotCompositional => write!(
                f,
                "manifest does not support compositional reconstruction with a cleanup decode"
            ),
            VsaError::UnknownRole { role } => {
                write!(f, "role {role:?} is not named in the manifest's recipe")
            }
            VsaError::BelowCleanupThreshold {
                confidence,
                threshold,
            } => write!(
                f,
                "cleanup confidence {confidence} is below the manifest threshold {threshold}"
            ),
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

/// A **trial-validated empirical profile**: the regime over which a crate-declared `Empirical`
/// bound was actually validated, and the bound it backs. The honest counterpart of the M-131
/// checked-instantiation pattern for operations whose corpus basis is trials rather than a cited
/// theorem (RFC-0003 §4 "else `Empirical`"; M-I3/VR-5): the constants below are **exercised by
/// this crate's own trial tests** (`tests/empirical_profiles.rs`) with exactly the declared
/// `trials` count, and a Value-level op refuses — explicitly — outside the profile's
/// side-conditions rather than stretching the tag past its evidence.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmpiricalProfile {
    /// Maximum number of operands the trials covered.
    pub max_items: usize,
    /// Whether the trials covered only an odd operand count (majority/sign bundles).
    pub odd_items_only: bool,
    /// Minimum dimensionality the trials covered.
    pub min_dim: u32,
    /// The validated failure probability (the δ the trials stayed at or below).
    pub delta: f64,
    /// Number of trials the validation runs.
    pub trials: u64,
    /// The fitting/validation method recorded in the `EmpiricalFit` basis.
    pub method: &'static str,
}

impl EmpiricalProfile {
    /// Check the profile's side-conditions for an op over `items` operands at `dim`; a violation
    /// is an explicit [`VsaError::OutsideEmpiricalProfile`].
    pub fn check(&self, items: usize, dim: u32) -> Result<(), VsaError> {
        if items == 0 || items > self.max_items {
            return Err(VsaError::OutsideEmpiricalProfile {
                detail: format!("validated for 1..={} items, got {items}", self.max_items),
            });
        }
        if self.odd_items_only && items.is_multiple_of(2) {
            return Err(VsaError::OutsideEmpiricalProfile {
                detail: format!("validated for an odd item count only, got {items}"),
            });
        }
        if dim < self.min_dim {
            return Err(VsaError::OutsideEmpiricalProfile {
                detail: format!("validated for dim ≥ {}, got {dim}", self.min_dim),
            });
        }
        Ok(())
    }

    /// The δ bound this profile backs, with its honest `EmpiricalFit` basis (M-I3).
    #[must_use]
    pub fn bound(&self) -> Bound {
        Bound {
            kind: BoundKind::Probability { delta: self.delta },
            basis: BoundBasis::EmpiricalFit {
                trials: self.trials,
                method: self.method.to_owned(),
            },
        }
    }
}
