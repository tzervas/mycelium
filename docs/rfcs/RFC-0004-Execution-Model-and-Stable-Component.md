# RFC-0004 — Execution Model, Backends & "Stable Component"

| Field | Value |
|---|---|
| **RFC** | 0004 |
| **Status** | **Accepted** (solidified from the research pass) |
| **Type** | Foundational / normative |
| **Date** | June 08, 2026 |
| **Depends on** | RFC-0001 (WF5 metadata-preservation, `ExecutionMode`, `Meta.physical`); ADR-009 (hybrid execution, interpreter-as-reference); DN-01 (schedule-staged packing, Resolved); Research Findings **T1.1**, **T1.4**, **T1.5** |
| **Coupled with** | RFC-0002 (shares the single certificate checker), RFC-0005 (packing-schedule selection), ADR-010 (trusted numeric base) |

## 1. Scope
The lowering `Core IR → Substrate IR → backend`; the AOT backend choice; the "stable component" gate; interpreter-as-reference equivalence (NFR-7); and the **schedule-staged packing** mechanism (DN-01, confirmed by T1.4).

## 2. Backend: MLIR backbone, LLVM native codegen (T1.5) — decision
- **Interpreter / VM = reference semantics** (NFR-7), implemented in Rust, kept as the **trusted base**. MLIR's complexity must not infect it.
- **AOT = MLIR → LLVM.** Each substrate becomes a dialect (`ternary` first; `vsa`, `embedding` deferred) lowering progressively to `linalg`/`vector`/`arith` → LLVM dialect → LLVM IR → native. Progressive, per-stage-dumpable lowering *is* the "no hidden behavior" anchor (CIRCT/IREE precedent shows the extensibility and the forward-path for a future native-ternary dialect/backend).
- **JIT** (Phase 3): same lowering + runtime specialization.
- **Custom codegen:** only a thin ternary-hardware backend later, if/when native ternary hardware arrives.
- *Cost acknowledged:* MLIR is a large, fast-moving C++ codebase with API churn and an FFI boundary from Rust; mitigated by confining it to the AOT performance path.
- *Revisit if:* a tiny stable substrate set + modest perf needs would favor a lighter direct-LLVM backend.

## 3. Single certificate checker (T1.1) — shared with RFC-0002
One refinement/equivalence-certificate checker `(A, B, R, bound, certificate)` serves **both** representation-swap validation (RFC-0002) **and** interpreter-vs-compiled equivalence. Interpreter-vs-compiled uses R = observational (or bounded) equivalence. Build once, use twice.

**Equivalence assurance is graded:**
- **Differential testing** (run interpreter + compiled, compare) as the cheap baseline — catches RR-12 divergence broadly.
- **Per-artifact translation validation** for **stable components** (the artifacts that matter), via the §3 checker.
- **Full verified compilation** (CompCert-level) is **out of scope** (KC-4 cost).

## 4. "Stable component" gate — normative
A definition is a *stable component*, and thus **AOT-eligible**, iff: (1) content-addressed and hash-frozen (Unison identity, ADR-003); (2) its spec is ratified; (3) its verification obligations (swap certificates, bound checks, reference equivalence) are discharged. **Promotion is an explicit act gated on automatic checks** (CI step): the checks must pass, but marking-stable is deliberate. Everything else runs interpreted/JIT.

## 5. Schedule-staged packing (DN-01 + T1.4) — normative
The *type* stays packing-agnostic (RFC-0001 §4.1). Packing is chosen **here, at a lowering stage** ("schedule"), recorded as inspectable `Meta.physical` on the lowered artifact, and validated against the reference semantics (no silent layout; E3 soundness check).
- **Selector:** a **cost-model + exhaustive-over-the-fixed-set benchmark** — **NOT** a Halide-class autoscheduler. T1.4 confirms the small, enumerable layout set (≈5 schemes) is *materially easier* than Halide's exponential schedule search; the "modularize scheduling without losing performance" open problem does not bite at this scale. Selection may be policy-driven via the **RFC-0005** mechanism (one mechanism, two sites).
- **Packings (reuse bitnet.cpp / Wang et al.):** **I2_S** (2-bit, lossless, multiply-add; default), **TL1** (4-bit LUT, 2.0 b/w, ARM/NEON), **TL2** (1.67 b/w, x86/AVX2, memory-bound). All match full precision within ~0.01 PPL / 0.1% accuracy; pack-and-unpack keeps int16 sums for lossless inference. Align to SIMD width.
- *Revisit if:* the layout set grows to dozens or interacts with loop structure → it re-acquires Halide's difficulty.

## 6. Lowering inspectability
Every stage is dumpable/diffable (SC-4); each pass preserves `Meta` (WF5); no-opaque-lowering applies to **all** backends (ADR-009).

## 7. Interfaces
Honors RFC-0001 WF5, `ExecutionMode`, `Meta.physical`. Shares the §3 checker with **RFC-0002**. Packing-schedule selection uses **RFC-0005**. Trusted numeric base from **ADR-010**.

## 8. Residual experiments
- **E1:** confirm staged packing reaches hand-packed perf for the 5-scheme set (expected easy per T1.4).
- **E3:** confirm a wrong `Meta.physical`/schedule tag is caught by the NFR-7 reference-equivalence check (expected: yes).
