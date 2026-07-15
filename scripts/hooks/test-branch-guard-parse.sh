#!/usr/bin/env bash
# Test matrix for scripts/hooks/branch_guard_parse.py — the per-invocation structural parser
# behind the branch-guard PreToolUse hook (mitigation #12: variant 3, 2026-07-01; variant 4,
# 2026-07-13 — effective-target-worktree resolution).
#
# Exercises the parser directly (bypassing the bash hook's payload/cwd plumbing except where a
# test row explicitly passes a run-dir to exercise that plumbing) against:
#   - the must-ALLOW cases that were false-blocked before the variant-3 rewrite (compound commands
#     where a protected-branch word appears only as a merge SOURCE, in a commit message, or
#     attached to a force flag on an unrelated git subcommand);
#   - the must-BLOCK cases that must remain blocked (real protected-branch pushes/commits/merges
#     and real force-pushes, including --force-with-lease / --force-if-includes / a leading '+'
#     refspec);
#   - the fail-safe (default-deny) cases: dynamic content ($()/`` `` /<()/>()), unterminated
#     quoting, unparseable push flags, and a git --git-dir/--work-tree redirect combined with an
#     operation whose verdict depends on the (now-ambiguous) current branch;
#   - variant-4 cases: a leading `cd <path>` and/or a `git -C <path>` that changes the EFFECTIVE
#     target worktree away from the payload/harness `run-dir` — both the now-fixed false-positive
#     (a cd into a non-protected leaf worktree must ALLOW even when the initial branch/run-dir is
#     protected) and the still-must-BLOCK/fail-closed companions (cd onto a protected branch,
#     unresolvable cd/-C targets).
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

# check <expect ALLOW|BLOCK|UNSAFE> <branch> <description> <command> [run-dir]
# `run-dir` is optional (5th arg) — omitted, the parser behaves exactly as variant 3 (no cd/-C
# resolution possible without a known base path); passed, it seeds the effective-cwd state so a
# `cd`/`-C` in `cmd` can be resolved against a real fixture worktree (see the variant-4 section
# below, which builds two tiny fixture repos to exercise this for real rather than mocking git).
# BLOCK and UNSAFE both exit non-zero from the hook's point of view (both mean "the Bash tool
# call is blocked") but are reported distinctly here since they are different rows in the brief:
# BLOCK = a real protected-branch/force-push violation; UNSAFE = the fail-safe default-deny path.
check() {
  local expect="$1" branch="$2" desc="$3" cmd="$4" run_dir="${5:-}"
  local out rc verdict
  out="$(python3 "$PARSE" "$cmd" "$branch" "$PROTECTED" "$run_dir" 2>&1)"
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
echo "=== variant 4: resolve the EFFECTIVE target worktree (cd / -C), fail-closed on ambiguity ==="
# These exercise the REAL fix (branch_guard_parse.py's Cwd tracking) against real fixture git
# worktrees checked out on known branches — not mocked — so the resolution path (a real
# `git -C <path> rev-parse --abbrev-ref HEAD` subprocess call) is genuinely proven, not assumed.
FIXTURE_ROOT="$(mktemp -d)"
trap 'rm -rf "$FIXTURE_ROOT"' EXIT

make_fixture_repo() {
  # make_fixture_repo <dir> <branch>: a minimal git repo checked out on <branch>. Explicitly
  # disables commit signing (-c commit.gpgsign=false) since some environments set it globally
  # without a usable gpg binary, which would otherwise fail this fixture's own setup commit.
  local dir="$1" branch="$2"
  git init -q -b "$branch" "$dir"
  git -C "$dir" -c user.email=test@example.com -c user.name=test -c commit.gpgsign=false \
    commit -q --allow-empty -m init
}

LEAF_WT="$FIXTURE_ROOT/leaf-worktree"    # simulates an isolated leaf worktree — NOT protected
DEV_WT="$FIXTURE_ROOT/dev-worktree"      # simulates a worktree still sitting on the protected branch
make_fixture_repo "$LEAF_WT" claude/leaf/cdtest
make_fixture_repo "$DEV_WT" dev

echo ""
echo "--- must ALLOW: the false-positive this variant fixes ---"
check ALLOW dev "cd into an isolated leaf worktree, then commit — payload cwd/branch is stale 'dev'" \
  "cd $LEAF_WT && git commit -m wip" "$DEV_WT"
check ALLOW dev "cd into a leaf worktree, then a bare push with no refspec (resolves to leaf branch)" \
  "cd $LEAF_WT && git push" "$DEV_WT"
check ALLOW dev "git -C <leaf-worktree> commit — payload cwd/branch is stale 'dev'" \
  "git -C $LEAF_WT commit -m wip" "$DEV_WT"
check ALLOW dev "a -C to the leaf dir on a no-op status segment; nothing to judge either way" \
  "git -C $LEAF_WT status && git status" "$DEV_WT"

echo ""
echo "--- must still BLOCK: the effective target genuinely IS protected ---"
check BLOCK claude/leaf/cdtest "cd onto a worktree still sitting on 'dev' — the real target IS protected" \
  "cd $DEV_WT && git commit -m wip" "$LEAF_WT"
check BLOCK claude/leaf/cdtest "git -C <dev-worktree> commit — the real target IS protected (scoped -C)" \
  "git -C $DEV_WT commit -m wip" "$LEAF_WT"

echo ""
echo "--- fail-closed: unresolvable cd/-C targets never silently ALLOW ---"
check UNSAFE claude/leaf/cdtest "cd to a nonexistent path, then commit — unresolvable, must fail CLOSED not open" \
  "cd $FIXTURE_ROOT/does-not-exist && git commit -m wip" "$LEAF_WT"
check UNSAFE claude/leaf/cdtest "git -C <nonexistent> commit — unresolvable -C target fails CLOSED" \
  "git -C $FIXTURE_ROOT/does-not-exist commit -m wip" "$LEAF_WT"
check UNSAFE claude/leaf/cdtest "cd - (previous dir, untracked) then commit — ambiguous, fails CLOSED" \
  "cd - && git commit -m wip" "$LEAF_WT"
# shellcheck disable=SC2016  # single-quoted on purpose: literal argv test data, not for local expansion.
check UNSAFE claude/leaf/cdtest "cd to a dynamic \$(...) target (quoted as one token) then commit — fails CLOSED, never guesses" \
  'cd "$(echo /tmp)" && git commit -m wip' "$LEAF_WT"

echo ""
echo "--- persistence semantics: cd persists across segments; -C never leaks into later segments ---"
check ALLOW dev "cd into leaf, unrelated status, THEN commit — cd's effect persists to later segments" \
  "cd $LEAF_WT && git status && git commit -m wip" "$DEV_WT"
check ALLOW dev "cd into leaf; a -C to the dev worktree for an unrelated status must NOT leak; commit still resolves to leaf" \
  "cd $LEAF_WT && git -C $DEV_WT status && git commit -m wip" "$DEV_WT"
check BLOCK dev "a -C to the leaf worktree on an earlier segment must NOT leak forward: a later bare push (no -C, no prior cd) is still judged by the ORIGINAL 'dev' cwd" \
  "git -C $LEAF_WT status && git push" "$DEV_WT"

echo ""
echo "--- explicit-refspec push destinations are unaffected by cd/-C resolution (unchanged) ---"
check BLOCK dev "cd into a leaf worktree, then push to an EXPLICIT protected destination — still BLOCKed" \
  "cd $LEAF_WT && git push origin dev" "$DEV_WT"
check ALLOW dev "cd into a leaf worktree, explicit non-protected destination — ALLOW regardless" \
  "cd $LEAF_WT && git push origin claude/leaf/cdtest" "$DEV_WT"

echo ""
echo "=== backward compatibility: variant-3 behavior is unchanged when run-dir is omitted ==="
check UNSAFE claude/leaf/x "git -C <dir> bare push, no run-dir supplied — behaves as before (UNSAFE)" \
  'git -C /some/other/dir push'
check ALLOW claude/leaf/x "git -C <dir> push explicit refspec, no run-dir supplied — behaves as before (ALLOW)" \
  'git -C /some/dir push origin claude/leaf/x'

echo ""
echo "=== variant 5 (adversarial hardening): CRITICAL 1 — newline-separated compound commands ==="
# Exact payload class from the security review: a plain multi-line bash block, exactly how an
# agent's multi-line Bash tool call is often written. Before this fix, shlex silently consumed the
# unquoted newline as whitespace, flattening the whole block into ONE segment judged only by its
# FIRST line's leading word.
check BLOCK dev $'multi-line commit+push, HEAD=dev — the exact review payload (add is inert, commit BLOCKs before push is even reached)' \
  $'git add -A\ngit commit -m wip\ngit push origin main'
check BLOCK dev "echo then commit on a newline, HEAD=dev (review's second literal example)" \
  $'echo hi\ngit commit -m wip'
check BLOCK dev "read-only line then an explicit protected push on the next line" \
  $'git status\ngit push origin main'
check ALLOW claude/leaf/x "multi-line block on a non-protected leaf branch stays ALLOW (no regression)" \
  $'git add -A\ngit commit -m wip\ngit push origin claude/leaf/x'

echo ""
echo "--- newline INSIDE a quoted commit message must NOT be mis-split (quote-awareness proof) ---"
check ALLOW claude/leaf/x "a literal newline inside a quoted -m message, leaf branch — single commit, unaffected" \
  $'git commit -m "line1\nline2"'
check BLOCK dev "a literal newline inside a quoted -m message, HEAD=dev — still recognized as ONE commit, still BLOCKed" \
  $'git commit -m "line1\nline2"'

echo ""
echo "=== variant 5 (adversarial hardening): CRITICAL 2 — command-word indirection ==="
echo "--- must BLOCK: real git reached via indirection, on a protected branch ---"
check BLOCK dev "absolute path to the git binary (basename match)" \
  '/usr/bin/git commit -m wip'
check BLOCK dev "env FOO=bar git commit — env's assignment shifts git off seg[1]" \
  'env FOO=bar git commit -m wip'
check BLOCK dev "bare FOO=bar git commit — no 'env' needed at all" \
  'FOO=bar git commit -m wip'

echo ""
echo "--- must fail CLOSED (UNSAFE): git hidden behind an opaque/unparsed wrapper ---"
check UNSAFE dev "eval 'git commit ...' — the guard cannot parse eval's argument string" \
  "eval 'git commit -m wip'"
check UNSAFE dev "sh -c 'git commit ...' — an inline shell script string" \
  "sh -c 'git commit -m wip'"
check UNSAFE dev "bash -c \"git push origin main\" — an inline shell script string" \
  'bash -c "git push origin main"'
check UNSAFE dev "xargs git add -A — xargs's own argument IS the real command" \
  'xargs git add -A'
check UNSAFE claude/leaf/x "eval/sh -c/xargs are UNSAFE regardless of branch (fail-closed, not branch-gated) — still UNSAFE even on a leaf branch" \
  "eval 'git commit -m wip'"

echo ""
echo "--- must fail CLOSED (UNSAFE): a variable in the COMMAND-WORD position itself ---"
# shellcheck disable=SC2016  # single-quoted on purpose: literal argv test data, not for local expansion.
check UNSAFE dev 'GIT=git; $GIT commit -- a bare $VAR as the effective command word' \
  'GIT=git; $GIT commit -m wip'
# shellcheck disable=SC2016
check UNSAFE dev '${GIT} commit -- braced variable form, same indirection' \
  '${GIT} commit -m wip'

echo ""
echo "--- additional hardening beyond the reviewer's literal examples: unmodeled wrapper flags ---"
check UNSAFE dev "env -i FOO=bar git commit — env's OWN flag (-i) is not specially parsed; fails closed rather than silently inert" \
  'env -i FOO=bar git commit -m wip'

echo ""
echo "--- DRY proof: cd-detection shares the same prefix consumption as git-detection ---"
check ALLOW dev "env FOO=bar cd <leaf-worktree> && commit — prefixed cd is tracked exactly like a prefixed git invocation" \
  "env FOO=bar cd $LEAF_WT && git commit -m wip" "$DEV_WT"

echo ""
echo "--- must remain ALLOW: ordinary, unambiguous, non-git commands are untouched ---"
check ALLOW dev "ls -la — an ordinary non-git command, HEAD=dev" 'ls -la'
check ALLOW dev "python3 script.py — an ordinary external program, HEAD=dev" 'python3 script.py'
check ALLOW dev "sudo ls -la — a modeled wrapper around an ordinary command" 'sudo ls -la'

echo ""
echo "--- documented, deliberately out-of-scope boundary: a script FILE (no -c) is not opened ---"
# sh/bash *without* -c runs an external script FILE, not an inline string this guard could parse —
# structurally identical to any other external program potentially shelling out to git (make,
# npm run test, python foo.py, ...), which this guard does not attempt to cover (unbounded). Only
# the inline eval/-c/xargs indirection explicitly named by the review is closed.
check ALLOW dev "bash script.sh (no -c) — documented scope boundary, NOT covered by this hardening" \
  'bash deploy.sh'

echo ""
echo "=== variant 5 (adversarial hardening): INFORMATIONAL 3 — detached HEAD fails closed ==="
DETACHED_WT="$FIXTURE_ROOT/detached-worktree"
git init -q -b main "$DETACHED_WT"
git -C "$DETACHED_WT" -c user.email=t@e.com -c user.name=t -c commit.gpgsign=false commit -q --allow-empty -m c1
FIRST_SHA="$(git -C "$DETACHED_WT" rev-parse HEAD)"
git -C "$DETACHED_WT" -c user.email=t@e.com -c user.name=t -c commit.gpgsign=false commit -q --allow-empty -m c2
git -C "$DETACHED_WT" checkout -q "$FIRST_SHA"  # now genuinely detached

check UNSAFE dev "cd into a genuinely DETACHED worktree, then commit — 'HEAD' is ambiguous, fails CLOSED" \
  "cd $DETACHED_WT && git commit -m wip" "$LEAF_WT"
check UNSAFE dev "git -C <detached-worktree> commit — same ambiguity via scoped -C" \
  "git -C $DETACHED_WT commit -m wip" "$LEAF_WT"
check UNSAFE HEAD "current branch is already the literal 'HEAD' sentinel (payload cwd itself detached/unresolved), then commit" \
  'git commit -m wip'

echo ""
echo "----------------------------------------------------------------------"
echo "TOTAL: pass=$pass fail=$fail"
if [[ $fail -ne 0 ]]; then
  echo "FAIL: one or more branch-guard parser rows did not match the expected verdict." >&2
  exit 1
fi
echo "OK: all rows matched the expected verdict."
exit 0
