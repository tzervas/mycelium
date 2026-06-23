#!/usr/bin/env python3
"""Preflight consistency check for the Mycelium GitHub PM manifests.

Verifies — *before* any `gh` call — that every label and milestone referenced
in issues.yaml is actually defined in labels.json / milestones.json. This closes
the silent gap that previously stalled a bootstrap rerun: `gh issue create
--label <name>` errors if the label was never created from labels.json, so a
label used by issues.yaml but absent from the manifest leaves issues uncreated
with no obvious cause. Here that drift is surfaced loudly (G2: never silent) — a
non-fatal WARNING (the label-sync auto-creates such a label with a default colour
until you add it properly), not a half-finished silent sync.

It also reports the reverse (manifest entries unused by any issue) as info, and
cross-validates idmap.tsv against issues.yaml (the id↔number map must be 1:1 — see
check_idmap).

Exit status: 0 = OK — label/milestone gaps and idmap coverage / missing-db-id /
stale-row findings are loud but NON-FATAL warnings; 1 = a BLOCKING error: a
malformed/unreadable manifest, a dangling doc_ref, or idmap corruption (a malformed
row, or a duplicate task-id / issue-number breaking the 1:1 id↔number map).

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


def parse_idmap(idmap_path: Path):
    """Parse idmap.tsv → ``(rows, malformed)``.

    ``rows`` is ``[(task_id, number:int, db_id:str|None, lineno:int)]`` for every well-formed data
    row — a non-blank line that does not start with ``#`` is a data row. ``malformed`` is
    ``[(lineno, rawline, reason)]`` for any data row missing a task-id or an integer issue number.

    Unlike the lenient runtime loader in ``gh-issues-sync.py`` (which silently skips a non-digit
    second column), the preflight is STRICT: a data line that does not parse is surfaced as
    malformed, never silently dropped (G2 — never silent).
    """
    rows, malformed = [], []
    for lineno, line in enumerate(
        idmap_path.read_text(encoding="utf-8").splitlines(), 1
    ):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        parts = line.split("\t")
        tid = parts[0].strip()
        num = parts[1].strip() if len(parts) >= 2 else ""
        db = parts[2].strip() if len(parts) >= 3 else ""
        if not tid or not num.isdigit():
            malformed.append(
                (
                    lineno,
                    line,
                    "expected '<task_id>\\t<number>[\\t<db_id>]' with an integer number",
                )
            )
            continue
        rows.append((tid, int(num), db or None, lineno))
    return rows, malformed


def check_idmap(issues, idmap_path: Path) -> int:
    """Cross-validate idmap.tsv (task_id → number → db_id) against issues.yaml (M-675).

    Returns the ERROR count (0 = clean). The split mirrors the label/milestone checks above:

    * ERRORS (block — corruption that would break a sync): a malformed data row; a task-id mapped
      twice; an issue number mapped to two different task-ids (the id↔number map must be 1:1).
    * WARNINGS (never-silent, non-fatal — normal authoring drift): an issues.yaml id with no idmap
      row (not yet created/synced on GitHub — issues.yaml is authored ahead of the live repo); an
      idmap row carrying a number but no db_id (the sub-issue-link key still needs a live fetch);
      an idmap task-id absent from issues.yaml (a stale or GitHub-only row).

    Honesty (VR-5/G2): a missing db_id is REPORTED, never invented — a number-only row is a legal,
    flagged state, not an error.
    """
    if not idmap_path.exists():
        print(f"  note: {idmap_path.name} not found — idmap cross-check skipped")
        return 0

    rows, malformed = parse_idmap(idmap_path)
    errors = 0
    for lineno, raw, reason in malformed:
        print(
            f"ERROR (idmap): {idmap_path.name}:{lineno} malformed row — {reason}: {raw!r}",
            file=sys.stderr,
        )
        errors += 1

    by_id: dict[str, tuple[int, str | None, int]] = {}
    num_owner: dict[int, tuple[str, int]] = {}
    for tid, num, db, lineno in rows:
        if tid in by_id:
            print(
                f"ERROR (idmap): task-id '{tid}' mapped twice "
                f"(lines {by_id[tid][2]} and {lineno})",
                file=sys.stderr,
            )
            errors += 1
        else:
            by_id[tid] = (num, db, lineno)
        if num in num_owner and num_owner[num][0] != tid:
            print(
                f"ERROR (idmap): issue #{num} mapped to two task-ids "
                f"('{num_owner[num][0]}' line {num_owner[num][1]}, '{tid}' line {lineno}) "
                "— the id↔number map must be 1:1",
                file=sys.stderr,
            )
            errors += 1
        else:
            num_owner.setdefault(num, (tid, lineno))

    yaml_ids = [e["id"] for e in issues]
    yaml_set = set(yaml_ids)

    missing = [i for i in yaml_ids if i not in by_id]
    for i in missing:
        print(
            f"WARNING (idmap): issues.yaml id '{i}' has no idmap row — create + map it "
            "(run gh-issues-sync.py to assign its number/db_id)",
            file=sys.stderr,
        )
    no_db = sorted(tid for tid, (_n, db, _l) in by_id.items() if db is None)
    for tid in no_db:
        print(
            f"WARNING (idmap): '{tid}' (#{by_id[tid][0]}) has no db_id — fetch it live before "
            "sub-issue / dependency linking (the db_id is the sub_issue link key; never guessed — G2)",
            file=sys.stderr,
        )
    stale = sorted(tid for tid in by_id if tid not in yaml_set)
    for tid in stale:
        print(
            f"WARNING (idmap): idmap task-id '{tid}' (#{by_id[tid][0]}) is not in issues.yaml "
            "— a stale or GitHub-only row",
            file=sys.stderr,
        )

    warns = len(missing) + len(no_db) + len(stale)
    if errors:
        print(
            f">> idmap check FAILED: {errors} error(s) — fix before syncing.",
            file=sys.stderr,
        )
    else:
        with_db = sum(1 for _t, (_n, db, _l) in by_id.items() if db)
        covered = len(yaml_set) - len(missing)
        suffix = f" ({warns} warning(s) — see above)" if warns else ""
        print(
            f">> idmap check OK: {len(by_id)} id(s) mapped ({with_db} with db_id); "
            f"{covered}/{len(yaml_set)} issues.yaml id(s) covered{suffix}."
        )
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate PM manifest consistency.")
    parser.add_argument("--issues-yaml", type=Path, default=HERE / "issues.yaml")
    parser.add_argument("--labels", type=Path, default=HERE / "labels.json")
    parser.add_argument("--milestones", type=Path, default=HERE / "milestones.json")
    parser.add_argument("--idmap", type=Path, default=HERE / "idmap.tsv")
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

    # idmap cross-validation (M-675; additive — id↔number↔db_id consistency vs issues.yaml).
    # Errors here block (corruption); warnings are never-silent but non-fatal.
    rc = 1 if check_idmap(issues, args.idmap) else 0

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

    return rc


if __name__ == "__main__":
    sys.exit(main())
