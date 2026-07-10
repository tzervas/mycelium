#!/usr/bin/env bash
# fetch-tero-index — resolve a checksum-verified `tero-index` binary from the published `tero-rs`
# release (tools/tero-rs/PROVENANCE.md), caching it locally so repeated gate runs don't re-fetch.
#
# `crates/mycelium-tero` was extracted verbatim into tzervas/tero-rs (renamed `tero`) and is now
# consumed as a published binary rather than recompiled in-tree (see the PROVENANCE.md three-way
# differential this pin was adopted under). Future tero changes happen in tero-rs via its own
# issues, not in mycelium (maintainer directive) — this script is the one place mycelium reaches
# out to fetch that consumed artifact.
#
# Contract: prints the resolved binary's ABSOLUTE PATH on stdout on success (nothing else — callers
# can `bin="$(scripts/fetch-tero-index.sh)"`); all diagnostics go to stderr. Exit 0 = resolved
# (either cache hit or fresh verified fetch), non-zero = unresolved (caller decides skip vs fail;
# see scripts/checks/tero-index.sh, which treats this as skip-graceful, mirroring the "skip if
# cargo absent" convention every other Rust gate uses).
#
# FLAG (honest, not silent): tero-rs is a PRIVATE repo, so a fresh fetch requires an authenticated
# `gh` CLI. This is a new prerequisite the prior self-contained `cargo run -p mycelium-tero` did not
# have. Once cached, no network/auth is needed again until the pin is bumped.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "$SCRIPT_DIR/lib.sh"
cd "$REPO_ROOT" || exit 1

PIN_DIR="$REPO_ROOT/tools/tero-rs"
PIN_VERSION="v0.1.3"
ASSET="tero-index-v0.1.3-linux-x86_64"
CACHE_DIR="${MYCELIUM_TERO_CACHE:-$HOME/.cache/mycelium/tero-rs}"
BIN_PATH="$CACHE_DIR/$ASSET"

sha256_of() { sha256sum "$1" 2>/dev/null | awk '{print $1}'; }
expected_sha256() { grep -F " $ASSET" "$PIN_DIR/SHA256SUMS.txt" 2>/dev/null | awk '{print $1}'; }

expected="$(expected_sha256)"
if [[ -z "$expected" ]]; then
  echo "fetch-tero-index: no pinned checksum for $ASSET in $PIN_DIR/SHA256SUMS.txt" >&2
  exit 1
fi

# Cache hit: verify, don't blindly trust a stale/tampered cache file.
if [[ -x "$BIN_PATH" ]] && [[ "$(sha256_of "$BIN_PATH")" == "$expected" ]]; then
  echo "$BIN_PATH"
  exit 0
fi

# Cache miss (or checksum mismatch) — fetch fresh via an authenticated `gh` (the repo is private).
if ! have gh; then
  echo "fetch-tero-index: no cached+verified binary at $BIN_PATH and 'gh' is unavailable to fetch $ASSET from tzervas/tero-rs (private repo)" >&2
  exit 1
fi
if ! gh auth status >/dev/null 2>&1; then
  echo "fetch-tero-index: no cached+verified binary and 'gh' is not authenticated — run 'gh auth login' (needs read access to the private tzervas/tero-rs repo)" >&2
  exit 1
fi

mkdir -p "$CACHE_DIR"
tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT
if ! gh release download "$PIN_VERSION" -R tzervas/tero-rs -p "$ASSET" -D "$tmpdir" >/dev/null 2>&1; then
  echo "fetch-tero-index: 'gh release download $PIN_VERSION -R tzervas/tero-rs -p $ASSET' failed (network, auth-scope, or a moved/deleted release asset)" >&2
  exit 1
fi

got="$(sha256_of "$tmpdir/$ASSET")"
if [[ "$got" != "$expected" ]]; then
  echo "fetch-tero-index: CHECKSUM MISMATCH for $ASSET — expected $expected, got $got. Refusing to use it (see tools/tero-rs/PROVENANCE.md before re-pinning)." >&2
  exit 1
fi

chmod +x "$tmpdir/$ASSET"
mv "$tmpdir/$ASSET" "$BIN_PATH"
echo "$BIN_PATH"
