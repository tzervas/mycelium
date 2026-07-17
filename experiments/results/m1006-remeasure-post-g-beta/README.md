# m1006-remeasure-post-g-beta

Empirical `mycelium-transpile --vet` artifacts after G-β (#1700 no-fabricate method-call)
landed on `dev` (tip `4acd3a20`, with #1699 D1).

| Path | Target |
|------|--------|
| `eval/` | `crates/mycelium-l1/src/eval.rs` |
| `fuse/` | `crates/mycelium-l1/src/fuse.rs` |
| `time/` | `crates/mycelium-std-time/src` |
| `rand/` | `crates/mycelium-std-rand/src` |
| `cmp/` | `crates/mycelium-std-cmp/src` |
| `fs/` | `crates/mycelium-std-fs/src` |
| `io/` | `crates/mycelium-std-io/src` |

Each directory holds `.myc` emissions, `.gap.json`, and `vet.json` (plus crate
`summary.json` / `union.gap.json` / `REMAP.md` where batch mode applies).

**Headline (Empirical):** all-7 `checked_fraction` **28.7%** (98/342), **+3.0pp** vs
post-G-α combined 25.7%; std-io **Clean×5** / 40.7%; **zero** CheckError on the pilot set.

**Write-up:**
[`docs/planning/gap-analysis-2026-07-16/M1006-remeasure-post-G-beta-2026-07-17.md`](../../../docs/planning/gap-analysis-2026-07-16/M1006-remeasure-post-G-beta-2026-07-17.md)

Honesty: fractions are Empirical; residual ranking is Declared. No one-shot claim.
