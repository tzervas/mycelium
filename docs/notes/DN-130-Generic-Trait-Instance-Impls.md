# Design Note DN-130 — Generic Trait-Instance Impls (`impl[T] Trait for Foo[T]`) — the impl-generics residual of DN-103

| Field | Value |
|---|---|
| **Note** | DN-130 |
| **Status** | **Accepted** (2026-07-12, ratified under explicit maintainer delegation — mirrors the DN-115/117/118/122/123/124/125/126/127/128/129 precedent). Was **Draft** (2026-07-12, same day). **Accepted, not Enacted** (house rule #3) — **builds nothing** yet; every mechanism stays `Declared`/unbuilt until the FLAGGED build issue (M-1087, minted at this close-out, `depends_on: [M-1080]`) lands and is differential-witnessed. Does not edit `crates/**`; `Doc-Index.md`/`CHANGELOG.md`/`issues.yaml` are applied by this ratification's integration close-out (recorded here, append-only). |
| **Ratification basis (recorded verbatim, 2026-07-12)** | The generic trait-instance impl `impl[T] Trait for Foo[T]` — the one residual DN-103 explicitly deferred — is a **parametric instance head**, monomorphized as a **family of concrete instances** (M-673 α-substitution) with **zero new L0/kernel node** (Native Equivalent, DN-111). Coherence is keyed on the **type-constructor head** (`(Trait, Foo)`, not a fully-applied type): global uniqueness, reject-overlap (no specificity order, RFC-0019 §4.5), and an orphan rule keyed on the constructor's home, reusing DN-122's **home-qualified `CoherenceView`**. **Scoped to v1**: single-parameter, structurally-covering, non-overlapping heads only — blanket impls (`impl[T] Trait for T`), nested/non-covering heads, higher-kinded heads, and conditional/overlapping coherence are all **explicit never-silent refusals** (G2), not silently accepted. **Real landing-order dependency, honestly stated (not a landed reuse, VR-5):** DN-130's coherence implementation needs **M-1080** (DN-122's home-qualified `CoherenceView` MVP, `status: todo`) to land first — the *landed* `CoherenceView` today is phylum-wide/pub-blind, not home-qualified; only the `type_head`/`qualify_type_name` primitives (M-1036) are reused as landed. Alternative 1 (this design) is ratified over the status-quo refusal (not a family, breaks on new `C`), a full overlapping-instance solver (RFC-0019 §4.5 explicitly rejects non-confluence), and runtime-dictionary dispatch (diverges from the landed mono path). Coherence stays `Declared`-with-argument (not machine-checked) — no `Proven` upgrade. Gate PASS after the citation-correction patch — ratified on the merits under maintainer delegation; this note's own reasoning (§1–§9) is not re-litigated, only executed and recorded (VR-5). |
| **Verify-first reframing (mitigation #14 — the register lags the code)** | The naive framing — "impl-level generics is an open L3 grammar residual (DN-119 §3 L3-G4 / DN-99 register row #63)" — is **substantially superseded**. **DN-103 (Accepted 2026-07-11)** designed and landed the impl-level type-parameter slot for the **inherent** form: `impl[T] Foo[T] { … }` parses (`parse.rs:1220`), rides `InherentImplDecl.params` (`ast.rs:231`), and desugars at Phase-0 by prepending the impl's params to each lifted method so monomorphization reuses the existing fn-generics path (`checkty.rs:2053`; witnessed `tests/check.rs:1493`, `generic_inherent_method_monomorphizes_across_two_type_args`). **The genuine open residual is the one form DN-103 explicitly deferred (its §6 / §3 Fork 2): the generic *trait-instance* impl `impl[T] Trait for Foo[T]`.** This note designs *only* that residual; it does not re-decide DN-103 (append-only). |
| **Decides (proposes, for ratification)** | (1) the **native mechanism** — a *parametric instance head* on a trait-instance impl, `impl[T] Trait for Foo[T] { … }`, checked and monomorphized as a **family of concrete instances**, one per reachable `T`, reusing the landed dictionary-free static resolution (RFC-0019 / M-673 / DN-55) with **zero new L0/kernel node**; (2) the **coherence extension** — the RFC-0019 §4.5 orphan rule + global uniqueness + reject-overlap re-stated over **parametric** instance heads (the overlap check becomes unification of two parametric heads; the orphan rule keys on the head's type-constructor and the free variables), reusing DN-122's home-qualified `CoherenceView`; (3) the **v1 scope restriction** — *single-parameter, structurally-covering, non-overlapping* parametric heads only (the head `Foo[T]` binds each impl param exactly once as a direct type-constructor argument); everything richer (multi-param overlap resolution, blanket `impl[T] Trait for T`, negative reasoning, higher-kinded) is a never-silent refusal (G2), tracked as a residual (§8); (4) the **build split** — a checker-first change (coherence + a per-instantiation dictionary synthesis at mono) with the transpiler emit-arm as a fast-follow, **no runtime/L0 growth**. It does **not** edit `issues.yaml`, `CHANGELOG.md`, or `Doc-Index.md` (integration-owned — FLAGGED up). |
| **Feeds** | DN-103 §6 (the deferred generic-trait-instance residual this note picks up); DN-119 §3 L3-G4 (impl-generics — the inherent half closed by DN-103, this is the remaining half); DN-99 register row #63 (generic-parameterized-impl-block) + row #27/#90 (dyn/auto-trait); M-876 (surface completeness for transpilation); M-1026/ENB-3 (the DN-103 increment this extends). |
| **Depends on** | RFC-0019 (Enacted — traits, dictionary-passing/monomorphization, coherence = orphan + global uniqueness + reject-overlap, KC-3 node budget unchanged); **DN-103** (Accepted — the inherent-impl generic slot + the Phase-0 desugar-prepend vehicle); **DN-122** (Accepted, but **implementation unbuilt** — its home-qualified `CoherenceView` extension is tracked as **M-1080** (`status: todo`, `depends_on: [M-1060, M-1079]`); the *landed* `CoherenceView` today (`checkty.rs:2252`) is phylum-wide/pub-blind, not home-qualified, so DN-130's coherence work has a real **landing-order dependency** on M-1080, or must build the home-qualification itself — VR-5, Accepted-but-unbuilt is not a landed reuse); DN-55 (static specialization = zero kernel primitives); DN-112/DN-113 (home-qualified identity + acyclic import closure, both landed — M-1036/M-1060, whose `type_head`/`qualify_type_name` primitives DN-130 does reuse). |
| **Grounds on** | KC-3 (small kernel — check-time + mono only, no new L0 node); DRY (reuse the DN-103 desugar vehicle + the M-673 monomorphizer + the DN-122 `CoherenceView`); G2 (never-silent — an orphan / overlap / out-of-scope head each prints the fix); VR-5 (no tag upgraded past its basis — every mechanism here is `Declared` until built + differential-witnessed); KISS/YAGNI (single-parameter covering heads over a full overlapping-instance solver). |
| **Date** | July 12, 2026 |
| **Author** | design-reasoner (Opus). Owns only this note. |
| **Task** | impl-generics cluster — the generic trait-instance residual (companion: DN-131, bounds on non-fn sites). |

> **Grounding + honesty (house rule #4 / VR-5 / G2).** This is a **design recommendation**, not a
> decision (house rule #3 — the maintainer ratifies). `Empirical` claims are read against `dev@fa53dc46`
> and cite `file:line`; every proposed mechanism is `Declared` (unbuilt). **No sycophancy:** §1 states
> plainly that the headline "impl-generics gap" is mostly already closed (DN-103), so this note is
> narrow; §7 runs the recommendation through the coherence sequences that would break it and flags the
> higher-kinded / blanket-impl / overlap cases as honestly out of v1 scope rather than hand-waving them.

---

## §1 Purpose — and the honest scope (verify-first)

The task framed "impl-level generics" as an open grammar residual. It mostly is not, any longer. Read
against `dev@fa53dc46`:

| Form | State today | Evidence (`file:line` @ fa53dc46) |
|---|---|---|
| `impl[T] Foo[T] { … }` (inherent, generic) | **LANDED + Accepted (DN-103)** | `parse.rs:1220` parses the slot; `ast.rs:231` `InherentImplDecl.params`; `checkty.rs:2053` desugar-prepend; `tests/check.rs:1493` two-specialization witness (`generic_inherent_method_monomorphizes_across_two_type_args`) |
| `impl Foo[T] { … }` (inherent, `[T]` as head arg — no slot) | LANDED (M-664) | `parse.rs:1264` inherent branch |
| **`impl[T] Trait for Foo[T] { … }` (generic trait instance)** | **REFUSED / deferred** | `parse.rs:1234`–`1243` never-silent refusal, deferring "generic trait-instance coherence (DN-103 §3 / RFC-0019 §4.5)" |

So the only genuinely-open piece of the impl-generics gap is the **generic trait-instance** form. That is
what a Rust porter hits when translating `impl<T> Display for Wrap<T> { … }` — a single `impl` block that
stands for a **family** of instances (`Wrap<i8>`, `Wrap<i16>`, …), not one. DN-103 correctly refused it
rather than silently accept a form whose coherence it could not yet check soundly (G2). This note closes
that refusal with a design.

## §2 The PROBLEM, and Mycelium's native-solution class (DN-110/DN-111)

**Do not translate Rust's mechanism; translate the problem it solves (DN-111 §9 step 2).** Rust's
`impl<T> Trait for Foo<T>` is not "a generic block"; the underlying problem is: *provide a trait's methods
uniformly for a whole type-constructor family `Foo[_]`, without writing one impl per element of the
family, while keeping instance selection unambiguous (coherent).*

- **{exact?}** — yes: each concrete instantiation `Foo[C]` must behave exactly as a hand-written
  `impl Trait for Foo[C]` would.
- **{native?}** — yes: Mycelium already has the two ingredients — trait instances as content-addressed
  registry entries (RFC-0019 §4.2) and dictionary-free static specialization (DN-55 / M-673). A parametric
  head is a *quantifier over the existing instance vehicle*, not a new vehicle.

**Classification (DN-111 taxonomy): Native Equivalent.** The parametric trait-instance impl maps onto the
existing instance-decl + monomorphization machinery with no semantic approximation and no interop bridge.
The only genuinely new work is *coherence over a parametric head* (§4) — an extension of a check that
already exists, not a new philosophy.

## §3 The surface + grammar

The grammar slot **already exists** — DN-103 added `type_params?` immediately after `impl` (`parse.rs:1220`
via `parse_type_params_opt`). What v1 does is **lift the trait-branch refusal** (`parse.rs:1234`) for the
scoped subset, threading the impl params through to the trait-instance AST:

```
impl_item   ::= 'impl' type_params? base_type impl_tail
impl_tail   ::= 'for' type_ref impl_body        // trait instance — v1 extends this arm
              | impl_body                        // inherent (DN-103, landed)
type_params ::= '[' ident (',' ident)* ']'       // unbounded names (a ': bound' is DN-131's scope, refused here)
```

- `impl[T] Trait for Foo[T] { fn m(x: Foo[T]) => T = … }` — the impl introduces `T`; the trait-instance
  head `Foo[T]` and the method signatures are checked with `T` in scope as an abstract type variable
  (`Ty::Var`, RFC-0019 §4.6 repr-opaque).
- **Backward compatible** — every existing `impl Trait for C` (monomorphic head) has an empty slot and is
  unchanged; the DN-103 inherent form is unchanged.
- **AST** — `ImplDecl` (`ast.rs`, the trait-instance decl) gains a `params: Vec<TypeParam>` field, mirroring
  DN-103's `InherentImplDecl.params` (fork-1 resolved for DN-103 as *AST slot, not parse-flatten* — this
  note follows that ratified choice for fidelity: the block stays faithful for print/`EXPLAIN`).
- **`: bound` in the slot is still refused here** — bounds on the impl slot are DN-131's scope. `impl[T]`
  (unbounded) is DN-130; `impl[T: Bound]` is the DN-130×DN-131 intersection (§8).

## §4 Coherence over a parametric instance head (the hard part)

RFC-0019 §4.5 defines coherence for **monomorphic** heads: for each `(Trait, Type)` pair at most one
`InstanceDecl` (global uniqueness), the orphan rule keyed on the trait's or the type's home nodule, and
overlapping instances rejected. A parametric head `Foo[T]` makes three of these non-trivial:

1. **Global uniqueness → keyed on the type-constructor head.** `impl[T] Trait for Foo[T]` claims the whole
   family `Foo[_]`. The uniqueness key becomes `(Trait, Foo)` — the trait plus the type *constructor*, not
   a fully-applied type. A second `impl[U] Trait for Foo[U]` is the same key ⇒ the existing never-silent
   duplicate-instance error (G2). A monomorphic `impl Trait for Foo[Binary{8}]` **overlaps** the parametric
   one (both apply at `Foo[Binary{8}]`).
2. **Overlap = unifiability of two heads.** Two instance heads overlap iff their type terms unify (§4.5's
   "there exists a `T` for which both apply"). For the covering single-parameter subset this is decidable
   and cheap: `Foo[T]` unifies with `Foo[C]` (any `C`) and with `Foo[U]`, but not with `Bar[_]`. v1
   **rejects any overlap** (RFC-0019's Haskell-98/Rust rule — no specificity order), so a parametric impl
   and *any* other impl for the same `(Trait, Foo)` is an explicit coherence error. This is the same
   reject-overlap rule, applied to a head with a variable.
3. **Orphan rule over the constructor.** The orphan rule (RFC-0019 §4.5, extended to the home boundary by
   DN-122) keys on "the impl is in the home of the trait, or the home of the type." For a parametric head
   the relevant home is `Foo`'s declaration home (the type-constructor), which is exactly what DN-112's
   `type_head`/`qualify_type_name` already computes (`checkty.rs:296`, landed M-1036). **The landed
   primitives are reusable, but the home-qualified coherence substrate itself is not yet built — this is
   a real landing-order dependency, not a landed reuse (VR-5).** The currently-landed `CoherenceView`
   (`checkty.rs:2252`) is phylum-wide and pub-blind (`traits`/`types: BTreeSet<String>`), not
   home-qualified. The home-qualified extension DN-130 needs is DN-122's own **unbuilt** MVP, tracked as
   **M-1080** (`status: todo`, `depends_on: [M-1060, M-1079]`) — DN-130's coherence implementation needs
   M-1080 to land first, or must build the home-qualification itself.

**Soundness boundary (VR-5).** Like RFC-0019 §4.5 and DN-122, the coherence result here is
**`Declared`-with-argument**, not machine-checked. Ratification does not upgrade that tag; a future
mechanization (the RFC-0019 §9 open) is the basis for any `Proven` upgrade.

## §5 Lowering — monomorphization, dictionary-free (reuse M-673 / DN-55)

No new kernel node, no runtime dictionary (RFC-0019's literal §4.5 `Construct` form stays deferred; the
landed path is monomorphization — M-673). A parametric trait-instance impl lowers exactly as the inherent
form does, one step earlier in resolution:

1. At a call site the checker resolves the receiver's concrete type `Foo[C]`, looks up the parametric
   instance for `(Trait, Foo)`, and **instantiates** it at `T ↦ C` — producing a concrete
   `InstanceDecl` for `Foo[C]` on demand (α-substitution over the method bodies; the same substitution the
   generic-fn monomorphizer already performs, M-673).
2. That concrete instance monomorphizes to a mangled specialization (`m$Trait$Foo$C`) with **no reachable
   type variable** — the M-673 closure invariant, the same one `tests/mono.rs:222` asserts for
   `cmp$Cmp$Binary8`.
3. Two call sites at two type args emit two specializations; identity fragmentation is *recorded, not
   hidden* (the DN-55 / M-673 discipline). `EXPLAIN` shows the parametric impl and the chosen instantiation
   (S4).

**Net kernel/runtime delta: zero** (KC-3). The change is: coherence keys on the constructor head (§4), and
the instance registry synthesizes a concrete `InstanceDecl` per reachable instantiation (a substitution the
monomorphizer already knows how to do).

## §6 Ranked alternatives + recommendation (the objective function)

**Objective (weighted):** *faithfulness* to the Rust family semantics (3) · *soundness of coherence* (3) ·
*kernel/runtime cost* KC-3 (2) · *reuse of landed machinery* DRY (2) · *v1 build size* KISS/YAGNI (1).

| # | Alternative | Faithful (3) | Coherence-sound (3) | KC-3 cost (2) | Reuse (2) | v1 size (1) | Score |
|---|---|---|---|---|---|---|---|
| **1** | **Parametric head, single-param covering subset; coherence keys on constructor head; per-instantiation mono synthesis** (this note) | ✔ full (3) | ✔ decidable overlap, reuses DN-122 view (3) | zero new node (2) | reuse M-673 + DN-122 (2) | small (1) | **11** |
| 2 | Keep the DN-103 refusal; require the porter to hand-expand to one monomorphic `impl Trait for Foo[C]` per used `C` | ✗ not a family; breaks on new `C` (0) | ✔ trivially (3) | zero (2) | full (2) | zero (1) | 8 |
| 3 | Full overlapping-instance solver (specificity order, blanket impls, negative reasoning) | ✔ + more (3) | ✗ non-confluence, RFC-0019 §4.5 explicitly rejects (0) | new solver, fragile (0) | little (0) | large (0) | 3 |
| 4 | Runtime dictionary dispatch (RFC-0019 §4.3 literal `Construct`) for parametric instances | ✔ (3) | ✔ (3) | runtime dict = deferred form, extra vehicle (0) | diverges from landed mono path (0) | large (0) | 6 |

**Recommendation: Alternative 1.** It is the Native-Equivalent close — a quantifier over the *existing*
instance + monomorphization vehicle, with coherence keyed on the type-constructor head (reusing DN-122's
home-qualified `CoherenceView`), scoped to the single-parameter covering subset where overlap is decidable
and the reject-overlap rule stays confluent. Alternative 2 (the status quo) is not a family and silently
breaks the moment a new element is used — it fails the faithfulness objective DN-99 §A2 set for a
"faithful impl-block-preserving self-host." Alternative 3 is exactly the non-confluent design RFC-0019 §4.5
rejects on first principles. Alternative 4 introduces a second lowering vehicle the project deliberately
deferred (M-673 chose mono).

## §7 Adversarial stress-test (VR-5 — argue against the recommendation)

Run Alternative 1 through the sequences that would break it:

- **Overlap with a monomorphic sibling.** `impl[T] Trait for Foo[T]` **and** `impl Trait for Foo[Binary{8}]`
  in scope. Both apply at `Foo[Binary{8}]`. **Verdict:** explicit coherence-overlap error naming both
  (§4.2). *This is correct and intended* — v1 has no specificity order (RFC-0019 §4.5). Honest cost: a
  porter who wrote a Rust blanket impl **plus** a specialized one (legal in Rust with `min_specialization`)
  cannot express both in v1 — a never-silent refusal, tracked as a residual (§8). **Flagged, not hidden.**
- **Blanket impl `impl[T] Trait for T`.** The head is a bare variable — it unifies with *every* type, so it
  overlaps every other instance for `Trait`. **Verdict:** out of v1 scope — refused never-silently (the head
  is not a covering type-constructor application). This is the classic coherence footgun (Rust restricts it
  too); deferring it keeps v1 confluent. **Flagged (§8).**
- **Orphan across homes.** `impl[T] ForeignTrait for LocalFoo[T]` where `ForeignTrait`'s home ≠ this phylum.
  **Verdict:** governed by DN-122's home-qualified orphan closure exactly as the monomorphic case — the
  parametric head keys on `LocalFoo`'s home (local), so it is *not* an orphan. Reuses DN-122; no new rule.
- **Nested / non-covering head.** `impl[T] Trait for Foo[Bar[T]]` (the param appears under a second
  constructor). Overlap-checking stays decidable (unification), but the *coherence key* is muddier
  (`(Trait, Foo)` claims all `Foo[_]`, but this impl only covers `Foo[Bar[_]]`). **Verdict:** refuse in v1 —
  restrict the head to `Foo[T₁, …, Tₙ]` with each `Tᵢ` a distinct bound param (structurally covering). This
  is the KISS/YAGNI boundary; nested heads are a residual (§8).
- **Higher-kinded `impl[F] Trait for F[Int]`** (abstracting the *constructor*). **Verdict:** honestly out of
  scope — Mycelium v0 has no higher-kinded type surface (RFC-0019 defers HKT/associated types to v2). Flag
  and refuse never-silently; do **not** pretend the parametric-head design reaches it.
- **Negative reasoning / conditional impls** (`impl[T] Trait for Foo[T] where T: Other`). The `where`/bound
  part is **DN-131's** scope; the *conditional* selection (pick this impl only when `T: Other` holds) is a
  coherence complication (two conditional impls may be mutually exclusive yet both parse). **Verdict:** v1
  admits an *unconditional* parametric impl only; conditional/bounded parametric impls are the DN-130×DN-131
  intersection and inherit DN-131's bound-discharge plus a residual note on conditional coherence (§8).
- **Recursion / infinite instance family.** `impl[T] Trait for Foo[T]` used at `Foo[Foo[Foo[…]]]` via a
  recursive type. Monomorphization is demand-driven per *reachable* concrete type; a genuinely-unbounded
  family would be caught by the existing mono depth budget (`MAX_CHECK_DEPTH`, M-657) as a never-silent
  refusal, not a hang. **Verdict:** bounded by the existing guard; no new risk.

**Stress-test verdict:** the recommendation survives for the *single-parameter, unconditional, structurally-
covering* subset with the reject-overlap rule intact. Every richer case (overlap-with-specialization,
blanket, nested, HKT, conditional) is a *never-silent refusal with a named residual*, not a silent
mis-accept — the honest boundary (G2/VR-5).

## §8 Residuals + out-of-scope (stated plainly)

- **Bounded parametric trait impls** `impl[T: Bound] Trait for Foo[T]` — the DN-130 head × DN-131 bound.
  Needs both this note's parametric head *and* DN-131's bound-discharge; recommend landing DN-130 unbounded
  first, then the intersection once both are ratified.
- **Overlapping / specialized instances** (a blanket + a specialized impl) — refused; needs a specificity
  order RFC-0019 §4.5 rejects for confluence. Out of scope, likely permanently.
- **Blanket `impl[T] Trait for T`** — refused (§7); a future scoped design could admit it with fundeps-style
  restrictions. Residual.
- **Nested / non-covering heads** `impl[T] Trait for Foo[Bar[T]]` — refused; structurally-covering heads
  only in v1.
- **Higher-kinded heads** `impl[F] Trait for F[…]` — out of scope (RFC-0019 v2, no HKT surface in v0).
- **Conditional coherence** (mutually-exclusive bounded parametric impls) — out of scope; v1 is
  unconditional parametric impls only.

## §9 Python carry-forward (source-language-general)

The transpiler is source-language-general (Rust today, Python next — the loose-typing DX lane). The Python
analogue of `impl<T> Trait for Foo<T>` is a **`Generic[T]` class implementing a `Protocol`**:

```python
T = TypeVar("T")
class Wrap(Generic[T]):
    def get(self) -> T: ...          # a method uniform over T
```

A `Protocol` conformance (`class Wrap(Generic[T])` structurally satisfying `Displayable`) maps to the same
parametric trait-instance family: the `Protocol` is the trait, `Wrap[_]` is the parametric head, and each
concrete `Wrap[int]` monomorphizes to a concrete instance. So DN-130's parametric-head design is the native
target for **both** Rust generic trait impls and Python generic-class Protocol conformance — the underlying
PROBLEM (a trait/protocol provided uniformly over a type-constructor family) is source-language-general,
which is why the note frames the *problem*, not Rust's `impl` keyword.

## §10 Definition of Done (what ratification + landing require)

**For the maintainer to move DN-130 Draft → Accepted:** confirm (a) the single-parameter covering-subset
scope, (b) coherence keyed on the type-constructor head reusing DN-122's `CoherenceView`, (c) reject-overlap
(no specificity) as the v1 rule, and (d) the residual boundary (§8) is the right KISS/YAGNI line.

**For a later Enacted (implementation) — the build DoD:**

- **Parse/AST** — lift the trait-branch refusal (`parse.rs:1234`) for the covering subset; `ImplDecl` gains
  `params`; the head is validated structurally-covering, else a never-silent refusal.
- **Coherence** — `(Trait, constructor-head)` uniqueness; overlap = head-unification, rejected; orphan via
  DN-122's home-qualified closure. Each an explicit `CheckError` (G2).
- **Mono** — per-reachable-instantiation `InstanceDecl` synthesis (α-substitution, M-673); the closure
  invariant (`no_reachable_var`) holds on the emitted specializations.
- **Witnesses** — an accept (`impl[T] Trait for Foo[T]` used at two type args → two specializations, mirror
  of `tests/check.rs:1493`, `generic_inherent_method_monomorphizes_across_two_type_args`); the rejects of §7
  (overlap, blanket, nested, HKT, bounded-parametric); a DN-26 Rust↔`.myc` differential at the layer each
  implements.
- **Transpiler** — the emit arm turns a Rust `impl<T> Trait for Foo<T>` into the parametric `.myc` form;
  `checked_fraction` (not `expressible_fraction`) is the number that moves (DN-124 phylum-mode basis).
- **Guarantee** — `Declared` until built; `Empirical` once the conformance + differential witnesses above
  are green; **no `Proven`** claim (coherence is `Declared`-with-argument — VR-5).

**Applied at the 2026-07-12 ratification close-out (append-only note):** `Doc-Index.md` DN-130 row added
at status **Accepted**; `CHANGELOG.md` carries the ratification entry; **M-1087** minted (the
generic-trait-instance-impl build — parse/AST lift, coherence, mono, witnesses, transpiler emit arm,
`depends_on: [M-1080]` per the real landing-order dependency stated above).

## §11 Changelog

- **2026-07-12** — DN-130 created (**Draft**). Designs the generic **trait-instance** impl
  (`impl[T] Trait for Foo[T]`) — the residual DN-103 §6 deferred — as a Native-Equivalent parametric head
  over the landed monomorphization + DN-122 coherence machinery, scoped to the single-parameter covering
  subset, ranked against the status-quo refusal / full-overlap-solver / runtime-dictionary alternatives
  (§6), and stress-tested against overlap/blanket/nested/HKT/conditional coherence (§7). Read against
  `dev@fa53dc46` (`Empirical` cites); the proposed mechanism is `Declared` (unbuilt). Authored the READ +
  this DN only — no edit to `issues.yaml`, `CHANGELOG.md`, or `Doc-Index.md` (integration-owned; FLAGGED
  up). Append-only; status advances only by maintainer ratification (house rule #3).
- **2026-07-12** — grounding/readiness fixes ahead of ratification (Draft stays Draft; append-only, no
  design change). (1) Corrected **four** mis-citations of `tests/mono.rs:268` (the "Verify-first
  reframing" row, the §1 table, and the §10 DoD "Witnesses" bullet each cited it as the impl-generics
  two-specialization witness — that line is inside `two_widths_emit_two_distinct_specializations`, which
  asserts `first_or$Binary8`/`first_or$Binary4`, not impl-generics; corrected to
  `tests/check.rs:1493`/`generic_inherent_method_monomorphizes_across_two_type_args`, the real ENB-3
  two-specialization witness). The §5 closure-invariant cite for `cmp$Cmp$Binary8` was correct in intent
  but pointed 46 lines off; corrected to `tests/mono.rs:222`
  (`a_trait_method_call_resolves_statically_with_an_explain_record`). All four corrected citations were
  re-verified against `dev@fa53dc46` before writing. (2) Tightened §4 point 3 and the "Depends on" line,
  which read as if DN-130 reuses a **landed** home-qualified `CoherenceView`. It does not: the
  currently-landed `CoherenceView` (`checkty.rs:2252`) is phylum-wide/pub-blind
  (`traits`/`types: BTreeSet<String>`), not home-qualified; the home-qualified extension is DN-122's own
  **unbuilt** MVP (**M-1080**, `status: todo`, `depends_on: [M-1060, M-1079]`). Restated honestly: DN-130's
  coherence implementation has a real **landing-order dependency** on M-1080 (or must build the
  home-qualification itself); it does reuse the *landed* `type_head`/`qualify_type_name` primitives
  (M-1036), not an already-built home-qualified coherence view (VR-5 — Accepted-but-unbuilt is not a
  landed reuse). No mechanism, scope, or recommendation changed.
- **2026-07-12** — Ratified **Accepted** (delegated ratification, gap-close-2 batch). Status moved Draft
  → Accepted under explicit maintainer delegation (mirrors DN-115/117/118/122/123/124/125/126/127/128/
  129). The parametric-head mechanism, the constructor-keyed coherence extension, and the v1
  single-parameter covering-subset scope are accepted as designed; the M-1080 landing-order dependency is
  carried forward, not resolved by ratification. Builds nothing yet — **M-1087** minted for the
  implementation (`depends_on: [M-1080]`). Append-only; VR-5/G2.
