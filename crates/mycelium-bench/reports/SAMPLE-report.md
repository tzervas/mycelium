# Mycelium honest benchmark report

> Tool `mycelium-bench` — profile `release` — `mlir-dialect` feature: OFF — host: x86_64-linux, 4 hw threads (provenance only)

Guarantee lattice: `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`.

**Honesty:** Every measured number is Empirical (a trial mean with its trial count + spread); a capability loss / skip / runtime error is Declared. No verdict is Proven or Exact, and no performance target is pre-written (VR-5). A differential divergence from the trusted interpreter is a recorded correctness LOSS; an unlowerable node is a recorded capability LOSS (G2 — never omitted).

Speed band: a backend within ±10% of the interpreter is *neutral*; faster is a **WIN**, slower a **LOSS (speed)**. Trusted baseline: the **interpreter** (in-process; NFR-7/ADR-007).

Tally across the run: **1 win(s)**, 1 neutral, **26 speed-loss(es)**, **0 correctness-loss(es)**, **14 capability-loss(es)**, 0 runtime-error(s), 14 skip(s).

**Microbench caveats (honest):** numbers are warmup + min-mean over batches via `std::time::Instant` (no `criterion`). The compiled native paths (`direct-llvm`, `mlir-dialect`) are **process-spawn-bound**: each invocation execs a fresh native artifact, so for a trivial kernel the per-invocation figure is spawn-dominated, **not** kernel compute (the honest M-602/E1 finding — surfaced, not buried). `jit` runs in-process (`dlopen`) so it is not spawn-bound. A debug build is refused for perf numbers.

## WIN / LOSS / regression table

Each non-baseline backend vs the interpreter, per case. `ratio` is `interp / backend` (>1 ⇒ backend faster). Tag is per-row.

| case | fragment | backend | verdict | ratio | tag | reason / detail |
|---|---|---|---|---|---|---|
| `bit-literal` | bit-subset | `aot-env` | LOSS (speed) | 0.04x | Empirical | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `bit-literal` | bit-subset | `jit` | LOSS (speed) | 0.03x | Empirical | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `bit-literal` | bit-subset | `direct-llvm` | LOSS (speed) | 0.00x | Empirical | process-spawn-bound: the per-invocation time is dominated by spawning a fresh native process, not kernel compute (M-602/E1) — expected for a trivial kernel vs the in-process interpreter |
| `bit-literal` | bit-subset | `mlir-dialect` | skipped | — | Declared | the `mlir-dialect` feature is off (build with --features mlir-dialect) |
| `bit-not` | bit-subset | `aot-env` | LOSS (speed) | 0.09x | Empirical | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `bit-not` | bit-subset | `jit` | LOSS (speed) | 0.05x | Empirical | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `bit-not` | bit-subset | `direct-llvm` | LOSS (speed) | 0.00x | Empirical | process-spawn-bound: the per-invocation time is dominated by spawning a fresh native process, not kernel compute (M-602/E1) — expected for a trivial kernel vs the in-process interpreter |
| `bit-not` | bit-subset | `mlir-dialect` | skipped | — | Declared | the `mlir-dialect` feature is off (build with --features mlir-dialect) |
| `bit-xor-not` | bit-subset | `aot-env` | LOSS (speed) | 0.18x | Empirical | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `bit-xor-not` | bit-subset | `jit` | LOSS (speed) | 0.11x | Empirical | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `bit-xor-not` | bit-subset | `direct-llvm` | LOSS (speed) | 0.00x | Empirical | process-spawn-bound: the per-invocation time is dominated by spawning a fresh native process, not kernel compute (M-602/E1) — expected for a trivial kernel vs the in-process interpreter |
| `bit-xor-not` | bit-subset | `mlir-dialect` | skipped | — | Declared | the `mlir-dialect` feature is off (build with --features mlir-dialect) |
| `bit-let-chain` | bit-subset | `aot-env` | LOSS (speed) | 0.26x | Empirical | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `bit-let-chain` | bit-subset | `jit` | LOSS (speed) | 0.24x | Empirical | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `bit-let-chain` | bit-subset | `direct-llvm` | LOSS (speed) | 0.01x | Empirical | process-spawn-bound: the per-invocation time is dominated by spawning a fresh native process, not kernel compute (M-602/E1) — expected for a trivial kernel vs the in-process interpreter |
| `bit-let-chain` | bit-subset | `mlir-dialect` | skipped | — | Declared | the `mlir-dialect` feature is off (build with --features mlir-dialect) |
| `trit-neg` | bit-subset | `aot-env` | LOSS (speed) | 0.09x | Empirical | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `trit-neg` | bit-subset | `jit` | LOSS (speed) | 0.05x | Empirical | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `trit-neg` | bit-subset | `direct-llvm` | LOSS (speed) | 0.00x | Empirical | process-spawn-bound: the per-invocation time is dominated by spawning a fresh native process, not kernel compute (M-602/E1) — expected for a trivial kernel vs the in-process interpreter |
| `trit-neg` | bit-subset | `mlir-dialect` | skipped | — | Declared | the `mlir-dialect` feature is off (build with --features mlir-dialect) |
| `trit-add` | bit-subset | `aot-env` | LOSS (speed) | 0.12x | Empirical | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `trit-add` | bit-subset | `jit` | LOSS (speed) | 0.08x | Empirical | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `trit-add` | bit-subset | `direct-llvm` | LOSS (speed) | 0.00x | Empirical | process-spawn-bound: the per-invocation time is dominated by spawning a fresh native process, not kernel compute (M-602/E1) — expected for a trivial kernel vs the in-process interpreter |
| `trit-add` | bit-subset | `mlir-dialect` | skipped | — | Declared | the `mlir-dialect` feature is off (build with --features mlir-dialect) |
| `swap-roundtrip` | swap | `aot-env` | LOSS (speed) | 0.17x | Empirical | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `swap-roundtrip` | swap | `jit` | LOSS (capability) | — | Declared | unsupported node for the AOT subset: swap to Ternary { trits: 6 } (the subset is straight-line bit/trit ops; M-301) |
| `swap-roundtrip` | swap | `direct-llvm` | LOSS (capability) | — | Declared | unsupported node for the AOT subset: swap to Ternary { trits: 6 } (the subset is straight-line bit/trit ops; M-301) |
| `swap-roundtrip` | swap | `mlir-dialect` | skipped | — | Declared | the `mlir-dialect` feature is off (build with --features mlir-dialect) |
| `data-match-repr` | data | `aot-env` | LOSS (speed) | 0.10x | Empirical | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `data-match-repr` | data | `jit` | LOSS (speed) | 0.07x | Empirical | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `data-match-repr` | data | `direct-llvm` | LOSS (speed) | 0.00x | Empirical | process-spawn-bound: the per-invocation time is dominated by spawning a fresh native process, not kernel compute (M-602/E1) — expected for a trivial kernel vs the in-process interpreter |
| `data-match-repr` | data | `mlir-dialect` | skipped | — | Declared | the `mlir-dialect` feature is off (build with --features mlir-dialect) |
| `data-construct` | data | `aot-env` | LOSS (speed) | 0.07x | Empirical | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `data-construct` | data | `jit` | LOSS (capability) | — | Declared | unsupported node for the AOT subset: Construct field: expected a repr lane but found a data value |
| `data-construct` | data | `direct-llvm` | LOSS (capability) | — | Declared | unsupported node for the AOT subset: Construct field: expected a repr lane but found a data value |
| `data-construct` | data | `mlir-dialect` | skipped | — | Declared | the `mlir-dialect` feature is off (build with --features mlir-dialect) |
| `data-nested-match` | data | `aot-env` | LOSS (speed) | 0.12x | Empirical | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `data-nested-match` | data | `jit` | LOSS (capability) | — | Declared | unsupported node for the AOT subset: Construct field: expected a repr lane but found a data value |
| `data-nested-match` | data | `direct-llvm` | LOSS (capability) | — | Declared | unsupported node for the AOT subset: Construct field: expected a repr lane but found a data value |
| `data-nested-match` | data | `mlir-dialect` | skipped | — | Declared | the `mlir-dialect` feature is off (build with --features mlir-dialect) |
| `rec-self` | recursion | `aot-env` | LOSS (speed) | 0.65x | Empirical | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `rec-self` | recursion | `jit` | LOSS (capability) | — | Declared | unsupported node for the AOT subset: Construct field: expected a repr lane but found a data value |
| `rec-self` | recursion | `direct-llvm` | LOSS (capability) | — | Declared | unsupported node for the AOT subset: Construct field: expected a repr lane but found a data value |
| `rec-self` | recursion | `mlir-dialect` | skipped | — | Declared | the `mlir-dialect` feature is off (build with --features mlir-dialect) |
| `rec-build` | recursion | `aot-env` | neutral | 1.07x | Empirical |  |
| `rec-build` | recursion | `jit` | LOSS (capability) | — | Declared | unsupported node for the AOT subset: Construct field: expected a repr lane but found a data value |
| `rec-build` | recursion | `direct-llvm` | LOSS (capability) | — | Declared | unsupported node for the AOT subset: Construct field: expected a repr lane but found a data value |
| `rec-build` | recursion | `mlir-dialect` | skipped | — | Declared | the `mlir-dialect` feature is off (build with --features mlir-dialect) |
| `rec-mutual` | recursion | `aot-env` | WIN | 1.11x | Empirical |  |
| `rec-mutual` | recursion | `jit` | LOSS (capability) | — | Declared | unsupported node for the AOT subset: FixGroup: mutual recursion is not supported in Increment-3 (only single Fix with a λparam.Match body is supported; RFC-0004 §11.6; G2) |
| `rec-mutual` | recursion | `direct-llvm` | LOSS (capability) | — | Declared | unsupported node for the AOT subset: FixGroup: mutual recursion is not supported in Increment-3 (only single Fix with a λparam.Match body is supported; RFC-0004 §11.6; G2) |
| `rec-mutual` | recursion | `mlir-dialect` | skipped | — | Declared | the `mlir-dialect` feature is off (build with --features mlir-dialect) |
| `rec-fold` | recursion | `aot-env` | LOSS (speed) | 0.26x | Empirical | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `rec-fold` | recursion | `jit` | LOSS (capability) | — | Declared | unsupported node for the AOT subset: Construct field: expected a repr lane but found a data value |
| `rec-fold` | recursion | `direct-llvm` | LOSS (capability) | — | Declared | unsupported node for the AOT subset: Construct field: expected a repr lane but found a data value |
| `rec-fold` | recursion | `mlir-dialect` | skipped | — | Declared | the `mlir-dialect` feature is off (build with --features mlir-dialect) |

## Per-case timings (ns/call, Empirical)

Interpreter baseline + each backend that produced a timed value. `spread` is worst/best batch (a noise flag). `—` = not timed (skip / capability loss / error).

| case | interp ns | aot-env ns | jit ns | direct-llvm ns | mlir-dialect ns |
|---|---|---|---|---|---|
| `bit-literal` | 1.3k | 29.9k | 48.7k | 1.81M | — |
| `bit-not` | 3.4k | 37.9k | 63.5k | 2.07M | — |
| `bit-xor-not` | 7.7k | 42.1k | 69.4k | 2.56M | — |
| `bit-let-chain` | 14.1k | 53.8k | 59.1k | 1.23M | — |
| `trit-neg` | 3.3k | 35.8k | 60.8k | 2.43M | — |
| `trit-add` | 4.6k | 38.4k | 56.4k | 1.56M | — |
| `swap-roundtrip` | 8.1k | 46.4k | — | — | — |
| `data-match-repr` | 4.3k | 42.7k | 66.2k | 2.10M | — |
| `data-construct` | 2.7k | 37.4k | — | — | — |
| `data-nested-match` | 7.9k | 65.2k | — | — | — |
| `rec-self` | 50.0k | 76.6k | — | — | — |
| `rec-build` | 71.3k | 66.7k | — | — | — |
| `rec-mutual` | 87.0k | 78.3k | — | — | — |
| `rec-fold` | 58.7k | 223.4k | — | — | — |

One-time compile cost (emit IR → toolchain → native, NOT in the per-run figures above):

- `bit-literal` / `jit`: 185.94M (one-time)
- `bit-literal` / `direct-llvm`: 197.30M (one-time)
- `bit-not` / `jit`: 121.90M (one-time)
- `bit-not` / `direct-llvm`: 292.49M (one-time)
- `bit-xor-not` / `jit`: 119.43M (one-time)
- `bit-xor-not` / `direct-llvm`: 228.86M (one-time)
- `bit-let-chain` / `jit`: 120.88M (one-time)
- `bit-let-chain` / `direct-llvm`: 278.79M (one-time)
- `trit-neg` / `jit`: 230.96M (one-time)
- `trit-neg` / `direct-llvm`: 187.79M (one-time)
- `trit-add` / `jit`: 118.30M (one-time)
- `trit-add` / `direct-llvm`: 218.81M (one-time)
- `data-match-repr` / `jit`: 121.16M (one-time)
- `data-match-repr` / `direct-llvm`: 250.18M (one-time)

## Where we're losing (explicit)

### Capability losses (a backend cannot lower the program — the reason, never omitted, G2)

| case | backend | reason |
|---|---|---|
| `swap-roundtrip` | `jit` | unsupported node for the AOT subset: swap to Ternary { trits: 6 } (the subset is straight-line bit/trit ops; M-301) |
| `swap-roundtrip` | `direct-llvm` | unsupported node for the AOT subset: swap to Ternary { trits: 6 } (the subset is straight-line bit/trit ops; M-301) |
| `data-construct` | `jit` | unsupported node for the AOT subset: Construct field: expected a repr lane but found a data value |
| `data-construct` | `direct-llvm` | unsupported node for the AOT subset: Construct field: expected a repr lane but found a data value |
| `data-nested-match` | `jit` | unsupported node for the AOT subset: Construct field: expected a repr lane but found a data value |
| `data-nested-match` | `direct-llvm` | unsupported node for the AOT subset: Construct field: expected a repr lane but found a data value |
| `rec-self` | `jit` | unsupported node for the AOT subset: Construct field: expected a repr lane but found a data value |
| `rec-self` | `direct-llvm` | unsupported node for the AOT subset: Construct field: expected a repr lane but found a data value |
| `rec-build` | `jit` | unsupported node for the AOT subset: Construct field: expected a repr lane but found a data value |
| `rec-build` | `direct-llvm` | unsupported node for the AOT subset: Construct field: expected a repr lane but found a data value |
| `rec-mutual` | `jit` | unsupported node for the AOT subset: FixGroup: mutual recursion is not supported in Increment-3 (only single Fix with a λparam.Match body is supported; RFC-0004 §11.6; G2) |
| `rec-mutual` | `direct-llvm` | unsupported node for the AOT subset: FixGroup: mutual recursion is not supported in Increment-3 (only single Fix with a λparam.Match body is supported; RFC-0004 §11.6; G2) |
| `rec-fold` | `jit` | unsupported node for the AOT subset: Construct field: expected a repr lane but found a data value |
| `rec-fold` | `direct-llvm` | unsupported node for the AOT subset: Construct field: expected a repr lane but found a data value |

### Speed losses (slower than the in-process interpreter — measured, with the derivable reason)

| case | backend | ratio (interp/backend) | reason |
|---|---|---|---|
| `bit-literal` | `aot-env` | 0.04x | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `bit-literal` | `jit` | 0.03x | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `bit-literal` | `direct-llvm` | 0.00x | process-spawn-bound: the per-invocation time is dominated by spawning a fresh native process, not kernel compute (M-602/E1) — expected for a trivial kernel vs the in-process interpreter |
| `bit-not` | `aot-env` | 0.09x | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `bit-not` | `jit` | 0.05x | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `bit-not` | `direct-llvm` | 0.00x | process-spawn-bound: the per-invocation time is dominated by spawning a fresh native process, not kernel compute (M-602/E1) — expected for a trivial kernel vs the in-process interpreter |
| `bit-xor-not` | `aot-env` | 0.18x | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `bit-xor-not` | `jit` | 0.11x | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `bit-xor-not` | `direct-llvm` | 0.00x | process-spawn-bound: the per-invocation time is dominated by spawning a fresh native process, not kernel compute (M-602/E1) — expected for a trivial kernel vs the in-process interpreter |
| `bit-let-chain` | `aot-env` | 0.26x | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `bit-let-chain` | `jit` | 0.24x | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `bit-let-chain` | `direct-llvm` | 0.01x | process-spawn-bound: the per-invocation time is dominated by spawning a fresh native process, not kernel compute (M-602/E1) — expected for a trivial kernel vs the in-process interpreter |
| `trit-neg` | `aot-env` | 0.09x | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `trit-neg` | `jit` | 0.05x | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `trit-neg` | `direct-llvm` | 0.00x | process-spawn-bound: the per-invocation time is dominated by spawning a fresh native process, not kernel compute (M-602/E1) — expected for a trivial kernel vs the in-process interpreter |
| `trit-add` | `aot-env` | 0.12x | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `trit-add` | `jit` | 0.08x | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `trit-add` | `direct-llvm` | 0.00x | process-spawn-bound: the per-invocation time is dominated by spawning a fresh native process, not kernel compute (M-602/E1) — expected for a trivial kernel vs the in-process interpreter |
| `swap-roundtrip` | `aot-env` | 0.17x | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `data-match-repr` | `aot-env` | 0.10x | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `data-match-repr` | `jit` | 0.07x | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `data-match-repr` | `direct-llvm` | 0.00x | process-spawn-bound: the per-invocation time is dominated by spawning a fresh native process, not kernel compute (M-602/E1) — expected for a trivial kernel vs the in-process interpreter |
| `data-construct` | `aot-env` | 0.07x | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `data-nested-match` | `aot-env` | 0.12x | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `rec-self` | `aot-env` | 0.65x | slower than the in-process interpreter on this case (measured; no target — VR-5) |
| `rec-fold` | `aot-env` | 0.26x | slower than the in-process interpreter on this case (measured; no target — VR-5) |

## LLM-harness leverage (KC-2 / SC-5b)

Source: `/home/user/mycelium/.claude/worktrees/agent-ac9d36a1c13fe5cbd/tools/llm-harness/reports/20260617T182214Z-report.json` — **SYNTHETIC sample** (a fixture run — NOT real model quality; never treated as evidence, per the harness's own VR-5/V-03 rule).

> mycelium-llm-validation v0.1.0 — run 20260617T182214Z — mode=mock — SYNTHETIC (fixture; not real model quality) (4 validations: 1 pass / 3 mock-pass / 0 skip / 0 fail)

| validation | status | tag | latency (s) | prompt tok | gen tok | message |
|---|---|---|---|---|---|---|
| `V-01-determinism` | mock-PASS | Declared | — | — | — | [MOCK] Determinism check simulated with fixture — not a real model run. Fixture outputs matched: True |
| `V-02-json-projection` | mock-PASS | Declared | — | — | — | [MOCK] JSON-projection check against fixture — not a real model run. Fixture parsed and validated OK |
| `V-03-tag-honesty` | PASS | — | — | — | — | Tag-honesty gate PASSED. Correctly rejected 2 forbidden tag(s), correctly allowed 2 compliant tag(s).  |
| `V-04-latency-tokens` | mock-PASS | Declared | 0.0000 | 12 | 3 | [MOCK] Latency/token report — synthetic numbers. wall_seconds=0.0 is a sentinel meaning 'not measured (mock mode)'. Prompt tokens and generated tokens are fixture values. |
