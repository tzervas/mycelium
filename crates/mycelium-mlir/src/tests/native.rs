//! In-crate tests for `dialect/native.rs` (CLAUDE.md test-layout rule; extracted as-touched by
//! M-725 from the former inline `#[cfg(test)] mod tests`). White-box access via
//! `use crate::dialect::native::*`; the logic file carries no inline `#[cfg(test)]` code.
//!
//! Feature-gated: `dialect::native` only compiles under `mlir-dialect`, so this module is gated to
//! match (`super::super` declares it `#[cfg(feature = "mlir-dialect")]`). These tests are pure
//! **emission** checks — they exercise `emit_mlir` (deterministic text, no toolchain) and the
//! refusal boundary; the toolchain-dependent compile/run differential lives in
//! `tests/threeway_differential.rs`.

use crate::dialect::native::*;
use mycelium_core::{Meta, Node, Payload, Provenance, Repr, Trit, Value};

fn byte(bits: [bool; 8]) -> Value {
    Value::new(
        Repr::Binary { width: 8 },
        Payload::Bits(bits.to_vec()),
        Meta::exact(Provenance::Root),
    )
    .unwrap()
}

fn tern(trits: Vec<Trit>) -> Value {
    let m = trits.len() as u32;
    Value::new(
        Repr::Ternary { trits: m },
        Payload::Trits(trits),
        Meta::exact(Provenance::Root),
    )
    .unwrap()
}

const A: [bool; 8] = [true, false, true, true, false, false, true, false];

fn not_a_xor_b() -> Node {
    let b = byte([false, false, true, false, true, false, true, true]);
    Node::Op {
        prim: "bit.not".into(),
        args: vec![Node::Op {
            prim: "bit.xor".into(),
            args: vec![Node::Const(byte(A)), Node::Const(b)],
        }],
    }
}

/// A `trit.add` over two 4-trit constants whose sum stays in range (no overflow).
fn trit_add_in_range() -> Node {
    Node::Op {
        prim: "trit.add".into(),
        args: vec![
            Node::Const(tern(vec![Trit::Zero, Trit::Pos, Trit::Neg, Trit::Pos])),
            Node::Const(tern(vec![Trit::Zero, Trit::Neg, Trit::Pos, Trit::Neg])),
        ],
    }
}

/// A `trit.sub` over two 3-trit constants with a named numeric oracle: `3 - 1 = 2`, in range (no
/// overflow). Balanced-ternary MSB-first: `3 = [0,+,0]` (0·9 + 1·3 + 0·1), `1 = [0,0,+]`, and the
/// difference `2 = [0,+,-]` (0·9 + 1·3 + (-1)·1) all fit 3 trits. Mirrors `trit_add_in_range`'s
/// named-helper style so the emission test exercises a known in-range pair, not an anonymous one.
fn trit_sub_in_range() -> Node {
    Node::Op {
        prim: "trit.sub".into(),
        args: vec![
            Node::Const(tern(vec![Trit::Zero, Trit::Pos, Trit::Zero])),
            Node::Const(tern(vec![Trit::Zero, Trit::Zero, Trit::Pos])),
        ],
    }
}

#[test]
fn emits_a_real_arith_func_module() {
    let (m, kind, width) = emit_mlir(&not_a_xor_b()).expect("emit");
    assert!(m.starts_with("module {"));
    assert!(m.contains("func.func @main()"));
    assert!(m.contains("func.func private @putchar"));
    // Real arith ops (the lowering, not the textual skeleton):
    assert!(m.contains("arith.xori"), "expected arith.xori in:\n{m}");
    assert!(m.contains("func.call @putchar"));
    assert!(m.contains("func.return"));
    assert_eq!(kind, ResultKind::Binary);
    assert_eq!(width, 8);
}

#[test]
fn emission_is_deterministic() {
    assert_eq!(
        emit_mlir(&not_a_xor_b()).unwrap().0,
        emit_mlir(&not_a_xor_b()).unwrap().0
    );
    // The trit-carry module (M-725) is deterministic too (no nondeterministic SSA naming).
    assert_eq!(
        emit_mlir(&trit_add_in_range()).unwrap().0,
        emit_mlir(&trit_add_in_range()).unwrap().0
    );
}

/// M-725: `trit.add` now lowers through the real dialect path — a ripple-carry over `arith` ops
/// with the shared overflow-sentinel read-back branch (`cf.cond_br`). Asserts the genuine carry
/// arithmetic (`arith.remsi`/`arith.divsi`) and the never-silent overflow branch are emitted.
#[test]
fn trit_add_emits_real_ripple_carry_with_overflow_branch() {
    let (m, kind, width) = emit_mlir(&trit_add_in_range()).expect("emit trit.add");
    assert_eq!(kind, ResultKind::Ternary);
    assert_eq!(width, 4);
    // The balanced-ternary carry step (`x = s + 4`, then `srem 3 − 1` / `sdiv 3 − 1`):
    assert!(m.contains("arith.remsi"), "expected arith.remsi in:\n{m}");
    assert!(m.contains("arith.divsi"), "expected arith.divsi in:\n{m}");
    // The never-silent overflow read-back: a conditional branch on the folded overflow flag.
    assert!(m.contains("cf.cond_br"), "expected cf.cond_br in:\n{m}");
    assert!(m.contains("^ovf:"), "expected ^ovf block in:\n{m}");
    assert!(m.contains("^ok:"), "expected ^ok block in:\n{m}");
    // Both terminating blocks return.
    assert!(m.contains("func.return"));
}

/// `trit.sub` lowers via `add(a, neg(b))` — same ripple + overflow branch (M-725). Exercises the
/// named `3 - 1 = 2` in-range pair (numeric oracle, mirroring `trit_add_in_range`).
#[test]
fn trit_sub_lowers_through_the_dialect_path() {
    let (m, kind, width) = emit_mlir(&trit_sub_in_range()).expect("emit trit.sub");
    assert_eq!(kind, ResultKind::Ternary);
    assert_eq!(width, 3);
    assert!(m.contains("arith.remsi"), "expected ripple-carry in:\n{m}");
    assert!(
        m.contains("cf.cond_br"),
        "expected overflow branch in:\n{m}"
    );
}

/// An overflow-free, purely element-wise program emits NO control flow — the M-601 single-block
/// shape is preserved exactly (the branch is added only when a trit additive op needs it).
#[test]
fn element_wise_program_has_no_overflow_branch() {
    let (m, _, _) = emit_mlir(&not_a_xor_b()).expect("emit");
    assert!(
        !m.contains("cf.cond_br"),
        "element-wise program must stay single-block (no cf.cond_br):\n{m}"
    );
}

/// A `trit.mul` over two 3-trit constants with a named numeric oracle: `2 · 3 = 6`, in range (no
/// overflow). Balanced-ternary MSB-first: `2 = [0,+,-]` (1·3 + (−1)·1), `3 = [0,+,0]` (1·3), and the
/// product `6 = [+,-,0]` (1·9 + (−1)·3 + 0·1) all fit 3 trits. Mirrors the `trit_add_in_range` /
/// `trit_sub_in_range` named-helper style so the emission test exercises a known in-range pair.
fn trit_mul_in_range() -> Node {
    Node::Op {
        prim: "trit.mul".into(),
        args: vec![
            Node::Const(tern(vec![Trit::Zero, Trit::Pos, Trit::Neg])),
            Node::Const(tern(vec![Trit::Zero, Trit::Pos, Trit::Zero])),
        ],
    }
}

/// M-857: `trit.mul` now lowers through the real dialect path — shifted accumulation of `±a` into a
/// 2m-trit buffer (`arith.muli` per digit) plus the shared ripple adder, with the never-silent
/// overflow read-back branch (`cf.cond_br`). Asserts the genuine multiply + carry arithmetic and the
/// overflow branch are emitted (mirrors `trit_add_emits_real_ripple_carry_with_overflow_branch`).
#[test]
fn trit_mul_emits_real_shifted_accumulate_with_overflow_branch() {
    let (m, kind, width) = emit_mlir(&trit_mul_in_range()).expect("emit trit.mul");
    assert_eq!(kind, ResultKind::Ternary);
    assert_eq!(width, 3);
    // The per-digit scaling factor (`±a / 0`) is an integer multiply:
    assert!(m.contains("arith.muli"), "expected arith.muli in:\n{m}");
    // The shared balanced-ternary carry step resolves the accumulation (`x = s + 4`, `remsi`/`divsi`):
    assert!(m.contains("arith.remsi"), "expected arith.remsi in:\n{m}");
    assert!(m.contains("arith.divsi"), "expected arith.divsi in:\n{m}");
    // The never-silent overflow read-back: a conditional branch on the folded overflow flag.
    assert!(m.contains("cf.cond_br"), "expected cf.cond_br in:\n{m}");
    assert!(m.contains("^ovf:"), "expected ^ovf block in:\n{m}");
    assert!(m.contains("^ok:"), "expected ^ok block in:\n{m}");
    assert!(m.contains("func.return"));
}

#[test]
fn out_of_fragment_nodes_are_explicitly_refused() {
    // A Swap is refused (routed to interp / direct-LLVM), never silently lowered.
    let swap = Node::Swap {
        src: Box::new(Node::Const(byte(A))),
        target: Repr::Ternary { trits: 6 },
        policy: mycelium_core::ContentHash::parse("blake3:round_trip_safe").unwrap(),
    };
    match emit_mlir(&swap) {
        Err(DialectError::Unsupported(_)) => {}
        other => panic!("Swap must be Unsupported, got {other:?}"),
    }
    // The NEW boundary (M-857 moved it past `trit.mul`, which now lowers): everything richer than the
    // fixed-width bit/trit arithmetic is still refused — here a closure (`Lam`) stays on the
    // direct-LLVM / interp path. The message routes it explicitly (never a silent drop).
    let lam = Node::Lam {
        param: "x".into(),
        body: Box::new(Node::Var("x".into())),
    };
    match emit_mlir(&lam) {
        Err(DialectError::Unsupported(_)) => {}
        other => panic!("a closure (Lam) must be Unsupported (the new boundary), got {other:?}"),
    }
}

#[test]
fn toolchain_resolves_or_skips() {
    // Either the tools resolve (this container) or we get a graceful ToolchainMissing — never a
    // panic, never a silent mismatch.
    match resolve_tools() {
        Ok(t) => {
            assert!(t.mlir_opt.contains("mlir-opt"));
            assert!(t.mlir_translate.contains("mlir-translate"));
            assert!(t.llvm_major >= 1);
        }
        Err(DialectError::ToolchainMissing(_)) => {}
        Err(e) => panic!("unexpected toolchain error: {e}"),
    }
}
