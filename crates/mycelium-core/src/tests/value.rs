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

// --- RFC-0032 D3 (M-749): Repr::Seq payload matching + never-silent indexing --------------------

/// A `Binary{1}` element value, for building small homogeneous sequences in tests.
fn bit(b: bool) -> Value {
    Value::new(
        Repr::Binary { width: 1 },
        Payload::Bits(vec![b]),
        Meta::exact(Provenance::Root),
    )
    .expect("well-formed bit")
}

/// A well-matched sequence (count == len, every element's repr == elem) constructs.
#[test]
fn seq_well_matched_constructs() {
    let v = Value::new(
        Repr::Seq {
            elem: Box::new(Repr::Binary { width: 1 }),
            len: 3,
        },
        Payload::Seq(vec![bit(true), bit(false), bit(true)]),
        Meta::exact(Provenance::Root),
    );
    assert!(v.is_ok());
    // The empty sequence is a legitimate value.
    let empty = Value::new(
        Repr::Seq {
            elem: Box::new(Repr::Binary { width: 1 }),
            len: 0,
        },
        Payload::Seq(vec![]),
        Meta::exact(Provenance::Root),
    );
    assert!(empty.is_ok());
}

/// A sequence whose element count differs from its declared `len` is rejected (never silently
/// truncated/padded).
#[test]
fn seq_count_must_match_len() {
    let v = Value::new(
        Repr::Seq {
            elem: Box::new(Repr::Binary { width: 1 }),
            len: 3,
        },
        Payload::Seq(vec![bit(true), bit(false)]), // 2 != 3
        Meta::exact(Provenance::Root),
    );
    assert_eq!(v.unwrap_err(), WfError::PayloadReprMismatch);
}

/// A heterogeneous sequence — an element whose repr differs from the declared `elem` — is rejected
/// (homogeneity is enforced, never silently accepted).
#[test]
fn seq_elements_must_be_homogeneous() {
    let wrong = Value::new(
        Repr::Ternary { trits: 1 },
        Payload::Trits(vec![Trit::Pos]),
        Meta::exact(Provenance::Root),
    )
    .expect("well-formed trit");
    let v = Value::new(
        Repr::Seq {
            elem: Box::new(Repr::Binary { width: 1 }),
            len: 2,
        },
        Payload::Seq(vec![bit(true), wrong]), // second element is Ternary, not Binary{1}
        Meta::exact(Provenance::Root),
    );
    assert_eq!(v.unwrap_err(), WfError::PayloadReprMismatch);
}

/// Never-silent indexing (G2): `seq_get` returns the element in range and `None` out of range —
/// never a panic, never a silent default. `seq_len` reports the element count.
#[test]
fn seq_get_is_never_silent() {
    let seq = Value::new(
        Repr::Seq {
            elem: Box::new(Repr::Binary { width: 1 }),
            len: 2,
        },
        Payload::Seq(vec![bit(true), bit(false)]),
        Meta::exact(Provenance::Root),
    )
    .expect("well-formed seq");

    assert_eq!(seq.seq_len(), Some(2));
    assert_eq!(seq.seq_get(0), Some(&bit(true)));
    assert_eq!(seq.seq_get(1), Some(&bit(false)));
    // Out of bounds → None, never a panic or a silent default.
    assert_eq!(seq.seq_get(2), None);
    assert_eq!(seq.seq_get(usize::MAX), None);

    // The accessors return None for a non-sequence value (never an empty-slice coercion).
    let not_seq = bit(true);
    assert_eq!(not_seq.seq_len(), None);
    assert_eq!(not_seq.seq_get(0), None);
    assert!(not_seq.seq_elems().is_none());
}

/// A sequence value round-trips through JSON faithfully (the wire form carries self-describing
/// elements), and deserializing a count≠len wire form is rejected (never silently accepted).
#[test]
fn seq_json_round_trips_and_rejects_mismatch() {
    let seq = Value::new(
        Repr::Seq {
            elem: Box::new(Repr::Binary { width: 1 }),
            len: 2,
        },
        Payload::Seq(vec![bit(true), bit(false)]),
        Meta::exact(Provenance::Root),
    )
    .expect("well-formed seq");
    let json = serde_json::to_string(&seq).expect("serialize");
    let back: Value = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(seq, back, "seq must round-trip faithfully: {json}");

    // A wire seq whose element count disagrees with the declared len is rejected on the way in.
    let bad = r#"{"repr":{"kind":"Seq","elem":{"kind":"Binary","width":1},"len":3},
                  "payload":{"seq":[
                     {"repr":{"kind":"Binary","width":1},"payload":{"bits":"1"},
                      "meta":{"provenance":{"kind":"Root"},"guarantee":"Exact"}}]},
                  "meta":{"provenance":{"kind":"Root"},"guarantee":"Exact"}}"#;
    assert!(
        serde_json::from_str::<Value>(bad).is_err(),
        "a count≠len seq wire form must be rejected, never silently accepted"
    );
}

/// The content hash distinguishes sequences by their elements and is order-sensitive; an identical
/// sequence collides. (Confirms the `Repr::Seq`/`Payload::Seq` content-addressing arms are wired —
/// without them a constructed seq would panic in `Canon`.)
#[test]
fn seq_content_hash_distinguishes_and_collides() {
    let mk = |a: bool, b: bool| {
        Value::new(
            Repr::Seq {
                elem: Box::new(Repr::Binary { width: 1 }),
                len: 2,
            },
            Payload::Seq(vec![bit(a), bit(b)]),
            Meta::exact(Provenance::Root),
        )
        .expect("well-formed seq")
    };
    // Identical sequences collide.
    assert_eq!(
        mk(true, false).content_hash(),
        mk(true, false).content_hash()
    );
    // Different elements differ.
    assert_ne!(
        mk(true, false).content_hash(),
        mk(true, true).content_hash()
    );
    // Order-sensitive: [t, f] ≠ [f, t].
    assert_ne!(
        mk(true, false).content_hash(),
        mk(false, true).content_hash()
    );
}
