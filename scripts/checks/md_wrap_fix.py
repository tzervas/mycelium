#!/usr/bin/env python3
"""Reflow the persistent MD004 soft-wrap marker-at-line-start pitfall (findings-driven, never-silent).

## The pitfall (CLAUDE.md §"Markdown authoring")

`markdownlint` reads any line whose first non-space char is `+`, `-`, or `*` (followed by a
space) as an unordered-list item. So prose that *soft-wraps* such that a binary operator or
conjunction lands at line start — e.g. wrapping ``the self-hosting differentials + the VSA
bundle`` so `+ the VSA bundle` begins the next line — trips **MD004/ul-style** and fails the
`markdown` gate, even though it is plain prose, not a list. Recurring, manual toil.

## Why NOT `markdownlint --fix`, and why NOT a blanket regex

- `markdownlint-cli2 --fix` *mis-fixes* MD004: it normalizes marker STYLE, so under the repo's
  `consistent` MD004 it treats the prose-wrap marker as canonical and rewrites genuine lists to
  match (verified: a real ``- item`` list became ``+ item``). Green gate, corrupted content.
- A blanket "reflow every `+`/`*` at line start" regex is also unsafe: some docs (e.g.
  `DN-15`) legitimately use `+` as their list marker, so a blanket pass mangles real lists that
  markdownlint — being per-doc `consistent` — never flagged.

## What this does (safe by construction)

It asks **markdownlint itself** for the MD004 findings, then reflows ONLY those exact lines:
move the offending marker off line-start by appending it to the previous line and dropping it
from the current line — behaviour-neutral for prose (``A\n+ B`` and ``A +\nB`` render the same)
and reversible. Because it only touches lines the gate already flags, a GREEN doc is a true
no-op (a `+`-list doc like DN-15 has zero MD004 findings ⇒ untouched). `+ `/`* `/`- ` with a
space is never emphasis, so the move is safe.

For the rare finding that looks like a genuinely mis-markered *list* item rather than a prose
wrap — the previous line is a list intro (ends with `:`) or is itself a list item, or the next
line is another item at the same marker — it REPORTS for manual review instead of rewriting
(G2: never silently mangle a real list). Every reflow is printed.

## Usage

    md_wrap_fix.py --fix   [FILE...]   # rewrite in place (pre-commit hook + `just md-fix`)
    md_wrap_fix.py --check [FILE...]   # report only; exit 1 if any reflow/ review is pending

With no FILE args it lints every git-tracked `*.md`. Skips gracefully (exit 0) when node/npx is
absent — mirrors `scripts/checks/markdown.sh`. Exit: 0 clean/fixed; 1 (--check) pending; 2 usage.
"""

from __future__ import annotations

import os
import re
import subprocess
import sys

# markdownlint-cli2 pin — MUST match scripts/checks/markdown.sh (reproducibility / no silent drift).
_MDL = "markdownlint-cli2@0.22.1"
_CONFIG = ".markdownlint.jsonc"

# A finding line from markdownlint-cli2, tolerant of the optional column and an "error" severity
# word: "<file>:<line>[:<col>] [error ]MD004/ul-style …".
_FINDING = re.compile(r"^(?P<file>.+?):(?P<line>\d+)(?::\d+)?\s+(?:error\s+)?MD004\b")
# The offending marker at line start (any of the three; the DIFFERING one is what markdownlint
# flagged — we don't need to know which, just to lift it off line-start).
_MARKER = re.compile(r"^(?P<indent>\s*)(?P<marker>[-+*]) (?P<rest>\S.*)$")
_ANY_MARKER = re.compile(r"^\s*(?:[-+*] |\d+[.)] )")


def _repo_root() -> str:
    """The repo top-level, so the run is cwd-independent (`.markdownlint.jsonc` and the tracked-file
    list resolve there regardless of where the script is invoked)."""
    root = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        capture_output=True,
        text=True,
        check=False,
    ).stdout.strip()
    return root or "."


def _tracked_md(root: str) -> list[str]:
    out = subprocess.run(
        ["git", "ls-files", "*.md"],
        capture_output=True,
        text=True,
        check=False,
        cwd=root,
    ).stdout
    return [p for p in out.splitlines() if p]


def _md004_findings(files: list[str], root: str) -> dict[str, set[int]] | None:
    """Map file → set of 1-based line numbers markdownlint flags as MD004. None ⇒ tool absent."""
    if subprocess.run(
        ["sh", "-c", "command -v npx"], capture_output=True, check=False
    ).returncode:
        return None
    proc = subprocess.run(
        ["npx", "--yes", _MDL, "--config", _CONFIG, *files],
        capture_output=True,
        text=True,
        check=False,
        cwd=root,
    )
    findings: dict[str, set[int]] = {}
    for line in (proc.stdout + proc.stderr).splitlines():
        m = _FINDING.match(line.strip())
        if m:
            findings.setdefault(m["file"], set()).add(int(m["line"]))
    return findings


def _looks_like_real_list(lines: list[str], i: int) -> bool:
    """True if the flagged line i (0-based) is more plausibly a real (mis-markered) list item than
    a prose wrap — in which case we report rather than rewrite."""
    if i == 0:
        return True  # nothing to reflow onto
    prev = lines[i - 1]
    if not prev.strip():
        return True  # list after a blank line
    if prev.rstrip().rstrip("*_`").rstrip().endswith(":"):
        return True  # a list intro ("Items:", "**Column definitions:**")
    if _ANY_MARKER.match(prev):
        return True  # previous line is itself a list item
    if i + 1 < len(lines):
        nxt = _MARKER.match(lines[i + 1])
        cur = _MARKER.match(lines[i])
        if nxt and cur and nxt["marker"] == cur["marker"]:
            return True  # next line is another item at the same marker
    return False


def process(path: str, flagged: set[int], write: bool, root: str = "."):
    """Return (reflowed:[(line, preview)], review:[(line, text)])."""
    full = os.path.join(root, path)
    with open(full, encoding="utf-8") as fh:
        original = fh.read()
    trailing = "\n" if original.endswith("\n") else ""
    lines = original.splitlines()
    reflowed: list[tuple[int, str]] = []
    review: list[tuple[int, str]] = []
    for ln in sorted(flagged):
        i = ln - 1
        if not 0 <= i < len(lines):
            continue
        m = _MARKER.match(lines[i])
        if not m:
            continue
        if _looks_like_real_list(lines, i):
            review.append((ln, lines[i].strip()[:70]))
            continue
        lines[i - 1] = f"{lines[i - 1]} {m['marker']}"
        lines[i] = f"{m['indent']}{m['rest']}"
        reflowed.append((ln, lines[i - 1].strip()[-60:]))
    if reflowed and write:
        with open(full, "w", encoding="utf-8") as fh:
            fh.write("\n".join(lines) + trailing)
    return reflowed, review


def main(argv: list[str]) -> int:
    args = argv[1:]
    if not args or args[0] not in ("--fix", "--check"):
        sys.stderr.write("usage: md_wrap_fix.py --fix|--check [FILE...]\n")
        return 2
    write = args[0] == "--fix"
    root = _repo_root()
    files = args[1:] or _tracked_md(root)
    if not files:
        print("  ok    no markdown")
        return 0
    findings = _md004_findings(files, root)
    if findings is None:
        print(
            "  skip  node/npx not found — MD004 reflow needs markdownlint-cli2 (`just setup`)"
        )
        return 0
    if not findings:
        print(f"  ok    {len(files)} doc(s): no MD004 soft-wrap findings")
        return 0
    any_reflow = any_review = False
    for path in sorted(findings):
        reflowed, review = process(path, findings[path], write, root)
        verb = "reflowed" if write else "would reflow"
        for ln, preview in reflowed:
            any_reflow = True
            print(f"  {verb} {path}:{ln}  MD004 soft-wrap → …{preview}")
        for ln, text in review:
            any_review = True
            print(
                f"  REVIEW {path}:{ln}  MD004 but looks like a real list (not reflowed): {text}"
            )
    if any_review:
        print(
            "  ↑ manual review: markdownlint flagged these but they resemble real lists"
        )
    if (any_reflow and not write) or any_review:
        if any_reflow and not write:
            print("markdown soft-wrap reflow pending — run `just fmt`")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
