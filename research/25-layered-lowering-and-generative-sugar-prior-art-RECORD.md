# Layered Lowering & Generative-Sugar — Prior-Art RECORD

*Backs DN-38. Two external research passes (one re-run clean after the first agent malfunctioned by spawning sub-agents). Part 1 = pit-of-success + nanopass/language-tower; Part 2 = inspectable desugaring, generative lowering, value-semantic delegation, content-addressed generation, lowering verification.*

---

## Part 1 — Pit of Success + Language Tower / Nanopass

# Prior Art — Pit of Success (DX) + Language Tower / Nanopass (verified layered lowering)

## A — The "Pit of Success" DX principle
- **Rico Mariani** (coinage, Microsoft) / **Brad Abrams** (popularized, .NET, 2008): "we want our customers to simply **fall into winning practices** … make it easy to do the right thing and hard to do the wrong thing." Inverse anti-pattern = "pit of despair/failure" (easy path → bugs).
  - https://blog.codinghorror.com/falling-into-the-pit-of-success/ ; https://learn.microsoft.com/en-us/archive/blogs/brada/the-pit-of-success
- **Progressive disclosure** (Nielsen / NN/g, 2006): defer advanced/rarely-used features to a secondary surface → improves **learnability, efficiency, error rate**. "Show users only the information they need when they need it."
  - https://www.nngroup.com/articles/progressive-disclosure/
- **"Make illegal states unrepresentable"** (Yaron Minsky, Jane Street, *Effective ML*): encode invariants in the type structure (sum types over loosely-coupled fields) so the compiler rejects invalid states — the *type-system* form of pit-of-success (the only compilable path is the correct one).
  - https://blog.janestreet.com/effective-ml-revisited/ ; https://fsharpforfunandprofit.com/posts/designing-with-types-making-illegal-states-unrepresentable/
- Synthesis: pit-of-success = the easy/default/lowest-effort path is *also* correct+safe. Type-form (illegal states unrepresentable) + surface-form (progressive disclosure) both **teach correct usage through the structure of the tool**, not docs/discipline.

## B — Language Tower / "Languages as Libraries"
- **"Languages as Libraries," PLDI 2011** (Tobin-Hochstadt, St-Amour, Culpepper, Flatt, Felleisen): Racket's **tower of languages**, each a library; `#lang L` selects a language = a module exporting syntax/semantics (reader + macros). Language extension = library extension. Surface `#lang` → macro expansion → fully-expanded tiny core (`#%kernel`).
  - https://www2.ccs.neu.edu/racket/pubs/pldi11-thacff.pdf
- **"A Programmable Programming Language," CACM 2018** (Felleisen et al.): language-oriented programming — build a *tower of small domain-tuned languages*, each lowering into the next.
  - https://cacm.acm.org/research/a-programmable-programming-language/

## C — Nanopass (many tiny verified lowering passes)
- **"A Nanopass Infrastructure," ICFP 2004** (Sarkar, Waddell, Dybvig): "a compiler comprised of many single-task passes with a **well-defined intermediate language between each pass**." (1) ILs are **formally specified**; (2) each pass threads only relevant parts; (3) one pass = one task. Framework **auto-generates verification passes** checking each pass's output conforms to its IL grammar.
  - https://legacy.cs.indiana.edu/~dyb/pubs/nano-jfp.pdf
- **"A Nanopass Framework for Commercial Compiler Development," ICFP 2013** (Keep & Dybvig): scales to production — **Chez Scheme = 50+ nanopasses**. Each IL defined as a *delta* from the previous one (specify only what changes).
  - https://dl.acm.org/doi/10.1145/2500365.2500618 ; https://nanopass.org/
- **Why a verified chain of small lowerings beats one big elaboration** (grounded synthesis):
  1. **Independent checkability** — each pass has a specified input/output IL; auto-generated checker validates conformance (a monolithic pass leaves internal invariants unchecked).
  2. **Narrow invariants / local reasoning** — one construct per pass → small auditable correctness argument.
  3. **Per-pass differential testing** — each IL is a concrete runnable representation; can run a reference interpreter at each level to check observational equivalence pass-by-pass.
  4. **Failure isolation** — a bug is localized to one tiny pass, not a 5000-line elaboration.
  5. **Maintainability / kernel-never-grows** — new surface features add an *early* pass; the small core + its proofs are untouched (exactly KC-3).
  6. **Spec = IL grammar** — IL definitions are machine-checked contracts between passes; the lowering is *honest* because every intermediate stage is a typed/grammar-checked artifact.
- **CompCert bridge** (Leroy, CACM 2009): ~20 passes over ~10 IRs, each with small-step semantics + a forward-simulation proof — small passes with well-defined IRs are exactly what makes per-pass semantic-preservation proofs (or translation validation) tractable.
  - https://dl.acm.org/doi/10.1145/1538788.1538814

## Mycelium read
- The L0–L3 layering + KC-3 ("kernel never grows") + RFC-0012 ("lowering is observationally the identity") + NFR-7 (interp≡AOT differential) is **precisely the nanopass/language-tower thesis**: a tower of small, grammar-checked, semantics-preserving lowering passes bottoming out at a tiny frozen core. The academic backing is strong and direct.
- The "pit of success" + progressive disclosure + illegal-states-unrepresentable give the DX law for the generative sugar: the terse path must be the correct path, and the desugaring must be *inspectable* (Racket Macro Stepper / `cargo expand` precedent) so it teaches rather than hides.

---

## Part 2 — Inspectable Desugaring · Generative Lowering · Value-Semantic Delegation · Content-Addressed Generation · Lowering Verification

# Prior-Art Research — Inspectable Desugaring & Generative Lowering for Mycelium

**Scope.** External prior-art for a Mycelium design note. Covers exactly five topics:
(1) inspectable desugaring / "see the expansion" tooling; (2) generative lowering from terse
intent (GOOD vs OPAQUE); (3) value-semantic delegation/forwarding without aliasing; (4)
content-addressed generated artifacts ("same intent → same identity"); (5) semantics-preserving
lowering verification (desugar/macro angle only). The pit-of-success DX principle and the
nanopass/language-tower verified-lowering backbone are **already** researched and are NOT re-covered.

**Mycelium premise carried throughout.** Immutable, acyclic, content-addressed, value-semantics
language with L0–L3 layering: an ergonomic surface desugars/lowers to a tiny frozen functional L0
core; the kernel never grows. Where this premise makes a pattern *easier* or *constrains* it is
flagged inline with **[EASIER]** / **[CONSTRAINS]**.

Grounding convention (house rule 1/4): each claim is tagged with the strength its source supports.
`Declared` = asserted by a vendor/author doc; `Empirical` = observed behavior / community-reported;
`Proven` reserved for checked theorems (used sparingly, e.g. translation-validation results).

---

## Topic 1 — Inspectable desugaring / "see the expansion" tooling

The universal principle across every mature metaprogramming ecosystem: **the lowering must be
*viewable*, never hidden.** Each toolchain ships a first-class way to render the post-expansion
form back to the developer.

### Rust — `cargo expand`
`cargo expand` "prints out the result of macro expansion and `#[derive]` expansion applied to the
current crate," optionally re-formatted with `rustfmt` so the output is more readable than the raw
compiler dump. It covers **both** `#[derive]` and procedural macros. (`Declared`, dtolnay README.)

Critical caveat the README itself states: **"Macro expansion to text is a lossy process. This is a
debugging aid only. There should be no expectation that the expanded code can be compiled
successfully, nor that if it compiles then it behaves the same as the original code."** The lossiness
is precisely *hygiene*: textual expansion cannot faithfully render Rust's hygienic scopes, so a
text-only "see the expansion" view is an approximation, not the ground truth. (`Declared`/`Empirical`.)

> **Mycelium takeaway:** a text-dump expander is necessary but *insufficient* for a transparency
> guarantee. The viewable form must be the **same artifact the kernel actually runs** (a real L0
> term), not a lossy textual re-render — otherwise "see the expansion" silently lies. This is the
> never-silent rule applied to the inspector itself.

### Racket — Macro Stepper (macro-debugger)
The strongest prior art for *faithful* inspection. The Macro Stepper "shows the programmer the
expansion of a program as a **sequence of rewriting steps**," with forward/back/jump navigation
through the expansion, rendered by a syntax browser. (`Declared`, Racket docs; Culpepper, *A Stepper
for Scheme Macros*, 2006.) Distinctive properties beyond a text dump:
- **Hygiene made visible:** the browser uses colors keyed to syntax *marks* — "two syntax subterms
  [get] the same color if they have the same marks" — so a developer can *see* which identifiers
  were original vs macro-introduced, and `bound-identifier=?`/`free-identifier=?` equivalence classes
  are highlightable. (`Declared`.)
- **Source locations + binding info** in a properties panel; the docs are explicit that binding info
  is only *finalized* after full expansion (a subtlety any stepper must surface honestly).
- **Macro hiding:** the developer can mark chosen macros *opaque* — "see how expansion would look if
  certain macros were actually primitive syntactic forms" — stepping over them while still expanding
  their subterms. This is the level-of-detail control that makes a tower of passes navigable.

> **Mycelium takeaway:** the *gold standard* to emulate. Mycelium's L0–L3 tower maps directly onto
> "macro hiding": let a developer view lowering **at a chosen layer** (stop at L2, or fully to L0),
> stepping pass-by-pass. **[EASIER]** Because L0 is a real frozen core (not lossy text), the stepper
> can show *actual* intermediate terms with real binding structure — Racket has to reconstruct this;
> Mycelium gets it from the IR for free.

### Lean 4 — `set_option pp.all`, delaboration/unexpansion, `#print`
Lean inspects the *elaborated core* rather than a textual macro output. `set_option pp.all true`
turns on maximal pretty-printing; the **delaborator** turns fully-elaborated `Expr` core terms back
into surface `Syntax`, and an **unexpansion** post-pass "tries to reverse macro expansions." The docs
claim `pp.all` should make the delaborator **injective** so re-elaborating the printed `Syntax`
**round-trips**. (`Declared`, Lean metaprogramming book / reference.) `#print` shows a definition's
elaborated form.

Lean's hygiene is a *checked* property, not a convention: the paper *Hygienic Macro Expansion for
Theorem Proving Languages* (Ullrich & de Moura, arXiv:2001.10490) builds a Scheme-derived hygienic
macro system specifically because **accidental name capture "often produces unexpected and
counterintuitive behavior"** and, in a prover, captured names can invalidate proofs. Hygiene there is
a correctness requirement, not ergonomics. (`Declared`, abstract.)

> **Mycelium takeaway:** the delaborate→unexpand→**round-trip** discipline is the model for a
> *certified* "see the expansion": print the L0 term such that re-lowering the printed surface
> reproduces it. That round-trip is itself a transparency check (VR-5) — and aligns with Mycelium's
> `certified` mode (RFC-0034/ADR-032).

### Template Haskell — `-ddump-splices`
`-ddump-splices` "shows the expansion of all top-level declaration splices, both typed and untyped,
**as they happen**," e.g. `Splicing expression nth 3 5 ======> \(_,_,x,_,_) -> x`. Pairs with
`-ddump-to-file`. (`Declared`, GHC users guide.) A straightforward, well-scoped "show me what the
splice generated" flag.

### Scala 3 — `-Xprint:<phase>`, inline, reflection
Scala 3 can print the tree after any compiler phase (`-Xprint:...`), and its metaprogramming is
layered: `inline` (source-visible inlining) → quote-and-splice macros (type-safe) → a reflection API
(weaker typing, finer control over inspecting/transforming code). The progression itself is a
transparency gradient: prefer `inline` (most visible) and escalate only when needed. (`Declared`,
Scala 3 docs.)

**Cross-cutting principle (Topic 1).** Every serious system ships an expansion viewer; the *quality*
ladder runs: lossy text dump (`cargo expand`, `-ddump-splices`) → core-term print with round-trip
(Lean) → **stepped, hygiene-aware, layer-hideable** browser (Racket). Mycelium should target the top
of that ladder because its IR is a real frozen core, not reconstructed text.

---

## Topic 2 — Generative lowering from terse intent: GOOD vs OPAQUE

The question is *not* "should terse intent generate code" (every modern language does) but **what
makes the generation legitimate.** The dividing line, drawn from the cases below, is a single rule.

### GOOD — explicit, inspectable, reusable artifacts

- **Rust `#[derive(...)]`.** Terse intent (`#[derive(Clone, PartialEq)]`) generates real `impl`
  blocks that are ordinary Rust, fully recoverable via `cargo expand`, and behave like
  hand-written code. The generated output is a *first-class, inspectable artifact*. (`Declared`/`Empirical`.)
- **Kotlin `by` delegation.** `class Derived(b: Base) : Base by b` — "the compiler generates all the
  methods of `Base` that forward to `b`." Terse intent (`by b`), explicit forwarding semantics, no
  hidden mutation; delegated properties generate a visible `prop$delegate` backing. (`Declared`,
  Kotlin docs.) **Note for Topic 3:** this is *forwarding by held value*, the value-semantic-friendly
  form of delegation.
- **Go struct embedding.** Embedding promotes the embedded type's fields/methods into the outer
  struct — composition-over-inheritance with **no runtime indirection magic**: promotion is a
  visible, statically-resolved forwarding, and you can always name the embedded field explicitly.
  (`Declared`, Go docs/community.)
- **Zig `comptime`.** Andrew Kelley designed `comptime` explicitly **to remove C macros**: generic
  functions are ordinary functions with `comptime` parameters; "the programmer writes ordinary,
  readable Zig" and the compiler specializes. No separate macro sublanguage, no hidden token surgery
  — the generator *is* the same language, executed earlier. (`Declared`, Loris Cro / Zig docs.)
- **Scala 3 `inline`/macros.** Source-visible inlining first; quote-and-splice (type-checked) before
  reflection. Visibility is the default, opacity the escalation. (`Declared`.)

### OPAQUE — the anti-pattern: Lombok
Project Lombok is the canonical *bad* case and the most-cited warning. It **modifies the compiler's
AST at compile time**, so "the code you write isn't the code that runs"; it "generat[es] the complete
bytecode for methods while the source file remained clean," producing an effect "like magic — or to
some purists, like cheating." Concrete harms reported:
- **No source artifact.** There is no generated `.java` to read or check in; the methods exist only in
  bytecode. (`Empirical`, widely reported.)
- **Undebuggable.** "Classes with Lombok annotations hide generated methods that you can't step
  through during debugging," so you debug code that doesn't exist in your source.
- **Hack/fragility.** It relies on internal compiler data structures (a known portability/upgrade
  hazard; cf. non-deterministic bytecode issues).

The defect is **not** "generation" — `#[derive]` generates just as much. The defect is **opacity**:
no inspectable, reusable artifact stands between intent and running code.

### The extracted rule
> **Generation is GOOD when its output is an explicit, inspectable, reusable artifact in the same
> representation the system actually runs; it is BAD ("magic") when the artifact is hidden and only
> the running form exists.** Phrased for Mycelium: *every desugaring must produce a nameable L0 term
> a developer can view, diff, and (ideally) round-trip — never a side-effect on an invisible internal
> structure with no surfaced form.*

> **[EASIER] for Mycelium.** Because lowering targets a **real frozen L0 core**, the "explicit
> inspectable artifact" exists *by construction* — the generated thing is just an L0 value, not a
> separate compiler-internal AST mutation. The Lombok failure mode is structurally impossible if the
> only output channel is "produce an L0 term." Mycelium gets `#[derive]`-grade transparency for free
> and cannot accidentally become Lombok.

---

## Topic 3 — Value-semantic delegation/forwarding WITHOUT aliasing

### Hylo / Val — Mutable Value Semantics (Abrahams & Racordon)
Hylo (formerly Val) is built on **mutable value semantics (MVS)**: it **"bans sharing instead of
mutation,"** supporting "part-wise in-place mutation and local reasoning, while maintaining a simple
type system." The mechanism that forbids aliasing: **"references are second-class: they are only
created implicitly, at function boundaries, and cannot be stored in variables or object fields,"** so
**"variables can never share mutable state."** (`Declared`/`Proven`-flavored — Racordon et al.,
*Native Implementation of Mutable Value Semantics*, arXiv:2106.12678.)

Forwarding/composition/delegation without a shared object is expressed through **subscripts** and
**projections**:
- A **subscript** declares custom access (`let` / `inout` / `sink` / `set` variants) — different
  access modes for the same logical element.
- A **projection** "exposes an object yielded by a subscript call or property access." Its
  exclusivity discipline is what replaces reference identity: **"If a projection `p` projects an
  object `o` immutably, `o` is immutable for the duration of `p`'s lifetime. If `p` projects `o`
  mutably, `o` is inaccessible for the duration of `p`'s lifetime."** (`Declared`, Hylo spec.)

So "delegation" in Hylo = a method that *yields a projection of a held value*, with exclusivity
enforced for the projection's lifetime. There is **no shared mutable object and no reference
identity** — composition is by value, and apparent forwarding is a lifetime-scoped, exclusive view,
not a pointer into shared state.

### Swift — value types + protocol witness tables
Swift's `struct`/`enum` value types with protocol conformances give delegation-like polymorphism via
**witness tables** (a per-conformance dispatch table) rather than a shared base object. Forwarding to
a conformance is table-dispatched on a *copied/independent value*, not an alias. (Background context;
Swift is the lineage Abrahams brought into Val/Hylo.)

### Contrast — OOP prototype-chain delegation (Lieberman 1986)
Lieberman's *Using Prototypical Objects to Implement Shared Behavior* (OOPSLA 1986) is the classic
delegation model: an object **forwards unhandled messages to a designated prototype object**, and the
mechanism **"is dependent upon dynamic [late] binding"** so a call resolves to different code at
runtime, against an **actor**/message-passing substrate of *shared, live* objects. The prototype is a
**shared mutable object referenced by identity**, and `self`/late-binding semantics mean a method
found up the chain runs with the *original* receiver — a fundamentally **cyclic, reference-identity,
late-bound** relationship.

> **[CONSTRAINS → actually liberates] for Mycelium.** Mycelium is **immutable + acyclic +
> value-semantics**, so prototype-chain delegation is *forbidden by construction*: there is no shared
> mutable prototype, no reference identity to forward against, and the acyclicity rule bans the
> back-reference (`self` pointing up a live chain) that late-bound delegation requires. This is not a
> loss — it removes the single hardest thing to reason about (aliased, cyclic, late-bound shared
> state). The **Hylo MVS model is the correct prior art**: express delegation as *forwarding to a
> held value via projections/subscripts*, with exclusivity replacing reference identity. Lieberman's
> dynamism that Mycelium can't have (mutate-the-prototype-and-all-delegators-change) is exactly the
> dynamism the value model deliberately rejects. Recommend: model "delegation" as **static, by-value
> forwarding** (Kotlin-`by` / Go-embedding shape) lowered to L0 projection-style accessors — never a
> runtime chain walk.

---

## Topic 4 — Content-addressed generated artifacts: "same intent → same identity"

### Unison — definitions identified by hash
Unison is the sharpest prior art for Mycelium's premise. **"Each Unison definition is identified by a
hash of its syntax tree"** — a 512-bit hash over its structure *plus the hashes of all its
dependencies*. Decisive properties:
- **Names are metadata, not identity.** "Names are simply metadata for human consumption" pointing at
  immutable hash addresses; code depends on a definition *by hash*, so renaming breaks nothing.
- **Structural dedup is automatic.** "These hashes depend only on the structure of the code, not on
  the actual names used" — **two definitions with identical structure get the same hash** even under
  different names. Identical code is the *same object* in the codebase.
- **Cache-permanence.** Once a hash typechecks it "stays cached permanently"; no naming conflicts,
  free refactoring. (`Declared`/`Empirical`, Unison docs.)

### Nix / Bazel — content-addressed build artifacts
Build systems show the same identity-by-content at the artifact level:
- **Nix content-addressed derivations** (experimental since 2.4): store paths "determined by their
  contents." This enables **early cutoff** — an output-invariant source change (e.g. adding a Haskell
  comment) rebuilds the immediate component, but because the *output hash is unchanged*, downstream
  rebuilds are skipped. The build resolves a derivation against its inputs' **content-addressed
  paths**. (`Declared`/`Empirical`, Tweag / NixOS.)
- **Bazel + Nix.** Nix store paths are content-addressed, so any change yields a new path that
  invalidates Bazel's cache exactly when (and only when) content changed.

### The pattern for Mycelium
> **Same intent → same content hash → same identity, with no mutable registry.** Identical generated
> lowerings deduplicate *by hash*: there is no central name table to update, no singleton-registry to
> mutate. The "value-semantic singleton-in-a-set" falls out — a generated artifact for given
> parameters *is* its hash; producing it again yields the identical hash and therefore the identical
> object. "Early cutoff" gives the compile-time dual: if a desugaring's output hash is unchanged,
> everything downstream is reusable untouched.

> **[EASIER] — this is Mycelium's strongest free win.** Content-addressing **gives dedup and the
> singleton for free** and **eliminates the mutable registry** that an OOP/identity model would need
> to coordinate "one instance per params." Because the language is *already* immutable + acyclic +
> content-addressed, generated L0 terms are content-addressed like everything else: a generative
> lowering that fires twice with the same intent produces one hash-identical term, automatically
> shared. No de-dup pass, no interning table, no singleton lifecycle to manage — the property Unison
> bolts on is Mycelium's substrate. (The acyclicity premise also keeps the hash well-founded: a DAG
> of dependency hashes terminates; no cyclic hashing problem.)

---

## Topic 5 — Semantics-preserving lowering verification (desugar/macro angle)

(Brief — the nanopass/CompCert backbone is already covered; here only the *desugaring/macro*-specific
verification angle.)

- **Translation validation** (Pnueli/Siegel/Singerman; Necula, *Translation Validation for an
  Optimizing Compiler*, PLDI 2000). Rather than prove the transformer correct once, **validate each
  run**: compare the program's intermediate form *before and after each pass* and check semantics are
  preserved, generating a checkable witness. Necula reports the infrastructure costs "about the effort
  …to implement one compiler pass." (`Proven`-per-run, i.e. each validated lowering carries a checked
  certificate.) **This is the right shape for per-pass desugaring:** validate that *this* desugaring
  of *this* program preserved meaning, emitting a witness — a natural fit for Mycelium's per-op
  provenance and `certified` mode.
- **Differential testing of desugaring.** The Csmith lineage (randomized differential testing) and
  metamorphic testing are "the gold standard" for compiler validation: **semantically equivalent
  inputs — or the same input across implementations — must yield identical observable results.**
  Applied to desugaring: the *sugared* program and its *desugared L0* form are two
  implementations-of-the-same-meaning and must observably agree on a generated corpus. (`Empirical`.)
- **Property-based testing of macro/desugar expansions.** Generate surface programs, expand, and
  assert invariants on the expansion: it type-checks, **round-trips** (delaborate∘lower = id, per
  Lean's injective-`pp` discipline), and is **observationally equal** to the source on random inputs.
  Hygiene is itself a testable property (no free-variable capture across expansion).

> **Mycelium recommendation.** Treat each desugaring pass as **translation-validated per run**
> (witness emitted, gating `certified` mode) and **differentially + property-tested** in CI (LOW
> proptest cases per commit, HIGH on release — DN-20): for every surface program, `observe(surface)
> == observe(lower(surface))`, plus a hygiene-no-capture property and a delaborate-round-trip
> property. This makes "each pass preserves observable meaning" a *checked* claim (lattice: `Proven`
> when the validator certifies the run; `Empirical` from the differential corpus) rather than a
> `Declared` assertion — exactly the VR-5 discipline. **[EASIER]** immutability + value semantics make
> "observable meaning" clean to define (no aliasing/effect interleaving to quantify over), so the
> differential oracle is just *value equality of results*.

---

## Annotated bibliography (URLs)

**Topic 1 — inspectable expansion**
- dtolnay, *cargo-expand* (README; "lossy… debugging aid only" disclaimer; covers derive + proc-macro).
  https://github.com/dtolnay/cargo-expand
- Racket, *Macro Debugger: Inspecting Macro Expansion* (stepper, rewriting steps, mark-colors,
  macro hiding, binding info). https://docs.racket-lang.org/macro-debugger/index.html
- Culpepper & Felleisen, *A Stepper for Scheme Macros* (Scheme 2006) — design of the stepper.
  https://www2.ccs.neu.edu/racket/pubs/scheme2006-cf.pdf
- Lean 4 metaprogramming book, *Options* (`set_option pp.all`, delaborator injective/round-trip).
  https://leanprover-community.github.io/lean4-metaprogramming-book/extra/01_options.html
- Lean reference, *Extending Lean's Output* (delaboration/unexpansion).
  https://lean-lang.org/doc/reference/latest/Notations-and-Macros/Extending-Lean___s-Output/
- Ullrich & de Moura, *Hygienic Macro Expansion for Theorem Proving Languages* (arXiv:2001.10490) —
  name-capture as correctness bug; Scheme-derived hygiene. https://arxiv.org/abs/2001.10490
- GHC users guide, *Template Haskell* (`-ddump-splices`, `======>` splice output).
  https://downloads.haskell.org/ghc/latest/docs/users_guide/exts/template_haskell.html
- Scala 3 docs, *Inline* / *Macros* / *Reflection* (`-Xprint`, inline→quote→reflection ladder).
  https://docs.scala-lang.org/scala3/guides/macros/inline.html ·
  https://docs.scala-lang.org/scala3/guides/macros/macros.html

**Topic 2 — generative lowering (GOOD vs OPAQUE)**
- Rust derive — recoverable via cargo-expand (above).
- Kotlin docs, *Delegation* (`by`: "compiler generates all the methods… that forward to b").
  https://kotlinlang.org/docs/delegation.html ·
  *Delegated properties* (`prop$delegate`). https://kotlinlang.org/docs/delegated-properties.html
- Go struct embedding (field/method promotion; composition over inheritance).
  https://eli.thegreenplace.net/2020/embedding-in-go-part-3-interfaces-in-structs/
- Zig comptime (Loris Cro, *What is Zig's Comptime?* — comptime as the no-macro answer to C macros).
  https://kristoff.it/blog/what-is-zig-comptime/
- **Lombok anti-pattern** — hidden AST/bytecode generation, no source artifact, undebuggable:
  https://dev.to/yanev/why-i-believe-lombok-should-be-discarded-from-java-projects-1g4h ·
  https://www.danvega.dev/blog/no-lombok ·
  https://news.ycombinator.com/item?id=19048335

**Topic 3 — value-semantic forwarding without aliasing**
- Racordon, Shabalin, Zheng, Abrahams, Saeta, *Native Implementation of Mutable Value Semantics*
  (arXiv:2106.12678) — "bans sharing instead of mutation"; second-class references.
  https://arxiv.org/abs/2106.12678
- Hylo, *Language Specification* — subscripts (`let`/`inout`/`sink`/`set`), projections, lifetime
  exclusivity. https://hylo-lang.org/docs/reference/specification/ · https://hylo-lang.org/
- Lieberman, *Using Prototypical Objects to Implement Shared Behavior in OO Systems* (OOPSLA 1986) —
  prototype-chain delegation, late binding, shared/cyclic objects (the contrast case).
  https://www.semanticscholar.org/paper/91b4af9ff2c0f9d7985544d901f6ab2ef01fe271 ·
  https://en.wikipedia.org/wiki/Delegation_(object-oriented_programming)

**Topic 4 — content-addressed generated artifacts**
- Unison, *The big idea* (content-addressed code; hash = AST+deps; names are metadata; dedup).
  https://www.unison-lang.org/docs/the-big-idea/ · https://github.com/unisonweb/unison
- SoftwareMill, *Trying out Unison: code as hashes* (structural hash → dedup by content).
  https://softwaremill.com/trying-out-unison-part-1-code-as-hashes/
- Tweag, *Implementing a content-addressed Nix* (CA derivations; early cutoff).
  https://www.tweag.io/blog/2021-12-02-nix-cas-4/ · https://nixos.wiki/wiki/Ca-derivations
- Bazel/Nix CA interplay (content-addressed store paths invalidate cache iff content changes).
  https://blog.consumingchaos.com/posts/nix-bazel-cross-compiling/

**Topic 5 — semantics-preserving lowering verification (desugar/macro angle)**
- Necula, *Translation Validation for an Optimizing Compiler* (PLDI 2000) — per-run validation,
  before/after each pass, checkable witness. https://people.eecs.berkeley.edu/~necula/Papers/tv_pldi00.pdf
- Differential/metamorphic testing as gold standard (Csmith lineage), summary survey context.
  https://arxiv.org/pdf/2504.04321
- Lean injective `pp.all` round-trip (above) — round-trip as a testable expansion property.

---

## Recommended design rules (10 lines)

1. **Make the expansion viewable, always** — ship an expansion *stepper*, not just a text dump
   (Racket-grade), because a lossy text view silently lies (cargo-expand's own caveat).
2. **View the real artifact** — show the actual frozen **L0 term** the kernel runs, not a re-rendered
   text approximation; "see the expansion" must be the ground truth, never a guess.
3. **Layer-hideable inspection** — let developers stop lowering at any layer (L2→L0), mirroring
   Racket "macro hiding," so the L0–L3 tower is navigable pass-by-pass.
4. **Generation is legitimate only with an explicit, inspectable, reusable artifact** (`#[derive]`,
   Kotlin `by`, Go embedding, Zig comptime) — **never Lombok-style** AST/bytecode magic with no
   surfaced form. The only output channel is "produce a nameable L0 term."
5. **Round-trip discipline** — desugaring should be reversible enough that delaborate∘lower = id for
   `certified` mode (Lean's injective-`pp`), making the inspector itself a transparency check.
6. **Delegation = static by-value forwarding** (Hylo projections/subscripts; Kotlin-`by` shape) —
   exclusivity replaces reference identity; **forbid** prototype-chain late-bound delegation (the
   acyclic, immutable, value model bans it by construction — a feature, not a gap).
7. **Same intent → same identity, free** — content-address generated L0 terms; identical lowerings
   dedup by hash with **no mutable registry/singleton lifecycle** (Unison). This is Mycelium's biggest
   structural win — the singleton-in-a-set is the substrate, not a feature to build.
8. **Early cutoff** — if a desugaring's output hash is unchanged, reuse everything downstream
   untouched (Nix CA derivations).
9. **Verify lowering per run + differentially** — translation-validate each pass (emit a witness,
   gate `certified`) and CI-test `observe(surface) == observe(lower(surface))` + hygiene-no-capture +
   round-trip properties (tiered LOW/HIGH per DN-20).
10. **Net premise effect:** value-semantics + acyclic + content-addressed makes 4/5 of these *easier*
    (free dedup/singleton, no prototype-chain to support, clean observable-equality oracle); the only
    "constraint" — no late-bound mutable delegation — is precisely the complexity the model is right
    to reject.
