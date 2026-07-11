# Design Note DN-114 — Validated Narrative Generation (a sentence-level faithfulness oracle for the `gen-book`/`gen-manual` interpretive layer, so generated prose never hallucinates)

| Field | Value |
|---|---|
| **Note** | DN-114 |
| **Status** | **Draft** (2026-07-11) — proposes a **faithfulness oracle** that gates the *interpretive* prose the Narrative-Authoring-Pipeline's `gen-book`/`gen-manual` generators emit (M-363), so the "actual read" can be **generated** — via parameterized, idempotent prompts over context chunks — while remaining **validated (no hallucination, no trash)**. **Refines** `docs/spec/Narrative-Authoring-Pipeline.md` §4/§4.1/§6 and the *Narrative-Capture-and-Authoring* §5 draft-then-review floor; **enacts nothing** and **moves no other doc's status** (house rule #3, append-only). Requires maintainer ratification to move Draft → Accepted. All tags `Declared` unless a cited source holds them higher (VR-5). |
| **Decides** | *Proposes, for ratification:* (a) a **sentence-level faithfulness oracle** — every generated interpretive sentence must be *grounded in its cited extracted facts and nothing else*, checked by a resolvable-`doc_refs` requirement + an adversarial claim-grounding pass, yielding a **`validated_fraction`** (the honest number, modeled on the transpiler's `checked_fraction`); (b) **commit-only-validated**: only sentences that pass are written to the corpus; the rest are **dropped, never-silently reported** (G2), and unvalidated material stays `Declared`; (c) **idempotent, content-addressed generation** (cache key = hash of facts + template + model-id + seed) so re-runs are stable and updates are **differential**; (d) the operationalization as a `narrate/` harness on the `coauthor.py` skeleton + a `/gen-book` skill applying DN-96 context-windowing. |
| **Consumes** | `docs/spec/Narrative-Authoring-Pipeline.md` (§1 "no hallucinating prose"; §4 total-projection / "flag undocumented, never invent"; §4.1 quality-bar lint; §6/G2); `docs/notes/Narrative-Capture-and-Authoring.md` (§5 draft-then-review floor); `tools/llm-harness/coauthor.py` (the generate→validate→feedback skeleton, VR-5 tag gate, dual reports); `crates/mycelium-transpile/src/vet.rs` (the `checked_fraction` oracle discipline); `tools/github/doc_refs_check.py` (the `api:`/`corpus:`/`src:` grounding grammar); `crates/mycelium-doc/src/{apiref,lib_index,doc_lint}.rs` (the validated extraction basis); DN-96 (context-windowing); DN-09 (KC-2 verdict: LLM leverage sanctioned). |
| **Feeds** | `tools/llm-harness/narrate/` (the harness, landed Draft with this note); `.claude/skills/gen-book/` (the skill); on ratification, the M-363 `gen-book`/`gen-manual` build + `docs/spec/Narrative-Authoring-Pipeline.md` §4.1 (a new faithfulness check) + a `CHANGELOG`/`Doc-Index` row (FLAGged, not edited here). |
| **Grounds on** | `research/07-narrative-authoring-pipeline-RECORD.md` (T7.1–T7.7); the `checked_fraction`/`/transpile-vet` precedent (DN-34 §8.7–§8.8). |
| **Date** | July 11, 2026 |
| **Task** | doc-generation — design + scaffold the validated-narrative generator. |

> **Posture (transparency rule / VR-5 / G2 / house rule #4).** This note **works up a decision** and ships
> the `narrate/` harness + `/gen-book` skill as **Draft** operational material; it does **not** ratify the
> generated-prose program (a maintainer does). **No sycophancy:** §7 states the real limits plainly — the
> shipped faithfulness *checker* runs a **deterministic lexical stand-in** (`MockChecker`) for what a real
> adversarial-LLM verifier would do, so `validated_fraction` is an **`Empirical`/`Declared` lower-bound
> proxy**, not a proof of semantic faithfulness; the human draft-then-review floor is **not removed**, only
> given an automated pre-gate. Generated prose is **always `Empirical`/`Declared`, never `Proven`/`Exact`**
> (VR-5). The oracle is designed so that its *failure mode is to drop good prose*, never to admit bad prose
> (a false-negative bias — §2.3).

---

## §1 Problem — "generated read" needs faithfulness, not just provenance

The Narrative-Authoring-Pipeline (Accepted) already forbids hallucinated prose by construction for the
**pure-projection** outputs (`gen-apiref`, `gen-manual` reference bodies): an item that cannot be grounded
is "flagged *undocumented*, never invented" (§4/§6, G2), and `doc_lint` check #7 enforces that every
api-item carries a non-empty `provenance.source`. That is a **provenance-*presence*** guarantee.

But the maintainer's ask — a **generated "read"** for the language book + reference manuals + a
learning/writing corpus — lands squarely on the spec's **interpretive layer**: `gen-book` is "projection
**+ light interpretation** … the interpretive glue is draft-then-review" (§4). Interpretive sentences are
exactly where a generator can *have* a provenance link and still **say something the source does not
support** — a plausible but unfounded relationship, an over-strong claim, an invented rationale. Check #7
cannot catch this: a sentence can cite a real fact and still misstate it. **Provenance-presence ⊉
faithfulness.** This note adds the missing layer.

## §2 The faithfulness oracle

Generation is constrained and then **validated at sentence granularity**, copying the transpiler's honest
discipline: *generate freely, but only commit/claim what passes the checker; the rest stays `Declared`.*

### 2.1 Grounded generation (the input contract)
A generator never sees the open web or model priors as source — only a **context chunk**: the
already-extracted, already-validated facts for one unit (fn signatures + `@summary` + preceding `//`
doc-blocks + **type-checked** examples), read from the committed `docs/lib-index/index.json` /
`docs/api-index/index.json` projections. Undocumented items arrive as explicit "undocumented" facts, never
as blanks to be filled (G2). The prompt instructs: *write only what these facts support; cite each claim.*

### 2.2 The two-gate check (per generated sentence/paragraph)
1. **Resolvable `doc_refs`** — every paragraph carries ≥1 `api:`/`corpus:`/`src:` token that resolves
   (reusing `doc_refs_check.py`'s grammar + resolver). No citation ⇒ reject.
2. **Claim-grounding (adversarial)** — each declarative sentence is checked to be *entailed by ONLY its
   cited facts*. The design target is an **adversarial-LLM verifier** ("list every clause not supported by
   these facts"); the **shipped** implementation is a deterministic `MockChecker` (a lexical stand-in that
   flags sentences introducing identifiers/claims absent from the fact set) plus a pluggable `Checker`
   protocol for the real pass. Either way the output is a per-sentence pass/fail.

### 2.3 `validated_fraction` and commit-only-validated
The oracle reports **`validated_fraction` = validated sentences / total** (the number that matters, per
`vet.rs`). Only validated sentences are emitted into the committed prose; failed sentences are **dropped
and reported** (never-silent — the report names each drop + why). A mostly-dropped result is a *successful,
honest* output, not a failure (G2/VR-5). The bias is deliberately **false-negative**: when the checker is
unsure it drops, so the oracle can lose good prose but must never admit unfaithful prose.

## §3 Idempotence & differential update
Generation is **content-addressed and deterministic**: cache key = `blake2b(facts ‖ template ‖ model-id ‖
seed)`. Re-running an unchanged unit returns cached prose ⇒ stable diffs and **differential regeneration**
(only changed units re-generate). This mirrors, and composes with, the doc-IR's `DocModel::id_set()`
differential-render cache (a source line that reflows keeps its node id, so committed prose + rendered page
stay stable). The generated prose is committed corpus (mirroring the `docs/*-index/` pattern: a regen
recipe plus a drift gate plus a `linguist-generated` mark), so it ships in the `git archive` release
artifact like the rest of `docs/`.

## §4 Honesty tags & the human floor
Generated prose is **always tagged `Empirical`/`Declared`, never `Proven`/`Exact`** (enforced as in
`coauthor.py`/`harness.py` V-03). Committed generated prose carries a provenance header (source facts,
model-id, `validated_fraction`, tag). **The draft-then-review floor (spec §4, Capture §5) is preserved** —
the oracle is an *automated pre-review gate that raises the floor*, not a replacement for the human gate.
Unreviewed generated prose stays `Declared`; a human review (or, later, a passing real-adversarial-verifier
run with a named method) is what can lift a unit to `Empirical`.

## §5 Relationship to the Accepted spec (append-only)
This note **refines, does not supersede**, `Narrative-Authoring-Pipeline.md`:
- §1's "without hallucinating prose" goal gains a *sentence-level* mechanism (was corpus-level).
- §4's `gen-book`/`gen-manual` interpretive layer gains the faithfulness gate before its draft-then-review.
- §4.1's quality-bar lint gains a proposed **9th check** (faithfulness: `validated_fraction == 1.0` for any
  committed generated prose; unvalidated content must not be committed) — to be specified into the spec on
  ratification (FLAGged, not edited here). No status of any other doc moves.

## §6 Operationalization
- **`tools/llm-harness/narrate/`** — the prose `Generator → FaithfulnessChecker → Loop` on the `coauthor.py`
  skeleton: a fact loader over the committed index JSON, a parameterized idempotent prompt family
  (`book-chapter` / `ref-manual-entry` / `learning-lesson`), the two-gate checker, the content-addressed
  cache, dual JSON+human reports. Mock backends make it deterministic/offline/CI-safe; a documented backend
  protocol admits a real LLM later.
- **`/gen-book` skill** — applies DN-96 context-windowing: one chapter/module = one bounded working set
  (load facts → generate → validate → persist validated prose → drop from context → next). This is the
  "small tasks, context chunks, parameterized idempotent prompts" shape the maintainer asked for.

## §7 Trade-offs & honest limitations (no sycophancy)
- **The shipped checker is a stand-in.** The `MockChecker` is lexical; it approximates faithfulness, it does
  not prove it. `validated_fraction` from the mock path is an **`Empirical` lower-bound proxy** — real
  semantic faithfulness needs the adversarial-LLM verifier (deferred; interface shipped). Do not read a mock
  `validated_fraction` as a semantic guarantee.
- **Cost & drift of a second registry** — prompt templates are new material to maintain against the corpus;
  they must be regenerated/reviewed as the fact schema evolves.
- **Residual risk** — a false fact *in the source* propagates faithfully (the oracle checks faithfulness to
  the source, not the source's truth); source correctness remains the corpus's own job (VR-5: source is
  ground truth).
- **Not a bulk auto-publisher** — like `/transpile-vet`, this is a *gap-profiling, honesty-preserving*
  instrument: it tells you how much of a unit's read can be honestly generated, and leaves the rest
  `Declared` for a human.

## §8 Definition of Done (this note)
- The problem (provenance-presence ⊉ faithfulness) and the oracle design are stated and grounded. ✅
- The `narrate/` harness + `/gen-book` skill land as **Draft** with a runnable mock demo + a **negative-
  control test** (an injected unsupported sentence is dropped; `validated_fraction < 1.0`). ✅ (this wave)
- Append-only: refines the Accepted spec, moves no other doc's status, all tags at supportable strength. ✅
- **Ratification gate (maintainer):** move Draft → Accepted, then specify the §4.1 9th check and schedule
  the real-adversarial-verifier backend + the full generation wave (M-363 + the decomposed `M-` tasks).

---

### Changelog
- 2026-07-11 — DN-114 created (Draft): the faithfulness oracle for validated narrative generation; refines
  Narrative-Authoring-Pipeline §4/§4.1/§6; ships the `narrate/` harness + `/gen-book` skill as Draft. Tags
  `Declared` pending ratification (VR-5).
