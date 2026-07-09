---
name: sync-down
description: >-
  Propagate a squashed landing on `main` back down to the persistent lower trunks -- main ->
  integration -> dev -- via a plain, no-force merge per tier (DN-97 Rank 1, maintainer-ratified:
  same-content trunks make this trivially conflict-free -- no merge-driver, no veto, no keep-list).
  Landed through the same PR-merge mechanism /pr-land and /land already use for these protected
  branches, never a raw git push. Run once per batch after 1-6 disjoint work-set squashes land on
  main. Also covers the in-flight-adapt one-liner for a still-open work-set branch.
when_to_use: >-
  Use immediately after any PR squash-merges onto `main` -- a single landing, or the tail of a
  batch of up to 6 disjoint work-set squashes plus a close-out squash. Also point an in-flight
  /forward work-set at this skill's "adapt" step whenever it needs to pull a freshly-synced `dev`
  down into its own branch mid-flight.
allowed-tools: Bash(git fetch:*), Bash(git switch:*), Bash(git merge:*), Bash(git push:*), Bash(gh pr:*), Bash(scripts/checks/worktree-guard.sh:*), Bash(scripts/checks/branch-guard.sh:*)
---

# sync-down

The downward half of the ratified pattern
(`docs/notes/DN-97-Unified-Branch-Merge-Propagation-Pattern.md` §§2.2/3.3/3.4/8, Rank 1). All
three trunks — `dev` (working), `integration` (staging), `main` (release) — carry **identical
tracked content**; they differ by **rigor**, not content. That is what makes this propagation a
**plain `git merge --no-ff`, every time**: same-content trunks means there is nothing to delete or
reconcile, so the down-merge cannot lose data and needs no driver, veto, or keep-list (§4.1). This
skill is the single-invocation driver for that step — idempotent, conflict-loud, **never a force**.

## Why this is trivial, not just simple (DN-97 §2.2)

A landed work-set was **both** merged into `dev` (staging, `--no-ff`) **and** squashed into `main`
(landing) — so `main`'s squash `S_i` carries content the lower tiers **already hold**. The
three-way down-merge sees "both sides made the same change" and auto-resolves clean. The lower
tiers' **graph** still diverges from `main`'s linear squash history (DN-95's accepted funk,
decision #5) — that is cosmetic and never produces a *content* conflict under Rank 1.

## The propagation, tier by tier

`dev`/`integration` are **protected + PR-gated** (`CLAUDE.md`; the harness-level branch-guard,
mitigation #10, blocks any Bash-level `git commit`/`merge`/`push` whose destination is
`main`/`integration`/`dev`/`claude/head/*`). So — unlike DN-97 §3.3's literal raw-`git push`
sketch — this skill lands each hop through **the same PR-merge mechanism `/pr-land` and `/land`
already use to write to these exact branches**: open a PR, then merge it via the GitHub API
(`gh pr merge`), never a direct push. Non-squash `--no-ff` (lineage-preserving — this propagates
already-landed content, it is not a new curated landing).

```
# 1) main -> integration
git fetch origin main integration
git switch -c sync/main-to-integration origin/integration
git merge --no-ff origin/main -m "chore(sync-down): main -> integration"
git push -u origin sync/main-to-integration
gh pr create --base integration --head sync/main-to-integration \
  --title "chore(sync-down): main -> integration" \
  --body "Plain --no-ff propagation of the freshly-squashed main (DN-97 Rank 1 / /sync-down). Same-content trunks: trivially conflict-free, introduces no new code."
gh pr merge <PR#> --merge            # --no-ff method; NEVER --squash (this is propagation, not landing)
git push origin --delete sync/main-to-integration   # ephemeral, auto-pruned (decision #2)

# 2) integration -> dev (same shape)
git fetch origin integration dev
git switch -c sync/integration-to-dev origin/dev
git merge --no-ff origin/integration -m "chore(sync-down): integration -> dev"
git push -u origin sync/integration-to-dev
gh pr create --base dev --head sync/integration-to-dev \
  --title "chore(sync-down): integration -> dev" \
  --body "Plain --no-ff propagation (DN-97 Rank 1 / /sync-down)."
gh pr merge <PR#> --merge
git push origin --delete sync/integration-to-dev
```

Run in an **isolated worktree** (`/worktree-guard --leaf`) — this is still concurrent git work.

## Skip gracefully when there's nothing to sync

If `git merge --no-ff origin/main` (or `origin/integration`) produces an empty diff, the lower
tier already has everything — report "already in sync" and stop. Never open a no-op PR.

## Never-silent guard (G2/VR-5)

Under Rank 1 this merge should **always** auto-resolve clean (§2.2 above). If either merge ever
reports a real conflict: **stop loudly.** Do not force-resolve (`-X ours`/`-X theirs`), do not
force-push, do not skip the tier. A conflict here is not supposed to be structurally possible under
the ratified pattern — it signals genuine content divergence between trunks, which is exactly the
Rank-2 sharp edge Rank 1 was chosen to avoid (DN-97 §9). Flag it for the maintainer as an anomaly
rather than papering over it.

## In-flight adapt (DN-97 §3.4) — not a protected-branch operation, no PR needed

A still-open `/forward` work-set (`ws/<other>`, ephemeral, never in the protected set) pulls the
freshly-synced `dev` down into itself directly, in its own isolated worktree:

```
git switch ws/<other>
git fetch origin dev
git merge --no-ff origin/dev       # plain, no force; disjoint work-sets => conflict-free
git push origin ws/<other>         # resume working up
```

This is a normal working-branch push — no branch-guard involvement, no PR.

## FLAG — relationship to `scripts/sync-heads.sh` (interpretive judgment call)

DN-97 §8 says `/sync-down` "extends `scripts/sync-heads.sh` almost unchanged." That existing
script propagates `origin/main` into each persistent `claude/head/*` base (the older Wave-N
multi-session pattern) via a **raw `git push`** straight to the protected head branch. This skill
deliberately does **not** reuse that raw-push form for `main`→`integration`→`dev`, because those
three are in the exact same protected set the branch-guard enforces (`main integration dev
claude/head/*`), and `CLAUDE.md` states plainly that "landing onto a protected branch is via
GitHub PR, never local git." The PR-merge mechanism above is not a new invention — it is the
mechanism `/pr-land`/`/land` already use for these branches, applied here.

**This leaves an open inconsistency this leaf did not resolve** (out of scope — `scripts/
sync-heads.sh` is not one of this task's owned files): the script's existing raw-push behavior for
`claude/head/*` has the identical protected-branch tension this skill routes around via a PR. The
maintainer/integrator should decide whether `sync-heads.sh` gets the same PR-merge treatment, or
whether it has a standing exception (e.g. it is understood to run only outside the agent harness,
by a human, where the `PreToolUse` hook is not in the loop) that `/sync-down` should not inherit.

## Composition

Pairs with **`/forward`** (which hands off here after a work-set's squash lands on `main`; DN-97
§1 step 4), **`/worktree-guard`** (isolation), **`/branch-guard`** (confirms no accidental direct
write to a protected branch mid-reconciliation). Full rationale: `docs/notes/DN-97-Unified-Branch-
Merge-Propagation-Pattern.md` §§2.2/3.3/3.4/8, `CLAUDE.md` §Autonomous PR workflow item 6,
mitigation #10.
