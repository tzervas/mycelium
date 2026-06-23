# Design Note DN-23 — Operator Syntax: Words vs Symbols (and the desugaring path)

| Field | Value |
|---|---|
| **Note** | DN-23 |
| **Status** | **Draft** (2026-06-23) |
| **Feeds** | RFC-0007 (L1 Kernel Calculus); RFC-0024 (HOF via static defunctionalization); DN-02/DN-03 (lexicon/syntax); E7-5 (operator-syntax leg) |
| **Date** | June 23, 2026 |
| **Decides** | *Nothing normatively* — advisory. Records the design space for basic mathematics / operator syntax (symbols vs words) and the recommended path, so the "whole other leg" of operator support is tracked and grounded before it is taken. A binding decision is deferred to **RFC-0025** (forthcoming). |
| **Task** | E7-5 (operator syntax) |

> **Posture (honesty rule / VR-5).** Advisory. "Current state" claims cite the actual surface
> (`examples/repr-tour/*.myc`, `crates/mycelium-l1/`). No syntax is enacted by this note.

---

## 1. The question

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
