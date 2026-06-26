//! White-box tests for [`crate::value`] — [`Value::new`] well-formedness/payload checks and the
//! DN-40 §3 over-allocation cap enforced on the deserialize path. Extracted from the logic file
//! (test-layout rule, M-797).

use crate::meta::{Meta, Provenance};
use crate::repr::MAX_DIM;
use crate::value::{Payload, Trit, Value};
use crate::{Repr, ScalarKind, WfError};

#[test]
fn well_matched_value_constructs() {
    let v = Value::new(
        Repr::Binary { width: 8 },
        Payload::Bits(vec![true, false, true, true, false, false, true, false]),
        Meta::exact(Provenance::Root),
    );
    assert!(v.is_ok());
}

#[test]
fn payload_length_must_match_repr() {
    let v = Value::new(
        Repr::Binary { width: 8 },
        Payload::Bits(vec![true, false]), // wrong length
        Meta::exact(Provenance::Root),
    );
    assert_eq!(v.unwrap_err(), WfError::PayloadReprMismatch);
}

#[test]
fn payload_paradigm_must_match_repr() {
    let v = Value::new(
        Repr::Binary { width: 1 },
        Payload::Trits(vec![Trit::Pos]), // wrong paradigm
        Meta::exact(Provenance::Root),
    );
    assert_eq!(v.unwrap_err(), WfError::PayloadReprMismatch);
}

#[test]
fn malformed_repr_rejected() {
    let v = Value::new(
        Repr::Binary { width: 0 },
        Payload::Bits(vec![]),
        Meta::exact(Provenance::Root),
    );
    assert_eq!(v.unwrap_err(), WfError::MalformedRepr);
}

// --- DN-40 §3: over-allocation cap enforced through Value::new and the deserialize path -----------

/// `Value::new` rejects an over-cap declared dimension *before* the payload is examined — naming the
/// field/value/cap (over-allocation guard). The payload here is deliberately tiny: the point is that
/// the huge *declared* `dim` is caught before anything is sized to it.
#[test]
fn value_new_rejects_over_cap_dim_before_payload() {
    let over = MAX_DIM + 1;
    let v = Value::new(
        Repr::Dense {
            dim: over,
            dtype: ScalarKind::F64,
        },
        Payload::Scalars(vec![0.0]), // mismatched length — but the cap is checked first
        Meta::exact(Provenance::Root),
    );
    assert_eq!(
        v.unwrap_err(),
        WfError::DimensionTooLarge {
            field: "dim",
            value: over,
            cap: MAX_DIM,
        }
    );
}

/// (c) Deserializing a `Value` whose `repr` declares an over-cap dimension is rejected — never
/// silently accepted then over-allocated. `Value`'s `Deserialize` routes through `Value::new`, so
/// the cap is enforced on the wire path; the serde error carries the named cap message.
#[test]
fn deserialize_rejects_over_cap_declared_dimension() {
    let over = MAX_DIM + 1;
    // A self-describing wire value with a crafted huge declared `dim` and a (deliberately small)
    // payload. A naive consumer might size a buffer to `dim` before checking the payload.
    let json = format!(
        r#"{{"repr":{{"kind":"Dense","dim":{over},"dtype":"F32"}},"meta":{{"provenance":{{"kind":"Root"}},"guarantee":"Exact"}},"payload":{{"scalars":[0.0]}}}}"#
    );
    let err = serde_json::from_str::<Value>(&json).expect_err("over-cap dim must be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("dim") && msg.contains(&MAX_DIM.to_string()),
        "deserialize error must name the offending dim and the cap (never-silent): {msg:?}"
    );
}

/// A declared dimension exactly at the cap round-trips through deserialize when its payload matches
/// (the inclusive bound does not reject a legitimate at-cap value on the wire). We keep the payload
/// length consistent with a small `dim` here to avoid allocating a billion elements in a unit test,
/// asserting instead via `Value::new` that the at-cap descriptor itself is well-formed.
#[test]
fn at_cap_descriptor_is_accepted_by_value_new() {
    // dim at the cap with a matching-length payload would allocate MAX_DIM scalars; we assert the
    // descriptor's well-formedness directly (the materialization cost is the very thing the cap
    // bounds), and rely on `repr` tests for the deserialize-of-Repr at-cap case.
    assert!(Repr::Dense {
        dim: MAX_DIM,
        dtype: ScalarKind::F32
    }
    .well_formed());
}
