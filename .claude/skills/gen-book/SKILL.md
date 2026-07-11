---
name: gen-book
description: >-
  Generate the validated "read" (prose) that accompanies extracted code facts —
  language-book chapters, reference-manual entries, and learning-corpus lessons —
  from the committed index JSON, with a mandatory faithfulness gate so nothing is
  hallucinated. Wraps tools/llm-harness/narrate/ (Generator -> FaithfulnessChecker
  -> Loop): only validated, cited prose is committed; the rest is dropped and
  reported (never silently). Deterministic Mock backends by default (offline/CI).
when_to_use: >-
  Use when producing prose documentation for a nodule/module or a book section
  from the extracted facts (docs/lib-index, docs/api-index) — one chapter/entry/
  lesson at a time. Also when adding a new prompt target, extending the grounding
  checker, or wiring a real-LLM backend behind the same validation gate. Not for
  regenerating the index itself (that's /doc-index).
allowed-tools: Bash(python3:*), Bash(pytest:*), Bash(ruff:*), Read, Grep, Glob
---

# /gen-book — validated narrative generation (the "read" for the corpus)

Generates prose for the language book, the reference manuals, and a learning
corpus **from grounded facts**, with a **mandatory anti-hallucination gate**. It
is the prose-vs-facts analogue of the transpiler vet loop's `checked_fraction`
(`crates/mycelium-transpile/src/vet.rs`): only validated prose is committed, the
rest stays `Declared` / is dropped, and a `validated_fraction` is reported —
never-silently (G2). Backing package: `tools/llm-harness/narrate/`.

## The DN-96 context-windowing discipline (one unit = one bounded working set)

Narrate **one** chapter/module/section at a time. Each unit is a bounded working
set: **load its facts → generate → validate → persist the validated prose → drop
it from context → next unit.** Never hold the whole corpus in context at once.

```
for each unit (nodule / module / book section):
  1. LOAD    facts = narrate.facts.load_facts(index_json, unit)   # consume, don't re-extract
  2. GENERATE prose via a parameterized template (Mock by default)
  3. VALIDATE with the FaithfulnessChecker (grounding + doc_refs gate)
  4. PERSIST  only the validated sentences + a provenance header
  5. DROP     the unit from context; move to the next
```

## Inputs — consume the committed index JSON (never re-extract)

Facts come from the already-committed, deterministic indices — the ground-truth
pointers, tagged `Empirical/Declared`:

- `docs/lib-index/index.json` — the `.myc` stdlib surface (per-`nodule` items)
- `docs/api-index/index.json` — the Rust-crate surface (per-`module`/`crate`)

`narrate.facts` normalises either schema to `Fact` records
(`id, kind, unit, source_path:line, signature, summary, guarantee_tag`). A
missing summary becomes an **explicit `documented=False` fact** ("undocumented"),
never invented away (G2).

## Parameterized prompt family (templates are files)

Three targets, each a template under `tools/llm-harness/narrate/prompts/`
(reference them; don't duplicate their text here):

- `book-chapter.md.tmpl` — a language-book chapter section
- `ref-manual-entry.md.tmpl` — a reference-manual entry
- `learning-lesson.md.tmpl` — a learning-corpus lesson

Each template has an INSTRUCTIONS block (for a real LLM) and an EMIT SKELETON the
deterministic `MockGenerator` fills from the facts. **Idempotent**: the cache key
is a blake2b hash over `(facts + full template text + model-id + seed=42)`, so a
re-run returns byte-identical prose and any input change regenerates.

## The validation gate (the crux — mandatory, non-negotiable)

`narrate.checker.FaithfulnessChecker` (Mock: deterministic; a `Checker` protocol
lets a real adversarial-LLM verifier drop in) enforces, per sentence:

- **(a) doc_refs gate** — every paragraph must carry ≥1 *resolvable* `doc_refs`
  token (`api:` / `corpus:` / `src:` grammar, per `tools/github/doc_refs_check.py`),
  and the token must be one the unit's facts license.
- **(b) claim-grounding gate** — every code token a sentence uses (backticked
  identifiers + bare snake_case/camelCase) must be in the fact-set vocabulary. A
  token absent from the facts ⇒ the sentence is an ungrounded hallucination.
- **(c) `validated_fraction`** = validated sentences / total. **Only validated
  sentences are committed;** dropped ones are listed with reasons (never-silent).

Guarantee posture (VR-5): mock output is `Declared`, a real model's is
`Empirical` — **never `Proven`/`Exact`** (enforced by `session.assert_model_tag`,
mirroring `coauthor.py`).

## Run it

```bash
cd tools/llm-harness

# End-to-end demo on one real lib/std unit (Mock backends, offline):
python3 -m narrate.demo          # loads std.result facts -> ref-manual entry
                                 # -> validates -> writes out/ + reports/
                                 # -> prints validated_fraction

# The test suite (loader, cache idempotence, checker + negative control, loop):
pytest narrate -q                # or: python3 -m pytest narrate
ruff check narrate && ruff format --check narrate
```

Outputs land in `tools/llm-harness/narrate/out/<unit>.<target>.md` (validated
prose + a deterministic provenance header) and dual JSON+human reports in
`narrate/reports/` (G11). Both are gitignored — reproducible on demand.

## Extending

- **A new target**: add `prompts/<target>.md.tmpl` (front-matter `target:` + an
  EMIT SKELETON), add a fact-renderer branch in `narrate.generator`, and register
  it in `narrate.prompts.TARGETS`.
- **A real LLM backend**: implement the `narrate.generator.Generator` protocol
  (tag output `Empirical`, SKIP when unavailable — see the `LlmNarrator` stub);
  the loop and the validation gate are unchanged.
- **A stronger checker**: implement the `narrate.checker.Checker` protocol (e.g.
  an adversarial-LLM verifier); `MockChecker` is the deterministic reference.

Honesty survives every extension (VR-5/G2): validated-only commits, explicit
`validated_fraction`, never-silent drops, no black boxes.
