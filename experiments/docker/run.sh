#!/usr/bin/env bash
# Single-command, fire-and-forget KC-2 run in the CUDA container.
#
# From the repo root, on a host with an NVIDIA GPU + Docker (Desktop/WSL2 or native):
#
#   bash experiments/docker/run.sh                       # 0.5B + 1.5B + 7B × {minimal, examples}
#   MODELS="qwen2.5-coder-7b qwen2.5-coder-14b" bash experiments/docker/run.sh
#   SEEDS=42,123,7 MAXITERS=4 bash experiments/docker/run.sh
#
# It (1) builds the image (idempotent — cached after the first slow CUDA build),
# (2) verifies the GPU is visible (warns + continues CPU-only if not),
# (3) runs the model × primer matrix inside the container with the GPU offloaded.
#
# Every report/log/JSONL lands on the HOST under experiments/results/<model>-<primer>/,
# ready to review and `git add` from the host. No interaction needed once it starts;
# background it with `nohup … &` or a tmux/screen session for a truly hands-off run.
#
# HONESTY: the CUDA path is NOT exercised in this repo's design-phase sandbox (no GPU there).
# The orchestration/logic below is syntax-checked; the GPU build + offload are verified by YOU
# via the nvidia-smi step on first run. CPU-only fallback: see experiments/docker/README.md.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE=(docker compose -f "$SCRIPT_DIR/docker-compose.yml")

# A desktop GPU (e.g. RTX 5080, 16 GB) runs the 7B coder comfortably — far stronger than the
# phone's 0.5/1.5B. Override any of these via the environment.
export MODELS="${MODELS:-qwen2.5-coder-0.5b qwen2.5-coder-1.5b qwen2.5-coder-7b}"
export SEEDS="${SEEDS:-42}"
export MAXITERS="${MAXITERS:-3}"

if ! command -v docker >/dev/null 2>&1; then
  echo "ERROR: docker not found on PATH. Install Docker (Desktop/WSL2 or native) first." >&2
  exit 1
fi

echo "== [1/3] build image (cached after the first CUDA build) =="
if ! "${COMPOSE[@]}" build; then
  echo "ERROR: image build failed." >&2
  exit 1
fi

echo "== [2/3] verify GPU is visible in the container =="
if "${COMPOSE[@]}" run --rm kc2 nvidia-smi -L; then
  echo "GPU OK — llama-server will offload to it (--serve auto-sets --n-gpu-layers)."
else
  echo "WARN: no GPU visible in the container — the run will fall back to CPU (slow," >&2
  echo "      and the 7B model may be impractical). Fix GPU passthrough, or set" >&2
  echo "      MODELS to just the small models. Continuing in 10s (Ctrl-C to abort)…" >&2
  sleep 10 || true
fi

echo "== [3/3] run matrix: {${MODELS}} × {minimal, examples} =="
echo "        seeds=${SEEDS} max-iters=${MAXITERS} — outputs → experiments/results/<model>-<primer>/"
"${COMPOSE[@]}" run --rm \
  -e MODELS -e SEEDS -e MAXITERS \
  kc2 bash ./run-kc2-matrix.sh

echo "== done. Per-combo reports + index.json on the host under experiments/results/. =="
echo "   Review and commit from the host (the image is generic; the repo is bind-mounted)."
