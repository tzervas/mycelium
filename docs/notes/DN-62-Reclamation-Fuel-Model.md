# Design Note DN-62 — Reclamation Fuel Model (Drop-Latency / Pause-Budget Research Note)

| Field | Value |
|---|---|
| **Note** | DN-62 |
| **Status** | **Draft** (2026-06-28) — research and measurement note. Commissioned by **DN-59** (the ratified reclamation-strategy note that left the drop-latency / fuel-bound axis open) to attempt lifting RFC-0027 OQ-3's drop-latency SLO from `Declared` to `Empirical`. *FLAG-1: DN-59 is not present on this branch's `origin/dev` base — its specific section numbers cannot be verified here; the verified commission anchor is RFC-0027 §11 OQ-3 (`RFC-0027:493`).* No guarantee is upgraded here; the upgrade is the *goal* of the measurement plan (§4). Promotion to **Resolved** requires the §5 Definition of Done. |
| **Feeds** | **DN-59** (the commissioning note — the one genuinely-open drop-latency SLO / fuel-bound axis; *see FLAG-1: not on this `origin/dev` base*); **RFC-0027 §11 OQ-3** (mitigated-not-closed, verified `RFC-0027:483-494`: "Can the fuel model bound the cascade, lifting latency `Declared → Empirical / Proven`? … A dedicated research note is warranted."); the forward build epic **E12 Increment 3** (real env-machine reclamation — DN-35 §9). |
| **Date** | 2026-06-28 |
| **Decides** | When resolved: (a) whether a per-epoch fuel budget can bound RC-cascade drop latency in the deferred-reclamation model (Layer-3, DN-32 §2.3); (b) what the concrete fuel unit is (nodes reclaimed / RC decrements / region sweeps) and how the budget is computed; (c) what pass/fail SLO thresholds — if any — can be honestly tagged `Empirical` rather than `Declared`; (d) whether "no silent GC pause" can be promoted from an honesty stance to a measured latency bound, or whether `Declared` is the durable honest tag. This note **moves no other doc's status** and **enacts no code**. |
| **Task** | commissioned (DN-59 — see FLAG-1; verified anchor RFC-0027 §11 OQ-3) |

> **Posture (transparency rule / VR-5 / G2).** This note is a **research framing**. Every claim in it
> is either a **confirm-the-record** re-statement of an already-Accepted decision (held at its source
> strength — RFC-0027/DN-32/DN-35/DN-59) or a **`Declared`-tagged design proposal** (a reasoned design
> intent, not a measured result). The SLO is `Declared` and stays `Declared` until the §4 measurement
> plan lands. "No silent GC pause" is ratified as a G2/EXPLAIN **honesty commitment** — it is **not** a
> latency bound, and this note must not be read as asserting one. Quantitative figures in §3
> (fuel-unit candidates, budget exemplars) are **`Declared`** (design intent); they become `Empirical`
> only after the §4 workloads are run and the §5 gate is passed. VR-5: do not upgrade any claim past
> its checked basis.

---

## §1 The question

**What bounds per-epoch deferred-reclamation work so that drop-latency is honest, not a silent GC
pause?**

The root problem is RFC-0027 §7.2: each `rc_dec` is O(1), but dropping a deep value tree is O(*n*)
in its node count — a **bounded-but-large** pause, not a sub-millisecond one. "Bounded" here means
*deterministic, observable, and EXPLAIN-able* (the §9 record — G2), **not** constant-time. So "no
silent GC pause" is satisfied as an **honesty stance** only: every reclamation step is observable, but
there is no commitment on *how long it takes*.

DN-32 Layer 3 (region-based allocation + **batched scope-exit reclamation**) and the sweep-epoch
model mitigate this by deferring and spreading the O(*n*) cost across epochs (DN-32 §2.3/§9 R-4;
RFC-0027 §12). The mitigation moves the worst case from an *inherent limitation* toward an
*engineering/measurement* problem:

- **Deferred reclamation** (batched scope-exit) avoids a single O(*n*) drop on the critical path;
  the cost is paid at scope-exit, not at the final `rc_dec`.
- **Sweep-epoch spreading** divides the scope-exit reclamation budget across epochs, so each epoch's
  work is bounded by the fuel budget rather than the full live-set size.

**The open question (RFC-0027 §11 OQ-3):** Can a *fuel model* make that per-epoch bound explicit,
honest, and measurable — lifting the latency SLO from `Declared` (design intent) to `Empirical`
(measured, reproducible, with a recorded pass/fail threshold)?

RFC-0027 §11 OQ-3 is explicit (verified `RFC-0027:491-493`): "Can the fuel model (RFC-0014 §4.8) bound
the cascade, lifting latency `Declared → Empirical / Proven`?" and "A dedicated research note is
warranted." This is that note. (DN-59 commissioned it; *FLAG-1 — DN-59's own text is not on this base,
so the verified commission anchor is OQ-3 above, not a DN-59 subsection.*)

**Honest scope.** This note does *not* claim the fuel model works. It frames the candidate model,
states what must be measured to evaluate it honestly, and identifies the pass/fail criterion that
would justify a tag upgrade. Until §4 runs and §5's gate is cleared, the SLO is honestly `Declared`.

---

## §2 Background — what the existing cluster already settled

This section is **confirm-the-record**: re-stating Accepted decisions at their source strength (VR-5
— no upgrade). DN-62 does not re-decide these; it builds on them.

### §2.1 The reclamation mechanism is RC (RFC-0027, Accepted)

Precise reference counting (§7.1). Each `rc_dec` is O(1) — `Exact` by Rust specification. A
cascade drop is O(*n*) in node count — deterministic, but bounded-but-large. "No cycle collector" —
LR-9 acyclicity removes the need (`Proven`-modulo-LR-9 at source; Perceus Thm 2). The reclamation
EXPLAIN/audit record (RFC-0027 §9) is the never-silent mechanism: every reclamation event has a
`scope_id`, `sweep_epoch`, `trigger ∈ {RcZero, ScopeExit, ChannelClose}`, `value_meta_hash`,
optional `channel_id`.

### §2.2 The three-layer hybrid — Layer 3 is the key (DN-32, Accepted)

DN-32 §2.3 Layer 3: **region-based allocation + batched scope-exit reclamation** (Tofte–Talpin
mapped onto the RT7 scope tree). Values within a scope are region-allocated; at scope-exit, the
region batch-frees in one sweep. This converts a cascade of individual `rc_dec` calls into a
**single ScopeExit sweep** — the unit of work the fuel model must bound.

The sweep-epoch model (DN-32 §9 R-4): deferred reclamation can *spread* the ScopeExit sweep across
epochs. Instead of one large ScopeExit sweep at scope-close, the sweep runs incrementally, epoch by
epoch, each epoch consuming a bounded fuel quantum.

### §2.3 The "no silent GC pause" stance (RFC-0027 §1, ratified; carried by DN-59)

RFC-0027 §1 (verified `RFC-0027:67`,`:123-124`) codifies — and DN-59 (the commissioning note) carries
forward — that **"No silent GC pause" is an honesty stance, not a latency SLO.** This means every reclamation event must be `EXPLAIN`-recorded (G2) — but it does *not*
mean the event completes in sub-millisecond time. The acceptable pause is a *bounded, logged,
never-silent* event (RFC-0027 §3/§9). Any future construct introducing a silent pause must supersede
RFC-0027 (append-only). The SLO is `Declared` until a methodology'd benchmark exists.

### §2.4 The RC ⊕ region exactly-once coupling (DN-35 §6, Accepted direction)

`RcZero` is the eager trigger; `ScopeExit` is the batch trigger — they must not both free a cell.
The double-free guard: a cell freed by `RcZero` is removed from its region's free-list immediately.
The missed-free guard: `ScopeExit` batch-frees only cells that are region-internal with zero
cross-region references. Escaped cells (rc > 0 via cross-region reference) are promoted to the
parent region, never freed. This is the `Empirical`/`Declared` coupling protocol (DN-35 §6 — Gay–
Aiken design, no combined mechanized proof yet).

---

## §3 The candidate model — per-epoch fuel budget (`Declared`)

> **Tag throughout this section: `Declared`.** Every item in §3 is a design proposal — argued design
> intent, not a measured or proven result. Numbers (where given as exemplars) are illustrative, not
> measured bounds. They become `Empirical` only after the §4 measurement plan is executed.

### §3.1 What "fuel" counts

A fuel unit is the *granularity at which the budget is consumed* during a deferred-reclamation epoch.
Three candidate fuel units, each with a different granularity–overhead trade-off:

| Candidate unit | What it counts | Granularity | Overhead to track |
|---|---|---|---|
| **(A) Nodes reclaimed** | one unit per value cell freed (`RcZero` or `ScopeExit`-freed) | fine-grained | `Exact` (the `ReclamationRecord` already counts this) |
| **(B) RC decrements** | one unit per `rc_dec` call (leaf-level, before cascade) | very fine | adds a counter to the hot path |
| **(C) Region sweeps** | one unit per region-level `ScopeExit` batch (coarser) | coarse | minimal (one counter per region close) |

**`Declared` recommendation for R1:** candidate **(A) — nodes reclaimed** — is the natural choice
given the existing `ReclamationRecord` machinery (RFC-0027 §9 / MEM-1): the record already emits
one entry per reclaimed value, so the fuel counter is already implicit in the record count. Switching
from "count records" to "bound the record count per epoch" requires no new instrumentation on the hot
path. Candidate (B) adds overhead without commensurate benefit at this scale; candidate (C) is too
coarse to bound tail latency when a single region sweep can still be O(*n*) for a large scope.

The fuel unit is **`Declared`**: the measurement plan (§4) must validate that nodes-per-epoch is the
right proxy for actual drop latency, not just reclamation record count.

### §3.2 How the budget is computed

A per-epoch fuel budget *F* must be chosen such that: consuming *F* nodes of reclamation work
per epoch produces a drop-latency distribution whose tail (P99/P999) falls within the target
threshold *T* (to be determined by §4).

**`Declared` candidate budget approach:**

1. **Fixed budget.** *F* is a constant configured at runtime (e.g., the number of cells the
   reclaimer may free in a single epoch tick). Simple; does not adapt to allocation rate. Risk: under
   high allocation pressure, the fixed budget may not keep pace, leading to deferred-reclamation
   debt accumulating across epochs.

2. **Proportional budget.** *F* is proportional to the number of live allocations in the epoch's
   region (e.g., *F = α × region_size* for some constant *α ∈ (0, 1]*). Adapts to workload size;
   bounded per-epoch by the region count; but requires tracking live allocation count per region.

3. **Latency-feedback budget.** *F* is adjusted dynamically based on observed per-epoch reclamation
   latency — increase *F* when the epoch completes fast (under budget), decrease when it is slow
   (over budget). This is a feedback loop, not a static bound; it introduces adaptation overhead and
   the risk of oscillation under bursty workloads.

For R1 the **`Declared` starting point is a fixed budget** — least mechanism, most auditable, and
testable against a static threshold. The measurement plan (§4) should evaluate whether a fixed budget
is sufficient or whether adaptation is required.

### §3.3 How fuel exhaustion is handled

When the epoch's fuel budget is exhausted before the reclamation work is complete, two policies:

**(A) Deferral to next epoch.** Remaining reclamation work is re-queued for the next sweep epoch.
The current epoch completes promptly (within its latency budget); deferred work carries over.
**Risk:** under sustained high allocation/drop rate, deferred debt can grow unboundedly —
*reclamation debt*. The fuel model must include a debt-monitoring mechanism (the `sweep_epoch`
counter is already available for tracking epoch lag) so that unbounded deferral does not silently
accumulate.

**(B) Partial sweep (bounded in-epoch work, remainder orphaned until next epoch).** Same as (A)
but makes the "partial" nature explicit in the `EXPLAIN` record: the epoch's `ReclamationRecord`
stream includes a `trigger: ScopeExit(partial)` variant indicating that the region sweep was
interrupted by fuel exhaustion. The next epoch picks up the remainder. This is the
**`Declared`-preferred R1 policy** — it is never-silent (G2): the partial sweep is recorded, debt is
observable, and the accumulation rate is measurable.

**Reclamation debt visibility (G2 obligation).** A fuel-bounded system that silently accumulates
deferred work violates the never-silent stance. The `EXPLAIN` record must surface:
- The number of cells remaining in the deferred queue at epoch close.
- The `sweep_epoch` lag (current epoch minus the epoch at which the cell was first deferred).
- An explicit `Declared`-tagged debt bound (debt remains bounded iff the allocation rate does not
  permanently exceed *F* per epoch — to be validated by §4).

### §3.4 Relation to the existing sweep-epoch machinery

The sweep-epoch model already exists in skeleton form:

- `SweepEpoch` is a `u64` placeholder counter (`reclamation.rs:76`) advanced once per
  `Region::close` (`region.rs:147,154`).
- The `ReclamationRecord` carries `sweep_epoch` as a field (RFC-0027 §9 / `reclamation.rs:148`).
- The `trigger ∈ {RcZero, ScopeExit, ChannelClose}` is exhaustive and never-silent.

The fuel model does **not** require new machinery at the record level — it requires:

1. A **fuel counter** per epoch (nodes processed in this epoch so far).
2. A **budget threshold** *F* (static for R1; measured to set).
3. A **partial-sweep sentinel** in the `ReclamationTrigger` enum (to record exhaustion without
   silently carrying over).
4. A **debt queue** — the ordered list of cells not yet freed, with their queued-at epoch.

None of these are built. All are `Declared` design proposals. The measurement plan (§4) must run
*before* committing to any specific enumeration or counter structure.

---

## §4 The measurement plan — what moves the SLO from `Declared` to `Empirical`

> **Tag:** The *plan* is `Declared`; the *results of running it* will be `Empirical`. The SLO
> remains `Declared` until the plan is executed, the data is recorded, and the §5 gate is cleared.

### §4.1 What to instrument

To evaluate the fuel model honestly, the following must be measurable:

| Instrument | What it measures | Note |
|---|---|---|
| **Per-epoch reclamation latency** | Wall-clock time from epoch start to epoch end (both the reclamation work and the epoch overhead) | Use `std::time::Instant`; record min/mean/P99/P999 across workload runs |
| **Nodes reclaimed per epoch** | The fuel consumption count — nodes freed in a single epoch sweep | Already implicit in `ReclamationRecord` count; need a per-epoch accumulator |
| **Reclamation debt** | Cells in the deferred queue at epoch close | New counter; measures whether debt is bounded under the workload |
| **Epoch lag** | `current_epoch - queued_at_epoch` for each deferred cell | Detects accumulating debt before it becomes unbounded |
| **Peak RSS / heap allocations** | Process-level memory footprint during workload | `jemalloc` allocation stats or `/proc/self/status`; needed to confirm reclamation is *actually reducing* live heap |
| **Tail latency P99/P999** | Distribution tail for per-epoch latency | The primary SLO signal; the pass/fail metric |

**Existing harness.** The `mycelium-bench` harness (used in the DN-35 baseline, 2026-06-26) provides
the execution-backend timing framework. The fuel-model measurement plan builds on this:
- Add a `--reclamation` mode that wires the `ReclamationSink` to a per-epoch accumulator.
- Add heap-allocation counters (not yet wired — flagged in the DN-35 baseline as a follow-up).
- Record the latency distribution (not just min/mean) — the tail is the signal.

### §4.2 The workloads

Three workload classes are necessary to cover the design space of the fuel model:

**W1 — Allocation-heavy (deep value trees).** Construct and drop deeply nested value trees (e.g.,
100-deep binary trees of `Const` nodes). This is the adversarial case for RC-cascade latency: the
O(*n*) drop is maximised. The fuel model must either: (a) bound each epoch's work to *F* nodes and
defer the rest, or (b) demonstrate that the tail latency remains within the target threshold *T*
without deferral.

- *Expected signal:* high per-epoch node count; high variance in epoch latency without fuel bounding;
  bounded variance with fuel bounding (validating the model).

**W2 — Mixed drop patterns (interleaved allocation and reclamation).** Interleave allocations and
drops with varying shapes — some short-lived (dropped in the same epoch they are allocated), some
long-lived (persist across epochs). This stresses the `RcZero`-vs-`ScopeExit` coupling (DN-35 §6):
cells freed by `RcZero` must be removed from their region's deferred queue before `ScopeExit` runs.

- *Expected signal:* the double-free guard is exercised; the missed-free guard fires on escaped
  cells; the epoch debt counter reveals whether mixed patterns cause accumulation.

**W3 — Hypha fan-out (parallel scopes, child→parent reclamation ordering).** Spawn N concurrent
hyphae, each owning a disjoint value set; collect at the parent scope. This exercises the
parent–child TOTAL / siblings CONCURRENT coupling (RFC-0027 §12 / DN-32 §3): siblings reclaim
concurrently, but each sibling's reclamation must complete before the parent scope exits. The fuel
model must not deadlock or starve siblings under a fixed per-epoch budget.

- *Expected signal:* per-hypha epoch latency distribution; whether the fuel budget is sufficient per
  hypha or must be partitioned across the fan-out; whether concurrent reclamation under a shared
  budget causes contention.

### §4.3 The pass/fail criterion

The SLO moves from `Declared` to `Empirical` when:

1. **The latency distribution is recorded across W1/W2/W3** with the fuel model active (fixed budget
   *F*, partial-sweep deferral).
2. **A threshold *T*** — the P99 per-epoch reclamation latency — is **stated before the workloads
   run** (not fitted to the data after the fact, which would be `Declared` re-labelled as
   `Empirical`). *T* is to be determined by the team based on the target use-case: a systems
   language targeting embedded/real-time contexts has a different *T* from a server-side language.
   Until *T* is stated, this criterion cannot be met.
3. **The measured P99 is ≤ *T*** across W1/W2/W3 under a documented fuel budget *F*.
4. **The reclamation debt is bounded** under each workload: the epoch lag does not grow without
   bound over the workload run (measured by the debt counter).
5. **The pass/fail results are committed** to a measurement artifact in `docs/measurements/` with the
   same honest-baseline format as `DN-35-baseline-2026-06-26.md` — release build, reproducible
   commands, caveats load-bearing.

If the P99 exceeds *T* under the fixed budget, the measurement plan must document *why* (too-small
budget, bursty workload, inherent O(*n*) tail that the fuel model cannot flatten) and whether a
higher-F or adaptive budget can bring it within *T* — or whether `Declared` is the durable honest
tag for this design point.

**On `Declared` → `Proven`.** The `Proven` path requires a mechanized proof that a fuel budget *F*
bounds the per-epoch reclamation time to a function of *F* and the program's structural properties
(e.g., value tree depth, region size). This is a significantly higher bar than `Empirical` and is
not in scope for R1. The target for this note is `Empirical`; `Proven` is a Phase-3 aspiration
(consistent with the FP²/certified path in DN-35 §7).

---

## §5 The honesty stance — G2/EXPLAIN as the floor

Until the §4 measurement plan lands, the non-negotiable floor is:

1. **Every reclamation event is EXPLAIN-recorded** (RFC-0027 §9). An epoch that defers work must
   record the deferral — partial-sweep exhaustion is never silent.
2. **The `sweep_epoch` counter in `ReclamationRecord` is the debt-visibility anchor.** An agent or
   supervision policy can observe epoch lag without any additional surface by comparing the
   `sweep_epoch` at record emission time against the current epoch counter.
3. **No latency claim stronger than `Declared` is made until §4 runs.** Any prose in the codebase
   or documentation that asserts a sub-millisecond or bounded-by-constant drop latency is a VR-5
   violation until backed by the §4 measurement.
4. **A construct that introduces a silent pause must supersede RFC-0027** (append-only, house rule
   #3). This is the G2 safeguard: the fuel model *extends* the never-silent stance, it does not
   loosen it.

The honesty stance is **not** a weakening of the design — it is a prerequisite for any honest
upgrade. `Declared` is the *accurate* tag for the current state; pretending otherwise would be VR-5
violation.

---

## §6 Definition of Done — the gate for Draft → Resolved

DN-62 moves **Draft → Resolved** when:

1. **§4 workloads are run** (W1/W2/W3) with the fuel-model instrumentation wired (`ReclamationSink`
   per-epoch accumulator, heap-allocation counters, latency distribution recording).
2. **A threshold *T*** is stated (pre-run, not post-hoc fitted) and a fuel budget *F* is documented.
3. **Pass/fail results are committed** to `docs/measurements/DN-62-fuel-model-<date>.md` in the
   honest-baseline format — release build, reproducible commands, caveats load-bearing.
4. **One of two outcomes is recorded honestly:**
   - **(A) Upgrade:** measured P99 ≤ *T* across W1/W2/W3; debt bounded; results support upgrading
     the SLO from `Declared` to `Empirical`. DN-59 and RFC-0027 §11 OQ-3 are updated
     (append-only) to reflect the new `Empirical` tag.
   - **(B) Durable `Declared`:** measurement reveals that the fuel model cannot bound P99 ≤ *T* under
     the tested conditions, or that *T* is not achievable for the R1 fragment; `Declared` is
     confirmed as the honest durable tag, and the note records *why* (adversarial workload class,
     inherent O(*n*) tail, etc.).
5. **Outcome (B) does not block R1 progress** — the honesty stance (§5) holds in either case.
   A `Declared` SLO that is explicitly and honestly `Declared` is better than a `Declared` SLO
   that is tacitly assumed to be `Empirical`.

**On ratification**, the integrating parent (not this note) updates: `docs/Doc-Index.md` (add DN-62
row), `CHANGELOG.md` (append-only entry), `tools/github/issues.yaml` (link the tracking issue), and
RFC-0027 §11 OQ-3 status (if the SLO is upgraded). **DN-62 itself moves no other doc's status**
(house rule #3).

---

## §7 Open questions surfaced by this note

These are not decisions — they are honest residuals the measurement plan must inform:

- **OQ-A — What is the right fuel unit?** The `Declared` recommendation (§3.1) is nodes reclaimed,
  but RC decrements or region sweeps may be more faithful proxies for wall-clock latency. The §4
  measurement must test whether nodes-per-epoch correlates with measured P99 latency, or whether
  a different unit is a better predictor.

- **OQ-B — What threshold *T* is appropriate for Mycelium's target use cases?** *T* is use-case
  dependent and must be stated by the team before §4 runs. This note cannot fill that gap — it would
  be an `Declared` SLO number presented as meaningful without a target context.

- **OQ-C — Does the fixed-budget model hold under adversarial allocation patterns?** The W1 workload
  (deep value trees) is the adversarial case. A fixed budget *F* that bounds P99 under W2/W3 may not
  bound it under W1 if the tree depth exceeds *F*. The measurement plan must include the
  worst-case tree depth relative to *F* as a parameter.

- **OQ-D — What is the reclamation debt bound under sustained high load?** The `Declared` claim is
  that debt is bounded iff allocation rate does not permanently exceed *F* per epoch (§3.3). This
  must be measured under W2's mixed drop patterns. If the workload is bursty, peak debt may
  transiently exceed a stable bound — the measurement must characterize the burst behaviour.

- **OQ-E — How does the fuel model interact with the RC ⊕ region exactly-once coupling (DN-35 §6)?**
  The double-free guard (a cell freed by `RcZero` is removed from the deferred queue) requires that
  the deferred queue is scannable in O(1) or O(log n) — otherwise the guard itself introduces latency
  proportional to queue size. This is a design question that must be answered before the fuel counter
  structure is committed.

- **OQ-F — Fuel model and the `certified` path (DN-35 §7(B) / ADR-032).** In `certified` mode, the
  FP²-linearity check statically proves no-alloc / constant-stack for the certified subset. For
  programs in the certified subset, the fuel model may be vacuous (no heap allocation → no
  reclamation cascade). The interaction should be documented: is the fuel model relevant only for
  the `fast`-mode path, or does it also apply to `certified`-mode programs that fall outside the FIP
  linear fragment?

---

## §8 Grounding and corpus references

**Confirmed sources re-read for this note:**

- **RFC-0027** (Accepted, 2026-06-25): §7.2 (O(*n*) cascade — honest caveat); §9 (EXPLAIN/audit
  record, five-field set); §11 OQ-3 (mitigated — "Can the fuel model bound the cascade? A dedicated
  research note is warranted."); §12 (three-layer hybrid pointer).
- **DN-32** (Accepted, 2026-06-25): §2.3 (Layer 3 — region-based allocation + batched scope-exit
  reclamation + sweep-epoch model); §9 R-4 (sweep-epoch spreading — the key mitigation for OQ-3).
- **DN-35** (Accepted, 2026-06-26): §2.1 (MEM-1/2/3 runtime substrate — the built machinery); §2.2
  (the `eval_machine` seam); §6 (RC ⊕ region exactly-once coupling); §8 (open-question ledger);
  §9 (increment sequencing — Increment 1 is real free for the straight-line fragment, the immediate
  predecessor to fuel-model work).
- **DN-59** (the commissioning note — drop-latency / fuel-bound axis). *FLAG-1: DN-59 is **not present
  on this branch's `origin/dev` base**; its specific section numbers cannot be verified here. The
  verified, in-tree commission anchor is RFC-0027 §11 OQ-3 (`RFC-0027:493`). The integrating parent must
  reconcile DN-59's actual section references once DN-59 lands on the shared base.*
- **Measurements:** `docs/measurements/DN-35-baseline-2026-06-26.md` — the baseline captured before
  real env-machine reclamation work. The fuel-model measurement (§4) builds on this baseline.

**Value-model + house basis:**
- **LR-9** (acyclic values — no cycle detector; Perceus precondition).
- **LR-8** (immutable values — no write barrier; RC is correct).
- **RT7** (structured scopes — parent–child TOTAL, siblings CONCURRENT by construction).
- **G2** (never-silent — every reclamation event EXPLAIN-recorded; fuel exhaustion is not silent).
- **VR-5** (downgrade-don't-overclaim — `Declared` until measured; no upgrade past basis).
- **KC-3** (small auditable kernel — fuel counter must not grow the trusted core).
- **ADR-032** (`fast`/`certified` tunable certification — fuel model is relevant to `fast` path;
  interaction with `certified` is OQ-F above).

**Honest limits.** DN-62 introduces no new mechanism and no new measurement. Every quantitative
claim in §3 is `Declared`. The upgrade to `Empirical` is the *goal*, not the current state. This
note is the commission, not the result.

---

## Meta — changelog

- **2026-06-28 — Created (Draft).** Research-framing and measurement-plan note for the
  **reclamation fuel model** — commissioned by **DN-59** (the ratified reclamation-strategy note; *FLAG-1:
  not on this `origin/dev` base — verified commission anchor is* **RFC-0027 §11 OQ-3**, `RFC-0027:493`)
  ("Can the fuel model bound the RC-cascade drop latency? A dedicated research note is warranted.")
  to attempt lifting the drop-latency SLO from `Declared` to `Empirical`. **Four sections:**
  (§1) the question — what bounds per-epoch deferred-reclamation work so drop-latency is honest,
  not a silent GC pause; (§2) confirm-the-record from the Accepted cluster (RFC-0027 RC mechanism,
  DN-32 Layer-3 region batching + sweep-epoch spreading, the RFC-0027 §1 "no silent GC pause" honesty
  stance carried by DN-59, DN-35 RC ⊕ region exactly-once coupling) — no tag upgraded (VR-5); (§3) the candidate fuel model
  — **`Declared` throughout**: three fuel-unit candidates (nodes reclaimed / RC decrements / region
  sweeps; `Declared` recommendation is nodes reclaimed for R1), fixed/proportional/latency-feedback
  budget options (fixed for R1), partial-sweep deferral with never-silent debt visibility (G2), and
  relation to the existing `SweepEpoch`/`ReclamationRecord` skeleton; (§4) the measurement plan —
  **what moves the SLO to `Empirical`**: three instrumentation categories (per-epoch latency
  distribution P99/P999, fuel consumption per epoch, reclamation debt / epoch lag), three workload
  classes (W1 allocation-heavy deep trees, W2 mixed drop patterns, W3 hypha fan-out), and a
  **pre-specified pass/fail criterion** (threshold *T* stated pre-run; P99 ≤ *T*; debt bounded;
  results committed to `docs/measurements/`); (§5) the **honesty floor** — G2/EXPLAIN as the
  non-negotiable minimum regardless of §4 outcome: every reclamation event recorded, partial sweeps
  never silent, no latency claim stronger than `Declared` until §4 runs, silent-pause constructs
  must supersede RFC-0027 (append-only); (§6) Definition of Done — Resolved when §4 runs and an
  honest outcome (Empirical upgrade **or** confirmed durable `Declared`) is committed; (§7) six open
  questions (OQ-A fuel unit, OQ-B threshold *T*, OQ-C adversarial allocation, OQ-D debt bound under
  burst, OQ-E double-free-guard scan cost, OQ-F fuel-vs-certified interaction). **Enacts no code;
  moves no other doc's status** (house rule #3). Shared files (Doc-Index / CHANGELOG / issues.yaml
  / RFC-0027 OQ-3 status) **flagged to the integrating parent** — not edited here. All §3 design
  proposals **`Declared`-tagged**; all §2 confirm-the-record items held at source strength — no tag
  upgraded (VR-5). Never-silent throughout (G2: partial sweep recorded, debt visible, `Declared`
  where `Declared` is the honest tag). (Append-only; VR-5; G2.)
