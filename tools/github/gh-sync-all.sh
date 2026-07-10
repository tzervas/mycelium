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

# This wrapper ALWAYS runs the full suite (--all), so it forwards only the modifier flags. A single
# level (--labels/--milestones/--issues/--prs/--project/--validate) is NOT a wrapper mode — it would
# be a redundant no-op under --all — so we reject it and point at the engine instead of pretending.
# --all routes through the engine's preflight, which is where the SHARED least-privilege auth
# automation lives (DRY/KC-3 — implemented once in the engine; both wrappers trigger it via --all).
# --no-auth-fix is the CI escape hatch (never prompt to change scopes).
ENGINE_ARGS=(--all --repo "$REPO")
for arg in "$@"; do
  case "$arg" in
    --dry-run|--update-bodies|--no-preflight|--no-auth-fix) ENGINE_ARGS+=("$arg") ;;
    --all) : ;;  # already implied
    --labels|--milestones|--issues|--prs|--project|--validate)
      echo "gh-sync-all always runs the FULL suite (--all); '$arg' would be a redundant no-op." >&2
      echo "for a single level, call the engine directly, e.g.:" >&2
      echo "  python tools/github/gh-issues-sync.py $arg --dry-run" >&2
      exit 2 ;;
    *) echo "unknown argument: $arg (accepted: --dry-run --update-bodies --no-preflight --no-auth-fix)" >&2
       exit 2 ;;
  esac
done

# Prefer uv's managed dev venv — tools/github/pyproject.toml declares PyYAML, so `uv run` executes
# the engine against a reproducible, isolated `.venv` (auto-synced from pyproject.toml + uv.lock)
# rather than a system-wide PyYAML. `--project "$HERE"` pins the project regardless of the caller's
# CWD; uv does NOT change directory (same relative-path behavior as the bare-python path below).
# Fall back to a bare python3 (+ system PyYAML) when uv is absent (minimal CI images / Termux). Both
# drive the SAME engine (DRY). Set MYC_GH_SYNC_NO_UV=1 to force the bare-python path.
if [[ "${MYC_GH_SYNC_NO_UV:-0}" != "1" ]] && command -v uv >/dev/null 2>&1 && [[ -f "$HERE/pyproject.toml" ]]; then
  exec uv run --project "$HERE" python "$HERE/gh-issues-sync.py" "${ENGINE_ARGS[@]}"
fi

# Fallback: resolve a python interpreter (python3 preferred; Termux often has bare `python`).
PY=""
for cand in python3 python; do
  if command -v "$cand" >/dev/null 2>&1; then PY="$cand"; break; fi
done
[[ -n "$PY" ]] || { echo "ERROR: no python3/python on PATH (and no uv) — install uv, or python3 + PyYAML" >&2; exit 1; }

exec "$PY" "$HERE/gh-issues-sync.py" "${ENGINE_ARGS[@]}"
