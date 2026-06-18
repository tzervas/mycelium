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

**Status:** Open (the M-002 run cleared the KC-2 kill criterion but did not run this ablation).

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

**Status:** **Research discharged (2026-06-18)** — `research/09-stage-1-grading-noninterference-RECORD.md`
delivers the counterexample, the data-provenance noninterference theorem + proof sketch, R7-Q2
closure, and the Design-A / R18-Q4 recommendations (RFC-0018 §11 marked discharged). The soundness
result is tagged **Declared-with-argument** (not machine-checked — a future `Proven` upgrade needs
mechanization). The *research* gate is closed; RFC-0018 ratification now awaits only the maintainer's
append-only R18-Q1 (Design A vs B) and R18-Q4 decisions. (Original prompt retained below.)

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

**Status:** Open (RFC-0006 §4.2 LR-5 is a committed direction; the coherence mechanism and
restriction set are not yet decided).

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

**Status:** Open (M-380 is design-active since DN-09; the ergonomics question is the design
constraint for the projection RFC).

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

**Status:** Open (the elaboration *mechanics* are settled by RFC-0001 r5; the *surface grammar
question* is the residual — see DN-10 §2.6).

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

## Resolved Prompts

*(None yet — entries move here when their feeding decision is recorded and the prompt is closed.
Mark: append the resolution date, verdict, and pointer to the decision doc. Do not delete.)*
