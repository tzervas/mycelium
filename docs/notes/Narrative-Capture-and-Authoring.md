# Note — Narrative Capture & Automated Authoring (blog · language book · reference manual)

| Field | Value |
|---|---|
| **Status** | **Living — initial capture** (2026-06-16; full pipeline design is a fresh session — tracked M-363) |
| **Purpose** | Capture *enough* development narrative — decisions, struggles, problems solved, the how **and** the why — to enable **partially-to-fully automated** authoring of (1) project **blog** posts, (2) a full **language book**, and (3) a **language reference manual**, distributed **free** with the language in digital formats |
| **Grounding** | The honesty rule (ground every claim; cite the basis) already makes the corpus a *cited, append-only, grounded* narrative — this note names the capture discipline + authoring pipeline that builds on it |

## 1. Why this is unusually tractable here

Most projects would have to *reconstruct* their development story after the fact. Mycelium does not,
because the house rules already force the raw material into existence:

- **Every decision cites its basis** (G1–G11, tensions, RT/S invariants, research records). The *why* is
  not lost — it is normative.
- **Every doc is append-only with a `Meta — changelog`.** The decision *timeline* (Draft → Accepted →
  Enacted → Superseded) is recorded, including supersessions (the dead-ends and course-corrections).
- **RFC `Rationale & alternatives` / `Drawbacks` sections** already capture the roads *not* taken and the
  costs accepted — the heart of an honest engineering narrative.
- **Commit messages** state which FR/NFR/VR/SC a change advances and *how it was verified* (CONTRIBUTING).
- **Research records** ground the prior art.

So the corpus is already ~80% of a grounded story. The missing piece is the **messy middle** — the
*struggle* and the *problem-solving how* that a polished RFC deliberately smooths over.

## 2. What to capture (the data model)

| Layer | The question it answers | Where it lives today | Gap? |
|---|---|---|---|
| **Decision + rationale** | *what* we chose and *why* | RFC/ADR/DN bodies + changelogs | covered |
| **Grounding / provenance** | *on what basis* | citations (G*, T*, RT*, research) | covered |
| **Timeline / supersession** | *when, and what it replaced* | per-doc append-only changelogs | covered |
| **Tradeoffs / dead-ends** | *what we rejected and the cost we accepted* | RFC §Rationale/§Drawbacks | mostly covered |
| **Struggle / problem-solving how** | *what was hard, what broke, how we got unstuck* | — (commit msgs, partial) | **the gap → §3 devlog** |
| **Verification story** | *how we knew it worked* | commit "verified by …" + tests | covered |

## 3. The capture discipline (lightweight, append-only)

Keep doing what already works (per-doc changelogs; grounded commit messages). Add **one** low-friction
artifact for the gap:

- **`docs/devlog/` — dated narrative entries.** Append-only, informal, one file per session/wave:
  `YYYY-MM-DD-<slug>.md`. Each entry captures, per notable problem, a compact **problem → why it mattered
  → approach (incl. what we *didn't* do) → resolution**, with refs to the commits/docs it produced. This
  is the *story* layer the formal docs intentionally omit — the seed material for blog posts and the
  book's "design rationale" asides. It is **not** normative (the RFCs remain the source of truth); it is
  *narrative*, and like everything else it stays honest (no embellishment; cite what shipped).
- The first entry — `docs/devlog/2026-06-16-rfc0008-integration-wave.md` — is seeded alongside this note
  as a worked example of the format.

## 4. The three authoring outputs (the pipeline, sketched)

All three are **synthesis from the cited corpus**, never free invention — the *same* grounding discipline
applied to prose, with a human in the loop ("partially automated" = **draft-then-review**, never
auto-publish):

- **(a) Blog posts — partially automated.** Short, narrative pieces ("how we unified the budget
  mechanism", "the colony naming collision") drafted from the **devlog + changelogs**, then edited. The
  devlog entry *is* the outline; the post is its readable form.
- **(b) The language book — partially/fully automated.** The *pedagogical* narrative: concepts → worked
  examples → design-rationale asides. Synthesized from the **RFC guide-level sections** (each RFC has a
  "Guide-level explanation"), the **Example Programs Reference**, and the **Glossary** — sequenced for a
  learner. The honesty/EXPLAIN ethos becomes the book's through-line.
- **(c) The reference manual — partially/fully automated.** The *normative* synthesis: the authoritative,
  reader-facing projection of the **RFC reference-level designs + grammar + schemas + the lexicon**. A
  *projection of* the spec (like RFC-0013's "a file is a projection of the canonical declaration"), so it
  cannot drift from the corpus — regenerated, not hand-maintained in parallel.
- **(d) Documentation & API reference — *fully* automated.** The most automatable output, because it is
  *pure projection from the code + schemas + metadata*, with no interpretive layer: per-`nodule`/`phylum`
  API docs generated from the **doc-comments + the structured nodule header** (M-359 — `@summary`,
  `@version`, `@license`, authors), the **JSON schemas** (`docs/spec/schemas/`), and the public surface a
  `mycelium-proj.toml` declares. Rust-first now (`rustdoc` over the crates), Mycelium-lang's own
  doc-comments later (dogfooding). **Fully automated** is the *goal* here (the source is already
  normative); the only human gate is reviewing the *generator/templates*, not each page.

Outputs (b)–(d) **ship free with the language** in digital formats (HTML/PDF/EPUB); (d) is also served
live (a browsable site, in-editor hover via the LSP).

### 4.1 Format & quality bar (clean · presentable · legible · intelligible · digestible)

Generation is necessary but **not sufficient** — generated docs are infamously ugly, and "fully
automated" is worthless if the result is unreadable. The output must clear a deliberate quality bar, and
that bar is itself a *checkable contract* (the toolchain, M-361, can lint it), not a hope:

- **One consistent, reviewed template.** A single predictable structure and visual language across blog /
  book / manual / API ref — a reader learns the shape once.
- **Navigable: index → detail, always.** The `Doc-Index` / `Glossary` "summarized index that points into
  the detail" pattern, applied everywhere; every page reachable in a few clicks; a search index.
- **Progressive disclosure (graded depth).** Reuse RFC-0013's `minimal / medium / detailed` **level**
  idea for *docs*: a reader picks how deep to go — a one-line summary, the working explanation, or the
  full normative detail — over *one* truth, never three divergent ones.
- **Worked examples inline.** Every concept carries a runnable example (the Example Programs Reference is
  the source); examples are *checked* (they compile/run), so the docs can't rot.
- **Dual human + machine projection (G11).** Human formats (HTML/PDF/EPUB) *and* a structured machine form
  (JSON/JSONL) for search/tooling/LSP hover — two renderers of one content-addressed truth (RFC-0013's
  posture), sharing identity (ADR-003), so links are stable and never dead.
- **Legible by construction.** Readable typography, real code highlighting, accessible semantic HTML (alt
  text, headings, contrast), responsive layout. Legibility is part of "honest" — a true statement nobody
  can read isn't communicated.
- **Honest like the rest (never-silent for docs).** A missing/undocumented item is **flagged**, never
  papered over with invented prose; an API doc statement traces to the code/spec it projects (no
  hallucinated descriptions). "Undocumented" is an explicit, visible state — the doc analogue of G2.

## 5. Honesty constraints on automated authoring (non-negotiable)

The automation is held to the **same** rules as the language it documents:

- **Grounded + cited (no hallucination).** Every claim traces to a corpus source; an authoring tool that
  cannot cite a statement flags it rather than asserting it — the prose analogue of never-silent (G2) and
  the honesty lattice (a claim with no basis is `Declared`/flagged, never dressed as fact).
- **Projection, not a parallel truth (no drift).** The manual/book are *generated from* the RFCs/specs;
  the corpus stays the single source of truth (the RFC-0013 "projection of the canonical declaration"
  posture, ADR-003 content-addressing makes drift detectable).
- **Human-in-the-loop.** "Partially automated" is the floor: a draft is *reviewed* before it ships;
  "fully automated" is a goal only for the *projection* parts (manual) where the source is already
  normative, never for interpretive narrative (blog/book asides).
- **Append-only provenance.** The devlog and generated artifacts record what they were generated *from*
  (commit/doc refs), so any reader can trace a sentence back to the decision that produced it.

## 6. Scope now vs. later

- **Now (this note + the seed devlog):** the *intent*, the *data model*, the *capture discipline*, and
  the *honesty constraints* — so the right material starts accumulating immediately (the devlog), and the
  pipeline has a spec to build against.
- **Later (a fresh session — M-363):** the actual authoring tooling (the generators, the templates, the
  HTML/PDF/EPUB build), its place in the Phase-8 toolchain (M-361), prior-art tracing (docs-as-code,
  literate programming, `mdBook`/Sphinx/Antora, "books generated from a spec"), and ratification.

## Meta — changelog

- **2026-06-16 — Pipeline design landed (M-363, design-first).** The "later" of §6 is now designed:
  `docs/spec/Narrative-Authoring-Pipeline.md` (**Proposed**) specifies the **one content-addressed doc IR
  → many renderers** architecture (HTML/PDF/EPUB + machine JSON), the four projection generators
  (apiref/manual/book/blog) with their corpus sources, one reviewed template, and the **§4.1 quality bar
  as a checkable 8-point lint** (incl. *checked examples* and *undocumented-is-flagged*). Prior art traced
  in `research/07-narrative-authoring-pipeline-RECORD.md` (T7.1–T7.7). The **build stack and format choices
  are flagged for ratification — no pipeline code lands until ratified** (this note's §6 "ratification"
  gate). Append-only.
- **2026-06-16 — Added fully-automated docs + API reference + the format quality bar.** Per the
  maintainer's future-planning addition: a fourth output **(d) Documentation & API reference — *fully*
  automated** (the most automatable, pure projection from code + schemas + the M-359 nodule-header
  metadata; rustdoc now, Mycelium-lang doc-comments later; shipped free + served live/LSP-hover), and a
  new **§4.1 Format & quality bar** making "clean · presentable · legible · intelligible · digestible" a
  *checkable* contract (one consistent template; index→detail navigation; progressive-disclosure graded
  depth reusing RFC-0013's levels; checked inline examples; dual human/machine projection — G11/ADR-003;
  legibility/accessibility by construction; and "undocumented is flagged, never invented" — the doc
  analogue of never-silent G2). Folded into M-363. Append-only.
- **2026-06-16 — Created (initial capture).** Records the maintainer's intent (2026-06-16) to capture
  enough development narrative — decisions, struggles, problems solved, the how **and** why — to enable
  **partially-to-fully automated** authoring of project **blog** posts, a **language book**, and a
  **reference manual**, distributed **free** in digital formats. Notes that the honesty rule already makes
  the corpus a grounded, cited, append-only narrative (~80% of the raw material), identifies the **gap**
  (the struggle / problem-solving *how*), and fills it with a lightweight append-only **`docs/devlog/`**
  (seeded with `2026-06-16-rfc0008-integration-wave.md`). Sketches the three authoring outputs as
  **synthesis from the cited corpus** under the same honesty discipline (grounded/cited, projection-not-
  parallel-truth, human-in-the-loop, append-only provenance). Full pipeline design + tooling is a fresh
  session, tracked **M-363** (Phase 8). Append-only.
