#!/usr/bin/env bash
# Mycelium GitHub PM sync — ONE idempotent command to close every gap.
#
# WHY THIS EXISTS
# ---------------
# The PM bootstrap is split across two idempotent tools (each safe to rerun):
#   * gh-bootstrap-local.sh  — labels (create-or-update) + milestones (create-absent)
#   * gh-issues-sync.py      — issues (create-absent-by-title) + milestone assignment + idmap
# Only termux-setup.sh chained them, and only as part of a full device provision. When
# issues.yaml / labels.json / milestones.json gain new entries between runs, you want to
# reconcile the repo with a SINGLE command — without re-provisioning anything. That is this.
#
# It runs, in order:
#   0. manifest-check.py   — preflight: every label/milestone issues.yaml references must be
#                            defined in the manifests (a missing label would otherwise make
#                            `gh issue create --label …` fail mid-run — explicit, not silent).
#   1. gh-bootstrap-local.sh — labels + milestones (so the labels the issues need exist first).
#   2. gh-issues-sync.py     — create absent issues (with labels), assign milestones, append idmap.
#
# Idempotent: rerun any time. Labels are create-or-updated; milestones and issues are created
# only when absent; idmap.tsv is append-only. Nothing is duplicated or rewritten.
#
# Requires: gh (authenticated to the repo owner), jq, python3 (+ PyYAML).
#
# Usage:
#   bash tools/github/gh-sync-all.sh
#   REPO=tzervas/mycelium bash tools/github/gh-sync-all.sh
#   bash tools/github/gh-sync-all.sh --dry-run    # preview issue creation (no repo writes)
set -euo pipefail

REPO="${REPO:-tzervas/mycelium}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export REPO

DRY_RUN=0
for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=1 ;;
    *) echo "unknown argument: $arg (only --dry-run is accepted)" >&2; exit 2 ;;
  esac
done

# Resolve a python interpreter (python3 preferred; Termux often has bare `python`).
PY=""
for cand in python3 python; do
  if command -v "$cand" >/dev/null 2>&1; then PY="$cand"; break; fi
done
[[ -n "$PY" ]] || { echo "ERROR: no python3/python on PATH" >&2; exit 1; }

echo "============================================================"
echo ">> Mycelium PM sync — repo: $REPO  (dry-run: $DRY_RUN)"
echo "============================================================"

echo
echo ">> [0/2] preflight: manifest consistency"
"$PY" "$HERE/manifest-check.py"

echo
echo ">> [1/2] labels + milestones (gh-bootstrap-local.sh)"
if [[ "$DRY_RUN" -eq 1 ]]; then
  echo "   (dry-run: skipping label/milestone writes)"
else
  bash "$HERE/gh-bootstrap-local.sh"
fi

echo
echo ">> [2/2] issues + milestone assignment + idmap (gh-issues-sync.py)"
sync_args=(--repo "$REPO")
[[ "$DRY_RUN" -eq 1 ]] && sync_args+=(--dry-run)
"$PY" "$HERE/gh-issues-sync.py" "${sync_args[@]}"

echo
echo ">> sync complete — repo reconciled with issues.yaml / labels.json / milestones.json."
