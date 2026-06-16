# RFC-0014 — Declarative Error Recovery & Bounded Effects

| Field | Value |
|---|---|
| **RFC** | 0014 |
| **Status** | **Draft (Proposed)** (drafted 2026-06-16; the isolated recovery subsystem RFC-0013 §8/§9 deferred — maintainer direction captured, design open for ratification) |
| **Type** | Foundational / normative (once Accepted) — a **separable** surface + runtime subsystem; minimal/no kernel change (KC-3) |
| **Date** | 2026-06-16 |
| **Feeds** | RFC-0006 (the optional recovery/effect surface); RFC-0008 (runtime — where effect budgets are enforced, alongside fuel/depth); RFC-0013 (the diagnostic *presentation* of an error this RFC *acts on*); the stdlib (a `result`/`effect` module candidate, M-346) |
| **Depends on** | RFC-0001 (errors are explicit `Option`/error/refusal *values*; `CheckVerdict::NotValidated` carries reason + fallback); RFC-0013 (sibling — shared error-class registry + reified-policy pattern; this RFC is the recovery concern RFC-0013 §4.4/§8 deferred); RFC-0005 / ADR-006 (the reified, inspectable, content-addressed policy pattern); RFC-0006 (surface/term-layer, KC-2-gated syntax); RFC-0004 §4 / RFC-0007 §4.5 / M-347 / DN-05 (the **budget discipline** — fuel clock, control-stack depth ceiling, dynamically-resolved budgets — that bounded effects generalise); G2 (never-silent), VR-5 (honest, downgrade-only guarantees), KC-3 (small kernel), SC-3 (transparent control), NFR-2 / SC-5b (semantic feedback), NFR-7 (differential) |
| **Tracks** | (new milestone — *declarative recovery & bounded effects*; to be filed. Lineage: DN04-Q1 deferred half → RFC-0013 §8/§9 → here) |

---

## 1. Summary

Errors in Mycelium are **load-bearing values**, never telemetry: a swap out of range, a failed
certificate, an unresolved name, a `CheckVerdict::NotValidated` carrying a *reason* and a *fallback* are
all explicit `Option`/error/refusal values that **propagate** (G2 never-silent). RFC-0013 added a way to
**present** that structured error (graded, dual-format, reified policy). It deliberately stopped at
presentation and **deferred recovery** (DN04-Q1) — because recovery edges into control flow and could,
done carelessly, become the substitutive black box the project forbids.

This RFC designs that deferred half: a way for a developer to **declaratively recover from — and act on —
errors**, so an error can *trigger functionality* (fallback, retry, cleanup, branch selection, escalation,
and novel uses), built as an **isolated, separable subsystem** with a **bounded blast radius**. Its
governing discipline, per the maintainer (2026-06-16):

> Errors propagate / bubble and can **trigger functionality**. Effects — including cascades — are
> **allowed when they are explicitly declared and implemented**, so they stay **known and bounded**. The
> enemy is *unintended / unknown / unbounded* effects (a memory explosion, a runaway cascade, spooky
> action at a distance), **not** effects per se. The default is **tightly scoped, bounded** effects; a
> developer **opts into broader effect/cascade behaviour by explicitly declaring and implementing it** —
> it never arrives implicitly. Recovery is always **additive over** the explicit error: it may act on the
> error and produce a *new explicit outcome*, but it may never make the original refusal vanish unobserved
> (G2).

The design rests on three pillars — **errors-as-propagating-values** (the substrate, RFC-0001), **explicit
declarative recovery** (the control: an explicit handling site + a reified RFC-0005-pattern recovery
policy), and **declared, bounded effects** (the safety: every potentially-unbounded effect carries an
explicit budget and overruns gracefully, generalising the fuel clock). It is **separable from RFC-0013**
(presentation vs. recovery) and keeps the trusted kernel small (KC-3): recovery elaborates to existing L0
where possible (a `Match` on an error sum), and the only runtime addition is **effect-budget enforcement**,
which lives where fuel and depth budgets already live (RFC-0004/0008, DN-05) — not in the kernel calculus.

## 2. Motivation

- **It completes the error story honestly.** RFC-0001/0013 made errors explicit and presentable; without
  recovery, the only responses are "propagate to the top" or "match by hand everywhere." Mycelium needs a
  *declarative, reified* way to act on errors that stays inside the never-silent contract (G2).
- **It is the maintainer's signature requirement, recorded.** Errors must be usable for everything errors
  are used for today (fallback, retry, cleanup, control branching) **and** novel uses — but the language
  must stay *maintainable, stable, and easy to use*, with **no unintended/unknown/unbounded side effects**.
  That tension — expressive error-driven control vs. no spooky cascades — is exactly what this RFC resolves
  by making effects explicit, declared, and budgeted.
- **It separates two concerns cleanly (SoC).** A *diagnostic policy* (RFC-0013) changes how an error is
  *shown*; a *recovery policy* (this RFC) changes what *happens* on an error. Keeping them in two RFCs /
  two subsystems means neither can destabilise the other, and a recovery feature can never silently weaken
  RFC-0013's "additive presentation" invariant.
- **It reuses Mycelium's own safety idiom.** The kernel already refuses to hang: every `Fix`/`FixGroup`
  unfold is clocked (RFC-0007 §4.5) and the control stack has a depth ceiling (M-347, DN-05), each overrun
  an *explicit, graceful* error. Bounded effects are the same idea applied to recovery: a retry has a max
  count, a cascade a depth bound, an allocation a ceiling — overrun is `EffectBudgetExhausted`, never OOM.

## 3. Guide-level explanation

> **Syntax below is illustrative, not normative.** Concrete surface syntax is KC-2-gated (RFC-0006 §10);
> this RFC fixes the *semantics and the discipline*, not the spelling.

### 3.1 Errors are values that bubble up

A fallible operation yields an explicit result value — a sum of success or a structured error (the
RFC-0001 `Option`/error/refusal; `Err` carries a reason and, where available, a fallback). By default an
error **propagates** (bubbles) to the caller; nothing is silently swallowed (G2). An explicit propagation
form (a `?`-style operator or an explicit match) makes "pass this error up" visible at the use site —
never an invisible unwinding:

```text
let v = parse(bytes)?        -- on Err, the error bubbles to this function's caller (explicit `?`)
```

### 3.2 Recovery happens at an explicit site

To *act on* an error instead of propagating it, a developer writes an **explicit handling site** that
matches the error and produces a new explicit outcome — a recovered value, or a transformed/re-raised
error. This is just pattern-matching on the error sum at a visible site (it elaborates to an L0 `Match`):

```text
handle parse(bytes) {
  Ok(v)                 => v,
  Err(Truncated)        => default_record,          -- recover with a fallback value
  Err(e)                => reraise(annotate(e)),     -- transform and re-propagate (still explicit)
}
```

A handling site must cover the error's cases or re-propagate the rest — it can **never** drop an unmatched
error (never-silent). A fallback value carries an **honest guarantee tag** (e.g. `Declared`), never a
fabricated one (VR-5).

### 3.3 Reusable recovery is a reified policy

For recovery that recurs across definitions, a **reified recovery policy** (the RFC-0005 pattern, like
RFC-0013's diagnostic policy) attaches a *named, content-addressed, `EXPLAIN`-able* recovery action to a
definition or scope:

```text
on Truncated  => fallback(default_record)            -- a declared recovery action
on Timeout    => retry(max_attempts: 3)              -- a BOUNDED retry (see §4.5)
on Fatal      => cleanup_then_propagate(release)     -- act, then let the error continue (additive)
```

The action is explicit. If a policy does not fully recover (e.g. retries exhausted), the error **continues
to propagate** — a policy is additive over the error, never a silent terminator.

### 3.4 Effects are declared and bounded

Recovery actions can *do things* (retry, allocate, clean up, cascade). Those are **effects**, and Mycelium
requires them to be **declared** and **bounded**:

- **Declared:** a definition that performs an effect names it in its signature, so a caller can see what it
  can do — there are **no undeclared effects** (no unknown side effects).
- **Bounded:** any effect that could be unbounded carries an explicit **budget** (a retry's `max_attempts`,
  a cascade's `max_depth`, an allocation's ceiling). Exceeding the budget is an **explicit, graceful error**
  (`EffectBudgetExhausted`), never a hang or an out-of-memory abort.
- **Tightly scoped by default; broader is opt-in.** The default effect scope is the narrowest. A developer
  who *wants* a broader effect or a cascade **declares and implements it explicitly** — it never arrives by
  default.

```text
fn save(r: Record) -> Result<Unit> !{ retry(<=3), alloc(<=64KiB) } = ...   -- declared, bounded effects
```

Reading rules a developer internalises: *a recovery never silently swallows an error; an effect a function
can perform is visible in its signature; every effect that could run away is budgeted, and a budget overrun
is an explicit error; broader/cascading effects are something you opt into by declaring them, never a
surprise.*

## 4. Reference-level design (normative once Accepted)

### 4.1 The substrate: errors are explicit, propagating values (G2; RFC-0001)

This RFC adds **no new error representation**: it builds on RFC-0001's explicit `Option`/error/refusal
values and `CheckVerdict::NotValidated` (reason + fallback). A fallible computation yields a **result sum**
`Ok(τ) | Err(ε)` where `ε` is a structured error value (carrying a class, a reason, optionally a fallback —
the same structured error RFC-0013 *presents*). Errors **propagate by default**; an explicit propagation
form surfaces "bubble this up" at the use site. There is no implicit, invisible unwinding.

### 4.2 The governing invariant — recovery is additive, never silent (G2)

**(I1) A handler acts on an error and produces a new explicit outcome; it never makes the error vanish
unobserved.** For every error `e`, a recovery construct either (a) **recovers** — yields an explicit
success value, where the original `e` is *consumed by an explicit, total-over-its-cases match* — or (b)
**re-propagates** — yields an explicit error (possibly `e` transformed/annotated). There is **no handler,
policy, or effect that can cause `e` to neither surface nor be explicitly recovered.** A handling site that
does not cover all cases re-propagates the remainder; it cannot drop it. This is the operational form of
never-silent (G2) for recovery, and the normative core the §5 verification defends.

**(I2) Recovery never fabricates or upgrades a guarantee (VR-5).** A recovered/fallback value carries an
honest guarantee on the lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`; a substituted fallback is at most
`Declared` (flagged) unless it has an independent checked basis. Recovery may only **downgrade** to stay
honest, never upgrade.

### 4.3 Explicit handling sites (KC-3: no new kernel node)

The explicit handling site (§3.2) is **surface sugar that elaborates to existing L0**: a `handle e { … }`
is a `Match` on the result-sum value (RFC-0001 r3 `Match` + the data registry), with the `Err` arms
producing recovered values or re-raised errors. **No new L0 node is required** for explicit handling — it
is `Construct`/`Match` over the error sum, exactly the fragment the three-way differential already covers
(NFR-7). This keeps the trusted kernel untouched (KC-3) and recovery *inspectable* (it is ordinary matching,
`EXPLAIN`-able like any term).

### 4.4 Reified recovery policies (RFC-0005 pattern; ADR-006)

A recovery policy is a **reified, inspectable, content-addressed artifact**, in the posture ADR-006/RFC-0005
mandate (and RFC-0013 reuses for diagnostics):

```text
on <ErrorClass> => <RecoveryAction>
```

- **`<ErrorClass>`** resolves through the **shared error-class registry** (RFC-0013 §4.5) — a name in a
  known set, **never an evaluated string** (the RFC-0013 X1 exclusion applies equally here). An unknown
  class is an explicit configuration error.
- **`<RecoveryAction>`** is one of a closed, declared set (v0): `fallback(value)` (recover with an honest
  fallback, §4.2 I2); `retry(<=N)` (re-attempt, **bounded**, §4.5); `escalate(class')` (re-propagate as a
  transformed error); `cleanup_then_propagate(effect)` (run a bounded effect, then let the error continue —
  additive). Each action is explicit; none may silently terminate an error (I1).

A policy is **content-addressed and `EXPLAIN`-able**: every recovered or re-propagated outcome records the
`PolicyRef` of the policy that shaped it (RFC-0005 §3), so one can always answer *"which policy acted on
this error, and what does it do?"*. A recovery policy is a **strict superset of permission** over an
RFC-0013 diagnostic policy on the same error — but the two are **separate artifacts** (§4.9): a diagnostic
policy can never change control flow, and a recovery policy's presentation is delegated to RFC-0013.

### 4.5 Declared, bounded effects (the safety discipline)

This is the heart of the RFC and the maintainer's central constraint.

**(I3) Effects are declared.** A definition that performs an effect declares it in an **effect annotation**
on its signature — a set drawn from a closed kernel of effect *kinds* (e.g. `retry`, `alloc`, `io`,
`cascade`) plus user-declared effect names. A caller of a definition sees its declared effects; **a
definition may not perform an undeclared effect** (checked above the kernel). This is the "no unknown side
effects" guarantee: effects are visible, not ambient. (v0 is **coarse** — a declared effect *set*, not full
effect-row polymorphism; richer effect typing is deferred, §8/§9 — KISS/YAGNI.)

**(I4) Effects that can be unbounded are budgeted, and an overrun is explicit and graceful.** Every effect
whose cost is not statically bounded carries an explicit **budget** — a `retry` carries `max_attempts`, a
`cascade` carries `max_depth`, an `alloc` carries a memory ceiling, a `time`-bearing effect a fuel-style
clock. Exceeding a budget yields an **explicit `EffectBudgetExhausted`** value (a structured error, subject
to §4.2 I1) — **never** a hang, a stack overflow, or an out-of-memory abort. This is the direct
generalisation of the established budget discipline: the `Fix`/`FixGroup` fuel clock (RFC-0007 §4.5), the
control-stack depth ceiling (M-347), and the dynamically-resolved budgets of DN-05 §2.4. Budgets are
themselves **`EXPLAIN`-able** and may be resolved by policy (a static default or a memory-derived ceiling,
DN-05), never hidden.

**(I5) The default is tightly scoped; broader effects are opt-in by explicit declaration + implementation.**
The narrowest effect scope is the default. A broader or cascading effect is permitted **only** when a
developer **explicitly declares and implements it** (its declaration *and* its bound). A cascade is allowed
*iff* it is declared with a `max_depth` (or equivalent) bound — so cascades are *known and bounded*, never
unbounded. There is no path by which an effect or cascade arrives implicitly.

### 4.6 Effects are reified & inspectable (no black box)

An effect declaration, its budget, and its handling policy are **reified, content-addressed artifacts** —
`EXPLAIN` can always answer *"what effects can this code perform, with what bounds, handled by what
policy?"*. This is the SC-3 / no-black-box stance applied to effects: control flow that *does something* is
never opaque or ambient; it is a declaration you can read, diff, and trace.

### 4.7 Totality & honesty interaction (matured ⟹ total under budgets)

- **Totality (RFC-0004 §4 / RFC-0007 §4.5).** Because every effect that could diverge is budgeted (I4), a
  recovering, bounded-effect definition **terminates**: it recovers, re-propagates, or hits a budget and
  yields `EffectBudgetExhausted`. The structural totality checker (outside the kernel) accounts for declared
  effects and their budgets when classifying `total`/`partial`; only `total` definitions may be `matured`
  (promoted stable). A mis-classification gates packaging, **never meaning** — the runtime clocks effects
  exactly as it clocks `Fix` (the same gate-not-meaning discipline).
- **Honesty (VR-5).** Per I2, recovery never fabricates or upgrades a guarantee; a fallback is honestly
  tagged. The effect/recovery subsystem *reports* bounds and outcomes; it never launders a weaker guarantee
  into a stronger one.

### 4.8 Isolation & implementation boundary (separable; KC-3)

- **A separable subsystem with a bounded blast radius (SoC).** Recovery and effects are **not** woven into
  the kernel or into RFC-0013's renderer. Errors propagate through RFC-0001's existing value mechanism;
  explicit handling elaborates to L0 `Match` (§4.3 — no new kernel node); reified policies are RFC-0005
  artifacts. The **only** runtime addition is **effect-budget enforcement**, which lives where fuel/depth
  budgets already live (RFC-0004/0008 execution, DN-05 budget resolution) — *outside* the trusted kernel
  calculus. So adding (or later changing) recovery cannot destabilise the kernel, the differential, or the
  diagnostics layer.
- **Design goal: minimal/no new L0 node.** v0 aims for **zero** new L0 nodes (recovery = error sums +
  `Match` + runtime budget policy). Whether effect-budget enforcement needs *any* kernel-visible hook, or is
  entirely a runtime/checker concern, is an explicit open question (§8) — the bias is to keep it out of the
  kernel (KC-3), mirroring how the totality checker lives outside the trusted base.

### 4.9 Relationship to RFC-0013 (presentation vs. recovery)

RFC-0013 and RFC-0014 are deliberately **two subsystems over one error**:

- **RFC-0013 presents** an error (graded levels, dual human/JSON projection, message/tags/route) — and
  **never changes control flow** (its I1: additive presentation).
- **RFC-0014 recovers from / acts on** an error (fallback, retry, escalate, cleanup) — explicitly and
  boundedly.
- They **share** the error-class registry (RFC-0013 §4.5) and the reified-policy pattern (RFC-0005), but a
  diagnostic policy and a recovery policy are **distinct artifacts**. A recovery outcome is *presented* via
  RFC-0013; RFC-0014 does not re-implement presentation. **RFC-0014 does not weaken RFC-0013's I1** — it
  generalises "additive" from *presentation* to *control*: a handler acts explicitly and either recovers or
  re-propagates, but the error never silently vanishes.

## 5. Verification (per CONTRIBUTING — which FR/NFR/VR/SC, and how)

This RFC advances **SC-3** (transparent, non-black-box control), **G2** (never-silent), **VR-5** (honest
guarantees), and **NFR-2 / SC-5b** (semantic feedback — recovery acts on the same structured error the
feedback loop consumes). When the subsystem lands, the invariants I1–I5 are verified by:

- **Never-silent recovery invariant (I1) — the central test.** A property test asserting that for a corpus
  of errors and *any* recovery policy/handler, the error is **either explicitly recovered (consumed by a
  total-over-its-cases match) or re-propagated** — never dropped. A mutant handler that silently discards an
  unmatched error is caught.
- **Bounded-overrun-is-explicit test (I4).** For each budgeted effect (retry/cascade/alloc/time), exceeding
  the budget yields an explicit `EffectBudgetExhausted` (a structured error), never a hang, stack overflow,
  or OOM — the analogue of the existing `FuelExhausted`/`DepthLimit` tests (RFC-0007 §4.5, M-347, DN-05).
- **No-undeclared-effect test (I3).** A definition performing an effect absent from its signature is an
  explicit checker error; a caller's view of callee effects is exact.
- **Honest-guarantee test (I2/VR-5).** A fallback/recovered value carries an honest (downgrade-only)
  guarantee; recovery never upgrades a bound.
- **Totality-under-budgets test (§4.7).** A bounded-effect recovering definition classifies `total`; the
  gate is packaging, not meaning.
- **Three-way differential where it touches L0 (NFR-7).** Explicit handling, elaborating to `Match` over
  error sums, runs on the L1-eval ≡ elaborate→L0-interp ≡ AOT differential like any data/recursion program.

## 6. Drawbacks

- **It is the riskiest surface in the project** — control flow over errors is exactly where a black box
  could creep in. Mitigated by making I1 (never-silent) and I3–I5 (declared/bounded/opt-in) **normative
  invariants**, by keeping the subsystem **separable** (§4.8) so a mistake is contained, and by reusing the
  proven budget discipline rather than inventing new unbounded machinery.
- **An effect-annotation system is real surface and real cognitive load** (KISS/YAGNI cost). Mitigated by a
  **coarse** v0 (a declared effect *set*, manual not inferred) with richer effect typing explicitly deferred
  (§8/§9), and by the payoff: no unknown/unbounded effects.
- **Declared effects can be verbose** (every effectful function annotates). Accepted as the price of "no
  unknown side effects"; ergonomic sugar (effect aliases, defaulting) is a future possibility (§9), never a
  way to *hide* an effect.
- **Two policy kinds (diagnostic + recovery) over one error** could confuse. Mitigated by the strict §4.9
  split and by `EXPLAIN` always naming which policy did what.

## 7. Prior art

> These are **design inspirations**, recorded honestly; they are **not yet traced to the research corpus**
> (`research/`). Folding them into the evidence base (or refuting them) is an explicit task before
> ratification (§8) — this Draft does not claim them as established Mycelium grounding.

- **Result/`?` error values** (Rust, Swift, Go) — errors as explicit, propagating values matched at explicit
  sites; the substrate posture (§4.1/§4.3). Mycelium's never-silent rule is stricter (no silent drop).
- **Algebraic effects & handlers** (Koka, Eff, OCaml 5) — effects as a typed, handled capability; the
  inspiration for *declared, reified* effects (§4.5/§4.6). v0 is far simpler (coarse declared set, bounded
  actions), with full effect handlers/rows as a possible future (§9).
- **Erlang/OTP supervision** — *bounded* restart strategies (max-restart-intensity over a window): the
  canonical example of a **declared, bounded cascade** (a restart storm is capped, not unbounded). Direct
  grounding for I4/I5 (bounded cascades).
- **Structured concurrency cancellation** (nurseries / scopes) — effects and failures bounded to an explicit
  scope; the inspiration for "tightly scoped by default" (I5). Relevant once RFC-0008 concurrency lands.
- **Capability-based effect control** — effects available only where a capability is passed; an alternative
  to annotations for "no ambient effects," noted for the §8 effect-mechanism question.
- **Mycelium's own budget idiom** (RFC-0007 §4.5 fuel; M-347 depth; DN-05 resolved budgets) — the in-repo
  precedent that bounded effects directly generalise (the strongest, already-grounded basis).

## 8. Unresolved questions

- **Effect mechanism (genuinely open).** Declared effect *annotations* (rows/sets on signatures) vs.
  *capabilities* (effect granted by a passed token) vs. a hybrid. v0 leans annotations (coarse set), but the
  choice is open and shapes the surface (RFC-0006) — must be decided before ratification.
- **Any kernel-visible hook? (§4.8).** Whether effect-budget enforcement is *entirely* a runtime/checker
  concern (the KC-3-preferred bias — **zero** new L0 nodes) or needs a kernel-visible marker. Open; the
  design goal is none.
- **Budget vocabulary & composition.** The concrete set of budgets (`max_attempts`, `max_depth`, memory
  ceiling, time/fuel) and how they **compose** with the existing `Fix` fuel clock and M-347 depth ceiling
  (one budget space or several?) — left to the RFC-0004/0008 + DN-05 integration.
- **Effect inference vs. manual declaration.** v0 = manual (explicit is honest); whether to infer/propagate
  declared effects (ergonomics) without letting an effect become implicit is open (§9).
- **Recovery-action set.** The closed v0 set (`fallback`/`retry`/`escalate`/`cleanup_then_propagate`) — is
  it complete? Are user-defined recovery actions admitted, and if so how do they stay bounded + never-silent?
- **Concurrency interaction (RFC-0008).** How effects, budgets, and recovery compose across tasks
  (cancellation, failure propagation, per-task budgets) — deferred to the RFC-0008 integration.
- **Handler composition & re-entrancy.** Nested handlers, cascade ordering, and whether a handler may itself
  be effectful (and thus budgeted) — open; the bias is that a handler's effects are declared + bounded like
  any other.
- **Research grounding (§7).** Tracing the prior art into `research/` (or refuting it) before Accepted.

## 9. Future possibilities

- **Richer effect typing** — effect rows / polymorphism / inference, *only* if it never lets an effect
  become implicit or unbounded (would extend, not weaken, I3–I5).
- **User-defined recovery actions & effect kinds** — an extensible, still-reified, still-bounded set.
- **Capability-passing effects** — effects granted by explicit capability tokens (no-ambient-effects via
  capabilities), as an alternative or complement to annotations.
- **Effect/recovery ergonomics** — effect aliases, scoped defaults, and `?`-chaining sugar that reduce
  verbosity *without* hiding an effect or a budget.
- **Stdlib `result`/`effect` module** (M-346) — the recovery combinators and the standard error/effect kinds
  as a self-contained, dogfoodable core-library citizen.
- **Self-hosting** — the recovery/effect runtime eventually written in Mycelium-lang, consuming its own
  bounded-effect machinery.

## Meta — changelog

- **2026-06-16 — Draft (Proposed).** Created at the maintainer's request to capture the **declarative error
  recovery & bounded-effects** subsystem that RFC-0013 §8/§9 deferred (the DN04-Q1 recovery half). Designs
  three pillars: **errors-as-propagating-values** (the RFC-0001 substrate; G2), **explicit declarative
  recovery** (an explicit handling site that elaborates to L0 `Match` — KC-3, no new kernel node — plus a
  reified RFC-0005-pattern recovery policy), and **declared, bounded effects** (effects named on signatures,
  every unbounded effect budgeted with an *explicit, graceful* `EffectBudgetExhausted` overrun — the direct
  generalisation of the `Fix`/`FixGroup` fuel clock, M-347 depth ceiling, and DN-05 budgets). Records the
  maintainer's governing discipline: errors propagate/bubble and **trigger functionality**; effects and even
  cascades are allowed **when explicitly declared/implemented** so they stay *known and bounded* — the enemy
  is *unintended/unknown/unbounded* effects, not effects per se; default tightly scoped, broader opt-in by
  explicit declaration; recovery is **additive over** the explicit error (never silent — G2; never fabricates
  or upgrades a guarantee — VR-5). Defines the **isolation** boundary (a separable subsystem; budget
  enforcement lives with RFC-0004/0008/DN-05, not the kernel — KC-3) and the **RFC-0013 split** (presentation
  vs. recovery; shared registry/pattern; this RFC does not weaken RFC-0013's additive-presentation invariant).
  Prior art (Result/`?`, algebraic effects, **Erlang/OTP bounded supervision**, structured-concurrency
  cancellation, capabilities, and Mycelium's own budget idiom) is recorded as **design inspiration not yet
  traced to `research/`** — tracing it is a pre-ratification task. Advances SC-3, G2, VR-5, NFR-2/SC-5b;
  verification = a never-silent recovery invariant test (I1), a bounded-overrun-is-explicit test (I4), a
  no-undeclared-effect test (I3), an honest-guarantee test (I2/VR-5), a totality-under-budgets test, and the
  three-way differential where recovery touches L0 (NFR-7). **No code lands with this draft** — many design
  choices (the effect mechanism, the budget vocabulary, any kernel hook) are explicit open questions (§8);
  ratification (Draft → Accepted) and a tracking milestone are the maintainer's, presented here first.
  Append-only. Lineage: DN04-Q1 → RFC-0013 §8/§9 → RFC-0014.
