# Design Note DN-63 — RFC-0008 R2 Distribution Vocabulary: Per-Construct Decomposition and Implementation RFC Planning

| Field | Value |
|---|---|
| **Note** | DN-63 |
| **Status** | **Draft** (2026-06-29 — initial planning capture; advisory; not ratifiable until each per-construct research pass is completed; see posture below) |
| **Decides** | *Nothing normatively.* Decomposes RFC-0008 §4.6 R2 into six per-construct implementation RFC tracks, each with explicit typing/elaboration strategy, dependency ordering, gate conditions, and honest guarantee tags. Decides no design question that belongs to those implementation RFCs. |
| **Feeds** | RFC-0008 §4.5/§4.6 (the R2 activation gate); DN-61 (the R2 distributed-execution research agenda); DN-59 (R1 reclamation — upstream of distributed reclamation); RFC-0005 (the one selection mechanism — `forage`/`backbone`); RFC-0028 (FFI/capability — `graft`); RFC-0027 (reclamation — `cyst` durability); RFC-0023 (session-merge R23-Q2 — `fuse`/`mesh`); future per-construct implementation RFCs for each of the six R2 terms |
| **Task** | M-668 |
| **Date** | June 29, 2026 |

> **Posture (transparency rule / VR-5 / G2).** This is an advisory **planning capture** — the
> same role DN-11 played for the Phase-5 wave and DN-61 played for the R2 distributed agenda.
> Every claim in this note is tagged `Declared` unless cited to a checked basis. Nothing here
> ratifies any R2 construct; each construct's implementation RFC does that, with its own research
> pass. Planning-stage sketches are `Declared` design directions, explicitly not `Proven` or
> `Empirical`. Open questions are marked as such — not resolved by plausible-sounding answers
> (VR-5). **Append-only:** this note may gain dated sections as open questions resolve; it must
> not be rewritten.

---

## 1. Context and gate

RFC-0008 §4.6 stages the runtime build:

- **R0:** the model and invariants (RT1–RT7) — complete.
- **R1:** structured concurrency and deterministic channels on a single node — substantially
  enacted (M-666 `hypha`/`colony`; M-667 `fuse`/`reclaim`/`tier`; see RFC-0008 §4.5 append).
- **R2:** distribution — `xloc`, `mesh`, `cyst`, `graft`, `forage`, `backbone` — **not yet
  active**. Per RFC-0008 §4.5's status rule: these six names remain **reserved, not active
  syntax** until an implementation RFC commits each construct's typing and elaboration.

**Gate (M-668 acceptance criterion, grounded in RFC-0008 §4.5 / E7-2 dependency ordering):**
R2 is explicitly gated on R1 completion (M-667). M-667 is the immediate predecessor of M-668
in the E7-2 epic (tools/github/issues.yaml §E7-2). Until M-667 is done, no R2 implementation
RFC should advance to Accepted; this planning note is preparatory work that runs in parallel
with the final M-667 work but does not bypass the gate.

**R2 gated on R1 completion (M-667) — `Declared`.** This gate is structural: RFC-0008 §4.6
states "R1 before R2 is load-bearing: every R2 guarantee is stated against R1's deterministic
core." Planning can proceed; activation cannot.

---

## 2. The six R2 constructs — authoritative definitions

All six are defined normatively in RFC-0008 §4.5 (the vocabulary table) and §4.6 (staging).
The descriptions below quote or paraphrase that table; they are not independent definitions.

| Construct | RFC-0008 §4.5 operational meaning | Invariants |
|---|---|---|
| `xloc` | explicit, fallible, `Meta`-preserving value movement across nodes ("trans-locate") | RT1/RT4 |
| `mesh` | gossip/pub-sub overlay with honest probabilistic guarantees | RT5 |
| `cyst` | content-addressed checkpoint of a dormable computation | RT2 and §4.4 |
| `graft` | capability contract with external infrastructure; affine `substrate` handle (LR-8) | RT4 |
| `forage` | adaptive placement/discovery as a reified RFC-0005 policy (the third site) | RT3 |
| `backbone` | declared/promoted high-bandwidth transport path — a placement-policy artifact, semantics-free | RT3 |

These six exhaust the RFC-0008 §4.5 table's R2 remainder (after R1 activation of `fuse`/`reclaim`/`tier`/`hypha`/`colony`/`tier`).

---

## 3. Per-construct typing and elaboration strategy

Each subsection below sketches the strategy a future implementation RFC should commit to.
All sketches are `Declared` design directions, not accepted design. Each needs a research pass
before it can advance to `Empirical` or support an Accepted RFC.

### 3.1 `xloc` — explicit, fallible, `Meta`-preserving value movement

**RFC-0008 grounding:** RT4 (partial failure is explicit; distribution transparency is forbidden);
RT1 (only immutable values cross boundaries, `Meta` intact, WF5 extended); §4.6 summary
"explicit, fallible, `Meta`-preserving value movement with backpressure." DN-61 §B.4 names
the network-FFI/capability composition question as the primary open research gap.

**Typing strategy (`Declared`):**

`xloc` is an expression form, not a statement — because it is typed fallible (RT4) and its
outcome is a value the caller must act on (RFC-0014 I1: never-silent). A candidate surface:

```
xloc(target: NodeRef, value: T) -> Result<T, XlocError>
```

where `XlocError` is a non-exhaustive enum covering: `PartitionError` (network failure before
delivery confirmed), `CapabilityError` (remote node cannot hold this value type — see §3.4
`graft`), `SerializationError` (the value's active `Repr` has no agreed wire form — a `Swap`
may be needed first, S1), and `TimeoutError` (delivery not confirmed within declared deadline,
R8-Q3/DN-61 §B.1). This is `Declared`; the full error set is an open question requiring a
research pass.

**Meta-preservation obligation (RT1/WF5):** the value's `Meta` travels with the value. Wire
serialization is a `Swap` if the wire repr differs from the value's active `Repr` (S1 — never
silent); the `Meta.policy_used` records the wire-format swap. The destination's `Meta` is the
source `Meta` compose-with-delivery-record (guarantee = meet of source guarantee and the
delivery channel's guarantee per RT5 — probabilistic if the mesh is the carrier).

**Elaboration strategy (`Declared`):** `xloc` elaborates to an L1 node with explicit
`Result`-typed output: `XlocNode { target, value, deadline_hint }` → `Result<T, XlocError>`.
The elaboration must produce no implicit retries (RT4) and must surface the wire-format swap
as a visible `Swap` node in the L1 tree if serialization is needed (S1). The R1 `xloc` stub
(DN-61 §B.4 prerequisite: "not yet landed; the next R1 slice after channels") must leave
this extension point clean — the R2 implementation RFC closes the actual elaboration.

**Dependency:** R1 channels (landed M-357 follow-on) and the RFC-0028 §7 follow-on (runtime
capability — DN-61 §B.4). DN-59/RFC-0027 follow-on (Option A/B cross-hypha ownership)
upstream.

**Open questions (FLAG):**
- FLAG-1: Can a `wild`-backed value cross a node boundary? (DN-61 §B.4) — needs RFC-0028
  follow-on's capability decision. Do not resolve here; `Declared` direction only.
- FLAG-2: Does `xloc` require a declared `backbone` path, or does it fall back to the `mesh`?
  The interaction between `xloc` and `backbone`/`forage` is a design question for the
  implementation RFC.
- FLAG-3: The wire-format swap story (S1) for structured values (not just `Binary`) — does
  every repr have a canonical wire form, or does `xloc` require an explicit `Swap` before it?
  Open question; RFC-0002 `Swap` rules apply but the wire-format extension is not specified.

---

### 3.2 `mesh` — gossip/pub-sub overlay with honest probabilistic guarantees

**RFC-0008 grounding:** RT5 (guarantees tagged on the same lattice — probabilistic ones carry
`ProbabilityBound`); §4.3 ("guarantees are explicitly probabilistic — per-protocol
`ProbabilityBound`s with declared bases, T4.2's verified results for epidemic broadcast");
§4.3 explicit: "Byzantine participants are out of scope of v0 (R8-Q4)." T4.2 cites Demers
et al. (epidemic broadcast) and HyParView/Plumtree/gossipsub as the grounding literature.

**Typing strategy (`Declared`):**

`mesh` is a runtime-level resource (a connection to the overlay), not a value expression.
Its surface likely mirrors the `colony` scoping pattern — a structured scope that manages
the mesh connection's lifetime (RT7 applies: no orphaned connection):

```
mesh(policy: MeshPolicy) {
    // hyphae inside here can publish/subscribe via the mesh
    subscribe(topic: Topic<T>, handler: fn(T) -> ())
    publish(topic: Topic<T>, value: T) -> Result<(), MeshError>
}
```

All `mesh` operations are typed fallible at the output (`Result`). Delivery guarantees are
`ProbabilityBound`-tagged on the `Meta` of returned values (RT5). Subscribe handlers receive
values with their guarantee degraded to at-most-once or at-least-once (whichever the
underlying protocol commits — `Declared` per-protocol, grounded in T4.2 evidence).

**Meta and guarantee tagging (`Declared`):** a value received over the mesh carries:
- `guarantee = meet(sender_guarantee, mesh_delivery_bound)` per RT5/RT6
- `provenance = Derived{op:"mesh_recv", topic, sender_id}` (G2 — never silent about origin)
- A `ProbabilityBound δ` on delivery, per the declared protocol (epidemic broadcast: proven
  convergence under the Demers et al. model, tagged `Empirical` in-repo pending a
  deployment-specific bound check, as RT5 requires)

**Elaboration strategy (`Declared`):** `mesh` elaborates to a scoped L1 node that acquires
a runtime handle to the overlay and manages the topic registry. The runtime-tier driver
(parallel to `mycelium_mlir::run_reclaim` for `reclaim`) provides the actual gossip/pub-sub
engine. Honest boundary: the in-repo elaboration and runtime stub are the commitment of the
implementation RFC; the actual overlay protocol (HyParView, gossipsub, or a simpler broadcast
tree for v0) is an implementation choice, not a normative commitment, provided the
`ProbabilityBound` is honestly declared for that choice.

**Dependency:** R1 `hypha`/`colony` (M-666, done). DN-61 §B.1 (time/clocks — mesh ordering
requires a clock model, R8-Q3; deferred). DN-61 §B.2 (Byzantine — out of v0 scope, R8-Q4).
DN-61 §B.5 (concurrent session-merge — consumes `mesh` after DN-58/E7-2).

**Open questions (FLAG):**
- FLAG-4: Which gossip protocol is v0's concrete mesh engine? (HyParView partial-view
  membership plus Plumtree broadcast, or a simpler broadcast-tree?) This affects what
  `ProbabilityBound` can honestly be stated. A research pass on T4.2 literature for the
  specific deployment model is needed before the implementation RFC can commit a bound.
- FLAG-5: Multi-producer topic semantics — does `publish` from multiple hyphae on the same
  topic require the topic type `T` to satisfy `Fuse`'s semilattice laws (RT6)? Or is
  at-most-once delivery the default and convergent merge an opt-in `mesh_fuse` variant?
  This is an open design question that interacts with DN-58's `fuse` surface and RFC-0023
  R23-Q2. Do not resolve here; flag to the implementation RFC.
- FLAG-6 (DN-61 FLAG-2 echoed): Byzantine-safe mesh variant may require a separate
  capability-annotated type, not a flag. Flag to the G1 surface team.

---

### 3.3 `cyst` — content-addressed checkpoint of a dormable computation

**RFC-0008 grounding:** RT2 (the deterministic fragment is the default; a checkpoint is
sound because determinism makes resume-and-replay reach the same observable); §4.4
("content-addressed artifact containing: (a) the values in scope, (b) the continuation by
content hash, (c) the `Meta` needed to resume honestly"). The precedent is durable-execution
runtimes that require workflow determinism (T4.4: Temporal/Durable Functions). A `cyst` is
data: storable, `xloc`-able, inspectable, EXPLAIN-able.

**Typing strategy (`Declared`):**

`cyst` is both a constructor expression and a resumable value:

```
// Construction — checkpoint the current computation's continuation
let c: Cyst<T> = cyst(expr)    // T is the output type of the dormable expr

// Resumption — resume from a checkpoint (fallible: the cyst may be stale)
let result: Result<T, CystError> = c.resume()
```

`Cyst<T>` is a content-addressed value (ADR-003): its identity is the hash of (a) the
continuation hash, (b) the values-in-scope hash, and (c) the `Meta` hash. This makes
two cysts with identical computational state identical values (`Exact`-grounded).

**Dormability gate (`Declared`):** only computations in the deterministic fragment (RT2)
are dormable. The `cyst` expression is a type error if the enclosed computation uses RT3
nondeterministic constructs (racing, policy-arbitrated selection) or un-`matured` scopes
(RFC-0017). This is an elaboration-time check, not a runtime check — `Declared` gate;
the implementation RFC must specify the exact dormability criterion.

**Meta and guarantee (`Declared`):** `cyst(expr).Meta.guarantee = guarantee(expr)` —
the checkpoint inherits the enclosedcomputation's guarantee. Resume adds a `Derived`
provenance record `{op:"cyst_resume", cyst_hash}`. A `CystError::Stale` result is
emitted if the continuation's code content hash is no longer available (the code was
re-published — ADR-003 immutability means old continuations may not be garbage-collected,
but this is an implementation concern, not a semantic one).

**Elaboration strategy (`Declared`):** `cyst(expr)` elaborates to a checkpoint node that
(a) evaluates `expr` up to its yield point, (b) serializes the continuation (as a
content-addressed blob, ADR-003), and (c) produces a `Cyst<T>` value. `.resume()` is a
method call that re-deserializes and re-enters the continuation from the checkpoint.
The implementation RFC must specify the continuation representation — a candidate is the
existing L0/MLIR `Value` serialization path, extended to carry a `ContinuationRef` (the
code hash of the next instruction and the in-scope value heap).

**Dependency:** `xloc` (§3.1) — a `Cyst<T>` is `xloc`-able, so the wire-format story
for `cyst` inherits the `xloc` decisions. RFC-0027 (reclamation — how does the `cyst`'s
value heap interact with the R1 reclamation model when the cyst is serialized/stored?
`Declared` open question; RFC-0027 §11 OQ-3 notes this as deferred). R1 channels (landed)
for the continuation serialization path. `matured` scope semantics (RFC-0017 Enacted) as
the dormability precondition.

**Open questions (FLAG):**
- FLAG-7: How are values in a cyst's scope reclaimed (or not) while the cyst is dormant?
  RFC-0027 §11 OQ-3 explicitly defers this. The implementation RFC must decide: does the
  cyst hold a `Declared` strong reference (keeping the value graph live) or a content-
  addressed `Empirical` reference that must be re-verified on resume? Do not resolve here.
- FLAG-8: Is `cyst` restricted to `matured` scopes, or is there a weaker dormability
  predicate for non-total computations? RFC-0008 §4.4 says "deterministic fragment" and
  §4.2 says "checkpointability and migration presuppose replayable determinism" — but the
  exact gate is not formalized. The implementation RFC must formalize it. `Declared` open.
- FLAG-9: Continuation representation — is the L0 MLIR bytecode + value heap the right
  continuation format, or does `cyst` require a higher-level IR (L1 AST with bindings)?
  Affects the EXPLAIN story and the `xloc` wire-format question (FLAG-3). Open question
  for the implementation RFC.

---

### 3.4 `graft` — capability contract with external infrastructure

**RFC-0008 grounding:** RT4 (distribution transparency forbidden; local and remote are
different types); §4.5 "capability contract with external infrastructure; the capability
is an affine `substrate` handle (LR-8)." RFC-0028 (Accepted 2026-06-23) is the existing
v0 FFI model (`@std-sys` build-time gate, `wild` blocks, `Declared` guarantee baseline);
RFC-0028 §7 explicitly defers runtime-enforced `Capability<io>` sandboxing and `xloc`
capability composition — `graft` is the R2 face of that deferral.

**Typing strategy (`Declared`):**

`graft` is a declaration (like `wild`) not an expression — it names the external capability
a node or phylum requires from its host infrastructure:

```
graft infra: SubstrateCapability<K> = substrate_handle
```

where `K` is a capability kind (e.g. `K = NetworkIO`, `K = FileIO`, `K = GpuCompute`)
and `SubstrateCapability<K>` is an affine handle (LR-8: uniquely owned, not aliasable).
The affinity ensures the capability is not duplicated silently. The `graft` declaration
is visible in the phylum manifest (the `spore`'s declared capability requirements — ADR-013).

**Meta and guarantee (`Declared`):** operations on a `SubstrateCapability<K>` carry
`guarantee = Declared` as the baseline (RFC-0028 §4: `wild` blocks carry `Declared` because
the host is opaque). The implementation RFC may raise this for specific well-understood hosts
(e.g. a `Declared`-with-argument POSIX-file capability) but may not exceed `Empirical`
without a checked measurement program (VR-5). `xloc` of a capability is a `CapabilityError`
if the destination node cannot satisfy the capability kind (FLAG-1 above echoed).

**Elaboration strategy (`Declared`):** `graft` elaborates to a capability-binding node that:
(a) checks the declared capability kind against the host's `graft` manifest at activation
time (not compile time — this is a runtime check, RT4), (b) produces an affine
`SubstrateCapability<K>` handle, and (c) records the capability binding in the EXPLAIN
trail. If the host cannot satisfy the capability, the result is `Result::Err(GraftError::Unsupported)` —
never a silent fall-back (G2).

**Dependency:** RFC-0028 follow-on (runtime `Capability<io>` model — explicitly deferred
in RFC-0028 §7a; `graft` IS that follow-on). `xloc` (§3.1) for cross-node capability
queries. Spore/manifest schema (ADR-013/RFC-0003 §R8-Q5 — capability declarations in the
deployable manifest).

**Open questions (FLAG):**
- FLAG-10: Should `graft` be a keyword or a macro/attribute? The `wild` precedent (RFC-0028)
  uses a block form; `graft` as a declaration form is a different syntactic class. The
  implementation RFC must decide. `Declared` direction only.
- FLAG-11: Does `graft` require a per-node capability manifest, and does `xloc` carry a
  capability-check step against the remote node's manifest? This is the core of RFC-0028 §7b
  (deferred). Do not resolve here; flag to the `graft` implementation RFC.
- FLAG-12: Affine capability handles and `fuse` — can two `graft`-acquired handles of the
  same kind be merged? LR-8 says affine values are not `Clone`-able; merging two capabilities
  is a design question. `Declared` open.

---

### 3.5 `forage` — adaptive placement as a reified RFC-0005 policy

**RFC-0008 grounding:** RT3 (nondeterminism is reified, named, explained); §4.5 "adaptive
placement/discovery as a reified RFC-0005 policy (the third site)"; §4.1 RT3 "one selection
mechanism now serves three sites: swap-target (RFC-0002), packing schedule (RFC-0004),
placement (this RFC)." RFC-0005 is the authoritative policy mechanism. DN-61 §A.2 notes
"`forage` (RT3) — where work runs is a reified RFC-0005 policy with EXPLAIN, semantics-free
by RT3. The scheduler is not a placement mechanism; `forage` is."

**Typing strategy (`Declared`):**

`forage` is an RFC-0005 policy site — not a new mechanism, but the third application of
the existing one. Its surface is a policy annotation on a `hypha` or `colony`:

```
hypha @forage(policy: PlacementPolicy) { … }
```

where `PlacementPolicy` is a content-addressed `SelectionPolicy` (RFC-0005 §2) whose
inputs are node-level `Meta` (available-node set, resource signals, `backbone` declarations)
and whose output is a `NodeRef`. Like all RFC-0005 policies: total, non-learned, deterministic
given the same inputs, mandatory EXPLAIN.

**Semantics-free placement (`Declared`):** the running node may change performance, never the
observable (RFC-0008 RT3; the Legion mapper separation, T4.3). The implementation RFC must
specify the RT2 differential obligation for `forage`: two different placements of the same
deterministic computation must produce the same per-task `TaskOutcome` and channel transcripts
(the same differential gate as the R1 scheduler, DN-61 §A.4 extended to placement).

**Elaboration strategy (`Declared`):** `forage` elaborates to a placement-policy binding on
the `hypha`/`colony` AST node. The runtime scheduler (M-357 + successors) consults the
active `PlacementPolicy` via the RFC-0005 EXPLAIN path before scheduling the hypha onto a
node. The policy result is recorded in the EXPLAIN trail (RFC-0005 §3: the `PolicyRef` and
per-candidate costs). No new mechanism — the existing RFC-0005 `SelectionPolicy` infrastructure
handles it.

**Dependency:** RFC-0005 (already Accepted — the mechanism is available). R1 `hypha`/`colony`
(M-666, done). `mesh` (§3.2) — the policy's "available node set" comes from the mesh overlay.
`backbone` (§3.6) — a backbone declaration is an input to the placement policy. DN-61 §B.1
(time/clocks — dynamic resource signals may need a clock model; deferred to R8-Q3).

**Open questions (FLAG):**
- FLAG-13: What node-level `Meta` signals are available to the `PlacementPolicy`? (Resource
  utilization, network topology, `backbone` availability, `graft` capability set.) This must
  be specified by the `forage` implementation RFC; using unspecified signals would be a
  black-box violation (ADR-006, RFC-0005 §2). `Declared` open.
- FLAG-14: What happens when the `PlacementPolicy` has no valid node (the available set is
  empty or all nodes fail the policy's predicates)? The result must be a
  `ForageError::NoCandidates` — a typed, explicit error, not a silent hang (RT4). The
  implementation RFC must specify this. `Declared` open.

---

### 3.6 `backbone` — declared high-bandwidth transport path

**RFC-0008 grounding:** RT3 (a placement-policy artifact, semantics-free); §4.5 "declared/
promoted high-bandwidth transport path — a placement-policy artifact, semantics-free." T4.3
(Legion mapper separation: the mapping of work to hardware is separated from semantics).

**Typing strategy (`Declared`):**

`backbone` is a declaration (analogous to `graft`) that promotes a transport path to a
first-class, named, EXPLAIN-able resource:

```
backbone link: BackboneRef = backbone_handle(from: NodeRef, to: NodeRef, bandwidth_hint: BandwidthHint)
```

A `BackboneRef` is a content-addressed handle (ADR-003 — two `backbone` declarations with
the same `{from, to, bandwidth_hint}` hash to the same `BackboneRef`). It is an input to
`forage` policies: a `PlacementPolicy` may prefer nodes reachable via a declared backbone
when the value being transferred exceeds a threshold (the cost function in the RFC-0005
policy decision table).

**Semantics-free (`Declared`):** using a `backbone` transport path changes no observable — it
is a performance hint. The implementation RFC must specify the fall-back (if the declared
backbone is unavailable, the result is `backbone_fallback: Err(BackboneError::Unavailable)` 
and the policy falls back to the default transport — never a silent hang, RT4).

**Elaboration strategy (`Declared`):** `backbone` elaborates to a transport-hint binding on
the runtime mesh layer. It is a passive declaration: it does not allocate a connection but
registers a preference in the placement-policy context. The runtime may or may not have a
dedicated transport path; `EXPLAIN` must record which transport was used and whether the
backbone preference was honored (RFC-0005 mandatory EXPLAIN).

**Dependency:** `forage` (§3.5) — `backbone` is an input to placement policies, so it is
consumed by `forage`. `mesh` (§3.2) — the mesh layer is the underlying transport that
`backbone` promotes or annotates. `graft` (§3.4) — a `backbone` between two nodes may
require a declared `graft` capability on both ends (e.g. `NetworkIO`).

**Open questions (FLAG):**
- FLAG-15: How is `backbone` declared and managed? Is it a per-phylum declaration (in the
  `spore` manifest, alongside `graft`) or a runtime dynamic declaration? The manifest path
  fits the "declared" framing; the runtime dynamic path fits the "promoted" framing. Both
  appear in RFC-0008 §4.5 ("declared/promoted") — the implementation RFC must choose.
  `Declared` open; maintainer decision needed.
- FLAG-16: Is `BackboneRef` affine (like a `graft` capability handle) or freely shareable?
  If a transport path can be used by multiple hyphae simultaneously, it is not affine. But
  if it carries a bounded-capacity guarantee, affinity (or a counted access token) may be
  appropriate. `Declared` open.

---

## 4. Dependency ordering

The six R2 constructs are not independent. The ordering below is `Declared` — grounded in
the RFC-0008 RT invariant cross-references and the open-question dependencies identified
above, but not `Proven` (no formal dependency proof; the ordering follows from the
typing/elaboration strategies described in §3).

```
R1 completion (M-667)
    └── [all R2 constructs gated here]

forage   (RFC-0005 already exists; pure annotation — can proceed first among R2)
backbone (consumes forage; needs mesh for the transport layer)
    └── mesh   (B.1/B.2 research passes; the forage/backbone transport base)
        └── xloc   (needs mesh as carrier; needs RFC-0028 follow-on for graft/capability)
            └── graft  (RFC-0028 §7 follow-on; consumed by xloc capability check)
            └── cyst   (needs xloc for mobility; needs RFC-0027 OQ-3 for reclamation)
```

A cleaner tabular view:

| Construct | Direct upstream blockers | Parallel workstreams |
|---|---|---|
| `forage` | R1 done (M-667) | none (RFC-0005 mechanism exists) |
| `backbone` | `forage` RFC (the policy annotation framework) | `mesh` (transport layer) |
| `mesh` | R1 done; DN-61 B.1 clock research; DN-61 B.2 Byzantine decision | `forage` (placement of mesh nodes) |
| `xloc` | `mesh` (carrier); RFC-0028 §7 follow-on (capability); R1 `xloc` stub | `graft` (capability check) |
| `graft` | RFC-0028 §7 follow-on; `xloc` extension point | `mesh` (node discovery) |
| `cyst` | `xloc` (mobility); RFC-0027 OQ-3 (reclamation in dormancy); `matured` scopes (RFC-0017 Enacted) | `forage` (placement of resumed cyst) |

**Natural launch order (advisory, `Declared`):**

1. **`forage` RFC first** — it adds the third RFC-0005 policy site with no new mechanism;
   depends only on R1 done and RFC-0005 (both complete). It is the lightest R2 construct.
2. **`backbone` RFC in parallel or immediately after `forage`** — it is a declaration that
   references the `forage` policy framework and the `mesh` transport layer; the `forage`
   RFC's policy-annotation framework must exist first, but `backbone` does not require `mesh`
   to be fully implemented (it only needs the transport-layer interface).
3. **`mesh` RFC second** — requires the clock (DN-61 B.1) and Byzantine (DN-61 B.2)
   research passes. These are the longest-lead items in R2; starting research early is the
   critical path. The `mesh` RFC is the infrastructure the remaining three (`xloc`, `graft`,
   `cyst`) depend on.
4. **`graft` RFC third** — requires the RFC-0028 §7 capability follow-on. Can be drafted in
   parallel with the `mesh` research pass; the implementation RFC requires the `mesh` node-
   discovery mechanism before `graft`'s capability manifest can be deployed.
5. **`xloc` RFC fourth** — requires `mesh` (carrier) and `graft` (capability check) to be
   at least at Accepted before `xloc` can commit its `CapabilityError` variant and the
   full wire-format story.
6. **`cyst` RFC last** — requires `xloc` (for mobility) and the RFC-0027 OQ-3 resolution
   (reclamation in dormancy). Can be drafted in parallel with `xloc` but may not be Accepted
   before `xloc` and the reclamation question are settled.

---

## 5. Honest guarantee tags — planning-stage inventory

All entries below are `Declared` unless explicitly noted otherwise. No R2 claim here is
`Proven` (no mechanized proof) or `Empirical` (no in-repo measurement). These are the honest
tags for a planning-stage decomposition, per VR-5.

| Construct | Guarantee claims | Tag | Basis / gap |
|---|---|---|---|
| `xloc` typing as fallible | `xloc` is `Result<T, XlocError>`; partial failure is explicit | `Declared` | RFC-0008 RT4 (normative); the exact error variants are open (FLAG-1/FLAG-3) |
| `xloc` Meta preservation | `Meta` travels with value, guarantee = meet(source, channel) | `Declared` | RFC-0008 RT1/WF5; channel delivery bound from RT5 |
| `mesh` delivery bound | probabilistic delivery with `ProbabilityBound δ` | `Declared` | RFC-0008 RT5; grounded in T4.2 (Demers et al.) — `Empirical` once a specific protocol is measured; `Declared` until then |
| `mesh` convergence | semilattice merge converges (RT6 CRDT sufficient condition) | `Declared` | RFC-0008 RT6 cites "mechanized in Isabelle/HOL" for the semilattice law (T4.2) — but that theorem's side-conditions (delivery assumptions) must be checked for the specific v0 mesh protocol; the in-repo tag stays `Declared` until checked |
| `cyst` soundness | resume reaches same observable as un-checkpointed run | `Declared` | RFC-0008 §4.4 cites T4.4 (durable execution precedent); the checked basis is RT2's sequential reference semantics + the dormability gate; soundness is `Declared`-with-argument; `Empirical` once a differential test is written |
| `graft` guarantee baseline | `Declared` baseline (host is opaque) | `Declared` | RFC-0028 §4 (normative for `wild`/`graft`); may rise to `Declared`-with-argument for specific well-understood hosts |
| `forage` semantics-free | placement does not change the observable | `Declared` | RFC-0008 RT3 (normative); the RT2 differential for placement is the conformance gate — `Empirical` once the differential test is written |
| `backbone` semantics-free | transport path does not change the observable | `Declared` | RFC-0008 §4.5 (normative claim: "semantics-free"); no test yet |

**Special note on `mesh` probabilistic tags (RT5/T4.2):** the `mesh` implementation RFC
must be explicit about which gossip protocol it commits and what `ProbabilityBound` that
protocol gives. The T4.2 literature (Demers et al., HyParView, Plumtree) gives `Empirical`
bounds for specific topologies and parameter choices. A generic "probabilistic" tag with no
committed protocol is `Declared` at best — and the implementation RFC must not advance to
Accepted until at least `Declared`-with-protocol (the specific protocol and its literature
basis are named, even if the in-repo measurement is deferred). This is a VR-5 requirement
for the `mesh` RFC: the bound is not `Exact` (no closed-form proof for realistic networks)
and must not be presented as one.

---

## 6. The per-construct RFC vehicle plan

Each R2 construct needs its own implementation RFC (RFC-0008 §4.5 status rule: "activation
still requires an implementation RFC committing each construct's typing and elaboration per
RFC-0006 §4.3"). This note recommends treating them as six separate RFCs (rather than one
omnibus R2 RFC) because the dependency ordering (§4) makes a monolithic RFC either
over-committed (activating all six at once before their research gaps are closed) or
under-committed (deferring activation of all six waiting for the longest-lead item).

**Recommended RFC vehicles (`Declared` recommendation, maintainer decision):**

| RFC | Construct | Scope | Primary prerequisite |
|---|---|---|---|
| RFC-00XX-forage | `forage` | Third RFC-0005 policy site; placement annotation; RT2 differential for placement | R1 done (M-667) |
| RFC-00XX-backbone | `backbone` | Transport-path declaration; forage policy input; fallback semantics | RFC-forage; mesh transport interface |
| RFC-00XX-mesh | `mesh` | Gossip/pub-sub overlay; `ProbabilityBound` declaration; scoped lifetime; v0 protocol choice | R1 done; DN-61 B.1/B.2 research |
| RFC-00XX-graft | `graft` | Capability contract; affine substrate handle; `graft` manifest in spore | RFC-0028 §7 follow-on |
| RFC-00XX-xloc | `xloc` | Fallible value translocation; wire-format swap; `CapabilityError` variant | RFC-mesh; RFC-graft; R1 xloc stub |
| RFC-00XX-cyst | `cyst` | Content-addressed checkpoint; dormability gate; reclamation in dormancy | RFC-xloc; RFC-0027 OQ-3 |

RFC numbers are not assigned here — the orchestrator assigns RFC numbers to avoid collision
(issues.yaml constraint — see swarm mitigation #1 in CLAUDE.md).

---

## 7. R2 gated on R1 completion (M-667) — explicit statement

Per RFC-0008 §4.6: "R1 before R2 is load-bearing: every R2 guarantee is stated against R1's
deterministic core." And per E7-2 dependency ordering in issues.yaml: M-668 depends on M-667.

**No R2 implementation RFC should advance to Accepted before M-667 is done.** This is a hard
gate, not a soft preference. The planning in this note (§3–§6) is preparatory work; research
passes for `mesh` (DN-61 B.1/B.2) may run in parallel with M-667 completion, but the
implementation RFCs for any R2 construct must not land before R1 is complete.

M-667 is the gate for: `fuse`/`reclaim`/`tier` activation (from the R1 side) and all six
R2 activation tracking entries in RFC-0008 §4.5.

---

## 8. Open questions (FLAGS — for orchestrator and maintainer)

Consolidated list of all FLAGs raised in §3 and §4:

| FLAG | Section | What it is | Who must decide |
|---|---|---|---|
| FLAG-1 | §3.1 | Can a `wild`-backed value cross a node boundary via `xloc`? | RFC-0028 §7 follow-on / `graft` implementation RFC |
| FLAG-2 | §3.1 | Does `xloc` require a declared `backbone`, or does it fall back to `mesh`? | `xloc` implementation RFC |
| FLAG-3 | §3.1 | Wire-format swap story for structured values — explicit `Swap` or implicit? | `xloc` implementation RFC; RFC-0002 extension |
| FLAG-4 | §3.2 | Which gossip protocol is the v0 mesh engine? | `mesh` implementation RFC (needs research pass T4.2) |
| FLAG-5 | §3.2 | Do multi-producer `mesh` topics require `Fuse` semilattice compliance? | `mesh` implementation RFC; interacts with DN-58/RFC-0023 R23-Q2 |
| FLAG-6 | §3.2 | Byzantine-safe `mesh` variant — separate type or a flag? | G1 surface team; maintainer before `mesh` RFC Accepted |
| FLAG-7 | §3.3 | Reclamation of values in a dormant `cyst` — strong vs content-addressed ref? | RFC-0027 OQ-3 resolution; `cyst` implementation RFC |
| FLAG-8 | §3.3 | Is dormability restricted to `matured` scopes or a weaker predicate? | `cyst` implementation RFC |
| FLAG-9 | §3.3 | Continuation representation — L0 bytecode or higher-level IR? | `cyst` implementation RFC |
| FLAG-10 | §3.4 | Is `graft` a keyword or a macro/attribute? | Syntactic decision; `graft` implementation RFC |
| FLAG-11 | §3.4 | Does `graft` require a per-node capability manifest; does `xloc` carry a capability check? | RFC-0028 §7b resolution; `graft`/`xloc` implementation RFCs |
| FLAG-12 | §3.4 | Can two `graft`-acquired handles of the same kind be `fuse`-merged? | `graft` implementation RFC; LR-8 interaction |
| FLAG-13 | §3.5 | What node-level `Meta` signals are available to `PlacementPolicy`? | `forage` implementation RFC |
| FLAG-14 | §3.5 | What is the result when the `PlacementPolicy` has no valid node candidate? | `forage` implementation RFC |
| FLAG-15 | §3.6 | Is `backbone` a per-phylum manifest declaration or a runtime dynamic declaration? | **Maintainer decision needed** — the RFC-0008 §4.5 description ("declared/promoted") allows both |
| FLAG-16 | §3.6 | Is `BackboneRef` affine or freely shareable? | `backbone` implementation RFC |

**Orchestrator FLAGs:**
- **FLAG-O1:** `CHANGELOG.md`, `docs/Doc-Index.md`, and `tools/github/issues.yaml` need
  updates to register DN-63. This note does not touch those files — they are orchestrator-owned
  (CLAUDE.md swarm discipline). FLAG to the integrating parent.
- **FLAG-O2:** RFC numbers for the six per-construct implementation RFCs are not assigned here.
  The orchestrator must assign RFC numbers (checking against the current RFC list to avoid
  collision — swarm mitigation #1).
- **FLAG-O3:** The `mesh` implementation RFC has the longest research lead time (DN-61 B.1
  clock/R8-Q3 and B.2 Byzantine/R8-Q4 research passes). The orchestrator should consider
  starting the clock/Byzantine research pass as a parallel workstream immediately, not waiting
  for the `forage`/`backbone` RFCs to land first.

---

## Definition of Done

This DN is **Accepted** when:

1. The maintainer ratifies the per-construct decomposition (§3), the dependency ordering (§4),
   and the recommended RFC vehicle plan (§6) as the R2 implementation strategy.
2. Each open question in §8 is flagged to the appropriate vehicle (not resolved here).

This DN is **Resolved** when:

- All six per-construct implementation RFCs have reached Accepted (or been superseded by an
  alternative plan), and each has explicitly closed the FLAGs in §8 that belonged to it.

---

## Meta — changelog

- **2026-06-29 — Draft created (M-668).** Initial R2 decomposition planning capture.
  Per-construct typing and elaboration strategies for `xloc` (§3.1), `mesh` (§3.2), `cyst`
  (§3.3), `graft` (§3.4), `forage` (§3.5), `backbone` (§3.6) — all `Declared`, planning-stage
  only. Dependency ordering (§4) and honest guarantee-tag inventory (§5). Recommended
  per-construct RFC vehicle plan (§6). R2 gated on R1 completion (M-667) stated explicitly (§7).
  Sixteen construct-level FLAGs (FLAG-1..FLAG-16) and three orchestrator FLAGs (FLAG-O1..FLAG-O3)
  surfaced. Grounded in RFC-0008 §4.5/§4.6, DN-61 (B.1–B.5 agenda), DN-59, RFC-0005, RFC-0028,
  RFC-0027, RFC-0023 R23-Q2, T4.2–T4.5. Advisory; append-only. No shared file touched; owned
  by the integrating parent.
