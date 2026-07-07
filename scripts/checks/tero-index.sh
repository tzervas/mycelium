#!/usr/bin/env bash
# tero-index — drift gate: the committed docs/tero-index/ must match a fresh regeneration from the
# corpus (M-1015 / DN-87 §6.3; the whole-corpus generalization of scripts/checks/lib-index.sh's
# docs/lib-index/ gate, itself the docs/api-index/ analogue). Mirrors lib-index.sh's
# regenerate-to-temp-and-diff shape, driven by the `tero-index` bin
# (crates/mycelium-tero/src/bin/tero-index.rs). Skip-graceful if cargo (or Cargo.toml) is absent —
# same pattern as api.sh/lib-index.sh/myc-doc.sh.
#
# No separate generator self-test here: the extractor's determinism / extraction-correctness /
# flagged-path white-box tests (crates/mycelium-tero/src/tests/) are real `cargo test` unit tests
# already run by the `test` gate — re-running them here would duplicate cost, not add coverage
# (the same reasoning lib-index.sh records for omitting a `--self-test` re-run).
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "$SCRIPT_DIR/../lib.sh"
cd "$REPO_ROOT" || exit 1

section "tero-index"
# Specific reason sub-codes (consumed by all.sh's packed exit byte): 2 = committed index is stale
# (run `just tero-index-gen` + commit), 3 = the generator failed to run. 0 = current.

if ! { [[ -f Cargo.toml ]] && have cargo; }; then
  skip "no Cargo.toml or cargo — tero-index gate skipped"
  exit 0
fi

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT

if ! cargo run -q -p mycelium-tero --bin tero-index -- --repo-root "$REPO_ROOT" --out "$tmpdir" >/dev/null; then
  fail "tero-index failed to run — see above"
  exit 3
fi

if diff -rq "$tmpdir" docs/tero-index/ >/dev/null 2>&1; then
  ok "docs/tero-index/ is current"
else
  diff -r "$tmpdir" docs/tero-index/ || true
  fail "docs/tero-index/ is stale — run 'just tero-index-gen' (or the cargo command below) and commit the result"
  echo "  reproduce: cargo run -q -p mycelium-tero --bin tero-index -- --repo-root . --out docs/tero-index"
  exit 2
fi
