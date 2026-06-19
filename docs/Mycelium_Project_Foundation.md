# Mycelium — Project Foundation

**Status:** Foundation — design corpus Accepted; **Rust-first implementation underway** (Phases 0–3, 5, 7 complete; 4, 6, 8 in progress). Research: eleven passes recorded (`research/01…11`).
**Revision:** r4 — records the **Rust-first implementation** status: the kernel + reference interpreter + certified swaps + the 23-crate standard library have landed (Phases 0–3/5/7 complete, 4/6/8 in progress); per the honesty rule the stdlib specs read *"implemented (Rust-first), pending ratification,"* and self-hosting (M-502) is not yet established. Extends the §6 roadmap with Phases 4–8 and refreshes §10. **Design decisions are unchanged (append-only)** — this is a status/roadmap refresh, not a decision change.
**Revision history:** r3 — integrates the follow-up research findings: ratifies ADR-010 (two bound kernels) and ADR-007 (Rust kernel + MLIR); records the **KC-1 pass** (proven VSA bundling bounds exist); marks RFC-0001…0005 Accepted and DN-01 Resolved (packing → schedule-staged); de-risks RR-1 and adds residual risks RR-13/RR-14. · r2 — added the execution-model and VSA-packaging decisions (ADR-008, ADR-009) plus a first-class toolchain treatment (§5.8); folded hybrid-compilation and tooling constraints into Requirements (§3) and the Roadmap (§6).
**Date:** June 08, 2026
**Working name:** Mycelium (formerly *Verid* — provenance note; sole remaining reference)
**Source of truth for claims:** the prior-art *Survey & Synthesis* and the follow-up *Research Findings* (T0/T1/T2). Inline references use those labels (Areas, **G1–G11**, **A–E**, **R1–R8**, **T0.x/T1.x/T2.x**) so every assertion is traceable. The r2/r3 additions are *engineering decisions* grounded in those findings, not new prior art.

---

## 1. Project Charter & Vision

**Mission (one sentence).** Mycelium is a programming language that makes the *encoding and representation of information* a transparent, first-class, formally-auditable artifact — unifying binary, balanced ternary, dense embeddings, and sparse/dense VSA under one metadata-native model with explicit, verifiable representation swaps, intelligible to both human programmers and AI agents.

**Core value proposition.** The survey established that no existing system unifies even two of {binary, balanced ternary, dense embedding, sparse/dense VSA} as *co-equal, first-class* substrates with verifiable inter-conversion; the four-way union is unprecedented (survey TL;DR; **G1**). Every *component* capability is mature and separable — MLIR's progressive inspectable lowering, Unison's content-addressed AST-as-truth with names-as-metadata, Arrow's self-describing custom metadata, dependent/refinement types, the verified-numerics toolchain (Gappa/FPTaylor/Rosa-Daisy/Flocq), and the Kleyko et al. VSA taxonomy. Mycelium's contribution is therefore *integrative*, and its single biggest unsolved problem — the one no existing system solves — is **metadata-native, non-opaque, formally-bounded representation swapping across fundamentally different algebras**: binary↔ternary is bijective and provable, while ↔VSA/embedding is inherently lossy/approximate and must carry an explicit, inspectable error/crosstalk/probability bound rather than a hidden approximation (survey cross-cutting **A.1**, **B**; **G1**, **G4**, **G7**).

---

## 2. Scope Definition

### 2.1 In-scope substrates & capabilities

| Capability | Phase intro | Survey grounding |
|---|---|---|
| Binary computation/data (first-class) | P1 | baseline; Area 1 |
| Balanced ternary as a **logical/semantic** substrate {−1,0,+1} on binary hardware, forward-compatible with native ternary HW | P1 | Area 3; **R7**; **G3** |
| Dense embedding values (typed, dim-tracked) | P2 | Area 2/4 |
| Sparse & dense VSA hypervectors (MAP/BSC/HRR/FHRR/SBC), **exposed via an optional core submodule** (ADR-008) | P2 | Area 2; torchhd taxonomy; NFR-6 |
| Representation **metadata** carried with values (paradigm, sparsity, holographic props, reconstruction info, provenance, guarantee-strength) | P1→P2 | Area 4 (Arrow); **G2** |
| **Explicit, inspectable** representation swaps with certificates | P2 | cross-cutting **A.1**; **G9** |
| Progressive, inspectable lowering (no opaque passes) | P1 | Area 1 (MLIR/CompCert); **R3** |
| Content-addressed code identity + names-as-metadata | P1 | Area 1 (Unison) |
| Typed error/crosstalk/probability bounds on approximate ops | P1→P2 | **B**; **R1**, **R5** |
| **Hybrid execution**: AOT-compiled native executables (preferred for stable components) + interpreter/JIT (dev, exploration, dynamic VSA) (ADR-009) | P1 | Area 1 (MLIR multi-backend, MetaOCaml staging); **R3** |
| **Minimal toolchain surface**: language server (LSP), linter, formatter, build system — load-bearing for human + AI use | P1 (design) → P3 (full) | Area 6 (semantic feedback); cross-cutting **C** |

### 2.2 Explicitly out-of-scope for Phases 1–2 (with justification)

| Out-of-scope (P1–P2) | Justification (survey-grounded) |
|---|---|
| **Native balanced-ternary hardware codegen** | No competitive ternary hardware exists; MVL is industry-standard only in flash, not logic; even Setun was a 4-state magnetic-core emulation (Area 3). Treat ternary as logical-now (**R7**, **G3**); keep a forward-compatible value-semantics contract only. |
| **Resonator-network factorization in the core** | Factorization is convergence-*heuristic*, not guaranteed; suffers limit cycles and capacity limits (Area 2; survey Caveats). Compositional queries needing factorization are deferred and, when introduced, carry only probabilistic guarantees (**G4**). |
| **A novel surface syntax tuned purely for machine-optimality** | LLM performance is dominated by training-data exposure (Python favored); novel syntax forfeits this (**C**, **G10**). Surface-syntax novelty is gated behind the P0 LLM-leverage experiment. |
| **Deep holographic invariants requiring full dependent-type proofs** | Some holographic properties may not be exactly statable; dependent-type proofs about approximate kernels hit the real-vs-float kernel-opacity gap (Area 5; **G8**). Start with refinement-type-checkable *bounds*, not deep proofs. |
| **General-purpose stdlib, package manager, IDE** | YAGNI for a foundation phase; the integration risk, not tooling, is the open problem (survey TL;DR). |
| **"Auto-magic" representation selection without a reified policy** | Any implicit selection is exactly the black box Mycelium forbids (**G2**); only *reified, queryable* selection policies are in scope. |

### 2.3 Success criteria (measurable, testable)

- **SC-1 (round-trip integrity).** Binary↔ternary swaps satisfy `decode ∘ encode = id` for 100% of a property-test corpus *and* admit a machine-checkable proof of bijectivity (Area 5; **G3**).
- **SC-2 (honest bounds).** Every exposed approximate op (bundle, unbind+cleanup, embedding-similarity) ships a stated capacity/crosstalk/probability bound whose *measured* behaviour falls within the stated bound across ≥10⁴ randomized trials, with the bound tagged `Proven | Empirical` and a citation (**B**, **G5**; survey Caveats).
- **SC-3 (no silent swaps).** Static + runtime invariant: zero representation swaps occur without producing an inspectable certificate (auditable; cross-cutting **A.1**, **G9**).
- **SC-4 (no opaque lowering).** Every lowering step is dumpable and diffable at each stage (MLIR-grade), with a documented per-stage semantics (Area 1; **R3**).
- **SC-5 (dual intelligibility).** (a) A human reviewer can reconstruct what a swap did from its certificate alone; (b) an AI agent generates syntactically valid, type-checking Mycelium for ≥X% of a fixed benchmark set (X set by the P0 LLM probe baseline, **G10**).

### 2.4 Kill / major-redirect criteria

- **KC-1.** If the P0 bundling probe shows *no* core VSA op admits a usefully tight, honestly-statable bound (**R1**'s stated falsifier), VSA leaves the **core** and becomes a clearly-demarcated approximate sublanguage — or the project redirects away from native VSA. **— Checked at the research gate: PASSED; confirmed (build) 2026-06-09.** Proven non-asymptotic bundling bounds exist (Clarkson-Ubaru-Yang 2023; Thomas-Dasgupta-Rosing 2021; **T0.2**), so MAP-I/sparse `bundle` carry honest `Proven` tags. The confirming build is **complete**: the Liquid-Haskell MAP-I `bundle` capacity refinement (`proofs/lh-bundle/`, M-001) type-checks **SAFE** and Z3 discharged all constraints (LiquidHaskell 0.9.8.2 / Z3 4.8.12), ratifying the axiomatized-theorem + checked-instantiation strategy (RFC-0003 §5; ADR-010).
- **KC-2.** If the P0 LLM-leverage experiment shows code-gen/reasoning collapses *irrecoverably* even with projections + semantic feedback (**G10**, **R6**), reweight toward human-primary design or fall back to an embedded DSL in a high-resource host language.
- **KC-3.** If integrative complexity makes the kernel un-auditable by a single domain expert, Mycelium has reintroduced a *meta-level* black box (violating its own mandate); scope must be cut until the kernel is auditable.
- **KC-4.** If verified-swap infrastructure (translation validation) proves so costly that certificates cannot be produced per-swap at acceptable overhead (**G9**; CompCert-level effort caveat), downgrade lossy-swap guarantees from *certified* to *declared-and-property-tested* and document the loss.

---

## 3. Requirements

### 3.1 Functional Requirements (MoSCoW)

**Must**
- **FR-M1.** Represent values as `(payload, representation-descriptor, metadata)` with the representation paradigm reflected in the static type (Area 4; **G1**).
- **FR-M2.** Provide first-class binary and logical-ternary {−1,0,+1} values with balanced-arithmetic identities (negation = digit flip; rounding ≡ truncation) (Area 3).
- **FR-M3.** Provide an explicit `swap` operation that never runs implicitly and always emits a certificate (cross-cutting **A.1**; **G2**, **G9**).
- **FR-M4.** Distinguish, in the type/cert system, **bijective** swaps (binary↔ternary) from **bounded/lossy** swaps (↔VSA/embedding) (cross-cutting **B**; **R2**).
- **FR-M5.** Carry representation metadata *through* lowering without erasure (dimensional-persistence, contra F# units erasure; Area 4/5; **G2**).
- **FR-M6.** Expose lowering stages as inspectable, diffable artifacts (MLIR-grade) (Area 1; **R3**).
- **FR-M7.** Support a **hybrid execution model** — both interpreted/JIT execution and AOT-compiled native executables — with AOT preferred for stable components and interpretation/JIT for development, exploration, and dynamic VSA workloads; *both paths obey the no-opaque-lowering rule* and share one set of stage semantics (ADR-009; Area 1).
- **FR-M8.** Expose VSA through an **optional core submodule**: the core type/metadata/swap machinery natively recognizes the `Hypervector` paradigm, but the VSA *operational algebra* (bind/unbind/bundle/permute/cleanup) is dependency-gated so the kernel stays lean for non-VSA users (ADR-008; NFR-6; **KC-3**).

**Should**
- **FR-S1.** Provide selectable VSA models (MAP, BSC, HRR, FHRR, SBC) with per-model `bind/unbind/bundle/permute/cleanup` and attached bounds (Area 2).
- **FR-S2.** Provide content-addressed definition identity with names-as-metadata (Unison-style) (Area 1).
- **FR-S3.** Provide reified, queryable representation-selection **policies** (so "intelligent" swapping is auditable) (**G2**; cross-cutting D).
- **FR-S4.** Provide a clean-up memory abstraction (item memory / nearest-neighbor) for approximate unbinding (Area 2).
- **FR-S5.** Provide a **minimal toolchain surface** from Phase 1: a language server (LSP) exposing typechecker diagnostics, swap certificates, bound violations, and lowering-stage dumps; plus a linter and formatter. This is the concrete delivery vehicle for the dual-intelligibility goal — the same semantic-feedback surface serves human IDEs and AI co-authors (Area 6; cross-cutting **C**; NFR-2).

**Could** *(exploratory — gated on P0/P1 findings; not committed for Phases 1–2)*
- **FR-C1.** *(exploratory)* Multiple human/machine **projections** of the same artifact (MPS/Unison-style); deferred to Phase 3 and contingent on the projection-ergonomics question (Area 6; **G11**).
- **FR-C2.** *(exploratory)* Resonator-network factorization as an explicitly *probabilistic*, opt-in op; deferred to Phase 3, never in the core, contingent on convergence behaviour (Area 2; **G4**).
- **FR-C3.** BitNet-style packed-ternary acceleration paths (I2_S/TL1/TL2) exposed as inspectable metadata, not hidden lowering (Area 3; **G3**).

**Won't (this phase)**
- **FR-W1.** Native ternary-hardware backends (§2.2).
- **FR-W2.** Implicit/automatic representation selection without a reified policy (§2.2; **G2**).
- **FR-W3.** Exact equality proofs over inherently approximate VSA kernels (§2.2; **G8**).

### 3.2 Non-Functional Requirements

- **NFR-1 (human intelligibility).** Intelligibility of raw machine-optimal forms (packed trits, hypervectors) comes from inspectable *metadata + projections about* those forms, not from the forms themselves (cross-cutting **C**).
- **NFR-2 (AI leverage).** Favor explicitness, strong types, locality, content-addressing, and semantic feedback loops — properties the survey found help *both* humans and machines (cross-cutting **C**; Area 6).
- **NFR-3 (formal auditability).** Everything inspectable; the *policy* that selects a representation is itself a first-class, queryable artifact (**G2**).
- **NFR-4 (performance, current binary HW).** Ternary emulation must use known-efficient packing (BitNet pack-store-load-unpack-compute) with packing exposed as metadata (Area 3; **G3**).
- **NFR-5 (forward compatibility).** The ternary value-semantics contract must preserve {−1,0,+1} semantics and leave a clean mapping to native 3-state hardware later (**R7**).
- **NFR-6 (modularity).** Small auditable kernel (KISS/YAGNI); substrates and VSA models as composable modules with narrow interfaces (SRP/ISP/SoC); swaps depend on representation *abstractions*, not concrete encodings (DIP). *Rationale: a large kernel is itself a black box — see **KC-3**.* The VSA submodule (ADR-008) is the first concrete application of this principle.
- **NFR-7 (execution-path equivalence).** The interpreter is the **executable reference semantics**; AOT- and JIT-compiled output must be observably equivalent to it, validated by the same translation-validation machinery used for swaps (ADR-009; **VR-4**; Area 5 CompCert/translation validation). Two execution paths must never mean two semantics.

### 3.3 Verification & Assurance Requirements

- **VR-1 (must be provable).** Bijectivity of binary↔ternary swaps; well-scoped/well-typed generated code from any lowering stage (MetaOCaml-grade) (Area 5).
- **VR-2 (must be auditable).** Every lowering pass and every swap certificate; the active representation-selection policy (Area 1; **G2**).
- **VR-3 (must be explicit bounds).** Every approximate op carries a typed bound (error ε, crosstalk, or failure-probability δ), tagged `Proven | Empirical`, following the exact-spec-plus-proven-bound pattern (cross-cutting **B**; **R1**, **R5**).
- **VR-4 (translation validation, not whole-engine proof).** Lossy swaps are validated **per-swap** via certificate checking (CompCert/Valex/Daisy-checker model), not by once-proving a swap engine (**R2**; **G9**).
- **VR-5 (honest guarantee-strength).** Where a bound rests on Gaussian-approximation (Frady-Sommer) rather than a proven non-asymptotic result (Clarkson-Ubaru-Yang 2023; Thomas-Dasgupta-Rosing 2021), it must be marked `Empirical` in metadata (survey Caveats; **G5**).

---

## 4. Key Tensions & Resolved Positions

### 4.1 Tension B — exact semantics vs. first-class approximate operations *(primary)*

**Restatement.** "No statistical approximations in core semantics" appears to conflict with native, first-class support for inherently approximate VSA/embedding operations (clean unbinding fails under superposition; capacity is crosstalk-limited) (cross-cutting **B**).

**Working resolution (adopted, see ADR-001).** Redefine the constraint as **"no *hidden/unspecified* approximations."** Every approximate op is modeled as an *exactly specified deterministic function* together with a *proven or declared-and-checkable bound* on its error/crosstalk/failure-probability — the Rosa/Daisy "ideal-real spec + certified error ε" pattern, and the differential-privacy "(ε,δ) as an exact, proven inequality" pattern (Area 5; cross-cutting **B**; **R1**). Under this reading the approximation is fully explicit, inspectable, and auditable (satisfying *no black boxes*) while remaining first-class.

**Honesty conditions (non-negotiable).** (a) Some VSA bounds are only Gaussian-approximate (Frady-Sommer), so the "proven bound" is sometimes an "empirically-validated bound" and **must be tagged as such** (**G5**, **VR-5**). (b) Anything depending on resonator factorization carries at best a *probabilistic* success guarantee (**G4**). (c) Whether the end state is genuinely "no statistical approximation in core" or rather "fully-disclosed statistical approximation" was a *definitional* choice — **now settled** in favour of fully-disclosed approximation (ADR-001; §9), since the bundling probe confirmed honest bounds exist (**T0.2**).

### 4.2 Split verification regime *(adopted, see ADR-002)*

| Swap class | Guarantee | Mechanism | Grounding |
|---|---|---|---|
| binary ↔ ternary | **Bijective**, exact, provable | Round-trip proof + property tests | **R2**; Area 3/5 |
| ↔ dense embedding / VSA | **Bounded / probabilistic** | Per-swap certificate (translation validation); typed ε/δ bound, tagged `Proven \| Empirical` | **R2**, **VR-4**; cross-cutting **B** |

The two regimes are deliberately *not* unified; pretending lossy swaps are exact is forbidden (cross-cutting **B**; survey Caveats).

### 4.3 Other tensions to manage explicitly

- **Tension C (human-intelligibility vs AI-leverage).** Conflict is *localized* to surface syntax and raw representation; mediated by projections + metadata (cross-cutting **C**; **G10**, **G11**). Managed via the P0 LLM experiment and projection design.
- **Tension "intelligent swap vs opacity."** Any policy-driven selection is a candidate black box; resolved by reifying policies as inspectable artifacts (**G2**; ADR-006).
- **Tension "metadata survives lowering vs erasure-for-performance."** Resolved in favor of dimensional persistence (Area 4/5; **G2**), accepting some lowering complexity.

---

## 5. Architecture Anchors & Core Model Sketch

### 5.1 The three anchors and their integration *(see ADR-003)*

| Anchor | What it gives Mycelium | Integration role | Where it falls short (survey D) |
|---|---|---|---|
| **MLIR-style progressive lowering** | Multiple coexisting representations at different abstraction levels; inspectable, diffable passes; open/extensible type system | The **lowering pipeline & multi-substrate IR** | Lowering is *inspectable*, not *proven*; not human-facing |
| **Unison-style content-addressing** | Hash-of-AST identity; names-as-metadata; AST-is-truth | **Code identity, provenance, projection substrate** | Addresses *code*, not *data representations* |
| **Arrow-style self-describing metadata** | Schema + custom field/schema metadata travels with data; faithful round-trips | The **value-level metadata layer** | No encoding-*paradigm* semantics or swap verification |

The integration *is* the unsolved part (survey D / TL;DR): combine Arrow-grade value metadata + Unison-grade content-addressed code/provenance + MLIR-grade inspectable lowering, then add the missing pieces — representation-paradigm metadata, verified swaps, and typed bounds — none of which any anchor provides alone.

### 5.2 Core computational/type/metadata model — candidate sketch

> Illustrative early sketch. **Superseded by RFC-0001 (Accepted)**, which is now the normative Core IR & metadata schema. Two differences to note: RFC-0001 moves ternary **packing out of the type** (it is *schedule-staged* and recorded in `Meta.physical` — see DN-01 / RFC-0004 §5), and it uses the full four-point guarantee lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` (the `guarantee: Proven | Empirical` below is the older two-point form). Retained for historical context; read RFC-0001 for the authoritative model.

A **Value** is conceptually a triple, with the representation paradigm lifted into the type:

```text
Value<Repr, Meta> = { payload: Bits|Trits|Vector,
                      repr:    Repr,
                      meta:    Meta }
```

Representation descriptor (lightweight EBNF):

```ebnf
Repr      ::= Binary  "{" "width:"  Nat "}"
            | Ternary "{" "trits:"  Nat "," "packing:" PackScheme "}"
            | Dense   "{" "dim:"    Nat "," "dtype:"   FloatKind "}"
            | VSA     "{" "model:"  VsaModel "," "dim:" Nat "," "sparsity:" Sparsity "}"
VsaModel  ::= "MAP" | "BSC" | "HRR" | "FHRR" | "SBC" | "VTB"
Sparsity  ::= "Dense" | "Sparse" "{" "active:" Nat "}"
PackScheme ::= "TwoBitPerTrit" | "FiveTritsPerByte" | "I2S" | "TL1" | "TL2"
```

Metadata (the auditable core; Area 4, **G2**, **VR-3/5**):

```text
Meta = { provenance:        ContentHash,           // Unison-style
         guarantee:         Proven | Empirical,
         bound:             Option<Bound>,          // ε / crosstalk / δ
         reconstruction:    Option<ReconInfo>,      // for lossy/holographic forms
         policy_used:       Option<PolicyRef> }     // reified selection policy
```

### 5.3 First-class concepts that must exist in the core

`Bit`, `Trit` ({−1,0,+1}), `Hypervector<Model, Dim>` *(type recognized by the core; operations live in the VSA submodule, ADR-008)*, `Embedding<Dim>`, `RepresentationDescriptor`, `Metadata`, `Provenance(ContentHash)`, `Bound` (error/crosstalk/probability), `SwapCertificate` (`Bijective(proof) | Bounded{ε, guarantee, basis}`), `SelectionPolicy` (reified), `ItemMemory` (clean-up), `LoweringStage` (inspectable), `ExecutionMode` (`Interpreted | Jit | Aot`, ADR-009).

### 5.4 Candidate operation signatures

```text
// Swaps — never implicit; always certified (FR-M3, FR-M4)
swap : Value<R1,M1> -> target:R2 -> policy:SelectionPolicy
     -> Result<(Value<R2,M2>, SwapCertificate), SwapError>

// Ternary/binary — bijective (VR-1)
to_ternary : Value<Binary,_>  -> (Value<Ternary,_>, Bijective)
to_binary  : Value<Ternary,_> -> (Value<Binary,_>,  Bijective)

// VSA primitives (FR-S1); bounds attached per Tension B
bind     : Hv<M,D> -> Hv<M,D> -> Hv<M,D>                       // algebraic
unbind   : Hv<M,D> -> Hv<M,D> -> (Hv<M,D>, Crosstalk)         // approximate
bundle   : [Hv<M,D>] -> (Hv<M,D>, CapacityBound{tag})        // superposition
permute  : Hv<M,D> -> Perm -> Hv<M,D>                         // algebraic, invertible
cleanup  : Hv<M,D> -> ItemMemory -> (Item, Confidence)
factorize: Hv<M,D> -> [Codebook] -> Result<[Item], NoConverge> // resonator; opt-in, probabilistic (FR-C2)
```

### 5.5 Lowering pipeline (sketch)

`Surface (human/AI projections)  →  Core IR (typed, metadata-bearing)  →  Substrate IR (binary | packed-ternary | VSA-numeric)  →  Backend (interpreter/VM · JIT · AOT native codegen)`. Every arrow is a dumpable, diffable stage (SC-4, **R3**); representation metadata persists across all arrows (FR-M5). The pipeline fans out to **multiple backends** at the final stage (next subsection), not a single target.

### 5.6 Execution model & backends *(see ADR-009)*

Mycelium is **hybrid**: one Core IR, several backends.

| Backend | Role | When used | Survey grounding |
|---|---|---|---|
| Interpreter / VM | **Reference semantics** (NFR-7) | Development, REPL, exploration, dynamic/early VSA work | Area 1; Area 5 (reference for translation validation) |
| JIT | Adaptive speed for dynamic workloads | Long-running exploratory or data-dependent VSA pipelines | Area 1 (MLIR multi-backend) |
| AOT native codegen | **Preferred for stable components** | Frozen, spec-ratified, content-addressed components | Area 1 (MLIR→LLVM); **R3** |

"**Stable component**" has a precise meaning here: a definition that is content-addressed (Unison-style, ADR-003), has a ratified spec, and has passed its verification gate — only then is it eligible for AOT compilation. Everything else runs interpreted/JIT. Because the interpreter is the reference semantics, every AOT/JIT lowering is validated against it per **VR-4** / NFR-7 (no-opaque-lowering applies to *all* backends, ADR-009). Native path (**now committed**, ADR-007 / RFC-0004 §2): lower the Substrate IR through an **MLIR** dialect (a `ternary` dialect first) to **LLVM** for AOT (Area 1; **T1.5**); the Rust interpreter remains the reference semantics and trusted base, with MLIR confined to the performance path.

### 5.7 VSA as an optional core submodule *(see ADR-008)*

The decision is to keep VSA *in the core semantics* (so its algebra is coherent and first-class) while *packaging* it as an optional, dependency-gated submodule. The boundary:

- **In the kernel (always present):** the `Hypervector` representation paradigm as a *type slot*, its metadata fields (model, dim, sparsity, bounds), and the swap machinery that targets/sources it. A kernel built without the submodule still type-checks programs that *mention* hypervectors but offers no operations on them.
- **In the VSA submodule (optional):** the operational algebra — `bind/unbind/bundle/permute/cleanup`, model implementations (MAP/BSC/HRR/FHRR/SBC), capacity/crosstalk bound derivations, and (Phase 3, exploratory) factorization.

This satisfies "powerful native VSA" *and* "lean kernel for everyone else" (NFR-6, **KC-3**), and mirrors how torchhd is a separable numeric layer over a host (Area 2). The submodule boundary is itself a first-class design artifact to be specified in an RFC.

### 5.8 Toolchain surface (first-class)

Per the survey, semantic-feedback loops and explicitness help *both* humans and AI (Area 6; cross-cutting **C**), so the toolchain is load-bearing, not a Phase-3 afterthought. The minimal surface (designed in Phase 1, matured in Phase 3):

- **Language server (LSP):** exposes typechecker diagnostics, swap certificates, bound/`Proven|Empirical` annotations, and lowering-stage dumps through one protocol surface consumed identically by human IDEs and AI co-authors (FR-S5; NFR-2).
- **Linter:** structural + semantic lints (e.g., an implicit-swap attempt, an untagged bound, a swap without a `PolicyRef`) — these encode the project's own invariants (SC-3, **G2**).
- **Formatter:** canonical rendering; with content-addressing, formatting is a *projection*, not a mutation of identity (ADR-003).
- **Build system:** manages the stable-vs-experimental split (which components are AOT-eligible), content-addressed caching, and per-swap certificate artifacts.

Designing the LSP protocol surface early is high-leverage: it is the channel through which the dual-intelligibility goal (SC-5) is actually delivered.

---

## 6. Deliverables Roadmap (Phased)

> Tasks expressed to suit a dependency-ordered, priority-tagged Kanban flow.

### Phase 0 — Feasibility probes & foundational specs
**Goal:** de-risk the two existential questions (Tension B, LLM leverage) and close the survey's four coverage gaps before any kernel commitment.
**Status (June 2026): largely complete.** The two research passes resolved P0.3 (coverage gaps closed), delivered the **KC-1 verdict (PASSED)**, and settled the foundational design questions (ADR-010, DN-01, RFCs 0001–0005). P0.4/P0.5/P0.6 are specified in the RFCs and carry into the build. **Remaining P0 items:** the P0.1 *formalization* artifact (the Liquid-Haskell `bundle` instantiation, RFC-0003 §5). **The P0.2 LLM-leverage experiment (KC-2) now has its written verdict: KC-2 verdict recorded (DN-09) = proceed** — the last open P0 probe is closed (2026-06-18). Honest scope: the measured leverage is *weak but recoverable* (not the irrecoverable collapse the kill criterion guards against); the T3.6 rigorous ablation was not run and remains an honest open follow-up (DN-09 §4; VR-5 — "proceed" does not claim the strong Q1 hypothesis). The selected L3 strategy = committed text syntax + co-equal projection layer (M-380).
- **Deliverables:** (P0.1) end-to-end formalization of **one** VSA op — bundling — with a capacity/crosstalk bound in a refinement-type or Rosa/Daisy style (**R5**, **G7**); (P0.2) LLM-leverage experiment, Mycelium surface fragment vs Python-embedded DSL (**R6**, **G10**); (P0.3) targeted survey closing the four coverage gaps — Halide/Dex/Futhark/APL-family, neurosymbolic synthesis IRs, verified probabilistic numerics, Rust VSA/ternary crates (**R8**); (P0.4) binary↔ternary bijective-swap spec + property tests + BitNet packing evaluation (**R7**, **G3**); (P0.5) reified selection-policy prototype (**G2**); (P0.6) **execution-model + LSP-surface spec sketch** — define "stable component," the interpreter-as-reference contract (NFR-7), the backend fan-out, and the minimal LSP semantic-feedback surface (ADR-008, ADR-009; §5.6–5.8).
- **Success metrics:** SC-1 (P0.4), SC-2 partial (P0.1), an LLM baseline X for SC-5b (P0.2), a written verdict on KC-1 and KC-2, and an agreed execution-model/toolchain spec sketch (P0.6).
- **Major risks:** bundling admits no usefully tight bound (→ KC-1); LLM collapse (→ KC-2); toolchain/backend scope balloons (→ RR-11, manage via a minimal Phase-1 surface).

### Phase 1 — Minimal viable core (kernel)
**Goal:** a small, auditable language kernel.
**Status (2026-06): complete.** The typed Core IR (`mycelium-core`), the reference interpreter (`mycelium-interp`), the certified binary↔ternary swap (`mycelium-cert`), the first VSA ops (`mycelium-vsa`), and the LSP skeleton landed; interpreter↔AOT equivalence checked (NFR-7).
- **Deliverables:** typed core IR with `Value<Repr,Meta>`; first-class `Bit`/`Trit`; binary + logical-ternary execution on binary HW (packing as metadata); **interpreter (reference semantics) + an AOT path for the kernel itself**, with the interpreter↔AOT equivalence checked (NFR-7); inspectable lowering (≥2 stages, dumpable/diffable, shared across backends); content-addressed definition identity; **one or two** VSA ops via the optional submodule (ADR-008), with attached, tagged bounds (carrying P0.1 forward); certified binary↔ternary swap; **minimal toolchain surface** — an LSP skeleton exposing diagnostics + stage dumps, a linter enforcing the core invariants (no implicit swap, no untagged bound), and a formatter (FR-S5).
- **Success metrics:** SC-1, SC-3 (for the in-scope swaps), SC-4; SC-2 for the shipped VSA op(s); interpreter↔AOT observable equivalence on the kernel test corpus (NFR-7); LSP emits the four semantic-feedback artifact kinds (SC-5 channel).
- **Major risks:** integrative complexity → un-auditable kernel (→ KC-3); real-vs-float/VSA kernel-opacity blocks proofs (**G8**); dual-path semantic divergence (→ RR-12).

### Phase 2 — Full substrate unification + verified swap infrastructure
**Goal:** all four substrates co-equal; lossy swaps certified per-swap.
**Status (2026-06): complete.** Verified numerics (ε/δ, `mycelium-numerics`), the full swap + shared certificate checker, the selection-policy engine + EXPLAIN (`mycelium-select`), Dense/VSA breadth, and the schedule-staged packing selector landed.
- **Deliverables:** dense embeddings + sparse/dense VSA models (MAP/BSC/HRR/FHRR/SBC); the full `swap` with the split regime; per-swap certificate checker (translation-validation model, **VR-4**, **G9**); clean-up memory; reified policies promoted from prototype to feature.
- **Success metrics:** SC-2 across all exposed approximate ops; SC-3 globally; certificate-check overhead within an agreed budget (else → KC-4).
- **Major risks:** translation-validation cost (→ KC-4); VSA bounds insufficiently rigorous to tag `Proven` (**G5**).

### Phase 3 — Tooling, projections, AI co-authoring, acceleration
**Goal:** dual-intelligibility tooling + performance paths.
**Status (2026-06): complete (gate re-asserted 2026-06-15).** The direct-LLVM AOT backend, the native differential + JIT, BitNet-class packed-ternary acceleration, resonator factorization (RFC-0009), and the semantic-projection framework (RFC-0021, M-380) landed.
- **Deliverables:** semantic-level projections (beyond MPS notational projections, **G11**); AI co-authoring with semantic-feedback loops (Area 6); **matured toolchain** (full LSP, rich diagnostics, linter/formatter, build-system stable/experimental management) and **JIT optimization** for dynamic VSA workloads (ADR-009; §5.6–5.8); opt-in resonator factorization with probabilistic guarantees (**FR-C2**, **G4**, exploratory); BitNet-class packed-ternary acceleration (**FR-C3**, **G3**); a documented forward-compatibility path to native ternary HW (**R7**).
- **Success metrics:** SC-5a/b at target thresholds; projection-editing ergonomics validated; JIT path meets interpreter equivalence (NFR-7) and an agreed speedup target.
- **Major risks:** projectional-editor usability friction (Area 6); factorization non-convergence in practice (**G4**).

### Phase 4 — Interpreted↔compiled ABI, hot-inject & AOT-fragment completion
**Goal:** complete the interpreted↔compiled story so stable components compile and hot-swap by content hash.
**Status (2026-06): in progress.** ADR-016 (ABI) and ADR-017 (hot-inject) are Accepted; an in-process inject prototype lives in `mycelium-mlir`. Open: the full AOT env-machine over the v0 calculus, RFC-0001 r5 mutual recursion, the RFC-0012 ambient, dynamic budgets, and stack-robustness (M-341…M-354).

### Phase 5 — Self-hosting & core library
**Goal:** the standard library (RFC-0016) and the Rust-first → Mycelium-lang migration.
**Status (2026-06): Rust-first complete; self-hosting open.** 23 `std-*` crates implement the RFC-0016 contract with their guarantee matrices asserted in tests; the specs read *"implemented (Rust-first), pending ratification."* Self-hosting readiness (M-502) is **not yet established**.

### Phase 6 — Native acceleration & deployment
**Goal:** native MLIR→LLVM codegen for the full calculus, native differential, BitNet-class acceleration, deployable spore units, and the VR-4 hardening gate.
**Status (2026-06): anticipated** (M-601…M-630), gated on the Phase-4 three-way NFR-7 differential.

### Phase 7 — Runtime & concurrency execution model (RFC-0008)
**Goal:** the RT1–RT7 runtime invariants — deterministic-fragment-first, partial failure explicit, honest probabilistic guarantees.
**Status (2026-06): complete.** RFC-0008 Accepted; concurrency/supervision, the RT2 fork/join differential, and the structured-nodule manifest landed (M-355…M-362).

### Phase 8 — Toolchain & release engineering
**Goal:** the matured toolchain (`mycfmt`/`myc-check`/`myc-lint`/`myc-sec`/`myc-doc`/`spore`) and local↔CI check parity.
**Status (2026-06): largely complete.** The five tool crates are folded and wired into `just check` (local↔CI parity); the narrative-authoring pipeline (M-363) design is Accepted, with the build a separate task.

---

## 7. User Stories & Personas

- **US-1 (Human programmer, mixed representations).** *As a systems engineer, I declare a buffer as ternary and bind it into a VSA structure, so that the compiler tracks the paradigm in the type and refuses an implicit conversion* — exercising FR-M1/M2/M4 (Area 3/4).
- **US-2 (AI agent, generate + verify).** *As an AI agent, I synthesize a Mycelium function and immediately read back the typechecker/bound feedback, so that I can self-correct against explicit semantic signals* — exercising NFR-2 and the survey's semantic-feedback finding (Area 6; **G10**).
- **US-3 (AI agent, refactor a swap).** *As an AI agent, I replace a dense-embedding step with a sparse-VSA step and obtain the swap certificate, so that the change's error bound is explicit and auditable rather than silently degrading accuracy* — exercising FR-M3/M4, VR-3/4 (cross-cutting **B**).
- **US-4 (VSA/holographic filesystem-style usage).** *As a developer building a holographic store (à la the Embeddenator use-cases), I bundle records into a fixed-width hypervector and retrieve via clean-up memory, with the capacity bound surfaced so I know when superposition saturates* — exercising FR-S1/S4 and the honest-capacity requirement (Area 2; **G5**, **G6**).
- **US-5 (Representation swap with explicit bounds).** *As an ML engineer, I swap an FP embedding to packed ternary and the system reports ε and tags it `Empirical (Frady-Sommer basis)`, so that I can decide if the loss is acceptable* — exercising VR-3/VR-5 (cross-cutting **B**; survey Caveats).
- **US-6 (Debug/audit a complex VSA composition).** *As an auditor, I inspect each lowering stage and each swap certificate for a multi-bind/bundle pipeline, so that I can localize where crosstalk accumulated* — exercising SC-3/SC-4, FR-M6 (Area 1; **G4**).

---

## 8. Architecture Decision Records (initial)

> **ADR template:** *ID · Title · Status · Context · Decision · Consequences · Grounding.* Status set: Proposed / Accepted / Superseded.

**ADR-001 · Tension-B framing: exact-spec + proven/declared bound**
*Status:* **Accepted** (definitional question settled — see §9; KC-1 passed, **T0.2**).
*Context:* Core forbids hidden approximations, yet VSA/embeddings are inherently approximate (cross-cutting **B**).
*Decision:* Reinterpret as "no *hidden* approximations"; every approximate op = deterministic spec + typed, tagged bound (Rosa/Daisy + DP pattern). The guarantee lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` (RFC-0001) is the typed realization.
*Consequences:* Enables first-class VSA without opacity; requires honest `Proven|Empirical|Declared` tagging per model/op (**VR-5**). The end-state is "fully-disclosed approximation," not "no statistical approximation" — settled at the research gate.
*Grounding:* **B**, **R1**, **R5**; Area 5; **T0.2**.

**ADR-002 · Split verification regime**
*Status:* Accepted.
*Context:* binary↔ternary is exact; ↔VSA/embedding is lossy.
*Decision:* Bijective proofs for binary↔ternary; per-swap certificates with ε/δ bounds for lossy swaps; do not unify.
*Consequences:* Two code paths; honest non-uniformity; translation-validation infra needed (**G9**).
*Grounding:* **R2**, **VR-4**; cross-cutting **B**.

**ADR-003 · Architecture anchors = MLIR ⊕ Unison ⊕ Arrow**
*Status:* Accepted.
*Context:* The four-way union is unprecedented; components are mature (TL;DR).
*Decision:* Build on MLIR-style lowering + Unison-style content-addressing + Arrow-style metadata; prototype their *combination* as the novel work.
*Consequences:* Integration is the main risk (**KC-3**); each anchor's shortfall must be filled (representation-paradigm metadata, verified swaps, bounds).
*Grounding:* **R3**; cross-cutting **D**.

**ADR-004 · Embeddenator = provisional inspiration, not reference**
*Status:* Accepted.
*Context:* Embeddenator is VSA-*inspired* indexed storage, not compositional VSA (lacks invertible binding, clean-up, factorization, capacity accounting).
*Decision:* Treat its Manifests/Engram/chunking/adaptive-sparse-dense patterns as evidence of *real needs*, not as a target. To reach language-grade, the missing pieces in **G6** must be added.
*Consequences:* Avoids inheriting aspirational-but-unmet VSA claims; clarifies what "true VSA" requires.
*Grounding:* survey Embeddenator framing; **G6**; Area 2.

**ADR-005 · Ternary = logical substrate now, native later**
*Status:* Accepted.
*Context:* No competitive ternary hardware; BitNet success is *weight quantization*, not general ternary compute; Setun itself emulated ternary (Area 3 caveats).
*Decision:* Ternary is a logical/semantic substrate on binary HW using BitNet-class packing exposed as inspectable metadata; preserve a forward-compatible value-semantics contract.
*Consequences:* Lowest-risk substrate; research risk stays in VSA + verification, not ternary.
*Grounding:* **R7**, **G3**.

**ADR-006 · Representation-selection policies are reified, inspectable artifacts**
*Status:* Accepted.
*Context:* "Intelligent" metadata-driven selection is the most likely place to reintroduce a black box (**G2**, cross-cutting **D**).
*Decision:* No implicit selection; every policy is a first-class, queryable value, and each swap records the `PolicyRef` it used.
*Consequences:* Slightly more verbose; preserves auditability (NFR-3).
*Grounding:* **G2**; cross-cutting **D**.

**ADR-007 · Kernel in Rust (reference interpreter); MLIR→LLVM for AOT; tooling/experiments in Python**
*Status:* **Accepted** (was Proposed; ratified by **T1.5**, **T2.6**).
*Context:* Embeddenator is Rust; the reader's stack is Rust + Python; affine ownership supports no-hidden-state (Area 5). Research (T1.5) selected MLIR as the AOT backbone; (T2.6) found no production-grade Rust VSA library.
*Decision:* Implement the kernel + the **reference-semantics interpreter** in Rust (MSRV 1.92), kept as the trusted base. AOT path = **MLIR → LLVM** (a `ternary` dialect first), confined to the performance path so its complexity does not infect the trusted base (RFC-0004 §2). Build the **VSA submodule** in Rust (port torchhd's op set as reference; reuse the `balanced-ternary` crate). Probes/experiments and the LLM harness in Python (3.13/3.14, UV, pytest, codecov).
*Consequences:* MLIR adds an FFI boundary and a large C++ dependency (RR-11), accepted for multi-substrate fit + forward-ternary path; the immature Rust VSA ecosystem means building the submodule (RR-14).
*Grounding:* Area 5 (Rust/affine); **T1.5**, **T2.6**.

**ADR-008 · VSA is in core semantics but packaged as an optional submodule**
*Status:* Accepted.
*Context:* VSA must be powerful and first-class, but forcing every user (and every kernel audit) to carry the full VSA algebra bloats the core and threatens auditability (**KC-3**, NFR-6).
*Decision:* Keep the `Hypervector` paradigm, its metadata, and the swap machinery in the kernel as a *type slot*; move the operational algebra (bind/unbind/bundle/permute/cleanup, model implementations, bound derivations, factorization) into an optional, dependency-gated submodule with a narrow, RFC-specified boundary (§5.7).
*Consequences:* Lean kernel for non-VSA users; coherent first-class VSA for those who opt in; the submodule boundary becomes a key spec artifact; mirrors torchhd-as-separable-layer (Area 2).
*Grounding:* NFR-6, **KC-3**; Area 2; survey TL;DR (component separability).

**ADR-009 · Hybrid execution model; AOT preferred for stable components; interpreter is reference semantics**
*Status:* Accepted.
*Context:* The project needs both fast stable artifacts and a flexible path for development and dynamic/early VSA work; and "no opaque lowering" must hold regardless of how code runs (Area 1; cross-cutting **A.1**).
*Decision:* One Core IR, multiple backends — interpreter/VM (the **reference semantics**), JIT, and AOT native codegen. AOT is preferred for *stable components* (content-addressed + spec-ratified + verification-passed); interpretation/JIT serves development, exploration, and dynamic VSA. AOT/JIT output is validated against the interpreter via the swap-grade translation-validation machinery (NFR-7, **VR-4**). No-opaque-lowering applies to all backends.
*Consequences:* Multi-backend lowering effort; a precise "stable component" definition is required (delivered in RFC-0004); the interpreter↔AOT equivalence obligation (RR-12) must be discharged; the AOT path is **MLIR→LLVM** (committed via ADR-007 / RFC-0004; **T1.5**).
*Grounding:* Area 1 (MLIR multi-backend, MetaOCaml staging, CompCert/translation validation); **R3**; cross-cutting **A.1**.

**ADR-010 · Verified-numerics foundation: two bound kernels + shared certificate** *(full record: standalone ADR-010 file)*
*Status:* **Accepted** (ratified by **T0.1**).
*Context:* Approximate ops need sound, composable, honestly-tagged bounds; a single algebra cannot unify error-magnitude (ε) and failure-probability (δ) composition (T0.1c).
*Decision:* Two kernels meeting at one `{ε, δ, strength}` certificate — `ErrorBound` (affine arithmetic, Daisy/FloVer) for ε; `ProbBound` (union-bound, with apRHL couplings for relational certs) for δ. `strength` composes by meet; the one cross-kernel inference is accuracy→probability. Trusted base = certificate-checker-in-Rust (FloVer-style), not a hosted prover. VSA crosstalk *content* comes from the concentration-inequality literature (RFC-0003), not the verified-numerics tools.
*Consequences:* Unblocks RFC-0001 §4.7 (composed results carry `Proven`/`Empirical`, not `Declared`); one bound vocabulary across RFC-0001/0002/0003.
*Grounding:* **T0.1**, **T0.2**; survey Area 5.

---

## 9. Risk Register & Open Questions

| ID | Risk | Likelihood · Impact | Mitigation | Gap |
|---|---|---|---|---|
| RR-1 | ~~Tension B unsolvable for VSA in core~~ — **de-risked: KC-1 passed (T0.2)**; residual is only that proven bounds are loose sufficient conditions | L · M | Proven bounds axiomatized + arithmetic checked (ADR-010); opt into `Empirical` for tighter Frady-style numbers | **G7**, **G5** |
| RR-2 | Integrative complexity → meta-level black box | H · M | Small auditable kernel (NFR-6); KC-3; phase gates | TL;DR |
| RR-3 | LLM leverage collapses on novel syntax | M · H | P0.2 experiment; projections + semantic feedback; KC-2; embedded-DSL fallback | **G10** |
| RR-4 | VSA bounds too weak to tag `Proven` | M · M | Use Clarkson-Ubaru-Yang / Thomas-Dasgupta-Rosing where applicable; else tag `Empirical` honestly | **G5** |
| RR-5 | Resonator factorization non-convergence breaks compositional queries | M · M | Keep factorization opt-in & probabilistic (FR-C2); never in core | **G4** |
| RR-6 | Metadata-driven selection reintroduces opacity | M · H | Reified policies (ADR-006); selection is logged per-swap | **G2** |
| RR-7 | Per-swap certificate (translation validation) too costly | M · M | Per-swap validation not whole-engine proof; KC-4 downgrade path | **G9** |
| RR-8 | Real-vs-float/VSA kernel-opacity blocks exact proofs | L · M | **Resolved-direction:** ADR-010 Accepted — Flocq trusted base + certificate-checker-in-Rust (FloVer-style) | **G8** |
| RR-9 | Ternary emulation overhead degrades performance | L · M | BitNet packing (I2_S/TL1/TL2), schedule-staged + benchmarked (T1.4) | **G3** |
| RR-10 | Adoption/ecosystem failure (ternary niche has repeatedly failed to sustain) | M (long-term) · M | Lead with the *integration* value, not ternary novelty; dual human/AI intelligibility | Area 3 |
| RR-11 | Toolchain + multi-backend scope balloons (LSP + linter + formatter + build + interpreter + JIT + AOT) | M · M | Minimal Phase-1 surface only (FR-S5); mature in Phase 3; reuse LLVM/MLIR rather than build-our-own where possible | Area 1/6; ADR-009 |
| RR-12 | Interpreter↔AOT/JIT semantic divergence (two execution paths, two behaviours) | M · H | Interpreter = single reference semantics (NFR-7); validate every compiled artifact against it via **VR-4**; shared stage semantics | Area 1/5; ADR-009 |
| RR-13 | **HRR/FHRR are the VSA weak link**: non-self-inverse bind ⇒ lossy unbind (Empirical only); resonator factorization may not converge | M · M | Prefer MAP/BSC (self-inverse bind, Exact) for compositional work; restrict HRR/FHRR to single-factor; factorization Phase-3, probabilistic-only (RFC-0003 §4/§6; **T1.2**) | **G4**, **G5** |
| RR-14 | **Rust VSA ecosystem immature** (no torchhd analogue) — must build the VSA submodule | M · M | Reuse `balanced-ternary` crate for trit storage; port torchhd's op set as reference; budget build effort (**T2.6**) | impl |

**Open questions (post-research):** the four survey coverage gaps are now **closed** by the follow-up pass — array/representation languages (Dex/Futhark, **T2.4**), neurosymbolic synthesis IRs (**T2.5**), verified probabilistic numerics (**T0.1**), and the Rust VSA/ternary ecosystem (**T2.6**). The ADR-001 definitional question is **settled** in favour of "fully-disclosed approximation" (the guarantee-lattice tags). **Genuinely remaining:** the full term language (a later RFC); the one confirming Liquid-Haskell `bundle` probe (RFC-0003 §5); and whether semantic-level (not merely notational) projections are ergonomically viable (**G11**, an E4-adjacent question).

---

## 10. Recommended Immediate Next Actions

> The original Phase-0 confirm + Phases 1–3 build items (1–7) are **complete**; the live next actions follow. The authoritative task board is `tools/github/issues.yaml` (+ `idmap.tsv`) and `docs/planning/phase-*.md`.

**Complete (Phases 0–3, 5, 7):** the LH `bundle` probe (KC-1 confirmed, `proofs/lh-bundle/`); the Core IR + reference interpreter; the binary↔ternary `LosslessWithinRange` swap + the single certificate checker; the `ternary` MLIR dialect + schedule-staged packing (E1 measured); the VSA submodule; the LLM-leverage experiment (KC-2 verdict **proceed**, DN-09); and the Rust-first standard library (23 `std-*` crates).

**Live, dependency-ordered:**
1. **[P1 · build] Phase-4 ABI/AOT completion** — the full AOT env-machine, RFC-0001 r5 mutual recursion, the RFC-0012 ambient, dynamic budgets, and stack-robustness (M-341…M-354), toward the three-way NFR-7 differential.
2. **[P1 · gate] Self-hosting readiness (M-502)** — whether the surface can host a stdlib module migrated from Rust-first to Mycelium-lang; unblocks Batch P5-C.
3. **[P2 · build · dep: 1] Phase-6 native codegen + deployable spores** (M-601…M-630).
4. **[ongoing] Keep the navigational docs current** (README / Doc-Index / this charter) and the `just check` doc-currency gate green (M-371).
5. **[ongoing] Maintain the ADR log, risk register, and `Proven|Empirical|Declared` tags** per model/op (VR-5).

---

## Meta — maintaining and evolving this foundation

- **Treat this as a living, version-controlled artifact.** Keep it in git; consider content-addressing in spirit (ADR-003/Unison) — each ratified version gets an immutable reference.
- **ADRs are append-only** with status transitions (Proposed → Accepted → Superseded). Never silently edit a decision; supersede it and link forward.
- **Re-run the kill criteria (KC-1…KC-4) at every phase gate.** They are the project's circuit-breakers; a gate that doesn't check them is a gate that hides risk (**KC-3** spirit).
- **Keep `Proven | Empirical` tags honest as the literature moves.** New non-asymptotic VSA results can *upgrade* a tag; absence of one keeps it `Empirical` (**G5**, **VR-5**).
- **Update grounding references when new research lands.** This phase forbade new prior art; subsequent research passes will add citations — re-anchor claims to them rather than letting assertions float.
- **Freeze the Phase-1 spec only after the four coverage gaps are closed** (**R8**) and after ADR-001's definitional question is ratified.
- **Track work as dependency-ordered, priority-tagged tasks** (§10 is already in that shape) so the Kanban board mirrors this document, and the document remains the single source of truth the board points back to.
