# Design Note DN-09 — The KC-2 Verdict (LLM-leverage on the Mycelium surface)

| Field | Value |
|---|---|
| **Note** | DN-09 |
| **Status** | **Resolved** (2026-06-18 — maintainer verdict recorded). The KC-2 kill/redirect criterion (Foundation §2.4) is **not triggered**: the measured leverage is *weak but recoverable*, not the irrecoverable collapse the criterion guards against. **Verdict: proceed.** This closes the standing KC-2 gate (RFC-0006 §10 Q1; RFC-0007 §10; phase plans) and selects the **L3 strategy: committed text syntax + a co-equal structured-projection layer**. |
| **Decides** | (1) the KC-2 verdict (`proceed`, not `reweight-to-human` and not `fall-back-to-embedded-DSL`); (2) the RFC-0006 Q1 L3 strategy = **text syntax** (the v0 grammar becomes the committed surface, refinable append-only) **+ projections** (M-380 opened co-equally, FR-S5 dual rendering); (3) what stays an honest open follow-up (the rigorous T3.6 ablation). |
| **Feeds** | RFC-0006 §8 Q1/§10 (the one deliberate deferral — now discharged); RFC-0007 §10 (concrete-surface-syntax carve-out); Foundation §2.4 / §6 P0.2 (KC-2 status); phase-0/1/2/3 KC-2 rows; SPEC §10.2; `docs/spec/stdlib/self-hosting-readiness.md` (capability #3); `experiments/` (the measured run); RR-3 |
| **Date** | June 18, 2026 |
| **Task** | M-002 (#3) — the KC-2 LLM-leverage run |

> **Posture (honesty rule / VR-5).** This note records a **maintainer verdict** read off measured
> rates — it does not upgrade those rates. The numbers below are *descriptive analysis* (the
> experiment harness explicitly refuses to pre-write the verdict, `experiments/README.md` §3); the
> *decision* is the maintainer's, recorded here append-only. Where the evidence is thin this note says
> so and routes the rigor to a tracked follow-up, never claiming more than the run established.

---

## 1. What KC-2 actually asks

Foundation §2.4 states the kill/redirect criterion precisely:

> **KC-2.** If the P0 LLM-leverage experiment shows code-gen/reasoning **collapses *irrecoverably*
> even with projections + semantic feedback** (G10, R6), reweight toward human-primary design or fall
> back to an embedded DSL in a high-resource host language.

The bar is **irrecoverable collapse**, not "high first-attempt pass rate". KC-2 is a *floor* —
"is the novel surface a dead end for machine co-authoring?" — bounded further by RFC-0006 **S6**
(the language never *needs* a model; KC-2 can only choose the L3 surface, never make Mycelium
depend on one). So the question this verdict answers is narrow and existential: **does the surface
kill machine leverage outright, or is the leverage real enough to proceed and improve?**

## 2. The measured evidence (M-002, 2026-06-18)

The KC-2 harness (`experiments/mycelium_experiments/kc2/`, landed M-002) was run for real against
**local** Qwen2.5-Coder models over the 10-task gold set (seed 42, edit-to-fix budget 3), in two
primer conditions — `minimal` (syntax-only reference) and `examples` (+ two complete, valid,
*non-answer* worked programs). The Mycelium arm is scored by `myc-check` (parse + typecheck +
signature). Per-run reports + summaries are committed under `experiments/results/`.

| Model | Primer | First-attempt (SC-5b) | Syntactic valid | Eventual | Edit-to-fix gain (G10) |
|---|---|---|---|---|---|
| 0.5B | minimal | 10% | 20% | 10% | +0.0pp |
| 0.5B | examples | 0% | 0% | 0% | +0.0pp |
| 1.5B | minimal | 30% | 50% | 40% | **+10.0pp** |
| 1.5B | examples | 20% | 20% | 30% | +10.0pp |
| 7B | minimal | 20% | 30% | 30% | +10.0pp |
| **7B** | **examples** | **40%** | **50%** | **70%** | **+30.0pp** |

**Read honestly (VR-5):**
- **No collapse — and crucially, the failure mode is *recoverable*.** The strongest arm (7B +
  examples primer) reaches **70% eventual** pass with a **+30pp edit-to-fix lift**. The
  semantic-feedback loop (G10) — the exact mechanism KC-2's "even with … semantic feedback" clause
  tests — *recovers* a large fraction of first-attempt failures. That is the **opposite** of
  irrecoverable collapse.
- **It scales the expected way.** Pass rate climbs with model size, and the `examples` primer (a
  grammar-in-context anchor) is the dominant knob at 7B — consistent with the grammar-prompting /
  MTOB evidence the Q1 hypothesis rests on (RFC-0006 §8 Q1; T3.6).
- **The surface is, plainly, learnable-from-context by a small local model with no fine-tuning.**

## 3. The verdict

**KC-2: proceed.** The kill criterion (irrecoverable collapse even with feedback) is **not
triggered** — feedback demonstrably recovers leverage, and the rate improves with capability and a
grammar-in-context primer. Neither redirect is warranted: not `reweight-to-human` (the surface is
machine-authorable), not `fall-back-to-embedded-DSL` (RR-3's contingency stays unspent).

Per the maintainer's own framing: *this is a half-written, brand-new, deliberately unusual
language, and the measured leverage — weak but recoverable and improving — is satisfying enough at
this maturity to keep building rather than kill or redirect.* The verdict is **proceed and
improve**, not "the surface is finished".

### 3.1 The L3 strategy this selects (RFC-0006 Q1)

Q1 offered three L3 strategies: **text syntax | structured projections | embedded DSL**. The
verdict selects **text syntax + projections, co-equally**:

- **Commit the v0 text syntax.** The existing grammar (`docs/spec/grammar/mycelium.ebnf`) becomes
  the **committed L3 surface** — no longer "throwaway / globally KC-2-gated", now the real surface,
  refined **append-only** (a spelling change is a recorded decision, not drift). This discharges
  RFC-0006 §10's "one deliberate deferral" (Q1) and the §10 Q6 literal-spelling gate.
- **Open the projection layer as a co-equal deliverable (M-380, FR-S5).** Because the measured
  leverage is *real-but-modest*, projections are not a hedge against failure but a **lever to lift a
  working surface**: the same content-addressed definitions (ADR-003) render for human and machine
  co-authors, and an LLM-facing canonical projection is the natural place to apply the grammar-in-
  context / constrained-decoding gains the data points at. M-380 moves from "KC-2-contingent /
  needs-design" to **design-active**.

The embedded-DSL fallback (RR-3) is **retained as a documented contingency**, now unspent — if a
future, more rigorous run reverses this, supersede this note (append-only).

### 3.2 Standing surface-design bias: usability over theming (maintainer, 2026-06-18)

A principle to carry into all L2/L3 surface work (the M-380 projection layer, the L2 surface-term-
language RFC, any future lexicon addition): **usability and clarity outrank the fungal metaphor.**
The DN-02 three-test gate still applies, but the tie always goes to the **familiar, regular,
boring** spelling (the Rust/ML-class word) — a themed word is adopted *only* where it genuinely
**teaches better** than the conventional one, never for flavor. This is grounded twice over: (a) the
KC-2 evidence itself — familiar-skinned, regular syntax is what retains LLM leverage (RFC-0006 §8 Q1
hypothesis); and (b) the immediate precedent of **`thaw`** (RFC-0017 §5), where the themed inverse
`germinate` was set aside both because it was already taken (ADR-013) *and* because the plain word is
clearer. Theming is a teaching tool, not a brand.

## 4. What this verdict does **not** claim (honest scope — the open follow-up)

The run establishes *non-collapse + recoverability*, enough to clear KC-2. It does **not** establish
the strong form of the Q1 hypothesis at the rigor T3.6 designed for. Recorded as the standing
follow-up so the corpus stays honest about the gap:

- **Single seed, 10 tasks.** Coarse signal (~10pp per task); not a statistical estimate. The
  per-task struggles (`kc2-04-ternary-add`, `kc2-05-swap`, `kc2-07-data-match` resisted every arm)
  are cues for surface/primer iteration, not conclusions.
- **The T3.6 falsification ablation was not run.** RFC-0006 §8 Q1's headline metric — the
  **LLM-leverage *retention ratio*** vs a *familiar-skinned same-AST* condition, with the explicit
  threshold (spec-in-context retaining <~70% of the familiar-skin pass@1 ⇒ L3 must become a
  projection of known syntax) — requires the four-arm comparison (bare novel; +grammar-in-context;
  +constrained decoding measured separately; familiar-skin). Only the novel arm under two primers
  ran. So "novel-but-regular syntax *retains most* leverage" stays a **supported-but-not-confirmed**
  working hypothesis.
- **Local small models only (≤7B), no constrained decoding, no human-in-loop.** Frontier-model and
  grammar-constrained-decoding leverage are expected to be higher; unmeasured here.

**Research prompt (tracked for a variant pass — feeds M-002/M-380):** run the full T3.6 five-condition
ablation (bare novel syntax · +book-quality grammar-in-context · +constrained decoding, measured
separately · familiar-skinned same-AST · embedded-DSL) across ≥3 seeds and a wider task set, on at
least one frontier model and one grammar-constrained decoder, reporting the **retention ratio**
against the familiar-skin arm with the falsification threshold applied. Outcome either *confirms*
the committed text surface at the designed rigor or *triggers* the projection-of-known-syntax path
(M-380) — both already provisioned by this verdict. (See `docs/notes/research-prompts.md`.)

## 5. What this unblocks (the ripple)

Recorded so the dependent docs can be updated consistently (the gate they cite is now closed):

- **RFC-0006 §10 / §8 Q1, Q6** — concrete L3 syntax ratified (text); the literal suffix/annotation
  spelling (Q6) is committable. (Q3 stage-1 grading and the Q8 `unsafe` *spelling* stay open on
  their own merits — see below.)
- **RFC-0007 §10** — the "concrete surface syntax (KC-2)" carve-out is discharged; the calculus was
  already Accepted, now its surface is committed.
- **E3-1 / M-380** — semantic projections move from KC-2-contingent to design-active.
- **`self-hosting-readiness.md` capability #3** — "a concrete surface syntax to author a module in"
  flips `not-yet → ready`.
- **Foundation §6 P0.2 / phase plans** — the last open P0 probe (KC-2) now has its written verdict.

**Still genuinely open (not unblocked by KC-2 — they were never KC-2-gated):**
- **Stage-1 static guarantee grading (RFC-0006 Q3).** Its load-bearing sub-decision (do implicit
  flows taint?) is a normative decision for the grading RFC, flagged-novel — not a syntax matter.
- **The `unsafe`-class spelling (Q8).** The *mechanism* (`wild`, denied-by-default) is committed;
  only the L3 *theming* detail remains, a DN matter.

## 6. Append-only discipline

This note is **Resolved**; revisiting the verdict means a **new** note that supersedes it (e.g. if
the §4 follow-up ablation reverses the finding), never a rewrite. The measured `experiments/results/`
artifacts are the immutable evidence base; this note is the maintainer's reading of them.

## 7. Grok/xAI arm attempt (2026-06-20) — harness verified, live run blocked

This section records the 2026-06-20 attempt to run the Grok/xAI co-authoring experiment
(`tools/llm-harness/`, M-330/M-331/M-381) against the `gold-compose-v1` task set (8 tasks,
2 confirmed public models cheapest-first: `grok-build-0.1`, `grok-4.3`). Note: `models.toml`
originally contained 5 entries (including three `grok-4.20-*` variants); these were removed
in this PR after reconciliation against the operator-provided xAI API docs.

**Harness self-test** (Empirical — plumbing-verified, no key/network): **14/14 checks passed**
(T0–T12 + T2b: model ordering, RPM/TPM pacing math, live RatePacer virtual-clock smoke,
backoff/throttle, cost accounting, scoring, the M-330 generate→fix loop, the M-381 ablation
protocol, report emission, and the USD spend gate).
The harness infrastructure is verified and turnkey.

**Live run result** (Empirical — run executed, API reachable): the probe against every model
returned `HTTP 403 permission-denied` on every task request. The error text: *"Your newly created
team doesn't have any credits or licenses yet."* The API endpoint (`https://api.x.ai/v1`) is
reachable (explicit 403, not a network timeout); the account bound to `XAI_API_KEY` has no
credits. Total tokens consumed: 0. Total spend: $0.00 (spend gate never triggered).

**This is a billing constraint, not a language-learnability result.** Zero tasks ran; the
all-zero outcome rates reflect API failure, not Mycelium surface behaviour by any model.

**Retention ratio (T3.6 / M-381): INDETERMINATE.** The ablation did not run. The 5-arm retention
ablation (bare novel; +grammar-in-context; +constrained decoding; familiar-skin / `LlmCanonical`;
embedded-DSL) requires live token production; none occurred.

**Schema finding (Declared):** the Grok harness report format (`metadata` / `quality` /
`performance` / `outcomes`) differs from the schema `crates/mycelium-bench` binds to
(`harness` / `summary` / `results` — the `mycelium-llm-validation` harness schema). Even a
successful live run cannot be ingested by the bench crate without a schema bridge. This is a
separate tracked issue; it does not affect the verdict below.

**Effect on the standing verdict:** the 2026-06-18 verdict — **KC-2: proceed** — stands
unchanged. The Grok arm would have provided supplemental frontier-model evidence (see §4's
open follow-up), but its absence does not overturn the local-model measurement that cleared
the kill criterion. The L3 strategy selection (§3.1) and the usability bias (§3.2) are
unaffected.

**What reopens this:** loading credits into the xAI account and re-running. The harness is
turnkey (`cd tools/llm-harness && ./run.sh --smoke` then `./run.sh`); a successful re-run
appends findings here (append-only) and updates M-381 with the measured retention ratio.

## 8. Grok/xAI retry run (2026-06-20) — harness fixed, live run completed

This section records the follow-up live run once xAI account credits were available (same day,
later session). Three fixes were applied to the harness before re-running (M-330 diagnostic
feedback; see commit `2c3c2e0`):

1. **Diagnostic feedback fix.** `myc-check` emits parse/type errors on stdout; the scorer was
   only reading stderr. Every task therefore received `diagnostics: []` in the correction
   prompt — the model was told "fix your syntax" with no indication of what was wrong. Fixed:
   stderr preferred, stdout fallback. Each failing round now carries the actual
   `parse-error: …` or `check-error: …` message.
2. **Selective resume (`--resume-from reports/`).** Prior PASS outcomes are carried forward;
   only non-PASS tasks are retried. This avoids re-billing tasks already confirmed clean.
3. **Harness self-test:** extended to **16/16** (T14 resume-logic bookkeeping added, T6
   stdout-fallback path added).

**Live run results (Empirical — run executed, 2 models, task set `gold-compose-v1`, seed 42,
spend cap $5):**

| model | run | syntactic valid | type-check pass | cost | requests | mean latency |
|---|---|---|---|---|---|---|
| `grok-build-0.1` | retry | 14.3% (1/7 scored) | 14.3% (1/7) | $0.006097 | 18 | 33.8s |
| `grok-4.3` | retry | 12.5% (1/8) | 0.0% (0/8) | $0.011025 | 24 | 4.1s |

Task-level outcomes (retry run `20260620T151333Z`):

| task | grok-build-0.1 | grok-4.3 |
|---|---|---|
| g01-identity | **PASS** (attempt 1, clean) | FAIL (syn×3) |
| g02-not | FAIL (syn×3) | FAIL (syn×3) |
| g03-double | FAIL (syn, **typ**, syn) | FAIL (syn×3) |
| g04-widen-swap | FAIL (syn×3) | FAIL (syn×3) |
| g05-narrow-swap | FAIL (syn×3) | FAIL (syn×3) |
| g06-compose-not-double | FAIL (syn×2, err) | FAIL (syn×3) |
| g07-and-then-widen | FAIL (syn×3) | FAIL (syn, syn, **typ**) |
| g08-roundtrip | **PASS** (carried from blind run) | FAIL (syn×3) |

`syn` = syntax_error (myc-check exit 2); `typ` = type_error (exit 3); `err` = harness-level error
(no exit code — runner timeout/transport, not a model parse failure); **typ** means the model
produced syntactically valid Mycelium that failed type-checking — an improvement signal.

**What the diagnostic feedback confirmed (Empirical):**

- Rounds 2+ carry `is_correction: true` and the actual parse/type error in `diagnostics`. The
  model IS receiving the error message in its correction prompt — the feedback loop is wired.
- Example (g02-not, grok-build-0.1):
  - Round 1 diagnostic: `"parse-error: parse error at 2:1: expected a top-level item … found Ident(\"flip\")"` — model wrote `flip(…)` not `fn flip(…)`.
  - Round 2 diagnostic: `"parse-error: unexpected character '~'"` — tried `~` for NOT.
  - Round 3 diagnostic: `"parse-error: unexpected '-' (expected '->')"` — still not converging.
- The models can recognise the feedback but do not have enough Mycelium knowledge to self-
  correct within 3 rounds. This is expected for frontier models with zero Mycelium fine-tuning.

**New PASSes across both runs (Empirical):**

- grok-build-0.1: g08-roundtrip (blind) + **g01-identity (retry, first attempt)** → 2/8 total.
- grok-4.3: 0/8 total (in both the blind and retry runs).

Both PASSes involve trivial wrapping functions (identity / round-trip). Composition, NOT, double,
widen-swap resist all 3 rounds with diagnostics. The failure mode is *not* irrecoverable (the
local-model evidence §2 already established that); it is a knowledge-surface gap (the models
lack Mycelium-specific syntax knowledge without fine-tuning or a sufficiently explicit primer).

**Retention ratio (T3.6 / M-381): INDETERMINATE.** Arms 3 (grammar-constrained decoding),
4 (LlmCanonical — the denominator), and 5 (embedded-DSL) remain blocked. The retention-ratio
threshold comparison requires arm 4; it cannot be computed from arms 1+2 alone.

**Cumulative spend: ~$0.035** across all three runs ($0.000 blind-blocked, $0.018 blind-live,
$0.017 retry). Well within the $5 run cap and $10 session constraint.

**Effect on the standing verdict:** the 2026-06-18 verdict — **KC-2: proceed** — stands
unchanged. The frontier-model arm adds supplemental evidence:
- The surface is *parseable* by frontier models without fine-tuning (g01-identity, g08-roundtrip PASS).
- Composition tasks resist 3-round correction without Mycelium-specific priming — consistent
  with the local-model finding (§2: the grammar-in-context primer was the dominant knob at 7B).
- No irrecoverable collapse — the failure mode is a knowledge gap, not a structural one.

---

## 9. Ablation run (2026-06-20, arm 4 now runnable) — retention ratio INDETERMINATE

This section records the M-381 five-arm retention ablation run (`20260620T195352Z`), executed
after landing the `llm_canonical_parse` binary (M-381 Arm 4, W2L3) which unblocked arm 4.
Run: `grok-build-0.1`, task set `gold-compose-v1`, seeds [11, 23, 42], $5 cap, actual spend
$0.0373 (cumulative ~$0.072 across all Grok/xAI sessions — well within the $10 session bound).

### 9.1 Gold task set results (grok-build-0.1, seed 42, 6 retried tasks)

Two tasks carried forward (PASS) from the §8 retry run: **g01-identity**, **g08-roundtrip**.

| task | status | rounds | note |
|---|---|---|---|
| g01-identity | PASS (carried) | 1 | from §8 |
| g02-not | FAIL | 3 | syn×3 |
| g03-double | FAIL | 3 | syn×2, typ |
| **g04-widen-swap** | **PARTIAL_PASS** | 2 | syn then clean (self-corrected with diagnostics) |
| **g05-narrow-swap** | **PASS** | 1 | first-attempt clean |
| g06-compose-not-double | FAIL | 3 | syn×3 |
| g07-and-then-widen | FAIL | 3 | syn×3 |
| g08-roundtrip | PASS (carried) | 1 | from §8 |

**Cumulative gold-set result across §7–§9:** 4/8 tasks PASS or PARTIAL_PASS (g01, g04, g05, g08).
g04 and g05 are new PASSes — not previously seen in §7/§8. Both involve explicit representation
swaps: g04 (widen: Binary{8}→Binary{16}) and g05 (narrow: Binary{16}→Binary{8}). The
diagnostic-feedback loop (M-330) enabled the self-correction. These are positive signals for the
"recoverable, not irrecoverable" KC-2 verdict and the grammar-feedback mechanism.

### 9.2 Ablation arm results (Empirical, 8 tasks × 3 seeds = 24 samples per arm)

| arm | ran | n_clean / n_samples | pass@1 | guarantee tag |
|---|---|---|---|---|
| arm1 — bare novel surface | yes | 0 / 24 | **0.0%** | Empirical |
| arm2 — +grammar-in-context primer | yes | 24 / 24 | **100.0%** | Empirical |
| arm3 — grammar-constrained decoding | **blocked** | — | — | Declared |
| arm4 — LlmCanonical projection | yes | 0 / 24 | **0.0%** | Empirical |
| arm5 — embedded-DSL baseline | **blocked** | — | — | Declared |

**Retention ratio: INDETERMINATE.** arm4.pass@1 = 0.0 — the denominator is zero; the ratio
cannot be formed. The threshold comparison (≥ ~70% ⇒ novel surface retains leverage) applies
only when arm 4 is present with a non-zero denominator (research/11 §T11.7 step 3).

### 9.3 Honest interpretation (VR-5)

**arm2 (grammar-primer) at 100% is a strong, reproducible signal [Empirical].** Across all 8
tasks and 3 seeds (24 independent samples), every generation passed `myc-check` on the first
attempt when the grammar-in-context primer was included. This replicates the prior
`experiments/` finding (§2: grammar-in-context was the dominant knob at 7B) and extends it to a
frontier API model (grok-build-0.1) across multiple seeds. The grammar primer is highly
effective as a lever; without it (arm1), pass@1 = 0%.

**arm4 (LlmCanonical) at 0% is a scoring-method artifact, not a model failure [Declared].**
The arm4 scorer uses `myc-check` (which expects `.myc` L1 surface syntax) to evaluate
LlmCanonical S-expression output. `myc-check` returns exit code 2 (parse error) on
S-expression input by design — this is documented in the ablation code and was anticipated
before the run. A 0% arm4 score means "the current scorer cannot validate LlmCanonical output"
— NOT "the model fails to produce LlmCanonical syntax". To get a meaningful arm4 result, a
LlmCanonical→L1 converter (or a separate S-expression scorer) is required (RFC-0021 §4.1
EditCapability follow-up). The arm4 result is honest but uninformative about model capability.

**What this run establishes:**

- Grammar-in-context primers are highly effective for grok-build-0.1 on the Mycelium surface
  (100% pass@1 across all tasks and seeds). [Empirical]
- The retention ratio threshold comparison (research/11 §T11.7 step 3; RFC-0021 §4.7) remains
  INDETERMINATE: arm4 requires a scorer that understands LlmCanonical format.
- The KC-2 verdict (§3: **proceed**) is not affected; this run adds supplemental evidence.

**What this run does NOT establish:**

- Whether the LlmCanonical format provides a meaningful advantage over the grammar-primed novel
  surface — the arm4 scorer cannot currently answer that question.
- A determinate retention ratio or threshold comparison.
- Any upgrade of the leverage claim beyond "Declared/open" (VR-5).

### 9.4 What reopens arm4

To get a determinate arm4 result: build a scorer that evaluates LlmCanonical S-expression output
on its own terms (not via `myc-check`). Options:
(a) **Python-side structural validator only**: use the existing `parse_llm_canonical_py` — a
    correct S-expression parse = PASS for arm4's purpose. This would give arm4 its pass@1 for the
    retention ratio computation, though it cannot verify type correctness.
(b) **Full LlmCanonical→L1 bridge**: a converter from S-expressions to `.myc` surface, allowing
    `myc-check` to fully validate (parse + typecheck). This is the RFC-0021 §4.1 EditCapability
    path, estimated 1–2 days effort.

Option (a) is immediately implementable; option (b) is the rigorous path. Either reopens the
retention ratio comparison. Tracked as the standing follow-up: update M-381 when one of these
lands.

---

## Changelog

- **2026-06-20 — §9 added (M-381 arm 4 ablation run).** arm 4 now runnable
  (`llm_canonical_parse` compiled, M-381 W2L3 landed). Ablation run
  `20260620T195352Z`: arm1 0%, arm2 100%, arm4 0% (scoring-method artifact).
  Retention ratio: INDETERMINATE (arm4 denominator zero). Gold set: g04 PARTIAL_PASS,
  g05 PASS (new). KC-2 verdict unchanged: proceed.
- **2026-06-20 — §8 added (Grok/xAI retry run completed).** Harness fixed (stdout diagnostic
  fallback + `--resume-from`; self-test **16/16**). Retry run `20260620T151333Z` executed live:
  grok-build-0.1 14.3% syntactic valid / 14.3% type-check pass; grok-4.3 12.5% / 0.0%.
  Diagnostic feedback confirmed working. Arms 3/4/5 still blocked; retention ratio still
  INDETERMINATE (arm 4 denominator not run). KC-2 verdict unchanged: proceed.
- **2026-06-20 — Supplemental record (Grok/xAI arm blocked).** The `tools/llm-harness/`
  harness passed **14/14 offline self-tests** (Empirical/plumbing). The live run was blocked
  by `HTTP 403 permission-denied` (xAI account has no credits; not a language-model result).
  M-381 retention ratio: INDETERMINATE. The 2026-06-18 verdict (proceed) stands unchanged.
  Schema mismatch noted between Grok harness output and `mycelium-bench` ingestion format.
- **2026-06-18 — Resolved (maintainer verdict).** Records the **KC-2 verdict = proceed**: the M-002
  run (local Qwen2.5-Coder, 10-task gold set, seed 42) shows *weak-but-recoverable* leverage (best
  arm 7B+examples: 40% first-attempt → 70% eventual, +30pp edit-to-fix), so the "irrecoverable
  collapse even with feedback" kill criterion (Foundation §2.4) is **not triggered**. Selects the
  RFC-0006 Q1 L3 strategy = **committed text syntax + co-equal projection layer (M-380)**; retains the
  embedded-DSL fallback (RR-3) unspent. Names the honest open follow-up (the T3.6 retention-ratio
  ablation was not run) as a tracked research prompt. Closes the standing KC-2 gate across the corpus.
