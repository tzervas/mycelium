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
    python tools/github/gh-issues-sync.py --all --verbose   # echo each `gh` call (pinpoint a hang)

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
import threading
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from urllib.parse import quote

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

# Module-level flag (set by main from --verbose). When True, every `gh` invocation is echoed to
# stderr BEFORE it runs, so a hang is pinpointable to the exact call (M-382 diagnosability).
VERBOSE = False
DEBUG = False  # --debug: show full tracebacks on unexpected failure (set in main())

# Bounded retry policy for transient network blips on a `gh` call (M-382). MAX retries (so up to
# MAX+1 attempts total: the initial call + MAX retries); the backoff delays below are applied
# BEFORE each retry (1s, 2s, 4s, 8s — exponential).
_GH_RETRY_MAX = 4
_GH_RETRY_DELAYS = (1, 2, 4, 8)
_GH_TIMEOUT = 120  # per-call ceiling so a HUNG gh request can never block forever (M-382 follow-up)

# A process-wide lock so concurrent worker prints (and --verbose `gh` echoes) never interleave
# under the M-397 thread pool. Defined here (before _run_gh) so the VERBOSE echo can use it; the
# public ``safe_print`` wrapper lives in the concurrency section below.
_PRINT_LOCK = threading.Lock()


def _safe_stderr(msg):
    """Lock-guarded single-line stderr print (interleave-safe under the M-397 pool)."""
    with _PRINT_LOCK:
        print(msg, file=sys.stderr)


def _is_transient_network(stderr) -> bool:
    """PURE: True iff ``stderr`` looks like a *transient* network failure that is worth retrying
    (no I/O — exercised offline by --self-test).

    This is the EOF-aware classifier at the heart of the M-382 fix: the original symptom
    ``gh: Post "...": unexpected EOF`` was not recognized as a network error, so it fell through to
    the non-retryable generic branch and aborted the whole paginated sync. We recognize EOF, TCP
    reset, TLS-handshake, and I/O-timeout failures in addition to the DNS/dial/timeout set.
    """
    low = (stderr or "").lower()
    needles = (
        # EOF / reset / handshake / io-timeout class (the M-382 additions)
        "unexpected eof",
        "eof",
        "connection reset",
        "reset by peer",
        "tls handshake",
        "i/o timeout",
        # the pre-existing DNS / dial / timeout / unreachable set
        "could not resolve host",
        "dial tcp",
        "timeout",
        "timed out",
        "network is unreachable",
        "connection refused",
    )
    return any(n in low for n in needles)


def _stderr_tail(stderr) -> str:
    """Last non-empty line of a gh stderr blob (compact, for one-line never-silent messages)."""
    lines = [ln for ln in (stderr or "").splitlines() if ln.strip()]
    return lines[-1].strip() if lines else "(no stderr)"


def _run_gh(args, *, input_text=None):
    """Low-level: run `gh` and return ``(returncode, stdout, stderr)``. Never raises a traceback —
    a missing `gh` binary is an explicit, classified ``sys.exit`` (G2).

    Centralizes the M-382 fix for EVERY gh call (pagination, edits, graphql): on a transient
    network failure (``_is_transient_network``) it retries with bounded exponential backoff
    (``_GH_RETRY_MAX`` retries; 1/2/4/8s delays), printing a never-silent line per retry. A
    non-transient failure (or success) returns immediately; after the last retry it returns the
    final ``(rc, out, err)`` so ``gh()``/``_gh_fail`` still produce the classified error.
    """
    rc, out, err = 0, "", ""
    # attempt 0 = the initial call; attempts 1.._GH_RETRY_MAX = the bounded retries.
    for attempt in range(0, _GH_RETRY_MAX + 1):
        if VERBOSE:
            _safe_stderr(f"   → gh {' '.join(args)}")
        try:
            proc = subprocess.run(
                ["gh", *args],
                check=False,
                text=True,
                input=input_text,
                capture_output=True,
                timeout=_GH_TIMEOUT,
            )
            rc, out, err = proc.returncode, proc.stdout, proc.stderr
        except FileNotFoundError:  # pragma: no cover - environment guard
            sys.exit(
                "ERROR: `gh` (GitHub CLI) not found on PATH. Install it and run `gh auth login` "
                "(Windows: `winget install GitHub.cli`)."
            )
        except subprocess.TimeoutExpired:
            # A hung call (e.g. a malformed request that never returns) is turned into a
            # classified TRANSIENT failure so the retry/backoff handles it — never an infinite
            # block. ('timed out' is recognized by _is_transient_network.)
            rc, out, err = 124, "", f"gh call timed out after {_GH_TIMEOUT}s"
        # Success, or a non-transient failure: return immediately (no retry).
        if rc == 0 or not _is_transient_network(err):
            return rc, out, err
        # Transient failure with retries remaining: never-silent retry + backoff.
        if attempt < _GH_RETRY_MAX:
            delay = _GH_RETRY_DELAYS[attempt]
            _safe_stderr(
                f"   ~ transient network error (gh exit {rc}) — "
                f"retry {attempt + 1}/{_GH_RETRY_MAX} in {delay}s: {_stderr_tail(err)}"
            )
            time.sleep(delay)
    # Exhausted: hand back the last classified failure for gh()/_gh_fail to surface.
    return rc, out, err


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
    elif _is_transient_network(blob):
        hint = (
            "network error reaching GitHub — check connectivity/proxy and retry "
            "(this was already retried with backoff; the blip persisted)."
        )
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


# ─────────────────────────────────────────────────────────────────────────────────────────────
# Bounded-concurrency, rate-negotiated, batched execution (M-397)
# ─────────────────────────────────────────────────────────────────────────────────────────────
# The reconcilers issue many small, independent, idempotent `gh` mutations per run (a label per
# label, a field-update per issue, …). They are subprocess/IO-bound, so a bounded thread pool over
# the EXISTING synchronous `gh()` (NOT an asyncio rewrite) overlaps the latency without changing a
# single call's behaviour — `gh()`/`_run_gh` keep their retry/backoff/120s-timeout verbatim.
#
# Three layers keep us inside GitHub's limits (never-silent, G2):
#   1. bounded concurrency — at most ``--concurrency N`` (default 6) calls in flight;
#   2. a shared PAUSE gate — on a secondary-rate-limit / abuse / 403 / 429 / Retry-After signal we
#      pause the WHOLE pool for the advised window, then resume (never keep bursting);
#   3. an optional start-of-run budget probe (``gh api rate_limit``) that reduces N when the
#      remaining core budget is low — never-silently.
#
# N=1 reproduces today's exact sequential behaviour (a clean fallback for --verbose / debugging):
# ``run_batch`` then runs each task inline, in order, with no executor and no extra threads.

_DEFAULT_CONCURRENCY = 6


# A run-scoped pause gate shared by every worker thread. When ``resume`` is set (the steady state)
# workers proceed; on a secondary-rate-limit signal a worker clears it, sleeps the advised window,
# and sets it again — pausing the whole pool for the duration. Guarded by a lock so only ONE worker
# performs the sleep for a given burst (the rest just wait on the Event).
class RateGate:
    """Shared, thread-safe pause gate for the worker pool (never-silent on every pause)."""

    def __init__(self):
        self._resume = threading.Event()
        self._resume.set()  # steady state: not paused
        self._lock = threading.Lock()

    def wait(self):
        """Block while the pool is paused; return immediately in the steady state."""
        self._resume.wait()

    def pause(self, seconds, *, reason):
        """Pause the whole pool for ``seconds`` (bounded). Only the first worker into the lock for a
        given burst actually sleeps; concurrent callers coalesce onto the same window (G2: printed
        once, under the print lock)."""
        # Try to become THE pauser for this burst. If another worker already holds the lock we are
        # mid-pause already — just wait on the event rather than stacking a second sleep.
        if not self._lock.acquire(blocking=False):
            self._resume.wait()
            return
        try:
            self._resume.clear()
            safe_print(
                f"   ~ rate-limit pause: sleeping {seconds}s before resuming ({reason})",
                file=sys.stderr,
            )
            time.sleep(seconds)
        finally:
            self._resume.set()
            self._lock.release()


def safe_print(*args, **kwargs):
    """``print`` guarded by a global lock so concurrent/`--verbose` output never interleaves (G2)."""
    with _PRINT_LOCK:
        print(*args, **kwargs)


# secondary-rate-limit / abuse needles + a Retry-After parser — PURE, exercised offline by --self-test.
_SECONDARY_LIMIT_NEEDLES = (
    "secondary rate limit",
    "abuse detection",
    "abuse",
    "you have exceeded a secondary rate limit",
    "have triggered an abuse",
    "retry your request again later",
)
_RETRY_AFTER_RE = re.compile(r"retry[- ]after[:\s]+(\d+)", re.IGNORECASE)


def should_pause_for_rate_limit(stderr, *, default_backoff=60, max_backoff=300):
    """PURE: decide whether a gh failure's ``stderr`` indicates a secondary-rate-limit / abuse stop,
    and for how long to pause the whole pool (no I/O — exercised offline by --self-test).

    Returns ``(should_pause: bool, seconds: int)``. We treat as a pause-worthy signal: an explicit
    ``secondary rate limit`` / ``abuse`` message, OR an HTTP ``403``/``429`` (GitHub returns 403 for
    secondary limits and 429 for some abuse cases). The pause length is the ``Retry-After`` header
    value when present (clamped to ``max_backoff``), else ``default_backoff``.

    NOTE (honesty): a *primary* rate-limit (``API rate limit exceeded``) is NOT handled here — it
    resets on a fixed hourly window, not a short pause, and ``_gh_fail`` already surfaces it with the
    ``gh api -i rate_limit`` remediation. We only negotiate the *secondary*/abuse pause-and-resume.
    """
    blob = stderr or ""
    low = blob.lower()
    # A primary rate-limit is a different beast — do not absorb it into a short pause (stay honest).
    if "api rate limit exceeded" in low:
        return False, 0
    secondary = any(n in low for n in _SECONDARY_LIMIT_NEEDLES)
    http_throttle = ("403" in blob) or ("429" in blob)
    if not (secondary or http_throttle):
        return False, 0
    m = _RETRY_AFTER_RE.search(blob)
    if m:
        seconds = int(m.group(1))
    else:
        seconds = default_backoff
    seconds = max(1, min(seconds, max_backoff))
    return True, seconds


def parse_rate_remaining(rate_limit_json):
    """PURE: extract the core ``remaining`` budget from a ``gh api rate_limit`` JSON blob, or None.

    Tolerates the shapes ``{"resources":{"core":{"remaining":N}}}`` and a bare ``{"rate":{...}}``
    (older API) — returns None when neither is present so the caller degrades never-silently.
    """
    if not isinstance(rate_limit_json, dict):
        return None
    core = (rate_limit_json.get("resources") or {}).get("core")
    if isinstance(core, dict) and isinstance(core.get("remaining"), int):
        return core["remaining"]
    rate = rate_limit_json.get("rate")
    if isinstance(rate, dict) and isinstance(rate.get("remaining"), int):
        return rate["remaining"]
    return None


def negotiate_concurrency(requested, remaining, *, low_water=100):
    """PURE: reduce the requested worker count when the remaining core budget is low (never-silent
    decision; the caller prints when it differs). ``remaining is None`` (probe failed/offline) ⇒
    trust the request unchanged. Below ``low_water`` ⇒ clamp to 1 (serialize, safest). Returns the
    effective worker count (always ≥ 1)."""
    n = max(1, int(requested))
    if remaining is None:
        return n
    if remaining < low_water:
        return 1
    return n


def aggregate_results(results):
    """PURE: split a list of ``(item, ok, err)`` task results into (ok_count, fail_count, failures),
    where ``failures`` is the ``[(item, err), …]`` sublist of the non-ok ones (order preserved).
    The single place batch summaries are computed (exercised offline by --self-test)."""
    ok_count = sum(1 for _item, ok, _err in results if ok)
    failures = [(item, err) for item, ok, err in results if not ok]
    return ok_count, len(failures), failures


# The run-scoped rate gate + effective worker count, set once by main() before any batch runs.
RATE_GATE = RateGate()
CONCURRENCY = 1


def run_gh_task(args, *, input_text=None):
    """Run one `gh` mutation inside a worker: honour the shared pause gate BEFORE the call, run the
    existing ``_run_gh`` (retry/backoff/timeout unchanged), and if it fails with a secondary-rate-
    limit / abuse / 403 / 429 signal, PAUSE THE WHOLE POOL for the advised window (never-silent)
    then retry ONCE. Returns ``(rc, out, err)`` exactly like ``_run_gh`` so callers are unchanged."""
    RATE_GATE.wait()
    rc, out, err = _run_gh(args, input_text=input_text)
    if rc != 0:
        pause, seconds = should_pause_for_rate_limit(err)
        if pause:
            RATE_GATE.pause(seconds, reason=_stderr_tail(err))
            # One post-pause retry: the burst that tripped the limit is now spread out.
            rc, out, err = _run_gh(args, input_text=input_text)
    return rc, out, err


def run_batch(name, tasks, *, concurrency=None, summary=True, return_results=False):
    """Run a batch of independent ``tasks`` (callables returning ``(item, ok, err)``) with bounded
    concurrency, never-silently, fault-tolerant + ordered-summary aggregation (M-397).

    - ``concurrency=1`` (or the module ``CONCURRENCY`` being 1) ⇒ EXACT sequential behaviour: each
      task runs inline, in submission order, with no executor — the clean debugging fallback.
    - otherwise a bounded ``ThreadPoolExecutor(max_workers=N)`` runs them concurrently; results are
      collected in SUBMISSION order (deterministic summary/failures). One task raising/failing NEVER
      aborts the batch — its failure (incl. a stray ``SystemExit``) is captured.
    - returns ``(ok_count, fail_count, failures)`` (the pure ``aggregate_results`` split), or the
      full ``[(item, ok, err)]`` results list when ``return_results`` is set (the create pass needs
      the ok payloads). When ``summary`` is set, prints a ``>> <name>: N ok, M failed`` line (G2)."""
    n = concurrency if concurrency is not None else CONCURRENCY
    n = max(1, int(n))
    tasks = list(tasks)
    results = []
    if n == 1 or len(tasks) <= 1:
        # Sequential fallback — byte-for-byte today's ordering/behaviour.
        for task in tasks:
            results.append(_run_one_task(task))
    else:
        with ThreadPoolExecutor(max_workers=n) as pool:
            futures = [pool.submit(_run_one_task, task) for task in tasks]
            # Collect in SUBMISSION order (not completion order) so the aggregation + failure
            # report are deterministic. Each .result() blocks until that task is done; all run
            # concurrently (already submitted). Live per-item progress still prints as tasks finish.
            results = [fut.result() for fut in futures]
    ok_count, fail_count, failures = aggregate_results(results)
    if summary:
        safe_print(f">> {name}: {ok_count} ok, {fail_count} failed")
    if return_results:
        return results
    return ok_count, fail_count, failures


def _run_one_task(task):
    """Run a single batch task, turning ANY exception into a captured ``(item, ok, err)`` failure so
    one bad task never aborts the batch (build on the existing migration fault-tolerance)."""
    try:
        return task()
    except SystemExit as exc:
        # A task that hit _gh_fail/sys.exit (its remediation is already printed) would, in a worker
        # thread, bubble out of fut.result() and abort the batch — so capture it as a failure to
        # honour 'one failure never aborts a batch' (G2; never-silent in the aggregation below).
        return (getattr(task, "item", "?"), False, f"task exited: {exc}")
    except (
        Exception
    ) as exc:  # pragma: no cover - defensive: a task should return, not raise
        return (getattr(task, "item", "?"), False, f"{type(exc).__name__}: {exc}")


def probe_rate_budget(requested):
    """Optionally read ``gh api rate_limit`` at start and reduce N if the remaining budget is low.

    Never-silent: prints the probed budget and any reduction. A failed/declined probe (offline,
    unauth, older gh) degrades to the requested N unchanged (``parse_rate_remaining`` ⇒ None)."""
    rc, out, _err = _run_gh(["api", "rate_limit"])
    remaining = None
    if rc == 0:
        try:
            remaining = parse_rate_remaining(json.loads(out))
        except (ValueError, TypeError):
            remaining = None
    effective = negotiate_concurrency(requested, remaining)
    if remaining is None:
        print(
            f"   ~ rate budget unknown (probe unavailable) — using --concurrency {effective}"
        )
    elif effective != requested:
        print(
            f"   ~ rate budget low ({remaining} core remaining) — reducing concurrency "
            f"{requested} -> {effective} (never-silent)"
        )
    else:
        print(
            f"   = rate budget OK ({remaining} core remaining) — concurrency {effective}"
        )
    return effective


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
# Token-based GitHub API (gh-CLI-INDEPENDENT REST + GraphQL/Projects-v2 path) — M-RELS
# ─────────────────────────────────────────────────────────────────────────────────────────────
# The `gh()` plumbing above shells to the `gh` CLI. Some environments (CI runners, the agent
# sandbox here) have **no `gh` CLI**, only a token in the environment. This client gives the same
# capabilities — REST for issues/PRs/dependencies/sub-issues, GraphQL for the Projects v2 board —
# over stdlib ``urllib`` reading ``GITHUB_TOKEN``/``GH_TOKEN``. No new dependency (RECONCILE.md
# rule), never-silent (G2: a non-2xx raises with the status + body), and it is OPT-IN (``--use-api``
# / auto when `gh` is absent) so the default `gh`-driven behaviour is unchanged.
#
# Honesty (Declared): the **live Projects-v2 GraphQL mutation** path is wired but, like the `gh`
# GraphQL path, is **Declared until run against the live API with a `project`-scoped token** — the
# pure request-construction below is unit-tested offline (``--self-test``); the network round-trip
# is not exercised here (no token in this environment). See project-v2-spec.md.

import os  # noqa: E402  (kept local to this gh-independent section for clarity)
import urllib.error  # noqa: E402
import urllib.request  # noqa: E402

_GITHUB_API = "https://api.github.com"
_GITHUB_GRAPHQL = "https://api.github.com/graphql"
_API_TIMEOUT = (
    60  # per-request ceiling (mirrors the gh path's bounded timeout discipline)
)


def github_token():
    """Return the GitHub token from the environment, or None. Never stored; read at call time (G2).

    Honours ``GITHUB_TOKEN`` then ``GH_TOKEN`` (the two conventional names). Returning None lets the
    caller FLAG the token-gated step rather than fabricate a sync — the honest failure mode.
    """
    return os.environ.get("GITHUB_TOKEN") or os.environ.get("GH_TOKEN")


class GitHubApi:
    """Minimal, never-silent GitHub REST+GraphQL client over urllib (no `gh`, no new dependency).

    A failed request raises ``RuntimeError`` carrying the HTTP status + response body (G2 — never a
    silent swallow). ``dry_run`` short-circuits every **mutating** verb (POST/PATCH/PUT/DELETE) to a
    printed, no-write preview; GET always runs (reads are side-effect-free).
    """

    def __init__(self, token, *, dry_run=False):
        if not token:
            raise RuntimeError(
                "GitHubApi requires a token (GITHUB_TOKEN / GH_TOKEN); none in the environment. "
                "This is the honest token-gated stop — no live sync is fabricated (G2)."
            )
        self._token = token
        self.dry_run = dry_run

    def _headers(self, *, graphql=False):
        h = {
            "Authorization": f"Bearer {self._token}",
            "Accept": "application/vnd.github+json",
            "X-GitHub-Api-Version": "2022-11-28",
            "User-Agent": "mycelium-gh-issues-sync",
        }
        if graphql:
            h["Accept"] = "application/json"
        return h

    def request(self, method, path, *, body=None):
        """Issue a REST request. ``path`` is API-relative (``/repos/...``) or absolute. Returns the
        parsed JSON (or {} on 204). Mutating verbs honour ``dry_run`` (printed, no write)."""
        url = path if path.startswith("http") else f"{_GITHUB_API}{path}"
        mutating = method.upper() not in ("GET", "HEAD")
        if mutating and self.dry_run:
            safe_print(
                f"   [dry-run] {method} {url}  body={json.dumps(body) if body else '{}'}"
            )
            return {}
        data = json.dumps(body).encode() if body is not None else None
        req = urllib.request.Request(
            url, data=data, method=method.upper(), headers=self._headers()
        )
        if data is not None:
            req.add_header("Content-Type", "application/json")
        try:
            with urllib.request.urlopen(req, timeout=_API_TIMEOUT) as resp:
                raw = resp.read().decode()
                return json.loads(raw) if raw.strip() else {}
        except urllib.error.HTTPError as e:  # never-silent: surface status + body (G2)
            detail = e.read().decode(errors="replace") if hasattr(e, "read") else ""
            raise RuntimeError(
                f"GitHub REST {method} {url} -> HTTP {e.code}: {detail[:500]}"
            )
        except urllib.error.URLError as e:
            raise RuntimeError(f"GitHub REST {method} {url} -> network error: {e}")

    def paginate(self, path, *, params=None):
        """GET every page of a list endpoint, following the RFC-5988 ``Link: rel=next`` header.

        Yields each item across pages. ``params`` is a dict merged into the query string (``per_page``
        defaults to 100). Pure-ish: GET only, no mutation.
        """
        from urllib.parse import urlencode

        q = dict(params or {})
        q.setdefault("per_page", 100)
        url = (
            f"{_GITHUB_API}{path}?{urlencode(q)}"
            if not path.startswith("http")
            else path
        )
        while url:
            req = urllib.request.Request(url, method="GET", headers=self._headers())
            try:
                with urllib.request.urlopen(req, timeout=_API_TIMEOUT) as resp:
                    page = json.loads(resp.read().decode() or "[]")
                    link = resp.headers.get("Link", "") or ""
            except urllib.error.HTTPError as e:
                detail = e.read().decode(errors="replace") if hasattr(e, "read") else ""
                raise RuntimeError(
                    f"GitHub REST GET {url} -> HTTP {e.code}: {detail[:500]}"
                )
            for item in page:
                yield item
            url = _next_link(link)

    def graphql(self, query, variables=None):
        """POST a GraphQL query/mutation (the Projects v2 path). Returns ``data``; raises on errors.

        ``dry_run`` previews a **mutation** (heuristic: the document starts with ``mutation``) without
        writing; a query (read) always runs. Never-silent: a GraphQL ``errors`` array raises (G2).
        """
        is_mutation = query.lstrip().lower().startswith("mutation")
        if is_mutation and self.dry_run:
            safe_print(
                f"   [dry-run] GraphQL mutation (suppressed): {query.strip().splitlines()[0]} "
                f"vars={json.dumps(variables or {})}"
            )
            return {}
        payload = {"query": query, "variables": variables or {}}
        req = urllib.request.Request(
            _GITHUB_GRAPHQL,
            data=json.dumps(payload).encode(),
            method="POST",
            headers={**self._headers(graphql=True), "Content-Type": "application/json"},
        )
        try:
            with urllib.request.urlopen(req, timeout=_API_TIMEOUT) as resp:
                out = json.loads(resp.read().decode() or "{}")
        except urllib.error.HTTPError as e:
            detail = e.read().decode(errors="replace") if hasattr(e, "read") else ""
            raise RuntimeError(f"GitHub GraphQL -> HTTP {e.code}: {detail[:500]}")
        if out.get("errors"):
            raise RuntimeError(f"GitHub GraphQL error: {json.dumps(out['errors'])}")
        return out.get("data", {})


def _next_link(link_header):
    """Return the ``rel="next"`` URL from an RFC-5988 ``Link`` header, or None. Pure (unit-tested)."""
    for part in (link_header or "").split(","):
        seg = part.strip()
        if 'rel="next"' in seg and "<" in seg and ">" in seg:
            return seg[seg.index("<") + 1 : seg.index(">")]
    return None


def api_merged_pr_index(api, repo, patterns):
    """Build ``{task_id: pr_number}`` from the live merged-PR list via the token API (no `gh`).

    Enumerates ``state=all`` PRs, keeps the merged ones (``merged_at`` set), and maps each task-id in
    the PR title to its number (first-created wins on a tie — created-asc; the earliest merged PR
    whose title names the id, matching the ``setdefault`` below). Honest: only titles are parsed, exactly
    as the offline git-log path — the live list is a *cross-check*, never the sole basis. ``patterns``
    is the conventions.json ``task_id_patterns`` list (kept DRY with the `gh` path).
    """
    owner, name = repo.split("/", 1)
    index = {}
    for pr in api.paginate(
        f"/repos/{owner}/{name}/pulls", params={"state": "all", "sort": "created"}
    ):
        if not pr.get("merged_at"):
            continue
        num = pr.get("number")
        for tid in expand_task_id_run(pr.get("title") or ""):
            index.setdefault(tid, num)  # first (created-asc) wins deterministically
    return index


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


def plan_label_migrations(repo_label_names, canonical_names, aliases, retire=None):
    """Plan noncompliant-label reconcile actions from the repo's live label set.

    Pure (no I/O): returns (migrations, retirements, flags) where:
    - ``migrations`` is a sorted list of (old_name, new_name) for noncompliant labels that have a
      declared alias;
    - ``retirements`` is a sorted list of noncompliant names declared in ``retire`` — to be deleted
      ONLY if unused at reconcile time (a retired-but-still-carried label is FLAGGED there, never
      silently dropped, G2);
    - ``flags`` is a sorted list of noncompliant names that are neither aliased nor retired (handled
      manually — never silently deleted, G2).

    A label is COMPLIANT when its name appears in ``canonical_names`` (the labels.json set). A
    noncompliant label is MIGRATED (has an alias), else RETIRED (declared in ``retire``), else
    FLAGGED. An alias WINS over retire — migrating preserves the label↔issue link. An alias whose
    target is not in ``canonical_names`` is itself flagged (the target must be declared first).
    """
    canonical = set(canonical_names)
    retire = set(retire or ())
    noncompliant = [n for n in repo_label_names if n not in canonical]
    migrations, retirements, flags = [], [], []
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
        elif name in retire:
            retirements.append(name)
        else:
            flags.append(name)
    return sorted(migrations), sorted(retirements), sorted(flags)


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


# Scopes that are area-less BY DESIGN, not by omission (the conventions.json scope_to_area policy):
# doc-reference scopes (rfc-*/adr-*/dn-*/rp-* and the bare forms), task-id scopes (M-###, E#-#,
# m###-p#), and wave planning markers. These are surfaced as INFO at the print site (a `~`, not a
# `!`), so a genuine missing-area mapping stands out from one that is unmapped on purpose.
_INTENTIONAL_SCOPE_RE = re.compile(
    r"^(?:adr|dn|rfc|rp)(?:[-_].*)?$"  # doc-reference scopes (incl. the bare adr/dn/rfc/rp)
    r"|^m-?\d.*$"  # M-task ids: m-381, m376-p1
    r"|^e\d+-\d+.*$"  # E-task ids: e3-8
    r"|^wave.*$",  # wave / wave-1 / wave4 planning markers
    re.IGNORECASE,
)


def is_intentional_unmapped_scope(scope):
    """True when ``scope`` is unmappable by design (doc-ref / task-id / wave) — reported as info,
    not a flag-to-fix. See ``_INTENTIONAL_SCOPE_RE``."""
    return bool(_INTENTIONAL_SCOPE_RE.match((scope or "").strip()))


def derive_pr_labels(parsed, type_map, area_set, scope_aliases=None):
    """Map a parsed title to (labels:set, flags:list, infos:list) via conventions.json + area:* set.

    Pure + honest. The split is **structural**, not by message text, so the print site never sniffs
    wording to decide severity (a brittleness the earlier substring check had):
    - ``flags`` — genuine refusals-to-invent (unknown type, an unmapped scope, a bad alias). `!`.
    - ``infos`` — scopes that are area-less BY DESIGN (doc-ref/task-id/wave per the scope_to_area
      policy). `~`. Surfaced, never silently dropped (G2) — just not a gap to fix.

    The type label is required; area labels are best-effort on an exact scope match or a ratified
    alias.
    """
    scope_aliases = scope_aliases or {}
    labels, flags, infos = set(), [], []
    if parsed is None:
        return labels, ["title not Conventional-Commit form; type unresolved"], infos
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
        elif is_intentional_unmapped_scope(scope):
            # By-design area-less scope (doc-ref / task-id / wave) — info, not a flag-to-fix.
            infos.append(
                f"scope '{scope}' unmapped by design (doc-ref/task-id/wave) — no area inferred"
            )
        else:
            flags.append(f"scope '{scope}' is not an area:* label (no area inferred)")
    if parsed["breaking"]:
        flags.append("breaking change marked (no breaking:* label to apply)")
    return labels, flags, infos


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
    ignored. A phase-ordering utility (sorting/grouping milestones by phase); milestone
    *selection* anchors to the primary task (see `infer_milestone`), never the highest rank.
    Examples: 'Phase 8 — Toolchain' -> 8; 'Backlog' -> -1.
    """
    m = re.match(r"Phase\s+(\d+)", title or "", re.IGNORECASE)
    return int(m.group(1)) if m else -1


def infer_milestone(task_ids, task_to_ms):
    """Infer the single GitHub milestone for a PR/issue from its referenced task-ids.

    GitHub allows only ONE milestone per item, so a PR that spans phases must anchor to one. We
    anchor to the **primary task** — the FIRST referenced task-id with a known milestone. Callers
    pass task-ids title-first (``f"{title}\\n{body}"``), so the primary is the PR's leading/title
    task, never an incidental later-phase reference in the body. The span is recorded as an
    informational note; it is **never** used to over-advance the milestone to the highest phase
    touched (the previous behavior, which mis-filed a mostly-Phase-5 PR into Phase 8 for a single
    Phase-8 cross-reference).

    - No milestone found            → (None, None) — silent, no claim made.
    - Exactly one milestone spanned → (milestone, None).
    - Multiple milestones spanned   → (primary_milestone, note) — primary = first known; the note
                                      lists the full span. The note is informational, not a refusal.
    """
    ordered = [task_to_ms[t] for t in task_ids if t in task_to_ms]
    if not ordered:
        return None, None  # nothing to infer from — silent (no claim made)
    spanned = set(ordered)
    if len(spanned) == 1:
        return ordered[0], None
    # Multi-milestone: anchor to the primary (first-referenced) task's milestone — never the
    # highest phase (that over-advances on an incidental cross-reference). Order is preserved
    # because callers pass title-first text, so ordered[0] is the PR's primary task.
    primary = ordered[0]
    note = (
        f"note: spanned {sorted(spanned)} -> anchored to the primary (first-referenced) "
        f"task's milestone '{primary}', not the highest phase"
    )
    return primary, note


def infer_milestone_from_scope(scopes, scope_ms_aliases):
    """Declared scope→milestone fallback (Declared/G2 — never-invent).

    Consulted ONLY after task-id inference yields (None, None). Returns:
    - (milestone, None)         — all mapped scopes agree on one milestone.
    - (None, flag_string)       — scopes resolve to DIFFERENT milestones → FLAG, set nothing.
    - (None, None)              — no scopes are mapped (unmapped scopes are flagged by the
                                   caller via the normal derive_pr_labels flag path).

    Pure + honest: every lookup is a declared alias from conventions.json; no interpolation.
    Unmapped scopes are deliberately ignored here — the area-label FLAG path already surfaces
    them for the maintainer (double-FLAG would be noise, not signal).
    """
    resolved = {scope_ms_aliases[s] for s in scopes if s in scope_ms_aliases}
    if len(resolved) == 1:
        return next(iter(resolved)), None
    if not resolved:
        return (
            None,
            None,
        )  # no mapped scopes — silent; caller has already flagged unmapped ones
    # Multiple different milestones from the scope set — ambiguous, refuse to guess.
    flag = (
        f"scope milestone fallback: scopes {sorted(s for s in scopes if s in scope_ms_aliases)!r} "
        f"resolve to different milestones {sorted(resolved)} — not set (ambiguous)"
    )
    return None, flag


# ─────────────────────────────────────────────────────────────────────────────────────────────
# Relationship/date extraction (issue↔PR landings + dates from CHANGELOG.md + git log) — M-RELS
# ─────────────────────────────────────────────────────────────────────────────────────────────
# This block is PURE (no network, no `gh`): it derives, from the two in-repo dated sources of
# truth — ``CHANGELOG.md`` headers and ``git log origin/main`` squash subjects — a per-issue
# relationship/date manifest (landed_pr, landed_date, the grounding basis). Honesty (VR-5/G2): a
# date is asserted ONLY when grounded in a CHANGELOG header line or a commit SHA/subject; the basis
# string records WHICH (so the assertion is auditable and never upgraded past Empirical/Declared).
# These are exercised offline by ``--self-test`` and applied by ``--relationships`` (dry-run-able).

# A task-id token: M-### or E#-#. Used to find ids in a CHANGELOG header / commit subject. The
# trailing boundary is left open so a slash-run (``M-656/657/658``) is captured by the run-expander
# below rather than truncated mid-token.
_TASK_ID_TOKEN = re.compile(r"\b(M-\d+|E\d+-\d+)\b")
# A slash/abbreviated run of M-ids: ``M-656/657/658`` or ``M-510/512/520/522`` — the leading id is
# full (``M-656``); the rest are bare numbers sharing the ``M-`` prefix. Matched greedily so the
# whole run is one token before expansion.
_MID_RUN = re.compile(r"\bM-\d+(?:/\d+)+\b")
# A PR back-reference in a curated squash subject: the trailing ``(#NNN)``. GitHub squash subjects
# carry it; a curated subject may omit it (then the PR is left unknown — never invented, G2).
_PR_REF = re.compile(r"\(#(\d+)\)")
# A dated CHANGELOG section header: ``### Kind (YYYY-MM-DD: <rest>)``. ``rest`` carries the M-id(s).
_CHANGELOG_HEADER = re.compile(
    r"^###\s+(?P<kind>[A-Za-z]+)\s+\((?P<date>\d{4}-\d{2}-\d{2}):\s*(?P<rest>.*?)\)\s*$"
)


def expand_task_id_run(text):
    """Return the ordered, de-duplicated task-ids in ``text``, expanding abbreviated M-id runs.

    ``M-656/657/658`` → ``['M-656', 'M-657', 'M-658']``; a lone ``E7-1`` stays ``['E7-1']``.
    Pure + total (empty/None → ``[]``). Never invents an id the text does not contain (G2): a bare
    number only becomes an id when it is part of an explicit ``M-…/…`` run.
    """
    if not text:
        return []
    found, seen = [], set()
    covered = []  # char spans already consumed by an expanded run (so members aren't re-matched)
    for m in _MID_RUN.finditer(text):
        run = m.group(0)
        head = run.split("/", 1)[0]  # 'M-656'
        prefix = head.split("-", 1)[0] + "-"  # 'M-'
        ids = [head] + [f"{prefix}{n}" for n in run.split("/")[1:]]
        covered.append((m.start(), m.end()))
        for tid in ids:
            if tid not in seen:
                seen.add(tid)
                found.append(tid)
    # Standalone tokens not already inside an expanded run.
    for m in _TASK_ID_TOKEN.finditer(text):
        if any(s <= m.start() < e for (s, e) in covered):
            continue
        tid = m.group(1)
        if tid not in seen:
            seen.add(tid)
            found.append(tid)
    return found


def parse_changelog_landings(changelog_text):
    """Parse ``CHANGELOG.md`` into ``{task_id: {date, kind, header, basis}}``.

    Each dated section header ``### Kind (YYYY-MM-DD: M-xxx — …)`` attributes its date to every
    task-id named in the header text (slash-runs expanded). Pure + honest (G2): the FIRST (topmost,
    most-recent) header that names an id wins its ``landed_date`` (the changelog is newest-first, so
    the topmost mention is the landing record); the ``basis`` records the exact header line so the
    date is auditable. A header with no task-id contributes nothing (never guessed).
    """
    out = {}
    for raw in (changelog_text or "").splitlines():
        m = _CHANGELOG_HEADER.match(raw)
        if not m:
            continue
        date = m.group("date")
        kind = m.group("kind")
        rest = m.group("rest").strip()
        for tid in expand_task_id_run(rest):
            if tid in out:
                continue  # newest-first: keep the topmost (most recent) attribution
            out[tid] = {
                "date": date,
                "kind": kind,
                "header": rest,
                "basis": f"CHANGELOG.md '### {kind} ({date}: {rest})'",
            }
    return out


def parse_git_log_landings(git_log_text):
    """Parse ``git log`` ``SHA|YYYY-MM-DD|subject`` lines into ``{task_id: {pr, date, sha, subject}}``.

    Feed it the output of ``git log <ref> --format='%H|%ad|%s' --date=short``. For each commit, every
    task-id in the subject is attributed the commit's short-SHA, date, and ``(#NNN)`` PR number when
    the subject carries one (a curated subject may omit it → ``pr`` is None, never invented, G2).
    Newest-first wins (matches ``git log`` default order). Pure + total.
    """
    out = {}
    for raw in (git_log_text or "").splitlines():
        line = raw.strip()
        if not line or line.count("|") < 2:
            continue
        sha, date, subject = line.split("|", 2)
        subject = subject.strip()
        pr_m = _PR_REF.search(subject)
        pr = int(pr_m.group(1)) if pr_m else None
        for tid in expand_task_id_run(subject):
            if tid in out:
                continue
            out[tid] = {"pr": pr, "date": date, "sha": sha[:7], "subject": subject}
    return out


def epic_of_task_id(task_id):
    """Return the epic id a leaf ``E#-#`` id rolls up to, or None.

    Honest by construction (G2): only the **explicit** ``E#-#`` form yields an epic (``E7-1`` →
    ``E7``); an ``M-###`` id carries no epic in its own text, so this returns None (the epic, when
    known, comes from issues.yaml ``epic:``/``depends_on`` or the idmap comments — never guessed
    from the number range).
    """
    m = re.match(r"^(E\d+)-\d+$", task_id or "")
    return m.group(1) if m else None


def _status_of(entry):
    """Return the ``status:*`` value of an issue entry (e.g. 'done'), or None. Pure."""
    for lb in entry.get("labels", []) or []:
        if isinstance(lb, str) and lb.startswith("status:"):
            return lb.split(":", 1)[1]
    return None


def build_relationship_manifest(
    issues, changelog_landings, gitlog_landings, pr_index=None
):
    """Merge the two evidence sources into a per-issue relationship/date manifest.

    Returns ``{task_id: {…}}`` for every issue with grounded evidence. **Status-aware honesty**
    (VR-5/G2 — the crux): a strong ``landed_pr``/``landed_date`` claim ("this issue's work landed
    in this PR on this date") is asserted **only for a `status:done` issue**. For an issue that is
    NOT done (in-progress / blocked / needs-design), the very same evidence is real but a *weaker*
    claim — the id was merely **referenced** by that PR/CHANGELOG entry (a partial tranche, a filing
    commit, an aspirational title) — so it is emitted under ``evidence_pr``/``evidence_date`` with a
    note, **never** as a completed landing. This refuses to overclaim completion (the honesty rule).

    Field provenance (both the strong and weak forms):
      * the date comes from the CHANGELOG header (preferred — the in-repo dated record) else the
        git-log commit date; the chosen source is named in the basis.
      * the PR comes from the git-log ``(#NNN)`` subject else a ``pr_index`` ``{task_id: pr}``
        cross-checked against the live merged-PR list (the caller supplies it from the MCP/`gh`
        enumeration — never fabricated here). When both name a PR and they DISAGREE, the
        disagreement is recorded in the basis (never silently reconciled — G2).
      * a field is emitted ONLY when grounded; an issue with no evidence is absent (never null-filled).
      * the ``epic`` edge is a separate, always-grounded relationship (the explicit E#-# id form).
    """
    pr_index = pr_index or {}
    by_id = {e.get("id"): e for e in issues if e.get("id")}
    out = {}
    for tid in sorted(by_id):
        entry = by_id[tid]
        is_done = _status_of(entry) == "done"
        date_field = "landed_date" if is_done else "evidence_date"
        pr_field = "landed_pr" if is_done else "evidence_pr"
        cl = changelog_landings.get(tid)
        gl = gitlog_landings.get(tid)
        idx_pr = pr_index.get(tid)
        rec = {}
        basis_bits = []
        if not is_done:
            basis_bits.append(
                f"status:{_status_of(entry) or 'unknown'} (NOT done) — evidence is a REFERENCE/"
                "partial, not a completed landing"
            )
        # date: CHANGELOG header preferred, else git-log commit date.
        if cl and cl.get("date"):
            rec[date_field] = cl["date"]
            basis_bits.append(cl["basis"])
        elif gl and gl.get("date"):
            rec[date_field] = gl["date"]
            basis_bits.append(f"git log {gl['sha']} ({gl['date']}) '{gl['subject']}'")
        # PR: git-log (#NNN) preferred (carries the SHA), else the cross-checked index.
        gl_pr = gl.get("pr") if gl else None
        if gl_pr is not None:
            rec[pr_field] = gl_pr
            basis_bits.append(f"git log {gl['sha']} subject '(#{gl_pr})'")
            if idx_pr is not None and idx_pr != gl_pr:
                basis_bits.append(
                    f"FLAG: PR disagreement — git-log #{gl_pr} vs merged-PR-list #{idx_pr}"
                )
        elif idx_pr is not None:
            rec[pr_field] = idx_pr
            basis_bits.append(
                f"merged-PR-list #{idx_pr} (cross-checked; no (#NNN) in the curated subject)"
            )
        have_evidence = bool(rec.get(date_field) or rec.get(pr_field))
        if have_evidence:
            rec["landed_basis"] = "; ".join(basis_bits)
        epic = epic_of_task_id(tid)
        if epic:
            rec["epic"] = epic
        # Emit when ANY grounded relationship is present (evidence OR an explicit epic edge).
        if have_evidence or rec.get("epic"):
            out[tid] = rec
    return out


def plan_relationship_enrichment(issues, manifest):
    """Plan the ADDITIVE per-issue field writes (never overwrite/rewrite an existing field).

    Returns ``{task_id: {field: value}}`` containing only the fields the manifest grounds that the
    issue does NOT already carry. Append-only by construction (G2/house-rule 3): an issue that
    already has ``landed_date`` keeps it; only absent fields are added. Pure — does no IO.
    """
    by_id = {e.get("id"): e for e in issues if e.get("id")}
    plan = {}
    for tid, rec in manifest.items():
        entry = by_id.get(tid)
        if entry is None:
            continue
        adds = {}
        for field in (
            "landed_pr",
            "landed_date",
            "evidence_pr",
            "evidence_date",
            "landed_basis",
            "epic",
        ):
            if field in rec and field not in entry:
                adds[field] = rec[field]
        if adds:
            plan[tid] = adds
    return plan


def derive_subissue_edges(issues):
    """Derive epic→sub-issue edges from issues.yaml, returning ``{epic_id: [child_id, …]}``.

    Honest source (G2): an edge exists when a child entry **already declares** ``epic:``/``parent:``.
    ``depends_on`` is deliberately NOT used as a parenthood signal (it is a *dependency*, not a
    parent — conflating them would invent structure); the idmap.tsv comments (e.g. "M-364..M-368
    sub-issues of M-361") are the human record an editor uses to ADD an ``epic:`` field, not
    auto-inferred here.
    """
    edges = {}
    for e in issues:
        cid = e.get("id")
        if not cid:
            continue
        parent = e.get("epic") or e.get("parent")
        if parent:
            edges.setdefault(parent, []).append(cid)
    return edges


# ─────────────────────────────────────────────────────────────────────────────────────────────
# Pure Project-v2 mapping/diff logic (drives --project; exercised by --self-test)
# ─────────────────────────────────────────────────────────────────────────────────────────────
def label_to_field_values(labels, field_label_map, field_option_order=None):
    """Map an item's labels to {field_name: option_name} per project.json.

    Returns ``(values, flags, infos)``:
      * ``values`` — {field: option_name} to set (names; the engine resolves names→option ids and
        writes only on drift).
      * ``flags``  — never-silent problems that leave a field UNSET (a genuine conflict / unmapped
        label); the caller prints them and the value is never guessed past (G2).
      * ``infos``  — informational notes for a value that WAS set under a documented rule (e.g. a
        multi-area item anchored to its primary area, with the full span recorded).

    Pure: no I/O — exercised offline by ``--self-test``.

    Multi-value handling (M-676): a ``label_prefix`` rule may carry ``"multi": "primary"``. With it,
    an item bearing several ``<prefix>*`` labels is ANCHORED to a deterministic *primary* — the
    label whose value is earliest in the field's declared option order
    (``field_option_order[field]``, taken from project.json's ``fields``), with an alphabetical
    tie-break for any value outside the declared order — and the full span is recorded as an
    ``info`` (never a silent skip, never a "not set" punt). This mirrors the primary-task milestone
    anchor (resolve_milestone_span / #353): pick a primary, record the span, refuse nothing.
    WITHOUT ``multi`` (the default), a multi-hit field stays a never-set ``flag`` — correct for
    fields like Phase/Priority where two values is a real conflict, not legitimate multi-membership.
    """
    labels = set(labels)
    field_option_order = field_option_order or {}
    values, flags, infos = {}, [], []
    for field, rule in field_label_map.items():
        if field.startswith("_"):
            continue
        if rule.get("from") == "label_prefix":
            prefix = rule["prefix"]
            hits = sorted(lb[len(prefix) :] for lb in labels if lb.startswith(prefix))
            if len(hits) == 1:
                values[field] = rule["template"].replace("{value}", hits[0])
            elif len(hits) > 1:
                if rule.get("multi") == "primary":
                    order = field_option_order.get(field, [])
                    # primary = earliest in the declared option order; a value outside the order
                    # sorts last, then alphabetically — fully deterministic, no tie left to chance.
                    primary = min(
                        hits,
                        key=lambda h: (order.index(h) if h in order else len(order), h),
                    )
                    values[field] = rule["template"].replace("{value}", primary)
                    infos.append(
                        f"{field}: multiple {prefix}* labels {hits} -> anchored to primary "
                        f"'{primary}' (declared option order); span recorded"
                    )
                else:
                    flags.append(f"{field}: multiple {prefix}* labels {hits} — not set")
        elif rule.get("from") == "label_exact":
            mapped = sorted(rule["map"][lb] for lb in labels if lb in rule["map"])
            if len(mapped) == 1:
                values[field] = mapped[0]
            elif len(mapped) > 1:
                flags.append(f"{field}: conflicting status labels {mapped} — not set")
    return values, flags, infos


def plan_option_reconcile(desired_options, actual_option_names):
    """Return the option names to ADD so a single-select field covers ``desired_options``.

    Add-only: we never delete an option (it may be in use). Order-preserving on desired.
    """
    actual = set(actual_option_names)
    return [opt["name"] for opt in desired_options if opt["name"] not in actual]


def plan_option_additions(live_options, desired_options):
    """Build the additive, non-destructive option union for ``updateProjectV2Field`` (pure).

    ``live_options`` are the field's CURRENT options (dicts: name/color/description, from
    ``fetch_project_fields``). ``desired_options`` are the manifest's options (same shape, from
    ``project.json``). ``updateProjectV2Field`` matches by name and DELETES any omitted option, so
    we return the **full union** the caller must send verbatim.

    Returns ``(full_union_options, added_names, extra_live_names)``:
      * ``full_union_options`` — every live option first (preserved verbatim: id is kept by name,
        color/description unchanged) then each manifest option not already live (appended). Sending
        this set adds the new options and deletes nothing.
      * ``added_names`` — manifest options newly appended (order-preserving on the manifest).
      * ``extra_live_names`` — live options absent from the manifest (a would-be deletion): kept in
        the union (never deleted) and surfaced so the caller can FLAG them (G2 — never silently
        destructive). Order-preserving on the live list.
    """
    live_names = [o["name"] for o in live_options]
    live_set = set(live_names)
    desired_set = {o["name"] for o in desired_options}

    full_union_options = list(live_options)  # preserve every live option verbatim
    added_names = []
    for opt in desired_options:
        if opt["name"] not in live_set:
            full_union_options.append(opt)
            added_names.append(opt["name"])
    extra_live_names = [n for n in live_names if n not in desired_set]
    return full_union_options, added_names, extra_live_names


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


def load_pr_overrides(here):
    """Load pr-overrides.json from ``here``, returning {number(int): {milestone, labels}}.

    Tolerates the file being absent — returns an empty dict (no overrides). Never-silent:
    if the file exists but is malformed, an explicit error is raised (G2). The returned dict
    maps PR/issue number (int) to {'milestone': str, 'labels': list[str]}. Internal _* keys
    are stripped (rationale/confirm annotations are informational only).

    Declared (not Empirical): these overrides are explicit, ratified per-item decisions —
    NOT inference — and take highest precedence over task-id and scope fallback (G2 escape hatch).
    """
    overrides_path = here / "pr-overrides.json"
    if not overrides_path.exists():
        return {}
    try:
        raw = json.loads(overrides_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise ValueError(f"pr-overrides.json: JSON parse error — {exc}") from exc
    result = {}
    for key, entry in raw.get("overrides", {}).items():
        # PR/issue numbers are positive integers — `isdigit()` rejects '-67', 'abc', '' (a typo
        # must surface, not be silently routed past). Never-silent (G2): a malformed key/entry is
        # an explicit error here, not a dropped override that would vanish on a `--prs` run.
        if not key.isdigit():
            raise ValueError(
                f"pr-overrides.json: key {key!r} is not a valid PR/issue number "
                "(must be a positive integer string)"
            )
        number = int(key)
        milestone = entry.get("milestone")
        if not milestone:
            raise ValueError(
                f"pr-overrides.json: override #{key} has no 'milestone' field "
                "(every override must declare a milestone title)"
            )
        labels = [lb for lb in (entry.get("labels") or []) if not lb.startswith("_")]
        result[number] = {"milestone": milestone, "labels": labels}
    return result


def apply_pr_override(pr_number, overrides, existing_milestone, existing_labels):
    """PURE: apply a declared override for ``pr_number``, returning (milestone, labels_to_add).

    Highest-precedence (G2 escape hatch): the override milestone ALWAYS wins over task-id
    and scope inference. Labels in the override are union-added (add-only, idempotent).
    Returns (None, []) when no override is declared for this PR (the inference path applies).
    """
    override = overrides.get(pr_number)
    if override is None:
        return None, []
    ms = override.get("milestone")
    labels_to_add = [
        lb for lb in (override.get("labels") or []) if lb not in existing_labels
    ]
    return ms, labels_to_add


def issue_override_changes(live, override):
    """PURE: render a declared override into an issue ``changes`` plan (consumed by
    ``apply_issue_update``). The issue-side twin of ``apply_pr_override``: the override milestone
    wins (highest precedence) and labels are union-added (add-only) — never stripped. Returns an
    empty dict when the issue already matches the override (idempotent no-op)."""
    changes = {}
    ms = override.get("milestone")
    if ms and ms != live.get("milestone"):
        changes["milestone"] = ms
    add = sorted(
        lb
        for lb in (override.get("labels") or [])
        if lb not in live.get("labels", set())
    )
    if add:
        changes["labels"] = (
            add,
            [],
        )  # (add, remove) — add-only, never removes a human's label
    return changes


def apply_issue_overrides(repo, by_number, overrides, dry_run):
    """Apply declared overrides to existing ISSUES — the ``--issues`` counterpart of the ``--prs``
    override path in ``reconcile_prs``.

    For each override number that is an issue on ``repo`` (e.g. a closed issue like #67 that is NOT a
    task in ``issues.yaml``, so the create/update passes never touch it), set its milestone + add-only
    labels (idempotent). Numbers that are not issues here are PRs → applied by ``reconcile_prs``.
    Never-silent (G2): prints a one-line summary (applied / already-in-sync / routed-to-``--prs``) so
    no override is silently unaccounted-for."""
    if not overrides:
        return
    applied = in_sync = routed = 0
    for number, override in overrides.items():
        live = by_number.get(number)
        if live is None:
            routed += 1  # not an issue on this repo → a PR; reconcile_prs applies it
            continue
        changes = issue_override_changes(live, override)
        if not changes:
            in_sync += 1
            continue
        apply_issue_update(repo, number, changes, None, dry_run)
        applied += 1
    verb = "would apply" if dry_run else "applied"
    tail = f"; {routed} are PRs -> applied by --prs" if routed else ""
    print(f">> issue overrides — {applied} {verb}, {in_sync} already in sync{tail}")


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


def _load_label_retire(here):
    """Load the declared ``retire`` list from label-aliases.json — names approved for deletion when
    UNUSED. Absent file/key → empty set (nothing retired). Each entry is a ratified decision: the
    label is deleted only if it carries no issues/PRs at reconcile time, else FLAGGED (G2 — never a
    silent drop). Lets the standard GitHub stock labels (duplicate/help wanted/…) be cleared without
    re-flagging them every run, while still refusing to drop one that is actually in use."""
    aliases_path = here / "label-aliases.json"
    if not aliases_path.exists():
        return set()
    data = json.loads(aliases_path.read_text(encoding="utf-8"))
    return set(data.get("retire", []))


def _label_task(repo, lb):
    """Build a batch task (callable -> (item, ok, err)) that create-or-updates one label.

    Idempotent + disjoint (each label is its own resource), so the create-or-update batch is safe
    to parallelize. ``--force`` makes `gh label create` create-or-update (color+description)."""
    name, color, desc = lb["name"], lb.get("color", ""), lb.get("description", "")

    def task():
        rc, _out, err = run_gh_task(
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
        if rc != 0:
            return (name, False, _stderr_tail(err))
        safe_print(f"   • {name}")
        return (name, True, None)

    task.item = name
    return task


def _relabel_task(repo, number, old_name, new_name):
    """Build a batch task that relabels one issue/PR ``old_name``->``new_name`` (disjoint, idempotent)."""

    def task():
        rc, _out, err = run_gh_task(
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
        if rc != 0:
            return (number, False, _stderr_tail(err))
        safe_print(f"   • #{number}: label '{old_name}' -> '{new_name}'")
        return (number, True, None)

    task.item = number
    return task


DEFAULT_LABEL_COLOR = (
    "ededed"  # neutral gray for an auto-created (undefined-in-manifest) label
)


def reconcile_labels(repo, labels_json, dry_run, issues_yaml=None):
    labels = json.loads(labels_json.read_text(encoding="utf-8"))
    # Robustness (matches manifest-check's non-fatal WARNING): a label USED by an issue but absent
    # from labels.json is auto-included here with a default colour + a loud warning (never-silent),
    # so the gap never becomes a hard sync break. Add it to labels.json for a proper colour/description.
    if issues_yaml and Path(issues_yaml).exists():
        declared = {lb["name"] for lb in labels}
        spec = yaml.safe_load(Path(issues_yaml).read_text(encoding="utf-8")) or {}
        used = {lab for e in spec.get("issues", []) for lab in (e.get("labels") or [])}
        for name in sorted(used - declared):
            safe_print(
                f"   ! label '{name}' is used by issues but absent from {labels_json.name} — "
                f"auto-creating with a default colour (add it to {labels_json.name} properly)",
                file=sys.stderr,
            )
            labels.append(
                {
                    "name": name,
                    "color": DEFAULT_LABEL_COLOR,
                    "description": "auto-created (undefined in labels.json) — please define properly",
                }
            )
    print(f">> labels: {len(labels)} declared in {labels_json.name}")
    if dry_run:
        for lb in labels:
            print(f"   ~ would create-or-update: {lb['name']}")
    else:
        # Batch: create-or-update each label concurrently (idempotent, disjoint resources).
        _ok, fail, failures = run_batch(
            "labels", [_label_task(repo, lb) for lb in labels]
        )
        for name, err in failures:
            safe_print(
                f"   ! could not create-or-update label '{name}' ({err}) — re-run to retry",
                file=sys.stderr,
            )

    # ── noncompliant-label reconcile (runs after create-or-update) ───────────────────────────
    # Snapshot the repo's actual labels and compare against labels.json. Noncompliant labels
    # with a declared alias are migrated (relabel every open+closed issue/PR, then delete the
    # stale label). Noncompliant labels without an alias are FLAGGED — never silently deleted (G2).
    aliases = _load_label_aliases(HERE)
    retire = _load_label_retire(HERE)
    canonical_names = {lb["name"] for lb in labels}
    raw_repo_labels = json.loads(gh(["api", "--paginate", f"repos/{repo}/labels"]))
    repo_label_names = [lb["name"] for lb in raw_repo_labels]
    migrations, retirements, flags = plan_label_migrations(
        repo_label_names, canonical_names, aliases, retire
    )

    if not migrations and not retirements and not flags:
        print("   = noncompliant labels: none found (repo labels match labels.json)")
    else:
        if migrations:
            print(f">> noncompliant-label migrations: {len(migrations)} to migrate")
        for old_name, new_name in migrations:
            # Find every open + closed issue/PR carrying the stale label. The label name is
            # URL-encoded (it can contain spaces, e.g. the GitHub default `good first issue`) —
            # an unencoded space makes a malformed query that HANGS the request (M-382 follow-up).
            raw_issues = json.loads(
                gh(
                    [
                        "api",
                        "--paginate",
                        f"repos/{repo}/issues?state=all&per_page=100&labels={quote(old_name, safe='')}",
                    ]
                )
            )
            # Prioritize OPEN issues (maintainer's note "at least for open issues"): process open
            # before closed so a closed-issue blip never blocks the actively-tracked ones.
            open_items = [it for it in raw_issues if it.get("state") == "open"]
            closed_items = [it for it in raw_issues if it.get("state") != "open"]
            ordered_numbers = [it["number"] for it in open_items] + [
                it["number"] for it in closed_items
            ]
            if dry_run:
                print(
                    f"   ~ would migrate label '{old_name}' -> '{new_name}' "
                    f"({len(ordered_numbers)} issue(s)/PR(s): "
                    f"{len(open_items)} open, {len(closed_items)} closed), then delete '{old_name}'"
                )
            else:
                # Fault-tolerant: a per-issue edit that fails is reported and SKIPPED — never an
                # abort of the whole label (M-382). We track failures so the stale label is only
                # deleted when EVERY issue was successfully relabeled (G2: never break the
                # label↔issue link silently). The relabels target DISJOINT issues + are idempotent,
                # so they run as a bounded-concurrency batch (M-397) — open-before-closed ordering
                # is preserved at submission; failures are aggregated, never abort the batch.
                _ok, failed, failures = run_batch(
                    f"relabel '{old_name}'->'{new_name}'",
                    [
                        _relabel_task(repo, number, old_name, new_name)
                        for number in ordered_numbers
                    ],
                )
                for number, err in failures:
                    safe_print(
                        f"   ! could not relabel #{number} '{old_name}'->'{new_name}' "
                        f"({err}) — left as-is; re-run to retry",
                        file=sys.stderr,
                    )
                if failed:
                    # Never delete a label still carried by issues — a stale-but-present label is
                    # safer than a silently-broken link. FLAG + skip the delete; re-run to finish.
                    safe_print(
                        f"   ! '{old_name}' still on {failed} issue(s) — not deleted; re-run",
                        file=sys.stderr,
                    )
                else:
                    rc, _out, err = run_gh_task(
                        ["label", "delete", old_name, "--repo", repo, "--yes"]
                    )
                    if rc != 0:
                        safe_print(
                            f"   ! could not delete stale label '{old_name}' "
                            f"({_stderr_tail(err)}) — re-run to retry",
                            file=sys.stderr,
                        )
                    else:
                        safe_print(f"   • deleted stale label '{old_name}'")

        for old_name in retirements:
            # A retired label (declared in label-aliases.json `retire`) is deleted ONLY when it
            # carries no issues/PRs. A retired-but-still-used label is FLAGGED, never silently
            # dropped (G2) — add an alias to migrate it, or clear it manually. URL-encode the name
            # (it can contain spaces, e.g. `help wanted`) — an unencoded space HANGS the query.
            raw_issues = json.loads(
                gh(
                    [
                        "api",
                        "--paginate",
                        f"repos/{repo}/issues?state=all&per_page=100&labels={quote(old_name, safe='')}",
                    ]
                )
            )
            carriers = [it["number"] for it in raw_issues]
            if carriers:
                safe_print(
                    f"   ! retired label '{old_name}' still on {len(carriers)} issue(s)/PR(s) "
                    f"— not deleted; add an alias to migrate it, or clear it manually (G2)",
                    file=sys.stderr,
                )
                continue
            if dry_run:
                print(f"   ~ would retire (delete unused) label '{old_name}'")
                continue
            rc, _out, err = run_gh_task(
                ["label", "delete", old_name, "--repo", repo, "--yes"]
            )
            if rc != 0:
                safe_print(
                    f"   ! could not retire label '{old_name}' "
                    f"({_stderr_tail(err)}) — re-run to retry",
                    file=sys.stderr,
                )
            else:
                print(f"   • retired unused label '{old_name}'")

        for flag in flags:
            # Never-silent: a noncompliant label with no alias is left untouched and reported (G2).
            print(
                f"   ! noncompliant label '{flag}' has no declared alias in label-aliases.json "
                f"— left untouched; add an alias, retire it, or delete it manually",
                file=sys.stderr,
            )


def _milestone_create_task(repo, ms):
    """Build a batch task that creates one absent milestone (disjoint by title, idempotent here
    because the caller only enqueues titles not already present)."""
    title = ms["title"]

    def task():
        rc, out, err = run_gh_task(
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
        if rc != 0:
            return (title, False, _stderr_tail(err))
        safe_print(f"   + created #{json.loads(out)['number']}: {title}")
        return (title, True, None)

    task.item = title
    return task


def reconcile_milestones(repo, milestones_json, dry_run):
    milestones = json.loads(milestones_json.read_text(encoding="utf-8"))
    existing = {
        m["title"]: m["number"]
        for m in json.loads(
            gh(["api", f"repos/{repo}/milestones?state=all", "--paginate"])
        )
    }
    print(f">> milestones: {len(milestones)} declared in {milestones_json.name}")
    to_create = []
    for ms in milestones:
        title = ms["title"]
        if title in existing:
            print(f"   = exists #{existing[title]}: {title}")
            continue
        if dry_run:
            print(f"   + would create: {title}")
            continue
        to_create.append(ms)
    if to_create:
        # Batch: each absent milestone is a disjoint create — run them concurrently (M-397).
        _ok, _fail, failures = run_batch(
            "milestones", [_milestone_create_task(repo, ms) for ms in to_create]
        )
        for title, err in failures:
            safe_print(
                f"   ! could not create milestone '{title}' ({err}) — re-run to retry",
                file=sys.stderr,
            )


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


def _create_issue_task(repo, entry):
    """Build a batch task that creates one issue (pass-1) and returns a result carrying the new
    number/id so the caller can record an idmap row. Fault-tolerant: a failure is captured as
    ``(tid, False, err)`` and never aborts the create batch (G2)."""
    tid = entry["id"]
    title = entry["title"]

    def task():
        args = ["issue", "create", "--repo", repo, "--title", title, "--body-file", "-"]
        for label in entry.get("labels") or []:
            args += ["--label", label]
        rc, out, err = run_gh_task(args, input_text=entry.get("body", "") or "")
        if rc != 0:
            return (tid, False, _stderr_tail(err))
        number = int(out.strip().rstrip("/").rsplit("/", 1)[-1])
        rc2, out2, err2 = run_gh_task(["api", f"repos/{repo}/issues/{number}"])
        if rc2 != 0:
            return (tid, False, _stderr_tail(err2))
        node = json.loads(out2)
        safe_print(f"   + created #{number}: {title}")
        if entry.get("milestone"):
            rc3, _o3, err3 = run_gh_task(
                [
                    "issue",
                    "edit",
                    str(number),
                    "--repo",
                    repo,
                    "--milestone",
                    entry["milestone"],
                ]
            )
            if rc3 != 0:
                safe_print(
                    f"     ! milestone absent (run with --milestones first): {entry['milestone']}",
                    file=sys.stderr,
                )
            else:
                safe_print(f"     ~ milestone: {entry['milestone']}")
        # Carry the new number/id back via the result's err slot (a dict) so reconcile_issues can
        # record the idmap row; ok=True signals success.
        return (tid, True, {"number": number, "id": node["id"]})

    task.item = tid
    return task


def _update_issue_task(repo, live, changes, update_bodies):
    """Build a batch task that applies an issue's ``changes`` plan (pass-2 update). Disjoint per
    issue + idempotent; a failure is captured, never aborts the update batch (G2)."""
    number = live["number"]

    def task():
        label_args, summary = _issue_update_args(changes)
        if label_args:
            rc, _o, err = run_gh_task(
                ["issue", "edit", str(number), "--repo", repo, *label_args]
            )
            if rc != 0:
                return (number, False, _stderr_tail(err))
        if "body" in changes:
            rc, _o, err = run_gh_task(
                ["issue", "edit", str(number), "--repo", repo, "--body-file", "-"],
                input_text=changes["body"],
            )
            if rc != 0:
                return (number, False, _stderr_tail(err))
        if "state" in changes:
            verb = "close" if changes["state"] == "closed" else "reopen"
            rc, _o, err = run_gh_task(["issue", verb, str(number), "--repo", repo])
            if rc != 0:
                return (number, False, _stderr_tail(err))
        safe_print(f"   ~ updated #{number}: {', '.join(summary)}")
        return (number, True, None)

    task.item = number
    return task


def _issue_update_args(changes):
    """PURE: render an issue ``changes`` plan to (gh-edit-args, human-summary) (no I/O).

    Factored out of ``apply_issue_update`` so the batch path and the sequential path build the
    exact same `gh issue edit` arguments + the same never-silent summary (exercised by --self-test)."""
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
    return label_args, summary


def apply_issue_update(repo, number, changes, entry, dry_run):
    """Apply a non-empty ``changes`` plan to issue ``number`` via `gh`, reporting each field."""
    label_args, summary = _issue_update_args(changes)

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


def partition_issue_work(
    issues, idmap, by_number, by_title, *, do_update, update_bodies
):
    """PURE: split issues.yaml entries into the create / update / in-sync work classes (no I/O).

    Returns ``(to_create, to_update, idmap_rows, in_sync)``:
      * ``to_create`` — entries whose live issue was not found (pass-1 create work);
      * ``to_update`` — ``[(entry, live, changes)]`` whose live issue drifts (pass-2 update work);
      * ``idmap_rows`` — ``[(tid, number, id)]`` for ALREADY-EXISTING matches (create rows are
        appended later from the create results);
      * ``in_sync`` — count of matched-but-unchanged entries.
    Matching is idmap-number-first (rename-safe) then title — identical to the sequential logic;
    factoring it pure lets --self-test cover the batch-partitioning decision (M-397)."""
    to_create, to_update, idmap_rows, in_sync = [], [], [], 0
    for entry in issues:
        tid = entry["id"]
        live = by_number.get(idmap.get(tid)) or by_title.get(entry["title"])
        if live is None:
            to_create.append(entry)
            continue
        idmap_rows.append((tid, live["number"], live["id"]))
        if not do_update:
            continue
        changes = plan_issue_update(entry, live, update_bodies=update_bodies)
        if changes:
            to_update.append((entry, live, changes))
        else:
            in_sync += 1
    return to_create, to_update, idmap_rows, in_sync


def reconcile_issues(
    repo, issues, idmap_path, *, do_update, update_bodies, dry_run, overrides=None
):
    by_number, by_title = snapshot_issues(repo)
    idmap = load_idmap(idmap_path)
    print(
        f">> issues: {len(issues)} task(s) in issues.yaml; "
        f"{len(by_number)} issue(s) on {repo} "
        f"(update={'on' if do_update else 'off'}, bodies={'on' if update_bodies else 'off'})"
    )

    to_create, to_update, idmap_rows, in_sync = partition_issue_work(
        issues,
        idmap,
        by_number,
        by_title,
        do_update=do_update,
        update_bodies=update_bodies,
    )

    if dry_run:
        # Preview only — sequential, mutates nothing (preserves the exact dry-run output).
        for entry in to_create:
            create_issue(repo, entry, dry_run)
        for entry, live, changes in to_update:
            apply_issue_update(repo, live["number"], changes, entry, dry_run)
        print(
            f">> issues done — {len(to_create)} would create, {len(to_update)} would update, "
            f"{in_sync} already in sync"
        )
        apply_issue_overrides(repo, by_number, overrides, dry_run)
        return

    # Pass-1: create absent issues as a batch. MUST complete before pass-2 so a future
    # depends_on/sub-issue linking pass (not yet implemented in this script) would see every new
    # number. Cross-batch order preserved: creates fully aggregate before updates dispatch.
    created = 0
    if to_create:
        # _create_issue_task returns the new number/id in the ok result's payload slot, so we ask
        # for the full results list to append idmap rows for the successes + flag the failures.
        create_results = run_batch(
            "issue create (pass 1)",
            [_create_issue_task(repo, e) for e in to_create],
            return_results=True,
        )
        for tid, ok, payload in create_results:
            if ok and isinstance(payload, dict):
                idmap_rows.append((tid, payload["number"], payload["id"]))
                created += 1
            else:
                safe_print(
                    f"   ! could not create issue {tid} ({payload}) — re-run to retry",
                    file=sys.stderr,
                )

    # Pass-2: apply drift updates as a batch (disjoint per issue, idempotent).
    updated = 0
    if to_update:
        _ok, _fail, upd_failures = run_batch(
            "issue updates (pass 2)",
            [
                _update_issue_task(repo, live, changes, update_bodies)
                for _e, live, changes in to_update
            ],
        )
        updated = len(to_update) - len(upd_failures)
        for number, err in upd_failures:
            safe_print(
                f"   ! could not update #{number} ({err}) — re-run to retry",
                file=sys.stderr,
            )

    append_idmap(idmap_path, idmap_rows)
    print(
        f">> issues done — {created} created, {updated} updated, {in_sync} already in sync"
    )
    # Declared overrides for issues present on the repo (e.g. #67, not an issues.yaml task). The PR
    # path applies overrides whose number is a PR; this applies them where the number is an issue.
    apply_issue_overrides(repo, by_number, overrides, dry_run)


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
def reconcile_prs(repo, conventions, area_set, task_to_ms, *, dry_run, overrides=None):
    """Backfill PR labels and milestones from CC-title inference, with declared overrides winning.

    Override precedence (highest first):
    1. Declared override from pr-overrides.json (G2 escape hatch — explicit, ratified decision).
    2. Task-id inference (issues.yaml task_id -> milestone).
    3. Scope→milestone fallback (conventions.json scope_to_milestone aliases).
    4. Nothing set (FLAGGED for manual assignment).
    """
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
    scope_ms_aliases = conventions.get("scope_to_milestone", {}).get("aliases", {})
    overrides = overrides or {}
    prs = snapshot_prs(repo)
    print(f">> PRs: {len(prs)} on {repo} — add-only label/milestone backfill")

    # Phase A — derive each PR's plan SEQUENTIALLY (keeps flag output ordered + stable; the
    # commit-title fallback is a cheap read). Phase B — batch the actual edits (disjoint per PR,
    # add-only ⇒ idempotent), aggregating results never-silently (M-397).
    in_sync = 0
    edits = []  # [(number, edit_args, summary_line)]
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

        desired, flags, infos = derive_pr_labels(
            parsed, type_map, area_set, scope_aliases
        )

        # ── Milestone resolution: override > task-id > scope-fallback > nothing ──────────────
        # 1. Declared override (highest precedence — G2 escape hatch for items with no inferable
        #    milestone). Never-silent: an override hit is always reported as an info.
        override_ms, override_labels = apply_pr_override(
            number, overrides, pr["milestone"], pr["labels"]
        )
        if override_ms is not None:
            ms = override_ms
            infos.append(
                f"milestone set via declared override: '{ms}' (pr-overrides.json)"
            )
            if override_labels:
                desired = desired | set(override_labels)
        else:
            # 2. Task-id inference (issues.yaml task-ids always beat scope fallback).
            ms, ms_note = infer_milestone(extract_task_ids(text, patterns), task_to_ms)
            if ms_note:
                # The milestone WAS set (to the primary/first-referenced task's phase); the note
                # records the full span — it is informational, never a refusal.
                infos.append(ms_note)
            if ms is None and parsed is not None:
                # 3. No task-id resolved → try the declared scope→milestone fallback (G2: never-invent).
                scopes = parsed.get("scopes", [])
                ms, scope_ms_flag = infer_milestone_from_scope(scopes, scope_ms_aliases)
                if scope_ms_flag:
                    # Ambiguous multi-scope: different milestones — refuse to set, flag it.
                    flags.append(scope_ms_flag)
                    ms = None
                elif ms is not None:
                    infos.append(
                        f"milestone set via scope fallback: scope(s) {scopes!r} -> '{ms}'"
                    )

        if ms is None and not pr["milestone"]:
            # 4. Neither an override, task-id, nor mapped scope yielded a milestone, and the PR
            # has none on GitHub: surface it (never silent, G2) for manual assignment.
            # Covers scope-less CC titles (`docs: …`) and area-valid-but-milestone-unmapped scopes
            # (`mlir`, `l1`, …) that derive_pr_labels maps cleanly but scope_to_milestone leaves
            # unmapped by design.
            infos.append(
                "no milestone inferable (no task-id; scope absent or unmapped) — assign manually"
            )

        to_add = sorted(desired - pr["labels"])
        set_ms = ms if (ms and ms != pr["milestone"]) else None

        # Structural severity (set by derive_pr_labels), never sniffed from message text:
        # infos are by-design/informational (`~`); flags are genuine refusals-to-invent (`!`).
        # Both are surfaced — never silently dropped (G2).
        for info in infos:
            print(f"   ~ #{number}: {info}", file=sys.stderr)
        for flag in flags:
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
        edits.append((number, edit_args, f"{', '.join(summary)}{via}"))

    updated = 0
    if not dry_run and edits:
        _ok, _fail, failures = run_batch(
            "PR backfill",
            [_pr_edit_task(number, args, line) for number, args, line in edits],
        )
        updated = len(edits) - len(failures)
        for number, err in failures:
            safe_print(
                f"   ! could not update PR #{number} ({err}) — re-run to retry",
                file=sys.stderr,
            )

    print(f">> PRs done — {updated} updated, {in_sync} already in sync")


def _pr_edit_task(number, edit_args, summary_line):
    """Build a batch task that applies one PR's add-only label/milestone edit (disjoint, idempotent)."""

    def task():
        rc, _out, err = run_gh_task(edit_args)
        if rc != 0:
            return (number, False, _stderr_tail(err))
        safe_print(f"   ~ updated #{number}: {summary_line}")
        return (number, True, None)

    task.item = number
    return task


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
    """Return {field_name: {'id', 'options', 'option_objs'}} for single-select fields.

    ``options`` is {option_name: option_id} (used to set an item's field value). ``option_objs``
    is the live option list verbatim — name + color + description — so an additive reconcile can
    re-send every existing option unchanged (``updateProjectV2Field`` matches by name and would
    DELETE any omitted option; we never omit a live one). See ``add_field_options``.
    """
    query = (
        f"query{{ node(id:{gql_str(project_id)}){{ ... on ProjectV2{{ "
        f"fields(first:50){{ nodes{{ __typename "
        f"... on ProjectV2FieldCommon{{ id name }} "
        f"... on ProjectV2SingleSelectField{{ id name "
        f"options{{ id name color description }} }} }} }} }} }} }}"
    )
    nodes = (
        ((gh_graphql(query).get("node") or {}).get("fields") or {}).get("nodes")
    ) or []
    fields = {}
    for node in nodes:
        if not node.get("name"):
            continue
        live = node.get("options", [])
        options = {o["name"]: o["id"] for o in live}
        option_objs = [
            {
                "name": o["name"],
                "color": o.get("color") or "GRAY",
                "description": o.get("description", "") or "",
            }
            for o in live
        ]
        fields[node["name"]] = {
            "id": node["id"],
            "typename": node.get("__typename"),
            "options": options,
            "option_objs": option_objs,
        }
    return fields


def _single_select_options_gql(options):
    """Render a ``singleSelectOptions:[...]`` literal from option dicts (name/color/description).

    Shared by the create + additive-update mutations so colors and quoting are mapped identically.
    """
    return ", ".join(
        f"{{name:{gql_str(o['name'])}, color:{o.get('color') or 'GRAY'}, "
        f"description:{gql_str(o.get('description', ''))}}}"
        for o in options
    )


def create_single_select_field(project_id, field):
    """Create an absent single-select field with all its options (one safe, non-destructive call)."""
    opts = _single_select_options_gql(field.get("options", []))
    query = (
        f"mutation{{ createProjectV2Field(input:{{projectId:{gql_str(project_id)}, "
        f"dataType:SINGLE_SELECT, name:{gql_str(field['name'])}, "
        f"singleSelectOptions:[{opts}]}}){{ projectV2Field{{ "
        f"... on ProjectV2SingleSelectField{{ id name }} }} }} }}"
    )
    gh_graphql(query)


def add_field_options(field_id, full_option_list, dry_run):
    """Additively reconcile an existing single-select field's options via ``updateProjectV2Field``.

    ``full_option_list`` is the **complete** desired option set (the union of live + manifest
    options) — ``updateProjectV2Field`` matches by NAME (existing names keep their option id and
    item assignments; new names are appended; **omitted names are DELETED**), so the caller MUST
    pass every live option to avoid a deletion. ``--dry-run`` mutates nothing.
    """
    if dry_run:
        return
    opts = _single_select_options_gql(full_option_list)
    query = (
        f"mutation{{ updateProjectV2Field(input:{{fieldId:{gql_str(field_id)}, "
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
    ADDITIVELY reconcile missing options on existing fields (union of live + manifest, name-matched,
    never-deleting); FLAG every settings-only view/workflow as a manual step; add absent items; set
    Status/Phase/Area/Priority only where the value drifts.
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
    create, _ = plan_field_reconcile(manifest["fields"], actual_by_name)
    options_added = False
    for name in create:
        field = next(f for f in manifest["fields"] if f["name"] == name)
        if dry_run:
            print(
                f"   + would create field: {name} ({len(field.get('options', []))} options)"
            )
        else:
            create_single_select_field(pid, field)
            print(f"   + created field: {name}")
    # Existing fields: additively reconcile their options via updateProjectV2Field. The mutation
    # matches options by NAME and DELETES any omitted one, so we send the UNION of the field's live
    # options (preserved verbatim — keeps their ids + item assignments) and the manifest's options.
    # New names are appended (never-silent: each printed); a live option absent from the manifest is
    # a would-be deletion — kept in the union and FLAGGED, never silently removed (G2).
    for field in manifest["fields"]:
        name = field["name"]
        live = actual.get(name)
        if not live:
            continue  # absent field was handled by the create path above
        if live.get("typename") != "ProjectV2SingleSelectField":
            # A same-named field of a different type exists on the board — never attempt a
            # single-select option mutation against it (it would error / abort the run). FLAG + skip (G2).
            if field.get("options"):
                print(
                    f"   ! field '{name}' exists on the board but is not a single-select "
                    f"(type {live.get('typename')!r}) — option reconcile skipped; resolve it in the UI.",
                    file=sys.stderr,
                )
            continue
        union, added, extra_live = plan_option_additions(
            live["option_objs"], field.get("options", [])
        )
        for opt in extra_live:
            print(
                f"   ! field '{name}' has live option '{opt}' absent from project.json — "
                f"left in place; remove it in the UI if intended.",
                file=sys.stderr,
            )
        if not added:
            continue
        add_field_options(live["id"], union, dry_run)
        options_added = True
        prefix = "would add " if dry_run else "+ "
        for opt in added:
            print(f"   {prefix}field '{name}' option: {opt}")
    if not dry_run and (create or options_added):
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
    # Declared option order per field (project.json `fields`) — drives the M-676 primary-area
    # anchor: which area:* wins when an item carries several. Built once, passed in (keeps
    # label_to_field_values pure).
    field_option_order = {
        f["name"]: [o["name"] for o in f.get("options", [])]
        for f in manifest.get("fields", [])
    }
    existing = project_items(pid)
    added = set_count = synced = 0
    for rec in contents:
        number, node_id, labels = rec["number"], rec.get("node_id"), rec["labels"]
        values, flags, infos = label_to_field_values(
            labels, field_label_map, field_option_order
        )
        for flag in flags:
            print(f"   ! #{number}: {flag}", file=sys.stderr)
        for info in infos:
            print(f"   i #{number}: {info}")

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
    # scope_to_milestone alias VALUES must be real milestone titles — catch a typo at --validate
    # time, before a runtime `gh pr edit --milestone` would fail on a bad title (Copilot, #304).
    ms_titles = {m["title"] for m in json.loads((here / "milestones.json").read_text())}
    for scope, ms_title in (
        conventions.get("scope_to_milestone", {}).get("aliases", {}).items()
    ):
        if ms_title not in ms_titles:
            errors.append(
                f"conventions.json: scope_to_milestone alias '{scope}' -> '{ms_title}' "
                "is not a milestone title in milestones.json"
            )

    # pr-overrides.json: each override milestone must be a real milestone title; each label must
    # exist in labels.json. A typo must fail --validate before any live run (mirrors the
    # scope_to_milestone validation above — G2: never-invent with a bad value).
    overrides_path = here / "pr-overrides.json"
    if overrides_path.exists():
        try:
            overrides_raw = json.loads(overrides_path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            errors.append(f"pr-overrides.json: JSON parse error — {exc}")
            overrides_raw = {}
        for pr_num_str, entry in overrides_raw.get("overrides", {}).items():
            # PR/issue numbers are positive integers — reject '-67'/typos (mirror load_pr_overrides).
            if not pr_num_str.isdigit():
                errors.append(
                    f"pr-overrides.json: key '{pr_num_str}' is not a valid PR/issue number"
                )
                continue
            ov_ms = entry.get("milestone")
            if not ov_ms:
                errors.append(
                    f"pr-overrides.json: override #{pr_num_str} has no 'milestone' field"
                )
            elif ov_ms not in ms_titles:
                errors.append(
                    f"pr-overrides.json: override #{pr_num_str} milestone '{ov_ms}' "
                    "is not a milestone title in milestones.json"
                )
            # `_`-prefixed entries are informational annotations (mirror load_pr_overrides' filter).
            for lb in entry.get("labels") or []:
                if lb.startswith("_"):
                    continue
                if lb not in labels:
                    errors.append(
                        f"pr-overrides.json: override #{pr_num_str} label '{lb}' "
                        "is not in labels.json"
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
    labels, flags, infos = derive_pr_labels(
        parse_conventional("feat(swap): x"), type_map, areas
    )
    assert labels == {"type:feature", "area:swap"} and flags == [] and infos == [], (
        labels,
        flags,
        infos,
    )
    labels, flags, infos = derive_pr_labels(
        parse_conventional("feat(mlir): x"), type_map, areas
    )
    assert labels == {"type:feature"} and any("mlir" in f for f in flags), (
        labels,
        flags,
    )
    labels, flags, infos = derive_pr_labels(
        parse_conventional("spec(x): y"), type_map, areas
    )
    assert labels == set() and any("unknown commit type" in f for f in flags)
    labels, flags, infos = derive_pr_labels(None, type_map, areas)
    assert labels == set() and flags  # unparsed title is flagged, not invented
    # a declared scope alias turns a FLAG into a mapping; an alias to a non-area still FLAGs.
    labels, flags, infos = derive_pr_labels(
        parse_conventional("feat(mlir): x"),
        type_map,
        {"area:execution"},
        {"mlir": "execution"},
    )
    assert labels == {"type:feature", "area:execution"} and flags == [], (labels, flags)
    labels, flags, infos = derive_pr_labels(
        parse_conventional("feat(zzz): x"),
        type_map,
        {"area:execution"},
        {"zzz": "nope"},
    )
    assert labels == {"type:feature"} and any("alias" in f for f in flags), (
        labels,
        flags,
    )

    # milestone inference: unambiguous → set; multi → anchor to the PRIMARY (first-referenced)
    # task's milestone (NOT the highest phase touched); none → silent.
    t2m = {"M-150": "Phase 1", "M-151": "Phase 1", "M-201": "Phase 2"}
    assert extract_task_ids("does M-150 and M-151", ["M-[0-9]+"]) == ["M-150", "M-151"]
    assert infer_milestone(["M-150", "M-151"], t2m) == ("Phase 1", None)
    assert infer_milestone([], t2m) == (None, None)
    # Multi-milestone: anchors to the FIRST-referenced task's milestone (callers pass task-ids
    # title-first ⇒ the PR's primary task), with an informational span note (never a refusal).
    ms, note = infer_milestone(["M-150", "M-201"], t2m)
    assert ms == "Phase 1", f"expected primary 'Phase 1', got {ms!r}"
    assert (
        note and note.startswith("note:") and "Phase 1" in note and "Phase 2" in note
    ), f"unexpected note: {note!r}"
    # milestone_rank: Phase N prefix → N (int); unprefixed → -1.
    assert milestone_rank("Phase 8 — Toolchain & Release Engineering") == 8
    assert milestone_rank("Phase 0") == 0
    assert milestone_rank("Backlog") == -1
    assert milestone_rank("") == -1
    assert milestone_rank(None) == -1
    # The PRIMARY (first-referenced) task's milestone wins even when a LATER reference is a higher
    # phase — the anchor is the PR's primary work, never over-advanced to the max phase touched.
    t2m_mixed = {"A": "Phase 10 — Future", "B": "Phase 2 — Now", "C": "Backlog"}
    ms2, note2 = infer_milestone(["B", "A"], t2m_mixed)
    assert (
        ms2 == "Phase 2 — Now" and note2 and "Phase 2" in note2 and "Phase 10" in note2
    ), (
        ms2,
        note2,
    )
    # Order-based, not rank-based: the first known milestone wins even if it's unprefixed.
    ms3, note3 = infer_milestone(["C", "B"], t2m_mixed)
    assert ms3 == "Backlog" and note3 and note3.startswith("note:"), (ms3, note3)

    # ── plan_label_migrations (pure, offline) ────────────────────────────────────────────────
    canonical = {
        "type:bug",
        "type:feature",
        "type:docs",
        "good-first-issue",
        "area:swap",
    }
    aliases = {
        "bug": "type:bug",
        "enhancement": "type:feature",
        "documentation": "type:docs",
        "good first issue": "good-first-issue",
    }
    # compliant labels → no migrations, no retirements, no flags.
    migs, rets, flgs = plan_label_migrations(
        ["type:bug", "area:swap"], canonical, aliases
    )
    assert migs == [] and rets == [] and flgs == [], (migs, rets, flgs)
    # mix: two aliased + one unaliased noncompliant.
    migs, rets, flgs = plan_label_migrations(
        ["type:bug", "bug", "enhancement", "orphan-label"], canonical, aliases
    )
    assert migs == [("bug", "type:bug"), ("enhancement", "type:feature")], migs
    assert rets == [] and flgs == ["orphan-label"], (rets, flgs)
    # alias pointing to a non-canonical target → appears in flags, not migrations.
    bad_aliases = {"stale": "nonexistent-canonical"}
    migs2, rets2, flgs2 = plan_label_migrations(["stale"], canonical, bad_aliases)
    assert migs2 == [] and rets2 == [], (migs2, rets2)
    assert any("nonexistent-canonical" in f for f in flgs2), flgs2
    # retire: an unaliased noncompliant label declared in `retire` → retirement, not a flag.
    migs3, rets3, flgs3 = plan_label_migrations(
        ["bug", "wontfix", "orphan-label"], canonical, aliases, {"wontfix", "question"}
    )
    assert migs3 == [("bug", "type:bug")], migs3
    assert rets3 == ["wontfix"] and flgs3 == ["orphan-label"], (rets3, flgs3)
    # an alias WINS over retire (migration preserves the label↔issue link).
    m4, r4, _f4 = plan_label_migrations(["bug"], canonical, aliases, {"bug"})
    assert m4 == [("bug", "type:bug")] and r4 == [], (m4, r4)
    # empty repo labels → all empty (idempotent on an already-clean repo).
    assert plan_label_migrations([], canonical, aliases) == ([], [], [])
    # all canonical → no output.
    assert plan_label_migrations(list(canonical), canonical, aliases) == ([], [], [])

    # ── is_intentional_unmapped_scope (pure) ─────────────────────────────────────────────────────
    for s in (
        "adr-021",
        "ADR-021",
        "dn-16",
        "rfc-0009",
        "rp-8",
        "m-381",
        "m376-p1",
        "wave",
        "wave5",
    ):
        assert is_intentional_unmapped_scope(s), s
    for s in ("toolchain", "llm-harness", "readme", "widget", "core", ""):
        assert not is_intentional_unmapped_scope(s), s
    # in derive_pr_labels: a by-design scope lands in INFOS (structurally), not flags; a genuine
    # unknown scope lands in FLAGS with the plain "is not an area:* label" message.
    _l, fl, inf = derive_pr_labels(
        parse_conventional("docs(adr-021): x"), {"docs": "type:docs"}, areas
    )
    assert (
        _l == {"type:docs"} and fl == [] and any("unmapped by design" in i for i in inf)
    ), (_l, fl, inf)
    _l, fl, inf = derive_pr_labels(
        parse_conventional("feat(widget): x"), type_map, areas
    )
    assert any("is not an area:* label" in f for f in fl) and inf == [], (fl, inf)

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
    vals, vflags, vinfos = label_to_field_values(
        {"phase:8", "area:toolchain", "priority:P3", "status:done"}, field_map
    )
    assert vals == {
        "Phase": "Phase 8",
        "Area": "toolchain",
        "Priority": "P3",
        "Status": "Done",
    }, vals
    assert vflags == [] and vinfos == [], (vflags, vinfos)
    # status:needs-design has no Status option → unmapped (no Status key), not invented.
    vals2, _, _ = label_to_field_values({"phase:0", "status:needs-design"}, field_map)
    assert vals2 == {"Phase": "Phase 0"}, vals2

    # ── M-676: multi-value Area → deterministic primary anchor (never a "not set" punt) ──────────
    area_order = {
        "Area": [
            "core-ir",
            "swap",
            "vsa",
            "execution",
            "numerics",
            "selection",
            "toolchain",
            "project",
            "language",
            "stdlib",
            "release",
            "spec",
        ]
    }
    field_map_multi = dict(field_map)
    field_map_multi["Area"] = {
        "from": "label_prefix",
        "prefix": "area:",
        "template": "{value}",
        "multi": "primary",
    }
    # {language, toolchain}: 'toolchain' precedes 'language' in the declared order → it is primary.
    mvals, mflags, minfos = label_to_field_values(
        {"area:language", "area:toolchain", "phase:5"}, field_map_multi, area_order
    )
    assert mvals["Area"] == "toolchain" and mvals["Phase"] == "Phase 5", mvals
    assert mflags == [] and any(
        "anchored to primary 'toolchain'" in i and "'language'" in i for i in minfos
    ), (mflags, minfos)
    # WITHOUT the multi policy (default rule): the same labels leave Area UNSET + flagged (guard the
    # conflict-flag path that Phase/Priority still rely on; no info emitted).
    dvals, dflags, dinfos = label_to_field_values(
        {"area:language", "area:toolchain"}, field_map, area_order
    )
    assert "Area" not in dvals and dinfos == [], (dvals, dinfos)
    assert any("multiple area:* labels" in f and "not set" in f for f in dflags), dflags
    # A single area:* label is unaffected by the policy (still set, no info, no flag).
    svals, sflags, sinfos = label_to_field_values(
        {"area:swap"}, field_map_multi, area_order
    )
    assert svals == {"Area": "swap"} and sflags == [] and sinfos == [], (
        svals,
        sflags,
        sinfos,
    )
    # Values outside the declared order fall back to a stable alphabetical tie-break (never crash).
    uvals, _, uinfos = label_to_field_values(
        {"area:zzz", "area:aaa"}, field_map_multi, {"Area": []}
    )
    assert uvals["Area"] == "aaa" and any(
        "anchored to primary 'aaa'" in i for i in uinfos
    ), (
        uvals,
        uinfos,
    )
    # The shipped project.json Area rule actually carries multi:primary (config ↔ engine wired).
    _proj = json.loads((HERE / "project.json").read_text(encoding="utf-8"))
    assert _proj["field_label_map"]["Area"].get("multi") == "primary", (
        "project.json Area rule must set multi:primary for M-676"
    )

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

    # ── plan_option_additions (pure, additive, never-destructive) ─────────────────────────────
    # Overlap (kept verbatim) + a new manifest option (appended) + a live-only extra (preserved + flagged).
    live = [
        {"name": "core-ir", "color": "BLUE", "description": "WS-B"},
        {"name": "legacy", "color": "GRAY", "description": "old"},  # live-only extra
    ]
    desired = [
        {
            "name": "core-ir",
            "color": "PURPLE",
            "description": "changed",
        },  # overlap by name
        {"name": "stdlib", "color": "GREEN", "description": "new area"},  # new
    ]
    union, added, extra = plan_option_additions(live, desired)
    # Every live option is preserved verbatim (color/description unchanged — never re-colored).
    assert union[0] == {"name": "core-ir", "color": "BLUE", "description": "WS-B"}, (
        union
    )
    assert union[1] == {"name": "legacy", "color": "GRAY", "description": "old"}, union
    # The genuinely-new manifest option is appended with its manifest color/description.
    assert union[2] == {
        "name": "stdlib",
        "color": "GREEN",
        "description": "new area",
    }, union
    assert [o["name"] for o in union] == ["core-ir", "legacy", "stdlib"], union
    assert added == ["stdlib"], added  # only the new name is reported added
    assert extra == ["legacy"], (
        extra
    )  # live-only option surfaced for a FLAG, not deleted
    # Fully in sync (manifest ⊆ live, no extras) → nothing added, nothing flagged, union == live.
    union2, added2, extra2 = plan_option_additions(
        [{"name": "a", "color": "GRAY", "description": ""}],
        [{"name": "a", "color": "GRAY", "description": ""}],
    )
    assert (
        added2 == []
        and extra2 == []
        and union2 == [{"name": "a", "color": "GRAY", "description": ""}]
    ), (union2, added2, extra2)
    # Empty live (e.g. a field with no options yet) → every manifest option is an add, no extras.
    union3, added3, extra3 = plan_option_additions([], desired)
    assert added3 == ["core-ir", "stdlib"] and extra3 == [] and union3 == desired, (
        union3,
        added3,
        extra3,
    )

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

    # _is_transient_network: the M-382 EOF symptom + the TCP/TLS/io-timeout class must be retryable;
    # auth/rate-limit/generic failures must NOT be (so they fail fast, never spin on backoff).
    assert _is_transient_network(
        'gh: Post "https://api.github.com/...": unexpected EOF'
    )
    assert _is_transient_network("read tcp 1.2.3.4: connection reset by peer")
    assert _is_transient_network("net/http: TLS handshake timeout")
    assert _is_transient_network("dial tcp: i/o timeout")
    assert _is_transient_network("could not resolve host: api.github.com")
    assert _is_transient_network("dial tcp 140.82.x.x: connection refused")
    assert _is_transient_network("network is unreachable")
    assert _is_transient_network("Client.Timeout exceeded while awaiting headers")
    # Non-transient: must fail fast, not retry.
    assert not _is_transient_network("HTTP 401: Bad credentials")
    assert not _is_transient_network("API rate limit exceeded")
    assert not _is_transient_network("HTTP 422: Validation Failed")
    assert not _is_transient_network("")
    assert not _is_transient_network(None)
    # case-insensitive
    assert _is_transient_network("Unexpected EOF")
    # _stderr_tail: last non-empty line, with a stable empty fallback.
    assert _stderr_tail("first\n\nlast line\n") == "last line"
    assert _stderr_tail("") == "(no stderr)"
    assert _stderr_tail(None) == "(no stderr)"

    # ── M-397 bounded-concurrency / rate-negotiation pure logic ─────────────────────────────────
    # should_pause_for_rate_limit: secondary-limit / abuse / 403 / 429 pause; primary-limit + clean
    # do NOT; Retry-After honored (clamped); default backoff otherwise.
    pause, secs = should_pause_for_rate_limit(
        "You have exceeded a secondary rate limit. Retry-After: 30"
    )
    assert pause and secs == 30, (pause, secs)
    pause, secs = should_pause_for_rate_limit(
        "HTTP 403: You have triggered an abuse detection"
    )
    assert pause and secs == 60, (pause, secs)  # no Retry-After ⇒ default 60
    assert should_pause_for_rate_limit("HTTP 429 Too Many Requests")[0] is True
    # Retry-After above the ceiling is clamped; below the floor is raised to ≥1.
    assert (
        should_pause_for_rate_limit("secondary rate limit; retry-after: 9999")[1] == 300
    )
    assert should_pause_for_rate_limit("secondary rate limit; retry-after: 0")[1] == 1
    # A PRIMARY rate-limit is NOT a short pause (it resets hourly) — stay honest, don't absorb it.
    assert should_pause_for_rate_limit("API rate limit exceeded") == (False, 0)
    # Clean / unrelated failures never pause.
    assert should_pause_for_rate_limit("HTTP 422: Validation Failed") == (False, 0)
    assert should_pause_for_rate_limit("") == (False, 0)
    assert should_pause_for_rate_limit(None) == (False, 0)

    # parse_rate_remaining: both API shapes, with a safe None fallback.
    assert parse_rate_remaining({"resources": {"core": {"remaining": 4321}}}) == 4321
    assert parse_rate_remaining({"rate": {"remaining": 17}}) == 17
    assert parse_rate_remaining({"resources": {}}) is None
    assert parse_rate_remaining({}) is None
    assert parse_rate_remaining(None) is None
    assert parse_rate_remaining("not-a-dict") is None

    # negotiate_concurrency: unknown budget ⇒ trust the request; low budget ⇒ serialize; floor ≥ 1.
    assert negotiate_concurrency(6, None) == 6
    assert negotiate_concurrency(6, 5000) == 6
    assert negotiate_concurrency(6, 50) == 1  # below low_water ⇒ clamp to 1
    assert negotiate_concurrency(6, 100) == 6  # at the water line ⇒ unchanged
    assert negotiate_concurrency(0, 5000) == 1  # always at least one worker

    # aggregate_results: counts split + the (item, err) failure sublist preserved in order.
    results = [
        ("a", True, None),
        ("b", False, "boom"),
        ("c", True, None),
        ("d", False, "x"),
    ]
    ok, fail, failures = aggregate_results(results)
    assert ok == 2 and fail == 2 and failures == [("b", "boom"), ("d", "x")], (
        ok,
        fail,
        failures,
    )
    assert aggregate_results([]) == (0, 0, [])

    # _issue_update_args: the gh-edit args + summary mirror the sequential path exactly.
    args, summary = _issue_update_args(
        {
            "labels": (["x"], ["y"]),
            "title": "T",
            "milestone": "M",
            "body": "b",
            "state": "closed",
        }
    )
    assert "--add-label" in args and "--remove-label" in args and "--title" in args, (
        args
    )
    assert args[args.index("--milestone") + 1] == "M"
    assert summary == [
        "labels +x -y",
        "title",
        "milestone=M",
        "body",
        "state=closed",
    ], summary
    assert _issue_update_args({}) == ([], [])

    # partition_issue_work: idmap-number-first match, then title; create / update / in-sync split.
    by_number = {
        10: {
            "number": 10,
            "id": "i10",
            "title": "Keep",
            "labels": {"phase:8"},
            "milestone": None,
            "state": "open",
        },
        11: {
            "number": 11,
            "id": "i11",
            "title": "Drift",
            "labels": set(),
            "milestone": None,
            "state": "open",
        },
    }
    by_title = {rec["title"]: rec for rec in by_number.values()}
    p_issues = [
        {
            "id": "M-1",
            "title": "Keep",
            "labels": ["phase:8"],
        },  # in sync (matched by idmap #10)
        {
            "id": "M-2",
            "title": "Drift",
            "labels": ["phase:8"],
        },  # drift (matched by title #11)
        {"id": "M-3", "title": "Brand New", "labels": []},  # absent ⇒ create
    ]
    p_idmap = {"M-1": 10}
    to_create, to_update, idmap_rows, in_sync = partition_issue_work(
        p_issues, p_idmap, by_number, by_title, do_update=True, update_bodies=False
    )
    assert [e["id"] for e in to_create] == ["M-3"], to_create
    assert [e["id"] for (e, _l, _c) in to_update] == ["M-2"], to_update
    assert ("M-1", 10, "i10") in idmap_rows and ("M-2", 11, "i11") in idmap_rows
    assert in_sync == 1, in_sync
    # do_update=False ⇒ no updates planned, but existing matches still record idmap rows.
    tc2, tu2, rows2, sync2 = partition_issue_work(
        p_issues, p_idmap, by_number, by_title, do_update=False, update_bodies=False
    )
    assert (
        tu2 == []
        and sync2 == 0
        and len(rows2) == 2
        and [e["id"] for e in tc2] == ["M-3"]
    )

    # run_batch: N=1 sequential ⇒ identical aggregation; fault-tolerant (one failure never aborts).
    def _mk(item, ok, err):
        def t():
            return (item, ok, err)

        return t

    ok, fail, failures = run_batch(
        "self-test",
        [_mk("a", True, None), _mk("b", False, "e")],
        concurrency=1,
        summary=False,
    )
    assert ok == 1 and fail == 1 and failures == [("b", "e")], (ok, fail, failures)

    # concurrency>1 aggregates the SAME totals (order of failures may differ under as_completed, so
    # compare as a set); a raising task is captured, not propagated.
    def _boom():
        raise ValueError("kaboom")

    res = run_batch(
        "self-test",
        [_mk("a", True, None), _boom, _mk("c", True, None)],
        concurrency=4,
        summary=False,
        return_results=True,
    )
    ok2, fail2, _f = aggregate_results(res)
    assert ok2 == 2 and fail2 == 1, (ok2, fail2)
    assert any(not r[1] and "kaboom" in str(r[2]) for r in res), res

    # ── infer_milestone_from_scope (pure, offline) ────────────────────────────────────────────
    _P8 = "Phase 8 — Toolchain & Release Engineering"
    _scope_ms = {
        "gh-sync": _P8,
        "gh": _P8,
        "toolchain": _P8,
        "llm-harness": _P8,
    }
    # Single mapped scope → milestone set, no flag.
    ms_s, flag_s = infer_milestone_from_scope(["gh-sync"], _scope_ms)
    assert ms_s == _P8 and flag_s is None, (ms_s, flag_s)

    # Multiple scopes that all agree → milestone set, no flag.
    ms_s2, flag_s2 = infer_milestone_from_scope(["gh", "gh-sync"], _scope_ms)
    assert ms_s2 == _P8 and flag_s2 is None, (ms_s2, flag_s2)

    # Scopes resolving to DIFFERENT milestones → no milestone, flag returned.
    _scope_ms_mixed = {"gh-sync": _P8, "vsa": "Phase 3 — VSA & HDC"}
    ms_s3, flag_s3 = infer_milestone_from_scope(["gh-sync", "vsa"], _scope_ms_mixed)
    assert ms_s3 is None and flag_s3 and "ambiguous" in flag_s3, (ms_s3, flag_s3)

    # No mapped scopes at all → (None, None) — silent; area-label path already flags unmapped.
    ms_s4, flag_s4 = infer_milestone_from_scope(["mlir", "l1"], _scope_ms)
    assert ms_s4 is None and flag_s4 is None, (ms_s4, flag_s4)

    # Empty scope list → (None, None).
    ms_s5, flag_s5 = infer_milestone_from_scope([], _scope_ms)
    assert ms_s5 is None and flag_s5 is None, (ms_s5, flag_s5)

    # Task-id wins over scope: task-id inference resolves a milestone → scope fallback not reached.
    # Simulate: task-id gives Phase 1; scope would give Phase 8. task-id path is called first in
    # reconcile_prs, so if it returns non-None the scope path is skipped. We verify infer_milestone
    # itself here to confirm it returns Phase 1, meaning the caller would never reach scope fallback.
    _t2m_p1 = {"M-001": "Phase 1 — Foundation"}
    ms_tid, note_tid = infer_milestone(["M-001"], _t2m_p1)
    assert ms_tid == "Phase 1 — Foundation" and note_tid is None, (ms_tid, note_tid)
    # (scope fallback would yield Phase 8, but is never consulted because ms_tid is not None)

    # chore(gh-sync): … → scope fallback → Phase 8 (representative real-world case).
    ms_real, flag_real = infer_milestone_from_scope(["gh-sync"], _scope_ms)
    assert ms_real == _P8 and flag_real is None, (ms_real, flag_real)

    # ── apply_pr_override (pure, offline) ─────────────────────────────────────────────────────
    # Override wins with highest precedence; no override → (None, []) so inference path applies.
    _P0 = "Phase 0 — Confirm & Specify"
    _overrides = {
        1: {"milestone": _P0, "labels": []},
        135: {
            "milestone": "Phase 6 — Native Acceleration & Deployment",
            "labels": ["phase:6"],
        },
    }
    # Override present: milestone returned, any labels not already present are in to_add.
    ov_ms, ov_labels = apply_pr_override(1, _overrides, None, set())
    assert ov_ms == _P0 and ov_labels == [], (ov_ms, ov_labels)

    # Override with labels: labels absent from existing set are returned for union-add.
    ov_ms2, ov_labels2 = apply_pr_override(135, _overrides, None, set())
    assert ov_ms2 == "Phase 6 — Native Acceleration & Deployment", ov_ms2
    assert ov_labels2 == ["phase:6"], ov_labels2

    # Label already present on PR → not in the to_add list (idempotent).
    ov_ms3, ov_labels3 = apply_pr_override(135, _overrides, None, {"phase:6"})
    assert (
        ov_ms3 == "Phase 6 — Native Acceleration & Deployment" and ov_labels3 == []
    ), (ov_ms3, ov_labels3)

    # No override → (None, []) — the inference path applies unchanged.
    ov_ms4, ov_labels4 = apply_pr_override(999, _overrides, None, set())
    assert ov_ms4 is None and ov_labels4 == [], (ov_ms4, ov_labels4)

    # Override milestone beats scope fallback — verifying the caller's precedence logic:
    # scope fallback would give _P8; override gives _P0. We check apply_pr_override returns
    # _P0 regardless of what the scope path would have returned (the caller checks override first).
    ov_ms5, _ = apply_pr_override(1, _overrides, _P8, set())
    assert ov_ms5 == _P0, f"override must win over scope fallback: got {ov_ms5!r}"

    # ── issue_override_changes (pure, offline) — the issue-side override path (#67 etc.) ───────
    # Milestone differs → set it; labels add-only; idempotent no-op when already matching.
    _live_bare = {"milestone": None, "labels": set()}
    chg = issue_override_changes(_live_bare, {"milestone": _P0, "labels": ["phase:0"]})
    assert chg.get("milestone") == _P0 and chg.get("labels") == (["phase:0"], []), chg
    # Already on the override milestone, label already present → empty changes (idempotent).
    _live_synced = {"milestone": _P0, "labels": {"phase:0"}}
    assert (
        issue_override_changes(_live_synced, {"milestone": _P0, "labels": ["phase:0"]})
        == {}
    ), "issue override must be a no-op when the issue already matches"
    # Milestone matches but a new label is declared → labels-only change (add-only, never removes).
    _live_ms_only = {"milestone": _P0, "labels": {"keep:me"}}
    chg2 = issue_override_changes(
        _live_ms_only, {"milestone": _P0, "labels": ["phase:0"]}
    )
    assert chg2 == {"labels": (["phase:0"], [])}, chg2

    # ── validate_manifests: pr-overrides validation (pure subset, offline) ────────────────────
    # This tests the validation logic directly without a real filesystem. We call the relevant
    # pure sub-logic: each override milestone must be in ms_titles; each label must be in labels.
    _ms_titles = {_P0, _P8, "Phase 6 — Native Acceleration & Deployment"}
    _label_set = {"phase:6", "phase:0", "type:docs"}

    # Good override: milestone in ms_titles, labels in label set → no errors.
    def _check_overrides(overrides_dict, ms_titles, label_names):
        """PURE sub-logic extracted from validate_manifests for self-test coverage."""
        errs = []
        for pr_num_str, entry in overrides_dict.items():
            if (
                not pr_num_str.isdigit()
            ):  # PR/issue numbers are positive integers (no '-67')
                errs.append(f"#{pr_num_str}: not a valid PR/issue number")
                continue
            ov_ms = entry.get("milestone")
            if not ov_ms:
                errs.append(f"#{pr_num_str}: no milestone")
            elif ov_ms not in ms_titles:
                errs.append(f"#{pr_num_str}: bad milestone '{ov_ms}'")
            for lb in entry.get("labels") or []:
                if lb.startswith("_"):
                    continue  # informational annotation, not a real label
                if lb not in label_names:
                    errs.append(f"#{pr_num_str}: unknown label '{lb}'")
        return errs

    good_ov = {
        "1": {"milestone": _P0, "labels": ["phase:0"]},
        "135": {
            "milestone": "Phase 6 — Native Acceleration & Deployment",
            "labels": ["phase:6"],
        },
    }
    assert _check_overrides(good_ov, _ms_titles, _label_set) == [], _check_overrides(
        good_ov, _ms_titles, _label_set
    )

    # Bad milestone title → error (typo must fail --validate before a live run).
    bad_ms_ov = {"999": {"milestone": "Phase 99 — Nonexistent", "labels": []}}
    errs = _check_overrides(bad_ms_ov, _ms_titles, _label_set)
    assert any("bad milestone" in e for e in errs), errs

    # Bad label name → error.
    bad_lb_ov = {"1": {"milestone": _P0, "labels": ["nonexistent:label"]}}
    errs2 = _check_overrides(bad_lb_ov, _ms_titles, _label_set)
    assert any("unknown label" in e for e in errs2), errs2

    # Missing milestone field → error.
    no_ms_ov = {"1": {"labels": []}}
    errs3 = _check_overrides(no_ms_ov, _ms_titles, _label_set)
    assert any("no milestone" in e for e in errs3), errs3

    # Negative / non-numeric key → error (PR/issue numbers are positive; a typo must not slip past).
    bad_key_ov = {"-67": {"milestone": _P0, "labels": []}}
    errs4 = _check_overrides(bad_key_ov, _ms_titles, _label_set)
    assert any("not a valid PR/issue number" in e for e in errs4), errs4

    # `_`-prefixed annotation labels are skipped (not flagged as unknown).
    annot_ov = {"1": {"milestone": _P0, "labels": ["_note", "phase:0"]}}
    assert _check_overrides(annot_ov, _ms_titles, _label_set) == [], _check_overrides(
        annot_ov, _ms_titles, _label_set
    )

    # ── Relationship/date extraction (issue↔PR landings + dates) — M-RELS ──────────────────────
    # expand_task_id_run: slash-runs expand; standalone ids kept; bare numbers never invented (G2).
    assert expand_task_id_run("stage-1 (M-656/657/658, M-674)") == [
        "M-656",
        "M-657",
        "M-658",
        "M-674",
    ]
    assert expand_task_id_run("epics E7-1/E7-2 + M-665") == ["E7-1", "E7-2", "M-665"]
    assert expand_task_id_run("no ids here") == []
    assert expand_task_id_run("") == []
    # A bare "657" alone is NOT an id (only an explicit M-…/… run promotes a bare number).
    assert expand_task_id_run("see 657 and 658") == []

    # parse_changelog_landings: newest-first wins; basis records the exact header; runs expand.
    _cl_text = (
        "## [Unreleased]\n"
        "### Changed (2026-06-22: M-674 — evaluator on the deep worker stack)\n"
        "- body\n"
        "### Added (2026-06-21: M-656/657 — earlier mention of M-674 too)\n"
        "- body\n"
        "### Added (2026-06-20: no task id in this header)\n"
    )
    _cl = parse_changelog_landings(_cl_text)
    assert _cl["M-674"]["date"] == "2026-06-22", _cl[
        "M-674"
    ]  # topmost wins, not the 06-21 line
    assert _cl["M-656"]["date"] == "2026-06-21" and _cl["M-657"]["date"] == "2026-06-21"
    assert "CHANGELOG.md '### Changed (2026-06-22:" in _cl["M-674"]["basis"]
    assert "M-700" not in _cl  # never invents an id absent from the text

    # parse_git_log_landings: SHA|date|subject; (#NNN) captured; curated subject → pr None; runs ok.
    _gl_text = (
        "abc1234def|2026-06-22|feat(l1): tranche (M-656/657/658, M-674)\n"
        "9999888777|2026-06-21|docs(dfr): discharge (#344)\n"
        "0000111222|2026-06-20|docs(context): M-666 landed (no PR ref)\n"
        "malformed line without pipes\n"
    )
    _gl = parse_git_log_landings(_gl_text)
    assert (
        _gl["M-674"]["sha"] == "abc1234" and _gl["M-674"]["pr"] is None
    )  # short sha; no (#NNN)
    assert _gl["M-666"]["pr"] is None and _gl["M-666"]["date"] == "2026-06-20"
    assert (
        "M-344" not in _gl
    )  # the (#344) is a PR number, not a task id — never conflated

    # epic_of_task_id: explicit E#-# only; M-### carries no epic (never guessed from the number).
    assert epic_of_task_id("E7-1") == "E7" and epic_of_task_id("M-657") is None

    # _status_of: extracts the status:* label value.
    assert _status_of({"labels": ["phase:5", "status:done"]}) == "done"
    assert _status_of({"labels": ["phase:5"]}) is None

    # build_relationship_manifest — STATUS-AWARE HONESTY (the crux): a `status:done` issue gets the
    # strong landed_pr/landed_date; a NOT-done issue gets the weaker evidence_pr/evidence_date with a
    # note (the id was referenced, not completed). CHANGELOG date preferred; git-log (#NNN) preferred
    # for PR; pr_index fills a PR when the subject has none, FLAGs a disagreement (G2).
    _issues = [
        {"id": "M-674", "labels": ["status:done"]},  # done → strong landed_*
        {"id": "M-666", "labels": ["status:in-progress"]},  # not done → weak evidence_*
        {"id": "E7-1", "labels": ["status:needs-design"]},
        {
            "id": "M-999"
        },  # no evidence anywhere → absent from the manifest (never null-guessed)
    ]
    _man = build_relationship_manifest(
        _issues, _cl, _gl, pr_index={"M-666": 343, "M-674": 349}
    )
    # done issue → landed_*; CHANGELOG wins over git-log 06-22; PR from the cross-check index.
    assert (
        _man["M-674"]["landed_date"] == "2026-06-22"
        and _man["M-674"]["landed_pr"] == 349
    )
    assert "evidence_date" not in _man["M-674"]
    # not-done issue → evidence_* (NEVER landed_*), with the honest "REFERENCE/partial" note.
    assert (
        _man["M-666"]["evidence_pr"] == 343
        and _man["M-666"]["evidence_date"] == "2026-06-20"
    )
    assert "landed_pr" not in _man["M-666"] and "landed_date" not in _man["M-666"]
    assert "NOT done" in _man["M-666"]["landed_basis"]
    assert _man["E7-1"]["epic"] == "E7"
    assert "M-999" not in _man  # no grounded evidence ⇒ omitted (G2: never invented)
    # Disagreement is surfaced in the basis, never silently reconciled (done issue → landed_pr).
    _gl_pr = parse_git_log_landings("aaa1111|2026-06-22|feat: x (M-500) (#11)\n")
    _man_dis = build_relationship_manifest(
        [{"id": "M-500", "labels": ["status:done"]}], {}, _gl_pr, pr_index={"M-500": 22}
    )
    assert "FLAG: PR disagreement" in _man_dis["M-500"]["landed_basis"]
    assert (
        _man_dis["M-500"]["landed_pr"] == 11
    )  # git-log (#NNN) wins; index disagreement flagged

    # plan_relationship_enrichment: additive only — an already-present field is never re-added.
    _plan = plan_relationship_enrichment(
        [{"id": "M-674", "landed_date": "2026-06-22"}], _man
    )
    assert "landed_date" not in _plan.get("M-674", {}), (
        _plan
    )  # already present → not in the plan
    assert _plan["M-674"]["landed_pr"] == 349  # the absent field IS planned

    # _insert_fields_into_issue_block: surgical, idempotent, append-only text insertion.
    _src = (
        "issues:\n"
        "  - id: M-674\n"
        '    title: "x"\n'
        "    depends_on: []\n"
        "    body: |\n"
        "      hello\n"
        "  - id: M-675\n"
        '    title: "y"\n'
    )
    _new, _added = _insert_fields_into_issue_block(
        _src, "M-674", {"landed_pr": 349, "landed_date": "2026-06-22"}
    )
    assert "landed_pr: 349" in _new and "landed_date: 2026-06-22" in _new
    assert _added == ["landed_pr", "landed_date"]
    assert yaml.safe_load(_new)["issues"][0]["landed_pr"] == 349  # still valid YAML
    assert (
        yaml.safe_load(_new)["issues"][1]["id"] == "M-675"
    )  # the next block is untouched
    # Idempotent: re-inserting the same field adds nothing.
    _new2, _added2 = _insert_fields_into_issue_block(_new, "M-674", {"landed_pr": 349})
    assert _added2 == [] and _new2 == _new
    # A missing block is a no-op (never invents an entry).
    _new3, _added3 = _insert_fields_into_issue_block(_src, "M-000", {"landed_pr": 1})
    assert _added3 == [] and _new3 == _src

    # _yaml_scalar: ints bare; basis-style strings with ':'/'#'/quotes are quoted + round-trip.
    assert _yaml_scalar(349) == "349"
    _basis = (
        "CHANGELOG.md '### Added (2026-06-22: M-674 — x)'; git log abc1234 '(#349)'"
    )
    _q = _yaml_scalar(_basis)
    assert yaml.safe_load(f"v: {_q}")["v"] == _basis  # exact round-trip through YAML

    # _next_link: extracts rel="next"; absent → None.
    assert (
        _next_link('<https://api.github.com/x?page=2>; rel="next", <...>; rel="last"')
        == "https://api.github.com/x?page=2"
    )
    assert _next_link('<...>; rel="prev"') is None

    print(
        "self-test OK: label_delta, normalize_body, plan_issue_update, parse_conventional, "
        "derive_pr_labels, milestone_rank, infer_milestone (multi-milestone), "
        "infer_milestone_from_scope (scope fallback), "
        "apply_pr_override (declared override precedence), "
        "issue_override_changes (issue-side override), "
        "plan_label_migrations, label_to_field_values, plan_field_reconcile, "
        "plan_option_additions, required_scopes, missing_scopes, over_grants, _auth_command, "
        "_is_transient_network, _stderr_tail, should_pause_for_rate_limit, parse_rate_remaining, "
        "negotiate_concurrency, aggregate_results, _issue_update_args, partition_issue_work, run_batch, "
        "expand_task_id_run, parse_changelog_landings, parse_git_log_landings, epic_of_task_id, "
        "_status_of, build_relationship_manifest (status-aware landed_/evidence_), "
        "plan_relationship_enrichment, _insert_fields_into_issue_block, _yaml_scalar, _next_link"
    )


# ─────────────────────────────────────────────────────────────────────────────────────────────
# Relationship/date extraction driver (--relationships) — M-RELS
# ─────────────────────────────────────────────────────────────────────────────────────────────
def _git_log_lines(repo_root, ref):
    """Return ``git log <ref> --format='%H|%ad|%s' --date=short`` output, or '' if git/ref absent.

    Never-silent on a real failure path: a missing ref/git is reported as a skip (the offline date
    source is the CHANGELOG; git-log is the corroborating PR/SHA source). Pure-IO, read-only.
    """
    for candidate in (ref, "main", "HEAD"):
        try:
            res = subprocess.run(
                [
                    "git",
                    "-C",
                    str(repo_root),
                    "log",
                    candidate,
                    "--format=%H|%ad|%s",
                    "--date=short",
                ],
                capture_output=True,
                text=True,
                check=False,
            )
        except FileNotFoundError:
            print(
                "   ~ git not found — skipping the git-log PR/SHA corroboration (CHANGELOG only)."
            )
            return ""
        if res.returncode == 0 and res.stdout.strip():
            if candidate != ref:
                print(
                    f"   ~ git ref '{ref}' unavailable; using '{candidate}' for landing SHAs."
                )
            return res.stdout
    print(
        f"   ~ no usable git ref ({ref}/main/HEAD) — CHANGELOG-only dates (git-log corroboration skipped)."
    )
    return ""


def load_pr_index(here):
    """Load the optional ``pr-index.json`` ({task_id: pr_number}) cross-check map, or {}.

    This file records the issue↔PR mapping DERIVED FROM THE LIVE MERGED-PR LIST (GitHub REST/MCP),
    so the offline ``--relationships`` run can cross-check the git-log subjects against the real
    merged-PR numbers without a network call. Absence is fine (returns {}); a malformed file is
    never-silent (it raises with the path). Keys may be bare ints or strings; values are PR numbers.
    """
    path = here / "pr-index.json"
    if not path.exists():
        return {}
    try:
        raw = json.loads(path.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError) as exc:
        raise RuntimeError(f"pr-index.json is present but unreadable: {exc}")
    data = raw.get("map", raw) if isinstance(raw, dict) else {}
    out = {}
    for k, v in data.items() if isinstance(data, dict) else []:
        if str(k).startswith("_"):  # allow _about / _comment annotation keys
            continue
        try:
            out[str(k)] = int(v)
        except (TypeError, ValueError):
            raise RuntimeError(
                f"pr-index.json: PR number for '{k}' is not an int: {v!r}"
            )
    return out


def _insert_fields_into_issue_block(text, task_id, adds):
    """Return ``text`` with ``adds`` (dict of field->value) inserted into the ``- id: <task_id>``
    block, after its ``depends_on:`` line (or after ``id:`` when none). Surgical text edit — it does
    NOT reformat the file (preserving comments/order/round-trip; PyYAML dump would clobber them).

    Idempotent: a field already textually present in the block is skipped (so re-running adds
    nothing). Returns ``(new_text, added_fields_list)``. Never reorders or rewrites existing lines
    (append-only, house-rule 3).
    """
    lines = text.splitlines(keepends=True)
    # Find the block: a line matching ``^  - id: <task_id>$`` (allow trailing spaces).
    id_re = re.compile(rf"^(\s*)- id:\s*{re.escape(task_id)}\s*$")
    start = None
    indent = "    "
    for i, ln in enumerate(lines):
        m = id_re.match(ln.rstrip("\n"))
        if m:
            start = i
            indent = m.group(1) + "  "  # field indent = id indent + 2
            break
    if start is None:
        return text, []  # block not found (never invent one)
    # Block end = next ``- id:`` at the same list indent, or EOF.
    list_indent = id_re.match(lines[start].rstrip("\n")).group(1)
    next_re = re.compile(rf"^{re.escape(list_indent)}- id:\s")
    end = len(lines)
    for j in range(start + 1, len(lines)):
        if next_re.match(lines[j]):
            end = j
            break
    block = lines[start:end]
    present = set()
    depends_idx = None
    for k, ln in enumerate(block):
        fm = re.match(r"^\s*([a-z_]+):", ln)
        if fm:
            present.add(fm.group(1))
        if re.match(r"^\s*depends_on:", ln):
            depends_idx = k
    new_field_lines = []
    added = []
    for field, value in adds.items():
        if field in present:
            continue  # idempotent: do not duplicate an existing field
        new_field_lines.append(f"{indent}{field}: {_yaml_scalar(value)}\n")
        added.append(field)
    if not new_field_lines:
        return text, []
    # Insert after depends_on (keeps the relational fields grouped with depends_on); else after id.
    insert_at_in_block = (depends_idx + 1) if depends_idx is not None else 1
    new_block = (
        block[:insert_at_in_block] + new_field_lines + block[insert_at_in_block:]
    )
    new_lines = lines[:start] + new_block + lines[end:]
    return "".join(new_lines), added


def _yaml_scalar(value):
    """Render a scalar for a single-line YAML field value, quoting strings that need it.

    Ints stay bare; a string with YAML-special characters (``:``/``#``/quotes/leading-special) is
    double-quoted with internal quotes/backslashes escaped. Keeps the inserted line valid + round-
    trippable. (The basis string can contain ``:``/``#``/quotes, so this matters.)
    """
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, int):
        return str(value)
    s = str(value)
    needs_quote = (
        s == ""
        or s[0] in "!&*?|>%@`\"'#-{}[],: "
        or s[-1] == " "
        or ": " in s
        or " #" in s
        or any(c in s for c in '":#')
    )
    if needs_quote:
        esc = s.replace("\\", "\\\\").replace('"', '\\"')
        return f'"{esc}"'
    return s


def run_relationships(args):
    """Extract the issue↔PR + date relationship manifest from CHANGELOG.md + git log and (unless
    --dry-run) enrich issues.yaml additively. Idempotent, never-silent (G2), append-only.

    Returns True on success. The live GitHub targets that this manifest FEEDS — issue dependencies,
    sub-issue links, and Projects-v2 field values — are reported as token/Projects-API-gated FLAGs
    (see project-v2-spec.md): this offline driver produces + validates the manifest; the live push
    is a separate, token-scoped step (--use-api or the maintainer's `gh project` run).
    """
    repo_root = HERE.parents[1]
    print(">> relationships: extract issue↔PR + dates from CHANGELOG.md + git log")
    spec = yaml.safe_load(args.issues_yaml.read_text(encoding="utf-8")) or {}
    issues = spec.get("issues", [])
    changelog_path = repo_root / "CHANGELOG.md"
    changelog = (
        changelog_path.read_text(encoding="utf-8") if changelog_path.exists() else ""
    )
    if not changelog:
        print(
            "   ~ CHANGELOG.md not found — no dated landing evidence; nothing to extract."
        )
    cl = parse_changelog_landings(changelog)
    gl = parse_git_log_landings(_git_log_lines(repo_root, args.git_ref))

    # PR cross-check index: a live API enumeration ONLY when --use-api is passed (opt-in); a token
    # alone never triggers network calls (CI often sets GITHUB_TOKEN by default). Otherwise the
    # committed pr-index.json (derived from the live merged-PR list), else empty.
    pr_index = {}
    token = github_token()
    if args.use_api:
        if token:
            try:
                conventions = json.loads(
                    args.conventions_json.read_text(encoding="utf-8")
                )
                patterns = conventions["milestone_inference"]["task_id_patterns"]
                api = GitHubApi(token, dry_run=args.dry_run)
                pr_index = api_merged_pr_index(api, args.repo, patterns)
                print(
                    f"   = live merged-PR cross-check via token API: {len(pr_index)} id↔PR links."
                )
            except RuntimeError as exc:
                print(
                    f"   ! live PR cross-check unavailable ({exc}); falling back to pr-index.json."
                )
                pr_index = load_pr_index(HERE)
        else:
            print(
                "   ! --use-api given but no GITHUB_TOKEN/GH_TOKEN — FLAG: token-gated; using pr-index.json."
            )
            pr_index = load_pr_index(HERE)
    else:
        pr_index = load_pr_index(HERE)
        if pr_index:
            print(
                f"   = PR cross-check from committed pr-index.json: {len(pr_index)} id↔PR links."
            )
        else:
            print(
                "   ~ no token and no pr-index.json — PR numbers come only from git-log (#NNN) subjects."
            )

    manifest = build_relationship_manifest(issues, cl, gl, pr_index)
    plan = plan_relationship_enrichment(issues, manifest)
    print(
        f"   = manifest: {len(manifest)}/{len(issues)} issues have grounded landing evidence; "
        f"{len(plan)} need additive field(s)."
    )

    # Never-silent preview/apply of the additive enrichment.
    if not plan:
        print(
            "   = issues.yaml already carries every grounded field (idempotent — zero writes)."
        )
    elif args.dry_run:
        for tid in sorted(plan):
            fields = ", ".join(
                f"{k}={plan[tid][k]!r}" for k in plan[tid] if k != "landed_basis"
            )
            print(f"   [dry-run] {tid}: + {fields}")
        print(
            f"   [dry-run] would enrich {len(plan)} issue block(s) in {args.issues_yaml.name}."
        )
    else:
        text = args.issues_yaml.read_text(encoding="utf-8")
        total_added = 0
        for tid in sorted(plan):
            text, added = _insert_fields_into_issue_block(text, tid, plan[tid])
            if added:
                total_added += 1
                print(f"   + {tid}: added {', '.join(added)}")
        # Validate the result parses before writing (never leave issues.yaml broken — G2).
        try:
            yaml.safe_load(text)
        except yaml.YAMLError as exc:
            raise RuntimeError(
                f"refusing to write: enriched issues.yaml does not parse: {exc}"
            )
        args.issues_yaml.write_text(text, encoding="utf-8")
        print(
            f"   = enriched {total_added} issue block(s); issues.yaml re-validated OK."
        )

    # Honest capability FLAGs for the live-sync targets this manifest feeds.
    print(
        "   FLAGs (live-sync targets — token/Projects-API-gated, NOT performed by this offline run):"
    )
    print(
        "     - issue dependencies (depends_on → 'blocked by'): REST, needs a token (--use-api)."
    )
    print(
        "     - sub-issue links (epic → children): REST, needs a token (or the MCP sub_issue_write)."
    )
    print(
        "     - Projects-v2 field values (Status / Start / Target date): GraphQL, needs a"
    )
    print(
        "       `project`-scoped token; Declared until run live (project-v2-spec.md)."
    )
    return True


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
            ]
            + (["--debug"] if DEBUG else []),
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
        "--relationships",
        action="store_true",
        help="extract issue↔PR + landing dates from CHANGELOG.md + git log and enrich issues.yaml "
        "additively (append-only). Offline + idempotent; --dry-run previews. Reports the "
        "token/Projects-API-gated live-sync targets as FLAGs.",
    )
    parser.add_argument(
        "--git-ref",
        default="origin/main",
        help="git ref whose squash subjects carry the landing PRs/SHAs (default origin/main; "
        "falls back to main/HEAD when absent).",
    )
    parser.add_argument(
        "--use-api",
        action="store_true",
        help="use the gh-CLI-INDEPENDENT token API (GITHUB_TOKEN/GH_TOKEN over urllib) for the live "
        "merged-PR cross-check (and the Projects-v2 path). Without a token this FLAGs the gated step "
        "and falls back to the committed pr-index.json — never a fabricated sync (G2).",
    )
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
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="echo every `gh` invocation to stderr before it runs (pinpoints a hang to the call)",
    )
    parser.add_argument(
        "--concurrency",
        type=int,
        default=_DEFAULT_CONCURRENCY,
        metavar="N",
        help=(
            "max in-flight `gh` calls per batch (default 6, conservative for GitHub secondary "
            "limits). N=1 reproduces the exact sequential behaviour (clean --verbose/debug "
            "fallback). The pool auto-PAUSES on a secondary-rate-limit/abuse/Retry-After signal."
        ),
    )
    parser.add_argument(
        "--no-rate-probe",
        action="store_true",
        help="skip the start-of-run `gh api rate_limit` budget probe (which can reduce N when the "
        "remaining budget is low); use the requested --concurrency unchanged.",
    )
    parser.add_argument(
        "--debug",
        action="store_true",
        help="debug mode: print the full Python traceback on an unexpected failure (and imply "
        "--verbose) — for investigating manifest/sync issues further.",
    )
    args = parser.parse_args()

    if args.self_test:
        self_test()
        return

    # Wire the module-level flags so _run_gh echoes each call (M-382 diagnosability) and the
    # top-level guard can show a full traceback under --debug.
    global VERBOSE, DEBUG
    DEBUG = args.debug
    VERBOSE = args.verbose or args.debug  # debug implies verbose

    # --relationships is a standalone, offline-by-default extraction + additive enrichment op
    # (it consults the token API only with --use-api/a token). It never reconciles labels/issues.
    if args.relationships:
        mode = "dry-run" if args.dry_run else "apply"
        print("=" * 60)
        print(f">> Mycelium relationship/date extraction — repo: {args.repo}  ({mode})")
        print("=" * 60)
        sys.exit(0 if run_relationships(args) else 1)

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

    # M-397 — set the run-scoped worker count. A live run optionally probes the rate budget and
    # reduces N when it is low (never-silent); --dry-run mutates nothing so it stays sequential
    # (N=1) for a stable, ordered preview. The shared RATE_GATE pauses the whole pool on a
    # secondary-rate-limit signal mid-run (see run_gh_task).
    global CONCURRENCY
    requested = max(1, args.concurrency)
    if args.dry_run:
        CONCURRENCY = 1
    elif args.no_rate_probe or args.no_preflight:
        CONCURRENCY = requested
        if requested > 1:
            print(f">> concurrency: {requested} worker(s) (rate probe skipped)\n")
    else:
        print(">> concurrency: negotiating worker count against the gh rate budget")
        CONCURRENCY = probe_rate_budget(requested)
        print()

    if do_labels:
        reconcile_labels(args.repo, args.labels_json, args.dry_run, args.issues_yaml)
        print()
    if do_milestones:
        reconcile_milestones(args.repo, args.milestones_json, args.dry_run)
        print()
    if do_issues:
        spec = yaml.safe_load(args.issues_yaml.read_text(encoding="utf-8"))
        issues = spec.get("issues", []) if spec else []
        # Declared overrides also apply to ISSUES (e.g. closed #67 — not an issues.yaml task). Same
        # manifest the --prs path consumes; never-silent on a present-but-malformed file.
        reconcile_issues(
            args.repo,
            issues,
            args.idmap,
            do_update=not args.no_update,
            update_bodies=args.update_bodies,
            dry_run=args.dry_run,
            overrides=load_pr_overrides(HERE),
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
        # Load declared per-PR/issue overrides (highest precedence — G2 escape hatch).
        # load_pr_overrides tolerates absence (returns {}); never-silent on a present-but-malformed file.
        pr_overrides = load_pr_overrides(HERE)
        reconcile_prs(
            args.repo,
            conventions,
            area_set,
            task_to_ms,
            dry_run=args.dry_run,
            overrides=pr_overrides,
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
    except Exception as exc:  # pragma: no cover - last-resort guard
        if DEBUG:
            import traceback

            traceback.print_exc()
        sys.exit(
            f"ERROR: unexpected failure: {type(exc).__name__}: {exc}\n"
            "  (this should have been an explicit, classified error — please report it; "
            "re-run with --debug for the full traceback.)"
        )
