#!/usr/bin/env bash
# run.sh — one-command runner for the Grok/xAI co-authoring harness (M-330/M-331/M-381).
#
# Does, in order:
#   1. resolve a runner (uv preferred; falls back to the system python3),
#   2. sync deps (uv only; +xai_sdk when --batch),
#   3. the OFFLINE self-test — the green gate; ABORTS if it fails so you never spend on a
#      broken harness (G2),
#   4. list the resolved (cheapest-first) models — $0,
#   5. the capped live/batch run — ONLY if an xAI key is present; otherwise it stops after
#      the free checks and tells you how to set the key.
#
# The live path is pure stdlib (urllib) — no xai_sdk needed. The harness itself enforces the
# spend cap (--max-usd, default $10): a unit whose estimate would breach it is refused before
# it is sent. This wrapper adds nothing to the bill; it just sequences the calls.
#
# Usage (the script cd's to its own dir, so it operates correctly from any cwd — invoke it as
# `./run.sh` from tools/llm-harness/, or by an absolute/relative path from elsewhere, e.g.
# `tools/llm-harness/run.sh`):
#   ./run.sh                       # setup + self-test + list-models + (if key) the $10 sweep
#   ./run.sh --smoke               # + a cheap single-model ($2) smoke before the full sweep
#   ./run.sh --max-usd 5           # change the total spend cap (must be finite, >= 0)
#   ./run.sh --models grok-build-0.1[,grok-4.3]   # restrict to a subset (cheapest-first within it)
#   ./run.sh --no-ablation         # gold-set only (skip the M-381 retention-ratio ablation)
#   ./run.sh --batch               # batch-API mode (uv sync --extra batch → xai_sdk)
#   ./run.sh --check-only          # just the offline self-test + list-models; no key, no spend
#   ./run.sh -- --seed 23 ...      # everything after `--` is passed straight to grok.cli
set -euo pipefail

# --- locate ourselves so the script works from any cwd ----------------------------------
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$HERE"

# --- defaults ---------------------------------------------------------------------------
MAX_USD="10"
MODE="live"
ABLATION=(--ablation)  # gold-set + the M-381 retention-ratio ablation by default
MODELS=""              # empty => all, cheapest-first
SMOKE=0
CHECK_ONLY=0
PASSTHRU=()            # extra args after `--`

# --- parse args -------------------------------------------------------------------------
while [[ $# -gt 0 ]]; do
  case "$1" in
    --max-usd)      MAX_USD="${2:?--max-usd needs a value}"; shift 2 ;;
    --models)       MODELS="${2:?--models needs a value}"; shift 2 ;;
    --mode)         MODE="${2:?--mode needs a value}"; shift 2 ;;
    --batch)        MODE="batch"; shift ;;
    --ablation)     ABLATION=(--ablation); shift ;;
    --no-ablation)  ABLATION=(); shift ;;
    --smoke)        SMOKE=1; shift ;;
    --check-only)   CHECK_ONLY=1; shift ;;
    --)             shift; PASSTHRU=("$@"); break ;;
    -h|--help)      sed -n '2,32p' "$HERE/run.sh"; exit 0 ;;
    *)              echo "run.sh: unknown arg '$1' (use --help, or '--' to pass through)" >&2; exit 2 ;;
  esac
done

# Validate --mode early (grok.cli only accepts live|batch) so an invalid value fails clearly
# now — not later, after the setup + self-test have already run.
if [[ "$MODE" != "live" && "$MODE" != "batch" ]]; then
  echo "run.sh: --mode must be 'live' or 'batch', got '$MODE'" >&2; exit 2
fi

# --- resolve the runner: uv (preferred) else system python3 -----------------------------
# uv gives us the project venv + the console script + extras. Without uv, the LIVE path and
# self-test still work on plain python3 (stdlib only) — but --batch needs uv (the xai_sdk extra).
if command -v uv >/dev/null 2>&1; then
  RUNNER=(uv run python -m grok.cli)
  echo ">> syncing deps with uv ..."
  if [[ "$MODE" == "batch" ]]; then
    uv sync --extra batch
  else
    uv sync
  fi
elif command -v python3 >/dev/null 2>&1; then
  RUNNER=(python3 -m grok.cli)
  echo ">> uv not found — using system python3 (live + self-test are stdlib-only)."
  if [[ "$MODE" == "batch" ]]; then
    echo "!! --batch needs uv (the xai_sdk extra). Install uv, or run --mode live instead." >&2
    exit 1
  fi
else
  echo "!! neither uv nor python3 found on PATH." >&2; exit 1
fi

# --- 1. the offline green gate — abort if the plumbing is broken (never spend on it) ----
echo ">> offline self-test (no key, no network) ..."
if ! "${RUNNER[@]}" --self-test; then
  echo "!! self-test FAILED — refusing to proceed to any priced run (fix the harness first)." >&2
  exit 1
fi

# --- 2. show the resolved (cheapest-first) model order — $0 -----------------------------
echo ">> resolved models (cheapest-first) ..."
"${RUNNER[@]}" --list-models

[[ "$CHECK_ONLY" -eq 1 ]] && { echo ">> --check-only: done (offline checks passed; no spend)."; exit 0; }

# --- 3. need a key for any priced run; stop gracefully (not an error) if absent ---------
if [[ -z "${XAI_API_KEY:-}" && -z "${GROK_API_KEY:-}" ]]; then
  cat >&2 <<'EOF'
>> Offline checks passed. No xAI API key in the environment, so stopping before any priced run.
   To run the live experiment (capped):
       export XAI_API_KEY=...        # or GROK_API_KEY=...
       ./run.sh                      # re-run; it will do the capped sweep
EOF
  exit 0
fi

# --- 4. (optional) a cheap single-model smoke first -------------------------------------
SUBSET=()
[[ -n "$MODELS" ]] && SUBSET=(--models "$MODELS")
if [[ "$SMOKE" -eq 1 ]]; then
  echo ">> smoke: cheapest model, \$2 cap ..."
  "${RUNNER[@]}" --models grok-build-0.1 --mode "$MODE" "${ABLATION[@]}" --max-usd 2 "${PASSTHRU[@]}"
fi

# --- 5. the real, capped run ------------------------------------------------------------
echo ">> run: mode=$MODE  cap=\$$MAX_USD  ablation=${ABLATION[*]:-off}  models=${MODELS:-all-cheapest-first}"
"${RUNNER[@]}" --mode "$MODE" "${ABLATION[@]}" --max-usd "$MAX_USD" "${SUBSET[@]}" "${PASSTHRU[@]}"

echo ">> done. Reports written under: $HERE/reports/  (send me the newest *-report.json)"
