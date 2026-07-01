# Design Note DN-67 — Persistent Work-Stealing Pool (M-864)

| Field | Value |
|---|---|
| **Note** | DN-67 |
| **Status** | **Draft** (2026-07-01, authored by the M-864 leaf) |
| **Decides** | Ratifies the `Scheduler::run_indexed` closure/return-type contract change M-864
requires — `F: FnOnce() -> T + Send + 'static` and `T: Send + 'static` (previously `F: Send`,
`T: Send`, with borrowing freely permitted via `std::thread::scope`) — and records the
help-stealing design that makes nested `run_indexed` submission safe and cheap on a **persistent,
bounded** pool. |
| **Feeds** | `mycelium-sched::scheduler::Scheduler::run_indexed`; its consumers
`mycelium-mlir::llvm::emit_llvm_ir_many_with_swap_mode` (M-860), `mycelium-interp::parallel::eval_top_batch`
(M-862), `mycelium-std-runtime::dataflow::run_dataflow_scheduled` (M-711), `mycelium-std-runtime::supervision::run_supervised` (M-713). |
| **Depends on** | DN-61 §A.2 (R1 scheduler normativity — internal scheduling strategy is
explicitly **not** normative; only RT2 determinism + fuel-compatible cooperative stepping + RT7
scope discipline are); RFC-0008 §4.1 RT1–RT3, §4.7; M-861 (per-worker deques + steal-on-empty, the
dispatch/steal logic this note's pool reuses verbatim). |
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

`Scheduler::run_indexed` keeps its existing per-call dispatch shape from M-861 **completely
unchanged**: `workers` lanes (per-batch deques), round-robin feed, LIFO-own/FIFO-steal via
`StealPolicy::select_victim`, a `capacity`-bounded backpressure counter. The only thing that
changed is *how the lane-loop bodies get executed*: instead of `thread::scope` spawning `workers`
fresh OS threads to run them, each lane-loop is submitted as one `'static`, boxed task
(`Box<dyn FnOnce() + Send + 'static>`) to the shared pool's queue, and the calling thread — after
finishing its feeder duty — **helps** drain the shared queue (any pending task, from this batch or
any other, at any nesting depth) until every one of its own batch's lanes has finished.

### 3.1 The help-stealing pattern

A bounded, persistent pool creates an obvious hazard: if a pool thread submits a nested batch and
then just *blocks* waiting for it, and every other pool thread is doing the same for its own nested
batch, the pool can wedge with no thread actually running anything.

`Pool::help_while(done: impl FnMut() -> bool)` is the fix — the Cilk/TBB/Rayon "work-helping"
pattern: a thread that would otherwise block instead executes any pending task from the shared
queue until its own `done` condition holds. A thread waiting on its own batch's completion is
therefore never idle-and-unproductive; it is an *additional* worker for as long as it waits. This
is what makes a **fixed**-size pool safe under **unbounded** nesting depth: the resource that grows
with nesting is *helpers-currently-waiting*, never OS threads.

### 3.2 Deadlock-freedom argument — `Empirical`

Model the live system as a forest of "batches" (one per in-flight `run_indexed` call), each with a
finite, positive count of outstanding lane-loop tasks. A *helper* is any thread not currently
running a task — the `P ≥ 1` persistent workers, or any caller (at any nesting depth) currently
inside `help_while`. Every helper is, by construction, actively trying to pop a task from the
shared queue, never merely parked. So as long as the queue is non-empty, some helper dequeues and
runs a task in bounded time (the queue is a `Mutex`-guarded FIFO; nothing is skipped or starved by
construction). Running a task either (a) completes a lane-loop permanently, or (b) runs a user job,
which — by the pre-existing "pure task" contract `run_indexed` has always carried — terminates in
finite time, possibly after submitting and waiting (recursively, via the same `help_while`) on a
strictly nested, strictly smaller sub-batch. Because the nesting is a finite call tree (finite
program, finite fuel per RFC-0007 §4.5's cooperative-stepping budget bounding every task's own
runtime), induction on tree depth gives: every leaf batch completes outright (its lane-loops are
plain jobs with no further waiting); given every leaf batch completes, every batch one level up
completes (its own lane-loops' jobs each wait only on leaf batches); and so on to the root. No step
assumes `P` is large enough for any particular *concurrency width* — only that `P ≥ 1`, so the
queue is never permanently unattended. **Conclusion: no deadlock, for any fixed `P ≥ 1`, at any
nesting depth.**

This is the informal, structural argument, not a mechanized proof — tagged **`Empirical`**, checked
by (`mycelium-sched::tests::scheduler`):

- `nested_deep_chain_matches_sequential_reference_no_deadlock` — a 40-level chain (width 1 at every
  level), under a 30s wall-clock timeout.
- `nested_wide_fanout_matches_sequential_reference_no_deadlock` — 3 levels deep, wide fan-out
  (15×15×6), every sibling recursing, so many nested batches are concurrently in flight.
- `nested_mixed_batch_sizes_including_empty_and_single_item_match_reference` — an irregular shape
  mixing a zero-width level, a single-item level, and ordinary wider levels.
- `nested_empty_and_single_item_batches_never_hang` — the `n == 0` fast path (never touches the
  pool at all) and a single-job nested call.
- `nested_recursion_is_deterministic_across_many_repeated_runs` — 50 repeated runs each over a deep
  chain, a wide fan-out, and a mixed shape, asserting exact equality with a pure sequential
  reference every time (not a one-off pass).
- `nested_recursion_thread_count_is_bounded_not_growing_with_depth` (Linux-only, via
  `/proc/self/status`'s `Threads:` line) — a 40-deep chain's peak observed OS thread count stays
  within a small, depth-independent constant of `available_parallelism()`, the direct regression
  witness for "the persistent pool never grows with nesting depth".

All of the above ran green across repeated full-suite invocations (10+ consecutive runs with zero
flakes observed) during this change's own verification, in addition to being part of the committed
suite.

## 4. Determinism is untouched

`run_indexed`'s RT2 contract — spawn-order-indexed results, regardless of steal schedule — is
**unchanged**: results are still written into a pre-sized `Vec<Option<T>>` by job index, read off
after every lane has finished. The pool module knows nothing about job *order*; it is a bag of
`'static` closures. M-860's byte-identical parallel-vs-sequential emit test
(`tests::llvm::parallel_emit_matches_sequential_emit_byte_identical`) and M-862's parallel-eval
differential (`tests::parallel::parallel_eval_matches_sequential_eval_over_the_pure_corpus`, plus
its own repeated-run determinism test) both stayed green, unmodified in their assertions, through
this change.

## 5. Definition of Done

- [x] The `'static` contract change is ratified here (this note), not silently shipped as an
      implementation detail.
- [x] A nested-join wait-loop (`Pool::help_while`) is documented with a structural deadlock-freedom
      argument and checked by nested-submission property/stress tests (§3.2) — `Empirical`, not
      `Proven` (no mechanized proof is in-repo; VR-5).
- [x] Nested stress tests (multiple levels of nested `run_indexed`, wide fan-out, mixed shapes)
      pass without deadlock or starvation, under a wall-clock timeout.
- [x] M-860 and M-862 are re-validated unaffected: their existing differentials
      (`parallel_emit_matches_sequential_emit_byte_identical`,
      `parallel_eval_matches_sequential_eval_over_the_pure_corpus`, and the rest of each crate's
      suite) pass unmodified. Neither was extended to exploit nesting (out of scope for this
      issue — M-862's module docs still note the top-level-only bound is a *choice*, not a
      limitation, now that nesting is cheap at the scheduler level).
- [x] `just check`-equivalent gates green for the touched crates: `cargo fmt --check`; `cargo
      clippy -p mycelium-sched -p mycelium-mlir -p mycelium-interp --all-targets -- -D warnings -A
      unsafe_code`; `cargo test` for `mycelium-sched`, `mycelium-mlir`, `mycelium-interp`, and
      `mycelium-std-runtime` (the two additional callers found during the audit); `cargo build
      --workspace` succeeds.
- [x] Honest tags throughout: `Empirical` where a property/stress test is the checked basis, never
      upgraded to `Proven` without a mechanized argument (VR-5).

## Meta — changelog

- 2026-07-01 — Drafted (M-864 leaf). Records the `'static` contract change, the help-steal
  persistent-pool design, its deadlock-freedom argument, and the caller-by-caller audit (including
  the two `mycelium-std-runtime` callers not named in the original issue body).
