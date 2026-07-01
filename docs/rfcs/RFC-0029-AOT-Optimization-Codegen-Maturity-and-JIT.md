# RFC-0029 — AOT Optimization, Codegen Maturity, and JIT

| Field | Value |
|---|---|
| **RFC** | 0029 |
| **Status** | **Accepted** (2026-06-23) · Draft (2026-06-23) |
| **Type** | Foundational / normative — the optimization and native-codegen maturity model |
| **Date** | 2026-06-23 |
| **Feeds** | E15-1 (native AOT maturity, optimization, and acceleration) |
| **Decides** | Which optimization passes are sanctioned and in what form; the full libMLIR lowering path; the JIT policy (relative to ADR-009); the BitNet packed-ternary acceleration surface; the never-silent, EXPLAIN-able discipline for transforms; the three-way `interp ≡ AOT ≡ JIT` differential durability requirement |
| **Depends on** | RFC-0004 (execution model — §2 revisit clause, §5 packing, §6 inspectability); DN-15 (native-path direct-LLVM decomposition — the existing incremental plan); ADR-019 (libMLIR toolchain — **Enacted** 2026-06-23); ADR-006 (no black boxes — EXPLAIN obligation); ADR-009 (hybrid execution: AOT preferred, interpreter the reference, interpreter/JIT for dynamic VSA); ADR-014 (unsafe policy — the JIT's confined FFI); G2/VR-5 (never-silent, honest tags); KC-3 (small auditable kernel) |
| **Coupled with** | `crates/mycelium-mlir/` (the AOT/JIT backend — M-150/M-301/M-340/M-360/M-373/M-379/M-601/M-602); E6-1 (the native-path wave — **`done`**, whose mechanisms this RFC builds on); E15-1 children M-725…M-729 |
| **Task** | E15-1 (epic) / M-725 (first child) |

> **Posture (honesty rule / VR-5).** Accepted as the **design** for E15-1; it decides the
> optimization-pass discipline, JIT policy, BitNet surface, and three-way differential requirement.
> Acceptance ratifies the *design* — it does **not** assert the implementation is complete. Where a
> mechanism already exists (JIT M-340, BitNet M-360, MLIR dialect M-601, three-way native
> differential M-602) the RFC says so and tags it at its honestly-supportable strength
> (`Empirical` for a differentially-checked path; `Declared` for an as-yet-unbuilt goal). Where it
> does **not** exist (the optimization passes of M-726), the RFC says that too. Every claim about
> "what the backend can do today" is grounded in `crates/mycelium-mlir/` (checked 2026-06-23).

---

## 1. Problem / Goal

Mycelium's interpreter is the trusted base (RFC-0004 §6). The native AOT path is more mature than
the E15-1 framing assumed: the E6-1 native-path wave is **`done`**, having landed a direct-LLVM-IR
backend (`llvm.rs`; M-301/M-373/M-379), an in-process `dlopen` JIT (`jit.rs`; M-340), BitNet-class
packed-ternary compute kernels (`bitnet.rs`; M-360), a real `arith`/`func`→LLVM **MLIR-dialect**
lowering for the element-wise fragment (`dialect/native.rs`; M-601, `mlir-dialect` feature), and a
**three-way native differential** (`tests/threeway_differential.rs`; M-602: interp ≡ direct-LLVM ≡
MLIR-dialect). What the path still lacks, and what this RFC sanctions the design for:

1. **Optimization passes** — inlining, CSE, DCE — expressed as EXPLAIN-able, never-silent
   transforms so the no-black-box rule (ADR-006/G2) holds in the optimization layer, not just the
   frontend. **This is the one genuinely-new mechanism in E15-1** (no `src/passes/` exists today).
2. **Full libMLIR lowering coverage** — the M-601 dialect path covers only the bit/trit
   element-wise fragment; widening it (incrementally, honestly) is M-725's residual scope.
3. **JIT formalized as a first-class never-silent mode** — the M-340 JIT path exists; ADR-009
   already *sanctions* interpreter/JIT for dynamic VSA. M-727 makes JIT an explicit, never-silently
   selected execution mode with `interp ≡ JIT` as the correctness bar — it is **not** greenfield.
4. **BitNet acceleration behind an explicit capability flag** — the M-360 kernels exist; M-728
   formalizes the capability gate and the never-silent graceful-degradation contract (FR-C3).
5. **A unified, mutant-witnessed `interp ≡ AOT ≡ JIT` three-way differential** — M-602 is a
   three-way differential over *codegen backends* (interp/LLVM/MLIR); M-729 unifies the AOT and JIT
   legs into one harness and witnesses it with `cargo-mutants` (a codegen mutation is *caught*).

The interpreter **stays the trusted base** throughout. Performance is the goal of the native path;
correctness is always measured against the interpreter (NFR-7).

## 2. User stories

- As a **language user**, I want programs compiled AOT to produce the same results as the
  interpreter, so that I can trust AOT output without re-auditing my code under a different
  execution model.
- As a **compiler engineer**, I want optimization passes (inlining, CSE, DCE) to be EXPLAIN-able —
  each transform reified, inspectable, auditable — so that no optimization is a black box that
  violates ADR-006.
- As a **stdlib author**, I want the AOT backend to cover the language surface it honestly can, and
  to refuse the rest *explicitly* (never silently), so that I always know whether a `.myc` module
  compiled natively or fell back to the interpreter.
- As a **downstream app developer**, I want JIT compilation available for VSA/HDC dynamic
  workloads, so that I get native performance for exploratory or streaming pipelines without an
  offline AOT step.
- As a **maintainer**, I want the three-way differential (interp ≡ AOT ≡ JIT) to be mutant-witnessed
  so that a codegen regression is caught before it reaches `main`.
- As a **tool author**, I want the BitNet packed-ternary acceleration path behind an explicit
  capability flag and never silently engaged, so that portability and reproducibility are not
  compromised on hardware that lacks the acceleration.

## 3. Scope and decision space

### In scope
- The optimization-pass discipline: which passes are sanctioned and how each is expressed as a
  never-silent, EXPLAIN-able transform (extending RFC-0004 §6 inspectability to the optimization
  layer; extending the M-673 `MonoSelections` reification pattern to a pass transform-log).
- The libMLIR lowering coverage policy: how the M-601 dialect path widens (incrementally, with an
  explicit refusal at every honest boundary — never a second divergent codegen for the same
  semantics).
- JIT policy: the M-340 JIT becomes a first-class, never-silently-selected execution mode for
  dynamic VSA/HDC workloads, with `interp ≡ JIT` (Empirical) as the correctness bar.
- The BitNet-class packed-ternary acceleration surface: FR-C3, the capability flag, the correctness
  obligation (accel result ≡ reference ternary, Empirical), and the never-silent
  graceful-degradation contract.
- The `interp ≡ AOT ≡ JIT` three-way differential durability requirement and its mutant-witnessed
  verification strategy.

### Out of scope
- The interpreter (`mycelium-core` / L0) — the trusted base; this RFC concerns the native path
  above it.
- The L1 frontend (parsing, type-checking, monomorphization, defunctionalization) — RFC-0006/0007/
  0019/0024. Generics/traits *run* via the interpreter after M-673 elaboration; native codegen of
  the post-elaboration surface widens under M-725 but is not gated on this RFC.
- The stdlib self-hosting migration — independent track; a `.myc` module is differentially testable
  without a native codegen path.
- Hardware-specific intrinsics beyond the BitNet packed-ternary surface (general SIMD autovec, GPU,
  tile/MMA operations) — deferred beyond 1.0.0.

## 4. Definition of Done

- [x] The full libMLIR lowering path is specified, with each stage's correctness obligation stated
  (§7.1).
- [x] The optimization-pass discipline is documented: which passes (inlining, CSE, DCE) are
  sanctioned; how each is an EXPLAIN-able, never-silent transform; how the pipeline is selected
  without black-box heuristics (§7.2).
- [x] The JIT policy is decided with stated rationale and scope (§7.3).
- [x] The BitNet packed-ternary acceleration surface is specified: capability flag, portability
  contract, correctness equivalence (§7.4).
- [x] The three-way differential durability requirement and its mutant-witnessed verification
  strategy are specified (§7.5).
- [x] This RFC reaches **Accepted** before any *new* M-726…M-729 implementation work begins.
- [x] All open questions in §5 are resolved or explicitly deferred with direction.

## 5. Resolved decisions (was: open questions)

The seven Draft open questions, resolved against `crates/mycelium-mlir/` (checked 2026-06-23):

1. **MLIR binding status — RESOLVED.** ADR-019 (libMLIR toolchain) is **Enacted** (2026-06-23);
   `scripts/setup-mlir.sh` provisions the version-matched tools (`mlir-opt-<major>`,
   `mlir-translate-<major>`) and the repo container ships them. The real dialect lowering
   (`dialect/native.rs`, feature `mlir-dialect`, M-601) is implemented for the element-wise fragment
   and **probes the toolchain at runtime**, returning a graceful `DialectError::ToolchainMissing`
   (the caller skips, never fails) on any box where the tools are absent — so `cargo test
   --features mlir-dialect` is green with or without libMLIR, and the direct-LLVM/interp paths carry
   the rest. M-348 provisioning is therefore resolved (setup via `setup-mlir.sh`; ADR-019); the
   binding is *present where provisioned, gated-but-graceful otherwise*. M-725's residual scope is
   **coverage widening**, not unblocking.
2. **Optimization-pass EXPLAIN model — RESOLVED (§7.2).** Each pass emits a **transform log**: an
   ordered, reified record of `(pass, rule, site, before → after, reason)` entries, queryable via
   `EXPLAIN`, mirroring how M-673 reifies `MonoSelections`. No per-node mutation happens without a
   corresponding log entry (never-silent). This is a transform log, not a per-IR-node annotation and
   not a separate heuristic black box.
3. **JIT scope — RESOLVED (§7.3).** JIT (the existing M-340 in-process `dlopen` path) is for
   **dynamic VSA/HDC and the bit/trit subset it already compiles**, selected **only** by explicit
   API/flag (never a heuristic). REPL/interactive JIT is **deferred beyond 1.0.0**. Correctness bar:
   `JIT result == interpreter result` (Empirical), already exercised by `tests/jit_differential.rs`.
   ADR-009 already permits JIT, so **no superseding ADR is required** — M-727 is a formalization +
   honest status note, not a deferral lift of a forbidden feature.
4. **BitNet acceleration granularity — RESOLVED (§7.4).** 1.0.0 scope is **SIMD-level 2-bit packing
   of `{−1, 0, +1}`** via the three packings already in `bitnet.rs` (I2_S default, TL1, TL2) and the
   runtime-data ternary multiply-accumulate (dot) kernel. Higher-level tile/MMA operations are
   **deferred beyond 1.0.0**. Targets: portable scalar baseline + the packed loop; no
   vendor-specific intrinsics required for 1.0.0.
5. **Three-way differential scope — RESOLVED (§7.5).** `cargo-mutants` over the lowering/pass code
   is the **witness** (a mutation must be caught by the differential suite); `cargo-fuzz` smoke is
   defense-in-depth. A property-based generator over the bit/trit corpus drives the three legs. The
   M-729 claim is `Empirical` only once a mutant is demonstrably caught (otherwise `Declared`).
6. **Codegen coverage of the full surface — RESOLVED (honest map, §7.1).** Covered today: the
   bit/trit **element-wise** fragment (interp / direct-LLVM / MLIR-dialect, three-way checked); the
   bit subset of the direct-LLVM compiled artifact; the env-machine ANF model (`aot.rs`); the
   bit/trit JIT subset (`jit.rs`); the BitNet dot kernels (`bitnet.rs`). **Explicitly refused**
   (never silent) in the MLIR-dialect path and routed to direct-LLVM/interp: trit *carry*
   arithmetic, the `Construct`/`Match` data fragment, closures, recursion, `Swap`, Dense/VSA. M-725
   widens this incrementally; each increment keeps an explicit refusal at its honest boundary.
7. **E6-1 vs E15-1 separation — RESOLVED.** **E6-1 is `done`.** M-340/M-360/M-601/M-602 landed
   under it. M-725's "subsumes E6-1 remaining" therefore reduces to **dialect coverage widening**;
   the JIT and BitNet *mechanisms* already exist, so M-727/M-728 are **formalization + capability
   gating + honest status**, not greenfield builds. This re-grounding is the substantive output of
   acceptance and re-scopes the epic honestly (VR-5).

## 6. Grounding / honesty

- RFC-0004 (Execution Model — §2 revisit clause sanctions advancing the native path; §5 packing;
  §6 inspectability applies to every optimization pass).
- DN-15 (Draft, 2026-06-19) — the honest decomposition of M-348 into libMLIR-gated vs
  direct-LLVM-advanceable halves; the incremental-coverage plan M-725 follows.
- DN-25 (Draft) — Road to Full-Language 1.0.0; the planning context E15-1 sits in.
- ADR-019 (libMLIR toolchain — **Enacted**), ADR-009 (hybrid execution — AOT preferred, interpreter
  the reference, interpreter/JIT for dynamic VSA), ADR-014 (unsafe policy — the JIT's confined FFI),
  ADR-006 (no black boxes), G2/VR-5, KC-3 — the non-negotiable constraints.
- FR-C3 (BitNet-class packed ternary), NFR-7 (execution-path equivalence) — the requirements
  driving the acceleration surface and the differential.
- `crates/mycelium-mlir/` (checked 2026-06-23) — the actual current backend: `llvm.rs`
  (direct-LLVM), `jit.rs` (M-340 JIT), `bitnet.rs` (M-360 accel), `dialect/native.rs` (M-601 MLIR),
  `tests/threeway_differential.rs` (M-602), `tests/jit_differential.rs`, `tests/bitnet_throughput.rs`.

## 7. Normative decisions

### 7.1 libMLIR lowering coverage (M-725)
The ratified path is `ternary → arith/func → LLVM dialect → LLVM IR` (RFC-0004 §2), implemented for
the element-wise fragment (M-601) and toolchain-probed/graceful (ADR-019). Coverage **widens
incrementally**; **every** unhandled construct is an explicit, never-silent `DialectError::Unsupported`
that routes to the direct-LLVM backend or the interpreter — the project **does not** ship a second,
divergent codegen for the same semantics (DRY; G2). Each widening increment carries a three-way
differential (§7.5). Guarantee: `Empirical` per increment (the differential), never `Proven` absent a
checked equivalence proof.

### 7.2 Optimization-pass discipline (M-726 — the genuinely-new work)
Sanctioned passes for 1.0.0: **inlining, CSE, DCE**. Each pass is:
- **EXPLAIN-able** — emits a transform log of `(pass, rule, site, before → after, reason)`,
  reified like M-673's `MonoSelections` and queryable; a user can ask *why* an inline/CSE/DCE
  decision was made. No black-box heuristic (ADR-006).
- **Never-silent** — no IR node is added, folded, or removed without a corresponding log entry; a
  pass that would change *observable* behavior is an **error**, not a feature (G2).
- **Differentially correct** — `output-with-passes == output-without-passes == interpreter`
  (`Empirical`), enforced by the §7.5 harness. Passes live in `crates/mycelium-mlir/src/passes/`.
Additional passes beyond inlining/CSE/DCE are out of scope for 1.0.0 (YAGNI) and require a follow-up.

### 7.3 JIT policy (M-727)
The in-process `dlopen` JIT (M-340, `jit.rs`) is a **first-class execution mode**, selected **only**
explicitly (API/flag) — **never** silently substituted for the interpreter or AOT (G2). Scope:
dynamic VSA/HDC and the bit/trit subset it compiles; REPL/interactive deferred beyond 1.0.0.
Correctness bar before JIT may be chosen over the interpreter: `JIT == interpreter` (`Empirical`,
`tests/jit_differential.rs`). ADR-009 already sanctions JIT for dynamic VSA, so **no superseding ADR
is required**; M-727 records the formalization and keeps the never-silent selection contract.

### 7.4 BitNet packed-ternary acceleration surface (M-728)
FR-C3 acceleration (M-360, `bitnet.rs`) is gated behind an **explicit capability flag** (compile-time
feature + runtime query) — **never** silently engaged. Correctness: `accelerated == reference ternary`
(`Empirical` differential). Portability: on hardware/builds without the acceleration the **reference
ternary path is taken without error and never silently** (a recorded, EXPLAIN-able degradation, G2).
1.0.0 scope: 2-bit `{−1,0,+1}` packing (I2_S/TL1/TL2) + the runtime-data dot kernel; tile/MMA deferred.

### 7.5 Three-way differential durability (M-729)
A unified harness drives `interp ≡ AOT ≡ JIT` over a property-based bit/trit corpus, each pair
validated through the shared M-210 observational-equivalence checker (`repr + payload + guarantee`,
NFR-7). The harness is **mutant-witnessed**: at least one `cargo-mutants` (or `cargo-fuzz`)
mutation of the lowering/pass code must be **caught** by the suite for the claim to be `Empirical`;
absent a demonstrated catch the claim stays `Declared`. This extends M-602 (interp/LLVM/MLIR) and
M-340's `jit_differential` into one durability gate that closes E15-1.

---

### Changelog

| Date | Status | Note |
|---|---|---|
| 2026-06-30 | **Accepted** (unchanged — implementation note) | **E25-1's M-850/851/852/853/854/857 landed** this wave (PRs #818/#821/#823/#824/#825/#820), implementing parts of this RFC's native-codegen-maturity track under ADR-034's re-gating (full-language coverage, not just the M-725…M-729 scope this RFC originally framed): M-850 full recursion (extends §7.5's differential-durability posture to non-tail/`FixGroup` Fix), M-851 closure-ABI widening, M-852 `Swap` native codegen, M-853/M-854 Dense/VSA codegen (the RFC-0039 design vehicle this RFC's §3/§5 Q6 explicitly excluded and handed off), M-857 `trit.mul` through the dialect path (extends §7.5's three-way-differential coverage). **This does not move RFC-0029's Status** — it stays **Accepted, pending ratification**: → Enacted is reserved for when E25-1 fully closes (M-855/856/858/859/860/861/862/863 still open) and the unified mutant-witnessed three-way differential (M-858) durability gate lands, per house rule #3 (Enacted = complete and stable, not a partial landing). Task: E25-1. |
| 2026-06-23 | **Accepted** | Open questions §5 resolved against `crates/mycelium-mlir/` (checked 2026-06-23): E6-1 is `done` (M-340 JIT / M-360 BitNet / M-601 MLIR dialect / M-602 three-way native diff already landed); ADR-019 Enacted; ADR-009 already sanctions JIT. Normative §7 added (pass discipline, JIT policy, BitNet surface, three-way durability). Honest re-scope: M-726 optimization passes are the one new mechanism; M-725/727/728/729 are coverage-widening / formalization / capability-gating / unification. Task: E15-1. |
| 2026-06-23 | **Draft** | Initial stub — open questions enumerated; no normative decisions. Task: E15-1. |
