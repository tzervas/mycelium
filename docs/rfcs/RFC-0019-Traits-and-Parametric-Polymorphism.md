# RFC-0019 — Traits and Parametric Polymorphism (LR-2)

| Field | Value |
|---|---|
| **RFC** | 0019 |
| **Status** | **Accepted** (2026-06-18, maintainer ratification) — coherence = **orphan rule + global uniqueness + reject-overlap** (R1, `research/10`); the **Repr-polymorphism restriction set** (§4.6, locally checkable, S1-preserving) is normative (R2); **newtype-derived coherence waivers rejected in v1** (need roles); **multi-parameter traits and associated types deferred to v2**; dictionary-passing elaboration, kernel node budget unchanged (KC-3). *Honesty note (VR-5):* the coherence and S1-preservation results are **Declared-with-argument** (not machine-checked) — acceptance does not upgrade that tag; mechanization is the basis for a future `Proven` upgrade. |
| **Type** | Foundational / normative (once Accepted) |
| **Date** | June 18, 2026 |
| **Depends on** | RFC-0006 (S1–S6, LR-2/LR-5/LR-6; KC-2 verdict DN-09); RFC-0007 §4.1–4.4 (L1 kernel calculus: `Lam/App/Construct/Match/Fix`; declarations-as-registry; v0 monomorphic judgments; §4.4 "polymorphism/traits deliberately out of v0 — its own later RFC"); ADR-003 (Unison-style content-addressed identity); ADR-006 (no black boxes); KC-3 (small auditable kernel); DN-03 (`impl` for inherent methods; `grow` = derive-like); DN-09 §3.2 (usability over theming); the v0 grammar `docs/spec/grammar/mycelium.ebnf` (existing `trait`/`type_params` productions) |
| **Coupled with** | `docs/spec/grammar/mycelium.ebnf` (§4.1 grammar extensions); the forthcoming **stage-1 grading RFC** (referred to here as "RFC-0018/grading RFC" — the static graded coeffect judgment for guarantee-indexed types, RFC-0006 §8 Q3); `crates/mycelium-l1` (the kernel prototype whose node budget must not grow) |

## 1. Summary

RFC-0007 §4.4 explicitly defers parametric polymorphism and traits to their own RFC: "Polymorphism
(type parameters, traits/LR-2) is **deliberately out of v0** … The trait system is its own later
RFC." This is that RFC.

It specifies the **LR-2 layer**: Rust/ML-class typeclasses with **coherence** (no
overlapping-instance ambiguity), integrated with the L1 kernel calculus via **dictionary-passing
translation** (recommended over monomorphization — see §4.4), and extended with two
Mycelium-specific novelties that have no mainstream analogue:

- **Repr-polymorphism (LR-5):** abstracting over the representation paradigm (`∀ r: Repr`) while
  keeping `Swap` insertion explicit (S1 — never-silent swaps).
- **Guarantee-indexed trait methods (LR-6):** trait method signatures that are polymorphic over
  the guarantee lattice, returning the `meet` of the inputs' strengths.

Both novelties are **flagged as open research** (§9): their restriction sets and soundness
arguments need a survey + worked sketch before this RFC can be ratified as Accepted.

The kernel node budget (RFC-0007 §4.1 — ten nodes) **does not grow**. Traits elaborate entirely
via existing nodes (`Construct`, `Lam`, `App`, `Let`), plus registry entries for trait
declarations and instance dictionaries (§4.3).

## 2. Motivation

### 2.1 What is blocked without LR-2

RFC-0007 §4.4 leaves a deliberate gap: type parameters are hashed as-is in registry entries, but
any attempt to *instantiate* a generic at a concrete type is an explicit deferred error. This
blocks:

- Generic data structures (`List<T>`, `Option<T>`, `Map<K, V>`) — already appearing in the v0
  grammar's `type_params` production but not yet elaborable.
- Polymorphic functions (`fn id<T>(x: T) -> T`).
- Any code that is parameterized by a *capability*: sorting requires `Ord`, hashing requires
  `Hash`, an equality check requires `Eq`. Without trait bounds these are either monomorphic
  (concrete type hard-wired) or untyped (defeating the honesty rule).
- The whole Mycelium-specific stack: swap policies are functions, their certificates have types,
  and those types must be generic over `Repr` to write a single reusable policy library.

### 2.2 Why coherence is non-negotiable (LR-2)

LR-2 explicitly requires "coherent, no overlapping-instance ambiguity." The reason is structural,
not stylistic. Mycelium uses **content-addressed identity** (ADR-003): a definition's identity is
its α-normalized structure hash. If two programs could each define `impl Ord for Binary{8}` with
different orderings and have both valid, the *same hash* would map to *two semantics* depending on
which instance is in scope — a direct violation of ADR-003. Global coherence is the content-
addressing–consistent choice, not an optional style rule.

### 2.3 Why the kernel node budget must not grow (KC-3)

The ten-node L1 budget is the trusted base that KC-3 guards. Every researcher who surveyed
typed-kernel architectures (T3.1) found the same answer: when the kernel grows to accommodate a
high-level feature, the trusted base becomes un-auditable. Traits are a *surface* convenience; the
evidence (GHC Core, Lean 4, Coq) is that they elaborate cleanly to the lambda calculus via
dictionary-passing, with zero new trusted nodes. This RFC commits to that path.

## 3. Guide-level explanation (surface — usability-first, DN-09 §3.2)

### 3.1 Trait declarations

A trait names a set of function signatures parameterized over one or more type variables:

```mycelium
// A type that can be compared for equality.
trait Eq<A> {
    fn equal(x: A, y: A) -> Binary{1}
    fn not_equal(x: A, y: A) -> Binary{1}
}

// A type with a total order.
trait Ord<A> {
    fn compare(x: A, y: A) -> Ordering
}

// A type that can be shown as a binary string (illustrates a simple projection).
trait Show<A> {
    fn show(x: A) -> List<Binary{8}>
}
```

The existing `trait_item` production in `docs/spec/grammar/mycelium.ebnf` already parses this
form:

```ebnf
trait_item ::= 'trait' Ident type_params? '{' fn_sig* '}'
```

No grammar change is needed for basic trait declarations.

### 3.2 Trait instances

A **trait instance** (also called an *implementation* or *witness*) provides a concrete
definition for every method in a trait, for a specific type. The keyword is `impl … for …`:

```mycelium
// Instance of Eq for Binary{8}.
impl Eq<Binary{8}> {
    fn equal(x: Binary{8}, y: Binary{8}) -> Binary{1} = …
    fn not_equal(x: Binary{8}, y: Binary{8}) -> Binary{1} = …
}

// Instance of Ord for Binary{8}.
impl Ord<Binary{8}> {
    fn compare(x: Binary{8}, y: Binary{8}) -> Ordering = …
}
```

**Keyword rationale.** The choice of `impl … for …` vs alternatives (e.g., `instance`, `grow …
for …`, `embody … for …`) is decided here on the DN-09 §3.2 usability-over-theming principle and
the DN-03 record:

- **`impl` for inherent methods** is already decided (DN-03 §1: `embody` was declined, `impl` is
  kept for when inherent methods enter the grammar). Reusing `impl` for trait instances
  (`impl Trait for T`) follows the exact Rust idiom, which is the highest-familiarity spelling for
  both human and machine authors (SC-5b, G10). The word already appears in the reserved set; no
  new keyword is needed.
- **`grow`** (DN-03 §1) is the reserved themed word for a *derive-like* capability extension
  (`grow Debug for T`). It is appropriate when the instance is *derived* or *auto-generated* (a
  derive macro analogue). For *hand-written* instances `impl … for …` is preferred — the derive
  path calls `grow`, which may desugar to the same `impl … for …` form. This two-word discipline
  (hand-write with `impl`, derive with `grow`) is the chosen split.
- The grammar extension for `impl_item` is specified in §4.1.

### 3.3 Bounded generics

A function (or type) parameterized over a type variable can constrain it with trait bounds:

```mycelium
// A function generic over any type with Eq.
fn contains<T: Eq<T>>(needle: T, haystack: List<T>) -> Binary{1} = …

// Multiple bounds, separated by '+'.
fn sort_and_deduplicate<T: Ord<T> + Eq<T>>(xs: List<T>) -> List<T> = …

// A generic data type (bounds on the type itself are on the methods, not the decl — see §4.2).
type Map<K, V> = Leaf | Node(K, V, Map<K, V>, Map<K, V>)
```

The existing `type_params` grammar production handles the `<T>` side; the bound annotation `T:
Trait` requires a small grammar extension (§4.1) to add a `where_clause` or inline bound syntax
to `fn_item`.

### 3.4 Desugaring (preview; full detail in §4.3)

Every `fn f<T: Trait>(…)` desugars to a function that **explicitly receives the trait dictionary
as an argument**:

```text
fn f<T: Trait>(x: T) -> T
    ⤳   fn f(dict_Trait_T: DictTrait<T>, x: T) -> T
```

A call site `f(v)` where the solver knows `T = Binary{8}` resolves to:

```text
f(v)   ⤳   f(impl_Trait_Binary8_dict, v)
```

This elaboration is explicit, inspectable (S4), and uses only existing L1 nodes. The dictionary
is a `Construct` record. Dictionary passing is a well-understood scheme (GHC 1993; Haskell Report;
Rust trait objects share the same underlying representation for `dyn Trait`).

## 4. Reference-level design (normative once Accepted)

### 4.1 Grammar extensions

The following changes to `docs/spec/grammar/mycelium.ebnf` are required (additive; no production
is removed):

```ebnf
/* Trait instances. */
item           ::= use_item
                 | default_item
                 | type_item
                 | trait_item
                 | impl_item          (* NEW *)
                 | fn_item

/* A trait instance: provides all methods of the named trait for a specific type. */
impl_item      ::= 'impl' Ident type_args? 'for' type_ref '{' impl_method* '}'
impl_method    ::= 'fn' Ident '(' params? ')' '->' type_ref '=' expr

/* Bounded type parameters (inline bound syntax; 'where' clauses deferred to L2 sugar). */
type_param     ::= Ident (':' bound)?
bound          ::= Ident type_args? ('+' Ident type_args?)*   (* one or more trait names *)

/* Replace: type_params ::= '<' Ident (',' Ident)* '>'
 * With: *)
type_params    ::= '<' type_param (',' type_param)* '>'

/* Function items: type_params is updated to allow bounds per above. No other fn_item change. */
```

All additions are additive; every existing v0 program that parses today still parses under this
extension (the empty-bound case `type_param ::= Ident` is the identity of the old production).

### 4.2 Trait declarations — registry entries, not kernel nodes

Following the RFC-0007 §4.2 declarations-as-registry discipline, a trait declaration is a
**registry entry**:

```text
TraitDecl {
    name:        Ident,                      (* names are metadata, not identity — ADR-003 *)
    params:      Vec<TypeParam>,             (* the type variables, possibly with super-trait bounds *)
    method_sigs: Vec<(Ident, MethodSig)>,    (* ordered list — order is significant for the dictionary layout *)
    hash:        DeclHash,                   (* α-normalized structural hash; identity *)
}
```

A `TraitDecl` is content-addressed over its α-normalized structure: the parameter count, the
method signatures in declaration order, and any super-trait constraints. *Names are metadata*.
Two traits with the same structure hash are the same trait (ADR-003); rename-refactors that
preserve structure preserve identity.

A trait **instance** (`impl Trait for T`) is also a registry entry:

```text
InstanceDecl {
    trait_ref:   DeclHash,                   (* the trait's hash *)
    type_arg:    Type,                        (* the type this instantiates for *)
    methods:     Vec<(Ident, TermHash)>,     (* each method body, content-addressed *)
    hash:        DeclHash,
}
```

Instance hashing: the hash of `InstanceDecl` is computed over `(trait_ref, type_arg, methods)` in
canonical form. This means the dictionary's identity is determined by **which trait**, **which
type**, and **which method bodies** — consistent with ADR-003 and the content-addressed `Swap`
certificate story (ADR-003, RFC-0002).

### 4.3 Elaboration to L1: dictionary-passing translation

**Core translation.** A trait `Trait` over type variable `A` with methods `m₁: τ₁, …, mₙ: τₙ`
elaborates to a **dictionary type** (a data declaration in the registry):

```text
type Dict_Trait<A> = MkDict_Trait(
    field_m₁: τ₁[A ↦ A],   (* function type, possibly Arrow(A, …) *)
    …
    field_mₙ: τₙ[A ↦ A]
)
```

This is a `Construct` record (RFC-0007 §4.1 `W6`-saturated) with one field per method.

**Function elaboration.** A function with a trait bound:

```text
fn f<T: Trait>(x: T) -> τ = body
```

elaborates to an L1 term that takes the dictionary as an **explicit first argument**:

```text
fn f_elab(dict: Dict_Trait<T>, x: T) -> τ =
    let m₁ = dict.field_m₁ in
    …
    body_elab
```

where each method call `m_i(…)` in `body` becomes `dict.field_mᵢ(…)` after elaboration (a
`Match` or a direct field projection; the implementation can use a flat `Match` on the
single-constructor `MkDict_Trait` to bind all method fields — this is W7-compliant).

**Call-site elaboration.** A call `f(v)` where the type checker resolves `T = C` (a concrete
type with a known instance `impl Trait for C`):

```text
f(v)   ⤳   f_elab(dict_Trait_C, v)
```

where `dict_Trait_C` is a `Construct` expression:

```text
dict_Trait_C = MkDict_Trait(
    field_m₁ = impl_m₁_for_C,
    …
    field_mₙ = impl_mₙ_for_C
)
```

and each `impl_mᵢ_for_C` is a `Var` bound to the corresponding method body from the
`InstanceDecl` registry entry.

**Multi-parameter bounds.** `fn g<T: Trait1 + Trait2>(x: T) -> τ` elaborates to two dictionary
arguments, one per bound, in bound declaration order (canonical and stable for hashing):

```text
fn g_elab(dict1: Dict_Trait1<T>, dict2: Dict_Trait2<T>, x: T) -> τ = …
```

**Super-traits.** If `trait Ord<A>` has super-trait `Eq<A>`, the `Dict_Ord<A>` record carries an
`eq_dict: Dict_Eq<A>` field as its first field. A call requiring `Ord<T>` implicitly has `Eq<T>`
available via `dict_ord.eq_dict` — no extra argument, no implicit lookup.

**No new L1 nodes.** The translation uses only:

- `Construct` (build a dictionary record),
- `Match` (destructure a dictionary to project a field),
- `Lam`/`App` (method calls via function application),
- `Let` (bind projected fields),
- `Var` (reference to a method implementation body).

The kernel node budget remains at ten (RFC-0007 §4.1). The elaborator is the untrusted layer; the
kernel is unchanged (KC-3).

**Inspectability (S4).** The LSP's stage-dump channel (SC-5/M-140) must be able to show the
dictionary-passing elaboration of any generic definition, as it does for any other L2→L1→L0
step. A user should be able to ask "what did this `fn f<T: Eq<T>>` elaborate to?" and receive a
human-readable L1 term with explicit dictionary arguments.

### 4.4 Dictionary-passing vs. monomorphization — analysis and recommendation

Both strategies are in production use (Haskell/OCaml use dictionary-passing; Rust/C++ use
monomorphization). Here is the trade-off analysis specific to Mycelium's requirements:

**Dictionary-passing (recommended):**

| Property | Assessment |
|---|---|
| Kernel node budget | No change — dictionaries are ordinary `Construct` records. |
| Content-addressed identity (ADR-003) | **Strong.** A generic function has *one* content-addressed hash regardless of how many types it is instantiated at. The instance dictionary is *also* content-addressed, separately. The policy-reference story ("which policy chose this swap?", RFC-0002/RFC-0005) extends cleanly: the dictionary is the policy. |
| Separate compilation | **Direct.** A `nodule` exports its generic definitions as-is; the consuming `nodule` does not need to re-elaborate the generic body. Only the dictionary construction is call-site-local. |
| Code size | **Small.** One copy of each generic function body. Dictionaries are small (one function pointer per method) and shared at call sites with the same type. |
| Runtime cost | **Indirect dispatch** per method call (one indirection through the dictionary record). For tight inner loops this is a known cost; monomorphization is faster there. For a design-phase corpus that does not yet have a performance-critical inner-loop story this is the right trade. A monomorphizing specialization pass (as an optional AOT optimization step outside the trusted kernel) can recover performance without adding complexity to the kernel or the elaboration rules. |
| Guarantee-indexed polymorphism (LR-6) | **Natural fit.** The guarantee index can be carried as a *field* of the dictionary, making the polymorphism over guarantees a dictionary-level concern, not a kernel-level one (§4.6). |
| Repr-polymorphism (LR-5) | **Workable, but restricted** — see §4.5 and §9 for the open soundness question. |

**Monomorphization:**

| Property | Assessment |
|---|---|
| Kernel node budget | No change at the *kernel* level, but the elaborator must generate one copy of the function body per instantiation — the *elaborated* corpus grows with usage. |
| Content-addressed identity | **Problematic.** `fn f<T: Eq<T>>` specialized to `Binary{8}` would have a *different* hash from the same function specialized to `Binary{16}`. This means "which version of `f` ran on this value?" is a separate tracking concern; the Unison-style "a definition is its hash" story fragments across specializations. Under ADR-003's model this is avoidable complexity. |
| Separate compilation | **Harder.** The full body of a generic must be available at every instantiation site — the Rust/C++ template inclusion-model problem. For a content-addressed, module-hashing system (LR-3) this interacts badly with the `nodule`/`phylum` boundary. |
| Code size | **Grows with usage** — the classic monomorphization blowup. Not a concern in a design-phase corpus; becomes relevant once a standard library exists (RFC-0016). |
| Runtime cost | **No indirect dispatch** — each specialization is a direct call. Faster in tight loops. |

**Recommendation:** implement dictionary-passing as the normative elaboration. An optional
monomorphizing AOT specialization pass (outside the trusted kernel, producing non-normative
specialized copies for performance-critical paths) is a later RFC-0004-style promotion step, not
a semantic requirement. This matches GHC's architecture (dictionary-passing in the Core, optional
worker/wrapper + inlining for specialization) and preserves ADR-003 identity cleanly.

### 4.5 Coherence — orphan rule and overlapping-instance discipline

**Why coherence is required** is explained in §2.2 (content-addressing demands it). This section
specifies *how* to enforce it.

**Global uniqueness rule.** For any pair `(Trait, Type)` there is **at most one** `InstanceDecl`
in the registry at any point. Attempting to add a second is an **explicit error** (never a silent
shadowing), naming both the conflicting instance hashes and the `(Trait, Type)` pair.

**The orphan rule.** An instance `impl Trait for T` is **legal** iff at least one of the following
holds:

1. The `impl` appears in the same `nodule` (or `phylum`) as the **trait declaration**, or
2. The `impl` appears in the same `nodule` (or `phylum`) as the **type declaration** of `T`.

An `impl` that satisfies neither condition is an **orphan instance** and is an **explicit error**
at check time, with a diagnostic that names the `nodule` where the trait was declared and the
`nodule` where `T` was declared. This is the Rust orphan rule, stated against `nodule`/`phylum`
boundaries rather than crate boundaries.

**Rationale for the orphan rule (content-addressed framing, ADR-003).** Allowing orphan instances
would mean that adding a `nodule` dependency could silently change which instance the solver
picks for a `(Trait, Type)` pair — a kind of hash-identity instability. The orphan rule ensures
that the instance for `(Trait, Type)` is determined by the definition-site `nodule`, which is
content-addressed and stable.

**Overlapping instances.** Two instances **overlap** if there exists a type `T` for which both
instances would apply (after unification). Overlapping instances are **rejected** at check time,
even if one is "more specific" than the other. This is the Haskell 98 / Rust coherence rule, not
GHC's `OverlappingInstances` extension. The rationale: "more specific" requires a global
specificity order that has well-known edge cases (fragility, non-confluence); for a language where
**the dictionary is content-addressed**, a non-confluent resolution is a correctness failure, not
just a usability inconvenience.

**Open question (Q-coherence; §9):** whether a restricted form of **newtype-derived coherence
waivers** (a la Haskell `GeneralizedNewtypeDeriving`, Rust's `From`/`Into` blanket instances) is
sound under the orphan rule and content-addressed identity. This must be resolved before
ratification.

### 4.6 Repr-polymorphism (LR-5) — flagged research

**What LR-5 asks for.** RFC-0006 §4.2 states:

> LR-5: **Repr polymorphism**: abstracting over the paradigm (`∀ r: Repr-of-kind-K`) without
> violating S1 — swap insertion stays explicit even in generic code.

The goal is to write a single function that works for `Binary{n}`, `Ternary{m}`, `Dense{d, s}`,
and `VSA{A, n, sp}` values, without duplicating the body for each paradigm:

```mycelium
// Hypothetical: a function generic over the representation, not the width.
fn zero_pad<R: Repr>(x: R) -> R = …
```

**The S1 constraint (never-silent swap, RFC-0006 §4.1 S1).** S1 says that no elaboration step
may *insert* a `Swap` — swaps must be written explicitly at every layer. This interacts with Repr-
polymorphism in a non-trivial way: if `f<R>` is called at `R = Binary{8}` and then at `R =
Binary{16}`, the bodies are structurally the same but the representations differ. If the body
ever *converts* between `R` values of different widths, that conversion must be an explicit
`swap(…)` in the source — it cannot be inferred or inserted by the elaborator. This is strictly
more restrictive than Haskell's kind-system approach to representation-indexed types.

**Proposed restriction set (subject to soundness research — §9):**

1. **Kind stratification.** Introduce a `Repr` kind with sub-kinds `BinaryRepr`, `TernaryRepr`,
   `DenseRepr`, `VSARepr` and a super-kind `AnyRepr`. A type variable `R: Repr` may only be
   abstracted over values whose kind the context declares. Cross-paradigm generic code requires
   `R: AnyRepr` *and* must carry all `Swap` operations explicitly as method signatures in the
   trait.
2. **No implicit cross-paradigm operations in generic bodies.** A generic body that uses `x: R`
   may apply operations drawn from a `Repr`-bound trait, but may not apply operations whose type
   signature requires a *specific* paradigm (e.g., `binary_and` requires `Binary{_}` — it may not
   appear in an `R: AnyRepr` context unless the trait bound includes a `BinaryOps` sub-bound).
3. **Swap-explicitness in generic code (restates S1 at the polymorphic level).** Any representation
   change within a generic body must be a lexically-visible `swap(…, to: R2, policy: p)` call.
   An elaboration step that specializes `R = Binary{8}` and discovers that a `swap` is needed
   must reject with an **explicit elaboration error** naming the expression, the source and target
   types, and the missing `swap`. It may never silently insert one.
4. **Witness term for `Repr` kind.** A function `f<R: Repr>` receives a `Repr`-kind witness as
   part of its elaborated dictionary — a `ReprDesc` value (RFC-0001 §3.3's paradigm descriptor)
   that the body may inspect explicitly for dispatch on the paradigm, but only through a
   `match` on the witness, never through implicit coercion.

**Flagged novel.** Repr-polymorphism as specified here — kind-stratified, with S1 enforced
at the generic level — has no found mainstream analogue. Futhark's parametric types and Dex's
typed arrays are the closest relatives (T3.1/T3.3), but neither enforces a never-silent-swap
rule at the kind level. The restriction set in (1)–(4) is a **research proposal**, not a proven
design; §9 records the required survey and soundness sketch. Until that research lands, Repr-
polymorphic code should be **rejected with an explicit diagnostic** (`UnresolvedReprPolymorphism`)
rather than silently compiled under potentially unsound rules.

### 4.7 Guarantee-indexed trait methods (LR-6) — flagged research

**What LR-6 asks for.** RFC-0006 §4.2 states:

> LR-6: **Guarantee-indexed types**: the lattice tag as a type-level index, with `meet` as the
> composition law — the honesty rule moved into the type system.

RFC-0007 §4.3 establishes the v0 stage-0 semantics: the guarantee index `τ @ g` is checked
*dynamically* against `Meta`; the static graded judgment is stage-1, a revision of RFC-0007
(the forthcoming grading RFC). This section specifies how LR-6 interacts with traits at the
surface level — what "guarantee-polymorphic" means in practice.

**The key use case.** A trait method that processes values should return a result whose guarantee
strength is determined by the *inputs'* strengths via the `meet` of the guarantee lattice
(`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`):

```mycelium
// A trait for values that can be combined.
// The guarantee of the result is the meet of the input guarantees.
trait Combine<A> {
    fn combine(x: A @ G1, y: A @ G2) -> A @ meet(G1, G2)
}
```

Here `G1` and `G2` are **guarantee-level variables** — polymorphic over the four-point lattice.
The return guarantee is `meet(G1, G2)`, computed statically (or at the stage-0 runtime-check
level if static grading has not yet landed).

**The surface form.** Guarantee variables are bound in the function's type parameter list
alongside type variables, using the `@` sigil from the existing `type_ref` grammar (
`base_type ('@' strength)?`). Full guarantee-polymorphic methods require the static graded
judgment from the grading RFC; at stage 0 the dynamic `Meta` check is the fallback.

**Three levels of LR-6 integration (staged):**

1. **Stage 0 (v0, currently ratified, RFC-0007 §4.3).** The guarantee annotation `τ @ g` exists
   in the type language; it is checked *dynamically*. A trait method may *declare* its guarantee
   signature (as a documented interface contract — `Declared` in the honesty lattice), but the
   type checker does not statically verify it. This is honest (VR-5): the declaration says what
   the implementer intends; a future stage-1 check will verify it.
2. **Stage 1 (grading RFC, forthcoming).** The static graded coeffect judgment (RFC-0006 §8 Q3;
   FlowCaml-style over the 4-chain) verifies that `combine`'s implementation actually respects
   `meet(G1, G2)`. At this stage, the guarantee annotation in a trait signature becomes a
   *checked* contract, not merely a declaration.
3. **Stage 2 (fully guarantee-polymorphic traits, research).** Guarantee variables `G1, G2` are
   proper type-level variables, not placeholders; `meet` is a computable type-level function on
   the 4-point lattice. This is the full LR-6 vision: a function demanding `Exact` input is a
   *type error* to call with `Declared`. This requires the grading RFC's type theory to be
   extended to handle polymorphic grading — which is the **flagged-novel** piece (RFC-0006 §8 Q3:
   "grading + runtime certificates has no found precedent — needs its own soundness argument").

**For this Draft.** Trait method signatures may carry `@ g` annotations on parameters and return
types. The elaborator treats guarantee variables as *deferred*: if `G1`/`G2` appear in a trait
signature, the elaborated dictionary field carries the declared signature with a `Declared` tag on
the guarantee binding. The stage-1 grading RFC will upgrade these to checked `Proven` bindings.
No new L1 node is required; the `@ g` annotation is already part of `type_ref` in the grammar.

**Guarantee-polymorphism and dictionary-passing.** Under dictionary-passing, the guarantee
variables in a trait signature become *type-level* fields of the dictionary type. For a fully
polymorphic `combine<A, G1, G2>(x: A @ G1, y: A @ G2) -> A @ meet(G1, G2)`, the dictionary for
`Combine<A>` carries the function at the polymorphic guarantee type. At a call site where `G1 =
Empirical` and `G2 = Declared` are statically known (stage 1+), the dictionary field is
specialized to return `A @ Declared` (the meet). This is structurally a type-level computation
on a 4-point totally-ordered lattice — decidable and cheap. The detail of how to represent
`meet(G1, G2)` in the type language without adding a new type-former is part of the grading RFC's
brief; this RFC records the interaction and defers the mechanism.

### 4.8 `grow` for derived instances (DN-03 §1)

DN-03 reserves `grow` for a derive-like mechanism: `grow Debug for T` generates an instance
`impl Debug for T` automatically from `T`'s structure. The `grow` keyword is already reserved
(DN-03 §2). This RFC specifies its relationship to trait instances:

- `grow Trait for T` is sugar: it signals that the instance should be *derived* from `T`'s
  structure according to a derivation rule registered for `Trait`. The derivation rule is
  itself a function from `T`'s `InstanceDecl` structure to a complete set of method bodies.
- The generated instance is elaborated to a normal `impl_item` (an `InstanceDecl` in the
  registry) with a content-addressed hash — it is not a second-class citizen. The fact that it
  was generated by `grow` is metadata in the registry entry, inspectable via `EXPLAIN`.
- The coherence rules (§4.5) apply equally to `grow`-derived instances: a `grow`-derived orphan
  is still an error.
- Derivation rules are not specified in this RFC (they require the macro/metaprogramming story);
  this RFC records only that `grow … for …` desugars to an `impl … for …` form and that the
  DN-03 split (`impl` = hand-written, `grow` = derived) is the normative discipline.

### 4.9 Typing judgments for v1 (polymorphic)

The v0 typing judgment `Γ ⊢ e : τ` (RFC-0007 §4.4) is monomorphic. This RFC extends it to
handle trait bounds. The full polymorphic judgment is stratified:

**Type scheme.** A polymorphic definition has a *type scheme* `∀ ā. (C) ⇒ τ` where `ā` are the
type variables and `C` is the set of trait constraints. At the *elaboration* level (L1), this
becomes a function that takes one dictionary argument per constraint.

**Instance resolution judgment.** `Σ, Π, Γ ⊢ inst(Trait, τ) ↝ dict` ("in the registry `Σ`,
with the typing context `Γ`, the instance of `Trait` at type `τ` resolves to dictionary term
`dict`"). Resolution is *deterministic*: global coherence (§4.5) ensures at most one instance per
`(Trait, Type)` pair exists in `Σ`.

**Extended T-App-Inst rule.** When a bounded-generic function is applied:

```text
 T-App-Inst   Γ ⊢ f : ∀ T: Trait. τ₁→τ₂
              Γ ⊢ a : τ₁[T ↦ C]
              Σ, Γ ⊢ inst(Trait, C) ↝ dict_Trait_C
              ──────────────────────────────────────
              Γ ⊢ f(a) : τ₂[T ↦ C]
              (elaborated to: App(App(f_elab, dict_Trait_C), a))
```

The instance-resolution side condition (`inst(Trait, C) ↝ dict`) is the elaborator's job — it
produces the dictionary term, which then appears explicitly in the elaborated L1 term. If no
instance exists, `inst(Trait, C)` fails with an explicit diagnostic naming the missing
`(Trait, Type)` pair.

**Depth-guarded elaboration (banked guard 4 from CONTRIBUTING/dev-workflow).** The elaborator
that performs instance resolution and dictionary-passing translation recurses on type structure.
It must carry an explicit depth budget and return a clean error past it — never lean on the host
stack. This is especially relevant for super-trait chains and multi-parameter bounds.

## 5. Drawbacks

- **Dictionary indirection.** Every method call through a trait becomes one indirection. For
  tight inner loops (e.g., an `Eq` check inside a sort) this is a measurable cost. The
  monomorphizing specialization pass (§4.4) is the mitigation; it must be a later, explicit
  decision, not a silent elaboration shortcut that defeats S4's inspectability.
- **Coherence strictness.** Rejecting all overlapping instances and all orphan instances rules
  out some Haskell-style trickery (`OverlappingInstances`, `IncoherentInstances`) that
  experienced users sometimes want. The price is predictability and content-addressed stability —
  a trade that is correct for this substrate.
- **Repr-polymorphism is deferred in practice.** Until the §9 soundness research is complete,
  Repr-polymorphic code is rejected with an explicit diagnostic. This means the LR-5 goal is not
  fully realized at this Draft stage; it is architecturally scaffolded but not executable.
- **Guarantee polymorphism is staged.** Stage-0 trait methods with guarantee annotations are
  Declared-quality contracts; stage-1 verification requires the grading RFC. A library author
  writing guarantee-polymorphic code must accept that their guarantees are `Declared` until the
  grading RFC lands.

## 6. Rationale and alternatives

### 6.1 Why not type classes as in Haskell 98 (exactly)?

Haskell 98's type classes are the most directly analogous design. The deviations here are:

- **Stricter coherence.** Haskell 98 is already strict; GHC extensions relax it. We do not adopt
  the extensions (§4.5 rationale: content-addressing demands global uniqueness).
- **Dictionary-passing is explicit and inspectable (S4).** In GHC, the dictionary is an
  implementation detail. In Mycelium it is observable via the stage-dump channel — this is a
  deliberate S4 enhancement.
- **LR-5 and LR-6.** No Haskell/GHC equivalent; these are the "beyond Rust" novelties.

### 6.2 Why not Rust traits exactly?

Rust traits are the closest surface analogue. The differences:

- **No ownership/borrowing.** Mycelium is value-semantics; the `&T`, `&mut T`, `Box<T>`, and
  lifetime-parameterized trait stories do not apply (RFC-0006 §4.2 LR-8, Q5 position).
- **No trait objects without a type parameter.** Rust's `dyn Trait` (vtable dispatch) is
  dictionary-passing at runtime. Mycelium's dictionary-passing is always the elaboration; there
  is no separate `impl Trait` / `dyn Trait` split.
- **Coherence is the same.** Rust's orphan rule is the direct inspiration for §4.5.
- **Guarantee and Repr polymorphism** have no Rust equivalent.

### 6.3 Why not Scala-style implicits or Agda instance arguments?

Both are more powerful and less predictable than a global-coherence typeclass system. For a
substrate whose identity story is content-addressed hashing, unpredictable implicit search is
directly at odds with the design (ADR-003). The coherence requirement is the design, not a
limitation.

## 7. Prior art

- **GHC / Haskell 98 typeclasses** (Wadler & Blott 1989; Haskell Report): the source of
  dictionary-passing elaboration, coherence, and the type-scheme judgment. The deviations are
  specified in §6.
- **Rust traits** (RFC 0000, RFC 0195, RFC 1210 "specialization" — the last being exactly what
  §4.5 rejects): the source of the orphan rule framing and `impl Trait for T` syntax. Rust's
  `dyn Trait` confirms dictionary-passing as a sound representation.
- **GHC Core** (Tolmach & Cheung, TLDI 2007): confirms that dictionary-passing compiles to the
  same small typed lambda calculus this RFC targets; zero new trusted nodes needed.
- **ML modules** (SML/NJ; OCaml first-class modules): an alternative to typeclasses for
  parametric abstraction over capabilities; more expressive but harder to infer. Reserved as
  the RFC-0006 LR-3 modules story, separate from traits.
- **Coq type classes** (Sozeau & Oury 2008): typeclass resolution as proof search; unification-
  based, non-coherent in general. Ruled out by §4.5.
- **Dex / Futhark**: typed array calculi with constrained polymorphism. The closest relatives for
  LR-5 (Repr-polymorphism, kind-stratified); both lack the never-silent-swap rule.
- **F* / Liquid Haskell**: refinement-indexed types, nearest relatives for LR-6 stage-2. The
  graded-coeffect approach (RFC-0006 §8 Q3; DCC; FlowCaml) is the mechanism position already
  recorded for the grading RFC.
- **Unison** (Chiusano et al.): content-addressed definitions, names-as-metadata. Confirms
  that the ADR-003 story is sustainable at language scale; the identity stability argument in
  §4.5's orphan rule rationale is drawn directly from Unison's coherence story.

## 8. Unresolved questions

- **Q-coherence.** Is a restricted form of newtype-derived coherence waivers (Haskell
  `GeneralizedNewtypeDeriving` / Rust `From`/`Into` blanket instances) sound under the orphan
  rule and ADR-003 content-addressed identity? Must be resolved before ratification. (→ §9
  research prompt.)
- **Q-reprpoly.** What is the **minimal restriction set** for Repr-polymorphism (LR-5/T3.3) that
  is sound under S1 (never-silent swap), kind-stratified as in §4.6? The restriction set in §4.6
  is a research proposal; the soundness argument must be worked out. (→ §9 research prompt.)
- **Q-grading-interact.** How exactly do guarantee variables (`G1`, `G2`, `meet(G1, G2)`) appear
  in the dictionary type for stage-1 and stage-2? The grading RFC specifies the mechanism;
  this RFC records the interaction shape. (→ grading RFC / RFC-0018.)
- **Q-derivation.** What is the specification of `grow … for …` derivation rules? (→ macro/
  metaprogramming RFC, deferred; this RFC commits only the split `impl`/`grow` and that
  `grow`-derived instances are normal `InstanceDecl` registry entries.)
- **Q-multi-param.** Are multi-parameter traits (`trait Coerce<A, B>`) in scope for v1, or
  deferred? They add complication to the orphan rule (the "defining nodule" is no longer
  unambiguous). **Position (not yet ratified):** defer multi-parameter traits to a v2 revision
  of this RFC; v1 covers single-parameter traits only, which covers the bulk of the use-case
  space.
- **Q-associated-types.** Haskell/Rust both use associated types inside trait bodies (`type
  Output = …`) to avoid multi-parameter traits in practice. This RFC defers associated types;
  they are a natural extension once single-parameter coherence is established.

## 9. Research prompts (open — must be surveyed before ratification)

**Status: DISCHARGED — `research/10-traits-coherence-repr-polymorphism-RECORD.md` (2026-06-18).**
Both prompts below are executed (RP-3). **R1:** the coherence mechanism is Rust-style orphan rule +
global uniqueness + reject-overlap (the only mechanism consistent with content-addressed identity),
with a total/deterministic/hash-stable resolution theorem + sketch (record T10.2–T10.3); Q-coherence
is resolved **reject newtype waivers in v1** (safe admission needs a *roles* mechanism — Weirich et
al.; T10.4). **R2:** the Repr-polymorphism restriction set ("no paradigm-specific `Op` on a
Repr-abstract argument; passthrough / trait-interface / lexical-`Swap` only") is **locally checkable**
and **S1-preserving**, with a theorem + sketch grounded as the dual of GHC levity polymorphism
(T10.5–T10.7). Q-multi-param and Q-associated-types are recommended **deferred to v2** (T10.8).
**Honest scope:** both soundness results are tagged **Declared-with-argument** (not machine-checked —
VR-5; mechanization is the future `Proven` basis). The maintainer's append-only decisions (adopt the
mechanism + restriction set; the deferrals) remain the ratification gate; the *research* gating them
is now closed. The original prompts are retained below verbatim (append-only).

---

These are the two pieces of this RFC that have no mainstream analogue and require original
soundness arguments. They are marked **[research]** in the normative sections above; a
ratification vote should be conditioned on their completion.

---

**[R1] Coherence mechanism for dictionary-passing under content-addressed identity (Q-coherence)**

*Background.* The §4.5 orphan rule and global-uniqueness requirement are justified by ADR-003
content-addressing: two definitions with the same `(Trait, Type)` hash collision are the same
definition, so they must have the same semantics. This is stricter than Haskell's coherence
(which is about determinism of instance selection, not hash-collision semantics) and Rust's
(which is about crate-boundary isolation, not content-addressed identity).

*Research question.* Design and verify a **formal coherence criterion** for the Mycelium registry
model:
1. State the coherence invariant: "for any well-formed registry `Σ`, instance resolution
   `inst(Trait, τ) ↝ dict` is *total*, *deterministic*, and *hash-stable*" (the same `(Trait, τ)`
   pair always resolves to the same `dict` hash, regardless of the order in which `InstanceDecl`
   entries were added).
2. Prove that the orphan rule (§4.5) is **sufficient** for this invariant under the content-
   addressed registry model.
3. Investigate whether a restricted newtype-derived coherence waiver (Q-coherence) can be
   admitted without breaking hash-stability — specifically whether `GeneralizedNewtypeDeriving`-
   style derivation, which copies a dictionary from the wrapped type, preserves or violates
   hash-stability.
4. Survey the Rust "specialization" RFC 1210 failure modes and confirm that §4.5's rule avoids
   them.

*Output expected.* A worked soundness sketch (informal proof or proof outline), plus a verdict on
Q-coherence (admit restricted waivers / reject all waivers / defer with conditions).

---

**[R2] Repr-polymorphism restriction set and soundness under S1 (LR-5 / T3.3 / Q-reprpoly)**

*Background.* RFC-0006 §4.2 LR-5 asks for `∀ r: Repr. f(x: r) -> r` with S1 (never-silent
swap) enforced at the generic level. RFC-0006 §8 notes "restriction set researched (T3.3)";
T3.1's survey found Dex and Futhark as the closest relatives, but neither enforces a
never-silent-swap rule at the kind level. The restriction set in §4.6 is a design proposal, not
a proven result.

*Research question.* Survey and extend:
1. How do Dex's `Type` kind, Futhark's type system for arrays, and similar typed array / typed
   effects systems handle abstraction over the *representation* of data, and what restrictions
   do they impose?
2. Can the §4.6 restriction set (kind stratification, no implicit cross-paradigm ops, swap-
   explicitness restated at the generic level, `ReprDesc` witness term) be shown **sound** in
   the following sense: "every well-typed Repr-polymorphic term, specialized to any concrete
   `Repr`, produces an L1 term that is well-typed under RFC-0007 §4.4's monomorphic judgment,
   and contains no `Swap` node that was not lexically present in the generic source"?
3. Identify any tightening or relaxation of the §4.6 restrictions that the soundness argument
   requires.
4. Produce a worked example: a generic `fn map_bits<R: BinaryRepr>(x: R, f: Binary{1} ->
   Binary{1}) -> R` — show its elaborated form, its S1-compliance, and what a violation
   (e.g., attempting to `swap` inside the generic body without a lexical `swap` node) looks
   like as a check error.

*Output expected.* A survey summary (≤ 2 pages), a soundness sketch for the §4.6 restriction set
(or a revised set if the sketch fails), and the worked elaboration example. Until this research
lands, Repr-polymorphic terms are rejected with `UnresolvedReprPolymorphism` (§4.6).

---

## 10. Ratification scope (this Draft — not yet ratified)

> **RATIFIED (2026-06-18, maintainer).** All gates below are discharged. **R1/R2** are executed in
> `research/10-traits-coherence-repr-polymorphism-RECORD.md`; the maintainer **adopts** the
> orphan-rule + global-uniqueness + reject-overlap coherence mechanism and the §4.6 Repr-polymorphism
> restriction set (both now normative), **rejects** newtype-derived coherence waivers in v1
> (Q-coherence — safe admission needs a roles mechanism, deferred with associated types), and
> **defers** multi-parameter traits and associated types to v2 (Q-multi-param, Q-associated-types).
> Q-grading-interact is settled by the now-Accepted RFC-0018. The "explicitly NOT ratified" list
> below is **superseded** by this ratification for the coherence proof and the Repr-polymorphism
> restriction set (now adopted as Declared-with-argument designs); multi-param and associated types
> remain out of scope (deferred to v2), as stated. *Honesty (VR-5):* acceptance is of the **design**;
> the soundness *claims* stay **Declared-with-argument** (not machine-checked). The original Draft
> ratification criteria are retained below verbatim (append-only).

This RFC is **Draft**. The maintainer must ratify it (move to Accepted) based on:

1. Completion of research prompts R1 (coherence soundness) and R2 (Repr-polymorphism), which are
   the two pieces with flagged-novel status.
2. Resolution of Q-multi-param (defer multi-parameter traits to v2?) and Q-associated-types
   (defer? or include in v1?).
3. Confirmation of the elaboration strategy (dictionary-passing, §4.4 — §4.3 normative content)
   against the non-normative `crates/mycelium-l1` prototype once polymorphic elaboration is
   implemented there.
4. Confirmation that the grammar extensions (§4.1) do not break any existing conformance corpus
   programs in `docs/spec/grammar/conformance/accept/`.

**What is committed by this Draft (not ratified, but design-direction stable):**

- The `impl … for …` keyword choice for trait instances (§3.2, DN-09 §3.2, DN-03 §1).
- Dictionary-passing as the elaboration strategy (§4.3/§4.4 — preferred over monomorphization).
- The coherence/orphan rule design (§4.5) — motivated by ADR-003; the formal proof is in §9 R1.
- The stage-0 / stage-1 / stage-2 LR-6 integration ladder (§4.7) — coordinated with the grading
  RFC.
- The `grow … for …` / `impl … for …` split (§4.8; DN-03 §1).
- That the kernel node budget (RFC-0007 §4.1) does not grow.

**What is explicitly NOT ratified and must not be treated as decided:**

- The restriction set for Repr-polymorphism (§4.6 / Q-reprpoly / R2 research prompt).
- The formal coherence proof (§4.5 / Q-coherence / R1 research prompt).
- Multi-parameter traits (Q-multi-param — position is "defer", not yet ratified).
- Associated types (Q-associated-types — deferred; no design proposed in this Draft).
- The exact typing judgment for guarantee-polymorphic trait methods at stage 1+ (Q-grading-
  interact — belongs to the grading RFC).

---

## Meta — changelog

- **2026-06-18 — Draft, initial authoring.** First draft of the trait / parametric-polymorphism
  layer (LR-2), resolving RFC-0007 §4.4's explicit deferral ("polymorphism/traits deliberately
  out of v0 — its own later RFC"). Specifies surface forms (`impl … for …`, bounded generics,
  `grow … for …`), dictionary-passing elaboration to L1 (zero new kernel nodes), coherence +
  orphan rule grounded in ADR-003, and the two Mycelium-specific novelties (Repr-polymorphism
  LR-5 and guarantee-indexed trait methods LR-6) with their research prompts. Status: **Draft**;
  ratification conditioned on R1/R2 research completion.
- **2026-06-18 — §9 research prompts R1+R2 DISCHARGED (RP-3).** Executed in
  `research/10-traits-coherence-repr-polymorphism-RECORD.md`: the coherence mechanism +
  total/deterministic/hash-stable resolution theorem (T10.2–T10.3), the Q-coherence verdict
  (reject newtype waivers in v1 — needs roles, T10.4), the Repr-polymorphism restriction set
  (locally checkable, T10.5) + its S1-preservation theorem + sketch (T10.6) grounded as the dual of
  GHC levity polymorphism (T10.7), and the recommend-defer verdicts for multi-param/associated types
  (T10.8). Both soundness results tagged **Declared-with-argument** (not machine-checked — VR-5).
  **Status stays Draft:** the §10 research gate (R1/R2) is now discharged; the remaining gates are
  the maintainer's append-only decisions (adopt the recommended mechanism + restriction set; record
  the deferrals). No normative rule changed — this is the grounding the decision was waiting on.
  Append-only.
- **2026-06-18 — ACCEPTED (maintainer ratification).** All §10 gates discharged. The maintainer
  **adopts** (now normative): the coherence mechanism = orphan rule + global uniqueness +
  reject-overlap (R1/`research/10` T10.2–T10.3); the §4.6 **Repr-polymorphism restriction set** ("no
  paradigm-specific `Op` on a Repr-abstract argument; passthrough / trait-interface / lexical-`Swap`
  only" — locally checkable, S1-preserving, R2/T10.5–T10.6) — so the §4.6 "reject until the research
  lands" posture is now lifted: code obeying the restriction set is admitted, code violating it gets
  `UnresolvedReprPolymorphism`. **Rejects** newtype-derived coherence waivers in v1 (Q-coherence —
  needs a roles mechanism, deferred with associated types). **Defers** multi-parameter traits and
  associated types to v2 (Q-multi-param/Q-associated-types). Q-grading-interact is settled by the
  now-Accepted RFC-0018. Dictionary-passing elaboration confirmed; kernel node budget unchanged
  (KC-3). *Honesty (VR-5):* acceptance is of the **design**; the coherence and S1-preservation
  *claims* stay **Declared-with-argument** (not machine-checked) — mechanization remains the basis for
  a future `Proven` upgrade. Append-only.
