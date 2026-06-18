#!/usr/bin/env bash
# One-time WSL2 (or native Linux) GPU preflight for the KC-2 container — Podman or Docker.
# Ensures a container can see the NVIDIA GPU, and configures access if it can't. Run ONCE:
#
#   bash experiments/docker/gpu-setup.sh           # check + configure (uses sudo where needed)
#   INSTALL=1 bash experiments/docker/gpu-setup.sh # also apt-install the toolkit if it is missing
#   CONTAINER_ENGINE=docker bash experiments/docker/gpu-setup.sh
#
# Commands are vetted against NVIDIA's official docs (see README "Sources"):
#   - WSL2 needs ONLY the Windows NVIDIA driver (R495+); never install a Linux driver in WSL.
#   - Podman accesses the GPU via CDI:  nvidia-ctk cdi generate  +  --device nvidia.com/gpu=all
#   - Docker uses the nvidia runtime:   nvidia-ctk runtime configure --runtime=docker  +  --gpus all
#
# HONESTY: not exercisable in this repo's design-phase sandbox (no GPU/WSL/engine). Syntax-checked;
# you are the one who runs it on the real host.
set -uo pipefail

CUDA_BASE="docker.io/nvidia/cuda:12.8.1-base-ubuntu24.04"

ENGINE="${CONTAINER_ENGINE:-}"
if [[ -z "$ENGINE" ]]; then
  if command -v podman >/dev/null 2>&1; then ENGINE=podman
  elif command -v docker >/dev/null 2>&1; then ENGINE=docker
  else echo "ERROR: neither podman nor docker found on PATH." >&2; exit 1; fi
fi
echo "== engine: $ENGINE =="

# 1) GPU visible to the host (WSL bare metal). On WSL the Windows driver stubs libcuda.so and puts
#    nvidia-smi under /usr/lib/wsl/lib (root often won't find it on PATH).
echo "== [1/4] host GPU (nvidia-smi) =="
if nvidia-smi -L 2>/dev/null || /usr/lib/wsl/lib/nvidia-smi -L 2>/dev/null; then
  echo "host GPU OK."
else
  echo "ERROR: nvidia-smi found no GPU on the host." >&2
  echo "  WSL2: install the NVIDIA *Windows* driver (R495+) from nvidia.com — and do NOT install" >&2
  echo "        any Linux display driver inside WSL. Then reopen WSL and retry." >&2
  exit 1
fi

# 2) NVIDIA Container Toolkit present (provides nvidia-ctk + the CDI/runtime hooks).
echo "== [2/4] NVIDIA Container Toolkit (nvidia-ctk) =="
if ! command -v nvidia-ctk >/dev/null 2>&1; then
  echo "nvidia-ctk not found. Install it on Ubuntu/Debian with (vetted vs NVIDIA install-guide):"
  INSTALL_CMDS=$(cat <<'EOS'
sudo apt-get update && sudo apt-get install -y --no-install-recommends ca-certificates curl gnupg2
curl -fsSL https://nvidia.github.io/libnvidia-container/gpgkey | sudo gpg --dearmor -o /usr/share/keyrings/nvidia-container-toolkit-keyring.gpg
curl -s -L https://nvidia.github.io/libnvidia-container/stable/deb/nvidia-container-toolkit.list | sed 's#deb https://#deb [signed-by=/usr/share/keyrings/nvidia-container-toolkit-keyring.gpg] https://#g' | sudo tee /etc/apt/sources.list.d/nvidia-container-toolkit.list
sudo apt-get update
sudo apt-get install -y nvidia-container-toolkit
EOS
)
  echo "$INSTALL_CMDS"
  if [[ "${INSTALL:-0}" == 1 ]]; then
    echo "-- INSTALL=1: running the above --"
    bash -c "$INSTALL_CMDS" || { echo "ERROR: toolkit install failed." >&2; exit 1; }
  else
    echo "Re-run with INSTALL=1 to execute these, then run this script again." >&2
    exit 1
  fi
fi

# 3) Configure GPU access for the chosen engine.
echo "== [3/4] configure $ENGINE for GPU access =="
if [[ "$ENGINE" == podman ]]; then
  # CDI: generate the device spec podman consumes via --device nvidia.com/gpu=all.
  # Regenerate this after any GPU driver update (WSL usually lacks the systemd auto-refresh).
  sudo nvidia-ctk cdi generate --output=/etc/cdi/nvidia.yaml || {
    echo "ERROR: 'nvidia-ctk cdi generate' failed." >&2; exit 1; }
  echo "-- CDI devices --"
  nvidia-ctk cdi list || true
else
  sudo nvidia-ctk runtime configure --runtime=docker || {
    echo "ERROR: 'nvidia-ctk runtime configure' failed." >&2; exit 1; }
  # WSL may run Docker with or without systemd.
  if command -v systemctl >/dev/null 2>&1 && systemctl is-system-running >/dev/null 2>&1; then
    sudo systemctl restart docker || true
  else
    sudo service docker restart 2>/dev/null || true
  fi
fi

# 4) Verify a container can see the GPU.
echo "== [4/4] verify GPU inside a container =="
if [[ "$ENGINE" == podman ]]; then
  GPU_ARGS=(--device nvidia.com/gpu=all --security-opt=label=disable)
else
  GPU_ARGS=(--gpus all)
fi
if "$ENGINE" run --rm "${GPU_ARGS[@]}" "$CUDA_BASE" nvidia-smi -L; then
  echo "SUCCESS: $ENGINE can see the GPU. Now run:  bash experiments/docker/run.sh"
else
  echo "ERROR: the container still can't see the GPU. Check the toolkit install + (podman) CDI spec," >&2
  echo "       and that 'nvidia-smi -L' works on the host. See README 'Sources'." >&2
  exit 1
fi
