# RFC-0040 — `Vec` List-Literal Desugaring (`[…]` for cons-list ADTs)

| Field | Value |
|---|---|
| **RFC** | 0040 |
| **Status** | **Accepted** (2026-07-03) — *Proposed → Accepted: Rust-first implemented and green in `crates/mycelium-l1` (checkty desugaring + `lib/std` `[…]` adoption), the full `mycelium-l1` + `mycelium-std-conformance` + `mycelium-std-*` suites green (1854 tests, behaviour-neutrality by AST identity — see §5). Ratification `Accepted → Enacted` is the maintainer's (VR-5: implemented, not yet ratified). Minted under the maintainer's explicit 2026-07-03 directive to "pursue the list-literal source change now" (resolving DN-82 FLAG-976-1). **Ratified `Proposed → Accepted` by the maintainer 2026-07-03** — the "accept the default recommendations" directive (the type-directed list literal was the accepted default) IS the ratification act, confirmed explicitly at the terminal PR review (RFC-0039 Proposed→Accepted precedent). `Enacted` remains the maintainer's later call.* |
| **Type** | Normative — surface **semantics** extension (type-directed elaboration); **no grammar/parser/L0 change** |
| **Date** | 2026-07-03 |
| **Task** | M-977 |
| **Feeds** | DN-82 §7.3–7.4 (resolves FLAG-976-1 — the each-item-closed list ideal); the Shape-Dispatched Readable renderer (M-976) which already lays a `[…]` one element per line |
| **Decides** | That a `[e1, …, en]` literal checked against a **cons-list-shaped** user ADT desugars to the right-nested `Cons(e1, Cons(…, Nil))` chain; the *structural* recogniser for "cons-list-shaped"; and that this composes with the existing RFC-0032 D3 `Seq{T,N}` literal without ambiguity |
| **Depends on** | RFC-0032 D3 (the `[…]` `Seq{T,N}` literal — this RFC adds a *disjoint* type-directed reading, leaving Seq untouched); RFC-0007 §11 (the checker produces the elaborated body the interpreter/AOT consume); RFC-0037 §4.3 (D3 — the list/`Seq` bracket literal) + §4.1 (D1 — the type-arg bracket family) |
| **Coupled with** | `crates/mycelium-l1/src/checkty.rs` (`check_list` + `cons_list_ctors`); `lib/std/*.myc` (the `matrix()`/`guarantee_matrix()`/`modes_all()` tables adopt `[…]`); `crates/mycelium-fmt` (renders `[…]` readably — unchanged) |

> **Posture (transparency rule / VR-5 / G2).** The desugaring is **behaviour-neutral by construction**:
> a `[…]` in a cons-list context elaborates to the **byte-for-byte identical AST** as the hand-written
> `Cons` chain (§5), so every downstream path (L1-eval, elaborate→L0-interp, AOT) is unchanged. The
> behaviour-neutrality is `Empirical` — AST-identity unit tests + the `mycelium-std-conformance`
> three-way differential — **not** `Proven`. The recogniser is a `Declared` structural heuristic; it
> never *silently* reinterprets a Seq literal or a non-list ADT (both are disjoint, both refused/kept
> explicitly — G2).

---

## 1. Problem / Goal

Mycelium's user list types are ordinary recursive ADTs — `type Vec[A] = Nil | Cons(A, Vec[A])`
(and its siblings `Trits`/`TNil`/`TCons`, `ByteList`/`BNil`/`BCons`, `GRowList`/`GLNil`/`GLCons`, …).
A literal table therefore had to be written as a **right-nested `Cons` chain**:

```mycelium
fn matrix() => Vec[GuaranteeRow] =
  Cons(row_value_repr_meta(), Cons(row_corevalue_datum(), … Cons(row_provenance_of(), Nil)…));
```

DN-82's Shape-Dispatched Readable renderer (M-976) collapses that pyramid to a flat spine, but a
**residual coalesced closer run** (`Nil))))))))));`) is irreducible with `Cons` tokens: the reader
still counts closers to confirm balance. DN-82 §7.3 grounded that the each-item-fully-closed ideal
(`[row_a(), row_b(), …];`) is **not reachable by whitespace alone**, flagged the fix as FLAG-976-1
(this RFC), and grounded that a *naïve* rewrite to the RFC-0032 `Seq{T,N}` literal is
**behaviour-CHANGING** (it types to `Seq{T,N}` ≠ `Vec[T]`, and the evaluator refuses data-constructor
elements). The honest fix is a **type-directed desugaring**, decided here.

## 2. User stories

- *As a stdlib author, I want to write a fixed list table as `[a, b, c]` — each item closed on its own
  line, comma-separated, one terminal `;`, no closer run — so that appending a row is a one-line,
  clean-diff edit and the table reads as the flat sequence it is.*
- *As a reader/reviewer, I want a flat list to look flat (no pyramid, no `)` wall) so that I never
  count delimiters to confirm well-formedness.*
- *As a kernel maintainer, I want the new surface to change **zero** runtime behaviour (same AST, same
  lowering, same guarantee tags) so that adopting it in `lib/std` cannot perturb any conformance
  obligation.*

## 3. Decision

**A `[e1, …, en]` literal whose *expected* type is a cons-list-shaped user ADT desugars to
`Cons(e1, Cons(…, Cons(en, Nil)…))` and is checked as that chain.** Precisely, in
`checkty::check_list`, before the `Seq{T,N}` path:

1. If there is an expected type `T` and `cons_list_ctors(types, T)` yields `(nil, cons)` (§3.1),
   build the right-nested chain `Expr::App{cons, [e_i, acc]}` (folding from the right, `acc` seeded to
   `Expr::Path(nil)`) and **re-check it** against `T`. Re-checking (rather than hand-building the
   typed form) means the checker's ordinary constructor-call path produces the elaborated body, so it
   is *identical* to the hand-written chain.
2. Otherwise the existing behaviour is **unchanged**: an expected `Seq{T,N}` (or no expected type)
   takes the RFC-0032 D3 Seq path exactly as before.

`[]` in a cons-list context desugars to the bare `nil` constructor (the context supplies the element
type — no guess). An over-/under-length concern does not arise (a `Vec` is unbounded, unlike `Seq{T,N}`).

### 3.1 The structural recogniser (`cons_list_ctors`)

"Cons-list-shaped" is recognised **structurally — no annotation, no privileged names**: the expected
type is a `Ty::Data(name, _)` whose declaration has **exactly two** constructors —

- one **nullary** constructor (the *nil*: `Nil`/`GLNil`/`TNil`/`BNil`/…), and
- one **binary** constructor whose **second field is the recursive `Self` type** `Data(name, …)` at the
  type's own parameter arity (the *cons*: `Cons(A, Self)`).

This matches every `lib/std` list type uniformly. Any other shape (a different ctor count, a 2-ctor
type without the nullary+recursive pair — e.g. `Pair[A] = MkA(A) | MkB(A, A)`) yields `None`, so the
`[…]` is **not** reinterpreted: it falls through to the Seq/no-context path (refused where a Seq is not
expected). There is no ambiguity with the Seq literal because `Seq{T,N}` and a user ADT are disjoint
expected types.

## 4. What this does NOT do (scope fence)

- **No grammar/parser/lexer change.** `[…]` already parses to `Literal::List`; this RFC only changes
  its *type-directed elaboration*. RFC-0037's bracket allocation is untouched.
- **No L0 / lowering change (KC-3).** The desugared AST is the current `Cons` chain, so L1→L0 lowering,
  monomorphization, and the interpreter/AOT surfaces are byte-identical.
- **No variadic fold.** The `bool_and`/`cat` *pyramids* (only binary, right-nested) are **not** list
  literals; DN-82 FLAG-976-2 (a variadic `all_of([…])`/`concat([…])`) remains a separate, deferred RFC.

## 5. Definition of Done — met (Rust-first)

- **Desugaring implemented** in `checkty.rs` (`check_list` early-return + the `cons_list_ctors`
  structural recogniser); `mycelium-l1` builds `-D warnings` clean. ✔
- **Behaviour-neutrality proven `Empirical` by AST identity** (the strongest available, stronger than
  eval-equivalence): `crates/mycelium-l1/tests/list_literal.rs` asserts the checked/elaborated fn body
  of a `[…]` program is **byte-for-byte equal** to the hand-written `Cons`-chain program; `[]`→nil; a
  `Seq` literal is unchanged; a non-list 2-ctor ADT is refused. ✔
- **`lib/std` adopts `[…]`** for the static tables (`matrix()` in core/diag/spore/swap/recover/testing,
  `guarantee_matrix()` in select, `modes_all()` in testing) — the each-item-closed / single-`;` ideal,
  no closer run. ✔
- **The full behaviour suite is green after adoption**: `mycelium-l1` + `mycelium-std-conformance`
  (L1-eval ≡ elaborate→L0-interp ≡ AOT ≡ the Rust reference checkers) + every touched `mycelium-std-*`
  eval crate — **1854 tests, zero failures**. ✔
- **`Empirical`, not `Proven`** (VR-5): trials evidence (AST-identity + the differential suites), not a
  theorem. ✔

## 6. Open / out of scope (FLAGs — VR-5)

- **FLAG-0040-1 (recogniser scope).** The structural recogniser accepts *any* nullary-nil + recursive-
  `Cons(A, Self)` 2-ctor ADT. This is intentional (it generalises to every list type without
  annotation) and benign (it desugars to *that* type's own ctors). A future tightening (e.g. an
  opt-in `#[list]` marker) is possible but not needed; flagged, not claimed.
- **FLAG-0040-2 (variadic folds).** DN-82 FLAG-976-2 (`all_of`/`concat`) is the analogous fix for the
  `bool_and`/`cat` pyramids and remains a separate RFC — this RFC does not address non-list folds.
