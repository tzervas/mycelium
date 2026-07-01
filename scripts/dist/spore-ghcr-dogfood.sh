#!/usr/bin/env bash
# spore-ghcr-dogfood.sh — LIVE dogfood of the ADR-037 / M-871 remote spore registry against the
# GitHub Packages container registry (GHCR). Publishes the example phyla to
# `ghcr.io/<owner>/<phylum>:<version>` and resolves them back, verifying end-to-end — the live proof
# (and stress test) of the registry design (DN-28) + implementation (mycelium-spore) the maintainer's
# release strategy (ADR-036) calls for before the repo flips public.
#
# Auth: needs `oras` and a token with `write:packages`,`read:packages` in GH_TOKEN or CR_PAT.
#   GH_TOKEN=<pat> bash scripts/dist/spore-ghcr-dogfood.sh <owner>
# Outward-facing (creates packages under the owner's namespace) — run deliberately. Never-silent (G2):
# a missing tool/token is an explicit refusal, not a silent skip; a publish/resolve failure is fatal.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

OWNER="${1:-tzervas}"
VERSION="${SPORE_DOGFOOD_VERSION:-0.0.0-dogfood}"
REF="ghcr://${OWNER}"

die() { echo "spore-ghcr-dogfood: $1" >&2; exit "${2:-1}"; }
command -v oras >/dev/null 2>&1 || die "'oras' not installed (ADR-037: oras is the v0 OCI transport)" 69

TOKEN="${GH_TOKEN:-${CR_PAT:-}}"
[ -n "$TOKEN" ] || die "no token — export GH_TOKEN or CR_PAT with write:packages,read:packages scope" 77

echo "== oras login ghcr.io as ${OWNER} =="
printf '%s' "$TOKEN" | oras login ghcr.io -u "$OWNER" --password-stdin \
  || die "oras login failed — check the token has write:packages,read:packages" 77

echo "== build the spore CLI =="
cargo build -q -p mycelium-spore --bin spore
SPORE="$ROOT/target/debug/spore"

phyla=(examples/hello-phylum lib/std)
fail=0
for proj in "${phyla[@]}"; do
  manifest="$proj/mycelium-proj.toml"
  [ -f "$manifest" ] || { echo "  -- skip $proj (no manifest)"; continue; }
  name="$("$SPORE" explain --config "$manifest" | head -1 | sed -E 's/^spore:[[:space:]]*([^[:space:]]+).*/\1/')"
  [ -n "$name" ] || { echo "  -- FAIL $proj: could not read package name from explain"; fail=1; continue; }

  echo "== publish ghcr.io/${OWNER}/${name}:${VERSION}  ($proj) =="
  if ! "$SPORE" publish --registry "$REF" --config "$manifest" --version "$VERSION"; then
    echo "  -- FAIL: publish ${name}"; fail=1; continue
  fi

  out="$(mktemp -d)"
  echo "== resolve ${name}@${VERSION} from GHCR -> ${out} =="
  if ! "$SPORE" resolve "$name" "$VERSION" --registry "$REF" -o "$out"; then
    echo "  -- FAIL: resolve ${name}"; fail=1; rm -rf "$out"; continue
  fi
  echo "  resolved + verified tree:"
  find "$out" -type f | sed "s#^$out/#    #" | sort
  rm -rf "$out"
done

if [ "$fail" -eq 0 ]; then
  echo "spore-ghcr-dogfood: OK — live GHCR publish/resolve round-trip verified for ${#phyla[@]} phyla under ghcr.io/${OWNER}"
  echo "  (packages are hosted in the GitHub Packages registry; ADR-037 -> Enacted on this basis)"
else
  echo "spore-ghcr-dogfood: FAILED"; exit 1
fi
