# RFC-0013 — Structured Diagnostics & Reified Error-Handling Policy

| Field | Value |
|---|---|
| **RFC** | 0013 |
| **Status** | **Draft (Proposed)** (drafted 2026-06-16; ratifies the DN-04 direction — maintainer sign-off pending) |
| **Type** | Foundational / normative (once Accepted) — tooling/observability-layer feature; no kernel change |
| **Date** | 2026-06-16 |
| **Feeds** | RFC-0008 (runtime/observability — the diagnostic stream lives here); the AI co-author loop (M-330); the stdlib `diagnostics` candidate (M-346) |
| **Depends on** | DN-04 (the design basis this RFC extracts and ratifies); RFC-0005 (the reified-policy pattern — `on <ErrorClass> => {…}` reuses it; ADR-006 reified, inspectable, content-addressed); RFC-0006 §3/§4 (the optional surface this rides; tooling-layer, not L0); RFC-0008 (runtime/observability home); RFC-0001/0002 (the never-silent `Option`/error/refusal and `CheckVerdict::NotValidated`/`NotValidatedReason` this presents); the LSP `FeedbackSummary` facade (M-140/M-310); ADR-007 (Rust-first); G2 (never-silent), G11 (multiple projections), NFR-2 / SC-5b (semantic feedback), KC-3 (small kernel); the security posture (`/security-review`: no `eval`, least-privilege, no secret leakage) |
| **Tracks** | M-345 (#107) |

---

## 1. Summary

Mycelium already produces **structured, reasoned errors** — a swap out of range, a failed certificate, an
unresolved name, a `CheckVerdict::NotValidated` carrying a *reason* and a *fallback* are all explicit
`Option`/error/refusal values (G2 never-silent). What it lacks is a *presentation* layer over them: a way
to render that one structured truth at graded verbosity, in both human and machine form, and to attach a
per-definition, inspectable policy that shapes the message, tags, level, and routing of a diagnostic.

This RFC ratifies the DN-04 direction (DynEL-inspired, source-investigated 2026-06-16) into a ratifiable
design. It imports **three** good contracts — graded context levels, dual human + JSON projection of one
content-addressed diagnostic, and a reified per-definition error-handling policy in the RFC-0005 pattern —
and **explicitly excludes** three anti-patterns found in the source (config-string `eval`, wholesale
env/locals dump at the detailed tier, and `logger.catch`-style swallowing). The whole feature rests on one
governing principle (DN-04 §1):

> **Diagnostics are *additive presentation* over an explicit error — never a substitute for one.** A level,
> a tag, a message, or a routing target may *describe* a refusal more richly; it may **never** swallow,
> soften, or stand in for the explicit `Option`/error/refusal the never-silent rule (G2) requires.

Per the maintainer (2026-06-16), v0 is scoped to **presentation/routing only**: the reified policy sets
message / tags / level / output route, and the explicit error **still propagates** unchanged. Declarative
*recovery* (custom fallback/handler) edges into control flow and is **deferred** to a future revision (§8).
The implementation is **Rust, tooling-layer only** (`mycelium-lsp` / `xtask`) with **no kernel logging
dependency** (KC-3, DN04-Q4) and **no Python** (ADR-007 Rust-first; DynEL is reference-only, not ported).

## 2. Motivation

- **It operationalizes NFR-2 / SC-5b (semantic feedback) and the AI co-author loop (M-330).** A graded,
  machine-readable diagnostic stream is exactly what a generate → feedback → self-correct loop consumes:
  JSON + tags + levels are a ready-made shape for that channel (DN-04 §4). Today a tool sees the raw
  reasoned error but has no stable, projected, tagged surface to consume.
- **It improves DX in the same key as RFC-0012.** Reduce noise by default (minimal level), reveal detail on
  demand (detailed level / EXPLAIN), while the underlying truth stays explicit and inspectable — the same
  "elide, never hide" stance RFC-0012 takes for ambient reprs (tension A).
- **It is the location-independent answer to representation-crossing auditability.** RFC-0012 routed its
  R12-Q2 auditability need here (M-351): the value of "where do my lossy/precision-changing conversions
  live, and under what honesty bound?" can be delivered as a *structured diagnostic view* without
  constraining where swaps sit in a program (§4.6).
- **It is a clean first stdlib / dogfooding citizen.** A `diagnostics` module that consumes Mycelium's own
  reasoned errors is a self-contained candidate for the core library (M-346) and an eventual self-hosting
  target (DN-04 §3), though v0 does not block on either.

## 3. Guide-level explanation

The trusted kernel keeps emitting the same explicit, reasoned errors it always has. A **diagnostic
renderer** (Rust, tooling layer) consumes one of those and presents it. Two knobs and one declaration:

```text
-- A graded context level chooses HOW MUCH of one diagnostic is shown — never WHETHER it exists.
--   minimal : the refusal + its reason + its site
--   medium  : + the NotValidatedReason detail / FeedbackSummary expansion
--   detailed: + an ALLOWLISTED set of additional context fields (never a wholesale env/locals dump)

-- One diagnostic, two projections (G11). Human and JSON are two renderers of ONE
-- content-addressed truth, and they must round-trip to the same diagnostic:
--   human : a formatted, colorizable report
--   json  : a serialized record { id, class, message, tags, level, site, reason, route, … }

-- A reified per-definition error-handling policy, in the RFC-0005 pattern:
on SwapOutOfRange => {
  message: "value left the certified range here",   -- presentation only
  tags:    { "swap", "range", "review" },            -- free-form string set (v0)
  level:   detailed,
  route:   diagnostics_channel                        -- routing only; the error STILL PROPAGATES
}
```

Reading rules a developer internalizes:

- **A policy never suppresses an error.** `on <ErrorClass> => {…}` configures *presentation and routing*
  of the diagnostic that accompanies an error; the explicit `Option`/error/refusal value propagates exactly
  as it would with no policy at all (G2). Absent the renderer entirely, you still get the raw reasoned error.
- **The level is a verbosity knob, not a gate.** Lowering the level shows *less of* one diagnostic; it can
  never make the error *not exist*. The minimal level still names the refusal and its reason.
- **Error classes are looked up, never evaluated.** A policy names an error class from a **known registry**;
  the renderer resolves the name through that registry. There is no `eval` of config strings (§4.5).
- **The detailed tier is allowlisted.** "Detailed" adds a fixed, declared set of context fields — never a
  full environment or locals dump (§4.5).
- **The declaration is the source of truth; a file is a projection.** A JSON/YAML/TOML policy file is a
  *projection of* the canonical content-addressed declaration, not the source of truth (§4.4, DN04-Q3).

## 4. Reference-level design (normative once Accepted)

### 4.1 The governing invariant — additive, never substitutive (G2)

**(I1) A diagnostic is additive over an explicit error and never replaces it.** For every error/refusal
`e` the kernel or checker emits, applying any diagnostic policy `π` yields a *presentation* `present(π, e)`
while `e` itself still propagates to the caller unchanged. There is **no policy, level, tag, route, or
message that can cause `e` not to surface.** This is the operational form of never-silent (G2) for this
subsystem, and it is the normative core defended by the §5 verification (the never-silent invariant test).

The renderer is therefore a *pure function of an already-emitted error* plus configuration; it is
structurally incapable of catching, softening, or standing in for that error (contrast the
`logger.catch` anti-pattern, §4.5).

### 4.2 Graded context levels (verbosity over one truth)

A diagnostic carries a **level** `∈ { minimal, medium, detailed }`, a verbosity knob over the *existing*
tiered detail of Mycelium errors — the `EXPLAIN` dump, the LSP `FeedbackSummary` (M-140/M-310), and the
`NotValidatedReason` (DN-04 §2):

- **minimal** — the refusal, its reason, and its site. The error is always present at this level.
- **medium** — adds the `NotValidatedReason` detail / `FeedbackSummary` expansion.
- **detailed** — adds an **allowlisted** set of additional context fields (§4.5), never a wholesale dump.

**(I2) The level changes *how much* of a diagnostic is shown, never *whether* the underlying error
exists.** This is a corollary of I1 and is normative: no level may elide the existence of the refusal.

### 4.3 Dual human + JSON projection of one content-addressed diagnostic (G11)

A diagnostic is **one content-addressed value**; "human" and "JSON" are **two renderers of one truth**
(G11; the M-380 projection framework), the same expand/collapse stance RFC-0012 takes for ambient reprs.
This mirrors the DynEL dual-sink model (one event → a human sink + a `serialize=True` JSON sink, DN-04 §6)
but as *projections of a single content-addressed record*, not two independently-written outputs.

**(I3) The two projections round-trip to the same diagnostic.** The human and JSON renderings of one
diagnostic must be recoverable to — and carry the same content-addressed identity as — the same underlying
diagnostic. Neither projection is "new truth"; both are views of the one record (the §5 round-trip test
is the executable proof). A JSON projection is a renderer, not a second source.

### 4.4 The reified per-definition error-handling policy (RFC-0005 pattern; ADR-006)

A per-definition error-handling policy is a **reified, inspectable, content-addressed artifact**, in the
same posture ADR-006 mandates for selection policies and RFC-0005 mandates for swap/packing selection. It
is **not hidden control flow**: it is an `EXPLAIN`-able declaration attached to a definition.

```text
on <ErrorClass> => { message?, tags?, level?, route? }
```

- **`<ErrorClass>`** is resolved through the **error-class registry** (§4.5) — a name in a known set, never
  an evaluated string. An unknown class name is an explicit configuration error (never silently ignored,
  never coerced).
- **`message`** sets a presentation message for the diagnostic; it does not change the error's identity or
  its reason.
- **`tags`** is a **free-form set of strings** for v0 (KISS/YAGNI; DN04-Q2) — categorization / routing /
  severity hints carried on the diagnostic. First-class typed/queryable tags are a recorded future
  possibility (§9), not v0.
- **`level`** sets the default verbosity (§4.2) for diagnostics under this policy.
- **`route`** names an output target for the diagnostic (e.g. a diagnostics channel vs. the default sink).
  Routing concerns *where the presentation goes*, never *whether the error propagates* (I1).

**(I4) The policy configures presentation/routing only.** It carries no recovery, no fallback, no handler,
and no control-flow effect. The explicit error/`Option`/refusal it attaches to propagates unchanged (I1).
This is the v0 disposition of DN04-Q1 (recovery deferred — §8).

Every emitted diagnostic records the content hash of the policy that shaped it (the `PolicyRef`-style move
of RFC-0005 §3): one can always answer *"which policy shaped this diagnostic, and what does that policy
do?"* via `EXPLAIN`. A policy is content-addressed, diffable, and inspectable like any RFC-0005 policy.

### 4.5 Excluded anti-patterns (normative exclusions — `/security-review`, G2)

The DN-04 source investigation (DN-04 §6) found three patterns that **must not** be imported. Excluding
them is normative:

- **(X1) No `eval` of config strings.** DynEL's `eval(exception_str)` turns config strings into exception
  classes — arbitrary code execution / injection from a config file (a supply-chain hole). Mycelium maps an
  error-class name through a **registry / known-set lookup only**; there is no path from configuration text
  to evaluated code. An unknown name is an explicit error.
- **(X2) No wholesale env/locals dump at the detailed level.** DynEL's `env_details: dict(os.environ)` at
  the detailed tier dumps the entire environment (secrets included) into a log — a secret-leakage smell.
  Mycelium's detailed tier (§4.2) is an **allowlist**: a fixed, declared set of context fields, never a
  full `os.environ` / `f_locals` dump. Context not on the allowlist is not gathered.
- **(X3) No `logger.catch`-style swallowing.** Wrapping definitions so that an exception is
  caught-and-logged risks turning an error into *only* a log line — the exact never-silent violation §4.1
  forbids. In Mycelium, the renderer is a pure presentation of an error that **still propagates** (I1); it
  never wraps or intercepts control flow.

### 4.6 The representation-crossing audit view (routed from RFC-0012 R12-Q2 / M-351)

RFC-0012 deliberately kept representation crossings at free, first-class **swap sites** rather than
constraining them to block edges, and routed the *auditability* value here (RFC-0012 §8 R12-Q2, M-351).
This RFC carries that view as a concrete structured diagnostic this machinery produces:

A **representation-crossing audit view** enumerates, for a program or module, **every `swap`** (every
representation crossing) together with each crossing's:

- **location** (site), **from-repr** and **to-repr**;
- the crossing's **honesty bound** on the lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` (VR-5), reported
  per-crossing — swaps are exactly where lossy / precision-changing conversions live, so this is where the
  honesty bound is load-bearing for review and security;
- the **selection policy** that chose the target, if any (the RFC-0005 `PolicyRef`).

**(I5) The audit view is location-independent.** It enumerates crossings *wherever they sit* and therefore
delivers the auditability value **without constraining where crossings live** — exactly the property
RFC-0012 R12-Q2 required (it must not fight the language's fluid, paradigm-agnostic traversal). It is a
read-only projection (a structured diagnostic in the §4.3 dual-projection form), not a control over
placement. The view reports honesty bounds it reads off each crossing's `Meta`; it never *upgrades* one
(VR-5) — an unproven bound stays `Empirical`/`Declared` in the view.

### 4.7 Config source of truth (DN04-Q3)

Mycelium prefers **content-addressed declarations** over free-floating config (cf. R7-Q4 — the prim table
as content-addressed declarations). A JSON/YAML/TOML policy file is a **projection of** the canonical
declaration, **not** the source of truth: the declaration is the content-addressed artifact (§4.4); a file
is one serialization of it, ingested in the tooling layer. This keeps the policy diffable, inspectable, and
identity-stable (RFC-0001 §4.6) regardless of which on-disk format a user prefers.

### 4.8 Implementation boundary (KC-3, ADR-007; DN04-Q4)

- **Rust, tooling layer only.** The renderer, the registry, the policy artifact, and the audit view live in
  the Rust tooling crates (`mycelium-lsp` / `xtask`) — over a structured + JSON rendering substrate (e.g.
  the `tracing` / `tracing-subscriber` layers, or a small in-house renderer). The **trusted kernel keeps
  emitting its existing structured/reasoned errors and gains no logging dependency** (KC-3, DN04-Q4).
- **No Python.** DynEL is reference-only and is not ported (ADR-007 Rust-first; the project minimizes
  Python). The feature imports DynEL's *contracts*, not a line of its code.
- **Trajectory, not v0:** eventually self-hosted in Mycelium-lang (a `diagnostics` module consuming
  Mycelium's own reasoned errors — DN-04 §3) as a dogfooding target. Recorded as direction (§9), not scope.

## 5. Verification (per CONTRIBUTING — which FR/NFR/VR/SC, and how)

This RFC advances **NFR-2 / SC-5b** (semantic feedback) and the **AI co-author loop** (M-330), and is
orthogonal to the guarantee lattice (it *reports* honesty bounds, never sets them — VR-5). The normative
invariants I1–I5 are verified, when the tooling lands, by:

- **Never-silent invariant test (I1/I2/I4) — the central one.** A property test asserting that for a corpus
  of emitted errors/refusals, applying *any* policy `π` (any message/tags/level/route, including a policy
  that names a route or a minimal level) leaves the underlying explicit error **still propagating** — a
  policy can never suppress, swallow, or gate the error. A mutant renderer that dropped the error is caught.
- **Round-trip projection test (I3).** A property test that the human and JSON projections of one diagnostic
  round-trip to — and share the content-addressed identity of — the same underlying diagnostic (two views,
  one truth; G11).
- **Registry-lookup test (X1).** A test that error-class names resolve only through the registry and that
  there is **no `eval`/code-exec path** from configuration text; an unknown class name is an explicit error.
- **Allowlist test (X2).** A test that the detailed tier gathers only allowlisted context fields and never a
  wholesale env/locals dump (no secret-bearing field reachable that is not on the allowlist).
- **Audit-view tests (I5/VR-5).** Tests that the representation-crossing view enumerates every `swap`
  regardless of placement, reports each crossing's honesty bound as read off `Meta` without upgrading it,
  and is a read-only projection.

## 6. Drawbacks

- **More surface and a new tooling crate / module to maintain** — a real KISS/YAGNI cost, accepted because
  it operationalizes the semantic-feedback / AI-co-author requirement (NFR-2 / M-330) and the kernel is
  untouched (KC-3).
- **A footgun if I1 is ever weakened.** If a future revision let a policy *recover* (catch/fallback), the
  additive-not-substitutive property could be lost and a diagnostic could become a black box that hides an
  error. I1 is therefore a **normative invariant** (§4.1), and recovery is an explicit open question (§8),
  not a quiet extension.
- **Local readability now depends on a non-local policy.** A definition's diagnostic presentation depends on
  an `on <ErrorClass>` policy that may be declared elsewhere. Mitigation (recommended, not normative):
  `EXPLAIN` always names the shaping policy and its content hash (§4.4); the default presentation is
  *configured*, never *hidden*.

## 7. Prior art

- **DynEL** (`gitlab:albedo_black/DynEL`, source-read 2026-06-16; DN-04 §6) — the direct inspiration:
  graded `ContextLevel` (`MINIMAL`/`MEDIUM`/`DETAILED`), a dual human + JSON (`serialize=True`) sink model,
  and a declarative per-function `{exceptions, custom_message, tags}` table. Mycelium imports these
  *contracts* (§4.2–§4.4) and excludes DynEL's `eval` / env-dump / `logger.catch` patterns (§4.5). DynEL's
  WIP gap (context/tags computed but not wired into the emitted record, DN-04 §6) is moot here — Mycelium
  builds from the contracts, not the code.
- **RFC-0005 selection policies / ADR-006** — the reification pattern this RFC reuses: a policy as a
  first-class, content-addressed, inspectable, `EXPLAIN`-able artifact, recording its `PolicyRef` on each
  decision. The error-handling policy is the same move applied to *presentation/routing* instead of
  *swap/packing selection*.
- **RFC-0012 ambient representation** — the same expand/collapse, elide-never-hide projection stance (G11),
  and the source of the §4.6 audit-view requirement (R12-Q2 / M-351).
- **Database `EXPLAIN`/observability tooling** — structured, queryable views over engine internals; the
  Mycelium stance (RFC-0005 §2) is that such views are fine *iff* reified and inspectable, never opaque.

## 8. Unresolved questions

- **DN04-Q1 (recovery) — DEFERRED, open for a future revision (direction recorded).** v0 is
  presentation/routing only (§4.4 I4); declarative *recovery* (custom fallback/handler) is **not** in
  scope. The maintainer (2026-06-16) **does want** error-driven control flow eventually — errors that
  **propagate / bubble up** and can **trigger functionality** (the things errors do across programming
  today: fallback, retry, cleanup, branch selection — and, ideally, novel uses too). The governing
  constraints on that future work, per the maintainer, are:
  - **Isolation / separation of concerns.** Recovery must be its **own, separable subsystem** with a
    **bounded blast radius** — not woven into the diagnostics renderer (this RFC) or the kernel. A
    presentation policy (this RFC) and a recovery mechanism are *different concerns* and must stay
    decoupled, so that adding recovery later cannot destabilize either the renderer or the trusted base.
  - **Effects are explicit, declared, and bounded — known, never unintended.** The goal is *not* to
    forbid side effects or even cascades: sometimes a cascade is exactly what a developer wants. The line
    is that any such effect must be **explicitly declared and implemented**, so it is **known and
    bounded** — never an *unintended/unknown* effect and never an *unbounded* one (no memory explosion,
    no runaway cascade, no spooky action at a distance). The default behaviour is **tightly scoped,
    bounded** effects; a developer **opts into broader effect/cascade behaviour by explicitly declaring
    and implementing it** (it does not arrive implicitly). The shape that fits Mycelium's posture is
    **errors-as-explicit-values** that propagate and are matched/handled at an explicit site (an
    `Option`/result-style or reified-effect-handler surface where the effect is *named*), never an
    invisible unwinding that swallows the error or triggers undeclared work.
  - **Never-silent preserved (G2).** A handler must remain *additive over* — never *substitutive for* —
    the explicit error: recovery may *act on* a propagating error (and may itself produce a new explicit
    outcome), but it may not make the original refusal vanish unobserved. I1 (§4.1) is the line a recovery
    design must not cross without superseding this RFC (append-only).

  How Mycelium admits this — surface, semantics, and the totality interaction — is designed in the
  **separate** RFC **RFC-0014 (Declarative Error Recovery & Bounded Effects)**, not a v0 gap here and not
  an extension of §4.4. RFC-0014 does **not** weaken this RFC's I1 (it generalises *additive* from
  presentation to control: a handler acts on the error explicitly and either recovers or re-propagates).
- **DN04-Q2 (first-class tags) — future.** Tags are a free-form string set in v0 (§4.4). Whether they become
  a typed, queryable, content-addressed field on diagnostics/`Meta` (more useful, more honest, more spec) is
  recorded as a future possibility (§9).
- **DN04-Q5 (stdlib graduation) — future.** This is drafted as a standalone RFC now; whether a `diagnostics`
  module graduates into the stdlib RFC (M-346) is an open option, not a v0 blocker (§9).
- **Registry scope (genuinely open).** The exact membership and extension discipline of the error-class
  registry (§4.5) — which classes are nameable, and how a downstream module registers its own — is left
  open for the implementation task; v0 requires only that resolution is registry-based, never `eval`-based.
- **Route targets (genuinely open).** The concrete set of `route` targets (§4.4) and how they compose with
  RFC-0008's runtime/observability sinks is left to the RFC-0008 integration; v0 requires only that routing
  never affects propagation (I1).

## 9. Future possibilities

- **First-class typed tags** (DN04-Q2): a typed, queryable, content-addressed tag field on diagnostics /
  `Meta`, enabling tag-based queries over the diagnostic stream.
- **Declarative recovery** (DN04-Q1): an opt-in, reified recovery/fallback surface — errors that
  **propagate / bubble up** and **trigger functionality** (fallback, retry, cleanup, branch selection,
  and novel uses), built as an **isolated, separable subsystem** with a bounded blast radius (SoC), with
  **explicit, declared, bounded** effect semantics (errors-as-values / reified effect handlers — effects
  and even cascades are allowed *when explicitly declared and implemented*, so they stay known and
  bounded; the enemy is *unintended/unknown/unbounded* effects, not effects per se; default tightly
  scoped, opt into broader behaviour explicitly), *only* if it stays **additive over** the explicit
  error (never-silent G2, totality). This is now designed in its **own RFC — RFC-0014 (Declarative Error
  Recovery & Bounded Effects)** — which supersedes this RFC's §4.4 *scope boundary* for the recovery
  concern (append-only) without weakening I1; see §8 DN04-Q1 for the maintainer's recorded constraints.
- **Stdlib graduation** (DN04-Q5 / M-346): a self-contained `diagnostics` core-library module — a clean
  first stdlib citizen and dogfooding target.
- **Self-hosting** (DN-04 §3): the renderer eventually written in Mycelium-lang, consuming Mycelium's own
  reasoned errors — part of the long-term goal of being free of *other languages*.
- **Richer audit views** (§4.6): the representation-crossing view generalized to other structured-diagnostic
  reports (e.g. per-module honesty-bound summaries) over the same content-addressed projection machinery.

## Meta — changelog

- **2026-06-16 — Draft (Proposed).** Created from DN-04 (M-345, #107) — turns the DynEL-inspired
  structured-diagnostics direction into a ratifiable design. Imports the three DN-04 contracts (graded
  context **levels** as verbosity over EXPLAIN/`FeedbackSummary`/`NotValidatedReason`; **dual human + JSON
  projection** of one content-addressed diagnostic, G11; a **reified per-definition error-handling policy**
  in the RFC-0005 pattern, content-addressed and `EXPLAIN`-able) and **excludes** the three anti-patterns
  found in the source (config-string `eval` → registry lookup; wholesale env/locals dump → allowlisted
  detailed tier; `logger.catch` swallowing → additive-over-a-still-propagating-error), per `/security-review`
  and never-silent (G2). Records the maintainer's fixed decisions (2026-06-16): **DN04-Q1 = presentation /
  routing only** for v0 (a policy sets message/tags/level/route; the explicit error/`Option`/refusal STILL
  PROPAGATES — recovery is **deferred**, §8); **DN04-Q2 = free-form string tags** for v0 (typed tags future);
  implementation is **Rust, tooling-layer only** (`mycelium-lsp`/`xtask`; **no kernel logging dep**, KC-3,
  DN04-Q4; **no Python**, ADR-007; self-hosting as trajectory only); **DN04-Q3 = file is a projection** of
  the canonical content-addressed declaration; **DN04-Q5 = standalone RFC now**, stdlib graduation (M-346) a
  future option. Carries the **representation-crossing audit view** routed here from RFC-0012 R12-Q2 / M-351
  (every `swap` + its honesty bound + policy, location-independent — auditability without constraining where
  crossings live). Advances **NFR-2 / SC-5b** (semantic feedback) and the AI co-author loop (M-330);
  verification = round-trip projection tests (I3), registry-lookup / no-`eval` tests (X1), detailed-tier
  allowlist tests (X2), audit-view tests (I5/VR-5), and the central **never-silent invariant** test (a policy
  never suppresses the underlying error — I1/I2/I4). Feeds RFC-0008 (runtime/observability), the RFC-0005
  reification pattern, RFC-0006 (optional surface), the LSP `FeedbackSummary` (M-140/M-310), and
  G2/G11/NFR-2/KC-3. **No code lands with this draft** — ratification (Draft → Accepted) and the tooling
  wiring are the maintainer's append-only decision, presented here first. Append-only. Tracked as M-345
  (#107).
