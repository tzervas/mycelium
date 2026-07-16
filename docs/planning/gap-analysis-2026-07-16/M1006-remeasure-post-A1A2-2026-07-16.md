# M-1006 pilot re-measure post ORACLE-R1 A1+A2 (Empirical, advisory)

**Date:** 2026-07-16
**Leaf:** A3 — `claude/leaf/ORACLER1-A3-m1006-remeasure`
**Base tip:** `6d61b3b8a7ceea66324858c37c3632b501c10ac7` (`origin/dev` = merge of A1 #1647 + A2 #1648)
**Scope:** Default `just transpile-vet` five-target set (M-1001 profile) — **not** the full M-1006 17-target ladder (VR-5: do not treat these numbers as corpus-wide).
**Artifacts:** `experiments/results/m1006-remeasure-oracler1-a3/` (per-target `vet.json` + headlines).

## Commands

```bash
git fetch origin dev
# tip must be 6d61b3b8 (or later with A1+A2)
just transpile-vet
# equivalent: build myc-check + mycelium-transpile once, MYC_CHECK_CMD=…/target/debug/myc-check
```

Oracle: real `myc-check` via `MYC_CHECK_CMD` (`scripts/checks/transpile-vet.sh`). Honesty: fractions below are **`Empirical`**.

## Per-target results (`Empirical`)

| Target | non-test items | checked (myc-check-clean) | `checked_fraction` | emitted | `expressible_fraction` | File class | First `myc check` poison (oracle diagnostic) |
|--------|---------------:|--------------------------:|-------------------:|--------:|-----------------------:|------------|-----------------------------------------------|
| `crates/mycelium-l1/src/eval.rs` | 42 | 0 | **0.0%** | 7 | **16.7%** | CheckError | `unknown name DEFAULT_FUEL` (site `init`) |
| `crates/mycelium-l1/src/fuse.rs` | 12 | 0 | **0.0%** | 0 | **0.0%** | Clean | *(no emission; empty file check OK — numerator still 0)* |
| `crates/mycelium-std-time/src` | 37 | 0 | **0.0%** | 17 | **45.9%** | CheckError | `no instance Show for std.time::WallInstant` (site `render`) |
| `crates/mycelium-std-rand/src` | 34 | 6 | **17.6%** | 6 | **17.6%** | **Clean** | *(none — file fully clean)* |
| `crates/mycelium-std-cmp/src` | 111 | 14 | **12.6%** | 14 | **12.6%** | **Clean** | *(none — file fully clean)* |

**Union over the default 5 targets:** **20 / 236** items myc-check-clean → **`checked_fraction` 8.5%**; **44 / 236** emitted → **`expressible_fraction` 18.6%** (file-gated: one check error zeros that file’s numerator).

**Phylum-mode** (transpiler `--vet --phylum`, same denominators where run):

| Target | `checked_fraction_phylum` | Δ_basis vs oracle | phylum ok |
|--------|--------------------------:|------------------:|-----------|
| std-time | **0.0%** | +0.0pp | false (same Show residual) |
| std-rand | **17.6%** | +0.0pp | true |
| std-cmp | **12.6%** | +0.0pp | true |

No false-fail recovery on these pilots (Δ_basis +0.0pp).

## Comparison baselines (`Empirical` / prior Declared)

| Baseline | Tip / note | Union checked | Union expressible | std-cmp | std-rand | std-time | eval |
|----------|------------|--------------:|------------------:|--------:|---------:|---------:|------|
| Pre-G3 ledger (Phase-0) | older tip | ~4.3% (10/234) | ~12.8% | partial | — | partial | partial |
| Post-G3 pilot (`M1006-remeasure-2026-07-16.md`) | `8b35c2df` | **0.0%** (0/236) | 22.0% | 0.0% / 21.6% `Widen` | 0.0% / 17.6% `rotate_left` | 0.0% / 45.9% `as_nanos` dup | 0.0% / 11.9% `Default` |
| Post-#1645 companion (`EXPRESS-ORACLE-BLOCKERS`) | post oracle-poison close | — | — | **12.6% Clean** | **17.6% Clean** | 0.0% bare `0` / `is_negative` | Strength residual (pre-A2) |
| **This remeasure (post A1+A2)** | **`6d61b3b8`** | **8.5% (20/236)** | **18.6%** | **12.6% Clean** | **17.6% Clean** | **0.0%** `Show`/`WallInstant` | **0.0%** `DEFAULT_FUEL` |

### What moved after A1 + A2 (honest)

1. **A1 (std-time lit-zero / `is_negative`):** The **bare `0` / `is_negative` poison is gone** from the first oracle diagnostic. Residual is a **new surface**: missing `Show` instance for `WallInstant` on `render`. File still **0% checked** (file-gated) — **no net `checked_fraction` gain on std-time**, but poison **rotated** (G2: never claim “std-time fixed” from A1 alone).
2. **A2 (eval Strength):** The **`unknown type Strength` poison is gone**. Residual is **`unknown name DEFAULT_FUEL`** in `init`. Emission count **5 → 7** (11.9% → 16.7% expressible); checked still **0%** file-gated.
3. **std-cmp / std-rand:** Unchanged vs post-#1645 companion — still **file Clean** at **12.6%** / **17.6%** checked (= expressible). Confirms A1/A2 did not regress those pilots.

**Union `checked_fraction` 8.5%** is the first default-set union **above zero** in this post-G3 series of remeasures (driven entirely by cmp+rand clean files; time/eval/fuse still contribute 0 to the numerator).

## Residual still open (FLAG for L0 / integrating parent — do **not** edit `issues.yaml` here)

| Item | FLAG |
|------|------|
| **std-time** | Oracle poison: `no instance Show for std.time::WallInstant` (`render`). Candidate leaf **A5** (derive/Show lowering) — careful VR-5, no fabricated Show. |
| **eval.rs** | Oracle poison: `unknown name DEFAULT_FUEL` (`init`). Candidate leaf **A2b/A4**: co-emit or gap `DEFAULT_FUEL` / `DEFAULT_DEPTH` so Init not poisoned (transpile only). |
| **fuse.rs** | 0% expressible / 0% checked; Clean class with zero emissions — gap profile, not an oracle regression. |
| **M-1006** | Full 17-target / phased ladder **not run** — only M-1001 default 5-target profile. Append this path to M-1006 `doc_refs` at orch close-out. |
| **M-1090 / M-1084 / M-1037** | No new close-out evidence beyond prior notes; cmp/rand clean files predate this tip (companion #1645). |
| **Release gate** | Epic R criteria still open: std-time **still file-poisoned**; do not claim one-shot readiness. |
| **Shared files** | FLAG: `CHANGELOG.md`, `tools/github/issues.yaml`, `docs/Doc-Index.md` — orch-owned. |

## Orchestrator actions suggested

1. Record A1 success as **poison rotation** on std-time (lit-zero cleared; Show residual) — do not mark std-time pilot “done”.
2. Record A2 success as **Strength cleared**; queue **DEFAULT_FUEL / DEFAULT_DEPTH** (A4) before claiming eval progress on checked_fraction.
3. Prefer next code leaf **A4** (eval Init constants) or **A5** (WallInstant Show) — serial on transpile; remeasure again after either lands.
4. Keep **M-1006** `in-progress` until a fuller ladder or explicit scoped close-out decision.

## Method notes (VR-5)

- File-gated metric: any `CheckError` on a `.myc` file sets that file’s checked numerator to 0 even if most items emitted cleanly.
- `expressible_fraction` counts emission only; **`checked_fraction` is the port-accuracy headline** (DN-34 §8.7).
- When Clean and expressible equal checked, the file is oracle-clean for what was emitted — remaining gap is **non-emission**, not check failure.
- Phylum Δ_basis +0.0pp here is **not** lever progress (DN-124 §4).
