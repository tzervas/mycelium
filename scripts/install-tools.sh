#!/usr/bin/env bash
# Best-effort install of the local check tools. Idempotent; safe to re-run.
# Prefers `uv tool` (isolated), falls back to `pip --user`. Node tools run via `npx`.
# No `curl | bash`: everything comes from pinned package indexes.
source "${BASH_SOURCE%/*}/lib.sh"
cd "$REPO_ROOT"
section "install check tools"

PY_TOOLS=(check-jsonschema codespell shellcheck-py)

if have uv; then
  for t in "${PY_TOOLS[@]}"; do uv tool install --quiet "$t" 2>/dev/null && ok "uv tool: $t" || skip "uv tool: $t (already present or failed)"; done
elif have python3; then
  python3 -m pip install --user --quiet "${PY_TOOLS[@]}" && ok "pip --user: ${PY_TOOLS[*]}" || skip "pip install failed"
else
  skip "no uv/python3 — skipped python tools"
fi

# Node tool: markdownlint-cli2 is invoked on demand via `npx --yes`; warm the cache.
if have npx; then
  npx --yes markdownlint-cli2 --version >/dev/null 2>&1 && ok "npx markdownlint-cli2 ready" || skip "npx markdownlint-cli2 warmup failed"
else
  skip "no node/npx — markdown lint will skip"
fi

# gitleaks is optional (no pip package). If absent, secrets.sh uses a narrow fallback.
have gitleaks && ok "gitleaks present" || skip "gitleaks not installed (optional; secrets.sh has a fallback)"

echo
ok "setup done — run \`just check\`"
