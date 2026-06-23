#!/usr/bin/env bash
# drift — editor-grammar drift gate (M-731; RFC-0026). The committed grammars under tools/grammar/
# must match a fresh regeneration from the canonical lexer keyword() table (G2: the lexer can never
# silently diverge from the grammars that colour editors). Skip-graceful if python3 absent.
set -euo pipefail
. "$(dirname "$0")/../lib.sh"
cd "$REPO_ROOT" || exit 1

section "drift"
# Specific reason sub-codes (consumed by all.sh's packed exit byte): 2 = committed grammars are
# stale (run `just grammar-gen` + commit), 3 = generator self-test failed. 0 = current.

if ! have python3; then
  skip "python3 not found — install it or run: just setup"
  exit 0
fi

# Generator logic gate first: extraction + determinism (offline, no committed-state dependency).
if python3 tools/grammar/generate.py --self-test >/dev/null 2>&1; then
  ok "grammar generator self-test (extraction · determinism)"
else
  python3 tools/grammar/generate.py --self-test || true
  fail "grammar generator self-test failed"
  exit 3
fi

# Drift gate: committed artifacts must equal a fresh regeneration from token.rs::keyword().
if python3 tools/grammar/generate.py --check; then
  ok "tools/grammar/ is current with the lexer keyword() table"
else
  fail "editor grammars are stale — run 'just grammar-gen' and commit the result (G2)"
  exit 2
fi
