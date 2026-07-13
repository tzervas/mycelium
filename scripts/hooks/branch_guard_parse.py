#!/usr/bin/env python3
"""Structural parser for the branch-guard PreToolUse hook (claude-git-branch-guard.sh).

Parses a shell command string PER GIT INVOCATION (not as one regex-scanned blob) so that
compound commands are judged by what each git subcommand actually does, not by which words
happen to appear anywhere in the string. See the header comment in
scripts/hooks/claude-git-branch-guard.sh for the full design rationale and the fail-safe
boundary this module implements.

Usage: branch_guard_parse.py <command> <current-branch> <protected-patterns-space-separated> [run-dir]
Exit 0 + prints "ALLOW"           -> caller allows the Bash tool call.
Exit 1 + prints "BLOCK: <reason>" -> caller blocks; a real protected-branch/force-push violation.
Exit 2 + prints "UNSAFE: <reason>"-> caller blocks; the command could not be confidently parsed
                                      (fail-safe / default-deny — never a false ALLOW).

No third-party dependencies beyond the stdlib (shlex, fnmatch, subprocess, os) so it runs in any
environment that has python3, matching the other scripts/checks/*.sh hooks that already shell out
to python3.

--- mitigation #12, variant 4 (2026-07-13): resolve the EFFECTIVE target worktree ------------------
Variant 3 (below) judges every mutating/bare-push segment against a single `current_branch` string
resolved ONCE, from the harness payload's `.cwd` (see claude-git-branch-guard.sh). That is wrong
whenever the command itself changes which worktree it actually operates in — a leading `cd <path> &&`
(an agent that `git worktree add`-ed its own isolation, or lost its worktree binding across a context
compaction, so the payload `.cwd` stays pinned at the shared main checkout) or a `git -C <path>` /
`--git-dir=`/`--work-tree=` argument. Variant 3 already failed CLOSED (UNSAFE) on `-C`/`--git-dir`/
`--work-tree` when the verdict depended on the current branch — safe, but a permanent false-positive
for every legitimate isolated-worktree commit whose payload cwd disagreed with its real target.

Variant 4 tracks an **effective cwd** (`Cwd`: a path + its resolved branch) as segments are walked
left to right:
  - A plain (non-git) `cd <path>` segment is a REAL shell cd — it persists for every later segment
    in the same command, so it updates the effective cwd/branch for the rest of the walk.
  - A git invocation's own `-C <path>` (one or more, applied like a chained cd) is SCOPED to that
    single invocation only — it resolves that invocation's branch without mutating the persistent
    effective cwd a later `cd` segment builds on (this matches git's own `-C` semantics: it does not
    change the shell's working directory).
  - `--git-dir=`/`--work-tree=`/`--namespace=` remain in the ORIGINAL variant-3 fail-closed path
    (mapped to an unresolved branch) — resolving them fully requires reconstructing a repo from a
    possibly-decoupled git-dir/work-tree pair, which is more ambiguous than a plain path; per the
    non-negotiable fail-safe boundary, ambiguity means BLOCK, not a guess.
  - Any path that cannot be resolved to a real worktree HEAD (target doesn't exist, isn't a git
    worktree, `cd -`/a flag-bearing `cd` we don't special-case, a dynamic `$(...)`/backtick target,
    or the git call itself fails/hangs) marks the branch UNRESOLVED for the rest of that state's
    lifetime. An UNRESOLVED branch only matters — and only then fails safe (UNSAFE) — for a segment
    whose verdict actually depends on knowing the current branch (commit/merge/cherry-pick/rebase/
    revert, or a bare `git push`/`git push <remote>` with no explicit refspec); an explicit-refspec
    push's destination never depends on cwd and is judged exactly as before.

Net effect: a `cd <isolated-leaf-worktree> && git commit` is now judged by the LEAF branch (ALLOW if
not protected) instead of the payload cwd's (possibly protected) branch — eliminating the recurring
false-positive — while a `cd <worktree-on-a-protected-branch> && git commit` still resolves to that
protected branch and is correctly BLOCKed, and any unresolvable target still fails CLOSED. This is
strictly a refinement of variant 3: every variant-3 verdict that did not depend on `cd`/`-C` is
unchanged; only the previously-unconditional `-C`-implies-UNSAFE case is relaxed to an actual
resolution attempt (still UNSAFE if that resolution fails).
"""

import fnmatch
import os
import shlex
import subprocess
import sys

FORCE_FLAGS = {"--force", "-f", "--force-with-lease", "--force-if-includes"}
# push flags that take a SEPARATE value token we cannot safely skip without git's full option
# grammar; seeing one of these means we cannot reliably tell a flag's value apart from a
# positional remote/refspec, so the caller fails safe (UNSAFE) rather than guess.
PUSH_FLAGS_WITH_VALUE = {"-o", "--repo", "--receive-pack", "--push-option"}
MUTATING_SUBCMDS = {"commit", "merge", "cherry-pick", "rebase", "revert"}
# git global options that redirect git at a repo/worktree we do NOT attempt to resolve (see the
# variant-4 docstring above): reconstructing the effective worktree from a git-dir/work-tree pair
# is more ambiguous than a plain path, so these stay on the original fail-closed-when-it-matters
# path. `-C` is handled separately (and DOES get resolved) — it is intentionally not in this set.
UNSUPPORTED_REDIRECT_FLAGS = {"--git-dir", "--work-tree", "--namespace"}
WRAPPER_TOKENS = {"sudo", "command", "time", "env"}
# Command/process substitution: the actual text git would see is produced at runtime and is not
# statically knowable from the command string alone. This is fail-safe-relevant ONLY when the
# substitution can obscure a git-MUTATING operation's target or force flags — i.e. only inside a
# `git push`'s own argument tokens, a `-C` path, or the git subcommand position itself. A
# substitution anywhere else (a read-only `git diff`/`git log`, an `echo`, a `VAR=$(…)` assignment,
# a non-git command) cannot change a protected-branch/force verdict and MUST be allowed — scoping
# it this way is what keeps ubiquitous command-substitution from being blocked everywhere.
DYNAMIC_MARKERS = ["$(", "`", "<(", ">("]
# A hung/slow filesystem or a huge/foreign repo must not wedge the hook indefinitely.
GIT_RESOLVE_TIMEOUT_SECONDS = 3


def has_dynamic_marker(token: str) -> bool:
    return any(marker in token for marker in DYNAMIC_MARKERS)


class Unsafe(Exception):
    """Raised whenever the command's git structure cannot be confidently resolved. The caller
    always treats this as BLOCK (exit 2) — the fail-safe default-deny path."""


class Cwd:
    """The effective shell working directory + its resolved git branch, threaded through the
    segment walk. Updated ONLY by a real (non-git) `cd` segment — a git invocation's own `-C` is
    scoped to that invocation alone and must never mutate this persistent state (see module
    docstring, variant 4). `path`/`branch` are both `None` when we have lost track (fail-closed
    marker): a later segment that NEEDS the branch to render a verdict then raises `Unsafe`."""

    __slots__ = ("path", "branch")

    def __init__(self, path, branch):
        self.path = path
        self.branch = branch


def resolve_branch(path):
    """Best-effort: the branch checked out at `path`'s worktree HEAD, or None if it cannot be
    determined (path missing/not a git worktree, git itself fails, or the call hangs/errors).
    `None` is always treated as UNRESOLVED by callers, which fail safe rather than silently
    assume the operation is fine."""
    if not path:
        return None
    try:
        proc = subprocess.run(
            ["git", "-C", path, "rev-parse", "--abbrev-ref", "HEAD"],
            capture_output=True,
            text=True,
            timeout=GIT_RESOLVE_TIMEOUT_SECONDS,
        )
    except (OSError, subprocess.SubprocessError):
        return None
    if proc.returncode != 0:
        return None
    branch = proc.stdout.strip()
    return branch or None


def resolve_path(base, target):
    """Join `target` onto `base` the way a shell `cd` / git `-C` would. An absolute `target`
    always wins outright (matches both `cd`/`-C` semantics); a relative `target` needs a known
    `base` and returns None (unresolvable) if `base` is itself unknown."""
    if os.path.isabs(target):
        return os.path.normpath(target)
    if base is None:
        return None
    return os.path.normpath(os.path.join(base, target))


def apply_cd(seg, cwd: Cwd) -> Cwd:
    """Handle a plain (non-git) `cd` segment: resolve the new effective path + branch, or mark
    both UNRESOLVED (None) if the target is dynamic, `cd -` (a previous-dir stack we don't
    track), or carries a flag we don't special-case — a later mutating/bare-push segment that
    depends on this state then fails safe rather than guessing."""
    if len(seg) > 2:
        # `cd -L /path`, `cd -P /path`, etc. — flags we don't special-case; don't guess.
        return Cwd(None, None)
    target_tok = seg[1] if len(seg) == 2 else "~"
    if has_dynamic_marker(target_tok) or target_tok == "-":
        return Cwd(None, None)
    target = os.path.expanduser(target_tok)
    new_path = resolve_path(cwd.path, target)
    new_branch = resolve_branch(new_path)
    if new_path is None or new_branch is None:
        return Cwd(None, None)
    return Cwd(new_path, new_branch)


def split_top_level(cmd: str):
    """Split `cmd` into a list of token-lists, one per top-level shell segment, cutting at
    unquoted &&, ||, ;, bare &, |, and newlines. Uses shlex (POSIX mode) for tokenization so an
    operator INSIDE a quoted string (e.g. a commit message containing "&&") is part of that
    token, never treated as a real operator boundary."""
    lexer = shlex.shlex(cmd, posix=True, punctuation_chars="&|;")
    lexer.whitespace_split = True
    try:
        tokens = list(lexer)
    except ValueError as e:
        # e.g. "No closing quotation" - malformed/obfuscated quoting.
        raise Unsafe(f"unterminated quote or escape in command ({e})") from e

    segments = []
    current = []
    for tok in tokens:
        if tok in ("&&", "||", ";", "&", "|", "\n"):
            segments.append(current)
            current = []
            continue
        current.append(tok)
    segments.append(current)
    return [seg for seg in segments if seg]


def refspec_dest(refspec: str):
    """Return (destination_branch, force) for a single git-push refspec token, per git's own
    refspec grammar: a leading '+' forces that refspec; 'src:dst' pushes src to dst; a bare
    'branch' pushes local 'branch' to remote 'branch'; 'refs/heads/x' prefixes are stripped."""
    spec = refspec
    force = spec.startswith("+")
    if force:
        spec = spec[1:]
    dst = spec.split(":", 1)[1] if ":" in spec else spec
    if dst.startswith("refs/heads/"):
        dst = dst[len("refs/heads/") :]
    return dst, force


def is_protected(branch: str, protected_patterns):
    return any(fnmatch.fnmatch(branch, pat) for pat in protected_patterns)


def analyze_push(push_args, invocation_branch, protected_patterns):
    """push_args = tokens after 'git push' (git's own global flags/subcommand already consumed).
    `invocation_branch` is the branch this specific invocation resolves to (None if unresolved) —
    only consulted for a BARE push with no explicit refspec, since that is the only case whose
    destination depends on the current branch. Returns None if allowed, or a block-reason string.
    """
    force = False
    positionals = []
    i = 0
    while i < len(push_args):
        tok = push_args[i]
        # A command/process substitution INSIDE a push's own args can obscure the destination
        # branch or a force flag (e.g. `git push origin $(echo dev)` or `git push $(cat what.txt)`),
        # so it is unresolvable HERE and must fail safe.
        if has_dynamic_marker(tok):
            raise Unsafe(
                "'git push' argument contains a command/process substitution "
                f"({tok!r}); the destination/force cannot be statically resolved"
            )
        if tok in FORCE_FLAGS:
            force = True
            i += 1
            continue
        if tok.startswith("-"):
            if tok in PUSH_FLAGS_WITH_VALUE:
                raise Unsafe(
                    f"'git push' flag '{tok}' takes a separate value we do not parse; "
                    "cannot confidently tell its value apart from a positional remote/refspec"
                )
            i += 1
            continue
        positionals.append(tok)
        i += 1

    if not positionals or len(positionals) == 1:
        # Bare 'git push' or 'git push <remote>' with no explicit refspec: pushes the CURRENT
        # branch (per configured upstream / push.default) — depends on this invocation's resolved
        # branch, so an unresolved one (an unresolvable cd/-C/--git-dir/--work-tree target) fails
        # safe rather than silently allowing.
        if invocation_branch is None:
            raise Unsafe(
                "'git push' with no explicit refspec depends on the current branch, which this "
                "guard could not resolve (an unresolvable cd/-C/--git-dir/--work-tree target)"
            )
        dst = invocation_branch
        if is_protected(dst, protected_patterns):
            return f"push with no explicit refspec targets current (protected) branch '{dst}'"
        if force:
            return "force-push flag present on 'git push'"
        return None

    # positionals[0] is the remote; the rest are refspecs. An explicit refspec's destination is
    # self-contained text — it never depends on the current branch, so it is judged the same way
    # regardless of whether `invocation_branch` resolved.
    for rs in positionals[1:]:
        dst, rs_force = refspec_dest(rs)
        if rs_force:
            return f"refspec '{rs}' has a leading '+' (force-push)"
        if is_protected(dst, protected_patterns):
            return f"push destination '{dst}' is a protected branch"
    if force:
        return "force-push flag present on 'git push'"
    return None


def analyze_segment(seg, cwd: Cwd, protected_patterns):
    """Returns None if the segment is allowed/inert, or a block-reason string. Raises Unsafe if
    the segment cannot be confidently classified. `cwd` is the PERSISTENT effective cwd/branch as
    of this point in the walk (mutated only by `cd` segments in analyze(), never here)."""
    idx = 0
    while idx < len(seg) and seg[idx] in WRAPPER_TOKENS:
        idx += 1
    if idx >= len(seg) or seg[idx] != "git":
        return None  # not a git invocation - inert to this guard

    idx += 1
    c_paths = []  # this invocation's own -C path(s), applied like a chained cd, in order
    unsupported_redirect = False
    while idx < len(seg) and seg[idx].startswith("-") and seg[idx] != "-":
        tok = seg[idx]
        bare = tok.split("=", 1)[0]

        if bare == "-C":
            if "=" in tok:
                c_paths.append(tok.split("=", 1)[1])
                idx += 1
            else:
                if idx + 1 >= len(seg):
                    raise Unsafe("git '-C' given with no following path argument")
                nxt = seg[idx + 1]
                if has_dynamic_marker(nxt):
                    raise Unsafe(
                        "git '-C' path is produced by a command/process substitution "
                        f"({nxt!r}); cannot resolve the redirected worktree"
                    )
                c_paths.append(nxt)
                idx += 2
            continue

        if bare in UNSUPPORTED_REDIRECT_FLAGS:
            unsupported_redirect = True

        # options that take a separate value token (git global options); '=' form is one token.
        if bare in ("-c", "--git-dir", "--work-tree", "--namespace") and "=" not in tok:
            idx += 2
        else:
            idx += 1

    if idx >= len(seg):
        return None  # bare 'git' (or only global flags) - nothing to judge

    subcmd = seg[idx]
    rest = seg[idx + 1 :]

    # If the subcommand token ITSELF is produced by a substitution (e.g. `git $(echo push) …`), we
    # cannot tell whether this is a push/mutating op or an inert one — fail safe.
    if has_dynamic_marker(subcmd):
        raise Unsafe(
            "git subcommand is produced by a command/process substitution "
            f"({subcmd!r}); cannot tell whether it is a push/mutating operation"
        )

    # Resolve the branch THIS INVOCATION judges against. `-C` is scoped to this call only (it
    # never mutates the persistent `cwd` a later `cd` segment builds on); `--git-dir`/`--work-tree`
    # remain on the original fail-closed-when-it-matters path (see module docstring).
    if unsupported_redirect:
        invocation_branch = None
    elif c_paths:
        path = cwd.path
        for c in c_paths:
            path = resolve_path(path, c)
        invocation_branch = resolve_branch(path)
    else:
        invocation_branch = cwd.branch

    if subcmd == "push":
        return analyze_push(rest, invocation_branch, protected_patterns)

    if subcmd in MUTATING_SUBCMDS:
        if invocation_branch is None:
            raise Unsafe(
                f"'git {subcmd}' targets a branch this guard could not resolve — an "
                "unresolvable cd/-C/--git-dir/--work-tree target; refusing to guess"
            )
        if is_protected(invocation_branch, protected_patterns):
            return f"'git {subcmd}' on protected branch '{invocation_branch}'"
        return None

    return None  # every other git subcommand (status, fetch, worktree, branch, log, ...) is inert


def analyze(cmd: str, current_branch: str, protected_patterns, run_dir=None):
    """Top-level entry: returns None if allowed, else a block-reason string. Raises Unsafe for
    the fail-safe (default-deny) path.

    `current_branch`/`run_dir` seed the initial effective cwd (the payload's `.cwd` and its
    already-resolved HEAD, computed once by the bash hook). A leading `cd <path>` segment then
    updates this state for every later segment in the SAME command (a real shell cd persists);
    a git invocation's own `-C` resolves only that invocation (see module docstring).

    Note: command/process substitution ($(…)/`…`/<(…)/>(…)) is NOT rejected at the whole-command
    level — that over-blocked ubiquitous read-only substitution (e.g. `echo "$(git diff … | wc -l)"`,
    a `VAR=$(git rev-parse HEAD)` assignment) that can never change a protected-branch/force verdict.
    The substitution fail-safe is applied narrowly: a `git push`'s own argument tokens, a `-C` path,
    or the git subcommand position — the only places it could actually obscure a verdict.
    """
    cwd = Cwd(run_dir, current_branch)
    for seg in split_top_level(cmd):
        if seg[0] == "cd":
            cwd = apply_cd(seg, cwd)
            continue
        reason = analyze_segment(seg, cwd, protected_patterns)
        if reason is not None:
            return reason
    return None


def main() -> int:
    if len(sys.argv) < 2:
        print("UNSAFE: missing command argument", file=sys.stderr)
        return 2
    cmd = sys.argv[1]
    current_branch = sys.argv[2] if len(sys.argv) > 2 else "HEAD"
    protected_patterns = (
        sys.argv[3].split()
        if len(sys.argv) > 3
        else ["main", "integration", "dev", "claude/head/*"]
    )
    run_dir = sys.argv[4] if len(sys.argv) > 4 and sys.argv[4] else None

    try:
        reason = analyze(cmd, current_branch, protected_patterns, run_dir)
    except Unsafe as e:
        print(f"UNSAFE: {e}", file=sys.stderr)
        return 2

    if reason is not None:
        print(f"BLOCK: {reason}", file=sys.stderr)
        return 1

    print("ALLOW")
    return 0


if __name__ == "__main__":
    sys.exit(main())
