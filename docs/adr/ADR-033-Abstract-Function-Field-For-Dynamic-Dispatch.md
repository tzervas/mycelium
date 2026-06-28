# ADR-033 — Abstract Function-Typed Field in `FieldSpec` for Dynamic Dispatch

| Field | Value |
|---|---|
| **ADR** | 033 |
| **Status** | **Accepted** (2026-06-27) — *Proposed → Accepted, **ratified by the maintainer 2026-06-27** (R2 gate): the `FieldSpec::Fn` trusted-core extension for dynamic dispatch is approved, with KC-3 growth (one enum variant) accepted. **FLAG-1 (arity-only-hashing soundness) stays `Declared` — never upgraded (VR-5) — and is an explicit pre-`Enacted` gate:** a machine-checked soundness basis (or a revised type-carrying hash) is required before this moves to **Enacted**.* Prior: **Proposed** (2026-06-27). **FLAG-1 RESOLVED (2026-06-28, in-session maintainer ratification): Path A selected — full function signature (params + return) encoded in the `FieldSpec::Fn` dispatch hash** (see §6 and §10). FLAG-1 moves from "open" to **resolved-pending-implementation**. → **Enacted** is now gated only on landing the full-sig encoding (tracked as a sub-task under M-810; impl must replace `FieldSpec::Fn { arity }` with a variant carrying `FieldTyRef` for each param + return type). Soundness tag stays `Declared` until mechanized (VR-5). Prior proposal record: **Path A (type-carrying hash)** — `FieldSpec::Fn { arity, sig }` was recommended (§10); **Path B (arity-only machine-checked argument) was NOT recommended** (§10.4 — single-producer side-condition structurally false at L0 trusted-input boundary). |
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

**RESOLVED (2026-06-28, in-session maintainer ratification) — Path A selected:** `FieldSpec::Fn` is extended to carry the full function signature (all parameter types + return type), encoded as `FieldTyRef` values in the dispatch hash. This closes the type-soundness gap at the kernel level: two `FieldSpec::Fn` fields with different signatures produce different hashes. See §10 for the full analysis. Implementation: replace `FieldSpec::Fn { arity }` with `FieldSpec::Fn { sig: FnSig }` (or equivalent) carrying `FieldTyRef` for each param and the return type — sub-task under M-810. Soundness tag stays `Declared` until a mechanized proof or full-coverage differential test confirms the new encoding is collision-free (VR-5). → `Enacted` gated on landing this full-sig encoding.

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
| FLAG-1: arity-only encoding collides distinct same-arity fn types on content identity | `Exact` | `data.rs:328–345` (`encode_decl` emits `FIELD_FN`+`u32` arity only — no signature); `content.rs:212` (`Canon::repr`) shows the `Repr` distinction the field hash drops |
| FLAG-1 Path A (type-carrying hash) is kernel-encodable; signature recursion is well-founded | `Declared` | §10.2 — leaves are `Repr` (`Canon::repr`, already injective) / `Data(hash)` (`Canon::hash`); nesting strips one `Fn` layer per step, terminates at `Repr`/`Data` |
| FLAG-1 Path B (arity-only soundness theorem INV-AO) cannot be established | `Declared` | §10.4 — its single-producer side-condition is false at the L0 trusted-input boundary (ADR-003/RFC-0007 admit hand-/fuzz-authored `Construct`s); making it true moves the elaborator into the trusted core, inverting KC-3 |
| FLAG-1 resolution recommends Path A; tag stays `Declared` (not `Proven`) until mechanized | `Declared` | §10.5 — argument is §10.1–10.4; injectivity-over-typed-structure is unmechanized; VR-5 forbids upgrade without a checked basis |

---

## 10. FLAG-1 resolution — type-carrying hash vs arity-only soundness argument

*Status: **proposed resolution**, awaiting maintainer ratification → then `Enacted`. This section is
appended (append-only #3); §6 FLAG-1 is left intact as the original open statement. The argument
below is `Declared`-with-argument throughout — it is **not** `Proven`: no part of it is mechanized,
so VR-5 forbids the upgrade until a checked proof (or a passing differential that exercises the
collision case) exists. This resolves G4+G5 of the ratification map and DN-56 kernel-freeze
condition #3 only **once ratified** — proposing it does not freeze the kernel.*

### 10.1 The soundness hole, stated precisely

`FieldSpec::Fn { arity }` (§2.1) encodes **only** the parameter count into the identity-bearing
content hash. The hash tag scheme (§2.2) is `c.tag(FIELD_FN); c.u32(arity)`. Two function-typed
fields therefore receive the **same** `FieldSpec`-level encoding whenever their arities match,
regardless of their parameter or return *types*.

Concretely, with the kernel's `Repr` set (`crates/mycelium-core/src/repr.rs`: `Binary{width}`,
`Ternary{trits}`, `Dense{dim,dtype}`, `Vsa{model,dim,sparsity}`):

- `field_equal : (Binary{8},  Binary{8})  -> Binary{1}` — `Fn { arity: 2 }`
- `field_equal : (Binary{16}, Binary{16}) -> Binary{1}` — `Fn { arity: 2 }`

These two semantically distinct function types **hash identically** at the field level. Two
dictionary declarations `MkDict_Eq8` and `MkDict_Eq16` that differ *only* in these field types are
therefore the **same content-addressed declaration** (`#T#0` is identical). That is the precise
failure: **content-addressed identity is no longer injective over the program's *typed* structure**
— it is injective only over (arity, data-edges, repr-edges), with the function *signature* erased.

**What can actually go wrong (the exploit shape).** Content identity is load-bearing in three
places the kernel trusts: (i) `DataRegistry` keying (`decls: BTreeMap<ContentHash, DataDecl>`,
data.rs:154) — two distinct typed dictionaries deduplicate to one entry; (ii) `CtorRef` equality —
a `Match`/`project` written against `MkDict_Eq16` will match a value built as `MkDict_Eq8` because
their `CtorRef`s are equal; (iii) the three-way differential (§5) compares results under *one*
`CtorRef` set, so an L1/L0/AOT path that silently accepts the cross-typed value will agree with
itself and the differential will **not** catch it. The kernel's only remaining check on a `Fn`
field is **saturation** (arity — §5.2 property 4), which both `Eq8` and `Eq16` pass identically.

So if the elaborator ever emits — through a bug, or through an adversarially crafted L0 term that
bypasses the elaborator entirely (the L0 fragment is a *trusted-input* surface; a hand-written or
fuzzed `Construct` is admissible input to the registry) — a value that projects `MkDict_Eq8`'s
field and applies it to `Binary{16}` arguments, **the kernel cannot reject it**. The result is a
type-confused application: the `Binary{8}` method body executes over `Binary{16}` operands. Whether
that is *memory*-unsafe depends on the downstream lowering, but it is unconditionally a **silent
type-soundness violation** at the kernel boundary — exactly what G2 (never-silent) forbids the
kernel to permit.

### 10.2 Path A — the type-carrying hash (RECOMMENDED)

**Encode a minimal, resolved type signature into the dispatch key**, so two distinct fn types can
never collide on identity. The variant becomes:

```rust
pub enum FieldSpec {
    Repr(Repr),
    Data(String),
    /// A function-typed field: `arity` explicit parameters with the given resolved signature.
    /// `sig` carries each parameter type and the return type, so two function types that differ
    /// in any parameter/return type hash distinctly (FLAG-1 resolution, ADR-033 §10).
    Fn { arity: u32, sig: FnSig },
}

/// A resolved function signature: parameter types in order, then the return type.
/// `arity == params.len()` is a well-formedness invariant checked at build (never-silent on
/// mismatch — RegistryError::FnArityMismatch).
pub struct FnSig { pub params: Vec<FieldTyRef>, pub ret: Box<FieldTyRef> }

/// A type that can appear as a function parameter or return: the same leaf set a data field can
/// hold, plus nested functions. Bottoms out in `Repr` / `Data(hash)` — no unbounded recursion.
pub enum FieldTyRef {
    Repr(Repr),              // encodes via the existing c.repr() — already injective
    Data(String),            // a data-decl reference by build-name → resolves to ContentHash
    Fn(Box<FnSig>),          // nested function type (higher-order method) — finite, well-founded
}
```

**Why this is kernel-encodable (the key correction to §7.1).** §7.1 argued a full function type
would "conflate the data registry with a type registry" and force "a self-referential encoding that
either loops or requires a separate type-level registry." That argument is **too strong** and is
the load-bearing reason FLAG-1 looked unresolvable-without-cost. It is wrong on the mechanics: every
leaf of a resolved signature is *already* a kernel-encodable term —

- a `Repr` parameter encodes via the existing `Canon::repr` (`content.rs:212`), which is already
  injective over `Repr` (it is exactly how `FieldSpec::Repr` hashes today);
- a `Data` parameter resolves (dependencies-first, exactly as `FieldSpec::Data` does today,
  data.rs:334–343) to a `ContentHash` and encodes via `Canon::hash` — already injective by hash;
- a nested `Fn` parameter recurses, and the recursion is **well-founded**: each nesting strips one
  `Fn` layer and the leaves are `Repr`/`Data`, so the encoding terminates (no fixpoint, no
  type-registry — the recursion is over the *finite* signature term the elaborator already holds).

The new content tag is `FIELD_FN` plus `c.u32(arity)` plus a structural encode of `sig`
(`FN_SIG_PARAMS` count, each param's `FieldTyRef` tagged `FTR_REPR`/`FTR_DATA`/`FTR_FN`, then
`FN_SIG_RET`). Fresh tags, disjoint from `FIELD_REPR`/`FIELD_DATA`/`FIELD_CYCLE`/`FIELD_FN`, so
injectivity is preserved by the same disjoint-tag argument as §2.3. **Cycle interaction:** a `Data`
leaf *inside* a signature is a genuine declaration edge and **must** be threaded through the same
in-cycle placeholder scheme (`FIELD_CYCLE`) as a top-level `Data` field — otherwise a dictionary
whose method takes a recursive data type would mis-hash. This widens FLAG-3's SCC `succ` change:
`succ` must traverse `Data` edges **inside `Fn` signatures** too, not only top-level `Data` fields.
That is a real, bounded cost and is called out as sub-question Q3 below.

**What Path A buys.** The collision of §10.1 is **closed at the kernel level**: `Fn(Binary{8}…)`
and `Fn(Binary{16}…)` now hash distinctly, so `MkDict_Eq8 ≠ MkDict_Eq16` as content-addressed
declarations, their `CtorRef`s differ, and a `Match` written against one **cannot** match a value
of the other — the kernel rejects the type-confused projection by construction (a `Match` on a
`CtorRef` that does not equal the value's `CtorRef` is already a never-silent no-match → explicit
error). Type-soundness for dictionary dispatch no longer *depends on the elaborator being
bug-free*; it is enforced by the trusted core, which is where KC-3 wants a soundness-critical
invariant to live.

**Cost (stated honestly, KC-3).** Path A grows the trusted base by more than one `u32`: it adds
`FnSig`/`FieldTyRef` to the identity-bearing surface, three-or-four new content tags, the
recursive encode arm, the build-time `arity == params.len()` well-formedness check
(`RegistryError::FnArityMismatch`, never-silent), and the FLAG-3 SCC traversal into signatures.
This is a larger KC-3 increment than §1.3 scoped for the `arity`-only variant. The trade is
explicit: **a bounded, one-time trusted-core growth in exchange for moving the dictionary-dispatch
soundness invariant from `Declared`-elaborator-discipline into a kernel-checked property.** That is
the correct side of the KC-3 line — KC-3 is *small auditable kernel*, not *minimal kernel at the
cost of an unchecked soundness obligation*; an auditable kernel that **cannot** express the
soundness check is not the cheaper option, it is the unsound one.

### 10.3 Path B — a machine-checked argument that arity-only is sufficient

Path B keeps `FieldSpec::Fn { arity }` unchanged and instead discharges FLAG-1 with a checked
soundness theorem of the shape:

> **Claimed invariant (INV-AO).** For every well-formed program `P` admitted by the kernel, and
> every dictionary value `d` of declaration `#T` projected at field `i` and applied to arguments
> `a̅`, the function value `f` resolved at `(#T, i)` has a parameter type list whose `Repr`/`Data`
> shape matches the runtime shapes of `a̅` — *even though the kernel hash records only `arity`*.

For INV-AO to hold **as a kernel guarantee**, the following side-conditions would all have to be
*checked* (transparency rule: `Proven` needs checked side-conditions):

1. **Single-producer.** Every `Fn`-fielded `Construct` value reaching the registry/interpreter was
   produced by the type-checking elaborator — i.e. the L0 fragment is **not** an independent
   trusted-input surface for `Fn`-fielded constructs.
2. **Elaborator type-preservation.** The elaborator's instantiation of a dictionary at a concrete
   type is type-correct (it never binds a `Binary{8}` method into a slot used at `Binary{16}`).
3. **No post-elaboration retyping.** No pass between elaboration and execution (mono, swap
   insertion, AOT lowering) can substitute a same-arity-different-type method into a dictionary
   slot.

### 10.4 Why Path B cannot be established (and therefore Path A is recommended)

I cannot establish INV-AO, and I will not assert it (G2/VR-5 — say "unproven" plainly). The
obstruction is **side-condition 1, and it is structural, not merely unproven-yet**:

- The L0 term model is, by the project's own framing, a **trusted-input fragment**: ADR-003 /
  RFC-0007 treat a `Construct` as admissible kernel input independent of how it was produced (the
  three-way differential in §5 exists precisely because the L0 interpreter is a *trusted base* run
  directly, not only as the elaborator's output). A hand-authored, fuzzed, or
  third-party-tool-emitted `Construct` of a `Fn`-fielded declaration is therefore an input the
  kernel **must** judge on its own content — and on its own content, `Eq8` and `Eq16` are
  *indistinguishable*. So side-condition 1 is **false** as stated for the kernel boundary; making
  it true would require declaring the type-checking elaborator part of the **trusted** core, which
  (a) inverts KC-3's placement of the trusted boundary (the elaborator is explicitly the *untrusted*
  layer, §2.1) and (b) is a far larger trusted-base growth than Path A's encoding.

- Even setting that aside, conditions 2–3 are `Empirical` at best (they would rest on the
  elaborator/mono test suite, not a theorem), so the *strongest* honest tag INV-AO could carry is
  `Empirical`-modulo-an-unprovable-side-condition. That is **not** a soundness basis the freeze
  gate (DN-56 condition #3) should accept: DN-56 §3 names FLAG-1 as "a soundness item that must be
  resolved before this primitive can be frozen," and a freeze cannot rest on an invariant whose
  premise (single-producer) the trusted boundary contradicts.

Therefore Path B does not close the hole at the layer where the hole lives. **Per the task's own
instruction — "if you cannot establish them, say so plainly and prefer A" — I recommend Path A.**

### 10.5 Recommendation and tag

**Adopt Path A (type-carrying hash).** It moves dictionary-dispatch type-soundness from an
unprovable elaborator-discipline obligation into a kernel-checked injectivity property, at a
bounded, explicit, one-time KC-3 cost. **Tag: `Declared`-with-argument** — the argument is §10.1–10.4
(disjoint-tag injectivity + well-founded signature recursion + the structural impossibility of
Path B's single-producer premise). It is **not `Proven`**: the injectivity-over-typed-structure
claim and the "kernel rejects the type-confused projection" claim are unmechanized. The upgrade path
to a higher tag is explicit: a property test that two same-arity-different-`Repr` `Fn` fields
produce **distinct** `DataDecl` hashes (the §5.2 property 1/2 extended to signatures), plus a
differential case that builds `MkDict_Eq8` and projects it under an `Eq16`-typed `Match` and
asserts a **never-silent no-match error** — both green — would lift this to `Empirical`; a
mechanized injectivity proof over the encoding would lift it to `Proven`. Until then it stays
`Declared` (VR-5).

### 10.6 Open sub-questions (for maintainer ratification)

- **Q1 — minimality of `sig`.** Must the *return* type be encoded, or do parameter types suffice
  for soundness? Encoding params-only closes the §10.1 argument-side confusion but leaves two fields
  that differ *only* in return type colliding. I recommend encoding **both** params and return (full
  signature) — half a signature is a smaller, subtler version of the same hole — but a maintainer
  may judge return-type confusion unreachable given how dictionaries are projected-then-applied. This
  is a `Declared` judgement either way until tested.
- **Q2 — `Repr`-only vs full `FieldTyRef` leaves.** A cheaper Path A encodes only the *`Repr`* of
  each parameter (ignoring `Data` and nested `Fn` leaves), betting that data-typed and higher-order
  method parameters are rare in v0 dictionaries. This shrinks the KC-3 increment but reintroduces a
  (narrower) collision for `Data`-typed parameters. I do **not** recommend it (it is a partial fix
  wearing a full-fix label — a transparency hazard), but it is the maintainer's KC-3 call.
- **Q3 — FLAG-3 SCC widening.** Path A requires the SCC `succ` (data.rs:378) and
  `canonical_cycle_order` to traverse `Data` edges **inside** `Fn` signatures, and the in-cycle
  `FIELD_CYCLE` placeholder scheme to apply to in-signature `Data` leaves. This is a bounded but
  real extension of FLAG-3's scope; it must be coded and property-tested (a dictionary whose method
  takes the enclosing recursive data type must hash deterministically and not loop).
- **Q4 — interaction with the Swift-existential AOT layout (§7.3).** A type-carrying `FieldSpec::Fn`
  gives the eventual existential-layout ADR the parameter `Repr`s it needs for VWT/PWT sizing *for
  free* (they are now in the kernel identity). This is a side-benefit, not a requirement, but it
  argues further for Path A over Path B. `Declared`.

### 10.7 Effect on the Definition of Done (§8)

If Path A is ratified, the §8 `Accepted → Enacted` checklist item *"FLAG-1 is resolved with a
written argument added to this ADR"* is satisfied by **this §10** (append-only), and the remaining
`Enacted` gates gain: (i) `FieldSpec::Fn { arity, sig }` + `FieldTy::Fn` + `FnSig`/`FieldTyRef` land
with their `FIELD_FN`/`FN_SIG_*`/`FTR_*` tags; (ii) the build-time `arity == params.len()`
never-silent check; (iii) the FLAG-3 SCC widening (Q3); (iv) the two new tests of §10.5 (distinct
hashes for distinct signatures; never-silent no-match on cross-typed projection) green. **§8 is not
rewritten here** (append-only #3) — these are the deltas a future Enacted transition must satisfy;
the maintainer folds them into §8 at ratification, or supersedes this resolution.

---

## Changelog

| Date | Status | Note |
|---|---|---|
| 2026-06-28 | **Accepted** (FLAG-1 resolved-pending-implementation) | **FLAG-1 resolution ratified (in-session): Path A — full function signature (params + return) encoded in `FieldSpec::Fn` dispatch hash.** FLAG-1 moves from "open" to resolved-pending-implementation (§6 + §10 record the analysis). Implementation: `FieldSpec::Fn { arity }` → `FieldSpec::Fn { sig: FnSig }` with `FieldTyRef` per param + return (sub-task M-810). → Enacted gated solely on landing the full-sig encoding. Soundness tag stays `Declared` (VR-5). |
| 2026-06-28 | **Accepted** (unchanged) | **FLAG-1 resolution proposed** (new §10): analyzed both candidate paths and **recommend Path A (type-carrying hash — `FieldSpec::Fn { arity, sig }`)** over Path B (arity-only machine-checked argument). Stated the soundness hole precisely (§10.1: same-arity-different-type fn fields collide on content identity → kernel cannot reject a type-confused dictionary projection, a silent G2 violation). Showed Path A *is* kernel-encodable — every signature leaf bottoms out in `Repr` (already `Canon::repr`-injective) or `Data(hash)`, recursion well-founded — correcting §7.1's over-strong "would conflate a type registry" claim. Showed Path B **cannot** be established: its single-producer side-condition is structurally false at the L0 trusted-input boundary, and adopting it would move the type-checking elaborator into the *trusted* core (inverting KC-3). Tagged the resolution **`Declared`-with-argument, NOT `Proven`** (unmechanized — VR-5); named the test/proof upgrade path. **Status stays `Accepted`** — this is a proposal pending maintainer ratification → then `Enacted` (append-only #3; §6 FLAG-1 left intact; §8 DoD not rewritten — deltas noted in §10.7). Resolves G4+G5 of the ratification map / DN-56 kernel-freeze condition #3 **only once ratified**. FLAGs raised (cannot edit): Doc-Index, CHANGELOG, issues.yaml, DN-56, `docs/adr/README.md` index — all orchestrator/owner-owned. (VR-5; G2; KC-3.) |
| 2026-06-27 | **Proposed** | Initial design. Authored as the trusted-core `FieldSpec`-abstract-function-field ADR called for by DN-37 §8 Q3 (2026-06-27 maintainer ruling) and RFC-0019 §4.5 changelog (M-673 deferred normative target). Enacts no code; moves no spec status. CHANGELOG / Doc-Index / issues.yaml / docs/api-index owned by the integrating orchestrator. (Append-only; VR-5; G2.) |
