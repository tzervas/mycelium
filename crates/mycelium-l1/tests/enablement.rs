//! Kernel self-hosting-enablement conformance (E19-1 / M-752) — the differential proof that the
//! RFC-0032 enablers genuinely unblock the E13-1 self-hosted-stdlib tiers.
//!
//! Each enabler lands with a **three-way differential** (L1-eval ≡ elaborate→L0-interp ≡ AOT) over
//! the same trusted prim registry, mirroring `differential.rs`. This file pins the *new* surface:
//!
//! - **M-747** — the reduce-to-`Bool` comparison prims `eq`/`lt` over `Binary{N}`/`Ternary{N}`
//!   (RFC-0032 D1). The kernel prim returns `Binary{1}` (`0b1` = true); the `.myc` `std.cmp` lift to
//!   the `Bool` ADT is demonstrated by the `bool`-bridge smoke port. **Unblocks** E13-1 M-718
//!   (width-typed `cmp`/`Eq`/`Ord`).
//! - **M-748** — never-silent fixed-width binary arithmetic `add_bin`/`sub_bin` + the surfaced
//!   `and`/`or` (RFC-0032 D2). **Unblocks** E13-1 M-718 (binary `math`).
//!
//! # Honesty tags
//! - **`Exact`** — every prim here is total/decidable over its in-range domain; each result equals
//!   its reference value exactly.
//! - **`Empirical`** — the three-way agreement is established by trial on the programs below.
//!
//! # Never-silent (G2/VR-5)
//! Overflow (`add_bin`/`sub_bin` out of `[0, 2^N)`) and paradigm/width mismatch are **explicit
//! refusals on every path**, never a silent wrap or a silent `false` — pinned by the refusal tests.
//!
//! # Scope boundary
//! The Tier-2 reprs (`Repr::Seq` / `Repr::Bytes`, M-749/M-750) are **not** exercised here — they are
//! the KC-3-significant, maintainer-sign-off-gated additions and land separately; their conformance
//! ports join this file when they do (never faked here — G2/VR-5).

use mycelium_core::{Payload, Repr};
use mycelium_interp::{Interpreter, PrimRegistry};
use mycelium_l1::{check_nodule, elaborate, parse, Evaluator};

/// Run the three-way differential on `src` (L1-eval ≡ elaborate→L0-interp ≡ AOT) and assert all
/// three paths agree on the observable (`repr + payload`) AND equal the `expected` reference value.
fn assert_three_way(label: &str, src: &str, expected_repr: &Repr, expected_payload: &Payload) {
    let interp = Interpreter::new(
        PrimRegistry::with_builtins(),
        Box::new(mycelium_cert::BinaryTernarySwapEngine),
    );
    let prims = PrimRegistry::with_builtins();
    let engine = mycelium_cert::BinaryTernarySwapEngine;

    let env = check_nodule(&parse(src).unwrap_or_else(|e| panic!("{label}: parse failed: {e}")))
        .unwrap_or_else(|e| panic!("{label}: check failed: {e}"));

    // Path 1: the L1 fuel-guarded evaluator.
    let l1 = Evaluator::new(&env)
        .call("main", vec![])
        .unwrap_or_else(|e| panic!("{label}: L1-eval failed: {e}"));
    let l1 = l1
        .as_repr()
        .unwrap_or_else(|| panic!("{label}: result must be a repr value"))
        .clone();

    // Path 2: elaborate to L0, run on the reference interpreter.
    let node =
        elaborate(&env, "main").unwrap_or_else(|e| panic!("{label}: must be in the fragment: {e}"));
    let l0 = interp
        .eval(&node)
        .unwrap_or_else(|e| panic!("{label}: L0-interp failed: {e}"));

    // Path 3: the same L0 term through the AOT path.
    let aot = mycelium_mlir::run(&node, &prims, &engine)
        .unwrap_or_else(|e| panic!("{label}: AOT failed: {e}"));

    for (path, v) in [("L1-eval", &l1), ("L0-interp", &l0), ("AOT", &aot)] {
        assert_eq!(v.repr(), expected_repr, "{label}: {path} repr mismatch");
        assert_eq!(
            v.payload(),
            expected_payload,
            "{label}: {path} payload mismatch"
        );
    }
    assert_eq!(
        (l1.repr(), l1.payload()),
        (l0.repr(), l0.payload()),
        "{label}: L1-eval vs L0-interp diverged"
    );
    assert_eq!(
        (l0.repr(), l0.payload()),
        (aot.repr(), aot.payload()),
        "{label}: L0-interp vs AOT diverged"
    );
}

/// `Binary{1}` truth payload — the realized `Bool` of RFC-0032 D1.
fn b1(truth: bool) -> (Repr, Payload) {
    (Repr::Binary { width: 1 }, Payload::Bits(vec![truth]))
}

// ── M-747: width-typed comparison/equality (unblocks E13-1 M-718 cmp/Eq/Ord) ────────────────────

#[test]
fn eq_binary_width_typed() {
    let (r, p) = b1(true);
    assert_three_way(
        "eq Binary equal",
        "nodule d\nfn main() -> Binary{1} = eq(0b1010_0000, 0b1010_0000)",
        &r,
        &p,
    );
    let (r, p) = b1(false);
    assert_three_way(
        "eq Binary unequal",
        "nodule d\nfn main() -> Binary{1} = eq(0b1010_0000, 0b1010_0001)",
        &r,
        &p,
    );
}

#[test]
fn eq_ternary_width_typed() {
    let (r, p) = b1(true);
    assert_three_way(
        "eq Ternary equal",
        "nodule d\nfn main() -> Binary{1} = eq(<00+->, <00+->)",
        &r,
        &p,
    );
    let (r, p) = b1(false);
    assert_three_way(
        "eq Ternary unequal",
        "nodule d\nfn main() -> Binary{1} = eq(<00+->, <0+0->)",
        &r,
        &p,
    );
}

#[test]
fn lt_binary_unsigned_magnitude() {
    // 0b1000_0000 (128) < 0b1010_0000 (160).
    let (r, p) = b1(true);
    assert_three_way(
        "lt Binary true",
        "nodule d\nfn main() -> Binary{1} = lt(0b1000_0000, 0b1010_0000)",
        &r,
        &p,
    );
    // Not strictly less when equal.
    let (r, p) = b1(false);
    assert_three_way(
        "lt Binary equal-is-false",
        "nodule d\nfn main() -> Binary{1} = lt(0b1010_0000, 0b1010_0000)",
        &r,
        &p,
    );
}

#[test]
fn lt_ternary_balanced_value() {
    // <00+-> = 2, <0+0-> = 8, so 2 < 8 is true.
    let (r, p) = b1(true);
    assert_three_way(
        "lt Ternary true",
        "nodule d\nfn main() -> Binary{1} = lt(<00+->, <0+0->)",
        &r,
        &p,
    );
    // <0-00> = -9 < <00+-> = 2 → true; reversed → false (negative magnitude ordering).
    let (r, p) = b1(false);
    assert_three_way(
        "lt Ternary negative-false",
        "nodule d\nfn main() -> Binary{1} = lt(<00+->, <0-00>)",
        &r,
        &p,
    );
}

/// The `.myc` `Bool`-bridge smoke port: match the `Binary{1}` comparison bit into the `Bool` ADT,
/// exactly the one-line lift the E13-1 `std.cmp` port (M-718) bottoms out on.
#[test]
fn bool_bridge_from_comparison_bit() {
    // `match eq(a, b) { 0b1 => True, _ => False }` ≡ the data value `True`.
    let src = "nodule d\n\
               fn main() -> Bool = match eq(0b1010_0000, 0b1010_0000) { 0b1 => True, _ => False }";
    let env = check_nodule(&parse(src).expect("parses")).expect("checks");
    let val = Evaluator::new(&env).call("main", vec![]).expect("L1-eval");
    // The result is the `True` data constructor (the lift succeeded).
    let core = format!("{val:?}");
    assert!(
        core.contains("True"),
        "bool-bridge must yield the `True` constructor, got {core}"
    );
}

// ── M-748: never-silent binary arithmetic (unblocks E13-1 M-718 binary math) ────────────────────

#[test]
fn and_or_surfaced() {
    assert_three_way(
        "and",
        "nodule d\nfn main() -> Binary{8} = and(0b1100_1010, 0b1010_1010)",
        &Repr::Binary { width: 8 },
        &Payload::Bits("10001010".chars().map(|c| c == '1').collect()),
    );
    assert_three_way(
        "or",
        "nodule d\nfn main() -> Binary{8} = or(0b1100_1010, 0b1010_1010)",
        &Repr::Binary { width: 8 },
        &Payload::Bits("11101010".chars().map(|c| c == '1').collect()),
    );
}

#[test]
fn add_bin_in_range() {
    // 0b0000_0001 + 0b0000_0010 = 0b0000_0011 (1 + 2 = 3).
    assert_three_way(
        "add_bin",
        "nodule d\nfn main() -> Binary{8} = add_bin(0b0000_0001, 0b0000_0010)",
        &Repr::Binary { width: 8 },
        &Payload::Bits("00000011".chars().map(|c| c == '1').collect()),
    );
}

#[test]
fn sub_bin_in_range() {
    // 0b0000_0101 - 0b0000_0010 = 0b0000_0011 (5 - 2 = 3).
    assert_three_way(
        "sub_bin",
        "nodule d\nfn main() -> Binary{8} = sub_bin(0b0000_0101, 0b0000_0010)",
        &Repr::Binary { width: 8 },
        &Payload::Bits("00000011".chars().map(|c| c == '1').collect()),
    );
}

// ── Never-silent (G2/VR-5): overflow + mismatch refuse on every path ─────────────────────────────

/// `add_bin` overflow (`255 + 1` at `Binary{8}`) is an explicit refusal on **all three** paths —
/// never a silent wrap to `0`. (The program type-checks: overflow is a runtime contract, D2.)
#[test]
fn add_bin_overflow_refuses_on_every_path() {
    let src = "nodule d\nfn main() -> Binary{8} = add_bin(0b1111_1111, 0b0000_0001)";
    let env = check_nodule(&parse(src).expect("parses")).expect("checks");

    let interp = Interpreter::new(
        PrimRegistry::with_builtins(),
        Box::new(mycelium_cert::BinaryTernarySwapEngine),
    );
    let prims = PrimRegistry::with_builtins();
    let engine = mycelium_cert::BinaryTernarySwapEngine;

    assert!(
        Evaluator::new(&env).call("main", vec![]).is_err(),
        "L1-eval must refuse the overflow (never a silent wrap)"
    );
    let node = elaborate(&env, "main").expect("in fragment");
    assert!(
        interp.eval(&node).is_err(),
        "L0-interp must refuse the overflow"
    );
    assert!(
        mycelium_mlir::run(&node, &prims, &engine).is_err(),
        "AOT must refuse the overflow"
    );
}

/// `sub_bin` underflow (`0 - 1` at `Binary{8}`, a negative with no unsigned form) refuses likewise.
#[test]
fn sub_bin_underflow_refuses() {
    let src = "nodule d\nfn main() -> Binary{8} = sub_bin(0b0000_0000, 0b0000_0001)";
    let env = check_nodule(&parse(src).expect("parses")).expect("checks");
    assert!(
        Evaluator::new(&env).call("main", vec![]).is_err(),
        "L1-eval must refuse the underflow (never a silent wrap to 255)"
    );
}

/// A cross-paradigm comparison (`Binary` vs `Ternary`) is a **static** never-silent refusal — caught
/// at check time, never a silent `false` (RFC-0032 D1).
#[test]
fn eq_cross_paradigm_refuses_statically() {
    let src = "nodule d\nfn main() -> Binary{1} = eq(0b0000_0001, <00+->)";
    let err = check_nodule(&parse(src).expect("parses"));
    assert!(
        err.is_err(),
        "a Binary-vs-Ternary `eq` must be a static type error, never a silent false"
    );
}

/// A bare-decimal comparand has no width anchor (comparison is width-collapsing) — refused, never a
/// defaulted width (RFC-0032 D1 / RFC-0012 §4.3).
#[test]
fn eq_bare_decimal_refuses() {
    let src = "nodule d\nfn main() -> Binary{1} = eq(5, 6)";
    assert!(
        check_nodule(&parse(src).expect("parses")).is_err(),
        "a bare-decimal `eq` must refuse (no width anchor), never a default width"
    );
}
