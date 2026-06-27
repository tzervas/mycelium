# Design Note DN-55 — Static Specialization: The Polymorphism Model

| Field | Value |
|---|---|
| **Note** | DN-55 |
| **Status** | **Accepted** (2026-06-27; **ratified by the maintainer 2026-06-27**, R1 gate) — formalizes the static-specialization model name and the zero-primitive kernel consequence; enacts no code. Prior: **Proposed** (2026-06-27). |
| **Date** | June 27, 2026 |
| **Decides** | *Formalizes the already-implemented model.* Names and unifies Mycelium's single polymorphism mechanism — **static specialization** (short form: **static comp**) — and records its most important downstream consequence: polymorphism consumes **zero kernel primitives** (KC-3). Documents the one escape hatch (dynamic dispatch, ADR-033) and the surface distinction between the three parameter axes. Feeds the kernel-completeness-and-freeze criterion that a forthcoming orchestrator capstone note will close. |
| **Task** | M-814 |
| **Feeds** | `crates/mycelium-l1/src/mono.rs` (M-673/M-753 — the existing implementation); RFC-0019 (traits + coherence + enacted monomorphization); DN-42 (width as a const-generic, M-753); DN-38 (the lowering law + `reveal`); ADR-033 (forthcoming — the dynamic-dispatch `FieldSpec` boundary). |

> **Posture (transparency rule / VR-5 / G2).** This note **documents** the ratified, implemented
> mechanism; it enacts no code and moves no other decision's status. Guarantee-tag discipline:
> the model itself (`Declared` — a named framing of the existing behavior); the three-way
> execution agreement for the generic stdlib surface (`Empirical` — by trial, grounded in
> `crates/mycelium-l1/tests/{differential.rs,std_generic_conformance.rs}`); nothing is `Proven`
> (no machine-checked theorem exists). No tag is upgraded past its basis (VR-5/house rule #4).

---

## Definition of Done

This note is **done** when:

1. The model name ("static specialization" / "static comp") is recorded here as the canonical
   term, cross-referenced from RFC-0019 and the forthcoming kernel-freeze capstone note.
2. The zero-primitive consequence is stated precisely: polymorphism adds no L0 node and no
   kernel prim, with the ADR-033 dynamic-dispatch boundary as the one exception.
3. ADR-033 exists and cross-references this note at its boundary definition.
4. The forthcoming kernel-freeze capstone note cites this note as the polymorphism ledger entry.

Items 3 and 4 are downstream work; this note's own content is complete on authoring.

---

## §1 The single mechanism: static specialization

Mycelium has one polymorphism mechanism: you write code once over a **parameter**, and
**monomorphization** produces a closed, concrete specialization per distinct instantiation.
The parameter is one of three axes:

- A **type parameter** (`[A]`) — abstracts over the element type.
- A **const/width parameter** (`{N}`) — abstracts over a representation width (DN-42, M-753).
- A **bounded type parameter** (`[A: Trait]`) — abstracts over a type, constrained to a trait
  interface (RFC-0019).

In every case the output is the same: **closed monomorphic L0**. No polymorphism survives past
the front-end elaboration pass. The existing implementation is `crates/mycelium-l1/src/mono.rs`
(M-673; M-753 for width generics):

> *"turns a checked generic-and-trait `Env` into a closed, monomorphic `Env`"*
> — `mono.rs` module doc

The model is called **static specialization**, or **static comp** in short form. "Static" because
all specialization resolves at compile time with no runtime dispatch. "Comp" because it is a
compile-time composition step — a specialization pass, analogous to Zig's `comptime` (see §5).

---

## §2 How monomorphization handles each axis

### 2.1 Type parameters

A function `fn first_or[A](xs: List[A], default: A) -> A` is specialized per distinct `A` at
each call site. The monomorphizer (`mono.rs::emit_fn`) infers the concrete type arguments by
unifying the declared parameter types against the actual argument types, then rewrites the body
with the concrete type substituted throughout. The mangled name records the specialization:
`first_or$Binary8` and `first_or$Binary4` are two distinct functions with two distinct content
hashes (`mono.rs` lines 19-24 — "honest identity fragmentation").

### 2.2 Width (const-generic) parameters

A function `fn id_bits{N}(x: Binary{N}) -> Binary{N}` is specialized per distinct `N` (DN-42,
M-753). The `Width::Var(N)` in the type is resolved to a `Width::Lit(n)` at each call site by
the same unification machinery, extended with a `Width` arm (`mono.rs` lines 411-444). An
undetermined width at mono time is an explicit `Residual`, never a defaulted `Binary{8}` — the
never-silent invariant (G2/VR-5 / DN-42 §4).

### 2.3 Bounded type parameters (trait dispatch)

A function `fn contains[A: Eq](needle: A, haystack: List[A]) -> Binary{1}` is specialized per
concrete `A`. The monomorphizer additionally resolves each unqualified trait-method call (`equal`,
`not_equal`) to the **one coherent instance's method body**, rewriting the call to a direct call
to the mangled method function. The dispatch choice is reified in a queryable `MonoSelections`
EXPLAIN record (`mono.rs` lines 63-130):

```rust
pub struct InstanceSelection {
    pub trait_name: String,
    pub for_ty: Ty,
    pub impl_mangled: String,
}
```

This is **dictionary-free**: no runtime dispatch table is constructed; the trait method resolution
is entirely static, emitting a direct call. The term "dictionary-free static resolution" is used
in the RFC-0019 enacted changelog (2026-06-23 entry): *"monomorphization with static instance
resolution, not the literal §4.4-recommended runtime-dictionary form."*

---

## §3 The load-bearing consequence: zero kernel primitives

The most important architectural consequence of static specialization:

> **Polymorphism costs zero kernel primitives.**

Because static specialization fully erases in the frontend — no polymorphism appears at L0 at
all — the L0 kernel (`mycelium-core`) needs no polymorphism node, no trait-dispatch primitive,
and no representation of type variables or width variables. The L1 monomorphizer is the entire
polymorphism implementation; the kernel is untouched.

The `mono.rs` module doc states this directly:

> *"No `mycelium-core` change (KC-3): this is a pure frontend rewrite over the checked `Env`;
> the kernel/registry path is untouched."*

On the kernel-freeze ledger (KC-3 — the small, auditable kernel goal), polymorphism is a
**zero-primitive feature**. It is accounted for as follows:

- **Before monomorphization:** generic functions, trait declarations, and impl instances live in
  the L1 `Env` (checked, but not yet elaboratable by the kernel-path elaborator).
- **After monomorphization:** the `Env` is fully closed (`params` empty, no `Ty::Var`, no trait
  entries). The existing L1-eval / L0-interp / AOT path runs unchanged on this closed env.
- **The kernel itself:** unchanged. Zero new nodes. `mycelium-core`'s `FieldSpec` (currently
  `Repr | Data`) is not altered by type-parameter or trait polymorphism — those are surface
  features that lower entirely away before reaching the kernel.

This is the maximal alignment with KC-3: not "polymorphism can be added without growing the
trusted base much" but "polymorphism adds **nothing** to the trusted base" — it is a pure
front-end pass that produces exactly the closed L0 input the existing kernel already processes.

---

## §4 Transparency by construction: `reveal` and the lowering law

Every polymorphic use is `reveal`-able to its concrete monomorphic L0 equivalent. This follows
directly from DN-38's lowering law:

> *"Every surface feature lowers to L0. Each lowering pass is small, IL-grammar-checked, and
> semantics-preserving."* — DN-38 §2

Static specialization is precisely one such pass. A generic function `fn id[A](x: A) -> A`,
instantiated at `Binary{8}`, lowers to the mangled closed function `id$Binary8` with parameter
type `Binary{8}` and body `x` — a concrete L0 function that `reveal` can show (DN-38 §5, §8.1).
No black box: the specialization is reified in the `MonoSelections` record (trait dispatch) and
in the mangled function registry (type and width generics). Both are programmatically inspectable
(house rule #2 — no black boxes; `mono.rs` lines 63-130 / `MonoSelections::get`,
`MonoSelections::iter`).

The three-way agreement (L1-eval ≡ L0-interp ≡ AOT) holds over the generic stdlib surface
(`Empirical` — by trial, `crates/mycelium-l1/tests/std_generic_conformance.rs`). The conformance
gate runs each width-generic stdlib op at two or more distinct widths from one source definition,
confirming that the specialized L0 outputs agree across execution paths.

---

## §5 Best-fit analogue: Zig's `comptime`

The best analogue in existing languages is **Zig's `comptime`** — compile-time specialization,
monomorphizing, with no hidden dispatch. Zig evaluates `comptime` arguments at compile time,
producing a specialized function per distinct argument value. The philosophical match with
Mycelium's static specialization is strong:

- **No hidden dispatch.** The output is always a direct, concrete function call — no vtable,
  no dictionary indirection.
- **Transparency.** The programmer sees exactly which specialization fires; nothing is implicit.
- **No new runtime primitives.** The specialization is a frontend transformation; the runtime
  sees only concrete code.

The surface form differs (Zig uses `comptime` explicitly at the call site; Mycelium infers
specialization from the type-checked context), but the mechanism and the "no black boxes"
philosophy are the same.

**Why not a fungal analogy?** Conventional terms (`fn`, `trait`, `impl`, type parameter,
bound) were kept by the DN-02 three-test gate: T-illuminate fails for any fungal-themed
parametric specialization metaphor (no fungal concept maps to "one body, N concrete
specializations"). The same reasoning applies here: "static specialization" and "static comp"
are the conventional-with-a-modifier approach — precise, unambiguous, no mapping friction.

---

## §6 Surface: distinguishing the axes transparently

The kind-split bracket scheme (DN-31, once ratified as a supersession of RFC-0019 §4.1) makes
the parameter axis visible at the surface:

| Bracket | Axis | Example |
|---|---|---|
| `[A]` | type parameter (any type) | `fn id[A](x: A) -> A` |
| `[A: Trait]` | bounded type parameter | `fn sort[A: Ord](xs: List[A]) -> List[A]` |
| `{N}` | const/width parameter | `fn id_bits{N}(x: Binary{N}) -> Binary{N}` |

The bracket form tells the reader which axis is being abstracted: `[]` = type position (a
type the caller supplies); `{}` = const/width position (a width the caller supplies). A bounded
type `[A: Trait]` is a `[]`-type with a constraint — the same type-parameter axis, narrowed.
The three forms are all static specialization; the bracket is a transparency marker, not a
different mechanism.

---

## §7 The one escape: dynamic dispatch (ADR-033 boundary)

There is exactly one polymorphism form that **cannot** be fully monomorphized: **heterogeneous
immutable collections** (e.g., a list that holds values of different concrete types), where the
concrete type is not statically known at the use site.

For this case, Mycelium's kernel needs exactly one additional capability: an **abstract
function-field** in `FieldSpec` that can hold a method reference at a type that is not fully
known until runtime. This is the boundary ADR-033 defines (being authored as a sibling to this
note).

Even the dynamic-dispatch case stays honest:

- The dispatch record is a **reified value** (`reveal`-able, never a hidden vtable).
- The method reference in the field is content-addressed and inspectable.
- The field boundary is explicit in the type — a value that carries a dynamic-dispatch field
  is typed differently from a fully-static value.

This is the **only** polymorphism form that touches the kernel — and it does so through a
single, minimal `FieldSpec` extension, not through general runtime-dictionary machinery.

**Summary of kernel cost accounting:**

| Form | Kernel cost |
|---|---|
| Type-parameter generics (`[A]`) | Zero — monomorphizes away |
| Width generics (`{N}`) | Zero — monomorphizes away |
| Trait-bounded generics (`[A: Trait]`) | Zero — dictionary-free static resolution, monomorphizes away |
| Dynamic dispatch (heterogeneous collections) | One `FieldSpec` extension (ADR-033) |

---

## §8 Terminology

| Term | Definition |
|---|---|
| **Static specialization** | The unifying model name: write once over a parameter, specialize statically per instantiation. |
| **Static comp** | The short form. "Comp" for compile-time composition (the specialization pass). |
| **Monomorphization** | The implementation mechanism: the frontend pass that produces closed monomorphic L0 from a generic-and-trait `Env`. |
| **Dictionary-free** | No runtime dispatch table (dictionary) is constructed for trait methods; all resolution is static. |
| **Static resolution** | Trait-method dispatch resolved at compile time to a direct call to the concrete instance's method. |

The surface-level terms **generic**, **type parameter**, and **bound** are kept: they are the
conventional names for the user-facing surface forms and pass the DN-02 three-test gate on
familiarity (T-illuminate, T-precise, T-ergonomic). "Static specialization" names the
**mechanism** that implements all of them; "generic" / "type parameter" / "bound" name the
**surface forms** the programmer writes.

---

## §9 Grounding

- **`crates/mycelium-l1/src/mono.rs`** (read 2026-06-27, lines 1-47, 63-130, 151-164, 349-369,
  381-540, 605-707) — the module doc establishing the mechanism ("turns a checked
  generic-and-trait `Env` into a closed, monomorphic `Env`"; "No `mycelium-core` change (KC-3)");
  `MonoSelections`/`InstanceSelection` (the EXPLAIN record); `monomorphize_with_selections` (the
  entry point); `finish` (the closed-env construction, empty traits/instances/impls); `emit_fn`
  (type and width pinning); `emit_method` (trait-method static resolution). Grounds §2-§3.
- **`docs/rfcs/RFC-0019-Traits-and-Parametric-Polymorphism.md`** — §4.4 (dictionary-passing vs
  monomorphization analysis); enacted changelog 2026-06-23 ("monomorphization with static instance
  resolution, not the literal §4.4-recommended runtime-dictionary form"; "dictionary-free static
  resolution, EXPLAIN-reified in `mono.rs`"; "kernel node budget is unchanged (KC-3)"; deferred
  `FieldSpec` change to ADR-033). Grounds §2.3, §3, §7.
- **`docs/notes/DN-42-Width-Generics.md`** (read 2026-06-27, all) — width as a const-generic
  parameter bound at monomorphization (Option A, ratified); `Width::Var` / `Width::Lit`; the
  undetermined-width `Residual`; the never-silent discipline; M-753 landing note (changelog
  2026-06-27 entry). Grounds §2.2.
- **`docs/notes/DN-51-Binary-Width-Arithmetic-Promotion-and-Narrowing.md`** (read 2026-06-27)
  — per-instance guarantee tags; the mixed-width promotion policy superseding DN-42's refusal.
  Grounds the guarantee-inheritance claim in §4 (tags are per-specialization, not per template).
- **`docs/notes/DN-38-Layered-Lowering-Atlas.md`** (read 2026-06-27, §1-§2) — the lowering law
  ("every surface feature lowers to L0 … the kernel never grows for ergonomics — KC-3");
  `reveal` as the inspectability affordance. Grounds §4.
- **`docs/notes/DN-31-Delimiter-and-Operator-Deconfliction.md`** (read 2026-06-27, §2) — the
  `[]` / `{}` kind-split bracket scheme. Grounds §6 (surface axis distinction). Note: DN-31 is
  currently Draft; §6's bracket examples are the proposed surface forms, subject to DN-31
  ratification as a supersession of RFC-0019 §4.1.
- **`crates/mycelium-l1/tests/std_generic_conformance.rs`** (read 2026-06-27, header) — the
  three-way conformance gate for the generic stdlib surface (M-719); "three-way differential
  agreement, by trial" (`Empirical`). Grounds the three-way execution claim in §4.
- **House rules:** KC-3 (small auditable kernel); G2 (never-silent); VR-5 (no tag upgrade past
  basis); house rule #2 (no black boxes); DN-38 lowering law; DN-02 (three-test gate for naming).

---

## §10 Open questions (FLAGGED)

1. **DN-31 ratification.** §6's bracket scheme (`[A]` / `{N}` / `[A: Trait]`) is the proposed
   surface form pending DN-31 ratification and the RFC-0019 §4.1 supersession. The mechanism
   (static specialization) is independent of the surface spelling; the brackets name the axis.
   **FLAG: §6 should be updated when DN-31 is ratified as a supersession.**
2. **ADR-033 content.** §7 cross-references ADR-033 as the dynamic-dispatch boundary. ADR-033
   is being authored in parallel; this note's §7 is a forward reference.
   **FLAG: verify ADR-033 cross-reference once the sibling leaf's work lands.**
3. **Kernel-freeze capstone note.** §3 records the zero-primitive consequence as feeding a
   "forthcoming orchestrator capstone note." That note does not yet exist; this note's DoD item
   4 is pending it. **FLAG: link the capstone note here once authored.**
4. **Width-generic instances (DN-42 Q5).** Whether v1 admits width-generic *instances*
   (e.g., `impl Eq for Binary{N}`) or only width-generic free functions is deferred (DN-42 §7
   Q5). If width-generic instances land, §2.3 and §7 need an update.
   **FLAG: revisit when Q5 is decided.**

---

### Changelog

| Date | Status | Note |
|---|---|---|
| 2026-06-27 | **Proposed** | Initial authoring (M-814). Names the static-specialization model, records the zero-primitive consequence (§3), the dynamic-dispatch escape (§7), and the surface axis distinction (§6). Feeds the kernel-freeze capstone. Enacts no code; moves no other decision's status. |
