# RFC-0022 — Web-Tooling Phylum (HTTP client/server/routing/JSON)

| Field | Value |
|---|---|
| **RFC** | 0022 |
| **Status** | **Accepted** (2026-06-21 — maintainer ratification: design agreed; **Enacted** still gated on the `mycelium-web` build). *Ratification trail:* **Draft** (2026-06-21 — Phase-1 design pass. Per the two-phase research discipline, a **follow-up deep-research pass gates ratification** (tracked as RP-10); the §10 Honest-Uncertainty Register enumerates exactly what that pass must verify. No status moves past Draft until §10.2 is discharged.) · **RP-10 research gate DISCHARGED (2026-06-21, `dfr` session)** — §10.2 verified against primary specs (RFC 9110/9112, RFC 8259, WHATWG-URL) + landed source by four fractured Opus sub-reasoners (`research/12-web-phylum-RECORD.md` §8); all four obligations (HTTP never-silent · JSON codec-reuse · server-determinism · routing/EXPLAIN) **design-sound, no falsification**. Residuals named, not closed — **deferred-to-build** (the ≥100-vector corpus, the RT2 differential, per-dispatch `RouteMatch`) + **scoped-future** (HTTP/2-3, TLS, the cross-peer smuggling model, wall-clock time, async-runtime choice, RT3). **Pending maintainer ratification** to Accepted (an agent stages the discharge; the maintainer ratifies — RFC-0018 precedent). *Honesty (VR-5):* Empirical/Declared, never `Proven`; empirical confirmation lands at the `dfb` build — the basis for a future Accepted→Enacted. · **RATIFIED → Accepted (2026-06-21, maintainer).** Decisions: **IDNA/UTS-46** — ratify the *policy* (pin a Unicode version, **nontransitional**, fail-not-best-effort; `dfb` pins the exact version current at build + records it — U7); **`web.server`** runs on the **Mycelium colony/hypha runtime** (no external executor; realization `mycelium-mlir::runtime`, tracking RFC-0023 R23-Q1); **v1 non-goals confirmed** (HTTP/2-3, TLS, the cross-peer smuggling threat model — §6, named not dropped). **Enacted** gated on the `mycelium-web` build (+ E7-1/E7-2 for the `.myc` surface). |
| **Type** | Foundational / normative (once Accepted) — a **standard-library phylum** above the kernel (KC-3); no kernel change |
| **Date** | June 21, 2026 |
| **Tracks** | A new web-tooling phylum (working name `web`). Decomposes into per-nodule tasks once ratified; v1 lands Rust-first as a `mycelium-web` crate (mirroring the RFC-0016 §4.6 Rust-first order). |
| **Depends on** | RFC-0016 (the standard-library **scope + per-op contract** C1–C6, ring layering, the guarantee-matrix obligation, the Rust-first→Mycelium-lang migration — the model this RFC mirrors); RFC-0001 (the value model `Value`/`Repr`/`Meta`, the guarantee lattice, §4.8 (de)serialization); RFC-0008 (the runtime/concurrency model — the server as a `colony` of request-handling `hyphae`; RT1–RT7; §4.7 budgets/cancellation/supervision); RFC-0014 §4.5 (declared, bounded effects); RFC-0013 §4.1 (the diagnostic record carried by parse errors); ADR-003 (content-addressed identity; serialization is a projection); ADR-007 (Rust-first toolchain); ADR-013 (`spore` = the deployable unit a server germinates from); ADR-020 (the `runtime`/`colony` phylum + `std.runtime` facade — the concurrency-surface template); the std phyla `std.io` (M-514), `std.text` (M-524), `std.error`/`std.core` (M-527/M-515), `std.collections` (M-511); G2 (never-silent), G11 (dual projection), VR-5 (honest tags), KC-3 (small kernel) |
| **Gated by** | **E7-1** (L1 Stage-1 language completeness — **M-657** generics, **M-659** traits; the typed `Json<T>` handler surface + in-`.myc` authoring); **E7-2** (RFC-0008 runtime vocabulary — **M-665** lexer reservation → **M-666** `hypha`+`colony` L1 constructs; the `.myc` `colony { hypha … }` server surface). DN-14 (the self-hosting gate) records generics/traits/effects/`wild` as **gate-fails**, which is why v1 is **Rust-first**. |
| **Research** | `research/12-web-phylum-RECORD.md` (T12.1.x–T12.4.x; the §6 Honest-Uncertainty Register). Findings are **Empirical/Declared** (design-informing prior-art + corpus-grounding), **never `Proven`**. |

---

## 1. Summary

This RFC designs a **web-tooling phylum** (working name `web`): an HTTP/1.1 **client**, **server**,
**routing** surface, and a **JSON ↔ `Value`** surface — held to the same honesty contract as the
rest of the standard library (RFC-0016 §4.1 C1–C6), and **never-silent throughout**. Every parse is
a located `Result` (a malformed request-line, header, status, or URL is an explicit error naming
*where* it failed — never a sentinel, clamp, or partial value), every selection (route match,
content negotiation) is reified and `EXPLAIN`-able (no black-box routing — C3), and every accuracy
claim is tagged on the lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` at its honestly-supportable
strength (VR-5).

The phylum is **one `web` phylum** (DN-06; content-addressed, library-scale) whose
independently-importable `nodule`s are `web.http` (the request/response value model + never-silent
parsing), `web.json` (a thin convenience over `std.io`'s one canonical JSON — **no new codec**),
`web.route` (the inspectable route table + dispatch), `web.server` (the server as a `colony` of
request-handling `hyphae`), and `web.client` (`get`/`post`/`request`). It is **Ring 2** — a library
**above** the kernel that adds **zero trusted code** (KC-3): it builds on the landed std phyla
(`std.io`, `std.text`, `std.error`, `std.collections`) and the landed Rust runtime
(`mycelium-mlir::runtime` / the ADR-020 `runtime` phylum).

**Self-hosting is blocked** (DN-14 gate-fails: generics, traits, effect annotations, `wild`/FFI are
all absent at the L1 surface), so **v1 is Rust-first** — a Rust crate `mycelium-web` exposing the
phylum surface, exactly as the 25 `mycelium-std-*` crates landed Rust-first (RFC-0016 §4.6,
Enacted). The `.myc` server surface (`colony { hypha … }`) and the typed `Json<T>` handler surface
are **gated** on epics **E7-2** (M-665 → M-666) and **E7-1** (M-657 generics, M-659 traits)
respectively, and this RFC names those dependencies explicitly rather than sketching unavailable
surface as if it existed.

This RFC is **scope + design**, not implementation; per the two-phase discipline it stays **Draft**
and the §10 Honest-Uncertainty Register gates ratification.

## 2. Motivation

- **The standard library should reach the web honestly.** RFC-0016 (Enacted) gave Mycelium a
  honest standard library; the natural next surface a usable language needs — talking HTTP and
  JSON — is missing. The danger in any web library is precisely the silent default the honesty rule
  forbids: a parser that returns a best-effort partial request on malformed bytes, a JSON encoder
  that emits `null` for a value it cannot represent, a router that silently picks a wrong handler, a
  server that silently drops a failed request. Mycelium's thesis is that **every** such outcome is
  explicit and tagged (G2/VR-5). A web phylum that violated that would make the substrate's promise
  a lie at a layer users touch constantly. So the **never-silent discipline (§4) is the
  load-bearing part of this RFC**, not an afterthought.

- **The runtime model already has a place for a server.** RFC-0008 defined concurrency as a
  `colony` of structurally-scoped `hyphae` over immutable values, with structured lifetimes (RT7),
  explicit partial failure (RT4), and reified nondeterminism (RT3). An HTTP server is *exactly* a
  `colony` of one request-handling `hypha` per connection — a faithful instance of that model, not
  a new mechanism (research T12.2.1). ADR-013 made a server's deployable form a `spore` that
  germinates into a running hypha (RFC-0008 §4.4). The web phylum is where this comes together.

- **Reuse, don't re-invent (DRY/KC-3).** `std.io` already owns the **one canonical JSON
  projection** over the value model (RFC-0001 §4.8; `std.fmt` already delegates to it — M-372). A
  second JSON codec in the web phylum would violate DRY and re-introduce a trusted-code surface
  `std.io` deliberately avoids. The web JSON surface is therefore a **thin convenience layer** over
  `std.io`'s codec (research T12.3.1). Likewise `Headers` is a multi-map over `std.collections`, not
  a fresh structure (T12.1.2).

- **Honesty must survive the language gaps.** The L1 surface language cannot yet author this phylum
  (DN-14: generics, traits, effects, `wild` all gate-fail). The honest response is **Rust-first**
  (ADR-007; the RFC-0016 §4.6 precedent), with the gated `.myc` surfaces (typed handlers via E7-1;
  the `colony`/`hypha` server surface via E7-2) named as explicit dependencies — never sketched as
  available (research T12.3.6/T12.4.6).

## 3. Guide-level explanation

A program that talks to the web in Mycelium imports the `web` phylum's nodules:

- **`web.http`** is the value model: `Request`, `Response`, `Headers`, `Status`, `Method`, `Url`,
  `Body` — all **immutable values** with content-addressed identity (ADR-003). Every "mutator"
  (`with_header`, `with_status`) returns a *new* value, as every `std.collections` structure does.
  Turning raw bytes into one of these is the never-silent crux: `parse_request(bytes)` is a
  `Result<Request, HttpParseError>`, and a malformed request is an explicit error naming the **byte
  offset / field** where it failed — never a half-built `Request` (research T12.1.1, T12.1.7).

- **`web.json`** moves a request/response body to and from a `Value`, **delegating** to `std.io`'s
  one canonical JSON (`to_json`/`from_json`). `decode_body` of a malformed body is an explicit
  located error (`Empirical` round-trip, inherited from `std.io`); `encode_body` of a value with a
  non-finite `f64` is **refused** (explicit `Err`), never a silent JSON `null` (research
  T12.3.2–T12.3.5).

- **`web.route`** is an **inspectable** route table mapping `(Method, path-pattern)` to a handler.
  Dispatch is a `Result` — a no-route is an explicit 404, a wrong-method an explicit 405 — and
  *which* route matched and *why* is reifiable: opaque-heuristic routing would violate the
  no-black-boxes rule (C3) (research T12.1.11).

- **`web.server`** runs a handler as a **`colony` of request-handling `hyphae`** (RFC-0008): the
  accept loop spawns one request `hypha` per connection, all under the server `colony`'s structured
  scope. A handler receives an **immutable `Request` value** and returns a **`Response` value** (no
  shared mutable session state — RT1); a failed handler resolves to an explicit `TaskOutcome` (→ an
  HTTP error response), never a silently-dropped request (RT4); graceful shutdown is the colony
  scope closing after in-flight requests drain (RT7) (research T12.2.1–T12.2.7).

- **`web.client`** is `get`/`post`/`request` — every call a `Result` (a connection failure or
  timeout is explicit, never a sentinel response) declaring an `io` effect (research T12.1.10).

**The honest reality (§7).** None of this is `.myc`-authorable yet. v1 is a **Rust crate
`mycelium-web`** exposing the phylum surface — exactly how the 25 `mycelium-std-*` crates landed.
The `.myc` `colony { hypha … }` server surface waits on **E7-2** (M-666); typed `Json<T>` handlers
wait on **E7-1** (M-657 generics + M-659 traits). The Rust runtime substrate
(`mycelium-mlir::runtime` `Scope`/`Colony`/`Task`, M-357/M-356/M-521) is already landed, so the
server is Rust-first-buildable today.

## 4. Reference-level design (normative once Accepted)

### 4.1 The phylum surface — five nodules over one `phylum`

`web` is one content-addressed `phylum` (DN-06) whose independently-importable `nodule`s mirror the
RFC-0016 `std` structure (one phylum, one contract, one matrix format, one EXPLAIN style — RFC-0016
§3/§4.2/§6):

| Nodule | What it is | Depends on |
|---|---|---|
| `web.http` | the core request/response/header/status/method/url **value model** + never-silent parsing | `std.core`/`std.error`, `std.text`, `std.collections` |
| `web.json` | a **thin convenience** over `std.io`'s one canonical JSON (delegates; no new codec) | `std.io`, `web.http` |
| `web.route` | the **inspectable** route table + dispatch (EXPLAIN-able — C3) | `web.http`, `std.collections` |
| `web.client` | `get`/`post`/`request` — fallible, `io`-effecting | `web.http`, `web.json`, the `std.io`/effect surface |
| `web.server` | the server as a **`colony` of request-handling `hyphae`** | `web.http`, `web.route`, the runtime/`colony` surface (RFC-0008 R1 / `std.runtime`) |

The dependency graph is **acyclic, bottom-up** (research T12.4.2): `web.http` is the base value
model; `web.json` sits over it + `std.io`; `web.route` over `web.http`; `web.client` and
`web.server` at the top. The full surface sketch (illustrative signatures, house style per `io.md`/
`fs.md` §3 — **not a committed grammar**) is in `research/12-web-phylum-RECORD.md` §2/§5; the
load-bearing shapes:

```
// nodule: web.http   — the value model (pure, value-semantic)
type Method  = Get | Head | Post | Put | Delete | Connect | Options | Trace | Patch | Extension(Token)
type Status  // validated 100..=599 (never a bare integer)
type Headers // value-semantic multi-map over collections::Map<HeaderName, Seq<HeaderValue>>
type Url     // parsed WHATWG components
type Body    // in-memory Bytes | streaming Source (LR-8, threads std.io)
type Request  // { method, url, headers, body, version }
type Response // { status, headers, body, version }

Status::from_u16(n: u16)        -> Result<Status, HttpParseError>   // out-of-range -> Err, never clamp
Method::parse(t: &Text)         -> Result<Method,  HttpParseError>  // Err names the bad token
Url::parse(t: &Text)            -> Result<Url,     UrlParseError>   // WHATWG; Err @ offending component
parse_request(raw: &Bytes)      -> Result<Request, HttpParseError>  // RFC 9112 wire; Err @ byte offset
serialize_request(r: &Request)  -> Bytes                           // total faithful projection

// nodule: web.json   — thin convenience over std.io's one canonical JSON (delegates; no new codec)
encode_body(v: &Value)          -> Result<Body, JsonError>         // refuses non-finite f64 (never silent null)
decode_body(b: &Body)           -> Result<Value, JsonError>        // malformed -> Err(@locus), never partial

// nodule: web.route  — REIFIED, inspectable dispatch (opaque routing violates C3)
type Handler = fn(Request) -> Result<Response, HttpError>          // conceptually: !{ io }
match_route(t: &RouteTable, m: &Method, p: &Path)
                                -> Result<(Handler, PathParams), RouteError>   // EXPLAIN-able
enum RouteError { NotFound, MethodNotAllowed { allowed: Seq<Method> } }        // -> 404 / 405

// nodule: web.client — fallible + io-effecting (conceptually: !{ io })
get(u: Url)                     -> Result<Response, HttpError>
post(u: Url, b: Body)           -> Result<Response, HttpError>
request(r: Request)             -> Result<Response, HttpError>

// nodule: web.server — bind + serve (the request->handler->response flow; conceptually: !{ io })
serve(table: RouteTable, listener: Substrate{Socket}) -> Result<(), HttpError>
```

> **Note on effect annotations.** The `io` effect on client/server ops is `!{ io }` (RFC-0014 §4.5).
> The surface effect-annotation *syntax* is itself gate-failed (DN-14 §3 row 8 / M-660), so the
> `.myc` examples below prose-comment the effect rather than writing it in the signature (research
> T12.4.8).

### 4.2 Never-silent parsing (C1/G2) — the crux

Every HTTP parse is `parse → Result<T, ParseError>` carrying a **WHERE locus** (byte offset /
field), modeled exactly on `std.io`'s `SerError` and `std.text`'s `ParseErr` (research T12.1.7). A
malformed request-line, header block, status-line, or URL is an **explicit, located** error, and
the error carries an **RFC-0013 diagnostic record** so the failure is legible (where + why — never
a silent partial value). The explicit error sets (illustrative; full enums in the RECORD §2):

```
enum HttpParseError {
  RequestLineMalformed { at, why },  StatusLineMalformed { at, why },
  BadVersion { at, found },          BadMethodToken { at, found },
  StatusOutOfRange { at, code },     // never a clamp (RFC 9110 §15)
  HeaderMalformed { at, why },       BadHeaderName { at },          // CR/LF/NUL rejected — header-injection guard
  ObsFold { at },                    Truncated { at },
  BadContentLength { at, why },      BadTransferEncoding { at, why },  // request-smuggling vectors closed
}
enum UrlParseError {
  MissingScheme { at },  BadScheme { at, found },  BadHost { at, why },
  BadPort { at, found }, // not 0..=65535, never a clamp
  BadPercentEnc { at },  NonAsciiUnencoded { at },  // needs IDNA / percent-encoding
}
```

**Security-critical refusals are *parse errors*, not lenient acceptance** (research T12.1.9):
`BadHeaderName`/`HeaderMalformed` reject CR/LF/NUL (header injection / response splitting);
`BadContentLength`/`BadTransferEncoding`/`ObsFold` reject the duplicate-`Content-Length`,
conflicting-`Transfer-Encoding`-vs-`Content-Length`, and obs-fold cases that drive HTTP request
smuggling (RFC 9112 §5.2/§6.3/§7). The **full** smuggling/normalization threat model is a
deep-research follow-up (§10.2 U5), but the known vectors are closed by construction.

### 4.3 The JSON surface delegates to `std.io` — no new codec (DRY/KC-3)

`web.json` adds **zero** trusted serialization code; its body helpers **call**
`mycelium_std_io::{to_json, from_json}` — the same delegation `std.fmt` performs (research T12.3.1;
io.md §7-Q1 / M-372). A JSON (de)serialize is a **projection, not a `swap`**: it preserves the
value's `Repr` and content-id and emits **no** certificate (ADR-003; io.md §C4; research T12.3.7).
The tags are **inherited unchanged** from the `std.io` codec — the web layer mints none (VR-5/KC-3):

- **encode** (`encode_body`, Value → JSON bytes): `Exact`-**when-`Ok`** but **fallible** — a
  non-finite `f64` has no JSON form and is **refused** (`Err(OutOfDomain)`), never a silent `null`
  (the exact `std.io` `to_json` rule, asserted in `mycelium-std-io/src/guarantee_matrix.rs`).
- **decode** (`decode_body`, JSON bytes → Value): **`Empirical`** — round-trip fidelity
  (`from_json ∘ to_json ≡ id`) established by a proptest corpus, **no checked theorem → not
  `Proven`** (VR-5; io.md §4-Q2). A malformed body → explicit **located** `Err`, never a
  partially-filled `Value` (C1/G2), carrying the RFC-0013 diagnostic naming where it failed.

The body helpers are **pure over the byte body** (`effects: none`) — the network I/O is the HTTP
layer's declared `io` effect (`web.client`/`web.server`), not the JSON helpers' (research T12.3.5).

### 4.4 The server is a `colony` of request-handling `hyphae` (RFC-0008)

The server is a long-lived **`colony`** (DN-06; the dynamic runtime grouping of cooperating
`hyphae`; RFC-0008 §4.7) whose accept loop spawns one request-handling **`hypha`** per connection.
The RFC-0008 invariants bind it (research T12.2.2–T12.2.7):

- **RT1 (isolation):** a handler receives an **immutable `Request` value** and returns a **`Response`
  value**; there is no shared mutable session state — *as an absence, not a discipline*. Any
  cross-request shared state (cache, connection pool, session store) is **not ambient** — it must be
  an explicit reified mechanism (`fuse`d semilattice state — RT6, an `xloc`-moved value, or an
  external `graft` capability — RT4), never a silently-aliased mutable global (§10.2 U11).
- **RT2 (deterministic default):** a pure `Request → Response` handler is in the deterministic
  fragment, so request-dispatch has a sequential reference semantics the trusted interpreter can
  replay; the honesty target is **concurrent observable ≡ the deterministic reference's** (NFR-7).
  Handlers that read wall-clock time / randomness / external I/O are explicit effects outside the
  pure fragment (§10.2 U12).
- **RT4 (explicit partial failure):** a request `hypha` resolves to exactly one explicit
  `TaskOutcome` — `Done`/`Failed`/`BudgetExhausted`/`Cancelled` — with no silent/dropped variant
  (§4.7 C3); a handler error becomes an explicit `Failed` the colony **must** act on (→ an HTTP
  error response). Never silently dropped or retried.
- **RT7 (structured lifetimes):** every request `hypha` lives inside the server `colony`'s scope,
  which does not exit until each child completes / is cancelled. **Graceful shutdown = the colony
  scope closing**: stop accepting, then drain in-flight requests or have them observe cancellation
  (§4.7 C2). A leaked in-flight request is **not expressible** (RT7).
- **Per-request budgets/timeouts (§4.7 C1):** each request `hypha` carries its own `Budgets` ledger;
  one runaway request exhausting *its* budget cannot exhaust another's, and surfaces as that
  request's explicit `BudgetExhausted` (→ 503/504), never a server-wide stall. Wall-clock timeouts
  depend on RFC-0008 R8-Q3 (v0 budgets use a **logical clock**; §10.2 U12).
- **Supervised accept loop (§4.7 C2/C4):** cooperative cancellation propagates down the colony tree;
  a `reclaim` supervisor restarts a failed child under a **bounded** cascade (total cap + windowed
  max-restart-intensity over a logical clock) — exceeding either escalates, never a restart storm.

**RT3 nondeterministic constructs are out of the deterministic default** and must be FLAGGED with a
reified RFC-0005 policy + mandatory EXPLAIN (research T12.2.11): racing (first-response-wins / hedged
requests), load-balanced upstream selection, and multi-source `select`/`merge` are all RT3; the
multi-source primitive is **deferred / not yet landed** in `mycelium-mlir::runtime` (§10.2 U15). v1
either avoids racing (deterministic upstream choice) or FLAGs it as a pending RT3 construct.

**RATIFIED (2026-06-21).** `web.server`'s concurrency is the **Mycelium runtime**, not an external
executor (tokio/async-std) — one concurrency substrate across the stdlib (KC-3, no trusted external
dependency; uniform RT2 determinism). v1 realization = **`mycelium-mlir::runtime`** (tracking the
RFC-0023 R23-Q1 ratification); migrate to the `std.runtime` facade (ADR-020) when it grows the surface.

### 4.5 Honest per-op guarantee tags + the guarantee-matrix obligation (RFC-0016 §4.5)

The honesty load is carried — as in `std.text`/`std.io` — almost entirely by the **fallibility**
column, not by exotic tags (research T12.1.12/T12.4.4). Each nodule ships a `guarantee_matrix.rs`
(rows = exported ops; columns `{op, guarantee tag, fallibility, error set, effects, EXPLAIN-able}`)
**asserted in tests, never prose-only**, with the VR-5 guard-tests carried verbatim from `std-io`'s
matrix (`no_op_is_proven_without_a_checked_theorem`, `no_op_is_declared`,
`fallibility_and_error_set_are_consistent`, `io_ops_declare_io_effect`,
`serialize_and_to_json_refuse_non_finite_fallibly`). The summary matrix:

| Nodule | Op | Tag | Fallibility | Error set (explicit, located) | Effects | EXPLAIN |
|---|---|---|---|---|---|---|
| `web.http` | `parse_request` | **`Exact`-when-`Ok`** | Fallible | `Err(Malformed{at} \| BadHeader{at} \| BadRequestLine{at} \| BadVersion{at})` | `none` | yes (byte offset) |
| `web.http` | `parse_response` | `Exact`-when-`Ok` | Fallible | `Err(Malformed{at} \| BadStatus{code} \| BadHeader{at})` | `none` | yes |
| `web.http` | `status_from_u16` | `Exact`-when-`Ok` | Fallible | `Err(OutOfRange{code})` — never a clamp | `none` | yes |
| `web.http` | `header_get`/`method`/`path` | `Exact` | Total | — | `none` | n/a |
| `web.http` | `Request::new`/`with_header` | `Exact`-when-`Ok` | Fallible | `Err(InvalidHeaderName \| InvalidHeaderValue)` | `none` | n/a |
| `web.json` | `encode_body` | **`Exact`-when-`Ok`** | Fallible | `Err(OutOfDomain)` — non-finite f64 refused, **never silent `null`** | `none` | n/a (faithful projection) |
| `web.json` | `decode_body` | **`Empirical`** (round-trip; proptest, no theorem — VR-5) | Fallible | `Err(Malformed{at} \| UnknownTag{path} \| OutOfDomain{path} \| BudgetExceeded)` @locus | `none` | yes (RFC-0013 @locus) |
| `web.route` | `match_route` | `Exact`-when-`Ok` | Fallible | `Err(NoRoute)` / explicit 404 — never silent wrong-handler | `none` | **yes (C3-mandatory: which pattern + captures)** |
| `web.route` | `table` | `Exact` | Total | — | `none` | yes (reified table) |
| `web.client` | `get`/`request` (wire) | `Exact`-when-`Ok` | Fallible | `Err(UnexpectedEof{read} \| Refused{why} \| EffectBudget)` | **`io`** | yes (IoError record) |
| `web.client` | `get_json` (transfer+decode) | **`Empirical`** (composes `io`+round-trip) | Fallible | `Err(HttpError \| IoError \| JsonError)` @locus | **`io`** | yes |
| `web.server` | per-request join (`Scope`) | **`Empirical`** (RT2 differential; **not `Proven`**) | Fallible | `Err(ServeError \| TaskPanicked \| EffectBudget)` | **`io`** | yes |
| `web.server` | handler purity contract | **`Declared`** (asserted by caller; type system can't enforce — VR-5) | n/a | — (FLAGGED `Declared`) | — | yes (reified assertion) |
| `web.server` | `serve` (accept loop) | `Exact`-when-`Ok` | Fallible | `Err(BindFailed \| Refused \| EffectBudget)` | **`io`** | yes |

The two non-`Exact` postures both trace to landed precedents, not new claims: the **`Empirical`**
JSON-decode + per-request-determinism tags are inherited from `std.io`'s `from_json` row and
`std.runtime`'s `Scope`/Kahn-determinism rows (both `Empirical`-via-differential, explicitly **not**
`Proven`); the **`Declared`** handler-purity row mirrors `std.runtime`'s `Task` purity-contract row
(a declared contract the type system cannot enforce, never claimed `Proven`). The web phylum
therefore meets the honesty bar **by reuse**, not by minting tags (research T12.4.4; ADR-020 §4;
`crates/mycelium-std-runtime/src/guarantee_matrix.rs`).

### 4.6 §4.1 contract conformance (C1–C6)

- **C1 — never-silent (G2).** Every fallible op returns an explicit **located** `Result` — a
  malformed message → `Err(HttpParseError{at})` not a zeroed `Request`; an unknown method → explicit
  `Err`; a non-finite f64 in a JSON response → **refused**, never silent `null`; a no-route →
  explicit 404; a short socket read → `Err(UnexpectedEof{read})` (research T12.4.7).
- **C2 — honest per-op tag (VR-5).** HTTP/accessor ops are `Exact`-when-`Ok` (no accuracy semantics
  → the floor `Exact`); JSON decode + per-request determinism are `Empirical` (differential, not
  theorem); handler purity is `Declared` (FLAGGED). No tag is `Proven` without a checked basis
  (§4.5; the matrix guard-tests enforce this).
- **C3 — no black boxes / EXPLAIN.** Routing is over a **reified, inspectable** route table and a
  match yields an EXPLAIN record (which pattern matched + captures) — an opaque-heuristic router is
  forbidden. Every parse/IO failure reifies an RFC-0013 diagnostic with its locus. The JSON projection
  selects/approximates nothing (EXPLAIN `n/a`).
- **C4 — content-addressed, value-semantic (ADR-003).** HTTP values are immutable with
  content-addressed identity; every mutator returns a new value. A JSON (de)serialize is a
  **projection, not identity** — it preserves the content-id and emits no certificate (§4.3).
- **C5 — above the small kernel (KC-3).** Every nodule is **Ring 2**, adds **zero trusted code**:
  `web.json` wraps the landed M-104 codec via `std.io`; `web.http` is pure value logic over
  `std.text`/`std.collections`; `web.server` consumes the `wild`-free `std.runtime`/RFC-0008 R1
  executor. The only OS-facility floor (socket syscalls) is `wild`/FFI, confined to the separate
  `std-sys` phylum so pure `web` stays leak-free by construction (LR-9; RFC-0016 §8-Q6; §10.2 U2).
- **C6 — declared, bounded effects (RFC-0014).** Client/server ops **declare** `io` on their
  signatures; per-request work runs under a bounded `Budgets` ledger (§4.4 C1). The pure value-model
  and JSON-projection ops declare `effects: none`.

### 4.7 The Rust-first build plan (ADR-007 + DN-14)

v1 is a **genuine Rust crate `mycelium-web`** (the kernel-impl distinction: `mycelium-*` Rust crates
are real crates; Mycelium-*language* units are phyla/nodules — DN-06), mirroring the 25
`mycelium-std-*` crates that landed Rust-first under RFC-0016 §4.6 (Enacted). Why Rust-first, not
`.myc`-first: **DN-14 is decisive** — of 11 surface-language capabilities a non-trivial module needs,
**5 gate-fail** (generics M-657, traits M-659, effects M-660, `wild`/FFI M-661, static guarantee
index M-663) and cross-nodule phyla is partial (M-662); `web` needs all of these, so `.myc`-authoring
is blocked. Build order (deepest-dependency-first, research T12.4.5):

1. **`web.http`** — the value model + never-silent parsing (pure; over `std.text`/`std.collections`).
2. **`web.json`** — thin delegation to `mycelium_std_io::{from_json, to_json}` (no new codec).
3. **`web.client`** — `get`/`post`/`request` over an injected `Source`/`Sink` (the affine
   `substrate`, LR-8, so the pure logic stays `wild`-free and the OS floor is isolated to `std-sys`).
4. **`web.route`** — the inspectable route table + dispatch + EXPLAIN record (pure).
5. **`web.server`** — `serve` over the landed `mycelium-std-runtime` `Colony`/`Scope`/`Task` (R1
   deterministic fork-join + the RT2 differential); one request-handling `Task`-per-connection joined
   by the `Scope` (RT7). A server **germinates from a `spore`** (ADR-013; RFC-0008 §4.4).

Each nodule ships its `guarantee_matrix.rs` (§4.5), asserted in tests. The workspace `Cargo.toml`
members list is an orchestrator-owned shared file (the swarm convention).

## 5. Drawbacks

- **Surface area + the contract cost.** A web stack held to C1–C6 (located errors, EXPLAIN-able
  routing, declared effects, a per-nodule guarantee matrix) is more work than a conventional HTTP
  library. This is the point (it is the product), but it must stay ergonomic — the RFC-0016 §8-Q3
  "honesty's verbosity" tension applies (§10.2 U8).
- **Two language gates bound the `.myc` surface.** The typed `Json<T>` handler ergonomic (E7-1) and
  the `colony { hypha … }` server surface (E7-2) cannot be written in `.myc` until those epics land.
  v1 is Rust-first only; the gap is named, not hidden.
- **HTTP/1.1 only in v1.** HTTP/2, HTTP/3, TLS, and WebSockets are explicit non-goals (§6); the value
  model survives a later transport, but the transport itself is deferred.
- **The deterministic default costs racing ergonomics.** Hedged/first-wins upstream patterns are RT3
  constructs with named policies (more ceremony than mainstream async); the multi-source primitive
  they need is not yet landed (§10.2 U15). That ceremony *is* the honesty rule (RFC-0008 §5).

## 6. Scope & non-goals

**In scope (v1):** HTTP/1.1 client + server + routing; JSON ↔ `Value` over `std.io`'s canonical
codec; never-silent parsing everywhere; the server as a `colony` of request `hyphae`; the Rust-first
`mycelium-web` crate.

**Non-goals (FLAGGED for deep-research / a later RFC):** HTTP/2 + HTTP/3 (multiplexing, HPACK/QPACK,
flow control — RFC 9113/9114; §10.2 U3); TLS/HTTPS (cert validation, ALPN — §10.2 U4); WebSockets;
streaming-body backpressure tuning (§10.2 U6); the full request-smuggling/header-injection
adversarial pass (§10.2 U5); the typed-handler ergonomics (E7-1) and the `.myc` runtime surface
(E7-2). These are honestly deferred, not silently omitted.

## 7. Rationale & alternatives

- **Why one `web` phylum, not many independent libraries?** The RFC-0016 §6 rationale applies: one
  content-addressed phylum with independently-importable nodules gives coherence (one contract, one
  matrix format, one EXPLAIN style) without an all-or-nothing dependency. (Rejected: a constellation
  of unrelated packages, which would let the honesty contract drift per-library.)
- **Why delegate JSON to `std.io` instead of a web-native codec?** DRY/KC-3 (§4.3): `std.io` already
  owns the one canonical JSON; a second codec would duplicate it and re-introduce trusted code.
  (Rejected: a `web`-internal JSON parser.)
- **Why model the server as a `colony` of `hyphae` rather than an actor mailbox or a thread pool?**
  RFC-0008's deterministic-fragment-first model gives request isolation (RT1), structured shutdown
  (RT7), explicit failure (RT4), and a replayable reference (RT2) — the never-silent guarantees the
  rest of Mycelium has, extended to the server. (Rejected: an actor model nondeterministic at the
  root, which surrenders RT2's deterministic default — RFC-0008 §6.)
- **Why Rust-first, not Mycelium-first?** ADR-007 + DN-14: the surface language cannot author this
  phylum yet (generics/traits/effects/`wild` gate-fail). Building `.myc`-first now would be building
  on sand with no reference to differential against — the RFC-0016 §4.6 migration path is the honest
  route (research T12.4.5/T12.4.6).

## 8. Prior art

*(Empirical — design-informing, never `Proven`; research T12.1.13.)* Value model ≈ the Rust `http`/
`hyper` family (`Request<B>`, `Response<B>`, `HeaderMap`, fallible `StatusCode::from_u16`, `Method`,
`Uri`); ergonomic client ≈ `reqwest`; server/routing ≈ `axum` (Router + extractors); integrated
cross-check = Go `net/http` (`ServeMux`, `Handler`, explicit error returns). Normative protocol
grounding: **RFC 9110** (HTTP semantics) + **RFC 9112** (HTTP/1.1 message syntax) — these obsolete
RFC 7230–7235 — and the **WHATWG URL Standard** (the modern, test-suite-backed, never-silent URL
parser; the Rust `url` crate) over the looser RFC 3986 ABNF where they diverge. The Mycelium-specific
shape (never-silent parsing, EXPLAIN-able routing, honest per-op tags, the `colony`-of-`hyphae`
server) is **not** borrowed — it is the web-library form of the Accepted RFC-0008/RFC-0016 contracts.

## 9. Future possibilities

- **Self-hosted `web` in Mycelium-lang** once E7-1 lands (generics + traits + effects + `wild`), with
  the Rust reference retired after a migration differential (the RFC-0016 §4.6 trajectory).
- **The `.myc` `colony { hypha … }` server surface** once E7-2 (M-666) activates `hypha`/`colony` as
  L1 constructs.
- **Typed `Json<T>` handlers + a `Handler`/`Service` trait** once E7-1 (M-657/M-659) lands.
- **A v2 transport layer** — HTTP/2/3, TLS, WebSockets — each landing with its honest bounds and a
  placement decision for the `wild`/FFI floor (`std-sys`).
- **Guarantee-aware routing/placement** — forage policies that weigh a node's ability to serve a
  route (RFC-0008 §9), once distribution (R2) lands.

## 10. Honest-Uncertainty Register (what gates ratification)

> The load-bearing output of this Phase-1 pass: the explicit split between what is
> **design-decidable now** (this RFC fixes it, Rust-first, grounded) and what the **follow-up
> deep-research pass must verify** before RFC-0022 can move past Draft. Per the two-phase discipline
> and the honesty rule (G2/VR-5), nothing in §10.2 is invented into a false-confident choice — each
> is a FLAGGED open question. **The RFC stays Draft until §10.2 is discharged.** Full detail (D1–D10,
> U1–U17) is in `research/12-web-phylum-RECORD.md` §6.

### 10.1 Design-decidable now (this RFC fixes these)

- **D1–D2.** The five-nodule decomposition + acyclic dependency graph; the Ring-2, above-kernel,
  zero-trusted-code posture (KC-3). *(RFC-0016 §4.2/§6/C5.)*
- **D3–D4.** The immutable value model (Request/Response/Headers/Status/Method/Url/Body); the
  never-silent parse discipline + the located error enums. *(RFC-0001; G2/C1; io.md/text.md.)*
- **D5.** `Headers` over `std.collections`; `web.json` delegates to `std.io`'s one canonical JSON
  (no new codec). *(DRY/KC-3; io.md §7-Q1.)*
- **D6.** The honest per-op tags (HTTP `Exact`-when-Ok/fallible; JSON decode `Empirical` / encode
  `Exact`-fallible-refusing-non-finite; server determinism `Empirical`-via-differential not
  `Proven`; handler purity `Declared`) + the matrix-as-data obligation with VR-5 guard-tests.
  *(RFC-0016 §4.1/§4.5; io.md/`std-runtime` precedents.)*
- **D7.** Routing is reified/EXPLAIN-able (no opaque heuristic); 404/405 explicit. *(C3.)*
- **D8.** Server = a `colony` of request `hyphae` (RT1/RT2/RT4/RT7; §4.7 C1–C4). *(RFC-0008.)*
- **D9.** v1 is Rust-first (`mycelium-web`) on the landed runtime; build order
  http→json→client→route→server. *(ADR-007; RFC-0016 §4.6; DN-14.)*
- **D10.** HTTP/1.1 (RFC 9110 + 9112) + WHATWG-URL as the v1 protocol target; never-silent
  everywhere.

### 10.2 Deep-research-must-verify (FLAGGED — gates ratification)

- **U1 — Phylum + nodule naming** (`web`/`http`/`json`/…): themed-vs-conventional + the phylum name,
  a DN-level lexicon call (RFC-0016 §8-Q2; DN-02 three-test gate).
- **U2 — The socket `wild`/FFI floor:** a `web`-internal nodule vs the shared `std-sys` phylum
  (LR-9 leak-free) (RFC-0016 §8-Q6; io.md §7-Q4 analogue).
- **U3 — HTTP/2, HTTP/3** (RFC 9113/9114); **U4 — TLS/HTTPS** (cert validation, ALPN; ties to U2 +
  a security pass); **U5 — the full request-smuggling/header-injection threat model** (a
  `/security-review`-grade adversarial pass).
- **U6 — Streaming-body backpressure** (touches `hypha`/colony scheduling — undecidable until the R1
  scheduler + the E7-2 surface land).
- **U7 — WHATWG-vs-RFC-3986 URL divergences + IDNA/Punycode** (needs a versioned, reified,
  inspectable Unicode/IDNA table — text.md §7-Q2 analogue). **RATIFIED (2026-06-21):** the *policy* is
  fixed — pin a Unicode version, **nontransitional** processing, **fail-not-best-effort**; `dfb` pins
  the exact Unicode/UTS-46 version current at build and records it (WHATWG leaves it unpinned). The
  table is versioned + reified + inspectable.
- **U8 — The `net` effect granularity** (coarse `io` vs a finer `net`/socket capability; RFC-0014 /
  RFC-0016 §8-Q3; effect-annotation surface is E7-1-gated, M-660).
- **U9 — Content-negotiation selection** (`Accept`/`q`-value ranking, RFC 9110 §12 — the one
  *selection* op; stays `Exact` but **must reify which representation it chose and why**, C3).
- **U10 — The exact body → `Source`/`Sink` seam** (in-memory `Bytes` vs a streaming `Source`;
  io.md §7-Q3 analogue); **U11 — the cross-request shared-state mechanism** (`fuse` vs `graft` vs a
  per-colony owned value; `xloc`/`graft` are R2/M-668, `fuse` is M-667).
- **U12 — Wall-clock request timeouts / deadlines / keep-alive** (v0 budgets use a logical clock;
  RFC-0008 R8-Q3); **U13 — graceful-shutdown precise contract** (drain-vs-deadline-vs-force; ties to
  U12).
- **U14 — Async runtime choice + scheduler fairness/backpressure under load** (the corpus runtime is
  single-threaded cooperative; RFC-0008 R8-Q1); **U15 — the multi-source `select`/`merge` (RT3)
  primitive** (deferred / not yet landed; any racing feature is an RT3 construct with a reified
  policy + EXPLAIN).
- **U16 — Whether any JSON round-trip reaches `Proven`** (inherited from io §7-Q2; tagged at
  *established* strength, not fabricated — VR-5).
- **U17 — Typed `Json<T>` handler ergonomics + the `.myc` `colony`/`hypha` server surface** (both
  gated: E7-1 M-657/M-659; E7-2 M-665→M-666; design-sketchable, not buildable in `.myc` until those
  epics land).

## Meta — changelog

- **2026-06-21 — RATIFIED → Accepted (maintainer).** RP-10 discharged clean (no soundness/completeness
  gap), so the design is ratified. Decisions: **IDNA/UTS-46** — ratify the policy (pin a Unicode
  version, nontransitional, fail-not-best-effort; `dfb` pins the exact version at build — U7);
  **`web.server`** runs on the Mycelium colony/hypha runtime (no external executor; realization
  `mycelium-mlir::runtime`, tracking RFC-0023 R23-Q1); **v1 non-goals confirmed** (HTTP/2-3, TLS, the
  cross-peer smuggling threat model — §6). **Accepted = design agreed; Enacted gated on the
  `mycelium-web` build.** Append-only; trail preserved in the Status cell. Empirical/Declared, never
  `Proven` (VR-5).
- **2026-06-21 — RP-10 research gate discharged (Phase-2 deep-research follow-up; `dfr` session).**
  Four fractured Opus sub-reasoners (W1 HTTP/never-silent · W2 JSON-codec-reuse · W3
  server-determinism · W4 routing/EXPLAIN) verified §10.2 against primary specs + landed source; all
  four obligations **design-sound, no falsification** (`research/12-web-phylum-RECORD.md` §8).
  Residuals categorized (deferred-to-build empirical-on-code; scoped-future dependency-gated/non-goal)
  and named — none silently closed (G2). Status appended: **"RP-10 research gate discharged; pending
  maintainer ratification"** (the agent stages; the maintainer ratifies to Accepted). Findings
  Empirical/Declared, never `Proven` (VR-5). Append-only; no design content rewritten.
- **2026-06-21 — Draft (Phase-1 design pass).** Stands up the web-tooling phylum (working name
  `web`) as five nodules over one `phylum` — `web.http` (value model + never-silent parsing),
  `web.json` (a thin convenience over `std.io`'s one canonical JSON — no new codec), `web.route`
  (inspectable, EXPLAIN-able dispatch), `web.server` (a `colony` of request-handling `hyphae` per
  RFC-0008), `web.client` (fallible, `io`-effecting). **Never-silent throughout** (every parse a
  located `Result`; a non-finite f64 in JSON refused, never silent `null`; a no-route an explicit
  404; a failed handler an explicit `TaskOutcome`, never a dropped request). **Honest per-op tags**
  carried from the landed std precedents (HTTP `Exact`-when-Ok/fallible; JSON decode `Empirical` /
  encode `Exact`-fallible; server determinism `Empirical`-via-RT2-differential, **not `Proven`**;
  handler purity `Declared`) with the §4.5 guarantee-matrix obligation + the VR-5 guard-tests.
  **Ring 2, above the kernel, zero trusted code (KC-3).** **Rust-first** (`mycelium-web`, mirroring
  the 25 `std-*` crates) because DN-14 records generics/traits/effects/`wild` as gate-fails; the
  `.myc` typed `Json<T>` surface is **E7-1-gated** (M-657 generics + M-659 traits) and the `colony {
  hypha … }` server surface is **E7-2-gated** (M-665 → M-666) — both named as explicit dependencies,
  not sketched as available. The **§10 Honest-Uncertainty Register** (D1–D10 decidable-now; U1–U17
  deep-research-must-verify — naming, the `wild`/socket floor, HTTP/2-3, TLS, smuggling,
  backpressure, IDNA, the `net` effect, content-negotiation, the body↔Source seam, shared state,
  wall-clock time, graceful shutdown, the async runtime, multi-source RT3, the `Proven` round-trip,
  the gated surfaces) **gates ratification**; the RFC stays Draft until §10.2 is discharged.
  Grounded in `research/12-web-phylum-RECORD.md` (T12.1.x–T12.4.x); no guarantee tag is `Proven`
  without a checked basis; every claim cites its basis or is FLAGGED (G2/VR-5). No code with this
  draft; no kernel change (KC-3). Lineage: RFC-0016 (the std contract this mirrors) + RFC-0008 (the
  runtime model the server instances) → **RFC-0022** (this scope) → Rust-first `mycelium-web` +
  the E7-1/E7-2-gated `.myc` surfaces.
