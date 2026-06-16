# Design Note DN-04 — Optional Structured Diagnostics & Error-Handling (DynEL-inspired)

| Field | Value |
|---|---|
| **Note** | DN-04 |
| **Status** | **Draft — investigation done, direction open** (2026-06-16). A design *direction* to evaluate, not a decision. **DynEL source read** (maintainer-supplied zip, `DynEL-main`) — findings in §6. |
| **Feeds** | RFC-0005 (selection-policy pattern — the reification model for declarative error-handling policies); RFC-0006 (optional surface for error handling); RFC-0008 (runtime/observability); the LSP feedback facade (M-140/M-310 `FeedbackSummary`); the never-silent error/refusal surface (RFC-0001/0002 `Option`/error, `CheckVerdict::NotValidated`); G2 (never-silent), G11 (multiple projections), NFR-2 (semantic feedback), KC-3 (small kernel); the security posture (`/security-review`: no `eval`, least-privilege, no secret leakage) |
| **Source** | DynEL — `gitlab:albedo_black/DynEL` (read from the maintainer's `DynEL-main` zip, 2026-06-16): a WIP Python structured/formatted error-logging wrapper over Loguru. ~219 LOC single module (`src/dynel/dynel.py`) + JSON/YAML/TOML config + tests. Maintainer wants its **feature set available as *optional* ways to handle errors/logging in Mycelium.** |
| **Question** | Which DynEL capabilities map cleanly onto Mycelium's existing never-silent / EXPLAIN / provenance surfaces as **opt-in** features, and what are the constraints that keep them from eroding the honesty rules? |

> Decision-support note: it frames the direction, maps DynEL's features onto existing Mycelium machinery, and isolates the one constraint that governs the whole idea. Ratification (and any RFC this feeds) is the maintainer's, after the source investigation in §6.

---

## 1. The governing constraint (read this first)

Mycelium's errors are **load-bearing semantics**, not telemetry: a swap out of range, a failed
certificate, an unresolved name are **explicit `Option`/error/refusal** values (G2 never-silent;
`CheckVerdict::NotValidated` carries a *reason* and a *fallback*). DynEL is a **logging/diagnostics**
layer. So the one rule that governs everything below:

> **Diagnostics are *additive presentation* over explicit errors — never a substitute for one.** A
> log line, a context level, a tag, or a formatted report may *describe* a refusal more richly; it may
> **never** swallow, soften, or stand in for the explicit error/`Option` that the never-silent rule
> requires. Logging that *replaced* an error would be the very black box the project forbids.

Under that rule, DynEL's feature set is a natural fit — Mycelium already produces *structured,
reasoned* errors (`NotValidatedReason`, `Provenance`, `Meta`); DynEL's value is in **graded,
dual-format, declaratively-configured presentation** of exactly that kind of structured diagnostic.

## 2. Feature map — DynEL → Mycelium (all opt-in)

| DynEL feature | Mycelium analogue / home | Fit & constraint |
|---|---|---|
| **Graded context levels** (`minimal`/`medium`/`detailed`) | a verbosity knob over `EXPLAIN` dumps, the LSP `FeedbackSummary` (M-140/M-310), and `NotValidatedReason` detail | Strong fit — Mycelium errors already carry tiered detail; this exposes it as a level. The level changes *how much* is shown, never *whether* the error exists. |
| **Human + machine-readable (JSON) output** | two projections of one content-addressed diagnostic (G11; M-380 projection framework) | Strong fit — the EXPLAIN/IR are already content-addressed; a JSON projection is a renderer, not new truth. Dual-format must round-trip to the *same* diagnostic. |
| **Per-function exception config** (`exceptions`, `custom_message`, `tags`) | a **reified error-handling policy** attached to a definition — same pattern as RFC-0005 selection policies (reified, inspectable, content-addressed; ADR-006) | Fit *iff* reified: a policy that maps an error class → (message, tags, routing) must be an inspectable artifact, `EXPLAIN`-able, not hidden control flow. `tags` → categorization/routing/severity metadata on a diagnostic. |
| **Multi-format config** (JSON/YAML/TOML) | config ingestion in the tooling layer | Fit, but Mycelium prefers **content-addressed declarations** over free-floating config (cf. R7-Q4 — the prim table as content-addressed declarations); a config file is a *projection of* the canonical declaration, not the source of truth. |
| **CLI configurability** | `cargo xtask` / tooling flags (the build-target CLI follow-on) | Fit — purely tooling-layer. |
| **Loguru/Python backend** | **reference only — not ported.** DynEL is Python; the Mycelium feature is a **Rust** implementation (ADR-007 Rust-first; the maintainer wants *minimal* Python). A Rust diagnostics renderer (e.g. over the `tracing`/`tracing-subscriber` structured+JSON layers, or a small in-house renderer) lives in the **Rust tooling crates** (`mycelium-lsp`/`xtask`), never in the trusted kernel (KC-3 — no logging dep in the kernel). | DynEL supplies the *feature contracts*, not a line of code to port. |

## 3. Where it plausibly lands (without touching the kernel)

**Implementation language (maintainer-directed).** DynEL is reference inspiration only — **no Python
is added** (the project minimizes Python; ADR-007 is Rust-first). The feature is built in **Rust**, in
the tooling crates. *Trajectory:* Rust now → **eventually self-hosted in Mycelium-lang itself**
(dogfooding), as part of the long-term goal of being free of *other languages* (Python, then Rust),
not other repos. A structured-diagnostics renderer over Mycelium's own reasoned errors is a natural,
high-value dogfooding target once the surface language is self-hosting.

- **Tooling / observability layer**, not L0. The trusted interpreter + checkers keep emitting the
  same explicit, reasoned errors; a DynEL-style **diagnostic renderer** (Rust) consumes them and
  produces graded, dual-format, tagged output. This keeps KC-3 intact (the kernel stays small and
  dependency-light) and makes the feature genuinely optional (absent the renderer, you still get the
  raw explicit errors).
- **Reified error-handling policy** as an opt-in declaration (RFC-0005 pattern): `on <ErrorClass> =>
  { message, tags, level }` attached to a definition, content-addressed and `EXPLAIN`-able. Crucially
  it configures *presentation/routing*, not whether the error propagates.
- **A projection** (G11 / M-380): human vs JSON is two views of one content-addressed diagnostic —
  the same "expand/collapse" stance RFC-0012 takes for ambient reprs.

## 4. Why this is attractive

- It operationalizes **NFR-2 / SC-5b** (semantic feedback) and the **AI co-author loop** (M-330): a
  graded, machine-readable diagnostic stream is exactly what a generate→feedback→self-correct loop
  consumes. DynEL's JSON + tags + levels are a ready-made shape for that channel.
- It improves DX **in the same key as RFC-0012**: reduce noise by default (minimal level), reveal
  detail on demand (detailed level / EXPLAIN), while the underlying truth stays explicit and
  inspectable.

## 5. Open questions

- **DN04-Q1 — policy vs handler.** Is a per-definition error policy *only* presentation/routing
  (safe), or does the maintainer also want declarative *recovery* (custom fallback)? Recovery edges
  toward control flow and must be reconciled with never-silent + the totality story — needs care.
- **DN04-Q2 — tags as first-class metadata.** Do `tags` become a typed field on diagnostics/`Meta`
  (queryable, content-addressed) or a free-form string set? First-class is more useful and more
  honest, but more spec.
- **DN04-Q3 — config source of truth.** Reconcile DynEL's file-config ergonomics with Mycelium's
  content-addressed-declaration preference (R7-Q4): file as projection vs file as source.
- **DN04-Q4 — kernel boundary.** Confirm all of this is tooling-layer (no kernel logging dep, KC-3);
  the kernel only needs to keep emitting *structured, reasoned* errors (it already does).
- **DN04-Q5 — stdlib home.** This is an early candidate for the **Mycelium core library** (the
  maintainer's stdlib-for-usability goal, M-346): a `diagnostics` module is a clean, self-contained
  first stdlib citizen and an excellent dogfooding target (a Mycelium program consuming Mycelium's own
  errors). Should DN-04 graduate into the stdlib RFC rather than a standalone feature?

## 6. Investigation findings (source read 2026-06-16)

The module is one file (`src/dynel/dynel.py`, ~219 LOC). Concrete contracts:

- **`ContextLevel`** = `MINIMAL | MEDIUM | DETAILED`, with a string map (`min/minimal`, `med/medium`,
  `det/detailed`). `handle_exception` graduates the context it gathers: minimal → `timestamp`; medium
  → `+ local_vars` (stringified `frame.f_locals`); detailed → `+ free_memory, cpu_count, env_details`.
- **`DynelConfig`** holds `CUSTOM_CONTEXT_LEVEL`, `DEBUG_MODE`, `FORMATTING_ENABLED`, `panic_mode`
  (param accepted, currently unused), and **`EXCEPTION_CONFIG`**: `func_name → { exceptions[],
  custom_message, tags[] }`.
- **`configure_logging`** = two Loguru sinks: `dynel.log` (human, colorized format, 10 MB rotation)
  and `dynel.json` (`serialize=True` → machine-readable, rotation). *Dual-format is literally two
  sinks over one event* — which is exactly the "two projections of one diagnostic" model (§2/G11).
- **`module_exception_handler`** wraps every module function via `logger.catch(...)`.
- CLI: `--context-level`, `--debug`, `--no-formatting`.

**The good (worth importing — the *intent*):** graded context levels, the dual human/JSON sink model,
and the **declarative per-function `{exceptions, custom_message, tags}`** table are all clean,
valuable shapes that map onto Mycelium's existing structured-error surfaces (§2).

**The WIP gap (intent ≠ implementation):** `handle_exception` *computes* `custom_context` and the
`detailed_context`, and reads `function_config`, but then only calls `logger.exception(error_message)`
— **the gathered context, the `custom_message`, and the `tags` are never actually attached to the
emitted record.** So today the feature set is *designed* more than *delivered*; Mycelium would import
the **contracts**, not this code.

**What Mycelium must NOT import (anti-patterns — `/security-review`, G2):**

1. **`eval(exception_str)`** in `load_exception_config` (turning config strings into exception
   classes via `eval`) is **arbitrary code execution from a config file** — a supply-chain/injection
   hole. Mycelium maps error classes through a **registry/known-set lookup**, never `eval`.
2. **`env_details: dict(os.environ)`** at the *detailed* level dumps the **entire environment
   (secrets included) into a log** — a secret-leakage smell. Any Mycelium "detailed" tier must
   **allowlist** context, never wholesale-dump env/locals.
3. **`logger.catch`-style swallowing.** Wrapping functions so exceptions are caught-and-logged
   risks turning an error into *only* a log line — the exact never-silent violation §1 forbids. In
   Mycelium, diagnostics are additive over an explicit error that **still propagates**.

**Next, to turn this into an RFC:** extract the three "good" contracts (level / dual-format / reified
per-definition `{classes, message, tags}` policy), explicitly exclude the three anti-patterns, decide
policy-vs-handler scope (DN04-Q1), and draft an RFC feeding RFC-0008 (observability) + an
RFC-0005-style reified error-handling policy — present before folding.

## Meta — changelog

- **2026-06-16 — Draft (investigation done; direction open).** Created at the maintainer's request to
  capture DynEL (`gitlab:albedo_black/DynEL`) as an **optional** structured-diagnostics /
  error-handling direction for Mycelium. Records the governing constraint (diagnostics are additive
  over explicit errors, never a substitute — G2), maps DynEL's feature set onto existing Mycelium
  surfaces (EXPLAIN / `FeedbackSummary` / `NotValidatedReason` / reified RFC-0005 policies / G11
  projections), and proposes a tooling-layer home that leaves the kernel untouched (KC-3). **Updated
  same day** after reading the maintainer-supplied source (`DynEL-main` zip): §6 now records the
  concrete contracts worth importing (graded levels / dual-sink human+JSON / reified per-function
  `{exceptions, custom_message, tags}`), the WIP gap (the context/tags are computed but not yet wired
  into the emitted record), and three **anti-patterns to exclude** (`eval` on config strings; full
  `os.environ` dump at the detailed level; `logger.catch` exception-swallowing) per the security
  posture + never-silent. **Maintainer clarifications (same day):** the feature is implemented in
  **Rust** (no Python added — ADR-007 Rust-first; DynEL is reference-only), and **eventually
  self-hosted in Mycelium-lang** as part of the dogfooding goal of being free of *other languages*
  (not repos). Append-only; not yet resolved. Tracked as M-345 (#107).
