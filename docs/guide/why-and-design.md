# Why Mycelium exists, and the core design ideas

One-line purpose: the motivating problem the language solves, and the core design commitments
that follow from it.

## Contents

- [Why this exists](#why-this-exists)
- [The core ideas](#the-core-ideas)

## Why this exists

Modern computing keeps four representation families in separate worlds: bits for traditional
computation, dense embeddings for ML, hypervectors for symbolic-connectionist work, and balanced
ternary as a recurring "what if" in hardware. Moving between them is where correctness quietly
leaks — conversions are implicit, lossy in undocumented ways, and impossible to audit.

Mycelium's thesis is that the **representation-swap** should be the explicit, verifiable,
first-class operation of the language. The central design problem is therefore
**metadata-native, explicit, verifiable swapping between substrates** — with every approximation
disclosed, bounded, and tagged by how trustworthy that bound is.

Three non-negotiables shape every decision:

1. **No hidden / opaque behavior** in core semantics.
2. **Human-intelligible *and* useful for AI agents** (a dual-intelligibility goal).
3. **Formally reasoning-amenable** — "no black boxes" is realized as mechanically-checkable
   invariants, not a slogan.

## The core ideas

- **Representation is part of the type.** `Binary{width}`, `Ternary{trits}`, `Dense{dim,dtype}`,
  `VSA{model,dim,sparsity}` are distinct type families. There is **no implicit conversion**
  between paradigms.
- **`Swap` is the only representation-changing operation**, and every swap emits a **certificate**
  describing exactly what the conversion cost — bijective for binary↔ternary, bounded/probabilistic
  for ↔VSA/embedding (the split verification regime, ADR-002).
- **Transparency is a typed, monotone property.** A guarantee lattice —
  **`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`** — travels with every value and degrades by *meet*
  through operations, so a disclosed guarantee can never spuriously strengthen.
- **Metadata is self-describing and survives lowering** (Apache-Arrow-grade): provenance, bounds,
  layout, and reconstruction info are queryable at runtime and exposed to tooling.
- **All four substrates are co-equal, first-class.** Binary and balanced ternary share the kernel
  type system; dense embeddings and VSA/HDC are not optional add-ons — they participate in the same
  type + swap + certificate machinery, with VSA packaged as an optional-but-first-class submodule
  (ADR-008).
- **Physical packing is a *schedule*, not a type.** Lossless layout (e.g., ternary packing) is
  chosen at a lowering stage and *recorded* as inspectable metadata — values of the same logical
  type stay interoperable whether packed or not (DN-01, RFC-0004 §5).
- **Selection policies are reified, EXPLAIN-able artifacts.** Any policy-driven selection is a
  first-class, queryable value; every swap records the `PolicyRef` it used (ADR-006, RFC-0005).
- **Definitions are content-addressed.** Identity is the content hash; names are metadata
  (Unison-style, ADR-003). A stable component is content-addressed + spec-ratified +
  verification-passed, and only then eligible for AOT compilation.

For how these ideas cash out as a concrete guarantee lattice and verification regime, see
[Guarantees & verification](guarantees-and-verification.md).

---

**See also:** [Guarantees & verification](guarantees-and-verification.md) ·
[How Mycelium compares](comparisons.md) · [Workspace map](workspace-map.md)

[← Back to README](../../README.md)
