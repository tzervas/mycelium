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
def _run_gh(args, *, input_text=None):
    """Low-level: run `gh` and return ``(returncode, stdout, stderr)``. Never raises a traceback —
    a missing `gh` binary is an explicit, classified ``sys.exit`` (G2)."""
    try:
        proc = subprocess.run(
            ["gh", *args],
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
    return proc.returncode, proc.stdout, proc.stderr


def _gh_fail(args, rc, stderr):
    """Turn a `gh` failure into an EXPLICIT, classified, remediation-bearing exit — never a raw
    traceback (G2). This is the fix for the unguarded ``proc.check_returncode()`` crash that surfaced
    as a ``CalledProcessError`` on a `gh api` 401 inside reconcile_prs/reconcile_project."""
    blob = (stderr or "").strip()
    low = blob.lower()
    cmd = "gh " + " ".join(args)
    if "401" in blob or "bad credentials" in low or "requires authentication" in low:
        hint = (
            "re-authenticate — your token is missing/expired/invalid. Run `gh auth login` "
            "(or `gh auth refresh`), then re-run."
        )
    elif "rate limit" in low:
        hint = (
            "GitHub API rate limit hit — wait for the reset shown by `gh api -i rate_limit`, "
            "or authenticate with a token that has a higher limit."
        )
    elif "403" in blob and (
        "scope" in low or "saml" in low or "sso" in low or "forbidden" in low
    ):
        hint = (
            "the token lacks a required scope or SSO authorization — see the preflight EXPLAIN, "
            "or grant it with `gh auth refresh -s <scope>`."
        )
    elif (
        "could not resolve host" in low
        or "dial tcp" in low
        or "timeout" in low
        or "timed out" in low
        or "network is unreachable" in low
        or "connection refused" in low
    ):
        hint = "network error reaching GitHub — check connectivity/proxy and retry."
    else:
        hint = "see the gh stderr above; fix the cause and re-run (`gh auth status` to check auth)."
    sys.exit(
        f"ERROR: `{cmd}` failed (exit {rc}).\n"
        f"  gh: {blob or '(no stderr)'}\n"
        f"  remediation: {hint}\n"
        f"  (never-silent, G2 — an explicit error, not a Python traceback.)"
    )


def gh(args, *, input_text=None, check=True):
    """Run a `gh` subcommand and return its stdout. On a non-zero exit (when ``check``), fail with an
    explicit, classified remediation — NEVER a raw traceback (G2)."""
    rc, out, err = _run_gh(args, input_text=input_text)
    if check and rc != 0:
        _gh_fail(args, rc, err)
    return out


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
    rc, out, err = _run_gh(["auth", "status"])
    blob = (out or "") + (err or "")
    authed = rc == 0 or "Logged in to" in blob
    scopes = None
    for line in blob.splitlines():
        if "Token scopes:" in line:
            raw = line.split("Token scopes:", 1)[1]
            scopes = {tok.strip().strip("'\"") for tok in raw.split(",") if tok.strip()}
            break
    return authed, scopes


def repo_is_public(repo):
    """Best-effort: True/False/None(unknown) for ``repo`` visibility. Never raises (own gh handling).

    Drives least-privilege: a *public* target lets the write ops use the tighter ``public_repo``
    instead of the broad ``repo``. Unknown (offline / unauth / older gh) is treated conservatively
    as not-public (so we ask for ``repo`` — correctness over tightness, stated honestly).
    """
    rc, out, _ = _run_gh(
        ["repo", "view", repo, "--json", "visibility", "-q", ".visibility"]
    )
    if rc != 0:
        return None
    v = (out or "").strip().lower()
    if v == "public":
        return True
    if v in ("private", "internal"):
        return False
    return None


# ── least-privilege scope computation (PURE — no I/O; exercised offline by --self-test) ──────────
# The classic-OAuth granularity FLOOR (honest): `repo` is coarse — it spans issues/PRs/labels/
# milestones and CANNOT be narrowed further by a classic scope. A fine-grained PAT is the path to
# tighter per-resource permissions; its scopes are not enumerable here, so it is trusted to fail
# loudly at the call site (`gh()` classifies the error).
WRITE_OPS = frozenset({"labels", "milestones", "issues", "prs"})


def required_scopes(ops, *, repo_public, read_only):
    """The MINIMAL classic-OAuth scope set for the arg'd operation set ``ops``.

    - offline-only (``self-test`` / ``validate``)  -> no scope;
    - any repo write (labels/milestones/issues/prs) -> ``public_repo`` if the repo is public, else
      the broader ``repo`` (least-privilege prefers the tighter one);
    - ``project`` (board) -> ``read:project`` for a read-only/dry-run run, else ``project``.
    """
    ops = set(ops)
    scopes = set()
    if ops & WRITE_OPS:
        scopes.add("public_repo" if repo_public else "repo")
    if "project" in ops:
        scopes.add("read:project" if read_only else "project")
    return scopes


def _covers(have_scope, need_scope):
    """Whether a held scope satisfies a needed one (`repo` ⊇ `public_repo`; `project` ⊇ `read:project`)."""
    if have_scope == need_scope:
        return True
    if need_scope == "public_repo" and have_scope == "repo":
        return True
    if need_scope == "read:project" and have_scope == "project":
        return True
    return False


def missing_scopes(required, have):
    """The required scopes NOT covered by ``have`` (empty for a fine-grained token, ``have is None``)."""
    if have is None:
        return set()
    return {s for s in required if not any(_covers(h, s) for h in have)}


def over_grants(required, have):
    """Advisory notes when the token carries MORE than the computed minimum (never blocking)."""
    if have is None:
        return []
    notes = []
    if "public_repo" in required and "repo" in have:
        notes.append(
            "`repo` is broader than the `public_repo` these ops need (public target)"
        )
    if "read:project" in required and "project" in have:
        notes.append(
            "`project` is broader than the read-only `read:project` this run needs"
        )
    family = {"repo", "public_repo", "project", "read:project"}
    extras = sorted(s for s in have if s not in family and s not in required)
    for e in extras:
        notes.append(f"`{e}` is not used by this run")
    return notes


# ── auth automation (state mutation → opt-in, EXPLAIN-ed, never silent — G2) ─────────────────────
def _auth_command(required, *, authed):
    verb = "refresh" if authed else "login"
    scope_args = " ".join(f"-s {s}" for s in sorted(required))
    return f"gh auth {verb} {scope_args}".strip()


def _print_explain(ops, required, repo, *, read_only, authed):
    print(
        "   ! least-privilege auth EXPLAIN — changing a token's scopes is a state mutation, so it is "
        "opt-in and never silent (G2):"
    )
    print(
        f"       these ops  : {sorted(ops)}{' (read-only/dry-run)' if read_only else ''}"
    )
    print(
        f"       → scopes   : {sorted(required) or ['(none)']}  "
        f"(the MINIMAL classic set for these ops on {repo})"
    )
    # `login` when not authenticated (the common failure mode), `refresh` to add scopes to a token.
    print("       → command  : " + _auth_command(required, authed=authed))
    print(
        "       floor      : classic OAuth scopes are coarse — `repo` spans issues/PRs/labels/\n"
        "                    milestones and can't be narrowed further; a fine-grained PAT is the path\n"
        "                    to tighter per-resource perms (trusted to fail loudly at the call site)."
    )


def _consent(cmd):
    """Ask the user to authorize the scope change. EOF/non-interactive → declined (never assumed)."""
    print(f"   → proposed (one-time, per machine): {cmd}")
    try:
        ans = (
            input("   Run this now to grant the minimal scopes? [y/N] ").strip().lower()
        )
    except EOFError:
        return False
    return ans in ("y", "yes")


def _run_auth(required, *, authed):
    """Run `gh auth refresh/login -s <minimal-set>` interactively (inherits stdio). Explicit on fail."""
    base = ["gh", "auth", "refresh" if authed else "login"]
    for s in sorted(required):
        base += ["-s", s]
    print(f"   → running: {' '.join(base)}")
    rc = subprocess.run(base, check=False).returncode
    if rc != 0:
        sys.exit(
            f"ERROR: `{' '.join(base)}` failed (exit {rc}). Run it manually to grant the scopes "
            "(never-silent, G2)."
        )


def preflight_gh(*, ops, repo, read_only, no_auth_fix):
    """Sanity-check `gh` and enforce LEAST-PRIVILEGE before a live reconcile. Returns ``project_ok``.

    Computes the minimal scope set for the arg'd ``ops``, compares it to the active token, and — only
    when a needed scope is provably absent — EXPLAINs and (with consent, unless ``no_auth_fix``)
    automates the exact-minimal `gh auth refresh`. An over-granted token gets a non-blocking advisory.
    A fine-grained token (scopes unreadable) is trusted to fail loudly at the call site.
    """
    want_project = "project" in ops
    authed, scopes = get_gh_scopes()
    repo_public = repo_is_public(repo) if (ops & WRITE_OPS) else None
    required = required_scopes(
        ops, repo_public=(repo_public is True), read_only=read_only
    )

    if not authed:
        if not required:
            print(
                "   = gh: not authenticated, but the arg'd ops need no scope — proceeding offline"
            )
            return False
        return _remediate(
            required,
            ops,
            repo,
            read_only,
            no_auth_fix,
            authed=False,
            have=None,
            want_project=want_project,
        )

    print("   = gh: authenticated")
    if not required:
        print("   = gh: arg'd ops are offline/read-only — no scope required")
        return want_project
    if scopes is None:
        print(
            "   ~ gh token scopes not enumerable (fine-grained PAT?) — trusting; a missing permission "
            "fails loudly at the call site. (fine-grained PATs give tighter per-resource perms.)"
        )
        return want_project

    for note in over_grants(required, scopes):
        print(f"   ~ least-privilege advisory: {note}")

    miss = missing_scopes(required, scopes)
    if not miss:
        print(f"   = gh: token scopes satisfy the computed minimum {sorted(required)}")
        return want_project
    return _remediate(
        required,
        ops,
        repo,
        read_only,
        no_auth_fix,
        authed=True,
        have=scopes,
        want_project=want_project,
    )


def _remediate(
    required, ops, repo, read_only, no_auth_fix, *, authed, have, want_project
):
    """Handle a missing-scope situation: EXPLAIN, then automate (with consent) or exit/degrade.

    Missing a *repo* scope blocks the selected write ops (hard exit). Missing only *project* degrades
    gracefully (skip the board) — preserving the original never-silent behaviour.
    """
    miss = missing_scopes(required, have) if authed else set(required)
    repo_blocked = bool({"repo", "public_repo"} & miss) or (
        not authed and bool(set(ops) & WRITE_OPS)
    )
    cmd = _auth_command(required, authed=authed)
    _print_explain(ops, required, repo, read_only=read_only, authed=authed)

    if no_auth_fix:
        head = f"   ! missing gh scope(s) {sorted(miss)} and --no-auth-fix is set — not prompting."
        if repo_blocked or not authed:
            sys.exit(
                f"{head}\n     remediation: {cmd}\n  (cannot proceed; never-silent, G2.)"
            )
        print(
            f"{head}\n     remediation: {cmd}\n     proceeding without --project (board skipped); "
            "never silent.",
            file=sys.stderr,
        )
        return False

    if not _consent(cmd):
        if repo_blocked or not authed:
            sys.exit(
                f"   ! consent declined — cannot proceed without {sorted(miss)}. "
                f"Run manually: {cmd}  (never-silent, G2.)"
            )
        print(
            "   ! consent declined — proceeding without --project (board skipped); never silent.",
            file=sys.stderr,
        )
        return False

    _run_auth(required, authed=authed)
    authed2, scopes2 = get_gh_scopes()
    if not authed2 or missing_scopes(required, scopes2):
        sys.exit(
            f"   ! auth did not grant the required scopes {sorted(required)}. Run manually: {cmd} "
            "(never-silent, G2)."
        )
    print(f"   = gh: scopes now satisfy the computed minimum {sorted(required)}")
    return want_project


# ─────────────────────────────────────────────────────────────────────────────────────────────
# Pure reconcile logic (no I/O — exercised offline by --self-test)
# ─────────────────────────────────────────────────────────────────────────────────────────────
def label_delta(desired, actual):
    """Return (to_add, to_remove) as sorted lists so a label set converges to ``desired``."""
    desired, actual = set(desired), set(actual)
    return sorted(desired - actual), sorted(actual - desired)


def plan_label_migrations(repo_label_names, canonical_names, aliases):
    """Plan noncompliant-label migrations from the repo's live label set.

    Pure (no I/O): returns (migrations, flags) where:
    - ``migrations`` is a sorted list of (old_name, new_name) for noncompliant labels that have a
      declared alias;
    - ``flags`` is a sorted list of noncompliant label names with NO declared alias (they must be
      handled manually — never silently deleted, G2).

    A label is COMPLIANT when its name appears in ``canonical_names`` (the labels.json set). A
    noncompliant label is either MIGRATED (has an alias) or FLAGGED (has no alias). An alias that
    maps to a name not in ``canonical_names`` is itself flagged (the target must be declared).
    """
    canonical = set(canonical_names)
    noncompliant = [n for n in repo_label_names if n not in canonical]
    migrations, flags = [], []
    for name in noncompliant:
        if name in aliases:
            target = aliases[name]
            if target in canonical:
                migrations.append((name, target))
            else:
                # alias target is not a canonical label — flag it, never guess (G2)
                flags.append(
                    f"{name} (alias -> '{target}' is not in labels.json; add it first)"
                )
        else:
            flags.append(name)
    return sorted(migrations), sorted(flags)


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


def milestone_rank(title):
    """Return the phase number (int) from a 'Phase N' prefix, or -1 for unprefixed titles.

    Pure: parses the leading 'Phase N' token only; everything after the first word boundary is
    ignored. Used to resolve multi-milestone spans deterministically to the highest-phase
    (most advanced) milestone. Examples: 'Phase 8 — Toolchain' -> 8; 'Backlog' -> -1.
    """
    m = re.match(r"Phase\s+(\d+)", title or "", re.IGNORECASE)
    return int(m.group(1)) if m else -1


def infer_milestone(task_ids, task_to_ms):
    """Infer a single milestone from referenced task-ids.

    - Exactly one milestone spanned → return (milestone, None).
    - No milestone found            → return (None, None) — silent, no claim made.
    - Multiple milestones           → resolve to the HIGHEST-phase milestone (milestone_rank);
                                      return (chosen, note_string) where the note is
                                      informational, not a blocking flag — callers that check
                                      truthiness of the second return value now get a milestone
                                      AND a note (not a refusal).
    """
    milestones = {task_to_ms[t] for t in task_ids if t in task_to_ms}
    if len(milestones) == 1:
        return next(iter(milestones)), None
    if not milestones:
        return None, None  # nothing to infer from — silent (no claim made)
    # Multi-milestone: resolve deterministically to the highest-phase milestone.
    chosen = max(milestones, key=milestone_rank)
    note = f"note: spanned {sorted(milestones)} -> chose highest-phase '{chosen}'"
    return chosen, note


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
def _load_label_aliases(here):
    """Load label-aliases.json from ``here``, returning the aliases dict.

    Tolerates the file being absent — returns an empty dict so all noncompliant labels are
    flagged (never-silent G2) rather than silently ignored.
    """
    aliases_path = here / "label-aliases.json"
    if not aliases_path.exists():
        print(
            "   ~ label-aliases.json not found — all noncompliant labels will be FLAGGED "
            "(add label-aliases.json to declare migrations)",
            file=sys.stderr,
        )
        return {}
    data = json.loads(aliases_path.read_text(encoding="utf-8"))
    return {k: v for k, v in data.get("aliases", {}).items()}


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

    # ── noncompliant-label reconcile (runs after create-or-update) ───────────────────────────
    # Snapshot the repo's actual labels and compare against labels.json. Noncompliant labels
    # with a declared alias are migrated (relabel every open+closed issue/PR, then delete the
    # stale label). Noncompliant labels without an alias are FLAGGED — never silently deleted (G2).
    aliases = _load_label_aliases(HERE)
    canonical_names = {lb["name"] for lb in labels}
    raw_repo_labels = json.loads(
        gh(["api", "--paginate", f"repos/{repo}/labels"])
    )
    repo_label_names = [lb["name"] for lb in raw_repo_labels]
    migrations, flags = plan_label_migrations(repo_label_names, canonical_names, aliases)

    if not migrations and not flags:
        print("   = noncompliant labels: none found (repo labels match labels.json)")
    else:
        if migrations:
            print(f">> noncompliant-label migrations: {len(migrations)} to migrate")
        for old_name, new_name in migrations:
            # Find every open + closed issue/PR carrying the stale label.
            raw_issues = json.loads(
                gh(
                    [
                        "api",
                        "--paginate",
                        f"repos/{repo}/issues?state=all&per_page=100&labels={old_name}",
                    ]
                )
            )
            numbers = [item["number"] for item in raw_issues]
            if dry_run:
                print(
                    f"   ~ would migrate label '{old_name}' -> '{new_name}' "
                    f"({len(numbers)} issue(s)/PR(s)), then delete '{old_name}'"
                )
            else:
                for number in numbers:
                    gh(
                        [
                            "issue",
                            "edit",
                            str(number),
                            "--repo",
                            repo,
                            "--add-label",
                            new_name,
                            "--remove-label",
                            old_name,
                        ]
                    )
                    print(f"   • #{number}: label '{old_name}' -> '{new_name}'")
                gh(["label", "delete", old_name, "--repo", repo, "--yes"])
                print(f"   • deleted stale label '{old_name}'")

        for flag in flags:
            # Never-silent: a noncompliant label with no alias is left untouched and reported (G2).
            print(
                f"   ! noncompliant label '{flag}' has no declared alias in label-aliases.json "
                f"— left untouched; add an alias or delete it manually",
                file=sys.stderr,
            )


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

    # milestone inference: unambiguous → set; multi → highest-phase + note; none → silent.
    t2m = {"M-150": "Phase 1", "M-151": "Phase 1", "M-201": "Phase 2"}
    assert extract_task_ids("does M-150 and M-151", ["M-[0-9]+"]) == ["M-150", "M-151"]
    assert infer_milestone(["M-150", "M-151"], t2m) == ("Phase 1", None)
    assert infer_milestone([], t2m) == (None, None)
    # Multi-milestone: resolves to highest-phase, returns informational note (not a blocking flag).
    ms, note = infer_milestone(["M-150", "M-201"], t2m)
    assert ms == "Phase 2", f"expected 'Phase 2', got {ms!r}"
    assert note and note.startswith("note:") and "Phase 2" in note, f"unexpected note: {note!r}"
    # milestone_rank: Phase N prefix → N (int); unprefixed → -1.
    assert milestone_rank("Phase 8 — Toolchain & Release Engineering") == 8
    assert milestone_rank("Phase 0") == 0
    assert milestone_rank("Backlog") == -1
    assert milestone_rank("") == -1
    assert milestone_rank(None) == -1
    # Highest-phase wins even when it's not the lexicographically last name.
    t2m_mixed = {"A": "Phase 10 — Future", "B": "Phase 2 — Now", "C": "Backlog"}
    ms2, note2 = infer_milestone(["A", "B"], t2m_mixed)
    assert ms2 == "Phase 10 — Future" and note2 and "Phase 10" in note2, (ms2, note2)
    # All unprefixed → highest by rank (-1 tie) → max() picks deterministically (first in sort).
    ms3, note3 = infer_milestone(["B", "C"], t2m_mixed)
    assert ms3 is not None and note3 and note3.startswith("note:"), (ms3, note3)

    # ── plan_label_migrations (pure, offline) ────────────────────────────────────────────────
    canonical = {"type:bug", "type:feature", "type:docs", "good-first-issue", "area:swap"}
    aliases = {
        "bug": "type:bug",
        "enhancement": "type:feature",
        "documentation": "type:docs",
        "good first issue": "good-first-issue",
    }
    # compliant labels → no migrations, no flags.
    migs, flgs = plan_label_migrations(
        ["type:bug", "area:swap"], canonical, aliases
    )
    assert migs == [] and flgs == [], (migs, flgs)
    # mix: two aliased + one unaliased noncompliant.
    migs, flgs = plan_label_migrations(
        ["type:bug", "bug", "enhancement", "orphan-label"], canonical, aliases
    )
    assert migs == [("bug", "type:bug"), ("enhancement", "type:feature")], migs
    assert flgs == ["orphan-label"], flgs
    # alias pointing to a non-canonical target → appears in flags, not migrations.
    bad_aliases = {"stale": "nonexistent-canonical"}
    migs2, flgs2 = plan_label_migrations(["stale"], canonical, bad_aliases)
    assert migs2 == [], migs2
    assert any("nonexistent-canonical" in f for f in flgs2), flgs2
    # empty repo labels → no migrations, no flags (idempotent on an already-clean repo).
    assert plan_label_migrations([], canonical, aliases) == ([], [])
    # all canonical → no output.
    assert plan_label_migrations(list(canonical), canonical, aliases) == ([], [])

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

    # ── least-privilege scope computation (pure, offline) ───────────────────────────────────────
    # offline-only ops need nothing.
    assert required_scopes(set(), repo_public=True, read_only=False) == set()
    assert required_scopes({"validate"}, repo_public=False, read_only=False) == set()
    # a public-repo write prefers the tighter `public_repo`; a private one needs broad `repo`.
    assert required_scopes({"issues"}, repo_public=True, read_only=False) == {
        "public_repo"
    }
    assert required_scopes({"labels", "prs"}, repo_public=False, read_only=False) == {
        "repo"
    }
    # the board needs `read:project` when read-only, `project` when it will write.
    assert required_scopes({"project"}, repo_public=True, read_only=True) == {
        "read:project"
    }
    assert required_scopes(
        {"issues", "project"}, repo_public=True, read_only=False
    ) == {
        "public_repo",
        "project",
    }
    # coverage: `repo` ⊇ `public_repo`; `project` ⊇ `read:project`.
    assert missing_scopes({"public_repo"}, {"repo"}) == set()
    assert missing_scopes({"read:project"}, {"project"}) == set()
    assert missing_scopes({"public_repo", "project"}, {"repo"}) == {"project"}
    assert missing_scopes({"repo"}, {"public_repo"}) == {
        "repo"
    }  # public_repo does NOT cover repo
    # a fine-grained token (None) is trusted: nothing reported missing.
    assert missing_scopes({"repo", "project"}, None) == set()
    # over-grant advisories: broader-than-needed + entirely-unused scopes; none when exact.
    assert over_grants({"public_repo"}, {"repo"})  # repo broader than public_repo
    assert any("read-only" in n for n in over_grants({"read:project"}, {"project"}))
    assert any(
        "gist" in n for n in over_grants({"public_repo"}, {"public_repo", "gist"})
    )
    assert over_grants({"public_repo", "project"}, {"public_repo", "project"}) == []
    assert over_grants({"repo"}, None) == []
    # the auth command is the exact-minimal set, refresh vs login by auth state.
    assert _auth_command({"public_repo", "project"}, authed=True) == (
        "gh auth refresh -s project -s public_repo"
    )
    assert _auth_command({"repo"}, authed=False) == "gh auth login -s repo"

    print(
        "self-test OK: label_delta, normalize_body, plan_issue_update, parse_conventional, "
        "derive_pr_labels, milestone_rank, infer_milestone (multi-milestone), "
        "plan_label_migrations, label_to_field_values, plan_field_reconcile, "
        "required_scopes, missing_scopes, over_grants, _auth_command"
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
        "--no-auth-fix",
        action="store_true",
        help="never prompt to change gh scopes (CI/non-interactive); a missing scope is an explicit "
        "error or a flagged board-skip, never an interactive prompt",
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

    # The arg'd operation set drives the least-privilege scope computation in preflight.
    ops = set()
    if do_labels:
        ops.add("labels")
    if do_milestones:
        ops.add("milestones")
    if do_issues:
        ops.add("issues")
    if do_prs:
        ops.add("prs")
    if want_project:
        ops.add("project")

    # Auto sanity-check + least-privilege enforcement: proceed when scopes match the computed
    # minimum; EXPLAIN + (consented) automate a refresh only for a genuinely-absent needed scope.
    project_ok = False
    if args.no_preflight:
        project_ok = (
            want_project  # caller vouches; the API call fails loudly if it cannot
        )
    else:
        print(">> preflight: gh auth + least-privilege scope check")
        project_ok = preflight_gh(
            ops=ops,
            repo=args.repo,
            read_only=args.dry_run,
            no_auth_fix=args.no_auth_fix,
        )
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
    # Top-level guard: a `gh` failure already exits explicitly via _gh_fail; this catches anything
    # else so the user never sees a raw Python traceback (G2 — every failure is an explicit message).
    try:
        main()
    except KeyboardInterrupt:
        sys.exit("\nERROR: interrupted by user.")
    except SystemExit:
        raise
    except Exception as exc:  # pragma: no cover - last-resort guard, never a traceback
        sys.exit(
            f"ERROR: unexpected failure: {type(exc).__name__}: {exc}\n"
            "  (this should have been an explicit, classified error — please report it.)"
        )
