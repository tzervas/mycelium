---
name: wave-land
description: Land work the Wave-N way, respecting protected bases. A child/working branch lands onto its head via a --no-ff PR (lineage preserved, NO squash); a completed head lands onto main via a curated SQUASH PR; then the squashed main is propagated back down (scripts/sync-heads.sh). Use when merging a completed unit into a head, or a head into main, in the multi-session head-branch workflow. Not for plain child-to-child merges below a head (those stay octopus/--no-ff with no PR) — use /land only for a single-branch squash onto main outside a wave.
---

# /wave-land

Identify the boundary you're crossing, then apply the rule.

## Child → head (a persistent, protected base)
Open a PR from the completed child onto the head. Self-review with the `/pr-review` lens
(honesty/per-op tags, append-only, grounding, never-silent G2). Ensure the head's `just check` is
green. Merge **`--no-ff`** (a merge commit — **never squash**; heads preserve lineage).
*Child branches below a head do NOT need this* — they octopus/`--no-ff` into each other freely, no PR.

## Head → `main`
1. Pull the latest squashed `main` down into the head first (`scripts/sync-heads.sh <head>`, or
   `git fetch origin main` → merge), resolve, re-run `just check` green.
2. Run the full `/land` discipline: self-review, handle **every** CI/bot review comment, green check.
3. **Squash-PR into `main`** with a curated subject + body (the net change — never the WIP/`wip(batch
   …)`/fixup/merge trail). `main` is the **only** branch that squashes.

## After any landing on `main`
Run `scripts/sync-heads.sh` to **propagate the squashed `main` down** into every other head (pull-down
flows down — mitigation #6). Conflicts are **flagged for the owning session to resolve on its head**,
never force-resolved here.

## Final wave integration
When the heads are each complete + green: octopus-merge them onto one integration branch, **reconcile
the shared files once** (`issues.yaml` dedup, `CHANGELOG`, `Doc-Index`, workspace `Cargo.toml`), run
the full `just check`, then squash-PR to `main` and sync-heads down.

**Invariants:** `main` + `claude/head/*` are PR-gated & protected; only `main` squashes; honesty +
append-only hold (VR-5/G2) at every boundary. The protected bases are **enforced**, not just
convention: the branch-guard (`/branch-guard`; PreToolUse + pre-commit/pre-push hooks) blocks any
local commit/merge/push to `main`/`integration`/`dev`/`claude/head/*` — every landing onto them goes
through a GitHub PR (squash for `main`, `--no-ff` for a head).
