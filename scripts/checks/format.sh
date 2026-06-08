#!/usr/bin/env bash
# Formatting. Default = check only; pass --fix to write. Skips languages not present.
source "${BASH_SOURCE%/*}/../lib.sh"
cd "$REPO_ROOT"
section "format ($([[ "${1:-}" == "--fix" ]] && echo write || echo check))"

fix=0; [[ "${1:-}" == "--fix" ]] && fix=1
rc=0

# Rust
if [[ -f Cargo.toml ]] && have cargo; then
  if [[ $fix -eq 1 ]]; then cargo fmt && ok "cargo fmt (wrote)"
  else cargo fmt --check && ok "cargo fmt --check" || { fail "rust not formatted (\`just fmt\`)"; rc=1; }; fi
else
  skip "rust: no Cargo.toml or cargo"
fi

# Python
tracked '*.py'
if [[ ${#TRACKED[@]} -gt 0 ]] && have ruff; then
  if [[ $fix -eq 1 ]]; then ruff format "${TRACKED[@]}" && ok "ruff format (wrote)"
  else ruff format --check "${TRACKED[@]}" && ok "ruff format --check" || { fail "python not formatted (\`just fmt\`)"; rc=1; }; fi
else
  skip "python: no *.py or ruff"
fi

exit $rc
