# Research Record 06 — Automatic Baseline Diagnostics & Recovery (RFC-0015 grounding)

> **What this file is.** A durable record tracing the prior art **RFC-0015 (Automatic Baseline
> Diagnostics & Recovery)** cites as *design inspiration*, into the evidence base — discharging the
> RFC-0015 §7/§8 obligation to ground (or refute) its prior art before the RFC can move
> `Draft → Accepted`. Conducted 2026-06-16. Load-bearing facts checked against primary or
> primary-adjacent sources where noted; anything from background knowledge is flagged in the
> uncertainty register. Findings are labeled **T6.1–T6.5** (continuing the T0–T5 scheme) and map onto
> RFC-0015's honesty-boundary rules **A1–A4** and its four resolved §8 questions.

## Scope

RFC-0015 adds an **automation layer over RFC-0013 (presentation) + RFC-0014 (recovery)**: a zero-config
**baseline** diagnostic/logging policy, *auto-derived* from the language's own structured mapping
(error-class registry + each class's level/route + a definition's declared effects) and *auto-applied*
by wrapping definitions/scopes — with a ladder of *light* overrides → *fully manual*. The governing
question: **what may be automatic without becoming the silent black box the project forbids (G2/SC-3)?**
The five inspirations (RFC-0015 §7):

1. **DynEL** — the automated-baseline + dynamic-application inspiration (DN-04) (→ T6.1, A1/A2/A4).
2. **Rust `tracing`/`log` + default subscribers** — zero-config logging that stays explicit (→ T6.2, A1/A3).
3. **Erlang/OTP default loggers + SASL reports** — baseline supervision/logging, opt-in customization (→ T6.3, A2).
4. **Python `logging.basicConfig`** — the zero-config baseline, *minus* the honesty discipline (→ T6.4, A1/A4 — the cautionary case).
5. **Structured-logging frameworks (auto-context)** — automatic context capture, and its risk (→ T6.5, A1/X2).

## Results by inspiration

### T6.1 — DynEL: "automated baseline error handling + logging" (the seed)
DynEL's signature intent (DN-04 §1) is *ease*: a project gets sensible baseline error handling/logging
with no boilerplate, applied automatically and derived dynamically from program structure, then
customizable down to fully manual. DN-04 already split the *inspiration* into the explicit substrate
(presentation → RFC-0013, recovery → RFC-0014) and flagged DynEL's two anti-patterns to **avoid**: the
`eval(exception_str)` class-from-config (DN-04 §6 — answered by the registry's looked-up-never-evaluated
`ClassName`, X1) and silent swallowing. **Maps to A1/A4:** the automated baseline is salvageable
precisely when it is (a) additive presentation only by default — it cannot swallow — and (b) a *total,
inspectable derivation* of the mapping, not a learned/`eval`-ed guess. **Refutes** the naive reading of
DynEL (auto = magic): the value is the *ease*, not the opacity.

### T6.2 — Rust `tracing` / `log` + default subscribers (zero-config, still explicit)
`tracing`/`log` provide instrumentation that is **inert until a subscriber is installed**; a default
subscriber (e.g. `tracing_subscriber::fmt`) gives zero-config console logging, and events carry
structured fields. The load-bearing property for RFC-0015: logging is **purely additive** — emitting a
span/event never alters control flow — so a zero-config default is *safe* to apply broadly. **Maps to
A1/A3:** the auto-baseline is RFC-0013 presentation (additive, I1), and "what is the active subscriber/
policy?" is always answerable (RFC-0015 makes the baseline a *materialized, content-addressed* policy,
the analogue of an installed-but-inspectable subscriber). *(Checked against background knowledge of the
`tracing`/`log` facade model; the additive-logging property is definitional, not version-specific.)*

### T6.3 — Erlang/OTP default loggers + SASL (baseline + opt-in supervision)
OTP ships a **default logger** and SASL progress/crash reports out of the box (baseline logging with no
app code), while *recovery* is the **supervisor**: restart strategies are **explicitly declared** in a
supervision tree and **bounded** (`intensity`/`period` — too many restarts in a window escalates rather
than looping forever). The split is exactly RFC-0015's A1/A2: **logging baseline = on by default**
(additive), **recovery = explicit + bounded + opt-in** (a supervisor is something you *write*, with a
finite restart budget). **Strongest external basis for A2** (recovery default-off, declared, bounded —
the I4/I5 reuse) and for §8-Q3 (presentation default-on, recovery default-off).

### T6.4 — Python `logging.basicConfig` (the baseline — and the cautionary tale)
`basicConfig` is the canonical zero-config baseline: one call wires a root handler/format. It validates
the *ergonomics* RFC-0015 wants. But it is the **cautionary** case for honesty: root-logger config is
**global, order-dependent, and silently no-ops** if a handler already exists (the first `basicConfig`
wins; later calls are ignored without error) — ambient, non-inspectable, surprise-prone. **Maps to
A4/§8-Q1 by contrast:** RFC-0015's baseline must be a *per-target, materialized, EXPLAIN-able* derivation
(answerable: "what baseline applied here, and why?"), not a hidden global default that can silently
no-op. The ease is worth copying; the ambient global state is exactly what to avoid.

### T6.5 — Structured-logging auto-context (power, and the X2 risk)
Structured loggers auto-attach context (request ids, spans, MDC/thread-locals). Useful, but the risk is
a **wholesale context/locals dump** — the same secrets-leak vector RFC-0013 §4.5 X2 closes with the
**detailed-tier allowlist**. **Maps to A1/X2:** an auto-baseline may route and present, but it inherits
RFC-0013's allowlist — it can only carry allowlisted context fields, never an environment dump. Automatic
context is bounded by the same closed allowlist as manual.

## How the findings resolve RFC-0015 §8

- **Q1 — the derivation function.** Grounded by T6.1+T6.4: a **total, inspectable** function from the
  registry (classes) + a **closed default class→(level, route) table** + declared-effect scope, never a
  global ambient default. Resolved (§8) as the `derive_baseline` mapping; per-`phylum` configuration
  rides the M-359 manifest (future, §9).
- **Q2 — auto-recovery profiles.** Grounded by T6.3 (OTP): a **closed, named, opt-in, bounded** set —
  v0 `strict` (propagate-all) and `resilient` (bounded `retry(≤3)` on declared classes) — built from the
  RFC-0014 `RecoveryPolicy` (no new mechanism).
- **Q3 — scope + defaults.** Grounded by T6.2+T6.3: **presentation default-on** (additive, A1-safe),
  **recovery default-off** (A2); **per-definition** auto-wrap (scoped by declared effects) in v0,
  per-`nodule`/`phylum` via the M-359 manifest deferred.
- **Q4 — toolchain/stdlib integration.** The M-361 lint "class only logged, no handler" and the M-346
  stdlib defaults consume the derived baseline; named here, enacted there.

## Uncertainty register

- T6.2/T6.4/T6.5 are checked against **background knowledge** of these widely-used libraries (the
  additive-logging and ambient-global-config properties are definitional, not version-specific); no
  primary docs were fetched in this environment. Flagged **Empirical/Declared**, not Proven.
- T6.3 (OTP supervision intensity/period + default logger/SASL) is background knowledge of OTP design;
  the *bounded-restart* property is the load-bearing one and is well-established. Flagged the same.
- No claim here upgrades any guarantee; all findings are inspiration-grounding, consistent with VR-5.

## Changelog
- **2026-06-16 — Created.** Traces RFC-0015's five prior-art inspirations (DynEL, `tracing`/`log`,
  OTP, Python `logging`, structured-logging) into the evidence base as **T6.1–T6.5**, resolving §8's
  four questions ahead of the `Draft → Accepted` move (M-362). Append-only.
