#!/usr/bin/env bash
# Offline cross-reference / relative-link checker (no network). Catches broken
# intra-repo links and @imports — e.g. a moved file that left a dangling reference.
source "${BASH_SOURCE%/*}/../lib.sh"
cd "$REPO_ROOT"
section "links & cross-references (offline)"

if ! have python3; then skip "python3 not found"; exit 0; fi
if python3 "$REPO_ROOT/scripts/lint_links.py"; then
  ok "all relative links resolve"
else
  fail "broken relative links / cross-references"; exit 1
fi
