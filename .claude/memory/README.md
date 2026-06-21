# Mycelium — Component Memory Files

Compact, agent-oriented orientation per key component. **Load the relevant file before deep work
on that component** — it's faster than re-deriving from the corpus, and points you at the exact
crates/specs to `Read`. These are *navigational/orientation* aids (Empirical/Declared): **source +
the normative corpus are ground truth** (CLAUDE.md, the RFC/ADR/DN, the crate). Not normative.

Each file follows the same shape: **What it is · Where it lives (crates + key files) · Key types &
operations · Key invariants (honesty) · Read-more entry points · Gotchas.**

| Memory file | Component | Primary crates | Normative source |
|---|---|---|---|
| [`value-model.md`](./value-model.md) | The unified value model: `Value<Repr,Meta>`, the 4 substrates, the guarantee lattice, content-addressing | `mycelium-core` | RFC-0001, ADR-003 |
| [`honesty-model.md`](./honesty-model.md) | The cross-cutting honesty system: lattice, VR-5, never-silent G2, EXPLAIN, append-only | (all) | CLAUDE.md §rules, Foundation |
| [`swaps-certificates.md`](./swaps-certificates.md) | Certified never-silent representation swaps + translation-validation checker + split regime | `mycelium-cert` | RFC-0002, ADR-002/010 |
| [`numerics-dense.md`](./numerics-dense.md) | Verified numerics (ε/δ bound kernels) + dense embedding ops + Proven/Empirical bounds | `mycelium-numerics`, `mycelium-dense` | ADR-010/011, RFC-0001 §4.1 |
| [`vsa.md`](./vsa.md) | VSA/HDC: models (MAP/BSC/HRR/FHRR/SBC), bind/bundle/unbind/cleanup, capacity bounds, resonator | `mycelium-vsa` | RFC-0003/0009 |
| [`selection-explain.md`](./selection-explain.md) | Reified, inspectable selection policies + the mandatory EXPLAIN trace | `mycelium-select` | RFC-0005, ADR-006 |
| [`language-execution.md`](./language-execution.md) | L1 surface + grammar + parser, Core IR, the reference interpreter (trusted base), MLIR→LLVM AOT, hot-inject | `mycelium-l1`, `mycelium-interp`, `mycelium-mlir` | RFC-0004/0007/0011, ADR-009/016/017 |
| [`toolchain.md`](./toolchain.md) | The tool crates + `just check` + `just docs-site` | `myc-check`, `mycfmt`, `myc-lint`, `myc-sec`, `myc-doc`, `spore`, `bench`, `lsp`, `build`, `proj` | RFC-0013, the contract specs |
| [`stdlib.md`](./stdlib.md) | The 25 `mycelium-std-*` crates: rings/tiers, per-op guarantee matrices, the `runtime` phylum | `mycelium-std-*` | RFC-0016, DN-07/DN-16 |
| [`experiments-llm.md`](./experiments-llm.md) | KC-2/M-381 LLM-leverage: the harness, the ablation arms + the LlmCanonical→L1 bridge, the determinate retention ratio, RP-8 | `tools/llm-harness`, `mycelium-bench` | DN-09, RFC-0021, RP-8 |
| [`lang-lexicon-syntax.md`](./lang-lexicon-syntax.md) | Mycelium lexicon, surface syntax, grammar & naming conventions: reserved-word table, generic→Mycelium mapping, naming law (three-test gate), layer cake, `// nodule:` header, `mycelium-proj.toml`, example snippets | `crates/mycelium-l1/src/token.rs`, `docs/spec/grammar/mycelium.ebnf` | RFC-0006/0007/0020, DN-02/03/06, Glossary |

**Maintenance:** when a component changes materially, refresh its memory file (append-only is not
required here — these are living orientation aids, but keep them honest + currency-accurate; cite
the source so drift is catchable). This README is the only index of the set; it is intentionally
not registered in `docs/Doc-Index.md` (that map tracks the normative corpus, not these agent aids).
