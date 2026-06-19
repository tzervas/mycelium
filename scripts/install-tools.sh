#!/usr/bin/env bash
# Best-effort install of the local check tools. Idempotent; safe to re-run.
# Prefers `uv tool` (isolated), falls back to `pip --user`. Node tools run via `npx`.
# No `curl | bash`: everything comes from pinned package indexes.
#
# ── Claude Code on the web (cloud sessions) ──────────────────────────────────────────────────
# This is the canonical "install the toolchain" script. Wire it as the environment **Setup
# script** (Cloud environment settings → "Setup script" field):
#
#     bash scripts/install-tools.sh
#
# A setup script runs the FIRST time a session starts in an environment; Anthropic then snapshots
# the filesystem and reuses that snapshot for later sessions, so the compiled binaries (cargo
# tools land in ~/.cargo/bin) persist and the setup step is SKIPPED on subsequent sessions — the
# toolchain is compiled once, not per session. Snapshot rebuilds only when the setup script or the
# network allowlist changes, or after ~7 days. (Docs: code.claude.com/docs/en/claude-code-on-the-web
# § "Setup scripts" / "Environment caching".) Do NOT put this in a SessionStart hook — those run on
# every session and are not cached, which would recompile the toolchain every time.
#
# Setup scripts have a ~5-minute budget for the cache to build. The cargo introspection tools
# (`just map` / `just api`: cargo-modules/depgraph/public-api) are the slow tail and are NOT part
# of `just check`; set MYCELIUM_SKIP_OPTIONAL_CARGO=1 to skip them and stay well under budget. The
# security gates (cargo-deny/cargo-audit, used by `just deny`) and everything else still install.
# ─────────────────────────────────────────────────────────────────────────────────────────────
source "${BASH_SOURCE%/*}/lib.sh"
cd "$REPO_ROOT" || exit 1

# Ensure uv's tool-bin and cargo's bin are on PATH for this run (a fresh container may not have
# re-sourced the profile yet). The snapshot preserves these dirs, and the base image already has
# them on PATH for later sessions, so no per-session PATH wiring is needed.
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"

section "bootstrap gating tools (just / pre-commit / yamllint)"
# These are the tools the local↔CI check spine assumes exist (`just check` routes through them).
# install-tools.sh can't rely on `just` to install `just` (chicken-and-egg), so bootstrap them
# directly here via prebuilt wheels (`rust-just` ships a binary — no cargo compile). Idempotent.
declare -A BOOTSTRAP=( [just]=rust-just [pre-commit]=pre-commit [yamllint]=yamllint )
if have uv; then
  for bin in "${!BOOTSTRAP[@]}"; do
    if have "$bin"; then ok "present: $bin"
    elif uv tool install --quiet "${BOOTSTRAP[$bin]}" 2>/dev/null; then ok "uv tool: $bin (${BOOTSTRAP[$bin]})"
    else skip "uv tool: $bin (install failed)"; fi
  done
elif have python3; then
  if python3 -m pip install --user --quiet rust-just pre-commit yamllint 2>/dev/null; then
    ok "pip --user: just/pre-commit/yamllint"
  else
    skip "pip bootstrap failed"
  fi
else
  skip "no uv/python3 — cannot bootstrap just/pre-commit/yamllint"
fi

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
# These are the slow compile tail and are NOT part of `just check`; MYCELIUM_SKIP_OPTIONAL_CARGO=1
# skips them to keep a cloud Setup-script run inside the ~5-min cache-build budget.
if [[ "${MYCELIUM_SKIP_OPTIONAL_CARGO:-0}" == "1" ]]; then
  skip "code-map/api tools skipped (MYCELIUM_SKIP_OPTIONAL_CARGO=1) — \`just map\`/\`just api\` will skip"
elif have cargo; then
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
