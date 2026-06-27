# Design Note DN-48 — L4 and the "Reveal" Lowering (research + DN/RFC)

| Field | Value |
|---|---|
| **Note** | DN-48 |
| **Status** | **Draft (2026-06-27) — capture-only; frames the question, decides nothing.** Records the maintainer's intent to consider whether an L4 layer is appropriate in Mycelium's layered architecture, what it would mean, and how layers "reveal" (lower to the next-lower level) — including the open question of whether L4 can reveal straight to L0. No design position is taken; all are open. |
| **Feeds** | The layered-lowering model (RFC-0006 §3, RFC-0007, RFC-0011, RFC-0030, the L0/L1/L2/L3 ratification arc). Intersects DN-49 (F6) as the "L3 review + the L4 question" pass. |
| **Date** | June 27, 2026 |
| **Decides** | *Nothing.* Capture-only — frames the research questions. Whether L4 exists, what it comprises, and how "reveal" works are all open. |
| **Task** | `rsm` kickoff, F4. Deliverables (later): DN/RFC + research. Referenced by DN-49 (F6) as one of the post-critical quality passes. |

> **Posture (transparency rule / VR-5 / G2).** This note is **Declared** — it records a
> future-design investigation. The layer model (L0–L3) is cited as the existing ratified
> basis; no new layer or lowering mechanism is asserted or implied by this note. The
> "reveal" term is the maintainer's chosen vernacular for lowering; this note records it
> without committing to any design.

---

## §1 The framing (maintainer's intent)

The current layer cake in Mycelium (RFC-0006 §3, `.claude/memory/language-execution.md`) runs:

> **L0** (frozen, trusted — the Core IR + reference interpreter) →
> **L1** (kernel calculus — RFC-0007) →
> **L2** (surface term language — RFC-0020) →
> **L3** (concrete surface grammar — RFC-0030)

Each layer **reveals** (the maintainer's chosen vernacular for lowering / elaboration to the
next-lower level) downward. Currently the reveal chain is L3→L2→L1→L0; L0 is the trusted base
that interpretation and compilation both root in.

**The investigation this note captures:**

1. **Is an L4 layer appropriate?** What would it look like, what would it mean, and what would
   it comprise? (What exists at L4 that is not already at L3 or expressible through L3?)

2. **How does "reveal" work as a concept and a mechanism?** The maintainer's framing: lowering
   is *mechanized, schematized, highly repeatable*, and all of L4/L3/L2/L1 are **lowered to L0**
   for both interpretation-time and compile-time. This raises an important structural question:

3. **Can L4 reveal straight to L0?** If lowering is highly regular and all upper layers
   ultimately bottom out at L0 anyway, a direct L4→L0 reveal may be coherent — skipping
   intermediate layers where they add no distinct semantic content. This is not asserted; it is
   the open structural question to investigate.

This note also intersects DN-49 (F6): the "L3 review + the L4 question" is listed there as one
of the post-critical quality passes, to be done as part of the same lowering-architecture review.

## §2 Relationship to existing docs

- **RFC-0006** (`docs/rfcs/RFC-0006-Surface-Language-and-Term-Layering.md`) §3 defines the
  current L3 projection layer and the layer cake. The `reveal` / lowering mechanism lives there
  and in RFC-0007/RFC-0011. This note extends the question to L4.
- **RFC-0007** (`docs/rfcs/RFC-0007-L1-Kernel-Calculus.md`) defines L1 — the kernel calculus,
  the ten-node budget, typing, totality. L4 (if it exists) would eventually lower through or
  past L1 to L0.
- **RFC-0011** (`docs/rfcs/RFC-0011-L0-Match-and-L1-in-Core-IR.md`) covers Construct/Match/
  Lam/App/Fix in L0 — the concrete joint between L0 and L1. The L4→L0 direct reveal question
  hinges on whether intermediate nodes (L1/L2/L3) are semantically load-bearing or purely
  syntactic.
- **RFC-0030** (`docs/rfcs/RFC-0030-Concrete-Surface-Grammar-L3-Ratification.md`) ratified L3.
  The "L3 review" in DN-49 §1 references the current L3 surface; any L4 decision would
  supersede or extend that review.
- **RFC-0004** (`docs/rfcs/RFC-0004-Execution-Model-and-Stable-Component.md`) and **RFC-0029**
  (`docs/rfcs/RFC-0029-AOT-Optimization-Codegen-Maturity-and-JIT.md`) cover the execution paths
  (interpretation + AOT). The reveal-to-L0 question affects both: interpretation walks L0 terms;
  AOT compiles from L0 via MLIR→LLVM. A layer that bypasses intermediates must still produce
  valid L0.
- **DN-49** (`docs/notes/DN-49-Post-Critical-Quality-Passes.md`) references this note as one
  of the passes in the post-critical quality arc; see there for sequencing.
- **`.claude/memory/language-execution.md`** provides the orientation map for the layer model
  (advisory / non-normative; source + RFC are ground truth).

## §3 Open Questions

**OQ-1.** What is "L4" in the context of Mycelium's layer cake? What semantic content would
live at L4 that is not already representable at L3? Is L4 a higher-level surface (macro,
meta, or module-level constructs), a different abstraction axis (e.g. effect-typed or
annotated surface), or something else?

**OQ-2.** The maintainer's framing says "reveal" is the chosen vernacular for lowering. Is this
a new term that will enter the Mycelium lexicon, or is it already used somewhere? (Cross-check
against `docs/Glossary.md` and `docs/notes/Lexicon-Reference.md` — the term does not appear
in those refs at time of writing, `Declared`.) If it is a new lexicon entry, should it be added
via the normal DN/glossary process?

**OQ-3.** Can L4 reveal straight to L0? The argument (as framed by the maintainer): lowering is
mechanized / highly repeatable, and all upper layers bottom out at L0 anyway, so a direct reveal
may be coherent. The counter-argument (not asserted, but to be investigated): intermediate layers
may be load-bearing as semantic checkpoints, not just syntactic sugar — eliminating them could
lose guarantees the type system / checker relies on at those levels. Which is correct? What is
the honest basis for either position?

**OQ-4.** If intermediate layers are not semantically load-bearing (i.e. L4→L0 direct reveal is
coherent): does that change the architecture of the other layers (L1/L2/L3) retroactively, or are
they independently warranted even without L4?

**OQ-5.** The "reveal" mechanism must be "mechanized, schematized, highly repeatable" (the
maintainer's framing). What does that mean concretely — is it a function, a macro system, a
translation-validation certificate (RFC-0002), or something else? Can the reveal be certified
(RFC-0002 style)?

**OQ-6.** What are the 2nd-order effects of adding L4 on the trusted core (KC-3)? The trusted
core is deliberately minimal (DN-39 — zero promotions ratified). Does L4 require any kernel
change, or does it elaborate entirely above L0?

**OQ-7.** Sequencing relative to DN-49 (F6): the "L3 review + L4 question" is a post-critical
quality pass. Is there value in investigating L4 earlier (before full 1.0), or does the answer
depend on the L3 review's conclusions?

## §4 Definition of Done (gate to move Draft → Proposed)

- A **research record** is produced surveying at least one precedent for a multi-layer language
  architecture with direct top-to-bottom lowering (e.g. GHC's Core / STG / Cmm pipeline;
  MLIR's multi-level IR), with honest grounding labels.
- OQ-1 and OQ-3 are answered with a grounded proposal or a grounded "not yet determinable"
  conclusion.
- The "reveal" vernacular is either confirmed as a new lexicon entry (and a glossary ticket filed)
  or grounded in an existing term.
- A direction is identified (new RFC, extension to existing RFC-0006/RFC-0007, or defer)
  and the maintainer ratifies the direction.

---

## Changelog

- 2026-06-27 — **Draft** (`rsm` kickoff, F4 / DN-48). Capture-only: frames the L4 and
  reveal-lowering research questions, places them in the existing layer cake (L0–L3), names
  RFC-0006/0007/0011/0030 as the primary references, and lists OQ-1 through OQ-7. No design
  decided; "reveal" recorded as the maintainer's chosen vernacular for lowering (not yet in
  the Glossary). References DN-49 (F6) for sequencing.
