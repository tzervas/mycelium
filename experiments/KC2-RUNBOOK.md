# KC-2 runbook — steps 1 through 4

Detail doc for [`experiments/README.md`](README.md) — the full step-by-step for validating the
harness, running the KC-2 experiment for real, and capturing results. Read
[`README.md`](README.md) first for the run order overview and the current KC-2 status; come here
for the actual commands.

## Contents

- [1. Validate the model + honesty gates (`tools/llm-harness`)](#1-validate-the-model--honesty-gates-toolsllm-harness)
- [2. Experiment unit tests (no model needed)](#2-experiment-unit-tests-no-model-needed)
- [3. The KC-2 experiment, for real (M-002)](#3-the-kc-2-experiment-for-real-m-002)
- [4. Capture results](#4-capture-results)

## 1. Validate the model + honesty gates (`tools/llm-harness`)

Confirms the model loads and the harness's correctness/honesty properties hold
(determinism, JSON projection, the VR-5 guarantee-tag gate, latency). Auto-finds the
`llama` binary and the cached model — no flags:

```sh
python tools/llm-harness/harness.py            # real mode (READY) — writes reports/ + a log
python tools/llm-harness/harness.py --mock      # fixture mode — needs no model (CI/sanity)
```

Reports land in `tools/llm-harness/reports/<run-id>-report.{json,txt}` + `<run-id>.log`.

> **If the process dies with `[Process completed (signal 9)]`** that's the Android
> low-memory killer (SIGKILL) — almost always the **KV cache**: llama.cpp otherwise
> allocates context for the model's full trained window (Qwen2.5 = 32k). Both tools
> now **auto-size the context from available RAM** (`/proc/meminfo`) by default and log
> the choice; `--doctor` shows the detected RAM/swap + the context it would pick. If a
> phone still OOMs, force it smaller or drop to the smallest model tier:
>
> ```sh
> python tools/llm-harness/harness.py --ctx-size 512
> python tools/llm-harness/harness.py --ensure-model --model-id qwen2.5-0.5b-instruct
> ```
>
> **One-shot generation is enforced by default** (`-no-cnv --no-display-prompt`): without
> them recent llama-cli enters its interactive chat REPL — it generates, then waits at a
> `>` prompt until the subprocess **times out**, and echoes the prompt into stdout. If a
> *particular* build rejects those flags, drop them with `--llama-arg=` / use `--server`.
> CPU phones run ~1–2 tok/s, so if a generation times out, raise `--timeout SEC`.

## 2. Experiment unit tests (no model needed)

Exercises the KC-2 harness logic against fixtures (the well-posedness of every task,
the edit-to-fix loop, source extraction). Skips the arms whose toolchain is absent:

```sh
cd experiments
uv run pytest -q          # or: PYTHONPATH=. python3 -m pytest -q
```

`experiments/tests/test_kc2.py` itself is skipped by default (`MYC_RUN_KC2 != 1`) — see the
["Current status" note](README.md#current-status-kc-2-already-has-a-verdict) in the landing
README for why.

## 3. The KC-2 experiment, for real (M-002)

Generates programs with the local model in **two arms** — the Mycelium surface fragment
vs a Python-embedded DSL baseline — and measures syntactic validity, first-attempt
pass rate (the SC-5b number), and edit-to-fix iterations.

```sh
cd experiments

# RECOMMENDED: auto-manage a llama-server (model loads ONCE, no interactive REPL,
# picks a free port, tears the server down after). Mycelium arm only by default.
# Defaults are tuned light: 2 attempts/task (first + one edit-to-fix), n_predict 128:
PYTHONPATH=. python3 -m mycelium_experiments.kc2 --serve

# Lighter still for a first pass on a slow phone — just the first 4 tasks:
PYTHONPATH=. python3 -m mycelium_experiments.kc2 --serve --limit 4

# A SEQUENCE of seeds, unattended — one report each + an index.json:
PYTHONPATH=. python3 -m mycelium_experiments.kc2 --serve --seeds 42,123,7

# Or an already-running server, or the CLI backend (EOF-guarded against the REPL):
PYTHONPATH=. python3 -m mycelium_experiments.kc2 --server http://localhost:8080
PYTHONPATH=. python3 -m mycelium_experiments.kc2 --model PATH.gguf
```

**Primer A/B + model matrix.** The Mycelium *primer* (a grounded, leak-free syntax reference —
it contains **no** task answer) is the chief generator knob. Two variants live in `primers/`:
`mycelium-minimal.txt` (syntax only) and `mycelium-examples.txt` (+ two complete, valid,
*non-answer* worked programs to anchor a weak model). Run the full
{0.5B, 1.5B} × {minimal, examples} matrix unattended with:

```sh
./run-kc2-matrix.sh                 # writes results/<model>-<primer>/ for each combo
MAXITERS=3 SEEDS=42,123 ./run-kc2-matrix.sh
```

Override one run's primer directly with `--primer-mycelium primers/mycelium-examples.txt`.
**Mobile caps at the 1.5B model** — the matrix skips anything larger unless `KC2_ALLOW_LARGE=1`.
On a desktop GPU run the containerized path, which lifts the cap (adds 7B+) and handles GPU access
for **Podman or Docker** under WSL2/Linux: `bash docker/gpu-setup.sh` once, then `bash docker/run.sh`
(see [`docker/README.md`](docker/README.md)).

Reports land under `--results-dir` (default `experiments/results/`): per run a
`<utc>-<name>.json` + `.summary.txt`, plus a combined `index.json` and a suite `.log`.
Each report carries **per-attempt records** (generated source, checker verdict,
generation wall-time) and a `timing` block, and every attempt is also streamed to
`<run>.attempts.jsonl` — so an OOM-kill or outer timeout **loses nothing** and a backend
error mid-run still writes a `partial` report. `--out PATH` copies a single-seed report
to a fixed path.

> **Tuning for a glacial phone.** A 1.5B model decodes at ~0.3–0.7 tok/s on a phone CPU,
> so generation time dominates. The timeout is **per generation** and **refreshes every
> attempt** — there is no cumulative suite timeout — so a long suite completes as long as
> each *single* generation fits its budget. Levers: `--n-predict` (DEFAULT auto — each task
> uses a token budget sized to its own complexity; pass N to force a fixed cap), `--timeout`
> (raise it rather than let a slow but valid generation get cut off, default 600 s),
> `--max-iters` (attempts/task, default 2), `--limit N` (fewer tasks). For a real speedup, use
> the **0.5B coder** — it decodes ~2–3× faster than the 1.5B. Fetch it once, then it's picked up
> automatically (the experiment prefers the 0.5B when cached, else the 1.5B, else any `.gguf`):
>
> ```sh
> # Prefetch ahead of time, robustly (auto-resumes a dropped/slow download; 0 = keep
> # retrying until complete — ideal for a flaky phone link):
> python ../tools/llm-harness/harness.py --ensure-model --model-id qwen2.5-coder-0.5b --download-retries 0
> PYTHONPATH=. python3 -m mycelium_experiments.kc2 --serve     # now runs on the 0.5B
> # or pin it explicitly by id (no long path):
> PYTHONPATH=. python3 -m mycelium_experiments.kc2 --serve --model-id qwen2.5-coder-0.5b
> ```

**Desktop GPU (containerised).** To run on a CUDA desktop (e.g. an RTX-class NVIDIA GPU) without
touching the host toolchain — Python, Rust, and a CUDA `llama-server` all inside the image, with
reports written back to the host for git — see [`docker/README.md`](docker/README.md). Same
`--serve` command; GPU is auto-detected and offloaded, and you can use a much stronger model
(`--model-id qwen2.5-coder-7b`).

> **Why `--serve` beats the bare CLI.** The CLI reloads the model for *every* generation
> (~1.4 tok/s on a phone) and some builds ignore `-no-cnv` and drop into an interactive
> REPL. `--serve` loads the model once and uses `/completion` (clean, one-shot). The
> manual `llama-server … &` route fights port collisions ("couldn't bind … port 8080"
> when an old server lingers); `--serve` reuses a healthy server or picks a free port.

**Server teardown.** `--serve` **auto-tears-down** the server it launched once all reports
and logs are written. To keep it up for a follow-up run, add `--keep-server` (its URL is
logged; reuse with `--server URL`). To reap an **orphan** (e.g. from a manual launch that
lost the port race), either:

```sh
python -m mycelium_experiments.kc2 --stop-server          # all; or --port 8080 for one
../tools/llm-harness/llama-server-stop.sh                  # standalone, no Python needed
```

**Gentle RAM reclaim.** Before sizing the context, every run does a *non-destructive*
reclaim (`gc` + `malloc_trim` + `sync`, plus `drop_caches` **only if root**) so the freed
memory is available to the model + KV cache — logged with a before→after delta. It never
kills processes; on an unrooted phone the gain is modest (the kernel already counts
reclaimable cache), so the real lever is reaping orphan servers first (`--stop-server`).
Disable with `--no-reclaim`. Max free RAM for a heavy run:

```sh
python -m mycelium_experiments.kc2 --stop-server      # free an orphan's ~1GB first
python -m mycelium_experiments.kc2 --serve --seeds 42,123,7   # reclaim runs automatically
```

**The Mycelium arm needs `myc-check`** (parse + typecheck + signature). The checker
builds it on first use via cargo; on a phone that's heavy but works, or build it once:

```sh
cargo build -p mycelium-check --bin myc-check     # → target/debug/myc-check
# or point at an existing binary:  export MYC_CHECK=/path/to/myc-check
```

> **Termux / Android (ARM64) build notes.** Use the **Termux-packaged** Rust
> (`pkg install rust` → `cargo`/`rustc` under `$PREFIX/bin`), not a rustup toolchain
> (rustup's binaries aren't built for Termux). `cargo` compiles for a while and then a
> **build script fails at link** (`linking with cc failed`, for every crate). Read the
> `note:` line — the cause is one of:
>
> - **The C compiler is not actually clang** (note: `Unknown command '…/symbols.o'.
>   Try: cc help`). A personal script has taken a compiler name — `cc`/`clang`/`gcc`, or
>   even the **versioned `clang-NN`** itself (the symlink target). rustc links via `cc`,
>   so every build script fails. Tell the impostor from the real thing by **size/type**:
>   the real `clang-NN` is a multi-MB ELF; a 1–3 KB script is the impostor.
>
>   ```sh
>   file "$(command -v cc)"; ls -la "$PREFIX"/bin/clang-*   # tiny script == impostor
>   # if clang-NN is the real binary, just point Rust at it:
>   export CC="$PREFIX/bin/clang-NN" CXX="$PREFIX/bin/clang++-NN"
>   export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$PREFIX/bin/clang-NN"
>   # if clang-NN ITSELF is the impostor: save it, then restore the real compiler:
>   cp "$PREFIX/bin/clang-NN" "$PREFIX/bin/ccode"; pkg install --reinstall clang
>   ```
>
>   Never name a personal wrapper `cc`/`clang`/`gcc`/`clang-NN` — those are the compiler;
>   use a shell `alias` instead (aliases don't affect what `cargo` execs). Persisting the
>   env route: `~/.cargo/config.toml` → `[target.aarch64-linux-android]` /
>   `linker = "clang-NN"`.
> - **A missing library** (note: `unable to find library -landroid-spawn` or `-lXXX`).
>   `pkg install libandroid-spawn binutils` (Termux's libc lacks `posix_spawn`; the
>   patched rust links `-landroid-spawn`), or `pkg install` the package matching `-lXXX`.

If `myc-check` can't be built, the Mycelium arm **SKIPs with an explicit reason** (the
report records it) — it never reports a fake 0%.

### The baseline arm (opt-in; executes generated code)

The baseline arm **executes model-generated Python in-process**, so it is **off by
default** and SKIPs unless you opt in. Only run it inside a disposable sandbox
(container/VM — the dev-container is the intended home for this):

```sh
PYTHONPATH=. python3 -m mycelium_experiments.kc2 \
    --arms mycelium,baseline --allow-untrusted-baseline --out kc2-report.json
```

### Useful knobs

- `--model PATH.gguf` — pick a specific model (else the default cache is auto-found).
- `--max-iters N` — edit-to-fix budget (default 3).
- `--ctx-size N` — override the auto context size (auto = sized from free RAM).
- `--use-swap` — count ~half of free swap toward the context budget (slower if the KV
  cache pages out; lets the context grow on a RAM-tight phone with swap enabled).
- `--cpu-only` / `--n-gpu-layers N` — on a desktop with a CUDA/ROCm/Metal build, the
  context auto-offloads to a detected GPU; force CPU or set the layer count explicitly.
  (No effect on a phone's CPU-only `llama.cpp`.)
- `--primer-mycelium FILE` / `--primer-baseline FILE` — override the per-arm **primer**.
  The primer is *generator configuration* (a generic syntax cheatsheet, no task answers);
  it's the chief tuning knob and it affects the numbers — **record which primer a run
  used**, and keep the two arms comparably generous.
- `--llama-extra-arg=-no-cnv` (repeatable) — pass extra flags to the llama CLI.

Every run prints (and, with `--out`, writes a companion `*.summary.txt`) an **executive
summary**: per-arm ratings, first-attempt vs eventual pass, the edit-to-fix (G10) gain,
which tasks struggled, and — when both arms ran — the comparison gap. It is *descriptive
analysis with cues, not conclusions*; the decision is explicitly left to you (VR-5).

## 4. Capture results

The report is JSON (`--out`) + its `*.summary.txt`, plus the `tools/llm-harness/reports/`
artifacts from step 1. Commit them to your working branch so they can be evaluated:

```sh
git add kc2-report.json kc2-report.json.summary.txt tools/llm-harness/reports/
git commit -m "experiment(kc2): capture local-llama run results (M-002)"
git push
```

> **Honesty / VR-5:** every script reports *measured rates only*. The KC-2 **verdict**
> (proceed / reweight-to-human / fall-back-to-embedded-DSL) is a **maintainer-written
> analysis** of these numbers — none of these scripts pre-writes it. That verdict has
> already been recorded once (DN-09: **proceed**); see the
> ["Current status"](README.md#current-status-kc-2-already-has-a-verdict) note in the
> landing README before re-running this for a fresh conclusion.

---
[Back to the experiments landing README](README.md)
