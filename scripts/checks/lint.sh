#!/usr/bin/env bash
# Lint code. Skips languages not present yet.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "lint"
rc=0

# Rust — clippy, warnings are errors (CONTRIBUTING: `clippy -D warnings`), EXCEPT `unsafe_code`,
# which ADR-014 makes permitted-but-warned: `-A unsafe_code` exempts only that lint so intentional,
# justified unsafe (e.g. FFI/JIT) passes the gate while every *other* warning stays a hard error.
# Note (2026-07-09): the `-A unsafe_code` exemption allows intentional unsafe, but the workspace lint
# `clippy::undocumented_unsafe_blocks` (Cargo.toml `[workspace.lints.clippy]`) still fires here (via
# `-D warnings`) on any unsafe block lacking a `// SAFETY:` comment — so undocumented/unintentional
# unsafe IS caught; only documented, intentional unsafe passes.
if [[ -f Cargo.toml ]] && have cargo; then
  if cargo clippy --all-targets --all-features -- -D warnings -A unsafe_code; then ok "cargo clippy -D warnings (unsafe_code warned, not gated — ADR-014)"
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
