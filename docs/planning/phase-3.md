# Phase 3 — Tooling, Projections, AI Co-authoring & Acceleration (working plan)

| Field | Value |
|---|---|
| **Status** | **Exit-gate MET** (re-asserted 2026-06-15 — the §6 gate's three conditions are satisfied and the two prior residuals **R1 + R2 are closed**, §6.1: native execution path met+measured; matured toolchain met (text→`Node`→L0 pipeline exists — the `didOpen`/`didChange` wiring is an ordinary M-310 build task, no longer gate-blocking); L1 surface met (RFC-0011 r3 enacted, RFC-0001 → r3, RFC-0006/0007 Accepted r4). Claimed at the strength the checked runs establish (VR-5): 497 workspace tests + the M-210 data-fragment differential. Phase 3's named build tasks continue, but the **exit gate is closed**. Supersedes the prior "Living draft / no exit gate claimed" line — append-only.) |
| **Owns** | the concrete, issue-coupled decomposition of the Phase-3 epics (#35–#41 / `E3-1…E3-7`) into `M-3xx` build tasks |
| **Source of truth above this doc** | `docs/Mycelium_Project_Foundation.md` §6 (roadmap, Phase 3), `docs/spec/SPECIFICATION.md` §10 (open build items), `tools/github/issues.yaml` + `idmap.tsv` (task/epic ids), RFC-0004 §2 (backend decision) / RFC-0006 + RFC-0007 (surface/L1, **Accepted r5/r4**; concrete L3 syntax committed, DN-09) / ADR-007/009 (hybrid execution), DN-01 (schedule-staged packing) |
| **Mirrors** | the GitHub board: every task row carries (or will carry) its issue number from `tools/github/idmap.tsv`; the epics E3-1…E3-7 are #35–#41 |
| **Companion docs** | `phase-2.md` (predecessor, **exit-gate met** 2026-06-12); `phase-1.md`; `phase-0.md` |

> **Grounding discipline.** This is a planning artifact, not a normative one. It cites the corpus
> (`FR/NFR/VR/SC/KC`, `RFC-xxxx §`, `ADR-0xx`, `Tx.y`, `G#`, `RR-#`) for every claim about *what* is
> required; it introduces no new requirements. Where it records a *decision about sequencing or
> scope* it says so and routes anything normative back to an RFC/ADR. The transparency rule applies to the
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
   §5; M-002 / #3). *(This was the entering state; **since closed**: RFC-0006/0007 are Accepted
   r5/r4, KC-2 verdict = proceed (DN-09, 2026-06-18) — see §5/§2/§7 for the current status.)*

Phase 3 (Foundation §6 Phase-3) is **"dual-intelligibility tooling + performance paths."** Its
deliverables: semantic-level **projections** (G11/FR-C1), **AI co-authoring** with semantic-feedback
loops (Area 6/SC-5b), a **matured toolchain** (full LSP, diagnostics, linter/formatter, build-system
stable/experimental management), **JIT** for dynamic VSA workloads (ADR-009), opt-in **resonator
factorization** (FR-C2/G4, probabilistic-only), **BitNet-class packed-ternary acceleration**
(FR-C3/G3), and a documented **native-ternary forward-compatibility** path (R7).

These map to the epics #35–#41:

| Epic | Title | Foundation basis | Depends on (epic) | Disposition entering Phase 3 |
|---|---|---|---|---|
| **E3-1** (#35) | Semantic-level projections | FR-C1 / G11 (exploratory) | E3-3 | **design-active** — KC-2 cleared (DN-09, 2026-06-18); M-380 opened co-equally as the projection layer (FR-S5 dual rendering) |
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

The epics decompose into `M-3xx` build tasks below. **Issues created 2026-06-15** — the M-3xx tasks
are now on the board as **#86–#98**, linked as sub-issues of the E3-1…E3-7 epics (#35–#41), with the
`idmap.tsv` join landed (§8); the six already-shipped tasks (M-301/302/303/311/312/370) are closed as
completed. Readiness is relative to the corpus + landed Phase-1/2 deps.

| Task | Epic | Pri | Depends on | Maps to | Readiness |
|---|---|---|---|---|---|
| **M-301** Direct-LLVM-IR AOT backend (kernel subset) | E3-7 (prereq) | P1 | M-150, M-110 | RFC-0004 §2 / ADR-007/009 | **Done (2026-06-15)** — bit subset + `trit.neg` + trit *carry arithmetic* (`add/sub/mul`, ripple-carry/shifted-accumulate, runtime-overflow read-back); shared by AOT + JIT |
| **M-302** interp↔native differential (extend M-151) | E3-7 (prereq) | P1 | M-301, M-151 | NFR-7 / VR-4 / RR-12 | **Done (2026-06-15)** — `tests/native_differential.rs` (bit subset; toolchain-gated skip) |
| **M-303** E1 perf verdict on the native path | E3-7 (prereq) | P1 | M-301, M-302 | E1 / NFR-4 | **Done (2026-06-15)** — `cargo xtask e1` §2 measures native AOT vs interp; compute throughput now measured over runtime data in §3 (M-360) |
| **M-310** Full-LSP maturation (rich diagnostics) | E3-3 | P1 | M-140, M-141 | §5.6–5.8 / SC-5 | **Done (2026-06-16)** — structured `FeedbackSummary` + navigable `Diagnostic::path()`; **LSP wire protocol** (`wire`: JSON-RPC `Content-Length` framing, `publishDiagnostics`, `initialize`/`shutdown`/`exit`); and now **real document sync** (`sync`: `didOpen`/`didChange`/`didClose` over the parse→check→L0 pipeline, `TextDocumentSyncKind.Full`, `publishDiagnostics` on each edit). **Honest spans:** a *parse* diagnostic carries a real `line:col` range (lexer `Pos`); a *check* diagnostic is located at its `fn <name>` declaration with the function in `data.breadcrumb` (precise sub-expression spans await the checker carrying spans). |
| **M-311** Build-system: stable/experimental + cert artifacts | E3-3 | P1 | RFC-0004 §4 | RFC-0004 §4 / ADR-003 | **Done (2026-06-15)** — `mycelium-build` crate (decide + content-addressed `BuildCertificate`) |
| **M-312** Content-addressed build cache | E3-3 | P2 | M-311 | ADR-003 | **Done (2026-06-15)** — `mycelium-build::cache` (`BuildCache`, request-addressed) |
| **M-320** L1 term-language extension (interpreter/prototype) | E3-3 / RFC-0007 | P1 | M-110, RFC-0007 | RFC-0007 §§3–4 | **Done (2026-06-15)** — literal-pattern `match` + **nested patterns** (Maranget usefulness: exhaustiveness/redundancy with witnesses) + the **Maranget decision-tree compiler**, and now the **codegen half lands in L0**: `elab` emits the tree's leaves as **kernel `Construct`/flat `Match` nodes** (RFC-0011 r3 enacted; RFC-0001 → r3). The M-110 interpreter evaluates them and the M-210 differential covers the data fragment (L1-eval ≡ elaborate→L0-interp, mutant-witness). `Lam/App/Fix` → r4 |
| **M-330** AI co-authoring loop (generate→feedback→fix) | E3-2 | P1 | M-140, E2-6 | NFR-2 / SC-5b | Harness local; **run needs LLM API** (KC-2-adjacent) |
| **M-340** JIT path (shares lowering + runtime specialization) | E3-4 | P2 | M-301, ADR-014 | ADR-009 / RR-12 | **In progress (2026-06-15)** — in-process `dlopen` JIT (`mycelium-mlir::jit`), NFR-7 checked; **+ runtime-specialization layer** (`mycelium-mlir::specialize`): bakes the runtime-known weight vector into the dot kernel (zero lanes elided, unpack dropped, ±1 → add/sub), validated generic↔specialized through the shared M-210 checker, E1 §4 records the **measured** speedup (no target pre-written) |
| **M-350** Resonator-network factorization (opt-in, probabilistic) | E3-5 | P2 | E2-4, M-260 | FR-C2 / G4 / RFC-0003 §6 | **Design drafted (2026-06-15)** — RFC-0009 (convergence regime, `Empirical`-ceiling honesty, never-silent verdicts); prototype gated on ratification |
| **M-360** Production packed-ternary acceleration | E3-6 | P2 | E2-7, M-301 | FR-C3 / G3 | **In progress (2026-06-15)** — runtime-data dot kernels for **all three** bitnet packings (I2_S/TL1/TL2; in-process JIT, inspectable per-scheme unpack IR), each oracle-checked; E1 §3 measured. **+ hand-vectorized SIMD** (`mycelium-mlir::simd`, 8-wide I2_S, vector-IR unpack) differential-checked against the scalar oracle through the shared M-210 checker, E1 §5 measured. **+ true 1.67-b/w TL2** (`pack`: 3 trits → 5-bit LUT-index bitstream; the dot kernel decodes it with a bounds-clamped 2-byte window) — **closes A5-08** (codec now matches the selector cost model). Remaining: TL1/TL2 vectorized SIMD unpacks |
| **M-370** Native-ternary forward-compat mapping (+ stub target) | E3-7 | P2 | M-150, M-301 | R7 | **Done (2026-06-15)** — `docs/notes/Native-Ternary-Forward-Compat.md`; dialect = stub target |
| **M-380** Semantic-level projection framework | E3-1 | P2 | E3-3 | FR-C1 / G11 | **design-active** — KC-2 cleared (DN-09); the projection layer is now a **co-equal deliverable** (FR-S5 dual rendering; DN-09 §3.1 / M-380 opened co-equally) |
| **M-002** KC-2 LLM-leverage run (carried; gates E3-1 + concrete syntax) | E4 | P0 | M-020 (harness landed) | SC-5b / G10 / KC-2 | **Done — verdict recorded (DN-09) = proceed** (weak-but-recoverable; 7B+examples 70% eventual, +30pp edit-to-fix; kill criterion not triggered; T3.6 ablation still open — DN-09 §4) |

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
- **Batch M — exploratory** (depends on numerics/recon): **M-350** (resonator
  factorization, needs-design), **M-380** (projections — **KC-2 cleared; now design-active, DN-09**),
  **M-330** (AI co-authoring, run needs API).
- **External probe — KC-2** (M-002): **CLOSED — verdict recorded (DN-09, 2026-06-18) = proceed.**
  The run completed (local Qwen2.5-Coder, seed 42); see §5 KC-2 row and §2 M-002 row.

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
   M-320 L1 term-language extension  (RFC-0006/0007 ratification = maintainer; concrete syntax now committed — DN-09)

 Batch L (depends J):  M-340 JIT ── M-360 BitNet accel ── M-370 native-ternary forward-compat doc

 Batch M (exploratory): M-350 resonator (needs-design) · M-380 projections (design-active; KC-2 cleared DN-09) · M-330 AI co-authoring (run needs API)

 KC-2 (M-002): CLOSED — verdict recorded (DN-09) = proceed; unblocked E3-1 + concrete-syntax ratification
```

**Why the native path is the keystone.** The E1 perf verdict, the JIT path (E3-4 reuses the same
lowering + a runtime-specialization layer), the BitNet acceleration paths (E3-6 needs a real codegen
target to accelerate), and the native-ternary forward-compat mapping (E3-7) all assume a backend that
actually compiles and runs. So Batch J is built first; the toolchain/surface work (Batch K) is
genuinely independent and runs in parallel.

**Honest blockers on the critical path.** One original external blocker remains; the other is closed:

- **KC-2 (M-002) — CLOSED (2026-06-18).** Verdict = proceed (DN-09); the run completed
  (local Qwen2.5-Coder). **E3-1/M-380 is now design-active**; **concrete L3 syntax is committed**
  (RFC-0006 r5). The T3.6 rigorous ablation remains an honest open follow-up (DN-09 §4); see §5.
- **The MLIR dialect lowering** needs libMLIR. Deferred per the §1 scope decision; the direct-LLVM
  backend (M-301) discharges the runnable-native obligation in the meantime.

---

## 5. Gate verdicts — Phase-2→3 re-run of KC-1…KC-4 (honest status)

Per the transparency rule and VR-5, kill-criterion status is tracked at the strength actually
*established*. Re-run at the Phase-2→3 gate. **No Phase-3 exit gate is claimed yet** — this doc opens
the phase; the gate is defined in §6 and verdicts are filled as tasks land.

| Gate | Question | Phase-2→3 verdict (2026-06-15) | What moves it in Phase 3 |
|---|---|---|---|
| **KC-1** | Honest, usefully-tight bound for a core VSA op? | ✅ **confirmed (build)** — carried; no regression. M-131 `Proven` capacity bound; the §4-matrix tags hold. | Resonator factorization (M-350) adds a **probabilistic-only** bound (FR-C2/G4) — tagged at the strength its convergence analysis supports, never upgraded. |
| **KC-2** | LLM code-gen/reasoning survives the Mycelium surface? | **verdict recorded (DN-09) = proceed** — the M-002 run (local Qwen2.5-Coder, 10-task gold set, seed 42) shows *weak-but-recoverable* leverage (best arm 7B+examples: 40% first-attempt → 70% eventual, +30pp edit-to-fix); the "irrecoverable collapse even with feedback" kill criterion (**not triggered**). Selects L3 strategy = **committed text syntax + co-equal projection layer (M-380)**; retains embedded-DSL fallback (RR-3) unspent. The T3.6 retention-ratio ablation (the rigorous falsification run) was **not run** — this remains an honest open follow-up (DN-09 §4); VR-5 — weak-but-recoverable is the established strength, not the strong Q1 hypothesis. | E3-1 (M-380) moves to design-active; concrete-syntax ratification unblocked (RFC-0006 r5). T3.6 ablation tracked for a future variant pass (DN-09 §4 / research-prompts.md). |
| **KC-3** | Kernel stays single-expert auditable? | **holding** — `mycelium-core` stayed small through Phase 2 (numerics/select/mlir all *outside* it). | Phase 3 adds the most surface yet (native backend, JIT, toolchain). **Decision:** the native backend lives in `mycelium-mlir` (already outside core); the build-system + JIT land as their own crates; core gains nothing executable. Re-assess at the Phase-3 gate. |
| **KC-4** | Per-swap certificate-check overhead within budget? | **measured (M-212), unchanged** — same order as the swap; downgrade path not triggered. Numeric budget still unratified (maintainer). | The native path lets KC-4 be **re-measured on a compiled artifact** (M-303) rather than the interpreter — a more representative number for the eventual budget ratification. |

---

## 6. Phase-3 exit gate (proposed — **met; both residuals closed 2026-06-15**, §6.1)

Phase 3 is large and partly exploratory/external-blocked, so the gate is scoped to the **buildable,
local** deliverables; the exploratory epics (E3-1 projections — **KC-2 now cleared, design-active
(DN-09)**; the E3-2 operational arm; E3-5 resonator) are tracked as **stretch** items whose verdicts
stay honest (VR-5) and do **not** hold the gate. Proposed: Phase 3 closes when **all** of —

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

**Stretch (honest, out-of-gate):** JIT (M-340 — **delivered**: in-process JIT + runtime
specialization with a measured ≈10.7× E1 §4 speedup, no pre-written target), BitNet acceleration
(M-360 — in progress; SIMD + true-TL2 next), native-ternary forward-compat doc (M-370 — **done**),
resonator factorization (M-350 — design-drafted, RFC-0009/0010 ratified), projections (M-380 —
**design-active, KC-2 cleared DN-09**), AI co-authoring run (M-330, needs API), and the KC-2 verdict
(M-002 — **CLOSED, verdict = proceed, DN-09 2026-06-18**).
Each is delivered as far as the environment allows and its verdict pinned at the established strength.

### 6.1 Verdict fill (2026-06-15, VR-5 — at the strength actually established)

Re-read against the landed work. Each gate condition is filled at the strength a checked run
established; **gate-blocking** vs **stretch** is flagged, and no verdict is pre-written.

| Gate condition | Verdict (2026-06-15) | Gate-blocking? |
|---|---|---|
| **Native execution path** (M-301/302/303) | ✅ **MET (build + measured).** M-301 compiles+runs the bit/trit kernel subset to a native artifact; M-302 passes on the kernel corpus **and** catches a deliberately divergent lowering (the discrimination test — guard 7); M-303's E1 is **measured** (native-vs-interp, honestly captioned), and compute throughput is now genuinely measured over runtime data (M-360 §3 packed-ternary kernels ≈1.13–1.67× vs scalar; M-340 §4 weight-specialization ≈10.7×) — no "within an agreed target" claim. | **yes — satisfied** |
| **Matured toolchain** (M-310/311/312) | ✅ **MET — document sync unblocked (R1 closed 2026-06-15).** M-311 (build-system gate + content-addressed `BuildCertificate`, forge-proof deserialize) and M-312 (request-addressed `BuildCache`, demonstrably reuses a prior cert; a weakened obligation misses) ship with tests — **met**. M-310 ships structured diagnostics (`FeedbackSummary`) + the LSP **wire protocol** (JSON-RPC framing, `publishDiagnostics`, lifecycle). The **text→`Node` blocker is now cleared**: RFC-0011 r3 is enacted, so the parser→checker→**elaborate→L0** pipeline reaches data/matching programs (the differential ties L1-eval ≡ L0-interp) — the path real document sync rides on now exists. (The `didOpen`/`didChange` wiring itself is the remaining M-310 build task, no longer a *gate* residual.) | **no — R1 closed** |
| **L1 surface** (M-320 + RFC-0007) | ✅ **MET — enacted (R2 ratified, then enacted 2026-06-15).** M-320 lands in `mycelium-l1` with tests (literal + nested `match`, Maranget usefulness with witnesses, the decision-tree compiler), and its remaining half — **emitting the tree's leaves as L0 kernel nodes** — is now done: `elab` lowers nested patterns to nested flat L0 `Match`/`Construct` (RFC-0011 r3). RFC-0006/0007 are **Accepted (r4)** and RFC-0011 is **r3 ENACTED** (RFC-0001 → r3). Concrete syntax stays KC-2-gated (unchanged). | **no — satisfied** |

**Overall verdict (honest, VR-5): both gate residuals are now closed — the gate is met pending a
maintainer re-assertion.** The keystone native-execution-path condition is fully met and measured;
the matured-toolchain and L1-surface conditions are now also met, both prior residuals discharged:

- **R1 — M-310 document sync → CLOSED (2026-06-15).** The text→`Node` blocker was the RFC-0011
  enactment; it is done — `Construct`/`Match` are L0 nodes and the parser→checker→**elaborate→L0**
  pipeline reaches data/matching programs (the M-210 differential ties L1-eval ≡ L0-interp). The
  remaining `didOpen`/`didChange` LSP wiring is an ordinary M-310 build task, not a gate residual.
- **R2 — RFC-0006 + RFC-0007 completed and ratified → DONE (2026-06-15).** Both Accepted (r4); RFC-0011
  is **r3 ENACTED** and RFC-0001 is **r3**.

The build order is now complete: **M-360 SIMD + true-TL2 → ratify RFC-0006/0007 → enact RFC-0011 r3 +
M-320 wiring (closed R1) → re-assert the gate.** With R1 + R2 closed, the §6 exit gate is
**RE-ASSERTED MET (2026-06-15)** — the doc status moves `Living draft → exit-gate met`; nothing here
upgrades a verdict beyond the checked run that established it (497 workspace tests; the data-fragment
differential). The phase's remaining build tasks (M-310 `didOpen`/`didChange` sync, M-350/M-360 local
items) continue *past* the gate; they no longer block it. The standing core-language continuation is
**RFC-0001 r4** (`Lam/App/Fix` into L0 — full L1-in-Core-IR, retiring RFC-0007 §4.6 entirely).

> The §6 gate is itself a **proposed** scope decision (what counts as "Phase-3 done" given two
> external blockers). It is recorded here for the maintainer to ratify or adjust; §6.1 fills it at the
> established strength but does **not** assert the phase closed — two named residuals remain.

---

## 7. Risks & open questions

| Id | Item | Disposition |
|---|---|---|
| **RR-11** | Toolchain + multi-backend scope balloons. | Phase-3 mitigation: reuse LLVM (M-301 emits textual IR, no hand-rolled codegen); the native backend stays in `mycelium-mlir`; build-system/JIT are separate crates (KC-3). Batches serialize on real deps so scope stays auditable. |
| **RR-12** | Interpreter↔AOT/JIT semantic divergence. | The M-151/M-251 differential already folds into the M-210 checker. M-302 extends it to the **real compiled artifact** (not just the env-machine), and M-340 (JIT) validates through the same machinery (NFR-7). |
| **RR-N1** *(new)* | The **direct-LLVM AOT** path diverges from the ratified **MLIR→LLVM** decision (RFC-0004 §2). | **Sequencing decision (§1):** direct-LLVM first (libMLIR absent), MLIR deferred; the textual `ternary`-dialect emitter is retained as the MLIR skeleton. If direct-LLVM becomes load-bearing rather than a stopgap, ratify an RFC-0004 erratum / short ADR (maintainer). Flagged, not silently adopted. |
| **KC-2 / RR-3** | LLM leverage on the Mycelium surface. | **Verdict recorded (DN-09) = proceed** (2026-06-18). Kill criterion not triggered — feedback recovers leverage; the surface is machine-authorable at this maturity. E3-1/M-380 moves to design-active; concrete L3 text syntax committed (RFC-0006 r5). The embedded-DSL fallback (RR-3) is retained **unspent** as the documented contingency. The T3.6 rigorous ablation (retention ratio vs familiar-skin baseline) was **not run** — honest open follow-up (DN-09 §4); "weak-but-recoverable" is the established strength (VR-5). |
| **G4 / RR-5** | Resonator factorization may not converge. | M-350 stays **opt-in, probabilistic-only, never in core** (FR-C2; the M-260 manifest already enforces the `Resonator` probabilistic-only ceiling in the type). needs-design: document the convergence regime + bounds before building. |
| **G11** | Semantic-level (not notational) projections may not be ergonomically viable. | E3-1/M-380 is **exploratory** — **KC-2 cleared (DN-09)**; now **design-active** in Batch M. The content-addressed IR + EXPLAIN dumps already give a projection substrate (Stage-D roadmap §"Mycelium-lang forward"). The ergonomic-viability question remains open, addressed through the M-380 design. |

---

## 8. How this doc stays honest

- **Append-only with status transitions**, mirroring the ADR/RFC discipline: this file moves
  `Living draft → exit-gate met` only when the §6 gate is met; task rows update in place as their
  issues progress, but gate verdicts (§5) and the E1 perf number never pre-record an upgrade.
- **The `M-3xx` tasks are bootstrapped onto the board** (#86–#98, 2026-06-15) from `issues.yaml`,
  linked under the E3-1…E3-7 epics, with `idmap.tsv` carrying the join (M-301→#86 … M-380→#98); shipped
  tasks are closed as completed. The Phase-2 epics/tasks (#28–34, #58–65) are closed as completed at the
  same sync.
- **External blockers are named, not hidden** — KC-2 has its verdict (M-002 closed, DN-09 = proceed,
  2026-06-18); the remaining external blocker is the MLIR path (needs libMLIR, §1). KC-2's T3.6
  ablation remains an honest open follow-up (DN-09 §4), tracked separately.

---

## 9. Per-task detail (filled as tasks land)

### 9.1 M-301 — Direct-LLVM-IR AOT backend (bit/trit subset) · Batch J · P1 · done 2026-06-15

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
- **Delivered (trit slice — `neg` + carry arithmetic).** The backend is **kind-aware** (a `Lane` is
  `Binary{w}` or `Ternary{m}`). `trit.neg` is digit-wise (`0 - x`). `trit.add` lowers to a fixed-width
  **ripple-carry** over the trits (LSB→MSB): with `x = aᵢ + bᵢ + carry + 4` (always ≥ 1, so the LLVM
  `srem`/`sdiv` coincide with euclidean rem/div by 3), the balanced digit is `x srem 3 − 1` and the
  next carry is `x sdiv 3 − 1` — mirroring `mycelium_core::ternary::add` digit-for-digit. `trit.sub`
  is `add(a, neg b)`; `trit.mul` is **shifted accumulation** into a 2m-trit buffer (each `b` digit
  scales `a` by an `i32 mul`, the digit being ±1/0), keeping the low `m` trits. **Fixed-width overflow
  is computed at runtime** — a non-zero final carry (add/sub) or any non-zero product high trit (mul)
  sets an `i1` flag (folded across the program). The **read-back protocol** is extended to carry it:
  on overflow the AOT artifact prints the `'!'` sentinel line and the JIT kernel (now
  `i32 @myc_kernel(ptr)`) returns a non-zero status, both surfaced as an explicit `AotError::Overflow`
  — never a silent wrap (SC-3/G2), matching the interpreter's `EvalError::Overflow`.
- **Tests (trit slice).** `trit_add_emits_ripple_carry_ir` (srem/sdiv + overflow branch + sentinel);
  arithmetic determinism; width/kind refusals; oracle round-trips for add (`5+4=9`), sub (`9−4=5`),
  mul (`2×3=6`) on both AOT and JIT; and explicit-overflow tests on both paths (`4+4`, `4×4` in 2
  trits). The M-302/M-340 differential corpora gain in-range arithmetic + nested `(5+4)−4`, and an
  overflow-parity test asserts interpreter and native **both** refuse the same out-of-range sum.
- **Honesty / scope.** The MLIR `ternary`-dialect lowering stays the **eventual** path (`dialect::emit`
  is its dumpable skeleton) and is **deferred** until libMLIR exists (RR-N1). No guarantee is
  upgraded: the reconstructed `Value` is `Exact` only because the bit/trit ops are exact and the
  subset refuses approximate inputs; an out-of-range arithmetic result is an explicit overflow, not a
  fabricated value (VR-5/G2).

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
- **Honesty / scope.** Now covers the full bit/trit subset M-301 lowers — bit logic, `trit.neg`, and
  the `trit.add/sub/mul` carry arithmetic (in-range cases + a nested `(5+4)−4`), plus an
  overflow-parity test asserting interpreter and native **both** refuse the same out-of-range sum
  (`AotError::Overflow` ↔ `EvalError::Overflow`). The env-machine M-151 differential is unchanged and
  still covers the wider corpus (swaps) — M-302 *adds* the compiled-artifact path, it does not
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

### 9.4 M-320 — L1 term-language extension: literal-pattern `match` · Batch K · P1 · in progress 2026-06-15

- **Goal (from §2 / issues.yaml).** Extend the `mycelium-l1` prototype's term language per RFC-0007
  §§3–4, with tests; RFC ratification is the maintainer's (append-only), concrete syntax KC-2-gated.
- **Delivered (first increment).** `match` now covers `Binary{n}`/`Ternary{m}` scrutinees with
  **literal patterns**, not just data types (the explicitly-deferred v0 gap at `checkty`/`eval`).
  `checkty::infer_literal_match` types a literal match: each literal arm must have exactly the
  scrutinee's repr+width (`lit_ty`), duplicate literals and arms-after-default are redundancy errors,
  and — because the 2ⁿ/3ᵐ domain is **not enumerated** — a literal match **requires** a `_`/binder
  default; without one it is non-exhaustive and refused (W7 — coverage never assumed).
  `eval::eval_literal_match` fires a literal arm on `repr + payload` equality (reusing
  `elab::lit_value` as the one literal interpretation) and binds the scrutinee on a binder default.
  Elaboration is unchanged: the whole `Match` family already lowers to `Residual` (L0 Core IR has no
  match node), so literal match is evaluable + type-checked but not yet L0-elaborable — consistent,
  not a new asymmetry. Five tests (`eval::tests`): arm selection, default fall-through, and three
  mutant-witnessed refusals (non-exhaustive, duplicate literal, width mismatch).
- **Honesty / scope.** No guarantee upgraded; the parser and totality checker already handled literal
  patterns (this unlocks the typechecker + evaluator). Nested patterns / the Maranget compiler and
  full L1-in-Core-IR remain ahead. **RFC-0007 ratification is presented, not flipped:** this increment
  exercises more of §4.4/§4.5 in the non-normative prototype; moving RFC-0006/0007 `Draft → Accepted`
  stays the maintainer's append-only decision, and concrete-syntax ratification stays KC-2-gated.

### 9.5 M-311 — Build-system: stable/experimental gate + certificate artifacts · Batch K · P1 · done 2026-06-15

- **Goal (from §2 / issues.yaml).** A build-system layer distinguishing **stable** components
  (content-addressed + spec-ratified + verification-passed → AOT-eligible, RFC-0004 §4) from
  **experimental** (interpreted/JIT); emits a certificate artifact per build (ADR-003).
- **Delivered.** New crate `mycelium-build` (KC-3: outside the trusted kernel, depends only on
  `mycelium-core` for `ContentHash`). `check_eligibility` runs the automatic RFC-0004 §4 checks —
  (1) content-addressed identity is structural (a `ContentHash`), (2) spec ratified, (3) the three
  obligations discharged (swap certs / bound checks / interp↔AOT reference equivalence) — returning
  the **specific** blocking reasons (never a silent refusal). `decide(component, promote)` routes a
  component: **AOT only for an eligible, *explicitly promoted* one** ("marking-stable is deliberate",
  §4), everything else interpreted/JIT — and a `promote` request for an ineligible component is
  **refused** (route stays Interpreted, reasons recorded), never a silent AOT. It emits a
  `BuildCertificate`: an inspectable, **content-addressed** (`cert_ref`, BLAKE3 of the canonical
  serialization) record with **private fields** (guard 2) and a **re-validating `Deserialize`**
  (guard 3, `deny_unknown_fields`) — a hand-edited certificate claiming `Aot` without discharged
  obligations is rejected on the way in (the forge guard). Seven tests incl. the forged-AOT and
  unknown-field rejections (mutant-witnessed).
- **Honesty / scope.** The obligations are *recorded* facts discharged elsewhere (`mycelium-cert`,
  the tier-i checker, the M-302 differential) — this crate is the gate + artifact, not a re-prover.
  A JSON schema for the certificate (mirroring the M-260 manifest discipline) is a small follow-on.

### 9.6 M-312 — Content-addressed build cache · Batch K · P2 · done 2026-06-15

- **Goal (from §2 / issues.yaml).** Cache build outputs + certificates keyed by content hash
  (ADR-003); a re-build of an unchanged definition reuses the cached certificate.
- **Delivered.** `mycelium-build::cache`: `BuildCache` maps a **build-request content address** to
  the emitted `BuildCertificate`. The key (`request_key`) folds the component's identity hash with
  *every* decision input — spec ratification, the three obligations, and the `promote` flag — so an
  identical request is a `Hit` (reuses the prior certificate verbatim) and **any** change in
  verification state is a `Miss` that re-decides, never a stale hit (G2). `build(component, promote)`
  returns a `CacheOutcome::{Hit, Miss}`. Three tests: the unchanged-second-build hit (the M-312
  acceptance), a weakened-obligation miss whose re-decided certificate flips `Aot → Interpreted`
  (mutant-witness: a key ignoring obligations would return a stale AOT cert), and a `promote`-flip
  miss.
- **Honesty / scope.** The cache addresses the *request*, not just the definition, precisely because
  the build decision depends on verification state that is not part of the code hash — so "unchanged
  definition" only hits when its obligations are also unchanged. **Batch K's gate items (M-310 LSP
  pending, M-311 + M-312 done) advance the matured-toolchain exit condition.**

### 9.7 M-310 — Full-LSP maturation: structured summary + navigable positions · Batch K · P1 · in progress 2026-06-15

- **Goal (from §2 / issues.yaml).** Mature the M-140 LSP skeleton: rich diagnostics over the
  existing artifact kinds with structured positions and severity levels (§5.6–5.8; SC-5).
- **Delivered (first increment).** `mycelium-lsp::FeedbackSummary` (via `Feedback::summary()`): a
  structured roll-up of the analysis — per-artifact-kind counts (guarantees / swaps / stages /
  explanations), the `Error`/`Warning` diagnostic breakdown, the **worst** severity present, and an
  `is_clean()` predicate. This is the at-a-glance health signal an AI co-author's feedback loop
  (SC-5b / E3-2, whose *run* is LLM-blocked) or an IDE status line consumes without re-walking the
  channels. Plus `Diagnostic::path()`: the `at` breadcrumb as a structured, navigable `Vec<&str>`
  (empty at the root, never `[""]`). Two tests (summary roll-up incl. the worst-severity
  mutant-witness; the breadcrumb-path split).
- **Honesty / scope.** Severity stays `Error`/`Warning` (the existing lattice); the L0 Core IR has no
  source spans, so "structured positions" are the navigable breadcrumb path (source line/col live in
  the L1 surface, a later step).
- **Delivered (second increment — LSP wire protocol).** `mycelium-lsp::wire`: the byte-level JSON-RPC
  codec (`read_message`/`write_message` with `Content-Length` header framing — clean EOF vs an
  explicit error on a truncated frame, never a silent drop), the `Diagnostic` → LSP-`Diagnostic`
  mapping with the spec `DiagnosticSeverity` codes (Error→1, Warning→2) and the
  `textDocument/publishDiagnostics` notification builder, and a minimal [`serve`] lifecycle loop
  (`initialize` → capabilities + `serverInfo`, `shutdown` → null, `exit` → stop; any other request →
  JSON-RPC `MethodNotFound`, never silence). New dep: the workspace-pinned `serde_json`. Seven tests
  (framing round-trip + many, clean-EOF, truncated-body error, severity mapping, `publishDiagnostics`
  shape, the scripted-client lifecycle, the unknown-request refusal). **Honest scope (VR-5):** not a
  document-syncing server — the facade analyzes Core IR `Node`s, not text, so the server advertises
  `TextDocumentSyncKind.None` and the diagnostic `range` is a **zero placeholder** with the navigable
  location in `data.breadcrumb`. Real spans + `didOpen`/`didChange` sync arrive with the L1 surface
  (M-320); the wire layer carries them without a protocol change.

### 9.8 M-360 — BitNet packed-ternary acceleration (first increment) · Batch L · P2 · in progress 2026-06-15

- **Goal (from §2 / issues.yaml).** BitNet-class packed-ternary acceleration (I2_S/TL1/TL2) exposed as
  inspectable metadata, not hidden lowering (FR-C3 / NFR-4 / G3); the runtime-input kernel the E1
  compute-throughput verdict needs (RFC-0004 §5/§8).
- **Delivered (I2_S dot kernel).** `mycelium-mlir::bitnet` emits the canonical BitNet **ternary
  multiply-accumulate** — `y = Σ digit(wᵢ)·xᵢ`, weights ternary, activations integer — as **textual,
  inspectable LLVM IR** (`i64 @myc_bitnet_dot(ptr %w, ptr %x, i64 %n)`: load the packed I2_S byte,
  extract the 2-bit code at lane `i&3`, signed weight `code−1`, load the activation, multiply-add into
  an `i64`; one transparent op per loop-body step — RFC-0004 §6). It is JIT-compiled (`clang -shared
  -O2`) and called **in-process** via the M-340 dynamic loader (refactored into a reusable
  `dlopen_path`/`Lib::sym`), over weight/activation buffers passed as **runtime pointers**. Bounds are
  checked against `n` (≥ `n.div_ceil(4)` weight bytes, ≥ `n` activations) so the native loads are
  always in-range — a short buffer is an explicit `AotError`, never an OOB read.
- **Why it closes the open E1 item.** The M-301/M-303/M-340 kernels bake inputs in as constants, so
  `clang` constant-folds the compute and the measured time is call/spawn overhead (honestly captioned,
  never claimed as throughput). Here the buffers are *arguments* — the optimiser cannot fold them — so
  `cargo xtask e1` **§3** times **genuine packed-ternary compute** over `n = 4096` runtime elements,
  against a hand-written Rust scalar baseline doing the *identical* I2_S unpack-compute (apples to
  apples). The §2 constant-fold/spawn caveat is resolved; the verdict reports the measured number.
- **Tests (`bitnet::tests`).** IR inspectability + determinism; the semantic oracle pinned on a
  hand-computed dot; `jit_dot_matches_reference` over `n ∈ {1,4,5,7,64,256,1000}` (mutant-witness: a
  wrong shift/mask or `code` vs `code−1` diverges); compile-once/call-many consistency; and short-buffer
  refusals (mutant-witness: dropping the bounds checks would read OOB). All toolchain-gated skips.
- **Honesty / scope.** **All three** bitnet packings now have a kernel — I2_S (rot=0), TL1 (rot=2
  inverted via `d01 = (code+1) mod 3`), and TL2 (base-3, 5 trits/byte, `digit = (byte / 3ᵖ) mod 3`
  via a select-chain `3ᵖ` lookup) — each a **scalar** loop with the unpack inlined and visible in the
  emitted IR (`emit_bitnet_dot_ir_for(scheme)`; a non-bitnet scheme is an explicit
  `AotError::UnsupportedScheme`, never a silent misdecode). Each is differential-checked against the
  packing-independent oracle `ternary_dot_ref` over the *same* `pack_trits` packing, so the in-IR
  unpack is verified. The kernel's weight-buffer bound tracks the scheme density (`n.div_ceil(4)` for
  I2_S/TL1, `/5` for TL2). **Not** claimed: parity with bitnet.cpp's hand-tuned **SIMD** — the next
  M-360 increment. No guarantee upgraded; the E1 number is whatever was measured (VR-5 / G3). The
  `unsafe` fn-pointer call carries a `// SAFETY:` justification under ADR-014 (the bounds checks
  discharge the in-range obligation).
- **Delivered (hand-vectorized SIMD — I2_S).** New `mycelium-mlir::simd`: a **hand-vectorized** I2_S
  dot kernel (`i64 @myc_bitnet_dot_simd(ptr %w, ptr %x, i64 %n)`) that unpacks + multiply-accumulates
  **8 trits/iteration** with LLVM vector types — broadcast the two packed bytes across 8 lanes
  (`shufflevector` mask `<0,0,0,0,1,1,1,1>`), bring each lane's 2-bit code to bit 0 (`lshr` by the
  constant vector `<0,2,4,6,0,2,4,6>`), mask `& 3`, `− 1` to the signed weight, `mul <8 x i32>` with
  the contiguous activations, widen, accumulate into an `<8 x i64>` phi, then horizontally reduce
  (`@llvm.vector.reduce.add.v8i64`) and a **scalar epilogue** finishes the `n mod 8` tail. Every vector
  op is visible in the emitted IR (no opaque pass — FR-C3 / RFC-0004 §6); the vector loads carry
  explicit `align 1`/`align 4` so a sub-vector-aligned offset is a legal unaligned load. It reuses
  `BitnetDotKernel`'s bounds-checked `call` (same C signature + I2_S density model — a
  `pub(crate) from_loaded` ctor; DRY), so a short buffer is still an explicit refusal. **The vector
  unpack is correctness-critical, so it is differential-checked**: `tests/simd_differential.rs` runs
  the SIMD kernel against the **scalar kernel as the oracle** over a corpus bracketing the 8-lane width
  and the tail (n ∈ {0,1,7,8,9,15,16,17,31,33,64,255,256,257,1000}) and validates each pair **through
  the shared M-210 checker** (`ObservationalEquiv`/`Exact`), with a mismatched-buffer discrimination
  test (guard 7). `cargo xtask e1` **§5** times SIMD vs scalar over the same runtime buffer (indicative
  ≈1.2× — honest: clang already auto-vectorizes the scalar loop at `-O2`, so the hand-vectorized gain
  is real-but-modest; as-measured, no target pre-written). **Scope (VR-5/G3):** **I2_S only** this
  increment; TL1/TL2 vectorized unpacks are next. No parity with bitnet.cpp's AVX2/AVX512 LUT kernels
  is claimed; same exact dot product, no guarantee upgraded; the scalar kernels stay the oracle.
- **Delivered (true 1.67-b/w TL2 layout — closes A5-08).** `mycelium-mlir::pack` now realizes `TL2` as
  the **true bitnet.cpp layout**: 3 trits → a **5-bit LUT-index** code (`c = d₀ + 3·d₁ + 9·d₂ ∈
  [0,27)`), bit-packed as a contiguous 5-bit-field stream ⇒ **1.67 b/w** (`5/3`). This is *less* dense
  than the `FiveTritPerByte` base-3 reference (1.6 b/w) **on purpose** — the 5-bit index is directly
  LUT-addressable (the "TL" = ternary lookup), the trade bitnet.cpp makes for fast decode. The prior
  placeholder (which packed TL2 identically to `FiveTritPerByte` at 1.6 b/w) is retired, so the two are
  now genuinely distinct densities; `needed_bytes(scheme, count)` is the new shared bound model
  (`⌈5·⌈count/3⌉/8⌉` for TL2). The native TL2 **dot kernel** (`bitnet`) decodes the bitstream inline —
  group `g = i/3`, position `p = i%3`, 5-bit code at bit offset `5g`, `digit = (code / 3ᵖ) mod 3` — with
  a **branch-free bounds-clamped 2-byte window** (the second byte index is clamped to `needed − 1`,
  computed from `n`, so the final group's read never goes OOB even when its field fits in one byte; the
  spilled high bits are masked off by `& 31`). It is **oracle-checked across widths** (the existing
  `jit_dot_matches_reference_all_schemes` over n up to 1000 now exercises the new layout) and the
  bound is a `tl2_uses_the_true_167_bitstream_bound` refusal test. The selector cost model
  (`packing_bits_per_element(Tl2) = 1.67` in `mycelium-select`) now **matches** the codec — **A5-08 is
  resolved**; the notes in `pack.rs` and `mycelium-select` are updated from "stand-in / inert
  discrepancy" to "resolved". E1 §3 times the true TL2 kernel over runtime data (≈1.25× vs the scalar
  baseline — honestly *slower per-element* than I2_S, since the bitstream decode is more work than a
  byte-aligned 2-bit extract; as-measured). New `pack` property tests pin the 1.67 b/w density
  (`tl2_realizes_the_true_167_bits_per_weight`) and the TL2≠`FiveTritPerByte` layout distinctness.
  **Honest scope (VR-5):** this realizes the bitnet.cpp TL2 *density and 5-bit-LUT-index semantics*;
  the exact upstream byte/bit ordering of bitnet.cpp's internal buffer is **not** claimed byte-identical
  (verifying that needs the upstream source) — the codec is self-consistent (round-trip identity) and
  oracle-checked, which is what the value semantics + the differential require.

### 9.9 M-320 — L1 nested patterns + Maranget usefulness · Batch K · P1 · in progress 2026-06-15

- **Goal (from §2 / RFC-0007 §4.4/§4.7).** Lift the flat-match restriction so L1 `match` supports
  **nested** constructor/literal patterns, with exhaustiveness and redundancy *checked* (W7) — the
  L1 doc's named big item.
- **Delivered.** New `mycelium-l1::usefulness`: the Maranget usefulness algorithm `U(P, q)` over a
  typed pattern matrix (Maranget 2007), returning a **witness** when useful. Two derived checks drive
  the typechecker — **exhaustiveness** (a `_` must not be useful; its witness is a concrete missing
  case, e.g. `S(Z)`, reported verbatim) and **redundancy** (an arm covered by the earlier rows is
  unreachable; this subsumes the M-320 duplicate-literal check). `checkty` gained a recursive,
  type-directed `check_pattern` (nested ctor/literal patterns, binders typed by field type, linearity)
  and a unified `infer_match` (data + `Binary`/`Ternary`, no more flat-only refusal). The evaluator's
  `try_match` matches nested patterns recursively (binders bound left-to-right; a partial nested
  failure simply falls through). The totality checker now seeds smallness from **nested** sub-binders
  of a smaller scrutinee (`S(S(m)) → m` descends), so structural recursion through nested patterns is
  admissible for `matured`.
- **Tests.** `usefulness` unit tests (flat/nested exhaustiveness, deep witness `S(S(_))`, redundancy,
  literal-needs-default); checker tests (nested typechecks, precise missing-witness `S(Z)`, nested
  redundancy, nested structural descent gates `matured`); evaluator end-to-end (`pred2` over depth-2
  `Nat` selects and binds correctly). All existing flat-match tests still pass.
- **Honesty / scope.** RFC-0007 is **Draft** and the prototype **non-normative**; this advances the
  surface checker + reference evaluator. The Maranget *usefulness analysis* (Maranget 2007) is the
  analysis half; the **decision-tree compilation** (Maranget 2008; RFC-0007 §3 — "compiled away by the
  elaborator") now lands too (next increment). Coverage stays *checked*, never assumed (W7); no
  guarantee is touched.
- **Delivered (decision-tree compiler — the codegen half).** New `mycelium-l1::decision`: the Maranget
  2008 compilation of a checked nested-pattern match into a flat `Tree` of `switch`/`leaf` nodes over
  **occurrences** (paths into the scrutinee) — the left-to-right column heuristic, constructor/literal
  specialization, and a `default` exactly when a column's signature is incomplete or its domain is open
  (`Binary`/`Ternary`). It is **verified**, not asserted: a test-only tree evaluator (`eval_tree` over
  concrete `Pat` values) is checked to agree with a reference matcher on every `Nat` value up to a
  depth (a wrong column choice or specialization would diverge), plus first-match-on-overlap and the
  literal-needs-a-default shape. It is **wired into `checkty`**: after exhaustiveness passes,
  `infer_match` compiles the match and confirms the tree is `has_reachable_fail`-free — an exhaustive
  match must compile to total coverage, so usefulness and the compiler must agree (defense in depth,
  never silent). **Scope (VR-5):** the tree is an internal analysis/IR artifact — its leaves are **not
  yet emitted as L0 kernel nodes** (L0 has no `Match`; that is the RFC-0001 revision, RFC-0007 §4.6),
  so it does not yet run programs. The compilation algorithm is real and checked; the L0 emission is
  the remaining step. No guarantee touched.

### 9.10 M-340 — JIT runtime specialization (weight-specialized dot kernel) · Batch L · P2 · in progress 2026-06-15

- **Goal (from §2 / issues.yaml #93).** Add a **runtime-specialization layer over the lowering** and
  record an honest speedup (no pre-written target), validated through the shared M-210 checker
  (NFR-7). The first M-340 increment landed the in-process `dlopen` JIT path; this adds the classic
  JIT win — specialize on data known *at JIT time*.
- **Delivered.** New `mycelium-mlir::specialize`: `emit_specialized_dot_ir(weights)` emits a
  **weight-specialized** ternary dot kernel `i64 @myc_bitnet_dot_spec(ptr %x)` with the (runtime-known)
  weight vector **baked in as constants**. Because the weights are compile-time-constant *in the JIT'd
  kernel*, the optimiser (a) **drops the unpack entirely** (no packed-byte load / shift / mask /
  `code−1`), (b) **elides every zero-weight lane** — a `0` weight's activation load + multiply simply
  do not appear in the emitted IR (the model's sparsity becomes inspectable, FR-C3), and (c)
  **strength-reduces the ±1 multiply** to a single `add`/`sub`. The only runtime argument is the
  activation pointer; the length and weights are compiled in. `compile_specialized_dot` JIT-compiles
  it (`clang -shared -O2`) and loads it via the M-340 dynamic loader; `SpecializedDotKernel::call`
  takes **no weight argument** (you cannot run it against weights it was not built for — misuse is
  unrepresentable, never a silent stale-weights run) and **bounds-checks** the activation buffer
  against the baked lane count (a short buffer is an explicit `AotError::Run`, never an OOB read).
  `nonzero()` exposes the surviving-lane count (EXPLAIN/inspection).
- **Validation (NFR-7).** `tests/specialize_differential.rs`: the specialized kernel and the generic
  runtime-pointer kernel (`compile_bitnet_dot_for`) are run over the same activations and validated
  as observationally equivalent **through the single shared M-210 checker**
  (`check(.., ObservationalEquiv, Certificate::exact(), Observational) ⇒ Validated{Exact}`) — the same
  checker the AOT/JIT differentials use — plus a discrimination test that specializes on *negated*
  weights and confirms the checker reports `NotValidated` (guard 7, so a pass is meaningful). Module
  tests pin the IR shape (zero lanes elided — mutant-witnessed against loading an elided lane), the
  all-zero → `ret i64 0` kernel, determinism, oracle agreement over several widths, and the
  short-buffer refusal.
- **Honest speedup (E1 §4 / VR-5).** `cargo xtask e1` gains **§4**: it times the specialized kernel
  vs the generic kernel over the *same* runtime activation buffer (both still take runtime activation
  pointers — no constant folding, so the compute is real), gated by an oracle cross-check before
  timing. Indicative single run (containerized x86-64, `n = 4096`, ~66 % dense weights): generic
  ≈ 3.9 µs/call, specialized ≈ 0.36 µs/call — **≈ 10.7× as measured**, driven by dropping the unpack,
  eliding the zero lanes, and `-O2` vectorization of the straight-line adds/subs. The number is
  reported **as-measured**; no target is pre-written, and the result is sparsity- and machine-dependent.
- **Honesty / scope.** No guarantee upgraded — the specialized kernel computes the *same exact* dot
  product (both `Exact` integer arithmetic). The weights are honest **runtime data** baked at JIT
  time (the inference shape: weights fixed, activations vary), not a constant-folded closed kernel.
  Shares the dot semantics + the M-340 loader with the generic kernel (DRY); the specialization
  applies to the BitNet dot kernel today and generalizes to other runtime-fixed operands later.

## Meta — changelog & maintenance

- **2026-06-18 (KC-2 verdict received — DN-09; M-002 closed; E3-1/M-380 design-active):** The
  M-002 run (local Qwen2.5-Coder, 10-task gold set, seed 42, edit-to-fix budget 3) delivered the
  **KC-2 verdict = proceed** (weak-but-recoverable leverage; best arm 7B+examples 40%→70% eventual,
  +30pp edit-to-fix; kill criterion not triggered — DN-09 §3). §2 M-002 row flipped to **Done —
  verdict recorded (DN-09)**. §2 M-380 row flipped from *KC-2-contingent / needs-design* to
  **design-active** (the projection layer is now a co-equal deliverable, FR-S5 / DN-09 §3.1).
  §5 KC-2 gate updated from *open — blocked (external)* to **verdict recorded (DN-09) = proceed**
  (at the established strength — the T3.6 retention-ratio ablation was **not run**; "proceed" does
  not claim the strong Q1 hypothesis, only non-collapse + recoverability — DN-09 §4). §7 KC-2/RR-3
  row updated accordingly. Embedded-DSL fallback (RR-3) retained unspent. Concrete L3 text syntax
  committed (RFC-0006 r5; DN-09 §3.1). Append-only; no gate verdict upgraded beyond the checked run.
- **2026-06-15 (M-360 true 1.67-b/w TL2 layout — closes A5-08):** `mycelium-mlir::pack` now realizes
  `TL2` as the **true bitnet.cpp layout** — 3 trits → a 5-bit LUT-index (`c = d₀+3d₁+9d₂ ∈ [0,27)`),
  bit-packed as a contiguous 5-bit-field stream ⇒ **1.67 b/w** (`5/3`), distinct from the
  `FiveTritPerByte` 1.6-b/w base-3 reference (the prior placeholder that made them coincide is retired).
  `needed_bytes(scheme, count)` is the new shared bound model. The native TL2 **dot kernel**
  (`bitnet`) decodes the bitstream with a **bounds-clamped 2-byte window** (the second byte index is
  branch-free-clamped to the last valid byte, so the final group's read never goes OOB), oracle-checked
  across widths (`jit_dot_matches_reference_all_schemes`). The selector cost model
  (`packing_bits_per_element(Tl2) = 1.67`) now **matches** the codec — **A5-08 resolved** (the note in
  `pack.rs` + `mycelium-select` updated). E1 §3 times the true TL2 kernel (measured ≈1.25× vs scalar,
  honestly slower per-elem than I2_S — the bitstream decode is more work, as expected). New property
  tests pin the 1.67 b/w density and TL2≠FiveTritPerByte distinctness. **Honest scope (VR-5):** realizes
  the bitnet.cpp TL2 *density + 5-bit-LUT-index semantics*; exact upstream byte/bit ordering not claimed
  byte-identical (needs the source to verify) — codec self-consistent + oracle-checked. §2 M-360 row +
  §9.8 updated.
- **2026-06-15 (M-360 hand-vectorized SIMD — I2_S, first SIMD increment):** new `mycelium-mlir::simd`
  emits an 8-wide hand-vectorized I2_S dot kernel (vector-IR unpack: `shufflevector` byte-broadcast +
  per-lane `lshr` + `mul <8 x i32>` + `@llvm.vector.reduce.add.v8i64`, scalar tail for `n mod 8`),
  reusing `BitnetDotKernel`'s bounds-checked `call` (`from_loaded` ctor, DRY). The correctness-critical
  vector unpack is differential-checked against the **scalar kernel as the oracle** through the shared
  M-210 checker over a width/tail corpus (`tests/simd_differential.rs`), with a discrimination test
  (guard 7). `cargo xtask e1` §5 times SIMD vs scalar (≈1.2×, as-measured; clang already
  auto-vectorizes the scalar `-O2` loop). §2 M-360 row + §9.8 updated. **Scope:** I2_S only; TL1/TL2
  SIMD + the true 1.67-b/w TL2 layout (A5-08) next; no bitnet.cpp AVX parity claimed; same exact dot,
  no guarantee upgraded (VR-5/G3).
- **2026-06-15 (Phase-3 exit-gate assembly — §6.1 verdict fill, VR-5):** re-read §6 against the landed
  work and filled each gate condition at the strength a checked run established. **Native execution
  path** ✅ met + measured (M-301/302/303; compute throughput now real via M-360 §3 + M-340 §4).
  **Matured toolchain** 🟡 met except **document sync** (M-311/312 done; M-310 diagnostics+wire done) —
  residual **R1**, blocked on the RFC-0011 text→`Node` enactment. **L1 surface** 🟡 met in prototype
  (M-320) with the L0-`Match` revision **ratified** (RFC-0011 staged r3); **RFC-0006/0007 full
  ratification sequenced** (maintainer-directed) — residual **R2**. Overall: **substantially met, not
  yet closed**; doc stays *Living draft*. Close-out order: **M-360 SIMD + true-TL2 → ratify
  RFC-0006/0007 → enact RFC-0011 r3 + M-320 wiring (closes R1) → re-assert the gate.** No verdict
  upgraded beyond its checked basis.
- **2026-06-15 (keystone — RFC-0011 L0 `Match` / L1-in-Core-IR ratified as a decision; enactment
  sequenced):** authored `docs/rfcs/RFC-0011-L0-Match-and-L1-in-Core-IR.md` — the *normative* keystone
  both M-320's remaining half (emit decision-tree leaves as real L0 nodes) and M-310's document sync ride
  on — and the maintainer **ratified the staged path** (RFC-0001 r3 = data-and-matching core:
  `Construct`, flat `Match`, content-addressed registry, WF6/WF7/WF8; flat `Match` as the kernel node,
  Maranget tree untrusted; `Lam/App/Fix` deferred to r4). It is the **named RFC-0001 revision** (RFC-0006 §4.4 step 2 /
  RFC-0007 §9). **Decision recorded, not yet enacted:** RFC-0011 depends on RFC-0007, and the maintainer
  directed that **RFC-0006 + RFC-0007 be completed and ratified first**, so the RFC-0001 r2 → r3 text-fold,
  the RFC-0007 §4.6 narrowing, and the M-320 wiring land together as the core-lang step — **after** the
  Phase-3 exit-gate assembly → M-360 SIMD → RFC-0006/0007 ratification. RFC-0001 stays **r2/frozen** and
  the prototype keeps returning `Residual` until then (no grounding inversion). Registered in the RFC
  README + Doc-Index. M-320 row note points the remaining half at RFC-0011.
- **2026-06-15 (M-340 runtime specialization — weight-specialized dot kernel; honest ≈10.7× E1 §4
  speedup):** new `mycelium-mlir::specialize` bakes the runtime-known weight vector into the ternary
  dot kernel as constants (`i64 @myc_bitnet_dot_spec(ptr %x)`), so the optimiser drops the unpack,
  elides the zero lanes, and strength-reduces ±1 to add/sub. Validated generic↔specialized through the
  shared M-210 checker (`ObservationalEquiv`/`Exact`) with a negated-weights discrimination test
  (`tests/specialize_differential.rs`); `cargo xtask e1` §4 times specialized-vs-generic over the same
  runtime activations and reports the speedup **as-measured** (no target pre-written). The call site
  takes no weight argument (stale-weights misuse unrepresentable) and bounds-checks the activation
  buffer (no OOB). §2 M-340 row updated, §9.10 added. **Scope:** same exact dot product, no guarantee
  upgraded; weights are runtime data baked at JIT time, activations stay runtime pointers (VR-5/G3).
- **2026-06-15 (M-320 Maranget decision-tree compiler — the codegen half):** new
  `mycelium-l1::decision` compiles a checked nested-pattern match into a flat `switch`/`leaf` `Tree`
  over occurrences (Maranget 2008) — column heuristic, ctor/literal specialization, `default` only when
  a signature is incomplete or open. Verified by a test-only tree evaluator agreeing with the reference
  matcher over `Nat` values (first-match-on-overlap; literal-needs-default shape), and wired into
  `checkty::infer_match` as a `Fail`-free cross-check of exhaustiveness (usefulness and the compiler
  must agree, never silent). §2 M-320 row + §9.9 updated. **Scope:** the tree is an internal IR
  artifact — emitting its leaves as L0 kernel nodes awaits the RFC-0001 L0 revision (RFC-0007 §4.6); no
  guarantee touched (VR-5).
- **2026-06-15 (M-310 LSP wire protocol — stdio JSON-RPC + LSP-shaped diagnostics):** new
  `mycelium-lsp::wire` wraps the M-140 feedback facade in the LSP transport — `Content-Length`
  message framing (`read_message`/`write_message`; clean EOF vs explicit truncated-frame error), the
  `Diagnostic`→LSP-`Diagnostic` mapping (spec severity codes; zero-range placeholder + breadcrumb in
  `data` since L0 has no spans), the `textDocument/publishDiagnostics` notification builder, and a
  minimal `serve` lifecycle (`initialize`/`shutdown`/`exit`; other requests → `MethodNotFound`, never
  silent). Adds the workspace-pinned `serde_json`. 7 tests. §2 M-310 row + §9.7 updated. **Scope:**
  not a document-syncing server yet — text→`Node` sync needs the L1 surface (M-320); honestly
  advertised as `TextDocumentSyncKind.None` (VR-5).
- **2026-06-15 (M-360 follow-ups — E1 §3 all-three + A5-08 reconciliation):** `cargo xtask e1` §3
  now times **all three** bitnet packings in-process over runtime data (I2_S/TL1/TL2), each vs a
  hand-written scalar baseline doing the identical per-scheme unpack (measured: JIT beats scalar
  ≈1.69×/1.31×/1.15×; reported as-measured, VR-5). Re-exported `compile_bitnet_dot_for` /
  `emit_bitnet_dot_ir_for` / `jit_ternary_dot_for`. The **A5-08** notes in `mycelium-mlir::pack` and
  `mycelium-select` are refined to record that the scalar TL2 kernel decodes the 1.6-b/w *placeholder*
  codec — it does **not** resolve the published 1.67-b/w discrepancy (inert for selection); the true
  bitnet.cpp TL2 layout is tied to the **real-layout / SIMD** increment, kept flagged not silent.
- **2026-06-15 (M-360 TL1/TL2 kernels — full bitnet packing breadth):** `mycelium-mlir::bitnet`
  generalised from I2_S-only to `emit_bitnet_dot_ir_for(scheme)` covering **all three** bitnet
  packings — TL1 inverts the rot=2 LUT (`d01 = (code+1) mod 3`), TL2 decodes base-3 5-trits/byte
  (`digit = (byte / 3ᵖ) mod 3` via a select-chain divisor lookup) — each a scalar loop with the
  scheme-specific unpack inlined and inspectable; a non-bitnet scheme is an explicit
  `AotError::UnsupportedScheme`. `BitnetDotKernel` now carries its scheme so the weight-buffer bound
  tracks density (`/4` vs `/5`). All three are differential-checked against the packing-independent
  oracle (`jit_dot_matches_reference_all_schemes`, n up to 1000) — clang present, so the kernels
  actually ran and matched. §2 M-360 row + §9.8 updated; SIMD is the remaining increment. **Scope:**
  scalar only, no bitnet.cpp SIMD parity claimed (VR-5/G3).
- **2026-06-15 (board sync — Phase-2 closed, Phase-3 tasks bootstrapped):** synced the GitHub board to
  the corpus. Closed the completed **Phase-2** epics (E2-1…E2-7, #28–34) and tasks (M-230…M-260, #58–65)
  as *completed*, each with a grounding comment citing where it landed (CHANGELOG Batch G/H; Phase-2
  exit gate met 2026-06-12). Created the **Phase-3** M-3xx tasks from `issues.yaml` as **#86–#98**,
  linked as sub-issues under E3-1…E3-7 (#35–41); closed the six shipped ones
  (M-301/302/303/311/312/370) as completed, left the in-progress / needs-design / blocked ones open
  with status-annotated bodies. Updated `tools/github/idmap.tsv` (M-301→#86 … M-380→#98) and §2/§8
  above. Tracker hygiene only — no code or corpus-normative change.
- **2026-06-15 (M-350 needs-design — RFC-0009 resonator-network factorization drafted):** authored
  `docs/rfcs/RFC-0009-Resonator-Network-Factorization.md` — the *needs-design* deliverable for M-350
  (document the convergence regime + bounds **before** building, per RR-5/G4). Fixes: the iterative
  resonator update over `VsaModel` bind/unbind/cleanup (Frady et al. 2020); the **probabilistic-only**
  honesty contract — basis capped at `Empirical` (exact bind) / `Declared` (approximate), **never**
  `Proven`, with the regime `{F, kᵢ, d}` as a checked `EmpiricalProfile` side-condition (the
  `mycelium-core::recon` `Resonator` schema already enforces the ceiling, A6/FR-C2); never-silent
  termination (bounded budget; `BudgetExhausted`/`Oscillating` are explicit verdicts, never a wrapped
  answer); full reification/`EXPLAIN` of the run trace; and the open questions (init, cleanup shape,
  oscillation detection, δ-derivation, multiplicity, per-model scope). Prior art
  (`embeddenator-retrieval`/`-vsa`) flagged to mine, not copy. **No code; nothing in the kernel.** §2
  M-350 row → design-drafted; registered in the Doc-Index. Prototype gated on ratification (maintainer's).
- **2026-06-15 (M-320 nested patterns — Maranget usefulness; exhaustiveness/redundancy with
  witnesses):** L1 `match` now supports **nested** constructor/literal patterns. New
  `mycelium-l1::usefulness` implements Maranget's `U(P, q)` over a typed pattern matrix (witness-
  returning); the typechecker derives **exhaustiveness** (a `_` must not be useful — the witness names
  a concrete missing case like `S(Z)`) and **redundancy** (an arm covered by earlier rows is
  unreachable, subsuming the duplicate-literal check) from it. `check_pattern` checks nested patterns
  type-directed (binders typed by field type, linearity enforced); the evaluator's `try_match` matches
  nested patterns recursively; the totality checker seeds smallness from nested sub-binders so
  `S(S(m)) → m` descends (admits `matured`). §2 M-320 row updated, §9.9 added. **Scope/honesty:**
  RFC-0007 is Draft / prototype non-normative; this is the analysis half — Maranget *compilation* to
  the flat kernel `Match` (the elaborator/L0 path) lands with full L1-in-Core-IR. Coverage stays
  checked (W7), no guarantee touched.
- **2026-06-15 (M-360 first increment — BitNet I2_S runtime-data dot kernel; closes the open E1
  compute-throughput item):** new `mycelium-mlir::bitnet` emits the canonical BitNet ternary
  multiply-accumulate (`Σ digit(wᵢ)·xᵢ`) as inspectable LLVM IR (`i64 @myc_bitnet_dot(ptr,ptr,i64)`:
  load packed I2_S byte → extract 2-bit code → signed weight `code−1` → multiply-add), JIT-compiles it
  (`clang -shared -O2`), and calls it **in-process over runtime-pointer buffers** (M-340 loader,
  refactored into reusable `dlopen_path`/`Lib::sym`). Because the inputs are arguments, not baked-in
  constants, `cargo xtask e1` gains **§3** measuring **genuine packed-ternary compute** (n=4096) vs a
  hand-written Rust scalar baseline doing the identical unpack-compute — the runtime-input kernel that
  resolves §2's constant-fold/spawn caveat. Differential-checked against the Rust oracle over several
  widths; bounds-checked (short buffer → explicit `AotError`, never OOB). §2 M-360 row → in progress,
  §9.8 added, M-303 row note updated. **Scope/honesty:** I2_S + scalar only; no bitnet.cpp SIMD parity
  claimed, TL1/TL2 next; E1 number is measured, not pre-written (VR-5/G3).
- **2026-06-15 (M-301 trit slice — carry arithmetic `add/sub/mul`, M-301 done):** the direct-LLVM
  backend now lowers balanced-ternary **carry arithmetic** over `Ternary{m}`: `trit.add` as a
  fixed-width **ripple-carry** (LSB→MSB, balanced digit `x srem 3 − 1` / carry `x sdiv 3 − 1` with
  `x = aᵢ+bᵢ+carry+4 ≥ 1` so the LLVM `srem`/`sdiv` are euclidean), `trit.sub = add(a, neg b)`, and
  `trit.mul` as **shifted accumulation** in a 2m-trit buffer (each `b` digit scales `a` via `i32 mul`,
  the digit being ±1/0). Each mirrors `mycelium_core::ternary` digit-for-digit. **Fixed-width overflow
  is computed at runtime** (non-zero final carry, or non-zero product high trits) and signalled
  through an extended **read-back protocol** — an out-of-range result prints the `'!'` sentinel line
  (AOT) / returns a non-zero kernel status (JIT, now `i32 @myc_kernel`) and surfaces as an explicit
  `AotError::Overflow`, matching the interpreter's `EvalError::Overflow` (never a silent wrap, SC-3/G2).
  Both differential corpora (M-302 native, M-340 JIT) gain in-range add/sub/mul + nested arithmetic,
  and a new overflow-parity test asserts interp **and** native both refuse the same out-of-range sum.
  §2 M-301 row → **done**; §9.1 updated. This closes the last open slice of M-301.
- **2026-06-15 (M-370 native-ternary forward-compat map):** authored
  `docs/notes/Native-Ternary-Forward-Compat.md` — the ternary value-semantics contract (§1), the
  emulated-on-binary → native 3-state mapping with the `ternary` dialect (`dialect::emit`) as the
  **stub target** (§2), the R7 portability guarantee (§3), and the deferred native
  arithmetic/layout items (§4). Documentation + stub only; **no 3-state backend built** (ADR-005 /
  VR-5). §2 M-370 row → done; registered in the Doc-Index. **E3-7 (native-ternary forward-compat) is
  now complete at the documentation level** (M-150 + M-301 native path + M-370 map).
- **2026-06-15 (M-340 in-process JIT — first increment, uses ADR-014):** `mycelium-mlir::jit` emits
  the kernel as `void @myc_kernel(ptr)`, compiles it to a shared object (`clang -shared`), and calls
  it **in-process** via `dlopen`/`dlsym` — the **first intentional `unsafe` FFI under ADR-014**
  (justified `// SAFETY:` + `#[cfg_attr(not(debug_assertions), allow(unsafe_code))]`; no new
  dependency). It reuses the same `lower_program` + `emit_char_code`/`decode_result` as the AOT path,
  so it agrees with the interpreter through the shared M-210 checker (`tests/jit_differential.rs`,
  NFR-7). Removes the process-spawn overhead of the M-303 AOT path. **Honest E1:** the closed kernel
  constant-folds, so the in-process per-call time is call overhead, not compute — a calibrated
  throughput verdict still needs runtime-input kernels (M-360); not pre-written (VR-5). §2 M-340 row
  → in progress. Confirms ADR-014's incentive works: gate clippy (`-A unsafe_code`) clean, dev clippy
  emits the 4 unsafe warnings.
- **2026-06-15 (M-301 trit slice — `trit.neg`):** the direct-LLVM backend (`mycelium-mlir::llvm`) is
  now **kind-aware** (a `Lane` is `Binary{w}` or `Ternary{m}`) and lowers `trit.neg` over `Ternary{m}`
  end-to-end (digit-wise `0 - x`; ternary output via a branch-free `'-'`/`'0'`/`'+'` `select` chain;
  read-back to a `Ternary{m}` value) — compiled and differential-checked (two trit-`neg` programs
  added to the M-302 corpus). The parse shape is derived from the actual lowering (`lower_program` is
  the single source of truth). `trit.add/sub/mul` (balanced-ternary **carry** arithmetic) stay an
  explicit next-slice refusal; `bit.*`/`trit.*` on the wrong lane kind is a `require_kind` refusal.
  §2 M-301 row updated. (Enabled later JIT/FFI work via ADR-014, separately.)
- **2026-06-15 (M-310 first increment — structured feedback summary + navigable positions):**
  `Feedback::summary()` rolls up artifact-kind counts + worst severity (`FeedbackSummary`);
  `Diagnostic::path()` exposes the navigable breadcrumb. §2 M-310 row → in progress; §9.7 added.
  **Batch K's matured-toolchain trio (M-310 / M-311 / M-312) now all have substantive local
  progress** — the §6 matured-toolchain exit condition is materially advanced.
- **2026-06-15 (M-312 content-addressed build cache lands):** `mycelium-build::cache::BuildCache`
  caches certificates by build-request content address (component identity + decision inputs); an
  unchanged request hits and reuses the certificate, a changed verification state misses (never a
  stale hit). §2 M-312 row → done; §9.6 added.
- **2026-06-15 (M-311 build-system stable-component gate lands):** new `mycelium-build` crate makes
  the RFC-0004 §4 stable/experimental gate executable + inspectable (`check_eligibility` / `decide` /
  content-addressed `BuildCertificate` with a forge-proof validating deserialize). §2 M-311 row →
  done; §9.5 added. Next: M-312 content-addressed build cache.
- **2026-06-15 (M-320 first increment — literal-pattern `match`):** the L1 prototype's `match` now
  covers `Binary`/`Ternary` literal patterns with a mandatory default (W7), redundancy/width checks,
  and `repr+payload` arm selection (`checkty` + `eval`; elaboration inherits `Match ⇒ Residual`). §2
  M-320 row → in progress; §9.4 added. RFC-0007 ratification presented, not flipped (maintainer's).
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
