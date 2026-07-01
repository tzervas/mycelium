#!/usr/bin/env bash
# spore-oci-selftest.sh — local OCI round-trip proof of the ADR-037 / M-871 remote spore-registry
# backend (GHCR/OCI dense-map). Stands up a throwaway `registry:2` (podman/docker), publishes the
# example phyla to `oci://localhost:<port>`, resolves them back into a fresh tree, and lets the
# built-in fetch-and-verify (every object BLAKE3-checked; spore_id recomputed) confirm integrity.
# Account-free — the CI-shaped twin of the live `spore-ghcr-dogfood.sh`.
#
# Never-silent (G2): a missing tool SKIPs (exit 0, loudly); a publish/resolve failure FAILs (exit 1).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

REG_PORT="${REG_PORT:-5000}"
REG="localhost:${REG_PORT}"
REF="oci://${REG}"
VERSION="${SPORE_SELFTEST_VERSION:-0.0.0-selftest}"

skip() { echo "spore-oci-selftest: SKIP — $1"; exit 0; }
command -v oras >/dev/null 2>&1 || skip "'oras' not installed (see ADR-037: oras is the v0 transport)"
command -v curl >/dev/null 2>&1 || skip "'curl' not installed"
RUNTIME=""
for r in podman docker; do
  if command -v "$r" >/dev/null 2>&1; then RUNTIME="$r"; break; fi
done
[ -n "$RUNTIME" ] || skip "need podman or docker to run a local registry:2"

echo "== build the spore CLI =="
cargo build -q -p mycelium-spore --bin spore
SPORE="$ROOT/target/debug/spore"

CID=""
cleanup() { [ -n "$CID" ] && "$RUNTIME" rm -f "$CID" >/dev/null 2>&1 || true; }
trap cleanup EXIT

echo "== start throwaway registry:2 on :${REG_PORT} =="
CID="$("$RUNTIME" run -d -p "${REG_PORT}:5000" docker.io/library/registry:2)"
for _ in $(seq 1 20); do
  curl -fsS "http://${REG}/v2/" >/dev/null 2>&1 && break
  sleep 0.5
done
curl -fsS "http://${REG}/v2/" >/dev/null 2>&1 || { echo "spore-oci-selftest: FAIL — registry did not come up"; exit 1; }

phyla=(examples/hello-phylum lib/std)
fail=0
for proj in "${phyla[@]}"; do
  manifest="$proj/mycelium-proj.toml"
  [ -f "$manifest" ] || { echo "  -- skip $proj (no manifest)"; continue; }
  # The package name is the manifest's [project].name — read it back from the spore's own EXPLAIN
  # receipt (our format), never guessed.
  name="$("$SPORE" explain --config "$manifest" | head -1 | sed -E 's/^spore:[[:space:]]*([^[:space:]]+).*/\1/')"
  [ -n "$name" ] || { echo "  -- FAIL $proj: could not read package name from explain"; fail=1; continue; }

  echo "== publish ${name}@${VERSION}  ($proj) =="
  if ! "$SPORE" publish --registry "$REF" --config "$manifest" --version "$VERSION"; then
    echo "  -- FAIL: publish ${name}"; fail=1; continue
  fi

  out="$(mktemp -d)"
  echo "== resolve ${name}@${VERSION} -> ${out} =="
  if ! "$SPORE" resolve "$name" "$VERSION" --registry "$REF" -o "$out"; then
    echo "  -- FAIL: resolve ${name}"; fail=1; rm -rf "$out"; continue
  fi
  echo "  resolved + verified tree:"
  find "$out" -type f | sed "s#^$out/#    #" | sort
  rm -rf "$out"
done

if [ "$fail" -eq 0 ]; then
  echo "spore-oci-selftest: OK — local OCI publish/resolve round-trip verified (ADR-037 dense-map, fetch-and-verify green)"
else
  echo "spore-oci-selftest: FAILED"; exit 1
fi
