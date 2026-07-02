#!/usr/bin/env bash
# verify.sh — run the full `just check` gate and FAITHFULLY propagate its exit code.
#
# Why this exists: `just check` (via scripts/checks/all.sh) already exits with the
# correct code, but automation/background runners that pipe it through a trailing
# command — `just check >log; tail log`, `... | tee`, `... && echo done` — report
# the LAST command's status, silently occluding a real gate failure (a green-looking
# run that actually failed). This wrapper's OWN exit status IS `just check`'s: it runs
# the gate, captures the code, prints one grep-able CHECK_RESULT line, and `exit`s with
# that code and nothing after it. Safe to run in the background or capture in CI without
# masking the result.
#
# Usage: scripts/checks/verify.sh [logfile]   (default: target/verify-check.log)
set -uo pipefail

log="${1:-target/verify-check.log}"
mkdir -p "$(dirname "$log")"

just check >"$log" 2>&1
rc=$?

if [ "$rc" -eq 0 ]; then
    echo "CHECK_RESULT=PASS exit=0 (log: $log)"
else
    echo "CHECK_RESULT=FAIL exit=$rc (log: $log)"
    echo "--- last 20 lines ---" >&2
    tail -n 20 "$log" >&2
fi

exit "$rc"
