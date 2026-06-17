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

### Step 3 — Download a small GGUF model

Use a small quantised model for initial testing (≤4 GB, Q4_K_M or smaller).
Example using `wget` (replace URL with a model you have rights to use):

```sh
# Example: TinyLlama-1.1B-Chat Q4_K_M (~660 MB)
# Find current URLs at https://huggingface.co — search "gguf tinyllama q4_k_m"
wget -O tinyllama.gguf "https://huggingface.co/<org>/<repo>/resolve/main/<filename>.gguf"
```

**FLAG:** Hugging Face direct download URLs require authentication for some models.
Use `huggingface-cli download` (pip) or the HF web UI to get a direct link, or use
a model explicitly published for anonymous download.

**FLAG on storage:** GGUF files are large. Ensure you have enough space:
`df -h $HOME` before downloading.

### Step 4 — Run the harness (real mode)

```sh
cd $HOME/llama.cpp
python3 $HOME/mycelium/tools/llm-harness/harness.py \
  --llama-cli ./build/bin/llama-cli \
  --model /path/to/tinyllama.gguf
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
