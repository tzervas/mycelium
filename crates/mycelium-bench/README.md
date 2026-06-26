# mycelium-bench

> Honest benchmarking and evaluation harness (E-BENCH): measures the existing execution backends over a shared v0-calculus corpus and emits a deterministic WIN/LOSS/REGRESSION report.

**Tier:** tooling  ·  **Status:** Rust-first implementation  ·  **License:** MIT

## Overview

`mycelium-bench` is the measurement counterpart to the whole project — it tells us where Mycelium
wins and where it loses, across execution backends, and surfaces regressions rather than hiding
them. Over a shared corpus of v0-calculus programs, it times the interpreter (the trusted
differential baseline), the AOT env-machine, the JIT, the direct-LLVM backend, and (behind the
`mlir-dialect` feature) the MLIR-dialect path. For each (backend, case) pair it captures wall time
and result, then classifies the result against the interpreter into a `Verdict`: speed WIN/LOSS,
correctness LOSS (differential divergence), capability LOSS (unlowerable node with reason), runtime
error, or environmental skip. It also ingests the LLM-harness report so language-leverage data
(KC-2/SC-5b: quality, latency, token cost) sits alongside execution data. Every measured number is
`Empirical`; a debug build is refused for performance numbers; a differential divergence is always
recorded as a correctness LOSS (VR-5 — no pre-written performance target).

## Key items

- `run_corpus` / `RunRecord` / `CaseRecord` — measure all corpus cases across all backends.
- `Backend` / `Engines` / `Outcome` — the execution backend abstraction.
- `corpus` / `Case` / `Fragment` — the shared v0-calculus program corpus.
- `classify` / `Verdict` / `Speed` — classify a measured outcome against the interpreter baseline.
- `Report` / `Honesty` / `LlmSection` — the deterministic markdown + JSON report with WIN/LOSS table.
- `parse_any_llm_json` / `LlmReport` — LLM-harness report ingestion.
- `bench` binary — the harness entry point.

## Design references

- E-BENCH
- NFR-7, ADR-007
- M-212, M-250, M-303, M-340, M-360
- KC-2, SC-5b
- VR-5, G2

## Role in the workspace

Depends on `mycelium-core`, `mycelium-interp`, `mycelium-mlir`, `mycelium-l1`, and `mycelium-cert`; measures the backends without modifying them. See the [workspace overview](../../README.md).
