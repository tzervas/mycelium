#!/usr/bin/env python3
"""Idempotent, cross-platform GitHub PM reconcile for Mycelium, driven by the manifests + `gh`.

Runs identically on Linux/macOS and on **Windows (PowerShell)** — it is pure Python plus the
`gh` CLI, with **no bash and no jq**. Invoke it the same way in any shell:

    python tools/github/gh-issues-sync.py            # issues: create absent + update drifted
    python tools/github/gh-issues-sync.py --all      # FULL maintenance suite (see below)
    python tools/github/gh-issues-sync.py --all --dry-run   # preview the whole reconcile
    python tools/github/gh-issues-sync.py --prs      # backfill labels/milestone on every PR
    python tools/github/gh-issues-sync.py --project  # reconcile the Project v2 board
    python tools/github/gh-issues-sync.py --validate # cross-manifest + codebase accuracy check
    python tools/github/gh-issues-sync.py --self-test       # offline check of the pure logic

It reconciles the repo to the committed source-of-truth manifests, idempotently and
**never-silently**:

  * ``labels.json``     -> create-or-update each label (color/description)         [--labels]
  * ``milestones.json`` -> create each absent milestone (by title)                 [--milestones]
  * ``issues.yaml``     -> create each absent issue, AND intelligently **update** an
                           existing one to match the manifest: labels, milestone, title
                           (and body only with --update-bodies).                   [--issues]
  * PRs (state=all)     -> derive type:*/area:* labels (+ milestone if inferable) from each
                           PR's Conventional-Commit title (fallback: its commits) and
                           reconcile **add-only** — never strip a human's labels.   [--prs]
  * ``project.json``    -> the Project v2 board: find-or-create "Mycelium", reconcile custom
                           fields + single-select options, add items, and set Status/Phase/
                           Area/Priority from each item's labels. Views + built-in workflows
                           are settings-only -> the run FLAGS them as manual steps.  [--project]
  * ``conventions.json``-> the commit/PR-title grammar -> label/milestone map (drives --prs).
  * ``idmap.tsv``       -> append any new task_id -> number -> db_id rows (append-only).

``--all`` runs the full maintenance suite in order: preflight (auth/scope sanity) -> validate
(manifests vs the codebase) -> labels -> milestones -> issues -> PRs -> project (when the
``project`` scope is present; else FLAGGED-skipped, never silently). Each level is
create-if-absent + update-to-match + ``--dry-run`` + never-silent.

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
import re
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


def gh_graphql(query):
    """Run a GraphQL query/mutation through `gh api graphql` and return the parsed `data`.

    Values are embedded into ``query`` via ``gql_str`` (a JSON-escaped GraphQL literal) — they are
    our trusted manifest strings or GitHub-returned ids/cursors, so no token is ever handled here
    (`gh` holds the credential). Raises on a GraphQL error (surfaced via gh's non-zero exit +
    stderr) so a failure is never silent (G2).
    """
    out = json.loads(gh(["api", "graphql", "-f", f"query={query}"]))
    if "errors" in out and out["errors"]:
        raise RuntimeError(f"GraphQL error: {json.dumps(out['errors'])}")
    return out.get("data", {})


def gql_str(text):
    """Encode a Python string as a GraphQL string literal (JSON escaping is a valid superset)."""
    return json.dumps(text or "")


# ─────────────────────────────────────────────────────────────────────────────────────────────
# Preflight / sanity check (never-silent; proceed when good, remediate only when lacking)
# ─────────────────────────────────────────────────────────────────────────────────────────────
def get_gh_scopes():
    """Return (authed, scopes) for the active `gh` token.

    ``scopes`` is the set parsed from ``gh auth status`` ("Token scopes: ...") or ``None`` when
    no scope line is present (e.g. a fine-grained PAT) — in which case we must NOT claim a scope
    is missing (the operation itself will surface any real permission error, never-silently).
    """
    try:
        proc = subprocess.run(
            ["gh", "auth", "status"], check=False, text=True, capture_output=True
        )
    except FileNotFoundError:
        return False, None
    blob = (proc.stdout or "") + (proc.stderr or "")
    authed = proc.returncode == 0 or "Logged in to" in blob
    scopes = None
    for line in blob.splitlines():
        if "Token scopes:" in line:
            raw = line.split("Token scopes:", 1)[1]
            scopes = {tok.strip().strip("'\"") for tok in raw.split(",") if tok.strip()}
            break
    return authed, scopes


def preflight_gh(*, need_project):
    """Sanity-check `gh` before a live reconcile. Returns ``project_ok`` (may we touch the board?).

    Honest by construction: if auth is good and the needed scope is present we proceed silently;
    we only emit the ``gh auth refresh`` remediation when a scope the operation NEEDS is provably
    absent (so a good token is never asked to refresh — and a fine-grained token, whose scopes we
    cannot read, is trusted to fail loudly at the call site instead).
    """
    authed, scopes = get_gh_scopes()
    if not authed:
        scope_hint = "repo, project" if need_project else "repo"
        sys.exit(
            f"ERROR: gh is not authenticated. Run `gh auth login` (scopes: {scope_hint})."
        )
    print("   = gh: authenticated")
    if not need_project:
        return False
    if scopes is None:
        # Unknown scope set (fine-grained token): trust it; the board call fails loudly if not.
        print(
            "   ~ gh token scopes not enumerable (fine-grained?) — proceeding; "
            "the board API will surface any permission gap explicitly."
        )
        return True
    if "project" in scopes:
        print("   = gh: 'project' scope present")
        return True
    # The ONLY case that needs a refresh: the scope is genuinely missing.
    print(
        "   ! Projects v2 needs the 'project' scope, which this token lacks.\n"
        "     Remediation (one-time, per machine):  gh auth refresh -s project\n"
        "     (verify with: gh auth status). Skipping --project this run (never silent, G2).",
        file=sys.stderr,
    )
    return False


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
# Pure Conventional-Commit logic for the PR path (drives --prs; exercised by --self-test)
# ─────────────────────────────────────────────────────────────────────────────────────────────
# `type(scope)!: subject` — scope is optional; one or more comma/'/'-separated scopes; optional
# breaking `!`. We are deliberately strict: a title that does not match yields None so the caller
# can fall back to the commits, then FLAG — never guess a type (G2).
_CONVENTIONAL = re.compile(
    r"^(?P<type>[a-z]+)(?:\((?P<scope>[^)]*)\))?(?P<breaking>!)?:\s+(?P<subject>.+)$"
)


def parse_conventional(title):
    """Parse a Conventional-Commit subject line into {type, scopes, breaking, subject} or None."""
    if not title:
        return None
    match = _CONVENTIONAL.match(title.strip())
    if not match:
        return None
    raw_scope = match.group("scope") or ""
    scopes = [s.strip() for s in re.split(r"[,/]", raw_scope) if s.strip()]
    return {
        "type": match.group("type").lower(),
        "scopes": scopes,
        "breaking": bool(match.group("breaking")),
        "subject": match.group("subject").strip(),
    }


def derive_pr_labels(parsed, type_map, area_set, scope_aliases=None):
    """Map a parsed title to (labels:set, flags:list) using conventions.json + the area:* set.

    Pure + honest: an unknown type, or a scope that is neither an area:* label nor a declared
    ``scope_to_area`` alias, produces a FLAG, never an invented label (G2). The type label is
    required; area labels are best-effort on an exact scope match or a ratified alias.
    """
    scope_aliases = scope_aliases or {}
    labels, flags = set(), []
    if parsed is None:
        return labels, ["title not Conventional-Commit form; type unresolved"]
    type_label = type_map.get(parsed["type"])
    if type_label:
        labels.add(type_label)
    else:
        flags.append(f"unknown commit type '{parsed['type']}' (no type:* mapping)")
    for scope in parsed["scopes"]:
        if f"area:{scope}" in area_set:
            labels.add(f"area:{scope}")
        elif scope in scope_aliases:  # a declared alias turns a FLAG into a mapping
            aliased = f"area:{scope_aliases[scope]}"
            if aliased in area_set:
                labels.add(aliased)
            else:
                flags.append(
                    f"scope alias '{scope}' -> '{scope_aliases[scope]}' is not an area:* label"
                )
        else:
            flags.append(f"scope '{scope}' is not an area:* label (no area inferred)")
    if parsed["breaking"]:
        flags.append("breaking change marked (no breaking:* label to apply)")
    return labels, flags


def extract_task_ids(text, patterns):
    """Return the ordered, de-duplicated task-ids matching any of ``patterns`` in ``text``."""
    found, seen = [], set()
    for pat in patterns:
        for match in re.findall(pat, text or ""):
            if match not in seen:
                seen.add(match)
                found.append(match)
    return found


def infer_milestone(task_ids, task_to_ms):
    """Infer a single milestone from referenced task-ids, or (None, flag) when not unambiguous."""
    milestones = {task_to_ms[t] for t in task_ids if t in task_to_ms}
    if len(milestones) == 1:
        return next(iter(milestones)), None
    if not milestones:
        return None, None  # nothing to infer from — silent (no claim made)
    return None, f"ambiguous milestone across {sorted(task_ids)}: {sorted(milestones)}"


# ─────────────────────────────────────────────────────────────────────────────────────────────
# Pure Project-v2 mapping/diff logic (drives --project; exercised by --self-test)
# ─────────────────────────────────────────────────────────────────────────────────────────────
def label_to_field_values(labels, field_label_map):
    """Map an item's labels to {field_name: option_name} per project.json, with unmapped flags.

    Pure: resolves prefix rules (phase:/area:/priority:) and the exact status map to option
    *names*; the engine later resolves names to option ids and writes only on drift. Unmapped
    status:* labels are FLAGGED, never guessed (G2).
    """
    labels = set(labels)
    values, flags = {}, []
    for field, rule in field_label_map.items():
        if field.startswith("_"):
            continue
        if rule.get("from") == "label_prefix":
            prefix = rule["prefix"]
            hits = [lb[len(prefix) :] for lb in labels if lb.startswith(prefix)]
            if len(hits) == 1:
                values[field] = rule["template"].replace("{value}", hits[0])
            elif len(hits) > 1:
                flags.append(
                    f"{field}: multiple {prefix}* labels {sorted(hits)} — not set"
                )
        elif rule.get("from") == "label_exact":
            mapped = [rule["map"][lb] for lb in labels if lb in rule["map"]]
            if len(mapped) == 1:
                values[field] = mapped[0]
            elif len(mapped) > 1:
                flags.append(
                    f"{field}: conflicting status labels {sorted(mapped)} — not set"
                )
    return values, flags


def plan_option_reconcile(desired_options, actual_option_names):
    """Return the option names to ADD so a single-select field covers ``desired_options``.

    Add-only: we never delete an option (it may be in use). Order-preserving on desired.
    """
    actual = set(actual_option_names)
    return [opt["name"] for opt in desired_options if opt["name"] not in actual]


def plan_field_reconcile(desired_fields, actual_fields_by_name):
    """Plan field/option creation. Returns {'create': [...], 'add_options': {field: [names]}}.

    Pure: ``actual_fields_by_name`` maps name -> {options:[names]}. A field absent entirely is a
    create; a present single-select gets its missing options added (never-silent, add-only).
    """
    create, add_options = [], {}
    for field in desired_fields:
        name = field["name"]
        if name not in actual_fields_by_name:
            create.append(name)
            continue
        missing = plan_option_reconcile(
            field.get("options", []), actual_fields_by_name[name].get("options", [])
        )
        if missing:
            add_options[name] = missing
    return create, add_options


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
            "node_id": item.get(
                "node_id"
            ),  # GraphQL global id — needed to add a project item
            "title": item["title"],
            "body": item.get("body") or "",
            "labels": {lb["name"] for lb in item.get("labels", [])},
            "milestone": (item.get("milestone") or {}).get("title"),
            "state": item.get("state"),
        }
        by_number[item["number"]] = rec
        by_title[item["title"]] = rec
    return by_number, by_title


def snapshot_prs(repo):
    """Index every pull request (state=all) with the fields the PR path reconciles."""
    raw = gh(["api", "--paginate", f"repos/{repo}/pulls?state=all&per_page=100"])
    prs = []
    for item in json.loads(raw):
        prs.append(
            {
                "number": item["number"],
                "node_id": item.get("node_id"),
                "title": item["title"],
                "body": item.get("body") or "",
                "labels": {lb["name"] for lb in item.get("labels", [])},
                "milestone": (item.get("milestone") or {}).get("title"),
                "merged": bool(item.get("merged_at")),
                "state": item.get("state"),
            }
        )
    return prs


def pr_commit_titles(repo, number):
    """Return the commit subject lines of PR ``number`` (the fallback when the title won't parse)."""
    raw = gh(["api", "--paginate", f"repos/{repo}/pulls/{number}/commits?per_page=100"])
    titles = []
    for commit in json.loads(raw):
        message = (commit.get("commit") or {}).get("message") or ""
        titles.append(message.splitlines()[0] if message else "")
    return titles


def build_task_to_milestone(issues):
    """Map each issues.yaml task-id (and its title-derived id) to its milestone title."""
    mapping = {}
    for entry in issues:
        if entry.get("milestone"):
            mapping[entry["id"]] = entry["milestone"]
    return mapping


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
# PR reconcile (add-only label/milestone backfill from the Conventional-Commit title)
# ─────────────────────────────────────────────────────────────────────────────────────────────
def reconcile_prs(repo, conventions, area_set, task_to_ms, *, dry_run):
    type_map = {
        **conventions["type_to_label"],
        **{
            k: v
            for k, v in conventions.get("type_to_label_repo_ext", {}).items()
            if not k.startswith("_")
        },
    }
    patterns = conventions["milestone_inference"]["task_id_patterns"]
    scope_aliases = conventions.get("scope_to_area", {}).get("aliases", {})
    prs = snapshot_prs(repo)
    print(f">> PRs: {len(prs)} on {repo} — add-only label/milestone backfill")

    updated, in_sync = 0, 0
    for pr in prs:
        number = pr["number"]
        text = f"{pr['title']}\n{pr['body']}"
        parsed = parse_conventional(pr["title"])
        source = "title"
        if parsed is None:  # fallback: the PR's own commit subjects
            for subject in pr_commit_titles(repo, number):
                cand = parse_conventional(subject)
                if cand is not None:
                    parsed, source, text = cand, "commit", text + "\n" + subject
                    break

        desired, flags = derive_pr_labels(parsed, type_map, area_set, scope_aliases)
        ms, ms_flag = infer_milestone(extract_task_ids(text, patterns), task_to_ms)
        if ms_flag:
            flags.append(ms_flag)

        to_add = sorted(desired - pr["labels"])
        set_ms = ms if (ms and ms != pr["milestone"]) else None

        for flag in flags:  # never-silent: report every refusal to invent
            print(f"   ! #{number}: {flag}", file=sys.stderr)

        if not to_add and not set_ms:
            in_sync += 1
            continue

        summary = []
        if to_add:
            summary.append("labels +" + " +".join(to_add))
        if set_ms:
            summary.append(f"milestone={set_ms}")
        via = "" if source == "title" else f" (via {source})"
        if dry_run:
            print(f"   ~ would update #{number}: {', '.join(summary)}{via}")
            continue
        edit_args = ["pr", "edit", str(number), "--repo", repo]
        if to_add:
            edit_args += ["--add-label", ",".join(to_add)]
        if set_ms:
            edit_args += ["--milestone", set_ms]
        gh(edit_args)
        print(f"   ~ updated #{number}: {', '.join(summary)}{via}")
        updated += 1

    print(f">> PRs done — {updated} updated, {in_sync} already in sync")


# ─────────────────────────────────────────────────────────────────────────────────────────────
# Project v2 reconcile (via `gh api graphql`). Pure mapping/diff lives above (--self-test'd);
# the I/O below is never-silent and --dry-run-guarded. The live GraphQL path must be validated
# with --dry-run on a gh-authed machine carrying the `project` scope (see RECONCILE.md).
# ─────────────────────────────────────────────────────────────────────────────────────────────
def _owner_field(owner_type):
    return "organization" if owner_type == "org" else "user"


def find_project(owner, owner_type, title):
    """Return {'id', 'number'} for the owner's Project v2 named ``title``, or None."""
    field = _owner_field(owner_type)
    query = (
        f"query{{ {field}(login:{gql_str(owner)}){{ projectsV2(first:100){{ "
        f"nodes{{ id number title }} }} }} }}"
    )
    data = gh_graphql(query)
    nodes = (((data.get(field) or {}).get("projectsV2") or {}).get("nodes")) or []
    for node in nodes:
        if node.get("title") == title:
            return {"id": node["id"], "number": node["number"]}
    return None


def owner_id(owner, owner_type):
    field = _owner_field(owner_type)
    data = gh_graphql(f"query{{ {field}(login:{gql_str(owner)}){{ id }} }}")
    return (data.get(field) or {}).get("id")


def create_project(owner, owner_type, title):
    oid = owner_id(owner, owner_type)
    if not oid:
        sys.exit(f"ERROR: could not resolve owner id for {owner_type} '{owner}'.")
    query = (
        f"mutation{{ createProjectV2(input:{{ownerId:{gql_str(oid)}, "
        f"title:{gql_str(title)}}}){{ projectV2{{ id number }} }} }}"
    )
    proj = gh_graphql(query)["createProjectV2"]["projectV2"]
    return {"id": proj["id"], "number": proj["number"]}


def fetch_project_fields(project_id):
    """Return {field_name: {'id', 'options': {option_name: option_id}}} for single-selects."""
    query = (
        f"query{{ node(id:{gql_str(project_id)}){{ ... on ProjectV2{{ "
        f"fields(first:50){{ nodes{{ __typename "
        f"... on ProjectV2FieldCommon{{ id name }} "
        f"... on ProjectV2SingleSelectField{{ id name options{{ id name }} }} }} }} }} }} }}"
    )
    nodes = (
        ((gh_graphql(query).get("node") or {}).get("fields") or {}).get("nodes")
    ) or []
    fields = {}
    for node in nodes:
        if not node.get("name"):
            continue
        options = {o["name"]: o["id"] for o in node.get("options", [])}
        fields[node["name"]] = {"id": node["id"], "options": options}
    return fields


def create_single_select_field(project_id, field):
    """Create an absent single-select field with all its options (one safe, non-destructive call)."""
    opts = ", ".join(
        f"{{name:{gql_str(o['name'])}, color:{o.get('color', 'GRAY')}, "
        f"description:{gql_str(o.get('description', ''))}}}"
        for o in field.get("options", [])
    )
    query = (
        f"mutation{{ createProjectV2Field(input:{{projectId:{gql_str(project_id)}, "
        f"dataType:SINGLE_SELECT, name:{gql_str(field['name'])}, "
        f"singleSelectOptions:[{opts}]}}){{ projectV2Field{{ "
        f"... on ProjectV2SingleSelectField{{ id name }} }} }} }}"
    )
    gh_graphql(query)


def project_items(project_id):
    """Return {content_number: {'item_id', 'fields': {field_name: option_name}}} (paginated)."""
    items, cursor = {}, None
    while True:
        after = f", after:{gql_str(cursor)}" if cursor else ""
        query = (
            f"query{{ node(id:{gql_str(project_id)}){{ ... on ProjectV2{{ "
            f"items(first:100{after}){{ pageInfo{{ hasNextPage endCursor }} nodes{{ id "
            f"content{{ __typename ... on Issue{{ number }} ... on PullRequest{{ number }} }} "
            f"fieldValues(first:20){{ nodes{{ __typename "
            f"... on ProjectV2ItemFieldSingleSelectValue{{ name "
            f"field{{ ... on ProjectV2SingleSelectField{{ name }} }} }} }} }} }} }} }} }} }}"
        )
        block = ((gh_graphql(query).get("node") or {}).get("items")) or {}
        for node in block.get("nodes", []):
            content = node.get("content") or {}
            num = content.get("number")
            if num is None:
                continue  # a draft item with no issue/PR content
            values = {}
            for fv in (node.get("fieldValues") or {}).get("nodes", []):
                fname = (fv.get("field") or {}).get("name")
                if fname and "name" in fv:
                    values[fname] = fv["name"]
            items[num] = {"item_id": node["id"], "fields": values}
        page = block.get("pageInfo") or {}
        if not page.get("hasNextPage"):
            return items
        cursor = page.get("endCursor")


def add_project_item(project_id, content_node_id):
    query = (
        f"mutation{{ addProjectV2ItemById(input:{{projectId:{gql_str(project_id)}, "
        f"contentId:{gql_str(content_node_id)}}}){{ item{{ id }} }} }}"
    )
    return gh_graphql(query)["addProjectV2ItemById"]["item"]["id"]


def set_item_field(project_id, item_id, field_id, option_id):
    query = (
        f"mutation{{ updateProjectV2ItemFieldValue(input:{{projectId:{gql_str(project_id)}, "
        f"itemId:{gql_str(item_id)}, fieldId:{gql_str(field_id)}, "
        f"value:{{singleSelectOptionId:{gql_str(option_id)}}}}}){{ projectV2Item{{ id }} }} }}"
    )
    gh_graphql(query)


def reconcile_project(repo, manifest, contents, *, dry_run):
    """Reconcile the Project v2 board to project.json. ``contents`` = the issue/PR records to add.

    Idempotent + never-silent: find-or-create the project; create absent fields (with options);
    FLAG missing options on existing fields + every settings-only view/workflow as a manual step;
    add absent items; set Status/Phase/Area/Priority only where the value drifts.
    """
    proj = manifest["project"]
    owner, owner_type, title = (
        proj["owner"],
        proj.get("owner_type", "user"),
        proj["title"],
    )

    print(f">> project: '{title}' (owner {owner_type} {owner})")
    found = find_project(owner, owner_type, title)
    if found is None:
        if dry_run:
            print(f"   + would create project '{title}' and reconcile it")
            return
        found = create_project(owner, owner_type, title)
        print(f"   + created project #{found['number']}")
    else:
        print(f"   = exists: project #{found['number']}")
    pid = found["id"]

    # 1) Fields + options. Reads are always live (even under --dry-run) so the preview is
    # accurate — only the mutations below are suppressed.
    actual = fetch_project_fields(pid)
    actual_by_name = {n: {"options": list(f["options"])} for n, f in actual.items()}
    create, add_options = plan_field_reconcile(manifest["fields"], actual_by_name)
    for name in create:
        field = next(f for f in manifest["fields"] if f["name"] == name)
        if dry_run:
            print(
                f"   + would create field: {name} ({len(field.get('options', []))} options)"
            )
        else:
            create_single_select_field(pid, field)
            print(f"   + created field: {name}")
    for name, missing in add_options.items():
        # An add-to-existing option mutation replaces the option set and is not safely
        # idempotent without live validation — so we FLAG it (never silently skip; G2).
        print(
            f"   ! field '{name}' is missing option(s) {missing} — add them in the project UI "
            f"(API option-edit is not enabled in v0; see RECONCILE.md).",
            file=sys.stderr,
        )
    if not dry_run and create:
        actual = fetch_project_fields(pid)  # refetch so new field/option ids resolve

    # 2) Views + built-in workflows are settings-only — record intent, FLAG as manual.
    for view in manifest.get("views", []):
        if not view.get("api_writable", False):
            print(
                f"   ! manual view: '{view['name']}' — {json.dumps(view.get('intent', {}))}"
            )
    for auto in manifest.get("automation", []):
        if not auto.get("api_writable", False):
            print(f"   ! manual workflow: {auto['id']} — {auto.get('intent', '')}")

    # 3) Items + field values. Items are read live even under --dry-run so the preview reports
    # real adds AND real field-value drift on items that already exist.
    field_label_map = manifest["field_label_map"]
    existing = project_items(pid)
    added = set_count = synced = 0
    for rec in contents:
        number, node_id, labels = rec["number"], rec.get("node_id"), rec["labels"]
        values, flags = label_to_field_values(labels, field_label_map)
        for flag in flags:
            print(f"   ! #{number}: {flag}", file=sys.stderr)

        if number not in existing:
            if dry_run:
                fv = ", ".join(f"{k}={v}" for k, v in values.items())
                print(f"   + would add item #{number}" + (f" [{fv}]" if fv else ""))
                added += 1
                continue  # the item/option ids do not exist yet — nothing more to preview
            if not node_id:
                print(
                    f"   ! #{number}: no node_id — cannot add to board", file=sys.stderr
                )
                continue
            existing[number] = {"item_id": add_project_item(pid, node_id), "fields": {}}
            added += 1

        item = existing[number]
        for field_name, option_name in values.items():
            field = actual.get(field_name)
            if (
                not field
            ):  # field not created yet (a would-create field in a --dry-run preview)
                continue
            option_id = field["options"].get(option_name)
            if not option_id:
                print(
                    f"   ! #{number}: field '{field_name}' has no option '{option_name}'",
                    file=sys.stderr,
                )
                continue
            if item["fields"].get(field_name) == option_name:
                synced += 1
                continue
            if dry_run:
                print(f"   ~ would set #{number}: {field_name} = {option_name}")
                set_count += 1
                continue
            set_item_field(pid, item["item_id"], field["id"], option_id)
            print(f"   ~ #{number}: {field_name} = {option_name}")
            set_count += 1

    verb = "would change" if dry_run else "done"
    print(
        f">> project {verb} — {added} item(s) {'to add' if dry_run else 'added'}, "
        f"{set_count} field value(s) {'to set' if dry_run else 'set'}, {synced} already in sync"
    )


# ─────────────────────────────────────────────────────────────────────────────────────────────
# Cross-manifest + codebase-accuracy validation (part of --all; runnable via --validate)
# ─────────────────────────────────────────────────────────────────────────────────────────────
def validate_manifests(here, *, repo_root):
    """Check the manifests are internally consistent AND accurate to the codebase.

    Returns (errors, warnings). Errors are inconsistencies that would mis-drive a reconcile
    (label/option targets that do not exist, project↔labels parity) and BLOCK ``--all``.
    Warnings are advisory drift (a stale idmap row, changelog hygiene) — reported never-silently
    but non-blocking. Both lists empty == clean.
    """
    errors, warnings = [], []
    labels = {d["name"] for d in json.loads((here / "labels.json").read_text())}
    conventions = json.loads((here / "conventions.json").read_text())
    project = json.loads((here / "project.json").read_text())

    # conventions: every type→label / repo-ext target is a real label; aliases hit real areas.
    type_targets = dict(conventions["type_to_label"])
    type_targets.update(
        {
            k: v
            for k, v in conventions.get("type_to_label_repo_ext", {}).items()
            if not k.startswith("_")
        }
    )
    for ctype, label in type_targets.items():
        if label not in labels:
            errors.append(
                f"conventions.json: type '{ctype}' -> '{label}' is not in labels.json"
            )
    for scope, area in conventions["scope_to_area"].get("aliases", {}).items():
        if f"area:{area}" not in labels and area not in labels:
            errors.append(
                f"conventions.json: scope alias '{scope}' -> '{area}' is not an area label"
            )

    # project: Area options == area:* labels; field-map targets are real options/labels.
    area_labels = {n[len("area:") :] for n in labels if n.startswith("area:")}
    area_opts = {
        o["name"]
        for f in project["fields"]
        if f["name"] == "Area"
        for o in f["options"]
    }
    if area_labels != area_opts:
        errors.append(
            f"project.json: Area options {sorted(area_opts)} != area:* labels {sorted(area_labels)}"
        )
    status_opts = {
        o["name"]
        for f in project["fields"]
        if f["name"] == "Status"
        for o in f["options"]
    }
    status_rule = project["field_label_map"].get("Status", {})
    for lbl, opt in status_rule.get("map", {}).items():
        if lbl not in labels:
            errors.append(f"project.json: Status map references unknown label '{lbl}'")
        if opt not in status_opts:
            errors.append(
                f"project.json: Status map target '{opt}' is not a Status option"
            )

    # idmap ↔ issues.yaml: a mapped task-id absent from issues.yaml is advisory drift (history
    # is append-only, so a removed-from-plan task can legitimately still have a mapping).
    issues_path = here / "issues.yaml"
    if issues_path.exists():
        spec = yaml.safe_load(issues_path.read_text()) or {}
        issue_ids = {e["id"] for e in spec.get("issues", [])}
        stale = [t for t in load_idmap(here / "idmap.tsv") if t not in issue_ids]
        if stale:
            warnings.append(
                f"idmap.tsv: {len(stale)} task(s) not in issues.yaml (e.g. {sorted(stale)[:5]})"
            )

    # changelog hygiene: present, Keep-a-Changelog Unreleased section.
    changelog = repo_root / "CHANGELOG.md"
    if not changelog.exists():
        warnings.append("CHANGELOG.md is missing")
    elif "## [Unreleased]" not in changelog.read_text(encoding="utf-8"):
        warnings.append("CHANGELOG.md has no '## [Unreleased]' section")

    return errors, warnings


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

    # ── Conventional-Commit PR path ───────────────────────────────────────────────────────────
    assert parse_conventional("feat(swap): add cert") == {
        "type": "feat",
        "scopes": ["swap"],
        "breaking": False,
        "subject": "add cert",
    }
    assert parse_conventional("feat(l1,grammar)!: redo")["scopes"] == ["l1", "grammar"]
    assert parse_conventional("feat(l1,grammar)!: redo")["breaking"] is True
    assert parse_conventional("docs: plain") == {
        "type": "docs",
        "scopes": [],
        "breaking": False,
        "subject": "plain",
    }
    assert (
        parse_conventional("Add MCP split bootstrap") is None
    )  # non-conventional → None

    type_map = {"feat": "type:feature", "docs": "type:docs"}
    areas = {"area:swap", "area:toolchain"}
    # exact scope match → area label; unknown scope/type → FLAG, never invented (G2).
    labels, flags = derive_pr_labels(
        parse_conventional("feat(swap): x"), type_map, areas
    )
    assert labels == {"type:feature", "area:swap"} and flags == [], (labels, flags)
    labels, flags = derive_pr_labels(
        parse_conventional("feat(mlir): x"), type_map, areas
    )
    assert labels == {"type:feature"} and any("mlir" in f for f in flags), (
        labels,
        flags,
    )
    labels, flags = derive_pr_labels(parse_conventional("spec(x): y"), type_map, areas)
    assert labels == set() and any("unknown commit type" in f for f in flags)
    labels, flags = derive_pr_labels(None, type_map, areas)
    assert labels == set() and flags  # unparsed title is flagged, not invented
    # a declared scope alias turns a FLAG into a mapping; an alias to a non-area still FLAGs.
    labels, flags = derive_pr_labels(
        parse_conventional("feat(mlir): x"),
        type_map,
        {"area:execution"},
        {"mlir": "execution"},
    )
    assert labels == {"type:feature", "area:execution"} and flags == [], (labels, flags)
    labels, flags = derive_pr_labels(
        parse_conventional("feat(zzz): x"),
        type_map,
        {"area:execution"},
        {"zzz": "nope"},
    )
    assert labels == {"type:feature"} and any("alias" in f for f in flags), (
        labels,
        flags,
    )

    # milestone inference: unambiguous → set; conflicting → flag; none → silent.
    t2m = {"M-150": "Phase 1", "M-151": "Phase 1", "M-201": "Phase 2"}
    assert extract_task_ids("does M-150 and M-151", ["M-[0-9]+"]) == ["M-150", "M-151"]
    assert infer_milestone(["M-150", "M-151"], t2m) == ("Phase 1", None)
    assert infer_milestone([], t2m) == (None, None)
    ms, flag = infer_milestone(["M-150", "M-201"], t2m)
    assert ms is None and flag and "ambiguous" in flag

    # ── Project v2 pure mapping/diff ──────────────────────────────────────────────────────────
    field_map = {
        "Phase": {
            "from": "label_prefix",
            "prefix": "phase:",
            "template": "Phase {value}",
        },
        "Area": {"from": "label_prefix", "prefix": "area:", "template": "{value}"},
        "Priority": {
            "from": "label_prefix",
            "prefix": "priority:",
            "template": "{value}",
        },
        "Status": {
            "from": "label_exact",
            "map": {"status:blocked": "Blocked", "status:done": "Done"},
        },
    }
    vals, vflags = label_to_field_values(
        {"phase:8", "area:toolchain", "priority:P3", "status:done"}, field_map
    )
    assert vals == {
        "Phase": "Phase 8",
        "Area": "toolchain",
        "Priority": "P3",
        "Status": "Done",
    }, vals
    assert vflags == []
    # status:needs-design has no Status option → unmapped (no Status key), not invented.
    vals2, _ = label_to_field_values({"phase:0", "status:needs-design"}, field_map)
    assert vals2 == {"Phase": "Phase 0"}, vals2

    desired_fields = [
        {
            "name": "Status",
            "options": [{"name": "Todo"}, {"name": "Blocked"}, {"name": "Done"}],
        },
        {"name": "Phase", "options": [{"name": "Phase 0"}, {"name": "Phase 1"}]},
    ]
    actual_fields = {
        "Status": {"options": ["Todo", "Done"]}
    }  # Phase absent; Blocked missing
    create, add_opts = plan_field_reconcile(desired_fields, actual_fields)
    assert create == ["Phase"] and add_opts == {"Status": ["Blocked"]}, (
        create,
        add_opts,
    )
    # already in sync → no creates, no option adds.
    synced_fields = {
        "Status": {"options": ["Todo", "Blocked", "Done"]},
        "Phase": {"options": ["Phase 0", "Phase 1"]},
    }
    assert plan_field_reconcile(desired_fields, synced_fields) == ([], {})

    print(
        "self-test OK: label_delta, normalize_body, plan_issue_update, parse_conventional, "
        "derive_pr_labels, infer_milestone, label_to_field_values, plan_field_reconcile"
    )


# ─────────────────────────────────────────────────────────────────────────────────────────────
# Validation driver (manifest-check.py + the cross-manifest checks above)
# ─────────────────────────────────────────────────────────────────────────────────────────────
def run_validation(args):
    """Run the full manifest validation. Returns True when there are no blocking errors."""
    print(">> validate: manifests vs codebase")
    ok = True
    checker = HERE / "manifest-check.py"
    if (
        checker.exists()
    ):  # reuse the existing issues↔labels/milestones preflight (cross-platform)
        rc = subprocess.run(
            [
                sys.executable,
                str(checker),
                "--issues-yaml",
                str(args.issues_yaml),
                "--labels",
                str(args.labels_json),
                "--milestones",
                str(args.milestones_json),
            ],
            check=False,
        ).returncode
        ok = ok and rc == 0
    errors, warnings = validate_manifests(HERE, repo_root=HERE.parents[1])
    for warn in warnings:
        print(f"   ~ warning: {warn}")
    for err in errors:
        print(f"   ERROR: {err}", file=sys.stderr)
    if errors:
        ok = False
    if ok:
        print("   = cross-manifest checks pass (conventions/project/labels in parity)")
    return ok


# ─────────────────────────────────────────────────────────────────────────────────────────────
# CLI
# ─────────────────────────────────────────────────────────────────────────────────────────────
def main():
    parser = argparse.ArgumentParser(
        description="Idempotent, cross-platform gh PM reconcile "
        "(labels + milestones + issues + PRs + Project v2)."
    )
    parser.add_argument("--repo", default="tzervas/mycelium")
    parser.add_argument("--issues-yaml", type=Path, default=HERE / "issues.yaml")
    parser.add_argument("--idmap", type=Path, default=HERE / "idmap.tsv")
    parser.add_argument("--labels-json", type=Path, default=HERE / "labels.json")
    parser.add_argument(
        "--milestones-json", type=Path, default=HERE / "milestones.json"
    )
    parser.add_argument(
        "--conventions-json", type=Path, default=HERE / "conventions.json"
    )
    parser.add_argument("--project-json", type=Path, default=HERE / "project.json")
    parser.add_argument(
        "--all",
        action="store_true",
        help="FULL maintenance suite: validate + labels + milestones + issues + PRs + project",
    )
    parser.add_argument("--labels", action="store_true", help="reconcile labels.json")
    parser.add_argument(
        "--milestones", action="store_true", help="reconcile milestones.json"
    )
    parser.add_argument(
        "--issues", action="store_true", help="reconcile issues.yaml (default)"
    )
    parser.add_argument(
        "--prs", action="store_true", help="backfill PR labels/milestone"
    )
    parser.add_argument(
        "--project", action="store_true", help="reconcile the Project v2 board"
    )
    parser.add_argument(
        "--validate",
        action="store_true",
        help="run the offline manifest/codebase consistency check (no writes)",
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
        "--no-preflight",
        action="store_true",
        help="skip the gh auth/scope sanity check (the API call still fails loudly if lacking)",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="run the offline pure-logic check and exit",
    )
    args = parser.parse_args()

    if args.self_test:
        self_test()
        return

    reconcile_selected = (
        args.all
        or args.labels
        or args.milestones
        or args.issues
        or args.prs
        or args.project
    )
    # --validate with no reconcile mode is a standalone, offline check.
    if args.validate and not reconcile_selected:
        sys.exit(0 if run_validation(args) else 1)

    do_labels = args.all or args.labels
    do_milestones = args.all or args.milestones
    do_issues = args.all or args.issues or not reconcile_selected
    do_prs = args.all or args.prs
    want_project = args.all or args.project

    mode = "dry-run" if args.dry_run else "live"
    print("=" * 60)
    print(f">> Mycelium PM reconcile — repo: {args.repo}  ({mode})")
    print("=" * 60)

    # Full-suite validation gate: --all (and an explicit --validate) must pass before mutating.
    if args.all or args.validate:
        if not run_validation(args):
            sys.exit(
                "ERROR: manifest validation failed — fix the manifests before reconciling (G2)."
            )
        print()

    # Auto sanity-check: proceed when auth/scopes are good; only flag what is genuinely missing.
    project_ok = False
    if args.no_preflight:
        project_ok = (
            want_project  # caller vouches; the API call fails loudly if it cannot
        )
    else:
        print(">> preflight: gh auth / scope sanity check")
        project_ok = preflight_gh(need_project=want_project)
        print()
    do_project = want_project and project_ok

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
    if do_prs:
        conventions = json.loads(args.conventions_json.read_text(encoding="utf-8"))
        defined = {
            d["name"] for d in json.loads(args.labels_json.read_text(encoding="utf-8"))
        }
        area_set = {n for n in defined if n.startswith("area:")}
        spec = yaml.safe_load(args.issues_yaml.read_text(encoding="utf-8")) or {}
        task_to_ms = build_task_to_milestone(spec.get("issues", []))
        reconcile_prs(
            args.repo, conventions, area_set, task_to_ms, dry_run=args.dry_run
        )
        print()
    if do_project:
        manifest = json.loads(args.project_json.read_text(encoding="utf-8"))
        by_number, _ = snapshot_issues(args.repo)
        contents = list(by_number.values()) + snapshot_prs(args.repo)
        reconcile_project(args.repo, manifest, contents, dry_run=args.dry_run)
        print()
    elif want_project and not project_ok:
        print(
            ">> project: SKIPPED — 'project' scope missing (remediation above); never silent.\n"
        )

    print(f">> reconcile complete ({mode}).")


if __name__ == "__main__":
    main()
