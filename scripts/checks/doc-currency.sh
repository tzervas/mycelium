#!/usr/bin/env bash
# Documentation-currency gate (offline, M-371). Asserts the navigational docs stay in
# agreement with the repo they describe: the README "Repository structure" tree matches the
# real top-level layout, every RFC/ADR/DN is indexed in docs/Doc-Index.md, and any
# `<!-- doc-currency:crate-count -->` marker cites the real crate count. Complements (does NOT
# duplicate) the `links` cross-reference check and the `myc-doc` doc-IR quality bar.
source "${BASH_SOURCE%/*}/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "doc currency (structure tree · index coverage · cited counts)"

if ! have python3; then skip "python3 not found — doc-currency gate skipped"; exit 0; fi
if python3 "$REPO_ROOT/scripts/doc_currency.py"; then
  ok "navigational docs are current (structure · index · counts agree with the repo)"
else
  fail "navigational docs are stale — see findings above"; exit 1
fi
