#!/usr/bin/env bash
# Test matrix for scripts/hooks/branch_guard_parse.py — the per-invocation structural parser
# behind the branch-guard PreToolUse hook (mitigation #12, variant 3: 2026-07-01 rewrite).
#
# Exercises the parser directly (bypassing the bash hook's payload/cwd plumbing, which is
# unchanged from the prior fix and not what this rewrite touches) against:
#   - the must-ALLOW cases that were false-blocked before this rewrite (compound commands where
#     a protected-branch word appears only as a merge SOURCE, in a commit message, or attached to
#     a force flag on an unrelated git subcommand);
#   - the must-BLOCK cases that must remain blocked (real protected-branch pushes/commits/merges
#     and real force-pushes, including --force-with-lease / --force-if-includes / a leading '+'
#     refspec);
#   - the fail-safe (default-deny) cases: dynamic content ($()/`` `` /<()/>()), unterminated
#     quoting, unparseable push flags, and a git -C/--git-dir/--work-tree redirect combined with
#     an operation whose verdict depends on the (now-ambiguous) current branch.
#
# Run: scripts/hooks/test-branch-guard-parse.sh
# Exit 0 iff every row matches its expected verdict; a mismatch prints the row and the script
# exits 1 (never-silent — CI/agents should treat a non-zero exit as a genuine regression).

set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PARSE="$HERE/branch_guard_parse.py"
PROTECTED="main integration dev claude/head/*"

pass=0
fail=0

# check <expect ALLOW|BLOCK|UNSAFE> <branch> <description> <command>
# BLOCK and UNSAFE both exit non-zero from the hook's point of view (both mean "the Bash tool
# call is blocked") but are reported distinctly here since they are different rows in the brief:
# BLOCK = a real protected-branch/force-push violation; UNSAFE = the fail-safe default-deny path.
check() {
  local expect="$1" branch="$2" desc="$3" cmd="$4"
  local out rc verdict
  out="$(python3 "$PARSE" "$cmd" "$branch" "$PROTECTED" 2>&1)"
  rc=$?
  case "$rc" in
    0) verdict="ALLOW" ;;
    1) verdict="BLOCK" ;;
    2) verdict="UNSAFE" ;;
    *) verdict="ERROR($rc)" ;;
  esac
  if [[ "$verdict" == "$expect" ]]; then
    printf 'PASS  %-6s  %-70s  branch=%s\n' "$verdict" "$desc" "$branch"
    pass=$((pass + 1))
  else
    printf 'FAIL  got=%-6s want=%-6s  %-60s  branch=%s\n  cmd: %s\n  out: %s\n' \
      "$verdict" "$expect" "$desc" "$branch" "$cmd" "$out"
    fail=$((fail + 1))
  fi
}

echo "=== must ALLOW: previously false-blocked (mitigation #12 variant 3) ==="
check ALLOW claude/leaf/x "merge dev-as-source && push to leaf dest" \
  'git merge --ff-only origin/dev && git push --no-verify origin claude/aot-1-0-0-orchestration-scp1kf'
check ALLOW claude/leaf/foo "push leaf && worktree remove -f -f (force flag on unrelated subcommand)" \
  'git push --no-verify origin claude/leaf/foo && git worktree remove -f -f /some/path'
check ALLOW claude/leaf/foo "commit message contains the word 'integration'" \
  'git commit -m "…integration…" && git push origin claude/leaf/foo'
check ALLOW claude/leaf/x "worktree remove -f then unrelated push" \
  'git worktree remove -f <p> && git push origin claude/leaf/x'
check ALLOW claude/leaf/x "commit message contains main/dev/integration words, HEAD is a leaf" \
  'git commit -m "touches main dev integration words"'
check ALLOW claude/leaf/x "merge main-as-source, push leaf dest" \
  'git merge --ff-only origin/main && git push origin claude/leaf/x'

echo ""
echo "=== must ALLOW: force-shaped flags on NON-push subcommands / non-git commands ==="
check ALLOW claude/leaf/x "git branch -D (delete flag, not a push)" \
  'git branch -D old && git push origin claude/leaf/x'
check ALLOW claude/leaf/x "git reset --hard (not a push)" \
  'git reset --hard HEAD~1 && git push origin claude/leaf/x'
check ALLOW claude/leaf/x "rm -f + commit message containing the word 'force'" \
  "rm -f foo && git commit -m 'force cleanup' && git push origin claude/leaf/x"
check ALLOW claude/leaf/x "git checkout -f (not a push)" \
  'git checkout -f . && git push origin claude/leaf/x'

echo ""
echo "=== must ALLOW: harmless/read-only git subcommands and non-git commands ==="
check ALLOW claude/leaf/x "git status" 'git status'
check ALLOW claude/leaf/x "git fetch" 'git fetch origin'
check ALLOW claude/leaf/x "git worktree list" 'git worktree list'
check ALLOW claude/leaf/x "git log" 'git log --oneline -5'
check ALLOW claude/leaf/x "non-git command" 'ls -la && echo hi'
check ALLOW claude/leaf/x "bare 'git' with no subcommand" 'git'
check ALLOW claude/leaf/x "bare git push on a non-protected branch" 'git push'
check ALLOW claude/leaf/x "git push origin with no refspec, non-protected branch" 'git push origin'
check ALLOW claude/leaf/x "git -C <dir> push with explicit refspec to a leaf branch" \
  'git -C /some/dir push origin claude/leaf/x'

echo ""
echo "=== must BLOCK: real protected-branch pushes ==="
check BLOCK claude/leaf/x "push origin dev" 'git push origin dev'
check BLOCK claude/leaf/x "push origin main" 'git push origin main'
check BLOCK claude/leaf/x "push origin integration" 'git push origin integration'
check BLOCK claude/leaf/x "push origin claude/head/foo" 'git push origin claude/head/foo'
check BLOCK claude/leaf/x "push origin HEAD:main" 'git push origin HEAD:main'
check BLOCK claude/leaf/x "push origin mylocal:dev" 'git push origin mylocal:dev'
check BLOCK dev "bare git push while HEAD is dev" 'git push'
check BLOCK dev "git push origin (no refspec) while HEAD is dev" 'git push origin'

echo ""
echo "=== must BLOCK: real force-pushes (force flag on the push itself) ==="
check BLOCK claude/leaf/x "push --force" 'git push --force origin claude/leaf/x'
check BLOCK claude/leaf/x "push -f" 'git push -f origin claude/leaf/x'
check BLOCK claude/leaf/x "push origin +refs/heads/x (leading + refspec)" 'git push origin +refs/heads/x'
check BLOCK claude/leaf/x "push --force-with-lease" 'git push --force-with-lease origin claude/leaf/x'
check BLOCK claude/leaf/x "push origin +claude/leaf/x (leading + refspec)" 'git push origin +claude/leaf/x'
check BLOCK claude/leaf/x "push --force-if-includes" 'git push --force-if-includes origin claude/leaf/x'
check BLOCK claude/leaf/x "worktree remove -f (harmless) THEN push --force (real)" \
  'git worktree remove -f <p> && git push --force origin claude/leaf/x'

echo ""
echo "=== must BLOCK: mutating subcommand while HEAD is protected ==="
check BLOCK dev "git commit while HEAD=dev" 'git commit -m "wip"'
check BLOCK main "git merge while HEAD=main" 'git merge x'
check BLOCK dev "git rebase while HEAD=dev" 'git rebase'
check BLOCK integration "git cherry-pick while HEAD=integration" 'git cherry-pick abc123'
check BLOCK 'claude/head/foo' "git revert while HEAD=claude/head/foo" 'git revert abc123'

echo ""
echo "=== must BLOCK: compound command, ANY segment violates ==="
check BLOCK claude/leaf/x "first segment ok, second segment pushes protected" \
  'git push origin claude/leaf/x && git push origin main'
check BLOCK dev "first segment read-only, second segment is a bare push while HEAD=dev" \
  'git status && git push'
check BLOCK dev "first segment pushes a fine leaf dest, second segment commits while HEAD=dev" \
  'git push origin claude/leaf/x && git commit -m "wip"'

echo ""
echo "=== must BLOCK (fail-safe / UNSAFE — cannot confidently parse) ==="
# shellcheck disable=SC2016  # single-quoted on purpose: this is literal test-data text fed to
# branch_guard_parse.py as an argv string, not a command we want the shell here to expand.
check UNSAFE claude/leaf/x "command substitution \$(...) in push target" \
  'git push origin $(git branch --show-current)'
# shellcheck disable=SC2016  # same as above: literal backtick test data, not meant to expand here.
check UNSAFE claude/leaf/x "backtick command substitution" \
  'git push origin `echo main`'
check UNSAFE claude/leaf/x "unterminated quote" \
  "git commit -m 'unterminated && git push origin claude/leaf/x"
check UNSAFE claude/leaf/x "process substitution <(...)" \
  'git push origin <(echo main)'
check UNSAFE claude/leaf/x "push flag -o takes a separate value we don't parse" \
  'git push -o ci.skip origin claude/leaf/x'
check UNSAFE claude/leaf/x "git -C <dir> bare push (branch depends on redirected repo)" \
  'git -C /some/other/dir push'
check UNSAFE claude/leaf/x "git --git-dir=/x --work-tree=/y commit (redirected repo)" \
  'git --git-dir=/x --work-tree=/y commit -m hi'

echo ""
echo "=== additional must-ALLOW: -C push with explicit non-protected refspec is still fine ==="
check ALLOW claude/leaf/x "git -C <dir> push explicit refspec, not bare (no cwd dependency)" \
  'git -C /some/dir push origin claude/leaf/x'

echo ""
echo "=== must ALLOW: command-substitution OUTSIDE a push's target (regression fix) ==="
# The substitution fail-safe must be scoped to a git push's own args (or the git subcommand
# position); a substitution in a read-only git, an echo, or an assignment can never change a
# protected/force verdict, so it must be allowed rather than blocked (the prior whole-command
# reject was a regression that blocked ubiquitous command-substitution everywhere).
# shellcheck disable=SC2016  # single-quoted on purpose: literal argv test data, not for local expansion.
check ALLOW claude/leaf/x "read-only git diff piped to wc, inside an echo substitution" \
  'echo "count: $(git diff --name-only origin/dev...origin/claude/leaf/x | wc -l)"'
# shellcheck disable=SC2016
check ALLOW claude/leaf/x "VAR=\$(git rev-parse HEAD) assignment then commit on a leaf branch" \
  'VAR=$(git rev-parse HEAD) && git commit -m x'
# shellcheck disable=SC2016
check ALLOW claude/leaf/x "git status; echo with a date substitution" \
  'git status; echo "$(date)"'
# shellcheck disable=SC2016
check ALLOW claude/leaf/x "for-loop over a git worktree list substitution, no mutating op" \
  'for wt in $(git worktree list); do echo $wt; done'
# shellcheck disable=SC2016
check ALLOW claude/leaf/x "substitution in an echo, then a push to a STATIC leaf dest" \
  'echo "$(git log -1)" && git push origin claude/leaf/x'

echo ""
echo "=== must BLOCK (UNSAFE): substitution that DETERMINES a push's destination ==="
# shellcheck disable=SC2016  # literal argv test data: the substitution is the fail-safe trigger.
check UNSAFE claude/leaf/x "push dest produced by \$(echo dev) substitution" \
  'git push origin $(echo dev)'
# shellcheck disable=SC2016
check UNSAFE claude/leaf/x "push dest produced by \$(cat target.txt) substitution" \
  'git push origin $(cat target.txt)'
# shellcheck disable=SC2016
check UNSAFE claude/leaf/x "git subcommand itself produced by a substitution" \
  'git $(echo push) origin main'

echo ""
echo "----------------------------------------------------------------------"
echo "TOTAL: pass=$pass fail=$fail"
if [[ $fail -ne 0 ]]; then
  echo "FAIL: one or more branch-guard parser rows did not match the expected verdict." >&2
  exit 1
fi
echo "OK: all rows matched the expected verdict."
exit 0
