# Devlog — 2026-06-16 · Designing the docs pipeline without building it

> **What this is** (see `docs/notes/Narrative-Capture-and-Authoring.md`): the *narrative* layer — the
> messy middle the RFCs smooth over. Append-only, informal, honest. The RFCs/ADRs/DNs remain the source
> of truth; this is the *story* of how a decision actually got made. Refs point at what shipped.

**Theme.** M-363 is the meta-task: the pipeline that will one day write the blog, the book, the manual,
and the API reference *from this very corpus* — including, eventually, this devlog entry. The maintainer
flagged it as a "fresh session, full design," and the issue is explicit: **design first, ratify before
building.** So the discipline this session was to *resist building it* and produce a design good enough
to ratify against.

---

## 1. The temptation to ship a static-site generator

The fastest dopamine here is `cargo add` an mdBook-class toolchain, point it at `docs/`, and have a site
by lunch. That would have been the wrong move twice over. First, the build stack is a **dependency
decision** (the same line M-359 drew at the TOML crate) — and a docs toolchain is a *big* one (mdBook vs
Sphinx vs custom-IR+Pandoc each drag in very different costs). Second, picking it unilaterally would skip
the maintainer's gate. So the build stack is the headline **open choice (§8)**, with a recommendation
(custom doc-IR + Pandoc/Typst) and the two alternatives laid out with their real trade-offs — not a
`cargo add` already in a commit.

## 2. The one idea that makes the whole thing honest: one IR, many renderers

The research kept returning the same lesson (T7.6/T7.7): generated docs drift and lie when each format is
its own pipeline. The fix is structural — **one content-addressed document IR, and HTML/PDF/EPUB/JSON are
all renderers of it.** That single decision discharges three house rules at once: dual human/machine
projection (G11) becomes "two views of one node," no-drift (ADR-003) becomes "same hash or the build
fails," and stable deep links fall out for free. It's the RFC-0013 "a file is a projection of the
canonical declaration" posture, applied to the manual instead of a diagnostic.

## 3. "Clean and readable" had to become a lint, not a wish

The §4.1 quality bar in the Capture note is a lovely list of adjectives — clean, legible, digestible —
and adjectives don't gate anything. The design's job was to turn each into a **checkable pass/fail** over
the doc IR (8 of them). Two matter most. **Checked examples** (T7.5/T7.1): every inline example
compiles/runs in CI, so a stale example is a *build failure*, not silent rot — the doc analogue of
never-silent. And **undocumented-is-flagged**: a missing doc-comment becomes an explicit "undocumented"
node, never invented filler — the prose form of G2, and the one rule that keeps "fully automated" from
becoming "fully hallucinated."

## 4. What I deliberately did not do

No generators, no templates, no renderers, no `cargo add`. The deliverable is the **Proposed** spec +
the prior-art record, with the consequential choices flagged for ratification — exactly the shape
RFC-0015 had before M-362 ratified it. When the maintainer picks a build stack, the design is ready to
execute against; until then, building would be guessing in code instead of in a doc.

Refs: `docs/spec/Narrative-Authoring-Pipeline.md` (Proposed); `research/07-…` (T7.1–T7.7); the
*Narrative Capture & Automated Authoring* note; issue M-363 (#134). No kernel change (KC-3); no pipeline
code (design-first).
