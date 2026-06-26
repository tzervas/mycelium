//! Differential tests for `std.iter` (M-715, E13-1) — the self-hosted first-order iterator surface
//! over the `List<A>` cons-list shape.
//!
//! The nodule source is loaded verbatim via `include_str!` (the single source of truth), then a
//! typed driver `fn` is appended to pin every generic parameter to a concrete type (`Binary{8}`).
//! Without explicit pinning the monomorphizer emits a never-silent `Residual` (undetermined type
//! parameter — G2), so every driver uses typed helpers and explicit return types.
//!
//! # Honesty tags
//! - **`Exact`** — constructors (`Nil`/`Cons`) and the total discriminator `is_empty_l` — total
//!   over the finite domain (RFC-0016).
//! - **`Declared`** — the type-level contract of `length` (O(n) spine-walk) — a structural check,
//!   not a theorem.
//! - **`Empirical`** — the three-way differential agreement (L1-eval ≡ L0-interp ≡ AOT), validated
//!   by trial on the programs below; not a machine-checked proof.
//!
//! # FLAG: recursive HOF combinators cannot execute three-way (dropped, not shipped as type-check-only)
//! `map`, `filter`, `foldl`, `any`, `all`, `find` — all recursive HOF combinators — are defined in
//! `iter.myc` and type-check correctly, but CANNOT execute three-way via the monomorphize +
//! elaborate pipeline. The root cause: the stage-1 defunctionalization (RFC-0024 §4, M-687) handles
//! *saturated* HOF application (`f(x)` where `f` is in `fn_param_subst`), but does NOT handle a
//! *recursive call that re-passes a HOF parameter* (e.g. `map(rest, f)` inside `map`'s body passes
//! `f` — an Expr::Path naming a parameter — as a HOF argument; `mono::resolve_fn_args` looks up
//! `"f"` in `self.src.fns` and correctly refuses because `f` is a parameter, not a top-level fn).
//! This is never-silent (G2): the monomorphizer returns `ElabError::Residual` with an explicit
//! message. The combinators are retained in the nodule as design-phase surface (RFC-0031 §5 D4
//! Tier-0 design intent); three-way differential coverage awaits recursive HOF support (M-753 era
//! or a future stage-1 extension). Per the task requirement: "Anything that won't execute
//! three-way: drop it + FLAG with the reason." — these tests are dropped, not type-check-only stubs.
//!
//! # What three-way covers
//! - `is_empty_l` — total discriminator (Exact), three-way green
//! - `length` — O(n) spine-walk (Declared), three-way green
//! - `length` never-silent overflow bound (Empirical), three-way green

use mycelium_cert::{check_core, BinaryTernarySwapEngine, CheckVerdict};
use mycelium_core::GuaranteeStrength;
use mycelium_interp::{Interpreter, PrimRegistry};
use mycelium_l1::elab::build_registry;
use mycelium_l1::{check_nodule, elaborate, monomorphize, parse, Evaluator};

/// The std.iter nodule source, loaded at compile time — the single source of truth.
const ITER_SRC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../lib/std/iter.myc"
));

/// Build a full test program by appending a typed driver to the nodule source.
fn program(driver: &str) -> String {
    format!("{ITER_SRC}\n{driver}")
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

// ── is_empty_l ────────────────────────────────────────────────────────────────────────────────────

/// `is_empty_l(Nil)` → `True` (Exact: the empty case always returns True).
/// Expected (hand-computed, three-way verified): is_empty_l on empty List returns True.
#[test]
fn is_empty_l_on_nil_returns_true() {
    let driver = "\
fn mk_nil() -> List<Binary{8}> = Nil\n\
fn main() -> Bool = is_empty_l(mk_nil())";
    let src = program(driver);
    let expected = "nodule ref\nfn main() -> Bool = True";
    assert_three_way("is_empty_l(Nil)", &src, expected);
}

/// `is_empty_l(Cons(x, Nil))` → `False` (Exact: the Cons arm always returns False).
/// Expected (hand-computed, three-way verified): is_empty_l on non-empty List returns False.
#[test]
fn is_empty_l_on_cons_returns_false() {
    let driver = "\
fn mk_one() -> List<Binary{8}> = Cons(0b0000_0001, Nil)\n\
fn main() -> Bool = is_empty_l(mk_one())";
    let src = program(driver);
    let expected = "nodule ref\nfn main() -> Bool = False";
    assert_three_way("is_empty_l(Cons)", &src, expected);
}

// ── length ────────────────────────────────────────────────────────────────────────────────────────

/// `length([0b01, 0b02])` → `0b0000_0010`. O(n) spine-walk; Declared.
/// Expected (hand-computed, three-way verified): same provenance-matching rationale as
/// std_collections.rs::len_of_two_element_list — the reference uses add_bin, not a literal,
/// to match the Derived provenance produced by `length`'s `add_bin` spine.
#[test]
fn length_of_two_element_list() {
    let driver = "\
fn mk_two() -> List<Binary{8}> = Cons(0b0000_0001, Cons(0b0000_0010, Nil))\n\
fn main() -> Binary{8} = length(mk_two())";
    let src = program(driver);
    // length([e1, e2]) = add_bin(1, add_bin(1, 0)) = 2 — Derived provenance matches.
    let expected =
        "nodule ref\nfn main() -> Binary{8} = add_bin(0b0000_0001, add_bin(0b0000_0001, 0b0000_0000))";
    assert_three_way("length([1,2])", &src, expected);
}

/// `length(Nil)` → `0b0000_0000`. Base case (Exact: match-defined, returns the literal).
/// Expected (hand-computed, three-way verified).
#[test]
fn length_of_nil_is_zero() {
    let driver = "\
fn mk_nil() -> List<Binary{8}> = Nil\n\
fn main() -> Binary{8} = length(mk_nil())";
    let src = program(driver);
    let expected = "nodule ref\nfn main() -> Binary{8} = 0b0000_0000";
    assert_three_way("length(Nil)", &src, expected);
}

/// `length` of a three-element list → `0b0000_0011`. Declared.
/// Expected (hand-computed, three-way verified).
#[test]
fn length_of_three_element_list() {
    let driver = "\
fn mk_three() -> List<Binary{8}> = Cons(0b0000_0001, Cons(0b0000_0010, Cons(0b0000_0011, Nil)))\n\
fn main() -> Binary{8} = length(mk_three())";
    let src = program(driver);
    // length([e1,e2,e3]) = add_bin(1, add_bin(1, add_bin(1, 0))) = 3
    let expected = "nodule ref\nfn main() -> Binary{8} = add_bin(0b0000_0001, add_bin(0b0000_0001, add_bin(0b0000_0001, 0b0000_0000)))";
    assert_three_way("length([1,2,3])", &src, expected);
}

/// `length` never-silent overflow bound: `add_bin(0b0000_0001, 0b1111_1111)` refuses on ALL paths.
/// This pins the Binary{8} capacity ceiling of `length`, mirroring
/// std_collections.rs::len_bound_add_bin_overflow_refuses_on_every_path. Empirical.
#[test]
fn length_bound_add_bin_overflow_refuses_on_every_path() {
    let src = program("fn main() -> Binary{8} = add_bin(0b0000_0001, 0b1111_1111)");

    let env =
        check_nodule(&parse(&src).expect("length_bound: parse must succeed (overflow is runtime)"))
            .expect("length_bound: check must succeed (overflow is runtime contract)");

    let interp = Interpreter::new(
        PrimRegistry::with_builtins(),
        Box::new(BinaryTernarySwapEngine),
    );
    let prims = PrimRegistry::with_builtins();
    let engine = BinaryTernarySwapEngine;

    assert!(
        Evaluator::new(&env).call("main", vec![]).is_err(),
        "length_bound: L1-eval must refuse the add_bin overflow (never a silent wrap to 0)"
    );
    let node = elaborate(&env, "main").expect("length_bound: must elaborate");
    assert!(
        interp.eval(&node).is_err(),
        "length_bound: L0-interp must refuse the overflow"
    );
    assert!(
        mycelium_mlir::run(&node, &prims, &engine).is_err(),
        "length_bound: AOT must refuse the overflow"
    );
}

// ── Smoke-check: recursive HOF combinators type-check (but cannot monomorphize) ─────────────────
//
// FLAG: map/filter/foldl/any/all/find are self-hosted in iter.myc and PASS the type-checker
// (check_nodule succeeds). They cannot proceed to the three-way differential because the
// stage-1 monomorphizer (RFC-0024 §4) cannot defunctionalize recursive HOF calls where a
// function parameter is re-passed at a recursive call site (see file-level FLAG comment above).
// These smoke-checks pin that the nodule is well-typed and that the refusal is in monomorphize,
// not in check — this is the never-silent (G2) contract for the defunctionalization boundary.

/// `map` type-checks in the nodule (Exact: the type is structurally sound). Mono refuses it
/// at the recursive-HOF boundary (never-silent, G2 — `ElabError::Residual` with explicit message).
#[test]
fn map_typechecks_but_recursive_hof_cannot_monomorphize() {
    // A driver that calls map with a HOF — type-checks, but mono refuses.
    let src = program(
        "fn not_el(x: Binary{8}) -> Binary{8} = not(x)\n\
         fn mk_one() -> List<Binary{8}> = Cons(0b0000_0001, Nil)\n\
         fn main() -> List<Binary{8}> = map(mk_one(), not_el)",
    );
    // Step 1: the nodule + driver must type-check.
    let env = check_nodule(&parse(&src).expect("parse"))
        .expect("map: check_nodule must succeed — the nodule is well-typed");
    // Step 2: monomorphize explicitly refuses at the recursive-HOF boundary (never-silent, G2).
    let err = monomorphize(&env, "main")
        .expect_err("map: monomorphize must refuse (recursive HOF re-pass not yet supported)");
    // The refusal must name the HOF/defunctionalization cause — not a silent generic failure.
    let msg = format!("{err}");
    assert!(
        msg.contains("function-valued argument")
            || msg.contains("HOF")
            || msg.contains("defunctionalize")
            || msg.contains("top-level function"),
        "map: mono refusal must name the HOF boundary cause (never-silent), got: {msg}"
    );
}

/// `filter` type-checks but cannot monomorphize (same recursive-HOF boundary as `map`).
#[test]
fn filter_typechecks_but_recursive_hof_cannot_monomorphize() {
    let src = program(
        "fn is_nonzero(x: Binary{8}) -> Bool = match eq(x, 0b0000_0000) { 0b1 => False, _ => True }\n\
         fn mk_one() -> List<Binary{8}> = Cons(0b0000_0001, Nil)\n\
         fn main() -> List<Binary{8}> = filter(mk_one(), is_nonzero)",
    );
    let env =
        check_nodule(&parse(&src).expect("parse")).expect("filter: check_nodule must succeed");
    assert!(
        monomorphize(&env, "main").is_err(),
        "filter: monomorphize must refuse (recursive HOF re-pass not yet supported)"
    );
}

/// `any` type-checks but cannot monomorphize (same recursive-HOF boundary).
#[test]
fn any_typechecks_but_recursive_hof_cannot_monomorphize() {
    let src = program(
        "fn is_nonzero(x: Binary{8}) -> Bool = match eq(x, 0b0000_0000) { 0b1 => False, _ => True }\n\
         fn mk_one() -> List<Binary{8}> = Cons(0b0000_0001, Nil)\n\
         fn main() -> Bool = any(mk_one(), is_nonzero)",
    );
    let env = check_nodule(&parse(&src).expect("parse")).expect("any: check_nodule must succeed");
    assert!(
        monomorphize(&env, "main").is_err(),
        "any: monomorphize must refuse (recursive HOF re-pass not yet supported)"
    );
}

/// `find` type-checks but cannot monomorphize (same recursive-HOF boundary).
/// Never-silent (G2): find's None case is self-hosted correctly; the refusal is in mono, not the
/// nodule design.
#[test]
fn find_typechecks_but_recursive_hof_cannot_monomorphize() {
    let src = program(
        "fn is_nonzero(x: Binary{8}) -> Bool = match eq(x, 0b0000_0000) { 0b1 => False, _ => True }\n\
         fn mk_one() -> List<Binary{8}> = Cons(0b0000_0001, Nil)\n\
         fn main() -> Option<Binary{8}> = find(mk_one(), is_nonzero)",
    );
    let env = check_nodule(&parse(&src).expect("parse")).expect("find: check_nodule must succeed");
    assert!(
        monomorphize(&env, "main").is_err(),
        "find: monomorphize must refuse (recursive HOF re-pass not yet supported)"
    );
}
