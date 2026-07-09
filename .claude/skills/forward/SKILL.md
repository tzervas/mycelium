---
name: forward
description: >-
  Develop a change forward (up the tiers) the spec-first, staged, context-windowed way: spec/DN
  (requirements, DoD, user stories) -> public API -> private API -> component seam-map -> code,
  each stage a bounded working set that persists a compact artifact and drops from context once
  consumed (DN-96). Owns the staged design pipeline, the change-sizing check (soft ~1-2k / hard
  4,000 LOC), and the precursor doc/index-branch mechanic; delegates isolation, review+merge-up,
  index regen, and the main landing to /worktree-guard, /pr-land, /doc-index + /tero-refresh, and
  /land, following DN-97's ratified Rank-1 batch-engine command sequences for the actual landing.
when_to_use: >-
  Use to develop a single feature/fix/change forward from spec to landed code, especially when the
  change's shape (its API + component split) isn't yet known -- deriving the disjoint ownership
  seams up front avoids the mid-flight merge rework of discovering them after the code exists.
  Resume mid-pipeline with STAGE=<stage> after a compaction or handoff instead of replaying earlier
  stages. Not for a multi-item concurrent wave -- use /wave, which calls this skill's precursor
  step at its integration close-out.
allowed-tools: Bash(git fetch:*), Bash(git switch:*), Bash(git merge:*), Bash(git push:*), Bash(gh pr:*), Bash(just:*), Bash(scripts/checks/worktree-guard.sh:*), Bash(scripts/checks/branch-guard.sh:*)
---

# forward

The upward half of the maintainer-ratified branch/merge/propagation pattern
(`docs/notes/DN-96-Forward-Development-And-Change-Sizing-Pattern.md`,
`docs/notes/DN-97-Unified-Branch-Merge-Propagation-Pattern.md` §§1-3/§8, Rank 1). **The point is
context-thrift:** the agent invokes `/forward CHANGE=… TARGET=dev`; it does not hold the whole
procedure or all intermediate state in one window. Each stage below is a bounded working set that
consumes the prior stage's compact, *persisted* artifact and drops the rest once it hands the next
stage what it needs (mitigations #6/#8).

## Parameters

- `CHANGE` — the feature/fix/change (an issue id, or a short description).
- `TARGET` — the tier this work-set stages into (default `dev`).
- `STAGE` — optional resume point (`spec` / `public-api` / `private-api` / `components` / `code`)
  so a compaction or handoff re-enters the pipeline at the stage whose artifact is already
  persisted, without replaying the earlier stages.
- `MODE` — the swarm model assignment when the change decomposes into a wave (default Sonnet;
  `CLAUDE.md` §Swarm modes), passed explicitly to any spawned agent.

## The staged pipeline (DN-96 §2.1) — the central organizing principle

Five stages, strict order, each *feeds forward* only what the next needs and persists its
artifact (issue/PR/spec body) **before** dropping from the live window:

1. **SPEC.** Requirements · deliverables · Definition of Done · success criteria · user stories
   (house rule #6, placed at the front of the flow). A DN/RFC/ADR for a change big enough to
   warrant one; the issue body's stories+DoD for a small one. *Hands forward:* the compact spec.
2. **PUBLIC API** (from the spec). The `pub` surface / `std.*` names, each with its honest
   per-op tag (`Exact/Proven/Empirical/Declared` — house rule #1, the tag *is* part of the API).
   *Spec drops out. Hands forward:* the public API + I/O + guarantees.
3. **PRIVATE API** (from the public API). The internal ops that implement it, with the
   `EXPLAIN`-able selection points (house rule #2). *Hands forward:* the private API surface.
4. **COMPONENTS + seam map** (from both APIs' I/O/behaviors). The component split falls out
   easily once the APIs are fixed. **Load-bearing by-product: the touched-file/crate/dir set —
   the ownership seam map.** Two changes whose Stage-4 sets are disjoint branch and merge in
   parallel conflict-free *by construction* (DN-65, derived up front instead of discovered after
   the code exists); an overlap is visible before a line is written and rises to the shared
   parent (`CLAUDE.md` §Core file-ownership rule), never a merge conflict. **Change-sizing lives
   here** — see below. *Hands forward:* the seam map + per-unit interfaces.
5. **CODE** (stapled in against the now-fixed interfaces). `/dev-workflow` discipline: small
   auditable steps, a property test for every bound, never-silent fallibility, honest tags.
   *Holds:* the interfaces, not the design prose that produced them.

**Trivial-change collapse (DN-96 §5 Trade-off 2b).** A two-line fix does not earn five formal
stages. Stage 1 collapses to the issue's stories+DoD (already required by house rule #6); Stages
2–4 collapse to a sentence each. The pipeline earns its keep on changes big enough that holding
spec+API+components+code at once would actually strain the window — below that, it's the ordinary
`/dev-workflow` loop.

**Backtracking is supported, not forbidden.** Because every stage's artifact is *persisted* before
the drop (mitigation #8), a return to an earlier stage **re-loads** the artifact from disk rather
than re-deriving it — `STAGE=public-api` re-enters there directly. Exploratory/spike work may
collapse the stages entirely until the surface stabilizes (`dev` is the messy working tier).

## Change-sizing (Stage 4; DN-97 §2.3, maintainer steer 3)

Bind working→`dev` and `dev`→`integration` to a **soft ≈1–2k-LOC** target (warn) and an
**absolute 4,000-LOC hard cap** on total source churn (additions+removals+inline mods; hard-stop).
**Auto-generated bulk is excluded from both** — it rides the precursor branch (below), never the
source PR. Cohesion still wins: an atomic unit whose minimal honest form exceeds the soft target
stays one work-set (never ship a half-exposed `pub` API to hit a line count — the no-half-API rule
outranks the size target, DN-96 §5 Trade-off 3) — but it should still respect the 4k hard cap; a
genuine exception is a **flagged**, not silent, overrun.

## Precursor doc/index-branch mechanic (Stage 6; DN-96 §2.3, DN-97 §8)

**Only when Stage 2 or Stage 4 marked a public-API / corpus-surface delta.** A private-only change
regenerates nothing and this step is skipped entirely.

| Generated artifact | Regen recipe | Drift gate |
|---|---|---|
| Agent API index | `just docs-index` | `just doc-index` |
| Tero memory index | `just tero-index-gen` | `just tero-index` |
| Lib (`.myc`) index | `just lib-index-gen` | `just lib-index` |

**Ownership + ordering (do not duplicate — a leaf never commits this itself):**

1. This work-set is reviewed + green under change-scoped gates (`/pr-land`).
2. **The integrating parent** (never the leaf) cuts a precursor branch off the current `TARGET`
   tip, runs the marked recipe(s) **against this work-set's reviewed tip**, commits **only** the
   `docs/*-index/` delta, and merges the precursor into `TARGET` *immediately ahead* of this
   work-set — so `TARGET`'s drift gate stays green across the pair and this work-set's own PR diff
   stays source-only (small, reviewable). Calls **`/doc-index`** (api-index leg) and
   **`/tero-refresh`** if a running server must pick up the new rows.
3. This work-set's PR merges next.

A leaf that touches a public API **FLAGs** "public API moved, precursor regen needed" up — it
never commits `docs/*-index/` bulk into its own PR (`CLAUDE.md` §Auto-generated docs: orchestrator-
owned, regenerated by the integrating parent, never hand-merged).

**Concurrency guard (DN-96 §5 Failure mode A).** Two work-sets landing into the same `TARGET` in
the same batch that both move the public API touch the *same* generated trees — a collision the
disjoint-source guarantee does not cover. The precursor is **serialized per `TARGET`**: the
integrator runs **one** regen after all of that batch's public-API-touching changes are reviewed,
never concurrent precursors.

**High-churn escape hatch (DN-96 §5 Failure mode B).** Per-hop regeneration may be too slow on a
very busy `dev`. The documented, non-default relaxation is deferring index regen to the
`integration` tier only (accepting a standing `dev` drift-red between times) — pick this only if
measured regen cost justifies it; the default here is per-hop precursor (every tier green).

## Landing — DN-97's ratified Rank-1 batch engine (delegates to `/worktree-guard`, `/pr-land`, `/land`)

Follow DN-97 §3's **exact** command shapes, not a generic PR flow — the canonical `main`-landing
head is this work-set's **own branch**, not the accumulated `dev`/`integration` diff:

```
# start (§3.1) — off the working tier, never main (mitigation #13); isolated worktree (/worktree-guard --leaf)
git fetch origin
git switch -c ws/<topic> origin/dev
# … Stage 5 code, staple against the fixed interfaces …
git push -u origin ws/<topic>
```

1. **Stage into `dev`** via **`/pr-land TARGET=dev`** — non-squash `--no-ff`; the branch is
   **not consumed**, it lands on `main` later from itself.
2. **Batch promotion `dev`→`integration`** (`--no-ff`, tighter gate, once-per-batch shared-file
   reconciliation) is **integrator-driven at the batch level**, not per `/forward` invocation.
3. **Land into `main`** (§3.2c, once this work-set is part of a ready batch): adapt onto the
   current tip (`git merge --no-ff origin/main`, no force — disjoint ⇒ clean), push, then run
   **`/land`** to squash-PR `ws/<topic>` → `main` directly (curated subject+body — never the WIP
   trail). `main` gains one clean, ≤4k-LOC squash for this disjoint unit; the branch **auto-prunes**
   (`git push origin --delete ws/<topic>`, decision #2).
4. **Hand off down-propagation.** After `main` lands, defer to **`/sync-down`** to refresh
   `integration`/`dev`. The *next* `/forward` invocation branches off that freshly-synced `dev`
   (mitigation #13) — never off a superseded base.

**The mega-PR caveat, refined by DN-97.** DN-96 §2.4 said bite-sizing does not bind
`integration→main` because that hop was one curated mega-squash. Under the ratified batch engine
each disjoint work-set gets its **own** ≤4k-LOC squash directly onto `main` instead — so the
"unbounded size" allowance now applies to the **batch total** (1–6 such squashes plus one
shared-surface close-out squash), not to any single PR (DN-65: total delta unbounded, PR size
bounded — already the standing rule).

## Composition (extend, don't duplicate)

`/forward` **owns** only the staged design pipeline (Stages 1–4) and the precursor mechanic
(Stage 6) — the new material. Everything else **delegates**:

- **`/worktree-guard --leaf`** — isolation for Stage 5+ (mitigation #11).
- **`/pr-land`** — the stage-into-`dev` review+merge loop.
- **`/doc-index`** + **`/tero-refresh`** — the precursor's regen/refresh legs.
- **`/land`** — the curated squash landing onto `main` (step 3 above).
- **`/sync-down`** — the down-propagation handoff (step 4 above).
- **`/wave`** gains one call into this skill's precursor step at its integration close-out,
  replacing the ad-hoc "regenerate `docs/api-index/`" line with the defined mechanic — `/forward`
  serves the **single-change** path; `/wave` serves the **multi-item concurrent** path and reuses
  this skill's Stage-6 machinery rather than re-deriving it.

No change to `/pr-land` / `/land` / `/dev-workflow` / `/doc-index` / `/tero-refresh` themselves —
they are consumed as-is.

## Honest trade-offs carried forward (DN-96 §5 — do not oversell)

The precursor's "regenerate from the reviewed tip, land one commit *ahead*" ordering is the single
most error-prone part done by hand — it is safe **only because it is automated** here; the drift
gate is the never-silent backstop if it's ever inverted (a red gate, not silent corruption). The
staged pipeline is up-front overhead that pays off on changes with a knowable surface, not on pure
research spikes — those legitimately collapse to `/dev-workflow` on `dev`.

Full rationale: `docs/notes/DN-96-Forward-Development-And-Change-Sizing-Pattern.md`,
`docs/notes/DN-97-Unified-Branch-Merge-Propagation-Pattern.md` §§1–3/§8, `CLAUDE.md` §Concurrent-PR
development, §Autonomous PR workflow, mitigations #6/#8/#11/#13.
