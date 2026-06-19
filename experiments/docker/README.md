# KC-2 in a container (desktop GPU — Podman or Docker, from WSL2 or native Linux)

## Ubuntu WSL — quickstart

```sh
# 0) Windows (once): install the NVIDIA *Windows* driver (R495+). Do NOT install a driver inside WSL.
# 1) Ubuntu WSL — engine + repo (Podman = rootless, no daemon; preferred in WSL):
sudo apt update && sudo apt install -y podman git
git clone https://github.com/tzervas/mycelium && cd mycelium
# 2) One-time GPU wiring (installs the toolkit, generates the Podman CDI spec, verifies):
INSTALL=1 bash experiments/docker/gpu-setup.sh
# 3) Build image (first run only) + run the matrix on the GPU; outputs → experiments/results/:
bash experiments/docker/run.sh
```

- **Docker instead:** `CONTAINER_ENGINE=docker bash experiments/docker/{gpu-setup,run}.sh` (start the
  daemon first: `sudo service docker start`).
- The `libnvidia-sandboxutils.so.1 not found` warning during step 2 is **expected on WSL and
  harmless** — the CDI spec still generates and the GPU check passes.
- Desktop auto-runs `{0.5B, 1.5B, 7B}` (the 1.5B mobile cap is lifted here). Tune with
  `MODELS="…" SEEDS=42,123 MAXITERS=4 bash experiments/docker/run.sh`.
- Re-run `gpu-setup.sh` after a GPU driver update (regenerates the Podman CDI spec). Commit results
  from the host. Details below.

---

Run the KC-2 experiment on a desktop GPU **without touching the host toolchain** — Python, Rust
(`myc-check`), and a CUDA `llama-server` all live inside the image. The repo is bind-mounted, so
**every report/log/JSONL lands on the host** under `experiments/results/<model>-<primer>/`; do all
your git from the host.

**Podman is preferred** (rootless-friendly — bind-mounted outputs land owned by *you*, not root);
Docker works too. The runner auto-detects the engine. Mobile caps at the 1.5B model; the desktop
path here lifts that and adds the **7B** (and you can go bigger — `14b`/`32b`).

> **Honesty note.** The CUDA/GPU/Podman/Docker path is **not** exercised in the project's
> design-phase sandbox (no GPU/engine there). The Python pipeline, the CPU path, and the
> resilience/teardown logic are tested. The container scripts are **syntax-checked and their GPU
> commands are vetted against NVIDIA's official docs** (see *Sources*), but the GPU build + offload
> are verified by *you* via the `nvidia-smi` steps below before a long run.

## TL;DR — two commands from the repo root

```sh
bash experiments/docker/gpu-setup.sh   # ONCE: verify + configure GPU access (uses sudo where needed)
bash experiments/docker/run.sh         # build image + run {0.5B,1.5B,7B} × {minimal,examples} on the GPU
```

Overrides: `MODELS="qwen2.5-coder-7b qwen2.5-coder-14b" SEEDS=42,123,7 MAXITERS=4 bash experiments/docker/run.sh`,
or `CONTAINER_ENGINE=docker …` to force the engine. `run.sh` needs no interaction once started —
background it with `nohup … &`, tmux, or screen for a hands-off run.

## Prerequisites (WSL2 host)
1. **NVIDIA *Windows* driver** (R495+) from nvidia.com. On WSL2 this is the **only** driver you
   need — **do not install any Linux display driver inside WSL** (it stubs `libcuda.so` and puts
   `nvidia-smi` under `/usr/lib/wsl/lib`).
2. **Podman** (`sudo apt install podman`) *or* **Docker** in the WSL distro.
3. **NVIDIA Container Toolkit** in the WSL distro — `gpu-setup.sh` checks for it and prints the
   exact apt commands (run that script with `INSTALL=1` to install it for you).
4. Clone the repo **inside WSL2** (e.g. `~/dev/mycelium`), not under `C:\…` — a Windows-path bind
   mount is much slower for the Rust/llama.cpp build.

## What `gpu-setup.sh` does (vetted vs NVIDIA docs)
- Confirms `nvidia-smi` sees the GPU on the host (or `/usr/lib/wsl/lib/nvidia-smi`).
- Ensures the NVIDIA Container Toolkit is installed (prints the apt block; installs it with `INSTALL=1`).
- **Podman:** generates the **CDI** spec — `sudo nvidia-ctk cdi generate --output=/etc/cdi/nvidia.yaml`
  — which podman consumes via `--device nvidia.com/gpu=all --security-opt=label=disable`.
  *Re-run after any GPU driver update* (WSL usually lacks the systemd auto-refresh).
- **Docker:** `sudo nvidia-ctk runtime configure --runtime=docker` + restart, enabling `--gpus all`.
- Verifies a throwaway CUDA container can run `nvidia-smi -L`.

## What `run.sh` does
Detects the engine, builds the image (the CUDA `llama.cpp` build is the slow one-time step, then
cached), re-checks GPU visibility, and runs the matrix. `--serve` launches the CUDA `llama-server`
inside the container and `detect_gpu` (via `nvidia-smi`) sets `--n-gpu-layers` automatically, so no
GPU flag is needed in the run command itself. Reports land on the host; the server is torn down at
the end of each combo.

## Where the outputs are
On the **host**: `experiments/results/<model>-<primer>/<utc>-seed<N>.json` + `.summary.txt`, the
per-run `.attempts.jsonl` checkpoint, `index.json`, and the suite/server `.log`s. Review and commit
them from the host. The first run also builds `myc-check` into the host `target/debug/` (gitignored).

**Reproducibility:** the image pins the Rust toolchain + uv version and records the exact llama.cpp
commit it built — `<engine> run --rm mycelium-kc2-cuda cat /opt/llama-cpp.commit` — so a result set
traces to the precise code that produced it.

## CPU-only box
Build without CUDA — `LLAMA_CUDA=OFF` (Dockerfile build arg) — and run only the small `MODELS`
(`qwen2.5-coder-0.5b qwen2.5-coder-1.5b`). The 7B on CPU is impractical.

## Docker Compose (Docker only — optional)
`docker-compose.yml` is a Docker convenience (its `gpus: all` key is **Docker-specific**; Podman
does **not** use it — use `run.sh`, which wires CDI). Manual single-model run:

```sh
docker compose -f experiments/docker/docker-compose.yml build
docker compose -f experiments/docker/docker-compose.yml run --rm kc2 nvidia-smi          # verify GPU
docker compose -f experiments/docker/docker-compose.yml run --rm kc2 \
  uv run python -m mycelium_experiments.kc2 --serve --model-id qwen2.5-coder-7b --seeds 42,123,7
```

## Sources (vetted)
- NVIDIA Container Toolkit — Installation: <https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/latest/install-guide.html>
- NVIDIA Container Toolkit — CDI support (podman `--device nvidia.com/gpu=all`, `--security-opt=label=disable`, `nvidia-ctk cdi generate/list`): <https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/latest/cdi-support.html>
- CUDA on WSL User Guide (Windows-driver-only, `/usr/lib/wsl/lib`, container requirements): <https://docs.nvidia.com/cuda/wsl-user-guide/index.html>
