# Research Record 11 — Semantic-level projections: design grounding + the empirical gate (RFC-0021 / RP-1, RP-4)

> **What this file is.** A durable record that **grounds the non-empirical design** of RFC-0021
> (Semantic-Level Projection Framework) and **states precisely which gate it cannot discharge**. It
> addresses the design-decidable parts of **RP-4** (semantic-projection ergonomics, `docs/notes/
> research-prompts.md`) and explains why **RP-1** (the KC-2 retention-ratio ablation) and RP-4's
> LLM-leverage sub-question are **irreducibly empirical** — they require LLM runs this pass cannot
> perform — and supplies a **turnkey protocol** so the run is mechanical when compute is available.
> Conducted 2026-06-18 from the RFC-0021 draft, RFC-0006 §8 Q1/§9, DN-09 §4, the `experiments/`
> harness (M-002), and the projectional-editing / content-addressing / constrained-decoding
> literature. Findings are labeled **T11.1–T11.7** (continuing the T0–T10 scheme).
>
> **Posture (honesty rule / VR-5).** This record **does not produce any LLM-leverage result** and
> **must not be read as discharging the retention-ratio gate.** It grounds the *framework design*
> (which is design-decidable) and tags every empirical claim it touches as **open** (`Declared`,
> per DN-09 §4 and RFC-0006 §8 Q1). Pre-writing the verdict is forbidden (VR-5). Append-only.

---

## 1. Scope

RFC-0021 is **Draft**, gated by two prompts (its §9): an **ergonomics/feasibility** prompt (can a
total, dumpable, semantic-level projection be authored over the L1/L2 node set?) and an **LLM-leverage**
prompt (does an LLM-facing canonical projection raise pass@1 above the committed text surface, clearing
the T3.6 threshold?). This record splits them honestly:

- **Design-decidable** (dischargeable here): the projection model + invariants P1–P6 grounding
  (T11.1–T11.2); the dual human/machine rendering architecture (RP-4 sub-q 3, T11.3); the
  authoring-feasibility assessment (T11.4); the human-usability posture recommendation (RP-4 sub-q 1,
  T11.5).
- **Irreducibly empirical** (NOT discharged here): the LLM-leverage retention ratio (RP-1; RP-4
  sub-q 2). T11.6 states why; T11.7 gives the turnkey protocol.

---

## 2. Findings

### T11.1 — The projection model is grounded prior art, not novel

"A projection is a total, inspectable function from a content-addressed L1/L2 node tree to a rendered
surface; identity is the content hash, not the rendering" (RFC-0021 §3.1) is the **Unison** model
directly: Unison stores definitions content-addressed (codebase-as-database), with names and renderings
as *metadata over one hash* — exactly RFC-0021 P4. **JetBrains MPS** is the production
**projectional-editing** precedent: multiple notations/projections over one underlying AST, edited
through the projection. So the framework's core (multi-projection over one content-addressed
definition) is **established**, not flagged-novel. What is Mycelium-specific is the *honesty overlay*
(P2/P3/P6), addressed next. Grounding: Unison (codebase-as-database; ADR-003 cites it); MPS (Voelter et
al., projectional editing); RFC-0021 §3.1/§4.5.

### T11.2 — Invariants P1–P6 are sound by construction (the honesty overlay is an additive, locally-checkable constraint)

P1 (no meaning change) is the projectional-editor "view, not model" discipline. P4 (identity = hash) is
Unison (T11.1). P5 (dumpable/diffable) is ADR-006. The Mycelium-specific invariants — **P2** (honest
tags survive), **P3** (`Swap` never elided), **P6** (EXPLAIN survives) — are **additive constraints on
the projection function**: a projection rule set is rejected at definition time if any node kind whose
rendering could drop a guarantee tag, a `Swap`, or an EXPLAIN record is present. This is **locally
checkable** over the closed node set (it is a per-node-kind property of the rule table), and it is the
same "never-silent" discipline the LSP feedback facade already enforces (e.g. the M-390 prim-site and
the swap-site surfacing both refuse to drop a node silently). So P1–P6 are **sound by construction**,
not a research risk. Grounding: RFC-0021 §4.3; ADR-006; `crates/mycelium-lsp/src/feedback.rs` (the
never-silent surfacing precedent).

### T11.3 — RP-4 sub-question 3 (dual rendering): one architecture suffices

*Question:* does a single projection architecture satisfy both human-idiomatic and LLM-canonical
rendering, or do they need different designs? *Answer (grounded):* **one architecture.** Unison already
serves multiple renderings of one content-addressed store (the pretty-printer + the codebase UI); MPS
serves multiple projections of one AST. A human-idiomatic projection and an LLM-canonical projection are
two `Projection` values (RFC-0021 §4.1) over the same hash, differing only in their rule tables and
`target: SurfaceKind`. No second architecture is required; FR-S5 dual intelligibility is **one
content-addressed store, many projections**. **RP-4 sub-question 3 is answered.** Grounding: Unison; MPS;
RFC-0021 §4.1/§3.4; FR-S5.

### T11.4 — Authoring feasibility (RFC-0021 §9 ergonomics gate): a grounded *assessment*, not a measured study

The L1 node set is **small and closed**: `Const | Var | Let | Op | Swap | Construct | Match | Lam | App
| Fix | FixGroup` (11 kinds). A projection is a **total function over those 11 cases** — structurally
the same shape as two traversals that **already exist and are maintained by one engineer**:
`mycelium-lsp::feedback::collect` (walks every node kind) and `mycelium-core::lower::stages` (the
dumpable lowering stages, M-112). Authoring a projection is therefore comparable to a Rust visitor /
`Display` impl over an 11-variant enum, and **dumpability is free** because RFC-0021 §4.2 requires the
rules to be *declared data*, not compiled closures (ADR-006). **Assessment: authoring a total, dumpable
projection over the current grammar is feasible at single-engineer scale.**

*Honest tag.* This is a **feasibility assessment grounded in the existing node-walk code**, **not** the
measured authoring-cost study RP-4 sub-q 1 calls for (authoring time, rule count, dumpability legibility
over the *growing* L2 grammar). That measured study — and how cost scales as L2 adds inference/traits/
modules — remains **open** (a prototyping follow-up, ideally a real `LlmCanonical` renderer over
`mycelium-core`). So the ergonomics gate is **partially** addressed: feasibility = grounded *yes*;
measured cost = open. Grounding: `mycelium-lsp::feedback`, `mycelium-core::lower`; RFC-0021 §4.2/§9.

### T11.5 — RP-4 sub-question 1 (human usability): adopt the Unison posture (edit-in-text, project read-mostly)

*Question:* does editing a *projected view* feel faithful, or does the projection↔IR mismatch create
friction (the Foundation §6 "projectional-editor usability friction" risk)? *Grounded reading:* the
projectional-editing literature is **mixed** — MPS carries a documented learning-curve/usability cost
(structured editing is unfamiliar; Voelter et al. usability studies), while **Unison sidesteps most of
it** by editing in *ordinary text* and content-addressing **on save** (not full projectional editing).
*Recommendation (de-risks G11):* adopt the **Unison posture** — the committed **text surface**
(RFC-0020) is the *primary authoring* mode; projections are **read-mostly views** plus an **opt-in**
`RoundTrip` capability (RFC-0021 §4.1 `EditCapability`). This confines the "edit a projected view"
friction to the opt-in case and keeps the high-traffic path on familiar text. This matches RFC-0021
§3.5 (text grammar = the edit-primary projection) and DN-09 §3.2 (usability-first). **RP-4 sub-q 1 is
addressed as a design recommendation**, not a user study (which stays a follow-up). Grounding: Unison;
MPS usability studies; Foundation §6; RFC-0021 §3.5; DN-09 §3.2.

### T11.6 — The LLM-leverage gate is irreducibly empirical and is NOT discharged here

RFC-0021's second gate — "does an `LlmCanonical` projection raise the LLM-leverage retention ratio above
the committed text surface, clearing the T3.6 < ~70% threshold?" — is **RP-1** (the KC-2 retention-ratio
ablation) and RP-4 sub-question 2. It requires **LLM runs**: ≥5 conditions, ≥3 seeds, ≥1 frontier model,
and a grammar-constrained decoder, over a composition-task set (DN-09 §4; RFC-0006 §8 Q1). **This pass
cannot perform those runs**, and per VR-5 / DN-09 §4 **no leverage result may be asserted without running
them** — "novel-but-regular syntax retains most leverage" remains a *supported-but-not-confirmed* working
hypothesis, and the canonical-projection-lift claim is `Declared`, not `Empirical`. **This record makes
that gate's status explicit and leaves it open.** It is the one part of the four-RFC ratification wave
that cannot be advanced by analysis alone. Grounding: DN-09 §4; RFC-0006 §8 Q1; RFC-0021 §4.7/§9.

### T11.7 — Turnkey protocol for the empirical gate (a protocol, not a result)

So the run is mechanical when compute is available, reusing the existing `experiments/` harness (the
M-002 infrastructure: model runners, `primers/`, `results/`, `kc2-report.json` schema):

1. **Arms (RFC-0006 §8 Q1 / DN-09 §4):** (1) bare novel text surface; (2) + book-quality
   grammar-in-context primer; (3) + grammar-constrained decoding (a **new** dependency — GBNF via
   `llama.cpp`, or Outlines/Guidance — measured *separately* from arm 2); (4) **`LlmCanonical`
   projection** (the familiar-skin same-AST arm this RFC enables — requires a projection renderer over
   `mycelium-core`, T11.4); (5) embedded-DSL baseline (RR-3).
2. **Controls:** ≥3 seeds; ≥1 frontier model (beyond the local ≤7B used in M-002); a composition-task
   subset wider than `kc2-01…10`.
3. **Headline metric:** **retention ratio** = pass@1(best novel-surface arm) ÷ pass@1(familiar-skin
   arm 4), over composition tasks. **Falsification threshold:** retention < ~70% ⇒ L3 must become a
   projection of known syntax (`LlmCanonical`-primary; RFC-0021 §4.7 trigger; RFC-0006 §8 Q1).
4. **Output discipline:** report each arm at the **strength of the arms actually run** —
   `Empirical` (measured) or `Declared` (asserted) — and **never pre-write the verdict** (the harness
   refuses to, `experiments/README.md` §3; VR-5). The threshold comparison applies only when arm 4 is
   present.

New build dependencies this protocol needs (none of which exist yet): the grammar-constrained decoder
integration (arm 3) and the `LlmCanonical` projection renderer (arm 4) — both tracked under M-380.

---

## 3. Decisions this record supports

- **RFC-0021's framework/design is grounded and design-ratifiable:** the projection model (T11.1),
  the P1–P6 honesty overlay as a locally-checkable additive constraint (T11.2), the single dual-rendering
  architecture (T11.3, RP-4 sub-q 3 = yes), and authoring feasibility (T11.4). A maintainer **could
  ratify the framework** at this scope (the RFC-0006 r5 carve-out pattern), with the **LLM-leverage
  claim explicitly carved out as empirically open**.
- **Human-usability posture recommendation:** edit-in-text, projections read-mostly + opt-in RoundTrip
  (the Unison posture, T11.5) — de-risks G11.
- **The LLM-leverage gate (RP-1 / RP-4 sub-q 2) stays OPEN** (T11.6); the turnkey protocol (T11.7) makes
  the run mechanical but is **not** a result. RFC-0021's §4.7 fallback trigger remains `Declared`.
- **Net:** of RFC-0021's two gates, the **design/ergonomics-feasibility** gate is grounded (with a
  measured-cost follow-up), and the **LLM-leverage** gate is honestly **not** dischargeable without
  compute. RFC-0021 is the one RFC of the four that **cannot** reach full ratification by this research
  pass — and saying so is the honest outcome.

---

## 4. Key sources

- **Unison** (codebase-as-database; content-addressed definitions; names/renderings as metadata) —
  the projection-model precedent (T11.1/T11.3); cited by ADR-003.
- **JetBrains MPS** (projectional editing; Voelter et al. usability studies) — the multi-projection
  precedent and the usability-cost evidence (T11.1/T11.5).
- **Hazel / Lamdu** — structured/projectional editors (usability context, T11.5).
- **Grammar-constrained decoding:** `llama.cpp` GBNF; **Outlines** (Willard & Louf, *Efficient Guided
  Generation for LLMs*, 2023); Guidance — the arm-3 dependency (T11.7).
- **MTOB** (Tanzer et al., *A Benchmark for Learning to Translate a New Language from One Grammar
  Book*, 2023) and **grammar prompting** — the grammar-in-context evidence behind the working
  hypothesis (T11.6); cited by RFC-0006 §8 Q1.
- In-repo: RFC-0021 (the framework, §3–§4, §9); RFC-0006 §8 Q1/§9; DN-09 §4 (the honest-scope
  statement); `experiments/` (the M-002 harness, the protocol's substrate); `crates/mycelium-lsp/src/
  feedback.rs` and `crates/mycelium-core/src/lower.rs` (the node-walk/dumper grounding T11.4);
  ADR-003/ADR-006.

---

## 5. Honest-uncertainty register

- **The LLM-leverage gate is NOT discharged.** No run was performed; no retention ratio is reported;
  the canonical-projection-lift claim stays `Declared` (VR-5; DN-09 §4). T11.7 is a *protocol*, not a
  result, and explicitly cannot be read as one.
- **Authoring feasibility (T11.4) is an assessment, not a measurement.** It is grounded in the existing
  node-walk code, but the measured authoring-cost study over the *growing* L2 grammar (RP-4 sub-q 1) is
  open.
- **The human-usability recommendation (T11.5) is grounded in mixed external prior art, not a Mycelium
  user study.** The "Unison posture de-risks G11" claim is a design judgment, not measured.
- **"Constrained decoding is expected to lift leverage"** (arm 3 rationale) is a hypothesis from
  MTOB/grammar-prompting, **not** measured here.
- **MPS/Unison usability evidence is qualitative and external;** its transfer to Mycelium's specific
  projections is an inference, flagged as such.

---

## Meta — changelog

- **2026-06-18 — Created.** Grounds RFC-0021's *non-empirical* design: the projection model + P1–P6 as
  established prior art with a locally-checkable honesty overlay (T11.1–T11.2), the single dual-rendering
  architecture (T11.3, answering RP-4 sub-q 3), an authoring-feasibility assessment grounded in the
  existing node-walk code (T11.4), and the Unison-posture human-usability recommendation (T11.5,
  addressing RP-4 sub-q 1). States that the **LLM-leverage gate (RP-1 / RP-4 sub-q 2) is irreducibly
  empirical and is NOT discharged** (T11.6), and supplies a turnkey five-arm protocol over the existing
  `experiments/` harness (T11.7). Recommends the maintainer may ratify the *framework* with the
  LLM-leverage claim carved out as empirically open. No leverage result asserted (VR-5). Append-only.
