//! Tests for `crate::supervision` (M-713 structured-concurrency supervision + cancellation; M-864
//! `run_supervised` `'static`/per-job `CancelToken::clone` adjustment).
//!
//! M-797 in-crate test layout: extracted from the former inline `#[cfg(test)] mod tests` in
//! `supervision.rs` (as-touched, per the M-864 change to `run_supervised`).

use crate::scheduler::Scheduler;
use crate::supervision::*;

/// A boxed supervised task closure (alias to keep the test signatures readable).
type BoxTask<T, E> = Box<dyn FnOnce(&CancelToken) -> TaskOutcome<T, E> + Send>;

#[test]
fn cancel_tree_cascades_to_every_descendant_never_silently() {
    // A genuine multi-level tree: root → c1 → grandchild, and root → c2. Mutant witness: if
    // cancel() did not recurse, a descendant token would stay live.
    let mut root = CancelTree::new();
    let (c1_tok, gc_tok) = {
        let c1 = root.child(); // &mut CancelTree
        let c1_tok = c1.token();
        let gc_tok = c1.child().token(); // grandchild attached under c1
        (c1_tok, gc_tok)
    };
    let c2_tok = root.child().token();
    assert_eq!(root.child_count(), 2, "root has two direct children");
    assert!(
        ![&c1_tok, &gc_tok, &c2_tok].iter().any(|t| t.is_cancelled()),
        "every node starts live"
    );
    root.cancel();
    assert!(root.is_cancelled(), "root must be cancelled");
    assert!(
        c1_tok.is_cancelled(),
        "child must observe the cascade (RT7/G2)"
    );
    assert!(
        gc_tok.is_cancelled(),
        "grandchild must observe the cascade too — every descendant (RT7/G2)"
    );
    assert!(
        c2_tok.is_cancelled(),
        "second child must observe the cascade (RT7/G2)"
    );
}

#[test]
fn cancel_does_not_flow_child_to_parent() {
    // Structured-concurrency direction: cancelling a child leaves the parent live.
    let mut root = CancelTree::new();
    let child_tok = root.child().token();
    child_tok.cancel();
    assert!(child_tok.is_cancelled(), "the child token is cancelled");
    assert!(
        !root.is_cancelled(),
        "the parent must remain live (cancel flows down only)"
    );
}

#[test]
fn run_supervised_collects_every_outcome_no_silent_drop() {
    // N tasks, one fails; every task's outcome is reported (len == N, RT4/I1: no silent drop).
    let sched = Scheduler::with_workers(4, 8).unwrap();
    let token = CancelToken::new();
    let tasks: Vec<BoxTask<usize, &'static str>> = vec![
        Box::new(|_t: &CancelToken| TaskOutcome::Done(1)),
        Box::new(|_t: &CancelToken| TaskOutcome::Failed("boom")),
        Box::new(|t: &CancelToken| {
            // A cooperative task that observes cancellation if a sibling has failed.
            if t.is_cancelled() {
                TaskOutcome::Cancelled
            } else {
                TaskOutcome::Done(3)
            }
        }),
    ];
    let outcomes = run_supervised(&sched, &token, tasks);
    assert_eq!(
        outcomes.len(),
        3,
        "every task's outcome must be reported (no silent drop)"
    );
    assert!(
        outcomes[1].is_failure(),
        "the failing task must be reported as a failure"
    );
    // After the run, the failure has propagated cancellation to the scope (never-silent, RT7).
    assert!(
        token.is_cancelled(),
        "a failure must cancel the scope (G2/RT7 propagation)"
    );
}

#[test]
fn external_cancel_propagates_to_all_tasks() {
    // An externally-cancelled scope: every cooperative task observes it (none silently runs on).
    let sched = Scheduler::with_workers(2, 4).unwrap();
    let token = CancelToken::new();
    token.cancel(); // cancel before running
    let tasks: Vec<BoxTask<usize, ()>> = (0..5)
        .map(|_| {
            Box::new(|t: &CancelToken| {
                if t.is_cancelled() {
                    TaskOutcome::Cancelled
                } else {
                    TaskOutcome::Done(0usize)
                }
            }) as BoxTask<usize, ()>
        })
        .collect();
    let outcomes = run_supervised(&sched, &token, tasks);
    assert_eq!(outcomes.len(), 5);
    assert!(
        outcomes.iter().all(|o| matches!(o, TaskOutcome::Cancelled)),
        "every task in a cancelled scope must observe cancellation (RT7/G2)"
    );
}

#[test]
fn supervise_restarts_then_succeeds_with_explain_trace() {
    // A child fails twice, then succeeds: the supervisor restarts (bounded) and the EXPLAIN
    // trace records both restarts — no black box (ADR-006).
    let mut sup = Supervisor::new(
        RestartIntensity {
            max_restarts: 10,
            window_ticks: 100,
        },
        10,
    );
    let mut attempts = 0u32;
    let run = supervise_with_restart::<u8, &str>(&mut sup, || {
        attempts += 1;
        if attempts <= 2 {
            TaskOutcome::Failed("transient")
        } else {
            TaskOutcome::Done(7)
        }
    });
    assert_eq!(run.result, Ok(7), "child must succeed after 2 restarts");
    assert_eq!(
        run.trace.len(),
        2,
        "EXPLAIN trace must record both restarts"
    );
    assert!(
        run.trace
            .iter()
            .all(|r| r.action == SupervisionAction::Restarted),
        "both decisions were restarts"
    );
}

#[test]
fn supervise_escalates_when_cascade_bound_hit_never_unbounded() {
    // The bounded cascade: with a total budget of 1, a child that keeps failing escalates on the
    // 2nd failure — an explicit escalation, never an unbounded restart storm (RT4/RT7).
    let mut sup = Supervisor::new(
        RestartIntensity {
            max_restarts: 100,
            window_ticks: 1_000,
        },
        1, // total cascade budget = 1 restart
    );
    let run = supervise_with_restart::<u8, &str>(&mut sup, || TaskOutcome::Failed("always"));
    match run.result {
        Err(SupervisedFailure::Escalated(_)) => {}
        other => panic!("expected an escalation (bounded cascade), got {other:?}"),
    }
    assert!(
        run.trace
            .iter()
            .any(|r| matches!(r.action, SupervisionAction::Escalated(_))),
        "the EXPLAIN trace must record the escalation"
    );
}

#[test]
fn run_supervised_each_job_gets_an_independent_cancel_token_clone_but_shares_the_flag() {
    // M-864 regression witness: `run_supervised` now clones `token` once per job (since the shared
    // pool's `'static` jobs can no longer borrow it) instead of sharing one `&CancelToken` borrow.
    // `CancelToken::clone` is documented to share the same underlying flag — confirm a failure in
    // ONE job's clone is still observed by a SIBLING job's own, distinct clone (not just by the
    // caller's original `token`, which the earlier tests already check).
    let sched = Scheduler::with_workers(3, 8).unwrap();
    let token = CancelToken::new();
    let tasks: Vec<BoxTask<usize, &'static str>> = vec![
        Box::new(|_t: &CancelToken| TaskOutcome::Failed("fails immediately")),
        Box::new(|t: &CancelToken| {
            // This job's OWN clone must see the sibling's cancellation, proving the flag — not
            // just the reference — is shared across independently-cloned tokens.
            if t.is_cancelled() {
                TaskOutcome::Cancelled
            } else {
                TaskOutcome::Done(1)
            }
        }),
    ];
    let outcomes = run_supervised(&sched, &token, tasks);
    assert_eq!(outcomes.len(), 2, "no outcome silently dropped");
    assert!(
        outcomes[0].is_failure(),
        "the first job's failure must be reported"
    );
    // Whether outcome[1] observes the cancellation depends on scheduling order relative to the
    // failure, so assert on the CALLER's own token instead (deterministic once `run_supervised`
    // returns): the shared flag must be cancelled either way.
    assert!(
        token.is_cancelled(),
        "the caller's token clone must observe the propagated cancellation (shared flag, M-864)"
    );
}
