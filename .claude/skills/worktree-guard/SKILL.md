---
name: worktree-guard
description: >-
  Assert the worktree-isolation discipline before any concurrent git work: one isolated
  git worktree per concurrent agent, and the orchestrator's main worktree a clean pointer.
  The idempotent, parameterized guard that keeps parallel agents from racing one checkout
  and cross-contaminating branches (CLAUDE.md mitigation #11) — the worktree analogue of
  /branch-guard.
when_to_use: >-
  A concurrent/leaf agent runs `--leaf` before its first git commit/checkout to confirm it is
  isolated. An orchestrator runs `--orchestrator` (default) before/after fanning out, and any time the
  main tree shows unexpected changes, to confirm it stayed a clean pointer. Invoke when a spawn omitted
  isolation:"worktree" or when a stray branch/edit appears in the main worktree.
allowed-tools: Bash(scripts/checks/worktree-guard.sh:*), Bash(git worktree:*), Bash(git rev-parse:*), Bash(git status:*)
---

# worktree-guard

Parallel agents that share one working directory race on `HEAD` and the index — one agent's
`git checkout -b` switches the tree out from under another, and a branch's uncommitted changes get
carried onto a sibling's checkout (cross-contamination). This skill enforces the fix from CLAUDE.md
**mitigation #11**: **one isolated `git worktree` per concurrent agent**; the orchestrator's **main
worktree stays a clean pointer**. It is **idempotent** (a pure read of git state) and **parameterized**
(the mode is the input).

## Use it

```sh
just worktree-guard --leaf       # (alias: just wg) — a concurrent agent: assert I'm in an ISOLATED worktree
just worktree-guard              # orchestrator (default): assert the main tree is a clean pointer
scripts/checks/worktree-guard.sh --quiet   # the script directly; --quiet suppresses the ok line (hook/CI use)
```

The default (`--orchestrator`) resolves the **main** worktree (the first `git worktree list` entry) and
checks *its* status, so it is correct even when invoked from a linked worktree.

- **`--leaf`** — blocks (non-zero) if the CWD is the shared main worktree rather than a linked one.
  Detection: a linked worktree's per-worktree git dir differs from the shared common dir
  (`git rev-parse --git-dir` ≠ `--git-common-dir`).
- **`--orchestrator`** (default) — blocks if the **main** worktree has uncommitted changes (it must
  stay a clean pointer; a stray agent likely edited it). The fix is never to `git checkout` away from a
  dirty shared tree — first **preserve the stray work to its own branch** (commit + push, durability
  #9), then reconcile.
- **Never-silent (G2):** a violation is an explicit non-zero exit with the fix; success prints one
  `ok` line.

## Where it fits

- **Spawning concurrent agents:** always pass `isolation:"worktree"` (the harness creates the
  worktree); a leaf may also `git worktree add` itself. Each leaf runs `--leaf` before its first git
  write so the discipline holds by construction, not by memory.
- **Orchestrator loop:** run the default mode before and after a fan-out, and whenever `git status`
  on the main tree is unexpectedly non-empty — that is the signature of a non-isolated agent landing
  in the shared checkout.
- **Recovery (if a stray agent did land in the main tree):** commit its in-progress changes to **its
  own** branch and push (preserve first), *then* move the main tree off it. Never overwrite or
  `checkout`-away from a dirty shared tree.

Pairs with **/branch-guard** (right branch, off the protected tiers) and the **/wave** + **/pr-land**
skills (which assume every concurrent agent is isolated). Full rationale: CLAUDE.md §Concurrent-PR
development + mitigation #11.
