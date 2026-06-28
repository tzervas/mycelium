# Design Note DN-61 — Concurrency & Distribution Runtime Semantics (R1 normativity + R2 agenda)

| Field | Value |
|---|---|
| **Note** | DN-61 |
| **Status** | **Accepted** (Part A — R1 scheduler normativity; 2026-06-28, in-session ratification) · Part B remains **Draft** (open research agenda) — **Part A (R1 scheduler normativity) RATIFIED by maintainer 2026-06-28**: RT2 + fuel-compatible cooperative stepping + RT7 normative commitment accepted. Part B (R2 distributed agenda: clocks R8-Q3, Byzantine R8-Q4, distributed reclamation RFC-0027 OQ-2, network-xloc, concurrent fuse-merge) stays **Draft** — needs dedicated research passes before it is ratifiable. The A/B split is explicit: Part A is the near-term implementation gate; Part B is the bounded forward agenda for a dedicated R2 RFC. All tags stay `Declared` (VR-5). |
| **Feeds** | **RFC-0008** (the normative R1 runtime — scheduler-normativity question R8-Q1); a future **R2 distributed-execution RFC** (closes R8-Q3/Q4 + RFC-0027 OQ-2 + RFC-0028 §7 + RFC-0023 R23-Q2); **DN-58** (fuse surface — the `fuse` semilattice merge ties concurrent-session-merge to the G1 surface). |
| **Date** | June 28, 2026 |
| **Decides** | *Proposes, for ratification:* **(Part A)** how much of the R1 single-node scheduler is normative within RFC-0008's RT1–RT7 frame — the scheduler-normativity question (R8-Q1). **(Part B)** captures the open distributed-execution questions (R8-Q3, R8-Q4, RFC-0027 OQ-2, RFC-0028 §7 `xloc`, RFC-0023 R23-Q2) as a bounded forward agenda for a dedicated R2 RFC. Part B does **not** resolve these questions; it names what each needs. |

> **Posture (transparency rule / VR-5 / G2).** Part A is tagged `Declared`-with-argument: the
> scheduler-normativity proposal is ratifiable from RFC-0008's existing Accepted frame, but no
> in-repo mechanized proof or scheduler implementation exists yet (the v0 fork/join executor
> and Kahn-deterministic channels landed in M-357; the full scheduler is the next R1 slice).
> Part B items are tagged `Declared` agenda-only: each is an *honest acknowledgment of an open
> question* that needs its own research pass, not a resolution dressed as one. Downgrading is
> always safe; upgrading requires a checked basis (VR-5). No status of any other decision moves
> (house rule #3 / append-only).

---

## Scope and motivation

RFC-0008 (Accepted 2026-06-16) defines RT1–RT7, the normative runtime invariants, and stages
the runtime build into R0 (the model), R1 (single-node), and R2 (distribution). The G8 ratification
group in the Blocked-Decisions map identifies two distinct axes requiring design capture before
the R1 implementation RFC and the R2 RFC can launch cleanly:

1. **R1 axis** — "How much of the R1 scheduler is normative?" (R8-Q1). This is ratifiable now
   because RFC-0008's RT1–RT7 already supply the invariants the answer depends on; the question
   is only what the implementation RFC is committed to.
2. **R2 axis** — Byzantine tolerance (R8-Q4), time/clocks/deadlines (R8-Q3), distributed
   reclamation provenance (RFC-0027 OQ-2), network-FFI / `xloc` capability composition
   (RFC-0028 §7), and concurrent session-merge (RFC-0023 R23-Q2). None of these is ratifiable
   without a dedicated research pass; pretending otherwise would upgrade a `Declared` agenda to
   a `Proven` resolution — a VR-5 violation.

The cross-link between the two parts: concurrent-session-merge (Part B item 5) ties
operationally to the `fuse` surface (DN-58 Group A/B), which is in the G1 ratification track
and must land before R23-Q2 can close. Distributed-reclamation provenance (Part B item 2) is
the R2 twin of DN-59's R1 cross-hypha reclamation work.

---

## Part A — Scheduler normativity (R1, near-term) `Declared`

**Tag: `Declared`-with-argument** — RFC-0008 §4.6 and the landed M-357 implementation supply
the basis; the full scheduler is not yet in-repo, so this cannot yet be `Empirical` or
`Proven`. The argument is grounded in RFC-0008 RT2 and the landed differential evidence.

### A.1 The question

RFC-0008 §8 R8-Q1 defers: "How much of the R1 scheduler is normative? Candidate: only the RT2
determinism obligation + fuel-compatible cooperative stepping; everything else is implementation."
This note proposes an answer for ratification.

### A.2 Proposed resolution

**The normative commitment of the R1 scheduler is exactly RT2 and the three structural
properties that make RT2 verifiable. Everything else is an implementation artifact.**

Concretely, the implementation RFC for the R1 scheduler is committed to, and only to:

1. **RT2 determinism obligation** (`Declared`): the concurrent observable of any program in the
   deterministic fragment must be equivalent to its sequential reference execution (`run_sequential`),
   validated by the M-357 sequentialization differential. This is the only observable-level
   commitment — schedulers that produce different observables in the deterministic fragment are
   non-conformant, independent of their internal strategy.

2. **Fuel-compatible cooperative stepping** (`Declared`): a task may only be suspended at its
   budget-check / fuel-check points (RFC-0014 §4, M-353 C2). This is the minimal safety
   condition that allows per-task budgets (C1) and cooperative cancellation (C2) to function
   correctly. Preemptive suspension at an arbitrary instruction boundary would break C2's
   never-drop-an-in-flight-explicit-outcome invariant.

3. **RT7 structured scope discipline** (`Declared`): the scheduler must honor the RT7 no-orphan
   invariant — a parent scope must not exit until all its children have completed, been cancelled,
   or been explicitly detached. The v0 `Scope`/`Task` structure in `mycelium-mlir::runtime`
   enforces this structurally; the implementation RFC must preserve it.

**What is explicitly not normative:**

- **Scheduling strategy within the deterministic fragment** — round-robin, work-stealing, FIFO,
  priority-ordered, randomized: all conformant provided they satisfy (1)–(3). The M-357 baseline
  uses round-robin (`SweepOrder::Ascending`/`Descending`) and shows both produce the same
  observable, which is the differential check (not a policy preference).
- **Parallelism** — parallel task execution (multi-threaded) is an RT2-validated performance
  path, not a separate mode. The trusted reference stays sequential; a parallel implementation
  is conformant iff it passes the same RT2 differential over the same corpus. This is exactly
  the interpreter/AOT relationship (RFC-0004 NFR-7) applied to scheduling.
- **Placement** (RT3 / `forage`) — where work runs is a reified RFC-0005 policy with EXPLAIN,
  semantics-free by RT3. The scheduler is not a placement mechanism; `forage` is.
- **Nondeterministic constructs** (RT3) — multi-source `select`/`merge`, racing, adaptive
  placement: these are RT3 constructs with named policies, not scheduler policy. The scheduler
  delivers the fairness condition these policies require (each ready source gets a turn), but
  the *arbitration* is a content-addressed policy, not a scheduling heuristic.
- **Time** — logical clocks (monotonic counters for supervisor restart intensity, M-356 C4) are
  normative for the R1 supervision model; wall-clock / physical / hybrid clocks are R8-Q3
  (Part B below) and not a scheduler commitment.

### A.3 Why this is the right cut

The cut is derived from RFC-0008's own design decisions, not invented here:

- RT2's sequential reference semantics already says the scheduler is *outside* the trusted base
  (KC-3): concurrency "adds scheduling outside the kernel, never new meaning inside it." The
  normative commitment should match this: the scheduler is accountable to the observable
  (RT2), not to a particular strategy.
- The v0 M-357 differential already demonstrates the approach in-repo: two distinct fair
  schedules produce equal observables. The normative commitment is the *equality*, and the
  differential is the enforcement mechanism — not the strategy.
- Fuel-compatible cooperative stepping is already enforced by the M-356 C2 mechanism. This is
  not a new commitment; it is naming an existing structural invariant as normative.

### A.4 The RT2 differential as the conformance gate

The implementation RFC must specify the conformance obligation concretely:

> **R1 scheduler conformance:** given any program in the deterministic fragment (structured
> fork/join or Kahn-deterministic channel network), running it under the candidate scheduler
> and running it under `run_sequential` on the same inputs must produce the same per-task
> `TaskOutcome` values and the same channel transcripts.

This is `Empirical` as a differential test (the M-357 evidence tags it this way: "tagged
`Empirical` — the differential is the evidence"), not `Proven` (no mechanized proof of Kahn
determinism is in-repo). The implementation RFC may not upgrade this to `Proven` without a
checked mechanized proof with its side-conditions verified (VR-5).

### A.5 Parallel execution as a performance path

Parallel (multi-threaded) execution of the deterministic fragment is ratifiable as a
conformant performance path under the same conformance gate (A.4). This mirrors RFC-0004's
structure: the trusted interpreter is the reference, MLIR/AOT is the performance path,
NFR-7 is the equivalence obligation. The implementation RFC should:

1. Confirm the `Send + Sync` requirements for values crossing thread boundaries are discharged
   by LR-9 (immutable, acyclic values) and RT1 (no shared mutable state).
2. Specify the RT2 differential test extended to parallel schedules — the same observable
   requirement, now also across parallel schedules.
3. Tag the parallel path's equivalence claim `Empirical` (differential-based) pending a
   mechanized proof.

This does not require any change to RFC-0008 or the trusted base — the kernel stays
single-threaded and sequential. The note is that the *scheduler* is a parallel runtime
around a per-task sequential execution model.

### A.6 What this unblocks (R1)

- The R1 implementation RFC can now specify its normativity precisely without re-opening
  RFC-0008.
- RT3 nondeterministic constructs (`select`/`merge` — M-668 deferred) can be specified
  knowing that the scheduler's fairness obligation is separate from arbitration policy.
- The `reclaim` bounded-cascade supervisor (M-356 C4) retains its logical-clock basis; the
  upgrade to physical/hybrid time waits for R8-Q3 (Part B).

---

## Part B — Distributed-execution agenda (R2) `Declared`

**Tag: `Declared` agenda-only.** These are five clusters of open questions that collectively
define the scope of the R2 distributed-execution RFC. Each is plainly stated as an open
question requiring its own research pass. None is resolved here. Stating the agenda is itself
a design contribution (G2 / VR-5): it makes the gap visible and its scope bounded, so the R2
RFC can launch with clean preconditions rather than discovering its scope mid-flight.

### B.1 Time, clocks, and deadlines (R8-Q3) — needs research

**Open question (RFC-0008 §8 R8-Q3):** How are wall-clock time, timeouts, and deadlines
reified in a system that is otherwise deterministic and honest?

**What is already decided (R1):** The R1 supervision model (M-356 C4) uses a *logical clock*
— a deterministic monotonic counter the supervisor advances — for its max-restart-intensity
bound. This is intentionally not wall-clock time: the R8-Q3 deferral was explicit in M-356
("physical/hybrid clocks for real time are R8-Q3 deferred").

**Why this needs a research pass:** Clocks introduce nondeterminism into an otherwise
deterministic model. The RT3 frame (RFC-0008 §4.1 RT3) says nondeterministic inputs must be
reified and explained — time reads are the canonical case. The candidate ("time reads are
explicit effects with declared resolution; logical/hybrid clocks for mesh ordering") is a
design direction, not a researched decision. The questions the R2 RFC must answer include:

- What clock types are exposed (logical / physical / hybrid Lamport-style)? Each has
  different honesty properties: physical clocks are `Empirical` (NTP-bounded, not
  `Exact`); hybrid clocks (HLC) give causal ordering with physical bounds, but the
  precision guarantee depends on deployment-specific NTP drift, making it `Declared` at
  design time.
- Are time reads an explicit effect in the RFC-0014 effect row, or a separate mechanism?
  (This interacts with RFC-0008 R8-Q2 / RFC-0014 §4.8 — the hypha-in-effect-row
  question, which is itself a G6 item.)
- How do deadlines interact with structured concurrency (RT7)? A deadline-expired task
  must produce an explicit outcome (RT4, I1) — never silently dropped.

**Prerequisite:** a research pass over HLC/TrueTime/GPS-synchronized clocks and their
guarantee-lattice mappings. The R2 RFC owns this.

### B.2 Byzantine tolerance and consensus (R8-Q4) — needs research

**Open question (RFC-0008 §8 R8-Q4):** Where does Mycelium's honest-guarantee model meet
adversarial participants? The v0 mesh trusts its members; hostile-member hardening is
explicitly deferred.

**What is already decided:** RFC-0008 §4.3 states plainly: "Byzantine participants are out of
scope of v0 (R8-Q4): the mesh trusts its members; hostile-member hardening is its own future
RFC." RFC-0006 §4.1 RT5 establishes that failure-detector verdicts are *suspicion with
confidence*, never `Exact` — and FLP makes crash-vs-slow undecidable in an asynchronous
system. This means Byzantine tolerance, if it lands, must be tagged accordingly.

**Why this needs a research pass:** Byzantine fault tolerance (BFT) and consensus
(Paxos/Raft/PBFT/BFT-CRDT hybrids) have fundamentally different cost profiles and guarantee
structures. RFC-0008 §6 deliberately chose semilattice merge over consensus for the default
case — "consensus buys linearizability at the cost of availability and of a global
coordination dependency; semilattice merge buys deterministic convergence with local
decisions." BFT requires additional assumptions (2f+1 replicas for f failures, etc.) and
its guarantee is `Proven` only under those assumptions. The questions the R2 RFC must answer:

- What threat model is in scope? Crash faults only (tolerant of up to f node failures under
  BFT assumptions), or active Byzantine behavior?
- Where does consensus land relative to semilattice merge? RFC-0008 §6 positions consensus
  as a future explicit construct "where truly needed, with its own honest costs" — the R2
  RFC must specify which constructs carry consensus guarantees and at what honest cost.
- How are BFT guarantees tagged on the lattice? A BFT-backed operation may earn `Proven`
  under the checked side-conditions (2f+1, synchrony/asynchrony assumption), but the
  deployment-specific verification of those conditions must also be in scope.

**Prerequisite:** a research pass over BFT-CRDT / BFT-consensus trade-off literature, and
a decision on whether the v0 mesh abstraction is a viable target for BFT or requires a
separate mesh variant. The R2 RFC owns this.

### B.3 Distributed reclamation provenance (RFC-0027 OQ-2) — needs research

**Open question (RFC-0027 §11 OQ-2):** When `xloc`/`mesh` land (R2), a value may cross node
boundaries and the reclaiming scope may not be the creating scope. This is CRDT-tombstone GC
and weighted reference counting territory — explicitly out of scope for the R1 reclamation
model.

**What is already decided (R1):** RFC-0027 (Accepted 2026-06-25) decides reclamation for
single-owner, intra-hypha values. RC is the mechanism; cross-hypha transfer at R1 rides the
affine channel protocol (affine `Sender`/`Receiver` — no distributed refcount needed at the
single-node level because the channel's close protocol gives exactly-one-owner transfer for
free). RFC-0027 §12 names an explicit open sub-question: when a value crosses a hypha
boundary, is it a sole-ownership move (Option A: affine move, §7.3 as-is) or may a
shared value cross (Option B: atomic RC engages)? This sub-question is "not decided here" and
is "tabled for the RFC-0027 follow-on."

**Relationship to DN-59:** DN-59 (the G3 R1 cross-hypha reclamation note) addresses the
cross-hypha sharing protocol at R1 — the Option A / Option B decision. DN-61 Part B item 3
is the R2 twin: once values cross *node* boundaries (`xloc`), the R1 affine-move guarantee
no longer holds in the same way, because the reclaiming scope may be on a different node.
The R2 RFC inherits whichever option DN-59 / the RFC-0027 follow-on chooses as its starting
point, then extends it to distributed reclamation.

**Why this needs a research pass:** Distributed GC is a well-studied problem with no
universally cheap solution: weighted reference counting (Bevan 1987; Watson & Watson 1987)
avoids cyclic scanning but requires reliable message delivery and tombstone bookkeeping.
CRDT-tombstone GC (Shapiro et al.) gives provable convergence but requires tombstone storage
and a garbage-collection pass to prune them. The Mycelium constraint (LR-9: no cycles in the
value graph) eliminates some complexity (no cycle detector needed, as RFC-0027 §7.3 notes),
but cross-node ownership still requires a protocol. The questions the R2 RFC must answer:

- Given DN-59/RFC-0027 follow-on's Option A/B decision: how does the ownership protocol
  extend to cross-node transfers?
- What is the provenance EXPLAIN-record schema for a value reclaimed on a remote node? The
  RFC-0027 §9 schema (`scope_id`, `sweep_epoch`, `trigger`, `value_meta_hash`, optional
  `channel_id`) must gain a `node_id` and a causal-delivery record.
- What delivery guarantees does the reclamation protocol require from the mesh layer (B.1's
  clocks and RT5's probabilistic delivery bounds)?

**Prerequisite:** a research pass on distributed GC for acyclic immutable value graphs, and
the RFC-0027 follow-on's settled Option A/B decision (R1 prerequisite). The R2 RFC owns this.

### B.4 Network-FFI and `xloc` capability composition (RFC-0028 §7) — needs research

**Open question (RFC-0028 §7):** When a value is translocated to another node (`xloc`), its
`wild` host operations do not travel with it. The v0 FFI capability model (the build-time
`@std-sys` gate) is forward-compatible with a runtime `Capability<io>` model, but how
capability composition works across node boundaries is explicitly deferred.

**What is already decided:** RFC-0028 (Accepted 2026-06-23) decides the v0 FFI model:
build-time `@std-sys` gate, `wild` blocks, `Declared` guarantee baseline. §7 explicitly
defers two things: (a) runtime-enforced `Capability<io>` sandboxing (left forward-compatible
by the build-time gate's admissibility being a subset of what a runtime check would admit);
(b) `xloc` composition across node boundaries.

**Why this needs a research pass:** `xloc` is an R2 construct (RFC-0008 §4.6). It is typed
fallible and `Meta`-preserving (RT4/WF5). The FFI question is: when a value produced by a
`wild` host operation on node A is `xloc`-ed to node B, what capability does node B hold?
The `@std-sys` gate is lexical / build-time — it says nothing about the runtime capability of
a *remote* node. This is not merely a capability-propagation question; it is a trust question
(RT4: "Local and remote are *different types*"). The questions the R2 RFC must answer:

- Can a `wild`-backed value cross a node boundary at all? If so, what is the honest tag on
  that value at the destination (it may not be `wild`-accessible there)?
- Does the R2 RFC define a per-node `Capability` manifest (a `graft`-style declaration of
  what host operations a node can perform), and does `xloc` carry a capability check?
- How does the runtime-capability deferral (RFC-0028 §7a) interact with `xloc`? A node that
  receives a value produced by a `wild` op on a remote node may need to verify the remote
  node's capability before it can act on the value.

**Prerequisite:** the R1 `xloc` stub (not yet landed; the next R1 slice after channels is
§4.5 vocabulary activation) and the RFC-0028 follow-on for runtime capability. The R2 RFC
owns this, but the R1 `xloc` implementation RFC must leave the extension point clean.

### B.5 Concurrent session-merge and the `fuse` surface (RFC-0023 R23-Q2) — needs the G1 surface first

**Open question (RFC-0023 §4.3 / R23-Q2):** How does concurrent `State` merge work in the
ADK phylum when multiple `hypha` branches modify session state simultaneously? The v0
positions the snapshot model (mutation = a new content-addressed snapshot that moves, RT1) as
the honest baseline, and defers the `fuse`-merge concurrent story to R23-Q2.

**What is already decided:** RFC-0023 (Accepted 2026-06-21) settles: "Session snapshot-v0,
concurrent merge deferred to `fuse` (R23-Q2; never silent-overwrite)." The §3.3 and §3.7
concept-map rows confirm: "concurrent-branch merge = an explicit `fuse` (deferred, E7-2
M-667), never a silent overwrite." The State immutability tension is named in §4.3 and §9 as
"genuinely unresolved at the language level."

**Cross-link to DN-58 and G1:** The `fuse` construct's surface form (DN-58 Group A: fuse
trait/op surface — F-A1..F-A3) and its activation (Group B: reclaim form + policy type,
F-B1/F-B2) are G1-track items, not G8. The concurrent session-merge question is therefore
*downstream of DN-58*: the R23-Q2 story cannot be specified until the `fuse` surface is
ratified and the RT6 semilattice-merge law is embodied in an active construct (E7-2/M-667).

**What the R2 RFC must answer (after DN-58 and E7-2 land):**

- For the `adk` phylum's `State` scratchpad: what is the merge policy when two parallel
  `hypha` branches each produce a `state_delta`? The semilattice condition (RT6:
  commutative, associative, idempotent) must hold; the `adk` phylum must declare its
  merge op, and a merge that cannot satisfy the laws is an explicit conflict surfaced to a
  policy (RT3), not a silent overwrite (G2).
- What is the RT6 `Meta` composition rule for a merged `Session` value? The guarantee is
  the meet of the inputs (RFC-0008 RT6: "guarantee = meet of the inputs' guarantees");
  provenance is "derived-from-both."
- How does the `fuse`-merge interact with the Session's event log (an append-only
  `List<Event>`)? Appending is an idempotent, commutative merge if events are
  content-addressed (ADR-003); the corner case is concurrent events with a causal ordering
  dependency (ordering-sensitive state deltas).

**Note on Part B scope:** items B.1–B.4 are R2 distributed constructs. Item B.5 (concurrent
session-merge) is technically an R1 concurrent concern (multiple hyphae on one node), but it
is deferred to the R2 RFC because its answer is blocked on the G1 surface (DN-58/E7-2)
rather than a research gap. The R2 RFC should consume B.5 once DN-58 is Accepted and E7-2
is landed, and close R23-Q2 explicitly.

---

## Cross-links and sequencing

| Item | Upstream blocker | Vehicle |
|---|---|---|
| A (scheduler normativity) | RFC-0008 RT1–RT7 (already Accepted) | R1 implementation RFC |
| B.1 (time/clocks) | Research pass needed | R2 distributed RFC |
| B.2 (Byzantine/consensus) | Research pass needed | R2 distributed RFC |
| B.3 (distributed reclamation) | DN-59/RFC-0027 follow-on (Option A/B settled) + research pass | R2 distributed RFC |
| B.4 (network-FFI/xloc) | R1 `xloc` stub + RFC-0028 follow-on | R2 distributed RFC |
| B.5 (session-merge/fuse) | DN-58 Accepted + E7-2/M-667 landed | R2 distributed RFC (after G1) |

The **sequencing dependency** within Part B: B.3 requires the R1 cross-hypha ownership
decision (DN-59/RFC-0027 follow-on) as its starting point. B.5 requires DN-58 (G1). B.1
(clocks) has no upstream corpus blocker but needs research. B.2 (Byzantine) and B.4
(xloc/FFI) similarly need research and the R1 `xloc` implementation respectively.

The natural launch order for the R2 RFC is: confirm B.3's R1 prerequisite is settled (DN-59 +
RFC-0027 follow-on), do the B.1/B.2 research passes in parallel, and draft the R2 RFC once
B.1/B.2 are `Empirical`-grounded. B.4 and B.5 are design-time questions that the R2 RFC
drafts and delivers after their respective R1/G1 gates land.

---

## Definition of Done

This DN is **Accepted** when:

1. Part A (scheduler normativity) is ratified by the maintainer as the normative commitment
   for the R1 implementation RFC. No implementation is required before ratification; the
   commitment is a design decision.
2. Part B is acknowledged as the bounded forward agenda for the R2 RFC, with no item treated
   as resolved by this DN.

This DN is **Enacted** when:

- Part A: the R1 implementation RFC cites this DN as its normativity source and the full
  scheduler (beyond M-357's fork/join + Kahn-channel baseline) ships with the RT2
  differential green.
- Part B: a R2 distributed-execution RFC lands that explicitly closes B.1–B.5 with
  research-grounded decisions, citing this DN as the agenda source.

---

## Open questions (FLAGs)

- **FLAG-1** (for the R1 implementation RFC): is the conformance gate (A.4's
  per-task-`TaskOutcome` + channel-transcript equality) sufficient, or does the RT2
  differential also need to cover cross-task event ordering? The M-357 baseline checks
  per-task outcomes and channel transcripts; ordering within a single task is determined by
  the sequential reference. The question is whether event ordering *across* tasks (e.g. which
  task's `TaskOutcome` arrives first in the parent scope) needs to be in the differential or
  is deliberately scheduler-variable.
- **FLAG-2** (for the R2 RFC): the B.2 Byzantine/consensus question may require a mesh
  *variant* (a separate capability-annotated mesh type) rather than a flag on the existing
  `mesh` construct. This affects DN-58 Group A surface decisions if `fuse` must carry a
  Byzantine-safe variant. Flag to the G1 surface team before DN-58 Enacted.
- **FLAG-3** (for the orchestrator / Doc-Index owners): `docs/Doc-Index.md`,
  `CHANGELOG.md`, and `tools/github/issues.yaml` need updates to register DN-61 and the G8
  items as captured. This DN does **not** touch those files — they are orchestrator-owned
  (CLAUDE.md swarm discipline). FLAG to the integrating parent.

---

## Meta — changelog

- **2026-06-28 — Part A: Accepted (ratified by maintainer, in-session); Part B: stays Draft (open research agenda).** Part A (R1 scheduler normativity — RT2 + fuel-compatible cooperative stepping + RT7) ratified. Part B (R2 distributed: clocks R8-Q3, Byzantine R8-Q4, distributed reclamation RFC-0027 OQ-2, network-xloc, concurrent fuse-merge) explicitly recorded as open research agenda — not ratifiable until dedicated research passes. Status split made explicit in the Status field. (Append-only; VR-5; G2.)
- **2026-06-28 — Draft created.** Part A: scheduler-normativity proposal (R8-Q1), tagged
  `Declared`-with-argument, grounded in RFC-0008 RT2/RT7 and M-357 differential baseline.
  Part B: five-item distributed-execution agenda (R8-Q3 clocks, R8-Q4 Byzantine, RFC-0027
  OQ-2 distributed reclamation, RFC-0028 §7 xloc/FFI, RFC-0023 R23-Q2 session-merge), each
  tagged `Declared` agenda-only with an honest statement of what each needs. Cross-links to
  DN-58 (fuse surface), DN-59 (R1 reclamation), RFC-0027/0028/0023/0008. Three FLAGs
  surfaced (FLAG-1 for R1 RFC; FLAG-2 for G1 mesh/Byzantine surface; FLAG-3 for
  orchestrator-owned shared files). No shared file touched; owned by the integrating parent.
  (Append-only; VR-5; G2.)
