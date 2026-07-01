# Architecture

Mycelium is a Rust workspace of **50 crates** (MSRV-pinned Rust 1.92, ADR-007) plus the
Mycelium-language surface that lowers onto them. The public surface of each crate is gated by a
committed API baseline (`docs/spec/api/`, KC-3). This page is the conceptual map; the
[Crate Index](Crate-Index.md) lists every crate with a link to its README.

## The tiers

- **Kernel / trusted base** — the value model and the small auditable core (KC-3). `mycelium-core`
  is the Core IR (`Value<Repr,Meta>`, the guarantee lattice, content-addressing, the `Node`
  grammar); `mycelium-numerics` carries the verified ε/δ bound kernels (ADR-010); `mycelium-vsa`,
  `mycelium-dense`, and `mycelium-select` are the paradigm + selection libraries.
- **Compiler / execution** — `mycelium-interp` is the **reference interpreter** (the trusted
  small-step semantics every other path is validated against); `mycelium-cert` holds the swap
  certificates + the single shared translation-validation checker; `mycelium-l1` is the surface
  prototype (lexer → parser → typechecker → totality → elaborator to Core IR); `mycelium-mlir` is
  the AOT path (env-machine, direct-LLVM, optional MLIR dialect, JIT, hot-inject); `mycelium-stack`
  keeps the L1 frontend's recursion `unsafe`-free; `mycelium-mir-passes` is the MEM-4 RC-elision
  pass crate (outside the trusted Core IR).
- **Runtime & memory model** — `mycelium-std-runtime` is the fungal concurrency surface
  (Colony/Scope/Task/Network/scheduler/supervision) **and** the landed three-layer memory model. See
  [Memory Model](Memory-Model.md).
- **Standard library** — 26 `mycelium-std-*` crates implementing RFC-0016's three-ring contract,
  every op honestly tagged and never-silent.
- **Toolchain** — `myc` (the one-command driver), `myc-check`, `mycfmt`, `myc-lint`, `myc-sec`,
  `myc-doc`, `spore`, the LSP, build/proj/bench/diag, and shared CLI helpers.

## The central ideas

- **Representation is part of the type.** `Binary{width}`, `Ternary{trits}`, `Dense{dim,dtype}`,
  `VSA{model,dim,sparsity}` are distinct families with **no implicit conversion**.
- **`Swap` is the only representation-changing operation**, and at the `certified` mode every swap
  emits a **certificate** describing exactly what the conversion cost — bijective for
  binary↔ternary, bounded/probabilistic for ↔VSA/embedding (the split regime, RFC-0002).
- **Transparency is a typed, monotone property.** The guarantee lattice
  **`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`** travels with every value and degrades by *meet*, so a
  disclosed guarantee never spuriously strengthens (VR-5). See [Tunable Certification](Tunable-Certification.md).
- **No black boxes (G2).** Selections, conversions, and reclamations are reified, inspectable, and
  `EXPLAIN`-able; out-of-range is an explicit `Option`/error, never a quiet value.
- **Metadata survives lowering** (Arrow-grade): provenance, bounds, layout, and reconstruction info
  are queryable at runtime and exposed to tooling.

## Execution paths

- **Reference interpreter** (`mycelium-interp`) — the trusted base and the reference semantics;
  every other path is validated against it, never the other way around.
- **AOT** (`mycelium-mlir`) — MLIR → LLVM, confined to the performance path. The native-codegen
  wave (epic **E25-1**, ADR-034) has landed direct-LLVM coverage for non-tail/mutual recursion
  (heap trampoline, M-850), closures via specialize-at-application inlining (M-851), the
  certified binary↔ternary `Swap` (M-852), `Dense` (M-853), and `VSA` (M-854) — plus a **JIT**
  path for dynamic-VSA/HDC workloads (M-855) and **MLIR-dialect** catch-up for `Construct`/`Match`/
  `Swap` (M-856; Dense/VSA through the dialect leg is tracked separately as M-856b). A **unified
  three-way differential** (interp / direct-LLVM / MLIR-dialect, plus JIT for its in-subset
  fragment, M-858) is the checked basis for these claims. **Status: implemented (Rust-first),
  tagged `Empirical`** — mutant-witnessed on a checked basis, not `Proven`; the epic itself is
  still **in-progress** (parallel codegen and the post-landing performance-evaluation/ratification
  steps remain open). Every compiled path is validated against the interpreter by the shared
  certificate checker (`mycelium-cert`); an unsupported fragment is an explicit, never-silent
  refusal rather than a silent fallback (G2).
- **Memory** — values are immutable (LR-8) + acyclic (LR-9) + content-addressed; the runtime memory
  model reclaims them through the three-layer hybrid, with the static RC-elision passes
  (`mycelium-mir-passes`) removing provably-redundant reference-counting work.

> **Grounding note (VR-5).** RFC-0029 (AOT optimization/codegen maturity/JIT) and RFC-0039 (native
> Dense/VSA codegen) plus ADR-034 (the T6 re-gating that brought E25-1 into the `lang 1.0.0` gate)
> are **Accepted** — ratified as design, not yet **Enacted** (Enacted requires the full E25-1 path
> complete + stable, per house rule #3). Don't read "implemented" above as "ratified-complete."

## House rules (enforced)

Small auditable kernel (KC-3); the transparency & auditability rule (per-op tags never upgraded
without a checked basis); never-silent operations (G2); append-only decisions (RFC/ADR/DN advance,
never rewrite); grounded claims. See [Decision Records](Decision-Records.md) and the repository
`CONTRIBUTING.md` / `CLAUDE.md`.

---

**Up:** [Home](Home.md) · [Crate Index](Crate-Index.md) ·
[Doc Index](https://github.com/tzervas/mycelium/blob/main/docs/Doc-Index.md) ·
[repo root README](https://github.com/tzervas/mycelium/blob/main/README.md)
