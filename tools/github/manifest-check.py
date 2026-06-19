#!/usr/bin/env python3
"""Preflight consistency check for the Mycelium GitHub PM manifests.

Verifies — *before* any `gh` call — that every label and milestone referenced
in issues.yaml is actually defined in labels.json / milestones.json. This closes
the silent gap that previously stalled a bootstrap rerun: `gh issue create
--label <name>` errors if the label was never created from labels.json, so a
label used by issues.yaml but absent from the manifest leaves issues uncreated
with no obvious cause. Here that drift is an explicit, fail-fast error (G2:
never silent) rather than a half-finished sync.

It also reports the reverse (manifest entries unused by any issue) as a *warning*
only — an unused label/milestone is harmless, just noted.

Exit status: 0 = consistent; 1 = a referenced label/milestone is undefined.

Usage:
  python3 tools/github/manifest-check.py
  python3 tools/github/manifest-check.py --issues-yaml ... --labels ... --milestones ...
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

try:
    import yaml
except ImportError:  # pragma: no cover - environment guard
    sys.exit("PyYAML is required: `pip install pyyaml`.")

HERE = Path(__file__).resolve().parent

# Import doc_refs validator (same directory); soft-fail if unavailable so
# manifest-check can still run without the doc-index infrastructure.
try:
    import importlib.util as _ilu
    _spec = _ilu.spec_from_file_location(
        "doc_refs_check", HERE / "doc_refs_check.py"
    )
    _mod = _ilu.module_from_spec(_spec)  # type: ignore[arg-type]
    _spec.loader.exec_module(_mod)  # type: ignore[union-attr]
    _doc_refs_validate = _mod.validate  # type: ignore[attr-defined]
except Exception:  # pragma: no cover
    _doc_refs_validate = None  # type: ignore[assignment]


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate PM manifest consistency.")
    parser.add_argument("--issues-yaml", type=Path, default=HERE / "issues.yaml")
    parser.add_argument("--labels", type=Path, default=HERE / "labels.json")
    parser.add_argument("--milestones", type=Path, default=HERE / "milestones.json")
    args = parser.parse_args()

    spec = yaml.safe_load(args.issues_yaml.read_text())
    issues = spec.get("issues", []) if spec else []
    defined_labels = {d["name"] for d in json.loads(args.labels.read_text())}
    defined_ms = {d["title"] for d in json.loads(args.milestones.read_text())}

    used_labels: dict[str, list[str]] = {}
    used_ms: dict[str, list[str]] = {}
    for entry in issues:
        for label in entry.get("labels") or []:
            used_labels.setdefault(label, []).append(entry["id"])
        if entry.get("milestone"):
            used_ms.setdefault(entry["milestone"], []).append(entry["id"])

    errors = 0
    missing_labels = sorted(set(used_labels) - defined_labels)
    for name in missing_labels:
        ids = used_labels[name]
        print(
            f"ERROR: label '{name}' used by {len(ids)} issue(s) "
            f"(e.g. {', '.join(ids[:5])}) is absent from {args.labels.name}",
            file=sys.stderr,
        )
        errors += 1
    missing_ms = sorted(set(used_ms) - defined_ms)
    for title in missing_ms:
        ids = used_ms[title]
        print(
            f"ERROR: milestone '{title}' used by {len(ids)} issue(s) "
            f"(e.g. {', '.join(ids[:5])}) is absent from {args.milestones.name}",
            file=sys.stderr,
        )
        errors += 1

    # Reverse direction is advisory only — an unused manifest entry is harmless.
    for name in sorted(defined_labels - set(used_labels)):
        print(f"  note: label '{name}' is defined but unused by any issue")
    for title in sorted(defined_ms - set(used_ms)):
        print(f"  note: milestone '{title}' is defined but unused by any issue")

    if errors:
        print(
            f">> manifest check FAILED: {errors} undefined reference(s) "
            "— add them to the manifest before syncing.",
            file=sys.stderr,
        )
        return 1
    print(
        f">> manifest check OK: {len(issues)} issue(s); "
        f"{len(used_labels)} label(s) + {len(used_ms)} milestone(s) all defined."
    )

    # doc_refs validation (additive — runs after label/milestone checks pass)
    if _doc_refs_validate is not None:
        repo_root = HERE.parent.parent
        index_json = repo_root / "docs" / "api-index" / "index.json"
        doc_index = repo_root / "docs" / "Doc-Index.md"
        doc_ref_errors = _doc_refs_validate(
            args.issues_yaml, index_json, doc_index, repo_root
        )
        if doc_ref_errors:
            for err in doc_ref_errors:
                print(f"ERROR (doc_refs): {err}", file=sys.stderr)
            print(
                f">> doc_refs check FAILED: {len(doc_ref_errors)} dangling "
                "reference(s) — fix before syncing.",
                file=sys.stderr,
            )
            return 1
        if doc_ref_errors == []:
            # Only print OK if there were any doc_refs to check
            has_any = any(
                (issue.get("doc_refs") or []) for issue in issues
            )
            if has_any:
                print(">> doc_refs check OK: all doc_refs entries resolve.")
    else:
        print("  note: doc_refs_check.py not available — doc_refs validation skipped")

    return 0


if __name__ == "__main__":
    sys.exit(main())
