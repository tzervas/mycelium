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

> **Posture (honesty rule / VR-5).** Still **Draft** — and honestly so: a *full L3 ratification*
> (this RFC's DoD) requires a **complete** grammar, which cannot be claimed until the RFC-0020 L2
> carve-outs (M-707) and the angle-bracket operators (M-745) land. This revision does the work that
> **is** ready: it integrates the M-705 operator grammar (now in `mycelium.ebnf`), **proposes** the
> Q8 resolution (§4.1), and **corrects** the Q3 framing this stub originally got wrong (§4.2). The
> committed EBNF at `docs/spec/grammar/mycelium.ebnf` remains the normative oracle for the
> conformance corpus. Status stays **Draft**; **Draft → Proposed is gated on M-707 + M-745** (named
> in §4.3). Nothing is ratified here (append-only house rule #3).

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

## 4. Partial decision (this revision — proposed, not ratifying)

Full L3 ratification is **gated** (§4.3); this revision commits only the ready pieces.

### 4.1 RFC-0006 Q8 — unsafe-class spelling (proposed: ratify `wild`)

**Proposal:** the committed unsafe-class surface spelling is **`wild { … }`**, gated by the
`@std-sys` nodule-header attribute and the `!{ffi}` effect (M-661). This ratifies the *implemented*
status quo rather than inventing a spelling: `wild` is the DN-02-themed term, it is denied by
default (a `wild` outside an `@std-sys` nodule is a hard `CheckError`, G2), its enclosing `fn` must
declare `!{ffi}` (the `wild` block is the `ffi` effect source — RFC-0014/M-660), and its body is the
trusted/opaque FFI escape (audited, not verified — ADR-014/VR-5). The alternatives DN-23/RFC-0006
§8-Q2 floated (`unsafe { }` for familiarity, `audited { }` for honesty) are **declined**: `wild`
already carries the themed-lexicon meaning and is in the grammar + checker; changing it now would be
churn for no honesty gain. *(Normative once this RFC is Accepted; until then, the spelling is the
implemented one regardless.)*

### 4.2 RFC-0006 Q3 — correction of this stub's framing

The original stub (§1) described Q3 as "implicit vs explicit *representation* (the RFC-0012
ambient-paradigm tension)". **That mis-states RFC-0006's Q3.** RFC-0006 Q3 is the **LR-6
guarantee-grading mechanism** (the graded-coeffect modality over the guarantee lattice — RFC-0006
§8 "Q3 (LR-6 mechanism)"), and RFC-0006 r5 records it as **already discharged by RFC-0018** (grading
stage-1a, Enacted; M-663). The *representation* implicit/explicit question is governed by **RFC-0012
(ambient paradigm, Enacted)**, not Q3. So there is **no open Q3 for this RFC to resolve** — both the
guarantee-grading mechanism (RFC-0018) and the ambient-representation surface (RFC-0012) are settled
and reflected in the grammar (`@ strength` annotation; `default paradigm` / `with paradigm`). This
correction is recorded honestly (house rule #4 — ground every claim) rather than silently resolving
the wrong question.

### 4.3 What gates the move to Proposed

This RFC cannot honestly move Draft → Proposed (a *complete, ratified* L3 grammar) until:

- **M-707** — the RFC-0020 L2 carve-outs (§4.2 polymorphic-instantiation sugar, §4.5 `grow`-derived
  forms) land or are folded into the grammar as explicit `Residual` productions with forward refs.
- **M-745** — the angle-bracket operators `< <= > >= << >>` (RFC-0025 §4.3), so the operator grammar
  is whole.

What **is** already integrated into `docs/spec/grammar/mycelium.ebnf` and need not wait: the
operator-expression grammar (`op_expr` … `unary_expr`; RFC-0025/M-705), effect annotations (`!{…}`),
the phylum/nodule organization (`phylum`/`pub`/`use` glob; M-662), the `for` sugar, the `colony`
expression (M-666), and the `@ strength` guarantee annotation. RFC-0025 is recorded as **integrated**
(its productions are in the EBNF), not as a pending placeholder (resolving this RFC's open
question #4 for the operator case).

---

## 5. Definition of Done

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

## 6. Open questions

> **Note (this revision).** Questions 1 and 2 are **answered in §4** and are retained here only for
> the record (append-only); the framing of the original #1 was itself incorrect — see §4.2.

1. **RFC-0006 Q3 — RESOLVED/CORRECTED (see §4.2):** this stub originally framed Q3 as the
   ambient-*representation* spelling, which **mis-states** RFC-0006's Q3 (the LR-6 guarantee-grading
   mechanism, already discharged by RFC-0018). The representation surface is RFC-0012's (Enacted:
   `default paradigm` / `with paradigm` + the `@ strength` annotation). There is **no open Q3** for
   this RFC. *(Historical wording: "which ambient-representation spelling does the surface commit —
   implicit, explicit `in Binary { … }`, or hybrid?" — superseded by this correction.)*
2. **RFC-0006 Q8 — PROPOSED (see §4.1):** `wild { … }` is proposed as the committed unsafe-class
   spelling (gated by `@std-sys` + `!{ffi}`); the `unsafe { … }` / `audited { … }` alternatives are
   declined. Becomes normative on maintainer ratification of this RFC.
3. **EBNF artifact ownership:** should `mycelium.ebnf` remain a separate file extended by this
   RFC, or should this RFC inline the complete grammar and `mycelium.ebnf` become a generated
   extract? One canonical source avoids drift.
4. **Relation to RFC-0025 — ANSWERED (see §4.3):** RFC-0025 (Proposed) is **integrated** — its
   operator-expression productions (`op_expr` … `unary_expr`) are committed in `mycelium.ebnf` now,
   not held as `Residual` placeholders. The angle-bracket-operator tail (M-745) is the only pending
   piece of the operator grammar.
5. **Conformance-corpus scale:** the current corpus has O(tens) of fixtures. A complete grammar
   ratification implies O(hundreds). How is this maintained without becoming a burden? A
   grammar-based fuzzer (e.g., `cargo-fuzz` + a grammar-guided generator) may replace hand-written
   fixtures at scale.
6. **Versioning:** once this grammar is ratified, what is the compatibility policy? Any grammar
   extension is an RFC? Or are additive-only extensions permitted without a new RFC, per
   RFC-0006's "refined append-only" discipline?

---

## 7. Grounding / honesty

This stub cites RFC-0006 (the grammar discipline and the Q3/Q8 open questions), RFC-0007 (the L1
elaboration target), RFC-0020 (the L2 surface layer), and `docs/spec/grammar/mycelium.ebnf` (the
current working grammar). The claim that Q3 and Q8 are open is grounded in RFC-0006 §8 and the
ADR-021 gate B1 note ("surface-spelling open items (RFC-0006 Q3/Q8) are `1.x`"). No normative
choice is made here (VR-5).

---

## Meta — changelog

- **2026-06-23 — Partial decision (M-706; stays Draft).** Integrated the M-705 operator grammar
  reference (now in `mycelium.ebnf`); **proposed** the Q8 unsafe-class spelling = ratify `wild`
  (gated by `@std-sys` + `!{ffi}`; §4.1); **corrected** the stub's Q3 mischaracterization — RFC-0006
  Q3 is the LR-6 guarantee-grading mechanism (discharged by RFC-0018), and the representation
  question is RFC-0012's, so there is no open Q3 here (§4.2). Status stays **Draft**: Draft → Proposed
  is **gated on M-707** (RFC-0020 L2 carve-outs) **+ M-745** (angle-bracket operators) (§4.3). No
  ratification, no tag upgrade (VR-5 / house rule #3).
- **2026-06-23 — Draft stub created.** Scope, user stories, decision space, Definition of Done,
  and open questions captured as a planning stub. Feeds E11-1 (M-706). Relates RFC-0006 (Q3/Q8),
  RFC-0007, RFC-0020, and the current `mycelium.ebnf`. Decides nothing normatively. Status:
  **Draft** (VR-5 / house rule #3).
