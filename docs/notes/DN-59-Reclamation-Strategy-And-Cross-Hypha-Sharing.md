# Design Note DN-59 — Reclamation Strategy & Cross-Hypha Sharing (G3 ratification consolidation)

| Field | Value |
|---|---|
| **Note** | DN-59 |
| **Status** | **Draft** (2026-06-28) — a **single ratification-consolidation vehicle** for the G3 "memory / reclamation strategy" group of the Blocked-Decisions Ratification Map. **Proposes** a consolidated resolution of the seven G3 axes for one maintainer sign-off. **Enacts no code** and **moves no other doc's status**: it is a *record-and-confirm* over the already-Accepted RFC-0027 / DN-32 / DN-33 / DN-35 cluster plus a small set of **proposals** for the residue those docs left genuinely open. Several G3 items are **already resolved elsewhere** — for those this note is *confirm-the-record*, not *re-decide* (each flagged inline). Promotion past Draft requires the §10 Definition of Done + maintainer ratification (house rule #3, append-only). |
| **Feeds** | the **G3** batch of `docs/planning/Blocked-Decisions-Ratification-Map.md`; consolidates **RFC-0027** (Accepted) §11 OQ-1..OQ-6 + §12, **DN-32** (Accepted, three-layer hybrid), **DN-33** (Accepted, MEM-4 static uniqueness + §8.1 Q1/Q2), **DN-35** (Accepted, env-machine reclamation); unblocks **MEM-4 / E12** forward work, the eventual **RFC-0027 follow-on** (the cross-hypha boundary normative commit), and the Layer-2 RC-optimization roadmap. |
| **Date** | June 28, 2026 |
| **Decides** | *Proposes, for ratification (nothing enacted):* a consolidated G3 resolution across seven axes — **(a)** cross-hypha sharing protocol (sole-move vs atomic-RC), **(b)** the DN-33 §8 Q2 ownership-mode representation (the MEM-4 prerequisite), **(c)** Layer-2 static-uniqueness analysis scope, **(d)** drop-latency SLO / fuel-bound model (RFC-0027 OQ-3), **(e)** `rc==1` reuse visibility (OQ-4), **(f)** sweep-coupling strength (OQ-1), **(g)** env-machine reclamation wiring. Each carries: the proposal, its grounding, an honest tag, and the never-silent (G2) behaviour. The note **distinguishes R1-local resolutions (this DN) from R2-distributed deferrals (the concurrency group, G8)**. |
| **Task** | G3 batch ratification (post-DN-32 follow-on consolidation) |

> **Posture (transparency rule / VR-5 / G2).** This note is a **consolidation record**, not new
> mechanism design. Most G3 axes were *already* settled by the Accepted RFC-0027 / DN-32 / DN-33 / DN-35
> cluster; for those, DN-59 **confirms the append-only record** and re-states the resolution at the
> strength its source already holds it — it does **not** upgrade any tag (VR-5). Where an axis is
> *genuinely open*, DN-59 makes a **proposal** tagged **`Declared`-with-argument** — an argued design
> intent, not a measured or proven result. The two distinct postures are marked per-axis so the
> maintainer ratifies the *right kind of thing*: a confirm-the-record item asks "is the record correct?";
> a proposal item asks "do you adopt this resolution?". Every reclamation event remains reified and
> `EXPLAIN`-able (no black boxes), and every refusal / fallback is never-silent (G2). Per the
> ratification-map posture, the G3 group membership + citations are `Empirical` finder-sourced leads;
> the anchors here (RFC-0027 / DN-32 / DN-33 / DN-35 statuses + their cited sections) were re-read and
> confirmed before drafting (§9).

---

## §1 Purpose — why a consolidation note, and what it is *not*

The G3 group of the ratification map (`Blocked-Decisions-Ratification-Map.md` §G3) names the
reclamation/ownership questions *not* in G1 (runtime-vocabulary surface): cross-hypha sharing
protocol, Layer-2 static uniqueness analysis, drop-latency SLO / fuel bound, `rc==1` reuse
visibility, sweep-coupling, and env-machine reclamation wiring — "all share the three-layer hybrid
memory model (RFC-0027 + DN-32/33/35)."

The map's own assessment is the load-bearing fact this note honours up front: **"Several OQs are
already resolved (OQ-1 weak-coupling, OQ-4 EXPLAIN-only) and only need a ratification-record."** A
read of the cluster confirms the resolution surface is *wider* than that line implies — DN-33 §8.1
already ratified the cross-hypha boundary (Option A) and the ownership-mode representation (separate
RC-annotated IR), and DN-35 already ratified the env-machine reclamation *direction*. So the honest
shape of G3 today is:

- **Mostly confirm-the-record.** The reclamation *mechanism* (RC), the coupling resolution (OQ-1),
  the reuse-visibility resolution (OQ-4), the cross-hypha boundary (Option A), the ownership-mode
  representation (Q2), and the env-machine direction are **already Accepted** in the cluster. DN-59's
  job for these is to **assemble them into one batch and confirm the append-only record is
  consistent** — not to re-decide them.
- **A small genuinely-open residue.** The drop-latency SLO / fuel-bound model (OQ-3) is *mitigated,
  not closed*; the Layer-2 analysis *scope boundary* (what R1 targets vs defers) deserves an explicit
  consolidated statement; and the R1-vs-R2 split (which deferrals belong to the concurrency group G8,
  not here) should be stated once, plainly. For these, DN-59 makes **proposals**.

**What this note is not.** It is **not** the RFC-0027 follow-on. The follow-on is the *normative*
vehicle that would move the cross-hypha boundary commit (Option A) and any newly-stated invariant
into RFC-0027's body (append-only). DN-59 is a **planning/consolidation record** that gathers the G3
resolutions for a single ratification gate and routes the normative commits to the follow-on. It
**moves no RFC/ADR/DN status by itself** (house rule #3).

---

## §2 The seven G3 axes at a glance (posture per axis)

| # | Axis | Posture | Resolution (proposed / confirmed) | Source / honest tag |
|---|---|---|---|---|
| **(a)** | Cross-hypha sharing protocol — sole-move vs atomic-RC | **Confirm-the-record** | **Option A** (sole-ownership move only; `RcCell<T>` stays `!Send`; no cross-hypha atomic RC) for R1; **Option B** (shared-crosses-atomic-RC) deferred to **R2** (the G8 concurrency group). | DN-33 §8.1 Q1 (Accepted); RFC-0027 §7.3 / §12; DN-32 §7. "A sufficient for R1" `Declared`; prior art `Empirical`. |
| **(b)** | Ownership-mode representation (DN-33 §8 Q2 — MEM-4 prerequisite) | **Confirm-the-record** | **Separate RC-annotated IR** in `mycelium-mir-passes`; the trusted Core IR (`mycelium-core/src/node.rs`) stays **pristine** — no `BorrowMode` field on kernel binding forms. | DN-33 §8.1 Q2 (Accepted); built (`mir-passes` crate exists). KC-3 argument. |
| **(c)** | Layer-2 static-uniqueness analysis scope | **Proposal (consolidating)** | R1 targets **frame-limited** reuse via the **additive** Perceus/Lorenzen lowering: Increment 1 (non-escaping borrow elision) → Increment 2 (`rc==1` reuse annotation); **full FIP / garbage-free is Phase-3**. Sound-but-incomplete; runtime `RcCell` probe is the sound fallback. | DN-33 §2/§6/§8.1 Q7; DN-32 §2.2/§6b/§8. `Declared`; soundness strategy `Empirical` (differential). |
| **(d)** | Drop-latency SLO / fuel-bound model (RFC-0027 OQ-3) | **Proposal (genuinely open)** | **SLO stays `Declared`**; adopt the **`Declared` per-epoch deferred-reclamation model** (Layer-3 regions + sweep-epoch batching) as the R1 stance; commission a **fuel-model research note** to attempt `Declared → Empirical`. "No silent GC pause" is an **honesty stance**, not a sub-ms SLO. | RFC-0027 §11 OQ-3 (mitigated) + §7.2; DN-32 §2.3/§9 R-4. `Declared`. |
| **(e)** | `rc==1` reuse visibility (RFC-0027 OQ-4) | **Confirm-the-record** | **EXPLAIN-record-only by default** — the `Provenance` DAG always captures `push_reuse` vs `push_copy` (G2); a surface-visible `fip`-style variant is deferred to **Phase-3**. | RFC-0027 §11 OQ-4 (resolved by DN-32); DN-32 §8 Phase-1; DN-33 §8.1 Q6. Confirmed. |
| **(f)** | Sweep-coupling strength (RFC-0027 OQ-1) | **Confirm-the-record** | **Parent–child TOTAL, siblings CONCURRENT by default** (weak/partial coupling); strong/total coupling an **opt-in** for high-assurance subsets. | RFC-0027 §11 OQ-1 (resolved by DN-32) + §12; DN-32 §3. Safety `Proven`-modulo-LR-9; throughput `Declared`. |
| **(g)** | Env-machine reclamation wiring | **Confirm-the-record (direction) + Proposal (R1 sequencing)** | Direction **ratified** (DN-35): static-decisions / dynamic-verification; drop-guided reuse; the `is-unique` gate is the mandatory safety valve; `fast`/`certified` split. DN-59 confirms it and proposes the **R1 increment floor**: real free for the straight-line fragment behind the reference RC-evaluator oracle; reuse + recursion later. | DN-35 §3/§9/§10 (Accepted). Direction `Declared`; prior-art single-mechanism results `Proven`-at-source. |

The remainder of the note takes each axis in turn at its appropriate depth (confirm-the-record items
are stated + grounded compactly; proposal items carry the argument).

---

## §3 (a) Cross-hypha sharing protocol — **CONFIRM-THE-RECORD: Option A for R1**

**The question (RFC-0027 §12 / DN-32 §7).** When a value crosses a hypha boundary, is it a *shared*
value (→ atomic RC, DN-32 Layer 2) or *sole* ownership being *moved* (→ affine channel move, no
cross-hypha RC, RFC-0027 §7.3)?

**Already resolved.** DN-33 §8.1 **Q1 → Option A** (Accepted, ratified 2026-06-25): only **sole
ownership** crosses a hypha boundary (an affine move, which the RFC-0027 §7.3 `Sender`/`Receiver`
non-`Clone` channel protocol already enforces); **`RcCell<T>` stays `!Send`**; **no atomic RC**.
Option B (a genuinely-shared `rc > 1` value crossing into concurrent use, promoting RC to atomic) is
**gated to R2** (`xloc`/`mesh`) — i.e. it belongs to the **G8 concurrency/distribution group**, not
G3 (see §8 below + the ratification-map cross-dep #2).

**Grounding (unchanged from DN-33 §5/§8.1).** Prior art is `Empirical`: Pony `iso` + `consume` (safe
actor transfer, no RC); Rust `Box<T>: Send` vs `Rc<T>: !Send` (unique ownership moves across threads
with no atomic ops). The channel-close synchronization gives the same guarantee as Pony's `consume`,
and LR-9 removes the need for Pony's ORCA cycle machinery. "Option A is **sufficient** for R1" is
**`Declared`** — an argument, with the **named honest risk** (carried forward, not buried) that real
R1 programs may need restructuring to avoid holding a shared `RcCell<T>` in one hypha and sending it
to another; that ergonomic cost is **unmeasured** until example programs exist.

**Never-silent (G2).** Option A is enforced *by construction* — `RcCell<T>: !Send` makes a
cross-hypha send of a shared value a **compile-time type error**, not a silent runtime promotion. An
attempt to cross a shared value is a loud `!Send` rejection, never a quiet atomic-RC upgrade.

**DN-59 action:** **confirm** Option A as the G3 R1 record; **route** the normative commit (moving
"Option A" into RFC-0027's body) to the RFC-0027 follow-on; **note** Option B is an R2 / G8 item, not
re-opened here.

---

## §4 (b) Ownership-mode representation (DN-33 §8 Q2) — **CONFIRM-THE-RECORD: separate RC-annotated IR**

**The question (DN-33 §8 Q2).** How is the own-vs-borrow ownership mode represented — a `BorrowMode`
annotation on `Node` binding sites (Lean-4 `@`-borrow style), a separate borrow IR layer, or a
compiler-internal flag invisible to the surface IR? This is the **hard prerequisite for MEM-4** the
G3 map calls out: MEM-4 cannot elide RC ops it has no representation for.

**Already resolved.** DN-33 §8.1 **Q2 → separate RC-annotated IR** (Accepted): the own/borrow mode
and the emitted `dup`/`drop` ops live on a **new RC-annotated IR** (`RcNode`, in
`crates/mycelium-mir-passes/`) produced by the lowering pass; the trusted Core IR
(`mycelium-core/src/node.rs` `Node`) **stays pristine** — **no `BorrowMode` field on kernel binding
forms**.

**Grounding (KC-3 merit argument, unchanged from DN-33 §8.1).** This was chosen over the simpler
"annotate `Node`" option *precisely because* annotating the kernel would tax the trusted base
everyone audits. Keeping the analysis in a separate, **untrusted, optimization-only** crate means
MEM-4's entire correctness obligation lives outside the trusted Core IR + type checker — the kernel
does not grow, honouring KC-3 and DN-33 §4's "lowering-pass-not-type-checker; trusted base
unchanged." This resolution is **confirmed by the built state**: the `mir-passes` crate exists with
`rc_ir.rs` / `emit.rs` / `eval.rs` / `balance.rs` / `corpus.rs`, and `node.rs` is untouched (the
DN-33 §6.1 "prerequisite gap — add a field to node.rs" finding was *superseded*; that step is **moot**
under the Q2 ruling — DN-33 §6.1 Correction).

**DN-59 action:** **confirm** Q2 as resolved and consistent with the built tree. No re-decision; no
normative move. The MEM-4 prerequisite the G3 map flagged is **already discharged** — DN-59 records
that plainly so the maintainer is not asked to re-ratify a settled question.

---

## §5 (c) Layer-2 static-uniqueness analysis scope — **PROPOSAL (consolidating): frame-limited, additive, increment-staged**

This axis is the one where the cluster has *ratified the design* (DN-33) but the **R1 scope boundary**
benefits from an explicit consolidated statement, because the G3 map lists "Layer-2 static uniqueness
analysis" as an open axis and the build plan shows increments landing piecemeal.

**Proposal (consolidating the DN-33 §6/§8.1 + DN-32 §8 direction):**

1. **MEM-4 is additive and sound-but-incomplete.** It only ever **elides provably-redundant RC ops**;
   the runtime `RcCell` probe (MEM-2) is the **sound fallback** for everything it cannot prove. A bug
   downgrades to a missed optimization (a fired probe), **never** unsafety (DN-33 §2). This is the
   property that makes a non-trivial analysis pass *tolerable* under KC-3.
2. **R1 target is frame-limited reuse, not garbage-free.** Increment 1 (non-escaping borrow elision)
   yields *frame-limited* memory (peak within a constant factor of the live set — Lorenzen–Leijen),
   which is **sufficient for systems use**. Full *garbage-free* (Perceus) and a static `fip` guarantee
   are the **Phase-3 aspiration**, not an R1 gate (DN-33 §8.1 Q7/Q6).
3. **Increment staging (dependency-ordered):** Increment 1 (non-escaping borrow elision; smallest,
   testable as a refcount-invariant static check) → Increment 2 (`rc==1` reuse annotation) → Increment 3
   (full FIP static guarantee, Phase-3, deferred). `substrate`/`consume` uniqueness **subsumes into**
   the same elision mechanism (DN-33 §8.1 Q4), not a parallel path (DRY).
4. **Soundness strategy is differential + structural-invariant, tagged `Empirical`** (not `Proven`):
   run the RC-annotated IR through the reference RC-evaluator with and without elision, asserting
   identical observable results **and** identical reclamation records, backed by the balance invariant
   (`1 + dups == uses + drops`). A mechanized proof is a Phase-3 option (VR-5: no upgrade past basis)
   (DN-33 §8.1 Q3).
5. **The KC-3 cost is watched + measured per increment** (DN-32 §6b). The enforceable gate is a
   **measured `dup`/`drop`-reduction ratio** on a representative corpus (DN-33 §8.1 Q5); the count
   itself is **`Exact`** (read off the IR), so the gate is enforceable as soon as a corpus lands —
   while the *performance* interpretation stays `Declared` until measured.

**Honest tag.** All Mycelium-specific behaviour/perf is **`Declared`**; the soundness strategy is
**`Empirical`** (differential trials); external prior art (Perceus, Lorenzen, Koka/Lean) is
**`Empirical`** at source, its *transfer* to Mycelium's typed Core IR `Declared`. **Never-silent:**
any term outside the analysed fragment (recursion, higher-order, escaping use) stays **owned** /
refuses explicitly (`Fix`/`FixGroup` refused in `emit.rs`, `UnsupportedNode` in the evaluator) — G2,
never a silent mis-elision.

**DN-59 action:** **adopt** this as the consolidated R1 scope statement for Layer-2 analysis (it
re-states the DN-33-ratified direction with the R1/Phase-3 boundary made explicit). No new mechanism;
no normative move.

---

## §6 (d) Drop-latency SLO / fuel-bound model (RFC-0027 OQ-3) — **PROPOSAL (genuinely open): `Declared` per-epoch model; commission a fuel-model note**

This is the **one G3 axis that is genuinely open**, not merely confirm-the-record. RFC-0027 §11 OQ-3
is **mitigated, not closed** by DN-32.

**The problem (RFC-0027 §7.2 / OQ-3).** Each `rc_dec` is O(1), but dropping a deep value tree is
O(*n*) in its node count — a **bounded-but-large** pause, not a sub-millisecond one. "Bounded" here
means *deterministic, observable, and EXPLAIN-able*, **not** constant-time. So "no silent GC pause"
is satisfied as an **honesty stance** (every reclamation step is observable via the §9 record) but is
**not** a worst-case latency SLO.

**What is already mitigated (DN-32, confirm).** DN-32 Layer 3 (region-based allocation + **batched
scope-exit reclamation**) + the sweep-epoch model move the worst case from an *inherent limitation*
toward an *engineering/measurement* problem: batched region reclamation bounds per-epoch latency at
the price of *deferred* reclamation, and the sweep-epoch model can spread the cost across epochs.

**Proposal for the open residue:**

1. **The latency SLO stays `Declared` for R1.** There is no methodology'd in-repo benchmark and no
   fuel-model bound; per VR-5 the tag must not be upgraded. DN-59 **adopts the `Declared` per-epoch
   deferred-reclamation model** (Layer-3 batching + sweep-epoch spreading) as the R1 stance, and is
   explicit that this is a *target/engineering posture*, not a guarantee.
2. **"No silent GC pause" is ratified as an honesty stance, not an SLO.** This is the never-silent
   (G2) invariant: the only acceptable pause is a *bounded, logged, never-silent* reclamation event
   (RFC-0027 §3/§9). A future construct introducing a silent pause must **supersede** RFC-0027, not
   silently contradict it.
3. **Commission a dedicated fuel-model research note** (RFC-0027 §11 OQ-3 already says "a dedicated
   research note is warranted"). The open question it must answer: can the fuel model (RFC-0014 §4.8)
   bound the RC-cascade drop latency, lifting it **`Declared` → `Empirical` / `Proven`**? Until that
   note exists and a methodology'd benchmark lands, the SLO is honestly `Declared`.

**Honest tag.** **`Declared`** throughout — this is the *honest* posture, not a weakness papered over.
The per-op `rc_dec` cost is `Exact` (O(1)); the *cascade* latency is `Declared`; the SLO is **open**.

**DN-59 action:** **adopt** the `Declared` per-epoch stance + the honesty-stance ratification, and
**flag the fuel-model note as the open work item** (FLAG-1, §11). This is the axis where the
maintainer is genuinely asked to *adopt a posture*, not confirm a settled fact.

---

## §7 (e) `rc==1` reuse visibility (RFC-0027 OQ-4) & (f) sweep-coupling (OQ-1) — **CONFIRM-THE-RECORD**

These two are the items the G3 map itself flags as "already resolved … only need a ratification-record."

**(e) OQ-4 — reuse visibility → EXPLAIN-record-only (confirmed).** RFC-0027 §11 OQ-4 is **resolved by
DN-32**: the `rc==1` reuse-vs-copy choice is **EXPLAIN-record-only by default** — the `Provenance` DAG
always captures it as distinct ops (`Derived{op:"push_reuse"}` vs `Derived{op:"push_copy"}`, G2,
never-silent), and **surface visibility is deferred** (a future may expose an FP²-`fip`-style
surface-visible variant — DN-33 §8.1 Q6 keeps that Phase-3). The guarantee tag is **`Exact` either
way** (the operation carries no approximation); only the *path* differs. **DN-59 action: confirm.**

**(f) OQ-1 — sweep-coupling → parent-child TOTAL, siblings CONCURRENT (confirmed).** RFC-0027 §11
OQ-1 is **resolved by DN-32** (§3, §12): parent–child reclamation is **TOTAL** (children fully reclaim
before the parent exits — forced by RT7), and sibling reclamation is **CONCURRENT by default**
(weak/partial coupling) — RT7 already makes siblings concurrent *by construction*, and LR-9 rules out
cross-sibling aliases, so concurrent sibling reclamation is **safe** with no serialization. **Strong/
total coupling is an opt-in** for high-assurance subsets (one property test covering both scheduling +
reclamation, at the cost of serialized sibling cleanup).

**Honest tag (carried unchanged — VR-5).** The **safety** of concurrent sibling reclamation is
**`Proven`-modulo the LR-9 side-condition** (the same strength RFC-0027 §8 holds the RC-soundness
claim — external argument, corpus-invariant side-condition, no in-repo mechanized check yet). The
**throughput benefit** of weak over strong coupling is **`Declared`** — *expected, not measured*.
OQ-1 was resolved **by argument** (RC + LR-9 acyclicity ⇒ order-independent safety; RT7 ⇒ siblings
already concurrent), **not** by the lane-B prototype-and-measure path; the follow-on may still measure
before locking the property-test surface (DN-32 §6c). **DN-59 action: confirm**, holding both tags at
their source strength — no upgrade.

---

## §8 (g) Env-machine reclamation wiring & the **R1 / R2 split**

**(g) Env-machine reclamation — direction confirmed (DN-35), R1 floor proposed.** DN-35 (Accepted,
2026-06-26) **ratified the design direction** for threading *actual* Mycelium-level reclamation into
the AOT env-machine (the deferred step past the §9 audit trail). DN-59 **confirms** it and re-states
the load-bearing parts for the G3 record:

- **Static decisions, dynamic verification.** Reclamation events are first-class IR ops inserted by a
  separate *untrusted, re-checkable* pass; the **trusted core interprets only**
  `incref`/`decref`/`is-unique`/`reuse-or-alloc`. The trampoline (explicit heap control stack) *forces*
  explicit scheduled drops — auditable, no black boxes (DN-35 §3).
- **The runtime `is-unique` gate is the mandatory safety valve.** A wrong static "reuse here" decision
  **degrades to a correct fresh allocation** — worst case "no reuse," never use-after-free. This is the
  additive invariant made operational (DN-35 §1/§3.5).
- **The R1 increment floor (proposal, sequencing only):** **Increment 1 — real free for the
  straight-line fragment** (`Const/Let/Op/Swap`): thread `sink`/`scope_id`/`sweep_epoch` through
  `eval_machine` (DN-35 §8 Q4), wrap values in `RcCell<CoreValue>`, `drop_ref` at scope-exit, with the
  `ReclamationRecord` stream **differential-checked against the reference RC-evaluator oracle** (DN-35
  §2.3/§9). Drop-guided reuse (Increment 2) and recursion (`Fix`/`FixGroup`, refused never-silently
  until a recursive RC-evaluator lands — DN-35 §8 Q7) come **later**.
- **Two Mycelium-specific obligations stay `Empirical`/`Declared`** until property-tested (DN-35 §5/§6):
  (i) **in-place reuse vs content-address identity** (§5 — reuse only at rc==1, weak intern table,
  evict-then-reuse-or-copy; the highest-priority new side-condition, no prior proof); (ii) **eager-RC ⊕
  batched-region exactly-once** (§6 — Gay–Aiken design, no combined mechanized proof). DN-59 **does not
  upgrade** either (VR-5).
- **`fast`/`certified` split (ADR-032).** `fast` (default) keeps the dynamic gate (`Empirical` tier);
  `certified` adds an FP²-FIP-linearity check that can elide the gate (`Proven` tier). The swap between
  them is itself a reified, `EXPLAIN`-able, never-silent choice.

**The R1 / R2 split (stated once, plainly — this is the cross-group boundary G3 must own).** The G3
map's cross-dep #2 says distributed reclamation (RFC-0027 OQ-2) "spans G3 (R1 RC strategy) and G8 (R2
distributed) — split by tier." DN-59 makes the split explicit:

| Tier | Belongs to | What it covers | Status here |
|---|---|---|---|
| **R1-local** (this DN / G3) | DN-59 | Single-node reclamation: the three-layer hybrid, RC mechanism, intra-hypha non-atomic RC, sole-move cross-hypha (Option A), the coupling resolution, the env-machine R1 floor, the Layer-2 R1 scope. | **Resolved / proposed here.** |
| **R2-distributed** (defer → G8) | the **R2 distributed-execution RFC** (G8 concurrency group) | Cross-node reclamation when `xloc`/`mesh` land: a value crossing node boundaries where the reclaiming scope ≠ the creating scope; **Option B** (shared-crosses-atomic-RC); CRDT tombstone GC + weighted reference counting; distributed-reclamation provenance (RFC-0027 **OQ-2**). | **Explicitly deferred — out of scope for DN-59.** |

This split is itself the resolution of RFC-0027 **OQ-2** *as a routing decision*: OQ-2 is **not closed**
here, it is **assigned to G8 / the R2 RFC** with a clean handoff (RFC-0027 §3/§11 OQ-2 already mark
distributed reclamation out of scope for R1). DN-59 closes the *G3* question by declaring it an *R2*
question — never-silent about the deferral (G2).

**Also out of scope for DN-59 (flag-and-defer, not re-opened):** RFC-0027 **OQ-5** (`substrate`/`graft`
drop-without-consume protocol — depends on the `graft` implementation RFC) and **OQ-6** (whether the
reclamation record *is* a `Provenance::Derived` node or *cites* one — routed to the transpilation DN +
the RFC-0027 follow-on). Both are legitimate non-blockers the cluster already deferred; DN-59 records
them as deferred so a future doc cannot silently contradict the model.

---

## §9 Grounding & re-verification (VR-5)

**Anchors re-read before drafting (the ratification-map posture requires confirming citations at
ratification time, not trusting finder leads):**

- **RFC-0027** — **Accepted** (2026-06-25). §7 (RC mechanism), §7.2 (O(*n*) deep-drop honesty), §7.3
  (affine cross-hypha transfer, no cross-hypha RC), §8 (guarantee tags — RC-soundness
  `Proven`-modulo-LR-9), §9 (the EXPLAIN/audit record, never-silent), §10 (the `rc`-probe unification),
  §11 (OQ-1..OQ-6 — OQ-1/OQ-4 resolved, OQ-3 mitigated, OQ-2/5/6 deferred), §12 (three-layer pointer +
  the cross-hypha sub-question). The §12 honest-scope note confirms reclamation is **not yet threaded
  into the AOT env-machine** (seam `crates/mycelium-mlir/src/aot.rs::eval_machine`).
- **DN-32** — **Accepted** (2026-06-25). Three-layer hybrid (§2), coupling resolution (§3, resolves
  OQ-1), the verified-parent caveats (§6 — perf `Declared`, Layer-2 the hardest leg, OQ-1-by-argument),
  the named cross-hypha sub-question (§7), the Phase 1–3 roadmap (§8).
- **DN-33** — **Accepted** (2026-06-25). MEM-4 additive principle (§2), cross-hypha recommendation (§5),
  the decomposition (§6), and the ratified **§8.1 resolutions** (Q1 → Option A, Q2 → separate RC-IR, Q3
  → differential `Empirical`, Q4–Q7 defaults). The §6.1 "prerequisite gap" is a *superseded* historical
  snapshot (its Correction neutralizes the standing claim).
- **DN-35** — **Accepted** (2026-06-26). Env-machine reclamation direction (§3), the EASIER/HARDER split
  (§4/§5), the RC ⊕ region exactly-once coupling (§6), the `fast`/`certified` split (§7), the
  open-question ledger (§8), the increment sequence (§9), and the guarantee posture + DoD (§10).
- **E12-Memory-Model-Build-Plan.md** — confirms MEM-1/2/3 + the MEM-4 static tier + the §9 AOT
  audit-trail bridge are **built**; env-machine real reclamation is the **forward epic** (Increment 3 /
  task #6).

**Value-model & house basis:** LR-8 (immutable values — no write barrier), LR-9 (acyclic — Perceus's
garbage-free precondition, no cycle detector), RT7 (structured scopes — siblings concurrent by
construction), KC-3 (small auditable kernel — the Q2 separate-IR choice + the additive-pass
discipline), VR-5 (downgrade-don't-overclaim — every tag held at source strength), G2 (never-silent —
`!Send` rejection, refused recursion, the EXPLAIN/audit record), ADR-032 (`fast`/`certified` tunable
certification).

**Honest limits of this note.** DN-59 introduces **no new mechanism** and **no new measurement**; it
**cannot upgrade** any tag, and does not. Where it makes proposals ((c) consolidating scope, (d) the
`Declared` SLO stance + fuel-model commission, the R1/R2 split, the (g) R1 increment floor), those are
**`Declared`-with-argument** — adopt-or-revise judgements, not proven results. The confirm-the-record
items ((a), (b), (e), (f), and the (g) direction) re-state Accepted resolutions and ask only whether
the **assembled record is correct**.

---

## §10 Definition of Done — what the maintainer ratifies

DN-59 moves **Draft → Accepted** when the maintainer ratifies the consolidated G3 resolution, i.e.
confirms/adopts the following as one batch (append-only; this note enacts no code and moves no other
doc's status):

1. **(a) Cross-hypha → Option A for R1** (sole-move-only; `RcCell<T>: !Send`; no cross-hypha atomic
   RC). *Confirm the DN-33 §8.1 Q1 record.* The normative commit into RFC-0027's body routes to the
   **RFC-0027 follow-on**.
2. **(b) Ownership-mode representation → separate RC-annotated IR** (Core IR pristine). *Confirm the
   DN-33 §8.1 Q2 record* — the MEM-4 prerequisite is discharged.
3. **(c) Layer-2 analysis R1 scope** → additive, frame-limited, increment-staged (Inc-1 borrow elision
   → Inc-2 reuse annotation → Inc-3 FIP Phase-3), soundness `Empirical` (differential + balance
   invariant), KC-3-watched per increment. *Adopt the consolidating statement.*
4. **(d) Drop-latency SLO → `Declared` per-epoch model** (Layer-3 batching + sweep-epoch spreading) +
   **"no silent GC pause" as an honesty stance, not an SLO**, + **commission the fuel-model research
   note**. *Adopt the open-residue posture* (FLAG-1).
5. **(e) `rc==1` reuse visibility → EXPLAIN-record-only by default** (surface variant Phase-3).
   *Confirm the OQ-4 record.*
6. **(f) Sweep-coupling → parent-child TOTAL, siblings CONCURRENT by default** (strong coupling
   opt-in); safety `Proven`-modulo-LR-9, throughput `Declared`. *Confirm the OQ-1 record.*
7. **(g) Env-machine reclamation → DN-35 direction confirmed** + the **R1 increment floor** (real free
   for the straight-line fragment behind the RC-evaluator oracle; reuse + recursion later). *Confirm
   the direction; adopt the R1 sequencing.*
8. **The R1 / R2 split adopted:** R1-local reclamation is resolved here (G3); R2-distributed
   reclamation — Option B, distributed-reclamation provenance (**OQ-2**), CRDT tombstone GC — is
   **assigned to G8 / the R2 RFC**, out of scope for DN-59. **OQ-5** (`substrate`/`graft` drop) and
   **OQ-6** (reclamation-record-as-`Derived`-node) remain deferred non-blockers. *Adopt the routing.*

**On ratification**, the orchestrator/integrating parent (not this note) updates the G3 row of the
ratification map and routes the normative commits to the RFC-0027 follow-on. **DN-59 itself moves no
status** (house rule #3).

---

## §11 FLAGs (raised up; not resolved here)

- **FLAG-1 (open work item — the one genuinely-open axis).** The **drop-latency fuel-model research
  note** (RFC-0027 OQ-3) does not exist. Until it lands + a methodology'd in-repo benchmark exists, the
  drop-latency SLO is honestly **`Declared`**. This is the only G3 axis that is *adopt-a-posture* rather
  than *confirm-a-record*; the maintainer should note it as the residual open question, not a closed one.
- **FLAG-2 (shared-file ownership — orchestrator action required).** Per swarm file-ownership rules,
  this note **does not touch** `docs/Doc-Index.md`, `CHANGELOG.md`, or `tools/github/issues.yaml`. The
  integrating parent must: (i) add the DN-59 row to `docs/Doc-Index.md`; (ii) add a `CHANGELOG.md`
  entry; (iii) update the **G3 row of `Blocked-Decisions-Ratification-Map.md`** on ratification; (iv) if
  a tracking issue exists for the G3 batch, link DN-59 in `issues.yaml`. **Flagged up — not edited here.**
- **FLAG-3 (normative routing).** The cross-hypha Option A commit (§3) and any newly-stated invariant
  are **planning records here**; their *normative* landing is the **RFC-0027 follow-on**'s job
  (append-only into RFC-0027's body). DN-59 must not be read as having moved RFC-0027's text or status.
- **FLAG-4 (R2 handoff).** Option B + distributed-reclamation provenance (OQ-2) are **assigned to the
  G8 concurrency group / R2 RFC**. If G8 ratifies before this note, reconcile the routing so OQ-2 is
  owned in exactly one place (the R2 RFC), not twice.

---

## Meta — changelog

- **2026-06-28 — Created (Draft).** A single **G3 ratification-consolidation** vehicle gathering the
  reclamation/ownership decisions of the already-Accepted RFC-0027 / DN-32 / DN-33 / DN-35 cluster into
  one maintainer sign-off, and proposing resolutions for the residue those docs left genuinely open.
  Seven axes, marked by posture: **confirm-the-record** — (a) cross-hypha **Option A** for R1
  (`RcCell<T>: !Send`, no cross-hypha atomic RC — DN-33 §8.1 Q1), (b) ownership-mode = **separate
  RC-annotated IR**, Core IR pristine (DN-33 §8.1 Q2 — the MEM-4 prerequisite, discharged), (e) `rc==1`
  reuse = **EXPLAIN-record-only** (RFC-0027 OQ-4), (f) sweep-coupling = **parent-child TOTAL / siblings
  CONCURRENT** by default, strong opt-in (RFC-0027 OQ-1; safety `Proven`-modulo-LR-9, throughput
  `Declared`), (g) env-machine reclamation **direction** (DN-35: static-decisions/dynamic-verification,
  drop-guided reuse, `is-unique` safety valve, `fast`/`certified`); **proposals** — (c) Layer-2 analysis
  R1 scope = additive, **frame-limited**, increment-staged (soundness `Empirical`), (d) drop-latency SLO
  stays **`Declared`** (Layer-3 batching + sweep-epoch model; "no silent GC pause" = honesty stance, not
  SLO) with a **fuel-model research note commissioned** (the one genuinely-open axis — FLAG-1), and the
  (g) **R1 increment floor**. States the **R1-local (this DN) / R2-distributed (defer → G8)** split once
  and plainly: Option B + distributed-reclamation provenance (RFC-0027 **OQ-2**) are **assigned to the
  G8 / R2 RFC**, not re-opened here; **OQ-5** (`substrate`/`graft` drop) + **OQ-6** (reclamation record
  as `Derived` node) remain deferred non-blockers. **Enacts no code; moves no other doc's status.** All
  proposals **`Declared`-with-argument**; all confirmed records held at source strength — no tag upgraded
  (VR-5). Never-silent throughout (G2: `!Send` rejection, refused recursion, the EXPLAIN/audit record).
  Shared files (Doc-Index / CHANGELOG / issues.yaml / the ratification-map G3 row) **flagged up to the
  integrating parent** (FLAG-2), not edited here. Promotion past Draft requires the §10 Definition of
  Done + maintainer ratification (house rule #3, append-only). (Append-only; VR-5; G2.)
