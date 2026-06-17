# Mycelium LLM-validation harness

A small, portable Python harness for validating llama.cpp (GGUF) model behaviour
against Mycelium's honesty and correctness properties. Designed to run under
Termux on Android (ARM/aarch64) with zero external Python dependencies.

> **Running the full test/experiment sequence?** This harness is **step 1** (validate
> the model + honesty gates). The end-to-end run order — doctor → these validations →
> the KC-2 LLM-leverage experiment (M-002) against the same local model — lives in
> [`experiments/README.md`](../../experiments/README.md).

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

### Downloading models: stdlib downloader (default) + `$HF_TOKEN`; hf CLI is optional

The default download path is the **built-in stdlib downloader** (pure `urllib`, resumable via HTTP
Range). It needs no extra packages, so it just works on a fresh phone. For a **gated** repo, export a
token and the downloader sends it as a bearer header — no CLI required:

```sh
export HF_TOKEN=hf_xxxxxxxx        # only needed for gated repos; public registry needs nothing
python3 tools/llm-harness/harness.py --ensure-model
```

The **`hf` CLI is optional.** It is *not* auto-installed (by `--doctor` or otherwise) because it is a
Python package whose recent versions pull the native **`hf-xet`** dependency, which has no aarch64 wheel
and **fails to build under Termux** — the exact breakage that motivated moving off the Python-package
install path. If an `hf`/`huggingface-cli` is already on your `PATH`, `--ensure-model` will use it
(robust, resumable, auth-aware); otherwise the stdlib path is used and nothing breaks.

```sh
python3 tools/llm-harness/harness.py --ensure-model                    # stdlib path (or hf CLI if present)
python3 tools/llm-harness/harness.py --ensure-model --install-hf-cli   # OPT-IN install of the CLI (may fail to build hf-xet on aarch64)
```

- **Detection** searches `PATH`, then the dirs installers actually use — `~/.local/bin`,
  `$PREFIX/bin` (Termux). An installed-but-unlinked `hf` (see the Termux note below) is found
  and used, with a warning telling you the exact `export PATH=…` to add for your shell.
- **Install** is **opt-in only** (`--install-hf-cli` / `--setup-hf`): it installs the published
  `huggingface_hub[cli]` package via **`uv` / `pipx` / `pip`** — **never** `curl … | bash`
  (CONTRIBUTING.md supply-chain rule) — and warns first that the `hf-xet` build may fail on aarch64.
  You don't need it; prefer `$HF_TOKEN` + the stdlib downloader.
- **Auth** (when a CLI is present) runs `hf auth whoami`; if unauthenticated it prompts to log in, or
  use `--hf-token TOKEN` / `$HF_TOKEN`. Non-fatal — the default registry is public.
- **Flags:** `--hf-cli PATH` (explicit binary), `--no-hf-cli` (force the stdlib downloader),
  `--install-hf-cli`, `--hf-token TOKEN`, `--setup-hf`, `--model-sha256 HEX`, `-y`/`--yes`.

### Troubleshooting: `--doctor` and PATH self-healing

If `--ensure-model` / real mode can't find a tool that you *know* is installed, it's almost
always the **off-PATH trap** (a `pip --user`/`pipx`/`uv`/npm install whose bin dir was never
added to `PATH`; or a hand-built `llama.cpp` under `~/llama.cpp/build/bin`). The harness handles
this without you editing dotfiles:

```sh
python3 tools/llm-harness/harness.py --doctor             # diagnose AND heal: auto-install + fix PATH
python3 tools/llm-harness/harness.py --doctor --yes        # same, non-interactive (skip the prompts)
python3 tools/llm-harness/harness.py --doctor --check-only # read-only report (no installs, no PATH writes)
```

- **`--doctor` is self-healing.** It prints platform/PATH, installers, and the resolved state of
  **llama.cpp**, the **hf CLI** (+ auth), the **Claude Code CLI**, and the model cache — and then
  *fixes what it can*:
  - missing **llama.cpp** → installs it from the **OS package manager** (Termux `pkg install llama-cpp`;
    `brew install llama.cpp`) — repo-signed, no fragile source build, **never** curl|bash. Where no package
    exists it prints the vetted from-source / pinned-release steps instead of guessing;
  - missing **hf CLI** → **left alone (it is optional)**. The hf CLI is a Python package that drags in the
    native `hf-xet` build (no aarch64 wheel — it fails on Termux), so the doctor does **not** install it. The
    built-in stdlib downloader fetches the public registry, and `$HF_TOKEN` unlocks gated repos without it;
  - **Claude Code CLI** installed-but-unlinked → links it onto PATH; missing entirely → `npm install -g
    @anthropic-ai/claude-code` (on Termux it points npm's prefix at `$PREFIX` first so the link lands on PATH);
  - off-PATH binary → PATH healed in-process **and** persisted to your shell rc (healing implies `--fix-path`);
  - absent default model → offers to download it.

  Every mutation **prompts for consent unless `--yes`**; a non-interactive run without `--yes` declines
  safely (never-silent, G2). Use **`--check-only`** for the classic read-only report — diagnose and print
  the fix for each miss, but install nothing and touch no files. Wrong-arch/corrupt binaries (an
  `Exec format error`) are reported with the reinstall command rather than auto-"fixed" (arch can't be patched).
- **Discovery** for every binary searches `PATH` first, then the dirs installers/builds actually
  use: hf → scripts dir, `~/.local/bin`, pipx/uv venvs, `$PREFIX/bin`; llama.cpp →
  `~/llama.cpp/build/bin`, `$PREFIX/bin`, `$MYCELIUM_LLAMA_DIR`, shallow globs; claude → npm global
  bin (`npm config get prefix`), nvm/bun/volta/pnpm dirs, `$PREFIX/bin`. If `hf` has no console
  script anywhere but `huggingface_hub` is importable, it falls back to `python -m huggingface_hub…`.
- **Self-healing PATH:** a binary found off-`PATH` is used anyway (its dir is prepended to *this run's*
  `PATH` so child processes see it), with the exact `export PATH=…` printed. `--fix-path` (implied by
  `--doctor` unless `--check-only`) appends that line to your shell rc (idempotent; prompts unless
  `--yes`) so it sticks.
- **Cached model is reused automatically:** once a model is in the cache, real mode uses it with no
  `--ensure-model` and no hf round-trip — so after the model is downloaded you only need
  `llama.cpp`. If a model is already present, `--ensure-model` won't even set up the hf CLI.
- **Unlinked Claude Code CLI:** if the npm package is installed but `claude` isn't on `PATH`,
  `--doctor` reports it as *installed but not linked* and (in heal mode) links `cli.js` into a PATH
  bin dir for you (`--check-only` just prints the exact `ln -s …`/relink fix instead).

**Honesty (G2/VR-5).** Registry URLs/filenames are **best-effort** and may change upstream.
A download is verified by the **GGUF magic header** (`GGUF`) + a clean, complete transfer
before it is promoted from `*.part` to the final name — a 404/gated **HTML page can never
masquerade as a model**. A failed/partial fetch is an explicit, logged error and the run
**SKIPs** the model-dependent validations (never a false PASS). Re-running resumes the
partial download.

**Checksum gate.** When a **SHA-256** is known — pass `--model-sha256 HEX`, or pin one in the
registry entry — the freshly-downloaded file **must** match it before promotion; a mismatch is a
loud, explicit failure and the `*.part` is kept for inspection (supply-chain integrity, CONTRIBUTING.md).
The registry stores **no fabricated checksums** (the honesty rule forbids asserting a value we
haven't vetted); where none is pinned, integrity rests on the GGUF magic + complete transfer, and
you can always supply a self-verified file via `--model` or a vetted `--model-sha256`.

> **Note on gated models (Llama-3.2, Gemma, …).** These require a Hugging Face token and
> are deliberately **not** in the default registry (a token-less `urlopen` would get an
> HTML login page, which the GGUF guard rejects). Authenticate with `--setup-hf` (or
> `--hf-token`/`$HF_TOKEN`), then either add an entry with `--model-url`, or download with
> `hf download … --local-dir DIR` yourself and pass `--model PATH`.

## How to run on Termux (Android, ARM/aarch64)

### Step 1 — Install base packages

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

### Step 2 — (only if `pkg install llama-cpp` is unavailable) build from source

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
