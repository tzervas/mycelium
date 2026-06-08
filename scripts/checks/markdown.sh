#!/usr/bin/env bash
# Lint markdown with markdownlint-cli2 (config: .markdownlint.jsonc).
source "${BASH_SOURCE%/*}/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "markdownlint"

tracked '*.md'
if [[ ${#TRACKED[@]} -eq 0 ]]; then skip "no markdown"; exit 0; fi
if ! have npx; then skip "node/npx not found — run \`just setup\`"; exit 0; fi

if npx --yes markdownlint-cli2 --config .markdownlint.jsonc "${TRACKED[@]}"; then
  ok "${#TRACKED[@]} doc(s) clean"
else
  fail "markdownlint findings"; exit 1
fi
