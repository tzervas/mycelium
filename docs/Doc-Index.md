# Mycelium — Document Index & Status

**Date:** June 08, 2026
**Purpose:** map of the solidified document corpus. The two research passes are complete; the design corpus below is finalized (Accepted/Resolved) and ready for detailed design + hard planning.

---

## 1. Corpus & status (post-solidification)

| Doc | Role | Status |
|---|---|---|
| **Prior-Art Survey & Synthesis** | evidence base (pass 1) | Recorded → `research/01-prior-art-survey-RECORD.md` (full narrative is a conversation artifact) |
| **Research Findings (T0/T1/T2)** | evidence base (pass 2) | Recorded → `research/02-research-findings-RECORD.md` (full narrative is a conversation artifact) |
| **Project Foundation (r3)** | charter, scope, requirements, ADR-001…010, roadmap, risks | **Living — updated** |
| **RFC-0001 — Core IR & Metadata Schema** | value model, `Repr`, `Meta`, guarantee lattice, content-addressing | **Accepted** |
| **DN-01 — Packing Placement** | tradeoff study → schedule-staged decision | **Resolved → folded into RFC-0001 §4.1 + RFC-0004 §5** |
| **RFC-0002 — Swap Certificate & Split Regime** | certificate, legal pairs, bijection semantics, shared checker | **Accepted** |
| **RFC-0003 — VSA Submodule Boundary** | boundary, guarantee matrix, sparsity refinement, manifest | **Accepted** |
| **RFC-0004 — Execution Model & "Stable Component"** | MLIR backbone, shared checker, schedule-staged packing | **Accepted** |
| **RFC-0005 — Selection-Policy Language** | total cost-based policy + EXPLAIN | **Accepted** |
| **ADR-010 — Verified-Numerics Foundation** | two bound kernels + shared certificate | **Accepted** |

## 2. Dependency DAG

```
Survey ─► Foundation(r3) ─► RFC-0001 ─► { RFC-0002, RFC-0003, RFC-0004, RFC-0005 }
                               │
                               └─► DN-01 (Resolved) ─► RFC-0001 §4.1, RFC-0004 §5

ADR-010 ─► RFC-0001 §4.7,  RFC-0002 (swap bounds),  RFC-0003 (VSA bounds)

Shared machinery (decided):
  • ONE certificate checker   ⇄  RFC-0002 (swaps) & RFC-0004 (interp-vs-compiled)
  • ONE selection mechanism   ⇄  RFC-0002 (swap target) & RFC-0004 (packing schedule)
  • TWO bound kernels (ε, δ)  ⇄  ADR-010 → RFC-0001/0002/0003   (one shared certificate)
```

## 3. What the research resolved (decisions now baked in)
- **KC-1 (VSA-in-core viability): PASSED.** Proven non-asymptotic bundling bounds exist (Clarkson-Ubaru-Yang 2023; Thomas-Dasgupta-Rosing 2021) → VSA stays in core with honest `Proven` tags for MAP-I/sparse `bundle`. (T0.2)
- **Bounds: two kernels, one certificate** — ε via affine arithmetic (Daisy/FloVer), δ via union-bound/apRHL; `strength` composes by meet. (ADR-010 / T0.1)
- **Packing: schedule-staged**, not in the type; cost-model+exhaustive selector over the fixed bitnet.cpp set (I2_S/TL1/TL2). (DN-01 / T1.4)
- **Sparsity: declared class = static refinement; capacity = axiomatized-theorem + checked instantiation; observed sparsity = runtime metadata.** (T1.3)
- **Backend: MLIR backbone → LLVM**, Rust interpreter as reference + trusted base. (T1.5)
- **Validation: one translation-validation certificate checker** for swaps *and* interpreter-vs-compiled equivalence; graded (differential testing + per-artifact TV for stable components). (T1.1)
- **Bijection: `LosslessWithinRange`** (total bijection impossible at fixed widths), `Option`-typed inverse, Exact-within-range, never silent. (T2.1)
- **Selection: total, non-learned cost policy + mandatory EXPLAIN**; avoids the DB cardinality-estimate trap because Mycelium's statistics are exact metadata. (T2.3)
- **VSA guarantee matrix** per model × operation encoded into RFC-0003; **HRR/FHRR unbind is the residual `Empirical` weak link.** (T1.2)

## 4. Remaining experiments (small; for the build phase, not blockers to design)
- **LH bundling-bound instantiation** — encode MAP-I `bundle` capacity refinement in Liquid Haskell; confirm Z3 discharges it. The one confirming build for the cited-theorem + checked-instantiation strategy (ratifies ADR-010 / KC-1). *(RFC-0003 §5)*
- **E1** staged-packing perf over the 5 schemes *(RFC-0004 §8)* · **E3** wrong-layout-tag soundness vs NFR-7 *(RFC-0004 §8)* · **E4** LLM surface comparison, "packing in type" vs absent *(G10)*.

## 5. Next phase — detailed design & hard planning
With the corpus Accepted, the work shifts to: (a) the confirming LH probe; (b) the Core IR concrete grammar + the Rust interpreter (reference semantics) + the kernel data structures; (c) the `ternary` MLIR dialect and the certificate checker; (d) the VSA submodule (per the §4-matrix tags), reusing `balanced-ternary`. Track as dependency-ordered, priority-tagged tasks; this index remains the map the board points back to.

## 6. Maintenance
Append-only with status transitions (Draft/Proposed/Preliminary → Accepted → Superseded), mirroring the ADR discipline. Keep `Proven | Empirical | Declared` tags honest per VR-5 — per model/op, never in aggregate. New non-asymptotic results may *upgrade* a tag; absence keeps it `Empirical`.
