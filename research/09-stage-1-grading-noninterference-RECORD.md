# Research Record 09 — Stage-1 Guarantee Grading: the noninterference argument (RFC-0018 / RP-2)

> **What this file is.** A durable record discharging the **RP-2** research prompt
> (`docs/notes/research-prompts.md`) and the **RFC-0018 §11** pre-ratification proof obligation:
> the implicit-flows design decision and the noninterference argument for stage-1 static guarantee
> grading. Conducted 2026-06-18 from the RFC-0018 draft, RFC-0006 §8 Q3, RFC-0007 §4.3, the
> `mycelium-core::guarantee` implementation, and the IFC/graded-coeffect literature already cited in
> `research/03` (T3.2). Findings are labeled **T9.1–T9.8** (continuing the T0–T8 scheme) and map onto
> RFC-0018 §4.5 (the Design A/B decision), §8 (R18-Q1…R18-Q5), and §11 (items 1–3).
>
> **Posture (honesty rule / VR-5).** This record produces a **stated theorem with a hand-constructed
> proof sketch**, grounded in cited prior art. It is **not machine-checked**; per VR-5 the soundness
> *claim* is therefore tagged **Declared-with-argument**, never **Proven** — mechanization (in the
> Lean/LiquidHaskell track) is the basis a future `Proven` upgrade would require, and is named as
> follow-up. The Design A vs B *decision* (R18-Q1) is a maintainer call; this record **recommends**
> Design A with grounds, it does not ratify. Append-only.

---

## 1. Scope

RFC-0018 ships a complete graded typing judgment (§4.3) with both implicit-flows designs stated in
full (§4.5: Design A = data-lineage-only, Design B = `pc`-indexed) and a recommendation of Design A.
Three things gate ratification (§10): (1) the R18-Q1 maintainer decision, (2) the §11 noninterference
argument, (3) the R18-Q4 certificate-validity scope. This record discharges **(2)** and supplies the
**grounds + recommendation** for (1) and (3). It delivers the three §11 / RP-2 items:

- **Item a** — the counterexample that distinguishes Design A from Design B (T9.3).
- **Item b** — the noninterference theorem + proof sketch for the chosen design (T9.4–T9.6).
- **Item c** — worked examples exercising the decision (T9.3, T9.7).

---

## 2. Findings

### T9.1 — The lattice is a Biba integrity lattice; "honest = high-integrity"; endorsement = certified `Swap`

`G = Exact ⊐ Proven ⊐ Empirical ⊐ Declared` is a 4-element total order (a chain), with `meet` =
greatest-lower-bound = "weakest wins". Read as a **Biba integrity lattice** (Biba 1977, the
order-dual of Bell–LaPadula confidentiality): `Exact = ⊤` (most trusted / highest integrity),
`Declared = ⊥` (least trusted). Composition by `meet` is exactly Biba's "an output is no more trusted
than its least-trusted input"; raising integrity ("endorsement") requires a *trusted action*. In
Mycelium that trusted action is a **certified `Swap`** — the `G-Swap` rule is the only grade-raising
rule, and it raises only when `cert.valid` (RFC-0002's per-swap checker, the VR-5 enforcement point).
This is the integrity-direction analogue of Jif's `declassify`/`endorse` (Myers, POPL 1999), but
gated on *checked evidence* rather than a privilege — strictly stronger.

*Position.* The grading system is an integrity/provenance analysis, not a confidentiality one. The
`meet` it composes over is already implemented and exhaustively law-checked (commutative,
associative, idempotent, identity `Exact`) in `crates/mycelium-core/src/guarantee.rs`
(`GuaranteeStrength::meet`/`propagate`). Grounding: T3.2; RFC-0006 §8 Q3; RFC-0018 §3.1/§7.

### T9.2 — Two candidate properties, and which one VR-5 actually demands

Two distinct, each internally-consistent, noninterference properties are available:

- **Full integrity NI (control + data).** An output graded `g` is unchanged under *any* variation of
  inputs graded `⋢ g`, **including influence via control flow** (which branch was taken). Soundness
  requires a `pc` index that degrades branch results by the scrutinee's grade — **Design B**
  (Volpano–Smith–Irvine 1996; DCC, Abadi et al. POPL 1999).
- **Data-provenance integrity (data only).** The **content** of an output graded `g` is constructed
  only from data that *flowed* (operand → result, via `meet`) from inputs graded `⊒ g`; the *choice
  among* equally-graded results may depend on weaker control. This is a **graded-coeffect
  data-dependency** property (Granule, Orchard–Liepelt–Eades ICFP 2019; Choudhury et al. ESOP 2022) —
  **Design A**.

*Position.* VR-5 / the honesty rule is a statement about **what evidence backs a value's content**
("this byte is `Exact` because its bits came from `Exact` data"), **not** about information leakage
through control channels. The property Mycelium actually asserts is therefore **data-provenance
integrity**, and Design A is the design that states *exactly* that — no more, no less. Grounding:
CLAUDE.md §1 (the honesty rule); RFC-0018 §4.5 reason 1; RFC-0006 §8 Q3.

### T9.3 — The distinguishing counterexample (RP-2 item a; §11 item 3)

```
fn pick(d: Bool @ Declared, x: Binary{8} @ Exact, y: Binary{8} @ Exact) -> Binary{8} =
    match d { true => x, false => y }
```

- Under **Design A** (`G-Match/A`): result grade `= meet(Exact, Exact) = Exact`. The scrutinee grade
  `g_d = Declared` does not appear in the result.
- Under **Design B** (`G-Match/B`): `pc' = pc ∧ g_d = Exact ∧ Declared = Declared`; result grade
  `= Declared`.

This is a genuine **control** dependence — *which* `Exact` byte you receive depends on a `Declared`
bit — and it is the canonical implicit-flow witness. **Verdict:**

- It **is** a counterexample to *full integrity NI under Design A* (confirming RP-2's "implicit-flows
  variant is confirmed necessary **if** full IFC NI is the goal"): a `Declared` condition influenced
  an `Exact`-typed result.
- It is **not** a counterexample to *data-provenance NI*: every bit of the output originates from an
  `Exact` input (`x` or `y`); **no `Declared` datum is present in the output's content**. The
  `Declared` bit influenced *selection*, not *content*.

So the two designs are not "right vs wrong" — they soundly characterize **two different properties**,
and the example is precisely the program on which they diverge. The maintainer's R18-Q1 choice is a
choice of *which property the guarantee system is for*. Grounding: RFC-0018 §4.5; RP-2 falsification
conditions.

### T9.4 — Design A's guarantee, stated precisely (RP-2 item b; §11 item 1)

Bridge to the implementation: the reference interpreter already threads runtime guarantees by
`GuaranteeStrength::propagate(intrinsic, inputs)` = `meet` over the **data** inputs of each
`Op`/`Construct`/`Let`/`App`, and a `Match` result simply *is* the selected branch's value (the
scrutinee is consumed by selection, not folded into the result's content). The Design A static rules
(`G-Op`, `G-Con`, `G-Let`, `G-App`, `G-Match/A`, `G-Swap`) mirror that runtime composition rule for
rule. This yields a two-part statement.

> **Theorem (Design-A data-provenance soundness).** Let `e` be a closed, well-graded L1 v0 term with
> `Γ ⊢ e : τ @ g` under the Design A rules, evaluated by the reference interpreter to a value `v`
> with runtime tag `grade(v.Meta)` (RFC-0001 §4.6/§4.7).
>
> **(i) Static–dynamic agreement (grade preservation).** `grade(v.Meta) ⊒ g` — the static grade is
> a sound under-approximation (in the integrity order) of the runtime tag; the static checker never
> certifies a result *more* trusted than the runtime `meet` makes it. Equivalently, evaluation never
> *lowers* a result below its statically predicted grade except where the dynamic `Meta` already
> records a weaker basis.
>
> **(ii) Data-provenance noninterference.** `grade(v.Meta)` equals the `meet` of the grades of
> exactly those inputs whose **content** data-flows into `v` (through `Op`/`Construct`/`Let`/`App`
> and through a `Swap` that did *not* endorse); a `Match` contributes only the selected branch's
> grade, never the scrutinee's. Consequently an `Exact` tag on `v` **certifies that `v`'s content is
> built only from `Exact` data** — *except across an endorsing `Swap`*, where `cert.valid`
> (RFC-0002, VR-5) is the sole sanctioned grade-raise.

### T9.5 — Proof sketch (RP-2 item b)

Standard **type preservation + substitution** for a graded judgment, with the grade as a coeffect the
`meet`-rules track. Induction on the typing derivation; representative cases:

- **`G-Const`.** `v`'s content is the literal; provenance is its own `Meta` grade; (i)/(ii) hold by
  the `g ≤ grade(v.Meta)` premise.
- **`G-Op` / `G-Con`.** Result grade `= meet` of operand grades (and the prim's intrinsic floor). By
  IH each operand's content-provenance is `⊒` its grade, hence `⊒` the meet; the union of operand
  provenances is `⊒` the meet. This is exactly `propagate`'s fold. ✓
- **`G-Let` / `G-App`.** A **substitution lemma** (substituting a value graded `g₁` for a variable
  demanded at `g₁` preserves grades by `meet`) gives the result. `G-App`'s `g₁' ⊒ g₁` premise (the
  LR-6 demand check) is the side-condition the lemma needs. ✓
- **`G-Match/A` (the load-bearing case).** The result *is* one of the branch bodies `eᵢ`; its
  content-provenance is `P(eᵢ) ⊒ gᵢ ⊒ meet over branches = ` result grade. The scrutinee's
  provenance is **not** part of the result's content (selection ≠ data flow), mirroring the static
  rule's omission of `g_s`. The static rule and the content-provenance semantics agree **by
  construction**. ✓ (This is also where the honest *gap* lives — see T9.6.)
- **`G-Swap`.** The only grade-raising rule. When `cert.valid` it endorses to `g_out`; provenance is
  reset to the certificate's endorsed basis. The theorem's "except across an endorsing `Swap`" clause
  is discharged here, and its soundness is **delegated to the existing RFC-0002 per-swap checker** —
  no new trusted machinery (KC-3). Without a valid certificate, `g_out ⊑ g_in`, so `meet` is
  preserved. ✓
- **`G-Fix`.** The fixpoint's grade is invariant (premise grade = conclusion grade); a standard
  step-indexed/approximation argument over the total-and-partial fragment closes it. ✓

The novel interaction (lattice grading × machine-checked runtime certificates, flagged in T3.2 / §7)
is **isolated to the single `G-Swap` case** and reduced to the certificate checker that already
enforces VR-5. *No new soundness burden is introduced beyond RFC-0002.*

### T9.6 — The honest limit: Design A's sufficiency rests on purity (the precondition to record)

Design A is sufficient **because L1 v0 is a pure, total, call-by-value expression calculus with no
mutable state**, and reified/declared effects (RFC-0014). In such a calculus an implicit (control)
flow becomes observable *only* through the **data result** of the branching construct — which the
data-provenance theorem (T9.4) already covers. The `pc`-index of Design B is therefore **redundant on
observable outputs** in v0.

This sufficiency has a precise boundary: **if Mycelium later gains effects observable outside a
`Match`'s result value** (mutable state, or RFC-0008 runtime side-channels not reified as graded
outputs), a branch could influence an `Exact` output *without* that influence appearing in any
operand's content-provenance — reinstating the T9.3 control channel as a real integrity leak. At that
point stage-1b / the effect system must either **(route i)** treat each effect as a **graded output**
(so it re-enters the data-provenance theorem — RFC-0014's *reified, declared* effects already point
here) or **(route ii)** add a local `pc`-index for the effectful sub-language (Design B locally). This
precondition is the honest fine print on choosing Design A and should be recorded with the decision.
Grounding: RFC-0014 (reified effects); RFC-0018 §9 (effects future possibility); RFC-0008.

### T9.7 — R7-Q2 discharged (the `Match` default arm)

Under `G-Match/A` the default arm `d` is meet-folded **identically** to named alternatives
(`g₁ ∧ … ∧ gₙ ∧ g_d`); it carries no special degradation. RFC-0007 §8 **R7-Q2** is therefore closed
under Design A — and the T9.5 induction treats the default branch as just another `eᵢ`, so the
closure is *grounded by the proof*, not merely asserted. (Under Design B it is likewise symmetric —
both run under `pc'` — so R7-Q2 closes either way; the maintainer's R18-Q1 choice fixes which rule is
normative.) Grounding: RFC-0018 §4.4/§8 R18-Q2.

### T9.8 — R18-Q4 (certificate-validity scope) recommendation

The T9.5 proof needs `cert.valid` only to be **decidable and sound at the point it is relied on**; it
does **not** need the proof checker to run *inside* the type checker. Recommend: the static checker
carries the certificate *reference* + the claimed `(r₁ → r₂, g_out)` at the type level (decidable;
keeps the trusted checker free of the proof checker — KC-3), and `cert.valid` is discharged by the
existing RFC-0002 checker at elaboration/runtime, exactly as VR-5 is enforced today. This separation
matches the proof's structure (endorsement soundness is *delegated*, not inlined). Recommend recording
**R18-Q4 = "certificate reference at the type level; validity at elaboration/runtime."**

---

## 3. Decisions this record supports

- **R18-Q1 (implicit-flows decision) → recommend Design A**, grounded in T9.2 (VR-5 = provenance, not
  secrecy), T9.3 (Design B mistypes the `pick` example as `Declared` — less accurate for provenance),
  KC-3 (no `pc`-register in the trusted checker), and T9.4–T9.5 (Design A is *sound for the property
  VR-5 actually asserts*). **The decision remains the maintainer's**; this record discharges the
  research that was gating it.
- **§11 / RP-2 noninterference obligation → discharged** by T9.3 (counterexample), T9.4 (theorem),
  T9.5 (proof sketch), T9.7 (worked default-arm case). Tagged **Declared-with-argument** (not
  machine-checked); mechanization is the basis for a future `Proven` upgrade.
- **R18-Q2 / R7-Q2 → closed** (T9.7).
- **R18-Q4 → recommend "reference at type level, validity at elaboration/runtime"** (T9.8).
- **A recorded precondition on Design A** (T9.6): when observable effects land, effects must be graded
  outputs (route i) or carry a local `pc` (route ii). Feeds RFC-0018 §9 and the RFC-0008/0014 effect
  track.

Not resolved here (out of scope / not gating ratification): R18-Q3 (prim-table grading schedule —
the conservative `G-Op` default is sound meanwhile) and R18-Q5 (grade inference in stage 1a — a
checker-scope decision). Both are recorded in RFC-0018 §8 as maintainer decisions.

---

## 4. Key sources

- **Biba**, *Integrity Considerations for Secure Computer Systems* (MITRE, 1977) — the integrity
  lattice; "weakest wins"; endorsement requires a trusted action.
- **Denning**, *A Lattice Model of Secure Information Flow* (CACM 1976) — the lattice/meet basis.
- **Volpano, Smith, Irvine**, *A Sound Type System for Secure Flow Analysis* (J. Computer Security
  1996) — first type-system NI proof; the `pc` label (Design B precedent).
- **Myers**, *JFlow / Jif* (POPL 1999) — declassification/endorsement via gated operations; the
  `Swap`-as-endorsement analogue.
- **Pottier & Simonet**, *Information Flow Inference for ML* / FlowCaml (POPL 2002; TOPLAS 2003) —
  constraint-based label inference (stage-1b model) and the documented annotation-burden cost under
  `pc` tracking (grounds the Design A recommendation).
- **Abadi, Banerjee, Heintze, Riecke**, *A Core Calculus of Dependency* (DCC, POPL 1999); **Algehed**
  (PLAS 2018); **Choudhury, Eades, Weirich, et al.** graded dependency (ESOP 2022) — the dependency-
  calculus soundness story read in the integrity direction.
- **Orchard, Liepelt, Eades**, *Quantitative Program Reasoning with Graded Modal Types* / Granule
  (ICFP 2019) — graded coeffect modality over a semiring; the data-provenance soundness shape; the
  ships-a-security-`Level`-grade precedent. (Research prototype — production deployment not
  established; flagged.)
- In-repo: `crates/mycelium-core/src/guarantee.rs` (the `meet`/`propagate` implementation +
  exhaustive law check); RFC-0018 (the graded judgment, Design A/B, §7 prior art); RFC-0006 §8 Q3
  (the mechanism position); RFC-0007 §4.3/§8 R7-Q2; RFC-0002 (the per-swap certificate checker);
  RFC-0014 (reified effects, for T9.6); `research/03` T3.2 (the originating finding).

---

## 5. Honest-uncertainty register

- **The noninterference result is a stated theorem + proof *sketch*, not a machine-checked proof.**
  Per VR-5 the soundness claim is tagged **Declared-with-argument**, not **Proven**. A `Proven`
  upgrade requires mechanization (Lean/LiquidHaskell track) — named as follow-up, not claimed.
- **"No found precedent" for lattice grading × machine-checkable per-value runtime certificates**
  (T3.2 / RFC-0018 §7) is *absence of evidence*, not proof of novelty — flagged as such. The
  argument's response (isolate the novelty to `G-Swap`, delegate to RFC-0002) is itself a design
  claim, not a theorem about the literature.
- **The T9.6 effect boundary is a precondition, not an enforced check.** Nothing today prevents a
  future effectful extension from violating Design A's purity assumption; recording the obligation is
  the honest mitigation, not a guarantee.
- **Design A's "sufficiency" is relative to the chosen property** (data-provenance, T9.2). It is, by
  construction (T9.3), *insufficient* for full control-sensitive integrity NI. The recommendation is
  conditional on the maintainer wanting the provenance property — which the honesty rule indicates,
  but which is a normative choice (R18-Q1), not a fact this record can settle.

---

## Meta — changelog

- **2026-06-18 — Created.** Discharges RP-2 / RFC-0018 §11 (the stage-1 grading noninterference
  obligation): the data-provenance vs full-IFC framing (T9.2), the Design-A/B distinguishing
  counterexample (T9.3), the Design-A data-provenance soundness theorem + proof sketch reduced to the
  existing `meet`/`propagate` implementation and the RFC-0002 certificate checker (T9.4–T9.5), the
  purity precondition (T9.6), R7-Q2 closure (T9.7), and the R18-Q4 recommendation (T9.8). Recommends
  recording R18-Q1 = Design A. Tagged Declared-with-argument (not machine-checked); mechanization is
  the basis for a future Proven upgrade. Grounds RFC-0018 toward ratification; the RFC stays Draft
  pending the maintainer's R18-Q1/R18-Q4 decisions. Append-only.
