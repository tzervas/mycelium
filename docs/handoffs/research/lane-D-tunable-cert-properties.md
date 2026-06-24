# Lane D — Properties of Tunable Certification (RFC-0034)

| Field | Value |
|---|---|
| **Lane** | D — Tunable Certification Properties |
| **Status** | Draft (handoff, 2026-06-24) |
| **Confidence** | Mixed — see per-section tags (VR-5) |
| **Author** | Research agent (Sonnet-4.6), worktree branch |
| **Corpus anchors** | RFC-0034 §3–§7, §11, §13; ADR-032; RFC-0001 §3.4/§4.7; `crates/mycelium-core/src/cert_mode.rs`; `crates/mycelium-core/src/guarantee.rs`; DN-29 |
| **External citations** | See §3 |

---

## §1 Questions

This report addresses three questions commissioned for the E21-1 implementation cycle:

**Q-A.** Can a `fast` (unverified) mode still carry *useful structural provenance*? What survives without running trials or proofs?

**Q-B.** What are the *precise conditions* under which switching `fast → certified` restores full guarantees *without* a silent soundness gap at the mode boundary — the "gradual guarantee" analogue?

**Q-C.** Is composing a `fast` value into a `certified` computation sound, and how must the boundary be tagged?

---

## §2 Mycelium-Corpus Grounding

### 2.1 The guarantee lattice and what "structural provenance" means

`Declared`: strength tag confidence is **`Empirical`**. The lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` (RFC-0001 §3.4; `guarantee.rs`) is a **meet-semilattice**: `Exact` is top (identity of `meet`), `Declared` is bottom (absorbing). The lattice is proven algebraically over all 4×4×4 triples — commutativity, associativity, idempotence, identity (`guarantee.rs` tests, exhaustive — `Empirical` for the code, `Declared` for the claim that no missing cases exist since the value space is finite).

*Structural provenance* is the subset of the lattice that `fast` can supply without running the expensive machinery. RFC-0034 §7 and `cert_mode.rs::gate_guarantee` make this explicit:

```rust
CertMode::Fast => match intended {
    Exact    => Exact,          // bijection-class swap: no approximation exists → structural
    Proven | Empirical | Declared => Declared,  // anything requiring trials/proofs → floors
}
```

**What survives in `fast` (`Declared`):** the fact that *some* representation change occurred, which swap was applied, input and output type identity, the inspectability trace (RFC-0034 §7: *generation* of the signal is always-on from the middle tier up). This is *non-certified auditability* (DN-29 §6, ADR-032 §Context): you can see what happened and why, but no certificate backs it.

**What survives structurally as `Exact`:** bijective exact swaps — cases where the representation change carries no approximation by construction (e.g. exact binary↔ternary bijections). `Exact` passes `gate_guarantee` untouched because the structural argument requires neither trials nor a checked theorem.

**Confidence (Q-A):** `Empirical` (the gate function is implemented and tested exhaustively in `cert_mode.rs`; the interpretation of what `Declared` means in `fast` is `Declared` pending the runtime mechanism landing, per RFC-0034's own VR-5 posture).

### 2.2 Never-silent mode tag and the floor invariant

RFC-0034 §3 (the three normative invariants) establishes:

1. **Never-silent about mode (G2):** every result carries an explicit `CertMode` tag; cross-mode composition is an *explicit visible event*, never a silent upgrade (RFC-0034 §3.1, §6).
2. **No overclaim (VR-5):** `fast` results never carry `Empirical`/`Proven` — the M-787 invariant, enforced by `gate_guarantee` and verified exhaustively in `cert_mode.rs::fast_never_yields_empirical_or_proven`.
3. **Transparency floor (mode-independent):** inspectability signal generation, memory safety, and Axis-B never-silent failure hold in *every* mode including `fast` (RFC-0034 §3.3, §9, §10).

The floor is the *load-bearing* structural contribution `fast` provides without any certification cost. Point (3) is the direct answer to Q-A: even at minimum depth, the inspectability trace is generated (RFC-0034 §7: "always generated from the middle tier up") — it simply has lean *consumption* by default, not absent generation. This means history is not lost: a developer can dial consumption up mid-session to inspect `fast` runs without a re-run or mode switch.

**Confidence:** `Declared` for the runtime behaviour (the mechanism is not yet implemented; claims are `Declared` per RFC-0034's own VR-5 posture), `Empirical` for the structural model (the gate function and lattice are code-tested).

### 2.3 Mode resolution and the cross-mode composition rule

RFC-0034 §6 states: *"combining a `fast` value into a `certified` computation surfaces the mode boundary; the result cannot silently inherit `certified` strength it did not earn."* This is the normative cross-mode composition rule.

The scoping (`global ⊐ phylum ⊐ nodule`, most-specific-wins, reusing RFC-0012's ambient scoping machinery) is the *declaration* mechanism; cross-mode composition is the *runtime* event. M-790 (issues.yaml) states its acceptance criterion: *"a `fast` value entering a `certified` computation is an explicit, visible event — never a silent upgrade to a strength it did not earn."*

The key structural invariant: **the meet-semilattice composition rule** (`guarantee.rs::propagate`) already forces the result guarantee to be the weakest of all inputs. So if a `fast` value tagged `Declared` enters a `certified` operation whose intrinsic strength is `Proven`, the result is `meet(Proven, Declared) = Declared`. The lattice is the mechanistic ground truth; `CertMode` tags the *context* in which that guarantee was established, the `GuaranteeStrength` tags the *epistemic level* of the claim.

This two-layer tagging (mode-context + guarantee-strength) is the precise answer to Q-B and Q-C.

**Confidence:** `Declared` for the runtime wiring (not yet implemented); `Empirical` for the lattice semantics (exhaustively tested).

---

## §3 External Prior Art

### 3.1 Gradual typing and the gradual guarantee

Gradual typing (Siek & Taha 2006) introduced the **dynamic unknown type `?`** to allow partial annotation. The **gradual guarantee** (Siek, Vitousek, Cimini, Boyland, "Refined Criteria for Gradual Typing," SNAPL 2015, [DROPS](https://drops.dagstuhl.de/storage/00lipics/lipics-vol032-snapl2015/LIPIcs.SNAPL.2015.274/LIPIcs.SNAPL.2015.274.pdf)) formalizes the intuition:

- **Static gradual guarantee:** if a term is well-typed, it stays well-typed when annotations are made *less precise*.
- **Dynamic gradual guarantee:** adding more-precise type annotations can *only* add runtime checks; it does not otherwise change program behavior.

The **boundary** between typed and untyped code is handled by runtime *casts* inserted by the elaboration phase. These casts are the load-bearing enforcement mechanism — without them, unsound values could silently enter the typed context. The dynamic gradual guarantee means `typed → fully annotated` is *monotone* in its ability to find errors: more precision can only catch more things, never break correct programs.

**Relevance to RFC-0034:** the `fast → certified` mode upgrade is the direct analogue. The "gradual guarantee" for Mycelium would state: *switching a nodule from `fast` to `certified` can only engage additional machinery (cert checking, `Empirical`/`Proven` computation); it cannot change the observable result of a computation that was already correct in `fast`* — except by catching a swap that `fast` accepted with a `Declared` tag but that `certified` correctly rejects. This is the condition under which `fast → certified` is *sound without a silent soundness gap*.

**Confidence:** `Empirical` (the analogy is structurally sound; whether RFC-0034's implementation will satisfy the precise criterion is `Declared`).

### 3.2 The blame theorem and boundary accountability

Wadler & Findler, "Well-typed programs can't be blamed" (ESOP 2009, [Northwestern](https://users.cs.northwestern.edu/~robby/pubs/papers/esop2009-wf.pdf); [Springer](https://link.springer.com/chapter/10.1007/978-3-642-00590-9_1)) introduced the **blame calculus**, unifying gradual and hybrid types. The **blame theorem** states: when a boundary cast fails, blame falls on the *more dynamic* (less typed) party — "well-typed programs can't be blamed."

The mechanism is **directed casts** at the typed↔untyped boundary: the boundary is never invisible; cast failure is attributed to exactly one side. The **negative subtyping** decomposition ensures the untyped party's value must survive a check to enter the typed context; if it fails, the untyped party is blamed.

**Relevance to RFC-0034:** the blame theorem is the theoretical grounding for RFC-0034's G2/never-silent requirement at mode boundaries. When a `fast` value enters a `certified` context, the "cast" is the explicit mode-boundary surfacing event (RFC-0034 §6). If the value's `Declared` tag is insufficient for the `certified` computation's invariant, the failure is attributed to the `fast` context — not silently swallowed. The two-layer tagging (`CertMode` + `GuaranteeStrength`) gives Mycelium a richer vocabulary than a binary typed/untyped split: attribution is to the *mode context* of origin, not just to "untyped."

**Confidence:** `Empirical` (structural analogy); the direct mapping to RFC-0034's runtime implementation is `Declared`.

### 3.3 Gradual refinement types

Lehmann & Tanter, "Gradual Refinement Types" (POPL 2017, [ACM](https://dl.acm.org/doi/10.1145/3009837.3009856); [preprint](https://pleiad.cl/papers/2017/lehmannTanter-popl2017.pdf)) extend gradual typing to **logically refined types** — refinements of the form `{x:T | φ(x)}`. The key contributions:

- **Type safety and soundness survive partial annotation.** When refinement specifications are missing, the system maintains standard type invariants; dynamic monitors enforce refinement predicates *at the interfaces* between annotated and unannotated regions.
- **Boundary enforcement:** static checks verify constraints within annotated regions; runtime assertion insertion enforces refinements at code boundaries. Violations surface at the precise boundary, not by silent propagation.
- The **gradual guarantee** holds for refinement types: adding more-precise refinement annotations adds checks, not breakage.

**Relevance to RFC-0034:** Mycelium's `GuaranteeStrength` lattice is structurally analogous to a refinement hierarchy: `Exact` is the fully-refined level, `Declared` is the unrefined level. The "gradual refinement" framing suggests that `fast`'s `Declared` tags are a valid *partial refinement* of `certified`'s `Proven`/`Empirical` tags — not a loss of soundness, but a deferral of proof obligation, disclosed by the tag.

**Confidence:** `Empirical` for the structural parallel; `Declared` for any claim that RFC-0034's system satisfies gradual refinement type soundness formally.

### 3.4 Proof-carrying code (PCC)

Necula, "Proof-Carrying Code" (POPL 1997, [ACM](https://dl.acm.org/doi/10.1145/263699.263712); [Cornell slides](https://www.cs.cornell.edu/courses/cs7194/2019sp/slides/necula.pdf)) attaches a *checkable certificate* to compiled code. The key architecture:

- **Certifier (compile-time):** produces the proof/certificate alongside the code.
- **Checker (host, pre-execution):** verifies the certificate against a safety policy.
- **Separation:** the host never trusts the certifier, only the certificate format and the policy.

RFC-0002's swap-certificate and the M-210 checker are the direct PCC analogue in Mycelium. The `certified` mode engages PCC-style checking per swap (M-788: emit + check); `fast` skips both steps, producing an uncertified execution.

**The composition problem in PCC:** If certified module A calls uncertified module B, A's certificate cannot vouch for B's behaviour. The standard PCC solution is a **safety policy** that governs what uncertified code may do — typically a *safe subset* restricted to memory-safe, non-crashing operations. Mycelium's analogue is the Axis-B / never-silent failure floor: even `fast` code is memory-safe and never silently out-of-range (RFC-0034 §3.3, §9, §10) — this is exactly the "safe subset" a `fast` component is guaranteed to respect even without a certificate.

**Confidence:** `Empirical` for the structural analogy to PCC; `Declared` for the claim that RFC-0034's runtime mechanism satisfies PCC-style compositional guarantees.

### 3.5 Sound gradual verification

Dardinier et al., "Sound Gradual Verification with Symbolic Execution" (2023, [arXiv:2311.07559](https://arxiv.org/pdf/2311.07559)) prove soundness for a gradual verification system using symbolic execution. The system:

- Supports **explicitly partial specifications** verified by a mix of static and dynamic checks.
- Inserts **runtime checks at the boundaries** between fully-specified and partially-specified code to protect verified invariants.
- Proves the approach sound — dynamic boundary checks are the *load-bearing* guarantee mechanism.
- Found and fixed a soundness bug in an existing gradual verification tool during the formalization.

**Relevance:** this confirms the general pattern from the literature: *mode boundaries require explicit runtime enforcement to be sound*. The "sound" property requires that the unverified region cannot silently violate the verified region's invariants. For RFC-0034, this means the cross-mode boundary event (RFC-0034 §6, M-790) is not optional aesthetics — it is the mechanism that makes mode composition sound.

**Confidence:** `Empirical` (the paper is published and peer-reviewed; the mapping to RFC-0034's design is `Declared`).

### 3.6 Design by Contract (DbC)

Meyer, "Applying Design by Contract" (IEEE Computer, October 1992; [Eiffel overview](https://www.eiffel.com/values/design-by-contract/)) established the precondition / postcondition / invariant model. Runtime monitoring is the dynamic enforcement mechanism. The **composition rule** in DbC: a caller's postcondition is the callee's context; if the callee weakens its precondition, the caller's contract may be violated silently unless the boundary is checked.

**Relevance:** RFC-0034's `Declared` floor in `fast` is analogous to a DbC "contract-free" call: the operation's postcondition is asserted but not checked. The guarantee that survives is the *structural* one (G2: the operation occurred, is visible, is tagged) — the equivalent of the function executing without undefined behaviour, without the postcondition being machine-verified. Memory safety (RFC-0034 §9) is the analogue of never violating the baseline DbC *class invariant* (memory integrity) — the floor that DbC also requires even of unchecked methods.

**Confidence:** `Empirical` for the structural parallel.

---

## §4 Boundary-Soundness Analysis and Implications for E21-1

### 4.1 What "soundness at the mode boundary" means for RFC-0034

Drawing the prior art together: a mode-boundary crossing is *sound* in RFC-0034's model if and only if:

1. **No silent upgrade.** A `fast`-tagged value entering a `certified` computation does not gain `Certified` mode provenance or an `Empirical`/`Proven` guarantee it did not earn. The result's guarantee is `meet(result_intrinsic, Declared) = Declared` (since `Declared` is lattice-bottom). This is mechanically enforced by `gate_guarantee` in `fast` mode and by the meet-propagation rule in `guarantee.rs`. (`Empirical` confidence: lattice is tested exhaustively.)

2. **Explicit boundary event.** The mode tag on the incoming value is inspectable (RFC-0034 §3.1, §6; G2). The `certified` computation *knows* it received a `fast` value; the `CertMode` tag survives the boundary. This is the RFC-0034 analogue of gradual typing's cast / blame calculus's blame-assignment: the origin-mode of the value is always attributable. (`Declared` confidence for runtime wiring; `Empirical` for the structural model.)

3. **The floor invariants hold unconditionally.** Memory safety (Rust kernel), never-silent failure (Axis B), and signal generation are *not gated* by `CertMode`. So even a `fast` value entering a `certified` computation arrived via memory-safe operations that did not silently discard out-of-range events. This is the "safe subset" guarantee — the PCC analogue — that makes cross-mode composition safe to permit at all. (`Empirical` confidence: the floor is structural from the Rust kernel and Axis-B enforcement.)

4. **`fast → certified` upgrade is monotone in found problems.** Switching a nodule from `fast` to `certified` engages the full cert machinery; it cannot *miss* a swap failure that `certified` would catch, since `certified` always emits + checks (M-788). The only case where `fast → certified` produces a *different observable result* is when `certified` catches a genuinely incorrect swap that `fast` accepted with a `Declared` tag. This is the *desired* behavior, not a soundness gap. (`Declared` confidence for the runtime claim; `Empirical` for the structural argument from the gate function.)

### 4.2 The single condition for no silent soundness gap

**Condition:** The mode boundary is sound (no silent gap) if and only if the *consumer* of a `fast` value never uses that value's lattice position as evidence to satisfy a `certified` computation's trust requirement without re-computing at `certified` depth.

In Mycelium's model this is enforced structurally by two independent mechanisms:

- The **meet propagation rule** (`guarantee.rs::propagate`) makes it algebraically impossible for a `Declared`-tagged input to yield a `Proven`/`Empirical` output — meet is monotone-downward, absorbing at bottom. This is a *semantic* guarantee.
- The **`gate_guarantee` gate** prevents any `fast`-mode operation from emitting `Empirical`/`Proven` in the first place. This is an *operational* guarantee at the production site.

There is therefore *no need for a runtime cast at the boundary in the gradual-typing sense* — the lattice *is* the enforcement mechanism. A `fast` value crossing into a `certified` computation degrades the result's guarantee strength to `Declared` automatically, via the semilattice meet. This is structurally stronger than gradual typing's cast-based approach: rather than inserting a runtime check that may fail, the lattice simply floors the result honestly.

**The one gap case — rejection:** if a `certified` computation *rejects* a `Declared`-tagged input as insufficient (a function with a contract requiring `Empirical` or `Proven` inputs), the rejection must be *explicit and never-silent* (G2). RFC-0034 §6 mandates this; M-790's acceptance criterion includes it. Whether the runtime implementation makes this a type-level static error vs. a runtime `Result::Err` is *left open* in RFC-0034 (deferred to the TDD cycle, M-790). The gradual verification literature (§3.5) strongly suggests *dynamic* enforcement at the boundary is required for full soundness — static analysis alone cannot always know at compile time what mode a value was produced under when mode is a runtime configuration (`@certification` is source data, but cross-phylum mode is resolved at runtime).

**Confidence for §4.2:** the structural argument is `Empirical` (the lattice laws are tested); the specific claim that the lattice enforcement is *sufficient* without additional runtime checks is `Declared` pending M-790.

### 4.3 What `fast` structural provenance carries — a precise inventory

| Survives in `fast` | Guarantee | Basis |
|---|---|---|
| That *some* swap occurred | `Declared` | inspectability trace always generated ≥ middle tier (RFC-0034 §7) |
| *Which* swap was applied | `Declared` | trace carries swap identity |
| Input and output `Repr` identity | `Declared` | value carries `Repr` in `Meta` regardless of mode |
| Memory safety of the result | structural (`Exact`-class) | Rust kernel + no raw pointers on surface (RFC-0034 §9) |
| Never-silent failure (out-of-range → `Option`/error) | structural (`Exact`-class) | Axis-B enforced in every mode (RFC-0034 §10) |
| Spore identity of the producing code | structural (`Exact`-class) | compile-time hash, mode-independent (RFC-0034 §8, ADR-003) |
| Bijective exact swap — no approximation | `Exact` | `gate_guarantee(Exact) = Exact` |
| Approximation bound tightness | — | **not available** in `fast`; requires trials (`Empirical`) or proof (`Proven`) |
| Certificate existence | — | **not available** in `fast`; `balanced` emits unchecked, `certified` emits + checks |

**Confidence:** `Empirical` for the gate function behavior (tested exhaustively); `Declared` for the runtime inspectability trace claim (mechanism not yet implemented).

### 4.4 Implications for the E21-1 runtime mechanism

**Implication 1 (M-790, cross-mode composition):** The mode tag on a value must be *preserved through composition* — it is evidence of origin context, not just a display label. When a `fast` value is used in a `certified` computation, the result's guarantee strength must be floored by the meet (to `Declared`), and the origin `CertMode` must remain attributable. The implementation *must not* silently discard the origin `CertMode` tag. This is stated in M-790's acceptance criterion; it is `Declared` as code.

**Implication 2 (M-788, cert-gating and chain provenance):** A `certified` computation that receives a `fast`/`Declared` input and continues produces a `Declared` result — not a `certified` one. A swap certificate emitted in this case (M-788: `balanced` emits unchecked; `certified` emits + checks) certifies the *swap step*, not the *full chain from inputs*. This chain-provenance gap must be visible in the certificate or flagged; a certificate whose *step* is correct but whose *inputs* were `Declared` is a partial certificate, not a full one. This is the open PCC-composition question (OQ-1 below).

**Implication 3 (M-794, conformance gate):** The cross-mode *negative* test cases are mandatory: a test that a `certified` computation receiving `fast`/`Declared` input does *not* silently upgrade the output guarantee to `Proven`/`Empirical` is as important as a test that `certified` checking works when inputs are fully `certified`. The existing all-on suite implicitly assumed all inputs were `certified`; M-794's DoD explicitly requires the cross-mode negative cases.

---

## §5 Open Research Questions

The following questions are open in the sense that the Mycelium corpus does not yet settle them, and the external literature does not cleanly resolve them for Mycelium's specific model. These are not defects in RFC-0034; they are design decisions that the E21-1 TDD cycle must resolve.

**OQ-1 (chain provenance — `Declared` confidence).** When a `certified` computation consumes `fast`/`Declared` inputs and emits a swap certificate (M-788), what does the certificate certify — the swap step alone, or the full input chain? If only the step, how does a downstream auditor know the *inputs* were `fast`-provenance? The PCC literature handles this via *proof obligations* that propagate through composition; RFC-0034 does not specify chain provenance semantics for certificates. This should be resolved before M-788 ships.

**OQ-2 (static vs. dynamic boundary rejection — `Declared` confidence).** Should a `certified` function that requires `Empirical`/`Proven` inputs *statically* reject a `fast`/`Declared` value at the call site (type-level), or *dynamically* at runtime (`Result::Err`)? The gradual verification literature (§3.5) shows runtime enforcement is required for soundness when mode is a runtime configuration (the static compiler cannot always determine mode at compile time for cross-phylum composition). But static rejection provides better ergonomics and earlier feedback. Both mechanisms may be needed: static where mode is statically known, dynamic where it is not.

**OQ-3 (`balanced` epistemic status — `Declared` confidence).** `balanced` mode emits swap certificates but does not check them (RFC-0034 §5). The precise epistemic status of an *unchecked* certificate relative to a checked `certified` certificate is not formalized. Can a `balanced` certificate be used as partial evidence in a subsequent `certified` check? What `GuaranteeStrength` tag should such partial evidence carry? The lattice does not have a dedicated `balanced` position; `balanced` results likely fall under `Declared` (since the certificate is emitted but not proven). Worth resolving before M-788 lands.

**OQ-4 (sub-nodule mode granularity — deferred by RFC-0034 §14).** RFC-0034 §6 and §14 explicitly defer per-op granularity (the `thaw`-style per-op override). The gradual typing analogy suggests sub-nodule annotation is natural. The question of whether the mode tag on a value should track the *per-op* mode or the *nodule-ambient* mode at time of production affects the precision of boundary attribution (OQ-2 and OQ-1). Left open by RFC-0034 as a YAGNI deferral.

**OQ-5 (mode-tag stability under concurrent composition — open, not addressed).** In a concurrent hypha model (`colony`/`hypha` — ratified names but not yet lexed), a value produced `fast` in one hypha may be consumed `certified` in another. The mode of the consuming context may be determined at a different time than the producing context. Does the mode tag on a value-as-message remain stable across hypha boundaries? Is the never-silent mode event implementable lock-free in a concurrent setting? RFC-0034 discusses scoping at nodule/phylum level only; concurrent mode composition is not addressed.

---

## Changelog

| Date | Status | Note |
|---|---|---|
| 2026-06-24 | **Draft** | Initial research report — Lane D, commissioned for E21-1. Covers Q-A/Q-B/Q-C, corpus grounding (RFC-0034/ADR-032/RFC-0001/`cert_mode.rs`/`guarantee.rs`/DN-29), external prior art (gradual typing gradual guarantee, blame calculus, gradual refinement types, proof-carrying code, sound gradual verification, Design by Contract), boundary-soundness analysis, and 5 open questions. `Declared` overall for runtime claims; `Empirical` for structural/lattice-level claims. |
