# Phase 3 — Tooling, Projections, AI Co-authoring & Acceleration (working plan)

| Field | Value |
|---|---|
| **Status** | **Living draft** (initial cut, 2026-06-15 — scopes the Phase-3 epics into build tasks; no exit gate claimed yet) |
| **Owns** | the concrete, issue-coupled decomposition of the Phase-3 epics (#35–#41 / `E3-1…E3-7`) into `M-3xx` build tasks |
| **Source of truth above this doc** | `docs/Mycelium_Project_Foundation.md` §6 (roadmap, Phase 3), `docs/spec/SPECIFICATION.md` §10 (open build items), `tools/github/issues.yaml` + `idmap.tsv` (task/epic ids), RFC-0004 §2 (backend decision) / RFC-0006 + RFC-0007 (surface/L1, **Draft**) / ADR-007/009 (hybrid execution), DN-01 (schedule-staged packing) |
| **Mirrors** | the GitHub board: every task row carries (or will carry) its issue number from `tools/github/idmap.tsv`; the epics E3-1…E3-7 are #35–#41 |
| **Companion docs** | `phase-2.md` (predecessor, **exit-gate met** 2026-06-12); `phase-1.md`; `phase-0.md` |

> **Grounding discipline.** This is a planning artifact, not a normative one. It cites the corpus
> (`FR/NFR/VR/SC/KC`, `RFC-xxxx §`, `ADR-0xx`, `Tx.y`, `G#`, `RR-#`) for every claim about *what* is
> required; it introduces no new requirements. Where it records a *decision about sequencing or
> scope* it says so and routes anything normative back to an RFC/ADR. The honesty rule applies to the
> gate verdicts below: a guarantee tag, kill-criterion verdict, or perf number stays at the strength
> actually *established* by a checked run (VR-5), never pre-written.

---

## 1. What Phase 3 is for

Phase 2 delivered the honest-approximation substrate: the ε/δ numerics foundation, the full certified
swap surface with one shared translation-validation checker, reified selection + mandatory EXPLAIN,
the Dense + full VSA model breadth, the schedule-staged packing selector, and the reconstruction
manifest — all interpreter-side, with the reference interpreter (M-110) as the trusted base. Three
things were deliberately **left as skeletons or stubs**, by design, and recorded as such:

1. **The native execution path is a textual+env-machine skeleton.** `mycelium-mlir` emits a
   *textual* `ternary`-dialect rendering (`dialect::emit`) and runs an independent big-step
   **env-machine** (`aot::run`) for the interp↔AOT differential — but it does **not** compile to
   native code. The real `MLIR → LLVM → native` AOT backend is deferred (phase-2.md §6.18; RFC-0004
   §2; ADR-009).
2. **The E1 perf verdict is "not established."** `cargo xtask e1` times the packing codec round-trip
   only; the calibrated kernel benchmark "awaits the native libMLIR/LLVM path" (phase-2.md §6.17).
3. **The surface language above the Core IR is unratified.** RFC-0006 (surface/term-layering) and
   RFC-0007 (L1 kernel calculus) are **Draft**; concrete syntax is explicitly **KC-2-gated**
   (RFC-0006 status line). The KC-2 verdict itself is open — blocked on LLM API access (phase-2.md
   §5; M-002 / #3).

Phase 3 (Foundation §6 Phase-3) is **"dual-intelligibility tooling + performance paths."** Its
deliverables: semantic-level **projections** (G11/FR-C1), **AI co-authoring** with semantic-feedback
loops (Area 6/SC-5b), a **matured toolchain** (full LSP, diagnostics, linter/formatter, build-system
stable/experimental management), **JIT** for dynamic VSA workloads (ADR-009), opt-in **resonator
factorization** (FR-C2/G4, probabilistic-only), **BitNet-class packed-ternary acceleration**
(FR-C3/G3), and a documented **native-ternary forward-compatibility** path (R7).

These map to the epics #35–#41:

| Epic | Title | Foundation basis | Depends on (epic) | Disposition entering Phase 3 |
|---|---|---|---|---|
| **E3-1** (#35) | Semantic-level projections | FR-C1 / G11 (exploratory) | E3-3 | **needs-design**; *contingent on projection ergonomics + KC-2* |
| **E3-2** (#36) | AI co-authoring loop | NFR-2 / Area 6 / SC-5b | M-140, E2-6 | ready-to-design; *operational arm needs LLM API (KC-2-adjacent)* |
| **E3-3** (#37) | Matured toolchain | §5.6–5.8 / FR-S5 | M-140/141/142 | ready (local) |
| **E3-4** (#38) | JIT optimization | ADR-009 / RR-12 | E2-3 | ready-after native path (shares lowering) |
| **E3-5** (#39) | Resonator factorization | FR-C2 / G4 (exploratory) | E2-4, E2-5 | **needs-design**; probabilistic-only, never in core |
| **E3-6** (#40) | BitNet-class acceleration | FR-C3 / G3 | E2-7 | ready-after native path |
| **E3-7** (#41) | Native-ternary forward-compat | R7 | **M-150**, E2-7 | ready-after native path (documentation + stub target) |

### The keystone: the native execution path (maturing M-150)

Four of the seven epics (**E3-4 JIT**, **E3-6 BitNet acceleration**, **E3-7 native-ternary
forward-compat**, and the open **E1 perf verdict**) sit *downstream* of a real native backend. M-150
shipped a skeleton; Phase 3's first job is to turn it into a backend that actually compiles and runs
the kernel subset, validated against the interpreter (NFR-7). So the native path is **the keystone**
— the same role E2-4 numerics played in Phase 2 — and is sequenced first (§4).

> **Scope decision (sequencing, 2026-06-15) — direct-LLVM AOT first, MLIR deferred.** RFC-0004 §2
> commits the AOT path to **MLIR → LLVM**, and explicitly anticipates a revisit: *"a tiny stable
> substrate set + modest perf needs would favor a lighter direct-LLVM backend."* The Phase-3 build
> environment has **LLVM 18 tooling present** (`llvm-config`, `clang`, `opt`, `llc`) but **libMLIR
> absent** (`mlir-opt`/`mlir-translate` not installed, and libMLIR is a large C++ build). A textual
> LLVM-IR → `llc` → `clang` → native → run roundtrip works here today; an MLIR one does not. So the
> realized first step is a **direct LLVM-IR AOT backend for the kernel subset** (the RFC-0004 §2
> fallback), which discharges the *runnable-native-artifact* obligation and the E1 verdict without
> libMLIR. The `ternary`-dialect MLIR lowering stays the **eventual** path (the textual emitter is
> its dumpable skeleton, retained) and is **deferred** until libMLIR is provisioned. This is a
> sequencing/scope decision; if the direct-LLVM backend becomes load-bearing rather than a stopgap,
> it should be ratified as an RFC-0004 erratum or a short ADR (maintainer decision) — flagged in §7
> (RR-N1), not silently adopted.

---

## 2. The Phase-3 task set (proposed decomposition & readiness)

The epics decompose into `M-3xx` build tasks below. **Issue numbers are pending** — these tasks are
not yet created on the board; the `idmap.tsv` join lands when they are bootstrapped from
`issues.yaml` (§8). Readiness is relative to the corpus + landed Phase-1/2 deps.

| Task | Epic | Pri | Depends on | Maps to | Readiness |
|---|---|---|---|---|---|
| **M-301** Direct-LLVM-IR AOT backend (kernel subset) | E3-7 (prereq) | P1 | M-150, M-110 | RFC-0004 §2 / ADR-007/009 | **In progress (2026-06-15)** — bit subset landed (`mycelium-mlir::llvm`); trit subset is the next slice |
| **M-302** interp↔native differential (extend M-151) | E3-7 (prereq) | P1 | M-301, M-151 | NFR-7 / VR-4 / RR-12 | **Done (2026-06-15)** — `tests/native_differential.rs` (bit subset; toolchain-gated skip) |
| **M-303** E1 perf verdict on the native path | E3-7 (prereq) | P1 | M-301, M-302 | E1 / NFR-4 | **Done (2026-06-15)** — `cargo xtask e1` §2 measures native AOT vs interp; compute-throughput verdict still pending in-process exec |
| **M-310** Full-LSP maturation (rich diagnostics) | E3-3 | P1 | M-140, M-141 | §5.6–5.8 / SC-5 | Ready (local) |
| **M-311** Build-system: stable/experimental + cert artifacts | E3-3 | P1 | RFC-0004 §4 | RFC-0004 §4 / ADR-003 | Ready (local) |
| **M-312** Content-addressed build cache | E3-3 | P2 | M-311 | ADR-003 | Ready after M-311 |
| **M-320** L1 term-language extension (interpreter/prototype) | E3-3 / RFC-0007 | P1 | M-110, RFC-0007 | RFC-0007 §§3–4 | Ready (local); **RFC ratification is maintainer's** |
| **M-330** AI co-authoring loop (generate→feedback→fix) | E3-2 | P1 | M-140, E2-6 | NFR-2 / SC-5b | Harness local; **run needs LLM API** (KC-2-adjacent) |
| **M-340** JIT path (shares lowering + runtime specialization) | E3-4 | P2 | M-301 | ADR-009 / RR-12 | Ready after native path |
| **M-350** Resonator-network factorization (opt-in, probabilistic) | E3-5 | P2 | E2-4, M-260 | FR-C2 / G4 / RFC-0003 §6 | **needs-design** |
| **M-360** Production packed-ternary acceleration | E3-6 | P2 | E2-7, M-301 | FR-C3 / G3 | Ready after native path |
| **M-370** Native-ternary forward-compat mapping (+ stub target) | E3-7 | P2 | M-150, M-301 | R7 | Ready after native path |
| **M-380** Semantic-level projection framework | E3-1 | P2 | E3-3 | FR-C1 / G11 | **needs-design**; *KC-2-contingent* |
| **M-002** KC-2 LLM-leverage run (carried; gates E3-1 + concrete syntax) | E4 | P0 | M-020 (harness landed) | SC-5b / G10 / KC-2 | **Blocked (external)** — needs LLM API |

Legend — **Ready (local)**: can start now from the corpus + landed deps in this environment.
**Blocked (external)**: a hard dependency outside the repo (LLM API, libMLIR). **needs-design**: an
RFC/design step precedes the build.

---

## 3. Batch structure (the parallelization plan)

Phase 3 sequences into batches; tasks **within** a batch touch different modules/crates and
parallelize, while batches serialize on real dependencies.

- **Batch J — native execution path** (keystone): **M-301** (direct-LLVM-IR backend) → **M-302**
  (interp↔native differential) → **M-303** (E1 verdict). All local. Unblocks E3-4/E3-6/E3-7 + E1.
- **Batch K — toolchain & surface** (independent of J): **M-310** (LSP), **M-311/M-312**
  (build-system + cache), and **M-320** (L1 term-language extension) parallelize — they touch
  `mycelium-lsp`, a new build crate, and `mycelium-l1` respectively. RFC-0006/0007 *ratification*
  rides alongside but is a **maintainer decision** (and concrete syntax is KC-2-gated).
- **Batch L — acceleration & execution breadth** (depends on J): **M-340** (JIT), **M-360**
  (BitNet acceleration), **M-370** (native-ternary forward-compat doc + stub).
- **Batch M — exploratory** (depends on numerics/recon, and KC-2): **M-350** (resonator
  factorization, needs-design), **M-380** (projections, KC-2-contingent), **M-330** (AI co-authoring,
  run needs API).
- **External probe — KC-2** (M-002): the harness is landed (phase-2.md §5); the **run** is blocked on
  LLM API access and stays out of every batch's gate until that exists.

---

## 4. Critical path & sequencing

```
 Batch J — native execution path (KEYSTONE, local):
   M-301 direct-LLVM-IR AOT backend (kernel subset) ─► M-302 interp↔native differential (NFR-7)
                                                          │
                                                          ▼
                                                       M-303 E1 perf verdict (retires "not established")
   unblocks ▼
   E3-4 (JIT, M-340) · E3-6 (BitNet accel, M-360) · E3-7 (native-ternary, M-370)

 Batch K — toolchain & surface (PARALLEL to J, local):
   M-310 full-LSP ──┐
   M-311 build-system ─► M-312 content-addressed cache
   M-320 L1 term-language extension  (RFC-0006/0007 ratification = maintainer; concrete syntax KC-2-gated)

 Batch L (depends J):  M-340 JIT ── M-360 BitNet accel ── M-370 native-ternary forward-compat doc

 Batch M (exploratory): M-350 resonator (needs-design) · M-380 projections (KC-2-contingent) · M-330 AI co-authoring (run needs API)

 External probe: KC-2 (M-002) — blocked on LLM API; gates E3-1 + concrete-syntax ratification
```

**Why the native path is the keystone.** The E1 perf verdict, the JIT path (E3-4 reuses the same
lowering + a runtime-specialization layer), the BitNet acceleration paths (E3-6 needs a real codegen
target to accelerate), and the native-ternary forward-compat mapping (E3-7) all assume a backend that
actually compiles and runs. So Batch J is built first; the toolchain/surface work (Batch K) is
genuinely independent and runs in parallel.

**Honest blockers on the critical path.** Two Phase-3 deliverables cannot be *completed* in this
environment and are sequenced as out-of-gate external probes, never pre-resolved:

- **KC-2 (M-002)** needs LLM API access. It gates **E3-1 projections** (Foundation: "contingent on
  projection ergonomics + KC-2") and **concrete-syntax ratification** (RFC-0006 status). The harness
  is ready; the verdict stays "not established" until a generator runs (VR-5).
- **The MLIR dialect lowering** needs libMLIR. Deferred per the §1 scope decision; the direct-LLVM
  backend (M-301) discharges the runnable-native obligation in the meantime.

---

## 5. Gate verdicts — Phase-2→3 re-run of KC-1…KC-4 (honest status)

Per the honesty rule and VR-5, kill-criterion status is tracked at the strength actually
*established*. Re-run at the Phase-2→3 gate. **No Phase-3 exit gate is claimed yet** — this doc opens
the phase; the gate is defined in §6 and verdicts are filled as tasks land.

| Gate | Question | Phase-2→3 verdict (2026-06-15) | What moves it in Phase 3 |
|---|---|---|---|
| **KC-1** | Honest, usefully-tight bound for a core VSA op? | ✅ **confirmed (build)** — carried; no regression. M-131 `Proven` capacity bound; the §4-matrix tags hold. | Resonator factorization (M-350) adds a **probabilistic-only** bound (FR-C2/G4) — tagged at the strength its convergence analysis supports, never upgraded. |
| **KC-2** | LLM code-gen/reasoning survives the Mycelium surface? | **open — blocked (external)** — unchanged from phase-2.md §5. The M-002 harness is landed; *running* it needs LLM API access; the report hard-codes "not established." | Plug an LLM generator into the harness `Generator` protocol when API access exists; then it gates E3-1 + concrete-syntax ratification. |
| **KC-3** | Kernel stays single-expert auditable? | **holding** — `mycelium-core` stayed small through Phase 2 (numerics/select/mlir all *outside* it). | Phase 3 adds the most surface yet (native backend, JIT, toolchain). **Decision:** the native backend lives in `mycelium-mlir` (already outside core); the build-system + JIT land as their own crates; core gains nothing executable. Re-assess at the Phase-3 gate. |
| **KC-4** | Per-swap certificate-check overhead within budget? | **measured (M-212), unchanged** — same order as the swap; downgrade path not triggered. Numeric budget still unratified (maintainer). | The native path lets KC-4 be **re-measured on a compiled artifact** (M-303) rather than the interpreter — a more representative number for the eventual budget ratification. |

---

## 6. Phase-3 exit gate (proposed — not yet met)

Phase 3 is large and partly exploratory/external-blocked, so the gate is scoped to the **buildable,
local** deliverables; the exploratory/KC-2-gated epics (E3-1 projections, the E3-2 operational arm,
E3-5 resonator) are tracked as **stretch** items whose verdicts stay honest (VR-5) and do **not**
hold the gate. Proposed: Phase 3 closes when **all** of —

- **Native execution path** — the direct-LLVM-IR backend (M-301) compiles and runs the kernel subset
  to a native artifact; the interp↔native differential (M-302) passes on the kernel corpus and
  **catches a deliberately divergent lowering** (NFR-7/RR-12, the mutant-witness convention); the E1
  perf verdict (M-303) is **measured** (a real native-vs-interpreter number, honestly captioned —
  not "within an agreed target" unless one is ratified).
- **Matured toolchain** — the full-LSP diagnostics (M-310) and the build-system stable/experimental
  management + certificate artifacts (M-311) ship with tests; content-addressed caching (M-312)
  demonstrably reuses a prior build's certificate.
- **L1 surface** — the term-language extension (M-320) lands in the `mycelium-l1` prototype with
  tests; RFC-0007 is presented for ratification (the **decision** is the maintainer's, recorded
  append-only; concrete syntax stays KC-2-gated).

**Stretch (honest, out-of-gate):** JIT (M-340, needs an agreed speedup target), BitNet acceleration
(M-360), native-ternary forward-compat doc (M-370), resonator factorization (M-350, needs-design),
projections (M-380, KC-2-contingent), AI co-authoring run (M-330, needs API), and the KC-2 verdict
(M-002, needs API). Each is delivered as far as the environment allows and its verdict pinned at the
established strength.

> The §6 gate is itself a **proposed** scope decision (what counts as "Phase-3 done" given two
> external blockers). It is recorded here for the maintainer to ratify or adjust; it is not asserted
> as met.

---

## 7. Risks & open questions

| Id | Item | Disposition |
|---|---|---|
| **RR-11** | Toolchain + multi-backend scope balloons. | Phase-3 mitigation: reuse LLVM (M-301 emits textual IR, no hand-rolled codegen); the native backend stays in `mycelium-mlir`; build-system/JIT are separate crates (KC-3). Batches serialize on real deps so scope stays auditable. |
| **RR-12** | Interpreter↔AOT/JIT semantic divergence. | The M-151/M-251 differential already folds into the M-210 checker. M-302 extends it to the **real compiled artifact** (not just the env-machine), and M-340 (JIT) validates through the same machinery (NFR-7). |
| **RR-N1** *(new)* | The **direct-LLVM AOT** path diverges from the ratified **MLIR→LLVM** decision (RFC-0004 §2). | **Sequencing decision (§1):** direct-LLVM first (libMLIR absent), MLIR deferred; the textual `ternary`-dialect emitter is retained as the MLIR skeleton. If direct-LLVM becomes load-bearing rather than a stopgap, ratify an RFC-0004 erratum / short ADR (maintainer). Flagged, not silently adopted. |
| **KC-2 / RR-3** | LLM leverage on the Mycelium surface unverified. | **Blocked (external):** harness landed, run needs LLM API. Gates E3-1 + concrete-syntax ratification. Verdict stays "not established" (VR-5); the embedded-DSL fallback (Foundation KC-2) is the documented contingency. |
| **G4 / RR-5** | Resonator factorization may not converge. | M-350 stays **opt-in, probabilistic-only, never in core** (FR-C2; the M-260 manifest already enforces the `Resonator` probabilistic-only ceiling in the type). needs-design: document the convergence regime + bounds before building. |
| **G11** | Semantic-level (not notational) projections may not be ergonomically viable. | E3-1/M-380 is **exploratory + KC-2-contingent**; deferred to Batch M. The content-addressed IR + EXPLAIN dumps already give a projection substrate (Stage-D roadmap §"Mycelium-lang forward"). |

---

## 8. How this doc stays honest

- **Append-only with status transitions**, mirroring the ADR/RFC discipline: this file moves
  `Living draft → exit-gate met` only when the §6 gate is met; task rows update in place as their
  issues progress, but gate verdicts (§5) and the E1 perf number never pre-record an upgrade.
- **Every task row will carry its issue number** once the `M-3xx` tasks are bootstrapped from
  `issues.yaml` into the board (`idmap.tsv` is the join key) — until then they are marked
  *issue pending* (§2).
- **The two external blockers are named, not hidden** (KC-2 needs an LLM API; the MLIR path needs
  libMLIR) — each is sequenced as an out-of-gate probe with an honest "not established" verdict.

---

## 9. Per-task detail (filled as tasks land)

### 9.1 M-301 — Direct-LLVM-IR AOT backend (bit subset) · Batch J · P1 · in progress 2026-06-15

- **Goal (from §2 / issues.yaml).** A genuinely compiled native artifact for the kernel subset via
  the RFC-0004 §2 direct-LLVM fallback (libMLIR absent; LLVM 18 present), each stage dumpable,
  unsupported ops an explicit refusal.
- **Delivered (first slice — bit subset).** `mycelium-mlir::llvm`: `emit_llvm_ir(node)` lowers the
  `mycelium-core::lower` ANF for the **bit subset** (`core.id`, `bit.not/and/or/xor` over
  `Binary{w}`) to **textual LLVM IR** — one SSA op per output bit (`xor i32 x, 1` for `not`;
  `and`/`or`/`xor i32` for the binops), result bits written via `@putchar` as a `'0'`/`'1'` line
  (no opaque pass — RFC-0004 §6, deterministic). `compile_and_run(node)` drives `llc -filetype=obj`
  then `clang` to a native executable, runs it, parses stdout, and reconstructs an **`Exact`**
  `Binary{w}` `Value` (bit ops are exact; approximate inputs are out of subset). Everything outside
  the subset is an explicit `AotError` — `UnsupportedRepr` (non-`Binary`), `UnsupportedPrim`
  (`trit.*`), `UnsupportedNode` (swap), `WidthMismatch` — and `llc`/`clang` absence is a **skippable**
  `ToolchainMissing` (the house "skip gracefully" idiom), so the compiled smoke test no-ops where
  the toolchain is missing rather than failing.
- **Tests (`llvm::tests`).** Emit shape + determinism; the four refusals (each with a mutant-witness
  comment — guard 7); a width-mismatch refusal; and the compiled `native_bit_not_matches_interpreter`
  roundtrip (toolchain-gated) asserting the native payload equals the complemented input (mutant:
  an `or`/`and` mis-lowering would diverge).
- **Honesty / scope.** The MLIR `ternary`-dialect lowering stays the **eventual** path (`dialect::emit`
  is its dumpable skeleton) and is **deferred** until libMLIR exists (RR-N1). The **trit subset**
  (balanced-ternary carry chains) is the next M-301 slice; it is refused here, not half-lowered. No
  guarantee is upgraded: the reconstructed `Value` is `Exact` only because the bit ops are exact and
  the subset refuses approximate inputs (VR-5).

### 9.2 M-302 — interp↔native differential · Batch J · P1 · done 2026-06-15

- **Goal (from §2 / issues.yaml).** Extend the M-151 differential so the kernel corpus runs under
  the interpreter **and** the real compiled native artifact, asserting observable equivalence through
  the M-210 checker; a divergent lowering must be caught (NFR-7/VR-4/RR-12).
- **Delivered.** `crates/mycelium-mlir/tests/native_differential.rs`: a small deterministic
  **bit-subset** corpus (const, `core.id`, `let`/`var`, `bit.not/and/or/xor`, a nested
  `not(a xor b)`) is run through the M-110 reference interpreter and
  `mycelium_mlir::compile_and_run` (the M-301 compiled path), asserting the observable
  `(repr, payload, guarantee)` matches **and** the pair validates through
  `check(.., ObservationalEquiv, Certificate::exact(), Observational)` — the same shared TV checker
  the M-151 env-machine differential uses. A second test compiles two different programs
  (`not(A)` vs `id(A)`) and asserts the checker reports `NotValidated` — the differential
  discriminates, so a pass is meaningful (guard 7, mutant-witness comments inline). Both tests
  **skip** on `AotError::ToolchainMissing` (no `llc`/`clang`), never a false failure.
- **Honesty / scope.** Scoped to the bit subset (what M-301 lowers today); the trit subset joins the
  corpus when M-301's trit slice lands. The env-machine M-151 differential is unchanged and still
  covers the full corpus (swaps/trits) — M-302 *adds* the compiled-artifact path, it does not
  replace it.

### 9.3 M-303 — E1 perf verdict on the native path · Batch J · P1 · done 2026-06-15

- **Goal (from §2 / issues.yaml).** Replace the `cargo xtask e1` stub's "not established" with a
  measured native-vs-interpreter number now that the M-301 native path exists; honest caption, no
  pre-written perf claim (VR-5).
- **Delivered.** `xtask::e1` gains §2 (M-303): using the compile-once/run-many split
  (`mycelium_mlir::{compile, CompiledArtifact::run}`, refactored from `compile_and_run`), it times,
  for `not(a xor b)` over 8 bits — (a) the one-time **AOT compile** (emit IR → `llc` → `clang`),
  (b) the warm **native per-invocation** (process spawn + run + read-back), and (c) the reference
  **interpreter** per-eval — and skips on `ToolchainMissing`. Indicative single run (containerized
  x86-64): AOT compile ≈ 112 ms one-time; native per-invocation ≈ 1.3 ms; interpreter ≈ 3.8 µs.
- **Honest verdict.** The native AOT path is now **established and measured end-to-end** (was: no
  native path at all). A *calibrated compute-throughput* verdict ("reaches hand-packed perf") stays
  **NOT established** — and the reason is now precise: the standalone tiny-kernel artifact's
  per-invocation cost is **process-spawn-bound** (1.3 ms ≫ the 3.8 µs interpreter eval), and the
  constant inputs constant-fold, so a meaningful kernel-throughput number needs **in-process
  execution** (JIT/FFI — M-340, or the deferred libMLIR backend). This narrows the open E1 question
  from "no native path" to "needs in-process execution," recorded honestly (VR-5) rather than
  pronounced.
- **Scope.** The §1 packing-codec measurement is retained (staging-cheap confirmation). KC-4 §5 notes
  the native path now allows a compiled-artifact re-measure when an in-process path lands.

## Meta — changelog & maintenance

- **2026-06-15 (M-303 E1 native-path measurement lands — Batch J complete at the task level):**
  `cargo xtask e1` §2 measures the native AOT path (compile-once / run-many) against the interpreter;
  the E1 verdict moves from "no native path (stub)" to "native path established + measured, compute
  throughput pending in-process execution." `compile_and_run` refactored into `compile` +
  `CompiledArtifact::run`. §2 M-303 row → done; §9.3 added. **Batch J (M-301→M-302→M-303) is now done
  at the task level** — the native execution path keystone is in place (bit subset).
- **2026-06-15 (M-302 interp↔native differential lands):** the compiled M-301 path is now checked
  against the reference interpreter on the bit-subset corpus through the single shared M-210 checker
  (`tests/native_differential.rs`), with a discrimination test and graceful toolchain-absent skips.
  §2 M-302 row → done; §9.2 added. Next: M-303 E1 perf verdict on the native path.
- **2026-06-15 (M-301 bit-subset slice lands):** the direct-LLVM-IR AOT backend
  (`mycelium-mlir::llvm`) compiles the bit subset to native code via `llc`/`clang` and reads the
  result back — the first *compiled* execution path (RFC-0004 §2 direct-LLVM fallback). §2 M-301 row
  → in progress; §9.1 per-task detail added. Next: M-301 trit slice, then the M-302 interp↔native
  differential and the M-303 E1 verdict.
- **2026-06-15 (initial scoping cut):** authored the Phase-3 plan. Decomposes epics #35–#41
  (E3-1…E3-7) into `M-3xx` build tasks (§2), records the batch/parallelization plan (§3) with the
  **native execution path as the keystone** (§1, §4), the Phase-2→3 KC re-run (§5), a **proposed**
  exit gate scoped to the buildable/local deliverables with the exploratory + KC-2-gated epics as
  honest out-of-gate stretch items (§6), and the risk register incl. the new **RR-N1** (direct-LLVM
  vs MLIR sequencing decision) and the carried KC-2/libMLIR external blockers (§7). No exit gate
  claimed. Status: **Living draft**.
- Maintain append-only; supersede, don't rewrite. Re-run KC-1…KC-4 at the phase gate (Foundation
  Meta). Keep `Proven|Empirical|Declared` verdicts and the E1 perf number honest per VR-5.
</content>
</invoke>
