# Research Record 01 — Prior-Art Survey & Synthesis (Pass 1)

> **What this file is.** A durable record of the first research pass: its scope, the structure of its findings, the decisions it drove, and its source base. The *full narrative* survey (≈6k words, with inline citations) was delivered as a conversation artifact; this record preserves the parts a repository needs. Ask the maintainer (or re-run the assistant) to drop the full narrative in here if a self-contained copy is wanted.

## Scope
Survey the prior art across the six areas Mycelium sits astride, then synthesize: identify what already exists, where the genuine gaps are, and what tensions any unifying design must resolve. Six areas:
1. Programming-language design & IRs (typed, inspectable, multi-level).
2. Vector Symbolic Architectures / Hyperdimensional Computing (VSA/HDC).
3. Balanced ternary representation and hardware/software history.
4. Self-describing data & metadata-native formats.
5. Formal verification & verified numerics.
6. AI-assisted / LLM-leveraged programming.

## Output structure (labels used throughout the corpus)
- **Gaps G1–G11** — the unmet needs (e.g., G2 selection-as-black-box, G3 ternary ecosystem, G4 compositional-VSA limits, G5 proven-vs-empirical bound honesty, G6 reconstruction manifest, G7 honest-bound feasibility, G8 verified-numerics trusted base, G10 LLM leverage on novel syntax, G11 semantic projection ergonomics).
- **Cross-cutting tensions A–E** — design forces in conflict (notably **B**: "no statistical approximation" vs first-class approximate VSA).
- **Recommendations R1–R8** — the actionable conclusions that seeded the Foundation and RFCs.

## Key findings (condensed)
- **No existing system unifies** binary + balanced ternary + dense embeddings + VSA under transparent, metadata-native, verifiable semantics. The *components*, however, are individually mature.
- **PL/IR substrate exists:** MLIR (multi-level, inspectable, progressive lowering), Unison (content-addressed definition identity, names-as-metadata), Apache Arrow (self-describing schema travelling with data). These became Mycelium's architectural anchors.
- **Type-level representation tracking works** (F# units-of-measure; dependent/refinement types — Idris, Agda, Liquid Haskell, F\*), but mainstream systems *erase* it at runtime — the anti-pattern Mycelium rejects (metadata must persist).
- **VSA is powerful but capacity/crosstalk-limited:** bundling/binding have real information-theoretic limits; some bounds are proven, others only Gaussian-approximate — forcing an *honesty* distinction (→ guarantee lattice). True compositional reconstruction differs from indexed retrieval.
- **Balanced ternary** is elegant (symmetric, sign-as-digit) but has no sustained modern ecosystem; its value here is *logical*, with forward-compatibility to native-ternary hardware.
- **Verified numerics prove *bounds*, not equalities:** Gappa, FPTaylor, Flocq, Rosa/Daisy, Herbie establish the "ideal-real spec + certified error" pattern — exactly the model Mycelium's bounds encode (later ADR-010).

## Tension B resolution (seeded here, ratified in ADR-001)
Reframed "no hidden approximation" as the operating rule: every approximate operation = an exact deterministic spec + a declared/proven bound, tagged by guarantee strength. This dissolved the apparent contradiction between the "no black boxes" principle and first-class VSA.

## Decisions this pass drove (pointers)
- Guarantee lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` → RFC-0001 §3.4/§4.7.
- Architectural anchors (MLIR + Unison + Arrow) → Foundation §5, RFC-0001, RFC-0004.
- Split verification regime (provable binary↔ternary; bounded/probabilistic VSA) → ADR-002, RFC-0002.
- VSA in core but as an optional submodule → ADR-008, RFC-0003.
- The four survey coverage gaps that Pass 2 then closed (array languages; neurosymbolic IRs; verified probabilistic numerics; Rust VSA/ternary ecosystem).

## Key sources
(Representative; full inline citations are in the conversation artifact.)
- Kleyko, Rahimi, Gayler, Sommer — *A Survey on Hyperdimensional Computing / VSA* (two-part, ACM Computing Surveys).
- Schlegel, Neubert, Protzel — *A comparison of Vector Symbolic Architectures*.
- Frady, Kleyko, Sommer — VSA capacity / sequence-indexing work (Neural Computation).
- Lattner et al. — *MLIR: A Compiler Infrastructure for the End of Moore's Law*.
- Unison language documentation (content-addressed code).
- Apache Arrow columnar/format specification.
- Kennedy — units-of-measure in F#; dimensional type-system literature.
- Vazou et al. — Liquid Haskell / refinement types; F\* project.
- Darulova & Kuncak — Rosa; Darulova et al. — Daisy.
- de Dinechin et al. — Gappa; Solovyev et al. — FPTaylor; Boldo & Melquiond — Flocq.
- Panchekha et al. — Herbie (PLDI 2015).
- Knuth — *TAOCP* Vol. 2 (balanced ternary); Setun historical record.
