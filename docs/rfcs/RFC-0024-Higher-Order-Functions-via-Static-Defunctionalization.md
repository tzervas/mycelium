# RFC-0024 — Higher-Order Functions via Static Defunctionalization

| Field | Value |
|---|---|
| **RFC** | 0024 |
| **Status** | **Proposed** (2026-06-23) — narrows RFC-0007 §4.4's "no function values" clause for the **named-function** case (append-only; RFC-0007 unchanged). Implementation = epic **E7-3** (M-684 this RFC · M-685 surface · M-686 checker · M-687 mono · M-688 tests). Per the swarm-integration rule it moves to *"implemented (Rust-first), pending ratification"* as the leaves land — **never silently to `Accepted`** (VR-5). **implemented (Rust-first), pending ratification** (2026-06-23, M-685/686/687/688 + M-649 — landed on the l1-capstone head; function type `BaseType::Fn` + checker `Ty::Fn` + defunctionalizing mono + `std.result` consumer; no `mycelium-core` change KC-3; three-way differential agreement `Empirical`). |
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

## 5. Scope & honesty boundary

- **In scope:** named top-level functions as first-class values, **statically resolved at the call site**.
- **Out of scope — explicit never-silent `Residual`, deferred:** closures/lambdas (which capture an
  environment); a function value that flows **dynamically** (out of a `match`, a data field, or a fn
  return) so it is not statically resolvable at the call site; partial application; a **still-generic**
  function passed as a value; and full **Reynolds defunctionalization** (a fn-tag sum + an `apply`
  dispatch) — the eventual generalization for the dynamic cases.
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
r4 (the `Lam`/`App` nodes that stay unused). Multi-argument arrows, closures, and full
defunctionalization remain future work (a v2 of this RFC). Operator-as-value (DN-23) is unblocked as a
side benefit: a word operator (`add`) is a named fn, hence a first-class value once this lands.
