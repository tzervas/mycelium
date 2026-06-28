# RFC-0014 — Declarative Error Recovery & Bounded Effects

| Field | Value |
|---|---|
| **RFC** | 0014 |
| **Status** | **Enacted** (drafted 2026-06-16; ratified 2026-06-16 — maintainer sign-off; the §4 design and all §8 dispositions are normative. **Enacted 2026-06-16 — M-352** (#116): the reified recovery subsystem — the result-sum error value, the closed recovery-action set, the registry-shared `on <ErrorClass> => <action>` recovery policy, declared + budgeted effects with a graceful `EffectBudgetExhausted`, the no-undeclared-effect check, and the never-silent `handle` (every error recovered or re-propagated, never dropped) — is code in `crates/mycelium-lsp/src/recover`; the L0-`Match`-over-error-sums lowering target is differentially verified in `mycelium-l1` (no new kernel node, KC-3). Wiring effect-budget enforcement into the AOT env-machine was the RFC-0008 integration — **completed in M-353** (§4.8 changelog entry; the ledger lifted into `mycelium-interp`, an overrun is `EvalError::EffectBudget`).) **r2 (§10) — Phase-2 effect-system enrichment: Proposed (2026-06-28).** Drafted append-only from DN-60's G6 directions: surface budget syntax (D1), effect-row polymorphism + minimal-set inference (D2), hypha-creation in the effect row via Position B+ (D3). No existing v0 content altered. Ratification (Proposed → Accepted) is the maintainer's gate. |
| **Type** | Foundational / normative (once Accepted) — a **separable** surface + runtime subsystem; minimal/no kernel change (KC-3) |
| **Date** | 2026-06-16 |
| **Feeds** | RFC-0006 (the optional recovery/effect surface); RFC-0008 (runtime — where effect budgets are enforced, alongside fuel/depth); RFC-0013 (the diagnostic *presentation* of an error this RFC *acts on*); the stdlib (a `result`/`effect` module candidate, M-346) |
| **Depends on** | RFC-0001 (errors are explicit `Option`/error/refusal *values*; `CheckVerdict::NotValidated` carries reason + fallback); RFC-0013 (sibling — shared error-class registry + reified-policy pattern; this RFC is the recovery concern RFC-0013 §4.4/§8 deferred); RFC-0005 / ADR-006 (the reified, inspectable, content-addressed policy pattern); RFC-0006 (surface/term-layer, KC-2-gated syntax); RFC-0004 §4 / RFC-0007 §4.5 / M-347 / DN-05 (the **budget discipline** — fuel clock, control-stack depth ceiling, dynamically-resolved budgets — that bounded effects generalise); G2 (never-silent), VR-5 (honest, downgrade-only guarantees), KC-3 (small kernel), SC-3 (transparent control), NFR-2 / SC-5b (semantic feedback), NFR-7 (differential) |
| **Tracks** | M-352 (#116). Lineage: DN04-Q1 deferred half → RFC-0013 §8/§9 → here |

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

> **Note (2026-06-22 — append-only; M-660): the `!{…}` surface is now pinned for the L1 stage-1
> frontend.** The `!{ … }` spelling above — *illustrative* under the §3 banner — is **pinned as the
> normative L1 stage-1 effect-annotation surface** (maintainer decision, 2026-06-22) and implemented
> Rust-first in `crates/mycelium-l1/`: an **optional** `!{ eff1, eff2 }` after the return type
> (`-> ret !{…}`), **absent ⇒ pure**, effect names plain identifiers (the §4.5 kernel kinds +
> user `Named`). The v0 **coverage check** (declared ⊇ performed, where performed = the union of
> callees' declared effects — I3, §8 manual-declare + compositional-check) lands with it; an impl
> method's effects must equal the trait method's. The **budgets** in the example (`retry(<=3)`,
> `alloc(<=64KiB)`) are **not** part of the stage-1 *surface* yet — the frontend parses the **effect
> *set*** only; budget *enforcement* stays the M-353 `mycelium_interp::budget` runtime ledger, and a
> per-effect budget syntax is later work. No new L0 node (KC-3). See the Meta-changelog entry of the
> same date for the full disposition.

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

  > **Note (2026-06-18 — append-only; RFC-0017 Accepted):** "only `total` definitions may be `matured`"
  > now reads **over the matured scope** (RFC-0017 §4.2): maturation is declared at nodule/phylum/program
  > scope, so the obligation becomes — every reachable non-`thaw` definition in a matured scope must be
  > `total` *and* pass the RFC-0004 §4 stable-component checks. This is the **same obligation, universally
  > quantified** over the scope — not a new or weaker one. The budget/effect interaction and the
  > `EffectBudgetExhausted` semantics described here are **unchanged**; the totality checker still
  > classifies per-definition, and a `thaw` definition is excluded from the matured set (it carries no
  > totality obligation from maturation, and runs on the fuel-guarded interpreter as usual). The "gates
  > packaging, never meaning" discipline applies at scope granularity exactly as it did per-definition.
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
- **Design goal: minimal/no new L0 node.** v0 has **zero** new L0 nodes (recovery = error sums +
  `Match` + runtime budget policy). Whether effect-budget enforcement needs *any* kernel-visible hook is
  **resolved** (§8, maintainer 2026-06-16): **none** — it is entirely a runtime/checker concern (KC-3),
  mirroring how the totality checker lives outside the trusted base.

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

> These were recorded as **design inspirations** at drafting; **as of 2026-06-16 they are traced into
> the evidence base** — see **Research Record 05** (`research/05-error-recovery-and-bounded-effects-RECORD.md`,
> findings T5.1–T5.6), which discharges the §8 grounding obligation. The strongest claims (bounded
> cascades, the budget generalisation) rest on *verified* grounding (Erlang/OTP max-restart-intensity;
> Mycelium's own ratified fuel/depth/DN-05 budgets); the substrate and effect-reification postures are
> grounded with honest *deltas* (Mycelium's never-silent rule is stricter than Result/`?`; v0 takes only
> a coarse bounded subset of algebraic effects, not general resumable handlers).

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

- **Effect mechanism — RESOLVED (maintainer, 2026-06-16): declared annotations (coarse set).** v0 uses
  **declared effect annotations** — a coarse effect *set* on a signature (§4.5 I3) — not capabilities and
  not a hybrid. Rationale: it is the simplest mechanism that delivers "no unknown side effects" (KISS/YAGNI),
  matches the §4.5 draft, and keeps the surface (RFC-0006) small. Capabilities (passed tokens) and richer
  effect typing remain recorded **future possibilities** (§9), explicitly *additive* — they may extend, never
  weaken, I3–I5.
- **Any kernel-visible hook? — RESOLVED (maintainer, 2026-06-16): none.** Effect-budget enforcement is
  **entirely a runtime/checker concern** — **zero** new L0 nodes (KC-3). Recovery elaborates to L0 `Match`
  over error sums (§4.3); budget enforcement lives where fuel/depth budgets already live (RFC-0004/0008,
  DN-05, §4.8), *outside* the trusted kernel — mirroring how the totality checker lives outside the trusted
  base. The trusted kernel gains no effect/budget marker.
- **Budget vocabulary & composition — RESOLVED (maintainer, 2026-06-16): separate named budgets, one
  enforcement mechanism.** Each effect kind keeps its **own named, `EXPLAIN`-able budget** (`retry`'s
  `max_attempts`, `cascade`'s `max_depth`, `alloc`'s memory ceiling, a `time`-bearing effect's fuel-style
  clock — distinct vocabulary, not collapsed into one number), but **all are resolved and enforced by the
  single existing budget machinery** that already clocks the `Fix`/`FixGroup` fuel and the M-347 depth
  ceiling and resolves DN-05 dynamic budgets. So effect budgets *compose alongside* fuel + depth in the same
  runtime/DN-05 plumbing (shared mechanism), rather than coupling those established clocks into one shared
  budget abstraction. Each overrun is its own explicit, graceful `EffectBudgetExhausted` (§4.5 I4).
- **Effect inference vs. manual declaration — RESOLVED v0 (maintainer, 2026-06-16).**
  v0 is **manual declaration only** — explicit is honest, no inference (an effect set is *written* on
  the signature). To keep that from being a correctness hole rather than just verbose, the checker
  **composes declared effects as a *check*, not an inference**: a definition calling an effectful
  callee must itself declare (a superset of) the callee's effects, or it is an explicit
  `UndeclaredEffect` error (I3). That is *checking* that declared effects compose up the call graph —
  it never *infers* (synthesises) an undeclared effect, so an effect can still never become implicit.
  Computing a minimal effect set (true inference) is deferred to §9; the v0 line is **manual-declare +
  compositional-check**.
- **Recovery-action set — RESOLVED v0 (maintainer, 2026-06-16).** The v0 set is
  **closed and complete for v0**: `fallback(value)` (recover — explicit, honestly-tagged value, I2),
  `retry(<=N)` (re-attempt — bounded, I4), `escalate(class')` (transform + re-propagate), and
  `cleanup_then_propagate(effect)` (act, then let the error continue — additive). These cover the four
  canonical error-driven behaviours, and **each is provably never-silent** (every action yields either
  an explicit success or a re-propagated error — I1) **and bounded** (the only re-attempting action,
  `retry`, carries `<=N`). **User-defined recovery actions are NOT admitted in v0** (YAGNI) — they are a
  §9 future. When added, a user action is a function `Err(ε) -> Result<τ>` that **must** be total over
  the error's cases (I1) and **declare + bound** any effect it performs (I3/I4) — i.e. it inherits the
  same obligations as the built-in set; it is never a privileged escape hatch.
- **Concurrency interaction (RFC-0008) — RESOLVED in RFC-0008 §4.7 (M-356, 2026-06-16); single-task v0
  boundary now LIFTED, additively.** The deferral below fixed a *safe* single-task boundary; **RFC-0008
  §4.7** now lifts it without weakening any invariant here. The composition (designed there, enacted as
  scheduler-independent primitives in `mycelium_interp::supervise`): **per-task budgets** — each task
  instances its own M-353 ledger, so an overrun is an *in-that-task* `EffectBudgetExhausted` (I4), never
  global; **cooperative cancellation** — observed at budget-check points, an explicit *additive*
  `Cancelled` (I1), never preemptive; **cross-task failure propagation** — an explicit `TaskOutcome`
  with no silent/dropped variant, so a child failure is a value the parent must act on (I1 across the
  boundary); and **`reclaim` bounded-cascade supervision** — a restart storm bounded on **both** the
  `cascade` effect budget (I4/I5) **and** a windowed max-restart-intensity (logical clock; Erlang/OTP,
  Research Record 05 T5.3), exceeding either an explicit escalation (a declared, bounded cascade — I5),
  never unbounded. The actual task **scheduler** and the RT2 differential are RFC-0008 R1 (M-357), built
  on these primitives. *Everything stays additive over the explicit error (I1) and declared + bounded
  (I3/I4): no spooky cross-task action, no unbounded cascade.* The original (now-superseded) deferral is
  preserved below, append-only.
  - *(superseded 2026-06-16 — original deferral)* v0 recovery/effects are **single-task / synchronous**:
    budgets are per-evaluation (the same scope the `Fix` fuel clock already uses), and there is **no
    cross-task effect or cascade** in v0 (no spooky action across tasks — there are no tasks yet). The
    genuinely-open composition (per-task budgets, cancellation, cross-task failure propagation) is
    **RFC-0008's** design, and it must compose **additively**: a task failure is an explicit error
    subject to I1, a per-task budget overrun is an in-that-task `EffectBudgetExhausted`. Fixing the v0
    boundary now makes the deferral *safe* (v0 cannot accidentally admit an unbounded cross-task
    cascade) rather than merely postponed.
- **Handler composition & re-entrancy — RESOLVED v0 (maintainer, 2026-06-16).**
  - **Nesting is lexical and deterministic.** Handlers nest like `Match`: the **innermost** handling
    site whose pattern matches an error handles it; an unmatched case **re-propagates** to the next
    enclosing site (never dropped — I1). No ambiguity, no ordering surprises.
  - **A handler may itself be effectful — and is then declared + budgeted like any other code** (I3/I4):
    a handler is *not* a privileged effect-free or budget-free zone (a `cleanup_then_propagate` that
    allocates declares `alloc(<=…)`).
  - **A cascade (a handler that triggers a further error) is bounded by an explicit `cascade(max_depth)`
    budget** (I4/I5); overrun is `EffectBudgetExhausted`, never unbounded handler recursion. Cascade
    *ordering* is just the deterministic innermost-first propagation above.
  - This makes composition deterministic, never-silent, and bounded with **no machinery beyond** the
    declared + budgeted-effects discipline already in §4.5.
- **Research grounding (§7) — DONE (2026-06-16).** The prior art is traced into `research/` (Research
  Record 05, T5.1–T5.6), discharging this obligation; the externals were verified by web search and the
  in-repo budget precedent confirmed. The *remaining* open questions above are design choices, not
  grounding gaps.

## 9. Future possibilities

- **Richer effect typing** — effect rows / polymorphism / inference, *only* if it never lets an effect
  become implicit or unbounded (would extend, not weaken, I3–I5).
- **User-defined recovery actions & effect kinds** — an extensible, still-reified, still-bounded set.
- **Capability-passing effects** — effects granted by explicit capability tokens (no-ambient-effects via
  capabilities), as an alternative or complement to annotations.
- **Effect/recovery ergonomics** — effect aliases, scoped defaults, and `?`-chaining sugar that reduce
  verbosity *without* hiding an effect or a budget.
- **Automatic baseline recovery profiles** (DynEL's *automated baseline* intent) — named, **opt-in**,
  **declared + bounded** recovery profiles (e.g. `resilient`/`strict`) the automation layer applies only
  when a developer opts in, plus auto-generated handler *skeletons* — for boilerplate QoL without ever an
  implicit control-flow change (I3/I4/I5 hold; never-silent I1 preserved). Designed in **RFC-0015
  (Automatic Baseline Diagnostics & Recovery)**, which generates/applies this RFC's policies without
  weakening its invariants (append-only). Tracked M-362.
- **Stdlib `result`/`effect` module** (M-346) — the recovery combinators and the standard error/effect kinds
  as a self-contained, dogfoodable core-library citizen.
- **Self-hosting** — the recovery/effect runtime eventually written in Mycelium-lang, consuming its own
  bounded-effect machinery.

## 10. r2 — Phase-2 effect-system enrichment (Proposed, 2026-06-28)

> **Posture (VR-5 / G2 / append-only).** This section is an **additive revision** — it does not
> alter any prior v0 content (§1–§9, the Meta changelog). Every design decision it records is
> **`Declared`** (design argument, not a theorem) unless stated otherwise. Nothing here activates
> any surface, supersedes any v0 invariant (I1–I5), or advances RFC-0014's status: the revision is
> **Proposed** and awaits maintainer ratification. Grounded in DN-60 §2–§5, RFC-0008 §4.5/§6/§8
> R8-Q2, RFC-0018 §4.5. Disconfirming evidence and open tensions are flagged in each sub-section.

This revision turns the G6 group of deferred directions (DN-60, 2026-06-28) into a concrete
Proposed design. The three directions are:

| Direction | v0 locus deferred | r2 §10 design |
|---|---|---|
| **D1 — Surface budget syntax** | §3.4 / §4.8 / §8 "budget vocabulary" | §10.1 — O1 static literals first; budget composition semantics |
| **D2 — Effect-row polymorphism + minimal-set inference** | §8 "effect inference" / §9 "richer effect typing" | §10.2 — effect variables (O1 target), inference-with-shown-output |
| **D3 — Hypha-creation in the effect row** | §9 "concurrency interaction" / RFC-0008 R8-Q2 | §10.3 — Position B+ (scope-level boundary) |

**What this revision does NOT change:**

- v0 invariants I1 (never-silent recovery), I2 (honest guarantees), I3 (no undeclared effects),
  I4 (budget overrun is explicit and graceful), and I5 (default tightly scoped; broader opt-in) are
  **unchanged and non-negotiable**. Every r2 direction must extend, never weaken, these invariants.
- The v0 `!{…}` surface (M-660), the runtime budget ledger (M-353, `mycelium_interp::budget`), and
  the compositional coverage check (M-660) remain the live, active mechanism. r2 adds **on top of**
  them.
- KC-3 applies: effect rows and inference are checker/surface metadata — **zero new L0 nodes**.
  Any direction that would require L0 support must be explicitly justified as a KC-3 exception
  through the kernel-freeze gate (DN-56 §4, condition #3 / ADR-033 precedent).

---

### 10.1 D1 — Surface budget syntax for per-effect bounds (Proposed)

#### 10.1.1 Motivation and current state

The v0 effect system declares effect *kinds* (`retry`, `alloc`, `io`, `cascade`, `time`,
user-declared `Named`) but does not yet place budget bounds in the signature surface. The
illustrative form in §3.4:

```text
fn save(r: Record) -> Result<Unit> !{ retry(<=3), alloc(<=64KiB) } = ...
```

— is not yet parsed by the M-660 frontend (§3.4 append-only note, 2026-06-22 records this
explicitly). The runtime budget ledger (M-353 `mycelium_interp::budget`) enforces bounds
dynamically; the *caller* cannot statically see *how bounded* a callee's effects are. This is an
I3-completeness gap: the "no undeclared effects" check verifies kind but not magnitude (`Declared`).

#### 10.1.2 Design decision — O1 (static literal bounds) first

**Adopted (Proposed): Option O1 — static literal bounds.** The budget bound is a compile-time
literal in the effect annotation:

```text
fn save(r: Record) -> Result<Unit> !{ retry(<=3), alloc(<=64KiB) } = ...
fn process(data: Data) -> Result<Unit> !{ io, cascade(<=2) } = ...
```

- `retry(<=N)` — the maximum number of attempts; `N` is a non-negative integer literal.
- `alloc(<=B)` — memory ceiling; `B` is a byte-count literal (e.g. `64KiB`, `1MiB`).
- `cascade(<=D)` — maximum handler-cascade depth; `D` is a non-negative integer literal.
- `time` and `io` carry no static literal bound (their quantities are not statically knowable);
  a future budget parameter form (O2, §10.1.4) may add them.
- A bare effect name without a bound (`retry` without `<=N`) is an **error** if that kind
  requires a bound — consistent with I4 (every potentially-unbounded effect must be bounded).

A missing bound where one is required is a `CheckError` naming the effect kind and the missing
bound, never silent (G2). `Declared` — the structural budget-coverage check is declared, not proven.

**Rationale (KISS/YAGNI, DN-60 §2.2):** O1 is the simplest mechanism that closes the
I3-completeness gap and delivers the SC-3 transparency property (a caller can see how bounded a
callee is). O2 (budget parameters) adds significant signature surface and is deferred to a
follow-on increment. O3 (policy-reference budgets) is a §9-class future.

**Disconfirming tension:** O1 cannot express DN-05-style dynamic resolution (a budget derived from
a runtime signal, e.g. available memory). The v0 runtime ledger already supports this; O1 simply
does not surface it statically. A developer who needs dynamic budget resolution today uses the
runtime API directly (declared as an effect without a static bound — honesty requires marking it
`Declared`). O2 would unlock static dynamic resolution; this revision defers it.

#### 10.1.3 Budget composition semantics (Proposed)

v0 checks that a caller's declared effects are a superset of its callees' effects (I3). With
bounds in the surface, the composition rule must specify what "superset" means for a bounded effect:

**Rule (Proposed):** When a fn `f` calls `g !{ retry(<=M) }` and `h !{ retry(<=N) }`, `f`'s
declared retry bound must satisfy:

```
f's declared retry(<=K) where K >= max(M, N)
```

The **outer bound must dominate** (be at least as large as) the largest callee bound. The
rationale: `f` can call `g` *or* `h` (or both, sequentially) — the worst case is whichever is
larger. Summing bounds (K >= M + N) is a conservative option but over-approximates for a fn that
calls `g` *or* `h`, not both simultaneously; the max-dominance rule is the tightest sound choice
for a single-pass checker without call-graph reachability. `Declared` — this is a design argument,
not a mechanized proof.

**Open tension (flagged for ratification):** For a fn that calls `g` and `h` *sequentially*
(both will execute), the max rule under-approximates the total budget consumed. Whether the Phase-2
RFC adopts max-dominance or call-graph-path-sum requires a concrete operational semantics decision.
The budget ledger deducts per-invocation at runtime (M-353); the static annotation is a *contract*
(upper bound on what can be consumed in one top-level invocation), not an exact count. This
distinction must be made explicit in the ratification. `Declared` direction; `Empirical` grounding
minimum for ratification (DN-60 §5 DoD item 1).

**Per-task boundary (RFC-0008 §4.7 C1, DN-60 §2.3, Proposed):** A fn-level budget annotation is
a **static property of the fn's body**; the per-hypha budget ledger (RFC-0008 §4.7 C1) is a
**dynamic property of the created task**. These remain **separate concerns** (DN-60 §2.3 direction):
the spawn form receives a per-hypha budget as a runtime argument or scope default, not conflated
with the spawning fn's own effect signature. The fn's `!{ … }` annotation does not propagate to
the spawned hypha's ledger; the hypha's ledger is configured at spawn time. `Declared`.

#### 10.1.4 O2 (budget parameters) — deferred follow-on

O2 (budget parameters in fn signatures, e.g. `fn f<const N: usize>() -> T !{ retry(<=N) }`)
is **explicitly deferred**: it requires extending fn signatures with budget parameters and deciding
how they interact with the M-660 trait/impl coverage check (impl-must-equal-trait for effects;
a budget parameter would require parametric trait methods). O2 is the natural next increment if
DN-05-style surface resolution is needed; it is not adopted here (YAGNI).

---

### 10.2 D2 — Effect-row polymorphism and minimal-set inference (Proposed)

#### 10.2.1 Motivation

v0 higher-order functions (HOFs) must conservatively **over-declare** the effects of their
function arguments, or restrict arguments to pure functions. Example:

```text
-- v0: over-declares; a pure argument triggers a spurious io check
fn map<T, U>(f: fn(T) -> U !{ io }, list: List<T>) -> List<U> !{ io } = ...
```

Without effect-row polymorphism a HOF cannot express "I carry whatever effects my argument
carries" — every stdlib combinator must pick a concrete effect set or pure. This blocks
effect-polymorphic stdlib APIs (the G6 "unblocks" in the Ratification Map).

#### 10.2.2 D2a — Effect variables (Proposed, direction O1)

**Target direction (Proposed): O1 — effect variables.** A function may be universally quantified
over an effect row `!E`:

```text
fn map<T, U, !E>(f: fn(T) -> U !{!E}, list: List<T>) -> List<U> !{!E} = ...
```

`!E` is an **effect row variable** — it stands for any set of effects. At each call site,
`!E` is instantiated to the concrete effects of the argument:

```text
map(pure_fn, xs)          -- !E = {} (pure); map's declared effect = {}
map(io_fn, xs)            -- !E = {io}; map's declared effect = {io}
map(retry3_fn, xs)        -- !E = {retry(<=3)}; map's declared effect = {retry(<=3)}
```

**Extension without weakening I3–I5 (obligation):** Effect variables must not let an effect
become implicit. Instantiation is always explicit at each call site (the checker infers the
instantiation from the argument's declared effect set — it is not hidden). An uninstantiated `!E`
defaults to the **empty set** (pure), not to an unconstrained open set. `Declared` direction.

**KC-3 check (Proposed):** Effect variables are **checker metadata** — they remain in the type
checker and the `!{…}` surface; they do not require new L0 nodes. The L0 term language sees only
the concrete (instantiated) effect sets after elaboration. `Declared` — confirmed by the v0
design: the M-660 coverage check does not lower effects to L0, and this extension would not
change that. If any L0 node is needed, it must go through the kernel-freeze gate (ADR-033
precedent) — that is an explicit stop condition for this revision.

**O2 (bounded effect quantification) as a simpler fallback (Proposed):** If O1's implementation
cost or interaction with the G4 (RFC-0019) polymorphism surface proves prohibitive, O2 —
bounded row quantification `!{!E, io}` (at least `io`, polymorphic over `!E`) — is an acceptable
simpler alternative. O3 (named effect groups) is a last-resort fallback with limited HOF
ergonomics. The Phase-2 RFC must decide which option and justify it. `Declared`.

**Disconfirming tension (flagged):** RFC-0019 (traits and parametric polymorphism) is the G4
group (a separate ratification cluster). Adding effect variables before G4 is settled risks
surface-level inconsistency between value type parameters and effect row parameters. The Phase-2
RFC must either (a) coordinate with RFC-0019 ratification, or (b) explicitly scope effect
variables as syntactically distinct from value type parameters and justify the separation.
`Declared` — open sub-question for the maintainer.

#### 10.2.3 RFC-0018 §4.5 Design A soundness precondition — mandatory obligation

RFC-0018 §4.5 (ratified, Design A) records a **load-bearing precondition** (`research/09` T9.6):

> *"Design A's sufficiency rests on the calculus being pure; when observable effects land, they
> must become graded outputs (RFC-0014, route i) or carry a local `pc` (route ii)."*

**Route i (Proposed):** Observable effects that produce results visible to the caller must become
**graded outputs** — the result type carries the effect's guarantee grade (via meet, RFC-0018
§4.1). This is the existing mechanism: `TaskOutcome` (RFC-0008 §4.7 C3) is already a graded
output for concurrency; the same route applies to effect-row-polymorphic operations. A fn
`!{!E}` whose argument `f` has effects produces a result whose grade is the meet of the input
grades and the effect row's observable grade.

**Why route ii is not adopted (Proposed):** Route ii (a local `pc` at each effect site) was
considered but rejected: it reintroduces the `pc` index at effect boundaries, significantly
increasing annotation burden and checker complexity (the same objection that led to ratifying
Design A over Design B for grading — RFC-0018 §4.5). Route i is the simpler, existing mechanism.
`Declared`.

**Obligation for ratification (non-negotiable, DN-60 §3.2):** This precondition discharge
**cannot** be `Declared` in the Phase-2 RFC. It requires at minimum `Empirical` grounds
(a worked-out argument for how effect-row polymorphism satisfies route i, with distinguishing
examples). A `Proven` tag (mechanized) is the ideal; `Empirical` (worked argument + examples)
is the minimum for ratifiability. This is a stop condition: a Phase-2 RFC that marks this
precondition `Declared` is not ratifiable without an upgrade to the grounding.

#### 10.2.4 D2b — Minimal-set inference (Proposed)

**Adopted direction (Proposed): inference-with-shown-output.** The checker may *infer* the
minimal effect set from the call graph (the union of all reachable callee effects), but the
inferred set is **always shown explicitly** — on inspect, explain, and hover — and the developer
is never surprised by an implicit effect. An undeclared effect is never silently permitted (I3).

- **Explicit manual annotations remain legal.** A developer who writes `!{ io, retry(<=3) }`
  explicitly is not forced to remove it if inference agrees or over-approximates. Manual
  over-declaration continues to be legal (I5: opt-in broader effects).
- **When inference disagrees with a manual annotation:**
  - If the inferred set is a **subset** of the manual annotation, the manual annotation stands
    (it is a legal over-declaration per I5). The checker may emit an advisory (non-error) note
    that the annotation is broader than required.
  - If the inferred set **exceeds** the manual annotation (inference finds an undeclared effect),
    the checker emits a `CheckError` — exactly as in v0. The developer must either add the effect
    to the annotation or fix the call graph.
- **Completeness tag obligation:** The Phase-2 RFC must specify the inference algorithm and tag
  its completeness claim. A flow-analysis over the call graph is `Empirical`; a proven type
  inference algorithm with completeness theorem is `Proven`. The revision cannot claim `Proven`
  for an algorithm that is not mechanized. `Declared` direction; `Empirical` minimum for
  ratification (DN-60 §5 DoD item 3).

**Disconfirming tension (flagged):** Inference-with-shown-output weakens the "I wrote what my fn
does" property (RFC-0014 §8 explicit reasoning: "explicit is honest"). An inferred annotation
that the developer did not write is honest if always surfaced, but it shifts the annotation
source from "developer intent" to "checker derivation." The Phase-2 RFC should explicitly
address this tension and state whether inference is opt-in (per-fn or per-nodule) or the
default. This revision does not settle that question — it defers it to the maintainer.

---

### 10.3 D3 — Hypha-creation in the effect row (Proposed)

#### 10.3.1 The open question (RFC-0008 R8-Q2)

RFC-0008 §8 R8-Q2 asks: does hypha creation (spawning a hypha via the structured concurrency
form) appear in the effect row of a function's signature?

The three positions (Position A: `spawn` as an effect kind; Position B: RT7 structure alone;
Position B+: scope-level boundary — described in full in DN-60 §4.2).

#### 10.3.2 Adopted design — Position B+ (scope-level boundary) (Proposed)

**Adopted direction (Proposed): Position B+.** Hypha-creation is **not** a per-fn effect kind
that propagates through the `!{…}` row of individual functions. Instead, the structured
concurrency scope (a `colony { … }` block, DN-06/RFC-0008 §4.7, or an equivalent scope-level
construct) is the **syntactic boundary** that enables spawning within it.

Under B+:

1. **`spawn` does not appear in the `!{…}` effect row.** Individual functions that create hyphae
   do not carry `spawn` in their signature. The spawning capability is granted by the enclosing
   scope, not per-fn.

2. **The structured scope is the spawning boundary.** A `colony { … }` block (or equivalent
   future construct) syntactically declares that hypha creation may occur within it. Code outside
   a structured scope cannot spawn — this is the RT7 structural invariant (RFC-0008 §4.1),
   unchanged.

3. **Rationale (Proposed, `Declared`):**
   - Position A (`spawn` as a per-fn effect) would require `spawn` to propagate up the entire
     call graph above any spawning function — the verbosity problem (§6 "declared effects can be
     verbose"), amplified for deeply-nested call graphs. RT7 already provides the structural
     invariant; per-fn `spawn` is redundant.
   - Position B (RT7 alone, no annotation) under-communicates: a caller cannot see from a fn's
     signature whether it may spawn. B+ is a middle path: the spawning *context* is explicit at
     the scope level, which is where it matters to a reader.
   - KC-3: B+ requires no new L0 node. The `colony` block is already designed (DN-06/RFC-0008
     §4.7); B+ adds only a checker rule ("spawning outside a structured scope is a `CheckError`").

4. **RFC-0018 §4.5 Design A precondition — already satisfied (Proposed, `Declared`):**
   RFC-0008 §4.7 C3 (Enacted, M-356) established `TaskOutcome` as the graded output of a spawned
   hypha's result. Under B+, the spawned task's result is already on route i (graded output via
   `TaskOutcome` — RFC-0018 §4.5's route i). No additional grading rule for `spawn` is needed at
   the per-fn level; the grade of the scope's result is the meet of the `TaskOutcome`'s grades
   (RFC-0018's meet-composition rule, §4.1 G-App). The Design A precondition is therefore
   satisfied by the existing `TaskOutcome` mechanism. `Declared` — grounding at `Empirical` or
   better is the Phase-2 RFC's obligation (DN-60 §5 DoD item 4).

5. **Composition with grading (Proposed, `Declared`):**
   A `colony { … }` block that spawns hyphae yields a scope result of grade = meet of each
   `TaskOutcome` grade (RFC-0018 §4.1 meet rule). The scope itself carries no special effect
   annotation; the grading flows through the value result, consistent with v0 grading and route i.

6. **`cascade` effect kind (unchanged):** The v0 `cascade(<=D)` effect kind (§4.5 I4/I5) bounds
   re-entrant spawning patterns (bounded restart cascades). B+ does not add a new budget concern
   for task creation; `cascade` already serves the bounded re-entry obligation. The per-hypha
   budget ledger (RFC-0008 §4.7 C1; M-353 per-task) is configured at spawn time, not via the
   spawning fn's effect annotation. `Declared`.

#### 10.3.3 D2/D3 interaction — open sub-question (flagged)

**Flagged tension (DN-60 §4.3):** If D2 (effect variables, O1) is adopted, the interaction
with B+ must be addressed. Under B+, `spawn` is not a row element and cannot appear in an
effect variable `!E`; under Position A, it can. The Phase-2 RFC must decide D2 and D3
**jointly** — or explicitly scope one out and record the rationale. Deciding D3 before D2 is
settled (as this revision does) is acceptable *for the Proposed status* but is not ratifiable
until the joint treatment is resolved. `Declared` open sub-question for the maintainer.

---

### 10.4 Definition of Done (ratification gate for r2)

Per house rule #6, this revision carries an explicit Definition of Done. **Ratification
(Proposed → Accepted) requires the following, all discharged:**

1. **D1 — Budget surface:** The normative surface spelling is fixed (extending M-660's grammar).
   The budget-composition rule (max-dominance or call-graph-path-sum) is decided with an
   `Empirical`-or-better operational semantics argument. The trait/impl interaction for bounded
   effects (impl-must-equal-trait extended to bounds) is specified. The per-hypha budget
   separation (§10.1.3) is confirmed as correct by operational argument.

2. **D2a — Effect-row polymorphism:** Which option (O1/O2/O3) is normative, with justification.
   The RFC-0018 §4.5 Design A soundness precondition (§10.2.3) is discharged at **`Empirical`
   minimum** (route i argument + distinguishing examples). KC-3: zero new L0 nodes confirmed.
   Interaction with RFC-0019 (G4 group) is resolved or explicitly scoped out with rationale.

3. **D2b — Minimal-set inference:** Adopted or not; if adopted, the inference algorithm is
   specified and its completeness claim is tagged honestly (`Empirical`/`Proven`). The
   "inference-with-shown-output" behavior is specified (what the developer sees, when, in what
   form). Whether inference is opt-in or default is decided.

4. **D3 — Position B+:** The surface form for a spawn-enabling scope (colony block or equivalent)
   is normative. The RFC-0018 Design A precondition (route i via `TaskOutcome`) is discharged at
   `Empirical`-or-better. D2 and D3 are jointly decided or independently scoped with explicit
   rationale.

5. **Invariant preservation (non-negotiable):** A verification argument (at minimum `Declared`)
   that I3 (no undeclared effects), I4 (budget overrun is explicit and graceful), and I5 (default
   tightly scoped; broader is opt-in) hold under every adopted direction.

6. **KC-3:** Effect rows, variables, and inference remain checker/surface metadata. If any
   direction requires a new L0 node, it is explicitly justified through the kernel-freeze gate
   (DN-56 §4, condition #3 / ADR-033 precedent) — and this is a blocker if not resolved.

7. **Implementation path:** A concrete implementation scope (crate, issue ID) and a `just check`
   green state (or a stated reason for no code at ratification) are recorded.

**This revision is Proposed.** None of the DoD items above are discharged here; that is the
work the Phase-2 RFC ratification process entails.

---

## Meta — changelog

- **2026-06-28 — §10 r2 Phase-2 effect-system enrichment added (Proposed; append-only, VR-5).** Added §10 as an append-only revision section covering the three G6 deferred directions from DN-60: **(D1)** surface budget syntax — O1 static literal bounds first (normative form `retry(<=N)`, `alloc(<=B)`, `cascade(<=D)`), max-dominance composition rule (pending `Empirical` ratification), per-hypha budget kept separate (DN-60 §2.3); **(D2a)** effect-row polymorphism — O1 effect variables (`!E`) as target, instantiation at call-site, KC-3 (no L0 node), RFC-0018 §4.5 Design A precondition via route i (`TaskOutcome`, RFC-0008 §4.7 C3), with O2 bounded quantification as fallback; **(D2b)** minimal-set inference — inference-with-shown-output, explicit-annotation-wins on over-declaration, completeness tag obligation for the Phase-2 RFC; **(D3)** hypha-creation — Position B+ (scope-level boundary, not per-fn effect kind), `colony { … }` as the spawning-enabling scope, RFC-0018 Design A precondition satisfied by existing `TaskOutcome`, `cascade(<=D)` unchanged for re-entry bounds. §10.4 carries the Definition of Done (7 items for Proposed→Accepted). Status header updated append-only to note the r2 Proposed revision. All directions `Declared`; ground-floor obligations for ratification are `Empirical`-minimum (DN-60 §5 DoD). No v0 content altered. No Doc-Index/CHANGELOG/issues.yaml changes — **FLAG to orchestrator/owning parent** for update. Append-only.
- **2026-06-22 — §3.4 effect-annotation *surface* pinned for the L1 stage-1 frontend; coverage check implemented Rust-first (M-660; append-only, VR-5).** §3.4's previously-**illustrative** `!{…}` effect-annotation form (the §3 banner: "Syntax below is illustrative, not normative … concrete surface syntax is KC-2-gated") is now **pinned as the normative L1 stage-1 surface** — a maintainer decision on the spelling (2026-06-22) — and **implemented Rust-first in `crates/mycelium-l1/`** (pending RFC-0006 ratification of the surface; status header **unchanged — still `Enacted`**, this records an implementation + a surface-spelling pin, not a status move). **Surface:** an **optional** `!{ eff1, eff2 }` after a fn signature's return type — `fn name<params>(value_params) -> ret !{eff1, eff2}`; **absent ⇒ pure** (the empty set; RFC-0014 I5 default-tightly-scoped), and the explicit `!{}` is the same empty/pure set. Effect names are **plain identifiers** (NOT reserved words) — the closed kernel effect kinds `retry|alloc|io|cascade|time` (§4.5) plus user-declared `Named` effects; a **duplicate** name in one annotation is a never-silent **parse** refusal (G2). **Coverage check (I3):** a fn's **declared** effects must be a **superset** of the effects it **performs**, where *performing* = the union of the declared effects of every callee it invokes — a top-level fn OR an unqualified trait method (the latter's effects from the trait registry) — checked over fn bodies **and** impl-method bodies (the §8 "manual-declare + compositional-check" line — the checker *composes* declared effects up the call graph as a **check**, never *infers* an undeclared one; checking impl-method bodies too is what keeps a trait-method/impl effect from being hidden from a caller). **Under-declaration** (performing an undeclared effect) is an explicit `CheckError` naming the effect + the callee; **over-declaration is allowed** (a declaration is a contract — I5). A **trait method** carries effects too: an `impl` method's declared effect set must **equal** the trait method's (exact match in stage-1; an unannotated trait method ⇒ the impl method is pure). **No new L0 node** — effects are **checker metadata** (KC-3); they do **not** lower, and the **runtime budget ledger stays the separate M-353** `mycelium_interp::budget` concern (not wired by this frontend work). Guarantee on the pass: **`Declared`** (a structural coverage check, not a theorem — I3). **Effect *sources* expand later:** `wild`-sourced effects arrive with **M-661** (`wild` stays rejected here), and richer effect typing (rows/inference) remains the §9 future — this frontend lands the **annotation + compositional-coverage** mechanism only. Verified in `mycelium-l1` (`tests/check.rs` effect suite incl. a monotonicity property sweep + the trait/impl effect-conformance cases; `tests/parse.rs` grammar; `accept/16`/`reject/17` conformance fixtures). Append-only.
- **2026-06-20 — status spelling normalized.** Status header `Accepted — Enacted` → **`Enacted`** (the now-canonical standalone token, per the ratified `Draft/Proposed → Accepted → Enacted → Superseded` lattice, #236); semantics unchanged. Append-only.
- **2026-06-18 — Append-only note to §4.7 (RFC-0017 Accepted): maturation is now a scope attribute.**
  Added an inline note to §4.7 recording that "only `total` definitions may be `matured`" now reads
  **over the matured scope** (RFC-0017 §4.2): every reachable non-`thaw` definition in a matured
  nodule/phylum/program must be `total` and pass RFC-0004 §4 stable-component checks. The obligation
  is the **same, universally quantified** — the budget/effect interaction (I4), `EffectBudgetExhausted`
  semantics, and "gates packaging, never meaning" discipline are **unchanged**. `thaw` definitions are
  excluded from the matured set and carry no totality obligation from maturation. No normative content
  altered. Append-only.
- **2026-06-16 — §8 concurrency deferral RESOLVED; single-task boundary lifted (M-356; RFC-0008 §4.7).**
  The §8 concurrency interaction — the last RFC-0008-tied deferral — is resolved in **RFC-0008 §4.7**:
  per-task budgets (each task instances its own M-353 ledger; an overrun is an *in-that-task*
  `EffectBudgetExhausted`), cooperative **additive** cancellation (an explicit `Cancelled`, never
  preemptive — I1), cross-task failure propagation via an explicit `TaskOutcome` with no silent/dropped
  variant (I1 across the boundary), and `reclaim` **bounded-cascade** supervision bounded on *both* the
  `cascade` effect budget (I4/I5) and a windowed max-restart-intensity (logical clock; Erlang/OTP,
  Record 05 T5.3) — exceeding either an explicit escalation, never a storm. Enacted as
  scheduler-independent primitives in `mycelium_interp::supervise` (no L0 node — KC-3; the trusted base
  stays sequential), composed with the recovery driver in `tests/recover.rs`. Everything stays additive
  over the explicit error (I1) and declared + bounded (I3/I4). The task **scheduler** + RT2 differential
  are RFC-0008 R1 (M-357). `just check` green. Append-only.
- **2026-06-16 — §4.8 integration completed (M-353; RFC-0008 Accepted).** The deferred half of §4.8 —
  *wiring the `Budgets` ledger into the runtime/AOT env-machine's budget enforcement* — is enacted. The
  ledger primitive (`EffectKind`/`EffectBudget`/`EffectBudgetExhausted`/`Budgets`) is **lifted into
  `mycelium-interp`** (`mycelium_interp::budget`), the common ancestor both the recovery subsystem
  (`mycelium-lsp`) and the env-machine (`mycelium-mlir`) depend on — the *shared budget-resolution
  surface* §8 anticipated, placed to avoid a crate cycle and to sit where the fuel clock already lives
  (no kernel change — KC-3, **no** new L0 node, **no** kernel hook). An overrun now routes through
  **`mycelium_interp::EvalError::EffectBudget`** — the effect sibling of `FuelExhausted`/`DepthLimit` on
  the **one runtime refusal channel** (§8: *separate named budgets, one enforcement mechanism*): a
  budgeted effect overruns *gracefully at runtime exactly as recursion does*, never a hang/OOM (I4). The
  env-machine threads the *same* ledger (`run_core_with_effects`) and charges a declared **`alloc`**
  budget per control-stack frame (the opt-in sibling of the DN-05 depth ceiling, same per-frame-bytes
  basis) — absent ⇒ unchanged behaviour (I5 opt-in); the `retry`/`cascade` budgets are spent by the
  recovery *driver* over that same ledger and channel. Verified: the **bounded-overrun-is-explicit test
  extended to the runtime path** (`mycelium-mlir`: a declared `alloc` budget overruns as
  `EvalError::EffectBudget`, an absent one leaves behaviour identical) and a **meaning-preserving
  three-way differential** where it touches L0 (`mycelium-l1`: threading an ample ledger is
  observable-transparent on the recovery `Match`, NFR-7). This **completes RFC-0014 §4.8**; the
  *concurrency* composition (§8 — per-task ledgers on this seam) remains RFC-0008's, now in progress
  (M-355). `just check` green. Append-only.
- **2026-06-16 — Accepted + Enacted (M-352).** Maintainer ratified `Draft → Accepted` (approving the
  draft and all §8 dispositions — the four proposed v0 dispositions below move from *sign-off pending* to
  normative) and approved proceeding. The §4 design is enacted as a **separable, tooling-layer** subsystem
  in `crates/mycelium-lsp/src/recover` (no kernel change — KC-3, zero new L0 nodes; no Python — ADR-007):
  `error` (the result-sum `Outcome` over a structured error reusing RFC-0013's shared `ClassRegistry` —
  X1, no `eval`); `effect` (the closed `EffectKind` set + per-kind named `EffectBudget`, the `Budgets`
  ledger whose overrun is an explicit, graceful `EffectBudgetExhausted` — I4, and the compositional
  `check_effects` no-undeclared-effect check — I3); `policy` (the reified `on <ErrorClass> => <action>`
  recovery policy, closed action set `fallback`/`retry`/`escalate`/`cleanup_then_propagate`,
  content-addressed `PolicyRef`); and `handle` (the never-silent driver — every error is **recovered**
  with an honestly-tagged value or **re-propagated**, never dropped — I1; a fallback carries an honest,
  downgrade-only guarantee — I2/VR-5). Verified by `crates/mycelium-lsp/tests/recover.rs` (RFC-0014 §5):
  the central **never-silent recovery invariant** (a corpus of errors × every action yields Recovered or
  Propagated, never a drop — I1), the **bounded-overrun-is-explicit** test (a budgeted effect overrun is
  `EffectBudgetExhausted`, never a hang/OOM — I4), the **no-undeclared-effect** test (I3), the
  **honest-guarantee** test (I2/VR-5), and the **opt-in default scope** test (I5). The
  **L0-`Match`-over-error-sums** lowering target — "recovery introduces no new kernel node" — is
  differentially verified in `mycelium-l1` (`recovery_match_over_a_result_sum_agrees_three_ways`):
  a `Result = Ok | Err` match runs identically on L1-eval ≡ elaborate→L0-interp ≡ AOT (NFR-7).
  **Out of v0 scope (honest boundary):** wiring the `Budgets` ledger into the AOT env-machine's runtime
  budget resolver is the RFC-0008 integration (the §4.8 boundary), tracked separately; v0 delivers the
  reified mechanism + semantics + invariants. `just check` green. Append-only.
- **2026-06-16 — Remaining §8 questions given proposed v0 dispositions (sign-off pending; still
  Draft).** At the maintainer's direction (draft the remaining answers before any code), the four §8
  questions left after the three gating decisions now carry concrete **proposed v0 dispositions**,
  marked *maintainer sign-off pending* (append-only — these are proposals, not yet Accepted decisions):
  (1) **effect inference** = *manual-declare + compositional-check* (the checker requires a caller to
  declare a superset of its callee's effects — `UndeclaredEffect` otherwise — but never *infers* an
  undeclared effect; true minimal-set inference deferred to §9); (2) **recovery-action set** = the
  *closed* v0 set `fallback`/`retry`/`escalate`/`cleanup_then_propagate` (each provably never-silent +
  bounded; user-defined actions are a §9 future and would inherit I1/I3/I4); (3) **concurrency** =
  *deferred to RFC-0008* with a v0 boundary fixed now (single-task / synchronous; per-evaluation
  budgets; no cross-task cascade — so the deferral is safe, not merely postponed); (4) **handler
  composition** = *lexical innermost-first* (unmatched re-propagates, never drops — I1), a handler's own
  effects are declared + budgeted like any code, and a cascade is bounded by `cascade(max_depth)`
  (overrun → `EffectBudgetExhausted`) — all with no machinery beyond §4.5. With these + the §7 prior-art
  tracing (Record 05) done, the RFC is **ready for a Draft→Accepted decision**; **no code lands until
  Accepted** (RFC-0014's gate). Append-only.
- **2026-06-16 — Three §8 design questions resolved (maintainer; still Draft).** The maintainer settled
  the three questions that gate any recovery/effects code, all on the KC-3/KISS-aligned options: (1)
  **effect mechanism = declared annotations, coarse set** (not capabilities, not hybrid — capabilities/
  effect-rows stay additive future possibilities, §9); (2) **no kernel-visible hook** — effect-budget
  enforcement is entirely a runtime/checker concern, zero new L0 nodes (§4.8); (3) **separate named budgets,
  one enforcement mechanism** — each effect kind keeps its own `EXPLAIN`-able budget vocabulary but all are
  resolved/enforced by the existing DN-05 budget machinery that already clocks `Fix`/`FixGroup` fuel and the
  M-347 depth ceiling (composed alongside, not collapsed into one number). §8/§4.8 and the status line record
  the dispositions. The RFC **stays Draft** — the remaining §8 questions (recovery-action set, effect
  inference, concurrency, handler composition) and the §7 prior-art tracing into `research/` are still open
  before Accepted; **no code lands until then.** Append-only.
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
