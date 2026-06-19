#!/usr/bin/env bash
# KC-2 experiment matrix: {0.5B, 1.5B} coder models × {minimal, examples} primers, run in
# sequence and UNATTENDED. Each combo prefetches its model robustly, runs with --serve
# (auto-managed GPU/CPU server), and writes its own report set under results/<model>-<primer>/.
#
#   cd experiments && ./run-kc2-matrix.sh
#   MAXITERS=3 SEEDS=42,123 ./run-kc2-matrix.sh        # override budget / seeds
#   MODELS="qwen2.5-coder-0.5b" ./run-kc2-matrix.sh    # a single model
#
# Mobile cap: models larger than 1.5B are skipped unless KC2_ALLOW_LARGE=1 (the desktop
# container path experiments/docker/run.sh sets it). Phones stay at 0.5B/1.5B.
#
# Per-generation timeout refreshes every attempt; per-task n_predict is auto-sized. A run
# that is interrupted still writes a partial report, and the matrix continues to the next.
set -uo pipefail
cd "$(dirname "$0")" || exit             # → experiments/
export PYTHONPATH=.

HARNESS="../tools/llm-harness/harness.py"
read -r -a MODELS <<< "${MODELS:-qwen2.5-coder-0.5b qwen2.5-coder-1.5b}"
PRIMERS=(minimal examples)
MAXITERS="${MAXITERS:-3}"
SEEDS="${SEEDS:-42}"

# Mobile caps at 1.5B; anything larger is desktop-only. The desktop container path
# (experiments/docker/run.sh) opts in with KC2_ALLOW_LARGE=1; on a phone the cap holds.
ALLOW_LARGE="${KC2_ALLOW_LARGE:-0}"
KEPT=()
for m in "${MODELS[@]}"; do
  size="${m##*-}"; size="${size%b}"   # qwen2.5-coder-7b → 7 ; -1.5b → 1.5
  if [[ "$size" =~ ^[0-9.]+$ ]] && awk "BEGIN{exit !($size > 1.5)}" && [[ "$ALLOW_LARGE" != 1 ]]; then
    echo "SKIP $m: models larger than 1.5B are desktop-only (set KC2_ALLOW_LARGE=1, or use" >&2
    echo "         experiments/docker/run.sh on a desktop GPU)." >&2
    continue
  fi
  KEPT+=("$m")
done
MODELS=("${KEPT[@]}")
if [[ ${#MODELS[@]} -eq 0 ]]; then
  echo "ERROR: no eligible models (all exceeded the 1.5B mobile cap)." >&2; exit 1
fi

echo "== Prefetch models (download only — NOT the harness validation suite) =="
for m in "${MODELS[@]}"; do
  echo "--- ensure $m ---"
  python3 "$HARNESS" --ensure-model --ensure-only --model-id "$m" --download-retries 0 \
    || echo "WARN: could not fetch $m; its combos will be skipped."
done

echo "== Matrix: ${MODELS[*]} × ${PRIMERS[*]}  (max-iters=$MAXITERS, seeds=$SEEDS) =="
for m in "${MODELS[@]}"; do
  for p in "${PRIMERS[@]}"; do
    echo "=== run: model=$m primer=$p ==="
    python3 -m mycelium_experiments.kc2 --serve --model-id "$m" \
      --primer-mycelium "primers/mycelium-$p.txt" \
      --seeds "$SEEDS" --max-iters "$MAXITERS" \
      --results-dir "results/$m-$p" \
      || echo "WARN: run $m/$p ended early; see results/$m-$p/."
  done
done

echo "== Done. Per-combo reports + index.json under results/<model>-<primer>/ =="
