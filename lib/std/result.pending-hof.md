# `std.result` — pending higher-order combinators (PSEUDOCODE)

> **Status: PSEUDOCODE — NOT YET EXECUTABLE.** Pending the HOF capstone
> (**RFC-0024** — function types + static defunctionalization). Honest:
> `Declared`/deferred — this is the *intended* API, **not** a silent stub
> (G2/VR-5). M-649 / `run`-kickoff capstone.

## Why a companion file (not interior comments in `result.myc`)

`lib/std/` is a `mycfmt`-gated project root, and **`mycfmt` v0 preserves only the
structured metadata header + code** — interior comments and stray header comments are
explicitly *refused* (a documented v0 deferral, never silently dropped — G2). So the
pseudocode lives here, beside the nodule, until it can be authored as real code.

## The combinators (finalized RFC-0024 surface)

The v0 surface is first-order: there is no function type `A -> B` as a value yet, so
`myc-check` rejects these today. They are written in the **finalized RFC-0024 surface**
(single-arg arrow `A -> B`; a bare top-level fn name is a function value) so the swap
is mechanical once the capstone lands.

```mycelium
fn map<A, B, E>(r: Result<A, E>, f: A -> B) -> Result<B, E> =
  match r { Ok(x) => Ok(f(x)), Err(e) => Err(e) }

fn and_then<A, B, E>(r: Result<A, E>, f: A -> Result<B, E>) -> Result<B, E> =
  match r { Ok(x) => f(x), Err(e) => Err(e) }

// Result.fold is the catamorphism: two SINGLE-arg fns (no multi-arg arrow needed).
fn fold<A, E, B>(r: Result<A, E>, on_ok: A -> B, on_err: E -> B) -> B =
  match r { Ok(x) => on_ok(x), Err(e) => on_err(e) }
```

## The swap (once RFC-0024 lands and is pulled back down into this branch)

1. Move each `fn` above into `result.myc`'s body as executable code.
2. Add a differential test per combinator in `crates/mycelium-l1/tests/std_result.rs`
   (three-way `L1-eval ≡ L0-interp ≡ AOT` on the monomorphized env; agreement `Empirical`).
3. Delete this file (or mark it implemented); flip the DN-14 HOF line to `present`.

## Honesty boundary (what stays deferred even after RFC-0024 v1)

- Closures/lambdas (env capture); a fn value flowing dynamically (out of a `match`, a
  data field, or a fn return); partial application; a still-generic fn passed as a value
  — all remain never-silent `Residual` (RFC-0024 §scope; full Reynolds defunctionalization
  is the deferred generalization).
- A *collection* `fold` over `(A, E) -> A` needs multi-arg arrows (RFC-0024 v2). The
  `Result.fold` above does **not** — it uses two single-arg fns.
