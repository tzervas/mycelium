# README-scope glossary

One-line purpose: the handful of terms used across this landing page and its guide docs, kept
short. This is a *subset*, not the canonical reference — see
[`docs/Glossary.md`](../Glossary.md) for the full fungal lexicon (phylum/nodule/spore/hypha/colony)
and architecture terms, and [`.claude/memory/lang-lexicon-syntax.md`](../../.claude/memory/lang-lexicon-syntax.md)
for the reserved-words/surface-syntax reference.

- **Substrate / paradigm** — one of the four representation families (binary, balanced ternary,
  dense embedding, VSA).
- **Balanced ternary** — base-3 with digits {−1, 0, +1}; symmetric, sign-is-a-digit. Used here
  as a *logical* substrate, forward-compatible with native-ternary hardware.
- **VSA / HDC** — Vector Symbolic Architectures / hyperdimensional computing: high-dimensional
  vectors with algebraic operations (bind, bundle, permute) for symbolic-connectionist computation.
- **Swap** — the explicit, certificate-emitting operation that changes a value's representation.
  The only such operation.
- **Guarantee lattice** — `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`: how trustworthy a value's
  accuracy claim is; composes by *meet* (weakest wins).
- **Certificate** — a checkable record that a swap (or a compiled artifact vs. the interpreter)
  meets a claimed `{ε, δ, strength}` bound.
- **Translation validation** — proving each *instance* of a lowering/conversion correct, rather
  than proving the whole compiler correct once.
- **Schedule-staged packing** — choosing a lossless physical layout at a lowering stage (a
  "schedule"), recorded as inspectable metadata, not encoded in the type.
- **`ErrorBound` / `ProbBound`** — the two ADR-010 bound kernels: error-magnitude (ε) via affine
  arithmetic; failure-probability (δ) via the union bound / approximate couplings.
- **Reconstruction manifest** — the explicit recipe (model, codebooks, compositional structure,
  decoding procedure, bound) needed to recover content from a VSA representation; distinguishes
  indexed retrieval from true compositional reconstruction.
- **Stable component** — a definition that is content-addressed, spec-ratified, and
  verification-passed, and therefore eligible for AOT compilation.
- **EXPLAIN** — a first-class, queryable artifact that records why a selection, conversion, or
  approximation was made; required for any policy-driven or approximate operation (ADR-006,
  RFC-0005).

---

**See also:** [Why & design](why-and-design.md) · [Guarantees & verification](guarantees-and-verification.md)

[← Back to README](../../README.md)
