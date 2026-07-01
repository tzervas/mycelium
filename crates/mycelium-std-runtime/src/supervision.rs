//! Structured-concurrency supervision + cancellation (M-713 / RFC-0008 RT4·RT7 / E12-1).
//!
//! RFC-0008 RT7: a scope does not exit until every child completes or is cancelled — "an orphan
//! hypha is not expressible". The scheduler-independent composition kernel (M-356) already provides
//! [`CancelToken`], the explicit [`TaskOutcome`], and the bounded-cascade [`Supervisor`]; this
//! module makes them **execute end-to-end on the OS-thread pool** (M-709):
//!
//! - [`CancelTree`] — a cancellation **tree**: cancelling a node cascades to every descendant
//!   (parent→child, never child→parent), so a cancelled colony propagates to all its children (RT7).
//! - [`run_supervised`] — runs a task set on the [`Scheduler`], collects **every** child's explicit
//!   [`TaskOutcome`] (never a dropped/silent variant — RT4/I1), and on the first failure cancels the
//!   remaining siblings (never-silent propagation, G2).
//! - [`supervise_with_restart`] — a live restart policy ([`Supervisor`] bounds) that is
//!   **EXPLAIN-able**: each decision is a reified [`SupervisionRecord`] (no black boxes; ADR-006).
//!
//! # Honesty (VR-5)
//!
//! - Cancellation propagation + outcome collection are **`Empirical`** (property-tested; cooperative
//!   observation, not preemption — a task sees cancellation at its own checkpoints).
//! - The restart bound (rate + total cascade) is **`Exact`** — inherited from [`Supervisor`], whose
//!   bounds are enforced structurally (M-356).

pub use mycelium_interp::{
    CancelToken, Cancelled, Escalation, RestartIntensity, Supervisor, TaskOutcome,
};

use mycelium_core::GuaranteeStrength;

use crate::scheduler::Scheduler;

/// Guarantee strength for cancellation propagation + explicit outcome collection.
pub const SUPERVISION_PROPAGATION_STRENGTH: GuaranteeStrength = GuaranteeStrength::Empirical;

/// Guarantee strength for the restart bound (inherited from the M-356 [`Supervisor`]).
pub const SUPERVISION_RESTART_BOUND_STRENGTH: GuaranteeStrength = GuaranteeStrength::Exact;

/// A cancellation **tree** (RFC-0008 RT7): a node with its own [`CancelToken`] and child tokens.
///
/// Cancelling a node cascades to **every descendant** (parent→child), so cancelling a colony
/// propagates failure to all its children — never-silent (G2). Cancellation never flows the other
/// way: a child cancel leaves the parent live (structured-concurrency direction).
///
/// Deliberately **not `Clone`**: cloning would deep-copy the child subtree, so attaching a child to
/// one clone after the split would not cascade from the other — silently violating the
/// "cancels every descendant" contract. Share a node's cancellation via [`token`](CancelTree::token)
/// (the [`CancelToken`] *is* `Clone`, sharing one flag) instead of cloning the tree.
#[derive(Debug, Default)]
pub struct CancelTree {
    token: CancelToken,
    children: Vec<CancelTree>,
}

impl CancelTree {
    /// A fresh, un-cancelled root.
    #[must_use]
    pub fn new() -> Self {
        CancelTree {
            token: CancelToken::new(),
            children: Vec::new(),
        }
    }

    /// This node's cooperative cancel token (clones share the same flag).
    #[must_use]
    pub fn token(&self) -> CancelToken {
        self.token.clone()
    }

    /// Attach a fresh child scope and return a mutable handle to it, so callers can build a genuine
    /// multi-level tree (attach grandchildren via the returned node's own [`child`](CancelTree::child)).
    /// The child — and everything attached under it — is cancelled if **this** node is later
    /// cancelled (the cascade), but cancelling the child does not cancel this node (RT7). Use
    /// [`token`](CancelTree::token) on the returned handle to get its cooperative cancel token.
    pub fn child(&mut self) -> &mut CancelTree {
        self.children.push(CancelTree::new());
        self.children
            .last_mut()
            .expect("a child was just pushed, so last_mut is Some")
    }

    /// Whether this node has been cancelled.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.token.is_cancelled()
    }

    /// Cancel this node **and every descendant** — the never-silent cascade (G2/RT7). Idempotent.
    pub fn cancel(&self) {
        self.token.cancel();
        for c in &self.children {
            c.cancel();
        }
    }

    /// The number of direct child scopes (for inspection/tests).
    #[must_use]
    pub fn child_count(&self) -> usize {
        self.children.len()
    }
}

/// Run `tasks` on the OS-thread pool under a shared [`CancelToken`], collecting **every** child's
/// explicit [`TaskOutcome`] in spawn order (RT4/I1 — no outcome is ever silently dropped).
///
/// Each task is a closure `FnOnce(&CancelToken) -> TaskOutcome<T, E>`: it observes the token
/// cooperatively (at its own checkpoints). The **never-silent failure-propagation** contract: if a
/// task returns a failure outcome ([`TaskOutcome::is_failure`]), it cancels the shared token, so
/// still-running siblings that next check the token resolve to [`TaskOutcome::Cancelled`] — a
/// cancelled scope never silently leaks a sibling (G2/RT7). Every task's outcome is still reported.
///
/// Guarantee: **`Empirical`** ([`SUPERVISION_PROPAGATION_STRENGTH`]).
#[must_use]
pub fn run_supervised<T, E, F>(
    scheduler: &Scheduler,
    token: &CancelToken,
    tasks: Vec<F>,
) -> Vec<TaskOutcome<T, E>>
where
    F: FnOnce(&CancelToken) -> TaskOutcome<T, E> + Send,
    T: Send,
    E: Send,
{
    let jobs: Vec<_> = tasks
        .into_iter()
        .map(|task| {
            move || {
                let outcome = task(token);
                if outcome.is_failure() {
                    // Never-silent propagation: a failure cancels the scope so siblings observe it
                    // at their next checkpoint and resolve to Cancelled (RT7/G2), never leak.
                    token.cancel();
                }
                outcome
            }
        })
        .collect();
    scheduler.run_indexed(jobs, None, None)
}

/// What a supervisor did about one child failure — a reified, inspectable action (EXPLAIN; no black
/// boxes — ADR-006).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupervisionAction {
    /// The child was restarted (under the bounded cascade).
    Restarted,
    /// The restart cascade hit a bound; the supervisor escalated (its own explicit failure).
    Escalated(Escalation),
}

/// One reified supervision decision — the EXPLAIN record (RFC-0008 §4.7; ADR-006: selections are
/// inspectable, never silent). A driver emits one of these per child failure it handles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupervisionRecord {
    /// The supervisor's logical tick at which the decision was made (RestartIntensity clock).
    pub logical_tick: u64,
    /// Restarts already consumed from the total `cascade` budget before this decision.
    pub restarts_before: u64,
    /// What the supervisor did.
    pub action: SupervisionAction,
}

/// The result of supervising a restartable child to resolution or escalation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupervisedRun<T> {
    /// The final outcome: `Ok(value)` if the child eventually succeeded, `Err` if the supervisor
    /// escalated (a bounded cascade hit a bound — an explicit failure, never an unbounded storm).
    pub result: Result<T, SupervisedFailure>,
    /// The EXPLAIN trace: every restart/escalation decision, in order (no black boxes; ADR-006).
    pub trace: Vec<SupervisionRecord>,
}

/// Why a supervised child run ended in failure — always explicit (G2).
///
/// The child's *transient* per-attempt errors are not surfaced here: each one is **handled** by a
/// restart and recorded in the EXPLAIN trace as a [`SupervisionAction::Restarted`] decision (so no
/// failure is silently dropped — RT4/I1), and the terminal outcome is exactly one of these two
/// explicit cases. (The supervisor's contract is "restart on failure, escalate when the bound is
/// hit"; a non-restartable error is not a case `supervise_with_restart` itself produces.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupervisedFailure {
    /// The supervisor escalated: the restart cascade hit a bound (rate or total).
    Escalated(Escalation),
    /// The child was cancelled (cooperative; RT7).
    Cancelled,
}

/// Run a restartable child under a live [`Supervisor`] (M-356) until it succeeds or the supervisor
/// escalates, recording an EXPLAIN [`SupervisionRecord`] for each decision (RFC-0008 RT7/RT4).
///
/// `attempt` runs the child once and returns its [`TaskOutcome`]. On a failure outcome the
/// supervisor tries to restart (consuming the bounded cascade); on success it returns the value with
/// the full trace; on a bound hit it escalates explicitly (never an unbounded restart storm). The
/// child's `E` is the per-attempt error type; transient failures are *handled* by restart (traced),
/// so the terminal [`SupervisedRun`] does not itself carry an `E`.
///
/// Guarantee: the restart **bound** is **`Exact`** ([`SUPERVISION_RESTART_BOUND_STRENGTH`], inherited
/// from [`Supervisor`]); the trace is exact (every decision is recorded).
pub fn supervise_with_restart<T, E>(
    supervisor: &mut Supervisor,
    mut attempt: impl FnMut() -> TaskOutcome<T, E>,
) -> SupervisedRun<T> {
    let mut trace = Vec::new();
    let mut restarts_before = 0u64; // honest local count of restarts already granted this run
    loop {
        match attempt() {
            TaskOutcome::Done(v) => {
                return SupervisedRun {
                    result: Ok(v),
                    trace,
                };
            }
            TaskOutcome::Cancelled => {
                return SupervisedRun {
                    result: Err(SupervisedFailure::Cancelled),
                    trace,
                };
            }
            TaskOutcome::Failed(_) | TaskOutcome::BudgetExhausted(_) => {
                // A failure: try to restart under the bounded cascade. Record the decision (EXPLAIN).
                let tick = supervisor.tick();
                match supervisor.record_restart() {
                    Ok(()) => {
                        trace.push(SupervisionRecord {
                            logical_tick: tick,
                            restarts_before,
                            action: SupervisionAction::Restarted,
                        });
                        restarts_before += 1;
                        // Loop: re-attempt the child (the restart).
                    }
                    Err(escalation) => {
                        trace.push(SupervisionRecord {
                            logical_tick: tick,
                            restarts_before,
                            action: SupervisionAction::Escalated(escalation.clone()),
                        });
                        return SupervisedRun {
                            result: Err(SupervisedFailure::Escalated(escalation)),
                            trace,
                        };
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
