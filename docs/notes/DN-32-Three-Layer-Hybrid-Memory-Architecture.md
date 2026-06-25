# Design Note DN-32 — Three-Layer Hybrid Memory Architecture

| Field | Value |
|---|---|
| **Note** | DN-32 |
| **Status** | **Accepted** (2026-06-25, **ratified by the maintainer**) — the maintainer's strategic direction for the runtime memory architecture. *(Status flagged: this note is authored from a verified maintainer proposal, so it starts above bare `Draft` — but it enacts nothing and changes no normative text; the binding decision is the RFC-0027 follow-on.)* |
| **Feeds** | resolves **RFC-0027** §11 **OQ-1** + **OQ-4** (and mitigates **OQ-3**); feeds E12-1 (runtime & concurrency execution maturity); anchors the future RFC-0027 follow-on (Layer-2 uniqueness analysis + the cross-hypha sharing sub-question, §7) |
| **Date** | June 25, 2026 |
| **Decides** | *Nothing normatively* — advisory + strategic-direction capture. Records the maintainer's **three-layer hybrid memory architecture** for the Mycelium runtime: (1) affine/linear ownership as the **primary** path for unique data, (2) optimized reference counting only for **explicit** sharing, (3) region-based allocation/reclamation **within scopes**, with parent–child reclamation **total** and sibling reclamation **concurrent-by-default** (weak/partial coupling; strong coupling opt-in). Captures the goals, the Phase 1–3 roadmap, the risks, and the relation to the corpus, **with the verified-parent caveats (§6) as honest scope.** The binding decision is the RFC-0027 follow-on. |
| **Task** | E12-1 (M-712) — runtime memory model evolution (RFC-0027 OQ-resolution) |

> **Posture (transparency rule / VR-5).** This note **captures a maintainer-authored strategic
> direction** and **enacts nothing** — it moves no status on any RFC/ADR by itself, changes no
> normative text, and ships no code or property test. Every performance figure herein is a
> **`Declared` goal/target**, not a measurement (§6a). The architecture is sound *as an argument*
> on the corpus (LR-8/LR-9 + RT7 + the RC-over-acyclic-values prior art — §5), but the soundness
> of Layer-2's static uniqueness analysis, the throughput benefit of weak-over-strong sibling
> coupling, and every Mycelium-specific binding remain **unbuilt and `Declared`**. The
> resolutions DN-32 licenses for RFC-0027 (OQ-1, OQ-4) are tagged at their supportable strength in
> the RFC, not here-upgraded (VR-5). Nothing is claimed `Proven` without a checked basis; the
> reconciliation sub-question in §7 is surfaced, not buried (G2).

---

## §1 Purpose

RFC-0027 (Proposed, 2026-06-24) decided the reclamation **mechanism** — precise reference counting
(RC), justified because LR-9 acyclicity *is* Perceus's garbage-free precondition — but deliberately
left its highest-uncertainty design point open: **OQ-1**, whether sibling-scope reclamation must
total-order with the sweep order (strong coupling) or may run concurrently (weak/partial coupling).
It also left **OQ-4** (is the `rc==1` reuse-vs-copy choice surface-visible or EXPLAIN-record-only?)
and **OQ-3** (the worst-case RC-cascade drop-latency / pause budget) open.

This note captures the maintainer's **strategic resolution**: a **three-layer hybrid memory
architecture** that places affine/linear ownership *first* (so RC is paid for only on explicit
sharing), wraps both in region-based allocation/reclamation within scopes, and answers the
sibling-coupling question (OQ-1) in favor of **concurrent-by-default** sibling reclamation — which
RT7 already licenses (sibling scopes are concurrent by construction, RFC-0008 §4.1) and LR-9 makes
*safe* (no cross-sibling aliases to police). The goal triad is **maximum performance/throughput +
memory safety + "stupid easy" ergonomics**, preserving LR-8/LR-9/RT7.

---

## §2 The three layers

The architecture is a **layered hybrid**: each layer handles the case it is best at, and the layers
compose so the cheapest path is the default and the more expensive machinery engages only when the
program actually needs it. This is the memory-tier analogue of DN-29/RFC-0034's "pay for what you
use" posture — the expensive mechanism is opt-in by *construction of the program*, not a tax on every
value.

### §2.1 Layer 1 — Affine/Linear ownership (PRIMARY; the default path)

**Unique data is owned affinely/linearly — moved, not shared — at (near-)zero cost.** This is the
default and primary path: a value with a single owner needs no reference count at all; ownership
transfer is a move, reclamation is a scope-exit drop. The corpus already has the lever — the affine
`substrate`/`consume` discipline (DN-02/DN-03; Glossary; RFC-0006 LR-8) — today scoped to *external
resources*; Layer 1 is the design point where affine ownership is the primary discipline for unique
*values*, with RC (Layer 2) engaged only when a value is *explicitly* shared.

- **Cost:** zero / near-zero — no refcount traffic on the unique path.
- **Reclamation:** scope-exit drop (RT7 LIFO), `Exact` by Rust's drop specification.
- **Corpus basis:** RFC-0008 RT1 ("values move, state is never shared"), the affine `Sender`/
  `Receiver` channel pair (non-`Clone` — single-owner cross-hypha transfer, RFC-0027 §7.3),
  the `substrate` affinity lever (lane-A; lane-F §2.1). Prior art: Clean uniqueness types, Rust
  `&mut`/move semantics, linear types (lane-F §3.3).

### §2.2 Layer 2 — Optimized reference counting (only for EXPLICIT sharing)

**When data is *explicitly* shared, it falls to optimized RC — not the all-paths RC of a naive
counted runtime.** The optimizations:

- **Static uniqueness analysis removes RC ops** where the compiler can prove a value is uniquely
  referenced — the Perceus/Lean-4/Koka "borrowed reference" and reuse-analysis lever (lane-F §3.2,
  §4.2.1; Perceus PLDI'21; *Counting Immutable Beans* arXiv'19). A borrow does not increment/
  decrement; only genuine sharing pays.
- **Non-atomic intra-hypha; atomic after cross-hypha transfer.** Within a single hypha values are
  single-threaded, so RC updates are **non-atomic** (a significant win — lane-F §5.1). The atomic
  cost is paid **only after a value crosses a hypha boundary** into genuinely concurrent use.
  *(See §7 — exactly **which** cross-hypha values incur atomic RC is the named reconciliation
  sub-question against RFC-0027 §7.3.)*
- **`rc==1` in-place reuse (FBIP).** A value about to be dropped whose refcount is 1 has its
  allocation **reused** in place for the next value of compatible shape — Perceus/FBIP, Swift CoW,
  surface-semantically still a new value (lane-F §4.1; RFC-0027 §10.1–§10.2). Single-owner only;
  the concurrent multi-writer path is optimistic concurrency control (OCC), not a uniqueness probe
  (the embeddenator lesson — `SYNTHESIS-wave2-addendum.md` CH-1).

- **Cost:** RC traffic only on explicitly-shared values, minimized by static analysis; non-atomic
  intra-hypha.
- **Corpus basis:** RFC-0027 §7 (RC mechanism), §10.1–§10.2 (the `rc`-probe unifying reclamation +
  copy/mut); lane-F F-1/F-2/F-4.
- **This is the hardest layer to build — see §6b (KC-3 tension).**

### §2.3 Layer 3 — Region-based allocation & reclamation (within scopes)

**Within a scope, allocation is region-local and reclamation is batched at scope-exit** — bulk
efficiency, the Tofte-Talpin region discipline mapped onto the RT7 scope tree (which *already*
enforces the region-LIFO analogue at the task level — lane-B §3.1). Rather than freeing each value
with an individual `rc_dec`, a scope's region is reclaimed as a batch when the scope exits.

- **Cost:** amortized — batched bulk reclamation at scope-exit rather than per-value drops.
- **Reclamation order:** parent–child **total** (children fully reclaim before the parent exits —
  RT7); siblings **concurrent** (§3).
- **Corpus basis:** lane-B §3.1 (Tofte-Talpin region inference; RT7 already enforces the LIFO);
  RFC-0027 §10.3 (sweep-order derives from the scope tree; deferred RC drops accumulate per scope
  and flush at scope-exit). This is the layer that **mitigates OQ-3** (§6, RFC-0027 §11): batched
  region reclamation moves the worst-case drop-latency problem from an *inherent* limitation toward
  an *engineering/measurement* one.

---

## §3 Reclamation coupling — parent–child TOTAL, siblings CONCURRENT (resolves OQ-1)

The architecture answers RFC-0027 OQ-1 (the sweep-order/reclamation-order sibling-coupling question):

- **Parent–child: TOTAL.** Children fully reclaim before the parent exits — a total order along the
  scope-tree child→root path. This is forced by RT7 (a scope does not exit until its children
  complete/cancel/detach — RFC-0008 §4.1) and is not in question.
- **Siblings: CONCURRENT by default (weak / partial coupling).** Sibling scopes reclaim
  *concurrently* — the default. This is:
  - **Already licensed by RT7:** sibling scopes are concurrent *by construction* (RFC-0008 §4.1;
    research record T4.1). No mainstream structured-concurrency runtime (Swift `TaskGroup`, Kotlin
    scopes, Trio nurseries) total-orders sibling finalizers — they are concurrent by design (lane-B
    §3.7).
  - **Safe by RC + acyclicity:** LR-9 rules out cross-sibling aliases, so concurrent sibling
    reclamation has **no safety cost** — serializing siblings would buy auditability at a pure
    throughput cost with **no safety benefit** (lane-B R-2, OQ-1).
- **Strong / total coupling: OPT-IN for high-assurance subsets.** A program (or a certified subset)
  may request that reclamation order be *identical* to `SweepOrder`, so **one** property test covers
  both scheduling and reclamation (maximal auditability) — at the cost of serialized sibling cleanup.
  This is offered as an opt-in, not the default.

**Honesty (§6c):** the *safety* of concurrent sibling reclamation rests on LR-9 (acyclicity ⇒ no
cross-sibling aliases) and is `Proven`-**modulo** that side-condition (the same strength RFC-0027 §8
holds the RC-soundness claim at — external theorem, corpus-invariant side-condition, no in-repo
mechanized check yet). The **throughput benefit** of weak over strong coupling is **expected, not
measured** — `Declared`. lane-B explicitly recommended prototyping *both* and measuring before
committing; this note resolves OQ-1 *by argument* (siblings are already concurrent under RT7, and RC
+ acyclicity make order-independent reclamation safe), **not** by the prototype lane-B suggested. The
throughput claim must not be carried above `Declared` until measured (§6c).

---

## §4 Goals

The architecture's stated goals (maintainer strategic direction), each tied to the layer that serves
it:

1. **Maximum performance / throughput.** Affine ownership pays nothing on the unique path (Layer 1);
   RC engages only on explicit sharing, with static analysis removing ops and non-atomic intra-hypha
   counting (Layer 2); region reclamation batches the bulk work (Layer 3); concurrent sibling
   reclamation removes a serialization point (§3). **All performance figures are `Declared` goals
   (§6a), not measurements.**
2. **Memory safety.** Preserved from the corpus invariants: LR-8 (immutable values — no aliased
   mutation), LR-9 (acyclic — no leaks expressible, no cycle detector needed), RT7 (structured
   lifetimes — no leaked task expressible). The kernel stays Rust, `#![forbid(unsafe)]`-rooted save
   ADR-014's warned escape.
3. **"Stupid easy" ergonomics.** The default path (affine ownership) requires no annotation; sharing
   is explicit and the cost is opt-in by construction; the programmer writes ordinary value-semantic
   code and the layers pick the cheapest valid mechanism — never-silent (G2): the reuse-vs-copy and
   reclamation choices are reified in the `Provenance` DAG / EXPLAIN record (RFC-0027 §9, §10.2).

These goals **preserve LR-8/LR-9/RT7** — the architecture is an optimization *within* the existing
value-semantics + structured-concurrency invariants, not a relaxation of them.

---

## §5 Relation to the corpus & external prior art

**Corpus relation.** DN-32 is the architectural synthesis sitting above RFC-0027's resolved
mechanism (§7 RC) and resolving its open coupling question (§11 OQ-1). It draws on the same
just-landed provenance/ownership research cluster RFC-0027 §§7–11 incorporate:

- **RFC-0027** — RC mechanism (§7), guarantee tagging (§8), EXPLAIN record (§9), the `rc`-probe
  unification + sweep-order-derives-from-scope-tree (§10), the consolidated open questions (§11).
  DN-32 resolves OQ-1/OQ-4 and mitigates OQ-3.
- **RFC-0008** — RT1 (values move, state never shared), RT7 (structured lifetimes; siblings
  concurrent — §4.1), §4.3 (sweep order), §4.4 (`cyst`).
- **RFC-0006 / LR-8 / LR-9** — immutable, acyclic values; the affine `substrate` lever.
- **Research handoffs** — `lane-B-reclamation-provenance.md` (reclamation order, region discipline,
  the channel-close synchronization point), `lane-F-efficient-immutable-value-mgmt.md` (Perceus/FBIP
  unification, the RC-over-acyclic-values soundness, the honest costs), `SYNTHESIS-wave2-addendum.md`
  (RC-not-tracing-GC resolution, the OCC-vs-refcount-reuse layering lesson),
  `04-runtime-concurrency-RECORD.md` (T4.1 structured concurrency; sibling concurrency).

**External prior art (the maintainer's named references).**

- **Perceus** (Reinking, Xie, de Moura, Leijen — *Garbage-Free Reference Counting with Reuse*,
  PLDI 2021) — precise RC + FBIP reuse over cycle-free values; the basis for Layer 2's
  optimized RC and `rc==1` reuse.
- **Lorenzen, *Optimizing Reference Counting with Borrowing*** (2021 thesis) — the static
  uniqueness/borrowing analysis that removes RC ops; the basis (and the hard part — §6b) of Layer 2.
- **Koka / Lean 4** (de Moura & Ullrich, *Counting Immutable Beans*, arXiv 2019) — RC schemes for
  immutable-acyclic value models structurally identical to Mycelium's.
- **Smith, *Notes on structured concurrency, or: Go statement considered harmful*** (2018) and the
  structured-concurrency lineage (**Trio** nurseries, **Kotlin** coroutine scopes, **Java**
  `StructuredTaskScope`) — the basis for the RT7 scope tree and the **siblings-are-concurrent**
  finalization model that licenses §3's concurrent sibling reclamation.

---

## §6 Verified-parent caveats — honest scope (VR-5; do not omit)

The integrating parent verified this proposal against the corpus and attaches the following caveats
as **honest scope**. They are part of the capture, not footnotes — VR-5 forbids carrying the
proposal's claims above their supportable strength.

### §6a Performance claims are `Declared` GOALS/targets, not measurements

Every performance characterization in this note — "zero/near-zero cost," "sub-ns to low-ns," "bulk
efficiency," "maximum throughput," and the weak-coupling throughput benefit — is a **`Declared`
goal/target**. Nothing here is measured in a Mycelium implementation. The external benchmarks cited
(Perceus competitive with OCaml/GHC; Lean-4 competitive with GHC; the embeddenator ~90× single-byte-
edit speedup) are `Empirical` for **those systems**, not for Mycelium — they evidence that the
*approach* can be fast, not that this architecture *is* fast as built. Any Mycelium performance claim
stays `Declared` until a methodology'd in-repo benchmark exists.

### §6b Layer-2 static uniqueness analysis is the hardest part — a real KC-3 tension

Layer 2's **static uniqueness analysis** (Perceus reuse analysis + Lorenzen-style borrowing
inference) is the **hardest part of the architecture to build, and it grows the kernel** — a genuine
tension with **KC-3 (small auditable kernel)**. Uniqueness/borrow inference is a non-trivial analysis
pass (cf. Lorenzen, *Optimizing Reference Counting with Borrowing*, 2021): it adds compiler surface,
inference rules, and a correctness obligation. The discipline is **watch + measure as it lands** — do
not assume the analysis is small or free; track its kernel-node cost against KC-3 and the auditability
budget as each increment of it is implemented. (lane-F §4.2.2 F-5/F-6 already flags widening
uniqueness tracking as a *future* direction, not a current proposal, on exactly KC-3/YAGNI grounds —
DN-32 names it as the architecture's hardest, most kernel-growing leg.)

> **Honest-scope update (2026-06-25, append-only).** Two clarifications, neither a status move:
> (1) **Integration bound — not yet execution-wired.** The model is implemented at the **runtime tier**
> (MEM-1/2/3 — RC probe, regions, reclamation records in `mycelium-std-runtime`) and the **MEM-4 static
> tier**, and all three §9 reclamation triggers (RcZero / ScopeExit / ChannelClose) are live — **but
> reclamation is NOT yet threaded into the AOT env-machine.** The env-machine still **Rust-manages**
> values; the §9 output is an **additive audit trail of where reclamation *would* occur**, not actual
> Mycelium-level reclamation. The integration seam is
> `crates/mycelium-mlir/src/aot.rs::eval_machine` (research/16 §2). This keeps a single-doc reader from
> over-reading "the three layers" as execution-wired. (2) **§6b update — MEM-4 has since landed
> additively.** MEM-4 **Increments 1–2** are now built in `crates/mycelium-mir-passes/` (a separate,
> optimization-only crate; **Core IR `node.rs` left pristine**, so KC-3 held); the full **FIP /
> Increment-3** leg remains **Phase-3** (see §8). See DN-33 §8.1 + the E12 build plan. (Append-only;
> VR-5/G2; no status change.)

### §6c OQ-1 is resolved by ARGUMENT, not by the lane-B prototype; the throughput benefit is `Declared`

OQ-1 is resolved here **by argument** — RC + LR-9 acyclicity make reclamation safe *order-
independently* (no cross-sibling aliases), and RT7 *already* makes siblings concurrent — **not** by
the prototype-both-and-measure path lane-B recommended (lane-B OQ-1, R-2). The argument licenses the
**safety** of concurrent sibling reclamation at `Proven`-**modulo-LR-9** strength. It does **not**
license the **throughput** claim: that weak (concurrent) sibling reclamation outperforms strong
(serialized) coupling is **expected, not measured** — **`Declared`**. The follow-on RFC may still
choose to run lane-B's measurement before locking the property-test surface; resolving OQ-1 by
argument settles the *default* (concurrent), not the *measured magnitude* of its benefit.

### §6d Named reconciliation point — Layer-2 "atomic RC after cross-hypha transfer" vs RFC-0027 §7.3

See **§7** — this is a genuine reconciliation point between Layer 2 and RFC-0027 §7.3, named (not
buried) as an explicit sub-question for the follow-on DN/RFC.

---

## §7 Open sub-question for the follow-on — cross-hypha sharing vs. affine transfer (named, not buried)

**The reconciliation point.** Layer 2 (§2.2) says RC becomes **atomic after a value crosses a hypha
boundary** (for genuinely concurrent shared use). RFC-0027 §7.3 says **cross-hypha transfer rides the
affine channel protocol, NOT a distributed/cross-hypha refcount** — the `Sender`/`Receiver` pair is
affine (non-`Clone`), giving exactly-one-owner cross-hypha transfer at R1 with no cross-hypha RC.

These are not in contradiction once the cases are separated — **but the boundary between them must be
decided explicitly.** The sub-question:

> **When a value crosses a hypha boundary, is it a *shared* value (→ atomic RC, Layer 2) or *sole*
> ownership being moved (→ affine move via the channel protocol, no cross-hypha RC, RFC-0027 §7.3)?**

The two options to decide in the follow-on:

- **Option A — sole-ownership move only (RFC-0027 §7.3 as-is).** Only *uniquely-owned* values cross a
  hypha boundary, via the affine channel move. There is then **no cross-hypha shared value and no
  atomic RC** — Layer 2's "atomic after cross-hypha transfer" never fires, because a value is either
  affinely moved (Layer 1, no RC) or it never crosses the boundary while shared. Simpler; keeps RC
  strictly intra-hypha and non-atomic; matches RFC-0027 §7.3 exactly.
- **Option B — shared values may cross (atomic RC engages).** A genuinely *shared* value (rc > 1) is
  permitted to cross a hypha boundary into concurrent use, at which point its RC becomes **atomic**
  (Layer 2). This admits cross-hypha sharing beyond the affine-move model and is what Layer 2's
  "atomic after cross-hypha transfer" describes — but it reaches past RFC-0027 §7.3's "no cross-hypha
  refcount" and must define the atomic-RC ownership/release protocol (closer to Pony ORCA's deferred
  weighted refcount, which RFC-0027 §7.3 deliberately avoided via the channel close protocol).

**This is surfaced as an explicit, named sub-question for the RFC-0027 follow-on (or a dedicated DN),
with both options on the table.** It is **not** resolved here (VR-5/G2 — name it, do not bury it). It
does not block OQ-1/OQ-4: those resolve under either option (sibling reclamation concurrency and the
`rc==1` reuse-record-only default are orthogonal to the cross-hypha sharing boundary).

---

## §8 Roadmap — Phases 1–3

The maintainer's staged rollout (each phase `Declared`; sequencing follows lane-F §6's "RC first,
reuse second, static uniqueness later" ranking and the KC-3/YAGNI discipline of §6b):

- **Phase 1 — Affine ownership primary + RC for sharing (the mechanism floor).** Establish Layer 1 as
  the default (affine move for unique values; scope-exit drop) and Layer 2's *baseline* RC for
  explicit sharing (the RFC-0027 §7 mechanism), non-atomic intra-hypha. Ship the reclamation EXPLAIN
  record (RFC-0027 §9). At this phase structural sharing remains the correct default and the reuse
  check is not yet required (lane-F F-6). **Resolves OQ-4 at the record level** (rc==1 reuse is
  EXPLAIN-record-only; surface visibility deferred — §6, RFC-0027 OQ-4).
- **Phase 2 — Region-based reclamation + `rc==1` FBIP reuse.** Add Layer 3 (region-local allocation,
  batched scope-exit reclamation) and the O(1) `rc==1` in-place reuse check in `std.collections`
  mutators (RFC-0027 §10.2; lane-F F-4). This is the phase that **mitigates OQ-3** (batched region
  reclamation bounds per-epoch drop latency toward an engineering/measurement problem). Single-owner
  only; OCC stays the concurrent-writer path.
- **Phase 3 — Static uniqueness analysis (the hard, kernel-growing leg — §6b).** Add Layer 2's
  static uniqueness/borrowing analysis (Perceus reuse analysis + Lorenzen borrowing) to remove RC ops
  where uniqueness is provable, and decide the cross-hypha sharing boundary (§7). **Watch + measure
  the KC-3 cost as it lands (§6b).** Widening `substrate`-style static uniqueness from external
  resources to ordinary values (lane-F F-5) belongs here, gated on the interpreter being mature
  enough to infer it (KC-3/YAGNI).

The phasing is **low-regret-first**: Phases 1–2 deliver the architecture's safety + the bulk of its
throughput with no type-system change; Phase 3's static analysis is the highest-value, highest-cost,
most-kernel-growing leg and is sequenced last and measured.

---

## §9 Risks

- **R-1 (KC-3 / Layer-2 analysis growth — §6b).** Static uniqueness analysis grows the kernel and is
  the hardest leg. *Mitigation:* sequence it last (Phase 3); watch + measure kernel-node cost; keep
  Phases 1–2 (which need no type-system change) as the floor that delivers safety + most throughput.
- **R-2 (Performance is `Declared` — §6a).** The throughput goals are unmeasured. *Mitigation:*
  treat every figure as a target until a methodology'd in-repo benchmark exists; never upgrade a
  performance tag without measurement (VR-5).
- **R-3 (Cross-hypha sharing boundary unresolved — §7).** Layer-2 atomic-RC-after-cross-hypha vs
  RFC-0027 §7.3 affine-move. *Mitigation:* named as an explicit follow-on sub-question with both
  options; OQ-1/OQ-4 resolve independently of it.
- **R-4 (Worst-case drop latency — OQ-3).** A deep-tree drop is O(n); "no silent GC pause" is an
  honesty stance, not a sub-ms SLO (RFC-0027 §11 OQ-3; lane-F F-D). *Mitigation:* Layer 3 batched
  region reclamation + the sweep-epoch model spread the cost; the SLO stays `Declared` pending the
  fuel-model bound (a dedicated research note is warranted — RFC-0027 OQ-3).
- **R-5 (Resolution-by-argument vs. measurement — §6c).** OQ-1 resolved by argument, not the lane-B
  prototype. *Mitigation:* the *safety* is argued (`Proven`-modulo-LR-9); the *throughput* benefit
  stays `Declared`; the follow-on may still measure before locking the property-test surface.

---

## Meta — changelog

- **2026-06-25 — Created (Accepted (ratified 2026-06-25); status flagged).** Captures the maintainer's
  **three-layer hybrid memory architecture**: (1) affine/linear ownership PRIMARY for unique data
  (zero/near-zero cost, default path), (2) optimized reference counting only for EXPLICIT sharing
  (static uniqueness analysis removes ops; non-atomic intra-hypha, atomic after cross-hypha transfer;
  `rc==1` in-place reuse), (3) region-based allocation & reclamation within scopes (region-local
  alloc, batched scope-exit reclamation). Parent–child reclamation TOTAL; sibling reclamation
  CONCURRENT by default (weak/partial coupling), strong coupling opt-in for high-assurance subsets.
  Goals: max performance/throughput + memory safety + "stupid easy" ergonomics, preserving
  LR-8/LR-9/RT7. Roadmap Phases 1–3; risks; corpus + external-prior-art relation (Perceus PLDI'21,
  Smith 2018, Lorenzen 2021 thesis, Koka/Lean 4, structured-concurrency lit). **Resolves RFC-0027
  OQ-1 (siblings concurrent, safe by RC+acyclicity; strong coupling opt-in) and OQ-4 (rc==1 reuse =
  EXPLAIN-record-only by default); mitigates OQ-3 (regions + batched reclamation).** Verified-parent
  caveats incorporated as honest scope (§6): (a) performance claims are `Declared` goals/targets, not
  measured; (b) Layer-2 static uniqueness analysis is the hardest part + a real KC-3 tension —
  watch + measure as it lands (Lorenzen); (c) OQ-1 resolved by **argument** (RC + LR-9 acyclicity ⇒
  order-independent safety; RT7 ⇒ siblings already concurrent), NOT the lane-B prototype — the
  weak-over-strong throughput benefit is expected-not-measured (`Declared`); (d) the named
  reconciliation sub-question (§7): Layer-2 "atomic RC after cross-hypha transfer" vs RFC-0027 §7.3
  "cross-hypha rides the affine channel, no cross-hypha refcount" — does a SHARED value cross a hypha
  boundary (→ atomic RC) or only SOLE ownership (→ affine move, no cross-hypha RC)? Both options
  tabled for the follow-on. Enacts nothing; moves no normative status by itself; all performance
  figures `Declared`. **⚠️ Status move to a full `Accepted` requires maintainer ratification (house
  rule #3, append-only).** Touches no other doc's normative text; CHANGELOG.md / issues.yaml /
  docs/api-index are owned by the integrating parent. (Append-only; VR-5; G2.)
- **2026-06-25 — Honest-scope update (§6b note; append-only; no status move).** Added a §6 callout
  recording, per an alignment audit, that the model is implemented at the runtime tier (MEM-1/2/3) and
  the MEM-4 static tier with all three §9 triggers live, **but reclamation is not yet threaded into the
  AOT env-machine** (env-machine still Rust-manages values; §9 output is an additive audit trail; seam =
  `crates/mycelium-mlir/src/aot.rs::eval_machine`, research/16 §2); and that MEM-4 Increments 1–2 have
  since landed **additively** in `mycelium-mir-passes` (Core IR pristine — KC-3 held; full FIP/
  Increment-3 stays Phase-3). Status remains **Accepted**; no normative text changed. (Append-only;
  VR-5; G2.)
</content>
