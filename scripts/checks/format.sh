#!/usr/bin/env bash
# Formatting. Default = check only; pass --fix to write. Skips languages not present.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib.sh"
cd "$REPO_ROOT" || exit 1

fix=0; mode="check"
if [[ "${1:-}" == "--fix" ]]; then fix=1; mode="write"; fi
section "format ($mode)"
# Specific reason sub-codes (consumed by all.sh's packed exit byte): 2 = rust not formatted,
# 3 = python not formatted, 4 = a formatter tool errored. 0 = clean. (When several fail, rc carries
# the last; the digest tail still lists every one.)
rc=0

# Rust
if [[ -f Cargo.toml ]] && have cargo; then
  if [[ $fix -eq 1 ]]; then
    if cargo fmt; then ok "cargo fmt (wrote)"; else fail "cargo fmt failed"; rc=4; fi
  elif cargo fmt --check; then ok "cargo fmt --check"
  else fail "rust not formatted (\`just fmt\`)"; rc=2; fi
else
  skip "rust: no Cargo.toml or cargo"
fi

# Python
tracked '*.py'
if [[ ${#TRACKED[@]} -gt 0 ]] && have ruff; then
  if [[ $fix -eq 1 ]]; then
    if ruff format "${TRACKED[@]}"; then ok "ruff format (wrote)"; else fail "ruff format failed"; rc=4; fi
  elif ruff format --check "${TRACKED[@]}"; then ok "ruff format --check"
  else fail "python not formatted (\`just fmt\`)"; rc=3; fi
else
  skip "python: no *.py or ruff"
fi

exit $rc
