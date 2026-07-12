# Design Note DN-131 — Bounds on Non-Function Type-Parameter Sites (`impl[T: Bound]`, `type`/`trait` decls) — the bounded-generics residual

| Field | Value |
|---|---|
| **Note** | DN-131 |
| **Status** | **Draft** (2026-07-12). A design-reasoner note working the **bounded-generics** cluster gap (M-876 item 3) forward to a **ranked recommendation for maintainer ratification** (house rule #3 — enacts nothing, ratifies nothing, moves no other doc's status). Tags are `Empirical` where read against the tree at `dev@fa53dc46` with a `file:line` cite, `Declared` for any design not yet built or ratified (VR-5). |
| **Verify-first reframing (mitigation #14 — the tracker/register lag the code)** | The naive framing — "bounded generics `fn f<T: Bound>` / `impl<T: Bound>` needs design (M-876, P3)" — is **half already landed**. **Trait-bounded generics on *functions* are Closed** (DN-99 register row #5 `generic-bound: closed`): `fn f[T: Cmp](…)` parses (`parse.rs:1164` `parse_type_params_bounded` → `parse_type_param` `:1174` → `parse_bound` `:1190`, multi-bound `+`), the bounds are checked (`checkty.rs:4850` `check_bounds`), and the call **monomorphizes dictionary-free** — witnessed by `tests/mono.rs:240`: `fn use_cmp[T: Cmp](a: T, b: T) => Binary{2}` lowers to `use_cmp$Binary8` + `cmp$Cmp$Binary8` with `no_reachable_var`. M-876's own body ("the bound slot on impl/fn generics is missing") is stale on the *fn* half. **The genuine residual is bounds on the _non-function_ sites**: the impl slot (`impl[T: Bound] Foo[T]`) and the `type`/`trait` decl heads — all three are today a **never-silent refusal** (`parse.rs:1145`–`1152` for `type`/`trait`/impl-slot via `parse_type_params_opt`). This note designs *only* that residual. |
| **Decides (proposes, for ratification)** | (1) the **native mechanism** — admit a `: bound` on the **inherent-impl** type-parameter slot (`impl[T: Bound] Foo[T] { … }`), discharged by **redistribution to the lifted methods**: at DN-103's Phase-0 desugar the impl's bounds ride onto each lifted method's own `fn` type-parameter, so the *already-landed* fn-bound path (`check_bounds` + dictionary-free mono, row #5) discharges them with **zero new discharge code**; (2) the **`type`/`trait` decl-head decision** — recommend **declining** decl-head bounds in v1 per RFC-0019 §4.2 ("bounds on the type itself are on the methods, not the decl"), keeping the existing never-silent refusal, because decl-head bounds add checker surface for no `checked_fraction` win the method-site bound does not already give (YAGNI); (3) the **surface** — inline `T: A + B` (multi-bound via `+`, the landed `parse_bound` grammar), **no `where`-clause** in v1 (L2 sugar, deferred exactly as RFC-0019 §4.1 already defers it); (4) the **build split** — a parse + Phase-0-desugar change reusing the landed `parse_type_params_bounded` and `check_bounds`, **zero L0/kernel/runtime/mono growth**. It does **not** edit `issues.yaml`, `CHANGELOG.md`, or `Doc-Index.md` (integration-owned — FLAGGED up). |
| **Feeds** | M-876 item (3) trait-bounded generics `<T: Bound>` (this note is the design gate for the residual half); DN-103 §7 reject-(b) (the impl-slot bound refusal this note lifts); DN-99 register row #5 (generic-bound — the fn half already closed) + row #63 (impl-block); RFC-0019 §3.3/§4.1/§4.2/§4.3 (the bound model + dictionary translation this reuses). |
| **Depends on** | RFC-0019 (Enacted — bounded type parameters `type_param ::= Ident (':' bound)?`, dictionary-free discharge via monomorphization/M-673, coherence unchanged, KC-3 node budget unchanged); **DN-103** (Accepted — the impl-level slot + Phase-0 desugar-prepend, the exact vehicle the bound rides); the landed fn-bound machinery (`parse_type_params_bounded` `parse.rs:1164`, `check_bounds` `checkty.rs:4850`, M-657/M-659/M-673). |
| **Grounds on** | KC-3/DRY (reuse `parse_bound` + `check_bounds` + the M-673 mono path — no new discharge logic); G2 (never-silent — an unsatisfiable/duplicate/decl-head bound each prints the fix); VR-5 (`Declared` until built + differential-witnessed; no tag upgraded past its basis); KISS/YAGNI (impl-slot bounds where there is a real ergonomic + `checked_fraction` win; decline decl-head bounds + `where`-clauses where there is not). |
| **Date** | July 12, 2026 |
| **Author** | design-reasoner (Opus). Owns only this note. |
| **Task** | bounded-generics cluster — bounds on non-fn sites (companion: DN-130, generic trait-instance impls). |

> **Grounding + honesty (house rule #4 / VR-5 / G2).** A design recommendation, not a decision (house
> rule #3). `Empirical` claims are read against `dev@fa53dc46` and cite `file:line`; proposed mechanisms
> are `Declared`. **No sycophancy:** §1 states plainly that the fn half of "bounded generics" is already
> Closed, so this note is narrow; §6 recommends *declining* part of what a naive scope would build
> (decl-head bounds, `where`-clauses) because the evidence says they add checker surface for no win — the
> honest answer even though it is a smaller feature than "add bounded generics."

---

## §1 Purpose — and the honest scope (verify-first)

Read against `dev@fa53dc46`, "bounded generics" is not one gap but four sites, three of which are settled:

| Site | State today | Evidence (`file:line` @ fa53dc46) |
|---|---|---|
| `fn f[T: Bound](…)` (function) | **LANDED / Closed** (row #5) | `parse.rs:1164/1174/1190`; `checkty.rs:4850` `check_bounds`; mono witness `tests/mono.rs:240` (`use_cmp$Binary8`, `cmp$Cmp$Binary8`) |
| Multi-bound `[T: A + B]` | LANDED (parse+check) | `parse.rs:1190`–`1195` `parse_bound` (`+`-separated `TraitRef`s) |
| Self-bound sugar `T: Cmp ≡ T: Cmp[T]` | LANDED | RFC-0019 §4.1; M-659 |
| **`impl[T: Bound] Foo[T]` (impl slot)** | **REFUSED / deferred** | DN-103 §7 reject-(b); `parse.rs:1220` uses `parse_type_params_opt`, which refuses `: bound` (`:1145`–`1152`) |
| **`type Foo[T: Bound]` / `trait X[T: Bound]` (decl head)** | **REFUSED / deferred** | `parse.rs:1145`–`1152` never-silent refusal, "bounds live only on function type-parameters" |

So the residual is bounds on the **non-function** sites. The one with a real ergonomic + transpilation
payoff is the **impl slot**: a Rust `impl<T: Clone> Foo<T> { fn dup(&self)… }` where several methods share
`T: Clone` today forces the `.myc` porter to repeat `[T: Clone]` on *every* method, or to widen the impl to
DN-103's unbounded `impl[T]` and lose the constraint. The decl-head sites are a different question (§6).

## §2 The PROBLEM, and Mycelium's native-solution class (DN-110/DN-111)

**Translate the problem, not Rust's mechanism (DN-111 §9 step 2).** A bound `T: Bound` is not "angle-bracket
syntax"; the underlying problem is: *constrain a type parameter to the operations of a trait, so the
generic body may call those operations, and so instance selection is decided at each concrete
instantiation.* Mycelium already solves this at the fn site by **dictionary-free monomorphization**: at a
call the concrete `T = C` is known, the `(Bound, C)` instance is resolved statically, and the method call
inlines to `m$Bound$C` — no runtime dictionary, no vtable (DN-55 / M-673; RFC-0019 §4.3 preview, §4.4
mono-recommendation).

- **{exact?}** — yes; each instantiation behaves as a hand-written monomorphic call.
- **{native?}** — yes; the impl-slot bound is *the same bound at a different lexical site*, and DN-103
  already carries impl params to the methods. So the bound just needs to ride the existing carry.

**Classification (DN-111 taxonomy): Native Equivalent.** No approximation, no bridge — the impl-slot bound
redistributes to the landed fn-bound discharge path.

## §3 The surface + grammar

The bound grammar **already exists** — RFC-0019 §4.1 / `parse.rs:1174` `parse_type_param` (`Ident (':' bound)?`)
and `:1190` `parse_bound` (`Ident type_args? ('+' Ident type_args?)*`). What v1 does is **let the impl slot
use the bounded parser instead of the unbounded one**:

```
impl_item      ::= 'impl' impl_type_params? base_type impl_tail       // inherent tail only (DN-103)
impl_type_params ::= '[' type_param (',' type_param)* ']'             // v1: bounded — was parse_type_params_opt
type_param     ::= Ident (':' bound)?                                  // RFC-0019 §4.1 (landed for fns)
bound          ::= Ident type_args? ('+' Ident type_args?)*           // landed: parse_bound (parse.rs:1190)
```

- `impl[T: Cmp] Foo[T] { fn max(a: Foo[T], b: Foo[T]) => Foo[T] = … }` — the impl binds `T: Cmp`; every
  method may call `Cmp`'s operations on a `T`.
- Concretely: change DN-103's `parse_impl_item` slot call from `parse_type_params_opt` (unbounded,
  `parse.rs:1220`) to `parse_type_params_bounded` (already used for fns, `parse.rs:1164`) — *for the
  inherent tail only*. The trait-instance tail stays refused (that is DN-130's scope + coherence).
- **Backward compatible** — an unbounded `impl[T] Foo[T]` yields `bounds: []` (the RFC-0019 §4.1 identity,
  `parse.rs:1179`), so every DN-103 program is unchanged.
- **No `where`-clause** — inline bounds only, exactly as RFC-0019 §4.1 defers `where` to L2 sugar.

## §4 Bound discharge — redistribution to the lifted methods (reuse `check_bounds` + M-673)

DN-103 already lowers an inherent-impl block by **prepending** the impl's `params` to each method at the
Phase-0 desugar (`checkty.rs:2053`), turning each method into an ordinary generic free function. The **only**
change for bounds: the prepended `TypeParam` carries its `bounds` (instead of the forced `bounds: []` DN-103
§4 uses). Consequences, all free:

1. The lifted method is a **bounded generic free function** — the landed `check_bounds` (`checkty.rs:4850`)
   validates each bound against the trait registry, and the landed dictionary-free monomorphizer discharges
   it (M-673) exactly as `tests/mono.rs:240` witnesses for a hand-written `fn use_cmp[T: Cmp]`. **Zero new
   discharge code** (DRY/KC-3).
2. A **duplicate bound source** — the impl binds `T: Cmp` and a method re-binds `T: Ord` — resolves by the
   existing duplicate-type-parameter refusal on the lifted sig (DN-103 §4 point 3), a never-silent error
   (G2). (Merging an impl `T: A` with a method `T: B` into `T: A + B` is a possible future ergonomic; v1
   refuses the re-bind rather than silently union — the conservative, never-silent choice.)
3. An **unsatisfiable bound** at a call site (`T = C` with no `impl Cmp for C`) is the existing never-silent
   no-instance error from `check_bounds`/resolution — unchanged.

**Net kernel/runtime/mono delta: zero.** The change is: the impl slot parses bounds, and DN-103's
desugar-prepend stops erasing them. Everything downstream already exists.

## §5 The `type`/`trait` decl-head sites — RFC-0019 §4.2 says the bound belongs on the method

RFC-0019 §3.3 line 147 and §4.2 are explicit: *"A generic data type (bounds on the type itself are on the
methods, not the decl)."* The RFC's design intent is that a `type Foo[T]` declaration is an **unbounded**
abstraction, and any constraint is expressed where the operation is actually used — the method (or the
impl). A `type Foo[T: Cmp]` decl-head bound would mean "you may not even *name* `Foo[C]` unless `C: Cmp`,"
which:

- adds a **use-site well-formedness check** the checker does not have today (every `Foo[…]` mention must
  discharge the bound), for a constraint the method-site bound already enforces where it matters;
- has **no `checked_fraction` win** — the transpiler's Rust `struct Foo<T: Cmp>` maps to an unbounded `.myc`
  `type Foo[T]` plus bounded methods with no loss of checkability (the data layout does not depend on the
  bound; DN-123 records are positional/structural, ADR-003, names/bounds off the identity hash);
- is exactly what RFC-0019 §4.2 designed *away* from.

**So v1 declines decl-head bounds and keeps the existing never-silent refusal** (`parse.rs:1145`), pointing
the porter at the method/impl-slot bound. This is a YAGNI call grounded in the RFC's own model — see §7 for
the adversarial check that this does not drop real programs.

## §6 Ranked alternatives + recommendation (the objective function)

**Objective (weighted):** *ergonomic/faithfulness win* for the porter (3) · *reuse of landed discharge* DRY
(3) · *checker/kernel cost* KC-3 (2) · *`checked_fraction` payoff* (2) · *v1 size* KISS/YAGNI (1).

| # | Alternative | Ergonomic (3) | Reuse (3) | KC-3 cost (2) | `checked_fraction` (2) | v1 size (1) | Score |
|---|---|---|---|---|---|---|---|
| **1** | **Impl-slot bounds via bounded parser + DN-103 desugar carries bounds; decline decl-head + `where`** (this note) | ✔ shares bound across methods (3) | ✔ `check_bounds` + M-673 unchanged (3) | zero new logic (2) | ✔ impl-bound shapes clean (2) | small (1) | **11** |
| 2 | Impl-slot bounds **and** `type`/`trait` decl-head bounds (use-site WF check) | ✔ + (3) | new decl-head WF check (1) | new checker surface (0) | no extra win (0) | larger (0) | 4 |
| 3 | Full `where`-clause on fn + impl + decl | ✔ + (3) | new parse + scope logic (1) | grammar + checker growth (0) | none (0) | large (0) | 4 |
| 4 | Do nothing — porter repeats `[T: Bound]` on every method, or widens to unbounded `impl[T]` | ✗ verbose or loses constraint (0) | full (3) | zero (2) | ✗ constraint lost on widen (0) | zero (1) | 6 |

**Recommendation: Alternative 1.** It is the Native-Equivalent close — the impl-slot bound is *the landed fn
bound at a different site*, redistributed by *the DN-103 desugar that already runs*, discharged by *the
`check_bounds` + M-673 path that already runs*. It buys the real ergonomic + `checked_fraction` win (shared
constraint across an impl's methods) at zero new discharge/kernel cost. Alternative 2 adds a use-site
well-formedness check RFC-0019 §4.2 deliberately avoided, for no measured payoff. Alternative 3 (`where`) is
L2 sugar the RFC already deferred. Alternative 4 (status quo) is the verbose/lossy state the porter is in
today. **The recommendation deliberately declines the decl-head + `where` scope** — the honest,
evidence-following answer even though it is a smaller feature than a naive "add bounded generics
everywhere."

## §7 Adversarial stress-test (VR-5 — argue against the recommendation, esp. against declining decl-head bounds)

- **A Rust program `struct Foo<T: Cmp>` whose methods never mention the bound.** Does declining decl-head
  bounds (§5) drop it? **Verdict:** no — the `.myc` target is `type Foo[T]` (unbounded) + bounded methods.
  The data *layout* never depends on `T: Cmp`, so the decl-head bound carried no checkable content; nothing
  is lost. If a method uses `Cmp`, that method carries `[T: Cmp]`. **The decline is safe.** *Honest edge:*
  a Rust program that constructs `Foo<NonCmp>` and relies on the compiler *rejecting* it purely on the
  decl-head bound (never calling a `Cmp` method) would not be rejected in `.myc` — but such a program is a
  *type-checking-only* artifact with no runtime meaning; flagged as a (vanishingly rare) faithfulness
  residual (§8), not a silent accept of wrong behavior.
- **Impl-slot bound vs method re-bind.** `impl[T: Cmp] Foo[T] { fn g[T: Ord](…) }`. **Verdict:** the method
  re-binds `T` → the existing never-silent duplicate-type-parameter refusal (§4.2). Correct — no silent
  union, no silent shadow.
- **Impl-slot bound the methods do not use.** `impl[T: Cmp] Foo[T] { fn id(x: Foo[T]) => Foo[T] = x }` — the
  bound is dead. **Verdict:** accepted (the bound is validated against the registry by `check_bounds`, and
  monomorphization simply never needs the `Cmp` dictionary — dictionary-free, so a dead bound costs
  nothing). Optionally an advisory unused-bound diagnostic (never an error — G2/KISS). Non-blocking.
- **Multi-bound + supertrait on the impl slot.** `impl[T: Ord + Show] Foo[T]`. **Verdict:** `parse_bound`
  already handles `+`; supertraits already flow through the fn-bound path (RFC-0019 §4.3 super-traits ride
  the dictionary as a field). Reused unchanged.
- **Repr-bound interaction (S1).** `impl[R: Repr] Foo[R]` where a method converts between widths of `R`.
  **Verdict:** governed by RFC-0019 §4.6 exactly as at the fn site — a representation change in a generic
  body must be an explicit `swap(…)`, never elaborator-inserted; a mono step that discovers a needed swap
  refuses with an explicit error (S1, never-silent). No new interaction; the impl slot inherits the fn
  site's S1 discipline.
- **Bounded parametric *trait-instance* impl** `impl[T: Bound] Trait for Foo[T]`. **Verdict:** out of DN-131
  scope — the *trait-instance parametric head* is DN-130; this note only lifts the bound refusal on the
  **inherent** slot. The intersection needs both notes (DN-130 §8). Flagged.

**Stress-test verdict:** impl-slot bounds are a clean, zero-new-logic reuse of the landed fn-bound path. The
decision to *decline* decl-head bounds and `where`-clauses survives the adversarial check — the only thing
lost is a type-check-only rejection with no runtime meaning, flagged as a rare residual rather than silently
mis-handled.

## §8 Residuals + out-of-scope (stated plainly)

- **`type`/`trait` decl-head bounds** — declined in v1 (§5); the never-silent refusal stays, pointing at the
  method/impl-slot bound. Residual: a type-check-only decl-head rejection is not reproduced (rare, no
  runtime meaning).
- **`where`-clauses** — deferred (L2 sugar, RFC-0019 §4.1). Inline `T: A + B` only in v1.
- **Impl `T: A` merged with method `T: B` into `T: A + B`** — refused (duplicate re-bind) rather than
  unioned in v1; a future ergonomic upgrade.
- **Bounded parametric trait-instance impl** `impl[T: Bound] Trait for Foo[T]` — the DN-131 bound × DN-130
  head; land each note's half first.
- **Width/const-param bounds** `{N: …}` — permanently declined (DN-42 §7: width params carry no trait
  bounds; already a never-silent refusal, `parse.rs:944`).

## §9 Python carry-forward (source-language-general)

The Python analogue of `impl<T: Bound>` is a **bounded `TypeVar`**:

```python
T = TypeVar("T", bound=Comparable)      # T: Comparable
def top(xs: list[T]) -> T: ...          # body may call Comparable ops on T
```

A `TypeVar(bound=…)` or a `TypeVar` constrained by a `Protocol` maps onto exactly the same bound: the
`Protocol` is the trait, `bound=` is the `: Bound`, and each concrete instantiation monomorphizes to a
dictionary-free specialization. A Python `class Box(Generic[T])` whose methods rely on `T: Comparable` maps
to DN-131's impl-slot bound (the constraint shared across the class's methods). So DN-131's redistribution
design is the native target for **both** Rust `impl<T: Bound>` and Python bounded-`TypeVar` generic classes
— the PROBLEM (constrain a type parameter to a trait/protocol's operations) is source-language-general.

## §10 Definition of Done (what ratification + landing require)

**For the maintainer to move DN-131 Draft → Accepted:** confirm (a) impl-slot bounds via the bounded parser
+ DN-103 desugar carrying bounds, (b) the **decline** of `type`/`trait` decl-head bounds (RFC-0019 §4.2
grounds + §7 adversarial check), (c) inline `T: A + B` only, no `where` in v1, and (d) duplicate re-bind
refused (not unioned) in v1.

**For a later Enacted (implementation) — the build DoD:**

- **Parse** — DN-103's impl-slot call switches `parse_type_params_opt` → `parse_type_params_bounded`
  (inherent tail only); the trait-instance tail bound-refusal is unchanged (DN-130's scope).
- **Desugar** — DN-103's Phase-0 prepend (`checkty.rs:2053`) carries the impl param's `bounds` instead of
  forcing `[]`; the lifted method is a bounded generic fn.
- **Check + mono** — `check_bounds` validates the (now non-empty) prepended bounds; monomorphization
  discharges them dictionary-free (M-673). No new discharge path.
- **Witnesses** — an accept (`impl[T: Cmp] Foo[T]` whose method calls `cmp` → the two-specialization mono
  witness, mirror of `tests/mono.rs:240`); the rejects (decl-head bound still refused; method re-bind
  duplicate; unsatisfiable bound at a call); a DN-26 Rust↔`.myc` differential.
- **Transpiler** — a Rust `impl<T: Bound>` emits the bounded `.myc` impl slot; a `struct Foo<T: Bound>`
  emits unbounded `type Foo[T]` + bounded methods (§5). `checked_fraction` is the number that moves.
- **Guarantee** — `Declared` until built; `Empirical` once the conformance + differential witnesses are
  green; **no `Proven`** claim (VR-5).

## §11 Changelog

- **2026-07-12** — DN-131 created (**Draft**). Designs bounds on the **non-function** type-parameter sites —
  the residual half of M-876's bounded-generics item (the *fn* half is already Closed, DN-99 row #5) — as a
  Native-Equivalent redistribution: the impl-slot bound rides DN-103's Phase-0 desugar onto each lifted
  method and is discharged by the landed `check_bounds` + dictionary-free monomorphizer (zero new discharge
  code), while `type`/`trait` decl-head bounds and `where`-clauses are **declined** per RFC-0019 §4.2 (§5),
  ranked (§6) and adversarially checked (§7). Read against `dev@fa53dc46` (`Empirical` cites); the proposed
  mechanism is `Declared` (unbuilt). Authored the READ + this DN only — no edit to `issues.yaml`,
  `CHANGELOG.md`, or `Doc-Index.md` (integration-owned; FLAGGED up). Append-only; status advances only by
  maintainer ratification (house rule #3).
