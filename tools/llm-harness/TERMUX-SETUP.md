# How to run on Termux (Android, ARM/aarch64)

Detail doc for [`tools/llm-harness/README.md`](README.md) — the full from-scratch setup on a
Termux phone. If you're on a desktop/WSL box, you don't need this page; the landing README's
quickstart is enough.

## Contents

- [Step 1 — Install base packages](#step-1--install-base-packages)
- [Step 2 — (only if `pkg install llama-cpp` is unavailable) build from source](#step-2--only-if-pkg-install-llama-cpp-is-unavailable-build-from-source)
- [Step 3 — Let the harness fetch the model (idempotent)](#step-3--let-the-harness-fetch-the-model-idempotent)
- [Step 4 — (optional) point at an existing model or a server](#step-4--optional-point-at-an-existing-model-or-a-server)
- [Step 5 — Read the report](#step-5--read-the-report)

## Step 1 — Install base packages

```sh
pkg update && pkg upgrade
pkg install python git              # Python drives the harness; git to clone
pkg install llama-cpp               # CLI lands in $PREFIX/bin (on PATH) — note: Termux
                                    # names it `llama` (not `llama-cli`); the harness accepts either
```

`pkg install llama-cpp` is the **easy path** on Termux — a repo-signed, prebuilt
binary, no source build. `--doctor` will run it for you (with consent). Only fall
back to a source build (Step 2) if the package is unavailable for your device, in
which case also install the toolchain:

```sh
pkg install cmake clang ninja wget
```

**FLAG:** `clang` in Termux pulls in its bundled libstdc++. If cmake cannot find
it during the llama.cpp build, try also installing `binutils` and `libandroid-spawn`.

**FLAG on Termux PATH (`hf`/`claude` "command not found").** `pip install --user` (and some
npm/pipx installs) drop their console scripts in `~/.local/bin` (or `$PREFIX/bin`) **without
adding that dir to `PATH`** — so `which hf` finds nothing even though the package installed,
and Termux's helper just suggests unrelated `pkg`s. Fix it once:

```sh
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc && source ~/.bashrc
which hf            # should now resolve
```

The harness works around this for `hf` specifically — it searches `~/.local/bin` and
`$PREFIX/bin` even when they're off `PATH`, and prints the exact `export` line to run. For
the Claude Code CLI (npm), the same fix applies to npm's global bin dir: check
`npm config get prefix` and add its `bin` to `PATH` (or `npm config set prefix "$PREFIX"`
so links land on Termux's existing `PATH`).

**Note: `pkg` ≡ `apt` on Termux.** `pkg` is Termux's thin wrapper over `apt` — both work
(`pkg install …` ≈ `apt install …`). The installers the harness uses for the hf CLI come from
there: `pkg install python` provides `pip`; `pkg install pipx` (or `pkg install uv`) gives the
isolated installers. Once one is present, `--setup-hf` / `--install-hf-cli` can take over.

## Step 2 — (only if `pkg install llama-cpp` is unavailable) build from source

Prefer the package in Step 1. Build from the official source only as a fallback:

```sh
git clone https://github.com/ggml-org/llama.cpp
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

## Step 3 — Let the harness fetch the model (idempotent)

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
model and the validations simply `SKIP` until `llama-cli` exists. See
[`MODEL-ACQUISITION.md`](MODEL-ACQUISITION.md) for the full registry + download mechanics.

**FLAG on storage:** GGUF files are large. Check space first: `df -h $HOME`. The cache
dir is `~/.cache/mycelium-llm-harness/models` unless you set `--model-dir`/`$MYCELIUM_LLM_MODEL_DIR`.

## Step 4 — (optional) point at an existing model or a server

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

## Step 5 — Read the report

```sh
# Human-readable projection
cat $HOME/mycelium/tools/llm-harness/reports/*-report.txt | tail -50

# Machine projection
cat $HOME/mycelium/tools/llm-harness/reports/*-report.json | python3 -m json.tool
```

---
[Back to the llm-harness landing README](README.md) · See also
[`tools/termux/README.md`](../termux/README.md) for the Claude-Code-on-Android bootstrap, and
[`experiments/KC2-RUNBOOK.md`](../../experiments/KC2-RUNBOOK.md) → "Termux / Android (ARM64) build
notes" for the separate `myc-check`/Rust build prerequisites.
