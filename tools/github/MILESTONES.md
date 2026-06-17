# Mycelium milestone map

The canonical map of **all current and anticipated milestones**, the phase each task lands in, and how
the bootstrap turns this into GitHub milestones. This file is the human-readable overview; the
machine-consumed sources are alongside it.

## How this is consumed (the bootstrap pipeline)

- **`milestones.json`** — the machine source the bootstrap reads to **create** milestones (title, state,
  description). One object per milestone.
- **`labels.json`** — the `phase:N` (and other) labels, created with colors/descriptions.
- **`issues.yaml`** — each task carries a `milestone:` field (its exact `milestones.json` title) and a
  `phase:N` label; this is the **per-issue assignment** source of truth.
- **`gh-bootstrap-local.sh`** — run **locally with `gh`**, it creates any missing milestone/label.
  Idempotent (label = create-or-update; milestone = create-if-absent).
- **`gh-issues-sync.py`** — the **cross-platform reconcile engine** (pure Python + `gh`, no bash/jq,
  so it runs the same in **PowerShell**). Creates any **absent issue** (matched by `idmap` number,
  then title — so a renamed title updates instead of duplicating) **and intelligently updates** an
  existing one to match `issues.yaml`: labels, milestone, title (and body only with `--update-bodies`).
  Bodies are off by default (GitHub bodies accrue enactment notes) and OPEN/CLOSED state is **never
  inferred** from a `status:*` label (only an explicit `state:` field moves it). Appends new rows to
  `idmap.tsv` (append-only). Idempotent + never-silent; `--dry-run` previews; `--all` also does
  labels + milestones; `--self-test` checks the diff logic offline. *(`gh-bootstrap-local.sh` remains
  the bash-native labels/milestones path; the Python engine is its cross-platform superset.)*
- **`gh-sync-all.sh`** / **`gh-sync-all.ps1`** — the **single command** that runs the whole
  reconciliation. The `.sh` (Linux/macOS): manifest preflight → `gh-bootstrap-local.sh`
  (labels + milestones) → `gh-issues-sync.py` (issues create + reconcile + idmap). The `.ps1`
  (Windows/PowerShell): manifest preflight → `gh-issues-sync.py --all` (labels + milestones + issues),
  needing no bash/jq. Use either to close gaps after editing any manifest.
- **`manifest-check.py`** — the preflight: every label/milestone `issues.yaml` references must be
  defined in `labels.json`/`milestones.json`, else an explicit error (a missing label would otherwise
  make `gh issue create --label …` fail mid-run — never-silent, G2).

> **Why local `gh` and not the agent:** the GitHub **MCP server cannot create milestones or
> colored/described labels** (it can only *assign* an issue to a milestone that already exists, by
> number). So milestone/label creation is split to `gh-bootstrap-local.sh`. `idmap.tsv` records the
> issue-id map; milestones are assigned by `gh-issues-sync.py`, not by the MCP runner.

To reconcile the repo with every manifest (milestones + labels + issues + assignments) in **one
idempotent pass** — safe to rerun any time a manifest gains entries:

```sh
# Linux/macOS (bash + gh + jq):
bash tools/github/gh-sync-all.sh                 # needs `gh` authenticated to tzervas/mycelium
bash tools/github/gh-sync-all.sh --dry-run       # preview the reconcile, no repo writes

# Windows (PowerShell + python + gh; no bash/jq needed):
pwsh tools/github/gh-sync-all.ps1                # full reconcile (labels + milestones + issues)
pwsh tools/github/gh-sync-all.ps1 -DryRun        # preview, no repo writes

# Any OS, directly (the engine itself):
python tools/github/gh-issues-sync.py --all --dry-run
```

(`gh-bootstrap-local.sh` on its own still creates only labels + milestones — `gh-sync-all.sh` is the
labels-then-milestones-then-issues superset; the Python engine's `--all` is its cross-platform twin.)

## The milestone ladder

| Phase | Title | Status | Theme & gate |
|---|---|---|---|
| 0 | Confirm & Specify | ratified (Foundation §6) | Close existential/decision risks; ratify spec + schemas; stand up build infra. Gate: M-001 green, M-002 verdict, M-010 ratified, infra in CI. |
| 1 | Minimal Viable Core | ratified (Foundation §6) | Auditable kernel: Core IR + reference interpreter, binary/ternary, certified binary↔ternary swap, 1–2 VSA ops with bounds, minimal toolchain, interp↔AOT equivalence. Gate: SC-1/3/4, SC-2 for shipped ops, NFR-7. |
| 2 | Full Unification & Verified Swaps | ratified (Foundation §6) | All four substrates co-equal; lossy swaps certified per-swap; verified-numerics checker; selection policy + packing selector. Gate: SC-2 all ops, SC-3 global, cert overhead in budget. |
| 3 | Tooling, Projections & Acceleration | ratified (Foundation §6) | Dual-intelligibility tooling, JIT, factorization, BitNet acceleration, native-ternary forward path. Gate: SC-5a/b, JIT NFR-7 + speedup, projections validated. |
| 4 | Interpreted↔Compiled ABI, Hot-Inject & AOT-Fragment Completion | active (cut 2026-06-16) | The interp↔compiled ABI (ADR-016) + hot-inject/recompile (ADR-017); AOT env-machine completion over the full v0 calculus + stack-robustness/dynamic budgets (M-342/347/349, DN-05); mutual recursion (M-343, RFC-0001 r5); ambient (M-344, RFC-0012); diagnostics + recovery (M-345, RFC-0013/0014); stdlib roadmap (M-346). Gate: NFR-7 three-way differential across the calculus; budgets explicit/never-silent (G2); kernel small (KC-3). |
| 5 | Self-Hosting & Core Library | **anticipated** (not yet ratified) | Write the stdlib + diagnostics/recovery runtime in Mycelium-lang itself (dogfooding; "free of other languages"): decompose the M-346 stdlib epic; self-host RFC-0013/0014. Gate: a stdlib module self-hosts with the guarantee/EXPLAIN contract every op must meet (G2, VR-5, KC-3, ADR-003). |
| 6 | Native Acceleration & Deployment | **anticipated** (not yet ratified) | Native MLIR→LLVM codegen for the full calculus incl. data/closure (M-348, RFC-0004 §2, ADR-009); BitNet / native-ternary acceleration; deployable Spore units (ADR-013); production hardening. Gate: native NFR-7 differential + speedup; deployable Spore (VR-4 no-opaque-lowering). |
| 7 | Runtime & Concurrency Execution Model (RFC-0008) | **active** (RFC-0008 **Accepted** 2026-06-16) | RFC-0008 ratified RT1–RT7 (M-355 ✓); RFC-0014's single-task boundary lifted to per-task budgets, cancellation, and `reclaim` bounded-cascade supervision (M-356 ✓, RFC-0008 §4.7 / Erlang-OTP grounding, Research Record 05); the RT2 deterministic fork/join runtime + sequentialization differential v0 (M-357 ✓; typed channels = next slice); and the **DN-06 lexicon migration** — static keyword `colony` → `nodule`, introduce `phylum`, free `colony` for the dynamic grouping (M-358 ✓; the structured header/manifest M-359 ✓ and RFC-0015 auto-baseline M-362 ✓ also folded). The §4.5 vocabulary (hypha/fuse/xloc/cyst/graft/forage/backbone/mesh/tier/reclaim) decomposes at the Phase-7 gate. Gate: RFC-0008 Accepted ✓; RT2 differential holds (NFR-7-equiv) ✓; guarantees tagged honestly (RT5/VR-5); kernel small (KC-3). |

Phases 0–4 are in `milestones.json` as active milestones. Phases 5–7 are **forward roadmap anchors**:
they are also created (as empty milestones) so future issues have a home, but their scope is **not yet
ratified** — it firms up at the preceding phase gate, and the descriptions say so. **Phase 7** (Runtime &
Concurrency, RFC-0008) is the newest such anchor (2026-06-16); its sequencing relative to Phases 5–6 is a
gate decision (the runtime track may precede self-hosting). Removing any anchor from `milestones.json`
before a run simply skips creating it; the assignment of current issues is unaffected.

## Task → milestone assignment

The authoritative per-issue assignment is the `milestone:` field on each task in **`issues.yaml`**; this is
the summary (counts as of 2026-06-16):

| Phase | Tasks (themes / id families) | Count |
|---|---|---|
| 0 | foundational specs + probes — M-001/002, M-010/011/012, M-020, … | 9 |
| 1 | minimal viable core — kernel, interpreter, binary/ternary, first swap, LSP | 17 |
| 2 | full unification + verified swaps — Dense/VSA, numerics checker, selection/packing | 7 |
| 3 | tooling/projections/acceleration — M-1xx/M-2xx/M-3xx build tasks, M-350 | 20 |
| 4 | ABI/hot-inject/AOT-completion — M-341/342/343/344/345/346/347/348/349 + M-352; RFC-0008 integration M-353/354 | 12 |
| 5 | (anticipated) self-hosting + stdlib — decomposed from M-346 at the Phase-4 gate | — |
| 6 | (anticipated) native acceleration + deployment — M-348 native path, Spore | — |
| 7 | runtime & concurrency (RFC-0008, **Accepted**) — M-355 ratify ✓, M-356 concurrency/supervision ✓, M-357 RT2 differential ✓, M-358 lexicon migration ✓, M-359 structured nodule header/manifest ✓, M-362 RFC-0015 auto-baseline ✓; §4.5 vocabulary decomposes at the gate | 6 |
| 8 | (anticipated) toolchain & release engineering — the full-fat suite folded as five above-the-kernel crates: M-364 `mycfmt` ✓, M-365 `myc-check` ✓, M-366 `myc-lint` ✓, M-367 `myc-sec` ✓, M-368 `spore` ✓; M-361 epic (CI-parity gate wiring pending) + M-363 authoring pipeline (design Accepted, build unscheduled) | 7 |

A task moves phases by editing its `milestone:` + `phase:N` label in `issues.yaml` (then re-running the
bootstrap, which is idempotent). Keep the `milestone:` string byte-identical to the `milestones.json`
title or the script will create a duplicate.

## Grounding

- **Foundation §6 (Deliverables Roadmap)** ratifies Phases 0–3 and their gates.
- **Phase 4** was cut 2026-06-16 for the enacted M-34x work (ADR-016/017; RFC-0001 r5; RFC-0012/0013/0014;
  DN-05) — see `issues.yaml` "# Phase 4 (M-34x)".
- **Phases 5–7** are anticipated anchors grounded in existing corpus intent: self-hosting / stdlib
  (M-346; DN-04 §3; RFC-0013 §4.8 / RFC-0014 §9); the native path (M-348; RFC-0004 §2; ADR-009;
  ADR-013 Spore); and the **runtime & concurrency** track (Phase 7) grounded in RFC-0008 (RT1–RT7;
  ADR-012 §7.3; Research Record 04), the RFC-0014 §8 concurrency deferral, and RFC-0008 §4.7's "RT2
  runtime as the natural successor to the L1 track" — **not** new ratified roadmap, and marked so.

## Meta — changelog

- **2026-06-17 — `gh-issues-sync.py` grows up: idempotent reconcile + cross-platform (PowerShell).**
  The sync engine no longer just *creates* absent issues — it now **intelligently updates** existing
  ones to match `issues.yaml` (labels, milestone, title; bodies only with `--update-bodies`), which is
  exactly the gap that let the M-358…368 labels drift (the reconciliation below had to be done by hand).
  Matching is `idmap`-number-first then title (a renamed title updates instead of duplicating).
  Honest by construction: never-silent (every change printed; `--dry-run` previews), never-destructive by
  default (bodies opt-in; OPEN/CLOSED **never** inferred from a `status:*` label — only an explicit
  `state:` field moves it), idempotent (in-sync issues untouched). The engine is now **pure Python + `gh`
  (no bash/jq)**, so it runs identically in **PowerShell**; `--all` folds in labels + milestones so a
  Windows user needs no bash. Added **`gh-sync-all.ps1`** (the `.sh` orchestrator's twin) and an offline
  **`--self-test`** for the diff logic. `gh-sync-all.sh` updated to match (honest comments +
  `--update-bodies` passthrough). Tooling only (KC-3); no new dependency (PyYAML already required).
- **2026-06-17 — Issue-state reconciliation: M-358/359/362 + M-364–368 → `status:done`.** Eight tasks
  folded across the Phase-7/8 waves but left labelled `status:needs-design` are flipped to **`status:done`**
  (keeping `type:design` — the M-344/345/352/355 precedent) in `issues.yaml` and on GitHub
  (#130/#131/#133 + #136–#140). **Issues left OPEN** (maintainer's call): the M-361 epic (#132) stays open
  until its CI-parity gate is wired (the next wave), and the five children close with it. `type`/milestone/
  open-state otherwise untouched; since `gh-issues-sync.py` is create-absent-only, the live flip is a direct
  `issue_write`, not a bootstrap run. Ladder + summary corrected (Phase 7 = 6 tasks, all ✓; Phase 8 = 7 — the
  five tool children ✓, M-361 gate + M-363 build pending), resolving the prior contradiction (changelog said
  "enacted"; the ladder still said "staged"). `idmap.tsv` carries no status column, so it gains only a dated
  note. Append-only.
- **2026-06-16 — M-361 decomposed into per-tool children (staged).** The Phase-8 toolchain epic **M-361**
  is decomposed into the five per-tool tasks its body names — **M-364** (`mycfmt` formatter), **M-365**
  (correctness/type-check driver), **M-366** (lint + auto-fix, incl. the RFC-0015 baseline lint + the
  M-363 §4.1 doc quality-bar lint), **M-367** (security checks as tooling), **M-368** (packaging/publishing:
  `mycelium-proj.toml` → spore, ADR-013) — **staged in `issues.yaml`** (sub-issues of M-361; `phase:8` +
  the Phase 8 milestone already exist). `manifest-check.py` passes (78 issues). GitHub issues + sub-issue
  links to M-361 (#132) are created **at the Phase-8 gate** via `gh-sync-all.sh` (the established
  staging→gated-sync flow; the MCP cannot create milestones/colored labels). idmap records the numbers on
  that run. Also reconciled this session: **M-358/M-359 enacted** (DN-06 lexicon migration; the structured
  nodule header + `mycelium-proj` crate, spec **Accepted**) and **M-362 enacted** (RFC-0015 **Accepted** —
  auto-baseline diagnostics/recovery), **M-363 designed** (authoring pipeline spec **Proposed**). Append-only.
- **2026-06-16 — Phase 8 added + Phase 7 reconciled.** New **Phase 8 — Toolchain & Release Engineering**
  (anticipated) for the maintainer's "full-fat" toolchain (format / correctness / lint+fix / security +
  packaging), anchored by **M-361** (epic) with **M-359** (the structured nodule header + `mycelium-proj.toml`
  manifest, Proposed) as its metadata substrate. Phase 7 reconciled: **M-355/356/357 done** (RFC-0008
  Accepted; single-task boundary lifted; the RT2 fork/join runtime + sequentialization differential v0),
  **M-358** (DN-06 lexicon migration) and **M-359** staged. `milestones.json` (Phase 8), `labels.json`
  (`phase:8`), `issues.yaml` (M-359, M-361; M-357 status→done), and the ladder/summary updated. Phase-8
  sequencing firms at a later gate. Append-only; idmap recorded on the next bootstrap run.
- **2026-06-16 — Phase 7 added (RFC-0008 integration roadmap).** Added the RFC-0008 integration tasks to
  the bootstrap mapping: **M-353** (unify the effect-budget mechanism — RFC-0014 §4.8) and **M-354**
  (diagnostic route targets ↔ observability sinks — RFC-0013 §8) join the **active Phase 4** (shovel-ready
  over already-landed machinery, no RFC-0008 ratification needed); a new **anticipated Phase 7 — Runtime &
  Concurrency Execution Model (RFC-0008)** anchors **M-355** (RFC-0008 RT1–RT7 ratification), **M-356**
  (RFC-0014 concurrency: per-task budgets, cancellation, `reclaim` bounded-cascade supervision), and
  **M-357** (RT2 deterministic-fragment runtime + sequentialization differential). `milestones.json`
  (Phase 7), `labels.json` (phase:5/6/7), `issues.yaml` (M-353..M-357), and the ladder/summary/grounding
  tables updated; idmap recorded on filing. Phase-7 sequencing vs Phases 5–6 is a gate decision. Append-only.
- **2026-06-16 — Created.** Captures the full milestone ladder (Phases 0–4 active + 5–6 as anticipated
  forward anchors) in one human-readable map alongside the machine sources (`milestones.json`,
  `labels.json`, `issues.yaml`), so a single `gh-bootstrap-local.sh` run creates every milestone + label
  and assigns every issue. Records the MCP-server limitation (cannot create milestones — assignment by
  number only) and the task→milestone summary. Append-only.
