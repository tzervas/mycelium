# Kickoff `mem` — mycelium-tero: The Transparent Memory Substrate & Agent Knowledge API (DN-87)

> **Base: `dev`.** Branch off the current `dev` tip, one isolated worktree per concurrent agent,
> merge to `dev`, promote `dev → integration → main` per the tiered workflow. Fire in a **fresh
> session** via `/kickoff mem`. **▶ FIRED 2026-07-07 — named `mycelium-tero`** by the maintainer
> (a quiet homage to Atsushi Tero, for his contribution to science and engineering — the name is
> sugar; the system is code). Scaffold landed with the naming PR; wave 1 = the M-1015 lane.

## Mission

Build DN-87's program: the project corpus (methodologies, decisions, intents, language docs,
tracker state) converted into a **generated, transparent, provenance-carrying encoding** that
*supplements* the human-friendly format — search/access **improved upon RAG** (the claim is
`Empirical`-gated, DN-87 §6) — exposed as a **secure, platform-agnostic API** (MCP + HTTP) with an
optimized skill set, so any agent platform loads *cited answers* instead of re-reading the corpus.
MIT-only.

## The wave (the maintainer's requested shape — CLAUDE.md §Fractal Swarm + §Concurrent-PR)

One orchestrator; one **epic-orchestrator lane per issue**, each spawning issue/change-scoped
**Sonnet leaf agents** in isolated worktrees, disjoint dirs by construction; every common-touch or
conflicting file owned by the **lowest single common parent**; change-scoped PRs (each `/pr-land`
agent-reviewed) worked up the tree `dev → integration → main`.

| Lane | Issue | Owns (disjoint) |
|---|---|---|
| L1 index | **M-1015** | `crates/mycelium-tero/**` (core) · `tools/tero/**` (Python ingestion) |
| Query+provenance | **M-1016** | the crate's query modules (after M-1015's skeleton lands) |
| API+skills | **M-1017** | `crates/tero-api/**` (or a bin) · `.claude/skills/tero-*/**` |
| VSA layer+eval | **M-1018** | the crate's vsa modules + `experiments/tero-eval/**` |
| .myc package | **M-1019** | `lib/` target — **blocked on M-993 by design**, not wave-1 |

Sequencing: M-1015 first (the skeleton every lane builds on) → M-1016/M-1017/M-1018 parallel
(disjoint dirs) → M-1019 phase-gated. Orchestrator owns the shared surface (workspace `Cargo.toml`,
CHANGELOG, Doc-Index, `issues.yaml`, api-index regen).

## House rules (every agent)

- DN-87 §6 binds: **mandatory provenance** (an uncited answer is a refusal), deterministic +
  drift-gated regeneration, the improved-on-RAG claim only after the M-1018 harness verdict,
  token-scoped read-only API default, MIT headers, honest per-layer tags.
- The four §2 design resolutions are `Declared` (orchestrator-resolved) — if the maintainer
  overrides any at naming time, update DN-87 append-only before fanning out.
- Verify against the codebase before implementing (mitigation #14); slots/ids per mitigation #1.

## First steps

1. Receive the name → scaffold `crates/mycelium-tero` (workspace-registered, buildable stub) →
   commit + push the scaffold before fanning out (swarm pre-flight).
2. Confirm DN-87 §2 resolutions stand; flip DN-87's naming field; E39-1 → `in-progress`.
3. Fan out the M-1015 lane; the rest per the sequencing above.
