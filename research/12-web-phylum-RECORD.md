# Research Record 12 — Web-Tooling Phylum (HTTP client/server/routing/JSON), Phase-1 design pass (RFC-0022)

> **What this file is.** A durable record of the **fractured research pass** that produced the
> Draft **RFC-0022 — Web-Tooling Phylum**. Conducted 2026-06-21 from the cross-context packet:
> `CLAUDE.md` house rules + `.claude/memory/lang-lexicon-syntax.md` (lexicon/honesty lattice),
> `docs/rfcs/RFC-0016` + `docs/spec/stdlib/{io,fs,text,error,collections}.md` +
> `crates/mycelium-std-io` (the std phyla this builds on; API/guarantee-tag conventions),
> `docs/rfcs/RFC-0008` (colony/hypha concurrency — the E7-2 gap), `docs/rfcs/RFC-0001`
> (value model), `docs/adr/ADR-013` (spore), and `docs/notes/DN-14` (the self-hosting gate).
> Findings are labelled **T12.1–T12.N** under four sub-endeavours (each a focused max-effort
> sub-reasoner sharing the packet): **(1)** HTTP protocol + parsing surface, **(2)** server
> concurrency model, **(3)** JSON ↔ `Value` never-silent (de)serialization, and **(4)** the phylum
> surface, honest guarantees, examples, and Rust-first build plan.
>
> **Posture (honesty rule / VR-5).** This is a **Phase-1 design** record, not a verification pass.
> Findings are **Empirical/Declared** (design-informing prior-art + corpus-grounding), **never
> `Proven`** — none is a theorem. Per the two-phase discipline, RFC-0022 stays **Draft** and a
> **follow-up deep-research pass gates ratification** (the §Honest-Uncertainty Register, below,
> enumerates exactly what that pass must verify). Every claim cites its basis or is **FLAGGED**.
> Append-only.

---

## 1. Scope

RFC-0022 designs a **web-tooling phylum** (working name `web`): an HTTP/1.1 client + server +
routing surface and a JSON ↔ `Value` surface, **never-silent throughout** (every parse a
`Result`, every selection reified/EXPLAIN-able). It is a **library above the kernel** (KC-3) that
builds on the landed std phyla (`std.io`, `std.text`, `std.error`, `std.collections`) and the
landed Rust runtime (`mycelium-mlir::runtime` / the ADR-020 `runtime` phylum). Self-hosting is
blocked (DN-14 gate-fails: generics, traits, effects, `wild`/FFI all absent at the L1 surface),
so **v1 is Rust-first** — a Rust crate `mycelium-web` exposing the phylum surface, mirroring how
the 25 `mycelium-std-*` crates landed Rust-first (RFC-0016 §4.6, Enacted). The `.myc` server
surface (`colony { hypha … }`) and typed `Json<T>` handlers are gated on epics **E7-2** (M-665 →
M-666) and **E7-1** (M-657 generics, M-659 traits) respectively.

The four sub-endeavours below decompose the design surface; §6 is the **Honest-Uncertainty
Register** (design-decidable-now vs deep-research-must-verify) that gates ratification.

---

## 2. Sub-Endeavour 1 — HTTP protocol + parsing surface (T12.1.x)

> Grounding spine: `docs/spec/stdlib/{io,text,fs,collections,error}.md` (all Accepted, DN-07);
> RFC-0001 (value model); RFC 9110 (HTTP semantics) + RFC 9112 (HTTP/1.1 message syntax) + the
> WHATWG URL Standard (prior art, Empirical). The `web.http` parse layer is the never-silent
> floor the other three sub-endeavours build on.

**T12.1.1 — Every HTTP value is an immutable Mycelium value with content-addressed identity
(C4/ADR-003).** `Request`, `Response`, `Headers`, `Status`, `Method`, `Url`, `Body` are
value-semantic, immutable-by-default `type`s; every "mutator" (`with_header`, `with_status`)
returns a *new* value, exactly as `std.collections` structures do. *Grounding: RFC-0001 value
model + §4.6 content-addressing; ADR-003 (metadata is not identity); `collections.md` C4.* Prior
art (Empirical): the Rust `http` crate — `Request<B>`, `Response<B>`, `HeaderMap`, `StatusCode`,
`Method`, `Uri`.

**T12.1.2 — `Headers` is a value-semantic multi-map built on `std.collections`, not a fresh
structure (DRY/KC-3).** HTTP headers are a case-insensitive multi-map (a field-name may repeat:
`Set-Cookie`, `Via`; RFC 9110 §5). Model it as a thin wrapper over
`collections::Map<HeaderName, Seq<HeaderValue>>`, reusing `collections`' never-silent
`get → Option` and explicit-named-default `get_or` (a default is never silent — `collections.md`
§3). `HeaderName` normalizes ASCII case **on construction** (case-insensitive *by value*, not by
an opaque heuristic — C3). *Grounding: RFC 9110 §5; `collections.md`; DRY/KISS. Prior art:
`http::HeaderMap`.*

**T12.1.3 — `Status` is a validated value, never a bare integer; out-of-range is an explicit
parse error.** A status code is `100..=599` (RFC 9110 §15). `Status::from_u16(n) -> Result<Status,
HttpParseError>` refuses an out-of-range code with an explicit located `Err`, never a clamp or
sentinel — the analogue of `text`'s `parse_int` (never a sentinel `0`) and `http`'s fallible
`StatusCode::from_u16`. Accessors are total + `Exact`. *Grounding: RFC 9110 §15; `text.md` C1.*

**T12.1.4 — `Method` is a closed sum plus an explicit extension arm — never a silently-accepted
free string.** `type Method = Get | Head | Post | Put | Delete | Connect | Options | Trace | Patch
| Extension(Token)`; `Extension` carries a *validated* `Token` (RFC 9110 §9.1 method = token;
§16.1 registry). A non-standard method is validated as a syntactic `token` and routed to
`Extension` explicitly — never silently coerced. *Grounding: RFC 9110 §9/§16.1; G2.* FLAG: whether
`Extension` is admitted at all is a policy knob (some servers reject unknown methods) — Register.

**T12.1.5 — `Url` is a parsed structured value; raw text → `Url` is the URL never-silent crux.**
`Url::parse(s: Text) -> Result<Url, UrlParseError>` follows the **WHATWG URL Standard** (the
modern, test-suite-backed, never-silent parser; the Rust `url` crate implements it) over the
looser RFC 3986 ABNF where they diverge — a malformed URL is an explicit located error, never a
best-effort partial. *Grounding: WHATWG URL Standard; RFC 3986; `text.md` C1.* FLAG:
WHATWG-vs-RFC-3986 divergences + IDNA/Punycode for non-ASCII hosts need a *versioned* Unicode/IDNA
table — the same reified-table FLAG as `text`'s grapheme segmentation (`text.md` §7-Q2) — Register.

**T12.1.6 — `Body` is an affine-`Substrate`-backed byte stream OR an in-memory value; the
streaming case threads `std.io`'s single-consumption handle (LR-8).** A small/known body is an
in-memory `Bytes` value; a streaming body is read over `std.io`'s affine `Source` consumed exactly
once (`io.md` §3). `Body` does **not** re-define byte transfer — it threads `std.io`'s
`Source`/`Sink`, exactly as `std.fs` threads io ("does not re-define `Read`/`Write`", `fs.md`
§2/§7-Q2). *Grounding: RFC-0006 LR-8; `io.md` §3; `fs.md`.* FLAG: streaming-body **backpressure**
is a runtime/effect concern (touches `hypha`/colony scheduling, not yet lexed) — Register.

**T12.1.7 — Every HTTP parse is `parse → Result<T, ParseError>` carrying a WHERE locus, modeled
exactly on `std.io`'s `SerError` and `std.text`'s `ParseErr`.** A malformed request-line, header
block, status-line, or URL is an **explicit, located** error — never a sentinel/clamp/partial. The
error carries an RFC-0013 diagnostic record so the failure is legible (where + why). *Grounding:
G2/C1; `io.md` §3 (`SerError @locus`); `text.md` §3 (`ParseErr` with WHERE); RFC-0013 §4.1.*

**T12.1.8 — Sketched error enums (illustrative, house-style, each carrying a WHERE):**

```
enum HttpParseError {
  RequestLineMalformed { at: ByteOffset, why },
  StatusLineMalformed  { at: ByteOffset, why },
  BadVersion           { at: ByteOffset, found },     // not HTTP/1.0|1.1 (RFC 9112 §2.3)
  BadMethodToken       { at: ByteOffset, found },      // not a valid token (RFC 9110 §9.1)
  StatusOutOfRange     { at: ByteOffset, code },       // not in 100..=599 (RFC 9110 §15)
  HeaderMalformed      { at: ByteOffset, why },
  BadHeaderName        { at: ByteOffset },             // not a token (header-injection guard)
  ObsFold              { at: ByteOffset },             // obsolete line-folding rejected (RFC 9112 §5.2)
  Truncated            { at: ByteOffset },
  BadContentLength     { at: ByteOffset, why },        // absent/duplicate/non-numeric/conflicting (§6.3)
  BadTransferEncoding  { at: ByteOffset, why },        // malformed chunked framing (RFC 9112 §7)
}
enum UrlParseError {
  MissingScheme { at: ByteOffset },  BadScheme { at: ByteOffset, found },
  BadHost { at: ByteOffset, why },   BadPort  { at: ByteOffset, found },   // not 0..=65535, never a clamp
  BadPercentEnc { at: ByteOffset },  NonAsciiUnencoded { at: ByteOffset },  // needs IDNA / percent-encoding
}
```

*Grounding: RFC 9112 §2–§7; RFC 9110 §9/§15; WHATWG URL Standard; the `@locus`/`why` shape is
`io.md`/`text.md` house style.*

**T12.1.9 — Security-critical refusals are *parse errors*, not lenient acceptance —
header-injection and request-smuggling vectors are closed by construction.** `BadHeaderName`/
`HeaderMalformed` reject CR/LF/NUL in field-names/values (header injection / response splitting);
`BadContentLength`/`BadTransferEncoding`/`ObsFold` reject the duplicate-`Content-Length`,
conflicting-`Transfer-Encoding`-vs-`Content-Length`, and obs-fold cases that drive **HTTP request
smuggling** (RFC 9112 §5.2 obs-fold MUST-reject; §6.3 length precedence; §7 chunked). *Grounding:
RFC 9112 §5.2/§6.3/§7; G2 (refuse, never silently normalize).* FLAG: the **full** smuggling /
normalization threat model is a `/security-review`-grade follow-up — Register.

**T12.1.10 — The client surface is `get/post/request`, every call fallible + `io`-effecting.** A
network request declares an `io` (or finer `net`) effect on its signature, as every `fs` op does
(`fs.md` §3); the response is `Result<Response, HttpError>` — connection failure, timeout, or a
malformed response are explicit located errors, never a sentinel response. *Grounding: prior art
`reqwest` / Go `net/http`; RFC-0014 §4.5; `fs.md` C6.*

**T12.1.11 — Routing is over an *inspectable, reified* route table — opaque-heuristic routing would
violate C3.** A route table maps `(Method, PathPattern) → Handler`; dispatch returns an explicit
`Result<(Handler, PathParams), RouteError>` where `NotFound`/`MethodNotAllowed` are explicit
(→ 404/405, never a silent fallthrough); *which* route matched and *why* is reifiable (C3 — "a
route that matches by an opaque heuristic violates C3"). *Grounding: cross-context C3; prior art
`axum` Router / Go `ServeMux`.* FLAG: pattern-precedence (longest-prefix vs first-match vs
most-specific) must be a documented, deterministic, inspectable rule, never an opaque ranker.

**T12.1.12 — HTTP ops carry no accuracy/probability semantics → every op tags `Exact`; the honesty
is carried entirely by the *fallibility* column.** HTTP parsing is **exact-or-error** by nature
(no ε, no δ, no `BoundBasis`): a parse is `Exact`-*when-Ok* and an explicit located `Err`
otherwise (no "approximately parsed" state). So no op is `Proven`/`Empirical`/`Declared`; `Exact`
is the honest floor (RFC-0016 C2), identical to `text`/`fs`/`collections`. The one *future*
non-floor case is **content-negotiation** (`Accept`/`q`-value ranking, RFC 9110 §12) — a
*selection* that must be reified/EXPLAIN-able, but a deterministic ranking, so still `Exact` with
the selection reified. *Grounding: `text.md`/`fs.md`/`collections.md` §4; RFC-0016 C2; RFC 9110
§12; VR-5.*

**T12.1.13 — Prior art (Empirical — design-informing, never Proven).** Value model ≈ Rust
`http`/`hyper`; ergonomic client ≈ `reqwest`; server/routing ≈ `axum`; integrated cross-check =
Go `net/http`. Normative protocol grounding: **RFC 9110** (HTTP semantics) + **RFC 9112**
(HTTP/1.1 message syntax) — these obsolete RFC 7230–7235 — plus the **WHATWG URL Standard** for
never-silent URL parsing.

### Surface sketch (illustrative — NOT committed grammar; house style per `io.md`/`fs.md` §3)

```
// nodule: web.http   — the value model (pure, value-semantic)
type Method  = Get | Head | Post | Put | Delete | Connect | Options | Trace | Patch | Extension(Token)
type Status  // validated 100..=599
type Headers // value-semantic multi-map over collections::Map<HeaderName, Seq<HeaderValue>>
type Url     // parsed WHATWG components
type Body    // in-memory Bytes | streaming Source (LR-8, threads std.io)
type Request  // { method, url, headers, body, version }
type Response // { status, headers, body, version }

Status::from_u16(n: u16)             -> Result<Status, HttpParseError>   // out-of-range -> Err, never clamp
Headers::get(h: &Headers, n: &HeaderName) -> Option<&HeaderValue>        // None on absent (never a default)
Method::parse(t: &Text)              -> Result<Method,  HttpParseError>   // Err names the bad token
Url::parse(t: &Text)                 -> Result<Url,     UrlParseError>    // WHATWG; Err @ offending component
parse_request(raw: &Bytes)           -> Result<Request, HttpParseError>  // RFC 9112 wire; Err @ byte offset
serialize_request(r: &Request)       -> Bytes                            // total faithful projection

// nodule: web.client  — fallible + io-effecting
get(u: Url)                          -> Result<Response, HttpError> !{ io }
post(u: Url, b: Body)                -> Result<Response, HttpError> !{ io }
request(r: Request)                  -> Result<Response, HttpError> !{ io }

// nodule: web.route   — REIFIED, inspectable dispatch (opaque routing violates C3)
type Handler = fn(Request) -> Result<Response, HttpError> !{ io }
match_route(t: &RouteTable, m: &Method, p: &Path)
                                     -> Result<(Handler, PathParams), RouteError>   // EXPLAIN-able
enum RouteError { NotFound, MethodNotAllowed { allowed: Seq<Method> } }             // -> 404 / 405

enum HttpError { Parse(HttpParseError), Url(UrlParseError), Connect{why}, Timeout, Closed, Io(IoError) }
```

---

## 3. Sub-Endeavour 2 — Server concurrency model (T12.2.x)

> Grounding spine: RFC-0008 (RT1–RT7 §4.1; §4.7 C1–C4; §4.5 vocabulary; §4.6 staging; §8
> R8-Q1/Q3); ADR-020 (the `runtime` phylum + `std.runtime` facade + the test-enforced honest
> guarantee tags — **the template** for the web concurrency surface); the E7-2 epic (issues.yaml).

**T12.2.1 — Server = a `colony` of one request-handling `hypha` per connection/request.** The
server is a long-lived `colony` (dynamic runtime grouping of cooperating `hyphae`; DN-06; RFC-0008
§4.7) whose accept-loop spawns one request `hypha` (a structurally-scoped concurrent execution
unit over immutable values; RFC-0008 §3/§4.5) per accepted connection/request. A faithful instance
of the OTP / structured-concurrency server shape (RFC-0008 §7 prior art), not a new mechanism.
*Grounding: RFC-0008 §4.7; DN-06.*

**T12.2.2 — RT7 binds the server's lifetime: no orphan request `hypha`; graceful shutdown is
structural.** Every request `hypha` is created inside the server `colony`'s scope, which does not
exit until each child has completed / been cancelled / been explicitly detached (RT7 §4.1; "a
leaked task is not expressible"). **Graceful shutdown = the colony scope closing**: stop accepting,
then wait for in-flight request `hyphae` to drain or observe cancellation (§4.7 C2). *Grounding:
RFC-0008 RT7 §4.1, §4.7 C2; the landed `Scope` join-all discipline (M-357).*

**T12.2.3 — RT1 binds request isolation: a handler gets an immutable `Request` value, returns a
`Response` value; no ambient shared session state.** The only thing crossing a boundary is an
immutable `Value` with its `Meta` intact (RT1 §4.1). Any cross-request shared state (cache,
connection pool, session store) is **not ambient** — it must be an explicit reified mechanism (an
`xloc`-moved value, a `fuse`-merged semilattice state — RT6, or an external `graft` capability —
RT4), never a silently-aliased mutable global. *Grounding: RFC-0008 RT1 §4.1; RT6; §4.5 `graft`.*
FLAG: the *concrete* shared-state default (the right primitive for a web cache / DB pool) is
design-decidable but not yet specified, and `xloc`/`graft`/`fuse` are themselves reserved-not-yet
-lexed (R2/M-667/M-668) — v1 must pick a concrete Rust representation and FLAG the anticipated
construct — Register.

**T12.2.4 — RT2 binds the default handler form: a per-request pure handler is in the deterministic
fragment.** A pure `Request → Response` handler lives inside RT2's deterministic fragment, so
request-dispatch has a sequential reference semantics the trusted interpreter can replay; the
honesty target is **concurrent observable ≡ the deterministic reference's observable** (NFR-7
extended; §4.2). *Caveat (honest boundary):* handlers that read wall-clock time, randomness, or
external I/O are **not** in the pure fragment — those are explicit effects (time = R8-Q3,
unresolved §8) and push the handler into RT3/effectful territory; the determinism guarantee then
covers only the pure core. *Grounding: RFC-0008 RT2 §4.1, §4.2, §8 R8-Q3.*

**T12.2.5 — RT4 binds partial failure: a failed handler resolves to an explicit
`TaskOutcome::Failed`, never a silently-dropped request.** A request `hypha` resolves to exactly
one explicit `TaskOutcome` — `Done`/`Failed`/`BudgetExhausted`/`Cancelled` — with **no
silent/dropped variant** (§4.7 C3); a handler error becomes an explicit `Failed` the colony *must*
act on (→ an HTTP error response). Never silently dropped or retried. Local and remote failures are
*different types* (RT4); v1 is the local (single-node) case. *Grounding: RFC-0008 RT4 §4.1, §4.7
C3 — the never-silent rule (G2) at the request boundary.*

**T12.2.6 — §4.7 C1 binds per-request budgets/timeouts: each request `hypha` carries its own budget
ledger; an overrun is in-that-request, never global.** A per-request timeout / resource budget is a
per-`hypha` `Budgets` ledger (M-353); one runaway request exhausting *its* budget cannot exhaust
another's, and surfaces as that request's explicit `BudgetExhausted` (→ 503/504), never a
server-wide stall. *Grounding: RFC-0008 §4.7 C1; M-353.* FLAG: wall-clock *timeouts* depend on
**R8-Q3** — v0 budgets use a deterministic **logical clock** (M-356 C4), so a real-time deadline is
not yet honestly expressible — Register.

**T12.2.7 — §4.7 C2 + C4 bind a supervised, cancellable accept loop.** Cancellation is a
**cooperative token** observed at budget-check points (never preemptive), yielding an explicit
additive `Cancelled`, propagating down the colony tree (C2/RT7). A `reclaim` supervisor restarts a
failed child under a cascade **bounded on both axes** — total restart cap (the `cascade` budget,
M-353) **and** a windowed max-restart-intensity (≤ N within W *logical* ticks; Erlang/OTP; Record
05 T5.3) — exceeding either escalates (never an unbounded restart storm, C4). So an accept loop
that keeps crashing escalates rather than hot-looping. *Grounding: RFC-0008 §4.7 C2/C4; enacted in
`mycelium_interp::supervise` (M-356, done).* FLAG: the intensity window is a **logical clock**;
real-time intensity is R8-Q3-deferred — Register.

**T12.2.8 — The E7-2 gap: `hypha`/`colony` surface constructs are NOT yet lexed/active in L1; the
`.myc` web server is E7-2-pending (M-665 → M-666). [FLAG — load-bearing dependency].** The runtime
vocabulary is **ratified-not-yet-lexed**: the 10 DN-03 §4 runtime terms are not in `keyword()` and
lex as ordinary identifiers; `colony` *is* lexed (`Tok::Colony`) but is **reserved-not-active** —
no L1 parser production consumes it. Activation track = epic **E7-2**: **M-665** (reserve all 10
terms in `keyword()`, never-silent G2; `depends_on: []`) → **M-666** (RFC-0008 R1 `hypha` +
`colony` L1 constructs: `colony { … }` → `Scope::join_all`, `hypha <expr>` → a `Task`;
deterministic forms only; `depends_on: [M-665]`) → M-667 (`fuse`/`reclaim`/`tier`) → M-668 (R2).
**Therefore `colony { hypha handle(req) … }` cannot be written today** — it is gated on M-666.
*Grounding: lexicon reserved-word table; issues.yaml E7-2 / M-665 / M-666.*

**T12.2.9 — The Rust-first interim: v1 web server is built on the landed Rust runtime; the L1
surface arrives with M-666.** Landed substrate: **M-357** (done) `mycelium-mlir::runtime` —
`Scope<T,E>` (with `type Colony<T,E> = Scope<T,E>`), `Task`, `TaskCtx`, `Poll`, `SweepOrder`,
`Deadlock`; `run_sequential`/`run_interleaved`/`run_dataflow`; RT2 differential green; plus
`mycelium-mlir::channel` (typed SPSC, bounded demand-signalled backpressure, explicit
close/`Deadlock`, Kahn-determinism differential green). **M-356** (done) — per-task budgets +
cooperative cancellation + bounded-cascade supervision. **ADR-020** (Enacted; M-521) already
exposes this as a stdlib surface — a dedicated **`runtime` phylum** with a thin **`std.runtime`**
facade. **Conclusion:** v1 web server `colony`/`hypha` is built on `mycelium-mlir::runtime`'s
`Scope`/`Colony`/`Task` (or the `std.runtime` facade), with the `.myc` surface arriving at M-666.
The **ADR-020 hybrid phylum + facade shape is the template** for how the web phylum exposes
concurrency. *Grounding: issues.yaml M-356/M-357 (done); ADR-020; `crates/mycelium-mlir/src/{runtime,channel}.rs`.*

**T12.2.10 — Honest guarantee tags for the concurrency surface (VR-5) — inherited from ADR-020
§4 / `std.runtime`'s test-enforced matrix.** Per-request handler determinism (RT2/NFR-7) is
**`Empirical`** — observable-equivalent to the sequential reference *by differential, not theorem*
(`Scope` row; Kahn T4.1 the basis; **not `Proven`** — no mechanized proof in-repo; the matrix test
`kahn_determinism_is_empirical_not_proven` enforces this, mirroring RFC-0008's own M-357 tagging).
Request isolation (RT1) is **structural / `Declared`** (an absence of shared mutable state, a
declared contract on `Task` impls checked by the differential, never claimed `Proven`).
Structural/mechanical ops (`Poll`/`SweepOrder`/`Deadlock`/`TaskCtx`/bounded-channel) are `Exact`.
*Grounding: ADR-020 §4; `crates/mycelium-std-runtime/src/guarantee_matrix.rs`; RFC-0008 changelog
(M-357 Empirical tagging); VR-5.*

**T12.2.11 — RT3 nondeterministic constructs are OUT of the deterministic default and must be
FLAGGED with a reified policy + EXPLAIN.** Racing (first-response-wins / hedged requests across
upstreams), load-balanced upstream selection, and multi-source `select`/`merge` on inbound channels
are all **RT3** constructs requiring a named, reified RFC-0005 policy + mandatory EXPLAIN — **not**
part of the deterministic default (RFC-0008 RT3 §4.1; §5 names "speculative hedged requests,
first-wins racing" as RT3). The multi-source `select`/`merge` primitive is **deferred / not yet
landed** in `mycelium-mlir::runtime`. So v1 must either avoid racing (deterministic upstream
choice) or FLAG it as a pending RT3 construct. *Grounding: RFC-0008 RT3 §4.1, §5, changelog
("Deferred … multi-source select/merge (RT3)").*

### Request → handler → response, mapped onto colony / hypha

*Illustrative; the `.myc` surface is **E7-2-pending (M-666)** — `colony`/`hypha` do not yet parse;
v1 is Rust-first on `mycelium-mlir::runtime` / `std.runtime`.*

```mycelium
// nodule: web.server
// (E7-2-pending illustration — colony{…}/hypha gated on M-666; not yet lexed/active)
colony server {
    hypha handle(req)   // req : Request (immutable Value, RT1) -> Response (immutable Value, RT1)
    // ... one `hypha handle(reqᵢ)` per request ...
    // RT7: `server` cannot exit until every request hypha completes / is Cancelled.
}
fn handle(req: Request) -> Response = ...
//   success            -> TaskOutcome::Done(Response)            (RT4 / §4.7 C3)
//   handler error      -> TaskOutcome::Failed(e)        -> 5xx   (never a dropped request — G2)
//   per-request budget -> TaskOutcome::BudgetExhausted  -> 503/504 (in-that-request — §4.7 C1)
//   shutdown/cancel    -> TaskOutcome::Cancelled                 (cooperative — §4.7 C2)
```

**v1 Rust-first equivalent:** `Scope`/`Colony` from `mycelium-mlir::runtime` (or `std.runtime`),
one `impl Task` per request, `run_*`-driven, `Supervisor` (M-356) for the accept loop.

---

## 4. Sub-Endeavour 3 — JSON ↔ `Value` never-silent (de)serialization (T12.3.x)

> Grounding spine: `docs/spec/stdlib/io.md` + `crates/mycelium-std-io/src/guarantee_matrix.rs`
> (the **one canonical JSON** + the exact tag precedent); RFC-0001 §4.8 (the wire form);
> DN-16 (the io↔fmt scope-distinct tag framing); DN-14 + E7-1 (the typed-`Json<T>` gate).

**T12.3.1 — The web JSON surface is a thin convenience layer over `std.io`'s one canonical JSON,
NOT a new codec (DRY / KC-3).** `std.io` already owns "the one canonical JSON projection"
(`to_json`/`from_json` over the M-003/M-104 contracts; io.md §Scope/§1/§2), and `std.fmt` already
delegates to it (M-372). The web phylum's body helpers **call** `mycelium_std_io::{to_json,
from_json}`, adding zero trusted serialization code (KC-3 as already practiced, io.md §C5). A
second JSON parser would violate DRY and re-introduce a trusted surface the io spec deliberately
avoids. *Grounding: io.md §Scope/§1/§2/§C5; `mycelium-std-fmt/src/lib.rs` (decode-error
classification lives ONCE in `std.io`).*

**T12.3.2 — The serialize direction is `Exact`-when-Ok but **fallible**: a non-finite f64 is
refused (explicit `Err(OutOfDomain)`), never a silent JSON `null`.** `to_json`/`serialize` are
`Exact` (no accuracy semantics, RFC-0016 C2 floor) but `Fallibility::Fallible`: "a non-finite f64
has no JSON form and is refused (never a silent `null`)" (`guarantee_matrix.rs:11,107–114,124–132`;
`serialize.rs` refuses with the payload index of the first non-finite scalar). The guard test
`serialize_and_to_json_refuse_non_finite_fallibly` fails if either op is flipped back to `Total`.
The web `json_response` serialize step inherits this exactly — a `Value` carrying `NaN`/`±∞`
yields a located `Err`, never a `null`-poisoned response body (C1/G2). *Grounding:
`guarantee_matrix.rs:11,107–132,311–324`; `serialize.rs`; io.md §4.*

**T12.3.3 — The decode direction (`from_json`/`deserialize`) is `Empirical`, NOT `Proven` —
round-trip fidelity rests on a proptest corpus, not a checked theorem (VR-5).** `from_json`/
`deserialize`/`read_value` are `Empirical`: "Round-trip property established by proptest corpus; no
checked theorem → `Empirical`, not `Proven`" (`guarantee_matrix.rs:12,116–145`). The guard
`no_op_is_proven_without_a_checked_theorem` fails on any inadvertent upgrade. The web `body_json`
decode step **inherits this `Empirical` tag** — it cannot claim a stronger guarantee than the codec
it delegates to (VR-5). *Grounding: `guarantee_matrix.rs:12,116–145,255–270`; io.md §4-Q2/§7-Q2.*

**T12.3.4 — The decode direction is never-silent: malformed/truncated body → explicit *located*
`Err` (byte offset / field path), never a partial or empty `Value` (C1/G2).** The io decode error
set is explicit and locus-bearing (`Err(Truncated|Malformed|UnknownTag|OutOfDomain|BudgetExceeded)
@locus`); io.md §C1 is categorical ("No op returns a partially-filled `Value`, a zeroed field, a
clamp, or a sentinel"); each decode failure is EXPLAIN-able (RFC-0013 diagnostic naming *where*). A
web `body_json` over a malformed request body produces a located, classified `Err`, never a
half-built value a handler might mistake for valid input — exactly the never-silent property an
HTTP body-parse boundary needs. *Grounding: `guarantee_matrix.rs:120,142,372–389`; io.md §3/§C1/§C3.*

**T12.3.5 — The web JSON body helpers compose the io codec with the HTTP body model; the helpers
are pure over the byte body (the io is the caller's).** Proposed thin surface: `body_json(req) ->
Result<Value, JsonError>` (read body bytes, then `from_json`; malformed → located `Err`,
`Empirical`); `json_response(v: Value, status) -> Result<Response, JsonError>` (`to_json(&v)` then
wrap; non-encodable → `Err(OutOfDomain)`, never `null`). These are **pure over the byte body**
(`effects: none`) — matching the io discipline that serialize/JSON ops "perform no IO themselves
(the io is the *caller's*, via a `Source`/`Sink`)". The *network* I/O (socket read/response write)
is the HTTP layer's declared `io` effect, not the JSON helpers'. *Grounding: io.md §4 (declared
effects), §C6; `guarantee_matrix.rs:347–360`.* FLAG: the precise body→bytes seam (in-memory
`Bytes` vs a streaming `Source` drained via `read_value`) is a co-design point with the HTTP body
model (mirrors io.md §7-Q3) — Register.

**T12.3.6 — Typed `Json<T>` is Rust-first / E7-1-gated; state the dependency with issue IDs
(FLAG).** A typed handler (`Json<T>` decoding a body into a user type `T`, encoding a `T` to a JSON
response) needs **generics + a `Serialize`/`Deserialize`-style trait parametric over `T`** — both
DN-14 **gate-fails**: generics (DN-14 §3 row 6; tracked **M-656 spec → M-657 impl**), traits
(DN-14 §3 row 7; tracked **M-658 spec → M-659 impl**), strict chain under epic **E7-1**. M-659's
acceptance is precisely "RFC-0016 §4.1 C1–C6 contract expressible as surface trait constraints" —
the very trait `Json<T>` needs. **Conclusion:** the untyped `Value`-level JSON (T12.3.5) works
**today, Rust-first**; the typed `Json<T>` is **Rust-first only / blocked at the Mycelium-lang
surface until M-657 + M-659 land**. M-649 (first self-hosted module) is itself DEFERRED post-1.0.
RFC-0022 must state this, not sketch `Json<T>` as available. *Grounding: DN-14 §3 rows 6–7, §4;
issues.yaml E7-1, M-656/657/658/659.*

**T12.3.7 — A JSON (de)serialize is a *projection*, NOT a `swap` — keep the lexicon distinction
load-bearing.** A `swap` changes a `Repr` paradigm and **emits a swap certificate**; a JSON
projection moves a value to/from text **preserving the same `Repr` and the same content-id**
(ADR-003: serialization is a projection, not identity; io.md §2/§9/§C4). RFC-0022 must NOT describe
`body_json`/`json_response` as swaps or imply they emit certificates. Header: `// nodule: web.json`.
*Grounding: io.md §2/§9/§C4; ADR-003; RFC-0001 §4.8.*

**T12.3.8 — The io↔fmt DN-16 scope-distinct precedent governs how the web surface tags decode.** The
same `from_json` call is tagged differently in two phyla deliberately: `std.io.from_json` =
**`Empirical`** (round-trip fidelity) vs `std.fmt.from_json` = **`Exact`** (decode determinism) —
both honest, different properties, neither over-claiming `Proven` (io.md §7-Q1; DN-16). For the web
surface, the load-bearing property is **round-trip fidelity of the body**, so `body_json` carries
the **`Empirical`** framing matching `std.io` (the codec it delegates to), aligned with the actual
property and honest (downgrade rather than overclaim). *Grounding: io.md §7-Q1; DN-16;
`guarantee_matrix.rs:133–145`.*

### Guarantee matrix — web JSON ops (tags inherited unchanged from `std.io` — the web layer mints none)

| Op | Guarantee tag | Fallibility (explicit error set) | Declared effects | EXPLAIN-able? |
|---|---|---|---|---|
| `body_json(req) -> Result<Value, JsonError>` | **`Empirical`** (round-trip fidelity via `std.io.from_json`; proptest, no theorem — VR-5) | `Err(Malformed \| UnknownTag \| OutOfDomain \| BudgetExceeded) @locus` (never a partial/empty `Value` — C1/G2) | `none` (pure over the byte body; the socket-read `io` is the HTTP layer's) | **yes** — wraps the RFC-0013 diagnostic @locus from `std.io` |
| `json_response(v: Value, status) -> Result<Response, JsonError>` | **`Exact`-when-`Ok`** (one canonical form; no accuracy semantics — RFC-0016 C2) | `Err(OutOfDomain)` — non-finite f64 refused, never a silent `null` (C1/G2) | `none` (pure projection; the response *write* is the HTTP layer's `io`) | **n/a** — faithful projection, no selection |
| *(opt.)* `body_json_from_source(src) -> Result<Value, JsonError> !{io}` (drain a streaming body) | **`Empirical`** (composes `read_value` = io + decode) | `Err(SerError \| IoError) @locus` | **`io`** | **yes** — diagnostic @locus |

**Cross-cutting honesty note:** every tag here is **inherited, not minted** — the web phylum
delegates to `std.io` and adds no codec, so it has no independent basis to raise any tag (VR-5/KC-3).
Any future `Json<T>` row must be marked **Rust-first / E7-1-gated (M-657 + M-659)**.

---

## 5. Sub-Endeavour 4 — Phylum surface, honest guarantees, examples, Rust-first build plan (T12.4.x)

> Grounding spine: **RFC-0016** (the model to mirror — one `phylum`, independently-importable
> `nodule`s, §4.1 C1–C6, §4.2 ring layering, §4.5 matrix-as-data, §4.6 Rust-first migration);
> `docs/spec/stdlib/{io,text}.md` + `crates/mycelium-std-io/src/guarantee_matrix.rs` (spec format +
> matrix idiom + the VR-5 guard-tests); `crates/mycelium-std-runtime/*` + ADR-020 (the server
> precedent); DN-14 + E7-1/E7-2 (the gates); ADR-013 + RFC-0008 §4.4 (spore → server germination).

**T12.4.1 — The phylum is one `web` phylum with five nodules; mirror RFC-0016's `std` structure.**
`web` is one content-addressed `phylum` (DN-06) whose independently-importable `nodule`s are
`web.http`, `web.json`, `web.route`, `web.server`, `web.client` — coherence (one contract, one
matrix format, one EXPLAIN style) without all-or-nothing dependency (RFC-0016 §3/§4.2/§6).
*Grounding: RFC-0016 §3/§4.2/§6; DN-06.* FLAG (naming): `web`/`http`/`json`/`route`/`server`/
`client` are working names; the phylum-name + themed-vs-conventional call is a DN-level lexicon
decision (RFC-0016 §8-Q2; DN-02 three-test gate). The conventional names score high on T-learn
(human/LLM familiarity), so themed names are not indicated, but this needs an append-only sign-off —
Register.

**T12.4.2 — Dependency graph (layered, acyclic, bottom-up).**

```
                         web.client ──┐         web.server
                              │       │            │  │
                              ▼       ▼            ▼  │
   web.route ───────────────► web.json ────► web.http (core value model)
        │                         │                │
        └─────────────────────────┴────────────────┘
                                  │
   depends on the std phyla:      ▼
   std.io (canonical JSON codec + Source/Sink) · std.text (UTF-8, parse) ·
   std.error/std.core (Option/Result, lattice tags) · std.collections (header map) ·
   [web.server only] the runtime/colony surface (std.runtime / RFC-0008 R1)
```

- `web.http` — base value model; depends on `std.core`/`std.error` (M-515/M-527), `std.text`
  (M-524), `std.collections` (M-511).
- `web.json` — thin convenience over `std.io`'s one canonical JSON (delegates to
  `mycelium_std_io::{to_json, from_json}`, the M-372 pattern); no new codec (KC-3). Depends on
  `std.io` (M-514) + `web.http`.
- `web.route` — depends on `web.http`; the inspectable route table is a `std.collections` structure.
- `web.client` — depends on `web.http` + `web.json` + the `std.io`/effect surface.
- `web.server` — depends on `web.http` + `web.route` + the runtime/colony surface (RFC-0008 R1;
  landed Rust-side as `mycelium-std-runtime`'s `Colony`/`Scope`/`Task`, M-521/ADR-020).

*Grounding: RFC-0016 §4.2; io.md §7-Q1/§2 (delegation seam); `mycelium-std-runtime/src/lib.rs`;
RFC-0008 §4.4 (a server germinates from a spore into a running hypha).*

**T12.4.3 — Ring layering / above-kernel posture (KC-3): `web` is entirely Ring 2, adds zero
trusted code.** Every web nodule is **Ring 2** (RFC-0016 §4.2) — consumes the trusted base + Ring-1
surfaces, enlarges nothing in the trusted base (C5). `web.json` wraps the landed M-104 codec via
`std.io` (no trusted serialization code); `web.http` is pure value logic over
`std.text`/`std.collections`; `web.server` consumes the `std.runtime`/RFC-0008 R1 executor (itself
`#![forbid(unsafe_code)]`, `wild`-free in v0). The only place `web` could bottom out in OS
facilities (the socket syscalls under `web.client`/`web.server`) is `wild`/FFI — which per
RFC-0016 §8-Q6 lives in the **separate `std-sys` phylum** so pure `web` stays leak-free by
construction (LR-9). *Grounding: RFC-0016 §4.2/C5/§8-Q6; io.md §C5/§7-Q4; `mycelium-std-runtime/src/lib.rs`.*
FLAG: the socket/`wild` floor is the web analogue of io §7-Q4 — Register.

**T12.4.4 — Per-nodule honest-tag posture — the honesty load is carried by the *fallibility*
column, not exotic tags (exactly as `std.text`/`std.io`).**
- **`web.http`** (parse/accessor/construct): `Exact`-**when-Ok** but **fallible** — malformed input
  → explicit **located** `Err`, never sentinel/clamp/partial (the `std.text` idiom + the `std.io`
  `serialize` idiom). No accuracy semantics → floor `Exact` (C2). Effects `none`.
- **`web.json`**: decode `Empirical` (round-trip via `std.io.from_json`, proptest, no theorem →
  not `Proven`, VR-5); encode `Exact`-when-Ok but fallible (non-finite f64 refused, never a silent
  `null` — the `std.io` `to_json` rule). Effects `none` (pure over the byte/value input).
- **`web.route`** (dispatch): `Exact`-when-Ok (no-route → explicit `Err`/404, never a silent wrong
  handler). **Mandatory EXPLAIN (C3 — no black-box routing):** the route table is a reified
  inspectable artifact and a match yields an EXPLAIN record. Effects `none`.
- **`web.server`** (per-request determinism): **`Empirical`-via-differential, NOT `Proven`** — RT2
  sequentialization differential (concurrent observable ≡ deterministic reference; NFR-7), exactly
  as `std.runtime` tags it. Handler purity is **`Declared`** (the type system cannot enforce it —
  the `std.runtime` "Task purity contract" row). Effects `io`.
- **`web.client`** (get/post/request): wire transfer `Exact`-when-Ok/fallible declaring `io`; a
  call that *also* decodes a JSON body composes the `io` transfer with `web.json` decode →
  composite `Empirical` (the `std.io` `read_value` pattern).

*Grounding: RFC-0016 §4.1 C2/C3, §4.5; text.md §4; io.md §4 + `guarantee_matrix.rs`;
`mycelium-std-runtime/src/guarantee_matrix.rs` (`Empirical`/`Declared` rows); RFC-0008 RT2/RT3;
RFC-0014 §4.5.*

**T12.4.5 — The Rust-first build plan + order (ADR-007 + DN-14).** v1 is a genuine Rust crate
**`mycelium-web`** (the kernel-impl distinction), mirroring the 25 `mycelium-std-*` crates that
landed Rust-first under RFC-0016 §4.6 (Enacted). Why Rust-first, not `.myc`-first: DN-14 is decisive
— of 11 surface-language capabilities a non-trivial module needs, **5 gate-fail** (generics M-657,
traits M-659, effects M-660, `wild`/FFI M-661, static guarantee index M-663) and cross-nodule phyla
is partial (M-662); `web` needs all of these, so `.myc`-authoring is blocked. Build order
(deepest-dependency-first): **(1)** `web.http` value model + never-silent parsing; **(2)** `web.json`
over `std.io`; **(3)** `web.client`; **(4)** `web.route`; **(5)** `web.server` on the landed
`mycelium-std-runtime` `Colony`/`Scope`/`Task` (a request-handling `Task`-per-connection joined by
the `Scope` — RT7; a server germinates from a spore — ADR-013/RFC-0008 §4.4). Each nodule ships a
`guarantee_matrix.rs` (rows × {op, tag, fallibility, error set, effects, EXPLAIN}), **asserted in
tests** with the VR-5 guard-tests carried verbatim from `std-io`'s matrix
(`no_op_is_proven_without_a_checked_theorem`, `no_op_is_declared`,
`fallibility_and_error_set_are_consistent`, `io_ops_declare_io_effect`,
`serialize_and_to_json_refuse_non_finite_fallibly`). *Grounding: ADR-007; RFC-0016 §4.6/§4.5; DN-14
§3/§4; the 25 landed std crates; ADR-013; RFC-0008 §4.4.*

**T12.4.6 — Two distinct gates: E7-1 (the `.myc` *authoring* + typed handlers) vs E7-2 (the `.myc`
runtime *surface*).** **E7-1-gated:** self-hosting `web` in Mycelium-lang waits on M-656…M-664 —
generics (M-657) for `Json<T>` / a generic `Handler`; traits + `impl` (M-659) for a `Handler`/
`Service` trait + the in-language C1–C6 contract; effect annotations (M-660) for `io`/`ffi` on
socket ops; the `wild` floor (M-661); `phylum` + cross-nodule (M-662). Typed `Json<T>` handlers are
specifically generics+traits-gated. **E7-2-gated:** the `colony { hypha … }` server surface waits on
M-665 (reserve the 10 runtime words) → M-666 (activate `hypha` + `colony` as L1 constructs). Until
then `colony` is reserved-not-active and `hypha` ratified-not-yet-lexed. The *Rust* runtime already
exists (M-357/M-521), so `web.server` is buildable Rust-first **now**; only the `.myc` surface is
E7-2-pending. *Grounding: issues.yaml E7-1 (M-656…M-664) + E7-2 (M-665…M-668); DN-14 §3/§4; lexicon
status legend.*

**T12.4.7 — Never-silent everywhere is the load-bearing invariant (C1/G2), uniform across all five
nodules.** A malformed HTTP message → `Err(HttpParseError{at, why})`, not a zeroed `Request`; an
unknown method → explicit `Err`, not a silent fallback; a non-finite f64 in a JSON response →
**refused**, never silent `null`; a no-route → explicit 404, never a silent wrong-handler; a short
socket read → `Err(UnexpectedEof{read})`, never a truncated body. *Grounding: RFC-0016 C1; G2;
io.md §C1 + matrix; text.md §C1.*

**T12.4.8 — Effect-annotation surface form is itself gate-failed; examples prose-comment the
effect.** The corpus has two honest spellings: `!{ io }` (RFC-0014 §4.5 spec form, used in the
std-io design sketch) and the proposed stage-1 `/ {io}` (M-660). The annotation *syntax* is
gate-failed (DN-14 §3 row 8 / M-660), so the `.myc` examples below prose-comment the effect rather
than writing it in the signature. *Grounding: RFC-0014 §4.5; DN-14 §3 row 8; issues.yaml M-660.*

### Summary guarantee matrix (house style — spanning key ops; tags grounded per T12.4.4)

| Nodule | Op | Guarantee tag | Fallibility | Error set (explicit, located) | Effects | EXPLAIN |
|---|---|---|---|---|---|---|
| `web.http` | `parse_request` (bytes → Request) | **`Exact`-when-`Ok`** | Fallible | `Err(Malformed{at} \| BadHeader{at} \| BadRequestLine{at} \| BadVersion{at})` | `none` | yes (byte offset) |
| `web.http` | `parse_response` (bytes → Response) | `Exact`-when-`Ok` | Fallible | `Err(Malformed{at} \| BadStatus{code} \| BadHeader{at})` | `none` | yes |
| `web.http` | `status_from_u16` (u16 → Status) | `Exact`-when-`Ok` | Fallible | `Err(OutOfRange{code})` — never a clamp | `none` | yes |
| `web.http` | `header_get` / `method` / `path` | `Exact` | Total | — | `none` | n/a |
| `web.http` | `Request::new` / `with_header` | `Exact`-when-`Ok` | Fallible | `Err(InvalidHeaderName \| InvalidHeaderValue)` | `none` | n/a |
| `web.json` | `encode_body` (Value → JSON bytes) | **`Exact`-when-`Ok`** | Fallible | `Err(OutOfDomain)` — non-finite f64 refused, **never silent `null`** | `none` | n/a (faithful projection) |
| `web.json` | `decode_body` (JSON bytes → Value) | **`Empirical`** (round-trip; proptest, no theorem — VR-5) | Fallible | `Err(Malformed{at} \| UnknownTag{path} \| OutOfDomain{path} \| BudgetExceeded)` @locus | `none` | yes (RFC-0013 @locus) |
| `web.route` | `match_route` (Method×path → Handler) | `Exact`-when-`Ok` | Fallible | `Err(NoRoute)` / explicit 404 — never silent wrong-handler | `none` | **yes (C3-mandatory: which pattern + captures)** |
| `web.route` | `table` (inspect route set) | `Exact` | Total | — | `none` | yes (reified table) |
| `web.client` | `get` / `request` (wire transfer) | `Exact`-when-`Ok` | Fallible | `Err(UnexpectedEof{read} \| Refused{why} \| EffectBudget)` | **`io`** | yes (IoError record) |
| `web.client` | `get_json` (transfer + decode) | **`Empirical`** (composes `io`+round-trip) | Fallible | `Err(HttpError \| IoError \| JsonError)` @locus | **`io`** | yes |
| `web.server` | `Scope`/per-request join | **`Empirical`** (RT2 differential; **not `Proven`**) | Fallible | `Err(ServeError \| TaskPanicked \| EffectBudget)` | **`io`** | yes |
| `web.server` | handler purity contract | **`Declared`** (asserted by caller; type system can't enforce — VR-5) | n/a | — (FLAGGED `Declared`) | — | yes (reified assertion) |
| `web.server` | `serve` (accept loop on a substrate) | `Exact`-when-`Ok` | Fallible | `Err(BindFailed \| Refused \| EffectBudget)` | **`io`** | yes |

### Example `.myc` programs (ratified surface syntax; gated surfaces annotated inline)

> Only ratified-active forms are used (the `// nodule:` header, `nodule path`, `fn f(x:T) -> U =
> expr`, `type T = A | B(X)`, `match`, `let … in …`). Generics / `Json<T>` / effect annotations are
> **E7-1-pending** (Examples A/B use monomorphic types + prose-comment the effect); `colony`/`hypha`
> are **E7-2-pending** (Example C is annotated loudly). **No superseded spellings** (`spawn_hyph`/
> `hyph`/`Sclerotium`/`sclrt`/`rhizo`/`anas`).

```mycelium
// nodule: examples.weather_client
nodule examples.weather_client

// Generic Json<T> is E7-1-pending (generics gate-fail, DN-14 §3 row 6 / M-657);
// v1 carries the decoded body as a core Value.
type Fetch = Ok(Value) | Failed(HttpError)

// NOTE (E7-1-pending): the `io` effect on a real client call is `!{ io }` (RFC-0014
// §4.5) — surface effect-annotation syntax is gate-failed (DN-14 §3 row 8 / M-660).
fn fetch_temp(host: Url) -> Fetch =                 // conceptually: -> Fetch !{ io }
  let resp = web.client.get(host, "/temp") in       // Result: Err is explicit, never a sentinel
  match resp {
    Err(e)   => Failed(e),
    Ok(body) =>
      match web.json.decode_body(body) {            // Empirical round-trip (VR-5);
        Err(_) => Failed(HttpError.BadBody),        //   malformed body -> Err(@locus), never a partial Value
        Ok(v)  => Ok(v),
      },
  }
```

```mycelium
// nodule: examples.echo_handler
nodule examples.echo_handler

// A typed fn(Json<Req>) -> Json<Resp> handler is E7-1-pending (generics + traits,
// DN-14 §3 rows 6–7 / M-657/M-659). v1 handlers map Request -> Response directly.
fn handle(req: Request) -> Response =
  match web.http.method(req) {
    Get  => web.http.ok(web.json.encode_body(web.http.path_value(req))),       // encode refuses non-finite f64 (Err), never silent null
    Post => web.http.created(web.json.encode_body(web.http.body_value(req))),
    _    => web.http.method_not_allowed(),          // unmatched method -> explicit 405 (C1/G2), never a silent wrong-handler
  }

// The route table is an inspectable value (C3 — dispatch is EXPLAIN-able, no black box).
fn routes() -> RouteTable =
  web.route.table([
    web.route.get("/echo", handle),
    web.route.post("/echo", handle),
  ])
```

```mycelium
// nodule: examples.echo_server
nodule examples.echo_server

// ┌──────────────────────────────────────────────────────────────────────────┐
// │ E7-2-PENDING SURFACE.  `colony` is reserved-not-active and `hypha` is       │
// │ ratified-not-yet-lexed (lexicon table; issues M-665/M-666).  INTENDED v1+   │
// │ surface ONLY — it does NOT parse today.  v1 is Rust-first: the server runs  │
// │ on the LANDED mycelium-std-runtime Colony/Scope/Task (M-521/M-357), under   │
// │ the RT2 sequentialization differential (Empirical, NOT Proven). Per-request │
// │ determinism = RFC-0008 RT2; handler purity is Declared (VR-5).              │
// └──────────────────────────────────────────────────────────────────────────┘
fn serve(listener: Substrate{Socket}) -> ServeResult =    // conceptually: !{ io }
  colony {                                                 // RFC-0008 R1 structured scope: cannot exit
                                                           //   before every child hypha completes (RT7 join). E7-2-pending (M-666).
    hypha web.server.handle_conn(listener, examples.echo_handler.routes())   // one hypha per connection (RT1/RT2)
  }
```

---

## 6. Honest-Uncertainty Register (what gates ratification)

> This is the load-bearing output of the Phase-1 pass: the explicit split between what is
> **design-decidable now** (the RFC fixes it, Rust-first, grounded) and what the **follow-up
> deep-research pass must verify** before RFC-0022 can move past Draft. Per the two-phase discipline
> and the honesty rule (G2/VR-5), nothing in the right column is invented into a false-confident
> design choice — each is a FLAGGED open question. The RFC stays **Draft** until the right column is
> discharged.

### 6.1 Design-decidable now (grounded; the RFC fixes these, Rust-first)

| # | Decision | Basis |
|---|---|---|
| D1 | The five-nodule decomposition (`web.http`/`json`/`route`/`server`/`client`) + the acyclic dependency graph | RFC-0016 §4.2/§6 mirror; T12.4.1/T12.4.2 |
| D2 | Ring-2, above-kernel, zero-trusted-code posture (KC-3) | RFC-0016 §4.2/C5; T12.4.3 |
| D3 | The value model (Request/Response/Headers/Status/Method/Url/Body as immutable values) | RFC-0001 + the std specs; T12.1.1–T12.1.6 |
| D4 | The never-silent parse discipline + the error enums (`HttpParseError`/`UrlParseError` with `@locus`) | G2/C1; io.md/text.md house style; T12.1.7/T12.1.8 |
| D5 | `Headers` as a multi-map over `std.collections`; `web.json` delegates to `std.io`'s one canonical JSON (no new codec) | DRY/KC-3; collections.md / io.md §7-Q1; T12.1.2/T12.3.1 |
| D6 | The honest per-op tags (HTTP `Exact`-when-Ok/fallible; JSON decode `Empirical` / encode `Exact`-fallible; server determinism `Empirical`-via-differential not `Proven`; handler purity `Declared`) + the matrix-as-data obligation with VR-5 guard-tests | RFC-0016 §4.1/§4.5; io.md + `guarantee_matrix.rs`; ADR-020 / `std-runtime` matrix; T12.1.12/T12.3.2–3/T12.2.10/T12.4.4 |
| D7 | Routing must be reified/EXPLAIN-able (no opaque heuristic); 404/405 explicit | C3; T12.1.11/T12.4.4 |
| D8 | Server = a `colony` of request `hyphae` (RT1 isolation, RT2 default, RT4 explicit failure, RT7 lifetimes; §4.7 C1–C4 budgets/cancellation/supervision) | RFC-0008 §4.1/§4.7; T12.2.1–T12.2.7 |
| D9 | v1 is Rust-first (`mycelium-web`) on the landed runtime (`mycelium-mlir::runtime` / `std.runtime`), mirroring the 25 std crates; build order http→json→client→route→server | ADR-007; RFC-0016 §4.6; DN-14; T12.2.9/T12.4.5 |
| D10 | HTTP/1.1 (RFC 9110 + 9112) + WHATWG-URL as the v1 protocol target; never-silent everywhere | T12.1.13/T12.4.7 |

### 6.2 Deep-research-must-verify (FLAGGED; gates ratification)

| # | Open question (FLAGGED — not invented) | Why deferred / ties to |
|---|---|---|
| U1 | **Phylum + nodule naming** (`web`/`http`/`json`/…): themed-vs-conventional + the phylum name — a DN-level lexicon call | RFC-0016 §8-Q2; DN-02 three-test gate; append-only sign-off needed |
| U2 | **The socket `wild`/FFI floor** — do the OS socket syscalls live in a `web`-internal nodule or the shared `std-sys` phylum (LR-9 leak-free)? | RFC-0016 §8-Q6; io.md §7-Q4 analogue; security-sensitive |
| U3 | **HTTP/2, HTTP/3** — stream multiplexing, HPACK/QPACK, flow control (RFC 9113/9114): the value model survives, the transport does not | explicit v1 non-goal; a different framing layer |
| U4 | **TLS / HTTPS** — cert validation, trust store, ALPN; bottoms out in an audited `wild`/FFI block or a vetted TLS phylum (`rustls`?) | placement decision (U2) + a `/security-review`-grade pass |
| U5 | **The full header-injection / request-smuggling threat model** — T12.1.9 closes the *known* vectors at parse time; the adversarial pass (lenient-peer interop, `\n` tolerance, proxy-chain precedence) is a follow-up | `/security-review`-grade; G2/VR-5 |
| U6 | **Streaming-body backpressure** — flow control over a streaming `Body` touches `hypha`/colony scheduling (runtime tier, not yet lexed) | undecidable until the R1 scheduler + the E7-2 surface land |
| U7 | **WHATWG-vs-RFC-3986 URL divergences + IDNA/Punycode** — needs a *versioned*, reified, inspectable Unicode/IDNA table (C3), not a hidden default | text.md §7-Q2 grapheme-table analogue |
| U8 | **The `net` effect granularity** — coarse `io` (like `fs`) vs a finer-grained `net`/socket capability (WASI-preopen style) | RFC-0014 / RFC-0016 §8-Q3 co-design; effect-annotation surface is E7-1-gated (M-660) |
| U9 | **Content-negotiation selection** (`Accept`/`q`-value ranking, RFC 9110 §12) — the one *selection* op; stays `Exact` but **must reify which representation it chose and why** (C3) | cross-references `select`/`collections` reification |
| U10 | **The exact body → `Source`/`Sink` seam** — in-memory `Bytes` vs a streaming `Source` drained via `read_value`; the affine handle minting | io.md §7-Q3 analogue; cross-sub-endeavour reconciliation |
| U11 | **The cross-request shared-state mechanism** (cache / connection pool / session store) — `fuse`d value vs `graft`ed capability vs a per-colony owned value; `xloc`/`graft` are R2 (M-668), `fuse` is M-667 | RFC-0008 RT1/RT6; v1 picks a concrete Rust representation + FLAGs the anticipated construct |
| U12 | **Wall-clock request timeouts / deadlines / keep-alive timers** — v0 budgets/supervision use a **logical clock**, not wall-clock; real-time deadlines not yet honestly expressible | RFC-0008 R8-Q3 (time, unresolved §8) |
| U13 | **Graceful-shutdown semantics (precise contract)** — RT7 gives the structural shape; the exact drain-vs-deadline-vs-force policy (and the hard-cancel grace period — a wall-clock question) needs specifying + EXPLAIN-backing | RT7/§4.7 C2; ties to U12 |
| U14 | **Async runtime choice + scheduler fairness/backpressure under load** — the corpus runtime is single-threaded cooperative; whether v1 uses it directly, wraps tokio/async-std behind the `Scope`/`Colony` surface, or awaits a multi-threaded R1 scheduler | RFC-0008 R8-Q1 (scheduler spec, unresolved §8) |
| U15 | **Multi-source `select`/`merge` (RT3) primitive** — needed for any racing/hedging/upstream-fan-in; **deferred / not yet landed** in `mycelium-mlir::runtime`; any such feature is an RT3 construct (reified policy + EXPLAIN) | RFC-0008 RT3; changelog "Deferred" |
| U16 | **Whether any JSON round-trip reaches `Proven`** — inherited from io §7-Q2: tagged at *established* strength; `Proven` only with a checked-side-condition theorem over the closed grammar, else honestly `Empirical` | io.md §7-Q2; not fabricated here (VR-5) |
| U17 | **Typed `Json<T>` handler ergonomics + the `.myc` `colony`/`hypha` server surface** — both gated (E7-1: M-657 generics + M-659 traits; E7-2: M-665 → M-666); design-sketchable, not buildable in `.myc` until those epics land | DN-14; issues.yaml E7-1/E7-2 |

---

## 7. Synthesis verdict

The web-tooling phylum is **design-coherent and Rust-first-buildable now**: a five-nodule `web`
phylum mirroring RFC-0016's `std` structure, never-silent throughout (every parse a located
`Result`, every selection reified/EXPLAIN-able), with honest per-op tags **inherited from the
landed std precedents** (`std.io` JSON tags, `std.runtime` concurrency tags) rather than minted —
so the honesty bar is met *by reuse*, not by new claims (VR-5/KC-3). Two language gates bound the
`.myc` surface: **E7-1** (generics M-657 + traits M-659 → typed `Json<T>` handlers + in-language
authoring) and **E7-2** (M-665 → M-666 → the `colony { hypha … }` server surface); the Rust runtime
substrate (M-357/M-356/M-521) is already landed, so `web.server` is Rust-first-buildable today.

Per the two-phase discipline, **RFC-0022 stays Draft** and the §6.2 Register (17 FLAGGED open
questions — naming, the `wild`/socket floor, HTTP/2-3, TLS, smuggling, backpressure, IDNA, the
`net` effect, content-negotiation, the body↔Source seam, shared state, wall-clock time, graceful
shutdown, the async runtime, multi-source RT3, the `Proven` round-trip, and the E7-1/E7-2 surfaces)
**gates ratification**: the follow-up deep-research pass must discharge them before the RFC moves to
Accepted. No guarantee tag is `Proven` without a checked basis; every claim cites its basis or is
FLAGGED (G2/VR-5).

---

## Meta — changelog

- **2026-06-21 — Record created (Phase-1 design pass for RFC-0022).** Fractured research over four
  sub-endeavours (HTTP protocol + parsing; server concurrency; JSON↔`Value`; phylum surface +
  guarantees + examples + build plan), each a focused max-effort sub-reasoner sharing one
  cross-context packet. Findings **T12.1.x–T12.4.x** ground the Draft RFC-0022; the §6
  Honest-Uncertainty Register (D1–D10 decidable-now; U1–U17 deep-research-must-verify) gates
  ratification. Posture: Empirical/Declared, never `Proven`; design-informing, not a verification
  pass. Append-only.
