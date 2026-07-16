# M-1006 ONESHOT baseline — experiment results (2026-07-16)

**Tip measured:** `origin/dev` @ `2ac85a84` (post ORACLE-R1 A1–A5).
**Write-up:** `docs/planning/gap-analysis-2026-07-16/M1006-baseline-oneshot-2026-07-16.md`
**Honesty:** `Empirical` numbers from `mycelium-transpile --vet` + real `myc-check`. **Not** a one-shot claim.

## Targets

| Dir | Source |
|-----|--------|
| `crates_mycelium-l1_src_eval_rs/` | `crates/mycelium-l1/src/eval.rs` |
| `crates_mycelium-l1_src_fuse_rs/` | `crates/mycelium-l1/src/fuse.rs` |
| `crates_mycelium-std-time_src/` | `crates/mycelium-std-time/src` |
| `crates_mycelium-std-rand_src/` | `crates/mycelium-std-rand/src` |
| `crates_mycelium-std-cmp_src/` | `crates/mycelium-std-cmp/src` |
| `crates_mycelium-std-fs_src/` | `crates/mycelium-std-fs/src` |
| `crates_mycelium-std-io_src/` | `crates/mycelium-std-io/src` |

Each target directory holds `vet.json` + `vet-stdout.txt` / `vet-stderr.txt` (lean artifact set, matching A3 precedent).

## Headlines

| Set | checked | expressible |
|-----|--------:|------------:|
| default-5 union | **19.5%** (46/236) | **19.5%** (46/236) |
| all-7 union | **17.0%** (58/342) | **27.5%** (94/342) |

See `summary.txt` for the raw driver log.
