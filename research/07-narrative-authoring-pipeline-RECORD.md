# Research Record 07 — Narrative & Automated Authoring Pipeline (M-363 grounding)

> **What this file is.** A durable record tracing the prior art the **narrative/authoring pipeline**
> (the *Narrative Capture & Automated Authoring* note; M-363) cites as design inspiration, into the
> evidence base — the pre-design grounding obligation, so the pipeline design (`docs/spec/
> Narrative-Authoring-Pipeline.md`, Proposed) rests on checked precedent rather than vibe. Conducted
> 2026-06-16. Findings are labeled **T7.1–T7.7** (continuing the T0–T6 scheme) and map onto the four
> authoring outputs (blog / book / manual / API ref) and the **§4.1 quality bar**.

## Scope

The goal: **partially-to-fully automated** authoring of (a) blog posts, (b) a language book, (c) a
reference manual, and (d) docs + API reference — all **synthesis from the cited corpus** (projection,
never free invention), shipped free in HTML/PDF/EPUB and served live. The governing question: **which
docs-as-code precedents let a content-addressed, honesty-first corpus generate clean, navigable,
non-drifting docs — and which failure modes (ugly output, drift, hallucinated prose) to design out?**

Seven precedents:

1. **rustdoc / docs.rs** — API docs as pure projection from code + doc-comments (→ T7.1, output d).
2. **mdBook** — the Rust-book toolchain: Markdown → navigable static site (→ T7.2, output b/c, §4.1).
3. **Sphinx + reStructuredText/MyST** — cross-referencing, multi-output (HTML/PDF/EPUB) docs (→ T7.3, §4.1).
4. **Antora** — multi-repo, versioned docs-as-code with a component model (→ T7.4, versioning).
5. **Literate programming (WEB/noweb/`org-babel`/Rust doctests)** — prose+code one source, examples checked (→ T7.5, §4.1 "checked examples").
6. **Pandoc / Typst** — one source → many formats (HTML/PDF/EPUB) (→ T7.6, the build).
7. **Spec-generated manuals (WebAssembly, Rust Reference, JSON-Schema docs)** — a manual *projected from* the normative spec (→ T7.7, output c, no-drift).

## Results by precedent

### T7.1 — rustdoc / docs.rs (API docs as pure projection)
rustdoc generates API docs **directly from the code + `///` doc-comments**, with item signatures,
cross-links, and **doctests that are compiled and run** (the examples cannot rot). docs.rs builds and
hosts them per-release automatically. **Maps to output (d) + §4.1:** output (d) is the *most* automatable
because it is pure projection from code + the M-359 nodule-header metadata + the JSON schemas — no
interpretive layer. The doctest discipline is the model for §4.1 "worked examples are *checked*." The
near-term plan (rustdoc over the crates now, Mycelium-lang doc-comments later) is exactly the
dogfooding ladder. *(Checked against background knowledge of rustdoc/docs.rs; the projection + doctest
properties are definitional.)*

### T7.2 — mdBook (Markdown → navigable static site)
mdBook (the toolchain behind *The Rust Programming Language* and *The Rustonomicon*) compiles a
`SUMMARY.md`-driven tree of Markdown into a navigable static site with search, theming, and a stable
sidebar. **Maps to output (b)/(c) + §4.1:** it is the proof that "one consistent template + index→detail
navigation + search" (§4.1) is achievable from Markdown sources with a small toolchain. Strong candidate
for the book/manual HTML renderer. Limitation: mdBook's PDF/EPUB story is weaker (plugins), feeding the
output-format choice (→ T7.6).

### T7.3 — Sphinx + reST/MyST (cross-referencing, multi-output)
Sphinx pioneered docs-as-code cross-referencing (`:ref:`, autodoc, intersphinx) and emits HTML **and**
PDF (via LaTeX) **and** EPUB from one source; MyST brings Markdown to it. **Maps to §4.1 + the build:**
the strongest precedent for *one source → HTML/PDF/EPUB* with real cross-references and a search index.
Its autodoc (API docs from docstrings) parallels rustdoc for output (d). Cost: a heavier toolchain
(Python + LaTeX for PDF). Informs the build-stack choice.

### T7.4 — Antora (multi-repo, versioned docs-as-code)
Antora models docs as **versioned components** assembled from multiple sources into one site —
the versioning + component model docs-as-code precedent. **Maps to versioning:** as Mycelium grows
multiple `phyla`/releases, the manual/API-ref must be **versioned** (a reader picks the version), and the
M-359 `@version`/manifest metadata is the natural key. Antora shows the component/version model; v0 likely
defers multi-version to a single "current" build, with versioning named as future.

### T7.5 — Literate programming + checked examples (WEB/noweb, org-babel, doctests)
From Knuth's WEB through `org-babel` and Rust doctests: **prose and code share one source, and the code
is executed** so the document cannot lie about its examples. **Maps to §4.1 "worked examples inline,
checked":** the Example Programs Reference is the example source, and the pipeline must *run* examples in
CI so a stale example is a build failure, not silent rot — the doc analogue of never-silent (G2). This is
the single most important §4.1 property to enforce mechanically.

### T7.6 — Pandoc / Typst (one source → many formats)
Pandoc converts a common document model to HTML/PDF/EPUB/… ; Typst is a modern, fast typesetting system
(simpler than LaTeX) for PDF. **Maps to the HTML/PDF/EPUB build:** rather than three renderers, the
pipeline should target **one intermediate representation** (IR) and fan out to formats via Pandoc/Typst —
HTML from the IR, EPUB (HTML-based) and PDF (Typst/LaTeX) from the same. Keeps "dual human/machine
projection" (§4.1/G11) honest: all formats are renderers of *one* content-addressed IR, never parallel
truths.

### T7.7 — Spec-generated manuals (WebAssembly, Rust Reference, JSON-Schema docs)
The WebAssembly spec generates prose + a test corpus from one normative source; the Rust Reference is a
hand-maintained-but-projected normative manual; JSON-Schema-doc tools render schemas to readable HTML.
**Maps to output (c) + the no-drift constraint:** the reference manual must be a **projection of** the
RFCs/grammar/schemas/lexicon (RFC-0013's "a file is a projection of the canonical declaration"), so it
*cannot* drift — regenerated, never hand-maintained in parallel. Mycelium already has the machine-readable
substrate (EBNF, JSON schemas, the Glossary/Index) this needs.

## How the findings shape the design (→ `docs/spec/Narrative-Authoring-Pipeline.md`)

- **One IR, many renderers** (T7.6): corpus → a content-addressed **doc IR** → HTML/PDF/EPUB + JSON
  (machine), so all formats share identity (ADR-003/G11) and cannot diverge.
- **Projection, not parallel truth** (T7.1/T7.7): manual + API ref are *generated from* RFCs/grammar/
  schemas/code; the human gate is the generator/templates, not each page.
- **Checked examples as a build gate** (T7.5/T7.1): every inline example compiles/runs in CI; a stale
  example fails the build (the §4.1 quality-bar lint, never-silent for docs — G2).
- **One template, index→detail, progressive disclosure** (T7.2/T7.3): a single reviewed template; the
  Doc-Index/Glossary "summary→detail" pattern; RFC-0013 `minimal/medium/detailed` levels reused for docs.
- **Versioning deferred but designed-for** (T7.4): v0 builds "current"; the M-359 `@version`/manifest is
  the version key when multi-version lands.
- **Build stack is the open choice** (T7.2/T7.3/T7.6): mdBook-class HTML vs Sphinx/MyST vs a custom IR +
  Pandoc/Typst — flagged for ratification (the design's §-open-choices), since it sets the dependency and
  maintenance cost.

## Uncertainty register

- T7.1–T7.7 are checked against **background knowledge** of these widely-used, stable tools; their
  load-bearing properties (projection, multi-output, checked examples, versioned components) are
  definitional, not version-specific. No primary docs were fetched in this environment. Flagged
  **Empirical/Declared**, not Proven.
- No claim here commits the project to a tool; the build stack is explicitly an open choice for
  ratification. Nothing upgrades a guarantee (VR-5).

## Changelog
- **2026-06-16 — Created.** Traces the seven docs-as-code / spec-generation precedents (rustdoc/docs.rs,
  mdBook, Sphinx/MyST, Antora, literate programming, Pandoc/Typst, spec-generated manuals) into the
  evidence base as **T7.1–T7.7**, grounding the M-363 pipeline design ahead of ratification. Append-only.
