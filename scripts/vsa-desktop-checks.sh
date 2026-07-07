#!/usr/bin/env bash
# scripts/vsa-desktop-checks.sh — the VSA HEAVY-CHECK bundle for the maintainer's (GPU) desktop.
#
# WHY. Heavy VSA/GPU-bound work + the durability tier (HIGH proptest, cargo-mutants, cargo-fuzz) +
# the z3/LiquidHaskell/Lean proof discharge are deliberately held OUT of the cloud-session
# `just check` gate (the `/myc-dogfood` note; CLAUDE.md §Local checks) — they belong on a
# local/teleport machine. This script COLLECTS them into one runnable bundle so they run ONCE on
# the desktop instead of being re-run in constrained cloud sessions, and the outputs land in a
# committable directory to push back (the M-832 / OQ-F evidence — DN-34/RFC-0003/ADR-010).
#
# HONESTY (VR-5 / G2). The experiment's reported rates are **Empirical** (trial-measured). The
# emitted proof obligations are **Declared** until a solver discharges them AND the underlying
# theorem is formally established/cited. This script never upgrades those tags — it runs the checks
# and collects outputs; the maintainer's analysis is the verdict, not this script. Every stage
# **skips gracefully** (never-silent) when its toolchain is absent, so a partial desktop still
# yields a partial, honestly-labelled result set.
#
# USAGE (from the repo root, on the desktop):
#   bash scripts/vsa-desktop-checks.sh                     # full bundle (PROPTEST_CASES=64 default)
#   PROPTEST_CASES=256 bash scripts/vsa-desktop-checks.sh  # justfile full-tier parity batch count
#   PROPTEST_CASES=1024 bash scripts/vsa-desktop-checks.sh # heaviest documented property-test run
#   VSA_SKIP_MUTANTS=1 bash scripts/vsa-desktop-checks.sh  # skip cargo-mutants (the slowest stage)
#   VSA_IGNORED_ONLY=1 bash scripts/vsa-desktop-checks.sh  # run ONLY the #[ignore] heavy instruments
#                                                          #   (cargo --ignored --nocapture) into a SEPARATE
#                                                          #   log and skip stages 2-4 — the follow-up run that
#                                                          #   supplements the default (non-ignored) push
#   RESULTS_DIR=/path  bash scripts/vsa-desktop-checks.sh  # override the results location
# Then push the results back:
#   git add experiments/results/vsa-m832 && git commit -m "vsa: desktop heavy-check results" && git push

set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "$SCRIPT_DIR/lib.sh"
cd "$REPO_ROOT" || exit 1

RESULTS_DIR="${RESULTS_DIR:-$REPO_ROOT/experiments/results/vsa-m832}"
# Default 64 batches (each VSA proptest fn multiplies this into independent seed batches; the
# resonator profile alone is 1,000 trials/batch at dim 4096). 64 in --release keeps the whole
# bundle inside a desktop wall-clock envelope with far more statistical power than the old
# 256-batch DEBUG default ever delivered; opt up (256 = justfile full-tier parity, 1024 = the
# heaviest documented run) when you want the extra batches. (2026-07-06 fix: the original
# 256-batch debug default pinned one core for 36+ min on resonator_profile alone.)
PROPTEST_CASES="${PROPTEST_CASES:-64}"
VSA_IGNORED_ONLY="${VSA_IGNORED_ONLY:-0}"
mkdir -p "$RESULTS_DIR/obligations"

# Stages continue on failure by design (no `set -e`) — but a failure is NARRATED as a failure and
# the script exits non-zero at the end, so a broken run can never be pushed while labeled ok (G2).
FAILED=()

section "VSA desktop heavy-checks — collecting into $RESULTS_DIR"
echo "  Emission tags: experiment = Empirical (trial-measured); proof obligations = Declared until discharged (VR-5/G2)."

# ── IGNORED-ONLY follow-up mode ────────────────────────────────────────────────────────────────
# VSA_IGNORED_ONLY=1 runs ONLY the #[ignore]-marked heavy instruments (e.g. resonator_capacity_sweep,
# resonator_cleanup_ablation — "run manually with --ignored --nocapture") into a SEPARATE log, then
# exits — so a follow-up push supplements the default run's non-ignored results without overwriting.
if [ "$VSA_IGNORED_ONLY" = 1 ]; then
  section "IGNORED-ONLY — heavy #[ignore] instruments (mycelium-vsa + mycelium-std-vsa, --ignored --nocapture)"
  if have cargo; then
    # --release: these are heavy numeric instruments — the debug profile is 10-50x slower here.
    # (PROPTEST_CASES is the only live knob: no VSA test reads MYC_TEST_TIER — that var is
    # consumed solely by scripts/checks/test.sh, so prefixing it here was an inert no-op.)
    if PROPTEST_CASES="$PROPTEST_CASES" \
      cargo test --release -p mycelium-vsa -p mycelium-std-vsa -- --ignored --nocapture \
      2>&1 | tee "$RESULTS_DIR/vsa-crate-tests-ignored.log"; then
      ok "ignored instruments -> $RESULTS_DIR/vsa-crate-tests-ignored.log (Empirical)"
    else
      fail "ignored instruments FAILED — see $RESULTS_DIR/vsa-crate-tests-ignored.log"
      FAILED+=("ignored-instruments")
    fi
  else
    skip "no cargo — VSA ignored instruments skipped"
  fi
  section "done (ignored-only) — stages 2-4 intentionally skipped; supplement in $RESULTS_DIR/vsa-crate-tests-ignored.log"
  echo "  Push back:  git add experiments/results/vsa-m832 && git commit -m 'vsa: ignored heavy instruments' && git push"
  [ "${#FAILED[@]}" -eq 0 ] || exit 1
  exit 0
fi

# ── 1/4 · VSA crate durability (full tier, HIGH proptest) — DEFAULT run (EXCLUDES #[ignore]) ─────
section "1/4 VSA crate durability — mycelium-vsa + mycelium-std-vsa (full tier, PROPTEST_CASES=$PROPTEST_CASES)"
if have cargo; then
  # --release: see the IGNORED-ONLY note — debug was the 36-min-single-core failure mode.
  if PROPTEST_CASES="$PROPTEST_CASES" \
    cargo test --release -p mycelium-vsa -p mycelium-std-vsa 2>&1 | tee "$RESULTS_DIR/vsa-crate-tests.log"; then
    ok "crate durability -> $RESULTS_DIR/vsa-crate-tests.log (Empirical; #[ignore] instruments excluded — rerun with VSA_IGNORED_ONLY=1 for those)"
  else
    fail "crate durability FAILED — see $RESULTS_DIR/vsa-crate-tests.log"
    FAILED+=("crate-durability")
  fi
else
  skip "no cargo — VSA crate durability skipped"
fi

# ── 2/4 · M-832 GPU experiment + proof-obligation emission ─────────────────────────────────────
section "2/4 M-832 GPU experiment — VSA multi-hop Proven-bounds (OQ-F)"
if have uv; then
  ( cd experiments && { uv sync --group gpu 2>/dev/null || uv sync 2>/dev/null || true; } )
  # Gate the GPU-labeled log on an ACTUAL CUDA device, not torch's mere importability — a CPU-only
  # torch previously wrote m832-sweep-gpu.log (an honest log body under a misleading filename).
  if ( cd experiments && uv run python -c "import torch, sys; sys.exit(0 if torch.cuda.is_available() else 1)" 2>/dev/null ); then
    if ( cd experiments && uv run python -m mycelium_experiments.vsa_bounds --sweep both ) \
      2>&1 | tee "$RESULTS_DIR/m832-sweep-gpu.log"; then
      ok "GPU (CUDA) sweep -> $RESULTS_DIR/m832-sweep-gpu.log (Empirical)"
    else
      fail "GPU sweep FAILED — see $RESULTS_DIR/m832-sweep-gpu.log"
      FAILED+=("m832-sweep-gpu")
    fi
  elif ( cd experiments && uv run python -c "import torch" 2>/dev/null ); then
    skip "torch present but no CUDA device — full sweep on torch-CPU into the cpu log"
    if ( cd experiments && uv run python -m mycelium_experiments.vsa_bounds --sweep both ) \
      2>&1 | tee "$RESULTS_DIR/m832-sweep-cpu.log"; then
      ok "torch-CPU sweep -> $RESULTS_DIR/m832-sweep-cpu.log (Empirical; no CUDA device)"
    else
      fail "torch-CPU sweep FAILED — see $RESULTS_DIR/m832-sweep-cpu.log"
      FAILED+=("m832-sweep-cpu")
    fi
  else
    skip "torch absent — running the CPU numpy fallback (--quick, degraded coverage, still Empirical)"
    if ( cd experiments && uv run python -m mycelium_experiments.vsa_bounds --sweep both --quick --numpy-only --no-plots ) \
      2>&1 | tee "$RESULTS_DIR/m832-sweep-cpu.log"; then
      ok "numpy --quick sweep -> $RESULTS_DIR/m832-sweep-cpu.log (Empirical; degraded coverage)"
    else
      fail "numpy fallback sweep FAILED — see $RESULTS_DIR/m832-sweep-cpu.log"
      FAILED+=("m832-sweep-numpy")
    fi
  fi
  if ( cd experiments && uv run python -m mycelium_experiments.vsa_bounds --proof --emit-obligations --results-dir "$RESULTS_DIR/obligations" ) \
    2>&1 | tee "$RESULTS_DIR/m832-proof-emit.log"; then
    ok "proof obligations + PROOF-SUMMARY -> $RESULTS_DIR/obligations (Declared until discharged)"
  else
    fail "proof-obligation emission FAILED — see $RESULTS_DIR/m832-proof-emit.log"
    FAILED+=("m832-proof-emit")
  fi
else
  skip "no uv — M-832 experiment skipped (install: see experiments/README.md)"
fi

# ── 3/4 · VSA proof discharge (z3 · LiquidHaskell · Lean) ──────────────────────────────────────
section "3/4 VSA proof discharge — proofs/vsa-multihop-bound + the emitted obligations"
if have z3; then
  : > "$RESULTS_DIR/proof-z3.log"
  found=0
  for smt in "$RESULTS_DIR"/obligations/*.smt2 proofs/vsa-multihop-bound/*.smt2; do
    [ -e "$smt" ] || continue
    found=1
    { echo "== z3 $smt =="; z3 "$smt" 2>&1; echo; } >> "$RESULTS_DIR/proof-z3.log"
  done
  if [ "$found" = 1 ]; then
    ok "z3 -> $RESULTS_DIR/proof-z3.log (Empirical: per-obligation solver verdict)"
  else
    skip "no .smt2 obligations present for z3"
  fi
else
  skip "no z3 — SMT discharge skipped"
fi
if have cabal; then
  if ( cd proofs/vsa-multihop-bound && cabal build ) 2>&1 | tee "$RESULTS_DIR/proof-lh.log"; then
    ok "LiquidHaskell (cabal) -> $RESULTS_DIR/proof-lh.log"
  else
    fail "LiquidHaskell (cabal) build FAILED — see $RESULTS_DIR/proof-lh.log"
    FAILED+=("proof-lh")
  fi
else
  skip "no cabal — LiquidHaskell discharge skipped"
fi
if have lake || have lean; then
  if [ -d proofs/vsa-multihop-bound/lean ]; then
    if ( cd proofs/vsa-multihop-bound/lean && { have lake && lake build || lean --version; } ) \
      2>&1 | tee "$RESULTS_DIR/proof-lean.log"; then
      ok "Lean -> $RESULTS_DIR/proof-lean.log"
    else
      fail "Lean build FAILED — see $RESULTS_DIR/proof-lean.log"
      FAILED+=("proof-lean")
    fi
  else
    skip "no proofs/vsa-multihop-bound/lean project dir — Lean discharge skipped"
  fi
else
  skip "no lean/lake — Lean discharge skipped"
fi

# ── 4/4 · VSA mutation durability (slowest; opt-out) ───────────────────────────────────────────
section "4/4 VSA durability — cargo-mutants on the VSA crates (slowest stage)"
if [ "${VSA_SKIP_MUTANTS:-0}" = 1 ]; then
  skip "VSA_SKIP_MUTANTS=1 — cargo-mutants skipped by request"
elif cargo mutants --version >/dev/null 2>&1; then
  if cargo mutants -p mycelium-vsa -p mycelium-std-vsa 2>&1 | tee "$RESULTS_DIR/vsa-mutants.log"; then
    ok "cargo-mutants -> $RESULTS_DIR/vsa-mutants.log (no missed mutants)"
  else
    # A non-zero mutants exit usually means missed/timeout mutants — a durability FINDING the
    # maintainer grades, not a script failure (the bundle collects evidence, it does not grade
    # it). Narrated loudly, never as ok; not added to FAILED.
    echo "  note  cargo-mutants exited non-zero (missed/timeout mutants are findings — read $RESULTS_DIR/vsa-mutants.log)"
  fi
else
  skip "cargo-mutants not installed — VSA mutation testing skipped (install: cargo install cargo-mutants)"
fi

if [ "${#FAILED[@]}" -gt 0 ]; then
  section "done WITH FAILURES — results collected in $RESULTS_DIR"
  fail "failed stage(s): ${FAILED[*]} — fix or annotate before pushing results (never push a broken run labeled ok)"
  exit 1
fi
section "done — results collected in $RESULTS_DIR"
echo "  Push back:  git add experiments/results/vsa-m832 && git commit -m 'vsa: desktop heavy-check results' && git push"
echo "  Reminder (VR-5/G2): experiment rates are Empirical; proof obligations stay Declared until a"
echo "  solver discharges them AND the theorem is established — this bundle collects evidence, it does not grade it."
