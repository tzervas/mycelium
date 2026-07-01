# Workspace map — what is built

One-line purpose: the Rust workspace's 50 crates (+ `xtask`), grouped by role, plus the verified
proof artifacts and the LLM-leverage experiment.

## Contents

- [The Rust workspace — 50 crates (+ `xtask`)](#the-rust-workspace--50-crates--xtask)
- [The verified proof artifacts](#the-verified-proof-artifacts)
- [The LLM-leverage experiment](#the-llm-leverage-experiment-kc-2--resolved-dn-09)

## The Rust workspace — 50 crates (+ `xtask`)

The kernel and tooling live in `crates/` under MSRV-pinned Rust 1.92 (ADR-007). The public
surface is gated by a committed API baseline (`docs/spec/api/`, KC-3). **Every crate now carries its
own `README.md`** (linked below) for a 30-second orientation; this map is synthesised from them.
Grouped by role:

### Kernel / trusted base

| Crate | Role |
|---|---|
| [`mycelium-core`](../../crates/mycelium-core/README.md) | Core IR: `Value<Repr,Meta>`, the guarantee lattice, content-addressing, the node grammar; the never-silent fallibility contract (RFC-0001) |
| [`mycelium-numerics`](../../crates/mycelium-numerics/README.md) | Two bound kernels — `ErrorBound` (ε, affine arithmetic) + `ProbBound` (δ, union/apRHL) — meeting at one `{ε, δ, strength}` certificate (ADR-010) |
| [`mycelium-vsa`](../../crates/mycelium-vsa/README.md) | The VSA submodule: the `VsaModel` trait + MAP-I, dependency-gated so the kernel stays small (RFC-0003; ADR-008) |
| [`mycelium-dense`](../../crates/mycelium-dense/README.md) | Dense `Dense{dim,dtype}` values + elementwise ops with honest per-op rounding bounds (RFC-0001 §4.1) |
| [`mycelium-select`](../../crates/mycelium-select/README.md) | The total, EXPLAIN-able selection-policy engine — content-addressed decision tables, no black box (RFC-0005, ADR-006) |

### Compiler / execution

| Crate | Role |
|---|---|
| [`mycelium-interp`](../../crates/mycelium-interp/README.md) | The **reference interpreter** — the trusted small-step semantics; AOT/JIT paths are validated against it (RFC-0004; ADR-009) |
| [`mycelium-cert`](../../crates/mycelium-cert/README.md) | Swap certificates + the certified binary↔ternary swap, and the **one** shared translation-validation checker (RFC-0002) |
| [`mycelium-l1`](../../crates/mycelium-l1/README.md) | The L1 surface prototype (RFC-0006/0007): lexer, parser, typechecker, totality checker, evaluator, elaborator to Core IR; stage-1 generics/traits/effects landed (E7-1) |
| [`mycelium-stack`](../../crates/mycelium-stack/README.md) | Host-stack management for the L1 frontend's recursive passes — kept outside the kernel so `mycelium-l1` stays `unsafe`-free (KC-3) |
| [`mycelium-mlir`](../../crates/mycelium-mlir/README.md) | The AOT path: env-machine, direct-LLVM-IR backend, feature-gated MLIR `ternary` dialect, JIT + hot-inject (RFC-0004; ADR-007). **Implemented and landed** (epic E25-1, wave 2026-06-30/07-01): direct-LLVM now additionally lowers non-tail + mutual recursion, closures over any repr/width, certified binary↔ternary `Swap`, un-quantized Dense elementwise ops, and MAP-I/BSC/HRR/FHRR VSA `bind`/`unbind`/`bundle`/`permute`/`similarity` — each `Empirical` via a checked three-way differential (interp ≡ AOT ≡ JIT); the MLIR-dialect path additionally covers `trit.mul` and `Construct`/`Match`/`Swap`; a JIT path for dynamic VSA/HDC workloads now exists. **Not yet complete:** dialect-path Dense/VSA (tracked M-856b) and the full parallel-codegen/perf-eval increments (M-860/862/863) — see [Status & roadmap](status-and-roadmap.md) for the honest gate state (ADR-034) |
| [`mycelium-mir-passes`](../../crates/mycelium-mir-passes/README.md) | **MEM-4 (DN-33):** the RC-annotated IR + Perceus-style RC emission/elision passes — optimisation-only, **outside** the trusted Core IR (KC-3) |

### Runtime & memory model

| Crate | Role |
|---|---|
| [`mycelium-std-runtime`](../../crates/mycelium-std-runtime/README.md) | The fungal concurrency surface (Colony/Scope/Task/Network/scheduler/supervision — ADR-020 / RFC-0008) **and** the landed **three-layer hybrid memory model** runtime — reclamation records, RC cell, regions, live scope/region wiring, and the three triggers (RcZero · ScopeExit · ChannelClose) — DN-32 / RFC-0027 / DN-33. Its fixed OS-thread `Scheduler` (M-709) also backs `mycelium-bench`'s multicore-scaling measurements (M-859) |

### Toolchain crates

| Crate | Binary | Role |
|---|---|---|
| [`mycelium-cli`](../../crates/mycelium-cli/README.md) | `myc` | The one-command driver: `myc init\|build\|check\|test\|run` over a phylum, with DN-22 structured diagnostics |
| [`mycelium-check`](../../crates/mycelium-check/README.md) | `myc-check` | Project-aware type-check driver; aggregates every refusal as an RFC-0013 diagnostic |
| [`mycelium-fmt`](../../crates/mycelium-fmt/README.md) | `mycfmt` | The canonical formatter — an identity-preserving projection that never changes content-addressed identity |
| [`mycelium-lint`](../../crates/mycelium-lint/README.md) | `myc-lint` | Lint + auto-fix with a `suggest`/`apply`/`scaffold` boundary (M-141 invariant lints) |
| [`mycelium-sec`](../../crates/mycelium-sec/README.md) | `myc-sec` | Security checks — the `wild`-block audit + secrets/supply-chain gates |
| [`mycelium-doc`](../../crates/mycelium-doc/README.md) | `myc-doc` | The doc build pipeline: content-addressed doc-IR, HTML/Typst/JSON renderers, §4.1 quality lint |
| [`mycelium-spore`](../../crates/mycelium-spore/README.md) | `spore` | Content-addressed packager: builds a deployable `spore` from a project (ADR-013) |
| [`mycelium-lsp`](../../crates/mycelium-lsp/README.md) | LSP | The semantic-feedback facade — diagnostics, swap certificates, bound/guarantee annotations, EXPLAIN traces over one surface |
| [`mycelium-build`](../../crates/mycelium-build/README.md) | — | Stable-vs-interpreted classification + content-addressed build certificates (RFC-0004 §4) |
| [`mycelium-proj`](../../crates/mycelium-proj/README.md) | — | Project metadata: nodule header, `mycelium-proj.toml`, the inheritance resolver, `@certification` scoping |
| [`mycelium-bench`](../../crates/mycelium-bench/README.md) | — | Honest benchmark harness: a deterministic WIN/LOSS/REGRESSION report over the execution backends, plus multicore scaling curves and baseline-regression gating (M-859) |
| [`mycelium-diag`](../../crates/mycelium-diag/README.md) | — | The canonical RFC-0013 `Diag` record types (the failure-legibility substrate) |
| [`mycelium-cli-common`](../../crates/mycelium-cli-common/README.md) | — | Small dependency-free helper shared by the toolchain CLIs |

### Standard library — 26 `mycelium-std-*` crates

The Rust-first standard library implements RFC-0016's three-ring contract. Every exported op carries
a per-op guarantee tag; every fallible op returns an explicit `Result`/`Option`, never a silent
fallback. The RFC-0016 §4.5 guarantee matrix is encoded as data and asserted in tests — never prose
only. Each crate's `README.md` links its `docs/spec/stdlib/<name>.md` spec.

**Tier A — differentiators** (the substrates + Mycelium-specific capabilities):
[`std-core`](../../crates/mycelium-std-core/README.md) · [`std-swap`](../../crates/mycelium-std-swap/README.md) ·
[`std-ternary`](../../crates/mycelium-std-ternary/README.md) · [`std-dense`](../../crates/mycelium-std-dense/README.md) ·
[`std-vsa`](../../crates/mycelium-std-vsa/README.md) · [`std-select`](../../crates/mycelium-std-select/README.md) ·
[`std-content`](../../crates/mycelium-std-content/README.md) · [`std-numerics`](../../crates/mycelium-std-numerics/README.md) ·
[`std-diag`](../../crates/mycelium-std-diag/README.md) · [`std-recover`](../../crates/mycelium-std-recover/README.md) ·
[`std-spore`](../../crates/mycelium-std-spore/README.md) · [`std-sys`](../../crates/mycelium-std-sys/README.md) ·
[`std-sys-host`](../../crates/mycelium-std-sys-host/README.md) (the `std-runtime` crate is listed under *Runtime & memory model* above).

**Tier B — common / expected** (same C1–C6 contract, above the Tier-A crates):
[`std-collections`](../../crates/mycelium-std-collections/README.md) · [`std-error`](../../crates/mycelium-std-error/README.md) ·
[`std-cmp`](../../crates/mycelium-std-cmp/README.md) · [`std-iter`](../../crates/mycelium-std-iter/README.md) ·
[`std-math`](../../crates/mycelium-std-math/README.md) · [`std-text`](../../crates/mycelium-std-text/README.md) ·
[`std-fmt`](../../crates/mycelium-std-fmt/README.md) · [`std-io`](../../crates/mycelium-std-io/README.md) ·
[`std-fs`](../../crates/mycelium-std-fs/README.md) · [`std-time`](../../crates/mycelium-std-time/README.md) ·
[`std-rand`](../../crates/mycelium-std-rand/README.md) · [`std-testing`](../../crates/mycelium-std-testing/README.md)

**Note on self-hosting.** The stdlib is Rust-first; the Mycelium-lang migration half (M-502)
is not yet established and is explicitly post-1.0 scope (ADR-021 §5).

## The verified proof artifacts

| Artifact | What it proves |
|---|---|
| `proofs/binary-ternary-roundtrip/` (Z3/SMT2) | Bijectivity of binary↔ternary swaps within range |
| `proofs/lh-bundle/` (Liquid-Haskell) | MAP-I `bundle` capacity refinement: types **SAFE**, Z3 discharged — ratifying the axiomatized-theorem + checked-instantiation strategy (RFC-0003 §5; KC-1 confirmed 2026-06-09) |

## The LLM-leverage experiment (KC-2 — Resolved, DN-09)

The M-002 harness (`experiments/`, `tools/llm-harness/`) ran the KC-2 LLM-leverage experiment
and a subsequent multi-arm retention-ratio ablation (M-381). Verdict (DN-09, Resolved
2026-06-18): **proceed** — the surface is learnable-from-context, the failure mode is a
knowledge-surface gap (not irrecoverable collapse), and the grammar-in-context primer reaches
91.7% pass@1 on frontier models. The retention ratio (arm2 grammar-primed vs arm4 LlmCanonical)
is **DETERMINATE** (DN-09 §10, 2026-06-21): 550% for `grok-build-0.1` and 220% for `grok-4.3`
— the RFC-0021 §4.7 promote-to-projection trigger does **not** fire. L3 strategy selected:
committed text syntax + a co-equal structured-projection layer (M-380, RFC-0021).

---

**See also:** [Guarantees & verification](guarantees-and-verification.md) ·
[Status & roadmap](status-and-roadmap.md) · [Repository structure](repository-structure.md)

[← Back to README](../../README.md)
