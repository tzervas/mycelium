#!/usr/bin/env bash
# sugar-index — drift gate: committed docs/sugar-index/ must match a fresh regeneration from
# tools/grammar/sugar.yaml, which must itself cross-check clean against the lexer keyword() table
# (crates/mycelium-l1/src/token.rs) — the DN-38 §6 per-feature Lowering Map realized as a
# generated, drift-gated artifact (v0). Mirrors scripts/checks/doc-index.sh's
# regenerate-to-temp-and-diff shape, adapted for tools/grammar/sugar_index.py. Skip-graceful if
# python3 (or pyyaml) is absent.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "$SCRIPT_DIR/../lib.sh"
cd "$REPO_ROOT" || exit 1

section "sugar-index"
# Specific reason sub-codes (consumed by all.sh's packed exit byte): 2 = committed index is stale
# (run `just sugar-index-gen` + commit), 3 = generator self-test failed, 4 = sugar.yaml fails its
# cross-check against token.rs::keyword() (a lexer keyword drifted from the registry). 0 = current.

if ! have python3; then
  skip "python3 not found — install it or run: just setup"
  exit 0
fi

if ! python3 -c "import yaml" >/dev/null 2>&1; then
  skip "pyyaml not installed — install it or run: just setup"
  exit 0
fi

# Generator logic gate first: determinism + the real registry's own cross-check + the synthetic
# drift-guard demos (offline, no committed-state dependency beyond sugar.yaml itself).
if python3 tools/grammar/sugar_index.py --self-test >/dev/null 2>&1; then
  ok "sugar_index generator self-test (cross-check · determinism · drift-guard demo)"
else
  python3 tools/grammar/sugar_index.py --self-test || true
  fail "sugar_index generator self-test failed"
  exit 3
fi

# Drift gate: committed artifacts must equal a fresh regeneration from sugar.yaml. The generator
# itself exits 4 if sugar.yaml fails its cross-check against token.rs::keyword() — surface that
# distinctly from a plain staleness diff. (Capture the real exit code WITHOUT `!`, which would
# collapse it to 0/1 — set +e around the call instead, per the doc-index.sh precedent.)
check_out=$(mktemp)
set +e
python3 tools/grammar/sugar_index.py --check >"$check_out" 2>&1
rc=$?
set -e
cat "$check_out"
rm -f "$check_out"
if [[ $rc -eq 4 ]]; then
  fail "tools/grammar/sugar.yaml is out of sync with token.rs::keyword() (G2) — see the cross-check output above"
  exit 4
elif [[ $rc -ne 0 ]]; then
  fail "docs/sugar-index/ is stale — run 'just sugar-index-gen' and commit the result"
  exit 2
fi
ok "docs/sugar-index/ is current with tools/grammar/sugar.yaml"
