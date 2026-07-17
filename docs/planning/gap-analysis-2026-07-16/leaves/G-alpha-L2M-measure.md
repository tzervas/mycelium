# L2-M brief — Empirical remeasure post G-α (measure leaf)

## Identity

- **Leaf:** G-alpha-L2M-measure
- **Branch:** `claude/leaf/G-alpha-measure`
- **Model assigned:** `grok-composer-2.5-fast` (record actual if unavailable)
- **Base:** tip after L2-A/B PRs open (or after L0 merges if sequential)
- **PR base:** `dev` — **do not merge**

## Mission

Run real `myc check` / `mycelium-transpile --vet` on:

- default-5 (eval, fuse, time, rand, cmp)
- std-fs, std-io

Write Empirical table + ranked residual list for G-β.

## Owns

- `docs/planning/gap-analysis-2026-07-16/M1006-remeasure-post-G-alpha-*.md`
- `experiments/results/m1006-remeasure-post-g-alpha/`

## Does not own

- Product code; shared CHANGELOG/issues/Doc-Index (FLAG)

## Template

Mirror `M1006-remeasure-post-C3C4-2026-07-16.md` structure: per-target table, first-poison list, phylum dual-report, rank residual, FLAGs. **No one-shot claim.**

## When to run

After L2-A and L2-B open PRs (or L0 merges). If code leaves still open, measure tip+PR SHAs and note open PRs.
