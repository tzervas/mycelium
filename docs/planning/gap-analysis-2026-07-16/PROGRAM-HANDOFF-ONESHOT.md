# Program handoff — ONE-SHOT transpile prep (2026-07-16)

| Field | Value |
|-------|--------|
| **Repo scope** | **`tzervas/mycelium` only** — no other repos, no DN-88 component split until one-shot prep gate |
| **Framework** | `maint-guide.md` · L0→L1→L2 · model floor `grok-composer-2.5-fast` |
| **L0** | PM/orchestrator only — no product self-implement |
| **Goal** | Language + transpiler **fully prepared** for hands-off whole-repo transpile (gap-profiler honesty preserved: never fabricate). Prep for later component-repo decomposition — **decomposition itself is out of this program** |
| **Headline metric** | **`checked_fraction`** (live `myc-check`) over pilots → port surface → whole corpus ladder (M-1006). `expressible_fraction` secondary |
| **Honesty** | VR-5/G2: gap + EXPLAIN over silent wrong; one-shot **claim** only when gate below is met |
| **Base tips** | `main` `aad96b7a` · `dev` `788574ab`+ (#1657–#1661; B1+B2 landed) · `integration` (promote after batch) |
| **Active baseline** | [`M1006-baseline-oneshot-2026-07-16.md`](./M1006-baseline-oneshot-2026-07-16.md) (post A5); **post-B remeasure** [`M1006-remeasure-post-B1B2-2026-07-16.md`](./M1006-remeasure-post-B1B2-2026-07-16.md) |

## What “100% one-shot prep” means here (DoD)

Not “every line checks green today,” but:

1. **Every construct class** on the first-party surface has a **named native strategy** (DN-111: Native / Idiomatic / Approximation / Bridge) — Accepted+built or honest gap with EXPLAIN.
2. **Transpiler** is a polished gap profiler: Import net-closed (M-1084), conversions non-fabricating (M-1037), derives/macros strategies (M-1086 landed / M-875 design→build), format path (M-1090 remeasure), const/lattice/Show residual classes from ORACLE-R1 generalized.
3. **M-1006 ladder** phases run with Empirical tables; whole-corpus residual is **ranked and shrinking**, not unknown.
4. **Self-host oracle** path clear: M-740 / M-993 drafts can be *profiled* by transpile-vet without file-poisoning the entire module on known classes.
5. **Release gate**: tip-bound remote CI green + no over-claim; SemVer when L0 authorizes — separate from “prep complete.”

## Done foundation (do not re-do)

ORACLE-R1 A1–A5 (#1647–#1651), promote #1652, maint-guide #1653–#1656. Post-A5 default pilots (this program’s C0 baseline):

| Target | checked% | expressible% | File |
|--------|----------|--------------|------|
| std-cmp | 12.6% | 12.6% | Clean |
| std-rand | 17.6% | 17.6% | Clean |
| std-time | **45.9%** | 45.9% | Clean |
| eval.rs | **21.4%** | 21.4% | Clean |
| fuse.rs | 0.0% | 0.0% | Clean (zero emission) |
| **union default-5** | **19.5%** (46/236) | **19.5%** | all Clean |

See `M1006-baseline-oneshot-2026-07-16.md` for expanded std-fs/std-io + residual ranking. **No one-shot claim** from these numbers.

## Done this cycle (ONESHOT #1657–#1661 + B3 remeasure)

| PR | Leaf | Result |
|----|------|--------|
| **#1657** | **B5** M-875 expand-first design draft | `M875-expand-first-design-DRAFT.md` landed; status stays **needs-design** (no implement) |
| **#1658** | **C0** handoff + M-1006 post-A5 baseline | `PROGRAM-HANDOFF-ONESHOT.md` + `M1006-baseline-oneshot-2026-07-16.md` + results dir; default-5 **19.5%** Clean |
| **#1659** | **B1** M-1084 Import full-path use emit | `use_emit_qualifier` identity on full nodule path; **emit form fixed** |
| **#1660** | **B2** M-1037 conversion residual | `to_string` Exact(Bytes) + literal typing; honesty gaps for `into`/`to_vec`; never fabricate |
| **#1661** | **PM** B1 close-out | tracker honesty after #1657–#1659 |
| **(this PR)** | **B3** M-1006 remeasure post B1+B2 | `M1006-remeasure-post-B1B2-2026-07-16.md` + `experiments/results/m1006-remeasure-post-b1b2/`; unions **flat 19.5% / 17.0%** vs baseline-oneshot; residual ranked (derive/`eq_*`, Import phase-2 resolve, Bool `or`, Macro) — **no one-shot claim** |

## Wave map (binding order)

### Epic B — Transpile net-close (**serial** on `crates/mycelium-transpile`)

| Order | Leaf | M-id | Owns | DoD |
|------:|------|------|------|-----|
| B1 | Import net-close | M-1084 | `symtab.rs` + minimal emit/batch | **#1659 landed** — full-path use emit fixed; residual is consumer resolve / single-file oracle (see B3) |
| B2 | Conversion identity residual | M-1037 | `emit`/`prim_map` + tests | **#1660 landed** — Bytes `to_string` + honesty gaps; pilot unions unchanged (B3) |
| B3 | Post-B remeasure | M-1006 / M-1090 | `docs/planning/` + `experiments/results/` only | **This leaf** — Empirical default-5+fs/io; flat vs baseline-oneshot; FLAG issues.yaml |
| B4 | Derive residual audit | M-1086 | verify-first (status **done**) | Confirm remaining DeriveAttr gaps; only open code leaf if residual real |
| B5 | Expand-first **design** | M-875 | planning draft only | **#1657 landed** — draft path; status stays **needs-design**; **no implement** until Accepted |

### Epic C — Corpus ladder (after B1 at least; parallel docs OK)

| Leaf | Owns | DoD |
|------|------|-----|
| C0 baseline remeasure | planning + results only | **This leaf** — post-A5 Empirical table; ranked residual classes |
| C1 M-1006 phase plan + run | planning + results + transpile-vet scripts | Bounded target set (stdlib crates + l1 frontend modules); ranked gap classes; Empirical JSON |
| C2 Top gap-class close | transpile only, one class per leaf | Serial after measure; each leaf one class |

### Epic D — Language surface prep (parallel if disjoint)

| Leaf | Notes |
|------|-------|
| D1 M-1090 remeasure / float OQ residual | After B3; verify WU-3 enough |
| D2 `lib/std` / prelude gaps flagged by C1 | One phylum or nodule per leaf |
| D3 M-740 stage readiness | Profile `lib/compiler` + l1 sources via transpile-vet; FLAG port stages — large, multi-wave |

### Epic I / R

- **I:** promote `dev`→`integration` after each batch of 1–6 PRs
- **R:** SemVer / one-shot **claim** only when DoD above + tip-bound CI — L0 authorize

## L1 spawn order (this cycle)

1. **L1-C0** — handoff + baseline remeasure (docs/results; no emit) — **#1658 done**
2. **L1-B** — **B1 #1659 + B2 #1660 landed**; **B3 remeasure** (this leaf) records flat unions + residual rank
3. **L1-D-design** — M-875 design draft only — **#1657 done** (needs-design residual)
4. **Next implement** (after B3): derive/`eq_*` · Bool `or` · Import phase-2 resolve — see B3 ranked table; **not** one-shot

## Process

- Inject `maint-guide.md` + this handoff into every L1 prompt
- L2: worktree, one leaf, PR→`dev`, no merge
- L0: review, merge, promote, GHA
- Mycelium-only paths under `/root/git/isolated/mycelium`

## Prior handoff (superseded as *active* program pointer)

[`PROGRAM-HANDOFF.md`](./PROGRAM-HANDOFF.md) remains the ORACLE-R1 / close-out packet (CI witness, Epic R HOLD). **Active program for new leaves is this ONESHOT file.**

## Changelog

| When | Note |
|------|------|
| 2026-07-16 | ONESHOT program opened from ORACLE-R1 foundation; Epic B binding path |
| 2026-07-16 | C0: land handoff + post-A5 M-1006 baseline (`M1006-baseline-oneshot-2026-07-16.md`) |
| 2026-07-16 | **#1657** B5 M-875 design draft; **#1658** C0 baseline; **#1659** B1 M-1084 full-path use emit; PM close-out: M-1084/M-1006/M-875 tracker honesty (emit fixed; 768 remeasure pending; needs-design kept) |
| 2026-07-16 | **#1660** B2 M-1037 conversion residual; **#1661** PM B1 close-out; **B3** M-1006 remeasure post B1+B2 — unions flat **19.5% / 17.0%** vs baseline-oneshot; residual rank for next implement leaves (no one-shot claim) |
