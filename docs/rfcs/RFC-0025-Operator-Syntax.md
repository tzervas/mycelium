# RFC-0025 — Operator Syntax: Symbolic Infix Sugar & Precedence Table

| Field | Value |
|---|---|
| **RFC** | 0025 |
| **Status** | **Proposed** (2026-06-23) — adopts DN-23's **hybrid** recommendation: a frontend-only infix/prefix sugar layer desugaring to canonical word functions, no L0/L1 kernel change (KC-3). The binding precedence/associativity table (§4.1), desugaring map (§4.2), and grammar extension are now normative **as proposed** (maintainer ratifies → Accepted; house rule #3 — never skipping steps). **implemented (Rust-first), pending ratification** (2026-06-23, M-705 — `crates/mycelium-l1` lexer+parser desugaring + `docs/spec/grammar/mycelium.ebnf` operator grammar + `accept/20-operator-syntax.myc`; sugar↔word agreement across L1-eval ≡ L0-interp ≡ AOT is **Empirical**). |
| **Type** | Surface / normative (once Accepted) — frontend-only; no L0 or L1 kernel change |
| **Date** | 2026-06-23 |
| **Feeds** | E7-5 (operator-syntax leg); E11-1 (surface-language completeness) |
| **Decides** | The binding decision DN-23 deferred: whether to add symbolic infix sugar (`a + b → add(a, b)`), and if so, the complete precedence/associativity table and the sugar grammar. DN-23's recommendation (hybrid: word kernel + optional symbol sugar as frontend-only desugaring) is the starting point. |
| **Depends on** | RFC-0006 §3/§4.3 (grammar discipline — S1–S6, machine-readable EBNF); RFC-0007 §4.1–§4.8 (L1 kernel calculus — the desugaring target); RFC-0024 (HOF via static defunctionalization — word operators become first-class values once HOF lands, which strengthens the word-canonical argument); DN-23 (design space + recommendation; source of truth for the three options); `docs/spec/grammar/mycelium.ebnf` (the EBNF to be extended or left unchanged) |
| **Task** | E7-5 (M-705) |

> **Posture (honesty rule / VR-5).** This RFC is **Proposed**: it makes the binding decisions DN-23
> deferred (§4 — the precedence table, the desugaring map, the grammar extension) and is implemented
> Rust-first in `mycelium-l1` (M-705). The sugar is **frontend-only** — a pure syntactic rewrite to
> the canonical word call, with **no L0 or L1 kernel change** (KC-3). No guarantee tag is upgraded:
> the desugaring is a structural transform (no theorem); the three-path sugar↔word agreement is
> **Empirical** (differential trials). The decisions are normative **as proposed** and become binding
> on maintainer ratification (Proposed → Accepted; house rule #3).

---

## 1. Problem / Goal

Mycelium's surface is word-based for operations: `add(a, b)`, `mul(x, y)`, `xor(acc, b)`. This
is consistent, greppable, and first-class for HOF once RFC-0024 lands. It is also verbose for
math-heavy code — `add(mul(a, b), c)` where a human would write `a * b + c`.

DN-23 surveyed three options (words only, symbols only, hybrid) and recommended the **hybrid**:
keep words canonical in the kernel, add optional infix symbol sugar at the surface that desugars
to word functions. The desugaring is a **frontend-only** pass — no `mycelium-core` or L1 change,
exactly the discipline used for monomorphization and HOF defunctionalization (KC-3).

DN-23 explicitly deferred the binding decision to this RFC because the sugar layer still requires:
(a) a normative **precedence / associativity table** (a real spec and parser cost, as DN-23 notes);
(b) a concrete **sugar grammar** extension to `mycelium.ebnf`; (c) a decision about **which
operations get sugar** (arithmetic only? bitwise? comparison? user-defined?); and (d) a decision
about whether the word form remains valid everywhere the sugar appears (it must — the words are
canonical). These choices belong in a binding RFC, not an advisory note.

---

## 2. User stories / motivating use cases

- As a **language user** writing a numeric algorithm, I want to write `a * b + c` instead of
  `add(mul(a, b), c)`, so that my code reads like the mathematics it encodes and cognitive overhead
  does not dominate reasoning about correctness.
- As a **stdlib author** implementing `std.math` or `std.numerics` combinators, I want to pass
  `add` and `mul` as first-class function values to `fold`/`map` (RFC-0024), so that I can build
  generic algebraic utilities without duplicating logic per operator.
- As a **compiler engineer** maintaining `mycelium-l1`, I want the precedence table and desugaring
  rules to be specified in one normative document, so that parser changes are auditable and the
  transformation is inspectable via `EXPLAIN` (ADR-006: no black boxes).
- As a **tool author** writing `mycfmt` or an editor syntax highlighter (DN-24), I want the
  operator grammar to be committed in `mycelium.ebnf`, so that formatting and highlighting rules
  can be derived mechanically rather than hard-coded ad-hoc.
- As an **AI co-author agent** generating Mycelium code, I want both the infix and word forms
  to be accepted everywhere (if sugar is adopted), so that I can produce idiomatic output without
  tracking which form a given context demands.

---

## 3. Scope & decision space

**In scope for this RFC:**
- The binary decision: adopt the hybrid (sugar on top of word canonical), or stay words-only.
- If sugar is adopted: the normative **precedence / associativity table** for all infix operators
  (`+`, `-`, `*`, `/`, `%`, `&`, `|`, `^`, `<<`, `>>`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `&&`,
  `||`, and unary `-` / `!`).
- The **desugaring map**: which infix operator desugars to which named word function (`+` → `add`,
  `*` → `mul`, `==` → `eq`, etc.), with grounding in the existing word function names in `lib/std/`.
- The **grammar extension** to `mycelium.ebnf`: the `expr` production and the operator-expression
  sub-grammar (left-recursive factoring, precedence climbing, or a Pratt parser approach).
- Whether **user-defined infix operators** (a sigil-based extension) are in scope or deferred.
- The decision about the **word form's continued validity**: must remain legal everywhere the
  infix form is legal (words are canonical — the sugar is additive, not a replacement).

**Out of scope (explicitly deferred):**
- Closures / lambda syntax for operators (e.g., `(+)` as a section) — deferred with the
  RFC-0024 dynamic-HOF residuals.
- Operator overloading / user-defined precedence tables — deferred; the initial table is fixed.
- The `->` function-type arrow and `=>` match-arm fat-arrow — already defined in the grammar and
  not touched by this RFC (DN-23 §4: no new ambiguity).
- Formatting decisions (`mycfmt` normalization of infix vs word forms) — a follow-on to E7-4.

---

## 4. Decision (normative as proposed)

**The hybrid is adopted** (DN-23's recommendation). Mycelium gains a symbolic infix/prefix operator
layer that is **frontend-only sugar**: each operator desugars at parse time to a canonical
word-function application. The kernel is unchanged — words stay canonical, the sugar is **additive**
(the word form remains valid everywhere the sugar is). This is the same discipline as
monomorphization and HOF defunctionalization: a frontend rewrite, **no `mycelium-core` and no L0/L1
node** (KC-3).

### 4.1 Precedence & associativity table (Rust-derived)

The precedence source of truth is **Rust's operator table** (*The Rust Reference*, §"Expressions —
Operator precedence"). Rust is chosen because it is the implementation language and Mycelium's
surface is syntactically adjacent, so reader/author expectations transfer with least surprise.
Tiers are highest-binding (tightest) first; every binary operator is **left-associative**, every
prefix operator is **right-associative** and binds tighter than every binary operator:

| Tier | Operators | Assoc | Class |
|---|---|---|---|
| 1 (tightest) | unary `-`, unary `!` | prefix (right) | negation / bitwise-not |
| 2 | `*` `/` `%` | left | multiplicative |
| 3 | `+` `-` | left | additive |
| 4 | `&` | left | bitwise-and |
| 5 | `^` | left | bitwise-xor |
| 6 | `\|` | left | bitwise-or |
| 7 | `==` `!=` | left | equality |
| 8 | `&&` | left | logical-and |
| 9 (loosest) | `\|\|` | left | logical-or |

Type ascription (`: T`) and the guarantee annotation (`@ strength`) bind **tighter** than every
infix operator — they are postfix on the applicative atom — so `a + b : Binary{8} @ Exact` parses
as `add(a, (b : Binary{8} @ Exact))` (resolving open question Q6). The function-type `->` and the
match-arm `=>` are not expression operators and are untouched (DN-23 §4 — no new ambiguity).

### 4.2 Desugaring map

Each operator desugars to the named word function below (grounded in the existing prim/word names —
`crates/mycelium-l1/src/checkty.rs::prim_kernel_name` and the `examples/`/`lib/std/` corpus):

| Operator | Word | Operator | Word | Operator | Word |
|---|---|---|---|---|---|
| `a + b` | `add(a, b)` | `a & b` | `band(a, b)` | `a == b` | `eq(a, b)` |
| `a - b` | `sub(a, b)` | `a ^ b` | `xor(a, b)` | `a != b` | `ne(a, b)` |
| `a * b` | `mul(a, b)` | `a \| b` | `bor(a, b)` | `a && b` | `and(a, b)` |
| `a / b` | `div(a, b)` | `-a` | `neg(a)` | `a \|\| b` | `or(a, b)` |
| `a % b` | `rem(a, b)` | `!a` | `not(a)` | | |

**Honesty boundary (G2/VR-5):** the desugaring is **purely syntactic** — it produces the word-call
AST regardless of whether the target function exists. The targets that resolve to a kernel prim
**today** are `add`/`sub`/`mul`/`neg` (`trit.*`) and `xor`/`not` (`bit.*`); these are exercised
end-to-end through all three execution paths (the M-705 differential corpus). The remaining targets
(`div`, `rem`, `band`, `bor`, `eq`, `ne`, `and`, `or`) parse and desugar correctly but currently
surface an **explicit** "unknown function/prim" refusal downstream (never silent) until their
stdlib/kernel definitions land — they are reserved in the table so the grammar is complete and the
sugar is stable as those prims arrive.

### 4.3 Deferred: the angle-bracket operators

The ordering/shift operators `<` `<=` `>` `>=` `<<` `>>` are **deferred** (tracked by **M-745**).
`<`/`>` collide with the type-argument `<…>` grammar (`f<T>(x)`, `type Box<A>`); disambiguating
expression `a < b` from a generic instantiation needs contextual lexing or speculative parsing — a
self-contained follow-up. The desugaring map reserves their word targets (`lt`/`le`/`gt`/`ge`/
`shl`/`shr`) so the extension is purely additive when M-745 lands. Until then, the comparison words
remain available in their function form (never silently refused — G2).

### 4.4 EXPLAIN / audit trail (resolves Q5)

The desugaring leaves **no separate record**: the emitted `App` node *is* the audit artifact —
`a + b` and `add(a, b)` are structurally identical after parsing, so the canonical word call is the
inspectable EXPLAIN form (ADR-006). No `DesugarRecord` and no entry in `MonoSelections` is needed;
the rewrite is observable directly in the AST. This keeps the sugar a zero-cost, fully-reified
front-end transform.

---

## 5. Resolved open questions

The §6 questions are resolved here (the table moves to "answered"); the original questions are
retained below for the record (append-only).

1. **Sugar or words-only?** → **Hybrid adopted** (§4), per DN-23's recommendation and the M-705
   implementation. Words stay canonical; sugar is additive.
2. **Which operators in v1?** → Arithmetic (`+ - * / %`), bitwise (`& ^ |`), equality (`== !=`),
   logical (`&& ||`), and unary (`- !`). Ordering/shift (`< <= > >= << >>`) **deferred** (§4.3,
   M-745). Word targets without a prim yet still parse (never-silent downstream refusal).
3. **Precedence source?** → **Rust** (§4.1), cited explicitly (implementation language, adjacent
   surface).
4. **User-defined infix?** → **Deferred** (conservative). The v1 table is fixed; operator
   overloading / custom precedence is out of scope (a future RFC if demand is shown).
5. **EXPLAIN record shape?** → The desugared `App` node is the record (§4.4); no separate structure.
6. **`@` / ascription binding vs infix?** → `@`/`:` bind **tighter** than every infix operator
   (§4.1): `a + b @ Exact` = `add(a, (b @ Exact))`.

---

## 6. Definition of Done

- [x] A binding precedence / associativity table grounded in established practice (**Rust**, cited
  explicitly) — §4.1. *(Maintainer ratification moves it Proposed → Accepted.)*
- [x] An updated `docs/spec/grammar/mycelium.ebnf` capturing the operator-expression grammar
  (`op_expr` … `unary_expr`, precedence-climbing tiers).
- [x] A desugaring specification: every adopted infix/prefix operator maps to a named word function
  (§4.2), with the audit trail being the desugared `App` node itself (§4.4 — ADR-006).
- [x] `crates/mycelium-l1` parser updated to accept infix expressions per the table, with the L1
  suite (lexer + parser + conformance + differential) green and clippy clean.
- [x] Conformance fixture `docs/spec/grammar/conformance/accept/20-operator-syntax.myc` covering
  precedence ordering, associativity, sugar↔word equivalence, and one case per operator class
  (arithmetic, bitwise, comparison, logical, unary). Sugar↔word equivalence is additionally pinned
  end-to-end by the M-705 entries in the `tests/differential.rs` corpus (all three execution paths).
- [x] RFC-0025 status **Draft → Proposed** (this change). **Proposed → Accepted** awaits maintainer
  ratification — never skipping steps (house rule #3).
- [x] DN-23 untouched (append-only); this RFC is the binding forward record.
- [ ] **Deferred (M-745):** the angle-bracket operators `< <= > >= << >>` (type-arg disambiguation;
  §4.3).

---

## 7. Open questions (original record — answered in §5)

*Retained verbatim for the append-only record; each is resolved in §5.*

1. **Sugar or words-only?** DN-23 recommends hybrid, but the decision is the maintainer's. Should
   this RFC adopt the hybrid recommendation or stay words-only? What additional evidence would
   settle this — a usability trial, a dogfooding build (`dfb`), or a community poll?
2. **Which operators get sugar in v1?** Arithmetic + comparison only? Include bitwise
   (`&`, `|`, `^`, `<<`, `>>`), logical (`&&`, `||`), and unary? Or a minimum viable set?
3. **Precedence source of truth:** Rust's table, C's table, or Haskell's? The choice affects
   portability expectations. Cite the chosen table's origin explicitly.
4. **User-defined infix (deferred or in-scope)?** A common request; but it expands the parser and
   makes tooling harder. Deferred is the conservative choice — document it explicitly.
5. **`EXPLAIN` record for desugaring:** should the desugaring reification live in the same
   `MonoSelections` structure as monomorphization (RFC-0024 §4), or in a separate `DesugarRecord`?
   This affects the audit trail shape.
6. **Interaction with grade annotations:** `a + b @ Exact` — does the `@` bind to `b` or to the
   whole expression? The precedence table must specify `@`'s binding level relative to infix ops
   (RFC-0018 §4.3: `@` currently binds tighter than `->` — what about `+`?).

---

## 8. Grounding / honesty

This RFC cites DN-23 (the advisory note that maps the full decision space and grounds the
three-option analysis) and RFC-0024 (HOF — the landing that makes word operators first-class
values). The decision (§4) is implemented Rust-first in `mycelium-l1` (M-705); no tag is upgraded
to `Proven` (VR-5) — the desugaring is a structural transform (no theorem), and the sugar↔word
three-path agreement is **Empirical** (differential trials). The word-function names (`add`, `mul`,
`xor`, …) are grounded in `crates/mycelium-l1/src/checkty.rs::prim_kernel_name`, the
`examples/repr-tour/` corpus, and `lib/std/result.myc` (cited in DN-23 §1).

---

## Meta — changelog

- **2026-06-27 — §4.3 deferred ordering/shift operators resolved by RFC-0037 (M-745; append-only, status unchanged).** The §4.3 deferral of the angle-bracket ordering/shift operators (`< <= > >= << >>`, the M-745 gate) is **resolved by RFC-0037** (the grammar deconfliction RFC, Proposed): type params/args move `<…>` → `[…]`, freeing `<>` for `< > << >>` with **no disambiguation machinery**; `<=`/`>=` **retire as glyphs → word operators `lte`/`gte`** (asymmetric with `<`/`>` — the documented tradeoff that keeps `<>` single-loaded). The desugaring map extends accordingly. No change to this RFC's word-canonical model; the surface allocation now lives in RFC-0037. Append-only.
- **2026-06-23 — Draft → Proposed (M-705).** Adopts DN-23's hybrid: a frontend-only infix/prefix
  operator sugar layer desugaring to canonical word functions. Adds the binding precedence table
  (§4.1, Rust-derived), the desugaring map (§4.2), the EXPLAIN/audit decision (§4.4), and the
  resolved open questions (§5). Angle-bracket operators (`< <= > >= << >>`) **deferred** to M-745
  (type-arg disambiguation, §4.3). Implemented Rust-first: `crates/mycelium-l1` lexer (new operator
  tokens) + parser (precedence-climbing desugaring) + `docs/spec/grammar/mycelium.ebnf` operator
  grammar + `accept/20-operator-syntax.myc` + `tests/differential.rs` sugar↔word corpus. No L0/L1
  kernel change (KC-3). Sugar↔word three-path agreement **Empirical**; no tag upgraded (VR-5).
  Proposed → Accepted awaits maintainer ratification (house rule #3).
- **2026-06-23 — Draft stub created.** Scope, user stories, decision space, Definition of Done,
  and open questions captured as a planning stub. Feeds E7-5 (M-705) and E11-1. Built on DN-23
  (Resolved) and RFC-0024 (implemented, pending ratification). Decides nothing normatively.
  Status: **Draft** (VR-5 / house rule #3).
