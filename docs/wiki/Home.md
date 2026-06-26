# Mycelium

A **fast, memory-safe, ergonomic** multi-paradigm language that treats **traditional binary**,
**balanced ternary**, **dense embeddings**, and **Vector Symbolic Architectures (VSA / HDC)** as
co-equal, first-class substrates — under semantics that are **transparent** (no hidden behavior) and
**metadata-native**, with **certification & auditability as *optional, tunable* capabilities**
(`fast` by default · `certified` on request) rather than a tax on every line.

> This wiki is generated from the in-repo source under
> [`docs/wiki/`](https://github.com/tzervas/mycelium/tree/main/docs/wiki) and the per-crate READMEs.
> The authoritative design corpus lives in
> [`docs/`](https://github.com/tzervas/mycelium/tree/main/docs); per-document status is in
> [`docs/Doc-Index.md`](https://github.com/tzervas/mycelium/blob/main/docs/Doc-Index.md).

## Start here

- **[Getting Started](Getting-Started)** — clone, build, and the `just` check loop.
- **[Architecture](Architecture)** — how the 50 crates fit together (kernel → compiler → runtime →
  stdlib → tooling), the value model, swaps, and execution paths.
- **[Crate Index](Crate-Index)** — every crate with a one-line purpose and a link to its README.
- **[API Reference](API-Reference)** — building and browsing the rustdoc + the agent API index.

## Core concepts

- **[Memory Model](Memory-Model)** — the three-layer hybrid (affine ownership → optimized reference
  counting → region reclamation), never-silent reclamation, and the static RC-elision passes
  (DN-32 / RFC-0027 / MEM-4 DN-33).
- **[Tunable Certification](Tunable-Certification)** — the `fast` / `balanced` / `certified` modes,
  the transparency & auditability rule, and the `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` lattice
  (RFC-0034 / ADR-032).
- **[Decision Records](Decision-Records)** — the RFC / ADR / DN index and how decisions evolve
  (append-only).

## What this is

Modern computing keeps four representation families in separate worlds — bits, dense embeddings,
hypervectors, and balanced ternary — and moving between them is where correctness quietly leaks.
Mycelium makes the **representation-swap** an explicit, verifiable, first-class operation: every
approximation is disclosed, bounded, and tagged by how trustworthy that bound is. Around that thesis
it is built to be a *usable systems language* — memory-safe, fast, and ergonomic — with the heavy
certification machinery engaged only where it earns its cost.

## Status

Design + **Rust-first implementation underway.** The Rust workspace has **50 crates** (a trusted
reference interpreter, certified swaps, a verified-numerics layer, a Rust-first standard library, an
L1 surface, a runtime with the three-layer memory model, and the static RC-elision passes). Per the
transparency rule, no claim is upgraded beyond what a checked basis supports (VR-5).
