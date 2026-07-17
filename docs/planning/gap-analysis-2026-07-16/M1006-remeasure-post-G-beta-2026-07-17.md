# M-1006 remeasure post G-β (Empirical, advisory)

**Date:** 2026-07-17
**Leaf:** G-beta-L2M-measure — `claude/leaf/G-beta-remeasure`
**Actual model used:** **Grok 4.5** (xAI agent runtime). Assigned / available identity
in this runtime is not `composer-2.5-fast`; recorded never-silently (VR-5/G2).
**Measure tip:** `origin/dev` @
`4acd3a20d9272660e11fcc8b3dceec1c163f0c48`
— includes **#1700** G-β (`96839724` merge of `claude/leaf/G-beta-read-to-end`) and
**#1699** D1 (`4acd3a20` merge of `claude/leaf/D1-component-repo-map`). Also ancestors of
landed G-α #1695+#1696+#1697+#1698.
**Scope:** Default M-1001 five-target set **plus** `std-fs` / `std-io` — same set as
[`M1006-remeasure-post-G-alpha-2026-07-17.md`](./M1006-remeasure-post-G-alpha-2026-07-17.md).
**Not** the full M-1006 17-target / whole-corpus ladder (VR-5).
**Artifacts:** `experiments/results/m1006-remeasure-post-g-beta/`
(per-target emit + `vet.json` + `summary.json` / README / `run.log`).
**Oracle:** real `myc-check` via `MYC_CHECK_CMD` (`scripts/checks/transpile-vet.sh`
discipline).

## Commands

```bash
git fetch origin dev
# tip >= 4acd3a20 (must include #1700 G-β and #1699 D1)
cargo build -q -p mycelium-check --bin myc-check
cargo build -q -p mycelium-transpile --bin mycelium-transpile
export MYC_CHECK_CMD="$PWD/target/debug/myc-check"
T="$PWD/target/debug/mycelium-transpile"
OUT=experiments/results/m1006-remeasure-post-g-beta
mkdir -p "$OUT"
$T --vet crates/mycelium-l1/src/eval.rs "$OUT/eval"
$T --vet crates/mycelium-l1/src/fuse.rs  "$OUT/fuse"
$T --vet crates/mycelium-std-time/src    "$OUT/time"
$T --vet crates/mycelium-std-rand/src    "$OUT/rand"
$T --vet crates/mycelium-std-cmp/src     "$OUT/cmp"
$T --vet crates/mycelium-std-fs/src      "$OUT/fs"
$T --vet crates/mycelium-std-io/src      "$OUT/io"
```

Honesty: all fractions below are **`Empirical`**. Residual ranking is **`Declared`**
unless tied to a cited diagnostic. **No one-shot claim. No SemVer claim.**

## Per-target table — tip `4acd3a20` (`Empirical`)

### Default M-1001 five-target set

| Target | non-test | checked | `checked_fraction` | emitted | `expressible_fraction` | File class | First oracle poison |
|--------|---------:|--------:|-------------------:|--------:|-----------------------:|------------|---------------------|
| `eval.rs` | 42 | 9 | **21.4%** | 9 | **21.4%** | **Clean** | *(none)* |
| `fuse.rs` | 12 | 0 | **0.0%** | 0 | **0.0%** | Clean | *(no emission — gap profile)* |
| `std-time` | 37 | 17 | **45.9%** | 17 | **45.9%** | **Clean** | *(none)* |
| `std-rand` | 34 | 6 | **17.6%** | 6 | **17.6%** | **Clean** | *(none)* |
| `std-cmp` | 111 | 14 | **12.6%** | 14 | **12.6%** | **Clean** | *(none)* |

**Union default-5:** **46 / 236** → **`checked_fraction` 19.5%** (flat vs post-G-α combined
and every prior pilot remeasure on this set).

### Expansion (std-fs / std-io)

| Target | non-test | checked | `checked_fraction` | emitted | `expressible_fraction` | File class | First / representative oracle poisons |
|--------|---------:|--------:|-------------------:|--------:|-----------------------:|------------|----------------------------------------|
| `std-fs` | 47 | 28 | **59.6%** | 28 | **59.6%** | **Clean×7** | *(none)* |
| `std-io` | 59 | 24 | **40.7%** | 24 | **40.7%** | **Clean×5** | *(none)* |

**Union all-7:** **98 / 342** checked → **28.7%**; **98 / 342** emitted → **28.7%**.

#### First-poison list (every CheckError file)

| Crate | File | First diagnostic (truncated) |
|-------|------|------------------------------|
| *(none)* | *(none)* | **Zero CheckError files** across the 7-target pilot set |

**Cleared vs post-G-α combined (#1695+#1696 measure):**

1. **`unknown function/constructor/prim read_to_end`** — **gone** as oracle + phylum
   first poison on both `io.myc` and `lib.myc`. G-β (#1700) **no-fabricate**: method-call
   free-fn desugar emits only when the callee is a proven same-file emission; unregistered
   methods (e.g. `read_to_end` after body gapped on `.to_vec()`) now **gap with EXPLAIN**
   rather than emit bare names that file-poison `myc-check`.
2. std-io file class **CheckError×2 → Clean×5**; phylum ok **false → true**.
3. std-io `checked_fraction` **23.7% → 40.7%** (**+17.0pp**, +10 checked items) — the
   file-gated numerator recovers the previously zeroed `io` + `lib` emissions once Clean.

**Emission honesty (not silent progress on expressible):**

- std-io emitted **27 → 24** (−3): the fabricated `read_all` body
  `Ok(read_to_end(src))` (and related fabrications) no longer count as expressible.
- all-7 expressible **29.5% → 28.7%** (−0.8pp) while checked **25.7% → 28.7%** (**+3.0pp**).
  Checked caught up to expressible: **checked == expressible** on every target in this
  pilot set (no CheckError file-gating residual).

### Phylum dual-report (`Empirical`)

| Target | `checked_fraction_phylum` | oracle `checked_fraction` | Δ_basis | phylum ok |
|--------|--------------------------:|--------------------------:|--------:|-----------|
| std-time / rand / cmp | = oracle | = oracle | +0.0pp | true |
| **std-fs** | **59.6%** | **59.6%** | **+0.0pp** | **true** (all 7 Clean) |
| **std-io** | **40.7%** | **40.7%** | **+0.0pp** | **true** (all 5 Clean) |

eval/fuse are single-file l1 sources (phylum field null in `vet.json`); oracle Clean.

**Emission witness (post G-β `io` gaps, not fabricate):**

```text
# io.gap.json — item read_all (category Other):
method call `.read_to_end(...)` has no proven-emitted free-fn referent in this file
… Emitting bare `read_to_end(recv, …)` would fabricate an unknown prim …
# parent residual on impl Source method read_to_end:
Rust `.to_vec()` … no verified bare-call Seq-copy prim … (M-1037 residual, G2/VR-5)
```

`read_all` is **not** co-emitted as a Clean fn with a bare unknown callee. The Rank-A
poison stop holds. Rank B (honest `to_vec` / Seq-copy prim) was **not** attempted in G-β.

## Comparison vs post-G-α combined (#1695+#1696 @ 25.7% all-7)

| Metric | post-G-α combined `22a7675c` | post-G-β tip `4acd3a20` | Δ |
|--------|----------------------------:|-----------------------:|---|
| default-5 `checked_fraction` | **19.5%** (46/236) | **19.5%** (46/236) | **0.0pp** |
| all-7 `checked_fraction` | **25.7%** (88/342) | **28.7%** (98/342) | **+3.0pp** |
| all-7 `expressible_fraction` | **29.5%** (101/342) | **28.7%** (98/342) | **−0.8pp** |
| std-fs checked / expressible | 59.6% / 59.6% | 59.6% / 59.6% | 0 |
| std-io checked / expressible | 23.7% / 45.8% | **40.7%** / **40.7%** | **+17.0pp / −5.1pp** |
| std-io file class | CheckError×2, Clean×3 | **Clean×5** | file-poisons **closed** |
| std-io first poison | `read_to_end` (both files) | *(none)* | Rank A **closed** |
| std-io phylum ok | false | **true** | residual **advanced** |
| CheckError files (all-7) | 2 | **0** | pilot oracle clean |

### What G-β moved (honest)

1. **#1700 no-fabricate method-call:** Closed the **oracle + phylum** first-poison
   `unknown … read_to_end` by refusing to emit unregistered bare method names. std-io
   `io`/`lib` go Clean; file-gated checked recovers +10 items on std-io.
2. **Expressible dropped, checked rose:** Expected under honesty — fabricated emissions
   that failed myc-check no longer inflate expressible. Headline port-accuracy metric is
   **`checked_fraction`** (DN-34 §8.7); it moved **+3.0pp** all-7.
3. **Default-5 flat:** G-β targeted the expansion residual, not the cmp Macro wall or
   fuse zero-emission profile.
4. **Next residual is non-emission heat**, not an unknown first-poison on this pilot set.

**Still not one-shot.** Default-5 floor holds at 19.5%; fuse stays 0/12; cmp Macro heat
unchanged; full M-1006 ladder **not** run; no SemVer claim.

## Ranked residual list for G-γ (`Declared` ranking on Empirical diagnostics)

Gap-category tallies (union of per-target gap records; eval/fuse single-file +
union.gap.json for crate targets; **514** total records):

| Count | Category |
|------:|----------|
| 85 | DeriveSatisfied |
| 68 | Other |
| 64 | MacroInvocation |
| 62 | DeriveAttr |
| 55 | Impl |
| 44 | NamedFieldDrop |
| 37 | Import *(gap-profile / co-include refusals — not oracle first-poison)* |
| 26 | MultiStmtBody |
| 15 | TestItem *(denominator-excluded)* |
| 13 | GenericBound |
| 10 | Struct / ModuleDecl |
| 5 | MacroDef |
| … | (remainder ≤7 each) |

Ranking weights **file-poison impact on `checked_fraction`** first (none remain on this
pilot set), then **emission-ceiling heat** that would move checked once items emit Clean.

| Rank | Class / poison family | Evidence (post G-β tip) | Suggested next leaf |
|-----:|----------------------|------------------------|---------------------|
| **1** | **MacroInvocation / MacroDef** (cmp wall) | 64 MacroInvocation + 5 MacroDef; std-cmp 12.6% ceiling on largest Clean pilot; design-gated **M-875** | **G-δ / M-875** after design Accepted — not G-γ implement |
| **2** | **Method-call Rank-A cascade** (no proven free-fn) | Post no-fabricate, honest gaps: `.ok_or` (5), `.contains` (3), `.starts_with`/`.trim_end_matches`/`.min`/`.map_err` (2 each), plus single-site `.read_to_end`/`.to_vec`/`.remaining`/… Emission-ceiling on time/fs/io/rand; **not** file-poison | **G-γ L2: ranked prim_map / combinator rows or body rewrite** for high-count methods (start with `ok_or`) |
| **3** | **M-1037 Seq-copy / `.to_vec`** | Parent of former `read_to_end` fabricate path; gapped never-silent; blocks honest `read_to_end` emission | **G-γ or dedicated M-1037** — only with verified prim, never fabricate |
| 4 | **Impl** + trait-impl refusals | Impl 55; Display/From/Error trait-impls refused (DN-34 §8.8) | trait surface / later |
| 5 | **DeriveSatisfied / DeriveAttr heat** | 85+62 — emission heat, not file-poison | later / only if new file poison |
| 6 | **NamedFieldDrop** / records | 44 (fs/io/time heat) | struct-field emission residual |
| 7 | **MultiStmtBody** | 26 (rand/io/fs/serialize heat) | body-lowering residual |
| 8 | **fuse zero-emission** | 0/12 profile | not a file-poison; low pilot weight |
| 9 | **Import gap-profile** | 37 records remain as **non-emission** heat; co-include refuses names sibling files gapped | do not re-open G-α Rank-2 path unless a new Import diagnostic reappears as **oracle** first-poison |
| 10 | **Fuller M-1006 ladder** | Pilot set only; G2 program gate needs ≥1 full ladder table | **G-γ measure** on expanded target set |

### Top next implement / measure leaves (orch pick order — `Declared`)

1. **G-γ measure** — expand beyond default-5+fs/io so ranking is not pilot-biased (G2).
2. **High-count method residuals** (`ok_or`, path string methods, `map_err`) — Rank-A
   cascade after no-fabricate; each needs proven emission or honest gap (never bare name).
3. **M-1037 `.to_vec` / Seq-copy** — unlocks honest `read_to_end` emission path if desired.
4. **M-875 implement** only after design **Accepted** (G-δ) — largest cmp non-emission wall.
5. **Do not** re-spawn Result ambient, Import non-type, or bare `read_to_end` fabricate for
   this residual set — those first-poisons are **Empirical-closed** on this tip.

## G1 gate assessment (`Declared` judgment on Empirical table)

Program criterion ([`PROGRAM-SELFHOST-DECOMPOSE-2026-07-17.md`](./PROGRAM-SELFHOST-DECOMPOSE-2026-07-17.md)):

> G1 | Default pilot set + std-fs/io: **file Clean** or residual **EXPLAIN + ranked**,
> no unknown first-poison classes we can close without design gates

| Sub-check | Result |
|-----------|--------|
| Default pilot file Clean | **Yes** — eval/fuse/time/rand/cmp all Clean (fuse zero-emit profile, not CheckError) |
| std-fs file Clean | **Yes** — Clean×7, phylum ok |
| std-io file Clean | **Yes** — Clean×5, phylum ok (**new post G-β**) |
| Residual EXPLAIN + ranked | **Yes** — gap JSON carries EXPLAIN reasons; G-γ rank table above |
| Closable unknown first-poisons without design gates | **None remaining** on this pilot set |

**G1 on this pilot set: PASS (Empirical evidence on tip `4acd3a20`).**

Caveats (never-silent):

- G1 is **not** “all gaps closed” and **not** one-shot readiness (G5 still forbids that claim).
- G2 (full M-1006 ladder remeasure), G3 (top residual classes closed or design-gated),
  G4 (remote CI) are **out of this leaf’s claim**.
- Remaining heat is **emission ceiling** (Macro / method Rank-A / Impl / …), not oracle
  unknown-first-poison on the pilot set. Closing it is G-γ/G-δ work, not a silent G1 fail.

## Residual FLAGs (orch-owned — do **not** edit `issues.yaml` here)

| Item | FLAG |
|------|------|
| **M-1006 / G-β measure** | Post-merge remeasure on `4acd3a20`: default-5 **19.5%** flat; all-7 checked **28.7%** (**+3.0pp** vs post-G-α 25.7%); std-io Clean×5 / **40.7%**; zero CheckError; G1 pilot **PASS**. Full 17-target ladder **still not** run. Append this path + results dir to `doc_refs` at integration close-out. |
| **#1700 G-β no-fabricate** | Landed; Rank-A `read_to_end` first-poison **closed**; expressible −3 on io is **honest**, not a regression to hide. |
| **Next residual rank 1 (implement heat)** | **MacroInvocation** (design-gated M-875) **or** Rank-A method cascade (`ok_or` …) if orch prefers non-design-gated G-γ leaf. |
| **Next residual rank 1 (measure)** | Full M-1006 ladder for G2. |
| **One-shot / SemVer** | **Forbidden** on this evidence. |
| **Shared files** | FLAG: `tools/github/issues.yaml` body/`doc_refs`, `CHANGELOG.md`, `docs/Doc-Index.md` — integrating parent. This leaf owns only `docs/planning/` + `experiments/results/`. |

## Method notes (VR-5)

- File-gated metric: any `CheckError` on a `.myc` file zeros that file’s checked numerator.
  With **zero** CheckError files, checked equals expressible on every target here.
- Closing a first-poison by **no-fabricate** is residual advancement **and** can lower
  expressible while raising checked — both must be reported (G2: never silent).
- Phylum Δ_basis is a **basis CORRECTION** when it recovers false-fails (DN-124 §4); here
  Δ_basis stays 0 — oracle and phylum agree, both Clean on fs/io.
- Gap-class tallies are **Empirical counts of gap records**, not a probability model of
  “fix next.”
- Ranking that prioritizes **oracle file poisons** (none left) then emission heat is
  **Declared** judgment on top of Empirical heat.
- Comparison anchor is the G-α combined measure at all-7 **25.7%** checked (document
  path above), not a fabricated baseline.

## Orchestrator actions suggested

1. Record L2-M: **G-β Rank-A first-poison closed**; all-7 checked **+3.0pp**; G1 pilot
   **PASS**; next wave **G-γ** (ladder measure and/or method Rank-A cascade).
2. Do **not** flip SemVer / one-shot readiness; do **not** re-open Result / Import /
   fabricate-`read_to_end` as Rank-1 for this pilot set unless a regression reintroduces
   those diagnostics.
3. Prefer a **G-γ measure** on the fuller M-1006 set before large implement waves so
   ranking is not pilot-biased.
4. M-875 remains **design-gated** (G-δ) until Accepted.
