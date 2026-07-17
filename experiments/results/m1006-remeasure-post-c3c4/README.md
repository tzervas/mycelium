# M-1006 remeasure post C3+C4 — experiment results (2026-07-16)

**Tip measured:** `origin/dev` @ `167f0bf2` (post C2 #1665 + C3 #1667 + **C4 #1670** + model policy #1671).
**Write-up:** `docs/planning/gap-analysis-2026-07-16/M1006-remeasure-post-C3C4-2026-07-16.md`
**Honesty:** `Empirical` numbers from `mycelium-transpile --vet` + real `myc-check`
(`MYC_CHECK_CMD`). **Not** a one-shot claim. **Not** a SemVer claim.

## Targets

Same set as `m1006-baseline-oneshot` / `m1006-remeasure-post-b1b2` (default-5 + std-fs + std-io):

| Dir | Source |
|-----|--------|
| `crates_mycelium-l1_src_eval_rs/` | `crates/mycelium-l1/src/eval.rs` |
| `crates_mycelium-l1_src_fuse_rs/` | `crates/mycelium-l1/src/fuse.rs` |
| `crates_mycelium-std-time_src/` | `crates/mycelium-std-time/src` |
| `crates_mycelium-std-rand_src/` | `crates/mycelium-std-rand/src` |
| `crates_mycelium-std-cmp_src/` | `crates/mycelium-std-cmp/src` |
| `crates_mycelium-std-fs_src/` | `crates/mycelium-std-fs/src` |
| `crates_mycelium-std-io_src/` | `crates/mycelium-std-io/src` |

Each target directory holds **full** emit (`.myc`, `*.gap.json`, and for dirs `union.gap.json` /
`summary.json` / `REMAP.md`) plus `vet.json` + `vet-stdout.txt` / `vet-stderr.txt`. Directory
targets include automatic `--vet --phylum` dual-report inside `vet.json`. See `summary.txt` for
the raw driver log.

## Headlines (`Empirical`, tip `167f0bf2`)

| Set | checked | expressible |
|-----|--------:|------------:|
| default-5 union | **19.5%** (46/236) | **19.5%** (46/236) |
| all-7 union | **22.2%** (76/342) | **27.5%** (94/342) |

| Expansion | oracle checked | expressible | phylum checked | Δ_basis |
|-----------|---------------:|------------:|---------------:|--------:|
| std-fs | **38.3%** (18/47) | 59.6% (28/47) | **59.6%** (28/47) | **+21.3pp** |
| std-io | **20.3%** (12/59) | 33.9% (20/59) | **23.7%** (14/59) | **+3.4pp** |

### What moved

- **C3 (on tip via #1667):** std-fs **38.3%** oracle confirmed; metadata among Clean×5.
- **C4 (on tip via #1670):** eval **21.4%** + rand **17.6%** Clean restored (single-variant enum
  `eq_*`); default-5 back to **19.5%**.
- **vs post-B1B2:** default-5 flat **0.0pp**; all-7 **+5.2pp** (fs/io gains retained).
- **Residual FLAGs not closed by C3/C4:** Import single-file (fs Δ_basis **+21.3pp**); std-io
  phylum `unknown type Source`.

**No one-shot / SemVer claim** from this remeasure.
