# Research Record 26 — DN-64 Type-System R&D Planning: OQ-A / OQ-C / OQ-D / OQ-E / OQ-S

> **What this file is.** A durable R&D planning record for the five type-system
> open questions from DN-64 §6 that the maintainer dispositioned for further R&D work:
> OQ-A (machine-checked soundness), OQ-C (chained approximation composition rule),
> OQ-D (three-layer memory ergonomics), OQ-E (substrate/hypha reclamation interaction),
> and OQ-S (grade propagation through monomorphization). Each section states the
> **maintainer decision faithfully**, provides a **grounded analysis** with corpus
> citations, recommends **concrete next steps** keyed to the M-ids minted in the
> commission, and carries an **honest strength tag** on every claim. OQ-F is cross-
> referenced but not duplicated; it is being pursued via the GPU experiment track.
>
> Findings are labeled **T26.1–T26.18** (continuing the T-scheme). Conducted 2026-06-29
> from DN-64 §6/§7, RFC-0018, RFC-0019, RFC-0027, RFC-0008, RFC-0002, ADR-010/011,
> DN-32, DN-33, DN-35, DN-59, `crates/mycelium-cert`, and `research/09`.
>
> **Posture (transparency rule / VR-5).** Every recommendation is `Declared` — a proposed
> design direction, not a proven result. Comparisons of mechanization tools are
> `Empirical` where based on published evaluation data from the literature; `Declared`
> where based on this record's assessment of fit. No claim is upgraded past its
> supportable basis.

---

## 1. Scope

DN-64 §6 enumerated nineteen open questions; the maintainer dispositioned five for active
R&D and task assignment: OQ-A (M-827), OQ-C (M-829), OQ-D (M-830), OQ-E (M-831), and
OQ-S (M-844). A sixth, OQ-F (M-832), is being pursued via a separate GPU experiment
track (`experiments/mycelium_experiments/vsa_bounds/`). This record covers the five
dispositioned questions in depth and cross-references OQ-F.

---

## 2. OQ-A — Graded type soundness (M-827)

### 2.1 Maintainer decision (faithful)

*Yes, R&D and enact a machine-checked noninterference proof for `Γ ⊢ e : τ @ g`.*

RFC-0018 §11 has always named mechanization as the basis for a future `Proven` upgrade;
the current status is `Declared-with-argument` (research/09, T9.5). The maintainer's
disposition makes this an active R&D task, not an indefinitely-deferred aspiration.

### 2.2 Grounded analysis

**T26.1 — The existing proof artifact.** `research/09` provides a complete hand-constructed
proof sketch for the Design-A data-provenance soundness theorem (T9.4–T9.5). The proof
has five cases (G-Const, G-Op/G-Con, G-Let/G-App, G-Match/A, G-Swap) and reduces the
novel `G-Swap` case to the existing RFC-0002 certificate checker. The `meet`/`propagate`
implementation in `crates/mycelium-core/src/guarantee.rs` is exhaustively law-checked.
This means the mechanization task has a concrete, well-structured target: the proof sketch
is the guide, not a blank-slate formalization effort. (`Empirical` — the proof sketch is
verified by its own internal consistency and grounded in cited prior art; not yet mechanized.)

**T26.2 — Tool comparison: Lean 4 vs Liquid Haskell vs Agda/Rocq.**

The existing `proofs/lh-bundle` uses **Liquid Haskell** (LH), which is the natural
continuation path. However, Lean 4 and Agda/Rocq deserve explicit comparison for this
task:

| Criterion | Lean 4 | Liquid Haskell | Agda / Rocq |
|---|---|---|---|
| Proof type | Interactive theorem prover (dependent types, tactics) | Refinement-type checker (SMT-backed, semi-automatic) | Interactive theorem prover (dependent types, tactics) |
| Expressive power for lattice NI proofs | High: dependent types directly express the graded judgment | Medium: SMT can discharge equational lattice laws but has limits on inductive proofs over derivation trees | High: both handle inductive proofs on typing derivations well |
| Prior art in this style | Iris (Coq), Mechanized Semantics (Lean), PLFA (Agda) | FlowCaml refinement (partial) | Numerous IFC mechanizations in Coq (Brumley/Picard, JSAC) |
| Continuity with `proofs/lh-bundle` | Requires rewrite in Lean | Direct continuation | Requires rewrite in Agda/Rocq |
| SMT automation for lattice laws | Limited (needs encodings) | Strong (`meet` laws dischargeable by Z3 / CVC5 directly) | Limited |
| Effort for derivation-tree induction | Moderate (Lean 4 tactics are mature) | Higher (LH is weakest here; inductive proofs over ADTs require `ple` + manual lemmas) | Lower (Agda/Rocq have mature eliminators for indexed derivations) |
| Ecosystem/maintenance | Active; mathlib4 growing fast | Production-supported by Tweag; well-tested on refinement properties | Rocq: mature; Agda: smaller |

(`Empirical` — based on the published characterizations of each system; the specific LH vs
Lean 4 inductive-proof comparison is grounded in the known LH limitation with deep
induction: the LH paper (Vazou et al., ICFP 2014) explicitly bounds its soundness to
properties expressible as SMT queries, and induction over derivation trees requires
termination-annotated `ple` lemmas that are substantially more difficult than Lean's
tactic-based structural induction. The tool-specific claims are therefore `Empirical`,
not `Proven`.)

**T26.3 — Recommended path: staged Lean 4 mechanization, building on the LH precedent.**

The recommendation is **Lean 4**, with the following rationale:

1. The target theorem (T9.4) is a **type-preservation/substitution induction over a
   graded typing derivation tree** — exactly the class where Lean 4's dependent-type
   eliminators and `simp`/`aesop` tactic automation outperform LH's SMT-backend induction.
2. The `G-Swap` case delegates to the RFC-0002 certificate checker; in Lean 4 this becomes
   an **axiom / admitted lemma** (the certificate checker's correctness) over which the
   main proof is parametric — the cleanest architectural boundary for a staged proof plan.
3. LH can continue to cover the **lattice law properties** (`meet` commutativity,
   associativity, idempotence, identity `Exact`) that it already handles well — these are
   equational SMT queries. The two tools work on disjoint parts of the proof obligation.

This is a `Declared` recommendation; the alternative (Rocq, which has mature IFC
mechanization precedent — Brumley-style security proofs, Iris) is a legitimate alternative
and should be recorded as a named option for the maintainer.

**T26.4 — Staged proof plan for M-827.**

Stage 1a (monomorphic, pure, call-by-value — the current RFC-0018 Enacted scope):

- Formalize the L1 v0 syntax (terms, types, graded contexts) as an inductive Lean 4 type.
- Formalize the graded typing judgment `⊢ e : τ @ g` as an indexed inductive proposition
  matching the seven rules (G-Const, G-Op, G-Con, G-Let, G-App, G-Match/A, G-Swap,
  G-Fix).
- State and prove the **grade preservation** lemma (T9.4 part i) by structural induction
  on the derivation.
- State and prove the **substitution lemma** (the load-bearing helper for G-Let/G-App).
- State and prove the **data-provenance NI** theorem (T9.4 part ii) over closed, well-graded
  terms.
- The `G-Swap` case is discharged by an `axiom cert_valid_endorses : ...` (the RFC-0002
  checker's correctness — a separately-provable property, flagged as an admitted lemma in
  stage 1a, the target of stage 1b).

Stage 1b (certificate checker correctness):

- Mechanize the RFC-0002 per-swap checker properties (`cert.valid` iff the claimed
  `(r₁ → r₂, g_out)` is correct for the certificate kind and value).
- Discharge the `cert_valid_endorses` axiom from stage 1a.

Stage 2 (extension, future): add the T9.6 purity precondition boundary as a formal
side-condition when effectful constructs land (RFC-0014/RFC-0008 route).

(`Declared` — the staged proof plan is a design proposal; the feasibility of each step
is grounded in the T9.5 proof sketch but not itself verified.)

**T26.5 — Open tension: purity precondition (T9.6).** Design A's sufficiency rests on
L1 v0 being a pure, total, call-by-value calculus. `research/09` T9.6 records this as
a named precondition: when observable effects land (RFC-0014 / RFC-0008 effectful
extensions), the mechanized proof must either treat each effect as a graded output (route i)
or add a local `pc`-index (route ii). The M-827 Lean formalization should state T9.6 as
an explicit side-condition on the main theorem, not silently depend on purity. (`Declared`
— the side-condition is already articulated in research/09; the mechanization task inherits
it.)

### 2.3 Recommended next steps

**M-827** deliverables (proposed, `Declared`):

1. Stand up a Lean 4 project under `proofs/lean4-grading/` with the L1 v0 core
   formalization (syntax, reduction, graded typing).
2. Prove the substitution lemma and grade preservation (T9.4 part i) — the two cheapest
   wins that establish the proof infrastructure is sound.
3. State the full NI theorem (T9.4 part ii) with `cert_valid_endorses` admitted.
4. Annotate the T9.6 purity precondition explicitly as a side-condition on the theorem.
5. Record the admitted lemma as a named M-827 FLAG feeding M-828 (stage 1b).

The LH `proofs/lh-bundle` continues to carry the lattice law mechanization (its natural
fit). No migration of LH proofs into Lean is required or recommended.

---

## 3. OQ-C — Chained approximation composition (M-829)

### 3.1 Maintainer decision (faithful)

*Define a tractable composition rule for bounds.*

The current `SwapError::ApproximateSource` path explicitly refuses to compose a
non-`Exact` source bound with a swap's own epsilon (E2-1 rule is marked "not defined yet"
in `crates/mycelium-cert/src/lib.rs:126` and `dense.rs:19`). The maintainer has decided
that a tractable rule should be found and enacted rather than leaving chained approximation
pipelines blocked at `Exact`-source-only.

### 3.2 Grounded analysis

**T26.6 — Current state.** The `dense.rs` checker and `dense_vsa.rs` both check
`source.guarantee == Exact` before proceeding; a non-`Exact` source triggers
`SwapError::ApproximateSource` with the message "composing its bound with the swap ε is
not a defined rule yet (E2-1)." This is never-silent (G2) — the refusal is explicit, not
a silent downgrade. The composition rule, when defined, must remain never-silent: a
composed bound must be derivable (not asserted) or the operation must remain refused.
(`Proven` — directly readable from `crates/mycelium-cert/src/{lib.rs,dense.rs}`.)

**T26.7 — The composition design space.** Three main approaches to bound composition exist
in the numerical analysis / error propagation literature:

**(A) Absolute additive composition.** If the source has an absolute error `ε_in` and the
swap introduces an additional absolute error `ε_swap`, the composed bound is
`ε_total = ε_in + ε_swap`. This is the simplest rule and is always valid (triangle
inequality). Its weakness: it is **pessimistic** — absolute bounds accumulate linearly
even when errors are uncorrelated, so deep pipelines produce loose bounds quickly. This
maps directly onto ADR-011's universal `BoundBasis` (the basis field records whether the
bound is absolute or relative, per ADR-011 §3). Strength: `Proven` where `ε_in` and
`ε_swap` are both absolute and the swap operation is a contraction or bounded-expansion
operator. (`Empirical` — the additive rule is a standard result in interval arithmetic
(Moore 1966); its mapping onto the ADR-011 `BoundBasis` structure is this record's
analysis and is `Declared` pending corpus validation.)

**(B) Relative multiplicative composition.** If bounds are expressed as relative errors
(`|x - x_true| / |x_true| ≤ ε_rel`), chaining through a well-conditioned operator with
condition number `κ` gives `ε_total_rel ≈ κ · ε_in_rel + ε_swap_rel`. This is tighter
than absolute addition when the pipeline's condition number is bounded and known. It
requires that the swap operation's epsilon is itself stated as a relative bound and that
the condition number is available (or assumed 1 for norm-preserving transforms). Strength
tag on the composed bound: `Proven` when `κ` is provably bounded; `Empirical` when
estimated by sampling; `Declared` when assumed. (`Empirical` — relative error propagation
is a standard floating-point analysis result (Higham 2002); the condition-number
requirement is a precondition this record assesses as applicable to Dense↔VSA swaps.)

**(C) Affine arithmetic (AA) / Taylor-model enclosures.** Track symbolic affine forms
`x_i = x_i_0 + Σ ε_k · α_{ik}` where the `ε_k` are noise symbols (Fijany et al. 1998;
De Figueiredo and Stolfi 2004). AA avoids dependency explosions of interval arithmetic
and gives tighter bounds for linear-dominated pipelines. It is, however, substantially
more complex to implement and requires that each swap produces an AA form (not just a
scalar bound). For the current Dense↔VSA swap, which is already a vector operation, AA
would require per-element noise tracking — a non-trivial trusted-kernel addition
(violating KC-3 proportionality unless the value is high). (`Declared` — AA's tighter
bounds are known; the KC-3 tension is this record's assessment.)

**T26.8 — Recommended tractable rule: absolute additive with BoundBasis-tagged kinds.**

The tractable first rule is **(A)** — absolute additive composition — with the following
structure:

```
composed_bound(ε_in: Bound, ε_swap: Bound) -> Result<Bound, SwapError>
  | both absolute: ε_total = ε_in.value + ε_swap.value, basis = meet(ε_in.basis, ε_swap.basis)
  | mixed abs/rel: return Err(ApproximateSource) still — no rule for mixed kinds yet
  | both relative: ε_total_rel = ε_in.value + ε_swap.value (pessimistic; see T26.9 note)
```

The composed `Bound`'s `basis` is the `meet` of the two input bases (ADR-011 universal
`BoundBasis` — the weaker basis wins, exactly as the guarantee lattice weakens under
`meet`). The composed bound's guarantee tag is `Proven` when both `ε_in` and `ε_swap` are
`Proven`-tagged absolute bounds and the `+` rule is the applied theorem; `Empirical` if
either input is `Empirical`; `Declared` if either is `Declared`.

This is `Declared` — a proposed rule form, not yet enacted. The rule must be validated
against a set of worked swap pipelines (Dense f64→f32→f64 round-trip, Dense→VSA→Dense)
before adoption.

**T26.9 — Honest limit: relative-plus-relative is NOT simply additive.** The relative
composition rule `ε_total_rel = ε_in_rel + ε_swap_rel` is conservative (pessimistic) but
not tight. A tighter relative bound requires the condition number or uses second-order
terms. Recording this as an explicit limitation: the simple relative additive rule
**over-bounds** the true error, which is safe (never under-reports the error) but may
produce bounds loose enough to downgrade otherwise-`Proven` pipeline outputs to `Empirical`
prematurely. This is an honest tradeoff the maintainer must accept or defer to a tighter
rule in a follow-on. (`Declared` — the over-bounding assessment is standard floating-point
analysis; the "prematurely downgrade" consequence is this record's design-inference.)

**T26.10 — Where the rule stays `Proven` vs drops to `Empirical`.** A composed bound is
`Proven` iff: (a) both component bounds carry `Proven` or stronger tags, (b) the
composition rule is the additive absolute form (a certified theorem), and (c) the swap
operation is proven to introduce at most `ε_swap` absolute error (the existing RFC-0002
Dense certifier already establishes this for f64→f32 rounding with `Proven`-modulo-the-
rounding-theorem). If either component is `Empirical`, the composed bound is `Empirical`.
If either is `Declared`, the composed bound is `Declared`. This is the `meet` law applied
to bound strength, consistent with ADR-011 §3's universal-basis design and the guarantee
lattice's never-upgrade rule (VR-5). (`Declared` — the lattice-extension reasoning is this
record's proposal; it follows from ADR-011 + VR-5 by construction but has not been
enacted in code.)

### 3.3 Recommended next steps

**M-829** deliverables (proposed, `Declared`):

1. Extend `Bound` (RFC-0002 / ADR-010/011) with a `kind: BoundKind` discriminant
   `{Absolute, Relative}` if not already present — the additive rule requires knowing kind.
2. Implement `compose_bounds(a: &Bound, b: &Bound) -> Result<Bound, SwapError>` in
   `crates/mycelium-cert` with the absolute-additive rule (T26.8) and the mixed-kind refusal.
3. Replace the `ApproximateSource` guard in `dense.rs` and `dense_vsa.rs` with a call to
   `compose_bounds`, keeping the explicit error path for the mixed-kind case (G2: never
   silently composing incommensurable bound kinds).
4. Property-test the rule against round-trip pipelines: `ε_composed ≥ |output - true_output|`
   must hold for all sampled inputs — this is the soundness condition the property test
   embodies.
5. Record the T26.9 over-bounding limitation in the E2-1 rule's doc comment, and open a
   follow-on task for relative-bound tightening (a separate M-task, not part of M-829).

---

## 4. OQ-D — Three-layer memory model "stupid easy" ergonomics (M-830)

### 4.1 Maintainer decision (faithful)

*Build to ensure and guarantee the "stupid easy" ergonomics.*

DN-32 §4 names "stupid easy" ergonomics as goal 3: "the default path (affine ownership)
requires no annotation; sharing is explicit and the cost is opt-in by construction; the
programmer writes ordinary value-semantic code and the layers pick the cheapest valid
mechanism." RFC-0027's OQ-1 and OQ-4 are resolved (DN-32 §3, §12). The maintainer's
disposition is to take the goal from `Declared` design direction to a concrete build
commitment.

### 4.2 Grounded analysis

**T26.11 — What "stupid easy" means concretely (from DN-32 §4 + RFC-0027 §12).** The
Layer-1-affine-primary design makes zero-annotation the common case: unique data is moved,
not shared, at (near-)zero cost; reclamation is a scope-exit drop (RT7 LIFO); no RC
traffic, no annotation required. Sharing engages Layer 2 only when the programmer writes
an explicit sharing construct (RC handle, channel). Layer 3 (region batching) is below
the surface. The "stupid easy" promise rests on **three design properties**:

1. **Affine as the default (no annotation for the unique path).** A value with one owner
   moves without any `rc_inc`/`rc_dec` instrumentation. This is the Layer-1 invariant
   (DN-32 §2.1; RFC-0027 §12). (`Proven`-modulo-the-Layer-1-build — the property is an
   invariant of the design; whether the current L1 prototype fully enforces it is tracked
   by MEM-1/2/3 completion state per `docs/planning/E12-Memory-Model-Build-Plan.md`.)

2. **Sharing is surface-visible and opt-in.** `RC<T>` / the `Sender`/`Receiver` pair is
   the explicit sharing gate; a non-sharing program touches no RC machinery. The layer
   boundary is never-silent (G2): the programmer cannot accidentally share a value (it
   would be an `rc_inc` — an explicit step). (`Declared` — the surface syntax for explicit
   sharing is not yet landed; it is the Layer-2 deferred normative target per RFC-0027
   §12 honest-scope note.)

3. **Inference / static analysis removes RC ops on the unique path.** Perceus-style
   uniqueness analysis (`rc == 1` proof) eliminates RC increments/decrements on proven-
   unique values without programmer annotation. In Lean 4 (Perceus PLDI'21), this is
   performed as a compiler pass over a monomorphic IR — directly analogous to Mycelium's
   MEM-4 static-tier (`mycelium-mir-passes`, DN-35). (`Empirical` — Perceus is a published
   result (Reinking et al. PLDI 2021) achieving 90th-percentile performance on the Koka
   benchmarks; the mapping onto Mycelium's MEM-4 pass is `Declared` pending the DN-35
   increment-3 build.)

**T26.12 — RFC-0027 OQ-1 and OQ-4 status.** Both are resolved per RFC-0027 §12:
OQ-1 (sweep-order sibling coupling) resolved by DN-32 §3 (siblings concurrent by default,
strong coupling opt-in); OQ-4 (`rc==1` reuse visibility) resolved as EXPLAIN-record-only
by default (G2: the DAG records the choice; surface opt-in deferred to Phase 3). Neither
resolution changes the "stupid easy" ergonomics goal — both are below the programmer's
annotation surface. (`Proven`-modulo-LR-9 for OQ-1 safety; `Declared` for OQ-4's
surface-deferred stance, per RFC-0027 §12.)

**T26.13 — RFC-0027 OQ-1 and OQ-4 implications for M-830.** M-830's build task does not
need to re-open OQ-1/OQ-4. It needs to deliver the programming-surface experience:
(a) the affine move as the default syntactic form (no annotation), (b) an explicit, named
sharing construct, and (c) MEM-4 static analysis making (a) cheap at the implementation
level. The `docs/planning/E12-Memory-Model-Build-Plan.md` increment sequence (MEM-1/2/3
built; env-machine reclamation = forward epic, Increment 3 / task #6) sets the order.

**T26.14 — The "stupid easy" ergonomics gap vs the prior-art precedent.** The three
strongest prior-art points for annotation-free affine ownership are:

- **Rust** (`let x = val; let y = x;` — move by default, no annotation): precedent that
  affine-by-default is ergonomic for systems-level code (`Empirical` — seven years of Rust
  production use).
- **Clean uniqueness types** (Barendsen and Smetsers 1993): the formal precursor, but
  required explicit uniqueness annotations at function boundaries. Mycelium avoids this by
  treating unique as the default (not the annotated case) — closer to Perceus's analysis
  than to Clean. (`Empirical` — literature result.)
- **Koka with Perceus** (Reinking et al. PLDI 2021; Leijen ICFP 2019): functional language
  with affine-by-default memory management at the compiler level, zero annotation by the
  programmer. This is the strongest positive precedent for Mycelium's goal. (`Empirical`.)

The gap between Koka/Perceus and Mycelium: Koka operates on a pure functional core;
Mycelium adds hypha concurrency (RFC-0008 RT7). DN-32 §3 argues (and OQ-1 resolves) that
the sibling-concurrency interaction does not compromise safety (LR-9 rules out cross-sibling
aliases). The "stupid easy" claim's residual uncertainty is **not in Layer 1** but in
Layer 2 — whether the Perceus-style analysis can be applied to the MIR without breaking
the `fast`/`certified` split (ADR-032/RFC-0034) — which is explicitly the DN-35 increment-
3 work item. (`Declared` — this assessment of where the uncertainty lives is this record's
analysis.)

### 4.3 Recommended next steps

**M-830** deliverables (proposed, `Declared`):

1. Define a **usability specification** for "stupid easy": three benchmark programs
   (a pure function, a shared-value producer, a hypha-tree computation) that must compile
   with **zero memory-management annotations** on the Layer-1 path.
2. Implement the **MEM-4 static tier** (Increment 3 / task #6 per E12-Build-Plan) —
   Perceus-style uniqueness analysis on the MIR, eliminating RC instrumentation for
   proven-unique values without programmer annotation.
3. Run the three benchmark programs against the usability specification; report
   annotation count as an `Empirical` metric (not a proof of ergonomics, but a measurable
   proxy).
4. Record the Layer-2 surface syntax decision (what the programmer writes to create an
   explicit RC handle) as a separate design note feeding RFC-0027's follow-on.
5. The T9.6 purity precondition analogue: document that "stupid easy" ergonomics applies
   to the value-semantics path; hypha-concurrent programs that cross Layer-2 boundaries
   require the explicit sharing construct (opt-in, not annotation burden).

---

## 5. OQ-E — Substrate/hypha reclamation interaction (M-831)

### 5.1 Maintainer decision (faithful)

*Investigate, R&D and plan.*

The interaction of LR-8 affine `substrate` handles with RT7 structured lifetimes and
RFC-0014 bounded-effects recovery is not fully specified. The question: when a hypha
holding a `substrate` is reclaimed (via `reclaim` / RFC-0008 RT7), is the substrate
consumed, dormanted, or escalated? The disposition routes this to DN-59 and a future R2
`graft` RFC.

### 5.2 Grounded analysis

**T26.15 — What the corpus currently says.** Three corpus anchors are relevant:

**(a) LR-8 / `substrate`.** RFC-0006 LR-8: "values are immutable." The `substrate` is
the affine, external-resource-capability handle (`graft` — an affine `substrate` handle,
RFC-0008 §4.1 table, `crates/mycelium-cert` surface). A `substrate` is affine: it cannot
be duplicated; it must be either **consumed** (via `consume`) or transferred. Dropping it
without consuming is an explicit protocol violation (RFC-0027 OQ-5 records this as
`substrate/graft drop-without-consume protocol — depends on the graft implementation RFC`).
(`Proven` — LR-8 is a ratified corpus invariant; the `substrate` affinity is stated in
RFC-0008 and DN-32.)

**(b) RT7 structured lifetimes.** RFC-0008 RT7: "Runtime lifetimes are structured; an
orphan hypha is not expressible." A hypha created in a scope is bounded by that scope's
lifetime; the scope does not exit until its children have completed, been cancelled, or
been explicitly detached. If the hypha is `reclaim`'d (supervision-tree reclamation of
runtime units, not memory — RFC-0008 §4.1), the RT7 structured-scope invariant means the
hypha's scope exits. Any value the hypha holds — including a `substrate` — must be
disposed of before the hypha scope exits. (`Proven` — RT7 is ratified; the constraint is
structural.)

**(c) RFC-0014 bounded-effects recovery.** RFC-0014 specifies the bounded-effects
mechanism with an explicit budget and recovery path. If a hypha is `reclaim`'d under
RFC-0014's recovery model, the question is whether the `substrate` handle is part of the
effect budget (consumed at reclamation) or an external resource that must be separately
handled. The RFC-0014 enacted v1 and the static budget-bound syntax (RFC-0014 §10.1 D1,
Proposed) do not yet specify the `substrate` interaction. (`Declared` — the absence is
recorded in DN-64 §6 OQ-E; it is this record's assessment that the gap is real and must
be resolved before `graft` implementation.)

**T26.16 — The three candidate dispositions (analysis).** When a hypha holding a
`substrate` is reclaimed:

**(i) Consumed.** The `substrate` is treated as if `consume` were called — the external
resource is released. This is the cleanest path: the RT7 scope-exit invariant and LR-8
affinity both point here (a `substrate` must be consumed; when the scope exits it is, by
the affine discipline, consumed). The open question is whether the `substrate`'s `consume`
protocol is well-defined at reclamation time (is the resource in a state that admits safe
`consume`? does the external system support forced release?). For a `graft` to an external
I/O resource, forced release may or may not be valid. This is the OQ-5 question in
RFC-0027 ("drop-without-consume protocol depends on the graft implementation RFC").
(`Declared` — the analysis is this record's; the "forced release" concern is standard for
external-resource-owning capability handles, see Rust's `Drop` on `File`/socket types.)

**(ii) Dormanted.** The `substrate` is moved to a dormant state (analogous to checkpointing
— RFC-0008 §4.4) rather than consumed. The external resource is suspended, not released.
This requires the external resource to support suspension and implies a re-attachment
protocol when the hypha is restarted (or the `substrate` is passed to a new hypha). This
aligns with RFC-0008's checkpointing vocabulary but requires `graft` to define what
"dormant `substrate`" means for each external resource type. (`Declared` — the dormant
option requires RFC-0008 checkpoint semantics to extend to `graft`'d resources; not
currently specified.)

**(iii) Escalated.** The `substrate` is escalated to the reclaiming supervisor (the parent
scope / supervision-tree parent). This requires the supervision tree to accept affine
handle transfer and for the parent to subsequently decide consume vs dormant. This is the
most flexible option but adds the most complexity to the supervision protocol. It is the
analogue of OTP's "linked process propagates resource" model — a valid approach but
requiring explicit supervision-tree protocol for resource ownership transfer. (`Declared`.)

**T26.17 — Specification questions for M-831.** The R&D plan must answer, in order:

1. **Does `graft` define a per-resource forced-release protocol?** If yes, option (i) is
   the cleanest path (consume at reclamation, protocol-defined). If no, option (iii)
   (escalate to parent) is the safe default.
2. **Does the supervision tree have a type for "handles an affine `substrate`"?** If the
   supervision tree carries type information about `substrate`s it may receive (option iii),
   the type system can verify that the parent scope has a compatible handler.
3. **What is the interaction with RFC-0014 effect budgets?** If the `substrate`'s effects
   are part of the hypha's effect budget, the budget must be settled (consumed or cancelled)
   before reclamation. If the `substrate` is outside the budget (a capability, not an
   effect), the budget settles independently and the `substrate` is handled separately.
4. **Does dormanting (option ii) require new vocabulary?** If dormanting is chosen, it
   requires a `dormant_substrate` construct (R2 vocabulary; DN-63's distribution vocabulary
   is the precedent for R2 naming decisions).

### 5.3 Recommended next steps

**M-831** deliverables (proposed, `Declared`):

1. Add a section to DN-59 (append-only) recording the three candidate dispositions above
   and the four specification questions.
2. Commission a `graft` R2 RFC (new document) that:
   (a) defines the `graft`/`substrate` protocol (per-resource forced-release or not),
   (b) specifies which of (i)/(ii)/(iii) applies per resource class (or as a default),
   (c) specifies the supervision-tree type for `substrate`-owning hyphae,
   (d) reconciles with RFC-0014 effect-budget settlement at hypha reclamation.
3. The M-831 research note (this record) feeds the `graft` RFC's §Scope. The `graft` RFC
   is itself the gate on implementing `substrate`/`graft` in the codebase (currently
   deferred as RFC-0027 OQ-5).
4. Until the `graft` RFC lands, the existing `ApproximateSource` / explicit-refusal pattern
   is the model: any operation that would require a `substrate` reclamation decision emits
   a diagnostic naming the open question, never silently drops or silently consumes
   (G2/VR-5).

---

## 6. OQ-S — Grade through monomorphization (M-844)

### 6.1 Maintainer decision (faithful)

*Yes — each monomorphized instantiation carries its own guarantee-tag context.*

RFC-0019 §4.4 describes dictionary-passing (the recommended elaboration strategy) and
the optional monomorphizing AOT specialization pass (outside the trusted kernel). RFC-0018
§4 defines the graded typing judgment `Γ ⊢ e : τ @ g`. OQ-S asks whether a generic
function instantiated at two different grade levels produces two distinct, grade-specific
checked contexts.

### 6.2 Grounded analysis

**T26.18 — How RFC-0019 §4.4 and RFC-0018 §4 compose.**

Under dictionary-passing (RFC-0019's preferred elaboration): a generic function
`fn combine<A, G1, G2>(x: A @ G1, y: A @ G2) -> A @ meet(G1, G2)` is elaborated to an
L1 function taking `G1` and `G2` as explicit guarantee-level parameters. At a call site
where `G1 = Empirical` and `G2 = Declared` are statically known (stage 1+ per RFC-0019
§4.7), the dictionary/type application specializes the function: the call site produces
`A @ Declared` (the `meet`), and the graded judgment for *this call site* is
`Γ ⊢ combine(x, y) : A @ Declared`. A different call site with `G1 = Exact, G2 = Exact`
produces `Γ ⊢ combine(x, y) : A @ Exact`. The two call sites carry **distinct graded
typing contexts** — they are different instantiation points with different `Γ` entries.

Under the **monomorphizing specialization pass** (RFC-0019 §4.4's optional AOT path,
outside the trusted kernel): the generic is instantiated to two separate concrete functions
`combine_Empirical_Declared` and `combine_Exact_Exact`. Each carries its own graded
typing context because the `meet` computation resolves at compile time to distinct lattice
elements (`Declared` vs `Exact`). The graded typing judgment applies to each concrete
instantiation independently. The pass is "outside the trusted kernel" (RFC-0019 §4.4) —
it is a performance optimization, not the normative elaboration — so the correctness
argument for graded types rests on the dictionary-passing path, not the monomorphization
pass. (`Proven`-modulo-stage-1b-landing — the dictionary-passing argument follows directly
from RFC-0018 §4's graded context rules and RFC-0019 §4.7's guarantee-polymorphism design;
the monomorphization pass correctness is `Declared` pending the pass's implementation.)

**Content-addressed identity implication.** Under ADR-003 (Unison-style content-addressed
identity), a function's identity hash includes its type (and grade annotations). Two
instantiations of `combine` at different grade pairs are therefore **distinct content-
addressed definitions** — they are not the same function at two dispatch points, but two
separate registry entries with different hashes. This is the correct behavior under OQ-S's
decision: each instantiation carries its own guarantee-tag context, and the content-address
enforces that distinct contexts produce distinct identities. A downgrade from `Exact` to
`Declared` at a call site is not a silent coercion; it is a distinct content-addressed
instantiation. (`Declared` — the content-address implication is this record's analysis;
it follows from ADR-003's content-addressing discipline applied to graded types.)

**Stage-1b interaction.** Stage 1b (grade polymorphism — RFC-0018 §9, post-ratification)
will add grade variables quantified over `G` to the type system. At stage 1b, a single
generic definition carries a grade-polymorphic type and its instantiations are the concrete
grade applications. OQ-S's decision — that each instantiation carries its own context —
is consistent with stage 1b: the polymorphic definition is one content-addressed entry;
each instantiation is a specialization. The stage-1b elaborator must produce stage-1a-
graded terms for each instantiation, maintaining the distinct graded contexts the OQ-S
decision mandates. Recording this as a requirement for the stage-1b RFC author, not a
blocker for M-844. (`Declared`.)

### 6.3 Recommended next steps

**M-844** deliverables (proposed, `Declared`):

1. Record the OQ-S decision in RFC-0018 (append-only, as a dated resolution following
   the existing §8 R18-Q1..R18-Q5 pattern): "R18-Q6 (OQ-S): each monomorphized
   instantiation carries its own graded context; two instantiations at different grade
   pairs are distinct content-addressed registry entries."
2. Add a test in `crates/mycelium-l1/tests/` (or `src/tests/`) that instantiates one
   generic function at two different grade pairs and asserts that the resulting types
   are distinct — verifying the property is checked by the stage-1a graded judgment, not
   merely documented.
3. Record the content-address implication (T26.18, final paragraph) as a note in
   ADR-003's changelog (append-only: "graded instantiations produce distinct
   content-addressed entries — consequence of OQ-S, recorded 2026-06-29").
4. For the stage-1b RFC: include OQ-S's decision as a requirement in the RFC's
   §Scope — stage-1b's grade-polymorphic elaboration must produce distinct-graded stage-1a
   terms per instantiation.

---

## 7. OQ-F — Cross-reference (M-832)

OQ-F (VSA multi-hop compositional `Proven` bounds — whether there exists a tractable
subset of compositional VSA programs admitting `Proven` capacity bounds) is being pursued
via the GPU experiment track at `experiments/mycelium_experiments/vsa_bounds/` (M-832).
This record does not duplicate that work. The OQ-F question is grounded in RFC-0009 §5
(resonator factorization at most `Empirical`; `Proven` bounds are an open research
question) and the RFC-0003 matrix HRR/FHRR rows. The GPU experiment is the designated
vehicle for gathering the `Empirical`-to-`Proven` upgrade evidence.

Any positive result from the GPU experiment that yields a `Proven` capacity bound for a
tractable VSA program class should be:

1. Recorded in a new research record citing the experiment artifact.
2. Fed back into RFC-0009 §5 as a dated resolution (append-only).
3. Considered for the OQ-C composition rule (T26.8): if VSA capacity bounds become
   `Proven`, the composed bound for a Dense→VSA→Dense pipeline could inherit a `Proven`
   tag on the capacity component while keeping the Dense-rounding component at `Empirical`
   — a split-bound design the E2-1 rule should accommodate.

---

## 8. Key sources

- **RFC-0018** §4 (graded typing judgment, Design A rules, G-Const…G-Swap/G-Fix),
  §8 (R18-Q1…R18-Q5), §9/§10 (stage 1b/2 future work), §11 (mechanization obligation).
- **RFC-0019** §4.4 (dictionary-passing vs monomorphization), §4.7 (guarantee polymorphism
  staged interaction), §4.9 (polymorphic typing judgment).
- **RFC-0027** §7 (RC mechanism), §8 (guarantee tags), §11 (OQ-1..OQ-6), §12 (three-layer
  hybrid pointer, honest-scope note on AOT-env-machine gap).
- **RFC-0008** RT7 (structured lifetimes, scope-exit, reclaim), §4.1 table (`graft`/
  `substrate`/`reclaim` vocabulary).
- **RFC-0002** §2–§5 (swap certificates, `SwapError::ApproximateSource`, E2-1 placeholder).
- **RFC-0014** (bounded-effects recovery, effect budget; §10.1 D1 Proposed).
- **ADR-003** (Unison-style content-addressed identity).
- **ADR-010** (verified numerics, ε/δ certificate structure).
- **ADR-011** (BoundBasis-Is-Universal — basis as a required field on every bound).
- **DN-32** (three-layer hybrid memory architecture; §2.1 Layer 1 affine primary; §3
  sibling-concurrent coupling; §4 goals including "stupid easy" ergonomics).
- **DN-33** (MEM-4 additive principle; §8.1 Q1/Q2 resolutions).
- **DN-35** (env-machine reclamation direction; increment sequence).
- **DN-59** (G3 reclamation resolution; OQ-5 `substrate`/`graft` deferred; R1/R2 split).
- `research/09` (T9.1–T9.8: the noninterference proof sketch, Design A/B distinction,
  purity precondition T9.6, the Lean/LH mechanization as follow-up).
- `crates/mycelium-cert/src/lib.rs` (`SwapError::ApproximateSource`, E2-1 marker).
- `crates/mycelium-cert/src/dense.rs` (the E2-1 refusal guard).
- `docs/planning/E12-Memory-Model-Build-Plan.md` (MEM-1/2/3 built; Increment 3 forward).

**External sources (cited, not verified in-repo):**

- Vazou et al., *Refinement Types for Haskell* (ICFP 2014) — LH expressivity bounds.
- Reinking et al., *Perceus: Garbage Free Reference Counting with Reuse* (PLDI 2021).
- Moore, *Interval Analysis* (1966) — absolute additive error propagation.
- Higham, *Accuracy and Stability of Numerical Algorithms* (SIAM 2002, 2nd ed.) — relative
  error composition.
- De Figueiredo and Stolfi, *Affine Arithmetic: Concepts and Applications* (2004) — AA.
- Barendsen and Smetsers, *Uniqueness Type Inference for Functional Languages* (1993).
- The Mathlib4 / Lean 4 documentation — dependent-type IFC proof infrastructure.

---

## 9. Honest-uncertainty register

- **All mechanization path recommendations (T26.2–T26.4) are `Declared` or `Empirical`.**
  No path has been prototyped in this record; feasibility is assessed from the published
  literature and the existing LH precedent, not from in-repo mechanization work.
- **The bound composition rule (T26.7–T26.10) is a `Declared` design proposal.** It has
  not been implemented or property-tested. The over-bounding limitation (T26.9) is
  identified but its practical severity (how often a pipeline becomes `Empirical` rather
  than `Proven`) is unknown until the property tests are written.
- **The "stupid easy" ergonomics claim (T26.11–T26.14) is `Declared` at the system level.**
  Individual components are `Empirical` (Perceus performance on Koka, Rust move semantics)
  but the Mycelium-specific realization has not been built (MEM-4 Increment 3 is the
  forward work item).
- **The substrate/hypha reclamation candidate dispositions (T26.16) are `Declared` design
  options.** They are grounded in the corpus invariants (LR-8, RT7, RFC-0014) but none has
  been selected; the `graft` RFC is the vehicle for selection.
- **The OQ-S content-address implication (T26.18) is `Declared`.** It follows from
  ADR-003's design by reasoning but has not been stated normatively or verified by a test.
- **Absence of a found prior-art precedent for lattice-graded NI × machine-checkable per-
  value runtime certificates** (inherited from T3.2 / research/09 T9.5) remains flagged.
  T26.1 confirms the mechanization target is well-defined; it does not resolve the
  precedent-absence finding.

---

## Meta — changelog

| Date | Change |
|---|---|
| 2026-06-29 | Created. R&D planning record for DN-64 §6 OQ-A/C/D/E/S dispositioning. Covers the Lean 4 mechanization recommendation for M-827 (T26.1–T26.5), the absolute-additive bound composition rule for M-829 (T26.6–T26.10), the Layer-1 ergonomics build plan for M-830 (T26.11–T26.14), the substrate/hypha reclamation specification questions for M-831 (T26.15–T26.17), and the OQ-S graded-instantiation confirmation for M-844 (T26.18). OQ-F (M-832) cross-referenced, not duplicated. All proposals `Declared`; `Empirical` where grounded in cited external results. Append-only. |
