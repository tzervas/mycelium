# Design Note DN-53 — Object-Composition Sugar & Granular Item-Level Visibility

| Field | Value |
|---|---|
| **Note** | DN-53 |
| **Status** | **Proposed** (2026-06-27; design DN; enacts no code) |
| **Task** | M-811 |
| **Feeds** | the **object-composition surface** (DN-37 Q1 follow-on: honest sugar for `type` + traits + `via`-delegation, transparent about what it really is) and **granular item-level `pub`** (DN-37 Q4 follow-on: `pub` at item granularity, superseding the nodule-only model); both are pre-implementation design gated on the grammar wave (RFC-0037 / DN-31 / RFC-0030 / epic #27) for their surface forms. |
| **Date** | June 27, 2026 |
| **Decides** | *Nothing normatively — advisory + design-direction capture.* Records (A) a candidate object-composition sugar keyword (`object`) that passes the DN-02 three-test naming gate (T-map / T-illuminate / T-learn), its desugaring discipline (lowers to `type` + `impl` + `via`, no kernel growth KC-3, `reveal`-able per DN-38 §5), and the transparency invariant (the sugar TEACHES the honest model, not an OOP veneer); and (B) the granular `pub` design: item-level `pub` on `fn` / trait method / `let` / `type` (and fields, DEFERRED — §B.5), composition with the existing nodule-level model and RFC-0019 §4.5 orphan/coherence, and the never-silent access refusal (G2). |

> **Posture (transparency rule / VR-5 / G2).** This note **records design proposals**, not ratified
> decisions. It enacts no code, moves no RFC/ADR/DN status (house rule #3). Every claim is tagged
> at its honest strength: the **existing** built items (`type`, `impl`, `pub`/`Vis`, coherence) are
> `Exact`/`Empirical` (cited to source); the **proposed** sugar forms + desugaring rules + visibility
> semantics are **`Declared`-with-argument** (the Rust precedent and the DN-37/38 reasoning are
> `Empirical`/`Proven`-at-source, but their Mycelium surface mappings are design proposals, not
> ratified). No tag is upgraded past its basis (VR-5). Every gap is named (G2).

---

## §A Object-Composition Sugar (DN-37 Q1 follow-on)

### §A.1 The design problem

DN-37 Q1 ruled **no `class` keyword**: objects are emulated with `type` + traits + `via`-delegation.
The ruling flagged a follow-on design task: *"find the right ergonomic sugar for object-composition
that is clear and transparent about what it really is (trait + static composition over a value),
without borrowing OOP-`class` connotations."*

The concrete ergonomic gap today: defining a record type with several trait implementations and
`via`-delegation forwarders requires four separate top-level constructs (`type`, three `impl` blocks,
`via` declarations), with no syntactic grouping cue that they belong together as a coherent unit.
The programmer must manually maintain the coherence of the ensemble. An object-composition sugar
closes this grouping gap without changing any semantics.

**What the sugar must NOT do.** It must not imply:

- **mutable self** (Mycelium values are immutable; mutation = functional update; LR-8),
- **inheritance or implementation sharing** (no super-impls; composition only),
- **dynamic dispatch** (monomorphization is the only dispatch; `dyn` is deferred/greenfield), or
- **open-closed extension** (closed sum on the ADT horn of Cook's duality; DN-37 §1).

Any keyword that carries any of these connotations in the ambient programming-language ecosystem
fails the T-map gate (DN-02 §1) and is **disqualified**, exactly as `class` was.

### §A.2 The three-test gate (DN-02 §1) applied to candidate keywords

The DN-02 naming law: *"theme where the metaphor is accurate and illuminating; keep conventional
where a borrowed term is clearer to learn and read."* The gate:

- **T-map (fidelity).** Does the term map accurately to the behavior — no false implications?
- **T-illuminate (teaching value).** Does it teach something the conventional term does not?
- **T-learn (dual readability).** Does a conventional term aid learnability for both human and
  machine readers more than theming would?

**Candidate: `class`** — REJECTED (DN-37 Q1 ruling). T-map fail: `class` universally implies
mutable self, inheritance, and dynamic dispatch — all absent. The false implication is worse than
any ergonomic benefit; rejected unconditionally.

**Candidate: `struct`** — REJECTED. T-map fail: `struct` implies a Rust/C named-field aggregate
with no associated behavior in the bundle. Mycelium's `type` is already the data keyword; `struct`
adds nothing but confusion with the external precedent (Rust `struct` has no `impl` grouping).
T-illuminate fail: teaches nothing about the composition ensemble.

**Candidate: `object`** — PASSES the gate. Detailed reasoning:

- T-map: `object` in PL tradition means "a value with behavior associated" — which is exactly what
  the sugar declares (a `type` value plus its trait implementations, grouped). Critically, `object`
  does **not** imply mutable self, inheritance, or dynamic dispatch in isolation — those are
  properties of OOP *classes*, not of the abstract notion of "object." The sugar's desugaring makes
  the honest model explicit (§A.3), so the term's accurate reading is reinforced by `reveal`.
  Caveat: some programmers will initially project OOP semantics onto `object`; this is **addressed
  by design** — the syntax inside the block explicitly shows `via`-delegation (not inheritance) and
  immutable-value update (not `self` mutation), so any OOP projection is corrected at the point of
  use. Tag: `Declared`.
- T-illuminate: `object` teaches "this is a coherent unit of data + behavior, designed to be used
  as a value-level abstraction" — more than `type` alone. It groups the ensemble visually and
  semantically, making the intent clear without implying an OOP implementation model.
- T-learn: `object` is a near-universal PL term (Python `object`, Java `Object`, JavaScript
  `object` literal; Go is the counterexample that proves the rule). Machine readers (LLMs) will
  parse `object` as "data plus behavior" — the correct read — and not as "a mutable class instance"
  if the body they see contains `via`-delegation and `impl` blocks. The dual-readability test is
  met: the term is familiar without importing incorrect semantics.

**Candidate: `form`** — REJECTED. T-map: `form` is too vague and carries no behavioral implication.
T-illuminate fail: does not teach "this has associated behavior." T-learn fail: unfamiliar as a
PL keyword in this role; machine readers would not parse it correctly.

**Candidate: `record`** — REJECTED for this role. T-map fail for the *ensemble*: `record` implies
data-only (a product type), not the grouping of data + behavior impls. It is an accurate term for
the data portion alone, but not for the composition sugar. (Reserving `record` as a possible future
alias for a single-constructor `type` is orthogonal and not blocked by this note.)

**Recommendation: `object`.** Passes all three tests; is transparent about composition over a
value; is corrected-in-context by its own syntax; and does not require coining an
unfamiliar term. Tag: `Declared`. The recommendation is flagged for maintainer ratification (§A.6 Q1).

### §A.3 Desugaring and the lowering law

The `object` sugar is **pure frontend desugaring** — it lowers to `type` + `impl` + `via` with no
semantic addition and no kernel growth (KC-3). The DN-38 lowering law applies: the lowering is
small, IL-grammar-checked, semantics-preserving (`observe(surface) == observe(lower(surface))`),
and `reveal`-able to its real L0 form (DN-38 §5).

#### §A.3.1 Surface form (Declared)

```
object Name<params> {
    // 1. The data fields (positional; same as type).
    data Field1Type, Field2Type, …

    // 2. Optional: delegation of an interface to an embedded field.
    via FieldN : SomeTrait

    // 3. Trait implementations (same body as a standalone impl block).
    impl SomeTrait {
        fn method(…) -> … = …
    }

    // 4. Inherent methods (standalone fns scoped to this object; sugar for
    //    a trait-less impl, listed as free fns associated with the type).
    fn helper(self: Name<params>, …) -> … = …
}
```

Where `data` inside an `object` block introduces the record fields (positional, same semantics as a
`type` constructor). `via FieldN : SomeTrait` generates forwarding methods from the value at
position `FieldN` for the named trait (DN-37 §3.3, the `~>` operator). `impl` blocks inside the
`object` are syntactic sugar for standalone `impl SomeTrait for Name<params>` declarations.
Inherent `fn` items inside the block are syntactic sugar for free top-level functions whose first
value parameter is `self: Name<params>` — they do not introduce a new language concept; they are
grouped for readability.

#### §A.3.2 Lowering (Declared)

The desugaring is a **one-to-one structural rewrite** with no information loss:

```
// object Foo { data T1, T2; via f1: Iface; impl Bar { fn m = … } }
// ↓ lowers to (reveal produces exactly this):

type Foo<> = Mk(T1, T2)              // the data constructor

// via-delegation generates forwarding methods:
// fn iface_method(self: Foo) -> … = self.f1.iface_method()   [for each method of Iface]

impl Iface for Foo {                  // generated forwarder block
    fn iface_method(self: Foo) -> … = (match self { Mk(f1, _) = f1 }).iface_method()
}

impl Bar for Foo {                    // the explicit impl, lifted verbatim
    fn m = …
}
```

Every generated forwarder is a **concrete, `EXPLAIN`-able function** — `reveal object Foo` shows
the exact L0 term, with no hidden virtual dispatch, no mutable `self`, no fixpoint. The
"no late binding into the wrapper" caveat (DN-37 §3.3 — Kotlin's explicit guarantee, Go embedding's
lesson) is **the honest behavior** under value semantics: a forwarded method's receiver is the inner
value, not the outer `object` — any other reading would require the cyclic self-reference LR-9
excludes.

#### §A.3.3 What `reveal` shows (Declared)

Running `reveal` on an `object` definition expands it to:

1. The `type` declaration for the data constructor.
2. Each `impl` block (explicitly attributed to the named trait + this type).
3. Each generated forwarding function (from `via`), labeled `[generated: via FieldN : TraitName]`
   so the provenance is never silent (G2).
4. Each inherent `fn`, lifted to top-level with the explicit `self` parameter visible.

The round-trip discipline (DN-38 §5, certified mode) requires `delaborate(reveal(object Foo)) ≡ object Foo`
— the desugaring is **injective**: no two `object` definitions produce the same lowered L0 set, and
the lowered set determines the source uniquely. This is `Declared` (follows from the structural
one-to-one rewrite above; the certified-mode round-trip checker enforces it at implementation time).

#### §A.3.4 Transparency invariant

The sugar TEACHES the honest model:

- **No mutable self.** The `fn helper(self: Name, …)` form makes the value-in argument explicit —
  self is a value parameter, not a mutable reference. Mutation is always a new value returned.
- **No inheritance.** The `via` keyword is explicitly delegation, not subtype inheritance. The
  generated forwarder is visible (T-map: `via` is a preposition = "through this field" — the honest
  read of static by-value forwarding, as DN-37 §5 + DN-38 §8.1 settled).
- **No dynamic dispatch.** The `impl` block inside `object` is syntactically identical to a
  standalone `impl SomeTrait for Name` — the programmer sees the same static dispatch they would
  write by hand. Monomorphization is the implicit path (the only dispatch in v1).
- **Content-addressed equality.** The lowered `type` inherits the structural identity of all
  `type` declarations (ADR-003); `object` adds nothing to the identity model.

### §A.4 Scope: what the sugar covers

The `object` sugar addresses the **common composition patterns** (DN-37 Q1): a named record type
with a fixed set of trait implementations and `via`-delegation forwarders. It does **not** address:

- **Dynamic dispatch / record-of-closures objects** (DN-37 §3.7 / §8 Q3) — greenfield, gated on
  the `FieldSpec`-abstract-function-field ADR. Not expressed by this sugar.
- **Open-recursion / inheritance** — excluded by design (DN-37 §3.7; LR-8/LR-9 make `fix`-over-`self`
  unbuildable in safe code).
- **Decorator `@` wrapping** (DN-37 §3.4) — a separate value-to-value wrapping pattern; not part
  of the composition sugar (it is a transform `T -> T`, not a definition form).
- **Embedding with wildcard promotion** (DN-37 §3.5) — defer; the `via` form is explicit per-trait.
  Wildcard promotion can be designed as `via Field : *` at the grammar wave if wanted.

Tag: `Declared`.

### §A.5 Grounding

- **DN-37 §3.1–§3.5** — the inheritance-emulation menu (ranked by composition-alignment); default
  methods, super-traits, `~>`/`via`, decorator, embedding. The `object` sugar encapsulates items 1–3
  (where used). `Empirical`/`Proven` at source; Mycelium mapping `Declared`.
- **DN-38 §2 (Lowering Law), §4 (generative lowering), §5 (`reveal`)** — the desugaring discipline;
  KC-3 (no kernel growth). `Exact` for the built lowerings; `Declared` for the new sugar's extension.
- **DN-02 §1 (naming gate)** — the three tests applied above. `Declared`.
- **RFC-0019 §4.1 (traits + impls, `Empirical`)** — the static dispatch the `impl` blocks inside
  `object` lower to, unchanged.
- **DN-37 §6 (the four standing constraints)** — value-semantics, acyclic, honest/EXPLAIN-able
  dispatch, content-addressed equality. The sugar inherits all four without exception.
- **Prior art (Empirical at source):** Kotlin `class C(b: Base): Iface by b` (the `by`/`via`
  delegation precedent; docs: *"members overridden in this way do not get called from the members of
  the delegate object"* — the no-late-binding guarantee); Go embedding (*"the receiver of the method
  is the inner type, not the outer one"* — Effective Go); Rust `#[derive]` (GOOD generative lowering —
  produces inspectable `impl` blocks, not opaque mutation; the anti-pattern is Lombok per DN-38 §4).

### §A.6 Open sub-questions (FLAGGED — maintainer ratification)

- **(Q1) Keyword recommendation — `object` or another form?** This note recommends `object`; the
  maintainer may rule a different keyword or prefer a keyword-less grouping form (e.g. an explicit
  `group` / `compose` / `bundle` annotation block, or simply encouraging prose-of-three-constructs
  with a convention). The three-test analysis above is the basis; a different ruling supersedes it.
  `Declared`. **FLAG: awaiting maintainer ratification on the recommended keyword.**
- **(Q2) `data` sub-keyword vs positional in the `object` body.** The draft uses `data` as a
  sub-keyword to introduce the positional fields. Alternative: the `object` block uses the same
  constructor syntax as `type` (`Name(T1, T2, …)` as the first item). The positional vs named-field
  question for `object` fields is the same as the wider field-naming question for `type` (deferred
  greenfield in DN-37 §2.2). This note defers it to that work. `Declared`.
- **(Q3) Inherent methods — a new concept or convention only?** The draft treats `fn helper(self: Name, …)`
  inside `object` as sugar for a top-level free function. If the language later adopts a more
  principled "method receiver" convention (a type-qualified `Self` or a named-first-param pattern),
  the `object` sugar should align with it. Deferred to the method-syntax work. `Declared`.
- **(Q4) Sequencing vs the grammar wave.** The `object` surface form rides RFC-0037 (the follow-on
  grammar RFC to DN-31/RFC-0030/epic #27). No surface form is normatively fixed by this note; the
  semantics and desugaring discipline are the deliverable. `Declared`.

---

## §B Granular Item-Level Visibility (DN-37 Q4 follow-on)

### §B.1 The design problem

DN-37 Q4 ruled: **adopt GRANULAR, item-level `pub`**. The current model (`Vis = Private | Pub` on
top-level `fn`/`trait`/`type`; `ast.rs:42-48`) is nodule-level: every item is either exported
to the whole phylum or private to its nodule. The ruling:

> *"Not strictly nodule-level: a function / method / value / variable (and, to be specified, a
> field) can be made `pub` individually, cleanly and in alignment with the rest of the language
> (item-granular `pub`, Rust-precedent)."*

The current `Vis` enum is `Exact` (built; `ast.rs:42-48`, `parse.rs`). The granular model is a
**strictly additive extension**: every item that is `pub` today stays `pub`; every `Private` item
stays private; the new capability is `pub` at finer granularity within items that did not previously
support it (trait methods, `let`-bindings, and eventually fields).

### §B.2 Surface syntax

Item-level `pub` follows the Rust precedent for item-granular visibility: a leading `pub` keyword
on any top-level item. The current model already handles `pub type`, `pub trait`, `pub fn`
(nodule-level). The extension adds:

1. **`pub fn` on a top-level function** — already built (`Vis` on `FnDecl`). No change needed.
2. **`pub` on a trait method** — a method inside a `trait` body can be individually `pub` or
   private-to-the-trait. A `pub` method is visible to the caller at the `impl` site; a private
   method is a trait-internal implementation detail (not part of the trait's public interface).
   Syntax: `trait Foo { pub fn visible(…); fn internal(…); }`.
3. **`pub` on a `let`-binding** — a module-level `let` value (a named constant or precomputed
   value at the nodule's top level) can be `pub` to export the value name phylum-wide.
   Syntax: `pub let name: Type = expr`.
4. **`pub` on a `type`** — already built. No change (this is the existing per-item `pub type`).
5. **`pub` on a field** — DEFERRED (§B.5). Positional fields do not have surface names in v0
   (DN-37 §2.1: *"fields are positional, no named fields in v0"*); a field-level `pub` requires
   the named-fields extension. Do not design or claim this in scope.

All `Declared` (the syntax is a design proposal; the `Vis` extension is additive over the built
`Vis = Private | Pub`).

### §B.3 Composition with nodule-level model and RFC-0019 §4.5

The existing model (M-662; RFC-0019 §4.5 changelog, 2026-06-22) separates two concerns:

- **`pub` gates the `use` namespace** — a cross-nodule `use` can only name `pub` items.
- **Coherence is pub-blind** — the orphan rule and global-uniqueness check span the whole phylum
  regardless of visibility (`ast.rs:37-48` doc comment: *"the coherence view is pub-blind"*).

Granular item-level `pub` inherits both invariants without change:

1. **The `use` namespace rule extends naturally.** A `use` path resolves only to items whose
   `Vis = Pub`. At item granularity: `use nodule_path.fn_name` imports the named function iff it
   is `pub`; a glob `use nodule_path.*` imports all `pub` items in that nodule. No change to the
   resolution rule; only the set of `pub` items changes.
2. **Coherence remains pub-blind.** An `impl Trait for T` in a private-visibility nodule is still
   checked for orphan/global-uniqueness regardless of visibility. The coherence authority is the
   phylum, not the visibility scope.
3. **Re-export is not implied.** A `use` of a `pub` item imports it into the current nodule's
   local scope; it does **not** re-export it. Re-export (if needed) requires an explicit `pub use`
   — the same Rust precedent (a `use` is never `pub`-gated for re-export purposes; `ast.rs:137`
   doc comment: *"a `use` is never `pub`-gated (importing is not re-exporting)"*).

**Interaction with trait methods.** A trait method's individual visibility (`pub` vs private)
is resolved at the trait-definition site. The `impl` must provide all required methods (signatures
are required, not optional); visibility on a required method means the generated method dispatch
exposes or hides that method at the `impl` site. The detail of how a private required method
interacts with phylum-wide impls (a private method in a `pub trait`'s required set) is flagged
as an open sub-question (§B.6 Q2).

All `Declared`.

### §B.4 Never-silent access refusal (G2)

The granular `pub` model follows the **never-silent** discipline (G2):

- A cross-nodule reference to a `Private` item (or a trait method not in the item's `pub` set)
  is an **explicit error** — not a silent miss, not a fallback, not a warning. The diagnostic names
  the item, its declaration site, and its visibility (`Private` / nodule-bound), and offers the fix
  (`add pub`, or access through the public interface). This is the same shape as the existing
  orphan `CheckError` (RFC-0019 §4.5; `checkty.rs:221-240`).
- **Intra-nodule access is unrestricted** — `Vis` gates only cross-nodule visibility (as today;
  `ast.rs:39-40`). Everything in the same nodule is always visible, regardless of `pub`.
- **The visibility check is a pre-coherence gate.** A cross-nodule `use` of a private item fails
  at the name-resolution pass, before coherence or type-checking. This matches the Rust model
  (privacy errors are resolved before type errors) and keeps the error unambiguous.

Tag: `Declared` (the extension of the existing `Vis` model; the never-silent shape is `Empirical`
in the built checker per the orphan-error precedent).

### §B.5 Field-level visibility — DEFERRED (recommendation)

DN-37 Q4 flagged fields as "to be specified." This note recommends **deferring field-level
visibility** for the following reasons:

1. **Fields are positional and unnamed in v0** (DN-37 §2.1; `ast.rs:169-174`). There is no surface
   name to attach a `pub` annotation to. Designing field-level visibility before named fields land
   would require inventing a positional-field `pub` syntax (`pub type Foo = Mk(pub T1, T2)`) that
   may not survive the named-fields extension.
2. **Encapsulation-by-type** is the pragmatic current alternative. A `type` exported as `pub` with
   private constructor helpers (private `fn` constructors + `pub` accessor fns) provides
   field-level encapsulation at the nodule boundary without field-syntax complexity. This is the
   Rust-Opaque-Type pattern (`pub struct Foo(T)` with private constructor); it composes with the
   item-level `pub fn` designed above.
3. **Named fields are greenfield** (DN-37 §2.2; no surface named-field form in v0).
   Field-level `pub` should be designed **with** named fields, not ahead of them.

**Recommendation: field-level `pub` is OUT OF SCOPE for this note.** It is a follow-on design task
gated on the named-fields extension. The encapsulation-by-type pattern covers the near-term need
without the premature syntax commitment. Tag: `Declared`. **FLAG: the maintainer may override this
deferral if field-level privacy is judged urgent before named fields land.**

### §B.6 Open sub-questions (FLAGGED — for the grammar wave)

- **(Q1) `pub(phylum)` vs `pub` vs `pub(nodule)`.** The current model is binary (`pub` = phylum-wide;
  absent = nodule-private). Rust has `pub(crate)` / `pub(super)` / `pub(in path)` for scoped
  visibility. Should Mycelium adopt scoped visibility beyond the binary? Recommendation: **keep
  binary for v1** (KISS; the phylum-level export is the only meaningful cross-boundary today). If
  multi-phylum visibility is needed (when package/registry work lands), revisit. `Declared`.
  **FLAG: if the grammar wave includes this, it supersedes the binary recommendation.**
- **(Q2) Private required method in a `pub trait`.** If a `trait` declares both `pub` and private
  required methods, an external `impl` must satisfy both signatures — but the private method is not
  nameable from outside the defining nodule. Options: (a) private required methods are
  automatically `pub` at the `impl` site (their visibility is promoted); (b) private required
  methods are an error (a trait method must be `pub` if the trait is `pub`); (c) private required
  methods are implementation helpers, not interface members — excluded from the trait's public
  contract and supplied only by impls in the same nodule. Recommendation: **(c)** — a private
  method in a trait body is a default-only helper (defaulted body only; not a required signature
  from external impls). Deferred to the grammar wave. `Declared`. **FLAG.**
- **(Q3) Visibility on `impl` blocks.** Can an `impl SomeTrait for T` block itself carry `pub`?
  The current model never pub-gates `impl` (`ast.rs:137`); coherence is pub-blind. Recommendation:
  **keep `impl` pub-ungated** — an `impl` is always a coherence-level declaration, not an
  export. `Declared`.
- **(Q4) `pub let` and const vs mutable.** A module-level `let` is a value binding. If it binds
  a pure immutable value (the standard case under value semantics), `pub let` is a named constant
  export. If the language later adds effect-tracked mutable globals (out of scope; not currently
  planned), the visibility model for mutable values requires separate design. This note covers only
  the pure `pub let` case. `Declared`.

---

## §C Guarantee Posture (VR-5) + Definition of Done

**Grounding posture (held throughout):**

- **`Exact` (built):** the existing `Vis = Private | Pub` model (`ast.rs:42-48`; M-662); traits +
  impls + monomorphization + coherence (RFC-0019 **Enacted**; `Empirical`); `type` (RFC-0011
  **Enacted r3**; `Exact`); the lowering law (DN-38 §2; the built lowerings are `Exact`).
- **`Empirical`:** the type-checker (the existing checker behavior; never-silent orphan error shape;
  `checkty.rs:221-240`).
- **`Declared` (design proposals, this note):** the `object` keyword recommendation, its surface
  syntax and desugaring, the T-map / T-illuminate / T-learn analysis; the granular `pub` syntax
  extension, its composition with the nodule-level model, the never-silent access-refusal semantics,
  the field-level-`pub` deferral recommendation, the open sub-questions' resolution leans.

No tag is upgraded past its basis (VR-5). The sugar and granular-pub designs are gated on the
grammar wave (RFC-0037 / DN-31 / RFC-0030 / epic #27) for their normative surface forms; this note
is the pre-implementation design, not the ratified landing.

**Definition of Done** (the gate for Proposed → Accepted). This note is `Accepted` when the
maintainer ratifies:

1. **(A1)** the **`object` keyword** (or an alternative) as the recommended object-composition
   sugar, with the three-test analysis (§A.2) as the record.
2. **(A2)** the **desugaring discipline** (§A.3) — `object` lowers to `type` + `impl` + `via`,
   no kernel growth, `reveal`-able — as the normative lowering rule for the sugar.
3. **(B1)** the **granular item-level `pub` model** (§B.2) — `pub fn`, `pub` on a trait method,
   `pub let`, `pub type` — as the target visibility model, superseding the nodule-only reading.
4. **(B2)** the **field-level-`pub` deferral** (§B.5) — defer until named fields land — or a
   contrary ruling.
5. **(B3)** the **never-silent access-refusal semantics** (§B.4) as the visibility-error contract.

Ratification moves Proposed → Accepted (a legal forward step, house rule #3). **Still enacts no
code** — the design is the deliverable; implementation rides the grammar wave. Append-only; VR-5; G2.

---

## Changelog

| Date | Status | Note |
|---|---|---|
| 2026-06-27 | **Proposed** | Created as DN-53 (design DN, M-811). Designs (A) honest `object` composition sugar (DN-37 Q1 follow-on) — recommends `object` keyword (passes DN-02 three-test gate; T-map, T-illuminate, T-learn), desugars to `type` + `impl` + `via` (DN-38 lowering law, no kernel growth KC-3, `reveal`-able), transparency invariant (no mutable self / inheritance / dynamic dispatch implied or expressed); and (B) granular item-level `pub` (DN-37 Q4 follow-on) — `pub fn`, `pub` on a trait method, `pub let` (additive over built `Vis`), pub-blind coherence preserved, never-silent access refusal (G2), field-level `pub` deferred pending named fields. Both gated on the grammar wave (RFC-0037). Enacts no code; moves no other status. Append-only; VR-5; G2. |
