# Crate Index

All **50 crates** of the Mycelium Rust workspace, grouped by tier. Each links to its README
in the repository. Roles are distilled from the crate docs (`lib.rs`).

## Kernel / trusted base

- [`mycelium-core`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-core/README.md) — Mycelium Core IR: `Value<Repr,Meta>`, the guarantee lattice, content-addressing, and the node grammar (RFC-0001).
- [`mycelium-dense`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-dense/README.md) — Dense paradigm operational surface: typed, dimension-tracked `Dense{dim,dtype}` values and elementwise ops with honest per-op rounding bounds (RFC-0001 §4.1; RFC-0002 §5; M-230).
- [`mycelium-numerics`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-numerics/README.md) — Verified-numerics foundation: `ErrorBound` (ε, affine arithmetic) + `ProbBound` (δ, union/apRHL) kernels meeting at one shared `{ε,δ,strength}` certificate with a tier-i Rust checker (ADR-010; E2-4).
- [`mycelium-select`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-select/README.md) — Selection-policy language: total, non-learned, content-addressed decision tables with an explicit cost function and mandatory EXPLAIN (RFC-0005; ADR-006; M-220/M-221/M-222).
- [`mycelium-vsa`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-vsa/README.md) — VSA submodule: the `VsaModel` trait and its first model MAP-I, dependency-gated so the kernel stays small (RFC-0003; ADR-008; M-130).

## Compiler / execution

- [`mycelium-cert`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-cert/README.md) — Swap certificates, the binary↔ternary certified swap (RFC-0002 §3/§4; M-120), and the single shared translation-validation checker (RFC-0002 §2; RFC-0004 §3; M-210).
- [`mycelium-interp`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-interp/README.md) — Reference interpreter — the trusted executable small-step semantics for the Core IR (RFC-0004; ADR-009; M-110).
- [`mycelium-l1`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-l1/README.md) — L1 surface prototype (RFC-0006/RFC-0007; NON-NORMATIVE until those RFCs are ratified): lexer, parser, typechecker, totality checker, evaluator, and elaborator to Core IR.
- [`mycelium-mir-passes`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-mir-passes/README.md) — MEM-4 (DN-33): the RC-annotated IR and reference-counting lowering passes (static uniqueness analysis / Perceus-style RC emission and elision). Optimisation-only and OUTSIDE the trusted Core IR (KC-3).
- [`mycelium-mlir`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-mlir/README.md) — AOT path: textual ternary-dialect skeleton, env-machine model, direct-LLVM-IR backend, real MLIR dialect lowering (optional), and the colony runtime (RFC-0004; ADR-007; M-150/M-301).
- [`mycelium-stack`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-stack/README.md) — Host-stack management for the L1 frontend's recursive passes (checker/elaborator), kept outside the trusted kernel so `mycelium-l1` stays `unsafe`-free and auditable (ADR-014; KC-3).

## Runtime & memory model

- [`mycelium-std-runtime`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-std-runtime/README.md) — `std.runtime` — the fungal concurrency surface: Colony/Scope structured concurrency, Task/Network,

## Toolchain

- [`mycelium-bench`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-bench/README.md) — Honest benchmarking and evaluation harness (E-BENCH): measures the existing execution backends over a shared v0-calculus corpus and emits a deterministic WIN/LOSS/REGRESSION report.
- [`mycelium-build`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-build/README.md) — Stable-component build layer (RFC-0004 §4; ADR-003/009): classifies AOT-eligible stable components vs interpreted/JIT and emits content-addressed build certificates.
- [`mycelium-check`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-check/README.md) — Project-aware correctness/type-check driver (`myc-check`): resolves a `mycelium-proj.toml` project, checks the whole phylum/program, and aggregates every refusal as a structured RFC-0013 diagnostic routed via the M-362 auto-baseline.
- [`mycelium-cli-common`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-cli-common/README.md) — Small, dependency-free helper shared by the toolchain CLIs (M-643): folds out the duplicated stdin-or-file reader, the `.myc` source walker, and the hand-rolled argument loop.
- [`mycelium-cli`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-cli/README.md) — `myc` — the one-command toolchain driver (M-733): `myc init|build|check|test|run` over a Mycelium phylum, with DN-22 structured, actionable diagnostics.
- [`mycelium-diag`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-diag/README.md) — Canonical RFC-0013 structured-diagnostic record types — the failure-legibility substrate (`Diag`/`Severity`/`Locus`/`Trace`/`Code`) consumed by `std.diag`, `std.recover`, and `std.testing`.
- [`mycelium-doc`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-doc/README.md) — `myc-doc` — the M-363 documentation build pipeline: a content-addressed doc-IR projected from the corpus, code, and nodule-header metadata, with HTML/Typst/JSON renderers and an eight-check §4.1 quality-bar lint.
- [`mycelium-fmt`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-fmt/README.md) — `mycfmt` — the canonical formatter (M-364): an identity-preserving projection over `.myc` sources that never changes a definition's content-addressed identity (RFC-0001 §4.6/§4.8; ADR-003).
- [`mycelium-lint`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-lint/README.md) — `myc-lint` — lint and auto-fix (M-366): surfaces the M-141 invariant lints and header lints as actionable, reified, opt-in fixes with a `suggest`/`apply`/`scaffold` boundary.
- [`mycelium-lsp`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-lsp/README.md) — Minimal toolchain surface (FR-S5): the invariant linter (M-141), canonical formatter (M-142), and the LSP feedback facade (M-140/M-221) that exposes semantic-feedback artifact kinds — diagnostics, swap certificates, bound/guarantee annotations, and selection EXPLAIN traces — over one surface (SC-5 channel).
- [`mycelium-proj`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-proj/README.md) — Project-metadata layer (M-359; DN-06 §6): the structured nodule header (`// @key: value`), the `mycelium-proj.toml` manifest (a minimal, dependency-free TOML-subset reader), the EXPLAIN-able top-down inheritance resolver, and `@certification` mode scoping.
- [`mycelium-sec`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-sec/README.md) — `myc-sec` — security checks as tooling (M-367): the `/security-review` posture as a suite tool, implementing the Mycelium-specific `wild`-block audit and orchestrating the existing secrets/supply-chain gates.
- [`mycelium-spore`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-spore/README.md) — spore — packaging and publishing (M-368; ADR-013): builds a content-addressed deployable `spore` from a `mycelium-proj.toml` project.

## Standard library

- [`mycelium-std-cmp`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-std-cmp/README.md) — Ordering, equality, and non-repr value conversions for the Mycelium standard library.
- [`mycelium-std-collections`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-std-collections/README.md) — Immutable persistent collections — Seq, Map, and Set — for the Mycelium standard library.
- [`mycelium-std-content`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-std-content/README.md) — Content-addressing and identity library for the Mycelium standard library.
- [`mycelium-std-core`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-std-core/README.md) — Ring-0 prelude for the Mycelium standard library — re-exports of the core value model.
- [`mycelium-std-dense`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-std-dense/README.md) — Dense tensor and embedding operations for the Mycelium standard library.
- [`mycelium-std-diag`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-std-diag/README.md) — Structured diagnostic surface for the Mycelium standard library.
- [`mycelium-std-error`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-std-error/README.md) — Option/Result combinators and recoverable-error surface for the Mycelium standard library.
- [`mycelium-std-fmt`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-std-fmt/README.md) — Dual human/machine projection — display, debug, and JSON formatting for the Mycelium standard library.
- [`mycelium-std-fs`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-std-fs/README.md) — Filesystem access over affine handles for the Mycelium standard library.
- [`mycelium-std-io`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-std-io/README.md) — Single-consumption I/O and canonical serialization for the Mycelium standard library.
- [`mycelium-std-iter`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-std-iter/README.md) — Iterator, fold, and transducer combinators for the Mycelium standard library.
- [`mycelium-std-math`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-std-math/README.md) — Ring-2 numeric function surface — abs, min/max, pow, sqrt, exp, log, trig, and rounding.
- [`mycelium-std-numerics`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-std-numerics/README.md) — ε/δ bound carrier and meet-composition surface for the Mycelium standard library.
- [`mycelium-std-rand`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-std-rand/README.md) — `std.rand` — random number generation with reified, named nondeterminism (declared entropy effects).
- [`mycelium-std-recover`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-std-recover/README.md) — `std.recover` — the declarative recovery bridge: every error is recovered or re-propagated, never dropped.
- [`mycelium-std-select`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-std-select/README.md) — `std.select` — selection DSL with mandatory EXPLAIN capability: every selection returns a choice and an inspectable explanation.
- [`mycelium-std-spore`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-std-spore/README.md) — `std.spore` — content-addressed deployable unit and reconstruction-manifest library surface over the `mycelium-spore` packager.
- [`mycelium-std-swap`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-std-swap/README.md) — `std.swap` — certified, never-silent representation-change surface: every swap yields a value and an inspectable certificate, or an explicit error.
- [`mycelium-std-sys-host`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-std-sys-host/README.md) — Production host wiring: connects the pure std crates' injectable seams to the audited `mycelium-std-sys` OS floor.
- [`mycelium-std-sys`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-std-sys/README.md) — Audited FFI/syscall floor for the Mycelium standard library — the single `wild`-contact phylum.
- [`mycelium-std-ternary`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-std-ternary/README.md) — `std.ternary` — balanced ternary and bit/trit capability surface: exact arithmetic, packed-ternary codecs, and a mandatory EXPLAIN for every packing decision.
- [`mycelium-std-testing`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-std-testing/README.md) — `std.testing` — property, golden, and differential test harness: a skipped or undetermined check is always reported, never a silent pass.
- [`mycelium-std-text`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-std-text/README.md) — `std.text` — UTF-8 string type and operations: parse returns a `Result`, never a sentinel; lossy transcoding is always an explicit, named op.
- [`mycelium-std-time`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-std-time/README.md) — `std.time` — typed clocks, durations, and instants: cross-source subtraction is a compile-time error; wall-clock reads are `Declared` + effectful, never dressed as pure values.
- [`mycelium-std-vsa`](https://github.com/tzervas/mycelium/blob/main/crates/mycelium-std-vsa/README.md) — `std.vsa` — hypervector/VSA encoding capability surface: every approximating op exposes its guarantee tag and an inspectable trace, never a black box.
