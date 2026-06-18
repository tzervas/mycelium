# KC-2 in a container (desktop GPU, e.g. RTX 5080)

Run the KC-2 experiment on your desktop **without touching the Windows toolchain** — Python,
Rust (`myc-check`), and a CUDA `llama-server` all live inside the image. The repo is
bind-mounted, so **every report/log/JSONL lands on the host** under `experiments/results/`;
do all your git from Windows as usual.

> **Honesty note.** The CUDA image is provided best-effort and is **not** validated in the
> project's design-phase sandbox (no GPU there). The Python pipeline, the CPU path, and the
> resilience/teardown logic are tested; the GPU build is verified by *you* with `nvidia-smi`
> below before a long run.

## Single command (fire-and-forget)
From the **repo root**, one command builds the image, verifies the GPU, and runs the full
**model × primer matrix** ({0.5B, 1.5B, 7B} × {minimal, examples}) with the GPU offloaded:
```sh
bash experiments/docker/run.sh
# overrides:
MODELS="qwen2.5-coder-7b qwen2.5-coder-14b" SEEDS=42,123,7 MAXITERS=4 bash experiments/docker/run.sh
```
It needs no interaction once started (background it with `nohup … &`, tmux, or screen for a
truly hands-off run). Every report lands on the host under `experiments/results/<model>-<primer>/`,
ready to review and `git add`. If the GPU isn't visible it warns and falls back to CPU (slow — the
7B becomes impractical; drop to the small `MODELS` for a CPU box). The steps below are the same
thing done manually, for when you want a single model or to debug the image.

## Prerequisites (Windows host)
1. **NVIDIA driver** for the 5080 (Game-Ready/Studio — includes the WSL2 CUDA driver).
2. **Docker Desktop** with the **WSL2 backend** (Settings → General → *Use the WSL 2 based
   engine*). Docker Desktop bundles GPU support; no separate Container Toolkit install needed.
3. Confirm GPU passthrough: `docker run --rm --gpus all nvidia/cuda:12.8.1-base-ubuntu24.04 nvidia-smi`
   should list the 5080. If it doesn't, fix this before anything else.

> **Tip:** clone the repo **inside WSL2** (e.g. `~/dev/mycelium`) rather than under `C:\…`.
> Bind-mounting a Windows path into Linux works but is much slower for the Rust build. Either
> way the outputs land in the repo dir and you git from there.

## Build (one-time; the CUDA `llama.cpp` build takes a while)
```sh
docker compose -f experiments/docker/docker-compose.yml build
```
CPU-only box? Build without CUDA: in `docker-compose.yml` set `build.args: {LLAMA_CUDA: "OFF"}`
and remove the `gpus: all` line.

## Verify the GPU is visible in the container
```sh
docker compose -f experiments/docker/docker-compose.yml run --rm kc2 nvidia-smi
```

## Fetch a model (persists in the `kc2-models` volume)
The 5080 (16 GB) comfortably runs the **7B coder** — far stronger than the phone's 0.5/1.5B:
```sh
docker compose -f experiments/docker/docker-compose.yml run --rm kc2 \
  uv run python ../tools/llm-harness/harness.py --ensure-model --model-id qwen2.5-coder-7b
```

## Run the sweep (GPU auto-detected → offloaded; same `--serve` command)
```sh
docker compose -f experiments/docker/docker-compose.yml run --rm kc2 \
  uv run python -m mycelium_experiments.kc2 --serve --model-id qwen2.5-coder-7b --seeds 42,123,7
```
`--serve` launches the CUDA `llama-server` inside the container, `detect_gpu` (via `nvidia-smi`)
sets `--n-gpu-layers` automatically, the suite runs, and the server is torn down at the end.
On a 5080 you can raise the budget — e.g. add `--n-predict 256` (the GPU is fast) — and drop the
phone-oriented `--limit`.

## Where the outputs are
On the **host**: `experiments/results/<utc>-seed<N>.json` + `.summary.txt`, the per-run
`.attempts.jsonl` checkpoint, `index.json`, and the suite/server `.log`s. Review and commit them
from Windows. The first run also builds `myc-check` into the host `target/debug/` (gitignored).
