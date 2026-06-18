# Design Note DN-11 — Next-Wave Plan: the post-KC-2-unblock work, grouped and gated

| Field | Value |
|---|---|
| **Note** | DN-11 |
| **Status** | **Draft / Resolved-as-capture** (planning capture, advisory — see posture). Indexes the next wave of work into dependency-ordered tracks with their gates; mints task ids for the two L1 gaps that had none. Decides nothing normatively. |
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

## Changelog

- **2026-06-18 — Draft / Resolved-as-capture.** Initial capture of the post-#194 wave as three
  dependency-ordered tracks (A: DN-10 L1 completion; B: RFC-0018…0021 ratification gated on
  RP-1…RP-4; C: Phase-5 stdlib gated on M-501/M-502). Mints M-390 (R7-Q4) and M-391 (R7-Q3).
  Records the stdlib spec-vs-code status nuance honestly. Grounded in DN-09, DN-10, RFC-0017,
  RFC-0016, research-prompts.md (RP-1…RP-7), RFC-0001 r3/r5, ADR-003. Advisory; append-only.
