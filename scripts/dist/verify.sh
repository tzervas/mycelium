#!/usr/bin/env bash
# Self-test for the reproducible-install mechanism (M-734). Builds a synthetic artifact, pins it,
# and asserts: (1) a re-install of the same pinned artifact is BYTE-IDENTICAL, (2) any mutation of
# the artifact is caught as a never-silent mismatch (G2), (3) a missing pinned file is caught.
#
# This proves the install/verify LOGIC end-to-end without depending on a reproducible compiler
# build (that build is the deferred piece — see install.sh header). STRICT: a missing hasher is a
# hard failure here too (never skip), matching the install path.
#
# Exit: 0 all assertions pass · non-zero on the first failed assertion.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL="$HERE/install.sh"

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
art="$work/artifact"
pins="$work/pins"
mkdir -p "$art/bin"

# A synthetic, deterministic toolchain artifact.
printf '#!/bin/sh\necho myc 0.1.0\n' > "$art/bin/myc"
printf '#!/bin/sh\necho spore 0.1.0\n' > "$art/bin/spore"
printf 'mycelium toolchain 0.1.0\n' > "$art/VERSION"

pass() { echo "  PASS: $*"; }
fail() { echo "  FAIL: $*" >&2; exit 1; }

echo "dist self-test:"

# --- pin it ---
bash "$INSTALL" --artifact "$art" --pins "$pins" --update-pins >/dev/null
[[ -s "$pins" ]] || fail "pin file was not written"
pass "pinned the artifact"

# --- (1) re-install is byte-identical ---
out1="$work/install1"; out2="$work/install2"
bash "$INSTALL" --artifact "$art" --pins "$pins" --prefix "$out1" >/dev/null
bash "$INSTALL" --artifact "$art" --pins "$pins" --prefix "$out2" >/dev/null
if diff -r "$out1" "$out2" >/dev/null; then
  pass "two installs of the same pin are byte-identical"
else
  fail "re-install was not byte-identical"
fi

# --- verify-only passes on the pristine artifact ---
if bash "$INSTALL" --artifact "$art" --pins "$pins" --verify-only >/dev/null; then
  pass "verify-only passes on the pinned artifact"
else
  fail "verify-only failed on a pristine artifact"
fi

# --- (2) a mutated file is caught (never-silent) ---
printf '#!/bin/sh\necho TAMPERED\n' > "$art/bin/myc"
if bash "$INSTALL" --artifact "$art" --pins "$pins" --verify-only >/dev/null 2>&1; then
  fail "a tampered artifact was NOT caught (G2 violated)"
else
  rc=$?
  [[ "$rc" -eq 5 ]] || fail "tamper detected but exit code was $rc, expected 5 (integrity mismatch)"
  pass "a mutated file is caught with the integrity exit code (5)"
fi
# restore
printf '#!/bin/sh\necho myc 0.1.0\n' > "$art/bin/myc"

# --- (3) a missing pinned file is caught ---
rm "$art/VERSION"
if bash "$INSTALL" --artifact "$art" --pins "$pins" --verify-only >/dev/null 2>&1; then
  fail "a missing pinned file was NOT caught (G2 violated)"
else
  pass "a missing pinned file is caught"
fi

echo "dist self-test: ALL PASSED"
