# RFC-0020 — The L2 Surface Term Language

| Field | Value |
|---|---|
| **RFC** | 0020 |
| **Status** | **Accepted (scoped)** (2026-06-18, maintainer ratification; DN-12) — the §4.1/§4.3/§4.4/§4.6/§4.7/§4.8/§4.9 core is Accepted (no research gate). **Carved out (deferred, not objections):** §4.2 polymorphic instantiation + grade-inference integration, §4.5 `grow`-derived traits, and R20-Q1…Q5 — each unblocking via the now-Accepted RFC-0018/RFC-0019 and the enacted RFC-0001 r5 `FixGroup`. The RFC-0006 r5 carve-out precedent. |
| **Type** | Foundational / normative (once Accepted) — L2 surface layer; no L1 or L0 change |
| **Date** | 2026-06-18 |
| **Depends on** | RFC-0006 §3/§4.1 (layering L0–L3, invariants S1–S6); RFC-0007 §4.1–4.8 (L1 kernel: ten nodes, registry, typing, elaboration); RFC-0012 §4 (ambient paradigm — the enacted model example of L2 elaboration-only sugar); RFC-0011 (flat `Match` and Maranget compilation into L0); DN-09 §3.2 (usability-first design bias, committed text surface); DN-03 (lexicon — reserved forms); LR-1/LR-3; KC-3; ADR-003; ADR-006 |
| **Coupled with** | `docs/spec/grammar/mycelium.ebnf` (the committed v0 text surface this layer elaborates from); `crates/mycelium-l1` (the L1 target); `docs/spec/grammar/` (conformance corpus); RFC-0018 (stage-1 guarantee grading — type-system basis for §4.3); RFC-0019 (traits/polymorphism — type-system basis for §4.4 inference and §4.5 derived forms); RFC-0021 (L3 projection rendering of L2 terms) |

---

## 1. Summary

The KC-2 verdict (DN-09) committed the v0 text syntax as the L3 surface. This RFC defines the
layer immediately beneath it: **L2, the surface term language** — the programmer-facing
conveniences that elaborate to L1 and that are defined **entirely by that elaboration** (RFC-0006
§3). L2 has no independent semantics; every construct's meaning is the L1 term it desugars to, and
every desugaring is inspectable (S4). The trust boundary does not move: L0 and L1 remain the only
trusted layers; L2 is untrusted-by-construction and its elaborator is auditable.

This RFC fixes:

1. **The invariants L2 must preserve** (S1–S6 restated at the surface elaboration level);
2. **The scope of each L2 convenience and its normative elaboration to L1** — type inference
   (§4.2), modules (`nodule`/`use`, §4.3), pattern sugar (§4.4), derived forms (§4.5), literal
   handling (§4.6), and the ambient paradigm (§4.7 — the enacted model);
3. **The usability-first design bias** (DN-09 §3.2) that governs all L2 spelling choices;
4. **A conformance-corpus plan** for the L2 elaborator (§4.8).

Sibling Drafts are referenced by number and title but not read here: **RFC-0018** (stage-1
guarantee grading) and **RFC-0019** (traits and polymorphism) supply the type-system machinery L2
inference and derived forms will use; **RFC-0021** (projections) is the L3 rendering layer above
L2 that renders the same content-addressed L2/L1 definitions.

## 2. Motivation

RFC-0007 committed the L1 kernel calculus: ten nodes, a content-addressed registry, and v0
monomorphic typing. That is the right trusted base — small, explicit, checkable. But writing
programs *directly* in L1 is impractical: every call site must be explicitly typed, every nested
pattern must be manually compiled to flat `Match` alternatives, every module `use` resolved by
hand, and every `if–else if` chain expanded into nested `Match`. The programmer-facing language
(L2) is the layer that makes L1 ergonomic *without* adding trusted complexity to it (KC-3).

The risk this RFC removes is the same one RFC-0006 removed for the whole language: if L2
conveniences accrete ad-hoc (in tests, the LSP, worked examples) without a governing elaboration
spec, the language acquires a de-facto semantics that was never designed, cannot be audited, and
may silently violate S1–S6. One governing document — written before the accretion begins — is
cheaper than retroactive cleanup.

The DN-09 KC-2 verdict also settles the design bias: **usability and clarity outrank theming** (§3.2).
L2 spellings follow Rust/ML conventions wherever they exist; novel spellings are adopted only where
they genuinely teach better. This aligns with the KC-2 evidence (familiar syntax retains LLM
leverage, RFC-0006 §8 Q1) and removes a recurring design ambiguity.

## 3. Guide-level explanation — what L2 adds and how it elaborates

L2 is a set of named desugarings. Nothing in L2 is new *meaning* — every L2 construct has an
equivalent L1 term that the elaborator produces and that the stage-dump channel (SC-5/M-140) can
display. A developer can always ask "what did this elaborate to?" and receive a fully-explicit L1
answer (S4/ADR-006).

### The elaboration contract

> **L2 elaboration is a meaning-preserving translation to L1, dumpable at every step, never
> inserting a `Swap`, and never hiding a failure.** The elaborated L1 term is content-addressed
> (ADR-003) and is the unit of identity, caching, and cross-module export. L2 syntax is metadata.

The elaborator is a **surface→L1 pass** that runs before the type checker, the totality classifier,
and the content-addresser — those tools see L1 only. The elaborator's output must satisfy all L1
well-formedness rules (W6/W7/W8); any elaboration failure is an **explicit diagnostic** (never a
partial artifact, never a silent fallback).

### Pedagogic example

```mycelium
nodule list_demo

type List<A> = Nil | Cons(A, List<A>)

fn length<A>(xs: List<A>) -> Binary{32} =
  match xs {
    Nil         => 0,
    Cons(_, rest) => length(rest)   -- nested recursive call; no type annotation needed
  }

fn sum(xs: List<Binary{32}>) -> Binary{32} =
  for x in xs, acc = 0 => add(acc, x)   -- for-sugar elaborates to Fix fold (RFC-0007 §4.8)
```

The type parameter `<A>` and the omitted type annotation on `length`'s recursive call are L2
inference (§4.2). The `Nil | Cons(_, rest)` pattern is L2 pattern sugar (§4.4, already flat here
but the elaborator handles nesting). The `for`-expression is L2 derived-form sugar (§4.5; enacted
in RFC-0007 §4.8). Nothing in this example inserts a `Swap`; the content hash of the elaborated L1
is the identity of these definitions (ADR-003, LR-3).

## 4. Reference-level design (normative once Accepted)

### 4.1 Invariants L2 must preserve (S1–S6 at the elaboration boundary)

These restate RFC-0006 §4.1 as **normative obligations on the L2 elaborator** — an elaboration
rule that violates any of them is ill-formed:

- **(S1 at L2) No sugar inserts a `Swap`.** The elaborator is Repr-transparent: no type-inference
  step, pattern-compilation step, module-resolution step, or derived-form expansion may introduce a
  `Swap` node. Any elaboration path that would require an implicit representation change must fail
  with an explicit `MissingConversion` (following RFC-0012 §4.4's model) rather than inserting one
  silently. This is the strongest invariant: it holds across *all* L2 constructs without exception.
- **(S2 at L2) Honesty tags surface through elaboration.** The guarantee lattice (`Exact ⊐ Proven
  ⊐ Empirical ⊐ Declared`) is preserved by every desugaring. The elaborator may not strengthen a
  tag (VR-5); a `Declared` annotation on any L2 expression propagates into the elaborated L1 term
  unmodified. Derived forms that synthesize L1 sub-terms assign the meet of their constituent tags
  as the resulting guarantee index.
- **(S3 at L2) Content-addressed identity is over the elaborated L1.** L2 syntax — identifier
  names, derived-form spellings, module paths — is metadata that resolves through elaboration. The
  content hash (ADR-003) is computed over the α-normalized L1 term, not the L2 source. Two L2
  programs that elaborate to the same L1 term have the same identity; renaming a variable is
  a cosmetic change that does not change the hash.
- **(S4 at L2) Every elaboration step is dumpable.** The stage-dump channel (SC-5/M-140) must show
  the L1 result of any elaboration — for any L2 expression in the editor or at the CLI, the
  developer can request the expanded, fully-explicit L1 form. The elaborator must not be a black
  box (ADR-006). This obligation extends into every sub-case: type-inference solutions, module
  resolutions, pattern compilations, and derived expansions must each be individually dumpable.
- **(S5 at L2) Explicit partiality is preserved.** Any L2 construct that might fail — a
  pattern-match that might be inexhaustive, a type-inference constraint that might be
  unsatisfiable, a module path that might not resolve — produces an explicit `Option`/`Result` or
  a compile-time diagnostic. L2 sugar may not erase a kernel refusal (G2). In particular, a
  non-exhaustive `match` in the surface must not become a silently partial `Match` in L1; it must
  either be rejected at check time (preferred) or wrapped in an explicit `Option` return type with
  the unmatched case returning `None`.
- **(S6 at L2) No AI or model dependency.** The L2 elaborator, type inferencer, module resolver,
  and pattern compiler are ordinary deterministic software. No inference step may delegate to a
  model; removing every AI component from the toolchain must leave the language fully operational.
  AI tooling sits above the elaboration pipeline (FR-S5, M-380) and may not influence it.

### 4.2 Type inference

**Scope.** v0 L1 is monomorphic (RFC-0007 §4.4): type checking is a simple bidirectional judgment
`Γ ⊢ e : τ` with no type variables in the kernel. L2 type inference therefore has two orthogonal
sub-problems:

1. **Monomorphic annotation inference** — inferring omitted type annotations on `let` bindings,
   function parameters, and intermediate expressions within a single, monomorphic definition.
2. **Polymorphic instantiation** — choosing a concrete instantiation of a type-parameterized
   declaration at a call site.

Sub-problem 1 is in scope for v0 L2: a **local, constraint-based inference pass** (HM-style
unification over the monomorphic type language) resolves omitted annotations within a definition
before handing the result to the L1 type checker. The elaborated L1 term carries fully-explicit
type annotations on every binding; the inference solution is the elaboration output, dumpable per
S4. Inference failure is an explicit diagnostic naming the conflicting constraints — it is never a
silent guess or a default.

Sub-problem 2 — instantiating a generic (`fn length<A>(…)` at a call site) — requires the trait
system (RFC-0019). It is **deliberately out of scope for this RFC**: v0 inference treats each
type-parameter occurrence as requiring an explicit instantiation annotation (a "deferred" error if
the annotation is omitted) until RFC-0019 defines the resolution rules. This is the same honest
"not a guess, an error" posture as RFC-0007 §4.4's treatment of generic instantiation.

**Invariants.** The inference pass:

- Never introduces a `Swap` (S1 — impossible to satisfy by inference, no polymorphism over `Repr`
  without LR-5's explicit bounds).
- Only *weakens* guarantee tags, never strengthens them (VR-5 / S2).
- Produces a fully-explicit monomorphic L1 type annotation for every binder; the result is
  dumpable (S4).
- Fails with a precise constraint-conflict diagnostic rather than defaulting (S5).

**What v0 inference covers:**

- Omitted return types inferred from the body expression's type.
- Omitted `let`-binding ascriptions inferred from the right-hand side.
- Omitted intermediate expression ascriptions within `app_expr` chains.
- Bidirectional propagation of known types into `match` arm bodies.

**Relation to RFC-0018 (stage-1 guarantee grading).** RFC-0018 extends the type system with a
static graded judgment for guarantee indices. When that RFC is ratified, the inference pass must
also infer guarantee indices (the 4-point lattice) under the FlowCaml-style rules RFC-0018
specifies. The obligation is recorded here so the L2 elaborator's architecture leaves a clean
extension point; the inference rules themselves are RFC-0018's normative content.

### 4.3 Modules — `nodule` and `use` (LR-3)

**The identity principle (ADR-003 / LR-3).** A `nodule` is a content-addressed unit: its identity
is the structural hash of its elaborated L1 definitions, not its file-system path. Separate
compilation falls out of this: two nodules whose definitions hash identically are the same nodule,
regardless of where the source lives. The name `nodule` is the committed DN-06 lexicon (Resolved
2026-06-16; supersedes the earlier `colony` for the static organizational unit).

**Surface additions (already in the committed grammar).** The grammar has:

```ebnf
nodule_header  ::= 'nodule' path
use_item       ::= 'use' path
```

The elaboration:

- A `nodule path` header declares the identity scope for the definitions that follow. Elaboration
  produces a registry entry for the nodule itself: its hash is computed over the set of
  content-addressed L1 definition hashes it transitively contains (the same Unison-style cycle
  hashing as RFC-0007 §4.2, applied at the nodule level).
- A `use path` item is a name-resolution directive. Elaboration resolves the path to a nodule hash
  in the known registry and imports the names of its exported definitions into the current scope. A
  path that does not resolve is an explicit `UnresolvedModule { path, site }` error (S5) — never a
  silent empty import.
- Exported definitions' parameter and return types are **concrete L1 types** (fully-elaborated,
  paradigm-explicit) — the ambient (RFC-0012 §4.5) never leaks across a module boundary, and the
  `use` site inherits the callee's concrete types, not its ambient defaults (ADR-016).

**Separate compilation.** Because identity is the content hash, a module's compiled artifact
(its registry entry and L1 term set) is cacheable and reusable with no dependency on file layout.
The elaborator checks that a `use`-imported name's definition hash matches the registry entry at
the call site; a hash mismatch is an explicit `ModuleHashConflict { name, expected, found }` error,
never a silent upgrade (LR-3 / ADR-003 — names are metadata, not identity).

**Phylum (reserved, not active).** `phylum` is the library-scale grouping above `nodule` (DN-06,
reserved-not-active). Its elaboration is deferred to the RFC that introduces it; this RFC reserves
the hook in the module elaboration architecture.

### 4.4 Pattern sugar

**The L1 target.** RFC-0007 §4.1 W7 defines the kernel `Match` node as **flat**: one scrutinee,
single-level constructor alternatives, at most one default. Nested patterns, guards, or or-patterns
do not exist in L1. The L2 elaborator is responsible for compiling all surface pattern richness
down to flat `Match` trees.

**Compilation algorithm.** The committed algorithm is **Maranget-style decision-tree compilation**
(T3.1; RFC-0007 §3; enacted for the L0 path in RFC-0011). The elaborator applies this algorithm to
the L2 `match` arms, producing a flat `Match(Match(…))` tree in L1. The compilation output is
dumpable per S4: the elaboration dump shows the decision tree, not just the flat L1 form.

The Maranget compilation is **usefulness-checked**: the elaborator rejects any pattern matrix that
contains a row that is never reached (useless pattern) or that fails to cover all constructors
(inexhaustive match). Both are explicit diagnostics, never silent. This preserves LR-1
(exhaustive `match`, no silent fall-through) and S5 (explicit partiality).

**Supported surface pattern forms and their elaboration:**

| Surface form | Elaboration to L1 | Notes |
|---|---|---|
| Wildcard `_` | A fresh `Var` binder unused in the arm body | Always total coverage for its position |
| Variable binder `x` | `Var(x)` binder in the flat alternative's binder list | Not a constructor check |
| Constructor `C(p₁, …, pₙ)` | Flat `Match` alternative over `#T#i` with nested patterns recursively compiled | The Maranget expansion; W6 saturation required |
| Nested constructor `C(D(y), z)` | Compiled to a nested `Match` tree: outer on `C`'s first field, inner on `D` | The multi-level expansion; dumpable per arm |
| Literal pattern (e.g. `0b101`) | Elaborated as a constructor match over the literal's repr type, or as an equality test via a builtin prim | Literal type must be ground at elaboration time; ambiguous width is an error |
| Or-pattern `p₁ \| p₂` *(reserved, not yet active)* | Would duplicate the arm body across both patterns in the Maranget matrix; reserved for a future revision | Not in v0 grammar; reserved as a direction |
| Default `_` arm | A `Match` default (RFC-0007 §4.1 W7); covers all remaining constructors | May not be reached if previous arms are exhaustive; usefulness check applies |

**W7 compliance.** Every `Match` node produced by the elaborator must satisfy W7: each alternative
binds exactly the constructor's arity, the same constructor appears at most once, and the
alternatives plus any default cover every constructor of the scrutinee type. The elaborator
enforces this; a violation is an internal elaboration error (not a user diagnostic), indicating a
bug in the pattern compiler itself.

### 4.5 Derived forms

Derived forms are L2 constructs that desugar to combinations of L1 nodes with no new kernel
concept. Each is listed with its normative elaboration rule and the L1 nodes it lowers to.

**`if`–`else if`–`else` chains.** The committed grammar provides `if_expr` with a single
`then`/`else`. Chained `else if` is L2 sugar:

```text
if e₁ then b₁ else if e₂ then b₂ else b₃
  ⤳
Match(e₁, [(True, b₁)], Some(Match(e₂, [(True, b₂)], Some(b₃))))
```

where `Bool` is a two-constructor type `True | False` in the standard registry (LR-1). The
elaboration uses the flat `Match` node; no `if` node exists in L1. The guarantee tag of the
elaborated `Match` is the meet of the tags of all branch bodies (S2).

**`let`-chains (`let … in let … in …`).** Already expressible in the grammar as nested
`let_expr`. No new syntax is introduced; the elaboration is simply nested `Let(x, e₁, Let(y, e₂,
body))` at L1. Recorded here because the elaborator must handle left-nesting correctly and must
infer types for each binder in dependency order (§4.2).

**`for`-loops.** Committed and enacted in RFC-0007 §4.8. The normative elaboration rule
(structural fold to a synthesized `Fix`) is RFC-0007 §4.8's, not redefined here. Recorded in this
RFC for completeness of the L2 inventory: `for` is an L2 derived form, its elaboration lowers to
`Fix`/`Match` L1 nodes, it is `Total` by construction, and it is inspectable per S4. No
elaboration change from RFC-0007.

**`grow Debug for T` and other derived trait implementations (provisional direction).** DN-03
reserves `grow` as the surface form for a compiler-derived implementation of a trait (analogous to
Rust's `derive`). The elaboration: given `grow Debug for T`, the elaborator synthesizes an
implementation function for each method in the `Debug` trait (LR-2/RFC-0019), producing L1 `Lam`
terms that pattern-match over `T`'s constructors and assemble the debug representation. The full
elaboration rules depend on RFC-0019's trait definition format; this RFC records the direction and
the L1 target (`Lam`/`Match`/`Construct` nodes, content-addressed like any hand-written
implementation) while deferring the normative elaboration to the traits RFC. The key constraint
from RFC-0019's perspective (stated here): a `grow`-generated implementation must be
**observationally equivalent** to the hand-written implementation a programmer would write for the
same trait, and it must be **dumpable** — the developer can inspect the synthesized L1 term (S4).

> **Note (2026-06-27 — keyword reconciled to `derive`; append-only, status unchanged).** The surface
> keyword for this compiler-derived form is now **`derive`**, not `grow`: **DN-38 §8.1** (Accepted
> 2026-06-26) ratified the conventional `derive` (over the coined `weave`), resolving the DN-03 §6
> `grow`-vs-`derive` overlap (see DN-03's 2026-06-27 changelog entry). The **`reveal`** inspector is the
> dumpability affordance referenced above (S4). The elaboration direction here is otherwise unchanged
> (observationally-equivalent, inspectable, content-addressed L1) — only the spelling moves `grow → derive`.
> §4.5 remains **carved-out/deferred** in this RFC's Accepted scope; normative elaboration still defers to
> RFC-0019 + DN-38. Tracker M-664 is re-scoped accordingly.

**`spore` expressions.** Already in the committed grammar:
`spore_expr ::= 'spore' '(' expr ')'`. Elaboration: produces a reconstruction manifest for a
value (RFC-0003 §6). The manifest is an L1 `Construct` value in the registry's manifest type. The
`spore` keyword is a committed DN-02/DN-03 themed form; the elaboration to L1 `Construct` is the
normative lowering. No guarantee-tag change: the manifest's tag is the tag of the enclosed
expression (S2).

### 4.6 Literal handling (Q6 — universal until elaboration)

**The committed position (RFC-0006 §8 Q6 / RFC-0012 §4.3).** Literals are
**universal-until-elaboration**: their representation family is not determined by syntax alone.
Specifically:

- **Representation-tagged literals** (`0b1011` for binary, `<+0->` for ternary) name their
  paradigm explicitly; elaboration assigns the repr type directly from the literal form with no
  inference. Their width/dimension is determined by the literal's content (bit-count for `BinLit`,
  trit-count for `TritLit`).
- **Bare integer literals** (`42`, `-7`) have no paradigm family on their own. Elaboration:
  (1) if an ambient paradigm is in scope (RFC-0012), the literal adopts the ambient's paradigm and
  takes its width from the checked context (bidirectional type information); (2) if no ambient is
  in scope, the literal must be ascribed an explicit type (`42 : Binary{8}`) or it is an
  `UnresolvedWidth` error — there is **no default repr family and no default width** (stricter
  than Rust's `i32` default; no cross-family defaulting, per Q6). This is enforced by the
  elaborator before the L1 type checker runs.
- **List literals** (`[e₁, …, eₙ]`) elaborate to a chain of `Construct` applications over a
  linearly-recursive `List<A>` type in the standard registry. The element type is inferred
  (§4.2) or must be ascribed. An empty list literal `[]` requires an explicit type ascription (the
  inferred `A` would be unconstrained — an error, not a default).

**No Ada-style VSA literal functions in v0.** RFC-0006 §8 Q6 mentions "Ada-2022-style literal
functions for `VSA{…}` construction" as a direction. This is **out of scope for v0 L2**:
VSA-literal elaboration (translating a surface literal into a `VSA` vector) requires the semantic
machinery of RFC-0003 and the trait system (RFC-0019). Recorded as a tracked direction here.

### 4.7 Ambient paradigm sugar (RFC-0012 — the enacted model)

The ambient paradigm (`default paradigm P` / `with paradigm P { … }`, enacted RFC-0012 M-344) is
the canonical example of an L2 surface convenience: it is a **pure elaboration** that fills
omitted paradigm tags and bare-literal encodings, elaborates away completely, and produces
**identical L0** to the longhand form (RFC-0012 §4.3 I1/I2). It is the model example because it
instantiates all six L2 invariants clearly:

- S1: the ambient never inserts a `Swap` (I1 — normative invariant, defended by the §4.6
  differential).
- S2: the ambient is orthogonal to the guarantee lattice (RFC-0012 §4.7 — it never reads or writes
  `Meta`).
- S3: identity is over the expanded L0 (I2 — the hash is computed over the longhand twin).
- S4: `EXPLAIN`/M-142 renders the resolved longhand form on demand; `ResolutionNote` provenance is
  returned per fill.
- S5: missing conversion is an explicit `MissingConversion`; unresolved ambient is an explicit
  `UnresolvedAmbient`; ill-fitting shape is an explicit `ParadigmShapeMismatch` — never silent.
- S6: the resolution pass is deterministic software; no model inference involved.

No new normative content on the ambient is introduced by this RFC. RFC-0012 is the governing spec;
this section records where it fits in the L2 surface inventory.

### 4.8 Usability-first design bias (DN-09 §3.2)

DN-09 §3.2 records a standing **maintainer direction** for all L2/L3 surface work:
**usability and clarity outrank the fungal metaphor.** Spelled out as a design criterion for
future L2 elaboration rules and syntax additions:

1. **Default to the familiar Rust/ML spelling.** Where a Rust or ML convention exists and is
   unambiguous, adopt it as the L2 spelling. The programmer population Mycelium targets has prior
   context with these forms; familiar forms retain LLM leverage (RFC-0006 §8 Q1 — the KC-2
   evidence; familiar-skinned syntax is the hypothesis the data supports). Examples: `type`, `fn`,
   `let`, `match`, `trait`, `use` are all conventional; they were chosen over themed alternatives.

2. **Adopt a themed spelling only where it teaches better.** The DN-02 three-test gate still
   applies: a themed word is adopted *only* if it (a) is unambiguous in context, (b) is grounded
   in the vocabulary the corpus already uses, and (c) genuinely teaches the underlying concept
   better than the conventional word. Examples where theming meets the bar: `nodule` (the static
   compilation unit — teaches that modules are content-addressed nodes in a hyphal graph; the
   conventional `module` would not); `wild` (the unsafe escape hatch — teaches that it is
   structurally alien to safe Mycelium, not just a lint); `spore` (reconstruction manifest — the
   fungal lifecycle metaphor is doing real explanatory work about reconstruction-from-spore). The
   `germinate` → `thaw` replacement (RFC-0017 §5) is the precedent for *reverting* a theming
   choice when the plain word is clearer and the themed word is already taken.

3. **The tie-breaker is the boring, familiar word.** When both conventions are plausible and no
   teaching advantage is clear, pick the conventional word (append-only — a later revision may
   revisit if evidence accumulates, but the default is familiar).

4. **Novel type-system features are where Mycelium is novel** (RFC-0006 §4.2: "beyond Rust" = LR-5 +
   LR-6, not exotic syntax). The surface should be boring *because* the type system is
   interesting — theming the surface on top of a novel type system compounds novelty rather than
   controlling it.

This criterion is **not retroactive normative review** of committed spellings; it is the governing
principle for *new* additions. Committed forms are append-only; revisiting them is an explicit
recorded decision.

### 4.9 Conformance corpus plan (RFC-0006 §4.3)

RFC-0006 §4.3 requires each layer to have a machine-readable grammar artifact and a
**conformance corpus** (`accept/` + `reject/` programs) that the elaborator and formatter are
tested against. The L2 layer extends this requirement to the elaboration pass itself.

**What the L2 conformance corpus covers:**

1. **Accept/elaborate programs.** Programs that (a) parse against `mycelium.ebnf`, (b) elaborate
   successfully through the L2 pass, and (c) type-check at L1. For each program the corpus records
   the expected elaborated L1 term (content hash + structural dump) so a conformant elaborator
   must produce the identical L1 output. This is the machine-checkable statement of S3 and S4.

2. **Reject/fail-elaborate programs.** Programs that parse (are grammatically valid L3) but fail
   the L2 elaboration pass with a specific diagnostic code. For each such program the corpus
   records the expected diagnostic (e.g. `UnresolvedModule`, `InexhaustiveMatch`,
   `UnresolvedWidth`, `MissingConversion`, `TypeMismatch`). A conformant elaborator must produce
   that diagnostic, not a different one and not a silent skip.

3. **Elaboration-dump roundtrips.** For each accept program, the corpus records the canonical
   formatter (M-142) rendering of the elaborated L1. A conformant elaborator's dump must
   round-trip through the formatter to the canonical form (S4 dumpability, S3 content identity).

**Grammar artifact.** The L2 layer does not add a new grammar file (the surface grammar is the
committed `mycelium.ebnf`); the conformance corpus lives under `docs/spec/grammar/conformance/l2/`
with the same `accept/` / `reject/` structure as the existing v0 conformance corpus. The L2
sub-directory is opened when the elaborator prototype is ready to validate against it; this RFC
records the plan now so the architecture is not ad-hoc.

**Canonical formatter scope.** M-142 (the α-normalizing canonical formatter) extends to L2 by
rendering the *elaborated L1* form as the canonical reading of any L2 expression. The L2 source
form (with omitted type annotations, syntactic sugar, module shorthand) is a **presentation form**
that round-trips through the formatter's "expand to L1" mode. Both forms are valid outputs; the
canonical form is the L1 expansion (S4). The formatter is not defined or implemented by this RFC;
this section records the extension obligation so M-142's design accommodates it.

## 5. Drawbacks

- **Another elaboration pass before the type checker.** The L2 pass is real machinery that can
  have bugs; a buggy elaborator that violates S1 or S4 is not caught by the L1 type checker (which
  only sees the elaborated output). Mitigations: the conformance corpus (§4.8) is the executable
  specification; the differential (NFR-7 / M-210) catches elaboration divergences; and every
  elaboration rule in this RFC is expressed as a normative L1 output obligation, checkable against
  the ten-node grammar.

- **Type inference adds non-local complexity.** HM-style unification means that a type error in
  one part of a definition can produce a reported error elsewhere. Mitigations: v0 inference is
  **within-definition only** (no cross-definition inference) and the error message must cite the
  conflicting constraints with sites; "type error at the call site, not inside the definition" is a
  conformance property of the error reporter.

- **Pattern compilation can blow up in the decision tree.** Maranget compilation of large pattern
  matrices is polynomial in common cases but exponential in degenerate ones. Mitigation: the
  elaborator is bounded to emit an explicit `PatternMatrixTooLarge` refusal for matrices exceeding
  a configurable bound, rather than hanging or producing a runaway L1 term. The refusal site names
  the problematic `match` expression.

## 6. Rationale & alternatives

- **Why elaboration-defined L2 (vs a directly-specified big language)?** KC-3: the trusted base
  stays L0/L1; every L2 convenience is reducible and inspectable (S4). This is the GHC Core / Rust
  HIR→MIR→LIR lesson, and it matches the repo's existing lowering discipline (RFC-0006 §6).

- **Why monomorphic-first inference (vs full HM from the start)?** v0 L1 is monomorphic. Shipping
  polymorphic inference before the trait system (RFC-0019) is defined would require speculating on
  coherence and resolution rules that are RFC-0019's normative content. The honest sequencing:
  monomorphic inference now; polymorphic inference (which is inference + RFC-0019 trait resolution)
  as a single, well-grounded extension.

- **Why content-hash module identity (vs path-based import)?** LR-3 / ADR-003: path-based identity
  makes refactoring unsafe (renaming a file changes identity), creates import-order sensitivity,
  and couples compilation to file-system layout. Content-addressed identity gives free caching,
  safe refactoring, and cross-module type-compatibility checks that do not depend on build order.
  This is Unison's core insight applied to Mycelium's module layer.

- **Why Maranget compilation (vs backtracking matching)?** The kernel `Match` is flat by design
  (W7; RFC-0007 §3). Maranget produces a flat decision tree with no backtracking, which is exactly
  W7's shape, and its usefulness analysis catches both exhaustiveness and useless patterns in one
  pass (T3.1). The alternative (runtime backtracking) would require a `Match` semantics more
  complex than W7 and would weaken the totality argument for matched functions.

- **Why `nodule` not `module`?** Adopted in DN-06 (Resolved 2026-06-16). `nodule` teaches that
  the compilation unit is a content-addressed node in a hyphal graph, not a file-system namespace.
  `module` has strong path-based-identity connotations from every major language that uses it,
  which would be actively misleading here (ADR-003 modules are content-addressed; `module` primes
  the wrong mental model). This is one of the cases where theming meets the §4.8 bar.

## 7. Prior art

- **GHC** (surface Haskell → Core → STG → Cmm): the canonical example of elaboration-defined L2
  with a small trusted core. Core's ten constructors, flat `Case`, and explicit type applications
  are the direct ancestor of RFC-0007's L1. GHC's "show Core" (`-ddump-simpl`) is the prior art
  for S4's stage-dump obligation.
- **Rust** (HIR → MIR → LIR): staged lowering, pattern compilation (the MIR `Terminator::SwitchInt`
  is a flat match), monomorphization as an elaboration step (the analogue of RFC-0019's
  instantiation). Rust's `--emit=mir` is the prior art for inspectable intermediate elaboration.
- **Lean 4** (surface → term → kernel): elaboration as a trusted pass that generates fully-explicit
  terms for the kernel, with `#check` and `#print` as inspection commands (S4 prior art). Lean's
  `Syntax → Expr → kernel` architecture is the closest prior to L3→L2→L1→L0.
- **Unison** (content-addressed names, projection-based editor surface): ADR-003's source; the
  module-as-hash-of-definitions identity model (§4.3) is directly from Unison's codebase.
- **Maranget 2008** ("Compiling Pattern Matching to Good Decision Trees", ML Workshop 2008): the
  normative algorithm for §4.4 pattern compilation (also RFC-0011's source).
- **Standard ML / OCaml / F#**: the ML family provides the prior art for HM-style type inference
  (§4.2) in a strict, value-semantics context. OCaml's module system (with its explicit functor
  application) is the prior art for explicit instantiation — Mycelium's approach in v0 (explicit
  instantiation annotation, error on ambiguity) is closer to this than to Haskell's `instance`
  resolution.

## 8. Unresolved questions

- **R20-Q1 (polymorphic inference boundary with RFC-0019).** The exact interface between the L2
  inference pass and the RFC-0019 trait resolver is not yet defined. Specifically: when the
  inferencer encounters a generic call site, does it emit a deferred unification variable (to be
  resolved by the trait resolver), or does it fail immediately and require an explicit annotation?
  The v0 answer (explicit annotation required, deferred = error) is conservative and honest; the
  question is whether to relax this as part of RFC-0019 or as a later revision of this RFC. Flagged
  as an **interface question between RFC-0020 and RFC-0019**, to be resolved in one of the two RFCs
  (whichever lands later).

- **R20-Q2 (guarantee-grade inference under RFC-0018).** RFC-0018 specifies the static graded
  judgment for guarantee indices. The L2 inference pass must integrate these grades; the question
  is whether grade inference is a **separate pass** (after monomorphic type inference) or
  **interleaved** with it (a graded HM algorithm). RFC-0018's normative content will determine
  this; recorded here as the integration point so RFC-0018's authors know to address it.

- **R20-Q3 (or-patterns in surface `match`).** Or-patterns (`p₁ | p₂ => body`) are a common
  convenience (Rust, OCaml, Haskell `{}` multi-head). They are **not in v0 grammar** but are a
  natural L2 derived form (Maranget compilation duplicates the body across both alternatives, or
  the elaborator hoists the shared body into a `let` binding). Reserved as a tracked direction;
  the decision (adopt / don't adopt / adopt with body-sharing rule) is a future revision.

- **R20-Q4 (mutual recursion in `nodule` elaboration).** RFC-0007 §8 R7-Q3 deferred the surface
  elaboration of mutually-recursive definition groups (the hash identity is fixed; the elaboration
  is not). When a `nodule` contains a mutually-recursive group of `fn` definitions, the L2
  elaborator must produce the correct `FixGroup` L1 node (RFC-0001 r5) and content-address it per
  RFC-0007 §4.2's cycle hashing. The question is whether this elaboration is part of this RFC's
  scope or is deferred to the RFC-0001 revision that fully lands `FixGroup`. Recorded here;
  currently the elaborator emits an explicit `MutualRecursionDeferred { names, site }` error (the
  same honest-refusal posture as RFC-0007 §4.6's `Residual` before r3/r4).

- **R20-Q5 (list-literal element-type inference in presence of a `for`-body).** A `for`-loop over
  a list literal where the accumulator body constrains the element type creates a bidirectional
  inference problem: the list literal's element type flows into the `for`-body, and the body's
  expected type flows back into the literal. The v0 inference pass handles this if the literal is
  typed before the `for` is elaborated; the question is whether the elaboration order guarantees
  this or whether a two-pass inferencer is needed. Conservative v0 answer: elaborate the literal
  with an explicit annotation requirement; the two-pass relaxation is recorded as a tracked
  improvement.

## 9. Future possibilities

- **Polymorphic inference across definitions** — once RFC-0019 ratifies trait coherence, the
  inferencer can resolve generic instantiations automatically, removing the explicit-annotation
  requirement at call sites.
- **Or-patterns** (R20-Q3) — a natural L2 derived form once the Maranget compilation path is
  exercised.
- **Row-polymorphic effects** — RFC-0006 Q4 records divergence-only tracking as the v0 posture
  with a growth path to a small fixed row. When that growth happens, L2 inference must infer
  effect rows; the extension point in the inference architecture is recorded here.
- **`grow`/derived-trait implementations** (§4.5) fully specified once RFC-0019 lands and the
  synthesized L1 term format for trait implementations is normative.
- **LSP "show elaborated L1"** as a first-class editor feature (FR-S5 / M-380) — S4's dumpability
  obligation becomes a user-visible IDE command, not just a CLI flag. The M-380 projection layer
  (DN-09 §3.1) is the natural home for this rendering.
- **Ambient width defaults** (RFC-0012 §9 — a per-axis default above the paradigm) as a later L2
  convenience, if the need becomes concrete.

## 10. Ratification scope

> **RATIFIED (scoped) (2026-06-18, maintainer; DN-12).** The "Ready" core below
> (§4.1/§4.3/§4.4/§4.6/§4.7/§4.8/§4.9) is **Accepted** — RFC-0020 carries no research gate. The
> "Deferred" items below are **carved out** (deferred, not objections), exactly the RFC-0006 r5
> pattern; each unblocks via the now-Accepted **RFC-0018** (R20-Q2 grade inference), the now-Accepted
> **RFC-0019** (R20-Q1 polymorphic inference, §4.5 `grow`), and the enacted **RFC-0001 r5 `FixGroup`**
> (R20-Q4 mutual recursion; surface front-end is M-391). The §4.9 conformance corpus is a *testing*
> deliverable that follows ratification, not a gate. The original split is retained below verbatim
> (append-only).

This is a **Draft**. Ratification is a maintainer decision, append-only. The split of what is
ready for ratification vs. what is explicitly deferred:

**Ready (KC-2-independent, grounded in existing Accepted RFCs):**

- §4.1 invariants (S1–S6 restated at the L2 elaboration boundary) — restatements of RFC-0006
  §4.1; no new content.
- §4.3 modules — `nodule`/`use` elaboration to content-addressed registry; `UnresolvedModule`/
  `ModuleHashConflict` refusals; ADR-003/LR-3 compliance.
- §4.4 pattern sugar — Maranget compilation to flat L1 `Match`; usefulness/exhaustiveness checks;
  the supported surface forms table.
- §4.6 literal handling — universal-until-elaboration (Q6, discharged by DN-09); ambient
  integration (RFC-0012); `UnresolvedWidth` refusal; no default repr family.
- §4.7 ambient paradigm — records RFC-0012 as the enacted model; no new content.
- §4.8 usability-first design bias — records DN-09 §3.2 as the governing principle; no new
  content.
- §4.9 conformance corpus plan — extends the RFC-0006 §4.3 discipline to L2; describes, does not
  implement.

**Deferred / not ratified by accepting this Draft:**

- §4.2 type inference (polymorphic instantiation) — deferred to RFC-0019's ratification (R20-Q1);
  v0 monomorphic inference is ready, polymorphic is not.
- §4.5 `grow`/derived-trait elaboration — deferred to RFC-0019 (requires trait definition format).
- R20-Q2 (guarantee-grade inference integration) — deferred to RFC-0018.
- R20-Q3 (or-patterns) — unresolved, reserved direction.
- R20-Q4 (mutual-recursion elaboration) — deferred to the RFC-0001 `FixGroup` revision.
- R20-Q5 (list-literal bidirectional inference) — conservative v0 answer in place; relaxation is
  a tracked improvement.

**Status line (Draft, not in force):** *Draft — L2 invariants (S1–S6 at elaboration boundary),
module elaboration (§4.3), pattern compilation (§4.4), literal handling (§4.6), usability bias
(§4.8), and conformance corpus plan (§4.9) are ready for ratification; polymorphic inference,
derived-trait elaboration, grade-inference integration, or-patterns, and mutual-recursion
elaboration are explicitly deferred.*

## Meta — changelog

- **2026-06-18 — Draft.** Initial draft: L2 surface term language scope, invariants (S1–S6
  restated at the elaboration boundary), and normative elaboration rules for type inference (§4.2,
  monomorphic v0; polymorphic deferred to RFC-0019), modules (§4.3, `nodule`/`use`,
  content-addressed identity per ADR-003/LR-3), pattern sugar (§4.4, Maranget compilation to flat
  `Match`), derived forms (§4.5, `if`-chains/`let`-chains/`for`/`grow`/`spore`), literal handling
  (§4.6, universal-until-elaboration Q6), the ambient paradigm as the enacted model example (§4.7,
  RFC-0012), usability-first design bias (§4.8, DN-09 §3.2), and a conformance corpus plan (§4.9,
  extending RFC-0006 §4.3). Sibling relationships recorded: RFC-0018 (guarantee grading, R20-Q2),
  RFC-0019 (traits, R20-Q1), RFC-0021 (L3 projections). Ratification is a maintainer decision;
  status is Draft, not Accepted.
- **2026-06-18 — Ratification-readiness assessed (DN-12).** `docs/notes/DN-12-RFC-0020-Ratification-Readiness.md`
  records that RFC-0020 carries **no research gate** (unlike RFC-0018/0019): its deferred items depend
  on sibling RFCs, two of which had their research gates discharged this pass (R20-Q1 → RFC-0019/RP-3,
  R20-Q2 → RFC-0018/RP-2) and one on the enacted RFC-0001 r5 `FixGroup` (R20-Q4). DN-12 recommends a
  **scoped** ratification (the §4.1/§4.3/§4.4/§4.6/§4.7/§4.8/§4.9 core) with a carve-out for the
  deferred sections — the RFC-0006 r5 precedent. Advisory; status unchanged (Draft). Append-only.
- **2026-06-18 — ACCEPTED (scoped) (maintainer ratification; DN-12).** The §4.1/§4.3/§4.4/§4.6/§4.7/
  §4.8/§4.9 core is Accepted (no research gate). The deferred sections (§4.2 polymorphic + grade
  inference, §4.5 `grow`, R20-Q1…Q5) are carved out — each now unblocking via the same-day-Accepted
  RFC-0018/RFC-0019 and the enacted RFC-0001 r5 `FixGroup`. The §4.9 conformance corpus follows
  ratification (a testing deliverable, not a gate). Append-only.
