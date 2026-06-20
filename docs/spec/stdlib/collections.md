# Spec — `std.collections` (value-semantic persistent collections: sequence / map / set)

| Field | Value |
|---|---|
| **Status** | **Accepted** (2026-06-20, maintainer-ratified per DN-07 — guarantee matrix asserted in tests; open §7/§8 questions are design/scope calls, not contract violations; was *Implemented (Rust-first) — pending ratification* 2026-06-18, Draft/needs-design 2026-06-17) — the Rust-first code landed as `mycelium-std-collections` (M-511, #152, Batch P5-B; guarantee matrix asserted in tests). The Mycelium-lang migration (M-502-gated) remains. |
| **Module / Ring** | `std.collections` · Ring 2 (RFC-0016 §4.2) · Tier B (RFC-0016 §4.4) |
| **Tracks** | `M-511` (#152) — the Phase-5 task this spec delivers |
| **Scope** | The value-semantic collection structures every program needs: an immutable persistent **`Seq`** (indexed sequence / vector-list), a persistent **`Map`** (key→value), and a persistent **`Set`**, with structural-sharing "updates" (an update returns a *new* value, never mutating in place). The module owns the structures and the *non-identity* hashing-for-buckets used to key a `Map`/`Set`. |
| **Boundary** | Out of scope: **identity / content-addressing** hashing — `hash_of_value`/`hash_of_def`/`digest_eq` are `std.content` (M-523), the canonical-digest surface; `collections`' map hashing is a *non-identity* bucketing mechanism, kept distinct (README §5 records this seam). **Iteration / fold combinators** (`map`/`filter`/`fold`/`zip`/…) are `std.iter` (M-526) — `collections` provides the structures, `iter` the traversal vocabulary. **Ordering / equality semantics** (the `Ord`/`Eq` a sorted map or `dedup` relies on) defer to `std.cmp` (M-532). A **representation** change is `std.swap` (M-516); element-type conversion is `cmp`/`convert` (M-532). |
| **Depends on** | RFC-0001 (the value model — `Value`/`Repr`/`Meta`, the guarantee lattice §4.3, content-addressing §4.6); ADR-003 (content-addressed identity; **metadata is not identity**); RFC-0016 §4.1 (the C1–C6 contract), §4.4 (the `collections` row — "value-semantic by default; a rehash/rebalance is not a silent reorder of observable results"), §4.5 (the guarantee matrix). |
| **Grounds on** | Ring 0 `std.core` (M-515) `Option`/`Result`/error values + the lattice tags; `std.content` (M-523) for *identity* (which `collections` does **not** re-implement); the RFC-0001 value model these structures *are* values within. KC-3: above the kernel — adds no trusted code. |

---

## 1. Summary

`std.collections` is the ordinary collection surface — a persistent indexed `Seq`, a persistent `Map`, a
persistent `Set` — made immutable and value-semantic, held to the §4.1 contract. Every "update" (`push`,
`insert`, `remove`, `with`) returns a **new** value with structural sharing; the input is never mutated.
Its **honesty crux** is twofold and both halves are structural, not advisory. First, **no silent reorder**:
iteration order is *deterministic and documented* for every structure — a `Map`/`Set` iterates in a
specified, stable order (insertion order or a `cmp`-sorted order, fixed per type), **never** an exposed,
build-dependent hash-bucket order that could silently change across a rehash/rebalance (RFC-0016 §4.4 —
"a rehash/rebalance is not a silent reorder of observable results"). Second, **out-of-bounds / missing-key
is explicit**: indexed access and key lookup return `Option`/`Result`, never a silent default, clamp, or
sentinel (C1/G2). The module owns *non-identity* hashing-for-buckets and keeps it strictly distinct from
`content`'s canonical content-addressing (ADR-003). It is Ring 2, Tier B, and adds no trusted code (KC-3):
the structures are library values over the Ring-0 value model.

## 2. Scope & module boundary

- **In scope:**
  - The three persistent structures and their value-semantic surface: **`Seq<E>`** (immutable indexed
    sequence — `len`/`get`/`push`/`pop`/`update`/`concat`/`slice`), **`Map<K,V>`** (immutable key→value —
    `get`/`insert`/`remove`/`contains_key`/`keys`/`values`), **`Set<E>`** (immutable membership —
    `contains`/`insert`/`remove`/`union`/`intersection`/`difference`). Every mutator returns a *new*
    structure with structural sharing.
  - **Non-identity hashing-for-buckets**: the `Hash`/`Hasher` mechanism a `Map`/`Set` uses to bucket keys.
    This is a *bucketing* hash (it may be seeded, may be a non-cryptographic mixer); it is **not** content
    identity. Whether the unordered-map variant is hash-bucketed or tree-ordered is a design choice fixed
    per type (see §3 / §7-Q1); either way its *iteration order* is documented and stable, never an exposed
    bucket order (the honesty crux).
  - **Deterministic, documented iteration order** as a first-class promise of each structure (the input to
    `iter`'s combinators).
- **Out of scope (and who owns it):**
  - **Identity / content-addressing hashing** — `hash_of_value`/`hash_of_def`/`digest_eq` and the typed
    `ContentHash`: that is `std.content` (M-523), the ADR-003 canonical-identity surface. `collections`'
    map hash is a *non-identity* bucketing hash and must stay distinct (RFC-0016 §4.3 names `content` as
    "distinct from hashing-for-maps"; README §5 records the seam). `collections` *consumes* `content` only
    for the value-identity of whole collections (C4), never to bucket keys.
  - **Iteration / fold combinators** — `map`/`filter`/`fold`/`scan`/`zip`/`take`/… are `std.iter` (M-526).
    `collections` exposes each structure as a finite, linearly-walkable `Foldable` (RFC-0007 §4.8 shape) in
    its **documented order** and stops there; `iter` owns the traversal vocabulary over it.
  - **Ordering / equality semantics** — the `Ord`/`Eq` a *sorted* `Map`/`Set` or a `dedup`/`sort` relies on
    is `std.cmp` (M-532). `collections` *parameterizes over* a `cmp` ordering for its ordered variants; it
    does not define comparison.
  - **Representation change** (`Repr` swap, dense↔packed) — `std.swap` (M-516); **element-type conversion**
    (lossy/typed narrowing of `E`) — `cmp`/`convert` (M-532). A `collections` op never silently swaps a
    representation or narrows an element type.
- **Ring & layering:** Ring 2 (RFC-0016 §4.2). `collections` **builds new** library structures written to
  the contract over Ring 0 `core` (the value model, `Option`/`Result`) and — only for whole-collection
  *identity* — Ring 1 `content`. It adds **no** trusted code (KC-3): persistence/structural-sharing is an
  ordinary value-level data-structure implementation, not a kernel extension; no `wild`/FFI (ADR-014).

## 3. Exported-op surface (design sketch)

A design sketch — enough to fix the surface and feed the guarantee matrix, not a committed grammar. Every
structure is an **immutable value**; every mutator returns a *new* value (structural sharing is an
implementation property, invisible to identity). Fallible access returns `Option`/`Result` — never a
sentinel. Iteration order is a **documented property of each type**, surfaced as a `Foldable` for `iter`.

```
// illustrative signatures (not a committed surface)

// --- Seq<E> : immutable indexed sequence (persistent vector/list) ---
opaque type Seq<E>
len     (s: &Seq<E>)                 -> Nat                         // Exact, total
is_empty(s: &Seq<E>)                 -> Bool                        // Exact, total
get     (s: &Seq<E>, i: Nat)         -> Option<&E>                  // None when i >= len (C1, never a default)
first   (s: &Seq<E>)                 -> Option<&E>                  // None on empty
push    (s: &Seq<E>, x: E)           -> Seq<E>                      // returns a NEW Seq (value-semantic)
pop     (s: &Seq<E>)                 -> Option<(Seq<E>, E)>         // None on empty, never a silent no-op
update  (s: &Seq<E>, i: Nat, x: E)   -> Result<Seq<E>, IndexOOB>    // Err(IndexOOB) when i >= len (C1)
concat  (a: &Seq<E>, b: &Seq<E>)     -> Seq<E>                      // order = a then b (documented)
slice   (s: &Seq<E>, lo: Nat, hi: Nat) -> Result<Seq<E>, IndexOOB> // Err on lo>hi or hi>len (no silent clamp)
foldable(s: &Seq<E>)                 -> Foldable<E>                 // in index order (for std.iter)

// --- Map<K,V> : immutable key->value (insertion-ordered OR cmp-sorted, fixed per type — §7-Q1) ---
opaque type Map<K, V>
get        (m: &Map<K,V>, k: &K)        -> Option<&V>               // None on missing key (C1, never a default)
contains_key(m: &Map<K,V>, k: &K)       -> Bool
insert     (m: &Map<K,V>, k: K, v: V)   -> Map<K,V>                 // NEW Map; replaces on duplicate key
remove     (m: &Map<K,V>, k: &K)        -> (Map<K,V>, Option<V>)    // NEW Map + the removed value (None if absent)
get_or     (m: &Map<K,V>, k: &K, d: &V) -> &V                       // EXPLICIT default — the default is a NAMED arg, not silent
keys       (m: &Map<K,V>)               -> Foldable<&K>             // DOCUMENTED order (the honesty crux), never bucket order
values     (m: &Map<K,V>)               -> Foldable<&V>             // same documented order as keys
entries    (m: &Map<K,V>)               -> Foldable<(&K, &V)>       // same documented order

// --- Set<E> : immutable membership ---
opaque type Set<E>
contains    (s: &Set<E>, x: &E)         -> Bool                     // Exact, total
insert      (s: &Set<E>, x: E)          -> Set<E>                   // NEW Set (idempotent on a present element)
remove      (s: &Set<E>, x: &E)         -> Set<E>                   // NEW Set (no-op-returning-new if absent)
union       (a: &Set<E>, b: &Set<E>)    -> Set<E>                   // deterministic result order (documented)
intersection(a: &Set<E>, b: &Set<E>)    -> Set<E>
difference  (a: &Set<E>, b: &Set<E>)    -> Set<E>
foldable    (s: &Set<E>)                -> Foldable<&E>             // DOCUMENTED order, never bucket order

// --- the non-identity bucketing hash (NOT content identity — see Boundary / §7-Q1) ---
trait Hash                                                          // a bucketing mixer for Map/Set keys; may be seeded
// NB: this is hashing-for-buckets, NOT std.content's hash_of_value (ADR-003). Distinct on purpose.

enum CollErr { IndexOOB }                                          // out-of-bounds / bad slice range
```

Notes on the sketch (each grounded, none committing detail the corpus has not fixed):
- **Every mutator returns a new value** (`push`/`insert`/`remove`/`update` → a fresh `Seq`/`Map`/`Set`).
  Structural sharing makes this cheap but is invisible to *identity* (C4): the returned value is a value,
  not a mutated handle.
- **`get`/`first`/`pop`/`get` (Map) return `Option`**, and `update`/`slice` return `Result<_, IndexOOB>` —
  an out-of-bounds index or a missing key is an explicit absence, **never** a silent default or clamp (C1).
- **A default value is never silent.** `get_or(m, k, d)` exists, but the default `d` is an *explicit named
  argument* — the caller asks for it. Bare `get` returns `None`; there is no hidden zero/empty fallback.
- **Iteration order is documented per type** — `keys`/`values`/`entries`/`foldable` walk in the structure's
  *specified* order (insertion order or a `cmp`-sorted order, fixed per type, §7-Q1), never an exposed,
  build-dependent hash-bucket order. This is the honesty crux made structural.

## 4. Guarantee matrix (the load-bearing deliverable — RFC-0016 §4.5)

Rows = exported ops. Encoded as a checked table (the RFC-0003 §4 template), asserted in tests once code
lands — never prose only. **Every op here is `Exact`**: collection operations carry **no**
accuracy/precision/probability semantics, so the lattice floor `Exact` applies directly (RFC-0016 C2 — "an
op with no accuracy semantics, e.g. `len`, is simply `Exact`"). The load-bearing columns for this module
are therefore **fallibility** (the explicit absence/error set — C1) and the **documented-order** invariant
called out below the table (the never-silent-reorder crux). `none*` in the effects column denotes "no
effect; bounded allocation for the new persistent node is intrinsic and budget-free at this layer" (see §7-Q3).

| Op | Guarantee tag | Fallibility (explicit error set) | Declared effects | EXPLAIN-able? |
|---|---|---|---|---|
| `Seq::len` / `is_empty` | `Exact` | total | none | n/a |
| `Seq::get` / `first` | `Exact` | `None` when index out of range / empty (C1) | none | n/a |
| `Seq::push` | `Exact` | total (returns a new `Seq`) | none* | n/a |
| `Seq::pop` | `Exact` | `None` on empty (C1, never a silent no-op) | none* | n/a |
| `Seq::update` | `Exact` | `Err(IndexOOB)` when `i >= len` (C1) | none* | yes (the refusal record) |
| `Seq::concat` | `Exact` | total (order = a‖b, documented) | none* | n/a |
| `Seq::slice` | `Exact` | `Err(IndexOOB)` on `lo > hi` or `hi > len` (no silent clamp) | none* | yes (the refusal record) |
| `Seq::foldable` | `Exact` | total (index order — documented) | none | n/a |
| `Map::get` | `Exact` | `None` on missing key (C1, never a default) | none | n/a |
| `Map::contains_key` | `Exact` | total | none | n/a |
| `Map::insert` | `Exact` | total (new `Map`; replaces on dup key) | none* | n/a |
| `Map::remove` | `Exact` | total (new `Map` + `Option<V>` removed) | none* | n/a |
| `Map::get_or` | `Exact` | total (default is an EXPLICIT named arg, never silent) | none | yes (the default is reified at the call) |
| `Map::keys` / `values` / `entries` | `Exact` | total | none | yes (the documented order is the inspectable artifact — see below) |
| `Set::contains` | `Exact` | total | none | n/a |
| `Set::insert` / `remove` | `Exact` | total (new `Set`; idempotent) | none* | n/a |
| `Set::union` / `intersection` / `difference` | `Exact` | total (deterministic, documented result order) | none* | yes (result order is documented/inspectable) |
| `Set::foldable` | `Exact` | total (documented order) | none | n/a |

**Tag justification (VR-5 — downgrade rather than overclaim, but here nothing is below `Exact`):**
- **Why every row is `Exact`.** No collection op carries accuracy, precision, or probability semantics:
  `len` counts, `get` retrieves, `insert` builds a new value — each is a deterministic structural fact.
  RFC-0016 C2 makes `Exact` the floor for such ops explicitly ("an op with no accuracy semantics … is
  simply `Exact`"). There is no `Proven`/`Empirical`/`Declared` to defend — and nothing to overclaim
  (VR-5). The *substantive* honesty claims for this module live in the **fallibility** and **order**
  columns, not the tag column.
- **Why the explicit-error rows are still `Exact` (C1, not a weaker tag).** `get`/`first`/`pop`/`Map::get`
  returning `None`, and `update`/`slice` returning `Err(IndexOOB)`, are the never-silent floor: an
  out-of-bounds index or a missing key is an honest *absence*, surfaced as `Option`/`Result`, **never** a
  silent default/clamp/sentinel. Fallibility and the `Exact` tag are orthogonal — the op is exact *and*
  honestly partial-in-result. A failure here is a legible structured diagnostic (RFC-0013), recovered or
  re-propagated (RFC-0014/I1), never swallowed.
- **The documented-order invariant (the honesty crux — never a silent reorder).** `Map::keys`/`values`/
  `entries`, `Set::foldable`/`union`/`…`, and `Seq` traversal each iterate in a **specified, stable,
  documented order** that is a *property of the type*, not of the build. A `Map`/`Set` is **never** allowed
  to expose a raw hash-bucket order that a rehash/rebalance could silently change (RFC-0016 §4.4). Two
  collections with the same observable contents have the same observable iteration sequence regardless of
  insertion history's effect on internal layout — i.e. a rehash is an *internal* operation with **no**
  observable reorder. This is the property the implementing task asserts as a checked test (§4.5): it is a
  **`Declared` invariant of the spec now, to become an `Empirical`/property-checked invariant once code
  lands** — it is *not* tagged `Proven` absent a checked theorem (VR-5; see §7-Q2).
- **`EXPLAIN`-able (C3).** Most ops *select/convert/approximate* nothing and are `n/a`. The ops that make a
  visible decision expose it: a refusal (`update`/`slice` → `Err(IndexOOB)`) carries its diagnostic record;
  `get_or`'s default is reified at the call (not a hidden fallback); and the iteration-order contract is
  itself the inspectable artifact for `keys`/`values`/`entries`/`union`/… — a reader can always see *which*
  documented order a structure yields.

## 5. §4.1 contract conformance (C1–C6)

- **C1 — never-silent (G2).** Indexed access and key lookup are explicit about absence: `Seq::get`/`first`/
  `Map::get` → `None`, `Seq::pop` → `None` on empty, `Seq::update`/`slice` → `Err(IndexOOB)` on a bad
  index/range — **never** a silent default, a clamp-to-bounds, or a sentinel. A default is only ever the
  *explicit* `get_or` named argument. The second never-silent guarantee is **order**: no op exposes a
  build-dependent bucket order; a rehash/rebalance produces **no** observable reorder (RFC-0016 §4.4). Every
  refusal is a structured RFC-0013 diagnostic that propagates (RFC-0014/I1), never a quiet no-op.
- **C2 — honest per-op tag (VR-5).** Every op is `Exact` — the lattice floor — because none carries
  accuracy/precision/probability semantics (RFC-0016 C2). The matrix has nothing to upgrade and nothing to
  overclaim. The order invariant is stated honestly as a spec-`Declared` property to be **property-checked**
  once code lands (§4.5), **not** asserted `Proven` without a checked theorem (§7-Q2).
- **C3 — no black boxes / EXPLAIN (SC-3/G11).** `collections` selects/converts/approximates nothing in the
  numeric sense, so most ops are `n/a` for EXPLAIN — but the *decision-bearing* surfaces are inspectable: an
  out-of-bounds refusal carries its diagnostic record naming the violated bound; `get_or`'s default is a
  reified argument; the documented iteration order is itself the inspectable contract (no hidden reorder).
  The non-identity bucketing `Hash` is an internal mechanism whose *only* observable consequence — iteration
  order — is documented and stable, so it is not an opaque box determining a user-visible outcome.
- **C4 — content-addressed, value-semantic (ADR-003 / RFC-0001).** This is the module's spine. Every
  structure is an **immutable value**; an "update" returns a *new* value with structural sharing (the
  sharing is invisible to identity). A whole collection's *identity* is the content-addressed digest of its
  normalized structure (RFC-0001 §4.6, via `std.content`); **metadata is not identity** (ADR-003) — two
  `Map`s equal up to internal bucket layout / insertion history but equal in observable
  contents-and-order are the **same value**. Crucially, the *bucketing* hash a `Map`/`Set` uses internally
  is **not** that identity hash (ADR-003 distinction; README §5 seam): identity is `content`'s canonical
  digest, bucketing is a private mechanism. Every op is a pure function of its inputs.
- **C5 — above the small kernel (KC-3).** `collections` adds **no** trusted code: a persistent
  `Seq`/`Map`/`Set` with structural sharing is an ordinary value-level data structure over the RFC-0001
  value model and Ring-0 `core`. It consumes `std.content` for whole-collection identity (it does not
  re-implement hashing-for-identity) and `std.cmp` for ordered variants. No `wild`/FFI (ADR-014).
- **C6 — declared, bounded effects (RFC-0014).** Every op is **pure** — `effects: none` across the matrix.
  No IO, no clock, no ambient randomness. The one nuance is **allocation**: building a new persistent node
  on an update allocates, marked `none*` (intrinsic, bounded by the input size, budget-free at this layer).
  If a seeded bucketing hash were to draw a per-process seed (a randomness source), *that* seed acquisition
  would be a declared effect owned by whoever provides the seed — FLAGGED §7-Q4; the default design uses a
  fixed/deterministic seed so `collections` stays pure.

## 6. Grounding

- **Value-semantics + immutability + content-addressed identity:** **RFC-0001** (the `Value`/`Repr`/`Meta`
  model; §4.6 content-addressing — `hash(def) = H(normalize(structure) ‖ types_with_repr ‖ static_contract)`,
  "Not hashed (metadata): … *all dynamic value metadata*"; the determinism that makes identity stable) and
  **ADR-003** (Unison-style content-addressing; "formatting is a *projection*, not a mutation of identity";
  **metadata is not identity** — `docs/Mycelium_Project_Foundation.md` §5, ADR-003). This grounds C4 and the
  identity-vs-bucketing-hash distinction.
- **The `collections` honesty crux (no silent reorder; value-semantic by default):** **RFC-0016 §4.4** (the
  `collections` row — "persistent/immutable Vec/List, Map, Set, Deque, ordered/sorted variants;
  value-semantic by default; a rehash/rebalance is **not** a silent reorder of observable results").
- **The identity-hash ↔ bucketing-hash boundary:** **RFC-0016 §4.3** (`content` row: "distinct from
  hashing-for-maps") and **README §5** (the recorded `content`(M-523) ↔ `collections`(M-511) seam —
  "`content` owns *identity* hashing … `collections` owns *non-identity* hashing-for-maps. Kept distinct.").
- **The per-op contract C1–C6, the ring/tier placement, the guarantee-matrix obligation:** **RFC-0016**
  §4.1 (C1–C6), §4.2 (Ring 2), §4.4 (the `collections` row + Tier B), §4.5 (the matrix as a checked table).
- **Failure semantics (legible, propagating refusals):** **RFC-0013** (structured diagnostics) and
  **RFC-0014 / I1** (recovery / propagation is the floor — a refusal is recovered or re-propagated, never
  silently swallowed). An out-of-bounds / missing-key is the collection-shaped instance.
- **The neighbours this spec aligns with:** `std.iter` (M-526) owns the fold/combinator vocabulary over the
  `Foldable` these structures expose in their documented order; `std.cmp` (M-532) owns the `Ord`/`Eq` the
  ordered variants parameterize over; `std.swap` (M-516) owns representation change. The honesty lattice
  `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`, **VR-5** (honest tags), **G2** (never-silent), **G11** (dual
  projection), **KC-3** (small kernel).

## 7. Open questions (FLAGGED — resolve before ratification)

- **(Q1) The concrete collection set, and the ordered-vs-bucketed `Map`/`Set` choice — M-501's to ratify.**
  RFC-0016 §4.4 lists "persistent/immutable Vec/List, Map, Set, Deque, ordered/sorted variants" as the
  *candidate* surface; this spec fixes a **`Seq`/`Map`/`Set`** core (design-first) and leaves `Deque` and
  the explicit *ordered/sorted* variants as a scope question. Critically, **whether the default `Map`/`Set`
  iterates in insertion order (hash-bucketed internally, order recorded separately) or in a `cmp`-sorted
  order (tree-based)** is unsettled — both satisfy the no-silent-reorder crux, but they pick different
  `cmp` (M-532) dependencies and ergonomics. **FLAGGED:** the concrete structure set + the default ordering
  discipline is **M-501's to ratify** (RFC-0016 §4.4); this spec commits only that *whichever* is chosen,
  iteration order is documented and stable, never an exposed bucket order. Ties to RFC-0016 §8-Q1 (the v0
  module set).
- **(Q2) The no-silent-reorder invariant's tag, once code lands.** The matrix records the documented-order /
  rehash-invisibility property as a spec-`Declared` invariant **to be promoted to a property-checked
  (`Empirical`) test** when the implementing task lands (RFC-0016 §4.5) — *not* `Proven`, since no checked
  theorem is cited here (VR-5). **FLAGGED:** confirm the property-test formulation (e.g. "for all insertion
  permutations yielding equal contents, `entries` yields the same sequence") as the checked obligation. Do
  not read the invariant as `Proven`.
- **(Q3) Whole-collection identity via `content` — confirm the entry point.** C4 relies on a collection's
  value-identity being the content-addressed digest of its normalized structure (RFC-0001 §4.6, via
  `std.content`'s `hash_of_value`). `content.md` §7-Q2 itself FLAGs whether `hash_of_value` of an arbitrary
  composite value is a sanctioned, total exported op. **FLAGGED:** `collections` *consumes* whatever
  `content` (M-523) sanctions for collection identity; it does not define it. Ties to `content` §7-Q2.
- **(Q4) The bucketing-hash seed — pure or a declared randomness effect?** If the `Map`/`Set` bucketing
  `Hash` is *seeded* (a HashDoS mitigation), acquiring the seed is a randomness source — a declared effect
  (C6/RFC-0014) and a `rand` (M-531) / RT3 concern, **not** silently pulled. The default design uses a
  **fixed/deterministic** seed so `collections` stays pure (`effects: none`) and reproducible.
  **FLAGGED:** if a seeded hash is wanted, the seed acquisition is an explicit, declared, named effect —
  never ambient entropy. Ties to RFC-0016 §8 (effects) and `rand` (M-531, RT3).
- **(Q5) Ergonomics vs the contract for absence at the call site.** How explicit must absence be — always
  `Option`/`Result` (as sketched), or a sanctioned `expect`-style op that turns `None` into a *propagating
  diagnostic* (still never a silent default)? This is the library instance of RFC-0016 §8-Q3 (ergonomics vs
  the contract, tension A). **FLAGGED:** default to always-explicit `Option`/`Result`; any `expect` form
  must propagate a legible diagnostic (RFC-0013), never clamp or default silently. Ties to RFC-0016 §8-Q3.

## Meta — changelog

- **2026-06-17 — Draft (needs-design).** Stands up the `std.collections` (M-511, #152; Ring 2, Tier B)
  module spec under RFC-0016 (Draft): the value-semantic persistent **`Seq`/`Map`/`Set`** core, every
  mutator returning a *new* value with structural sharing. Fixes the scope + boundary — owns the structures
  and the *non-identity* hashing-for-buckets, defers *identity* hashing to `content` (M-523; README §5
  seam), the fold/iteration vocabulary to `iter` (M-526), ordering/equality to `cmp` (M-532), and
  representation change to `swap` (M-516). The load-bearing deliverable is the per-op **guarantee matrix**:
  every op `Exact` (no accuracy semantics — RFC-0016 C2), with the substantive honesty in the
  **fallibility** column (out-of-bounds / missing-key → explicit `Option`/`Err(IndexOOB)`, never a silent
  default/clamp/sentinel — C1/G2) and the **documented-order / no-silent-reorder** invariant (a
  rehash/rebalance is never an observable reorder — RFC-0016 §4.4), the latter carried honestly as a
  spec-`Declared` invariant to be property-checked once code lands, **never** `Proven` without a checked
  theorem (VR-5). §4.1 conformance (C1–C6) stated concretely — including the C4 identity-vs-bucketing-hash
  distinction (ADR-003) and the C6 pure/`effects: none` stance (with the seeded-hash seed flagged as a
  would-be declared effect). Grounding traces to RFC-0001 §4.6, ADR-003, RFC-0016 §4.1/§4.4/§4.5, RFC-0013/
  0014, G2/VR-5/KC-3. Five questions FLAGGED (the concrete structure set + ordered-vs-bucketed default —
  M-501's to ratify; the order-invariant's checked tag; whole-collection identity via `content`; the
  bucketing-hash seed effect; absence ergonomics) — the concrete scope is M-501's to fix, never invented
  here. No code; no kernel change (KC-3). Append-only.

- **2026-06-20 — Accepted (maintainer ratification, DN-07).** The maintainer ratified this Rust-first spec: the §4.5 guarantee matrix is asserted in tests, never-silent fallibility and honest per-op tags hold, and the open §7/§8 questions are design/scope calls, not contract violations. No guarantee tag was upgraded without a checked basis (VR-5). Status moves *Implemented (Rust-first) — pending ratification → Accepted*. Append-only; no kernel change (KC-3).