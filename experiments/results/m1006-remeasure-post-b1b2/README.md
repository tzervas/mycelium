# M-1006 remeasure post B1+B2 — experiment results (2026-07-16)

**Tip measured:** `origin/dev` @ `788574ab` (post B1 #1659 + B2 #1660 + PM close-out #1661).
**Write-up:** `docs/planning/gap-analysis-2026-07-16/M1006-remeasure-post-B1B2-2026-07-16.md`
**Honesty:** `Empirical` numbers from `mycelium-transpile --vet` + real `myc-check`. **Not** a one-shot claim.

## Targets

Same set as `m1006-baseline-oneshot` (default-5 + std-fs + std-io):

| Dir | Source |
|-----|--------|
| `crates_mycelium-l1_src_eval_rs/` | `crates/mycelium-l1/src/eval.rs` |
| `crates_mycelium-l1_src_fuse_rs/` | `crates/mycelium-l1/src/fuse.rs` |
| `crates_mycelium-std-time_src/` | `crates/mycelium-std-time/src` |
| `crates_mycelium-std-rand_src/` | `crates/mycelium-std-rand/src` |
| `crates_mycelium-std-cmp_src/` | `crates/mycelium-std-cmp/src` |
| `crates_mycelium-std-fs_src/` | `crates/mycelium-std-fs/src` |
| `crates_mycelium-std-io_src/` | `crates/mycelium-std-io/src` |

Each target directory holds `vet.json` + `vet-stdout.txt` / `vet-stderr.txt` (lean artifact set).

## Headlines

| Set | checked | expressible |
|-----|--------:|------------:|
| default-5 union | **19.5%** (46/236) | **19.5%** (46/236) |
| all-7 union | **17.0%** (58/342) | **27.5%** (94/342) |

**Δ vs baseline-oneshot (`2ac85a84`):** **0.0pp** on both unions (fractions identical).

B1 form check: Import diagnostics now cite full nodule paths (`std.fs.error.*` / `std.io.error.*`) vs baseline short paths (`error.*`). Residual remains name-not-declared under single-file oracle; phylum mode recovers some io files (+3.4pp basis).

See `summary.txt` for the raw driver log.
