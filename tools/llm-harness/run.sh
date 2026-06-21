#!/usr/bin/env bash
# run.sh — unified one-command runner for all M-381 arms (M-330/M-331/M-381).
#
# Does, in order:
#   1. resolve a runner (uv preferred; falls back to the system python3),
#   2. sync deps (uv only; +xai_sdk when --batch),
#   3. the OFFLINE self-test — the green gate; ABORTS if it fails so you never spend on a
#      broken harness (G2),
#   4. list the resolved (cheapest-first) models — $0,
#   5. the capped live/batch xAI run — ONLY if an xAI key is present; otherwise it stops
#      after the free checks and tells you how to set the key (skipped entirely with --local).
#   6. (--all / --local only) idempotent local arm-3 setup via setup_local_llm.py, then
#      the local grammar-constrained run via run_arm3_local.py — skipped with an explicit
#      message if no backend is available (G2: never silent).
#
# The live path is pure stdlib (urllib) — no xai_sdk needed. The harness itself enforces the
# spend cap (--max-usd, default $10): a unit whose estimate would breach it is refused before
# it is sent. This wrapper adds nothing to the bill; it just sequences the calls.
#
# Prerequisites:
#   xAI/Grok path:   XAI_API_KEY (or GROK_API_KEY) — skipped gracefully if absent.
#   local arm-3:     RECOMMENDED an NVIDIA GPU + ~5 GB VRAM (CPU fallback works, just slower) +
#                    ~5 GB disk for the 7B Q4_K_M GGUF (setup_local_llm.py downloads it). A
#                    missing GPU is warn-only, never fatal. setup_local_llm.py always runs
#                    (idempotent); inference SKIPs (explicit, G2) only when the backend/model is
#                    still unavailable AFTER setup (no llama_cpp / no model) — not just without a GPU.
#
# Usage (the script cd's to its own dir, so it operates correctly from any cwd — invoke it as
# `./run.sh` from tools/llm-harness/, or by an absolute/relative path from elsewhere, e.g.
# `tools/llm-harness/run.sh`):
#   ./run.sh                       # setup + self-test + list-models + (if key) the $10 sweep
#   ./run.sh --all                 # BOTH paths: xAI sweep (if key) AND local arm-3 setup + run
#   ./run.sh --local               # local arm-3 only: setup_local_llm.py + run_arm3_local.py
#   ./run.sh --smoke               # + a cheap single-model ($2) smoke before the full sweep
#   ./run.sh --max-usd 5           # change the total spend cap (must be finite, >= 0)
#   ./run.sh --models grok-build-0.1[,grok-4.3]   # restrict to a subset (cheapest-first within it)
#   ./run.sh --no-ablation         # gold-set only (skip the M-381 retention-ratio ablation)
#   ./run.sh --batch               # batch-API mode (uv sync --extra batch → xai_sdk)
#   ./run.sh --discover-models     # query GET /v1/models at runtime instead of static models.toml
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
DISCOVER_MODELS=0
RUN_LOCAL=0            # --local or --all: run the local arm-3 path
SKIP_XAI=0             # --local: skip the xAI path entirely
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
    --smoke)            SMOKE=1; shift ;;
    --check-only)       CHECK_ONLY=1; shift ;;
    --discover-models)  DISCOVER_MODELS=1; shift ;;
    --all)              RUN_LOCAL=1; shift ;;
    --local)            RUN_LOCAL=1; SKIP_XAI=1; shift ;;
    --)             shift; PASSTHRU=("$@"); break ;;
    -h|--help)      awk 'NR>1 && /^#/{sub(/^# ?/,""); print; next} NR>1{exit}' "$HERE/run.sh"; exit 0 ;;
    *)              echo "run.sh: unknown arg '$1' (use --help, or '--' to pass through)" >&2; exit 2 ;;
  esac
done

# Validate --mode early (grok.cli only accepts live|batch) so an invalid value fails clearly
# now — not later, after the setup + self-test have already run.
if [[ "$MODE" != "live" && "$MODE" != "batch" ]]; then
  echo "run.sh: --mode must be 'live' or 'batch', got '$MODE'" >&2; exit 2
fi

# --- resolve the Python runner: uv (preferred) else system python3 ----------------------
# uv gives us the project venv + the console script + extras. Without uv, the LIVE path and
# self-test still work on plain python3 (stdlib only) — but --batch needs uv (the xai_sdk extra).
if command -v uv >/dev/null 2>&1; then
  PY=(uv run python)
  RUNNER=(uv run python -m grok.cli)
  echo ">> syncing deps with uv ..."
  if [[ "$MODE" == "batch" ]]; then
    uv sync --extra batch
  else
    uv sync
  fi
elif command -v python3 >/dev/null 2>&1; then
  PY=(python3)
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
DISCOVER=()
[[ "$DISCOVER_MODELS" -eq 1 ]] && DISCOVER=(--discover-models)

if [[ "$SKIP_XAI" -eq 0 ]]; then
  echo ">> resolved models (cheapest-first) ..."
  "${RUNNER[@]}" "${DISCOVER[@]}" --list-models
fi

[[ "$CHECK_ONLY" -eq 1 ]] && { echo ">> --check-only: done (offline checks passed; no spend)."; exit 0; }

# --- 3. xAI path: need a key for any priced run; stop gracefully (not an error) if absent
if [[ "$SKIP_XAI" -eq 0 ]]; then
  if [[ -z "${XAI_API_KEY:-}" && -z "${GROK_API_KEY:-}" ]]; then
    cat >&2 <<'EOF'
>> Offline checks passed. No xAI API key in the environment, so stopping before any priced run.
   To run the live experiment (capped):
       export XAI_API_KEY=...        # or GROK_API_KEY=...
       ./run.sh                      # re-run; it will do the capped sweep
   To also run local arm-3 (after xAI):
       ./run.sh --all
   To run local arm-3 only (no xAI):
       ./run.sh --local
EOF
    if [[ "$RUN_LOCAL" -eq 0 ]]; then
      exit 0
    fi
    echo ">> (continuing to local arm-3 path because --all was given)"
  else
    # --- 4. (optional) a cheap single-model smoke first ---------------------------------
    SUBSET=()
    [[ -n "$MODELS" ]] && SUBSET=(--models "$MODELS")
    if [[ "$SMOKE" -eq 1 ]]; then
      echo ">> smoke: cheapest model, \$2 cap ..."
      "${RUNNER[@]}" "${DISCOVER[@]}" --models grok-build-0.1 --mode "$MODE" "${ABLATION[@]}" --max-usd 2 "${PASSTHRU[@]}"
    fi

    # --- 5. the real, capped xAI run ----------------------------------------------------
    echo ">> run: mode=$MODE  cap=\$$MAX_USD  ablation=${ABLATION[*]:-off}  models=${MODELS:-all-cheapest-first}"
    "${RUNNER[@]}" "${DISCOVER[@]}" --mode "$MODE" "${ABLATION[@]}" --max-usd "$MAX_USD" "${SUBSET[@]}" "${PASSTHRU[@]}"

    echo ">> done (xAI). Reports written under: $HERE/reports/  (send me the newest *-report.json)"
  fi
fi

# --- 6. local arm-3 path (--all / --local) ----------------------------------------------
# Runs only when --all or --local is given.  Two sub-steps:
#   a) setup_local_llm.py — idempotent; always runs (even without GPU) to install deps and
#      write local/.env.  A missing GPU never hard-fails this script (CPU fallback allowed).
#   b) run_arm3_local.py  — runs the grammar-constrained generation.  When ConstrainedBackend
#      is unavailable (no llama_cpp / no MYC_ARM3_MODEL) it reports SKIP per task and exits 0
#      — never a fabricated result (G2).
if [[ "$RUN_LOCAL" -eq 1 ]]; then
  echo ""
  echo ">> ── local arm-3 (grammar-constrained decoding) ──────────────────────────────────"

  # a) idempotent setup — install llama-cpp-python, download model, write .env
  echo ">> local/setup_local_llm.py (idempotent) ..."
  if ! "${PY[@]}" "$HERE/local/setup_local_llm.py"; then
    # setup_local_llm.py exits 1 only on a genuinely fatal error (bad Python version or
    # install/download failure after retry).  In that case we report the skip explicitly
    # and do not run inference (G2: never silent; never fabricate a result).
    echo "!! local setup FAILED — skipping arm-3 inference." \
         "(Re-run 'python local/setup_local_llm.py' for details.)" >&2
    if [[ "$SKIP_XAI" -eq 1 ]]; then
      # --local only: a setup failure is the only thing we ran, so exit non-zero
      exit 1
    fi
    # --all: xAI already ran; treat local failure as a non-fatal skip so the
    # script exits 0 (the xAI work succeeded).
    echo ">> [SKIP] local arm-3 inference skipped due to setup failure."
  else
    # b) inference — run_arm3_local.py exits 0 even when all tasks SKIP (G2)
    echo ">> local/run_arm3_local.py ..."
    if ! "${PY[@]}" "$HERE/local/run_arm3_local.py"; then
      echo "!! run_arm3_local.py exited non-zero — see output above." >&2
      if [[ "$SKIP_XAI" -eq 1 ]]; then
        exit 1
      fi
      echo ">> [SKIP] local arm-3 inference reported an error; xAI results are unaffected."
    fi
  fi

  echo ">> ── local arm-3 done ─────────────────────────────────────────────────────────────"
fi
