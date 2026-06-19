# RFC-0004 — Execution Model, Backends & "Stable Component"

| Field | Value |
|---|---|
| **RFC** | 0004 |
| **Status** | **Accepted** (r2 — adds **§9** the *interpreted↔compiled continuum* + **build-target profiles** (`interpret` / `build --slim` / `build --target …` / `build --fat`) and **§10** open questions; **additive**, changes no r1 decision — append-only) (r3 — adds **§11** scoping the direct-LLVM backend's data-fragment extension as a sanctioned increment of the §2 revisit clause; **additive**, changes no prior decision — append-only) |
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
- **Stack-robust recursion is a normative requirement of this path (added 2026-06-16; DN-05 #1, M-347/M-348).** The native backend **must** execute object-level recursion **without an unbounded C stack** — a managed/segmented or heap-spilled call stack with an **explicit depth/budget limit** (a graceful error, never a SIGSEGV/abort; G2). This is *designed in, not retrofitted*: the reference interpreter is already O(1)-host-stack, and the AOT env-machine was made so via a trampoline (DN-05 #2, enacted M-347) — the native path inherits the same guarantee. Provisioning libMLIR to build this path is M-348 (near-term; desktop/WSL).

## 3. Single certificate checker (T1.1) — shared with RFC-0002
One refinement/equivalence-certificate checker `(A, B, R, bound, certificate)` serves **both** representation-swap validation (RFC-0002) **and** interpreter-vs-compiled equivalence. Interpreter-vs-compiled uses R = observational (or bounded) equivalence. Build once, use twice.

**Equivalence assurance is graded:**
- **Differential testing** (run interpreter + compiled, compare) as the cheap baseline — catches RR-12 divergence broadly.
- **Per-artifact translation validation** for **stable components** (the artifacts that matter), via the §3 checker.
- **Full verified compilation** (CompCert-level) is **out of scope** (KC-4 cost).

## 4. "Stable component" gate — normative
A definition is a *stable component*, and thus **AOT-eligible**, iff: (1) content-addressed and hash-frozen (Unison identity, ADR-003); (2) its spec is ratified; (3) its verification obligations (swap certificates, bound checks, reference equivalence) are discharged. **Promotion is an explicit act gated on automatic checks** (CI step): the checks must pass, but marking-stable is deliberate. Everything else runs interpreted/JIT.

> **Note (2026-06-18 — append-only; RFC-0017 Accepted):** Maturation is now declared at **scope**
> granularity (nodule/phylum header `// @matured: true`; program/package via `mycelium-proj.toml`
> manifest), not per-definition — **RFC-0017** supersedes RFC-0007 §4.5's *granularity* (the
> per-`matured fn` framing). The **stable-component eligibility checks in §4 above are unchanged**
> — they are applied per reachable definition in a matured scope. A scope is well-formed for
> maturation iff every reachable non-`thaw` definition satisfies both the `total` gate (RFC-0007
> §4.5) and the §4 AOT-eligibility checks (content-addressed + hash-frozen, spec-ratified,
> verification discharged). `thaw fn f` exempts one definition from the matured set; the remaining
> definitions' obligations are unaffected. The §4 gate is the *per-definition* obligation; RFC-0017
> §4.2 is the *scope-level* conjunction of those same obligations. No AOT guarantee is changed.

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

## 9. The interpreted↔compiled continuum & build-target profiles (r2) — normative

This section makes explicit the *developer-facing* shape of §2/§4: a program is not "interpreted" **or** "compiled" — it lives on a **continuum**, per definition, and the developer chooses how much to compile and for which targets. The goal (maintainer direction, 2026-06-15): **interpret freely during development at a perf cost; compile what is ready; never be forced into a heavyweight build; and never recompile what has not changed.**

### 9.1 The continuum (restates §2/§4; no new decision)
- **The interpreter is always available and is the meaning** (§2; ADR-009): a definition runs with **zero build step**. This is the dev default — rapid iteration on whatever is in flux.
- **Compilation is per *definition*, gated by the §4 stable-component check** — *not* per file, per crate, or per program. A definition is AOT-eligible when it is content-addressed + hash-frozen, spec-ratified, and its obligations are discharged (§4). Marking-stable stays a deliberate act.
- **Mixed execution is the normal case, not a special mode.** Compiled stable components and still-interpreted definitions **coexist in one run**: both speak the same L0 `CoreValue` semantics (RFC-0001 §4.2 r3), and the §3 checker guarantees they agree (NFR-7). A call from interpreted code into a compiled component (or back) crosses no semantic boundary — only a performance one.
- **Incrementality is "for free" from content-addressing (ADR-003), not a separate build system.** A definition's identity *is* its content hash, so a compiled artifact keyed by that hash is **never stale** and is reused across runs and machines without dependency bookkeeping. A build recompiles exactly the changed definitions and their hash-reachable dependents — nothing more. (The M-311/M-312 `mycelium-build` content-addressed `BuildCertificate`/cache is this mechanism; the RFC-0001 r3 registry Σ extends the same hash-identity to data declarations.)

### 9.2 Build-target profiles (normative)
A build's **target set** is an explicit, flexible choice — opt-in to breadth, never forced to it. The profiles (the `mycelium-build` surface; spellings illustrative, KC-2-gated like all surface syntax):

| Profile | Target set | Use |
|---|---|---|
| `interpret` (default) | none (runs on the reference interpreter) | active development; rapid iteration |
| `build --slim <os>-<arch>` | **exactly one** `(os, arch)` | a release for one platform — smallest artifact |
| `build --target <os>-<arch>[,…]` | a **chosen subset** of `(os, arch)` pairs | "support these two arches on these two OSes" — exactly as many as wanted |
| `build --fat` | **all supported** targets (universal) | one artifact that runs everywhere — the full multi-target build |

- **`--fat` is a first-class, supported-from-the-start option, not the mandatory path.** A developer who wants universal support gets it in one command; a developer who wants one or two targets pays only for those. The model never boxes anyone into the full-fat build.
- **`--slim` and `--target` are the same artifact shape as `--fat` with fewer variants** (§9.3) — there is one artifact format, parameterized by its target set, so moving from slim → selective → fat is a build-flag change, not a re-architecture.
- **Orthogonal to the §4 gate:** the target set says *for which platforms*; the stable-component gate says *which definitions are compiled at all*. A build compiles the stable-eligible (or developer-selected) definitions, each for the chosen target set; everything else stays interpreted.

### 9.3 The fat (multi-target) artifact & runtime dispatch
- A **fat artifact** carries, per compiled definition, the per-`(os, arch[, cpu-features])` code variants in a **content-addressed variant table**. A `--slim`/`--target` artifact is the identical structure with only the selected variants present.
- At load/run time the runtime **detects the host `(os, arch, cpu-features)` and selects the matching variant** — the in-tree precedent is the **M-360 I2_S SIMD runtime feature-dispatch** (a kernel choosing its implementation by detected CPU features), generalized to the platform triple.
- **Never-silent (G2/SC-3):** if the running host matches **no** present variant, the runtime takes the explicit fallback — run that definition on the **interpreter** if it is in the image, else **refuse with an explicit error**. It must *never* run a variant built for the wrong target. Variant selection is inspectable (EXPLAIN-able) like every other selection in the system.
- **Cross-target compilation rides §2's MLIR→LLVM path** (LLVM gives the cross-targets, Rust-style). Until the native libMLIR/LLVM backend lands (deferred, §2 / phase-2.md), `build` is **host-target only** and `--fat`/`--target` for non-host triples is an explicit "not yet built" refusal, never a silent host-only build mislabeled as fat.

## 10. Open questions (r2) — flagged, not yet decided
- **OQ-1 — the interpreted↔compiled ABI.** In-process today, interpreted and compiled code share Rust value types; a **persistent compiled-artifact store** (reused across processes/machines) needs a **stable serialized value + call ABI** at the boundary. Couples to RFC-0001 §4.8 serialization. Its own design (likely an ADR). → **DESIGNED: ADR-016 (Proposed)** — dispatch a compiled definition by its **content hash** (versioning is free, staleness structurally impossible — ADR-003); cross `CoreValue`s in the **self-describing wire form** (RFC-0001 §4.8, canonical), with a zero-copy fast-path as a later validated optimization.
- **OQ-2 — hot inject of recompiled definitions into a running image.** "Compile only the changed definitions and inject them without recompiling/relinking the whole binary." Realistic on the content-addressed foundation (the **M-340 in-process `dlopen` JIT** is the seed: content-addressed dynamic linking + variant selection), but a **reliable, robust** version needs OQ-1's ABI + a versioned dynamic-link story. High-value, explicitly deferred — not promised until designed. → **DESIGNED: ADR-017 (Proposed)** — a hash-keyed dispatch table + content-addressed dynamic linking; the atomicity hazard **dissolves** because definitions are immutable (a change is a new hash under a new entry, never an in-place mutation), and the recompile set is exactly the changed dependency-closure by hash reachability. Native codegen deferred (MLIR→LLVM); an in-process prototype on M-340 is the recommended first build step once ratified.
- **OQ-3 — fat-artifact packaging format.** The content-addressed multi-target variant store (§9.3) is a concrete format that needs specifying (manifest, dedup of shared variants, signing/cert linkage to the §4 `BuildCertificate`).
- **OQ-4 — target-set selection as policy.** Whether `--target`/`--fat` selection should be expressible through the RFC-0005 selection-policy mechanism (one mechanism, now three sites) or stay a build-flag. Lean: build-flag first, policy later if it earns it.

## 11. Direct-LLVM backend: data-fragment increment (r3; M-373)

This section is **additive** (r3). It changes no prior decision — it records the sanctioned scope
for extending the direct-LLVM backend (`crates/mycelium-mlir/src/llvm.rs`) to cover
`Construct`/`Match` nodes, grounding that extension in the §2 revisit clause.

### 11.1 Sanction: the §2 revisit clause

§2 contains the explicit revisit clause: *"Revisit if: a tiny stable substrate set + modest perf
needs would favor a lighter direct-LLVM backend."* The current situation fits: `llvm.rs` is
already a live, compiled, differentially tested direct-LLVM backend for the bit/trit subset
(M-301); LLVM 18 tooling (`llc`/`clang`) is present; libMLIR is absent. Extending `llvm.rs` to
lower `Construct`/`Match` in textual LLVM IR is a sanctioned increment of this clause. It does
not supersede the MLIR→LLVM commitment — it advances the unblocked half while the libMLIR-gated
half remains deferred.

### 11.2 Scope of the sanctioned increment

**Increment 1 (this wave; M-373 scope):** non-recursive `Construct`/`Match` codegen in textual
LLVM IR. Zero libMLIR required. Specifics:

- `Rhs::Construct`: emit a tagged heap struct (`@malloc`; tag word + field words as `i64` slots).
- `Rhs::Match`: emit a `switch i64` on the tag word; bind field words in each arm.
- Straight-line arms only — no closures, no heap recursion, no `App`/`Lam`/`Fix`/`FixGroup`.
- Out-of-memory → explicit `AotError::Run` (never silent; G2).
- No-match where patterns do not cover the discriminant → `AotError::UnsupportedNode` (never
  silent; G2).
- Guarantee tag on returned values: `Declared` (the tag-dispatch convention is declared, not
  formally verified — VR-5).

### 11.3 What remains deferred (VR-5)

The following fragments remain deferred / explicitly unadvanced by this increment:

- **Closures (App/Lam) + heap** — require closure-conversion (free-variable analysis, closure
  struct packing, indirect call through function pointer). A separate later increment.
- **Recursion (Fix/FixGroup) + stack-robustness** — require an explicit heap control stack (a
  trampoline) in the emitted LLVM IR. Must satisfy DN-05 #1 (no unbounded C stack; graceful
  explicit limit; G2) — that requirement applies to the native path and is designed in, not
  retrofitted. The `DepthBudget` trait (DN05-Q4/Q5; enacted M-349; `crates/mycelium-mlir::budget`)
  is reused as the policy interface. A separate later increment.
- **Real `ternary` MLIR dialect lowering** — `crates/mycelium-mlir/src/dialect.rs` is a textual
  skeleton only; real lowering requires libMLIR (M-348, blocked). Every verdict on this path
  stays "not established" (VR-5) until the toolchain is present.

### 11.4 Inspectability (§6 preserved)

Every stage of the Increment-1 extension must remain dumpable/diffable (SC-4; RFC-0004 §6);
each pass preserves `Meta` (WF5); no-opaque-lowering applies (ADR-009). The textual LLVM IR
emitter already satisfies this for the bit/trit subset — the extension must hold the same
standard: every `switch` arm, field-load, and struct-alloc is explicit IR, and the full module
is human-readable and dumpable. No opaque pass is introduced.

## Meta — changelog
- **2026-06-19 (additive — §11; r3; M-373):** Added **§11** recording the sanctioned scope for extending the direct-LLVM backend (`llvm.rs`) to the data-fragment (non-recursive `Construct`/`Match`) as an increment of the §2 revisit clause. Closures (`App`/`Lam`), recursion (`Fix`/`FixGroup`), and the real ternary MLIR dialect lowering remain deferred (VR-5); recursion is bound by DN-05 #1 (no unbounded C stack; `DepthBudget` trait reuse) when it lands. §6 no-opaque-lowering preserved: every IR stage stays dumpable. Changes no prior decision. Append-only.
- **2026-06-18 (append-only note after §4 — RFC-0017 Accepted):** Added an inline note recording
  that **RFC-0017** lifts maturation granularity from per-definition to **scope** (nodule/phylum
  header; program manifest); the §4 stable-component eligibility checks are **unchanged**, applied
  per reachable definition in a matured scope. `thaw fn f` exempts one definition from the matured
  set. No AOT guarantee altered; §4 is the per-definition obligation, RFC-0017 §4.2 is its
  scope-level conjunction. Append-only; no r1/r2 decision changed.
- **2026-06-16 (additive — §2):** Banked a **normative stack-robustness requirement** on the native AOT path (DN-05 #1): object recursion must use a managed/heap call stack with an explicit depth/budget limit — a graceful error, never an abort (G2). Designed in alongside the trampolined AOT env-machine (DN-05 #2, enacted M-347) and the O(1)-stack interpreter; libMLIR provisioning to build it is M-348. Changes no prior decision. Append-only.
- **r1 (initial):** **Accepted.** §2 backend (MLIR→LLVM), §3 single shared checker, §4 stable-component gate, §5 schedule-staged packing, §6 inspectability, §7 interfaces, §8 residual experiments.
- **r2 (2026-06-15):** **Accepted (additive — changes no r1 decision).** Adds **§9** (the interpreted↔compiled continuum made explicit + the **build-target profiles** `interpret`/`--slim`/`--target`/`--fat`, with fat multi-target as a first-class-but-optional path and never-silent runtime variant dispatch) and **§10** open questions (the interpreted↔compiled ABI, hot-inject of recompiled definitions, the fat-artifact packaging format, target-set-as-policy). Records the maintainer's interpret-for-dev / compile-when-ready / flexible-multi-target direction (2026-06-15) on the existing §2/§4 + ADR-003/ADR-009 foundation; the cross-target capability remains gated on the deferred MLIR→LLVM backend (§2). Append-only; maintain status transitions as the ADR/RFC discipline (Draft → Accepted → Superseded).
