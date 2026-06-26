//! White-box tests for [`crate::repr`] — the [`Repr`] well-formedness predicate, the never-silent
//! [`Repr::check_well_formed`], and the [`MAX_DIM`] over-allocation cap (DN-40 §3). Extracted from
//! the logic file (test-layout rule, M-797).

use crate::repr::{Repr, ScalarKind, SparsityClass, MAX_DIM};
use crate::WfError;

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

// Mutant-witness (repr.rs Dense{dim} and Vsa{dim}): the guard is `> 0` (strictly positive), NOT
// `>= 0`. A zero-dim Repr must be rejected (above). A dim-1 Repr MUST be accepted — pins the `>`
// side. Combined, both tests together kill the `> → >=` mutant on the Dense and Vsa dim checks.
#[test]
fn well_formed_rejects_zero_dim_accepts_one() {
    // Dense dim == 0 is rejected; dim == 1 is accepted (strict lower bound is 1, not 0).
    assert!(!Repr::Dense {
        dim: 0,
        dtype: ScalarKind::F16
    }
    .well_formed());
    assert!(Repr::Dense {
        dim: 1,
        dtype: ScalarKind::F16
    }
    .well_formed());
    // Vsa dim == 0 is rejected; dim == 1 is accepted.
    assert!(!Repr::Vsa {
        model: "MAP-I".to_string(),
        dim: 0,
        sparsity: SparsityClass::Dense,
    }
    .well_formed());
    assert!(Repr::Vsa {
        model: "MAP-I".to_string(),
        dim: 1,
        sparsity: SparsityClass::Dense,
    }
    .well_formed());
}

// --- DN-40 §3: over-allocation cap (MAX_DIM) -----------------------------------------------------

/// (a) A dimension *at* the cap is well-formed — the bound is inclusive, so the most extreme
/// legitimate value is still accepted (pins the `> MAX_DIM`, not `>= MAX_DIM`, side).
#[test]
fn dimension_at_cap_is_well_formed() {
    assert!(Repr::Binary { width: MAX_DIM }.well_formed());
    assert!(Repr::Ternary { trits: MAX_DIM }.well_formed());
    assert!(Repr::Dense {
        dim: MAX_DIM,
        dtype: ScalarKind::F64
    }
    .well_formed());
    assert!(Repr::Vsa {
        model: "MAP-I".to_string(),
        dim: MAX_DIM,
        sparsity: SparsityClass::Sparse {
            max_active: MAX_DIM
        },
    }
    .well_formed());
}

/// (b) A dimension *above* the cap is rejected, never-silently, with the error naming the offending
/// field, its value, and the cap. Each variant's dimension field is exercised.
#[test]
fn dimension_above_cap_rejected_naming_field() {
    let over = MAX_DIM + 1;

    assert_eq!(
        Repr::Binary { width: over }
            .check_well_formed()
            .unwrap_err(),
        WfError::DimensionTooLarge {
            field: "width",
            value: over,
            cap: MAX_DIM,
        }
    );
    assert_eq!(
        Repr::Ternary { trits: over }
            .check_well_formed()
            .unwrap_err(),
        WfError::DimensionTooLarge {
            field: "trits",
            value: over,
            cap: MAX_DIM,
        }
    );
    assert_eq!(
        Repr::Dense {
            dim: over,
            dtype: ScalarKind::F32
        }
        .check_well_formed()
        .unwrap_err(),
        WfError::DimensionTooLarge {
            field: "dim",
            value: over,
            cap: MAX_DIM,
        }
    );
    assert_eq!(
        Repr::Vsa {
            model: "MAP-I".to_string(),
            dim: over,
            sparsity: SparsityClass::Dense,
        }
        .check_well_formed()
        .unwrap_err(),
        WfError::DimensionTooLarge {
            field: "dim",
            value: over,
            cap: MAX_DIM,
        }
    );
    // The Sparse `max_active` field is also capped (its own over-alloc surface).
    assert_eq!(
        Repr::Vsa {
            model: "MAP-I".to_string(),
            dim: 10_000,
            sparsity: SparsityClass::Sparse { max_active: over },
        }
        .check_well_formed()
        .unwrap_err(),
        WfError::DimensionTooLarge {
            field: "max_active",
            value: over,
            cap: MAX_DIM,
        }
    );

    // …and `well_formed()` (the bool predicate) agrees with `check_well_formed()`.
    assert!(!Repr::Binary { width: over }.well_formed());
    assert!(!Repr::Dense {
        dim: u32::MAX,
        dtype: ScalarKind::F64
    }
    .well_formed());
}

/// The never-silent message names the field, the value, and the cap (G2 — not a bare "malformed").
#[test]
fn dimension_too_large_display_names_field_value_cap() {
    let err = Repr::Dense {
        dim: MAX_DIM + 1,
        dtype: ScalarKind::F32,
    }
    .check_well_formed()
    .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("dim"), "must name the field: {msg:?}");
    assert!(
        msg.contains(&(MAX_DIM + 1).to_string()),
        "must name the offending value: {msg:?}"
    );
    assert!(
        msg.contains(&MAX_DIM.to_string()),
        "must name the cap: {msg:?}"
    );
}

/// A non-positive dimension still maps to [`WfError::MalformedRepr`] (the cap check does not change
/// the existing lower-guard contract).
#[test]
fn zero_dim_still_malformed_repr_not_too_large() {
    assert_eq!(
        Repr::Binary { width: 0 }.check_well_formed().unwrap_err(),
        WfError::MalformedRepr
    );
}
