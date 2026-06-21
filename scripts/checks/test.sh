#!/usr/bin/env bash
# Run the Rust + Python test suites, tiered + change-scoped (DN-20). Skips languages not present.
#
# TIER (env MYC_TEST_TIER, default `check`):
#   fast   — Tier 0 (pre-commit): change-scoped crates only; unit + regression/witness tests.
#            NO integration tests, NO proptest, NO doctests. Ultra-fast feedback. `cargo test --lib`
#            on the scoped `-p` set (proptests live in tests/, so --lib excludes them inherently).
#   check  — Tier 1 (default; local↔CI parity): change-scoped crates + reverse-deps; unit +
#            regression/witness + integration + proptest at LOW cases (PROPTEST_CASES, default 8).
#            Runs all test targets on the scoped set, plus a cheap workspace doctest pass.
#   full   — Tier 2 (release/durability): the FULL workspace; proptest at HIGH cases (default 256).
#            Mutants + fuzz are wired by `just check-full` (separate recipes), not here.
#
# RUNNER: `cargo nextest run` when cargo-nextest is installed (faster, parallel), else `cargo test`
# (so local↔CI parity holds when nextest is absent — DN-20). nextest does NOT execute doctests, so
# the `check`/`full` tiers add an explicit `cargo test --doc` pass (cheap) to keep doctests covered.
#
# CHANGE-SCOPING (fast/check): scripts/checks/changed-crates.sh emits the cargo package selection
# (`-p ...`, `--workspace`, or the `--no-changes` sentinel). It is conservative — a shared/root-file
# change or any detection failure widens to `--workspace`; it NEVER under-tests (DN-20). The `full`
# tier ignores scoping and always runs `--workspace`.
#
# Honesty (house rule 1 / VR-5): no property/bound test is ever dropped — only its CASE COUNT is
# tiered (low every commit via PROPTEST_CASES, full on release). Every bound is exercised on every
# commit at low cases; the full statistical power runs in the `full` tier. (DN-20 honesty guardrail.)
source "${BASH_SOURCE%/*}/../lib.sh"
cd "$REPO_ROOT" || exit 1

TIER="${MYC_TEST_TIER:-check}"
section "tests (tier: $TIER)"
rc=0

# --- Pick the test runner: nextest if present, else cargo test (parity fallback). -----------------
# We expose the choice as two arrays so the per-tier logic reads the same regardless of runner.
runner_desc=""
if have cargo-nextest; then
  runner_desc="cargo nextest"
elif have cargo; then
  runner_desc="cargo test (nextest absent — parity fallback)"
fi

# run_tests <mode> <selection-args...> — execute the chosen runner over the selection.
#   mode=lib  : unit + regression tests only (the fast tier; excludes integration/proptest in tests/)
#   mode=all  : every test target (unit + integration + proptest)
# Uses nextest when available; nextest's `--lib` selects unit tests, matching `cargo test --lib`.
run_tests() {
  local mode="$1"; shift
  local sel=("$@")
  if have cargo-nextest; then
    case "$mode" in
      lib) cargo nextest run --lib "${sel[@]}" ;;
      all) cargo nextest run "${sel[@]}" ;;
    esac
  else
    case "$mode" in
      lib) cargo test --lib "${sel[@]}" ;;
      all) cargo test "${sel[@]}" ;;
    esac
  fi
}

# run_doctests <selection-args...> — the doctest pass nextest cannot run (it skips doctests).
# Always `cargo test --doc` (the only runner that executes doctests); honors the same crate
# selection so it stays scoped on the `check` tier. The selection may contain `--all-features` /
# `--workspace` / `-p ...` — all valid for `--doc`.
run_doctests() {
  cargo test --doc "$@"
}

# --- Rust ------------------------------------------------------------------------------------------
if [[ -f Cargo.toml ]] && have cargo; then
  # Resolve the crate selection for fast/check; full is always the whole workspace.
  selection=()
  run_rust=1
  if [[ "$TIER" == "full" ]]; then
    selection=(--workspace --all-features)
    log_sel="--workspace --all-features"
  else
    # changed-crates.sh prints the selection on stdout, narration on stderr (which we let through).
    sel_str="$(bash "$REPO_ROOT/scripts/checks/changed-crates.sh")" || sel_str="--workspace"
    if [[ "$sel_str" == "--no-changes" ]]; then
      skip "no changed crates vs base — Rust tests skipped for this tier ($TIER); \`just check-full\` runs the full workspace"
      run_rust=0
    elif [[ "$sel_str" == "--workspace" ]]; then
      selection=(--workspace)
      log_sel="--workspace (conservative)"
    else
      # shellcheck disable=SC2206  # intentional word-split of the `-p crate -p crate` arg string
      selection=($sel_str)
      log_sel="$sel_str"
    fi
  fi

  if [[ "$run_rust" -eq 1 ]]; then
    ok "runner: $runner_desc; selection: $log_sel"
    case "$TIER" in
      fast)
        # Tier 0: unit + regression/witness only (no integration, no proptest, no doctests).
        if run_tests lib "${selection[@]}"; then ok "cargo (lib/unit) — fast tier"; else fail "fast-tier test failures"; rc=1; fi
        ;;
      check|full)
        # Tier 1/2: all test targets (unit + integration + proptest). PROPTEST_CASES (exported by
        # the justfile recipe) tiers the proptest case count: low for `check`, high for `full`.
        if run_tests all "${selection[@]}"; then ok "cargo (all targets) — $TIER tier (PROPTEST_CASES=${PROPTEST_CASES:-unset})"; else fail "$TIER-tier test failures"; rc=1; fi
        # nextest does not run doctests; keep them covered with a cheap explicit pass (both tiers).
        # Scope the doctest pass to the same selection so `check` stays fast.
        if run_doctests "${selection[@]}"; then ok "cargo test --doc"; else fail "doctest failures"; rc=1; fi
        ;;
      *)
        fail "unknown MYC_TEST_TIER='$TIER' (expected: fast | check | full)"; rc=1
        ;;
    esac
  fi
else
  skip "rust: no Cargo.toml or cargo"
fi

# --- Python — the uv-managed experiments project (M-092), pinned interpreter (3.13; ADR-007). -----
# Python tests are cheap and not crate-scoped; run them on check/full (skip on the fast pre-commit
# tier to keep it Rust-only and instant). Override by running `just check`.
if [[ "$TIER" != "fast" ]]; then
  if [[ -f experiments/pyproject.toml ]] && have uv; then
    if ( cd experiments && uv run --frozen pytest ); then ok "uv run pytest (experiments)"; else fail "pytest failures"; rc=1; fi
  else
    skip "python: no uv experiments project (or uv missing)"
  fi
fi

exit $rc
