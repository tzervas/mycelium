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
#    from the payload's `cwd` HEAD by default, but re-resolved against a leading `cd <path>` and/or
#    that invocation's own `-C <path>` if present (variant 4, see below) — NEVER from words inside a
#    `-m`/`-F` message or heredoc.
# 5. Everything else (`status`, `fetch`, `worktree …`, `branch`, `log`, non-git commands, …): ignored.
#
# --- Fail-safe boundary (non-negotiable: zero false-negatives) ------------------------------------
# Reducing false-positives must never allow a real violation through. The guard BLOCKS (closed, exit
# 2) — never allows — whenever a git-touching command's structure cannot be confidently resolved:
#   - A command/process substitution ($( , a backtick, <( , >( ) appears where it can obscure a
#     git-MUTATING operation's target or force flags — SCOPED to: (a) a `git push`'s own argument
#     tokens (e.g. `git push origin $(echo dev)` — the destination is unresolvable), or (b) the git
#     SUBCOMMAND position itself (e.g. `git $(echo push) …` — we cannot tell if it is a push). A
#     substitution ANYWHERE ELSE — a read-only `git diff`/`git log`, an `echo "$(…)"`, a `VAR=$(…)`
#     assignment, a non-git command, or a mutating subcommand's *args* (commit/merge/cherry-pick/
#     rebase/revert are judged by the cwd HEAD, not by their args, so a substitution there cannot
#     flip the verdict) — CANNOT change a protected-branch/force verdict and is ALLOWED. (A prior
#     revision rejected substitution at the whole-command level; that over-blocked ubiquitous
#     read-only command-substitution and was a regression — this scoping is the fix.)
#   - The tokenizer reports an unterminated quote or escape (malformed/obfuscated quoting).
#   - A `git push` carries a flag that takes a separate value we do not special-case (-o / --repo /
#     --receive-pack / --push-option) — we cannot safely tell whether the NEXT token is that flag's
#     value or a positional remote/refspec, so we do not guess.
#   - A git invocation carries `--git-dir=` / `--work-tree=` / `--namespace=` (redirects git at a repo
#     this guard does not attempt to reconstruct) AND the operation's verdict would depend on the
#     CURRENT branch (a bare `git push` with no refspec, or a mutating subcommand's HEAD check) — this
#     combination is unresolvable and BLOCKS. (`-C <dir>` is now RESOLVED, not fail-closed
#     unconditionally — see the variant-4 note below.)
# Anything else that is not one of the recognized git subcommands, or not git at all, is inert and
# ALLOWED — the fail-safe is scoped to exactly the operations this guard exists to gate.
#
# --- mitigation #12, variant 4 (2026-07-13): resolve the EFFECTIVE target worktree ---------------
# Variant 3 above judged commit/merge/cherry-pick/rebase/revert and a bare push PURELY from the
# payload's `.cwd` HEAD, resolved once (line ~104 below) before any command-string analysis. That
# was itself already a fix for the worktree-aware "second variant" (an agent's ACTUAL git target is
# the worktree it runs in, not `CLAUDE_PROJECT_DIR`'s main checkout) — but it was still wrong for a
# command that ITSELF changes which worktree it operates in: `cd <path> && git commit …` (an agent
# that `git worktree add`-ed its own isolation, or lost its worktree binding across a context
# compaction, so the payload `.cwd` stayed pinned at the shared main checkout on a protected branch)
# was FALSE-BLOCKED even though the commit's real target was a fine, non-protected leaf branch.
#
# The fix (in branch_guard_parse.py — the bash layer here is unchanged except passing `run_dir` as a
# 4th argument): the parser now tracks an EFFECTIVE cwd/branch as it walks segments left to right. A
# plain `cd <path>` segment (a real shell cd) updates that state for every later segment in the same
# command; a git invocation's own `-C <path>` resolves ONLY that invocation (matching git's own `-C`
# semantics, which never changes the shell's cwd) without touching the persistent state a later `cd`
# builds on. Either way the actual branch at the resolved path is looked up via
# `git -C <path> rev-parse --abbrev-ref HEAD` — a real check, not a guess — and if that path can't be
# resolved (missing, not a worktree, a dynamic `$(...)` target, `cd -`, or the git call itself fails)
# the branch is marked UNRESOLVED, which still fails safe (UNSAFE/BLOCK) for any segment whose
# verdict actually needs to know the current branch. `--git-dir=`/`--work-tree=`/`--namespace=` are
# NOT resolved (too ambiguous — see the fail-safe boundary above) and remain unconditionally
# fail-closed exactly as in variant 3. Net effect: the recurring false-positive (a legit isolated
# worktree commit blocked because the payload cwd disagreed with the real target) is eliminated,
# while every real protected-branch/force-push violation — including one reached via `cd`/`-C` —
# still BLOCKs, and any unresolvable target still fails CLOSED. See branch_guard_parse.py's own
# module docstring for the full design.
#
# scripts/checks/branch-guard.sh is the backstop layer (git pre-commit/pre-push hooks + the
# `/branch-guard` skill); it judges the CURRENT branch directly via `git rev-parse` at hook-invocation
# time (no command-string parsing at all), so it was already immune to this class of false-positive.
#
# --- mitigation #12, variant 5 (2026-07-13, adversarial hardening): newline segmentation + -------
# --- indirection closure; ALSO widens the fast-path pre-filter below -----------------------------
# An adversarial security review of variant 4 found two PRE-EXISTING Critical false-negatives (not
# caused by variant 4, present before it): (1) `split_top_level` never actually split on newlines —
# `shlex` silently consumed an unquoted newline as whitespace, so a plain multi-line bash block
# (`git add -A\ngit commit -m wip\ngit push origin main`) flattened into ONE segment, judged only by
# its first line; (2) literal-`git`-only matching missed an absolute path to the binary, a leading
# `NAME=value` shell-assignment prefix shifting git off the first token, `eval`/`sh -c`/`bash -c`/
# `xargs` hiding git in an opaque argument string, and a bare `$VAR`/`${VAR}` in the command-word
# position. Both are fixed in `branch_guard_parse.py` (see its own module docstring for the full
# design + the fail-safe boundary each fix respects); this bash layer required exactly ONE change,
# below: the cheap fast-path pre-filter (previously `\bgit\b` only) is WIDENED to also trigger
# python3 on `$`/backtick (any variable/substitution — including a fully-obfuscated command word
# that never contains the literal substring "git" at all, e.g. a base64-decoded variable) and on
# eval/xargs/sh/bash/zsh/dash/ksh by name — otherwise a sufficiently obfuscated command could skip
# the python parser ENTIRELY at this pre-filter, before ever reaching its (correct) analysis. This
# necessarily makes the pre-filter trigger far more often (a bare `$HOME` now triggers it) — that is
# an accepted, deliberate cost: the pre-filter's own comment already says it exists only to skip the
# python3 spawn for "obviously-unrelated" commands, and a command containing an unresolved variable
# is no longer "obviously unrelated" once bare variables must be scrutinized (variant 5, CRITICAL 2).
# Also see INFORMATIONAL 3 in branch_guard_parse.py: a detached HEAD (or an outright branch-
# resolution failure — both surface as the literal string "HEAD") now fails closed for any op whose
# verdict needs the current branch, rather than silently matching no protected pattern and ALLOWing.

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
# Canonicalize to an absolute path — this is also passed to the python parser (variant 4) as the
# seed for its `cd`/`-C` effective-cwd resolution, which requires an unambiguous absolute base.
run_dir="$(pwd)"

# Fast path: only inspect commands that could plausibly involve git, directly or through
# indirection (cheap pre-filter; the real parse below is authoritative — this is purely to skip the
# python3 spawn for obviously-unrelated commands like `ls -la`). Widened in variant 5 beyond a bare
# `\bgit\b` word match: also triggers on `$`/backtick (ANY variable or substitution — including a
# command word fully obfuscated to never contain the literal substring "git", e.g. a base64-decoded
# variable) and on the named indirection wrappers (`eval`/`xargs`/`sh`/`bash`/`zsh`/`dash`/`ksh`) —
# so a sufficiently obfuscated command cannot skip the python parser at this pre-filter before ever
# reaching its (correct) analysis. This trades some of the pre-filter's cheapness for correctness —
# a command containing an unresolved variable is no longer "obviously unrelated" to git once bare
# variables must be scrutinized (branch_guard_parse.py, CRITICAL 2).
printf '%s' "$cmd" | grep -qE '\b(git|eval|xargs|sh|bash|zsh|dash|ksh)\b|\$|`' || exit 0

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

verdict="$(python3 "$hook_dir/branch_guard_parse.py" "$cmd" "$current" "$protected" "$run_dir" 2>&1)"
rc=$?

if [[ $rc -ne 0 ]]; then
  echo "branch-guard (PreToolUse): BLOCKED — ${verdict#*: }" >&2
  exit 2
fi

exit 0
