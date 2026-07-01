#!/usr/bin/env bash
# Claude Code PreToolUse(Bash) hook — the HARNESS-level branch guard (rsm 2026-06-27; per-invocation
# parse rewrite rsm 2026-07-01, mitigation #12 variant 3). It blocks a git commit / merge /
# cherry-pick / rebase / revert on a protected branch, a push whose DESTINATION is a protected
# branch, and any force-push, BEFORE the Bash tool runs — so an agent (or an orphaned sub-agent) can
# never land on `main`/`integration`/`dev`/`claude/head/*` directly. This is the layer that actually
# stops agents (they run git through Bash); scripts/checks/branch-guard.sh guards the git/CLI layer.
#
# Contract (Claude Code hooks): reads the PreToolUse JSON payload on stdin; exit 0 = allow, exit 2 =
# BLOCK (stderr is shown to the model). IDEMPOTENT / read-only decision. Fails OPEN (exit 0) on a
# payload/IO problem so a malformed payload never wedges the session — the git-level hook is the
# backstop. Fails CLOSED (exit 2, BLOCK) whenever the command's git structure cannot be confidently
# parsed — see "Fail-safe boundary" below. These are different failure axes: payload/IO errors happen
# before we know whether the command touches git at all (open); once we know it invokes git, any
# ambiguity in what that invocation *does* is resolved by blocking (closed).
#
# --- Why this file was rewritten (mitigation #12, variant 3) -----------------------------------
# The prior version scanned the WHOLE de-quoted command string with regex. That produced false
# blocks on compound commands: a `git merge origin/dev && git push … claude/leaf/x` was blocked
# because the word "dev" appeared anywhere in the string, even though "dev" was only a merge SOURCE
# and the push's actual DESTINATION was a fine working branch; a force flag on `git worktree remove
# -f -f` blocked an unrelated push in the same compound command; a commit message containing the
# word "integration" blocked the commit even on a leaf branch. The fix: parse PER GIT INVOCATION,
# not the whole string — split into segments, classify each segment's git subcommand, and for `push`
# resolve the actual destination branch(es) and force-flag from THAT segment's own arguments only.
#
# --- Parse design -------------------------------------------------------------------------------
# 1. Segment split: split the command on top-level (unquoted, unparenthesized) `&&`, `||`, `;`,
#    bare `&`, `|`, and newlines. Tokenization (so an operator INSIDE a quoted string, e.g. a commit
#    message containing "&&", is never treated as a real operator) uses Python's `shlex` — a
#    battle-tested POSIX-shell-like tokenizer — rather than hand-rolled bash/regex character-walking,
#    which is exactly the kind of fragile parsing that caused the original false-positives and is
#    too easy to get subtly wrong in a security-critical guard.
# 2. Each segment is classified by its git subcommand (skipping recognized wrapper tokens `sudo` /
#    `command` / `time` / `env`, and git's own global flags before the subcommand).
# 3. `git push`: walk its arguments; force flags (`--force`, `-f`, `--force-with-lease`,
#    `--force-if-includes`, or a refspec with a leading `+`) are recognized ONLY as that segment's
#    OWN push flags/refspecs — never from other subcommands (`git worktree remove -f`, `git branch
#    -D`, `git reset --hard`, `git checkout -f`, `rm -f`, …) and never from message/quoted content.
#    The destination of each refspec is resolved per git's own grammar (`branch` -> `branch`;
#    `src:dst` -> `dst`; `HEAD:dst` -> `dst`; a bare `git push`/`git push <remote>` with no refspec
#    pushes the CURRENT branch, judged from cwd HEAD). BLOCK iff any destination is protected, or a
#    force flag is present anywhere in that push's own flags/refspecs.
# 4. `commit` / `merge` / `cherry-pick` / `rebase` / `revert`: judged by the CURRENT branch, resolved
#    from the payload's `cwd` HEAD (unchanged from the prior worktree-aware fix — see mitigation #12
#    "second variant" note in CLAUDE.md) — NEVER from words inside a `-m`/`-F` message or heredoc.
# 5. Everything else (`status`, `fetch`, `worktree …`, `branch`, `log`, non-git commands, …): ignored.
#
# --- Fail-safe boundary (non-negotiable: zero false-negatives) ------------------------------------
# Reducing false-positives must never allow a real violation through. The guard BLOCKS (closed, exit
# 2) — never allows — whenever a git-touching command's structure cannot be confidently resolved:
#   - The raw command contains a dynamic-content marker ($( , a backtick, <( , >( ) — command/process
#     substitution means the actual destination/flags are not statically knowable.
#   - The tokenizer reports an unterminated quote or escape (malformed/obfuscated quoting).
#   - A `git push` carries a flag that takes a separate value we do not special-case (-o / --repo /
#     --receive-pack / --push-option) — we cannot safely tell whether the NEXT token is that flag's
#     value or a positional remote/refspec, so we do not guess.
#   - A git invocation carries `-C <dir>` / `--git-dir=` / `--work-tree=` (redirects git at a
#     DIFFERENT repo/worktree than the payload cwd) AND the operation's verdict would depend on the
#     CURRENT branch (a bare `git push` with no refspec, or a mutating subcommand's HEAD check) — the
#     guard judges branch state from the payload cwd only and does not shell out to inspect an
#     arbitrary redirected `-C` path, so this combination is unresolvable and BLOCKS.
# Anything else that is not one of the recognized git subcommands, or not git at all, is inert and
# ALLOWED — the fail-safe is scoped to exactly the operations this guard exists to gate.
#
# scripts/checks/branch-guard.sh is the backstop layer (git pre-commit/pre-push hooks + the
# `/branch-guard` skill); it judges the CURRENT branch directly via `git rev-parse` at hook-invocation
# time (no command-string parsing at all), so it was already immune to this class of false-positive.

set -uo pipefail

# Resolve the parser script's own directory as an ABSOLUTE path before any `cd` below — the hook
# changes its working directory to the command's cwd (possibly a different worktree entirely), so
# a relative path to the sibling branch_guard_parse.py would break once that `cd` happens.
hook_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

payload="$(cat || true)"
cmd="$(printf '%s' "$payload" | jq -r '.tool_input.command // empty' 2>/dev/null || true)"
[[ -z "$cmd" ]] && exit 0  # not a readable Bash command — allow; other guards still apply

# Resolve the branch from the directory the command actually RUNS in (the payload's `cwd`), NOT the
# main checkout. An isolated worktree agent on its own leaf branch must be judged by THAT worktree's
# HEAD — using CLAUDE_PROJECT_DIR (which points at the main checkout) false-positived worktree agents
# whenever the main checkout sat on a protected branch (CLAUDE.md mitigation #11; the worktree variant
# of #12). Fail-safe to CLAUDE_PROJECT_DIR / the repo top when the cwd is absent. This resolution is
# preserved unchanged from the prior fix — only the command-string analysis below was rewritten.
run_dir="$(printf '%s' "$payload" | jq -r '.cwd // empty' 2>/dev/null || true)"
[[ -z "$run_dir" || ! -d "$run_dir" ]] && run_dir="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null || echo .)}"
cd "$run_dir" 2>/dev/null || exit 0

# Fast path: only inspect commands that actually mention 'git' as a word (cheap pre-filter; the real
# parse below is authoritative — this is purely to skip the python3 spawn for obviously-unrelated
# commands like `ls -la`).
printf '%s' "$cmd" | grep -qE '\bgit\b' || exit 0

current="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo HEAD)"
protected="${MYC_PROTECTED_BRANCHES:-main integration dev claude/head/*}"

command -v python3 >/dev/null 2>&1 || {
  # No python3 available to run the structural parser — this guard cannot confidently analyze the
  # command. Fail-safe CLOSED only if the command looks like it could mutate/push; otherwise allow
  # (keeps read-only agent commands working even in a python3-less environment).
  if printf '%s' "$cmd" | grep -qE '\bgit[[:space:]]+(push|commit|merge|cherry-pick|rebase|revert)\b'; then
    echo "branch-guard (PreToolUse): BLOCKED — python3 is unavailable so the structural git-command parser cannot run; refusing to allow an unparsed push/commit/merge/cherry-pick/rebase/revert. Install python3 or run the operation manually after review." >&2
    exit 2
  fi
  exit 0
}

verdict="$(python3 "$hook_dir/branch_guard_parse.py" "$cmd" "$current" "$protected" 2>&1)"
rc=$?

if [[ $rc -ne 0 ]]; then
  echo "branch-guard (PreToolUse): BLOCKED — ${verdict#*: }" >&2
  exit 2
fi

exit 0
