# RFC-0036 — Kernel & Primitives Consolidation (minimal + complete + frozen-toward-1.0)

| Field | Value |
|---|---|
| **RFC** | 0036 |
| **Status** | **Draft (2026-06-27) — capture-only; frames the question, decides nothing.** Records the maintainer's intent to consolidate the kernel and its primitives toward a frozen, locked, pinned 1.0 kernel — the most heavily tested, benchmarked, and chaos-engineered artifact in the project — with an explicit "what must be a primitive vs built on top" boundary review, and an open structural question of one kernel vs several. No design position is taken. |
| **Type** | Architecture — kernel boundary, primitive taxonomy, 1.0 freeze strategy. |
| **Date** | June 27, 2026 |
| **Depends on** | KC-3 (small-auditable-kernel constraint — CLAUDE.md house rule #5; `docs/Mycelium_Project_Foundation.md`); DN-39 (`docs/notes/DN-39-Kernel-Promotion-Review-KC3.md` — ratified: zero promotions, boundary UNCHANGED); RFC-0001 (`docs/rfcs/RFC-0001-Core-IR-and-Metadata-Schema.md` — the Core IR the kernel grounds); RFC-0003 (`docs/rfcs/RFC-0003-VSA-Submodule-Boundary.md` — the VSA submodule boundary). |
| **Coupled with** | RFC-0001 (Core IR); RFC-0003 (VSA boundary); RFC-0007 (L1 kernel calculus); DN-39 (the promotion review this RFC extends); the spore/phylum packaging model (ADR-013) if a multi-kernel split changes the phylum shape. |
| **Task** | `rsm` kickoff, F5. Deliverables (later): an RFC + research. |

> **Posture (transparency rule / VR-5 / G2).** This RFC is **Declared** — it captures a
> future-design intent without deciding any design question. The KC-3 constraint and DN-39
> ratification are cited as the existing basis; this RFC neither loosens nor re-interprets
> them. The model described in §1 (primitives in the kernel; kernel frozen-toward-1.0) is
> the maintainer's framing, recorded verbatim-in-intent, not a design claim. All structural
> questions are open.

---

## §1 The framing (maintainer's intent)

The kernel stays **minimal** — KC-3 (small-auditable-kernel), ratified by DN-39 with zero
promotions and the boundary unchanged. But "minimal" is not "weak": the maintainer's model is:

> **All primitives live in the kernel, and the kernel is the most heavily tested, benchmarked,
> and chaos-engineered artifact in the project — every condition and error mapped and handled —
> so it can be frozen, locked, and pinned as the 1.0 kernel, rarely if ever touched again.**

Small *because targeted*: only primitives, and only doing in primitives what should be done in
primitives; everything else builds on top. The kernel earns its freeze by being exhaustively
verified, not by being minimal in a way that leaves gaps.

**This RFC investigates two things:**

1. **The "what must be a primitive vs built on top" boundary review.** Given the freeze goal,
   the boundary between "belongs in the kernel" and "belongs in the stdlib/phylum layer" must be
   reviewed and settled. The DN-39 bar (four conjunctive clauses — foundational · unverifiable-
   from-outside · net-trust-reducing · small-and-auditable) is the existing tool for this
   boundary; this RFC's boundary review applies it systematically.

2. **One kernel, or several?** Open structural question: is the right shape a single `mycelium`
   kernel, or a family — e.g. a `mycelium` kernel + a binary/ternary kernel + an embedding kernel
   - a VSA/HDC kernel? The best approach is **TBD** (`Declared`); this RFC captures the question
   and the considerations, decides nothing.

## §2 Relationship to existing docs

- **KC-3** (CLAUDE.md house rule #5; `docs/Mycelium_Project_Foundation.md`) is the primary
  constraint: the kernel must remain small and auditable. This RFC works toward a version of KC-3
  that is **proven, not merely aspired to** — a kernel small enough to be exhaustively verified.
- **DN-39** (`docs/notes/DN-39-Kernel-Promotion-Review-KC3.md`, Accepted 2026-06-26) is the
  immediately prior boundary review: it reviewed whether any non-kernel functionality should be
  promoted in, and ratified zero promotions. This RFC extends that work in the outward direction:
  given the freeze goal, what *must* remain in the kernel, and what is better moved out?
- **RFC-0001** (`docs/rfcs/RFC-0001-Core-IR-and-Metadata-Schema.md`) defines the Core IR and
  content-addressing — the substrate the kernel grounds. Any kernel consolidation must preserve
  RFC-0001's invariants.
- **RFC-0003** (`docs/rfcs/RFC-0003-VSA-Submodule-Boundary.md`) draws the VSA submodule
  boundary. The multi-kernel question (OQ-3) directly affects whether the VSA machinery is a
  separate kernel or a phylum.
- **RFC-0007** (`docs/rfcs/RFC-0007-L1-Kernel-Calculus.md`) defines the L1 kernel calculus —
  the ten-node budget, typing, totality. This RFC's boundary review includes L1 as part of the
  kernel surface.
- **DN-34** (`docs/notes/DN-34-Rust-to-Mycelium-Transpiler-Strategy.md`) and the
  **self-hosting arc** (RFC-0031, RFC-0032) are downstream of a frozen kernel: self-hosting
  becomes tractable once the kernel is stable and pinned.

## §3 Open Questions

**OQ-1.** What is the complete set of primitives that "belong in the kernel" by the DN-39
four-clause bar? Is the current kernel surface already complete for 1.0, or are there gaps?
(The DN-39 review found zero *promotions* needed, but did not audit whether existing kernel
items should be *demoted*.)

**OQ-2.** Is there anything currently in the kernel that fails the DN-39 bar's conjunctive
clauses and should be moved to the stdlib or a phylum instead? The freeze goal makes this
question more urgent: anything that *should not* be frozen must be identified before the pin.

**OQ-3.** One kernel or several? The candidate split (maintainer's framing, `Declared`):
a `mycelium` kernel + a binary/ternary kernel + an embedding kernel + a VSA/HDC kernel.
What are the arguments for and against the split?
- *For:* each kernel can be frozen and verified independently; the VSA/HDC surface has
  different stability guarantees than the core language kernel; separate kernels can be
  combined compositionally.
- *Against:* more kernels = more surface to test and maintain; the split may violate KC-3
  by distributing the trusted base across multiple artifacts; the interaction between kernels
  becomes a new trust surface.
The honest answer is unknown; the research must explore this.

**OQ-4.** What does "frozen, locked, and pinned" mean concretely? A semantic version pin? A
content-hash commitment (RFC-0001 §4.6)? A policy commitment (no new primitives after 1.0
without a superseding RFC)? All three? The mechanism is open.

**OQ-5.** What does "most heavily tested, benchmarked, and chaos-engineered" mean as a
completeness criterion? The goal is "every condition and error mapped and handled" — how is
that criterion discharged? Is it a property-test exhaustion requirement, a chaos-test
specification, or something else?

**OQ-6.** Does the freeze goal interact with the RFC-0034 `fast` / `certified` mode split?
The kernel is the substrate for both modes; freezing it must not inadvertently constrain
future certification machinery. Does the freeze apply to the kernel's interface, its
implementation, or both?

**OQ-7.** What is the sequencing relative to the self-hosting arc (RFC-0031/RFC-0032)?
The kernel freeze is a prerequisite for stable self-hosting (you can't bootstrap reliably on
a moving target). Does this RFC need to land (at least as Accepted) before RFC-0032's
self-hosting work begins in earnest?

## §4 Definition of Done (gate to move Draft → Proposed)

- A **research record** is produced covering: (a) the multi-kernel question (OQ-3) with at
  least one precedent from a language with compositional kernel architecture; (b) the
  completeness criterion (OQ-5) with a grounded definition of what "every condition and error
  mapped" means for the Mycelium kernel.
- OQ-1 and OQ-2 are answered: the current kernel surface is audited against the DN-39 bar,
  with any demotion candidates identified (even if none are found — that is itself the result).
- OQ-3 is answered with a grounded proposal (one kernel or a specified split), with honest
  labels on which claims are `Proven`/`Empirical`/`Declared`.
- The freeze mechanism (OQ-4) is specified.
- The maintainer ratifies the kernel count and freeze mechanism before the note moves to
  Proposed.

---

> **Provenance.** Captures the maintainer's `rsm`-kickoff F5 workstream intent. KC-3 and
> DN-39 are the grounding basis for the boundary constraints; RFC-0001/0003/0007 are the
> primary references for the current kernel surface. All design questions are open — this
> RFC implements nothing and decides nothing at Draft.

### Changelog

| Date | Status | Note |
|---|---|---|
| 2026-06-27 | **Draft** | Initial capture (`rsm` kickoff, F5 / RFC-0036). Records the maintainer's kernel-consolidation and freeze intent, the one-vs-many kernel open question, and the "what must be a primitive" boundary review. KC-3 and DN-39 cited as the grounding basis. All design decisions open. Implements nothing. |
