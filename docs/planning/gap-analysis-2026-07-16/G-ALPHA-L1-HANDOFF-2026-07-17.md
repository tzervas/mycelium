# G-α L1 handoff — transpile residual close (fractal)

| Field | Value |
|---|---|
| **Role** | L1 epic |
| **Model assigned** | `grok-composer-2.5-fast` |
| **Model actual (L1)** | session composer path |
| **L2 model assigned** | `grok-composer-2.5-fast` |
| **L2 model actual** | **Grok 4.5** via CLI worktree agents (assigned id not offered by `grok models` — never-silent VR-5/G2) |
| **Base tip (survey)** | `origin/dev` @ `67090f4a` → advanced to `6cdc5d1b` (program + survey landed #1693) |
| **Do not merge** | L0 merges L2 PRs |

## Survey summary (`Empirical` @ `67090f4a` / `6cdc5d1b`)

| Set | checked | expressible | File class |
|-----|--------:|------------:|------------|
| default-5 | **19.5%** (46/236) | 19.5% | all Clean |
| std-fs | **59.6%** (28/47) | 59.6% | Clean×7 (Import residual **closed** on tip) |
| std-io | **23.7%** (14/59) | 42.4% | CheckError×2 Clean×3 |
| all-7 | **25.7%** (88/342) | 28.9% | — |

**First poisons (baseline std-io):**

1. `io.myc`: `unknown type Result` on `read_all`
2. `lib.myc`: Import non-type `use std.io.io.read_all` (no such name)

Source/Sink already closed (#1675). std-fs fully Clean.

## Decomposition table

| Leaf | Rank | Class | Branch | PR | SHA | Status |
|------|-----:|-------|--------|----|-----|--------|
| **L2-A** | 1 | Result ambient co-emit | `claude/leaf/G-alpha-result-ambient` | **[#1695](https://github.com/tzervas/mycelium/pull/1695)** | `f8c72895` | OPEN → `dev`, not merged |
| **L2-B** | 2 | Import non-type free-fn co-include | `claude/leaf/G-alpha-import-non-type` | **[#1696](https://github.com/tzervas/mycelium/pull/1696)** | `e3a16c42` | OPEN → `dev`, not merged |
| **L2-M** | measure | Empirical table post G-α | `claude/leaf/G-alpha-measure` | **[#1697](https://github.com/tzervas/mycelium/pull/1697)** | `670f6bc4` | OPEN → `dev`, not merged |

**Parallelism:** L2-A/B both touch `emit.rs`/`transpile.rs` but auto-merged clean (local combined merge `22a7675c` in measure worktree; not pushed).

**L2 inject:** isolation worktree; cargo fmt/clippy/test -p mycelium-transpile; PR base=dev no merge.

## Metrics after L2 open PRs (combined local merge — `Empirical`)

| Metric | baseline | combined #1695+#1696 | Δ |
|--------|---------:|---------------------:|---|
| default-5 checked | 19.5% | 19.5% | 0 |
| all-7 checked | 25.7% | 25.7% | 0 (file-gated) |
| all-7 expressible | 28.9% | **29.5%** | **+0.6pp** |
| std-io checked | 23.7% | 23.7% | 0 (file-gated) |
| std-io expressible | 42.4% | **45.8%** | **+3.4pp** |
| std-fs | 59.6% Clean×7 | same | 0 |

**First-poison advance (oracle + phylum):**

| Baseline | Combined G-α |
|----------|--------------|
| `unknown type Result` | **closed** |
| Import `use …read_all` no such name | **closed** |
| — | **next:** `unknown function/constructor/prim read_to_end` on `read_all` body |

Headline checked_fraction flat is **expected** under file-gating until `io`/`lib` Clean — residual **class** closed, not a silent % win (VR-5).

## Next residual rank (G-β pick order — `Declared`)

| Rank | Class | Evidence |
|-----:|-------|----------|
| **1** | **`read_to_end` method/call surface** | Combined poison on `Ok(read_to_end(src))` in `read_all` |
| 2 | MacroInvocation / M-875 | design-gated until Accepted |
| 3 | Emission heat (Derive/NamedFieldDrop/Impl/MultiStmtBody) | not file-poison on pilot tip |
| 4 | fuse zero-emission | profile-only |

## FLAGs for L0

| FLAG | Note |
|------|------|
| Merge | Land **#1695 then #1696** (or either order — merge clean) then **#1697** measure; L1 does not merge |
| Shared files | CHANGELOG / issues.yaml `doc_refs` / Doc-Index — L0 close-out after land |
| Model | L2 CLI agents ran as **Grok 4.5** (composer id not in `grok models`) |
| Spawn path | L1 used headless `grok --worktree` (no in-session `spawn_subagent` tool); isolation held |
| One-shot / SemVer | **Forbidden** on this evidence |
| M-875 | still design-gated |

## Artifacts

- Survey: `docs/planning/gap-analysis-2026-07-16/G-ALPHA-SURVEY-2026-07-17.md`
- Leaf briefs: `leaves/G-alpha-L2A-result-ambient.md`, `…L2B-import-non-type.md`, `…L2M-measure.md`
- Measure: `M1006-remeasure-post-G-alpha-2026-07-17.md` + `experiments/results/m1006-remeasure-post-g-alpha/` (on #1697)
