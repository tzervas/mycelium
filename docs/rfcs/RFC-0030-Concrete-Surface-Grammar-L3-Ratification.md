# RFC-0030 — Concrete Surface Grammar & L3 Ratification

| Field | Value |
|---|---|
| **RFC** | 0030 |
| **Status** | **Draft** (2026-06-23) |
| **Type** | Normative / foundational (once Accepted) — grammar spec + conformance; no L0 or L1 kernel change |
| **Date** | 2026-06-23 |
| **Feeds** | E11-1 (surface-language completeness, M-706) |
| **Decides** | The committed EBNF reference grammar for the full Mycelium surface language (L3), resolving RFC-0006 open questions Q3 and Q8, and ratifying the concrete syntax as the canonical L3 surface. |
| **Depends on** | RFC-0006 §3–§4.3/§8 (layering L0–L3, invariants S1–S6, grammar discipline, open questions Q3/Q8); RFC-0007 §4.1–§4.8 (L1 kernel calculus — the elaboration target); RFC-0020 §4 (L2 surface term language — the desugaring layer between L3 text and L1); RFC-0025 (operator syntax — the infix-sugar grammar, once decided; RFC-0030 integrates or supersedes that choice); `docs/spec/grammar/mycelium.ebnf` (the current committed v0 grammar, the starting point) |
| **Task** | E11-1 (M-706) |

> **Posture (honesty rule / VR-5).** This is a **planning stub** — scope, user stories, decision
> space, and open questions only. The committed EBNF grammar lives at
> `docs/spec/grammar/mycelium.ebnf`; it is the current normative oracle for the conformance corpus.
> This RFC will extend and ratify that grammar once the surface-language completeness work (E11-1)
> settles. Nothing is enacted here. Status: **Draft** (append-only house rule #3).

---

## 1. Problem / Goal

RFC-0006 committed the grammar discipline (§4.3) and acknowledged two open questions that block
a full L3 ratification:

- **Q3 — implicit vs explicit representation** (the RFC-0012 ambient-paradigm tension): should the
  concrete surface expose the ambient representation implicitly (ergonomic) or require explicit
  `swap`/type annotations everywhere (honest/auditable)? DN-07 §3-Q3 accepted a direction; the
  concrete spelling is not yet committed.
- **Q8 — `unsafe`-class construct spelling**: what is the surface syntax for constructs that exit
  the verifiable core (the `wild` FFI escape, maturation bypass, unsafe arithmetic)? The spelling
  must be memorable, un-missable, and consistent with the lexicon (DN-02/DN-03), but has not been
  committed.

Additionally, the v0 grammar in `mycelium.ebnf` is a **working L1-facing fragment** — it covers
what is implemented in `crates/mycelium-l1` today, but it is not a committed **full-language**
grammar. It does not yet include: the complete L2 surface forms (RFC-0020 §4.2–§4.5 inference
sugar, derived forms, the `grow` construct); the operator-sugar grammar (RFC-0025, pending);
effect-annotation syntax (RFC-0014 `!{…}` — in the parser but not in the EBNF artifact); or a
ratified prose + EBNF for the phylum-level organization (M-662 landed the `phylum` construct, but
the EBNF update lags).

The goal of this RFC is to close all of these gaps and produce a **single, committed, machine-readable
EBNF reference grammar** that is the L3 oracle — the document tooling authors, compiler engineers,
and downstream language implementors cite as the normative source of truth.

---

## 2. User stories / motivating use cases

- As a **compiler engineer** implementing a parser for Mycelium, I want a single normative EBNF
  document that covers the complete surface language, so that I do not have to reconcile the
  partial grammar in `mycelium.ebnf` against scattered RFC sections and implementation details in
  `crates/mycelium-l1/src/parse.rs`.
- As a **tool author** writing a syntax highlighter (DN-24), formatter (`mycfmt`), or language
  server, I want the grammar to be machine-readable and complete, so that I can derive tokenization
  and parse rules mechanically without reading prose.
- As a **stdlib author** writing real programs in Mycelium and hitting parser edge cases, I want
  an authoritative reference for ambiguous surface forms, so that I can report bugs against a spec
  rather than against implementation behavior.
- As a **downstream app developer** evaluating Mycelium for production use, I want a ratified
  concrete syntax I can rely on not changing incompatibly, so that I can invest in tooling and
  training without fear of surface churn.
- As an **AI co-author agent**, I want the grammar in a machine-readable form that I can include
  in my context window, so that I can generate syntactically correct Mycelium programs without
  hallucinating surface forms.
- As a **language user** writing code that uses the `wild` construct or ambient-representation
  sugar, I want the RFC-0006 Q3/Q8 questions resolved with clear, committed spellings, so that I
  do not write code against a pending decision that may change under me.

---

## 3. Scope & decision space

**In scope:**
- Resolving RFC-0006 **Q3** (implicit/explicit representation tension — commit the ambient
  surface spelling, citing RFC-0012 §4 as the enacted model).
- Resolving RFC-0006 **Q8** (unsafe-class spelling — commit `wild { … }` or an alternative as
  the normative surface, with the `@std-sys` context gate per M-661).
- Extending `mycelium.ebnf` to a **complete L3 reference grammar** covering: all L2 surface forms
  (RFC-0020); effect annotations (`!{…}`, RFC-0014 §3.4 — already in the parser, lagging in the
  EBNF); the phylum/nodule organization (`phylum`, `pub`, `use` glob — M-662); the operator-sugar
  grammar if RFC-0025 adopts it; the guarantee-index annotation (`@g`, RFC-0018); the `for` sugar
  (RFC-0007 §4.8 r2); and the `colony` expression (RFC-0008 §4.7, M-666).
- A **conformance-corpus update**: new accept/reject fixtures for every grammar extension, using
  the same `docs/spec/grammar/conformance/` structure.
- A **ratification statement**: explicit prose declaring this grammar the canonical L3 oracle,
  superseding the "v0 working fragment" status of `mycelium.ebnf` (RFC-0006 §3 + §10).

**Out of scope:**
- L0 or L1 changes — this RFC is grammar only; all semantic changes belong in the relevant
  layer RFC.
- User-defined operators or extensible precedence — deferred per RFC-0025.
- Grade polymorphism / refinement types (RFC-0018 stage 1b/2) — grammar hooks reserved, full
  syntax deferred.
- The co-equal structured-projection layer (M-380, FR-S5) — RFC-0021 owns that.

---

## 4. Definition of Done

- RFC-0006 open questions Q3 and Q8 are answered with committed normative text, recorded in this
  RFC, and referenced append-only from RFC-0006 (not rewriting RFC-0006).
- `docs/spec/grammar/mycelium.ebnf` is updated to a complete, ratified L3 reference grammar with
  a **SURFACE STATUS** header updated to "ratified" (not "v0 working fragment").
- Every grammar production is covered by at least one accept and one reject conformance fixture in
  `docs/spec/grammar/conformance/accept/` and `.../reject/`.
- `crates/mycelium-l1/src/parse.rs` and the EBNF are in sync: every accepted production in the
  grammar parses without error; every rejected production is refused with an explicit error message
  (G2 — never-silent).
- RFC-0020's deferred carve-outs (§4.2 polymorphic instantiation, §4.5 `grow`-derived traits)
  are either landed or explicitly noted as `Residual` entries in the grammar with a forward
  reference to their tracking issue.
- RFC-0025 is either merged into this grammar (if Accepted by the time this RFC drafts) or
  recorded as a pending extension (if still Draft) — one of the two, never silent.
- RFC-0030 status moves **Draft → Proposed** (maintainer review) and then **Proposed → Accepted**
  (ratification) — never skipping (house rule #3).

---

## 5. Open questions

1. **RFC-0006 Q3 resolution:** which ambient-representation spelling does the surface commit?
   Implicit (no annotation, paradigm inferred from context), explicit (`in Binary { … }` block),
   or a hybrid? DN-07 §3-Q3 accepted the direction; the concrete spelling is TBD.
2. **RFC-0006 Q8 resolution:** is `wild { … }` the committed unsafe-class spelling, or does this
   RFC explore alternatives (`unsafe { … }` for familiarity, `audited { … }` for honesty)? The
   `@std-sys` context gate (M-661) is committed — the spelling of the inner block is open.
3. **EBNF artifact ownership:** should `mycelium.ebnf` remain a separate file extended by this
   RFC, or should this RFC inline the complete grammar and `mycelium.ebnf` become a generated
   extract? One canonical source avoids drift.
4. **Relation to RFC-0025:** if RFC-0025 is not yet Accepted when RFC-0030 drafts, how are the
   operator-expression productions handled — as a guarded extension, as `Residual` placeholders,
   or by sequencing RFC-0030 after RFC-0025?
5. **Conformance-corpus scale:** the current corpus has O(tens) of fixtures. A complete grammar
   ratification implies O(hundreds). How is this maintained without becoming a burden? A
   grammar-based fuzzer (e.g., `cargo-fuzz` + a grammar-guided generator) may replace hand-written
   fixtures at scale.
6. **Versioning:** once this grammar is ratified, what is the compatibility policy? Any grammar
   extension is an RFC? Or are additive-only extensions permitted without a new RFC, per
   RFC-0006's "refined append-only" discipline?

---

## 6. Grounding / honesty

This stub cites RFC-0006 (the grammar discipline and the Q3/Q8 open questions), RFC-0007 (the L1
elaboration target), RFC-0020 (the L2 surface layer), and `docs/spec/grammar/mycelium.ebnf` (the
current working grammar). The claim that Q3 and Q8 are open is grounded in RFC-0006 §8 and the
ADR-021 gate B1 note ("surface-spelling open items (RFC-0006 Q3/Q8) are `1.x`"). No normative
choice is made here (VR-5).

---

## Meta — changelog

- **2026-06-23 — Draft stub created.** Scope, user stories, decision space, Definition of Done,
  and open questions captured as a planning stub. Feeds E11-1 (M-706). Relates RFC-0006 (Q3/Q8),
  RFC-0007, RFC-0020, and the current `mycelium.ebnf`. Decides nothing normatively. Status:
  **Draft** (VR-5 / house rule #3).
