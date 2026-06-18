# RFC-0018 — Stage-1 Static Guarantee Grading

| Field | Value |
|---|---|
| **RFC** | 0018 |
| **Status** | **Draft** (revision of RFC-0007 §4.3; flagged novel — graded modality + runtime certificates has no found precedent; maintainer ratification required; do NOT read as Accepted) |
| **Type** | Foundational / normative (language type system) |
| **Date** | June 18, 2026 |
| **Depends on** | RFC-0006 §4.1 S2, §4.2 LR-6, §8 Q3, §10; RFC-0007 §4.3/§4.4 (guarantee index `τ @ g`, stage-0 dynamic check); research/03-language-layer-RECORD.md T3.2; VR-5; KC-3 |
| **Revises** | RFC-0007 §4.3 (stage-0 dynamic check only) — that section's v0 semantics remain in force until this RFC is Accepted |
| **Coupled with** | RFC-0002 (Swap certificates); RFC-0005 (reified selection); `crates/mycelium-l1` (the non-normative prototype, once this RFC lands) |

## 1. Summary

RFC-0007 §4.3 ships a **stage-0** guarantee index — the lattice tag `g ∈ {Exact, Proven,
Empirical, Declared}` is checked **dynamically** against the `Meta` field at runtime; a static
graded judgment was deferred with one rule already fixed (annotation may only weaken, VR-5).
RFC-0006 §8 Q3 named the mechanism — a **graded coeffect modality over the guarantee
meet-semilattice** — and left one load-bearing normative decision open: *whether implicit flows
taint* (the `pc` question). RFC-0007 §10 explicitly defers stage-1 static grading as a later
revision of that RFC.

This RFC is that revision. It specifies:

1. the **graded typing judgment** `Γ ⊢ e : τ @ g` — the guarantee lattice as a static type-level
   index, with meet as composition law and `Swap` as the sole controlled-endorsement point;
2. the **staged implementation path** (T3.2): stage 0 (done) → monomorphic grades → bounded grade
   polymorphism → refinement premises for certificate side-conditions;
3. the **two designs for implicit-flow taint** — fully laid out, with a recommendation — as a
   required maintainer decision (RFC-0006 Q3); and
4. a **research prompt** (§11) that must run before ratification, producing the noninterference
   proof obligation for this 4-point integrity lattice + runtime-certificate endorsement model.

**Novelty flag (VR-5).** The combination of lattice grading with machine-checkable per-value
runtime certificates has no found precedent (T3.2). This RFC therefore carries its own soundness
argument obligation (§11 research prompt) and is marked Draft pending that work — downgrading
honesty is always permitted; upgrading without a checked basis is not.

## 2. Motivation

The guarantee lattice (`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`) is part of the observable
interface of every binding (S2). In stage 0, violations are **silent until runtime**: a function
claiming to return `Exact` can receive `Declared` input and produce a result tagged `Exact` by
*annotation alone* — the runtime `Meta` check fires only when the value is inspected, and only if
the caller checks. The honesty rule (VR-5) says annotation may only weaken; but without a static
judgment, the compiler cannot enforce this for compound expressions.

The practical failure mode: a composition `f(g(x))` where `f` is annotated `Exact → Exact` and
`g` returns `Declared` produces a result whose tag should be `Declared ∧ Exact = Declared` by
meet — but stage-0 permits it to carry `Exact` until someone inspects the Meta field. This is a
silent violation of the honesty rule that the type system should reject at compile time.

Moving guarantee grading into the type system (LR-6) turns the honesty rule from an auditable
runtime property into a structural invariant enforced by the checker — the same move that gave
languages type-safe memory (from runtime null-checks to non-nullable types). The soundness story,
borrowed from information-flow control (IFC) literature (T3.2), is **DCC-style noninterference
read in the integrity direction**: no `Declared` input may influence an `Exact`-typed output
except through a certified `Swap` node (the endorsement point).

## 3. Guide-level explanation

### 3.1 The guarantee lattice as a grade

The four-point integrity lattice, ordered by trust (highest first):

```
  Exact
    |
  Proven
    |
  Empirical
    |
  Declared
```

The partial order: `Exact ⊒ Proven ⊒ Empirical ⊒ Declared` (higher = more trusted). This is an
integrity lattice in Biba's sense (1977, T3.2): information flows upward only through a certified
endorsement, never silently. The **meet** `g₁ ∧ g₂` gives the least upper bound in the *reverse*
order — equivalently the greatest lower bound in the trust order — i.e. `Proven ∧ Empirical =
Empirical` (the weaker grade wins). This is composition's pessimistic rule: a computation whose
inputs have mixed guarantees carries the weakest.

A **graded type** is `τ @ g`: a value of representation type `τ` whose guarantee is at least `g`.
The grade is a static upper bound on trust — `τ @ Proven` means "at least `Proven`"; asserting
`@ Exact` on a `Declared` value is ill-typed. (VR-5: annotation may only weaken. Stage-0's one
fixed rule carries forward verbatim.)

### 3.2 The graded judgment at a glance

Extend RFC-0007 §4.4's `Γ ⊢ e : τ` to `Γ ⊢ e : τ @ g`:

- **Variables and constants:** inherit the grade declared in `Γ`.
- **Application and let-binding:** the result grade is the **meet** of all input grades.
- **`Swap` (the endorsement point):** the only node allowed to *raise* a grade — it carries a
  certificate that the checker validates; without a valid certificate, a `Swap` may only output
  a grade ≤ its input grade.
- **`Match` / branching:** the result grade is the meet of the scrutinee's grade and all branch
  body grades (see §4.5 for the full design decision and the R7-Q2 treatment).
- **Annotation (`e @ g`):** if the inferred grade `g'` satisfies `g' ⊒ g` (i.e. the inferred
  grade is at least as trusted as the annotation claims), the annotation is accepted; otherwise
  it is a type error. This is VR-5 as a typing rule.

A function demanding `Exact` input is expressed as a function whose parameter type is `τ @ Exact`;
calling it with a `Declared`-graded value is a static type error — the honesty rule at the call
site, enforced by the checker.

### 3.3 `Swap` as controlled endorsement

In IFC literature, raising a label (declassification / endorsement) is a privileged operation.
Mycelium's analogue is a `Swap` node carrying a **certificate** (RFC-0002): the certificate
*is* the endorsement proof. This gives a stronger guarantee than Jif's `declassify` (which
requires only a trust context): Mycelium endorsements require **machine-checkable evidence** that
the conversion meets its stated accuracy bound (VR-5). The graded typing rule for `Swap` reflects
this: the output grade is determined by the certificate's attested grade, not asserted freely.

The key invariant: **every grade increase is lexically visible** (an explicit `Swap` node with an
explicit certificate), and no elaboration step may introduce a `Swap` (S1 / W8). The honesty
rule is therefore not just a type property — it is also a *syntactic* property of well-formed
terms.

## 4. Reference-level design (normative once Accepted)

### 4.1 The guarantee semilattice

```
G = { Exact, Proven, Empirical, Declared }
```

with meet `∧ : G × G → G` defined by the total order `Exact > Proven > Empirical > Declared`:

```
g₁ ∧ g₂  =  the lesser of g₁, g₂ in the trust order
```

Concretely: `Exact ∧ Proven = Proven`, `Proven ∧ Empirical = Empirical`,
`Empirical ∧ Declared = Declared`, and `g ∧ g = g`. (Meet is the binary operation in the
meet-semilattice `(G, ∧, Exact)` where `Exact` is the top element — the strongest grade.) This
semilattice structure is what qualifies `G` as a grade algebra for a graded coeffect system
(Orchard et al. ICFP 2019, Granule; T3.2): the composition `e₁ ; e₂` carries grade `g₁ ∧ g₂`.

### 4.2 Graded types and graded contexts

A **graded type** `τ @ g` annotates a value type with a guarantee grade. The grade is covariant
in the trust order: `τ @ Exact` is a subtype of `τ @ Proven` (a more-trusted value satisfies a
less-trusted demand). Formally, the subtyping/subsumption rule:

```
  g' ⊒ g
  ──────────────────    (G-Sub)
  τ @ g'  ≤  τ @ g
```

A **graded context** `Γ` assigns each variable a graded type: `Γ = x₁:τ₁ @ g₁, …, xₙ:τₙ @ gₙ`.
The **context grade** `|Γ|` is the meet of all grades in scope: `|Γ| = g₁ ∧ … ∧ gₙ` (for the
implicit-flow design below).

### 4.3 The graded typing judgment

The full judgment form: `Γ ; pc ⊢ e : τ @ g`

where `pc` is the **program-counter grade** (the implicit-flow index — see §4.5 for the design
decision). In the **data-lineage-only design** (Design A, §4.5), `pc = Exact` is invariant and
dropped from the notation, reducing to `Γ ⊢ e : τ @ g`. In the **`pc`-indexed design** (Design
B, §4.5), `pc` tracks the grade of the branching condition. The rules below are stated with the
`pc` slot explicit; a maintainer decision on §4.5 will fix which form is normative.

```
  G-Const   Σ ⊢ v wf    g ≤ grade(v.Meta)
            ──────────────────────────────────
            Γ ; pc ⊢ Const(v) : ReprTy(v.repr) @ grade(v.Meta)

  G-Var     (x : τ @ g) ∈ Γ
            ───────────────────
            Γ ; pc ⊢ x : τ @ g

  G-Weaken  Γ ; pc ⊢ e : τ @ g'    g' ⊒ g
            ───────────────────────────────    (annotation weakens, VR-5)
            Γ ; pc ⊢ (e @ g) : τ @ g

  G-Let     Γ ; pc ⊢ e₁ : τ₁ @ g₁
            Γ, x:τ₁ @ g₁ ; pc ⊢ e₂ : τ₂ @ g₂
            ─────────────────────────────────────
            Γ ; pc ⊢ Let(x, e₁, e₂) : τ₂ @ (g₁ ∧ g₂)

  G-Lam     Γ, x:τ₁ @ g₁ ; pc ⊢ e : τ₂ @ g₂
            ──────────────────────────────────────
            Γ ; pc ⊢ Lam(x, τ₁ @ g₁, e) : (τ₁ @ g₁ → τ₂ @ g₂)

  G-App     Γ ; pc ⊢ f : (τ₁ @ g₁ → τ₂ @ g₂)
            Γ ; pc ⊢ a : τ₁ @ g₁'    g₁' ⊒ g₁
            ──────────────────────────────────────    (G-Sub applied to argument)
            Γ ; pc ⊢ App(f, a) : τ₂ @ g₂

  G-Op      Π(p) = (τ₁ @ g₁…τₙ @ gₙ) → τ_r @ g_r
            Γ ; pc ⊢ eᵢ : τᵢ @ gᵢ'    gᵢ' ⊒ gᵢ  (for each i)
            ─────────────────────────────────────────
            Γ ; pc ⊢ Op(p, e₁…eₙ) : τ_r @ g_r

  G-Swap    Γ ; pc ⊢ e : ReprTy(r₁) @ g_in
            cert : SwapCert(r₁ → r₂, g_out, proof_ref)    g_out ⊑ g_in ∨ cert.valid
            ──────────────────────────────────────────────────────────────────────────
            Γ ; pc ⊢ Swap(e, r₂, cert) : ReprTy(r₂) @ g_out

  G-Con     Σ(#T#i) = C(τ₁ @ g₁ … τₙ @ gₙ)
            Γ ; pc ⊢ eⱼ : τⱼ @ gⱼ'    gⱼ' ⊒ gⱼ  (for each j)
            ─────────────────────────────────────────────────────
            Γ ; pc ⊢ Construct(#T#i, e₁…eₙ) : T @ (g₁ ∧ … ∧ gₙ)

  G-Fix     Γ, f:τ @ g ; pc ⊢ e : τ @ g
            ──────────────────────────────
            Γ ; pc ⊢ Fix(f, τ @ g, e) : τ @ g
```

Notes:
- **G-Const** reads the grade from the value's `Meta` field (the stage-0 runtime tag); this is
  the bridge between stage 0 and stage 1 — a constant's static grade is its runtime grade.
- **G-Op** uses the prim signature table `Π` (RFC-0007 §4.4). The graded prim table upgrade (§4.6
  below) assigns grades to prim arguments/results; until that upgrade, all prim arguments and
  results are treated as grade-preserving (output grade = meet of input grades).
- **G-Swap** is the only rule that may raise a grade: when `cert.valid` holds (the certificate's
  proof obligation is discharged), `g_out` may exceed `g_in`. Without a valid certificate, `g_out`
  is at most `g_in` — a `Swap` without a certificate may only weaken or preserve the grade, not
  strengthen it. This is the static analogue of VR-5's runtime enforcement.
- **G-Match** is stated in §4.5 because it is the load-bearing design decision (R7-Q2).

### 4.4 The `Match` rule and R7-Q2 — the load-bearing open decision

RFC-0007 §8 records **R7-Q2**: does a `Match` default arm's body meet-degrade differently from
named alternatives once stage-1 grading lands? This connects directly to RFC-0006 Q3's open
implicit-flows question. The answer depends on the `pc` design (§4.5), which this RFC presents as
a required maintainer decision. Section §4.5 states both designs fully, with the `Match` rule
under each.

Here the structure of `G-Match` is fixed independent of the `pc` choice: the **result grade** is
at minimum the meet of all branch body grades. What varies between Design A and B is whether the
scrutinee's grade additionally degrades the result via a `pc`-like mechanism.

### 4.5 The implicit-flows decision — the required maintainer decision

**Background.** In information-flow control, a *direct flow* is `x := y` — the output obviously
depends on the input's label. An *implicit flow* is:

```
if (secret == 0) then result := 0
                 else result := 1
```

Here `result` reveals one bit of `secret` even though no direct assignment copies the label. A
sound IFC system must account for this via a `pc` (program counter) label: the branches of a
conditional inherit at least the `pc ∧ condition_grade` label, so the result cannot be more
trusted than the condition's label. Read in the integrity direction: if the condition comes from
`Declared` data, the branch bodies — and the result — are degraded to `Declared`.

For Mycelium, the question is: **does branching on a `Declared`-graded scrutinee degrade the
result grade of the `Match`?**

This is a genuine normative design question with no mechanical answer — it is a policy decision
about what the guarantee system is *for*. The two designs are:

---

**Design A — Data lineage only (no implicit-flow taint)**

The `pc` slot is always `Exact` and is erased from the notation. The grade of a `Match` result
is the meet of all branch *body* grades; the scrutinee's grade does not appear in the result's
grade. The `G-Match` rule:

```
  G-Match/A   Γ ⊢ s : T @ g_s
              (for each named alt) Γ, x̄:fields(#T#i) @ g_fields ; ⊢ eᵢ : τ @ gᵢ
              (default, if any)    Γ ⊢ d : τ @ g_d
              coverage: alts ∪ default ⊇ ctors(T)    (W7)
              ─────────────────────────────────────────────────────────────────────
              Γ ⊢ Match(s, alts, default?) : τ @ (g₁ ∧ … ∧ gₙ ∧ g_d)
```

The scrutinee's grade `g_s` does not appear in the result grade. Named alternatives and the
default arm are treated **identically** — the meet is taken uniformly over all branch bodies.
R7-Q2 answer under Design A: no, the default arm does not meet-degrade differently; all arms
contribute equally to the meet.

*Costs of Design A.*
- Imprecise: a function that dispatches on a `Declared` flag and always returns a constant
  `Exact` value will be typed `Exact` — which is correct for data lineage but may surprise a
  user who expects "any computation touching `Declared` data produces `Declared` output."
- Simpler: no `pc` annotation burden; the type system is a standard graded coeffect system
  without program-counter complexity.
- Precedent: this is the graded-coeffect literature's standard position (Orchard et al. 2019;
  Granule security grades). Security types without a `pc` are a recognized legitimate design
  for systems tracking *data* provenance rather than *observation* secrecy.

**Design B — `pc`-indexed (implicit flows taint)**

A `pc` grade flows through the program counter, degraded at every conditional. The `G-Match` rule:

```
  G-Match/B   Γ ; pc ⊢ s : T @ g_s
              let pc' = pc ∧ g_s
              (for each named alt) Γ, x̄:fields(#T#i) @ g_fields ; pc' ⊢ eᵢ : τ @ gᵢ
              (default, if any)    Γ ; pc' ⊢ d : τ @ g_d
              coverage: alts ∪ default ⊇ ctors(T)    (W7)
              ─────────────────────────────────────────────────────────────────────────
              Γ ; pc ⊢ Match(s, alts, default?) : τ @ (pc' ∧ g₁ ∧ … ∧ gₙ ∧ g_d)
```

The scrutinee's grade `g_s` is folded into `pc'`, which degrades both the branches and the
result. R7-Q2 answer under Design B: no, the default arm does not meet-degrade *differently*
from named alternatives — both run under `pc'`. The `pc`-update is symmetric across all arms,
named and default. This resolves R7-Q2 unambiguously under either design: the default arm
carries no special degradation status.

*Costs of Design B.*
- Annotation burden: any dispatch on a `Declared` value propagates `Declared` to every result
  in scope of that branch, even if the results are independent constants. Programmers working
  with mixed-grade data would need frequent `Swap` endorsements to recover precision — the
  typical complaint against FlowCaml in practice (Pottier & Simonet, TOPLAS 2003).
- False degradation: `if (declared_flag) { pure_computation_a } else { pure_computation_b }`
  would type-check only if both branches return `Declared`-graded results, even though neither
  uses the flag's *value* inside the branch.
- Precedent: every sound IFC system with implicit-flow protection uses a `pc` index (Volpano–
  Smith–Irvine 1996; Myers POPL 1999; FlowCaml; DCC). The guarantee system is an integrity
  lattice; a sound integrity-noninterference statement technically requires the `pc` treatment.

*Comparison table.*

| Property | Design A (data lineage) | Design B (`pc`-indexed) |
|---|---|---|
| Implicit flows tracked | No | Yes |
| `pc` annotation burden | None | Present at every branch |
| False degradation on pure branches | No | Yes — common |
| IFC noninterference (full) | Only data flow | Full (data + control) |
| Annotation overhead for typical code | Low | Moderate–high without inference |
| Inference feasibility | Near-trivial over 4-chain | Feasible (FlowCaml precedent); more constraints |
| Conceptual model | "What data went into this result?" | "Could this result have been influenced (directly or via control) by low-grade data?" |
| R7-Q2 (default arm) | Same as named alts | Same as named alts (both run under `pc'`) |

**Recommendation (advisory; maintainer decides).**

The recommendation is **Design A (data lineage only)**, for three reasons grounded in the
project's specific context:

1. **The guarantee system tracks provenance, not secrecy.** The honesty rule (VR-5) is about
   *what evidence* backs a result, not about *information leakage* in the confidentiality sense.
   A computation that dispatches on a `Declared` flag but returns a constant derived entirely from
   `Exact` inputs *should* be typed `Exact` — the result's provenance is `Exact`, even though a
   branch condition touched `Declared` data. Design B would mistype this as `Declared`, which is
   not a more honest answer — it is a *less accurate* one for the intended semantics.

2. **Annotation burden is a usability cost with no corresponding honesty gain here.** S2
   requires guarantee tags to be part of the observable interface; it does not require the system
   to track implicit flows. DN-09's KC-2 verdict emphasizes usability (familiar surface,
   dual human/LLM rendering); a `pc` annotation system substantially raises the surface area of
   graded types. Per KC-3, the kernel stays small and auditable — implicit-flow tracking requires
   a `pc` register throughout the typing rules, increasing the trusted-checker complexity for a
   benefit that is not needed for the honesty guarantee.

3. **Design A is still sound for the stated property.** The noninterference property for Design
   A is: *no `Declared`-graded **data** input may flow into an `Exact`-typed output except through
   a certified `Swap`.* This is the integrity property Mycelium's VR-5 actually asserts. It is a
   strictly weaker property than full IFC noninterference, but it is the property the honesty rule
   requires — and stating it precisely is more honest than claiming full IFC noninterference when
   Design B's annotation burden would cause users to annotate around it with spurious endorsements.

**This recommendation is not a ratification.** The choice between Design A and Design B is a
normative decision about the semantics of the guarantee system. It must be recorded as an explicit
maintainer decision (append-only; RFC-0006 Q3) before this RFC can be ratified. The §11 research
prompt includes a required worked-example pass that tests both designs on representative programs.

### 4.6 Interaction with LR-6 guarantee-indexed types and the prim signature table

**LR-6 (RFC-0006 §4.2):** "a function demanding `Exact` input is a type error to call with
`Declared`" — this is exactly the G-App rule's `g₁' ⊒ g₁` premise. The graded typing judgment
*is* the LR-6 mechanism; the two are not separate. The grade on an `Arrow` type is a demand
annotation; meeting it is checked by G-App at call sites.

**The prim signature table `Π` (RFC-0007 §4.4, R7-Q4):** The v0 prim table is ungraded — prims
treat all inputs as grade-preserving and the output grade is the meet of the inputs (the
conservative default in G-Op). Upgrading the prim table to carry graded signatures is a separate
step, deferrable but important:

- Prims that are exact by construction (e.g., `Op(and, x, y)` on `Binary{n}` values, where the
  bitwise operation has no approximation) should be gradable as `(τ @ g, τ @ g) → τ @ g` —
  they preserve grade.
- Prims that introduce approximation (e.g., a density-to-binary threshold op) should be gradable
  as `(τ @ g₁, τ @ g₂) → τ @ (g₁ ∧ g₂ ∧ Empirical)` — the prim itself contributes an
  `Empirical` floor.
- A prim with an explicit `Proven` certificate (the certificate is the endorsement — RFC-0002
  style) should be gradable as `… → τ @ Proven`.

This upgrade is the natural evolution of R7-Q4 (the prim table as content-addressed declarations)
into a graded form. It is not required for stage-1 correctness — the conservative G-Op default is
sound (it can only over-degrade, never under-degrade) — but it is required for the system to be
*precise* for common prim-heavy programs. The prim-table grading upgrade is tracked as a
follow-on from this RFC; ratifying RFC-0018 does not require it.

### 4.7 Staging path (T3.2)

The staged implementation follows the T3.2 position, with stage 0 already shipped:

| Stage | Description | Status |
|---|---|---|
| **0** | Runtime tags: grade stored in `Meta`, checked dynamically; annotation may only weaken (VR-5 at runtime) | Done (RFC-0007 §4.3) |
| **1a** | Static grades, monomorphic: the graded judgment `Γ ⊢ e : τ @ g` (this RFC, §4.3); checker enforces G-Weaken at compile time; programs without grade-polymorphic abstractions typecheck fully statically | This RFC |
| **1b** | Bounded grade polymorphism: `∀(g : G). τ @ g → τ @ g` — grade variables quantified over `G`, with grade constraints `g₁ ⊒ g₂` as bounds; inference over the 4-chain is near-trivial (the chain has no branching, so constraint solving is linear comparison) — the FlowCaml-style approach (Pottier & Simonet, TOPLAS 2003) ported to a finite 4-chain | Post-ratification |
| **2** | Refinement premises for certificate side-conditions: `Proven` grade gated on a refinement obligation (e.g., `Swap ε ≤ ε_bound ∧ trials ≥ N`); the side-conditions become type-level premises, closing the loop with RFC-0002's certificate structure | Future (own RFC) |

Stage 1a is the deliverable of this RFC. Stages 1b and 2 are future possibilities (§10), each
requiring their own RFC.

**Runtime–static bridge.** The G-Const rule reads the grade from `v.Meta` — this is how stage-0
programs (which may have runtime-graded constants) interact with the stage-1 checker. A constant
whose `Meta` tag is weaker than its static annotation is a type error (VR-5 at compile time, not
just at runtime). For programs without dynamic grades, the checker is entirely static; for programs
that mix static and dynamic grading (via FFI or explicit `Meta` manipulation), the dynamic check
remains the fallback.

## 5. Drawbacks

1. **Annotation cost.** Graded types add a grade annotation to every binding in the graded
   context. In practice, grade inference (stage 1b) will infer most grades; monomorphic stage 1a
   requires explicit grades on function signatures. The net annotation burden depends on the
   implicit-flows decision: Design A is significantly lower-burden than Design B.

2. **Prim table is initially conservative.** Until the prim table is upgraded (§4.6), G-Op
   over-degrades results that prims could compute exactly. This is honest (never too optimistic)
   but can produce spurious `Declared`-or-`Empirical` results for programs that are actually
   `Exact`. The fix is progressive and does not require a re-architecture.

3. **Novelty of the grading + certificate combination.** There is no prior system to reference
   for the interaction between static grades and machine-checkable runtime certificates as the
   endorsement mechanism. The soundness argument is new and must be written from scratch (§11
   research prompt). Ratifying before that work lands would be an unsupported upgrade — VR-5
   applies to the RFC itself.

4. **Staged incompleteness is visible.** Stage 1a without grade polymorphism (1b) means
   some valid programs are rejected or over-annotated; library functions over grade-polymorphic
   data require explicit grade instantiation. This is honest sequencing — the checker never
   accepts an incorrect grade, it may only reject a correct one — but it is a usability cost
   until 1b lands.

## 6. Rationale and alternatives

**Why a graded coeffect modality rather than a type index?** A type index `τ[g]` in a dependent
or refinement type system would require a heavyweight type theory (F* / Liquid Haskell style).
Graded coeffect modalities (Orchard et al. 2019) give the same tracking over a pre-ordered
semiring with a much lighter metatheory — the grade algebra is just a bounded lattice, and the
typing rules are standard (§4.3 above). The 4-chain is the simplest non-trivial case; the
machinery is well-understood for this case (T3.2).

**Why meet as composition and not join?** This is the integrity lattice orientation (Biba 1977,
T3.2). The guarantee system tracks provenance trust, not confidentiality: a result whose inputs
are partially untrusted (`Declared`) must be at most as trustworthy as the least trusted input.
Meet gives the pessimistic composition rule. Join would give an *optimistic* rule ("trust the
most trusted input") — wrong for provenance, correct for confidentiality.

**Why `Swap` as the sole endorsement point?** This follows directly from S1 (never-silent swap)
and W8 (no elaboration step may insert a `Swap`): the grade-raising rule is exactly the same node
that enforces lexical visibility of representation changes. This is not coincidental — a
representation swap and a guarantee endorsement are both attestation acts; conflating them into
one node with one certificate is the minimal, auditable design (KC-3). The alternative — a
separate `endorse` node — would double the trusted surface area for no structural gain.

**Why not full IFC (Design B)?** See §4.5 recommendation. Short version: the property Mycelium
asserts is data provenance, not information-theoretic noninterference. Design B is *sound for a
stronger property* than VR-5 requires, at a usability cost that would drive users toward
spurious endorsements — and spurious endorsements defeat the honesty purpose of the whole system.
Design A's noninterference statement is weaker but honest; Design B's is stronger but practically
self-defeating for this use case.

## 7. Prior art

- **Denning lattice model** (CACM 1976): origin of integrity/confidentiality lattices and the
  meet-composition rule; the formal basis for the guarantee lattice's integrity orientation (T3.2).
- **Volpano–Smith–Irvine** (J. Computer Security 1996): first type-system proof of
  noninterference; introduced the `pc` label; direct precedent for Design B.
- **JFlow / Jif** (Myers, POPL 1999): label polymorphism and declassification via privileged
  operations; Mycelium's `Swap`-as-endorsement is analogous to Jif's `declassify`.
- **FlowCaml** (Pottier & Simonet, POPL 2002; TOPLAS 2003): constraint-based label inference over
  an ML type system with lattice labels; the direct model for stage-1b grade inference over the
  4-chain. Pottier & Simonet document the annotation-burden problem under `pc` tracking in
  practice — grounds the Design A recommendation.
- **Dependency Core Calculus (DCC)** (Abadi, Banerjee, Heintze, Riecke, POPL 1999): modal
  type-and-effect system for tracking computational dependencies; extended by Algehed (PLAS 2018)
  and Choudhury et al. (ESOP 2022) to modern settings. The soundness story for this RFC is
  DCC-style noninterference read in the integrity direction.
- **Granule** (Orchard, Liepelt & Eades, ICFP 2019): graded coeffect system over semirings,
  including a `Level` security grade; the direct technical precedent for the graded-modality
  machinery (T3.2). Granule is a research prototype; production deployment is not established.
- **Biba integrity model** (1977): the integrity-lattice dual of Bell-LaPadula (confidentiality);
  establishes that "weakest wins" (meet) is the correct composition rule for integrity/trust
  labels, and that raising a label requires a trusted endorsement. This RFC reads the guarantee
  lattice as a Biba integrity lattice (T3.2).
- **LWeb** (POPL 2019): labels-via-refinements; shows the refinement approach is feasible but
  heavyweight — grounds the decision to use graded coeffects for the 4-point tag and reserve
  refinements for certificate side-conditions (stage 2 only, §4.7).
- **Flagged as novel (T3.2):** no prior system was found combining (a) lattice grading over a
  typed term language with (b) machine-checkable per-value runtime certificates as the
  endorsement proof. The interaction between static grades and the certificate-validity premise
  in G-Swap is therefore a novel contribution requiring an independent soundness argument (§11).

## 8. Unresolved questions

1. **R18-Q1 (the implicit-flows decision — the required maintainer decision).** Choose Design A
   (data lineage only, no `pc`) or Design B (`pc`-indexed, full implicit-flow taint). This RFC
   recommends Design A (§4.5); the choice is the maintainer's and must be recorded as an explicit
   append-only decision before ratification. Neither design is implicit; silence is not an answer.

2. **R18-Q2 (R7-Q2 discharged contingently).** R7-Q2 asks whether the `Match` default arm
   degrades differently from named alternatives. §4.5 answers it for both designs: under both,
   the default arm is treated symmetrically with named alternatives (no special degradation).
   This resolution is contingent on the Design A/B choice — whichever design is adopted, R7-Q2
   is closed by the corresponding G-Match rule. The maintainer decision on R18-Q1 closes R7-Q2.

3. **R18-Q3 (prim table grading schedule).** When does the prim signature table upgrade (§4.6)
   land relative to stage-1 ratification? The conservative G-Op default is sound but imprecise;
   the upgrade is a separate tracked deliverable. This RFC does not gate ratification on it, but
   the decision about when to require it should be recorded.

4. **R18-Q4 (certificate validity checking).** The G-Swap rule has a `cert.valid` premise. In
   stage 1a, certificate validity is checked by the *existing* RFC-0002 certificate checker —
   the same checker that enforces VR-5 at runtime. The question is whether stage-1 static
   checking requires the certificate to be fully elaborated and validated *at type-checking time*
   (strong, but requires the proof checker to run during type checking), or whether a certificate
   *reference* is sufficient at the type level with validation deferred to elaboration/runtime
   (weaker, but separates concerns and keeps the type checker decidable without the proof
   checker). This is a trusted-kernel scope question (KC-3).

5. **R18-Q5 (grade inference scope for stage 1a).** Stage 1a is monomorphic (§4.7); grade
   inference is therefore only local (within a single expression). The question is whether even
   monomorphic inference (filling in grades from context, like type inference fills in types) is
   in scope for stage 1a, or whether stage 1a requires explicit grade annotations everywhere and
   inference arrives only with 1b. Inference in stage 1a is desirable for usability; it is
   technically straightforward over the 4-chain; but it expands the checker's scope. A decision
   is needed before implementation.

## 9. Future possibilities

- **Stage 1b grade polymorphism.** Grade-polymorphic functions (`∀g. τ @ g → τ @ g`), with
  FlowCaml-style constraint-based inference over the 4-chain. The 4-chain's total order makes
  constraint solving linear; no lattice-join ambiguity is possible. This gives guarantee-
  polymorphic libraries ("works at whatever strength your inputs have, returns the meet") as
  noted in RFC-0006 §9.
- **Stage 2 refinement premises.** `Proven`-graded `Swap` certificates carry refinement
  obligations (e.g. `ε ≤ ε_bound ∧ trials ≥ N`) as type-level premises. These are checked by
  an SMT backend or explicit proof terms, closing the loop with RFC-0002's certificate structure.
  The LWeb (POPL 2019) architecture is a reference for this level.
- **Certified-swap combinators as first-class values.** A `Swap` with a certificate could be
  reified as a first-class value `Cert(r₁ → r₂, g)`, composable with other certificates — the
  natural extension of RFC-0005's reified selection to the graded setting.
- **Grade-polymorphic standard library.** RFC-0016's standard library functions typed with grade
  polymorphism once stage 1b lands, so that library functions automatically preserve the grade
  of their inputs without requiring duplicate definitions at each grade level.
- **Interaction with effects (RFC-0006 Q4 position).** The divergence bit (RFC-0007 §4.5) is
  orthogonal to grades; a `partial` function's grade is still tracked by the graded judgment. If
  a full effect system is added later (the growth path in RFC-0006 §8 Q4), grades and effects
  combine naturally in a graded effect system (the Koka + Granule intersection; Orchard & Petricek
  2014 as the theoretical basis).

## 10. Ratification scope

This RFC is **Draft** and must not be read as Accepted. Before it can be ratified, the following
must be discharged:

1. **R18-Q1 (the implicit-flows decision):** an explicit, append-only maintainer decision recorded
   in this RFC's Meta-changelog, choosing Design A or Design B and updating §4.5's normative rule
   to remove the conditional framing.
2. **The §11 research prompt:** the variant pass must complete and its findings must be integrated.
   Specifically, the noninterference proof obligation (§11 item 1) must be stated as a theorem
   with side-conditions (even if not yet machine-checked), and the worked-example set (§11 item 3)
   must exercise the implicit-flows decision with concrete programs to ground the Design A/B
   recommendation.
3. **R18-Q4 (certificate validity checking scope):** a decision on whether `cert.valid` is a
   type-checking-time obligation or an elaboration/runtime one, so the checker's trusted scope
   can be defined (KC-3).

Items not required for ratification: the prim table grading upgrade (§4.6 — the conservative
G-Op default is sound), stage 1b polymorphism (§4.7 — stage 1a is the deliverable), and stage 2
refinement premises (§4.7 — a future RFC).

**Status line (in force):** *Draft — graded typing judgment (§4.3), staging path (§4.7), and
prior art (§7) are complete; the implicit-flows maintainer decision (R18-Q1), the soundness
argument (§11 research prompt), and the certificate-validity scope (R18-Q4) remain open and must
be discharged before ratification.*

## 11. Research prompt (pre-ratification variant pass)

> **This block is a required pre-ratification deliverable, not a normative part of this RFC.**
> It must be executed as a separate research pass before the maintainer ratifies this RFC. The
> findings feed back into §4.5 (the Design A/B choice), §8 (resolving or narrowing R18-Q1 through
> R18-Q5), and the soundness argument in §10. The prompt is stated precisely so the variant pass
> can be executed mechanically; adjust scope only with an explicit changelog entry.

**Prompt for the variant pass:**

Execute a deep-research pass on the following three items, cite primary sources, flag
unverified claims explicitly (honesty rule), and deliver a cited report that:

**Item 1 — Graded-modal-type soundness for this specific lattice and endorsement model.**

Survey the graded-modal-type soundness literature as it applies to **this system**: a 4-point
totally-ordered integrity lattice `(G, ∧, Exact)` with a graded coeffect modality, where the
sole grade-raising operation is a `Swap` node carrying a machine-checkable certificate. Targets:

- Abel's graded modalities (Abel & Bernardy, ICFP 2020: "A generalized modality for recursion");
  Bernardy et al. (POPL 2018 Linear Haskell) for graded linear-type soundness;
  Choudhury et al. (ESOP 2022) for the DCC extension. Does any of these directly yield a
  noninterference theorem for a finite integrity lattice?
- The DCC (Abadi et al. POPL 1999) noninterference theorem: can it be read in the integrity
  direction (not confidentiality) for a 4-point chain? What are the side-conditions?
- **Produce the noninterference proof obligation for this specific system:** state the theorem
  "no `Declared`-graded data input influences an `Exact`-typed output except through a certified
  `Swap`" as a formal statement with all side-conditions explicit. Identify any gaps between the
  cited literature's theorems and what this system requires.

**Item 2 — Settlement of the implicit-flows question with literature evidence.**

Survey the practical experience with `pc`-indexed IFC systems (FlowCaml, Jif, JIF successor
systems) for evidence on:

- Annotation burden in practice: what fraction of annotations in real FlowCaml/Jif programs are
  `pc`-driven endorsements (i.e., endorsements caused by branching on a sensitive value, not
  by actual data-flow declassification)?
- Are there published defenses of data-lineage-only (no `pc`) designs for integrity-flavored
  type systems (as opposed to confidentiality)? Cite.
- Does any graded-coeffect system in the literature (Granule; linear types with security grades)
  include a `pc` label? If not, what is the stated reason?

**Item 3 — Worked-example set for both designs.**

Construct at least five representative Mycelium-style programs (using the notation of §4.3) that
exercise the implicit-flows question and run them through both Design A (G-Match/A) and Design B
(G-Match/B):

- A dispatch on a `Declared` flag returning a constant `Exact` value in both branches.
- A dispatch on an `Empirical` grade value that produces `Exact` output in all branches.
- A function that aggregates a list of mixed-grade inputs using `for` (§4.8 / RFC-0007 §4.8).
- A `Swap`-endorsed conversion inside one branch of a match — does Design B require an extra
  endorsement to recover the pre-match grade?
- A mutual-recursive `Fix` group where one member branches on a `Declared`-graded value.

For each: show the inferred grade under Design A and Design B; identify cases where Design B
degrades a result that Design A would leave at a higher grade; and assess whether the degradation
is "correct" in the sense of VR-5 (does the result's provenance actually depend on the
`Declared` input?) or is a false positive.

**Deliverable format:** a cited research record in `research/` (follow the T3.x format), tagged
as the grounding for §4.5's maintainer decision. The record must explicitly state which
literature results are verified from primary sources and which are inferred. The noninterference
proof obligation (Item 1) must be stated as a self-contained theorem, even if not yet
machine-checked, so it can be tracked as a proof obligation.

## Meta — changelog

- **2026-06-18 — Draft filed.** Initial draft: graded typing judgment (§4.3/§4.4), the
  implicit-flows design decision (§4.5, both designs stated with full G-Match rules and
  recommendation for Design A), staging path T3.2 (§4.7), LR-6/prim-table interaction (§4.6),
  soundness argument obligation and research prompt (§11). Revises RFC-0007 §4.3
  (stage-0 dynamic check remains in force; this draft supersedes only if Accepted). Flagged novel
  per T3.2; ratification gated on the maintainer's Design A/B decision (R18-Q1), the §11 variant
  pass, and R18-Q4. RFC-0006 Q3 / RFC-0007 §8 R7-Q2 are discharged contingently on R18-Q1.
