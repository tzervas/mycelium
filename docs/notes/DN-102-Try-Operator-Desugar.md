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
- **FLAG-try-5 — `Result`/`Option` dispatch is by type *name*, not structure (v0).** `check_try_let`
  selects the `Ok`/`Err` vs `Some`/`None` constructor set by matching the operand's type name literally
  (`Ty::Data("Result", [_, _])` / `Ty::Data("Option", [_])`), then builds those constructor patterns.
  A nodule that *shadows* the name with a differently-shaped type (e.g. `type Result[A, E] = Foo(A) |
  Bar(E)`) and applies `?` to it **fails closed** — a never-silent constructor-mismatch `CheckError`,
  never a silent mis-desugar (G2) — but with the generic "`Ok` is not a constructor of `Result`"
  surface rather than a targeted diagnostic. This is the accepted v0 restriction (`?` is sugar over the
  *standard* `Result`/`Option`); structural dispatch (or a pre-check of the declared constructors with a
  pointed diagnostic) is the follow-up if a port witnesses the shadowing need. (PR #1363 review.)

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

## Ratification / Maintainer decision (2026-07-11)

> **Not ratified — second research pass required.** Maintainer: *"many must be resolved via language
> gap closure; the flag try 2 issue presents an issue that must be resolved from that gap review pass
> and should have been raised as part of it. much of this may just need a second research pass to
> stage for ratification based on the data, project intent, and solid reasoned research."*

**Recorded decision (append-only — this note's original text above is unchanged; this section adds the
maintainer's decision, per house rule #3):**

1. **DN-102 status stays Draft.** Ratification of the desugar rule (§2), the error-type unification
   rule (§3), and the v0 `let`-RHS position restriction (§5) is **deferred**, pending a **second
   research pass** — one grounded in the DN-99 surface-gap-closure register's data, the project's stated
   intent (the zero-hand-port north star, ADR-045), and solid reasoned research, rather than ratifying
   the first-pass design as-is.
2. **FLAG-try-2 routed into language-gap closure — it should have surfaced in the DN-99 gap review.**
   §6's **FLAG-try-2** ("no `From`-error widening — the error-type unification rule is exact-match
   only; a widening `?` is refused until an error-conversion trait lands") is a genuine language-surface
   gap (no error-conversion trait wiring exists yet) that the maintainer judges should have been raised
   as part of the DN-99 sweep rather than surfacing only as a residual FLAG here. It is now cross-routed
   into **DN-99's §8 `enb` backlog** (a dated addendum on the **ENB-2** row records the routing — see
   `docs/notes/DN-99-Surface-Gap-Closure-Register.md` §8) so the gap is tracked in the register's own
   surface, not left orphaned in this note's residual list.
3. **Follow-up filed:** **M-1049** — "DN-102 second research pass + resolve FLAG-try-2 via language gap
   closure; re-stage for ratification" (`status:todo`, `doc_refs: corpus:DN-102, corpus:DN-99`,
   `tools/github/issues.yaml`).

## Second research pass (2026-07-11) — M-1049 (re-stage for ratification)

> **Append-only (house rule #3).** The original design record (§1–§7, §8 Grounding) is **unchanged**.
> This section is the **second research pass** the maintainer required before ratifying (see the
> "Ratification / Maintainer decision" note above; tracked as **M-1049**). It re-examines the `?`
> desugar in light of the now-**ratified** design principles (DN-106 sugar-transparency + gap-closure
> default + native-solution mapping; DN-109 L4 idiom framework) and the **adopted** DN-107 finding
> (general-`?` is a decoupled CPS lift, **no never-type**), resolves **FLAG-try-2** (`From`-widening),
> reconciles with the **landed** M-1025 let-RHS increment, and gives a **ranked ratification
> recommendation**. Status stays **Draft** — the maintainer re-ratifies (house rule #3/#4).
> Tags: `Empirical` where read against the tree at this branch's base (`origin/dev` tip `6e9869b6`);
> `Declared` for every design not yet ratified; nothing here is `Proven` (VR-5).

### SP.1 What the ratified frame changes (and what it does not)

The first pass predates four ratifications that reframe the whole question. Three **strengthen** the
original design; one **completes** an unresolved residual:

- **DN-106 (Accepted) — surface-sugar transparency + the gap-closure default.** A missing surface
  construct is closed, **by default**, by *building a convenience sugar that lowers mechanically and
  reliably to the existing core grammar* — never silence, never automatically a new kernel primitive.
  `?` is the archetype: it is exactly a mnemonic sugar over the already-present `Result`/`Option`
  `match` bind, hiding nothing (its lowering is revealable). So the *shape* of DN-102's answer — sugar
  over an existing `match`, **no new L0 node** (§4/§8) — is now the **default-endorsed** pattern, not a
  bespoke choice needing independent justification.
- **DN-109 (Accepted) — the L4 idiom framework.** Row **D2** classifies "`Option`/`Result`/`?` →
  never-silent `Option`/`Result` + explicit `match`" as **Mechanical** — the bucket that **auto-fires
  in v0** because it is statically decidable, semantics-preserving, and upgrades no guarantee tag
  (§3.2 ratchet). This places `?` correctly: it is a *mechanical* lowering, the safest idiom class, and
  the transpiler may emit it without a flag. The DN-102 desugar **is** the D2 lowering rule.
- **DN-107 (Accepted) — general-`?` is decoupled from the never-type.** DN-107 §6, **adopted** at
  ratification (its §6-a resolution), found that the general-position CPS lift
  `g(f()?)` ⇓ `match f() { Ok(x) => Ok(g(x)), Err(e) => Err(e) }` unifies both arms by **ordinary
  structural equality** (both `Result[B, E]`) — **no `⊥`, no divergence, no `-> !`**. This **removes
  the coupling** DN-102 §5/§6 (FLAG-try-1) and the M-1025 issue body asserted between general-`?` and
  the never-type (M-1030/#88). General-`?` is unblocked by *implementing the CPS lift*, which is real
  transpiler/checker work but **bottom-type-free**.
- **What does NOT change:** the §2 fork resolution (no sound *local* desugar of a bare `e?` — the
  continuation must live inside the binding arm) and the type-directed constructor dispatch (§4) are
  **untouched and correct**. The v0 let-RHS increment **landed** (M-1025 / PR #1363) and is
  differential-witnessed; nothing here reopens it.

### SP.2 FLAG-try-2 resolved — the `From`-error-widening gap (task 1)

**The question (mitigation #14 framing).** Rust's `?` applies `From::from` to the error, so
`let x = f()? in body` type-checks in a `Result[B, E2]`-returning function even when `f : Result[A, E1]`,
provided `E2: From<E1>` — the error is **silently converted** at the propagation point. DN-102 v0's
error-type unification rule (§3) is **exact-match only** (`E1 = E2`), so a widening `?` is refused.
Under the ratified principles, is this widening **(a)** a gap to close via a mechanically-lowering sugar,
**(b)** a native-solution mapping, or **(c)** a deliberate exclusion?

**The resolution: it is (c) *of the implicit form*, mapped to (b) a native solution, offered as (a) an
explicit-conversion sugar — in that layering.** Grounded:

1. **The *implicit, trait-dispatched* `From`-widening is a deliberate exclusion** — the **error-channel
   analogue of an inferred `swap`.** Rust resolves *which* conversion to apply by trait search
   (`From<E1> for E2`); the conversion is **invisible at the `?` site**. Inserting a conversion the
   source does not name is exactly the black-box / silent-semantic-change class that house rule #2
   (no black boxes), G2 (never-silent), and VR-5 forbid — and it is the **same family** DN-109 rules
   **Judgment, never auto-insert** for representation change: D13 (`as`/`.into()` → `swap`) — "S1
   forbids any inferred `swap` … a `swap` names a `policy:` Rust's `as` fixes silently." An
   auto-inserted `From::from` on the error channel picks a conversion function silently, the identical
   defect one channel over. So building a *trait-based `From` subsystem* to make `?` widen implicitly is
   **rejected** (KC-3/YAGNI + no-black-boxes) — this **corrects the DN-99 §8 ENB-2 routing note's
   wording**, which framed the sub-gap as "error-conversion-**trait** wiring for `?`-widening": the
   resolution is that Mycelium does **not** want trait-dispatched implicit widening at all (see the
   FLAG below routing this refinement into DN-99).
2. **The underlying *problem* — propagate a sub-error `E1` out of an `E2`-error function — maps to a
   Mycelium native solution: explicit conversion (`Empirical`).** Errors-as-values means an error
   conversion is just a function `conv : E1 => E2`, and the stdlib already carries the combinator:
   `fn map_err[A, E, F](r: Result[A, E], f: E => F) => Result[A, F]`
   (`lib/std/result.myc:39`; duplicated `lib/std/error.myc:100`). The porter widens **explicitly**:
   `let x = map_err(f(), e1_to_e2)? in body` — the exact-match rule (§3) then applies to the already-
   converted `Result[A, E2]`, and the conversion is **visible, named, never-silent** (G2). This is a
   **problem→native-solution mapping** (DN-106 principle 3a), not a dead-end refusal.
3. **Optionally, an explicit-conversion `?` *sugar* (a mechanically-lowering sugar, deferred).** Because
   the widening `?` lowering is mechanical **once the conversion is named** — the desugar's propagation
   arm becomes `Err($f) => Err(conv($f))` instead of `Err($f) => Err($f)` — a surface form that carries
   an **explicit** conversion (e.g. `e ?with conv` / `e ?map err_conv`; **surface unbid**, a DN-02/03
   lexicon choice) would lower reliably per the gap-closure default (DN-106 principle 2). It stays
   **out of v0 scope** (YAGNI — `map_err(…)?` already expresses it; add the sugar only when a port
   witnesses the ergonomic need), and it is emphatically **not** implicit `From` dispatch: the
   conversion is always spelled by the developer.
4. **Transpiler behaviour on a Rust `?` that relied on `From` (DN-106 principle 3a / DN-109 D6).** The
   transpiler cannot silently supply the conversion (it would be inventing the `From` impl's choice).
   The correct output is **flag-with-suggested-native-idiom**, not the v0 bare refusal: "this `?`
   relies on `From<E1> for E2`; supply the explicit conversion — `map_err(e, «E1→E2»)` before `?`, or
   the widening-`?` form" (the `suggested_idiom` field, M-1045). **Bare exact-match refusal (DN-102 v0)
   is thereby demoted to the last-resort form**; the principled behaviour points the porter at the
   named native solution.

**Net for the register (task 1 routing).** FLAG-try-2 is **resolved, not open**: the *implicit* widening
is a deliberate exclusion (mapped to `map_err`-explicit-conversion, optionally an explicit-conversion
sugar later); **no error-conversion-trait subsystem is needed or wanted.** This grounds and refines the
DN-99 §8 ENB-2 addendum (see FLAG-SP-1).

### SP.3 `?` as a sugar/macro — does principle 5 simplify the desugar? (task 2)

DN-106 principle 5 frames many sugars as macros (macro-expansion = the mechanical lowering + the
desugar/expand capability). Applying it to `?`:

- **It clarifies the *transparency/reversibility* axis — genuinely helpful.** `?` is a **revealable
  lowering**: `expand`/`EXPLAIN` shows the `match` it compiles to (DN-106 principle 1(b); the
  general desugar/expand tool is M-1051). Framing `?` as "a macro over the `Result`/`Option` `match`"
  is the accurate DN-106-sense description and makes DN-102's "no black box, no new L0 node" property a
  direct instance of the ratified sugar-transparency rule rather than a one-off argument.
- **It does NOT let `?` become a parse-time syntactic macro — an honest limit.** A `macro_rules!`-style
  macro expands **before** type-checking, purely syntactically. `?`'s constructor set (`Ok`/`Err` vs
  `Some`/`None`) is **type-directed** — it needs the operand's *checked* type (§4; and per FLAG-try-5
  the v0 dispatch is by type **name**). So `?` is a **type-directed / elaboration-time desugar in the
  checker** (`check_try_let`, `checkty.rs:4435`), **not** a textual pre-pass macro. Calling it a
  "macro" is correct in the DN-106 *mechanical-lowering-plus-expand* sense but must not mislead into
  "move it to parse time": the type-directed dispatch keeps it in the checker. (Making it purely
  syntactic would require a Rust-`Try`-trait-style `Carrier`/`branch` abstraction to dispatch on — trait
  machinery Mycelium does not have and KC-3/YAGNI reject; the type-directed checker desugar is correct.)
- **It unifies let-RHS and general-position as ONE sugar — the load-bearing simplification.** Under the
  sugar frame, `?` is a single lowering whose job is: *bind the success value and thread the rest of
  the computation as the success continuation.* The **v0 let-RHS case is the degenerate base case**
  where the continuation is already syntactically `body` (no transform needed). **General position is
  the same sugar** with one extra lowering step — a **CPS lift** that *recovers* the implicit
  continuation (the enclosing expression with a hole) and threads it into the `Ok`/`Some` arm, per
  DN-107's bottom-type-free form: `g(f()?)` ⇓ `match f() { Ok(x) => Ok(g(x)), Err(e) => Err(e) }`.
  Both cases lower to the **same** type-directed `match`; they differ only in *how much continuation-
  recovery the lowering performs.* So the v0 position restriction (§5) is **not a different design** —
  it is an **incremental scope of one sugar**, and lifting it is "add the CPS-lift lowering step,"
  independent of the never-type (DN-107).

**Verdict (task 2):** the macro/sugar frame **helps** — it (i) makes `?`'s transparency a ratified-rule
instance, (ii) unifies let-RHS + general-position as one sugar with a continuation-recovering lowering,
and (iii) positions `?` as the DN-109 **D2 Mechanical** idiom rule. It **does not** simplify the
*mechanism* (the dispatch stays type-directed in the checker) — reported honestly, not overclaimed.

### SP.4 Reconciliation with the landed M-1025 let-RHS desugar (`Empirical`)

Read against the tree: `Tok::Question` (`token.rs:389`), `Expr::Try(Box<Expr>)` (`ast.rs:837`),
`check_try_let` (`checkty.rs:4435`) desugaring `let x = e? in body` type-directed to the §2 `match`,
`Try` never surviving the checker (no new L0 node), and the general-position refusal
(`checkty.rs:4078` — "`?` … is only supported as a `let`-binder RHS in v0 … needs the general-position
CPS lift, which is deferred"), witnessed by `tests/try_operator.rs` (6 behavioural + 4 reject). **The
landed increment is exactly the base case of the unified sugar in SP.3** and needs **no revision**. The
only correction the second pass makes to its *framing* is the one DN-107 already applied append-only to
the M-1025 issue body: FLAG-try-1's general-`?` residual is **independent of the never-type** — a
CPS-lift follow-up, not a never-type-gated one. This note's SP.3 supplies the *why* (one sugar, CPS
recovers the continuation) that the issue-body correction asserts.

### SP.5 Ratification recommendation — RANKED (a Draft for the maintainer, not a ratification)

**Objective function** (ordered): **(1)** faithful to the ratified frame (DN-106 default / DN-109 D2 /
DN-107 decoupling); **(2)** KC-3 smallest kernel (no new L0 node, no trait subsystem); **(3)** G2/VR-5
never-silent + no upgraded tag; **(4)** KISS/YAGNI (land the witnessed subset; defer unwitnessed
surface); **(5)** append-only upgrade path preserved.

| Criterion | **Rec. — ratify v0 as landed + record the SP resolutions** | Alt X — build trait-`From` widening now | Alt Y — defer again (no ratify) |
|---|---|---|---|
| Faithful to ratified frame | ✅ D2 Mechanical sugar, exact-match core | ❌ implicit `From` = inferred-conversion black box (D13 family) | ◐ leaves a landed, witnessed increment unratified |
| KC-3 / blast radius | ✅ no new node, no new subsystem | ❌ a whole trait-dispatch subsystem | ✅ (nothing) |
| G2 / VR-5 | ✅ widening is explicit `map_err`; refusal never-silent | ❌ silent error conversion | ✅ |
| KISS / YAGNI | ✅ `map_err(…)?` covers widening today | ❌ builds unwitnessed machinery | ◐ |
| Upgrade path preserved | ✅ → explicit-conversion sugar / CPS lift, both additive | — | ✅ |

**Rank 1 (recommended) — ratify the v0 desugar as landed, and adopt the second-pass resolutions.** A
maintainer ratifying DN-102 → Accepted confirms:

1. **The core desugar (§2), the exact-match error-unification rule (§3), and the v0 let-RHS position
   restriction (§5)** — as landed and differential-witnessed (M-1025 / PR #1363), and now framed as
   the DN-109 **D2 Mechanical** idiom rule and a DN-106 revealable sugar (no new L0 node).
2. **FLAG-try-2 resolved (SP.2):** implicit trait-`From` widening is a **deliberate exclusion** (the
   inferred-conversion black-box family, D13); the native solution is **explicit `map_err(e, conv)?`**;
   an **explicit-conversion `?` sugar** is a deferred, additive follow-up (not implicit `From`); the
   transpiler **flags-with-suggested-idiom** (M-1045) rather than bare-refuses. **No error-conversion
   trait subsystem is adopted.**
3. **FLAG-try-1 reframed (SP.3):** general-position `?` is **one sugar with a CPS-lift lowering**,
   **decoupled from the never-type** (DN-107 §6-a, adopted). It is its own follow-up (tracked via
   M-1049 → a CPS-lift increment), gating on nothing in M-1030.
4. **The `?`-as-macro framing (SP.3)** is accepted as the *transparency/reversibility* description
   (revealable via M-1051 expand/EXPLAIN), with the honest caveat that the dispatch stays type-directed
   in the checker (not a parse-time syntactic macro).

**Rank 2 — Alt Y (defer again).** Only if the maintainer wants the explicit-conversion-sugar **surface
token** bid (SP.2 item 3) *before* ratifying — but that is an additive follow-up, not a reason to hold
the sound, landed core. Not recommended.

**Rejected — Alt X (trait-`From` widening).** Reintroduces the inferred-conversion black box (house
rule #2, VR-5, DN-109 D13); KC-3/YAGNI. Recorded, not silently dropped (house rule #4).

### SP.6 Adversarial stress-test (argue against the recommendation — VR-5/house rule #4)

1. **"Deferring implicit widening will hurt the port-idiom faithfulness — Rust `?` widens constantly."**
   Real concern. Rebuttal: the widening is **preserved, made explicit** (`map_err(e, conv)?`), and the
   transpiler **names the required conversion** in the flag (M-1045), so the porter loses no capability —
   only the *invisibility*. That is the intended trade under G2 (the same trade S1 makes for `swap`).
   The *ergonomic* cost (an extra `map_err`) is the driver that would later justify the explicit-
   conversion sugar (SP.2 item 3) — witnessed-need, not speculative.
2. **"`map_err` is HOF — does it even run in `.myc`?"** Checked: `map`/`and_then`/`map_err` are executable
   via RFC-0024 static defunctionalization (M-685/686/687), three-way witnessed (M-688) — per
   `lib/std/result.myc:7` `@summary`. So the native solution is **`Empirical`-executable**, not a paper
   combinator. (Its *agreement* is `Empirical`; the type-level contract is `Declared` — unchanged.)
3. **"Ratifying a `let`-RHS-only `?` blesses a half-operator."** It blesses the **witnessed base case of
   one sugar** (SP.3), with the general case explicitly scoped as an additive CPS-lift follow-up — the
   KISS/YAGNI increment the project uses everywhere (`Wrapping` precedent). Not a different design later;
   the same sugar, more continuation-recovery.
4. **"Type-name dispatch (FLAG-try-5) is a latent unsoundness."** It **fails closed** — a shadowing type
   yields a never-silent constructor-mismatch `CheckError` (G2), never a mis-desugar. Accepted v0
   restriction; structural dispatch is the witnessed-need follow-up. Unchanged by this pass.
5. **"Is the never-type *really* not needed for the fully-general CPS lift?"** DN-107 §6 is **confident
   (`Empirical`) for the simple lift** and **honest (`Declared`) that no corner of a fully-general lift
   over every expression shape is proven bottom-free.** This pass inherits that exact tag: general-`?`
   is *very likely* bottom-free, and the CPS-lift follow-up carries the obligation to witness it (or
   surface the one shape that needs divergence — which would then reuse DN-107's `diverges` effect, not
   a new bottom type). Not overclaimed.

### SP.7 Definition of Done — what re-ratification requires (this pass)

- [ ] Maintainer confirms **Rank 1**: ratify the landed v0 desugar (§2/§3/§5), adopt the SP.2
      FLAG-try-2 resolution (deliberate-exclusion-of-implicit + `map_err` native mapping + deferred
      explicit-conversion sugar + flag-with-suggested-idiom; **no trait subsystem**), and the SP.3
      reframe (one sugar; general-`?` = CPS lift, never-type-decoupled).
- [ ] The DN-99 §8 ENB-2 routing note's wording is refined per **FLAG-SP-1** (trait-wiring → native
      explicit-conversion; the implicit form is excluded, not pending).
- [ ] On ratification, DN-102 moves **Draft → Accepted**; the general-`?` CPS-lift follow-up and the
      optional explicit-conversion-sugar surface stay tracked (M-1049 → a CPS increment), gating on
      nothing in M-1030. The desugar's *agreement* stays `Empirical` (the landed differential); no tag
      is upgraded past its basis (VR-5) — nothing here is `Proven`.

### SP.8 FLAGs (shared-file rows — FLAGGED up, not applied here)

- **FLAG-SP-1 — DN-99 §8 ENB-2 routing refinement.** Refine the existing (maintainer-added) ENB-2
  addendum: the FLAG-try-2 sub-gap is **resolved as a deliberate exclusion of *implicit* `From`
  widening** (native solution: explicit `map_err`-conversion; optional explicit-conversion sugar
  later), **not** "error-conversion-trait wiring … pending." A short append-only dated addendum to
  DN-99 §8 records this (added in this branch — see DN-99 changelog); the maintainer confirms the
  wording on ratification.
- **FLAG-SP-2 — `issues.yaml` (integration-owned).** On ratification: **M-1049** → record the second
  pass complete + the resolutions; move general-`?` to a **new CPS-lift follow-up issue** (decoupled
  from M-1030, cross-ref DN-107 §6-a); optionally file the explicit-conversion-`?`-sugar surface as a
  YAGNI-deferred idea. **M-1025** (`in-progress`): unchanged by this pass (already carries the DN-107
  progress note). Optionally add `corpus:DN-106`, `corpus:DN-109` to M-1049 `doc_refs`.
- **FLAG-SP-3 — `CHANGELOG.md` + `docs/Doc-Index.md` (integration-owned).** Add a CHANGELOG entry for
  the DN-102 second research pass (+ the DN-99 §8 refinement addendum); no Doc-Index row change (DN-102
  and DN-99 are already registered).

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

- **2026-07-11** — **Second research pass added (M-1049) — re-staged for ratification** (append-only;
  the original §1–§7 design record is unchanged). Grounds the `?` desugar in the now-ratified frame:
  DN-106 (sugar-transparency + the gap-closure default — `?` is the archetypal mechanically-lowering
  sugar), DN-109 (the L4 **D2 Mechanical** idiom rule), and DN-107 §6-a (adopted: general-`?` is a
  **CPS lift decoupled from the never-type**). **Resolves FLAG-try-2:** implicit trait-`From` widening
  is a **deliberate exclusion** (the inferred-conversion black-box family, DN-109 D13), mapped to the
  native solution **explicit `map_err(e, conv)?`** (`lib/std/result.myc:39`), with an explicit-
  conversion `?` sugar as a deferred additive follow-up and **no error-conversion-trait subsystem** —
  refining the DN-99 §8 ENB-2 routing (FLAG-SP-1). Finds the **`?`-as-macro** framing helps the
  transparency/reversibility axis and unifies let-RHS + general-position as **one sugar** (the CPS lift
  recovers the continuation), while honestly noting the dispatch stays type-directed in the checker (not
  a parse-time syntactic macro). Reconciles with the **landed** M-1025 let-RHS increment (no revision).
  **Ranked recommendation: ratify the landed v0 core + adopt the SP resolutions** (Rank 1). Status stays
  **Draft** — the maintainer re-ratifies (house rule #3/#4). All new claims `Declared`/`Empirical` per
  basis; nothing `Proven` (VR-5).
- **2026-07-11** — **Maintainer decision: NOT ratified — a second research pass is required** (house
  rule #3). Status stays **Draft**. FLAG-try-2 (`From`-error widening) cross-routed into the DN-99
  gap-closure register's §8 `enb` backlog (ENB-2 row addendum). Follow-up filed as **M-1049**.
  Append-only — the original design record above is unchanged; this is an added decision note.
- **2026-07-10** — DN-102 created as **Draft** (M-1025 / ENB-2). Records the `?` surface + the
  type-directed `match` desugar, the fork resolution (no sound local desugar without a never-type — §2),
  the error-type unification rule (§3), and the v0 `let`-RHS position restriction (§5). Authored with the
  first landable increment; enacts nothing, moves no other doc's status (append-only, house rule #3).
