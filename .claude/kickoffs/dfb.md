# Kickoff `dfb` — Dogfooding Rust-First Builds (`mycelium-web` + `mycelium-adk`)

> Stowed kickoff, UID **`dfb`**. Read `.claude/agent-context.md` + `CLAUDE.md` first.

## Head branch (your locked base)
**`claude/head/dogfood-build`** — protected, persistent base; all work merges into it; `main` is
PR-only; the head → `main` PR is the final step.

## Mission
Build the **Rust-first** `web` phylum (**M-670**, new `crates/mycelium-web`) + `adk` phylum (**M-671**,
new `crates/mycelium-adk`) per RFC-0022 / RFC-0023 — the dogfooding deliverables (web tools + the
Google **Agent Development Kit** port). v1 = Rust crates (RFC-0016 Rust-first order); the
Mycelium-language migration is later (M-502-gated, **not** here).

## Ownership (fully disjoint from `e7l`)
- **You own (NEW):** `crates/mycelium-web/**`, `crates/mycelium-adk/**`, and their member entries in
  the workspace `Cargo.toml`.
- **Read-only / FLAG up:** `issues.yaml`, `CHANGELOG.md`, `Doc-Index.md`.
- **Build on the existing kernel** — `mycelium-core` value model, `std.io` canonical JSON, the
  `mycelium-mlir` runtime (`Scope`/`Colony` from M-357/M-666). Use **Rust** generics + the runtime,
  **not** Mycelium-language generics ⇒ this does **not** depend on `e7l`.

## Gate (cross-work continuity)
**M-670 / M-671 `depends_on` the `dfr` research discharge (RP-10 / RP-9).** You MAY scaffold the crate
structure + non-gated primitives in parallel, but do **not** finalize a design-gated decision until
`dfr` clears it — watch the M-670 / M-671 issue **bodies** the `dfr` session updates.

## Swarm method — scoped to **2 DISJOINT crates → parallel leaves + octopus merge**
**Opus orchestrator** + **one leaf per crate** (web · adk) in isolated worktrees — disjoint dirs ⇒ safe
parallel (the classic octopus pattern; Sonnet leaves are fine for the bounded module work, Opus for the
phylum-shape design). Each leaf builds the five nodules as Rust modules with an honest guarantee matrix
(RFC-0016 §4.5), never-silent fallibility, and a property test per bound. **Octopus-merge** both leaves
into the head; the orchestrator reconciles the shared `Cargo.toml` + matrices once.

## Merge / branch method
leaf → `claude/head/dogfood-build` (octopus, `--no-ff`, pull-down first). Head → `main` PR is the final
step. (`web.json` delegates to `std.io`'s one canonical JSON — no new codec, RFC-0022.)

## Honesty / done
Per-op guarantee matrices: HTTP **Exact**-when-Ok, JSON **Empirical**, server determinism
**Empirical-via-RT2**, handler/tool purity **Declared** — **never `Proven`** without a checked basis.
Never-silent G2 throughout; "implemented Rust-first, pending ratification." **Done** = both phyla green
on the head, matrices asserted, M-670/M-671 bodies + status updated, ready for final integration to `main`.
