# RFC-0024 — Higher-Order Functions via Static Defunctionalization

| Field | Value |
|---|---|
| **RFC** | 0024 |
| **Status** | **Accepted** (2026-06-28) — **ratified by maintainer 2026-06-28 (in-session)**. Decisions recorded: **(a) currying for multi-arg arrows is IN SCOPE for M-704** — currying is the multi-arg route (partial application via §4A.5 machinery, gated on tuple-type prerequisite per §4A.8; no separate mechanism needed); **(b) still-generic-fn-passed-as-value is IN SCOPE for M-704** — no longer deferred; folded into the §4A Reynolds defunctionalization implementation (see §5). Algorithm tag stays `Declared` (no change in basis; VR-5). → Enacted once M-704 lands. Prior status chain (append-only): **Proposed** (2026-06-23) — narrows RFC-0007 §4.4's "no function values" clause for the **named-function** case (append-only; RFC-0007 unchanged). Implementation = epic **E7-3** (M-684 this RFC · M-685 surface · M-686 checker · M-687 mono · M-688 tests). Per the swarm-integration rule it moves to *"implemented (Rust-first), pending ratification"* as the leaves land — **never silently to `Accepted`** (VR-5). **implemented (Rust-first), pending ratification** (2026-06-23, M-685/686/687/688 + M-649 — landed on the l1-capstone head; function type `BaseType::Fn` + checker `Ty::Fn` + defunctionalizing mono + `std.result` consumer; no `mycelium-core` change KC-3; three-way differential agreement `Empirical`). **Design-draft extension (2026-06-28):** §4A specifies the **full Reynolds construction** (closures + partial application + dynamic fn-flow) generalizing the landed named-fn case — a `Declared`-with-argument *design proposal*, **not** yet implemented (impl tracked as M-704). |
| **Type** | Foundational / normative (once Accepted) |
| **Date** | June 23, 2026 |
| **Depends on** | RFC-0007 §4.1–§4.4 (L1 kernel calculus; `Lam`/`App` already exist in Core-IR per RFC-0001 r4; §4.4 keeps the v0 *surface* first-order — this RFC narrows that for named fns); RFC-0019 §4.4 (monomorphization as the elaboration vehicle — `crates/mycelium-l1/src/mono.rs`, M-673); RFC-0006 LR-2; ADR-003 (content-addressed identity); ADR-006 (no black boxes); KC-3 (small auditable kernel — **no `mycelium-core` change**); DN-23 (operator syntax — word operators become first-class once this lands); `docs/spec/grammar/mycelium.ebnf` |
| **Coupled with** | `crates/mycelium-l1` (parser/AST/checker/mono — M-685…M-688, the prototype whose kernel node budget must **not** grow); `docs/spec/grammar/mycelium.ebnf` (the `->` function-type production); `lib/std/result.myc` (M-649 — the first consumer: `map`/`and_then`/`fold`) |

## 1. Summary

RFC-0007 §4.4 keeps the v0 **surface** first-order: there is no function type as a value and
application is first-order only. This RFC adds a **minimal higher-order surface** — the function type
`A -> B` and **named top-level functions as first-class values** — and lowers it by **static
defunctionalization** entirely in the L1 **frontend** (`mono.rs`), so the **L0 kernel stays
first-order** and **`mycelium-core` is untouched (KC-3)**. It is the exact discipline M-673 used to
make generics/traits run (monomorphization is a frontend specialization pass; this extends it). The
immediate consumer is the first self-hosted stdlib nodule `std.result` (M-649): `map`/`and_then`/`fold`.

## 2. Motivation

`std.result`'s combinators (`map`/`and_then`/`fold`) cannot self-host without function values. Core-IR
already has `Lam`/`App`/`Fix` (RFC-0001 r4), but the v0 surface deliberately does not expose them, and
exposing closures wholesale would enlarge the trusted base. **Defunctionalization** is the standard way
to compile higher-order code to first-order: specialize each higher-order call site by the function it
is given. Because every in-scope call site passes a *statically-known top-level function*, the
specialization is complete and closes to first-order L0 — no kernel value for functions is needed.

## 3. Surface

- **Function type `A -> B`** in parameter and field type positions (e.g. `f: A -> B`). **Single-argument
  arrow in v1**; multi-argument `(A, B) -> C` is **deferred and flagged** (the v0 surface has no tuple
  type). Right-associative; `@` (the guarantee index) binds tighter than `->`.
  - *Unambiguous (grounded):* `->` lexes as the atomic token `Tok::Arrow` and `=>` as the atomic
    `Tok::FatArrow` (`crates/mycelium-l1/src/lexer.rs` `lex_dash`/`lex_eq`) — distinct tokens, so a `->`
    in type position never collides with a match arm's `=>`. `Tok::Arrow` is consumed today only by the
    signature's `(...) -> ret`; adding it inside `parse_type_ref` is a localized, additive change.
- **A bare top-level function name in value position is a function value** (no sigil). It already
  parses as `Expr::Path` (a name not followed by `(`); the checker gives it `Ty::Fn`. A *called*
  function `f(x)` stays `Expr::App`. Referencing a **generic** fn bare, without enough context to fix
  its type arguments, is a never-silent refusal.

```mycelium
fn map<A, B, E>(r: Result<A, E>, f: A -> B) -> Result<B, E> =
  match r { Ok(x) => Ok(f(x)), Err(e) => Err(e) }

fn and_then<A, B, E>(r: Result<A, E>, f: A -> Result<B, E>) -> Result<B, E> =
  match r { Ok(x) => f(x), Err(e) => Err(e) }

// fold = the catamorphism: two SINGLE-arg fns (needs no multi-arg arrow).
fn fold<A, E, B>(r: Result<A, E>, on_ok: A -> B, on_err: E -> B) -> B =
  match r { Ok(x) => on_ok(x), Err(e) => on_err(e) }

fn double(x: Binary{8}) -> Binary{8} = add(x, x)
fn main() -> Result<Binary{8}, Binary{8}> = map(mk_ok(), double)   // `double` is a value here
```

## 4. Static-defunctionalization semantics

Each higher-order call is **specialized by its statically-known function argument(s)**, extending the
M-673 worklist (`mono.rs`): the work-item key for a function carries the resolved fn-argument
identities, so `map` called with `f = double` becomes a distinct monomorphic function whose body has
each `f(x)` **rewritten to a direct call** to `double`, and the function-valued parameter is dropped.
The result is **closed first-order L0** — every application head is again a concrete name, which the
existing trusted elaborator / `mycelium-core` registry run unchanged. The dispatch choice is reified in
the EXPLAIN selection record (extends M-673's `MonoSelections`; no black boxes — house rule #2).

**No `mycelium-core` node is added; the kernel node budget is unchanged (KC-3).** The Core-IR `Lam`/`App`
nodes (RFC-0001 r4) remain deliberately **unused** by this approach — recorded here so the choice not
to spend kernel budget is explicit.

## 4A. The full Reynolds construction (closures, partial application, dynamic fn-flow)

> **Status of this section.** `Declared`-with-argument — a **design proposal**, not implemented code.
> §4 (named functions, **statically** resolved at the call site) is the **landed** case (`Empirical`
> three-way agreement). §4A is the **generalization** that closes §5's residuals: it specifies the
> algorithm, the lowering, the typing, and the verification, but **no leaf has landed it** (impl =
> **M-704**). Every claim here is therefore tagged `Declared` unless it restates §4's landed
> mechanism. The construction is the standard Reynolds defunctionalization (Reynolds 1972,
> *Definitional Interpreters*) specialized to Mycelium's frontend-only, **no-new-kernel-node**
> constraint (KC-3).

### 4A.1 Why a generalization is needed (the boundary §4 cannot cross)

§4's specialization is **complete only when every function value is statically known at its call
site** — the defunctionalizer reads the *syntactic* fn-argument (`Expr::Path` to a top-level fn) and
bakes its identity into the specialization key (`mono.rs` `resolve_fn_args` → `Item::Fn.fn_args` →
`mangle_hof_decl`). Three cases break that precondition and today emit a never-silent `Residual`
(grounded — `mono.rs`/`checkty.rs`):

1. **Closures (environment-capturing lambdas).** `Expr::Lambda { params, body }` parses (RFC-0037 D5)
   but the checker, mono, elaborator, and evaluator all refuse it (`checkty.rs:2146`, `mono.rs:862`,
   `elab.rs:899`, `eval.rs:572`). A lambda's body may reference **free variables** bound in the
   enclosing scope — there is no top-level name to bake in, so §4's `resolve_fn_args` has nothing to
   resolve.
2. **Dynamic fn-flow.** A function value that flows out of a `match` arm, a data field, or a function
   return is **not** an `Expr::Path` at the call site, so `resolve_fn_args` refuses it (`mono.rs:1431`,
   the "must be a top-level function name (a path)" residual). The call site cannot name *one* callee
   to specialize against — the function is chosen at runtime.
3. **Partial application.** Supplying *some* arguments to a multi-argument function yields a function
   value that captures the supplied arguments — a closure by another name (its captured environment is
   the already-applied prefix).

These are one problem with one answer: a **uniform first-order representation of "a function plus its
captured environment"** that a generated dispatcher can apply. That is exactly Reynolds
defunctionalization.

### 4A.2 The mechanism (a fn-tag sum + a generated `apply` dispatcher) — no new L0 node

For each function type `A => B` (`Ty::Fn`, `checkty.rs:111`) that **escapes** (is stored, returned,
match-flowed, or built from a lambda — i.e. cannot be statically specialized away by §4), the pass
synthesizes, **entirely in the L1 frontend over already-existing L0 constructs**:

- **A fn-tag sum type** `Fn$A$B` — a generated `data` type (a `DataInfo`, the *same* node `emit_data`
  already produces) with **one constructor per distinct lambda / partial-application / named-fn-as-
  escaping-value** of that arrow type. Each constructor's **fields are exactly that closure's captured
  free variables** (§4A.3), in a deterministic order. A captureless lambda or a bare named fn becomes a
  **nullary** constructor.
- **A generated `apply` dispatcher** `apply$A$B(clo: Fn$A$B, x: A) -> B` — an ordinary monomorphic
  `FnDecl` (the *same* node `emit_fn` already produces) whose body is a **`match` on the closure value**
  (`Expr::Match`, already in L0): one arm per constructor, binding the captured fields, then evaluating
  the original lambda body (or the named-fn call) with those bindings in scope.

So a closure becomes a **tagged struct** (its environment) and the apply fn becomes a **first-order
`match`** — both are constructs the trusted elaborator / `mycelium-core` registry already lower
**unchanged**. **No `mycelium-core` node is added; the kernel node budget is unchanged (KC-3)** — this
is the §4-discipline extended, exactly as §6 anticipated ("the eventual generalization"). The Core-IR
`Lam`/`App` nodes (RFC-0001 r4) **remain unused** (§4 ground; recorded so the choice not to spend
kernel budget stays explicit).

**Hybrid with §4 (never-regress).** §4A does **not** replace §4: a call site whose fn-argument is
statically known still takes §4's **direct-call specialization** (no tag, no dispatch — zero overhead).
§4A is reached **only** for the escaping/dynamic residue §4 cannot close. The pass tries §4 first; §4A
is the fallback for what remains. (Decision D1, §4A.7.)

### 4A.3 Capture-set analysis (which free variables a lambda closes over)

The closure-struct's fields are the lambda's **captured free variables** — computed by a standard
free-variable walk over `Expr::Lambda { params, body }`:

```
capture(λ) = freevars(body) \ (params(λ) ∪ toplevel-names)
```

- `freevars(e)` is the set of `Expr::Path` single-segment names occurring in `e` that are **not**
  bound by an enclosing binder *within* `e` (`Let`, `Match` arm patterns, `For`, inner `Lambda`
  params). This is a pure structural walk — the AST already exposes every binder (`ast.rs`), and
  `mono.rs`'s `rewrite_*` family already threads a `scope: Vec<(String, Ty)>` that pins exactly which
  names are local vs. captured.
- We **subtract the lambda's own parameters** (`params(λ)`) and **all top-level names** (functions,
  constructors, prims — these are not captured; a named fn referenced inside a lambda body lowers by
  §4, not by capture). What remains is the genuine environment.
- Each captured variable carries its **concrete type** from the enclosing `scope` (mono runs *after*
  checking, so every binder type is known), giving the struct field's type. **A captured variable
  whose type is itself `Ty::Fn` recursively becomes a closure field** (a closure capturing a closure)
  — the construction nests.
- **Determinism (G2).** Capture order is fixed (first-occurrence, or sorted by binder name — an impl
  detail to pin in M-704; the spec requires *a* total deterministic order so two builds produce the
  identical struct and identical content hash, per §4's identity-fragmentation discipline). The pass
  records the capture set in the EXPLAIN record (§4A.6) — never a black box (house rule #2).

### 4A.4 Closure-struct lowering (one synthetic struct + a capture binding per lambda)

A lambda expression `λ = lambda(p: A) => body` of type `A => B` at a program point lowers in two parts:

1. **At the lambda's *definition* site** (where it is written): emit a **construction** of its tag-sum
   constructor, applied to the **captured values** (the current bindings of `capture(λ)` from scope) —
   an ordinary `Expr::App { head: Ctor_λ, args: [captured…] }`. This is the **capture binding**: the
   environment is snapshotted by value at definition (value-semantics — Mycelium's model; RFC-0001),
   so the closure is a plain immutable data value.
2. **At each *application* site** `f(x)` where `f: A => B` is an escaping closure value: rewrite to
   `apply$A$B(f, x)` — a call to the generated dispatcher. (A statically-known `f` still takes §4's
   direct call; §4A.2 hybrid.)

The **generated tag-sum + dispatcher** are emitted **once per arrow type** that escapes, accumulating a
constructor + an apply-arm per distinct closure of that type — mirroring how `emit_data`/`emit_fn`
accumulate monomorphic instances on the worklist (`mono.rs`). The naming reuses the existing injective
mangling (`$` joints, the `#` nullary-data tag — `mono.rs:1771`+), extended with a reserved arrow
infix (proposal: `Fn$<mangled A>$<mangled B>`; the existing `HOF_FN_…__TO__…` mangle at `mono.rs:1848`
is the precedent token and can be reused). **No surface-identifier collision** (the mangle alphabet is
disjoint from surface names — §4 ground).

### 4A.5 Partial application & dynamic fn-flow as the same construction

- **Partial application.** `g(a)` where `g: A => B => C` and only `a: A` is supplied yields a value of
  type `B => C`. This is **a closure capturing `a`**: lower it to the tag-sum constructor for the
  arrow `B => C` whose single field is `a`, with the apply-arm evaluating `g(a, b)`. Partial
  application is therefore **not a new mechanism** — it is §4A.4 with the captured environment being the
  already-applied argument prefix. (Prerequisite: the **multi-argument arrow**, §4A.8.)
- **Dynamic fn-flow.** A function value out of a `match`/field/return is, by typing, *some* value of
  `Fn$A$B`; every producer of that arrow type has already contributed its constructor to the sum, and
  every consumer applies it through `apply$A$B`. The dispatcher's `match` **is** the dynamic dispatch —
  resolved at runtime over the closed (whole-program) set of constructors of that arrow type. Because
  defunctionalization is **whole-program** (the worklist sees every reachable closure of each arrow),
  the sum is **complete and closed** — no open-world case, no fallback arm needed; an unreachable
  constructor simply never appears. (Whole-program closure is the §4 precondition that makes the
  named-fn case complete; §4A inherits it.)

### 4A.6 Composition with `Expr::Lambda` (RFC-0037) and the typing

- **Surface.** `Expr::Lambda { params: Vec<Param>, body }` already parses (RFC-0037 D5; `ast.rs:604`)
  and today is a never-silent `Residual` at every stage (`checkty.rs:2146`, `mono.rs:862`, `elab.rs:899`,
  `eval.rs:572`, `grade.rs:270` gives it `Strength::Declared`). §4A is precisely the work that **turns
  those residuals into the lowering above** — the surface is reserved and waiting; M-704 wires it. The
  `lambda` keyword is the closure form (DN-31, 2026-06-27; gated behind the DN-31 grammar wave per §7's
  M-704 note).
- **Typing (`Ty::Fn`).** A lambda `lambda(p: A) => body` checks to `Ty::Fn(A, B)` where `B =
  infer(body)` under `scope ∪ {p: A}` — the checker already builds `Ty::Fn` for the **typed-parameter
  monomorphic** case (`checkty.rs:2216`–`2274`, currently gated to the named-fn/ascribed forms). §4A
  extends that arm to admit a lambda whose free variables are in scope: the lambda is **well-typed iff
  its capture set is well-typed in the enclosing scope and its body checks at the expected codomain**.
  No new `Ty` variant — `Ty::Fn` is the carrier (`checkty.rs:111`); the closure *struct* is an ordinary
  `Ty::Data` after lowering, so post-mono typing is unchanged. Inferred-parameter lambdas (no `: A`
  ascription) require expected-type-driven inference from the `Ty::Fn` context — already sketched at
  `checkty.rs:2238`–`2242` and completed under M-704.
- **Totality / grade.** A lambda body is walked by the totality and grade passes already
  (`totality.rs:184`/`368`, `grade.rs:270`) — the closure-struct match the pass generates is an
  ordinary `match`, so totality is **recomputed structurally** over the generated `apply` (a
  specialization's verdict equals its source's — the §4 discipline; never fabricated, VR-5).

### 4A.7 Algorithm (the worklist extension, end to end)

Extending `mono.rs`'s worklist (`Item` / `Mono::run`), per closure shape:

1. **Try §4 first.** At a HOF call site, `resolve_fn_args` attempts static resolution (named fn → bake
   identity, direct call). **Unchanged.** (Decision D1.)
2. **Escape detection.** If a function value **escapes** — it is a lambda, a partial application, or
   flows dynamically (not an `Expr::Path` to a top-level fn) — route to §4A instead of emitting the
   current `Residual`.
3. **Per arrow type `A => B`**, enqueue (idempotently, by a canonical key — the dedup discipline of
   `seen`/`item_key`) a **`Fn$A$B` tag-sum** item and an **`apply$A$B` dispatcher** item.
4. **Per escaping closure of that arrow**, compute `capture(λ)` (§4A.3), add a **constructor**
   `Clo$A$B$<n>` (fields = captured types) to the sum, and add a **match-arm** to the dispatcher that
   binds the captured fields and evaluates the body. Record the `ClosureSpecialization` EXPLAIN
   (mirroring `HofSpecialization`, `mono.rs:251`).
5. **Rewrite** the definition site to a constructor application (capture binding) and each dynamic
   application site to `apply$A$B(clo, x)`.
6. **Emit** the sum (`emit_data`) and the dispatcher (`emit_fn`) — both existing emitters; the result
   is **closed first-order L0**, lowered by the unchanged elaborator/registry. Termination holds by the
   same dedup-⟹-finite argument as §4 (finitely many arrow types × finitely many reachable closures).

### 4A.8 Multi-argument arrows — the tuple-type prerequisite (explicit)

Partial application and true binary combinators (e.g. a real `foldl` with `f: A => B => B` consumed as
a 2-ary step) need a **multi-argument arrow `(A, B) -> C`**. The v0 surface has **no tuple type** (§3
ground — "the v0 surface has no tuple type"), and the arrow is single-argument (`Ty::Fn(Box<Ty>,
Box<Ty>)`, `checkty.rs:111`). **The prerequisite is therefore a tuple type** (or, equivalently,
currying `(A, B) -> C` to `A -> B -> C` and lowering each arrow by §4A — which *is* the partial-
application path, §4A.5). The defunctionalization itself is **arity-agnostic** (the apply dispatcher
takes one closure + one argument tuple, or is curried), so **no new defunctionalization mechanism** is
needed for multi-arg — only the surface/`Ty` support for the product. This RFC **specifies** the
construction and **flags the tuple type as the gating prerequisite** for multi-arg/partial application
(a separate surface decision; not granted here — never a silent dependency, G2).

### 4A.9 Verification — three-way differential per closure shape

The acceptance bar mirrors §4's landed `Empirical` agreement, **per closure shape**: for each of
{captureless lambda, single-capture lambda, multi-capture lambda, closure-capturing-closure, partial
application, dynamic-fn-out-of-match, dynamic-fn-as-field}, a fixture must evaluate **identically across
the three paths** — L1-eval ≡ L0-interp ≡ AOT (NFR-7) — on the **defunctionalized** program. This is
`Empirical` (trials) and is the §4A Definition-of-Done gate (below). The capture-set analysis is
additionally checked by a **property test**: `freevars` is invariant under α-renaming of bound
variables, and `capture(λ)` ⊆ the enclosing scope's binders (a bound that, if violated, is a
never-silent failure, not a guess). No tag is upgraded to `Proven` without a checked basis (VR-5).

### 4A.10 Definition of Done (§4A)

§4A is **done** when: (a) `Expr::Lambda` checks, lowers, elaborates, and evaluates (the four current
`Residual`s become the §4A lowering); (b) the §4A.9 three-way differential passes for **every** listed
closure shape; (c) the capture-set property test passes; (d) the EXPLAIN closure record is queryable
(no black box); (e) **no `mycelium-core` node was added** (KC-3 — the STOP-and-flag of §5 still binds);
and (f) the multi-arg/partial cases land **only after** the tuple-type prerequisite (§4A.8) is ratified
separately. Until all hold, the status stays **Proposed / "specified, pending impl (M-704)"** — never
silently Accepted/Enacted (house rule #3 / VR-5).

## 5. Scope & honesty boundary

- **In scope:** named top-level functions as first-class values, **statically resolved at the call site**.
- **Specified (this change, §4A), pending impl — `Residual` until M-704 lands:** **closures/lambdas**
  (environment-capturing), a function value that flows **dynamically** (out of a `match`, a data field,
  or a fn return), **partial application**, and the full **Reynolds defunctionalization** (a fn-tag sum
  + an `apply` dispatch) — **now specified in §4A** as the design-draft generalization (`Declared`-with-
  argument). The surface still emits a never-silent `Residual` at every stage until M-704 wires the §4A
  lowering (G2 — the surface parses, does not yet evaluate). Status of these cases moves from *deferred /
  unspecified* to **"specified, pending impl (M-704)"**.
- **Moved IN SCOPE for M-704 (ratified 2026-06-28):** a **still-generic** function passed as a value (no longer deferred — folded into the M-704 §4A implementation; needs type-argument context to defunctionalize but that context is available at the M-704 call site); and **multi-argument arrows / partial application** (currying = the multi-arg route via §4A.5 `apply` machinery, gated on tuple-type prerequisite per §4A.8 — currying is the multi-arg route; no separate mechanism needed).
- **Guarantee:** `Declared` (a type-level contract + a structural rewrite, not a theorem). Differential
  agreement across the three evaluation paths (L1-eval ≡ L0-interp ≡ AOT) is **`Empirical`** (trials).
  No tag is upgraded to `Proven` without a checked basis (VR-5).
- **STOP-and-flag (KC-3):** if any in-scope case is found that cannot be made closed first-order at L1
  without adding an L0 node, that breaks KC-3 and invalidates this approach — escalate; do not add a
  kernel node.

## 6. Supersession

This RFC **narrows** RFC-0007 §4.4's "function values are out of v0" restriction for the
named-function case only; RFC-0007 is **not** rewritten (append-only — the forward decision lives
here). It references RFC-0019 §4.4 (monomorphization as the shared elaboration vehicle) and RFC-0001
r4 (the `Lam`/`App` nodes that stay unused). **Closures, dynamic fn-flow, and full Reynolds
defunctionalization are now specified in §4A** (this change — a `Declared` design proposal, impl =
M-704), no longer indefinite future work; **multi-argument arrows / partial application** remain gated
on a **tuple-type prerequisite** (§4A.8) — a separate surface decision. Operator-as-value (DN-23) is
unblocked as a side benefit: a word operator (`add`) is a named fn, hence a first-class value once this
lands.

## 7. Changelog

| Date | Status | Note |
|---|---|---|
| 2026-06-27 | **Proposed** (M-704 direction recorded; maintainer-confirmed in-session) | **M-704 (full HOF) scheduled — closures via Reynolds defunctionalization, KC-3-safe.** The §5/§6 residual (closures, multi-arg arrows, partial application, true binary `foldl`) moves from *indefinite Residual* to **scheduled near-term**, gated **behind the DN-31 grammar wave** (closures need the `lambda` keyword + the layout-independent delimiter rules first). **Direction:** generalize §4's static defunctionalization to closures by **Reynolds defunctionalization** — a closure lowers to a **tagged data value** (its captured environment) + a generated **`apply` dispatch**, all in **existing L0** (no new kernel node — KC-3 holds, per §6's "eventual generalization"); multi-arg arrows + partial application generalize from the same function-type + `apply` machinery. **Surface:** the explicit **`lambda` keyword** (DN-31, 2026-06-27) is the closure form; the layout-independent grammar lets lambda-chains stream or format freely. **Sequencing:** closures first (the headline), then multi-arg / partial / true `foldl`. Guarantee stays `Declared` (contract) + `Empirical` (three-way) — nothing `Proven` (VR-5). RFC stays **Proposed**; this records the M-704 direction, enacts no code. |
| 2026-06-28 | **Proposed** (design draft — status unchanged; forward-note: *Proposed → Accepted pending maintainer review*) | **§4A added — the full Reynolds construction (closures + partial application + dynamic fn-flow), group G2 of the ratification map.** Generalizes §4's landed named-fn defunctionalization to environment-capturing closures by a **fn-tag sum type + a generated `apply` dispatcher**, both lowered over **existing L0 constructs** (`emit_data` for the sum, `emit_fn` + an `Expr::Match` for the dispatcher) — **no new `mycelium-core` kernel node** (KC-3 holds, exactly as §6 anticipated). Specifies: **capture-set analysis** (free-variable walk minus params/top-level names → struct fields, recursively for closure-capturing-closure), **closure-struct lowering** (a synthetic constructor per lambda + a value-snapshot capture binding at the definition site), **apply-dispatch generation** (a `match` over the whole-program-closed constructor set = the dynamic dispatch), composition with **`Expr::Lambda`** (RFC-0037 D5 — the four current never-silent `Residual`s become this lowering) and the **`Ty::Fn`** typing (no new `Ty` variant — the closure struct is an ordinary `Ty::Data` post-mono), and the **three-way differential per closure shape** (L1-eval ≡ L0-interp ≡ AOT) plus a capture-set property test. **Multi-arg arrows / partial application are §4A.8-gated on a tuple type the v0 surface lacks** — flagged as a separate surface prerequisite (never a silent dependency). §5's closures/dynamic/partial residual moves from *deferred* to **"specified, pending impl (M-704)"**. **Tags (honest):** §4A is `Declared`-with-argument (a type-level contract + a structural rewrite, design proposal — *not* implemented); the **landed named-fn case (§4) is `Empirical`** (three-way trials); nothing `Proven` (VR-5). **Status not moved** — this records the maintainer-review *request*; only the maintainer accepts (house rule #3). Editorial scope: edits `docs/rfcs/RFC-0024-*.md` only. |
| 2026-06-27 | **implemented (Rust-first), pending ratification** (status unchanged) | **M-715 (rsm S3) — recursive-HOF re-pass.** Extends §4 to the case the base landing did not close: a higher-order parameter **re-passed at a recursive call site** (e.g. `fn map(xs, f) = … Cons(f(h), map(rest, f))`). `mono::resolve_fn_args` now threads a re-passed HOF parameter through `fn_param_subst` as the **same** static specialization, so the recursive callee resolves to the already-specialized function instead of refusing with `Residual`. This makes the self-hosted `lib/std/iter.myc` combinators (`map`/`filter`/`foldl`/`any`/`all`/`find`) execute three-way (L1-eval ≡ L0-interp ≡ AOT; `crates/mycelium-l1/tests/std_iter.rs`). Still `Residual` (§5 unchanged): closures, dynamically-flowing fn values, partial application, multi-arg arrows, and a true binary `foldl` (`f: A -> B -> B`) — deferred to **M-704**. Guarantee unchanged: `Declared` contract + `Empirical` three-way agreement; nothing upgraded to `Proven` (VR-5). No `mycelium-core` node added (KC-3). RFC stays **Proposed / implemented (Rust-first), pending ratification** (not silently Accepted). |
| 2026-06-28 | **Accepted** (ratified by maintainer, in-session) | **Scope decisions recorded for M-704.** (a) Currying for multi-arg arrows is IN SCOPE for M-704 — currying is the multi-arg route via the §4A.5 `apply` dispatch, gated on tuple-type prerequisite (§4A.8); (b) still-generic-fn-passed-as-value is IN SCOPE for M-704 — no longer deferred; folded into the §4A implementation. §5 updated to reflect these moves. Algorithm tag stays `Declared` (VR-5). → Enacted once M-704 lands. |
