//! `Scheduler` — a real **OS-thread** work-stealing scheduler (M-709 / M-861 / M-864 / RFC-0008
//! RT1·RT2·RT3 / E12-1 / E25-1).
//!
//! The v0 R1 surface ([`crate::colony`]) ran tasks cooperatively on the calling thread. M-709 grew
//! that into a fixed pool of OS worker threads over a single shared FIFO queue. M-861 grew it
//! again: **per-worker deques with steal-on-empty** (LIFO-own / FIFO-steal), which cuts contention
//! on the single shared queue while preserving every guarantee M-709 established. **M-864** replaces
//! the per-call `std::thread::scope` (fresh OS threads every call) with dispatch onto the
//! process-wide **persistent** [`crate::pool`] — see that module's docs for the help-stealing design
//! that makes *nested* `run_indexed` submission safe on a **fixed**-size pool.
//!
//! # Honesty (VR-5)
//!
//! - **RT2 sequentialization differential — `Empirical`, unchanged by stealing or by M-864's pool
//!   redesign.** RT1 (tasks share no mutable state) makes the observable result order-independent —
//!   the differential ("parallel run ≡ spawn-order sequential reference") asserts *result*-equality,
//!   never *scheduling*-equality, so work-stealing (which only reorders *execution*, never the
//!   RT1/RT2 observable) leaves the differential's claim unchanged, and neither does *how many OS
//!   threads* execute that reordered work (M-864: persistent pool vs. per-call fresh threads is an
//!   execution-substrate change, not an observable one). It is *checked* by a property test
//!   ([`tests`]) run under many randomized worker/steal configurations, not assumed — but it is not
//!   `Proven` (no mechanized theorem), so it stays `Empirical`.
//! - **RT3 — stealing is kept semantics-free.** The victim-selection policy
//!   ([`StealPolicy`]/[`StealDecision`]) is a small, total, deterministic, inspectable decision
//!   procedure — the same posture RFC-0008 §4.1 RT3 requires of a placement policy (mirroring the
//!   reserved `forage` construct's EXPLAIN posture, without depending on the unbuilt `forage`
//!   mechanism or on `mycelium-select`'s heavier RFC-0005 machinery, which is out of scope for this
//!   crate). Completion order and worker identity are never surfaced through the public API —
//!   [`Scheduler::run_indexed`] still returns outputs **in spawn order** — so RT2's deterministic
//!   default is preserved regardless of which worker executed which job or in what order steals
//!   occurred.
//! - **Backpressure bound — `Exact`.** The **total** pending-job count across every per-worker
//!   deque (this batch's own lanes — M-864 keeps this bound per-call, unaffected by the pool being
//!   shared) holds **at most** `capacity` pending jobs *by construction*: a job is enqueued only
//!   while `total < capacity`, under one shared lock guarding all deques together (so "total ≤
//!   capacity" is a single structural invariant, not a race between N independently-locked
//!   counters). The bound is asserted by a mutant-witness test.
//! - **Liveness (every submitted job runs exactly once) — `Empirical`.** Property-tested over
//!   random job sets and random worker/steal configurations; not `Proven`.
//! - **Nested-submission deadlock-freedom (M-864) — `Empirical`.** [`crate::pool::Pool::help_while`]
//!   carries a structural argument (module docs) that a **fixed**-size pool never deadlocks under
//!   arbitrarily deep nested `run_indexed` submission; checked by deep-chain and wide-fan-out stress
//!   tests under a wall-clock timeout ([`tests`]), not mechanically proven — `Empirical`, not
//!   `Proven` (VR-5).
//!
//! # The `'static` contract (M-864 — ratified: `docs/notes/DN-67-Persistent-Work-Stealing-Pool.md`)
//!
//! [`Scheduler::run_indexed`] now requires `F: 'static` and `T: 'static` (previously only `F: Send`
//! and `T: Send`, borrowing freely within the `std::thread::scope` call). A **persistent** pool's
//! worker threads outlive any single `run_indexed` call, so a job closure can no longer safely
//! borrow data from the calling stack frame the way `thread::scope`'s scoped threads could — it must
//! own (or `Arc`-share) everything it touches. Every current caller already passes owned data or was
//! adjusted to (M-860's per-node `Node::clone`, M-862's `Arc`-shared fuel counter and cloned
//! `Interpreter`); see DN-67 for the full rationale and the caller-by-caller audit.
//!
//! The crate stays `#![forbid(unsafe_code)]`: [`crate::pool`] is built from `Arc`/`Mutex`/`Condvar`/
//! `VecDeque`/`thread::spawn` — no `unsafe`, no `rayon`/`crossbeam` (a Chase-Lev lock-free deque
//! needs `unsafe` or an external crate; both are out of scope here, ADR/DN ratified: zero new
//! dependencies).

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

use mycelium_core::GuaranteeStrength;

use crate::pool;

/// Guarantee strength for the scheduler's RT2 sequentialization differential.
///
/// `Empirical`: the parallel run equals the sequential reference by RT1 (no shared mutable state),
/// checked by a property test — not `Proven` (no mechanized theorem). Unchanged by work-stealing
/// (M-861): stealing reorders *execution*, never the RT1/RT2 *observable*. (RFC-0008 RT2.)
pub const SCHEDULER_RT2_STRENGTH: GuaranteeStrength = GuaranteeStrength::Empirical;

/// Guarantee strength for the demand-signalled backpressure bound.
///
/// `Exact`: the **total** pending-job count across every per-worker deque never exceeds `capacity`
/// *by construction* (enqueue only while `total < capacity`, under the one lock guarding all
/// deques). (RFC-0008 §4.3.)
pub const SCHEDULER_BACKPRESSURE_STRENGTH: GuaranteeStrength = GuaranteeStrength::Exact;

/// Guarantee strength for liveness (every submitted job runs exactly once).
///
/// `Empirical`: property-tested over random job sets and random worker/steal configurations; not
/// `Proven`.
pub const SCHEDULER_LIVENESS_STRENGTH: GuaranteeStrength = GuaranteeStrength::Empirical;

/// Guarantee strength for the steal-victim-selection policy's determinism/inspectability (RT3).
///
/// `Exact`: [`StealPolicy::select_victim`] is a total, deterministic function of its inputs (worker
/// count, thief index, deque occupancy snapshot) — same inputs always produce the same
/// [`StealDecision`], every decision is inspectable. This is the RT3 "reified, named, explained"
/// obligation for the one piece of scheduling that is not RT1/RT2-neutral by inspection alone: a
/// caller can ask *why* a steal targeted a given worker.
pub const STEAL_POLICY_STRENGTH: GuaranteeStrength = GuaranteeStrength::Exact;

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

/// The victim-selection policy for a worker whose own deque is empty (RFC-0008 RT3).
///
/// A policy is a **total, deterministic** procedure: same `(workers, thief, occupancy)` in ⇒ same
/// [`StealDecision`] out. This keeps stealing a placement-only concern (RT3: "where a hypha runs
/// may change performance, never the observable") — never a source of surprise, and always
/// EXPLAIN-able via [`StealPolicy::select_victim`]'s returned [`StealDecision`].
///
/// v0 ships exactly one policy, [`StealPolicy::RoundRobin`]; the type is an enum (not a bare
/// function) so a future policy is additive, not a breaking change — mirroring the reserved
/// `forage` construct's posture (a content-addressed, swappable decision procedure) without
/// depending on the unbuilt `forage` mechanism or the heavier RFC-0005 `mycelium-select` machinery
/// (out of scope for this crate; FLAG for a future placement-policy unification, see module docs).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StealPolicy {
    /// Starting one slot after the thief's own index, scan the other workers' deques in a fixed
    /// deterministic rotation and steal from the first non-empty one found (FIFO — `pop_front` —
    /// from the victim's deque, so the oldest work at the victim is taken first, minimizing
    /// disruption to the victim's own LIFO-recency locality).
    #[default]
    RoundRobin,
}

impl StealPolicy {
    /// Decide which worker `thief` should steal from, given a snapshot of every worker's deque
    /// length (`occupancy[i]` = worker `i`'s pending-job count, `occupancy[thief]` == 0 by the
    /// caller's precondition — a worker only consults the policy once its own deque is empty).
    ///
    /// Returns `None` if every other worker's deque is empty too (nothing to steal). Total,
    /// deterministic, EXPLAIN-able: the returned [`StealDecision`] records exactly which worker was
    /// picked, its occupancy, and how many candidates were scanned before it — the RT3 obligation.
    ///
    /// Guarantee: **Exact** ([`STEAL_POLICY_STRENGTH`]) — a pure function of its inputs, no hidden
    /// state, no randomness.
    #[must_use]
    pub fn select_victim(
        &self,
        workers: usize,
        thief: usize,
        occupancy: &[usize],
    ) -> Option<StealDecision> {
        debug_assert_eq!(
            occupancy.len(),
            workers,
            "occupancy snapshot must cover every worker"
        );
        match self {
            StealPolicy::RoundRobin => {
                for offset in 1..workers {
                    let candidate = (thief + offset) % workers;
                    let depth = occupancy[candidate];
                    if depth > 0 {
                        return Some(StealDecision {
                            thief,
                            victim: candidate,
                            victim_depth: depth,
                            candidates_scanned: offset,
                        });
                    }
                }
                None
            }
        }
    }
}

/// The EXPLAIN record for one [`StealPolicy::select_victim`] decision (RFC-0008 RT3: "every
/// departure from RT2's fragment ... is an explicit construct whose decision procedure ... [has]
/// mandatory EXPLAIN"). Inspectable, never silent — a caller (or a test) can reconstruct exactly
/// why a given worker was chosen as the victim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StealDecision {
    /// The worker whose own deque was empty and who is looking to steal.
    pub thief: usize,
    /// The worker chosen as the steal source.
    pub victim: usize,
    /// The victim's deque depth *at the time of the decision* (a snapshot — the victim's actual
    /// deque may change before the steal executes under the lock; the steal itself re-checks).
    pub victim_depth: usize,
    /// How many candidates (including the chosen victim) were scanned before landing on `victim`.
    pub candidates_scanned: usize,
}

/// A real OS-thread scheduler: a fixed worker pool, **per-worker deques with steal-on-empty**
/// (LIFO-own / FIFO-steal), and a **bounded**, demand-signalled ready queue (RFC-0008
/// RT1·RT2·RT3·§4.3; M-709/M-861).
///
/// # Guarantee
/// - RT2 sequentialization: **`Empirical`** ([`SCHEDULER_RT2_STRENGTH`]), unchanged by stealing.
/// - RT3 steal-policy determinism/inspectability: **`Exact`** ([`STEAL_POLICY_STRENGTH`]).
/// - Backpressure bound (total pending ≤ `capacity`): **`Exact`** ([`SCHEDULER_BACKPRESSURE_STRENGTH`]).
/// - Liveness (each job runs once): **`Empirical`** ([`SCHEDULER_LIVENESS_STRENGTH`]).
#[derive(Debug, Clone, Copy)]
pub struct Scheduler {
    workers: usize,
    capacity: usize,
    steal_policy: StealPolicy,
}

impl Scheduler {
    /// A scheduler sized to the host's available parallelism (fallback: 1 worker), with a ready
    /// queue capacity of `2 × workers` (allows up to two pending jobs per worker, independent of
    /// in-flight work), using the default [`StealPolicy`].
    ///
    /// Guarantee: **Exact** (construction is deterministic given the probed parallelism).
    #[must_use]
    pub fn new() -> Self {
        let workers = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
        Scheduler {
            workers,
            capacity: workers.saturating_mul(2),
            steal_policy: StealPolicy::default(),
        }
    }

    /// A scheduler with exactly `workers` OS threads and a ready-queue `capacity`, using the
    /// default [`StealPolicy`].
    ///
    /// # Errors
    /// [`SchedulerError::ZeroWorkers`] if `workers == 0`; [`SchedulerError::ZeroCapacity`] if
    /// `capacity == 0` (both fail-closed — G2: never a silent substitution).
    pub fn with_workers(workers: usize, capacity: usize) -> Result<Self, SchedulerError> {
        Self::with_workers_and_policy(workers, capacity, StealPolicy::default())
    }

    /// A scheduler with exactly `workers` OS threads, a ready-queue `capacity`, and an explicit
    /// [`StealPolicy`] — the RT3 EXPLAIN entry point: a caller who cares *which* deterministic
    /// victim-selection procedure is in effect can name it, rather than relying on the default.
    ///
    /// # Errors
    /// [`SchedulerError::ZeroWorkers`] if `workers == 0`; [`SchedulerError::ZeroCapacity`] if
    /// `capacity == 0` (both fail-closed — G2: never a silent substitution).
    pub fn with_workers_and_policy(
        workers: usize,
        capacity: usize,
        steal_policy: StealPolicy,
    ) -> Result<Self, SchedulerError> {
        if workers == 0 {
            return Err(SchedulerError::ZeroWorkers);
        }
        if capacity == 0 {
            return Err(SchedulerError::ZeroCapacity);
        }
        Ok(Scheduler {
            workers,
            capacity,
            steal_policy,
        })
    }

    /// The number of OS worker threads this scheduler runs.
    #[must_use]
    pub fn workers(&self) -> usize {
        self.workers
    }

    /// The bounded ready-queue depth — the backpressure ceiling (never exceeded across the *sum*
    /// of every per-worker deque; G2).
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// The steal-victim-selection policy this scheduler uses (RT3 EXPLAIN entry point).
    #[must_use]
    pub fn steal_policy(&self) -> StealPolicy {
        self.steal_policy
    }

    /// Run `jobs` across the persistent, process-wide work-stealing pool ([`crate::pool`], M-864)
    /// and return their outputs **in spawn order** (so the result vector is directly comparable to
    /// the sequential reference — the RT2 differential).
    ///
    /// Dispatch: this batch's own feeder distributes jobs round-robin across `workers` **lanes**
    /// (per-batch deques — unchanged from M-861). Each lane's loop pops its **own** deque LIFO
    /// (`pop_back` — recency locality) and, once empty, consults [`StealPolicy::select_victim`] to
    /// steal FIFO (`pop_front`) from another lane's deque. Completion order and which physical thread
    /// ran which job are **never observable** through this API (RT3-neutral: only the returned,
    /// spawn-order-indexed result vector is visible) — so RT2's deterministic-result default holds
    /// regardless of the steal schedule, and regardless of M-864's dispatch substrate. Liveness
    /// (every job runs exactly once) is `Empirical`.
    ///
    /// **M-864 — persistent pool, help-stealing join.** Each of the `workers` lane-loops is
    /// submitted as one task to the shared [`crate::pool`] rather than spawned as a fresh OS thread;
    /// the calling thread runs the feeder loop below and then **helps** the pool drain (any pending
    /// task, from this batch or — under nested submission — any other) until every lane of this
    /// batch has finished ([`crate::pool::Pool::help_while`]) — never a bare block. This is what
    /// makes a **nested** `run_indexed` call (one submitted from inside a job running on a pool
    /// worker) safe on a **fixed**-size pool: see `pool`'s module docs for the deadlock-freedom
    /// argument.
    ///
    /// The **total** pending-job count across every lane's deque (this call's own, independent of
    /// any other concurrently in-flight batch) is bounded at [`capacity`](Scheduler::capacity): the
    /// feeder enqueues a job only while the total has room, blocking (while still available to be
    /// helped by other threads) otherwise — demand-signalled backpressure, never an unbounded silent
    /// buffer (G2 / RFC-0008 §4.3).
    ///
    /// `peak_depth` (when `Some`) records the **maximum** observed *total* pending depth (summed
    /// across every lane), so a test can confirm the bound is real (it must be `≤ capacity`).
    ///
    /// `steal_count` (when `Some`) records how many jobs were actually completed via a steal
    /// (`pop_front` from another lane's deque) rather than from the popping lane's own deque — a
    /// mutant-witness: a scheduler that silently regressed to single-queue dispatch (never
    /// stealing) would report `0` here under a steal-forcing job shape, and a test asserts it is
    /// nonzero (see `tests::scheduler`).
    ///
    /// # The `'static` contract (M-864)
    /// `F`/`T` must be `'static`: the shared pool's worker threads outlive this call, so a job can no
    /// longer borrow from the caller's stack frame the way the pre-M-864 `std::thread::scope`
    /// allowed. See the module docs and `docs/notes/DN-67-Persistent-Work-Stealing-Pool.md`.
    ///
    /// Guarantee: outputs equal the sequential reference — **`Empirical`** (RT2; RT1 ⇒
    /// schedule-independence, unaffected by stealing or by the M-864 pool redesign). Pure tasks only
    /// (the [`crate::task`] purity contract is `Declared`).
    #[must_use]
    pub fn run_indexed<T, F>(
        &self,
        jobs: Vec<F>,
        peak_depth: Option<&mut usize>,
        steal_count: Option<&mut usize>,
    ) -> Vec<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let n = jobs.len();
        if n == 0 {
            return Vec::new();
        }

        let workers = self.workers.min(n); // no point creating more lanes than jobs
        let pool = pool::get();

        // One lock guards every lane's deque together, so "total pending ≤ capacity" is a single
        // structural invariant (never a race between N independently-locked counters). `deques[i]`
        // is lane `i`'s own ready deque: LIFO pop from the back (own work), FIFO pop from the front
        // when another lane steals from it. `closed` signals the feeder is done so an idle, empty
        // lane can exit instead of waiting forever (never a silent hang).
        struct Deques<F> {
            deques: Vec<VecDeque<(usize, F)>>,
            total: usize,
            closed: bool,
            peak: usize,
            steals: usize,
        }
        let state = Arc::new(Mutex::new(Deques::<F> {
            deques: (0..workers).map(|_| VecDeque::new()).collect(),
            total: 0,
            closed: false,
            peak: 0,
            steals: 0,
        }));
        let not_empty = Arc::new(Condvar::new()); // a lane waits here for work anywhere (or close)
        let not_full = Arc::new(Condvar::new()); // the feeder waits here for total room (backpressure)

        // Results, pre-sized and written by spawn index → the output stays in spawn order (RT2).
        let results = Arc::new(Mutex::new((0..n).map(|_| None::<T>).collect::<Vec<_>>()));

        // The per-batch "how many lanes are still running" countdown — the nested-join wait
        // condition `help_while` polls. Reaching 0 means every lane has drained its own deque, found
        // nothing left to steal, and observed `closed` — i.e. this whole batch is done.
        let remaining = Arc::new((Mutex::new(workers), Condvar::new()));

        let capacity = self.capacity;
        let policy = self.steal_policy;

        for me in 0..workers {
            let state = Arc::clone(&state);
            let results = Arc::clone(&results);
            let not_full = Arc::clone(&not_full);
            let not_empty = Arc::clone(&not_empty);
            let remaining = Arc::clone(&remaining);
            let pool_for_signal = Arc::clone(&pool);
            pool.submit(Box::new(move || {
                loop {
                    // Pull the next job — own deque first (LIFO), then steal (FIFO) — or exit once
                    // the queue is closed + every deque is drained.
                    let item = {
                        let mut s = state.lock().expect("mycelium-sched: scheduler mutex poisoned");
                        loop {
                            if let Some(item) = s.deques[me].pop_back() {
                                s.total -= 1;
                                not_full.notify_one(); // made room → wake a blocked feeder
                                break Some(item);
                            }
                            // The occupancy snapshot and the steal below happen under the SAME held
                            // lock, so no concurrent thief can race the victim's deque empty between
                            // the two — `victim_depth > 0` in the snapshot guarantees `pop_front`
                            // below succeeds.
                            let occupancy: Vec<usize> =
                                s.deques.iter().map(VecDeque::len).collect();
                            if let Some(decision) = policy.select_victim(workers, me, &occupancy) {
                                let item = s.deques[decision.victim]
                                    .pop_front()
                                    .expect("victim_depth > 0 under the same held lock ⇒ pop_front succeeds");
                                s.total -= 1;
                                s.steals += 1;
                                not_full.notify_one();
                                break Some(item);
                            }
                            if s.closed {
                                break None;
                            }
                            s = not_empty.wait(s).expect("mycelium-sched: scheduler mutex poisoned");
                        }
                    };
                    match item {
                        Some((idx, job)) => {
                            let out = job();
                            let mut r = results.lock().expect("mycelium-sched: results mutex poisoned");
                            r[idx] = Some(out);
                        }
                        None => break,
                    }
                }
                // This lane is done. Decrement the batch's countdown; the last lane out wakes
                // anyone waiting on `remaining`'s own condvar, and (via `notify_all`) any helper
                // parked in `Pool::help_while` with nothing to steal right now.
                let (lock, cv) = &*remaining;
                let mut r = lock.lock().expect("mycelium-sched: remaining-lanes mutex poisoned");
                *r -= 1;
                if *r == 0 {
                    cv.notify_all();
                }
                drop(r);
                pool_for_signal.notify_all();
            }));
        }

        // Feeder (this thread — may be an external caller, or a pool worker running a job that
        // called `run_indexed` again, i.e. a nested submission): enqueue every job in order,
        // round-robin across the lanes, blocking while the *total* is at capacity —
        // demand-signalled backpressure (the bound is enforced under the lock; G2). This thread is
        // never "just" blocked: `not_full.wait` only parks while genuinely no room exists, and every
        // lane consuming this batch's own work is itself a pool task, reachable via `help_while`
        // below once feeding is done.
        for (idx, job) in jobs.into_iter().enumerate() {
            let mut s = state
                .lock()
                .expect("mycelium-sched: scheduler mutex poisoned");
            while s.total >= capacity {
                s = not_full
                    .wait(s)
                    .expect("mycelium-sched: scheduler mutex poisoned");
            }
            let target = idx % workers;
            s.deques[target].push_back((idx, job));
            s.total += 1;
            if s.total > s.peak {
                s.peak = s.total;
            }
            not_empty.notify_all(); // any idle lane may steal this new item — wake all
        }
        // Close: no more jobs. Wake every idle lane so it can drain + exit (never a hang).
        {
            let mut s = state
                .lock()
                .expect("mycelium-sched: scheduler mutex poisoned");
            s.closed = true;
            not_empty.notify_all();
        }

        // The nested-join wait: help the shared pool drain — this batch's own lanes, or (under
        // nested submission) anyone else's — until every lane of THIS batch has exited. Never a bare
        // block (M-864's help-steal deadlock-freedom argument; see `pool` module docs).
        {
            let (lock, _cv) = &*remaining;
            pool.help_while(|| {
                *lock
                    .lock()
                    .expect("mycelium-sched: remaining-lanes mutex poisoned")
                    == 0
            });
        }

        if peak_depth.is_some() || steal_count.is_some() {
            let s = state
                .lock()
                .expect("mycelium-sched: scheduler mutex poisoned");
            if let Some(slot) = peak_depth {
                *slot = s.peak;
            }
            if let Some(slot) = steal_count {
                *slot = s.steals;
            }
        }

        // Every lane has exited (`remaining == 0`), and each lane's own writes into `results`
        // happen-before its own countdown decrement (program order on that lane's thread), which
        // happens-before this thread observing `remaining == 0` (via `remaining`'s mutex) — so every
        // write is visible here. Take the contents out from under the lock (never `Arc::try_unwrap`:
        // other lane closures' `Arc<Mutex<..>>` clones are dropped on their own threads shortly
        // after their final decrement, on a schedule this thread does not control — racing that
        // drop would be a spurious, non-deterministic failure mode, not a real one).
        let contents = std::mem::take(
            &mut *results
                .lock()
                .expect("mycelium-sched: results mutex poisoned"),
        );
        // Every slot is `Some` (liveness: each job ran exactly once); unwrap in spawn order.
        contents
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
