#!/usr/bin/env bash
# Branch-protection guard (rsm 2026-06-27) — keep work on the actual working branch; never commit or
# push to a protected branch directly. Standalone (no lib.sh dependency, so it runs identically as a
# git hook, a Claude Code hook, a `just` recipe, and a CLI). IDEMPOTENT (a pure read of git state —
# safe to run any number of times). PARAMETERIZED (protected set + expected working branch). NEVER-
# SILENT (G2): any violation is an explicit non-zero exit with a clear, actionable message.
#
# Modes:
#   branch-guard.sh                 commit-mode (default): HEAD must NOT be a protected branch
#   branch-guard.sh --expect B      ...and HEAD must equal B (the session's working branch)
#   branch-guard.sh --push          pre-push mode: reject pushes whose REMOTE ref is protected (stdin)
#   branch-guard.sh --quiet         suppress the success line (violations always print)
#
# Parameters (env, all overridable):
#   MYC_PROTECTED_BRANCHES   space-separated names/globs (default: "main integration dev claude/head/*")
#   CLAUDE_WORKING_BRANCH    expected working branch (same as --expect; an explicit --expect wins)
#
# Exit: 0 = ok; 1 = protected-branch / wrong-branch violation; 64 = bad usage.
set -euo pipefail

PROTECTED_DEFAULT="main integration dev claude/head/*"
read -r -a PROTECTED <<<"${MYC_PROTECTED_BRANCHES:-$PROTECTED_DEFAULT}"

expect="${CLAUDE_WORKING_BRANCH:-}"
mode="commit"
quiet=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --expect) expect="${2:-}"; shift 2 ;;
    --push)   mode="push"; shift ;;
    --quiet)  quiet=1; shift ;;
    -h|--help) sed -n '2,20p' "$0"; exit 0 ;;
    *) echo "branch-guard: unknown argument: $1" >&2; exit 64 ;;
  esac
done

is_protected() { # $1 = branch name; matches any protected name/glob
  local b="$1" pat
  for pat in "${PROTECTED[@]}"; do
    # shellcheck disable=SC2053  # glob match against the protected pattern is intended
    [[ "$b" == $pat ]] && return 0
  done
  return 1
}

if [[ "$mode" == "push" ]]; then
  # git pre-push stdin lines: <local_ref> <local_sha> <remote_ref> <remote_sha>
  blocked=0
  while read -r _lref _lsha rref _rsha; do
    [[ -z "${rref:-}" ]] && continue
    rb="${rref#refs/heads/}"
    if is_protected "$rb"; then
      echo "branch-guard: BLOCKED — refusing to push to protected branch '$rb' — protected branches are PR-only (CLAUDE.md, never a direct push)." >&2
      blocked=1
    fi
  done
  [[ "$blocked" == 1 ]] && exit 1
  [[ "$quiet" == 0 ]] && echo "branch-guard: ok — no protected push target."
  exit 0
fi

# commit-mode
current="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo HEAD)"
if [[ "$current" == "HEAD" ]]; then
  echo "branch-guard: BLOCKED — detached HEAD; checkout your working branch before committing." >&2
  exit 1
fi
if is_protected "$current"; then
  echo "branch-guard: BLOCKED — on protected branch '$current'. Work on a feature/working branch and land via PR (CLAUDE.md: main/integration/dev/claude/head/* are PR-only, never a direct commit/merge)." >&2
  exit 1
fi
if [[ -n "$expect" && "$current" != "$expect" ]]; then
  echo "branch-guard: BLOCKED — on branch '$current' but this session's working branch is '$expect'. Checkout '$expect' (commits must stay on the designated working branch)." >&2
  exit 1
fi
[[ "$quiet" == 0 ]] && echo "branch-guard: ok — on '$current' (not protected${expect:+; matches expected $expect})."
exit 0
