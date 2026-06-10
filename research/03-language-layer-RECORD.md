# Research Record 03 — Language-Layer Targets T3 (Pass 3)

> **What this file is.** A durable record of the third research pass, which grounds the open
> questions of **RFC-0006** (surface language, grammar & term-language layering): scope, results
> by target, the positions they support, and the source base. Conducted 2026-06-10 as four
> parallel deep-dives with primary-source verification; load-bearing claims were checked against
> primary sources, and unverified details are flagged inline. Findings are labeled **T3.1–T3.6**
> (continuing the T0/T1/T2 scheme) and map onto RFC-0006's Q1–Q6 / LR-1…LR-8.

## Scope

Resolve, with evidence, the design-blocking questions RFC-0006 (Draft) left open: the L1 kernel
node budget (Q2), the mechanism for guarantee-indexed types (Q3/LR-6), the Repr-polymorphism
restriction set (LR-5), the effect/totality posture (Q4/LR-4), the ownership/linearity verdict
(Q5/LR-8), the grammar/conformance stack and literal design (Q6/§4.3), and the evidence base for
the KC-2 surface-syntax experiment (Q1).

## Results by target

### T3.1 — Kernel-calculus node budget (→ Q2)

Four independent ecosystems converge on the same architecture: a ~10–16-constructor expression
grammar with **data-type definitions in a separately-checked declaration/environment layer, not
in the term language**.

- GHC Core (`Expr b`) has exactly **10 constructors** (`Var, Lit, App, Lam, Let, Case, Cast,
  Tick, Type, Coercion`; GHC 9.0.1 haddock) — all of Core is "3 types, 15 constructors" (Peyton
  Jones, *Into the Core*, 2016; figure from the talk abstract — moderately verified). Foundational
  paper: Sulzmann, Chakravarty, Peyton Jones & Donnelly, *System F with Type Equality Coercions*,
  TLDI 2007. `TyCon`/`DataCon` live in the environment; a program is `[CoreBind]`.
- Lean 4's kernel `Expr` has ~12 constructors; **inductive types are kernel declarations** with
  derived recursors. Coq/Rocq's `constr` has ~16 (it puts `Fix`/`CoFix`/`Case` *in* the term
  language — and pays with a much larger kernel reduction engine).
- Pattern matching: kernel `Case` is **flat, exhaustive, single-level**; nested surface matches
  are compiled away (Augustsson, FPCA 1985; Maranget, *Compiling Pattern Matching to Good
  Decision Trees*, ML Workshop 2008).
- Recursion: recursor-only kernels (Lean) buy the strongest certification but cost a heavy
  elaborator and create reduction traps (lean4 issue #5192, kernel-irreducible WF recursion);
  letrec-only (GHC) has no total fragment; **Idris 2's posture — general recursion by default,
  an opt-in conservative totality checker certifying a structural/measure-decreasing subset —
  fits a fuel-guarded trusted interpreter best.**
- Content-addressing recursive groups: Unison hashes via **abstract binding trees**; a mutually
  recursive cycle hashes as a unit with members canonically ordered by their cycle-removed
  hashes (`#x.n`); data constructors hash as `#type#c` (Unison hashes doc; Chiusano 2015). This
  is the recipe for hashing L1 `Fix` groups and data declarations.

**Position.** L1 = L0's five nodes + **`Lam, App, Construct (saturated, by DataCon ref), Match`
(flat/exhaustive, Maranget-compiled), `Fix`** (≈ 9–10 expression nodes total); **data
declarations are content-addressed registry entries**, never expression nodes; general `Fix` in
the kernel with the totality checker *outside* it (T3.4).

### T3.2 — Guarantee-indexed types (→ Q3 / LR-6)

The guarantee lattice is formally an **integrity lattice**, and the right machinery is 25 years
old:

- Lineage: Denning's lattice model (CACM 1976) → Volpano–Smith–Irvine soundness-as-noninterference
  (J. Computer Security 1996) → JFlow/Jif label polymorphism + declassification (Myers, POPL
  1999) → FlowCaml's full constraint-based label inference over ML (Pottier & Simonet, POPL
  2002/TOPLAS 2003) → the Dependency Core Calculus (Abadi, Banerjee, Heintze, Riecke, POPL 1999;
  Algehed PLAS 2018; Choudhury et al. ESOP 2022).
- **Meet vs join:** integrity is the order-dual of confidentiality (Biba 1977); "weakest wins" by
  meet *is* a trust/integrity label read of the lattice. `Declared` = "untrusted"; the forbidden
  silent upgrade is exactly IFC's **endorsement** — which Jif gates behind privileged operations
  and **Mycelium gates behind a checked certificate** (strictly stronger: VR-5 with evidence).
- Graded coeffect systems **subsume** flat labels: grades over pre-ordered semirings (Petricek/
  Orchard/Mycroft ICFP 2014; Katsumata POPL 2014; Gaboardi et al. ICFP 2016; Granule — Orchard,
  Liepelt & Eades, ICFP 2019). A bounded totally-ordered lattice (the 4-chain) forms such an
  algebra, and Granule ships a security-lattice grade (`Level`) proving lattice grading works in
  practice. Maturity: **research prototype** (no production graded systems).
- Refinement types (Liquid Haskell — Vazou et al., Haskell 2014; F*) are the right tool for the
  **ε/δ side-conditions** ("Proven only with checked side-conditions" as a refinement premise on
  `Swap`), with documented SMT-ergonomics costs; they are overkill as the mechanism for the
  4-point tag itself. LWeb (POPL 2019) shows labels-via-refinements is feasible but heavyweight.
- **Novelty flag:** no prior system was found combining lattice grading with machine-checkable
  per-value certificates at runtime (absence of evidence, flagged as such).

**Position.** LR-6 = a **graded coeffect modality over the guarantee meet-semilattice**, with
IFC's metatheory as the soundness story (a DCC-style noninterference theorem read in the
integrity direction: *no `Declared` input influences an `Exact`-typed output except through a
certified `Swap`* — the certificate is controlled endorsement). Refinements are reserved for
certificate side-conditions (stage 2). Staged: **0** runtime tags + meet (already built) → **1**
static grades, monomorphic first, then bounded grade polymorphism with Pottier–Simonet-style
inference (near-trivial over a 4-chain) → **2** refinement premises on `Swap`/`Proven`. One
decision RFC-0006 must record explicitly: whether **implicit flows** taint (does branching on a
`Declared` value degrade the result?) — IFC needs a `pc` label for this; tracking only data
lineage is legitimate but must be a recorded decision.

### T3.3 — Repr polymorphism (→ LR-5)

- GHC levity polymorphism (Eisenberg & Peyton Jones, PLDI 2017 — production-grade) enforces
  "never move or store a representation-polymorphic value" via exactly **two restrictions**
  (verified from the paper): no representation-polymorphic **binders**, no
  representation-polymorphic **function arguments**. Generalization: *Kinds Are Calling
  Conventions* (Downen et al., ICFP 2020).
- F# units of measure (Kennedy, POPL 1997; CEFP 2009) is the model erased-index system with full
  inference and zero implicit conversion. Mycelium keeps indices at **runtime**, which *relaxes*
  GHC's constraints: a runtime `Repr` witness can size/move values dictionary-style.

**Position (LR-5 restriction set).** (a) `Repr` is a type-level index; unification never inserts
a conversion — cross-repr signatures must name `Swap` + certificate explicitly; (b) default to
**monomorphization** (under which the two GHC restrictions vanish); (c) for polymorphic
compilation/dynamic dispatch, adopt the levity restrictions verbatim *unless* a runtime `Repr`
witness is in scope; (d) **no repr subtyping** — generic bounds are interfaces of
repr-homogeneous operations only.

### T3.4 — Effects & totality (→ Q4 / LR-4)

- Koka (Leijen, MSFP 2014/POPL 2017) is the mature precedent for **divergence as just another
  inferred effect** (`total` = the empty row); OCaml 5 deliberately shipped *untyped* handlers
  (Sivaramakrishnan et al., PLDI 2021) — viable, but sacrifices the static story Mycelium's
  honesty rule needs; Effekt (OOPSLA 2020) and WasmFX (OOPSLA 2023; not yet standardized) round
  out the field. Mycelium's intrinsic effects beyond what values already carry: **only
  divergence** (partiality is explicit `Option`/`Result`; swap is lexically explicit).
- Totality-as-privilege precedents: Idris 2 per-definition `total`/`covering`/`partial` pragmas
  (default `covering`); **Lean 4's `partial def` is accepted but kernel-opaque** (never unfolded;
  needs only an inhabited return type), with `unsafe` as the escape hatch — the cleanest
  precedent for "unchecked recursion exists but is quarantined from the trusted layer".
- Fuel: CakeML defines all of its ILs in **functional big-step (clocked) style** — a fuelled,
  total-by-construction interpreter with provable divergence-preservation (Owens, Myreen, Kumar,
  Tan, ESOP 2016); step-indexing is canon (Appel & McAllester, TOPLAS 2001). The architecture
  "fuel in the trusted interpreter + totality checker outside the kernel gating only a
  *privilege*" is sound: a wrong checker can mis-gate promotion, never change semantics.
- **Novelty flag:** no exact precedent gates *AOT compilation* on totality (Idris gates trust,
  Lean gates kernel reduction). Mycelium's "checked-total gates stable-component promotion"
  (RFC-0004 §4) is a novel composition of established pieces — cite as analogy, not precedent.

**Position.** Q4: **no algebraic-effect system**; adopt **divergence-only tracking** — one
per-definition bit (`total` vs `partial`), Koka's `div` in degenerate form. Documented growth
path (bit → small fixed row → row polymorphism), never untyped handlers. LR-4: Lean's
`partial`-opaque split + Idris-style pragmas + CakeML clocked semantics; structural (Foetus-style)
checking first, sized types later. "Checked total" formally = terminates under the reference
interpreter for every sufficiently large fuel (CakeML's clock quantification).

### T3.5 — Ownership & linearity (→ Q5 / LR-8)

- Hylo's **mutable value semantics** (Racordon et al., JOT 2022): in-place part-wise mutation via
  second-class references (`inout`/projections, never stored) — no aliasing, hence **no borrow
  checker needed**; Swift independently validates (CoW value types + Law of Exclusivity,
  SE-0176). Perceus/FBIP (Reinking, Xie, de Moura, Leijen, PLDI 2021; used by Koka and Lean 4)
  recovers in-place update from purely functional source via reuse analysis.
- Linear Haskell (Bernardy et al., POPL 2018) in practice clusters around exactly two uses —
  safe in-place mutation and resource/protocol enforcement — and remains officially experimental
  six years on. Futhark's uniqueness types serve in-place updates (covered for Mycelium by
  Perceus-style techniques). Granule/Marshall–Orchard (ESOP 2022) unify linearity and uniqueness
  in graded settings if ever needed — and compose with the T3.2 grading choice.

**Position.** **Ownership/borrowing: not applicable** — it polices aliased mutation, which the
value-semantics model excludes; in-place performance is an implementation story (Perceus-style
reuse first; Hylo-style `inout` only if surface mutation syntax is ever wanted). **Linearity:
defer, reserve a hook** — design the kind structure so a small affine `Resource` kind can be
added without breaking value-kinded code; ship nothing now.

### T3.6 — LLM leverage on novel syntax (→ Q1 / the KC-2 experiment)

What is *measured* (preferring 2023–2026 sources):

- **The low-resource gap is real, large, scale-resistant — and data-fixable.** MultiPL-E
  (Cassano et al., TSE 2023; arXiv:2208.08227): pass@1 tiers by corpus frequency (≤~30% for
  R/Racket/Perl vs 50–75% Python/JS on comparable models; survey arXiv:2410.03981). MultiPL-T
  (OOPSLA 2024; arXiv:2308.09895) and the Verilog line (VerilogEval → CodeV) show targeted
  fine-tuning data largely closes it.
- **A spec in long context can teach an unseen formal system to human level.** MTOB (Tanzer et
  al., ICLR 2024; arXiv:2309.16575): translating Kalamang from one grammar book in-context;
  Gemini 1.5 reported 58.3 chrF English→Kalamang, *exceeding* the 57.0 human baseline — and
  near-random without the book. Book *quality* mattered.
- **Constrained decoding fixes syntax, not semantics — and can hurt.** SynCode
  (arXiv:2403.01632): CFG-masking eliminates ~91–99% of syntax errors but claims validity only;
  Grammar-Aligned Decoding (NeurIPS 2024; arXiv:2405.21047) proves greedy constrained decoding
  *distorts* the model's distribution; Tam et al. (EMNLP-Industry 2024) find strict format
  constraints degrade reasoning-heavy tasks. Grammar prompting helps DSL generation (Wang et
  al., NeurIPS 2023; arXiv:2305.19234).
- **Nobody has measured syntax-skinning in isolation** (no "rename the keywords" ablation
  exists); ReCode (ACL 2023) shows code models are most sensitive to syntax-level perturbations
  among 30 transformation types — surface form matters. Two unreviewed-preprint datapoints,
  flagged as such: an esoteric-PL in-context study (docs+examples help substantially, parity not
  reached) and **Anka** (arXiv:2512.23214 — a deliberately novel, verbose, designed-for-LLMs DSL
  taught via ~100 lines of in-context grammar: 99.9% parse rate and *beating* Python on
  multi-step tasks on small models; n=100, two models).
- Unison demonstrates content-addressed ASTs with names-as-metadata rendering on demand in
  production (incl. agent-facing MCP commands as of 1.0); JetBrains MPS/mbeddr lessons (Voelter,
  SoSyM 2017) attribute projectional editing's niche adoption to breaking the text-editor
  paradigm — for any new *textual* syntax, tree-sitter + LSP is the table-stakes cost. eDSLs
  inherit host *syntax* fluency, not semantic fluency (PyTorch ≫ JAX despite one host).

**Position (KC-2 design).** Five conditions over identical task semantics: (1) novel syntax
bare; (2) novel syntax + book-quality grammar/spec + worked examples in context; (3) = 2 +
constrained decoding (measured separately — syntax-fix vs distribution-distortion pull opposite
ways); (4) same AST, familiar-skinned concrete syntax (the unmeasured ablation KC-2 can be first
to run); (5) eDSL in a high-resource host. Metrics: parse rate; pass@1/k on hidden tests; a
syntax-vs-logic error taxonomy; in-context token cost; and the headline **LLM-leverage retention
ratio** (per-model degradation vs that model's Python baseline). Include a multi-step
composition tier (Anka suggests differences amplify there). ≥2 frontier + 1 small model;
per-model reporting (honesty rule). **Working hypothesis** (inference, falsifiable): novel-but-
regular syntax + grammar-in-context + canonical formatting over content-addressed storage (skins
as cheap projections) retains most leverage. **Falsified if** condition 2 retains <~70% of
condition 4's pass@1 on composition tasks, or if parity requires constrained decoding that
measurably degrades semantic correctness — then L3 becomes a projection of known syntax (eDSL
only if condition 5 dominates).

## Decisions this pass supports (pointers)

- RFC-0006 §8 Q1–Q6 → positions recorded in the RFC's 2026-06-10 revision (still Draft;
  ratification is the maintainer's).
- The L1 node budget + registry pattern (T3.1) → the planned RFC-0001 revision for Phase 3.
- The grammar/conformance stack (T3.1 cluster B): W3C-notation EBNF in the spec (not ISO 14977 —
  Wheeler's critique), a Menhir-class LR(1) machine-readable grammar as the normative oracle
  (PEG rejected for specs: ordered choice hides ambiguity), tree-sitter as a derived advisory
  artifact, and a WASM-style accept/reject/elaborates-to conformance corpus run in CI.
  Literals: universal-until-elaboration (Ada-style), suffix-or-inferred with **no defaulting
  across representation families** (stricter than Rust's `i32` default).
- The KC-2 protocol (T3.6) → the M-002 experiment design (still blocked on LLM API access).

## Key sources

(Representative; per-finding citations inline above.) GHC Core haddock + Sulzmann et al. TLDI
2007 + Peyton Jones *Into the Core*; Maranget 2008; Lean 4 kernel docs; Unison hashes doc;
WebAssembly spec repo; Wheeler on ISO-14977; Kennedy units-of-measure; Volpano–Smith–Irvine
1996; Myers POPL 1999; Pottier–Simonet TOPLAS 2003; Abadi et al. POPL 1999 (DCC); Orchard et
al. ICFP 2019 (Granule); Eisenberg & Peyton Jones PLDI 2017; Leijen MSFP 2014/POPL 2017;
Sivaramakrishnan et al. PLDI 2021; Owens et al. ESOP 2016 (CakeML clocked semantics); Racordon
et al. JOT 2022 (Hylo); Reinking et al. PLDI 2021 (Perceus); Bernardy et al. POPL 2018 (Linear
Haskell); Cassano et al. 2023/2024 (MultiPL-E/T); Tanzer et al. ICLR 2024 (MTOB); Wang et al.
NeurIPS 2023 (grammar prompting); Park et al. NeurIPS 2024 (GAD); Voelter SoSyM 2017 (mbeddr).

## Honest-uncertainty register

- "3 types / 15 constructors" for whole-Core: from talk abstract, not re-derived.
- Lean `Expr` constructor names partly from memory; count order-of-magnitude verified.
- "Totality gates AOT" and "grading + runtime certificates" have **no found precedent** —
  treat both as novel contributions requiring their own soundness arguments, not citations.
- Anka and the esoteric-PL study are unreviewed preprints (small n).
- WasmFX status as of early-2025 sources.
