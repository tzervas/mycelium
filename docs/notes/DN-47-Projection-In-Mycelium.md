# Design Note DN-47 — Projection in Mycelium (research + DN/RFC)

| Field | Value |
|---|---|
| **Note** | DN-47 |
| **Status** | **Draft (2026-06-27) — capture-only; frames the question, decides nothing.** Records the maintainer's intent to research what "projection" is in software engineering and programming languages more broadly, and how it relates to Mycelium — with a primary open question being whether RFC-0021 (the Semantic-Level Projection Framework, Enacted) already covers the concept in the broader sense, or whether something distinct is absent or complementary. No design position is taken; all are open. |
| **Feeds** | RFC-0021 (the Projection Framework — the primary reference to be clarified relative to this note's questions). The research deliverable of this note feeds a future DN and possibly a superseding or complementary RFC. |
| **Date** | June 27, 2026 |
| **Decides** | *Nothing.* Capture-only — frames the research questions. All positions (whether broader projection is the same as RFC-0021's notion, whether it is absent, whether it is wanted) are open. |
| **Task** | `rsm` kickoff, F3. Deliverables (later): research record + a DN; possibly an RFC. |

> **Posture (transparency rule / VR-5 / G2).** This note is **Declared** — it captures a
> research intent. RFC-0021 is cited by its current status (Enacted for the framework); this
> note does not re-decide or re-interpret RFC-0021. Anything that sounds like a design
> position below is an **open question**, not a claim.

---

## §1 The framing (maintainer's intent)

Explore *what projection is* in software engineering and programming languages broadly, and *how
it works*; then assess whether **Mycelium has it**.

A **critical preliminary**: RFC-0021 "Semantic-Level Projection Framework" (`docs/rfcs/RFC-0021-Semantic-Level-Projections.md`) already exists and is **Enacted (framework)**. RFC-0021's central claim is that "a projection is a *total, inspectable function* from the L1/L2 node structure of a content-addressed definition to a rendered surface … a view — it does not create a second source of truth and cannot change meaning." The **primary open question for this note** is:

> Is RFC-0021's notion of projection the **same** notion as "projection" in the broader SE/PL
> literature — a **subset** of it — or something **adjacent** that merely shares the name?

This question must be answered before any of the downstream questions (below) can be addressed
honestly. The answer is not assumed; this note captures the research task.

**If projection in the broader sense is absent from Mycelium** (i.e. RFC-0021 covers only a
subset or adjacent concept, and the fuller notion is not yet present): do we want it, how would we
implement it, what would it comprise, what are the drawbacks and considerations, and what are the
**2nd / 3rd / 4th-and-beyond-order effects** of having or not having it?

**If projection in the broader sense is present** (i.e. RFC-0021 captures it fully or RFC-0021 is
that notion): can we **prove** it, and can we make it **more economic / performant / safe /
secure**?

Deliverables (later, `Declared`): a research record + a DN capturing the findings; possibly an
RFC if a new design direction is warranted.

## §2 Relationship to existing docs

- **RFC-0021** (`docs/rfcs/RFC-0021-Semantic-Level-Projections.md`) is the primary reference.
  Status: Enacted (framework) as of 2026-06-21. RFC-0021 defines projections as total inspectable
  functions from the L1/L2 node structure to a rendered surface, with invariants P1–P6, the
  `Projection` interface + registry, the `LlmCanonical` design, and the supersession mechanism
  (§4.7). The research question of this note is whether that notion coincides with, subsumes, or
  is a proper subset of "projection" in the broader SE/PL sense.
- **RFC-0006 §3** (L3 projection layer) and **RFC-0020** (the text grammar as one projection
  among several) are context for RFC-0021's design. They do not themselves define "projection" in
  the broader sense — they apply RFC-0021's framework.
- **RFC-0001 §4.6** (content-addressed identity) is the substrate RFC-0021 projects over. The
  relationship between content-addressing and projectability is a potential grounding for
  a `Proven` claim about what projection over content-addressed artifacts can and cannot do.
- **G2 (never-silent)** and **VR-5** apply: if the research concludes RFC-0021's notion is
  narrower than the broader one, that gap must be stated plainly, not softened.

## §3 Open Questions

**OQ-1.** What is "projection" in the broader SE/PL literature? The term has uses in:
relational database theory (projection of a relation onto a subset of columns), functional
programming (lens / prism / optic theory), type theory (projection out of a product type),
program analysis (abstraction / concretization pairs in abstract interpretation), and
language-workbench / structured-editing contexts. Which of these senses is the maintainer's
intent, or is it all of them? The research record must survey the landscape before asserting a
relationship.

**OQ-2.** Is RFC-0021's "total inspectable function from L1/L2 node structure to a rendered
surface" the same notion as any of the above? If so, which? If not, what is its closest relative?

**OQ-3.** Does Mycelium have projection in any of the senses the research identifies as distinct
from RFC-0021? If so: where, and how? If not: is the absence intentional, an oversight, or a
future-design question?

**OQ-4.** If a broader or distinct projection notion is wanted: what would it require at the
language level (new syntax, new Core IR nodes, new invariants)? What are the 2nd/3rd/4th-order
effects — on the type system, on the lowering, on the swap/certification machinery, on
ergonomics?

**OQ-5.** If RFC-0021's projection is the full notion: can P1–P6 be proven (rather than
`Declared`/`Empirical`) over the content-addressed substrate? What would the proof obligation look
like, and is it tractable?

**OQ-6.** What are the performance, safety, and security implications of making projection
"more economic / performant / safe / secure" (the maintainer's framing)? Are these improvements
to RFC-0021's existing mechanism, or do they imply a new design?

**OQ-7.** If the research concludes that RFC-0021 already covers the broader notion fully: should
this note be resolved without a new RFC, or does the performance/safety improvement question
warrant a separate design document?

## §4 Definition of Done (gate to move Draft → Proposed)

- A **research record** is produced surveying "projection" in the relevant SE/PL literature
  (at minimum: relational theory, optic theory, structured-editing / language-workbench uses),
  with honest grounding labels on each survey claim.
- OQ-1 is answered: the relevant sense(s) of "projection" are enumerated, cited, and distinguished.
- OQ-2 is answered: RFC-0021's notion is placed within that taxonomy, with an honest label
  (`Proven` / `Empirical` / `Declared`) on the placement claim.
- OQ-3 is answered: the presence/absence determination is stated with its grounding basis.
- A direction is identified (new RFC, amending DN, or resolve-without-action) and the maintainer
  ratifies the direction before the note moves to Proposed.

---

## Changelog

- 2026-06-27 — **Draft** (`rsm` kickoff, F3 / DN-47). Capture-only: frames the projection
  research question, names RFC-0021 as the critical primary reference, and lists OQ-1 through
  OQ-7 as the undecided points. No design decided; no position taken on whether RFC-0021 covers
  the broader notion.
