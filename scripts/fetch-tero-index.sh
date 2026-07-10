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
# can `bin="$(scripts/fetch-tero-index.sh)"`); all diagnostics go to stderr. Exit codes are
# meaningful so the caller can tell a benign environment gap (skip) from a tampering signal (fail):
#   0 = resolved (cache hit or fresh verified fetch)
#   1 = UNRESOLVED / NOT FETCHABLE — no pin, unsupported platform, no `gh`, unauthenticated, or the
#       download failed. A benign environment gap; the caller SKIPS (never a false-red), mirroring
#       the "skip if cargo absent" convention every other Rust gate uses.
#   4 = CHECKSUM MISMATCH — a freshly-downloaded asset did not match its pinned SHA256. A
#       tampering/compromise signal, NOT an environment gap; the caller FAILS loudly, never skips.
# See scripts/checks/tero-index.sh, which maps 1->skip and 4->fail (G2: a mismatch is never silent).
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

# Platform guard: the pin (SHA256SUMS.txt) ships a linux-x86_64 asset only. On any other host the
# downloaded binary could pass the checksum yet fail to *execute*, which would surface as a hard
# error downstream rather than the intended skip. So refuse early with exit 1 (NOT-FETCHABLE ->
# skip), never a false-red. (Add per-platform asset selection here if the release later ships more.)
_os="$(uname -s 2>/dev/null || echo unknown)"
_arch="$(uname -m 2>/dev/null || echo unknown)"
case "$_os/$_arch" in
  Linux/x86_64 | Linux/amd64) : ;;
  *)
    echo "fetch-tero-index: pinned asset $ASSET is linux-x86_64 only; this host is $_os/$_arch — no fetchable binary (skip, not a failure)" >&2
    exit 1
    ;;
esac

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
  echo "fetch-tero-index: CHECKSUM MISMATCH for $ASSET — expected $expected, got $got. Refusing to use it (possible tampering; see tools/tero-rs/PROVENANCE.md before re-pinning)." >&2
  # exit 4 (NOT 1): a mismatch is a supply-chain tampering signal, not a benign not-fetchable gap —
  # the caller must FAIL, never skip. The mismatched download stays in $tmpdir (trap-cleaned), never
  # moved into the cache, so no partial/unverified artifact is left behind.
  exit 4
fi

chmod +x "$tmpdir/$ASSET"
mv "$tmpdir/$ASSET" "$BIN_PATH"
echo "$BIN_PATH"
