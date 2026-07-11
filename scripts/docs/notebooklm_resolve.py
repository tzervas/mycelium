#!/usr/bin/env python3
"""Resolve a book-manifest-shaped cluster manifest to its ordered file list.

Shared by scripts/docs/export-notebooklm-pdfs.sh (the Markdown fallback path) and
usable standalone. Mirrors the myc-doc book-manifest resolution: per chapter,
`sources` (verbatim order) then `globs` (sorted) minus `exclude`. Never-silent:
prints resolved repo-relative paths one per line; a glob that matches nothing is
reported to stderr, not silently dropped (G2).

Guarantee: Empirical/Declared — a filesystem projection; the manifest + corpus are
ground truth.
"""

from __future__ import annotations

import glob as _glob
import json
import sys
from pathlib import Path


def resolve(manifest_path: str, repo_root: str) -> list[str]:
    root = Path(repo_root)
    data = json.loads(Path(manifest_path).read_text(encoding="utf-8"))
    out: list[str] = []
    seen: set[str] = set()
    for chapter in data.get("chapters", []):
        exclude = set(chapter.get("exclude", []))
        picked: list[str] = []
        for src in chapter.get("sources", []):
            if src not in exclude:
                picked.append(src)
        for pattern in chapter.get("globs", []):
            hits = sorted(
                str(Path(p).relative_to(root)) for p in _glob.glob(str(root / pattern))
            )
            if not hits:
                print(f"warn: glob matched nothing: {pattern}", file=sys.stderr)
            for h in hits:
                if h not in exclude:
                    picked.append(h)
        for rel in picked:
            if rel in seen:
                continue
            if not (root / rel).is_file():
                print(f"warn: missing file: {rel}", file=sys.stderr)
                continue
            seen.add(rel)
            out.append(rel)
    return out


def main() -> int:
    if len(sys.argv) < 2:
        print(
            "usage: notebooklm_resolve.py <manifest.json> [repo_root]", file=sys.stderr
        )
        return 2
    manifest = sys.argv[1]
    repo_root = sys.argv[2] if len(sys.argv) > 2 else "."
    files = resolve(manifest, repo_root)
    if not files:
        print(f"error: manifest resolved to zero files: {manifest}", file=sys.stderr)
        return 1
    print("\n".join(files))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
