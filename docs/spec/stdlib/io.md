# Spec — `std.io` + `serialize` ((de)serialization & byte/text IO over the content-addressed value model)

| Field | Value |
|---|---|
| **Status** | **Implemented (Rust-first) — pending ratification** (2026-06-18; was Draft/needs-design 2026-06-17) — RFC-0016 is **Accepted**, so the Rust-first code landed as `mycelium-std-io` (M-514, #155, Batch P5-B; guarantee matrix asserted in tests). The maintainer's append-only **ratification** of this spec, and the Mycelium-lang migration (M-502-gated), remain. |
| **Module / Ring** | `std.io` (+ the `serialize` half) · Ring 2 (RFC-0016 §4.2) · Tier B (RFC-0016 §4.4) |
| **Tracks** | `M-514` (#155) — the Phase-5 task this spec delivers (the io half + the serialize half of the M-346 "I/O + serialization" candidate) |
| **Scope** | Two coupled surfaces over the content-addressed value model: **(serialize)** projecting a `Value` to/from a byte/text form — `serialize`/`deserialize` (the self-describing wire form, RFC-0001 §4.8) and the **one canonical JSON** projection; **(io)** moving bytes over an abstract source/sink — `read`/`write` against an affine `substrate` handle consumed exactly once (LR-8). Round-trip is a **checked property**; serialization is a **projection**, not identity (ADR-003 — the content hash stays canonical). |
| **Boundary** | **NOT** the filesystem: paths, handles, directory/permission semantics are `fs` (M-528), which *builds on* this module's byte source/sink + serialization abstraction — `io` owns the abstract `Source`/`Sink` + the codec; `fs` owns paths/handles. **NOT** content-addressed identity: the canonical hash is `content` (M-523); serialization reports/round-trips a value, it never *defines* identity. **NOT** the display projection: the human-readable render is `fmt` (M-533) — and `fmt.to_json` **delegates** to this module's one canonical JSON (the README §5 seam; FLAGGED §7-Q1). A representation change is `swap` (M-516), never a (de)serialize. |
| **Depends on** | RFC-0001 §4.8 ((de)serialization over the value model — the self-describing `[Repr]‖[Meta]‖[payload]` wire form, `deserialize(serialize(v)) ≡ v`); ADR-003 (serialization is a *projection*, not identity; the content hash stays canonical); RFC-0016 §4.1 (the C1–C6 contract), §4.4 (`io`/`serialize` rows); RFC-0013 §4.3 (the diagnostic record — robust + legible failure); RFC-0014 §4.5 (declared, bounded effects — `io` is a declared effect); RFC-0001 (the value model — `Value`/`Repr`/`Meta`, the guarantee lattice §4.3). |
| **Grounds on** | **M-104** (landed, Phase 1: Core IR (de)serialization to the JSON contracts, `docs/planning/phase-1.md`) — the trusted codec this module is the library form of; the `Meta` JSON contracts (M-003/M-104); RFC-0006 LR-8 (the affine `Resource`/`substrate` hook). KC-3: this module is **above** the kernel — it adds no trusted serialization or IO code, it wraps M-104's codec and (where OS facilities are needed) confines them per §7-Q4. |

---

## 1. Summary

`std.io` is two coupled surfaces over Mycelium's content-addressed value model. The **serialize** half
projects a `Value` to and from a byte/text form: the self-describing wire form of RFC-0001 §4.8
(`[Repr descriptor] ‖ [Meta] ‖ [payload]`, schema-travels-with-data, Arrow-grade) and the one **canonical
JSON** projection. The **io** half moves bytes over an abstract `Source`/`Sink` whose underlying resource is
an affine `substrate` handle **consumed exactly once** (LR-8). Its **honesty crux** is twofold and
structural: (1) round-trip — `deserialize(serialize(v)) ≡ v` including `Meta` (RFC-0001 §4.8) — is a
**checked property** tagged at its *established* strength, never pre-claimed; and (2) a truncated, malformed,
or decode-failed input is an **explicit, traceable error** that points at *where* it failed (byte offset /
field path), **never a silent partial read** (C1/G2). And the load-bearing identity stance: serialization is
a **projection, not identity** — the content hash stays canonical (ADR-003), so re-serializing and reading a
value back recovers the *same* content-id. It is Ring 2, Tier B, and adds **no trusted code** (KC-3): it
wraps the landed M-104 codec and consumes the affine-resource hook.

## 2. Scope & module boundary

- **In scope:**
  - **serialize:** `serialize(v)` / `deserialize(bytes)` over the RFC-0001 §4.8 self-describing wire form
    (`Repr`-descriptor ‖ `Meta` ‖ payload), faithfully round-trippable including `Meta`; the **one canonical
    JSON** projection (`to_json`/`from_json` over the M-003/M-104 JSON contracts) that `fmt` delegates to;
    format-tagged entry points (`Wire` binary vs `Json` text) where the projection differs.
  - **io:** the abstract byte `Source`/`Sink` and `read`/`write`/`read_all` over them; a **streaming** read
    that consumes its `substrate` handle exactly once (LR-8) — single-consumption is the structural promise,
    not a runtime hope; the deserialize-from-a-source / serialize-to-a-sink bridge that joins the two halves.
- **Out of scope (and who owns it):**
  - **Filesystem** — paths, open/create, directory walks, permission semantics: `fs` (**M-528**). `fs`
    *builds on* this module's `Source`/`Sink` + codec; it provides the path→substrate binding, this module
    provides the byte-movement + serialization abstraction over whatever substrate `fs` hands it.
  - **Content-addressed identity** — the canonical hash of a value/definition: `content` (**M-523**, ADR-003).
    Serialization is a projection *of* a value; it never defines or re-keys its identity (C4).
  - **The human display projection** — readable text/`debug`: `fmt` (**M-533**). Critically, `fmt.to_json`
    **delegates** to *this* module's canonical JSON (one projection, two entry points — README §5; §7-Q1), so
    the round-trip property is shared, not duplicated.
  - **Representation change** — turning a value into another `Repr` (binary↔ternary, dense↔packed): `swap`
    (**M-516**), which emits a swap certificate. A (de)serialize is never a swap; the wire form *records* the
    existing `Repr` descriptor, it does not change it.
- **Ring & layering:** Ring 2 (general library, RFC-0016 §4.2). The serialize half is **new library code
  written to the §4.1 contract over the landed M-104 codec** — it wraps the trusted (de)serializer, it does
  not re-implement it. The io half wraps the affine `substrate` hook (LR-8). Any OS facility the io half
  bottoms out in is `wild`/FFI (ADR-014) and is **FLAGGED §7-Q4** (it may live in a `std-sys` phylum so pure
  `std` stays leak-free, RFC-0016 §8-Q6 / LR-9). KC-3 is preserved: no new trusted code at this layer.

## 3. Exported-op surface (design sketch)

A design sketch — enough to fix the surface and feed the guarantee matrix, not a committed grammar.
Value-semantic, immutable-by-default. Every fallible op returns `Result` with an **explicit** error set; the
io ops **declare** an `io` effect on the signature (C6, RFC-0014 §4.5); a streaming read **consumes** its
affine `substrate` handle by-move (LR-8 — single-consumption is in the type, not a convention). The
`SerError`/`IoError` records carry an RFC-0013 diagnostic with the failure **locus** (byte offset / field
path) so a failure is legible, never a silent partial value.

```
// illustrative signatures (not a committed surface)

// ---- serialize: Value <-> byte/text projection (RFC-0001 §4.8) ----

enum Format { Wire, Json }            // Wire = self-describing [Repr]‖[Meta]‖[payload]; Json = canonical text

serialize(v: &Value, f: Format) -> Bytes                       // total: every Value has a projection
deserialize(b: &Bytes, f: Format) -> Result<Value, SerError>   // C1: malformed/truncated -> explicit Err(@locus)

// the ONE canonical JSON projection fmt.to_json delegates to (README §5 seam)
to_json(v: &Value)    -> Text                                  // total; == serialize(v, Json)
from_json(t: &Text)   -> Result<Value, SerError>               // == deserialize(t.bytes(), Json)

// ---- io: bytes over an abstract source/sink; the underlying resource is affine (LR-8) ----

// `Source`/`Sink` wrap a `substrate` handle; a streaming read consumes it exactly once.
read_all(src: Source)        -> Result<Bytes, IoError> !{ io }     // consumes src by-move (LR-8)
read(src: Source, n: Budget) -> Result<(Bytes, Source), IoError> !{ io }  // returns the handle to continue
write(snk: Sink, b: &Bytes)  -> Result<Sink,  IoError> !{ io }     // returns the sink to continue

// the bridge that joins the two halves (deserialize straight from a source)
read_value(src: Source, f: Format) -> Result<Value, SerError | IoError> !{ io }

// explicit, traceable error sets (each carries an RFC-0013 diagnostic record; never a sentinel)
enum SerError {
  Truncated      { at: ByteOffset },        // input ended mid-value
  Malformed      { at: ByteOffset, why },   // bytes do not parse to the wire/JSON grammar
  UnknownTag     { path: FieldPath, tag },  // an unrecognized Repr/ctor/Meta tag
  OutOfDomain    { path: FieldPath, why },  // a field decodes but violates a value-model invariant
  BudgetExceeded { kind },                  // a declared decode budget (e.g. enum/depth) overrun (ADR-015)
}
enum IoError {
  UnexpectedEof  { read: ByteCount },       // source/sink closed before the requested bytes
  Refused        { why },                   // the underlying substrate refused the operation
  EffectBudget   { kind },                  // a bounded io/alloc budget overrun (RFC-0014 §4.5)
}
```

> **Note (single-consumption, LR-8).** `read_all` takes its `Source` **by-move** and does not return it: the
> affine handle is consumed exactly once and cannot be re-read. A chunked `read` returns the handle so a
> caller may continue — but the *same* handle value is threaded linearly, never aliased. A double-consume is a
> type error, not a runtime check. (LR-8 is "affine `Resource` for external resources only" — RFC-0006 §Q5.)
>
> **Note (one canonical JSON, README §5 / §7-Q1).** `to_json`/`from_json` here are the canonical JSON
> projection; `fmt.to_json` (M-533) **delegates** to them so the round-trip property is established once. This
> spec *proposes* delegation; the maintainer's sign-off is FLAGGED §7-Q1.

## 4. Guarantee matrix (the load-bearing deliverable — RFC-0016 §4.5)

Rows = exported ops. To be encoded as a checked table (the RFC-0003 §4 template) and asserted in tests once
code lands — never prose only. The **round-trip** — `deserialize(serialize(v, f)) ≡ v` including `Meta`, and
the recovered value carries the **same content-id** (ADR-003 / RFC-0001 §4.6/§4.8) — is the one *checked
property* of this module, asserted as a property test and tagged at its *established* strength (§ justification
below).

| Op | Guarantee tag | Fallibility (explicit error set) | Declared effects | EXPLAIN-able? |
|---|---|---|---|---|
| `serialize` (Value → bytes) | `Exact` | total | `none` | n/a (a faithful full projection; no selection/approx) |
| `deserialize` (bytes → Value) | **`Proven`-if-checked, else `Empirical`** (round-trip property) | `Err(Truncated \| Malformed \| UnknownTag \| OutOfDomain \| BudgetExceeded)` @locus | `none` (pure over the byte input) | **yes** — the failure carries an RFC-0013 diagnostic naming the byte offset / field path |
| `to_json` (canonical JSON) | `Exact` | total | `none` | n/a (the machine view of one canonical form) |
| `from_json` (canonical JSON → Value) | **`Proven`-if-checked, else `Empirical`** (round-trip property) | `Err(Malformed \| UnknownTag \| OutOfDomain \| BudgetExceeded)` @locus | `none` | **yes** — diagnostic @locus |
| `read_all` (Source → bytes) | `Exact` (the bytes read are exactly the bytes delivered) | `Err(UnexpectedEof \| Refused \| EffectBudget)` | **`io`** | **yes** — `IoError` is a traceable diagnostic record |
| `read` (chunked, returns handle) | `Exact` | `Err(UnexpectedEof \| Refused \| EffectBudget)` | **`io`** (+ `alloc(Budget)` for the chunk) | yes |
| `write` (Sink ← bytes) | `Exact` (the bytes written are exactly the bytes given) | `Err(UnexpectedEof \| Refused \| EffectBudget)` | **`io`** | yes |
| `read_value` (Source → Value) | **`Proven`-if-checked, else `Empirical`** (round-trip; composes io + deserialize) | `Err(SerError \| IoError)` @locus | **`io`** | yes — diagnostic @locus |

**Tag justification (VR-5 — downgrade rather than overclaim):**
- **`Exact` rows.** `serialize`/`to_json`/`read_all`/`read`/`write` carry **no** accuracy/precision/probability
  semantics, so the lattice floor `Exact` applies (RFC-0016 C2 — "an op with no accuracy semantics is simply
  `Exact`"). `serialize`/`to_json` are *total faithful projections* — every `Value` has a wire/JSON form
  (RFC-0001 §4.8). The byte-movement ops are `Exact` in the precise sense that **the bytes moved are exactly
  the bytes delivered/given** — they neither approximate nor reorder; their *failure* (a short read, a refusal)
  is an explicit `IoError`, not a silently-truncated success.
- **The round-trip rows (`deserialize`/`from_json`/`read_value`) — the honest crux.** The substantive claim is
  not a numeric bound but the **round-trip invariant** `deserialize(serialize(v)) ≡ v` (RFC-0001 §4.8). That is
  a *checked property*, and its guarantee tag is exactly the **established strength of the property test**
  (C2/VR-5): **`Proven`** only when the round-trip is discharged by a theorem whose side-conditions are checked
  (e.g. a totality/injectivity argument over the closed `Repr`/`Meta` grammar — a candidate, *not asserted
  here*); otherwise honestly **`Empirical`** (the property holds over a generated corpus with a stated method).
  This spec **does not pre-claim `Proven`** — it fixes the *discipline* (the round-trip is the checked
  property, tagged at what is established) and FLAGs the reachable strength to ratification (§7-Q2). The
  recovered value carries the **same content-id** (ADR-003): the round-trip preserves identity, the projection
  is not identity.
- **No silent partial value anywhere (C1/G2).** Every round-trip row whose input may be truncated/malformed
  returns an explicit `Err(...)` **with a locus** (byte offset / field path) instead of a partially-filled
  `Value` or a best-effort coercion. The explicit, located error *is* the never-silent guarantee — a decode
  failure is **legible**, naming *where* it failed (the maintainer's failure-semantics directive; RFC-0013 I1).
- **Declared effects.** The io rows declare **`io`** (a declared effect, RFC-0014 §4.5); chunked `read`
  additionally declares its `alloc(Budget)` for the chunk buffer. The serialize/JSON rows are **pure over the
  byte input** (`none`) — they read bytes and compute; they perform no IO themselves (the io is the *caller's*,
  via a `Source`/`Sink`). `read_value` composes the two and therefore declares `io`.

## 5. §4.1 contract conformance (C1–C6)

- **C1 — never-silent (G2).** This is the module's crux. A truncated input is `Err(Truncated{at})`, malformed
  bytes are `Err(Malformed{at, why})`, an unknown `Repr`/ctor/`Meta` tag is `Err(UnknownTag{path})`, a short
  read is `Err(UnexpectedEof{read})`. **No op returns a partially-filled `Value`, a zeroed field, a clamp, or a
  sentinel** for a malformed/truncated input — a decode that cannot complete is an explicit, *located* error,
  never a silent partial read. A decode budget overrun (ADR-015) is `Err(BudgetExceeded)`, not an OOM or a hang.
- **C2 — honest per-op tag (VR-5).** `Exact` ops (`serialize`/`to_json`/the byte-movement) carry no accuracy
  semantics. The round-trip ops (`deserialize`/`from_json`/`read_value`) tag at the **established strength of
  the round-trip property** — `Proven` *only* with a checked-side-condition theorem, else honestly `Empirical`.
  The spec deliberately **does not assert `Proven`** for the round-trip; it asserts the *checked-property
  discipline* and the downgrade rule (§7-Q2).
- **C3 — no black boxes / EXPLAIN (SC-3/G11).** Every decode/IO failure reifies an inspectable **RFC-0013
  diagnostic record** — the `SerError`/`IoError` carries the failure *locus* (byte offset / field path), the
  violated grammar/invariant, and is EXPLAIN-able: *why* the decode failed and *where*. The serialize ops
  *select* and *approximate* nothing (a faithful projection has no hidden heuristic), so their EXPLAIN is `n/a`
  rather than unmet; the **decode method/budget** (if a future codec selects one, ADR-015) would itself be a
  reified, inspectable policy.
- **C4 — content-addressed, value-semantic (ADR-003).** **Serialization is a PROJECTION, not identity** — the
  load-bearing §-level statement. `serialize`/`to_json` are pure functions of a borrowed `&Value`; they never
  mutate it and never change its content hash (ADR-003 — formatting/serialization is a projection, RFC-0001
  §4.8/§4.6). The round-trip recovers a value with the **same content-id**; the **content hash stays canonical**
  and the wire form is a *view* of it, not a re-keying. `Meta` travels with the payload (RFC-0001 §4.8) but
  metadata is **not** identity — two values equal up to `Meta` share their content hash.
- **C5 — above the small kernel (KC-3).** The serialize half **wraps the landed M-104 codec** (Phase-1 Core IR
  (de)serialization to the JSON contracts) — it adds no trusted serialization code. The io half wraps the
  affine `substrate` hook (LR-8). Any OS facility the io path bottoms out in is `wild`/FFI confined to an
  audited block (ADR-014, LR-9/S6) and inventoried — **FLAGGED §7-Q4**; until that floor is fixed the C5 "no
  `wild` at this layer" claim is narrowed accordingly. The trusted base is unchanged.
- **C6 — declared, bounded effects (RFC-0014).** **IO is a declared effect**: every io op declares `io` on its
  signature (RFC-0014 §4.5 — no undeclared side effect); chunked `read` declares its `alloc(Budget)`. A decode
  declares its **budget** (depth/enumeration ceiling, ADR-015) and an overrun is the explicit
  `BudgetExceeded`/`EffectBudget` value, never a hang or OOM. The serialize/JSON ops are pure (`effects: none`).

## 6. Grounding

- **(De)serialization over the value model + the round-trip** — **RFC-0001 §4.8** (the self-describing wire
  form `[Repr descriptor] ‖ [Meta] ‖ [payload]`, "faithfully round-trippable: `deserialize(serialize(v)) ≡ v`
  including `Meta`", the Arrow schema-travels-with-data analogue; the canonical text dump as a projection that
  "does not affect identity, §4.6"). The landed codec this is the library form of: **M-104** (Phase-1 Core IR
  (de)serialization to the JSON contracts, `docs/planning/phase-1.md`).
- **Serialization is a projection, not identity; the content hash stays canonical** — **ADR-003**
  (content-addressed identity; "formatting is a *projection*, not a mutation of identity") and RFC-0001
  §4.6/§4.8, via §4.1 clause **C4**. Identity itself is owned by `content` (M-523).
- **Never-silent / no silent partial read** — **G2** and §4.1 clause **C1** ("an out-of-range, malformed, or
  unsupported input is an explicit error, never a sentinel, a clamp, or a partial result"). The **robust +
  legible** failure (the diagnostic record with the byte offset / field path; recovered or re-propagated, never
  swallowed) is **RFC-0013 §4.3** (the content-addressed diagnostic record, dual human/JSON projection) and its
  invariant **I1** (a diagnostic is additive over an explicit error and never replaces it).
- **`io` is a declared, bounded effect** — **RFC-0014 §4.5** (declared effects on the signature, no undeclared
  side effect; an overrun yields an explicit `EffectBudgetExhausted`); the decode budget default is **ADR-015**
  (`DEFAULT_ENUM_BUDGET`).
- **Substrate single-consumption** — **LR-8** (RFC-0006 §Q5 — the affine `Resource` kind for external resources
  only; "linearity/affinity for external resources"), the honesty crux RFC-0016 §4.4 names for `io`.
- **Ring / contract / matrix / dual projection** — **RFC-0016** §4.1 (C1–C6), §4.2 (Ring 2), §4.4 (the `io`
  and `serialize` rows), §4.5 (the per-op guarantee matrix over the RFC-0003 §4 template); **G11** (dual
  human/machine projection — the JSON view `fmt` shares); **VR-5** (honest tags), **KC-3** (above the kernel).

## 7. Open questions (FLAGGED — resolve before ratification)

- **(Q1) The one canonical JSON — `fmt.to_json` delegation.** RFC-0016 §4.4 lists "the dual human/machine
  projection" against **both** `fmt` (M-533) and `serialize` (this module); the README §5 seam records the
  overlap. **Proposed:** `serialize` owns *the one canonical JSON projection*, and `fmt.to_json` **delegates**
  to it (one canonical JSON, two entry points), so the round-trip property is established once and not
  duplicated. `fmt.md` §7-Q1 proposes the same delegation from its side. — *Disposition: FLAGGED; proposed
  default is delegation-to-`serialize`. Needs maintainer sign-off; ties to RFC-0016 §8-Q1 (the v0 module set /
  overlap) and §8-Q3 (ergonomics vs the contract).*
- **(Q2) Which round-trip reaches `Proven`, and over which grammar.** The matrix tags `deserialize`/`from_json`
  at the round-trip property's *established* strength — **`Proven` only** with a checked-side-condition theorem
  (e.g. an injectivity/totality argument over the *closed* `Repr`/`Meta`/ctor grammar), else honestly
  `Empirical` (a generated-corpus property test with a stated method). **Whether** the closed grammar admits a
  `Proven` round-trip, or it settles at `Empirical`, is **not fabricated here**. — *Disposition: FLAGGED; the
  strength column is filled when the codec's round-trip proof obligation is discharged (or not). This spec fixes
  the discipline (round-trip is the checked property, tagged at what is established), not the verdict.*
- **(Q3) The `io` ↔ `fs` build-on seam.** This module owns the abstract `Source`/`Sink` + codec; `fs` (M-528)
  *builds on* it, owning paths/handles and binding a path to a `substrate`. The exact `Source`/`Sink`
  constructor surface `fs` consumes (and whether the affine handle is minted by `io` or by `fs` and threaded
  in) is a co-design point with M-528. — *Disposition: FLAGGED; `io` commits to the abstract byte-movement +
  single-consumption (LR-8) surface; the path→substrate binding is `fs`'s. Reconcile the exact constructor when
  M-528 is authored.*
- **(Q4) The `wild`/FFI floor for the io half.** Byte movement over a real OS source/sink ultimately needs OS
  facilities → `wild`/FFI (ADR-014). Whether that floor lives inside this module (narrowing its C5 "no `wild`"
  claim) or in a separate `std-sys` phylum so pure `std` stays leak-free (LR-9) is **not settled here**. —
  *Disposition: FLAGGED; ties to RFC-0016 §8-Q6 (the `wild`/FFI floor / `std-sys` split). The serialize half is
  pure and `wild`-free; the io half inherits whatever §8-Q6 decides.*
- **(Q5) Ergonomics vs the contract for the format/budget at the call site.** How explicit must the `Format`
  argument and the decode `Budget`/policy (ADR-015) be at a call site versus a reified ambient default (the
  RFC-0012 lesson)? The decode method/budget must stay reified and EXPLAIN-able (C3), not a hidden default. —
  *Disposition: FLAGGED; default to required-explicit until the per-ring ergonomics pass. The library instance
  of RFC-0016 §8-Q3 (tension A) for this module.*

## Meta — changelog

- **2026-06-17 — Draft (needs-design).** Stands up `std.io` + the `serialize` half (Ring 2, Tier B; M-514,
  #155) as **two coupled surfaces over the content-addressed value model**: (serialize) the RFC-0001 §4.8
  self-describing wire form + the one canonical JSON projection `fmt` delegates to, and (io) byte movement over
  an abstract `Source`/`Sink` whose underlying resource is an affine `substrate` consumed exactly once (LR-8).
  Honesty crux: (1) **round-trip** `deserialize(serialize(v)) ≡ v` including `Meta` is a **checked property**,
  tagged at its *established* strength — **`Proven` only** with checked side-conditions, else honestly
  `Empirical`, **never pre-claimed** (VR-5); (2) a truncated/malformed/decode-failed input is an **explicit,
  located** error (byte offset / field path, an RFC-0013 diagnostic), **never a silent partial read** (C1/G2);
  (3) **serialization is a PROJECTION, not identity** — the content hash stays canonical, the round-trip
  recovers the same content-id (ADR-003). Load-bearing **guarantee matrix**: eight rows — `serialize`/`to_json`/
  byte-movement `Exact`; the round-trip ops at `Proven`-if-checked-else-`Empirical`; io ops declare the `io`
  effect (+ `alloc(Budget)`), serialize/JSON ops pure. §4.1 conformance C1–C6 stated concretely; boundary fixed
  against `fs` (M-528, *builds on* this surface), `content` (M-523, identity), `fmt` (M-533, delegates JSON),
  `swap` (M-516). Grounding traces to RFC-0001 §4.8, ADR-003, M-104 (landed), RFC-0013 §4.3/I1, RFC-0014 §4.5,
  ADR-015, LR-8, RFC-0016 §4.1/§4.4/§4.5, G2/G11/VR-5/KC-3. Five questions FLAGGED (the canonical-JSON/`fmt`
  delegation → §8-Q1; which round-trip reaches `Proven` → not fabricated; the `io`↔`fs` build-on seam; the
  `wild`/FFI floor → §8-Q6; the format/budget ergonomics → §8-Q3). No code; no kernel change (KC-3). Append-only.
