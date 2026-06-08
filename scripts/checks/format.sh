#!/usr/bin/env bash
# Formatting. Default = check only; pass --fix to write. Skips languages not present.
source "${BASH_SOURCE%/*}/../lib.sh"
cd "$REPO_ROOT" || exit 1

fix=0; mode="check"
if [[ "${1:-}" == "--fix" ]]; then fix=1; mode="write"; fi
section "format ($mode)"
rc=0

# Rust
if [[ -f Cargo.toml ]] && have cargo; then
  if [[ $fix -eq 1 ]]; then
    if cargo fmt; then ok "cargo fmt (wrote)"; else fail "cargo fmt failed"; rc=1; fi
  elif cargo fmt --check; then ok "cargo fmt --check"
  else fail "rust not formatted (\`just fmt\`)"; rc=1; fi
else
  skip "rust: no Cargo.toml or cargo"
fi

# Python
tracked '*.py'
if [[ ${#TRACKED[@]} -gt 0 ]] && have ruff; then
  if [[ $fix -eq 1 ]]; then
    if ruff format "${TRACKED[@]}"; then ok "ruff format (wrote)"; else fail "ruff format failed"; rc=1; fi
  elif ruff format --check "${TRACKED[@]}"; then ok "ruff format --check"
  else fail "python not formatted (\`just fmt\`)"; rc=1; fi
else
  skip "python: no *.py or ruff"
fi

exit $rc
