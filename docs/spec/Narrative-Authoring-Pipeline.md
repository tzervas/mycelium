# Spec (Proposed) — Narrative & Automated Authoring Pipeline

| Field | Value |
|---|---|
| **Status** | **Accepted** (2026-06-16 — design ratified; **§8 build stack ratified** 2026-06-16: custom doc-IR + Typst + static HTML · Typst PDF · v0 single-version. The pipeline *design* is now ratified; **building M-363 remains a separate, not-yet-scheduled task** — this unblocks M-366's §4.1 doc quality-bar lint, which can be specified against the chosen stack.) |
| **Scope** | The generators, templates, HTML/PDF/EPUB build, and the **§4.1 quality-bar lint** that turn the cited corpus into the four authoring outputs (blog · book · manual · API reference) |
| **Depends on** | The *Narrative Capture & Automated Authoring* note (intent, data model, §4.1 quality bar, honesty constraints); RFC-0013 (the `minimal/medium/detailed` levels reused for docs; "a file is a projection of the canonical declaration"); M-359 (the nodule header / manifest metadata output (d) projects); ADR-003 (content-addressing → stable links, no drift); G2/G11/VR-5; KC-3 (tooling layer) |
| **Feeds** | M-361 (the full-fat toolchain — this is one of its tools); M-346 (stdlib docs); M-363 (this, enacted after ratification) |
| **Grounds on** | `research/07-narrative-authoring-pipeline-RECORD.md` (T7.1–T7.7) |

## 1. Summary

The corpus is already ~80% of a grounded, cited, append-only narrative (the *Capture* note). This spec
designs the **authoring pipeline** that projects it into reader-facing outputs without (a) drifting from
the corpus, (b) hallucinating prose, or (c) producing the ugly, unnavigable output generated docs are
infamous for. The architecture is **one content-addressed document IR, many renderers** (T7.6): the
corpus is compiled to a doc IR, and HTML/PDF/EPUB **and** a machine-JSON form are all *renderers of that
one IR* — so every format shares identity (ADR-003/G11) and cannot diverge. Generation is **projection,
not authorship**: the human gate is the generator + templates, not each page.

## 2. The four outputs and their sources (recap, from the Capture note §4)

| Output | Automation | Primary corpus sources |
|---|---|---|
| (a) **Blog** | partial (draft-then-review) | `docs/devlog/` + per-doc changelogs |
| (b) **Language book** | partial → full | RFC *Guide-level* sections, Example Programs Reference, Glossary |
| (c) **Reference manual** | partial → full | RFC *Reference-level* designs + grammar (EBNF) + schemas + Lexicon/Glossary |
| (d) **Docs + API reference** | **full** | doc-comments + M-359 nodule header (`@summary`/`@version`/`@license`/authors) + JSON schemas + the `mycelium-proj.toml` public surface |

Outputs (c)/(d) are **pure projection** (the source is already normative — regenerate, never hand-author
in parallel; T7.1/T7.7). Outputs (a)/(b) carry an *interpretive* layer and stay **draft-then-review**
(human-in-the-loop is the floor; the *Capture* note §5).

## 3. Architecture — one IR, many renderers

```
corpus (RFCs/ADRs/DNs · grammar EBNF · JSON schemas · Glossary/Index · devlog · code doc-comments · M-359 metadata)
   │   extract + project (per-output generators, §4)
   ▼
doc IR  (content-addressed nodes: section, prose, example, xref, api-item, level-graded block)
   │   render (one per format)
   ├── HTML  (the live site + LSP-hover JSON sidecar)
   ├── PDF   (print/offline)
   ├── EPUB  (e-readers)
   └── JSON / JSONL  (search index · tooling · machine consumers — G11)
```

- **The doc IR is content-addressed** (ADR-003): a node's identity is the hash of its projected content,
  so cross-references and deep links are **stable** and a renderer cannot silently desync from another.
- **Renderers are pure functions of the IR** (the RFC-0013 dual-projection posture): HTML, PDF, EPUB, and
  JSON are *views*, never parallel truths. Changing prose changes the IR node, which re-renders everywhere.

## 4. Generators (projection, per output)

Each generator is a **total projection** from named corpus sources to doc-IR nodes; an item it cannot
ground is **flagged "undocumented," never invented** (§6 / G2). v0 generators:

- **`gen-apiref` (output d, fully automated).** Walks the crates (rustdoc JSON now; Mycelium-lang
  doc-comments later — dogfooding), the JSON schemas (`docs/spec/schemas/`), and the M-359 nodule
  headers/manifest; emits api-item IR nodes (signature + `@summary` + provenance). Missing doc-comment →
  an explicit "undocumented" node (not blank, not invented).
- **`gen-manual` (output c, projection).** Projects the RFC *Reference-level* sections, the grammar EBNF,
  the schemas, and the Glossary into a normative manual IR; cross-references resolve to content addresses.
- **`gen-book` (output b, projection + light interpretation).** Sequences the RFC *Guide-level* sections,
  the Example Programs Reference, and the Glossary into a learner ordering; the interpretive glue is
  draft-then-review.
- **`gen-blog` (output a, draft-then-review).** Turns a `docs/devlog/` entry (problem → why → approach →
  resolution, already the outline) into a post draft; never auto-published.

## 5. Templates

**One reviewed template** (the §4.1 "learn the shape once"): a shared visual language + structure across
all outputs — a header (title, version, provenance), an index→detail body, level-graded sections, an
example block style, and a footer (source refs / "generated from"). The template is the **human gate for
the fully-automated outputs**: review the template, not each page. Templates are versioned in-repo and the
build pins the template content hash into each artifact (provenance, §6).

## 6. The §4.1 quality-bar lint (the checkable contract)

"Clean · presentable · legible · intelligible · digestible" is enforced as a **lint over the doc IR**
(part of M-361), not a hope. Each check is an explicit pass/fail (never-silent for docs — the G2 analogue):

1. **Single template conformance** — every page uses the one template; a divergent structure fails.
2. **Navigability** — every IR node is reachable from the index; no orphan pages; a search index exists.
3. **Progressive disclosure present** — graded blocks carry a `minimal/medium/detailed` level (RFC-0013
   levels reused); a concept with only one depth where graded depth is required is flagged.
4. **Examples are checked** — every inline example compiles/runs in CI (T7.5/T7.1); a stale example is a
   **build failure**, not silent rot.
5. **No dangling xref / dead link** — every cross-reference resolves to a content address (extends the
   existing `scripts/checks/links.sh` to the generated site).
6. **Dual projection parity (G11)** — the JSON/machine form and the human form derive from the *same* IR
   node (same content hash); a mismatch fails.
7. **No hallucinated prose / undocumented-is-flagged** — every api-item statement traces to the code/spec
   it projects; an ungrounded statement is rejected, and a missing doc surfaces as an explicit
   "undocumented" marker (never invented filler).
8. **Legibility/accessibility** — semantic HTML, alt text on figures, heading order, contrast, code
   highlighting present (checked structurally where possible).

## 7. Placement in the toolchain (M-361)

The pipeline is a **tool in the full-fat suite** (M-361), above the kernel (KC-3): it consumes the corpus,
the code, the schemas, and M-359 metadata, and emits artifacts plus the quality-bar lint result. It runs in CI parity
(`scripts/checks/`): the apiref/manual/book builds and the §4.1 lint are advisory in the design phase and
become a release gate when the language ships free with its docs. Rust-first now (rustdoc over the
crates); Mycelium-lang self-hosted doc-comments later.

## 8. Open choices — flagged for ratification (build before none of these is decided)

Recommendations marked; **no code lands until the maintainer ratifies §8** (the design-first gate — like
RFC-0015 was Draft before M-362).

> **Ratified (2026-06-16).** The maintainer ratified the recommended stack: **§8.1(a)** a small custom
> in-repo **doc-IR generator** (Rust, no heavy dep) + **Typst** for the PDF/EPUB fan-out + a static HTML
> renderer; **§8.2** **Typst** as the PDF engine; **§8.3** **v0 single-version** ("current" only;
> multi-version deferred). §8.4 stands at its recommendation (rustdoc **JSON** behind a thin adapter so the
> Mycelium-lang doc-comment source can replace it later); §8.5 stays deferred (a static in-repo/CI artifact
> plus the LSP-hover sidecar JSON for v0; a hosted site is later). The options below are retained verbatim for
> the record (append-only). This ratification lifts the §8 gate: the *pipeline design* is now Accepted, and
> **M-366's §4.1 doc quality-bar lint may now be specified** against this stack. **Building the pipeline
> (M-363) is still a separate, not-yet-scheduled task** — ratifying the design is not scheduling the build.

1. **Build stack (the consequential one).** (a) *Recommended:* a small **custom doc-IR generator** (Rust,
   in-repo, no heavy dep) + **Pandoc/Typst** for PDF/EPUB fan-out + a static HTML renderer — maximal
   control, fits the content-addressed-IR design, dependency-light (T7.6). (b) **mdBook-class** HTML +
   plugins — fastest to a site, weaker PDF/EPUB (T7.2). (c) **Sphinx/MyST** — strongest multi-output +
   xref out of the box, heaviest toolchain (Python + LaTeX) (T7.3). The choice sets the dependency and
   maintenance cost (the same "adding a dependency is a decision" discipline as M-359's TOML reader).
2. **PDF engine** — Typst (modern, light) vs LaTeX (ubiquitous, heavy). *Recommended:* Typst.
3. **Versioning** — v0 builds "current" only; multi-version (Antora-style, keyed on M-359 `@version`) is
   future (T7.4). Confirm v0 single-version.
4. **API-ref source now** — rustdoc **JSON** (unstable format) vs scraping rustdoc HTML. *Recommended:*
   rustdoc JSON behind a thin adapter (so the Mycelium-lang doc-comment source can replace it later).
5. **Hosting / live serving** — static site in-repo/CI artifact for v0; the LSP-hover sidecar JSON is in
   scope; a hosted site is later.

## 9. Honesty constraints (inherited, non-negotiable — Capture note §5)

Grounded + cited (no hallucination — an authoring tool that cannot cite a statement flags it);
**projection, not parallel truth** (generated from the corpus, ADR-003 makes drift detectable);
human-in-the-loop (draft-then-review the floor; full automation only for the projection outputs);
append-only provenance (every artifact records what it was generated *from* — commit/doc/template hashes).

## Meta — changelog

- **2026-06-16 — Proposed (M-363 design).** Designs the authoring pipeline over the *Capture* note's
  intent: a content-addressed **doc IR → many renderers** architecture (one truth, HTML/PDF/EPUB + JSON);
  four projection generators (apiref/manual/book/blog) with their corpus sources; one reviewed template;
  the **§4.1 quality bar as a checkable lint** (8 explicit checks incl. *checked examples* and
  *undocumented-is-flagged*); placement in the M-361 toolchain. Prior art traced in `research/07-…`
  (T7.1–T7.7). The **build stack (§8) and the other format/versioning choices are flagged for
  ratification — no code lands until they are ratified** (design-first). No kernel change (KC-3).
  Append-only.
- **2026-06-16 — Accepted (§8 ratified).** The maintainer ratified the recommended build stack: custom
  in-repo **doc-IR generator + Typst** (PDF/EPUB) + static HTML (§8.1a); **Typst** PDF engine (§8.2);
  **v0 single-version** (§8.3). §8.4 (rustdoc JSON adapter) stands at recommendation; §8.5 (hosting)
  deferred. The §8 gate is lifted and the pipeline *design* moves **Proposed → Accepted**; this unblocks
  **M-366's §4.1 doc quality-bar lint** (now specifiable against the chosen stack). **Building M-363
  remains a separate, not-yet-scheduled task** (don't start unless asked). Append-only.
- **2026-07-01 — Enacted addendum: BOOK output (M-363 output (b)) + a local Podman/Docker docs
  container.** `crates/mycelium-doc` gained a fifth renderer, `book` (§`mycelium_doc::book`),
  alongside HTML/Typst/JSON (§3): a curated, linear, chaptered reading order over the *same*
  content-addressed doc-IR, driven by a small committed manifest (`docs/book-manifest.json` — explicit
  `sources` for curated order, drift-proof `globs` for the Standard Library/RFC/ADR/DN appendix
  chapters so a new file is picked up automatically), with per-page prev/next navigation, a
  chapter breadcrumb, and a client-side search index (`book/search-index.json` + a hand-rolled
  `search.js` — no new dependency, KC-3). It **composes** the existing per-page HTML projection
  (byte-identical `data-cid`s) rather than re-rendering; the one non-`.md` source
  (`docs/spec/grammar/mycelium.ebnf`) is synthesized as a single verbatim, unchecked `Example` node
  (grounded — the exact file bytes — never invented), and `CONTRIBUTING.md` (outside `docs/`) rides
  in via a new `BuildInput::extra_md_files` field so its cross-references resolve through the same
  pipeline as the rest of the corpus; `BuildInput::conventional`'s default (and so the existing
  `build`/`lint` commands) is unchanged. A manifest entry that resolves to no ingested document is a
  build error, never a silently-dropped chapter (G2). New CLI: `myc-doc book`; new `just docs-book`.
  Also added: `docs/Containerfile` (a two-stage Podman/Docker build — a pinned Rust builder runs
  `myc-doc build` + `myc-doc book` + `cargo doc`, then a minimal Python image serves the assembled
  static site) + `scripts/docs-container.sh` (`just docs-container-build`/`docs-container-run`,
  podman-preferred, docker-fallback) — the agent code index (`docs/api-index/`) is served alongside
  the human-facing book/corpus/rustdoc views, so the container serves agents and the maintainer alike.
  Both advisory (not part of `just check`), the same posture as `scripts/docsite.sh`. Append-only.
