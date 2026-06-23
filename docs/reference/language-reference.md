# The Mycelium Language Reference

> **Status: Empirical/Declared** — this reference describes the Mycelium surface as it is actually
> implemented. The **normative oracle** is `docs/spec/grammar/mycelium.ebnf` plus the accept/reject
> conformance corpus under `docs/spec/grammar/conformance/`; the **active keyword set** is
> `crates/mycelium-l1/src/token.rs`. Where this page and a normative source disagree, the source
> wins — file an issue. Every behavioural claim here is grounded in a cited spec, the grammar, the
> conformance corpus, or the interpreter; nothing is invented (G2 / VR-5).
>
> **Audience.** A developer learning Mycelium from the official documentation, without reading
> compiler source. For a hands-on walkthrough that builds a complete program, start with the
> [tutorial](./tutorial.md); use this reference to look up a construct.

Mycelium is a unified value-semantics substrate (binary / ternary / dense / VSA) whose defining
properties are **certified, never-silent representation swaps** and **honest, per-operation
guarantees**. Two house rules shape the entire surface:

- **The honesty rule (VR-5).** Every accuracy or guarantee claim is tagged on the lattice
  `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`. You downgrade to stay honest; you never upgrade without
  a checked basis. The tag is part of a value's observable type.
- **Never-silent (G2).** A representation change is *always* lexically visible (`swap`), an
  out-of-range or unsupported operation is an explicit `Option`/`Result`/diagnostic, and a reserved
  word can never be a silent identifier. No construct erases a refusal.

This reference covers the surface in the order you meet it: [files & nodules](#1-files-nodules-and-phyla),
[the type system](#2-types), [guarantee tags](#3-guarantee-strength-tags),
[functions & effects](#4-functions-and-effects), [expressions](#5-expressions),
[pattern matching](#6-pattern-matching), [the swap system](#7-the-swap-system),
[generics & traits](#8-generics-and-traits), [ambient paradigms](#9-ambient-paradigms),
[maturation](#10-maturation-and-thaw), [`wild` & FFI](#11-wild-and-ffi),
[the keyword set](#12-the-keyword-set), and the [layer model](#13-the-layer-model).

---

## 0. Lexical structure

A Mycelium source file uses the `.myc` extension. Tokens:

| Token | Form | Example |
|---|---|---|
| Identifier | `(Letter \| '_') (Letter \| Digit \| '_')*` | `classify`, `acc_total` |
| Binary literal | `0b` followed by `0`/`1`/`_` | `0b1011_0010` |
| Ternary literal | balanced trits, MSB-first, over `{+, 0, -}` | `<+0--0>` |
| Integer literal | optional `-` then digits | `42`, `-7` |
| List literal | `[` comma-separated exprs `]` | `[1.5, -2.25]` |
| Line comment | `//` to end of line | `// nodule: demo` |

**Comments are trivia** — ignored by the grammar — with one structured exception: the `// nodule:`
marker and the `// @key: value` header lines (below) are recognised by the toolchain even though
they are lexically comments. They are **metadata, never part of content-addressed identity**
(ADR-003).

Integer literals are **representation-typed and universal-until-elaboration**: a bare `42` has no
representation family on its own. It resolves only when an [ambient paradigm](#9-ambient-paradigms)
or an explicit type supplies one; otherwise the checker emits an explicit `UnresolvedWidth`
(RFC-0012 §4.3). There is no silent defaulting across representation families (G2).

---

## 1. Files, nodules, and phyla

### The `// nodule:` header marker

A source file names its nodule on its **first non-blank line**, as a header comment:

```mycelium
// nodule: geometry.shapes
```

or bare, for a subnodule that inherits metadata from its parent/manifest:

```mycelium
// nodule
```

File and directory names stay conventional — `nodule` is never forced into a path.

### Structured `// @key: value` metadata

Optional `@`-prefixed metadata lines follow the marker on a nodule/phylum root. The v0 key set is
closed; an **unknown key is an explicit lint error** (G2 — never silently ignored):

```mycelium
// nodule: ml.inference
// @license: MIT
// @authors: Tyler Zervas
// @since: 2026-01-10
// @updated: 2026-06-16
// @version: 0.3.0
// @matured: true
```

> **Licensing (house rule #6 / ADR-023 §3.4).** First-party `@license` headers and
> `[package].license` fields are **MIT** — the project is MIT-only. (Third-party *dependencies* may
> carry other permissive licenses; that is a distinct axis.)

### The `nodule` construct (active)

The grammar requires a `nodule` declaration to open the program body:

```mycelium
nodule geometry.shapes
```

A **nodule** is the basic static organizational unit (what other languages call a *module*). Its
items — `use`, `type`, `trait`, `impl`, `fn`, and the `default paradigm` ambient — are **unordered
for name resolution**: every top-level `fn` is mutually visible, so a function may call a sibling
declared later in the file. No mutual-recursion keyword exists; the elaborator infers each
strongly-connected component and materialises it as an inspectable `FixGroup` (RFC-0007 §8;
DN-13).

### The `phylum` construct (active — M-662)

A **phylum** is the library-scale grouping above nodules (what other languages call a *package* or
*crate*). An optional `phylum <path>` header groups one-or-more nodules in a single source file:

```mycelium
phylum geometry

nodule geometry.shapes
  // … items …

nodule geometry.transforms
  // … items …
```

Identity stays per-nodule (ADR-003); the phylum header is a metadata grouping. A header-less single
nodule is a *phylum-of-one* (backward-compatible). Cross-nodule names are exported with `pub` and
imported with `use`:

```mycelium
use geometry.shapes.Shape      // import one name
use geometry.shapes.*          // glob: every `pub` name under the prefix
```

Resolution precedence (highest first): a local declaration, then an explicit `use`, then a glob —
higher shadows lower deterministically. A `use` of an absent/private name, a duplicate import, or a
name reachable through two globs is a **never-silent refusal** (G2). The cross-nodule orphan rule
(RFC-0019 §4.5) is enforced phylum-wide.

---

## 2. Types

### Representation types (the four paradigms)

| Type | Form | Meaning |
|---|---|---|
| Binary | `Binary{N}` | `N`-bit binary value |
| Ternary | `Ternary{N}` | `N`-trit balanced-ternary value |
| Dense | `Dense{N, scalar}` | dense embedding of `N` elements; `scalar ∈ {F16, BF16, F32, F64}` |
| VSA | `VSA{model, dim, sparsity}` | hypervector; `sparsity ∈ {Dense, Sparse{N}}` |

```mycelium
fn rotate(p: Dense{2, F32}) -> Dense{2, F32} = p
fn checksum(bs: Binary{8}) -> Binary{8} = bs
```

A **paradigm-less repr** writes the size/shape only and lets the enclosing
[ambient](#9-ambient-paradigms) supply the paradigm: `{8}` (Binary width / Ternary trits),
`{2, F32}` (Dense shape), `{model, dim, sparsity}` (VSA shape). An ill-fitting shape is an explicit
`ParadigmShapeMismatch`; no ambient in scope is an explicit `UnresolvedAmbient`.

### Algebraic data types

A `type` declares a **sum of constructors**, each with optional positional fields:

```mycelium
type Shape = Circle(Binary{8}) | Square(Binary{8}) | Triangle(Binary{8}, Binary{8})
type Bool  = True | False
```

Add `pub` to export the type to sibling nodules of a phylum; absent ⇒ private to its nodule:

```mycelium
pub type Class = Low | High
```

Type parameters give a [generic](#8-generics-and-traits) type: `type List<A> = Nil | Cons(A, List<A>)`.

### Substrate types (affine resources)

`Substrate{Name}` is an **affine external resource** (LR-8) — consumed exactly once. It models a
handle to something outside the value world (a file, a device). Misuse (double-consume) is a
checker refusal, never silent.

---

## 3. Guarantee-strength tags

Any type may carry an honesty tag as a **type-level index** (LR-6), written `T @ Strength`:

```mycelium
fn estimate(x: Dense{4, F32}) -> Binary{8} @ Declared = approx(x)
```

The lattice, strongest to weakest:

| Tag | Meaning | When permitted |
|---|---|---|
| `Exact` | bit-exact / referentially exact | the operation is exact by construction |
| `Proven` | a theorem bounds the error/behaviour | **only** with a theorem whose side-conditions are *checked* |
| `Empirical` | bound holds over measured trials | backed by trials (property tests, benchmarks) |
| `Declared` | asserted by the author, always flagged | the honest default when no stronger basis exists |

You may **downgrade** freely to stay honest. You may **never upgrade** without a checked basis
(VR-5). `Proven` without a checked theorem is forbidden; use `Empirical` (trials) or `Declared`
(asserted). A `Declared` value is always visibly flagged at its binding (invariant S2).

---

## 4. Functions and effects

A function is `fn name(params) -> return_type effects? = body`:

```mycelium
fn area(s: Shape) -> Binary{16} =
  match s {
    Circle(r)      => r,
    Square(w)      => w,
    Triangle(b, h) => b,
  }
```

Parameters are `name: type`. The body is a single [expression](#5-expressions) (Mycelium is
expression-oriented — there are no statements). Add `pub` to export across a phylum. Bodies may
reference any sibling `fn` regardless of order (see [nodules](#1-files-nodules-and-phyla)).

### Effect annotations (M-660; checker-only)

An optional `!{ … }` after the return type declares the function's **effect set**:

```mycelium
fn source() -> Binary{8} !{io} = 0b00000000
fn relay()  -> Binary{8} !{io} = source()
fn wide()   -> Binary{8} !{io, time} = source()
fn pure_one() -> Binary{8} !{} = 0b00000000   // explicit empty set = pure
fn plain()    -> Binary{8} = 0b00000000        // unannotated = pure
```

- An **absent** annotation means **pure** (the empty effect set, RFC-0014 I5); the explicit `!{}`
  is the same pure set.
- Effect names are **plain identifiers** — the closed kernel kinds `retry | alloc | io | cascade |
  time` plus user-declared `Named` effects. They are *not* reserved words.
- A **duplicate** effect in one annotation is a never-silent parse refusal (G2).
- **Coverage rule (checker, not grammar):** a function's *declared* effects must be a **superset**
  of the effects it *performs* (the union of every callee's declared effects). Under-declaration is
  an explicit `CheckError`; over-declaration is allowed (a declaration is a contract).

> **Honesty (VR-5).** Effect annotations are **checker-only metadata** — they do **not** add a
> Core-IR node and do **not** yet wire to the interpreter budget (that is M-677). Their guarantee is
> `Declared`. Do not read this as "effects are enforced at runtime"; they are *type-checked*, not
> *run*.

---

## 5. Expressions

Everything is an expression. The forms:

### `let … in` — local binding

```mycelium
let widened: Binary{8} = sample in
swap(widened, to: Ternary{6}, policy: rt)
```

The type annotation is optional (`let x = e in body`). The binding scopes over the body after `in`.

### `if … then … else` — conditional

```mycelium
if is_zero(x) then Low else High
```

Both branches are required (it is an expression, so it must yield a value).

### `match` — pattern match

See [§6](#6-pattern-matching).

### `for` — bounded iteration (a Total fold)

`for` is the **only** iteration form. It is elaboration-defined sugar for a structurally-recursive
fold over a linearly-recursive data value — **Total by construction** (RFC-0007 §4.8):

```mycelium
fn checksum(bs: Bytes) -> Binary{8} =
  for b in bs, acc = 0b0000_0000 => xor(acc, b)
```

Read it as: fold over `bs`, with accumulator `acc` starting at `0b0000_0000`, combining via
`xor(acc, b)`. There is **no** `while` / `loop` / `break` / `continue` / `return` — unbounded
iteration would undermine the divergence guarantee. Those words are **not reserved**; the toolchain
emits a *teaching diagnostic* pointing you at recursion or `for` if you write them.

### `swap` — never-silent representation change

See [§7](#7-the-swap-system).

### `with paradigm` — block-scope ambient override

See [§9](#9-ambient-paradigms).

### `wild` — denied-by-default FFI escape

See [§11](#11-wild-and-ffi).

### `spore` — reconstruction manifest

`spore(expr)` builds a content-addressed reconstruction manifest for a value (RFC-0003 §6) — the
basis of the deployable `.spore` artifact (ADR-013).

### `colony { hypha … }` — structured concurrency (active — M-666)

A `colony` is the dynamic runtime grouping of `hypha` execution units. Its body is a non-empty list
of `hypha <expr>` spawns:

```mycelium
colony {
  hypha work_a(input),
  hypha work_b(input),
}
```

This is the **deterministic R1 fragment** only (RFC-0008 §4.6): the reference semantics is
spawn-order sequentialization (RT2), so the colony's observable value is its children run in order
(yielding the last `hypha`'s value in the v0 surface). A `hypha` is **only** expressible inside a
`colony` — there is no free `hypha` (an orphan hypha is not expressible, RT7).

### Application & ascription

A function call is `f(arg, …)`; calls chain (`f(x)(y)`). A trailing `: type` ascribes a type to an
expression and binds loosest. Atoms are literals, paths (`a.b.c`), and parenthesised expressions.

---

## 6. Pattern matching

`match` scrutinises a value against arms `pattern => expr`:

```mycelium
match s {
  Circle(r)      => r,
  Square(w)      => w,
  Triangle(b, h) => b,
}
```

Patterns:

- `_` — wildcard (matches anything, binds nothing)
- a literal — `0b00000000`, `<+0->`, `42`
- `Ident` — a binder (lowercase-ish convention) or a nullary constructor
- `Ident(p1, …, pn)` — a constructor with sub-patterns

A trailing comma before `}` is allowed. **Exhaustiveness and redundancy are checked** by the
Maranget usefulness algorithm (`crates/mycelium-l1/src/usefulness.rs`) — a non-exhaustive match or a
redundant arm is an explicit error, never assumed away (G2).

---

## 7. The swap system

A **swap** is the never-silent representation change — Mycelium's signature operation. Both the
target and the policy are **always lexically written** (invariant S1 / WF2):

```mycelium
fn f(x: Binary{8}) -> Ternary{6} =
  swap(x, to: Ternary{6}, policy: rt)
```

`swap(expr, to: <type>, policy: <path>)`:

- `to:` names the **target representation** (a full type ref).
- `policy:` names the **conversion policy** (a path) — the reified, inspectable strategy that
  governs how the conversion is performed and what it guarantees.

No sugar, inference, or elaboration step may **ever** insert a swap (S1). Omitting `policy` is a
parse error — *"a swap is never silent"* (conformance `reject/02-swap-missing-policy.myc`). An
out-of-range conversion is an explicit `Option`/error, never a silent truncation. The selection of
a policy is `EXPLAIN`-able — no black box (KC, ADR-006).

> **`swap` vs `tier`.** A `swap` changes the *representation* (Binary ↔ Ternary, Dense ↔ Sparse). A
> `tier` (reserved-not-active, §12) switches the *execution mode* (interpreted ↔ native). They are
> distinct operations (RFC-0008 §4.5).

---

## 8. Generics and traits

### Generics

Types and functions take type parameters in `<…>`:

```mycelium
type List<A> = Nil | Cons(A, List<A>)

fn map<A, B, E>(r: Result<A, E>, f: A -> B) -> Result<B, E> =
  match r { Ok(x) => Ok(f(x)), Err(e) => Err(e) }
```

Type checking is unification-based (`Ty::Var` + applied `Ty::Data(name, args)`).

### Traits and impls

A `trait` is a named set of function signatures (the type-class / interface concept; the
conventional term is kept). An `impl` provides them for a concrete type:

```mycelium
trait Cmp<A> {
  fn cmp(a: A, b: A) -> Bool
}

impl Cmp<Binary{8}> for Binary{8} {
  fn cmp(a: Binary{8}, b: Binary{8}) -> Bool = eq(a, b)
}
```

**Bounded type parameters** constrain a parameter to traits, combined with `+`:

```mycelium
fn min<T: Cmp>(a: T, b: T) -> T = if cmp(a, b) then a else b
fn pick<T: Cmp + Eq>(a: T, b: T) -> T = a
```

The self-bound `T: Cmp` is sugar for `T: Cmp<T>`. A bound that is not a trait name is an explicit
refusal (G2). **Coherence** is enforced: global uniqueness per `(trait, type-head)` plus the
single-nodule (phylum-wide) orphan rule; method-set conformance is exact.

> **Honesty (VR-5).** Generics and traits **type-check but do not yet run.** Monomorphization and
> dictionary-passing elaboration are **STAGED → M-673**: the checker accepts them and emits an
> explicit `Residual` placeholder rather than executable Core-IR. Do not claim generics or traits
> *execute* — they are *checked*, not *run*, at this language version.

---

## 9. Ambient paradigms

An **ambient** supplies an omitted paradigm to paradigm-less reprs and bare integer literals,
without ever inserting a swap (RFC-0012). Two scopes:

### Nodule-scope ambient — `default paradigm`

At most one per nodule (the outermost ambient frame):

```mycelium
nodule signal

default paradigm Ternary

fn negate(x: {6}) -> {6} = x     // `{6}` adopts the ambient Ternary paradigm
```

`paradigm ∈ {Binary, Ternary, Dense, VSA}`.

### Block-scope ambient — `with paradigm`

A block-scope override; not a conversion — it fills interior tags and elaborates away:

```mycelium
with paradigm Binary { add(0b0001, 0b0010) }
```

A cross-paradigm edge in or out of the block still needs an **explicit swap**; an unbridged edge is a
never-silent `MissingConversion`. Both ambient forms elaborate to exactly the same Core-IR as the
longhand they abbreviate (no behavioural difference, no silent conversion).

---

## 10. Maturation and `thaw`

**Maturation** promotes a scope to AOT-compiled, stable form. It is declared **at scope** — a
nodule/phylum header `// @matured: true` or the `mycelium-proj.toml` manifest — **not** per function
(RFC-0017 §4.1):

```mycelium
// nodule: ml.inference
// @matured: true
nodule ml.inference

fn pipeline(x: Dense{1024, F32}) -> Dense{1024, F32} = x

thaw fn experimental(x: Dense{1024, F32}) -> Dense{1024, F32} = x
```

- `matured` is a **reserved keyword** but opens no term form. A bare `matured fn …` at item position
  is a **parse error** with a teaching diagnostic pointing at the header form.
- `thaw fn …` is the in-source **de-maturation** marker: it keeps one definition interpreted inside
  an otherwise-matured scope (the rare iterate/debug escape hatch, RFC-0017 §4.3). The intuitive
  inverse name `germinate` is taken by spore activation (ADR-013), so `thaw` is the conventional-
  clearest choice.

---

## 11. `wild` and FFI

`wild { expr }` is the **denied-by-default unsafe escape hatch** (LR-9 / S6; ADR-014) — the only
site for trusted/opaque FFI:

```mycelium
// nodule: std.sys @std-sys
nodule std.sys @std-sys

fn read_byte() -> Binary{8} !{ffi} =
  wild { raw_read() }
```

Its **legality is a checker gate**, not a parse rule (M-661):

- A `wild` block may legally appear **only** inside a nodule tagged `@std-sys` (the audited FFI-floor
  context). A `wild` in any other nodule is a hard checker refusal (never silent — G2).
- Inside one, the enclosing `fn` must declare the `ffi` effect (`!{ffi}`) — `wild` is the `ffi`
  *source* for the [coverage checker](#4-functions-and-effects).
- The body is the trusted/opaque FFI escape — **audited, not verified**; it is not recursively
  type-checked (ADR-014 / VR-5), and its execution is staged (an elaboration `Residual`) in v0.

The `myc-sec` wild-audit gate flags every unapproved `wild`.

---

## 12. The keyword set

Verified against `crates/mycelium-l1/src/token.rs` (the lexer's `keyword()` function) — the
source-of-truth for what is reserved **today**. Three statuses:

- **Active** — lexed *and* consumed by a construct.
- **Reserved-not-active** — lexed as a keyword (so it can never be a silent identifier — G2) but no
  production consumes it yet; using it as an identifier is a parse error, but it opens no program.
- **Ratified — not yet lexed** — a name ratified in the lexicon (DN-02/03) but not yet in
  `keyword()`, so it currently lexes as an ordinary identifier.

### Active keywords

| Group | Keywords |
|---|---|
| Static structure | `nodule`, `phylum`, `use`, `pub` |
| Declarations | `type`, `trait`, `impl`, `fn` |
| Maturation | `matured` (reserved keyword; header-declared), `thaw` |
| Binding & control | `let`, `in`, `if`, `then`, `else`, `match`, `for` |
| Representation | `swap`, `to`, `policy` |
| Ambient | `default`, `paradigm`, `with` |
| FFI / deploy | `wild`, `spore` |
| Concurrency | `colony`, `hypha` |
| Representation types | `Binary`, `Ternary`, `Dense`, `VSA`, `Substrate`, `Sparse` |
| Scalars | `F16`, `BF16`, `F32`, `F64` |
| Guarantee tags | `Exact`, `Proven`, `Empirical`, `Declared` |

### Reserved-not-active keywords

The nine runtime-vocabulary names lex as keywords but no production consumes them at this language
version — they activate when their runtime-model constructs land (RFC-0008 §4.6):

| Keyword | Future meaning |
|---|---|
| `fuse` | lawful state fusion / CRDT join (RT6) |
| `mesh` | decentralized gossip / pub-sub overlay (RT5) |
| `graft` | capability contract with infrastructure (RT4) |
| `cyst` | durable checkpoint / dormant resumable form (RT2) |
| `xloc` | explicit value movement / trans-locate |
| `forage` | adaptive placement policy (RT3) |
| `backbone` | priority transport path (RT3) |
| `tier` | execution-mode switch (interpreted ↔ native) |
| `reclaim` | runtime-unit reclamation (stale units only, **never memory**) |

### Not reserved (teaching-diagnostic words)

`while`, `loop`, `break`, `continue`, `return` are **not** reserved (unbounded iteration would
undermine the divergence bit) — the toolchain emits a teaching diagnostic pointing at recursion or
`for` where they appear. `consume` and `grow` are *ratified — not yet lexed* (their lexer reservation
lags the spec, M-664); they currently lex as identifiers.

---

## 13. The layer model

Mycelium is defined as a layer cake (RFC-0006 §3); every construct above elaborates downward:

```
L3  Projections / editor surface     ← text syntax + structured projection (co-equal)
L2  Surface term language ("Myc")    ← what this reference describes: ADTs, traits, nodules, recursion
L1  Kernel calculus                  ← small typed core: λ + data + explicit recursion + Repr types
L0  Core IR (frozen, RFC-0001)       ← Const | Var | Let | Op | Swap + Meta/WF1–WF5
```

- **L0** is the small, frozen, trusted base (KC-3, ADR-007); the reference interpreter runs L0, and
  the guarantee tags live here.
- **L1** adds five nodes to L0: `Lam | App | Construct | Match | Fix` (RFC-0007 §3).
- **L2** is **elaboration-defined** — every surface construct has a *specified, inspectable*
  desugaring to L1. There is no independent L2 semantics, and elaboration is never a black box
  (S4 / ADR-006): every L2 → L1 → L0 step is dumpable and diffable.
- **Content-addressed identity** (ADR-003) is over the elaborated L0 term, never the surface
  keyword. A header date-bump does not change identity; a code change does.

The five surface invariants every layer preserves (RFC-0006 §4.1):

| Invariant | Guarantee |
|---|---|
| **S1** | never-silent swap — a representation change is lexically visible at every layer |
| **S2** | honest tags surface — the guarantee lattice is part of every binding's interface |
| **S3** | content-addressed identity — definition identity is the structure hash |
| **S4** | inspectable elaboration — every lowering step is dumpable and diffable |
| **S5** | explicit partiality — out-of-range / illegal / unsupported is an explicit refusal |

---

## See also

- **[Tutorial](./tutorial.md)** — build a complete program from scratch.
- `docs/spec/grammar/mycelium.ebnf` — the normative grammar oracle.
- `docs/spec/grammar/conformance/` — the accept/reject corpus (the parser's ground truth).
- `docs/Glossary.md` — per-term definitions with normative citations.
- `docs/spec/stdlib/` — the standard-library specifications.
- `docs/adr/ADR-023-Stability-and-API-Compatibility-Guarantees.md` — what is stable at 1.0.0.
- `.claude/memory/lang-lexicon-syntax.md` — the lexicon/syntax orientation aid.

---

## Changelog

- **2026-06-23 — Created (M-735).** Initial full-surface language reference for the full-language
  1.0.0 documentation gate (E17-1). Grounded in `mycelium.ebnf`, the accept/reject conformance
  corpus, and `crates/mycelium-l1/src/token.rs`; honest VR-5 notes mark the type-check-but-don't-run
  surface (generics/traits → M-673; effects checker-only → M-677). Guarantee: `Declared`. Append-only.
