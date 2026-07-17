# Gap analysis — 2026-07-16 (transpile-to-Mycelium readiness)

| Field | Value |
|---|---|
| **Status** | **Design pack active** — implement waves **paused** for design quality; ONESHOT prep remains prior implement program (Epic R / one-shot claim still **HOLD**) |
| **Active design phase** | **Three docs only** — `DESIGN-01` · `DESIGN-02` · `DESIGN-03` (below) |
| **Active implement handoff** | **`PROGRAM-HANDOFF-ONESHOT.md`** (resume L0↔L1 after council capture + workstream re-rank) |
| **Prior handoff** | `PROGRAM-HANDOFF.md` — ORACLE-R1 close-out + CI host-witness (history; not the active spawn packet) |
| **Framework** | Repo-root **`maint-guide.md`** (L0→L1→L2; Phase 0–3; PM close-out) |
| **Model floor** | Implement agents: `grok-composer-2.5-fast`. L0 may use `grok-4.5`. Design capture complete in three-doc pack |
| **Goal** | (1) **Now:** review distilled ergonomics design (swaps/policy, tags/containment, diagnostics/UX). (2) **After steer:** re-rank waves and resume honest whole-repo transpile prep — **not** one-shot claim until DoD + release gate |

## Design pack (maintainer review — these three only)

| Doc | Topic |
|-----|--------|
| [`DESIGN-01-SWAPS-AND-POLICY.md`](./DESIGN-01-SWAPS-AND-POLICY.md) | Swaps + **policy streamline** (catalog, default, resolve-and-record) |
| [`DESIGN-02-TAGS-META-AND-CONTAINMENT.md`](./DESIGN-02-TAGS-META-AND-CONTAINMENT.md) | Tags/Meta + **honesty-poison containment** |
| [`DESIGN-03-MACHINERY-DIAGNOSTICS-AND-UX.md`](./DESIGN-03-MACHINERY-DIAGNOSTICS-AND-UX.md) | AX ranks + **first-fault diagnostics** + broader UX |

**Reading order:** start with **03** (§2 map + §8 steering board), then 01/02 as needed.

Exactly **three** files. Prior agent annexes (A–F) and Draft DN-141 were distilled into this pack and
removed. All **Draft** — not Accepted. Mermaid diagrams for mental models.

## Other structure (program management)

| Doc | Role |
|-----|------|
| **`PROGRAM-HANDOFF-ONESHOT.md`** | Implement handoff (resume after council) |
| `PROGRAM-HANDOFF.md` | ORACLE-R1 residual close-out (history) |
| `M1006-baseline-oneshot-2026-07-16.md` | Post-A5 Empirical baseline |
| `M1006-remeasure-post-B1B2-2026-07-16.md` | Post B1+B2 remeasure |
| `M1006-remeasure-post-C3C4-2026-07-16.md` | Post C3+C4 remeasure |
| `WAVE-L0-ORCHESTRATION-2026-07-16.md` | L0 wave map |
| `PARTITION.md` | epic scopes → crate ownership |
| `WAVE-EXPRESSIBILITY-NEXT.md` | critical path backlog |
| `leaves/<crate>.md` | per-crate leaf reports |
| `SYNTHESIS.md` / `SYNTHESIS-G2.md` | orch integration after G1/G2 |

## Prior assessments (read, do not rewrite)

- `docs/planning/language-completeness-gap-inventory.md`
- `docs/planning/DN-136-phase2-bulk-gap-close-worklist.md`
- `docs/notes/DN-99-Surface-Gap-Closure-Register.md`
- `docs/planning/zero-hand-port-delta-ledger.md`
- `docs/notes/DN-34-Rust-to-Mycelium-Transpiler-Strategy.md`
- `docs/CURRENT-STATE.md`
- Repo-root `maint-guide.md` + `.claude/kickoffs/_doc-maintenance.md`
