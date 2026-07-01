#!/usr/bin/env bash
# Lint markdown with markdownlint-cli2 (config: .markdownlint.jsonc).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "markdownlint"

tracked '*.md'
if [[ ${#TRACKED[@]} -eq 0 ]]; then skip "no markdown"; exit 0; fi
if ! have npx; then skip "node/npx not found — run \`just setup\`"; exit 0; fi

# Pin the version (B1-02): unpinned `npx --yes markdownlint-cli2` fetches LATEST, which is a
# silent supply-chain + reproducibility risk. Bump deliberately (Dependabot can't see npx pins).
if npx --yes markdownlint-cli2@0.22.1 --config .markdownlint.jsonc "${TRACKED[@]}"; then
  ok "${#TRACKED[@]} doc(s) clean"
else
  fail "markdownlint findings"; exit 1
fi
