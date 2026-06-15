# RFC-0006 — Surface Language, Grammar & Term-Language Layering

| Field | Value |
|---|---|
| **RFC** | 0006 |
| **Status** | **Accepted** (r4 — ratified 2026-06-15 per §10: layering §3, invariants S1–S6, capability targets LR-1…LR-9, grammar discipline §4.3, and §8 positions Q2/Q4/Q5/Q7. **Concrete L3 syntax stays KC-2-gated** (Q1/Q6); stage-1 static grading (Q3 incl. implicit-flows) stays open for the grading RFC; the `unsafe`-class L3 spelling (Q8) is DN/KC-2-gated. Supersedes the r3 Draft.) |
| **Type** | Foundational / normative (once Accepted) |
| **Date** | June 10, 2026 |
| **Depends on** | RFC-0001 (Core IR §4.5, WF1–WF5, content-addressing §4.6); RFC-0002 (never-silent swaps); RFC-0005 (reified selection); ADR-003 (Unison-style identity); ADR-006 (no black boxes); ADR-007 (interpreter = trusted base); KC-2/KC-3; RR-3; SPEC §10.1/§10.2 |
| **Coupled with** | `experiments/surface-fragment/` (M-020, throwaway KC-2 fixture); the KC-2 experiment (M-002, #3) |

## 1. Summary

Mycelium needs a full programming language above the Core IR — today the committed term language
is only `Const | Var | Let | Op | Swap` (RFC-0001 §4.5), and SPEC §10.2 explicitly defers
"abstraction, recursion, modules" to a later RFC. **This is that RFC.** It nails down, *now*,
the parts that must not drift while implementation proceeds:

1. the **layering** of the language (what elaborates to what, and which layer is trusted);
2. the **invariants every layer must preserve** (the honesty rules, made syntactic);
3. the **capability target** — "Rust-class and beyond", stated as checkable requirements;
4. the **grammar/spec discipline** (formalism, machine-readable artifacts, conformance corpus);

and it **defers exactly one thing**: the concrete human-facing syntax, which the corpus already
gates on the KC-2 LLM-leverage experiment (Foundation §2.2/§2.4; RR-3 — if novel syntax hurts LLM
leverage, the surface becomes projections or an embedded DSL instead). Committing a syntax before
that verdict would overturn an existing decision without evidence.

## 2. Motivation

The risk this RFC removes is *architectural lock-in by accident*: as Phase-2/3 code lands, ad-hoc
surface decisions (in tests, tools, the LSP, examples) would otherwise accrete into a de-facto
language that was never designed. Conversely, the *capability ceiling* must be declared early: a
substrate that wants Rust-class general programming cannot retrofit polymorphism, data types, or
modules onto a five-node IR without planning the elaboration path. Both failure modes are cheaper
to prevent than to undo (append-only discipline applies to decisions, and should not have to apply
to regrets).

## 3. Guide-level explanation — the layer cake

Four layers, each with its own grammar and spec artifact; **only L0 is trusted** (KC-3, ADR-007).

```
 L3  Projections / editor surface      ← KC-2-gated: text syntax, or projections/embedded DSL (RR-3)
 L2  Surface term language ("Myc")     ← the Rust-class language: ADTs, traits, modules, recursion
 L1  Kernel calculus                   ← small typed core: λ + data + explicit recursion + Repr types
 L0  Core IR (RFC-0001 §4.5, frozen)   ← Const | Var | Let | Op | Swap  + Meta/WF1–WF5
```

- **L0 — Core IR.** Already committed and frozen (changes need their own RFC). It stays the
  *semantic* ground truth: the reference interpreter (M-110) executes L0, certificates and
  differentials (E2-3) speak about L0 values.
- **L1 — kernel calculus.** A deliberately small, *fully explicit*, typed calculus: lambda
  abstraction/application, algebraic data + `match`, explicit (checked) recursion, and the
  paradigm types (`Repr` in the types, RFC-0001 §3.3). L1 **elaborates** to L0 plus a bounded set
  of new Core nodes (at minimum `Lam/App/Match/Fix` — each addition is an RFC-0001 revision with
  its own well-formedness rules). L1 is the layer the type checker, the content-addresser
  (ADR-003), and the formal semantics are defined against.
- **L2 — surface term language.** The programmer-facing language: type inference, traits/
  typeclasses, modules, pattern sugar, derived forms. L2 is defined **entirely by elaboration to
  L1** — every L2 construct has a specified desugaring; there is no L2 semantics independent of
  it (no black boxes, ADR-006: you can always ask "what did this elaborate to?", and the LSP's
  stage-dump channel — SC-5/M-140 — must show it).
- **L3 — projection layer.** Whatever KC-2 says humans-and-LLMs should *write*: a concrete text
  syntax, structured projections of L1/L2 (Unison-style, consistent with ADR-003 identity), or an
  embedded DSL in a host language (RR-3's fallback). The M-020 fragment is an L3 *experiment
  fixture*, not a commitment.

## 4. Reference-level design (normative once Accepted)

### 4.1 Invariants every layer must preserve (the honesty rules, made syntactic)

These bind L1–L3 regardless of which syntax wins; they are restatements of committed corpus rules
at the language level, so violating them in a syntax proposal is a rejection criterion:

- **S1 (never-silent swap).** A representation change is *lexically visible* at every layer
  (WF1/WF2; SC-3). No sugar, inference step, or trait resolution may insert a `Swap`; elaboration
  is **Repr-transparent**. A surface form that auto-converts is ill-formed by definition.
- **S2 (honest tags surface).** The guarantee lattice (`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`)
  is part of the *observable interface* of every binding: tooling must be able to annotate any
  expression with its tag/bound (the M-140 channel), and a `Declared` value is always visibly
  flagged (M-I4).
- **S3 (content-addressed identity).** Definition identity is the α-normalized structure hash
  (ADR-003, RFC-0001 §4.6) at every layer — names are bindings to hashes, never identity. This is
  what makes modules/refactors safe and the EXPLAIN/`policy_used` story ("which policy chose
  this?") uniform across the language.
- **S4 (inspectable elaboration).** Every L2→L1→L0 step is dumpable and diffable (the RFC-0004 §6
  no-opaque-lowering rule, extended upward to the front-end). The elaborator is not a black box.
- **S5 (explicit partiality).** Anything that can fail — out-of-range, illegal pair, unsupported
  composition — is an explicit `Option`/`Result`/diagnostic in the language, mirroring the kernel
  (G2). No surface construct may erase a kernel refusal.
- **S6 (self-sufficiency / AI-independence) — foundational.** Mycelium is a *programming
  language for software engineering*, complete and runnable with **no AI/LLM in the loop**: the
  parser, type checker, elaborator, interpreter, and AOT path are ordinary deterministic
  software. An AI model is a *co-authoring convenience* (the FR-S5 dual-intelligibility surface,
  the KC-2 question), **never a runtime, compile-time, or semantic dependency** — nothing in the
  language's meaning, execution, or toolchain may require model inference to resolve. A program's
  behavior is defined by the L0/L1 semantics alone; remove every model and the language still
  builds, checks, runs, and reproduces bit-for-bit. (AI tooling *may* call out to providers as an
  optional editor feature — that is integration, not dependence; it sits strictly above S4's
  inspectable elaboration and may not influence it.) This bounds KC-2: the experiment measures
  how *well* models co-author Mycelium, and can only ever choose the L3 surface — it can never
  make the language *need* a model.

### 4.2 Capability target — "Rust-class and beyond", as requirements

Stated as language requirements **LR-1…LR-9** so the eventual L1/L2 designs are checkable against
them rather than against a vibe:

| Id | Requirement | Posture |
|---|---|---|
| **LR-1** | Algebraic data types + exhaustive `match` (no fall-through silently) | committed direction |
| **LR-2** | Parametric polymorphism with bounded abstraction (traits/typeclasses; coherent, no overlapping-instance ambiguity) | committed direction |
| **LR-3** | Modules with content-addressed identity (ADR-003); separate compilation falls out of hashing, not file layout | committed direction |
| **LR-4** | General recursion, **with a declared totality posture**: the trusted interpreter keeps its fuel guard; a checked-total fragment is what "stable components" (RFC-0004 §4) promote from | committed direction; mechanism researched (T3.4) |
| **LR-5** | **Repr polymorphism**: abstracting over the paradigm (`∀r: Repr-of-kind-K`) without violating S1 — swap insertion stays explicit even in generic code | committed direction (the "beyond Rust" core); restriction set researched (T3.3) |
| **LR-6** | **Guarantee-indexed types**: the lattice tag as a type-level index (e.g. a function demanding `Exact` input is a *type error* to call with `Declared`), with `meet` as the composition law — the honesty rule moved into the type system | beyond-Rust goal; mechanism position recorded (Q3/T3.2) |
| **LR-7** | Effects: at minimum, partiality and swap-effects tracked; full effect system | position recorded (Q4/T3.4): divergence-only |
| **LR-8** | Ownership/borrowing: **likely not applicable as in Rust** — Mycelium is a value-semantics substrate (no aliased mutable state to police). What may be needed is *linearity/affinity for external resources* only | position recorded (Q5/T3.5): confirmed not applicable; affine `Resource` hook reserved |
| **LR-9** | **Memory safety by construction, leaks structurally excluded** — Rust-grade safety *outcomes* without Rust's mechanism. Value semantics (immutable values, no aliased mutable state) already removes use-after-free, data races, and double-free from the language model; on top of that the language exposes **no manual allocation/free**, reclamation is automatic and deterministic (Perceus-style reference counting + region/scope inference; T3.5), and the *only* leak vector — an unreleased **external resource** — is closed by the affine `Resource` kind (LR-8). Any operation that could violate these (raw FFI, foreign memory) is **not reachable from safe code**: it must be lexically marked (an explicit `unsafe`-class form, themed in L3) and is denied by default; safe code cannot leak, and unsafe regions are auditable and minimal. The guarantee: *in safe Mycelium a memory leak is not expressible.* | committed direction; mechanism researched (T3.5); see Q8 |

"Beyond Rust" therefore means LR-5 + LR-6 (+ LR-7 if accepted): paradigm- and honesty-aware types
that no mainstream language has — *not* exotic syntax. KC-2 will test whether the *surface* can
stay boring while the type system carries the novelty (the RR-3 hypothesis). LR-9 makes explicit
that "beyond Rust" is *also* about reaching Rust's safety **outcomes** by a simpler route — value
semantics gets most of them for free, and the rest (leaks, external resources) are closed by
construction rather than by a borrow checker the substrate doesn't need.

### 4.3 Grammar & spec discipline

- **Formalism:** every layer's grammar is **EBNF in the spec** plus a **machine-readable grammar
  artifact** under `docs/spec/grammar/` (one file per layer, content-addressed like the JSON
  schemas), with a **conformance corpus** (`accept/` + `reject/` programs) that parsers and the
  formatter (M-142) are tested against — the same artifact-plus-checker pattern as
  `docs/spec/schemas/` (M-104).
- **Static semantics:** typing rules land in the RFC that introduces each L1 construct, in the
  same judgment style as RFC-0001 §4.5's WF rules; the linter (M-141) and type checker implement
  exactly those rules (local↔CI parity for the language itself).
- **Dynamic semantics:** L1 gets a small-step semantics like M-110's; L2 gets **none** (it is
  elaboration-defined, §3) — this is what keeps the trusted base small (KC-3) while the language
  grows.
- **One canonical formatter:** the α-normalizing formatter (M-142) extends to each layer as it
  lands; the canonical form is what gets content-addressed (S3).

### 4.4 Sequencing (proposed; routes to the phase plan)

1. **Now (this RFC, Draft):** ratify the layering (§3), invariants (§4.1), capability targets
   (§4.2), and spec discipline (§4.3). None of this depends on KC-2.
2. **Phase 3 RFC-0001 revision:** add the L1 node set (`Lam/App/Match/Fix` + data declarations)
   to the Core IR with WF rules; extend interpreter, lowering, differential.
3. **KC-2 (M-002) runs** against the M-020 fragment (or its successor) → verdict picks the L3
   strategy: text syntax | projections | embedded DSL.
4. **L2/L3 RFCs** (typeclasses, modules, the chosen surface), each with grammar artifacts +
   conformance corpus per §4.3.

## 5. Drawbacks

- Four layers is real machinery for a project whose kernel is five nodes; the mitigations are that
  L2/L3 are *untrusted* by construction and that L1 is the only mandatory addition.
- Deferring concrete syntax can feel slow; the counterweight is the corpus's own KC-2 gate — the
  syntax decision is *already* deferred by an Accepted posture, and this RFC just makes the
  deferral safe (nothing else now blocks on it).

## 6. Rationale & alternatives

- **Why elaboration-defined L2 (vs a directly-specified big language)?** KC-3: the trusted base
  stays the L0/L1 kernel; every convenience is reducible to it and inspectable (S4). This is the
  GHC Core / Rust HIR→MIR lesson, and it matches the repo's existing lowering discipline.
- **Why not adopt Rust's surface wholesale?** Ownership/borrowing solves aliased mutation, which a
  value-semantics substrate doesn't have (LR-8); and S1/S2/LR-5/LR-6 have no Rust counterpart —
  the type system here must carry honesty and Repr, which is the actual product.
- **Why not commit the M-020 syntax?** It is explicitly throwaway, exists to run KC-2, and
  committing it would invert the experiment's purpose.

## 7. Prior art

Unison (content-addressed definitions, names-as-metadata — ADR-003's source); GHC (surface →
Core elaboration with a small trusted core); Rust (HIR/MIR staging; trait coherence); Dex/Futhark
(typed array calculi with explicit effects); F* / Liquid Haskell (refinement-indexed types — the
nearest relatives of LR-6); Idris/Agda (totality checking informing LR-4's posture).

## 8. Unresolved questions — with Pass-3 positions

The third research pass (`research/03-language-layer-RECORD.md`, findings **T3.1–T3.6**)
grounded each open question. A *position* below is a researched recommendation awaiting
ratification with the rest of this Draft; what remains genuinely open is marked.

- **Q1 (KC-2 gate):** text syntax vs projections vs embedded DSL for L3 — still decided by
  M-002 (#3), but the experiment is now **designed from evidence** (T3.6): five conditions
  (bare novel syntax; + book-quality grammar-in-context; + constrained decoding measured
  separately; familiar-skinned same-AST — the ablation no published study has run; eDSL), with
  the **LLM-leverage retention ratio** as headline metric and an explicit falsification
  threshold (spec-in-context retaining <~70% of the familiar-skin condition's pass@1 on
  composition tasks ⇒ L3 becomes a projection of known syntax). Working hypothesis, falsifiable:
  novel-but-regular syntax + grammar-in-context + content-addressed skins retains most leverage
  (MTOB; grammar prompting; Unison-proven projection architecture).
- **Q2 (L1 node budget) — position (T3.1):** L1 = L0 + `Lam, App, Construct (saturated), Match
  (flat/exhaustive, Maranget-compiled), Fix` (~10 expression nodes — the GHC-Core/Lean/Coq
  convergence zone); **data declarations are content-addressed registry entries**, not nodes
  (every surveyed kernel does this); recursive groups hash Unison-style (cycle as a unit,
  members ordered by cycle-removed hashes). General `Fix` in the kernel; the totality checker
  lives outside it (T3.4). Open: exact `Match` binder/default form; whether `Fix` is a node or a
  recursive-`Let` flag.
- **Q3 (LR-6 mechanism) — position (T3.2):** a **graded coeffect modality over the guarantee
  meet-semilattice**, soundness stated as DCC-style noninterference read in the *integrity*
  direction ("no `Declared` input influences an `Exact`-typed output except through a certified
  `Swap`" — the certificate is controlled endorsement; the guarantee lattice is formally an
  integrity lattice, Biba-dual to confidentiality). Refinement types are reserved for
  certificate side-conditions ("Proven with checked side-conditions" as a refinement premise on
  `Swap`), not for the 4-point tag. Staged: runtime tags (built) → static grades (monomorphic,
  then bounded grade polymorphism with FlowCaml-style inference — near-trivial over a 4-chain) →
  refinement premises. Open and **must be recorded as a decision**: whether *implicit flows*
  taint (does branching on a `Declared` value degrade the result, requiring a `pc`-like index,
  or do guarantees track data lineage only?). Flagged novel: grading + runtime certificates has
  no found precedent — needs its own soundness argument.
- **Q4 (effects) — position (T3.4):** **no algebraic-effect system.** Divergence-only tracking:
  one per-definition bit (`total` vs `partial`), Koka's `div` effect in degenerate form; growth
  path documented (bit → small fixed row → row polymorphism), untyped handlers ruled out
  (honesty rule). Partiality stays in values; swap stays lexical (S1).
- **Q5 (linearity) — position (T3.5):** ownership/borrowing **not applicable** (it polices
  aliased mutation, which value semantics excludes; Hylo/Swift demonstrate
  exclusivity-by-construction; in-place performance is Perceus-style implementation work, not
  typing). Reserve an affine `Resource` kind hook for external handles (what linearity is
  actually used for in practice — Linear Haskell experience); ship nothing now.
- **Q6 (literals & bounds) — position (T3.1-B):** literals are **universal until elaboration**
  (Ada-style), then assigned exactly one representation type by suffix or inference with **no
  defaulting across representation families** (stricter than Rust's `i32` default); Ada-2022-
  style literal functions for `VSA{...}` construction. Open: concrete suffix/annotation
  spelling (an L3/KC-2 matter).
- **Q7 (new, from T3.4):** the formal statement of "checked total" for stable-component
  promotion — adopt CakeML-style clock quantification (terminates for every sufficiently large
  fuel under the reference interpreter)? Flagged novel: totality *gating AOT promotion*
  specifically has no found precedent (Idris gates trust; Lean gates kernel reduction) — needs
  its own argument, by analogy.
- **Q8 (new, from LR-9):** the reclamation/safety *mechanism* — confirm Perceus-style reference
  counting (Reinking et al., PLDI 2021; T3.5) as the default reclamation strategy, with
  region/scope inference where it tightens lifetimes, and the affine `Resource` kind (LR-8) as
  the sole external-resource discipline. Open sub-questions: cycle handling under reference
  counting for a value-semantics language (immutable acyclic value graphs make cycles rare but
  not impossible once recursive closures/data enter at L1 — does the language forbid value cycles,
  detect them, or fall back to a tracing pass?); the exact spelling and audit story of the
  `unsafe`-class boundary (S6/LR-9 require it denied-by-default and lexically marked — the L3
  spelling is a DN/KC-2 matter). The *outcome* (no leaks in safe code) is committed; the
  mechanism choice is the open part.

## 9. Future possibilities

Guarantee-polymorphic libraries ("works at whatever strength your inputs have, returns the
meet"); certified-swap combinators as first-class values; an LLM-facing canonical projection
(KC-2's "beyond" case) where the *same* content-addressed definitions render differently for
human and machine co-authors (FR-S5's dual intelligibility).

## 10. Ratification scope (ratified 2026-06-15 — the carve-out, **Accepted**)

This RFC moved `Draft → Accepted` on 2026-06-15 (maintainer sign-off) with the scope below — a
completion-review found **no missing normative content** in the KC-2-independent scope, the §8
positions are researched, and the dependent designs have since landed. The split of what is **now
ratified** vs what stays explicitly gated:

**Ratified (KC-2-independent — RFC-0006 §4.4 step 1).** The load-bearing, evidence-grounded content
that nothing else blocks on:

- **§3 layering** (L0–L3; only L0/L1 trusted) and **§4.1 invariants S1–S6** (never-silent swap,
  honest tags surface, content-addressed identity, inspectable elaboration, explicit partiality, and
  the foundational **S6 AI-independence**) — restatements of committed corpus rules at the language
  level; no open question.
- **§4.2 capability targets LR-1…LR-9** (the "Rust-class and beyond" requirements, incl. **LR-9**
  memory-safety-by-construction) — requirements, not mechanisms; the *outcomes* are committed.
- **§4.3 grammar/spec discipline** (EBNF-in-spec + machine-readable grammar artifact + accept/reject
  conformance corpus per layer) — already realized in `docs/spec/grammar/`.
- **§8 positions Q2, Q4, Q5, Q7** — these are **now realized** by the dependent designs (so the
  positions are no longer merely "awaiting ratification", they are discharged): **Q2** L1 node budget
  with declarations-as-registry → RFC-0007 §4.1/§4.2 (and the `Match`-into-L0 path → **RFC-0011**, the
  ratified staged r3); **Q4** divergence-only effect tracking → RFC-0007 §4.5; **Q5** affine
  `Resource` hook, ownership-not-applicable → RFC-0007 §4.7; **Q7** "checked total" = CakeML clock
  quantification → RFC-0007 §4.5. Ratifying RFC-0006 ratifies these positions as the committed
  direction.

**Stays gated / deferred (explicit, honest — must NOT be read as ratified).**

- **Concrete L3 surface syntax — KC-2-gated (Q1; the one deliberate deferral).** Blocked on M-002
  (#3), LLM-API-external. The Q1 experiment is *designed* (T3.6) but unrun; its verdict picks text /
  projections / embedded DSL. Also gated: **Q6** concrete literal suffix/annotation spelling, and the
  **Q8** `unsafe`-class L3 spelling (a DN/KC-2 matter). Ratifying RFC-0006 ratifies the layering and
  the *deferral itself*, never a syntax.
- **Stage-1 static guarantee grading (Q3).** The graded-coeffect mechanism is *positioned* (T3.2) but
  its load-bearing open sub-decision — **whether implicit flows taint** (a `pc`-like index vs data
  lineage only) — is a genuine open **normative decision for the grading RFC** (a revision of
  RFC-0007 §4.3), not ratified here. v0 stays runtime-tag-checked (RFC-0007 §4.3).
- **Q8 reclamation mechanism details.** The *outcome* (no leaks in safe code, LR-9) is committed;
  Perceus-as-default + cycle handling + region inference are implementation/mechanism choices
  refined later (T3.5), not blocking this ratification.

**Status line (now in force):** *Accepted — layering, invariants (S1–S6), capability targets
(LR-1…LR-9), grammar discipline, and the §8 Q2/Q4/Q5/Q7 positions; concrete L3 syntax (Q1/Q6) stays
KC-2-gated and stage-1 grading (Q3) stays open for the grading RFC.*

## Meta — changelog

- **2026-06-15 (r4) — Accepted (maintainer sign-off).** Moved `Draft → Accepted` with the **§10
  carve-out**: ratified = §3 layering, §4.1 invariants S1–S6, §4.2 capability targets LR-1…LR-9, §4.3
  grammar discipline, and the §8 positions Q2/Q4/Q5/Q7 (realized by RFC-0007 §4.1–4.7 and the ratified
  **RFC-0011** staged-r3 `Match`-into-L0 decision). **Still gated/deferred (NOT ratified):** concrete
  L3 syntax (Q1/Q6, KC-2/M-002-external), the `unsafe`-class L3 spelling (Q8), and stage-1 static
  grading (Q3, incl. the open implicit-flows decision → the grading RFC). A completion-review found no
  missing normative content in scope; no design content changed on acceptance — §10 records the scope.
  The status line carries the carve-out so "Accepted" is never read as ratifying the gated parts (VR-5).
- **2026-06-10 (r3) — Draft, two foundational requirements added (maintainer direction).**
  **S6 (self-sufficiency / AI-independence):** Mycelium is a complete SWE programming language
  runnable with no AI/LLM in the loop — models are an optional co-authoring convenience, never a
  runtime/compile-time/semantic dependency; this *bounds* KC-2 (it can choose the surface, never
  make the language need a model). **LR-9 (memory safety by construction):** Rust-grade safety
  outcomes without the borrow checker — value semantics removes use-after-free/data-races/
  double-free from the model, no manual alloc/free, automatic deterministic reclamation, the only
  leak vector (external resources) closed by the affine `Resource` kind, and any unsafe operation
  denied-by-default and lexically marked; *in safe Mycelium a memory leak is not expressible*. New
  **Q8** records the open mechanism question (Perceus + regions; cycle handling; `unsafe` spelling).
  Both grounded in T3.5.
- **2026-06-10 (r2) — Draft, research-grounded.** Folded in Research Pass 3
  (`research/03-language-layer-RECORD.md`, T3.1–T3.6): §8 now records a researched *position*
  per question — L1 node budget + declarations-as-registry + Unison-style cycle hashing (Q2);
  guarantee-indexing as a graded coeffect modality with IFC/DCC integrity-noninterference as the
  soundness story and refinements reserved for certificate side-conditions (Q3); divergence-only
  effect tracking (Q4); ownership not-applicable + reserved affine `Resource` hook (Q5);
  universal-until-elaboration literals with no cross-family defaulting (Q6); the KC-2 experiment
  redesigned from measured evidence with a falsification threshold (Q1); new Q7 (formal
  "checked total" statement; flagged-novel pieces named). §4.2 posture column updated to point
  at the findings. Still **Draft** — positions await ratification.
- **2026-06-10 — Draft.** Initial deliberation draft: layering L0–L3, syntactic honesty
  invariants S1–S5, capability requirements LR-1…LR-8 ("Rust-class and beyond" made checkable),
  grammar/spec discipline, sequencing, and the open questions Q1–Q6. Concrete L3 syntax is
  explicitly gated on KC-2 (M-002), per the existing M-020/RR-3 posture. Ratification
  (Draft → Accepted) is a maintainer decision.
