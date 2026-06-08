#!/usr/bin/env bash
# Best-effort install of the local check tools. Idempotent; safe to re-run.
# Prefers `uv tool` (isolated), falls back to `pip --user`. Node tools run via `npx`.
# No `curl | bash`: everything comes from pinned package indexes.
source "${BASH_SOURCE%/*}/lib.sh"
cd "$REPO_ROOT" || exit 1
section "install check tools"

PY_TOOLS=(check-jsonschema codespell shellcheck-py)

if have uv; then
  for t in "${PY_TOOLS[@]}"; do
    if uv tool install --quiet "$t" 2>/dev/null; then ok "uv tool: $t"
    else skip "uv tool: $t (already present or failed)"; fi
  done
elif have python3; then
  if python3 -m pip install --user --quiet "${PY_TOOLS[@]}"; then ok "pip --user: ${PY_TOOLS[*]}"
  else skip "pip install failed"; fi
else
  skip "no uv/python3 — skipped python tools"
fi

# Node tool: markdownlint-cli2 is invoked on demand via `npx --yes`; warm the cache.
if have npx; then
  if npx --yes markdownlint-cli2 --version >/dev/null 2>&1; then ok "npx markdownlint-cli2 ready"
  else skip "npx markdownlint-cli2 warmup failed"; fi
else
  skip "no node/npx — markdown lint will skip"
fi

# gitleaks is optional (no pip package). If absent, secrets.sh uses a narrow fallback.
if have gitleaks; then ok "gitleaks present"
else skip "gitleaks not installed (optional; secrets.sh has a fallback)"; fi

echo
ok "setup done — run \`just check\`"
