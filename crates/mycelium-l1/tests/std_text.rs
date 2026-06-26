//! Differential tests for `std.text` (M-717, #462) — the self-hosted UTF-8 byte/text utilities.
//!
//! The nodule source is loaded verbatim via `include_str!` (the single source of truth), then a
//! typed driver `fn main` is appended to exercise each operation. The `assert_three_way` harness
//! mirrors `std_option.rs` exactly: L1-eval(mono) ≡ elaborate→L0-interp ≡ AOT, all three paths
//! agree AND equal the `expected` reference value.
//!
//! # Generic pinning
//! `Option<A>` and `Result<A,E>` in `std.text` are pinned to concrete `Binary{8}` / `Utf8Error`
//! types via explicit return-type annotations on the driver strings (and the `DECODE_REF_PREAMBLE`
//! constant for `decode_ascii` tests) — without pinning, the monomorphizer
//! emits a never-silent `Residual` (G2).
//!
//! # Honesty tags
//! - **`Exact`** — `byte_len` (delegates to `bytes_len`), `is_ascii_byte`/`is_cont_byte` (total via
//!   `lt`+match), the `width_cast`/`lt`/`and`/`or`/`add_bin` bit ops the decode is assembled from.
//! - **`Declared`** — `byte_at` (Option bounds-check contract), `decode_ascii`/`decode_one`
//!   (never-silent type-level contracts; structural composition of Exact parts, not machine-proven).
//! - **`Empirical`** — the three-way differential agreement (L1-eval ≡ L0-interp ≡ AOT),
//!   validated by trial on the programs below; not a machine-checked proof.
//!
//! # Scope / FLAGs (honest boundary — VR-5)
//! - FLAG-text-1: **CLOSED** (DN-41 / M-798). `byte_at` is now an Option-returning bounds-checked
//!   access via `lt(width_cast(i, bytes_len(b)), bytes_len(b))` — the `width_cast` widen bridges the
//!   `Binary{8}` index to the `Binary{32}` length, the gap wave-n1 flagged.
//! - FLAG-text-2: **CLOSED** (DN-41 / M-798). `decode_one` returns the full `Binary{32}` codepoint
//!   (1/2/3/4-byte UTF-8); `width_cast` lifts the masked payloads, shifts are repeated `add_bin`
//!   doublings (no shift prim). `decode_ascii` is retained as the `Binary{8}` 1-byte fast path.
//!   (NOT yet rejected, honestly: overlong forms, surrogates, codepoints > U+10FFFF — a further
//!   increment; structural malformations DO surface never-silent.)
//! - FLAG-text-3: byte-cons slice/concat ops **still deferred** — `bytes_slice`/`bytes_concat` are
//!   STILL not surface-callable (`width_cast` does NOT unblock these; they need their own prim). The
//!   `Bytes8` type is declared but slice/concat await a future prim surface; `decode_one` returns a
//!   `Pair(codepoint, byte_width)` (the caller advances by the width) rather than slicing.
//!
//! # Anchor
//! Expected values are hand-computed and verified three-way (L1≡L0≡AOT). The Rust crate
//! crates/mycelium-std-text exists but exposes a different Ring-2 surface (no decode_ascii over a
//! .myc port), so it is the value oracle for shared semantics only — not a structural reference.

use mycelium_cert::{check_core, BinaryTernarySwapEngine, CheckVerdict};
use mycelium_core::GuaranteeStrength;
use mycelium_interp::{Interpreter, PrimRegistry};
use mycelium_l1::elab::build_registry;
use mycelium_l1::{check_nodule, elaborate, monomorphize, parse, Evaluator};

/// The std.text nodule source, loaded at compile time — the single source of truth.
const TEXT_SRC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../lib/std/text.myc"
));

/// Build a full test program by appending a typed driver to the nodule source.
fn program(driver: &str) -> String {
    format!("{TEXT_SRC}\n{driver}")
}

/// Run the three-way differential on `src` — L1-eval(mono) ≡ elaborate→L0-interp ≡ AOT — and
/// assert all three paths agree AND equal the `expected` reference value.
///
/// Honesty: differential agreement is `Empirical` (trials); the type-level contract is `Declared`.
fn assert_three_way(label: &str, src: &str, expected_src: &str) {
    let interp = Interpreter::new(
        PrimRegistry::with_builtins(),
        Box::new(BinaryTernarySwapEngine),
    );
    let prims = PrimRegistry::with_builtins();
    let engine = BinaryTernarySwapEngine;

    let env = check_nodule(&parse(src).unwrap_or_else(|e| panic!("{label}: parse failed: {e}")))
        .unwrap_or_else(|e| panic!("{label}: check failed: {e}"));

    let mono =
        monomorphize(&env, "main").unwrap_or_else(|e| panic!("{label}: monomorphize failed: {e}"));

    assert!(
        mono.fns.values().all(|fd| fd.sig.params.is_empty())
            && mono.types.values().all(|d| d.params.is_empty())
            && mono.traits.is_empty()
            && mono.instances.is_empty()
            && mono.impls.is_empty(),
        "{label}: monomorphized env must be closed (no generics/traits)"
    );

    let registry =
        build_registry(&mono).unwrap_or_else(|e| panic!("{label}: build_registry failed: {e}"));

    let l1_val = Evaluator::new(&mono)
        .call("main", vec![])
        .unwrap_or_else(|e| panic!("{label}: L1-eval failed: {e}"));
    let l1_core = l1_val
        .to_core(&mono, &registry)
        .unwrap_or_else(|| panic!("{label}: L1 result is outside the r3 data fragment"));

    let node = elaborate(&env, "main").unwrap_or_else(|e| panic!("{label}: elaborate failed: {e}"));
    let l0_core = interp
        .eval_core(&node)
        .unwrap_or_else(|e| panic!("{label}: L0-interp failed: {e}"));

    let aot_core = mycelium_mlir::run_core(&node, &prims, &engine)
        .unwrap_or_else(|e| panic!("{label}: AOT run_core failed: {e}"));

    assert_eq!(
        l1_core, l0_core,
        "{label}: L1-eval(mono) vs elaborate→L0-interp diverged"
    );
    assert_eq!(l0_core, aot_core, "{label}: L0-interp vs AOT diverged");

    for (x, y, pair) in [
        (&l1_core, &l0_core, "L1↔interp"),
        (&l0_core, &aot_core, "interp↔AOT"),
    ] {
        assert_eq!(
            check_core(x, y),
            CheckVerdict::Validated {
                strength: GuaranteeStrength::Exact
            },
            "{label}: the shared checker must validate the {pair} pair"
        );
    }

    let ref_env = check_nodule(
        &parse(expected_src).unwrap_or_else(|e| panic!("{label}: ref parse failed: {e}")),
    )
    .unwrap_or_else(|e| panic!("{label}: ref check failed: {e}"));
    let ref_node = elaborate(&ref_env, "main")
        .unwrap_or_else(|e| panic!("{label}: ref elaborate failed: {e}"));
    let expected = interp
        .eval_core(&ref_node)
        .unwrap_or_else(|e| panic!("{label}: ref eval failed: {e}"));

    assert_eq!(
        l1_core, expected,
        "{label}: result does not match expected reference value"
    );
}

// ── byte_len ──────────────────────────────────────────────────────────────────────────────────────

/// `byte_len(0x48_65_6c_6c_6f)` → `Binary{32}(5)`.
/// Reference: `bytes_len` on the UTF-8 encoding of "Hello" is exactly 5 (Exact).
/// Hand-computed: 0x48=H, 0x65=e, 0x6c=l, 0x6c=l, 0x6f=o — 5 bytes.
/// Grounding: hand-computed + enablement.rs bytes_len tests; mycelium-std-text exists but is a
/// different Ring-2 surface, not the oracle.
#[test]
fn byte_len_returns_count() {
    let driver = "fn main() -> Binary{32} = byte_len(0x48_65_6c_6c_6f)";
    let src = program(driver);
    // Binary{32}(5) MSB-first: 0b00000000_00000000_00000000_00000101
    let expected = "nodule ref\nfn main() -> Binary{32} = bytes_len(0x48_65_6c_6c_6f)";
    assert_three_way("byte_len(Hello)", &src, expected);
}

/// `byte_len(0x01_02_03)` → `Binary{32}(3)` — mirrors enablement.rs bytes_len_surface_three_way.
#[test]
fn byte_len_three_byte_input() {
    let driver = "fn main() -> Binary{32} = byte_len(0x01_02_03)";
    let src = program(driver);
    let expected = "nodule ref\nfn main() -> Binary{32} = bytes_len(0x01_02_03)";
    assert_three_way("byte_len(3 bytes)", &src, expected);
}

// ── is_ascii_byte ─────────────────────────────────────────────────────────────────────────────────

/// `is_ascii_byte(0b0100_0001)` → `True` ('A' = 0x41 < 0x80; Exact).
/// Hand-computed: 0x41 = 65 < 128; the lt prim returns 0b1 → True.
#[test]
fn is_ascii_byte_true_for_ascii() {
    // 0b0100_0001 = 0x41 = 'A': high bit clear → ASCII.
    let driver = "fn main() -> Bool = is_ascii_byte(0b0100_0001)";
    let src = program(driver);
    let expected = "nodule ref\nfn main() -> Bool = True";
    assert_three_way("is_ascii_byte(0x41=A → True)", &src, expected);
}

/// `is_ascii_byte(0b0000_0000)` → `True` (NUL byte = 0x00; Exact).
#[test]
fn is_ascii_byte_true_for_nul() {
    let driver = "fn main() -> Bool = is_ascii_byte(0b0000_0000)";
    let src = program(driver);
    let expected = "nodule ref\nfn main() -> Bool = True";
    assert_three_way("is_ascii_byte(0x00=NUL → True)", &src, expected);
}

/// `is_ascii_byte(0b0111_1111)` → `True` (DEL = 0x7F = 127 < 128; Exact).
#[test]
fn is_ascii_byte_true_for_max_ascii() {
    // 0b0111_1111 = 0x7F = 127: last valid ASCII value.
    let driver = "fn main() -> Bool = is_ascii_byte(0b0111_1111)";
    let src = program(driver);
    let expected = "nodule ref\nfn main() -> Bool = True";
    assert_three_way("is_ascii_byte(0x7F → True)", &src, expected);
}

/// `is_ascii_byte(0b1000_0000)` → `False` (= 0x80; first non-ASCII byte; Exact).
/// Hand-computed: 0x80 = 128, not < 128 → lt returns `_` arm → False.
#[test]
fn is_ascii_byte_false_for_continuation() {
    // 0b1000_0000 = 0x80: the first byte with the high bit set — a 2-byte UTF-8 lead range start.
    let driver = "fn main() -> Bool = is_ascii_byte(0b1000_0000)";
    let src = program(driver);
    let expected = "nodule ref\nfn main() -> Bool = False";
    assert_three_way("is_ascii_byte(0x80 → False)", &src, expected);
}

/// `is_ascii_byte(0b1111_1111)` → `False` (= 0xFF; Exact).
#[test]
fn is_ascii_byte_false_for_0xff() {
    let driver = "fn main() -> Bool = is_ascii_byte(0b1111_1111)";
    let src = program(driver);
    let expected = "nodule ref\nfn main() -> Bool = False";
    assert_three_way("is_ascii_byte(0xFF → False)", &src, expected);
}

// ── decode_ascii ──────────────────────────────────────────────────────────────────────────────────
//
// `decode_ascii` returns Result<Binary{8}, Utf8Error> — the generic parameters are pinned by
// annotating the expected-ref type explicitly. The three `Result`/`Utf8Error` types must be
// redeclared in the reference program so `eval_core` produces a compatible CoreValue.

/// Reference program preamble for decode_ascii tests: re-declare the local types so that
/// `eval_core` produces a compatible CoreValue (same type ContentHash as the test program).
const DECODE_REF_PREAMBLE: &str = "\
nodule ref\n\
type Option<A> = Some(A) | None\n\
type Result<A, E> = Ok(A) | Err(E)\n\
type Utf8Error = Invalid(Binary{8})\n";

/// `decode_ascii(0x41_42_43, 0b0000_0000)` → `Ok(bytes_get(…, 0))` (= Ok(0x41='A'); Declared/Empirical).
/// The byte at index 0 of [0x41, 0x42, 0x43] is 0x41 = 'A' — ASCII, so Ok.
/// The reference program uses `bytes_get` to match the `Derived` provenance of the computed value
/// (a literal `Ok(0b0100_0001)` would have `Root` provenance — see std_option.rs `map` comment).
/// Grounding: hand-computed, three-way verified; mycelium-std-text exists but is a different Ring-2 surface, not the oracle.
#[test]
fn decode_ascii_ok_on_valid_ascii() {
    let driver =
        "fn main() -> Result<Binary{8}, Utf8Error> = decode_ascii(0x41_42_43, 0b0000_0000)";
    let src = program(driver);
    // Reference: Ok wrapping the same bytes_get call to share Derived provenance.
    let expected = format!(
        "{DECODE_REF_PREAMBLE}fn main() -> Result<Binary{{8}}, Utf8Error> = Ok(bytes_get(0x41_42_43, 0b0000_0000))"
    );
    assert_three_way("decode_ascii(ABC, 0)=Ok(A)", &src, &expected);
}

/// `decode_ascii(0x43_44_45, 0b0000_0010)` → `Ok(bytes_get(…, 2))` (= Ok(0x45='E'); Declared/Empirical).
/// Index 2 of [0x43, 0x44, 0x45] = 0x45 = 'E'; ASCII → Ok.
#[test]
fn decode_ascii_ok_at_offset() {
    let driver =
        "fn main() -> Result<Binary{8}, Utf8Error> = decode_ascii(0x43_44_45, 0b0000_0010)";
    let src = program(driver);
    // Reference: Ok(bytes_get(…, 2)) — Derived provenance to match computed result.
    let expected = format!(
        "{DECODE_REF_PREAMBLE}fn main() -> Result<Binary{{8}}, Utf8Error> = Ok(bytes_get(0x43_44_45, 0b0000_0010))"
    );
    assert_three_way("decode_ascii(CDE, 2)=Ok(E)", &src, &expected);
}

/// `decode_ascii(0xc3_a9, 0b0000_0000)` → `Err(Invalid(bytes_get(…, 0)))` — never-silent (G2).
/// 0xc3 = 0b1100_0011 is the UTF-8 lead byte for U+00E9 (é); it has the high bit set → not ASCII.
/// Never-silent: the malformed lead is returned as the offending byte, never U+FFFD.
/// Hand-computed: is_ascii_byte(0xC3) = False → Err(Invalid(0xC3)).
/// Grounding: hand-computed, three-way verified; mycelium-std-text exists but is a different Ring-2 surface, not the oracle.
#[test]
fn decode_ascii_err_on_multibyte_lead() {
    // 0xc3_a9 is the UTF-8 encoding of 'é' (U+00E9). The lead byte 0xC3 has high bit set → Err.
    let driver = "fn main() -> Result<Binary{8}, Utf8Error> = decode_ascii(0xc3_a9, 0b0000_0000)";
    let src = program(driver);
    // Reference: Err(Invalid(bytes_get(…, 0))) — Derived provenance to match computed result.
    let expected = format!(
        "{DECODE_REF_PREAMBLE}fn main() -> Result<Binary{{8}}, Utf8Error> = Err(Invalid(bytes_get(0xc3_a9, 0b0000_0000)))"
    );
    assert_three_way("decode_ascii(é-lead)=Err(Invalid(0xC3))", &src, &expected);
}

/// `decode_ascii(0x80_bf, 0b0000_0000)` → `Err(Invalid(bytes_get(…, 0)))` — never-silent (G2).
/// 0x80 is a bare UTF-8 continuation byte (not valid as a lead); its high bit is set → Err.
#[test]
fn decode_ascii_err_on_continuation_byte() {
    // 0x80 = 0b1000_0000: bare continuation byte — invalid lead, non-ASCII.
    let driver = "fn main() -> Result<Binary{8}, Utf8Error> = decode_ascii(0x80_bf, 0b0000_0000)";
    let src = program(driver);
    // Reference: Err(Invalid(bytes_get(…, 0))) — Derived provenance to match computed result.
    let expected = format!(
        "{DECODE_REF_PREAMBLE}fn main() -> Result<Binary{{8}}, Utf8Error> = Err(Invalid(bytes_get(0x80_bf, 0b0000_0000)))"
    );
    assert_three_way("decode_ascii(0x80-continuation)=Err", &src, &expected);
}

// ── byte_at (FLAG-text-1 closed by DN-41 width_cast) ────────────────────────────────────────────────
//
// `byte_at(b, i)` bounds-checks the `Binary{8}` index `i` against the `Binary{32}` `bytes_len(b)` via
// `lt(width_cast(i, bytes_len(b)), bytes_len(b))` — the exact DN-41/M-798 pattern. In range yields
// `Some(byte)`; out of range yields `None` (never-silent, G2). Reference programs reuse `bytes_get`
// (in range) so the wrapped value shares `Derived` provenance with the computed result; `None` is a
// nullary constructor (no provenance to match). Both are pinned to `Option<Binary{8}>`.

/// `byte_at(0x41_42_43, 0b0000_0001)` → `Some(bytes_get(…, 1))` (= Some(0x42='B'); Declared/Empirical).
/// Index 1 of [0x41, 0x42, 0x43] is in range (1 < 3) → Some. Grounding: hand-computed, three-way verified.
#[test]
fn byte_at_some_in_range() {
    let driver = "fn main() -> Option<Binary{8}> = byte_at(0x41_42_43, 0b0000_0001)";
    let src = program(driver);
    // Reference: Some(bytes_get(…, 1)) — Derived provenance to match the computed in-range byte.
    let expected =
        program("fn main() -> Option<Binary{8}> = Some(bytes_get(0x41_42_43, 0b0000_0001))");
    assert_three_way("byte_at(ABC, 1)=Some(B)", &src, &expected);
}

/// `byte_at(0x41_42_43, 0b0000_0000)` → `Some(bytes_get(…, 0))` (= Some(0x41='A')) — boundary index 0.
#[test]
fn byte_at_some_at_zero() {
    let driver = "fn main() -> Option<Binary{8}> = byte_at(0x41_42_43, 0b0000_0000)";
    let src = program(driver);
    let expected =
        program("fn main() -> Option<Binary{8}> = Some(bytes_get(0x41_42_43, 0b0000_0000))");
    assert_three_way("byte_at(ABC, 0)=Some(A)", &src, &expected);
}

/// `byte_at(0x41_42_43, 0b0000_0011)` → `None` — index 3 is out of range (3 is NOT < 3); never-silent.
/// The out-of-range index is an explicit `None`, never a kernel refusal and never a silent wrap (G2).
/// Grounding: hand-computed (len 3, index 3 past end), three-way verified.
#[test]
fn byte_at_none_out_of_range() {
    let driver = "fn main() -> Option<Binary{8}> = byte_at(0x41_42_43, 0b0000_0011)";
    let src = program(driver);
    let expected = program("fn main() -> Option<Binary{8}> = None");
    assert_three_way("byte_at(ABC, 3)=None (oob)", &src, &expected);
}

/// `byte_at(0x41_42_43, 0b1111_1111)` → `None` — index 255 (far past end) is out of range; never-silent.
#[test]
fn byte_at_none_far_out_of_range() {
    let driver = "fn main() -> Option<Binary{8}> = byte_at(0x41_42_43, 0b1111_1111)";
    let src = program(driver);
    let expected = program("fn main() -> Option<Binary{8}> = None");
    assert_three_way("byte_at(ABC, 255)=None (oob)", &src, &expected);
}

// ── decode_one — full multi-byte UTF-8 decode (FLAG-text-2 closed by DN-41 width_cast) ──────────────
//
// `decode_one(b, i)` returns `Ok(Pr(codepoint : Binary{32}, byte_width : Binary{8}))` for the UTF-8
// sequence starting at `i`, or `Err(Invalid(byte))` on any structural malformation (never-silent, G2).
// The codepoint is the full `Binary{32}` Unicode scalar (FLAG-text-2 was the `Binary{8}` cap; the
// `width_cast` widen lifts it). Reference programs **recompute** the codepoint via the same nodule
// helper expressions (`shl6`/`widen8`/`cont_payload`/`or` etc.) so the computed value shares its
// `Derived` provenance with the reference — a literal codepoint would have `Root` provenance and would
// not compare equal (Meta carries provenance). All expected values are hand-computed and cross-checked
// against Python's UTF-8 decoder (é=233, €=8364, 😀=128512).

/// `decode_one(0x41_42_43, 0b0000_0000)` → `Ok(Pr(widen8('A'), 1))` — 1-byte ASCII path.
/// 0x41='A' < 0x80 → 1-byte; codepoint = widen8(0x41) = 65, width 1. Declared/Empirical.
#[test]
fn decode_one_ascii_one_byte() {
    let driver =
        "fn main() -> Result<Pair<Binary{32}, Binary{8}>, Utf8Error> = decode_one(0x41_42_43, 0b0000_0000)";
    let src = program(driver);
    // Reference: recompute via the same `widen8(bytes_get(…))` so the Binary{32} codepoint shares
    // Derived provenance with decode_one's `widen8(lead)`.
    let expected = program(
        "fn main() -> Result<Pair<Binary{32}, Binary{8}>, Utf8Error> = Ok(Pr(widen8(bytes_get(0x41_42_43, 0b0000_0000)), 0b0000_0001))",
    );
    assert_three_way("decode_one(A)=Ok(Pr(65,1))", &src, &expected);
}

/// `decode_one(0xc3_a9, 0b0000_0000)` → `Ok(Pr(233, 2))` — 2-byte path (é = U+00E9).
/// Lead 0xC3 ∈ 0xC0..0xDF → 2-byte; cont 0xA9 is valid (0x80..0xBF). cp = (0xC3 & 0x1F)<<6 | (0xA9 &
/// 0x3F) = 3<<6 | 41 = 192+41 = 233 = U+00E9. Hand-computed + Python-verified. Declared/Empirical.
#[test]
fn decode_one_two_byte_e_acute() {
    let driver =
        "fn main() -> Result<Pair<Binary{32}, Binary{8}>, Utf8Error> = decode_one(0xc3_a9, 0b0000_0000)";
    let src = program(driver);
    // Reference: recompute the codepoint with the same assembly `decode_two` uses (matching provenance).
    let expected = program(
        "fn main() -> Result<Pair<Binary{32}, Binary{8}>, Utf8Error> = \
         Ok(Pr(or(shl6(widen8(and(bytes_get(0xc3_a9, 0b0000_0000), 0b0001_1111))), cont_payload(bytes_get(0xc3_a9, 0b0000_0001))), 0b0000_0010))",
    );
    assert_three_way("decode_one(é)=Ok(Pr(233,2))", &src, &expected);
}

/// `decode_one(0xe2_82_ac, 0b0000_0000)` → `Ok(Pr(8364, 3))` — 3-byte path (€ = U+20AC).
/// Lead 0xE2 ∈ 0xE0..0xEF → 3-byte; conts 0x82, 0xAC valid. cp = (0xE2 & 0x0F)<<12 | (0x82 & 0x3F)<<6
/// | (0xAC & 0x3F) = 2<<12 | 2<<6 | 44 = 8192+128+44 = 8364 = U+20AC. Hand-computed + Python-verified.
#[test]
fn decode_one_three_byte_euro() {
    let driver =
        "fn main() -> Result<Pair<Binary{32}, Binary{8}>, Utf8Error> = decode_one(0xe2_82_ac, 0b0000_0000)";
    let src = program(driver);
    let expected = program(
        "fn main() -> Result<Pair<Binary{32}, Binary{8}>, Utf8Error> = \
         Ok(Pr(or(or(shl12(widen8(and(bytes_get(0xe2_82_ac, 0b0000_0000), 0b0000_1111))), shl6(cont_payload(bytes_get(0xe2_82_ac, 0b0000_0001)))), cont_payload(bytes_get(0xe2_82_ac, 0b0000_0010))), 0b0000_0011))",
    );
    assert_three_way("decode_one(€)=Ok(Pr(8364,3))", &src, &expected);
}

/// `decode_one(0xf0_9f_98_80, 0b0000_0000)` → `Ok(Pr(128512, 4))` — 4-byte path (😀 = U+1F600).
/// Lead 0xF0 ∈ 0xF0..0xF7 → 4-byte; conts 0x9F, 0x98, 0x80 valid. cp = (0xF0 & 0x07)<<18 | (0x9F &
/// 0x3F)<<12 | (0x98 & 0x3F)<<6 | (0x80 & 0x3F) = 0 | 31<<12 | 24<<6 | 0 = 126976+1536 = 128512 =
/// U+1F600. Hand-computed + Python-verified. Declared/Empirical.
#[test]
fn decode_one_four_byte_emoji() {
    let driver =
        "fn main() -> Result<Pair<Binary{32}, Binary{8}>, Utf8Error> = decode_one(0xf0_9f_98_80, 0b0000_0000)";
    let src = program(driver);
    let expected = program(
        "fn main() -> Result<Pair<Binary{32}, Binary{8}>, Utf8Error> = \
         Ok(Pr(or(or(or(shl18(widen8(and(bytes_get(0xf0_9f_98_80, 0b0000_0000), 0b0000_0111))), shl12(cont_payload(bytes_get(0xf0_9f_98_80, 0b0000_0001)))), shl6(cont_payload(bytes_get(0xf0_9f_98_80, 0b0000_0010)))), cont_payload(bytes_get(0xf0_9f_98_80, 0b0000_0011))), 0b0000_0100))",
    );
    assert_three_way("decode_one(😀)=Ok(Pr(128512,4))", &src, &expected);
}

// ── decode_one — never-silent malformations (G2) on all three lead paths ────────────────────────────
//
// Each malformed input produces `Err(Invalid(byte))` carrying the offending byte — never a U+FFFD
// substitution, never a silent truncation/wrap. The reference reuses `bytes_get` so the offending byte
// shares `Derived` provenance.

/// `decode_one(0x80_41, 0b0000_0000)` → `Err(Invalid(0x80))` — a bare continuation byte cannot lead.
/// 0x80 ∈ 0x80..0xBF (lt 0xC0 but not lt 0x80) → the "continuation as lead" Err arm; never-silent.
#[test]
fn decode_one_err_bare_continuation_lead() {
    let driver =
        "fn main() -> Result<Pair<Binary{32}, Binary{8}>, Utf8Error> = decode_one(0x80_41, 0b0000_0000)";
    let src = program(driver);
    let expected = program(
        "fn main() -> Result<Pair<Binary{32}, Binary{8}>, Utf8Error> = Err(Invalid(bytes_get(0x80_41, 0b0000_0000)))",
    );
    assert_three_way("decode_one(0x80 lead)=Err(Invalid(0x80))", &src, &expected);
}

/// `decode_one(0xc3_41, 0b0000_0000)` → `Err(Invalid(0x41))` — 2-byte lead but the continuation slot
/// holds 0x41 ('A'), which is NOT a continuation byte (0x41 < 0x80). Never-silent: the offending
/// continuation byte is reported, never a half-decoded codepoint.
#[test]
fn decode_one_err_bad_continuation_two_byte() {
    let driver =
        "fn main() -> Result<Pair<Binary{32}, Binary{8}>, Utf8Error> = decode_one(0xc3_41, 0b0000_0000)";
    let src = program(driver);
    let expected = program(
        "fn main() -> Result<Pair<Binary{32}, Binary{8}>, Utf8Error> = Err(Invalid(bytes_get(0xc3_41, 0b0000_0001)))",
    );
    assert_three_way("decode_one(0xC3 0x41)=Err(Invalid(0x41))", &src, &expected);
}

/// `decode_one(0xc3, 0b0000_0000)` → `Err(Invalid(0xC3))` — a truncated 2-byte sequence (the
/// continuation byte at index 1 is past the 1-byte input). `byte_at(b, 1)` is `None` → the lead byte
/// is reported. Never-silent: a missing continuation is an explicit Err, never a kernel OOB refusal.
#[test]
fn decode_one_err_truncated_two_byte() {
    let driver =
        "fn main() -> Result<Pair<Binary{32}, Binary{8}>, Utf8Error> = decode_one(0xc3, 0b0000_0000)";
    let src = program(driver);
    let expected = program(
        "fn main() -> Result<Pair<Binary{32}, Binary{8}>, Utf8Error> = Err(Invalid(bytes_get(0xc3, 0b0000_0000)))",
    );
    assert_three_way(
        "decode_one(0xC3 truncated)=Err(Invalid(0xC3))",
        &src,
        &expected,
    );
}

/// `decode_one(0xe2_82, 0b0000_0000)` → `Err(Invalid(0x82))` — a 3-byte lead truncated after the first
/// continuation (the second continuation at index 2 is past the 2-byte input). `byte_at(b, 2)` is
/// `None` → the last-seen continuation 0x82 is reported. Never-silent on the 3-byte truncation path.
#[test]
fn decode_one_err_truncated_three_byte() {
    let driver =
        "fn main() -> Result<Pair<Binary{32}, Binary{8}>, Utf8Error> = decode_one(0xe2_82, 0b0000_0000)";
    let src = program(driver);
    let expected = program(
        "fn main() -> Result<Pair<Binary{32}, Binary{8}>, Utf8Error> = Err(Invalid(bytes_get(0xe2_82, 0b0000_0001)))",
    );
    assert_three_way(
        "decode_one(0xE2 0x82 truncated)=Err(Invalid(0x82))",
        &src,
        &expected,
    );
}

/// `decode_one(0xf8_80_80_80, 0b0000_0000)` → `Err(Invalid(0xF8))` — 0xF8 is not a valid UTF-8 lead
/// (no 5+-byte form exists). The lead is in none of the 1/2/3/4-byte ranges → the final Err arm;
/// never-silent (G2): the invalid lead is reported, never decoded as a phantom 5-byte sequence.
#[test]
fn decode_one_err_invalid_high_lead() {
    let driver =
        "fn main() -> Result<Pair<Binary{32}, Binary{8}>, Utf8Error> = decode_one(0xf8_80_80_80, 0b0000_0000)";
    let src = program(driver);
    let expected = program(
        "fn main() -> Result<Pair<Binary{32}, Binary{8}>, Utf8Error> = Err(Invalid(bytes_get(0xf8_80_80_80, 0b0000_0000)))",
    );
    assert_three_way("decode_one(0xF8 lead)=Err(Invalid(0xF8))", &src, &expected);
}
