# Design Note DN-103 — Impl-Level Generic-Parameter Slot (the ENB-3 grammar-slot close)

| Field | Value |
|---|---|
| **Note** | DN-103 |
| **Status** | **Draft** (2026-07-10). Authored alongside the **first landable increment** of M-1026 (ENB-3). It records the design of the impl-level **type-parameter slot** (`impl[T] …`) — surface, grammar, AST, checking, monomorphization — recommends a scoping decision **for ratification**, and **enacts nothing** and **moves no other doc's status** (house rule #3, append-only). Tags are `Empirical` where read against the code / witnessed by a running differential, `Declared` for any design not yet ratified (VR-5). |
| **Decides** | *Proposes, for ratification:* (1) an `impl[T, …]` **type-parameter slot** parsed **immediately after the `impl` keyword**, reusing the existing unbounded-name parser (`parse_type_params_opt`) — a `: bound` here is the same never-silent refusal as on a `type`/`trait` head (bounds live only on `fn` type-params, RFC-0019 §4.1); (2) the slot is **threaded into the inherent-impl AST** (`InherentImplDecl.params`), not flattened at parse time — so the impl block stays faithful for printing/round-trip and any AST consumer; (3) at the **Phase-0 desugar** each lifted method gains the impl's params **prepended** to its own `fn` type-parameters, so the method becomes an ordinary **generic free function** and **monomorphization reuses the existing fn-generics path** (zero new mono code, KC-3/DRY); (4) a **duplicate** between an impl param and a method's own param is the existing never-silent duplicate-type-parameter refusal (G2); (5) the **v0 scope restriction**: the slot is honored on **inherent** impls (`impl[T] Foo[T] { … }`); a non-empty slot on a **trait-instance** impl (`impl[T] Trait for Foo[T]`) is a **never-silent refusal** deferring generic trait-instance coherence (RFC-0019 §4.5) to a follow-up; (6) **lifetime erasure is N/A** — Mycelium v0 has no lifetime surface, so the slot is type-params only (the DN-99 "decide lifetime erasure" question resolves to "no lifetimes to erase"). It does **not** edit `issues.yaml`, `CHANGELOG.md`, or `Doc-Index.md` (the integrating session owns those). |
| **Feeds** | DN-99 §A2 / register row #63 (generic-parameterized impl-block), ENB-3; M-1026; RFC-0019 (traits/generics — the generic + monomorphization machinery this note reuses, already Enacted); DN-03 §1 / M-664 (the inherent-impl block this note extends); DN-26 (SCC self-hosting, the Rust↔`.myc` dual); DN-34 §8 (surface-gap census). |
| **Grounds on** | KC-3 (small kernel — no new L0 node, no new mono pass; the slot desugars into the existing generic-`fn` form), DRY (reuse `parse_type_params_opt` + the fn-generics monomorphizer), G2 (never-silent — a refused bound / a generic trait-impl / a duplicate param each prints the fix), VR-5 (no tag upgraded past its basis), KISS/YAGNI (inherent-only, type-params-only over a full generic-trait-instance + width-param close). |
| **Date** | July 10, 2026 |
| **Task** | M-1026 (ENB-3) — impl-level generic-parameter slot. |

> **Grounding + honesty (house rule #4 / VR-5 / G2).** This note records a design and a running
> increment; it does **not** take a decision (house rule #3 — the maintainer ratifies). Empirical claims
> are witnessed by the differential/conformance witnesses named in §7. The slot semantics and the scope
> restriction are `Declared` until ratified. **No sycophancy:** §3 confronts the two genuine forks
> (parse-time flatten vs AST slot; inherent-only vs generic trait-instance) and §6 states the residuals
> (generic trait impls, width-param impls) plainly rather than claiming a whole "generic impl".

---

## §1 Purpose

Close the impl-level-generics surface gap (DN-99 register row #63 / §A2, ENB-3). A semcore/stdlib porter
translating a Rust `impl<T> Foo<T> { fn get(&self) -> T { … } }` today has no faithful target: the `.myc`
frontend's inherent-impl block (`impl T { fn … }`, DN-03 §1 / M-664) carries **no type-parameter slot**
(verified: `parse_impl_item` at `crates/mycelium-l1/src/parse.rs:1179` parses the head base type then
branches on `for`/`{` with no `[T]` between `impl` and the head; `InherentImplDecl` at
`crates/mycelium-l1/src/ast.rs:217` has only `for_ty` + `methods`). The interim workaround —
**impls-as-free-functions** — keeps the stdlib lane unblocked but flattens the block, losing the surface
association DN-99 §A2 calls for in a "faithful impl-block-preserving self-host". This note adds the slot
as **surface + a Phase-0 lowering over the already-Enacted RFC-0019 generic-function machinery** — **no
new kernel semantics, no new L0 node, no new monomorphization pass** (KC-3).

## §2 The surface + grammar

```
impl_item   ::= 'impl' type_params? base_type impl_tail
impl_tail   ::= 'for' type_ref impl_body        // trait-instance (RFC-0019 §4.1)
              | impl_body                        // inherent (DN-03 §1 / M-664)
type_params ::= '[' ident (',' ident)* ']'       // reuses parse_type_params_opt (unbounded names)
```

- **`impl[T] Foo[T] { … }`** — an inherent block generic in `T`. The `[T]` slot sits **immediately after
  the `impl` keyword**, before the head. This is unambiguous: no `base_type` production begins with `[`
  (heads are repr keywords `Binary{…}`/`Ternary{…}`/… or a `Named` identifier; type **arguments** `[…]`
  are a *suffix* on a `Named` head), so a leading `[` after `impl` is always the type-params slot
  (verified against `parse_base_type`, `crates/mycelium-l1/src/parse.rs:1522`).
- **Backward compatible** — the current `impl Foo[T] { … }` (no impl-level slot; `[T]` is the head's type
  argument) still parses: after `impl` the next token is the identifier `Foo`, so the optional slot is
  empty and unconsumed, then the head `Foo[T]` parses as before.
- **Bounds refusal** — a `: bound` inside the impl slot is the **same never-silent refusal** the parser
  already raises for a bound on a `type`/`trait` head (`parse_type_params_opt`,
  `crates/mycelium-l1/src/parse.rs:1109`): bounds live **only** on `fn` type-parameters (the dictionary
  site, RFC-0019 §4.1). Write the bound on the bounded method instead.

## §3 The design forks (confronted, not glossed)

**Fork 1 — parse-time flatten vs AST slot.** Two ways to make `impl[T] Foo[T] { fn get(x: Foo[T]) => T }`
monomorphize:

- **(A) Parse-time flatten.** At parse, prepend `T` to each method's `fn` type-params and construct the
  ordinary `InherentImplDecl` with **no** new field — the method is born as `fn get[T]` over `(x: Foo[T]) => T`.
  *Zero* AST/walker/encoder blast radius. **But** it discards the impl-block structure at the earliest
  stage: the AST no longer records "these methods share an impl-level `T`", so the canonical printer
  (`print_inherent_impl_decl`) and any AST consumer see a flattened form — i.e. it **is** the
  impls-as-functions interim DN-99 §A2 says ENB-3 should *improve upon*.
- **(B) AST slot + Phase-0 desugar-prepend.** Carry `params` on `InherentImplDecl`; keep the impl block
  intact through parse/ambient/print; prepend the params to each method **at the existing Phase-0
  desugar** (`checkty.rs` `check_phylum_inner`, where inherent methods are already lifted verbatim to
  `Item::Fn`). Costs a struct field + its mirror sweep, but the AST stays faithful (round-trip,
  `EXPLAIN`, a future qualified `T::m` surface all see the real block).

**Recommendation: (B).** DN-99 §A2 asks to "thread [the slot] into the impl AST" for a
*faithful impl-block-preserving* self-host; (A) is definitionally the flatten it supersedes. The extra
cost is a mechanical mirror sweep, paid once. *(Note: `mycelium-fmt` is token-stream based — it renders
`impl [ T ]` from the tokens regardless of the AST — so fmt round-trips under either fork; the fidelity
argument is about the AST/printer/`EXPLAIN` layer, not fmt.)*

**Fork 2 — inherent-only vs generic trait-instance.** `impl[T] Trait for Foo[T] { … }` is a **generic
trait instance**. Honoring it correctly means extending RFC-0019 §4.5 coherence/orphan checking and
dictionary construction over a **parametric** instance head (one `impl` standing for a family of
instances) — a materially larger change than the inherent lowering (which reduces to already-supported
generic free functions). DN-99 §A2's DoD and register row #63 target `impl[T] Foo[T]` (inherent).

**Recommendation: scope this increment to inherent impls; refuse a non-empty slot on a trait impl with a
never-silent deferral (G2).** The message names the deferral and the two unblocked paths (inherent impl,
or impls-as-functions). Generic trait instances get their own follow-up (an RFC-0019 amendment), tracked
as a residual (§6). This keeps the landable increment honest and small (KISS/YAGNI) without ever silently
accepting a form we don't yet check soundly.

**Fork 3 — lifetime erasure (the DN-99 open question).** DN-99 §A2 flags "decide lifetime erasure".
Mycelium v0 is a **value-semantics language with no lifetime/borrow surface** — there is no `'a` syntax,
no reference type, nothing to erase. The slot therefore admits **type parameters only**. The question
resolves to **N/A**: there are no lifetime parameters in the impl slot to erase or keep. (Width
parameters — `impl[N] Binary{N}` — are a *separate* deferral: DN-42 scopes width params to free
functions; the impl slot is type-params (`ParamKind::Type`) only in v0. Residual §6.)

## §4 Checking + monomorphization (the reuse)

The inherent-impl block already lowers, in the Phase-0 desugar of `check_phylum_inner`
(`crates/mycelium-l1/src/checkty.rs`), to its methods lifted **verbatim** as top-level `Item::Fn`s (M-664
— "methods are ordinary explicitly-typed free functions"). The **only** change: before lifting each
method, **prepend the impl's `params`** (as unbounded `TypeParam { kind: Type, bounds: [] }`) to that
method's `sig.params`. Consequences, all free:

1. The lifted method is now a **generic free function** — every existing generic-`fn` path (checking,
   the width/type classification, and crucially **monomorphization**, RFC-0019 / M-673) applies
   **unchanged** (DRY; zero new mono code, KC-3).
2. A call `get(some_foo_of_Binary8)` infers `T = Binary{8}` and monomorphizes to `get$Binary8` exactly as
   any generic free function does; two call sites at two type args emit two specializations — the §7
   witness.
3. A **duplicate** between an impl param and a method's own param — an `impl[T] Foo[T]` whose method re-binds `T` as `fn g[T]` — is
   caught by the existing duplicate-type-parameter check on the lifted sig (never-silent — G2).
4. The impl's `for_ty` stays advisory metadata in v0 exactly as today (no qualified `T::m` call surface
   binds to it — the M-664 known gap is unchanged; the generic slot does not alter that boundary).

No new L0 node, no new kernel rule, no new monomorphization entry point: the slot is **pure front-matter
that desugars into the existing generic-`fn` vehicle** (KC-3).

## §5 The Rust↔`.myc` dual (DN-26)

Per DN-26 the change lands in **both** the Rust frontend (`crates/mycelium-l1`) and the self-hosted
`.myc` mirror (`lib/compiler/*.myc`), at the **parse + AST layer** they share:

- **Rust:** `parse_impl_item` parses the slot; `InherentImplDecl` gains `params`; the Phase-0 desugar
  prepends; `ambient.rs` (resolve + print) and `checkty.rs` (`collect_tuple_arities`) thread the field;
  the trait-impl slot-refusal is raised in `parse_impl_item`.
- **`.myc`:** the `IID` constructor gains the params list in each mirror declaration (`ast.myc`,
  `ambient.myc`, `parse.myc`, `semcore.myc`), with accessors, the fold-walkers, and the printer updated;
  `parse_impl_item_tail` parses the slot and raises the same trait-impl refusal. The **desugar-prepend
  itself is Rust-only** — the `.myc` semcore mirror does **not** port the Phase-0 lift (it is documented
  as pre-desugared in the real pipeline, `lib/compiler/semcore.myc` §collect_tuple_arities header), so the
  mirror carries the slot structurally without re-implementing the lift. This is a **pre-existing,
  documented** division of the port, not a new silent gap.

The parity contract the dogfood dual checks (§7): the `.myc` frontend files still `myc check`-clean with
the extended `IID`, and a generic inherent method behaves identically Rust-oracle vs `.myc` at the layer
each implements.

## §6 Residuals (stated plainly — G2/VR-5)

- **Generic trait instances** (`impl[T] Trait for Foo[T]`) — deferred; a non-empty slot on a trait impl
  is a never-silent refusal (§3 Fork 2). Needs an RFC-0019 §4.5 amendment (coherence/orphan + dictionary
  construction over a parametric instance head). Residual tracked for a follow-up `enb`.
- **Width-parameter impls** (`impl[N] Binary{N}`) — deferred; the slot is type-params only in v0 (§3 Fork
  3). DN-42 scopes width params to free functions; a width-generic impl awaits that scope widening.
- **`for_ty` resolution** — unchanged M-664 v0 boundary: the impl head is advisory until a qualified
  `T::m` call surface lands (documented, never-silent).

## §7 Verification (Definition of Done)

- **Parse** — `impl[T] Foo[T] { fn get(x: Foo[T]) => T = … }` parses; the `params` ride
  `InherentImplDecl`; the current `impl Foo[T] { … }` (no slot) is unchanged (accept).
- **Check + monomorphize** — the generic inherent method checks, and `monomorphize(env, "main")` over a
  `main` that calls it at **two** type arguments emits **two** specializations (`get$Binary8`,
  `get$Binary16`), each with empty type-params (the M-673 closure invariant) — the ENB-3 witness.
- **Reject (never-silent, G2)** — (a) `impl[T] Trait for Foo[T] { … }` → the generic-trait-instance
  deferral refusal; (b) `impl[T: Cmp] Foo[T] { … }` → the impl-slot bound refusal; (c)
  an `impl[T] Foo[T]` whose method re-binds `T` as `fn g[T]` → the duplicate-type-parameter refusal.
- **Dual (DN-26 / `/myc-dogfood`)** — the `.myc` frontend `myc check`-clean with the extended `IID`; the
  Rust-oracle vs `.myc` differential agrees on the accept + the two rejects at the layer each implements.
- **Guarantee** — `Declared` (surface + a structural desugar into the existing generic-`fn` vehicle),
  upgraded to `Empirical` by the running conformance + differential witnesses above; **not** upgraded past
  that (no `Proven` claim — VR-5).

## §8 Changelog

- **2026-07-10** — DN-103 created (**Draft**). Recorded the impl-level type-parameter slot design
  (surface/grammar §2; the parse-flatten-vs-AST-slot, inherent-vs-trait-instance, and lifetime-erasure
  forks §3; the fn-generics reuse §4; the Rust↔`.myc` dual §5; residuals §6; the DoD/witnesses §7).
  Authored READ + DN + the M-1026 increment only — no edit to `issues.yaml`, `CHANGELOG.md`, or
  `Doc-Index.md` (integration-owned; FLAGGED up). `Empirical` where cited against the tree (dev
  `bbf01f37`) or witnessed by a running test; `Declared` for the unratified slot semantics + scope
  decision. Append-only; status advances only by maintainer ratification (house rule #3).
