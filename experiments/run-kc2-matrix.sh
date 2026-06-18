#!/usr/bin/env bash
# KC-2 experiment matrix: {0.5B, 1.5B} coder models × {minimal, examples} primers, run in
# sequence and UNATTENDED. Each combo prefetches its model robustly, runs with --serve
# (auto-managed GPU/CPU server), and writes its own report set under results/<model>-<primer>/.
#
#   cd experiments && ./run-kc2-matrix.sh
#   MAXITERS=3 SEEDS=42,123 ./run-kc2-matrix.sh        # override budget / seeds
#   MODELS="qwen2.5-coder-0.5b" ./run-kc2-matrix.sh    # a single model
#
# Per-generation timeout refreshes every attempt; per-task n_predict is auto-sized. A run
# that is interrupted still writes a partial report, and the matrix continues to the next.
set -uo pipefail
cd "$(dirname "$0")"                      # → experiments/
export PYTHONPATH=.

HARNESS="../tools/llm-harness/harness.py"
read -r -a MODELS <<< "${MODELS:-qwen2.5-coder-0.5b qwen2.5-coder-1.5b}"
PRIMERS=(minimal examples)
MAXITERS="${MAXITERS:-3}"
SEEDS="${SEEDS:-42}"

echo "== Prefetch models (robust, auto-resuming) =="
for m in "${MODELS[@]}"; do
  echo "--- ensure $m ---"
  python3 "$HARNESS" --ensure-model --model-id "$m" --download-retries 0 \
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
