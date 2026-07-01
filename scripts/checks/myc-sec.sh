#!/usr/bin/env bash
# Security gate (M-367; `myc-sec`) — the `wild`-block audit over each real project root: every
# `wild` escape hatch must be justified, and an unjustified one is a finding (a high/critical fails;
# `--strict` also fails a medium). Scoped here to the wild-audit family with `--no-secrets
# --no-supply-chain`: secrets and supply-chain have their own dedicated gates in this suite
# (`secrets.sh`, `deny.sh`), so coverage of those families is preserved at the suite level — not
# dropped — while myc-sec reports FULL coverage of the family it owns (skip != pass; G2/VR-5).
# Skips gracefully when cargo is absent or there is no project. Scope excludes tests/fixtures/
# (intentionally-bad must-fail inputs; locked decision #3).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "myc-sec (wild-block audit — myc-sec)"

if ! have cargo; then skip "no cargo — myc-sec gate skipped"; exit 0; fi

myc_roots
if [[ ${#MYC_ROOTS[@]} -eq 0 ]]; then
  skip "no project root (mycelium-proj.toml outside tests/fixtures/) — nothing to audit"
  exit 0
fi

rc=0
for root in "${MYC_ROOTS[@]}"; do
  if cargo run -q -p mycelium-sec --bin myc-sec -- --project "$root" --no-secrets --no-supply-chain; then
    ok "$root: wild-audit clean (secrets + supply-chain covered by their own gates)"
  else
    fail "$root: wild-audit finding(s) — review the unjustified escape hatch"
    rc=1
  fi
done

exit $rc
