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
| **`dfr`** | `dfr.md` | `research/12,13` · RFC-0022/0023 Status · `docs/notes/research-prompts.md` | Opus reasoners (docs-only) | — (gates `dfb`) |
| **`dfb`** | `dfb.md` | `crates/mycelium-web/` · `crates/mycelium-adk/` (NEW) | Sonnet · parallel-leaf | `dfr` + the L1 surface |

**`dfr` is ready now** (docs-only, disjoint — fire it any time). `dfb` is gated (needs `dfr`'s research
discharged + the L1 surface). Cross-work continuity rides the **issues**
(`tools/github/issues.yaml` `depends_on` + body notes), never by touching another tree's files.
(`dfr`/`dfb` predate this workflow — ignore their old `claude/head/*` references; they now branch off
`dev` like everything else.)

### Next candidates (unblocked — for the next short-coded kickoffs)
The L1 critical path continues from where `lex` left off. Disjoint, ready to cut as fresh kickoffs:
- **`M-673`** — monomorphization elaboration for generic instantiations (`crates/mycelium-l1/**`). The
  highest-value unblock: it flips **M-657**/**M-659** from checker-only (in-progress) to fully done
  (DN-14 §3 rows 6/7 → `present`) and unblocks **M-649** self-hosting of a *generic* stdlib nodule.
- **`E7-2 R1`** — **M-667** (`fuse`/`reclaim`/`tier`) after M-665/M-666 (`hypha`/`colony`) landed;
  then **M-668** (R2 planning, docs).
- **`M-664`** — `consume`/`grow`/`impl` surface keywords (depends on the M-659 trait checker, landed).
- **`M-649`** — first self-hosted stdlib module: a *non-generic* candidate (`std.ternary`/`std.option`)
  is doable on the current surface **now**; a generic one waits on `M-673`.
- Tooling (lower priority, mostly disjoint): **M-675** (idmap↔GitHub reconcile), **M-676**, **M-677**.

## Completed (archived)
- **`e7l` / `e7lb` / `e7lc`** — the E7-1/E7-2 L1-surface chain **M-656 → M-662 LANDED** on `main`
  (generics · traits · effects · `wild`/FFI · phylum + cross-nodule). Continued by **`lex`**.
- **`lex`** — **M-663 LANDED** on `main` (#375→`dev`, #377→`integration`, #380 release→`main`): RFC-0018
  stage-1a static guarantee grading (`grade.rs` Pass 3d) enacted; RFC-0018 → **Enacted**; DN-14 §3 row
  11 → `present`. Plus a Copilot-caught grade-upgrade soundness fix + the check-tooling packed exit
  codes / failure digest (**DN-22** design capture). Continues via **`M-673`** (above).
- **`u78`** — **M-678 epic (M-679…M-683) LANDED** on `main` (#378): DN-21 unsafe-code hardening —
  all workspace `unsafe` confined to `jit.rs`, the trusted base `#![forbid(unsafe_code)]`-pinned, and
  the `just safety-check` SAFETY-adjacency gate added.

## Reserved (maintainer-only; excluded from every kickoff)
**M-655** (cut the 1.0.0 tag) · **M-381 / M-646** (LLM local runs).
