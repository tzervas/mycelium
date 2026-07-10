#!/usr/bin/env bash
# tero-index — drift gate: the committed docs/tero-index/ must match a fresh regeneration from the
# corpus (M-1015 / DN-87 §6.3; the whole-corpus generalization of scripts/checks/lib-index.sh's
# docs/lib-index/ gate, itself the docs/api-index/ analogue). Mirrors lib-index.sh's
# regenerate-to-temp-and-diff shape.
#
# Trim pass (2026-07-10): `crates/mycelium-tero` was extracted verbatim into its own published repo
# (`tzervas/tero-rs`, renamed `tero`) — mycelium no longer recompiles it in-tree. This gate now runs
# a checksum-verified, cached PUBLISHED `tero-index` binary via scripts/fetch-tero-index.sh instead
# of `cargo run -p mycelium-tero`. See tools/tero-rs/PROVENANCE.md for the three-way differential
# (in-tree crate / published binary / committed index, all byte-identical) this switch was adopted
# under, and the honest FLAG on the new `gh`-auth prerequisite for a cold cache.
#
# Skip-graceful (never a false-red for an environment-shaped gap, matching every other optional-tool
# gate — api.sh/lib-index.sh/myc-doc.sh): no cached-or-fetchable binary ⇒ skip, not fail. This is a
# *coverage* skip like "no cargo" ever was — the committed docs/tero-index/ is still the ground
# truth consumers read; only its regeneration-verification step is what's skipped.
#
# No separate generator self-test here: the extractor's determinism / extraction-correctness /
# flagged-path white-box tests lived in `crates/mycelium-tero/src/tests/` (now `tero-rs`'s own
# `cargo test -p tero` — 113 passing per the v0.1.3 release notes) — that coverage now belongs to
# the tero-rs repo, not this one (same "future tool changes happen in the TOOL repo" boundary).
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "$SCRIPT_DIR/../lib.sh"
cd "$REPO_ROOT" || exit 1

section "tero-index"
# Specific reason sub-codes (consumed by all.sh's packed exit byte): 2 = committed index is stale
# (run `just tero-index-gen` + commit), 3 = the generator failed to run, 4 = fetched binary FAILED
# its pinned checksum (a tampering signal — a hard fail, never a skip). 0 = current.

# Resolve the published tero-index binary. Distinguish a benign not-fetchable gap (skip) from a
# checksum MISMATCH (fail) by the fetch script's exit code (1 -> skip, 4 -> fail; see its header).
fetch_rc=0
tero_index_bin="$(bash "$SCRIPT_DIR/../fetch-tero-index.sh" 2>&1)" || fetch_rc=$?
if [[ $fetch_rc -eq 4 ]]; then
  fail "tero-index binary CHECKSUM MISMATCH — refusing to use it (possible tampering) — ${tero_index_bin##*$'\n'}"
  exit 4
elif [[ $fetch_rc -ne 0 ]]; then
  skip "tero-index binary unavailable — ${tero_index_bin##*$'\n'}"
  exit 0
fi

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT

if ! "$tero_index_bin" --repo-root "$REPO_ROOT" --out "$tmpdir" >/dev/null; then
  fail "tero-index failed to run — see above"
  exit 3
fi

if diff -rq "$tmpdir" docs/tero-index/ >/dev/null 2>&1; then
  ok "docs/tero-index/ is current"
else
  diff -r "$tmpdir" docs/tero-index/ || true
  fail "docs/tero-index/ is stale — run 'just tero-index-gen' and commit the result"
  echo "  reproduce: bin=\$(bash scripts/fetch-tero-index.sh) && \"\$bin\" --repo-root . --out docs/tero-index"
  exit 2
fi
