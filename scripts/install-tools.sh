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
# § "Setup scripts" / "Environment caching".)
#
# The Setup script stays the PRIMARY mechanism (it compiles the toolchain once, inside the cached
# snapshot). This script is **idempotent** — every install probes for the tool first (`have` / `cargo
# <sub> --version`) and skips it when present, so a re-run only fills gaps and never recompiles an
# already-installed toolchain. That makes a SessionStart-hook **safety-net** re-run cheap and safe on
# a normal (snapshotted) session — it just confirms "present" for everything. Caveat: on a *cold*
# container where the snapshot is unavailable, a SessionStart re-run would compile the missing cargo
# tools inside the hook (slow, uncached), so prefer the cached Setup script and treat the hook only as
# a belt-and-suspenders gap-filler.
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
# The binary each package provides (so a re-run can skip an already-installed tool by presence,
# only filling gaps — `shellcheck-py` ships `shellcheck`, not `shellcheck-py`).
declare -A PY_BIN=( [check-jsonschema]=check-jsonschema [codespell]=codespell [shellcheck-py]=shellcheck )

if have uv; then
  for t in "${PY_TOOLS[@]}"; do
    if have "${PY_BIN[$t]}"; then ok "present: $t (${PY_BIN[$t]})"
    elif uv tool install --quiet "$t" 2>/dev/null; then ok "uv tool: $t"
    else skip "uv tool: $t (install failed)"; fi
  done
elif have python3; then
  # Only install the packages whose binary is missing (idempotent gap-fill, not a blanket reinstall).
  missing=()
  for t in "${PY_TOOLS[@]}"; do have "${PY_BIN[$t]}" || missing+=("$t"); done
  if [[ ${#missing[@]} -eq 0 ]]; then ok "present: ${PY_TOOLS[*]}"
  elif python3 -m pip install --user --quiet "${missing[@]}"; then ok "pip --user: ${missing[*]}"
  else skip "pip install failed"; fi
else
  skip "no uv/python3 — skipped python tools"
fi

section "node runtime (for the npx-based gates)"
# Node/npm: the npx-driven gates (markdownlint; any structured-doc / json-schema check that shells to
# `npx`) need a Node runtime. The cloud base image ships Node at /opt/node22, but a bare or minimal
# container may lack it — provision it via the distro package manager if absent. Idempotent (probes
# `npm` first; installs ONLY when missing — a snapshot re-run is a no-op), never `curl | bash`,
# best-effort (an apt failure on a restricted network allowlist prints a skip and the npx gates skip,
# never blocking the rest of setup — G2).
if have npm; then
  ok "node/npm present (npm $(npm --version 2>/dev/null || echo '?'))"
elif have apt-get; then
  if DEBIAN_FRONTEND=noninteractive apt-get update -qq >/dev/null 2>&1 \
     && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends nodejs npm >/dev/null 2>&1; then
    ok "apt: installed node $(node --version 2>/dev/null || echo '?') + npm $(npm --version 2>/dev/null || echo '?')"
  else
    skip "apt: nodejs/npm install failed (offline/restricted allowlist) — the npx checks (markdown lint) will skip"
  fi
else
  skip "npm absent and no apt-get — install Node 20+ via your package manager so the npx-based checks run"
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
# Best-effort, skip-if-missing. `deny.sh` skips gracefully when either is absent. Idempotent: the
# `--version` probe short-circuits when the tool is already installed (a re-run never recompiles),
# and `--locked` pins the install to each crate's committed Cargo.lock (deterministic, no surprise
# dep rebuilds across snapshot rebuilds).
if have cargo; then
  for t in cargo-deny cargo-audit; do
    if cargo "${t#cargo-}" --version >/dev/null 2>&1; then ok "cargo: $t present"
    elif cargo install --locked --quiet "$t" 2>/dev/null; then ok "cargo install: $t"
    else skip "cargo: $t (install failed or offline; \`just deny\` will skip it)"; fi
  done
fi

# cargo-nextest (DN-20): the tiered test runner `scripts/checks/test.sh` prefers. Best-effort,
# idempotent (`cargo nextest --version` short-circuits when present), `--locked` for a deterministic
# install. When absent, test.sh falls back to `cargo test` so local↔CI parity holds either way — so
# this is a pure speed-up, never a gate. Skip-graceful (offline / install failure → plain `cargo test`).
if have cargo; then
  if cargo nextest --version >/dev/null 2>&1; then ok "cargo: cargo-nextest present"
  elif cargo install --locked --quiet cargo-nextest 2>/dev/null; then ok "cargo install: cargo-nextest"
  else skip "cargo: cargo-nextest (install failed or offline; tests fall back to \`cargo test\`)"; fi
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
    elif cargo install --locked --quiet "$t" 2>/dev/null; then ok "cargo install: $t"
    else skip "cargo: $t (install failed or offline; \`just map\`/\`just api\` will skip it)"; fi
  done
  # `cargo public-api` (the `just api` surface gate) builds rustdoc-JSON, which needs a **nightly**
  # rustdoc. Provision it here (idempotent: `rustup` is a no-op when nightly is already present) so
  # the gate runs in the snapshot rather than failing at runtime on a missing toolchain. Minimal
  # profile + the `rustdoc` component is all the surface build needs (not a full nightly std).
  if have rustup; then
    if rustup run nightly rustdoc --version >/dev/null 2>&1; then ok "rustup: nightly (rustdoc) present"
    elif rustup toolchain install nightly --profile minimal --component rustdoc >/dev/null 2>&1; then
      ok "rustup: nightly installed (rustdoc for \`cargo public-api\`)"
    else skip "rustup: nightly install failed (\`just api\` will fail to build the surface)"; fi
  else
    skip "no rustup — \`cargo public-api\` cannot build rustdoc-JSON (\`just api\` will fail)"
  fi
else
  skip "no cargo — skipped code-map tools"
fi
if have dot; then ok "graphviz (dot) present"
else skip "graphviz (dot) absent — \`just map\` writes .dot sources"; fi

echo
ok "setup done — run \`just check\`"
