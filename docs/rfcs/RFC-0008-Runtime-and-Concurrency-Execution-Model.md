# RFC-0008 — Runtime & Concurrency Execution Model

| Field | Value |
|---|---|
| **RFC** | 0008 |
| **Status** | **Accepted** (2026-06-16 — maintainer sign-off; the Runtime-tier grounding ADR-012 §7.3 required. RT1–RT7 and the §4 model are now **normative**. Enactment is staged: the budget-unification slice — RFC-0014 §4.8 — landed first (M-353); the concurrency/supervision track — per-task budgets, cancellation, cross-task propagation, `reclaim` — is the §4.7 revision in progress, M-355.) |
| **Type** | Foundational / normative (once Accepted) |
| **Date** | June 10, 2026 |
| **Depends on** | RFC-0004 (single-node execution model — **extended, not changed**); RFC-0001 (Value/`Meta`/guarantee lattice, WF1–WF5); RFC-0005 (the one selection mechanism + EXPLAIN); RFC-0006 (S1–S6, LR-4/LR-8/LR-9); RFC-0007 (totality / `matured`); RFC-0002 (`ProbabilityBound`, ADR-010 δ-kernel); ADR-012 §7.3 (the gap this fills); DN-02 (naming law); research **T4.1–T4.6** (`research/04-runtime-concurrency-RECORD.md`) |
| **Coupled with** | RFC-0003 (the `spore` scope revision, ADR-012 §7.4 — *deliberately not decided here*); DN-03 (Runtime-tier vocabulary ratification through the three-test gate — *also not decided here*) |

## 1. Summary

ADR-012 introduced a Runtime-tier vocabulary (`hyph`, `anas`, `xloc`, `sclrt`, `myco`, `forage`,
`rhizo`, `cmn`, `dimorph`, `reclaim`) that presupposed a concurrency + distribution execution
model the corpus had never defined or grounded — RFC-0004 is single-machine value semantics, and
T0–T3 ground representations/numerics/VSA/language, not distributed systems (ADR-012 §7.3). This
RFC closes that gap **the way the rest of the corpus was built**: a research pass first
(Pass 4, T4.1–T4.6), then a normative model.

The model in one sentence: **concurrency and distribution are layered *above* the unchanged
RFC-0004 per-node kernel, values are the only thing that ever crosses a boundary, the
deterministic fragment is the default and the trusted base extends to replay it, and every
nondeterministic, lossy, or fallible aspect of the runtime — placement, merging, delivery,
failure — is reified, policy-driven, and honestly tagged on the same guarantee lattice as
everything else.** Seven runtime invariants (RT1–RT7, §4.1) make that checkable. The Runtime
vocabulary stays **reserved, not active syntax**, until DN-03 ratifies names against this model
and implementation RFCs land (§4.5).

## 2. Motivation

Three forces make this RFC necessary now, and shape what it may *not* do:

1. **The grounding rule.** The Runtime tier was documented vocabulary with no defined semantics —
   exactly the "ungrounded normative claim" the house rules forbid. Either the model gets defined
   and grounded, or the vocabulary stays aspirational forever. (ADR-012 §7.3.)
2. **The honesty rule does not stop at the machine boundary.** Distribution introduces the most
   silent-failure-prone phenomena in computing: lost messages, partial failure, stale replicas,
   racy merges. A substrate whose identity is *certified, never-silent, honest-per-operation*
   must extend those guarantees to the runtime — or visibly refuse to offer one. The literature's
   sharpest result here is negative and forty years stable: papering over the local/remote
   difference is the canonical design error (T4.5, Waldo et al. 1994). Mycelium's never-silent
   rule and that result are the same principle; this RFC commits the runtime to it.
3. **Value semantics is an unusually strong starting position.** The single hardest problem of
   concurrent runtimes — shared mutable state — *does not exist in this language* (LR-8/LR-9:
   immutable, acyclic values; no aliased mutation to police). The model below is deliberately
   built so that this advantage is never given back.

## 3. Guide-level explanation — the fungal runtime, honestly

A Mycelium program today runs on one node: the trusted interpreter or the AOT artifact
(RFC-0004). This RFC defines what it means for *many* computations to run, communicate, move,
pause, and fail — in fungal terms, because the metaphor genuinely maps (DN-02's T-map test),
checked term-by-term in §4.5:

- A **hypha** is a unit of concurrent execution: a checked computation over immutable values,
  living inside a *scope* that outlives none of its children (structured concurrency — T4.1).
  Hyphae never share state; they exchange **values** (with their `Meta` intact) over typed
  connections.
- `fuse` (anastomosis) is how two hyphae's *state* merges: only through declared merge
  operations with join-semilattice discipline (commutative, associative, idempotent — the
  CRDT *sufficient condition* for convergence, T4.2), and the merged value's guarantee is the
  **meet** of the inputs. Fusion is an operation with semantics, never an ambient effect.
- `xloc` (translocation) moves values between nodes. It is explicit, typed as fallible, preserves
  metadata (WF5 does not stop at the NIC), and is *not* a representation change — if the wire
  format differs from the value's `Repr`, that is a `Swap` and it is visible (S1).
- A `cyst` (encystment — the dormant form) is a checkpoint: because the deterministic fragment's
  computations are replayable values-plus-continuation, a checkpoint is an ordinary
  content-addressed artifact — the same move durable-execution runtimes made by *requiring*
  determinism of workflow code (T4.4).
- `forage` is placement: *where* work runs. Placement affects performance, never meaning —
  the Legion lesson (mapping is separated from semantics, T4.3) — so it is an RFC-0005 policy
  with EXPLAIN, the third site of the one selection mechanism.
- The `mesh` (the common mycorrhizal network) is decentralized coordination: gossip/pub-sub whose
  delivery and convergence guarantees are *probabilistic* — so they carry `ProbabilityBound`s
  and honest strength tags, like every other approximate thing in Mycelium (T4.2).
- `reclaim` is supervision: detecting and reclaiming stale *runtime units* (never memory —
  LR-9 already handles memory automatically), under reified supervision policies, in the OTP
  tradition (T4.5).

None of this is syntax yet. It is the semantic ground the syntax will stand on.

## 4. Reference-level design (normative once Accepted)

### 4.1 The runtime invariants RT1–RT7

These extend the syntactic honesty invariants S1–S6 (RFC-0006 §4.1) to concurrent and
distributed execution. They bind every future runtime construct and implementation; violating
one is a rejection criterion for a runtime design, exactly as S1–S6 are for a syntax.

- **RT1 — Values move; state is never shared.** The only thing that crosses a hypha, channel,
  or node boundary is an immutable [`Value`] with its `Meta` intact (WF5 extended across
  boundaries). There is no shared mutable state, no locks, no data races — not as a discipline
  but as an absence (LR-8/LR-9 made concurrent). *Grounding:* per-process heaps and
  share-nothing message passing are the load-bearing choice of the most battle-tested concurrent
  runtime in production (Erlang/OTP, T4.1/T4.5); Mycelium gets the same property from value
  semantics without copying discipline.

- **RT2 — The deterministic fragment is the default.** The default concurrency forms are
  *deterministic*: structured fork-join over pure computations, and single-producer/
  single-consumer typed channels (the Kahn condition — a network of deterministic processes with
  blocking single-reader channels is itself deterministic, T4.1). A program in this fragment has
  a *sequential reference semantics* the trusted interpreter can execute and replay; NFR-7
  extends to concurrency as: **concurrent observable ≡ the deterministic reference's
  observable**, validated through the M-210 shared checker. The trusted base stays sequential
  and fuel-guarded — concurrency adds scheduling *outside* the kernel, never new meaning inside
  it (KC-3).

- **RT3 — Nondeterminism is reified, named, and explained.** Every departure from RT2's fragment
  — racing, selection among ready sources, adaptive placement, merge ordering — is an explicit
  construct whose *decision procedure* is a content-addressed RFC-0005 policy with mandatory
  EXPLAIN. Placement/foraging is the canonical case: it may consult signals and feedback, but
  the deciding artifact is total, inspectable, non-learned (RFC-0005's posture), and
  semantics-free — *where* a hypha runs may change performance, never the observable
  (the Legion mapper separation, T4.3). One selection mechanism now serves **three** sites:
  swap-target (RFC-0002), packing schedule (RFC-0004), placement (this RFC).

- **RT4 — Partial failure is explicit; distribution transparency is forbidden.** Any operation
  that can fail partially — remote calls, translocation, fusion across a partition — is typed
  fallible (`Result`/timeout), and no construct may erase that (S5 extended to the network).
  Local and remote are *different types*; the runtime never silently retries, drops, or
  reorders. *Grounding:* the canonical negative result of distributed systems engineering
  (T4.5: Waldo et al. 1994 — latency, concurrency, and partial failure make
  location-transparent RPC a category error) — independently rediscovered by every
  transparent-distribution system since.

- **RT5 — Runtime guarantees are tagged on the same lattice.** Delivery, convergence,
  failure-detection, and freshness guarantees are per-operation claims like any other:
  probabilistic ones carry a `ProbabilityBound` (δ) with a basis (gossip convergence:
  proven-or-empirical per protocol and deployment, T4.2); a failure detector's verdict is
  *suspicion with confidence*, never `Exact` (FLP: in an asynchronous system, crash vs slow is
  undecidable — T4.5), so acting on it is acting on a tagged, bounded claim. The honesty rule's
  one lattice covers the runtime; no second vocabulary of informal "guarantees" is introduced.

- **RT6 — Fusion is lawful merge: join on payload, meet on guarantee.** Anastomosis (state
  fusion) happens only through declared merge operations that are commutative, associative, and
  idempotent — the monotonic-semilattice condition under which convergence is a *theorem*
  (sufficient, not necessary — CRDT strong eventual consistency, T4.2; mechanized in
  Isabelle/HOL, so the tag can be `Proven` with the side-conditions *checked*: semilattice laws
  property-tested, delivery assumptions certified by the mesh). The merged value's `Meta` composes honestly: guarantee =
  meet of the inputs' guarantees (and the merge op's own intrinsic strength, RFC-0001 §4.7),
  provenance = derived-from-both. A merge that cannot satisfy the laws is not an anastomosis —
  it is an explicit conflict surfaced to a policy (RT3). The pleasing symmetry — **payload joins
  up its state lattice while honesty meets down the guarantee lattice** — is the design's
  load-bearing rhyme, not an accident: both are semilattice disciplines, which is exactly what
  makes merge order irrelevant (RT2-compatible).

- **RT7 — Runtime lifetimes are structured; an orphan hypha is not expressible.** Every hypha is
  created inside a scope, and a scope does not exit until its children have completed, been
  cancelled, or been explicitly detached *into another owning scope* (structured concurrency —
  T4.1). This extends LR-9's guarantee ("in safe Mycelium a memory leak is not expressible") to
  runtime units: **in safe Mycelium a leaked task is not expressible**. `reclaim` (supervision)
  operates on this tree: failure handling and reclamation policies attach to scopes, OTP-style
  (T4.5), and are themselves reified policies (RT3). This is the answer to ADR-012 §7.3's
  warning that "a spawn/mesh world reopens leak/lifetime questions the value-semantics model had
  closed" — it reopens them only if lifetimes are unstructured, so they are not.

### 4.2 Relation to RFC-0004 (unchanged per-node model)

RFC-0004 remains the normative *per-node* execution model: trusted interpreter as reference,
MLIR→LLVM AOT for `matured` definitions, schedule-staged packing, the one certificate checker.
This RFC composes nodes; it changes nothing inside one. Specifically:

- The **trusted base stays sequential** (KC-3): the reference semantics of a concurrent program
  is its deterministic sequentialization (RT2); the parallel/distributed implementations are
  *performance paths* validated against it, exactly as AOT is validated against the interpreter
  (NFR-7, graded per RFC-0004 §3).
- **`matured` and totality** (RFC-0004 §4, RFC-0007 §4.5) gain runtime significance:
  checkpointability (§4.4) and migration presuppose replayable determinism, so the
  deterministic-fragment + checked-total discipline is what makes a computation *dormable* —
  the same gate, one more privilege behind it.
- **`tier`** (mode switching) is not a new mechanism: switching interpreted ↔ native is
  RFC-0004's existing `ExecutionMode` story (observable-equivalent by NFR-7, the JIT-tiering
  precedent — T4.6); switching dense ↔ sparse *representation* is a `Swap` (S1) — lexically
  visible, certified, never a runtime's silent prerogative.

### 4.3 Communication & the mesh

- **Channels** are typed, value-carrying, and by default single-producer/single-consumer (the
  RT2 fragment). Multi-source forms (select/merge) exist but are RT3 constructs: the
  arbitration is a named policy. Backpressure is part of the channel contract
  (demand-signalled, Reactive-Streams-style — T4.3): an unbounded silent buffer is a hidden
  resource leak and is excluded by construction (RT7's spirit applied to queues).
- **Session/protocol typing** for multi-step channel protocols is the researched direction for
  static fusion safety (T4.2); v0 commits only the *hook*: a channel's element type may be a
  protocol reference, and protocol conformance is a checkable obligation (the same
  pattern as guarantee indices: dynamic check first, static discipline staged later).
- **The `mesh`** (the common mycorrhizal network) is gossip/pub-sub overlay coordination for discovery, signals, and
  resource accounting. Its guarantees are explicitly probabilistic (RT5): per-protocol
  `ProbabilityBound`s with declared bases (T4.2's verified results for epidemic broadcast),
  surfaced through `Meta` like every other δ. Byzantine participants are **out of scope of v0**
  (R8-Q4): the mesh trusts its members; hostile-member hardening is its own future RFC.

### 4.4 Durability: cysts and spores

A computation in the dormable fragment (§4.2) checkpoints to a `cyst` (encystment): a
content-addressed artifact containing (a) the values in scope, (b) the continuation *by content
hash* (the code is already content-addressed — ADR-003), and (c) the `Meta` needed to resume
honestly. Determinism makes this sound: resume-and-replay reaches the same observable (the
durable-execution precedent, which *requires* workflow determinism for exactly this reason —
T4.4). A `cyst` is data: storable, `xloc`-able, inspectable, EXPLAIN-able.

The **spore** is the deployment-shaped sibling: code + initial values + manifest, germinating
into a running hypha. Its precedents are content-addressed artifact systems (Nix derivations,
OCI digests, Wasm components — T4.4) plus Unison's by-hash code shipping (T4.3). **This RFC
deliberately does not redefine `spore`:** DN-02/RFC-0003 fix it as the reconstruction manifest,
ADR-012 §7.4 flags the broader deployable-artifact sense, and reconciling the two is the
RFC-0003 revision (coupled work) — this section only establishes that the *runtime model has a
place for it* and that the manifest is the natural component of the larger artifact.

### 4.5 The Runtime vocabulary, grounded — and still reserved

The operational meaning each term now has (the T-map test ADR-012 §7.3 said could not yet be
run) — with **one name per term** ratified by DN-03 (flat; ADR-012 §7.6's canonical+alias scheme
was rejected as needless surface area):

| Name | Operational meaning (this RFC) | Grounding | Invariants |
|---|---|---|---|
| `hypha` | structurally-scoped concurrent computation over immutable values | T4.1 | RT1/RT2/RT7 |
| `fuse` | lawful state fusion: semilattice merge, meet-composed `Meta` — RT6 is genuine *merge* (two states converge into one), so `fuse` (not `weave`/`anastomose`) | T4.2 | RT6 |
| `xloc` | explicit, fallible, `Meta`-preserving value movement with backpressure ("trans-locate") | T4.3 | RT1/RT4 |
| `cyst` | content-addressed checkpoint of a dormable computation — encystment *is* the dormant-resumable form; `cyst(…)` constructor-style like `spore(…)` | T4.4 | RT2 + §4.4 |
| `graft` | capability contract with external infrastructure; the capability is an affine `substrate` handle (LR-8) | T4.3/T4.5 | RT4 |
| `forage` | adaptive placement/discovery as a reified RFC-0005 policy (third site) | T4.3 | RT3 |
| `backbone` | a declared/promoted high-bandwidth transport path — a placement-policy artifact, semantics-free | T4.3 | RT3 |
| `mesh` | gossip/pub-sub overlay with honest probabilistic guarantees | T4.2 | RT5 |
| `tier` | execution-mode switch: tiering = RFC-0004 `ExecutionMode` (NFR-7-equivalent); a representation switch is a `Swap` (S1), not this | T4.6 | RT2/S1 |
| `reclaim` | supervision-tree reclamation of *runtime units* (never memory — LR-9) | T4.5 | RT7 |

**Status rule (normative):** these remain **reserved vocabulary, not active syntax** — DN-03 has
ratified the *names* (above) through the DN-02 three-test gate, but activation still requires an
implementation RFC committing each construct's typing and elaboration per RFC-0006 §4.3.
Examples using them remain illustrations of intent (ADR-012 §7.3's marking stands).

### 4.6 Staging

- **R0 (this RFC):** the model and invariants. No syntax, no implementation obligation.
- **R1 (single node):** structured concurrency + deterministic channels in the runtime, behind
  the RT2 reference-sequentialization differential. The natural successor to the L1 track.
- **R2 (distribution):** `xloc`, `mesh`, `cyst`s — each construct landing with its
  honest bounds (RT5) and its differential/TV obligations.

R1 before R2 is load-bearing: every R2 guarantee is stated against R1's deterministic core.

## 5. Drawbacks

- **Determinism-first costs expressiveness up front.** Racy, latency-hiding idioms (speculative
  hedged requests, first-wins racing) are RT3 constructs with named policies — more ceremony
  than mainstream async. That ceremony *is* the honesty rule; the alternative is silent
  nondeterminism.
- **A sequential reference for concurrent programs** means the trusted base cannot itself
  exploit parallelism; performance lives entirely in the (validated) implementation paths. This
  mirrors the interpreter/AOT split and inherits its discipline and its costs (NFR-7 testing
  surface grows).
- **The metaphor can overreach.** The three-test gate (DN-02) and the §4.5 table are the
  containment: a term with no RT-invariant-respecting operational meaning does not ship.

## 6. Rationale & alternatives

- **Why not a full actor model (Erlang/Akka) as the core?** Actors are nondeterministic at the
  root (mailbox arrival order), which would surrender RT2's deterministic default and make the
  honest answer to "what does this program mean?" scheduler-dependent. Mycelium takes actors'
  *isolation* (RT1) and *supervision* (RT7) and leaves arrival-order nondeterminism as an
  opt-in RT3 construct. (T4.1/T4.5.)
- **Why not effect-handler concurrency as the semantic base (Koka/OCaml 5)?** Attractive and
  consistent with T3.4's divergence-only posture — but it defines concurrency *inside* the
  language kernel, growing the trusted base (KC-3). Layering scheduling outside the kernel,
  with sequential reference semantics, keeps the kernel auditable. The effect-typing *surface*
  (does `spawn` appear in an effect row?) is deliberately open (R8-Q2).
- **Why not full distribution transparency (classic RPC/DSM)?** Forty years of negative
  results; forbidden by RT4. (T4.5.)
- **Why CRDT discipline for fusion instead of consensus?** Consensus (Paxos/Raft) buys
  linearizability at the cost of availability and of a *global* coordination dependency;
  semilattice merge buys deterministic convergence with local decisions — the right default for
  a value-semantics substrate. Consensus, where truly needed, is a future explicit construct
  with its own honest costs (R8-Q4 territory). (T4.2.)

## 7. Prior art

Erlang/OTP (isolation, supervision, let-it-crash); structured concurrency (Trio nurseries,
Kotlin scopes, Java StructuredTaskScope); Kahn process networks (deterministic dataflow); LVars
(lattice-based deterministic concurrency — the nearest relative of RT6's semilattice story);
CRDTs/strong eventual consistency (Shapiro et al.); session types (Honda et al.); epidemic/
gossip protocols (Demers et al.; HyParView/Plumtree; gossipsub); Waldo et al., *A Note on
Distributed Computing*; Unison's `Remote` (content-addressed code mobility — with ADR-003
already adopted, Mycelium is unusually close to this); Cloud Haskell (static pointers — the
weaker non-content-addressed cousin); Legion (mapping/semantics separation); timely dataflow;
Ray; Reactive Streams (backpressure); work stealing (Blumofe–Leiserson); Temporal/Durable
Functions (determinism-gated durable execution); CRIU; Nix/OCI/Wasm components (content-addressed
deployables); JIT tier-up/deopt (mode switching with semantic equivalence). Full citations:
`research/04-runtime-concurrency-RECORD.md`.

## 8. Unresolved questions

- **R8-Q1 (scheduler spec).** How much of the R1 scheduler is normative? (Candidate: only the
  RT2 determinism obligation + fuel-compatible cooperative stepping; everything else is
  implementation.) Decided at the R1 implementation RFC.
- **R8-Q2 (effects surface).** Does hypha creation appear in the T3.4 effect row (growth path
  "bit → small fixed row"), or is structure (RT7) enough? Interacts with RFC-0007's stage-1
  grading.
- **R8-Q3 (time).** Clocks, timeouts, and deadlines are unavoidably nondeterministic inputs —
  reified how? (Candidate: time reads are explicit effects with declared resolution; logical/
  hybrid clocks for mesh ordering.) Needs its own research slice.
- **R8-Q4 (trust scope).** v0 mesh trusts its members (no Byzantine tolerance) and defers
  consensus. The boundary where "honest guarantees" meets "adversarial participants" is a
  future RFC with its own research pass.
- **R8-Q5 (`spore` reconciliation).** *Resolved at the scope level* (ADR-013 + RFC-0003 r2,
  2026-06-10): spore = the content-addressed deployable unit, the manifest one component,
  `spore(v)` the degenerate single-value case — §4.4's composition reading is ratified. Still
  open: the deployable artifact's schema, signing story, and germination contract (the R2
  implementation stage's obligation).
- **R8-Q6 (vocabulary ratification).** *Resolved* — DN-03 ran the three-test gate over §4.5's
  meanings and ratified **one name per term** (flat), rejecting ADR-012 §7.6's canonical+alias
  scheme. §4.5's table now carries the single ratified names.

## 9. Future possibilities

Guarantee-aware placement (forage policies that weigh a node's ability to *preserve* strength —
e.g. avoid lossy re-encodes); mesh-wide EXPLAIN ("why is this value here?" answered from
provenance + placement records); `cyst`-based time-travel debugging (checkpoints are values;
diffing them is `EXPLAIN` over history); native-ternary nodes joining the mesh as just another
substrate (the RFC-0004 backend story, distributed).

## Meta — changelog

- **2026-06-16 — Accepted.** Maintainer ratified `Draft → Accepted`: RT1–RT7 and the §4 model are
  now **normative** (the Runtime-tier grounding ADR-012 §7.3 required). Ratification opens the runtime
  track in staged slices — *separate named budgets, one enforcement mechanism*, then concurrency:
  (1) the **budget-unification slice** (RFC-0014 §4.8) lands first as M-353 — the recovery `Budgets`
  ledger is lifted into `mycelium-interp` (the shared budget-resolution surface both the env-machine
  and the recovery driver consume) and an effect overrun routes through
  `mycelium_interp::EvalError::EffectBudget`, the effect sibling of `FuelExhausted`/`DepthLimit` on the
  one runtime refusal channel — needing **no** RT1–RT7 commitment and **no** kernel hook (KC-3; the
  ledger lives where fuel/depth live); (2) the **route → observability-sink** binding (RFC-0013 §8)
  lands next as M-354, honouring RT5 (sink delivery guarantees tagged on the lattice) and I1 (routing
  never gates propagation); (3) the **concurrency/supervision** track — lifting RFC-0014's single-task
  v0 boundary to per-task budgets, cancellation, cross-task failure propagation, and bounded cascades
  under `reclaim` supervision (RT4/RT7; the Erlang/OTP max-restart-intensity grounding, Research Record
  05 T5.3) — is the §4.7 revision, M-355, presented as a frozen-spec change before folding. The
  vocabulary stays **reserved, not active syntax** (§4.5 status rule unchanged) until the implementation
  RFCs land. Append-only.
- **2026-06-10 — Draft.** Initial draft from Research Pass 4 (T4.1–T4.6): the RT1–RT7 runtime
  invariants extending S1–S6 to concurrency/distribution; deterministic-fragment-first posture
  with sequential reference semantics (NFR-7 extended); placement as the third RFC-0005 site;
  lawful fusion (payload join / guarantee meet); honest probabilistic runtime guarantees
  (`ProbabilityBound` on delivery/convergence/failure-suspicion); structured lifetimes closing
  the task-leak vector (LR-9 extended); the grounded-but-still-reserved Runtime vocabulary table
  with DN-03 and the RFC-0003 `spore` revision left as coupled, separately-owned decisions.
  Fills the gap flagged in ADR-012 §7.3.
