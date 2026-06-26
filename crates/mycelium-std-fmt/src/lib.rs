//! `std.fmt` — Ring-2 dual human/machine projection over one canonical form (M-533).
//!
//! # Summary
//!
//! `std.fmt` renders a [`Value`] into two views of **one canonical form**: a **human**
//! projection (`display`/`debug`) and a **machine** projection (`to_json`/`from_json`), exactly
//! as RFC-0013 §4.3 renders one diagnostic as "two renderers of one truth" (G11). A bounded
//! display (`display_bounded`) uses a [`Budget`] limit and returns a [`Rendering`] whose
//! [`Truncation`] record says what was elided — never a silent drop (C1/G2).
//!
//! # Honesty crux (spec §1 / §4.1)
//!
//! Two structural honesty facts:
//!
//! 1. **Display is a projection, not identity.** Formatting a borrowed `&Value` is a pure
//!    function that never mutates the value and never changes its content hash (ADR-003; C4).
//!    The human and machine views are two projections of *one* content-addressed canonical
//!    form; neither is "new truth".
//!
//! 2. **A truncated rendering says so.** `Truncation::Elided { omitted, marker }` is the
//!    never-silent guard made type-level: a bounded display that drops data *cannot* be
//!    constructed without the `omitted`/`marker` fields, so a silent truncation is
//!    unrepresentable rather than merely discouraged (C1/G2).
//!
//! # Guarantee matrix (RFC-0016 §4.5)
//!
//! Encoded as data in [`GUARANTEE_MATRIX`] and asserted in tests — never prose-only.
//!
//! # Contract conformance (RFC-0016 §4.1 C1–C6)
//!
//! - **C1 never-silent (G2):** `from_json` returns `Result` with explicit `Malformed` /
//!   `UnknownTag` / `OutOfDomain` variants; `display_bounded` returns `Truncation::Elided`
//!   (never a `Complete`-shaped struct when data was dropped).
//! - **C2 honest per-op tag (VR-5):** every op is `Exact` — `fmt` has no accuracy/precision
//!   semantics; the round-trip invariant is the one checked property, not a numeric bound.
//! - **C3 no black boxes / EXPLAIN (SC-3/G11):** `display_bounded` reifies *what* was elided
//!   in the inspectable [`Truncation`] record; other ops neither select, convert, nor
//!   approximate.
//! - **C4 content-addressed, value-semantic (ADR-003):** all ops are pure functions of a
//!   borrowed `&Value`; none mutates the value or its content hash.
//! - **C5 above the kernel (KC-3):** consumes `mycelium-core`; no `unsafe`, no new trusted
//!   code.
//! - **C6 declared bounded effects (RFC-0014):** pure ops declare `none`; `display_bounded`
//!   declares `alloc(budget)` — the bound is on the signature.
//!
//! # JSON delegation to `mycelium-std-io` (M-514) — wired (ratified 2026-06-19)
//!
//! `fmt.to_json`/`from_json` **delegate** to the **one canonical JSON projection** owned by
//! `io`/`serialize` (M-514) — one canonical JSON, two entry points (spec §7-Q1; `README.md §5`).
//! The maintainer ratified the converged delegation (2026-06-19), so the codec, the non-finite
//! refusal (`NaN`/`±∞` refused, never a silent `null`), and the never-silent decode-error
//! classification all live **once**, in `std.io`; this crate keeps only its thin display facade
//! (`Json`/`ToJsonError`/`FromJsonError`) over them. The round-trip invariant
//! (`from_json(to_json(v)).content_hash() == v.content_hash()`) is established once in `std.io`
//! and re-checked here.
//!
//! **Tag-framing note (honesty, VR-5 — RESOLVED 2026-06-19; DN-16, maintainer-ratified).** The two
//! `from_json` tags are **deliberately scope-distinct**, not a contradiction — each names a different
//! property of the shared op, and both are kept. `std.fmt` `from_json` = **`Exact`** claims *decode
//! determinism* (the same JSON text always decodes to the same `Value`, with no accuracy semantics —
//! an `Exact` structural property of the parse, RFC-0016 C2). `std.io` `from_json` = **`Empirical`**
//! claims *round-trip fidelity* (`from_json(to_json(v)) ≡ v`), established by a proptest corpus, not a
//! theorem (VR-5: no checked theorem ⇒ not `Proven`). Neither over-claims (`Proven`); the tags answer
//! different questions about the same call and are intentionally retained as-is. (Cross-ref: `std.io`
//! `guarantee_matrix.rs` `from_json` row.)
//!
//! Design spec: `docs/spec/stdlib/fmt.md`; contract: RFC-0016 §4.1 (C1–C6);
//! guarantee matrix: spec §4.
//!
//! ## Ambient Representation (RFC-0012 §8-Q3)
//!
//! This crate's public API participates in the RFC-0012 ambient-representation contract:
//! the representation choice (binary/ternary/dense/VSA) is implicit at the call site but
//! always reified, queryable, and EXPLAIN-able — never a black box (C3/SC-3).
//! [Declared per RFC-0012; direction accepted in DN-07 §8-Q3; per-ring pass scheduled as M-540.]
//!
//! **For this crate (Ring 2, Tier B):** Format ops render to text; the source representation
//! is always named in the format receipt — `to_json` serializes the `Repr` tag as part of the
//! canonical JSON form (the `Value`'s `Repr` is an observable field, never omitted). A rendered
//! `Value` includes its `Repr`; an `EXPLAIN`-able format rendering is a first-class goal (G11).
#![forbid(unsafe_code)]

use mycelium_core::{GuaranteeStrength, Repr, Value};
use serde::{Deserialize, Serialize};

// ── Re-exports (convenience) ───────────────────────────────────────────────

pub use mycelium_core::{Payload, Trit};

// ── §1. Human projection types ────────────────────────────────────────────

/// A rendered text string (the output of a human projection).
///
/// A thin `String` wrapper so the type surface makes the projection explicit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text(pub String);

impl Text {
    /// Borrow the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Display for Text {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

// ── §2. Bounded display types ─────────────────────────────────────────────

/// A budget for `display_bounded`: the maximum number of *elements* (bits, trits, scalars,
/// or hypervector components) to render before eliding.
///
/// The budget is specified in element units so the contract is paradigm-uniform. A budget of
/// 0 is allowed and produces an immediately-elided rendering (all content elided).
///
/// Declared as the `alloc(budget)` effect in the C6 guarantee for `display_bounded` (spec §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Budget(pub usize);

/// Whether a [`Rendering`] is complete or whether some content was elided.
///
/// `Elided` is the never-silent guard made type-level (spec §3; C1/G2): a bounded display that
/// drops data **cannot** be constructed without the `omitted`/`marker` fields, so a silent
/// truncation is unrepresentable rather than merely discouraged. This is the EXPLAIN-able
/// artifact for `display_bounded` (C3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Truncation {
    /// The rendering is faithful and complete — nothing was elided.
    Complete,
    /// Some content was elided. The `omitted` count and the `marker` string are the
    /// reified record of *what* was dropped and *why* (C3 — inspectable EXPLAIN artifact).
    Elided {
        /// Number of elements omitted (always >= 1).
        omitted: usize,
        /// The elision marker embedded in the rendered text (e.g. `"...<N omitted>"`).
        /// This string is part of the rendered text, not a separate annotation, so it cannot
        /// be confused with real content.
        marker: String,
    },
}

/// The result of `display_bounded`: a rendered text paired with its truncation record.
///
/// When `truncation` is `Truncation::Elided`, the `text` field already contains the `marker`
/// verbatim so the output is self-describing without having to inspect the `Truncation` variant.
/// The `Truncation` variant is still the machine-readable EXPLAIN artifact (C3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rendering {
    /// The bounded human-readable text (may contain the elision marker).
    pub text: Text,
    /// Whether and what was elided (the EXPLAIN-able artifact; C3).
    pub truncation: Truncation,
}

// ── §3. Machine projection types ──────────────────────────────────────────

/// The machine-projection JSON view of a [`Value`] (spec §3 / G11).
///
/// Produced by [`to_json`]; round-trips via [`from_json`]. This is **not** the canonical
/// transport codec (that is `io`/`serialize`, M-514) — it is the *display* machine projection.
///
/// # Delegation (M-514) — wired
/// `to_json`/`from_json` delegate the canonical projection to `mycelium-std-io` (the ratified
/// fmt→io seam; spec §7-Q1, README §5 — "one canonical JSON, two entry points"). This `Json` is
/// the thin display wrapper over that one canonical JSON; the round-trip guarantee
/// (`from_json(to_json(v)).content_hash() == v.content_hash()`) holds through the delegation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Json(pub serde_json::Value);

impl Json {
    /// Borrow the inner `serde_json::Value` for inspection.
    #[must_use]
    pub fn inner(&self) -> &serde_json::Value {
        &self.0
    }
}

/// Errors the `from_json` machine projection can raise.
///
/// Never-silent (C1): a malformed, unknown-tag, or out-of-domain input is an explicit `Err`,
/// never a coercion or a sentinel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FromJsonError {
    /// The JSON structure does not match the expected `Value` wire schema (wrong shape, missing
    /// field, wrong type for a field). Carries a human-readable description of the span / cause.
    Malformed(String),
    /// The `repr.kind` tag is not one of `Binary|Ternary|Dense|VSA`. Carries the unknown name.
    UnknownTag(String),
    /// A field value is out of its stated domain (e.g. `width: 0`, negative dim, empty model).
    OutOfDomain(String),
}

impl core::fmt::Display for FromJsonError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            FromJsonError::Malformed(s) => write!(f, "malformed JSON value: {s}"),
            FromJsonError::UnknownTag(t) => write!(f, "unknown repr.kind tag: {t:?}"),
            FromJsonError::OutOfDomain(s) => write!(f, "field out of domain: {s}"),
        }
    }
}

mycelium_std_core::impl_std_error!(FromJsonError);

/// Error the `to_json` machine projection can raise.
///
/// Never-silent (C1): a `Value` that has no faithful JSON form is an explicit `Err`, never a
/// lossy coercion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToJsonError {
    /// A non-finite `f64` (`NaN`/`±∞`) in a `Dense`/`Vsa` payload has no JSON representation.
    ///
    /// `serde_json` would silently emit `null` (collapsing `NaN`/`±∞` together and breaking the
    /// round-trip), so it is refused (C1/G2). Carries the payload index of the first offender.
    NonFinite {
        /// Index of the first non-finite scalar in the payload.
        index: usize,
    },
}

impl core::fmt::Display for ToJsonError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ToJsonError::NonFinite { index } => write!(
                f,
                "non-finite f64 at payload index {index} has no JSON form \
                 (refused — never a silent null)"
            ),
        }
    }
}

mycelium_std_core::impl_std_error!(ToJsonError);

/// The payload index of the first non-finite `f64`, if any (`Dense`/`Vsa` payloads only).
fn first_non_finite(v: &Value) -> Option<usize> {
    let scalars: &[f64] = match v.payload() {
        Payload::Scalars(s) | Payload::Hypervector(s) => s,
        Payload::Bits(_) | Payload::Trits(_) => return None,
        // A sequence (RFC-0032 D3) carries no flat f64 payload at this level; nested non-finite
        // scalars are caught by the recursive `mycelium-std-io` representability check, not here —
        // return None for the seq's own (absent) scalar payload.
        Payload::Seq(_) => return None,
    };
    scalars.iter().position(|x| !x.is_finite())
}

// ── §4. Human projection operations ───────────────────────────────────────

/// Render `v` as a human-readable string.
///
/// **Guarantee tag: `Exact`** (total, pure, no selection / approximation; C2).
/// **Fallibility: total** — every `Value` has a human display.
/// **Effects: none**.
/// **EXPLAIN: n/a** — no selection or approximation is hidden.
///
/// The output is a projection of a borrowed `&Value`; it never mutates `v` and never changes
/// its content hash (ADR-003; C4).
#[must_use]
pub fn display(v: &Value) -> Text {
    Text(format_value_human(v, false))
}

/// Render `v` as a structural debug string (more detailed than `display`).
///
/// **Guarantee tag: `Exact`** (total, pure; C2).
/// **Fallibility: total**.
/// **Effects: none**.
/// **EXPLAIN: n/a**.
///
/// Like `display` but includes representation metadata (repr kind, width/dim, guarantee
/// strength) so the output is useful for diagnostics. Still a projection, not identity.
#[must_use]
pub fn debug(v: &Value) -> Text {
    Text(format_value_human(v, true))
}

// ── §5. Bounded display operation ─────────────────────────────────────────

/// Render `v` within `limit` elements, emitting a typed `Truncation` record when content is
/// elided — **never a silent drop** (C1/G2).
///
/// **Guarantee tag: `Exact`** — the result is faithful to *what it claims to render*. An
/// `Elided` rendering does not assert completeness; it carries `{omitted, marker}` evidence.
/// **Fallibility: total** — always returns a `Rendering`, even for `limit = Budget(0)`.
/// **Effects: `alloc(budget)`** — the output size is capped at `limit.0` elements (C6).
/// **EXPLAIN: yes** — `Rendering::truncation` is the reified, inspectable artifact of *what*
/// was elided and *why* (C3).
#[must_use]
pub fn display_bounded(v: &Value, limit: Budget) -> Rendering {
    display_bounded_impl(v, limit)
}

// ── §6. Machine projection operations ─────────────────────────────────────

/// Project `v` to a machine-faithful JSON view (the `to_json` half of the dual projection, G11).
///
/// **Guarantee tag: `Exact`** (when `Ok`) — the machine view is a deterministic function of the
/// value's canonical form.
/// **Fallibility: `Err(ToJsonError::NonFinite)`** — a non-finite `f64` has no JSON form and is
/// refused rather than silently coerced to `null` (C1/G2). Every finite `Value` has a JSON view.
/// **Effects: none**.
/// **EXPLAIN: n/a** — no selection or approximation.
///
/// The JSON projection is the identity-preserving wire view: `from_json(to_json(v))` recovers a
/// `Value` with the **same content hash** as `v` (ADR-003; RFC-0001 §4.6). This is the checked
/// round-trip invariant (spec §4; RFC-0016 §4.5).
///
/// # Delegation (M-514) — wired
/// This op delegates the canonical Value→JSON projection (and the non-finite refusal) to
/// `mycelium-std-io::to_json` — the ratified fmt→io seam (spec §7-Q1; README §5). `fmt` reports
/// the first offending payload index as its typed `ToJsonError::NonFinite`.
pub fn to_json(v: &Value) -> Result<Json, ToJsonError> {
    Ok(Json(value_to_json(v)?))
}

/// Reconstruct a [`Value`] from its machine JSON view (the `from_json` half).
///
/// **Guarantee tag: `Exact`** — if the input is well-formed, the output is deterministic.
/// The round-trip property (`from_json(to_json(v))` has the same content hash as `v`) is
/// the one checked invariant of this module (spec §4; RFC-0001 §4.6 / ADR-003).
/// **Fallibility: `Err(Malformed | UnknownTag | OutOfDomain)`** — never a best-effort coercion
/// (C1).
/// **Effects: none**.
/// **EXPLAIN: n/a** — round-trip is the checked property, not a heuristic.
///
/// # Errors
///
/// - [`FromJsonError::Malformed`] — the JSON does not match the value wire schema.
/// - [`FromJsonError::UnknownTag`] — the `repr.kind` field is not `Binary|Ternary|Dense|VSA`.
/// - [`FromJsonError::OutOfDomain`] — a field value is out of its domain (e.g. `width: 0`).
///
/// # Delegation (M-514) — wired
/// Delegates the canonical decode (and its located, classified errors) to
/// `mycelium-std-io::from_json` — the ratified fmt→io seam (spec §7-Q1; README §5).
pub fn from_json(j: &Json) -> Result<Value, FromJsonError> {
    json_to_value(j.inner())
}

// ── §7. Guarantee matrix (RFC-0016 §4.5) — encoded as data, asserted in tests ──

/// One row of the `std.fmt` guarantee matrix (RFC-0016 §4.5; spec §4).
///
/// Encoded as data so tests can assert invariants rather than relying on prose only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixRow {
    /// The exported operation's name.
    pub op: &'static str,
    /// The honest guarantee tag on `Exact > Proven > Empirical > Declared`.
    pub tag: GuaranteeStrength,
    /// Whether the op is fallible (returns `Result`).
    pub fallible: bool,
    /// Whether this op surfaces an inspectable EXPLAIN artifact.
    pub explainable: bool,
    /// The declared effects (`"none"` or `"alloc(budget)"`).
    pub effects: &'static str,
}

/// The `std.fmt` guarantee matrix (spec §4 / RFC-0016 §4.5).
///
/// All five rows are `Exact` — `fmt` has no accuracy/precision semantics (C2). The one
/// substantive honest claim is the **round-trip invariant** (`from_json(to_json(v))` preserves
/// content hash), which is checked in the test suite, not merely stated here.
pub const GUARANTEE_MATRIX: &[MatrixRow] = &[
    MatrixRow {
        op: "display",
        // Exact: a faithful full render; no selection, no approximation (spec §4).
        tag: GuaranteeStrength::Exact,
        fallible: false,
        explainable: false,
        effects: "none",
    },
    MatrixRow {
        op: "debug",
        // Exact: a faithful structural render; no selection, no approximation.
        tag: GuaranteeStrength::Exact,
        fallible: false,
        explainable: false,
        effects: "none",
    },
    MatrixRow {
        op: "to_json",
        // Exact when Ok: the machine view of one canonical form, deterministic. Fallible: a
        // non-finite f64 has no JSON form and is refused (never a silent null) — C1/G2.
        tag: GuaranteeStrength::Exact,
        fallible: true,
        explainable: false,
        effects: "none",
    },
    MatrixRow {
        op: "from_json",
        // Exact: round-trip is the checked property; explicit error set on Err.
        tag: GuaranteeStrength::Exact,
        fallible: true,
        explainable: false,
        effects: "none",
    },
    MatrixRow {
        op: "display_bounded",
        // Exact: faithful to what it claims to render; Elided record is the EXPLAIN artifact.
        tag: GuaranteeStrength::Exact,
        fallible: false,
        explainable: true,
        effects: "alloc(budget)",
    },
];

/// Assert the structural invariants of the guarantee matrix — called from tests.
///
/// Discharges the RFC-0016 §4.5 obligation: "encoded as data, asserted in tests, never
/// prose-only." Panics with a descriptive message on any violation.
pub fn assert_matrix_invariants() {
    assert_eq!(
        GUARANTEE_MATRIX.len(),
        5,
        "spec §4 lists exactly 5 rows (display/debug/to_json/from_json/display_bounded)"
    );
    for row in GUARANTEE_MATRIX {
        assert!(
            !row.op.is_empty(),
            "every matrix row must have a non-empty op name"
        );
        assert_eq!(
            row.tag,
            GuaranteeStrength::Exact,
            "op '{}': every fmt row must be Exact (no accuracy semantics; C2 / spec §4)",
            row.op
        );
        // Only display_bounded has an EXPLAIN artifact and a budget effect.
        if row.op == "display_bounded" {
            assert!(row.explainable, "display_bounded must be EXPLAIN-able (C3)");
            assert_eq!(
                row.effects, "alloc(budget)",
                "display_bounded must declare alloc(budget) (C6)"
            );
        } else {
            assert!(
                !row.explainable,
                "op '{}': only display_bounded is EXPLAIN-able",
                row.op
            );
            assert_eq!(
                row.effects, "none",
                "op '{}': pure ops must declare 'none' effects (C6)",
                row.op
            );
        }
    }
    // The fallible ops are the two JSON ops: to_json (refuses non-finite f64 — never a silent
    // null) and from_json (explicit decode errors). The human projections are total.
    let fallible_ops: Vec<&str> = GUARANTEE_MATRIX
        .iter()
        .filter(|r| r.fallible)
        .map(|r| r.op)
        .collect();
    assert_eq!(
        fallible_ops,
        ["to_json", "from_json"],
        "the JSON ops (to_json, from_json) must be the fallible ones (spec §3 / C1)"
    );
}

// ── §8. Internal rendering helpers ────────────────────────────────────────

/// Render a value in human form (used by both `display` and `debug`).
fn format_value_human(v: &Value, detailed: bool) -> String {
    match v.repr() {
        Repr::Binary { width } => {
            let bits = match v.payload() {
                Payload::Bits(b) => b
                    .iter()
                    .map(|&b| if b { '1' } else { '0' })
                    .collect::<String>(),
                _ => unreachable!("Binary value must have Bits payload"),
            };
            if detailed {
                format!("Binary<{width}>(0b{bits})")
            } else {
                format!("0b{bits}")
            }
        }
        Repr::Ternary { trits } => {
            let ts = match v.payload() {
                Payload::Trits(t) => t
                    .iter()
                    .map(|t| match t {
                        Trit::Neg => '-',
                        Trit::Zero => '0',
                        Trit::Pos => '+',
                    })
                    .collect::<String>(),
                _ => unreachable!("Ternary value must have Trits payload"),
            };
            if detailed {
                format!("Ternary<{trits}>({ts})")
            } else {
                format!("0t{ts}")
            }
        }
        Repr::Dense { dim, dtype } => {
            let xs = match v.payload() {
                Payload::Scalars(s) => s
                    .iter()
                    .map(|x| format!("{x}"))
                    .collect::<Vec<_>>()
                    .join(", "),
                _ => unreachable!("Dense value must have Scalars payload"),
            };
            if detailed {
                format!("Dense<{dim},{dtype:?}>([{xs}])")
            } else {
                format!("[{xs}]")
            }
        }
        Repr::Vsa { model, dim, .. } => {
            let xs = match v.payload() {
                Payload::Hypervector(h) => h
                    .iter()
                    .map(|x| format!("{x}"))
                    .collect::<Vec<_>>()
                    .join(", "),
                _ => unreachable!("Vsa value must have Hypervector payload"),
            };
            if detailed {
                format!("Vsa<{model},{dim}>([{xs}])")
            } else {
                format!("hv[{xs}]")
            }
        }
        Repr::Seq { elem, len } => {
            // RFC-0032 D3 (M-749): render each element recursively in human form, comma-joined.
            let xs = match v.payload() {
                Payload::Seq(elems) => elems
                    .iter()
                    .map(|e| format_value_human(e, detailed))
                    .collect::<Vec<_>>()
                    .join(", "),
                _ => unreachable!("Seq value must have Seq payload"),
            };
            if detailed {
                format!("Seq<{},{len}>([{xs}])", format_repr_short(elem))
            } else {
                format!("[{xs}]")
            }
        }
    }
}

/// A compact element-type label for a sequence's `Repr` (the head paradigm + width). Used only in
/// the *detailed* human rendering, so it is intentionally terse.
fn format_repr_short(r: &Repr) -> String {
    match r {
        Repr::Binary { width } => format!("Binary<{width}>"),
        Repr::Ternary { trits } => format!("Ternary<{trits}>"),
        Repr::Dense { dim, dtype } => format!("Dense<{dim},{dtype:?}>"),
        Repr::Vsa { model, dim, .. } => format!("Vsa<{model},{dim}>"),
        Repr::Seq { elem, len } => format!("Seq<{},{len}>", format_repr_short(elem)),
    }
}

/// Implement `display_bounded` with the never-silent elision discipline (spec §3 / C1/G2).
fn display_bounded_impl(v: &Value, limit: Budget) -> Rendering {
    let Budget(max_elems) = limit;

    match v.repr() {
        Repr::Binary { .. } => {
            let bits = match v.payload() {
                Payload::Bits(b) => b,
                _ => unreachable!("Binary value must have Bits payload"),
            };
            let total = bits.len();
            let rendered_count = total.min(max_elems);
            let omitted = total - rendered_count;

            let rendered: String = bits[..rendered_count]
                .iter()
                .map(|&b| if b { '1' } else { '0' })
                .collect();

            if omitted == 0 {
                Rendering {
                    text: Text(format!("0b{rendered}")),
                    truncation: Truncation::Complete,
                }
            } else {
                let marker = format!("...<{omitted} omitted>");
                Rendering {
                    text: Text(format!("0b{rendered}{marker}")),
                    truncation: Truncation::Elided { omitted, marker },
                }
            }
        }

        Repr::Ternary { .. } => {
            let trits = match v.payload() {
                Payload::Trits(t) => t,
                _ => unreachable!("Ternary value must have Trits payload"),
            };
            let total = trits.len();
            let rendered_count = total.min(max_elems);
            let omitted = total - rendered_count;

            let rendered: String = trits[..rendered_count]
                .iter()
                .map(|t| match t {
                    Trit::Neg => '-',
                    Trit::Zero => '0',
                    Trit::Pos => '+',
                })
                .collect();

            if omitted == 0 {
                Rendering {
                    text: Text(format!("0t{rendered}")),
                    truncation: Truncation::Complete,
                }
            } else {
                let marker = format!("...<{omitted} omitted>");
                Rendering {
                    text: Text(format!("0t{rendered}{marker}")),
                    truncation: Truncation::Elided { omitted, marker },
                }
            }
        }

        Repr::Dense { .. } => {
            let scalars = match v.payload() {
                Payload::Scalars(s) => s,
                _ => unreachable!("Dense value must have Scalars payload"),
            };
            let total = scalars.len();
            let rendered_count = total.min(max_elems);
            let omitted = total - rendered_count;

            let rendered: Vec<String> = scalars[..rendered_count]
                .iter()
                .map(|x| format!("{x}"))
                .collect();

            if omitted == 0 {
                Rendering {
                    text: Text(format!("[{}]", rendered.join(", "))),
                    truncation: Truncation::Complete,
                }
            } else {
                let marker = format!("...<{omitted} omitted>");
                let inner = if rendered.is_empty() {
                    marker.clone()
                } else {
                    format!("{}, {marker}", rendered.join(", "))
                };
                Rendering {
                    text: Text(format!("[{inner}]")),
                    truncation: Truncation::Elided { omitted, marker },
                }
            }
        }

        Repr::Vsa { .. } => {
            let hv = match v.payload() {
                Payload::Hypervector(h) => h,
                _ => unreachable!("Vsa value must have Hypervector payload"),
            };
            let total = hv.len();
            let rendered_count = total.min(max_elems);
            let omitted = total - rendered_count;

            let rendered: Vec<String> = hv[..rendered_count]
                .iter()
                .map(|x| format!("{x}"))
                .collect();

            if omitted == 0 {
                Rendering {
                    text: Text(format!("hv[{}]", rendered.join(", "))),
                    truncation: Truncation::Complete,
                }
            } else {
                let marker = format!("...<{omitted} omitted>");
                let inner = if rendered.is_empty() {
                    marker.clone()
                } else {
                    format!("{}, {marker}", rendered.join(", "))
                };
                Rendering {
                    text: Text(format!("hv[{inner}]")),
                    truncation: Truncation::Elided { omitted, marker },
                }
            }
        }

        Repr::Seq { .. } => {
            // RFC-0032 D3 (M-749): elide on the element *count*, same never-silent discipline as the
            // other paradigms. Each rendered element is shown in (non-detailed) human form.
            let elems = match v.payload() {
                Payload::Seq(e) => e,
                _ => unreachable!("Seq value must have Seq payload"),
            };
            let total = elems.len();
            let rendered_count = total.min(max_elems);
            let omitted = total - rendered_count;

            let rendered: Vec<String> = elems[..rendered_count]
                .iter()
                .map(|e| format_value_human(e, false))
                .collect();

            if omitted == 0 {
                Rendering {
                    text: Text(format!("[{}]", rendered.join(", "))),
                    truncation: Truncation::Complete,
                }
            } else {
                let marker = format!("...<{omitted} omitted>");
                let inner = if rendered.is_empty() {
                    marker.clone()
                } else {
                    format!("{}, {marker}", rendered.join(", "))
                };
                Rendering {
                    text: Text(format!("[{inner}]")),
                    truncation: Truncation::Elided { omitted, marker },
                }
            }
        }
    }
}

// ── §9. JSON serialization helpers — delegated to `mycelium-std-io` (M-514) ──
//
// The canonical Value↔JSON projection, its non-finite refusal, and the never-silent, located
// decode-error classification all live ONCE in `mycelium-std-io` (M-514; `to_json`/`from_json`).
// Per the maintainer-ratified delegation (2026-06-19; spec §7-Q1, README §5 — "one canonical
// JSON, two entry points"), `std.fmt` no longer carries its own codec: it calls `std.io` and
// adapts the result to its thin, display-facing facade. The round-trip property
// (`from_json(to_json(v)).content_hash() == v.content_hash()`) is therefore established once, in
// `std.io` (where it is honestly tagged `Empirical`), and merely re-checked here.

/// Project a [`Value`] to a `serde_json::Value` for `fmt`'s display wrapper, delegating the
/// canonical projection to [`mycelium_std_io::to_json`].
///
/// Refuses a non-finite `f64` with `Err(ToJsonError::NonFinite { index })` — `fmt` reports the
/// first offending payload index for an ergonomic typed error; `std.io` independently refuses the
/// same domain (never a silent `null`), so the seam is consistent (C1/G2).
fn value_to_json(v: &Value) -> Result<serde_json::Value, ToJsonError> {
    if let Some(index) = first_non_finite(v) {
        return Err(ToJsonError::NonFinite { index });
    }
    // Delegate the canonical Value→JSON-text projection to std.io (M-104 grammar, owned by M-514).
    // The text is valid JSON, so re-parsing it into a serde_json::Value for the display wrapper is
    // total.
    let text = mycelium_std_io::to_json(v)
        .expect("io::to_json is total over finite Values (non-finite excluded above)");
    Ok(
        serde_json::from_str(&text)
            .expect("io::to_json emits valid JSON, so the re-parse is total"),
    )
}

/// Reconstruct a [`Value`] from `fmt`'s display JSON wrapper, delegating the canonical decode
/// (and its located, classified errors) to [`mycelium_std_io::from_json`].
///
/// Never-silent (C1): every failure is an explicit [`FromJsonError`] mapped from `std.io`'s
/// located [`mycelium_std_io::SerError`]; no partially-filled `Value` is ever returned.
fn json_to_value(j: &serde_json::Value) -> Result<Value, FromJsonError> {
    // Pre-check `repr.kind` before the full delegation. A `serde_json::Map` serialises its keys in
    // sorted order when the `preserve_order` feature is off (the default), so when this wrapper is
    // rendered via `serde_json::to_string(j)` below, `meta` precedes `repr` — and a missing-field
    // error in `meta` would surface in `std.io` before serde ever reaches `repr.kind`. Checking the
    // tag eagerly here preserves the pre-delegation error priority: unknown `repr.kind` →
    // `UnknownTag`, regardless of field order in the serialised text (C1 — never-silent, classified
    // error set, spec §3).
    if let Some(kind) = j
        .get("repr")
        .and_then(|r| r.get("kind"))
        .and_then(|k| k.as_str())
    {
        match kind {
            "Binary" | "Ternary" | "Dense" | "VSA" => {}
            other => return Err(FromJsonError::UnknownTag(other.to_owned())),
        }
    }
    // Render the display wrapper to canonical text, then delegate the decode to std.io.
    let text = serde_json::to_string(j).expect("a serde_json::Value always re-serializes to text");
    mycelium_std_io::from_json(&text).map_err(from_ser_error)
}

/// Map `std.io`'s located [`mycelium_std_io::SerError`] onto `fmt`'s display [`FromJsonError`],
/// preserving the never-silent error class (C1). The structured locus (byte offset / field path)
/// is folded into the human-readable description `fmt` carries.
///
/// Classification note: `std.io`'s `is_domain_error` heuristic catches `"missing field payload"`
/// as `OutOfDomain` (because "payload" is in its domain-keyword list). For `fmt`, a missing
/// required field is a structural/grammar failure (`Malformed`), not a value-model invariant
/// violation (`OutOfDomain`). We reclassify accordingly.
fn from_ser_error(e: mycelium_std_io::SerError) -> FromJsonError {
    use mycelium_std_io::SerError;
    match &e {
        SerError::UnknownTag { tag, .. } => FromJsonError::UnknownTag(tag.clone()),
        SerError::OutOfDomain { why, .. } => {
            // Reclassify "missing field" as Malformed: missing a required JSON field is a
            // structural grammar failure (C1 — wrong shape), not a domain invariant violation.
            // std.io's domain-keyword heuristic over-classifies these (e.g. "missing field
            // `payload`" → OutOfDomain) because "payload" is in its domain word list.
            if why.contains("missing field") {
                FromJsonError::Malformed(e.to_string())
            } else {
                FromJsonError::OutOfDomain(why.clone())
            }
        }
        SerError::Truncated { .. }
        | SerError::Malformed { .. }
        | SerError::BudgetExceeded { .. } => FromJsonError::Malformed(e.to_string()),
    }
}

// ── §10. Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use mycelium_core::{
        meta::{Meta, Provenance},
        repr::{Repr, ScalarKind, SparsityClass},
        value::{Payload, Value},
    };

    // ── Test helpers ─────────────────────────────────────────────────────

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
        let count = trits.len() as u32;
        Value::new(
            Repr::Ternary { trits: count },
            Payload::Trits(trits.to_vec()),
            Meta::exact(Provenance::Root),
        )
        .expect("well-formed ternary value")
    }

    fn dense_value(xs: &[f64]) -> Value {
        let dim = xs.len() as u32;
        Value::new(
            Repr::Dense {
                dim,
                dtype: ScalarKind::F32,
            },
            Payload::Scalars(xs.to_vec()),
            Meta::exact(Provenance::Root),
        )
        .expect("well-formed dense value")
    }

    fn vsa_value(xs: &[f64]) -> Value {
        let dim = xs.len() as u32;
        Value::new(
            Repr::Vsa {
                model: "MAP-I".to_owned(),
                dim,
                sparsity: SparsityClass::Dense,
            },
            Payload::Hypervector(xs.to_vec()),
            Meta::exact(Provenance::Root),
        )
        .expect("well-formed vsa value")
    }

    // ── Guarantee matrix invariants ───────────────────────────────────────

    /// The guarantee matrix is internally consistent (RFC-0016 §4.5).
    /// Mutation witness: set any row's tag to Proven -> assertion fires.
    #[test]
    fn guarantee_matrix_invariants_hold() {
        assert_matrix_invariants();
    }

    /// All expected ops appear in the matrix exactly once.
    /// Mutation witness: remove a row -> count == 0.
    #[test]
    fn matrix_contains_all_five_ops_exactly_once() {
        let expected = [
            "display",
            "debug",
            "to_json",
            "from_json",
            "display_bounded",
        ];
        for op in &expected {
            let count = GUARANTEE_MATRIX.iter().filter(|r| r.op == *op).count();
            assert_eq!(count, 1, "op '{op}' must appear exactly once in the matrix");
        }
    }

    /// Every row in the matrix is `Exact` (spec §4 tag justification: no accuracy semantics).
    /// Mutation witness: set one row's tag to Empirical -> assertion fires.
    #[test]
    fn all_matrix_rows_are_exact() {
        for row in GUARANTEE_MATRIX {
            assert_eq!(
                row.tag,
                GuaranteeStrength::Exact,
                "op '{}' must be Exact — fmt has no accuracy semantics (C2)",
                row.op
            );
        }
    }

    // ── Human projection: display ─────────────────────────────────────────

    /// `display` on a binary value produces a `0b...` string (total, Exact).
    /// Mutation witness: remove the `0b` prefix -> assertion fails.
    #[test]
    fn display_binary_starts_with_0b() {
        let v = binary_value(&[true, false, true, true]);
        let t = display(&v);
        assert!(
            t.as_str().starts_with("0b"),
            "binary display must start with '0b'; got {t}"
        );
        assert_eq!(t.as_str(), "0b1011");
    }

    /// `display` on a ternary value produces a `0t...` string.
    /// Mutation witness: remove the `0t` prefix -> assertion fails.
    #[test]
    fn display_ternary_starts_with_0t() {
        let v = ternary_value(&[Trit::Pos, Trit::Zero, Trit::Neg]);
        let t = display(&v);
        assert!(
            t.as_str().starts_with("0t"),
            "ternary display must start with '0t'; got {t}"
        );
        assert_eq!(t.as_str(), "0t+0-");
    }

    /// `display` on a dense value produces a `[...]` bracketed list.
    #[test]
    fn display_dense_bracketed() {
        let v = dense_value(&[1.0, -1.0]);
        let t = display(&v);
        assert!(
            t.as_str().starts_with('[') && t.as_str().ends_with(']'),
            "dense display must be bracketed; got {t}"
        );
    }

    /// `display` on a VSA value produces a `hv[...]` string.
    #[test]
    fn display_vsa_hv_prefix() {
        let v = vsa_value(&[0.5, -0.5]);
        let t = display(&v);
        assert!(
            t.as_str().starts_with("hv["),
            "vsa display must start with 'hv['; got {t}"
        );
    }

    // ── Human projection: debug ───────────────────────────────────────────

    /// `debug` on a binary value includes the paradigm and width.
    /// Mutation witness: return the same as `display` -> assertion on "Binary<" fails.
    #[test]
    fn debug_binary_includes_repr_metadata() {
        let v = binary_value(&[true, false]);
        let t = debug(&v);
        assert!(
            t.as_str().contains("Binary<"),
            "debug binary must include 'Binary<'; got {t}"
        );
        assert!(
            t.as_str().contains("0b"),
            "debug binary must include the bit string; got {t}"
        );
    }

    /// `debug` on a ternary value includes the paradigm and trit count.
    #[test]
    fn debug_ternary_includes_repr_metadata() {
        let v = ternary_value(&[Trit::Zero]);
        let t = debug(&v);
        assert!(
            t.as_str().contains("Ternary<"),
            "debug ternary must include 'Ternary<'; got {t}"
        );
    }

    // ── Machine projection: to_json / from_json round-trip ────────────────

    /// The machine-projection round-trip: `from_json(to_json(v)).content_hash() == v.content_hash()`.
    ///
    /// This is the **one checked property** of `std.fmt` (spec §4; G11; RFC-0013 §4.3 / I3;
    /// RFC-0001 §4.6 / ADR-003). The round-trip must preserve the canonical content hash so
    /// that the JSON view is a faithful machine projection of the value's identity.
    ///
    /// Mutation witness: truncate the payload in `to_json` -> hash diverges.
    /// `to_json` refuses a non-finite `f64`, never silently coercing it to JSON `null`.
    ///
    /// `serde_json` maps `NaN`/`±∞` to `null` (a lossy, identity-colliding encoding); the machine
    /// projection must reject it explicitly (C1/G2). Regression guard for that silent-loss path.
    #[test]
    fn to_json_refuses_non_finite_f64_never_silent_null() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let v = dense_value(&[1.0, bad, 2.0]);
            assert_eq!(
                to_json(&v),
                Err(ToJsonError::NonFinite { index: 1 }),
                "to_json must refuse non-finite {bad:?}, not emit a silent null"
            );
        }
        // A wholly-finite dense value still projects fine.
        assert!(to_json(&dense_value(&[0.5, -1.0, 2.0])).is_ok());
    }

    #[test]
    fn machine_projection_round_trip_preserves_content_hash_binary() {
        let v = binary_value(&[true, false, true, false, true, true, false, false]);
        let j = to_json(&v).expect("to_json: finite test value");
        let recovered = from_json(&j).expect("round-trip must succeed on well-formed value");
        assert_eq!(
            v.content_hash(),
            recovered.content_hash(),
            "from_json(to_json(v)) must recover the same content hash (spec §4; ADR-003)"
        );
    }

    /// Round-trip property for ternary values.
    /// Mutation witness: omit trits from the JSON payload -> hash diverges.
    #[test]
    fn machine_projection_round_trip_preserves_content_hash_ternary() {
        let v = ternary_value(&[
            Trit::Pos,
            Trit::Neg,
            Trit::Zero,
            Trit::Pos,
            Trit::Neg,
            Trit::Zero,
        ]);
        let j = to_json(&v).expect("to_json: finite test value");
        let recovered = from_json(&j).expect("round-trip must succeed");
        assert_eq!(v.content_hash(), recovered.content_hash());
    }

    /// Round-trip property for dense values.
    /// Mutation witness: change a scalar in the JSON -> hash diverges.
    #[test]
    fn machine_projection_round_trip_preserves_content_hash_dense() {
        let v = dense_value(&[1.0, -1.0, 0.5, -0.5]);
        let j = to_json(&v).expect("to_json: finite test value");
        let recovered = from_json(&j).expect("round-trip must succeed");
        assert_eq!(v.content_hash(), recovered.content_hash());
    }

    /// Round-trip property for VSA values.
    #[test]
    fn machine_projection_round_trip_preserves_content_hash_vsa() {
        let v = vsa_value(&[0.25, -0.25, 0.75]);
        let j = to_json(&v).expect("to_json: finite test value");
        let recovered = from_json(&j).expect("round-trip must succeed");
        assert_eq!(v.content_hash(), recovered.content_hash());
    }

    /// Full-corpus round-trip property: all 2^8 = 256 byte values.
    ///
    /// Property test for the `Binary` round-trip bound: for *every* 8-bit value the round-trip
    /// is exact. Mutation witness: return a fixed hash from `content_hash` -> all but one fail.
    #[test]
    fn machine_round_trip_all_256_byte_values() {
        for byte_val in 0u16..=255 {
            let bits: Vec<bool> = (0..8).rev().map(|i| (byte_val >> i) & 1 == 1).collect();
            let v = binary_value(&bits);
            let j = to_json(&v).expect("to_json: finite test value");
            let recovered = from_json(&j).expect("round-trip must succeed for all byte values");
            assert_eq!(
                v.content_hash(),
                recovered.content_hash(),
                "round-trip failed for byte_val={byte_val:#04x}"
            );
        }
    }

    /// `from_json` returns `Err(Malformed)` for a JSON non-object (C1 — never-silent).
    /// Mutation witness: return Ok for string input -> assertion fails.
    #[test]
    fn from_json_rejects_non_object_as_malformed() {
        let bad = Json(serde_json::json!("not an object"));
        let err = from_json(&bad).expect_err("a string is not a valid Value");
        assert!(
            matches!(err, FromJsonError::Malformed(_)),
            "expected Malformed, got {err:?}"
        );
    }

    /// `from_json` returns `Err(Malformed)` when a required field is missing.
    #[test]
    fn from_json_rejects_missing_field() {
        let bad = Json(serde_json::json!({ "repr": { "kind": "Binary", "width": 8 } }));
        let err = from_json(&bad).expect_err("missing 'meta'/'payload' must be an error");
        assert!(
            matches!(err, FromJsonError::Malformed(_)),
            "expected Malformed, got {err:?}"
        );
    }

    /// `from_json` returns `Err(UnknownTag)` for a `repr.kind` it does not recognise (C1).
    /// Mutation witness: return Ok or Malformed for an unknown tag -> assertion fails.
    #[test]
    fn from_json_unknown_repr_kind_is_explicit_error() {
        let bad = Json(serde_json::json!({
            "repr": { "kind": "Quantum", "width": 8 },
            "meta": {},
            "payload": { "bits": "00000000" }
        }));
        let err = from_json(&bad).expect_err("unknown kind must be an error");
        assert!(
            matches!(&err, FromJsonError::UnknownTag(t) if t == "Quantum"),
            "expected UnknownTag(\"Quantum\"), got {err:?}"
        );
    }

    // ── Bounded display ───────────────────────────────────────────────────

    /// A budget larger than the value renders `Truncation::Complete` (no elision).
    /// Mutation witness: always return Elided -> assertion fails.
    #[test]
    fn display_bounded_ample_budget_is_complete() {
        let v = binary_value(&[true, false, true]);
        let r = display_bounded(&v, Budget(100));
        assert_eq!(
            r.truncation,
            Truncation::Complete,
            "budget > len must produce Complete, not Elided"
        );
        assert_eq!(r.text.as_str(), "0b101");
    }

    /// A budget of exactly the value's length renders `Truncation::Complete`.
    #[test]
    fn display_bounded_exact_budget_is_complete() {
        let v = binary_value(&[false, true, false, true]);
        let r = display_bounded(&v, Budget(4));
        assert_eq!(r.truncation, Truncation::Complete);
    }

    /// A budget smaller than the value length elides and records `omitted` and `marker` (C1/C3).
    /// Mutation witness: return Complete for any budget -> assertion fails.
    #[test]
    fn display_bounded_tight_budget_elides_and_records_omitted() {
        let v = binary_value(&[true, true, true, true, true, true, true, true]);
        let r = display_bounded(&v, Budget(4));
        match &r.truncation {
            Truncation::Elided { omitted, marker } => {
                assert_eq!(*omitted, 4, "must record 4 omitted bits");
                assert!(!marker.is_empty(), "marker must be non-empty");
                assert!(
                    r.text.as_str().contains(marker.as_str()),
                    "text must embed the marker verbatim (self-describing output)"
                );
            }
            Truncation::Complete => panic!("expected Elided for budget < len"),
        }
    }

    /// A budget of 0 elides everything (all content omitted).
    /// Mutation witness: return Complete for budget=0 -> assertion fails.
    #[test]
    fn display_bounded_zero_budget_elides_all() {
        let v = binary_value(&[true, false]);
        let r = display_bounded(&v, Budget(0));
        assert!(
            matches!(&r.truncation, Truncation::Elided { omitted, .. } if *omitted == 2),
            "budget=0 must elide all 2 bits; got {:?}",
            r.truncation
        );
    }

    /// Ternary bounded display: elision carries the correct `omitted` count.
    #[test]
    fn display_bounded_ternary_elides_correctly() {
        let ts = [
            Trit::Pos,
            Trit::Neg,
            Trit::Zero,
            Trit::Pos,
            Trit::Zero,
            Trit::Neg,
        ];
        let v = ternary_value(&ts);
        let r = display_bounded(&v, Budget(3));
        match &r.truncation {
            Truncation::Elided { omitted, .. } => {
                assert_eq!(*omitted, 3, "must record 3 omitted trits");
            }
            Truncation::Complete => panic!("expected Elided"),
        }
    }

    /// Dense bounded display: elision carries the correct `omitted` count.
    #[test]
    fn display_bounded_dense_elides_correctly() {
        let v = dense_value(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let r = display_bounded(&v, Budget(2));
        match &r.truncation {
            Truncation::Elided { omitted, .. } => {
                assert_eq!(*omitted, 3, "must record 3 omitted scalars");
            }
            Truncation::Complete => panic!("expected Elided"),
        }
    }

    /// VSA bounded display: elision carries the correct `omitted` count.
    #[test]
    fn display_bounded_vsa_elides_correctly() {
        let v = vsa_value(&[0.1, 0.2, 0.3, 0.4]);
        let r = display_bounded(&v, Budget(1));
        match &r.truncation {
            Truncation::Elided { omitted, .. } => {
                assert_eq!(*omitted, 3, "must record 3 omitted components");
            }
            Truncation::Complete => panic!("expected Elided"),
        }
    }

    /// A `Truncation::Elided` value cannot be confused with `Truncation::Complete` at the type
    /// level — the omitted/marker fields make silent truncation unrepresentable (C1/G2; spec §3).
    ///
    /// This test is a structural check: it constructs both variants and verifies they are
    /// distinct by `PartialEq`. The key guarantee is enforced by the type system (the `Elided`
    /// variant requires both `omitted` and `marker`), so this test serves as an explicit,
    /// mutation-witnessable documentation of that fact.
    ///
    /// Mutation witness: collapse Truncation to a bool `elided: bool` -> the type check
    /// collapses and the test would need to be rewritten, surfacing the regression.
    #[test]
    fn truncation_elided_is_not_confusable_with_complete() {
        let complete = Truncation::Complete;
        let elided = Truncation::Elided {
            omitted: 3,
            marker: "...<3 omitted>".to_owned(),
        };
        assert_ne!(
            complete, elided,
            "Complete and Elided must be distinct — silent truncation is unrepresentable (C1/G2)"
        );
    }

    // ── Projection is not identity (C4 / ADR-003) ─────────────────────────

    /// `display` is a pure function of a borrowed `&Value`; the value's content hash is
    /// unchanged after a call to `display` (C4 / ADR-003).
    ///
    /// Mutation witness: if `display` took `&mut Value` and mutated meta -> hash would differ.
    #[test]
    fn display_does_not_change_content_hash() {
        let v = binary_value(&[true, false, true, false]);
        let h_before = v.content_hash();
        let _t = display(&v);
        let h_after = v.content_hash();
        assert_eq!(
            h_before, h_after,
            "display must not change the value's content hash (ADR-003; C4)"
        );
    }

    /// `to_json` does not change the value's content hash (C4 / ADR-003).
    #[test]
    fn to_json_does_not_change_content_hash() {
        let v = ternary_value(&[Trit::Pos, Trit::Zero]);
        let h_before = v.content_hash();
        let _j = to_json(&v).expect("to_json: finite test value");
        let h_after = v.content_hash();
        assert_eq!(h_before, h_after, "to_json must not change content hash");
    }

    // ── Property tests (per-bound property for every stated bound) ─────────

    /// Property: for ALL 3-bit binary values (corpus of 8) the round-trip is exact.
    #[test]
    fn round_trip_property_all_3bit_binary_values() {
        for mask in 0u8..8 {
            let bits: Vec<bool> = (0..3).rev().map(|i| (mask >> i) & 1 == 1).collect();
            let v = binary_value(&bits);
            let recovered =
                from_json(&to_json(&v).expect("to_json: finite test value")).expect("round-trip");
            assert_eq!(
                v.content_hash(),
                recovered.content_hash(),
                "3-bit round-trip failed for mask={mask:#05b}"
            );
        }
    }

    /// Property: for all 27 2-trit ternary values the round-trip is exact.
    #[test]
    fn round_trip_property_all_2trit_ternary_values() {
        let all_trits = [Trit::Neg, Trit::Zero, Trit::Pos];
        for t1 in all_trits {
            for t2 in all_trits {
                let v = ternary_value(&[t1, t2]);
                let recovered = from_json(&to_json(&v).expect("to_json: finite test value"))
                    .expect("round-trip");
                assert_eq!(
                    v.content_hash(),
                    recovered.content_hash(),
                    "2-trit round-trip failed for ({t1:?}, {t2:?})"
                );
            }
        }
    }

    /// Property (display_bounded bound): for every budget in 0..=len+2, if budget < len then
    /// truncation is Elided and omitted == len - budget; if budget >= len then Complete.
    ///
    /// This is the property test for the `display_bounded` guarantee: "omitted == total -
    /// rendered_count, and truncation is Complete iff rendered_count == total" (spec §4).
    ///
    /// Mutation witness: return Complete for budget < len -> second branch fires.
    #[test]
    fn display_bounded_property_omitted_count_equals_total_minus_budget() {
        let bits: Vec<bool> = (0..8).map(|i| i % 2 == 0).collect();
        let v = binary_value(&bits);
        let total = 8usize;

        for budget in 0..=total + 2 {
            let r = display_bounded(&v, Budget(budget));
            if budget >= total {
                assert_eq!(
                    r.truncation,
                    Truncation::Complete,
                    "budget={budget} >= {total}: must be Complete"
                );
            } else {
                match &r.truncation {
                    Truncation::Elided { omitted, .. } => {
                        assert_eq!(
                            *omitted,
                            total - budget,
                            "budget={budget}: omitted must equal {total}-{budget}={}",
                            total - budget
                        );
                    }
                    Truncation::Complete => {
                        panic!("budget={budget} < {total}: expected Elided but got Complete")
                    }
                }
            }
        }
    }

    /// Property: the elision marker is always embedded in the rendered text when elided.
    /// This makes the output self-describing without inspecting the Truncation variant (C3).
    ///
    /// Mutation witness: omit the marker from the text -> assertion fails.
    #[test]
    fn display_bounded_elided_marker_is_in_text() {
        let v = dense_value(&[1.0, 2.0, 3.0, 4.0]);
        let r = display_bounded(&v, Budget(2));
        if let Truncation::Elided { marker, .. } = &r.truncation {
            assert!(
                r.text.as_str().contains(marker.as_str()),
                "elision marker must appear in the rendered text; text={:?}, marker={:?}",
                r.text.as_str(),
                marker
            );
        } else {
            panic!("expected Elided for budget=2 < dim=4");
        }
    }
}
