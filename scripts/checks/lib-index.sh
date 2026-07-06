#!/usr/bin/env bash
# lib-index — drift gate: committed docs/lib-index/ must match a fresh regeneration from lib/*.myc
# (M-1004/M-1005; the docs/api-index/ analogue for the self-hosted `.myc` tree). Mirrors
# scripts/checks/doc-index.sh's regenerate-to-temp-and-diff shape, adapted from the Python
# generator to the Rust `myc-doc lib-index` subcommand (crates/mycelium-doc/src/lib_index.rs).
# Skip-graceful if cargo (or Cargo.toml) is absent — same pattern as api.sh/myc-doc.sh.
#
# No separate "generator self-test" step here (unlike doc-index.sh's `--self-test` re-run): the
# extractor's determinism/extraction-correctness white-box tests
# (crates/mycelium-doc/src/tests/lib_index.rs) are real `cargo test` unit tests already covered by
# the crate's normal test run in the `test` gate — re-running them here would just duplicate that
# cost, not add coverage (code_index.py's `--self-test` earns its keep because it's a standalone
# script with no other test runner covering it; `lib_index.rs` already has one).
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "$SCRIPT_DIR/../lib.sh"
cd "$REPO_ROOT" || exit 1

section "lib-index"
# Specific reason sub-codes (consumed by all.sh's packed exit byte): 2 = committed index is stale
# (run `just lib-index` + commit), 3 = the generator failed to run. 0 = current.

if ! { [[ -f Cargo.toml ]] && have cargo; }; then
  skip "no Cargo.toml or cargo — lib-index gate skipped"
  exit 0
fi

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT

if ! cargo run -q -p mycelium-doc --bin myc-doc -- lib-index --repo-root "$REPO_ROOT" --out "$tmpdir" >/dev/null; then
  fail "myc-doc lib-index failed to run — see above"
  exit 3
fi

if diff -rq "$tmpdir" docs/lib-index/ >/dev/null 2>&1; then
  ok "docs/lib-index/ is current"
else
  diff -r "$tmpdir" docs/lib-index/ || true
  fail "docs/lib-index/ is stale — run 'just lib-index' (or the cargo command below) and commit the result"
  echo "  reproduce: cargo run -q -p mycelium-doc --bin myc-doc -- lib-index --repo-root . --out docs/lib-index"
  exit 2
fi
