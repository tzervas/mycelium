#!/usr/bin/env bash
# Supply-chain gate (C1-09): cargo-deny (advisories + licenses + sources, config: deny.toml)
# and cargo-audit (RustSec advisory DB). Both are optional and skip gracefully when absent —
# install them with `just setup`. A real finding fails non-zero; a missing tool is a skip.
source "${BASH_SOURCE%/*}/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "supply-chain (cargo-deny / cargo-audit)"

if ! have cargo; then skip "no cargo — supply-chain gate skipped"; exit 0; fi

rc=0

# cargo-deny: advisories, licenses, sources, bans — driven by deny.toml at the repo root.
if cargo deny --version >/dev/null 2>&1; then
  if [[ -f deny.toml ]]; then
    if cargo deny check; then ok "cargo deny: clean"
    else fail "cargo deny: findings"; rc=1; fi
  else
    skip "cargo deny present but no deny.toml — skipped"
  fi
else
  skip "cargo-deny not installed — run \`just setup\`"
fi

# cargo-audit: RustSec advisories against Cargo.lock.
if cargo audit --version >/dev/null 2>&1; then
  if cargo audit; then ok "cargo audit: no known advisories"
  else fail "cargo audit: advisory findings — review and bump"; rc=1; fi
else
  skip "cargo-audit not installed — run \`just setup\`"
fi

exit "$rc"
