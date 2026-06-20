# Mycelium — Session Kickoff Prompt

Read `.claude/agent-context.md` for the compact orientation brief (CLAUDE.md wins on any conflict).

## Current wave: M-646 through M-651

Six open issues; all documented in `tools/github/issues.yaml`. Priority order:

| ID | Title | Priority |
|----|-------|----------|
| **M-646** | Close M-381 arm4: standalone LlmCanonical scorer (gets determinate retention ratio) | P1 |
| **M-647** | RFC-0020 scoped ratification: §4.1/4.3/4.4/4.6–4.9 in; §4.2/4.5 carve-out | P1 |
| **M-648** | Editorial sweep: RFC-0016/0017/0021 → Enacted; DN-04/05/10 → Resolved | P2 |
| **M-649** | Self-hosting Stage-2: first stdlib module in `.myc` L1 syntax (Phase 5 gate) | P0 |
| **M-650** | DN-11 Next Wave Plan: Phase 5 summary + Phase 6 road map | P2 |
| **M-651** | Harness→bench schema bridge: Grok report ingestion in mycelium-bench | P2 |

## Swarm guidance

Default mode: **Sonnet Swarm** (all agents Sonnet; CLAUDE.md §Fractal Swarm Development System).

Parallelisable as two independent tracks:
- **Track A** (research/editorial, low blast-radius): M-647 + M-648 + M-650 — docs-only changes; can fan out as leaf agents, each owning a disjoint doc set
- **Track B** (implementation): M-646 + M-649 + M-651 — code + harness changes; can fan out by crate (tools/llm-harness, crates/mycelium-l1, crates/mycelium-bench)

Shared files (orchestrator-owned, read-only for leaves): `CHANGELOG.md`, `docs/Doc-Index.md`, `tools/github/issues.yaml`, `docs/api-index/`.

## Key invariants for this wave

- **Honesty rule (VR-5)**: never pre-write the arm4 retention-ratio verdict (M-646); record results as Empirical from an actual run.
- **Append-only**: every RFC/DN status flip adds a resolution record; no existing text is rewritten.
- **Never-silent (G2)**: every new Rust/Python path that can fail returns `Option`/`Result`/explicit error.
- **Phase 5 gate**: M-649 achieves the gate when ≥1 `.myc` module passes `myc-check` AND has a differential test.

## Branch and PR flow

```
Branch from main → develop → just check → PR → squash to main
```

Always PR into main. Never push main directly. Use `/land` for the final squash step.
