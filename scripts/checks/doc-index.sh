#!/usr/bin/env bash
# doc-index — drift gate: committed docs/api-index/ must match a fresh regeneration.
# Skip-graceful if python3 absent (same pattern as other tool-gated checks).
set -euo pipefail
. "$(dirname "$0")/../lib.sh"
cd "$REPO_ROOT" || exit 1

section "doc-index"

if ! have python3; then
  skip "python3 not found — install it or run: just setup"
  exit 0
fi

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT

python3 tools/docgen/code_index.py --output-dir "$tmpdir"

if diff -rq "$tmpdir" docs/api-index/ >/dev/null 2>&1; then
  ok "docs/api-index/ is current"
else
  diff -r "$tmpdir" docs/api-index/ || true
  fail "docs/api-index/ is stale — run 'just docs-index' and commit the result"
  exit 1
fi
