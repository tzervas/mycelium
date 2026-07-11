#!/usr/bin/env bash
# Portable rendered-docs bundle — ships ALONGSIDE the release artifact.
#
# `just package-release` archives the SOURCE tree (docs/** is not export-ignored, so the corpus
# already rides that tarball). This companion bundles the *rendered, browsable* docsite (the themed
# myc-doc corpus site + the agent index + rustdoc, assembled by scripts/docsite.sh) into a portable
# tarball so readers get a ready-to-open site with each released package — no build step needed.
#
# Skip-graceful + never-silent (G2): if the docsite can't be assembled (no cargo, etc.) it says so
# and exits 0 without writing a bundle, rather than failing the release. Deterministic given the
# corpus. The rendered site is a Declared projection of the corpus (source is ground truth).
#
# Usage: scripts/dist/package-docs.sh [version]
#   version  bundle version tag (default: `git describe`, else 0.0.0-dev)
set -uo pipefail
cd "$(git rev-parse --show-toplevel 2>/dev/null)" || {
  echo "package-docs: SKIP — not inside a git checkout." >&2
  exit 0
}

version="${1:-}"
if [[ -z "$version" ]]; then
  version="$(git describe --tags --always 2>/dev/null || echo "0.0.0-dev")"
fi

out_dir="dist"
mkdir -p "$out_dir"
out="$out_dir/mycelium-docs-${version}.tar.gz"

echo "package-docs: assembling the rendered docsite (scripts/docsite.sh)…"
if ! bash scripts/docsite.sh >/dev/null 2>&1 || [[ ! -f target/docsite/index.html ]]; then
  echo "package-docs: SKIP — docsite did not assemble (is cargo present?). No docs bundle written." >&2
  echo "  the SOURCE corpus still ships inside 'just package-release' (docs/ is not export-ignored)." >&2
  exit 0
fi

if ! tar -czf "$out" -C target/docsite .; then
  echo "package-docs: FAIL — could not write $out." >&2
  exit 1
fi

size="$(du -h "$out" 2>/dev/null | cut -f1)"
echo "package-docs: wrote $out (${size:-unknown size}) — portable browsable docs, ships alongside the release."
