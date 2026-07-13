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
import re
import subprocess
import sys
import tempfile
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
# The engine's declared-override manifest. We EXTEND it (never fork it): its `overrides` block is
# read as class-1 accounting; a new top-level `orphans` block (which the engine ignores — its loader
# only reads `overrides`) records class-1 allowlist + class-2 superseded reconciliations.
PR_OVERRIDES = HERE / "pr-overrides.json"

# Stop writing when the remaining core rate-limit budget drops below this floor (never-silent).
RATE_FLOOR = 50
# Auto-refetch the snapshot cache when it is older than this many seconds (24h default).
DEFAULT_MAX_CACHE_AGE = 24 * 60 * 60
# A title Jaccard-token overlap at/above this flags an orphan as a likely duplicate of a tracked
# entry (→ uncertain, human-confirmed) rather than a fresh adoptable issue (never-guess, G2).
SIMILARITY_FLAG = 0.5
# Idempotency marker embedded in the class-2 supersede link comment (so a re-run never re-posts).
SUPERSEDE_MARKER = "<!-- sync_issues:superseded -->"


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
# PURE orphan classification + reconcile-action rendering (no I/O) — covered by --self-test
# ─────────────────────────────────────────────────────────────────────────────────────────────
_ID_RE = re.compile(r"^(M-\d+|E\d+-\d+)\b")


def title_task_id(title):
    """PURE: the leading task-id token of a title (``M-###``/``E##-#``), or None."""
    m = _ID_RE.match((title or "").strip())
    return m.group(1) if m else None


def _tokens(title):
    return set(re.findall(r"[a-z0-9]+", (title or "").lower()))


def title_similarity(a, b):
    """PURE: Jaccard token overlap of two titles in [0,1] (0 when either is empty)."""
    ta, tb = _tokens(a), _tokens(b)
    if not ta or not tb:
        return 0.0
    return len(ta & tb) / len(ta | tb)


def best_similar(title, issues):
    """PURE: the (entry_id, score) of the issues.yaml title closest to ``title`` (or None)."""
    best = None
    for e in issues:
        s = title_similarity(title, e["title"])
        if best is None or s > best[1]:
            best = (e["id"], s)
    return best


def classify_orphan(rec, *, issues_by_id, idmap, similar):
    """PURE: classify one UNACCOUNTED orphan into a reconcile class + evidence (never guesses).

    Returns (klass, evidence) with klass in:
      * ``superseded`` — a **closed** issue whose title's task-id maps (via idmap) to a DIFFERENT,
        canonical issue number ⇒ a duplicate of an already-tracked task.
      * ``adoptable``  — an **open** issue with no duplicate signal ⇒ a genuine issue that should be
        reverse-imported into ``issues.yaml`` as a new task.
      * ``non-task``   — a **closed** issue with no duplicate signal ⇒ allowlist it as a tracked
        non-task (an RFC/discussion issue, like #67).
      * ``uncertain``  — a conflicting signal (an OPEN task-id-duplicate, or a strong title match to a
        tracked entry) ⇒ **no auto-action**, flagged for a human (G2/VR-5).
    ``similar`` is ``best_similar(rec["title"], issues)`` — passed in so this stays pure.
    """
    tid = title_task_id(rec["title"])
    if tid and tid in issues_by_id:
        canonical = idmap.get(tid)
        if canonical and canonical != rec["number"]:
            if rec.get("state") == "closed":
                return "superseded", {"task_id": tid, "canonical": canonical}
            return "uncertain", {
                "task_id": tid,
                "canonical": canonical,
                "why": "duplicate task-id but the orphan is OPEN — confirm before superseding",
            }
    if similar and similar[1] >= SIMILARITY_FLAG:
        return "uncertain", {
            "similar_to": similar[0],
            "score": round(similar[1], 2),
            "why": "title closely matches a tracked entry — duplicate or adopt? confirm",
        }
    if rec.get("state") == "open":
        return "adoptable", {}
    return "non-task", {}


def supersede_comment_body(canonical, task_id):
    """PURE: the idempotent link-comment body posted on a class-2 duplicate (carries the marker)."""
    return (
        f"Superseded by #{canonical} — the canonical tracking issue for {task_id}. "
        f"Recorded by `sync_issues.py --reconcile-orphans`. {SUPERSEDE_MARKER}"
    )


def allowlist_record(rec, reason):
    """PURE: the class-1 ``orphans.allowlist`` record for a tracked non-task."""
    return {"number": rec["number"], "title": rec["title"], "reason": reason}


def superseded_record(rec, canonical, task_id):
    """PURE: the class-2 ``orphans.superseded`` record for a duplicate issue."""
    return {
        "number": rec["number"],
        "canonical": canonical,
        "task_id": task_id,
        "title": rec["title"],
    }


def next_free_mid(issues):
    """PURE: mint the next free ``M-####`` id above the current max (collision-checked; mitigation #1)."""
    nums = [int(m.group(1)) for e in issues if (m := re.fullmatch(r"M-(\d+)", e["id"]))]
    taken = {e["id"] for e in issues}
    candidate = (max(nums) + 1) if nums else 1
    while f"M-{candidate:03d}" in taken:
        candidate += 1
    return f"M-{candidate:03d}"


def build_adopted_entry(rec, mid):
    """PURE: a reviewable ``issues.yaml`` entry reverse-imported from a GitHub issue (class 3).

    Best-effort field mapping — title/body/labels/state carried over; a ``status:*`` label is derived
    from the GitHub state. Fields we CANNOT infer (phase/type/area/priority) are surfaced in an
    ``_adopt_flags`` list so a human completes them before the entry is treated as final (G2). Never
    silently invents a phase/priority.
    """
    gh_labels = [lb for lb in (rec.get("labels") or []) if not lb.startswith("status:")]
    status = "status:todo" if rec.get("state") == "open" else "status:done"
    missing = [
        p
        for p in ("phase:", "type:", "area:", "priority:")
        if not any(lb.startswith(p) for lb in gh_labels)
    ]
    entry = {
        "id": mid,
        "title": rec["title"],
        "labels": sorted(gh_labels + [status]),
        "body": (rec.get("body") or "")
        + f"\n\n(Adopted from GitHub #{rec['number']} by sync_issues.py; review the FLAGged fields.)",
        "_adopt_flags": [
            f"missing {p.rstrip(':')} label — set before final" for p in missing
        ]
        + [
            f"reverse-imported from GitHub #{rec['number']} — confirm milestone/depends_on/doc_refs"
        ],
    }
    return entry


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


def load_orphan_accounting(path):
    """Return the set of issue numbers already ACCOUNTED FOR (so they are not re-reported as orphans).

    Reads ``pr-overrides.json``: the engine's ``overrides`` block (a declared tracked non-task like
    #67) PLUS the ``orphans`` block this tool owns (``allowlist`` = class-1 recorded, ``superseded``
    = class-2 recorded). Also returns the raw ``orphans`` config so the reconcile path can append to
    it. Tolerant of an absent file (returns an empty accounting) but never-silent on a parse error.
    """
    accounted, orphans_cfg = set(), {"allowlist": [], "superseded": []}
    if not path.exists():
        return accounted, orphans_cfg
    try:
        raw = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        sys.exit(f"pr-overrides.json: JSON parse error — {exc}")
    for key in raw.get("overrides") or {}:
        if key.isdigit():
            accounted.add(int(key))
    cfg = raw.get("orphans") or {}
    for section in ("allowlist", "superseded"):
        for rec in cfg.get(section) or []:
            num = rec.get("number")
            if isinstance(num, int):
                accounted.add(num)
            orphans_cfg[section].append(rec)
    return accounted, orphans_cfg


def write_orphan_accounting(path, allowlist_adds, superseded_adds):
    """Append class-1/class-2 records to the ``orphans`` block of pr-overrides.json (source edit).

    Idempotent: a number already present in a section is not duplicated. The engine ignores this
    block (its loader reads only ``overrides``), so extending the file is side-effect-free there.
    """
    raw = json.loads(path.read_text(encoding="utf-8")) if path.exists() else {}
    cfg = raw.setdefault("orphans", {})
    cfg.setdefault(
        "_about",
        "Orphan reconciliation ledger owned by sync_issues.py (the engine ignores this block). "
        "allowlist = tracked non-task issues; superseded = duplicate issues recorded, not deleted.",
    )
    added = 0
    for section, adds in (
        ("allowlist", allowlist_adds),
        ("superseded", superseded_adds),
    ):
        lst = cfg.setdefault(section, [])
        present = {r.get("number") for r in lst}
        for rec in adds:
            if rec["number"] not in present:
                lst.append(rec)
                present.add(rec["number"])
                added += 1
    path.write_text(
        json.dumps(raw, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )
    return added


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


def issue_has_marker(repo, number, marker, counter):
    """Return True if issue ``number`` already carries a comment with ``marker`` (idempotency read)."""
    raw = gh(
        ["api", f"repos/{repo}/issues/{number}/comments?per_page=100", "--paginate"],
        counter=counter,
        kind="read",
    )
    try:
        comments = json.loads(raw)
    except json.JSONDecodeError:
        return False
    return any(marker in (c.get("body") or "") for c in comments)


def post_supersede_comment(repo, number, body, counter):
    """Post the class-2 link comment on a duplicate issue (idempotency is the caller's guard)."""
    gh(
        ["issue", "comment", str(number), "--repo", repo, "--body", body],
        counter=counter,
        kind="write",
    )


# ─────────────────────────────────────────────────────────────────────────────────────────────
# Orphan reconciliation (classified; dry-run by default; never auto-delete)
# ─────────────────────────────────────────────────────────────────────────────────────────────
def reconcile_orphans(args, issues, idmap, by_number, matched, counter):
    """Classify every unaccounted orphan and (under --apply) reconcile it per class. Never deletes."""
    accounted, _cfg = load_orphan_accounting(PR_OVERRIDES)
    issues_by_id = {e["id"] for e in issues}
    orphans = [
        r for n, r in by_number.items() if n not in matched and n not in accounted
    ]

    plans = []  # (klass, rec, evidence)
    for rec in orphans:
        klass, ev = classify_orphan(
            rec,
            issues_by_id=issues_by_id,
            idmap=idmap,
            similar=best_similar(rec["title"], issues),
        )
        plans.append((klass, rec, ev))

    order = {"non-task": 0, "superseded": 1, "adoptable": 2, "uncertain": 3}
    plans.sort(key=lambda p: (order[p[0]], p[1]["number"]))
    counts = {k: sum(1 for p in plans if p[0] == k) for k in order}

    print("\n== ORPHAN RECONCILE PLAN ==")
    print(f"   unaccounted orphans: {len(orphans)}")
    for k in order:
        print(f"   {k:11s}: {counts[k]}")
    for klass, rec, ev in plans:
        note = f" [{ev}]" if ev else ""
        if klass == "non-task":
            action = (
                "allowlist as tracked non-task (pr-overrides.json orphans.allowlist)"
            )
        elif klass == "superseded":
            action = (
                f"record superseded → canonical #{ev['canonical']} ({ev['task_id']}); "
                f"post idempotent link comment"
            )
        elif klass == "adoptable":
            action = f"reverse-import as {next_free_mid(issues)} (best-effort; FLAG for review)"
        else:
            action = "NO auto-action — human confirm"
        print(f"   #{rec['number']} [{rec.get('state')}] {klass}: {rec['title'][:70]}")
        print(f"        → {action}{note}")

    if not args.apply:
        print(
            "\n>> DRY-RUN (default): no mutations. Re-run with --reconcile-orphans --apply to act. "
            "Class-2 comments + class-3 adoptions are persistent — run them deliberately."
        )
        return 0

    # ── --apply: perform each class's reconcile action (rate-gated + capped) ──
    remaining = core_remaining(counter)
    if remaining is not None and remaining < RATE_FLOOR:
        sys.exit(
            f"\n!! core rate-limit budget low ({remaining} < {RATE_FLOOR}); refusing to write."
        )
    allowlist_adds, superseded_adds = [], []
    adopted_entries, adopted_idmap = [], []
    writes = 0
    for klass, rec, ev in plans:
        if writes >= args.max_writes:
            print(
                f"   … --max-writes {args.max_writes} reached; stopping (re-run to continue)."
            )
            break
        if klass == "non-task":
            allowlist_adds.append(
                allowlist_record(rec, "closed non-task tracking issue")
            )
            print(f"   + allowlisted #{rec['number']}")
        elif klass == "superseded":
            body = supersede_comment_body(ev["canonical"], ev["task_id"])
            if not issue_has_marker(
                args.repo, rec["number"], SUPERSEDE_MARKER, counter
            ):
                post_supersede_comment(args.repo, rec["number"], body, counter)
                writes += 1
                print(
                    f"   ~ commented #{rec['number']} → superseded by #{ev['canonical']}"
                )
            else:
                print(
                    f"   = #{rec['number']} already has the supersede comment (idempotent)"
                )
            superseded_adds.append(
                superseded_record(rec, ev["canonical"], ev["task_id"])
            )
        elif klass == "adoptable":
            mid = next_free_mid(issues)
            entry = build_adopted_entry(rec, mid)
            issues.append({"id": mid, "title": rec["title"]})  # reserve the id in-run
            adopted_entries.append(entry)
            adopted_idmap.append((mid, rec["number"], by_number[rec["number"]]["id"]))
            print(f"   + adopted #{rec['number']} → {mid} (review the FLAGged fields)")
        else:
            print(
                f"   ? #{rec['number']} uncertain — skipped (human confirm): {ev.get('why')}"
            )

    if allowlist_adds or superseded_adds:
        added = write_orphan_accounting(PR_OVERRIDES, allowlist_adds, superseded_adds)
        print(f">> pr-overrides.json: recorded {added} orphan reconciliation(s)")
    if adopted_entries:
        _append_adopted_issues(ISSUES_YAML, adopted_entries)
        append_idmap(IDMAP_TSV, adopted_idmap)
        print(
            f">> issues.yaml: reverse-imported {len(adopted_entries)} adopted entry(ies) (FLAG)"
        )
    if SNAPSHOT_CACHE.exists():
        SNAPSHOT_CACHE.unlink()
    print(
        f"\n>> orphan reconcile done: {counter.writes} mutating gh call(s). Cache invalidated."
    )
    return 0


def _append_adopted_issues(path, entries):
    """Append reverse-imported entries to issues.yaml, preserving the existing document (append-only)."""
    data = yaml.safe_load(path.read_text(encoding="utf-8"))
    data["issues"].extend(entries)
    path.write_text(
        yaml.safe_dump(data, sort_keys=False, allow_unicode=True, width=100),
        encoding="utf-8",
    )


# ─────────────────────────────────────────────────────────────────────────────────────────────
# Plan + drive
# ─────────────────────────────────────────────────────────────────────────────────────────────
def print_plan(to_create, to_update, in_sync, orphans, accounted_orphans, *, verbose):
    print("\n== PLAN ==")
    print(f"   in-sync  : {len(in_sync):4d}  (skipped — zero API calls)")
    print(f"   update   : {len(to_update):4d}  (only drifted fields)")
    print(f"   create   : {len(to_create):4d}  (missing on GitHub)")
    print(
        f"   orphan   : {len(orphans):4d}  (on GitHub, not in issues.yaml — reported, not deleted)"
    )
    print(
        f"   accounted: {accounted_orphans:4d}  (orphans already recorded in pr-overrides.json)"
    )
    if orphans:
        print("   → reconcile them with:  --reconcile-orphans  (dry-run) then --apply")
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

    if args.reconcile_orphans:
        return reconcile_orphans(args, issues, idmap, by_number, matched, counter)

    accounted, _cfg = load_orphan_accounting(PR_OVERRIDES)
    orphans = [
        r for n, r in by_number.items() if n not in matched and n not in accounted
    ]
    accounted_orphans = sum(1 for n in by_number if n not in matched and n in accounted)

    print_plan(
        to_create,
        to_update,
        in_sync,
        orphans,
        accounted_orphans,
        verbose=args.verbose,
    )

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
    appended = 0
    for entry in to_create:
        if writes_done >= args.max_writes:
            print(
                f"   … --max-writes {args.max_writes} reached; stopping (re-run to continue)."
            )
            break
        number, db_id = apply_create(repo, entry, counter)
        writes_done += 1
        # Checkpoint THIS row immediately (never batched to the end of the loop): a mid-batch `gh`
        # failure on a LATER create must not lose the idmap row for every EARLIER create already
        # applied this run — a deferred single end-of-loop append would leave zero checkpoint on
        # such a failure, so a retry re-creates already-created issues as duplicates (never-silent
        # G2; `append_idmap` is itself idempotent/append-only, so calling it once per row here is
        # safe and cheap — no batching benefit was being bought by deferring it in the first place).
        appended += append_idmap(IDMAP_TSV, [(entry["id"], number, db_id)])
    for entry, live, changes in to_update:
        if writes_done >= args.max_writes:
            print(
                f"   … --max-writes {args.max_writes} reached; stopping (re-run to continue)."
            )
            break
        apply_update(repo, live["number"], changes, counter)
        writes_done += 1

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

    # ── orphan classifier (each class + the uncertain guards) ──
    orphan_issues = [
        {"id": "M-302", "title": "M-302 — real task", "labels": [], "body": ""},
        {
            "id": "M-855",
            "title": "JIT for dynamic VSA/HDC workloads + ADR-009 deferral",
            "body": "",
        },
    ]
    ids = {e["id"] for e in orphan_issues}
    om = {"M-302": 87}  # canonical lives at #87

    def cls(rec):
        return classify_orphan(
            rec,
            issues_by_id=ids,
            idmap=om,
            similar=best_similar(rec["title"], orphan_issues),
        )[0]

    check(
        "class-2 superseded (closed id-dup)",
        cls({"number": 126, "title": "M-302 — real task", "state": "closed"})
        == "superseded",
    )
    check(
        "open id-dup ⇒ uncertain",
        cls({"number": 500, "title": "M-302 — real task", "state": "open"})
        == "uncertain",
    )
    check(
        "class-3 adoptable (open, novel)",
        cls(
            {
                "number": 900,
                "title": "totally unrelated brand new topic xyz",
                "state": "open",
            }
        )
        == "adoptable",
    )
    check(
        "class-1 non-task (closed, novel)",
        cls(
            {
                "number": 901,
                "title": "totally unrelated brand new topic xyz",
                "state": "closed",
            }
        )
        == "non-task",
    )
    check(
        "strong title match ⇒ uncertain (the #468/M-855 case)",
        cls(
            {
                "number": 468,
                "title": "JIT for dynamic / VSA workloads (ADR-009 deferred to enacted)",
                "state": "open",
            }
        )
        == "uncertain",
    )

    # ── reconcile-action rendering (pure; proves the actions without touching the API) ──
    check(
        "supersede comment marker + link",
        SUPERSEDE_MARKER in supersede_comment_body(87, "M-302")
        and "#87" in supersede_comment_body(87, "M-302"),
    )
    check(
        "next_free_mid above max",
        next_free_mid([{"id": "M-001", "title": "a"}, {"id": "M-1037", "title": "b"}])
        == "M-1038",
    )
    adopted = build_adopted_entry(
        {
            "number": 468,
            "title": "Adopt me",
            "labels": ["area:vsa"],
            "state": "open",
            "body": "x",
        },
        "M-1038",
    )
    check(
        "adopted entry: id + derived status + FLAGs",
        adopted["id"] == "M-1038"
        and "status:todo" in adopted["labels"]
        and any("missing priority" in f for f in adopted["_adopt_flags"]),
    )

    # ── accounting round-trip (the offline acceptance-test mechanism) ──
    # Recording an orphan ⇒ a later load counts it as accounted ⇒ it is no longer an orphan.
    with tempfile.TemporaryDirectory() as td:
        tmp = Path(td) / "pr-overrides.json"
        tmp.write_text('{"overrides": {"67": {"milestone": "P"}}}', encoding="utf-8")
        acc0, _ = load_orphan_accounting(tmp)
        check("overrides key accounted (#67)", 67 in acc0)
        added = write_orphan_accounting(
            tmp,
            [{"number": 901, "title": "t", "reason": "r"}],
            [{"number": 126, "canonical": 87, "task_id": "M-302", "title": "t"}],
        )
        acc1, _ = load_orphan_accounting(tmp)
        check("recorded orphans accounted", added == 2 and {126, 901} <= acc1)
        # idempotent: a second identical write adds nothing.
        again = write_orphan_accounting(
            tmp,
            [{"number": 901, "title": "t", "reason": "r"}],
            [{"number": 126, "canonical": 87, "task_id": "M-302", "title": "t"}],
        )
        check("accounting idempotent", again == 0)

    # ── idmap incremental-checkpoint idempotency (the mid-batch-failure fix) ──
    # `run()` now calls `append_idmap` ONCE PER created issue, immediately after that create
    # succeeds — not once at the end of the whole create-loop — so a mid-batch `gh` failure on a
    # LATER create leaves a correct PARTIAL idmap covering every EARLIER create already applied
    # this run, and a retry must not re-duplicate rows already checkpointed. Prove the property
    # offline: two single-row appends both accumulate (not overwrite each other), and re-appending
    # an already-known id (what a retry naturally does, since it re-walks the same `to_create`
    # list) is a no-op that still lands any genuinely-new row alongside it.
    with tempfile.TemporaryDirectory() as td:
        idmap_tmp = Path(td) / "idmap.tsv"
        n1 = append_idmap(idmap_tmp, [("M-2001", 501, "gid-1")])
        n2 = append_idmap(idmap_tmp, [("M-2002", 502, "gid-2")])
        rows_after_two = load_idmap(idmap_tmp)
        check(
            "idmap incremental checkpoint: two single-row appends both land",
            n1 == 1 and n2 == 1 and rows_after_two == {"M-2001": 501, "M-2002": 502},
        )
        # Simulated retry after a mid-batch failure: re-append the row that was ALREADY
        # checkpointed (M-2001) alongside one genuinely new row (M-2003, as if this create
        # succeeded on the retry) — the known id must be skipped (never duplicated), the new one
        # must still land.
        n3 = append_idmap(
            idmap_tmp, [("M-2001", 501, "gid-1"), ("M-2003", 503, "gid-3")]
        )
        rows_after_retry = load_idmap(idmap_tmp)
        check(
            "idmap re-append after retry: known id skipped, new id lands, no duplicate",
            n3 == 1
            and rows_after_retry == {"M-2001": 501, "M-2002": 502, "M-2003": 503},
        )

    total = 27
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
        "--reconcile-orphans",
        action="store_true",
        help="classify + reconcile GitHub orphans (allowlist / superseded / adopt); never deletes",
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
