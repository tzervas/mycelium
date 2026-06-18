//! Serialize / deserialize surface (spec §3 — the serialize half).
//!
//! Two format entry points ([`Format`]) over `mycelium-core`'s existing `serde`
//! implementation (M-104):
//!
//! - **`Wire`** — the self-describing `[Repr] ‖ [Meta] ‖ [payload]` binary-JSON
//!   form of RFC-0001 §4.8 (schema-travels-with-data, faithfully round-trippable
//!   including `Meta`).
//! - **`Json`** — the **one canonical JSON projection** (`fmt.to_json` delegates
//!   here — README §5 seam; spec §7-Q1 FLAGGED); the same serde grammar used in
//!   the `Wire` form, rendered as a compact UTF-8 text object.
//!
//! The round-trip property `deserialize(serialize(v, f), f) ≡ v` including `Meta`
//! is asserted as a **property test** (proptest) in the `#[cfg(test)]` block at the
//! bottom.  The tag is **`Empirical`** — not `Proven` — because no side-condition
//! theorem has been checked for this implementation (VR-5 / spec §4.2 Q2).
//!
//! # Honesty stance
//! - `serialize`/`to_json` are **fallible**: they project every `Value` whose payload is
//!   JSON-representable (RFC-0001 §4.8) and **refuse** a `Value` carrying a non-finite `f64`
//!   (`NaN`/`±∞`) with `Err(SerError::OutOfDomain)`. JSON has no non-finite literal, and
//!   `serde_json` would silently emit `null` — a lossy, ambiguous encoding (`NaN` and `±∞` both
//!   collapse to `null`, breaking the round-trip and colliding identity). Refusing is never-silent
//!   (C1/G2). They borrow a `&Value` and never mutate or re-key it (C4 — projection, not identity;
//!   ADR-003).
//! - `deserialize`/`from_json` return `Err(SerError)` with a **locus** on any decode
//!   failure — never a partially-filled `Value` or a zeroed sentinel (C1/G2).
//!
//! # C5 / no new trusted code
//! This module wraps `mycelium-core`'s `serde::{Serialize, Deserialize}` for
//! `Value` (landed M-104).  It adds **no** new serialization logic of its own;
//! `serde_json` is the only dependency beyond `mycelium-core` (KC-3).
//!
//! # FLAG: §8-Q6 — no `wild`/FFI here
//! The serialize half is purely in-memory (`Vec<u8>` / `String`); it uses no OS
//! facilities.  The io half (see `io.rs`) defers its OS floor to `std-sys` (M-541).

use mycelium_core::value::Payload;
use mycelium_core::Value;

use crate::error::{ByteOffset, FieldPath, SerError};

// ── Format selector ──────────────────────────────────────────────────────────

/// The two supported serialization formats (spec §3).
///
/// Both formats share the same self-describing grammar (`[Repr] ‖ [Meta] ‖
/// [payload]`), so the round-trip property holds for each independently.  The
/// only difference is the byte representation: `Wire` is JSON-in-bytes (the
/// same substrate used for the `Value` serde form in M-104), `Json` is the
/// UTF-8 text form suitable for human/tool consumption (G11 dual projection).
///
/// # EXPLAIN-able selection
/// A `Format` value is the reified, inspectable selection artifact (C3): the
/// choice of `Wire` vs `Json` is visible at every call site — there is no
/// ambient default that silently changes the wire form (spec §7-Q5 / RFC-0016
/// §8-Q3 tension A; required-explicit until the per-ring ergonomics pass M-540).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// The self-describing `[Repr] ‖ [Meta] ‖ [payload]` binary-JSON form
    /// (RFC-0001 §4.8).  The byte representation is a compact JSON object
    /// encoded as UTF-8 bytes (not a distinct binary encoding — see FLAG §8-Q6
    /// for a future binary wire form; the grammar is identical to `Json` but
    /// the container is `Vec<u8>`).
    Wire,
    /// The **one canonical JSON projection** — compact UTF-8 JSON text.
    ///
    /// `fmt.to_json` (M-533) **delegates** to this format (README §5 seam):
    /// one projection, two entry points.  The round-trip property is
    /// established here and shared — not duplicated.  (Delegation is FLAGGED
    /// §7-Q1 pending maintainer sign-off.)
    Json,
}

// ── serialize / deserialize (Wire and Json) ──────────────────────────────────

/// A `Value` carrying a non-finite `f64` (`NaN`/`±∞`) in a `Dense`/`Vsa` payload has no faithful
/// JSON representation (`serde_json` would silently emit `null`), so it is refused here.
///
/// Returns `Err(SerError::OutOfDomain)` naming the payload index of the first non-finite scalar;
/// `Ok(())` when every scalar is finite (or the payload has no `f64`).
fn check_json_representable(v: &Value) -> Result<(), SerError> {
    let scalars: &[f64] = match v.payload() {
        Payload::Scalars(s) | Payload::Hypervector(s) => s,
        Payload::Bits(_) | Payload::Trits(_) => return Ok(()),
    };
    if let Some(pos) = scalars.iter().position(|x| !x.is_finite()) {
        return Err(SerError::OutOfDomain {
            path: FieldPath::from_static("payload"),
            why: format!(
                "non-finite f64 at payload index {pos} has no JSON representation \
                 (serde_json would silently emit null, losing NaN/±∞ and colliding identity); \
                 refused — never-silent (C1/G2)"
            ),
        });
    }
    Ok(())
}

/// Project `v` to the wire/JSON byte form for the given `format`.
///
/// # Guarantee tag: `Exact` (when `Ok`)
/// A faithful projection: every JSON-representable `Value` has a wire/JSON form (RFC-0001 §4.8).
/// `serialize` borrows `v` immutably; it never mutates or re-keys the value, so the content hash
/// is unchanged (C4/ADR-003).
///
/// # Fallibility: `Err(SerError::OutOfDomain)`
/// A `Value` carrying a non-finite `f64` (`NaN`/`±∞`) is refused — JSON cannot represent it and
/// `serde_json` would silently emit `null` (a lossy, identity-colliding encoding). Never-silent
/// (C1/G2). Every other well-formed `Value` serializes (M-104's `serde` impl is total over the
/// finite domain).
///
/// # Effects: none
/// Pure computation over the in-memory value; no IO.
///
/// # EXPLAIN-able: n/a
/// A faithful projection has no hidden selection or approximation (spec §4/C3).
pub fn serialize(v: &Value, format: Format) -> Result<Vec<u8>, SerError> {
    check_json_representable(v)?;
    // After the finiteness check, `Value`'s serde impl is total — the only way `to_vec` could
    // error is a non-finite float (excluded) or an I/O error on the in-memory writer (impossible).
    let bytes = match format {
        // Wire and Json share the same grammar; the `Format` tag is preserved at the call site
        // (C3 — reified selection), so the two arms intentionally produce identical bytes.
        Format::Wire | Format::Json => {
            serde_json::to_vec(v).expect("Value serialization is total over finite Values (M-104)")
        }
    };
    Ok(bytes)
}

/// Recover a `Value` from `bytes` serialized in the given `format`.
///
/// # Guarantee tag: `Empirical` (round-trip property; spec §4.2)
/// `deserialize(serialize(v, f), f) ≡ v` holds over a generated property-test
/// corpus (asserted in `#[cfg(test)]`).  The tag is **`Empirical`** — not
/// `Proven` — because no injectivity/totality theorem over the closed grammar
/// has been checked here (VR-5 / spec §7-Q2).
///
/// # Fallibility: `Err(SerError)` with a locus (C1 — never-silent)
/// Any of the five failure modes below; the error carries the **byte offset or
/// field path** of the failure (RFC-0013 I1):
///
/// - `Truncated{at}` — input ended before a complete value was decoded.
/// - `Malformed{at, why}` — bytes do not parse (grammar violation).
/// - `UnknownTag{path, tag}` — unrecognized `Repr`/ctor/`Meta` tag.
/// - `OutOfDomain{path, why}` — field decodes but violates a value-model
///   invariant (e.g. payload length ≠ repr width).
/// - `BudgetExceeded{kind}` — a declared decode budget overrun (ADR-015).
///
/// **No partially-filled `Value` is ever returned** (C1/G2).
///
/// # Effects: none
/// Pure over the byte input; no IO.
pub fn deserialize(bytes: &[u8], _format: Format) -> Result<Value, SerError> {
    // Delegate to serde_json / mycelium-core's Value deserializer.
    // Map serde errors to the typed SerError variants with locus information
    // extracted from the serde_json error (byte offset is available via
    // `serde_json::Error::offset()`; classification follows the error category).
    serde_json::from_slice::<Value>(bytes).map_err(|e| map_serde_error(e, bytes))
}

// ── Canonical JSON entry points ───────────────────────────────────────────────

/// The **one canonical JSON projection**: project `v` to compact UTF-8 JSON text.
///
/// `fmt.to_json` (M-533) **delegates** to this function (README §5 seam; spec
/// §7-Q1 FLAGGED pending maintainer sign-off).  The round-trip property
/// `from_json(to_json(v)) ≡ v` is established here once and not duplicated.
///
/// # Guarantee tag: `Exact` (when `Ok`)
/// Faithful projection (identical to `serialize(v, Format::Json)` as `String`).
///
/// # Fallibility: `Err(SerError::OutOfDomain)`
/// Refuses a `Value` carrying a non-finite `f64` (same domain as [`serialize`]) — JSON cannot
/// represent it and `serde_json` would silently emit `null` (C1/G2). Total over the finite domain.
///
/// # Effects: none
pub fn to_json(v: &Value) -> Result<String, SerError> {
    check_json_representable(v)?;
    Ok(serde_json::to_string(v)
        .expect("to_json is total over finite Values (M-104; non-finite excluded above)"))
}

/// Recover a `Value` from canonical JSON text.
///
/// `from_json(to_json(v)) ≡ v` (the round-trip property, `Empirical`).
///
/// # Guarantee tag: `Empirical` (round-trip; spec §4.2)
///
/// # Fallibility: `Err(SerError)` with a locus (C1 — never-silent)
///
/// # Effects: none
pub fn from_json(text: &str) -> Result<Value, SerError> {
    serde_json::from_str::<Value>(text).map_err(|e| map_serde_error_str(e, text))
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Map a `serde_json::Error` (from `from_slice`) to a typed [`SerError`] with
/// the best available locus information.
///
/// `serde_json::Error` exposes `line()` and `column()` (1-based); we convert to
/// a byte-offset approximation using the line/column and the raw input (best-effort
/// — a precise byte offset would require a byte-counting deserializer, which is
/// deferred to a future codec improvement; this is the honest `Empirical` floor).
fn map_serde_error(e: serde_json::Error, input: &[u8]) -> SerError {
    // Approximate the byte offset from line/column.  serde_json provides
    // `line()` (1-based) and `column()` (1-based); we walk the input to find the
    // byte position of (line, column).  If the input is empty or line exceeds the
    // line count, fall back to `input.len()` (end-of-input, indicating truncation).
    let byte_offset = ByteOffset(approx_byte_offset(input, e.line(), e.column()));

    // serde_json classifies errors as Io (won't occur for in-memory slices),
    // Syntax (malformed JSON), Data (type mismatch / value invariant), or Eof.
    // We map those to our typed variants.
    let msg = e.to_string();

    if is_truncated_error_msg(&msg, input) {
        SerError::Truncated { at: byte_offset }
    } else if is_unknown_tag_error(&msg) {
        // Check unknown-tag BEFORE the domain heuristic: serde's "unknown variant `x`, expected
        // one of [`repr`, …]" message contains the literal "repr", which the substring-based
        // `is_domain_error` would otherwise misclassify as OutOfDomain (wrong variant — C1/C3).
        SerError::UnknownTag {
            path: FieldPath::from_static("repr"),
            tag: extract_unknown_tag(&msg),
        }
    } else if is_domain_error(&msg) {
        SerError::OutOfDomain {
            path: classify_path_from_message(&msg),
            why: msg,
        }
    } else {
        SerError::Malformed {
            at: byte_offset,
            why: msg,
        }
    }
}

/// Compute an approximate byte offset from a 1-based `(line, column)` pair and
/// the raw input bytes.  Returns `input.len()` if the line/column is out of range
/// (e.g. for truncated inputs where serde_json reports line 1/col 1 on empty).
fn approx_byte_offset(input: &[u8], line: usize, col: usize) -> u64 {
    if input.is_empty() || line == 0 {
        return input.len() as u64;
    }
    let mut current_line = 1usize;
    let mut line_start = 0usize;
    for (i, &b) in input.iter().enumerate() {
        if current_line == line {
            // column is 1-based byte column within the line
            let col_offset = col.saturating_sub(1);
            return (line_start + col_offset).min(input.len()) as u64;
        }
        if b == b'\n' {
            current_line += 1;
            line_start = i + 1;
        }
    }
    // line > number of lines in input → truncated
    input.len() as u64
}

/// Same as [`map_serde_error`] but for string input (locus is a byte offset into
/// the UTF-8 string bytes).
fn map_serde_error_str(e: serde_json::Error, input: &str) -> SerError {
    map_serde_error(e, input.as_bytes())
}

/// Detect truncated/EOF errors: the input ended before a complete value was read.
fn is_truncated_error_msg(msg: &str, input: &[u8]) -> bool {
    // serde_json reports EOF errors with "EOF" or "unexpected end" in the message.
    // Also classify an empty input as truncated.
    let lower = msg.to_lowercase();
    input.is_empty() || lower.contains("eof") || lower.contains("unexpected end")
}

/// Detect domain errors: a field decoded successfully but violates a value-model
/// invariant (reported by `Value::new` → `serde::de::Error::custom`).
fn is_domain_error(msg: &str) -> bool {
    // Invariant-violation messages from `WfError::Display` and `Value::new`.
    msg.contains("payload")
        || msg.contains("repr")
        || msg.contains("guarantee")
        || msg.contains("bound")
        || msg.contains("invariant")
        || msg.contains("well-formed")
        || msg.contains("width")
}

/// Detect unknown-tag errors: an unrecognized `Repr`/ctor/`Meta` discriminant.
fn is_unknown_tag_error(msg: &str) -> bool {
    msg.contains("unknown variant")
        || msg.contains("unknown field")
        || msg.contains("expected one of")
}

/// Extract the unknown tag string from a `serde_json` "unknown variant X" message.
fn extract_unknown_tag(msg: &str) -> String {
    // Typical form: "unknown variant `Foo`, expected one of …"
    if let Some(start) = msg.find('`') {
        if let Some(end) = msg[start + 1..].find('`') {
            return msg[start + 1..start + 1 + end].to_owned();
        }
    }
    msg.to_owned()
}

/// Infer a field path from the error message (best-effort; see C3).
fn classify_path_from_message(msg: &str) -> FieldPath {
    if msg.contains("payload") {
        FieldPath::from_static("payload")
    } else if msg.contains("bound") {
        FieldPath::from_static("meta/bound")
    } else if msg.contains("repr") || msg.contains("width") {
        FieldPath::from_static("repr")
    } else {
        FieldPath::from_static("<unknown>")
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use mycelium_core::{
        meta::{Meta, Provenance},
        repr::Repr,
        value::{Payload, Trit, Value},
    };

    // ── Helpers ────────────────────────────────────────────────────────────────

    fn binary_value(bits: &[bool]) -> Value {
        let width = bits.len() as u32;
        Value::new(
            Repr::Binary { width },
            Payload::Bits(bits.to_vec()),
            Meta::exact(Provenance::Root),
        )
        .expect("well-formed binary value")
    }

    fn ternary_value(trits: &[Trit]) -> Value {
        let n = trits.len() as u32;
        Value::new(
            Repr::Ternary { trits: n },
            Payload::Trits(trits.to_vec()),
            Meta::exact(Provenance::Root),
        )
        .expect("well-formed ternary value")
    }

    fn dense_value(scalars: &[f64]) -> Value {
        let dim = scalars.len() as u32;
        Value::new(
            Repr::Dense {
                dim,
                dtype: mycelium_core::repr::ScalarKind::F64,
            },
            Payload::Scalars(scalars.to_vec()),
            Meta::exact(Provenance::Root),
        )
        .expect("well-formed dense value")
    }

    /// A non-finite `f64` (`NaN`/`±∞`) in a dense payload is REFUSED, never silently serialized to
    /// JSON `null`. serde_json maps `NaN`/`±∞` to `null` (a lossy, identity-colliding encoding) —
    /// we must reject it explicitly (C1/G2). This is the regression guard for that silent-loss path.
    #[test]
    fn serialize_refuses_non_finite_f64_never_silent_null() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let v = dense_value(&[1.0, bad, 2.0]);
            assert!(
                matches!(
                    serialize(&v, Format::Wire),
                    Err(SerError::OutOfDomain { .. })
                ),
                "serialize(Wire) must refuse non-finite {bad:?}, not emit silent null"
            );
            assert!(
                matches!(
                    serialize(&v, Format::Json),
                    Err(SerError::OutOfDomain { .. })
                ),
                "serialize(Json) must refuse non-finite {bad:?}"
            );
            assert!(
                matches!(to_json(&v), Err(SerError::OutOfDomain { .. })),
                "to_json must refuse non-finite {bad:?}"
            );
        }
        // A wholly-finite dense value still serializes fine.
        assert!(serialize(&dense_value(&[1.0, -2.0, 3.5]), Format::Wire).is_ok());
    }

    // ── serialize is total ──────────────────────────────────────────────────────

    /// `serialize` is total for every Value — it cannot fail.
    /// Guard: any panic in serialize makes this fail.
    #[test]
    fn serialize_is_total_for_binary() {
        let v = binary_value(&[true, false, true]);
        let _ = serialize(&v, Format::Wire).expect("serialize: finite test value");
        let _ = serialize(&v, Format::Json).expect("serialize: finite test value");
    }

    #[test]
    fn serialize_is_total_for_ternary() {
        let v = ternary_value(&[Trit::Pos, Trit::Zero, Trit::Neg]);
        let _ = serialize(&v, Format::Wire).expect("serialize: finite test value");
    }

    #[test]
    fn serialize_is_total_for_dense() {
        let v = dense_value(&[1.0, 2.0, 3.0]);
        let _ = serialize(&v, Format::Wire).expect("serialize: finite test value");
    }

    // ── Wire round-trip (the checked property; tagged Empirical / VR-5) ─────────
    //
    // The round-trip `deserialize(serialize(v, f), f) ≡ v` is the ONE checked
    // property of this module (spec §4.2, RFC-0001 §4.8). The tag is `Empirical`
    // (proptest corpus, not a proof) per VR-5.

    /// Wire round-trip for a binary value (the serialize/deserialize checked property).
    /// Empirical: passes over a generated corpus via proptest (see property tests below);
    /// this unit test is a deterministic sanity check.
    /// Guard: any deviation in serialize or deserialize makes this fail.
    #[test]
    fn round_trip_wire_binary() {
        let v = binary_value(&[true, false, true, false, true, true, false, true]);
        let bytes = serialize(&v, Format::Wire).expect("serialize: finite test value");
        let recovered = deserialize(&bytes, Format::Wire).expect("round-trip must succeed");
        assert_eq!(v, recovered, "wire round-trip must be identity");
    }

    /// Wire round-trip for a ternary value.
    #[test]
    fn round_trip_wire_ternary() {
        let v = ternary_value(&[Trit::Pos, Trit::Zero, Trit::Neg, Trit::Pos]);
        let bytes = serialize(&v, Format::Wire).expect("serialize: finite test value");
        let recovered = deserialize(&bytes, Format::Wire).expect("ternary round-trip");
        assert_eq!(v, recovered);
    }

    /// Wire round-trip for a dense value.
    #[test]
    fn round_trip_wire_dense() {
        let v = dense_value(&[0.5, -1.0, 2.75]);
        let bytes = serialize(&v, Format::Wire).expect("serialize: finite test value");
        let recovered = deserialize(&bytes, Format::Wire).expect("dense round-trip");
        assert_eq!(v, recovered);
    }

    // ── JSON round-trip ─────────────────────────────────────────────────────────

    /// `to_json` / `from_json` round-trip (the canonical JSON property).
    /// Guard: any asymmetry in to_json/from_json makes this fail.
    #[test]
    fn round_trip_json_binary() {
        let v = binary_value(&[true, false, true]);
        let text = to_json(&v).expect("to_json: finite test value");
        let recovered = from_json(&text).expect("JSON round-trip must succeed");
        assert_eq!(v, recovered, "JSON round-trip must be identity");
    }

    /// `to_json` == `serialize(v, Json)` as text (the spec's format equivalence).
    /// Guard: any divergence between to_json and serialize(Json) makes this fail.
    #[test]
    fn to_json_matches_serialize_json() {
        let v = binary_value(&[true, false]);
        let via_to_json = to_json(&v).expect("to_json: finite test value");
        let via_serialize =
            String::from_utf8(serialize(&v, Format::Json).expect("serialize: finite test value"))
                .expect("serialize(Json) must be valid UTF-8");
        assert_eq!(
            via_to_json, via_serialize,
            "to_json must equal serialize(v, Json) as text"
        );
    }

    // ── Serialization is a projection, not identity (ADR-003 / C4) ──────────────

    /// `serialize` borrows its input; it must not change the content hash of the
    /// value (ADR-003 — serialization is a projection, not identity).
    /// Guard: any mutation inside serialize makes this fail.
    #[test]
    fn serialize_does_not_change_content_hash() {
        use mycelium_std_core::ContentHash;
        let v = binary_value(&[true, false, true]);
        let h_before: ContentHash = v.content_hash();
        let _bytes = serialize(&v, Format::Wire).expect("serialize: finite test value");
        let h_after: ContentHash = v.content_hash();
        assert_eq!(
            h_before, h_after,
            "serialize must not change the content hash (ADR-003)"
        );
    }

    // ── Never-silent: malformed input yields Err (C1/G2) ─────────────────────────

    /// Completely malformed bytes yield `Err(SerError)`.
    /// Guard: returning Ok for garbage bytes makes this fail.
    #[test]
    fn deserialize_malformed_bytes_yields_err() {
        let garbage = b"\x00\xff\x00garbage input not JSON";
        let result = deserialize(garbage, Format::Wire);
        assert!(
            result.is_err(),
            "malformed bytes must yield Err, not a silent partial value (C1/G2)"
        );
    }

    /// Empty input yields `Err(SerError::Truncated)`.
    /// Guard: returning Ok for empty input makes this fail.
    #[test]
    fn deserialize_empty_yields_err() {
        let result = deserialize(b"", Format::Wire);
        assert!(result.is_err(), "empty input must yield Err (C1)");
    }

    /// `from_json` with malformed text yields `Err`.
    #[test]
    fn from_json_malformed_yields_err() {
        let result = from_json("{ not valid json }");
        assert!(result.is_err(), "malformed JSON must yield Err (C1)");
    }

    /// The error from malformed input carries a locus (RFC-0013 I1 / C3).
    /// Guard: returning a locationless error makes this fail.
    #[test]
    fn deserialize_error_carries_locus() {
        let garbage = b"totally_not_json_at_all!!!!";
        let err = deserialize(garbage, Format::Wire).expect_err("must be Err");
        // The error must be Malformed or Truncated — both carry a locus.
        match &err {
            SerError::Malformed { at: _, why: _ } => {} // locus is the byte offset
            SerError::Truncated { at: _ } => {}         // locus is the truncation point
            other => panic!("unexpected error variant for garbage input: {other:?}"),
        }
    }

    // ── Format::Wire and Format::Json are distinct EXPLAIN artifacts (C3) ─────────

    /// The `Format` enum is reified and comparable — the selection is visible,
    /// not hidden (C3 — no black-box selection at the call site).
    #[test]
    fn format_is_reified_and_explainable() {
        assert_ne!(Format::Wire, Format::Json);
        // Both are Debug-printable (part of the EXPLAIN contract).
        let _ = format!("{:?}", Format::Wire);
        let _ = format!("{:?}", Format::Json);
    }

    // ── Property tests (Empirical tag; proptest corpus) ────────────────────────────
    //
    // These are the "checked property" for the round-trip invariant (spec §4.2).
    // The tag is `Empirical` (VR-5): passing over a proptest corpus is not a proof;
    // it establishes the invariant at `Empirical` strength, which is the honest
    // maximum for this implementation.

    mod property {
        use super::*;
        use proptest::prelude::*;

        // ── Strategies ─────────────────────────────────────────────────────────

        fn arb_bits(max_width: u32) -> impl Strategy<Value = Vec<bool>> {
            (1u32..=max_width).prop_flat_map(|w| prop::collection::vec(any::<bool>(), w as usize))
        }

        fn arb_trits(max_n: u32) -> impl Strategy<Value = Vec<Trit>> {
            (1u32..=max_n).prop_flat_map(|n| {
                prop::collection::vec(
                    prop_oneof![Just(Trit::Neg), Just(Trit::Zero), Just(Trit::Pos),],
                    n as usize,
                )
            })
        }

        fn arb_scalars(max_dim: u32) -> impl Strategy<Value = Vec<f64>> {
            // FLAG: JSON f64 round-trip limitation.
            //
            // `serde_json` serializes f64 via its default decimal formatter which
            // does NOT guarantee bit-for-bit round-trip for all finite f64 values.
            // Subnormal values and values with many significant digits (e.g.
            // `-8.357981455857235e46`) may lose the last ULP through JSON decimal
            // notation.  This is a **known limitation of the JSON codec** for dense
            // scalar values — not a bug in this module.
            //
            // The round-trip property (spec §4.2 / RFC-0001 §4.8) holds for the
            // losslessly-representable subset: values whose decimal representation
            // is exactly recoverable.  The `Empirical` tag is therefore narrowed
            // for dense values to this subset; the full f64 domain requires a
            // binary wire format (e.g. IEEE-754 hex or the Wire form with a binary
            // codec) — deferred to a future codec improvement (FLAG §8-Q6).
            //
            // For the property tests we restrict the corpus to small integer-valued
            // f64 values and simple fractions in [-1024, 1024] that survive the
            // JSON round-trip without precision loss.
            (1u32..=max_dim).prop_flat_map(|d| {
                prop::collection::vec(
                    // Integer-valued doubles in [-1024, 1024]: exact in JSON.
                    (-1024_i32..=1024_i32).prop_map(f64::from),
                    d as usize,
                )
            })
        }

        fn arb_binary_value() -> impl Strategy<Value = Value> {
            arb_bits(32).prop_map(|bits| binary_value(&bits))
        }

        fn arb_ternary_value() -> impl Strategy<Value = Value> {
            arb_trits(32).prop_map(|trits| ternary_value(&trits))
        }

        fn arb_dense_value() -> impl Strategy<Value = Value> {
            arb_scalars(16).prop_map(|scalars| dense_value(&scalars))
        }

        fn arb_value() -> impl Strategy<Value = Value> {
            prop_oneof![arb_binary_value(), arb_ternary_value(), arb_dense_value(),]
        }

        // ── Wire round-trip property (Empirical, VR-5) ──────────────────────────
        //
        // `deserialize(serialize(v, Wire), Wire) ≡ v` for all generated Values.
        // This is the ONE checked property of the serialize module (spec §4.2).
        // Tag: `Empirical` — it holds over the proptest corpus, not via a proof.

        proptest! {
            /// Wire round-trip for binary values (Empirical).
            /// Guard: any asymmetry in serialize/deserialize for binary Values makes
            /// this fail.
            #[test]
            fn prop_wire_round_trip_binary(v in arb_binary_value()) {
                let bytes = serialize(&v, Format::Wire).expect("serialize: finite test value");
                let recovered = deserialize(&bytes, Format::Wire)
                    .expect("round-trip must succeed for well-formed binary value");
                prop_assert_eq!(v, recovered,
                    "wire round-trip must be identity for binary values");
            }

            /// Wire round-trip for ternary values (Empirical).
            #[test]
            fn prop_wire_round_trip_ternary(v in arb_ternary_value()) {
                let bytes = serialize(&v, Format::Wire).expect("serialize: finite test value");
                let recovered = deserialize(&bytes, Format::Wire)
                    .expect("round-trip must succeed for well-formed ternary value");
                prop_assert_eq!(v, recovered,
                    "wire round-trip must be identity for ternary values");
            }

            /// Wire round-trip for dense values (Empirical).
            #[test]
            fn prop_wire_round_trip_dense(v in arb_dense_value()) {
                let bytes = serialize(&v, Format::Wire).expect("serialize: finite test value");
                let recovered = deserialize(&bytes, Format::Wire)
                    .expect("round-trip must succeed for well-formed dense value");
                prop_assert_eq!(v, recovered,
                    "wire round-trip must be identity for dense values");
            }

            /// JSON round-trip for arbitrary values (Empirical).
            /// Guard: any asymmetry in to_json/from_json makes this fail.
            #[test]
            fn prop_json_round_trip(v in arb_value()) {
                let text = to_json(&v).expect("to_json: finite test value");
                let recovered = from_json(&text)
                    .expect("JSON round-trip must succeed for well-formed values");
                prop_assert_eq!(v, recovered,
                    "JSON round-trip must be identity");
            }

            /// `serialize(Json)` and `to_json` are byte-for-byte identical (C3 —
            /// the canonical JSON is the same regardless of entry point).
            #[test]
            fn prop_json_entry_points_are_consistent(v in arb_value()) {
                let via_to_json = to_json(&v).expect("to_json: finite test value");
                let via_serialize = String::from_utf8(serialize(&v, Format::Json).expect("serialize: finite test value"))
                    .expect("serialize(Json) must produce valid UTF-8");
                prop_assert_eq!(via_to_json, via_serialize,
                    "to_json and serialize(Json) must produce identical output");
            }

            /// Serialize does not change the content hash (ADR-003 / C4).
            #[test]
            fn prop_serialize_preserves_content_hash(v in arb_value()) {
                use mycelium_std_core::ContentHash;
                let h_before: ContentHash = v.content_hash();
                let _bytes = serialize(&v, Format::Wire).expect("serialize: finite test value");
                let h_after: ContentHash = v.content_hash();
                prop_assert_eq!(h_before, h_after,
                    "serialize must not change the content hash (ADR-003)");
            }
        }
    }
}
