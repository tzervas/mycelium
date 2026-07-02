#!/usr/bin/env bash
# Structural acyclic-deps gate (M-877/878/879; kickoff `acy`, ADR-038 §2; invariant: DN-68).
#
# Runs the `cargo xtask deps` check — a `cargo metadata` analysis (Tarjan SCC over normal + dev +
# build edges) enforcing the workspace's strictly-downward acyclic dependency structure:
#   * no dependency cycle (normal OR dev — cargo itself never rejects dev-dep cycles);
#   * no upward-tier edge (`core < std < tools`, per xtask/deps-strata.toml `[tiers]`);
#   * `mycelium-interp` never depends on any `mycelium-std-*` (the KC-3 trusted-base boundary).
# Diagnostics are per-edge and never-silent (G2); the check cites DN-68 for each rule.
#
# This is the AUTHORITATIVE structural gate. `deny.toml` carries a narrow belt-and-suspenders
# `[bans]` tripwire for the one already-fixed upward anomaly (mlir -> std-runtime); cargo-deny
# cannot express dependency cycles or the general tier rule, so `cargo xtask deps` is what enforces
# them (never-silent about the division of labour — see deny.toml's comment).
#
# Honesty (VR-5): the fine per-crate stratum map is `Empirical` (a frozen point-in-time topological
# derivation — a forward regression gate); the coarse `[tiers]` bucket is `Declared` (a manually
# asserted architectural intent). Source + `cargo metadata` are ground truth.
#
# Skip-graceful when cargo is absent (mirrors the other Rust gates — DN-20); in the GATE environment
# (CI=true, or MYCELIUM_REQUIRE_SUPPLY_CHAIN=1) a missing toolchain is a FAILURE, not a skip
# (a skip-pass is not a closed gate — G2).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib.sh"
cd "$REPO_ROOT" || exit 1

section "acyclic-deps structural gate (cargo xtask deps; M-877/878/879; DN-68)"

if ! have cargo; then
  if [[ "${CI:-}" == "true" || "${MYCELIUM_REQUIRE_SUPPLY_CHAIN:-}" == "1" ]]; then
    fail "cargo not found — the acyclic-deps gate cannot run in the gate environment (G2: a skip is not a closed gate)"
    exit 1
  fi
  skip "cargo not present — run \`just setup\`; the acyclic-deps gate is skipped locally (never-silent: NOT verified)"
  exit 0
fi

# Run the check; capture output + exit code (the tool exits non-zero on any violation, per its own
# never-silent contract). `--quiet` suppresses cargo's build chatter; the tool's own report remains.
out="$(cargo run --quiet -p xtask -- deps 2>&1)"
rc=$?
if (( rc == 0 )); then
  ok "no dependency-structure violations (acyclic, downward-only; interp free of std-*; DN-68)"
  exit 0
fi
fail "dependency-structure violation(s) detected (see DN-68 for the invariant + change-procedure):"
printf '%s\n' "$out" | sed 's/^/        /'
exit 1
