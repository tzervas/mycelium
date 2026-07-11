#!/usr/bin/env bash
# docs-assets — drift/lint gate for the committed docs/assets/ screenshot set (the docs asset
# automation companion to `just docs-assets` / scripts/docs-assets.sh). Deliberately
# BROWSER-FREE and lightweight (unlike scripts/docs-assets.sh's heavy Playwright capture step): a
# pure grep-based reference-integrity check, mirroring the docs/api-index/ / docs/tero-index/
# committed-generated + drift-gate pattern — scoped down to references rather than pixels because
# a screenshot isn't byte-deterministic across renders (fonts/timestamps) the way a code index is;
# this gate checks that the asset SET is honest (nothing dangling, nothing dead), not that the
# pixels are current.
#
# Fails if:
#   - a committed .md under docs/ references docs/assets/<file> that doesn't exist
#     (referenced-but-missing)
#   - docs/assets/ holds a file that no committed .md under docs/ references
#     (present-but-orphaned)
#
# Specific reason sub-codes (consumed by all.sh's packed exit byte, shared with the
# committed-index-drift family — see all.sh's COMPONENT_ID comment): 2 = orphan(s) present,
# 3 = referenced-but-missing file(s) (checked in that order; both classes are always reported).
# 0 = clean.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "$SCRIPT_DIR/../lib.sh"
cd "$REPO_ROOT" || exit 1

section "docs-assets"

ASSETS_DIR="docs/assets"

tracked '*.md'
if [[ ${#TRACKED[@]} -eq 0 ]]; then skip "no markdown tracked"; exit 0; fi

# Referenced basenames: any `assets/<name>.<ext>` substring in a tracked .md under docs/ (matches
# Markdown image syntax and a bare path alike — deliberately permissive, same spirit as the
# reference check scripts/docs-assets.sh's own prune step runs).
mapfile -t referenced < <(
  for f in "${TRACKED[@]}"; do
    case "$f" in
      docs/*) ;;
      *) continue ;;
    esac
    grep -ohE 'assets/[A-Za-z0-9._-]+\.(png|jpg|jpeg|gif|svg|webp)' "$f" 2>/dev/null || true
  done | sed -E 's#^assets/##' | sort -u
)

is_referenced() {
  local name="$1" r
  for r in "${referenced[@]}"; do
    [[ "$r" == "$name" ]] && return 0
  done
  return 1
}

missing=0
for name in "${referenced[@]}"; do
  if [[ ! -f "$ASSETS_DIR/$name" ]]; then
    fail "referenced-but-missing: a doc under docs/ references assets/$name, but $ASSETS_DIR/$name does not exist"
    missing=$((missing + 1))
  fi
done

orphaned=0
present=0
if [[ -d "$ASSETS_DIR" ]]; then
  shopt -s nullglob
  for f in "$ASSETS_DIR"/*; do
    [[ -f "$f" ]] || continue
    present=$((present + 1))
    name="$(basename "$f")"
    if ! is_referenced "$name"; then
      fail "present-but-orphaned: $ASSETS_DIR/$name is not referenced by any committed doc under docs/"
      orphaned=$((orphaned + 1))
    fi
  done
  shopt -u nullglob
fi

if [[ $missing -eq 0 && $orphaned -eq 0 ]]; then
  ok "docs/assets/ reference-clean (${#referenced[@]} referenced, $present present)"
  exit 0
fi

echo "  fix: run \`just docs-assets\` to recapture + prune, then review \`git status\` and commit"
if [[ $orphaned -gt 0 ]]; then exit 2; fi
exit 3
