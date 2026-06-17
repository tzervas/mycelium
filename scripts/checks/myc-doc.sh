#!/usr/bin/env bash
# Doc-build / §4.1 quality-bar gate (M-363; `myc-doc lint`). Projects the corpus + schemas + example
# `.myc` nodules into the content-addressed doc-IR and runs the eight §4.1 checks over it (single-
# template-conformance · navigability · progressive-disclosure · checked-examples · no-dead-xref ·
# dual-projection-parity · no-hallucinated-prose · legibility-accessibility). An error-severity
# finding fails the gate (contract: 0 clean · 1 error-severity finding · 64 usage · 66 io). Green-and-
# real, mirroring Wave A: checked inline examples actually type-check via the trusted L1 checker, and
# the legibility check is honestly PARTIALLY-DORMANT (structure checked; colour-contrast/typography
# need a rendering engine). Skips gracefully when cargo is absent. `typst` PDF compile is a separate,
# optional downstream step that also skips when typst is missing — never a half-build.
source "${BASH_SOURCE%/*}/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "myc-doc (M-363 doc-IR build + §4.1 quality-bar lint — myc-doc)"

if ! have cargo; then skip "no cargo — myc-doc gate skipped"; exit 0; fi

if cargo run -q -p mycelium-doc --bin myc-doc -- lint --repo-root "$REPO_ROOT"; then
  ok "§4.1 doc quality-bar lint: no error-severity findings (8 checks over the doc-IR)"
  if have typst; then
    skip "typst present — PDF compile is an optional downstream step (not gated here)"
  else
    skip "typst absent — PDF compile skipped gracefully (the .typ source still emits)"
  fi
  exit 0
else
  fail "§4.1 doc quality-bar lint: error-severity finding(s)"
  exit 1
fi
