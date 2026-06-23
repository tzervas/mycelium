# RFC-0025 — Operator Syntax: Symbolic Infix Sugar & Precedence Table

| Field | Value |
|---|---|
| **RFC** | 0025 |
| **Status** | **Draft** (2026-06-23) |
| **Type** | Surface / normative (once Accepted) — frontend-only; no L0 or L1 kernel change |
| **Date** | 2026-06-23 |
| **Feeds** | E7-5 (operator-syntax leg); E11-1 (surface-language completeness) |
| **Decides** | The binding decision DN-23 deferred: whether to add symbolic infix sugar (`a + b → add(a, b)`), and if so, the complete precedence/associativity table and the sugar grammar. DN-23's recommendation (hybrid: word kernel + optional symbol sugar as frontend-only desugaring) is the starting point. |
| **Depends on** | RFC-0006 §3/§4.3 (grammar discipline — S1–S6, machine-readable EBNF); RFC-0007 §4.1–§4.8 (L1 kernel calculus — the desugaring target); RFC-0024 (HOF via static defunctionalization — word operators become first-class values once HOF lands, which strengthens the word-canonical argument); DN-23 (design space + recommendation; source of truth for the three options); `docs/spec/grammar/mycelium.ebnf` (the EBNF to be extended or left unchanged) |
| **Task** | E7-5 (M-705) |

> **Posture (honesty rule / VR-5).** This is a **planning stub** — scope, user stories, decision
> space, and open questions only. It decides nothing normatively. Status: **Draft**. All normative
> choices wait on the design process this RFC governs. No syntax is enacted, no grammar change is
> made, and no guarantee tag is upgraded here.

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

## 4. Definition of Done

- A maintainer-ratified, binding precedence / associativity table with a normative grounding for
  each level (drawn from established practice — C / Haskell / Rust — cited explicitly).
- An updated `docs/spec/grammar/mycelium.ebnf` capturing the operator-expression grammar or a
  confirmed decision that the grammar stays words-only.
- A desugaring specification: every adopted infix operator maps to a named word function,
  auditable via `EXPLAIN` (ADR-006), with the map committed in this RFC.
- `crates/mycelium-l1` parser updated to accept (or reject) infix expressions per the table, with
  `just check` green (including `cargo clippy -D warnings` and the conformance corpus).
- Accept/reject conformance fixtures in `docs/spec/grammar/conformance/` covering: operator
  precedence ordering, associativity (left/right), the sugar↔word equivalence, and at least one
  case per operator class (arithmetic, bitwise, comparison, logical).
- RFC-0025 status moves from **Draft → Proposed** (for maintainer review) and then
  **Proposed → Accepted** (maintainer ratification) — never skipping steps (house rule #3).
- DN-23 remains untouched (append-only); this RFC is the binding forward record.

---

## 5. Open questions

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

## 6. Grounding / honesty

This stub cites DN-23 (the advisory note that maps the full decision space and grounds the
three-option analysis) and RFC-0024 (HOF — the landing that makes word operators first-class
values). Normative decisions wait on the design process. No implementation exists; no tag is
upgraded (VR-5). Claims about what the word functions are named (`add`, `mul`, etc.) are grounded
in `examples/repr-tour/` and `lib/std/result.myc`, cited in DN-23 §1.

---

## Meta — changelog

- **2026-06-23 — Draft stub created.** Scope, user stories, decision space, Definition of Done,
  and open questions captured as a planning stub. Feeds E7-5 (M-705) and E11-1. Built on DN-23
  (Resolved) and RFC-0024 (implemented, pending ratification). Decides nothing normatively.
  Status: **Draft** (VR-5 / house rule #3).
