# RFCs

An RFC is a detailed, normative design of a subsystem that multiple decisions plug into. RFCs are **append-only** with status transitions and cite their grounding (survey `G*/A–E/R*`, research `T0/T1/T2`, and the ADRs they implement).

## Status set

`Draft → Accepted → Superseded`. RFC-0001…0005 are **Accepted** (solidified from the two research passes); **RFC-0006 and RFC-0007 are now Accepted (r4, 2026-06-15)** with a scoped carve-out — the layering / L1 v0 calculus are ratified, while concrete L3 surface syntax stays **KC-2-gated** and stage-1 grading is deferred (each RFC's §10). RFC-0008 (runtime model) and RFC-0010 are **Drafts**.

## Index

| RFC | Title | Status | Implements / depends on |
|---|---|---|---|
| 0001 | Core IR & Metadata Schema | Accepted (r2) | Foundation FR-M*/VR-*; ADR-001/002/003/006/008; ADR-010 (§4.7); ADR-011 (§4.3 `Bound`) |
| 0002 | Swap Certificate & Split Regime | Accepted | RFC-0001; ADR-002; ADR-010; shares the certificate checker with RFC-0004 |
| 0003 | VSA Submodule Boundary | Accepted (r2) | RFC-0001; ADR-008; ADR-010; ADR-013 (r2 `spore` scope note) |
| 0004 | Execution Model & "Stable Component" | Accepted | RFC-0001; ADR-007; ADR-009; DN-01; shares the checker with RFC-0002 |
| 0005 | Selection-Policy Language | Accepted | RFC-0001; ADR-006; same mechanism used by RFC-0002 & RFC-0004 |
| 0006 | Surface Language, Grammar & Term-Language Layering | **Accepted** (r4; concrete L3 syntax KC-2-gated, stage-1 grading deferred — §10) | RFC-0001 §4.5; ADR-003/006/007; KC-2/KC-3; RR-3; SPEC §10.1/§10.2 |
| 0007 | The L1 Kernel Calculus | **Accepted** (r4; v0 calculus — stage-1 grading / R7-Q1…Q4 / surface syntax deferred — §10) | RFC-0006; RFC-0001 §4.5/§4.6; RFC-0004 §4; ADR-003; T3.1/T3.4 |
| 0008 | Runtime & Concurrency Execution Model | **Draft** | RFC-0004 (extends, per-node model unchanged); RFC-0001/0002/0005/0006/0007; ADR-012 §7.3; T4.1–T4.6 |
| 0009 | Resonator-Network Factorization | Accepted | RFC-0003 §6; RFC-0001; FR-C2/G2/G4; RR-5/VR-5 (M-350) |
| 0010 | Decode-Methodology Selection | **Draft** | RFC-0005 (same selection mechanism, third site); RFC-0009 (§10.3 matrix + regime gate); RFC-0003; G2/G4/VR-5 |
| 0011 | L0 `Match` & the L1-in-Core-IR Revision | **Accepted** (decision; enactment sequenced after RFC-0006/0007 ratification) | RFC-0001 §4.5/§4.6 (r3 revision); RFC-0006 §4.4 step 2; RFC-0007 §4.1–4.6; ADR-003; M-320 |

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
