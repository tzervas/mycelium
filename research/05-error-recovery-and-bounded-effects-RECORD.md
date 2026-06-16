# Research Record 05 — Declarative Error Recovery & Bounded Effects (RFC-0014 grounding)

> **What this file is.** A durable record tracing the prior art that **RFC-0014 (Declarative Error
> Recovery & Bounded Effects)** cites as *design inspiration*, into the evidence base — discharging
> the RFC-0014 §7/§8 obligation to ground (or refute) its prior art before the RFC can move
> `Draft → Accepted`. Conducted 2026-06-16; load-bearing facts were checked against primary or
> primary-adjacent sources (flagged below), and anything stated from background knowledge is marked
> in the uncertainty register. Findings are labeled **T5.1–T5.6** (continuing the T0–T4 scheme) and
> map onto RFC-0014's invariants **I1–I5** and its three resolved §8 design decisions
> (declared-annotation effects; no kernel hook; separate named budgets over one enforcement
> mechanism).

## Scope

RFC-0014 designs error recovery as **errors-as-propagating-values** + **explicit declarative
recovery** + **declared, bounded effects**, additive over the never-silent error (G2) and never
upgrading a guarantee (VR-5). Five external inspirations and one in-repo precedent need grounding:

1. Result / `?` error values (Rust, Swift, Go) — the substrate posture (→ T5.1, I1/§4.1/§4.3).
2. Algebraic effects & handlers (Koka, Eff, Frank, OCaml 5) — reified, handled effects (→ T5.2, §4.5/§4.6).
3. Erlang/OTP supervision — *bounded* restart strategies (→ T5.3, I4/I5 — the strongest external basis).
4. Structured-concurrency cancellation (nurseries / scopes) — scoped, bounded failure (→ T5.4, I5).
5. Capability-based effect control — effects gated by a passed token (→ T5.5, the §8 alternative not chosen).
6. Mycelium's own budget idiom (fuel / depth / DN-05) — the precedent bounded effects generalise (→ T5.6).

The question throughout: which disciplines let a value-semantics, honesty-first, totality-aware
substrate express error-driven control *without* admitting an unintended / unknown / unbounded
effect — the maintainer's governing line.

## Results by inspiration

### T5.1 — Errors as explicit, propagating values (→ RFC-0014 §4.1/§4.3, I1)

- **Rust `Result<T, E>` + `?`.** Errors are ordinary sum-typed *values*; the `?` operator makes
  "propagate this error to the caller" an explicit, visible use-site form (early-return on `Err`),
  not an invisible unwind. There is no implicit catch: an unhandled `Err` is a value you still hold.
  This is exactly RFC-0014 §4.1's "explicit propagation form surfaces 'bubble this up'" and §4.3's
  "recovery = `Match` on the result sum at a visible site." *Verified* (Rust language semantics, well
  established; primary: the Rust Reference / `std::result`).
- **Swift `throws`/`try` and Go `if err != nil`.** Two more points on the same axis: Swift's typed
  `throws` with explicit `try` at the call site; Go's returned-error-value convention checked
  explicitly. Both keep the error a value handled at a visible site (Go most explicitly — no
  exception machinery in the common path). *Background* (well-known; not re-verified this pass).
- **Mycelium's stricter rule.** All three *permit* dropping an error (Rust `let _ =`, Go ignoring the
  return, Swift `try?` discarding). RFC-0014's I1 is **stricter**: a handling site must cover the
  cases or re-propagate the remainder — it **cannot** silently drop (never-silent G2). So Mycelium
  *adopts the substrate posture* (errors-as-values, explicit propagation) but *tightens* the
  discipline. This is a genuine delta, recorded honestly — not a claim that Result/`?` already
  enforces never-silent.

### T5.2 — Algebraic effects & handlers (→ RFC-0014 §4.5/§4.6; the reified-effect inspiration)

- **The model.** Algebraic effects (Plotkin–Pretnar) denote an effect as a set of *operations*;
  **handlers** (user-defined) interpret them, with access to a *delimited continuation* that can
  resume the computation from the point the effect was performed. Available as primitive features in
  **Eff, Koka, Frank, and Multicore OCaml**. *Verified* (arXiv 1811.07332 framing; OCaml-multicore
  tutorial; search 2026-06-16).
- **OCaml 5.0.** Ships effect handlers built on **fibers** (small heap-allocated, growable stack
  chunks); they express resumable exceptions, delimited continuations, and async — but in 5.0 the
  **type system does not track effects** (effects are *untyped*). *Verified* (OCaml-multicore
  tutorial / OCaml 5.0 release notes; search 2026-06-16).
- **Koka.** Effects are a **row-typed** part of every function's type (`int -> <exn,div> int`); the
  compiler tracks and discharges them. The strongest example of *effects visible in the signature*.
  *Background / primary-adjacent* (Koka book; not re-fetched this pass — treat the row-typing claim as
  well-established background).
- **What RFC-0014 takes, and what it deliberately does NOT.** RFC-0014 imports the *reified, handled,
  inspectable* posture (§4.6: "what effects can this code do, with what bounds, handled by what
  policy?") and the idea that effects belong in the signature (§4.5 I3, Koka-style visibility). It
  **does not** import full delimited-continuation handlers or effect-row polymorphism for v0: the
  resolved §8 decision is a **coarse declared effect *set*** (KISS/YAGNI), with richer effect typing
  an explicitly *additive* future possibility (§9). Honest delta: Mycelium v0 is far weaker than
  OCaml/Koka effects — it is "named, bounded actions," not general resumable handlers. A general
  handler that captured/discarded a continuation could *swallow* an error, which I1 forbids; the v0
  closed action set (`fallback`/`retry`/`escalate`/`cleanup_then_propagate`) is the bounded subset
  that stays never-silent.

### T5.3 — Erlang/OTP bounded supervision (→ RFC-0014 I4/I5; the strongest external basis)

- **Max-restart-intensity is a *bounded cascade*.** An OTP supervisor caps restarts with two flags:
  `intensity` (MaxR) and `period` (MaxT, seconds). **If more than `MaxR` restarts occur within `MaxT`
  seconds, the supervisor terminates all children and then itself** — the restart "storm" is *capped*,
  not unbounded. Defaults: **`intensity = 1`, `period = 5`**. *Verified* (erlang.org supervisor docs /
  Supervisor Behaviour design principles; search 2026-06-16).
- **Why it is the strongest grounding.** This is precisely RFC-0014 I4/I5: a recovery action (restart)
  that *could* run away is made **declared + bounded**, and the overrun is an **explicit, graceful**
  outcome (the supervisor escalates by terminating) — **never** an unbounded loop. RFC-0014's
  `retry(<=N)` and `cascade(max_depth)` are the same idea, and `EffectBudgetExhausted` is the analogue
  of the supervisor hitting MaxR. The OTP precedent shows a *production* system where bounded cascades
  are the default discipline, which directly supports the maintainer's "default tightly scoped;
  broader/cascading is opt-in and bounded."
- **Delta.** OTP bounds *restarts of processes*; RFC-0014 generalises "bounded recurring effect" to
  retries, allocation, and depth in a single-node value calculus. The *bounding principle* transfers
  cleanly; the *process/supervision tree* does not (that maps to RFC-0008 concurrency, deferred).

### T5.4 — Structured-concurrency cancellation (→ RFC-0014 I5; "tightly scoped by default")

- **Scoped failure.** Structured concurrency (Trio "nurseries"; Kotlin coroutine scopes; Java
  Project Loom `StructuredTaskScope`) binds a child's lifetime and failure to an explicit *scope*: a
  failure cancels the scope's siblings, and no task outlives its scope. Failure and effects are
  **bounded to a lexical region**. *Background* (well established; not re-verified this pass).
- **Mapping.** This grounds RFC-0014 I5's "narrowest effect scope is the default": an effect/cascade
  is confined unless explicitly widened. It is most relevant *once RFC-0008 concurrency lands* (a
  single-node v0 recovery site is the degenerate scope); recorded as the inspiration for scope-bounded
  effects, not a v0 mechanism.

### T5.5 — Capability-based effect control (→ RFC-0014 §8 alternative, NOT chosen for v0)

- **The model.** Effects are available only where an explicit **capability** (a value/token) is in
  scope — "no ambient authority": a function that cannot name the capability cannot perform the
  effect. (Capability-safe languages; the object-capability tradition; capability-passing effect
  systems.) *Background* (well established; not re-verified this pass).
- **Disposition.** This was the explicit alternative to declared annotations in RFC-0014 §8. The
  maintainer (2026-06-16) chose **declared annotations (coarse set)** for v0 — simpler surface, same
  "no unknown effects" payoff — and recorded capabilities as an *additive* future possibility (§9),
  not a rejection. Recorded here so the road not taken is grounded, not lost.

### T5.6 — Mycelium's own budget idiom (→ RFC-0014 §4.5; the in-repo precedent, strongest basis)

- **Already grounded in-corpus.** The bounded-effect discipline is the direct generalisation of three
  *existing, ratified* Mycelium mechanisms: the `Fix`/`FixGroup` **fuel clock** (RFC-0007 §4.5 — every
  unfold is clocked; overrun is an explicit `FuelExhausted`), the **control-stack depth ceiling**
  (M-347 / DN-05 #2 — explicit `DepthLimit`, not a host stack overflow), and **DN-05 dynamically
  resolved budgets** (detect a safe depth at runtime). *Verified in-repo* (RFC-0007 §4.5;
  `crates/mycelium-mlir` budget/trampoline work; DN-05).
- **Why this matters for the §8 budget decision.** Because the precedent already enforces *separate*
  named limits (fuel ≠ depth) through *one* runtime/DN-05 enforcement path, the resolved §8 choice —
  **separate named budgets, one enforcement mechanism** — is the *least-surprising* extension: effect
  budgets (`max_attempts`, `max_depth`, alloc ceiling) join fuel and depth in the same plumbing rather
  than collapsing them into one number. This is the strongest, *already-grounded* basis for RFC-0014,
  and the reason the "no kernel hook" decision is credible: fuel/depth are already enforced outside
  the trusted kernel calculus, so effect budgets can be too.

## How this discharges the RFC-0014 §7/§8 grounding obligation

- The **strongest** claims (I4/I5 bounded cascades; the budget generalisation) rest on **verified**
  grounding: OTP max-restart-intensity (external, verified 2026-06-16) **and** Mycelium's own fuel/
  depth/DN-05 budgets (in-repo, ratified). These are not "design inspiration only" anymore — they are
  traced.
- The **substrate** posture (errors-as-values, explicit propagation) is grounded in Result/`?`
  (verified), with Mycelium's stricter never-silent rule recorded as an honest *delta*, not a borrowed
  guarantee.
- The **effect-reification** posture is grounded in algebraic effects / Koka row-effects (verified/
  background), with the honest delta that v0 takes only the *coarse, bounded, named-action* subset —
  **not** general resumable handlers (which could violate I1).
- The **not-chosen** alternative (capabilities) and the **deferred** mapping (structured concurrency)
  are recorded so the design space is complete.

This satisfies "trace the prior art into `research/` (or refute it) before Accepted" (RFC-0014 §8).
RFC-0014 stays **Draft** pending its *remaining* §8 questions (recovery-action-set completeness,
effect inference vs. manual, concurrency interaction, handler composition) — those are design
choices, not grounding gaps.

## Uncertainty register (honesty flags)

- **Verified this pass (search 2026-06-16):** OTP `intensity`/`period` semantics + defaults (1 / 5s);
  OCaml 5.0 effects on fibers + *untyped* in 5.0; algebraic effects as primitives in Eff/Koka/Frank/
  Multicore OCaml.
- **Background, not re-verified this pass:** Swift `throws`/`try?`, Go error-value convention; Koka's
  row-effect typing *specifics*; the structured-concurrency cancellation semantics of Trio/Kotlin/Loom;
  the object-capability / capability-passing-effects literature. Treat as well-established background;
  cite primary sources if any becomes load-bearing for a ratification claim.
- **In-repo, verified:** the fuel/depth/DN-05 budget precedent (RFC-0007 §4.5; M-347; DN-05).
- **No quantitative external claims** (throughput, fiber sizes, restart-storm frequencies) are made;
  none are load-bearing for RFC-0014, which borrows *disciplines*, not performance figures.
- **Novelty (no found precedent):** *additive-over-a-never-silent-error* recovery (I1 — a handler may
  never drop an error, stricter than Result/`?`); per-effect **honest guarantee tags** on recovered
  values (I2/VR-5); unifying retry/cascade/alloc/time budgets with the totality fuel clock under one
  enforcement mechanism. Treat these as Mycelium contributions needing their own soundness arguments,
  not citations.

## Key sources

- Erlang/OTP — *Supervisor Behaviour* design principles & `supervisor` module reference
  (erlang.org/doc) — `intensity`/`period`, defaults 1 / 5s, escalation on overrun.
- OCaml-multicore — *Concurrent Programming with Effect Handlers* tutorial; OCaml 5.0 release notes
  (effects on fibers; untyped in 5.0).
- Plotkin & Pretnar — algebraic effects & handlers (framing via arXiv 1811.07332, *Handling
  Polymorphic Algebraic Effects*); Koka, Eff, Frank as effect-handler languages.
- The Rust Reference / `std::result` — `Result` + `?` propagation (substrate posture).
- Structured concurrency: Trio nurseries; Java Project Loom `StructuredTaskScope` (scoped cancellation).
- In-repo: RFC-0007 §4.5 (fuel / `matured` gate), M-347 / DN-05 (depth ceiling, dynamic budgets),
  RFC-0001 r5 (`FixGroup`).

## Meta — changelog

- **2026-06-16 — Created.** Traces RFC-0014's §7 prior art (Result/`?`; algebraic effects & handlers;
  Erlang/OTP bounded supervision; structured-concurrency cancellation; capability-based effects; and
  Mycelium's own fuel/depth/DN-05 budget idiom) into the evidence base as findings T5.1–T5.6, mapped to
  RFC-0014 I1–I5 and the three resolved §8 decisions. Verified the load-bearing externals (OTP
  max-restart-intensity defaults 1/5s; OCaml 5.0 untyped effects on fibers) by web search 2026-06-16;
  flagged background-only and in-repo-verified claims and the novelty (no-found-precedent) items in the
  uncertainty register. Discharges the RFC-0014 §8 "trace prior art before Accepted" obligation;
  RFC-0014 stays Draft pending its remaining (design, not grounding) §8 questions. Append-only.
