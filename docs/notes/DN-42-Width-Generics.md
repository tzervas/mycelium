# Design Note DN-42 — Width-Generics (const-generic-over-representation-width functions)

| Field | Value |
|---|---|
| **Note** | DN-42 |
| **Status** | **Accepted** (2026-06-26; **ratified by the maintainer** — Option A [width-as-const-generic-at-mono]; **v1 scope = width-generic free functions**, deferring the instance/coherence corner Q5; impl owned by E11-1/`s10`) — a design proposal for **M-753** (E11-1/`s10`), the surface-language type-system feature that makes a function **generic over a representation width** `N` (`fn f<N>(x: Binary{N}) -> Binary{N}`). **Enacts no code** and moves no other decision's status (house rule #3). The recommendation, options, and DoD below are the design direction submitted **for maintainer ratification**; impl is gated on that ratification (RFC-0032 §5 D5). |
| **Date** | June 26, 2026 |
| **Decides** | *Nothing normatively* — a proposal for ratification. Submits (1) the **problem** (§1 — width `{N}` is a concrete `u32` literal today, so `eq`/`lt`/`add_bin` are width-fixed and a width-polymorphic `fn f<N>` cannot be written, forcing monomorphic-`Binary{8}` stdlib); (2) **≥3 design options** with trade-offs (§3); (3) a **recommendation** — width as a const-generic parameter bound at monomorphization, structurally analogous to the existing type-parameter machinery (§4) — with its never-silent / VR-5 implications; (4) the **interactions** (§5) with the monomorphic collections lookup (M-716), generic math (M-718), the prospective width-cast prim, and RFC-0032 D5; (5) a **Definition of Done** for the eventual M-753 impl + a **migration/sequencing** note (§6); and (6) the **open questions** for the maintainer (§7). |
| **Feeds** | the **L1 surface type system** (`crates/mycelium-l1/src/checkty.rs` — the `Ty` enum, `prim_sig`/`prim_kernel_name`, `unify`/`subst_ty`/`param_subst`, `resolve_ty`) and the **monomorphizer** (`crates/mycelium-l1/src/mono.rs` — type-parameter pinning, the never-silent `Residual` on an undetermined parameter). Unblocks **E13-1 M-718** (general, non-fixed-width `std.math`/`std.cmp`) and the **generic-key collection lookup** flagged-deferred in **M-716** (`lib/std/collections.myc`, `map_get`/`set_contains` are monomorphic at `Binary{8}` precisely because `eq` is width-typed). Owned by **E11-1/`s10`** per **RFC-0032 §5 D5**. Sibling to the **prospective width-cast / width-reinterpretation prim note** (a *forthcoming* note, not yet written — referenced here as a design dependency, **not** as existing authority). |
| **Task** | M-753 — width-generic functions (E11-1/`s10`); design note (this file). |

> **Posture (transparency rule / VR-5 / G2 / house rule #4).** This note **proposes** a design; it
> **enacts nothing** — no code, no property test, no status move of any other decision *from this note*.
> The **problem statement (§1) and the cited code behavior** are `Empirical`/`Proven`-at-source: each is
> tied to a file the survey read (`checkty.rs`'s `Ty::Binary(u32)` + the `prim_sig` width-equality arms;
> `mono.rs`'s `param_subst`/`unify` + `Residual`; `collections.myc`'s `Binary{8}` lookup; the AST's
> `BaseType::Binary(u32)`). The **design options and the recommendation (§3–§4)** are `Declared` — asserted
> design directions for ratification, **not** proven properties; where a claim is genuinely uncertain it is
> flagged as such (§7), not smoothed into confidence. The recommendation is given **on independently
> reasoned merit**, and the §7 open questions surface the places where the design is *not* settled even
> where that cuts against shipping the simplest option (VR-5 applied to assent). No guarantee is upgraded:
> a width-generic op inherits exactly the honest tag of its monomorphic specialization (§4).

---

## §1 Problem — width is a concrete literal, so the surface cannot be width-polymorphic

In the L1 type system a representation **width is a concrete `u32`**, baked into the type at parse time
and never abstracted:

- The AST carries it as a literal: `BaseType::Binary(u32)` / `BaseType::Ternary(u32)`
  (`crates/mycelium-l1/src/ast.rs:327,329`), resolved by `resolve_ty` into
  `Ty::Binary(u32)` / `Ty::Ternary(u32)` (`checkty.rs:55–92`, `464–466`).
- The builtin prim signature table `Π` matches widths as **concrete literals with an equality
  constraint**, never as a variable. For example (`checkty.rs::prim_sig`, lines 3779–3794):

  ```rust
  ("and" | "or" | "add_bin" | "sub_bin", [Ty::Binary(a), Ty::Binary(b)]) if a == b => Some(Ty::Binary(*a)),
  ("add" | "sub" | "mul", [Ty::Ternary(a), Ty::Ternary(b)]) if a == b => Some(Ty::Ternary(*a)),
  ```

  The width is *width-preserving and width-polymorphic at the prim level* (`a == b ⇒ Binary{a}`), but the
  type system can only express the result at a **concrete** `a`/`b` it already holds — there is no way to
  bind that `a` as a parameter `N` a caller chooses.
- The comparison prims `eq`/`lt` (RFC-0032 D1; `prim_kernel_name`, `checkty.rs:3809–3810`) likewise
  apply at a fixed operand width.

The existing generic machinery is **over types, not widths**. `Ty::Var(String)` is a skolem **type**
variable (`checkty.rs:78–84`); `unify`/`subst_ty`/`param_subst` (`checkty.rs:260–352`) instantiate
**type** parameters; mono pins them at call sites and refuses an undetermined one with an explicit
`Residual` (`mono.rs:1045–1060`). A function can be generic over an *element type* `A`
(`fn len<A>(xs: Vec<A>) -> Binary{8}`), but it **cannot** be generic over a *width* `N`: there is no
`Ty::Var`-equivalent in the **width** position of `Ty::Binary`/`Ty::Ternary` (which is a plain `u32`).

### The cost (grounded, not assumed)

Because `eq`/`lt`/`add_bin` are width-typed, the self-hosted stdlib must **bake in a concrete width**:

- **Collections lookup is monomorphic at `Binary{8}`.** `lib/std/collections.myc` `map_get` and
  `set_contains` are pinned at `Binary{8}` keys *precisely because* `eq` is width-typed
  (`collections.myc:15–16, 92–93, 106–110, 129–130`):

  ```myc
  fn map_get(m: Map<Binary{8}, Binary{8}>, k: Binary{8}) -> Option<Binary{8}> = …
  ```

  M-716 shipped this and **explicitly flagged** generic-key lookup as deferred to M-753 (a flagged
  enhancement, *not* a DoD gap; `issues.yaml` M-716 body — "Map/Set lookup is monomorphic at `Binary{8}`
  because `eq` is width-typed and width-generics (M-753) are not yet landed").
- **General math/cmp is blocked.** M-718 (`std.math`/`std.cmp` non-fixed-width surface) is `status:
  blocked` with `depends_on: [..., M-753]`; its body records that a fixed-width-only port "would bake
  `Ternary{4}` into the stdlib (a weak surface — not taken, VR-5)" (`issues.yaml:4566–4587`). `lib/std/cmp.myc`
  already carries the honesty boundary in its header: equality/ordering over the *width* types
  `Binary{N}`/`Ternary{N}` is **not** self-hosted there because the surface is width-fixed (`cmp.myc:14–19`).

So the absence of width-generics is the single reason two self-hosting deliverables are either fixed-width
or blocked. M-753 is the unblock. (Grounding: `issues.yaml` M-753 body, `depends_on: [M-708]`,
`status: needs-design`; RFC-0032 §5 D5.)

---

## §2 What "width-generic" means here (scope)

A width-generic function abstracts a **representation width** `N` (a non-negative integer slot in
`Binary{N}` / `Ternary{N}` / and, by extension, `Dense{D, S}` / `Seq{T, N}`):

```myc
fn id_bits<N>(x: Binary{N}) -> Binary{N} = x
fn eq_w<N>(a: Binary{N}, b: Binary{N}) -> Binary{1} = eq(a, b)   // eq is width-preserving-to-Binary{1}
```

In scope for this proposal:

- **Width parameters on functions**, bound at the call site (analogous to today's type parameters).
- The width parameter appears in `Binary{N}` / `Ternary{N}` (the two width-bearing scalar paradigms with
  surfaced arithmetic/comparison prims today). `Dense{D, S}` and `Seq{T, N}` width-genericity is a
  **possible extension**, flagged (§7 Q4) — not the first increment.

**Out of scope (explicitly deferred, never silently):**

- **Width arithmetic in types** (`fn double<N>(x: Binary{N}) -> Binary{N+1}` — a *type-level* `N+1`).
  This needs a const-expression layer in the type grammar; it is a separate, larger feature (§7 Q2).
  The first increment is **width-preserving** generics only (which is exactly what `prim_sig` already
  supports at the prim level — `a == b ⇒ result width a`).
- **Width bounds / where-clauses** (`fn f<N where N <= 64>`) — a refinement (§7 Q3).
- Any change to the **kernel prims or `Repr`** — width-generics is a *surface* feature (RFC-0032 §5 D5);
  it lowers to the existing concrete-width prims by monomorphization (§4). No KC-3 trusted-base growth.

---

## §3 Design options

Three options, each assessed for: **type-checker** changes (`checkty.rs`), **monomorphizer** changes
(`mono.rs`), **never-silent** implications (G2/VR-5), and **what breaks**.

### Option A — width as a const-generic parameter, bound at monomorphization *(recommended, §4)*

Treat a width parameter `N` exactly as today's type parameter `A` is treated, but in the **width slot**.
The checker carries `N` **symbolically**; the concrete-width prim check **defers to mono-time**, where `N`
is pinned at the call site and the body is specialized to a closed concrete width.

- **Type checker (`checkty.rs`):**
  - Extend the **width slot** of `Ty::Binary` / `Ty::Ternary` to be *either* a concrete `u32` *or* a
    **width variable**. Concretely, introduce a small `Width` sum — `Width::Lit(u32) | Width::Var(String)`
    — and change `Ty::Binary(u32)` → `Ty::Binary(Width)` (same for `Ternary`; `Seq`/`Dense` later). A
    monomorphic program only ever constructs `Width::Lit`, so its behavior is **byte-identical** (the
    `Display`, `type_head`, `prim_sig`, and `subst_ty` arms all pattern-match `Width::Lit` for the
    existing cases).
  - `resolve_ty` (`checkty.rs:457`) resolves a `Binary{N}` whose `N` names an **in-scope width
    parameter** to `Width::Var(N)` (the width analogue of the `Ty::Var` arm at `checkty.rs:483`), exactly
    as a bare type name in `tyvars` resolves to `Ty::Var`.
  - `unify` / `subst_ty` / `param_subst` (`checkty.rs:260–352`) gain a **width** arm: a `Width::Var`
    unifies against a concrete `Width::Lit` by binding it (at most once — a second conflicting binding is
    the same explicit "ambiguous instantiation" error already used for `Ty::Var`, `checkty.rs:313–325`),
    and `subst_ty` substitutes a bound width. No `Swap` is ever inserted (S1) — a `Binary{8}` vs
    `Binary{16}` disagreement stays an explicit mismatch, never a coercion (the existing invariant,
    `checkty.rs:299–305`).
  - The **prim check** (`check_app`'s prim path, `checkty.rs:2570–2617`): when an operand's width is a
    `Width::Var`, `prim_sig` cannot decide a concrete result, so the checker **carries the symbolic
    width-preservation** (`add_bin: Binary{N} × Binary{N} → Binary{N}`) and defers the *concrete* width
    decision to mono-time. The width-equality side-condition (`a == b`) becomes a **width-unification**
    obligation (`N ~ N` trivially; `N ~ 8` binds, `8 ~ 16` refuses).
  - Distinguish width parameters from type parameters in the surface. Two sub-options (§7 Q1): (i) **same
    `<…>` list, disambiguated by use** (a name used in a width slot is a width param) — minimal grammar
    change but context-sensitive; (ii) a **marked form** (`fn f<const N: Width>` or `fn f<{N}>`) —
    explicit, unambiguous, more grammar. `TypeParam` (`ast.rs:222`) would carry a `kind: Type | Width`
    either way.
- **Monomorphizer (`mono.rs`):** width parameters join the existing per-call specialization. `emit_fn`
  (`mono.rs:373`) already pins type args via `param_subst` + `unify`; width args pin the **same way** into
  the same substitution (extended to carry width bindings). The mangled name gains the concrete widths
  (`eq_w$N8` alongside `first_or$Binary8`) — honest identity fragmentation, exactly as for types
  (`mono.rs:19–24`). An **undetermined width** at a call site is the existing never-silent `Residual`
  (`mono.rs:1045–1060`) — **no silent default width** (this is the load-bearing VR-5 point, §4).
- **Never-silent:** the existing `UnresolvedWidth` refusal for an un-anchored bare decimal
  (`checkty.rs:2596–2605`) is the *precedent*: a width with no pin is already a never-silent error, not a
  default. Width-generics generalizes that — an unbound `N` at mono-time is a `Residual`, never `Binary{8}`
  by fiat (G2).
- **What breaks:** the `Ty::Binary(u32)` → `Ty::Binary(Width)` change touches **every match arm on
  `Ty::Binary`/`Ty::Ternary`** in `checkty.rs`, `mono.rs`, `elab.rs`, and the AST↔Ty bridges — a wide but
  *mechanical* change (the `Width::Lit(n)` arm is the old behavior). This is the main cost. It is contained
  to the L1 frontend (no kernel change). Risk: missing an arm would be a **compile error**, not a silent
  bug (Rust's exhaustiveness — a never-silent property the language gives us for free here).

### Option B — width-polymorphic prims with a runtime width tag

Make the prims accept a value that carries its width **at runtime** (a `(width, bits)` pair), so a single
`eq` works at any width without monomorphization.

- **Type checker:** smaller — `Binary{N}` could stay one type with the width erased from the static type.
- **Monomorphizer:** little to no change (nothing to specialize — one body serves all widths).
- **Never-silent:** **worse.** A runtime width tag invites silent width-mismatch handling (compare a
  `Binary{8}` against a `Binary{16}` → some runtime rule must fire), which is exactly the never-silent edge
  the static `a == b` constraint enforces *at compile time* today. Pushing it to runtime weakens G2.
- **What breaks:** this **contradicts RFC-0032 §5 D5** — it changes the **kernel prim ABI** (width becomes
  a runtime operand), i.e. it grows the trusted base (KC-3) and is a `kpr`/E19-1 concern, **not** a surface
  feature. It also breaks the value-model's static-width representation (`Repr::Binary` carries width
  structurally). **Rejected** as out-of-charter for M-753 (it is not a surface type-system change) and as a
  never-silent regression.

### Option C — bounded finite width sets (enumerate the supported widths)

Instead of a true parameter, generate one specialization per width in a **fixed allowlist**
(`{1, 8, 16, 32, 64}`), e.g. by a `derive`-style generative lowering (DN-38) that emits `eq_8`, `eq_16`, …

- **Type checker:** **no change** — each generated function is monomorphic. The genericity lives in a
  *desugaring* pass, not the type system.
- **Monomorphizer:** no change (the inputs are already monomorphic).
- **Never-silent:** honest *within* the allowlist; a width **outside** the set is an explicit "no such
  specialization" refusal (G2-clean). But it is **not actually width-generic** — a caller at `Binary{12}`
  is refused even though the operation is perfectly well-defined there. That refusal is honest but it does
  **not** satisfy M-718's "a single definition covers *every* width" user story (`issues.yaml:4659–4661`).
- **What breaks:** nothing in the kernel; it is the *weakest* unblock. It is a viable **interim** (it could
  ship the stdlib unblock fast without the `Ty::Binary(Width)` refactor) but it is a stopgap, not the
  feature. Flagged as a possible **phase-0** in §6 if the maintainer wants the stdlib unblock before the
  full refactor lands.

---

## §4 Recommendation — Option A (width as a mono-time-bound const-generic), width-preserving first

**Recommend Option A**, scoped to **width-preserving** generics in the first increment, for three reasons:

1. **It is the only option that is genuinely width-generic and stays a surface feature** (RFC-0032 §5 D5).
   Option B grows the kernel (rejected); Option C is not actually generic (a stopgap). Option A reuses the
   *existing* monomorphization model — pin the parameter at the call site, specialize to a closed concrete
   width, lower through the unchanged concrete-width prims — so **no kernel/`Repr`/prim change** and the
   L1-eval ≡ L0-interp ≡ AOT differential runs on a closed L0 program, exactly as it does for type
   generics today (`mono.rs:9–13`).
2. **It is structurally the type-parameter machinery, one slot over.** `Ty::Var` ↔ `Width::Var`;
   `subst_ty`/`unify`/`param_subst` gain a width arm; mono's `param_subst` pins widths the way it pins
   types; the undetermined-parameter `Residual` covers an undetermined width unchanged. The design is a
   *faithful analogue*, which lowers both the design risk and the review surface (the invariants are
   already proven for the type case).
3. **Never-silent falls out for free.** The width-preserving prim signatures (`a == b ⇒ result a`) are
   *already* the right shape; lifting `a`/`b` to a variable `N` makes the constraint `N ~ N` (trivially
   satisfied) and the concrete pin deferred to mono. An undetermined `N` is a `Residual`
   (`mono.rs:1045–1060`), never a guessed default — the **same** discipline the existing `UnresolvedWidth`
   bare-decimal refusal already enforces (`checkty.rs:2596–2605`). **No silent default width, ever (G2).**

### Honesty / never-silent implications (VR-5 / G2) — load-bearing

- **An undetermined width is a never-silent `Residual`, not `Binary{8}`.** This is the single most
  important invariant. The checker must refuse a call that does not pin `N` (from an argument, an
  ascription, or the expected return) — never fabricate a default (the `Ty::Var` precedent,
  `checkty.rs:2657–2663`; the mono precedent, `mono.rs:1045–1060`).
- **No `Swap` is inserted by width genericity.** A width *mismatch* (`Binary{8}` where `Binary{16}` is
  required) stays an explicit error, never a silent re-representation (S1; `checkty.rs:299–305`,
  `347–350`). Width-generics adds *parametricity over a width*, not *coercion between widths* — converting
  between two concrete widths is a separate, explicit `swap` / the prospective width-cast prim (§5).
- **Guarantee tags are inherited, not invented.** A width-generic `fn` carries **no new guarantee**: each
  monomorphic specialization inherits exactly the tag of the underlying concrete-width op (mono recomputes
  totality structurally and never fabricates a verdict, `mono.rs:341–361`). `id_bits<N>` is `Exact` because
  `Binary{N}` identity is; `eq_w<N>` is whatever `eq` is at each width. Width-genericity is **tag-neutral**
  (it does not upgrade or downgrade — VR-5).
- **Honest identity fragmentation.** `eq_w` at `Binary{8}` and at `Binary{16}` are **two** functions with
  **two** content hashes (the mangling records it, `mono.rs:19–24`); width-generics does not claim
  "one body for all widths" (it is monomorphized, like type generics). Stated, not hidden.

---

## §5 Interactions

- **Collections lookup (M-716, `lib/std/collections.myc`).** The direct unblock: `map_get` /
  `set_contains` become `fn map_get<N>(m: Map<Binary{N}, V>, k: Binary{N}) -> Option<V>`, generalizing the
  `Binary{8}` pin (`collections.myc:106–110, 129–130`). (Generic-key lookup also wants the **value** and
  the **key-type** generic — width-genericity covers the *width* dimension; the *type* dimension is already
  the existing `Ty::Var` generics, so the two compose. The current `map_get` is doubly-monomorphic
  (`Binary{8}` keys *and* values); the full generalization is `Map<K, V>` with `K: Eq` once an `Eq` trait
  over width-generic `eq` exists — see Q5.)
- **Generic math (M-718).** Directly satisfies the precondition M-718 names in its `depends_on`
  (`issues.yaml:4566–4587`): a single `fn add<N>(a: Ternary{N}, b: Ternary{N}) -> Ternary{N}` replaces a
  per-width copy. The body **already exists at the prim level** (trit arithmetic is surfaced,
  `prim_kernel_name`, `checkty.rs:3817–3819`), so — per M-718's body note — a **width-generic ternary-math
  leaf is the first increment** once M-753 lands.
- **The prospective width-cast / width-reinterpretation prim.** Width-generics is **parametricity** over a
  width, *not* conversion **between** widths. A program that needs to *change* a value's width (truncate /
  zero-extend / sign-extend) needs an explicit, never-silent cast op — a **separate** concern that should
  live in its own note (referenced by the prompt as "DN-41", which **does not yet exist** — flagged in §7
  Q6; this note does **not** depend on it and does **not** cite it as authority). The clean division: DN-42
  lets you write *one* body that works at *whatever* width the caller pins; the width-cast prim lets a
  caller *move* a value from one concrete width to another. They are orthogonal and compose.
- **RFC-0032 §5 D5 (ownership).** This note is the M-753 design, owned by **E11-1/`s10`** per D5 — a
  surface-language type-system feature editing `mycelium-l1`, deliberately **not** the prims/reprs leg
  (E19-1), so exactly one leg edits the L1 type system (no `kpr`↔`s10` collision). M-751 closed as a
  pointer to M-753 (RFC-0032 §5 D5; `issues.yaml` M-753 body).
- **Surface stability (M-708).** M-753 `depends_on: [M-708]` (done) — the stage-1 generics/traits surface
  is stabilized, so width-generics extends a settled foundation rather than a moving one
  (`issues.yaml:3500–3531`). M-708's audit already flagged "width-granular coherence" as a stage-1 boundary
  (`issues.yaml:3522–3524`) — width-generics intersects coherence (Q5).

---

## §6 Definition of Done (for the eventual M-753 impl) + migration/sequencing

**Definition of Done** (the gate the implementation must meet — mirrors `issues.yaml` M-753 DoD, refined):

1. A width parameter **parses, type-checks, and monomorphizes** to closed concrete-width L0: a
   width-generic op runs **three-way** (L1-eval ≡ L0-interp ≡ AOT) at ≥2 distinct widths from one
   definition.
2. An **undetermined width** at a call site is a **never-silent `Residual`/`CheckError`** with a teaching
   message — **never** a defaulted width (G2/VR-5). A property/regression test pins this (the
   `UnresolvedWidth`/`REJECT_EXPECTED` pattern).
3. A **width mismatch** (`Binary{8}` vs `Binary{16}`) is an explicit refusal, **never** a silent `Swap`
   (S1). Test-covered.
4. A **width-generic ternary-math `.myc` smoke port** (per M-718's body note — trit arithmetic already
   surfaced) demonstrates the unblock; **E13-1 M-718's** general-surface precondition is marked satisfied
   with a pointer.
5. **Tags are honest and inherited** — a width-generic op's specialization carries the underlying op's tag,
   no upgrade (VR-5). Monomorphic and non-generic programs are **byte-identical** (the `Width::Lit` arm is
   the old behavior — a regression-corpus check).
6. `just check` green; the `Ty::Binary(Width)` refactor is exhaustively covered (Rust exhaustiveness ⇒ a
   missed arm is a compile error, not a silent bug).

**Migration / sequencing:**

- **E11-1/`s10`-owned, serial on the L1 type system.** The impl edits `checkty.rs` (`Ty`, `resolve_ty`,
  `unify`/`subst_ty`/`param_subst`, the prim path) and `mono.rs` (width pinning) — the `s10` collision
  surface. Per RFC-0032 §5 D5 / the swarm file-ownership rule, these edits are **serial** (one leg editing
  the L1 type system), not fanned out across agents.
- **Suggested increment order:** (a) the `Ty::Binary(u32) → Ty::Binary(Width)` mechanical refactor
  (behavior-preserving, `Width::Lit` only — lands and stays green before any genericity); (b) the
  `Width::Var` resolution + unify/subst arms (checker carries `N` symbolically); (c) mono width-pinning +
  the `Residual` on undetermined `N`; (d) the width-generic ternary-math `.myc` leaf (the M-718 unblock).
- **Optional phase-0 (Option C interim).** If the maintainer wants the collections/math unblock **before**
  the full refactor, a generative-lowering allowlist (Option C, §3) ships a fixed-width-set stdlib quickly;
  it is a **stopgap, flagged as such**, superseded by Option A. Recommended only if schedule pressure
  warrants (it is honest but not the feature — §7 Q7).

---

## §7 Open questions (for the maintainer — flagged; this is a proposal, ratification pending)

1. **Q1 — Surface syntax for a width parameter.** Same `<…>` list disambiguated by use (a name in a width
   slot is a width param — minimal grammar, context-sensitive), or a **marked** form (`fn f<const N>` /
   `fn f<{N}>` — explicit, more grammar)? *Recommendation leans marked-for-clarity, but this is a genuine
   maintainer call — not settled.*
2. **Q2 — Width arithmetic in types** (`Binary{N+1}`). Deferred from the first increment (needs a
   const-expression layer in the type grammar). Is width-**preserving**-only acceptable for v1 (it covers
   `eq`/`lt`/`add_bin`/math), with `N+1`-style as a separate future feature? *Recommendation: yes, defer.*
3. **Q3 — Width bounds / refinements** (`fn f<N where N <= 64>`). Needed eventually for ops with a width
   ceiling; deferred. Acceptable to ship unbounded `N` first (a too-large width is refused at the
   *concrete* prim/encode boundary, never silently — `encode_binary` already refuses, `checkty.rs:3704`)?
4. **Q4 — Which paradigms in v1?** `Binary{N}` + `Ternary{N}` (surfaced arithmetic/comparison) for the
   first increment; `Dense{D, S}` and `Seq{T, N}` width-genericity later? *Recommendation: Binary/Ternary
   first.*
5. **Q5 — Width-generics × coherence (the hard interaction).** `type_head` **erases width** today
   (`checkty.rs:238–254`) — stage-1 keys instances per *head*, conservatively (M-708 flagged
   width-granular coherence as a stage-1 boundary, `issues.yaml:3522–3524`). A width-generic `impl Eq for
   Binary{N}` interacts with that erasure. Does v1 allow a **width-generic instance**, or only width-generic
   **free functions** (deferring generic-instance coherence)? *Recommendation: free functions first;
   width-generic instances deferred — this is the least-settled corner and should not block the unblock.*
   **FLAG: genuinely uncertain — do not over-claim a settled answer here (VR-5).**
6. **Q6 — The width-cast prim sibling note.** The prompt referenced a "width-cast prim DN-41" as a sibling;
   **no such note exists yet** (DN-40 is the current highest). Should a separate note specify never-silent
   width truncation/extension (orthogonal to this proposal — §5)? DN-42 does **not** depend on it.
7. **Q7 — Interim Option C?** Ship a generative-lowering fixed-width-set stdlib (Option C) as a flagged
   stopgap to unblock M-716/M-718 *before* the `Ty::Binary(Width)` refactor — or hold the unblock for the
   real feature (Option A)? *Recommendation: hold for Option A unless schedule pressure warrants; if taken,
   Option C is explicitly a superseded stopgap.*

---

## §8 Grounding / honesty

- **`crates/mycelium-l1/src/ast.rs`** (read 2026-06-26) — `BaseType::Binary(u32)` / `Ternary(u32)`
  (lines 327/329); `TypeParam { name, bounds }` (222), `FnSig { params, value_params, ret, effects }`
  (231), `param_names` (258). Grounds "width is a concrete `u32` literal" and the parameter machinery a
  width param would extend.
- **`crates/mycelium-l1/src/checkty.rs`** (read 2026-06-26) — `Ty::Binary(u32)`/`Ternary(u32)` (55–92);
  `unify`/`subst_ty`/`param_subst`/`has_var` (260–352); `resolve_ty` (457, the `Ty::Var` arm at 483);
  the prim path + `UnresolvedWidth` refusal (2570–2617); `check_app_generic_fn` (2626) + the
  undetermined-parameter refusal (2657–2663); `prim_sig` width-equality arms (3779–3794);
  `prim_kernel_name` (3798–3823); `encode_binary` (3704). Grounds §1, the Option-A checker changes, and
  the never-silent precedent.
- **`crates/mycelium-l1/src/mono.rs`** (read 2026-06-26) — the monomorphization model (1–47),
  `param_subst`/`unify` pinning (`emit_fn` 373), honest identity fragmentation (19–24), the
  undetermined-parameter `Residual` (1045–1060), totality recomputed-not-fabricated (341–361). Grounds the
  Option-A mono changes + the inherited-tag claim.
- **`lib/std/collections.myc`** (read 2026-06-26) — `map_get`/`set_contains` monomorphic at `Binary{8}`
  because `eq` is width-typed (15–16, 92–93, 106–110, 129–130). Grounds the M-716 cost + the unblock.
- **`lib/std/cmp.myc`** (read 2026-06-26) — the honesty boundary: width-type ordering/equality not
  self-hosted there (14–19). Grounds the §1 cost.
- **`tools/github/issues.yaml`** (read 2026-06-26) — M-753 (5635–5667, `depends_on: [M-708]`,
  `needs-design`), M-718 (4566–4613, `blocked`, `depends_on: [..., M-753]`), M-716 (4426–4454, generic-key
  lookup flagged deferred to M-753), M-708 (3500–3531, done — width-granular coherence flagged).
- **`docs/rfcs/RFC-0032-Kernel-Self-Hosting-Enablement-Surface.md`** (read 2026-06-26) — §5 D5 (168–175,
  width-generics → E11-1/`s10`, M-751 → M-753), D7 sequencing (193–204). Grounds the ownership + scope.
- **House rules** — G2 (never-silent), VR-5 (no tag upgrade past basis), S1 (no silent `Swap`), KC-3 (small
  auditable kernel — Option A grows **no** kernel), house rule #3 (append-only — this note is **Proposed**),
  house rule #4 (assent on merit — §7 surfaces the unsettled corners).

**Honest uncertainty.** The recommendation (Option A) is `Declared` — a reasoned design direction, not a
proven one. The least-settled corner is **width-generics × coherence** (Q5): whether v1 admits
width-generic *instances* or only free functions is **not** decided here, and the note deliberately does
**not** claim a settled answer (VR-5). The surface syntax (Q1) is likewise a maintainer call. The
`Ty::Binary(u32) → Ty::Binary(Width)` refactor is wide; its risk is *bounded* by Rust exhaustiveness (a
missed arm is a compile error, never a silent bug) but the **effort** is real and stated, not minimized.

---

### Changelog

| Date | Status | Note |
|---|---|---|
| 2026-06-27 | **Accepted** *(records the ratification + the M-753 landing; append-only — brings this changelog current with the Status line)* | The maintainer **ratified Option A** (width as a mono-time-bound const-generic; v1 scope = width-generic free functions, deferring the Q5 instance/coherence corner) — the transition recorded in the Status header on 2026-06-26 but not previously appended here. The implementation, **M-753**, is now **`status: done`** (issues.yaml; rsm W1, 2026-06-27): `Ty::Binary(Width)`/`Ty::Ternary(Width)` with `Width::{Lit, Var}`; positional-by-use `fn f<N>(x: Binary{N})`; `unify` binds a width-var **same-paradigm only** (binary↔binary, ternary↔ternary; a cross-paradigm or width-mismatch is a never-silent refusal — G2), **including width-var ↔ width-var pass-through** (mirroring type-var pass-through; this is what lets a width param delegate/recurse); mono pins `N` per call site, and an undetermined width is an explicit `Residual`, never a guessed default (VR-5). On this, the width-generic stdlib surface (**M-718**: `std.cmp`/`std.math`/`std.collections` over `Binary{N}`/`Ternary{M}`) executes three-way (`crates/mycelium-l1/tests/{width_generic,std_*,std_generic_conformance}.rs`); agreement `Empirical`. Q5 (instances/coherence over width-generics) remains deferred. Per house rule #3 this note records the landing; it does **not** itself move to *Enacted* (a design note's role ends at Accepted — the impl decision-state lives in RFC-0032 §5 D5 + issues.yaml M-753). |
| 2026-06-26 | **Proposed** | Initial proposal (M-753, E11-1/`s10`; RFC-0032 §5 D5). Records the problem (width is a concrete `u32` literal ⇒ width-fixed `eq`/`lt`/`add_bin` ⇒ monomorphic-`Binary{8}` stdlib), three design options (A: width as a mono-time-bound const-generic, recommended; B: runtime width tag, rejected — grows the kernel, weakens G2; C: bounded finite width sets, an honest stopgap), the recommendation + never-silent/VR-5 implications, the M-718/M-716/width-cast/RFC-0032-D5 interactions, a Definition of Done + migration/sequencing, and the open questions for the maintainer. **Enacts no code; moves no other decision's status.** Ratification pending (house rule #3 — Proposed, never self-Accepted). |
