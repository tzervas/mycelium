# Design Note DN-60 — Effect-System Surface Direction (Phase-2)

| Field | Value |
|---|---|
| **Note** | DN-60 |
| **Status** | **Draft** (2026-06-28) — Phase-2 direction-capture only. Records the design direction for enriching the effect system past the v0 declared-set. **NOT a ratification of any decision.** All directions here are `Declared` (design intent, not theorem). A future RFC (extends RFC-0014) is the ratification vehicle. |
| **Feeds** | the G6 group in the Blocked-Decisions Ratification Map · RFC-0014 (Phase-2 follow-on RFC) · RFC-0008 R8-Q2 · RFC-0018 §9 graded-effect interaction |
| **Date** | 2026-06-28 |
| **Grounding** | RFC-0014 §4.5/§4.8/§8/§9 · RFC-0008 §4.5/§6/§8 R8-Q2 · RFC-0018 §9/§4.5 note · Ratification Map §G6 |

---

> **Posture (VR-5 / G2).** This is a **direction-capture note**, not a decision document. Every
> direction recorded here is `Declared` — a design intent, not a theorem. Nothing here advances
> any RFC status, activates any surface, or commits any implementation. The honest purpose:
> prevent deferred effect-system decisions from being lost between Phase-1 (v0, Enacted) and
> Phase-2 ratification. A future RFC-0014 revision is the ratification vehicle; **that RFC** will
> carry the normative decisions, proofs, and Definition of Done. Disconfirming evidence and open
> tensions are flagged where found.

---

## 1. Context and scope

RFC-0014 is **Enacted** with a deliberately coarse v0 effect system: a **manually-declared effect
set** on signatures, a **compositional coverage check** (declared ⊇ performed, propagated up the
call graph), and a **runtime budget ledger** (`mycelium_interp::budget`, M-353) enforcing overruns
gracefully as `EffectBudgetExhausted`. RFC-0014 §8 resolved the v0 questions and explicitly
deferred richer effect typing to §9 ("Future possibilities"):

> *"Richer effect typing — effect rows / polymorphism / inference, **only** if it never lets an
> effect become implicit or unbounded (would extend, not weaken, I3–I5)."*

Three deferred directions from §8/§9 form the G6 group of the Ratification Map:

| G6 sub-question | RFC-0014 locus | RFC-0008 locus |
|---|---|---|
| **D1 — Dynamically-resolved effect budgets** | §4.8 / §8 "budget vocabulary" | §4.7 C1 (per-task budgets) |
| **D2 — Effect-row polymorphism / minimal-set inference** | §8 "effect inference" / §9 "richer effect typing" | §6 R8-Q2 adjacent (T3.4 growth path) |
| **D3 — Hypha-creation in the effect row** | §9 "concurrency interaction" | §8 R8-Q2 (explicit open question) |

**What this DN is not.** It does not resolve D1–D3. It does not move RFC-0014 toward a new
status. It records direction and surfaces the questions a Phase-2 RFC must answer. The G6 group is
**MED priority, not 1.0-gate** (KISS/YAGNI: the v0 system is sufficient for core-1.0.0; richer
effects unblock effect-polymorphic stdlib APIs and full concurrency/effect integration, both
Phase-2 concerns per the Ratification Map).

---

## 2. Direction D1 — Dynamically-resolved effect budgets

### 2.1 Current state (v0, Enacted)

The v0 effect system tracks a **set** of effect *kinds* (`retry`, `alloc`, `io`, `cascade`,
`time`, user-declared `Named`). Budgets — the quantitative limits (`retry(<=3)`,
`alloc(<=64KiB)`) that make RFC-0014 I4 meaningful — exist **only in the runtime ledger** (M-353).
The L1 stage-1 frontend (M-660) parses the **effect set** only; it does not parse per-effect
budget syntax. This split is recorded in RFC-0014 §3.4 (append-only note, 2026-06-22):

> *"The budgets in the example (`retry(<=3)`, `alloc(<=64KiB)`) are **not** part of the stage-1
> *surface* yet — the frontend parses the **effect set** only; budget enforcement stays the M-353
> `mycelium_interp::budget` runtime ledger, and a per-effect budget syntax is later work."*

The runtime budget ledger already uses **dynamically-resolved** budgets (DN-05 §2.4 precedent): a
budget may be resolved from a static default or from a memory-derived ceiling at runtime. The
mechanism is implemented (M-353); the surface form is not yet specified.

### 2.2 Direction: surface budget syntax as a Phase-2 RFC-0014 revision

The direction is to **lift per-effect budget bounds into the signature surface**, so the
illustrative form in RFC-0014 §3.4 becomes normative:

```text
fn save(r: Record) -> Result<Unit> !{ retry(<=3), alloc(<=64KiB) } = ...
```

Motivation (RFC-0014 I3 completeness): the "no undeclared effects" check can verify the *kind*
but not the *magnitude* if budgets stay runtime-only. A caller cannot statically see *how bounded*
a callee's effects are. Surface budget syntax completes the transparency story by making bounds
visible at the call site — consistent with SC-3 (no black box) and the existing budget vocabulary
in RFC-0014 §4.5 I4 ("every effect that could be unbounded carries an explicit budget").

**Options for budget resolution at the surface (direction, not decision):**

| Option | Description | Tension |
|---|---|---|
| **O1 — Static literals only** | `retry(<=3)` is a compile-time constant | Simplest (KISS). Cannot express policies where the budget derives from a runtime signal (DN-05 pattern). |
| **O2 — Budget parameters** | `retry(<=n)` where `n` is a compile-time parameter passed by the caller | More expressive. Aligns with DN-05's dynamically-resolved approach. Requires budget parameters in fn signatures — surface area cost. |
| **O3 — Policy reference** | `retry(<= budget_policy.retries)` — references a reified RFC-0005 policy | Maximal flexibility. Highest surface complexity. Most YAGNI risk; deferred-most. |

**Direction (Declared):** O1 first; O2 as a follow-on if DN-05-style dynamic resolution is needed
at the surface. O3 is a §9-class future. A Phase-2 RFC must decide which option(s) to adopt and
how budget parameters interact with trait method effects (the M-660 coverage check already enforces
impl-must-equal-trait for effect *sets*; budget parameters would tighten or complicate this).

**Open question (Declared) for the RFC:** Budget composition is not resolved by the v0 model. If
a fn with `retry(<=3)` calls a fn with `retry(<=2)`, does the outer require `retry(<=5)` (sum)
or `retry(<=3)` (outer absorbs)? The answer interacts with the per-task budget scoping in
RFC-0008 §4.7 C1 and must be decided in the Phase-2 RFC with a concrete semantic.

### 2.3 Per-task budget interaction (RFC-0008 §4.7 C1)

RFC-0008 §4.7 C1 (Enacted, M-356) established that each hypha instances its own budget ledger.
A surface budget annotation on a fn that spawns a hypha must distinguish between "the budget this
fn itself consumes" and "the budget the spawned hypha runs under." Whether this is a single
annotation or two separate concerns is **open for the Phase-2 RFC**.

Direction (Declared): keep them separate. Per-hypha budget is a runtime policy (an argument to
the spawn form, or a default from the enclosing scope), not a per-fn signature annotation. Merging
them would conflate the fn-effect budget (a static property of a fn's body) with the hypha
lifetime budget (a dynamic property of a created task). This separation is consistent with
RFC-0008 §4.7's design of keeping "scheduler" distinct from "composition semantics."

---

## 3. Direction D2 — Effect-row polymorphism and minimal-set inference

### 3.1 Current state (v0, Enacted)

v0 effect inference is **manual-declare + compositional-check** (RFC-0014 §8): a developer writes
the effect set explicitly; the checker verifies declared ⊇ performed (union of callee effects);
under-declaration is a `CheckError`. The checker *never infers* an undeclared effect. This is the
deliberate v0 choice (KISS/YAGNI, RFC-0014 §8):

> *"True minimal-set inference (computing a minimal effect set) is deferred to §9."*

The effect *surface* sits at `T3.4`'s "bit → small fixed row" growth path (RFC-0008 §6,
"Why not effect-handler concurrency"): a small fixed declared set, not a full row type. RFC-0014
§9 records "richer effect typing — effect rows / polymorphism / inference" as a future possibility
**only if** it never makes an effect implicit or unbounded.

### 3.2 D2a — Effect-row polymorphism (generic-over-effects)

The goal: functions polymorphic in their effects — a higher-order fn `map` that carries "whatever
effects its argument carries":

```text
fn map<T, U, !E>(f: fn(T) -> U !{!E}, list: List<T>) -> List<U> !{!E} = ...
```

Without this, a HOF must conservatively over-declare its effects (declaring the union of all
possible argument effects), or be restricted to pure arguments — limiting effect-polymorphic
stdlib APIs (the G6 group's "unblocks" in the Ratification Map).

RFC-0018 §9 notes this as the forward direction:

> *"If a full effect system is added later (the growth path in RFC-0006 §8 Q4), grades and effects
> combine naturally in a graded effect system (the Koka + Granule intersection; Orchard & Petricek
> 2014 as the theoretical basis)."*

**Options:**

| Option | Description | Tension |
|---|---|---|
| **O1 — Effect variables** | `!E` as a universal row variable, instantiated at each call site | Most expressive. Requires extending the type system with effect variables; interacts with G4 (polymorphism/trait surface). KC-3 risk if rows require L0 support. |
| **O2 — Bounded effect quantification** | `!{!E, io}` meaning "at least `io`, and polymorphic over `!E`"; bounded row polymorphism | Weaker but simpler. No open rows; only bounded quantification over effect sets. |
| **O3 — Named effect groups** | Named sets as named effect "kinds"; polymorphism via kind parameters | Minimal surface change. Less expressive for composition; limits HOF ergonomics. |

**Direction (Declared):** O1 is the target direction, deferred to Phase-2. O2 is an acceptable
simpler alternative. O3 is a fallback if O1's KC-3 cost is prohibitive.

**Load-bearing constraint (flag):** RFC-0018 §4.5 records a precondition for Design A's
noninterference result (`research/09` T9.6):

> *"Design A's sufficiency rests on the calculus being pure; when observable effects land, they
> must become graded outputs (RFC-0014, route i) or carry a local `pc` (route ii)."*

A Phase-2 RFC adopting D2 must provide a soundness argument for how this precondition is
preserved under effect-row polymorphism. The argument cannot be `Declared`; it requires at least
`Empirical` grounds (trials + the existing noninterference argument extended) and ideally a
checked proof for a `Proven` tag. This is a non-trivial obligation — flag it explicitly.
`Declared` direction; `Empirical`-or-better required for the Phase-2 RFC to be ratifiable.

### 3.3 D2b — Minimal-set inference

**Goal:** compute the minimal effect annotation from the call graph rather than require manual
declaration. The v0 checker already checks compositionally; inference would *derive* the set
automatically.

**Tension with RFC-0014 I3:** I3 requires effects to be visible in signatures. Inference-derived
sets are still visible (the inferred annotation is the signature), but a developer did not write
them — the set appears by checker derivation. RFC-0014's KISS/YAGNI rationale for deferring
inference was explicit: "explicit is honest." Inference is not dishonest if the result is always
surfaced, but it weakens the "I wrote what my fn does" property.

**Direction (Declared):** inference-with-shown-output — the checker infers but the inferred set
is always shown explicitly (on inspect/explain). An undeclared effect is never silently permitted.
A developer may still write explicit annotations (checked against the inferred set; explicit over-
declaration remains legal per RFC-0014 I5 "opt-in broader effects"). When inference disagrees with
a manual annotation (e.g., inferred `{io}` but declared `{io, retry}`), the manual annotation
stands (it is a legal over-declaration). The Phase-2 RFC must specify the inference algorithm and
its completeness claim (at what tag: `Empirical` for a flow-analysis, `Proven` only if mechanized).

---

## 4. Direction D3 — Hypha-creation in the effect row (R8-Q2)

### 4.1 The open question

RFC-0008 §8 R8-Q2 states:

> *"Does hypha creation appear in the T3.4 effect row (growth path 'bit → small fixed row'), or is
> structure (RT7) enough? Interacts with RFC-0007's stage-1 grading."*

The context (RFC-0008 §6, "Why not effect-handler concurrency"):

> *"The effect-typing *surface* (does `spawn` appear in an effect row?) is deliberately open
> (R8-Q2)."*

RFC-0008 §4.7 (Enacted, M-356) provided the runtime composition semantics (C1–C4) without
resolving R8-Q2: per-task budgets, cooperative cancellation, `TaskOutcome`, and bounded-cascade
supervision are all in place. The open question is whether a function that *creates* a hypha must
declare that in its effect annotation.

### 4.2 Options and their tradeoffs

**Position A — Hypha-creation is an effect kind (`spawn`):**

```text
fn launch_worker(data: Data) -> Handle !{ spawn, io } = ...
```

A function that creates a hypha declares `spawn`. Composition: a caller of `launch_worker` must
also declare `spawn` (I3's compositional propagation). The benefit: the effect annotation tells a
caller whether a function can create concurrent tasks — maximal transparency (SC-3). The cost:
every fn in the call chain above a spawning fn must declare `spawn`, even thin wrappers (the
RFC-0014 §6 "declared effects can be verbose" drawback, amplified for deeply-nested call graphs).
`spawn` is a creation effect, not a resource-consumption effect; it does not naturally carry a
budget bound the way `retry(<=N)` does. A "max tasks spawned" budget would be a runtime resource
policy, not a per-fn annotation in the same sense as `alloc(<=64KiB)`.

Interaction with RFC-0018 §4.5 Design A: if `spawn` is an effect, the Phase-2 RFC must show that
observable-effects land as graded outputs (route i) or carry a local `pc` (route ii). RFC-0008
§4.7 C3 already establishes `TaskOutcome` as the graded output of a spawned hypha's result — this
is route i, already in place. Position A is therefore consistent with the Design A precondition
via the existing `TaskOutcome` mechanism.

**Position B — Hypha-creation is captured by structure (RT7) alone:**

RT7 (RFC-0008 §4.1) enforces that every hypha is inside a structured scope; an orphaned hypha is
not expressible. From the caller's perspective, `spawn` is always paired with a scope-join, and
RT7's structural discipline makes task creation a language-level invariant rather than an effect
to track per-fn. Benefit: simpler effect annotations; no `spawn` propagation up the call chain.
Cost: the effect surface does not tell a caller whether a function can spawn tasks — they must
consult documentation or the scope-creation API.

**Position B+ (hybrid):** hypha-creation is **not** a per-fn effect kind, but a scope-level
declaration. A `colony { ... }` block (DN-06, RFC-0008 §4.7) or equivalent structured-concurrency
scope is itself a syntactic scope that *enables* spawning within it. This makes the spawning
context visible at the boundary where it matters (the scope, not every transitively-calling fn)
without requiring `spawn` to propagate through the entire call graph.

### 4.3 Direction (Declared)

**Preferred direction: Position B+.** RT7 already provides the structural invariant; adding
`spawn` as a per-fn effect kind risks verbose propagation. The scope-level declaration makes
concurrency explicit at the structural boundary. Under B+:

- `spawn` does not appear in the `!{…}` effect row of individual functions.
- The structured scope (colony / scope block) is the syntactic boundary that enables spawning.
- The RFC-0018 §4.5 Design A precondition is already satisfied by `TaskOutcome` (route i,
  RFC-0008 §4.7 C3) — B+ does not disturb this.
- The `cascade` effect kind (already in v0 — RFC-0014 §4.5) bounds re-entrant spawning patterns;
  B+ does not add a new budget concern for task creation.

**Open tension (flag):** If D2 (effect-row polymorphism, O1) is adopted, the interaction must be
addressed: under B+, `spawn` is not a row element and cannot appear in `!E`; under A, it can. The
Phase-2 RFC must decide D2 and D3 coherently, or explicitly scope one out and record why.
`Declared`.

**Interaction with RFC-0018 grading under B+:** The spawned task's graded `TaskOutcome` (already
established by RFC-0008 §4.7 C3) is the graded output of the concurrent computation. Under B+,
no additional grading rule for `spawn` is needed at the fn level; the grade of the scope's result
is the meet of the `TaskOutcome`'s grades (RFC-0008 RT6 / RFC-0018's meet-composition rule).
This is consistent with RFC-0018 §4.5 Design A. `Declared` — grounding at `Empirical` or better
is the Phase-2 RFC's obligation.

---

## 5. Phase-2 RFC — Definition of Done

A future RFC-0014 revision is the ratification vehicle. This section records the DoD — the
questions that RFC must answer, with what evidence strength, to be ratifiable. It is `Declared`
now; the RFC itself must ground each point at `Empirical` or better.

**DoD items (what the Phase-2 RFC must decide, per house rule #6):**

1. **D1 — Budget surface:** Which option (O1/O2/O3)? Normative surface spelling? How do budget
   parameters interact with the M-660 trait/impl coverage check? How do per-fn budgets compose
   up the call graph (sum, outer-absorbs, or another rule)? Grounding: `Empirical` minimum
   (operational semantics for budget composition).

2. **D2a — Effect-row polymorphism:** Adopted or not? If adopted, which option? Soundness
   argument that row polymorphism does not let an effect become implicit or unbounded (RFC-0014
   I3–I5 must hold). Interaction with RFC-0018 §4.5 Design A precondition (must be discharged
   at `Empirical`-or-better, not `Declared`). KC-3 check: zero L0 cost.

3. **D2b — Minimal-set inference:** Adopted or not? If adopted: inference algorithm, completeness
   tag (Empirical/Proven), what the developer sees when inferred differs from manual, and whether
   explicit annotations override or are checked against the inference.

4. **D3 — Hypha-creation in the effect row:** Position A, B, or B+? If B+, what is the surface
   form for a spawn-enabling scope? RFC-0018 Design A precondition discharge. Are D2 and D3
   jointly decided (preferred) or scoped independently with a stated rationale?

5. **Invariant preservation (non-negotiable):** RFC must verify that every adopted direction
   satisfies RFC-0014 I3 (no undeclared effects), I4 (budget overrun is explicit and graceful),
   and I5 (default tightly scoped; broader is opt-in). These may not be weakened.

6. **KC-3:** Effect rows and inference must remain checker/surface metadata — zero new L0 nodes.
   If any direction requires L0 support, it must be explicitly justified as a KC-3 exception and
   go through the kernel-freeze gate (DN-56 §4, condition #3 / ADR-033 precedent).

---

## 6. What this DN does NOT decide

- **No status change for RFC-0014.** It remains `Enacted`; this DN does not propose amending it.
- **No surface activation.** The `!{…}` v0 surface (M-660) is the only active surface; everything
  in §2–§4 above is deferred direction.
- **No implementation obligation.** The directions above are Phase-2 targets; no crate, test, or
  issue is spawned by this note alone.
- **Doc-Index / CHANGELOG / issues.yaml.** This DN does not update those files.
  **FLAG (for orchestrator / owning parent):** `docs/Doc-Index.md`, `CHANGELOG.md`, and
  `tools/github/issues.yaml` need to be updated to register DN-60 and record the G6
  direction-capture step. These are orchestrator-owned files per swarm discipline; this agent
  treats them as read-only.

---

## Meta — changelog

- **2026-06-28 — Draft.** Created to capture the Phase-2 direction for the G6 group
  (Blocked-Decisions Ratification Map §G6) of deferred effect-system decisions: **(D1)** surface
  budget syntax for per-effect bounds (RFC-0014 §3.4 / §4.8 deferred); **(D2)** effect-row
  polymorphism and minimal-set inference (RFC-0014 §8/§9 deferred); **(D3)** hypha-creation in
  the effect row (RFC-0008 R8-Q2). All directions `Declared`. Not a ratification; the vehicle is
  a future RFC-0014 revision. Grounded in RFC-0014 §4.5/§4.8/§8/§9, RFC-0008 §4.7 C1/§6/§8
  R8-Q2, RFC-0018 §4.5 note/§9. Key flags: RFC-0018 Design A soundness precondition must be
  discharged by the Phase-2 RFC at `Empirical`-or-better; D2 and D3 must be jointly decided (or
  independently scoped with rationale). Doc-Index/CHANGELOG/issues.yaml update flagged to
  orchestrator. Append-only.
