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

Both the book and the manual **ship free with the language** in digital formats (HTML/PDF/EPUB).

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
