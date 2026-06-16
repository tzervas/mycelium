#!/usr/bin/env python3
"""Idempotent GitHub issue sync for Mycelium, driven by issues.yaml + the gh CLI.

This is the local/mobile analogue of tools/github/mcp-bootstrap.md Steps 1-2
(which a model runs over the GitHub MCP server). It fills the one gap that
gh-bootstrap-local.sh leaves — issue *creation* — so the whole PM bootstrap can
run from a phone (Termux) with only `gh` authenticated, no MCP session.

What it does, idempotently:
  1. Snapshot every existing issue on the repo (by title) — never duplicate.
  2. Create only the issues whose title is ABSENT, with their labels.
  3. Assign each newly-created issue's milestone (by title; the milestone must
     already exist — run gh-bootstrap-local.sh first).
  4. Append any new task_id -> number -> db_id rows to idmap.tsv (non-destructive
     — existing rows and the curated comments are left intact).

Auth: uses `gh` for everything; no token is read or written here.

Dependency / sub-issue linking (mcp-bootstrap.md Step 4, the "Grok pass") is NOT
done here — it needs the GraphQL dependencies API. This script stops at
create + milestone + idmap, matching gh-bootstrap-local.sh's scope.

Usage:
  python3 tools/github/gh-issues-sync.py
  python3 tools/github/gh-issues-sync.py --repo tzervas/mycelium --dry-run
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

try:
    import yaml
except ImportError:  # pragma: no cover - environment guard
    sys.exit(
        "PyYAML is required: `pip install pyyaml` "
        "(the termux-setup.sh orchestrator installs it for you)."
    )

HERE = Path(__file__).resolve().parent


def gh(args, *, input_text=None):
    """Run a `gh` subcommand and return its stdout. Raises on non-zero exit."""
    return subprocess.run(
        ["gh", *args],
        check=True,
        text=True,
        input=input_text,
        capture_output=True,
    ).stdout


def existing_issues(repo):
    """Map title -> {number, id} for every issue (PRs excluded) on the repo."""
    raw = gh(["api", "--paginate", f"repos/{repo}/issues?state=all&per_page=100"])
    snapshot = {}
    for item in json.loads(raw):
        if "pull_request" in item:  # the REST issues endpoint also lists PRs
            continue
        snapshot[item["title"]] = {"number": item["number"], "id": item["id"]}
    return snapshot


def create_issue(repo, title, body, labels, dry_run):
    """Create one issue; return {number, id} (or None on a dry run)."""
    if dry_run:
        print(f"  + would create: {title}")
        return None
    args = ["issue", "create", "--repo", repo, "--title", title, "--body-file", "-"]
    for label in labels:
        args += ["--label", label]
    url = gh(args, input_text=body or "").strip()
    number = int(url.rstrip("/").rsplit("/", 1)[-1])
    item = json.loads(gh(["api", f"repos/{repo}/issues/{number}"]))
    print(f"  + created #{number}: {title}")
    return {"number": number, "id": item["id"]}


def assign_milestone(repo, number, milestone, dry_run):
    """Set an issue's milestone by title (no-op if the issue has no milestone)."""
    if not milestone:
        return
    if dry_run:
        print(f"    ~ would set milestone: {milestone}")
        return
    try:
        gh(["issue", "edit", str(number), "--repo", repo, "--milestone", milestone])
        print(f"    ~ milestone: {milestone}")
    except subprocess.CalledProcessError:
        print(
            f"    ! milestone absent (run gh-bootstrap-local.sh first): {milestone}",
            file=sys.stderr,
        )


def append_idmap(idmap_path, rows):
    """Append task rows whose id is not already recorded; never rewrite existing."""
    known = set()
    if idmap_path.exists():
        for line in idmap_path.read_text().splitlines():
            if line and not line.startswith("#"):
                known.add(line.split("\t", 1)[0])
    fresh = [(tid, num, db) for (tid, num, db) in rows if tid not in known]
    if not fresh:
        return
    with idmap_path.open("a") as handle:
        handle.write(f"# appended by gh-issues-sync.py ({len(fresh)} new)\n")
        for tid, num, db in fresh:
            handle.write(f"{tid}\t{num}\t{db}\n")
    print(f">> idmap.tsv: appended {len(fresh)} row(s)")


def main():
    parser = argparse.ArgumentParser(description="Idempotent gh issue sync.")
    parser.add_argument("--repo", default="tzervas/mycelium")
    parser.add_argument("--issues-yaml", type=Path, default=HERE / "issues.yaml")
    parser.add_argument("--idmap", type=Path, default=HERE / "idmap.tsv")
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="print what would be created without touching the repo",
    )
    args = parser.parse_args()

    spec = yaml.safe_load(args.issues_yaml.read_text())
    issues = spec.get("issues", []) if spec else []
    snapshot = existing_issues(args.repo)
    print(
        f">> {len(issues)} task(s) in {args.issues_yaml.name}; "
        f"{len(snapshot)} issue(s) already on {args.repo}"
    )

    idmap_rows = []
    created = 0
    for entry in issues:
        title = entry["title"]
        found = snapshot.get(title)
        if found is None:
            found = create_issue(
                args.repo,
                title,
                entry.get("body", ""),
                entry.get("labels") or [],
                args.dry_run,
            )
            if found is None:  # dry run — nothing to record
                continue
            created += 1
            assign_milestone(
                args.repo, found["number"], entry.get("milestone"), args.dry_run
            )
        idmap_rows.append((entry["id"], found["number"], found["id"]))

    if not args.dry_run:
        append_idmap(args.idmap, idmap_rows)
    mode = "dry-run" if args.dry_run else "live"
    print(f">> done — {created} issue(s) created ({mode})")


if __name__ == "__main__":
    main()
