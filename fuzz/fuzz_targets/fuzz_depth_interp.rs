//! Fuzz target 6 (PRIORITY — RFC-0041 / RR-29 §0.1 / §4): depth-structured L0 **reference
//! interpreter** fuzzing — the deep-nesting regression net for the remotely-reachable SIGABRT class
//! this whole wave exists to catch (W0 "safety net" wave).
//!
//! # Why this is the priority target, and why it bypasses `parse`/`elaborate`
//! `mycelium_l1::parse` already refuses over-deep source at an explicit `MAX_EXPR_DEPTH` guard
//! (`crates/mycelium-l1/src/parse.rs`, A4-02/DN-40) — so a fuzz target that goes through `parse`
//! first can never hand the L0 interpreter a Core IR [`Node`] deeper than that guard allows. But
//! `mycelium-interp` (the trusted reference interpreter — RFC-0004 §2, ADR-009) has **no depth
//! guard of its own**: its `Interpreter::step`/the crate-private `subst` are plain native Rust
//! recursion over the `Node` tree (see `crates/mycelium-interp/src/lib.rs`), and `EvalError::DepthLimit`'s
//! own doc comment claims "the reference interpreter is O(1)-stack and does not raise this" — a
//! claim this harness exists to *empirically* pressure-test, not to trust on read. RR-29 §0.1 flags
//! precisely this: the L0-interp path is the one place a deeply-nested `Node` is
//! remotely/programmatically reachable without going through the parser's guard at all — a Core IR
//! `Node` can be built directly (as this harness does, and as every other consumer of
//! `mycelium_core::Node`/`mycelium_interp::Interpreter` can), or reached via any future
//! non-parser producer (a serialized/deserialized program, a macro-expanded one, …).
//!
//! So this target constructs a **directly-nested `Node` tree** (never through `mycelium_l1::parse`)
//! at a fuzzed depth, deliberately unbounded by the L1 parser's guard, and feeds it straight to
//! `Interpreter::eval_core`. The nesting shape: `Let{x, bound: Let{x, bound: … Const(v) …, body:
//! Var(x)}, body: Var(x)}` — nested via the `bound` position so that a *single* call to
//! `Interpreter::step` on the outermost node recurses one native stack frame per nesting level
//! before reaching the innermost `Const` (the `Node::Let` arm calls `self.step(bound)` before doing
//! anything else). `subst` (also plain recursion, called on every `E-Let-Bind`) gets the same
//! treatment on the way back out. Neither is a bug in isolation — an O(depth) interpreter is a
//! reasonable trusted-base baseline — but it means a sufficiently deep, *directly-constructed*
//! `Node` is expected to raise `SIGABRT` (stack overflow) rather than return an `EvalError`, until
//! the RFC-0041 fix waves (a depth guard analogous to the parser's, or an explicit host-stack budget
//! the way `mycelium_l1::parse` already uses `mycelium_stack::with_deep_stack`) land.
//!
//! # Honesty note — do NOT try to make this target "pass"
//! This is a **regression net**, not a correctness assertion. It will, by design, surface the known
//! SIGABRT class this RFC exists to fix — that is its entire purpose (catching a regression/fix, not
//! demanding one right now). Success for *this leaf task* is: it compiles, it is structurally sound
//! (constructs genuinely deep, directly-nested `Node`s and evaluates them), and it is wired into the
//! durability tier (`just check-full`'s `cargo-fuzz` smoke) so later waves can watch it turn green.
//!
//! Guarantee tag: Empirical/Declared — this harness makes no claim the interpreter is safe at any
//! depth; it is the falsifier for that claim.
#![no_main]

use libfuzzer_sys::fuzz_target;

use mycelium_core::{Meta, Node, Payload, Provenance, Repr, Value};
use mycelium_interp::Interpreter;

#[path = "depth_common.rs"]
mod depth_common;
use depth_common::derive_depth;

/// Deliberately far beyond the L1 parser's `MAX_EXPR_DEPTH` (256) and beyond what any *parsed*
/// program could ever produce — this is exactly the point (see the module doc). Large enough that,
/// absent a depth guard, evaluation is expected to overflow a typical 8 MiB thread stack well before
/// this ceiling; bounded only so a single fuzz iteration (which builds the `Node` tree iteratively,
/// not recursively — see `build_nested_let`) stays within a sane time/memory budget even on a run
/// that never actually triggers the abort.
const MAX_DEPTH: usize = 400_000;

/// Build `depth`-deep, directly-nested `Node::Let` chain around a trivial `Const` leaf, nested via
/// the `bound` position (see the module doc for why that shape). Built **iteratively** — this
/// function itself must never be the thing that overflows the stack; only `Interpreter::eval_core`
/// evaluating the result is under test.
fn build_nested_let(depth: usize, leaf: Value) -> Node {
    let id = "d".to_owned();
    let mut node = Node::Const(leaf);
    for _ in 0..depth {
        node = Node::Let {
            id: id.clone(),
            bound: Box::new(node),
            body: Box::new(Node::Var(id.clone())),
        };
    }
    node
}

/// A trivial, well-formed `Binary{8}` zero value with an `Exact` guarantee — the leaf of the nested
/// chain. Its content is irrelevant to the depth-fuzzing purpose; only the nesting shape matters.
fn leaf_value() -> Value {
    Value::new(
        Repr::Binary { width: 8 },
        Payload::Bits(vec![false; 8]),
        Meta::exact(Provenance::Root),
    )
    .expect("a well-formed 8-bit all-zero Binary value always constructs")
}

fuzz_target!(|data: &[u8]| {
    let (depth, _rest) = derive_depth(data, MAX_DEPTH);
    let node = build_nested_let(depth, leaf_value());

    // INVARIANT UNDER TEST (not yet held — RFC-0041 is the fix): evaluation should complete with an
    // explicit Ok/Err (including a future EvalError::DepthLimit once a guard lands), never abort the
    // process. Until the fix waves land, a sufficiently large derived `depth` is EXPECTED to SIGABRT
    // here — that is this harness's reason to exist, not a defect in the harness.
    let interp = Interpreter::default();
    let _ = interp.eval_core(&node);
});
