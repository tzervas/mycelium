#!/usr/bin/env python3
"""Idempotent, cross-platform GitHub PM reconcile for Mycelium, driven by the manifests + `gh`.

Runs identically on Linux/macOS and on **Windows (PowerShell)** — it is pure Python plus the
`gh` CLI, with **no bash and no jq**. Invoke it the same way in any shell:

    python tools/github/gh-issues-sync.py            # issues: create absent + update drifted
    python tools/github/gh-issues-sync.py --all      # labels + milestones + issues
    python tools/github/gh-issues-sync.py --all --dry-run   # preview the whole reconcile
    python tools/github/gh-issues-sync.py --update-bodies   # also push issues.yaml bodies
    python tools/github/gh-issues-sync.py --self-test       # offline check of the diff logic

It reconciles the repo to the committed source-of-truth manifests, idempotently and
**never-silently**:

  * ``labels.json``     -> create-or-update each label (color/description)         [--labels]
  * ``milestones.json`` -> create each absent milestone (by title)                 [--milestones]
  * ``issues.yaml``     -> create each absent issue, AND intelligently **update** an
                           existing one to match the manifest: labels, milestone, title
                           (and body only with --update-bodies).                   [--issues]
  * ``idmap.tsv``       -> append any new task_id -> number -> db_id rows (append-only).

Honest by construction (the house rules in CLAUDE.md):

  * **never-silent (G2)** — every create/update is printed; ``--dry-run`` previews, writing nothing.
  * **never-destructive by default** — issue *bodies* change only with ``--update-bodies`` (GitHub
    bodies accrue enactment notes a manifest would clobber); **OPEN/CLOSED state is NEVER inferred**
    from a ``status:*`` label — it changes only if an ``issues.yaml`` entry carries an explicit
    ``state: open|closed``. We never remove a milestone, only set/replace one.
  * **idempotent** — re-running when already in sync performs no writes ("= in sync").
  * **robust matching** — an issue is matched by its ``idmap.tsv`` number first (so renaming a
    title in ``issues.yaml`` *updates the title* instead of creating a duplicate), then by title,
    then created.

This is the cross-platform superset of the split bash tools (``gh-bootstrap-local.sh`` for
labels/milestones + this script for issues). The bash flow still works unchanged; on Windows use
``gh-sync-all.ps1`` (which calls this with ``--all``) so no bash/jq is needed.

Auth: uses `gh` for everything; no token is read or written here. (`gh` resolves `gh.exe` on
Windows via PATHEXT — no shell is spawned.)
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


# ─────────────────────────────────────────────────────────────────────────────────────────────
# gh plumbing (cross-platform: subprocess with a list argv, never shell=True)
# ─────────────────────────────────────────────────────────────────────────────────────────────
def gh(args, *, input_text=None, check=True):
    """Run a `gh` subcommand and return its stdout. Raises on non-zero exit when ``check``."""
    try:
        proc = subprocess.run(
            ["gh", *args],
            # check=False: we surface stderr ourselves below, then raise — so a failure is
            # never silent (G2). Passing check=check here would raise inside subprocess.run
            # before that block could print stderr.
            check=False,
            text=True,
            input=input_text,
            capture_output=True,
        )
    except FileNotFoundError:  # pragma: no cover - environment guard
        sys.exit(
            "ERROR: `gh` (GitHub CLI) not found on PATH. Install it and run `gh auth login` "
            "(Windows: `winget install GitHub.cli`)."
        )
    if check and proc.returncode != 0:
        sys.stderr.write(proc.stderr)
        proc.check_returncode()
    return proc.stdout


# ─────────────────────────────────────────────────────────────────────────────────────────────
# Pure reconcile logic (no I/O — exercised offline by --self-test)
# ─────────────────────────────────────────────────────────────────────────────────────────────
def label_delta(desired, actual):
    """Return (to_add, to_remove) as sorted lists so a label set converges to ``desired``."""
    desired, actual = set(desired), set(actual)
    return sorted(desired - actual), sorted(actual - desired)


def normalize_body(text):
    """Normalize a body for comparison: LF line endings, no trailing space, no trailing blanks.

    GitHub may echo a body back with CRLF or a trailing newline; without this, ``--update-bodies``
    would re-push an unchanged body every run (not idempotent). This makes the compare stable.
    """
    text = (text or "").replace("\r\n", "\n").replace("\r", "\n")
    return "\n".join(line.rstrip() for line in text.split("\n")).strip()


def plan_issue_update(entry, live, *, update_bodies):
    """Compute the set of changes to bring ``live`` (a snapshot dict) to match the manifest ``entry``.

    Pure: returns a dict with any of {labels:(add,remove), milestone, title, body, state}. An empty
    dict means "in sync". ``state`` is included ONLY when the entry declares one explicitly.
    """
    changes = {}

    add, remove = label_delta(entry.get("labels") or [], live.get("labels") or [])
    if add or remove:
        changes["labels"] = (add, remove)

    want_ms = entry.get("milestone")
    if want_ms and want_ms != live.get("milestone"):
        changes["milestone"] = want_ms  # we set/replace, never clear

    if entry["title"] != live.get("title"):
        changes["title"] = entry["title"]

    if update_bodies:
        want_body = entry.get("body", "") or ""
        if normalize_body(want_body) != normalize_body(live.get("body")):
            changes["body"] = want_body

    want_state = entry.get(
        "state"
    )  # only honored when explicitly declared in issues.yaml
    if want_state in ("open", "closed") and want_state != live.get("state"):
        changes["state"] = want_state

    return changes


# ─────────────────────────────────────────────────────────────────────────────────────────────
# Snapshots / loaders
# ─────────────────────────────────────────────────────────────────────────────────────────────
def snapshot_issues(repo):
    """Index every issue (PRs excluded) by number and by title, with the fields we reconcile."""
    raw = gh(["api", "--paginate", f"repos/{repo}/issues?state=all&per_page=100"])
    by_number, by_title = {}, {}
    for item in json.loads(raw):
        if "pull_request" in item:  # the REST issues endpoint also lists PRs
            continue
        rec = {
            "number": item["number"],
            "id": item["id"],
            "title": item["title"],
            "body": item.get("body") or "",
            "labels": {lb["name"] for lb in item.get("labels", [])},
            "milestone": (item.get("milestone") or {}).get("title"),
            "state": item.get("state"),
        }
        by_number[item["number"]] = rec
        by_title[item["title"]] = rec
    return by_number, by_title


def load_idmap(idmap_path):
    """Return {task_id: number} from idmap.tsv (comments/blank lines ignored)."""
    mapping = {}
    if idmap_path.exists():
        for line in idmap_path.read_text(encoding="utf-8").splitlines():
            if not line or line.startswith("#"):
                continue
            parts = line.split("\t")
            if len(parts) >= 2 and parts[1].strip().isdigit():
                mapping[parts[0].strip()] = int(parts[1].strip())
    return mapping


# ─────────────────────────────────────────────────────────────────────────────────────────────
# Reconcilers (labels / milestones / issues)
# ─────────────────────────────────────────────────────────────────────────────────────────────
def reconcile_labels(repo, labels_json, dry_run):
    labels = json.loads(labels_json.read_text(encoding="utf-8"))
    print(f">> labels: {len(labels)} declared in {labels_json.name}")
    for lb in labels:
        name, color, desc = lb["name"], lb.get("color", ""), lb.get("description", "")
        if dry_run:
            print(f"   ~ would create-or-update: {name}")
            continue
        # --force makes `gh label create` create-or-update (color + description) — idempotent.
        gh(
            [
                "label",
                "create",
                name,
                "--repo",
                repo,
                "--color",
                color,
                "--description",
                desc,
                "--force",
            ]
        )
        print(f"   • {name}")


def reconcile_milestones(repo, milestones_json, dry_run):
    milestones = json.loads(milestones_json.read_text(encoding="utf-8"))
    existing = {
        m["title"]: m["number"]
        for m in json.loads(
            gh(["api", f"repos/{repo}/milestones?state=all", "--paginate"])
        )
    }
    print(f">> milestones: {len(milestones)} declared in {milestones_json.name}")
    for ms in milestones:
        title = ms["title"]
        if title in existing:
            print(f"   = exists #{existing[title]}: {title}")
            continue
        if dry_run:
            print(f"   + would create: {title}")
            continue
        out = gh(
            [
                "api",
                f"repos/{repo}/milestones",
                "-f",
                f"title={title}",
                "-f",
                f"state={ms.get('state', 'open')}",
                "-f",
                f"description={ms.get('description', '')}",
            ]
        )
        print(f"   + created #{json.loads(out)['number']}: {title}")


def create_issue(repo, entry, dry_run):
    """Create one issue from a manifest entry; return {number, id} (or None on a dry run)."""
    title = entry["title"]
    if dry_run:
        print(f"   + would create: {title}")
        return None
    args = ["issue", "create", "--repo", repo, "--title", title, "--body-file", "-"]
    for label in entry.get("labels") or []:
        args += ["--label", label]
    url = gh(args, input_text=entry.get("body", "") or "").strip()
    number = int(url.rstrip("/").rsplit("/", 1)[-1])
    item = json.loads(gh(["api", f"repos/{repo}/issues/{number}"]))
    print(f"   + created #{number}: {title}")
    if entry.get("milestone"):
        assign_milestone(repo, number, entry["milestone"], dry_run)
    return {"number": number, "id": item["id"]}


def assign_milestone(repo, number, milestone, dry_run):
    if dry_run:
        print(f"     ~ would set milestone: {milestone}")
        return
    try:
        gh(["issue", "edit", str(number), "--repo", repo, "--milestone", milestone])
        print(f"     ~ milestone: {milestone}")
    except subprocess.CalledProcessError:
        print(
            f"     ! milestone absent (run with --milestones or gh-bootstrap-local.sh first): "
            f"{milestone}",
            file=sys.stderr,
        )


def apply_issue_update(repo, number, changes, entry, dry_run):
    """Apply a non-empty ``changes`` plan to issue ``number`` via `gh`, reporting each field."""
    label_args = []
    if "labels" in changes:
        add, remove = changes["labels"]
        if add:
            label_args += ["--add-label", ",".join(add)]
        if remove:
            label_args += ["--remove-label", ",".join(remove)]
    if "title" in changes:
        label_args += ["--title", changes["title"]]
    if "milestone" in changes:
        label_args += ["--milestone", changes["milestone"]]

    summary = []
    if "labels" in changes:
        add, remove = changes["labels"]
        summary.append(
            "labels " + " ".join([f"+{a}" for a in add] + [f"-{r}" for r in remove])
        )
    if "title" in changes:
        summary.append("title")
    if "milestone" in changes:
        summary.append(f"milestone={changes['milestone']}")
    if "body" in changes:
        summary.append("body")
    if "state" in changes:
        summary.append(f"state={changes['state']}")

    if dry_run:
        print(f"   ~ would update #{number}: {', '.join(summary)}")
        return

    if label_args:
        gh(["issue", "edit", str(number), "--repo", repo, *label_args])
    if "body" in changes:
        gh(
            ["issue", "edit", str(number), "--repo", repo, "--body-file", "-"],
            input_text=changes["body"],
        )
    if "state" in changes:
        verb = "close" if changes["state"] == "closed" else "reopen"
        gh(["issue", verb, str(number), "--repo", repo])
    print(f"   ~ updated #{number}: {', '.join(summary)}")


def reconcile_issues(repo, issues, idmap_path, *, do_update, update_bodies, dry_run):
    by_number, by_title = snapshot_issues(repo)
    idmap = load_idmap(idmap_path)
    print(
        f">> issues: {len(issues)} task(s) in issues.yaml; "
        f"{len(by_number)} issue(s) on {repo} "
        f"(update={'on' if do_update else 'off'}, bodies={'on' if update_bodies else 'off'})"
    )

    created, updated, in_sync = 0, 0, 0
    idmap_rows = []
    for entry in issues:
        tid = entry["id"]
        # Match by idmap number first (rename-safe), then by title.
        live = by_number.get(idmap.get(tid)) or by_title.get(entry["title"])

        if live is None:
            made = create_issue(repo, entry, dry_run)
            if made is None:  # dry run
                continue
            created += 1
            idmap_rows.append((tid, made["number"], made["id"]))
            continue

        idmap_rows.append((tid, live["number"], live["id"]))
        if not do_update:
            continue
        changes = plan_issue_update(entry, live, update_bodies=update_bodies)
        if changes:
            apply_issue_update(repo, live["number"], changes, entry, dry_run)
            updated += 1
        else:
            in_sync += 1

    if not dry_run:
        append_idmap(idmap_path, idmap_rows)
    print(
        f">> issues done — {created} created, {updated} updated, {in_sync} already in sync"
    )


def append_idmap(idmap_path, rows):
    """Append task rows whose id is not already recorded; never rewrite existing (append-only)."""
    known = set()
    if idmap_path.exists():
        for line in idmap_path.read_text(encoding="utf-8").splitlines():
            if line and not line.startswith("#"):
                known.add(line.split("\t", 1)[0])
    fresh = [(tid, num, db) for (tid, num, db) in rows if tid not in known]
    if not fresh:
        return
    with idmap_path.open("a", encoding="utf-8") as handle:
        handle.write(f"# appended by gh-issues-sync.py ({len(fresh)} new)\n")
        for tid, num, db in fresh:
            handle.write(f"{tid}\t{num}\t{db}\n")
    print(f">> idmap.tsv: appended {len(fresh)} row(s)")


# ─────────────────────────────────────────────────────────────────────────────────────────────
# Offline self-test of the pure logic (no gh, runnable anywhere)
# ─────────────────────────────────────────────────────────────────────────────────────────────
def self_test():
    # label_delta converges a set, order-independent.
    assert label_delta(["a", "b"], ["b", "c"]) == (["a"], ["c"])
    assert label_delta(["x"], ["x"]) == ([], [])

    # body normalization makes CRLF / trailing-blank differences a no-op.
    assert normalize_body("a\r\nb\n\n") == normalize_body("a\nb")

    base_live = {
        "title": "T",
        "labels": {"status:needs-design", "phase:8"},
        "milestone": "Phase 8 — Toolchain & Release Engineering",
        "body": "hello\n",
        "state": "open",
    }
    entry = {
        "id": "M-364",
        "title": "T",
        "labels": ["phase:8", "status:done"],
        "milestone": "Phase 8 — Toolchain & Release Engineering",
        "body": "hello",
    }
    # The status flip is detected as a label delta; nothing else drifts; bodies off by default.
    plan = plan_issue_update(entry, base_live, update_bodies=False)
    assert plan == {"labels": (["status:done"], ["status:needs-design"])}, plan

    # In-sync entry yields an empty plan (idempotent).
    synced = dict(base_live, labels={"phase:8", "status:done"})
    assert plan_issue_update(entry, synced, update_bodies=True) == {}, (
        "should be in sync"
    )

    # Title + milestone drift detected; state only when explicitly declared.
    drift_live = dict(
        base_live, title="OLD", milestone=None, labels={"phase:8", "status:done"}
    )
    plan2 = plan_issue_update(entry, drift_live, update_bodies=False)
    assert plan2 == {
        "title": "T",
        "milestone": "Phase 8 — Toolchain & Release Engineering",
    }, plan2
    assert "state" not in plan_issue_update(
        entry, dict(synced, state="closed"), update_bodies=False
    )
    assert plan_issue_update(
        dict(entry, state="open"), dict(synced, state="closed"), update_bodies=False
    ) == {"state": "open"}

    print("self-test OK: label_delta, normalize_body, plan_issue_update")


# ─────────────────────────────────────────────────────────────────────────────────────────────
# CLI
# ─────────────────────────────────────────────────────────────────────────────────────────────
def main():
    parser = argparse.ArgumentParser(
        description="Idempotent, cross-platform gh PM reconcile (labels + milestones + issues)."
    )
    parser.add_argument("--repo", default="tzervas/mycelium")
    parser.add_argument("--issues-yaml", type=Path, default=HERE / "issues.yaml")
    parser.add_argument("--idmap", type=Path, default=HERE / "idmap.tsv")
    parser.add_argument("--labels-json", type=Path, default=HERE / "labels.json")
    parser.add_argument(
        "--milestones-json", type=Path, default=HERE / "milestones.json"
    )
    parser.add_argument(
        "--all", action="store_true", help="labels + milestones + issues"
    )
    parser.add_argument("--labels", action="store_true", help="reconcile labels.json")
    parser.add_argument(
        "--milestones", action="store_true", help="reconcile milestones.json"
    )
    parser.add_argument(
        "--issues", action="store_true", help="reconcile issues.yaml (default)"
    )
    parser.add_argument(
        "--no-update",
        action="store_true",
        help="create absent issues only; do not update existing ones",
    )
    parser.add_argument(
        "--update-bodies",
        action="store_true",
        help="also push issues.yaml bodies (off by default — GitHub bodies accrue notes)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="print what would change without touching the repo",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="run the offline diff-logic check and exit",
    )
    args = parser.parse_args()

    if args.self_test:
        self_test()
        return

    selected = args.all or args.labels or args.milestones or args.issues
    do_labels = args.all or args.labels
    do_milestones = args.all or args.milestones
    do_issues = (
        args.all or args.issues or not selected
    )  # default to issues when nothing asked

    mode = "dry-run" if args.dry_run else "live"
    print("=" * 60)
    print(f">> Mycelium PM reconcile — repo: {args.repo}  ({mode})")
    print("=" * 60)

    if do_labels:
        reconcile_labels(args.repo, args.labels_json, args.dry_run)
        print()
    if do_milestones:
        reconcile_milestones(args.repo, args.milestones_json, args.dry_run)
        print()
    if do_issues:
        spec = yaml.safe_load(args.issues_yaml.read_text(encoding="utf-8"))
        issues = spec.get("issues", []) if spec else []
        reconcile_issues(
            args.repo,
            issues,
            args.idmap,
            do_update=not args.no_update,
            update_bodies=args.update_bodies,
            dry_run=args.dry_run,
        )

    print()
    print(f">> reconcile complete ({mode}).")


if __name__ == "__main__":
    main()
