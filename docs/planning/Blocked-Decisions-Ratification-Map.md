# Blocked / Deferred Decisions — Ratification Map

| Field | Value |
|---|---|
| **Status** | **Living snapshot** (2026-06-27) — a corpus-wide audit of deferred/blocked design decisions, grouped for batch ratification. Regenerate when the corpus moves. |
| **Method** | `Empirical` — produced by a parallel sweep (6 finders over RFCs/ADRs/DNs + a status cross-cut → Opus synthesis). 173 raw findings deduped into 14 groups + 14 ungrouped. |
| **Purpose** | Let the maintainer **nail down deferred decisions in logical batches** rather than one-DN-at-a-time, and prevent waves from launching against an unratified surface (the R4/R3 lesson). |

> **Posture (VR-5 / G2).** This is a *finder-sourced* map: the line citations come from the sweep's
> raw findings and are **not independently re-verified** beyond confirming the two anchors (DN-58
> exists/Draft; RFC-0024 Proposed with the Reynolds closure case deferred). Treat group membership and
> citations as `Empirical` leads to confirm at ratification time, not `Exact` truth. Several members
> (esp. the LOW grading/diagnostics group) are **already resolved** — those are *confirm-the-record*,
> not re-decide; flagged per group. No status is moved by this doc (house rule #3).

---

## How to use this

Each group shares a **design axis** so it can be ratified as a unit. For each: the **vehicle** (the doc
that would close it), what **unblocks** on ratification, and a **priority** by downstream leverage. Work
HIGH-priority groups first — they gate the core-1.0.0 critical path. Group 1 (runtime-vocab) is already
in flight as **DN-58 / PR #685**.

---

## HIGH priority — gate the core-1.0.0 critical path

### G1 · Runtime-vocabulary surface forms (`fuse` / `reclaim` / `tier` / reserved keywords)
- **Axis:** concrete L1 surface spelling + activation of the reserved-not-active runtime vocabulary, *on top of already-ratified semantics* (RT6 semilattice, RT7 supervision, tier mode). "Name + surface, mechanism settled."
- **Vehicle:** **DN-58 (Draft, in flight — PR #685)** — promote to Accepted, resolving F-A* (fuse trait/op), **F-B1/F-B2 (reclaim form + policy type — the deferred surface)**, F-C* (tier attribute/words). Then per-construct implementation RFCs for the rest of the vocabulary (hypha/colony/mesh/cyst/forage), gated on DN-58 + DN-03.
- **Unblocks:** M-667, M-710, the `r4v` wave, ADR-020 reclaim activation, Phase-7 std-vs-runtime-phylum sequencing, and the `lambda` keyword-reservation tail.
- **Sources:** DN-58 §A/§B/§C · RFC-0008 §4.5 · RFC-0027 §10.5 (×2) · ADR-020 · RFC-0037 §7 FLAG-4 · RFC-0016 §8 Q4 · RFC-0031 §5.

### G2 · Higher-order / closure semantics (Reynolds generalization)
- **Axis:** the dynamic-function-value frontier — environment-capturing closures, partial application, multi-arg arrows, and their AOT native lowering. One missing mechanism: generalize RFC-0024's named-function defunctionalization to the full Reynolds construction (fn-tag sum + `apply` dispatch).
- **Vehicle:** **promote RFC-0024 (Proposed → Accepted) with the full Reynolds algorithm specified** (only the named-fn case is Rust-first implemented today). AOT-native closure/recursion increments (DN-15 Inc-2/3, RFC-0004 §11.5) attach as the codegen arm once the lambda surface (M-704) lands. Multi-arg arrows need tuple-type support (prerequisite).
- **Unblocks:** M-704 (full HOF / the `hof` wave), E7-3 closure surface, M-349 Increment-2 native codegen, operator-as-value (DN-23) ergonomics.
- **Sources:** RFC-0024 §5/§6 (×2), §3 · RFC-0004 §11.3/§11.5 (×2) · DN-15 §4.2.

### G3 · Memory / reclamation strategy + RC (post-DN-32 follow-on)
- **Axis:** the reclamation/ownership questions *not* in G1 — cross-hypha sharing protocol (sole-move vs atomic-RC), Layer-2 static uniqueness analysis, drop-latency SLO/fuel bound, `rc==1` reuse visibility, sweep-coupling, env-machine reclamation wiring. All share the three-layer hybrid memory model (RFC-0027 + DN-32/33/35).
- **Vehicle:** a single **RFC-0027 follow-on** (or DN) closing OQ-1..OQ-6, plus a maintainer decision on **DN-33 §8 Q2 (ownership-mode representation)** — the hard prerequisite for MEM-4. Several OQs are already resolved (OQ-1 weak-coupling, OQ-4 EXPLAIN-only) and only need a ratification-record.
- **Unblocks:** MEM-4 / E12 memory build, Layer-2 RC optimization, M-373 Increment-3 native stack-robustness (Fix/FixGroup), E12 Inc-3 env-machine reclamation, cross-hypha API.
- **Sources:** DN-32 §6/7/9 · DN-33 §5/6.1 · RFC-0027 §11 OQ-1..6 + §12 · RFC-0004 §11.3 · DN-35 · RFC-0006 §8 Q8.

### G4 · Polymorphism / dispatch / traits surface completion
- **Axis:** generic instantiation, coherence, multi-param traits, associated types, repr-polymorphism, grade-in-dictionary, `derive`/`grow` elaboration, width-generics, and `FieldSpec::Fn` dynamic-dispatch soundness — the trait/parametric machinery over the L1 kernel. Mostly RFC-0019/0020 carve-outs awaiting one trait-format ratification + the ADR-033 kernel dispatch primitive.
- **Vehicle:** mostly **ratification + amendment** — full **RFC-0019 enactment** (single-param coherence, repr-poly restriction set) discharges the RFC-0020 carve-outs; multi-param traits + associated types → an RFC-0019 v2; width-generics → RFC-0032 extension (DN-41/42); **`FieldSpec::Fn` soundness → ADR-033 FLAG-1** (needs a machine-checked basis before Enacted/kernel-freeze).
- **Unblocks:** E7-1/E7-2 traits + `derive` surface, monomorphization (M-673), RFC-0030 L3 carve-outs, M-753 width-generics, M-798 width-cast, ADR-033 Enacted gate, **kernel-freeze condition #3** (via FLAG-1).
- **Sources:** RFC-0020 §4.2/4.5/4.6 · RFC-0019 §8/9 · RFC-0030 · DN-41/42 · ADR-033 §6 FLAG-1..4 · RFC-0007 §4.4 · DN-56 §4/7.

### G5 · Kernel-freeze criterion + primitive-set closure
- **Axis:** the kernel-stabilization gate — five-condition freeze, parsable-vs-runnable census, reject-ledger completeness, primitive-set closure (incl. the VSA/HDC kernel question + ADR-033 `FieldSpec::Fn`), lowering-surface closure. "What must hold before core-1.0.0 kernel freeze."
- **Vehicle:** **DN-56 (Accepted) as the tracking framework** — satisfy the five conditions. The two open primitive-set items needing their own ratification: **RFC-0036 (Draft, VSA/HDC-kernel question)** and **ADR-033 FLAG-1** (shared with G4). The census (DN-50/M-807) and reject-ledger are work items, not doc decisions.
- **Unblocks:** core-1.0.0 kernel freeze (ADR-021/022 Gate-A), M-807 census, M-719 conformance, E19-1 core arithmetic, M-703 core tag.
- **Sources:** DN-56 §5 · DN-50 · RFC-0036 §3 · ADR-024 changelog · DN-22 §5.
- **⚠ Cross-dep:** condition #3 depends on **ADR-033 FLAG-1 (in G4)** — ratify FLAG-1 either here or in G4, not twice.

---

## MED priority — unblock major non-gate work

### G6 · Effect-system surface (budgets, rows, hypha-in-effect-row)
- **Axis:** enrich effect typing past the v0 declared-set — dynamically-resolved budgets, row-polymorphism / minimal-set inference, whether hypha-creation appears in the T3.4 effect row.
- **Vehicle:** a **Phase-2 effect-system RFC** (extends RFC-0014) deciding row-polymorphism + budget-inference together; the hypha-in-row question (RFC-0008 R8-Q2) decided in concert with RFC-0018 grading.
- **Unblocks:** effect-polymorphic stdlib APIs; concurrency/effect integration. *Not a 1.0.0 gate (KISS/YAGNI-deferred).*
- **Sources:** RFC-0014 §4.8, §8 · RFC-0008 §8 R8-Q2.

### G7 · AOT / native codegen + artifact-ABI maturity (→ 1.1)
- **Axis:** the MLIR→LLVM perf path + artifact/ABI plumbing — VSA/embedding dialects, hot-inject recompilation, interpreted↔compiled ABI, fat-artifact packaging, native-spore deploy, JIT/REPL.
- **Vehicle:** ratify the existing Proposed ABI/packaging ADRs as a unit (**ADR-016 ABI, ADR-017 hot-inject, RFC-0004 OQ-3 fat-packaging**) + a native-spore ADR (DN-18, amends ADR-013). The "1.1 native-maturity" bundle.
- **Unblocks:** T6 native AOT (E15-1), M-349 LLVM increments, M-620 native-spore. **Explicitly rolled to 1.1 by ADR-022 — post-1.0.0.**
- **Sources:** RFC-0004 §11/§10 OQ-1/2/3 · ADR-016/017 · ADR-013 §4 · DN-18 · RFC-0029 · ADR-022 §8.

### G8 · Concurrency / distribution runtime semantics (R8 + R2)
- **Axis:** runtime-model *semantics* (distinct from G1's surface) — scheduler normativity, time/clocks/deadlines, Byzantine tolerance, distributed reclamation, FFI sandboxing/network-xloc, concurrent session-merge.
- **Vehicle:** (a) the **R1 implementation RFC** fixes scheduler-normativity within RFC-0008's frame; (b) a dedicated **R2 distributed-execution RFC** (own research pass) closes Byzantine, distributed reclamation, network-FFI/xloc, clocks, concurrent fuse-merge.
- **Unblocks:** R1 scheduler RFC, RT3 nondeterministic constructs, R2 distributed RFC, concurrent-session fuse-merge, mesh ordering.
- **Sources:** RFC-0008 §8 R8-Q1/Q3/Q4 · RFC-0028 §7 · RFC-0023 §4.3 · RFC-0022 §4.3/10.2.

### G9 · Surface grammar deconfliction + operators + object/inheritance model
- **Axis:** the L3 concrete-grammar axis — bracket deconfliction, ordering/user-defined operators, short repr-keywords, delimiter/adjacency, or-patterns, `phylum` keyword, the object/behavior (inheritance-emulation) menu — all gated on the RFC-0037/DN-31 grammar wave.
- **Vehicle:** a binding **RFC-0025 + RFC-0030 update** (DN-31 grammar epic #27) closes bracket/operator deconfliction (auto-resolves M-745); the object-model menu (DN-37) becomes per-item follow-on surface epics. *Much of this already landed with RFC-0037 — confirm residue.*
- **Unblocks:** M-745 ordering/shift operators, RFC-0025/0030 ratification, object/behavior surface wave, VSA literal ergonomics.
- **Sources:** DN-31 · RFC-0025 §4 · RFC-0030 §5 · RFC-0037 §7 FLAG-2/3 · RFC-0020 §3 · DN-37 · RFC-0033.

### G10 · Stdlib ergonomics / self-hosting gates
- **Axis:** std shape + self-hostability — per-ring ergonomics pass, BF16 placement, error-value naming, wild/FFI floor split, migration differential bar, the five self-hosting feature-gates, stable-API retirement, transpiler phase.
- **Vehicle:** mostly **ratification + scheduled passes** — the per-ring ergonomics pass (M-540) carries ergonomics/BF16/naming (DN-07 accepts the direction); self-hosting gates discharge as RFC-0018/0019/0014/0028 enact; transpiler/full-retirement → a post-core-1.0.0 ADR.
- **Unblocks:** M-540, M-502 self-hosting verdict, E7-1 stdlib generics, M-717 text/fmt, M-799 bytes slice/concat, T9 self-hosting.
- **Sources:** RFC-0016 §8 · DN-07 §3 · RFC-0031 §5 · ADR-022 §8 · DN-26/34/43.
- **⚠ Cross-dep:** the five-gate-fails (DN-14) is fed by **G4 (polymorphism), G6 (effects), G11 (FFI)** — sequence after those.

### G11 · Security / FFI-hardening + tooling follow-ons
- **Axis:** the security/FFI boundary — host-encoding validation bridge, A1/A2/A3 input-validation gaps, insecurity-disclosure mechanism, OSV scanning, unsafe-SAFETY lint, forbid-unsafe in kernel crates, security-toolkit examples + fix-strength bar.
- **Vehicle:** one **RFC-0028 §4.4 host-encoding spec** (parse-into-typed / injective-encode / bounded) **before E14-1**; the A1/A2/A3 gaps + getrandom + unsafe-lint + forbid-unsafe are forward implementation issues; OSV/disclosure + toolkit examples → DN-45/46/RFC-0035 follow-ons.
- **Unblocks:** E14-1/M-722 FFI host-encoding bridge, security hardening increments, E22-1 toolkit, OSV scanning, getrandom.
- **Sources:** DN-40 · DN-44 §1.1/6 · DN-46 · DN-45 · ADR-014 · RFC-0028 §7 · RFC-0035 §9/10.
- **⚠ Must-fix-before-E14-1:** the DN-40 input-validation bridge + A1/A2/A3 are CRITICAL/HIGH.

---

## LOW priority — ratification-record cleanup or YAGNI re-open triggers

### G12 · Static guarantee grading + diagnostics integration (mostly DISCHARGED)
- **Axis:** stage-1 grading + error/diagnostic policy. **Nearly all already resolved** by Accepted RFCs (RFC-0018 implicit-flows + grade-inference + dynamic-index; RFC-0013 recovery-scope + tags + diagnostic codes). This is a **confirm-the-append-only-record** task, not open design. One genuinely-open tail: recovery-handler control-flow (RFC-0014 follow-on).
- **Sources:** RFC-0006 §8 Q3 · RFC-0007 §4.6 · RFC-0020 §4.2/4.6 · RFC-0013 §4.4/8 · DN-04 §5 · DN-13.

### G13 · VSA / resonator / value-model precision follow-ons
- **Axis:** VSA-specific deferrals — codebook learning, cleanup-variant selection, decode identifiability-precheck, enum-budget reopen, NormKind enumeration, policy-predicate grammar, dense-quant payloads, sparsity static-vs-runtime, layout-metadata soundness. **Mostly YAGNI re-open-on-measurement triggers**, no near-term wave dependency.
- **Sources:** RFC-0009 §1/§9 Q6 · RFC-0010 §8 · ADR-015 · RFC-0033 §9 OQ-4/5 · ADR-030 · DN-01 §6.2/7 · DN-09 §4.

### G14 · Release / packaging / publication governance (post-1.0.0)
- **Axis:** crates.io publish flip, repo decomposition into component+re-export repos, public MIT release, `@unstable` item-marking, registry publication scope. **All explicitly gated on 1.0.0 being reached.**
- **Vehicle:** a post-1.0.0 "distribution governance" ADR set (crates.io publish ADR-018 gate; repo-decomposition DN-27/ADR-022 §10; `@unstable` ADR-023; registry-scope DN-28).
- **Sources:** ADR-018 · ADR-022 §10 / DN-27 · ADR-023 §3.5 · DN-28.

---

## Ungrouped singletons (14) — no shared ratifiable axis
`release-readiness-gate-criteria`, `prim-table-intrinsic-guarantee-citation` (RP-7 spike; all v0 prims `Exact`, non-blocking), `rfc0023-adk-lmm-leverage`, `rfc0022-http-transport` (post-1.0 web), `rfc0026-editor-configs` + `dn-24-editor-binding` + `dn-23-operator-binding` (DN-23/24 already discharged by RFC-0025/0026; editor-configs community-driven), `rfc0034`/`dn-29` per-op-granularity-thaw (post-v0 certification extension), `rfc0021`/`projection-framework`/`l4-reveal-lowering` (projection research frame, capture-only), `post-critical-quality-passes` (process discipline, not a design decision), `fn-field-maintainer-sign-off` (resolved 2026-06-27; also a G4 member).

---

## Cross-group dependencies (sequence carefully)
1. **ADR-033 FLAG-1** is both a **G4** member and **G5** freeze-condition #3 — ratify once.
2. **Distributed-reclamation** (RFC-0027 OQ-2) spans **G3** (R1 RC strategy) and **G8** (R2 distributed) — split by tier.
3. **Concurrent-session-merge** (RFC-0023) sits in **G8** but ties to **G1**'s `fuse` semantics — could move if fuse surface+semantics ratify together.
4. **G10**'s five self-hosting gates are fed by **G4 + G6 + G11** — sequence G10 last among the MED tier.

## Suggested ratification order (by 1.0-critical-path leverage)
**G1 (in flight) → G2 → G4+G5 (together, via ADR-033 FLAG-1) → G3 → G11 (FFI must-fix) → G9 → G8 → G6 → G10 → G7 → (G12/G13/G14 as cleanup/post-1.0).**

---

## Meta — changelog
- **2026-06-27 — created** from a corpus-wide parallel sweep (173 findings → 14 groups). `Empirical`,
  finder-sourced; citations to be confirmed at ratification. Anchors verified: DN-58 (Draft), RFC-0024
  (Proposed, Reynolds closure deferred). Append-only.
