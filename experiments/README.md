# Mycelium experiments — run order & runbook

UV-managed Python (ADR-007; Python ≥3.13). Home of the **KC-2 LLM-leverage experiment**
(M-002) and other measurement code. Not the kernel (that's Rust).

This is the **optimal order** to run the local-LLM tests + experiments on a device
(Termux/Android included), capture results, and commit them. Each step is honest:
a missing tool/model is an explicit **SKIP**, never a false pass (G2/VR-5).

---

## 0. Prerequisites — heal the environment once

The `tools/llm-harness` doctor installs/links what's needed and prints a bottom-line
**READY / NOT READY** verdict:

```sh
python tools/llm-harness/harness.py --doctor          # install + fix PATH (prompts; --yes to skip)
python tools/llm-harness/harness.py --doctor --check-only   # read-only report (safe on a phone)
```

You want it to end with `✓ READY — llama.cpp CLI + a local model are both present.`
If not, the verdict lists exactly what's missing and the one command to fix it.

---

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
> **If a real run looks wrong** (the JSON check fails with the prompt echoed back, or a
> run hangs): recent llama.cpp defaults to interactive *conversation* mode and may echo
> the prompt. Confirm `llama --help`, then pass the flags through:
> `--llama-arg=-no-cnv --llama-arg=--no-display-prompt` — or run a llama.cpp **server**
> and use `--server http://localhost:8080` (clean output).

---

## 2. Experiment unit tests (no model needed)

Exercises the KC-2 harness logic against fixtures (the well-posedness of every task,
the edit-to-fix loop, source extraction). Skips the arms whose toolchain is absent:

```sh
cd experiments
uv run pytest -q          # or: PYTHONPATH=. python3 -m pytest -q
```

---

## 3. The KC-2 experiment, for real (M-002)

Generates programs with the local model in **two arms** — the Mycelium surface fragment
vs a Python-embedded DSL baseline — and measures syntactic validity, first-attempt
pass rate (the SC-5b number), and edit-to-fix iterations.

```sh
cd experiments

# Mycelium arm only (default). Needs the myc-check binary (see below):
PYTHONPATH=. python3 -m mycelium_experiments.kc2 --out kc2-report.json

# Or against a llama.cpp server (cleaner generation output than the CLI):
PYTHONPATH=. python3 -m mycelium_experiments.kc2 --server http://localhost:8080 --out kc2-report.json
```

**The Mycelium arm needs `myc-check`** (parse + typecheck + signature). The checker
builds it on first use via cargo; on a phone that's heavy but works, or build it once:

```sh
cargo build -p mycelium-l1 --bin myc-check        # → target/debug/myc-check
# or point at an existing binary:  export MYC_CHECK=/path/to/myc-check
```

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

---

Every run prints (and, with `--out`, writes a companion `*.summary.txt`) an **executive
summary**: per-arm ratings, first-attempt vs eventual pass, the edit-to-fix (G10) gain,
which tasks struggled, and — when both arms ran — the comparison gap. It is *descriptive
analysis with cues, not conclusions*; the decision is explicitly left to you (VR-5).

## 4. Capture results

The report is JSON (`--out`) + its `*.summary.txt`, plus the `tools/llm-harness/reports/`
artifacts from step 1. Commit them to this branch so they can be evaluated:

```sh
git add kc2-report.json kc2-report.json.summary.txt tools/llm-harness/reports/
git commit -m "experiment(kc2): capture local-llama run results (M-002)"
git push
```

> **Honesty / VR-5:** every script reports *measured rates only*. The KC-2 **verdict**
> (proceed / reweight-to-human / fall-back-to-embedded-DSL) is a **maintainer-written
> analysis** of these numbers — none of these scripts pre-writes it.
