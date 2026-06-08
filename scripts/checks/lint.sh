#!/usr/bin/env bash
# Lint code. Skips languages not present yet.
source "${BASH_SOURCE%/*}/../lib.sh"
cd "$REPO_ROOT"
section "lint"

rc=0

# Rust — clippy, warnings are errors (CONTRIBUTING: `clippy -D warnings`).
if [[ -f Cargo.toml ]] && have cargo; then
  cargo clippy --all-targets --all-features -- -D warnings && ok "cargo clippy -D warnings" \
    || { fail "clippy findings"; rc=1; }
else
  skip "rust: no Cargo.toml or cargo"
fi

# Python — ruff check.
tracked '*.py'
if [[ ${#TRACKED[@]} -gt 0 ]] && have ruff; then
  ruff check "${TRACKED[@]}" && ok "ruff check" || { fail "ruff findings"; rc=1; }
else
  skip "python: no *.py or ruff"
fi

exit $rc
