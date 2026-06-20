#!/usr/bin/env bash
# Doc-status currency gate (offline). Enforces the ratified decision-status lattice
# (Draft/Proposed/Preliminary -> Accepted -> Enacted -> Superseded; notes -> Resolved,
# #236) across every decision doc, cross-checks the index READMEs against the
# authoritative per-doc Status headers (the drift that left 8 stale RFC rows), and
# enforces the maintainer-DECLARED stale-phrase invariants in
# tools/doc-status-invariants.yaml. Complements doc-currency (structure/index/counts)
# and links (cross-refs); this one owns *status* currency. Source is ground truth.
source "${BASH_SOURCE%/*}/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "doc status (lattice · nav cross-check · declared stale-phrase invariants)"

if ! have python3; then skip "python3 not found — doc-status gate skipped"; exit 0; fi
if python3 "$REPO_ROOT/scripts/doc_status_check.py"; then
  ok "decision-doc statuses are current and on the ratified lattice"
else
  fail "decision-doc statuses are stale or off-lattice — see findings above"; exit 1
fi
