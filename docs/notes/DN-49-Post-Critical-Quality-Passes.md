# Design Note DN-49 — Post-Critical Quality Passes (sequenced after full 1.0)

| Field | Value |
|---|---|
| **Note** | DN-49 |
| **Status** | **Draft (2026-06-27) — capture-only; frames the question, decides nothing.** Records the maintainer's intent for a deliberate, ordered set of quality passes to be conducted once the critical 1.0 work is done and the language is fully working and usable. No design is decided; the order and scope of each pass are the maintainer's stated intent, captured here so they are not lost. |
| **Feeds** | M-797 (inline-test retrofit — the testing-refactor pass lands that backlog); CLAUDE.md §Test-layout (the test layout rules the testing-refactor enforces); DN-48 (F4 — the L4 / reveal question, referenced here as one of the passes). |
| **Date** | June 27, 2026 |
| **Decides** | *Nothing.* Capture-only — records the ordered pass sequence as the maintainer's stated intent. All design questions within each pass are open. |
| **Task** | `rsm` kickoff, F6. Sequenced after full 1.0 (language fully working/usable). |

> **Posture (transparency rule / VR-5 / G2).** This note is **Declared** — it records a
> future-workstream sequence that cannot be started until the language reaches full 1.0. The
> pass order is the maintainer's stated intent; this note does not validate or challenge it.
> Nothing here is designed, implemented, or decided — each pass generates its own DN/RFC
> when it begins. All forward-looking statements are marked `Declared`/open.

---

## §1 The framing (maintainer's intent)

Once the critical 1.0 work is done and the language is **fully working and usable**, conduct a
deliberate set of **quality passes in the following order** — each a focused, bounded arc of
work, not a perpetual refactor:

### Pass 1 — Testing refactor (FIRST)

Re-organize the test suite to be:

- **Parameterized / fixture-driven / dynamic** — an easy entry point to wire in multiple test
  kinds and configurations, including **chaos testing**.
- **Surfacing all the metrics and results** we want (coverage, variant enumeration, invariant
  tallies, chaos outcomes).

The right moment for this pass: once the **trusted core is nailed down enough to have mapped
every acceptable and rejecting variant and invariant**, decide **what actually needs testing vs
what does not**, then reorganize and clean optimally.

This pass ties to **M-797** (the inline-test retrofit backlog: ~185 files with inline tests to
be extracted to `src/tests/` per CLAUDE.md §Test-layout) and to the **test layout rules**
(CLAUDE.md §Test-layout): no `#[cfg(test)]` in logic files; every unit test in a dedicated
in-crate `src/tests/` module; complex test logic in fixtures + parameterization, not test
bodies.

**Threshold to start this pass (`Declared`):** the trusted core is stable enough that the set
of acceptable/rejecting variants is known and bounded — not before.

### Pass 2 — Language-level DRY / efficiency pass

Every place the language implementation unnecessarily repeats or redoes, or is less simple /
efficient / effective than its constraints allow — **while retaining**:

- Tunable certification (RFC-0034) + explicit swap (RFC-0002) — these are never sacrificed.
- Making certification and swap **easier and more organic to use**, not harder.
- Everything more **performant / secure / memory-safe / ergonomic**.

This is a cleanup pass, not a redesign. It assumes the critical design decisions are
stable (otherwise the cleanup is premature and will be invalidated).

### Pass 3 — Public/private + no-black-boxes

Resolve the public-vs-private testing-visibility conflict across the board:

- **White-box in-crate `src/tests/`** (CLAUDE.md §Test-layout) gives access to private items
  without forcing them `pub` — the preferred pattern.
- **Black-box `tests/`** (fully external) is appropriate for public-API integration tests.
- The conflict: some items are currently forced `pub` only to be testable, or tested inline in
  logic files violating the layout rule. This pass resolves those residuals.

This also ties to the **no-black-boxes principle** (CLAUDE.md house rule #2 / ADR-006): every
internal mechanism must be inspectable and `EXPLAIN`-able. A forced-`pub` is a smell; a private
item tested inline is a layout violation. The pass resolves both systemically.

### Pass 4 — L3 review + the L4 question (DN-48)

As part of the same **lowering-architecture review**:

- **L3 review:** audit the current L3 concrete surface grammar (RFC-0030 / the ratified L3
  surface) for completeness, ergonomics, and any accumulated debt.
- **The L4 question:** assess whether an L4 layer is appropriate (the full investigation is
  DN-48 / F4 — see that note for the open questions and research agenda).

These two are grouped because the L3 audit and the L4 decision share the same frame: the
lowering architecture and whether the current layer count is correct.

## §2 Relationship to existing docs

- **M-797** — the inline-test retrofit backlog (tracked in `tools/github/issues.yaml`). The
  testing refactor (Pass 1) is where M-797's remaining items are resolved systemically.
- **CLAUDE.md §Test-layout** — the test layout rules that Pass 1 and Pass 3 enforce. The rules
  are already in force going-forward; the passes apply them retroactively to the accumulated
  backlog.
- **DN-48** (`docs/notes/DN-48-L4-And-Reveal-Lowering.md`) — the L4 and reveal-lowering
  investigation. Pass 4 is where DN-48's research deliverables are acted on.
- **RFC-0034** (`docs/rfcs/RFC-0034-Tunable-Certification-and-Transparency-Modes.md`) — the
  tunable certification machinery that Pass 2 explicitly must not sacrifice or obscure.
- **RFC-0002** (`docs/rfcs/RFC-0002-Swap-Certificate-and-Split-Regime.md`) — the swap
  certificate that Pass 2 must retain and make more organic.
- **RFC-0030** (`docs/rfcs/RFC-0030-Concrete-Surface-Grammar-L3-Ratification.md`) — the L3
  ratification that Pass 4's L3 review audits.
- **ADR-006** (no black boxes) and CLAUDE.md house rule #2 — the never-silent, inspectable,
  `EXPLAIN`-able principle that Pass 3 enforces at the public/private boundary.

## §3 Open Questions

**OQ-1.** What is the exact threshold for "trusted core stable enough" (Pass 1 trigger)? Is
this a quantitative criterion (e.g. all variant/invariant property tests passing at HIGH case
count), a qualitative one (maintainer judgment), or both?

**OQ-2.** What does "parameterized / fixture-driven / dynamic" mean concretely for the Mycelium
test suite? Is it a framework (e.g. `rstest`, `proptest` parameterization), a convention, or a
custom harness? The existing `CertMode::ALL`-style parameterization, `REJECT_EXPECTED` pattern,
and `differential.rs::data_corpus()` (cited in CLAUDE.md §Test-layout) are precedents — does
Pass 1 generalize those patterns or introduce something new?

**OQ-3.** What does "chaos testing" mean in the Mycelium context? Fault injection into the
swap/certification machinery? Random input at the surface grammar? Fuzzing at the L0 Core IR
level? All of these? (Note: `cargo-fuzz` is already in `just check-full` — is Pass 1's chaos
testing distinct from fuzzing, or is it the same under a different name?)

**OQ-4.** Pass 2 says "every place we unnecessarily repeat or redo". How is "unnecessary"
determined without a prior enumeration? Does Pass 2 require a pre-pass audit (a `Declared`
list of repetition candidates) before the refactor begins, or is it a continuous-improvement
discipline?

**OQ-5.** Pass 3's "resolve the public/private testing-visibility conflict" — does this require
a new lint / gate (a `just check` addition that fails on `pub` items with no external consumer),
or is it handled by the as-touched enforcement already in place (CLAUDE.md §Test-layout)?

**OQ-6.** Sequencing: the four passes are ordered (1 → 2 → 3 → 4). Is this order load-bearing
(pass N depends on pass N-1 having completed), or is it a preference? In particular: can Pass 3
(public/private) be done in parallel with Pass 2 (DRY), since they touch different axes of the
codebase?

**OQ-7.** Does this note become a DN for each pass independently (i.e. each pass spawns its
own DN/RFC when it begins), or does it evolve in place as passes complete? The append-only
discipline (house rule #3) requires each phase to be recorded as a status move, not a rewrite.

## §4 Definition of Done (gate to move Draft → Proposed)

- The **trigger criterion for Pass 1** (OQ-1) is specified and grounded — "trusted core stable
  enough" has an explicit, checkable definition.
- OQ-2 is answered: "parameterized / fixture-driven / dynamic" is defined in terms of concrete
  patterns or tooling choices, with at least one worked example.
- OQ-6 is answered: the sequencing dependencies between passes are explicit (which are truly
  ordered and which can be parallelized).
- The maintainer confirms the pass order and the Pass-1 trigger before the note moves to
  Proposed.

---

## Changelog

- 2026-06-27 — **Draft** (`rsm` kickoff, F6 / DN-49). Capture-only: records the maintainer's
  post-critical quality pass sequence (testing refactor → DRY/efficiency → public/private →
  L3 review + L4). References M-797, CLAUDE.md §Test-layout, DN-48 (F4), RFC-0034, RFC-0002,
  RFC-0030, ADR-006. All mechanism and trigger decisions open.
