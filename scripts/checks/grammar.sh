#!/usr/bin/env bash
# Structural validation of the surface-grammar artifacts + conformance corpus
# (RFC-0006 §4.3; DN-02). Pure shell — no toolchain needed, so it rarely skips. The *parser* gate
# (accept/ programs parse, reject/ programs fail) lives in `crates/mycelium-l1` `tests/conformance.rs`
# and runs under `cargo test`; this check guards the corpus's shape and house conventions:
#   - the normative EBNF exists,
#   - both corpus categories exist and are non-empty,
#   - every .myc carries a leading `//` explanatory header (accept: what it exercises;
#     reject: why it is rejected — the never-silent expectation, surfaced in the corpus itself).
source "${BASH_SOURCE%/*}/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "grammar"

dir="docs/spec/grammar"
if [[ ! -d "$dir" ]]; then skip "no $dir yet (added with RFC-0006 grammar work)"; exit 0; fi

rc=0

if [[ ! -f "$dir/mycelium.ebnf" ]]; then
  fail "missing normative grammar $dir/mycelium.ebnf"; rc=1
fi

shopt -s nullglob
for cat in accept reject; do
  cdir="$dir/conformance/$cat"
  files=("$cdir"/*.myc)
  if [[ ${#files[@]} -eq 0 ]]; then
    fail "no .myc fixtures in $cdir (the corpus is the ground truth)"; rc=1; continue
  fi
  for f in "${files[@]}"; do
    first="$(head -n1 "$f")"
    if [[ "$first" != "//"* ]]; then
      fail "$f: missing leading '//' header (corpus files must state what they exercise / why rejected)"
      rc=1
    fi
  done
done

if [[ $rc -eq 0 ]]; then
  acc=$(find "$dir/conformance/accept" -name '*.myc' | wc -l | tr -d ' ')
  rej=$(find "$dir/conformance/reject" -name '*.myc' | wc -l | tr -d ' ')
  ok "grammar artifact + corpus well-formed ($acc accept, $rej reject; parser gate in cargo test)"
else
  fail "grammar artifacts/corpus malformed"
fi
exit $rc
