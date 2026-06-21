# Design Note DN-11 — Next-Wave Plan: the post-KC-2-unblock work, grouped and gated

| Field | Value |
|---|---|
| **Note** | DN-11 |
| **Status** | **Resolved** (2026-06-21 — Phase-5 completion summary, remaining gate items, and Phase-6 roadmap appended below (§5); M-650 editorial sweep. Append-only.) **Draft / Resolved-as-capture** (planning capture, advisory — see posture). Indexes the next wave of work into dependency-ordered tracks with their gates; mints task ids for the two L1 gaps that had none. Decides nothing normatively. |
| **Decides** | *Nothing normatively.* Frames the wave the #194 KC-2/RFC-0017 ratification unblocked as three tracks, names the leading item, and records honest status nuances. |
| **Feeds** | `docs/notes/DN-10-Remaining-L1-Gaps.md` (Track A); `docs/notes/research-prompts.md` (RP-1…RP-7 gates); RFC-0018/0019/0020/0021 (Track B, all Draft); RFC-0016 + `docs/spec/stdlib/` (Track C); `tools/github/issues.yaml` (the M-390/M-391 mint). |
| **Date** | June 18, 2026 |
| **Task** | (none yet — wave-planning capture; mints M-390/M-391 for Track A) |

> **Posture (honesty rule / VR-5).** Advisory **planning capture** — the same role DN-08 played
> for maturation and DN-10 plays for the L1 gaps. This note reopens no ratified decision and grades
> nothing `Proven`; it groups, orders, and cross-links work the corpus already records. **This note
> itself moves no RFC toward Accepted** — each Track-B RFC needs its research prompt run first (no
> Draft→Accepted without a checked basis, VR-5). Append-only: supersede, do not rewrite.
>
> **Update (2026-06-18, same wave — forward pointer, append-only).** The Track-B research was
> subsequently discharged (RP-2/RP-3 → `research/09`–`10`; RP-4 design grounded → `research/11`) and
> the maintainer ratified: **RFC-0018/0019/0020 → Accepted**, **RFC-0021 → Accepted (framework)** with
> the empirical leverage run split into the non-blocking **M-381**. The "Draft" framing below is this
> note's *as-written* planning capture (retained verbatim); the current statuses are the RFC headers +
> the CHANGELOG Decided entries.

---

## 1. Context — what #194 unblocked

The latest decision on `main` (#194, 2026-06-18) ratified the **KC-2 verdict** (DN-09 → Resolved:
*proceed* — the "irrecoverable collapse" criterion was not triggered) and **RFC-0017** (maturation
scope / de-maturation). That ratification *unblocked* a "Wave 2 maturation" batch that currently
sits as un-started design debt:

- two **L1 kernel gaps** captured in DN-10 (R7-Q3, R7-Q4) — **with no task id**;
- four **Draft RFCs** (RFC-0018…RFC-0021);
- seven **standing research prompts** RP-1…RP-7 (`research-prompts.md`), each with a falsification
  threshold, that gate the above.

This note is the index a future pass starts from: it sorts that debt into three tracks, each
dependency-ordered with an explicit gate, and names the **leading item** (Track A → R7-Q4).

---

## 2. The three tracks

### Track A — L1 completion (DN-10) — *leading track*

The two remaining L1 elaboration/registry gaps. Both are **purely additive** in the RFC-0001
r5 / RFC-0007 r4 world (DN-10 §4) and independent of each other.

| Task | What | Gate / blocker | Mirrors |
|---|---|---|---|
| **M-390** (new) | **R7-Q4** — migrate the hard-coded prim signature table `Π` into **content-addressed prim declarations** (DN-10 §3). | None open. The intrinsic-guarantee-with-citation sub-question is **RP-7** (a spike, deferred — all v0 prims are `Exact`, so it does not block the migration). | The data registry `Σ` (`mycelium-core::data::DataRegistry`). |
| **M-391** (new) | **R7-Q3** — surface elaboration for **mutually-recursive** function groups (DN-10 §2). | **Gated on RP-6** — the surface-grammar choice (`let rec … and …` vs nodule-boundary fixpoint vs explicit block) must be decided first. | The existing Tarjan SCC → `FixGroup` path in `mycelium-l1::elab`. |

**Leading item: M-390 (R7-Q4).** Chosen over M-391 because it has *no open surface-grammar
decision* (M-391 is blocked on RP-6), is independent of every currently-open RFC, and follows
directly from RFC-0001 r3 (`Σ`) — the registry pattern it mirrors already exists and is tested.

### Track B — Wave-2 RFC ratification (KC-2-unblocked) — *gated on spikes*

Four RFCs that were **Draft when this note was written**, each held Draft until its research prompt
produces a checked basis (VR-5). *(Same-wave outcome, per the posture update above: all four were
subsequently ratified — RFC-0018/0019/0020 Accepted, RFC-0021 Accepted (framework) — with the
empirical leverage run isolated as non-blocking M-381. The table below is the as-written gating
capture.)*

| RFC (Draft as-written) | What | Gating prompt(s) |
|---|---|---|
| **RFC-0018** | Stage-1 static guarantee grading | **RP-2** — implicit-flows decision + a noninterference proof over the 4-point integrity lattice (flagged novel — the soundness argument must be *constructed*, not cited). |
| **RFC-0019** | Traits & Repr-polymorphism | **RP-3** — coherence mechanism + the S1-preserving restriction set (no instantiation silently inserts a `Swap`). |
| **RFC-0020** | L2 surface term language | No blocking RP (usability-first; elaboration-defined). |
| **RFC-0021** | Semantic-projection framework (M-380) | **RP-1** (retention-ratio ablation — reuses the existing KC-2 harness in `experiments/`) + **RP-4** (projection ergonomics / LLM-facing canonical projection). |

### Track C — Phase-5 stdlib enactment — *gated on the Phase-4 gate*

| Task | What | Gate |
|---|---|---|
| **M-501** (keystone) | Core Library RFC — scope, module boundaries, the per-op guarantee/EXPLAIN contract, the Rust→Mycelium migration path. | RFC-0016 ratification-readiness (DN-07) is met; this is the keystone the stdlib build hangs off. |
| **M-502** | Self-hosting readiness — the surface language must be sufficient to *author* stdlib modules in Mycelium. | Track A R7-Q3 (surface maturity) feeds this. |
| **M-510…M-534** | The 23-module stdlib (Tier-A differentiators + Tier-B commons). | Gated on M-501 + M-502; **design specs already written** (see §3). |

---

## 3. Status reconciliation (honest)

Two `issues.yaml`/corpus nuances recorded here rather than silently "fixed":

1. **Stdlib: specs exist, code does not.** The 23 stdlib **design specs** are written
   (`docs/spec/stdlib/*.md`, the M-510–M-534 design wave), while the M-510…M-534 issues remain
   `status:needs-design`. This is *correct*, not a lag to paper over: the `status` field tracks
   **code**; a design spec is a doc. The honest reading is "design specced, implementation
   needs-design" — the statuses are **not** flipped to `done` here.
2. **The L1 gaps had no task id.** DN-10 framed R7-Q3/R7-Q4 as build tasks with `(none yet)` for
   the task field. This note mints **M-390** (R7-Q4) and **M-391** (R7-Q3) and adds them to
   `tools/github/issues.yaml` under the Phase-4 (AOT-fragment / L1-completion) milestone, with
   `depends_on` per DN-10 §2.5/§3.5. GitHub issue minting (MCP) and the `idmap.tsv` append are a
   *separate* step done only when the issues are actually created — not by this note.
3. **One blocked item, out of wave scope.** `M-348` (provision libMLIR) is the sole
   `status:blocked` entry; it gates the native MLIR path, not this wave.

---

## 4. Dependency map

```
RFC-0001 r3 (data registry Σ) ─────────────► M-390 (R7-Q4 prim declarations)   ◄── leading item
                                                  └─ RP-7 (prim BoundBasis schema — spike, deferred)

RFC-0001 r5 (FixGroup) ──► RP-6 (surface grammar) ──► M-391 (R7-Q3 surface mutual recursion)

KC-2 unblock ──► RP-2 ─► RFC-0018   RP-3 ─► RFC-0019   (RP-1+RP-4) ─► RFC-0021   RFC-0020
                                                                                       │
RFC-0016 / DN-07 ──► M-501 ──► M-502 ──► M-510…M-534 (specs written; code gated)  ◄────┘
```

The tracks are independent: Track A needs no Track-B RFC; Track B's spikes are parallelizable;
Track C waits on the Phase-4 gate. The leading item (M-390) blocks nothing else and is unblocked.

---

---

## 5. Phase-5 completion summary and Phase-6 roadmap (2026-06-21, M-650)

> **Posture.** Same advisory posture as §§1–4 above. `done` items cite their M-xxx; open questions
> are marked explicitly (VR-5). This section does not ratify any decision.

### 5.1 Phase-5 completion: what landed (M-5xx done)

All three tracks from §2 are closed on their Rust-first scope.

**Track A — L1 completion (DN-10) — DONE.**
- **M-390** (R7-Q4 — prim declarations): content-addressed prim signature table enacted in `mycelium-core::data`. Done (2026-06-18).
- **M-391** (R7-Q3 — mutual-recursion surface elaboration): Tarjan SCC → `FixGroup` landed in `mycelium-l1::elab`; RP-6 surface-grammar spike resolved. Done (2026-06-19).

**Track B — Wave-2 RFC ratification — DONE (all four Accepted).**
- **RFC-0018** (Stage-1 Static Guarantee Grading) — Accepted (2026-06-18, `research/09`, R18-Q1 = Design A).
- **RFC-0019** (Traits & Parametric Polymorphism) — Accepted (2026-06-18, `research/10`; coherence = orphan + global-uniqueness).
- **RFC-0020** (L2 Surface Term Language) — Accepted (scoped) (2026-06-18, DN-12; §4.2/§4.5 carved out).
- **RFC-0021** (Semantic-Level Projection Framework) — Enacted (framework) (M-380 LlmCanonical renderer landed; M-648 editorial sweep).

**Track C — Phase-5 stdlib enactment — DONE (Rust-first scope).**
- **M-501** (RFC-0016 ratification): Accepted (2026-06-17). RFC-0016 → Enacted (2026-06-21, M-648).
- **M-510…M-534** (23 stdlib crates): all done (Rust-first; 1883+722+230 tests; guarantee matrices asserted). Accepted (scoped, 2026-06-20 maintainer ratification). Self-hosting migration half (M-502) stays Phase-5-C/M-502-gated.
- **M-540** (per-ring ergonomics design pass, RFC-0016 §8-Q3): done.
- **M-541** (FFI inventory, `std-sys` phylum floor): done.
- **RFC-0017** — Enacted (2026-06-21, M-648; `thaw`/scope-`matured` in `mycelium-l1`).
- **M-381 / M-646** (LLM-leverage ablation + LlmCanonical scorer): done. Retention ratio determinate (DN-09 §10: grok-build-0.1 5.50×; grok-4.3 2.20× — both ≥ ~70%, RFC-0021 §4.7 trigger does NOT fire). Arms 3/5 backlogged per ADR-021 §5.

### 5.2 Phase-5 remaining gate items

The Phase-5 gate (per `docs/planning/phase-5.md`) has two items still open or deferred:

| Item | Status | Note |
|---|---|---|
| **M-647** (RFC-0020 scoped ratification) | **Done** (2026-06-21) | RFC-0020 Accepted (scoped); DN-12 Resolved. |
| **M-648** (editorial enactment sweep) | **Done** (2026-06-21) | This note is part of M-648. |
| **M-649** (first stdlib module in Mycelium-lang) | **DEFERRED (post-1.0)** | 5 gate-fails block self-hosting (generics, trait interfaces, effect annotations, wild/FFI, static guarantee index). Scoped to Phase-6 per ADR-021 §5. |
| **M-502** (self-hosting readiness verdict) | **Not yet established** (honest) | Verdict stays `not-yet` until M-649 gate-fails resolve. DN-14 records the honest 5/5 split. |

### 5.3 Phase-6 roadmap (high-level; open questions noted)

Phase-6 is not decomposed here — this is the planning capture. Each item below is either grounded in a done RFC/ADR or marked an open question (VR-5).

**1. Stage-1 generics and traits (unblocks M-649 gate-fails 1 + 2).**
RFC-0018 (Accepted) and RFC-0019 (Accepted) define the *design*; neither is yet Enacted (no implementation). A Phase-6 wave implements the graded coeffect type judgment `Γ; pc ⊢ e : τ @ g` (RFC-0018 §4) and `impl Trait for T` elaboration to dictionary-passing L1 (RFC-0019 §4). *Open question:* which subset of generics to add first (monomorphic grade polymorphism → bounded grade polymorphism, per RFC-0018 §5's staged path) — RFC-0007 amendment scope is a maintainer decision.

**2. Effect annotations (unblocks M-649 gate-fail 3).**
RFC-0014 §8 deferred `fn f() -> T / {time}` declared-effects surface to stage-1. A Phase-6 spec amends RFC-0007 and RFC-0014 to surface `/ {…}` effect annotations in the elaborator. *Open question:* ordering relative to generics/traits.

**3. Native codegen increments (DN-15).**
DN-15 §4 table: closures+heap (non-recursive `Construct`/`Match` landed; closures next), then recursion+DN-05-priority-1 stack-robustness, then the real MLIR dialect (libMLIR-gated, M-348). *Open question:* timing of M-348 (libMLIR provisioning).

**4. Self-hosting expansion.**
Once Stage-1 generics/traits land, the `std.ternary`, `std.math` (pure fragments), and `std.option` modules become self-hosting candidates (DN-14 §4 ranking). The three-way differential obligation (RFC-0016 §4.6 M-210 bar) applies. *Open question:* which module is the first milestone.

**5. Research gaps not yet closed.**
- **RP-7** (prim `BoundBasis`-with-citation schema) — deferred from M-390 as a spike; no open issue yet.
- **arm-3 / arm-5 live runs** (M-381) — backlogged per ADR-021 §5; resumption needs a local GBNF/llama.cpp model.
- **RFC-0018/0019 full enactment** — unblocks §4.2/§4.5 carve-outs from RFC-0020. *Open question:* timeline/priority.

---

## Changelog

- **2026-06-21 — Resolved (M-650 editorial sweep).** Phase-5 completion summary (§5.1), remaining gate items (§5.2), and Phase-6 roadmap (§5.3) appended. All Track-A/B/C items confirmed done or explicitly deferred with honest gate-status. DN-11 Status → Resolved. Grounded in M-390/391/501–534/540/541/646/647/648; open questions marked per VR-5. Append-only.
- **2026-06-21 — Phase-6 gap-closure wave cross-reference (append-only).** A Sonnet research
  agent produced a grounded gap inventory covering the delta between the ratified lexicon/spec and
  the current `mycelium-l1` implementation. Two new epics
  were minted and added to `tools/github/issues.yaml`: **E7-1** (L1 Stage-1 Language Completeness:
  generics, traits, effect annotations, wild/FFI, static guarantee grading, phylum + cross-nodule;
  Phase 5) and **E7-2** (RFC-0008 Runtime Vocabulary: lexer reservation → L1 construct activation,
  R1 then R2; Phase 7). E7-1 children: M-656…M-664 (9 issues). E7-2 children: M-665…M-668 (4
  issues). The tracks are: E7-1 gates DN-14 self-hosting (all 6 gate-fails + 1 missing-partial
  row); E7-2 gates concurrency dogfooding (RFC-0008 §4.6 R1/R2 vocabulary). Both are independent
  of each other (E7-2 can start immediately with M-665 lexer-only change). These IDs supersede the
  informally-noted "Stage-1 generics/traits RFC amendments" and "Effect annotation scope decision"
  placeholder descriptions that M-650 referenced. Append-only.

- **2026-06-18 — Draft / Resolved-as-capture.** Initial capture of the post-#194 wave as three
  dependency-ordered tracks (A: DN-10 L1 completion; B: RFC-0018…0021 ratification gated on
  RP-1…RP-4; C: Phase-5 stdlib gated on M-501/M-502). Mints M-390 (R7-Q4) and M-391 (R7-Q3).
  Records the stdlib spec-vs-code status nuance honestly. Grounded in DN-09, DN-10, RFC-0017,
  RFC-0016, research-prompts.md (RP-1…RP-7), RFC-0001 r3/r5, ADR-003. Advisory; append-only.
