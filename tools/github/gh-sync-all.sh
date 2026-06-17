#!/usr/bin/env bash
# Mycelium GitHub PM sync — ONE idempotent command for the ENTIRE project state.
#
# WHY THIS EXISTS
# ---------------
# The whole reconcile now lives in ONE cross-platform engine, gh-issues-sync.py (pure Python +
# gh — no bash, no jq). This wrapper is the Linux/macOS entry point; gh-sync-all.ps1 is the
# Windows twin. Both drive the same engine with `--all`, the FULL maintenance suite:
#
#   preflight (auth/scope sanity) -> validate (manifests vs codebase) -> labels -> milestones
#   -> issues -> PRs -> project (Project v2 board, when the `project` scope is present).
#
# Every level is create-if-absent + update-to-match + --dry-run + never-silent + idempotent.
# See RECONCILE.md for the full contract. (gh-bootstrap-local.sh remains a standalone bash
# labels+milestones tool, but the engine --all now supersedes it cross-platform.)
#
# Requires: gh (authenticated to the repo owner), python3 (+ PyYAML).
#
# Usage:
#   bash tools/github/gh-sync-all.sh                  # full suite (live)
#   bash tools/github/gh-sync-all.sh --dry-run        # preview the whole reconcile (no writes)
#   bash tools/github/gh-sync-all.sh --update-bodies  # also push issues.yaml bodies
#   REPO=owner/name bash tools/github/gh-sync-all.sh  # override the repo
set -euo pipefail

REPO="${REPO:-tzervas/mycelium}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Forward only the recognised modifier flags to the engine (which runs --all). An explicit single
# level (--labels/--issues/--prs/--project/--validate) is available by calling the engine directly.
ENGINE_ARGS=(--all --repo "$REPO")
for arg in "$@"; do
  case "$arg" in
    --dry-run|--update-bodies|--prs|--project|--validate|--no-preflight)
      ENGINE_ARGS+=("$arg") ;;
    --all) : ;;  # already implied
    *) echo "unknown argument: $arg" >&2
       echo "accepted: --dry-run --update-bodies --prs --project --validate --no-preflight" >&2
       exit 2 ;;
  esac
done

# Resolve a python interpreter (python3 preferred; Termux often has bare `python`).
PY=""
for cand in python3 python; do
  if command -v "$cand" >/dev/null 2>&1; then PY="$cand"; break; fi
done
[[ -n "$PY" ]] || { echo "ERROR: no python3/python on PATH" >&2; exit 1; }

exec "$PY" "$HERE/gh-issues-sync.py" "${ENGINE_ARGS[@]}"
