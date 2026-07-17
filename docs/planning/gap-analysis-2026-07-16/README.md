# Gap analysis — 2026-07-16 (transpile-to-Mycelium readiness)

| Field | Value |
|---|---|
| **Status** | **Design council active** (swaps · tags · UX/DX) — implement waves **paused** for design quality; ONESHOT prep remains the prior implement program (Epic R / one-shot claim still **HOLD**) |
| **Active design phase** | **`DESIGN-COUNCIL-SWAPS-TAGS-2026-07-17.md`** — research/design only (not implement); Agent A/B/C deliverables below |
| **Active implement handoff** | **`PROGRAM-HANDOFF-ONESHOT.md`** (resume L0↔L1 after council capture + workstream re-rank) |
| **Prior handoff** | `PROGRAM-HANDOFF.md` — ORACLE-R1 close-out + CI host-witness (still valid history; not the active spawn packet) |
| **Framework** | Repo-root **`maint-guide.md`** (L0→L1→L2; Phase 0–3; PM close-out) |
| **Tree tip** | land against current `origin/dev`; baseline `M1006-baseline-oneshot-2026-07-16.md`; post-B remeasure `M1006-remeasure-post-B1B2-2026-07-16.md` |
| **Model floor** | `grok-composer-2.5-fast` for implement agents; design council agents may be higher-effort (record actual) |
| **Goal** | (1) **Now:** ergonomic design for swaps + required tagging without VR-5/G2 compromise. (2) **After capture:** resume prep for honest whole-repo transpile; **not** one-shot claim until DoD + release gate |
| **Updates** | language-completeness-gap-inventory, DN-136 phase2 worklist, DN-99, zero-hand-port-delta-ledger, DN-34 §8 |

## Structure (program management)

| Doc | Role |
|-----|------|
| **`DESIGN-COUNCIL-SWAPS-TAGS-2026-07-17.md`** | **Active design phase** — council brief (swaps · Meta/lattice tagging · deterministic machinery) |
| **`AGENT-A-SWAPS-ERGONOMICS-2026-07-17.md`** | Agent A — swaps authoring/management/typing options (Draft) |
| **`docs/notes/DN-141-Tagging-Meta-Honesty-Lattice-UX.md`** | Agent B — tagging / Meta / lattice UX as **DN-141 Draft** |
| **`AGENT-C-AX-STACK-SYNTHESIS-2026-07-17.md`** | Agent C — cross-cutting AX-stack synthesis + ranked package (Draft) |
| **`PROGRAM-HANDOFF-ONESHOT.md`** | Implement handoff (resume after council) — one-shot prep DoD, Epic B/C/D |
| `PROGRAM-HANDOFF.md` | ORACLE-R1 residual close-out packet (A1–A5 done, CI witness, Epic R HOLD) |
| `M1006-baseline-oneshot-2026-07-16.md` | **Post-A5 Empirical baseline** (default-5 + std-fs/io) |
| `M1006-remeasure-post-B1B2-2026-07-16.md` | **Post B1+B2 Empirical remeasure** (unions flat 19.5%/17.0%; residual rank) |
| `WAVE-L0-ORCHESTRATION-2026-07-16.md` | L0 wave map (Epic A complete; B/C/R next) |
| `PARTITION.md` | epic scopes → crate ownership |
| `WAVE1-PRIORITY.md` / `WAVE-EXPRESSIBILITY-NEXT.md` | critical path backlog |
| `leaves/<crate>.md` | per-crate leaf report (one agent each) |
| `SYNTHESIS.md` / `SYNTHESIS-G2.md` | orch integration after G1/G2 |
| `M1006-remeasure-*.md` | earlier Empirical remeasures (pre-/mid-ORACLE-R1) |
| `EXPRESS-ORACLE-BLOCKERS-2026-07-16.md` | technical residual notes |

## Prior assessments (read, do not rewrite)

- `docs/planning/language-completeness-gap-inventory.md`
- `docs/planning/DN-136-phase2-bulk-gap-close-worklist.md`
- `docs/notes/DN-99-Surface-Gap-Closure-Register.md`
- `docs/planning/zero-hand-port-delta-ledger.md`
- `docs/notes/DN-34-Rust-to-Mycelium-Transpiler-Strategy.md`
- `docs/CURRENT-STATE.md`
- Repo-root `maint-guide.md` + `.claude/kickoffs/_doc-maintenance.md`
