# M-1006 remeasure post G-α — experiment results (2026-07-17)

**Write-up:**
`docs/planning/gap-analysis-2026-07-16/M1006-remeasure-post-G-alpha-2026-07-17.md`
**Honesty:** `Empirical` numbers from `mycelium-transpile --vet` + real `myc-check`
(`MYC_CHECK_CMD`). **Not** a one-shot claim. **Not** a SemVer claim.
**Model:** assigned `grok-composer-2.5-fast`; actual **Grok 4.5** (xAI agent runtime).

## Tips measured

| Condition | SHA | Notes |
|-----------|-----|-------|
| **baseline-dev/** | `origin/dev` @ `6cdc5d1b` | Confirmation of G-α survey residual |
| **combined-pr1695-1696/** (primary) | local merge `22a7675c` | OPEN #1695 (`f8c72895`) + #1696 (`e3a16c42`) onto dev; **not** pushed |

## Targets

Same set as `m1006-remeasure-post-c3c4` / G-α survey (default-5 + std-fs + std-io):

| Dir | Source |
|-----|--------|
| `crates_mycelium-l1_src_eval_rs/` | `crates/mycelium-l1/src/eval.rs` |
| `crates_mycelium-l1_src_fuse_rs/` | `crates/mycelium-l1/src/fuse.rs` |
| `crates_mycelium-std-time_src/` | `crates/mycelium-std-time/src` |
| `crates_mycelium-std-rand_src/` | `crates/mycelium-std-rand/src` |
| `crates_mycelium-std-cmp_src/` | `crates/mycelium-std-cmp/src` |
| `crates_mycelium-std-fs_src/` | `crates/mycelium-std-fs/src` |
| `crates_mycelium-std-io_src/` | `crates/mycelium-std-io/src` |

Each target directory holds full emit (`.myc`, `*.gap.json`, and for dirs
`union.gap.json` / `summary.json` / `REMAP.md` where produced) plus `vet.json` +
`vet-stdout.txt` / `vet-stderr.txt`. Directory targets include automatic
`--vet --phylum` dual-report inside `vet.json`. Machine rollup: `summary.json`.

## Headlines (`Empirical`)

| Set | baseline checked | combined checked | baseline expressible | combined expressible |
|-----|-----------------:|-----------------:|---------------------:|---------------------:|
| default-5 union | **19.5%** (46/236) | **19.5%** (46/236) | 19.5% | 19.5% |
| all-7 union | **25.7%** (88/342) | **25.7%** (88/342) | 28.9% (99/342) | **29.5%** (101/342) |

| Expansion | baseline oracle | combined oracle | combined expressible | combined phylum |
|-----------|----------------:|----------------:|---------------------:|----------------:|
| std-fs | 59.6% Clean×7 | 59.6% Clean×7 | 59.6% | ok=true |
| std-io | 23.7% (`Result` + Import) | **23.7%** (`read_to_end`) | **45.8%** | ok=false (`read_to_end`) |

### What moved

- **#1695 Result ambient (OPEN):** first poison `unknown type Result` **closed**
  (oracle + phylum); ambient `type Result[A,E]=Ok(A)|Err(E)` co-emitted.
- **#1696 Import non-type (OPEN):** first poison `use std.io.io.read_all` **closed**.
- **Next residual:** `unknown function/constructor/prim read_to_end` on `read_all`
  (oracle + phylum, both `io` and `lib`).
- **checked_fraction flat** under file-gating until those files Clean — residual
  **advancement**, not a headline % claim.

**No one-shot / SemVer claim** from this remeasure.
