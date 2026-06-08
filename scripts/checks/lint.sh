#!/usr/bin/env bash
# Lint code. Skips languages not present yet.
source "${BASH_SOURCE%/*}/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "lint"
rc=0

# Rust — clippy, warnings are errors (CONTRIBUTING: `clippy -D warnings`).
if [[ -f Cargo.toml ]] && have cargo; then
  if cargo clippy --all-targets --all-features -- -D warnings; then ok "cargo clippy -D warnings"
  else fail "clippy findings"; rc=1; fi
else
  skip "rust: no Cargo.toml or cargo"
fi

# Python — ruff check.
tracked '*.py'
if [[ ${#TRACKED[@]} -gt 0 ]] && have ruff; then
  if ruff check "${TRACKED[@]}"; then ok "ruff check"; else fail "ruff findings"; rc=1; fi
else
  skip "python: no *.py or ruff"
fi

exit $rc
