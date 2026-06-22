# Kickoffs — tiered `dev → integration → main` workflow

Post-M-662, development runs on a **three-tier branch model** with a **stringency gradient** — messy
below, polished on top — plus **stowed kickoffs** (one per isolated-tree work package) so multiple
**Sonnet swarms** run in parallel across **disjoint crates/directories**, collision-free by
construction (CLAUDE.md §Swarm).

## The tiers (each PR-gated; stringency rises with the tier)

```
feature/leaf  ──PR──▶  dev  ──PR──▶  integration  ──squash-PR──▶  main
 (isolated tree)      (messy OK)      (full gate)                (polished · released)
```

| Branch | Tier | Bar to land here (via PR) | Merge style |
|---|---|---|---|
| **`main`** | release | the **full** `just check` + `/pr-review` + a Copilot round + a **curated squash** — the clean, bisectable, released history | **squash only** (from `integration`) |
| **`integration`** | staging | the **full** `just check` green + honesty / grounding / append-only review; shared files reconciled once | `--no-ff` from `dev` (lineage preserved) |
| **`dev`** | working | **compiles + change-scoped tests pass** — messy is fine: WIP, exploration, octopus/swarm merges | octopus / `--no-ff` from feature/leaf |
| **`feature` / `leaf`** | work | the swarm's own `/dev-workflow` discipline | branched **off `dev`** |

- **`dev` is where work first lands.** Below `integration` things can be messy (WIP commits,
  exploratory branches, octopus swarm merges); only **compiles + scoped tests** is required.
- **`integration` is the promotion gate.** `dev → integration` requires the **full `just check`**
  green + the honesty review — this is where work is polished and the shared files reconciled once.
- **`main` is the release.** `integration → main` is a **single curated squash** (the squash-only-to-
  `main` policy is unchanged), gated by `/pr-review` + the Copilot round. `main` stays clean.
- **Persistent + PR-gated:** `main`, `integration`, `dev` — **no direct push, PR only** (set branch
  protection in the GitHub UI). Everything below `dev` is ephemeral and merges freely (no PR needed).
- **Fast-forward, not force** (CLAUDE.md mitigation #6): keep a session's *working* pointer clean;
  do work on feature/leaf branches; bring the tier branch up with `merge --ff-only` + a plain push.

## Parallel swarms — one kickoff per isolated tree

Each active kickoff **owns a disjoint directory**, so its **Sonnet swarm** (default mode) runs in its
own session/worktree without touching another's files. Fire each in a fresh session with `/kickoff
<uid>`; each branches **off `dev`**, merges its result **into `dev`**, then `dev → integration →
main` promotes it up.

| UID | Kickoff | Isolated tree (owns) | Swarm method | Depends on |
|---|---|---|---|---|
| **`lex`** | `lex.md` | `crates/mycelium-l1/**` · `docs/spec/grammar/**` | Sonnet · **serial-on-L1** (collision files) | — (critical path) |
| **`u78`** | `u78.md` | `crates/mycelium-mlir/**` · `scripts/checks/**` · `justfile` | Sonnet · **parallel-leaf** | — (fully disjoint from `lex`) |
| **`dfr`** | `dfr.md` | `research/12,13` · RFC-0022/0023 Status · `docs/notes/research-prompts.md` | Opus reasoners (docs-only) | — (gates `dfb`) |
| **`dfb`** | `dfb.md` | `crates/mycelium-web/` · `crates/mycelium-adk/` (NEW) | Sonnet · parallel-leaf | `dfr` + `lex` (surface) |

**`lex` ⟂ `u78` ⟂ `dfr` are fully disjoint — fire all three in parallel today.** `dfb` is gated
(needs `dfr`'s research discharged + the `lex` surface). Cross-work continuity rides the **issues**
(`tools/github/issues.yaml` `depends_on` + body notes), never by touching another tree's files.
(`dfr`/`dfb` predate this workflow — ignore their old `claude/head/*` references; they now branch off
`dev` like everything else.)

## Completed (archived)
- **`e7l` / `e7lb` / `e7lc`** — the E7-1/E7-2 L1-surface chain **M-656 → M-662 LANDED** on `main`
  (generics · traits · effects · `wild`/FFI · phylum + cross-nodule). Continued by **`lex`**.

## Reserved (maintainer-only; excluded from every kickoff)
**M-655** (cut the 1.0.0 tag) · **M-381 / M-646** (LLM local runs).
