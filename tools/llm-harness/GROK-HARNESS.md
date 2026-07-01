# Grok / xAI co-authoring harness (`grok/`) — M-330, M-331, M-381

Detail doc for [`tools/llm-harness/README.md`](README.md) — the internals of the `grok/`
sub-package: the model rubric, WSL setup, live/batch/ablation run modes, and the module map.
The landing README's ["Live xAI (Grok) runs"](README.md#live-xai-grok-runs--with-the-hard-spend-cap)
section covers the common case (`./run.sh`); read this page when you need the explicit
`grok.cli` invocations or you're extending the ablation.

## Contents

- [Honesty posture (the load-bearing rule)](#honesty-posture-the-load-bearing-rule)
- [The model rubric — `models.toml`](#the-model-rubric--modelstoml)
- [The green gate (offline, no key, no network)](#the-green-gate-offline-no-key-no-network)
- [Setup on a WSL host (M-331)](#setup-on-a-wsl-host-m-331)
- [Run it — live (sequential, rate-limit-aware)](#run-it--live-sequential-rate-limit-aware)
- [Run it — batch (cheaper, independent generations via `xai_sdk`)](#run-it--batch-cheaper-independent-generations-via-xai_sdk)
- [Run it — the M-381 retention-ratio ablation](#run-it--the-m-381-retention-ratio-ablation-research11-t117)
- [Reports](#reports)
- [What runs where (module map)](#what-runs-where-module-map)
- [Related issues](#related-issues)

The `grok/` package adds **xAI/Grok backends** to this harness for the
generate→feedback→fix co-authoring loop (M-330), a portable WSL-runnable runner
(M-331), and the M-381 retention-ratio ablation — **without** removing the local
llama backend (still reachable via `harness.py --server …` / `coauthor.py`).

It is **pure Python + `uv`** for live mode and the self-test; the native batch
backend needs the optional `xai_sdk`.

## Honesty posture (the load-bearing rule)

There is **no API key in this repo's CI/dev environment**, so the live experiment
**cannot be run here and its metrics are never fabricated**. What ships:

- a **runnable** harness (live + batch + ablation), and
- an offline, deterministic **`--self-test`** (mocked client, no network) that
  verifies the *plumbing*: model ordering, RPM/TPM pacing math, batch-vs-live cost
  accounting, scoring, the M-330 loop, the M-381 retention computation, and report
  emission. This is the **green gate**.

Honest tags: the **plumbing** is `Empirical` (self-test-evidenced). The live
**KC-2/SC-5b quality** and the **retention/leverage verdict** are a clearly-labelled
**USER-EXECUTED** step — `Declared` / **open, pending run** — until a human runs it
with a key. Any committed sample report is stamped **`SYNTHETIC (self-test)`**
(see `reports/SYNTHETIC-SAMPLE-*`). Per `research/11` §T11.7 step 4, the ablation
**never pre-writes the verdict** (VR-5).

Note: this **retention-ratio ablation is a distinct M-381 track** from the
[KC-2 LLM-leverage experiment](../../experiments/README.md) (M-002) run against the local
llama backend — KC-2 itself already returned a verdict (**proceed**, DN-09, 2026-06-18); this
`grok/` ablation is the separate, still-open retention-ratio follow-up cited above.

## The model rubric — `models.toml`

Five seed models, each with `{context, tpm, rpm, in/out price, batch in/out price}`.
Runs go **cheapest-first** (sorted by output price, then input price; ties keep file
order). `--models a,b,c` selects a subset; `--order a,b,c` forces an explicit order.

> **`Declared` numbers — confirm before a paid run.** Context windows, limits and
> prices are seeded from the xAI docs cited in the task brief
> (`docs.x.ai/developers/{models,pricing}`); they are asserted defaults, not values
> this harness verified. **Batch prices default to the sync prices** (no invented
> discount) with a `TODO(confirm)` — fill in the published batch rates from
> `docs.x.ai/developers/pricing#batch-api-pricing` before relying on batch cost
> figures. Cost accounting uses **batch** prices in batch mode and **sync** prices
> in live mode.

## The green gate (offline, no key, no network)

```sh
cd tools/llm-harness
uv run python -m grok.cli --self-test
# or, via the single entry point:  uv run python harness.py --grok --self-test
# (re)generate the committed SYNTHETIC sample report:
uv run python -m grok.cli --self-test --emit-sample
```

If `uv` is not yet wired, plain `python3 -m grok.cli --self-test` works too
(stdlib-only). Exit code is `0` iff every check passes.

## Setup on a WSL host (M-331)

```sh
# 1. In your WSL distro (Ubuntu etc.), get uv (one-time):
curl -LsSf https://astral.sh/uv/install.sh | sh        # then restart the shell
# 2. From the repo root:
cd tools/llm-harness
uv sync                       # creates .venv from pyproject.toml (no runtime deps)
uv sync --extra batch         # ALSO installs xai_sdk — only needed for --mode batch
# 3. Provide your key (never commit it). XAI_API_KEY or GROK_API_KEY:
export XAI_API_KEY=xai-...     # missing key => explicit error, never a silent run
```

`myc-check`-based scoring shells out to `cargo` (`cargo run -p mycelium-check --bin
myc-check`). If `cargo` is absent the scorer reports **SKIP** (never a false PASS);
install the Rust toolchain in WSL to get real syntactic/type-check scores.

## Run it — live (sequential, rate-limit-aware)

```sh
export XAI_API_KEY=xai-...
# whole rubric, cheapest-first, the M-330 generate->fix loop per gold task:
uv run python harness.py --grok --mode live --models-file models.toml
# a subset, explicit order, more correction rounds:
uv run python harness.py --grok --mode live --models grok-build-0.1,grok-4.3 --max-rounds 5
# list the resolved (ordered) models + prices and exit:
uv run python harness.py --grok --list-models
```

Live mode respects each model's **RPM/TPM** (sliding-window pacing) and backs off on
`429`/`Retry-After` (exponential, bounded) — the approach adapted from the repo's
tested `tools/github/gh-issues-sync.py` rate gate.

## Run it — batch (cheaper, independent generations via `xai_sdk`)

```sh
export XAI_API_KEY=xai-...
uv sync --extra batch                     # xai_sdk required for batch
uv run python harness.py --grok --mode batch --models-file models.toml
```

Batch mode submits the **independent first-pass generations** (one per gold task),
polls to completion, then scores — priced at the **batch** rate. The **iterative
correction loop is NOT batchable** (each fix depends on the prior round's
diagnostics) and stays a live-only follow-up; batch mode measures first-pass pass@1
quality and batch-priced cost. A missing `xai_sdk` is an explicit
`uv add xai_sdk` error — never a silent fallback to sync-priced live calls.

## Run it — the M-381 retention-ratio ablation (`research/11` §T11.7)

```sh
export XAI_API_KEY=xai-...
uv run python harness.py --grok --mode live --ablation --seeds 11,23,42
```

Runs the protocol's arms it **can** run (arm 1 bare surface, arm 2 grammar primer,
arm 4 `LlmCanonical` familiar-skin) over the composition task set × seeds, computes
**pass@1** per arm and the **retention ratio** = pass@1(best novel-surface arm) ÷
pass@1(familiar-skin arm 4), and compares to the `~70%` falsification threshold
(`RFC-0021 §4.7`). Arm 4 is scored at the **same** `myc-check` parse+typecheck bar
as arms 1/2 via the `llm_canonical_to_l1` bridge (DN-09 §9.4 option b): the model's
S-expression output is converted to `.myc` and typechecked by the real `myc-check`.
The bridge supplies the task's known fn signature (LlmCanonical, an expression IR,
cannot express one), so arm 4 is slightly advantaged and the retention ratio is a
**conservative** estimate — clearing the threshold is robust; falling below it is
ambiguous (see `RetentionVerdict.to_dict`'s `arm4_basis`). Arms 3
(grammar-constrained decoding) and 5 (embedded-DSL baseline) depend on build deps
that **do not exist yet** (a GBNF/Outlines decoder; an RR-3 host DSL); they are
wired but reported **`blocked`** with their reason — never fabricated. The leverage
claim stays **open (`Declared`)** until the full ≥3-seed, ≥1-frontier campaign.

## Reports

Each run writes, into `reports/`:

- `…-<model>-<mode>.json` — per-model metrics: **syntactic validity rate**,
  **type-check pass rate**, **mean edit-to-fix iterations** (KC-2/SC-5b), plus
  tokens, latency, request/batch counts, and **computed USD** (mode-appropriate
  price), with metadata (ISO timestamp, seed, task-set id, model, endpoint, mode);
- `…-comparison.md` — the cross-model comparison table.

Real runs use a wall-clock run id and are **git-ignored** (only the committed
`SYNTHETIC-SAMPLE-*` reference is tracked).

## What runs where (module map)

| module | role |
|---|---|
| `models.py` / `models.toml` | rubric load, cheapest-first ordering, USD cost math |
| `ratelimit.py` | per-model RPM+TPM pacer + exponential backoff on `429` |
| `budget.py` | the never-silent `--max-usd` spend gate (refuses a unit of work before it is sent) |
| `client.py` | `OpenAICompatClient` (live REST), `XaiBatchClient` (batch), `MockClient` |
| `scoring.py` | `myc-check` exit-code → syntactic / type-check / clean verdict |
| `tasks.py` | the gold composition-task set the loop and ablation run against |
| `coauthor_loop.py` | the M-330 generate→feedback→fix loop |
| `batch.py` | submit/poll/score independent generations at batch price |
| `runner.py` | orchestration: runs the gold task set (and/or the ablation) across models |
| `arm3_constrained.py` | ablation arm 3 (grammar-constrained decoding) — wired, reported `blocked` (no GBNF/Outlines decoder yet) |
| `arm5_embedded_dsl.py` | ablation arm 5 (RR-3 embedded-DSL baseline) — wired, reported `blocked` (no host DSL yet) |
| `llm_canonical_arm4.py` | ablation arm 4 (`LlmCanonical` familiar-skin): S-expression parse/validate |
| `llm_canonical_to_l1.py` | the arm-4 rigorous bridge: LlmCanonical → `.myc` surface, typechecked by `myc-check` |
| `ablation.py` | the M-381 retention-ratio ablation + (never-pre-written) verdict |
| `report.py` | per-model JSON + cross-model markdown (G11) |
| `selftest.py` | the offline deterministic green gate |
| `cli.py` | the `--mode live\|batch` / `--ablation` / `--self-test` driver |

## Related issues

- `#97` / `#127` (M-330) — LLM API integration (the `grok/` co-authoring loop)
- `#3` (M-002) — LLM leverage harness (the M-381 ablation reuses this framing)
- M-331 — portable, WSL-runnable Grok harness (the `grok/` package + this section)
- M-381 — `research/11` §T11.7 retention-ratio ablation (`--ablation`)

---
[Back to the llm-harness landing README](README.md)
