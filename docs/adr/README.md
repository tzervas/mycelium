# Architecture Decision Records (ADRs)

An ADR captures a single architectural decision, its context, and its consequences. ADRs are **append-only**: to change one, supersede it with a new ADR and link forward.

## Where ADRs live

- **ADR-001 … ADR-009** are recorded in the project charter, **`../Mycelium_Project_Foundation.md` §8** (they were authored together as the founding decision log).
- **ADR-010 and onward** are **standalone files in this directory**, one per file (`ADR-NNN-short-title.md`).

New ADRs go here as standalone files. (ADR-001…009 may be extracted into individual files later for uniformity; until then, the Foundation §8 is authoritative for them.)

## Status set

`Proposed → Accepted → Superseded`. All current ADRs are **Accepted** except ADR-012
(**Proposed** — its runtime model is now drafted as RFC-0008; ratification pending).

## Index

| ADR | Title | Status | Location |
|---|---|---|---|
| 001 | Tension-B framing: exact-spec + proven/declared bound (no *hidden* approximations) | Accepted | Foundation §8 |
| 002 | Split verification regime (bijective binary↔ternary; bounded/probabilistic VSA) | Accepted | Foundation §8 |
| 003 | Architecture anchors = MLIR ⊕ Unison ⊕ Arrow | Accepted | Foundation §8 |
| 004 | Embeddenator = provisional inspiration, not reference | Accepted | Foundation §8 |
| 005 | Ternary = logical substrate now, native hardware later | Accepted | Foundation §8 |
| 006 | Representation-selection policies are reified, inspectable artifacts | Accepted | Foundation §8 |
| 007 | Kernel in Rust (reference interpreter); MLIR→LLVM for AOT; tooling in Python | Accepted | Foundation §8 |
| 008 | VSA is in core semantics but packaged as an optional submodule | Accepted | Foundation §8 |
| 009 | Hybrid execution; AOT preferred for stable components; interpreter is reference | Accepted | Foundation §8 |
| 010 | Verified-numerics foundation: two bound kernels (ε / δ) + shared certificate | Accepted | `ADR-010-Verified-Numerics-Foundation.md` |
| 011 | `BoundBasis` is a property of every `Bound` (not just `CapacityBound`) | Accepted | `ADR-011-BoundBasis-Is-Universal.md` |
| 012 | Layered lexicon (Surface/Runtime/Formal tiers) + fungal runtime vocabulary | Proposed | `ADR-012-Layered-Lexicon-and-Fungal-Runtime-Model.md` |
| 013 | `spore` is the deployable unit; the reconstruction manifest is one component | Accepted | `ADR-013-Spore-Is-The-Deployable-Unit.md` |

## Template

```markdown
# ADR-NNN — <short title>

| Field | Value |
|---|---|
| **ADR** | NNN |
| **Status** | Proposed \| Accepted \| Superseded |
| **Date** | YYYY-MM-DD |
| **Supersedes / Superseded by** | (links, if any) |

## Context
<the forces at play; cite grounding labels: G*, A–E, R*, T0/T1/T2>

## Decision
<the decision, stated plainly>

## Consequences
<positive and negative; what this enables and what it costs>

## Grounding
<survey/research references>
```
