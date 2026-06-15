# ADR-015 — `DEFAULT_ENUM_BUDGET` decode policy default = 4096 (guarantee-maximal)

| Field | Value |
|---|---|
| **ADR** | 015 |
| **Title** | Fix the RFC-0010 decode-methodology selector's default enumeration budget at `DEFAULT_ENUM_BUDGET = 4096` (= `MAPI_RESONATOR_PROFILE.max_capacity`) — the *guarantee-maximal* arm: every in-regime request is also enumerable, so the brute-force `Exact` arm dominates the whole validated envelope, rather than the *cost-optimal* ≈128 |
| **Status** | **Accepted** (maintainer directive, 2026-06-15) |
| **Date** | 2026-06-15 |
| **Depends on** | RFC-0010 §3/§4.3/§8 (the decode-method selector, the `enum_budget` cost-model parameter, and the open question this ratifies); RFC-0009 §10.3 (`MAPI_RESONATOR_PROFILE`, `max_capacity = 4096`); RFC-0005 §2 (the one selection mechanism + mandatory `EXPLAIN`); **G2** (no black boxes / never silent), **G4** (resonator guarantees are probabilistic), **VR-5** (no tag upgrade without a checked basis), tension **D** (reified/auditable selection — the policy stays inspectable, the trade lives in `EXPLAIN`) |
| **Resolves** | RFC-0010 §8 — the `enum_budget` crossover open question (a policy call, now ratified) |

## Context

RFC-0010's decode-methodology selector routes a `reconstruct_factors` request between a
**brute-force `Exact`** arm (enumerate `∏ᵢ kᵢ` codebook combinations — the RFC-0009 §5.3 oracle) and
the **`Empirical` resonator** arm (RFC-0009 §3) at the `∏k ≤ enum_budget` boundary (RFC-0010 §3 table,
row 1). `enum_budget` is a declared, measurement-fit policy parameter (RFC-0010 §4.3): too high and an
enumeration that is too slow runs anyway; too low and a cheaply-`Exact` request is needlessly downgraded
to `Empirical`. The default value of that knob was an **unrecorded guarantee-vs-latency policy call** —
the prototype shipped with `DEFAULT_ENUM_BUDGET = MAPI_RESONATOR_PROFILE.max_capacity` (4096), but the
*decision* to set it there was never ratified (RFC-0010 §8). This ADR ratifies it.

The choice is now grounded, not guessed. The wall-clock crossover instrument
(`tests/decode_select.rs::decode_method_enum_budget_crossover`, recorded in `CHANGELOG.md` under the
RFC-0010 follow-ups and in RFC-0010 §8 — **cited here, not re-run**) measured:

- the **cost-parity crossover is `∏k ≈ 100–128`**, and it is **`d`-independent** — both arms scale with
  `d`, so the crossover is a function of `∏k` alone;
- **brute force is cheaper only for `∏k ≲ 64`**;
- at the validated regime edge **`∏k = 4096` brute force costs ≈ 19× the resonator** (≈ 76 ms vs ≈ 4 ms
  at `d = 4096`) — the latency price of the `Exact`-over-`Empirical` upgrade;
- setting the budget at 4096 keeps the brute-force arm's **absolute latency bounded ≤ ≈ 157 ms at
  `d = 8192`**.

So the two honest candidates are:

- **`DEFAULT_ENUM_BUDGET = 4096` — guarantee-maximal.** Equal to the resonator's validated
  `max_capacity`, so **every in-regime request is also enumerable**: the `Exact` arm dominates the
  *entire* current validated envelope, and the selector never takes `Empirical` when `Exact` is cheaply
  available. Cost: a bounded latency tax (up to ≈ 19× the resonator, ≤ ≈ 157 ms) on the slice
  `64 < ∏k ≤ 4096`.
- **`DEFAULT_ENUM_BUDGET ≈ 128` — cost-optimal / latency-minimal.** Sits at the measured crossover, so
  neither arm ever pays a latency penalty relative to the other. Cost: requests in `128 < ∏k ≤ 4096`
  that *could* have been answered `Exact`ly are instead routed to the resonator and earn only the weaker
  `Empirical` tag — guarantee traded for speed.

Neither is "best" in the abstract: 4096 maximizes the honest guarantee inside a bounded latency
envelope; ≈128 minimizes latency at a guarantee cost. The corpus must pick one *default* (callers can
override per call, RFC-0010 §4.5) and record the trade honestly.

## Decision

Set **`DEFAULT_ENUM_BUDGET = 4096` (= `MAPI_RESONATOR_PROFILE.max_capacity`)** — the **guarantee-maximal**
default.

Rationale, in priority order:

1. **Keep the stronger honest guarantee when it is cheaply available** (RFC-0010 §6 rationale; the
   lattice `Exact ⊐ Empirical`). Mycelium's thesis is honest, *maximal* guarantees per operation; routing
   a cheaply-enumerable request to the `Empirical` resonator to save latency throws away a free `Exact`
   answer (and free identifiability — RFC-0010 §4.4) for no correctness gain. The guarantee-maximal
   default makes that the *out-of-the-box* behaviour rather than an opt-in.
2. **The latency cost is real but bounded and disclosed.** ≈ 19× at the regime edge is a worst-case
   *ratio* on a ≈ 4 ms baseline; the *absolute* worst case is ≤ ≈ 157 ms at `d = 8192`. That is an
   acceptable interactive-decode latency for the design phase, and the `EXPLAIN` cost lines (RFC-0010
   §3/§4.5, SC-5) surface the per-candidate cost on every selection, so the tax is never hidden (G2).
3. **The knob is exposed, so latency-sensitive callers are not trapped** (RFC-0010 §4.5 override). A
   caller for whom the tax is unacceptable can pin `enum_budget ≈ 128` (or force the `Resonator` arm) per
   call, recorded in the trace. The *default* therefore optimizes the property that cannot be recovered
   after the fact — the guarantee tag — while leaving the recoverable one (latency) caller-tunable.

This default is a **`Declared` policy parameter**, tagged as such: the crossover *number* is measured
(`Empirical` evidence), but *where to set the budget along it* is a policy stance (guarantee-priority),
asserted and flagged, not derived. Critically, **neither value upgrades any guarantee** (VR-5): the
guarantee tag is always read off the chosen arm (RFC-0010 §4.4); the budget changes only *which arm runs*,
never *what tag that arm earns*. There is no honesty difference between the two defaults — only a
guarantee-coverage-vs-latency difference.

## Consequences

**Positive.**

- The brute-force `Exact` arm covers the **entire** validated regime by default; in-regime decodes are
  `Exact` + identifiability-checked unless a caller opts out. This is the strongest honest default the
  measured envelope allows.
- The choice is now a recorded decision with a cited basis, closing RFC-0010 §8's `enum_budget` open
  question (the §8 "guarantee-maximal vs cost-optimal" framing is ratified toward guarantee-maximal).
- No code, kernel, or test change: the shipped prototype already defaults to 4096
  (`mycelium-vsa::decode_select`); this ADR ratifies the value the implementation already carries, so
  corpus and code agree.

**Negative / costs (honestly recorded).**

- Requests in `64 < ∏k ≤ 4096` pay up to a ≈ 19× latency tax (≤ ≈ 157 ms) versus the resonator, to earn
  the `Exact` tag — a guarantee-for-latency trade the cost-optimal ≈128 default would not pay. Accepted
  deliberately; surfaced in `EXPLAIN`.
- With the budget pinned to today's `max_capacity`, the **`Empirical` resonator arm is unreachable by the
  default in-regime** — it becomes load-bearing only when a caller lowers the budget (latency) or once the
  resonator's *validated* capacity grows past the enumeration budget. That is the expected state, and it
  underlines (RFC-0010 §8) that pushing the resonator's validated capacity well beyond what is cheaply
  enumerable is what makes the `Empirical` arm earn its place.

**Re-open trigger (deferred, YAGNI).** Revisit this default if either holds: (a) measured workloads show
the bounded latency tax is unacceptable in practice (then lower toward ≈128, trading guarantee for
latency, per call or as a new default via an append-only superseding ADR); or (b) the resonator's
validated `max_capacity` grows past the enumeration budget, making the `Empirical` arm reachable in-regime
and the budget a genuine `Exact`/`Empirical` frontier rather than a no-op ceiling. The natural companion
is RFC-0010 §8's **cheap identifiability precheck for the resonator arm** — it would let a resonator-arm
`Refuse` distinguish "ambiguous instance" from "resonator miss," improving `EXPLAIN` quality at the
frontier. Both are **deferred, not built here** (YAGNI): no precheck implementation, no kernel surface,
no selector change lands with this ADR — it is a standalone policy ratification.

**Append-only.** This ADR *resolves* RFC-0010 §8's open question by ratifying a value; it does not rewrite
RFC-0010's design. To change the default later, **supersede** this ADR (don't rewrite it).

## Grounding

RFC-0010 §3/§4.3/§4.4/§4.5/§6/§8 (selector, cost model, tag-flow, override, rationale, the resolved open
question); RFC-0009 §10.3 (`MAPI_RESONATOR_PROFILE`, `max_capacity = 4096`); RFC-0005 §2 (one selection
mechanism, mandatory `EXPLAIN`); measured crossover in `CHANGELOG.md` (RFC-0010 follow-ups,
`decode_method_enum_budget_crossover`) and RFC-0010 §8 — **cited, not re-run**. House rules: **G2**
(never silent — the cost is in `EXPLAIN`), **G4** (the resonator arm is probabilistic, so the default
that keeps the `Exact` arm in reach is the honest-maximal one), **VR-5** (no tag upgrade; the budget
moves arms, not tags), the guarantee lattice `Exact ⊐ Empirical` (keep the stronger tag when cheaply
available — RFC-0010 §6), tension **D** (the selection policy stays reified and inspectable; the
guarantee-vs-latency trade is disclosed in `EXPLAIN`, not hidden), **KISS/YAGNI** (a single declared
default + a deferred re-open trigger, no precheck machinery built).

## Meta — changelog

- **2026-06-15 — Accepted (maintainer directive).** Ratifies `DEFAULT_ENUM_BUDGET = 4096`
  (= `MAPI_RESONATOR_PROFILE.max_capacity`), the guarantee-maximal default, over the cost-optimal ≈128,
  on the measured `∏k ≈ 100–128` cost-parity crossover (≈ 19× / ≤ ≈ 157 ms latency tax at the regime
  edge). Tagged a `Declared` policy stance; neither value upgrades any guarantee (VR-5). Resolves
  RFC-0010 §8's `enum_budget` open question; the identifiability-precheck companion is noted as the
  re-open trigger and deferred (YAGNI). Standalone decision — no code, kernel, or test change.
