#!/usr/bin/env bash
# worktree-guard — assert the worktree-isolation discipline (CLAUDE.md mitigation #11).
#
# One isolated git worktree per concurrent agent; the orchestrator's main worktree stays a clean
# pointer. This check is idempotent (pure reads of git state) and parameterized by mode. It is the
# worktree analogue of branch-guard: never-silent (a violation exits non-zero with an actionable
# message; success prints one `ok` line unless --quiet).
#
# Modes:
#   --leaf          assert the CWD is an *isolated* (linked) git worktree, not the shared main tree.
#                   A concurrent/leaf agent runs this before any git work to confirm it cannot collide
#                   with siblings in the parent's checkout.
#   --orchestrator  (default) assert the *main* worktree is a clean pointer (no uncommitted changes),
#                   so a stray agent's edits are never carried across a branch switch.
#   --quiet         suppress the ok line.
set -euo pipefail

mode=orchestrator
quiet=0
for a in "$@"; do
  case "$a" in
    --leaf) mode=leaf ;;
    --orchestrator) mode=orchestrator ;;
    --quiet) quiet=1 ;;
    -h|--help) sed -n '2,20p' "$0"; exit 0 ;;
    *) echo "worktree-guard: unknown arg '$a' (use --leaf | --orchestrator | --quiet)" >&2; exit 2 ;;
  esac
done

git_dir="$(git rev-parse --git-dir 2>/dev/null)" || { echo "worktree-guard: not inside a git repo" >&2; exit 2; }
common_dir="$(git rev-parse --git-common-dir 2>/dev/null)"
top="$(git rev-parse --show-toplevel 2>/dev/null || echo '?')"
# A linked (isolated) worktree has a per-worktree git dir distinct from the shared common dir.
is_linked=0
[ "$git_dir" != "$common_dir" ] && is_linked=1

case "$mode" in
  leaf)
    if [ "$is_linked" -ne 1 ]; then
      echo "worktree-guard: BLOCKED — this agent is in the SHARED main worktree ($top), not an isolated one." >&2
      echo "  Concurrent agents racing one checkout cross-contaminate branches (CLAUDE.md mitigation #11)." >&2
      echo "  Fix: spawn with isolation:\"worktree\", or run 'git worktree add <dir> <branch>' and work there." >&2
      exit 1
    fi
    [ "$quiet" -eq 1 ] || echo "worktree-guard: ok — isolated (linked) worktree at $top."
    ;;
  orchestrator)
    if [ "$is_linked" -eq 1 ]; then
      echo "worktree-guard: note — CWD is a linked worktree, not the main one (orchestrator mode expects the main tree)." >&2
    fi
    if [ -n "$(git status --porcelain)" ]; then
      echo "worktree-guard: BLOCKED — the main worktree ($top) has uncommitted changes; it must stay a clean pointer." >&2
      echo "  A stray agent likely edited the shared tree. Preserve its work to ITS OWN branch (commit + push)" >&2
      echo "  before switching the tree off it; spawn concurrent agents isolated (CLAUDE.md mitigation #11)." >&2
      exit 1
    fi
    n="$(git worktree list | wc -l | tr -d ' ')"
    [ "$quiet" -eq 1 ] || echo "worktree-guard: ok — main worktree clean on '$(git rev-parse --abbrev-ref HEAD)' (${n} worktree(s) total)."
    ;;
esac
