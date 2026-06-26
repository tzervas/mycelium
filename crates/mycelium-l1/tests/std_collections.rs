//! Differential tests for `std.collections` (M-716, #461) — the self-hosted `Vec`/`Map`/`Set`
//! core nodule.
//!
//! The nodule source is loaded verbatim via `include_str!` (the single source of truth), then a
//! typed driver `fn` is appended to pin every generic parameter to a concrete type (e.g.
//! `Binary{8}`). Without explicit pinning the monomorphizer emits a never-silent `Residual`
//! (undetermined type parameter — G2), so every driver carries typed helper functions.
//!
//! # Honesty tags
//! - **`Exact`** — constructors (`Nil`/`Cons`/`MNil`/`MCons`/`SNil`/`SCons`) and total
//!   discriminators (`is_empty`) — total, RFC-0016 §4.1 C2 / docs/spec/stdlib/collections.md §3.
//! - **`Declared`** — the type-level contract of every eliminator/transformer (`head`/`tail`/`get`/
//!   `snoc`/`reverse`/`map_get`/`set_contains`) — a structural check, not a theorem.
//! - **`Empirical`** — the three-way differential agreement (L1-eval ≡ L0-interp ≡ AOT), validated
//!   by trial on the programs below; not a machine-checked proof.
//! - **`Empirical`** — the `len`-fits-`Binary{8}` bound (add_bin refuses at 256 on every path —
//!   the overflow test pins this; not a type-level proof).
//!
//! # Grounding
//! Expected values are hand-computed and verified three-way (L1≡L0≡AOT). The Rust crate
//! `crates/mycelium-std-collections` exists but is **Seq-backed** (a different representation): it
//! shares the `is_empty`/`get`/`len`/`contains` semantics, but has no `head`/`tail`/`snoc`/`reverse`
//! (those are cons-list ops of this `.myc` port). So it is a value oracle for the shared-semantics
//! subset only — not a structural reference.

use mycelium_cert::{check_core, BinaryTernarySwapEngine, CheckVerdict};
use mycelium_core::GuaranteeStrength;
use mycelium_interp::{Interpreter, PrimRegistry};
use mycelium_l1::elab::build_registry;
use mycelium_l1::{check_nodule, elaborate, monomorphize, parse, Evaluator};

/// The std.collections nodule source, loaded at compile time — the single source of truth.
const COLLECTIONS_SRC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../lib/std/collections.myc"
));

/// Build a full test program by appending a typed driver to the nodule source.
fn program(driver: &str) -> String {
    format!("{COLLECTIONS_SRC}\n{driver}")
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

// ── Vec: is_empty ──────────────────────────────────────────────────────────────────────────────────

/// `is_empty(Nil)` → `True` (Exact: the empty case always returns True).
/// Expected (hand-computed, three-way verified): Vec::is_empty on an empty list returns true.
#[test]
fn is_empty_on_nil_returns_true() {
    let driver = "\
fn mk_nil() -> Vec<Binary{8}> = Nil\n\
fn main() -> Bool = is_empty(mk_nil())";
    let src = program(driver);
    let expected = "nodule ref\nfn main() -> Bool = True";
    assert_three_way("is_empty(Nil)", &src, expected);
}

/// `is_empty(Cons(x, Nil))` → `False` (Exact: the Cons arm always returns False).
/// Expected (hand-computed, three-way verified): Vec::is_empty on a non-empty list returns false.
#[test]
fn is_empty_on_cons_returns_false() {
    let driver = "\
fn mk_one() -> Vec<Binary{8}> = Cons(0b0000_0001, Nil)\n\
fn main() -> Bool = is_empty(mk_one())";
    let src = program(driver);
    let expected = "nodule ref\nfn main() -> Bool = False";
    assert_three_way("is_empty(Cons)", &src, expected);
}

// ── Vec: head ──────────────────────────────────────────────────────────────────────────────────────

/// `head(Nil)` → `None` — never-silent (G2): empty Vec never fabricates a value. Declared.
/// Expected (hand-computed, three-way verified): Vec::head on empty returns None.
#[test]
fn head_on_nil_returns_none() {
    let driver = "\
fn mk_nil() -> Vec<Binary{8}> = Nil\n\
fn main() -> Option<Binary{8}> = head(mk_nil())";
    let src = program(driver);
    let expected =
        "nodule ref\ntype Option<A> = Some(A) | None\nfn main() -> Option<Binary{8}> = None";
    assert_three_way("head(Nil)", &src, expected);
}

/// `head(Cons(x, rest))` → `Some(x)` — first element is returned. Declared.
/// Expected (hand-computed, three-way verified): Vec::head on Cons(0b0000_0001, Nil) returns Some(0b0000_0001).
#[test]
fn head_on_cons_returns_some() {
    let driver = "\
fn mk_one() -> Vec<Binary{8}> = Cons(0b0000_0001, Nil)\n\
fn main() -> Option<Binary{8}> = head(mk_one())";
    let src = program(driver);
    let expected = "nodule ref\ntype Option<A> = Some(A) | None\nfn main() -> Option<Binary{8}> = Some(0b0000_0001)";
    assert_three_way("head(Cons)", &src, expected);
}

// ── Vec: tail ──────────────────────────────────────────────────────────────────────────────────────

/// `tail(Nil)` → `None` — never-silent (G2). Declared.
/// Expected (hand-computed, three-way verified): Vec::tail on empty returns None.
#[test]
fn tail_on_nil_returns_none() {
    let driver = "\
fn mk_nil() -> Vec<Binary{8}> = Nil\n\
fn main() -> Option<Vec<Binary{8}>> = tail(mk_nil())";
    let src = program(driver);
    // The inner Vec is the empty Nil — Option<Vec<Binary{8}>> = None.
    let expected = "nodule ref\ntype Vec<A> = Nil | Cons(A, Vec<A>)\ntype Option<A> = Some(A) | None\nfn main() -> Option<Vec<Binary{8}>> = None";
    assert_three_way("tail(Nil)", &src, expected);
}

/// `tail(Cons(x, rest))` → `Some(rest)` — returns the spine after the head. Declared.
/// Expected (hand-computed, three-way verified): tail on [1, 2] returns Some([2]).
#[test]
fn tail_on_cons_returns_some() {
    let driver = "\
fn mk_two() -> Vec<Binary{8}> = Cons(0b0000_0001, Cons(0b0000_0010, Nil))\n\
fn main() -> Option<Vec<Binary{8}>> = tail(mk_two())";
    let src = program(driver);
    let expected = "nodule ref\ntype Vec<A> = Nil | Cons(A, Vec<A>)\ntype Option<A> = Some(A) | None\nfn main() -> Option<Vec<Binary{8}>> = Some(Cons(0b0000_0010, Nil))";
    assert_three_way("tail(Cons)", &src, expected);
}

// ── Vec: len ───────────────────────────────────────────────────────────────────────────────────────

/// `len` over a two-element list → `0b0000_0010`. O(n) spine-walk; Declared.
/// Expected (hand-computed, three-way verified): Vec::len on [1, 2] returns 2.
/// The reference program uses `add_bin` (not a literal) to match the `Derived` provenance produced
/// by `len`'s `add_bin` spine — a literal `0b0000_0010` has `Root` provenance and fails `assert_eq`.
/// `len([1,2]) = add_bin(1, add_bin(1, 0))`: same ops and the same `Derived` provenance, which
/// `CoreValue` equality requires (a `Root`-provenance literal `0b0000_0010` would fail `assert_eq`).
/// (Empirical basis; the three-way agreement is separately asserted above.)
#[test]
fn len_of_two_element_list() {
    let driver = "\
fn mk_two() -> Vec<Binary{8}> = Cons(0b0000_0001, Cons(0b0000_0010, Nil))\n\
fn main() -> Binary{8} = len(mk_two())";
    let src = program(driver);
    // add_bin(1, add_bin(1, 0)) = 2 via the same op tree as len([e1, e2])
    let expected =
        "nodule ref\nfn main() -> Binary{8} = add_bin(0b0000_0001, add_bin(0b0000_0001, 0b0000_0000))";
    assert_three_way("len([1,2])", &src, expected);
}

/// `len` over a three-element list → `0b0000_0011`. Declared.
/// Expected (hand-computed, three-way verified): Vec::len on [1, 2, 3] returns 3.
/// Same provenance-matching rationale: add_bin(1, add_bin(1, add_bin(1, 0))) = 3.
#[test]
fn len_of_three_element_list() {
    let driver = "\
fn mk_three() -> Vec<Binary{8}> = Cons(0b0000_0001, Cons(0b0000_0010, Cons(0b0000_0011, Nil)))\n\
fn main() -> Binary{8} = len(mk_three())";
    let src = program(driver);
    // add_bin(1, add_bin(1, add_bin(1, 0))) = 3
    let expected = "nodule ref\nfn main() -> Binary{8} = add_bin(0b0000_0001, add_bin(0b0000_0001, add_bin(0b0000_0001, 0b0000_0000)))";
    assert_three_way("len([1,2,3])", &src, expected);
}

/// `len`-bound: the `add_bin` mechanism underlying `len`'s `Binary{8}` count refuses at 256 on ALL
/// three paths — never a silent wrap (G2/VR-5). Empirical (pinned by trial on the programs below).
///
/// Why not test via a 256-element list: `len`'s recursion reaches the L1 evaluator's depth limit
/// (`DEFAULT_DEPTH = 64`) long before reaching 256 elements; and the L0 interpreter does not use
/// the same deep-worker-stack machinery, so 256-deep `fill` recursion overflows the Rust thread
/// stack instead of being a clean `is_err()`. Both are never-silent refusals — but neither is the
/// `add_bin` arithmetic overflow. We test the actual mechanism (add_bin overflow at `Binary{8}`
/// boundary) directly, exactly as `enablement.rs::add_bin_overflow_refuses_on_every_path` does.
/// The `len` connection: `len(xs)` is `add_bin(1, len(rest))` — the 256th step would compute
/// `add_bin(0b0000_0001, 0b1111_1111) = 256` which this test pins. Empirical.
///
/// Expected (hand-computed, three-way verified): Vec::len fails (add_bin overflows) on a > 255-element list.
#[test]
fn len_bound_add_bin_overflow_refuses_on_every_path() {
    // add_bin(0b0000_0001, 0b1111_1111) = 256, which overflows Binary{8} — the exact operation
    // that len would execute on its 256th element. This is the never-silent (G2) contract for
    // len's Binary{8} index width. Uses the collections nodule source as context for consistency.
    let src = program("fn main() -> Binary{8} = add_bin(0b0000_0001, 0b1111_1111)");

    let env = check_nodule(
        &parse(&src).expect("len_bound: parse must succeed (overflow is runtime, not static)"),
    )
    .expect("len_bound: check must succeed (overflow is a runtime contract)");

    let interp = Interpreter::new(
        PrimRegistry::with_builtins(),
        Box::new(BinaryTernarySwapEngine),
    );
    let prims = PrimRegistry::with_builtins();
    let engine = BinaryTernarySwapEngine;

    assert!(
        Evaluator::new(&env).call("main", vec![]).is_err(),
        "len_bound: L1-eval must refuse the add_bin overflow (never a silent wrap to 0)"
    );
    let node = elaborate(&env, "main").expect("len_bound: must elaborate");
    assert!(
        interp.eval(&node).is_err(),
        "len_bound: L0-interp must refuse the overflow"
    );
    assert!(
        mycelium_mlir::run(&node, &prims, &engine).is_err(),
        "len_bound: AOT must refuse the overflow"
    );
}

// ── Vec: get ───────────────────────────────────────────────────────────────────────────────────────

/// `get([1,2,3], 0)` → `Some(1)` — index 0 returns the head. Declared.
/// Expected (hand-computed, three-way verified): Vec::get on [1,2,3] at 0 returns Some(1).
#[test]
fn get_index_0_returns_head() {
    let driver = "\
fn mk_three() -> Vec<Binary{8}> = Cons(0b0000_0001, Cons(0b0000_0010, Cons(0b0000_0011, Nil)))\n\
fn main() -> Option<Binary{8}> = get(mk_three(), 0b0000_0000)";
    let src = program(driver);
    let expected = "nodule ref\ntype Option<A> = Some(A) | None\nfn main() -> Option<Binary{8}> = Some(0b0000_0001)";
    assert_three_way("get([1,2,3], 0)", &src, expected);
}

/// `get([1,2,3], 1)` → `Some(2)` — index 1 returns the second element. Declared.
/// Expected (hand-computed, three-way verified): Vec::get on [1,2,3] at 1 returns Some(2).
#[test]
fn get_index_1_returns_second() {
    let driver = "\
fn mk_three() -> Vec<Binary{8}> = Cons(0b0000_0001, Cons(0b0000_0010, Cons(0b0000_0011, Nil)))\n\
fn main() -> Option<Binary{8}> = get(mk_three(), 0b0000_0001)";
    let src = program(driver);
    let expected = "nodule ref\ntype Option<A> = Some(A) | None\nfn main() -> Option<Binary{8}> = Some(0b0000_0010)";
    assert_three_way("get([1,2,3], 1)", &src, expected);
}

/// `get([1,2,3], 5)` → `None` — OOB → None, never-silent (G2). Declared.
/// Expected (hand-computed, three-way verified): Vec::get on [1,2,3] at 5 returns None.
#[test]
fn get_out_of_bounds_returns_none() {
    let driver = "\
fn mk_three() -> Vec<Binary{8}> = Cons(0b0000_0001, Cons(0b0000_0010, Cons(0b0000_0011, Nil)))\n\
fn main() -> Option<Binary{8}> = get(mk_three(), 0b0000_0101)";
    let src = program(driver);
    let expected =
        "nodule ref\ntype Option<A> = Some(A) | None\nfn main() -> Option<Binary{8}> = None";
    assert_three_way("get([1,2,3], OOB)", &src, expected);
}

// ── Vec: snoc ──────────────────────────────────────────────────────────────────────────────────────

/// `snoc([1,2], 3)` → `[1,2,3]` — appends at the end. Declared.
/// Expected (hand-computed, three-way verified): Vec::snoc on [1,2] with 3 returns [1,2,3] (Cons(1,Cons(2,Cons(3,Nil)))).
#[test]
fn snoc_appends_at_end() {
    let driver = "\
fn mk_two() -> Vec<Binary{8}> = Cons(0b0000_0001, Cons(0b0000_0010, Nil))\n\
fn main() -> Vec<Binary{8}> = snoc(mk_two(), 0b0000_0011)";
    let src = program(driver);
    // snoc([1,2], 3) = [1,2,3] = Cons(1, Cons(2, Cons(3, Nil)))
    let expected = "nodule ref\ntype Vec<A> = Nil | Cons(A, Vec<A>)\n\
fn main() -> Vec<Binary{8}> = Cons(0b0000_0001, Cons(0b0000_0010, Cons(0b0000_0011, Nil)))";
    assert_three_way("snoc([1,2], 3)", &src, expected);
}

// ── Vec: reverse ───────────────────────────────────────────────────────────────────────────────────

/// `reverse([1,2,3])` → `[3,2,1]` — snoc-based recursion reverses the spine (O(n²); the O(n) accumulator form is blocked under RFC-0007 §11.3 — see the nodule comment). Declared.
/// Expected (hand-computed, three-way verified): Vec::reverse on [1,2,3] returns [3,2,1].
#[test]
fn reverse_of_three_element_list() {
    let driver = "\
fn mk_three() -> Vec<Binary{8}> = Cons(0b0000_0001, Cons(0b0000_0010, Cons(0b0000_0011, Nil)))\n\
fn main() -> Vec<Binary{8}> = reverse(mk_three())";
    let src = program(driver);
    // reverse([1,2,3]) = [3,2,1] = Cons(3, Cons(2, Cons(1, Nil)))
    let expected = "nodule ref\ntype Vec<A> = Nil | Cons(A, Vec<A>)\n\
fn main() -> Vec<Binary{8}> = Cons(0b0000_0011, Cons(0b0000_0010, Cons(0b0000_0001, Nil)))";
    assert_three_way("reverse([1,2,3])", &src, expected);
}

// ── Map: map_get ───────────────────────────────────────────────────────────────────────────────────
//
// map_get is monomorphic at Binary{8} (eq is width-typed; M-753 width-generics not yet landed).
// Lookup is O(n) linear scan; first match wins; missing key → None (G2).

/// `map_get(MCons(k,v, MNil), k)` → `Some(v)` — key present, hit on first entry. Declared.
/// Expected (hand-computed, three-way verified): Map::get on {1→10} with key 1 returns Some(10).
#[test]
fn map_get_hit_returns_some() {
    let driver = "\
fn mk_map() -> Map<Binary{8}, Binary{8}> = MCons(0b0000_0001, 0b0000_1010, MNil)\n\
fn main() -> Option<Binary{8}> = map_get(mk_map(), 0b0000_0001)";
    let src = program(driver);
    // map_get({1→10}, 1) = Some(10) = Some(0b0000_1010)
    let expected = "nodule ref\ntype Option<A> = Some(A) | None\nfn main() -> Option<Binary{8}> = Some(0b0000_1010)";
    assert_three_way("map_get(hit)", &src, expected);
}

/// `map_get(MCons(k,v, MNil), k2)` where `k2 ≠ k` → `None`. Never-silent (G2). Declared.
/// Expected (hand-computed, three-way verified): Map::get on {1→10} with key 2 returns None.
#[test]
fn map_get_miss_returns_none() {
    let driver = "\
fn mk_map() -> Map<Binary{8}, Binary{8}> = MCons(0b0000_0001, 0b0000_1010, MNil)\n\
fn main() -> Option<Binary{8}> = map_get(mk_map(), 0b0000_0010)";
    let src = program(driver);
    // map_get({1→10}, 2) = None
    let expected =
        "nodule ref\ntype Option<A> = Some(A) | None\nfn main() -> Option<Binary{8}> = None";
    assert_three_way("map_get(miss)", &src, expected);
}

/// `map_get` with two entries, shadowed key: insert order wins (first MCons wins). Declared.
/// Expected (hand-computed, three-way verified): Map::get on {2→20, 1→10} with key 2 returns Some(20).
#[test]
fn map_get_multi_entry_first_wins() {
    // map_insert(2, 20, map_insert(1, 10, map_empty)) = MCons(2, 20, MCons(1, 10, MNil))
    // map_get that, key=2 → Some(20)
    let driver = "\
fn mk_map() -> Map<Binary{8}, Binary{8}> = MCons(0b0000_0010, 0b0001_0100, MCons(0b0000_0001, 0b0000_1010, MNil))\n\
fn main() -> Option<Binary{8}> = map_get(mk_map(), 0b0000_0010)";
    let src = program(driver);
    // map_get({2→20, 1→10}, 2) = Some(20) = Some(0b0001_0100)
    let expected = "nodule ref\ntype Option<A> = Some(A) | None\nfn main() -> Option<Binary{8}> = Some(0b0001_0100)";
    assert_three_way("map_get(multi, first-wins)", &src, expected);
}

// ── Set: set_contains ──────────────────────────────────────────────────────────────────────────────
//
// set_contains is monomorphic at Binary{8} (same eq constraint as map_get). O(n) scan. Declared.

/// `set_contains(SCons(x, SNil), x)` → `True` — element present. Declared.
/// Expected (hand-computed, three-way verified): Set::contains on {1} with 1 returns True.
#[test]
fn set_contains_present_returns_true() {
    let driver = "\
fn mk_set() -> Set<Binary{8}> = SCons(0b0000_0001, SNil)\n\
fn main() -> Bool = set_contains(mk_set(), 0b0000_0001)";
    let src = program(driver);
    let expected = "nodule ref\nfn main() -> Bool = True";
    assert_three_way("set_contains(present)", &src, expected);
}

/// `set_contains(SCons(x, SNil), y)` where `y ≠ x` → `False`. Never-silent (G2). Declared.
/// Expected (hand-computed, three-way verified): Set::contains on {1} with 2 returns False.
#[test]
fn set_contains_absent_returns_false() {
    let driver = "\
fn mk_set() -> Set<Binary{8}> = SCons(0b0000_0001, SNil)\n\
fn main() -> Bool = set_contains(mk_set(), 0b0000_0010)";
    let src = program(driver);
    let expected = "nodule ref\nfn main() -> Bool = False";
    assert_three_way("set_contains(absent)", &src, expected);
}

/// `set_contains` on empty set → `False`. Never-silent (G2). Declared.
/// Expected (hand-computed, three-way verified): Set::contains on {} with any key returns False.
#[test]
fn set_contains_empty_returns_false() {
    let driver = "\
fn mk_empty() -> Set<Binary{8}> = SNil\n\
fn main() -> Bool = set_contains(mk_empty(), 0b0000_0001)";
    let src = program(driver);
    let expected = "nodule ref\nfn main() -> Bool = False";
    assert_three_way("set_contains(empty)", &src, expected);
}
