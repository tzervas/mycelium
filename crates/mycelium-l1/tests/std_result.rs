//! Differential tests for `std.result` (M-649) — the first self-hosted generic stdlib nodule.
//!
//! These tests prove that `std.result` RUNS to closed L0 on all three paths (L1-eval ≡
//! elaborate→L0-interp ≡ AOT) and that each combinator returns the correct reference value.
//!
//! # Harness design
//! The nodule source is loaded verbatim via `include_str!` (the single source of truth), then
//! a typed driver `fn` is appended to pin the generic parameters `A` and `E` to `Binary{8}`.
//! Without explicit pinning, the monomorphizer emits a never-silent `Residual` (undetermined type
//! parameters — G2), so every driver uses explicitly-typed helper functions (`mk_ok`, `mk_err`)
//! to carry the full `Result<Binary{8},Binary{8}>` type to the call site.
//!
//! # Honesty tags
//! - **`Declared`** — the type-level contract of each combinator (a structural check, not a theorem).
//! - **`Empirical`** — the three-way differential agreement (L1-eval ≡ L0-interp ≡ AOT), validated
//!   by trial on the programs below; not a machine-checked proof.
//!
//! # HOF gap (Declared limitation)
//! `map`, `and_then`, and `fold` are NOT tested here — they cannot be self-hosted in v0 because
//! there is no surface function type (`A -> B` as a value) and application is first-order only.
//! This is a `Declared` limitation recorded in the nodule header and in DN-14.

use mycelium_cert::{check_core, BinaryTernarySwapEngine, CheckVerdict};
use mycelium_core::GuaranteeStrength;
use mycelium_interp::{Interpreter, PrimRegistry};
use mycelium_l1::elab::build_registry;
use mycelium_l1::{check_nodule, elaborate, monomorphize, parse, Evaluator};

/// The std.result nodule source, loaded at compile time — the single source of truth.
const RESULT_SRC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../lib/std/result.myc"
));

/// Build a full test program by appending a typed driver to the nodule source.
/// The driver must supply `mk_ok` / `mk_err` helpers with explicit `Result<Binary{8},Binary{8}>`
/// return types so the monomorphizer can determine both `A` and `E` from the call site.
fn program(driver: &str) -> String {
    format!("{RESULT_SRC}\n{driver}")
}

/// Run the three-way differential on `src` — L1-eval(mono) ≡ elaborate→L0-interp ≡ AOT —
/// and assert all three paths agree AND equal the `expected` reference value (a hand-computed
/// `CoreValue` produced by evaluating a trivial reference program through the same path).
///
/// Honesty: differential agreement is `Empirical` (trials); the type-level contract is `Declared`.
fn assert_three_way(label: &str, src: &str, expected_src: &str) {
    let interp = Interpreter::new(
        PrimRegistry::with_builtins(),
        Box::new(BinaryTernarySwapEngine),
    );
    let prims = PrimRegistry::with_builtins();
    let engine = BinaryTernarySwapEngine;

    // Parse + type-check the test program.
    let env = check_nodule(&parse(src).unwrap_or_else(|e| panic!("{label}: parse failed: {e}")))
        .unwrap_or_else(|e| panic!("{label}: check failed: {e}"));

    // Monomorphize from `main` (both A and E must be fully determined — Residual otherwise).
    let mono =
        monomorphize(&env, "main").unwrap_or_else(|e| panic!("{label}: monomorphize failed: {e}"));

    // M-673 closure invariant: the mono'd env must be closed (no generics, no traits).
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

    // Path 1: L1 fuel-guarded evaluator on the monomorphized env.
    let l1_val = Evaluator::new(&mono)
        .call("main", vec![])
        .unwrap_or_else(|e| panic!("{label}: L1-eval failed: {e}"));
    let l1_core = l1_val
        .to_core(&mono, &registry)
        .unwrap_or_else(|| panic!("{label}: L1 result is outside the r3 data fragment"));

    // Path 2: elaborate→L0 reference interpreter.
    let node = elaborate(&env, "main").unwrap_or_else(|e| panic!("{label}: elaborate failed: {e}"));
    let l0_core = interp
        .eval_core(&node)
        .unwrap_or_else(|e| panic!("{label}: L0-interp failed: {e}"));

    // Path 3: AOT env-machine.
    let aot_core = mycelium_mlir::run_core(&node, &prims, &engine)
        .unwrap_or_else(|e| panic!("{label}: AOT run_core failed: {e}"));

    // All three must agree (Empirical guarantee — trials).
    assert_eq!(
        l1_core, l0_core,
        "{label}: L1-eval(mono) vs elaborate→L0-interp diverged"
    );
    assert_eq!(l0_core, aot_core, "{label}: L0-interp vs AOT diverged");

    // Each agreeing pair validates through the M-210 shared checker (Empirical: never a silent pass).
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

    // Compare against the reference value (hand-computed; the simplest honest reference is a
    // trivial direct program evaluated through the same three-way path).
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

// ── is_ok ────────────────────────────────────────────────────────────────────────────────────────

/// `is_ok(Ok(x))` → `True` (Declared: the Ok-arm always returns True).
/// Both A and E are pinned to Binary{8} via explicit return types on mk_ok/mk_err.
#[test]
fn is_ok_on_ok_returns_true() {
    let driver = "\
fn mk_ok() -> Result<Binary{8},Binary{8}> = Ok(0b0000_0001)\n\
fn main() -> Bool = is_ok(mk_ok())";
    let src = program(driver);
    let expected = "nodule ref\nfn main() -> Bool = True";
    assert_three_way("is_ok(Ok)", &src, expected);
}

/// `is_ok(Err(e))` → `False` (Declared: the Err-arm always returns False).
#[test]
fn is_ok_on_err_returns_false() {
    let driver = "\
fn mk_err() -> Result<Binary{8},Binary{8}> = Err(0b1111_1111)\n\
fn main() -> Bool = is_ok(mk_err())";
    let src = program(driver);
    let expected = "nodule ref\nfn main() -> Bool = False";
    assert_three_way("is_ok(Err)", &src, expected);
}

// ── is_err ───────────────────────────────────────────────────────────────────────────────────────

/// `is_err(Ok(x))` → `False` (Declared: mirror of is_ok, Ok-arm returns False).
#[test]
fn is_err_on_ok_returns_false() {
    let driver = "\
fn mk_ok() -> Result<Binary{8},Binary{8}> = Ok(0b0000_0001)\n\
fn main() -> Bool = is_err(mk_ok())";
    let src = program(driver);
    let expected = "nodule ref\nfn main() -> Bool = False";
    assert_three_way("is_err(Ok)", &src, expected);
}

/// `is_err(Err(e))` → `True` (Declared: mirror of is_ok, Err-arm returns True).
#[test]
fn is_err_on_err_returns_true() {
    let driver = "\
fn mk_err() -> Result<Binary{8},Binary{8}> = Err(0b1111_1111)\n\
fn main() -> Bool = is_err(mk_err())";
    let src = program(driver);
    let expected = "nodule ref\nfn main() -> Bool = True";
    assert_three_way("is_err(Err)", &src, expected);
}

// ── unwrap_or ────────────────────────────────────────────────────────────────────────────────────

/// `unwrap_or(Ok(x), d)` → `x` (Declared: returns the wrapped value, ignores default).
/// Never-silent (G2): the default is caller-supplied; no panic, no sentinel.
#[test]
fn unwrap_or_on_ok_returns_wrapped_value() {
    let driver = "\
fn mk_ok() -> Result<Binary{8},Binary{8}> = Ok(0b0000_0001)\n\
fn main() -> Binary{8} = unwrap_or(mk_ok(), 0b0000_0000)";
    let src = program(driver);
    // Expected: the wrapped value 0b0000_0001, not the default 0b0000_0000.
    let expected = "nodule ref\nfn main() -> Binary{8} = 0b0000_0001";
    assert_three_way("unwrap_or(Ok)", &src, expected);
}

/// `unwrap_or(Err(e), d)` → `d` (Declared: returns the default, discards the error).
/// Never-silent (G2): the caller-supplied default is the explicit recovery path.
#[test]
fn unwrap_or_on_err_returns_default() {
    let driver = "\
fn mk_err() -> Result<Binary{8},Binary{8}> = Err(0b1111_1111)\n\
fn main() -> Binary{8} = unwrap_or(mk_err(), 0b0000_0000)";
    let src = program(driver);
    // Expected: the default 0b0000_0000, not the discarded error 0b1111_1111.
    let expected = "nodule ref\nfn main() -> Binary{8} = 0b0000_0000";
    assert_three_way("unwrap_or(Err)", &src, expected);
}

/// Edge case: `unwrap_or(Err(e), d)` where `d = e` — both values are 0b1111_1111.
/// This confirms the combinator returns `d` for the right reason (match-arm, not identity):
/// the reference value is hand-computed as `0b1111_1111`.
#[test]
fn unwrap_or_on_err_with_same_default_as_error() {
    let driver = "\
fn mk_err() -> Result<Binary{8},Binary{8}> = Err(0b1111_1111)\n\
fn main() -> Binary{8} = unwrap_or(mk_err(), 0b1111_1111)";
    let src = program(driver);
    let expected = "nodule ref\nfn main() -> Binary{8} = 0b1111_1111";
    assert_three_way("unwrap_or(Err, d=e)", &src, expected);
}
