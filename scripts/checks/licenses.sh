#!/usr/bin/env bash
# Third-party attribution drift gate: the committed `THIRD-PARTY-LICENSES.md` must match a fresh
# `cargo about generate` from the current `Cargo.lock` + `about.toml` + `about.hbs`. Companion to
# `deny.sh` (which gates WHICH licenses are ALLOWED in the dependency tree) and
# `license-first-party.sh` (which gates the FIRST-PARTY MIT-only axis) — this gate is the
# notice-preservation axis: MIT/BSD/ISC/Apache-2.0 all require the license text to travel with a
# shipped artifact, and a stale `THIRD-PARTY-LICENSES.md` (e.g. after a dependency bump) is a
# silent attribution gap (G2 — never-silent). Skip-graceful if `cargo-about` is absent (same
# pattern as `deny.sh`/`doc-index.sh`) — install with `cargo install cargo-about --locked --features cli`.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib.sh"
cd "$REPO_ROOT" || exit 1

section "third-party license attributions (cargo-about)"

# `command -v cargo-about` is unreliable here: `cargo install` places binaries in
# `$CARGO_HOME/bin`, which cargo itself always searches when dispatching `cargo <subcommand>` but
# which is NOT necessarily on `$PATH` (mitigation #3 — tool discovery/PATH gotchas). Probe via
# cargo's own subcommand dispatch instead of a raw PATH lookup.
if ! cargo about --version >/dev/null 2>&1; then
  skip "cargo-about not installed — run: cargo install cargo-about --locked --features cli (or \`just setup\`)"
  exit 0
fi

if [[ ! -f THIRD-PARTY-LICENSES.md ]]; then
  fail "THIRD-PARTY-LICENSES.md is missing — run \`just licenses\` and commit the result"
  exit 2
fi

tmpfile=$(mktemp)
trap 'rm -f "$tmpfile"' EXIT

if ! cargo about generate --workspace --fail about.hbs -o "$tmpfile" >/tmp/myc-licenses.out 2>&1; then
  fail "cargo about generate failed (a dependency's license could not be resolved) — see below"
  tail -30 /tmp/myc-licenses.out | sed 's/^/    /'
  exit 3
fi

if diff -q "$tmpfile" THIRD-PARTY-LICENSES.md >/dev/null 2>&1; then
  ok "THIRD-PARTY-LICENSES.md is current"
else
  diff "$tmpfile" THIRD-PARTY-LICENSES.md || true
  fail "THIRD-PARTY-LICENSES.md is stale — run \`just licenses\` and commit the result"
  exit 2
fi
