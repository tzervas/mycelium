#!/usr/bin/env bash
# Supply-chain gate (C1-09): cargo-deny (advisories + licenses + sources, config: deny.toml)
# and cargo-audit (RustSec advisory DB). For LOCAL dev both skip gracefully when absent — install
# them with `just setup`. A real finding always fails non-zero. In the GATE environment (CI sets
# CI=true, or set MYCELIUM_REQUIRE_SUPPLY_CHAIN=1) a MISSING tool is a FAILURE, not a skip — a
# skip-pass is not a closed gate (G2, never silently green). ADR-021 Gate A4 / M-652.
source "${BASH_SOURCE%/*}/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "supply-chain (cargo-deny / cargo-audit)"

rc=0
# Strict in the gate environment: a missing tool FAILS (the gate must actually run), not skips.
# CI sets CI=true; MYCELIUM_REQUIRE_SUPPLY_CHAIN=1 forces it anywhere. Local dev keeps graceful skip.
strict=0; [[ -n "${CI:-}" || -n "${MYCELIUM_REQUIRE_SUPPLY_CHAIN:-}" ]] && strict=1
absent() { # $1=tool: FAIL under strict (no silent skip-pass — G2), skip otherwise.
  if ((strict)); then
    fail "$1 not installed but the supply-chain gate is REQUIRED here (CI / MYCELIUM_REQUIRE_SUPPLY_CHAIN=1) — run \`just setup\`"; rc=1
  else
    skip "$1 not installed — run \`just setup\`"
  fi
}

if ! have cargo; then absent cargo; exit "$rc"; fi

# cargo-deny: advisories, licenses, sources, bans — driven by deny.toml at the repo root.
if cargo deny --version >/dev/null 2>&1; then
  if [[ -f deny.toml ]]; then
    if cargo deny check; then ok "cargo deny: clean"
    else fail "cargo deny: findings"; rc=1; fi
  else
    skip "cargo deny present but no deny.toml — skipped"
  fi
else
  absent cargo-deny
fi

# cargo-audit: RustSec advisories against Cargo.lock.
if cargo audit --version >/dev/null 2>&1; then
  if cargo audit; then ok "cargo audit: no known advisories"
  else fail "cargo audit: advisory findings — review and bump"; rc=1; fi
else
  absent cargo-audit
fi

exit "$rc"
