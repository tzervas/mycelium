# Design Note DN-12 — RFC-0020 (L2 Surface Term Language) Ratification-Readiness

| Field | Value |
|---|---|
| **Note** | DN-12 |
| **Status** | **Draft / Resolved-as-capture** (a ratification-*readiness* assessment, advisory — the same role DN-07 played for RFC-0016). Records that RFC-0020 carries **no research gate**, maps its deferred items onto the now-research-discharged RFC-0018/0019, and recommends a **scoped** ratification with a carve-out. Decides nothing normatively. |
| **Decides** | *Nothing normatively.* Frames RFC-0020's ratification readiness for the maintainer's append-only decision. |
| **Feeds** | `docs/rfcs/RFC-0020-L2-Surface-Term-Language.md` §10; RFC-0006 §10 (the carve-out precedent, r5); RFC-0018 / RFC-0019 (the deferred-item dependencies, both now research-discharged — `research/09`, `research/10`); RFC-0001 r5 `FixGroup` (the R20-Q4 dependency, enacted). |
| **Date** | June 18, 2026 |
| **Task** | (none — readiness capture; relates to RFC-0020) |

> **Posture (honesty rule / VR-5).** Advisory readiness assessment. It moves RFC-0020's status not at
> all; it organizes the ratification decision for the maintainer. Append-only.

---

## 1. The finding: RFC-0020 has no research gate

Unlike RFC-0018 (RP-2) and RFC-0019 (RP-3), **RFC-0020 is gated by no research prompt.** Its scope is
KC-2-independent: the §8 Q1 surface commitment (DN-09) already settled the L3 strategy, and L2 is the
*elaboration layer beneath* that committed surface, with **no independent semantics** (every construct
desugars to L1; identity is the content-addressed L1, never L2 syntax). Its deferred items are gated
on **other RFCs landing**, not on any experiment or unproved soundness argument. There is therefore no
"necessary research" to conduct for RFC-0020 — the honest assist is this readiness assessment.

## 2. What is ratifiable now (the scoped core)

Per RFC-0020 §10, the following are complete and self-contained (KC-2-independent, no external
dependency):

- **§4.1 — the L2 invariants S1–S6** restated at the elaboration boundary (no silent swap; honest
  tags surface; content-addressed identity; inspectable elaboration; explicit partiality;
  AI-independence). These are restatements of ratified RFC-0006 §4.1 invariants.
- **§4.3 — modules** (`nodule`/`use`, content-addressed per ADR-003/LR-3).
- **§4.4 — pattern sugar** (Maranget compilation to flat `Match`, usefulness/exhaustiveness checks).
- **§4.6 — literal handling** (universal-until-elaboration; Q6 discharged by DN-09).
- **§4.7 — ambient paradigm** (the enacted RFC-0012 model instance; elaborates to identical L0).
- **§4.8 — usability-first design bias** (the DN-09 §3.2 maintainer direction).
- **§4.9 — the conformance-corpus plan** (extends RFC-0006 §4.3; a testing requirement, not a gate).

## 3. What stays deferred — and why those dependencies are now unblocking

The deferred sections depend on sibling RFCs, **two of which had their research gates discharged this
same pass** (so the dependency chain is now moving, not stalled):

| RFC-0020 deferred item | Depends on | Dependency status (2026-06-18) |
|---|---|---|
| §4.2 polymorphic instantiation; §4.5 `grow`-derived traits (R20-Q1) | **RFC-0019** (traits) | Research gate **discharged** (RP-3 → `research/10`); awaits maintainer design decisions. |
| §4.2 guarantee-grade inference integration (R20-Q2) | **RFC-0018** (grading) | Research gate **discharged** (RP-2 → `research/09`); awaits maintainer R18-Q1/Q4. |
| §4.4 mutual-recursion elaboration (R20-Q4) | **RFC-0001 r5** `FixGroup` | **Enacted** (M-343); the surface front-end is **M-391** (gated on RP-6 grammar choice, DN-11). |
| or-patterns (R20-Q3) | internal design | Open; reserved direction; not blocking the scoped core. |
| list-literal bidirectional inference in `for` (R20-Q5) | internal design | Conservative v0 answer (explicit annotation); tracked improvement. |

None of these block the §2 scoped core; each is marked `deferred` in the RFC and unblocks as its
sibling lands.

## 4. Recommendation

**Ratify RFC-0020 at the scope of §4.1/§4.3/§4.4/§4.6/§4.7/§4.8/§4.9, with an explicit carve-out** for
the deferred sections — exactly the pattern RFC-0006 used at r5 (Accepted with `Q1/Q6` and stage-1
grading carved out). The carve-out names §4.2 (polymorphic + grade inference), §4.5 (`grow`), and
R20-Q1…Q5 as "deferred, unblocking with RFC-0018/0019/0001-r5," not as open objections to the core.

This is a **maintainer decision** (append-only). No LLM measurement and no soundness argument are
required — the readiness is a matter of scoping, which this note frames. The one remaining build
obligation (the §4.9 conformance corpus) is a *testing* deliverable that follows ratification, not a
gate on it.

---

## Changelog

- **2026-06-18 — Draft / Resolved-as-capture.** Records that RFC-0020 carries no research gate; maps
  its deferred items onto the now-research-discharged RFC-0018 (RP-2/`research/09`) and RFC-0019
  (RP-3/`research/10`) and the enacted RFC-0001 r5 `FixGroup`; recommends a scoped ratification with a
  carve-out (the RFC-0006 r5 precedent). Advisory; RFC-0020 status unchanged. Append-only.
