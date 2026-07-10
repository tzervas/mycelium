#!/usr/bin/env python3
"""Rate-limit-frugal, diff-based issue sync for Mycelium — one bulk read, only-the-deltas writes.

    python tools/github/sync_issues.py                 # DRY-RUN plan (default; ZERO writes)
    python tools/github/sync_issues.py --refresh        # force a fresh bulk snapshot, then plan
    python tools/github/sync_issues.py --apply          # perform only the delta writes
    python tools/github/sync_issues.py --apply --max-writes 5   # cap the mutations (safety)
    python tools/github/sync_issues.py --self-test      # offline check of the pure diff logic

WHY THIS EXISTS
---------------
`issues.yaml` (544 tasks) is the source of truth; the repo's GitHub issues must track it. The
naive bootstrap (`mcp-bootstrap.md`) touches the API **per issue** — a `list_issues`/`issue_write`
round-trip each, ~544×, whether or not an issue actually drifted. That is wasteful of the 5000/hr
core rate limit.

This tool instead does the whole reconcile with **one bulk read and only-the-deltas writes**:

  1. **One bulk snapshot** — `gh api --paginate repos/<repo>/issues?state=all&per_page=100`
     (~6 paginated calls for 544 issues, NOT 544). The snapshot is **cached** to a gitignored
     `.gh-snapshot.json`, so repeated dry-runs and offline diff development re-fetch **nothing**
     (`--refresh` forces a re-fetch; `--max-cache-age` bounds staleness before an auto-refetch).
  2. **Local diff (pure, no I/O)** — for each `issues.yaml` entry, resolve its GitHub number via
     `idmap.tsv` (rename-safe: number first, then title), compute the *desired* GitHub
     representation (title, labels, milestone, body?, state?) and compare field-by-field against
     the snapshot. Each issue is classified **in-sync** (skip, ZERO API calls), **needs-update**
     (only the drifted fields), **missing** (no live issue → create), or **orphan** (on GitHub,
     not in `issues.yaml` → reported, never auto-deleted).
  3. **Apply only deltas** — `gh issue create` for missing (new number appended to `idmap.tsv`),
     `gh issue edit`/`close`/`reopen` for the specific changed fields of needs-update. In-sync
     issues cost nothing. A running tally reports writes made vs. the naive per-issue baseline.
  4. **Never-silent + dry-run-by-default (G2)** — the default run prints the full plan and makes
     **zero** mutating calls; an explicit `--apply` is required to write. Every create/update is
     printed with the exact fields.
  5. **Rate-limit aware** — probes `gh api rate_limit` before writing and **stops** (never-silent)
     if the remaining core budget is below a floor; `--max-writes N` is an independent hard cap.
  6. **Idempotent** — a second run with no `issues.yaml` change classifies everything in-sync and
     writes nothing.

RELATIONSHIP TO gh-issues-sync.py (READ THIS)
---------------------------------------------
`gh-issues-sync.py` is the full cross-platform reconcile **engine** (labels + milestones + issues
+ PRs + Project v2 board) and **already** does a bulk `snapshot_issues()` + field diff for the
issue level — it is NOT a per-issue fan-out. This tool is the **focused, cached, issue-only,
dry-run-by-default** path meant for **periodic** desktop use, and it adds what the engine lacks: a
persisted snapshot cache, a dry-run-first default, a `--max-writes` cap, explicit orphan reporting,
and an API-call tally. The diff **semantics are deliberately identical** to the engine's
(idmap-number-first matching; labels converge as a full set incl. `status:*`; OPEN/CLOSED state is
changed **only** when an entry carries an explicit `state:` — never inferred from a `status:*`
label; bodies compared only under `--update-bodies` and normalized before compare). See README.md.

Honest tags: the diff is **Exact** over the cached snapshot (a pure set/string comparison). The
*snapshot* is only as fresh as the last fetch (**Empirical** — bounded by `--max-cache-age`;
`--refresh` re-grounds it). Orphan classification is **Declared** (we report, we never delete).

Requires: `gh` (authenticated to the repo owner) on PATH, and PyYAML (the `issues.yaml` parser).
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from pathlib import Path

try:
    import yaml
except ImportError:  # pragma: no cover - environment guard
    sys.exit(
        "PyYAML is required: `uv sync --project tools/github` (or `pip install pyyaml`)."
    )

HERE = Path(__file__).resolve().parent
DEFAULT_REPO = "tzervas/mycelium"
ISSUES_YAML = HERE / "issues.yaml"
IDMAP_TSV = HERE / "idmap.tsv"
SNAPSHOT_CACHE = HERE / ".gh-snapshot.json"

# Stop writing when the remaining core rate-limit budget drops below this floor (never-silent).
RATE_FLOOR = 50
# Auto-refetch the snapshot cache when it is older than this many seconds (24h default).
DEFAULT_MAX_CACHE_AGE = 24 * 60 * 60


# ─────────────────────────────────────────────────────────────────────────────────────────────
# PURE diff logic (no I/O) — exercised offline by --self-test
# ─────────────────────────────────────────────────────────────────────────────────────────────
def normalize_body(text):
    """Normalize a body for comparison: LF endings, no trailing spaces, no trailing blank lines.

    GitHub may echo a body back with CRLF or a trailing newline; without this, a body compare would
    report spurious drift every run. (Same normalization as gh-issues-sync.py, kept identical.)
    """
    text = (text or "").replace("\r\n", "\n").replace("\r", "\n")
    return "\n".join(line.rstrip() for line in text.split("\n")).strip()


def label_delta(desired, actual):
    """Return (to_add, to_remove) sorted, so the actual label set converges to ``desired``."""
    desired, actual = set(desired), set(actual)
    return sorted(desired - actual), sorted(actual - desired)


def desired_from_entry(entry):
    """PURE: the GitHub representation an ``issues.yaml`` entry *wants* (the compare target)."""
    return {
        "title": entry["title"],
        "labels": list(entry.get("labels") or []),
        "milestone": entry.get("milestone"),
        "body": entry.get("body", "") or "",
        # Honored ONLY when explicitly declared — never inferred from a status:* label.
        "state": entry.get("state"),
    }


def diff_entry(entry, live, *, update_bodies):
    """PURE: the changes to bring ``live`` (a snapshot record) to match ``entry``.

    Returns a dict with any of {labels:(add,remove), milestone, title, body, state}; empty ⇒
    in-sync. Semantics identical to gh-issues-sync.py:plan_issue_update.
    """
    want = desired_from_entry(entry)
    changes = {}

    add, remove = label_delta(want["labels"], live.get("labels") or [])
    if add or remove:
        changes["labels"] = (add, remove)

    if want["milestone"] and want["milestone"] != live.get("milestone"):
        changes["milestone"] = want["milestone"]  # set/replace, never clear

    if want["title"] != live.get("title"):
        changes["title"] = want["title"]

    if update_bodies and normalize_body(want["body"]) != normalize_body(
        live.get("body")
    ):
        changes["body"] = want["body"]

    if want["state"] in ("open", "closed") and want["state"] != live.get("state"):
        changes["state"] = want["state"]

    return changes


def classify(issues, idmap, by_number, by_title, *, update_bodies):
    """PURE: partition entries into create / update / in-sync, plus orphan detection.

    Returns (to_create, to_update, in_sync, matched_numbers) where
      * to_create — [entry] with no live match (needs `gh issue create`);
      * to_update — [(entry, live, changes)] whose live issue drifts;
      * in_sync   — [entry] matched and identical (ZERO API cost);
      * matched_numbers — set of live issue numbers claimed by an entry (for orphan detection).
    Matching is idmap-number-first (rename-safe), then title.
    """
    to_create, to_update, in_sync, matched = [], [], [], set()
    for entry in issues:
        live = by_number.get(idmap.get(entry["id"])) or by_title.get(entry["title"])
        if live is None:
            to_create.append(entry)
            continue
        matched.add(live["number"])
        changes = diff_entry(entry, live, update_bodies=update_bodies)
        (to_update.append((entry, live, changes)) if changes else in_sync.append(entry))
    return to_create, to_update, in_sync, matched


def render_changes(changes):
    """PURE: a compact, never-silent one-line summary of a changes plan."""
    parts = []
    if "labels" in changes:
        add, remove = changes["labels"]
        parts.append(
            "labels " + " ".join([f"+{a}" for a in add] + [f"-{r}" for r in remove])
        )
    if "title" in changes:
        parts.append("title")
    if "milestone" in changes:
        parts.append(f"milestone={changes['milestone']}")
    if "body" in changes:
        parts.append("body")
    if "state" in changes:
        parts.append(f"state={changes['state']}")
    return ", ".join(parts)


def edit_args(changes):
    """PURE: render the `gh issue edit` args for a changes plan (state handled separately)."""
    args = []
    if "labels" in changes:
        add, remove = changes["labels"]
        if add:
            args += ["--add-label", ",".join(add)]
        if remove:
            args += ["--remove-label", ",".join(remove)]
    if "title" in changes:
        args += ["--title", changes["title"]]
    if "milestone" in changes:
        args += ["--milestone", changes["milestone"]]
    return args


# ─────────────────────────────────────────────────────────────────────────────────────────────
# I/O — loaders, the gh CLI, the cached bulk snapshot
# ─────────────────────────────────────────────────────────────────────────────────────────────
def load_issues(path):
    data = yaml.safe_load(path.read_text(encoding="utf-8"))
    issues = data["issues"] if isinstance(data, dict) else data
    ids = [e["id"] for e in issues]
    if len(set(ids)) != len(ids):
        dupes = sorted({i for i in ids if ids.count(i) > 1})
        sys.exit(f"issues.yaml: duplicate id(s): {', '.join(dupes)}")
    return issues


def load_idmap(path):
    """Return {task_id: number} from idmap.tsv (comments/blank lines ignored)."""
    mapping = {}
    if path.exists():
        for line in path.read_text(encoding="utf-8").splitlines():
            if not line or line.startswith("#"):
                continue
            parts = line.split("\t")
            if len(parts) >= 2 and parts[1].strip().isdigit():
                mapping[parts[0].strip()] = int(parts[1].strip())
    return mapping


class ApiCounter:
    """Tally read vs write `gh` calls so the run can report frugality (never-silent)."""

    def __init__(self):
        self.reads = 0
        self.writes = 0

    def read(self, n=1):
        self.reads += n

    def write(self, n=1):
        self.writes += n


def gh(args, *, input_text=None, counter=None, kind="read"):
    """Run a `gh` command, returning stdout. Raises on failure (never-silent)."""
    if counter is not None:
        counter.write() if kind == "write" else counter.read()
    proc = subprocess.run(
        ["gh", *args],
        input=input_text,
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        raise RuntimeError(
            f"gh {' '.join(args[:3])}… failed (rc={proc.returncode}): "
            f"{(proc.stderr or '').strip()[:400]}"
        )
    return proc.stdout


def core_remaining(counter=None):
    """Return the remaining core rate-limit budget (int), or None if it can't be read."""
    try:
        raw = gh(["api", "rate_limit"], counter=counter, kind="read")
    except RuntimeError:
        return None
    blob = json.loads(raw)
    core = (blob.get("resources") or {}).get("core") or blob.get("rate") or {}
    rem = core.get("remaining")
    return rem if isinstance(rem, int) else None


def fetch_snapshot(repo, counter):
    """ONE bulk paginated read of every issue (PRs excluded). ~ceil(total/100) calls, not per-issue."""
    raw = gh(
        ["api", "--paginate", f"repos/{repo}/issues?state=all&per_page=100"],
        counter=counter,
        kind="read",
    )
    # `gh api --paginate` concatenates pages as separate JSON arrays only with --slurp; without it,
    # it emits one merged array. Parse defensively: try one array, else concatenate arrays.
    try:
        items = json.loads(raw)
        if isinstance(items, dict):
            items = [items]
    except json.JSONDecodeError:
        items = []
        for chunk in raw.replace("][", "]\x00[").split("\x00"):
            items.extend(json.loads(chunk))
    records = []
    for item in items:
        if "pull_request" in item:  # the REST issues endpoint also lists PRs
            continue
        records.append(
            {
                "number": item["number"],
                "id": item["id"],
                "title": item["title"],
                "body": item.get("body") or "",
                "labels": sorted(lb["name"] for lb in item.get("labels", [])),
                "milestone": (item.get("milestone") or {}).get("title"),
                "state": item.get("state"),
            }
        )
    return records


def load_snapshot(repo, counter, *, refresh, max_cache_age):
    """Return (records, source) using the cache when fresh; fetch+cache otherwise (frugal)."""
    if not refresh and SNAPSHOT_CACHE.exists():
        blob = json.loads(SNAPSHOT_CACHE.read_text(encoding="utf-8"))
        age = time.time() - blob.get("fetched_at", 0)
        if blob.get("repo") == repo and age <= max_cache_age:
            return blob["records"], f"cache ({int(age)}s old, 0 API calls)"
    records = fetch_snapshot(repo, counter)
    SNAPSHOT_CACHE.write_text(
        json.dumps(
            {"repo": repo, "fetched_at": time.time(), "records": records}, indent=1
        ),
        encoding="utf-8",
    )
    return records, f"fresh fetch ({counter.reads} paginated read call(s))"


def index(records):
    by_number = {r["number"]: r for r in records}
    by_title = {r["title"]: r for r in records}
    return by_number, by_title


def append_idmap(path, rows):
    """Append task rows whose id is not already recorded; never rewrite existing (append-only)."""
    known = set()
    if path.exists():
        for line in path.read_text(encoding="utf-8").splitlines():
            if line and not line.startswith("#"):
                known.add(line.split("\t", 1)[0])
    fresh = [(tid, num, db) for (tid, num, db) in rows if tid not in known]
    if not fresh:
        return 0
    with path.open("a", encoding="utf-8") as fh:
        fh.write(f"# appended by sync_issues.py ({len(fresh)} new)\n")
        for tid, num, db in fresh:
            fh.write(f"{tid}\t{num}\t{db}\n")
    return len(fresh)


# ─────────────────────────────────────────────────────────────────────────────────────────────
# Apply — only the deltas
# ─────────────────────────────────────────────────────────────────────────────────────────────
def apply_create(repo, entry, counter):
    """Create one issue; return (number, db_id). Milestone set in the same create when present."""
    args = [
        "issue",
        "create",
        "--repo",
        repo,
        "--title",
        entry["title"],
        "--body-file",
        "-",
    ]
    for label in entry.get("labels") or []:
        args += ["--label", label]
    if entry.get("milestone"):
        args += ["--milestone", entry["milestone"]]
    url = gh(
        args, input_text=entry.get("body", "") or "", counter=counter, kind="write"
    ).strip()
    number = int(url.rstrip("/").rsplit("/", 1)[-1])
    node = json.loads(
        gh(["api", f"repos/{repo}/issues/{number}"], counter=counter, kind="read")
    )
    print(f"   + created #{number}: {entry['title']}")
    return number, node["id"]


def apply_update(repo, number, changes, counter):
    """Apply only the changed fields of one issue via `gh issue edit`/`close`/`reopen`."""
    args = edit_args(changes)
    if "body" in changes:
        gh(
            ["issue", "edit", str(number), "--repo", repo, *args, "--body-file", "-"],
            input_text=changes["body"],
            counter=counter,
            kind="write",
        )
    elif args:
        gh(
            ["issue", "edit", str(number), "--repo", repo, *args],
            counter=counter,
            kind="write",
        )
    if "state" in changes:
        verb = "close" if changes["state"] == "closed" else "reopen"
        gh(["issue", verb, str(number), "--repo", repo], counter=counter, kind="write")
    print(f"   ~ updated #{number}: {render_changes(changes)}")


# ─────────────────────────────────────────────────────────────────────────────────────────────
# Plan + drive
# ─────────────────────────────────────────────────────────────────────────────────────────────
def print_plan(to_create, to_update, in_sync, orphans, *, verbose):
    print("\n== PLAN ==")
    print(f"   in-sync  : {len(in_sync):4d}  (skipped — zero API calls)")
    print(f"   update   : {len(to_update):4d}  (only drifted fields)")
    print(f"   create   : {len(to_create):4d}  (missing on GitHub)")
    print(
        f"   orphan   : {len(orphans):4d}  (on GitHub, not in issues.yaml — reported, not deleted)"
    )
    if to_create:
        print("\n   -- to create --")
        for e in to_create:
            print(f"   + {e['id']}: {e['title']}")
    if to_update:
        print("\n   -- to update --")
        for e, live, changes in to_update:
            print(f"   ~ #{live['number']} {e['id']}: {render_changes(changes)}")
    if orphans and verbose:
        print("\n   -- orphans (FLAG) --")
        for r in orphans:
            print(f"   ? #{r['number']} [{r['state']}]: {r['title']}")


def run(args):
    repo = args.repo
    counter = ApiCounter()
    issues = load_issues(ISSUES_YAML)
    idmap = load_idmap(IDMAP_TSV)

    records, source = load_snapshot(
        repo, counter, refresh=args.refresh, max_cache_age=args.max_cache_age
    )
    by_number, by_title = index(records)
    print(f">> snapshot: {len(records)} live issue(s) via {source}")
    print(
        f">> source of truth: {len(issues)} task(s) in issues.yaml; {len(idmap)} idmap row(s)"
    )

    to_create, to_update, in_sync, matched = classify(
        issues, idmap, by_number, by_title, update_bodies=args.update_bodies
    )
    orphans = [r for n, r in by_number.items() if n not in matched]

    print_plan(to_create, to_update, in_sync, orphans, verbose=args.verbose)

    naive = len(issues)  # the per-issue baseline: one write round-trip per task
    planned_writes = len(to_create) + len(to_update)
    print(
        f"\n>> frugality: naive per-issue baseline ≈ {naive} write round-trips; "
        f"this plan needs {planned_writes} write(s) + {counter.reads} bulk "
        f"`gh --paginate` read(s) (≈{-(-len(records) // 100)} HTTP page(s)) "
        f"→ {naive - planned_writes} write(s) saved."
    )

    if not args.apply:
        print(
            "\n>> DRY-RUN (default): no mutations made. Re-run with --apply to write the deltas."
        )
        return 0

    if planned_writes == 0:
        print("\n>> --apply: nothing to do — already in sync.")
        return 0

    # Rate-limit gate before any write (never-silent).
    remaining = core_remaining(counter)
    if remaining is not None and remaining < RATE_FLOOR:
        sys.exit(
            f"\n!! core rate-limit budget low ({remaining} < {RATE_FLOOR}); refusing to write. "
            "Wait for the reset and re-run."
        )
    print(
        f"\n>> --apply: core rate budget = {remaining}; cap = {args.max_writes} write(s)"
    )

    writes_done = 0
    idmap_rows = []
    for entry in to_create:
        if writes_done >= args.max_writes:
            print(
                f"   … --max-writes {args.max_writes} reached; stopping (re-run to continue)."
            )
            break
        number, db_id = apply_create(repo, entry, counter)
        idmap_rows.append((entry["id"], number, db_id))
        writes_done += 1
    for entry, live, changes in to_update:
        if writes_done >= args.max_writes:
            print(
                f"   … --max-writes {args.max_writes} reached; stopping (re-run to continue)."
            )
            break
        apply_update(repo, live["number"], changes, counter)
        writes_done += 1

    appended = append_idmap(IDMAP_TSV, idmap_rows)
    if appended:
        print(f">> idmap.tsv: appended {appended} new row(s)")
    # The snapshot is now stale (we mutated) — drop the cache so the next run re-grounds.
    if SNAPSHOT_CACHE.exists():
        SNAPSHOT_CACHE.unlink()
    print(
        f"\n>> done: {writes_done} write(s) applied; {counter.writes} mutating gh call(s); "
        f"{counter.reads} read call(s). Cache invalidated (mutated) — next run re-fetches."
    )
    return 0


# ─────────────────────────────────────────────────────────────────────────────────────────────
# Offline self-test of the pure diff logic (no network)
# ─────────────────────────────────────────────────────────────────────────────────────────────
def self_test():
    ok = 0

    def check(name, cond):
        nonlocal ok
        print(f"   {'ok  ' if cond else 'FAIL'} {name}")
        ok += 1 if cond else 0
        return cond

    # in-sync: identical entry ⇒ empty diff
    entry = {
        "id": "M-001",
        "title": "M-001 — x",
        "labels": ["status:done", "phase:0"],
        "milestone": "Phase 0",
        "body": "b\n",
    }
    live = {
        "number": 2,
        "title": "M-001 — x",
        "labels": ["phase:0", "status:done"],
        "milestone": "Phase 0",
        "body": "b",
        "state": "open",
    }
    check("in-sync ⇒ no changes", diff_entry(entry, live, update_bodies=True) == {})

    # label drift
    e2 = dict(entry, labels=["status:todo", "phase:0"])
    d2 = diff_entry(e2, live, update_bodies=False)
    check(
        "label drift detected", d2.get("labels") == (["status:todo"], ["status:done"])
    )

    # title + milestone drift
    e3 = dict(entry, title="M-001 — y", milestone="Phase 1")
    d3 = diff_entry(e3, live, update_bodies=False)
    check("title drift", d3.get("title") == "M-001 — y")
    check("milestone drift", d3.get("milestone") == "Phase 1")

    # body only compared under --update-bodies
    e4 = dict(entry, body="different")
    check(
        "body ignored w/o flag", "body" not in diff_entry(e4, live, update_bodies=False)
    )
    check("body drift w/ flag", "body" in diff_entry(e4, live, update_bodies=True))

    # state only when explicit; never from status:*
    e5 = dict(entry, state="closed")
    check(
        "explicit state drift",
        diff_entry(e5, live, update_bodies=False).get("state") == "closed",
    )
    check(
        "no implicit state", "state" not in diff_entry(entry, live, update_bodies=False)
    )

    # classify: create / update / in-sync / orphan
    issues = [
        entry,
        {"id": "M-999", "title": "new", "labels": [], "milestone": None, "body": ""},
    ]
    by_number = {
        2: live,
        7: {
            "number": 7,
            "title": "ghost",
            "labels": [],
            "milestone": None,
            "body": "",
            "state": "open",
        },
    }
    by_title = {r["title"]: r for r in by_number.values()}
    c, u, s, matched = classify(
        issues, {"M-001": 2}, by_number, by_title, update_bodies=True
    )
    check("classify create", [e["id"] for e in c] == ["M-999"])
    check("classify in-sync", [e["id"] for e in s] == ["M-001"])
    check("classify orphan", 7 not in matched)

    # normalize_body idempotence
    check("normalize CRLF", normalize_body("a\r\nb\r\n") == "a\nb")

    # write-path arg rendering (pure) — proves the gh args without touching the API
    ea = edit_args(
        {"labels": (["status:done"], ["status:todo"]), "title": "T", "milestone": "M"}
    )
    check(
        "edit_args render",
        ea
        == [
            "--add-label",
            "status:done",
            "--remove-label",
            "status:todo",
            "--title",
            "T",
            "--milestone",
            "M",
        ],
    )
    check(
        "render_changes summary",
        render_changes({"labels": (["a"], ["b"]), "state": "closed"})
        == "labels +a -b, state=closed",
    )

    total = 14
    print(f"\n>> self-test: {ok}/{total} checks passed")
    return 0 if ok == total else 1


def main(argv=None):
    p = argparse.ArgumentParser(
        description="Diff-based, rate-limit-frugal GitHub issue sync (dry-run by default).",
    )
    p.add_argument(
        "--repo", default=DEFAULT_REPO, help=f"owner/name (default {DEFAULT_REPO})"
    )
    p.add_argument(
        "--apply",
        action="store_true",
        help="perform the delta writes (default: dry-run)",
    )
    p.add_argument(
        "--refresh",
        action="store_true",
        help="force a fresh bulk snapshot (ignore cache)",
    )
    p.add_argument(
        "--update-bodies",
        action="store_true",
        help="also reconcile issue bodies (off by default; GitHub bodies accrue enactment notes)",
    )
    p.add_argument(
        "--max-writes",
        type=int,
        default=25,
        help="hard cap on mutating calls (default 25)",
    )
    p.add_argument(
        "--max-cache-age",
        type=int,
        default=DEFAULT_MAX_CACHE_AGE,
        help="auto-refetch the snapshot when older than N seconds (default 86400)",
    )
    p.add_argument(
        "--verbose", action="store_true", help="list orphan issues in the plan"
    )
    p.add_argument(
        "--self-test", action="store_true", help="run the offline pure-logic self-test"
    )
    args = p.parse_args(argv)

    if args.self_test:
        return self_test()
    try:
        return run(args)
    except RuntimeError as exc:
        sys.exit(f"!! {exc}")


if __name__ == "__main__":
    raise SystemExit(main())
