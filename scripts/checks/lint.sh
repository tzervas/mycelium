#!/usr/bin/env bash
# Lint code. Skips languages not present yet.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "lint"
rc=0

# Rust — clippy, warnings are errors (CONTRIBUTING: `clippy -D warnings`). ADR-014 (refined
# 2026-07-09): `unsafe_code = "allow"` in `[workspace.lints.rust]` lets documented, intentional
# unsafe pass, and `clippy::undocumented_unsafe_blocks` (fired here via `-D warnings`) catches any
# unsafe block lacking a `// SAFETY:` comment. So undocumented/unintentional unsafe IS caught; only
# documented, intentional unsafe passes — the SAME for local and remote (`just ci` runs this suite),
# and even for ad-hoc `cargo clippy -D warnings` (no exemption flag needed). The `-A unsafe_code`
# below is now REDUNDANT (unsafe_code is already `allow`); kept as a defensive belt-and-suspenders so
# the gate stays green even if the workspace level is ever re-pinned to `warn`.
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
