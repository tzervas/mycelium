#!/usr/bin/env bash
# Run machine-checkable proofs under proofs/. Skips when a prover toolchain is absent.
# (Heavy on a cold build — compiling LiquidHaskell from scratch is slow; incremental rebuilds
# are near-instant. The whole CI suite is manual-dispatch only, so this is acceptable there.)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "proofs"
rc=0

# ghcup installs outside the default PATH in some setups — make cabal discoverable.
export PATH="$HOME/.ghcup/bin:$PATH"

# M-001 — LiquidHaskell MAP-I `bundle` capacity probe (proofs/lh-bundle/).
# LiquidHaskell writes a UTF-8 HTML report, so a UTF-8 locale is required. A successful
# `cabal build` implies LH verified SAFE — an UNSAFE result fails the build.
if [[ -f proofs/lh-bundle/lh-bundle.cabal ]] && have cabal && have z3; then
  if out="$( cd proofs/lh-bundle && LC_ALL=C.UTF-8 LANG=C.UTF-8 cabal build 2>&1 )"; then
    if grep -q "LIQUID: SAFE" <<<"$out"; then
      ok "lh-bundle: $(grep -o 'LIQUID: SAFE.*\*' <<<"$out" | head -1)"
    else
      ok "lh-bundle: build OK (LH verified; nothing to recompile)"
    fi
  else
    fail "lh-bundle: LiquidHaskell/Z3 did not verify"; echo "$out" | tail -25; rc=1
  fi
else
  skip "lh-bundle: cabal/z3 absent (needs GHC + LiquidHaskell + z3)"
fi

# M-121 — binary<->ternary round-trip / injectivity (proofs/binary-ternary-roundtrip/).
# A pure SMT-LIB2 obligation: Z3 must report `unsat` (no two distinct 6-trit vectors collide ⟹ the
# value map is injective ⟹ dec(enc b) = b for all b ∈ B_8). Skips when z3 is absent.
if [[ -f proofs/binary-ternary-roundtrip/roundtrip_8x6.smt2 ]] && have z3; then
  if out="$( z3 -smt2 proofs/binary-ternary-roundtrip/roundtrip_8x6.smt2 2>&1 )" \
     && [[ "$(printf '%s' "$out" | tr -d '[:space:]')" == "unsat" ]]; then
    ok "binary-ternary-roundtrip: Z3 unsat (injectivity ⟹ round-trip, 8↔6)"
  else
    fail "binary-ternary-roundtrip: expected 'unsat', got: $out"; rc=1
  fi
else
  skip "binary-ternary-roundtrip: z3 absent (needs z3)"
fi

exit $rc
