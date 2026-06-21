//! nodule: `web.json` — a thin convenience over `std.io`'s one canonical JSON codec
//! (no new codec — DRY/KC-3), per RFC-0022 §4.1 / §4.5.
//!
//! # Honesty stance
//! - **No new codec:** `encode_body` delegates to `mycelium_std_io::to_json`; `decode_body`
//!   delegates to `mycelium_std_io::from_json`. The round-trip property is established in
//!   `std.io` — it is not re-certified here.
//! - **Never-silent non-finite (C1/G2):** a `Value` carrying a non-finite `f64` (`NaN`/`±∞`)
//!   is refused by `encode_body` with `Err(JsonError::OutOfDomain)` — JSON has no non-finite
//!   literal, and `serde_json` would silently emit `null`. We delegate that check to `to_json`.
//! - **Located decode errors (C3):** `decode_body` wraps `std.io`'s `SerError` which carries
//!   a byte offset locus — never a locationless "parse error".
//!
//! # Guarantee summary (RFC-0022 §4.5)
//! - `encode_body`: `Exact`-when-`Ok`, Fallible `Err(OutOfDomain)`, effects none.
//! - `decode_body`: `Empirical` (round-trip corpus from `std.io`), Fallible `Err(SerError…)`,
//!   effects none, EXPLAIN-able (locus from `SerError`).

use mycelium_core::Value;
use mycelium_std_io::SerError;
use std::fmt;

use crate::http::Body;

// ── JsonError ─────────────────────────────────────────────────────────────────

/// Error type for JSON encode / decode operations (C1 — never-silent).
///
/// Wraps `std.io`'s [`SerError`] to add the web-layer context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonError {
    /// A `Value` carrying a non-finite `f64` was refused — JSON cannot represent it
    /// and `serde_json` would silently emit `null` (C1/G2 — refused, never silent).
    OutOfDomain {
        /// Why the value was rejected (usually the payload index and the non-finite scalar).
        why: String,
    },
    /// A decode failure from `std.io`'s JSON parser (carries a byte-offset locus — C3).
    Decode(SerError),
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonError::OutOfDomain { why } => {
                write!(
                    f,
                    "JSON encode refused: value out of domain — {why} \
                     (non-finite f64 has no JSON representation; refused, never silent null — C1/G2)"
                )
            }
            JsonError::Decode(e) => {
                write!(f, "JSON decode failed: {e}")
            }
        }
    }
}

mycelium_std_core::impl_std_error!(
    JsonError,
    source = |this| {
        match this {
            JsonError::Decode(e) => Some(e),
            JsonError::OutOfDomain { .. } => None,
        }
    }
);

impl From<SerError> for JsonError {
    fn from(e: SerError) -> Self {
        // SerError::OutOfDomain from std.io maps to our OutOfDomain wrapper.
        match &e {
            SerError::OutOfDomain { why, .. } => JsonError::OutOfDomain { why: why.clone() },
            _ => JsonError::Decode(e),
        }
    }
}

// ── encode_body / decode_body ─────────────────────────────────────────────────

/// Encode a `Value` as a JSON [`Body`], delegating to `std.io`'s one canonical JSON codec.
///
/// # Guarantee: `Exact`-when-`Ok` (faithful projection via `std.io`)
/// A `Value` with a finite scalar payload serializes to compact JSON. Non-finite `f64` values
/// (`NaN`/`±∞`) have no JSON representation — this function refuses them with
/// `Err(JsonError::OutOfDomain)`, never emitting a silent `null` (C1/G2). No new codec: the
/// serialization grammar is the same as `std.io::to_json` (DRY/KC-3).
///
/// # Fallibility: `Err(JsonError::OutOfDomain)`
/// Exactly the domain error from `to_json` — any well-formed `Value` with a finite scalar
/// payload succeeds; binary/ternary payloads always succeed.
///
/// # Effects: none
/// Pure; no IO.
pub fn encode_body(value: &Value) -> Result<Body, JsonError> {
    let json_text = mycelium_std_io::to_json(value).map_err(JsonError::from)?;
    Ok(Body::from_string(json_text))
}

/// Decode a [`Body`] as a `Value` using `std.io`'s one canonical JSON codec.
///
/// # Guarantee: `Empirical` (round-trip property inherited from `std.io::from_json`)
/// `decode_body(encode_body(v)) ≡ v` is `Empirical` — the property is established over the
/// `std.io` proptest corpus, not by a checked theorem (VR-5). No new codec: decode grammar is
/// the same as `std.io::from_json` (DRY/KC-3).
///
/// # Fallibility: `Err(JsonError::Decode(SerError))` with locus (C3 — RFC-0013 I1)
/// Any malformed / truncated / domain-violating input yields a located `JsonError::Decode`
/// carrying a `SerError` with a byte offset — never a partially-filled `Value` (C1/G2).
///
/// # Effects: none
/// Pure over the in-memory `Body`; no IO.
pub fn decode_body(body: &Body) -> Result<Value, JsonError> {
    let text = std::str::from_utf8(body.as_bytes()).map_err(|e| {
        JsonError::Decode(SerError::Malformed {
            at: mycelium_std_io::ByteOffset(e.valid_up_to() as u64),
            why: "body is not valid UTF-8".to_owned(),
        })
    })?;
    mycelium_std_io::from_json(text).map_err(JsonError::from)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use mycelium_core::{
        meta::{Meta, Provenance},
        repr::Repr,
        value::{Payload, Value},
    };

    fn binary_value() -> Value {
        Value::new(
            Repr::Binary { width: 4 },
            Payload::Bits(vec![true, false, true, false]),
            Meta::exact(Provenance::Root),
        )
        .expect("well-formed binary value")
    }

    fn dense_value(scalars: &[f64]) -> Value {
        Value::new(
            Repr::Dense {
                dim: scalars.len() as u32,
                dtype: mycelium_core::repr::ScalarKind::F64,
            },
            Payload::Scalars(scalars.to_vec()),
            Meta::exact(Provenance::Root),
        )
        .expect("well-formed dense value")
    }

    // ── encode_body: never-silent non-finite (C1/G2) ──────────────────────────

    /// A `Value` with non-finite `f64` is refused — **never silent `null`** (C1/G2).
    /// This is the load-bearing guard for the codec honesty invariant.
    /// Guard: returning Ok (or emitting `null`) for non-finite f64 makes this fail.
    #[test]
    fn encode_body_refuses_non_finite_f64_never_silent_null() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let v = dense_value(&[1.0, bad]);
            let result = encode_body(&v);
            assert!(
                matches!(result, Err(JsonError::OutOfDomain { .. })),
                "encode_body must refuse non-finite {bad:?}, not emit silent null (C1/G2)"
            );
        }
    }

    /// A `Value` with finite scalars encodes successfully.
    #[test]
    fn encode_body_ok_for_finite_value() {
        let v = dense_value(&[1.0, -2.0, 3.5]);
        assert!(encode_body(&v).is_ok());
    }

    /// A binary value encodes successfully (no f64 scalars).
    #[test]
    fn encode_body_ok_for_binary_value() {
        let v = binary_value();
        assert!(encode_body(&v).is_ok());
    }

    // ── decode_body: never-silent (C1/G2) + locus ────────────────────────────

    /// Malformed JSON body yields an explicit Err (C1/G2).
    /// Guard: returning Ok for garbage input makes this fail.
    #[test]
    fn decode_body_malformed_is_err() {
        let body = Body::from_string("not valid json {{{");
        let result = decode_body(&body);
        assert!(
            result.is_err(),
            "malformed JSON body must yield Err (C1/G2 — never-silent)"
        );
    }

    /// Empty body yields an explicit Err (C1).
    #[test]
    fn decode_body_empty_is_err() {
        let body = Body::empty();
        assert!(decode_body(&body).is_err());
    }

    // ── encode↔decode round-trip (inherits Empirical from std.io) ────────────

    /// `decode_body(encode_body(v)) ≡ v` (Empirical — inherited from `std.io::to_json`/
    /// `from_json`; no new codec here, so no re-certification).
    /// Guard: any asymmetry in delegation makes this fail.
    #[test]
    fn round_trip_binary_value() {
        let v = binary_value();
        let body = encode_body(&v).expect("encode binary value");
        let recovered = decode_body(&body).expect("decode binary value");
        assert_eq!(v, recovered, "encode↔decode round-trip must be identity");
    }

    // ── JsonError Display / locus (C3/G11) ───────────────────────────────────

    #[test]
    fn json_error_out_of_domain_display_mentions_domain() {
        let e = JsonError::OutOfDomain {
            why: "non-finite NaN at index 1".to_owned(),
        };
        let s = e.to_string();
        assert!(s.contains("NaN"), "Display must include the why");
        assert!(
            s.contains("null"),
            "Display must explain the never-silent-null policy (C1/G2)"
        );
    }

    #[test]
    fn json_error_is_std_error() {
        let e = JsonError::OutOfDomain {
            why: "test".to_owned(),
        };
        let _: &dyn std::error::Error = &e;
    }

    #[test]
    fn json_error_decode_chains_source() {
        use mycelium_std_io::{ByteOffset, SerError};
        let inner = SerError::Truncated { at: ByteOffset(0) };
        let e = JsonError::Decode(inner);
        let src = std::error::Error::source(&e);
        assert!(
            src.is_some(),
            "JsonError::Decode must chain to the underlying SerError (C3)"
        );
    }

    // ── Property tests (VR-5 / one per bound) ─────────────────────────────────

    mod property {
        use super::*;
        use mycelium_core::value::Trit;
        use proptest::prelude::*;

        fn arb_binary_value() -> impl Strategy<Value = Value> {
            (1u32..=16u32).prop_flat_map(|w| {
                prop::collection::vec(any::<bool>(), w as usize).prop_map(move |bits| {
                    Value::new(
                        Repr::Binary { width: w },
                        Payload::Bits(bits),
                        Meta::exact(Provenance::Root),
                    )
                    .expect("well-formed binary value")
                })
            })
        }

        fn arb_ternary_value() -> impl Strategy<Value = Value> {
            (1u32..=8u32).prop_flat_map(|n| {
                prop::collection::vec(
                    prop_oneof![Just(Trit::Neg), Just(Trit::Zero), Just(Trit::Pos)],
                    n as usize,
                )
                .prop_map(move |trits| {
                    Value::new(
                        Repr::Ternary { trits: n },
                        Payload::Trits(trits),
                        Meta::exact(Provenance::Root),
                    )
                    .expect("well-formed ternary value")
                })
            })
        }

        fn arb_finite_dense_value() -> impl Strategy<Value = Value> {
            (1u32..=8u32).prop_flat_map(|d| {
                prop::collection::vec(
                    // Integer-valued doubles — exact in JSON (no f64 precision loss)
                    (-256_i32..=256_i32).prop_map(f64::from),
                    d as usize,
                )
                .prop_map(move |scalars| {
                    Value::new(
                        Repr::Dense {
                            dim: d,
                            dtype: mycelium_core::repr::ScalarKind::F64,
                        },
                        Payload::Scalars(scalars),
                        Meta::exact(Provenance::Root),
                    )
                    .expect("well-formed dense value")
                })
            })
        }

        proptest! {
            /// JSON encode↔decode round-trip (Empirical — inherited from std.io; no new codec).
            /// Guard: any asymmetry in delegation makes this fail.
            #[test]
            fn prop_json_round_trip(
                v in prop_oneof![
                    arb_binary_value(),
                    arb_ternary_value(),
                    arb_finite_dense_value(),
                ],
            ) {
                let body = encode_body(&v).expect("encode must succeed for well-formed finite value");
                let recovered = decode_body(&body).expect("decode must succeed for well-formed JSON");
                prop_assert_eq!(v, recovered, "encode↔decode round-trip must be identity");
            }

            /// Non-finite f64 scalars are always refused — never emit silent null (C1/G2).
            /// Guard: returning Ok for non-finite input makes this fail.
            #[test]
            fn prop_non_finite_always_refused(
                bad_scalar in prop_oneof![
                    Just(f64::NAN),
                    Just(f64::INFINITY),
                    Just(f64::NEG_INFINITY),
                ],
            ) {
                let v = Value::new(
                    Repr::Dense { dim: 1, dtype: mycelium_core::repr::ScalarKind::F64 },
                    Payload::Scalars(vec![bad_scalar]),
                    Meta::exact(Provenance::Root),
                ).expect("value can be constructed with non-finite scalar");
                let result = encode_body(&v);
                prop_assert!(
                    matches!(result, Err(JsonError::OutOfDomain { .. })),
                    "non-finite f64 must produce OutOfDomain, never Ok (C1/G2)"
                );
            }
        }
    }
}
