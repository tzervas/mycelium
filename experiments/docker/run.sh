#!/usr/bin/env bash
# Single-command, fire-and-forget KC-2 run in the CUDA container — Podman OR Docker, compose-free.
#
# From the repo root (inside WSL2 or native Linux), one command builds the image and runs the
# model × primer matrix on the GPU:
#
#   bash experiments/docker/run.sh
#   MODELS="qwen2.5-coder-7b qwen2.5-coder-14b" SEEDS=42,123,7 MAXITERS=4 bash experiments/docker/run.sh
#   CONTAINER_ENGINE=docker bash experiments/docker/run.sh        # force the engine
#
# Engine: Podman is preferred (rootless-friendly; files land owned by YOU), Docker is the fallback.
# GPU wiring differs by engine and is handled below (Podman = CDI `--device nvidia.com/gpu=all`;
# Docker = `--gpus all`). If the GPU check fails, run `bash experiments/docker/gpu-setup.sh` once.
#
# Outputs land on the HOST under experiments/results/<model>-<primer>/, ready to `git add`.
# No interaction once started; background with `nohup … &`, tmux, or screen for hands-off.
#
# HONESTY: the CUDA/GPU/Podman/Docker path is NOT exercisable in this repo's design-phase sandbox.
# The commands are vetted against NVIDIA's container-toolkit + WSL docs (see README "Sources") and
# the script is syntax-checked; the GPU build + offload are verified by YOU via the nvidia-smi step.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
IMAGE="${IMAGE:-mycelium-kc2-cuda}"

# Desktop GPU (e.g. RTX 5080, 16 GB) runs the 7B coder comfortably. Override via env.
export MODELS="${MODELS:-qwen2.5-coder-0.5b qwen2.5-coder-1.5b qwen2.5-coder-7b}"
export SEEDS="${SEEDS:-42}"
export MAXITERS="${MAXITERS:-3}"
# This is the desktop path → lift the 1.5B mobile cap enforced by run-kc2-matrix.sh.
export KC2_ALLOW_LARGE=1

# --- pick the engine (Podman preferred) -------------------------------------------------
ENGINE="${CONTAINER_ENGINE:-}"
if [[ -z "$ENGINE" ]]; then
  if command -v podman >/dev/null 2>&1; then ENGINE=podman
  elif command -v docker >/dev/null 2>&1; then ENGINE=docker
  else echo "ERROR: neither podman nor docker found on PATH." >&2; exit 1; fi
fi
echo "== engine: $ENGINE =="

# --- per-engine GPU flags (vetted: NVIDIA container-toolkit CDI / Docker runtime) --------
if [[ "$ENGINE" == podman ]]; then
  GPU_ARGS=(--device nvidia.com/gpu=all --security-opt=label=disable)
else
  GPU_ARGS=(--gpus all)
fi

# --- build (one-time-ish; CUDA llama.cpp build is the slow part, then cached) ------------
# The Dockerfile COPYs nothing (repo is bind-mounted at runtime), so the build context is just
# this dir → a fast upload. Podman reads the same Dockerfile as Docker.
echo "== [1/3] build image '$IMAGE' (cached after the first CUDA build) =="
if ! "$ENGINE" build -t "$IMAGE" -f "$SCRIPT_DIR/Dockerfile" "$SCRIPT_DIR"; then
  echo "ERROR: image build failed." >&2; exit 1
fi

# --- verify the GPU is visible in a container -------------------------------------------
echo "== [2/3] verify GPU visibility in a container =="
if "$ENGINE" run --rm "${GPU_ARGS[@]}" "$IMAGE" nvidia-smi -L; then
  echo "GPU OK — llama-server will offload to it (--serve auto-sets --n-gpu-layers)."
else
  echo "WARN: no GPU visible to $ENGINE. Run:  bash experiments/docker/gpu-setup.sh" >&2
  echo "      (configures CDI for podman / the nvidia runtime for docker, vetted vs NVIDIA docs)." >&2
  echo "      Falling back to CPU is slow; the 7B becomes impractical. Continuing in 10s (Ctrl-C to abort)…" >&2
  sleep 10 || true
fi

# --- run the matrix inside the container (fire-and-forget) ------------------------------
echo "== [3/3] run matrix: {${MODELS}} × {minimal, examples}  (seeds=$SEEDS, max-iters=$MAXITERS) =="
"$ENGINE" run --rm "${GPU_ARGS[@]}" \
  -v "$REPO_ROOT":/workspace \
  -v kc2-models:/models \
  -w /workspace/experiments \
  -e MYCELIUM_LLM_MODEL_DIR=/models \
  -e MODELS -e SEEDS -e MAXITERS -e KC2_ALLOW_LARGE \
  "$IMAGE" bash ./run-kc2-matrix.sh

echo "== done. Per-combo reports + index.json on the host under experiments/results/. =="
echo "   Review and commit from the host."
