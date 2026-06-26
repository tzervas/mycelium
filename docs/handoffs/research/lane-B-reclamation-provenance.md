# Lane B Research Report — RFC-0027 Reclamation Provenance

> **Status:** Research handoff — `Declared` throughout (design-phase stub analysis).
> No claims are `Proven`; nothing here is normative. This report is input to the
> r10 kickoff and to RFC-0027's open-questions resolution.
>
> **Date:** 2026-06-24
> **Lane:** B — RFC-0027 reclamation-provenance
> **Confidence tagging:** per VR-5 — `Declared` (asserted intent / external-doc summary),
> `Empirical` (observed in multiple independent implementations), `Proven` (mechanized
> results cited from published work). No claim in this report is `Proven` unless a
> mechanized or published proof is explicitly cited.

---

## 1. Central Question

RFC-0027 (Draft, 2026-06-23) names the gap: Mycelium's runtime has a structured-concurrency
scope tree (RFC-0008 RT7), a sweep-order property (RFC-0008 §4.3), and a "no silent GC pause"
honesty stance (G2/VR-5) — but no normative model for *how memory backing immutable values
is reclaimed*, *what an EXPLAIN/audit record for a reclamation event must contain*, or *whether
reclamation order must total-order with sweep order or may be partial*.

Three subquestions this report addresses:

1. **What must a reclamation EXPLAIN/audit record contain** to make resource finalization
   trustworthy when ownership crosses hypha/channel boundaries?
2. **Is there a required total order between RFC-0008 sweep-order and reclamation order**, or
   can it be partial while preserving auditability?
3. **How do interior-mutability / shared-mutable patterns reclaim under value-semantics**,
   and do Mycelium's LR-8/LR-9 guarantees eliminate the hard cases?

---

## 2. Mycelium Corpus Grounding

### 2.1 RFC-0027 open questions (quoted verbatim)

RFC-0027 §5 lists five open questions that define the decision space:

> **Model choice:** Should reclamation be purely Rust-drop-order (implicit, no surface) with
> `reclaim` as a supervision primitive only, or should Mycelium expose explicit "reclaim regions"
> analogous to arena allocators? The former is simpler (KC-3); the latter is more EXPLAIN-able.
>
> **Sweep-order coupling:** RFC-0008 §4.3 defines sweep order for Kahn determinism. Is the
> reclamation cascade *required* to follow the same order (strong coupling), or merely *permitted*
> to? The strong coupling is property-testable; the weak coupling is more flexible.
>
> **`reclaim` construct scope:** ADR-020 lists `reclaim` as a supervision primitive for runtime
> units (tasks), not a memory primitive. Should this RFC also define memory-level reclamation, or
> only the task-reclamation surface?
>
> **Pause budget:** Is "no silent GC pause" a hard real-time bound (must specify a worst-case
> budget) or a best-effort stance (the guarantee is `Declared`/honesty, not a latency SLO)?
>
> **Checkpoint interaction:** A `cyst` checkpoint writes live values to content-addressed storage.
> Does the checkpoint operation reclaim the original allocation (checkpoint-and-free) or copy
> (checkpoint-and-keep)? The former is more memory-efficient; the latter is safer.

### 2.2 RFC-0008 sweep-order property (§4.3)

The Kahn-determinism obligation requires that under any two fair schedules (`SweepOrder::Ascending`
/ `SweepOrder::Descending`) a communicating network of tasks produces identical per-task outcomes
and channel transcripts. The `SweepOrder` is currently `Empirical`-tagged (the differential is the
evidence; no mechanized proof). This gives sweep order a *scheduling* semantics but not yet a
*reclamation* semantics.

RFC-0008 §4.3 also establishes the never-silent backpressure and close protocol: `TrySend::Full(v)`
returns the value rather than dropping it; `TrySend::Disconnected(v)` surfaces a hung-up receiver.
These are the concrete G2 applications. **The reclamation model must preserve these invariants
through scope exit**: a value that was live in a channel at scope-exit must be accounted for, not
silently dropped.

### 2.3 RFC-0008 RT7 structured lifetimes

> "Every hypha is created inside a scope, and a scope does not exit until its children have
> completed, been cancelled, or been explicitly detached into another owning scope." (RFC-0008 §4.1)

This structured-concurrency invariant is load-bearing for reclamation: it means the scope tree
is a **DAG with a single exit order** — parent cannot exit before children. This is the same
constraint Tofte-Talpin LIFO enforces at the region level, but RT7 enforces it at the task level.
The question RFC-0027 must answer is whether *memory* reclamation inherits the same DAG discipline
or can be independently scheduled.

### 2.4 RFC-0008 §4.4 `cyst` checkpointing

A `cyst` contains: (a) values in scope, (b) continuation by content hash, (c) `Meta` needed to
resume honestly. The checkpoint operation serializes live values. RFC-0027 §5's "checkpoint
interaction" open question is whether checkpoint-and-free or checkpoint-and-keep is the model.
The content-addressed nature of `cyst` artifacts means that the *value* survives even if the
original allocation is freed — this is the key insight that makes checkpoint-and-free sound
(the value's identity is its hash, not its address).

### 2.5 RFC-0008 `reclaim` semantic scope (§4.5, DN-03 §4)

RFC-0008 §4.5 is explicit: "`reclaim` — supervision-tree reclamation of *runtime units*
(**never memory** — LR-9)." DN-03 §4 repeats: "runtime-unit reclamation — clear; scope clarified
(RFC-0008 RT7): reclaims *stale runtime units*, **never memory** (LR-9 makes memory reclamation
automatic; a memory-`reclaim` would contradict it)."

This is the most important corpus constraint: **the `reclaim` surface construct is task/unit
supervision, not a memory primitive**. Memory reclamation rides Rust's drop system at the
implementation level. RFC-0027 must decide what EXPLAIN surface, if any, sits above that drop.

### 2.6 LR-8/LR-9 value-semantics constraints

LR-9: "In safe Mycelium a memory leak is not expressible." LR-8: values are immutable. Together:
no shared mutable state (RT1), no cycles (RFC-0027 §3 "Out of scope: cycle detection — values are
acyclic by LR-9"), and no aliased mutation to police. This eliminates the hardest GC problems —
cycle detection and write-barrier cost — leaving only the **cross-boundary ownership transfer**
problem as the genuinely open question.

---

## 3. External Prior Art

### 3.1 Region-based memory management (Tofte-Talpin, 1994/1997)

*Confidence: `Empirical` — the ML Kit implementation is published and well-studied.*

Tofte and Talpin's region inference assigns every allocation to a statically-inferred region and
enforces a **LIFO (stack) discipline**: inner regions must be freed before outer ones. The key
reclamation property is **lexical containment** — a region's lifetime is bounded by the enclosing
`letregion` block. No runtime type tags are needed at deallocation; all safety comes from
static analysis.

**Relevance to RFC-0027:** RT7's structured-concurrency scope tree *already enforces* the analog of
LIFO at the task level (children before parents). If memory reclamation is coupled to scope-exit,
Mycelium gets the Tofte-Talpin safety property for free from the scope tree, with no separate
region inference required. The open question is whether values transferred *across* scope boundaries
(via channels) can be reclaimed with the same simple discipline — Tofte-Talpin's original
formulation explicitly forbids cross-scope transfer; the Aiken et al. relaxation introduces
complexity that Mycelium's channel-as-ownership-transfer model may avoid differently.

**Key implication for RFC-0027:** The simplest sound model is scope-exit reclamation (children
reclaimed before parent, following RT7 order). Cross-scope values in transit require explicit
ownership transfer — the channel close protocol already provides this (`TrySend::Disconnected`
means ownership returns to the sender; `Closed` at `TryRecv` means the receiver drains then the
channel is gone). The EXPLAIN record for this case need only capture: scope ID, epoch, which
channel closed, and the value's `Meta` provenance chain.

*Sources: [Region-Based Memory Management — Semantic Scholar](https://www.semanticscholar.org/paper/Region-based-Memory-Management-Tofte-Talpin/9117c75f62162b0bcf8e1ab91b7e25e0acc919a8);
[Wikipedia: Region-based memory management](https://en.wikipedia.org/wiki/Region-based_memory_management);
[A Retrospective on Region-Based Memory Management — Springer](https://link.springer.com/article/10.1023/B:LISP.0000029446.78563.a4)*

### 3.2 RAII and deterministic finalization (C++, Rust)

*Confidence: `Empirical` — well-established in production runtimes.*

RAII ties resource lifetime to object (stack) lifetime: destructors run in **reverse construction
order** when a scope exits. Rust's ownership + drop system is the direct ancestor: `Drop::drop` is
called at the end of the owning binding's lexical scope, in reverse order of construction.

**Key RAII property for provenance:** the implicit "reclamation record" under RAII is the *call
stack at drop time* — scope chain, drop order, and the type's `Drop` impl. This is adequate for
single-threaded programs. **The concurrency gap:** RAII does not by itself answer *which thread
owns the destructor obligation* when an object is transferred across an async task boundary. Rust's
`Send` bound makes the transfer explicit but does not record it. Java finalizers are nondeterministic
in timing and order across threads — the canonical example of what Mycelium's "no silent GC pause"
stance is rejecting.

**Relevance:** Mycelium's current baseline (Rust drop for values within `Scope<T,E>`) inherits
RAII's deterministic single-scope ordering *within* a scope but says nothing about the order of
drops *across* sibling scopes or across channel boundaries. RFC-0027 must either (a) declare that
within-scope RAII order is sufficient and document the cross-scope case as explicitly unspecified,
or (b) impose an ordering obligation (e.g., sweep order) on cross-scope drops.

*Sources: [Wikipedia: RAII](https://en.wikipedia.org/wiki/Resource_acquisition_is_initialization);
[C++ Destructors and RAII — hacking C++](https://hackingcpp.com/cpp/lang/destructors)*

### 3.3 Linear types as resource management (Rust, Linear Haskell, Dafny)

*Confidence: `Empirical` — Rust is in production; Linear Haskell is published research.*

Linear types enforce "use exactly once" — a linear value cannot be duplicated or silently dropped.
This gives the compiler a static proof that reclamation happens *exactly at the use site*, with no
possible leak or double-free. The "audit record" under linear types is implicit: the type system
itself is the proof, and the proof is the program.

**Limitation for cross-boundary audit:** linear types track ownership but not *provenance history*.
A value reclaimed after crossing a channel boundary carries no record of *where* it came from
unless provenance is threaded through the `Meta` type. This is the gap RFC-0005 EXPLAIN partially
fills: the `Meta.policy_used` field records *how* a value was selected/transformed, but there is no
`Meta.reclamation_provenance` field in the current corpus.

**Relevance:** Mycelium's LR-8/LR-9 (immutable + acyclic) give a weaker but sufficient
reclamation property: values are not aliased, so reclamation has no aliasing hazard. Linear types
would give a *stronger* proof (exactly-once), but Mycelium's immutability already rules out the
pathological aliasing case. The remaining provenance requirement is: *when* a value leaves scope
(at scope-exit or channel-close), *what scope* held it last, and *which `Meta`* it carried.

*Sources: [Linear Types for Large-Scale Systems Verification — CMU](https://www.andrew.cmu.edu/user/bparno/papers/linear-dafny.pdf);
[Safe memory management in inline-java using linear types — Tweag](https://www.tweag.io/blog/2020-02-06-safe-inline-java/);
[The Usability of Advanced Type Systems — arXiv](https://arxiv.org/pdf/2301.02308)*

### 3.4 Pony ORCA: per-actor GC without stop-the-world

*Confidence: `Empirical` — Pony's ORCA is a shipped production GC.*

Pony's ORCA GC uses per-actor heaps so that each actor collects independently, with no
stop-the-world step. Cross-actor references are tracked via a deferred, distributed weighted
reference count; a cycle-detector actor handles cyclic groups of blocked actors.

**Key protocol insight:** when actor A sends a value to actor B, ORCA increments a reference count
on B's behalf *before* the message is delivered — this is the "deferred" aspect: reclamation waits
until both the send-side and the receive-side agree the reference is gone. The **message-passing
order** (which Pony's actor mailbox enforces) is the synchronization primitive that makes this
safe.

**Relevance to RFC-0027:** Mycelium's channel close protocol (`TrySend::Disconnected`,
`TryRecv::Closed`) already plays the role of Pony's deferred reference count update: the channel
is the synchronization boundary. The reclamation of a value sent over a channel is safe *after*
the receiving scope acknowledges the channel is closed (drained to `Closed`). This gives a natural
reclamation epoch: **the channel-close event is the reclamation synchronization point**.

The critical advantage Mycelium has over Pony ORCA: because Mycelium values are **acyclic by
LR-9**, there are no cross-scope cycles to detect. The cycle detector — ORCA's most complex
component — is eliminated. Per-scope reclamation with a channel-close synchronization point is
sufficient.

*Sources: [Pony ORCA Tutorial](https://tutorial.ponylang.io/appendices/garbage-collection.html);
[ORCA paper — janvitek.org](http://janvitek.org/pubs/oopsla17a.pdf);
[Ownership and Reference Counting based GC in the Actor World — labri.fr](https://www.labri.fr/perso/fmoranda/icooolps15/p2.pdf)*

### 3.5 Distributed GC and weighted reference counting

*Confidence: `Empirical` — published research (Birrell et al. Network Objects, 1993).*

Distributed GC must handle: messages in transit, network partitions, and the absence of a global
synchronization point. Weighted reference counting (Bevan 1987; Watson & Watson 1987) addresses
this by assigning weights to references and tracking their sum. A reference can be deleted only
when all outstanding weights are accounted for.

**Key negative result for RFC-0027:** distributed GC requires coordination across process
boundaries to confirm reclamation. In Mycelium's R1 (single-node) scope, this is not needed — the
structured scope tree provides a total order (RT7) that is the synchronization mechanism. The
distributed case (R2: `xloc`, `mesh`) is explicitly out of scope for RFC-0027 §3.

**Relevance:** RFC-0027 can and should explicitly limit its reclamation model to the single-node
scope. The EXPLAIN record for a cross-hypha value does *not* need distributed GC overhead at R1
— it only needs: scope ID, channel ID, sweep epoch at close. Distributed reclamation provenance
is an R2 obligation, not an R1 one.

*Sources: [Distributed Garbage Collection for Network Objects — ResearchGate](https://www.researchgate.net/publication/2342871_Distributed_Garbage_Collection_for_Network_Objects);
[Reference counting — Wikipedia](https://en.wikipedia.org/wiki/Reference_counting);
[Indirect Reference Counting — Springer](https://link.springer.com/chapter/10.1007/978-3-662-25209-3_11)*

### 3.6 CRDT tombstone GC and causal reclamation

*Confidence: `Empirical` — well-studied in distributed systems literature.*

CRDT tombstone GC is the hardest case of reclamation under convergent state: a deleted item must
be represented as a tombstone until every replica has acknowledged the deletion, requiring causal
tracking. The key insight is that tombstone GC requires a **coordination step** — it cannot be
done purely locally, because a replica that hasn't seen the deletion would confuse a missing entry
with a deleted one.

**Relevance:** RFC-0008 RT6 requires lawful merge (semilattice join) for `fuse` operations.
However, RFC-0027 is scoped to single-node reclamation (§3), and Mycelium values are immutable
(LR-8) — there are no tombstones or CRDTs in the reclamation model per se. The CRDT tombstone
problem *would* arise if RFC-0027 attempted to define reclamation for `fuse` state across `mesh`
(R2), but that is explicitly deferred. **The tombstone GC problem is out of scope for RFC-0027 R1;
the report flags it as an R2 open question.**

*Sources: [Implementing a GC'd Graph CRDT — decomposition.al](https://decomposition.al/CMPS290S-2018-09/2018/12/08/implementing-a-garbage-collected-graph-crdt-part-2-of-2.html);
[CRDT Glossary — crdt.tech](https://crdt.tech/glossary)*

### 3.7 Structured concurrency and scope-exit cleanup (Swift, Kotlin, Trio)

*Confidence: `Empirical` — all three are shipped in production.*

Swift's structured concurrency SE-0304 (`TaskGroup`), Kotlin's coroutine scopes, and Python Trio
nurseries all enforce: child tasks cannot outlive their scope; cancellation propagates downward;
resources are released at scope exit in a well-defined order. Swift guarantees that `defer` blocks
and actor deinitializers run before a scope exits.

**Key finding:** structured concurrency *already provides* a natural reclamation ordering — the
scope-exit cleanup is the reclamation event. The ordering question (total vs partial) resolves as:
**total within a scope's children, partial across sibling scopes**. No mainstream structured
concurrency runtime enforces a total order across sibling tasks' finalizers — they are concurrent
by design.

**Relevance:** RFC-0008's `SweepOrder` is a *scheduling* order, not a *reclamation* order.
These can be the same (strong coupling, RFC-0027 §5 Q2) or different. Strong coupling would
mean: reclamation of a scope's values follows the same ascending/descending sweep order as the
Kahn-determinism differential. This is the property-testable path. Weak coupling allows the runtime
to reclaim values in any order provided the scope-exit invariant (RT7) holds.

*Sources: [Swift Structured Concurrency — GitHub swift-evolution](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0304-structured-concurrency.md);
[Structured Concurrency — Grokipedia](https://grokipedia.com/page/Structured_concurrency)*

### 3.8 Epoch-based and hazard-pointer reclamation

*Confidence: `Empirical` — used in production concurrent data structures (crossbeam-epoch, etc.).*

Epoch-based reclamation (Fraser 2004; Harris 2001 hazard pointers) defers reclamation until all
threads have passed through a quiescent epoch — guaranteeing no thread holds a pointer to the
object being reclaimed. The "audit record" in epoch-based systems is the epoch number at which
reclamation was deferred and at which it was confirmed safe.

**Relevance:** RFC-0008's `SweepOrder` has the structure of an epoch — a monotonically advancing
counter the scheduler increments per scheduling step. If RFC-0027 adopts sweep-epoch as the
reclamation epoch, it gets epoch-based reclamation's safety property *for free* from the existing
scheduler infrastructure: reclaim a value only after the sweep epoch in which its owning scope
exited has passed. The EXPLAIN record then contains: (scope_id, channel_id, sweep_epoch_at_close,
value_meta_hash). This is a tight, auditable, EXPLAIN-able record.

*Sources: [A marriage of pointer- and epoch-based reclamation — ACM PLDI 2020](https://dl.acm.org/doi/abs/10.1145/3385412.3385978);
[Verifying Concurrent Memory Reclamation — ESOP 2013](https://www.cs.ox.ac.uk/people/hongseok.yang/paper/esop13-full.pdf)*

---

## 4. Concrete Requirements for RFC-0027

The following requirements are `Declared` (derived from corpus analysis + prior art). They are
inputs for the r10 kickoff to evaluate and promote to normative decisions.

### R-1: Reclamation EXPLAIN record minimum field set

Every reclamation event (scope-exit, channel-close, `cyst`-and-free) MUST be observable as a
structured record containing at minimum:

| Field | Type | Rationale |
|---|---|---|
| `scope_id` | stable scope identifier | RT7 — identifies which scope's exit triggered reclamation |
| `sweep_epoch` | monotonic counter from `SweepOrder` | epoch-based safety; ties reclamation to the scheduling model |
| `trigger` | enum: `ScopeExit \| ChannelClose \| CystCheckpoint` | G2 — never silent; audit knows *why* reclamation happened |
| `value_meta_hash` | content hash of the value's `Meta` | provenance chain — ties the reclamation event to the value's guarantee history |
| `channel_id` | optional; present for `ChannelClose` trigger | identifies which channel boundary the value crossed |

This record is an EXPLAIN output, not a runtime invariant maintained per-value. It is emitted
*once*, at reclamation time, to the supervision policy's observability sink (RFC-0013 §8 route).
`Declared` tag: this field set is derived from the honesty rule (G2), the sweep-order model
(RFC-0008 §4.3), and the EXPLAIN contract (RFC-0005); it is not yet `Empirical` (no property test
exists) and not `Proven` (no mechanized proof).

### R-2: Reclamation order — partial order over siblings, total order within a scope

RFC-0027 SHOULD adopt the **partial-order model** for cross-scope reclamation:

- **Within a scope:** reclamation of a scope's owned values follows Rust drop order (reverse
  construction, within the scope's task) — `Exact` by Rust's specification.
- **Across sibling scopes:** no total order is required or imposed. Sibling scopes in RT7 are
  concurrent by design; imposing a total reclamation order across them would serialize concurrent
  cleanup (a performance cost with no safety benefit, since LR-9 ensures no cross-sibling aliases).
- **Across parent-child scopes:** reclamation follows RT7 — children fully reclaim before the
  parent exits. This *is* a total order along the scope-tree path from child to root.

This partial-order model is auditable: the EXPLAIN record (R-1) captures the scope and epoch,
so a post-hoc reconstruction of the reclamation order is always possible. The property test for
RFC-0027 §4 DoD should verify: for any two scopes at the same tree level, neither scope's
reclamation record appears in the other scope's sweep epoch — i.e., sibling reclamation epochs
are non-overlapping or unordered.

The **strong-coupling option** (RFC-0027 §5 Q2: require reclamation to follow `SweepOrder` across
all scopes) is property-testable but imposes a total order that the prior art does not support as
necessary for safety (section 3.7 above). It is flagged here as a viable alternative with higher
auditability cost.

### R-3: `reclaim` surface construct scope — task supervision only, memory via EXPLAIN sink

Per RFC-0008 §4.5 and DN-03 §4, `reclaim` is a task/runtime-unit supervision primitive.
RFC-0027 MUST NOT define `reclaim` as a memory primitive. Instead:

- **Memory-level reclamation** is Rust drop order, exposed to the supervision policy via the
  observability sink (RFC-0013 §8) through the EXPLAIN record defined in R-1.
- **Task-level `reclaim`** (supervision) triggers scope-exit, which in turn triggers
  memory-level reclamation. The causal chain is: `reclaim(scope)` → scope-exit → RT7 child
  join → Rust drops → EXPLAIN record emitted.
- This two-level model keeps KC-3 (small auditable kernel): the memory primitives are Rust's own,
  auditable at the Rust level; the Mycelium EXPLAIN layer sits above without duplicating the
  reclamation mechanism.

### R-4: Channel-close is the cross-hypha reclamation synchronization point

When a value is sent over a channel and the channel closes:

- **`TrySend::Disconnected(v)`:** the sender recovers ownership of `v` and is responsible for its
  reclamation (either by dropping it or retaining it in the sender's scope). The EXPLAIN record
  for this event: trigger = `ChannelClose`, scope_id = sender's scope, channel_id, epoch.
- **`TryRecv::Closed` (after drain):** the receiver has consumed all values; the channel buffer is
  empty; the backing allocation is reclaimed by Rust drop when the `Receiver<V>` is dropped.
  EXPLAIN record: trigger = `ChannelClose`, scope_id = receiver's scope.

This protocol **requires no distributed reference counting** at R1 (single-node). The channel's
`Sender`/`Receiver` pair (affine by construction — neither is `Clone`) already ensures exactly one
owner at any time. Reclamation is sound because ownership is exclusive.

### R-5: `cyst` checkpoint interaction — checkpoint-and-free is the preferred model

`Declared` preference (not yet decided):

- **Checkpoint-and-free:** the `cyst` checkpoint serializes the value to content-addressed storage,
  then the original allocation is freed. The value's identity is its content hash (ADR-003), so
  the content-addressed artifact IS the value — the original allocation is not the identity.
  This is memory-efficient and consistent with Mycelium's value-semantics model.
- **Checkpoint-and-keep:** the original allocation survives alongside the checkpoint artifact.
  Safer against bugs in the checkpoint serializer, but duplicates memory. Appropriate as a
  transitional default until the checkpoint serializer has `Empirical` evidence of correctness.

**Recommendation:** adopt checkpoint-and-keep as the R1 v0 default (safer, `Declared` correctness
of the serializer is insufficient justification for immediate free), with checkpoint-and-free as
a future `reclaim` policy variant gated on an `Empirical`-tagged checkpoint property test.
EXPLAIN record for checkpoint: trigger = `CystCheckpoint`, value_meta_hash, cyst artifact hash.

### R-6: "No silent GC pause" means bounded, logged, never-silent — not zero-latency

RFC-0027 §1 states: "the only acceptable pause is a bounded, logged, never-silent reclamation
event." This must be codified precisely:

- The guarantee tag for reclamation latency is `Declared` (asserted) at R1, not `Empirical`.
  To become `Empirical`, a property test measuring reclamation latency distribution must exist
  in-repo (RFC-0027 §4 DoD item 4).
- "Bounded" means: a worst-case budget is declared (even if `Declared`-strength). A reclamation
  event that exceeds the declared budget is a reclamation *failure*, escalated through the
  supervision tree (RFC-0008 §4.7 C4 bounded-cascade escalation).
- "Logged" means: the EXPLAIN record (R-1) is emitted. Failing to emit the record is a G2
  violation — exactly as dropping a value silently is.

### R-7: Interior mutability and shared-mutable patterns are eliminated by LR-8/LR-9

LR-8 (immutable values) and LR-9 (acyclic values) together eliminate the hardest reclamation
problems:

- **No interior mutability:** a Mycelium value has no `RefCell`/`RwLock`/`UnsafeCell` analog at
  the *language* level (the Rust implementation may use `RefCell` internally, but that is not
  visible at the Mycelium value level). Therefore, there is no "who holds the mutation lock when
  this value is dropped?" question at the Mycelium level.
- **No shared mutable state:** RT1 (values move, state is never shared) means there are no shared
  references to reclaim — only owned values transferred via channels. Reclamation is always from a
  single owner.
- **Acyclic values:** no cycle detection is needed (RFC-0027 §3 confirms this). Reference counting
  without a cycle-breaking GC (as Pony ORCA requires a cycle-detector actor) is sufficient.

The **one remaining open case** is `substrate` handles (LR-8 / `graft` vocabulary, RFC-0008 §4.5).
`substrate` is described as an affine handle to external infrastructure — affinity ensures
single-use, but the `graft` construct is reserved-not-active. RFC-0027 SHOULD explicitly defer
`substrate` reclamation to the `graft` implementation RFC and not attempt to cover it here.

### R-8: The EXPLAIN contract extends the RFC-0005 one-mechanism pattern

RFC-0005 defines EXPLAIN as mandatory for every automatic selection: `{inputs considered, cost of
each candidate, chosen option, deterministic override hook}`. The reclamation EXPLAIN record (R-1)
is a *reclamation-site* extension of this contract:

- `inputs considered` → which scope/channel triggered reclamation
- `chosen option` → the reclamation action taken (drop, checkpoint-and-free, return-to-sender)
- `sweep_epoch` → the deterministic audit anchor

The reclamation EXPLAIN record SHOULD be emitted to the same observability sink as other EXPLAIN
records (RFC-0013 §8 route). This avoids a second mechanism (KC-3/DRY).

---

## 5. Open Research Questions

These are **open** — the report does not resolve them. They are inputs for the RFC-0027
normative decision process.

### OQ-1: Weak vs strong sweep-order coupling (RFC-0027 §5 Q2, restated)

The partial-order model (R-2 above) is the minimal safe model. The strong-coupling model
(reclamation strictly follows `SweepOrder`) is maximally auditable. The tradeoff:

- **Strong coupling:** reclamation order is *identical* to scheduling order → one property test
  covers both. Cost: cross-scope drops are serialized (sequential cleanup of siblings).
- **Weak coupling:** sibling scopes reclaim concurrently → better throughput. Cost: the property
  test for reclamation order is separate from (and weaker than) the Kahn-determinism differential.

Neither the corpus nor the prior art surveyed definitively resolves this. It is a design decision
with real tradeoffs. The r10 kickoff should prototype both and measure the property-test surface.
`Declared` preference: weak coupling (R-2), but this is the highest-uncertainty decision in
RFC-0027 and warrants empirical investigation.

### OQ-2: `reclaim` surface typing and elaboration

ADR-020 §consequences item 5 defers the `reclaim` surface construct: "the `reclaim` *surface
construct* (L1 typing + elaboration) is gated on its own implementation RFC." RFC-0027 must decide
whether it *is* that implementation RFC or whether it remains a model-only document leaving the
surface RFC for later. The distinction matters for the DoD: if RFC-0027 specifies typing, it
becomes the activation RFC for `reclaim` (requiring a Rust implementation); if it only models the
memory reclamation semantics, `reclaim` stays reserved.

Recommendation (`Declared`): RFC-0027 should specify the *memory model* (what reclamation means,
what EXPLAIN contains) but leave the *surface construct typing* to a follow-on RFC. This keeps
RFC-0027's scope tight (KC-3) and lets the memory model be ratified before the surface shape is
locked in.

### OQ-3: Worst-case reclamation latency bound

RFC-0027 §5 Q4 asks: hard real-time bound or best-effort? The prior art (ORCA, Rust drop,
structured-concurrency cleanup) does not provide a general worst-case bound — it depends on the
depth of the scope tree and the size of values. At R1, the reclamation cascade depth is bounded
by the scope-tree depth, which is bounded by the task's fuel budget (RFC-0014 §4.8). This may
give an implicit latency bound derivable from the fuel model, but the derivation is not trivial.

This is open research: deriving a reclamation latency bound from the fuel model would allow the
latency tag to advance from `Declared` to `Empirical` (property-tested) or even `Proven` (if the
derivation is mechanized). This is a tractable formal methods problem and warrants a dedicated
research note.

### OQ-4: R2 reclamation provenance — deferred but explicitly flagged

When `xloc` and `mesh` land (RFC-0008 §4.6 R2), reclamation provenance becomes a distributed
problem: a value may cross node boundaries and the reclaiming scope may not be the creating scope.
The CRDT tombstone GC problem (section 3.6) and weighted reference counting (section 3.5) become
relevant. RFC-0027 MUST explicitly disclaim R2 reclamation in §3 (it already does implicitly) and
flag OQ-4 as a named open question so the R2 reclamation RFC has a clear handoff point.

### OQ-5: `substrate` / `graft` reclamation

`substrate` handles are affine (LR-8) — they can be consumed exactly once. But what is the
reclamation protocol when an affine `substrate` is dropped rather than consumed? Is that a runtime
error, a silent no-op, or an explicit EXPLAIN event? This is out of scope for RFC-0027 (it depends
on the `graft` implementation RFC), but it SHOULD be explicitly flagged as deferred in RFC-0027 §3
to prevent a future `graft` RFC from silently contradicting the reclamation model.

---

## 6. Summary for r10 Kickoff

**Highest-value finding:** Mycelium's LR-8/LR-9 (immutable + acyclic values) eliminate cycle
detection and cross-alias hazards, reducing the reclamation problem to **scope-exit + channel-close
synchronization**, which the existing RT7 scope tree and channel close protocol already provide.
The EXPLAIN record is the main new artifact RFC-0027 must define, and its minimum field set
(R-1 above: `scope_id`, `sweep_epoch`, `trigger`, `value_meta_hash`, `channel_id?`) is tight and
auditable.

**Second-highest finding:** the sweep-order / reclamation-order coupling question (OQ-1) is the
most consequential undecided question. The partial-order model (R-2) is the minimal safe default;
the strong-coupling model is maximally auditable but serializes sibling cleanup. The r10 kickoff
should prototype the property-test surface for each option before committing.

**Third finding:** the `cyst` checkpoint-and-keep default (R-5) is the conservative safe choice
at R1. Advancing to checkpoint-and-free requires an `Empirical`-strength checkpoint property test,
which does not yet exist in-repo.

---

## Meta — changelog

- **2026-06-24 — Created.** Lane B research handoff for RFC-0027 reclamation-provenance. Grounded
  in RFC-0027 §5 (open questions), RFC-0008 §4.1/§4.3/§4.4/§4.5/§4.7, DN-03 §4, ADR-020
  §consequences, LR-8/LR-9, G2, VR-5, KC-3, RFC-0005. External prior art: Tofte-Talpin region
  inference, RAII/Rust drop, Pony ORCA, distributed GC/weighted ref-counting, CRDT tombstone GC,
  structured concurrency cleanup (Swift/Kotlin/Trio), epoch-based reclamation. Status: `Declared`
  throughout — decision-grade inputs, not normative decisions. No normative content of any RFC
  is altered. Append-only.
