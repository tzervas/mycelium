#!/usr/bin/env bash
# Secret scan. Prefer gitleaks (respects .gitleaks.toml); else a narrow high-confidence fallback.
source "${BASH_SOURCE%/*}/../lib.sh"
cd "$REPO_ROOT"
section "secret scan"

if have gitleaks; then
  if gitleaks detect --no-banner --redact --source . ${GITLEAKS_CONFIG:+-c "$GITLEAKS_CONFIG"} \
       $([[ -f .gitleaks.toml ]] && echo "-c .gitleaks.toml"); then
    ok "gitleaks: no leaks"
  else
    fail "gitleaks: potential secret(s) found — investigate and rotate if real"; exit 1
  fi
  exit 0
fi

# Fallback: only high-confidence patterns, to avoid noise.
patterns='-----BEGIN [A-Z ]*PRIVATE KEY-----|AKIA[0-9A-Z]{16}|ASIA[0-9A-Z]{16}|gh[pousr]_[A-Za-z0-9]{36}|xox[baprs]-[A-Za-z0-9-]{10,}'
if git grep -nIE "$patterns" -- . ':!*.lock' >/tmp/secret-hits 2>/dev/null && [[ -s /tmp/secret-hits ]]; then
  fail "high-confidence secret pattern(s) found:"; cat /tmp/secret-hits; rm -f /tmp/secret-hits; exit 1
fi
rm -f /tmp/secret-hits
skip "gitleaks not installed — ran minimal fallback only (install for full coverage: \`just setup\`)"
