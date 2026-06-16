# RFC-0015 — Automatic Baseline Diagnostics & Recovery

| Field | Value |
|---|---|
| **RFC** | 0015 |
| **Status** | **Draft (Proposed)** (2026-06-16 — captures the DynEL "automated baseline" design point on the roadmap; ratification + enactment is M-362) |
| **Type** | Foundational / normative (once Accepted) — a **tooling-layer automation** over RFC-0013/0014; minimal/no kernel change (KC-3) |
| **Feeds** | RFC-0006 (the optional surface for opting in/out); the stdlib (M-346); the toolchain (M-361) |
| **Depends on** | RFC-0013 (presentation — the additive substrate the auto-baseline applies); RFC-0014 (declarative recovery & bounded effects — the opt-in control layer); RFC-0005/ADR-006 (reified, content-addressed, EXPLAIN-able policies); RFC-0008 §4.8/RFC-0013 §8 (the observability sinks routes bind to); G2 (never-silent), VR-5, KC-3, NFR-2/SC-5b |
| **Tracks** | M-362. Lineage: DN-04 (DynEL: *automated baseline error handling + logging*) → RFC-0013 (presentation) + RFC-0014 (recovery) → **RFC-0015 (the automation over both)** |

---

## 1. Summary

DN-04 took DynEL's inspiration in two halves: the **structured/graded/dual-format presentation** (now
RFC-0013) and the **declarative recovery** (now RFC-0014). Both deliberately built the *explicit,
reified* substrate — every policy is written, named, content-addressed. DynEL's *other* signature
intent was **ease**: a project should get **sensible baseline error handling and logging with no
boilerplate**, *automatically* applied, and *dynamically* derived from how the program is structured —
then progressively customized, down to fully manual.

This RFC adds that **automation layer on top of RFC-0013/0014**: a zero-config **baseline** diagnostic/
logging policy **auto-derived from the language's own structured mapping** (the error-class registry,
each class's level/route, a definition's *declared effects*), **auto-applied** by wrapping definitions/
scopes — for (1) boilerplate QoL and (2) an easier dev workflow — with a clean ladder to light-touch
customization and to fully manual control. The automation is **materialized, not magic**: the baseline
it applies is a reified, inspectable, `EXPLAIN`-able policy (RFC-0005), never an opaque default.

## 2. Motivation

- **DynEL's ease, honestly delivered.** RFC-0013/0014 make every policy explicit — *powerful*, but a
  developer who just wants "log my errors sensibly" should not have to write one. The baseline removes
  that boilerplate without removing the explicitness (the baseline is itself a readable policy).
- **The mapping is already there.** Mycelium *knows* a program's error structure — the registry of
  classes (RFC-0013 §4.5), each class's default level/route, and each definition's **declared effects**
  (RFC-0014 I3). A sensible default policy is a **total, inspectable derivation** from that mapping
  (RFC-0005's posture — non-learned, EXPLAIN-able), not a guess.
- **QoL + workflow.** Auto-wrapping for logging/diagnostics is the single biggest boilerplate sink in
  error handling; automating it (while leaving control flow untouched) is high-leverage and safe.

## 3. The layered model (progressive disclosure)

Three layers; **lower layers are the floor, manual always wins**:

- **Auto (zero-config baseline).** The toolchain derives a **baseline diagnostic policy** from the
  structured mapping and applies it to every definition/scope: each known error class gets a sensible
  level + route (a logging/observability sink, RFC-0013 §8) — no developer input. "Auto-wrapping" a
  definition = attaching this baseline *presentation* around the errors it can raise.
- **Light (overrides).** The developer overrides a *few* rules atop the baseline (a class's level/route/
  message), expressed as the same reified RFC-0013 policy but carrying only the **deltas**; everything
  else stays auto.
- **Manual (full control).** Write the whole diagnostic/recovery policy and handlers by hand
  (RFC-0013/0014 exactly as today). The auto-baseline steps aside wherever a manual policy is present.

## 4. Reference-level design (normative once Accepted)

### 4.1 The honesty boundary — what may be automatic (the load-bearing rule)

Automation must not become the silent black box the project forbids. The boundary:

- **(A1) Automatic = additive presentation/logging by default (G2/I1).** The auto-baseline *presents,
  routes, and logs* — RFC-0013 operations, which **never change control flow** (RFC-0013's I1). Precisely
  *because* it is additive and cannot suppress or alter an error, it is safe to auto-apply everywhere.
  The auto-baseline can **never** swallow, soften, or hide an error.
- **(A2) Automatic recovery is opt-in, declared, and bounded (RFC-0014 I3/I4/I5).** A recovery that
  *changes control flow* (fallback/retry/escalate/cleanup) **never arrives implicitly**. The automation
  layer may (a) **scaffold** — generate an explicit handler *skeleton* a developer completes — or
  (b) apply a **named, declared, bounded recovery profile** the developer **explicitly opts into** (e.g.
  `profile: resilient = { on Transient => retry(<=3); else propagate }`). Either way the recovery is
  explicit, declared, and budgeted (I3/I4), and broader behaviour is opt-in (I5). There is **no**
  automatic, implicit control-flow change.
- **(A3) Reified + EXPLAIN (no black box, SC-3).** The auto-derived baseline is a **materialized**,
  content-addressed RFC-0005 policy you can read, diff, and `EXPLAIN` — "what baseline applied here, and
  why?" is always answerable. Automation is a *visible default*, never hidden behaviour.
- **(A4) Honest by derivation, not learning (VR-5/RFC-0005).** "Dynamically applies based on the detailed
  mapping" = a **total, inspectable** function of the registry + declared effects + route vocabulary —
  non-learned, deterministic, no fabricated guarantee. A baseline never upgrades an error's honesty.

### 4.2 The derivation (sketch)

The baseline policy is computed from: the **error-class registry** (which classes exist, their default
level — RFC-0013 §4.5), the **route vocabulary** (which observability sink each class logs to by default
— RFC-0013 §8 / RFC-0008 sinks), and each definition's **declared effect set** (RFC-0014 I3 — which
errors it can raise, what it may do). The output is an ordinary `DiagnosticPolicy` (+ optionally, when
opted in, a `RecoveryPolicy` from a named profile), content-addressed like any other. The derivation is
the design's core open work (§8).

### 4.3 Relationship to RFC-0013 / RFC-0014

RFC-0015 **adds no new error mechanism** — it *generates and applies* RFC-0013/0014 policies. It cannot
weaken their invariants: the auto-baseline is RFC-0013 presentation (additive, I1); any auto-recovery is
RFC-0014 (declared/bounded/opt-in, I3/I4/I5). Remove RFC-0015 and the explicit substrate is unchanged —
the automation is a *convenience layer*, not a new semantics.

## 5. Verification (direction)

When enacted: the auto-baseline preserves **never-silent (I1)** across every auto-wrapped definition (the
RFC-0013 invariant test, now over auto-derived policies — a baseline can never suppress an error); any
**auto-recovery profile** is **declared + bounded (I3/I4)** and **opt-in (I5)** (no implicit control-flow
change — the RFC-0014 tests, over profiles); the **derived policy is content-addressed + `EXPLAIN`-able**
(A3); and the derivation is a **total function** of the mapping (A4, no learned/opaque step). Advances
**NFR-2/SC-5b** (easier feedback loop), **SC-3** (transparent control), **G2/VR-5** (never-silent, honest).

## 6. Drawbacks

- **A baseline can lull.** Auto-logging that "just works" could let a developer ignore an error they
  should handle. Mitigated by A1 (the error still *propagates* — the baseline only logs it) and by
  `EXPLAIN` surfacing exactly what the baseline does.
- **Derivation is design work.** A *sensible* default mapping is non-trivial (which class logs where, at
  what level). Mitigated by reusing RFC-0013's existing registry defaults and keeping the derivation
  total + inspectable (§8 is where it is pinned down).

## 7. Prior art

DynEL (the automated-baseline + dynamic-application inspiration; DN-04); Rust's `tracing`/`log` + default
subscribers (zero-config logging that's still explicit); Erlang/OTP default loggers + SASL reports
(baseline supervision/logging with opt-in customization); Python `logging.basicConfig` (the zero-config
baseline, minus the honesty discipline); structured-logging frameworks (auto-context). Full tracing into
`research/` is a pre-ratification task (as for RFC-0013/0014).

## 8. Unresolved questions

- **The derivation function.** The exact baseline mapping (class → level/route, definition → wrapped
  presentation) — closed default set, and how a `phylum` (the M-359 manifest) configures it.
- **Auto-recovery profiles.** The closed set of named, opt-in, bounded recovery profiles (e.g.
  `resilient`/`strict`) and their declaration surface (RFC-0006, KC-2-gated).
- **Scope of auto-wrapping.** Per-definition vs per-`nodule` vs per-`phylum`; default-on vs default-off
  (bias: presentation default-on — A1 makes it safe; recovery default-off — A2).
- **Self-hosting & toolchain.** How the auto-baseline integrates with the M-361 toolchain (lint
  "unhandled class with no baseline route") and the M-346 stdlib.

## 9. Future possibilities

- **Project-level profiles** in `mycelium-proj.toml` (M-359): a phylum declares its baseline logging/recovery
  posture once, inherited top-down by its nodules.
- **Fix-it suggestions** (M-361): the auto layer proposes an explicit handler skeleton as an actionable
  diagnostic ("this class is only logged; add a handler?").
- **Self-hosting** — the derivation written in Mycelium-lang, consuming its own registry/effect mapping.

## Meta — changelog

- **2026-06-16 — Draft (Proposed).** Captures the DynEL "automated baseline error handling + logging"
  design point the maintainer added to the roadmap: an **automation layer over RFC-0013/0014** that
  auto-derives a zero-config **baseline** diagnostic/logging policy from the language's structured mapping
  (registry + routes + declared effects), auto-applies it (wrapping for logging/QoL), and offers a ladder
  of *light* overrides → *fully manual*. The load-bearing **honesty boundary** is fixed up front:
  automatic = **additive presentation/logging only** (safe to auto-apply because RFC-0013 never changes
  control flow — I1); **automatic recovery is opt-in, declared, bounded** (never an implicit control-flow
  change — RFC-0014 I3/I4/I5); the baseline is a **reified, `EXPLAIN`-able** policy (no black box, SC-3);
  the derivation is a **total, inspectable** function of the mapping, not learned (VR-5/RFC-0005).
  Tooling-layer; no kernel change (KC-3). Ratification + enactment is **M-362**; **no code** with this
  draft. Lineage: DN-04 → RFC-0013 + RFC-0014 → RFC-0015. Append-only.
