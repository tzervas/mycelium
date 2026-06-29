# Research Record 27 — DN-64 Ergonomics R&D Planning: OQ-B/G/H/I/J/R/T Dispositions

> **What this file is.** A durable R&D planning record for the seven open questions from
> DN-64 §6 that the maintainer dispositioned (OQ-B, OQ-G, OQ-H, OQ-I, OQ-J, OQ-R, OQ-T — OQ-H
> the record-literal shorthand, added at integration), each mapped to a minted M-task
> (M-828, M-833, M-846, M-834, M-835, M-843, M-845 — minted in DN-64 §7).
> Conducted 2026-06-29 from DN-64 §2/§5/§6, RFC-0005, RFC-0008 RT3/§4.5, RFC-0018 §4/§5,
> RFC-0020 §4.1/§4.4, RFC-0025, RFC-0037 D2-b, RFC-0001 §4.7, RFC-0013 §4.6, DN-02 §1/§7,
> DN-03 §3, DN-31 §2, DN-63 §2/§3.5/§3.6, ADR-006.
>
> **Posture (transparency rule / VR-5).** This is a planning and analysis record, not a
> decision record. Every proposal is tagged `Declared` (design direction, advisory) unless
> grounded in a checked corpus basis. No claim here supersedes an existing ADR/RFC; all
> recommendations are for the maintainer's consideration. Append-only.

---

## 1. Scope

DN-64 §6 catalogs twenty open questions across four facets. The maintainer dispositioned six
for R&D planning and new M-tasks: OQ-B (forage/backbone activation and mechanized
SelectionPolicy capture), OQ-G (guard clauses and guarantee propagation), OQ-I (short-keyword
scope and trait split), OQ-J (ergonomic wrappers for stringent annotation doctrine), OQ-R
(composite-operation guarantee aggregation), and OQ-T (proposal-time three-test gate and
one-name discipline for library-defined names). This record provides grounded analysis and
recommended next steps for each, referencing the tasks by their minted M-ids.

---

## 2. OQ-B — `forage`/`backbone` activation and mechanized SelectionPolicy capture (M-828)

### 2.1 Maintainer decision (faithful)

`forage` and `backbone` must be made ACTIVE (promoted from R2 reserved-not-active to active
syntax). The selection-policy mechanism and swap machinery must support **mechanized capture
and setting** of selection policies to improve ergonomics, WHILE RETAINING transparency,
provenance, and explainability — no black box, no silent selection.

### 2.2 Grounded analysis

**Current status (`Exact` from corpus).** `forage` and `backbone` are R2 reserved-not-active
(DN-63 §2; RFC-0008 §4.5 table). Their operational meanings are normatively fixed: `forage` is
adaptive placement as a reified RFC-0005 policy (the third application site of the one selection
mechanism, RT3); `backbone` is a declared high-bandwidth transport path, semantics-free, consumed
as an input to `forage` placement policies. Neither has a typing strategy or elaboration rule
yet — per RFC-0008 §4.5/§4.6, activation requires an implementation RFC with those specified.
(`Exact` — RFC-0008 §4.5, DN-63 §3.5/§3.6.)

**The three-site unification is Accepted direction, not yet fully enacted (`Declared`).**
RFC-0005 §4 establishes SelectionPolicy as the single mechanism for swap-target selection
(RFC-0002), packing schedule (RFC-0004), and task placement (RFC-0008 RT3). The unification is
`Declared` (DN-64 §1.10: "Accepted direction, not yet fully enacted in code"); activating
`forage` is the third-site completion of this unification.

**Why "mechanized capture and setting" is ergonomically load-bearing (`Declared`).**
Policies are content-addressed first-class values (RFC-0005 §3). In the current design, a
developer who wants to reproduce a runtime selection decision or prescribe a known-good policy
must construct the policy value by hand. Mechanized capture means: the runtime can record which
policy it applied (already in `Meta.policy_used`, RFC-0001 §4.3) and a developer can
extract that policy value for reuse or diffing. Mechanized setting means: a setter surface
that binds a new SelectionPolicy to a `forage` site without re-writing the policy expression
from scratch. This improves ergonomics precisely because policy values are heavy-structured
artifacts, and round-trip record-and-replay capability removes a significant friction point.

**Transparency constraints that mechanized capture must respect (`Exact` from corpus).**
RFC-0005 §2 mandates that every automatic selection emits an inspectable record of inputs
considered, costs, the chosen option, and a deterministic override hook. ADR-006 (no black
boxes) applies at every level: a "capture" mechanism that hides how a policy was derived would
itself be a black box. The capture must surface the policy as a nameable, diffable, inspectable
RFC-0005-conformant value, not an opaque handle. The provenance DAG (RFC-0001 §4.3) must record
which captured policy drove which placement decision. EXPLAIN must remain answerable even after
a mechanized-set policy is active.

**Dependency ordering (`Declared`).** M-828's direct `depends_on` is **[M-825, M-668]**
(`issues.yaml`): M-825 (the backbone design decision) and M-668 (the R2 planning gate). M-668 in
turn depends on M-667 (the R1 completion gate, RFC-0008 §4.6 — R1 must complete before R2
activates), so M-667 is a *transitive* predecessor of M-828, not its immediate one. The mechanized
capture and setter surface is R2-tier work; it cannot advance to Accepted until R1 is done.
The SelectionPolicy language (RFC-0005) is already Accepted and its three-site unification
is the design basis; M-828 is an activation-and-ergonomics task within that design, not a
new policy language.

**Open tensions to surface (`Declared`).**
- FLAG-15 from DN-63 §3.6: how is `backbone` declared and managed — per-phylum or
  per-colony/image? This scoping question must be resolved in the `backbone` implementation
  RFC before activation.
- FLAG-2 from DN-63 §3.5: does `xloc` require a declared `backbone` path, or does it fall
  back to `mesh`? The `forage`/`backbone`/`xloc` interaction is an open design gap in the
  placement layer.
- "Mechanized setting" must not become a silent override: a policy setter that silently
  replaces a previously-EXPLAIN-able choice with an opaque one would violate ADR-006. The
  setter surface must itself be inspectable and record the transition (G2).

### 2.3 Recommended next steps

1. **M-828 implementation RFC — `forage` activation:** following DN-63 §3.5's typing and
   elaboration sketch (`Declared`), write the implementation RFC that specifies `forage`'s
   type surface, elaboration to a placement-policy binding on the hypha scope, the RT2
   differential obligation, and the input signals to the placement policy (DN-63 §3.5 lists
   these as currently unspecified — a FLAG that must be resolved before the RFC can be
   Accepted). (`Declared` direction.)
2. **`backbone` implementation RFC:** per DN-63 §3.6's sketch, specify `backbone` as a
   declaration elaborating to a transport-hint binding, with content-addressed handle type
   `BackboneRef`, fallback behavior (`Err(BackboneError::Unavailable)`), and the scoping
   question (FLAG-15) resolved. (`Declared` direction.)
3. **Mechanized capture design:** extend RFC-0005's mandatory EXPLAIN output with a
   capture-to-value operation that materializes the policy record into a `SelectionPolicy`
   value, content-addressed, reusable. The capture must be dumpable, not a side-channel
   (`Declared` design direction; no black box, ADR-006).
4. **Setter surface design:** a `forage(policy: capture)` form that accepts a captured
   or hand-constructed `SelectionPolicy` and binds it to the site; the binding is recorded
   in the provenance DAG so EXPLAIN answers "this placement used policy P, captured from
   run R." (`Declared`.)

All steps are post-R1-completion work (M-667 gate).

---

## 3. OQ-G — Guard clauses and guarantee propagation (M-833)

### 3.1 Maintainer decision (faithful)

Yes, ratify guards (`when <cond>` guard clauses on patterns). Unless the guard's truth is
mechanically proven, the arm drops to `Declared` (VR-5). The arm's guarantee tag is the
meet of the guard's tag and the arm body's tag; if the guard is `Declared`, the arm becomes
`Declared`.

### 3.2 Grounded analysis

**Current status of or-patterns and guards in RFC-0020 (`Exact`).** RFC-0020 §4.4 specifies
the committed L2 pattern compilation algorithm (Maranget-style decision-tree compilation).
The §4.4 surface pattern table marks or-patterns as `reserved, not yet active` in v0 and
guards are not in the table at all — RFC-0020 §4.1 S2 does not specify guard semantics.
(`Exact` from RFC-0020 §4.4.)

**How guards fit the Maranget compilation framework (`Declared`).** The Maranget usefulness
algorithm (RFC-0020 §4.4; enacted in `crates/mycelium-l1/src/usefulness.rs`) compiles pattern
matrices to flat `Match(Match(...))` trees. A guard is a predicate attached to an arm: the arm
fires only if the guard evaluates to `True`. In standard Maranget compilation, a guarded arm is
treated as a split: (a) if the guard holds, fire the arm body; (b) if the guard does not hold,
fall through to the next arm. The usefulness check must treat a guarded arm as potentially
non-exhaustive even when the constructor pattern is exhaustive (because the guard may fail),
which means the matrix must have a subsequent arm that covers the guard-failure case. This is
the correct S5 (explicit partiality) outcome: a guarded arm that has no fallback arm for
guard-failure is inexhaustive, and the elaborator must reject it. (`Declared` direction,
grounded in Maranget 2008 / RFC-0020 §4.4.)

**Guarantee tag propagation — the VR-5 application (`Exact` basis, `Declared` direction).**
RFC-0018 §4.3 specifies G-Match: the result grade is the meet of all branch body grades
(under Design A, the ratified choice — R18-Q1). The maintainer decision extends this to guards:
the arm's effective grade is `grade(guard) ∧ grade(arm_body)`. The reasoning is sound under
VR-5 (annotation may only weaken, never upgrade without a checked basis): if a guard's truth
is `Declared` (an assertion not mechanically verified), the decision to fire this arm depends
on an unverified condition, so the arm's output grade can be at most `Declared`. An `Exact`
or `Proven` guard (e.g., a guard whose condition is a checked side-condition) propagates its
strength into the arm, and the arm's final grade is the meet of the guard's grade and the body's
grade. This is the meet rule of RFC-0018 §4.1 applied to the guard as an additional input.
(`Exact` VR-5 basis; `Declared` elaboration direction for guards.)

**The one-canonical-form constraint.** The guard syntax must be S1-safe (no implicit swap in
the guard expression) and S4-transparent (the guard elaboration is dumpable). A guard that
invokes a representation-changing operation must produce a `MissingConversion` error, not a
silent swap (RFC-0006 §4.1 S1). Guard elaboration adds one elaboration step that is separately
dumpable per the S4 obligation (RFC-0020 §4.1 S4 at L2).

**Open tension.** The maintainer decision says "unless mechanically proven the arm drops to
`Declared`." The open question is what counts as "mechanically proven": is a guard expression
whose Maranget-checked exhaustiveness proof covers all guard outcomes sufficient to upgrade the
arm, or does upgrade require a `Swap` node with a certificate? The corpus is clear that only
`Swap` nodes are the endorsed upgrade points (RFC-0018 §4.3 G-Swap; §3.3). A guard that
evaluates to a `Proven`-grade Boolean can contribute at most `Proven` to the meet — but cannot
upgrade a `Declared` arm body to `Proven` without a certificate. This tension is correctly
captured by the meet rule: the arm is exactly the meet of the guard's grade and the body's grade.
No separate upgrade path is needed. (`Exact` — RFC-0018 §4.3.)

### 3.3 Recommended next steps

1. **M-833 RFC amendment to RFC-0020 §4.4:** add `when <cond>` guard syntax to the surface
   pattern table with the guard-elaboration rule (split into guard-true path and fallthrough),
   the usefulness rule (guarded arm is potentially partial, fallback required), and the
   guarantee propagation rule (arm grade = `grade(guard) ∧ grade(body)`). (`Declared` direction.)
2. **Conformance corpus extension:** add cases for exhaustiveness rejection (guarded arm with
   no fallback), `Declared`-guard degradation, and `Proven`-guard meet with `Proven` body.
   (`Declared` direction.)
3. **No upgrade mechanism for guards beyond the meet rule:** do not introduce a guard-specific
   upgrade path; `Swap` remains the sole endorsed upgrade point (RFC-0018 §3.3). The meet
   rule is sufficient and correct. (`Exact` basis.)

---

## 4. OQ-I — Short-keyword scope; split methods/associated types (M-834)

### 4.1 Maintainer decision (faithful)

Split trait methods and associated types for clarity / one-canonical-form discipline. The short
keywords (`bin{N}`, `tern{N}`, `emb{D,S}`, `hvec{E,D,Sp}`) bind to type literals ONLY — not
to trait methods or associated types.

### 4.2 Grounded analysis

**Current status of short keywords (`Declared`).** RFC-0037 D2-b (a **remaining follow-on**,
not in RFC-0037's Enacted scope) proposes `bin{N}`, `tern{N}`, `emb{D,S}`, `hvec{E,D,Sp}` as
syntactic aliases for `Binary{N}`, `Ternary{N}`, `Dense{D,S}`, `VSA{E,D,Sp}`. They appear in
RFC-0037's **proposed** `base_type` grammar (its normative §6 EBNF) but are **not yet in the
committed `docs/spec/grammar/mycelium.ebnf`** (whose `base_type` production carries only the long
forms). RFC-0037 §4 (D2-b paragraph) records FLAG-3 (short keyword lexer status: reserved keyword
vs soft keyword) as open and notes the open question of their scope — type literals only vs. also
trait methods/associated types. (`Declared` — RFC-0037 D2-b is a proposed follow-on; only the
long-form `base_type` production is `Exact` in the committed EBNF.)

**Why type-literal binding only is correct (`Declared` direction, grounded in corpus).**
The one-canonical-form discipline (DN-03 §3, DN-64 §5.8) holds that each concept has exactly
one name. If `bin{N}` also bound to trait method resolution or associated types, it would
introduce a second canonical form for paradigm-parameterized trait interfaces — meaning a
method declared as `fn add(a: Binary{8}, b: Binary{8}) -> Binary{8}` could also be invoked
via a `bin{8}`-keyed path. This forks the canonical form for one concept, which DN-03 §3
explicitly forbids. The ergonomic benefit of short aliases is at the type-literal level (less
typing in type signatures); extending them into method resolution creates a dual-lookup
surface that complicates elaboration without DX benefit.

**Associated types are deferred in v1 (`Exact`).** RFC-0019 §8 defers associated types to v2
(research record 10, T10.8: defer with the roles RFC). If associated types do not exist in v1,
the question of whether short keywords bind to them is moot for v1. Recording the constraint
now prevents a future RFC from accidentally extending the short keyword scope — the decision
is append-only: short keywords bind to type literals.

**The split between trait methods and associated types (`Declared` direction).**
The "split for clarity" part of the decision refers to the broader trait surface: trait methods
(behavior specifications) and associated types (type members) should have syntactically
distinct declaration forms, so a trait definition is unambiguous about which of its members
are callable methods vs. type constraints. This is a direction for the RFC-0019 scoped
revision (associated types track) and the inherent-method surface (`impl` keyword, DN-03 §1).
It does not change the short-keyword scope — it is a separate trait-surface clarification.

**Interaction with RFC-0037 FLAG-3 (`Declared`).** FLAG-3 is whether short keywords are
fully reserved (new reserved words in the lexer) or soft keywords (contextually recognized).
Making them fully reserved is the safer choice for the type-literal-only constraint: a fully
reserved `bin` cannot accidentally be used as an identifier in a method signature context,
which would blur the type-literal-only scope. The soft-keyword approach requires parse-context
tracking to prevent misuse. The maintenance cost of a small reserved-word set expansion is low
(DN-02 §6: the guiding principle is to reserve fewer words, but DN-02 §6 also accepts
reservations where they are load-bearing for clarity). (`Declared` direction; FLAG-3
resolution is M-834's decision to make.)

### 4.3 Recommended next steps

1. **M-834 RFC-0037 follow-on or FLAG-3 resolution task:** formally resolve FLAG-3 (reserved
   vs. soft keyword) with the constraint that short keywords bind to type literals only.
   Specify this in an RFC-0037 follow-on section or in the implementation RFC for D2-b
   completion. (`Declared` direction.)
2. **Elaboration rule for short keywords:** the elaborator must treat `bin{N}` as syntactic
   sugar for `Binary{N}` at the type-position only; any occurrence in a method signature
   position elaborates to the long form, and any attempted use in a trait item position that
   would require short-keyword-to-associated-type binding is an explicit error (not a silent
   fallback). (`Declared` direction.)
3. **Trait method/associated type split RFC direction:** record the split-for-clarity principle
   as a normative direction in the RFC-0019 scoped revision (associated types RFC, when it
   advances). This is M-834's companion planning note, not its implementation deliverable.
   (`Declared` direction.)

---

## 5. OQ-J — Ergonomic wrappers for the stringent annotation doctrine (M-835)

### 5.1 Maintainer decision (faithful)

For the stringent required doctrine (stage-1a explicit grade annotations on every function
signature, RFC-0018 §5), provide wrappers, decorators, and/or tooling ergonomic implementations
to ease use WITHOUT degrading guarantees. The guarantees remain; the annotation burden is the
target for ergonomic reduction.

### 5.2 Grounded analysis

**The annotation burden is real and unmitigated in stage 1a (`Exact`).** RFC-0018 §5 notes that
stage 1a (monomorphic grades, the enacted phase) requires explicit grade annotations on every
function signature. Stage 1b (grade polymorphism with inference over the 4-chain) is deferred —
it is the mechanism that would allow a function to be written once and typecheck at multiple
grades without re-annotation. DN-64 §6 OQ-J names T3.6 (the rigorous ablation — Mycelium
surface fragment vs. Python-embedded DSL, with and without semantic feedback) as not yet run.
Until stage 1b lands, every new function requires an explicit `@ g` annotation on every parameter
and return type. A medium-sized application with 50 functions has 50-plus explicit grade
annotations that stage 1b would infer. (`Exact` — RFC-0018 §4/§5, DN-64 §6 OQ-J.)

**Prior art for annotation-burden ergonomics (`Declared` analysis).**
- **Rust's `#[derive(…)]`** pattern (research/25 Topic 2): terse intent generates explicit,
  inspectable implementations. A `derive`-style `#[grade(Empirical)]` wrapper on a function
  that propagates its grade annotation through its signature is the direct analogue. The
  generated annotation would be dumpable (S4) and carry the same force as a hand-written one.
- **Default-grade affordances:** a nodule-level `default grade Empirical` declaration that
  assigns `Empirical` as the default grade for all functions in scope unless overridden,
  analogous to RFC-0012's ambient paradigm. This would reduce the per-function cost to an
  override annotation only when the grade deviates from the default. RFC-0012 §4.7 is the
  direct precedent: the ambient paradigm is a nodule-scoped default for repr that the
  elaborator fills in, dumpable per S4, never strengthening, never silent. The same pattern
  can apply to grades.
- **Tooling-assisted annotation (LSP affordance):** the LSP (SC-5/M-310) can infer and
  suggest the correct grade annotation for a function given its body's meet-of-inputs grade
  (which the stage-1a checker already computes). This converts the annotation burden from
  "figure out the grade" to "accept the LSP suggestion." The suggestion is advisory (the
  developer confirms), but the mechanical inference is already available from the checker.

**What "without degrading guarantees" means in corpus terms (`Exact`).**
S2 (honest tags surface, RFC-0020 §4.1) and VR-5 prohibit any ergonomic affordance from
strengthening a grade or hiding a weaker grade. A default-grade mechanism may only fill in
a grade that is as weak as or weaker than what the checker infers — it cannot upgrade.
A `derive`-style annotation wrapper propagates the stated grade to every parameter and return
position; if the body has a weaker implied grade, the checker rejects it (G-Weaken fails).
The ergonomic mechanism reduces authoring friction for the common case (functions that
legitimately carry a uniform grade) without creating a path to silent grade elevation.
(`Exact` — RFC-0018 §4.3 G-Weaken, RFC-0020 §4.1 S2.)

**Interaction with the ambient paradigm precedent (`Declared`).**
RFC-0012's ambient paradigm (enacted, RFC-0020 §4.7) shows how a nodule-level default
elaborates away: the ambient fills a position, the elaborated L1 carries the filled value,
and the stage-dump shows the filled form. A `default grade` mechanism follows the same
discipline: it is an elaboration default, not a semantic change, and it never fills a
position in a way that the checker would otherwise reject. The limit case: a function body
that provably returns `Exact` under the checker can be annotated `Exact`, whether by
hand or by a derive/default mechanism — the guarantee is not degraded; the burden is.

**Open tension.** Grade polymorphism (stage 1b) is the "real" ergonomic fix: once grade
inference is available, explicit annotations become optional for most functions. The ergonomic
wrappers proposed here are interim mitigations, not substitutes for stage 1b. M-835 should
specify them as stage-1a ergonomics, explicitly staging them as pre-1b affordances with a
note that stage 1b supersedes them where applicable. (`Declared` direction.)

### 5.3 Recommended next steps

1. **M-835 default-grade ambient design:** specify a `default grade <G>` nodule-scoped
   directive, in the same pattern as `default paradigm` (RFC-0012 §4.3), that fills
   unannotated grade positions with `<G>`. The elaborator fills and dumps per S4; the checker
   rejects any fill that would strengthen above the body's inferred grade. (`Declared` direction.)
2. **M-835 derive-style grade annotation:** specify a `derive Grade(<G>)` form (or equivalent
   decorator syntax, pending DN-54's generative-lowering surface) that synthesizes explicit
   `@ <G>` annotations on every parameter and return position of the target function. The
   generated annotations are inspectable via `reveal` (DN-38/DN-54). (`Declared` direction.)
3. **LSP grade-suggestion affordance:** specify an LSP quickfix/completion that proposes the
   inferred grade for an unannotated position, grounded in the stage-1a checker's output.
   The proposal is advisory (the developer approves); it generates a concrete `@ g` annotation
   in the source, not a hidden default. (`Declared` direction.)
4. **Stage 1b dependency note:** M-835 deliverables are explicitly stage-1a-era ergonomics;
   the M-835 task description must record that stage 1b (grade polymorphism RFC) supersedes
   points 1 and 2 for the inference-resolved subset. (`Declared`.)

---

## 6. OQ-R — Composite-operation guarantee aggregation (M-843)

### 6.1 Maintainer decision (faithful)

R&D. The exact scope of the per-operation guarantee tag in composite operations (a function
that calls multiple primitives) is not fully specified. The question is whether a function-level
summary beyond the meet of its operations is well-defined and useful, and if so, what the exact
scope of a composite operation's tag is.

### 6.2 Grounded analysis

**Current corpus position (`Exact`).** RFC-0001 §4.7 specifies the guarantee lattice and
bound composition: for an operation `f` with intrinsic guarantee `g_f` over inputs
`v_1..v_n`, `guarantee(result) = meet(guarantee(v_1), ..., guarantee(v_n), g_f)`. This rule
is per-operation (each swap, each prim). A `Datum` (compound value built by `Construct`) has
a meet-summary guarantee over its fields with no bound (RFC-0001 §4.7 meet-summary addendum).
RFC-0018 §4.3 lifts this to the graded type level: the G-Let/G-App/G-Op rules propagate
grades by meet through composition. A function's return type carries the grade of the composed
expression, which is the meet of all intermediate grades. (`Exact` — RFC-0001 §4.7,
RFC-0018 §4.3.)

**What is already fully specified (`Exact`).** The per-operation meet rule is complete and
enacted. For any composite expression tree, the grade of the result is the meet of all
constituent grades bottom-up. This is the "function-level summary" in the current corpus: the
function's return grade is implicitly the meet of its operations, enforced by the G-Let/G-App
rule chain. There is no gap at the expression level.

**What is genuinely open (`Declared`).**
Two sub-questions drive the R&D designation:

*Sub-question 1 — Function-level grade annotation in signatures vs. body-inferred meet.*
Stage 1a requires explicit grade annotations on function signatures (RFC-0018 §5). A
function annotated as returning `Exact` must have a body whose inferred grade is at most
`Exact` (the G-Weaken rule). But the question "what is the grade of this function as a
whole" has two answers: (a) the annotated grade in its signature (visible to callers) and
(b) the body's inferred meet grade (the checker's ground truth). These are the same in a
well-typed program; the gap OQ-R targets is whether a *function-level summary* distinct from
the body-inferred meet has any meaning or utility — e.g., a docstring-style grade claim
("this function is `Empirical` for these reasons") that goes beyond the body's inferred meet.
The corpus currently has no such construct; the signature grade IS the summary.

*Sub-question 2 — Scope of "composite operation" in the per-op tagging convention.*
DN-64 §5.5 says "each swap, each approximation, each lossy operation carries its own tag —
a function is NOT tagged once; its operations are." This is correct for fine-grained auditing.
But when a function is called at a distance (by a downstream function that only sees its
signature grade), the caller sees the signature grade — a coarser summary than the per-op
breakdown inside the body. The question is how this coarsening interacts with the convention
that "never aggregate": is a signature grade an aggregation (forbidden) or a type annotation
(permitted)? The corpus answer is that it is a type annotation (RFC-0018 §4.2 — a graded type
is `τ @ g`, not a summary over ops), but the convention language in DN-64 §5.5 is not
precise about this distinction.

**Candidate resolution paths (`Declared`).**
(a) Clarify that the "never aggregate" convention applies to runtime-level guarantee tracking
(every swap carries its own cert; the provenance DAG preserves per-op records), not to
static type signatures (which are per-expression-result grade annotations, not per-op
summaries). This resolves the apparent tension without new design.
(b) Specify whether a function-level grade claim in a docstring or annotation form (separate
from the type signature) is admissible and what its relationship to the body's inferred grade
must be. The corpus currently has no such form.
(c) Specify whether a composite operation (a named function) can be treated as an atomic
operation with its own intrinsic grade `g_f` (like a prim has in the G-Op rule). If yes, the
function's declared signature grade is its `g_f`, and G-App uses it exactly as G-Op uses prim
intrinsic grades. This is already the behavior of RFC-0018 G-App: the function's return type
carries the grade, and G-App does not re-derive it from the body. This is resolution (a).

**Recommended resolution direction (`Declared`).**
The cleanest path is resolution (a): clarify that "never aggregate" refers to the runtime
per-op cert layer, not to the static graded-type layer. A function's signature grade is a
static type annotation (τ @ g), not an aggregation — it is constrained by the checker to be
at most the body's inferred meet. The per-op cert and provenance DAG layer preserves the
fine-grained record. These are two complementary levels, not competing conventions.

**What remains genuinely open (`Declared`).** Whether a function-level grade claim beyond
the signature type is useful (e.g., a structured docstring that records "this function is
`Empirical` because of operation X, which uses the Frady-Sommer basis") is an ergonomics
and tooling question, not a type-system question. It is adjacent to RFC-0013's structured
diagnostics (additive presentation over an explicit truth) and may belong in the EXPLAIN
surface rather than in the type system proper. This sub-question is what makes M-843 R&D
rather than a straightforward spec amendment.

### 6.3 Recommended next steps

1. **M-843 clarification pass:** write a DN or RFC amendment that precisely distinguishes
   the runtime per-op cert layer (which the "never aggregate" convention governs) from the
   static graded-type layer (which G-App/G-Op govern). This removes the apparent ambiguity
   in DN-64 §5.5. (`Declared` direction.)
2. **M-843 function-as-composite-op question:** determine whether a named function with an
   explicit signature grade is treated as an atomic composite operation with intrinsic grade
   `g_f` by callers (the G-App-treats-signature-as-g_f model). If yes, specify this
   explicitly in RFC-0018 as a note to G-App. (`Declared` direction.)
3. **EXPLAIN surface for composite ops:** explore whether the EXPLAIN surface (M-140,
   RFC-0005 mandatory EXPLAIN) can answer "what are the per-op grades inside this function
   call?" for a downstream caller — making the fine-grained record accessible without
   requiring the caller to read the body. This is a tooling affordance, not a type-system
   change, and is the natural complement to the static summary. (`Declared` direction.)

---

## 7. OQ-T — Proposal-time three-test gate (M-845)

### 7.1 Maintainer decision (faithful)

Apply the same three-test gate (T-map/T-illuminate/T-learn) at proposal time, not only at
ratification. Additionally: address the DN-03 §3 sub-question of whether the "one name per
term" rule binds library-defined names — specifically whether a phylum can export both `add`
and `plus` as aliases for the same function.

### 7.2 Grounded analysis

**Current state of the three-test gate (`Exact`).** DN-02 §1 specifies the three-test gate
as a ratification-time criterion: a candidate term must pass T-map (fidelity — the name maps
accurately to behavior), T-illuminate (teaching value — the name teaches the semantics), and
T-learn (dual readability — the name reads well for both humans and LLMs). DN-02 §7 records
the maintainer ratifications that applied the gate to the founding vocabulary. DN-03 §3
establishes the "one name per term" flat rule, superseding ADR-012 §7.6's canonical-plus-alias
scheme, and applies it to the ratified Runtime vocabulary. (`Exact` — DN-02 §1/§7,
DN-03 §3.)

**Why proposal-time application matters (`Declared` direction).**
In the current corpus, the three-test gate was applied to the founding lexicon en-masse at
the design-phase onset. New terms entering the corpus (e.g., from a future RFC proposing a
new construct or from a design note naming a new concept) do not have a formalized process
for when the gate runs. Proposal-time application means: when an RFC or DN introduces a new
term for the first time, the RFC/DN explicitly applies the three-test gate and records the
verdict before the term enters the corpus. This prevents vocabulary drift — a situation where
a term becomes entrenched through use before anyone applies the gate, making later correction
costly. The gate is low-cost to apply at proposal time and expensive to apply retroactively
(existing codebase references, doc cross-refs, test names). (`Declared` direction,
grounded in DN-02's stated purpose and the append-only decision discipline.)

**The sub-question: does "one name per term" bind library-defined names? (`Declared`).**
DN-03 §3 states the flat rule for language keywords and the Runtime vocabulary — these are
terms reserved and active in the lexer/grammar. The question is whether the same rule applies
to identifiers defined by a phylum (library): can a phylum export both `add` and `plus` as
names for the same function? The corpus currently has no explicit ruling. The considerations:

*For binding library-defined names (the restrictive position):*
If a phylum exports `add` and `plus` as aliases pointing to the same content-addressed
L1 definition (same hash), they are two names for one identity (ADR-003 — identity is the
hash, not the name). This is precisely the situation DN-03 §3 intended to eliminate for
language terms. Permitting aliases in phyla would reintroduce the "two spellings per concept"
problem — a downstream nodule could use either alias, and the corpus would be read differently
by different readers, fragmenting LLM leverage (the T-learn dimension).

*Against binding library-defined names (the permissive position):*
Language keywords are different from library identifiers: keywords are part of the language
grammar and reserved at the lexer level, while library names are user-space bindings in the
content-addressed registry. Two names pointing to the same hash are not two definitions —
they are two access paths to one definition. Whether a phylum exposes `add` and `plus` as
aliases is a phylum-author decision, not a language constraint. The grammar and type system
are unchanged. Content-addressing means downstream users always get the same definition hash
regardless of which alias they use.

*Recommended position (`Declared`):*
The "one name per term" rule should bind language-level vocabulary (keywords, stdlib
canonical names) but should be stated as a convention (not a hard type-system constraint)
for user phyla. A phylum SHOULD export one canonical name per concept (the three-test gate
is advisory for phylum authors); the language MUST use one canonical name per keyword and
per stdlib export. This preserves the language's clean vocabulary discipline while respecting
that phylum authors have legitimate reasons for compatibility aliases (e.g., a math library
exporting both `add` and `plus` for discoverability). The constraint is stated in the
CONTRIBUTING/style guide rather than enforced by the type system.

**Interaction with the proposal-time gate process (`Declared`).**
Proposal-time application of the three-test gate should be specified as a normative authoring
requirement: any RFC or DN that introduces a new term (keyword, reserved word, or stdlib
canonical name) must include a three-test gate table before the term is named in the RFC's
normative section. The gate is applied by the RFC author and verified in the `/pr-review`
pass. Terms that fail the gate are declined at the RFC level, not retroactively corrected.
(`Declared` direction, consistent with VR-5: the naming claim "this name is clear" is itself
a claim that must be checked, not asserted.)

**Ground in the corpus of resolved decisions.** DN-02 §7 shows the gate in action for the
founding vocabulary. DN-03 §4 shows it for the Runtime vocabulary (one name each, scored
against RFC-0008's meanings). The proposal-time extension is the discipline generalization:
apply this rigor to every new term going forward, not just the founding sets. (`Exact`
basis for the existing practice; `Declared` for the generalization.)

### 7.3 Recommended next steps

1. **M-845 CONTRIBUTING amendment:** add a section specifying proposal-time gate application
   as a normative RFC/DN authoring requirement: new terms include a three-test table;
   the `/pr-review` skill verifies the table is present and the verdicts are grounded.
   (`Declared` direction.)
2. **M-845 one-name rule scope clarification:** add a note to DN-03 §3 (or a new DN amending
   DN-03) that explicitly scopes the flat rule to language keywords and stdlib canonical names;
   phylum-author aliases are a convention (SHOULD, not MUST). (`Declared` direction.)
3. **M-845 gate template:** provide a reusable table template for RFC/DN authors to include
   when proposing a new term:

   ```
   | Test | Question | Verdict |
   |------|----------|---------|
   | T-map | Does the name map accurately to the behavior? | [Pass/Fail + reasoning] |
   | T-illuminate | Does the name teach the semantics? | [Pass/Fail + reasoning] |
   | T-learn | Does it read well for human and LLM? | [Pass/Fail + reasoning] |
   | **Decision** | | [Accept themed / Keep conventional] |
   ```

   (`Declared` direction, mirroring the DN-02 §1 format.)

---

## 8. OQ-H — Record-literal shorthand shadowing (M-846)

### 8.1 Maintainer decision (faithful)

OQ-H disposition (maintainer 2026-06-29, supplied after the initial 19): **"R&D."** Research the
`{x, y}` → `{x: x, y: y}` deterministic shadowing rules — whether the readability win justifies the
shadowing discipline; the local-vs-field disambiguation must be explicit and never-silent (G2).

### 8.2 Grounded analysis

DN-64 §2.2 lists record-literal shorthand as a `Declared` sugar candidate that "requires careful
shadowing rules (the bound `x` must deterministically refer to the record field)," and §6 OQ-H frames
the open part: "If both a local binding and a record field share the name, the disambiguation rule
must be explicit and never silent." The candidate must satisfy the §2.1 ratified S-invariants:
S3 (identity over elaborated L1 — the shorthand changes no content address), S4 (inspectable
elaboration — `{x}`→`{x: x}` must be dumpable via the stage channel, M-140), and G2 (no silent
choice). (`Exact` for the S-invariants; `Declared` for the candidate.)

Prior art converges on one rule: the shorthand RHS is the **lexically in-scope binding of that name**.
Rust field-init shorthand (`Foo { x }` ≡ `Foo { x: x }`), ES6 object shorthand (`{x}` ≡ `{x: x}`),
OCaml record punning (`{ x; y }`), and Swift all resolve the value position by ordinary lexical
scoping — the field name on the LHS and the value expression on the RHS occupy different namespaces,
so there is no genuine field-vs-local ambiguity, only the ordinary question of which binding of `x` is
in scope. The never-silent obligation is therefore mild: the elaboration is a fixed rewrite and the
stage-dump (S4) already shows the resolved `{x: x}`; the residual risk is a reader *misreading* `{x}`
as a field reference rather than a value capture. The real cost/benefit is small-readability-win versus
one-more-rule-to-teach — which is exactly why the maintainer dispositioned it **R&D** rather than
adopt-or-reject. A secondary open question: whether Mycelium records even admit field names distinct
from the in-scope bindings in the common case (if not, the win shrinks further). (`Declared`.)

### 8.3 Recommended next steps

- **M-846**: confirm the lexical-resolution rule (shorthand RHS = in-scope binding) against the
  S3/S4/G2 invariants; decide adopt vs reject on the readability-win-vs-teaching-cost trade; if
  adopted, specify the rule + its stage-dump form in RFC-0006/RFC-0020 append-only. Default to
  **reject** if the win does not clearly exceed the added rule (YAGNI / KISS, CLAUDE.md house-rule 5).
- Tag stays `Declared` until ratified; no surface form ships that hides which binding `{x}` captures.

## 9. Honest-uncertainty register

- **All proposals in this record are `Declared`** — design directions for maintainer
  consideration, not accepted designs. No claim upgrades past its checked basis.
- **The OQ-R "never aggregate" tension** has a clear resolution path (clarify the two
  layers) but that resolution is `Declared` pending a formal DN or RFC amendment — it is
  not discharged by this record.
- **The OQ-T "one name per term" sub-question for library-defined names** has competing
  considerations of roughly equal weight; the recommended position (convention for phyla,
  mandate for keywords) is a judgment call, not a checked result. `Declared`.
- **OQ-B (forage/backbone activation)** depends on the R1 completion gate (M-667) and on
  several open FLAG items (FLAG-2, FLAG-15 from DN-63). The mechanized-capture design has
  no prior-art precedent in the corpus (SelectionPolicy record-and-replay at this level is
  novel). The analysis here is `Declared` direction.
- **OQ-G guard elaboration** follows the Maranget framework and VR-5/meet rule cleanly; the
  `Declared`-guard-degrades-arm result is the meet rule applied, which has an `Exact` basis.
  The specific elaboration rules (split into guard-true/fallthrough, usefulness update) are
  `Declared` pending an RFC-0020 amendment.
- **OQ-I (short keyword scope)** is clear: the decision (type literals only) is grounded in
  DN-03 §3 and RFC-0019 §8. FLAG-3 resolution (reserved vs. soft keyword) is `Declared`
  pending M-834.
- **OQ-J (ergonomic wrappers)** has three candidate mechanisms, all `Declared`, with the
  RFC-0012 ambient paradigm as the clearest prior art. Stage 1b (grade inference) is the
  long-term answer; M-835 covers interim ergonomics only.

---

## Meta — changelog

| Date | Change |
|---|---|
| 2026-06-29 | Created. R&D planning record for DN-64 §7 OQ-B/G/I/J/R/T maintainer dispositions, tasks M-828/833/834/835/843/845. Six sections covering forage/backbone activation and mechanized SelectionPolicy capture; guard clauses and guarantee propagation via the meet rule; short-keyword scope limited to type literals; ergonomic wrappers for stage-1a grade annotation burden; composite-operation guarantee aggregation clarification; and proposal-time three-test gate extension. All proposals `Declared`. Append-only. |
| 2026-06-29 | Added §8 OQ-H (record-literal shorthand shadowing, M-846) — maintainer R&D disposition supplied after the initial set; lexical-resolution rule surveyed against S3/S4/G2; default-reject on YAGNI/KISS unless the win is clear. Register §8→§9. Append-only. |
