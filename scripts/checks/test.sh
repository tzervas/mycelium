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

# Python — the uv-managed experiments project (M-092), under its pinned interpreter (3.13; ADR-007).
if [[ -f experiments/pyproject.toml ]] && have uv; then
  if ( cd experiments && uv run --frozen pytest ); then ok "uv run pytest (experiments)"; else fail "pytest failures"; rc=1; fi
else
  skip "python: no uv experiments project (or uv missing)"
fi

exit $rc
