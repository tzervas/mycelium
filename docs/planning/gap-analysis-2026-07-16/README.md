# Gap analysis — 2026-07-16 (transpile-to-Mycelium readiness)

| Field | Value |
|---|---|
| **Status** | **Active implement program** = selfhost decompose (below). **Design pack 01–04** remains design review (Draft). Epic R / one-shot claim still **HOLD** until G-gate. |
| **Active implement program** | **`PROGRAM-SELFHOST-DECOMPOSE-2026-07-17.md`** — phases **G → A → D → T → R** |
| **Active design phase** | **Four docs** — `DESIGN-01` · `02` · `03` · `04` (review only; not implement blockers except G-δ) |
| **Prior implement handoff** | `PROGRAM-HANDOFF-ONESHOT.md` (ONESHOT prep history; subsumed for new spawn briefs) |
| **Earlier handoff** | `PROGRAM-HANDOFF.md` — ORACLE-R1 close-out + CI host-witness (history) |
| **Framework** | Repo-root **`maint-guide.md`** (L0→L1→L2; Phase 0–3; PM close-out) |
| **Model floor** | Implement agents: `grok-composer-2.5-fast`. L0 may use `grok-4.5`. |
| **Goal** | Gap-close monorepo → archive `main` → component repos → `*-myc` transpile → umbrella re-export (self-hosted presentation). **Honesty:** no one-shot / SemVer claim until G1–G5. |

## Active implement program

| Doc | Role |
|-----|------|
| [`PROGRAM-SELFHOST-DECOMPOSE-2026-07-17.md`](./PROGRAM-SELFHOST-DECOMPOSE-2026-07-17.md) | **L0 program** — G gap-close → A archive → D decompose → T `*-myc` → R re-export; honest gates + mermaid |

**Reading order for implement:** program file Phase G residual waves first; design pack only when a residual is design-gated (G-δ).

## Design pack (maintainer review — these four only)

| Doc | Topic |
|-----|--------|
| [`DESIGN-01-SWAPS-AND-POLICY.md`](./DESIGN-01-SWAPS-AND-POLICY.md) | Swaps + **policy streamline** |
| [`DESIGN-02-TAGS-META-AND-CONTAINMENT.md`](./DESIGN-02-TAGS-META-AND-CONTAINMENT.md) | Tags/Meta + **honesty-poison containment** |
| [`DESIGN-03-MACHINERY-DIAGNOSTICS-AND-UX.md`](./DESIGN-03-MACHINERY-DIAGNOSTICS-AND-UX.md) | AX ranks + **first-fault diagnostics** + broader UX |
| [`DESIGN-04-LEDGER-RETENTION-AND-OFFLOAD.md`](./DESIGN-04-LEDGER-RETENTION-AND-OFFLOAD.md) | **Language/runtime internal** cert/Meta/trace retention (not app logs) |

**Reading order:** **03** (map + steers) → **01/02** as needed → **04** for long-run **language-internal** retention.

Design pack is **four** files (01–04). Prior agent annexes and Draft DN-141 were distilled into 01–03.
**04** is language/runtime memory only — app/ops log pipelines are out of scope. All **Draft** — not
Accepted. Mermaid diagrams included.

## Other structure (program management)

| Doc | Role |
|-----|------|
| **`PROGRAM-SELFHOST-DECOMPOSE-2026-07-17.md`** | **Active** implement program (G→A→D→T→R) |
| `PROGRAM-HANDOFF-ONESHOT.md` | Prior implement handoff (ONESHOT prep history) |
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
- `docs/notes/DN-88-Component-Repo-Decomposition-And-Managerial-Re-Export-Topology.md`
- `docs/CURRENT-STATE.md`
- Repo-root `maint-guide.md` + `.claude/kickoffs/_doc-maintenance.md`
