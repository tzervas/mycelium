# Design Note DN-46 — Operationalizing the Honest-Insecurity Disclosure Corollary

| Field | Value |
|---|---|
| **Note** | DN-46 |
| **Status** | **Draft (2026-06-27) — capture-only; frames the question, decides nothing.** Records the follow-on work explicitly named in DN-44 §1.1: operationalizing the never-silent residual-insecurity discipline into a **standard disclosure block** and a **gate** that asserts every `wild`/FFI/intentional-escape surface carries one. No form or gate is designed here; all mechanism decisions are open. |
| **Feeds** | DN-44 §1.1 (the honesty corollary this note operationalizes). Supports the **never-silent rule (G2)** applied to security surfaces. Informs `/security-review` and `scripts/checks/`. |
| **Date** | June 27, 2026 |
| **Decides** | *Nothing.* Capture-only — frames the disclosure requirement and records the preference order stated in DN-44 §1.1. All design decisions (block format, gate mechanism, tooling) are open questions (§3). |
| **Task** | `rsm` kickoff, F2. Explicitly named in DN-44 §1.1 as a tracked follow-on. |

> **Posture (transparency rule / VR-5 / G2).** This note is **Declared** — it captures a
> named follow-on task from DN-44 §1.1 without designing the mechanism. The preference order
> (fix in the language/toolchain first; disclosure is the fallback) is cited from DN-44 §1.1 as
> that note's stated discipline, not re-decided here. No claim is made about the disclosure
> block's form or the gate's implementation.

---

## §1 The framing (from DN-44 §1.1, preserved in intent)

DN-44 §1.1 states the **honest-insecurity-disclosure corollary**: for any surface that is
**intentionally not hardened** or **cannot be patched at the language/toolchain level** (a FFI
boundary, a platform ABI, a deliberate escape hatch like `wild`), the discipline is three-part:

1. **Prefer the in-language / in-toolchain fix.** The first question for any vulnerability is
   always *can we fix it where it lives* — in the kernel, the checker, a prim, the lowering, a
   gate. A program-level workaround is the **fallback**, taken only when the surface genuinely
   cannot be closed in the language or its toolchain.
2. **If it cannot be fixed there, it ships with a disclosure** — a documented **disclaimer + the
   reasoning/justification** for why it is unhardened (the trade-off that forced it), co-located
   with the surface (doc-comment, spec §, the `// SAFETY:`/`wild`-justification analogue), never
   buried.
3. **…and with practical guidance** — how a Mycelium *program author* insulates against it.

This is **G2 (never-silent) applied to security itself**: we never claim more safety than we have,
and an unavoidable gap is surfaced, explained, and worked-around-with-guidance, not quietly left
for a developer to trip over.

**This note** is the tracked follow-on that operationalizes that corollary. Its deliverables
(when designed): a **standard disclosure block** (a concrete format for the disclaimer +
justification + guidance), and a **gate** that asserts every `wild`/FFI/intentional-escape surface
carries one. Neither the block form nor the gate mechanism is decided here.

## §2 Relationship to existing docs

- **DN-44 §1.1** (`docs/notes/DN-44-Codebase-Security-Posture.md`) is the primary basis.
  The preference order and three-part discipline are cited from there; this note does not re-decide
  them. DN-44 §1.1 explicitly names this as follow-on F2 in the `rsm` kickoff.
- **G2 (never-silent)** is the grounding principle: an unhardened surface is never implicitly
  "safe" — it must surface its gap (CLAUDE.md house rule #2; the full G2 definition is in
  `docs/Mycelium_Project_Foundation.md`).
- **`scripts/checks/safety.sh`** (the existing `// SAFETY:` presence gate) is a precedent: a
  grep-based gate enforcing that every Rust `unsafe` block carries a `// SAFETY:` justification.
  The disclosure gate this note aims to design is the Mycelium-language analogue — every `wild`
  block / FFI / intentional-escape surface carries a disclosure block.
- **RFC-0035 §2** covers the `unsafe`/FFI misuse vulnerability class in the security scanning
  toolkit. The disclosure gate (this note) is a **source-level / doc-level enforcement** that
  complements RFC-0035's runtime-analysis surface.
- **ADR-014 / RFC-0028** (the `unsafe`/FFI surface policy) define the surfaces that require a
  `// SAFETY:` annotation today. The `wild`/FFI disclosure discipline extends that to the
  **program-author-facing** level.

## §3 Open Questions

**OQ-1.** What is the standard disclosure block's format? A structured doc-comment block (analogous
to `// SAFETY:`), a `@disclosure` annotation, a spec section, or something else? What fields must
it carry (disclaimer, trade-off justification, guidance for program authors)?

**OQ-2.** Where exactly does the disclosure block live for each surface type? In the doc-comment
of a `wild`-using construct? In the spec section that names the FFI boundary? Co-located in both?

**OQ-3.** What is the gate mechanism? A `grep`/`git grep`-based check (like `safety.sh`) over
`.myc` source files for `wild` blocks, or a semantic check over the Core IR (RFC-0001), or a
combination? What is the right gating tier (always-on blocking vs advisory)?

**OQ-4.** How does the gate handle the preference order — i.e., can the gate distinguish "this
surface genuinely cannot be fixed in the language" from "this developer chose not to fix it"? Does
it need to, or is the disclosure block itself the claim ("I assert this cannot be fixed in the
language — here's why")?

**OQ-5.** How does the disclosure gate interact with the RFC-0035 toolkit? Does RFC-0035's
`unsafe`/FFI misuse class redundantly cover this, or do they operate at different layers (semantic
analysis vs doc-presence enforcement)?

**OQ-6.** What is the scope of "intentional-escape surfaces" that require a disclosure block?
`wild` blocks, FFI calls via RFC-0028, platform ABI surfaces — what else?

**OQ-7.** Is there a machine-verifiable format for the disclosure block, or is it inherently
human-review territory? A structured annotation could be machine-checked; a free-text block cannot.

## §4 Definition of Done (gate to move Draft → Proposed)

- OQ-1 and OQ-2 are answered: the disclosure block format is specified (at least in enough detail
  to write one correctly), with at least one worked example showing the block applied to a real
  `wild`/FFI surface.
- OQ-3 is answered: the gate mechanism is specified well enough to implement, with an honest
  assessment of what it can and cannot check (grounded in G2: never overclaim the gate's coverage).
- The preference order (fix in the language first; disclosure is the fallback) is confirmed as the
  right disciplinary frame, or revised with an honest basis.
- The maintainer ratifies the disclosure block format and gate approach.

---

## Changelog

- 2026-06-27 — **Draft** (`rsm` kickoff, F2 / DN-46). Capture-only: operationalizes the
  DN-44 §1.1 honesty corollary into a tracked follow-on. Records the preference order and
  three-part discipline as cited from DN-44 §1.1. All mechanism decisions (block format, gate)
  are open. No design decided.
