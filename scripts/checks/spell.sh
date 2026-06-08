#!/usr/bin/env bash
# Spell-check prose with codespell (config: .codespellrc).
source "${BASH_SOURCE%/*}/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "codespell"

if ! have codespell; then skip "codespell not found — run \`just setup\`"; exit 0; fi
if codespell; then
  ok "no spelling issues"
else
  fail "codespell findings (fix, or add false positives to .codespellrc ignore list)"; exit 1
fi
