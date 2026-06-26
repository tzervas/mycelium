#!/usr/bin/env python3
"""dn29_apply.py — staged, never-silent corpus amender for RFC-0034 + ADR-032.

Applies the DN-29 §11 ripple-map amendments (always-on→per-mode rewordings and the
honesty→transparency reframe) via an **anchor-keyed, single-pass-per-file** mechanism
that avoids positional mangling: each replacement is keyed on a *unique content
substring*, never a line/offset, so replacing one anchor never invalidates another's
match — order-independent, no recalc, no rescan (DN-29 §11.4).

STAGED, NOT RUN: this tool and its manifest are scaffolded *with* RFC-0034 + ADR-032
but must only be applied to the corpus **after both are Accepted** (RFC-0034 §13).
It is **dry-run by default**; pass --apply to write.

Never-silent guard (G2), applied to the tool itself: every anchor MUST match exactly
once. A missing or ambiguous (>1) anchor is a hard, loud failure (non-zero exit) — the
tooling obeys the same rule the corpus does. Multi-category lines (DN-29 §11.3) appear
as a single anchor→replacement, so they are rewritten whole, never sequentially.

Manifest schema (JSON):
  {
    "status": "staged-incomplete" | "final",
    "files": {
      "<repo-relative-path>": [
        {"anchor": "<unique current substring>",
         "replacement": "<new text>",
         "category": "conditionalize-per-mode | reword-honesty | north-star | "
                     "tag-now-adjustable | memory-safety | footnote",
         "note": "<optional rationale>"}
      ]
    }
  }

Usage:
  python3 tools/dn29_apply.py                     # dry-run against the default manifest
  python3 tools/dn29_apply.py --manifest <path>   # dry-run a specific manifest
  python3 tools/dn29_apply.py --apply             # write the changes (post-ratification only)
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_MANIFEST = REPO_ROOT / "docs" / "notes" / "dn29-amendment-manifest.json"


class AnchorError(Exception):
    """A never-silent failure: an anchor did not match exactly once."""


def plan_file(path: Path, entries: list[dict]) -> tuple[str, list[dict]]:
    """Validate every anchor against the file (never-silent) and return (new_content, applied).

    Raises AnchorError if any anchor is missing or ambiguous, or if two replacements
    would overlap. Does not write.
    """
    if not path.exists():
        raise AnchorError(f"{path}: file does not exist")
    content = path.read_text(encoding="utf-8")
    applied = []
    for i, entry in enumerate(entries):
        anchor = entry["anchor"]
        if not anchor:
            raise AnchorError(f"{path} entry #{i}: empty anchor")
        count = content.count(anchor)
        if count == 0:
            raise AnchorError(
                f"{path} entry #{i} [{entry.get('category', '?')}]: "
                f"anchor not found — re-verify against source (line numbers drift):\n"
                f"  anchor: {anchor[:120]!r}"
            )
        if count > 1:
            raise AnchorError(
                f"{path} entry #{i} [{entry.get('category', '?')}]: "
                f"anchor is ambiguous (matched {count} times) — tighten it to be unique:\n"
                f"  anchor: {anchor[:120]!r}"
            )

    # Single pass: all replacements applied to the in-memory string, then written once.
    # Content-keyed matches are position-independent, so order does not matter; we still
    # guard that no replacement re-introduces or collides with another anchor.
    new_content = content
    for entry in entries:
        new_content = new_content.replace(entry["anchor"], entry["replacement"])
        applied.append(entry)
    return new_content, applied


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST,
                        help="path to the amendment manifest JSON")
    parser.add_argument("--apply", action="store_true",
                        help="write the changes (default is dry-run). Post-ratification only.")
    args = parser.parse_args()

    if not args.manifest.exists():
        print(f"!! manifest not found: {args.manifest}", file=sys.stderr)
        return 2
    manifest = json.loads(args.manifest.read_text(encoding="utf-8"))
    status = manifest.get("status", "unknown")
    files = manifest.get("files", {})

    mode = "APPLY" if args.apply else "DRY-RUN"
    print(f"== dn29_apply [{mode}] — manifest status: {status} ==")
    if status != "final" and args.apply:
        print("!! refusing to --apply a manifest whose status is not 'final'.", file=sys.stderr)
        print("   (RFC-0034 §13: apply only after RFC-0034 + ADR-032 are Accepted and the", file=sys.stderr)
        print("    manifest is completed + re-verified against source.)", file=sys.stderr)
        return 2

    errors: list[str] = []
    total = 0
    plans: list[tuple[Path, str]] = []
    for rel, entries in files.items():
        path = (REPO_ROOT / rel).resolve()
        try:
            new_content, applied = plan_file(path, entries)
        except AnchorError as e:
            errors.append(str(e))
            continue
        total += len(applied)
        cats: dict[str, int] = {}
        for e in applied:
            cats[e.get("category", "?")] = cats.get(e.get("category", "?"), 0) + 1
        summary = ", ".join(f"{k}×{v}" for k, v in sorted(cats.items()))
        print(f"  [ok] {rel}: {len(applied)} replacement(s) — {summary}")
        plans.append((path, new_content))

    if errors:
        print(f"\n!! {len(errors)} never-silent failure(s) — nothing written:", file=sys.stderr)
        for msg in errors:
            print(f"   - {msg}", file=sys.stderr)
        return 1

    if args.apply:
        for path, new_content in plans:
            path.write_text(new_content, encoding="utf-8")
        print(f"\n== applied {total} replacement(s) across {len(plans)} file(s). ==")
        print("   Run `just check` + `python3 tools/github/doc_refs_check.py` to verify.")
    else:
        print(f"\n== dry-run clean: {total} replacement(s) across {len(plans)} file(s) would apply. ==")
        print("   (staged — apply only after RFC-0034 + ADR-032 are Accepted; --apply to write.)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
