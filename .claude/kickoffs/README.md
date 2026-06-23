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
| **`run`** | `run.md` | `crates/mycelium-l1/**` (+ one new `.myc`) | Opus · **serial-on-L1** | — (critical path) |
| **`srf`** | `srf.md` | `crates/mycelium-l1/**` · `.claude/memory/lang-lexicon-syntax.md` | Opus · **serial-on-L1** | — (M-659 checker landed) |
| **`tul`** | `tul.md` | `tools/github/**` | Sonnet (docs/tooling) | — (needs GitHub read access) |
| **`dfb`** | `dfb.md` | `crates/mycelium-web/` · `crates/mycelium-adk/` (NEW) | Sonnet · parallel-leaf | `dfr` ✅ (discharged #344) + the L1 surface |

**Parallelism (collision profile):**
- **`run` and `srf` share `crates/mycelium-l1` → they SERIALIZE** (one L1 editor at a time —
  mitigation #7). Run them in **one** session: **`run` first** (it's the critical-path unblock that
  flips M-657/M-659 to done and opens self-hosting), then `srf`. Neither blocks the other; the order
  is by priority.
- **`tul` ⟂ (the L1 track) are fully disjoint — fire them in parallel** (separate sessions).
  `tul` = `tools/github/` only; the L1 track = `crates/mycelium-l1`. (`dfr` — research/docs only — is
  **done**: landed #344, see Completed.)
- **`dfb`** stays gated, but its **research dependency is now discharged** (`dfr` #344) — it needs only
  the L1 surface remaining.

Cross-work continuity rides the **issues** (`tools/github/issues.yaml` `depends_on` + body notes),
never by touching another tree's files. (`dfb` predates this workflow — ignore its old
`claude/head/*` reference; it now branches off `dev` like everything else.) **M-677** (effect→budget
runtime) is L1-collision and runs inside the `run`/`srf` serial track, not as its own parallel wave.

## Completed (archived)
- **`dfr`** — **RP-10/RP-9 research gate DISCHARGED + RFC-0022/0023 → Accepted, LANDED** on `main`
  (#344, 2026-06-21): four fractured Opus sub-reasoners per RFC verified the Honest-Uncertainty
  Registers against primary specs (RFC 9110/9112 · RFC 8259 · WHATWG-URL; ADK v2.3.0) + landed
  substrate — design-sound, no falsification (`research/12 §8` · `research/13 §6`). Both RFCs **Draft →
  Accepted** (maintainer ratification; **Enacted** still gated on the builds); M-670/M-671 bodies carry
  the cleared gate + the `dfb` build constraints. Unblocks **`dfb`** (now gated on the L1 surface only).
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
