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

# Generator logic gate: determinism + completeness + module-aware attribution (offline).
if python3 tools/docgen/code_index.py --self-test >/dev/null 2>&1; then
  ok "code_index self-test (determinism · completeness · module attribution)"
else
  python3 tools/docgen/code_index.py --self-test || true
  fail "code_index self-test failed"
  exit 1
fi
