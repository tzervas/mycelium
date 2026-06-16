# Mycelium grammar artifacts

The machine-readable grammar of Mycelium-the-language and its **conformance corpus** — the
WebAssembly-spec pattern (T3.1-B): a normative grammar plus an accept/reject test corpus that any
parser implementation is checked against. The corpus, not any single parser, is the ground truth
(RFC-0006 §4.3).

## Files

- **`mycelium.ebnf`** — the normative surface grammar, **v0** (the L1-facing core). W3C-notation
  EBNF (not ISO 14977; T3.1-B). This is the oracle: every program under `conformance/accept/`
  parses by it; every program under `conformance/reject/` does not.
- **`conformance/accept/*.myc`** — well-formed programs that **must parse**. Each file opens with a
  `//` comment naming what it exercises.
- **`conformance/reject/*.myc`** — malformed programs that **must fail to parse**, each with a
  `//` comment stating *why* it is rejected (the never-silent expectation: a rejection is an
  explicit diagnostic, never a silent accept).

## The ratified vocabulary (DN-02, Resolved 2026-06-10; DN-06, Resolved 2026-06-16)

| Concept | Keyword | Themed? |
|---|---|---|
| Static organizational unit (the basic "module") | `nodule` | themed (a small self-contained growth; DN-06, replaces static `colony`) |
| Import | `use` | conventional |
| Data type (sum) | `type` | conventional |
| Trait / typeclass | `trait` | conventional |
| Function | `fn` | conventional |
| Promoted stable component | `matured` | themed (grown to a hardened stage) |
| Local binding | `let` … `in` | conventional |
| Conditional | `if`/`then`/`else` | conventional |
| Pattern match | `match` | conventional |
| Representation change | `swap` | native corpus term |
| Affine external resource | `Substrate{…}` | themed (consumed once = affinity) |
| Reconstruction manifest | `spore` | themed (self-contained regrowth) |
| Unsafe escape hatch | `wild` | themed (uncultivated, denied by default) |
| Guarantee annotation | `T @ Exact` | the LR-6 type-level honesty index |

Guarantee tags (`Exact` `Proven` `Empirical` `Declared`) and scalar kinds (`F16` `BF16` `F32`
`F64`) are reserved words. Literals are **representation-typed and universal-until-elaboration**
with no defaulting across representation families (Q6): `0b1011_0010` (binary), `<+0--0>`
(MSB-first balanced ternary), decimal ints, `[…]` lists.

**Reserved, not yet active (DN-06, Resolved 2026-06-16).** `phylum` (the library-scale grouping
*above* nodules) and `colony` (reassigned to the **dynamic** runtime grouping of `hypha`, RFC-0008
§4.7) are reserved keywords: they lex as keywords — so they are never silent identifiers — but no
v0 construct consumes them, so neither opens a program (`conformance/reject/10`). They activate when
their constructs land (`phylum`: RFC-0006; `colony`: RFC-0008). Plurals (prose only, never reserved
identifiers): `phylum`/`phyla`, `nodule`/`nodules`, `colony`/`colonies`, `hypha`/`hyphae`.

### The `// nodule:` header marker (DN-06 §6)

A source file declares its nodule status with a comment on its **first non-blank line** —
`// nodule: <dotted.name>` (named) or bare `// nodule` — **not** in the filename/path (paths stay
conventional). The marker is a *source-text* convention, not grammar: comments are lexer trivia and
the marker is **never** part of content-addressed identity (metadata is not identity, ADR-003). A
near-miss named marker (`// nodule:` with an empty or ill-formed name) is an **explicit** error,
never silently dropped (G2). The recogniser is `mycelium_l1::parse_nodule_header`; the linter (M-141)
surfaces a malformed marker and the formatter (M-142) preserves a valid one. The richer **structured**
header (`// @key: value`) + `mycelium-proj.toml` manifest layer on top of this (M-359).

## How it is checked

- **`scripts/checks/grammar.sh`** (in `just check` / CI): structural validation — the EBNF exists,
  the corpus is present and categorized, every `.myc` carries its explanatory header. Pure shell;
  no toolchain needed, so it rarely skips.
- **`crates/mycelium-l1` `tests/conformance.rs`** (in `cargo test`): the real parser gate — the
  hand-written recursive-descent parser must **accept** every `accept/` program and **reject**
  every `reject/` program with an explicit `ParseError` (never a silent accept, never a panic).

## Status

**v0, non-normative until RFC-0006 is ratified.** The grammar tracks the ratified DN-02 lexicon;
it grows as the L1 RFC (kernel calculus, typing judgments) lands. `.myc` is the file extension
(language name = Mycelium, shared with the project).
