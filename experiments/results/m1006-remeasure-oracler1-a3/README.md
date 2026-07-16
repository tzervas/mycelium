# M-1006 pilot remeasure — ORACLE-R1 A3

**Tip:** `6d61b3b8` (origin/dev, A1 #1647 + A2 #1648)
**Date:** 2026-07-16
**Narrative + tables:** `docs/planning/gap-analysis-2026-07-16/M1006-remeasure-post-A1A2-2026-07-16.md`

## Headline union (default 5 targets)

| Metric | Value |
|--------|------:|
| checked (myc-check-clean) | **20 / 236 = 8.5%** |
| expressible (emitted) | **44 / 236 = 18.6%** |

## Per-target

| Target dir | checked_fraction | expressible_fraction | class | poison |
|------------|-----------------:|---------------------:|-------|--------|
| `crates_mycelium-l1_src_eval_rs` | 0.0% | 16.7% | CheckError | `unknown name DEFAULT_FUEL` |
| `crates_mycelium-l1_src_fuse_rs` | 0.0% | 0.0% | Clean (0 emit) | — |
| `crates_mycelium-std-time_src` | 0.0% | 45.9% | CheckError | `no instance Show for WallInstant` |
| `crates_mycelium-std-rand_src` | 17.6% | 17.6% | Clean | — |
| `crates_mycelium-std-cmp_src` | 12.6% | 12.6% | Clean | — |

Ground truth per target: `*/vet.json` + `*/vet-stdout.txt` (regenerable via `just transpile-vet`).
Large `*.gap.json` / full `.myc` dumps intentionally omitted from commit size; re-run to regenerate.
