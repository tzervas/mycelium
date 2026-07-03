//! W7 — RFC-0041 §4.7 depth-guard regression tests for the AOT RC/ownership passes
//! ([`crate::eval::eval`], [`crate::emit::emit_elided`], [`crate::emit::emit_reuse`]).
//!
//! Each `*_deep_*` case feeds a chain **deeper than** the shared [`RecursionBudget`] default ceiling
//! (4096) and asserts a **clean, never-silent refusal** ([`RcError::DepthExceeded`] /
//! [`EmitError::DepthExceeded`]) rather than a host-stack SIGABRT (the §4.7 self-DoS hole this wave
//! closes). Each `*_shallow_*` case asserts the guard is **behaviour-neutral** on ordinary input
//! (unchanged success) — the guard only ever *adds* a refusal at the ceiling, it changes nothing below
//! it.
//!
//! **Fixture note (as in `tests/guard_hole_census.rs`):** a deeply-nested `Box`-chain value must be
//! built, exercised, *and dropped* on a guarded deep stack — the derived recursive `Drop` glue would
//! itself SIGABRT the test's own (default-sized) thread when the value goes out of scope, independent
//! of the pass under test. Every case therefore runs its whole construct→call→drop lifecycle inside
//! one [`ensure_sufficient_stack`] closure.

use mycelium_core::Node;
use mycelium_workstack::{ensure_sufficient_stack, RecursionBudget};

use super::common::{c, let_, val, var};
use crate::emit::{emit_elided, emit_reuse, EmitError};
use crate::eval::{eval, RcError};
use crate::rc_ir::RcNode;

/// The default depth ceiling every guard trips at (RFC-0041 §4.0/§4.4 metric).
const CEILING: u32 = RecursionBudget::DEFAULT_DEPTH_LIMIT;
/// A chain depth comfortably **beyond** [`CEILING`] — deep enough that the guard genuinely trips (and
/// that the native `Let` frames would SIGABRT an unguarded default thread stack), without needlessly
/// re-deriving the `O(N²)` per-binder re-walk residual at a larger `N`.
const DEEP: usize = 5_000;

/// A right-nested [`RcNode::Let`] chain `n` deep, innermost a constant — the deep input for [`eval`].
fn deep_rc_let(n: usize) -> RcNode {
    let mut body = RcNode::Const(val(true));
    for i in 0..n {
        body = RcNode::Let {
            id: format!("y{i}"),
            bound: Box::new(RcNode::Const(val(false))),
            body: Box::new(body),
        };
    }
    body
}

/// A right-nested [`Node::Let`] chain `n` deep, innermost referencing `x` — the deep input for the
/// annotated emitters.
fn deep_node_let(n: usize) -> Node {
    let mut body = var("x");
    for i in 0..n {
        body = let_(&format!("y{i}"), c(), body);
    }
    body
}

/// The two annotated emitters that share the [`emit_ann`](crate::emit) guarded traversal.
type EmitFn = fn(&Node) -> Result<RcNode, EmitError>;
const EMITTERS: &[(&str, EmitFn)] = &[("emit_elided", emit_elided), ("emit_reuse", emit_reuse)];

#[test]
fn eval_deep_rc_let_chain_refuses_cleanly() {
    let budget = RecursionBudget::default();
    let got = ensure_sufficient_stack(&budget, || eval(&deep_rc_let(DEEP)));
    assert_eq!(
        got,
        Err(RcError::DepthExceeded { limit: CEILING }),
        "a deeper-than-ceiling RcNode must refuse cleanly, not SIGABRT or succeed"
    );
}

#[test]
fn eval_shallow_rc_succeeds() {
    // `let x = <const> in x` — evaluates cleanly (allocate, move-consume, reclaim); the guard is
    // behaviour-neutral here.
    let shallow = RcNode::Let {
        id: "x".to_owned(),
        bound: Box::new(RcNode::Const(val(true))),
        body: Box::new(RcNode::Var("x".to_owned())),
    };
    let budget = RecursionBudget::default();
    let got = ensure_sufficient_stack(&budget, || eval(&shallow));
    assert!(
        got.is_ok(),
        "shallow input must evaluate unchanged: {got:?}"
    );
}

#[test]
fn emit_annotated_deep_let_chain_refuses_cleanly() {
    let budget = RecursionBudget::default();
    ensure_sufficient_stack(&budget, || {
        let deep = deep_node_let(DEEP);
        for (name, f) in EMITTERS {
            assert_eq!(
                f(&deep),
                Err(EmitError::DepthExceeded { limit: CEILING }),
                "{name} must refuse a deeper-than-ceiling chain cleanly, not SIGABRT"
            );
        }
    });
}

#[test]
fn emit_annotated_shallow_succeeds() {
    // `let x = <const> in x` — a single owned move; both emitters lower it without refusing.
    let budget = RecursionBudget::default();
    ensure_sufficient_stack(&budget, || {
        let shallow = let_("x", c(), var("x"));
        for (name, f) in EMITTERS {
            assert!(
                f(&shallow).is_ok(),
                "{name} must emit shallow input unchanged"
            );
        }
    });
}
