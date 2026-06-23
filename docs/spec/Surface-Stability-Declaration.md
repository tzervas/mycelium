# Surface Stability Declaration — Mycelium L1 (Stage-1)

| Field | Value |
|---|---|
| **Status** | **Draft** (2026-06-23; advisory audit, DN-17 posture) |
| **Task** | M-708 (E11-1 — surface stabilization) |
| **Scope** | The L1 *surface* language as implemented in `crates/mycelium-l1/` — generics, traits, effects, the `wild`/FFI surface, phyla, guarantee grading, and operator syntax. |
| **Grounding** | `crates/mycelium-l1/src/` (ground truth); DN-14 §3 (self-hosting readiness table); RFC-0007 §11 (generics), RFC-0019 (traits), RFC-0014 (effects), RFC-0018 (grading), RFC-0024 (HOF), RFC-0025 (operators). |

> **Posture (honesty rule / VR-5).** This is an **audit**, not a ratification: it records, per
> surface feature, whether it is **present** (implemented + tested, with a source/test ref) or
> **deferred** (explicitly out of stage-1, with a forward issue ref — never a silent gap, G2). It
> decides nothing normatively and upgrades no guarantee tag. Each feature's own RFC remains the
> normative source; "stable" here means *the stage-1 surface is settled and documented* — additive
> extensions land per their tracking issues without breaking what is declared present.

---

## 1. Purpose

E11-1 (surface-language completeness) needs a single place that answers, for any surface form a
stdlib author or dogfooding build (`dfb`) might write: **is this stable today, or is it explicitly
deferred?** This document is that answer. It consolidates the per-feature verdicts that were
scattered across DN-14 §3, the checker's own comments, and the RFC residual sections, and it makes
every deferral point at a tracking issue so nothing is silently missing (G2).

A feature marked **present** is implemented in the typechecker/elaborator/evaluator and exercised by
tests; relying on it is safe. A feature marked **deferred** is a never-silent refusal today (an
explicit `CheckError`/`Residual`/parse error) and is tracked by the named issue; do not write code
against it until that issue lands.

---

## 2. Stable surface (present)

Each row is grounded in source + tests; see DN-14 §3 for the fuller evidence chain.

| Feature | Status | Source / test | Guarantee |
|---|---|---|---|
| Value types (repr literals `Binary{n}`/`Ternary{m}`, `Dense` types, `i64`, ADT booleans/tuples) | **present** | `ast.rs` `BaseType`, `checkty.rs` `Ty`, `eval.rs` | n/a (structural) |
| ADTs + pattern matching (nested patterns, Maranget exhaustiveness/redundancy) | **present** | `usefulness.rs`, `decision.rs`, `tests/check.rs` | n/a |
| Functions, self- + mutual recursion (Tarjan SCC → `FixGroup`), `let`, `for`-fold sugar | **present** | `elab.rs` `FixGroup`, `ast.rs` `Expr::{Let,For}` | n/a |
| Generic type parameters (`fn f<A>(…)`, `type List<A>`) — checked + monomorphized to closed L0 | **present** | `checkty.rs` (unification-based instantiation), `mono.rs` (M-673); `tests/differential.rs` | `Declared` |
| Traits + `impl … for …` with coherence (global uniqueness + single-nodule orphan), bounded calls | **present** | `checkty.rs` (M-659), static instance resolution in `mono.rs` (M-673) | `Declared` |
| Effect annotations `!{e1, e2}` (declared ⊇ performed coverage; duplicate = parse refusal) | **present** | `parse.rs`, `checkty.rs::check_effect_coverage` (M-660) | `Declared` |
| `wild` / FFI surface (gated: `@std-sys` nodule + `!{ffi}`; type-checks; **execution staged**) | **present (gated)** | `checkty.rs::check_wild` (M-661); execution is a `Residual` in `elab.rs` | `Declared` |
| Phyla + cross-nodule `pub`/`use` (specific + glob), phylum-wide orphan rule | **present** | `parse_phylum`, `checkty.rs::check_phylum` (M-662) | `Declared` |
| Guarantee grading, stage-1a (`@ g` weaken-only, meet composition, G-App/G-Match/A/G-Swap) | **present** | `grade.rs` (M-663; RFC-0018 Enacted) | `Declared` |
| Structured concurrency `colony { hypha e, … }` (RT2 spawn-order sequentialization) | **present** | `parse_colony`, `eval.rs`; `tests/differential.rs` (M-666) | determinism **Empirical** |
| Higher-order functions, **static** (named fn as value; defunctionalized in `mono.rs`) | **present** | `checkty.rs` `Ty::Fn`, `mono.rs` (RFC-0024; M-685…688) | `Declared`; 3-path agreement **Empirical** |
| Operator syntax — infix/prefix sugar desugaring to word functions (`a + b` → `add(a, b)`) | **present** | `lexer.rs`/`parse.rs` (M-705; RFC-0025); `accept/20`, `tests/differential.rs` | sugar↔word **Empirical** |

---

## 3. Deferred surface (explicit, with forward refs)

Each is a **never-silent refusal today** (explicit `CheckError`/`Residual`/parse error) — G2 — and
is tracked by the named issue. None is a silent gap.

### 3.1 Generics / traits edge cases (stage-1 boundaries)

| Deferred form | Refusal today | Tracked by |
|---|---|---|
| **Multi-parameter traits** (`trait T<A, B>`) and **associated types** | `mono.rs` refuses with `Residual`; checker is single-parameter (`checkty.rs` `Trait.params` stage-1) | RFC-0019 follow-up (no issue minted yet — see §4) |
| **Blanket instances** over a type variable (`impl T for A`, `A` abstract) | `CheckError` — `Ty::Var`/`Ty::Fn` are not legal instance heads (`checkty.rs::type_head`) | RFC-0019 follow-up |
| **Width/shape-granular coherence** (currently head-granular; documented over-rejection: a second `impl … for Binary{k}` at a different width is refused) | `CheckError` — head-keyed coherence (`checkty.rs`) | RFC-0019 follow-up (role/variance machinery, v2) |
| **Runtime dictionary-passing** trait form (RFC-0019 §4.5 literal) | static resolution only; dictionary form is a trusted-core ADR (KC-3) | M-673 follow-up / trusted-core ADR |

### 3.2 Higher-order functions — dynamic cases (RFC-0024 §5 residuals)

| Deferred form | Refusal today | Tracked by |
|---|---|---|
| **Closures / lambdas** (capturing an environment) | `mono.rs` `Residual` — "a function-typed *literal* (closure / lambda) is out of scope (RFC-0024 §5)" | **M-704** |
| **Dynamic fn-flow** (fn value out of a `match` arm, data field, or fn return — not statically resolvable) | `mono.rs` `Residual` (RFC-0024 §5) | **M-704** |
| **Partial application / multi-argument HOF** (`(A, B) -> C`) | `mono.rs` `Residual` — "multi-argument application — not supported in stage-1 (RFC-0024 §5)" | **M-704** |
| **Still-generic function** passed as a value | `mono.rs` `Residual` (FLAG: generic-fn-as-arg — never a silent guess) | **M-704** |

KC-3 note (RFC-0024 §5 STOP-and-flag): full Reynolds defunctionalization (fn-tag sum + `apply`
dispatch) is the eventual generalization for the dynamic cases; if any in-scope case is found that
cannot be made closed first-order at L1 without an L0 node, that breaks KC-3 and requires a new RFC
— it is **flagged, never added silently**.

### 3.3 Effects, grading, operators, runtime vocabulary

| Deferred form | Refusal today | Tracked by |
|---|---|---|
| **Effect → runtime budget** wiring + per-effect budget syntax (`!{retry(<=3)}`) | effects are checker-only metadata (no runtime ledger wiring) | **M-677** |
| **Grade polymorphism** (stage 1b) and **refinement/dependent types** (stage 2) | `grade.rs` is stage-1a only | RFC-0018 stage-1b/2 (future RFCs) |
| **Angle-bracket operators** `< <= > >= << >>` | not lexed as operators (type-arg `<…>` disambiguation pending); the word forms (`lt`/`le`/`gt`/`ge`/`shl`/`shr`) remain available | **M-745** |
| **Non-resolving operator word targets** (`div`/`rem`/`band`/`bor`/`eq`/`ne`/`and`/`or`) | parse + desugar, then explicit "unknown function/prim" downstream | stdlib/kernel prim work (per `lib/std`) |
| **`consume` / `grow` / inherent `impl T { … }`** keywords (DN-03 §1) | reserved-not-active / unimplemented surface | **M-664** |
| **R1 runtime vocabulary** `fuse` / `reclaim` / `tier` | reserved keywords; explicit teaching diagnostic at use | **M-667** |
| **R2 distribution vocabulary** `xloc` / `mesh` / `cyst` / `graft` / `forage` / `backbone` | reserved keywords; explicit teaching diagnostic at use | **M-668** |
| **`VSA{…}` types** | explicit **`CheckError`** in `checkty.rs::resolve_ty` — "VSA types are deferred in the L1 v0 prototype (no value forms yet)" (the *type* itself is refused, not merely staged) | RFC follow-ups (VSA execution) |
| **`Substrate{tag}` value forms** | the *type* resolves (`Ty::Substrate`, accepted in signatures/`consume`), but **no v0 value forms exist** (`checkty.rs:68` — affine `Substrate` is type-level only) | RFC follow-ups (Substrate execution; `consume` surface → M-664) |
| **`wild` execution** (FFI host) | `elab.rs` `Residual` (type-checks + gates; no FFI host in v0) | M-661 follow-up / FFI RFC (RFC-0028) |

---

## 4. Honesty notes & open items

- The **multi-parameter-trait / associated-type / width-granular-coherence** deferrals (§3.1) do
  not yet have a dedicated tracking issue beyond the RFC-0019 stage-1 comments. This declaration
  records them as known stage-1 boundaries; minting their tracking issue is itself a follow-up
  (flagged here rather than left implicit — G2).
- This document is **advisory** and **append-only** in spirit: a feature moving from deferred to
  present is recorded by updating its row (with the landing issue), never by silently deleting a
  deferral. It does not gate; the per-feature RFCs remain normative.
- No guarantee tag is upgraded here. Where a feature's agreement across execution paths is
  **Empirical** (differential trials), it stays Empirical — this audit does not manufacture a
  `Proven` (VR-5).

---

## Meta — changelog

- **2026-06-23 — Draft created (M-708).** Consolidated the stage-1 surface audit: 12 present
  features (generics, traits, effects, `wild`/FFI, phyla, grading, concurrency, static HOF,
  operators) and the explicit deferral set (RFC-0024 §5 dynamic HOF → M-704; angle-bracket ops →
  M-745; effect-budget wiring → M-677; `consume`/`grow`/inherent-`impl` → M-664; runtime vocab →
  M-667/M-668; VSA/Substrate + `wild` execution → RFC follow-ups). Advisory, append-only; no
  normative move (VR-5 / house rule #3).
