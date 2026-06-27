---
name: branch-guard
description: >-
  Assert you are on the designated working branch and NOT on a protected branch
  (main/integration/dev/claude/head/*) before committing, merging, or pushing. The
  idempotent, parameterized guard that keeps work on its actual working branch and
  off the protected tiers — so the wrong-branch / commit-to-main failure mode cannot
  recur. Backs the harness-level PreToolUse hook + the git pre-commit/pre-push hooks.
when_to_use: >-
  Run at session start (to confirm you're on the right working branch) and before any
  commit / octopus-merge / push step in /dev-workflow, /land, /wave-land, or a swarm.
  Also invoke when a git operation was unexpectedly blocked, to see exactly why.
allowed-tools: Bash(scripts/checks/branch-guard.sh:*), Bash(just branch-guard:*), Bash(git rev-parse:*), Bash(git status:*), Bash(git branch:*)
---

# branch-guard

The repo's branch discipline (CLAUDE.md) is now **enforced**, not just documented: `main`,
`integration`, `dev`, and every `claude/head/*` base are **PR-only** — never a direct commit, merge,
or push. This skill is the agent-facing entry to that guard. It is **idempotent** (a pure read of git
state) and **parameterized** (the protected set and the expected working branch are inputs).

## The three enforcement layers (defense-in-depth)

1. **Harness layer — `.claude/settings.json` `PreToolUse(Bash)` hook** → `scripts/hooks/claude-git-branch-guard.sh`.
   Blocks an agent's `git commit`/`merge`/`cherry-pick`/`rebase` on a protected branch, any push to a
   protected target, and any force-push, **before** the Bash tool runs. This is the layer that stops
   agents (and orphaned sub-agents) — they run git through Bash.
2. **Git layer — pre-commit + pre-push hooks** (`.pre-commit-config.yaml`, `repo: local`) → call
   `scripts/checks/branch-guard.sh`. Catches direct git use in normal dev (`just hooks` installs them).
3. **Workflow layer — this skill + `just branch-guard`** → an explicit checked step the workflows run.

## Use it

```sh
just branch-guard                      # assert HEAD is not a protected branch
scripts/checks/branch-guard.sh --expect "$CLAUDE_WORKING_BRANCH"   # ...and == the session branch
scripts/checks/branch-guard.sh --push  # pre-push mode (reads refs on stdin)
```

- **Parameters:** `MYC_PROTECTED_BRANCHES` (space-separated names/globs; default `main integration dev
  claude/head/*`) and `CLAUDE_WORKING_BRANCH` / `--expect <branch>` (the designated working branch).
- **Never-silent (G2):** a violation is an explicit non-zero exit with an actionable message; success
  prints one `ok` line (suppress with `--quiet`).

## Where it fits

- **Session start:** run `scripts/checks/branch-guard.sh --expect <your working branch>` to confirm
  you are where you should be before doing any work.
- **Before every commit / octopus-merge / push** in `/dev-workflow`, `/land`, `/wave-land`, and swarm
  integration — the workflows call this so the discipline holds by construction, not by memory.
- **Landing on `main`/`integration`/`dev` is via GitHub PR**, not local git — so blocking local
  commit/merge/push to those branches is correct and does not impede the PR-based landing flow.

If a git step is blocked, the message names the protected/wrong branch and the fix (switch to the
working branch; land via PR). Reconcile divergence by **merging**, never by force-push (CLAUDE.md).
