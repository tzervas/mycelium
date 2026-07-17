# L2-M brief — Empirical remeasure post G-β (measure leaf)

## Identity

- **Leaf:** G-beta-L2M-measure
- **Branch:** `claude/leaf/G-beta-remeasure`
- **Model actual:** Grok 4.5 (xAI agent runtime)
- **Base:** `origin/dev` tip ≥ `4acd3a20` (must include #1700 G-β and #1699 D1)
- **PR base:** `dev` — **do not merge**

## Mission

Run real `myc check` / `mycelium-transpile --vet` on:

- default-5 (eval, fuse, time, rand, cmp)
- std-fs, std-io

Write Empirical table + ranked residual list for G-γ + G1 gate assessment.

## Owns

- `docs/planning/gap-analysis-2026-07-16/M1006-remeasure-post-G-beta-*.md`
- `experiments/results/m1006-remeasure-post-g-beta/`
- this brief (optional)

## Does not own

- Product code; shared CHANGELOG/issues/Doc-Index (FLAG)

## Result (filled by leaf)

- **Write-up:** [`../M1006-remeasure-post-G-beta-2026-07-17.md`](../M1006-remeasure-post-G-beta-2026-07-17.md)
- **Tip:** `4acd3a20d9272660e11fcc8b3dceec1c163f0c48`
- **Headline:** all-7 checked **28.7%** (+3.0pp vs post-G-α 25.7%); std-io Clean×5 / 40.7%; zero CheckError
- **G1 pilot:** **PASS** (file Clean + EXPLAIN/ranked residual; no closable unknown first-poison)
- **Next residual rank 1:** MacroInvocation (M-875 design-gated) or Rank-A method cascade (`ok_or`); measure: full M-1006 ladder
- **No one-shot claim**

## Template

Mirror `M1006-remeasure-post-G-alpha-2026-07-17.md` structure: per-target table, first-poison
list, phylum dual-report, rank residual, FLAGs. **No one-shot claim.**
