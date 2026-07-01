//! Tests for `crate::scheduler` — M-709 (single-queue baseline) / M-861 (per-worker deques +
//! steal-on-empty) / RFC-0008 RT1·RT2·RT3.
//!
//! M-797 in-crate test layout: all tests live here, not in `scheduler.rs`.
//!
//! # DoD coverage (M-861)
//!
//! 1. **Construction refusals** stay fail-closed (`ZeroWorkers`/`ZeroCapacity`), unchanged by the
//!    deque redesign.
//! 2. **Results stay in spawn order** regardless of steal activity (RT2-comparable output).
//! 3. **RT2 sequentialization differential extended under stealing:** many randomized
//!    `(values, workers)` configurations — deliberately biased toward `workers` small relative to
//!    job count, to force steal activity — assert `parallel == sequential reference`.
//! 4. **Liveness** (every job runs exactly once) holds under random worker/job-count
//!    configurations, including single-worker (no stealing possible) and many-worker (steal-heavy)
//!    extremes.
//! 5. **Backpressure bound stays `Exact`:** the peak *total* pending depth (summed across every
//!    per-worker deque) never exceeds `capacity`, across random `(n, workers, cap)` configurations.
//! 6. **`StealPolicy::select_victim` (RT3 EXPLAIN) is total, deterministic, and inspectable:**
//!    same inputs → same `StealDecision`; returns `None` iff every other deque is empty; the
//!    returned `victim` is never the thief itself and always has nonzero occupancy in the snapshot.
//! 7. **Steal activity is actually exercised — a real mutant-witness, not just an isolated-policy
//!    check.** `run_indexed`'s `steal_count` out-param counts jobs completed via a cross-deque
//!    steal; under a steal-forcing shape (few workers, many jobs) it must be `> 0`. A scheduler
//!    that silently regressed to single-queue/no-steal dispatch would still pass checks 1–5 (the
//!    *outputs* would still be correct) but this test would catch the regression directly.

use proptest::prelude::*;

use crate::scheduler::{Scheduler, SchedulerError, StealPolicy};

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
fn new_uses_default_steal_policy() {
    let s = Scheduler::new();
    assert_eq!(
        s.steal_policy(),
        StealPolicy::RoundRobin,
        "Scheduler::new must use the documented default steal policy"
    );
}

#[test]
fn empty_job_set_returns_empty() {
    let s = Scheduler::with_workers(4, 8).unwrap();
    let out: Vec<i64> = s.run_indexed(Vec::<fn() -> i64>::new(), None, None);
    assert!(out.is_empty(), "no jobs ⇒ empty result (no hang)");
}

#[test]
fn results_are_in_spawn_order() {
    // Mutant witness: if results were collected in completion order rather than by spawn index,
    // this deterministic-order assertion would fail under real parallelism — including under
    // steal-heavy configurations (few workers, many jobs, so most jobs are stolen).
    let s = Scheduler::with_workers(4, 8).unwrap();
    let jobs: Vec<_> = (0..32usize).map(|i| move || i * 10).collect();
    let out = s.run_indexed(jobs, None, None);
    let expected: Vec<usize> = (0..32).map(|i| i * 10).collect();
    assert_eq!(
        out, expected,
        "output must be in spawn order (RT2-comparable)"
    );
}

#[test]
fn results_are_in_spawn_order_steal_heavy() {
    // Deliberately steal-heavy: 2 workers, 200 jobs, so nearly every worker will empty its own
    // deque and steal repeatedly. Output must still be spawn-order.
    let s = Scheduler::with_workers(2, 4).unwrap();
    let jobs: Vec<_> = (0..200usize).map(|i| move || i).collect();
    let out = s.run_indexed(jobs, None, None);
    let expected: Vec<usize> = (0..200).collect();
    assert_eq!(
        out, expected,
        "spawn-order output must hold even under heavy steal activity"
    );
}

#[test]
fn stealing_actually_occurs_under_a_lopsided_workload() {
    // Mutant witness for "stealing never happens": worker 0's round-robin share (even spawn
    // indices) is all slow (parks briefly), worker 1's share (odd indices) is all instant. Worker 1
    // will drain its own deque long before worker 0 finishes even its first slow job, so worker 1
    // MUST steal from worker 0's backlog to make progress — a scheduler that silently regressed to
    // single-queue/no-steal dispatch (or one where steal-selection was a no-op) would report
    // `steal_count == 0` here, and this assertion would catch it.
    let s = Scheduler::with_workers(2, 64).unwrap();
    let jobs: Vec<Box<dyn FnOnce() -> usize + Send>> = (0..64usize)
        .map(|i| -> Box<dyn FnOnce() -> usize + Send> {
            if i % 2 == 0 {
                Box::new(move || {
                    std::thread::sleep(std::time::Duration::from_millis(2));
                    i
                })
            } else {
                Box::new(move || i)
            }
        })
        .collect();
    let mut steals = 0usize;
    let out = s.run_indexed(jobs, None, Some(&mut steals));
    let expected: Vec<usize> = (0..64).collect();
    assert_eq!(
        out, expected,
        "spawn-order output must hold under a lopsided workload too"
    );
    assert!(
        steals > 0,
        "worker 1 (all-instant jobs) must have stolen at least one job from worker 0's \
         (all-slow) backlog — steal_count was 0, indicating stealing did not occur"
    );
}

// ── RT3: StealPolicy::select_victim is total, deterministic, inspectable ──────────────────────

#[test]
fn select_victim_none_when_all_empty() {
    let occupancy = vec![0usize; 4];
    let decision = StealPolicy::RoundRobin.select_victim(4, 0, &occupancy);
    assert_eq!(
        decision, None,
        "no candidate has work ⇒ select_victim must return None, never a spurious pick"
    );
}

#[test]
fn select_victim_never_targets_the_thief() {
    // thief = 1; only worker 3 (a neighbor, not the thief) has work.
    let occupancy = vec![0usize, 0, 0, 5];
    let decision = StealPolicy::RoundRobin
        .select_victim(4, 1, &occupancy)
        .expect("worker 3 has work to steal");
    assert_ne!(
        decision.victim, decision.thief,
        "a thief must never steal from itself"
    );
    assert_eq!(
        decision.victim, 3,
        "the only nonempty neighbor must be selected"
    );
}

#[test]
fn select_victim_ignores_thiefs_own_occupancy() {
    // Even if occupancy[thief] were nonzero (a caller violating the documented precondition —
    // "the caller only asks once its own deque is empty"), the scan starts at offset 1, so the
    // thief itself can never be the returned victim — here every OTHER worker is empty, so the
    // result must be None despite occupancy[thief] being nonzero.
    let occupancy = vec![0usize, 5, 0, 0];
    let decision = StealPolicy::RoundRobin.select_victim(4, 1, &occupancy);
    assert_eq!(
        decision, None,
        "select_victim must never report the thief's own occupancy as a steal target"
    );
}

#[test]
fn select_victim_picks_first_nonempty_in_rotation() {
    // thief=0, workers 1..3 scanned in order; worker 2 is the first nonempty.
    let occupancy = vec![0usize, 0, 3, 7];
    let decision = StealPolicy::RoundRobin
        .select_victim(4, 0, &occupancy)
        .expect("worker 2 has work");
    assert_eq!(
        decision.victim, 2,
        "round-robin must pick the first nonempty candidate"
    );
    assert_eq!(
        decision.victim_depth, 3,
        "the decision must record the victim's occupancy"
    );
    assert_eq!(
        decision.candidates_scanned, 2,
        "candidates_scanned counts worker 1 (empty) then worker 2 (chosen)"
    );
}

#[test]
fn select_victim_is_deterministic() {
    // Mutant witness: if selection consulted any hidden/random state, two calls with identical
    // inputs could disagree.
    let occupancy = vec![2usize, 0, 4, 0, 1];
    let d1 = StealPolicy::RoundRobin.select_victim(5, 1, &occupancy);
    let d2 = StealPolicy::RoundRobin.select_victim(5, 1, &occupancy);
    assert_eq!(
        d1, d2,
        "select_victim must be a pure, deterministic function of its inputs"
    );
}

proptest! {
    // RT2 sequentialization differential, EXTENDED UNDER STEALING (M-861): the parallel run
    // (per-worker deques + steal-on-empty) equals the spawn-order sequential reference, across
    // randomized worker counts including steal-forcing shapes (few workers, many jobs). Tagged
    // Empirical (this is the checked basis).
    #![proptest_config(ProptestConfig::with_cases(32))]
    #[test]
    fn parallel_run_equals_sequential_reference_under_stealing(
        values in proptest::collection::vec(any::<i32>(), 0..128usize),
        workers in 1usize..8,
    ) {
        let s = Scheduler::with_workers(workers, workers * 2).unwrap();
        // Pure task: a deterministic function of the (captured) value — no shared state (RT1).
        let seq_ref: Vec<i64> = values.iter().map(|&v| i64::from(v).wrapping_mul(3)).collect();
        let jobs: Vec<_> = values
            .iter()
            .map(|&v| move || i64::from(v).wrapping_mul(3))
            .collect();
        let parallel = s.run_indexed(jobs, None, None);
        prop_assert_eq!(
            parallel, seq_ref,
            "parallel run (with per-worker deques + steal-on-empty) must equal the sequential \
             reference (RT2) — stealing reorders execution, never the observable result"
        );
    }

    // Same differential, but deliberately steal-heavy: worker count is held small (1..4) while job
    // count ranges wider, so most workers exhaust their own round-robin share and must steal.
    #[test]
    fn parallel_run_equals_sequential_reference_steal_heavy(
        values in proptest::collection::vec(any::<i32>(), 0..256usize),
        workers in 1usize..4,
    ) {
        let s = Scheduler::with_workers(workers, workers * 2).unwrap();
        let seq_ref: Vec<i64> = values.iter().map(|&v| i64::from(v).wrapping_mul(7).wrapping_add(1)).collect();
        let jobs: Vec<_> = values
            .iter()
            .map(|&v| move || i64::from(v).wrapping_mul(7).wrapping_add(1))
            .collect();
        let parallel = s.run_indexed(jobs, None, None);
        prop_assert_eq!(
            parallel, seq_ref,
            "steal-heavy configuration (few workers, many jobs) must still equal the sequential \
             reference (RT2)"
        );
    }

    // Liveness: every submitted job runs exactly once (no job dropped, none run twice), under
    // random worker/job-count configurations spanning no-steal (workers >= n) through
    // steal-heavy (workers << n).
    #[test]
    fn every_job_runs_exactly_once(
        n in 1usize..200,
        workers in 1usize..8,
    ) {
        let s = Scheduler::with_workers(workers, workers * 2).unwrap();
        // Each job returns its own index; the multiset of outputs must be exactly 0..n.
        let jobs: Vec<_> = (0..n).map(|i| move || i).collect();
        let mut out = s.run_indexed(jobs, None, None);
        out.sort_unstable();
        let expected: Vec<usize> = (0..n).collect();
        prop_assert_eq!(out, expected, "each job runs exactly once (liveness), regardless of steal activity");
    }

    // Backpressure bound (Exact, by construction): the TOTAL pending depth across every
    // per-worker deque never exceeds `capacity`.
    #[test]
    fn ready_queue_never_exceeds_capacity(
        n in 1usize..200,
        workers in 1usize..6,
        cap in 1usize..8,
    ) {
        let s = Scheduler::with_workers(workers, cap).unwrap();
        let jobs: Vec<_> = (0..n).map(|i| move || i).collect();
        let mut peak = 0usize;
        let _ = s.run_indexed(jobs, Some(&mut peak), None);
        prop_assert!(
            peak <= cap,
            "total ready-queue peak {} (summed across every per-worker deque) must not exceed \
             capacity {} (G2: bounded, never unbounded)",
            peak, cap
        );
    }

    // RT3: select_victim never returns the thief as its own victim, and (when it returns Some) the
    // reported victim_depth matches the snapshot passed in — the EXPLAIN record is faithful, not
    // approximate.
    #[test]
    fn select_victim_decision_is_faithful_to_snapshot(
        occupancy in proptest::collection::vec(0usize..10, 2..12),
        thief_seed in any::<usize>(),
    ) {
        let workers = occupancy.len();
        let thief = thief_seed % workers;
        let mut occ = occupancy.clone();
        occ[thief] = 0; // precondition: the thief consults the policy only once its own deque is empty
        let decision = StealPolicy::RoundRobin.select_victim(workers, thief, &occ);
        if let Some(d) = decision {
            prop_assert_ne!(d.victim, thief, "victim must never be the thief");
            prop_assert_eq!(d.victim_depth, occ[d.victim], "EXPLAIN record must match the snapshot");
            prop_assert!(d.victim_depth > 0, "a chosen victim must have nonzero occupancy");
            prop_assert!(d.candidates_scanned >= 1 && d.candidates_scanned < workers);
        } else {
            // None is only valid if every non-thief worker was empty in the snapshot.
            let any_other_nonempty = occ.iter().enumerate().any(|(i, &d)| i != thief && d > 0);
            prop_assert!(!any_other_nonempty, "select_victim returned None despite a nonempty candidate existing");
        }
    }
}
