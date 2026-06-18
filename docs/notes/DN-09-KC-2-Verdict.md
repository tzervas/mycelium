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

---

## Changelog

- **2026-06-18 — Resolved (maintainer verdict).** Records the **KC-2 verdict = proceed**: the M-002
  run (local Qwen2.5-Coder, 10-task gold set, seed 42) shows *weak-but-recoverable* leverage (best
  arm 7B+examples: 40% first-attempt → 70% eventual, +30pp edit-to-fix), so the "irrecoverable
  collapse even with feedback" kill criterion (Foundation §2.4) is **not triggered**. Selects the
  RFC-0006 Q1 L3 strategy = **committed text syntax + co-equal projection layer (M-380)**; retains the
  embedded-DSL fallback (RR-3) unspent. Names the honest open follow-up (the T3.6 retention-ratio
  ablation was not run) as a tracked research prompt. Closes the standing KC-2 gate across the corpus.
