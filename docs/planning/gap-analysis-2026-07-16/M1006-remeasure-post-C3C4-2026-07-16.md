# M-1006 remeasure post C3+C4 (Empirical, advisory)

**Date:** 2026-07-16
**Leaf:** ONESHOT-L2A — `claude/leaf/ONESHOT-L2A-measure-post-c3c4`
**Actual model used:** **Grok 4.5** (xAI agent runtime). **Assigned** by L1:
`grok-composer-2.5-fast` — unavailable in this runtime; recorded never-silently (VR-5/G2).
**Base tip (primary measure):** `167f0bf2` (`origin/dev` — C2 #1665 + C3 #1667 + **C4 #1670** +
model policy #1671).
**Session note:** Contract started against post-C3 tip `ed80e850` with C4 OPEN. Mid-session
`origin/dev` advanced (C4 + model-policy merged). Primary tables below are the **live tip**
remeasure after that advance. A mid-flight **post-C3-only** snapshot is retained under
"Historical mid-flight" for Δ honesty (VR-5: do not erase the intermediate).
**Scope:** Default M-1001 five-target set **plus** `std-fs` / `std-io` — same set as
[`M1006-baseline-oneshot-2026-07-16.md`](./M1006-baseline-oneshot-2026-07-16.md) and
[`M1006-remeasure-post-B1B2-2026-07-16.md`](./M1006-remeasure-post-B1B2-2026-07-16.md). **Not** the
full M-1006 17-target / whole-corpus ladder (VR-5).
**Artifacts:** `experiments/results/m1006-remeasure-post-c3c4/` (per-target `vet.json` +
`summary.txt`).
**Oracle:** real `myc-check` via `MYC_CHECK_CMD` (`scripts/checks/transpile-vet.sh` discipline).

## Commands

```bash
git fetch origin dev
# tip >= 167f0bf2 (post C3 #1667 + C4 #1670 + model policy #1671)
cargo build -q -p mycelium-check --bin myc-check
cargo build -q -p mycelium-transpile --bin mycelium-transpile
export MYC_CHECK_CMD="$PWD/target/debug/myc-check"
T="$PWD/target/debug/mycelium-transpile"
OUT=experiments/results/m1006-remeasure-post-c3c4
# default five
$T --vet crates/mycelium-l1/src/eval.rs "$OUT/crates_mycelium-l1_src_eval_rs"
$T --vet crates/mycelium-l1/src/fuse.rs  "$OUT/crates_mycelium-l1_src_fuse_rs"
$T --vet crates/mycelium-std-time/src    "$OUT/crates_mycelium-std-time_src"
$T --vet crates/mycelium-std-rand/src    "$OUT/crates_mycelium-std-rand_src"
$T --vet crates/mycelium-std-cmp/src     "$OUT/crates_mycelium-std-cmp_src"
# expansion
$T --vet crates/mycelium-std-fs/src      "$OUT/crates_mycelium-std-fs_src"
$T --vet crates/mycelium-std-io/src      "$OUT/crates_mycelium-std-io_src"
```

Honesty: all fractions below are **`Empirical`**. Residual ranking is **`Declared`** unless tied to a
cited diagnostic. **No one-shot claim. No SemVer claim.**

## Per-target results (`Empirical`, tip `167f0bf2`)

### Default M-1001 five-target set

| Target | non-test | checked | `checked_fraction` | emitted | `expressible_fraction` | File class | First oracle poison |
|--------|---------:|--------:|-------------------:|--------:|-----------------------:|------------|---------------------|
| `crates/mycelium-l1/src/eval.rs` | 42 | 9 | **21.4%** | 9 | **21.4%** | **Clean** | *(none)* |
| `crates/mycelium-l1/src/fuse.rs` | 12 | 0 | **0.0%** | 0 | **0.0%** | Clean | *(no emission — gap profile)* |
| `crates/mycelium-std-time/src` | 37 | 17 | **45.9%** | 17 | **45.9%** | **Clean** | *(none)* |
| `crates/mycelium-std-rand/src` | 34 | 6 | **17.6%** | 6 | **17.6%** | **Clean** | *(none)* |
| `crates/mycelium-std-cmp/src` | 111 | 14 | **12.6%** | 14 | **12.6%** | **Clean** | *(none)* |

**Union default-5:** **46 / 236** myc-check-clean → **`checked_fraction` 19.5%**; **46 / 236**
emitted → **`expressible_fraction` 19.5%**.

All five files are oracle-**Clean** (fuse Clean with zero emission). Where Clean and expressible
equal checked, remaining gap is **non-emission**, not check failure (DN-34 §8.7).

### Expansion (std-fs / std-io)

| Target | non-test | checked | `checked_fraction` | emitted | `expressible_fraction` | File class | First / representative oracle poisons |
|--------|---------:|--------:|-------------------:|--------:|-----------------------:|------------|----------------------------------------|
| `crates/mycelium-std-fs/src` | 47 | 18 | **38.3%** | 28 | **59.6%** | CheckError×2, Clean×5 | Import residual (full path): `use std.fs.error.ErrnoClass` / `FsErr` |
| `crates/mycelium-std-io/src` | 59 | 12 | **20.3%** | 20 | **33.9%** | CheckError×3, Clean×2 | Import residual (full path): `std.io.error.IoError` / `ByteCount` / `ByteOffset` |

**Union all-7:** **76 / 342** checked → **22.2%**; **94 / 342** emitted → **27.5%**.

### Phylum dual-report (`Empirical`)

| Target | `checked_fraction_phylum` | oracle `checked_fraction` | Δ_basis | phylum ok |
|--------|--------------------------:|--------------------------:|--------:|-----------|
| std-time | 45.9% | 45.9% | +0.0pp | true |
| std-rand | 17.6% | 17.6% | +0.0pp | true |
| std-cmp | 12.6% | 12.6% | +0.0pp | true |
| **std-fs** | **59.6%** (28/47) | **38.3%** (18/47) | **+21.3pp** | **true** (all 7 nodules Clean) |
| **std-io** | **23.7%** (14/59) | **20.3%** (12/59) | **+3.4pp** | false |

**std-fs Import residual (FLAG #1, live re-measure):** oracle still poisons `lib` + `substrate` on
full-path `use std.fs.error.*` (name not declared in single-file phylum view). Under phylum mode,
**every** fs nodule is Clean and checked rises 18→28 (**Δ_basis +21.3pp**). This is a **basis
correction** signal (DN-124 §4), **not** lever progress on the oracle headline — the residual is
single-file / consumer-resolve visibility of sibling `error` nodule exports.

**std-io residual (FLAG #2):** oracle still Import-poisons `io` / `lib` / `serialize` on full-path
`use std.io.error.*`. Phylum recovers `serialize` (and lifts checked 12→14, **+3.4pp**), but
**phylum residual remains** `unknown type Source` on `read_all` in `std.io.io` and `std.io` lib —
Import form is no longer the first phylum diagnostic. Source/Sink type emission or resolve is the
next implement leaf for io (not a second path-string Import pass).

## Historical mid-flight: post-C3-only @ `ed80e850` (`Empirical`, session)

Before C4 landed on `dev`, this leaf measured tip `ed80e850` (C3 + model-floor #1669; C4 OPEN):

| Metric | post-C3-only (`ed80e850`) | post-C3+C4 (`167f0bf2`) | Δ |
|--------|--------------------------:|------------------------:|---|
| default-5 checked | **13.1%** (31/236) | **19.5%** (46/236) | **+6.4pp** |
| all-7 checked | **17.8%** (61/342) | **22.2%** (76/342) | **+4.4pp** |
| eval checked | **0.0%** (`eq_ForageError` W7) | **21.4%** Clean | restored by C4 |
| rand checked | **0.0%** (`eq_RngAlgo` W7) | **17.6%** Clean | restored by C4 |
| std-fs / std-io | 38.3% / 20.3% | 38.3% / 20.3% | 0 (Import residual unchanged) |

C4 closed the default-5 regression; **did not** close Import single-file or Source/Sink FLAGs.

## Comparison vs post-B1B2 (`788574ab`, #1663) — primary tip

| Metric | post-B1B2 | this tip (`167f0bf2`) | Δ |
|--------|----------:|----------------------:|---|
| default-5 `checked_fraction` | **19.5%** (46/236) | **19.5%** (46/236) | **0.0pp** |
| default-5 `expressible_fraction` | 19.5% | 19.5% | 0.0pp |
| all-7 `checked_fraction` | **17.0%** (58/342) | **22.2%** (76/342) | **+5.2pp** |
| all-7 `expressible_fraction` | 27.5% | 27.5% | 0.0pp |
| std-fs checked / expressible | 10.6% / 59.6% | **38.3%** / 59.6% | **+27.7pp** / 0 |
| std-io checked / expressible | 11.9% / 33.9% | **20.3%** / 33.9% | **+8.4pp** / 0 |
| std-fs phylum / Δ_basis | 10.6% / +0.0pp | **59.6%** / **+21.3pp** | basis open |
| std-io phylum / Δ_basis | 15.3% / +3.4pp | **23.7%** / +3.4pp | +8.4pp phylum |

### vs C2/C3/C4 claims

| Claim | Source | Live remeasure @ `167f0bf2` | Verdict |
|-------|--------|----------------------------:|---------|
| std-fs **27.7%→38.3%** after C3 | handoff #1667 | **38.3%** (18/47) | **Confirmed** |
| metadata Clean after C3 | handoff #1667 | metadata among Clean×5 on fs | **Consistent** |
| std-io **11.9%→20.3%** after C2 | handoff #1665 | **20.3%** (12/59) | **Confirmed** |
| C4 restores std-rand **0%→17.6%** | #1670 | **17.6%** Clean | **Confirmed** (also restores eval 21.4%) |

### What C2+C3+C4 moved (honest)

1. **C2+C3:** Real oracle gains on expansion targets — fs **38.3%**, io **20.3%** live-confirmed.
2. **C4:** Restores default-5 Clean floor (eval+rand) after single-variant enum `eq_*` unreachable-arm
   poison; default-5 union back to **19.5%** (flat vs post-B1B2).
3. **Import residual not closed:** fs phylum Δ_basis **+21.3pp** with phylum ok=true is the live
   witness for FLAG #1. io phylum still fails on `Source` after Import recovery (FLAG #2).
4. **all-7 union +5.2pp vs post-B1B2** — expansion gains retained while default-5 holds the Clean
   floor.

**Still not one-shot.** Default-5 matches post-A5/post-B floor; expanded targets still show high
expressible / partial file-clean under Import + type residuals.

## Ranked residual list for next implement leaves (`Declared` ranking)

Gap-category tallies are **Empirical** raw counts over this 7-target set (`union.gap.json`
`category_counts` + single-file gap for eval/fuse; no double-count). Ranking weights **file-poison
impact on `checked_fraction`** first, then emission heat.

| Rank | Class / poison family | Evidence (this tip) | Suggested next leaf |
|-----:|----------------------|---------------------|---------------------|
| 1 | **Import single-file oracle** | std-fs oracle **38.3%** vs phylum **59.6%** (**Δ_basis +21.3pp**, phylum ok); poisons `lib`/`substrate` on full-path `std.fs.error.*`. io oracle still Import-poisons 3 files | **L2B / Import phase-2** — multi-file / phylum resolution so Clean `error` nodules unpoison consumers under **oracle** metric (not re-fix path strings) |
| 2 | **std-io Source/Sink** | Phylum residual after Import recovery: `unknown type Source` on `read_all` (`std.io.io` + `std.io` lib) | **L2C Source/Sink** type emit or resolve for io traits/aliases |
| 3 | **MacroInvocation / MacroDef** | std-cmp heat: MacroInvocation **57** + MacroDef **5** (dominant non-emission on largest Clean pilot) | **M-875** expand-first — design draft landed; **no implement** until Accepted |
| 4 | **Impl** + method bodies | Impl **57** across targets; cmp/eval/fs/io | One-class close after measure; method-body lever where emission exists but bodies gap |
| 5 | **NamedFieldDrop** / records | NamedFieldDrop **41** (fs/io/time heat) | DN-123 surface if still open; struct-field emission residual |
| 6 | **MultiStmtBody** | MultiStmtBody **26** (rand/fs/io) | body-lowering residual |
| 7 | **fuse zero-emission** | 0/12; Other/Import/ReservedWord profile | Profile-only until tractable native strategy |
| 8 | **DeriveAttr / DeriveSatisfied heat** | DeriveSatisfied **85**, DeriveAttr **60** (hottest counts) — C2/C3/C4 closed many **file-poison** eq paths; remaining is emission heat more than oracle poison on this tip | Prefer Import/Source leaves over another derive pass unless a new file poison appears |

### Top next implement leaves (orch pick order — `Declared`)

1. **Import single-file / phase-2 resolve** — closes fs Δ_basis +21.3pp oracle lag; also unpoisons io oracle files where residual is still Import.
2. **Source/Sink** — closes io phylum residual after Import.
3. **M-875 implement** only after design Accepted — largest cmp non-emission wall.
4. **C1 fuller M-1006 ladder** for corpus ranking beyond this pilot set.

## Residual FLAGs (orch-owned — do **not** edit `issues.yaml` here)

| Item | FLAG |
|------|------|
| **M-1006** | Remeasure post-C3+C4 recorded: default-5 **19.5%**; all-7 **22.2%** (+5.2pp vs post-B1B2); C3 fs **38.3%** + Δ_basis **+21.3pp** confirmed; C4 eval/rand Clean confirmed. Full 17-target ladder **still not** run. Append this path + results dir to `doc_refs` at integration close-out. |
| **C4 #1670** | **Landed** on tip `167f0bf2` — default-5 restored. Residual FLAGs below remain open. |
| **Import single-file** | Residual FLAG #1 — **not** closed by C3/C4; live +21.3pp fs Δ_basis. |
| **Source/Sink** | Residual FLAG #2 — io phylum `unknown type Source` after Import form fixed. |
| **M-875** | Remains design/needs-design — Macro still top non-emission heat on cmp. |
| **One-shot / SemVer** | **Forbidden** on this evidence. |
| **Shared files** | FLAG: `tools/github/issues.yaml` body/`doc_refs`, `CHANGELOG.md`, `docs/Doc-Index.md` — integrating parent. This leaf owns only `docs/planning/` + `experiments/results/`. |
| **Handoff tip** | FLAG for L1/orch: bump PROGRAM-HANDOFF-ONESHOT base tip to `167f0bf2` and point active remeasure at this file (this leaf appends a short changelog row only). |

## Method notes (VR-5)

- File-gated metric: any `CheckError` on a `.myc` file zeros that file’s checked numerator.
- `expressible_fraction` counts emission only; **`checked_fraction` is the port-accuracy headline**
  (DN-34 §8.7).
- Phylum Δ_basis is a **basis CORRECTION** (recovered false-fails), **not** lever progress
  (DN-124 §4).
- Gap-class tallies are **Empirical counts of gap records**, not a probability model of “fix next.”
- Ranking that prioritizes **oracle file poisons** over raw gap counts is **Declared** judgment on
  top of Empirical heat.
- Mid-flight post-C3-only numbers are retained so the C4 Δ is visible, not silently overwritten.

## Orchestrator actions suggested

1. Record L2A: **C3+C4 claims confirmed** live; all-7 **22.2%**; default-5 floor restored.
2. Spawn **Import single-file** + **Source/Sink** implement leaves (C4 does not close those FLAGs).
3. Optional: C1 fuller ladder after Import/Source move oracle numbers.
4. Do **not** flip SemVer / one-shot readiness; Epic R remains HOLD.
