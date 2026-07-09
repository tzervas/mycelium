#!/usr/bin/env bash
# Lightweight production release artifact (DN-97 §4.1 Rank 1, maintainer-ratified 2026-07-09).
#
# All three persistent trunks (dev/integration/main) carry SAME-CONTENT tracked trees -- tiers
# differ by RIGOR, not content. "Lightweight production" is therefore a PACKAGING step, not a
# divergent filtered branch: `git archive` over `main`, honoring the `export-ignore` markers in
# `.gitattributes`, which strips dev-only tooling from the shipped tarball while leaving it fully
# committed, reviewed, and diffed on every trunk (export-ignore affects ONLY `git archive` output).
#
# Skip-graceful (never a hard crash) when the target ref isn't resolvable in this checkout -- same
# posture as the rest of the `just check` gate suite ("skip gracefully when a tool/language isn't
# present yet"). Never-silent (G2) about what it excludes: always prints the export-ignore list
# actually in effect on the archived ref before writing the tarball.
#
# Usage: scripts/dist/package-release.sh [version] [ref]
#   version   tarball version tag (default: `git describe` off the ref, else "0.0.0-dev")
#   ref       git ref to archive (default: main; falls back to origin/main if main isn't local)
set -uo pipefail
cd "$(git rev-parse --show-toplevel 2>/dev/null)" || {
  echo "package-release: SKIP -- not inside a git checkout." >&2
  exit 0
}

ref="${2:-main}"
if ! git rev-parse --verify --quiet "$ref" >/dev/null 2>&1; then
  if git rev-parse --verify --quiet "origin/$ref" >/dev/null 2>&1; then
    ref="origin/$ref"
  else
    echo "package-release: SKIP -- '$ref' is not resolvable in this checkout (no local main, no fetched origin/main)." >&2
    echo "  fix: git fetch origin main   (or pass an explicit ref: just package-release <version> <ref>)" >&2
    exit 0
  fi
fi

version="${1:-}"
if [[ -z "$version" ]]; then
  version="$(git describe --tags --always "$ref" 2>/dev/null || echo "0.0.0-dev")"
fi

out_dir="dist"
mkdir -p "$out_dir"
out="$out_dir/mycelium-${version}.tar.gz"

echo "package-release: archiving '$ref' -> $out (honoring .gitattributes export-ignore)"

# Never-silent: report the export-ignore rules actually in effect ON THE ARCHIVED REF -- git
# archive reads .gitattributes from the ref itself, so we do too, rather than the working tree's.
echo "package-release: excluded (export-ignore, dev-only tooling -- committed + reviewed on every trunk, never shipped in this artifact):"
excluded="$(git show "$ref:.gitattributes" 2>/dev/null | grep -E '[[:space:]]export-ignore([[:space:]]|$)' || true)"
if [[ -n "$excluded" ]]; then
  while IFS= read -r line; do echo "  - $line"; done <<<"$excluded"
else
  echo "  (none found -- .gitattributes on $ref carries no export-ignore rules; the archive is unfiltered)"
fi

if ! git archive --format=tar.gz --output="$out" "$ref"; then
  echo "package-release: FAIL -- git archive failed for ref '$ref'." >&2
  exit 1
fi

size="$(du -h "$out" 2>/dev/null | cut -f1)"
echo "package-release: wrote $out (${size:-unknown size})"
