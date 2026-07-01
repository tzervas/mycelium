# Design Note DN-67 — Persistent Work-Stealing Pool (M-864)

| Field | Value |
|---|---|
| **Note** | DN-67 |
| **Status** | **Draft** (2026-07-01, authored by the M-864 leaf) |
| **Decides** | Ratifies the `Scheduler::run_indexed` closure/return-type contract change M-864 requires — `F: FnOnce() -> T + Send + 'static` and `T: Send + 'static` (previously `F: Send`, `T: Send`, with borrowing freely permitted via `std::thread::scope`) — and records the help-stealing design that makes nested `run_indexed` submission safe and cheap on a **persistent, bounded** pool. |
| **Feeds** | `mycelium-sched::scheduler::Scheduler::run_indexed`; its consumers `mycelium-mlir::llvm::emit_llvm_ir_many_with_swap_mode` (M-860), `mycelium-interp::parallel::eval_top_batch` (M-862), `mycelium-std-runtime::dataflow::run_dataflow_scheduled` (M-711), `mycelium-std-runtime::supervision::run_supervised` (M-713). |
| **Depends on** | DN-61 §A.2 (R1 scheduler normativity — internal scheduling strategy is explicitly **not** normative; only RT2 determinism + fuel-compatible cooperative stepping + RT7 scope discipline are); RFC-0008 §4.1 RT1–RT3, §4.7; M-861 (per-worker deques + steal-on-empty, the dispatch/steal logic this note's pool reuses verbatim). |
| **Date** | 2026-07-01 |
| **Task** | M-864, branch `claude/dev-m864-workstealing-pool` |

> **Posture (transparency rule / VR-5).** Every claim below is either `Exact` (a structural
> invariant enforced by the type system or a lock), `Empirical` (checked by the property/stress
> tests cited), or explicitly flagged as unproven. Nothing here is upgraded past its checked basis.

---

## 1. The problem this note ratifies a fix for

Before M-864, `Scheduler::run_indexed` used `std::thread::scope`, spawning **fresh OS threads on
every call**. This is deadlock-safe under nesting (a nested `run_indexed` call just opens its own
fresh scope) but costs `O(depth × fan-out)` threads for recursive submission. M-860 (parallel AOT
codegen) and M-862 (parallel interpreter evaluation) both had to bound their own parallelism to a
single, non-nested, top-level batch specifically to avoid this blowup — a documented interim
measure in both modules' own docs, naming this issue as the follow-up that would remove the bound.

M-864 replaces the per-call `thread::scope` with one process-wide, persistent, bounded pool
(`mycelium-sched::pool`), sized once to `available_parallelism()` and reused for the life of the
process — including across arbitrarily deep nested `run_indexed` calls, with **no new OS thread
ever spawned by a nested call**.

## 2. The `'static` contract change (what this note ratifies)

A **persistent** pool's worker threads outlive any single `run_indexed` call — unlike
`thread::scope`, which structurally guarantees every spawned thread is joined before the scope
function returns, and can therefore soundly permit non-`'static` borrows. Once the pool's threads
are long-lived and shared across arbitrarily many, arbitrarily nested calls, a job closure can no
longer safely borrow data from the calling stack frame: the type system cannot express "trust me,
`run_indexed` is still synchronous and drains everything before returning" for a *shared* pool the
way it can for a *per-call* scope, short of `unsafe` (a lifetime-erasing transmute) — which this
crate forbids (`#![forbid(unsafe_code)]`, ADR-014) and which the M-864 issue explicitly said to
avoid ("if you find you cannot build a correct nested-safe pool in pure safe std ... STOP and
FLAG").

The safe alternative — the one this note ratifies — is to require ownership: `run_indexed`'s
signature tightens to

```rust
pub fn run_indexed<T, F>(&self, jobs: Vec<F>, ...) -> Vec<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
```

Every job closure (and its return type) must be `'static` — owned outright, or `Arc`-shared. This
is a **breaking API change** to `run_indexed`, surfaced here for ratification rather than shipped
silently as an implementation detail (house rule #3 / the M-864 DoD's own requirement).

### 2.1 Caller-by-caller audit

The M-864 brief's working assumption — "every current caller already passes `'static`/owned data"
— turned out to be **only partially correct**; the audit below is the actual, checked account, not
the assumption:

| Caller | Borrowed what, pre-M-864 | Fix |
|---|---|---|
| `mycelium-bench::scaling::time_batch` | Nothing — `independent_job` already captures only `case_src: &'static str` and `backend: Backend` (`Copy`). | **No change needed.** |
| `mycelium-mlir::llvm::emit_llvm_ir_many_with_swap_mode` (M-860) | `&nodes[i]` (a slice element borrowed from the caller's `nodes: &[Node]` parameter). | Clone the `Node` per job before capturing it (`Node: Clone`; already the pattern used elsewhere in the interpreter, e.g. `eval_to_normal_node`'s `node.clone()`). The content-hash sort that pins the batch's determinism runs over the *original* `nodes` before cloning, so this changes nothing about which node lands in which output slot — only how each job captures its input. |
| `mycelium-interp::parallel::eval_top_batch` (M-862) | `&self` (`&Interpreter`, which owns a `Box<dyn SwapEngine>`) and `fuel_ref: &AtomicU64` (a stack-local counter). | `Interpreter` is now `Clone` (see §2.2) and cloned once per batch behind an `Arc`, so every job gets a cheap `Arc::clone` handle rather than a borrow; the fuel counter moves from a stack-local `AtomicU64` borrowed by reference to an `Arc<AtomicU64>` cloned per job — same shared-counter semantics (still one logical counter, still `Arc`-shared, not duplicated), just `'static`-owned. |
| `mycelium-std-runtime::dataflow::run_dataflow_scheduled` (M-711) | `&mut Box<dyn PollTask + Send>` (a mutable slice-element borrow, released between sweeps). | Not mentioned in the M-864 brief at all — a real gap in its stated scope, found only by attempting to build the whole workspace. Fixed by taking **ownership** of each still-pending task out of the slice (`std::mem::replace` with a transient placeholder `PollTask`) for the duration of one sweep's parallel poll, then restoring it to its original index once the result is in hand. The boxed task's *contents* were always `'static` (a `Box<dyn Trait>` defaults to `+ 'static`); only the `&mut` borrow of the caller's slice was ever the obstacle. |
| `mycelium-std-runtime::supervision::run_supervised` (M-713) | `token: &CancelToken` (borrowed). | Each job clones `token` (`CancelToken: Clone`, `Arc<AtomicBool>`-backed) instead of borrowing it. Cloning preserves the sharing semantics exactly — every clone reads/writes the *same* underlying flag, so cancellation propagation across siblings is unchanged. |

**Grounding for the two `mycelium-std-runtime` fixes:** these were not named in the M-864 issue
body's "current callers" list, but both crates are in `mycelium-mlir`'s transitive dependency
graph, so leaving them broken would have failed `cargo build --workspace` outright — an
unacceptable silent regression. Both are recorded here for completeness and because DN-67 is the
place a future reader would look to understand why `run_indexed`'s signature changed; the fix in
each case follows the same "convert a borrow into an owned/`Arc`-shared value" pattern as the two
callers the issue anticipated.

### 2.2 `Interpreter` becomes `Clone`

`mycelium-interp::Interpreter` gained `#[derive(Clone)]`, made viable by changing its `swap` field
from `Box<dyn SwapEngine>` to `Arc<dyn SwapEngine>` (the public constructor `Interpreter::new` still
accepts an owned `Box<dyn SwapEngine>` — unchanged signature — converting it internally via the
unconditional, allocation-free `Arc::from(Box<T>)`). `SwapEngine`'s trait bound widened from `Sync`
to `Send + Sync` (`Arc<dyn Trait>` is `Send` only if the trait object is `Send + Sync`); every
shipped implementation (`IdentitySwapEngine`, `mycelium-cert`'s certified engines) is already a
plain, interior-mutability-free struct and satisfies both automatically. Cloning an `Interpreter` is
now cheap: an `Arc::clone` bump for `swap`, a small `BTreeMap` clone for `prims` (bounded by the
built-in prim count, not by program size), and a `Copy` for `fuel`.

## 3. The persistent pool design (`mycelium-sched::pool`)

One process-wide singleton (`static POOL: OnceLock<Arc<Pool>>`), lazily created on first use, sized
to `thread::available_parallelism()` (fallback 1 — never 0). Its threads are never joined; they
live for the process's duration, the same pattern as a typical global thread-pool singleton (e.g.
Rayon's default pool).

### 3.0 The correctness rewrite (2026-07-01 — an adversarial review reproduced a real hang)

**The first implementation of this design was unsound and is superseded by the account below.** It
kept M-861's demand-signalled backpressure: a **feeder** loop that `Condvar::wait`s while the
per-lane deques already hold `capacity` items, and it reached that wait **before** entering the
help-steal loop. Two defects followed:

- **Defect 1 (deadlock — reproduced).** A nested `run_indexed` call is a pool worker running a job
  that submits its own batch and then becomes that batch's feeder. If that feeder bare-blocks on the
  `capacity` gate (because its lanes haven't drained yet), it drains **nothing** while it waits.
  With a wide enough fan-out relative to the worker count (`width > capacity + P`), enough nested
  feeders bare-block simultaneously that **every** pool thread is parked and no thread is left to
  run the lane-tasks that would let any feeder proceed — a permanent hang. Reproduced
  deterministically at forced `P ∈ {1,2,3,4}` with the `[15,15,6]` shape (see §3.3). This shape
  would hang the original committed `nested_wide_fanout` test on any ≤ 4-core machine (e.g. a default
  4-vCPU CI runner); it only "passed" on the 28-core development box, where `capacity = 2·P = 56 >
  15` so the feeder never blocked.
- **Defect 2 (panic hangs the join + kills the pool).** A panicking job unwound past the batch's
  completion decrement, so the join's countdown never reached zero (`help_while` hangs forever) and
  the panic propagated out of the pool worker's stack, **permanently killing that worker** — the
  fixed pool shrinks toward zero with no replenishment. A regression from `std::thread::scope`,
  which propagates a panic cleanly at join.

Both are fixed at the root:

**Fix 1 — no bare-block on the batch's own progress; unbounded queue.** The demand-signalled
`capacity` backpressure is **removed entirely** (it was a non-normative implementation detail per
DN-61 §A.2, and it was the deadlock's cause). `Scheduler::run_indexed` now:

1. **populates every lane's deque up front**, round-robin by spawn index, with *no* capacity gate —
   the whole batch is materialized before any lane runs (memory bounded by the batch's job count,
   which the caller already holds in `jobs: Vec<F>` — no new blowup);
2. **then** submits the `min(workers, n)` lane-loop tasks to the pool.

Because every deque is full before any lane starts and **no work is ever added later**, a lane never
has to *wait* for more work: it pops its own deque (LIFO), steals when empty (FIFO), and **exits the
instant nothing remains anywhere**. There is no feeder `Condvar` and no lane `Condvar` — the
lane-loop is **totally non-blocking**. The only "wait" left on any batch's critical path is
`Pool::help_while`, which *runs* pending tasks rather than parking. This is what restores the
deadlock-freedom induction (§3.2): every thread that would otherwise wait is instead actively
draining the shared queue.

**Fix 2 — panic-safe join (thread::scope-like).** Each job runs under `std::panic::catch_unwind`
inside its lane-loop, so a panicking job (a) never kills the persistent pool worker, and (b) has its
payload captured. The **first** captured job panic is re-raised in the calling thread *after* the
join, via `std::panic::resume_unwind` — matching `std::thread::scope`'s panic-propagates-at-join
semantics. An RAII **drop-guard** decrements the batch's outstanding-lane counter on **every** exit
path (normal return or unwind) and wakes the join, so no panic can leave `help_while` hanging.
`Pool::worker_loop` and `Pool::help_while` additionally run every task under `catch_unwind` as a
last-line guard, so no stray unwind (e.g. a poisoned lock) can ever escape into the pool's control
loops and kill a worker.

Because backpressure is gone, the former `Exact` `SCHEDULER_BACKPRESSURE_STRENGTH` guarantee and its
re-export are **removed** (they tagged a bound that no longer exists — leaving them would be a false
`Exact` claim, VR-5). `Scheduler::capacity` / `with_workers(_, capacity)` are retained for source
compatibility but **no longer bound anything**, documented as such (never-silent).

### 3.1 The help-stealing pattern

`Pool::help_while(done: impl FnMut() -> bool)` is the Cilk/TBB/Rayon "work-helping" pattern: a
thread that would otherwise block on its own batch's completion instead executes any pending task
from the shared queue — its own batch's, or anyone else's, at any nesting depth — until its `done`
condition holds. A thread waiting on its own batch is therefore never idle-and-unproductive; it is
an *additional* worker for as long as it waits. Combined with Fix 1 (no other bare-block anywhere on
a batch's critical path), this is what makes a **fixed**-size pool safe under **unbounded** nesting
depth: the resource that grows with nesting is *helpers-currently-waiting*, never OS threads.

### 3.2 Deadlock-freedom argument — `Empirical`

Model the live system as a forest of "batches" (one per in-flight `run_indexed` call), each with a
finite, positive count of outstanding lane-loop tasks. A *helper* is any thread not currently
running a task — the `P ≥ 1` persistent workers, or any caller (at any nesting depth) currently
inside `help_while`. **The load-bearing invariant, now actually true (Fix 1): nothing on a batch's
critical path ever bare-blocks** — a lane-loop only pops/steals/exits (never waits), and the sole
wait is `help_while`, which runs tasks. So every helper is, by construction, actively trying to pop
a task from the shared queue, never merely parked. As long as the queue is non-empty, some helper
dequeues and runs a task in bounded time (the queue is a `Mutex`-guarded FIFO; nothing is skipped or
starved). Running a task either (a) completes a lane-loop permanently, or (b) runs a user job, which
— by the pre-existing "pure task" contract `run_indexed` has always carried — terminates in finite
time, possibly after submitting and waiting (recursively, via the same `help_while`) on a strictly
nested, strictly smaller sub-batch. Because the nesting is a finite call tree (finite program,
finite fuel per RFC-0007 §4.5's cooperative-stepping budget bounding every task's own runtime),
induction on tree depth gives: every leaf batch completes outright (its lane-loops are plain jobs
with no further waiting); given every leaf batch completes, every batch one level up completes; and
so on to the root. No step assumes `P` is large enough for any particular *concurrency width* — only
that `P ≥ 1`, so the queue is never permanently unattended. **Conclusion: no deadlock, for any fixed
`P ≥ 1`, at any nesting depth.**

### 3.3 Validation — the tests that hang on the pre-fix code and pass on this one

This is the informal, structural argument, not a mechanized proof — tagged **`Empirical`**, checked
by `mycelium-sched::tests::scheduler`. The **decisive** additions are the **forced-low-worker-count**
tests (a `pub(crate)` `Pool::with_workers_for_test(P)` + `Scheduler::run_indexed_on(pool, …)` hook
threads a small, explicit `P` through every nesting level — the only way to exercise the deadlock on
a many-core box, since the global pool is sized to `available_parallelism()`):

- `forced_low_p_wide_fanout_does_not_deadlock_p1_through_p4` — the `[15,15,6]` shape at forced
  `P ∈ {1,2,3,4}` under a 60s wall-clock timeout. **This is the direct regression test for the
  reproduced hang.** Verified honestly: with the pre-fix feeder-block reintroduced (a scratch revert
  of just Fix 1, with the same test hook added), this test **hangs at every `P`** (the timeout fires
  at ~15s with "suspected deadlock"); on the fixed code it passes.
- `forced_low_p_deep_chain_and_mixed_shapes_do_not_deadlock` — deep chain + irregular shapes (incl.
  empty/single-item sub-batches) at forced `P ∈ {1,2,3,4}`.
- `forced_p1_single_worker_nested_is_the_hardest_case_and_still_completes` — `P = 1` (all
  concurrency from the caller's own `help_while`), a width-6 depth-3 tree.
- `a_panicking_job_propagates_at_join_without_hanging_and_pool_survives` — Defect 2: a panicking job
  re-raises at the join, and a **subsequent** batch on the same forced pool still completes (proving
  the worker did not die and the join did not hang).
- `a_nested_panic_propagates_up_through_the_nesting_without_hanging` — a deeply-nested panic
  propagates up through every join level.

Plus the global-pool nested tests carried over from the first implementation (deep chain, wide
fan-out, mixed shapes, 50×-repeat determinism, and the Linux `/proc/self/status` thread-count
regression witness that the pool never grows with nesting depth). All ran green across 15+
consecutive full-suite invocations with zero flakes during this change's verification.

## 4. Determinism is untouched

`run_indexed`'s RT2 contract — spawn-order-indexed results, regardless of steal schedule — is
**unchanged** by both the original design and the §3.0 correctness rewrite: results are still
written into a pre-sized `Vec<Option<T>>` by job index, read off after every lane has finished (and,
post-rewrite, only after any captured job panic has been re-raised, so a panic surfaces before any
`None` slot is read). The pool module knows nothing about job *order*; it is a bag of `'static`
closures. M-860's byte-identical parallel-vs-sequential emit test
(`tests::llvm::parallel_emit_matches_sequential_emit_byte_identical`) and M-862's parallel-eval
differential (`tests::parallel::parallel_eval_matches_sequential_eval_over_the_pure_corpus`, plus
its own repeated-run determinism test) both stayed green, unmodified in their assertions, through
both the original change and the rewrite.

## 5. Definition of Done

- [x] The `'static` contract change is ratified here (this note), not silently shipped as an
      implementation detail.
- [x] A nested-join wait-loop (`Pool::help_while`) is documented with a structural deadlock-freedom
      argument (§3.2, updated for the §3.0 rewrite so the "no bare-block" invariant it relies on is
      actually true) and checked by **forced-low-worker-count** nested stress tests that *reproduce
      the original hang on the pre-fix code and pass on the fixed code* (§3.3) — `Empirical`, not
      `Proven` (no mechanized proof is in-repo; VR-5).
- [x] Panic-safety (§3.0 Fix 2): a panicking job does not hang the join or kill the pool, and the
      first panic re-raises at the join (thread::scope-like) — checked by two panic tests.
- [x] Nested stress tests (multiple levels of nested `run_indexed`, wide fan-out, mixed shapes) pass
      without deadlock or starvation, under a wall-clock timeout, at forced `P ∈ {1,2,3,4}` and on
      the global pool.
- [x] M-860 and M-862 are re-validated unaffected: their existing differentials
      (`parallel_emit_matches_sequential_emit_byte_identical`,
      `parallel_eval_matches_sequential_eval_over_the_pure_corpus`, and the rest of each crate's
      suite) pass unmodified. Neither was extended to exploit nesting (out of scope for this
      issue — M-862's module docs still note the top-level-only bound is a *choice*, not a
      limitation, now that nesting is cheap at the scheduler level).
- [x] `just check`-equivalent gates green for the touched crates: `cargo fmt --check`; `cargo
      clippy -p mycelium-sched -p mycelium-mlir -p mycelium-interp -p mycelium-std-runtime
      --all-targets -- -D warnings -A unsafe_code`; `cargo test` for `mycelium-sched`,
      `mycelium-mlir`, `mycelium-interp`, and `mycelium-std-runtime`; `cargo build --workspace`
      succeeds.
- [x] Honest tags throughout: `Empirical` where a property/stress test is the checked basis, never
      upgraded to `Proven` without a mechanized argument (VR-5). The obsolete `Exact`
      `SCHEDULER_BACKPRESSURE_STRENGTH` guarantee is **removed**, not left as a false claim.

## Meta — changelog

- 2026-07-01 — Drafted (M-864 leaf). Records the `'static` contract change, the help-steal
  persistent-pool design, its deadlock-freedom argument, and the caller-by-caller audit (including
  the two `mycelium-std-runtime` callers not named in the original issue body).
- 2026-07-01 — **Correctness rewrite (§3.0), same day, after an adversarial deadlock review
  reproduced a real hang.** The first implementation kept M-861's `capacity` backpressure, whose
  feeder bare-blocked *before* help-stealing → a nested-submission deadlock at `width > capacity + P`
  (reproduced at forced `P ∈ {1,2,3,4}` with `[15,15,6]`), plus a panic that hung the join and killed
  a pool worker. Fixed at the root: unbounded queue (populate-all-then-run, so no lane or feeder
  ever bare-blocks) + a panic-safe join (`catch_unwind` per job, first panic re-raised at join, RAII
  drop-guard on the countdown). Backpressure (`SCHEDULER_BACKPRESSURE_STRENGTH`, `Exact`) is dropped
  as a non-normative impl detail (DN-61 §A.2); `capacity` is retained but no longer bounds anything.
  Added forced-low-`P` deadlock tests + panic tests; the reproduction on pre-fix code was verified in
  a scratch revert.
