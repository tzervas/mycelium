# Devlog — 2026-06-16 · Typed channels, and the sequentialization that stopped working

> **What this is** (see `docs/notes/Narrative-Capture-and-Authoring.md`): the *narrative* layer — the
> messy middle the RFCs smooth over. Append-only, informal, honest. The RFCs/ADRs/DNs remain the source
> of truth; this is the *story* of how a decision actually got made. Refs point at what shipped.

**Theme.** The fork/join runtime (M-357) landed last session with a clean RT2 differential:
`run_interleaved` ≡ `run_sequential`. The module comment even named the next slice — *typed SPSC
channels*. This session built them. The interesting part wasn't the channel; it was discovering that the
differential we were so proud of **does not survive contact with channels**, and finding the honest
replacement.

---

## 1. The bug that isn't a bug: a strict sequential run *deadlocks*

`run_sequential` polls each child **to completion, in spawn order**. For pure fork/join that's the
canonical sequentialization — the thing the interleaved schedule must match. The instinct was to reuse it
as the reference for channels too.

It can't work. Spawn a consumer as child 0 and its producer as child 1. `run_sequential` polls child 0 to
completion *first* — but child 0 is blocked waiting for a value child 1 hasn't sent yet, because child 1
hasn't run. The "reference" run hangs. The sequentialization that defined RT2 for fork/join is **not even
a terminating schedule** once tasks communicate.

This is not a defect in the channel — it's the Kahn condition reasserting itself. A Kahn process network is
deterministic over *fair* schedules; "run one process to completion, then the next" is precisely the
**unfair** schedule the theorem excludes.

## 2. The honest replacement: two fair schedules, not one canonical one

So the RT2 obligation for communicating tasks had to be re-stated. We can't say "interleaved ≡ the
sequential run" because there is no valid sequential run. What Kahn (T4.1) *does* promise is that **any
fair schedule yields the same observable** — same per-task outcomes, same channel transcripts.

That's directly testable, and it's a *stronger* claim than the old differential: run the same network under
two genuinely different fair schedules — `SweepOrder::Ascending` and `Descending` — and assert they agree.
Ascending favours the producer-then-consumer order; descending the reverse; if backpressure or draining
order leaked into the result, they'd diverge. They don't. (The old fork/join differential still stands for
the no-channel case; `run_dataflow` is the communicating sibling, not a replacement.)

**Honesty note.** Kahn gives a *theorem*, so the temptation was to tag the determinism `Proven`. We didn't.
The side conditions (pure processes, single blocking reader, finite buffers) hold by construction here, but
no mechanized proof ships in-repo — so the shipped evidence is the differential, and the tag is
`Empirical` with T4.1 *cited as the basis*. Never upgrade without a checked basis (VR-5). The day someone
mechanizes it, the tag moves; not before.

## 3. Deadlock had to become *data*

A cooperative single-threaded scheduler cannot block — there is no OS thread to park. So a genuinely stuck
network (everyone waiting on a channel nobody will feed) can't be a hang; it has to be an explicit value.
But how does the scheduler *know* the network is stuck, versus just slow?

The signal we needed: "did anything happen this sweep?" A task resolving is obvious progress. The subtle
case is a task that does a successful `try_send`/`try_recv` but isn't done — it returns `Pending` just like
a *parked* task does. The poll result alone can't tell them apart.

The fix is to make channel progress observable to the scheduler without coupling it to channel internals.
A `Network` owns a single monotone **epoch** — a count of successful sends+recvs across all its channels.
`run_dataflow` reads it before and after each full sweep. *Progress this sweep = a task resolved OR the
epoch advanced.* A full sweep with neither, while children remain pending, is a `Deadlock { parked }` —
the blocked set, listed, inspectable (SC-3), never a silent hang (G2). The epoch is a clean by-product: a
readable, EXPLAIN-able "how much has this network actually done" counter.

This also kept the layering honest: `runtime.rs` never learns what a channel *is*. It takes a
`progress: impl Fn() -> u64` closure and watches a number. Channels live entirely in `channel.rs`; the
scheduler just needs a heartbeat.

## 4. The small stuff that mattered

- **Close has to be a `drop`.** A producer that holds its `Sender` for its whole lifetime never lets the
  consumer see end-of-stream — the `Sender` only drops at scope join, *after* the run. So the producer
  task holds `Option<Sender<…>>` and sets it to `None` when done; that drop flips `senders → 0`, and the
  consumer's next `try_recv` drains then returns `Closed`. End-of-stream is a real, observable event
  mid-run, not an afterthought.
- **Failure returns the value.** `try_send` on a full or disconnected channel hands the value *back*
  (`Full(v)`/`Disconnected(v)`). A send that doesn't happen must not eat the payload — that's the
  never-silent rule applied to a queue.
- **No `unsafe`.** Single-threaded cooperative scheduling means every `RefCell` borrow opens and closes
  inside one `poll`, before the task yields. Borrows never overlap, so `Rc<RefCell<…>>` is sufficient and
  honest — no atomics, no `unsafe`, no pretending we're thread-safe when we're not (yet; that's R2).

**Refs.** `crates/mycelium-mlir/src/channel.rs`, `…/runtime.rs` (`run_dataflow`, `SweepOrder`,
`Deadlock`); RFC-0008 §4.3 + §4.6 staging note + Meta-changelog. Verified: 6 channel tests (backpressure,
close, disconnect, in-order delivery, Kahn determinism, explicit deadlock); `just check` green.
