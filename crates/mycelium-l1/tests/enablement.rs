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
//! - **M-749** (`Repr::Seq`) lands its enabler coverage here as a **prim-level differential**
//!   (`seq.get`/`seq.len` over directly-built L0 `Node`s: **L0-interp ≡ AOT**), plus the never-silent
//!   out-of-bounds refusal on both paths. The full **three-way** (`.myc` surface) differential is
//!   **deferred**: there is no `Seq` surface literal in the lexer/parser yet, so L1-eval over a
//!   parsed `.myc` Seq program cannot be exercised — that wiring is FLAGGED for a follow-up (it edits
//!   the `mycelium-l1` lexer/parser/checker, the `s10` collision surface). The prim-level path *is*
//!   the trusted-base coverage (the kernel prim + the AOT env-machine over the same registry); it is
//!   not faked or upgraded past its basis (G2/VR-5).
//! - **M-750** (`Repr::Bytes`) is **not** exercised here — it lands separately (M-750); its
//!   conformance ports join this file when it does (never faked here — G2/VR-5).

use mycelium_core::{Meta, Node, Payload, Provenance, Repr, Value};
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

/// `sub_bin` underflow (`0 - 1` at `Binary{8}`, a negative with no unsigned form) refuses on **all
/// three** paths — never a silent wrap to `255` — exactly like the overflow test above.
#[test]
fn sub_bin_underflow_refuses_on_every_path() {
    let src = "nodule d\nfn main() -> Binary{8} = sub_bin(0b0000_0000, 0b0000_0001)";
    let env = check_nodule(&parse(src).expect("parses")).expect("checks");

    let interp = Interpreter::new(
        PrimRegistry::with_builtins(),
        Box::new(mycelium_cert::BinaryTernarySwapEngine),
    );
    let prims = PrimRegistry::with_builtins();
    let engine = mycelium_cert::BinaryTernarySwapEngine;

    assert!(
        Evaluator::new(&env).call("main", vec![]).is_err(),
        "L1-eval must refuse the underflow (never a silent wrap to 255)"
    );
    let node = elaborate(&env, "main").expect("in fragment");
    assert!(
        interp.eval(&node).is_err(),
        "L0-interp must refuse the underflow"
    );
    assert!(
        mycelium_mlir::run(&node, &prims, &engine).is_err(),
        "AOT must refuse the underflow"
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

/// When **both** comparands are bare ambient decimals neither pins a *width* (and the `Binary{1}`
/// result can't anchor them, comparison being width-collapsing) — refused, never a defaulted width
/// (RFC-0032 D1 / RFC-0012 §4.3). `default paradigm Binary` makes `5`/`6` ambient (paradigm known,
/// width unknown), so this exercises the width-anchor refusal specifically.
#[test]
fn eq_both_bare_decimals_refuse() {
    let src = "nodule d\ndefault paradigm Binary\nfn main() -> Binary{1} = eq(5, 6)";
    assert!(
        check_nodule(&parse(src).expect("parses")).is_err(),
        "a both-bare-decimal `eq` must refuse (no width anchor), never a default width"
    );
}

/// A **concrete** operand anchors a bare ambient comparand's width (consistent with the
/// width-preserving prims, e.g. `xor(0b1111_0000, 15)`): under `default paradigm Binary`,
/// `eq(0b0000_0101, 5)` type-checks with `5` resolving to `Binary{8}`, evaluating to `0b1`.
#[test]
fn eq_concrete_operand_anchors_bare_decimal() {
    let (r, p) = b1(true);
    assert_three_way(
        "eq concrete anchors bare decimal",
        "nodule d\ndefault paradigm Binary\nfn main() -> Binary{1} = eq(0b0000_0101, 5)",
        &r,
        &p,
    );
    // Order-independent: bare decimal first, concrete second (`4` ≠ `5`).
    let (r, p) = b1(false);
    assert_three_way(
        "eq bare-first anchored false",
        "nodule d\ndefault paradigm Binary\nfn main() -> Binary{1} = eq(4, 0b0000_0101)",
        &r,
        &p,
    );
}

// ── M-749: indexed-sequence prims — prim-level differential (L0-interp ≡ AOT) ────────────────────
//
// `Repr::Seq` has no `.myc` surface literal yet (lexer/parser wiring deferred — FLAGGED in the
// module header), so the three-way `.myc` path can't run. Instead we build the L0 `Node` tree
// directly and exercise the achievable, trusted-base differential: the reference interpreter
// (`L0-interp`) and the AOT env-machine (`mycelium_mlir::run_core`) dispatch `seq.get`/`seq.len`
// through the *same* prim registry, so they must agree on the observable — and refuse an
// out-of-bounds index identically (never-silent on both paths, G2).

/// A `Binary{1}` value (a sequence element / an index bit-source).
fn b1_val(truth: bool) -> Value {
    Value::new(
        Repr::Binary { width: 1 },
        Payload::Bits(vec![truth]),
        Meta::exact(Provenance::Root),
    )
    .expect("well-formed bit")
}

/// An unsigned `Binary{8}` index literal value (MSB-first).
fn idx8(n: u8) -> Value {
    let bits: Vec<bool> = (0..8).rev().map(|k| (n >> k) & 1 == 1).collect();
    Value::new(
        Repr::Binary { width: 8 },
        Payload::Bits(bits),
        Meta::exact(Provenance::Root),
    )
    .expect("well-formed index")
}

/// A `Seq<Binary{1}, 3>` const value `[true, false, true]`.
fn seq3() -> Value {
    Value::new(
        Repr::Seq {
            elem: Box::new(Repr::Binary { width: 1 }),
            len: 3,
        },
        Payload::Seq(vec![b1_val(true), b1_val(false), b1_val(true)]),
        Meta::exact(Provenance::Root),
    )
    .expect("well-formed seq")
}

/// Run a single-`Op` L0 program on **both** the reference interpreter and the AOT env-machine and
/// return `(l0_interp, aot)` results (each a `Result`), so a test can assert agreement on success
/// *and* on refusal.
fn run_l0_and_aot(node: &Node) -> (Result<Value, String>, Result<Value, String>) {
    let interp = Interpreter::new(
        PrimRegistry::with_builtins(),
        Box::new(mycelium_cert::BinaryTernarySwapEngine),
    );
    let prims = PrimRegistry::with_builtins();
    let engine = mycelium_cert::BinaryTernarySwapEngine;
    let l0 = interp.eval(node).map_err(|e| format!("{e:?}"));
    // `run` returns the repr `Value` (the seq prims always yield a repr value, never a data value).
    let aot = mycelium_mlir::run(node, &prims, &engine).map_err(|e| format!("{e:?}"));
    (l0, aot)
}

#[test]
fn seq_get_in_range_l0_interp_equals_aot() {
    // seq.get([t,f,t], 0) == t ; index 2 == t ; index 1 == f.
    for (i, want) in [(0u8, true), (1, false), (2, true)] {
        let node = Node::Op {
            prim: "seq.get".to_owned(),
            args: vec![Node::Const(seq3()), Node::Const(idx8(i))],
        };
        let (l0, aot) = run_l0_and_aot(&node);
        let l0 = l0.unwrap_or_else(|e| panic!("seq.get({i}) L0-interp failed: {e}"));
        let aot = aot.unwrap_or_else(|e| panic!("seq.get({i}) AOT failed: {e}"));
        assert_eq!(
            (l0.repr(), l0.payload()),
            (aot.repr(), aot.payload()),
            "seq.get({i}): L0-interp vs AOT diverged"
        );
        assert_eq!(l0.repr(), &Repr::Binary { width: 1 });
        assert_eq!(l0.payload(), &Payload::Bits(vec![want]));
    }
}

#[test]
fn seq_len_l0_interp_equals_aot() {
    let node = Node::Op {
        prim: "seq.len".to_owned(),
        args: vec![Node::Const(seq3())],
    };
    let (l0, aot) = run_l0_and_aot(&node);
    let l0 = l0.expect("seq.len L0-interp");
    let aot = aot.expect("seq.len AOT");
    assert_eq!(
        (l0.repr(), l0.payload()),
        (aot.repr(), aot.payload()),
        "seq.len: L0-interp vs AOT diverged"
    );
    // 3 as Binary{32}, MSB-first.
    assert_eq!(l0.repr(), &Repr::Binary { width: 32 });
    let want: Vec<bool> = (0..32).rev().map(|k| (3u32 >> k) & 1 == 1).collect();
    assert_eq!(l0.payload(), &Payload::Bits(want));
}

/// Never-silent (G2): an out-of-bounds `seq.get` is an explicit refusal on **both** paths — never a
/// panic, never a silent default. (`len == 3`, so index 3 is out of range.)
#[test]
fn seq_get_out_of_bounds_refuses_on_both_paths() {
    let node = Node::Op {
        prim: "seq.get".to_owned(),
        args: vec![Node::Const(seq3()), Node::Const(idx8(3))],
    };
    let (l0, aot) = run_l0_and_aot(&node);
    assert!(
        l0.is_err(),
        "L0-interp must refuse an out-of-bounds seq.get (never a silent default)"
    );
    assert!(
        aot.is_err(),
        "AOT must refuse an out-of-bounds seq.get (never a silent default)"
    );
}

/// `seq.get`/`seq.len` over a **non-sequence** operand is an explicit type refusal on both paths
/// (never a silent coercion).
#[test]
fn seq_prims_refuse_non_sequence_operand() {
    let get_bad = Node::Op {
        prim: "seq.get".to_owned(),
        args: vec![Node::Const(b1_val(true)), Node::Const(idx8(0))],
    };
    let (l0, aot) = run_l0_and_aot(&get_bad);
    assert!(
        l0.is_err() && aot.is_err(),
        "seq.get on a non-seq must refuse on both paths"
    );

    let len_bad = Node::Op {
        prim: "seq.len".to_owned(),
        args: vec![Node::Const(b1_val(true))],
    };
    let (l0, aot) = run_l0_and_aot(&len_bad);
    assert!(
        l0.is_err() && aot.is_err(),
        "seq.len on a non-seq must refuse on both paths"
    );
}
