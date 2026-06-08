#!/usr/bin/env python3
"""Offline cross-reference / relative-link checker for the Mycelium docs corpus.

Scans tracked markdown for:
  - inline links            [text](target)
  - reference definitions   [id]: target
  - CLAUDE.md @imports      @path/to/file
Verifies that *relative* targets resolve to a file (or directory) in the repo.
Skips external (http/https/mailto/tel), in-page anchors (#...), and template
placeholders. Anchors on a path (file.md#section) are stripped before checking.

No network. Exit 1 if any broken reference is found.
"""
from __future__ import annotations
import re, subprocess, sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
INLINE = re.compile(r"(?<!\!)\[[^\]]*\]\(\s*<?([^)\s>]+)>?(?:\s+\"[^\"]*\")?\s*\)")
REFDEF = re.compile(r"^\s{0,3}\[[^\]]+\]:\s*<?([^\s>]+)>?", re.MULTILINE)
ATIMPORT = re.compile(r"(?:^|\s)@([./~][\w./\-]+)")
SKIP_PREFIX = ("http://", "https://", "mailto:", "tel:", "#", "data:")
# Placeholder targets that are intentionally not real paths.
PLACEHOLDER = re.compile(r"\((issue|owner|repo|version|path|url|name)\)|<[^>]+>|\{[^}]+\}", re.I)


def tracked_markdown() -> list[Path]:
    out = subprocess.run(
        ["git", "ls-files", "-z", "--", "*.md"],
        cwd=REPO, capture_output=True, text=True, check=True,
    ).stdout
    return [REPO / p for p in out.split("\0") if p]


def resolve(target: str, src: Path) -> Path:
    target = target.split("#", 1)[0].strip()
    if target.startswith("/"):
        return REPO / target.lstrip("/")
    if target.startswith("~"):
        return Path(target).expanduser()
    return (src.parent / target)


def main() -> int:
    broken: list[str] = []
    for md in tracked_markdown():
        text = md.read_text(encoding="utf-8", errors="replace")
        targets = [(m.group(1), INLINE) for m in INLINE.finditer(text)]
        targets += [(m.group(1), REFDEF) for m in REFDEF.finditer(text)]
        if md.name in ("CLAUDE.md", "CLAUDE.local.md"):
            targets += [(m.group(1), ATIMPORT) for m in ATIMPORT.finditer(text)]
        rel = md.relative_to(REPO)
        for raw, _ in targets:
            t = raw.strip()
            if not t or t.startswith(SKIP_PREFIX) or PLACEHOLDER.search(t):
                continue
            if t.split("#", 1)[0] == "":  # pure anchor
                continue
            p = resolve(t, md)
            try:
                p = p.resolve()
            except OSError:
                broken.append(f"{rel}: bad path: {raw}")
                continue
            if not p.exists():
                broken.append(f"{rel} -> {raw}")
    if broken:
        print(f"  {len(broken)} broken relative reference(s):")
        for b in sorted(set(broken)):
            print(f"    {b}")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
