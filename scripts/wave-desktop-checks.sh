#!/usr/bin/env bash
# scripts/wave-desktop-checks.sh — the DOGFOODING-WAVE heavy-check bundle for the maintainer's
# (many-core / GPU) desktop. The wave analogue of scripts/vsa-desktop-checks.sh.
#
# WHY. The dogfooding port wave (self-hosting the L1 semantic core into lib/compiler/semcore.myc,
# plus the parallel lib/std/*.myc ports) develops in constrained cloud sessions that run only the
# LIGHT tiers (`just check` / `just test-fast` + a change-scoped `myc check`). Three heavy checks are
# deliberately held OUT of that cloud gate (CLAUDE.md §Local checks; the /myc-dogfood + canary-tier
# notes) because they balloon or need real hardware:
#   1. the FULL `just check` (Tier-1) — a touch to a base crate (mycelium-core/-l1) pulls in EVERY
#      reverse-dependent crate's tests, a near-whole-workspace multi-hour run;
#   2. the durability tier `just check-full` (HIGH proptest + cargo-mutants + cargo-fuzz smoke);
#   3. the VSA/GPU-bound bundle + the z3/LiquidHaskell/Lean proof discharge (scripts/vsa-desktop-checks.sh).
# This script COLLECTS all three into ONE runnable, many-core-tuned, cautiously-staged bundle so they
# run ONCE on the desktop instead of being re-run in constrained cloud sessions (the M-1014 "accelerate
# check-full multicore/GPU" direction). Outputs land in a committable directory to push back.
#
# HONESTY (VR-5 / G2). Every rate/verdict this bundle surfaces keeps its source tag: the differential
# agreements are **Empirical** (trial-measured), proof obligations stay **Declared** until discharged.
# This script runs the checks and collects outputs; it NEVER upgrades a tag — the maintainer's analysis
# is the verdict. Every stage SKIPS GRACEFULLY (never-silent) when its toolchain is absent, so a partial
# desktop still yields a partial, honestly-labelled result set. Stages continue on failure (no `set -e`)
# but a failure is NARRATED and the script exits non-zero at the end — a broken run can never be pushed
# while labelled ok.
#
# STAGING is fast -> slow -> hardware, on purpose: a fast Tier-1 failure aborts before the ~40-min
# cargo-mutants run. Skip knobs let you run a subset.
#
# USAGE (from the repo root, on the desktop):
#   bash scripts/wave-desktop-checks.sh                      # full bundle (PROPTEST_CASES=256 default)
#   PROPTEST_CASES=1024 bash scripts/wave-desktop-checks.sh  # heaviest documented property-test run
#   WAVE_SKIP_CHECK_FULL=1 bash scripts/wave-desktop-checks.sh  # skip Tier-2 durability (mutants/fuzz)
#   WAVE_SKIP_VSA=1        bash scripts/wave-desktop-checks.sh  # skip the VSA/GPU sub-bundle
#   WAVE_ONLY=selfhost     bash scripts/wave-desktop-checks.sh  # run ONLY the self-hosting differentials
#   JOBS=32                bash scripts/wave-desktop-checks.sh  # override the core count (default: nproc)
#   RESULTS_DIR=/path      bash scripts/wave-desktop-checks.sh  # override the results location
# Then push the results back (a plain add/commit/push — these are Empirical/Declared evidence, not code):
#   git add experiments/results/wave-desktop && git commit -m "wave: desktop heavy-check results" && git push
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "$SCRIPT_DIR/lib.sh"
cd "$REPO_ROOT" || exit 1

RESULTS_DIR="${RESULTS_DIR:-$REPO_ROOT/experiments/results/wave-desktop}"
PROPTEST_CASES="${PROPTEST_CASES:-256}"          # desktop parity default (cloud uses 8); 1024 = heaviest
JOBS="${JOBS:-$( (command -v nproc >/dev/null 2>&1 && nproc) || echo 4 )}"
WAVE_ONLY="${WAVE_ONLY:-all}"
mkdir -p "$RESULTS_DIR"

# Saturate the box: cargo build parallelism + the shared sccache compile cache if present. These are
# exports so every `just`/`cargo` sub-invocation inherits them (the M-1014 many-core direction).
export CARGO_BUILD_JOBS="$JOBS"
have sccache && export RUSTC_WRAPPER="${RUSTC_WRAPPER:-sccache}" CARGO_INCREMENTAL=0

if ! have cargo || ! have just; then
  skip "wave-desktop-checks: cargo/just absent — nothing to run (install via scripts/bootstrap.sh)"
  exit 0
fi

FAILED=()
want() { [[ "$WAVE_ONLY" == "all" || "$WAVE_ONLY" == "$1" ]]; }

# ── Stage 1 — the FULL Tier-1 `just check` (the reverse-dep balloon held out of cloud) ──────────────
if want check; then
  section "Stage 1/4 — just check (Tier-1 full: change-scoped + reverse-deps + LOW proptest + all gates)"
  if just check >"$RESULTS_DIR/stage1-just-check.log" 2>&1; then
    ok "just check (Tier-1) green — see $RESULTS_DIR/stage1-just-check.log"
  else
    fail "just check (Tier-1) FAILED — $RESULTS_DIR/stage1-just-check.log"; FAILED+=("just check")
  fi
fi

# ── Stage 2 — the self-hosting differentials at FULL scale (dogfooding's core witness) ──────────────
if want selfhost; then
  section "Stage 2/4 — self-hosting differentials (semcore + stdlib) + native myc-dogfood --strict"
  # The Rust-oracle differentials for the ported .myc: the l1 compiler_stage* harnesses + the stdlib
  # conformance three-way. Full crate test, not the cloud change-scoped subset.
  if cargo test -p mycelium-l1 --lib >"$RESULTS_DIR/stage2-l1-differential.log" 2>&1; then
    ok "mycelium-l1 differential suite green (compiler_stage* + all)"
  else
    fail "mycelium-l1 differential suite FAILED — $RESULTS_DIR/stage2-l1-differential.log"; FAILED+=("l1 differential")
  fi
  if cargo test -p mycelium-std-conformance >"$RESULTS_DIR/stage2-std-conformance.log" 2>&1; then
    ok "mycelium-std-conformance three-way ports green"
  else
    fail "mycelium-std-conformance FAILED — $RESULTS_DIR/stage2-std-conformance.log"; FAILED+=("std-conformance")
  fi
  # The native second witness over ALL self-hosted nodules (strict: a core `myc check` failure is hard).
  if bash "$SCRIPT_DIR/checks/myc-dogfood.sh" --strict >"$RESULTS_DIR/stage2-myc-dogfood.log" 2>&1; then
    ok "myc-dogfood --strict green (native toolchain over the self-hosted lib/compiler nodules)"
  else
    fail "myc-dogfood --strict FAILED — $RESULTS_DIR/stage2-myc-dogfood.log"; FAILED+=("myc-dogfood")
  fi
fi

# ── Stage 3 — the durability tier (HIGH proptest + cargo-mutants + cargo-fuzz smoke) ────────────────
if want all && [[ "${WAVE_SKIP_CHECK_FULL:-0}" != "1" ]]; then
  section "Stage 3/4 — just check-full (Tier-2: PROPTEST_CASES=$PROPTEST_CASES · cargo-mutants · fuzz smoke) [SLOW]"
  if PROPTEST_CASES="$PROPTEST_CASES" just check-full >"$RESULTS_DIR/stage3-check-full.log" 2>&1; then
    ok "just check-full green (durability tier) — $RESULTS_DIR/stage3-check-full.log"
  else
    fail "just check-full FAILED (a mutant survived / proptest / fuzz) — $RESULTS_DIR/stage3-check-full.log"; FAILED+=("check-full")
  fi
else
  skip "Stage 3 — check-full skipped (WAVE_SKIP_CHECK_FULL=1 or WAVE_ONLY!=all)"
fi

# ── Stage 4 — the VSA/GPU + proof-discharge sub-bundle (delegated, never duplicated) ────────────────
if want all && [[ "${WAVE_SKIP_VSA:-0}" != "1" ]] && [[ -f "$SCRIPT_DIR/vsa-desktop-checks.sh" ]]; then
  section "Stage 4/4 — VSA/GPU heavy bundle (delegates to scripts/vsa-desktop-checks.sh)"
  if PROPTEST_CASES="$PROPTEST_CASES" bash "$SCRIPT_DIR/vsa-desktop-checks.sh" >"$RESULTS_DIR/stage4-vsa.log" 2>&1; then
    ok "vsa-desktop-checks green — results in experiments/results/vsa-m832/ (+ $RESULTS_DIR/stage4-vsa.log)"
  else
    fail "vsa-desktop-checks FAILED — $RESULTS_DIR/stage4-vsa.log"; FAILED+=("vsa-desktop")
  fi
else
  skip "Stage 4 — VSA/GPU bundle skipped (WAVE_SKIP_VSA=1, WAVE_ONLY!=all, or the sub-bundle is absent)"
fi

# ── Verdict (never-silent) ──────────────────────────────────────────────────────────────────────────
section "wave-desktop-checks summary (JOBS=$JOBS · results in $RESULTS_DIR)"
if [[ ${#FAILED[@]} -eq 0 ]]; then
  ok "all run stages green — push the results dir back for the record"
  exit 0
fi
fail "FAILED stages: ${FAILED[*]} — inspect the logs in $RESULTS_DIR (nothing pushed as ok, G2)"
exit 1
