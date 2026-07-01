#!/usr/bin/env python3
"""Documentation-currency checker (offline).

Catches *staleness* the link/quality-bar checks do not: the navigational docs
drifting out of agreement with the repository they describe. Three assertions,
each cheap, deterministic, and skip-free:

1. **Structure currency** — the "Repository structure" tree (in
   docs/guide/repository-structure.md, or the root README as a fallback) lists
   every significant top-level entry, and names nothing that does not exist on disk.
2. **Index coverage** — every RFC / ADR / DN decision doc is referenced by id in
   `docs/Doc-Index.md` (a new RFC that never got indexed is the failure mode).
3. **Cited-count currency** — an opt-in `<!-- doc-currency:crate-count -->` marker
   line's first integer equals the real crate count, so a number cited in prose
   cannot silently rot.

It deliberately does NOT re-check relative links (that is `scripts/lint_links.py`
/ `links.sh`) nor the doc-IR quality bar (`myc-doc`). Pure currency/coverage.

Exit 0 = all current; exit 1 = at least one discrepancy (printed with a fix hint).
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

# Top-level directories that are config/build noise, not part of the documented
# project structure. Dot-entries are ignored wholesale (they may still be listed
# in the README — the phantom check validates anything that *is* listed).
# `mutants.out`/`mutants.out.old` are cargo-mutants runtime output (git-ignored, written by
# `just mutants` into the cwd) — transient artifacts, not repo structure (M-654).
IGNORE_TOPLEVEL = {"target", "oom", "__pycache__", "mutants.out", "mutants.out.old", "node_modules"}

# Top-level files the README tree is expected to mention (the load-bearing ones).
KEY_FILES = {
    "README.md",
    "LICENSE",
    "CONTRIBUTING.md",
    "CHANGELOG.md",
    "CLAUDE.md",
    "Cargo.toml",
    "justfile",
}

README = REPO_ROOT / "README.md"
DOC_INDEX = REPO_ROOT / "docs" / "Doc-Index.md"
# The repository-structure tree was decomposed out of the root README into a
# dedicated guide doc; the currency check follows it there (falls back to README
# if the guide is absent). The crate-count marker still lives in the README.
STRUCTURE_DOC = REPO_ROOT / "docs" / "guide" / "repository-structure.md"


def fail(msg: str) -> None:
    print(f"  doc-currency: {msg}")


def required_toplevel() -> set[str]:
    """Significant top-level entries the README must document."""
    required: set[str] = {f for f in KEY_FILES if (REPO_ROOT / f).exists()}
    for entry in REPO_ROOT.iterdir():
        if entry.name.startswith(".") or entry.name in IGNORE_TOPLEVEL:
            continue
        if entry.is_dir():
            required.add(entry.name)
    return required


def readme_structure_block(text: str) -> str | None:
    """Extract the fenced code block that follows the 'Repository structure'
    heading — the tree the structure check parses."""
    m = re.search(r"#+\s*Repository structure", text)
    if not m:
        return None
    rest = text[m.end() :]
    fence = re.search(r"```[a-zA-Z]*\n(.*?)```", rest, re.DOTALL)
    return fence.group(1) if fence else None


def first_level_entries(block: str) -> set[str]:
    """First-level children of the tree, i.e. lines like '├── crates/' (not the
    deeper '│   ├── ...'). Trailing slashes are normalized away."""
    names: set[str] = set()
    for line in block.splitlines():
        m = re.match(r"^[├└]──\s+(\S+)", line)
        if m:
            names.add(m.group(1).rstrip("/"))
    return names


def check_structure(errors: list[str]) -> None:
    doc = STRUCTURE_DOC if STRUCTURE_DOC.exists() else README
    if not doc.exists():
        errors.append("no repository-structure doc found (guide or README.md)")
        return
    rel = doc.relative_to(REPO_ROOT)
    text = doc.read_text(encoding="utf-8")
    block = readme_structure_block(text)
    if block is None:
        errors.append(
            f"{rel} has no fenced tree under a 'Repository structure' heading"
        )
        return
    listed = first_level_entries(block)
    required = required_toplevel()

    missing = sorted(r for r in required if r.rstrip("/") not in listed)
    for name in missing:
        errors.append(
            f"{rel} 'Repository structure' omits top-level entry: {name!r} "
            "(present on disk — add it to the tree)"
        )

    for name in sorted(listed):
        if not (REPO_ROOT / name).exists():
            errors.append(
                f"{rel} 'Repository structure' lists {name!r}, which does not "
                "exist at the repo root (phantom entry — remove or fix it)"
            )


def check_index_coverage(errors: list[str]) -> None:
    if not DOC_INDEX.exists():
        errors.append("docs/Doc-Index.md not found")
        return
    index_text = DOC_INDEX.read_text(encoding="utf-8")
    specs = [
        ("RFC", REPO_ROOT / "docs" / "rfcs", r"RFC-\d{4}"),
        ("ADR", REPO_ROOT / "docs" / "adr", r"ADR-\d{3}"),
        ("DN", REPO_ROOT / "docs" / "notes", r"DN-\d{2}"),
    ]
    for kind, directory, pattern in specs:
        if not directory.is_dir():
            continue
        for path in sorted(directory.glob(f"{kind}-*.md")):
            m = re.match(pattern, path.name)
            if not m:
                continue
            ident = m.group(0)
            if not re.search(rf"\b{re.escape(ident)}\b", index_text):
                errors.append(
                    f"docs/Doc-Index.md does not reference {ident} "
                    f"({path.relative_to(REPO_ROOT)} — index it)"
                )


def check_cited_counts(errors: list[str]) -> None:
    crate_count = len([p for p in (REPO_ROOT / "crates").glob("*") if p.is_dir()])
    marker = re.compile(r"<!--\s*doc-currency:crate-count\s*-->")
    for doc in (README, DOC_INDEX):
        if not doc.exists():
            continue
        for lineno, line in enumerate(doc.read_text(encoding="utf-8").splitlines(), 1):
            if not marker.search(line):
                continue
            nums = re.findall(r"\d+", marker.sub("", line))
            if not nums or int(nums[0]) != crate_count:
                errors.append(
                    f"{doc.relative_to(REPO_ROOT)}:{lineno} crate-count marker "
                    f"cites {nums[0] if nums else '<none>'}, but crates/ has "
                    f"{crate_count} (update the cited number)"
                )


def main() -> int:
    errors: list[str] = []
    check_structure(errors)
    check_index_coverage(errors)
    check_cited_counts(errors)
    if errors:
        for e in errors:
            fail(e)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
