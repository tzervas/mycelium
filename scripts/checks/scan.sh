#!/usr/bin/env bash
# Supplementary, ADVISORY local scanners (opt-in — NOT part of `just check`): extra supply-chain
# and code-quality coverage beyond clippy / cargo-deny, runnable fully in-env with NO CI runners.
# Each tool SKIPS GRACEFULLY when absent (install with `just setup-scan`); a real finding fails
# non-zero so this can be wired into a stricter gate later (never silently green — G2). Run: `just scan`.
#
#   - osv-scanner  : supply-chain vulns via OSV.dev — a WORKING alternative to cargo-audit, whose
#                    RustSec git-fetch 403s in some sandboxes (OSV.dev is reachable over plain HTTPS).
#   - cargo-geiger : `unsafe`-usage audit across the dep tree (ADR-014 unsafe-code policy).
#   - cargo-hack   : feature-powerset compile (catches broken feature-flag combos — e.g.
#                    mycelium-mlir's `mlir-dialect` / `bitnet-accel`).
source "${BASH_SOURCE%/*}/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "advisory scanners (osv-scanner / cargo-geiger / cargo-hack) — opt-in, not in \`just check\`"

rc=0
# osv-scanner (a Go binary) usually lands in the Go bin dir.
export PATH="$PATH:$(go env GOPATH 2>/dev/null)/bin:${HOME}/go/bin"

# --- osv-scanner: supply-chain via OSV.dev ---
if command -v osv-scanner >/dev/null 2>&1; then
  if osv-scanner --lockfile=Cargo.lock >/tmp/myc-osv.out 2>&1 \
     || osv-scanner scan source --lockfile=Cargo.lock >/tmp/myc-osv.out 2>&1; then
    ok "osv-scanner: no known vulnerabilities (OSV.dev) — $(grep -oE '[0-9]+ packages' /tmp/myc-osv.out | head -1)"
  else
    fail "osv-scanner: vulnerability findings — review and bump"; tail -20 /tmp/myc-osv.out | sed 's/^/    /'; rc=1
  fi
else
  skip "osv-scanner not installed — \`go install github.com/google/osv-scanner/v2/cmd/osv-scanner@latest\` (or \`just setup-scan\`)"
fi

# --- cargo-geiger: unsafe-usage audit (advisory; reports, does not fail) ---
if command -v cargo-geiger >/dev/null 2>&1; then
  cargo geiger --quiet 2>/dev/null | tail -4 | sed 's/^/    /'
  ok "cargo-geiger: unsafe-usage report above (advisory; ADR-014)"
else
  skip "cargo-geiger not installed — \`cargo install cargo-geiger\` (or \`just setup-scan\`)"
fi

# --- cargo-hack: feature-powerset compile, scoped to the feature-bearing crate ---
if command -v cargo-hack >/dev/null 2>&1; then
  if cargo hack --feature-powerset --no-dev-deps check -p mycelium-mlir >/tmp/myc-hack.out 2>&1; then
    ok "cargo-hack: every mycelium-mlir feature combination compiles (mlir-dialect / bitnet-accel)"
  else
    fail "cargo-hack: a feature combination fails to compile"; tail -15 /tmp/myc-hack.out | sed 's/^/    /'; rc=1
  fi
else
  skip "cargo-hack not installed — \`cargo install cargo-hack\` (or \`just setup-scan\`)"
fi

exit "$rc"
