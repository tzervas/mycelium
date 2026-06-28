# RFC-0025 — Operator Syntax: Symbolic Infix Sugar & Precedence Table

| Field | Value |
|---|---|
| **RFC** | 0025 |
| **Status** | **Enacted** (2026-06-28 — **ratified by the maintainer in-session**: the M-745 operator wiring landed Rust-first in `crates/mycelium-l1` and is green (`just check`, PR #723) — `<`/`>`→`lt`/`gt`, `<<`/`>>`→`shl`/`shr` at the §4.1 Tier-8/Tier-4 slots, `lte`/`gte` word-only, frontend-only/no L0/L1 change (KC-3). Stepped through **Accepted** first (house rule #3 — never skipped). The desugaring map (§4.2) and precedence table (§4.1) are now both binding **and implemented**; the EBNF follows §4.1, not RFC-0037 §6's illustrative sketch (FLAG-E — §6 to be corrected append-only). The remaining word targets without a prim (`div`/`rem`/`band`/`bor`/`eq`/`ne`/`and`/`or`/`lt`/`gt`/`shl`/`shr`/`lte`/`gte`) still refuse explicitly downstream until their prims land (M-809) — never silent (G2); Enacted is over the **surface wiring**, not those pending prim definitions.) Prior status chain (append-only): **Accepted** (2026-06-28 — operator residue ratified; M-745 wiring COMMISSIONED) — **operator residue ratified by maintainer 2026-06-28 (in-session)**: the M-745 operator wiring (lt/gt/shl/shr/lte/gte) is COMMISSIONED for implementation (M-745 close + M-809 grammar-supersession epic). → **Enacted** once the M-745 wiring implementation lands. Do NOT self-Enact — Enacted requires maintainer ratification after implementation (house rule #3). **Accepted** (2026-06-27) — *Proposed → Accepted, **ratified by the maintainer 2026-06-27** (R1 gate); the angle-bracket-operator deferral (M-745) is lifted by RFC-0037's operator reallocation. Already implemented Rust-first (M-705); → **Enacted** once the desugaring map is updated for RFC-0037 (`<=`/`>=` → `lte`/`gte`, add `lt`/`gt`/`shl`/`shr`).* **Proposed** (2026-06-23) — adopts DN-23's **hybrid** recommendation: a frontend-only infix/prefix sugar layer desugaring to canonical word functions, no L0/L1 kernel change (KC-3). Binding precedence table (§4.1), desugaring map (§4.2), grammar extension normative; implemented Rust-first (M-705 — `crates/mycelium-l1` lexer+parser + `docs/spec/grammar/mycelium.ebnf` + `accept/20-operator-syntax.myc`; sugar↔word agreement L1-eval ≡ L0-interp ≡ AOT **Empirical**). |
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
| 4 | `<<` `>>` | left | shift *(added — RFC-0037 §5-S2/D1, M-745 resolved; `Declared`)* |
| 5 | `&` | left | bitwise-and |
| 6 | `^` | left | bitwise-xor |
| 7 | `\|` | left | bitwise-or |
| 8 | `<` `>` | left | comparison *(added — RFC-0037 §5-S2/D1, M-745 resolved; `<=`/`>=` retire as glyphs → word-only `lte`/`gte`; `Declared`)* |
| 9 | `==` `!=` | left | equality |
| 10 | `&&` | left | logical-and |
| 11 (loosest) | `\|\|` | left | logical-or |

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

**Added by RFC-0037 §5-S2 (M-745 resolved; `Declared`):**

| Operator | Word | Note |
|---|---|---|
| `a < b` | `lt(a, b)` | Glyph operator — freed by RFC-0037 kind-split (D1) |
| `a > b` | `gt(a, b)` | Glyph operator — freed by RFC-0037 kind-split (D1) |
| `a << b` | `shl(a, b)` | Glyph operator — freed by RFC-0037 kind-split (D1) |
| `a >> b` | `shr(a, b)` | Glyph operator — freed by RFC-0037 kind-split (D1) |
| `lte(a, b)` | `lte(a, b)` | **Word-form only** — `<=` glyph retired by RFC-0037 D1 (type-arg edge) |
| `gte(a, b)` | `gte(a, b)` | **Word-form only** — `>=` glyph retired by RFC-0037 D1 (type-arg edge) |

`lte` and `gte` are **word operators only** — they have no glyph sugar. The asymmetry (single-char
`<`/`>` have glyphs; two-char `<=`/`>=` do not) is intentional: the two-character glyphs are the
ones that produce the most severe parsing ambiguity following a type argument (`f[T]` and `f[T]=…`),
even after the kind-split (RFC-0037 §4-D1 rationale). Word-canonical form is always available and
is the desugaring target for the glyph operators — `lte`/`gte` are already word-canonical and
require no desugaring step.

**Honesty boundary (G2/VR-5):** the desugaring is **purely syntactic** — it produces the word-call
AST regardless of whether the target function exists. The targets that resolve to a kernel prim
**today** are `add`/`sub`/`mul`/`neg` (`trit.*`) and `xor`/`not` (`bit.*`); these are exercised
end-to-end through all three execution paths (the M-705 differential corpus). The remaining targets
(`div`, `rem`, `band`, `bor`, `eq`, `ne`, `and`, `or`) parse and desugar correctly but currently
surface an **explicit** "unknown function/prim" refusal downstream (never silent) until their
stdlib/kernel definitions land — they are reserved in the table so the grammar is complete and the
sugar is stable as those prims arrive. The RFC-0037-added targets (`lt`, `gt`, `shl`, `shr`,
`lte`, `gte`) are likewise reserved in this map; their stdlib/kernel definitions arrive with the
RFC-0037 grammar-supersession epic (M-809) — until then, word form calls produce an explicit
"unknown function/prim" refusal, never silent (G2).

### 4.3 Deferred: the angle-bracket operators

The ordering/shift operators `<` `<=` `>` `>=` `<<` `>>` are **deferred** (tracked by **M-745**).
`<`/`>` collide with the type-argument `<…>` grammar (`f<T>(x)`, `type Box<A>`); disambiguating
expression `a < b` from a generic instantiation needs contextual lexing or speculative parsing — a
self-contained follow-up. The desugaring map reserves their word targets (`lt`/`le`/`gt`/`ge`/
`shl`/`shr`) so the extension is purely additive when M-745 lands. Until then, the comparison words
remain available in their function form (never silently refused — G2).

> **Resolution note — M-745 closed by RFC-0037 (append-only, 2026-06-28).**
> The collision above is resolved by RFC-0037 §4-D1 (bracket kind-split): type parameters and type
> arguments move from `<…>` to `[…]`, leaving `<>` **operators-only**. With no type-argument role,
> `a < b` is unambiguously a comparison expression — no contextual lexing required. The §4.1
> precedence table is extended with Tiers 4 (`<<`/`>>` shift) and 8 (`<`/`>` comparison), and the
> §4.2 desugaring map gains `lt`/`gt`/`shl`/`shr` as glyph-operator entries. The `<=`/`>=` glyphs
> are **retired** (RFC-0037 D1 rationale: the two-character forms are the most ambiguous in
> post-type-arg position even after the kind-split); their word-canonical forms `lte`/`gte` remain
> valid everywhere. The original word-target reservation (`le`/`ge`) is superseded by the
> RFC-0037-specified names `lte`/`gte`. See §4.2 second table and §4.1 updated tier list.
> M-745 gate: **met**. This RFC (Accepted) → **Enacted** once the grammar-supersession epic
> (RFC-0037 §8 / M-809) lands the updated `mycelium.ebnf` and parser.
> `Declared` (RFC-0037's collision-freedom claim is construction-argument, not mechanically verified).

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
   *Update (2026-06-28, RFC-0037):* M-745 is now resolved. Glyph operators `<`/`>`/`<<`/`>>` are
   added (§4.1 Tiers 4 and 8; §4.2 second table). `<=`/`>=` retire as glyphs; `lte`/`gte` are
   word-only. The §4.3 deferral is lifted; see §4.3 resolution note.
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
- [x] **M-745 — RESOLVED by RFC-0037 (2026-06-28):** the angle-bracket operators `<`/`>`/`<<`/`>>`
  are added to the precedence table (§4.1 Tiers 4 and 8) and desugaring map (§4.2). `<=`/`>=`
  retire as glyphs; `lte`/`gte` are word-only (§4.3 resolution note). This RFC → **Enacted** once
  the grammar-supersession epic (M-809) lands the updated `mycelium.ebnf` + parser.

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

- **2026-06-28 — Accepted → ENACTED (maintainer ratified in-session).** With the M-745 wiring landed
  and green (PR #723), the maintainer ratified the Accepted → Enacted move that this RFC's Status
  guard reserved for them ("do NOT self-Enact"). The §4.1 precedence table and §4.2 desugaring map
  are now binding **and implemented** in `crates/mycelium-l1` (frontend-only, KC-3); the operator
  surface (`+ - * / % & ^ | << >> < > == != && ||`, unary `- !`, word-only `lte`/`gte`) is complete
  and stable. Stepped through Accepted first (house rule #3 — not skipped). Scope of Enacted = the
  **surface wiring + desugaring**; word targets lacking a kernel/stdlib prim still refuse explicitly
  downstream until M-809 (never silent — G2). **FLAG-E stands** (RFC-0037 §6's illustrative EBNF
  sketch contradicts the binding §4.1 tiers; §6 to be corrected append-only — the implementation
  follows §4.1). (Append-only; VR-5; G2.)
- **2026-06-28 — M-745 operator wiring LANDED (ops kickoff; status UNCHANGED — Accepted).** The
  commissioned wiring is implemented Rust-first in `mycelium-l1` (frontend-only, no L0/L1 change —
  KC-3): the lexer lexes `<<`/`>>` whole as `Tok::Shl`/`Tok::Shr` (`<`/`>` stay `LAngle`/`RAngle`);
  `parse.rs::infix_op` desugars `<`/`>` → `lt`/`gt` at the §4.1 Tier-8 slot (bp 25) and `<<`/`>>` →
  `shl`/`shr` at the Tier-4 slot (bp 55); `mycelium.ebnf` gains the `cmp_expr`/`shift_expr`
  productions; parse/lexer unit tests + the `accept/20-operator-syntax.myc` oracle cover the new
  glyphs (precedence, left-associativity, whole-token shift lexing). `lte`/`gte` remain word-only
  (no glyph). M-745 → **done**; RFC-0030 §4.3 gate **met (implemented)**. The new word targets
  (`lt`/`gt`/`shl`/`shr`/`lte`/`gte`) parse + desugar but surface an explicit "unknown function/prim"
  refusal downstream until their prims land (M-809) — never silent (G2). **STATUS STAYS Accepted:**
  per this RFC's own Status guard and FLAG-C, the Accepted → Enacted move is a **maintainer
  ratification step** — *not* self-applied here even though the wiring is now in (house rule #3;
  VR-5 — assent is not upgraded past its basis). **FLAG-E (§4.1 vs RFC-0037 §6 precedence
  inconsistency):** RFC-0037 §6's *illustrative* EBNF sketch nests `cmp_expr ::= shift_expr` /
  `shift_expr ::= bor_expr`, which would place shift **looser** than the bitwise ops — contradicting
  this RFC's ratified §4.1 table (shift Tier 4 = *tighter* than `& ^ |`; comparison Tier 8 = looser
  than `|`, tighter than `==`) and Rust (§4.1's cited source of truth). The implementation and the
  regenerated `mycelium.ebnf` follow the **ratified §4.1 tiers** (the binding source), not the §6
  sketch; the §6 sketch is non-normative ("illustrative; the normative EBNF artifact is updated by
  the grammar-supersession epic"). RFC-0037 §6 should be corrected append-only to match §4.1.
  (Append-only; VR-5; G2.)
- **2026-06-28 — operator residue ratified; M-745 wiring COMMISSIONED (in-session).** Operator residue (lt/gt/shl/shr/lte/gte) ratified by maintainer. M-745 operator wiring is COMMISSIONED for implementation (M-745 close + M-809 grammar-supersession epic). → Enacted once M-745 wiring lands (maintainer ratification required — do NOT self-Enact; house rule #3). (Append-only; VR-5; G2.)
- **2026-06-28 — §4.1/§4.2/§4.3 updated: M-745 residue wired in (RFC-0037 Enacted; design-draft for review).** After RFC-0037 was promoted to Enacted (grammar-supersession epic landed), this RFC's body is updated to close the M-745 residue: (1) §4.1 precedence table extended with Tier 4 (`<<`/`>>` shift) and Tier 8 (`<`/`>` comparison), renumbering former Tiers 4–9 → 5–11; (2) §4.2 desugaring map gains second table with `lt`/`gt`/`shl`/`shr` glyph entries and `lte`/`gte` word-only entries (supersedes the §4.3 reservation of `le`/`ge` → RFC-0037 specifies `lte`/`gte`); (3) §4.3 resolution note added (M-745 met; Enacted gate = M-809 grammar-supersession epic); (4) §5 Q2 and §6 DoD updated accordingly. Guarantee tag `Declared` throughout (RFC-0037's collision-freedom is a construction argument, not mechanically verified). STATUS UNCHANGED — this is a design-draft update for maintainer review; no status move proposed here (house rule #3). FLAGs to maintainer: **FLAG-A** (M-745 close: issues.yaml M-745 status update + M-706 depends_on update needed — orchestrator-owned, not touched here); **FLAG-B** (EBNF regen: `mycelium.ebnf` needs the `cmp_expr`/`shift_expr` productions from RFC-0037 §6 and the updated `op_expr` chain — tied to M-809 grammar-supersession epic, not this doc update); **FLAG-C** (RFC-0025 Enacted gate: once M-809 lands, this RFC's status moves Accepted → Enacted — maintainer ratification step needed; never skip Accepted); **FLAG-D** (word-target naming: §4.3 originally reserved `le`/`ge`; RFC-0037 specifies `lte`/`gte` — confirmed supersession recorded in §4.3 resolution note and §4.2 second table; no ambiguity in the corpus).
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
