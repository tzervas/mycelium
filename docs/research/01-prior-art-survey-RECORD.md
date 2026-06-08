# Research Record 01 — Prior-Art Survey & Synthesis (Pass 1)

> **What this file is.** A durable record of the first research pass: its scope, the structure of its findings, the decisions it drove, and its source base. The *full narrative* survey (≈6k words, with inline citations) was delivered as a conversation artifact; this record preserves the parts a repository needs.

## Scope
Survey the prior art across the six areas Mycelium sits astride, then synthesize: identify what already exists, where the genuine gaps are, and what tensions any unifying design must resolve. Six areas:
1. Programming-language design & IRs (typed, inspectable, multi-level).
2. Vector Symbolic Architectures / Hyperdimensional Computing (VSA/HDC).
3. Balanced ternary representation and hardware/software history.
4. Self-describing data & metadata-native formats.
5. Formal verification & verified numerics.
6. AI-assisted / LLM-leveraged programming.

## Output structure (labels used throughout the corpus)
- **Gaps G1–G11** — the unmet needs.
- **Cross-cutting tensions A–E** — design forces in conflict.
- **Recommendations R1–R8** — the actionable conclusions that seeded the Foundation and RFCs.

## Key findings (condensed)
- **No existing system unifies** binary + balanced ternary + dense embeddings + VSA under transparent, metadata-native, verifiable semantics.
- **PL/IR substrate exists:** MLIR, Unison, Apache Arrow — these became Mycelium's architectural anchors.
- **VSA is powerful but capacity/crosstalk-limited:** forcing an *honesty* distinction (guarantee lattice).
- **Balanced ternary** is elegant but has no sustained modern ecosystem; its value here is *logical*.
- **Verified numerics** establish the "ideal-real spec + certified error" pattern.

## Tension B resolution
Reframed "no hidden approximation" as: every approximate operation = an exact deterministic spec + a declared/proven bound, tagged by guarantee strength.

## Decisions this pass drove
- Guarantee lattice `Exact ⊃ Proven ⊃ Empirical ⊃ Declared` → RFC-0001.
- Architectural anchors (MLIR + Unison + Arrow).
- Split verification regime → ADR-002, RFC-0002.
- VSA in core but as an optional submodule → ADR-008, RFC-0003.

## Key sources
Kleyko et al. surveys on HDC/VSA; Lattner et al. on MLIR; Unison and Arrow specifications; Darulova et al. on Rosa/Daisy; Kennedy on units-of-measure; Vazou et al. on Liquid Haskell; Knuth TAOCP Vol. 2.
