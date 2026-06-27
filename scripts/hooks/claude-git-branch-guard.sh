#!/usr/bin/env bash
# Claude Code PreToolUse(Bash) hook — the HARNESS-level branch guard (rsm 2026-06-27). It blocks a
# git commit / merge / cherry-pick / rebase on a protected branch, a push whose target is a protected
# branch, and any force-push, BEFORE the Bash tool runs — so an agent (or an orphaned sub-agent) can
# never land on `main`/`integration`/`dev`/`claude/head/*` directly. This is the layer that actually
# stops agents (they run git through Bash); scripts/checks/branch-guard.sh guards the git/CLI layer.
#
# Contract (Claude Code hooks): reads the PreToolUse JSON payload on stdin; exit 0 = allow, exit 2 =
# BLOCK (stderr is shown to the model). IDEMPOTENT / read-only decision. Fails OPEN (exit 0) on any
# parse/IO problem so a malformed payload never wedges the session — the git-level hook is the backstop.
set -uo pipefail

cd "${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null || echo .)}" 2>/dev/null || exit 0

payload="$(cat || true)"
cmd="$(printf '%s' "$payload" | jq -r '.tool_input.command // empty' 2>/dev/null || true)"
[[ -z "$cmd" ]] && exit 0  # not a readable Bash command — allow; other guards still apply

# Inspect the command STRUCTURE, not text inside quotes: strip single/double-quoted argument content
# (commit messages, echo strings, here-strings) BEFORE scanning, so a commit message that mentions
# "git push ... main" cannot false-positive. The git-level hook is the backstop if this ever misses.
scan="$(printf '%s' "$cmd" | sed -E "s/'[^']*'/''/g; s/\"[^\"]*\"/\"\"/g")"

# Fast path: only inspect commands that actually invoke git (after de-quoting).
printf '%s' "$scan" | grep -qE '\bgit\b' || exit 0

current="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo HEAD)"
is_protected() { [[ "$1" =~ ^(main|integration|dev)$ || "$1" == claude/head/* ]]; }
block() { echo "branch-guard (PreToolUse): BLOCKED — $*" >&2; exit 2; }

# 1) Force-push is prohibited outright (CLAUDE.md — reconcile by merging, never overwrite history).
if printf '%s' "$scan" | grep -qE '\bgit[[:space:]]+push\b' \
   && printf '%s' "$scan" | grep -qE -- '(--force(-with-lease)?|[[:space:]]-f([[:space:]]|$)|[[:space:]]\+[^[:space:]]+:)'; then
  block "force-push is prohibited (CLAUDE.md). Bring history together with a merge; never overwrite published history."
fi

# 2) commit / merge / cherry-pick / rebase while ON a protected branch.
if printf '%s' "$scan" | grep -qE '\bgit[[:space:]]+(commit|merge|cherry-pick|rebase)\b'; then
  if is_protected "$current"; then
    op="$(printf '%s' "$scan" | grep -oE 'commit|merge|cherry-pick|rebase' | head -1)"
    block "refusing 'git $op' on protected branch '$current' — it is PR-only (CLAUDE.md). Switch to your working branch; land via a GitHub PR."
  fi
fi

# 3) push whose explicit target ref is a protected branch (e.g. 'git push origin main',
#    'git push origin work:main'); and a bare 'git push' while ON a protected branch.
if printf '%s' "$scan" | grep -qE '\bgit[[:space:]]+push\b'; then
  if is_protected "$current"; then
    block "refusing to push protected branch '$current' — protected branches are PR-only (CLAUDE.md)."
  fi
  targets="$(printf '%s' "$scan" | sed -E 's/.*\bgit[[:space:]]+push\b//' | tr '[:space:]' '\n' | grep -vE '^-|^$' || true)"
  while read -r tok; do
    [[ -z "$tok" ]] && continue
    dst="${tok##*:}"          # 'src:dst' -> 'dst'; a bare token -> itself
    dst="${dst#refs/heads/}"
    if is_protected "$dst"; then
      block "refusing to push to protected branch '$dst' — protected branches are PR-only (CLAUDE.md, never a direct push)."
    fi
  done <<<"$targets"
fi

exit 0
