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
- **`gh-bootstrap-local.sh`** — run **locally with `gh`**, it creates any missing milestone/label, then
  ensures the issues exist and **assigns each issue's milestone**. Idempotent (create-if-absent).

> **Why local `gh` and not the agent:** the GitHub **MCP server cannot create milestones or
> colored/described labels** (it can only *assign* an issue to a milestone that already exists, by
> number). So milestone/label creation is split to `gh-bootstrap-local.sh`. `idmap.tsv` records the
> issue-id map; milestones are assigned by the script, not by the MCP runner.

To create every milestone + label and assign every issue in one pass:

```sh
bash tools/github/gh-bootstrap-local.sh          # needs `gh` authenticated to tzervas/mycelium
```

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
| 7 | Runtime & Concurrency Execution Model (RFC-0008) | **active** (RFC-0008 **Accepted** 2026-06-16) | RFC-0008 ratified RT1–RT7 (M-355 ✓); RFC-0014's single-task boundary lifted to per-task budgets, cancellation, and `reclaim` bounded-cascade supervision (M-356 ✓, RFC-0008 §4.7 / Erlang-OTP grounding, Research Record 05); the RT2 deterministic fork/join runtime + sequentialization differential v0 (M-357 ✓; typed channels = next slice); and the **DN-06 lexicon migration** — static keyword `colony` → `nodule`, introduce `phylum`, free `colony` for the dynamic grouping (M-358, staged). The §4.5 vocabulary (hypha/fuse/xloc/cyst/graft/forage/backbone/mesh/tier/reclaim) decomposes at the Phase-7 gate. Gate: RFC-0008 Accepted ✓; RT2 differential holds (NFR-7-equiv) ✓; guarantees tagged honestly (RT5/VR-5); kernel small (KC-3). |

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
| 7 | runtime & concurrency (RFC-0008, **Accepted**) — M-355 ratify ✓, M-356 concurrency/supervision ✓, M-357 RT2 differential ✓; M-358 lexicon migration + M-359 structured nodule header/manifest staged; §4.5 vocabulary decomposes at the gate | 5 |
| 8 | (anticipated) toolchain & release engineering — the full-fat suite (format/correctness/lint+fix/security) + packaging; M-361 epic anchor, M-359 metadata substrate | 1 |

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

- **2026-06-16 — Phase 8 added + Phase 7 reconciled.** New **Phase 8 — Toolchain & Release Engineering**
  (anticipated) for the maintainer's "full-fat" toolchain (format / correctness / lint+fix / security +
  packaging), anchored by **M-361** (epic) with **M-359** (the structured nodule header + `phylum.toml`
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
