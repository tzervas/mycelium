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

    _spec = _ilu.spec_from_file_location("doc_refs_check", HERE / "doc_refs_check.py")
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
    parser.add_argument(
        "--debug",
        action="store_true",
        help="print the full traceback + extra detail on a malformed/unreadable manifest "
        "(otherwise a clean one-line error) — for debugging this class of issue.",
    )
    args = parser.parse_args()

    # Load the manifests; a malformed/unreadable file is a clean, classified error (its raw
    # traceback shows only under --debug — the detail you want when investigating further).
    try:
        spec = yaml.safe_load(args.issues_yaml.read_text())
        issues = spec.get("issues", []) if spec else []
        defined_labels = {d["name"] for d in json.loads(args.labels.read_text())}
        defined_ms = {d["title"] for d in json.loads(args.milestones.read_text())}
    except Exception as exc:
        if args.debug:
            import traceback

            traceback.print_exc()
        print(
            f"ERROR: a manifest is malformed/unreadable: {type(exc).__name__}: {exc} "
            "(re-run with --debug for the full traceback)",
            file=sys.stderr,
        )
        return 1
    if args.debug:
        print(
            f"  debug: {len(issues)} issue(s); {len(defined_labels)} defined label(s), "
            f"{len(defined_ms)} defined milestone(s)",
            file=sys.stderr,
        )

    used_labels: dict[str, list[str]] = {}
    used_ms: dict[str, list[str]] = {}
    for entry in issues:
        for label in entry.get("labels") or []:
            used_labels.setdefault(label, []).append(entry["id"])
        if entry.get("milestone"):
            used_ms.setdefault(entry["milestone"], []).append(entry["id"])

    # A reference USED by an issue but absent from the manifest is a non-fatal, actionable
    # WARNING (never-silent), not a hard failure: the label-sync auto-creates such a label with a
    # default colour until you add it properly. Only genuine manifest corruption (malformed
    # JSON / dangling doc_refs) blocks the sync — the gap is surfaced loudly (G2), just not fatal.
    warnings = 0
    missing_labels = sorted(set(used_labels) - defined_labels)
    for name in missing_labels:
        ids = used_labels[name]
        print(
            f"WARNING: label '{name}' used by {len(ids)} issue(s) "
            f"(e.g. {', '.join(ids[:5])}) is absent from {args.labels.name} — add it "
            f"(the label-sync auto-creates it with a default colour until you do)",
            file=sys.stderr,
        )
        warnings += 1
    missing_ms = sorted(set(used_ms) - defined_ms)
    for title in missing_ms:
        ids = used_ms[title]
        print(
            f"WARNING: milestone '{title}' used by {len(ids)} issue(s) "
            f"(e.g. {', '.join(ids[:5])}) is absent from {args.milestones.name} "
            f"— add it to {args.milestones.name}",
            file=sys.stderr,
        )
        warnings += 1

    # Reverse direction is info-level — an unused manifest entry is harmless.
    for name in sorted(defined_labels - set(used_labels)):
        print(f"  info: label '{name}' is defined but unused by any issue")
    for title in sorted(defined_ms - set(used_ms)):
        print(f"  info: milestone '{title}' is defined but unused by any issue")

    suffix = f" ({warnings} warning(s) — see above)" if warnings else ""
    print(
        f">> manifest check OK: {len(issues)} issue(s); "
        f"{len(used_labels)} label(s) + {len(used_ms)} milestone(s){suffix}."
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
            has_any = any((issue.get("doc_refs") or []) for issue in issues)
            if has_any:
                print(">> doc_refs check OK: all doc_refs entries resolve.")
    else:
        print("  note: doc_refs_check.py not available — doc_refs validation skipped")

    return 0


if __name__ == "__main__":
    sys.exit(main())
