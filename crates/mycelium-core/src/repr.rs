//! Representation descriptors — the paradigm-in-the-type (RFC-0001 §4.1; `repr.schema.json`).
//!
//! The four paradigm *kinds* are closed in the kernel (a fifth needs an RFC + ADR); the parameter
//! registries (`ScalarKind`, VSA `model`, `SparsityClass`) are open.
//!
//! The `serde` wire forms are exactly `repr.schema.json` (M-104): `Repr` is tagged on `kind`
//! (`Binary|Ternary|Dense|VSA`), `SparsityClass` on `class` (`Dense|Sparse`), and `ScalarKind`
//! renders `BF16` (Rust's `Bf16`).

use serde::{Deserialize, Serialize};

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

impl Repr {
    /// Well-formed iff all widths/dims/trits (and any `max_active`) are positive and a VSA `model`
    /// id is non-empty — matching `repr.schema.json` (`minimum: 1` / `minLength: 1`).
    #[must_use]
    pub fn well_formed(&self) -> bool {
        match self {
            Repr::Binary { width } => *width > 0,
            Repr::Ternary { trits } => *trits > 0,
            Repr::Dense { dim, .. } => *dim > 0,
            Repr::Vsa {
                model,
                dim,
                sparsity,
            } => {
                *dim > 0
                    && !model.is_empty()
                    && match sparsity {
                        SparsityClass::Dense => true,
                        SparsityClass::Sparse { max_active } => *max_active > 0,
                    }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn well_formed_accepts_positive() {
        assert!(Repr::Binary { width: 8 }.well_formed());
        assert!(Repr::Ternary { trits: 6 }.well_formed());
        assert!(Repr::Dense {
            dim: 768,
            dtype: ScalarKind::F32
        }
        .well_formed());
        assert!(Repr::Vsa {
            model: "MAP-I".to_string(),
            dim: 10_000,
            sparsity: SparsityClass::Sparse { max_active: 100 },
        }
        .well_formed());
    }

    #[test]
    fn well_formed_rejects_zero_and_empty() {
        assert!(!Repr::Binary { width: 0 }.well_formed());
        assert!(!Repr::Ternary { trits: 0 }.well_formed());
        assert!(!Repr::Vsa {
            model: String::new(),
            dim: 10_000,
            sparsity: SparsityClass::Dense,
        }
        .well_formed());
        assert!(!Repr::Vsa {
            model: "MAP-I".to_string(),
            dim: 10_000,
            sparsity: SparsityClass::Sparse { max_active: 0 },
        }
        .well_formed());
    }
}
