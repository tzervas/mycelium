# Mycelium LLM-validation harness

A small, portable Python harness for validating llama.cpp (GGUF) model behaviour
against Mycelium's honesty and correctness properties. Designed to run under
Termux on Android (ARM/aarch64) with zero external Python dependencies.

> **Running the full test/experiment sequence?** This harness is **step 1** (validate
> the model + honesty gates). The end-to-end run order — doctor → these validations →
> the KC-2 LLM-leverage experiment (M-002) against the same local model — lives in
> [`experiments/README.md`](../../experiments/README.md).

## Contents

- [One command to run everything](#one-command-to-run-everything)
- [Honesty posture (non-negotiable)](#honesty-posture-non-negotiable)
- [Validations](#validations)
- [Quick start: mock mode](#quick-start-mock-mode-no-model-cicloud-safe)
- [Live xAI (Grok) runs — with the hard spend cap](#live-xai-grok-runs--with-the-hard-spend-cap)
- [Reports and logs](#reports-and-logs)
- [Adding a new validation](#adding-a-new-validation)
- Detail docs: [`MODEL-ACQUISITION.md`](MODEL-ACQUISITION.md) (fetching/verifying models,
  `--doctor`) · [`TERMUX-SETUP.md`](TERMUX-SETUP.md) (from-scratch Termux setup) ·
  [`GROK-HARNESS.md`](GROK-HARNESS.md) (the `grok/` xAI co-authoring sub-package)
- See also: [`local/README.md`](local/README.md) — the local arm-3 (grammar-constrained
  decoding) runbook

## One command to run everything

```sh
cd tools/llm-harness
./run.sh --all          # xAI ablation + local arm-3 — each skips gracefully if prerequisites absent
./run.sh --local        # local arm-3 only (no xAI key needed)
./run.sh                # xAI ablation only (today's default — same as before)
./run.sh --check-only   # offline self-test + model list only; no key, no spend, no GPU
```

**What runs and what gets skipped (skip-graceful, G2 — never silent):**

| Command | xAI path | local arm-3 |
|---|---|---|
| `./run.sh` | runs if `XAI_API_KEY`/`GROK_API_KEY` set; skips with message if absent | not run |
| `./run.sh --all` | same as above | always runs setup (GPU optional — CPU fallback); inference SKIPs only if the backend/model is still unavailable after setup |
| `./run.sh --local` | not run | always runs setup (GPU optional — CPU fallback); inference SKIPs only if the backend/model is still unavailable after setup |
| `./run.sh --check-only` | offline self-test + list-models only (no spend, no GPU) | not run |

**Prerequisites by path:**

| Path | Prerequisite | Skip behaviour when absent |
|---|---|---|
| xAI/Grok | `XAI_API_KEY` or `GROK_API_KEY` | prints a message, exits 0 |
| local arm-3 setup | Python >= 3.13 (load-bearing); `uv` recommended (warn-only — setup can still succeed if deps/model are already present) | wrong Python ⇒ exits 1; otherwise idempotent |
| local arm-3 inference | RECOMMENDED an NVIDIA GPU (CPU fallback works, just slower); the ~5 GB GGUF is auto-downloaded by setup | `run_arm3_local.py` reports SKIP per task only when `llama_cpp`/model is still unavailable after setup, exits 0 |

The local arm-3 setup (`local/setup_local_llm.py`) is **idempotent** — safe to re-run. It will
skip steps that are already done (llama-cpp-python installed, model already on disk).

---

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

## Live xAI (Grok) runs — with the hard spend cap

**One command (recommended):** `./run.sh` from `tools/llm-harness/` does the whole flow —
`uv sync` → the offline self-test (aborts if it fails, so you never spend on a broken
harness) → list models → the **capped** live run (only if a key is present; otherwise it
stops after the free checks and tells you how to set the key).

```sh
cd tools/llm-harness
./run.sh --check-only          # setup + self-test + list-models — no key, no spend
export XAI_API_KEY=…           # (or GROK_API_KEY=…)
./run.sh --smoke               # a $2 single-model smoke, then the full $10 cheapest-first sweep
# knobs: --max-usd N · --models a,b · --no-ablation · --batch · -- <extra grok.cli args>
```

The `grok/` sub-package drives real xAI runs (OpenAI-compatible `https://api.x.ai/v1`; key
from `$XAI_API_KEY` or `$GROK_API_KEY`). Models run **cheapest-first**; `--mode batch` uses
the xAI batch API (lower price). A **never-silent USD spend gate** (`--max-usd`, default
**$10.00**) guards the **total** spend across all models: a unit of work whose *estimated*
cost would breach the cap is **refused before it is sent**, and the run stops with a partial,
honestly-flagged report (G2). It is a **best-effort** gate, *not* a formal upper bound — the
token estimate is a heuristic and live completions are unbounded (no `max_tokens`), so a
single in-flight request can overrun; the gate biases high and stops **new** work early.

Equivalent explicit invocations (what `run.sh` calls under the hood):

```sh
cd tools/llm-harness

# Offline plumbing gate — no key, no network (17/17 checks).
uv run python -m grok.cli --self-test

# Live, batch-priced, cheapest-first, capped at $10 (the default):
XAI_API_KEY=…  uv run python -m grok.cli --mode batch --max-usd 10
# Sequential live mode + the M-381 retention-ratio ablation, capped:
XAI_API_KEY=…  uv run python -m grok.cli --mode live --ablation --max-usd 10
# Preview the resolved (cheapest-first) model list without spending:
uv run python -m grok.cli --list-models
```

Every measured number is **Empirical** (with its trial count); the model-quality / KC-2
retention **verdict stays open** until a real run lands (never pre-written — VR-5). The gate's
estimate is deliberately **biased high** (a conservative token figure), so it errs toward
stopping early — but because live completions are unbounded it cannot guarantee the final
billed total stays under the cap to the cent; set `--max-usd` with a little headroom.

For the model rubric, WSL setup, batch/ablation run modes, and the module map, see
[`GROK-HARNESS.md`](GROK-HARNESS.md).

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

For how the model itself is fetched/verified, or `--doctor` PATH troubleshooting, see
[`MODEL-ACQUISITION.md`](MODEL-ACQUISITION.md). For a from-scratch Termux setup, see
[`TERMUX-SETUP.md`](TERMUX-SETUP.md).

## Adding a new validation

Register a function with the `@validation(id, description)` decorator in `harness.py`.
It receives a `RunContext` and must return a `ValidationResult`. The function is
automatically included in the run loop. Handle the `ctx.mock` flag explicitly — never
omit a mock path, or the validation will attempt a real generation in CI.
