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
# Pinned to a specific version (B1-02) — keep this in sync with scripts/checks/markdown.sh.
if have npx; then
  if npx --yes markdownlint-cli2@0.22.1 --version >/dev/null 2>&1; then ok "npx markdownlint-cli2 ready"
  else skip "npx markdownlint-cli2 warmup failed"; fi
else
  skip "no node/npx — markdown lint will skip"
fi

# gitleaks (C1-09): best-effort install so secrets.sh runs the full scan, not just the narrow
# fallback. No pip package; try cargo (gitleaks has a Rust port? no — use the Go binary if go is
# present) else leave it to the system package manager. Skip-if-missing in all cases.
if have gitleaks; then
  ok "gitleaks present"
elif have go; then
  if go install github.com/gitleaks/gitleaks/v8@latest >/dev/null 2>&1; then ok "go install: gitleaks"
  else skip "gitleaks install via go failed (secrets.sh falls back)"; fi
else
  skip "gitleaks not installed (no go; install via your package manager — secrets.sh has a fallback)"
fi

# cargo-deny / cargo-audit (C1-09): supply-chain gates driven by scripts/checks/deny.sh.
# Best-effort, skip-if-missing. `deny.sh` skips gracefully when either is absent.
if have cargo; then
  for t in cargo-deny cargo-audit; do
    if cargo "${t#cargo-}" --version >/dev/null 2>&1; then ok "cargo: $t present"
    elif cargo install --quiet "$t" 2>/dev/null; then ok "cargo install: $t"
    else skip "cargo: $t (install failed or offline; \`just deny\` will skip it)"; fi
  done
fi

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
