# Mycelium

> A **fast, memory-safe, ergonomic** multi-paradigm language that treats **traditional binary**,
> **balanced ternary**, **dense embeddings**, and **Vector Symbolic Architectures (VSA /
> hyperdimensional computing)** as co-equal, first-class substrates — under semantics that are
> **transparent** (no hidden behavior) and **metadata-native**, with **certification & auditability
> baked in as *optional, tunable* capabilities** (`fast` by default · `certified` on request)
> rather than a tax on every line.

**Status:** design + **Rust-first implementation underway.** The design corpus spans Foundation,
RFC-0001…0039, ADR-001…034, DN-01 onward — per-document status (Draft / Proposed / Accepted /
Enacted / Resolved) is in [`docs/Doc-Index.md`](docs/Doc-Index.md). The Rust workspace has
**57 crates** (+ `xtask`) <!-- doc-currency:crate-count --> — a trusted reference interpreter,
explicit representation **swaps** (certified at the `certified` mode), the selection-policy
engine, a verified-numerics layer, a **Rust-first standard library**, an L1 surface with
**generics · traits · higher-order functions · operator syntax**, a **runtime** (scheduler,
structured concurrency) with a **three-layer hybrid memory model**, and a **native AOT compiler**
(`crates/mycelium-mlir`) that now lowers recursion, closures, `Swap`, Dense, and VSA — landed
(epic E25-1), with full-language coverage still in progress (ADR-034). Per the transparency rule,
no claim here is upgraded beyond what a checked basis supports (VR-5). See
[Status & roadmap](docs/guide/status-and-roadmap.md) for the honest, itemized state of every
in-progress piece.

> **Direction note (ADR-032, Enacted 2026-06-24).** The north star has been **repositioned** from
> the original "certified-everything substrate" premise toward **a fast, memory-safe, ergonomic
> multi-paradigm language**, with certification/transparency as **optional, tunable** capabilities
> (RFC-0034: `fast` default · `balanced` · `certified`). Memory-safety, speed, and ergonomics are
> now **first-class goals** alongside the transparent-swap thesis. The "honesty rule" is reframed
> as the **transparency & auditability rule** (mechanism unchanged — see
> [Guarantees & verification](docs/guide/guarantees-and-verification.md)).

---

## Contents

- [Why this exists, and the core ideas](docs/guide/why-and-design.md)
- [Guarantees & verification](docs/guide/guarantees-and-verification.md) — the
  `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` lattice and the split verification regime
- [Workspace map](docs/guide/workspace-map.md) — all 50 crates, the proof artifacts, the
  LLM-leverage experiment
- [How Mycelium compares](docs/guide/comparisons.md) — vs. typed systems languages, ML/array
  languages, VSA/HDC libraries, verification-oriented languages
- [Repository structure](docs/guide/repository-structure.md) — the directory map
- [Status & roadmap](docs/guide/status-and-roadmap.md) — what's built, what's in progress
  (including the native AOT state), the technology stack
- [Decisions & reading order](docs/guide/decisions-and-reading-order.md) — the load-bearing
  decision table and a suggested path through the corpus
- [Glossary (README scope)](docs/guide/glossary.md) — short local terms; the canonical lexicon is
  [`docs/Glossary.md`](docs/Glossary.md)
- [Contributing conventions & provenance](docs/guide/contributing-and-provenance.md) — the short
  version; full process in [`CONTRIBUTING.md`](CONTRIBUTING.md)

Other key entry points: [`docs/Mycelium_Project_Foundation.md`](docs/Mycelium_Project_Foundation.md)
(the charter), [`docs/Doc-Index.md`](docs/Doc-Index.md) (the whole-corpus map), and
[`CLAUDE.md`](CLAUDE.md) (the operating guide / house rules for agents working in this repo).

---

## Why this exists, in one paragraph

Modern computing keeps four representation families in separate worlds: bits for traditional
computation, dense embeddings for ML, hypervectors for symbolic-connectionist work, and balanced
ternary as a recurring "what if" in hardware. Moving between them is where correctness quietly
leaks — conversions are implicit, lossy in undocumented ways, and impossible to audit. Mycelium's
thesis is that the **representation-swap** should be the explicit, verifiable, first-class
operation of the language. Full rationale and the core design commitments:
[Why & design](docs/guide/why-and-design.md).

## The core ideas, at a glance

- **Representation is part of the type** — `Binary{width}`, `Ternary{trits}`, `Dense{dim,dtype}`,
  `VSA{model,dim,sparsity}` are distinct type families; there is **no implicit conversion**.
- **`Swap` is the only representation-changing operation**, and every swap emits a certificate
  describing exactly what the conversion cost.
- **Transparency is a typed, monotone property** — the guarantee lattice
  `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` travels with every value and only ever degrades.
- **Selection policies are reified, `EXPLAIN`-able artifacts** — no black-box "intelligent"
  behavior anywhere in the kernel.
- **Definitions are content-addressed** — identity is the content hash; names are metadata.

Full detail: [Why & design](docs/guide/why-and-design.md). Repository layout at a glance:
[Repository structure](docs/guide/repository-structure.md).

## Quickstart

```sh
just            # list recipes
just setup      # best-effort install of the check tools
just check      # the FULL suite — exactly what CI runs (build · clippy · test · docs · proofs · supply-chain)
just fmt        # auto-format (Rust + Python)
just docs-index # regenerate docs/api-index/ after a public-API change
```

Checks **skip gracefully** when a tool isn't present. Remote CI
(`.github/workflows/checks.yml`) is **manual-dispatch only and advisory**, running the same
`just ci` — see [`CONTRIBUTING.md`](CONTRIBUTING.md).

Worked examples live in [`examples/`](examples/) (`hello-phylum`, `repr-tour`); the reference
interpreter and kernel are in [`crates/mycelium-interp`](crates/mycelium-interp/README.md) and
[`crates/mycelium-core`](crates/mycelium-core/README.md) — see the
[workspace map](docs/guide/workspace-map.md) for the full crate-by-crate tour.

## What is built (short version)

The Core IR + Rust reference interpreter; the certified binary↔ternary swap (Z3-proved); the
verified-numerics layer; Dense/VSA breadth with per-model guarantee matrices; the
selection-policy engine + EXPLAIN; a **native AOT compiler** covering recursion, closures,
`Swap`, Dense, and VSA (epic E25-1, `Empirical` via a checked three-way differential); JIT
including dynamic-VSA/HDC JIT; hot-inject; the L1 surface (generics/traits/effects); the
runtime/concurrency model with a three-layer hybrid memory model; the full toolchain suite; and a
Rust-first standard library (25/25 crate specs `Accepted`). Full crate-by-crate detail:
[Workspace map](docs/guide/workspace-map.md). Full honest status (including what's still open on
the AOT full-coverage gate, ADR-034): [Status & roadmap](docs/guide/status-and-roadmap.md).

**Not yet established:** self-hosting (the Mycelium-lang stdlib migration) — post-1.0/1.x scope.
Surface-language ratification is likewise scoped to a tracked `1.x`.

---

## License

MIT — Copyright (c) 2026 **Tyler Zervas**. See [`LICENSE`](./LICENSE).
