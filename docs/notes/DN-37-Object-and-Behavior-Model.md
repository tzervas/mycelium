# Design Note DN-37 — Object & Behavior Model (and the Sigil Category Scheme)

| Field | Value |
|---|---|
| **Note** | DN-37 |
| **Status** | **Accepted** (2026-06-25; **ratified by maintainer**) — the **objects-vs-ADTs framing** (ADT/static-dispatch horn), the **composition-ranked inheritance-emulation menu**, and the **Sigil Category Scheme** are ratified as the **design direction**. **Accepted ratifies the direction, not every open question:** §8's build-order, first-class-`class`-vs-implicit, dynamic-dispatch design, field/opaque encapsulation, and the `~`-operator-vs-function questions remain **open** and tracked; the delegation **keyword** is resolved to **`via`** (DN-38 §8.1, 2026-06-25). Prior: **Draft** (2026-06-25; direction capture). Append-only; house rule #3. Enacts no code. |
| **Feeds** | the **inheritance-emulation surface** leg of the value-semantics language — **default methods** + **super-traits** (RFC-0019 §4.3, deferred), a **delegation `~>`** operator, an **`@`-decorator**, and a possible `class`-like desugaring; the **sigil-category scheme** that intersects the `[]`-grammar wave (**DN-31**, **RFC-0030**, **epic #27**); the eventual **dynamic-dispatch** design (`FieldSpec`-function-field trusted-core ADR, RFC-0019 §4.5). Builds on **RFC-0011** (`Construct`/`Match`), **RFC-0019** (traits + coherence + monomorphization), **RFC-0006** (LR-8/LR-9, Q8 acyclicity), **ADR-003** (content-addressed identity), **DN-31** (sigil/grammar home), and **DN-36** (functional-update / FBIP performance). |
| **Date** | June 25, 2026 |
| **Decides** | *Nothing normatively* — advisory + design-direction capture. Records (1) the **objects-vs-ADTs framing** (Cook's duality: Mycelium sits on the **ADT / static-dispatch horn** by construction — native strength = adding *operations* + clean binary methods; constrained axis = adding *open representations*); (2) a **scrupulously honest built-vs-deferred split** (data + traits + coherence + monomorphization = **built `Exact`**; the whole ergonomic-inheritance layer = mostly **deferred/greenfield**); (3) the **inheritance-emulation menu, ranked by composition-alignment**, each item carried at its honest grounding strength + its never-silent caveat; (4) the **Sigil Category Scheme** (one glyph → one layer of discourse, verified non-conflicting against the lexer); and (5) that **yes, the maintainer can design their own object system** — at the surface (L2/L3) + traits + derives, never the kernel (KC-3). |
| **Task** | object/behavior model + sigil scheme (pre-RFC capture); intersects the DN-31/RFC-0030/epic-#27 grammar wave and E19-1 |

> **Posture (transparency rule / VR-5 / G2).** This note **synthesises two sourced research records**
> — `research/23-object-behavior-model-internal-RECORD.md` (repo ground truth: what is built vs
> deferred/greenfield, the complete sigil table) and
> `research/24-object-behavior-model-prior-art-RECORD.md` (external prior art, primary-source-checked) —
> into a design direction. It **enacts nothing**: no RFC/ADR/DN status moves, no normative text changes,
> no code or property test ships. The grounding split is load-bearing and held throughout: the
> **foundation** (sum-of-products `type`, structural content-addressed identity, traits + coherence +
> monomorphization) is **built** (`Exact`); the **inheritance-emulation menu** is mostly **deferred or
> greenfield** — the surveyed prior-art mechanisms are `Empirical`/`Proven`-at-source, but their
> **Mycelium mappings are `Declared`-with-argument** design proposals, not ratified decisions. Every gap
> is named, not buried (G2). **This note also corrects an earlier overstatement**: the ergonomic-
> inheritance layer (super-traits, default bodies, delegation, dynamic dispatch, record-of-closures
> objects) is **mostly future work** — said plainly below.

---

## §1 Purpose & framing — Mycelium is functional *as a byproduct*

Mycelium did not set out to be a functional language. It set out to be a **fast, memory-safe,
value-semantics, honest, small-kernel** language. Functional-ness fell out of those requirements:
immutability (LR-8) + acyclicity (LR-9) + a tiny term kernel (KC-3) + content-addressed identity
(ADR-003) *together* yield a value-oriented, expression-based, statically-dispatched language — the
shape we recognize as "functional" — without that ever being the goal. The maintainer's question is the
natural next one: **how are "objects" rendered here, can a bespoke object system be designed, and how do
we emulate ergonomic inheritance cleanly** (composition preferred — but *some* inheritance is good)?

The right lens is **Cook's duality** (*On Understanding Data Abstraction, Revisited*, OOPSLA 2009),
which demolishes "objects are a kind of ADT." Both *are* data abstraction; they hide differently:

- An **ADT** is a public name + a hidden representation + operations — formally an **existential type**
  (`∃rep. {empty, insert, contains, union, …}`). One hidden representation per implementation; the type
  system lets an operation **inspect more than one value at once** → binary methods (`union`, `Ord` over
  two values) are cheap, and dispatch is **static**.
- An **object** is a value exporting a **procedural interface** — a **record of closures** with
  late binding; Cook: dynamic binding is *"essentially an invocation of a higher-order function"*
  selecting a closure from that record, and an object is self-referential (`µ this`). The
  **autognostic principle**: *"an object can only have detailed knowledge of itself; all other objects
  are abstract"* — which is **why pure objects cannot do efficient binary methods**.

Reynolds (1975) / Wadler (1998, the *Expression Problem*) give the matrix: objects make adding a new
**representation** (row) easy and a new **operation** (column) hard; ADTs invert it.

| | Add new **representation** (row) | Add new **operation** (column) | Binary / inspecting op |
|---|---|---|---|
| **Objects** (records of closures, late binding) | **easy** | **hard** — touch every class | **hard** — autognosis forbids it |
| **ADTs** (closed sum + functions, static dispatch) | **hard** — edit the closed type | **easy** — one new function | **easy** — inspect both reps |

**Mycelium sits on the ADT / static-dispatch horn *by construction*** (sealed value-model `Repr`,
coherent traits, monomorphization). That fixes the whole shape of this note:

- **Native strength:** adding *operations* (a new function, a new trait + impl) and **clean binary
  methods** — exactly the operations pure objects cannot express (autognosis). This is a genuine
  advantage of the value-semantics + coherent-trait premise, not a limitation.
- **Constrained axis:** adding *open representations* (a new case to a closed type, or true open
  recursion) is the dimension Mycelium pays for. When open extensibility in *both* directions is
  genuinely needed, the recommended escape is **object algebras / tagless-final** (Oliveira & Cook,
  ECOOP 2012; Carette/Kiselyov/Shan, JFP 2009) — which land the open axis *inside* coherent static
  dispatch with no kernel growth, where a hand-rolled record-of-closures (§3.7) forfeits coherence and
  monomorphization.

Grounding: framing `Empirical` (primary-source-verified prior art); the "Mycelium is on the ADT horn"
read is **`Declared`-with-argument** (it follows from the built premises cited above).

## §2 Current state — scrupulously honest (built vs deferred/greenfield)

Grounded in `research/23` with file:line re-verified against source. **This corrects any earlier
impression that the OOP-emulation layer already exists** — most of it is future work.

### §2.1 BUILT — the foundation (`Exact`)

- **Data via one keyword `type`.** There is **no** `struct`/`enum`/`class`/`object`/`record` keyword —
  all data surfaces through `type` (`crates/mycelium-l1/src/token.rs:83`). A **single-constructor** decl
  is a record/product; a **multi-constructor** decl is a sum:
  `type Point = Mk(Binary{16}, Binary{16})` vs `type Shape = Circle(..) | Square(..)`
  (`parse.rs:444-468`, `parse_type_decl` + `parse_ctor`; fields are positional, no named fields in v0).
  These lower to two L0 nodes — `Construct` (saturated ctor application, WF6) and a **flat**,
  coverage-checked `Match` (WF7) — `crates/mycelium-core/src/node.rs:70-87` (RFC-0011 **Enacted r3**).
- **Structural, content-addressed equality (names are *not* identity).** Declarations live in a
  content-addressed registry; a `CtorRef` is `#T#i` (decl-hash ‖ index), and identity is over the
  α-normalized *structure* — `Nat ≡ Peano`, constructor order and field `Repr` are identity-bearing,
  names are not (ADR-003; tested: `identity_is_structural_not_nominal`). Runtime `Value` equality is
  structural (`crates/mycelium-core/src/value.rs`). **`Exact`** (mechanically tested).
- **Traits — interface polymorphism, coherent, monomorphized.** `trait T<A> { fn sig }` +
  `impl T<args> for U { fn = body }` + bounded generics `fn f<T: A + B>` (`parse.rs:471-492`;
  `token.rs:85-91`). **Coherence** = orphan rule + global-uniqueness-per-`(trait, type-head)` +
  reject-overlap, **phylum-wide** (M-662); the instance key erases width/shape and a blanket
  `impl … for T` over a bare `Ty::Var` is **refused explicitly, never silently**
  (`checkty.rs:221-240`, `type_head`). Elaboration = **monomorphization + static dispatch**
  (`crates/mycelium-l1/src/mono.rs`) — *not* the literal runtime dictionary. **`Empirical`** (tested
  checker); coherence-result tag **`Declared`-with-argument** (not machine-checked).
- **Nodule-level `pub` encapsulation.** Top-level `fn`/`trait`/`type` are private-to-nodule by default;
  `pub` exports the name phylum-wide (`Vis = Private | Pub`, `ast.rs`; `token.rs:77-82`). **`Empirical`.**

### §2.2 DEFERRED / GREENFIELD — the ergonomic-inheritance layer (mostly *future* work)

The two ergonomic-inheritance levers and everything object-shaped past interface polymorphism are
**absent in v1** — say so plainly:

| Mechanism | Honest status | Evidence |
|---|---|---|
| **Super-traits** (`trait Ord<A>: Eq<A>`) | **Designed, NOT built.** RFC-0019 §4.3 designs the dict layout, but `TraitInfo` carries only `name`/`params`/`sigs` — **no super-trait field** (`checkty.rs:192-204`) and `parse_trait_decl` parses **no `:` super-bound** (`parse.rs:471-487`). Unparsed, unchecked. | `Declared` (greenfield-to-implement; small add) |
| **Default method bodies** | **Deferred.** A trait body is `fn_sig*` — **signatures only** (`parse.rs:475-480`); a `fn_sig` has **no `= body`** (contrast `parse_fn_decl` which requires `Tok::Eq` + body). Every `impl` must supply every method. | `Declared` (greenfield; small add) |
| **Delegation `~>`** | **Greenfield.** No `delegate`/`forward` keyword, no surface construct anywhere (grep-confirmed absent). Composition today = a value-typed `Data` field + hand-written forwarders. | `Declared` (absence verified) |
| **Decorator `@`** wrapping | **Greenfield.** No value-position decorator form (the `@` glyph is taken for guarantees/`@std-sys`, expression-position decorator is unbuilt). | `Declared` |
| **Dynamic dispatch / record-of-closures objects** | **Greenfield.** `FieldSpec` is `Repr \| Data(String)` only — there is **NO function-typed field** (`crates/mycelium-core/src/data.rs:98-104`), so the RFC-0019 §4.5 runtime-dictionary "object = record of method values" form is **not v0-expressible**. Gated on a trusted-core `FieldSpec`-abstract-field ADR. There is no `dyn`/trait-object path — monomorphization is the only dispatch. | `Declared` (normative target) |
| **Field-level / opaque encapsulation** | **Greenfield.** Fields are positional and a `type` is exported as a whole name — encapsulation is *nodule-level* only (export the type + accessor fns, keep helpers private). | `Declared` |
| **Associated types / multi-param traits** | **Deferred → v2.** Stage-1 is single-parameter (`checkty.rs:192-200`; RFC-0019). | `Declared` |

**Bottom line:** the ergonomic-inheritance layer this note designs is **mostly future work** — the only
behavior-sharing primitive that exists *today* is "manual composition": embed a value-typed field of the
"base" `type` and forward methods by hand. Everything sweeter than that is on the menu below, deferred.

### §2.3 …but the premise *pays off*

The constraint is not a tax that buys nothing — the value-semantics premises make the foundation
**firmer than the prior art's**:

- **Content-addressing *forces* coherence** (ADR-003: one structure-hash → one semantics). Mycelium gets
  the typeclass-as-interface ergonomics — retroactive impls, return-type dispatch, inlinable static
  dispatch — **on firmer ground than Haskell**, which enforces coherence *socially* (orphan instances are
  only a warning). Mycelium enforces it *structurally* because the identity model demands it. This is the
  single biggest "value-semantics makes it EASIER" finding (`research/24 §2.4`, Wadler & Blott POPL 1989;
  Kiselyov: *"monomorphization is the dictionary-passing translation followed by specialization"*).
- **Monomorphization is the natural default and a strict win.** Value semantics + content-addressing want
  **flat, canonical, vtable-free, hashable, RC-friendly** layouts — exactly what monomorphization
  produces, and exactly what the dictionary-passing-then-specialize path of RFC-0019 enables. Any
  vtable-in-the-value is a deliberate, EXPLAIN-able exception (§7).

## §3 The inheritance-emulation menu — ranked by composition-alignment

The core design section. The recurring, **never-silent** finding (verified across Rust / Kotlin / Go):
**delegation, embedding, and decoration give forwarding *without late binding* — an inner method that
calls a sibling does NOT dispatch back up into the outer wrapper.** For a never-silent, value-semantics
language this is the **honest** behavior, not a regression: forwarding-based reuse is statically
traceable *precisely because* there is no hidden virtual re-entry. The explicit anti-pattern to avoid is
Rust's `Deref`-as-inheritance (a *silent* fake "is-a"; rust-unofficial patterns: *"does not introduce
subtyping … interacts badly with bounds checking"*) — make delegation **explicit and named**, never an
implicit auto-deref. Each item below: what it is, the Mycelium mapping, its honest status, the caveat.

### §3.1 Default methods — *(pure composition; first-line mechanism)*

A trait method with a body is a default impls inherit unless overridden (Rust Reference: *"If the trait
function defines a body, this definition acts as a default"*; the Book frames it as the inheritance
substitute). **Limit:** defaults attach to a *trait*, not data — they call other trait methods, not
fields/state. **Mapping:** stateless behavior reuse, **zero kernel cost** (ordinary dictionary entries),
fully coherent, statically dispatched, no late-binding surprise. The no-state-in-defaults shape is a
natural fit for value semantics. **Composition-alignment: maximal.** **Status: DEFERRED** (trait bodies
are signatures-only today, §2.2 — a *small* add). Tag `Declared`.

### §3.2 Super-traits — *(interface inheritance only)*

`trait Sub: Super` *requires* `Super` and grants access to its items. **Limit:** a *requirement*
relation — **no implementation is inherited**; it is interface subtyping among traits, not struct
inheritance. **Mapping:** the clean way to say "trait B builds on trait A's contract" with no
implementation-inheritance machinery; pure composition, coherent, no kernel growth. **Composition-
alignment: maximal.** **Status: DESIGNED but DEFERRED** (RFC-0019 §4.3 designs the dict layout;
unparsed/unchecked, §2.2). Tag `Declared`.

### §3.3 Delegation `~>` — *(the headline ergonomic-inheritance operator)*

Kotlin `class C(b: Base) : Iface by b` makes the compiler **generate forwarding methods** to `b`. The
key limit, **verbatim** (Kotlin docs): *"members overridden in this way do not get called from the
members of the delegate object, which can only access its own implementations of the interface
members"* — the delegate has no reference to the wrapper → **no late binding into C**. Go embedding is
the same lesson (Effective Go: *"the receiver of the method is the inner type, not the outer one"*).
**Mapping:** `~>` is **surface sugar that desugars frontend-only to generated forwarding methods**
(the DN-23 / RFC-0024 discipline — no kernel change). It removes forwarder boilerplate while keeping
every forwarded method a concrete, **`EXPLAIN`-able** function; immutability + acyclicity make the
wrapped value a pure, hashable composition that cannot mutate or cyclically reference the wrapper. The
"no virtual re-entry" limit is the **honest** behavior (no fragile-base-class action-at-a-distance).
**Composition-alignment: high.** **Status: GREENFIELD.** Tag `Declared`.

### §3.4 Decorator `@` — *(pure value-to-value behavior wrapping, `g∘f`)*

GoF Decorator: *"attach additional responsibilities … a flexible alternative to subclassing."* A
decorator holds a wrapped component of the same interface, forwards, and adds behavior — functionally
`g ∘ f`. **Limit:** like all forwarding, the wrapped component does not see the decorator. **Mapping:**
an `@`-decorator is `~>` plus added/overridden behavior around the forwarded calls — a value-to-value
transform `T -> T`. Under value semantics a decorator is a **pure wrapping function** (no shared mutable
state, content-addressable, stackable) — the cleanest possible form; **frontend sugar desugaring to a
wrapping function**. Coherent with the `@` = *wrap/annotate* glyph meaning (§5). **Constraint:** scope
`@`-decorators to expression/value position, not header position, to stay clear of `@`-guarantee /
`@std-sys` (§5). **Composition-alignment: high.** **Status: GREENFIELD.** Tag `Declared`.

### §3.5 Embedding — *(`~>` with wildcard promotion)*

Go embedding promotes an embedded field's methods into the outer method set (*"the methods of embedded
types come along for free"*), with the same "receiver stays the inner value, outer is not a subtype"
limit. **Mapping:** embedding = `~>` with an implicit "promote *all* the inner's methods" wildcard rather
than a named interface — the same forwarding desugaring with a `*`. The receiver-stays-inner limit is
*automatic* under value semantics. **Caution:** blanket promotion can pollute the outer interface —
prefer explicit per-interface `~>` for auditability (KISS / SoC). **Composition-alignment: high.**
**Status: GREENFIELD.** Tag `Declared`.

### §3.6 Row polymorphism — *(structural extension; keep niche)*

A record type is a **row** = labeled fields; row variables make a function polymorphic over "the rest"
(`(.l) :: ∀r α. {l :: α | r} → α`), giving structural width subtyping (Leijen 2005, *scoped labels*;
OCaml object/variant rows; PureScript `{ x :: Int | r }`). **Tension:** rows are **structural and
anonymous** — any record of the right shape qualifies — which **gives up the global-uniqueness coherence
guarantee** the trait system exists to provide (one structure → one semantics). **Mapping recommendation:
narrow** — offer structural records only for genuinely structural, contract-free data (config-shaped,
open payloads), **not** as the behavior-reuse mechanism. **Composition-alignment: structural/orthogonal**
— it *sidesteps* rather than composes with the coherent trait model. **Status: DESIGN OPTION (in
tension).** Tag `Declared`.

### §3.7 Record-of-closures / open recursion — *(escape hatch only; lowest alignment)*

True late binding: an object is a record of functions over an explicit `self`; a class is a **generator**
`λself. {…}`; **`fix` ties the knot** `p = fix(MakeGen(…))`; inheritance modifies the generator *before*
the fixpoint so base-class `self`-calls see the override (Cook & Palsberg, IC 1994:
`W ▷ G = λself.(W(self)(G(self))) ⊕ G(self)`; *"self-reference must be changed to refer to the modified
definition"*). **This is dead against Mycelium's premises:**

- **`fix`-over-`self` IS the cyclic back-edge LR-9 excludes** — an immutable acyclic store cannot
  represent the self-record's back-reference without a special boxed/indirection construct. The sharpest
  "value-semantics + acyclic CONSTRAINS the pattern" finding.
- **Forfeits coherence** — hand-rolled records-of-functions have no global uniqueness (anyone can build a
  differently-wrapped generator of the same shape), abandoning the ADR-003 invariant.
- **Forfeits monomorphization** — the target is not statically known, so no devirtualization/inlining.

**Recommendation: do NOT provide open-recursion objects as a surface feature.** For genuine open
extensibility prefer **object algebras / tagless-final** *inside* coherent static dispatch (§1) — they
buy the open axis without abandoning the model. If a raw record-of-closures is ever truly needed, reserve
it as an **EXPLAIN-flagged, `Declared`-tagged escape hatch**, never the default. **Status: GREENFIELD,
discouraged.** Tag `Declared`.

### Ranking (most → least composition-aligned)

1. **Default methods** (§3.1) — stateless reuse, zero kernel cost. *(deferred — small add)*
2. **Super-traits** (§3.2) — interface-only "requires." *(designed, deferred)*
3. **Delegation `~>`** (§3.3) — generated forwarders; the headline operator. *(greenfield)*
4. **Decorator `@`** (§3.4) — pure value-to-value wrapping. *(greenfield)*
5. **Embedding** (§3.5) — `~>` with wildcard promotion. *(greenfield)*
6. **Row polymorphism** (§3.6) — structural, in tension with coherence; niche. *(design option)*
7. **Record-of-closures / open recursion** (§3.7) — true late binding; escape hatch only. *(greenfield)*

> **Not recommended (completeness only):** Scala-style **linearization** keeps virtual `super` across
> mixins — but it is genuine (ordered) *implementation inheritance with virtual re-entry*, the very
> action-at-a-distance the never-silent / acyclic / value-semantics premises reject. Excluded by design.

## §4 The acyclicity constraint — honest about what is enforced

The acyclicity that *defines* the constraint envelope (§3.7) is a value-semantics **consequence**, not
yet a hard checker rule — be precise:

- **Immutability (LR-8 / value-semantics) is the load-bearing fact.** With no aliased mutable state you
  **cannot mutate a field to point back at a parent**, so cyclic *runtime value* graphs are hard to even
  construct (RFC-0006 LR-9 row). The classic OOP graphs that *require* mutable back-references —
  bidirectional parent/child links, the Observer subject→observer back-list, doubly-linked structures —
  **cannot be expressed as value cycles**; they must be re-expressed acyclically (IDs/indices into an
  owning collection, one-directional ownership, a re-derived view).
- **Recursive *types* are fine** — self- and mutual-recursion are explicitly allowed in the data registry
  (a self-occurrence hashes as a cycle placeholder, Unison-style). What is excluded is a cyclic *value*
  (a value that contains itself), which an immutable bottom-up `Construct` cannot build.
- **The explicit forbid/detect mechanism is FLAGGED-OPEN (RFC-0006 Q8):** *"does the language forbid
  value cycles, detect them, or fall back to a tracing pass?"* — listed **open**. So **do NOT claim the
  compiler rejects value cycles today.** It is **safe-by-construction** (you can't reach a value cycle in
  safe code) but **not safe-by-checker yet** (the explicit check is unwritten).

This is exactly why the record-of-closures escape hatch (§3.7) is constrained: `fix`-over-`self` is the
one construct that *would* introduce the back-edge, so it sits outside the safe-by-construction envelope.
Tag: immutability/value-semantics **`Exact`** (structural); the acyclicity-forbidding *mechanism*
**`Declared`** (Q8 open).

## §5 The Sigil Category Scheme

The surface-notation section — settled with the maintainer. The **complete** scheme, verified
non-conflicting against the lexer (`crates/mycelium-l1/src/token.rs` `keyword()` + `lexer.rs` `run()`
dispatch; comments are `//` not `#`, `lexer.rs:393`; every **free** sigil hits the **never-silent**
`unexpected character` error path, `lexer.rs:200-205`). The mnemonic rule: **assign each glyph the
meaning its shape / convention already shouts**, so each sigil names exactly **one layer of discourse**.

| Glyph | Meaning (layer) | Examples | Lexer status |
|---|---|---|---|
| `@` | **wrap / decorate / annotate** (metadata-*about*) | `T @ Proven` (guarantees), `@std-sys`, attributes, decorators (§3.4) | **TAKEN** — `Tok::At` / `Tok::AtStdSys` (`token.rs:178-188`); coherent — the decorator reuse extends, not collides |
| `#` | **identity / content-address** | `#x` (the content-hash of `x`) | **FREE → assign** (`#` is not lexed; only inside `CtorRef::Display` strings) |
| `$` | **splice / interpolate / capture** | `"${e}"` interpolation, macro capture | **FREE → assign** |
| `?` | **optionality / fallibility** | `T?` (optional type), `e?` (propagate) | **FREE → assign** |
| `~` (bare) | **approximation** (≈, value-level *closeness*) | `a ~ b` (value-level approximate-equal) | **FREE → assign** |
| `~>` (digraph) | **delegation** (§3.3) | `C ~> Base` (generated forwarders) | **FREE → assign** (maximal-munch digraph, like `->`/`==`/`!=`) |
| `!` | **effects** | `!{io}` (effect set); `!=` (ne); `!e` (not) | **TAKEN** — `Tok::Bang`/`BangEq` (`token.rs:236-244`) |
| `&` | **conjoin** | `a & b` (bitwise-and); `&&` (logical and) | **TAKEN** — `Tok::Amp`/`AmpAmp` (`token.rs:220-225`); **NOT borrow** |
| `` ` `` (backtick) | **reserved** — quasiquote / raw / metaprogramming | (kept open, on-convention) | **FREE → reserved** (deliberately NOT spent on "approximate") |
| `+ - * / % ^` … | **value computation** | infix arithmetic/bitwise (RFC-0025) | **TAKEN** |

**The layer-principle** (one sigil = one layer of discourse):
`@` = metadata-*about* · `#` = identity · `?` = fallibility · `$` = capture · `~` = approximation /
`~>` = delegation · `!` = effects · `&` = conjoin · operators = compute. No glyph is overloaded across
two layers; each says one thing.

Notes (honest):

- **`~` bare vs `~>` digraph are distinct by maximal munch.** Bare `~` is value-level **approximation**
  — a *value*-closeness operator, **distinct from guarantee strength** (which is `@Empirical`, the `@`
  layer). `~>` is the adjacent digraph for **delegation**. Both currently hit the free-sigil
  never-silent path (`lexer.rs:200-205`), so assigning them is clean lexer capacity.
- **`&` is conjoin, NOT borrow.** Borrow/affinity is sourced from the affine **analysis** (DN-32/DN-33
  static uniqueness), not a surface sigil — so `&` stays the bitwise/logical conjunction glyph with no
  ownership meaning to collide with.
- **Open question (low-stakes):** whether bare `~` is an *operator* or a named function (`approx_eq`)
  is deferred to the operator-syntax work (RFC-0025 / DN-23) — it does not block the scheme.

**Grammar home + a bracket caveat.** Any normative landing of this scheme rides the **DN-31 / RFC-0030 /
epic #27** `[]`-for-type-args wave (the maintainer's decided `<>`→`[]` reallocation: free `<>` to
comparison/shift, move type/size args to `[]`). Because `[]` becomes type-args, **`@`-attribute argument
lists must use `(...)`, not `[...]`** (e.g. `@cert(Proven)`, not `@cert[Proven]`) — flagged so the
attribute grammar lands consistent with the bracket move. Tag: the taken/free split is **`Exact`**
(exhaustive lexer read); the scheme assignments are **`Declared`** (settled-with-maintainer design, not
yet a landed grammar).

## §6 "Can the maintainer design their own object system?" — Yes

**Yes — and it lives at the surface (L2/L3) + traits + derives/macros, NOT the kernel** (KC-3: the
small auditable kernel does not grow for ergonomics). A `class`-like sugar could **desugar** to
`type` (data) + `impl`s (behavior) + `~>`-delegation (reuse), exactly the established
sugar-desugars-to-the-word-kernel discipline used for generics (monomorphization), HOFs
(defunctionalization, RFC-0024), and operators (RFC-0025). The bespoke object system is therefore a
**library + front-end project**, not a kernel change.

The four standing constraints any such design inherits (each a house rule, not a limitation to engineer
around):

1. **Value-semantics.** Objects are **immutable values**; "mutation" is **functional update**
   (return a new value) — and FBIP-cheap when the old value is unique (cross-ref **DN-36** — the
   recursion-aware reuse increment makes functional update imperative-speed in place at `rc==1`).
2. **Acyclic.** No cyclic object graphs — use **IDs / content-addresses**, not mutable back-pointers
   (§4).
3. **Honest / never-silent dispatch.** Every selection/forward is reified and **`EXPLAIN`-able** (G2):
   no silent virtual re-entry, no auto-deref fake "is-a."
4. **Content-addressed equality.** Object equality is **structural** (names are not identity) — two
   objects are equal iff their structure hashes agree (§2.1).

## §7 Performance

**Monomorphization (flat, vtable-free, hashable, RC-friendly) is the natural default and a strict win**
(§2.3): value semantics + content-addressing want canonical flat layouts with no vtable welded into the
value — exactly what monomorphization produces (rustc-dev-guide: direct, **inlinable** calls →
devirtualization, const-prop, DCE; the cost is binary size / compile time, the deliberate tradeoff).

When greenfield **dynamic dispatch** eventually lands (the heterogeneous-immutable-collection driver),
**copy Swift's existential** (*Understanding Swift Performance*, WWDC 2016): a fixed container with a
**3-word inline value buffer** + a **Value Witness Table** (alloc/copy/destroy lifecycle) and a
**Protocol Witness Table** (method dispatch). The payoff: **small immutable values dispatch dynamically
*without* a heap box or refcount** (≤ 3 words live inline; only larger values are boxed). A content-
addressed RC runtime extends the VWT with retain/release/**hash**. The cost to budget — a `dyn`-like
value gives up the canonical flat layout and takes on an RC'd box — so **a box + RC is the deliberate,
EXPLAIN-able exception for heterogeneous collections only**, never the default. Tag: monomorphization
benefit `Empirical` (rustc); the Swift-existential transfer `Declared` (greenfield design target).

## §8 Open questions (deliberation agenda)

1. **First-class `class`/object sugar, or implicit?** A dedicated `class` keyword + desugaring, vs
   "just `type` + traits + a `~>`-delegation helper" (no new keyword). KISS leans implicit; ergonomics
   may want the sugar.
2. **Build order of the emulation menu.** **Default methods (§3.1) + super-traits (§3.2) are the cheap
   first wins** (small adds to existing trait machinery); delegation `~>` / decorator `@` / embedding
   are the next, larger frontend-sugar tier.
3. **Dynamic-dispatch design (when needed).** The Swift-existential template (§7) + the trusted-core
   `FieldSpec`-abstract-function-field ADR that unblocks the §4.5 runtime-dictionary form.
4. **Field-level / opaque encapsulation** (greenfield, §2.2) — sequencing vs the bracket/grammar work.
5. **The `~` operator-vs-function form** (§5) — low-stakes; defers to the operator-syntax work.
6. **Row polymorphism's niche** (§3.6) — is the contract-free-record use case worth a structural-record
   surface at all, or is it out of scope?
7. **Sequencing vs the `[]`-grammar wave (#27) + E19-1** — the sigil scheme lands normatively in that
   wave; the menu is independent and can sequence ahead.

## §9 Guarantee posture (VR-5) + Definition of Done

**Grounding posture (held throughout):**

- **Foundation** (data + traits + coherence + monomorphization) = **`Exact` / built** — §2.1, mechanically
  tested where claimed (structural identity), `Empirical` for the type-checker.
- **The emulation menu + sigil scheme + object sugar** = **`Declared` design proposals.** The surveyed
  prior-art mechanisms are **`Empirical`/`Proven`-at-source** (primary-source-verified in `research/24`);
  the **Mycelium mappings are `Declared`-with-argument** — they cite their basis (the repo RFCs + the
  surveyed mechanisms) but are **not** ratified decisions and **no tag is upgraded past that basis**
  (VR-5). The taken/free lexer split is **`Exact`** (exhaustive read).
- **Acyclicity** = value-semantics consequence **`Exact`**; the explicit forbid/detect *mechanism*
  **`Declared`** (RFC-0006 Q8 open) — the compiler is **not** claimed to reject value cycles today.

**Definition of Done (the gate for Draft → Accepted).** This note is `Accepted` when the maintainer
ratifies: **(a)** the **menu ranking** (§3) as the composition-aligned order of mechanisms; **(b)** the
**Sigil Category Scheme** (§5) as the surface-notation allocation; and **(c)** the **build order** (§8.2 —
default methods + super-traits first). Ratification moves Draft → Accepted (a legal forward step, house
rule #3) and feeds: the trait-extension RFC work (default bodies + super-traits), a delegation/decorator
surface RFC riding DN-31/RFC-0030/epic #27, and the eventual dynamic-dispatch + `FieldSpec` trusted-core
ADR. **Still enacts no code** — the design is the deliverable; the build is the forward epic. Append-only;
VR-5; G2.

---

## Meta — changelog

- **2026-06-27 — §8 open questions ruled by the maintainer in-session (append-only; design — no code).**
  - **Q1 (first-class `class` vs implicit) → IMPLICIT, no `class` keyword.** Objects are emulated with
    `type` + traits + `via`-delegation; a `class` keyword would mislead toward OOP semantics the language
    rejects (mutable self, inheritance, dynamic dispatch) — a T-map failure (DN-02). **Follow-on design task
    (flagged):** find the *right ergonomic sugar* for object-composition that is **clear and transparent
    about what it really is** (trait + static composition over a value), **without** borrowing OOP-`class`
    connotations. The sugar must teach the honest model, not paper an OOP veneer over it.
  - **Q2 (build order) → ratified.** Default methods (§3.1) + super-traits (§3.2) are the cheap first wins
    (small adds to the trait machinery); then delegation `via`/`~>` + decorator `@` + embedding; dynamic
    dispatch last (gated, see Q3).
  - **Q3 (dynamic dispatch) → DEFER but PLAN AHEAD; implement in the *near* future, before complete
    dogfooding (not now).** Monomorphization stays the only dispatch for now, but dynamic dispatch is **not**
    indefinitely deferred: the trusted-core `FieldSpec`-abstract-function-field ADR (RFC-0019 §4.5; KC-3-
    significant — grows the kernel) is to be **designed ahead** and **dug into at the appropriate stage,
    prior to the language's complete self-hosting/dogfooding**. Scheduled, not speculative.
  - **Q4 (encapsulation) → adopt GRANULAR, item-level visibility.** Not strictly nodule-level: a function /
    method / value / variable (and, to be specified, a field) can be made `pub` **individually**, cleanly and
    in alignment with the rest of the language (item-granular `pub`, Rust-precedent). This **supersedes the
    nodule-only `pub` model** as the target. **Follow-on design task (flagged):** the exact granularity (down
    to fields?) + the surface form, folded into the encapsulation/grammar work.
  - **Q6 (row polymorphism) → out of scope** (default; not contested) — in tension with coherence; revisit
    only if a concrete contract-free-record need appears. **Q5 (`~` operator-vs-fn) / Q7 (sequencing)** defer
    to the operator-syntax + `[]`-grammar wave. No guarantee upgraded (VR-5); the menu/sugar stay `Declared`
    until built. The two flagged follow-on design tasks (honest object sugar; granular-`pub` form) are
    pre-implementation design, gated on the grammar wave (DN-31).

- **2026-06-25 — Created (Draft, advisory) — authored.** Synthesises
  `research/23-object-behavior-model-internal-RECORD.md` (repo ground truth) and
  `research/24-object-behavior-model-prior-art-RECORD.md` (external prior art, primary-source-checked)
  into a design direction for the **object & behavior model and the sigil category scheme**. Records:
  **(1)** the **objects-vs-ADTs framing** — Cook's duality (objects = records of closures + late
  binding; ADTs = closed sum + static dispatch); **Mycelium sits on the ADT / static-dispatch horn by
  construction** (native strength = adding *operations* + clean binary methods; constrained axis = adding
  *open representations*); functional-ness is a **byproduct** of the value-semantics/honesty/small-kernel
  requirements. **(2)** A **scrupulously honest built-vs-deferred split** (correcting an earlier
  overstatement): **BUILT `Exact`** = data via one keyword `type` (single-ctor record / multi-ctor sum,
  `Construct`+flat `Match`), structural content-addressed equality (names ≠ identity), traits + impls +
  bounded generics + **coherence** (orphan + global-uniqueness + reject-overlap, phylum-wide) elaborated
  by **monomorphization + static dispatch**, nodule-level `pub`; **DEFERRED/GREENFIELD** = super-traits
  (designed RFC-0019 §4.3, unparsed/unchecked), default method bodies (sigs-only), delegation `~>`,
  decorator `@`, dynamic dispatch / record-of-closures objects (`FieldSpec` is `Repr|Data` only — no
  function field, so §4.5 dict form not v0-expressible), field-level/opaque encapsulation, associated
  types / multi-param traits (v2). The premise **pays off** — content-addressing *forces* coherence
  (firmer than Haskell's social orphan-warning), monomorphization is the vtable-free natural default.
  **(3)** The **inheritance-emulation menu ranked by composition-alignment** — default methods →
  super-traits → delegation `~>` → decorator `@` → embedding → row polymorphism → record-of-closures —
  each with its Mycelium mapping, honest status, and the **universal never-silent caveat** (forwarding
  without late-binding back into the wrapper is the *honest* behavior; avoid Rust's silent `Deref`
  anti-pattern; prefer object algebras / tagless-final for genuine open extensibility). **(4)** The
  **acyclicity constraint, honestly** — LR-8 immutability makes value cycles unbuildable by construction
  (no parent↔child back-pointers / Observer back-refs), recursive *types* are fine, but the explicit
  forbid/detect mechanism is **flagged-open (RFC-0006 Q8)** — **not** claimed as a checker rule today.
  **(5)** The **Sigil Category Scheme** (settled with the maintainer; verified non-conflicting against the
  lexer): `@` = wrap/decorate/annotate (taken/coherent), `#` = identity/content-address (free), `$` =
  splice/capture (free), `?` = fallibility (free), bare `~` = approximation + digraph `~>` = delegation
  (free, maximal-munch), `!` = effects (taken), `&` = conjoin **not borrow** (taken), `` ` `` = reserved
  for quasiquote (free), operators = compute — one glyph = one layer of discourse; grammar home =
  DN-31 / RFC-0030 / epic #27, with `@`-attribute args using `(...)` not `[...]`. **(6)** **Yes, a
  bespoke object system can be designed** — at the surface (L2/L3) + traits + derives, NOT the kernel
  (KC-3); a `class`-like sugar desugars to `type` + `impl`s + `~>`; constraints = value-semantics
  (immutable; mutation = functional update, FBIP-cheap, cross-ref DN-36), acyclic (IDs not back-pointers),
  honest/EXPLAIN-able dispatch, content-addressed equality. **(7)** **Performance** — monomorphization is
  the flat, vtable-free, hashable, RC-friendly default and a strict win; when dynamic dispatch lands
  (greenfield), copy **Swift's existential** (3-word inline buffer + VWT/PWT) so small immutable values
  dispatch without a heap box/RC, with box+RC the deliberate EXPLAIN-able exception for heterogeneous
  collections only. **(8)** Open questions (first-class `class` sugar vs implicit; build order — default
  methods + super-traits first; dynamic-dispatch design; field/opaque encapsulation; the `~`
  operator-vs-function form; row-poly's niche; sequencing vs the `[]`-grammar wave #27 + E19-1).
  **(9)** Guarantee posture — foundation `Exact`; menu + sigil scheme + object sugar = `Declared`
  proposals (prior-art mechanisms `Empirical`/`Proven`-at-source, Mycelium mappings
  `Declared`-with-argument); taken/free split `Exact`; acyclicity-mechanism `Declared` (Q8). DoD = the
  Draft→Accepted gate (maintainer ratifies the menu ranking + the sigil scheme + the build order).
  **Enacts nothing; moves no status; changes no normative text.** CHANGELOG / Doc-Index / issues.yaml /
  docs/api-index owned by the integrating parent. (Append-only; VR-5; G2.)
- **Ratified Draft → Accepted (2026-06-25).** The maintainer ratified the **objects-vs-ADTs framing**,
  the **composition-ranked inheritance-emulation menu**, and the **Sigil Category Scheme** as the
  design direction. The status move accepts the *direction only*: foundation stays `Exact` (built),
  menu + sigils + object sugar stay `Declared` proposals — no guarantee upgraded (VR-5). **§8's build
  order, first-class-`class`-vs-implicit, dynamic-dispatch, encapsulation, and `~`-form questions
  remain open** (Accepted ≠ all questions closed). The **delegation keyword is resolved to `via`**
  (preposition = conduit-not-agent — honest about static by-value forwarding; the prepositional twin
  of the `~>` flow-glyph; matches Kotlin's `by` precedent the design cites; the transport-network
  metaphor; the feature is still *named* "delegation" in prose — recorded in DN-38 §8.1).
