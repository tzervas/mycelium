//! `Scheduler` — a real **OS-thread** scheduler (M-709 / RFC-0008 RT1·RT2 / E12-1).
//!
//! The v0 R1 surface ([`crate::colony`]) ran tasks cooperatively on the calling thread. This
//! module grows that into a real scheduler: a fixed pool of OS worker threads runs independent
//! tasks in parallel, with **fair (FIFO) dispatch** and **demand-signalled backpressure** — never
//! an unbounded silent buffer (G2 / RFC-0008 §4.3: "an unbounded silent buffer is a hidden resource
//! leak and is excluded by construction").
//!
//! # Honesty (VR-5)
//!
//! - **RT2 sequentialization differential — `Empirical`.** Because tasks share no mutable state
//!   (RT1), running them across OS threads yields outcomes equal to the spawn-order sequential
//!   reference. This is *checked* by a property test ([`tests`]), not assumed — but it is not
//!   `Proven` (no mechanized theorem), so it stays `Empirical`.
//! - **Backpressure bound — `Exact`.** The ready queue holds **at most** `capacity` pending jobs
//!   *by construction*: a job is enqueued only while `len < capacity`, under the queue lock. The
//!   bound is a structural invariant, asserted by a mutant-witness test.
//! - **Liveness (every submitted job runs exactly once) — `Empirical`.** Property-tested over
//!   random job sets; not `Proven`.
//!
//! The crate stays `#![forbid(unsafe_code)]`: the pool uses [`std::thread::scope`] (so worker
//! threads may borrow the jobs without a `'static` bound) and a `Mutex`/`Condvar` ready queue.

use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};
use std::thread;

use mycelium_core::GuaranteeStrength;

/// Guarantee strength for the scheduler's RT2 sequentialization differential.
///
/// `Empirical`: the parallel run equals the sequential reference by RT1 (no shared mutable state),
/// checked by a property test — not `Proven` (no mechanized theorem). (RFC-0008 RT2.)
pub const SCHEDULER_RT2_STRENGTH: GuaranteeStrength = GuaranteeStrength::Empirical;

/// Guarantee strength for the demand-signalled backpressure bound.
///
/// `Exact`: the ready queue never exceeds `capacity` *by construction* (enqueue only while
/// `len < capacity`, under the lock). (RFC-0008 §4.3.)
pub const SCHEDULER_BACKPRESSURE_STRENGTH: GuaranteeStrength = GuaranteeStrength::Exact;

/// Guarantee strength for liveness (every submitted job runs exactly once).
///
/// `Empirical`: property-tested over random job sets; not `Proven`.
pub const SCHEDULER_LIVENESS_STRENGTH: GuaranteeStrength = GuaranteeStrength::Empirical;

/// Why constructing a [`Scheduler`] refused — always explicit, never a silent fallback (G2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerError {
    /// A scheduler with zero workers can make no progress; rejected at construction (fail-closed,
    /// G2) rather than silently substituting a single worker.
    ZeroWorkers,
    /// A ready queue with zero capacity can never accept a job; rejected at construction (G2).
    ZeroCapacity,
}

impl std::fmt::Display for SchedulerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchedulerError::ZeroWorkers => f.write_str(
                "scheduler refused: zero workers cannot make progress (G2: fail-closed, never a \
                 silent single-worker substitution)",
            ),
            SchedulerError::ZeroCapacity => f.write_str(
                "scheduler refused: a zero-capacity ready queue can never accept a job (G2: \
                 fail-closed)",
            ),
        }
    }
}

impl std::error::Error for SchedulerError {}

/// A real OS-thread scheduler: a fixed worker pool with fair FIFO dispatch and a **bounded**,
/// demand-signalled ready queue (RFC-0008 RT1·RT2·§4.3).
///
/// # Guarantee
/// - RT2 sequentialization: **`Empirical`** ([`SCHEDULER_RT2_STRENGTH`]).
/// - Backpressure bound (queue ≤ `capacity`): **`Exact`** ([`SCHEDULER_BACKPRESSURE_STRENGTH`]).
/// - Liveness (each job runs once): **`Empirical`** ([`SCHEDULER_LIVENESS_STRENGTH`]).
#[derive(Debug, Clone, Copy)]
pub struct Scheduler {
    workers: usize,
    capacity: usize,
}

impl Scheduler {
    /// A scheduler sized to the host's available parallelism (fallback: 1 worker), with a ready
    /// queue capacity of `2 × workers` (room for one in-flight job per worker plus one queued).
    ///
    /// Guarantee: **Exact** (construction is deterministic given the probed parallelism).
    #[must_use]
    pub fn new() -> Self {
        let workers = thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
        Scheduler {
            workers,
            capacity: workers.saturating_mul(2),
        }
    }

    /// A scheduler with exactly `workers` OS threads and a ready-queue `capacity`.
    ///
    /// # Errors
    /// [`SchedulerError::ZeroWorkers`] if `workers == 0`; [`SchedulerError::ZeroCapacity`] if
    /// `capacity == 0` (both fail-closed — G2: never a silent substitution).
    pub fn with_workers(workers: usize, capacity: usize) -> Result<Self, SchedulerError> {
        if workers == 0 {
            return Err(SchedulerError::ZeroWorkers);
        }
        if capacity == 0 {
            return Err(SchedulerError::ZeroCapacity);
        }
        Ok(Scheduler { workers, capacity })
    }

    /// The number of OS worker threads this scheduler runs.
    #[must_use]
    pub fn workers(&self) -> usize {
        self.workers
    }

    /// The bounded ready-queue depth — the backpressure ceiling (never exceeded; G2).
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Run `jobs` across the OS-thread pool and return their outputs **in spawn order** (so the
    /// result vector is directly comparable to the sequential reference — the RT2 differential).
    ///
    /// Dispatch is fair (FIFO): workers pull jobs in submission order; every job runs exactly once
    /// (liveness, `Empirical`). The ready queue is bounded at [`capacity`](Scheduler::capacity): the
    /// feeder enqueues a job only while the queue has room, blocking otherwise — demand-signalled
    /// backpressure, never an unbounded silent buffer (G2 / RFC-0008 §4.3).
    ///
    /// `peak_depth` (when `Some`) records the **maximum** observed ready-queue depth, so a test can
    /// confirm the bound is real (it must be `≤ capacity`).
    ///
    /// Guarantee: outputs equal the sequential reference — **`Empirical`** (RT2; RT1 ⇒
    /// schedule-independence). Pure tasks only (the [`crate::task`] purity contract is `Declared`).
    #[must_use]
    pub fn run_indexed<T, F>(&self, jobs: Vec<F>, peak_depth: Option<&mut usize>) -> Vec<T>
    where
        F: FnOnce() -> T + Send,
        T: Send,
    {
        let n = jobs.len();
        if n == 0 {
            return Vec::new();
        }

        // Ready queue: (spawn index, job). Bounded at `capacity`; FIFO. `closed` signals the
        // feeder is done so idle workers can exit instead of waiting forever (never a silent hang).
        struct Queue<F> {
            ready: VecDeque<(usize, F)>,
            closed: bool,
            peak: usize,
        }
        let queue = Mutex::new(Queue::<F> {
            ready: VecDeque::new(),
            closed: false,
            peak: 0,
        });
        let not_empty = Condvar::new(); // a worker waits here for work (or close)
        let not_full = Condvar::new(); // the feeder waits here for queue room (backpressure)

        // Results, pre-sized and written by spawn index → the output stays in spawn order (RT2).
        let results = Mutex::new(Vec::<Option<T>>::with_capacity(n));
        {
            let mut r = results.lock().expect("results mutex poisoned");
            r.resize_with(n, || None);
        }

        let capacity = self.capacity;
        let workers = self.workers.min(n); // no point spawning more workers than jobs

        thread::scope(|scope| {
            for _ in 0..workers {
                scope.spawn(|| loop {
                    // Pull the next FIFO job (fairness), or exit once the queue is closed + drained.
                    let item = {
                        let mut q = queue.lock().expect("queue mutex poisoned");
                        loop {
                            if let Some(item) = q.ready.pop_front() {
                                // Made room → wake a feeder blocked on backpressure.
                                not_full.notify_one();
                                break Some(item);
                            }
                            if q.closed {
                                break None;
                            }
                            q = not_empty.wait(q).expect("queue mutex poisoned");
                        }
                    };
                    match item {
                        Some((idx, job)) => {
                            let out = job();
                            let mut r = results.lock().expect("results mutex poisoned");
                            r[idx] = Some(out);
                        }
                        None => break,
                    }
                });
            }

            // Feeder (this thread): enqueue every job in order, blocking while the queue is full —
            // demand-signalled backpressure (the bound is enforced under the lock; G2).
            for (idx, job) in jobs.into_iter().enumerate() {
                let mut q = queue.lock().expect("queue mutex poisoned");
                while q.ready.len() >= capacity {
                    q = not_full.wait(q).expect("queue mutex poisoned");
                }
                q.ready.push_back((idx, job));
                if q.ready.len() > q.peak {
                    q.peak = q.ready.len();
                }
                not_empty.notify_one();
            }
            // Close: no more jobs. Wake every idle worker so it can drain + exit (never a hang).
            {
                let mut q = queue.lock().expect("queue mutex poisoned");
                q.closed = true;
                not_empty.notify_all();
            }
        });

        if let Some(slot) = peak_depth {
            *slot = queue.lock().expect("queue mutex poisoned").peak;
        }

        // Every slot is `Some` (liveness: each job ran exactly once); unwrap in spawn order.
        results
            .into_inner()
            .expect("results mutex poisoned")
            .into_iter()
            .map(|o| o.expect("liveness: every submitted job ran exactly once (RT2 join)"))
            .collect()
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn zero_workers_is_refused() {
        // Mutant witness: dropping the check would return Ok(_), so unwrap_err would panic.
        assert_eq!(
            Scheduler::with_workers(0, 4).unwrap_err(),
            SchedulerError::ZeroWorkers,
            "zero workers must fail closed (G2)"
        );
    }

    #[test]
    fn zero_capacity_is_refused() {
        assert_eq!(
            Scheduler::with_workers(4, 0).unwrap_err(),
            SchedulerError::ZeroCapacity,
            "zero capacity must fail closed (G2)"
        );
    }

    #[test]
    fn new_has_at_least_one_worker() {
        // Even on a single-core probe, available_parallelism fallback is 1 (never 0 — no silent hang).
        let s = Scheduler::new();
        assert!(s.workers() >= 1, "scheduler must have ≥ 1 worker");
        assert!(s.capacity() >= 1, "scheduler must have ≥ 1 capacity");
    }

    #[test]
    fn empty_job_set_returns_empty() {
        let s = Scheduler::with_workers(4, 8).unwrap();
        let out: Vec<i64> = s.run_indexed(Vec::<fn() -> i64>::new(), None);
        assert!(out.is_empty(), "no jobs ⇒ empty result (no hang)");
    }

    #[test]
    fn results_are_in_spawn_order() {
        // Mutant witness: if results were collected in completion order rather than by spawn index,
        // this deterministic-order assertion would fail under real parallelism.
        let s = Scheduler::with_workers(4, 8).unwrap();
        let jobs: Vec<_> = (0..32usize).map(|i| move || i * 10).collect();
        let out = s.run_indexed(jobs, None);
        let expected: Vec<usize> = (0..32).map(|i| i * 10).collect();
        assert_eq!(
            out, expected,
            "output must be in spawn order (RT2-comparable)"
        );
    }

    proptest! {
        // RT2 sequentialization differential: the parallel run equals the spawn-order sequential
        // reference (RT1 ⇒ schedule-independence). Tagged Empirical (this is the checked basis).
        #![proptest_config(ProptestConfig::with_cases(32))]
        #[test]
        fn parallel_run_equals_sequential_reference(
            values in proptest::collection::vec(any::<i32>(), 0..64usize),
            workers in 1usize..8,
        ) {
            let s = Scheduler::with_workers(workers, workers * 2).unwrap();
            // Pure task: a deterministic function of the (captured) value — no shared state (RT1).
            let seq_ref: Vec<i64> = values.iter().map(|&v| i64::from(v).wrapping_mul(3)).collect();
            let jobs: Vec<_> = values
                .iter()
                .map(|&v| move || i64::from(v).wrapping_mul(3))
                .collect();
            let parallel = s.run_indexed(jobs, None);
            prop_assert_eq!(parallel, seq_ref, "parallel run must equal sequential reference (RT2)");
        }

        // Liveness: every submitted job runs exactly once (no job dropped, none run twice).
        #[test]
        fn every_job_runs_exactly_once(
            n in 1usize..128,
            workers in 1usize..8,
        ) {
            let s = Scheduler::with_workers(workers, workers * 2).unwrap();
            // Each job returns its own index; the multiset of outputs must be exactly 0..n.
            let jobs: Vec<_> = (0..n).map(|i| move || i).collect();
            let mut out = s.run_indexed(jobs, None);
            out.sort_unstable();
            let expected: Vec<usize> = (0..n).collect();
            prop_assert_eq!(out, expected, "each job runs exactly once (liveness)");
        }

        // Backpressure bound (Exact, by construction): the ready queue never exceeds `capacity`.
        #[test]
        fn ready_queue_never_exceeds_capacity(
            n in 1usize..200,
            workers in 1usize..6,
            cap in 1usize..8,
        ) {
            let s = Scheduler::with_workers(workers, cap).unwrap();
            let jobs: Vec<_> = (0..n).map(|i| move || i).collect();
            let mut peak = 0usize;
            let _ = s.run_indexed(jobs, Some(&mut peak));
            prop_assert!(
                peak <= cap,
                "ready-queue peak {} must not exceed capacity {} (G2: bounded, never unbounded)",
                peak, cap
            );
        }
    }
}
