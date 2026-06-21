# Standing Research Prompts — Mycelium Variant Passes

> **Purpose.** One place to look for all standing research prompts that are tracked across the
> corpus. A *research prompt* is a clearly-scoped question — with a falsification threshold and a
> named decision target — that a future variant pass, spike, or dedicated experiment is expected to
> answer. This index does not duplicate the normative content of the docs it references; it
> summarizes and cross-links so that a new pass can start here.
>
> **Posture (honesty rule / VR-5).** Each entry records the *question* and the *conditions for
> confirmation or falsification*. It does not pre-write the verdict. Entries are append-only:
> add new prompts below existing ones; mark a prompt Resolved with a pointer to the decision it
> fed (do not delete or rewrite).

---

## RP-1 — KC-2 Retention-Ratio Ablation (T3.6)

**Status:** **Open — isolated, NON-BLOCKING validation track (task M-381).** As of 2026-06-18 this
ablation is **split out of RFC-0021** (whose framework is now Accepted *without* it) into its own
non-blocking track: it gates nothing, and its only coupling to the accepted design is a **supersession**
(RFC-0021 §4.7 — on falsification, retention < ~70%, a future RFC promotes `LlmCanonical` to primary).
`research/11` (T11.6–T11.7) confirms it cannot be discharged by analysis — it needs LLM runs (≥5 arms,
≥3 seeds, ≥1 frontier model, a grammar-constrained decoder) — and supplies the **turnkey five-arm
protocol** over the `experiments/` harness. Per VR-5 / DN-09 §4 no retention-ratio result may be
asserted without the run; this stays **Open**. (The M-002 run cleared the KC-2 kill criterion but did
not run this ablation.)

**Question.** Does Mycelium's novel-but-regular text syntax retain most LLM leverage relative to a
familiar-skinned same-AST control, at the rigor T3.6 was designed for? Specifically: what is the
**LLM-leverage retention ratio** of the novel Mycelium surface versus a familiar-skin condition
(same programs, same AST, familiar syntax — e.g. a Rust-like or Python-like skin)?

**Design (RFC-0006 §8 Q1 / T3.6).** Five conditions, ≥3 seeds, a wider task set than kc2-01–10,
at least one frontier model and at least one grammar-constrained decoder:

1. **Bare novel syntax** — Mycelium text surface, no primer.
2. **+grammar-in-context** — same surface + book-quality grammar-in-context primer.
3. **+constrained decoding** — same surface + grammar-constrained decoder (measured separately
   from the grammar-in-context condition).
4. **Familiar-skinned same-AST** — a Rust-like or Python-like reskin of the same programs (same
   AST, different surface lexemes) — the *ablation control* no published study has run.
5. **Embedded-DSL baseline** — the RR-3 fallback condition for reference.

**Headline metric:** the **retention ratio** = (pass@1 on novel surface, best primer condition) ÷
(pass@1 on familiar-skin condition). Report over the composition-task subset.

**Falsification threshold (RFC-0006 §8 Q1).** If the retention ratio is < ~70% (novel surface
retains less than 70% of the familiar-skin leverage), the verdict is that L3 *must become a
projection of known syntax* (M-380 path becomes mandatory, not optional). The 70% threshold is
the RFC-0006 §8 Q1 design criterion; it can be revised by a recorded decision (append-only).

**Confirmation threshold.** Retention ratio ≥ 70% *at* the falsification line confirms that the
committed text surface (DN-09 §3.1) retains most leverage and that M-380 projections remain a
co-equal lift rather than a mandatory fallback.

**What would falsify the working hypothesis.** Novel surface retention < ~70% of familiar-skin
pass@1 on composition tasks, across ≥3 seeds and ≥1 frontier model.

**What would confirm it.** Retention ratio ≥ ~70%, with grammar-in-context and/or constrained
decoding conditions showing further lift — consistent with the MTOB and grammar-prompting evidence
(RFC-0006 §8 Q1; T3.6).

**Feeds:** DN-09 §4 (honest scope statement); M-002 (#3 follow-up variant pass); M-380
(projection layer design — confirmed: co-equal lift; falsified: mandatory substitution); RFC-0021
(M-380 semantic-projection RFC, when authored).

---

## RP-2 — Stage-1 Grading: Implicit-Flows Decision and Noninterference Proof

**Status:** **RESOLVED (2026-06-18)** — research in `research/09-stage-1-grading-noninterference-RECORD.md`;
decision recorded in **RFC-0018 (Accepted)**: R18-Q1 = **Design A** (data-provenance integrity),
R18-Q4 = certificate reference at the type level / validity at elaboration-runtime; R7-Q2 closed.
RFC-0018 supersedes RFC-0007 §4.3's stage-1 deferral and discharges RFC-0006 §8 Q3. The soundness
result stays **Declared-with-argument** (not machine-checked — a future `Proven` upgrade needs
mechanization). (Original prompt retained below.)

**Question.** For the stage-1 static guarantee-grading RFC (a revision of RFC-0007 §4.3 / RFC-0006
Q3): do *implicit flows* taint guarantee grades, or do guarantee grades track data lineage only?

- **Implicit-flows taint (pc-index approach):** branching on a `Declared` (or weaker) scrutinee
  degrades the guarantee of *all* values bound in the branches, requiring a `pc`-like grade index
  (DCC-style, RFC-0006 §8 Q3). The grading becomes a graded coeffect modality over the guarantee
  meet-semilattice with a `pc`-index: `Γ; pc ⊢ e : τ @ g`.
- **Data-lineage only:** guarantee grades propagate only through data flow (operand inputs → result
  via `meet`); branching on a `Declared` scrutinee does not degrade the branches' result grades
  (unless those branches compute from the scrutinee's fields). Simpler static analysis; no `pc`
  index; but potentially unsound for integrity (a `Declared` condition used to choose an otherwise-
  `Exact` result would not show up as `Declared` in the output).

**Falsification / confirmation conditions.**

- The implicit-flows variant is *confirmed necessary* if a concrete counterexample program can be
  constructed in which data-lineage-only grading produces an `Exact`-tagged result for a
  computation whose value was chosen by a `Declared`-grade condition — i.e., where a user could
  observe a `Declared`-influenced outcome with an `Exact` tag. The DCC/Biba-dual integrity reading
  (RFC-0006 §8 Q3) already suggests this is likely; a formal proof that data-lineage-only fails
  noninterference (in the integrity direction: "no `Declared` input influences an `Exact`-typed
  output except through a certified `Swap`") would confirm it.
- Data-lineage-only is *sufficient* if a formal noninterference statement can be proved over the
  4-point integrity lattice with the runtime-certificate endorsement mechanism — i.e., the only
  way `Declared` reaches an `Exact`-typed output is through an explicit `Swap` with a certificate
  that endorses the grade upgrade.

**Proof obligation (flagged novel — RFC-0006 §8 Q3).** Grading + runtime certificates has no found
precedent; the soundness argument must be constructed (not merely cited). The noninterference proof
over the 4-point integrity lattice with runtime-certificate endorsement is explicitly flagged as
needing its own argument, by analogy with DCC noninterference (Abadi et al.) but in the integrity
direction.

**What a spike should produce.** (a) The counterexample program (or proof of its non-existence) for
the data-lineage-only variant. (b) A draft noninterference statement and sketch proof for the
chosen grading design. (c) The resulting grading RFC (a revision of RFC-0007 §4.3) with the
implicit-flows decision recorded append-only.

**Feeds:** RFC-0018 (stage-1 grading RFC, when authored); RFC-0006 §8 Q3; RFC-0007 §8 R7-Q2
(the `Match` default arm's interaction with guarantee indices, a sub-case of the implicit-flows
question — does a `default` arm's body meet-degrade differently from named alternatives?).

---

## RP-3 — Traits Coherence and Repr-Polymorphism Soundness (LR-5 / T3.3)

**Status:** **RESOLVED (2026-06-18)** — research in `research/10-traits-coherence-repr-polymorphism-RECORD.md`;
decisions recorded in **RFC-0019 (Accepted)**: coherence = orphan rule + global uniqueness +
reject-overlap; the Repr-polymorphism restriction set ("no paradigm-specific `Op` on a Repr-abstract
argument" — locally checkable, S1-preserving) is normative; newtype waivers rejected in v1 (need
roles); multi-param/associated types deferred to v2. Soundness stays **Declared-with-argument** (not
machine-checked). (Original prompt retained below.)

**Question.** What coherence mechanism and restriction set make **Repr-polymorphism** (LR-5:
abstracting over the paradigm `∀r: Repr`) sound while keeping `Swap` explicit (S1)?

**Sub-questions.**

1. *Coherence mechanism:* which of the standard approaches applies here — Haskell-style global
   coherence (one instance per type, globally), Rust-style orphan rules, or Scala-style local
   implicits? LR-5 imposes a constraint beyond ordinary typeclass coherence: a generic that
   abstracts over `Repr` must not be able to introduce a `Swap` silently (S1). The restriction
   set must guarantee that.
2. *Restriction set:* what restrictions on `Repr`-polymorphic definitions preserve S1? A candidate
   restriction is that a `Repr`-polymorphic function may not perform `Op` on its `Repr`-abstract
   arguments (since `Op` is paradigm-specific — RFC-0001 §4.5 *(Prim)* rule); it may only pass
   them through or `Swap` them explicitly. Does this restriction suffice, and is it checkable
   locally (without whole-program analysis)?
3. *Paradigm-polymorphism soundness:* is there a formal statement that a `Repr`-polymorphic
   function + all its instantiations satisfy S1 (no implicit swap inserted by instantiation)?

**Falsification / confirmation conditions.** A candidate restriction set is *sufficient* if: for
any `Repr`-polymorphic definition satisfying the restrictions, every monomorphic instantiation
passes S1 locally (no elaboration step inserts a `Swap`). It is *insufficient* if a
counterexample instantiation can be exhibited that violates S1 under the restrictions. A
*confirmation* is a proof (or proof sketch grounded in T3.3 prior art) that the restriction set
is both sound (S1-preserving) and complete enough to write useful generic libraries.

**Feeds:** RFC-0019 (traits / Repr-polymorphism RFC, when authored); RFC-0006 §4.2 LR-5 (the
"beyond Rust" core capability); T3.3 (the research pass findings on Repr-polymorphism).

---

## RP-4 — Semantic-Projection Ergonomics (G11 / FR-C1)

**Status:** **Partially addressed (2026-06-18) — `research/11-semantic-projection-framework-RECORD.md`.**
Sub-question 3 (dual rendering) is **answered** — one architecture suffices (Unison/MPS precedent);
sub-question 1 (human usability) is addressed as a **design recommendation** (the Unison posture:
edit-in-text, projections read-mostly + opt-in round-trip), with authoring assessed **feasible** at
single-engineer scale (grounded in the existing node-walk; a *measured* cost study stays open).
**Sub-question 2 (does a canonical projection raise LLM leverage?) is irreducibly empirical and stays
OPEN** (see RP-1's protocol). No leverage asserted (VR-5). (M-380 is design-active since DN-09.)

**Question.** Is **semantic projection** (not merely notational projection) usable and desirable
for human authors, and does an **LLM-facing canonical projection** raise leverage above the bare
text surface?

**Sub-questions.**

1. *Human usability:* a semantic projection renders the same content-addressed definition in a
   different notation. Does a human author editing a projected view feel that it faithfully
   represents the underlying program, or does the mismatch between the written projection and the
   canonical IR create friction (the "projectional-editor usability friction" risk, Foundation
   §6)?
2. *LLM leverage:* does an LLM-facing canonical projection (a rendering of L1/L2 definitions in
   a form optimized for machine co-authoring — e.g. maximally regular, minimal sugar, explicit
   types) produce higher pass@1 or retention ratio than the bare text surface? This is the
   "grammar-in-context + constrained decoding" lift measured in RP-1 as a condition, but here the
   question is specifically about a *canonical projection* as the LLM's input surface rather than
   a primer alongside the native surface.
3. *Dual rendering:* the FR-S5 dual-intelligibility requirement is that the same content-addressed
   definitions render for human and machine co-authors. Is there a single projection architecture
   (MPS/Unison-style) that satisfies both, or do human and machine projections require different
   design choices?

**Falsification / confirmation conditions.** The LLM-leverage claim is *falsified* if the
canonical projection produces no statistically significant lift over the grammar-in-context primer
condition (RP-1 condition 2) — i.e., the projection architecture adds no LLM leverage beyond what
a good primer already provides. It is *confirmed* if the canonical projection produces measurably
higher pass@1 or retention ratio on composition tasks, with ≥3 seeds.

**Feeds:** RFC-0021 (M-380 projection RFC, when authored); FR-C1 (exploratory, Phase 3);
RFC-0006 §9 (future: "LLM-facing canonical projection where the same content-addressed definitions
render differently for human and machine co-authors"); DN-09 §3.1 (M-380 design-active);
G11 (the dual human/machine intelligibility channel).

---

## RP-5 — Maturation Enactment Spikes (RFC-0017 R17-Q1/Q2/Q3)

**Status:** Open (RFC-0017 is Accepted; R17-Q1/Q2/Q3 are explicitly deferred to the enacting build
tasks — `mycelium-build` / `mycelium-proj`).

**Question.** Three enactment sub-questions left open by RFC-0017 §9:

**R17-Q1 — Manifest target-set spelling.** What is the exact `mycelium-proj.toml` spelling for
build-target maturation: `[project].matured = [...]`, `[build].matured = [...]`, or does a build
*profile* (RFC-0004 §4 target set) subsume the maturation declaration? The normative content
(maturation = a resolved inspectable scope attribute) is fixed in RFC-0017 §4.1; only the concrete
TOML key path is open.

*Confirmation:* a prototype `mycelium-proj.toml` with both options, validated against the M-311
content-addressed maturation certificate. Record the chosen spelling as an append-only note or
RFC-0017 amendment.

**R17-Q2 — `thaw` beyond `fn`.** When methods (`impl`), data declarations, or other definition
forms enter the grammar (DN-03 §1), does `thaw` prefix them uniformly? The expectation (RFC-0017
§9) is yes (additive, no new mechanism), but this must be confirmed when those grammar forms land.

*Confirmation:* at least one non-`fn` definition form (e.g. `thaw impl Foo::bar`) parses, passes
the maturation gate check, and is correctly excluded from the AOT compilation scope of the
enclosing matured `nodule`. No new mechanism required; confirmation is a grammar + gate test.

**R17-Q3 — Cross-scope inlining attribution.** A matured scope may inline a `total` definition
from an *unmatured* scope (RFC-0017 §4.2 — the inlined callee must itself pass the per-definition
gate). How is this attributed in the maturation record: does the matured scope's content-addressed
maturation certificate include the hash of every inlined callee, or only the hashes of the
directly-matured definitions?

*Confirmation:* a prototype maturation record for a `nodule` that inlines a callee from an
unmatured `nodule`, showing that the record is complete and EXPLAIN-able (no black boxes, G2 /
SC-3) and that the attribution is stable under callee rename (names-as-metadata, ADR-003).

**Feeds:** RFC-0017 §9 (R17-Q1/Q2/Q3); M-311 (the maturation-certificate build task);
`mycelium-build` crate; `mycelium-proj.toml` schema.

---

## RP-6 — R7-Q3 Surface Grammar for Mutual Recursion

**Status:** **Resolved** (2026-06-19 — candidate 2: nodule-wide mutual visibility, no new syntax;
recorded in **DN-13**, confirmed by **M-391**). The elaboration *mechanics* were already settled by
RFC-0001 r5 / M-343; the *surface grammar* residual is now decided — see DN-13.

**Question.** What is the surface grammar for mutually-recursive function definitions in
Mycelium's v0 first-order syntax? Three candidate designs:

1. **Explicit grouping** (`let rec f = … and g = …` — ML-style): the programmer marks the group
   explicitly; the elaborator feeds it to the existing Tarjan path.
2. **Nodule-boundary fixpoint** (Unison/ML module semantics): every top-level function in a
   `nodule` is mutually visible; the elaborator runs Tarjan over the full `nodule` call graph
   automatically; no syntax change required.
3. **Explicit `mutually_recursive { … }` block**: an explicit grouping form in the surface grammar,
   closer to Idris `mutual`.

*Confirmation* (of any candidate): the chosen form parses, feeds the existing Tarjan SCC path in
`mycelium-l1::elab`, produces the same `FixGroup` node as the programmatic path, and passes the
M-210 three-way differential (L1-eval ≡ elaborate→L0-interp) for ≥2 surface-written mutual-
recursion programs. The confirmed form is recorded append-only (a DN or RFC-0007 amendment).

*Falsification* (of a candidate): a concrete program in which the candidate grammar produces an
incorrect or ambiguous elaboration, or a diagnostic story that is worse than an alternative.

**Feeds:** DN-10 §2.6 (this note's spike prompt); RFC-0007 §8 R7-Q3 (surface half);
`docs/spec/grammar/mycelium.ebnf`; DN-09 §3.1 (surface commitments are append-only).

---

## RP-7 — R7-Q4 Prim BoundBasis Schema

**Status:** Open (RFC-0007 §8 R7-Q4 records the direction; DN-10 §3.6 is the spike prompt).

**Question.** How is a primitive's *intrinsic guarantee* specified in a content-addressed prim
declaration (the R7-Q4 migration from the fixed `Π` table to declarations)? Specifically: is the
`BoundBasis` field (RFC-0001 §4.3) stored *with* the prim declaration (making the
`Proven`/`Empirical` distinction content-addressable and auditable), or stored separately (e.g.
in a companion annotation)?

*Confirmation:* a prototype prim declaration schema that includes the `BoundBasis` field alongside
the type signature; validation that the guarantee meet (RFC-0001 §4.7) over prim calls produces
the same tags as the current hard-coded intrinsic guarantees; green `cargo test` after the registry
switch. The chosen schema is recorded as a RFC-0001 revision (r6 or later) or a companion ADR.

*Falsification:* a prim (e.g. `bundle`) for which the `BoundBasis` cannot be stored at
declaration-authoring time — only at call-site instantiation time — would falsify the "inline
BoundBasis" approach and require a late-binding or call-site annotation design instead.

**Feeds:** DN-10 §3.6 (this note's spike prompt); RFC-0007 §8 R7-Q4; RFC-0001 §4.3 (`BoundBasis`
schema); ADR-003 (content-addressed identity); KC-3 (small auditable kernel).

---

## RP-8 — Performance Optimization & Tool-Extraction Spike

**Status:** Open (maintainer direction, 2026-06-21). Near-future spike — **post-1.0 / parallel to
the 1.0.0 gate work**, not a 1.0.0 blocker (the 1.0.0 gate is honesty-integrity + durability, not
performance — ADR-021 §5; DN-19).

**Direction (maintainer).** The long-term aim is performance that is **fast, robust, reliable, and
easy to use across the full spectrum** — low-level systems code to high-level application code and
everything between — *while keeping the honesty guarantees intact* (never-silent swaps, the
per-op guarantee lattice, no black boxes). This is the language's north star; this prompt turns it
into a tracked, falsifiable optimization track rather than an aspiration.

**Question.** Where is the performance actually spent today, and which hot paths can be optimized
or **extracted into dedicated tools / native paths** without weakening any guarantee? Concretely:
1. **Profile the current Rust workspace** under representative workloads (the interpreter trusted
   base; `mycelium-cert` swap + certificate-check; `mycelium-vsa`/`-dense` ops; the MLIR→LLVM AOT
   path) and produce an honest hot-path inventory (Empirical — measured, this-machine, like KC-4).
2. **Cert-check overhead toward the nanosecond range** (the ADR-021 A5 / phase-2.md §6.7 long-term
   target — currently ~µs): can the translation-validation check be specialized/precompiled per
   swap class, memoized on content hash (ADR-003), or shifted off the hot path on the AOT/stable
   path (ADR-009) so the *interpreter* keeps full per-swap checking while *stable* code pays
   near-zero? Without ever making a swap silent (G2/VR-5).
3. **Extraction into tools.** Which perf-critical kernels (packing, VSA bind/bundle, dense numerics,
   the certificate checker) warrant extraction into a dedicated optimized crate / native codegen
   path / SIMD or BitNet-packed kernel (FR-C3) — and what's the honest guarantee-preserving boundary
   for each?
4. **Scope coverage.** What does "works across low→high level" require that isn't built yet
   (the intended-scopes gap), sequenced against the Phase-6 native path (libMLIR, M-348/M-601)?

*Confirmation:* a committed, reproducible profiling harness (extending the `xtask kc4` pattern) +
an Empirical hot-path report; at least one optimization landed with a before/after measurement and
a regression test proving the guarantee it touches is unchanged; the cert-check budget tracked
against the nanosecond target. Each optimization is honest (measured, this-machine) and
guarantee-preserving.

*Falsification:* a hot path whose only viable speedup *weakens* a guarantee (makes a swap silent,
drops a bound's basis, or removes EXPLAIN-ability) falsifies "optimize without weakening guarantees"
for that path — it must then be recorded as an explicit honesty/performance tension (a DN), not
silently taken.

**Feeds:** ADR-021 §5 (out-of-scope-for-1.0.0 maturation) + A5 (the nanosecond cert-overhead
target); `docs/planning/phase-2.md` §6.7 (KC-4 measurement + budget); Phase-6 native path
(M-348/M-601, libMLIR); FR-C3 (BitNet packed-ternary acceleration); ADR-009 (AOT vs interpreter);
ADR-003 (content-addressed memoization); the honesty rule / G2 / VR-5 (the non-negotiable
constraint on every optimization). NON-BLOCKING for 1.0.0.

---

## RP-9 — Agent Development Kit Phylum: discharge RFC-0023 gates

**Status:** **Research gate SUBSTANTIALLY DISCHARGED (2026-06-21, `dfr` session).** Four fractured Opus sub-reasoners (A1 concept-map · A2 honesty-differentiator · A3 tool-dispatch never-silent · A4 session/runner + harness reuse) verified the obligations against primary ADK source (pinned `adk-python` **v2.3.0** / "ADK 2.0") + landed in-repo substrate — **design soundness confirmed, no soundness falsification** (synthesis: `research/13-adk-phylum-RECORD.md` §6). RFC-0023 staged **"pending maintainer ratification"**; the discharge + the **M-671** body update unblock the `dfb` build (full ratification additionally needs the build + E7-1/E7-2). **NOT fully closed** — one OPEN completeness item (the ADK-2.0 concept-map repair: **graph/Workflow** + agent-mode/code-router/`RunConfig`-budget rows), plus deferred-to-build empirical-on-code, a `ToolError` budget-arm, the runtime-realization choice, and the LLM-leverage question (item 6, scoped, **no verdict** — inherits DN-09 INDETERMINATE; never pre-written, VR-5). *(Originally: Open 2026-06-21; gated M-671.)* **→ RATIFIED 2026-06-21 (maintainer): RFC-0023 → Accepted.** The one OPEN completeness item is now **CLOSED** — §3.7 maps ADK 2.0's graph Workflow Runtime + operating-`mode` + code-Router + `RunConfig`-budget; §3 pinned v2.3.0. Decisions ratified: runtime `mycelium-mlir::runtime` (R23-Q1); `ToolError` budget→`TaskOutcome::BudgetExhausted` (branded names canonical); Session snapshot-v0, merge deferred to `fuse` (R23-Q2); LLM-leverage no-verdict. Accepted=design; Enacted gated on the `mycelium-adk` build + E7-1/E7-2.

**Direction.** This is the **follow-up deep-research pass** that RFC-0023 requires before ratification. The Phase-1 design pass produced the RFC and `research/13-adk-phylum-RECORD.md` (the fractured-methodology pass: four Opus sub-reasoners over one cross-context packet; findings are Empirical/Declared, never Proven). This RP-9 pass must discharge the open items in RFC-0023's Honest-Uncertainty Register. All findings remain **Empirical/Declared** — never `Proven` (VR-5).

**Question(s).** Four verification obligations gating RFC-0023 ratification:

1. **ADK→Mycelium concept-map correctness** — verify that RFC-0023 §3's mapping of ADK's abstractions (Agent, Tool, Session/State/Memory, Runner, multi-agent composition, model layer) to Mycelium constructs (`colony`/`hypha`, typed `fn -> Result`, content-addressed value state, `graft` capability handles, `GrokLlmReport`) is sound: no ADK concept is silently dropped; every mapping is grounded in the source ADK APIs and the corresponding Mycelium corpus file:line. The `Declared` tag on the concept-map is appropriately conservative — upgrade to `Empirical` only if an implementation demonstrates the mapping in running code.

2. **Honesty-as-differentiator claim** — the RFC-0023 §6 claim that Mycelium's honesty contract (every LLM/tool output tagged Declared/Empirical, never Proven; every tool call a never-silent `Result`) constitutes a genuine differentiator over raw ADK is **Declared** today. This pass must produce a grounded, falsifiable argument: what ADK agent workflow would currently misrepresent a Declared output as Proven, and how does the Mycelium surface prevent it? The claim stays `Declared` (asserted, flagged) unless a concrete running example confirms the never-silent contract holds end-to-end.

3. **Tool-dispatch never-silent contract** — the `adk.tool` nodule's typed `fn -> Result<Out, ToolError>` dispatch model claims never-silent fallibility (every tool call that fails returns an explicit `ToolError`, never a silent default or panic). Verify that the Rust-first `mycelium-adk` implementation honors this for the documented tool-dispatch surface (at minimum: argument-type mismatch → `ToolError::TypeMismatch`, execution failure → `ToolError::Exec`, timeout → `ToolError::Budget`). The verification is `Empirical` (passes the test corpus); it is not `Proven`.

4. **Session/runner model and LLM-harness reuse** — confirm that the `adk.session` (content-addressed value state, ADR-003) and `adk.runner` (the orchestration surface) designs correctly reuse the existing LLM harness (`GrokLlmReport` / `tools/llm-harness/`) without introducing a second model-call path. Any second path would violate DRY (KC-3) and create an unaudited surface. Also confirm the `adk.model` nodule's model-transport dependency on the `web` phylum's HTTP/JSON surface (M-670) is correctly scoped as a dependency, not a duplicate.

**Confirmation thresholds.** Each sub-question is confirmed when: (1) every ADK concept maps to a named Mycelium construct with a corpus citation; (2) at least one concrete example shows a Declared-tagged output that would be un-tagged in raw ADK; (3) the tool-dispatch test suite covers the documented `ToolError` variants with zero silent failures; (4) the `mycelium-adk` crate imports `GrokLlmReport` from the existing harness with no competing model-call path.

**Falsification thresholds.** (1) A dropped ADK concept with no Mycelium equivalent falsifies the completeness of the concept-map; (2) an ADK workflow where the honesty-as-differentiator claim adds no observable change to the output tagging falsifies the differentiator case; (3) a tool-dispatch path that returns a default/silent result on failure falsifies the never-silent contract; (4) a second model-call path in `mycelium-adk` falsifies the LLM-harness reuse claim.

**Feeds:** RFC-0023 (the ADK phylum design + Honest-Uncertainty Register); `research/13-adk-phylum-RECORD.md` (the Phase-1 evidence base); M-671 (the build task blocked on this pass); RFC-0016 §4.1 C1–C6 (per-op contract); RFC-0008 RT1–RT7 (colony/hypha model); ADR-003 (content-addressed session state); G2 (never-silent); VR-5 (honest tags); KC-3 (no duplicate trusted-code surfaces).
## RP-10 — Web-Tooling Phylum: discharge RFC-0022 gates

**Status:** **Research gate DISCHARGED (2026-06-21, `dfr` session).** Four fractured Opus sub-reasoners (W1 HTTP/never-silent · W2 JSON-codec-reuse · W3 server-determinism · W4 routing/EXPLAIN) verified all four obligations against primary specs (RFC 9110/9112, RFC 8259, WHATWG-URL, RFC 9110 §12) + landed source — **all design-sound, no falsification** (synthesis: `research/12-web-phylum-RECORD.md` §8). RFC-0022 staged **"pending maintainer ratification"**; the discharge + the **M-670** body update unblock the `dfb` build. Residuals named, not closed: **deferred-to-build** empirical-on-code (the ≥100-vector corpus, the RT2 differential, per-dispatch `RouteMatch`) + **scoped-future** (HTTP/2-3, TLS, the cross-peer smuggling threat model, wall-clock time, async-runtime choice, RT3, the versioned IDNA pin). *(Originally: Open 2026-06-21; gated M-670.)* **→ RATIFIED 2026-06-21 (maintainer): RFC-0022 → Accepted.** Decisions ratified: IDNA policy (pin-at-build, nontransitional, fail-closed); `web.server` on the Mycelium runtime (realization `mycelium-mlir::runtime`); HTTP/2-3 + TLS + smuggling = v1 non-goals. Accepted=design; Enacted gated on the `mycelium-web` build.

**Direction.** This is the **follow-up deep-research pass** that RFC-0022 §10 (the Honest-Uncertainty Register) demands before ratification. The Phase-1 design pass produced the RFC and `research/12-web-phylum-RECORD.md` (T12.1.x–T12.4.x); this RP-10 pass must close the open items in the §10 register and that record's §6. All findings are **Empirical/Declared** (measured or asserted, grounded in evidence) — never `Proven` (no machine-checked theorem underpins any of these; VR-5).

**Question(s).** Four concrete verification obligations from RFC-0022 §10 / `research/12-web-phylum-RECORD.md` §6:

1. **HTTP/1.1 parsing edge cases + the never-silent located-error contract** — verify the `web.http` never-silent parser (every malformed input produces a located `ParseError` naming the field and byte offset, never a sentinel or partial result) against a real HTTP conformance corpus. The `Empirical` tag on the parser contract needs measured coverage across edge cases (folded headers, obsolete line folding, chunked encoding edge cases, request-target forms per RFC 7230). The tag must not be upgraded to `Proven` without a mechanized conformance proof.

2. **JSON↔Value surface delegating to `std.io`'s one canonical codec (no new codec)** — confirm that `web.json` introduces zero new trusted serialization code: the `to_json` / `from_json` surface is a thin ergonomic layer (convenience types + routing glue) exclusively delegating to `std.io`'s one canonical JSON projection over `Value` (RFC-0001 §4.8, M-514). Any second codec path would violate DRY (KC-3) and re-introduce trusted-code surface `std.io` deliberately avoids.

3. **Server-as-`colony`-of-`hyphae` determinism claim (Empirical-via-RT2 differential, NOT Proven)** — the RFC-0022 §4.5 claim that the `web.server` surface (a `colony` of one request-handling `hypha` per connection) satisfies the RFC-0008 RT2 determinism invariant must be verified via a differential test (matching interpreter-path vs AOT-path over a fixed request corpus), not declared. The determinism argument is **Empirical** (passes the RT2 differential on the test corpus); it is not `Proven` (no machine-checked concurrency proof exists — VR-5).

4. **Route-table inspectability / EXPLAIN** — confirm that `web.route`'s route-dispatch is EXPLAIN-able: every dispatch decision can be materialized as an inspectable `RouteMatch` record (the selected handler + the matched pattern + the priority ordering that resolved ambiguity), satisfying RFC-0016 §4.1 C3 (no black-box routing). Verify that ambiguous routes (two patterns matching the same path) produce an explicit `CheckError` at route-table construction time, never at dispatch time.

**Confirmation thresholds.** Each sub-question is confirmed when: (1) a measured edge-case corpus (≥100 HTTP/1.1 test vectors) passes the located-error contract with zero silent accepts; (2) the `web.json` crate has zero serialization code outside its delegation to `std.io`; (3) the RT2 differential test suite covers the server surface with ≥3 seeds and zero divergences; (4) the route-table EXPLAIN path materializes a `RouteMatch` for every dispatch in the test corpus, and the ambiguity-at-construction-time check fires on a hand-crafted ambiguous table.

**Falsification thresholds.** (1) A conformance-corpus HTTP vector accepted silently (no error returned) falsifies the never-silent contract; (2) any serialization path in `mycelium-web` that does not delegate to `std.io` falsifies the DRY claim; (3) a server test case where the `colony`/`hypha` path produces a different observable output from the sequential-interpreter path falsifies the `Empirical` determinism claim and requires a design change; (4) a route dispatch that selects a handler without materializing an EXPLAIN record falsifies the inspectability claim.

**Feeds:** RFC-0022 §10 (Honest-Uncertainty Register); `research/12-web-phylum-RECORD.md` §6; M-670 (the build task blocked on this pass); RFC-0016 §4.1 C1–C6 (the per-op contract); RFC-0008 RT2 (determinism differential); ADR-003 (content-addressed identity); G2 (never-silent); VR-5 (honest tags).

---

## Resolved Prompts

- **RP-6 — R7-Q3 Surface Grammar for Mutual Recursion.** **Resolved 2026-06-19.** Verdict: **candidate
  2 — nodule-wide mutual visibility, no new syntax** (every top-level `fn` in a `nodule` is mutually
  visible; the elaborator auto-groups each call-graph SCC of ≥2 into a `FixGroup`). Recorded in
  **DN-13** (`docs/notes/DN-13-RP-6-Surface-Grammar-Mutual-Recursion.md`); confirmed by **M-391**
  (M-210 differential + identity + never-silent).

*(Other prompts move here when their feeding decision is recorded and the prompt is closed. Mark:
append the resolution date, verdict, and pointer to the decision doc. Do not delete.)*
