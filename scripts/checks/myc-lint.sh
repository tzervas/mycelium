#!/usr/bin/env bash
# Lint gate (M-366; `myc-lint --project`). Surfaces the M-141 invariant lints + the M-358/M-359
# header lints over each real project root; an error-severity finding fails the gate (contract:
# 0 clean/warnings · 1 error-severity · 64 usage · 66 io). Honest scope: `--fix` applies nothing
# in v0 (every fix is suggest/scaffold; header canonicalization is `mycfmt`'s job), and the §4.1
# doc-quality lint stays dormant until the M-363 doc build (Wave B) — so this gate is purely a
# read-only check here. Skips gracefully when cargo is absent or there is no project. Scope
# excludes tests/fixtures/ (intentionally-bad must-fail inputs; locked decision #3).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "myc-lint (invariant + header lints — myc-lint)"

if ! have cargo; then skip "no cargo — myc-lint gate skipped"; exit 0; fi

myc_roots
if [[ ${#MYC_ROOTS[@]} -eq 0 ]]; then
  skip "no project root (mycelium-proj.toml outside tests/fixtures/) — nothing to lint"
  exit 0
fi

rc=0
for root in "${MYC_ROOTS[@]}"; do
  if cargo run -q -p mycelium-lint --bin myc-lint -- --project "$root"; then
    ok "$root: no error-severity lint findings (--fix is a no-op in v0; doc-quality lint dormant)"
  else
    fail "$root: error-severity lint finding(s)"
    rc=1
  fi
done

exit $rc
