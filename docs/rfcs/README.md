# RFCs

An RFC is a detailed, normative design of a subsystem that multiple decisions plug into. RFCs are **append-only** with status transitions and cite their grounding (survey `G*/A–E/R*`, research `T0/T1/T2`, and the ADRs they implement).

## Status set

`Draft → Accepted → Superseded`. RFC-0001…0005 are **Accepted** (solidified from the two research passes); **RFC-0006 is Accepted (r5, 2026-06-18)** and **RFC-0007 Accepted (r4)** — the layering / L1 v0 calculus are ratified, and the **KC-2 verdict (DN-09, 2026-06-18) = proceed** has now **committed the concrete L3 text surface** (+ a co-equal projection layer), discharging the prior KC-2 gate (RFC-0006 §10 Q1). **RFC-0017** (maturation scope & de-maturation) is **Accepted**. The follow-on surface/type-system designs are new **Drafts**: **RFC-0018** (stage-1 grading), **RFC-0019** (traits/LR-2), **RFC-0020** (L2 surface), **RFC-0021** (projections) — each carrying a pre-ratification research prompt (VR-5). RFC-0008/0010 are **Drafts**.

## Index

| RFC | Title | Status | Implements / depends on |
|---|---|---|---|
| 0001 | Core IR & Metadata Schema | Accepted (r2) | Foundation FR-M*/VR-*; ADR-001/002/003/006/008; ADR-010 (§4.7); ADR-011 (§4.3 `Bound`) |
| 0002 | Swap Certificate & Split Regime | Accepted | RFC-0001; ADR-002; ADR-010; shares the certificate checker with RFC-0004 |
| 0003 | VSA Submodule Boundary | Accepted (r2) | RFC-0001; ADR-008; ADR-010; ADR-013 (r2 `spore` scope note) |
| 0004 | Execution Model & "Stable Component" | Accepted | RFC-0001; ADR-007; ADR-009; DN-01; shares the checker with RFC-0002 |
| 0005 | Selection-Policy Language | Accepted | RFC-0001; ADR-006; same mechanism used by RFC-0002 & RFC-0004 |
| 0006 | Surface Language, Grammar & Term-Language Layering | **Accepted** (r5; **concrete L3 text surface committed** — DN-09 KC-2 verdict, Q1 discharged; stage-1 grading → RFC-0018) | RFC-0001 §4.5; ADR-003/006/007; KC-2/KC-3; RR-3; DN-09; SPEC §10.1/§10.2 |
| 0007 | The L1 Kernel Calculus | **Accepted** (r4; v0 calculus; surface now committed via DN-09; `matured` granularity → RFC-0017; R7-Q1…Q4 deferred — §10) | RFC-0006; RFC-0001 §4.5/§4.6; RFC-0004 §4; ADR-003; T3.1/T3.4 |
| 0008 | Runtime & Concurrency Execution Model | **Draft** | RFC-0004 (extends, per-node model unchanged); RFC-0001/0002/0005/0006/0007; ADR-012 §7.3; T4.1–T4.6 |
| 0009 | Resonator-Network Factorization | Accepted | RFC-0003 §6; RFC-0001; FR-C2/G2/G4; RR-5/VR-5 (M-350) |
| 0010 | Decode-Methodology Selection | **Draft** | RFC-0005 (same selection mechanism, third site); RFC-0009 (§10.3 matrix + regime gate); RFC-0003; G2/G4/VR-5 |
| 0011 | L0 `Match` & the L1-in-Core-IR Revision | **Accepted** (decision; enactment sequenced after RFC-0006/0007 ratification) | RFC-0001 §4.5/§4.6 (r3 revision); RFC-0006 §4.4 step 2; RFC-0007 §4.1–4.6; ADR-003; M-320 |
| 0012 | Ambient Representation: Declared Paradigm Defaults & Scoped Overrides | **Accepted** (design normative; enactment gated — M-344) | RFC-0006 (surface/term-layer; no kernel change); RFC-0005/ADR-006 (reified selection); RFC-0001 §4.5/§4.6 (content-addressing); ADR-016 (cross-module ABI); G2/KC-3/tension A; NFR-7 |
| 0013 | Structured Diagnostics & Reified Error-Handling Policy | **Draft** (presentation/routing only; recovery deferred — M-345) | DN-04 (basis); RFC-0005/ADR-006 (reified-policy pattern); RFC-0006 (optional surface); RFC-0008 (observability home); RFC-0001/0002 (the never-silent errors it presents); G2/G11/NFR-2/KC-3; ADR-007 |
| 0014 | Declarative Error Recovery & Bounded Effects | **Draft** (the isolated recovery subsystem RFC-0013 deferred; design open) | RFC-0001 (error values); RFC-0013 (sibling — shared registry/pattern, presentation vs. recovery); RFC-0005/ADR-006 (reified policy); RFC-0006 (surface); RFC-0004 §4 / RFC-0007 §4.5 / M-347 / DN-05 (the budget discipline bounded effects generalise); G2/VR-5/KC-3/SC-3 |
| 0015 | Automatic Baseline Diagnostics & Recovery | **Accepted — Enacted** (2026-06-16, M-362) | RFC-0013 (additive presentation it auto-applies) + RFC-0014 (opt-in declared recovery); RFC-0005/ADR-006 (reified EXPLAIN-able policy); RFC-0008 §4.8/RFC-0013 §8 (sinks); G2/VR-5/KC-3/NFR-2 |
| 0016 | Core Library & Standard Library | **Accepted** (2026-06-17, maintainer ratification M-501/DN-07 — §4.1 per-op contract + §4.3/4.4 taxonomy + §4.2 ring layering + §4.5 guarantee-matrix obligation + §4.6 migration order; §8 Q3/Q4 deferred-with-direction; §4.2 `spore`-ring erratum 2026-06-20) | M-346 (the stdlib epic it is the "Core Library RFC" for); RFC-0001 (value model); RFC-0002/0003/0005/0008/0013/0014 (the differentiator modules' designs); ADR-003/007/010/013/014; G2/G11/VR-5/KC-2/KC-3 |
| 0017 | Maturation Scope & De-maturation | **Accepted** (2026-06-18; `matured` → nodule/phylum/program scope, `matured fn` retired, `thaw` de-maturation; supersedes RFC-0007 §4.5 *granularity*, gate soundness unchanged) | RFC-0007 §4.5; RFC-0004 §4; DN-06/DN-08; Nodule-Header spec; DN-02/DN-03; ADR-003; KC-3/G2/VR-5 |
| 0018 | Stage-1 Static Guarantee Grading | **Draft** (graded coeffect over the guarantee semilattice; the implicit-flows decision R18-Q1 + research prompt gate ratification) | RFC-0007 §4.3/§4.4 (revision); RFC-0006 §8 Q3; RFC-0002 (Swap certificate = endorsement); LR-6; VR-5; T3.2 |
| 0019 | Traits & Parametric Polymorphism (LR-2) | **Draft** (dictionary-passing to L1, coherence; Repr-poly LR-5 + guarantee-indexed methods flagged-novel + research prompts) | RFC-0007 §4.4 (the deferral); RFC-0006 LR-2/LR-5/LR-6; ADR-003; DN-03; KC-3; T3.3 |
| 0020 | The L2 Surface Term Language | **Draft** (elaboration-defined surface: inference, modules, pattern sugar, derived forms; usability-first) | RFC-0006 §3 (L2); RFC-0007 (L1 target); RFC-0011/0012; DN-09 §3.2; LR-1/LR-3; ADR-003/006; KC-3 |
| 0021 | Semantic-Level Projection Framework | **Draft** (M-380/FR-C1/G11; views over content-addressed defs; LLM-facing canonical projection — gated on the G11/T3.6 research prompts) | RFC-0006 §3/§9 (L3, FR-S5); DN-09 §3.1; RFC-0001 §4.6/ADR-003; ADR-006; FR-C1/G11; RR-3 |

Cross-cutting machinery (decided across RFCs): **one** certificate checker (RFC-0002 ⇄ RFC-0004), **one** selection mechanism (RFC-0002 ⇄ RFC-0004), and **two** bound kernels meeting at one shared certificate (ADR-010 → RFC-0001/0002/0003). See `../Doc-Index.md` for the dependency DAG.

## Relationship to ADRs

An **ADR** records *a* decision; an **RFC** designs *a subsystem* and may rest on several ADRs. When an RFC makes a new architectural decision in passing, capture that decision as an ADR too, so it's discoverable from the ADR index.

## Template

```markdown
# RFC-NNNN — <title>

| Field | Value |
|---|---|
| **RFC** | NNNN |
| **Status** | Draft \| Accepted \| Superseded |
| **Type** | Foundational / normative \| Informational |
| **Date** | YYYY-MM-DD |
| **Depends on** | <RFCs / ADRs / Foundation labels> |

## 1. Summary
## 2. Motivation
## 3. Guide-level explanation
## 4. Reference-level design (normative)
## 5. Drawbacks
## 6. Rationale & alternatives
## 7. Prior art
## 8. Unresolved questions
## 9. Future possibilities

## Meta — changelog
```
