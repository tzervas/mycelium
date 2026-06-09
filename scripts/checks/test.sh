#!/usr/bin/env bash
# Run unit/integration tests. Skips languages not present yet (most code doesn't exist).
source "${BASH_SOURCE%/*}/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "tests"
rc=0

# Rust — cargo test across the workspace (MSRV pinned via rust-toolchain.toml; ADR-007).
if [[ -f Cargo.toml ]] && have cargo; then
  if cargo test --workspace --all-features; then ok "cargo test"; else fail "cargo test failures"; rc=1; fi
else
  skip "rust: no Cargo.toml or cargo"
fi

# Python — pytest, once an experiments/ project exists (M-092).
if have pytest && { [[ -f experiments/pyproject.toml ]] || [[ -f pyproject.toml ]]; }; then
  if pytest -q; then ok "pytest"; else fail "pytest failures"; rc=1; fi
else
  skip "python: no pytest project yet"
fi

exit $rc
