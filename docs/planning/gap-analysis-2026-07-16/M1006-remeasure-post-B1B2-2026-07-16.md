# M-1006 remeasure post B1+B2 (Empirical, advisory)

**Date:** 2026-07-16
**Leaf:** ONESHOT-B3 — `claude/leaf/ONESHOT-B3-remeasure-post-B1B2`
**Base tip:** `788574ab` (`origin/dev` — B1 #1659 M-1084 + B2 #1660 M-1037 + PM close-out #1661)
**Scope:** Default M-1001 five-target set **plus** `std-fs` / `std-io` — same set as
[`M1006-baseline-oneshot-2026-07-16.md`](./M1006-baseline-oneshot-2026-07-16.md). **Not** the full
M-1006 17-target / whole-corpus ladder (VR-5).
**Artifacts:** `experiments/results/m1006-remeasure-post-b1b2/` (per-target `vet.json` + `summary.txt`).
**Oracle:** real `myc-check` via `MYC_CHECK_CMD` (`scripts/checks/transpile-vet.sh` discipline).

## Commands

```bash
git fetch origin dev
# tip >= 788574ab (post B1+B2)
cargo build -q -p mycelium-check --bin myc-check
cargo build -q -p mycelium-transpile --bin mycelium-transpile
export MYC_CHECK_CMD="$PWD/target/debug/myc-check"
T="$PWD/target/debug/mycelium-transpile"
OUT=experiments/results/m1006-remeasure-post-b1b2
# default five
$T --vet crates/mycelium-l1/src/eval.rs "$OUT/crates_mycelium-l1_src_eval_rs"
$T --vet crates/mycelium-l1/src/fuse.rs  "$OUT/crates_mycelium-l1_src_fuse_rs"
$T --vet crates/mycelium-std-time/src    "$OUT/crates_mycelium-std-time_src"
$T --vet crates/mycelium-std-rand/src    "$OUT/crates_mycelium-std-rand_src"
$T --vet crates/mycelium-std-cmp/src     "$OUT/crates_mycelium-std-cmp_src"
# expansion (cheap)
$T --vet crates/mycelium-std-fs/src      "$OUT/crates_mycelium-std-fs_src"
$T --vet crates/mycelium-std-io/src      "$OUT/crates_mycelium-std-io_src"
```

Honesty: all fractions below are **`Empirical`**. Residual ranking is **`Declared`** unless tied to a
cited diagnostic. **No one-shot claim.**

## Per-target results (`Empirical`)

### Default M-1001 five-target set

| Target | non-test | checked | `checked_fraction` | emitted | `expressible_fraction` | File class | First oracle poison |
|--------|---------:|--------:|-------------------:|--------:|-----------------------:|------------|---------------------|
| `crates/mycelium-l1/src/eval.rs` | 42 | 9 | **21.4%** | 9 | **21.4%** | **Clean** | *(none)* |
| `crates/mycelium-l1/src/fuse.rs` | 12 | 0 | **0.0%** | 0 | **0.0%** | Clean | *(no emission — gap profile)* |
| `crates/mycelium-std-time/src` | 37 | 17 | **45.9%** | 17 | **45.9%** | **Clean** | *(none)* |
| `crates/mycelium-std-rand/src` | 34 | 6 | **17.6%** | 6 | **17.6%** | **Clean** | *(none)* |
| `crates/mycelium-std-cmp/src` | 111 | 14 | **12.6%** | 14 | **12.6%** | **Clean** | *(none)* |

**Union default-5:** **46 / 236** myc-check-clean → **`checked_fraction` 19.5%**; **46 / 236** emitted →
**`expressible_fraction` 19.5%**.

All five files remain oracle-**Clean**. Where Clean and expressible equal checked, remaining gap is
**non-emission**, not check failure (DN-34 §8.7).

### Expansion (std-fs / std-io)

| Target | non-test | checked | `checked_fraction` | emitted | `expressible_fraction` | File class | First / representative oracle poisons |
|--------|---------:|--------:|-------------------:|--------:|-----------------------:|------------|----------------------------------------|
| `crates/mycelium-std-fs/src` | 47 | 5 | **10.6%** | 28 | **59.6%** | CheckError×5, Clean×2 | Import residual (full path): `use std.fs.error.ErrnoClass` / `FsErr`; `eq_Fallibility` / `eq_FileKind`; Bool `or` T-Op |
| `crates/mycelium-std-io/src` | 59 | 7 | **11.9%** | 20 | **33.9%** | CheckError×4, Clean×1 | Import residual (full path): `std.io.error.IoError` / `ByteCount` / `ByteOffset`; `eq_GuaranteeTag` |

**Union all-7:** **58 / 342** checked → **17.0%**; **94 / 342** emitted → **27.5%**.

### Phylum dual-report (`Empirical`)

| Target | `checked_fraction_phylum` | oracle `checked_fraction` | Δ_basis | phylum ok |
|--------|--------------------------:|--------------------------:|--------:|-----------|
| std-time | 45.9% | 45.9% | +0.0pp | true |
| std-rand | 17.6% | 17.6% | +0.0pp | true |
| std-cmp | 12.6% | 12.6% | +0.0pp | true |
| std-fs | 10.6% | 10.6% | +0.0pp | false |
| **std-io** | **15.3%** (9/59) | **11.9%** (7/59) | **+3.4pp** | false |

**std-io phylum recovery (`Empirical`):** single-file oracle poisons `serialize` on
`use std.io.error.ByteOffset`; phylum-mode marks `std.io.serialize` **Clean** (sibling `error`
nodule visible) and lifts checked 7 → 9. Remaining phylum poisons: `eq_GuaranteeTag` (guarantee_matrix)
and `unknown type Source` (io/lib) — Import form is no longer the first phylum diagnostic.

**std-fs phylum:** Import residual **rotates** under phylum — first nodule diagnostics become
`eq_Fallibility` / `eq_FileKind` / Bool `or`, not missing `error.*` names. Checked numerator still
**5/47** (file-gated eq/op residuals keep the same files dirty). This is a **basis correction
signal**, not lever progress on the oracle headline (DN-124 §4).

## Comparison vs baseline-oneshot (`Empirical`)

| Metric | Baseline-oneshot (`2ac85a84`, post A5) | This remeasure (`788574ab`, post B1+B2) | Δ |
|--------|---------------------------------------:|----------------------------------------:|---|
| default-5 `checked_fraction` | **19.5%** (46/236) | **19.5%** (46/236) | **0.0pp** |
| default-5 `expressible_fraction` | 19.5% (46/236) | 19.5% (46/236) | 0.0pp |
| all-7 `checked_fraction` | **17.0%** (58/342) | **17.0%** (58/342) | **0.0pp** |
| all-7 `expressible_fraction` | 27.5% (94/342) | 27.5% (94/342) | 0.0pp |
| std-fs checked / expressible | 10.6% / 59.6% | 10.6% / 59.6% | 0 |
| std-io checked / expressible | 11.9% / 33.9% | 11.9% / 33.9% | 0 |
| std-io phylum checked | *(not highlighted in baseline table)* | **15.3%** (+3.4pp basis) | phylum-only |

### What B1 + B2 moved (honest)

1. **B1 (M-1084 full-path use emit — #1659):** **Emit form fixed.** Oracle Import diagnostics now
   cite `use std.fs.error.ErrnoClass` / `use std.io.error.IoError` (etc.) instead of short
   `use error.ErrnoClass` / `use error.IoError`. **`checked_fraction` unchanged** on this pilot set —
   residual is *name not declared in the single-file phylum view*, not wrong path shape. Phylum mode
   shows the residual is partly a **linking/basis** issue (io +3.4pp; fs poison rotates to eq/op).
2. **B2 (M-1037 conversion residual — #1660):** `to_string` Exact(Bytes) + literal typing landed in
   transpile; **no default-5 / all-7 union movement** on this remeasure. Pilot note from B2 itself:
   std-cmp expressible stays ~12.6% (numeric `to_string`/`into` still dominate residuals). Do **not**
   read this remeasure as “B2 had no value” — it closed conversion honesty DoD without fabricating
   prims; it simply is **not the file-poison lever** on these targets.
3. **Default-5 Clean set stable:** eval/time/rand/cmp unchanged Clean numerators; fuse still 0%
   emission. **No regression** from B1/B2 on the Clean pilots.

**Still not one-shot.** Union fractions match the post-A5 baseline; expanded targets still show
high expressible / file-gated checked under Import residual (oracle) + derive-eq + Bool `or`.

## Ranked residual classes for next implement leaves (`Declared` ranking)

Gap-category tallies are **Empirical** raw counts over this 7-target set (union.gap / single-file
gap.json, no double-count). Ranking below weights **file-poison impact on `checked_fraction`** first,
then emission heat.

| Rank | Class / poison family | Evidence (this tip) | Suggested next leaf |
|-----:|----------------------|---------------------|---------------------|
| 1 | **DeriveAttr / derived `eq_*`** | `eq_Fallibility`, `eq_FileKind`, `eq_GuaranteeTag` unknown prims poison guarantee_matrix + metadata (fs/io); DeriveAttr **79** gaps (hottest class); under phylum becomes first residual on several fs nodules | **B4 residual audit** (M-1086 verify-first) → implement leaf only if real: emit/co-emit derived eq helpers or honest gap |
| 2 | **Import residual (post-path-fix)** | Still file-poisons fs `lib`/`substrate` and io `io`/`lib`/`serialize` under **single-file** oracle; form is full path; error nodules themselves are **Clean** (types emit). Phylum recovers some (io serialize) | Not “re-do B1 path emit.” Next: **phylum-aware use resolution / co-check** or ensure consumers resolve sibling nodule exports when vetting multi-file crates; optional language-surface if symbols truly missing from std error phylums |
| 3 | **MacroInvocation / MacroDef** | std-cmp heat: MacroInvocation **57** + MacroDef **5** (dominant non-emission on largest Clean pilot) | **M-875** expand-first — design **#1657** landed; **no implement** until Accepted (B5) |
| 4 | **Impl** + method bodies | Impl **57** across targets; cmp/eval/fs/io | Epic **C2** one-class close after measure; method-body lever where emission exists but bodies gap |
| 5 | **T-Op / Bool `or`** | fs `options.rs` / `wants_write`: `or` does not accept `[Bool, Bool]` (RFC-0007 §4.4) — whole-file poison for 3 emitted items | Small **prim/op** leaf: Bool logical ops or desugar `or`→`\|\|`/match — high file-gated ROI on fs options |
| 6 | **NamedFieldDrop** / records | NamedFieldDrop **41** (fs/io/time heat) | DN-123 surface if still open; struct-field emission residual |
| 7 | **MultiStmtBody** | MultiStmtBody **26** (rand/fs/io) | body-lowering residual |
| 8 | **fuse zero-emission** | 0/12; Other/Import/ReservedWord profile | Profile-only until tractable native strategy |
| 9 | **Conversion residual (M-1037 tail)** | B2 closed Bytes `to_string` + honesty gaps; `into` / `to_vec` / non-Bytes `to_string` still gapped by design | No new implement leaf unless a pilot file is **poisoned** by conversion (not observed here) |

### Top next implement leaves (orch pick order — `Declared`)

1. **Derive / `eq_*` emission or honest gap** (closes guarantee_matrix + metadata poisons on fs/io; survives phylum).
2. **Bool `or` T-Op** (one-file poison on `std-fs` options — cheap, local).
3. **Import residual phase-2** — multi-file / phylum resolution so Clean `error` nodules unpoison consumers under the **oracle** metric (not re-fix path strings).
4. **M-875 implement** only after design Accepted — largest cmp non-emission wall.
5. **C1 fuller M-1006 ladder** for corpus ranking beyond this pilot set.

## Residual FLAGs (orch-owned — do **not** edit `issues.yaml` here)

| Item | FLAG |
|------|------|
| **M-1006** | Remeasure after B1+B2 recorded: unions **flat at 19.5% / 17.0%**; B1 form verified; residual ranked. Full 17-target ladder **still not** run. Append this path + results dir to `doc_refs` at integration close-out. Status stays **in-progress** until orch decides scoped close or C1 ladder. |
| **M-1084** | Emit form **done** (#1659). Residual Import is **resolution/visibility** under single-file oracle, not path shape. Consider tracker note: “path emit closed; consumer resolve residual open.” |
| **M-1037** | B2 #1660 landed conversion honesty; **no pilot `checked_fraction` delta** on this set. FLAG close-out body note for orch (status likely **done** if DoD was honesty mapping, not pilot %). |
| **M-1090** | Not re-run as a format-specific suite; this B3 does **not** alone satisfy a 30-body DoD — keep separate unless orch has other evidence. |
| **M-875** | Remains **needs-design** / design draft only — Macro still top non-emission heat on cmp. |
| **One-shot claim** | **Forbidden** on this evidence. Prep program uses this as post-B net-close measure, not completion. |
| **Shared files** | FLAG: `tools/github/issues.yaml` body/`doc_refs`, `CHANGELOG.md`, `docs/Doc-Index.md` — integrating parent. This leaf touches gap-analysis + results + handoff table only. |

## Method notes (VR-5)

- File-gated metric: any `CheckError` on a `.myc` file zeros that file’s checked numerator.
- `expressible_fraction` counts emission only; **`checked_fraction` is the port-accuracy headline** (DN-34 §8.7).
- Phylum Δ_basis is a **basis CORRECTION** (recovered false-fails), **not** lever progress (DN-124 §4).
- Gap-class tallies are **Empirical counts of gap records**, not a probability model of “fix next.”
- Ranking that prioritizes **oracle file poisons** over raw gap counts is **Declared** judgment on top of Empirical heat.

## Orchestrator actions suggested

1. Record B3: **B1+B2 did not move pilot union `checked_fraction`**; B1 form verified; B2 honesty-only on this set.
2. Spawn next implement leaves from ranked table — prefer **derive/`eq_*`** and **Bool `or`**, not a second path-string Import pass.
3. Optional: C1 fuller ladder after those close.
4. Do **not** flip SemVer / one-shot readiness; Epic R remains HOLD.
