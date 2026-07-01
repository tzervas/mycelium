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
#   - cargo-machete: unused-dependency surface (supply-chain-surface reduction; advisory — DN-44 §6.2).
#   - kernel-harden: clippy panic-path visibility over the trusted base (advisory — DN-44 §4.2/§6.1).
# Full posture map: docs/notes/DN-44-Codebase-Security-Posture.md.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "advisory scanners (osv / geiger / hack / machete / kernel-harden) — opt-in, not in \`just check\`"

rc=0
# osv-scanner (a Go binary) lands in the Go bin dir; cargo-machete in the cargo bin dir.
gopath="$(go env GOPATH 2>/dev/null)"
export PATH="$PATH:${gopath}/bin:${HOME}/go/bin:${HOME}/.cargo/bin"

# --- osv-scanner: supply-chain via OSV.dev (tuned by osv-scanner.toml — never-silent ignores) ---
cfg=(); [[ -f osv-scanner.toml ]] && cfg=(--config=osv-scanner.toml)
if command -v osv-scanner >/dev/null 2>&1; then
  if osv-scanner "${cfg[@]}" --lockfile=Cargo.lock >/tmp/myc-osv.out 2>&1 \
     || osv-scanner scan source "${cfg[@]}" --lockfile=Cargo.lock >/tmp/myc-osv.out 2>&1; then
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

# --- cargo-machete: unused-dependency surface (advisory; reports candidates, does NOT fail — machete is
# false-positive-prone, e.g. derive-only `serde`, so candidates are verified per-crate, not auto-stripped) ---
if command -v cargo-machete >/dev/null 2>&1; then
  if cargo machete >/tmp/myc-machete.out 2>&1; then
    ok "cargo-machete: no unused dependencies"
  else
    n=$(grep -cE 'Cargo\.toml:' /tmp/myc-machete.out || true)
    skip "cargo-machete: ${n} crate(s) with unused-dep CANDIDATES (advisory — verify per-crate; false positives possible; tracked DN-44 §6.2):"
    sed -n '1,/If you believe/p' /tmp/myc-machete.out | grep -vE 'If you believe|^$' | head -14 | sed 's/^/    /'
  fi
else
  skip "cargo-machete not installed — \`cargo install cargo-machete\` (or \`just setup-scan\`)"
fi

# --- kernel-hardening clippy: panic-path visibility over the TRUSTED BASE (advisory; counts, never fails) ---
# unwrap/expect/indexing in trusted-base logic are panic paths to review (DN-44 §4.2). Advisory ONLY — NOT a
# blocking deny: kernel arithmetic panics on overflow BY DESIGN (overflow-checks=true), and `-D warnings`
# would break the build. The DN-44 §6.1 ratchet promotes this to blocking once the base is clean.
if command -v cargo >/dev/null 2>&1; then
  cargo clippy --lib -p mycelium-core -p mycelium-cert -p mycelium-numerics -p mycelium-vsa --no-deps -- \
    -W clippy::unwrap_used -W clippy::expect_used -W clippy::indexing_slicing -W clippy::panic_in_result_fn \
    >/tmp/myc-kernel-harden.out 2>&1 || true
  n=$(sed -r 's/\x1b\[[0-9;]*[mGKH]//g' /tmp/myc-kernel-harden.out | grep -cE '^warning:' || true)
  ok "kernel-hardening clippy (trusted base): ${n} panic-path site(s) for review (advisory — DN-44 §6.1 ratchet)"
else
  skip "cargo not installed — kernel-hardening clippy pass skipped"
fi

exit "$rc"
