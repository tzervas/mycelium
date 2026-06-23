# RFC-0029 — AOT Optimization, Codegen Maturity, and JIT

| Field | Value |
|---|---|
| **RFC** | 0029 |
| **Status** | **Draft** (2026-06-23) |
| **Type** | Foundational / normative (once Accepted) — the optimization and native-codegen maturity model |
| **Date** | 2026-06-23 |
| **Feeds** | E15-1 (native AOT maturity, optimization, and acceleration) |
| **Decides** | Which optimization passes are sanctioned and in what form; the full libMLIR lowering path; the JIT policy (deferral from ADR-009); the BitNet packed-ternary acceleration surface; the never-silent, EXPLAIN-able discipline for transforms |
| **Depends on** | RFC-0004 (execution model — §2 revisit clause, §5 packing, §6 inspectability); DN-15 (native-path direct-LLVM decomposition — the existing incremental plan); ADR-019 (libMLIR toolchain); ADR-006 (no black boxes — EXPLAIN obligation); ADR-009 (JIT deferred — the source of the deferral this RFC lifts); G2/VR-5 (never-silent, honest tags); KC-3 (small auditable kernel) |
| **Coupled with** | `crates/mycelium-mlir/src/lib.rs` (the current AOT skeleton — M-150/M-601/M-301/M-373/M-379); E6-1 (the native-path wave whose remaining tasks M-725 subsumes); E15-1 children M-725…M-729 |
| **Task** | E15-1 (epic) / M-725 (first child — this RFC's authoring is part of the design phase) |

> **Posture (honesty rule / VR-5).** Advisory stub — decides nothing normatively. The
> optimization model, JIT policy, and acceleration surface are **open questions** enumerated in
> §5. Every claim about what "the backend can do today" is grounded in
> `crates/mycelium-mlir/src/lib.rs` (checked 2026-06-23): the current state is a textual
> ternary-dialect skeleton (M-150), real `arith`/`func`→LLVM dialect lowering behind the
> `mlir-dialect` feature (M-601), and a direct-LLVM-IR backend (M-301/M-373/M-378/M-379).
> Optimization passes do not exist yet; JIT does not exist yet. These are `Declared` goals,
> not `Proven` or `Empirical` capabilities.

---

## 1. Problem / Goal

Mycelium's interpreter is the trusted base (RFC-0004 §6). The native AOT path exists but is
immature: `crates/mycelium-mlir` currently provides a direct-LLVM-IR backend and an
`mlir-dialect` feature skeleton; real ternary-dialect → MLIR → LLVM lowering is partially
landed (M-601) but the path is not optimizing and covers only a subset of the language.

To be usable for real workloads — especially VSA/HDC (FR-C3: BitNet-class packed ternary) and
the self-hosted stdlib — the native path needs:

1. **Full libMLIR integration**: the complete ternary-dialect → MLIR → LLVM chain, subsuming
   the remaining work from E6-1.
2. **Optimization passes**: inlining, CSE, DCE — expressed as EXPLAIN-able, never-silent
   transforms so the house rule (no black boxes — ADR-006/G2) holds across the backend, not
   just the frontend.
3. **JIT for dynamic / VSA workloads**: ADR-009 deferred JIT; this RFC decides whether and
   when to enact it.
4. **BitNet-class packed-ternary acceleration**: FR-C3 — hardware packing for ternary ops,
   critical to the VSA/HDC differentiation case.
5. **Differential durability**: the interpreter ≡ AOT ≡ JIT three-way agreement obligation
   (extending the M-210/M-673 differential framework) must hold as new codegen paths land.

The interpreter **stays the trusted base** throughout. Performance is the goal of the native
path; correctness is always measured against the interpreter.

## 2. User stories

- As a **language user**, I want programs compiled AOT to produce the same results as the
  interpreter, so that I can trust AOT output without re-auditing my code under a different
  execution model.
- As a **compiler engineer**, I want optimization passes (inlining, CSE, DCE) to be
  EXPLAIN-able — meaning each transform is reified, inspectable, and auditable — so that no
  optimization is a black box that violates ADR-006.
- As a **stdlib author**, I want the AOT backend to cover the full language surface (including
  generics, traits, HOF, and pattern matching) so that `.myc` stdlib modules compile natively
  without falling back to the interpreter.
- As a **downstream app developer**, I want JIT compilation available for VSA/HDC dynamic
  workloads, so that I can get native performance for exploratory or streaming pipelines without
  an offline AOT step.
- As a **maintainer**, I want the three-way differential (interp ≡ AOT ≡ JIT) to be
  mutant-witnessed so that a codegen regression is caught before it reaches `main`.
- As a **tool author**, I want the BitNet packed-ternary acceleration path to be
  behind an explicit capability flag and never silently engaged, so that portability and
  reproducibility are not compromised on hardware that lacks the acceleration.

## 3. Scope and decision space

### In scope

- Defining the full libMLIR integration path: ternary-dialect → `arith`/`func` → LLVM dialect
  chain, building on M-601 and subsuming E6-1's remaining tasks.
- Specifying the optimization pass discipline: which passes are sanctioned, how each is
  expressed as a never-silent, EXPLAIN-able transform (extending the RFC-0004 §6 inspectability
  obligation to the optimization layer).
- JIT policy: whether to enact the ADR-009 deferral, for which workloads (dynamic VSA/HDC),
  and under what honesty discipline (never silently selected over AOT or interp).
- BitNet-class packed-ternary acceleration surface: FR-C3, the capability flag, the correctness
  obligation (accel result ≡ reference ternary), and the portability/graceful-degradation contract.
- The three-way differential durability requirement (interp ≡ AOT ≡ JIT) and its
  mutant-witnessed verification strategy.

### Out of scope

- The interpreter (`mycelium-core` / L0) itself — it is the trusted base; this RFC concerns
  the native path above it.
- The L1 frontend (parsing, type-checking, monomorphization) — covered by RFC-0006/0007/0019/0024.
- The stdlib self-hosting migration — covered by RFC-0031 (E13-1). The two tracks are
  independent: a `.myc` module can be differentially tested without a native codegen path.
- Hardware-specific intrinsics beyond the BitNet packed-ternary surface (SIMD, GPU, etc.) —
  deferred beyond 1.0.0.

## 4. Definition of Done

- [ ] The full libMLIR lowering path is specified: ternary-dialect → `arith`/`func` →
  LLVM dialect, with each stage's correctness obligation stated.
- [ ] The optimization pass discipline is documented: which passes (inlining, CSE, DCE) are
  sanctioned; how each is expressed as an EXPLAIN-able, never-silent transform; how the
  pass pipeline is selected without black-box heuristics.
- [ ] The JIT policy is decided (enact or re-defer ADR-009) with stated rationale and scope
  (which workloads, which execution mode, which honesty discipline).
- [ ] The BitNet packed-ternary acceleration surface is specified: the capability flag,
  the portability contract, the correctness equivalence to the reference ternary path.
- [ ] The three-way differential durability requirement is specified and its
  mutant-witnessed verification strategy is agreed.
- [ ] This RFC reaches **Accepted** (maintainer ratification) before any M-725…M-729
  implementation work begins.
- [ ] All open questions in §5 are resolved or explicitly deferred with direction.

## 5. Open questions

1. **MLIR binding status** — DN-15 (Draft, 2026-06-19) records that real ternary-dialect
   lowering is libMLIR-gated (`crates/mycelium-mlir/src/dialect.rs` lines 1–6: "no libMLIR
   binding"). ADR-019 (libMLIR toolchain) was the provisioning decision. What is the current
   state of the libMLIR binding in this environment? Is M-348 (provisioning) resolved?
2. **Optimization pass EXPLAIN model** — RFC-0004 §6 requires inspectability of the execution
   model; ADR-006 requires no black boxes. How is an optimization pass made EXPLAIN-able?
   Is the reification a transform log, a per-IR-node annotation, or a separate EXPLAIN query?
3. **JIT scope** — ADR-009 deferred JIT with the rationale that the interpreter serves dynamic
   needs for now. Is the JIT solely for VSA/HDC dynamic workloads, or also for interactive /
   REPL use cases? What is the correctness bar before it can be selected over the interpreter?
4. **BitNet acceleration granularity** — FR-C3 names BitNet-class packed ternary. Is this
   SIMD-level packing of `{-1, 0, +1}` into 2-bit fields, or is it a higher-level tile
   operation? What hardware targets are in scope for 1.0.0?
5. **Three-way differential scope** — the existing two-way differential (L1-eval ≡ L0-interp)
   is established (M-210/M-673). Adding AOT as a third leg requires running the same test suite
   through the native backend. Is mutant testing (`cargo-mutants` / `cargo-fuzz`) sufficient,
   or is a property-based three-way agreement test required?
6. **Codegen coverage of the full surface** — today the AOT backend covers a subset of L0.
   Which language features are unimplemented (generics post-monomorphization, match, recursion,
   traits, HOF after defunctionalization)? This is a gap that must be enumerated before
   M-725…M-729 can be scoped.
7. **Separation of E6-1 remaining vs E15-1** — M-725 ("full libMLIR integration: subsumes E6-1
   remaining") assumes E6-1 has open tasks. What is E6-1's current completion state, and which
   tasks does M-725 absorb?

## 6. Grounding / honesty

- RFC-0004 (Execution Model — §2 revisit clause, §5 packing, §6 inspectability) — the
  execution-model basis; the §2 revisit clause is the sanction for advancing the native path.
- DN-15 (Draft, 2026-06-19) — the honest decomposition of M-348 into libMLIR-gated vs
  direct-LLVM-advanceable halves; ground truth for the current backend state.
- ADR-019 (libMLIR toolchain) — the provisioning decision; status must be checked before
  claiming the MLIR path is unblocked.
- `crates/mycelium-mlir/src/lib.rs` (checked 2026-06-23) — the actual current backend: textual
  skeleton (M-150), `mlir-dialect` feature (M-601), direct-LLVM (M-301/M-373/M-378/M-379).
- ADR-006 (no black boxes), ADR-009 (JIT deferred), G2/VR-5, KC-3 — non-negotiable constraints.
- FR-C3 (BitNet-class packed ternary) — the functional requirement driving the acceleration
  surface.

---

### Changelog

| Date | Status | Note |
|---|---|---|
| 2026-06-23 | **Draft** | Initial stub — open questions enumerated; no normative decisions. Task: E15-1. |
