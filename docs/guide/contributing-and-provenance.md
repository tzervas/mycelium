# Contributing conventions, and provenance & evidence

One-line purpose: the short version of the house rules for contributors, and where the corpus's
claims trace back to. The full contributor process lives in `CONTRIBUTING.md`; this page is a
quick-reference summary, not a replacement.

## Contents

- [Conventions for contributing](#conventions-for-contributing)
- [Provenance & evidence](#provenance--evidence)

## Conventions for contributing

> Full detail (process, dev environment, workflow) is in [`CONTRIBUTING.md`](../../CONTRIBUTING.md).
> In brief:

- **Decisions are append-only.** Don't silently edit an ADR/RFC decision — supersede it with a
  new status (`Draft/Proposed → Accepted → Enacted → Superseded`) and link forward. Every claim
  cites its grounding (survey labels `G*`/`A–E`/`R*`; research labels `T0.x/T1.x/T2.x`).
- **Transparency rule.** Guarantee tags are assigned **per model and per operation**, never in
  aggregate. A bound may be tagged `Proven` *only* if it cites a theorem whose side-conditions
  are checked; otherwise it is `Empirical` (validated) or `Declared` (user-asserted, always
  flagged). New results may *upgrade* a tag; absence keeps it weaker.
- **No black boxes.** Any feature that introduces opaque behavior (especially "intelligent"
  automatic selection) must be reified, inspectable, and explainable (`EXPLAIN`).
- **Engineering principles** (the project's house style): SRP, OCP, LSP, ISP, DIP, DRY, KISS,
  YAGNI, Law of Demeter, separation of concerns; **composition over inheritance**; PEP 8 and
  `ruff format` for Python.
- **Squash-only into `main`.** Every PR lands as a single curated squash commit (a clean linear,
  bisectable history); internal swarm integration merges (leaf→epic→orch) stay octopus/`--no-ff`
  to preserve lineage. The `/land` skill drives the autonomous self-review, green `just check`,
  curated squash-merge, and cleanup loop.
- **Kill criteria** (KC-1…KC-4) are re-checked at every phase gate; a gate that doesn't check
  them is hiding risk.

## Provenance & evidence

Everything in `docs/` traces back to the research passes recorded in `research/` — **twenty-seven
records** (`01`…`27`) as of this writing, spanning the prior-art survey and T0/T1/T2 findings
through the language layer, runtime/concurrency, error-recovery and bounded effects,
automatic-baseline diagnostics, the narrative-authoring pipeline, honest-stdlib prior art, stage-1
grading non-interference, traits/coherence and Repr-polymorphism, the semantic-projection
framework, the web-tooling and ADK phyla (RFC-0022/0023), the value-model integration report, the
embeddonator leverage map, env-machine reclamation (internal + prior art), the Rust-to-Mycelium
transpiler (internal + prior art), corpus-alignment audit, safe iteration (internal + prior art),
the object-behavior model (internal + prior art), layered-lowering and generative-sugar prior art,
and the DN-64 type-system and ergonomics R&D dispositions.

Each record carries its structured findings and source list; normative claims in `docs/` cite
their grounding (survey labels `G*`/`A–E`/`R*`; research labels `T0.x`…`T13.x` and beyond) or are
flagged as open questions. The count above is verified against the current `research/` directory
listing rather than copied from an earlier snapshot — check `ls research/` if it may have grown
further since.

---

**See also:** [Decisions & reading order](decisions-and-reading-order.md) · [Status & roadmap](status-and-roadmap.md)

[← Back to README](../../README.md)
