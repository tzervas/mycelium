# RFC-0006 — Surface Language, Grammar & Term-Language Layering

| Field | Value |
|---|---|
| **RFC** | 0006 |
| **Status** | **Draft** (deliberation artifact — concrete syntax is explicitly **gated on KC-2**) |
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

### 4.2 Capability target — "Rust-class and beyond", as requirements

Stated as language requirements **LR-1…LR-8** so the eventual L1/L2 designs are checkable against
them rather than against a vibe:

| Id | Requirement | Posture |
|---|---|---|
| **LR-1** | Algebraic data types + exhaustive `match` (no fall-through silently) | committed direction |
| **LR-2** | Parametric polymorphism with bounded abstraction (traits/typeclasses; coherent, no overlapping-instance ambiguity) | committed direction |
| **LR-3** | Modules with content-addressed identity (ADR-003); separate compilation falls out of hashing, not file layout | committed direction |
| **LR-4** | General recursion, **with a declared totality posture**: the trusted interpreter keeps its fuel guard; a checked-total fragment is what "stable components" (RFC-0004 §4) promote from | committed direction |
| **LR-5** | **Repr polymorphism**: abstracting over the paradigm (`∀r: Repr-of-kind-K`) without violating S1 — swap insertion stays explicit even in generic code | committed direction (the "beyond Rust" core) |
| **LR-6** | **Guarantee-indexed types**: the lattice tag as a type-level index (e.g. a function demanding `Exact` input is a *type error* to call with `Declared`), with `meet` as the composition law — the honesty rule moved into the type system | beyond-Rust goal; open design (Q3) |
| **LR-7** | Effects: at minimum, partiality and swap-effects tracked; full effect system | open (Q4) |
| **LR-8** | Ownership/borrowing: **likely not applicable as in Rust** — Mycelium is a value-semantics substrate (no aliased mutable state to police). What may be needed is *linearity/affinity for external resources* only | open (Q5); do not cargo-cult Rust's borrow checker without a problem it solves here |

"Beyond Rust" therefore means LR-5 + LR-6 (+ LR-7 if accepted): paradigm- and honesty-aware types
that no mainstream language has — *not* exotic syntax. KC-2 will test whether the *surface* can
stay boring while the type system carries the novelty (the RR-3 hypothesis).

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

## 8. Unresolved questions

- **Q1 (KC-2 gate):** text syntax vs projections vs embedded DSL for L3 — decided by M-002 (#3),
  not by taste.
- **Q2 (L1 node budget):** exactly which nodes join the Core IR (`Fix` vs structured recursion;
  data declarations as nodes vs a registry like VSA models).
- **Q3 (LR-6 mechanism):** guarantee indices as a kind, as refinements, or as a coeffect — and
  what `Proven`'s *checked side-conditions* (VR-5) look like as a typing premise.
- **Q4 (effects):** dedicated effect system vs monadic encoding vs nothing-beyond-partiality.
- **Q5 (linearity):** does any real resource in scope (handles? reconstruction manifests?) need
  affine types, or is LR-8's "not applicable" the final answer?
- **Q6 (numeric literals & bounds):** how surface literals declare/infer `Repr` without violating
  S1 (the M-020 fragment's `0b…`/`<…>` approach is one candidate).

## 9. Future possibilities

Guarantee-polymorphic libraries ("works at whatever strength your inputs have, returns the
meet"); certified-swap combinators as first-class values; an LLM-facing canonical projection
(KC-2's "beyond" case) where the *same* content-addressed definitions render differently for
human and machine co-authors (FR-S5's dual intelligibility).

## Meta — changelog

- **2026-06-10 — Draft.** Initial deliberation draft: layering L0–L3, syntactic honesty
  invariants S1–S5, capability requirements LR-1…LR-8 ("Rust-class and beyond" made checkable),
  grammar/spec discipline, sequencing, and the open questions Q1–Q6. Concrete L3 syntax is
  explicitly gated on KC-2 (M-002), per the existing M-020/RR-3 posture. Ratification
  (Draft → Accepted) is a maintainer decision.
