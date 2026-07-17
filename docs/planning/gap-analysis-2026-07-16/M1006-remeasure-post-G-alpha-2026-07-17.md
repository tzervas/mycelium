# M-1006 remeasure post G-α (Empirical, advisory)

**Date:** 2026-07-17
**Leaf:** G-alpha-L2M-measure — `claude/leaf/G-alpha-measure`
**Actual model used:** **Grok 4.5** (xAI agent runtime). **Assigned** by L1:
`grok-composer-2.5-fast` — unavailable in this runtime; recorded never-silently (VR-5/G2).
**Base tip (baseline confirmation):** `origin/dev` @
`6cdc5d1b86c90765a7d437b4111519535be1e062` (post selfhost-decompose #1693; advanced vs
G-α survey tip `67090f4a` — default-5 / fs / io **checked** fractions match survey).
**Open implement PRs (not merged at measure time):**

| PR | Head SHA | Branch | Title |
|----|----------|--------|-------|
| **#1695** | `f8c72895112cf1d358cd303d77070a09945122bb` | `claude/leaf/G-alpha-result-ambient` | ambient Result/Option co-emit (G-α Rank-1) |
| **#1696** | `e3a16c42d28ff9c107609b85be658f4392939267` | `claude/leaf/G-alpha-import-non-type` | Import non-type free-fn co-include (G-α Rank-2) |

**Primary measure tip:** throwaway **local** merge of both OPEN PR heads onto
`origin/dev` → `22a7675c0d53dca0c10d1dd181a2bf1dd1c5c055` (**not** pushed to any
protected trunk; product code not in this measure PR).
**Scope:** Default M-1001 five-target set **plus** `std-fs` / `std-io` — same set as
[`M1006-remeasure-post-C3C4-2026-07-16.md`](./M1006-remeasure-post-C3C4-2026-07-16.md)
and [`G-ALPHA-SURVEY-2026-07-17.md`](./G-ALPHA-SURVEY-2026-07-17.md). **Not** the full
M-1006 17-target / whole-corpus ladder (VR-5).
**Artifacts:** `experiments/results/m1006-remeasure-post-g-alpha/`
(`baseline-dev/` + `combined-pr1695-1696/` per-target emit + `vet.json` + README /
`summary.json`).
**Oracle:** real `myc-check` via `MYC_CHECK_CMD` (`scripts/checks/transpile-vet.sh`
discipline).

## Commands

```bash
git fetch origin dev
# tip >= 6cdc5d1b
cargo build -q -p mycelium-check --bin myc-check
cargo build -q -p mycelium-transpile --bin mycelium-transpile
export MYC_CHECK_CMD="$PWD/target/debug/myc-check"
T="$PWD/target/debug/mycelium-transpile"
OUT=experiments/results/m1006-remeasure-post-g-alpha

# (a) baseline confirmation on origin/dev
# (b) throwaway merge (local only):
#   git checkout -b tmp/g-alpha-combined origin/dev
#   git merge --no-ff f8c72895   # #1695
#   git merge --no-ff e3a16c42   # #1696
#   rebuild T; measure into $OUT/combined-pr1695-1696

# default five + expansion (same for each tip)
$T --vet crates/mycelium-l1/src/eval.rs "$OUT/<tip>/crates_mycelium-l1_src_eval_rs"
$T --vet crates/mycelium-l1/src/fuse.rs  "$OUT/<tip>/crates_mycelium-l1_src_fuse_rs"
$T --vet crates/mycelium-std-time/src    "$OUT/<tip>/crates_mycelium-std-time_src"
$T --vet crates/mycelium-std-rand/src    "$OUT/<tip>/crates_mycelium-std-rand_src"
$T --vet crates/mycelium-std-cmp/src     "$OUT/<tip>/crates_mycelium-std-cmp_src"
$T --vet crates/mycelium-std-fs/src      "$OUT/<tip>/crates_mycelium-std-fs_src"
$T --vet crates/mycelium-std-io/src      "$OUT/<tip>/crates_mycelium-std-io_src"
```

Honesty: all fractions below are **`Empirical`**. Residual ranking is **`Declared`**
unless tied to a cited diagnostic. **No one-shot claim. No SemVer claim.**

## (a) Baseline confirmation — tip `6cdc5d1b` (`Empirical`)

Confirms G-α survey residual on live `origin/dev` (checked fractions **identical** to
survey @ `67090f4a`; emit all-7 **99/342=28.9%** vs survey **100/342=29.2%** — 1-item tip
delta, not a poison-class change).

### Default M-1001 five-target set

| Target | non-test | checked | `checked_fraction` | emitted | `expressible_fraction` | File class | First oracle poison |
|--------|---------:|--------:|-------------------:|--------:|-----------------------:|------------|---------------------|
| `eval.rs` | 42 | 9 | **21.4%** | 9 | **21.4%** | **Clean** | *(none)* |
| `fuse.rs` | 12 | 0 | **0.0%** | 0 | **0.0%** | Clean | *(no emission — gap profile)* |
| `std-time` | 37 | 17 | **45.9%** | 17 | **45.9%** | **Clean** | *(none)* |
| `std-rand` | 34 | 6 | **17.6%** | 6 | **17.6%** | **Clean** | *(none)* |
| `std-cmp` | 111 | 14 | **12.6%** | 14 | **12.6%** | **Clean** | *(none)* |

**Union default-5:** **46 / 236** → **`checked_fraction` 19.5%** (flat vs post-C3C4 /
post-A5 / G-α survey).

### Expansion (std-fs / std-io)

| Target | non-test | checked | `checked_fraction` | emitted | `expressible_fraction` | File class | First / representative oracle poisons |
|--------|---------:|--------:|-------------------:|--------:|-----------------------:|------------|----------------------------------------|
| `std-fs` | 47 | 28 | **59.6%** | 28 | **59.6%** | **Clean×7** | *(none)* |
| `std-io` | 59 | 14 | **23.7%** | 25 | **42.4%** | CheckError×2, Clean×3 | Result + Import non-type |

**Union all-7:** **88 / 342** checked → **25.7%**; **99 / 342** emitted → **28.9%**.

#### First-poison list (baseline — every CheckError file)

| Crate | File | First diagnostic (truncated) |
|-------|------|------------------------------|
| std-io | `io.rs` → `io.myc` | `check error in \`read_all\`: unknown type \`Result\`` |
| std-io | `lib.rs` → `lib.myc` | `` `use std.io.io.read_all`: no such name `std.io.io.read_all` in the phylum `` |

**Phylum dual-report (baseline):** std-fs phylum ok=true, Δ_basis +0.0pp (oracle=phylum
59.6%). std-io phylum ok=**false** — both `std.io.io` and `std.io` fail with
**`unknown type Result`** on `read_all` (Import form is not the first phylum diagnostic).

## (b) Combined OPEN PRs #1695+#1696 — tip `22a7675c` (`Empirical`, primary)

Local merge order: `origin/dev` ← #1695 (`f8c72895`) ← #1696 (`e3a16c42`). Auto-merge
clean on `emit.rs` / `transpile.rs` (no conflict resolution beyond ort).

### Default M-1001 five-target set

| Target | non-test | checked | `checked_fraction` | emitted | `expressible_fraction` | File class | First oracle poison |
|--------|---------:|--------:|-------------------:|--------:|-----------------------:|------------|---------------------|
| `eval.rs` | 42 | 9 | **21.4%** | 9 | **21.4%** | **Clean** | *(none)* |
| `fuse.rs` | 12 | 0 | **0.0%** | 0 | **0.0%** | Clean | *(no emission — gap profile)* |
| `std-time` | 37 | 17 | **45.9%** | 17 | **45.9%** | **Clean** | *(none)* |
| `std-rand` | 34 | 6 | **17.6%** | 6 | **17.6%** | **Clean** | *(none)* |
| `std-cmp` | 111 | 14 | **12.6%** | 14 | **12.6%** | **Clean** | *(none)* |

**Union default-5:** **46 / 236** → **19.5%** — **flat** (G-α leaves target expansion
poisons, not default-5).

### Expansion (std-fs / std-io)

| Target | non-test | checked | `checked_fraction` | emitted | `expressible_fraction` | File class | First / representative oracle poisons |
|--------|---------:|--------:|-------------------:|--------:|-----------------------:|------------|----------------------------------------|
| `std-fs` | 47 | 28 | **59.6%** | 28 | **59.6%** | **Clean×7** | *(none)* |
| `std-io` | 59 | 14 | **23.7%** | 27 | **45.8%** | CheckError×2, Clean×3 | **`read_to_end`** (both files) |

**Union all-7:** **88 / 342** checked → **25.7%** (**flat** vs baseline); **101 / 342**
emitted → **29.5%** (**+2 emit items** on io — ambient Result type surface co-emitted into
`io.myc` / `lib.myc`).

#### First-poison list (combined — every CheckError file)

| Crate | File | First diagnostic (truncated) |
|-------|------|------------------------------|
| std-io | `io.rs` → `io.myc` | `check error in \`read_all\`: unknown function/constructor/prim \`read_to_end\`` |
| std-io | `lib.rs` → `lib.myc` | `check error in \`read_all\`: unknown function/constructor/prim \`read_to_end\`` |

**Cleared vs baseline / G-α survey Rank 1–2:**

1. **`unknown type Result`** — **gone** as first poison (oracle + phylum). Emission witness:
   `type Result[A, E] = Ok(A) | Err(E);` co-emitted with Declared ambient comment in
   `io.myc` (G-α L2-A / #1695).
2. **Import non-type** `` `use std.io.io.read_all` `` — **gone** as first poison on
   single-file `lib` (G-α L2-B / #1696). Both CheckError files now share the **same**
   residual family.

**Not cleared (headline metric):** `checked_fraction` for std-io stays **23.7%** (14/59)
because the file-gated metric still zeros `io` + `lib` numerators under the **next**
poison. Expressible rose **42.4%→45.8%** (25→27 emitted) — emission progress without
oracle file-clean.

### Phylum dual-report (`Empirical`, combined)

| Target | `checked_fraction_phylum` | oracle `checked_fraction` | Δ_basis | phylum ok |
|--------|--------------------------:|--------------------------:|--------:|-----------|
| std-time / rand / cmp | = oracle | = oracle | +0.0pp | true |
| **std-fs** | **59.6%** | **59.6%** | **+0.0pp** | **true** (all 7 Clean) |
| **std-io** | **23.7%** | **23.7%** | **+0.0pp** | **false** |

**std-io phylum residual (combined):** both `std.io.io` and `std.io` fail with
**`unknown function/constructor/prim read_to_end`** on site `read_all` — same as oracle.
No Δ_basis open on this tip (oracle already sees the true residual once Result+Import
closed).

**Emission witness (combined `io.myc`):**

```text
type Result[A, E] = Ok(A) | Err(E);
…
pub fn read_all(src: Source) => Result[Vec[Binary{8}], IoError] =
  let src = src in Ok(read_to_end(src));
```

`read_to_end` is a **method/call** residual (Rust `Read::read_to_end` surface), not a
missing ambient type and not an Import path refusal.

## Comparison table (baseline tip → combined OPEN PRs)

| Metric | baseline `6cdc5d1b` | combined `22a7675c` (#1695+#1696) | Δ |
|--------|--------------------:|----------------------------------:|---|
| default-5 `checked_fraction` | **19.5%** (46/236) | **19.5%** (46/236) | **0.0pp** |
| all-7 `checked_fraction` | **25.7%** (88/342) | **25.7%** (88/342) | **0.0pp** |
| all-7 `expressible_fraction` | 28.9% (99/342) | **29.5%** (101/342) | **+0.6pp** |
| std-fs checked / expressible | 59.6% / 59.6% | 59.6% / 59.6% | 0 |
| std-io checked / expressible | 23.7% / 42.4% | **23.7%** / **45.8%** | **0 / +3.4pp** |
| std-io first poison | `Result` + Import `read_all` | **`read_to_end`** (both files) | Rank 1–2 **closed** as first poison |
| std-io phylum ok | false (`Result`) | false (`read_to_end`) | residual **advanced** |

### What G-α Rank 1+2 moved (honest)

1. **#1695 Result ambient:** Closed the **type** first-poison on `read_all`. Co-emit is
   visible in the `.myc` surface; checker no longer reports `unknown type Result`.
2. **#1696 Import non-type:** Closed the **single-file Import** first-poison on
   `use std.io.io.read_all`. `lib` now fails on the same body residual as `io`.
3. **Headline `checked_fraction` flat:** Expected under file-gating until `io`/`lib` go
   Clean. G-α did **not** claim a one-shot lift — it claimed residual **advancement**.
4. **Next residual is method/call:** `read_to_end` — matches pre-wave expectation in the
   G-α survey ("after Result+Import closed").

**Still not one-shot.** Default-5 floor holds; expansion still CheckError×2 on std-io;
gap-profile heat (Macro / Derive / Impl / NamedFieldDrop) unchanged at 512 total gap
records across the 7-target set.

## Ranked residual list for G-β (`Declared` ranking on Empirical diagnostics)

Gap-category tallies (`union.gap.json` / single-file gap; **identical** baseline vs
combined — G-α did not move non-emission heat):

| Count | Category |
|------:|----------|
| 85 | DeriveSatisfied |
| 64 | Other |
| 64 | MacroInvocation |
| 63 | DeriveAttr |
| 55 | Impl |
| 44 | NamedFieldDrop |
| 36 | Import *(gap-profile records, not current oracle first-poison)* |
| 26 | MultiStmtBody |
| 5 | MacroDef |
| … | (remainder ≤15 each) |

Ranking weights **file-poison impact on `checked_fraction`** first, then emission heat.

| Rank | Class / poison family | Evidence (combined tip) | Suggested next leaf |
|-----:|----------------------|-------------------------|---------------------|
| **1** | **`read_to_end` method/call** (or honest prim map / body rewrite) | Oracle + phylum first poison on **both** `io` and `lib`: `unknown function/constructor/prim read_to_end` inside `read_all` body `Ok(read_to_end(src))`. Closes std-io CheckError×2 → would lift checked 14→? toward expressible ceiling once files Clean. | **G-β L2: read_to_end / Read method surface** (map, co-emit, or never-silent gap) |
| 2 | **MacroInvocation / MacroDef** | cmp heat 64+5; non-emission wall on largest Clean pilot | **M-875** — design-gated until Accepted |
| 3 | **DeriveSatisfied / DeriveAttr heat** | 85+63 — emission heat, not file-poison on this tip | later / only if new file poison |
| 4 | **Impl** + method bodies | Impl 55 | method-body lever after Rank 1 |
| 5 | **NamedFieldDrop** / records | 44 (fs/io/time heat) | struct-field emission residual |
| 6 | **MultiStmtBody** | 26 | body-lowering residual |
| 7 | **fuse zero-emission** | 0/12 profile | not G-α/G-β file-poison |
| 8 | **Import gap-profile** | 36 records remain as **non-emission** heat; **not** the current oracle first-poison after #1696 | do not re-open Rank-2 path unless a new Import diagnostic reappears |

### Top next implement leaves (orch pick order — `Declared`)

1. **`read_to_end` / Read method call** — sole remaining **oracle + phylum** file poison on
   this pilot set after Result ambient + Import non-type.
2. **M-875 implement** only after design Accepted — largest cmp non-emission wall.
3. **Fuller M-1006 ladder (G-γ / C1)** for corpus ranking beyond this pilot set.
4. **Do not** re-spawn Result ambient or Import non-type for this residual — those first
   poisons are **Empirical-closed** on the combined tip.

## Residual FLAGs (orch-owned — do **not** edit `issues.yaml` here)

| Item | FLAG |
|------|------|
| **M-1006 / G-α measure** | Remeasure post OPEN #1695+#1696 recorded: default-5 **19.5%** flat; all-7 **25.7%** flat checked; std-io expressible **+3.4pp**; Rank 1–2 first-poisons **closed**; next poison **`read_to_end`**. Full 17-target ladder **still not** run. Append this path + results dir to `doc_refs` at integration close-out. |
| **#1695 Result ambient** | OPEN at measure; **combined tip confirms** Result type first-poison closed. Land via normal PR review — this leaf does not merge. |
| **#1696 Import non-type** | OPEN at measure; **combined tip confirms** `use std.io.io.read_all` first-poison closed. |
| **Next residual** | **`read_to_end`** method/call — G-β implement leaf. |
| **checked_fraction flat** | Never-silent: file-gating means Rank 1+2 success is a **poison-class advance**, not a headline % claim until files Clean. |
| **One-shot / SemVer** | **Forbidden** on this evidence. |
| **Shared files** | FLAG: `tools/github/issues.yaml` body/`doc_refs`, `CHANGELOG.md`, `docs/Doc-Index.md` — integrating parent. This leaf owns only `docs/planning/` + `experiments/results/`. |
| **Throwaway merge** | Combined SHA `22a7675c` was **local-only** for measure; **not** on any remote protected branch. Product code stays on #1695/#1696. |

## Method notes (VR-5)

- File-gated metric: any `CheckError` on a `.myc` file zeros that file’s checked numerator.
- `expressible_fraction` counts emission only; **`checked_fraction` is the port-accuracy
  headline** (DN-34 §8.7).
- Closing a first-poison without Clean files is **residual advancement**, not a silent
  claim of port progress on the headline fraction.
- Phylum Δ_basis is a **basis CORRECTION** when it recovers false-fails (DN-124 §4); here
  Δ_basis stays 0 — oracle and phylum agree on `read_to_end`.
- Gap-class tallies are **Empirical counts of gap records**, not a probability model of
  “fix next.”
- Ranking that prioritizes **oracle file poisons** over raw gap counts is **Declared**
  judgment on top of Empirical heat.
- Baseline and combined are both retained so the Rank 1–2 Δ is visible (VR-5: do not erase
  the pre-state).

## Orchestrator actions suggested

1. Record L2-M: **#1695+#1696 first-poison claims confirmed** on combined tip; headline
   checked **flat** (expected); next residual **`read_to_end`**.
2. Land #1695 / #1696 via normal `/pr-land` (this measure PR does **not** merge them).
3. Spawn **G-β `read_to_end` / Read method** implement leaf after (or parallel to) landings.
4. Do **not** flip SemVer / one-shot readiness; do **not** re-open Result/Import as Rank-1
   for this pilot set unless a regression reintroduces those diagnostics.
