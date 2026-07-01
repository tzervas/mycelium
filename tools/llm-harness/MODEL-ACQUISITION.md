# Model acquisition, downloading, and `--doctor`

Detail doc for [`tools/llm-harness/README.md`](README.md) — how the harness fetches, verifies,
and diagnoses its local model + tooling. Skip here once the landing README's quickstart has you
running; come back when `--ensure-model` or `--doctor` needs more than the one-liner.

## Contents

- [Model acquisition (idempotent — fetch only if missing)](#model-acquisition-idempotent--fetch-only-if-missing)
- [Downloading models: stdlib downloader (default) + `$HF_TOKEN`; hf CLI is optional](#downloading-models-stdlib-downloader-default--hf_token-hf-cli-is-optional)
- [Troubleshooting: `--doctor` and PATH self-healing](#troubleshooting---doctor-and-path-self-healing)

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
- **Desktop tier** (a workstation-class NVIDIA GPU): `qwen2.5-coder-7b` (~4.7 GB),
  `qwen2.5-coder-14b` (~9 GB; fits 16 GB / 24 GB), `qwen2.5-coder-32b` (~20 GB; 24 GB tight
  or offload). Same command, just `--model-id qwen2.5-coder-14b`.
- **Cache dir:** `$MYCELIUM_LLM_MODEL_DIR`, else `$XDG_CACHE_HOME/mycelium-llm-harness/models`,
  else `~/.cache/mycelium-llm-harness/models` — **outside the repo**, so models are never
  committed. Override with `--model-dir DIR`.
- **Other flags:** `--no-download` (presence-check only, never fetch); `--model-url URL`
  (fetch an arbitrary GGUF under any `--model-id` name); `--model PATH` (bypass the registry
  entirely with a local file you trust).

## Downloading models: stdlib downloader (default) + `$HF_TOKEN`; hf CLI is optional

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
  `$PREFIX/bin` (Termux). An installed-but-unlinked `hf` (see the Termux note in
  [`TERMUX-SETUP.md`](TERMUX-SETUP.md)) is found and used, with a warning telling you the exact
  `export PATH=…` to add for your shell.
- **Install** is **opt-in only** (`--install-hf-cli` / `--setup-hf`): it installs the published
  `huggingface_hub[cli]` package via **`uv` / `pipx` / `pip`** — **never** `curl … | bash`
  (CONTRIBUTING.md supply-chain rule) — and warns first that the `hf-xet` build may fail on aarch64.
  You don't need it; prefer `$HF_TOKEN` + the stdlib downloader.
- **Auth** (when a CLI is present) runs `hf auth whoami`; if unauthenticated it prompts to log in, or
  use `--hf-token TOKEN` / `$HF_TOKEN`. Non-fatal — the default registry is public.
- **Flags:** `--hf-cli PATH` (explicit binary), `--no-hf-cli` (force the stdlib downloader),
  `--install-hf-cli`, `--hf-token TOKEN`, `--setup-hf`, `--model-sha256 HEX`, `-y`/`--yes`.

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

## Troubleshooting: `--doctor` and PATH self-healing

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

---
[Back to the llm-harness landing README](README.md)
