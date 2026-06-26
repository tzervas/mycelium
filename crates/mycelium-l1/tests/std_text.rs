//! Differential tests for `std.text` (M-717, #462) — the self-hosted UTF-8 byte/text utilities.
//!
//! The nodule source is loaded verbatim via `include_str!` (the single source of truth), then a
//! typed driver `fn main` is appended to exercise each operation. The `assert_three_way` harness
//! mirrors `std_option.rs` exactly: L1-eval(mono) ≡ elaborate→L0-interp ≡ AOT, all three paths
//! agree AND equal the `expected` reference value.
//!
//! # Generic pinning
//! `Option<A>` and `Result<A,E>` in `std.text` are pinned to concrete `Binary{8}` / `Utf8Error`
//! types via explicitly-typed helpers (`mk_bytes`, etc.) — without pinning, the monomorphizer
//! emits a never-silent `Residual` (G2).
//!
//! # Honesty tags
//! - **`Exact`** — `byte_len` (delegates to `bytes_len`), `is_ascii_byte` (total via `lt`+match).
//! - **`Declared`** — `decode_ascii` (type-level contract; never-silent by construction).
//! - **`Empirical`** — the three-way differential agreement (L1-eval ≡ L0-interp ≡ AOT),
//!   validated by trial on the programs below; not a machine-checked proof.
//!
//! # Scope / FLAGs (honest boundary — VR-5)
//! - FLAG-text-1: `byte_at` (Option-returning bounds-checked access) not yet implemented —
//!   `bytes_len` returns `Binary{32}`, index is `Binary{8}`, `lt` is width-typed; no surface
//!   zero-extension prim available to perform the comparison.
//! - FLAG-text-2: `decode_one` returning `Binary{32}` codepoints deferred — same zero-extension
//!   blocker; `decode_ascii` returns `Binary{8}` (valid for U+0000–U+007F only).
//! - FLAG-text-3: byte-cons slice/concat ops deferred — `bytes_slice`/`bytes_concat` are not
//!   surface-callable; the `Bytes8` type is declared but slice/concat await a future prim surface.
//!
//! # Anchor
//! Expected values are grounded in the Rust reference: crates/mycelium-std-text (not yet landed).
//! Until that crate exists, the reference values are hand-computed and documented inline.

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
/// Anchor: crates/mycelium-std-text (not yet landed); grounded in enablement.rs bytes_len tests.
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
// re-declared in the reference program so `eval_core` produces a compatible CoreValue.

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
/// Anchor: crates/mycelium-std-text (not yet landed).
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
/// Anchor: crates/mycelium-std-text (not yet landed).
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
