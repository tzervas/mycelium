# B — External Research: Rust-AST → Mycelium Source-to-Source Transpiler

> **Research question.** What is the most effective architecture for a Rust-AST → (new language)
> transpiler that converts the *bulk* of idiomatic Rust while *honestly flagging* the untranslatable
> residue — given Mycelium is an immutable, **acyclic**, content-addressed, **affine-primary**
> (borrow/move/unique) value-semantics language, and the never-silent house rule forbids
> plausible-but-wrong emission?

**Method / transparency note.** External web research, adversarially verified against papers, specs,
and source files (not READMEs alone). Guarantee tags below follow Mycelium's lattice
(`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`). No claim here is `Proven`; tool *capabilities* checked
against primary docs are `Empirical`, second-hand figures are `Declared` and flagged. Two corrections
to the brief are recorded inline: **"Sherpa" (a C→Rust tool) does not exist** — do not cite it; the
real corrode-sibling is **Citrus**. PDFs for the OOPSLA papers could not be fetched/parsed in-env
(ACM 403; no `pdftotext`/working `pypdf`), so a few quantitative figures are second-hand from
arXiv:2501.14257 and are flagged `Declared`.

---

## 1. Rust front-end tooling — recommendation + semantic-level trade-offs

The single most consequential design choice. The transpiler is **ownership-aware** by mandate (it
must map move/`&`/`&mut`/`Drop` onto Mycelium's affine disciplines), so the front-end must surface
*ownership and borrow facts*. Those facts live at different semantic levels in different tools.

### 1.1 The four candidate front-ends and what each recovers

| Front-end | Semantic level | Types | Trait/method resolution | Name resolution | **Ownership / borrow / lifetime facts** | Stability | Effort |
|---|---|---|---|---|---|---|---|
| **`syn` + `proc-macro2`** | **Syntax only** (tokens → AST) | ❌ none | ❌ none | ❌ none | ❌ none (only *syntactic* `&`/`&mut`/`mut` as written) | ✅ stable, semver | Low to start, but you re-implement a type/name resolver |
| **`rust-analyzer` as a library (`ra_ap_*`)** | **HIR** + type inference (the `hir` façade) | ✅ resolved expr/pat types | ✅ (next-gen trait solver, method res) | ✅ | ⚠️ **mutability inference + autoref/autoderef adjustments, but NO borrow check, NO lifetime/region solving** | ❌ **no stability guarantee** (explicitly "never an API boundary" below `hir`) | Medium-high |
| **`rustc_*` internal crates + MIR (custom `rustc_driver`)** | **MIR** + `mir_borrowck` | ✅ full `Ty` | ✅ full | ✅ full | ✅ **authoritative** — moves, region constraints, the borrow check itself, drop elaboration | ❌ nightly-only, `rustc_private`, churns every toolchain | High |
| **Hybrid: `syn` for surface + rustc/RA for facts** | Mixed | via the analyzer | via the analyzer | via the analyzer | via rustc MIR | Mixed | Highest, but best fidelity |

### 1.2 Verified findings (the load-bearing facts)

- **`syn` is purely syntactic.** It is "a parsing library for parsing a stream of Rust tokens into a
  syntax tree" and operates on `proc-macro2` tokens with **no knowledge of types, name resolution,
  traits, or lifetimes** — semantic concerns are the caller's problem
  ([syn docs](https://docs.rs/syn/latest/syn/); [crates.io/crates/syn](https://crates.io/crates/syn)).
  It *can* parse a whole `syn::File`, and it runs **outside** a proc-macro context (build.rs, a CLI
  tool) because it sits on `proc-macro2` — good for a standalone transpiler binary. But it sees
  `&mut x` only as the *token* `&mut x`; it cannot tell you whether a binding is *actually* mutated,
  whether a call moves or borrows, or what a generic resolves to. Building ownership-aware translation
  on `syn` alone means **re-implementing name resolution + type inference + a move/borrow analysis** —
  i.e. rebuilding the front half of rustc. Not viable as the *sole* source of ownership facts.

- **`rust-analyzer`'s `hir` is the stable-ish façade and gives a lot — but not borrow facts.** "If you
  think about *using rust-analyzer as a library*, the `hir` crate is most likely the façade you'll be
  talking to" ([RA architecture](https://rust-analyzer.github.io/book/contributing/architecture.html)).
  Via the `Semantics` API it exposes **resolved expression/pattern types, trait + method resolution
  (next-gen solver), name resolution, mutability inference, and autoref/autoderef adjustments**
  (`expr_adjustments`) ([Type System and Inference](https://deepwiki.com/rust-lang/rust-analyzer/5.3-type-system-and-inference)).
  **Crucially it does *not* run the borrow checker and does not solve lifetimes/regions** — it
  represents `Region` and does lifetime *elision* but "deliberately avoids the full borrowck analysis
  that rustc performs"; its own guidance is that RA borrow diagnostics "may lag behind the actual
  compiler; always verify with `cargo check`"
  ([DeepWiki](https://deepwiki.com/rust-lang/rust-analyzer/5.3-type-system-and-inference); RA docs).
  The `ra_ap_*` crates are published (`ra_ap_hir`, `ra_ap_hir_ty`, `ra_ap_syntax`, …
  [docs.rs/ra_ap_hir](https://docs.rs/ra_ap_hir/)), but RA "doesn't introduce new stability
  guarantees" and the sub-`hir` crates "are not, and will never be, an api boundary" — versions move
  fast and break.

- **Only `rustc`'s MIR borrow-check is authoritative for ownership.** The borrow checker runs on
  **MIR**, in `rustc_borrowck` via the `mir_borrowck` query; it computes moves/initialization
  dataflow and the region/lifetime constraints, post-NLL as *sets of CFG points*
  ([rustc-dev-guide borrow-check](https://rustc-dev-guide.rust-lang.org/borrow-check.html);
  [MIR](https://rustc-dev-guide.rust-lang.org/mir/index.html)). Reaching it requires a **custom
  `rustc_driver`** with `rustc_private`, `rustc-dev` component, and `Config::override_queries` to grab
  `optimized_mir`/borrowck results — **nightly-only and unstable**, re-pinned every toolchain bump
  ([building your own rustc_driver](https://jyn.dev/rustc-driver/);
  [MIR instrumentation](https://emavan.com/blog/2025/mir-instrumentation/)). This is how every serious
  Rust analysis tool (MIRI, Prusti, Flowistry, Aeneas) gets ownership truth.

### 1.3 Recommendation — **hybrid, syntax-anchored, fact-enriched; MIR-grade ownership via a rustc driver**

**Primary architecture: drive translation from `rust-analyzer`'s HIR/`Semantics` for the bulk of
typed lowering, anchored to `syn`/`ra_ap_syntax` surface spans, and obtain *authoritative ownership
facts* (move vs borrow, exclusivity, drop points, region/lifetime constraints) from a `rustc` MIR
borrow-check pass.** Rationale, grounded:

1. **You cannot do ownership-aware translation from `syn` alone** (1.2) — it has no types, names, or
   borrow facts. Reject `syn`-only.
2. **rust-analyzer gets you ~80% of the typed front-end cheaply** (types, traits, method res,
   mutability, adjustments) and is *designed* to be embedded as a library — but it is the **wrong
   oracle for the borrow/lifetime facts** that the Mycelium affine mapping (§4) hinges on, because it
   doesn't borrow-check. Use it for typed lowering, **not** for ownership truth.
3. **rustc MIR borrowck is the only authoritative ownership oracle.** Because Mycelium's never-silent
   rule means *guessing* whether a `&mut` aliases is a defect, the move/borrow/exclusivity decision
   must come from the checker, not a heuristic. Run a `rustc_driver` pass over the same crate, harvest
   per-place move/borrow/region facts (à la **Flowistry**/**Polonius** consumers), and **join them
   onto the HIR/`syn` AST by span/`HirId`**.
4. **Trust posture:** because the front-end is the trusted base for a *self-hosting bootstrap* (§5),
   prefer rustc's own machinery for the safety-critical facts (it is the reference semantics) and treat
   any RA-only inference as `Empirical`, to be confirmed against rustc before it drives an
   irreversible translation decision.

**Pragmatic phasing.** v0: `syn`/`ra_ap_syntax` AST + RA `Semantics` for types → emit Mycelium for the
*typed, ownership-trivial* subset, and **FLAG everything touching `&mut`, raw pointers, `Rc/RefCell`,
or generics-with-trait-objects** as residue (§3) rather than guess. v1: add the rustc MIR-borrowck
side-channel to *clear* flags it can prove (move-vs-copy, unique `&mut`, non-escaping borrow,
drop points), shrinking the residue. This mirrors c2rust's *preserve-first, refine-later* split (§2)
and Laertes' "use the borrow checker as the oracle" idea (§2.1) — but turned around: we use the
checker to *certify translatability*, not to retro-fit safety.

**Effort/stability caveat (transparency).** The rustc-driver dependency pins the transpiler to a
*nightly* toolchain and will break on bumps — this is a `Declared` maintenance cost, not free. Budget a
toolchain-pin ADR (cf. CLAUDE.md "don't silently bump pins"). RA's instability is milder but real;
isolate both behind a `FrontendFacts` trait so the AST-walk core doesn't churn with them.

---

## 2. Transpiler-architecture prior art + lessons

### 2.1 c2rust (C→Rust) — the most directly relevant prior art

**Philosophy is explicit and load-bearing: "preserve functionality first, idiomatic later."** README,
verbatim: *"The primary goal of the translator is to preserve functionality; test suites should
continue to pass after translation"* ([github.com/immunant/c2rust](https://github.com/immunant/c2rust)).
Two-stage architecture, and the split matters:
- **`c2rust transpile`** — mechanical, near-line-by-line C → **unsafe** Rust; uses the **Relooper**
  algorithm (from Emscripten) to render arbitrary C control flow (incl. `goto`) as structured Rust.
- **`c2rust refactor`** — a *separate*, nightly-pinned tool that lifts unsafe toward safe idioms
  *post hoc* ([c2rust manual](https://c2rust.com/manual/)).

**(a) Partial translation = skip-and-warn, never silent.** README: *"The translator will emit a
warning and attempt to skip function definitions that cannot be translated."* It produces partial
output over a big codebase and leaves a flagged hole, rather than aborting or fabricating —
directly the pattern Mycelium wants.

**(b) Residue surfacing.** Because c2rust's *entire normal output is `unsafe`*, "unsafe" is **not** a
residue signal there (a lesson: don't overload a normal-case marker as your flag). Hard residue is a
**`docs/known-limitations.md`** list — `setjmp`/`longjmp` ("unlikely to ever support"), jumps in/out
of statement-expressions, `_Complex`, non-x86 SIMD, some builtins, `long double` variadics
([known-limitations.md](https://github.com/immunant/c2rust/blob/master/docs/known-limitations.md)).

**(c) Equivalence verification — cross-checking / ReMon MVEE (the most transferable mechanism).**
c2rust ships a differential harness ([c2rust.com/manual/cross-checks](https://c2rust.com/manual/cross-checks/);
[Cross-checks design wiki](https://github.com/immunant/c2rust/wiki/Cross-checks-design);
[tutorial](https://github.com/immunant/c2rust/blob/master/docs/cross-check-tutorial.md)):
- **Instrumentation points**: function **entry** / **exit** / **return-value hash** (args optional,
  off by default), via a **clang plugin** (C side) and a **`rustc` plugin + derive macro** (Rust side),
  configurable at file/function/arg granularity.
- **Offline**: each variant logs a trace (`libfakechecks` text / `zstd` compressed); user `diff`s logs.
- **Online**: the **ReMon MVEE** (Multi-Variant Execution Environment, an experimental fork) runs the C
  and Rust variants *in lockstep on replicated input*, cancels nondeterminism, and reports divergence
  automatically. Values are **hashed to a fixed 64-bit width**, excluding padding bytes and
  dereferencing pointers (compare pointed-to *value*, not address)
  ([cross-check-hash](https://c2rust.com/manual/docs/cross-check-hash.html)).
- **Lesson:** equivalence is established by **differential execution of value-hashes at function
  boundaries**, not proof → earns at most an **`Empirical`** tag in Mycelium's lattice. Never `Proven`.

**The "safe-by-translation" ceiling is fundamental (Laertes / Aliasing-Limits).** "Translating C to
Safer Rust" (Emre, Schroeder, Dewey, Hardekopf, **OOPSLA 2021**; tool **Laertes**) lifts raw pointers
to safe references by **ownership+lifetime inference using the Rust borrow checker's own errors as an
oracle** ([paper](https://hardekbc.github.io/files/emre21translating.pdf);
[DOI 10.1145/3485498](https://dl.acm.org/doi/10.1145/3485498)). **Verified ceiling: Laertes applies to
only ~11% of pointers** (per the SPLASH 2023 abstract;
[splashcon details](https://2023.splashcon.org/details/splash-2023-oopsla/20/Aliasing-Limits-on-Translating-C-to-Safe-Rust))
— `Declared`: a second-hand companion figure puts raw-pointer conversion at ~8.3%, ~21% in a follow-up,
covering only the ~20% of functions whose *sole* unsafety is missing ownership info, excluding pointer
arithmetic / unsafe casts / FFI boundaries (arXiv:2501.14257 — flag as second-hand). The follow-up
**"Aliasing Limits on Translating C to Safe Rust" (OOPSLA 2023,
[DOI 10.1145/3586046](https://dl.acm.org/doi/abs/10.1145/3586046))** argues the residue is
**fundamental, not an engineering gap**: C aliasing patterns that violate Rust's no-mutable-aliasing
invariant *cannot* be expressed in safe Rust. **Direct lesson for Mycelium:** a transpiler that
*promises* "safe/affine by translation" is making a claim it largely cannot keep; honesty (VR-5)
requires marking the unliftable residue — which is exactly what §3 designs.

### 2.2 corrode (Haskell C→Rust) — lineage + why c2rust superseded it; and "Sherpa"

corrode prioritized **behavior + ABI compatibility** (emitting `extern "C"`, link-compatible `.o`),
not idiom; it pioneered Relooper-style control-flow recovery; written in Haskell only because the
`language-c` parser existed there ([github.com/jameysharp/corrode](https://github.com/jameysharp/corrode)).
The author's own retrospective: *"c2rust looks exactly like what I planned for a from-scratch rewrite
of Corrode"* — he deprecated corrode as **unsalvageable due to early design mistakes** and pointed
users to c2rust ([c2rust-vs-corrode](https://jamey.thesharps.us/2018/06/30/c2rust-vs-corrode/)). corrode's
planned idiom-lifting companion **`idiomatic`** was split out and *never built* — the same
mechanical-then-idiomatic split c2rust later realized as `transpile`+`refactor`. **Adversarial
correction:** the brief's **"Sherpa" C→Rust tool does not exist**; the real lightweight sibling is
**Citrus**, which *"focuses on being lightweight rather than fully correct: output may not compile
without manual revision"* — a deliberately partial "rough starting point." Do not cite Sherpa (CLAUDE.md
rule 4: no ungrounded "facts").

### 2.3 JSweet / TeaVM (Java→JS / Java→wasm) — compile-time residue, not runtime surprise

- **JSweet** surfaces unsupported constructs as **transpile-time *sound errors*** ("JSweet will report
  sound errors … so that programmers can adjust") and offers an explicit **`@Erased`** escape hatch
  that *deletes* a member from output **with a documented warning that it "can lead to program
  inconsistencies"** — i.e. a *marked, acknowledged* hole, not a silent guess
  ([JSweet spec](https://raw.githubusercontent.com/cincheo/jsweet/master/doc/jsweet-language-specifications.md);
  [FAQ](https://www.jsweet.org/faq/)). JDK coverage is bounded by "candies"; outside them → transpile
  error.
- **TeaVM** is **AOT/whole-program**, so it turns many JVM-runtime errors into **build-time, *located*
  diagnostics** via `org.teavm.diagnostics.Diagnostics`, reporting an unsupported reference *with a
  reference-chain* showing where it originated ([troubleshooting](https://teavm.org/docs/intro/troubleshooting.html);
  [discussion #963](https://github.com/konsoletyper/teavm/discussions/963)). **Lesson:** whole-program
  reachability converts "unsupported construct" into a precise, *positioned* never-silent diagnostic —
  the ideal — at the cost of needing the whole program (fine for a self-contained kernel/stdlib).

### 2.4 Haxe (multi-target) — honest checked/unchecked boundary (and an anti-pattern)

Haxe's value is the **explicit boundary between verified and unverified code**: `untyped` and target
injectors `__js__`/`__cpp__` emit raw target code that *"is always untyped and can not be validated …
error-prone"* — the unsafety is surfaced *by the syntax itself*
([target-syntax](https://haxe.org/manual/target-syntax.html);
[untyped](https://haxe.org/manual/type-system-untyped.html)). Its macro API
(`haxe.macro.Context.error/warning`) raises **positioned** compile-time diagnostics — a precedent for
*transpiler-authored, located* residue messages ([macro manual](https://haxe.org/manual/macro.html)).
**Anti-pattern to avoid:** Haxe's *silent* per-target file substitution — when a `Module.js.hx` exists,
"the main file is **not loaded at all** … errors are not checked in the main file"
([lf-target-specific-files](https://haxe.org/manual/lf-target-specific-files.html)) — a textbook
never-silent violation. Don't do that.

### 2.5 AST-walk transpilers generally — py2many = the cleanest direct precedent

**py2many (Python → Rust/Go/C++/Julia/Kotlin/Nim/Dart/V/D)** — and notably the **seed architecture
mirrors the brief's "Python→Rust AST-walk transpiler with a CompatibilityAnalyzer"**
([github.com/py2many/py2many](https://github.com/py2many/py2many)):
- **Visitor/transformer architecture** on Python's `ast`: a chain of `ast.NodeTransformer` rewriter
  passes normalizes the tree, then per-target code-generator visitors emit source
  ([ast docs](https://docs.python.org/3/library/ast.html)).
- **Residue = a typed exception taxonomy carrying source location** — `py2many/exceptions.py` defines
  `AstErrorBase(lineno, col_offset, msg)` with subclasses **`AstNotImplementedError`** (unsupported
  node), **`AstTypeNotSupported`**, **`AstCouldNotInfer`** (inference failed),
  **`AstUnrecognisedBinOp`**, **`AstClassUsedBeforeDeclaration`**, **`AstIncompatibleAssign`**. Each is
  raised *by passing the offending AST node*, auto-extracting line/column → every untranslatable
  construct fails **loudly with a precise position, never silently**
  ([exceptions.py](https://github.com/py2many/py2many/blob/main/py2many/exceptions.py)). This is
  essentially Mycelium's never-silent / `Result` discipline implemented at transpile time, and the
  obvious thing to *generalize* into the structured report of §3.
- **Type annotations effectively required**; un-inferrable types raise `AstCouldNotInfer` rather than
  guess. **Verification = golden-file + behavioral tests** (`tests/expected/` per target + compile/run)
  → an `Empirical` guarantee.

### 2.6 Cross-cutting lessons (the transferable design rules)

1. **Two-phase split** (mechanical-faithful → idiomatic-lift) is the dominant successful architecture
   (c2rust `transpile`/`refactor`; corrode + planned `idiomatic`; SACTOR's unidiomatic→idiomatic).
   Keep semantics-preservation auditable and *separate* from beautification.
2. **Visitor over the AST with a typed "unsupported" error that carries a source span** is the standard
   residue mechanism (py2many). "Leave a hole and flag it," done right.
3. **Never silent = fail at transpile time with a *located* diagnostic** (py2many exceptions, TeaVM
   Diagnostics, JSweet sound errors, Haxe `Context.error`). Avoid Haxe's silent file substitution; avoid
   overloading a normal-case marker (c2rust's blanket `unsafe`) as the residue flag.
4. **Equivalence is verified *empirically*, essentially never proven** — differential value-hash
   execution (c2rust/ReMon), FFI co-execution (SACTOR), golden-file + behavioral suites (py2many). For
   Mycelium's lattice: a transpile's equivalence claim earns at most **`Empirical`**; `Proven` needs a
   checked equivalence theorem none of these tools provide.
5. **The "safe/affine by translation" promise has a hard, fundamental ceiling** (Laertes ~11% of
   pointers; Aliasing-Limits proves the rest irreducible). Honest residue-marking beats over-claiming.

---

## 3. Never-silent residue-reporting design

The transpiler's defining feature. Survey of how mature tools report "cannot translate X here," distilled
into a concrete, auditable design that satisfies the house rules (G2 never-silent, VR-5 don't-upgrade,
EXPLAIN-able).

### 3.1 What the prior art converged on

- **py2many**: typed exception per failure class, constructed *from the AST node*, carrying
  `lineno`/`col_offset` ([exceptions.py](https://github.com/py2many/py2many/blob/main/py2many/exceptions.py)).
- **rustc's own diagnostics** are the gold standard for *machine-actionable* reporting and give Mycelium
  a ready-made model. rustc emits **structured JSON** (`--error-format=json`) one-object-per-line, each
  with an **error code**, message, an array of **spans** (file/line/col byte ranges), child
  sub-diagnostics, and **structured suggestions** ([rustc JSON](https://doc.rust-lang.org/rustc/json.html);
  [rustc_errors](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_errors/index.html);
  [diagnostic-structs](https://rustc-dev-guide.rust-lang.org/diagnostics/diagnostic-structs.html)).
  Most importantly it tags each suggestion with an **`Applicability` lattice** — **`MachineApplicable`**
  (auto-apply), **`MaybeIncorrect`** (offered, may be wrong), **`HasPlaceholders`** (has `{}` holes, not
  auto-applicable), **`Unspecified`** (unknown)
  ([rustfix diagnostics](https://doc.rust-lang.org/nightly/nightly-rustc/rustfix/diagnostics/index.html)).
  **This lattice is a direct analogue of Mycelium's guarantee lattice** and the right shape for a
  *transpile-confidence* tag.
- **c2rust** = skip-and-warn at function granularity + a curated `known-limitations` doc.
- **TeaVM** = located diagnostic *plus a reference-chain* (provenance of *why* this construct was
  reached). **Haxe** = positioned macro diagnostics.

### 3.2 Recommended residue report (structured, auditable, EXPLAIN-able)

A machine-readable **`TranslationResidue`** record per untranslatable site, emitted as JSON-lines
(rustc-style) *and* a human table, with these fields (synthesizing py2many + rustc + TeaVM):

| Field | Why (grounded) |
|---|---|
| `span` (file, byte range, line:col) | py2many/rustc: a flag without a location is not actionable. |
| `granularity` (item / fn / stmt / **expr** / pattern / type) | **Per-expr/per-place** is required — Laertes/Aliasing-Limits show untranslatability is often a *single aliasing site* inside an otherwise-fine function, not the whole function. Per-function (c2rust) is too coarse for an affine mapping. |
| `category` (enum) | A *closed taxonomy* of residue classes (§3.3), like py2many's exception subclasses → enables triage, counting, and per-class burn-down. |
| `rust_construct` (the source snippet + resolved type) | The "what". Include the rustc-resolved `Ty` so the manual-refiner sees the *semantic* shape, not just syntax. |
| `reason` (why it can't translate) + `mycelium_constraint` (which invariant blocks it) | EXPLAIN: name the violated invariant (e.g. "acyclicity", "no shared mutation"), tying residue to a *house-rule basis*, not a vibe. |
| `confidence` (`Blocked` / `MaybeTranslatable` / `NeedsHumanChoice`) | rustc `Applicability` analogue. `Blocked` = provably untranslatable (cite the theorem/invariant); `MaybeTranslatable` = the front-end lacked a fact (e.g. RA couldn't borrow-check — rustc pass might clear it); `NeedsHumanChoice` = multiple valid Mycelium encodings, pick one. **Never auto-emit for the latter two** (VR-5: don't upgrade past basis). |
| `provenance` / reference-chain (TeaVM-style) | If the residue is *induced* by a callee/trait, show the chain — the refiner fixes the root, not the symptom. |
| `suggested_manual_action` (optional, `HasPlaceholders`-tagged) | Offer a hole-with-placeholder rewrite *marked non-applicable*, never a silent guess. |
| `guarantee_tag` of any partial output emitted | If the transpiler emits *some* code for the site, tag it on the lattice; a hole emits nothing + a `// MYCELIUM-RESIDUE[id]` marker. |

**Behavioral rules (the never-silent core):**
1. **A residue site emits a hole + a record, never plausible code.** The hole is a syntactically
   visible marker (`__residue!(id)` macro / typed `todo`) so downstream compilation *fails loudly* if
   the hole is left unfilled — mirroring py2many raising rather than emitting, and the opposite of
   Haxe's silent substitution.
2. **The CompatibilityAnalyzer runs *ahead* of emission** (it's the brief's seed concept generalized):
   a pre-pass walks the typed AST + ownership facts and produces the *full residue manifest first*, so
   the maintainer gets a **complete, countable** "here is everything that needs manual work" report
   before any output — enabling burn-down tracking and a Definition-of-Done ("residue count → 0 for the
   bootstrap subset").
3. **Confidence is a lattice tag, never silently upgraded.** A `MaybeTranslatable` only becomes
   `Translated` when a *checked* fact (rustc MIR borrowck) discharges it — exactly VR-5.
4. **The taxonomy is closed and append-only**; a newly-discovered residue class is *added*, never
   folded silently into "misc" — keeps the report honest as coverage grows.

### 3.3 Residue taxonomy (closed enum) — the categories that will dominate (see §4 for why)

`SharedMutableAliasing` · `CyclicData` · `InteriorMutability` · `RawPointer/Unsafe` ·
`Unbounded/Non-affineAliasing` · `TraitObjectDynamicDispatch` (if Mycelium lacks it) ·
`InterproceduralBorrowEscape` · `FFI/ExternBoundary` · `Macro/CodeGenResidue` (lost structure) ·
`UnsupportedStdlibIntrinsic` · `LifetimeEscapesScope` · `NeedsHumanEncodingChoice`.

---

## 4. Rust-ownership → affine / uniqueness mapping survey

What maps cleanly to an affine, **acyclic**, value-semantics target, and what is irreducible residue.
The cross-system conclusion is sharp and unanimous: **Rust's affine *move* core maps cleanly; the
shared-mutable + cyclic residue (`Rc<RefCell>`, back-edges, interior mutability) does not — and is
precisely what an acyclic value-semantics target structurally forbids.**

### 4.1 Rust's precise substructural classification (the foundation)

- **Rust is *affine*, not linear, and affinity is *separate from* borrow checking.** In the
  substructural lattice, *affine* = weakening (drop) allowed, contraction (reuse) forbidden →
  *at-most-once*. Rust's default types have weakening, lack contraction → **affine**: *"most types in
  Rust are … affine types … they can only be used one or zero times"*
  ([without.boats/ownership](https://without.boats/blog/ownership/)). The "zero uses" case is legal
  because `Drop` runs the destructor — *"to use them is to move them; not to use them is to drop
  them."* Borretti concurs and adds the key separation: safety *"comes from their types being affine,"*
  and borrowing *"suspend[s] linear/affine type rules for some delimited time"*
  ([borretti.me/type-systems-memory-safety](https://borretti.me/article/type-systems-memory-safety)).
- **The Law of Exclusivity**: at any time a value is owned, *or* shared via any number of `&T`, *or*
  exclusively via exactly one `&mut T` — never mixed (ibid).
- **Lifetimes (NLL) are an *analysis*, not runtime data** — a lifetime is "a set of points in the
  control-flow graph," inferred by liveness, discharged at *type-check time*
  ([RFC 2094](https://rust-lang.github.io/rfcs/2094-nll.html); stable since 1.63,
  [Rust blog](https://blog.rust-lang.org/2022/08/05/nll-by-default/)); **Polonius** restates this as a
  Datalog analysis over "origins" ([DeepWiki polonius](https://deepwiki.com/rust-lang/polonius)).
  **Transpiler consequence:** Mycelium need **not reproduce lifetimes as artifacts** — they discharge
  in rustc. It must reproduce the *move discipline* and `&mut`-*exclusivity*. (This is *why* §1 needs
  the rustc borrowck oracle — to read off those two facts.)

### 4.2 Cross-language comparison — construct × target mappability

Legend: ✅ clean/mechanical · ⚠️ maps with added obligations/restriction · ❌ no mechanical mapping (residue)

| Rust construct | **Mycelium (affine, acyclic, value-sem)** | Linear Haskell | ATS (viewtypes) | Austral (linear) | Vale (gen refs) | Pony (ref caps) |
|---|---|---|---|---|---|---|
| **`move` (affine consume)** | ✅ affine move is native | ✅ `⊸` linear arrow | ✅ consume viewtype | ✅ consume | ✅ linear move | ✅ `consume` of `iso` |
| **`Copy`/`Clone` (normal types)** | ✅ value copy | ✅ `ω` arrow | ✅ non-linear type | ✅ Free universe | ✅ | ✅ `val`/copy |
| **`&T` shared borrow** | ✅ `borrow` (immutable, non-escaping) | ❌ no native borrow | ⚠️ view threaded in/out | ✅ `&[T,R]` + region | ✅ gen ref (read) | ✅ `box`/`val` |
| **`&mut T` exclusive borrow** | ✅ `unique` mutable borrow (exclusivity) | ❌ no native borrow | ⚠️ linear view in/out | ✅ `&![T,R]` + region | ✅ gen ref (write) | ✅ `iso`/`trn` |
| **Lifetimes / NLL / Polonius** | ⚠️ discharge at rustc check-time; scope/region, *not* runtime data | ❌ none | ⚠️ proof terms | ⚠️ explicit named regions | ✅ runtime gen check | ⚠️ cap scoping |
| **`Drop` (implicit, used-zero-times)** | ✅ affine silent-drop kept (matches Rust) | ⚠️ explicit consume | ⚠️ explicit free | ⚠️ **must** insert explicit destructor | ✅ Higher RAII | ✅ scope drop |
| **`Rc`/`Arc` (acyclic, shared *immutable*)** | ⚠️ shared-immutable / structural (content-addressed) sharing OK | ⚠️ via `ω` | ⚠️ refcount proof | ⚠️ wrapper | ✅ direct | ✅ `val`/`tag` |
| **`Rc<RefCell>` shared *mutable* state** | ❌ **forbidden** (affine + value-sem) | ❌ | ❌ | ❌ | ✅ (runtime check) | ⚠️ only via actor + `ref` |
| **`Cell`/`RefCell` interior mutability** | ❌ no aliased mutation | ❌ | ❌ | ❌ | ✅ | ⚠️ `ref` within actor |
| **Cyclic data (graphs, doubly-linked, back-edges)** | ❌ **structurally impossible — Mycelium is acyclic** | ❌ | ❌ | ❌ | ✅ direct | ✅ via actor |
| **`unsafe` aliasing / raw ptrs** | ❌ no analogue | ❌ | ⚠️ raw views + proof | ❌ | ⚠️ | ⚠️ `tag` identity |

### 4.3 Per-system notes (what each teaches)

- **Linear Haskell** (Bernardy/Spiwack/SPJ et al., POPL 2018) puts linearity on the **arrow** via a
  **multiplicity** (`1` vs `ω`): `t ⊸ u` consumes its argument exactly once
  ([arXiv 1710.09756](https://arxiv.org/pdf/1710.09756); [SPJ](https://simon.peytonjones.org/linear-haskell/)).
  **`move` ≈ `⊸`, `Copy` ≈ `ω`; but it has NO native borrowing** — borrowing had to be bolted on later
  ("Pure Borrow," [DOI 10.1145/3808259](https://doi.org/10.1145/3808259)). Lesson: an affine/linear core
  carries Rust's *move*, but **borrowing is a distinct feature Mycelium must design**, not a freebie.
- **ATS** (Xi): linear **viewtypes** = a *view* (linear claim about memory at an address, `T@L`) + a
  type; aliasing handled by *linear consumption of views*, with `dataview`/`dataprop`/`dataviewtype`
  ([Wikipedia: ATS](https://en.wikipedia.org/wiki/ATS_(programming_language))). It is *more* explicit
  than Rust — translation would *gain* proof obligations. Useful only if Mycelium ever wants to *reify*
  borrow provenance (which fits its EXPLAIN ethos).
- **Austral** (Borretti): linear-first, two universes (**Free** = unrestricted, **Linear** = resource),
  with explicit **regions** for borrows (`&[T,R]` / `&![T,R]`), checked by a tiny 4-state state machine
  (Unconsumed/BorrowedRead/BorrowedWrite/Consumed)
  ([spec](https://austral-lang.org/spec/spec.html);
  [checker](https://borretti.me/article/how-australs-linear-type-checker-works)). **Closest design
  cousin of the set** — its `&[T,R]`/`&![T,R]` ≈ Rust `&T`/`&mut T`, region ≈ lexicalized lifetime.
  **Key lesson: Austral is *strictly linear* (must-use, no silent drop); Mycelium being *affine* keeps
  Rust's silent-drop and avoids Austral's "synthesize an explicit destructor at every branch/return"
  tax — making Mycelium a *closer* landing zone for idiomatic Rust than a linear target.**
- **Vale**: **generational references** replace the borrow checker with an ~8-byte-per-alloc generation
  + a runtime deref check; this **permits shared mutability and cyclic graphs directly** at ~2–10.8%
  overhead vs C++ ([generational-references](https://verdagon.dev/blog/generational-references)).
  Vale is the **anti-pattern reference**: it achieves exactly what Mycelium renounces by **moving safety
  to runtime**. Its author's own enumerated "four patterns linear/affine+borrowing *fundamentally
  cannot* support" — *intrusive structures/graphs, observers, back-references, callbacks/delegates* —
  **is the canonical Rust→Mycelium residue list** ([linear-types-borrowing](https://verdagon.dev/blog/linear-types-borrowing)).
- **Pony**: six **reference capabilities** (`iso`, `trn`, `val`, `ref`, `box`, `tag`) over a
  (aliasing × mutability) matrix; **`iso` ≈ unique owner/`&mut`**, **`val`/`box` ≈ shared-immutable
  `&T`**, **`consume` ≈ `move`** ([capability-matrix](https://tutorial.ponylang.io/reference-capabilities/capability-matrix.html);
  [bluishcoder](https://bluishcoder.co.nz/2017/07/31/reference_capabilities_consume_recover_in_pony.html)).
  Best evidence that a clean **two-axis** `unique`/`borrow` decomposition mirrors Rust ownership.
  Pony's `ref` (freely-aliased mutable) — the shared-mutable case — works only because the actor model
  *serializes* access, which Mycelium doesn't have. A good lattice model for Mycelium's reference
  disciplines.
- **Cyclone** (PLDI 2002): region-based memory management, the **direct ancestor of Rust lifetimes**;
  the region calculus is the theory to borrow for scoping non-escaping borrows
  ([cyclone-regions.pdf](https://www.cs.umd.edu/projects/cyclone/papers/cyclone-regions.pdf)). **Mezzo**
  (permission/separation-logic style) — *under-verified this pass, background only.*

### 4.4 The residue — categorically impossible (not merely hard)

Every source converges: these four cannot map mechanically to an affine, acyclic, value-semantics target:
1. **Shared *mutable* state** (`Rc<RefCell<T>>`, `Arc<Mutex<T>>` aliased mutation). Affine+value-sem
   gives exclusivity *by construction*; Vale's author: linear/affine+borrowing *"fundamentally cannot
   support shared mutability."* Only Vale (runtime checks) / Pony (actor serialization) admit it.
2. **Cyclic data** (graphs, doubly-linked, parent↔child back-edges, observer/callback graphs).
   **Mycelium is acyclic — a structural wall, not a checker limitation.** Even Rust needs `Rc`/`Weak`,
   and `Rc` *"allows reference cycles … destructors not to run"* ([without.boats](https://without.boats/blog/ownership/)).
3. **Interior mutability** (`Cell`/`RefCell`/`UnsafeCell`) — *defined* as aliased mutation behind a
   shared reference, the precise inverse of value semantics.
4. **`unsafe` aliasing / raw-pointer graphs** — no analogue in any pure substructural target.

**Clean core that *does* transpile:** `move`/affinity, `Copy`/`Clone`, `&T`/`&mut T` as scoped/region
borrows, `Drop`, acyclic structural sharing. The transpiler should translate these directly,
lexicalize/region-scope borrows (Austral/Cyclone style) rather than reproduce lifetimes-as-data, and
**FLAG the four residue classes with an EXPLAIN-able diagnostic** (§3) — the never-silent rule applied
to translation. *(One open caveat for the DN: idiomatic Rust uses `Rc<RefCell>`/interior mutability
pervasively; the **fraction** of the kernel/stdlib that is residue is an empirical unknown until the
CompatibilityAnalyzer runs — do not pre-assert "the bulk translates" without measuring it; Laertes'
~11% is the cautionary precedent.)*

---

## 5. Transpile verification strategy

How to bound trust in mechanically-produced Mycelium for a **self-hosting bootstrap**, given the
never-silent rule forbids unbounded trust in the generated output.

### 5.1 Differential testing (source vs transpiled) — the workhorse, `Empirical`

The dominant verified approach across all prior art:
- **c2rust cross-checks / ReMon MVEE** (§2.1): instrument both the Rust source and the Mycelium output
  at function boundaries, **hash inputs/outputs to fixed width** (excluding padding, deref pointers),
  and run **in lockstep on shared inputs** (online MVEE) or **diff logged traces** (offline); divergence
  = a located equivalence failure ([cross-checks](https://c2rust.com/manual/cross-checks/)). **This is
  the most directly transferable mechanism** — Mycelium runs the original Rust (via its own toolchain)
  and the transpiled Mycelium on identical inputs and compares value-hashes. Earns **`Empirical`**.
- **SACTOR's FFI co-execution** (arXiv:2503.12511): embed the translated code with the original via FFI
  and run **end-to-end tests against both**, accepting a translation only when all pass; reaches
  80–93% correctness with strong test suites ([SACTOR](https://arxiv.org/html/2503.12511v3)). The
  *test-suite-as-oracle* idea: **the Rust kernel/stdlib already has tests** — reuse them as the
  differential oracle for the Mycelium output (translate the tests too, or drive both via a shared
  harness).
- **EMI / differential fuzzing** ([Le et al., PLDI 2014, DOI 10.1145/2594291.2594334](https://dl.acm.org/doi/10.1145/2594291.2594334)):
  generate input variants to widen coverage of the equivalence check.
- **py2many's golden-file + behavioral suites**: snapshot the expected Mycelium output + compile/run it
  — cheap regression net.

### 5.2 Translation validation (per-translation proof) — aspirational, `Proven`-gated

Translation validation proves *this* output is a correct translation of *this* input (originally for
compiler optimizations: [Pnueli et al.](https://link.springer.com/chapter/10.1007/11513988_29);
[TVOC](https://link.springer.com/chapter/10.1007/11513988_29)). For source-to-source it's harder and
recent ([Validated Code Translation, arXiv:2602.18534](https://arxiv.org/html/2602.18534)). **For
Mycelium:** a per-function equivalence theorem would be the *only* basis for a `Proven` tag on a
transpile — but none of the surveyed practical tools achieve it, so **default transpile guarantee =
`Empirical` (differential) or `Declared` (asserted), never `Proven` unless a checked equivalence
theorem is supplied** (VR-5). Don't over-claim.

### 5.3 Coverage metrics — making "bulk translated" a *measured*, never-asserted claim

The never-silent rule forbids asserting "the bulk translates" without evidence. Track and **publish**:
(a) **residue count** by category/granularity (the §3 manifest is the source of truth); (b) **%
items/functions/exprs translated clean vs flagged**; (c) **differential-test pass rate + input
coverage**; (d) **burn-down** of residue toward a Definition-of-Done. This makes the headline claim
`Empirical` with a number attached, not `Declared` rhetoric — and Laertes' ~11% ceiling is the standing
reminder that the number can be *low* and must be honestly reported.

### 5.4 Bounding trust in a self-hosting bootstrap (historical practice)

A transpiler that produces the language's *own* kernel/stdlib is a bootstrapping act, and the classic
trust hazard is **Thompson's "trusting trust"** ([Reflections on Trusting Trust]). The historical
countermeasure is **Wheeler's Diverse Double-Compiling (DDC)**: compile the artifact with two
*independent* toolchains and check the results are **bit-for-bit identical** — if so, both are clean or
identically backdoored, and independent backdoors matching is vanishingly unlikely
([Wheeler DDC dissertation](https://dwheeler.com/trusting-trust/dissertation/html/wheeler-trusting-trust-ddc.html);
[dwheeler.com/trusting-trust](https://dwheeler.com/trusting-trust/)). **Applied to Mycelium:** the
*reference interpreter is the trusted base* (per CLAUDE.md); a mechanically-transpiled Mycelium stdlib
should be (a) differentially tested against the Rust original (§5.1), and (b) where it becomes
self-hosting, **DDC-style cross-checked** — run the transpiled Mycelium toolchain *and* the Rust-hosted
one over the same inputs and require identical output before trusting the mechanically-produced tier.
The transpiler's output is `Empirical` until such a check, **never** silently promoted to trusted.

### 5.5 Recommended verification stack (concrete)

1. **CompatibilityAnalyzer pre-pass** → complete residue manifest *before* emission (§3.2) — the
   never-silent gate. No emission of guessed code for flagged sites.
2. **Differential value-hash testing** (c2rust/ReMon style) of every translated function against the
   Rust original, driven by the **reused Rust test suite** (SACTOR-style oracle).
3. **Golden-file regression** (py2many style) for output stability.
4. **Coverage/residue dashboard** (§5.3) — "% clean, residue-by-category, diff pass-rate" as the
   measured basis for any "bulk translated" claim.
5. **DDC cross-check** at the self-hosting boundary (§5.4) before trusting the transpiled tier.
6. **Guarantee tags**: transpiled code = `Empirical` (passed differential trials) or `Declared` (only
   asserted); a residue hole = no code + a record; `Proven` only with a checked equivalence theorem
   (not expected from a mechanical transpiler).

---

## 6. Annotated bibliography (URLs)

**Rust front-end tooling**
- syn docs — *purely syntactic, no types/names/traits.* https://docs.rs/syn/latest/syn/ · https://crates.io/crates/syn
- rust-analyzer architecture — `hir` as the (unstable) library façade. https://rust-analyzer.github.io/book/contributing/architecture.html
- RA Type System & Inference — *exposes types/traits/mutability/adjustments, **no borrowck/lifetimes***. https://deepwiki.com/rust-lang/rust-analyzer/5.3-type-system-and-inference
- `ra_ap_hir` crate. https://docs.rs/ra_ap_hir/ · https://crates.io/crates/ra_ap_rust-analyzer
- rustc-dev-guide: borrow-check (`mir_borrowck`, region constraints). https://rustc-dev-guide.rust-lang.org/borrow-check.html · MIR: https://rustc-dev-guide.rust-lang.org/mir/index.html
- Building a `rustc_driver` (custom driver, `rustc_private`). https://jyn.dev/rustc-driver/ · MIR instrumentation (nightly-only, `override_queries`): https://emavan.com/blog/2025/mir-instrumentation/

**Transpiler prior art**
- c2rust repo + philosophy + skip-and-warn. https://github.com/immunant/c2rust · known-limitations: https://github.com/immunant/c2rust/blob/master/docs/known-limitations.md
- c2rust cross-checks (ReMon MVEE, value-hashing). https://c2rust.com/manual/cross-checks/ · design: https://github.com/immunant/c2rust/wiki/Cross-checks-design · hashing: https://c2rust.com/manual/docs/cross-check-hash.html
- Laertes — "Translating C to Safer Rust," OOPSLA 2021. https://hardekbc.github.io/files/emre21translating.pdf · https://dl.acm.org/doi/10.1145/3485498
- **"Aliasing Limits on Translating C to Safe Rust," OOPSLA 2023 — ~11% of pointers liftable; residue is *fundamental*.** https://dl.acm.org/doi/abs/10.1145/3586046 · https://2023.splashcon.org/details/splash-2023-oopsla/20/Aliasing-Limits-on-Translating-C-to-Safe-Rust
- C2SaferRust (second-hand Laertes figures — *flag `Declared`*). https://arxiv.org/html/2501.14257v1
- corrode + author's c2rust-vs-corrode retrospective (lineage; `idiomatic` never built). https://github.com/jameysharp/corrode · https://jamey.thesharps.us/2018/06/30/c2rust-vs-corrode/ — **"Sherpa" does not exist; real sibling = Citrus.**
- JSweet spec/`@Erased`/sound errors. https://raw.githubusercontent.com/cincheo/jsweet/master/doc/jsweet-language-specifications.md · https://www.jsweet.org/faq/
- TeaVM Diagnostics / AOT build-time located errors. https://teavm.org/docs/intro/troubleshooting.html · https://github.com/konsoletyper/teavm/discussions/963
- Haxe untyped/`__js__` (unvalidated, syntax-marked) + macro diagnostics; silent file-substitution anti-pattern. https://haxe.org/manual/target-syntax.html · https://haxe.org/manual/type-system-untyped.html · https://haxe.org/manual/macro.html · https://haxe.org/manual/lf-target-specific-files.html
- py2many (visitor + typed exception taxonomy). https://github.com/py2many/py2many · exceptions: https://github.com/py2many/py2many/blob/main/py2many/exceptions.py · Python ast: https://docs.python.org/3/library/ast.html

**Never-silent residue / diagnostics**
- rustc JSON diagnostics + `Applicability` lattice (MachineApplicable/MaybeIncorrect/HasPlaceholders/Unspecified). https://doc.rust-lang.org/rustc/json.html · https://doc.rust-lang.org/nightly/nightly-rustc/rustfix/diagnostics/index.html · https://rustc-dev-guide.rust-lang.org/diagnostics/diagnostic-structs.html

**Ownership → affine/uniqueness mapping**
- Rust is affine (drop=weakening, no contraction); safety from affinity; borrow = suspended affinity. https://without.boats/blog/ownership/ · https://borretti.me/article/type-systems-memory-safety
- NLL (lifetime = CFG points) RFC 2094 + stabilization; Polonius (Datalog origins). https://rust-lang.github.io/rfcs/2094-nll.html · https://blog.rust-lang.org/2022/08/05/nll-by-default/ · https://deepwiki.com/rust-lang/polonius
- Linear Haskell (multiplicity on arrows; no native borrow). https://arxiv.org/pdf/1710.09756 · https://simon.peytonjones.org/linear-haskell/ · Pure Borrow: https://doi.org/10.1145/3808259
- ATS (viewtypes/dataview/dataprop). https://en.wikipedia.org/wiki/ATS_(programming_language)
- Austral (linear-first, regions, 4-state checker; *strictly linear* vs Mycelium affine). https://austral-lang.org/spec/spec.html · https://borretti.me/article/how-australs-linear-type-checker-works
- Vale (generational refs; permits shared-mutable + cycles at runtime; the "four lost patterns" residue list). https://verdagon.dev/blog/generational-references · https://verdagon.dev/blog/linear-types-borrowing
- Pony (reference capabilities matrix; iso≈unique, val/box≈shared-immutable, consume≈move). https://tutorial.ponylang.io/reference-capabilities/capability-matrix.html · https://bluishcoder.co.nz/2017/07/31/reference_capabilities_consume_recover_in_pony.html
- Cyclone (regions; ancestor of Rust lifetimes). https://www.cs.umd.edu/projects/cyclone/papers/cyclone-regions.pdf

**Verification / bootstrapping**
- Translation validation (Pnueli; TVOC). https://link.springer.com/chapter/10.1007/11513988_29
- Validated Code Translation (recent source-to-source). https://arxiv.org/html/2602.18534
- SACTOR (FFI-based differential verification, two-stage). https://arxiv.org/html/2503.12511v3
- EMI / differential compiler testing. https://dl.acm.org/doi/10.1145/2594291.2594334
- Wheeler, Diverse Double-Compiling (countering Trusting Trust in bootstraps). https://dwheeler.com/trusting-trust/dissertation/html/wheeler-trusting-trust-ddc.html · https://dwheeler.com/trusting-trust/

---

### Confidence / transparency ledger
- **High (primary-source verified):** syn = syntax-only; RA `hir` exposes types/traits/mutability but *not* borrowck/lifetimes; rustc MIR `mir_borrowck` is the ownership oracle (nightly/`rustc_private`); c2rust skip-and-warn + cross-check/ReMon mechanism; py2many exception taxonomy; rustc `Applicability` lattice; Rust = affine; Austral strictly-linear vs Mycelium affine; Pony cap matrix; Vale's "four lost patterns"; DDC.
- **Medium (`Declared`, second-hand):** the Laertes ~8.3%/~21%/~20%-of-functions figures via C2SaferRust; the ~11% headline is from the OOPSLA-2023 abstract (primary-abstract, but the PDF body could not be parsed in-env). Re-check against the primary PDFs before quoting as anything above `Declared`.
- **Corrections to the brief:** "Sherpa" (C→Rust) — *does not exist*, do not cite; real sibling is **Citrus**. Mezzo — under-verified this pass, background only.
