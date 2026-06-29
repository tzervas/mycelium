# Design Note DN-57 — Delimiter Semantics: `:` ascribe · `,` separate · `;` terminate

| Field | Value |
|---|---|
| **Note** | DN-57 |
| **Status** | **Accepted** (2026-06-27) — *Draft → Accepted, **ratified by the maintainer 2026-06-27**: the delimiter role split (`:` ascribe · `,` separate · `;` terminate) and **`;` as the component/operation terminator** are the decided direction. **Enacts no code** — the §3 open questions (mandatory-vs-optional `;`; which components terminate; the `}` interaction; corpus + formatter migration) are the **enactment design**, settled in the follow-on `;`-terminator wave, after which this → **Enacted** (VR-5: ratifying the decision ≠ claiming the surface is implemented). Refines RFC-0037 layout-independence (FLAG-2). **Implemented (Rust-first), optional-terminator form** (2026-06-27, patched into the RFC-0037 grammar epic): `Tok::Semi` + lexer + parser accept an **optional** `;` terminating a top-level item or a trait/impl method, **AST-transparent** (adds no node), so whitespace-free/streamable source is legal; `crates/mycelium-l1` (`token`/`lexer`/`parse`/test) + `mycelium.ebnf` updated, full `mycelium-l1`+`mycelium-fmt` suite green. **Remaining before Enacted** (DN-57 §3): the **mandatory** form (the full streaming guarantee) + the nodule-header terminator + formatter `;`-emission + the corpus migration. Prior: **Draft** (2026-06-27, maintainer-proposed).* |
| **Feeds** | **RFC-0037** (Surface Grammar Deconfliction & Layout-Independence — the `,`-delimited, newline-insignificant grammar this completes); the **streamed / incremental parser variant**; RFC-0030 (concrete L3 grammar) once enacted. |
| **Date** | June 27, 2026 |
| **Decides** | *Proposes, for ratification:* a clean, non-overlapping role for the three punctuation delimiters so the surface is **fully whitespace-independent** and **streamable** — `:` ascribes, `,` separates siblings, `;` terminates a component. |

> **Posture (transparency / VR-5 / G2).** Design capture only. The streaming/parse-ergonomics claims
> below are `Declared` (a design rationale), to be confirmed `Empirical` by the parser + a streaming
> harness if/when enacted. No status of any other decision moves.

## 1. The decision (proposed)

Give the three ASCII delimiters **disjoint, purpose-clear** roles:

| Glyph | Role | Example |
|---|---|---|
| **`:`** | **Ascription / binding** — `name : type` (param, field, let-ascription, effect-of). Unchanged from today. | `x: Binary{8}` |
| **`,`** | **Sibling separation** — separates *items within one component* (args, fields, type/const params, list elements). RFC-0037 FLAG-2 Option B (between same-kind siblings, trailing optional). | `f(a, b)`, `[T, U]`, `{N, M}` |
| **`;`** | **Component termination** — marks the **end of a component**: a top-level item, a statement/operation, a method/function definition, or any other complete sequence-component. The unambiguous end-of-component token. | `fn f(x: Binary{8}) => Binary{8} = x;` |

The principle: **`,` is internal, `;` is terminal.** A reader (and a parser) always knows whether the
next token continues the current component (`,`) or ends it (`;`).

## 2. Why — whitespace-independence and streaming

RFC-0037 already made **newlines formatting-only** and chose **`,`** for sibling delimiting, so the
grammar no longer depends on indentation. The one remaining reliance on whitespace is **where one
component ends and the next begins** — today that is inferred from a newline / the next item-opening
keyword. A `;` **component terminator** removes that last dependency:

- **Whitespace-free source is legal and unambiguous.** A whole program can be written on one line —
  `nodule d; fn a() => …  = …; fn b() => … = …;` — with `;` ending each component and `,` separating
  siblings. No newline or indentation carries meaning.
- **The streamed / incremental variant parses without lookahead-to-whitespace.** A streaming parser
  can emit a completed component the instant it sees `;`, rather than scanning ahead for the next
  item-opening keyword or a newline. This bounds parser state per component and makes partial /
  resumable parsing (and the on-the-wire streamed encoding) clean — the end-of-component is a *token*,
  not the *absence* of more tokens.
- **Pairs with the never-silent stance (G2).** An unterminated component (missing `;` at EOF / before
  the next component) is an explicit parse error, not a silently-accepted run-on.

## 3. Scope / open questions (for the design pass)

1. **Required vs optional `;`.** Is `;` **mandatory** at every component end (cleanest for streaming;
   strongest invariant), or **optional** where a newline / next-item-keyword already disambiguates
   (gentler migration)? Recommendation to settle: **mandatory** for the streamable guarantee, with the
   formatter inserting it canonically so hand-written omissions are a single auto-fix, not a hard stop.
2. **Which components terminate with `;`.** Top-level items (`fn`/`type`/`trait`/`impl`/`use`/`default`)
   for sure. Do *expression-internal* sequences (e.g. a future statement-sequence, `let … ; …`, or
   match-arm bodies) also use `;`, or do arms stay `,`/`}`-delimited? (Today the surface is
   expression-based with `let … in …`; `;` as a *let/statement* sequencer is a larger question.)
3. **Interaction with `}`.** A block close `}` already terminates; is a trailing `;` before/after `}`
   required, optional, or forbidden? (Rust-style "`;` inside, `}` ends the block" is one model.)
4. **Corpus + formatter migration.** Enacting mandatory `;` is a corpus-wide migration (every component
   gains a terminator) — mechanical, like the RFC-0037 arrow/bracket migration, and formatter-insertable.
5. **Relationship to RFC-0037 FLAG-2.** This *is* the resolution of "what is the terminal delimiter" that
   FLAG-2 left at the principle level. If ratified, fold the `;` rule into RFC-0037 (append-only) /
   RFC-0030's concrete grammar, and regenerate `mycelium.ebnf`.

## 4. Disposition

Captured as **Draft** at the maintainer's request. **Not implemented** — the RFC-0037 grammar epic
(the `[T]`/`{N}`/`=>`/`0t` enactment) is complete and green **without** `;`; adding the `;` terminator is
a **follow-on** decision that should be ratified (and questions §3.1–§3.4 settled) before a second
grammar-migration wave. Recommended sequencing: land the RFC-0037 epic first, then a focused
`;`-terminator wave (parser + formatter + corpus migration + EBNF regen) once this note is ratified.

## 5. §3 resolution — the mandatory-terminator rule (M-818, implemented Rust-first; append-only)

> **Posture (VR-5/G2).** This section *settles* the §3 open questions and records the **mandatory**
> form as **implemented (Rust-first), pending the orchestrator's `Accepted → Enacted` step** at the
> `main` landing (house rule #3 — a leaf does not flip the status itself). The streaming-ergonomics
> claims stay `Declared` until a streaming-parser harness confirms them `Empirical`.

The simplest *consistent* rule — chosen so the surface has **one** terminal token per component with
**zero** special-casing:

1. **§3.1 Required vs optional → MANDATORY.** `;` is required at the end of **every** component; a
   missing one is an explicit, never-silent `ParseError` (G2) naming the component and where the `;`
   belongs. (There is **no** newline-equivalent: the rule is uniform and lookahead-free, which is
   precisely what the streaming guarantee needs.) The formatter inserts `;` canonically, so a
   hand-omitted terminator is a one-line auto-fix, not a hard wall in practice.
2. **§3.2 Which components terminate.** Exactly these: the **nodule header**; every **top-level item**
   (`use`/`default`/`type`/`trait`/`impl`/`fn`/`object`/`lower`/`derive`, incl. `pub`/`thaw`/`@tier`
   forms); every **trait signature**; every **`impl` / inherent-impl method**; and every **`object`
   member** (the constructor clause — already required — plus each `via`/`impl`/`fn` member).
   **Expression-internal sequences are unchanged**: the surface stays expression-based with
   `let … in …`; `;` is **not** a statement/`let` sequencer in v0 (that larger question is left open,
   to be revisited only if a statement layer is ever added). `match` arms remain `,`-separated and
   `}`-delimited — `,` is internal, `;` is terminal.
3. **§3.3 Interaction with `}` → UNIFORM: a `}`-closed block still takes the trailing `;`.** Every
   component ends with exactly one `;` *regardless of how its body ends* — so `trait T { … };`,
   `impl … { … };`, and `object … { … };` each carry a `;` **after** the closing `}`. This is
   deliberately **not** the Rust model ("`;` inside, `}` ends the block"): the streaming guarantee
   wants the end-of-component to always be the `;` *token* (never the *absence* of more tokens), and a
   single uniform rule is simpler to teach, parse, and stream than a `}`-vs-`;` special case. The
   terminator adds **no AST node** (it is purely a boundary), so it is observationally inert beyond
   making the boundary explicit.
4. **§3.4 Corpus + formatter migration.** Mechanical and complete (M-818): the whole `.myc` corpus,
   every in-test Mycelium program string across the workspace, the conformance accept fixtures (each
   gains `;`) and reject fixtures (each gains the header/leading `;` so it still fails for its
   *intended* reason), and a **new** reject fixture
   (`reject/29-missing-semicolon-terminator.myc`) for a missing terminator. `mycfmt` and
   `expand_to_source` emit `;` canonically; `mycelium.ebnf` updated to the mandatory productions.
5. **§3.5 Relationship to RFC-0037 FLAG-2.** This is the resolution of "what is the terminal
   delimiter" FLAG-2 left at the principle level; the `;` rule should be folded into RFC-0037 /
   RFC-0030's concrete grammar at ratification (append-only).

**Definition of Done (this resolution).** `;` is the enforced component terminator; a missing one is
an explicit error (G2); fully whitespace-free source parses; `mycfmt`/`expand_to_source` emit `;`
canonically; the whole corpus + test corpus + conformance fixtures are migrated and the change-scoped
gate is green. (`--flatten` / `myc --stream` are **separate** issues — M-819/M-820 — and out of scope
here.)
