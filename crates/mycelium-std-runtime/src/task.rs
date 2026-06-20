//! Task, TaskCtx, Poll, SweepOrder, Deadlock — task surface (ADR-020 v0 R1).
//!
//! # Guarantee (Declared — Task purity contract)
//!
//! `Task` purity is **Declared**: the type system cannot enforce that a task body
//! has no side effects, so this is an assertion-level guarantee (VR-5: not upgraded
//! to Empirical/Proven without a checked basis).

use mycelium_core::GuaranteeStrength;

/// Guarantee strength for the `Task` purity contract.
pub const TASK_PURITY_STRENGTH: GuaranteeStrength = GuaranteeStrength::Declared;

/// A computation that can be spawned into a `Scope`.
///
/// Guarantee: **Declared** — purity contract is asserted, not enforced by the type system.
/// Out-of-scope effects are a `wild`-level concern (ADR-014).
pub struct Task {
    _priv: (),
}

impl Task {
    /// Construct a task from a closure. The caller asserts purity (Declared).
    pub fn new<F: FnOnce() + Send + 'static>(_f: F) -> Self {
        Task { _priv: () }
    }
}

/// Context passed to a running task — carries cancellation signal and scope ref.
pub struct TaskCtx {
    cancelled: bool,
}

impl TaskCtx {
    pub fn new() -> Self {
        TaskCtx { cancelled: false }
    }

    /// Returns `true` if this task's scope has been cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Default for TaskCtx {
    fn default() -> Self {
        Self::new()
    }
}

/// Poll result for an async task step.
#[derive(Debug, PartialEq, Eq)]
pub enum Poll<T> {
    Ready(T),
    Pending,
}

/// Order in which tasks are swept from a scope's run queue.
///
/// Guarantee: **Exact** — the sweep order is deterministic given the same queue state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SweepOrder {
    /// FIFO (default): tasks are completed in the order they were spawned.
    #[default]
    Fifo,
    /// Priority: tasks are swept highest-priority first.
    Priority,
}

/// Deadlock descriptor: returned when a scope cannot make progress.
///
/// Guarantee: **Empirical** — detection is complete for the supported channel graph
/// (DAG channels); cyclic graphs are an open follow-up (FLAG: ADR-020 §7).
#[derive(Debug, PartialEq, Eq)]
pub struct Deadlock {
    pub task_count: usize,
}

impl Deadlock {
    pub fn new(task_count: usize) -> Self {
        Deadlock { task_count }
    }
}
