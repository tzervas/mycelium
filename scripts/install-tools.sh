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

# Code-map / API-surface tools (optional). `cargo public-api` (the `just api` gate) drives a
# nightly rustdoc — used only to introspect the surface, not to build the MSRV-pinned artifact.
# Graphviz (`dot`) renders the `just map` graphs; install it via your system package manager.
if have cargo; then
  for t in cargo-modules cargo-depgraph cargo-public-api; do
    if cargo "${t#cargo-}" --help >/dev/null 2>&1; then ok "cargo: $t present"
    elif cargo install --quiet "$t" 2>/dev/null; then ok "cargo install: $t"
    else skip "cargo: $t (install failed or offline; \`just map\`/\`just api\` will skip it)"; fi
  done
else
  skip "no cargo — skipped code-map tools"
fi
if have dot; then ok "graphviz (dot) present"
else skip "graphviz (dot) absent — \`just map\` writes .dot sources"; fi

echo
ok "setup done — run \`just check\`"
