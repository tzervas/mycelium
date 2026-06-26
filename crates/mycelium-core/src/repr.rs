//! Representation descriptors — the paradigm-in-the-type (RFC-0001 §4.1; `repr.schema.json`).
//!
//! The four paradigm *kinds* are closed in the kernel (a fifth needs an RFC + ADR); the parameter
//! registries (`ScalarKind`, VSA `model`, `SparsityClass`) are open.
//!
//! The `serde` wire forms are exactly `repr.schema.json` (M-104): `Repr` is tagged on `kind`
//! (`Binary|Ternary|Dense|VSA`), `SparsityClass` on `class` (`Dense|Sparse`), and `ScalarKind`
//! renders `BF16` (Rust's `Bf16`).

use serde::{Deserialize, Serialize};

use crate::WfError;

/// Upper bound (inclusive) on every declared dimension field of a [`Repr`] — `width`, `trits`,
/// `dim`, and a [`SparsityClass::Sparse`] `max_active`.
///
/// **Why a cap at all (input-validation / DoS guard, DN-40 §3).** The wire forms (`repr.schema.json`,
/// `value.schema.json`) carry these as `u32`, so a crafted descriptor can declare a dimension up to
/// `u32::MAX` (≈ 4.29 × 10⁹). Materializing a value of such a `Repr` allocates that many elements
/// (e.g. an `f64` `Hypervector`/`Scalars` vector), so an unbounded declared dimension is a latent
/// over-allocation (denial-of-service) vector on the deserialize path. The lower `> 0` guard alone
/// does not close it.
///
/// **Why `2^30` (1 073 741 824).** It is a generous-but-finite ceiling that no legitimate value
/// needs: VSA hypervectors are typically ~10⁴, dense embeddings ≤ ~10⁵, and bit/trit widths far
/// smaller — all orders of magnitude under the cap. At the same time a `Repr` at the cap is already
/// impractical to materialize (a `2^30`-element `f64` vector is 8 GiB), so anything above it is
/// firmly in DoS territory, never a real workload. A power of two keeps the constant auditable
/// (KC-3). The *check* is **`Exact`** — a declared dimension is either within the cap or it is not,
/// a total decidable predicate; the *cap value* `2^30` is itself a **`Declared`** policy choice (a
/// DoS heuristic, not an `Exact` fact about any value). The rejection is never-silent —
/// [`Repr::check_well_formed`] returns [`WfError::DimensionTooLarge`] naming the offending field,
/// its value, and this cap (G2).
pub const MAX_DIM: u32 = 1 << 30;

/// Scalar element kind for `Dense` values (extensible registry).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScalarKind {
    /// IEEE-754 binary16.
    F16,
    /// bfloat16.
    #[serde(rename = "BF16")]
    Bf16,
    /// IEEE-754 binary32.
    F32,
    /// IEEE-754 binary64.
    F64,
}

impl ScalarKind {
    /// A stable one-byte code for content-addressing (M-103). Append-only: existing codes are
    /// frozen so a definition's identity never shifts when the registry grows.
    #[must_use]
    pub fn tag(self) -> u8 {
        match self {
            ScalarKind::F16 => 0,
            ScalarKind::Bf16 => 1,
            ScalarKind::F32 => 2,
            ScalarKind::F64 => 3,
        }
    }
}

/// Declared sparsity class of a VSA value (RFC-0001 §4.1; RFC-0003 §5). The *declared* class is a
/// static refinement; *observed* sparsity lives in [`crate::meta::Meta`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "class")]
pub enum SparsityClass {
    /// Dense hypervector.
    Dense,
    /// Sparse hypervector with at most `max_active` non-zero components.
    Sparse {
        /// Upper bound on active components (`> 0` when well-formed).
        max_active: u32,
    },
}

/// The four closed paradigm kinds (RFC-0001 §4.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Repr {
    /// `n`-bit value.
    Binary {
        /// Bit width (`> 0` when well-formed).
        width: u32,
    },
    /// `m` balanced trits in `{-1, 0, +1}`.
    Ternary {
        /// Trit count (`> 0` when well-formed).
        trits: u32,
    },
    /// `dim`-dimensional dense embedding of the given scalar precision.
    Dense {
        /// Dimensionality (`> 0` when well-formed).
        dim: u32,
        /// Element precision (semantically significant — bounds embedding error).
        dtype: ScalarKind,
    },
    /// Hypervector of the named VSA model.
    #[serde(rename = "VSA")]
    Vsa {
        /// Model id, resolved against the VSA registry (ADR-008); non-empty when well-formed.
        model: String,
        /// Hypervector dimensionality (`> 0` when well-formed).
        dim: u32,
        /// Declared sparsity class.
        sparsity: SparsityClass,
    },
}

/// Check one dimension field against the `> 0` lower guard and the [`MAX_DIM`] upper guard,
/// returning a never-silent [`WfError::DimensionTooLarge`] (naming `field`, the value, and the cap)
/// when the cap is exceeded, or `Ok(false)` when the value is non-positive (the caller maps that to
/// [`WfError::MalformedRepr`], preserving the existing `> 0` contract). `Ok(true)` means in-range.
fn dim_in_range(field: &'static str, value: u32) -> Result<bool, WfError> {
    if value == 0 {
        return Ok(false);
    }
    if value > MAX_DIM {
        return Err(WfError::DimensionTooLarge {
            field,
            value,
            cap: MAX_DIM,
        });
    }
    Ok(true)
}

impl Repr {
    /// Well-formed iff all widths/dims/trits (and any `max_active`) are positive **and within
    /// [`MAX_DIM`]** and a VSA `model` id is non-empty — matching `repr.schema.json`
    /// (`minimum: 1` / `minLength: 1`) plus the over-allocation cap (DN-40 §3).
    ///
    /// This is the `bool` predicate; [`Repr::check_well_formed`] is the never-silent variant that
    /// names *why* (used on the construction/deserialize path via [`crate::Value::new`]).
    #[must_use]
    pub fn well_formed(&self) -> bool {
        self.check_well_formed().is_ok()
    }

    /// Never-silent well-formedness check (G2): returns `Ok(())` when the descriptor is well-formed,
    /// [`WfError::DimensionTooLarge`] (naming the field, value, and [`MAX_DIM`]) when a declared
    /// dimension exceeds the over-allocation cap, or [`WfError::MalformedRepr`] for a non-positive
    /// dimension or empty VSA model id. Enforced on the construction/deserialize path through
    /// [`crate::Value::new`], so a crafted huge declared dimension is rejected *before* any value is
    /// materialized — closing the DN-40 §3 over-allocation gap.
    pub fn check_well_formed(&self) -> Result<(), WfError> {
        let in_range = match self {
            Repr::Binary { width } => dim_in_range("width", *width)?,
            Repr::Ternary { trits } => dim_in_range("trits", *trits)?,
            Repr::Dense { dim, .. } => dim_in_range("dim", *dim)?,
            Repr::Vsa {
                model,
                dim,
                sparsity,
            } => {
                let dim_ok = dim_in_range("dim", *dim)?;
                let sparsity_ok = match sparsity {
                    SparsityClass::Dense => true,
                    SparsityClass::Sparse { max_active } => {
                        dim_in_range("max_active", *max_active)?
                    }
                };
                dim_ok && !model.is_empty() && sparsity_ok
            }
        };
        if in_range {
            Ok(())
        } else {
            Err(WfError::MalformedRepr)
        }
    }
}
