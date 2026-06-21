#!/usr/bin/env bash
# Propagate the latest origin/main DOWN into every persistent head branch.
# Wave-N workflow (CLAUDE.md): only `main` squashes; after a head lands on main, the squashed main
# must flow back down so no head/child keeps building on a superseded base (mitigation #6: pull-down
# flows down). Heads merge main with --no-ff (lineage preserved; squashing is main-only). Conflicts
# are FLAGGED, never force-resolved — the owning session resolves them on its head.
#
# Usage: scripts/sync-heads.sh            # sync all claude/head/* with origin/main
#        scripts/sync-heads.sh <head>     # sync one head
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 1

git fetch origin main --quiet || { echo "sync-heads: cannot fetch origin/main" >&2; exit 1; }

heads=()
if [[ $# -ge 1 ]]; then
  heads=("$1")
else
  mapfile -t heads < <(git ls-remote --heads origin 'claude/head/*' 2>/dev/null \
    | awk '{print $2}' | sed 's#refs/heads/##')
fi
[[ ${#heads[@]} -eq 0 ]] && { echo "sync-heads: no claude/head/* branches on origin"; exit 0; }

rc=0
for head in "${heads[@]}"; do
  echo "== sync $head <- origin/main =="
  git fetch origin "$head" --quiet 2>/dev/null || { echo "  (no origin/$head; skip)"; continue; }
  wt="$(mktemp -d)"
  if ! git worktree add --quiet "$wt" "origin/$head" 2>/dev/null; then echo "  (worktree add failed; skip)"; rm -rf "$wt"; continue; fi
  if git -C "$wt" merge --no-ff origin/main -m "chore(sync): propagate squashed main into $head" --quiet 2>/dev/null; then
    if git -C "$wt" push origin "HEAD:$head" --quiet 2>/dev/null; then echo "  ok: pushed $head"; else echo "  push failed for $head"; rc=1; fi
  else
    echo "  !! CONFLICT merging main into $head — the owning session must resolve on its head (not here)"
    git -C "$wt" merge --abort 2>/dev/null || true
    rc=1
  fi
  git worktree remove --force "$wt" 2>/dev/null || true
done
exit $rc
