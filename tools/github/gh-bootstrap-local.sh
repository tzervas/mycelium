#!/usr/bin/env bash
# Mycelium GitHub bootstrap — LOCAL gaps only (labels + milestones), via the gh CLI.
#
# WHY THIS EXISTS
# ---------------
# The original PM-package gh-bootstrap.sh (not committed here) assumes a host where `gh` is installed and authenticated.
# In the Claude Code *web* sandbox that is not the case: GitHub is reachable only through
# the GitHub MCP server, which can create/update ISSUES but has NO tool to create labels
# (with colors/descriptions) or milestones. So the work was split:
#
#   * Issues (+ labels-on-issues)          -> a model, over MCP   (see mcp-bootstrap.md)
#   * Label colors/descriptions + milestones -> THIS script, locally with gh
#
# This script therefore fills ONLY the two things MCP cannot, and is safe to run before
# or after the issues exist. It emits a milestone title->number map that the MCP runner
# consumes to assign milestones to the already-created issues.
#
# Requires: gh (authenticated to the repo owner), jq.
# Idempotent: `gh label create --force` creates-or-updates; milestones are created only
#             when the title is absent.
#
# Usage:
#   bash tools/github/gh-bootstrap-local.sh
#   REPO=tzervas/mycelium bash tools/github/gh-bootstrap-local.sh
#   MSMAP=/tmp/mycelium-msmap.tsv REPO=... bash tools/github/gh-bootstrap-local.sh
#
# Constraint: if any gh call fails, the script stops (set -e) and the failing command's
# stderr is shown — re-run after fixing; nothing is duplicated.
set -euo pipefail

REPO="${REPO:-tzervas/mycelium}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LABELS="$HERE/labels.json"
MILESTONES="$HERE/milestones.json"
# B1-05: default to a fresh mktemp file rather than a fixed, world-readable /tmp path
# (mild TOCTOU/symlink hardening). An explicit MSMAP=... still overrides for the MCP runner.
MSMAP="${MSMAP:-$(mktemp -t mycelium-msmap.XXXXXX.tsv)}"

echo ">> Repo: $REPO"
command -v gh >/dev/null || { echo "ERROR: gh not found (this script is for a local host with gh; in the web sandbox use mcp-bootstrap.md)"; exit 1; }
command -v jq >/dev/null || { echo "ERROR: jq not found"; exit 1; }
gh auth status >/dev/null 2>&1 || { echo "ERROR: gh not authenticated"; exit 1; }

# 1) Labels — idempotent: --force creates a new label or updates an existing one's color+description.
echo ">> Labels"
jq -c '.[]' "$LABELS" | while read -r row; do
  name=$(jq -r '.name' <<<"$row")
  color=$(jq -r '.color' <<<"$row")
  desc=$(jq -r '.description' <<<"$row")
  gh label create "$name" --color "$color" --description "$desc" --repo "$REPO" --force >/dev/null
  echo "   • $name"
done

# 2) Milestones — create only if the title is absent; always (re)emit the title->number map.
#    Map format (TAB-separated):  <number>\t<title>   — title is everything after the first tab.
echo ">> Milestones"
: > "$MSMAP"
existing_json=$(gh api "repos/$REPO/milestones?state=all" --paginate)
jq -c '.[]' "$MILESTONES" | while read -r row; do
  title=$(jq -r '.title' <<<"$row")
  desc=$(jq -r '.description' <<<"$row")
  state=$(jq -r '.state // "open"' <<<"$row")   # honor milestones.json state (default open)
  num=$(jq -r --arg t "$title" '.[] | select(.title==$t) | .number' <<<"$existing_json" | head -n1)
  if [[ -n "${num:-}" && "$num" != "null" ]]; then
    echo "   = exists #$num: $title"
  else
    num=$(gh api "repos/$REPO/milestones" -f title="$title" -f state="$state" -f description="$desc" | jq -r '.number')
    echo "   + created #$num: $title"
  fi
  printf '%s\t%s\n' "$num" "$title" >> "$MSMAP"
done

echo
echo ">> milestone number -> title map: $MSMAP"
cat "$MSMAP"
echo
echo ">> NEXT (over MCP, see tools/github/mcp-bootstrap.md):"
echo "   - ensure the 40 issues exist (idempotent), then assign each issue's milestone"
echo "     by resolving its issues.yaml 'milestone' title through THIS map to a number."
echo "   - sub-issue / 'blocked by' dependency linking is the Grok pass (uses idmap.tsv)."
