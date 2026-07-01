#!/usr/bin/env python3
"""Structural parser for the branch-guard PreToolUse hook (claude-git-branch-guard.sh).

Parses a shell command string PER GIT INVOCATION (not as one regex-scanned blob) so that
compound commands are judged by what each git subcommand actually does, not by which words
happen to appear anywhere in the string. See the header comment in
scripts/hooks/claude-git-branch-guard.sh for the full design rationale and the fail-safe
boundary this module implements.

Usage: branch_guard_parse.py <command> <current-branch> <protected-patterns-space-separated>
Exit 0 + prints "ALLOW"           -> caller allows the Bash tool call.
Exit 1 + prints "BLOCK: <reason>" -> caller blocks; a real protected-branch/force-push violation.
Exit 2 + prints "UNSAFE: <reason>"-> caller blocks; the command could not be confidently parsed
                                      (fail-safe / default-deny — never a false ALLOW).

No third-party dependencies (stdlib only: shlex, fnmatch) so it runs in any environment that has
python3, matching the other scripts/checks/*.sh hooks that already shell out to python3.
"""

import fnmatch
import shlex
import sys

FORCE_FLAGS = {"--force", "-f", "--force-with-lease", "--force-if-includes"}
# push flags that take a SEPARATE value token we cannot safely skip without git's full option
# grammar; seeing one of these means we cannot reliably tell a flag's value apart from a
# positional remote/refspec, so the caller fails safe (UNSAFE) rather than guess.
PUSH_FLAGS_WITH_VALUE = {"-o", "--repo", "--receive-pack", "--push-option"}
MUTATING_SUBCMDS = {"commit", "merge", "cherry-pick", "rebase", "revert"}
# git global options that redirect git at a DIFFERENT repo/worktree than the payload cwd. Any
# git invocation carrying one of these means the CURRENT branch we resolved (from the payload
# cwd) may not be the branch that invocation actually affects.
GIT_DIR_REDIRECT_FLAGS = {"-C", "--git-dir", "--work-tree", "--namespace"}
WRAPPER_TOKENS = {"sudo", "command", "time", "env"}
# Command/process substitution: the actual text git would see is produced at runtime and is not
# statically knowable from the command string alone. This is fail-safe-relevant ONLY when the
# substitution can obscure a git-MUTATING operation's target or force flags — i.e. only inside a
# `git push`'s own argument tokens (which determine the destination branch and force flags). A
# substitution anywhere else (a read-only `git diff`/`git log`, an `echo`, a `VAR=$(…)` assignment,
# a non-git command) cannot change a protected-branch/force verdict and MUST be allowed — scoping
# it to push args is what keeps ubiquitous command-substitution from being blocked everywhere.
DYNAMIC_MARKERS = ["$(", "`", "<(", ">("]


def has_dynamic_marker(token: str) -> bool:
    return any(marker in token for marker in DYNAMIC_MARKERS)


class Unsafe(Exception):
    """Raised whenever the command's git structure cannot be confidently resolved. The caller
    always treats this as BLOCK (exit 2) — the fail-safe default-deny path."""


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


def analyze_push(push_args, current_branch, protected_patterns, dir_redirected):
    """push_args = tokens after 'git push' (git's own global flags/subcommand already consumed).
    Returns None if allowed, or a block-reason string."""
    force = False
    positionals = []
    i = 0
    while i < len(push_args):
        tok = push_args[i]
        # A command/process substitution INSIDE a push's own args can obscure the destination
        # branch or a force flag (e.g. `git push origin $(echo dev)` or `git push $(cat what.txt)`),
        # so it is unresolvable HERE and must fail safe. (Substitution outside a push segment is
        # handled by simply not reaching this function — see analyze()/analyze_segment.)
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
        # branch (per configured upstream / push.default). This depends on cwd HEAD, so if the
        # invocation redirected git elsewhere (-C / --git-dir / --work-tree) we cannot resolve it.
        if dir_redirected:
            raise Unsafe(
                "'git push' with no explicit refspec, combined with -C/--git-dir/--work-tree, "
                "depends on the redirected repo's current branch which this guard does not inspect"
            )
        dst = current_branch
        if is_protected(dst, protected_patterns):
            return f"push with no explicit refspec targets current (protected) branch '{dst}'"
        if force:
            return "force-push flag present on 'git push'"
        return None

    # positionals[0] is the remote; the rest are refspecs.
    for rs in positionals[1:]:
        dst, rs_force = refspec_dest(rs)
        if rs_force:
            return f"refspec '{rs}' has a leading '+' (force-push)"
        if is_protected(dst, protected_patterns):
            return f"push destination '{dst}' is a protected branch"
    if force:
        return "force-push flag present on 'git push'"
    return None


def analyze_segment(seg, current_branch, protected_patterns):
    """Returns None if the segment is allowed/inert, or a block-reason string. Raises Unsafe if
    the segment cannot be confidently classified."""
    idx = 0
    while idx < len(seg) and seg[idx] in WRAPPER_TOKENS:
        idx += 1
    if idx >= len(seg) or seg[idx] != "git":
        return None  # not a git invocation - inert to this guard

    idx += 1
    dir_redirected = False
    while idx < len(seg) and seg[idx].startswith("-") and seg[idx] != "-":
        tok = seg[idx]
        bare = tok.split("=", 1)[0]
        if bare in GIT_DIR_REDIRECT_FLAGS:
            dir_redirected = True
        # options that take a separate value token (git global options); '=' form is one token.
        if (
            bare in ("-C", "-c", "--git-dir", "--work-tree", "--namespace")
            and "=" not in tok
        ):
            idx += 2
        else:
            idx += 1

    if idx >= len(seg):
        return None  # bare 'git' (or only global flags) - nothing to judge

    subcmd = seg[idx]
    rest = seg[idx + 1 :]

    # If the subcommand token ITSELF is produced by a substitution (e.g. `git $(echo push) …`), we
    # cannot tell whether this is a push/mutating op or an inert one — fail safe. (Narrow: this only
    # fires when the substitution is in the git subcommand position, never for read-only segments,
    # echo, or assignments, so it does not re-introduce the over-block this rewrite removes.)
    if has_dynamic_marker(subcmd):
        raise Unsafe(
            "git subcommand is produced by a command/process substitution "
            f"({subcmd!r}); cannot tell whether it is a push/mutating operation"
        )

    if subcmd == "push":
        return analyze_push(rest, current_branch, protected_patterns, dir_redirected)

    if subcmd in MUTATING_SUBCMDS:
        if dir_redirected:
            raise Unsafe(
                f"'git {subcmd}' combined with -C/--git-dir/--work-tree depends on the "
                "redirected repo's current branch which this guard does not inspect"
            )
        if is_protected(current_branch, protected_patterns):
            return f"'git {subcmd}' on protected branch '{current_branch}'"
        return None

    return None  # every other git subcommand (status, fetch, worktree, branch, log, ...) is inert


def analyze(cmd: str, current_branch: str, protected_patterns):
    """Top-level entry: returns None if allowed, else a block-reason string. Raises Unsafe for
    the fail-safe (default-deny) path.

    Note: command/process substitution ($(…)/`…`/<(…)/>(…)) is NOT rejected at the whole-command
    level — that over-blocked ubiquitous read-only substitution (e.g. `echo "$(git diff … | wc -l)"`,
    a `VAR=$(git rev-parse HEAD)` assignment) that can never change a protected-branch/force verdict.
    The substitution fail-safe is applied narrowly, inside analyze_push, only to a `git push`'s own
    argument tokens where it can actually obscure the destination or force flags. Mutating
    subcommands (commit/merge/cherry-pick/rebase/revert) are judged by the cwd HEAD independently of
    their args, so a substitution in their args cannot flip the verdict and is intentionally ignored.
    """
    for seg in split_top_level(cmd):
        reason = analyze_segment(seg, current_branch, protected_patterns)
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

    try:
        reason = analyze(cmd, current_branch, protected_patterns)
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
