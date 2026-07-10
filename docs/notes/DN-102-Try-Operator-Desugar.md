# Design Note DN-102 — The `?` Try-Operator Desugar (the ENB-2 grammar-sugar close)

| Field | Value |
|---|---|
| **Note** | DN-102 |
| **Status** | **Draft** (2026-07-10). Authored alongside the **first landable increment** of M-1025 (ENB-2). It records the design of the `?` try-operator **surface + desugar**, recommends a desugar rule and a position restriction **for ratification**, and **enacts nothing** and **moves no other doc's status** (house rule #3, append-only). Tags are `Empirical` where read against the code / witnessed by a running differential, `Declared` for any design not yet ratified (VR-5). |
| **Decides** | *Proposes, for ratification:* (1) a `Question` token (`?`) lexed as one atomic glyph; (2) `expr?` parsed as a **postfix marker** wrapping its operand in `Expr::Try`; (3) the desugar of `let x = e? in body` to a **type-directed `match`** over the operand's `Result`/`Option` type — the continuation `body` lives inside the binding arm, so the desugar type-checks **without** an early-return or a never-type (DN-99 #88/`->!` stays deferred); (4) the **error-type unification rule** (the `?`-operand's error/absence channel must match the enclosing function's return channel); (5) the **v0 position restriction**: `?` is legal **only** as a `let`-binder RHS — a `?` anywhere else is a **never-silent refusal** pointing at the deferred CPS-lift follow-up. It does **not** edit `issues.yaml`, `CHANGELOG.md`, or `Doc-Index.md` (the integrating session owns those). |
| **Feeds** | DN-99 §A1 / register rows #60 (`?` grammar sugar) + #52 (the try-operator idiom facet), ENB-2; M-1025; DN-26 (SCC self-hosting, the Rust↔`.myc` dual); DN-34 §8 (surface-gap census). |
| **Grounds on** | KC-3 (small kernel, no new L0 node — desugars to the existing `Match`), DRY (reuse `Result`/`Option` + the existing `match` machinery), G2 (never-silent — a refused position/type prints the fix), VR-5 (no tag upgraded past its basis), KISS/YAGNI (the `let`-RHS subset over a full CPS lift). |
| **Date** | July 10, 2026 |
| **Task** | M-1025 (ENB-2) — `?` try-operator grammar sugar + desugar. |

> **Grounding + honesty (house rule #4 / VR-5 / G2).** This note records a design and a running
> increment; it does **not** take a decision (house rule #3 — the maintainer ratifies). Empirical claims
> are witnessed by the differential/conformance witnesses named in §7 (running against a hand-written
> `match` oracle). The desugar **rule** and the **position restriction** are `Declared` until ratified.
> **No sycophancy:** §3 confronts head-on that the "obvious" local-`match`-with-early-return desugar is
> **ill-typed** in this language, and §6 states the residual (Option-position parity beyond the
> `let`-RHS, the CPS lift) plainly rather than claiming a whole `?` operator.

---

## §1 Purpose

Close the `?` try-operator surface gap (DN-99 register rows #60 + #52, ENB-2). A stdlib/semcore porter
writing an error-propagating Rust function threads `?` through a chain of fallible steps
(`let x = f()?; let y = g(x)?; Ok(h(y))`). Today the `.myc` frontend has **no `?`** (verified: zero
`?`/`Question` tokens in `token.rs`/`lexer.rs`), so every propagation must be hand-desugared to a nested
`match` or an `and_then` chain (see the hand-written combinators in `lib/std/result.myc`,
`lib/std/option.myc`, `lib/std/error.myc`). This note adds `?` as **surface + lowering over the existing
`Result`/`Option` runtime semantics** — **no new kernel semantics, no new L0 node** (KC-3).

## §2 The design fork — why the naive desugar is ill-typed here

Rust's `?` desugars (morally) to:

```
match e { Ok(x) => x, Err(err) => return Err(err) }
```

That `return` is an **early return** from the enclosing function. Mycelium is a **pure expression
language with no `return` statement** (iterate-by-recursion; `while`/`break`/`return` are non-forms —
`parse_unary`/`teach_imperative` reject them) **and no bottom/never type** (`-> !` is explicitly deferred
— DN-99 §A7, register row #88). So the naive **local** desugar is *doubly* unavailable:

- Without early return, `Err(err) => Err(err)` is a **branch value**, not a return — so it only
  propagates if the `match` is already in tail position.
- Even in tail position the two arms **do not unify**: `Ok(x) => x` has type `A`, but
  `Err(err) => Err(err)` has type `Result[_, E]`. A normal `match` requires both arms to share a type;
  `A ≠ Result[A, E]`. In Rust this is bridged by the `Err` arm **diverging** (the never-type `!` unifies
  with everything). Mycelium has no `!` (#88), so **there is no sound local-`match` desugar of a bare
  `e?`.**

**The resolution (the sound form).** The desugar must place the **continuation** inside the *binding*
arm, so the propagation arm yields the same type as the continuation:

```
let x = e? in body
  ⇓  (Result)
match e { Ok(x) => body, Err($f) => Err($f) }
```

Now **both arms have type `typeof(body)`**: `Ok(x) => body` is `typeof(body)`, and — because the
enclosing function returns `Result[B, E]` and `body` is in tail position — `body : Result[B, E]`, so the
`Err($f) => Err($f)` arm (`Result[B, E]`) **unifies** with it. **No never-type, no early return, KC-3.**
This *is* the monadic bind (`and_then(e, λx. body)`) written inline as a `match`, avoiding the HOF/
overload/`Unit`-domain wrinkles the hand-written `and_then` combinators carry (`lib/std/error.myc`
substitution notes). The `Option` analog:

```
let x = e? in body
  ⇓  (Option)
match e { Some(x) => body, None => None }
```

The choice of constructor set (`Ok`/`Err` vs `Some`/`None`) is **type-directed** — it needs the operand's
checked type — which is why the surface carries a small `Expr::Try` marker node through to the
type-aware layers rather than desugaring blindly at parse time (§4).

## §3 The error-type unification rule (pinned)

`let x = e? in body` where `e : Result[A, E]` requires the enclosing function to return `Result[B, E]`
for the **same `E`** — the desugar's `Err($f) => Err($f)` arm constructs a `Result[B, E]` from an `E`, so
`typeof(body) = Result[B, E]` forces `E` to match. The `Option` case requires the function to return
`Option[B]` (the absence channel is unparameterized). This is **enforced structurally by the desugar**
(the built `match` is checked by the ordinary exhaustive-`match` checker — Maranget usefulness, W7): a
mismatched error type, or a `?` in a function whose return type is neither `Result[_, E]` nor `Option[_]`,
is a **never-silent `CheckError`** (G2), never a silent coercion. No `From`/`Into` error-widening is in
scope for v0 (Rust's `?` calls `From::from` on the error — Mycelium has no such trait wiring yet; a
widening `?` is a FLAG, §6).

## §4 The surface + the lowering path (KC-3, DN-26 dual)

| Stage | Rust frontend (`crates/mycelium-l1/src`) | Self-hosted mirror (`lib/compiler/*.myc`) |
|---|---|---|
| **Lex** | `?` → `Tok::Question` (a single-char glyph; `single(Tok::Question)` in `lexer.rs`). No multi-char form (`??`/`?.` are not glyphs). | `token.myc`/`lex.myc`: the `Question` token + its lex arm. |
| **AST** | `Expr::Try(Box<Expr>)` — a one-operand marker (shape of `Consume`/`Wrapping`). | `ast.myc`: the `Try` variant + every `classify_expr` copy. |
| **Parse** | `parse_app`: a trailing `?` after the primary/call/ascription chain wraps the expression in `Expr::Try`. | `parse.myc`: the same postfix. |
| **Check** | `Expr::Let{ bound: Try(e), body }` → infer `e`'s `Result`/`Option` type, build the §2 `match`, check it (error-type unification falls out). A `Try` in any **other** position → never-silent `CheckError` "`?` is only supported as a `let`-binder RHS in v0 (DN-102 §6; the general-position CPS lift is deferred)". | `checkty.myc` / `semcore.myc` mirror. |
| **Elab (→L0)** | `Let{ bound: Try(e), body }` → re-infer the operand type (the elaborator already re-infers a scrutinee/bound type), build the §2 `match`, elaborate it. No new L0 node. | (elab mirror as the port reaches it.) |
| **Eval (L1 oracle)** | `Let{ bound: Try(e), body }` → evaluate `e`; dispatch on the **runtime** constructor (`Ok`/`Some` → bind `x`, run `body`; `Err`/`None` → yield it). | (eval mirror as the port reaches it.) |

**Structural walkers** (`totality`, `ambient`, `mono` free-vars/subst, `grade`, the `print_expr`
pretty-printer) treat `Try(b)` **transparently** — recurse into `b`, exactly as they treat `Consume(b)`/
`Wrapping(b)` — since a well-formed `Try` is always consumed at its enclosing `Let`. Every `classify_expr`
copy (`parse.rs`, `totality.rs`, the test prelude in `src/tests/*`, plus `ast.myc`/`parse.myc`/
`totality.myc`) gets a `Try` arm — **the missed-copy `Wrapping` red is the cautionary precedent**
(CLAUDE.md swarm-lesson): the sweep is driven by `cargo build` exhaustiveness, not by memory.

## §5 The position restriction (v0) — why `let`-RHS only

`?` binds the **rest of the computation** as its success continuation. In `let x = e? in body`, the
continuation is exactly `body` — statically visible, no transformation needed. A `?` in a **non-let**
position (`g(f()?)`, `f()?.method`) has an *implicit* continuation ("the enclosing expression with a
hole"), which requires a **CPS lift** of that enclosing expression to recover. The CPS lift is a real,
larger transformation (it must thread the continuation through arbitrary expression shapes) — out of
scope for this landable increment (YAGNI until a port witnesses the need). So v0:

- **Legal:** `let x = e? in body` (and, degenerate, `let x = e? in <tail>` chains — each `?` its own
  `let`).
- **Refused, never silently (G2):** `?` in any other position → a `CheckError` naming the restriction and
  pointing here. This is a *refusal*, not a mis-desugar — the porter sees exactly why and the one-line
  rewrite (`let tmp = inner? in outer(tmp)`).

This subset covers the **dominant** Rust port shape (`let x = f()?;` is a `let`-statement) and is the
KISS increment; §6 tracks the general position.

## §6 Residual / FLAGs (never claimed as done — VR-5/G2)

- **FLAG-try-1 — general `?` position (CPS lift).** `?` outside a `let`-RHS is refused, not lowered.
  The CPS-lift that would generalize it is deferred to a follow-up (an ENB continuation or a checker/
  desugar wave). Tracked as the residual of M-1025.
- **FLAG-try-2 — `From`-error widening.** Rust's `?` applies `From::from` to the error, letting a `?`
  cross error types. Mycelium has no error-conversion trait wiring in v0, so the error-type unification
  rule (§3) is **exact-match only** — a widening `?` is refused. Revisit when/if an error-conversion
  trait lands.
- **FLAG-try-3 — `.myc` elab/eval mirror.** The Rust frontend is the trusted base and lands elab+eval
  here; the self-hosted `lib/compiler/*.myc` mirror lands the **lex + AST + parse + check** surface in
  step with the port's current frontier (DN-26). The `.myc` elab/eval mirror follows the port's general
  cadence, not this increment.
- **FLAG-try-4 — surface honesty tag.** The `?` desugar is `Declared` (a type-level surface contract)
  until the differential in §7 witnesses `?` ≡ the hand-`match` oracle on both paths, which upgrades the
  *agreement* claim to `Empirical` (never the desugar rule itself past its basis — VR-5).

## §7 Definition of Done (this note's gate)

- [ ] A `Question` token lexes `?` (Rust + `.myc`), with a reject witness for a stray `?` where no
      operand precedes it.
- [ ] `let x = e? in body` desugars per §2 and **type-checks** via the error-type unification rule (§3);
      a `?` in a non-`let`-RHS position and a `?` in a non-`Result`/`Option`-returning function are both
      **never-silent refusals** (conformance **reject** witnesses).
- [ ] A **differential** witnesses `?` ≡ the explicit hand-`match` desugar on **both** the success and
      the propagation path, for **`Result`** and **`Option`** (the Rust oracle vs the `.myc` frontend,
      `myc check` + `cargo test -p mycelium-l1`).
- [ ] This note records the desugar, the fork resolution (§2), the unification rule (§3), and the v0
      restriction (§5); the maintainer ratifies (house rule #3).

## §8 Grounding

- **KC-3 / no new kernel node:** the desugar targets the existing `Expr::Match` + `Result`/`Option` data
  types; the L0 Core IR is unchanged (the same discipline as `TupleLit`, object-`impl`, `colony`).
- **DRY:** reuses the exhaustive-`match` checker (Maranget usefulness) and the existing `Result`/`Option`
  constructor machinery — the desugar is one built `Expr::Match`, checked/elaborated/evaluated by the
  ordinary paths.
- **G2 (never-silent):** every refused position/type prints the reason + the rewrite; no `?` silently
  drops an `Err`/`None` or coerces an error type.
- **VR-5 (no upgraded tag):** the desugar rule is `Declared` until the §7 differential upgrades the
  *agreement* claim to `Empirical`; the `->!`/never-type path (#88) stays deferred rather than faked.
- **DN-99 §A1 / rows #60 + #52:** this is the ratified-backlog close those rows point to.

---

## Changelog

- **2026-07-10** — DN-102 created as **Draft** (M-1025 / ENB-2). Records the `?` surface + the
  type-directed `match` desugar, the fork resolution (no sound local desugar without a never-type — §2),
  the error-type unification rule (§3), and the v0 `let`-RHS position restriction (§5). Authored with the
  first landable increment; enacts nothing, moves no other doc's status (append-only, house rule #3).
