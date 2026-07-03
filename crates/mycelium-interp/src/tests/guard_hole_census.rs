//! RFC-0041 §4.7/§5 — the guard-hole **census** (W0 safety net; RR-29 guard-hole inventory turned
//! into tracked failing tests, one per hole this crate owns).
//!
//! Real repros: each test constructs a genuinely deep `Node` and calls the hole's entry point.
//! Rust's default stack-overflow handler aborts the process directly (never through panic/unwind),
//! so none of this is `catch_unwind`-able — every test here stays `#[ignore = "Wn"]`d; running one
//! for real would crash the whole test binary. When the named wave lands, drop the `#[ignore]` and
//! the call must refuse cleanly instead. White-box access via `use crate::…` (CLAUDE.md test layout).

use crate::parallel::{is_pure, plan_parallel};
use crate::Interpreter;
use mycelium_core::{ContentHash, CtorRef, Meta, Node, Payload, Provenance, Repr, Value};

fn byte() -> Value {
    Value::new(
        Repr::Binary { width: 8 },
        Payload::Bits(vec![false; 8]),
        Meta::exact(Provenance::Root),
    )
    .expect("a well-formed Binary{8} const")
}

fn ctor() -> CtorRef {
    CtorRef::new(
        ContentHash::parse("blake3:round_trip_safe").expect("a well-formed content hash"),
        0,
    )
}

/// A right-nested `Node::Construct` chain, `n` deep, every leaf already a `Const` — i.e. already a
/// normal form, so evaluating it exercises pure TRAVERSAL recursion (no reduction), not fuel.
fn deep_construct(n: usize) -> Node {
    let mut acc = Node::Const(byte());
    for _ in 0..n {
        acc = Node::Construct {
            ctor: ctor(),
            args: vec![acc],
        };
    }
    acc
}

/// A single outer `Let` whose `bound` is already a value and whose `body` is a deep `Construct`
/// chain referencing the bound variable at its innermost leaf — `step` reduces `bound` in O(1) then
/// calls `subst(body, id, bound)`, which walks the whole `body` `n`-deep in one recursive call.
fn deep_let_body(n: usize) -> Node {
    let mut body = Node::Var("x".to_owned());
    for _ in 0..n {
        body = Node::Construct {
            ctor: ctor(),
            args: vec![body],
        };
    }
    Node::Let {
        id: "x".to_owned(),
        bound: Box::new(Node::Const(byte())),
        body: Box::new(body),
    }
}

#[test]
#[ignore = "W4"] // RFC-0041 §7 W4: L0-interp (substitution) work-stack.
fn eval_core_deep_construct_refuses_cleanly() {
    // Holes: `Interpreter::step`'s `Construct` arm (recurses into non-value args) and the private
    // `node_to_core_value` (crate::lib.rs) — both walk the deep chain via `Interpreter::eval_core`.
    let deep = deep_construct(200_000);
    let result = Interpreter::default().eval_core(&deep);
    assert!(
        result.is_err(),
        "expected an explicit over-budget refusal (a constructed DepthLimit), not success or a \
         SIGABRT"
    );
}

#[test]
#[ignore = "W4"] // RFC-0041 §7 W4.
fn eval_core_deep_subst_via_let_refuses_cleanly() {
    // Hole: the private `subst` (crate::lib.rs) — invoked from `step`'s `Let`/E-Let-Bind case,
    // walking `body` (here `n` deep) to substitute the bound variable.
    let deep = deep_let_body(200_000);
    let result = Interpreter::default().eval_core(&deep);
    assert!(
        result.is_err(),
        "expected an explicit over-budget refusal (a constructed DepthLimit), not success or a \
         SIGABRT"
    );
}

/// Hole: `parallel::is_pure` (`crates/mycelium-interp/src/parallel.rs:59`).
///
/// **Honesty (FLAG, VR-5):** `is_pure` returns a plain `bool` — infallible today, so this test
/// cannot assert a "clean refusal" the way the checker/L0-interp holes above do. It constructs the
/// real repro (the call itself is the SIGABRT on a large enough `n`) and documents that RFC-0041
/// §4.7/§7 W1 is expected to route this through the shared work-step budget, at which point the
/// signature (or an internal budget check reachable from callers) gains an explicit refusal path.
#[test]
#[ignore = "W1"] // RFC-0041 §4.7/§7 W1.
fn is_pure_deep_recursion() {
    let deep = deep_construct(200_000);
    let _ = is_pure(&deep);
}

/// Hole: `parallel::plan_parallel` (`crates/mycelium-interp/src/parallel.rs:131`).
///
/// **Honesty (FLAG, VR-5):** same infallible-signature caveat as `is_pure` above — `plan_parallel`
/// returns a `ParallelPlan` struct, not a `Result`.
#[test]
#[ignore = "W1"] // RFC-0041 §4.7/§7 W1.
fn plan_parallel_deep_recursion() {
    let deep = deep_construct(200_000);
    let _ = plan_parallel(&deep);
}
