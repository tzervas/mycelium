#!/usr/bin/env bash
# Secret scan. Prefer gitleaks (respects .gitleaks.toml); else a narrow high-confidence fallback.
source "${BASH_SOURCE%/*}/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "secret scan"

if have gitleaks; then
  args=(detect --no-banner --redact --source .)
  [[ -f .gitleaks.toml ]] && args+=(-c .gitleaks.toml)
  if gitleaks "${args[@]}"; then
    ok "gitleaks: no leaks"
  else
    fail "gitleaks: potential secret(s) found — investigate and rotate if real"; exit 1
  fi
  exit 0
fi

# Fallback: only high-confidence patterns, to avoid noise.
patterns='-----BEGIN [A-Z ]*PRIVATE KEY-----|AKIA[0-9A-Z]{16}|ASIA[0-9A-Z]{16}|gh[pousr]_[A-Za-z0-9]{36}|xox[baprs]-[A-Za-z0-9-]{10,}'
hits="$(git grep -nIE "$patterns" -- . ':!*.lock' || true)"
if [[ -n "$hits" ]]; then
  fail "high-confidence secret pattern(s) found:"
  printf '%s\n' "$hits"
  exit 1
fi
skip "gitleaks not installed — ran minimal fallback only (install for full coverage: \`just setup\`)"
