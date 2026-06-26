//! Differential tests for `std.cmp` (M-715) — the self-hosted ordering/equality core surface.
//!
//! The nodule source is loaded verbatim via `include_str!` (the single source of truth), then a
//! driver `fn main` is appended. Every op in `std.cmp` is concrete (over the finite types `Bool` and
//! `Ordering`), so — unlike `std.option`/`std.result` — no generic pinning is needed; the driver just
//! calls the op directly.
//!
//! # Honesty tags
//! - **`Exact`** — every op is total over a finite domain and match-defined (no kernel comparison
//!   prim involved), so each result equals its reference exactly.
//! - **`Empirical`** — the three-way differential agreement (L1-eval ≡ L0-interp ≡ AOT), validated by
//!   trial on the programs below.
//!
//! # Scope boundary (RFC-0031 §5 D4)
//! Equality/ordering over the *width* types `Binary{N}`/`Ternary{N}` is intentionally absent: the
//! kernel surfaces `bit.not`/`bit.xor` + ternary arithmetic but no reduce-to-`Bool` comparison prim,
//! so a width-typed `eq`/`cmp` cannot be honestly self-hosted yet (it would have nothing to bottom
//! out on). That port lands when the comparison prim is surfaced — never faked here (G2/VR-5).

use mycelium_cert::{check_core, BinaryTernarySwapEngine, CheckVerdict};
use mycelium_core::GuaranteeStrength;
use mycelium_interp::{Interpreter, PrimRegistry};
use mycelium_l1::elab::build_registry;
use mycelium_l1::{check_nodule, elaborate, monomorphize, parse, Evaluator};

/// The std.cmp nodule source, loaded at compile time — the single source of truth.
const CMP_SRC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../lib/std/cmp.myc"
));

/// Build a full test program by appending a `main` driver to the nodule source.
fn program(driver: &str) -> String {
    format!("{CMP_SRC}\n{driver}")
}

/// Run the three-way differential on `src` — L1-eval(mono) ≡ elaborate→L0-interp ≡ AOT — and assert
/// all three paths agree AND equal the `expected` reference value.
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

/// A `main` returning `Bool`; `expected` is the reference Bool literal.
fn assert_bool(label: &str, call: &str, expected_bool: &str) {
    let src = program(&format!("fn main() -> Bool = {call}"));
    let expected = format!("nodule ref\nfn main() -> Bool = {expected_bool}");
    assert_three_way(label, &src, &expected);
}

/// A `main` returning `Ordering`; `expected` is the reference Ordering constructor.
fn assert_ordering(label: &str, call: &str, expected_ord: &str) {
    let src = program(&format!("fn main() -> Ordering = {call}"));
    // The reference program redeclares Ordering so the constructor resolves.
    let expected =
        format!("nodule ref\ntype Ordering = Lt | Eq | Gt\nfn main() -> Ordering = {expected_ord}");
    assert_three_way(label, &src, &expected);
}

// ── Ordering projections ─────────────────────────────────────────────────────────────────────────

#[test]
fn is_lt_projects_each_arm() {
    assert_bool("is_lt(Lt)", "is_lt(Lt)", "True");
    assert_bool("is_lt(Eq)", "is_lt(Eq)", "False");
    assert_bool("is_lt(Gt)", "is_lt(Gt)", "False");
}

#[test]
fn is_eq_projects_each_arm() {
    assert_bool("is_eq(Lt)", "is_eq(Lt)", "False");
    assert_bool("is_eq(Eq)", "is_eq(Eq)", "True");
    assert_bool("is_eq(Gt)", "is_eq(Gt)", "False");
}

#[test]
fn is_gt_projects_each_arm() {
    assert_bool("is_gt(Lt)", "is_gt(Lt)", "False");
    assert_bool("is_gt(Eq)", "is_gt(Eq)", "False");
    assert_bool("is_gt(Gt)", "is_gt(Gt)", "True");
}

// ── reverse (involution) ─────────────────────────────────────────────────────────────────────────

#[test]
fn reverse_swaps_lt_and_gt_fixes_eq() {
    assert_ordering("reverse(Lt)", "reverse(Lt)", "Gt");
    assert_ordering("reverse(Eq)", "reverse(Eq)", "Eq");
    assert_ordering("reverse(Gt)", "reverse(Gt)", "Lt");
}

/// `reverse` is an involution: `reverse(reverse(o)) == o` for each arm.
#[test]
fn reverse_is_an_involution() {
    assert_ordering("reverse2(Lt)", "reverse(reverse(Lt))", "Lt");
    assert_ordering("reverse2(Eq)", "reverse(reverse(Eq))", "Eq");
    assert_ordering("reverse2(Gt)", "reverse(reverse(Gt))", "Gt");
}

// ── bool_eq (structural equality on Bool) ────────────────────────────────────────────────────────

#[test]
fn bool_eq_truth_table() {
    assert_bool("bool_eq(T,T)", "bool_eq(True, True)", "True");
    assert_bool("bool_eq(T,F)", "bool_eq(True, False)", "False");
    assert_bool("bool_eq(F,T)", "bool_eq(False, True)", "False");
    assert_bool("bool_eq(F,F)", "bool_eq(False, False)", "True");
}

// ── bool_cmp (total order, False < True) ─────────────────────────────────────────────────────────

#[test]
fn bool_cmp_total_order() {
    assert_ordering("bool_cmp(F,F)", "bool_cmp(False, False)", "Eq");
    assert_ordering("bool_cmp(F,T)", "bool_cmp(False, True)", "Lt");
    assert_ordering("bool_cmp(T,F)", "bool_cmp(True, False)", "Gt");
    assert_ordering("bool_cmp(T,T)", "bool_cmp(True, True)", "Eq");
}

// ── ord_eq (structural equality on Ordering) ─────────────────────────────────────────────────────

#[test]
fn ord_eq_reflexive_on_each_arm() {
    assert_bool("ord_eq(Lt,Lt)", "ord_eq(Lt, Lt)", "True");
    assert_bool("ord_eq(Eq,Eq)", "ord_eq(Eq, Eq)", "True");
    assert_bool("ord_eq(Gt,Gt)", "ord_eq(Gt, Gt)", "True");
}

#[test]
fn ord_eq_distinguishes_arms() {
    assert_bool("ord_eq(Lt,Eq)", "ord_eq(Lt, Eq)", "False");
    assert_bool("ord_eq(Lt,Gt)", "ord_eq(Lt, Gt)", "False");
    assert_bool("ord_eq(Eq,Gt)", "ord_eq(Eq, Gt)", "False");
}

/// Cross-op consistency: `is_eq(bool_cmp(a,b))` agrees with `bool_eq(a,b)` on the diagonal.
#[test]
fn bool_cmp_eq_agrees_with_bool_eq() {
    assert_bool("consistency(T,T)", "is_eq(bool_cmp(True, True))", "True");
    assert_bool("consistency(T,F)", "is_eq(bool_cmp(True, False))", "False");
}

// ── Width-typed comparison helpers at Binary{8} ───────────────────────────────────────────────────
//
// cmp_u8/le_u8/ge_u8/max_u8/min_u8 wrap the `eq`/`lt` kernel prims into the Ordering surface at
// Binary{8}. These are Exact over the finite Binary{8} domain (eq/lt are Exact prims). Three-way
// agreement is Empirical (trials below). FLAG: width-generic versions (cmp<N>) await M-753.
//
// Test strategy: cover all three arms of cmp_u8 (Lt/Eq/Gt) and the edge cases (min/max with
// equal inputs; le/ge boundary). Values chosen as recognisable unsigned magnitudes: 0b0000_0001
// (1), 0b0000_0010 (2), 0b0000_0011 (3), 0b0000_0000 (0), 0b1111_1111 (255).

// ── cmp_u8 ───────────────────────────────────────────────────────────────────────────────────────

/// `cmp_u8(1, 2)` → `Lt` — 1 < 2 unsigned. Exact (eq/lt prims over Binary{8}).
/// Expected (hand-computed, three-way verified).
#[test]
fn cmp_u8_lt_arm() {
    assert_ordering("cmp_u8(1,2)", "cmp_u8(0b0000_0001, 0b0000_0010)", "Lt");
}

/// `cmp_u8(2, 2)` → `Eq` — equal values. Exact.
/// Expected (hand-computed, three-way verified).
#[test]
fn cmp_u8_eq_arm() {
    assert_ordering("cmp_u8(2,2)", "cmp_u8(0b0000_0010, 0b0000_0010)", "Eq");
}

/// `cmp_u8(3, 1)` → `Gt` — 3 > 1 unsigned. Exact.
/// Expected (hand-computed, three-way verified).
#[test]
fn cmp_u8_gt_arm() {
    assert_ordering("cmp_u8(3,1)", "cmp_u8(0b0000_0011, 0b0000_0001)", "Gt");
}

/// Edge: `cmp_u8(0, 255)` → `Lt` — minimum vs maximum unsigned. Exact.
/// Expected (hand-computed, three-way verified).
#[test]
fn cmp_u8_min_vs_max() {
    assert_ordering("cmp_u8(0,255)", "cmp_u8(0b0000_0000, 0b1111_1111)", "Lt");
}

/// `cmp_u8` involution under `reverse`: `reverse(cmp_u8(a,b)) == cmp_u8(b,a)` for a sample pair.
/// Hand-computed: cmp_u8(1,3) = Lt; reverse(Lt) = Gt; cmp_u8(3,1) = Gt. Empirical cross-op.
#[test]
fn cmp_u8_reverse_symmetry() {
    assert_ordering(
        "reverse(cmp_u8(1,3))",
        "reverse(cmp_u8(0b0000_0001, 0b0000_0011))",
        "Gt",
    );
}

// ── le_u8 / ge_u8 ────────────────────────────────────────────────────────────────────────────────

/// `le_u8(1, 2)` → `True` — strict less satisfies le. Exact.
#[test]
fn le_u8_strict_less() {
    assert_bool("le_u8(1,2)", "le_u8(0b0000_0001, 0b0000_0010)", "True");
}

/// `le_u8(2, 2)` → `True` — equal satisfies le. Exact.
#[test]
fn le_u8_equal() {
    assert_bool("le_u8(2,2)", "le_u8(0b0000_0010, 0b0000_0010)", "True");
}

/// `le_u8(3, 2)` → `False` — greater does not satisfy le. Exact.
#[test]
fn le_u8_greater_is_false() {
    assert_bool("le_u8(3,2)", "le_u8(0b0000_0011, 0b0000_0010)", "False");
}

/// `ge_u8(3, 2)` → `True` — strict greater satisfies ge. Exact.
#[test]
fn ge_u8_strict_greater() {
    assert_bool("ge_u8(3,2)", "ge_u8(0b0000_0011, 0b0000_0010)", "True");
}

/// `ge_u8(2, 2)` → `True` — equal satisfies ge. Exact.
#[test]
fn ge_u8_equal() {
    assert_bool("ge_u8(2,2)", "ge_u8(0b0000_0010, 0b0000_0010)", "True");
}

/// `ge_u8(1, 2)` → `False` — lesser does not satisfy ge. Exact.
#[test]
fn ge_u8_lesser_is_false() {
    assert_bool("ge_u8(1,2)", "ge_u8(0b0000_0001, 0b0000_0010)", "False");
}

/// Cross-op: `le_u8(a,b)` and `ge_u8(b,a)` always agree (antisymmetry). Sample: a=1, b=3.
/// Hand-computed: le_u8(1,3) = True; ge_u8(3,1) = True. Empirical cross-op check.
#[test]
fn le_ge_antisymmetry() {
    assert_bool("le_u8(1,3)", "le_u8(0b0000_0001, 0b0000_0011)", "True");
    assert_bool("ge_u8(3,1)", "ge_u8(0b0000_0011, 0b0000_0001)", "True");
}

// ── max_u8 / min_u8 ──────────────────────────────────────────────────────────────────────────────
//
// max_u8/min_u8 return a `Binary{8}` value. The reference must share the same provenance (Root,
// since both args are Root literals and the result is matched from one of them — not a Derived
// computation like add_bin). We use the literal directly in the reference program.

/// A `main` returning `Binary{8}`; `expected` is the reference Binary{8} literal.
fn assert_u8(label: &str, call: &str, expected_lit: &str) {
    let src = program(&format!("fn main() -> Binary{{8}} = {call}"));
    let expected = format!("nodule ref\nfn main() -> Binary{{8}} = {expected_lit}");
    assert_three_way(label, &src, &expected);
}

/// `max_u8(1, 3)` → `3` (0b0000_0011). Exact.
/// Expected (hand-computed, three-way verified).
#[test]
fn max_u8_returns_larger() {
    assert_u8(
        "max_u8(1,3)",
        "max_u8(0b0000_0001, 0b0000_0011)",
        "0b0000_0011",
    );
}

/// `max_u8(3, 1)` → `3` — order-independent. Exact.
#[test]
fn max_u8_order_independent() {
    assert_u8(
        "max_u8(3,1)",
        "max_u8(0b0000_0011, 0b0000_0001)",
        "0b0000_0011",
    );
}

/// `max_u8(2, 2)` → `2` — equal inputs; returns the second (b) by definition. Exact.
/// (max_u8 is defined as: Eq => b, consistent with the nodule source.)
#[test]
fn max_u8_equal_inputs() {
    assert_u8(
        "max_u8(2,2)",
        "max_u8(0b0000_0010, 0b0000_0010)",
        "0b0000_0010",
    );
}

/// `min_u8(1, 3)` → `1` (0b0000_0001). Exact.
/// Expected (hand-computed, three-way verified).
#[test]
fn min_u8_returns_smaller() {
    assert_u8(
        "min_u8(1,3)",
        "min_u8(0b0000_0001, 0b0000_0011)",
        "0b0000_0001",
    );
}

/// `min_u8(3, 1)` → `1` — order-independent. Exact.
#[test]
fn min_u8_order_independent() {
    assert_u8(
        "min_u8(3,1)",
        "min_u8(0b0000_0011, 0b0000_0001)",
        "0b0000_0001",
    );
}

/// `min_u8(2, 2)` → `2` — equal inputs; returns the first (a) by definition. Exact.
/// (min_u8 is defined as: Eq => a, consistent with the nodule source.)
#[test]
fn min_u8_equal_inputs() {
    assert_u8(
        "min_u8(2,2)",
        "min_u8(0b0000_0010, 0b0000_0010)",
        "0b0000_0010",
    );
}

/// Cross-op consistency: `max_u8(a,b)` and `min_u8(a,b)` together cover the domain —
/// for unequal a,b: max_u8(1,3) = 3, min_u8(1,3) = 1. Neither equals the other. Empirical.
#[test]
fn max_min_complementary() {
    assert_u8(
        "max_u8(1,3)",
        "max_u8(0b0000_0001, 0b0000_0011)",
        "0b0000_0011",
    );
    assert_u8(
        "min_u8(1,3)",
        "min_u8(0b0000_0001, 0b0000_0011)",
        "0b0000_0001",
    );
}

/// Edge: `max_u8(0, 255)` → `255`; `min_u8(0, 255)` → `0`. Covers the full Binary{8} range. Exact.
#[test]
fn max_min_full_range_edge() {
    assert_u8(
        "max_u8(0,255)",
        "max_u8(0b0000_0000, 0b1111_1111)",
        "0b1111_1111",
    );
    assert_u8(
        "min_u8(0,255)",
        "min_u8(0b0000_0000, 0b1111_1111)",
        "0b0000_0000",
    );
}

/// Consistency: `is_lt(cmp_u8(a,b))` agrees with `le_u8(a,b) && !le_u8(b,a)` — structural
/// cross-check of cmp_u8 and le_u8 on a pair where a < b. Hand-computed: cmp_u8(1,2) = Lt,
/// is_lt(Lt) = True; le_u8(1,2) = True, le_u8(2,1) = False (its negation is True). Empirical.
#[test]
fn cmp_u8_is_lt_agrees_with_le_u8_strict() {
    assert_bool(
        "is_lt(cmp_u8(1,2))",
        "is_lt(cmp_u8(0b0000_0001, 0b0000_0010))",
        "True",
    );
    assert_bool(
        "le_u8(1,2)-strict",
        "le_u8(0b0000_0001, 0b0000_0010)",
        "True",
    );
    assert_bool(
        "le_u8(2,1)-inverted",
        "le_u8(0b0000_0010, 0b0000_0001)",
        "False",
    );
}
