---
name: pr-land
description: >-
  Review a PR with a dedicated agent and land it UP the tree (leaf→dev, dev→integration). Spawns an
  isolated Sonnet /pr-review agent that posts findings as PR comments, patches what it finds, replies
  with the resolution, and updates the PR description; then — once review-clean and green — merges the
  PR into its base tier. The terminal merge to `main` is held for the maintainer (use /land for that
  squash). The agent-driven per-PR review loop, parameterized by PR number + base tier.
when_to_use: >-
  Use to land a working/leaf or kickoff PR onto `dev` (or `dev` onto `integration`) the reviewed,
  collision-safe way — for each PR you bring up in a concurrent wave. NOT for the final squash onto
  `main` (that terminal, maintainer-gated step is /land or /wave-land).
allowed-tools: Bash(git fetch:*), Bash(git diff:*), Bash(scripts/checks/branch-guard.sh:*), Bash(scripts/checks/worktree-guard.sh:*)
---

# pr-land

The merge gate below `main` is the agent's, not a human's (CLAUDE.md §Autonomous PR workflow). This
skill operationalizes the **per-PR agent-review loop**: every PR worked up the tree gets a dedicated
reviewer, is driven clean through PR comment threads, and is merged into its base tier — with the
**merge to `main` the one terminal checkpoint held for the maintainer**.

## Parameters

- `PR` — the pull-request number.
- `BASE` — the base tier it lands on: `dev` (from a leaf/working/kickoff branch) or `integration`
  (from `dev`). For `main`, **stop and use `/land`** (the curated squash + maintainer checkpoint).
- `MODEL` — the reviewer model (default **Sonnet**, per the hybrid swarm assignment).

## The loop

1. **Spawn an isolated `/pr-review` reviewer** (`isolation:"worktree"` — mitigation #11; **never** the
   shared main tree). Brief it to apply `.claude/skills/pr-review/SKILL.md` + the shared rubric to the
   PR diff and audit the house rules: the transparency rule (per-op tags never upgraded without a
   checked basis), append-only decisions, grounding, never-silent G2, and a hallucination/consistency
   pass. Honest and non-sycophantic (house rule #4) — only approve on merit; do not manufacture
   findings; flag anything architecturally significant rather than guessing.
2. **Post findings as PR comments** (frugal, severity-ranked). If nothing material, one short approving
   comment beats nitpicks.
3. **Patch what's found** (the same reviewer if it has context, else a fresh isolated agent): fix on
   the PR branch, re-run the **change-scoped** gate for what changed (not the full `just check` — that
   tightens at integration), **reply to each comment with the resolution applied**, and **update the
   PR description** to match the net change. Commit + push as **separate** commands (mitigation #12 —
   no compound `commit && push`; keep `main`/`integration`/`dev` out of commit-message text); never
   push a protected branch (branch-guard is armed).
4. **Green, then merge up the tree.** When the review is resolved and CI/gates are green, **pull the
   base down first** (`update_pull_request_branch` / merge `BASE` in — mitigation #6, so the diff is
   only this branch's net change and the merge is conflict-free), then merge the PR into `BASE` with
   the **`merge` method (`--no-ff`, lineage preserved)** — squash is only for the `main` landing.
5. **Shared-file ordering.** When several PRs touch the same shared file (e.g. `issues.yaml`), land
   them **sequentially**, pulling the freshly-merged base down before the next (mitigation #6).

## Boundaries (never-silent)

- **Stops at `main`.** This skill merges only onto `dev`/`integration`. The `integration → main` step
  is a curated **squash** PR, gated by the maintainer / `/pr-review` — that is **/land** (or
  **/wave-land** for a head), never this skill.
- Leaves **do not** finalize `CHANGELOG`/`Doc-Index`/`api-index` or close out issues — they record
  only their own issue and FLAG the rest up; that reconciliation is the integration tier's
  (CLAUDE.md §Concurrent-PR development).
- Reviewer + patcher run **isolated** (`/worktree-guard --leaf`); the orchestrator's main tree stays a
  clean pointer (`/worktree-guard`).

Composes with **/wave** (which calls this per work-item), **/pr-review** (the rubric it applies),
**/worktree-guard** and **/branch-guard** (the safeguards). Full rationale: CLAUDE.md §Concurrent-PR
development + §Autonomous PR workflow.
