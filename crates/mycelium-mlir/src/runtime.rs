//! **RT2 deterministic fork/join executor** (M-357; RFC-0008 R1, §4.6/§4.7).
//!
//! The v0 slice of the runtime the RFC-0008 §4.7 composition primitives were built for: a
//! **deterministic, cooperative fork/join scheduler** over pure computations. It runs *outside* the
//! kernel (RT2 — "concurrency adds scheduling outside the kernel, never new meaning inside it"; KC-3):
//! the trusted evaluator stays sequential, and this layer only *schedules* tasks that each evaluate the
//! unchanged calculus.
//!
//! ## What lands here (the chosen scope)
//! - **Structured fork/join** ([`Scope`]): tasks are spawned into a scope that **joins all of them**
//!   before it returns — no task outlives its scope (RT7: "a leaked task is not expressible").
//! - **Per-task budgets + cancellation**: every task carries its own [`Budgets`] ledger (M-353) and a
//!   shared [`CancelToken`] (M-356); cancellation is cooperative (observed between steps), never
//!   preemptive, and yields an explicit additive [`TaskOutcome::Cancelled`] (I1).
//! - **The RT2 sequentialization guarantee** (the heart): tasks are **pure over immutable values with
//!   no shared state** (RT1), so a *deterministic interleaved* schedule and the *sequential* run in
//!   spawn order produce the **identical** per-task outcomes. That equivalence is the
//!   NFR-7-extension RFC-0008 §4.6 names — verified by [`tests`] across an interleaving corpus and a
//!   real-L0-evaluation corpus (each task runs the env-machine).
//!
//! ## What does **not** land here (honest boundary)
//! Typed single-producer/single-consumer **channels** (the other half of the RT2 fragment, the Kahn
//! determinism for *communicating* tasks) are the next slice; nondeterministic forms (`select`,
//! placement) stay RT3 constructs with reified policies — out of scope. With no channels, the fragment
//! is pure fork/join and its sequentialization is exactly the spawn-order sequential run.

use mycelium_interp::{Budgets, CancelToken, TaskOutcome};

/// The result of advancing a task one cooperative step.
pub enum Poll<T, E> {
    /// The task has more work; it yielded so siblings can run (cooperative, deterministic).
    Pending,
    /// The task resolved to its final, explicit [`TaskOutcome`].
    Ready(TaskOutcome<T, E>),
}

/// The per-step context a task observes (the same cadence it would check fuel/depth): its cancel token
/// and its **own** per-task budget ledger (RFC-0008 §4.7 C1/C2). `tick` is the scheduler's logical
/// clock (deterministic; not wall-clock — R8-Q3).
pub struct TaskCtx<'a> {
    /// The cooperative cancellation token (shared down the scope tree; RT7).
    pub cancel: &'a CancelToken,
    /// This task's own budget ledger — an overrun is an in-that-task refusal (C1).
    pub budgets: &'a mut Budgets,
    /// The scheduler's logical tick at this step.
    pub tick: u64,
}

/// A cooperative task: `poll` advances it by one step. A task must be **pure over immutable values**
/// (RT1) — it owns its local state and shares nothing mutable with siblings, which is exactly what
/// makes its outcome schedule-independent (RT2). A well-behaved task observes `cx.cancel` at the top of
/// each step so cancellation is honoured promptly (but cooperatively).
pub trait Task {
    /// The success value type.
    type Output;
    /// The explicit error type.
    type Error;
    /// Advance one step.
    fn poll(&mut self, cx: &mut TaskCtx) -> Poll<Self::Output, Self::Error>;
}

/// A task plus the state the scope tracks for it: its own budget ledger and its resolved outcome.
struct Child<T, E> {
    task: Box<dyn Task<Output = T, Error = E>>,
    budgets: Budgets,
    outcome: Option<TaskOutcome<T, E>>,
}

/// A **structured concurrency scope** (RT7): tasks spawned here are all joined before the scope
/// returns. Two execution strategies — a deterministic *interleaved* schedule and the *sequential*
/// reference — that the RT2 differential proves observationally equal (over pure tasks).
pub struct Scope<T, E> {
    children: Vec<Child<T, E>>,
    cancel: CancelToken,
}

/// A **`colony`** — the DN-06 dynamic runtime grouping of active `hypha` (a cooperating set of
/// concurrent tasks under a shared scope + supervision). The structured-concurrency [`Scope`] *is*
/// this concept; `Colony` is the ratified surface vocabulary adopted going forward (DN-06; RFC-0008
/// §4.7). The static `colony` keyword (DN-02) migrates to `nodule` under M-358, freeing the term for
/// this dynamic meaning.
pub type Colony<T, E> = Scope<T, E>;

impl<T, E> Default for Scope<T, E> {
    fn default() -> Self {
        Scope {
            children: Vec::new(),
            cancel: CancelToken::new(),
        }
    }
}

impl<T, E> Scope<T, E> {
    /// A fresh scope with its own cancel token.
    #[must_use]
    pub fn new() -> Self {
        Scope::default()
    }

    /// The scope's cancel token — cancelling it cooperatively cancels every child (RT7).
    #[must_use]
    pub fn cancel_token(&self) -> CancelToken {
        self.cancel.clone()
    }

    /// Spawn a task into the scope, carrying its own per-task `budgets` ledger (C1). Returns the
    /// task's index, which indexes the outcome vector the `run_*` methods produce (spawn order).
    pub fn spawn(&mut self, task: Box<dyn Task<Output = T, Error = E>>, budgets: Budgets) -> usize {
        self.children.push(Child {
            task,
            budgets,
            outcome: None,
        });
        self.children.len() - 1
    }

    /// The **sequential reference run** (RT2): poll each child to completion in spawn order. This is the
    /// deterministic sequentialization the interleaved schedule must match.
    #[must_use]
    pub fn run_sequential(mut self) -> Vec<TaskOutcome<T, E>> {
        let mut tick = 0u64;
        for child in &mut self.children {
            loop {
                tick += 1;
                let mut cx = TaskCtx {
                    cancel: &self.cancel,
                    budgets: &mut child.budgets,
                    tick,
                };
                if let Poll::Ready(o) = child.task.poll(&mut cx) {
                    child.outcome = Some(o);
                    break;
                }
            }
        }
        self.join()
    }

    /// The **deterministic interleaved run** (RT2): round-robin one step per still-pending child until
    /// all resolve. The order (ascending child index, repeated) is fixed, so the schedule is
    /// reproducible — and because children share no mutable state (RT1), the outcomes equal
    /// [`run_sequential`](Scope::run_sequential)'s (the RT2 sequentialization guarantee).
    ///
    /// `trace` (when `Some`) records the child index polled at each step, so a test can confirm the
    /// schedule genuinely interleaves (the equivalence is non-trivial), not that it secretly ran
    /// sequentially.
    #[must_use]
    pub fn run_interleaved(mut self, mut trace: Option<&mut Vec<usize>>) -> Vec<TaskOutcome<T, E>> {
        let mut tick = 0u64;
        let mut remaining = self.children.len();
        while remaining > 0 {
            for i in 0..self.children.len() {
                if self.children[i].outcome.is_some() {
                    continue;
                }
                tick += 1;
                if let Some(t) = trace.as_deref_mut() {
                    t.push(i);
                }
                let child = &mut self.children[i];
                let mut cx = TaskCtx {
                    cancel: &self.cancel,
                    budgets: &mut child.budgets,
                    tick,
                };
                if let Poll::Ready(o) = child.task.poll(&mut cx) {
                    child.outcome = Some(o);
                    remaining -= 1;
                }
            }
        }
        self.join()
    }

    /// Join: collect every child's resolved outcome in spawn order. A scope never returns with an
    /// unresolved child (RT7) — `run_*` only call this once all children are `Some`.
    fn join(self) -> Vec<TaskOutcome<T, E>> {
        self.children
            .into_iter()
            .map(|c| {
                c.outcome
                    .expect("a joined scope has resolved every child (RT7)")
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mycelium_interp::{EffectBudget, EffectKind};

    /// A pure counting task: yields `steps` times, then completes with `value`. Owns all its state
    /// (RT1) and observes cancellation each step (RT7/C2).
    struct Counter {
        remaining: u32,
        value: u64,
    }

    impl Task for Counter {
        type Output = u64;
        type Error = String;
        fn poll(&mut self, cx: &mut TaskCtx) -> Poll<u64, String> {
            if cx.cancel.check().is_err() {
                return Poll::Ready(TaskOutcome::Cancelled);
            }
            if self.remaining == 0 {
                Poll::Ready(TaskOutcome::Done(self.value))
            } else {
                self.remaining -= 1;
                Poll::Pending
            }
        }
    }

    fn counters() -> Scope<u64, String> {
        let mut scope = Scope::new();
        // Different step counts force a genuine interleave (a task finishing early drops out).
        scope.spawn(
            Box::new(Counter {
                remaining: 1,
                value: 10,
            }),
            Budgets::new(),
        );
        scope.spawn(
            Box::new(Counter {
                remaining: 3,
                value: 20,
            }),
            Budgets::new(),
        );
        scope.spawn(
            Box::new(Counter {
                remaining: 2,
                value: 30,
            }),
            Budgets::new(),
        );
        scope
    }

    #[test]
    fn rt2_interleaved_equals_sequential_and_genuinely_interleaves() {
        // The RT2 sequentialization guarantee: the deterministic interleaved schedule and the
        // sequential reference produce the IDENTICAL per-task outcomes (RT1 purity ⇒ schedule-free).
        let seq = counters().run_sequential();
        let mut trace = Vec::new();
        let inter = counters().run_interleaved(Some(&mut trace));
        assert_eq!(
            seq, inter,
            "RT2: concurrent observable ≡ deterministic sequentialization"
        );
        assert_eq!(
            seq,
            vec![
                TaskOutcome::Done(10),
                TaskOutcome::Done(20),
                TaskOutcome::Done(30)
            ]
        );
        // The interleave is real: the first three steps poll children 0,1,2 in turn (not 0,0,0…),
        // proving the equivalence is non-trivial.
        assert_eq!(
            &trace[..3],
            &[0, 1, 2],
            "the schedule genuinely interleaves"
        );
    }

    #[test]
    fn rt7_cancelling_the_scope_cancels_pending_children_additively() {
        // Cancelling the scope before the run → every child observes cancellation and resolves to an
        // explicit, additive Cancelled (I1); the scope still joins them all (RT7 — none is left).
        let scope = counters();
        scope.cancel_token().cancel();
        let outcomes = scope.run_interleaved(None);
        assert_eq!(
            outcomes.len(),
            3,
            "RT7: every child is joined, none orphaned"
        );
        assert!(
            outcomes.iter().all(|o| *o == TaskOutcome::Cancelled),
            "a cancelled scope yields an explicit Cancelled per child (additive, never silent)"
        );
    }

    /// A task that spends one unit of its own per-task `alloc` budget each step — used to show a
    /// per-task overrun is in-that-task and does not perturb a sibling (C1).
    struct Spender {
        steps: u32,
    }

    impl Task for Spender {
        type Output = u64;
        type Error = String;
        fn poll(&mut self, cx: &mut TaskCtx) -> Poll<u64, String> {
            if self.steps == 0 {
                return Poll::Ready(TaskOutcome::Done(0));
            }
            self.steps -= 1;
            match cx.budgets.consume(EffectKind::Alloc, 1) {
                Ok(()) => Poll::Pending,
                Err(e) => Poll::Ready(TaskOutcome::BudgetExhausted(e)),
            }
        }
    }

    #[test]
    fn c1_a_per_task_budget_overrun_is_isolated_to_that_task() {
        // Two tasks: the first has too little budget and overruns (in-that-task BudgetExhausted); the
        // second has ample budget and completes — one task's overrun never exhausts another's (C1).
        let mut scope: Scope<u64, String> = Scope::new();
        scope.spawn(
            Box::new(Spender { steps: 5 }),
            Budgets::new().with(EffectBudget::Bytes(2)), // too little → overruns
        );
        scope.spawn(
            Box::new(Spender { steps: 2 }),
            Budgets::new().with(EffectBudget::Bytes(100)), // ample → completes
        );
        let out = scope.run_interleaved(None);
        assert!(
            matches!(out[0], TaskOutcome::BudgetExhausted(_)),
            "task 0 overruns its own budget"
        );
        assert_eq!(
            out[1],
            TaskOutcome::Done(0),
            "task 1 is unaffected (isolation, C1)"
        );
    }

    // --- the RT2 differential over the REAL calculus: each task runs the env-machine ---

    use mycelium_core::{CoreValue, Meta, Node, Payload, Provenance, Repr, Value};
    use mycelium_interp::{EvalError, IdentitySwapEngine, PrimRegistry};

    fn byte(bits: [bool; 8]) -> Value {
        Value::new(
            Repr::Binary { width: 8 },
            Payload::Bits(bits.to_vec()),
            Meta::exact(Provenance::Root),
        )
        .unwrap()
    }

    /// `not(<byte>)` — a real L0 program (the same `bit.not` fragment the M-151 differential uses).
    fn not_prog(b: [bool; 8]) -> Node {
        Node::Op {
            prim: "bit.not".into(),
            args: vec![Node::Const(byte(b))],
        }
    }

    /// A task that evaluates an L0 program through the env-machine in one step, threading its own
    /// per-task budget ledger (the same `Budgets` the scope owns for it). Pure — it reads the shared
    /// prim/swap registries but mutates no shared state (RT1).
    struct EvalTask {
        node: Node,
        done: bool,
    }

    impl Task for EvalTask {
        type Output = CoreValue;
        type Error = EvalError;
        fn poll(&mut self, cx: &mut TaskCtx) -> Poll<CoreValue, EvalError> {
            if cx.cancel.check().is_err() {
                return Poll::Ready(TaskOutcome::Cancelled);
            }
            if self.done {
                // Defensive: a resolved task is not re-polled by the scheduler.
                return Poll::Pending;
            }
            self.done = true;
            let prims = PrimRegistry::with_builtins();
            match crate::run_core_with_effects(
                &self.node,
                &prims,
                &IdentitySwapEngine,
                1_000_000,
                1_000_000,
                cx.budgets,
            ) {
                Ok(v) => Poll::Ready(TaskOutcome::Done(v)),
                Err(e) => Poll::Ready(TaskOutcome::Failed(e)),
            }
        }
    }

    fn eval_scope() -> Scope<CoreValue, EvalError> {
        let mut scope = Scope::new();
        for prog in [
            not_prog([true, false, true, true, false, false, true, false]),
            not_prog([false; 8]),
            not_prog([true; 8]),
        ] {
            scope.spawn(
                Box::new(EvalTask {
                    node: prog,
                    done: false,
                }),
                Budgets::new(),
            );
        }
        scope
    }

    #[test]
    fn rt2_differential_over_the_real_env_machine() {
        // The genuine RT2 obligation: tasks that each run the unchanged env-machine produce the same
        // outcomes whether scheduled interleaved or sequentially (RT1 isolation ⇒ RT2 determinism),
        // and each equals the plain single-task evaluation of the same program (no new meaning — KC-3).
        let seq = eval_scope().run_sequential();
        let inter = eval_scope().run_interleaved(None);
        assert_eq!(seq, inter, "RT2: env-machine tasks agree across schedules");

        // …and each task's outcome equals the standalone env-machine run of its program.
        let prims = PrimRegistry::with_builtins();
        for (i, prog) in [
            not_prog([true, false, true, true, false, false, true, false]),
            not_prog([false; 8]),
            not_prog([true; 8]),
        ]
        .into_iter()
        .enumerate()
        {
            let standalone = crate::run_core(&prog, &prims, &IdentitySwapEngine).unwrap();
            assert_eq!(
                seq[i],
                TaskOutcome::Done(standalone),
                "task {i}'s scheduled outcome must equal the standalone evaluation"
            );
        }
    }
}
