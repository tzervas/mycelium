#!/usr/bin/env bash
# Correctness/type-check gate (M-365; `myc-check --project`). Parses + L1-type-checks every `.myc`
# under each real project root, aggregating diagnostics via the M-362 baseline (contract: 0 ok ·
# 2 parse · 3 check · 5 project-resolution · 64 usage · 66 io). Honest depth: checking stops at
# name-visibility — cross-phylum depth is the M-365 deferral, not yet exercised here.
# Skips gracefully when cargo is absent or there is no project. Scope excludes tests/fixtures/
# (intentionally-bad must-fail inputs; locked decision #3).
source "${BASH_SOURCE%/*}/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "myc-check (parse + L1 type-check — myc-check)"

if ! have cargo; then skip "no cargo — myc-check gate skipped"; exit 0; fi

myc_roots
if [[ ${#MYC_ROOTS[@]} -eq 0 ]]; then
  skip "no project root (mycelium-proj.toml outside tests/fixtures/) — nothing to check"
  exit 0
fi

rc=0
for root in "${MYC_ROOTS[@]}"; do
  if cargo run -q -p mycelium-check --bin myc-check -- --project "$root"; then
    ok "$root: parses + type-checks (name-visibility depth; M-365 cross-phylum deferred)"
  else
    fail "$root: parse/check finding(s)"
    rc=1
  fi
done

exit $rc
