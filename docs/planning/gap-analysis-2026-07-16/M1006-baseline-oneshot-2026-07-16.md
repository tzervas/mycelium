# M-1006 ONESHOT baseline remeasure — post ORACLE-R1 A5 (Empirical, advisory)

**Date:** 2026-07-16
**Leaf:** ONESHOT-C0 — `claude/leaf/ONESHOT-C0-handoff-baseline-measure`
**Base tip:** `2ac85a84` (`origin/dev` — includes A1–A5 #1647–#1651 + MAINT close-out)
**Scope:** Default M-1001 five-target set **plus** `std-fs` / `std-io` expansion — **not** the full M-1006 17-target / whole-corpus ladder (VR-5: do not treat these numbers as corpus-wide or as one-shot readiness).
**Artifacts:** `experiments/results/m1006-baseline-oneshot/` (per-target `vet.json` + `summary.txt`).
**Oracle:** real `myc-check` via `MYC_CHECK_CMD` (`scripts/checks/transpile-vet.sh` discipline).

## Commands

```bash
git fetch origin dev
# tip >= 2ac85a84 (post A5)
cargo build -q -p mycelium-check --bin myc-check
cargo build -q -p mycelium-transpile --bin mycelium-transpile
export MYC_CHECK_CMD="$PWD/target/debug/myc-check"
T="$PWD/target/debug/mycelium-transpile"
OUT=experiments/results/m1006-baseline-oneshot
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

Honesty: all fractions below are **`Empirical`**. Interpretation of residual classes is **`Declared`** unless tied to a cited diagnostic.

## Per-target results (`Empirical`)

### Default M-1001 five-target set

| Target | non-test | checked | `checked_fraction` | emitted | `expressible_fraction` | File class | First oracle poison |
|--------|---------:|--------:|-------------------:|--------:|-----------------------:|------------|---------------------|
| `crates/mycelium-l1/src/eval.rs` | 42 | 9 | **21.4%** | 9 | **21.4%** | **Clean** | *(none)* |
| `crates/mycelium-l1/src/fuse.rs` | 12 | 0 | **0.0%** | 0 | **0.0%** | Clean | *(no emission — gap profile)* |
| `crates/mycelium-std-time/src` | 37 | 17 | **45.9%** | 17 | **45.9%** | **Clean** | *(none)* |
| `crates/mycelium-std-rand/src` | 34 | 6 | **17.6%** | 6 | **17.6%** | **Clean** | *(none)* |
| `crates/mycelium-std-cmp/src` | 111 | 14 | **12.6%** | 14 | **12.6%** | **Clean** | *(none)* |

**Union default-5:** **46 / 236** myc-check-clean → **`checked_fraction` 19.5%**; **46 / 236** emitted → **`expressible_fraction` 19.5%**.

All five files are oracle-**Clean**. Where Clean and expressible equal checked, remaining gap is **non-emission**, not check failure (DN-34 §8.7).

### Expansion (std-fs / std-io)

| Target | non-test | checked | `checked_fraction` | emitted | `expressible_fraction` | File class | First / representative oracle poisons |
|--------|---------:|--------:|-------------------:|--------:|-----------------------:|------------|----------------------------------------|
| `crates/mycelium-std-fs/src` | 47 | 5 | **10.6%** | 28 | **59.6%** | CheckError×5, Clean×2 | Import: `use error.ErrnoClass` / `error.FsErr` (M-662/DN-113); `eq_Fallibility` / `eq_FileKind` unknown; `or` on Bool T-Op |
| `crates/mycelium-std-io/src` | 59 | 7 | **11.9%** | 20 | **33.9%** | CheckError×4, Clean×1 | Import: `error.IoError` / `error.ByteCount` / `error.ByteOffset`; `eq_GuaranteeTag` unknown |

**Union all-7:** **58 / 342** checked → **17.0%**; **94 / 342** emitted → **27.5%**.

Expanded crates show the classic **high expressible / file-gated checked** pattern: emission outruns oracle-clean files because Import / derived-eq / op residuals poison whole nodules.

## Comparison baselines (`Empirical`)

| Baseline | Tip / note | Union checked (default-5) | Union expressible | std-time | eval |
|----------|------------|--------------------------:|------------------:|---------:|------|
| Post-G3 pilot | `8b35c2df` | **0.0%** (0/236) | 22.0% | 0% / Show etc. | 0% / Default |
| Post A1+A2 (A3) | `6d61b3b8` | **8.5%** (20/236) | 18.6% | 0% Show/WallInstant | 0% DEFAULT_FUEL |
| **Post A5 (this)** | **`2ac85a84`** | **19.5%** (46/236) | **19.5%** | **45.9% Clean** | **21.4% Clean** |

### What moved after A4 + A5 (honest)

1. **A4 (`DEFAULT_FUEL`/`DEFAULT_DEPTH`):** eval **0% → 21.4%** checked; file **Clean**; emission 7 → 9 items.
2. **A5 (wide Show + call-arg BinLit):** std-time **0% → 45.9%** checked; file **Clean**. Wide Show remains **Declared** opaque `"<Binary{N}>"` — not Exact Debug (VR-5).
3. **cmp / rand:** unchanged Clean at 12.6% / 17.6% — no regression.
4. **fuse:** still 0% emission / 0% checked — gap profile, not an oracle regression.
5. **Union `checked_fraction` 19.5%** is the highest default-5 union in this post-G3 series, driven by time+eval joining cmp+rand as Clean numerators.

**Still not one-shot.** Default-5 expressible equals checked only because non-emitted items dominate; expanded targets show large expressible–checked gap under Import/derive/op poisons.

## Ranked residual gap classes (next leaves — `Declared` ranking from Empirical counts)

Counts are raw gap-category tallies across the measured targets (advisory; double-counting across files is fine for ranking heat, not for exact corpus totals).

| Rank | Class / poison family | Where it bites | Suggested next leaf |
|-----:|----------------------|----------------|---------------------|
| 1 | **Import** (cross-phylum `error.*` unresolved) | std-fs / std-io CheckError; eval gap heat | **B1 M-1084** Import net-close |
| 2 | **DeriveAttr** (+ partial derive eq emission) | time / fs / io / rand heat; `eq_*` unknown prims on fs/io | B4 residual audit → derive/eq emit if real |
| 3 | **MacroInvocation** / MacroDef | std-cmp (dominant non-emission) | M-875 expand-first **design** (B5); no implement until Accepted |
| 4 | **Impl** + method bodies | cmp / eval / fs / io | Epic C2 after Import; method-body lever notes already cold on some pilots |
| 5 | **NamedFieldDrop** / records | fs / io / time | DN-123 surface if still open |
| 6 | **MultiStmtBody** | rand / fuse / fs | body-lowering residual |
| 7 | **T-Op / prim** (`or` Bool; unknown `eq_*`) | std-fs options; derived equality helpers | B2 M-1037-adjacent + derive-eq |
| 8 | **fuse zero-emission** (Other / Import / ReservedWord) | fuse.rs | Profile-only until a tractable native strategy |

## Residual FLAGs (orch-owned — do **not** edit `issues.yaml` here)

| Item | FLAG |
|------|------|
| **M-1006** | Full 17-target ladder **not** run — only default-5 + fs/io. Append this path to `doc_refs` at integration close-out. |
| **M-1090** | WU-3 already landed; this remeasure does **not** alone satisfy a 30-body DoD — keep `todo` unless orch has separate evidence. |
| **M-1084** | Primary lever for fs/io Import poisons (`error.ErrnoClass`, `FsErr`, `IoError`, …). |
| **M-1037** | Still relevant for conversion / prim residual; not the first poison on Clean default pilots. |
| **One-shot claim** | **Forbidden** on this evidence. Prep program uses this as baseline, not completion. |
| **Shared files** | FLAG: `issues.yaml` / full `Doc-Index` row polish if orch wants; this leaf only touches gap-analysis + CHANGELOG 1-liner. |

## Method notes (VR-5)

- File-gated metric: any `CheckError` on a `.myc` file zeros that file’s checked numerator.
- `expressible_fraction` counts emission only; **`checked_fraction` is the port-accuracy headline** (DN-34 §8.7).
- Phylum dual-report present on directory targets where the harness ran it; default single-file eval/fuse may omit phylum block (`phylum: null`).
- Gap-class tallies are **Empirical counts of gap records**, not a probability model of “fix next.”

## Orchestrator actions suggested

1. Point gap-analysis **README** at **PROGRAM-HANDOFF-ONESHOT** as active program (this PR).
2. Spawn **B1 M-1084** next for Import heat on fs/io (serial transpile).
3. Do **not** flip SemVer / one-shot readiness; Epic R remains HOLD until tip-bound CI + prep DoD.
4. Optional: C1 fuller ladder after B1 lands and B3 remeasures.
