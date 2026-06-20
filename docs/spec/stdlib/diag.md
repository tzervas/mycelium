# Spec — `std.diag` (structured diagnostics: additive presentation + trace over an explicit error, never a substitute for one)

| Field | Value |
|---|---|
| **Status** | **Accepted** (2026-06-20, maintainer-ratified per DN-07 — guarantee matrix asserted in tests; open §7/§8 questions are design/scope calls, not contract violations; was *Implemented (Rust-first) — pending ratification* 2026-06-18, Draft/needs-design 2026-06-17) — the Rust-first code landed as `mycelium-std-diag` (M-510, #151, Batch P5 Tier-A completion) over a newly-extracted **`mycelium-diag` kernel crate** that homes the canonical RFC-0013 record types (a maintainer-resolved FLAG: a small, deliberate trusted-base growth so the record has one owner below the std layer; the std crate re-exports it, KC-3). Guarantee matrix asserted in tests; `present` returns the error **unchanged** (the I1 structural proof); identity is content-addressed and presentation-invariant (ADR-003). The Mycelium-lang migration (M-502-gated) remains. |
| **Module / Ring** | `std.diag` · Ring `1` ([RFC-0016 §4.2](../../rfcs/RFC-0016-Core-Library-and-Standard-Library.md)) · Tier `A` ([RFC-0016 §4.3](../../rfcs/RFC-0016-Core-Library-and-Standard-Library.md)) |
| **Tracks** | `M-510` (#151) — the Phase-5 task this spec delivers: self-host the RFC-0013 structured-diagnostics layer (today `crates/mycelium-lsp/src/diagnostics`, Rust) in Mycelium-lang, preserving the never-silent invariant **I1** and the honest sink guarantees (RT5/VR-5). |
| **Scope** | The **diagnostic record + trace substrate**: a content-addressed structured diagnostic over an *already-emitted* explicit error (class · message · site/span · reason · graded level · free-form tags · allowlisted context · route · `PolicyRef`), its **dual human + JSON projection** (G11), the reified per-definition `on <ErrorClass> => {…}` presentation/routing policy over an error-class **registry** (no `eval`), the closed v0 **route → sink** vocabulary with honest delivery guarantees (RT5), and the representation-crossing **audit view**. This is the legibility substrate every other module's failure path records its diagnostic on. |
| **Boundary** | OUT of scope: (a) **recovery policy** — *what to do* about an error (fallback/retry/escalate/cleanup, bounded effects) is `std.recover` (M-520, the library form of [RFC-0014](../../rfcs/RFC-0014-Declarative-Error-Recovery-and-Bounded-Effects.md)); `diag` only *presents* the propagating error, it never acts on it (I4). (b) **errors-as-values ergonomics** — the `Option`/`Result` combinators + `?`-propagation are `std.error` (M-527); `diag` is the *record* an error carries, not the propagation glue. (c) **general human/machine formatting** — the canonical `fmt`-string + display surface is `std.fmt` (M-533); `diag` owns the diagnostic's *own* dual projection (G11), and where the two meet `diag` delegates the generic rendering primitives to `fmt` (FLAGGED §7-Q1). (d) the **error class/value representation** itself (the `Option`/`Result`/structured-error sums + the lattice tags) — `core` (M-515, re-exporting RFC-0001); `diag` *presents* those, it does not define them. |
| **Depends on** | [RFC-0013](../../rfcs/RFC-0013-Structured-Diagnostics-and-Reified-Error-Policy.md) (Accepted — Enacted; the design this module is the library/self-hosted form of — I1–I5, the §4.5 exclusions X1–X3); RFC-0016 §4.1 (the C1–C6 contract) / §4.2 (ring) / §4.5 (the matrix obligation) / §4.6 (the Rust-first → self-hosted migration); RFC-0001 (the value model — `Value`/`Repr`/`Meta`, the guarantee lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`, content-addressing §4.6); RFC-0008 §4.8 (the observability sinks the closed route set binds to — RT5). |
| **Grounds on** | the **landed Rust reference** `crates/mycelium-lsp/src/diagnostics` (M-345 / DN-04, Enacted 2026-06-16): `registry` (X1), `record` (the content-addressed `DiagnosticRecord`, dual human/JSON round-trip, graded `Level`, `present` — I1), `policy` (the reified `on <ErrorClass> => {…}` + content-addressed `PolicyRef`), `sink` (the closed v0 route vocabulary + honest `Delivery` — RT5/M-354), `audit` (the representation-crossing view — I5); `mycelium-core` (`ContentHash`, `Bound`/`BoundBasis`, `GuaranteeStrength`). The M-345 candidate-module anchor (M-346 names "diagnostics (DN-04/M-345)"). KC-3: above the kernel — the kernel gains **no** logging dependency; `diag` is a pure presentation consumer. |

---

## 1. Summary

`std.diag` is the **structured-diagnostic record + trace substrate**: it takes an already-emitted explicit
error/refusal and produces a *content-addressed diagnostic* — class, message, source site/span, reason,
graded verbosity level, tags, allowlisted context, route, and the `PolicyRef` of any policy that shaped it —
projectable to both a human report and a JSON record that round-trip to the same identity (G11). Its
**honesty crux** is RFC-0013 **invariant I1**, lifted into the library: **presentation never gates
propagation.** A diagnostic is *additive over* an explicit error and structurally incapable of replacing it —
there is no level, tag, route, message, or sink-failure that can make the underlying `Option`/error/refusal
*not* surface (`present` returns the error **unchanged**). Because every other stdlib module records its
failure legibility *here* (the trace, source span, and actionable debug info that make a refusal robust and
legible — never a silent swallow, G2), `diag` is the substrate the rest of the library's never-silent floor
rests on. Its second crux is the honest **sink guarantee** (RT5/VR-5): each route's delivery claim is tagged
on the lattice and **never upgraded** without a checked basis — the `null` sink honestly reports *not
delivered*, the `mesh` sink carries a declared probability bound. Ring 1, Tier A; it adds **no** trusted code
(KC-3) — it is a pure presentation/projection consumer of the explicit errors `core` and the capability
crates already emit, with the kernel untouched.

## 2. Scope & module boundary

- **In scope:**
  - The **`Diagnostic` record** — a content-addressed value over an explicit `ReasonedError`: `class`,
    `message`, `site`/source span, optional `reason`, `level ∈ {Minimal, Medium, Detailed}`, free-form string
    `tags`, **allowlisted** detailed-tier `context`, optional `route`, and the optional `PolicyRef` of the
    shaping policy (RFC-0013 §4.2/§4.3).
  - **`present`** — the never-silent renderer: a pure function of `(ReasonedError, Option<Policy>)` returning
    a `Presentation { diagnostic, error }` where `error` is the **untouched, still-propagating** explicit
    error (RFC-0013 §4.1 **I1**). This return *is* the structural proof that the renderer cannot suppress.
  - The **dual projection** (G11): `to_human` (graded by `level`) and `to_json` / `from_json`, both carrying
    the diagnostic's content `id` and round-tripping to the same content-addressed identity (RFC-0013 §4.3
    **I3**).
  - The **error-class registry** — `ClassName` is obtainable *only* by registry lookup; an unknown class is
    an explicit `UnknownClass`; **no `eval`** (RFC-0013 §4.5 **X1**).
  - The reified **`on <ErrorClass> => { message?, tags?, level?, route? }` policy** — content-addressed
    (`PolicyRef`), presentation/routing **only** (RFC-0013 §4.4 **I4**), with a `PolicyFile` projection that
    re-validates every class through the registry on ingest (§4.7).
  - The closed v0 **route → sink** vocabulary (`stream`/`audit`/`log`/`null`/`mesh`) with honest `Delivery`
    guarantees on the lattice (RFC-0013 §8 / RFC-0008; **RT5**); resolution is **checked** (an out-of-set
    route is an explicit `UnknownRoute`) and lives **outside** `present`, so a sink failure never gates
    propagation (I1).
  - The **representation-crossing audit view** — enumerates every `swap`'s site, from/to repr, honesty bound
    (read off the certificate, **never upgraded** — VR-5), and policy, location-independently (RFC-0013 §4.6
    **I5**).
- **Out of scope (and who owns it):**
  - **Recovery** (what to *do* about an error: fallback/retry/escalate/cleanup, bounded effects) —
    `std.recover` (M-520, library form of RFC-0014). `diag` presents; `recover` acts. RFC-0013 §4.4 **I4**
    and RFC-0014 keep these decoupled (the SoC the maintainer required, RFC-0013 §8 DN04-Q1).
  - **Errors-as-values ergonomics** (`map`/`and_then`/`?`-propagation, partial accessors) — `std.error`
    (M-527). `diag` is the *record an error carries when presented*; `error` is the *propagation glue*. An
    `unwrap`/`expect` refusal carries a `diag` record (the cross-module seam in error §7-Q3).
  - **General human/machine formatting** (the `fmt`-string / display surface) — `std.fmt` (M-533). `diag`
    owns the diagnostic's *own* dual projection; the generic rendering primitives it leans on are `fmt`'s
    (FLAGGED §7-Q1).
  - **The error class/value representation** + the lattice tags — `core` (M-515, re-exporting RFC-0001).
    `diag` consumes `ReasonedError`/`Bound`/`GuaranteeStrength`; it does not define them.
- **Ring & layering:** Ring 1, Tier A (RFC-0016 §4.2/§4.3) — a **capability surface** that consumes the
  landed `mycelium-lsp/src/diagnostics` Rust reference and the kernel's explicit errors, and re-authors it
  in Mycelium-lang (the self-hosting target, RFC-0016 §4.6 Phase 5b). It **wraps** the error values `core`
  already owns and **builds** the record/projection/policy/audit surface to the §4.1 contract; it adds **no**
  trusted code and gives the kernel **no** logging dependency (KC-3, RFC-0013 §4.8). The self-hosted form is
  differentialled against the Rust reference (NFR-7-style, RFC-0016 §4.6; §7-Q4).

## 3. Exported-op surface (design sketch)

A design sketch — value-semantic, immutable-by-default — enough to fix the surface and feed the §4 matrix,
**not** a committed grammar. It mirrors the landed Rust reference (`crates/mycelium-lsp/src/diagnostics`) so
the self-hosted form is a faithful re-authoring; the field set is RFC-0013 §4.2–§4.6. `present` returns the
error **with** the presentation (never a bare diagnostic) — the type-level proof of I1.

```text
// illustrative signatures (not a committed surface). τ is a type var.

// ---- the explicit error this layer PRESENTS (never replaces) — built from kernel/checker errors ----
type ReasonedError = {
  class:   ClassName,            // registry-resolved (X1); the refusal's identity
  message: String,               // always shown, even at Minimal (I2)
  site:    Span,                 // source span / breadcrumb — the TRACE: where the refusal happened
  reason:  Option<String>,       // Medium-tier reason detail (the actionable DEBUG INFO)
  context: Map<String, String>,  // candidate Detailed-tier fields, allowlist-filtered at projection (X2)
}

// ---- the content-addressed diagnostic (one truth; human + JSON are projections of it) ----
type Level = Minimal | Medium | Detailed             // a verbosity knob over ONE truth, never a gate (I2)
type Diagnostic = {
  class, message, site, reason,
  level:   Level,                // set by policy; default Minimal
  tags:    Set<String>,          // free-form (v0; DN04-Q2)
  route:   Option<Route>,        // WHERE the presentation goes — never WHETHER the error propagates (I1)
  context: Map<String, String>,  // already allowlist-filtered (X2)
  policy:  Option<PolicyRef>,    // the content hash of the shaping policy (§4.4)
}
type Presentation = { diagnostic: Diagnostic, error: ReasonedError }   // error returned UNCHANGED (I1)

// ---- the never-silent renderer (the crux) ----
present  : ReasonedError, Option<Policy> -> Presentation
//   pure function of an already-emitted error; the error surfaces unchanged in `.error` (I1).
content_id : Diagnostic -> ContentHash               // BLAKE3 over canonical fields (ADR-003)
to_human : Diagnostic -> String                      // graded by level; carries the content id (I3)
to_json  : Diagnostic -> String                      // lossless machine record + embedded id (G11/I3)
from_json: String -> Result<Diagnostic, ParseErr>    // round-trips to the same content id (I3); explicit Err

// ---- the error-class registry (looked up, NEVER eval-ed — X1) ----
resolve  : Registry, String -> Result<ClassName, UnknownClass>   // unknown class = explicit error
register : Registry, String -> Registry                          // extension is still known-set membership

// ---- the reified presentation/routing policy (RFC-0005 pattern; presentation/routing ONLY — I4) ----
type Rule   = { message?, tags?, level?, route? }
on       : Policy, Registry, class: String, Rule -> Result<Policy, UnknownClass>   // class via registry (X1)
policy_ref : Policy -> PolicyRef                                  // content-addressed (ADR-006)
from_file: Registry, PolicyFile -> Result<Policy, UnknownClass>  // re-validates classes; whole-file reject

// ---- routes → RFC-0008 sinks, with HONEST delivery guarantees (RT5/VR-5) ----
type Route    = Stream | Audit | Log | Null | Mesh               // the CLOSED v0 set
type Delivery = Synchronous | Durable | BestEffort | Discarded | Probabilistic{ bound: Bound }
resolve_route : String -> Result<Route, UnknownRoute>            // checked; out-of-set = explicit error
sink     : Diagnostic -> Option<Result<SinkBinding, UnknownRoute>>   // dispatch — OUTSIDE present (I1)
guarantee : Delivery -> Option<GuaranteeStrength>                // ≤ Declared in v0; None for Null (honest)

// ---- the representation-crossing audit view (location-independent — I5) ----
type Crossing = { site, from: Option<Repr>, to: Repr, honesty: Option<GuaranteeStrength>, policy: PolicyRef }
audit_of : Program -> AuditView                                  // every swap, wherever it sits (I5)
//   honesty is READ OFF each certificate, never upgraded (VR-5); underivable => None (unknown ≠ Exact)
```

> **Note (design choice, FLAGGED §7-Q2):** `Span` is sketched as the source-span/trace carrier. Whether the
> trace is a structured source span (line/col/file), the kernel's content-addressed breadcrumb, or the
> existing free-form `site` string is the kernel/surface layer's call (RFC-0006/RFC-0011); the Rust reference
> uses a `String` site. `diag` commits to *carrying a trace*, not to its concrete shape.

## 4. Guarantee matrix (the load-bearing deliverable — [RFC-0016 §4.5](../../rfcs/RFC-0016-Core-Library-and-Standard-Library.md))

Rows = exported ops. Encoded as a checked table (the RFC-0003 §4 template); asserted in tests once code
lands — never prose only. **Every op here is `Exact`**: `diag` carries *no* accuracy/precision/probability
semantics of its own — it is structured presentation over an explicit truth (the `len`-style case, RFC-0016
C2). The one place a lattice tag appears is as **data the module reports, never as its own op's guarantee**:
`guarantee` *surfaces* a route's honest delivery strength (RT5), and `audit_of` *reads off* each crossing's
honesty bound — both **report** a tag without ever upgrading it (VR-5). The "Fallibility" column is each op's
explicit error set; the never-silent property (I1) is that **no row's failure or sink choice can suppress the
presented error** — `present` always returns it.

| Op | Guarantee tag | Fallibility (explicit error set) | Declared effects | EXPLAIN-able? |
|---|---|---|---|---|
| `present` | `Exact` | total — returns the error **unchanged** alongside the diagnostic (I1); cannot fail in a way that drops it | none (pure) | yes (the diagnostic *is* the EXPLAIN record) |
| `content_id` | `Exact` | total | none | n/a |
| `to_human` | `Exact` | total | none | yes (renders the diagnostic; level-graded) |
| `to_json` | `Exact` | total | none | yes (the machine projection) |
| `from_json` | `Exact` | `Err(ParseErr)` on malformed input — explicit, never a partial/sentinel record (C1) | none | n/a |
| `resolve` (class) | `Exact` | `Err(UnknownClass)` — class not in registry; **no `eval`** (X1) | none | yes (the unknown-class refusal record) |
| `register` (class) | `Exact` | total — known-set insert; extension is membership, never `eval` (X1) | none | n/a |
| `on` (add policy rule) | `Exact` | `Err(UnknownClass)` — rule names an unregistered class (X1) | none | yes (the resulting policy is content-addressed) |
| `policy_ref` | `Exact` | total | none | n/a (it *is* the EXPLAIN handle) |
| `from_file` (policy) | `Exact` | `Err(UnknownClass)` — whole-file reject on first unknown class (X1); never partial | none | yes |
| `resolve_route` | `Exact` | `Err(UnknownRoute)` — not in the closed v0 set; never a silent misroute (X1-for-routes) | none | yes (names the closed set) |
| `sink` (dispatch) | `Exact` | `Some(Err(UnknownRoute))` for a bad route; `None` for no route — **outside `present`**, so it never gates propagation (I1) | `io` (the actual sink transport, RFC-0008; bounded, declared at the call) | yes (the `SinkBinding` + its honest `Delivery`) |
| `guarantee` (of a `Delivery`) | `Exact` | total — **reports** the sink's honest strength (`None` for `Null`; ≤ `Declared` in v0); never upgrades it (RT5/VR-5) | none | yes (the delivery guarantee on the lattice) |
| `audit_of` | `Exact` | total — a crossing with no statically-derivable certificate reports `honesty: None` (**unknown ≠ `Exact`**), never a fabricated bound (VR-5) | none | yes (the read-only audit projection — I5) |

**Tag justification (VR-5 — downgrade rather than overclaim):**
- **All `diag` ops are `Exact`** because the module has **no accuracy semantics of its own**: a diagnostic is
  a faithful, content-addressed *re-presentation* of an explicit error (RFC-0016 C2, the `len`-style case).
  `present` does not compute an approximate result; it pairs a truth with a view of it. The honesty work
  `diag` does is *structural* (I1 — never suppress) and *reporting* (RT5/VR-5 — surface a tag honestly), not
  *bounding*.
- **The lattice tags that appear are reported data, not op guarantees.** `guarantee` returns the route's
  `Delivery` strength — `Declared` for `stream`/`audit`/`log`, a `Declared` `ProbabilityBound` δ for `mesh`,
  and **`None` for `null`** (it does not deliver, and says so) — read off the binding, **never upgraded**
  beyond what is honest for v0 (RT5; a checked no-loss/convergence basis is required to exceed `Declared`,
  VR-5). `audit_of` reports each crossing's honesty bound *as the certificate recorded it* and reports
  `None` (unknown) where it cannot be derived — never a silent `Exact` (VR-5, RFC-0013 §4.6 I5).
- **No row can suppress the presented error (I1/G2).** `present` is total and structurally returns the error;
  every fallible op (`from_json`, `resolve`, `on`, `from_file`, `resolve_route`, `sink`) fails *explicitly*,
  and route/sink resolution is **dispatched outside `present`** so even a `null` route or an `UnknownRoute`
  leaves the underlying error already surfaced and propagating.

## 5. §4.1 contract conformance (C1–C6)

- **C1 — never-silent (G2 / RFC-0013 I1):** *the crux.* `present` is a pure function of an already-emitted
  error and returns it **unchanged** in `Presentation.error` — there is no level, tag, route, message, or
  policy that makes the error not surface (I1/I2/I4). Sink/route resolution lives **outside** `present`
  (`sink` is a separate op), so a discarding `null` sink, a failed `mesh` delivery, or an `UnknownRoute`
  **cannot gate propagation** — the error has already surfaced. Every fallible op returns an explicit
  `Result`/`Option` (`UnknownClass`, `UnknownRoute`, `ParseErr`), never a sentinel or a partial record. This
  is exactly what makes `diag` the **legibility substrate** other modules rely on: a refusal anywhere records
  a structured diagnostic with a trace (`site`/span) and actionable reason, robust and legible, never a
  silent swallow (the maintainer's failure-semantics directive; RFC-0013 §4.5 X3).
- **C2 — honest per-op tag (VR-5):** every op is `Exact` (§4) — `diag` carries no accuracy semantics. Where a
  lattice tag appears it is **reported, never claimed**: `guarantee` surfaces a sink's honest delivery
  strength (≤ `Declared` in v0, `None` for `null`), and `audit_of` reports each crossing's bound as recorded
  — both downgrade-honest, never upgraded without a checked basis (RT5/VR-5).
- **C3 — no black boxes / EXPLAIN (SC-3/G11):** a diagnostic **is** an EXPLAIN record — content-addressed,
  inspectable, dual-projected (human + JSON, G11). Every diagnostic a policy shaped records that policy's
  `PolicyRef` (its content hash), so *"which policy shaped this, and what does it do?"* is always answerable
  (RFC-0013 §4.4). The route's sink binding and its honest delivery guarantee are inspectable (`sink`); the
  audit view is a read-only, EXPLAIN-able projection of every representation crossing (I5). No opaque
  heuristic sets a user-visible outcome.
- **C4 — content-addressed, value-semantic (ADR-003 / RFC-0001):** the `Diagnostic` is a content-addressed
  value (deterministic BLAKE3 over canonical fields, with domain separation); its human and JSON projections
  share that `content_id` (I3). A policy is likewise content-addressed (`PolicyRef`). All ops are pure
  functions of their inputs (the sole effect is the explicit, declared `sink` transport, C6). Metadata (the
  `route`, the `PolicyRef`, the `level`) rides the diagnostic but is **not** identity — two diagnostics equal
  on canonical content are the same diagnostic (ADR-003).
- **C5 — above the small kernel (KC-3):** `diag` adds **no** trusted code and gives the kernel **no** logging
  dependency (RFC-0013 §4.8) — it is a pure presentation/projection consumer of the explicit errors the
  kernel/checker/linter already emit. No `wild`/FFI is asserted at the `diag` layer; the actual sink
  *transports* are RFC-0008's (consumed through the honest `Delivery` contract), not new kernel machinery
  (the transport floor is FLAGGED §7-Q3).
- **C6 — declared, bounded effects (RFC-0014):** record construction, projection, registry, and policy ops
  are **pure** (`effects: none`). The single effect surface is **`sink`** — actually emitting a presentation
  to an RFC-0008 observability sink is `io`, **declared** on that op and **bounded** by the sink's contract
  (the `stream` buffer has explicit backpressure; `null` does no IO; `mesh` is a declared-probabilistic
  transport). Crucially, this effect is **downstream of propagation** (I1): the error has already surfaced
  before any sink IO is attempted, so a sink effect can never substitute for the error.

## 6. Grounding

- The module **is** the library/self-hosted form of **RFC-0013** (Accepted — Enacted 2026-06-16): the
  governing invariant **I1** (a diagnostic is additive over a still-propagating error — §4.1), the graded
  **levels** as verbosity-not-gate (**I2**, §4.2), the **dual human/JSON round-trip** projection of one
  content-addressed truth (**I3**, §4.3; G11), the reified presentation/routing-**only** policy (**I4**,
  §4.4; RFC-0005 pattern, ADR-006), the **exclusions** X1 (registry, no `eval`) / X2 (allowlisted detailed
  tier) / X3 (no `logger.catch` swallowing) (§4.5), and the location-independent **audit view** (**I5**,
  §4.6).
- The honest **sink guarantees** are **RFC-0013 §8** (resolved M-354) bound to **RFC-0008 §4.8** observability
  sinks — the closed v0 route set (`stream`/`audit`/`log`/`null`/`mesh`), each with a `Delivery` tagged on
  the lattice (**RT5**), the `null` sink honestly reporting *not delivered*, the `mesh` sink carrying a
  declared `ProbabilityBound` δ, none upgraded without a checked basis (**VR-5**).
- The landed surface `diag` re-authors: **`crates/mycelium-lsp/src/diagnostics`** (M-345, Enacted) —
  `registry.rs` (X1), `record.rs` (`DiagnosticRecord`, dual projection, graded `Level`, `present` — I1),
  `policy.rs` (`DiagnosticPolicy`/`PolicyRef`, `PolicyFile` — §4.7), `sink.rs` (closed routes + honest
  `Delivery` — RT5/M-354), `audit.rs` (the crossing view — I5/VR-5); types from `mycelium-core`
  (`ContentHash`, `Bound`/`BoundBasis`, `GuaranteeStrength`).
- The per-op contract C1–C6, the ring/tier placement, and the guarantee-matrix obligation: **RFC-0016**
  §4.1 / §4.2 / §4.3 (the `diag` row: "structured diagnostics — additive presentation over explicit errors,
  never substitutive, I1") / §4.5; the Rust-first → self-hosted migration with an NFR-7-style differential:
  **RFC-0016 §4.6** + the **M-502** readiness gate.
- The honesty lattice + value model + content-addressing: **RFC-0001** (`Exact ⊐ Proven ⊐ Empirical ⊐
  Declared` §4.3, `Value`/`Repr`/`Meta`, content-addressing §4.6); **G2** (never-silent), **G11** (dual
  projection), **VR-5** (honest tags), **KC-3** (small kernel), **ADR-003** (metadata is not identity).

## 7. Open questions (FLAGGED — resolve before ratification)

- **(Q1) The `diag` ↔ `fmt` rendering boundary.** `diag.to_human` renders a diagnostic; `std.fmt` (M-533)
  owns the canonical human/machine projection (G11) and its row names exactly that. The two must not
  re-implement display: `diag` owns the diagnostic's *structure + its own dual projection*, but the *generic
  rendering primitives* (string building, colorization, the JSON canonicalizer) should be `fmt`'s, delegated
  to. — *Disposition:* co-design the seam with `fmt` (M-533) so `diag` delegates generic rendering and keeps
  only the diagnostic-specific shaping; neither double-owns "dual projection". Ties to RFC-0016 §8-Q3
  (ergonomics-vs-contract) and the README §5 fmt/serialize JSON-projection seam.
- **(Q2) The trace/source-span carrier shape.** `present`'s legibility rests on a real **trace** — *where*
  the refusal happened. The Rust reference uses a free-form `site: String`; a self-hosted form should likely
  carry a structured source span (file/line/col) or the kernel's content-addressed breadcrumb. The concrete
  carrier is a kernel/surface-layer call (RFC-0006 surface, RFC-0011 Core IR), not `diag`'s to fabricate. —
  *Disposition:* `diag` commits to *carrying a trace*; the concrete `Span` shape is co-designed with the
  surface/IR layer. Ties to RFC-0016 §8-Q3.
- **(Q3) The sink-transport effect floor — `wild`/FFI?** §C5 asserts `diag` adds no `wild`. The actual sink
  *transports* (`stream`/`audit`/`log`/`mesh`) are RFC-0008's; whether a transport bottoms out in a platform
  IO/network call via `wild` (ADR-014) is **RFC-0008 / `runtime`'s** concern, and if so that `wild` block is
  inventoried *there* (LR-9), not at `diag`. `diag` consumes the honest `Delivery` contract regardless. —
  *Disposition:* FLAGGED; the transport floor is `runtime`/RFC-0008's (M-521, Phase-7-gated), ties RFC-0016
  §8-Q4/§8-Q6. `diag`'s C5 "no `wild`" claim holds for the presentation/record layer.
- **(Q4) The self-hosting migration differential's bar.** RFC-0016 §4.6 requires the Mycelium-lang `diag` to
  pass an NFR-7-style differential against the Rust reference before it graduates. What must match
  bit-for-bit — observable diagnostics only, or content-addressed `id`s + the human/JSON projections + the
  audit view exactly — is the §8-Q5 bar, and **M-502**'s verdict is currently *not-yet* (the surface is not
  self-hosting-ready). — *Disposition:* the Rust-first `diag` proceeds now (RFC-0016 §4.6 Phase 5a); the
  self-hosted form waits on M-502 and the §8-Q5 differential bar. Ties to `self-hosting-readiness.md`
  (M-502) Q-a/Q-b, which names `diag` as a candidate first self-hosting proof.
- **(Q5) First-class typed tags.** Tags are a free-form string set in v0 (RFC-0013 §4.4 DN04-Q2). Whether
  they become a typed, queryable, content-addressed field on the diagnostic (more useful, more honest) is a
  recorded RFC-0013 §9 future, not v0. — *Disposition:* keep free-form strings for v0 parity with the Rust
  reference; typed tags are a post-v0 RFC-0013 evolution, not this spec's to invent.

## Meta — changelog

- **2026-06-17 — Draft (needs-design).** Stands up the `std.diag` (M-510, #151; Ring 1, Tier A) module spec
  under RFC-0016 (Draft) as the library/self-hosted form of **RFC-0013** (Accepted — Enacted): the
  **structured diagnostic record + trace substrate** every other module's failure legibility rests on. Fixes
  the **scope + boundary** (the content-addressed `Diagnostic` + dual human/JSON projection + reified
  presentation/routing policy + closed route→sink vocabulary + audit view, bounded against `recover`'s
  *recovery* (M-520, RFC-0014 I4), `error`'s *propagation glue* (M-527), `fmt`'s *generic rendering* (M-533),
  and `core`'s *error representation* (M-515, RFC-0001)); the **exported-op surface** sketch (mirroring the
  landed `crates/mycelium-lsp/src/diagnostics` Rust reference — `present`/`content_id`/`to_human`/`to_json`/
  `from_json`/registry/`on`/`from_file`/`resolve_route`/`sink`/`guarantee`/`audit_of`); and — the
  load-bearing deliverable — the **guarantee matrix** (14 rows, **all `Exact`**: `diag` has no accuracy
  semantics of its own; lattice tags appear only as *reported* sink/crossing data, never upgraded — RT5/VR-5),
  with the structural property that **no row can suppress the presented error** (RFC-0013 I1 / RFC-0016 C1 /
  G2: presentation never gates propagation; sink/route resolution is dispatched *outside* `present`). States
  the **§4.1 conformance** (C1 the crux: `present` returns the error unchanged, `null`/`mesh`/`UnknownRoute`
  cannot gate it; C6 the lone `io` effect is `sink`, declared, bounded, downstream of propagation), the
  **grounding** (RFC-0013 I1–I5 + X1–X3, RFC-0008/RT5 sinks, the M-345 Rust reference, RFC-0016 §4.1–§4.6,
  RFC-0001, G2/G11/VR-5/KC-3), and **five FLAGGED questions** (the `diag`↔`fmt` rendering boundary; the
  trace/source-span carrier shape; the sink-transport `wild` floor owned by RFC-0008/`runtime`; the
  self-hosting migration differential bar tied to M-502/§8-Q5; first-class typed tags) — each tied to its
  owning task/RFC, none invented. No code; no kernel change (KC-3 — the kernel gains no logging dependency).
  Append-only.
- **2026-06-18 — Implemented (Rust-first), pending ratification.** Landed as `mycelium-std-diag` (M-510, #151; Batch P5 Tier-A completion, octopus-merged) over a newly-extracted **`mycelium-diag` kernel crate** homing the canonical RFC-0013 record types (`Diag`/`Severity`/`Locus`/`Trace`/`Code`) — a maintainer-resolved FLAG (scaffold decision #1): a small, deliberate trusted-base growth so the record has one owner below the std layer; the std crate re-exports it (KC-3). Delivers the dual human/JSON projection (G11, round-trip-checked), content-addressed presentation-invariant identity (ADR-003), and the §4.5 guarantee matrix as checked data (all rows `Exact`; `present` returns the error **unchanged** — the I1 structural proof). FLAGGED for fast-follow: reconcile `std.testing`'s placeholder `FailRecord` to delegate to `Diag` (the §7-Q2 / testing §Q2 seam); the `std.fmt` canonical-rendering delegation (§7-Q1). No kernel change (KC-3 — the kernel gains no logging dependency). Append-only.

- **2026-06-20 — Accepted (maintainer ratification, DN-07).** The maintainer ratified this Rust-first spec: the §4.5 guarantee matrix is asserted in tests, never-silent fallibility and honest per-op tags hold, and the open §7/§8 questions are design/scope calls, not contract violations. No guarantee tag was upgraded without a checked basis (VR-5). Status moves *Implemented (Rust-first) — pending ratification → Accepted*. Append-only; no kernel change (KC-3).
