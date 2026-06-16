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
| **RFC-0001 — Core IR & Metadata Schema** | value model, `Repr`, `Meta`, guarantee lattice, content-addressing; **r3** adds data (registry Σ, `Datum`/`CoreValue`, `Construct`/flat `Match`, WF6/WF7/WF8); **r4** adds `Lam`/`App`/`Fix` (full L1-in-Core-IR; closed-closure value model; R7-Q1/Q3) | **Accepted (r4)** |
| **DN-01 — Packing Placement** | tradeoff study → schedule-staged decision | **Resolved → folded into RFC-0001 §4.1 + RFC-0004 §5** |
| **DN-02 — Fungal Lexicon & Reserved Words** | the naming law + ratified surface vocabulary (themed vs conventional) feeding RFC-0006's grammar | **Resolved** (ratified 2026-06-10) |
| **DN-03 — Lexicon Amendment: Surface Additions & Runtime Names (one name each)** | amends DN-02: adopt `consume`/`grow`, decline `embody` (keep `impl`), reserve `for`; **one name per term** (flat — rejects ADR-012 §7.6's canonical+alias scheme); the single Runtime names (`hypha`/`fuse`/`xloc`/`cyst`/`graft`/`forage`/`backbone`/`mesh`/`tier`/`reclaim`) | **Resolved** (2026-06-10) |
| **DN-04 — Optional Structured Diagnostics (DynEL-inspired)** | evaluate DynEL's feature set as **opt-in** structured/graded/dual-format diagnostics over Mycelium's existing explicit errors; governing constraint = diagnostics are *additive*, never a substitute for a never-silent error (G2); imports the contracts (levels / human+JSON / reified per-fn `{exceptions, message, tags}`), excludes the anti-patterns (`eval`-on-config, env-dump, exception-swallowing); tooling-layer, kernel untouched (KC-3) (M-345) | **Draft** (2026-06-16; source read; direction open) |
| **DN-05 — AOT Recursion Execution Strategy** | make the AOT env-machine recursion stack-robust without bloat; **empirically grounded** (`xtask recursion-probe`: env-machine aborts at ~600 `Fix`-unfolds vs the interpreter's O(1) stack); priority = (1) bank native MLIR→LLVM stack-robustness as a design requirement [libMLIR-gated, M-348], (2) explicit control stack / trampoline [near-term, makes never-silent total], (3) tail-call detection [cautious, optional, KC-3/YAGNI] (M-347) | **Draft** (2026-06-16; recommendation presented; no code) |
| **DN-06 — Static Organization & Dynamic Grouping Lexicon** | introduces `phylum` (library-scale) / `nodule` (basic unit, replaces "module") and reassigns `colony` to the **dynamic** runtime grouping of `hypha` (RFC-0008 §4.7); supersedes DN-02 §2's `colony` line (append-only); nodule status declared by header comment, not filename; keyword migration = M-358 | **Resolved** (2026-06-16) |
| **Glossary & Term Index** (`docs/Glossary.md`) | the dedicated, separately-maintained reference for unique Mycelium terms — a summarized Index over a detailed Glossary (fungal lexicon + honesty/architecture concepts), each entry citing its normative source | **Living** (2026-06-16) |
| **Nodule Header & Phylum Manifest** (`docs/spec/Nodule-Header-and-Phylum-Manifest.md`) | the structured in-file nodule header (closed-key `// @key: value`), the `phylum.toml` manifest (pyproject analogue), and explicit top-down inheritance; metadata ≠ identity (ADR-003), no ambient metadata (G2) | **Proposed** (2026-06-16; format choices flagged for sign-off) |
| **Lexicon Reference (v0.4)** | the three-tier vocabulary catalog (Surface/Runtime/Formal) with short mnemonic forms; extends DN-02 | **Draft** (review notes in ADR-012 §7) |
| **Example Programs Reference (v0.2)** | 20 worked examples demonstrating the lexicon and surface syntax against RFC-0001 semantics | **Draft** (grounding notes appended) |
| **Native-Ternary Forward-Compat (M-370)** | forward map: the ternary value-semantics contract → a future 3-state HW backend; the `ternary` dialect as the stub target (ADR-005 / R7) | **Living note** (2026-06-15) — doc + stub, no backend built |
| **RFC-0002 — Swap Certificate & Split Regime** | certificate, legal pairs, bijection semantics, shared checker | **Accepted** |
| **RFC-0003 — VSA Submodule Boundary** | boundary, guarantee matrix, sparsity refinement, manifest | **Accepted (r4)** — r3 §4.1 erratum; r4 §6 resonator decode params (RFC-0009) |
| **RFC-0004 — Execution Model & "Stable Component"** | MLIR backbone, shared checker, schedule-staged packing; **r2** adds §9 interpreted↔compiled continuum + build-target profiles (`interpret`/`--slim`/`--target`/`--fat`) + §10 open questions | **Accepted (r2)** |
| **RFC-0005 — Selection-Policy Language** | total cost-based policy + EXPLAIN | **Accepted** |
| **RFC-0006 — Surface Language & Term-Language Layering** | L0–L3 layering, syntactic honesty invariants, Rust-class-and-beyond capability targets; concrete syntax KC-2-gated | **Accepted** (r4, 2026-06-15; layering/S1–S6/LR-1…LR-9/grammar ratified; concrete L3 syntax KC-2-gated, stage-1 grading deferred — §10) |
| **RFC-0007 — The L1 Kernel Calculus** | ten-node term budget, registry data declarations, totality/`matured` gate, v0 elaboration fragment | **Accepted** (r4, 2026-06-15; v0 calculus §4.1–4.8 ratified; stage-1 grading / R7-Q1…Q4 / surface syntax deferred — §10) |
| **ADR-010 — Verified-Numerics Foundation** | two bound kernels + shared certificate | **Accepted** |
| **ADR-011 — BoundBasis is universal** | `basis` is a companion of every `Bound` (supersedes RFC-0001 r1 §4.3) | **Accepted** |
| **ADR-012 — Layered Lexicon & Fungal Runtime Model** | three-tier lexicon + an (aspirational) distributed fungal runtime vocabulary | **Proposed** (architect review notes appended §7; the runtime model is now drafted as RFC-0008, grounded by research Pass 4) |
| **Research Record 04 (T4.1–T4.6)** | evidence base (pass 4): concurrency, merge/mesh, mobility/placement, durability, failure, mode switching | Recorded → `research/04-runtime-concurrency-RECORD.md` |
| **RFC-0008 — Runtime & Concurrency Execution Model** | RT1–RT7 runtime invariants; deterministic-fragment-first; placement as the third policy site; lawful fusion; honest probabilistic guarantees; Runtime vocabulary grounded but reserved | **Draft** |
| **RFC-0009 — Resonator-Network Factorization** | opt-in VSA factorization of bind products into unknown codebook factors; iterative resonator loop; probabilistic-only (`Empirical` ceiling, never `Proven`); never-silent termination verdicts; reuses `VsaModel`/cleanup/`EmpiricalProfile` (M-350) | **Accepted** (ratified 2026-06-15; prototype building) |
| **RFC-0010 — Decode-Methodology Selection** | choose the decode methodology (brute-force `Exact` vs resonator `Empirical` vs refuse) as a third site of the one RFC-0005 selection mechanism; reified, `EXPLAIN`-mandatory decision table over exact metadata; guarantee tag read off the chosen arm; cleanup-variant selection deferred (Hebbian dominates) (M-350) | **Accepted** (ratified 2026-06-15; prototype building) |
| **RFC-0011 — L0 `Match` & the L1-in-Core-IR Revision** | the named RFC-0001 revision (RFC-0006 §4.4 step 2): fold the L1 data-and-matching core (`Construct` + flat `Match` + content-addressed registry; WF6/WF7/WF8) into frozen L0 as **r3**, staged ahead of an r4 (`Lam/App/Fix`); flat `Match` is the kernel node (Maranget tree stays untrusted); unblocks M-320 leaf-emission + M-310 sync (M-320) | **Accepted — r3 ENACTED** (2026-06-15; RFC-0001 → r3, code in `mycelium-core`/`-interp`/`-l1`, M-210 differential covers the data fragment; `Lam/App/Fix` → r4) |
| **RFC-0012 — Ambient Representation & Scoped Overrides** | a **surface-only** declared, scoped, **paradigm-only** representation default + override/conversion blocks, to offset honesty's verbosity (tension A) without a black box: two normative invariants — the ambient emits no `Swap` (I1) and resolution is observationally the identity (I2, identical L0 ⟹ identical hash) — defended by a meaning-preservation differential (NFR-7); cross-paradigm edges stay explicit `Swap`s, a missing one is an explicit `MissingConversion`; kernel untouched (KC-3); exported signatures are concrete reprs (ADR-016) (M-344) | **Accepted** (ratified 2026-06-16; design normative, enactment gated — M-344) |
| **ADR-013 — `spore` Is the Deployable Unit** | spore = content-addressed code+values+metadata; the RFC-0003 manifest is one component; `spore(v)` is the degenerate case | **Accepted** |
| **ADR-015 — Decode `DEFAULT_ENUM_BUDGET` = 4096** | fixes the RFC-0010 decode-selector default at the *guarantee-maximal* enumeration budget (4096 = `MAPI_RESONATOR_PROFILE.max_capacity`) — every in-regime request is also enumerable, so the brute-force `Exact` arm dominates the whole validated envelope — over the cost-optimal ≈128; resolves RFC-0010 §8 | **Accepted** |
| **ADR-016 — Interpreted↔compiled ABI** | dispatch a compiled stable component by its content hash; cross `CoreValue`s in the RFC-0001 §4.8 self-describing wire form (canonical), zero-copy fast-path later; resolves RFC-0004 §10 OQ-1 | **Accepted** (ratified 2026-06-16) |
| **ADR-017 — Hot-inject recompiled definitions** | hash-keyed dispatch + content-addressed dynamic linking; immutable-by-construction (a change is a new hash under a new entry) dissolves the atomicity hazard; recompile = the changed dependency-closure by hash reachability; resolves RFC-0004 §10 OQ-2; in-process prototype in `mycelium-mlir::inject` (M-341) | **Accepted** (ratified 2026-06-16; prototype building) |

## 2. Dependency DAG

```
Survey ─► Foundation(r3) ─► RFC-0001 (r4) ─► { RFC-0002, RFC-0003, RFC-0004 (r2), RFC-0005, RFC-0006 (Accepted r4) ─► RFC-0007 (Accepted r4) ─► RFC-0011 (Accepted; r3 ENACTED → r4 in RFC-0001) }
                               │
                               └─► DN-01 (Resolved) ─► RFC-0001 §4.1, RFC-0004 §5

ADR-010 ─► RFC-0001 §4.7,  RFC-0002 (swap bounds),  RFC-0003 (VSA bounds)

Shared machinery (decided):
  • ONE certificate checker   ⇄  RFC-0002 (swaps) & RFC-0004 (interp-vs-compiled)
  • ONE selection mechanism   ⇄  RFC-0002 (swap target), RFC-0004 (packing schedule) & RFC-0010 (decode method)
  • TWO bound kernels (ε, δ)  ⇄  ADR-010 → RFC-0001/0002/0003   (one shared certificate)
```

## 3. What the research resolved (decisions now baked in)
- **KC-1 (VSA-in-core viability): PASSED → confirmed (build) 2026-06-09.** Proven non-asymptotic bundling bounds exist (Clarkson-Ubaru-Yang 2023; Thomas-Dasgupta-Rosing 2021) → VSA stays in core with honest `Proven` tags for MAP-I/sparse `bundle`. (T0.2) The confirming Liquid-Haskell `bundle` probe (`proofs/lh-bundle/`, M-001) is **SAFE** (Z3 discharged), ratifying the cited-theorem + checked-instantiation strategy.
- **Bounds: two kernels, one certificate** — ε via affine arithmetic (Daisy/FloVer), δ via union-bound/apRHL; `strength` composes by meet. (ADR-010 / T0.1)
- **Packing: schedule-staged**, not in the type; cost-model+exhaustive selector over the fixed bitnet.cpp set (I2_S/TL1/TL2). (DN-01 / T1.4)
- **Sparsity: declared class = static refinement; capacity = axiomatized-theorem + checked instantiation; observed sparsity = runtime metadata.** (T1.3)
- **Backend: MLIR backbone → LLVM**, Rust interpreter as reference + trusted base. (T1.5)
- **Validation: one translation-validation certificate checker** for swaps *and* interpreter-vs-compiled equivalence; graded (differential testing + per-artifact TV for stable components). (T1.1)
- **Bijection: `LosslessWithinRange`** (total bijection impossible at fixed widths), `Option`-typed inverse, Exact-within-range, never silent. (T2.1)
- **Selection: total, non-learned cost policy + mandatory EXPLAIN**; avoids the DB cardinality-estimate trap because Mycelium's statistics are exact metadata. (T2.3)
- **VSA guarantee matrix** per model × operation encoded into RFC-0003; **HRR/FHRR unbind is the residual `Empirical` weak link.** (T1.2)

## 4. Remaining experiments (small; for the build phase, not blockers to design)
- ~~**LH bundling-bound instantiation**~~ — **DONE (2026-06-09).** MAP-I `bundle` capacity refinement encoded in Liquid Haskell (`proofs/lh-bundle/`, M-001); LH reports **SAFE**, Z3 discharged all constraints — ratifies the cited-theorem + checked-instantiation strategy (ADR-010 / KC-1). *(RFC-0003 §5)*
- **E1** staged-packing perf over the 5 schemes *(RFC-0004 §8)* · **E3** wrong-layout-tag soundness vs NFR-7 *(RFC-0004 §8)* · **E4** LLM surface comparison, "packing in type" vs absent *(G10)*.

## 5. Next phase — detailed design & hard planning
With the corpus Accepted, the work shifts to: (a) the confirming LH probe; (b) the Core IR concrete grammar + the Rust interpreter (reference semantics) + the kernel data structures; (c) the `ternary` MLIR dialect and the certificate checker; (d) the VSA submodule (per the §4-matrix tags), reusing `balanced-ternary`. Track as dependency-ordered, priority-tagged tasks; this index remains the map the board points back to.

## 6. Maintenance
Append-only with status transitions (Draft/Proposed/Preliminary → Accepted → Superseded), mirroring the ADR discipline. Keep `Proven | Empirical | Declared` tags honest per VR-5 — per model/op, never in aggregate. New non-asymptotic results may *upgrade* a tag; absence keeps it `Empirical`.
