# Design Note DN-50 — Parsable-vs-Runnable Gap Analysis (the accept↔instantiate frontier)

| Field | Value |
|---|---|
| **Note** | DN-50 |
| **Status** | **Draft (2026-06-27) — capture-only; frames the question, decides nothing.** Records the maintainer's intent to systematically **analyze, parse, and determine the gap between code that is *parsable/checkable* and code that is actually *runnable/instantiable*** — the frontier where the parser/checker *accepts* a construct but the elaborator/monomorphizer/runtime cannot yet *instantiate or run* it three-way (L1-eval ≡ L0-interp ≡ AOT). No mechanism is decided here; the audit method, the surfacing form, and any gate are all open. |
| **Feeds** | The conformance discipline (M-719) + the three-way differential harness (`crates/mycelium-l1/tests/differential.rs`) + the never-silent `Residual` mechanism (`crate::mono::ElabError::Residual`). Generalizes the recurring "type-checks but does not run" pattern across the language surface. |
| **Date** | June 27, 2026 |
| **Decides** | *Nothing.* Capture-only — frames the gap and records why it recurs. The audit method, whether a standing gate is warranted, and how the frontier is surfaced are all open questions (§3). |
| **Task** | `rsm` kickoff, F7 (added 2026-06-27 from the maintainer's observation during the M-753 width-generics work). |

> **Posture (transparency rule / VR-5 / G2).** This note is **Declared** — it records a maintainer
> direction and an observed pattern, not a measured audit. The gap inventory in §2 is *illustrative*
> (drawn from known flagged sites), **not** an exhaustive or verified census; producing that census is
> precisely the work this note frames. No claim here upgrades past `Declared`.

## §1 — The framing (preserved)

There is a standing distinction between **parsable/checkable** code and **runnable/instantiable**
code, and the project should at some point **systematically analyze, parse, and determine the gap
between the two**. A construct can be *accepted* by the lexer/parser/checker (it has a well-formed
surface and type) yet not be *executable* end-to-end: the elaborator, monomorphizer, or runtime has
no lowering for it, so it never runs three-way (L1-eval ≡ L0-interp ≡ AOT) or never instantiates at a
concrete type/width. The goal is to **know the whole frontier** — every place the *accepted surface*
exceeds the *runnable surface* — so the gap is mapped, deliberate, and never silently mistaken for
"done."

This is the never-silent rule (G2) applied to the *implementation frontier*: a construct that
type-checks but cannot run must be **explicitly** flagged (a `Residual`, a documented FLAG, a
spec-staged note) — never a surface that quietly accepts what it cannot execute.

## §2 — Why it recurs (illustrative, not exhaustive — `Declared`)

The accept↔instantiate gap has appeared repeatedly, each time handled honestly *in isolation* but
never inventoried *as a class*:

- **Traits** type-checked (M-659) well before dictionary elaboration let them **run** (M-673) — a
  multi-wave "checks-but-doesn't-run" interval.
- **Recursive HOF / `iter` combinators** (`map`/`filter`/`fold`) **type-check but the monomorphizer
  refuses** recursive application of a function *parameter* (M-715; outside RFC-0024 defunctionalization)
  — re-flagged, not yet closed.
- **Width-generics** (M-753) — the surface **parses and checks shape** but does not **instantiate**
  until `unify` binds the width variable and the monomorphizer pins it (the gap that prompted this note).
- General: any construct gated behind a `Residual` or a staged-spec row is, by definition, on the
  parsable-not-yet-runnable side of the frontier.

The pattern is healthy *when surfaced* (the `Residual`/FLAG discipline) and dangerous *when assumed
away* ("it parses, therefore it works").

## §3 — Open questions

- **OQ-1.** What is the precise, testable definition of "runnable/instantiable" here — three-way
  agreement (L1≡L0≡AOT), or L0-interp alone, or per-tier? (Likely three-way, per the conformance bar.)
- **OQ-2.** Is the right artifact a **one-time audit/census**, a **standing gate**, or both? A gate
  would assert: every parser-accepted construct either runs three-way **or** carries an explicit
  `Residual`/FLAG (no silent accept-but-unrunnable).
- **OQ-3.** How is the frontier **enumerated** — drive the accept corpus (`crates/mycelium-l1/tests/`
  `accept/` fixtures + grammar) through elaboration/mono and classify each as runs / explicit-Residual
  / **silent-gap** (the bug class)?
- **OQ-4.** Where does this live relative to M-719 (conformance over a *specific* surface) — is DN-50
  the *generalization* (whole-surface frontier map) of which M-719 is one slice?
- **OQ-5.** How does the frontier interact with the tiered guarantees and `fast`/`certified` modes —
  does "runnable" mean runnable in every mode?
- **OQ-6.** 2nd/3rd-order effects: does mapping the frontier change what we *choose* to keep as a
  deliberate `Residual` (staged) vs. what becomes a must-fix?

## §4 — Relationship / grounding

- **M-719** (conformance over the generic surface) — a concrete slice; DN-50 frames the general case.
- **M-715** (recursive-HOF defunctionalization gap), **M-673** (trait elaboration), **M-753**
  (width-generics) — the recurring instances motivating a systematic treatment.
- The **three-way differential harness** (`crates/mycelium-l1/tests/differential.rs`) and the
  **`Residual`** mechanism (`crate::mono`) are the existing tools the audit/gate would build on.
- **RFC-0024** (defunctionalization) — one frontier whose extent this would map.

## §5 — Definition of Done (to leave Draft)

A research/audit record that (a) fixes the "runnable" definition (OQ-1), (b) produces a *census* of
the current accept↔instantiate frontier classified runs / explicit-Residual / silent-gap, and (c)
recommends whether to add a standing gate (OQ-2). Nothing in this note is decided; it records the
direction so it is not lost.

---

## Changelog

- **2026-06-27 — OQ-1 + OQ-2 ratified by the maintainer in-session (the design questions resolve; the
  census remains the work).** **OQ-1 (definition):** a construct is **runnable** when it elaborates to
  closed L0 and **executes three-way** (L1-eval ≡ L0-interp ≡ AOT) on at least one instantiation — the
  existing `differential.rs` bar; no new machinery. **OQ-2 (artifact):** **both, sequenced** — a one-time
  **census** first (classify every accepted construct *runs* / *explicit-`Residual`* / **silent-gap**),
  then a **narrow standing gate** that forbids **only** the silent-gap class (checker accepts ⇒ either it
  runs **or** it hits an explicit `Residual`/FLAG). The gate is **not** a *must-run* gate (that would block
  legitimate staged work); it is **G2 applied to the implementation frontier** — never a *silent*
  accept-but-unrunnable. **OQ-4:** confirmed — DN-50 is the whole-surface generalization of which **M-719**
  (width-generic conformance) is one slice. The remaining §5 DoD item is the **census** (the M-807 audit
  work) + wiring the narrow gate over the accept corpus; the design is now decided. Append-only; the Status
  header's "decides nothing" framing is superseded by this entry for OQ-1/OQ-2 (the census itself is still TODO).
- **2026-06-27** — Created (Draft, capture-only). `rsm` kickoff F7, from the maintainer's observation
  during M-753 that parsable code and runnable/instantiable code form a gap worth auditing as a class.
