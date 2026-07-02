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
//! - **M-749** (`Repr::Seq`) — the **`.myc` surface is now wired** (lexer/parser/checker/elaborator:
//!   the `Seq{T, N}` type, the `[e1, …]` list literal, and the `seq_get`/`seq_len` prims). The full
//!   **three-way** (`L1-eval ≡ elaborate→L0-interp ≡ AOT`) differential over the surface runs in the
//!   `seq_*_surface_*` tests below, alongside the original **prim-level** differential
//!   (`seq.get`/`seq.len` over directly-built L0 `Node`s: **L0-interp ≡ AOT**) and the never-silent
//!   out-of-bounds refusal on both paths. Both layers are real (no faked/upgraded basis — G2/VR-5).
//! - **M-750** (`Repr::Bytes`) — the **`.myc` surface is now wired** (the `Bytes` type, the `0x…` hex
//!   literal, and the `bytes_get`/`bytes_len` prims). The full **three-way** differential over the
//!   surface runs in the `bytes_*_surface_*` tests below, alongside the original **prim-level**
//!   differential (`bytes.get`/`bytes.len`/`bytes.slice`/`bytes.concat` over directly-built L0
//!   `Node`s: **L0-interp ≡ AOT**) and the never-silent out-of-range/inverted-range refusals. UTF-8
//!   decode is written in `.myc` over these byte prims (per RFC-0032 D4) and is not exercised here.
//! - **Never-silent surface rejects** (G2): a **heterogeneous** list literal and an **odd-hex** `0x…`
//!   literal are explicit refusals at check/parse time — pinned by the `*_rejects` tests below.

use mycelium_core::{
    Bound, BoundBasis, BoundKind, GuaranteeStrength, Meta, Node, NormKind, Payload, Provenance,
    Repr, Value,
};
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
        "nodule d;\nfn main() => Binary{1} = eq(0b1010_0000, 0b1010_0000);",
        &r,
        &p,
    );
    let (r, p) = b1(false);
    assert_three_way(
        "eq Binary unequal",
        "nodule d;\nfn main() => Binary{1} = eq(0b1010_0000, 0b1010_0001);",
        &r,
        &p,
    );
}

#[test]
fn eq_ternary_width_typed() {
    let (r, p) = b1(true);
    assert_three_way(
        "eq Ternary equal",
        "nodule d;\nfn main() => Binary{1} = eq(0t00+-, 0t00+-);",
        &r,
        &p,
    );
    let (r, p) = b1(false);
    assert_three_way(
        "eq Ternary unequal",
        "nodule d;\nfn main() => Binary{1} = eq(0t00+-, 0t0+0-);",
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
        "nodule d;\nfn main() => Binary{1} = lt(0b1000_0000, 0b1010_0000);",
        &r,
        &p,
    );
    // Not strictly less when equal.
    let (r, p) = b1(false);
    assert_three_way(
        "lt Binary equal-is-false",
        "nodule d;\nfn main() => Binary{1} = lt(0b1010_0000, 0b1010_0000);",
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
        "nodule d;\nfn main() => Binary{1} = lt(0t00+-, 0t0+0-);",
        &r,
        &p,
    );
    // <0-00> = -9 < <00+-> = 2 → true; reversed → false (negative magnitude ordering).
    let (r, p) = b1(false);
    assert_three_way(
        "lt Ternary negative-false",
        "nodule d;\nfn main() => Binary{1} = lt(0t00+-, 0t0-00);",
        &r,
        &p,
    );
}

/// The `.myc` `Bool`-bridge smoke port: match the `Binary{1}` comparison bit into the `Bool` ADT,
/// exactly the one-line lift the E13-1 `std.cmp` port (M-718) bottoms out on.
#[test]
fn bool_bridge_from_comparison_bit() {
    // `match eq(a, b) { 0b1 => True, _ => False }` ≡ the data value `True`.
    let src = "nodule d;\nfn main() => Bool = match eq(0b1010_0000, 0b1010_0000) { 0b1 => True, _ => False };";
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
        "nodule d;\nfn main() => Binary{8} = and(0b1100_1010, 0b1010_1010);",
        &Repr::Binary { width: 8 },
        &Payload::Bits("10001010".chars().map(|c| c == '1').collect()),
    );
    assert_three_way(
        "or",
        "nodule d;\nfn main() => Binary{8} = or(0b1100_1010, 0b1010_1010);",
        &Repr::Binary { width: 8 },
        &Payload::Bits("11101010".chars().map(|c| c == '1').collect()),
    );
}

#[test]
fn add_bin_in_range() {
    // 0b0000_0001 + 0b0000_0010 = 0b0000_0011 (1 + 2 = 3).
    assert_three_way(
        "add_bin",
        "nodule d;\nfn main() => Binary{8} = add_bin(0b0000_0001, 0b0000_0010);",
        &Repr::Binary { width: 8 },
        &Payload::Bits("00000011".chars().map(|c| c == '1').collect()),
    );
}

#[test]
fn sub_bin_in_range() {
    // 0b0000_0101 - 0b0000_0010 = 0b0000_0011 (5 - 2 = 3).
    assert_three_way(
        "sub_bin",
        "nodule d;\nfn main() => Binary{8} = sub_bin(0b0000_0101, 0b0000_0010);",
        &Repr::Binary { width: 8 },
        &Payload::Bits("00000011".chars().map(|c| c == '1').collect()),
    );
}

// ── Never-silent (G2/VR-5): overflow + mismatch refuse on every path ─────────────────────────────

/// `add_bin` overflow (`255 + 1` at `Binary{8}`) is an explicit refusal on **all three** paths —
/// never a silent wrap to `0`. (The program type-checks: overflow is a runtime contract, D2.)
#[test]
fn add_bin_overflow_refuses_on_every_path() {
    let src = "nodule d;\nfn main() => Binary{8} = add_bin(0b1111_1111, 0b0000_0001);";
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
    let src = "nodule d;\nfn main() => Binary{8} = sub_bin(0b0000_0000, 0b0000_0001);";
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
    let src = "nodule d;\nfn main() => Binary{1} = eq(0b0000_0001, 0t00+-);";
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
    let src = "nodule d;\ndefault paradigm Binary;\nfn main() => Binary{1} = eq(5, 6);";
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
        "nodule d;\ndefault paradigm Binary;\nfn main() => Binary{1} = eq(0b0000_0101, 5);",
        &r,
        &p,
    );
    // Order-independent: bare decimal first, concrete second (`4` ≠ `5`).
    let (r, p) = b1(false);
    assert_three_way(
        "eq bare-first anchored false",
        "nodule d;\ndefault paradigm Binary;\nfn main() => Binary{1} = eq(4, 0b0000_0101);",
        &r,
        &p,
    );
}

// ── M-887 (`enb` Gap B): never-silent two's-complement multiply ─────────────────────────────────
//
// `mul_bin` (kernel `bin.mul`) is the first Gap-B prim of the RFC-0033 §4.1.2/§4.1.3 shared
// two's-complement arithmetic set (ADR-028). It reads its `Binary{N}` operands under the
// two's-complement (signed) interpretation — distinct from `add_bin`/`sub_bin`'s existing
// **unsigned** overflow contract (RFC-0032 D2) — and refuses out-of-`B_N` products explicitly,
// never a silent wrap (G2/VR-5).

#[test]
fn mul_bin_in_range_positive_and_negative() {
    // 3 * 4 = 12.
    assert_three_way(
        "mul_bin positive",
        "nodule d;\nfn main() => Binary{8} = mul_bin(0b0000_0011, 0b0000_0100);",
        &Repr::Binary { width: 8 },
        &Payload::Bits("00001100".chars().map(|c| c == '1').collect()),
    );
    // -3 * 4 = -12 (two's complement: -3 = 0b1111_1101, -12 = 0b1111_0100).
    assert_three_way(
        "mul_bin negative operand",
        "nodule d;\nfn main() => Binary{8} = mul_bin(0b1111_1101, 0b0000_0100);",
        &Repr::Binary { width: 8 },
        &Payload::Bits("11110100".chars().map(|c| c == '1').collect()),
    );
    // -3 * -4 = 12.
    assert_three_way(
        "mul_bin both negative",
        "nodule d;\nfn main() => Binary{8} = mul_bin(0b1111_1101, 0b1111_1100);",
        &Repr::Binary { width: 8 },
        &Payload::Bits("00001100".chars().map(|c| c == '1').collect()),
    );
}

/// `mul_bin` overflow (`127 * 2` at `Binary{8}`, out of `B_8 = [-128, 127]`) is an explicit refusal
/// on **all three** paths — never a silent wrap. (The program type-checks: the two's-complement
/// overflow bound is a runtime contract, like `add_bin`/`sub_bin`'s unsigned one.)
#[test]
fn mul_bin_overflow_refuses_on_every_path() {
    let src = "nodule d;\nfn main() => Binary{8} = mul_bin(0b0111_1111, 0b0000_0010);";
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

/// The classic two's-complement multiply-overflow edge (`i8::MIN * -1 = 128`, out of `B_8`) refuses
/// on all three paths — never a silent wrap back to `-128`.
#[test]
fn mul_bin_min_times_neg_one_refuses_on_every_path() {
    let src = "nodule d;\nfn main() => Binary{8} = mul_bin(0b1000_0000, 0b1111_1111);";
    let env = check_nodule(&parse(src).expect("parses")).expect("checks");

    let interp = Interpreter::new(
        PrimRegistry::with_builtins(),
        Box::new(mycelium_cert::BinaryTernarySwapEngine),
    );
    let prims = PrimRegistry::with_builtins();
    let engine = mycelium_cert::BinaryTernarySwapEngine;

    assert!(
        Evaluator::new(&env).call("main", vec![]).is_err(),
        "i8::MIN * -1 must refuse on L1-eval (never a silent wrap to -128)"
    );
    let node = elaborate(&env, "main").expect("in fragment");
    assert!(
        interp.eval(&node).is_err(),
        "i8::MIN * -1 must refuse on L0-interp"
    );
    assert!(
        mycelium_mlir::run(&node, &prims, &engine).is_err(),
        "i8::MIN * -1 must refuse on AOT"
    );
}

/// A width/paradigm mismatch (`Binary{8}` vs `Binary{1}`) is a **static** never-silent refusal —
/// caught at check time, mirroring `add_bin`/`sub_bin`'s width-preserving contract.
#[test]
fn mul_bin_width_mismatch_refuses_statically() {
    let src = "nodule d;\nfn main() => Binary{8} = mul_bin(0b0000_0001, 0b0);";
    assert!(
        check_nodule(&parse(src).expect("parses")).is_err(),
        "a width-mismatched mul_bin must be a static type error, never a silent coercion"
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

/// A `Binary{1}` value carrying a **`Declared`** guarantee (a user-asserted, unvalidated bound) — the
/// pre-image for the VR-5 no-upgrade test below.
fn b1_declared(truth: bool) -> Value {
    let bound = Bound {
        kind: BoundKind::Error {
            eps: 0.1,
            norm: NormKind::L2,
        },
        basis: BoundBasis::UserDeclared,
    };
    let meta = Meta::new(
        Provenance::Root,
        GuaranteeStrength::Declared,
        Some(bound),
        None,
        None,
        None,
    )
    .expect("well-formed Declared meta (M-I4)");
    Value::new(Repr::Binary { width: 1 }, Payload::Bits(vec![truth]), meta)
        .expect("well-formed declared bit")
}

/// VR-5 (regression for the pr-review Medium): `seq.get` must return the indexed element at **its
/// own** established basis, never upgraded. A `Declared` element retrieved from an otherwise-`Exact`
/// sequence+index must come back **`Declared`** (carrying its bound), not silently re-stamped `Exact`.
/// Before the fix, `seq.get` propagated the guarantee from the container/index only and dropped the
/// element's `Meta`, yielding an `Exact` result — a silent upgrade past basis.
#[test]
fn seq_get_preserves_a_declared_elements_guarantee() {
    // An Exact-container `Seq<Binary{1}, 2>` whose element 0 is *Declared*, element 1 is Exact.
    let seq = Value::new(
        Repr::Seq {
            elem: Box::new(Repr::Binary { width: 1 }),
            len: 2,
        },
        Payload::Seq(vec![b1_declared(true), b1_val(false)]),
        Meta::exact(Provenance::Root),
    )
    .expect("well-formed seq with a declared element");

    // get(seq, 0) → the Declared element: guarantee stays Declared, bound carried, value preserved.
    let node = Node::Op {
        prim: "seq.get".to_owned(),
        args: vec![Node::Const(seq.clone()), Node::Const(idx8(0))],
    };
    let (l0, aot) = run_l0_and_aot(&node);
    let l0 = l0.expect("seq.get(declared elem) L0-interp");
    let aot = aot.expect("seq.get(declared elem) AOT");
    assert_eq!(
        l0.meta().guarantee(),
        GuaranteeStrength::Declared,
        "VR-5: seq.get must NOT upgrade a Declared element to Exact"
    );
    assert!(
        l0.meta().bound().is_some(),
        "the Declared element's bound must carry through, never silently dropped (G2)"
    );
    assert_eq!(l0.payload(), &Payload::Bits(vec![true]), "value preserved");
    assert_eq!(
        l0.meta().guarantee(),
        aot.meta().guarantee(),
        "L0-interp and AOT must agree on the preserved guarantee"
    );

    // get(seq, 1) → the Exact element stays Exact (no spurious downgrade either).
    let node1 = Node::Op {
        prim: "seq.get".to_owned(),
        args: vec![Node::Const(seq), Node::Const(idx8(1))],
    };
    let (l0_1, _) = run_l0_and_aot(&node1);
    assert_eq!(
        l0_1.expect("seq.get(exact elem)").meta().guarantee(),
        GuaranteeStrength::Exact,
        "an Exact element from an Exact container stays Exact"
    );
}

// ── M-750: byte-string prims — prim-level differential (L0-interp ≡ AOT) ─────────────────────────
//
// As with the seq prims, `Repr::Bytes` has no `.myc` surface literal yet (FLAGGED), so we build the
// L0 `Node` tree directly. The reference interpreter and the AOT env-machine dispatch
// `bytes.{len,get,slice,concat}` through the same prim registry, so they must agree — and refuse an
// out-of-range / inverted access identically (never-silent on both paths, G2).

/// A `Repr::Bytes` const value over `bytes`.
fn bytes_val(bytes: Vec<u8>) -> Value {
    Value::new(
        Repr::Bytes,
        Payload::Bytes(bytes),
        Meta::exact(Provenance::Root),
    )
    .expect("well-formed bytes")
}

#[test]
fn bytes_get_and_len_l0_interp_equal_aot() {
    let bytes = bytes_val(vec![0x01, 0x02, 0x03]);

    // bytes.len → Binary{32}(3).
    let len_node = Node::Op {
        prim: "bytes.len".to_owned(),
        args: vec![Node::Const(bytes.clone())],
    };
    let (l0, aot) = run_l0_and_aot(&len_node);
    let l0 = l0.expect("bytes.len L0-interp");
    let aot = aot.expect("bytes.len AOT");
    assert_eq!((l0.repr(), l0.payload()), (aot.repr(), aot.payload()));
    assert_eq!(l0.repr(), &Repr::Binary { width: 32 });

    // bytes.get(b, 1) → Binary{8}(0x02).
    let get_node = Node::Op {
        prim: "bytes.get".to_owned(),
        args: vec![Node::Const(bytes), Node::Const(idx8(1))],
    };
    let (l0, aot) = run_l0_and_aot(&get_node);
    let l0 = l0.expect("bytes.get L0-interp");
    let aot = aot.expect("bytes.get AOT");
    assert_eq!((l0.repr(), l0.payload()), (aot.repr(), aot.payload()));
    assert_eq!(l0.repr(), &Repr::Binary { width: 8 });
    // 0x02 == 0b0000_0010.
    let want: Vec<bool> = (0..8).rev().map(|k| (0x02u8 >> k) & 1 == 1).collect();
    assert_eq!(l0.payload(), &Payload::Bits(want));
}

#[test]
fn bytes_slice_and_concat_l0_interp_equal_aot() {
    let bytes = bytes_val(vec![0x0a, 0x0b, 0x0c, 0x0d]);

    // bytes.slice(b, 1, 3) → Bytes(0x0b 0x0c).
    let slice_node = Node::Op {
        prim: "bytes.slice".to_owned(),
        args: vec![
            Node::Const(bytes.clone()),
            Node::Const(idx8(1)),
            Node::Const(idx8(3)),
        ],
    };
    let (l0, aot) = run_l0_and_aot(&slice_node);
    let l0 = l0.expect("bytes.slice L0-interp");
    let aot = aot.expect("bytes.slice AOT");
    assert_eq!((l0.repr(), l0.payload()), (aot.repr(), aot.payload()));
    assert_eq!(l0.payload(), &Payload::Bytes(vec![0x0b, 0x0c]));

    // bytes.concat(b, b) → 8 bytes.
    let concat_node = Node::Op {
        prim: "bytes.concat".to_owned(),
        args: vec![Node::Const(bytes.clone()), Node::Const(bytes)],
    };
    let (l0, aot) = run_l0_and_aot(&concat_node);
    let l0 = l0.expect("bytes.concat L0-interp");
    let aot = aot.expect("bytes.concat AOT");
    assert_eq!((l0.repr(), l0.payload()), (aot.repr(), aot.payload()));
    assert_eq!(
        l0.payload(),
        &Payload::Bytes(vec![0x0a, 0x0b, 0x0c, 0x0d, 0x0a, 0x0b, 0x0c, 0x0d])
    );
}

/// Never-silent (G2): an out-of-bounds `bytes.get` and an inverted/out-of-range `bytes.slice` are
/// explicit refusals on **both** paths — never a panic, never a silently-clamped result.
#[test]
fn bytes_out_of_range_refuses_on_both_paths() {
    let bytes = bytes_val(vec![0x01, 0x02, 0x03]); // len 3

    // index 3 is out of range.
    let get_oob = Node::Op {
        prim: "bytes.get".to_owned(),
        args: vec![Node::Const(bytes.clone()), Node::Const(idx8(3))],
    };
    let (l0, aot) = run_l0_and_aot(&get_oob);
    assert!(
        l0.is_err() && aot.is_err(),
        "OOB bytes.get must refuse on both paths"
    );

    // slice [2, 1) is inverted; [0, 4) overruns len — both refuse.
    let slice_inv = Node::Op {
        prim: "bytes.slice".to_owned(),
        args: vec![
            Node::Const(bytes.clone()),
            Node::Const(idx8(2)),
            Node::Const(idx8(1)),
        ],
    };
    let (l0, aot) = run_l0_and_aot(&slice_inv);
    assert!(
        l0.is_err() && aot.is_err(),
        "inverted bytes.slice must refuse on both paths"
    );

    let slice_over = Node::Op {
        prim: "bytes.slice".to_owned(),
        args: vec![
            Node::Const(bytes),
            Node::Const(idx8(0)),
            Node::Const(idx8(4)),
        ],
    };
    let (l0, aot) = run_l0_and_aot(&slice_over);
    assert!(
        l0.is_err() && aot.is_err(),
        "out-of-range bytes.slice must refuse on both paths"
    );
}

/// `bytes.*` over a non-bytes operand is an explicit type refusal on both paths.
#[test]
fn bytes_prims_refuse_non_bytes_operand() {
    let len_bad = Node::Op {
        prim: "bytes.len".to_owned(),
        args: vec![Node::Const(b1_val(true))],
    };
    let (l0, aot) = run_l0_and_aot(&len_bad);
    assert!(
        l0.is_err() && aot.is_err(),
        "bytes.len on a non-bytes must refuse on both paths"
    );
}

// ── M-749 surface: Seq{T,N} / `[..]` literal — full three-way differential (RFC-0032 D3) ─────────
//
// Now that the `.myc` surface exists (the `Seq{T, N}` type, the `[e1, …]` list literal, and the
// `seq_get`/`seq_len` prims), the seq enabler runs the **full three-way** (L1-eval ≡
// elaborate→L0-interp ≡ AOT) differential over a parsed `.myc` program — not only the prim-level
// L0≡AOT layer above. `assert_three_way` checks all three paths agree AND equal the reference.

/// The `Binary{32}` MSB-first encoding of `n` (the `seq_len`/`bytes_len` result shape).
fn b32(n: u32) -> (Repr, Payload) {
    let bits: Vec<bool> = (0..32).rev().map(|k| (n >> k) & 1 == 1).collect();
    (Repr::Binary { width: 32 }, Payload::Bits(bits))
}

/// `[0b1, 0b0, 0b1]` ascribed to `Seq{Binary{1}, 3}` round-trips as a `Repr::Seq` value on all three
/// paths — the first end-to-end proof the surface list literal builds a kernel sequence.
#[test]
fn seq_literal_surface_three_way() {
    let expected_repr = Repr::Seq {
        elem: Box::new(Repr::Binary { width: 1 }),
        len: 3,
    };
    let expected_payload = Payload::Seq(vec![b1_val(true), b1_val(false), b1_val(true)]);
    assert_three_way(
        "seq literal [0b1,0b0,0b1]",
        "nodule d;\nfn main() => Seq{Binary{1}, 3} = [0b1, 0b0, 0b1];",
        &expected_repr,
        &expected_payload,
    );
}

/// `seq_get([0b1,0b0,0b1], i)` over the surface agrees on all three paths for each in-range index.
/// The index is written as an explicit 8-bit `Binary{8}` literal (MSB-first).
#[test]
fn seq_get_surface_three_way() {
    for (i, want) in [(0u8, true), (1, false), (2, true)] {
        let (r, p) = b1(want);
        let src =
            format!("nodule d;\nfn main() => Binary{{1}} = seq_get([0b1, 0b0, 0b1], 0b{i:08b});");
        assert_three_way(&format!("seq_get index {i}"), &src, &r, &p);
    }
}

/// `seq_len([0b1,0b0,0b1])` over the surface is `Binary{32}(3)` on all three paths.
#[test]
fn seq_len_surface_three_way() {
    let (r, p) = b32(3);
    assert_three_way(
        "seq_len",
        "nodule d;\nfn main() => Binary{32} = seq_len([0b1, 0b0, 0b1]);",
        &r,
        &p,
    );
}

/// Never-silent (G2): a **heterogeneous** list literal is a static check refusal — the elements must
/// be homogeneous, never silently coerced (RFC-0032 D3). `0b1` is `Binary{1}`, `0b00` is `Binary{2}`.
#[test]
fn seq_heterogeneous_elements_reject() {
    let src = "nodule d;\nfn main() => Seq{Binary{1}, 2} = [0b1, 0b00];";
    let err = check_nodule(&parse(src).expect("parses"))
        .expect_err("a heterogeneous list literal must be a static check error, never a coercion");
    assert!(
        err.to_string().contains("homogeneous"),
        "the refusal must name the homogeneity cause (never-silent): {err}"
    );
}

/// Never-silent (G2): a list literal whose count disagrees with the ascribed `Seq{T, N}` length is a
/// static refusal — never a silent truncation/padding (RFC-0032 D3).
#[test]
fn seq_length_mismatch_reject() {
    let src = "nodule d;\nfn main() => Seq{Binary{1}, 5} = [0b1, 0b0, 0b1];";
    let err = check_nodule(&parse(src).expect("parses"))
        .expect_err("a list-length vs Seq{N} mismatch must be a static check error");
    assert!(
        err.to_string().contains("expected `Seq` length"),
        "the refusal must name the length-mismatch cause (never-silent): {err}"
    );
}

// ── M-750 surface: Bytes / `0x..` literal — full three-way differential (RFC-0032 D4) ────────────

/// `0x48_65_6c_6c_6f` ("Hello") round-trips as a `Repr::Bytes` value on all three paths.
#[test]
fn bytes_literal_surface_three_way() {
    let expected_payload = Payload::Bytes(vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]);
    assert_three_way(
        "bytes literal 0x48_65_6c_6c_6f",
        "nodule d;\nfn main() => Bytes = 0x48_65_6c_6c_6f;",
        &Repr::Bytes,
        &expected_payload,
    );
}

/// `bytes_get(0x_…, i)` over the surface is the indexed byte (`Binary{8}`) on all three paths.
#[test]
fn bytes_get_surface_three_way() {
    // 0x01_02_03, index 1 → 0x02 == 0b0000_0010.
    let want: Vec<bool> = (0..8).rev().map(|k| (0x02u8 >> k) & 1 == 1).collect();
    assert_three_way(
        "bytes_get index 1",
        "nodule d;\nfn main() => Binary{8} = bytes_get(0x01_02_03, 0b0000_0001);",
        &Repr::Binary { width: 8 },
        &Payload::Bits(want),
    );
}

/// `bytes_len(0x01_02_03)` over the surface is `Binary{32}(3)` on all three paths.
#[test]
fn bytes_len_surface_three_way() {
    let (r, p) = b32(3);
    assert_three_way(
        "bytes_len",
        "nodule d;\nfn main() => Binary{32} = bytes_len(0x01_02_03);",
        &r,
        &p,
    );
}

/// Never-silent (G2): an **odd-hex** `0x…` literal is a lex/parse refusal — a byte is two hex chars,
/// never a silent half-byte (RFC-0032 D4). `0x123` has three hex digits.
#[test]
fn bytes_odd_hex_reject() {
    let src = "nodule d\nfn main() => Bytes = 0x123";
    let err = parse(src).expect_err("an odd-hex `0x…` literal must be a parse error");
    assert!(
        err.to_string().contains("odd hex-digit count"),
        "the refusal must name the odd-hex cause, never a silent half-byte: {err}"
    );
}

/// Never-silent (G2): an empty `0x` (no hex digits) is a lex/parse refusal.
#[test]
fn bytes_empty_hex_reject() {
    let src = "nodule d\nfn main() => Bytes = 0x";
    let err = parse(src).expect_err("an empty `0x` literal must be a parse error");
    assert!(
        err.to_string().contains("no hex digits"),
        "the refusal must name the empty-hex cause: {err}"
    );
}
