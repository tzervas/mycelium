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
| Promoted stable component (scope-level) | `matured` | themed (grown to a hardened stage); RFC-0017: a **header/manifest key** on a `nodule`/`phylum`/program, **not** a fn modifier |
| De-maturation (keep one def interpreted in a matured scope) | `thaw` | conventional-clearest — inverts `matured` = "compiled-and-**frozen**"; `germinate` is taken by spore-germination (ADR-013), RFC-0017 §4.3/§5 |
| Local binding | `let` … `in` | conventional |
| Conditional | `if`/`then`/`else` | conventional |
| Pattern match | `match` | conventional |
| Representation change | `swap` | native corpus term |
| Affine external resource | `Substrate{…}` | themed (consumed once = affinity) |
| Reconstruction manifest | `spore` | themed (self-contained regrowth) |
| Unsafe escape hatch | `wild` | themed (uncultivated, denied by default) |
| Structured-concurrency scope (dynamic grouping) | `colony { hypha …, … }` | themed (a living group of cooperating organisms; RFC-0008 §4.7, DN-06; **M-666**) |
| Concurrent execution unit (inside a `colony`) | `hypha <expr>` | themed (a fungal filament; RFC-0008 §4.5; **M-666**) |
| Guarantee annotation | `T @ Exact` | the LR-6 type-level honesty index |

Guarantee tags (`Exact` `Proven` `Empirical` `Declared`) and scalar kinds (`F16` `BF16` `F32`
`F64`) are reserved words. Literals are **representation-typed and universal-until-elaboration**
with no defaulting across representation families (Q6): `0b1011_0010` (binary), `<+0--0>`
(MSB-first balanced ternary), decimal ints, `[…]` lists.

**Active (M-666; RFC-0008 §4.7 R1).** `colony` and `hypha` are now active surface constructs (the
deterministic structured-concurrency fragment). `colony { hypha e, … }` is an **expression** (a
`colony_expr`): the dynamic runtime grouping of `hypha` execution units, whose reference semantics
is the spawn-order sequentialization (RT2). A `hypha` is **only** expressible inside a `colony` (RT7
— "an orphan hypha is not expressible"; a loose `hypha` is `conformance/reject/13`). The v0 surface
has no product type, so a colony yields the **last** hypha's value (a join-result product is later
work). The runtime realization is `mycelium-mlir::runtime` (`Scope`/`Colony`/`Task`, M-357) — a
performance path validated against the RT2 sequentialization, not an L0 kernel node (the trusted base
stays sequential; KC-3). Accept fixture: `conformance/accept/13`.

**Active (M-662; RFC-0006, DN-06 §6).** `phylum` (the library-scale grouping *above* nodules) is now
an active surface construct. An optional `phylum <path>` header groups one-or-more `nodule` blocks in
one source file (`program ::= phylum_header? nodule_block+`); a header-less single nodule is a
*phylum-of-one* (backward-compatible). Cross-nodule names are exported with **`pub`** (`pub fn` / `pub
type` / `pub trait`; absent ⇒ private to the nodule) and imported with **`use`** — specific (`use
a.b.X`) or glob (`use a.b.*`). Resolution precedence is local-decl > explicit-`use` > glob (higher
shadows lower deterministically); a `use` of an absent or private name, a duplicate import, or a
*referenced* glob-vs-glob collision is a never-silent refusal (G2). The cross-nodule **orphan rule**
(RFC-0019 §4.5) is enforced phylum-wide over a pub-blind coherence view (`Declared`). Accept fixture:
`conformance/accept/19`; the phylum-no-nodule parse refusal is `conformance/reject/10`. Plurals (prose
only, never reserved identifiers): `phylum`/`phyla`, `nodule`/`nodules`, `colony`/`colonies`,
`hypha`/`hyphae`.

**Reserved, not yet active — runtime vocabulary (DN-03 §4; RFC-0008 §4.5).** The remaining nine
runtime-model names are reserved keywords: they lex as keywords (never silent identifiers, G2) but
no v0 construct consumes them — they cannot open a program or appear in identifier position
(`conformance/reject/12`). They activate when their runtime-model constructs land (RFC-0008 §4.6
R1/R2). (`hypha` **left** this set with M-666 — see *Active* above.)

| Keyword | Runtime concept | RFC-0008 ref |
|---|---|---|
| `fuse` | lawful state fusion / CRDT join | RT6 |
| `mesh` | decentralized gossip/pub-sub overlay | RT5 |
| `graft` | capability contract with infrastructure | RT4 |
| `cyst` | durable checkpoint / dormant resumable form | RT2 |
| `xloc` | explicit value movement / trans-locate | — |
| `forage` | adaptive placement policy | RT3 |
| `backbone` | priority transport path | RT3 |
| `tier` | execution-mode switch (interpreted ↔ native) | — |
| `reclaim` | runtime-unit reclamation (stale units only, never memory) | RT7 |

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
  every `reject/` program with an explicit `ParseError` (never a silent accept, never a panic). The
  oracle is `parse_phylum` (M-662) — the top-level entry, a strict superset of `parse`: a bare nodule
  is a phylum-of-one, so every pre-phylum fixture still holds.

## Status

**Committed L3 text surface (DN-09 KC-2 verdict — proceed; RFC-0006 §10 Q1 resolved;
2026-06-18).** The grammar is the normative oracle for the accept/reject corpus; refinements are
append-only recorded decisions, not silent drift. `.myc` is the file extension (language name =
Mycelium, shared with the project).

Two recent additions to the operator + surface picture:

- **Operator-expression layer (RFC-0025 / M-705; implemented, pending ratification).** An
  `op_expr` production with precedence tiers `or → and → eq → bor → xor → band → add → mul →
  unary → app` desugars each symbolic operator to a canonical word function (`a + b → add(a, b)`,
  `-a → neg(a)`) — frontend-only sugar, no L0/L1 kernel change (KC-3). Angle-bracket comparisons
  (`<`, `<=`, `>`, `>=`, `<<`, `>>`) are deferred (RFC-0025 §3; M-745). RFC-0025 is **Proposed**
  (implemented Rust-first, pending maintainer ratification — house rule #3). Accept fixture:
  `conformance/accept/20-operator-syntax.myc`.
- **RFC-0030 (Concrete Surface Grammar + L3 Ratification) — Draft.** The formal ratification pass
  for the full committed grammar; **Draft → Proposed** is gated on M-707 (RFC-0020 L2 surface
  complete) and M-745 (angle-bracket disambiguation). Status stays Draft — no premature advancement
  (VR-5 / house rule #3).

### Changelog

- **2026-06-23 — Operator-expression layer integrated (RFC-0025 / M-705).** `op_expr` with the
  full precedence table added to `mycelium.ebnf`; `conformance/accept/20-operator-syntax.myc`
  added. RFC-0025 → Proposed (pending ratification). RFC-0030 opened (Draft). Append-only.
