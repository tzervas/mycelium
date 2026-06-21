# Local arm 3 runbook — M-381 grammar-constrained decoding

This directory contains idempotent Python automation to run **arm 3 (local
grammar-constrained decoding)** of the M-381 ablation on a developer workstation
(target: Ubuntu / WSL + NVIDIA RTX 5080).

Arm 3 cannot run via the xAI/Grok REST API (no GBNF support there).
It requires a local GGUF-capable backend: `llama-cpp-python` + a GGUF model.

---

## Quick start — single command

Run both the xAI ablation and local arm-3 together from `tools/llm-harness/`:

```sh
cd tools/llm-harness
./run.sh --all          # xAI sweep (if key) + local arm-3 setup + inference
./run.sh --local        # local arm-3 only — no xAI key needed
```

Both flags are **skip-graceful**: a missing GPU is fine (CPU fallback works, just slower) and the
model is auto-downloaded by setup, so `run_arm3_local.py` reports `SKIP` per task and exits 0 (G2:
never a fabricated result) **only when the backend/model is still unavailable after setup** (no
`llama_cpp` / no model). The one fatal case is Python < 3.13 (load-bearing) => setup exits 1; a
missing `uv` is warn-only (setup can still succeed if deps/model are already present).

See `tools/llm-harness/README.md` for the full flag reference and skip-behaviour table.

---

## Prerequisites

| Requirement | Notes |
|---|---|
| Python >= 3.13 | Install via `uv` or `pyenv` |
| `uv` | <https://docs.astral.sh/uv/> |
| NVIDIA driver + CUDA toolkit | Recommended; CPU fallback works but is slow |
| ~5 GB disk space | For the default 7B Q4_K_M GGUF model |
| ~5 GB VRAM | Qwen2.5-Coder-7B-Instruct Q4_K_M; CPU-offload if less |
| Mycelium repo + `cargo` | For `myc-check` scoring (see below) |

---

## Step 1: Install dependencies + download model

Run `setup_local_llm.py` from the `tools/llm-harness/` directory:

```bash
cd tools/llm-harness
python local/setup_local_llm.py
```

This script is **idempotent**: run it multiple times safely. Each step checks
first and acts only if needed. It will:

1. Check OS, Python version, and `uv` presence.
2. Detect your NVIDIA GPU via `nvidia-smi` (warns on missing GPU; CPU fallback is OK).
3. Install `llama-cpp-python` with CUDA support (skips if already installed with CUDA).
4. Download `qwen2.5-coder-7b-instruct-q4_k_m.gguf` to `~/.cache/mycelium-llm/`
   (skips if already present and size-verified).
5. Print the `export MYC_ARM3_MODEL=...` line and write `local/.env`.

### Dry-run first (optional)

```bash
python local/setup_local_llm.py --dry-run
```

Prints every command that would run; makes no changes.

### Offline self-check

```bash
python local/setup_local_llm.py --self-check
```

Exercises arg parsing, GPU probe parsing, model verification logic, GBNF loading,
and constant sanity checks. No GPU, no internet, no installation needed.

### Custom model

```bash
python local/setup_local_llm.py \
  --model-url https://huggingface.co/.../my-model.gguf \
  --model-path /data/models/my-model.gguf
```

---

## Step 2: Export MYC_ARM3_MODEL

After setup, set the environment variable so arm 3 can find the model:

```bash
export MYC_ARM3_MODEL=~/.cache/mycelium-llm/qwen2.5-coder-7b-instruct-q4_k_m.gguf
```

`setup_local_llm.py` writes this line to `local/.env`; `run_arm3_local.py` loads
it automatically. You can also add the export to your `~/.bashrc` or `~/.zshrc`.

---

## Step 3: Run arm 3

```bash
cd tools/llm-harness
python local/run_arm3_local.py
```

This runs arm 3 over all 8 gold tasks (`gold-compose-v1`) with seed 42 and prints
per-task pass@1 plus aggregate metrics.

### With multiple seeds (pass@k)

```bash
python local/run_arm3_local.py --seeds 42 1337 7 99
```

### Parallel execution (sharded across tasks/seeds)

```bash
python local/run_arm3_local.py --workers 2 --seeds 42 1337
```

**Warning:** each worker loads a separate model instance. With a 7B Q4_K_M model
(~4.5 GB VRAM), `--workers 2` needs ~9 GB VRAM. On a 16 GB RTX 5080, `--workers 2`
or `--workers 4` (with small models) is safe.

### Dry-run (probe backend, build prompts, skip inference)

```bash
python local/run_arm3_local.py --dry-run
```

### Offline self-check

```bash
python local/run_arm3_local.py --self-check
```

Verifies prompt building, GBNF loading, scoring math, and arg parsing without a
model or `cargo`. Safe to run in CI or any environment.

### JSON output

```bash
python local/run_arm3_local.py --output reports/arm3-local.json
```

---

## RTX 5080 / Blackwell CUDA caveat

The NVIDIA RTX 5080 is a **Blackwell** GPU (compute capability sm_120 / sm_12x).
Standard pre-built `llama-cpp-python` wheels may not include sm_120 support yet.

`setup_local_llm.py` will detect the RTX 5080 and print a warning. If inference
silently falls back to CPU despite a CUDA build:

```bash
# Rebuild from source with explicit Blackwell target:
CMAKE_ARGS="-DGGML_CUDA=on -DCMAKE_CUDA_ARCHITECTURES=120" \
  uv pip install llama-cpp-python --no-binary :all: --upgrade
```

This requires the CUDA toolkit to be installed (`nvcc` on PATH).
See: <https://github.com/ggerganov/llama.cpp/discussions/7693>

---

## CPU fallback

If no NVIDIA GPU is detected (or `llama-cpp-python` is CPU-only), arm 3 runs on
CPU. A 7B Q4_K_M model will generate at ~2-10 tokens/second on a modern CPU —
expect ~1-5 minutes per task. All `--self-check` and `--dry-run` modes work
immediately with no GPU.

---

## Scoring: myc-check dependency

`run_arm3_local.py` scores generated `.myc` programs via `myc-check` (requires
`cargo` and a built Mycelium workspace). If `cargo` is absent or `myc-check`
fails to build, scoring is reported as `SKIP` — never a false PASS (G2).

The scorer locates the repo root automatically (walks up from `run_arm3_local.py`
looking for `justfile` or `.git`). Override with `--repo-root /path/to/mycelium`.

---

## File index

| File | Purpose |
|---|---|
| `setup_local_llm.py` | Idempotent setup: install + download + env |
| `run_arm3_local.py` | Arm 3 driver: prompt + decode + score + report |
| `.env` | Written by setup; auto-loaded by run script (gitignored) |
| `README.md` | This file |
