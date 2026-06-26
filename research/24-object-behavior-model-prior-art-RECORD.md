# External Prior-Art Research — Object & Behavior Model for Mycelium

**Research question.** What are the clean, composition-aligned ways to provide object-oriented
ergonomics — encapsulation, polymorphism, and *especially* inheritance-like behavior reuse — in an
immutable, acyclic, content-addressed, value-semantics, **trait-based** language that prefers
composition over inheritance, has a small kernel, and elaborates traits by dictionary-passing /
monomorphization with static dispatch today (dynamic dispatch greenfield)? Surface-ergonomics and
performance tradeoffs, with primary sources, adversarially verified.

**Mycelium premises that shape every recommendation** (grounded in the repo):
- Traits are **LR-2 typeclasses with coherence** (orphan rule + global uniqueness + reject-overlap),
  elaborated via **dictionary-passing translation**, kernel node budget frozen at ten nodes (KC-3)
  — `docs/rfcs/RFC-0019-Traits-and-Parametric-Polymorphism.md` §1, §4.3–4.4.
- **Coherence is non-negotiable because identity is content-addressed** (ADR-003): one structure hash
  must map to one semantics, so two different `impl Ord for Binary{8}` cannot both be valid
  (RFC-0019 §2.2). This is the load-bearing constraint for the whole object model.
- The repo's established extension discipline is **surface sugar that desugars to the word kernel,
  frontend-only, no kernel growth** — used for generics (monomorphization), HOFs (defunctionalization,
  RFC-0024), and operators (RFC-0025). Any `~>` delegation operator or `@` decorator must follow this
  same shape (DN-23 §3; RFC-0025).
- Values are **immutable, acyclic, value-semantics, content-addressed** — which makes flat
  monomorphic layouts the natural default and any vtable-in-the-value a deliberate exception.

Verification posture: every primary quote below was pulled from the actual paper/spec PDF or official
doc (not memory). Secondary/inferred claims are tagged. This mirrors the house transparency rule
(VR-5): claims carry their basis.

---

## 1. Objects-vs-ADTs duality + the expression problem

### 1.1 Cook's duality — objects ≠ ADTs, they are complementary

William Cook, *On Understanding Data Abstraction, Revisited* (OOPSLA 2009), demolishes "objects are a
kind of ADT." His thesis, verbatim *(verified, primary)*: *"Objects and abstract data types are not
the same thing, and neither one is a variation of the other. They are fundamentally different and in
many ways complementary, in that the strengths of one are the weaknesses of the other."* Both *are*
data abstraction; they hide differently:

- **ADT** = *"a public name, a hidden representation, and operations to create, combine, and observe
  values"* — formally an **existential type** (`SetImp = ∃rep. {empty, insert, contains, union, …}`).
  There is **one** hidden representation per implementation, and *"the type system ensures that it is
  sound for the implementation to inspect any set value."* This is exactly what makes **binary /
  inspecting operations** efficient: *"These optimizations depend critically upon … the ability to
  inspect the representation of more than one abstract value at the same time."* Static dispatch.

- **Object** = a value exporting a **procedural interface**; *"object interfaces do not use type
  abstraction … objects use procedural abstraction to hide behavior."* An object is self-referential
  (`µ this`), and "dynamic binding" is *"essentially an invocation of a higher-order function"* —
  selecting a closure from a record of closures. Late binding / open recursion.

**The autognostic principle** (the crux), verbatim *(verified)*: *"An object can only access other
objects through their public interfaces … An autognostic object can only have detailed knowledge of
itself. All other objects are abstract."* Corollary: *"any programming model that allows inspection of
the representation of more than one abstraction at a time is not object-oriented."* This is precisely
why pure objects cannot do efficient binary methods — in `union`, *"the union method in a set object
cannot know the representation of the other set,"* only call its interface.

**The matrix / duality**, verbatim: *"organize the behaviors into a matrix with representations on one
axis and observations/actions on the other … extensibility can be viewed as adding a column or row."*
Cook credits Reynolds (1975): *"abstract data types facilitate adding new operations, while
'procedural data values' (objects) facilitate adding new representations,"* and notes *"Wadler later
gave the problem a catchy name, the 'Expression Problem.'"*

| | Add new **representation** (row) | Add new **operation** (column) | Binary / inspecting op |
|---|---|---|---|
| **Objects** (records of closures, late binding) | **easy** — new impl, no edits | **hard** — touch every existing class | **hard** — autognosis forbids it |
| **ADTs** (closed sum + functions, static dispatch) | **hard** — edit the closed type | **easy** — one new function | **easy** — inspect both reps |

### 1.2 The expression problem (Wadler, 1998)

Wadler's note states it verbatim *(verified, primary)*: *"define a datatype by cases, where one can
add new cases to the datatype and new functions over the datatype, without recompiling existing code,
and while retaining static type safety (e.g., no casts)."* Three load-bearing constraints: extend in
**both** dimensions, **no recompilation**, **static safety**. He gives the identical matrix: *"the rows
are fixed (cases) but it is easy to add new columns (functions)"* in FP; *"the columns are fixed
(methods) but it is easy to add new rows (subclasses)"* in OOP. **This is Cook's duality**: sealed ADTs
= the ADT horn (operations cheap, cases hard); class hierarchies = the object horn (cases cheap,
operations hard).

**Where traits/typeclasses and sealed ADTs each sit.** Cook's own §5.3 analyzes Haskell type classes:
*"Any type can be made an instance … without prearranged knowledge … type classes are flexible and
extensible"* — so a typeclass lets you add an **operation-as-instance** cheaply and retroactively (a
column). But *"A type can only be an instance of a class in one way … Type classes are not
autognostic … do not allow different instances to interoperate"* — new *cases* under a closed function
are hard. A sealed ADT inverts it: a new operation is one pattern-match (and binary ops are easy —
match both args), but a new case forces editing the sealed type and recompiling every match.

**The two canonical EP solutions** (both verified primary):
- **Tagless-final** (Carette, Kiselyov, Shan, *Finally Tagless, Partially Evaluated*, JFP 2009):
  *"encode … abstract syntax using combinator functions rather than data constructors … not in an
  initial algebra but using the coalgebraic structure."* Syntax becomes a typeclass `Symantics`; a new
  *interpretation* is a new **instance** (add operations), a new syntactic case is a new method
  (add cases) — both open, statically typed, no runtime tags. The **object/final** answer.
- **Object Algebras** (Oliveira & Cook, *Extensibility for the Masses*, ECOOP 2012): *"based on object
  algebras … an abstraction closely related to algebraic datatypes and Church encodings,"* needing only
  *"simple generics."* An algebra is a generic abstract factory; new operation = new algebra class, new
  variant = extend the factory signature. The **algebra/factory** answer.

**Mycelium mapping.** Mycelium is squarely on the **ADT / static-dispatch horn** by construction
(sealed value-model `Repr`, coherent traits, monomorphization). That makes *adding operations* its
native strength and *adding open representations* the constrained axis. Two consequences:
- *EASIER here:* coherence + content-addressing make the ADT horn's binary/inspecting operations
  (set union, `Ord` over two `Binary{8}`) clean and statically dispatched — exactly the operations
  pure objects cannot express (autognosis). This is a genuine advantage of the trait+value-semantics
  premise, not a limitation.
- *CONSTRAINED here:* the "add a new representation/case to a closed type without recompiling" axis is
  the one Mycelium pays for. **Object algebras (factory-as-trait) is the recommended escape** when open
  extensibility in *both* directions is genuinely needed — it lands inside coherent generics with no
  kernel growth, where a hand-rolled record-of-closures (§4) would forfeit coherence and static dispatch.

---

## 2. Typeclasses as interfaces

### 2.1 Wadler & Blott (POPL 1989) — the dictionary-passing translation

*How to make ad-hoc polymorphism less ad hoc* is the canonical source, and it itself draws the OOP
contrast then departs from it *(verified, primary)*: *"each object should carry with it a pointer to a
dictionary of appropriate methods. This is exactly the approach used in object-oriented programming …
This suggests that perhaps dictionaries should be passed around independently of objects … This is the
intuition behind type classes."* The crux: **typeclasses separate the dictionary from the value.**

The canonical compilation, verbatim: *"For each class declaration we introduce a new type,
corresponding to an appropriate 'method dictionary' … If the type of a function contains a class, this
is translated into a dictionary that is passed at run-time."* Worked: `class Num a` → record type
`NumD a`; `instance Num Int` → dictionary **value** `numDInt`; `square :: Num a => a -> a` →
`square' :: NumD a -> a -> a`, and *"each application … must be translated to pass in the appropriate
extra parameter"* (`square 3 → square' numDInt 3`). The dictionary is resolved *by the static type at
the use site*, and the paper already foresees specialization away.

### 2.2 Dictionary-passing vs vtables — the deep difference

A **vtable is welded to the value** (the object/class carries it; dispatch reads the receiver's runtime
type). A **dictionary travels alongside** the value, selected at the call site from the *static* type.
HaskellWiki states the contrast precisely *(verified, secondary)*: *"there is a fixed connection
between an object and its vtable … whereas the connection between a value and a dictionary may change
at runtime, depending on the type the value is used at."* Because the dictionary is *not* attached to
the value, the compiler can (a) prove a single instance applies (**coherence**), (b) **inline and
monomorphize** it to a direct call, and (c) dispatch on **return type** or **multiple type parameters**
— none of which a receiver-dispatched vtable can do. Oleg Kiselyov *(secondary, authoritative)*:
*"monomorphization is the dictionary-passing translation followed by specialization … and the inlining
of dictionaries."*

### 2.3 Rust traits, concretely

The Rust Book *(official)*: *"Traits are similar to … interfaces … with some differences."* Generic
`fn f<T: Trait>` monomorphizes — the dictionary is specialized per `T` and inlined away (static
dispatch). `dyn Trait` is the *only* place Rust resembles OOP: the Reference *(verified, primary)*
states a trait object holds *"a pointer to an instance of a type T … [and] a virtual method table …
a pointer to T's implementation,"* and *"calling a method on a trait object results in virtual dispatch
at runtime."* So `dyn` rejoins the vtable to the value; generics keep it separate and static.

### 2.4 Typeclasses vs OOP interfaces — the substantive deltas

- **Retroactive impl.** A typeclass instance can be written for a type you don't own (Rust's orphan
  rule scopes this); OOP interfaces must be declared at the type's definition (pre-extension-methods).
- **Return-type / multi-param dispatch.** `mempty :: a` dispatches on return type — impossible for a
  receiver-only interface (HaskellWiki: *"where does the dictionary come from?"*).
- **Coherence vs per-instance state.** Rust's orphan rule guarantees *global uniqueness* so the solver
  may pick any derivation; OOP trades coherence away for per-instance *state* bundled with methods.

**Adversarial check** *(verified):* the popular "typeclasses are always static, interfaces always
dynamic, dictionary == vtable" framing **overstates** it — `dyn`/existentials give typeclasses real
dynamic dispatch, templates/JIT give OOP static dispatch; and the dictionary↔vtable analogy is useful,
not an identity (the non-fixed value↔dictionary binding is the whole point).

**Mycelium mapping.** Mycelium's RFC-0019 *is* this design, with the content-addressing twist:
coherence is not merely a style choice but **forced by ADR-003** (one hash → one semantics). So
Mycelium gets the typeclass-as-interface ergonomics (retroactive impls, return-type dispatch,
inlinable static dispatch) **for free and on firmer ground than Haskell** — Haskell enforces coherence
socially (orphan instances are only a warning), whereas Mycelium enforces it structurally because the
identity model demands it. **This is the single biggest "value-semantics + content-addressed makes it
EASIER" finding in the report.** Encapsulation = the trait interface + the hidden `Repr`; polymorphism
= trait bounds. The only thing missing from the OOP triad is inheritance — §3.

---

## 3. The inheritance-emulation menu

Each mechanism below is given with its **precise limit** and a **Mycelium-mapping recommendation**,
ranked by composition-alignment. The recurring, load-bearing finding (verified across Rust/Kotlin/Go):
**delegation, embedding, and decoration give forwarding *without late binding* — an inner method that
calls a sibling does NOT dispatch back into the outer wrapper.** Only Scala linearization preserves
virtual `super` across reuse. For a never-silent, value-semantics language this is a *feature*:
forwarding-based reuse is statically traceable precisely because there is no hidden virtual re-entry.

### 3.1 Default methods on traits — **(pure composition; recommend: adopt as primary reuse)**

*(verified, primary — Rust Reference + Book)* A trait method with a body is a default that impls
inherit unless overridden: *"If the trait function defines a body, this definition acts as a default."*
The Book frames it as the inheritance substitute: *"similar to a parent class having an implementation
of a method."* **Limit:** defaults attach to a *trait*, not to data — they can call other trait methods
but cannot reference fields/state.

> **Mycelium:** Already implied by RFC-0019's trait system. This is the **first-line** behavior-reuse
> mechanism: zero kernel cost (elaborates to ordinary dictionary entries), fully coherent, statically
> dispatched, no late-binding surprise. Composition-alignment: **maximal.** EASIER under Mycelium's
> premises (no state in defaults is a natural fit for value semantics).

### 3.2 Supertraits — **(interface inheritance only; recommend: adopt for "requires")**

*(verified, primary)* `trait Sub: Super` *requires* `Super` and grants access to its items. **Limit:**
this is a *requirement* relation — **no implementation is inherited**; `Sub` does not get `Super`'s
bodies as data. It is interface subtyping among traits, not struct inheritance.

> **Mycelium:** The clean way to express "trait B builds on trait A's contract" without any
> implementation-inheritance machinery. Pure composition; coherent; no kernel growth. Composition-
> alignment: **maximal** (it is literally only an interface constraint).

### 3.3 Delegation / forwarding (`by` / `~>`) — **(composition + generated forwarders; recommend: the headline `~>` operator)**

Kotlin `class C(b: Base) : Iface by b` makes the compiler **generate forwarding methods** to `b`.
**The key limit, verbatim from the Kotlin docs** *(verified, primary)*: *"members overridden in this
way do not get called from the members of the delegate object, which can only access its own
implementations of the interface members."* The delegate has no reference to the wrapper → **no late
binding back into C.** This is the precedent for a Mycelium `~>` delegation operator.

> **Mycelium:** `~>` should be **surface sugar that desugars to generated forwarding methods**, exactly
> the DN-23 / RFC-0024 discipline (frontend-only, no kernel change). It removes the boilerplate of
> hand-writing forwarders while keeping the semantics *transparent and never-silent*: every forwarded
> method is a concrete, `EXPLAIN`-able function, and the "no virtual re-entry into the wrapper" limit is
> not a regression but the **honest** behavior (no fragile-base-class action-at-a-distance). EASIER
> under Mycelium: immutability + acyclicity mean the wrapped value cannot mutate or cyclically reference
> the wrapper, so forwarding is a pure, hashable composition. Composition-alignment: **high** (it *is*
> composition; the only caveat is it deliberately lacks override-down-a-hierarchy). **Caution:** keep
> `~>` distinct from `@matured` (RFC-0017, a header annotation) — different namespace, no collision.

### 3.4 Go-style embedding (method promotion) — **(composition + promotion; recommend: optional, if a "promote all methods" sugar is wanted)**

*(verified, primary — Go spec + Effective Go)* An embedded field's methods are **promoted** into the
outer type's method set: *"The methods of embedded types come along for free."* **The key limit,
verbatim:** *"There's an important way in which embedding differs from subclassing … when they are
invoked the receiver of the method is the inner type, not the outer one."* So **no virtual dispatch
back into the outer type**, and the outer type is **not a subtype** of the inner.

> **Mycelium:** Embedding is `~>` with an implicit "promote *all* of the inner's methods" default
> rather than a named interface. If offered, it should be the same forwarding desugaring as `~>` with a
> wildcard. The "receiver stays the inner value" limit is *automatic* under value semantics (the inner
> value is a distinct immutable value). Composition-alignment: **high.** Mild caution: blanket promotion
> can pollute the outer interface — prefer explicit `~>` per interface for auditability (KISS/SoC).

### 3.5 Decorator pattern, functionally — **(composition by wrapping; recommend: the `@`-decorator)**

*(GoF, secondary)* Decorator's intent: *"Attach additional responsibilities to an object dynamically …
a flexible alternative to subclassing."* A decorator holds a wrapped component of the same interface,
forwards, and adds behavior before/after — functionally `g ∘ f`. **Limit:** like all forwarding, the
wrapped component does not see the decorator (no late binding up into the wrapper) — which is exactly
what makes it composition, not inheritance.

> **Mycelium:** An `@decorator` is `~>` plus added/overridden behavior around the forwarded calls — a
> value-to-value transform `T -> T`. Under value semantics a decorator is a **pure wrapping function**,
> which is the cleanest possible form (no shared mutable state, content-addressable, stackable). This is
> the precedent for an `@`-decorator surface form, again **frontend sugar desugaring to a wrapping
> function** (DN-23 discipline). Composition-alignment: **high.** *Constraint:* avoid overloading the
> `@` glyph against `@matured`-style header annotations — scope `@`-decorators to expression/value
> position, not header position.

### 3.6 Row polymorphism / extensible records — **(structural extension; recommend: a niche escape, not the default)**

*(verified, primary — Leijen 2005)* A record type is a **row** = a sequence of labeled fields; **row
variables** make a function polymorphic over "the rest": `(.l) :: ∀r α. {l :: α | r} → α` accepts any
record that *has* `l` plus more fields — structural width subtyping via *parametric polymorphism*. The
*"extend the record with more fields/methods"* move is the structural flavor of inheritance. Leijen's
distinctive choice: **duplicate labels are allowed and retained** (*"the previous fields are always
retained, both in the value and in the type"* → label scoping), needing no lacks/absence machinery.
OCaml realizes the same twice (object rows `< m:t; .. >`; polymorphic variants `` [> `A] `` open /
`` [< `A|`B] `` closed); PureScript records `{ x :: Int | r }` likewise retain duplicates; Ur/Web
pushes to type-level record metaprogramming.

> **Mycelium:** Row polymorphism is "an object *is* a record; extend = add fields/rows" — purely
> structural, anonymous, no nominal contract. **It is in tension with Mycelium's premises**, not
> aligned with them: coherence and content-addressed identity want **nominal, coherent** trait
> contracts (one structure → one semantics), whereas rows are structural and any record of the right
> shape qualifies — giving up the global-uniqueness guarantee the trait system exists to provide. The
> recommendation is therefore **narrow**: offer row/structural records only for genuinely structural,
> contract-free data (config-shaped records, open extensible payloads), *not* as the behavior-reuse
> mechanism. Composition-alignment: **structural, orthogonal** — flexible but it sidesteps (rather than
> composes with) the coherent trait model. CONSTRAINED by Mycelium's premises.

### 3.7 Record-of-closures / open recursion — see §4 (the escape hatch; **lowest composition-alignment**, deliberately)

**Ranking by composition-alignment (most → least):**
1. **Default methods** (§3.1) — pure, stateless, coherent reuse.
2. **Supertraits** (§3.2) — interface-only "requires."
3. **Delegation `~>`** (§3.3) — composition + generated forwarders; the headline operator.
4. **Decorator `@`** (§3.5) — wrap-and-extend; pure value transform.
5. **Embedding / promotion** (§3.4) — `~>` with wildcard promotion.
6. **Row polymorphism** (§3.6) — structural, orthogonal to the coherent trait model.
7. **Record-of-closures / open recursion** (§4) — true late binding, but forfeits coherence + static
   dispatch; reserved escape hatch.

(Scala's linearization is the one mechanism surveyed that keeps virtual `super` across mixins — see the
note below — but it is *implementation inheritance with a deterministic order*, which Mycelium
deliberately does not want; it is included for completeness, not recommended.)

**Scala linearization (for completeness, not recommended)** *(verified, primary — Scala spec).*
`L(C) = C, L(Cₙ) +⃗ … +⃗ L(C₁)`: prepend `C`, concatenate parents' linearizations **right-to-left**,
dropping any class already present to its left so each appears once at its **leftmost (most-derived)**
position; `super` resolves along this single linear order (which is why traits are *stackable* and the
diamond problem is deterministic). Self-types (`this: T =>`) declare a *requirement* without becoming a
subtype. **Adversarial note:** the common "right-to-left depth-first, last occurrence kept" phrasing is
imprecise — the spec keeps the **leftmost** occurrence in the *resulting* order; "rightmost trait wins"
is true only of the *source `with` order*. Mycelium should **not** adopt linearization: it is genuine
(if ordered) implementation inheritance with virtual re-entry, the very action-at-a-distance the
never-silent / acyclic / value-semantics premises reject.

**The Rust `Deref` anti-pattern — explicit caution** *(verified, primary — rust-unofficial patterns).*
Do **not** emulate inheritance by implementing `Deref` on a wrapper to auto-forward methods: it is *"a
surprising idiom,"* `Deref` *"is designed for the implementation of custom pointer types,"* and crucially
*"does not introduce subtyping … traits implemented by Foo are not automatically implemented for Bar,
so this pattern interacts badly with bounds checking and thus generic programming."* The Mycelium lesson:
make delegation **explicit and named (`~>`)**, never an implicit auto-deref that fakes "is-a" — the
explicit forwarding is the never-silent, auditable form; auto-deref is exactly the silent black box
the house rules forbid.

---

## 4. The late-binding escape hatch + its cost

When you truly need a base method to see a subclass override (true late binding / open recursion), the
typed-FP encoding is **objects as records of functions with an explicit `self`** (Cook & Palsberg,
*A Denotational Semantics of Inheritance and its Correctness*, OOPSLA'89 / Inf.&Comp. 1994)
*(verified, primary, quotes from the IC94 PDF)*:

- An object is *"a record value whose fields represent methods."* A class is a **generator** — a
  function of `self` returning the method record:
  `MakeGenPoint(a,b) = λself.{ x↦a, y↦b, dist↦sqrt(self.x² + self.y²), … }`.
- **`fix` ties the knot:** `p = fix(MakeGenPoint(3,4))`, `fix(f) = ⊔ₙ fⁿ(⊥)`.
- **Inheritance = modify the generator *before* the fixpoint.** A subclass is a wrapper
  `λself.λsuper.{…}`, and inheritance is wrapper application on generators, verbatim:
  `W ▷ G = λself. (W(self)(G(self))) ⊕ G(self)` where `⊕` overrides. The decisive detail: the **same
  `self` is distributed to both `W` and `G` before `fix`**, so when base `G`'s methods call
  `self.dist`, after `fix(W ▷ G)` that `self` is the *combined* record — **base-class self-calls see
  the override.** The paper names it: self-reference *"must be changed to refer to the modified
  definition."* That is open recursion / true late binding.

**The cost (verified — follows from the encoding):**
1. **Runtime indirection.** `self` is a runtime record; every `self.m` is a projection through it, and
   `▷`/`⊕` build closures at construction. Dispatch is dynamic by construction.
2. **No whole-program devirtualization / monomorphization.** The target isn't statically known, so the
   compiler cannot specialize/inline the way coherent trait resolution can.
3. **Loss of coherence / closed-world guarantees.** Hand-rolled records-of-functions have no global
   uniqueness — anyone can build a differently-wrapped generator of the same shape.

**The typed alternative that keeps static dispatch** is **final-tagless** (Carette/Kiselyov/Shan):
interpretations are typeclass *instances* resolved by **compile-time instance selection (static
dispatch, no tags)**, recovering exactly the monomorphization/coherence the hand-rolled `fix`/`self`
encoding sacrifices — at the price of not having mutable late-bound `self`.

> **Mycelium mapping & cost.** This escape hatch sits **dead against** Mycelium's premises and should be
> the **last resort, explicitly flagged**:
> - **Acyclicity tension:** `fix` over a `self`-record is precisely the cyclic self-reference the
>   acyclic value model excludes; an immutable acyclic store cannot represent the back-edge without a
>   special boxed/indirection construct. This is the sharpest "value-semantics + acyclic CONSTRAINS the
>   pattern" finding.
> - **Coherence loss:** records-of-closures abandon the one-hash-one-semantics invariant (ADR-003) that
>   the whole trait system is built to guarantee.
> - **Performance loss:** forfeits the monomorphized static dispatch that §5 shows is Mycelium's
>   natural performance model.
> Recommendation: **do not provide open-recursion objects as a surface feature.** If genuine late
> binding is ever required, reach for **object algebras / final-tagless** (§1.2) — they buy the
> open-extensibility axis *inside* coherent static dispatch. Reserve a raw record-of-closures only as an
> EXPLAIN-flagged, `Declared`-tagged escape, never the default.

---

## 5. Performance — monomorphization vs dynamic dispatch (and the value-semantics + RC interaction)

### 5.1 Static dispatch / monomorphization

*(verified — rustc-dev-guide)* Monomorphization *"stamps out a different copy of the code of a generic
function for each concrete type."* **Benefit:** direct, **inlinable** calls → the gateway to
devirtualization, const-prop, DCE across the call boundary — "zero-cost abstraction" (the Rust Book:
generics carry *"no runtime cost"*). **Cost, verbatim:** *"fast programs, but … at the cost of compile
time … and binary size (all those copies might take a lot of space)."* Rule of thumb *(secondary,
corroborated by the dev guide's direction):* a function of size `n` used at `m` types costs ≈ `n×m`.
Mitigation — **polymorphization** (share copies whose blocks don't depend on the type param) — is an
*in-flux effort*, not a shipped guarantee (active proposal to delete the current impl); cite as
direction only.

### 5.2 Dynamic dispatch

*(verified — Rust std `dyn` docs)* A `dyn Trait` reference is a **fat pointer**: *"two pointers. One …
to the data … Another … to a … vtable."* **Cost, verbatim:** *"the additional runtime cost … Methods
called by dynamic dispatch generally cannot be inlined"* — the dominant cost is the **lost
optimization** (opaque call boundary), plus indirect-branch / I-cache pressure. **Benefit, verbatim:**
*"likely to produce smaller code … as the method won't be duplicated for each concrete type"* — plus
**heterogeneous collections** (`Vec<Box<dyn Trait>>`), stable ABI/plugin boundaries, separate
compilation, dynamic linking.

### 5.3 When each is appropriate

Monomorphize **hot generic code** with a small type set (bounded bloat, inlining wins dominate). Use
dynamic dispatch for **heterogeneous collections, plugin/ABI seams, and to cut code size / compile
time**, specifically where the indirect call is *not* hot. Real systems mix them — generics in hot
loops, `dyn` at architectural seams.

### 5.4 Interaction with value semantics + RC — Swift is the prime precedent

**Value semantics + monomorphization compose cleanly** *(secondary, design inference grounded in
layout facts):* values pass by value/move into specialized code; **no hidden vtable in the layout** →
flat, small, predictable layouts — exactly what content-addressing and structural dedup want (a
canonical monomorphic layout hashes to a stable identity). **Dynamic dispatch reintroduces
indirection:** a `dyn`-like value can't be flat — to be uniform-sized it needs a boxed/RC'd payload
plus a vtable (fat pointer + refcount), in tension with flat value layouts but the enabler of
heterogeneous immutable collections.

**Swift — the closest real-world analog** (value types + protocol-oriented programming + witness tables +
ARC), from *Understanding Swift Performance* (WWDC 2016) *(primary session; 3-word buffer / PWT-VWT
split confirmed by two independent transcriptions):*
- Three costs: **allocation, reference counting, method dispatch.** Stack alloc *"much much faster than
  … heap."*
- Dispatch: static is *"a candidate for inlining"*; dynamic *"prevents inlining and other
  optimizations"* — identical to the Rust framing.
- **Existential container** (boxed protocol value): a fixed struct with an **inline value buffer of 3
  words**, plus a pointer to the **Value Witness Table** (alloc/copy/destroy lifecycle) and the
  **Protocol Witness Table** (method dispatch). *Small types ≤ 3 words live **inline** (no heap box);
  larger types are **heap-boxed** with only a pointer in the buffer.*
- Reference counting is *"more costly than just a simple increment/decrement"* — indirection + **atomic
  (thread-safe) overhead**, paid **per reference member** of a struct.

> **Mycelium mapping.** Mycelium's premises make the **monomorphized / static-dispatch model the
> natural default and a strict win:** value semantics + content-addressing want flat, canonical,
> hashable layouts with no vtable welded into the value — which is exactly what monomorphization
> produces, and exactly what the dictionary-passing-then-specialize path of RFC-0019 enables. *EASIER
> under Mycelium.* When dynamic dispatch is eventually added (greenfield), the **Swift design is the
> template to copy**: a fat-pointer / witness-table existential with a **small-value inline buffer** so
> that small immutable values dispatch dynamically *without* a heap box or refcount, and only larger
> values pay the RC'd box. The PWT/VWT split is directly instructive — separate *method dispatch* (PWT)
> from *value lifecycle* (VWT), and a content-addressed RC runtime would extend the VWT with
> retain/release/**hash**. The cost to budget: a `dyn`-like value gives up the canonical flat layout and
> takes on a refcount — so it is the deliberate, EXPLAIN-able exception, never the default. *CONSTRAINED
> by Mycelium:* heterogeneous immutable collections are the legitimate driver for dynamic dispatch, and
> they cost a box + refcount; everything else should stay monomorphized.

---

## 6. Annotated bibliography (URLs)

**Objects, ADTs, expression problem**
1. William R. Cook, *On Understanding Data Abstraction, Revisited*, OOPSLA 2009 —
   <https://www.cs.utexas.edu/~wcook/Drafts/2009/essay.pdf> — **primary, verified (PDF→text).** Objects
   vs ADTs duality, autognosis, the representation/operation matrix, §5.3 type-class analysis.
2. Philip Wadler, *The Expression Problem* (1998 email) —
   <https://homepages.inf.ed.ac.uk/wadler/papers/expression/expression.txt> — **primary, verified.**
   The verbatim problem statement + rows/columns framing.
3. Jacques Carette, Oleg Kiselyov, Chung-chieh Shan, *Finally Tagless, Partially Evaluated*, JFP 19
   (2009) — <https://okmij.org/ftp/tagless-final/JFP.pdf> — **primary, verified.** `Symantics` typeclass;
   "combinator functions rather than data constructors"; static-dispatch EP solution.
4. Bruno C.d.S. Oliveira & William R. Cook, *Extensibility for the Masses: Practical Extensibility with
   Object Algebras*, ECOOP 2012 — <https://www.cs.utexas.edu/~wcook/Drafts/2012/ecoop2012.pdf> —
   **primary, verified.** Object algebras = generic-factory EP solution in plain generics.
5. John C. Reynolds, *User-Defined Types and Procedural Data Structures …* (1975) — **secondary**
   (originator of the duality; cited via Cook).

**Typeclasses as interfaces**
6. Philip Wadler & Stephen Blott, *How to make ad-hoc polymorphism less ad hoc*, POPL 1989 —
   <https://dl.acm.org/doi/10.1145/75277.75283> (PDF:
   <https://www.cse.iitk.ac.in/users/karkare/Courses/cs653/Papers/ad-hoc-polymorphism.pdf>) — **primary,
   verified.** Dictionary-passing translation (class→record, instance→value, constraint→extra arg).
7. The Rust Reference — *Trait objects* — <https://doc.rust-lang.org/reference/types/trait-object.html> —
   **primary/official, verified.** Fat-pointer + vtable + virtual dispatch.
8. The Rust Programming Language (Book) — *Traits* / *OOP features* —
   <https://doc.rust-lang.org/book/ch10-02-traits.html> ·
   <https://doc.rust-lang.org/book/ch18-01-what-is-oo.html> — **primary/official, verified.** Traits ≈
   interfaces; no struct inheritance; default-method reuse; trait objects for runtime polymorphism.
9. Oleg Kiselyov, *Implementing, and Understanding Type Classes* —
   <https://okmij.org/ftp/Computation/typeclass.html> — **secondary, authoritative.** Dictionary ↔
   monomorphization.
10. HaskellWiki, *OOP vs type classes* — <https://wiki.haskell.org/OOP_vs_type_classes> — **secondary.**
    Value↔dictionary binding "may change at runtime"; the value/dictionary vs object/vtable contrast.
11. Terbium, *Comparing Traits and Typeclasses* (2021) — <https://terbium.io/2021/02/traits-typeclasses/>
    — **secondary.** Retroactive impl, return-type dispatch, orphan-rule coherence.

**Inheritance emulation**
12. The Rust Reference — *Traits* (default methods, supertraits) —
    <https://doc.rust-lang.org/reference/items/traits.html> — **primary, verified.**
13. Rust Design Patterns (rust-unofficial) — *Deref Polymorphism (anti-pattern)* —
    <https://rust-unofficial.github.io/patterns/anti_patterns/deref.html> — **primary, verified.** The
    explicit caution against faking inheritance via `Deref`.
14. Scala Language Specification 2.13 — §5 *Classes and Objects* (linearization `+⃗`) —
    <https://www.scala-lang.org/files/archive/spec/2.13/05-classes-and-objects.html> — **primary,
    verified.** Leftmost-occurrence linearization rule; `super` along the linear order.
15. Scala — *Self-types* — <https://docs.scala-lang.org/tour/self-types.html> — **primary, verified.**
16. Kotlin Documentation — *Delegation* — <https://kotlinlang.org/docs/delegation.html> — **primary,
    verified.** Generated forwarders + the "delegate can't see overrides" caveat (verbatim).
17. The Go Programming Language Specification — *Struct types* (embedding/promotion) —
    <https://go.dev/ref/spec> — **primary, verified.**
18. Effective Go — *Embedding* — <https://go.dev/doc/effective_go> — **primary, verified.** "Embedding
    differs from subclassing … the receiver … is the inner type."
19. Gamma, Helm, Johnson, Vlissides, *Design Patterns* (1994), Decorator (p. 175) — **secondary**
    (printed; intent line widely reproduced — verify against print).

**Row polymorphism / extensible records**
20. Daan Leijen, *Extensible records with scoped labels*, TFP 2005 —
    <https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/scopedlabels.pdf> — **primary,
    verified (PDF→text).** Rows, row variables, retained-duplicate scoped labels, `(eq-swap)` rule.
21. OCaml Manual — *Polymorphic variants* (open `` [> ] `` / closed `` [< ] ``) —
    <https://ocaml.org/manual/5.2/polyvariant.html> — **primary, verified.**
22. OCaml Manual — *Object types* (row variable `..`) — <https://ocaml.org/manual/4.13/types.html> —
    **primary, verified.**
23. PureScript documentation — *Types* (`{ x :: Int | r }`, retained duplicates) —
    <https://github.com/purescript/documentation/blob/master/language/Types.md> — **primary, verified.**
24. Adam Chlipala, *Ur: Statically-Typed Metaprogramming with Type-Level Record Computation*, PLDI 2010
    — <https://adam.chlipala.net/papers/UrPLDI10/> — **primary, verified (abstract).**
25. Elm — *Records* — <https://elm-lang.org/docs/records> — **secondary.**

**Late binding / open recursion**
26. William Cook & Jens Palsberg, *A Denotational Semantics of Inheritance and its Correctness*,
    OOPSLA'89 / Information and Computation 114(2) 1994 — <https://web.cs.ucla.edu/~palsberg/paper/ic94.pdf>
    — **primary, verified (PDF→text).** Generator-over-`self`, `fix`, wrapper application
    `W ▷ G = λself.(W(self)(G(self))) ⊕ G(self)`; "self-reference must be changed to refer to the
    modified definition."
27. William Cook, *A Denotational Semantics of Inheritance* (PhD thesis, 1989) —
    <https://www.cs.utexas.edu/~wcook/papers/thesis/cook89.pdf> — **secondary** (located).

**Performance**
28. *Monomorphization* — Rust Compiler Development Guide —
    <https://rustc-dev-guide.rust-lang.org/backend/monomorph.html> — **primary/official, verified.**
29. `dyn` keyword — Rust std docs — <https://doc.rust-lang.org/std/keyword.dyn.html> — **primary/official,
    verified.** Fat pointer; "cannot be inlined"; "smaller code" benefit.
30. A. Lilley Brinker, *Monomorphization Bloat* (the `n×m` formula) —
    <https://www.alilleybrinker.com/blog/monomorphization-bloat/> — **secondary, corroborated.**
31. Rust Polymorphization WG + David Wood, *Polymorphisation* dissertation —
    <https://rust-lang.github.io/compiler-team/working-groups/polymorphization/> ·
    <https://github.com/rust-lang/compiler-team/issues/810> — **verified the effort exists; status in
    flux** (cite as direction, not shipped).
32. Apple, *Understanding Swift Performance*, WWDC 2016 (session 416) —
    <https://developer.apple.com/videos/play/wwdc2016/416/> — **primary.** Existential container, 3-word
    inline buffer, PWT/VWT, ARC. Community transcriptions:
    <https://saurabhs.org/wwdc-notes/wwdc-16-understanding-swift-performance> (**secondary, corroborated**).

---

## 7. Mycelium-premise impact summary — where the premises help vs constrain

| Pattern | Mycelium premise effect |
|---|---|
| Coherent typeclass-as-interface (§2) | **EASIER** — content-addressing *forces* coherence (ADR-003), so Mycelium gets retroactive impls + return-type dispatch on firmer ground than Haskell (structural, not social, coherence). |
| Default methods / supertraits (§3.1–3.2) | **EASIER** — stateless reuse fits value semantics; zero kernel cost. |
| Delegation `~>` / decorator `@` / embedding (§3.3–3.5) | **EASIER** — immutability + acyclicity make forwarding a pure, hashable composition; the "no virtual re-entry" limit is the *honest* (never-silent) behavior. Desugar as frontend sugar (DN-23). |
| Object algebras / final-tagless (§1.2) | **NEUTRAL/EASIER** — solves the open-extensibility axis inside coherent static dispatch; the recommended way to get EP-style flexibility without abandoning the model. |
| Row polymorphism (§3.6) | **CONSTRAINED** — structural/anonymous, in tension with coherent nominal contracts; keep niche (config-shaped data), not the behavior-reuse mechanism. |
| Record-of-closures / open recursion (§4) | **MOST CONSTRAINED** — `fix`-over-`self` *is* the cyclic back-edge the acyclic value model excludes; forfeits coherence + monomorphization. Escape hatch only, `Declared`-tagged + EXPLAIN-flagged. |
| Static dispatch / monomorphization (§5) | **EASIER** — flat, canonical, vtable-free layouts are exactly what value semantics + content-addressing want; the natural default and a strict win. |
| Dynamic dispatch (greenfield) (§5.4) | **CONSTRAINED but bounded** — a `dyn`-value costs a box + refcount; copy Swift's small-value-inline-buffer + PWT/VWT existential so small immutable values dispatch without a heap box. |

*Transparency (VR-5): primary-source quotes verified by direct PDF/spec extraction; secondary/inferred
claims tagged inline. The Mycelium-mapping recommendations are **Declared-with-argument** design
proposals — they cite their basis (the repo RFCs + the surveyed mechanisms) but are not themselves
ratified decisions.*
