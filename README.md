# Mycelium

> A programming language that treats **traditional binary**, **balanced ternary**, **dense embeddings**, and **Vector Symbolic Architectures (VSA / hyperdimensional computing)** as co-equal, first-class substrates — under semantics that are **transparent** (no hidden behavior), **metadata-native**, and **amenable to formal reasoning**.

**Status:** design phase. Both research passes are complete and the full design corpus (Foundation + RFC-0001…0005 + ADR-010 + DN-01) is **Accepted/Resolved**. The next step is implementation, plus one confirming proof-of-concept.

*(Formerly named **Verid** — retained only as a provenance note.)*

## Why this exists

Modern computing keeps these four representations in separate worlds: bits for traditional computation, embeddings for ML, hypervectors for symbolic-connectionist work, and balanced ternary as a recurring "what if" in hardware. Moving between them is where correctness quietly leaks — conversions are implicit, lossy in undocumented ways, and impossible to audit.

Mycelium's thesis is that the **representation-swap** should be the explicit, verifiable, first-class operation of the language. The central design problem is therefore **metadata-native, explicit, verifiable swapping between substrates** — with every approximation disclosed, bounded, and tagged by how trustworthy that bound is.

Three non-negotiables shape every decision:
1. **No hidden / opaque behavior** in core semantics.
2. **Human-intelligible *and* useful for AI agents** (a dual-intelligibility goal).
3. **Formally reasoning-amenable** — "no black boxes" is realized as mechanically-checkable invariants, not a slogan.

## The core ideas (in one screen)

- **Representation is part of the type.** `Binary{width}`, `Ternary{trits}`, `Dense{dim,dtype}`, `VSA{model,dim,sparsity}` are distinct type families. There is **no implicit conversion** between paradigms.
- **`Swap` is the only representation-changing operation**, and every swap emits a **certificate** describing exactly what the conversion cost.
- **Honesty is a typed, monotone property.** A guarantee lattice — **`Exact ⊃ Proven ⊃ Empirical ⊃ Declared`** — travels with every value and degrades by *meet* through operations, so a disclosed guarantee can never spuriously strengthen.
- **Metadata is self-describing and survives lowering** (Apache-Arrow-grade): provenance, bounds, layout, and reconstruction info are queryable at runtime and exposed to tooling.
- **Split verification regime:** binary↔ternary swaps are *provable/bijective-within-range*; VSA/embedding swaps carry *bounded/probabilistic* per-instance certificates (translation-validation style).
- **Physical packing is a _schedule_, not a type.** Lossless layout (e.g. ternary packing) is chosen at a lowering stage and *recorded* as inspectable metadata — values of the same logical type stay interoperable whether packed or not.

## Repository structure

```
mycelium/
├── README.md
├── LICENSE
├── .gitignore
├── .gitleaks.toml
├── docs/
│   ├── research/                    ← research records
│   ├── rfcs/
│   ├── adr/
│   └── notes/
└── (future: src/, tests/, etc.)
```

## Key decisions at a glance

See `docs/Mycelium_Project_Foundation.md` for the full charter, requirements, ADRs, and roadmap.

## Status

The design corpus is complete and accepted. Work is now in **build + confirm** phase.

## License

MIT — Copyright (c) 2026 Tyler Zervas.
