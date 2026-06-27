# Design Note DN-45 — OSV Scanning of Mycelium Programs (post-1.0 / dogfood)

| Field | Value |
|---|---|
| **Note** | DN-45 |
| **Status** | **Draft (2026-06-27) — capture-only; frames the question, decides nothing.** Records the maintainer's future-workstream intent to extend `osv-scanner` (and the in-env supply-chain posture, DN-44) so it can scan **actual Mycelium implementations** and give accurate security guidance on real `.myc` code. No design is decided here; sequencing, scope, and mechanism are all open. |
| **Feeds** | The **supply-chain and program-security posture** (DN-44 §ratchet / RFC-0035). Complements RFC-0035 (the Mycelium-native security-scanning toolkit for `.myc` programs — the detection logic) and DN-44 (the codebase implementation-hardening posture — the Rust kernel side). This note covers the **OSV / supply-chain audit surface** for programs written *in* Mycelium, which is a distinct concern from both. |
| **Date** | June 27, 2026 |
| **Decides** | *Nothing.* Capture-only — frames the question and records sequencing. All mechanism and scope decisions are open questions (§3). |
| **Task** | `rsm` kickoff, F1. Depends on: language reaching full 1.0 (high-level scanning); full self-hosting (low-level scanning). |

> **Posture (transparency rule / VR-5 / G2).** This note is **Declared** — it records a
> future-workstream intent that cannot yet be acted on accurately (the language does not exist as a
> shippable artifact). Nothing here is a design decision; every forward-looking statement is marked
> as an open question or sequenced dependency. No guarantee is claimed.

---

## §1 The framing (maintainer's intent)

Extend language and toolchain support so `osv-scanner` (and the in-env supply-chain posture captured
in DN-44) can scan **actual Mycelium implementations** — programs written *in* Mycelium — and give
accurate, actionable security guidance on real `.myc` code.

**Sequencing (maintainer-set, `Declared`):**

- **High-level scanning** becomes possible once the language is **fully implemented and usable** —
  roughly full 1.0: syntax + low-level functionality at a decent point, even while dogfooding back
  into Mycelium. At this point, a phylum/nodule supply-chain scan over Mycelium programs' declared
  dependencies becomes meaningful.
- **Low-level scanning** lands once the language is **fully self-hosted** (written in itself). The
  implementation-level audit that today runs over the Rust kernel would, at self-hosting, need to run
  over Mycelium source as well.

The constraint is explicit: **plan only now** — this cannot be done accurately or fully until the
language exists and is shippable. Attempting to specify the mechanism before the language is
realized would be premature and likely wrong.

This note captures the intent so it is not lost, and records the sequencing so future phases pick
it up at the right moment.

## §2 Relationship to existing docs

- **DN-44** (`docs/notes/DN-44-Codebase-Security-Posture.md`) covers the **Rust implementation**
  side: the Rust kernel, reference interpreter, and toolchain. DN-45 is the **Mycelium-program**
  side — the same supply-chain audit concern, applied to programs written *in* the language.
- **RFC-0035** (`docs/rfcs/RFC-0035-Security-Scanning-Toolkit.md`) designs the Mycelium-native
  security-scanning toolkit (semantic analysis, vulnerability-class detection, safe auto-fixes for
  `.myc` code). DN-45 is about the **OSV / supply-chain audit** layer specifically — cross-ecosystem
  advisory matching over `.myc` programs' phylum dependency graphs — which complements RFC-0035's
  semantic-analysis layer but is a distinct mechanism.
- **DN-44 §6 (the ratchet)** includes an RFC-0035 entry as a planned future tightening. DN-45 sits
  between those two: it is the **supply-chain arm** of the program-side scanning vision.
- **`osv-scanner.toml`** (present in the repo today) covers the Rust dependency surface. Extension
  to Mycelium programs is the subject of this note.

## §3 Open Questions

**OQ-1.** What does "OSV scanning of a Mycelium program" mean concretely? Is the relevant surface
the phylum dependency graph (the DN-28 content-hash DAG) — analogous to `Cargo.lock` for
Rust — or is it also the `.myc` source itself? Both? At what granularity?

**OQ-2.** Does the phylum registry (DN-28) need to expose an OSV-compatible manifest for
`osv-scanner` to consume, or does this require a custom integration point? Which is more tractable?

**OQ-3.** At the "high-level scanning" milestone (full 1.0 / first shippable), what is the
minimum meaningful scan? Supply-chain only (advisory matching over declared dependencies), or also
source-level class detection (RFC-0035 territory)? Where does the boundary sit?

**OQ-4.** At the "low-level scanning" milestone (full self-hosting), what additional surface opens
up? Does self-hosting change the threat model (the implementation *is* a Mycelium program; the
implementation's supply chain is now Mycelium phlya)?

**OQ-5.** How does the `osv-scanner.toml` configuration evolve to cover `.myc` roots alongside
Rust workspace roots? Is this a single config extension or a separate scan invocation?

**OQ-6.** What is the disclosure / reporting pipeline for a vulnerability found in a Mycelium
program's dependency? Does it flow through the RFC-0035 registry (§4 of that RFC), through
OSV.dev directly, or both?

**OQ-7.** Sequencing is described at a coarse level (full 1.0 / self-hosting). What are the
intermediate checkpoints — is there a partial scan possible before full 1.0 that is still accurate?

## §4 Definition of Done (gate to move Draft → Proposed)

- A **research record** is produced that surveys OSV-scanner's extension model, the phylum
  registry's content-hash DAG as an advisory matching surface, and at least one precedent from
  another language ecosystem (e.g. npm / Go modules).
- OQ-1 through OQ-3 are answered with a grounded proposal (mechanism sketched, not fully designed).
- The sequencing (high-level vs low-level) is confirmed or revised based on that research.
- The maintainer ratifies the revised sequencing and mechanism sketch.

---

## Changelog

- 2026-06-27 — **Draft** (`rsm` kickoff, F1 / DN-45). Capture-only: records the maintainer's
  future-workstream intent to extend OSV scanning to actual Mycelium programs (`.myc` code),
  with maintainer-set sequencing (high-level at full 1.0; low-level at full self-hosting). All
  mechanism decisions open. Relationship to DN-44 and RFC-0035 framed. No design decided.
