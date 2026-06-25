# Design Note DN-23 — Operator Syntax: Words vs Symbols (and the desugaring path)

| Field | Value |
|---|---|
| **Note** | DN-23 |
| **Status** | **Resolved** (2026-06-25 — the operator-syntax direction this note framed was ratified into **RFC-0025** and implemented Rust-first under **M-705**; symbolic infix operators now exist in the lexer. See the Meta/changelog footer.) · was **Draft** (2026-06-23) |
| **Feeds** | RFC-0007 (L1 Kernel Calculus); RFC-0024 (HOF via static defunctionalization); DN-02/DN-03 (lexicon/syntax); E7-5 (operator-syntax leg) |
| **Date** | June 23, 2026 |
| **Decides** | *Nothing normatively* — advisory. Records the design space for basic mathematics / operator syntax (symbols vs words) and the recommended path, so the "whole other leg" of operator support is tracked and grounded before it is taken. A binding decision is deferred to **RFC-0025** (forthcoming). |
| **Task** | E7-5 (operator syntax) |

> **Posture (honesty rule / VR-5).** Advisory. "Current state" claims cite the actual surface
> (`examples/repr-tour/*.myc`, `crates/mycelium-l1/`). No syntax is enacted by this note.

---

## 1. The question

> **Historical snapshot (annotated 2026-06-25, post-audit).** The "today entirely word-based / **no**
> symbolic infix operators" framing below is a **2026-06-23 snapshot** and is **superseded**: the
> recommended hybrid was ratified into **RFC-0025** and **implemented Rust-first (M-705)**, so the lexer
> now tokenizes the symbolic operators (`lexer.rs:181-193` → `Plus`/`Star`/`Slash`/`Percent`, `lex_dash`
> → `Minus`; `token.rs:201-219` defines them). Read §1 and §4 as the design-space record that motivated
> RFC-0025, not as current surface state. See the Meta/changelog footer.

Mycelium's surface is today **entirely word-based** for operations: arithmetic and logic are
ordinary first-order function applications — `add(x, y)`, `xor(acc, b)`, `neg(x)`, `and(x, y)`,
`cmp(a, b)` (see `examples/repr-tour/{swaps,iter,traits}.myc`). There are **no** symbolic infix
operators (`+`, `-`, `*`, `/`). The open question (maintainer, 2026-06-23): should basic
mathematics use **symbols** or **words**, and what are the side effects?

## 2. The decision space

| Option | Pros | Cons |
|---|---|---|
| **Words (status quo)** — `add(a, b)` | No lexer ambiguity; **no precedence/associativity machinery** (a real spec+parser cost avoided); a word operator is a **named function** ⇒ first-class and passable to HOFs (`map`/`fold`) once RFC-0024 lands; KISS; small kernel (KC-3); greppable | Verbose for math-heavy code (`add(mul(a,b), c)` vs `a*b + c`); unfamiliar for numeric work |
| **Symbols** — `a + b` | Concise / familiar for math | Precedence + associativity (spec + parser complexity — the "whole other leg"); `-` disambiguation; more lexer state |
| **Hybrid (recommended)** — word kernel + optional symbol **sugar** that desugars to the word functions (`a + b` ⟶ `add(a, b)`) | Both: a simple first-order word kernel (L0 unchanged) **and** ergonomic symbols at the surface; precedence defined once in the sugar's RFC; frontend-only (same shape as the HOF defunctionalization — **no kernel change**, KC-3) | The sugar layer still needs a precedence/associativity spec + parser support (deferred to its own RFC) |

## 3. Recommendation

- **Keep words canonical in the kernel** (status quo). Consistent with the existing surface, KISS,
  and it composes with RFC-0024 — a word operator is a named fn, first-class once HOF lands.
- **If/when symbolic math is wanted, add it as pure surface sugar** that desugars to the word
  functions — a **frontend-only** transform (no `mycelium-core` / L0 change), exactly the discipline
  used for generics (monomorphization) and HOF (defunctionalization). Ergonomics without precedence
  complexity leaking into the kernel.
- **Binding decision deferred to RFC-0025** (the operator-syntax / sugar RFC). This note enacts no
  choice.

## 4. Side effects — already handled or moot (grounded)

> **Historical snapshot (annotated 2026-06-25).** This section's "a *future* `-` math operator" /
> "RFC-0025 (forthcoming)" framing is superseded — the `-` math operator and the rest of the symbolic
> set landed via RFC-0025 / M-705 (`lexer.rs:262` already cites RFC-0025/M-705). The dash-disambiguation
> analysis below remains accurate as the *reason* the addition was clean.

- **The dash / `-`:** the lexer already tokenizes `-`, `->`, and `=>` as **distinct atomic tokens**
  (`crates/mycelium-l1/src/lexer.rs` `lex_dash` / `lex_eq`; this is what makes RFC-0024's `A -> B`
  unambiguous). A *future* `-` math operator therefore coexists with the `->` function-type arrow at
  the token level — no new ambiguity to design around.
- **Casing (and where kebab-case actually lives):** Mycelium identifiers are **snake_case** — grounded
  in the lexer: `is_ident_continue` admits only `is_ascii_alphanumeric() || '_'`
  (`crates/mycelium-l1/src/lexer.rs:270-272`), so a `-` can **never** be part of an identifier.
  Constructors are **PascalCase** (`Ok` / `Err`); nodule names are **dotted** `Ident` paths
  (`tour.iter`, `hello.greeting` — `path ::= Ident ('.' Ident)*`, EBNF). **None use dashes.** The
  kebab-case the project *does* use is **git branch names only** (the swarm convention
  `claude/leaf/<EPIC>-<LEAF>-<kebab-description>`, CLAUDE.md) — a VCS/tooling convention, **not** a
  language identifier — so a `-` operator has zero interaction with it. Net: snake_case already
  coexists with any future `-` operator; **no casing change is forced**, and kebab-case (which a `-`
  operator would preclude *in identifiers*) is not used in the language.

## 5. Scope / honesty

A deferred leg (**E7-5**), sequenced **after** the in-flight L1 surface work (HOF capstone E7-3;
comment-preserving `mycfmt` E7-4) and **before** the dogfooding builds (`dfb`, M-670/M-671), which
benefit from a complete, ergonomic surface. No tag is upgraded; no syntax is enacted here (VR-5).

---

## Meta — changelog

- **2026-06-25 — Resolved (post corpus-alignment audit).** This note's recommended path (a word kernel
  with optional symbol **sugar**) was **ratified into RFC-0025** and **implemented Rust-first under
  M-705**; the lexer now tokenizes the symbolic infix operators (`crates/mycelium-l1/src/lexer.rs:181-193` →
  `Plus`/`Star`/`Slash`/`Percent`; `lex_dash` → `Minus`; `crates/mycelium-l1/src/token.rs:201-219`
  defines the set). The §1/§4 "current state" prose (entirely word-based / no symbolic operators /
  RFC-0025 forthcoming) is therefore a **historical snapshot** and has been annotated as such (the prose
  itself is left intact — append-only). Status moves **Draft → Resolved** (forward; the direction is now
  ratified + built), mirroring sibling DN-24's move to Resolved when its RFC landed. No tag upgraded
  (VR-5).
- **2026-06-25 — Cross-note (D7, advisory).** The *glyph allocation* for the symbolic operators is the
  subject of a separate maintainer decision (DN-31, 2026-06-24/25): **`[]` is adopted for type-arguments**,
  freeing `<>` for the comparison/shift operators (`< > << >>`), which resolves the `<>`/type-arg vs
  operator collision (**M-745**). That is a recorded *direction* gated behind a future grammar-supersession
  wave (see DN-31 §2–§3 and RFC-0030); it does not reopen the word-vs-symbol decision this note resolved.
- **2026-06-23 — Draft.** Initial design-space capture for operator syntax (symbols vs words) and the
  recommended hybrid (word kernel + frontend-only symbol sugar). Decided nothing normatively; deferred the
  binding decision to RFC-0025.
