# Architecture Decision Records (ADRs)

An ADR captures a single architectural decision, its context, and its consequences. ADRs are **append-only**: to change one, supersede it with a new ADR and link forward.

## Where ADRs live

- **ADR-001 … ADR-009** are recorded in the project charter, **`../Mycelium_Project_Foundation.md` §8** (they were authored together as the founding decision log).
- **ADR-010 and onward** are **standalone files in this directory**, one per file (`ADR-NNN-short-title.md`).

New ADRs go here as standalone files. (ADR-001…009 may be extracted into individual files later for uniformity; until then, the Foundation §8 is authoritative for them.)

## Status set

`Proposed → Accepted → Enacted → Superseded`. ADRs 001–014/016/017 are **Accepted**; **ADR-015/ADR-018/ADR-019** are **Enacted** (2026-06-23, maintainer-approved); **ADR-020** is **Enacted** (M-521 v0 R1 implementation landed 2026-06-20). **ADR-021** (kernel 1.0.0 gate) is **Superseded by ADR-022** (2026-06-23; kernel gate preserved as track T1). **ADR-022** (full-language 1.0.0 gate, dual-version) is **Accepted** (2026-06-23, supersedes ADR-021). **ADR-023** (stability and API-compatibility guarantees) is **Draft** (2026-06-23, planning stub).

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
| 012 | Layered lexicon (Surface/Runtime/Formal tiers) + fungal runtime vocabulary | Accepted | `ADR-012-Layered-Lexicon-and-Fungal-Runtime-Model.md` |
| 013 | `spore` is the deployable unit; the reconstruction manifest is one component | Accepted | `ADR-013-Spore-Is-The-Deployable-Unit.md` |
| 014 | `unsafe` Rust: permitted-but-warned (explicit, justified, dev-warned, release-silenceable), not forbidden | Accepted | `ADR-014-Unsafe-Code-Policy.md` |
| 015 | `DEFAULT_ENUM_BUDGET = 4096` (guarantee-maximal): the RFC-0010 decode selector defaults to the `Exact` arm across the whole validated envelope, not the cost-optimal ≈128 | **Enacted** (2026-06-23, M-350; applied in `decode_select.rs`) | `ADR-015-decode-enum-budget-default.md` |
| 016 | The interpreted↔compiled ABI: dispatch a compiled definition by its content hash; cross values in the self-describing wire form (RFC-0004 §10 OQ-1) | Accepted | `ADR-016-Interpreted-Compiled-ABI.md` |
| 017 | Hot-inject recompiled definitions: hash-keyed dispatch + content-addressed dynamic linking, immutable-by-construction (RFC-0004 §10 OQ-2) | Accepted | `ADR-017-Hot-Inject-Recompiled-Definitions.md` |
| 018 | Versioning policy: per-crate `0.x` SemVer + source-only distribution (no crates.io publish in the design phase); CHANGELOG `[Unreleased]` → release-cut mapping | **Enacted** (2026-06-23, M-383/M-384; workspace pinned `0.0.0`; release-plz dry-run wired; first release cut deferred) | `ADR-018-Versioning-Policy.md` |
| 019 | libMLIR toolchain: the version-matched build dependency of the off-by-default `mlir-dialect` feature; provisioned via `scripts/setup-mlir.sh` (`just setup-mlir`); resolves M-348 on Linux | **Enacted** (2026-06-23, M-348/M-603; `setup-mlir.sh` present; `mlir-dialect` OFF by default; real dialect lowering present M-601) | `ADR-019-libMLIR-Toolchain.md` |
| 020 | `runtime`/`colony` phylum placement: dedicated `runtime` phylum + thin `std.runtime` facade; construct-by-construct activation at the Phase-7 gate; v0 API surface for the landed R1 slice (M-521) | **Enacted** | `ADR-020-Runtime-Colony-Phylum-Placement.md` |
| 021 | 1.0.0 release-readiness gate: kernel/core 1.0.0 criteria (Gate A honesty-integrity/durability; Gate B decision/external — KC-2 now met); surface language scoped to `1.x`; Phase-3+ maturation out of scope | **Superseded by ADR-022** (2026-06-23; kernel gate preserved as track T1 within the full-language program) | `ADR-021-1.0.0-Release-Readiness-Gate.md` |
| 022 | Full-Language 1.0.0 Release-Readiness Gate: dual-version core⟂lang — kernel track T1 (preserves ADR-021 Gate A/B) + full-language program tracks T2–T9; 9 epics E10-1…E18-1; DN-25 program map | **Accepted** (2026-06-23; supersedes ADR-021) | `ADR-022-Full-Language-1.0.0-Release-Gate.md` |
| 023 | Stability and API-Compatibility Guarantees: four-layer stability scope (surface · Core-IR · LSP wire · Rust crate API) with explicit carve-outs; dual-version model (full-language 1.0.0 = a release-event, not a crate version); release-based never-silent deprecation; MIT-only legal-readiness gate | **Accepted** (2026-06-23; M-737 — was Proposed/Draft 2026-06-23; →Enacted at the full-language 1.0.0 tag) | `ADR-023-Stability-and-API-Compatibility-Guarantees.md` |

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
