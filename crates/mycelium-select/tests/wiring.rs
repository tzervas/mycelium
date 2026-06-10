//! M-222 acceptance — selection wired into the **swap-target site** (RFC-0005 §4; RFC-0002):
//! an auto-selected target drives a real `Node::Swap` through the reference interpreter with the
//! certified engine; the result records `Meta.policy_used = PolicyRef` and the selection emitted
//! its mandatory EXPLAIN; an override forces the alternate target deterministically. (The packing
//! site consumes the same `select` via `select_packing` — wired for real in E2-7/M-250.)

use mycelium_cert::CertifiedSwapEngine;
use mycelium_core::{Meta, Node, Payload, Provenance, Repr, ScalarKind, Value};
use mycelium_interp::Interpreter;
use mycelium_select::{
    select_swap_target, Action, Candidate, CostModel, Predicate, Rule, SelectionInputs,
    SelectionPolicy,
};

/// The worked policy: an exact Dense F32 value swaps to BF16 (halve the storage); otherwise stay.
fn policy() -> SelectionPolicy {
    SelectionPolicy::new(
        "bf16-when-exact",
        vec![
            Candidate::Repr(Repr::Dense {
                dim: 3,
                dtype: ScalarKind::Bf16,
            }),
            Candidate::Repr(Repr::Dense {
                dim: 3,
                dtype: ScalarKind::F32,
            }),
        ],
        vec![Rule {
            when: Predicate::DtypeIs(ScalarKind::F32),
            action: Action::Cheapest, // BF16 wins on the explicit storage cost
        }],
        1,
        CostModel {
            storage_weight: 1.0,
        },
    )
    .unwrap()
}

fn f32_value() -> Value {
    Value::new(
        Repr::Dense {
            dim: 3,
            dtype: ScalarKind::F32,
        },
        Payload::Scalars(vec![1.5, -2.25, 0.0]),
        Meta::exact(Provenance::Root),
    )
    .unwrap()
}

/// **The headline wiring:** select → build the `Swap` node with the policy's `PolicyRef` → run
/// the interpreter — the result's `Meta.policy_used` is exactly the recorded `PolicyRef`, and the
/// EXPLAIN trace exists for the selection that chose the target.
#[test]
fn auto_selected_swap_records_policy_ref_and_explains() {
    let policy = policy();
    let v = f32_value();
    let inputs = SelectionInputs::of_value(&v);
    let (target, explanation) = select_swap_target(&policy, &inputs, None).unwrap();
    assert_eq!(
        target,
        Repr::Dense {
            dim: 3,
            dtype: ScalarKind::Bf16
        }
    );
    // The mandatory EXPLAIN: the policy hash matches and every candidate was costed.
    assert_eq!(explanation.policy, policy.policy_ref());
    assert_eq!(explanation.costs.len(), 2);
    assert!(!explanation.overridden);

    // Wire it: the Swap node carries the policy's content hash (WF2), the certified engine runs it.
    let node = Node::Swap {
        src: Box::new(Node::Const(v)),
        target,
        policy: policy.policy_ref(),
    };
    let interp = Interpreter::new(
        mycelium_interp::PrimRegistry::with_builtins(),
        Box::new(CertifiedSwapEngine),
    );
    let out = interp.eval(&node).unwrap();
    assert_eq!(
        out.repr(),
        &Repr::Dense {
            dim: 3,
            dtype: ScalarKind::Bf16
        }
    );
    // "Which policy chose this?" — answerable from the value alone (RFC-0005 §3).
    assert_eq!(out.meta().policy_used(), Some(&policy.policy_ref()));
}

/// The first-class override forces the alternate target deterministically — and the run still
/// records the policy that was (overridden but) in charge.
#[test]
fn override_forces_the_alternate_target() {
    let policy = policy();
    let v = f32_value();
    let inputs = SelectionInputs::of_value(&v);
    let (target, explanation) = select_swap_target(&policy, &inputs, Some(1)).unwrap();
    assert_eq!(
        target,
        Repr::Dense {
            dim: 3,
            dtype: ScalarKind::F32
        }
    );
    assert!(explanation.overridden);
    // Determinism of the override across repeated calls.
    for _ in 0..50 {
        let (t2, e2) = select_swap_target(&policy, &inputs, Some(1)).unwrap();
        assert_eq!(t2, target);
        assert_eq!(e2, explanation);
    }
    // Same-repr target → the certified engine's identity path still runs it fine.
    let node = Node::Swap {
        src: Box::new(Node::Const(v.clone())),
        target,
        policy: policy.policy_ref(),
    };
    let interp = Interpreter::new(
        mycelium_interp::PrimRegistry::with_builtins(),
        Box::new(CertifiedSwapEngine),
    );
    let out = interp.eval(&node).unwrap();
    assert_eq!(out.repr(), v.repr());
    assert_eq!(out.payload(), v.payload());
}
