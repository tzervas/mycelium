# experiments-llm.md — LLM-Leverage Validation (KC-2 / M-381)

> **Orientation aid (Declared/Empirical) — not normative.** Source + normative corpus are
> ground truth. Cite: `docs/notes/DN-09-KC-2-Verdict.md`, `tools/llm-harness/README.md`,
> RFC-0021, `docs/notes/research-prompts.md`.

---

## What it is

The LLM-leverage validation track for Mycelium's KC-2 kill criterion: does the novel surface
allow machine co-authoring, or does it collapse irrecoverably? The result is a **two-part
system**: (1) the **M-002 `experiments/` harness** (local Qwen2.5-Coder, ran 2026-06-18 →
cleared KC-2 → `proceed` verdict); (2) the **`tools/llm-harness/` Grok/xAI harness** (M-330,
M-331, M-381), which extends the co-authoring loop and runs the **T3.6 five-arm retention
ablation** for the rigorous follow-up (RP-1).

---

## Where it lives

| Path | Role |
|---|---|
| `tools/llm-harness/` | The Grok/xAI harness root |
| `tools/llm-harness/grok/` | Core Python package |
| `tools/llm-harness/grok/ablation.py` | M-381 five-arm retention ablation |
| `tools/llm-harness/grok/llm_canonical_to_l1.py` | Arm-4 bridge: `LlmCanonical` S-expr → `.myc` (DN-09 §9.4 option b) |
| `tools/llm-harness/grok/arm3_constrained.py` | Arm-3 grammar-constrained decoding (blocked: needs local GBNF backend) |
| `tools/llm-harness/grok/arm5_embedded_dsl.py` | Arm-5 embedded-DSL baseline (RR-3; offline-testable; runnable via xAI) |
| `tools/llm-harness/grok/scoring.py` | `myc-check` exit codes → `{clean, syntax_error, type_error, error, skip}` |
| `tools/llm-harness/grok/tasks.py` | Gold task set `gold-compose-v1` (8 tasks; `Declared` seed set) |
| `tools/llm-harness/grok/selftest.py` | Offline green gate — **19 checks** in `ALL_CHECKS` (current) |
| `tools/llm-harness/local/` | WSL/RTX-5080 setup for arm 3 (idempotent; `setup_local_llm.py`) |
| `tools/llm-harness/README.md` | Full runbook (xAI live, batch, self-test, Termux setup) |
| `tools/llm-harness/grok/models.toml` | 5-model rubric; `Declared` prices — confirm before a paid run |
| `docs/notes/DN-09-KC-2-Verdict.md` | The authoritative KC-2 verdict + all ablation run records |
| `docs/notes/research-prompts.md` | RP-1 (retention ablation), RP-8 (perf spike) |
| `docs/rfcs/RFC-0021-Semantic-Level-Projections.md` | Projection framework; §4.7 supersession trigger |

---

## KC-2 verdict: proceed (DN-09, 2026-06-18)

**KC-2 (Foundation §2.4):** If LLM code-gen collapses *irrecoverably* even with projections +
semantic feedback (G10, R6), reweight to human-primary or fall back to embedded DSL (RR-3).
The bar is **irrecoverable collapse**, not high first-attempt pass rate.

**M-002 run (local Qwen2.5-Coder, 10-task gold set, seed 42) [Empirical]:**

| Model | Primer | First-attempt | Eventual | Edit-to-fix gain |
|---|---|---|---|---|
| 0.5B | minimal | 10% | 10% | +0.0pp |
| 1.5B | minimal | 30% | 40% | +10.0pp |
| 7B | examples | **40%** | **70%** | **+30.0pp** |

Semantic-feedback loop (G10) recovered a large fraction of failures. **No collapse; recoverable.**
**Verdict: proceed.** Neither redirect triggered. L3 strategy: **committed text syntax +
co-equal projection layer** (M-380, RFC-0021); embedded-DSL fallback (RR-3) stays unspent.

---

## The Grok/xAI harness (M-330/M-331/M-381)

**Architecture:**
- Pure Python + `uv`; live (sequential, RPM/TPM-paced, exponential backoff on 429) or batch mode.
- xAI API endpoint `https://api.x.ai/v1` (OpenAI-compatible); key from `$XAI_API_KEY` or `$GROK_API_KEY`.
- **Spend gate** (`--max-usd`, default $10): best-effort pre-flight guard (biased high, heuristic —
  live completions are unbounded, so can overrun; stops new work early — G2).
- Cheapest-first model ordering; `--models a,b,c` or `--order a,b,c` overrides.

**Run (from `tools/llm-harness/`):**

```sh
uv run python -m grok.cli --self-test          # offline green gate (19/19, no key)
./run.sh --smoke                                # $2 single-model smoke + full sweep
uv run python -m grok.cli --mode live --ablation --seeds 11,23,42   # retention ablation
```

**Module map:**

| Module | Role |
|---|---|
| `models.py` / `models.toml` | Rubric + cheapest-first sort + USD cost math |
| `ratelimit.py` | Per-model RPM+TPM pacer + exponential backoff on 429 |
| `client.py` | `OpenAICompatClient` (live), `XaiBatchClient` (batch), `MockClient` |
| `scoring.py` | `myc-check` exit codes → verdicts; injectable for offline test |
| `coauthor_loop.py` | M-330 generate→feedback→fix loop |
| `batch.py` | Independent first-pass generations at batch price |
| `ablation.py` | M-381 five-arm retention-ratio ablation |
| `llm_canonical_to_l1.py` | Arm-4 bridge: `LlmCanonical` S-expr → `.myc` (Empirical heuristic) |
| `arm3_constrained.py` | Arm-3 (grammar-constrained; offline-tested, runtime-blocked) |
| `arm5_embedded_dsl.py` | Arm-5 (embedded-DSL; restricted sandbox; runnable via xAI) |
| `report.py` | Per-model JSON + cross-model markdown (G11 dual projection) |
| `selftest.py` | Offline green gate (19 checks in `ALL_CHECKS`) |
| `cli.py` | `--mode live\|batch / --ablation / --self-test` driver |

**Self-test (offline, no key/network):** `ALL_CHECKS` has **19** entries (as of current HEAD;
`selftest.py:975`). Each is named T1–T17 + T2b + others. Green gate: plumbing is `Empirical`
(self-test-evidenced). DN-09 §8 records 17/17 at the time of the rigorous ablation run; the
count grew as arms 3/5 were added. Live model-quality verdict stays `Declared`/open.

---

## The 5-arm T3.6 ablation (M-381, RP-1)

Protocol per `research/11 §T11.7`, RFC-0006 §8 Q1. Task set: `gold-compose-v1` (8 tasks,
`TASK_SET_ID`, `tasks.py:18`). Seeds: 11, 23, 42 (24 samples/arm).

| Arm | Status | Description |
|---|---|---|
| arm1 — bare novel surface | Runnable | Mycelium text surface, no primer |
| arm2 — grammar-in-context primer | Runnable | Same surface + book-quality grammar primer |
| arm3 — grammar-constrained decoding | **Blocked** | Needs local GBNF backend (llama.cpp/Outlines); xAI REST has no grammar param; wired + offline-tested via `arm3_selftest` |
| arm4 — LlmCanonical projection | Runnable | S-expression IR; scored via the `llm_canonical_to_l1` bridge → `myc-check` |
| arm5 — embedded-DSL baseline (RR-3) | Runnable | Python mini-DSL → `.myc`; restricted sandbox; runnable via xAI |

**Retention ratio** = pass@1(best novel-surface arm) ÷ pass@1(arm4 LlmCanonical)
**Threshold (RFC-0021 §4.7):** < ~70% ⇒ promote `LlmCanonical` to primary surface (supersession).

---

## The arm-4 LlmCanonical→L1 bridge (DN-09 §9.4 option b)

`tools/llm-harness/grok/llm_canonical_to_l1.py` — converts each arm-4 `LlmCanonical`
S-expression to `.myc` surface text so the **same `myc-check`** (parse + typecheck) scores arm 4
at the same quality bar as arms 1/2. This made the retention ratio **DETERMINATE** after §9
reported it INDETERMINATE (arm 4 = 0%, a scoring artifact — `myc-check` returned exit 2 on
S-expression input by design).

**Intrinsic asymmetry (Declared — VR-5, `llm_canonical_to_l1.py:28`):** `LlmCanonical` is
an expression IR; it cannot express a function signature. The bridge supplies the task's known
signature (from `Task.fn_name`/`param_type`/`return_type`). Arm 4's model is NOT asked to
author the signature → arm 4 is **slightly advantaged** → retention ratio is a **conservative**
estimate. Clearing ~70% is robust; falling below is ambiguous. Tag: `Empirical` (heuristic,
self-test-evidenced; never upgraded to `Proven` — `llm_canonical_to_l1.py:38`).

---

## Retention ratio: DETERMINATE (DN-09 §10, 2026-06-20)

Run: `grok-build-0.1` (`20260620T230403Z`) + `grok-4.3` (`20260620T234224Z`).
Harness self-test: 17/17 (T15 bridge check added at that time). Spend: $0.088; cumulative ~$0.17.

| Model | arm1 bare | arm2 grammar-primer | arm4 LlmCanonical (bridge→myc-check) | Retention | Threshold |
|---|---|---|---|---|---|
| `grok-build-0.1` | 8.3% (2/24) | **91.7%** (22/24) | 16.7% (4/24) | **5.50× (550%)** | NOT falsified |
| `grok-4.3` | 4.2% (1/24) | **91.7%** (22/24) | 41.7% (10/24) | **2.20× (220%)** | NOT falsified |

All rates [Empirical]. **RFC-0021 §4.7 falsification trigger does NOT fire.** The grammar-primed
novel text surface outperforms `LlmCanonical` — the ratio is > 100% for both models. The
arm-4 denominator bias (conservative: bridge rejects non-convertible output; arm 4 had
dominant bridge-non-convertible failures) means the true ratio could be lower; non-falsification
is still robust because arm2 = 91.7% dominates any plausible arm-4 range.

**What this does NOT establish:** full five-arm (arms 3/5 blocked); only 2 models, 3 seeds,
8 tasks. The overall leverage claim stays **`Declared`/open** pending the full campaign.
**KC-2 verdict (§3: proceed) is unchanged and reinforced.**

---

## Scoring (`scoring.py`)

`myc-check` exit codes (confirmed in `crates/mycelium-check/src/bin/myc-check.rs`):

| Code | Verdict | Meaning |
|---|---|---|
| 0 | `clean` | Parses AND type-checks |
| 2 | `syntax_error` | Parse error (syntactically invalid) |
| 3 | `type_error` | Parses but fails type-check |
| 5 | `project_error` | Project-resolution error |
| 64 | `error` | Usage error |
| 66 | `error` | I/O error |

If scorer binary cannot be built/run → `skip` (never false PASS — G2).

---

## Key invariants (honesty)

1. **Never pre-write the verdict.** The harness explicitly refuses to assert a leverage
   conclusion before a real run lands. Any committed sample report is stamped
   `SYNTHETIC (self-test)`. (VR-5 / `ablation.py:11`.)
2. **Never a false PASS.** A missing tool/model → explicit `SKIP`; an exception → `FAIL` (G2).
3. **`mock-PASS` ≠ `PASS`.** Mock mode labels model-dependent results `mock-PASS` — never
   evidence of real model quality.
4. **Spend gate is best-effort, not a formal bound.** Biased high; a single in-flight request
   can overrun. Set `--max-usd` with headroom. (README.md §"Live xAI runs".)
5. **`models.toml` prices are `Declared`.** Confirm batch prices from xAI docs before relying
   on cost figures; batch prices default to sync prices (no invented discount, `TODO(confirm)`).
6. **Raw reports are git-ignored.** Only `SYNTHETIC-SAMPLE-*` committed; real results are
   ephemeral. (`tools/llm-harness/.gitignore`.)

---

## Tracked follow-ups

| Item | Status | Notes |
|---|---|---|
| arm3 — local GBNF backend | Open (M-331 / RP-1) | `tools/llm-harness/local/` — WSL + RTX-5080 setup (`setup_local_llm.py`); needs `llama-cpp-python` + CUDA |
| arm5 — embedded-DSL baseline | Now runnable | RR-3 unspent; arm-5 wired and offline-tested; runnable via xAI REST |
| arm4 bridge completeness | Open (RFC-0021 §4.1) | Fuller `LlmCanonical→L1` converter would tighten arm-4 lower bound |
| batch mode (`xai_sdk`) | Open | Blocked by gRPC TLS + `xai-sdk` 1.17.0 API drift; supplemental only |
| Harness→bench schema bridge | M-651 | Grok report schema (`metadata/quality/performance/outcomes`) differs from `mycelium-bench` ingestion schema (`harness/summary/results`); tracked separately |
| RP-8 — Performance spike | Open (non-blocking) | Profile workspace; cert-check toward ns range; extraction into tools; post-1.0; not a 1.0.0 blocker (`docs/notes/research-prompts.md §RP-8`) |

---

## Read more

- `docs/notes/DN-09-KC-2-Verdict.md` — all runs §2–§10; the authoritative verdict chain
- `docs/rfcs/RFC-0021-Semantic-Level-Projections.md` — projection framework + §4.7 trigger
- `docs/notes/research-prompts.md §RP-1` — T3.6 ablation protocol, falsification threshold
- `docs/notes/research-prompts.md §RP-8` — perf spike scope and honesty constraints
- `tools/llm-harness/grok/selftest.py` — `ALL_CHECKS` list (19 entries; `selftest.py:975`)

---

## Gotchas

- **Arm 3 cannot run via xAI REST** — no grammar parameter on the OpenAI-compatible endpoint.
  Use the `local/` WSL+RTX-5080 path (llama-cpp-python + GBNF).
- **Arm 4 scoring uses the bridge, not raw `myc-check`** — bare `myc-check` returns exit 2
  on S-expression input (a known design fact, not a bug). The bridge (`llm_canonical_to_l1.py`)
  is required for a meaningful arm-4 result.
- **The 2026-06-20 "INDETERMINATE" result (DN-09 §9) is superseded** — §10 landed the bridge
  and obtained DETERMINATE ratios for both models.
- **`models.toml` originally had 5 entries** including three `grok-4.20-*` variants; these were
  removed after reconciliation against xAI API docs. Confirm model list before a live run.
- **Batch mode requires `uv sync --extra batch`** (`xai_sdk`); missing it is an explicit error,
  never a silent fallback to sync-priced live calls.
- **Cumulative Grok/xAI spend across all sessions: ~$0.17** (far under the $10 session bound
  as of 2026-06-20). The spend gate resets per run, not per session — track cumulative manually.
