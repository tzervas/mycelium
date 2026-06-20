# Mycelium Grok co-authoring — cross-model comparison

> **SYNTHETIC (self-test).** Produced by the deterministic offline mock client — NOT a real model measurement. Carries no quality evidence (VR-5). Plumbing only.

- run id: `SAMPLE`
- mode: `self-test`
- generated: `SAMPLE-DETERMINISTIC`
- guarantee lattice: Exact ⊐ Proven ⊐ Empirical ⊐ Declared
- model-allowed tags: Empirical, Declared (never Proven/Exact — VR-5)

| model | mode | syntactic-valid | type-check pass | mean edit-to-fix | tokens (in/out) | mean latency s | total cost |
|---|---|---|---|---|---|---|---|
| `grok-4.3` | self-test | 100.0% | 100.0% | 1.33 | 265/113 | 0.00 | $0.000614 |

## `grok-4.3`

- quality tag: **Declared (synthetic)**
- endpoint: `mock (offline)`
- task set: `gold-compose-v1`  seed: `42`

### Retention-ratio ablation (M-381)
- retention ratio: **n/a** (threshold ~0.7)
- INDETERMINATE — pending run: arm 4 (LlmCanonical) did not run. The threshold comparison applies only when arm 4 is present (research/11 §T11.7 step 3).
- leverage claim tag: **Declared (open — pending full campaign)**

| arm | ran | pass@1 | note |
|---|---|---|---|
| `arm1-bare-novel-surface` | yes | 100.0% | bare novel text surface |
| `arm2-grammar-primer` | yes | 100.0% | + book-quality grammar-in-context primer |
| `arm3-grammar-constrained-decoding` | **blocked** | n/a | needs a grammar-constrained decoder integration (M-380); not built yet, and the OpenAI-compatible REST surface does not expose GBNF. Not fabricated (VR-5). |
| `arm4-llm-canonical-projection` | **blocked** | n/a | needs the LlmCanonical projection renderer over mycelium-core (M-380 / T11.4); not built yet. The retention-ratio DENOMINATOR — so the threshold comparison is indeterminate until this arm runs (research/11 §T11.7 step 3). Not fabricated (VR-5). |
| `arm5-embedded-dsl-baseline` | **blocked** | n/a | needs an embedded-DSL baseline harness (RR-3); out of scope for the Grok harness wiring. Not fabricated (VR-5). |
