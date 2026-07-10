#!/usr/bin/env bash
# Lint markdown with markdownlint-cli2 (config: .markdownlint.jsonc).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "markdownlint"

tracked '*.md'
if [[ ${#TRACKED[@]} -eq 0 ]]; then skip "no markdown"; exit 0; fi
if ! have npx; then skip "node/npx not found — run \`just setup\`"; exit 0; fi

# markdownlint-cli2 (and its transitive `string-width`) uses the ES2024 `/v` regex flag, which needs
# Node >= 20; on older Node it crashes at import (not a lint finding). Skip gracefully (never-silent,
# G2) rather than fail the gate on a toolchain-version mismatch — mirrors the "checks skip gracefully
# when a tool/language isn't present" principle. Install Node 20+ (NodeSource/nvm) for markdown coverage.
NODE_MAJOR="$(node -v 2>/dev/null | sed 's/^v//; s/\..*//')"
if [[ -z "$NODE_MAJOR" || "$NODE_MAJOR" -lt 20 ]]; then
  skip "node $(node -v 2>/dev/null || echo '?') < 20 — markdownlint-cli2 needs Node >= 20; install Node 20+ for markdown lint coverage"
  exit 0
fi

# Pin the version (B1-02): unpinned `npx --yes markdownlint-cli2` fetches LATEST, which is a
# silent supply-chain + reproducibility risk. Bump deliberately (Dependabot can't see npx pins).
if npx --yes markdownlint-cli2@0.22.1 --config .markdownlint.jsonc "${TRACKED[@]}"; then
  ok "${#TRACKED[@]} doc(s) clean"
else
  fail "markdownlint findings — for MD004 \`+\`/\`*\` soft-wraps run \`just md-fix\` to auto-reflow"; exit 1
fi
