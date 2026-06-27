# ADR-033 — Abstract Function-Typed Field in `FieldSpec` for Dynamic Dispatch

| Field | Value |
|---|---|
| **ADR** | 033 |
| **Status** | **Proposed** (2026-06-27; design-phase; enacts no code; gated on maintainer ratification + KC-3 sign-off) |
| **Decides** | Extend the trusted-core `FieldSpec` enum with an **abstract function-typed field variant** (`FieldSpec::Fn`) so data values can carry method values (the record-of-method-values / dictionary form), making RFC-0019 §4.5 dynamic dispatch v0-expressible. This is a **deliberate, gated trusted-base growth** (KC-3). |
| **Task** | M-810 |
| **Date** | 2026-06-27 |
| **Depends on** | ADR-003 (content-addressed identity, structural hashing); RFC-0019 (traits, dictionary-passing elaboration, §4.5 runtime-dictionary form currently deferred); DN-37 §8 Q3 ruling (defer-but-PLAN-AHEAD; design before complete dogfooding); KC-3 (small auditable kernel — this ADR grows the trusted base deliberately); G2/SC-3 (never-silent, EXPLAIN-able dispatch); VR-5 (downgrade-don't-overclaim) |
| **Sequencing** | Per DN-37 Q3 ruling (2026-06-27): implement in the **near future, before complete dogfooding**. This ADR is the design ahead; the implementation follows after maintainer sign-off. |

> **Posture (transparency rule / VR-5).** This ADR is **Proposed** — it records a design direction,
> enacts no code, and moves no spec status. All claims are `Declared` unless marked otherwise.
> Ratification (Proposed → Accepted) requires maintainer sign-off on the KC-3 trusted-base growth.
> A further step to `Enacted` requires: the implementation lands, the three-way differential
> (L1 ≡ L0 ≡ AOT) over a dynamic-dispatch program runs green, and the Definition of Done below is
> met. No guarantee is upgraded past its basis (VR-5); every open concern is named, not buried (G2).

---

## 1. Context and motivation

### 1.1 The gap

RFC-0019 §4.5 designs the runtime-dictionary form of trait elaboration: a `Construct` record of
method values, where each field holds the implementation for one method. That form is currently
**not v0-expressible** because the trusted core's `FieldSpec` enum (`crates/mycelium-core/src/data.rs`)
accepts only two variants:

```rust
pub enum FieldSpec {
    Repr(Repr),            // a representation-typed field (Binary{n}, Ternary{m}, …)
    Data(String),          // a data-typed field referencing another declaration by name
}
```

There is **no function-typed field** — a constructor field that carries an arbitrary L1 function
value. `FieldTy` (the resolved form) has the same shape. Consequently, a dictionary value
`MkDict_Eq(field_equal: ..., field_not_equal: ...)` where each field is a function cannot be
expressed as a `Construct` at L0 and cannot be registered in the `DataRegistry`.

The RFC-0019 §4.5 changelog entry (2026-06-23, M-673) records this explicitly:

> *"The literal §4.5 runtime dictionary record (a `Construct` of method values) is not
> v0-expressible — the trusted core's `FieldSpec` is `Repr | Data` only, with no function-typed
> field — and remains the deferred normative target, gated on an abstract-parameter `FieldSpec`
> trusted-core change (a separate ADR)."*

This ADR is that separate ADR.

### 1.2 Why dynamic dispatch matters

Monomorphization (static dispatch) is the current elaboration strategy. It is correct and produces
flat, vtable-free, inlinable call graphs. But it has a structural limitation: **heterogeneous
immutable collections** — a `List` whose elements implement the same trait but have different
concrete types — require a single uniform element type. Without a dictionary value, the only
options are:

1. Enumerate a closed sum of all possible concrete types (loses the extensibility promise of traits).
2. Box everything in a `Data` field to a single erased hash (loses the per-element function values).
3. Use the literal runtime-dictionary form from RFC-0019 §4.5 (requires this ADR).

Option (3) is the ADT-horn equivalent of Cook's record of closures (DN-37 §1/§3.7) — but without
open recursion (`fix`-over-`self`), without mutable self, and without giving up coherence. It is the
heterogeneous-immutable-collection driver. The record-of-method-values form is also the natural
vehicle for the eventual **Swift-existential** layout (DN-37 §7: 3-word inline buffer + VWT + PWT)
when that AOT path lands.

### 1.3 KC-3 cost — stated explicitly

`FieldSpec::Fn` **grows the trusted base** (KC-3: small auditable kernel). Every variant of
`FieldSpec` is identity-bearing: it participates in the structural hash of a `DataDecl` and
therefore in the content-addressed identity of every `CtorRef` that references it. Adding a new
variant is a **permanent, append-only change** to the trusted-base surface — it cannot be rescinded
without changing the identity of every existing hash that references it.

This is not a free change. The cost is:

- The `encode_decl` path in `DataRegistry::build` gains a new branch.
- The `FieldTy` (resolved) enum gains a matching `Fn` variant.
- Every tool that pattern-matches on `FieldSpec` or `FieldTy` must handle the new case or fail
  explicitly — never silently (G2).
- The trusted-core audit surface grows by the hashing rule for function-typed fields (§3 below)
  and the dispatch lowering rule (§4 below).

The benefit is that the RFC-0019 §4.5 normative target becomes realizable without any further
kernel change, and the deferred dynamic-dispatch path is unblocked as a pure elaboration concern.
The maintainer must sign off on this trade before the change is implemented.

---

## 2. Design: the `FieldSpec::Fn` extension

### 2.1 The new variant — surface form

```rust
pub enum FieldSpec {
    Repr(Repr),
    Data(String),
    /// An abstract function-typed field: carries a function value at runtime.
    /// `arity` is the number of explicit (non-dictionary) parameters.
    /// The function's type is abstract — it is not structurally encoded in the field
    /// shape; the type system enforces it above the kernel (KC-3).
    Fn { arity: u32 },
}
```

Correspondingly, the resolved form:

```rust
pub enum FieldTy {
    Repr(Repr),
    Data(ContentHash),
    /// A function-typed field of `arity` parameters.
    Fn { arity: u32 },
}
```

**Why `arity` only, not a full function type?** The trusted core stores the minimum identity-bearing
information needed to hash injectively and check saturation. A full function type (parameter types +
return type) would require the core to know `Repr`, `Data`, and `Fn` types recursively — a
self-referential encoding that either loops or requires a separate type-level registry. The `arity`
encodes the saturation invariant (WF6 equivalent for dictionary fields) without that complexity.
The richer type is tracked by the elaborator, which is the untrusted layer. This is the `Declared`
design; the open question around whether `arity` alone is sufficient for the soundness argument is
flagged in §6.

**Why not a closure/capture descriptor?** Value semantics (LR-8) + acyclicity (LR-9). A function
field in a dictionary record is a top-level, content-addressed term reference — it captures no
runtime environment that is not itself a pure value. The function is identified by its content hash
(a `TermHash`), not by a closure pointer. The `Fn { arity }` variant signals that the field slot
accepts a function of that arity; the actual method body is a value in the term registry.

### 2.2 The content-addressed tag

The `encode_decl` function in `crates/mycelium-core/src/data.rs` must gain a new match arm. Using
the existing `content::tag` scheme (one unique tag per structural element), the encoding is:

```text
FieldSpec::Fn { arity } =>
    c.tag(FIELD_FN);
    c.u32(arity);
```

where `FIELD_FN` is a fresh tag, not colliding with `FIELD_REPR`, `FIELD_DATA`, `FIELD_CYCLE`.
This ensures **injectivity**: a `Fn { arity: 2 }` field hashes differently from a `Fn { arity: 1 }`
field and from any `Repr` or `Data` field. Two declarations that differ only in whether a field
is `Fn { arity: 2 }` vs `Repr(Binary{64})` will have distinct content hashes. No aliasing.

The Unison cycle-placeholder scheme (self-recursive declarations hash in-cycle references as an
index placeholder) is **unaffected**: `FieldSpec::Fn` carries no reference to another declaration
and never participates in a cycle. The `in_cycle` check in `encode_decl` is not exercised for
`Fn` fields. `Declared` — argument: function fields reference term hashes, not declaration
hashes; the cycle structure is over `DataDecl` references only.

### 2.3 Injectivity preservation — the load-bearing correctness concern

ADR-003 requires that content-addressed identity be **injective over structure**: two declarations
with the same hash must have the same structure, and the same structure must always produce the
same hash (determinism + collision-resistance, modulo the hash function's own properties).

The three existing field variants achieve this by:

- `Repr(r)` — tags `FIELD_REPR` then the repr encoding (already injective over `Repr`).
- `Data(name)` — tags `FIELD_DATA` then the referenced declaration's hash (injective by hash).
- `Data(name)` in-cycle — tags `FIELD_CYCLE` then the cycle index (injective over index within cycle).

Adding `Fn { arity }` with a unique `FIELD_FN` tag and then `c.u32(arity)` preserves injectivity
because:

1. `FIELD_FN` does not collide with any existing tag (new tag, explicit allocation).
2. `u32` encoding of `arity` is injective over `u32`.
3. The combined `(FIELD_FN, arity)` cannot collide with `(FIELD_REPR, r)`, `(FIELD_DATA, h)`, or
   `(FIELD_CYCLE, idx)` because the tag discriminant differs.

Grounding: `Declared`-with-argument (the argument is the disjoint-tag reasoning above; formal
collision-resistance follows from the underlying hash function's preimage resistance, which is
inherited from the existing `Canon` scheme, itself `Empirical` — tested in `content.rs`).

**Open concern (FLAG-1 — see §6):** the `arity`-only encoding means two dictionary declarations
with the same arity for all function fields but different function types (e.g., one takes
`Binary{8} → Binary{8}` and another takes `Binary{16} → Binary{16}`) hash identically at the
`FieldSpec` level. The type distinction is enforced by the elaborator (untrusted layer), not the
kernel. This is a deliberate KC-3 tradeoff — the kernel does not encode types — but it means
that the kernel-level identity of two dictionaries with same-arity-but-different-type function
fields is the same. The elaborator must reject misuse. This is flagged as an open concern in §6.

---

## 3. Dispatch lowering — the runtime dictionary call

### 3.1 How a dynamic call resolves

With `FieldSpec::Fn` available, a trait dictionary elaborates (per RFC-0019 §4.3) to a genuine
`Construct` record at L0:

```text
type Dict_Eq<A> = MkDict_Eq(
    field_equal:     Fn { arity: 2 },   // (A, A) -> Binary{1}
    field_not_equal: Fn { arity: 2 },
)
```

A dynamic dispatch call `dict.field_equal(x, y)` lowers to:

```text
let f = project(dict, field_index_equal) in    // Match on MkDict_Eq, bind field 0
App(App(f, x), y)                               // curried application at arity 2
```

where:

- `project(dict, i)` is a flat `Match` on the single-constructor `MkDict_Eq` (WF7-compliant;
  already expressible at L0 via the existing `Match` node).
- The projected value `f` is a **function value** in the L0 term model — a `Lam`-term, or a
  `Var` bound to a top-level definition.
- Application is ordinary curried `App` — no new kernel node.

The kernel node budget is **unchanged** (RFC-0007 §4.1 ten nodes): `Construct`, `Match`, `Lam`,
`App`, `Var` are the only nodes in play. The trusted-base growth is **only** in the `DataRegistry`
/ `FieldSpec` / `FieldTy` extension; the term grammar (`Node`) does not change.

### 3.2 Relationship to monomorphization

Monomorphization **remains the default and only dispatch for static calls**. The `FieldSpec::Fn`
extension adds the missing primitive for the heterogeneous case — it does not replace or
subsume the monomorphic path. The elaborator chooses:

- **Static type known at call site → monomorphize** (current behavior, unchanged).
- **Static type unknown (heterogeneous collection, `dyn`-equivalent) → dictionary dispatch**
  (new, enabled by `FieldSpec::Fn`).

The explicit `EXPLAIN`-of-dispatch (§4 below) makes the choice auditable. No call site silently
changes dispatch strategy.

### 3.3 Value semantics and acyclicity

The dictionary record is an **immutable value**. Its function fields are references to top-level,
content-addressed term hashes — not closure pointers over mutable captures. LR-8 (immutability)
and LR-9 (acyclicity) are preserved: the dictionary cannot reference itself (no `fix`-over-`self`),
and no function field captures a mutable back-reference to the enclosing value.

This is the key difference from DN-37 §3.7's discouraged record-of-closures / open recursion
form. That form requires `fix` and mutable self; the dictionary form is self-free and acyclic.
The DN-37 §3.7 caveat does **not** apply here. `Declared`-with-argument (argument: the dictionary
is built by `Construct` from pure value fields; `fix` is not required and not used).

---

## 4. Never-silent dispatch — EXPLAIN (G2/SC-3)

### 4.1 The dispatch is reified, not opaque

A dynamic dispatch call through a dictionary field is not an invisible vtable indirection. At
every level:

- **L2/surface:** the call `d.method(args)` desugars visibly to a dictionary projection + application.
  The elaborated form is inspectable via the LSP stage-dump channel (SC-5/M-140 — EXPLAIN).
- **L1/elaborated:** the `Match`-project + `App` chain is in the L1 term tree, not a black box.
- **L0/runtime:** the interpreter executes the `Match` projection and `App` sequence; the
  `EXPLAIN` query can report "dynamic dispatch via dictionary field `i` of `#T#0`" at any step.

### 4.2 EXPLAIN output form

The runtime or AOT execution of a dynamic dispatch step must produce (on `EXPLAIN` query) an
observation of the form:

```text
dispatch(dynamic):
  dictionary: #<dict-decl-hash>#0
  field:      <field-index>  // or field name if metadata is available
  resolved:   #<method-term-hash>
  arity:      <n>
```

This satisfies the G2 / SC-3 never-silent requirement: the dispatch choice is **reified and
inspectable**. There is no opaque vtable magic. The `Declared` tag applies: this is the intended
EXPLAIN form; the exact serialization format is a detail for the implementation.

### 4.3 Mode interaction (RFC-0034 / ADR-032)

In `fast` mode, the dispatch EXPLAIN record is available on demand but not emitted on every call.
In `certified` mode, dispatch provenance (which dictionary, which field, which resolved term) is
part of the audit trail. Both modes are never-silent (G2): the mode itself is tagged on every result.

---

## 5. Verification — three-way differential

Per ADR-003 / RFC-0019 §4.3 and DN-37 §7, the correctness criterion for the dynamic-dispatch
path is the **three-way differential**: the L1 elaborated semantics, the L0 interpreter semantics,
and the AOT-compiled semantics must agree on all outputs for any dynamic-dispatch program that is
within scope of the closed L0 fragment.

### 5.1 The test program shape

A minimal differential test for this ADR:

```text
// A trait with one method.
trait Eq<A> { fn equal(x: A, y: A) -> Binary{1} }

// Two distinct implementations.
impl Eq<Binary{8}> { fn equal(x, y) = ... }
impl Eq<Binary{16}> { fn equal(x, y) = ... }

// A heterogeneous list element type: a dictionary value.
type EqDict = MkEqDict(Fn { arity: 2 })

// Build two dictionary values and call through them.
let d8  = MkEqDict(impl_Eq_Binary8_equal)
let d16 = MkEqDict(impl_Eq_Binary16_equal)
// Dynamic call: project field 0 from d8, apply to (x, y).
```

The three-way differential checks:

- **L1 ≡ L0:** the elaborated L1 term and the L0 interpreter agree on the result of projecting the
  function field and applying it.
- **L0 ≡ AOT:** the interpreter and the AOT-compiled path agree (where AOT runs to a closed L0
  fragment — the AOT path may be partially stubbed at this stage).

Grounding: `Declared` (the test shape is a design target; the passing differential is the
`Enacted` evidence bar).

### 5.2 Property tests

Each of the following properties must have a covering property test before `Enacted`:

1. **Hash injectivity for `Fn` fields.** Two `DeclSpec`s that differ only in `FieldSpec::Fn`
   arity produce distinct `DataDecl` hashes. Two `DeclSpec`s identical in structure (including
   `Fn` arities) produce the same hash regardless of declaration names.
2. **`FIELD_FN` tag does not collide.** A spec with a `Fn` field and one with a `Repr` field or
   `Data` field of the same position always hash to distinct values.
3. **Dispatch agreement.** For any closed dictionary value and valid argument values,
   `eval_L1(project_and_apply(dict, args)) == eval_L0(project_and_apply(dict, args))`.
4. **Never-silent on arity mismatch.** Applying a dictionary's `Fn { arity: 2 }` field to one
   argument produces an explicit arity-error, never a silent wrong result.

---

## 6. Open concerns and flagged items

### FLAG-1 — Arity-only encoding and type-soundness at the kernel level

As noted in §2.3, the `FieldSpec::Fn { arity }` encoding does not embed the function's parameter
or return types in the kernel hash. Two dictionary declarations with identical arity for all
function fields but different function types hash to the same `FieldSpec`-level encoding. The
elaborator enforces type correctness above the kernel. This is a deliberate KC-3 tradeoff, but it
means:

- The kernel cannot reject a type-incorrect dictionary value by itself; it can only check
  saturation (correct number of arguments).
- A bug in the elaborator that incorrectly instantiates a `Fn`-fielded dictionary at a
  wrong type will not be caught by the trusted core.

**Resolution needed (pre-Enacted):** confirm (with a written argument, not just assertion) that
the elaborator's type discipline is sufficient to make this safe, or determine whether a minimal
type signature (e.g., just the `Repr` of the first parameter) should be added to `FieldSpec::Fn`.
This is the principal soundness concern for this ADR. `Declared` — unresolved at Proposed stage.

### FLAG-2 — `FieldTy::Fn` and the `DataRegistry` resolution path

The current `resolve_decl` function maps `FieldSpec::Data(name)` to `FieldTy::Data(hash)` by
looking up `by_name`. `FieldSpec::Fn { arity }` has no name-lookup step — it resolves trivially
to `FieldTy::Fn { arity }`. This is correct but must be made explicit in the implementation to
avoid a future refactor silently omitting the case. The implementation must add `FieldSpec::Fn` to
all match arms that are currently exhaustive on `FieldSpec` — any non-exhaustive arm is a
compilation error in Rust, so the "never silent" property is enforced by the type system.

### FLAG-3 — Interaction with `canonical_cycle_order` and `strongly_connected_components`

The SCC and cycle-ordering algorithms in `data.rs` traverse `FieldSpec::Data` edges to build the
dependency graph. `FieldSpec::Fn` carries no `Data` edge (it references a term, not a
declaration). The SCC algorithm's `succ` function must be updated to **ignore** `FieldSpec::Fn`
fields (they are not declaration dependencies). This is the correct behavior — method
implementations do not create declaration-level cycles — but it must be explicitly coded and
tested (property: a declaration with `Fn` fields only participates in SCCs via its `Data` fields).

### FLAG-4 — Maintainer KC-3 sign-off is a prerequisite

This ADR is **Proposed**. The KC-3 trusted-base growth cost (§1.3) is explicit. No implementation
work should begin on `FieldSpec::Fn` without the maintainer's explicit Accepted ratification of
this ADR. The sequencing per DN-37 Q3 is: design now (this ADR), implement before complete
dogfooding, not before the ADR is ratified.

### FLAG-5 — `docs/adr/README.md` index entry (orchestrator-owned)

`docs/adr/README.md` is orchestrator-owned. This leaf cannot edit it. The integrating orchestrator
must add an index entry for ADR-033 after merging this leaf's branch.

---

## 7. Rationale and alternatives

### 7.1 Why not encode a full function type in `FieldSpec::Fn`?

A full function type would be a recursive structure over `FieldSpec` variants (parameter types
can themselves be `Data` or `Fn` types). This recursion would require the `DataRegistry` to
simultaneously hash data and function types — conflating the data registry with a type registry.
The existing architecture cleanly separates: data declarations live in `DataRegistry`, function
types are elaborated-layer concerns. Adding a full function type to `FieldSpec` would erode this
separation and grow the trusted core further (KC-3 cost compounds). The `arity`-only encoding is
the minimal addition consistent with KC-3. See FLAG-1 for the open concern this creates.

### 7.2 Why not a separate `FnRegistry`?

A separate registry for function-typed field descriptors was considered. It has no clear
advantage: the `FieldSpec::Fn` variant needs only `arity` to participate in hashing (§2.2); a
separate registry would add indirection without reducing the trusted-core footprint. The minimal
change to `FieldSpec` / `FieldTy` is preferred (KISS, KC-3).

### 7.3 Why not defer until the Swift-existential layout ADR?

DN-37 §7 describes the Swift-existential template (3-word inline buffer + VWT + PWT) as the
eventual AOT layout for dynamic dispatch. That is an AOT-optimization concern, not a semantic
primitive. The `FieldSpec::Fn` extension is the semantic primitive that makes RFC-0019 §4.5
expressible at L0; the existential layout is a later, separate ADR (an optimization step outside
the trusted kernel). The two are sequenced: `FieldSpec::Fn` first (this ADR), existential layout
later (when AOT performance is the active concern). Conflating them would block the semantic
primitive on an AOT optimization that is not yet in scope.

### 7.4 Why not use a `Data` field holding an erased function hash?

One alternative is to store a function body as a `Data` field pointing to a synthetic
single-constructor declaration that wraps the term hash. This works around the `FieldSpec`
extension but is opaque: the dictionary field loses its arity information at the kernel level, and
dispatch becomes `EXPLAIN`-unable at the structural level (the field looks like an arbitrary data
value, not a function). This violates G2. The `FieldSpec::Fn` extension is the transparent path.

---

## 8. Definition of Done

This ADR moves **Proposed → Accepted** when the maintainer ratifies:

- **(a)** the `FieldSpec::Fn { arity }` design (§2) as the correct minimal extension.
- **(b)** the `FIELD_FN`-tag hashing scheme (§2.2) as injective and non-colliding.
- **(c)** the KC-3 trusted-base growth cost (§1.3) as an acceptable, deliberate tradeoff.
- **(d)** FLAG-1 (arity-only type safety) as a known open concern to be resolved pre-Enacted.

This ADR moves **Accepted → Enacted** when:

- [ ] `FieldSpec::Fn { arity }` and `FieldTy::Fn { arity }` are added to
  `crates/mycelium-core/src/data.rs`, with the `FIELD_FN` content-addressing tag.
- [ ] All match arms on `FieldSpec` / `FieldTy` are exhaustive and explicitly handle the new
  variant — no silent omission (G2).
- [ ] `resolve_decl`, `encode_decl`, `strongly_connected_components`, and `canonical_cycle_order`
  in `data.rs` are updated and handle `FieldSpec::Fn` correctly.
- [ ] The four property tests (§5.2) are added to the in-crate `src/tests/` module (test layout
  rule: no inline `#[cfg(test)]` in `data.rs`; tests live in `src/tests/data.rs`).
- [ ] The three-way differential test (§5.1) over a minimal dynamic-dispatch program is green
  (L1 ≡ L0; L0 ≡ AOT stub where applicable).
- [ ] FLAG-1 is resolved with a written argument added to this ADR (append-only — append before
  Enacted, do not rewrite §6).
- [ ] `just check` is green for `mycelium-core` and its reverse-dependents.
- [ ] RFC-0019's changelog gains an append-only note: the §4.5 deferred normative target is
  unblocked by ADR-033 (pointer only; no RFC normative text rewritten).
- [ ] This ADR is indexed in `docs/adr/README.md` (orchestrator-owned file — FLAG-5).

---

## 9. Grounding

| Claim | Tag | Basis |
|---|---|---|
| `FieldSpec` is `Repr \| Data` only, blocking RFC-0019 §4.5 | `Exact` | `crates/mycelium-core/src/data.rs` lines 99–104; RFC-0019 §4.5 changelog 2026-06-23 M-673 |
| Dictionary passing uses `Construct`/`Match`/`Lam`/`App`/`Var` only — no new kernel nodes | `Empirical` | RFC-0019 §4.3; `crates/mycelium-l1/src/mono.rs` (existing monomorphization path) |
| `Fn { arity }` + `FIELD_FN` tag is injective over the existing encoding scheme | `Declared` | Disjoint-tag argument (§2.3); full formal proof is future `Proven` basis |
| Dispatch is auditable and EXPLAIN-able (no opaque vtable) | `Declared` | Design intent; `Enacted` evidence bar = passing EXPLAIN query in implementation |
| Acyclicity preserved (no `fix`-over-`self`) | `Declared` | Argument: dictionary `Construct` is bottom-up, no cycle back-edge; LR-8/LR-9 structural |
| DN-37 Q3 ruling to design-ahead and implement before dogfooding | `Exact` | DN-37 changelog 2026-06-27 (append-only; maintainer-ratified) |
| KC-3 trusted-base growth is deliberate and gated | `Declared` | This ADR §1.3; maintainer decision, not implementer discretion |

---

## Changelog

| Date | Status | Note |
|---|---|---|
| 2026-06-27 | **Proposed** | Initial design. Authored as the trusted-core `FieldSpec`-abstract-function-field ADR called for by DN-37 §8 Q3 (2026-06-27 maintainer ruling) and RFC-0019 §4.5 changelog (M-673 deferred normative target). Enacts no code; moves no spec status. CHANGELOG / Doc-Index / issues.yaml / docs/api-index owned by the integrating orchestrator. (Append-only; VR-5; G2.) |
