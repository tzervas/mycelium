# mycelium-mlir

> AOT path: textual ternary-dialect skeleton, env-machine model, direct-LLVM-IR backend, real MLIR dialect lowering (optional), and the colony runtime (RFC-0004; ADR-007; M-150/M-301).

**Tier:** compiler  ·  **Status:** Rust-first implementation  ·  **License:** MIT

## Overview

`mycelium-mlir` is the ahead-of-time compilation and runtime crate. It provides four layered paths: a textual ternary-dialect rendering of lowered Core IR (always available; the per-stage-dumpable anchor); a real `arith`/`func`→LLVM dialect lowering behind the `mlir-dialect` feature (M-601) that probes for `mlir-opt`/`mlir-translate` and skips gracefully when absent; a direct-LLVM-IR backend (`llvm::compile_and_run`) that emits textual LLVM IR for the bit subset and drives `llc`/`clang` to a real native executable; and an env-machine (`aot::run`) that runs the full v0 calculus as a genuine second execution path for the interp↔AOT differential (M-151/NFR-7).

Additional modules: `inject::Image` (in-process hot-inject prototype, M-341), `budget::DepthBudget` (dynamic depth budget from `/proc` + RLIMIT_AS), `runtime::run_colony` (the colony runtime), and `simd`/`specialize`/`bitnet` backends for the ternary-dot kernel.

## Key items

- `aot::run` / `run_core` — env-machine evaluation over lowered ANF; the second execution path.
- `llvm::compile_and_run` / `emit_llvm_ir` — direct-LLVM-IR backend for the bit subset.
- `dialect::emit` — textual ternary-dialect rendering (always available, no toolchain needed).
- `inject::Image` / `recompile_closure` — content-addressed hot-inject dispatch (M-341).
- `budget::DepthBudget` / `AutoDepthBudget` — `EXPLAIN`-able dynamic depth budget (DN-05 §2.4).
- `runtime::run_colony` — colony runtime with `Task`/`Scope`/`Deadlock` detection.
- `vr4::cross_backend_gate` — VR-4 three-way cross-backend gate.

## Design references

- RFC-0004, ADR-007, ADR-009, ADR-016, ADR-017, ADR-019, DN-05, M-150, M-151, M-301, M-341, M-373, M-378, M-379, M-601, M-602, NFR-7, T1.5

## Role in the workspace

Depends on `mycelium-core` and `mycelium-interp`. Dev-dependencies include `mycelium-cert`, `mycelium-numerics`, and `mycelium-select` for the differential and layout tests. The trusted base remains the interpreter (NFR-7); this crate is the perf/inspectability path above it. See the [workspace overview](../../README.md).
