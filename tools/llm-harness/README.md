# Mycelium LLM-validation harness

A small, portable Python harness for validating llama.cpp (GGUF) model behaviour
against Mycelium's honesty and correctness properties. Designed to run under
Termux on Android (ARM/aarch64) with zero external Python dependencies.

## Honesty posture (non-negotiable)

These rules are enforced by the harness, not just documented:

- **NEVER-SILENT (G2, RFC-0013 I1):** a missing tool or model ⇒ explicit `SKIP`,
  never a false `PASS`. An exception is caught, logged, and turned into an explicit
  `FAIL` — never swallowed.
- **Guarantee lattice (VR-5):** `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`. Model-derived
  claims carry `Empirical` (validated by trials) or `Declared` (asserted) — **never
  `Proven` or `Exact`**. The harness checks this and raises an error on any violation.
- **DUAL PROJECTION (G11):** every run writes both a machine-readable JSON report and a
  human-readable text report. They are two renderers of one result, not two sources of truth.
- **`mock-PASS` ≠ `PASS`:** mock mode runs fixtures through the full plumbing but labels all
  model-dependent results `mock-PASS`. This is never evidence of real model quality.

## Validations

| ID | Name | Modes | Guarantee tag |
|---|---|---|---|
| V-01 | Deterministic-seed round-trip | real / mock-PASS | Empirical / Declared |
| V-02 | JSON-projection conformance (G11) | real / mock-PASS | Empirical / Declared |
| V-03 | Guarantee-tag honesty gate (VR-5) | **both** (pure logic) | none |
| V-04 | Latency and token-count report | real / mock-PASS | Empirical / Declared |

V-03 runs in both modes because it is pure logic — no model involved.

## Quick start: mock mode (no model, CI/cloud-safe)

```sh
python3 tools/llm-harness/harness.py --mock
```

Exits 0. Writes a timestamped JSON + text report under `tools/llm-harness/reports/`.
All model-dependent results are labelled `mock-PASS`, never `PASS`.

## Reports and logs

Each run writes three files under `tools/llm-harness/reports/`:

```
YYYYMMDDTHHMMSSZ-report.json   # machine projection (G11)
YYYYMMDDTHHMMSSZ-report.txt    # human projection (G11)
YYYYMMDDTHHMMSSZ.log           # full run trace (DEBUG)
```

The JSON report has top-level keys: `honesty_posture`, `summary`, `results`.
Each result has `id`, `status`, `guarantee_tag`, `message`, `detail`.

Exit code: `0` if no `FAIL` (PASS/SKIP/mock-PASS only); `1` if any `FAIL`.

## Model acquisition (idempotent — fetch only if missing)

The harness can fetch the model for you, so a phone can start it and walk away. It
is **idempotent**: a model already on disk is reused (a cheap presence check); only an
absent/invalid one is downloaded. Downloads are **resumable** (HTTP Range), so a
slow/interrupted phone transfer continues where it left off on the next run.

```sh
python3 tools/llm-harness/harness.py --list-models          # see the registry + cache dir
python3 tools/llm-harness/harness.py --ensure-model         # fetch the default, then run
python3 tools/llm-harness/harness.py --ensure-model --model-id qwen2.5-coder-3b
```

- **Default model:** `qwen2.5-coder-1.5b` (Qwen2.5-Coder-1.5B-Instruct, Q4_K_M, ~1 GB,
  Apache-2.0). Chosen for *this* harness's use case — code + **structured/JSON output** +
  instruction-following, which is exactly what V-02 (JSON projection) and V-03 (tag
  honesty) exercise, and the closest fit to Mycelium-surface generation.
- **Mobile tier** (phone/Termux/CPU; ungated, no auth needed): `qwen2.5-0.5b-instruct`
  (smoke), `qwen2.5-1.5b-instruct`, **`qwen2.5-coder-1.5b`** (default), `qwen2.5-3b-instruct`,
  `qwen2.5-coder-3b`. The 3B Qwen2.5 weights are Qwen-Research (non-commercial) — flagged
  in `--list-models`.
- **Desktop tier** (for your RTX 5080 / 3090Ti tonight): `qwen2.5-coder-7b` (~4.7 GB),
  `qwen2.5-coder-14b` (~9 GB; fits 16 GB / 24 GB), `qwen2.5-coder-32b` (~20 GB; 24 GB tight
  or offload). Same command, just `--model-id qwen2.5-coder-14b`.
- **Cache dir:** `$MYCELIUM_LLM_MODEL_DIR`, else `$XDG_CACHE_HOME/mycelium-llm-harness/models`,
  else `~/.cache/mycelium-llm-harness/models` — **outside the repo**, so models are never
  committed. Override with `--model-dir DIR`.
- **Other flags:** `--no-download` (presence-check only, never fetch); `--model-url URL`
  (fetch an arbitrary GGUF under any `--model-id` name); `--model PATH` (bypass the registry
  entirely with a local file you trust).

**Honesty (G2/VR-5).** Registry URLs/filenames are **best-effort** and may change upstream.
A download is verified by the **GGUF magic header** (`GGUF`) + a clean, complete transfer
before it is promoted from `*.part` to the final name — a 404/gated **HTML page can never
masquerade as a model**. A failed/partial fetch is an explicit, logged error and the run
**SKIPs** the model-dependent validations (never a false PASS). Re-running resumes the
partial download. The registry stores **no fabricated checksums** (it has none to verify
against); integrity rests on the GGUF magic + complete transfer, and you can always supply
a self-verified file via `--model`.

> **Note on gated models (Llama-3.2, Gemma, …).** These require a Hugging Face token and
> are deliberately **not** in the default registry (a token-less `urlopen` would get an
> HTML login page, which the GGUF guard rejects). Download them yourself with
> `huggingface-cli` and pass `--model PATH`, or add an entry with `--model-url`.

## How to run on Termux (Android, ARM/aarch64)

### Step 1 — Install base packages

```sh
pkg update && pkg upgrade
pkg install python git cmake clang ninja wget
```

**FLAG:** `clang` in Termux pulls in its bundled libstdc++. If cmake cannot find
it during the llama.cpp build, try also installing `binutils` and `libandroid-spawn`.

### Step 2 — Clone and build llama.cpp

```sh
git clone https://github.com/ggerganov/llama.cpp
cd llama.cpp
cmake -B build \
  -DCMAKE_BUILD_TYPE=Release \
  -DGGML_NATIVE=OFF \
  -DGGML_OPENMP=OFF
cmake --build build --config Release -j$(nproc) --target llama-cli
```

**FLAG on `GGML_NATIVE=OFF`:** On Termux/aarch64, `GGML_NATIVE=ON` (the default)
attempts to emit x86 SIMD intrinsics (AVX/SSE), which breaks the ARM build. Always
pass `GGML_NATIVE=OFF` explicitly. This was observed to be necessary as of llama.cpp
commit range circa 2024-2025; verify it is still the right flag for your checkout.

**FLAG on NEON/SVE acceleration:** llama.cpp auto-detects ARM NEON. If your device
supports SVE (Scalable Vector Extension, present on some Cortex-A55+/A76+), you may
see a meaningful speedup by adding `-DGGML_SVE=ON`. This is device-specific; the
harness does not depend on it.

**FLAG on memory:** Building llama.cpp with `-j$(nproc)` on a low-RAM device may OOM
the linker. If so, reduce to `-j2` or `-j1`.

The built binary will be at:

```sh
./build/bin/llama-cli
```

### Step 3 — Let the harness fetch the model (idempotent)

No manual download needed — the harness fetches a small, ungated GGUF on first run and
reuses it thereafter. Start it in the background and check back later:

```sh
# Default mobile model (Qwen2.5-Coder-1.5B, ~1 GB). Resumable; safe to re-run.
python3 $HOME/mycelium/tools/llm-harness/harness.py \
  --ensure-model \
  --llama-cli $HOME/llama.cpp/build/bin/llama-cli

# Background it on the phone (slow is fine), then read the report when you're home:
nohup python3 $HOME/mycelium/tools/llm-harness/harness.py --ensure-model \
  --llama-cli $HOME/llama.cpp/build/bin/llama-cli > $HOME/harness.out 2>&1 &
```

Pick a different size with `--model-id` (see `--list-models`). To pre-fetch the model
*before* llama.cpp finishes building, run `--ensure-model` on its own — it caches the
model and the validations simply `SKIP` until `llama-cli` exists.

**FLAG on storage:** GGUF files are large. Check space first: `df -h $HOME`. The cache
dir is `~/.cache/mycelium-llm-harness/models` unless you set `--model-dir`/`$MYCELIUM_LLM_MODEL_DIR`.

### Step 4 — (optional) point at an existing model or a server

If you already have a GGUF, skip the registry:

```sh
python3 $HOME/mycelium/tools/llm-harness/harness.py \
  --llama-cli $HOME/llama.cpp/build/bin/llama-cli \
  --model /path/to/your-model.gguf
```

Or against a running llama.cpp server:

```sh
# Start the server (in another Termux session)
./build/bin/llama-server -m /path/to/tinyllama.gguf --port 8080

# Run harness against it
python3 $HOME/mycelium/tools/llm-harness/harness.py \
  --server http://localhost:8080
```

### Step 5 — Read the report

```sh
# Human-readable projection
cat $HOME/mycelium/tools/llm-harness/reports/*-report.txt | tail -50

# Machine projection
cat $HOME/mycelium/tools/llm-harness/reports/*-report.json | python3 -m json.tool
```

## Adding a new validation

Register a function with the `@validation(id, description)` decorator in `harness.py`.
It receives a `RunContext` and must return a `ValidationResult`. The function is
automatically included in the run loop. Handle the `ctx.mock` flag explicitly — never
omit a mock path, or the validation will attempt a real generation in CI.

## Related issues

- `#97` / `#127` (M-330) — LLM API integration (backlogged; this harness de-risks it)
- `#3` (M-002) — LLM leverage harness (backlogged; needs LLM API)
