# Changelog

All notable changes to this project are recorded here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Dates are ISO-8601.

This project is in **design + Rust-first implementation**; entries cover both the documentation
corpus and the landing kernel/stdlib code. Semantic versioning will begin when the kernel stabilizes.

## [Unreleased]

### Added (2026-06-26: DN-39 — KC-3 kernel-promotion review (trusted core stays unchanged))

- **DN-39 — Kernel-Promotion Review (KC-3 trusted-core audit)**
  (`docs/notes/DN-39-Kernel-Promotion-Review-KC3.md`, **Draft/advisory**) — captures the
  maintainer-commissioned review: should any non-kernel functionality become a trusted-core component?
  **Recommendation: NO — zero promotions; the kernel boundary stays UNCHANGED**, KC-3 held on merit. The
  one candidate (spore `content_address`) is a decisive KEEP-OUT (verifiable ⇒ must be verified, not
  trusted). Records the **security finding** the review surfaced — the `v0` content-address injectivity
  flaw (supply-chain substitution vector), now **`Proven`-and-FIXED** in PR #617 — and the generalizable
  principle: *a deterministic encoding is the most testable artifact in the system, so it is the last
  thing that should be axiomatized into the kernel* (VR-5 applied to the trust boundary). Enacts nothing;
  awaiting ratification.

### Added (2026-06-26: DN-35 Phase-1a foundation — honest baseline + Binary[64] HTTPS example)

- **`docs/measurements/DN-35-baseline-2026-06-26.md`** — the release-honest "before" baseline for the
  reclamation prototype (the data Increments 1–2 are measured against). MEM-4 static dup-reduction
  91.3% (`Exact` count, corpus-dependent); `mycelium-bench` interp-vs-aot-env timings (aot-env wins
  rec-build 1.24× / rec-mutual 1.18×, slower on most small kernels — the gap DN-35 targets). Caveats are
  load-bearing (ephemeral dup-printer, `mlir-dialect` OFF, spawn-bound direct-llvm/jit); runtime
  heap/RSS counters still need wiring before Increment-1 memory claims are meaningful.
- **`docs/examples/binary64-https-downloader.myc`** (+ README) — illustrative, lexicon-correct
  `Binary[64]` general programming + a small HTTPS downloader with named security practice (TLS-verify
  on + non-disableable, HTTPS-only, bounded streamed reads, secrets-from-env, never-silent
  `?`-propagation, integrity + size check, mandatory finite timeout/budget). Labeled **non-runnable**
  (design phase); uses the DN-31 **recorded-direction (Draft), not-yet-landed** target surface (`[]`/`=>`/`Binary[64]`),
  flagged in-header — it previews the surface and rides epic #27 to become parseable.
- Phase-1a's lexicon docs-sweep correctly returned **zero edits** (G2/VR-5): the surface lexicon is
  Accepted *direction* but unlanded, and DN-31 mandates a coordinated all-at-once grammar wave (epic
  #27), so piecemeal edits would diverge the docs from the grammar oracle/lexer/conformance corpus.

### Security (2026-06-26: spore identity encoding — injectivity fix `v0 → v1`)

- **Fixed a content-address injectivity flaw in `mycelium-spore::content_address`** (surfaced by the
  KC-3 trusted-core review as a side finding). The `v0` pre-image emitted every author-influenced field
  (surface names, source `path`, dep `name`/`phylum`/`hash`) **space/newline-delimited with no
  length-prefix or escaping**, so a crafted field containing a space or newline could shift a record
  boundary and **alias two distinct spore DAGs onto one pre-image → one address** (a second-pre-image
  collision; a **supply-chain substitution vector** against dep-pinning / resolve-by-hash / immutability
  detection). All three `ResolvedDep` fields are free-text manifest strings, so the collision needed no
  hash-preimage or filesystem — **Proven** (a byte-identical `v0` pre-image for two distinct dep DAGs is
  exhibited in the regression tests). **Fix:** `v1` **length-prefixes every variable-length field**
  (`<bytelen>:<bytes>`, the load-bearing part) plus per-section record counts (defense-in-depth), making
  the encoding injective by construction; added adversarial injectivity property tests
  (`crates/mycelium-spore/src/tests/lib_tests.rs` → `mod injectivity`).
- **DRY unification (the real root cause):** spore identity was encoded in **two** places — the verify
  path (`mycelium-std-spore::recompute_identity`) was a hand-copied duplicate that still stamped `v0`,
  so a freshly-built `v1` spore failed `verify()` cross-crate. `content_address` is now the **single
  `pub` canonical encoder**, and `recompute_identity` (hence `from_value`) delegates to it — divergence
  is now structurally impossible. Verified green across the full reverse-dependent closure
  (`mycelium-spore` 22, `mycelium-std-spore` 44, `mycelium-std-recover`/`mycelium-cert`/`mycelium-cli`).
  **Note:** the header bumps `v0 → v1`, which **re-addresses every spore** (append-only supersession of
  the explicitly provisional format; acceptable pre-1.0 — no live registry).
- **KC-3 holds:** the right response to a verifiable encoding flaw is *more verification* (a property
  test), **not** promoting the encoding into the trusted core — identity is a deterministic, checkable
  function and must be *verified*, never *trusted* (VR-5 applied to the trust boundary). `mycelium-spore`
  stays a verified library above the kernel; the trusted-core boundary is unchanged.

### Changed (2026-06-26: DN-35 ratified Draft → Accepted)

- **DN-35 — Env-Machine Reclamation** ratified **Draft → Accepted** (maintainer). Accepts the §10 design
  direction (static-decisions/dynamic-verification with a trusted core interpreting only
  `incref`/`decref`/`is-unique`/`reuse-or-alloc`; drop-guided Frame-Limited reuse with the runtime
  `is-unique` gate as the mandatory safety valve; the §5 content-address side-condition; the §6 RC ⊕
  region exactly-once invariant; the `fast`/`certified` split) as the *design direction*. Accepted ratifies
  the direction, **not** an implementation — enacts no code, upgrades no guarantee (prior-art results
  `Proven`-at-source; the two Mycelium-specific obligations stay `Empirical`/`Declared` until prototyped +
  property-tested; the §9 audit trail `Exact` count / `Declared` correspondence — VR-5). The build is the
  forward epic (E12 Increment 3 / task #6), beginning at §9 Increment 1 behind the §2.3 oracle.

### Changed (2026-06-26: DN-38 ratified Draft → Accepted)

- **DN-38 — The Layered-Lowering Atlas & Generative Sugar** ratified **Draft → Accepted** (maintainer).
  Accepts the **seamless-gradient thesis**, the **lowering law** (every feature lowers to L0 with the
  same observable meaning; small IL-grammar-checked semantics-preserving passes; kernel never grows —
  KC-3), the **honest-refinement** rule, the **generative-lowering + inspectable-desugaring** construct
  set, and the **§8.1 naming** (delegation `via` · generative `derive` · inspector `reveal`) as the
  *design direction*. Accepted ratifies the direction, **not** every open question — §8's *architectural*
  questions (nanopass-separate-passes vs single `elab.rs`; `reveal` v0 vs post-core; round-trip scope;
  wave sequencing; the `@matured` gap) remain **open**. No guarantee upgraded (layering +
  observational-identity + built lowerings `Exact`; framing + construct + naming `Declared` — VR-5).
  Also tidies the now-stale header naming pairs flagged in review (`derive`/`weave` → `derive`,
  `reveal`/`expand` → `reveal`).

### Changed (2026-06-25: DN-38 §8.1 naming settled — `derive` + `reveal`)

- **DN-38 §8.1 Construct naming fully settled** (append-only; **DN-38 stays Draft**). The
  generative-lowering construct is **`derive`** (plain-first over the coined `weave` — conventional,
  discoverable: Rust `#[derive]` / Haskell `deriving`; coinage cleared no mnemonic bar). The inspector
  is **`reveal`** (over `expand`, which overloads macro-*expansion*; the construct *discloses* the real
  already-lowered L0 term, reinforcing the abstracted-never-hidden thesis). Combined with the earlier
  `via` resolution, all three §8.1 naming questions are now closed (delegation `via`; generative
  `derive`; inspector `reveal`).

### Changed (2026-06-25: DN-36 + DN-37 ratified Draft → Accepted; delegation keyword → `via`)

- **DN-36 — Safe & High-Performance Iteration** ratified **Draft → Accepted** (maintainer). Accepts
  the **two-tier surface** (bounded Tier-1 idiom + budget-gated Tier-2 open form, both desugaring to one
  tail-recursive `Fix`) and the **§6 high-performance roadmap** as the *design direction*. Enacts no
  code, upgrades no guarantee (safety `Exact`, perf mechanism `Proven`-in-lit / `Empirical`-`Declared`,
  surface `Declared`); each §6 item still gates on its own build epic (VR-5).
- **DN-37 — Object & Behavior Model (and the Sigil Category Scheme)** ratified **Draft → Accepted**
  (maintainer). Accepts the **objects-vs-ADTs framing**, the **composition-ranked inheritance-emulation
  menu**, and the **Sigil Category Scheme** as the *design direction*. Accepted ratifies the direction,
  **not** every open question — §8's build-order, first-class-`class`-vs-implicit, dynamic-dispatch,
  encapsulation, and `~`-form questions remain **open** and tracked. No guarantee upgraded (foundation
  `Exact`; menu + sigils + sugar `Declared`).
- **Delegation keyword resolved to `via`** (DN-38 §8.1, append-only; DN-38 stays Draft). The maintainer
  delegated the call; `via` (preposition = conduit, not agent) is honest about Mycelium's **static,
  by-value forwarding** — no late-binding/agency over-claim (VR-5 applied to naming), the prepositional
  twin of the `~>` flow-glyph, matching the cited Kotlin `by` precedent. `derive`/`weave` and
  `reveal`/`expand` remain open. (Note: the `0t` trit-literal shorthand — analogous to `0b`/`0x` — is
  already the DN-31 scheme; no change.)

### Added (2026-06-25: DN-35 — env-machine reclamation, research-backed)

- **DN-35 — Env-Machine Reclamation (the Deferred Big Step past the §9 Audit Trail)**
  (`docs/notes/DN-35-Env-Machine-Reclamation.md`, **Draft/advisory**), backed by `research/16`+`17`
  (internal `eval_machine`-seam dossier + external Perceus/Frame-Limited-Reuse/RC⊕region prior art).
  The load-bearing memory note behind DN-36's constant-heap loops.
  - **The deferred step:** thread *actual* Mycelium-level reclamation into the AOT env-machine
    (`eval_machine`, `crates/mycelium-mlir/src/aot.rs`) — today the §9 audit-trail bridge
    (`rc_plan.rs`) is **audit-only**; the env-machine lets Rust manage values. DN-35 designs the
    big step past it.
  - **Principle — static decisions, dynamic verification:** the compiler statically plans
    drop-guided / **Frame-Limited reuse** (Lorenzen–Leijen) and inserts reclamation points; the
    runtime **`is-unique` gate** (the `RcCell` probe, MEM-2) is the **sound safety valve** — a wrong
    static guess costs throughput, never safety.
  - **In-place-reuse-vs-content-address obligation:** reuse a cell in place only at `rc==1`; a
    **weak intern table** + **evict-or-copy** keeps content-addressing (ADR-003) sound when a reused
    cell was interned.
  - **RC⊕region exactly-once** (Gay–Aiken): region batch-reclaim and RC drop must not double-free.
  - **`fast`/`certified` split** (ADR-032/RFC-0034): `fast` trusts the static plan + the `is-unique`
    valve; `certified` additionally emits per-reclamation audit records (the §9 trail) and can run
    the reference RC-evaluator (`eval.rs`) as oracle.
  - Open-question gating set carried into the build epic (Q4 reuse-granularity / Q6
    intern-eviction-policy / Q2 region-RC-ordering). Internal claims `Empirical`/`Declared`; external
    mechanisms `Empirical`/`Proven`-in-literature (primary-source-checked). Enacts nothing.

### Added (2026-06-25: DN-38 — the layered-lowering atlas & generative sugar, research-backed)

- **DN-38 — The Layered-Lowering Atlas & Generative Sugar (one seamless gradient to L0)**
  (`docs/notes/DN-38-Layered-Lowering-Atlas.md`, **Draft/advisory**), backed by `research/25`
  (pit-of-success + nanopass/language-tower + inspectable-desugaring/generative-lowering prior art).
  The **unifying** note the feature DNs (DN-36/37, future DN-35) hang off.
  - **Thesis:** Mycelium is **one seamless language** — the L0–L3 "levels" are how the *compiler
    lowers*, not modes the *programmer declares*; a program freely intermixes high-sugar and
    low-explicit forms because they're the same program at different points on a **desugaring
    gradient**. The batteries-included feature **superset** abstracts L0 away — *abstracted, never
    hidden* (always `reveal`-able). (Racket "languages as libraries" taken further: the low level is
    the *same* language less-sugared.)
  - **The lowering law:** every feature lowers to L0 with the same observable meaning; each pass small,
    IL-grammar-checked, semantics-preserving; kernel never grows (KC-3). The "seamless gradient" and
    the "verified tower" (nanopass/CompCert) are one property from two seats (RFC-0012
    observational-identity + NFR-7).
  - **Honest refinement:** levels invisible, but `wild`/`!{io}`/`@matured`/guarantee-tags stay
    explicit + **level-independent** (they surface *what the code does* — the audit trail survives
    free mixing).
  - **Generative lowering:** terse-params → explicit, inspectable, **content-addressed** L0 artifact
    (`derive`/`weave`; Lombok-style hidden magic is structurally impossible — the only output is an
    L0 term; same-intent→same-identity free via ADR-003; Dynel/RFC-0013 errors woven in).
  - **Inspectable desugaring:** `reveal` shows the *real* L0 term (not a lossy text render),
    layer-hideable, `delaborate∘lower=id` round-trip gated by `certified` (ADR-032).
  - Includes the **per-feature Lowering Map** (living checklist). Layering + observational-identity +
    built lowerings `Exact`; framing + construct + naming `Declared`. Enacts nothing.

### Added (2026-06-25: DN-37 — object & behavior model + sigil category scheme, research-backed)

- **DN-37 — Object & Behavior Model (and the Sigil Category Scheme)**
  (`docs/notes/DN-37-Object-and-Behavior-Model.md`, **Draft/advisory**), backed by two new research
  records (`research/23` internal, `research/24` external prior-art). How objects render and how to
  emulate ergonomic inheritance, composition-aligned, in the value-semantics + trait model.
  - **Framing (Cook's duality):** Mycelium sits on the **ADT / static-dispatch horn** by construction
    — strong at adding *operations* + clean binary methods; constrained at *open representations*.
  - **Built (`Exact`):** data via one keyword `type` (record/sum, `Construct`/`Match`, content-addressed
    structural equality); traits + `impl` + bounded generics + **coherence** via **monomorphization**;
    nodule-level `pub`. **Content-addressing forces coherence** (ADR-003) — firmer than Haskell.
  - **Deferred/greenfield** (honest correction — the ergonomic-inheritance layer is mostly future work):
    super-traits (designed RFC-0019 §4.3, unparsed), default method bodies, delegation, `@`-decorator,
    dynamic dispatch / record-of-closures (`FieldSpec` is `Repr|Data` only — no function-typed field),
    field/opaque encapsulation, associated types (v2).
  - **Ranked emulation menu:** default methods → super-traits → `~>`-delegation → `@`-decorator →
    embedding → row-poly → record-of-closures (escape hatch — its `fix`-over-`self` *is* the cyclic
    back-edge LR-9 excludes; `Declared`). Never-silent caveat: forwarding-without-late-binding is the
    honest behavior (avoid Rust's silent `Deref` fake-"is-a").
  - **Settled sigil-category scheme** (verified non-conflicting against the lexer): `@` wrap/decorate ·
    `#` identity · `$` splice · `?` fallibility · bare `~` approximate / `~>` delegate · `!` effects ·
    `&` conjoin (not borrow) · `` ` `` reserved (quasiquote/raw). One glyph = one layer of discourse;
    grammar home DN-31/RFC-0030/epic #27.
  - **Perf:** monomorphization is the vtable-free default; Swift-existential (3-word inline buffer) for
    the eventual dynamic-dispatch. Foundation `Exact`; menu+sigils+sugar `Declared`. Enacts nothing.

### Added (2026-06-25: DN-36 — safe & high-performance iteration, research-backed)

- **DN-36 — Safe & High-Performance Iteration in the Value-Semantics Model**
  (`docs/notes/DN-36-Safe-and-High-Performance-Iteration.md`, **Draft/advisory**), backed by two new
  research records (`research/21` internal, `research/22` external prior-art). Captures how iteration
  stays **functional + safe + high-performance** without a mutable loop variable: keep value-semantics
  in the surface, source mutation from the lowering (TRMC + Perceus/FBIP reuse — Koka's lesson).
  - **Safety is already built** (`Exact`): bounded `for`→`Fix` fold, O(1)-host-stack trampoline, three
    never-silent budgets (`fuel`/`max_depth`/`alloc`), `while`/`loop` removed-by-construction with a
    teaching diagnostic, total `mycelium-std-iter` combinators.
  - **Recommended two-tier surface** (the maintainer's "both"): a bounded total-by-construction Tier-1
    idiom (termination `Proven`) + a sugared budget-gated Tier-2 open `loop`/`while` (catchable
    `BudgetExhausted`, `Declared`), both desugaring to one tail-recursive `Fix`; `recur`-checked tail
    position; folds in DN-31's grammar input (line-breaks/indentation, `,` delimiter).
  - **High-performance roadmap** makes the recursion-aware FBIP increment (E12 Increment 3 / task #6)
    load-bearing — RC across `Fix`, recursive verifying evaluator, value-affecting reuse, native
    structural accumulators (`tailcc`/`musttail`), region-per-iteration, interpreter TRMC parity,
    benchmarks — to deliver constant-memory loops for general structural accumulators (today only a
    scalar `Binary{8}` accumulator is allocation-free).
  - Honest posture: safety `Exact`; perf mechanism `Proven`-in-literature / `Empirical`-`Declared`-for-
    Mycelium until built+benchmarked; surface `Declared`. Enacts nothing; indexed in Doc-Index.

### Changed (2026-06-25: corpus-audit decision log — maintainer rulings encoded append-only)

- **Maintainer decisions from the corpus audit**, encoded across the corpus (append-only; no original
  text deleted; the only status changes are forward moves, each with a changelog entry). 26 docs +
  the H1 re-pin:
  - **H1 (code):** re-pinned `mycelium-std-math`, `mycelium-std-sys`, `mycelium-std-sys-host` from
    the drifted `0.1.0` back to `0.0.0` + `publish = false`, restoring ADR-018's uniform policy
    (Cargo.lock refreshed; workspace still builds). ADR-018 erratum records the corrected facts +
    count (44→50).
  - **H2/H3:** ADR-020 record-completion (Proposed→Accepted→Enacted entries; `runtime.rs` 106→116);
    DN-23 footer + RFC-0025/M-705 supersession, **Status Draft→Resolved**.
  - **Decisions-to-implement (recorded as decisions + tracked epics, no code yet):** RFC-0017 §4.1
    top-down `@matured` inheritance + §4.4 record (D1); E19-1 `Repr::Seq`/`Repr::Bytes` value-model
    additions + the T1-after-E19-1 gate (D2/D3, ADR-024); adopt **`[]` for type-args** superseding
    RFC-0030's `<>` direction, with the line-break/indentation + `,`-delimiter design input folded in
    (D7, DN-31); RFC-0030 Draft→Proposed sequenced behind the grammar wave (D6).
  - **Decisions-to-defer:** Gate A2/A3/A4/A5 pass-criteria post-T1 (D4); T4/T9 stdlib-in-Mycelium /
    self-hosting post-core (D5); Theme-A verification batch post-T1 (D8).
  - **Hygiene batch:** ADR-029/030/031 in-body `Status: Proposed`→Accepted (+ ADR-031 ID erratum);
    DN-21 **Draft→Enacted**; DN-16/17/19/06/14 + locator errata (DN-10/22/24/26, RFC-0004/0016/0035,
    ADR-023 index). doc_refs resolve; markdown clean in the edited files.

### Added (2026-06-25: exhaustive corpus-alignment audit record — all 92 DNs/ADRs/RFCs)

- **`research/20-corpus-alignment-audit-RECORD.md`** — an exhaustive read-only doc↔code alignment
  audit of the *entire* design corpus (34 DN + 23 ADR + 35 RFC = 92 docs), one auditor per doc,
  cross-checking every status + normative claim against the landed tree (code/lexicon treated as
  ground truth). Result: **0 Critical, 8 High, 7 Medium, 37 Low**; verdicts **76 honest /
  6 internally-inconsistent / 6 stale / 3 over-claims / 0 under-claims**; **no VR-5 guarantee-tag
  violation anywhere** — every drift is status/fact/locator staleness, not a strength over-claim.
  The record includes a per-status roster (every doc exactly once) and a consolidated open-question
  ledger (~9 near-term-gating, concentrated in the core/full-language 1.0.0 gate). All corrections
  are recommend-only + append-only-safe (none applied in this record). Flags one item for an explicit
  maintainer decision: three crates at `0.1.0` (vs the ADR-018 "all `0.0.0`" premise) suggest a
  partial undocumented release-cut to reconcile.

### Changed (2026-06-25: corpus alignment audit — append-only corrections to the memory/transpiler cluster)

- **Doc↔code alignment corrections** from a two-cluster audit (memory/reclamation + self-hosting/
  transpiler) against the actual codebase. All append-only (no Status moved; no original text deleted
  from any Accepted doc — the only `−` lines are in the Draft DN-34 and a Doc-Index row):
  - **DN-33 §6.1 (Critical):** the stale "prerequisite gap" passage claimed `mycelium-mir-passes`
    does not exist and MEM-4 is blocked — now false (the crate is fully built; the §8.1 Q2 ruling
    makes the "add a field to `node.rs`" step moot). Added a dated correction callout; §6.1 prose
    preserved as a historical snapshot.
  - **DN-34 §3 + Doc-Index (Critical/High):** corrected the "reuse the MEM-4 ownership analysis"
    category error — Rust ownership facts come from a rustc/rust-analyzer front-end (authoritative:
    rustc MIR `mir_borrowck`); `syn` is syntax-only; MEM-4 is a *downstream* RC optimizer over
    Mycelium Core IR, not the transpiler's ownership analyzer. Annotated the HOF/`?` rows with real
    surface status (RFC-0024 Proposed; capturing closures auto-Impossible; `?` absent from v0).
  - **DN-32 + RFC-0027 (High):** added honest-scope notes — the model is implemented at the runtime
    + MEM-4 static tiers with all three §9 triggers live, but reclamation is **not yet threaded into
    the AOT env-machine** (env-machine still Rust-manages values; §9 output is an additive audit
    trail; seam = `mycelium-mlir/src/aot.rs::eval_machine`).
  - **RFC-0028 §4.5 (Low):** clarified v0 `std.rand` entropy = `/dev/urandom` via `std::fs`
    (`getrandom(2)` deferred). **research/18 / DN-26:** errata for the MEM-4-built / `colony`-`hypha`-
    expression-only / DN-14-row-9 freshness nits.

### Added (2026-06-25: research records — env-machine reclamation + transpiler evidence base)

- **Four research records** (`research/16`–`research/19`), landed as the evidence base for the next
  two memory/self-hosting steps. Each is a multi-agent research pass (internal repo-grounded +
  external web prior-art), adversarially verified, with honest guarantee tags (internal claims
  `Empirical`/`Declared`; external mechanisms `Empirical` where primary-source-checked, `Declared`
  where second-hand and flagged):
  - **16/17 — Env-machine reclamation (a future DN-35).** The ground truth for threading *actual*
    Mycelium-level reclamation into the AOT env-machine (the deferred step past the §9 audit trail):
    the exact `eval_machine` seam (`crates/mycelium-mlir/src/aot.rs`), the reference RC-evaluator as
    correctness oracle, the open design decisions; plus a prior-art survey (Perceus / Frame-Limited
    Reuse, Counting Immutable Beans, FP² FIP, Lean 4 RC, RC⊕region coupling) with a concrete
    "static decisions, dynamic verification" recommendation and the one novel obligation Mycelium
    must own (in-place reuse vs content-address identity).
  - **18/19 — Rust→Mycelium transpiler (DN-34 evidence).** The two maintainer seed projects (py2rust
    + py-rust-bridge) with reuse verdicts; Mycelium-as-transpile-target readiness; the
    construct-mapping table; plus a prior-art survey (front-end choice — `syn` vs rust-analyzer HIR
    vs rustc MIR `mir_borrowck` as the ownership oracle; c2rust/SACTOR preserve-first architecture;
    never-silent residue reporting; the Laertes ~11% ceiling as an honest caution on "the bulk
    transpiles"; diverse-double-compiling for bootstrap trust).
  - Indexed in `docs/Doc-Index.md`. These enact nothing and ship no code — they back the future
    DN-35 and the DN-34 follow-on design work.

### Added (2026-06-25: MEM-4 corpus — Increment-2 reuse + audit-trail measurement)

- **MEM-4 measurement, broadened** (`mycelium-mir-passes::corpus`): the Q5 corpus measured only
  Increment 1's `Dup` reduction. A companion measurement now covers the rest of the built analysis
  tier over the **same** corpus — `measure_mem4` / `measure_mem4_standard` → `Mem4Report`:
  - **Reuse sites** — the count of `RcNode::MoveUnique` annotations `emit_reuse` emits (Increment 2's
    `rc == 1` reuse points), each **machine-verified sound** (reached at rc == 1 — the reference
    evaluator would error `UnsoundUnique` otherwise) and confirmed semantics-preserving against the
    owned emission.
  - **Reclamation records** — the per-term `rc → 0` reclamation count over the elided emission: the
    size of the RFC-0027 §9 audit trail the AOT tier (`mycelium-mlir::rc_plan`) emits.
  - Added three sole-owned-move terms to `standard_corpus` so the reuse dimension is exercised
    (`result_move`, `borrow_then_sole_move`, `sole_move_after_drop`); the Q5 dup-reduction stays
    ~91% (the additions are dup-neutral or a further win). 7 new tests (≥ 3 reuse sites located, all
    machine-verified sound + preserved, per-term/aggregate consistency).
  - Honest tags: the counts are `Exact` (read off the IR / reference machine); soundness is
    `Empirical` (the verifying evaluator over the corpus); corpus representativeness stays `Declared`
    (no Mycelium program population to sample yet — DN-33 §8.1). Q5 `CorpusReport`/`measure` left
    untouched (the gate is stable). Clippy `-D warnings` clean.

### Added (2026-06-25: MEM-4 → AOT — the RFC-0027 §9 reclamation audit trail)

- **MEM-4 AOT wiring** (`mycelium-mlir::rc_plan`): the bridge that finally **consumes** the MEM-4
  static analysis (`mycelium-mir-passes`) at execution time. It turns the borrow-elided RC-emission's
  predicted reclamations into the never-silent RFC-0027 §9 EXPLAIN/audit trail, emitted from the AOT
  path (previously only the runtime `RcCell` probe produced records).
  - **`emit_reclamation_plan(node, sink, scope_id, sweep_epoch)`** — runs `emit_elided` → the
    reference RC-evaluator (`eval`) and emits one `ReclamationRecord` (trigger `RcZero`) per
    `rc → 0` reclamation the analysis predicts, to a `ReclamationSink`. Returns the typed `RcPlanError`
    (not an empty plan) for a term outside the analysable fragment — never-silent (G2).
  - **`run_with_reclamation(node, prims, swap, sink)`** — computes the value with the **unmodified**
    trusted env-machine (`aot::run_core`) and emits the plan **additively** alongside it
    (`RcRun { value, reclaimed: Option<usize> }`); an out-of-fragment term yields `reclaimed: None`,
    an explicit documented skip.
  - **Honest scope (VR-5):** the AOT env-machine still **Rust-manages values**, so this is the
    *observable* audit trail of *where* the static analysis says reclamation occurs — **not** a
    change to execution. A bug here is a wrong/missing audit record, never a wrong value (DN-33 §2;
    the runtime `RcCell` probe remains the sound fallback). Threading actual reclamation into the
    env-machine is the deferred big step (RFC-0027 §10). The record *count* is `Exact`; the analysed
    fragment (straight-line) and the synthetic `rcplan:<id>` `value_meta_hash` are `Declared`.
  - 5 tests (lib); clippy `-D warnings` clean; `mycelium-core` + the env-machine untouched (KC-3);
    the dependency graph stays a DAG (`mir-passes`/`std-runtime` are upstream of `mlir`).

### Added (2026-06-25: MEM-4 Increment 2 — `rc == 1` reuse annotation, machine-verified)

- **MEM-4 Increment 2** (`mycelium-mir-passes`, DN-33 §6 D-3): the `rc == 1` reuse annotation, now
  unblocked by the Q5 gate. A new IR node `RcNode::MoveUnique` marks a consuming move **statically
  proven to be the sole owner** (reference count exactly 1 → the runtime `RcCell::drop_ref`
  `UniqueOwner` branch is guaranteed to fire → the allocation is **FBIP-reuse-eligible**).
  - **`emit::emit_reuse`** — a superset of `emit_elided`: a `let` binding that is a **sole-owned
    single move** (`is_sole_owned_move` — used exactly once, in a move position) has that move
    emitted as `MoveUnique`. Conservative (only the unambiguous single-move case; multi-move
    last-consume is a later refinement); `Lam` params still `Owned`.
  - **Machine-verified soundness:** the reference RC-evaluator (`eval`) **checks** every `MoveUnique`
    — it errors (`RcError::UnsoundUnique`) if the annotation is reached at a reference count other
    than 1. So a wrong annotation cannot slip through (mutation-tested: a hand-built `dup x;
    MoveUnique(x)` at rc=2 is caught). Semantics-preservation is confirmed against the owned emission
    (same reclamation multiset) over the corpus.
  - Honest tags: the annotation is semantics-preserving and **machine-verified** — `Empirical`
    (differential + the verifying evaluator), **not `Proven`** (a mechanized proof is Phase 3); the
    reuse-site **count** is `Exact` (read off the IR); the FBIP **performance** benefit stays
    `Declared` (the actual cell-recycling is downstream codegen, not yet wired). 43 tests (8 new);
    clippy `-D warnings` clean; Core IR untouched (KC-3).

### Added (2026-06-25: DN-34 — Rust→Mycelium transpiler strategy, Draft)

- **DN-34 — Rust→Mycelium Transpiler Strategy (Self-Hosting Acceleration)**
  (`docs/notes/DN-34-Rust-to-Mycelium-Transpiler-Strategy.md`, **Draft/advisory**): captures how a
  **Rust→Mycelium transpiler** would do the **bulk** of the Mycelium self-hosting rewrite
  (stdlib-in-Mycelium long pole) — transpile the project's Rust crates to Mycelium surface,
  **flagging** (never guessing) the hard residue for manual refinement. Seeded from the maintainer's
  **py2rust** (AST-walk transpilation + a never-silent `CompatibilityAnalyzer`) and **py-rust-bridge**
  (Python↔Rust FFI/SFI bridge) projects, retargeted `syn` (Rust AST) → Mycelium and **reusing the
  MEM-4 ownership/borrow analysis** (`mycelium-mir-passes`) — Rust already encodes the ownership facts
  DN-32 wants. Records a construct-mapping sketch, the **flag-don't-guess** analyzer as the
  load-bearing G2 principle, and the phasing (isolated branch → incremental interop-bridged
  transpile-then-refine → differential verify → DN-27 component-repo decomposition). Feeds DN-26 /
  DN-27 / RFC-0028 / M-502. **Gated** on the Mycelium surface being a viable target (post-core-1.0) —
  **enacts nothing, ships no code, begins no phase.** All effort/coverage claims `Declared`;
  seed-architecture `Empirical`, transfer `Declared`.

### Added (2026-06-25: MEM-4 Q5 measurement corpus — Increment 1 gate cleared)

- **`mycelium-mir-passes::corpus`** — the DN-33 §8.1 **Q5** measurement: a representative, **mixed**
  corpus of Core IR terms (elision-friendly reader-heavy `let`s + elision-neutral escaping/single-use
  terms) and a `measure()` that lowers each term both ways (`emit_owned` vs `emit_elided`), counts
  `Dup`s, and runs the differential to confirm semantics-preservation. **Result: 22 → 2 `Dup`s
  (20 removed, ~91% reduction), all terms semantics-preserved.** This clears the Q5 gate
  (Increment 1's effect is measured), **unblocking Increment 2**.
  - Honest tags: the `Dup` **count / ratio** is **`Exact`** (read off the IR — an exact measurement
    *of this corpus*); the corpus being **representative** of real Mycelium programs is **`Declared`**
    (no program population to sample yet — the corpus is a deliberate mix so the ratio isn't inflated);
    the runtime **performance** interpretation stays **`Declared`** (no Mycelium runtime benchmark
    yet — the gate is "the analysis does real work", not a perf SLO). 35 tests (4 new, incl. the Q5
    gate assertion + the honest-mix + per-term-monotonicity checks); clippy `-D warnings` clean.

### Added (2026-06-25: documentation wiki + paired API-docs publishing)

- **GitHub wiki source** (`docs/wiki/`): a browsable wiki generated from the unified per-crate
  READMEs + the design corpus — `Home`, `Architecture`, `Crate-Index` (all 50 crates, regenerated
  from the READMEs), `Memory-Model` (DN-32 / RFC-0027 / DN-33), `Tunable-Certification` (RFC-0034 /
  ADR-032), `Getting-Started`, `Decision-Records`, `API-Reference`, plus `_Sidebar`. Kept in-repo
  (versioned, reviewable) rather than authored directly in the wiki repo.
- **`publish-docs` GitHub Action** (`.github/workflows/publish-docs.yml`, **manual-dispatch only** —
  consistent with the repo's advisory-CI policy): mirrors `docs/wiki/*.md` to the GitHub wiki and
  builds + deploys the **rustdoc** API docs to GitHub Pages (both via the Actions `GITHUB_TOKEN`;
  requires the Wiki feature + Pages enabled in Settings). rustdoc verified to build clean
  (`cargo doc --no-deps --workspace`).
- **README** gains a **Documentation** section pointing to the wiki, the rustdoc/`just docs`, the
  committed `docs/api-index/INDEX.md`, the per-crate READMEs, and the design corpus. Markdownlint +
  codespell clean.

### Added (2026-06-25: MEM-4 Increment 1 — non-escaping borrow elision + the Q3 differential harness)

- **MEM-4 Increment 1 — borrow elision** (`mycelium-mir-passes`, DN-33 Accepted §8.1): the pass that
  actually **removes** RC ops. In the immutable value model a reader primitive (`Op`/`Swap`) reads its
  operands and produces a fresh result, retaining nothing — so an operand position is a **borrow**
  (non-consuming read), not a move.
  - **`emit::emit_elided`** — a `let` binding whose every use is such a read (**fully borrowable** —
    `emit::is_fully_borrowable`, conservative: any escaping use to the result / a `Construct` / an
    `App`/`Match` keeps it owned) is emitted with its uses as `RcNode::Borrow` (non-consuming), **no**
    `Dup`, and a single `RcNode::DropAfter` reclaiming it **after** its reads. Strictly fewer RC ops
    than B0's owned emission (`k-1` `Dup`s → `0`). Intraprocedural; `Lam` params stay `Owned`
    (interprocedural borrowing is a later increment). New IR nodes: `Borrow` (read) and `DropAfter`
    (reclaim-after-body — the correct drop placement so a borrowed value stays live through its reads).
  - **`eval` — the reference RC-evaluator + `eval::differential`** (the differential half of the
    ratified Q3 soundness strategy): an abstract RC machine over the straight-line fragment that runs
    a term's owned **and** elided emissions and checks they reclaim the **same multiset of values**
    with **no use-after-free / double-free**, while the `Dup` count strictly drops. Control-flow
    nodes outside the straight-line fragment are refused explicitly (`RcError::UnsupportedNode`, G2).
  - Honest tags: the elision's **semantics-preservation** is **`Empirical`** (differential trials over
    a corpus, backed by the structural `DropAfter`-after-reads + balance argument), **not `Proven`**
    (a mechanized proof is the Phase-3 option — VR-5); the `Dup`-count reduction is **`Exact`** (read
    off the IR), the *performance* benefit stays `Declared` until measured (Q5). 31 tests (10 new:
    borrow classification, elided shape, the differential over a corpus, dup-reduction, and evaluator
    use-after-free / double-free / unsupported-node witnesses); clippy `-D warnings` clean;
    `#![forbid(unsafe_code)]`. Core IR still untouched (KC-3).

### Added (2026-06-25: MEM-4·B0 — the RC-emission pipeline foundation (`mycelium-mir-passes`))

- **`crates/mycelium-mir-passes`** — the first MEM-4 code (DN-33 Accepted §8.1), building the
  RC-emission pipeline the §6.1 investigation found missing (nothing emitted RC ops, so there was
  nothing to elide). **Optimisation-only and OUTSIDE the trusted Core IR** (KC-3 / DN-33 §8.1 Q2): it
  consumes `mycelium_core::Node` read-only and produces a **separate** RC-annotated IR — the kernel
  `mycelium-core` is **untouched**, and a bug here is a missed optimisation, never unsafety (the
  runtime `RcCell` probe stays the sound fallback).
  - **`rc_ir` — the RC-annotated IR** (`RcNode`): mirrors the Core IR first-order fragment
    (`Const/Var/Let/Op/Swap/Construct/Match/Lam/App`) plus `Dup`/`Drop` wrapper nodes and a
    per-binding own/borrow `Mode` (the `Borrowed` variant is the forward hook for Increment 1).
  - **`emit` — naive fully-owned RC-emission** `Node → RcNode`: a binding used `k` times gets `k-1`
    `Dup`s (one reference per use) and each use consumes one; an unused binding gets one `Drop`.
    Occurrence counting is **shadowing-aware** (rubric A4-01). Recursion (`Fix`/`FixGroup`) is
    **refused explicitly** (`EmitError::UnsupportedNode`) — never silently mis-emitted (G2).
  - **`balance` — the structural balance invariant** (`1 + dups == uses + drops` per owned binding;
    a `Borrowed` binding must carry no `Dup`/`Drop`), verified **independently** over the emitted IR
    (re-derives the counts, so a buggy emission is caught — mutation-tested). This is the
    structural-invariant half of the ratified Q3 soundness strategy; the differential half lands with
    Increment 1.
  - Honest tags: the balance property is **`Exact`** by construction (independently checked); **no
    perf claim** — B0 deliberately emits the *most* RC ops (the `dup`/`drop` count is `Exact`, read
    off the IR; any reduction figure stays `Declared` until measured — DN-33 §8.1 Q5). 21 tests
    (emission shape, shadowing, recursion-refusal, mutation witnesses for unbalanced/over-released
    IR); clippy `-D warnings` clean; `#![forbid(unsafe_code)]`.
  - **Next:** MEM-4 Increment 1 — non-escaping borrow elision (mark non-consuming reads `Borrowed`,
    eliding their `Dup`/`Drop`) + the Q3 differential harness (with/without elision → identical
    results AND reclamation records).

### Changed (2026-06-25: DN-33 ratified Draft → Accepted — §8 deliberation settled)

- **DN-33 ratified Draft → Accepted (§8.1 resolutions, maintainer).** The MEM-4 design deliberation is
  settled: **Q1 → Option A** (only sole ownership crosses a hypha boundary; `RcCell<T>` stays `!Send`;
  Option B / atomic-RC sharing gated to R2), **Q2 → separate RC-annotated IR** (the trusted Core IR
  `mycelium-core/src/node.rs` stays pristine — KC-3 / DN-33 §4), **Q3 → differential + structural-
  invariant** soundness (tag `Empirical`, not `Proven`). Q4–Q7 adopted as defaults (subsume `substrate`
  uniqueness; perf gate = measured `dup`/`drop`-reduction ratio — count `Exact`, perf `Declared`; FIP
  user-surface deferred to Phase 3; frame-limited R1 target). Status moves Draft → Accepted (the design
  is ratified; **enacts no code**). The E12 build plan now sequences the MEM-4 build (B0: RC-emission
  pipeline foundation → Increment 1: borrow elision + differential harness). Does not move RFC-0027's
  status (the cross-hypha Option A feeds a later RFC-0027 follow-on). Append-only.

### Changed (2026-06-25: DN-33 §6.1 addendum — MEM-4 is blocked-by-prerequisite, not just deferred)

- **DN-33 §6.1 (append-only addendum)** records a grounded investigation finding: **MEM-4 Increment 1
  has no input to operate on** — the Core IR (`mycelium-core/src/node.rs`) carries no ownership-mode
  field on binding sites, there is no RC-annotated IR / `crates/mycelium-mir-passes/`, and
  `clone_ref`/`drop_ref` are hand-called only in `mycelium-std-runtime` tests (no lowering emits RC ops
  to elide). The prerequisite chain (resolve DN-33 §8 Q2 ownership-mode representation → add the field
  to `node.rs` → build the `mir-passes` RC-emission lowering → wire into `elab.rs` → then Increment 1)
  is a forward language-frontend epic **gated on the §8 Q2 maintainer decision** — not built
  speculatively (G2/VR-5). The E12 build plan's MEM-4 status is updated to **blocked-by-prerequisite**.
  The runtime substrate (MEM-1..3 + live triggers, landed) is the sound, complete fallback. Also fixes
  a stray `</content>` artifact at the end of DN-33. No status moves; no normative text changes.

### Added (2026-06-25: DN-33 — MEM-4 static uniqueness analysis design, research-backed, Draft)

- **DN-33 — Layer-1 Static Uniqueness Analysis (MEM-4) & Cross-Hypha Reconciliation**
  (`docs/notes/DN-33-Layer1-Static-Uniqueness-Analysis.md`, **Draft/advisory**): the research-backed
  design direction for the deferred MEM-4 leg of DN-32, authored design-first before implementation.
  Records (1) **MEM-4 = an additive, semantics-preserving compiler lowering pass** that elides
  provably-redundant RC ops, with the runtime `RcCell` probe (MEM-2) as the **sound fallback** — so
  the analysis is **sound-but-may-be-incomplete** (a bug costs throughput, never memory safety); (2)
  how LR-8 immutability + LR-9 acyclicity + content-addressing shrink the problem below Rust-style
  borrow checking; (3) KC-3 disciplines (lowering-pass-not-type-checker, additive-only, watch+measure
  — DN-32 §6b); (4) an incremental decomposition (non-escaping borrow elision → `rc==1` reuse
  annotation → full FIP static guarantee); and (5) a **recommendation for the cross-hypha
  sub-question** (DN-32 §7 / RFC-0027 §12) — **Option A** (sole-move-only / affine-channel boundary;
  `RcCell` stays `!Send`) for R1, **Option B** (shared-crosses-atomic-RC) deferred to R2. All
  Mycelium-specific claims `Declared`; external prior art (Perceus, Lorenzen, Koka FP², ASAP, Pony,
  Rust, Verona) `Empirical`; the cross-hypha recommendation is an argument with its ergonomic cost as
  the named open risk. **Enacts nothing; moves no status; changes no normative text** — promotion past
  Draft requires the §8 deliberation + maintainer ratification (house rule #3, append-only).

### Added (2026-06-25: E12 Wave-4 — ChannelClose trigger + live scope/region wiring + L1/L2/L3 composition)

- **ChannelClose — the third (and final) live reclamation trigger**
  (`crates/mycelium-std-runtime/src/network.rs`, RFC-0027 §9 / §7.3): tearing down a channel that
  still holds **in-transit** values (sent, never to be received) reclaims them —
  `Receiver::close_with_reclaim(sink, scope_id, sweep_epoch, hash_of)` drains the buffer FIFO and
  emits one `ReclamationRecord(ChannelClose)` per value, returning the reclaimed count. Never silent
  (G2): an undelivered value is reclaimed-and-recorded, never dropped. Normal drain (receiver gets
  the values) needs no reclamation — the teardown path is the distinct case.
  - **Canonical `ChannelNodeId` (resolves MEM-1's last placeholder):** a monotonic per-channel id
    (mirrors `region::ScopeNodeId`), bridged to the `ReclamationRecord` `ChannelId` field via
    `as_channel_id()`; allocated once per `Network::channel` and exposed on both `Sender`/`Receiver`.
    With this, all three of MEM-1's `u64` placeholders (`ScopeId`/`SweepEpoch`/`ChannelId`) are
    canonicalized.
- **Live-executor scope/region wiring** (`crates/mycelium-std-runtime/src/scope_region.rs`): the
  Layer-3 `ScopeExit` reclamation now fires from a **running structured scope**, not just the bare
  data structure. `with_region(sink, body)` (closure form — `close` is always called after `body`,
  `Exact` by construction) and `RegionScope` (explicit-close guard for interleaved deferrals) tie a
  `Region`'s lifecycle to a single-hypha scope; **nested `with_region` gives child-before-parent
  epoch order for free** (monotonic counter). No `Drop`-with-sink (KC-3 — a sink can't thread
  through `Drop`; mirrors `rc.rs`); a dropped guard with pending entries hits the `Region` debug
  panic (G2 — silent audit loss impossible in debug). Cross-hypha atomic-RC stays FLAGged
  (DN-32 §7 / RFC-0027 §12 — see DN-33).
- **End-to-end L1/L2/L3 composition test** (`crates/mycelium-std-runtime/src/tests/composition.rs`):
  one `CollectingSink` observes a single scope in which `RcZero` (L2 — last-handle drop), the
  `ChannelClose` channel teardown, and the batched `ScopeExit` (L3 — scope close) **interleave and
  compose**, with never-silent accounting (every reclamation event yields exactly one record;
  2 `RcZero` + 3 `ChannelClose` + 2 `ScopeExit` = 7) and child-before-parent epoch order across a
  nested scope. This validates the three triggers compose through one audit trail (RFC-0027 §9).
  - Tags honest: probe/close/ordering logic `Exact` (enforced-by-construction); batching-as-perf
    `Declared` (DN-32 §6a — no measurement). 104/104 tests (20 new across the wave); clippy
    `-D warnings` clean; `#![forbid(unsafe_code)]`.
  - **Phase-1 three-layer memory model now feature-complete at the runtime tier:** all three live
    triggers wired (RcZero · ScopeExit · ChannelClose), all placeholder ID types canonicalized,
    scope-exit reclamation fires from a live scope. Remaining: **MEM-4** Layer-1 *static* uniqueness
    analysis (Perceus-style RC elision) — design-first as **DN-33** (research-backed), the runtime
    `RcCell` probe is the sound fallback until it lands.

### Added (2026-06-25: E12 Wave-3 — MEM-3 regions + batched scope reclamation (DN-32 Layer 3))

- **MEM-3 — region-based batched scope reclamation** (`crates/mycelium-std-runtime/src/region.rs`, DN-32
  Layer 3 / RFC-0027 §10.3): `Region` = one RT7 scope-tree node; `Region::defer(value_meta_hash)` accrues
  reclamations during the scope, and `Region::close(&sink)` drains them in bulk, emitting one
  `ReclamationRecord(ScopeExit)` per value — **the second live reclamation trigger** (after MEM-2's `RcZero`).
  Bulk emission is G2-enforced (a Region dropped with deferred entries un-closed panics in debug). `ScopeTree`
  closes children before the parent.
  - **Canonical ID types (resolves MEM-1's placeholders):** `ScopeNodeId` + `RegionEpoch` (monotonic counters)
    replace the `u64` `ScopeId`/`SweepEpoch` placeholders, threaded through reclamation.rs/rc.rs; the epoch
    number-line encodes the child→root sweep order (child epoch < parent epoch always).
  - **OQ-1 (weak coupling) realized + tested:** parent-child reclamation total (child-before-parent, `Exact`
    by the monotonic counter), siblings order-independent (`Proven`-modulo-LR-9) — property tests both ways.
  - Tags honest: ordering logic `Exact`/by-construction; bulk-efficiency-as-perf `Declared` (DN-32 §6a). 84/84.
  - Downstream-flagged: live-executor `Region::close` wiring (the running scheduler/MLIR runtime), cross-hypha
    atomic RC (§7), strong-coupling opt-in, `ChannelId` canonicalization (network tier).

### Added (2026-06-25: E12 Wave-2 — MEM-2 explicit RC + rc==1 reuse (DN-32 Layer 2))

- **MEM-2 — explicit reference counting + `rc==1` reuse probe** (`crates/mycelium-std-runtime/src/rc.rs`,
  DN-32 Layer 2 / RFC-0027 §10): `RcCell<T>` wraps `std::rc::Rc<T>` — **non-atomic, `!Send + !Sync`**
  (intra-hypha fast path), immutable-value-only (LR-8), no `unsafe` (respects `#![forbid(unsafe_code)]`).
  `drop_ref(sink, …) -> RcProbe<T>` probes the strong count **before** decrement: `count==1` →
  `UniqueOwner(T)` — emits exactly one `ReclamationRecord(RcZero)` through MEM-1's `ReclamationSink` (**the
  first live reclamation trigger wired**) and returns the owned `T` (FBIP reuse, §10.2); `count>1` → `Shared`
  (decrement, no record). Probe logic tagged **`Exact`** (enforced by construction); FBIP reuse-as-perf-win
  **`Declared`** (no measurement, DN-32 §6a). 64/64 tests (12 new). Downstream-flagged: the **atomic
  cross-hypha** upgrade (the DN-32 §7 reconciliation sub-question), **region** deferred-drop accumulation
  (MEM-3), and **static RC elision** (MEM-4 / Perceus uniqueness analysis).

### Added (2026-06-25: E12 memory-model build — Wave 1 — MEM-1 + CI blocker removal)

- **E12 build plan** (`docs/planning/E12-Memory-Model-Build-Plan.md`) — the DN-32/RFC-0027 implementation
  roadmap, decomposed into tightly-scoped Sonnet-swarm waves (BLK blockers; MEM-1 record → MEM-2 RC →
  MEM-3 regions → MEM-4 static analysis).
- **MEM-1 — reclamation EXPLAIN/audit record** (`crates/mycelium-std-runtime/src/reclamation.rs`, RFC-0027 §9):
  `ReclamationRecord{ scope_id, sweep_epoch, trigger ∈ {RcZero, ScopeExit, ChannelClose}, value_meta_hash,
  channel_id? }` + the exhaustive `ReclamationTrigger` enum + the **`ReclamationSink` never-silent contract**
  (a proptest proves every reclamation event emits exactly one record — silent reclamation is structurally
  impossible) + EXPLAIN integration (RFC-0005). The observability FOUNDATION of the memory model. Placed in the
  runtime tier (not `mycelium-core` — KC-3). Tagged **`Declared`**: the record structure is concrete; the live
  trigger-wiring (rc→0 / scope-exit / channel-close) is downstream (MEM-2/MEM-3), `ScopeId`/`ChannelId`/
  `SweepEpoch` are `u64` placeholders pending the canonical types. 52/52 tests green.
- **CI blocker removal (BLK):** the `api` gate is green (regenerated the drifted `l1`/`lsp`/`spore`/`std-sys`
  baselines + new `cli`/`std-sys-host`) and the `myc-fmt` gate is green (`mycfmt`-canonicalized `lib/std/{cmp,
  option}.myc`) — formatting/baseline-only, no logic change. agent index regenerated.

### Added (2026-06-25: DN-32 three-layer memory architecture; RFC-0027 → Accepted (ratified))

- **DN-32 — Three-Layer Hybrid Memory Architecture** (`docs/notes/`, **Accepted**, ratified by maintainer):
  affine/linear ownership PRIMARY (unique data, ~zero cost) → optimized RC for EXPLICIT sharing
  (non-atomic intra-hypha, `rc==1` reuse) → region-based alloc/reclamation in scopes (batched at scope-exit).
  Parent-child reclamation total; siblings concurrent-by-default (weak coupling), strong opt-in. Grounded in
  Perceus / Smith-structured-concurrency / Lorenzen + the landed research. Carries four honest caveats (§6):
  perf claims are `Declared` GOALS; Layer-2 static uniqueness analysis is the hard leg + a KC-3 tension; OQ-1
  resolved by argument (not prototype); the cross-hypha RC-vs-affine reconciliation sub-question is named.
- **RFC-0027 advanced Proposed → Accepted (ratified by maintainer 2026-06-25):** DN-32 **resolves OQ-1**
  (weak/partial sibling coupling default — safe by RC + LR-9 acyclicity, RT7 siblings already concurrent;
  throughput benefit `Declared`) and **OQ-4** (`rc==1` reuse EXPLAIN-record-only by default); **mitigates OQ-3**
  (regions + batching; SLO stays `Declared`). OQ-2/5/6 deferred non-blockers. New §12 points to DN-32 + names
  the cross-hypha sub-question. Append-only (prior Draft→Proposed history preserved). The memory model is now
  decided end-to-end (design); implementation follows (E12-1).
- Registered DN-32 in `docs/Doc-Index.md`; refreshed the stale RFC-0027 index row.

### Changed (2026-06-24: RFC-0027 advanced Draft → Proposed — reclamation mechanism resolved)

- **RFC-0027 (Memory Management & Reclamation) advanced Draft → Proposed** (append-only: §§1–6 preserved;
  resolved design additive in §§7–11), incorporating the landed research cluster. **Awaiting maintainer
  ratification of the Draft → Proposed move** (banner in the RFC; does not skip to Accepted, house rule #3).
  - **§7 mechanism = REFERENCE COUNTING, not tracing GC** — justified by LR-9 acyclicity being *exactly*
    Perceus's garbage-free precondition (no cycle detector needed). Scoped single-owner intra-hypha;
    cross-hypha rides the affine channel protocol.
  - **§8 honest tags (VR-5):** RC-soundness `Proven`-**modulo the LR-9 side-condition** (external theorem, no
    in-repo mechanized check yet — *not* bare `Proven`); the ~32K-LOC embeddenator confirmation `Empirical`,
    **tempered** by the ground-truth correction (embeddenator actually chose OCC with inert refcount — the
    transfer is not 1:1); all Mycelium wiring `Declared`.
  - **§9 reclamation EXPLAIN/audit record** (never-silent G2): `scope_id`, `sweep_epoch`,
    `trigger ∈ {RcZero, ScopeExit, ChannelClose}`, `value_meta_hash`, optional `channel_id`.
  - **§10 copy/mut + reclamation unify** via the `rc==1` FBIP reuse probe (free / mutate-in-place /
    structural-share); `fuse` is **structurally** unified (δ-CRDT anti-entropy over the Provenance DAG) but
    **algebraically separate** (semilattice-join, independent of refcounting) — no overclaim.
  - **§11 OQ-1 (sweep-order vs reclamation-order, partial vs total) left OPEN** — the explicit reason status
    stops at Proposed; lane-B recommends prototyping both before committing.

### Changed (2026-06-24: maintainer ratifications — RFC-0034 advance + E21-1 design flags)

- **RFC-0034 advance RATIFIED.** The `Enacted (design-driven) → Enacted — with code (Rust-first)` advance
  (M-794, the §13 conformance gate) is ratified by the maintainer — the "pending ratification" qualifiers are
  cleared in RFC-0034 (Status + changelog row) and ADR-032 (decision 1 discharged). RFC-0034 is now fully
  **Enacted — with code**.
- **ADR-032 decision 5 also realized + noted:** the per-use unsafe-lint enforcement landed in M-793 (the
  `unsafe-per-use` gate) — ADR-032's M-794 update note is corrected (it had still listed decision 5 as the open
  residual). Remaining ADR-032/RFC-0034 residual: §14 per-op/per-knob granularity (deferred, named-not-silent).
- **E21-1 design flags ratified as-built** (maintainer): M-790 `@certification` surface spelling (lowercase
  `fast|balanced|certified`) + the v0 single-manifest `phylum`-tier modeling (`Global` reserved for
  multi-manifest); M-794's conformance suite using `mycelium-cert` dev-deps + **local duplication** of the
  harness shapes (not a shared `mycelium-test-support` crate); M-796's `ModeScope` `pub [bool;3]` field +
  granular-override `source: None`. All stand as implemented.

### Added (2026-06-24: E21-1 Group-B Wave-3 — M-794 conformance gate + M-796 toolkit; **CLOSES E21-1**)

- **M-794 — the §13 conformance gate (E21-1 capstone)** (`crates/mycelium-cert/tests/conformance.rs`, 19
  tests). Asserts **all six RFC-0034 §13 clauses (a)–(f)** end-to-end, EACH parameterized over
  `fast`/`balanced`/`certified` + the cross-mode NEGATIVE cases (the M-795 `assert_mode_scope` pattern —
  invariant present where it fires, absent/relaxed where it must not). Memory-safety clause (c) is `Proven`
  **by a checked side-condition** (the suite reads the trusted base's `#![forbid(unsafe_code)]`), not by fiat.
  **⚠️ Advances RFC-0034 `Enacted (design-driven)` → `Enacted — with code (Rust-first)` + realizes ADR-032
  decision 1 — append-only, PENDING MAINTAINER RATIFICATION of the advance** (the capstone milestone, flagged
  not routine). Residual deferred (named-not-silent): §14 per-op/per-knob granularity.
- **M-796 — native scoped mode-parametric testing toolkit** (`mycelium-std-testing`): `ModeScope` +
  `ModeTestConfig` (wiring M-790's `@certification` resolver — project>phylum>nodule, shared not parallel) +
  `assert_mode_scope` + `for_each_mode_in` (returns visited/**skipped**, never-silent) + a zero-boilerplate
  worked example; re-exports `CertDecl`/`CertScope`. Downstream devs get per-tier + cross-mode-negative
  coverage for free. Followed M-797 (extracted a 605-line inline block).
- **E21-1 is functionally complete:** the full tunable-certification mechanism (M-786…M-796) is landed
  Rust-first with the §13 conformance gate green. Statuses: `issues.yaml` M-794/796 → `done`.

### Added (2026-06-24: E21-1 Group-B Wave-2 — M-792/M-793/M-795)

- **M-792** (`mycelium-proj`) — EXPLAIN-of-mode + the **generation≠consumption split** (RFC-0034 §7/§3.1/§13d):
  `ModeSignal` (the always-generated, mode-independent inspectability record — cheap, no heap, present even in
  `fast`) + `ConsumptionTier {Lean,Medium,Full}` + `render_mode_signal` (dial verbosity up from *already-captured*
  history — no re-run, no mode switch). Lean output is contractually identical to `explain_mode` (the §13d floor).
- **M-793** (`scripts/checks/unsafe-per-use.sh` + justfile + `mycelium-mlir`) — sharpens ADR-014 to an **explicit
  per-use unsafe escape** (RFC-0034 §9): a new never-skip gate asserts (a) the 4 trusted-kernel crates retain
  `#![forbid(unsafe_code)]` and (b) every `unsafe` site carries a *per-use* `#[allow(unsafe_code)]` (no crate-global
  allows). One `mlir/jit.rs` site annotated; ADR-014's `// SAFETY:` + dev-warning discipline preserved (sharpened,
  not superseded).
- **M-795** (`mycelium-core`) — the shared **mode-parametric test harness** (`for_each_mode`, canonical per-strength
  `Bound` fixtures, and `assert_mode_scope` — the cross-mode NEGATIVE pattern as a first-class primitive: asserts an
  invariant fires in its in-scope tiers AND is absent in out-of-scope tiers). `#[cfg(test)]`-only (no public API,
  KC-3); 14 mode tests; `cert_mode.rs` adapted to use it. Broad cross-crate fixture-adaptation deferred to M-794.
- Statuses: `issues.yaml` M-792/793/795 → `done`. proj baseline + agent index regenerated (deterministic).

### Added (2026-06-24: E21-1 Group-B Wave-1 — M-789/M-790/M-791 + M-788 ratified)

- **M-788 bound/basis decision RATIFIED** (maintainer, 2026-06-24): when `fast` floors a computed
  `Proven`/`Empirical` result to `Declared`, keep the computed bound *value* and relabel its basis to
  `UserDeclared` (`CertMode::gate_result`) is the settled approach; the `BoundBasis::ModeFloored`
  refinement is set aside.
- **M-789** (`mycelium-spore`, `mycelium-mlir`) — spore identity + MLIR hot-inject/ABI dispatch keys are
  **`Proven`-by-construction independent of `CertMode`** (content_address/dispatch-key hash only
  code+deps+surface+kind; `CertMode` rides `Meta`, excluded per RFC-0001 §4.6 / ADR-003), exhaustively
  tested over `CertMode::ALL` — a spore is mintable + content-addressed in `fast` (RFC-0034 §8). The
  embedded/no-deploy compile-hash *disable* path is flagged as an open gap (`Declared` + TODO; never-silent, G2).
- **M-790** (`mycelium-proj`) — `@certification` resolution + scoping, most-specific-wins `global > phylum >
  nodule` (RFC-0034 §6) **reusing the RFC-0012 scoped-override mechanism** (no new scoping machinery);
  `resolve_mode` order-independent, defaults `Fast` with **named provenance (never ambient)**; cross-mode
  `compose` floors by the **producer's** mode (never a silent upgrade, VR-5/§3.1). **Resolves M-788's FLAG-2**
  (the deferred mode source). Two maintainer flags pending: surface spelling (lowercase `fast|balanced|certified`),
  and v0 single-manifest modeled at the `phylum` tier with `Global` reserved for multi-manifest/workspace.
- **M-791** (`mycelium-core`) — named, explicit `WrappingOpt` Axis-B opt-out marker on `Meta` (RFC-0034 §10);
  Axis-B never-silent failability stays default-on in every mode; the wraparound *op-layer wiring* is downstream.
- **M-797 (as-touched):** all three leaves extracted their inline `#[cfg(test)]` tests to in-crate `src/tests/`.
- Statuses: `issues.yaml` M-786..M-791 → `done`. api baselines (core/proj) + agent index regenerated.

### Added (2026-06-24: M-788 — mode-gated swap-cert emission/checking + bound/basis reconciliation)

- **`CertMode::gate_result(intended_guarantee, intended_bound) -> (GuaranteeStrength, Option<Bound>)`**
  (`mycelium-core`, RFC-0034 §7) — the bound/basis half M-787 explicitly deferred. When `Fast` floors a
  would-be `Proven`/`Empirical` result to `Declared`, it **keeps the computed bound value but relabels its
  basis to `UserDeclared`** ("computed, asserted-not-verified in fast") — the only basis M-I4 admits for
  `Declared`, and the honest tag (VR-5: the ε/δ was computed, but `Fast` ran no machinery to *certify* it).
  This keeps M-I1…M-I4 consistent by construction; the gated pair is exhaustively tested to be
  `Meta::new`-constructible across `CertMode::ALL × {Exact,Proven,Empirical,Declared}`.
  **Pending maintainer ratification** (the candidate-(a) "keep value, relabel basis" resolution — see
  `docs/handoffs/m788-context.md`).
- **Mode-gated swap certificates** (`mycelium-cert::mode`, RFC-0034 §4/§5): `gate_swap`, `GatedSwap`
  (value + optional certificate + optional check verdict — inspectable, no black box), and
  `ModeGatedSwapEngine` (a `SwapEngine`, default `Fast`). `Fast` runs the cert machinery **not at all**
  (no emit, no check; Meta reconciled via `gate_result`); `Balanced` **emits** without checking; `Certified`
  **emits + checks** through the unchanged M-210 `check` (a non-validating verdict is surfaced
  never-silently). **Axis-B is not gated** — out-of-range / illegal-pair stays an explicit error in every
  mode. The certificate machinery itself is unchanged. Mode-parametric tests (RFC-0034 §13) across the
  three tiers + cross-mode negatives. *Implemented (Rust-first), pending RFC-0034 ratification.*
- **Test-layout (M-797 as-touched):** `mycelium-core`'s `cert_mode` + crate-root `WfError` inline tests
  extracted to in-crate `src/tests/` (white-box) as part of touching those logic files.

### Changed (2026-06-24: DN-31 §4-Q1 resolved — empty `{}` = block, `{:}` = empty map)

- **Empty-`{}` ambiguity resolved (maintainer):** in the DN-31 delimiter scheme, **`{}` is an empty block**
  and **`{:}` is an empty map** (the colon is the same "map" marker, just with no entries). Non-empty cases
  were never ambiguous (`{ k: v }` maps already split from `{ e }` blocks on the `:` pairs); only the empty
  case needed a rule, and `{:}` does it minimally — no per-literal map tagging, JS block-vs-object trap
  avoided. Closes DN-31 §4-Q1 (the sharpest open question); §4-Q2 (list-at-statement-start) remains open.
  This **unblocks the bracket-implementation work**. Recorded append-only in DN-31 + Doc-Index.

### Added (2026-06-24: RFC-0035 — Security Scanning Toolkit (binding design, Proposed))

- **RFC-0035** (`docs/rfcs/`, **Proposed**). The binding security-scanning design — a native, inherited,
  scope-configurable security toolkit — lifted from the settled **DN-30** direction capture plus the
  maintainer's now-answered answers to DN-30 §7's five open questions (tabled as Decisions D1–D5, §10):
  - **D1 (§2)** — v0 vulnerability classes are a **fixed base of categories WITH an extensibility seam**
    (an extension class is first-class + versioned, never silently folded into the base); base grounded in
    Mycelium's own surfaces (RFC-0028/ADR-014 `unsafe`/FFI, the `/security-review` recurring-defect bank).
  - **D2 (§3)** — reporting is **SARIF + CWE + OSV + VEX** with **versioned pinning**: a pinned schema
    version is **immutable once pinned**; new versions are allowed (additive-by-new-version, append-only),
    the finding schema content-addressed (RFC-0001 §4.6) so the pin is mechanically enforced.
  - **§4** — find-once-report-to-**two-sinks** (CLI + registry); the registry hosts **screened/anonymized**
    advisories as a **second content-addressed catalog** reusing **DN-28** reconstruction-on-render
    (lightweight, tamper-evident — a poisoned advisory fails its hash, G2).
  - **D3/D5 (§5)** — honest, **RFC-0002-certificate-backed safe auto-fix** (proves the fix eliminates the
    vuln AND refines the original modulo it); **per-class fix-strength + a pedantic mode**; a **certified
    patch registry**; `/security-review` is a **supporting tool only** (not a replacement/prerequisite).
    `Declared` fixes are always flagged + human-gated — no black-box rewrites (VR-5/G2).
  - **D4 (§6)** — the **screening policy** is configurable-with-defaults, **mandatory for high-security
    classes by default** (cannot be silently disabled for them), per-project adjustable (every adjustment
    surfaced).
  - **§7** — native + scoped reusing the **RFC-0034 §6** `@certification` resolution (project/phylum/nodule/
    granular) — no new scoping machinery.
  - **Designs the toolkit; implements nothing** — every runtime claim is a `Declared` position for epic
    **E22-1** to discharge; the two **worked examples** (a safe-fix refinement-certificate; a screening
    case study) are the deferred pre-Accepted work and were **not fabricated** (§9, VR-5/G2).
  - **DN-30** gains an append-only rev. 3 note ("feeds RFC-0035"); registered in `docs/Doc-Index.md`;
    working notes in `docs/handoffs/security-rfc-context.md`. `doc_refs` check passes.

### Added (2026-06-24: DN-19 GAP-2 — Medium-findings verification ledger (draft, Gate A2))

- **DN-19 GAP-2 Medium-findings ledger** appended (append-only subsection, `docs/notes/DN-19-Road-to-1.0.0.md`,
  **draft**). Re-grounds ADR-021 **Gate A2** against the live tree (`origin/main` `db4a6be`): all **25 open
  Medium finding-ids** (WS2–WS6) re-located by their cited test/variant names, a representative subset
  **executed green** (A6-03, A3-09, A5-05/06, A5-03). Tally **25 FIXED · 0 DEFERRED · 0 N-A**, with one
  **citation flag** (A5-08's prior citation points at `mycelium-dense` but the fix lives in
  `mycelium-select::packing_bits_per_element` + `mycelium-mlir/pack.rs`). **Draft — pending maintainer
  sign-off** (Gate A2 ratification is the maintainer's, ADR-021 §6). Full per-finding evidence archived in
  `docs/handoffs/gap-2-ledger-context.md`. Decides nothing normatively (VR-5/G2).

### Added (2026-06-24: DN-31 — Delimiter & Operator Deconfliction (direction capture, Draft))

- **DN-31** (`docs/notes/`, **Draft**, advisory). Captures the maintainer's decided reallocation of the four
  bracket families into one collision-free scheme, grounded in a grep of the normative grammar showing **`<>`
  triple-loaded** (type params + type args + trit literals + the wanted comparison/shift operators) while
  **`[]` is near-empty** (list literals only):
  - **`<>` → comparison/shift operators only** (`< > << >>`); `<=`/`>=` retire to word operators `lte`/`gte`.
  - **`[]` → type args + sized/repr types + list literals** (`List[T]`, `Binary[64]`, `[1,2,3]`), position-
    disambiguated (clean because indexing is `get()` not `arr[i]`, calls are `()`).
  - **`()`** calls/grouping/tuples/ctors · **`{}`** blocks/effects/match/**maps**.
  - return arrow **`->` → `=>`** (bare `= - >` remain operators); trit literals **`<+-0>` → `0t+-0`** (types
    `trit[N]`/`tryte[9]`, extends to `byte[N]`).
  - **Resolves M-745 by reallocation** (no speculative parsing); proposes to **supersede RFC-0019 §4.1**
    (Enacted) + update RFC-0030/0025/0001/0033.
  - **Open questions flagged on merit:** empty `{}` block-vs-map ambiguity (sharpest — undecided), list-literal
    at statement start, and the `<=`/`>=`→words asymmetry. Enacts nothing (VR-5/G2); feeds the binding RFC.
  Registered in `docs/Doc-Index.md`.

### Changed (2026-06-24: CLAUDE.md — test-layout rule (no tests in logic files); retrofit tracked as M-797)

- **New binding convention (maintainer-directed):** logic `.rs` files carry **no test code**. Every
  `#[cfg(test)]` unit test lives in an **in-crate** test module — `#[cfg(test)] mod tests;` in `lib.rs` →
  `src/tests/` (one submodule per source module), each `use crate::…::*` for **white-box** access to
  private items (precedent: `mycelium-std-recover/src/tests.rs`). Chosen over fully-external `tests/` on
  merit — that would lose private-internal coverage or force internals `pub`. **Complex test logic → fixtures
  + parameterization**, not test bodies. New tests follow it now; the ~185-file inline-test retrofit
  (mycelium-core alone is 18 files / ~5k test lines) is tracked as **M-797**, a per-crate octopus-merge
  swarm (no behaviour change — identical test counts pre/post).

### Added (2026-06-24: M-664 — `consume` + `grow` lexed (DN-03 §1); closes the not-yet-lexed lexicon gap)

- **`consume` + `grow` are now reserved surface keywords** (`crates/mycelium-l1`). DN-03 §1 ratified
  them but they were the only two **NOT-YET-LEXED** terms (lexicon survey, 2026-06-24) — silent
  identifiers, a G2 hazard. Now lexed (`Tok::Consume`/`Tok::Grow` + `keyword()`), with a teaching
  diagnostic at item + expression position naming **DN-03 §1** ("reserved surface keyword … not yet
  active — its construct lands with M-664"); never a silent accept. Their parser constructs
  (`consume <expr>`, `grow Trait for Type { … }`) are the follow-on surface step.
- **The reservation immediately caught a real silent identifier:** the AOT differential test fragment
  defined `fn grow` (a `shrink`/`grow` mutual recursion) — renamed to `expand` (exactly the G2 case the
  reservation exists to surface).
- Tests: unit `surface_keywords_consume_grow_are_reserved_not_active` (item/expr/binder positions) +
  conformance reject fixtures `18-consume-…` / `19-grow-reserved-not-active.myc` (self-policing
  `REJECT_EXPECTED`). Lexicon status: **37 active-or-reserved**, **0 not-yet-lexed** (the 9 runtime-tier
  terms stay correctly reserved-not-active, blocked on RFC-0008 R2 design, not lexing).
- Verified: gate clippy (`-D warnings -A unsafe_code`, ADR-014) green; `// SAFETY:` gate green;
  `cargo test -p mycelium-l1` green.

### Changed (2026-06-24: CLAUDE.md house rule 4 — merit-based agreement / anti-sycophancy safeguard)

- **House rule 4 ("Ground every claim") extended to bind *agreement*** — a standing agent safeguard
  (maintainer-directed). Agree only on merit, never to please; an affirmation is a claim, so tag its
  strength (checked/`Proven` vs plausible/`Empirical` vs asserted/`Declared`) and surface disconfirming
  evidence even when it cuts against the stated direction. Sycophancy is ranked with an ungrounded claim;
  the maintainer's explicit standing preference is *be corrected over being wrongly affirmed — follow the
  evidence, not the speaker.* VR-5 applied to assent: don't upgrade agreement past its basis.

### Changed (2026-06-24: DN-30 rev. 2 — the security catalog is lightweight / reconstruction-on-render (DN-28))

- **DN-30 §4 + DN-28 §5:** the security catalog **reuses DN-28's reconstruction-based distribution model
  verbatim** — the registry stores the findings' **hashes + manifest** (the dense, verifiable map; the
  fingerprints + severity/affected-version index + DAG), **not** the heavy finding bodies inline; the full
  finding (screened pattern, description, mitigations) is **reconstructed + hash-verified on render** from
  the content store. So the security catalog is **as lightweight as the package registry**, and because
  every finding is content-addressed, a published advisory is **tamper-evident** — reconstruction verifies
  it against its hash, so a poisoned/altered advisory **fails the check** (never-silent integrity, G2). The
  registry hosts **two content-addressed catalogs of the same shape** — packages and findings. E22-1 DoD
  updated. Still Draft, advisory.

### Changed (2026-06-24: DN-30 rev. — the registry as a screened security-advisory host (DN-28))

- **DN-30 §4 expanded:** a finding reports to **two sinks** — the **CLI report** *and* the **registry**. The
  same registry that hosts packages (phyla + nodules; DN-28) **also hosts the security findings**, as
  **screened / anonymized / privatized** entries: the vulnerable logic is minimized to a
  **content-addressed pattern fingerprint** so other scans match the same weakness across phyla **without
  exposing the victim's source** (disclose enough to defend, never to weaponize). The hosted finding records
  *what* (CWE + screened pattern + fingerprint) / *severity* / *affected content-addressed versions* (+ VEX)
  / *logic-retaining mitigations* (honestly tagged) / *confidence*. §7 gains the **screening-policy**
  governance question (what is safe to publish; who approves). Cross-referenced in **DN-28 §5** and the
  **E22-1** epic. Still Draft, advisory — enacts nothing.

### Added (2026-06-24: DN-30 — Security Scanning Toolkit (direction capture, Draft))

- **DN-30 — Security Scanning Toolkit** (`docs/notes/`, **Draft**, advisory). Captures the direction for a
  shippable, Mycelium-native security toolkit as a first-class toolchain component (inherited by downstream
  devs, like the testing toolkit): **automated** vulnerability detection over the inspectable Core IR — not
  solely documented findings, running in conjunction with them (never-silent, G2 — a flaw the toolchain can
  see is surfaced); **standard machine-consumable reporting** (SARIF + CWE + CVE/OSV + VEX, each finding
  carrying provenance + an honest confidence tag); **find-once-report-to-community** via an OSV-shaped
  advisory feed keyed by the content-hash DAG (DN-28); and **honest, semantics-preserving safe auto-fix**
  that reuses the **RFC-0002** refinement/translation-validation certificate (proves a fix removes the vuln
  **and** refines the original modulo it, honestly tagged — `Declared` fixes always flagged + human-gated,
  no black-box rewrites). Native + scoped (project/nodule/granular, reusing the RFC-0034 §6 `@certification`
  resolution). **Enacts nothing**; feeds a future security-scanning RFC + the backlog epic **E22-1**
  (`tools/github/issues.yaml`). Registered in `docs/Doc-Index.md`.

### Changed (2026-06-24: E21-1 — mode-parametric testing as a native, scoped toolkit capability (RFC-0034 §13 + M-796))

- **Generalized the test contract into a developer-facing toolkit capability.** The §13 mode-parametric
  test discipline is not just *our* convention — the **Mycelium testing toolkit** (`mycelium-std-testing`)
  exposes it as **first-class and natively wired in**: a developer marks a test/suite to run across the
  `CertMode` tiers (with the cross-mode *negative* pattern built in), getting the discipline **for free**
  rather than hand-rolling it. Its coverage is **configurable in scope, reusing the §6 `@certification`
  resolution** (most-specific-wins): **project-wide** default (manifest) > **nodule-wide** (header) >
  **granular** per test/unit — the §7 ergonomic, never-cornering stance applied to testing (tool +
  default + scope dial; never forced one way).
- **Captured in:** RFC-0034 §13 (the native-scoped-toolkit paragraph; recorded in the RFC changelog,
  append-only) and a new leaf **M-796** (`tools/github/issues.yaml`) — the developer-facing surface
  generalization (depends on M-790 resolution + the testing toolkit). M-795 reframed as the *kernel
  instance* of the same principle.

### Changed (2026-06-24: E21-1 — capture the mode-aware test strategy (RFC-0034 §13 + M-795))

- **Test-adaptation captured as a first-class E21-1 requirement.** As the `CertMode` tiers land, the
  suite must become **mode-aware**: every mode-sensitive test is **parameterized over `fast`/`balanced`/
  `certified`** and maps to the *intended per-mode* behaviour, and each invariant is checked **both
  ways** — it must *fire* in the tiers it applies to **and** be *correctly absent/relaxed* in the tiers it
  doesn't (the cross-mode **negative** cases), so a `certified`-only invariant holding spuriously in
  `fast` is caught, not silently passed. The pre-existing all-on suite is **adapted, not dropped** (each
  fixture pinned to `certified` or parameterized; mode-scope made explicit — the DN-20 tiered-test
  transparency rule).
- **Captured in:** RFC-0034 §13 (the conformance contract, now mode-parametric + negative — recorded in
  the RFC changelog, append-only); the E21-1 epic body + DoD (`tools/github/issues.yaml`); a new
  cross-cutting leaf **M-795** (the shared parameterization harness + the negative-case pattern + fixture
  adaptation), which **M-794** (the conformance gate) now depends on. Examples encoded: swap-cert
  *checking* present in `certified`/absent in `fast`; certificate *emission* in `balanced`+`certified`/none
  in `fast`; Axis-B never-silent in **every** mode.

### Added (2026-06-24: E21-1 Group A — cert-mode core (M-786 + M-787), RFC-0034 §3.1/§7)

- **`CertMode { Fast, Balanced, Certified }`** in `mycelium-core` (`cert_mode.rs`) — the tunable
  certification mode (RFC-0034 §5), default **`Fast`**, ordered by `depth()` (`Fast < Balanced <
  Certified`); serde form is the bare variant string. The first E21-1 implementation leaf (M-786).
- **`Meta` now carries a never-silent `cert_mode` tag** (RFC-0034 §3.1) — a non-`Option` field
  defaulting to `Fast`, with a `.with_cert_mode()` builder + `cert_mode()` accessor (mirroring the
  existing `with_physical`/`physical` pattern). Non-breaking: the field is private and the
  `Meta::new`/`Meta::exact` signatures are unchanged, so no caller or dependent breaks
  (`cargo check --workspace` green).
- **Content-hash exclusion holds by construction** — `cert_mode` rides `Meta`, which RFC-0001 §4.6
  excludes from the content hash wholesale, so switching modes never perturbs a value's identity
  (ADR-003). Verified by a new exhaustive test (`cert_mode_is_excluded_from_the_content_hash`).
- **Wire persistence deferred, not silent** — `cert_mode` is a runtime tag resolved from the
  `@certification` scope (M-790), so it is intentionally not in `MetaWire` yet (keeps
  `meta.schema.json` unchanged); a deserialized `Meta` resolves to **`Fast`** — the weakest mode,
  never silently claiming a stronger one (the VR-5 floor). Documented on `MetaWire` + tested.
- **`CertMode::gate_guarantee()` — the mode→tag floor (M-787, RFC-0034 §7).** `Fast` floors
  `Empirical`/`Proven` (whose trials/proofs it does not run) to **`Declared`** — the honest
  "computed, bound asserted-not-verified" tag (VR-5) — while structural `Exact` passes untouched;
  `Balanced`/`Certified` pass every strength through unchanged (mechanism preserved). The policy
  primitive; the operation layer applies it (with the bound's basis relabelled in lockstep) where ops
  become mode-aware. The M-787 invariant — **no `fast` result ever carries `Empirical`/`Proven`** — is
  proven directly by an exhaustive test over the finite strength space.
- Verified: `cargo fmt --check`, `cargo clippy -p mycelium-core --all-targets -D warnings`,
  `cargo test -p mycelium-core` (all green), `cargo check --workspace`.

### Added (2026-06-24: E21-1 — RFC-0034 paired-TDD implementation epic queued)

- **E21-1 (epic) + M-786…M-794** queued in `tools/github/issues.yaml` — the paired-TDD Rust-first
  implementation of RFC-0034 + ADR-032 (the runtime mode mechanism the design-driven enactment left
  pending). Dependency-ordered from the `Meta` mode tag outward: `CertMode` + never-silent mode tag
  (M-786) → mode-gated provenance tagging, `fast` omits Empirical/Proven (M-787) → mode-gated swap-cert
  emit/check (M-788) → compile/runtime phase split so spores survive a cert-off runtime (M-789) →
  `@certification` resolution/scoping reusing RFC-0012 (M-790) → `wrapping` Axis-B opt-out (M-791) →
  EXPLAIN-of-mode + generation≠consumption (M-792) → memory-safe per-use unsafe escape sharpening ADR-014
  (M-793) → the RFC-0034 §13 conformance gate that advances the RFC to Enacted-with-code (M-794). Each
  leaf carries property-test acceptance criteria + an honest "implemented (Rust-first)" framing (VR-5/G2).
  RFC-0034's `Task` field updated to reference the epic. ids collision-checked (E21/M-786… free); YAML
  validated (no duplicate ids); `doc_refs` resolve.

### Changed (2026-06-24: editorial — trim redundant "honest" qualifiers (concision; honesty is implied))

- **Concision pass — dropped the redundant "honest"/"honestly" filler** across CLAUDE.md, CONTRIBUTING,
  README, Foundation, Glossary (**61 edits / 5 files** via the never-silent `tools/dn29_apply.py` +
  `docs/notes/honest-trim-manifest.json`). Rationale (maintainer): honesty is **implied by construction** —
  G2/no-black-boxes guarantee that a trace, error, or guarantee tag that *isn't* honest is useless — so the
  adjective is noise, not information ("honest tags" → "tags", "honest bounds exist" → "bounds exist",
  "honest probabilistic guarantees" → "probabilistic guarantees", etc.). The same pass fixed the few stale
  **"honesty rule" → "transparency rule"** name references left after the ADR-032 rename. **Kept
  deliberately:** the formal named criteria **SC-2 "(honest bounds)"** and **VR-5 "(honest
  guarantee-strength)"** (cross-referenced labels, not filler), the Glossary `(H)` taxonomy marker, and the
  proper nouns `honesty-integrity` (ADR-021 Gate A) / `honest-stdlib` (prior-art ref). Purely editorial — no
  decision changes, no guarantee semantics touched (VR-5/G2 mechanism unchanged).

### Changed (2026-06-24: RFC-0034 + ADR-032 ratified & Enacted (design-driven); corpus amendments applied; DN-29 Superseded)

- **RFC-0034 + ADR-032 ratified `Proposed → Accepted → Enacted (design-driven)`** (maintainer, stepped per
  house rule #3). **Tunable certification is now the governing model.** The corpus amendments (ADR-032's
  act) were **applied** via the never-silent `tools/dn29_apply.py`: **21 amendments across 13 files** —
  - **Charter conditionalize:** Foundation **SC-3** / **FR-M3** now read "at the active mode (`certified`); the mode itself never-silent," attributed to RFC-0034/ADR-032.
  - **Living-doc transparency reframe:** CLAUDE.md house-rule 1 (**"the honesty rule" → "the transparency rule"**) + the "what this repo is" north-star line; CONTRIBUTING heading; README headline principle + decision-process bullet; Glossary ("honesty lattice" → "transparency lattice", the rule gloss, the `Declared` floor). **Mechanism unchanged** — the guarantee lattice, VR-5, and G2 keep their force; only the model-vocabulary wording moved.
  - **Append-only footnotes** on the relaxed Accepted decisions: RFC-0001/0002/0005 + ADR-010/011/013/016/017 (mandates apply at `certified`; `fast`/`balanced` per RFC-0034; spore/ABI/inject hashing is compile-time and survives a cert-off runtime).
  - **Deliberately excluded** (stringent scope): the ~45 development-process / colloquial uses of "honest" (swarm-review discipline, "defer honestly", "made honestly", "honest open follow-up") — not the guarantee-model term; rewording them would change meaning.
- **DN-29 → `Superseded`** (by RFC-0034 + ADR-032), retained as the append-only rationale record.
- **Pacing note (doc-and-design-driven loop, paired with TDD).** "Enacted" here is **design-driven**: the
  design + corpus amendments are landed and **governing**, while the **runtime mode *mechanism*** (modes
  affecting execution, tag computation, resolution) is the **paired TDD cycle — not yet code, never claimed
  implemented** (VR-5/G2). RFC-0034 advances to fully Enacted-with-code as the modes land Rust-first. This
  records the slightly-less-rigid pacing (design lands ahead of code) while keeping the honesty discipline
  stringent: nothing is claimed done that isn't.

### Added (2026-06-24: RFC-0034 + ADR-032 — the binding tunable-certification act (drafted from the settled DN-29))

- **RFC-0034 — Tunable Certification & Transparency Modes** (`docs/rfcs/`, **Proposed**). The binding
  *mechanism* from the settled DN-29 deliberation: makes certification/hashing/tag machinery a **tunable
  policy** over RFC-0001/0002/0005 — the knob matrix (§4); two **first-class modes** `fast` (default) /
  `certified` + a `balanced` intermediate (§5); `global/phylum/nodule` resolution reusing **RFC-0012**
  scoping + a `@certification` attribute, no content-hash perturbation per ADR-003 (§6); the **provenance
  tag as an adjustable unit** (`fast` omits `Empirical`/`Proven`, sits at structural `Exact`/`Declared`)
  and the **generation≠consumption** signal split (§7); the **compile/runtime phase split** — spores
  survive a cert-off runtime (§8); memory-safe-by-default + an explicit per-use escape sharpening
  **ADR-014** (§9); the named `wrapping` Axis-B opt-out (§10); and the never-silent mode invariant (§3).
  Implementation-focused; decides the surface, implements nothing (VR-5/G2).
- **ADR-032 — Tunable certification supersedes always-on; transparency reframe** (`docs/adr/`,
  **Proposed**). The superseding ADR: supersedes the **unconditional** reading of **SC-3/FR-M3** +
  RFC-0001 §3.4/§4.6, RFC-0002 §2, RFC-0005 §2 (they hold **at the active mode**; the mode itself is
  never-silent; mechanisms unchanged); reframes **"honesty" → "transparency & auditability"** whole-corpus
  (the lattice/VR-5/G2 mechanism is unchanged — only the wording); repositions the **north star** toward a
  fast, memory-safe, ergonomic language with certification optional (the Foundation **mission sentence is
  already transparency-framed** — unchanged); **sharpens ADR-014** (safe-by-default + explicit per-use
  escape); and adds append-only **§-end footnotes** to RFC-0001/0002/0005 + ADR-010/011/013/016/017.
- **Staged amendment tooling** (`tools/dn29_apply.py` + `docs/notes/dn29-amendment-manifest.json`). The
  **anchor-keyed, single-pass-per-file** corpus amender (DN-29 §11.4) that applies the rewordings without
  positional mangling: **dry-run by default**, **never-silent** (each anchor must match exactly once or it
  fails loudly), and it **refuses `--apply` while the manifest is not `final`**. **Staged, not run** — the
  corpus edits land only **after** RFC-0034 + ADR-032 are Accepted (RFC-0034 §13). The manifest ships the
  two validated Foundation spine anchors (SC-3, FR-M3) as dry-run-clean worked examples; the rest are
  authored at ratification. Registered in `docs/Doc-Index.md`.

### Added (2026-06-24: DN-29 — Tunable Certification & Honesty Modes (Draft deliberation anchor, advisory))

- **DN-29 — Tunable Certification & Honesty Modes** (`docs/notes/`, **Draft**, advisory). Anchors the
  deliberation on making the certification/honesty/hashing machinery **tunable off → full** via scoped
  config instead of mandatory-everywhere, *while preserving honesty*: the two-axis split (certification
  **depth** vs cheap never-silent **failure semantics**), a strawman level ladder (L0 Raw…L3 Proven) +
  the content-hash sub-toggle (spores/hot-inject depend on it), the knob's home (reuse RFC-0012
  ambient-scoping + `mycelium-proj` manifest/header `@certification`; ADR-003 → no content-hash
  perturbation), and the honesty argument (generalizes **KC-4**; a systematic flagged downgrade to
  `Declared` per **VR-5**; **G2** never-silent about the active mode). **Enacts nothing**; the binding
  decision is the future **RFC-0034** + a superseding ADR. Open questions tracked in §9.

### Changed (2026-06-24: DN-29 rev. 4 — deliberation settled; §9 fully resolved (still Draft, advisory))

- **DN-29 deliberation settled (in place; stays Draft)** — owner confirmed the remaining forks; **§9 has no
  open questions left.** Resolutions: Q4 expose the named **`wrapping`/`fast`** Axis-B opt-out in v0; Q5
  ship **global/phylum/nodule** scope and defer the per-op `thaw`-style knob (YAGNI); Q6 the **two-step**
  path (settle DN-29 → **RFC-0034 + a superseding ADR**, §11 as the RFC appendix); Q7 **named modes ship
  first**, per-knob overrides later; Q11 the honesty→transparency reframe is **whole-corpus**; Q12 the
  **superseding ADR amends the Foundation/charter** while **RFC-0034 stays implementation-focused**; Q13
  Accepted RFCs/ADRs get **append-only §-end footnotes** pointing to the relaxations (§9/§10/§11.5). Still
  **advisory — enacts nothing** (VR-5/G2); the binding act is the forthcoming RFC-0034 + ADR.

### Changed (2026-06-24: DN-29 rev. 3 — provenance tag adjustable, generation≠consumption, §11 ripple map (still Draft, advisory))

- **DN-29 refined again (in place; stays Draft)** — **provenance tag is now an adjustable unit**: `fast`
  defaults to **not using `Empirical`/`Proven`** (they cost the trials/proofs `fast` skips), sitting at
  structural `Exact`/`Declared`; heavier tags dial up per mode/unit — the honest floor (`fast` never claims
  a tag it didn't earn, VR-5). **Signal generation split from consumption**: the cheap inspectability signal
  is **always generated ≥ middle tier** (safe default) while **consumption** (DX/UX surfacing, diagnostic
  noise) is tunable and lean at `fast`, so a dev can dial it up mid-session with history already captured
  (§3.1). Adds **§11 — a ripple map**: a ~40-hit / ~12-file `Empirical/Declared` inventory of every corpus
  location the binding RFC-0034 + superseding ADR must amend (change-type taxonomy: conditionalize-per-mode
  / honesty→transparency / north-star / tag-adjustable / memory-safety; high-collision files README/
  Foundation/CLAUDE.md flagged; multi-category mangle-risk lines called out; spine anchors SC-3/FR-M3/VR-5
  validated against source), plus an **anchor-keyed, single-pass-per-file batched-replacement mechanism**
  (content-keyed not positional ⇒ no mangling; never-silent guard fails loudly on a missing/ambiguous
  anchor). Still **advisory — enacts nothing** (VR-5/G2); the inventory is heuristic, to be re-verified at
  RFC-0034 drafting time.

### Changed (2026-06-24: DN-29 rev. 2 — first-class modes, memory-safety default, "honesty"→"transparency" reframe (still Draft, advisory))

- **DN-29 refined again (in place; stays Draft)** — **`fast` and `certified` elevated to two first-class
  modes** (`balanced` an optional intermediate, §3.2); **memory-safe by default with an *explicit per-use*
  unsafe escape hatch** (ADR-014 precedent sharpened to per-use, §3.1); a **tunable diagnostic-verbosity
  knob** mode-defaulted (lean at `fast` → full audit trail at `certified`), with guarantee **tagging kept
  on** as the inspectability substrate (§3.1); and a **vocabulary reframe — "honesty" → "transparency &
  auditability"** (§1/§6): default `fast` gives transparent, inspectable *non-certified auditability*,
  `certified` upgrades it to a *fully auditable* framework. The **mechanism is unchanged** (never-silent
  G2, the provenance lattice, `EXPLAIN`, VR-5 downgrade-don't-overclaim); only the framing moves. The
  CLAUDE.md house-rule 1 / CONTRIBUTING / VR-5 / G2 / Foundation §1 vocabulary ripple is **flagged for
  RFC-0034 + the superseding ADR** — *not* rewritten here (append-only). §9 Q2′/Q8–Q10 resolved; Q4–Q7
  open. Still **advisory — enacts nothing** (VR-5/G2).

### Changed (2026-06-24: DN-29 rev. — owner-steered refinement (still Draft, advisory))

- **DN-29 refined (in place; stays Draft)** — the single L0–L3 ladder is **decomposed into independent
  knobs split by *phase*** (compile/deploy vs runtime) composing into named **profiles**
  (`fast`/`balanced`/`certified`, §3). Keystone: **compile/deploy spore-identity hashing is decoupled from
  runtime certification**, so deployable, content-addressed units survive a fully cert-off runtime (§2,
  resolving the old "L0 loses spores" coupling). **Default profile = `fast`** (memory-safe, never-silent,
  spores available; runtime cert/hash/check off — certification opt-in, §5). Captures a **north-star
  reframe** — Mycelium as a fast, memory-safe, ergonomic multi-paradigm language with certification baked
  in as *optional* — with the Foundation §1 / SC-3 / FR-M3 ripple **flagged** for RFC-0034 + the
  superseding ADR (**not** amended here; append-only). Open questions Q1–Q3 resolved; Q4–Q7 open (§9).
  Still **advisory — enacts nothing** (VR-5/G2).

### Changed (2026-06-24: ADR-029/030/031 → Accepted — value-model ternary + the two content-address one-way doors ratified (owner approval))

- **ADR-029 — Ternary arithmetic is arbitrary-width → Accepted** (maintainer-ratified). The V0
  `BigTernary` reference implementation landed (#535); the decision is now locked.
- **ADR-030 — Dense granularity-descriptor quant → Accepted** and **ADR-031 — VSA element-space +
  block-sparse + complex carrier → Accepted** (maintainer-ratified). Both are **content-address
  one-way doors**; their implementation lands in the single E20-1 rehash (M-780) **before any
  Dense/VSA value is persisted** (RFC-0033 §7). RFC-0033 + ADR-025…028 remain **Proposed**
  (implementation proceeds as *"implemented (Rust-first), pending ratification"*; the collections
  decisions also defer to the already-Accepted RFC-0032).

### Added (2026-06-24: E20-1 V0 — arbitrary-width balanced ternary `BigTernary` landed (M-754…M-757); implemented, pending ratification)

- **M-754/M-755 — DRY balanced full-adder** (`crates/mycelium-core/src/ternary/`). Converted
  `ternary.rs` → a `ternary/` module dir and extracted the inline full-adder from the fixed-width `add`
  into a shared `add_with_carry(Trit, Trit, carry) -> (Trit, Trit)` — proven identical to the incumbent
  over all 27 inputs (an exhaustive truth-table test). Both the fixed-width `add` and the new growable
  `BigTernary` now ripple this single never-silent primitive. No new crate, no duplicate `Trit`; the
  public `mycelium_core::ternary::{digit,add,sub,mul,neg,int_to_trits,trits_to_int,max_magnitude}`
  surface and every consumer (std-ternary, std-swap, interp, cert, mlir) are unchanged.
- **M-756/M-757 — arbitrary-width `BigTernary`** (`…/ternary/big_ternary.rs`). A digit-serial,
  canonicalized `Vec<Trit>` integer that **grows instead of overflowing** — *removing* the fixed-width
  ~40-trit cap (which `core::ternary` was already never-silent about; the silent-overflow defect is
  `embeddonator`'s `dimensional.rs`, not Mycelium's). `from_i128`/`to_i128` (overflow-checked),
  `add`/`sub`/`mul`/`neg`, and the never-silent fixed-width boundary `FixedWidthTrits` +
  `checked_add_fixed` + `checked_to_width` (`None` on out-of-range, never a wrap). All ops **`Exact`**.
  Witnessed by the `3^41`-exact test and a cross-check that `BigTernary` agrees with the fixed-width
  `add` within range (bridged through the integer value — MSB-first ↔ LSB-first). RFC-0033 §4.2 / ADR-029.
- Verified on **MSRV 1.92**: `cargo fmt` / `clippy -D warnings` / `test -p mycelium-core` green (11 new
  tests); `cargo check --workspace` + the public-API surface gate + direct-consumer tests all green.
  `docs/spec/api/mycelium-core.txt` regenerated for the additive surface; `docs/api-index` refreshed.
  `PackedTernary` / Karatsuba (M-758/M-759) remain YAGNI follow-ons, gated on a benchmark.

### Added (2026-06-24: E20-1 — value-model collections & precision design (RFC-0033 + ADR-025…031, **Proposed**); docs-only, M-785)

- **RFC-0033 — Value-Model Collections & Precision** (`docs/rfcs/`, **Proposed**, E20-1/M-785). The
  corrected `Repr`/`Payload` as a normative-as-proposed spec: §3 `Seq`/`Bytes` length-in-type (aligned
  with the already-decided RFC-0032 D3/D4, not re-decided); §4 Binary **sign-free** (signedness is
  operations, not a `Repr` field) · Ternary **arbitrary-width** (`BigTernary` grows past the ~40-trit
  cap) · Dense **granularity-descriptor quant in `Repr`** + scale/zero-point **arrays in `Payload`** ·
  VSA **explicit element-space + block-sparsity + complex carrier**; §6 swap/guarantee reconciliation;
  §7 content-address identity set + the dogfood gate (single rehash M-780).
- **ADR-025…031** (`docs/adr/`, **Proposed**) — the seven value-model decision records (Seq/Bytes
  length-in-type · repr-value elements · `get→(Repr,bit)` + `lift_option` · Binary sign-free · Ternary
  arbitrary-width · Dense granularity descriptor · VSA element-space). ADR-030 (Dense) and ADR-031 (VSA)
  deliberately **disagree** with the input research draft on never-silent grounds and are
  **content-address one-way doors**.
- **Research records** `research/14-value-model-integration-report-RECORD.md` +
  `research/15-embeddonator-leverage-map-RECORD.md` (recorded external research input — an
  `embeddonator` value-model bundle; not normative), and the **plan/backlog**
  `docs/planning/value-model-{implementation-plan,backlog}.md` (tasks remapped `VM-010…071` →
  **M-754…M-784**, design gate **M-785**, all under epic **E20-1**).
- **Honesty reconciliations vs the source bundle** (recorded in the docs): (a) **no `mycelium-value`
  crate / no duplicate `Trit`** — the trusted arbitrary-width ternary reconciles into
  `crates/mycelium-core/src/ternary/` (DRY `add_with_carry` extracted from the existing fixed-width
  `add`); (b) the **"silent precision ceiling" is `embeddonator`'s** (`dimensional.rs`), **not
  Mycelium's** — `core::ternary` is already never-silent about the cap (`max_magnitude → None` at
  `m ≥ 41`), so `BigTernary` **removes** the cap rather than fixing a bug; (c) **OQ-3 is already closed
  by the ratified ADR-011** (BoundBasis is universal) — RFC-0033 §6.2 *extends* the dequant
  `bound.basis` with block structure, it does **not** reopen OQ-3; (d) collections align with RFC-0032.
- **Scope:** docs-only — no Rust, no `docs/api-index` regen. The V0 `BigTernary` kernel code
  (reconciled into `core::ternary`, cargo-gated) follows in a separate PR (M-754…M-757); the
  content-address one-way doors (Dense/VSA) land only after ratification, in a single rehash before any
  value is persisted (RFC-0033 §7).

### Added (2026-06-23: E19-1 — Tier-1 kernel enablers landed (M-747, M-748); implemented, pending ratification)

- **M-747 — reduce-to-`Bool` comparison/equality prims `eq`/`lt`** (RFC-0032 D1). New kernel prims
  `cmp.eq`/`cmp.lt` (`crates/mycelium-interp/src/prims.rs`) over `Binary{N}`/`Ternary{N}`: each takes
  two equal-width same-paradigm operands and returns `Binary{1}` (`0b1` = true), guarantee **`Exact`**.
  `eq` is structural width-typed equality; `lt` is the D1 total order (unsigned magnitude for Binary,
  balanced-integer value for Ternary, MSB-first lexicographic). Surfaced `eq`/`lt` via a dedicated
  **width-collapsing** checker branch (operands `T{N}` → `Binary{1}` does not fit the width-preserving
  `prim_family` path). Cross-paradigm / mismatched-width / bare-decimal comparands are explicit
  never-silent refusals (G2). **Realization note (engineering call, Q1):** a kernel prim returns a
  representation value, never a `.myc` data value, so D1's `Bool` bottoms out as `Binary{1}`; the
  `.myc` `std.cmp` lift to the `Bool` ADT is a one-line match (demonstrated by the bool-bridge smoke
  port — the E13-1 M-718 consumer). Declared in the content-addressed Π table with a new
  **`WidthRel::Collapse`** (the sanctioned "new width rule = a variant" extension). **Unblocks** E13-1
  M-718 (width-typed `cmp`/`Eq`/`Ord`). (RFC-0032 D1; E19-1/M-747)
- **M-748 — never-silent fixed-width binary arithmetic** (RFC-0032 D2). Surface the already-registered
  `bit.and`/`bit.or` (`and`/`or`); add kernel prims `bit.add`/`bit.sub` (surface `add_bin`/`sub_bin`):
  unsigned ripple-carry add / ripple-borrow subtract over `Binary{N}`, guarantee **`Exact`** on the
  in-range result. A result outside `[0, 2^N)` is an explicit `EvalError::Overflow`, **never** a silent
  wrap — mirroring the `trit.*` in-range contract (G2). Distinct surface names from the trit-backed
  `add`/`sub`. **Unblocks** E13-1 M-718 (binary `math`). (RFC-0032 D2; E19-1/M-748)
- **M-752 (partial — Tier-1) — enablement conformance.** `crates/mycelium-l1/tests/enablement.rs`:
  three-way differential smoke ports (L1-eval ≡ L0-interp ≡ AOT) per unblocked surface + never-silent
  refusal tests (overflow/underflow refuse on every path; mismatch refuses statically), plus a
  `Bool`-bridge port. Prim unit/mutant-witness tests + Π/surface consistency guards extended.
  `docs/api-index/` + the `mycelium-core` public-API baseline regenerated (deterministic). RFC-0032
  stays **Accepted** (not Enacted) — specs are "implemented, pending ratification" (VR-5). The Tier-2
  reprs (M-749 `Repr::Seq` / M-750 `Repr::Bytes`) are KC-3-significant, maintainer-sign-off-gated core
  additions and are **not** in this change. (RFC-0032 D7; E19-1/M-752)
### Changed (2026-06-23: RFC-0026 → Accepted — editor-grammar scope names ratified; M-693 done, M-731 finalized; E16-1 epic `done`)

- **RFC-0026 — Editor Syntax Highlighting Grammar → Accepted** (M-693, the E9-1 gate; Draft → Proposed
  → Accepted same day, maintainer-ratified). §3 fixed normatively: **§3.1** artifact scope (the three
  grammar layers — TextMate · tree-sitter · LSP semantic tokens — are the E9-1/E16-1 gate; the VS Code
  extension + GitHub Linguist registration are the **M-697** follow-up); **§3.2** the scope-name table
  — **standard names per layer** (TextMate scopes carry a `.mycelium` suffix; tree-sitter captures and
  LSP token types are the standard *unsuffixed* names — chosen for maximal theme compatibility, §5 Q2)
  mapped over the **lexer-derived** keyword/type/scalar/strength buckets; **§3.3**
  the single-source-of-truth/drift contract (already implemented: `tools/grammar/generate.py` +
  `just drift-check`). **DN-24 → Resolved** (its recommended layered stack adopted). (M-693; RFC-0026)
- **M-731 finalized — editor grammars now ship ratified scope names.** The `TODO.rfc-0026.*`
  placeholders are replaced with the RFC-0026 §3.2 names: TextMate (`keyword.control.mycelium`,
  `storage.type.mycelium`, `support.type.builtin.mycelium`, `storage.modifier.guarantee.mycelium`,
  `comment.line.double-slash.mycelium`, `constant.numeric.mycelium`), tree-sitter captures
  (`@keyword`/`@type`/`@type.builtin`/`@attribute`/…), and the LSP legend (the M-730 semantic-token
  types are that table's LSP layer). Still **lexer-derived + drift-checked** (G2 — `just drift-check`
  green). Subsumes the E9-1 leaves **M-694** (TextMate) / **M-695** (tree-sitter scaffold) / **M-696**
  (LSP semantic tokens). The full structural tree-sitter grammar + M-697 packaging remain follow-ups.
  (M-731; M-694/M-695/M-696)
- **Epic E16-1 → `done`** (all five children landed: M-730/M-731/M-732/M-733/M-734). **E9-1 →
  `in-progress`** (M-693 + M-694/M-695/M-696 done; **M-697** VS Code extension + Linguist remains). (E16-1; E9-1)

### Added (2026-06-23: ADR-024 — Core 1.0.0 Gate (T1) scope amendment, enacting RFC-0032 D6 append-only)

- **ADR-024 — Core 1.0.0 Gate (Track T1) Scope Amendment → Accepted.** The house-rule-correct
  (supersede-to-change-criteria — house rule #3) capture of RFC-0032 §5 D6: it **amends ADR-022 track
  T1** to add epic **E19-1** (the kernel self-hosting-enablement surface — `eq`/`lt` prims, binary
  arithmetic, `Repr::Seq`, `Repr::Bytes`) to the `core 1.0.0` Definition of Done, so the stdlib is
  fully `.myc`-self-hosted at the tag. A **scoped amendment**, not a wholesale supersession: ADR-022's
  dual-version model + tracks T2–T9 + the preserved ADR-021 Gate A/B rows all remain in force and
  **unchanged + met**; `M-703` now `depends_on` E19-1. ADR-022 is touched only with an append-only
  "amended by ADR-024" pointer (§4 note + §5 T1 row + changelog) — its normative §4/§5 criteria text is
  **not** rewritten (resolves Copilot #514: the earlier in-place criteria edit was reverted, then
  enacted here via the proper mechanism). All criteria `Declared` until each E19-1 leaf lands
  differential-tested (VR-5). →Enacted with ADR-022 T1 at the `core 1.0.0` tag. (ADR-024; RFC-0032 D6;
  E19-1)

### Changed (2026-06-23: RFC-0032 → Accepted — the kernel self-hosting-enablement surface ratified; M-746)

- **RFC-0032 — Kernel Self-Hosting Enablement Surface → Accepted** (M-746, the E19-1 gate). §5 D1–D7
  ratified: **D1** `eq`/`lt` comparison prims over `Binary{N}`/`Ternary{N}` (→ `Bool`, `Exact`; `cmp`/
  `Ordering` derives in `.myc`); **D2** binary arithmetic (surface the registered `bit.and`/`bit.or` +
  add a never-silent carry-chain `add`/`sub` — overflow is an explicit error, never a silent wrap, G2);
  **D3** a first-class **`Repr::Seq`** (indexed sequence + never-silent `get`/`push`) for efficient
  `Vec`/`Map`/`Set`; **D4** a dedicated **`Repr::Bytes`** (byte/string value + never-silent UTF-8
  decode) for `text`/`fmt`; **D5** width-generics → **E11-1/`s10`** (M-751 closed as a pointer to the
  new **M-753** under E11-1; E13-1 M-718 `depends_on` repointed); **D6** placement **in `core 1.0.0`**
  (maintainer); **D7** sequencing (comparison → binary-arith → `Repr::Seq` → `Repr::Bytes` →
  conformance). Enablers **M-747…M-750 → `todo`** (RFC gate cleared); E19-1 → in-progress.
- **D6 governance (append-only — house rule #3 + Copilot #514).** "In `core 1.0.0`" extends ADR-022
  track T1's Definition of Done (the core tag waits on E19-1) — a *criteria* change to an **Accepted**
  ADR, which ADR-022's Status requires capturing by **supersession**, not an in-place edit. The earlier
  in-place §4/§5 amendment was **reverted**, and the change is now enacted append-only by the focused
  amending **ADR-024** (Accepted 2026-06-23 — see the entry above): ADR-022's §4/§5 criteria text stays
  pristine (only an "amended by ADR-024" pointer), the decision lives in RFC-0032 D6, and `M-703`
  `depends_on` E19-1. (RFC-0032 D6; ADR-024; ADR-022 §4; E19-1/M-746)

### Added (2026-06-23: E19-1 — kernel self-hosting-enablement work leg scaffolded; RFC-0032 Draft + kickoff `kpr`)

- **New work leg E19-1 + RFC-0032 (Draft) — the kernel surface that unblocks E13-1's blocked tiers.**
  RFC-0031 §5 D4 found Tier-0 executable (landed, M-715) but Tier-1/Tier-2 blocked on kernel surface
  that does not exist: a reduce-to-`Bool` comparison prim + binary arithmetic (Tier-1), and
  sequence/array + byte/string value representations (Tier-2 — the value model `Repr` =
  `Binary`/`Ternary`/`Dense`/`Vsa` has neither). These **enlarge the value model / trusted base
  (KC-3)**, so the leg is **design-gated** by `docs/rfcs/RFC-0032-Kernel-Self-Hosting-Enablement-Surface.md`
  (Draft stub — 7 open questions: prim shape, binary-overflow semantics, whether an indexed `Repr` is
  required vs the recursive-ADT `List`, string repr, width-generics ownership, the core-1.0.0-vs-post-1.0.0
  placement against ADR-022, and sequencing). Epic **E19-1** + issues **M-746** (RFC authoring, the gate)
  → **M-747** (comparison prim) · **M-748** (binary arithmetic) · **M-749** (sequence repr) · **M-750**
  (byte/string repr) · **M-751** (width-generic fns — ownership per RFC-0032 Q5) → **M-752** (conformance +
  `.myc` smoke ports). Cross-leg continuity wired via `depends_on`: E13-1 M-716 ⟸ M-749, M-717 ⟸ M-750,
  M-718 ⟸ M-747/M-748/M-751. Kickoff **`kpr`** stowed (`.claude/kickoffs/kpr.md`, registered in the
  index) — owns `crates/mycelium-interp/src/prims.rs` + the `prim_kernel_name` map; the `mycelium-core`
  reprs + L1 width-generics are flagged **coordinated** with `c10`/`s10` (maintainer sign-off on the
  RFC-0032 KC-3/placement before merge — architecturally significant, flag-don't-guess). No kernel code
  changed yet (planning + design-gate scaffolding only). **Maintainer direction recorded (2026-06-23,
  for M-746 to ratify):** RFC-0032 Q6 = **in `core` 1.0.0** (the reprs/prims land before the 1.0.0 tag
  → E19-1 becomes a core-1.0.0 gate prerequisite; ADR-022 T1 / E10-1 / `c10` need a maintainer update so
  the core tag accounts for E19-1 — flagged, not edited here); Q5 = **E11-1/`s10`** (width-generics
  reassigned; M-751 → pointer). (RFC-0032; E19-1/M-746)
### Added (2026-06-23: E16-1 — toolchain, IDE & package distribution; M-730/M-732/M-733/M-734 `done`, M-731 scaffold)

- **Full LSP providers — `mycelium-lsp` hover / go-to-definition / semantic tokens (M-730).** Three
  position-aware providers extend the server beyond completions/diagnostics/fmt: `textDocument/
  semanticTokens/full` (a standard LSP legend + relative-delta encoding, classified by token kind),
  `textDocument/hover` (grounded descriptions for keywords, substrate types, and guarantee-strength
  tokens), and `textDocument/definition` (single-document `fn`/`type`/`trait` navigation). A shared
  lexical span layer recovers token lengths from the canonical L1 lexer without duplicating it.
  **Honest scope (`Declared`/VR-5):** classification is lexical, hover **refuses to fabricate** an
  identifier's type/guarantee (it flags the absence instead), definition is single-document, and an
  unknown position/symbol is a null result — never-silent (G2). Capabilities advertised; all prior
  tests green; clippy `-D warnings` clean. (M-730)
- **Editor-grammar generator + drift gate — `tools/grammar/` (M-731; SCAFFOLD).** A generator derives
  the keyword set + class buckets (keyword/type/scalar/strength) from `token.rs::keyword()` and renders
  TextMate (`.tmLanguage.json`) + tree-sitter (`grammar.js` + `highlights.scm`) artifacts; `just
  drift-check` (wired into `just check`) fails on any divergence from the lexer (G2 — the grammars can
  never silently drift from the language the compiler accepts). **Honestly a scaffold (VR-5):** RFC-0026
  §3.2 (the scope-name table) is still `Draft`, so the TextMate/tree-sitter scope names are emitted as
  explicit `TODO.rfc-0026.*` placeholders — **not finalized, never guessed**; they land when RFC-0026 is
  Accepted (M-693). (M-731 — `in-progress`)
- **`spore` registry — publish / resolve (M-732).** `mycelium-spore` grows from the build artifact to a
  package manager: a local, content-addressed store where `publish` records the descriptor bytes by
  BLAKE3 (the `artifact` integrity hash) plus an index entry carrying the `spore_id` DAG identity
  (ADR-003), and `resolve` fetches by name + exact-version-or-`latest` and **verifies the bytes against
  the recorded address before returning**. Never-silent (G2): republishing a different artifact under an
  existing `name@version` is a refused `Conflict` (immutability); a missing/tampered object is an
  `Integrity` error; a SemVer **range** is an explicit `Unsupported` error (v0 never mis-resolves a range
  it cannot honestly evaluate — ADR-018 deferred). proptest hash-verification bound; `spore publish`/
  `resolve` CLI subcommands. (M-732)
- **`myc` one-command toolchain driver — `mycelium-cli` (new crate; M-733).** `myc init|build|check|
  test|run` over a phylum, calling the real library APIs directly (no fragile subprocess plumbing).
  `init` scaffolds a gate-clean phylum (refuses a bad name, never overwrites — G2); `build` packages the
  spore; `check` parses + type-checks every `.myc` via the L1 front-end; `test` runs the check
  verification (explicit that a `.myc` unit-test runner is future work — not faked); `run` is **honestly
  not-yet-wired** and reports so with an actionable message + exit 70, never a silent no-op. **Error
  quality bar (DN-22/RFC-0013):** every failure is a structured `Report` (`error[<code>]: <message>` +
  `--> location` + `help:`) — no raw Rust panic reaches the user. (M-733)
- **Reproducible toolchain distribution — `scripts/dist/` (M-734).** A pinned, content-addressed install
  path: `install.sh` verifies a toolchain artifact against a committed self-describing content-address
  pin (`blake3`/ADR-003 when `b3sum` is present, else `sha256`) before copying it into place; re-running
  on the same pin is byte-identical, and a tampered/missing/added file is a never-silent integrity error
  (G2). **Strict, never skip-graceful:** a missing hasher hard-fails (exit 69) rather than skipping the
  check. `just dist-verify` self-tests the mechanism end-to-end. **Honest scope (VR-5):** pinning the
  actual compiler binaries additionally needs a reproducible release *build* (deferred); the
  pin/verify/install mechanism is what landed. (M-734)

### Added (2026-06-23: E14-1 completion — M-722/M-723 syscall floor wired + data guarantee matrix; epic `done`)

- **`mycelium-std-sys` guarantee matrix encoded as data (`guarantee_matrix.rs`; M-722).** The prior
  M-722 increment shipped the real `io`/`fs`/`sys` floors with per-op tags in **prose doc tables**;
  RFC-0016 §4.5 / VR-5 require the matrix as **data, asserted in tests, never prose-only**. The new
  module supplies exactly that: one `MatrixRow` per floor op (io/fs/sys/rand/time/math, 31 rows),
  **every op `Declared`** (the honest floor for an unaudited host wrapper — promotion needs its own
  checked basis), with fallibility/error-set/effect columns. Tests guard coverage, the all-`Declared`
  invariant (no silent upgrade, VR-5), fallibility↔error-set consistency, and the wall-clock /
  entropy effect declarations (RT3). (M-722; RFC-0016 §4.5)
- **Production host wiring — `mycelium-std-sys-host` (new crate; M-722/M-723).** The pure std crates
  kept their OS contact behind injectable seams (`EntropySource`, `ClockSource`) so they stay
  `wild`-free; this crate fills those seams with the audited floor: **`OsEntropy`** drives
  `std-rand`'s `EntropySource` from `std-sys::rand` (`/dev/urandom`), **`OsClock`** drives
  `std-time`'s `ClockSource` from `std-sys::time` (monotonic + wall + a FLAGged logical placeholder).
  It is the one crate depending on **both** the floor and the pure crates, so the dependency
  direction stays honest (pure std → seam ← host wiring → floor); `#![forbid(unsafe_code)]`, no kernel
  coupling. Every read is `Declared`; failures are explicit (`EntropyUnavailable`, `ClockUnavailable`,
  `Overflow`) — never a zero-fill or clock wrap (G2). End-to-end tests seed `EntropyRng` from the OS
  and assert monotonic-clock non-regression. (M-722/M-723; RFC-0028 §4.5)
- **M-722, M-723, and epic E14-1 → `done`.** With the floors executing, the data matrix landed, and
  the entropy/clock seams wired, the FFI epic's Definition of Done is met. **Honestly staged
  follow-up (not a regression — already deferred in RFC-0028 §4.4):** the Mycelium-surface `wild:`
  per-op byte encoding that makes the byte-oriented `io`/`fs` ops reachable from a `wild { io.write(…) }`
  block is the `@std-sys`-author host encoding, still uncommitted in §4.4; the entropy/clock seams are
  wired today and the io/fs surface encoding follows when §4.4 lands. (E14-1; VR-5/G2)

### Added (2026-06-23: E13-1 — self-hosted stdlib composition ratified + the executable core/prelude; RFC-0031 Accepted; M-714/M-715 Tier-0)

- **RFC-0031 — Self-Hosted Standard Library Composition → Accepted** (M-714, the E13-1 gate). §5 D1–D7
  ratified: **D1** the irreducible-Rust boundary (`mycelium-core`/`l0`/`l1`/`cert`/`swap`/`interp::prims`/
  `mlir`/`std-sys` stay Rust) + the decision criterion (trust-root / bootstrap-floor / value-model-FFI /
  unsafe-ABI); **D2** phylum layout (`std.<module>` → `lib/std/<module>.myc`, crate-mirrored); **D3** no
  bootstrap circularity (the Rust frontend compiles the `.myc` stdlib; ring layering forbids `use`-cycles);
  **D4** the surface-readiness-**tiered** migration order — the honesty crux (VR-5): only the
  structural/polymorphic core is executable today, so `collections`/`iter`/`text`/`fmt` (Tier-2) and
  width-typed `cmp`/`math` (Tier-1) are sequenced **behind** the kernel prims (a reduce-to-`Bool`
  comparison, binary arithmetic, a sequence/string representation) that would enable them — never claimed
  ahead; **D5** the per-op stability bar + the `std_result`/`std_option`/`std_cmp` differential-test
  prototype pattern; **D6** the `mycelium-std-*` Rust crate kept as the differential oracle (deprecated, not
  removed); **D7** one `spore` per phylum. (RFC-0031; E13-1/M-714)
- **M-715 (Tier-0) — the executable core/prelude self-hosts.** `lib/std/option.myc` (`Option<A>` +
  `is_some`/`is_none`/`unwrap_or`/`map`/`and_then`/`fold`/`or_else`/`flatten`, the never-silent sibling of
  `std.result`), `lib/std/cmp.myc` (`Ordering` + `is_lt`/`is_eq`/`is_gt`/`reverse` + structural
  `bool_eq`/`bool_cmp`/`ord_eq` over the finite kernel types), and an extended `lib/std/result.myc`
  (`map_err`/`or_else` added to the M-649 surface) are written in `.myc`, three-way **differential-tested**
  (L1-eval ≡ L0-interp ≡ AOT — `crates/mycelium-l1/tests/std_option.rs` + `std_cmp.rs` + `std_result.rs`,
  44 tests green via the M-649 harness), and registered in the `std` phylum manifest
  (`lib/std/mycelium-proj.toml`). Honest tags (VR-5): total finite/structural ops `Exact`; generic
  combinators `Declared`; differential agreement `Empirical`.
  Never-silent (G2): `unwrap_or`/`fold` take a caller-supplied fallback — `None` never silently becomes a
  value. **Honestly deferred:** width-typed `cmp`/`Eq`/`Ord` (needs a comparison prim → Tier-1, blocks with
  M-718) and the `iter` trait surface (needs a concrete sequence → Tier-2, blocks with M-716) are *not*
  claimed self-hosted ahead of their enabling surface. M-716/M-717/M-718 moved to `status:blocked` with the
  explicit kernel-prim precondition recorded; E13-1 → `in-progress`. (RFC-0031 §5 D4/D5; E13-1/M-715)

### Added (2026-06-23: E14-1 — the `wild`/FFI execution floor executes; RFC-0028 Accepted; M-720/M-721/M-722/M-724)

- **RFC-0028 — FFI and System Interface → Accepted** (maintainer sign-off on the three architecturally-significant forks). The normative v0 model: a **build-time `@std-sys` capability gate** (no runtime `Capability<io>` value — KISS/YAGNI/KC-3; runtime sandboxing deferred §7, flagged forward-compatible); the **prim registry as the execution host / capability handle**; `wild` lowers to `Op { prim: "wild:…" }` (**no new Core-IR node** — KC-3); `Declared` guarantee baseline with `Empirical` only for a differentially-covered deterministic op (VR-5); a full Mycelium-level `just safety-check` audit. (RFC-0028; E14-1)
- **M-720 — `wild` execution lands: the staged `Residual` becomes a real host dispatch.** `crates/mycelium-l1/src/elab.rs` `elab_wild` lowers a `wild { name(args…) }` host-call body to `Op { prim: "wild:name", args }` (the reserved `wild:` namespace; a non-host-call body is an explicit elaboration `Residual` — never a fabricated lowering, G2). DN-14 row 9 moves from *conditionally present (execution staged)* → **present (executes)**.
- **M-721 — host dispatch + three-way differential (`Empirical`).** The L1 surface evaluator (`eval.rs` `eval_wild`) dispatches a `wild:` op through the prim registry; the L0 interpreter and the AOT env-machine already dispatch `Op` through the *same* registry, so a deterministic `wild`-backed op now agrees **L1-eval ≡ L0-interp ≡ AOT** (new `wild_ffi_execution_agrees_three_ways` differential, validated by the shared M-210 checker). The default registry grants **no** `wild:` op, so an ungranted host op is an explicit, never-silent `UnknownPrim` whose message names the ungranted capability (G2; `crates/mycelium-interp`). Real syscalls stay `Declared`; the differentially-covered op is `Empirical` (VR-5).
- **M-722 — `mycelium-std-sys` gains `io` + `sys` modules** (`Declared`): standard-stream I/O (stdin/stdout/stderr, never-silent `write_all`) and process/env (`exit`, `get_env` → explicit `Option`, `args`). The crate stays a pure-std leaf (`#![forbid(unsafe_code)]`, no workspace deps); `fs`/`rand`/`time` already provided real floors. Each op carries a guarantee-matrix doc row (RFC-0016 §4.5). The host-registration *bridge* wiring these into the `wild:` dispatch is specified (RFC-0028 §4.3/§4.5) and proven via the mock differential — the real-op wiring (a host layer depending on both `mycelium-interp` and `std-sys`) is the next incremental step (honestly staged; M-722/M-723 stay `in-progress`, VR-5).
- **M-724 — `just safety-check` extended to a Mycelium-level `wild`-site audit** (`scripts/checks/safety.sh`): in addition to the Rust `// SAFETY:` adjacency gate (M-681), every `wild` block in a shippable `.myc` nodule must be in a `@std-sys` nodule, inside a fn declaring `!{ffi}`, and carry a `// SAFETY:` comment — a gate, not a lint (G2). The grammar-conformance corpus is excluded (parser fixtures, validated by checker tests). Forward-looking (no shippable `.myc` `wild` sites yet); green.

### Added (2026-06-23: E12-1 — runtime & concurrency execution maturity, M-709/M-711/M-713)

- **Real OS-thread scheduler (`mycelium-std-runtime::scheduler`; M-709).** The v0 R1 surface ran tasks
  cooperatively on the calling thread; the new `Scheduler` runs independent tasks across a fixed pool
  of OS worker threads (`std::thread::scope` — the crate stays `#![forbid(unsafe_code)]`, reusing
  `mycelium-interp` for supervision primitives per DRY) with **fair FIFO dispatch** and **demand-signalled, bounded backpressure**: the ready
  queue holds at most `capacity` pending jobs *by construction* (enqueue only while `len < capacity`),
  never an unbounded silent buffer (G2 / RFC-0008 §4.3). `run_indexed` returns outputs in spawn order,
  so the result is directly comparable to the sequential reference — the **RT2 sequentialization
  differential**, property-tested (`Empirical`, not `Proven`). Zero workers / zero capacity fail closed
  (explicit `SchedulerError`, never a silent single-worker substitution). Backpressure bound is `Exact`
  (structural); liveness (each job runs exactly once) is `Empirical`. (M-709; RFC-0008 RT1·RT2·§4.3)
- **Deadlock-freedom for communicating tasks (`mycelium-std-runtime::dataflow`; M-711).** Communicating
  tasks are **swept** (one non-blocking poll-step per pending task per sweep); a full sweep that makes
  no progress while tasks remain pending is an explicit `task::Deadlock`, **never a silent hang** (G2 /
  RFC-0008 §4.3 sweep-order). The std-runtime sibling of the proven `mycelium-mlir` `run_dataflow`
  model. Two paths share the *same* never-silent decision: a cooperative `run_dataflow` (schedule is
  `Exact` — a fixed function of the sweep direction; ascending/descending agree — Kahn-determinism) and
  `run_dataflow_scheduled`, which runs each sweep's independent polls **across the M-709 OS-thread pool**
  (the "checked across OS threads" requirement). Detection is complete for DAG channel graphs (`Empirical`);
  cyclic graphs are an explicit open follow-up (FLAG: ADR-020 §7), never silently mis-reported. (M-711)
- **Structured-concurrency supervision + cancellation (`mycelium-std-runtime::supervision`; M-713).**
  Reuses the M-356 composition kernel from `mycelium-interp` (`CancelToken`/`TaskOutcome`/`Supervisor`/
  `RestartIntensity`/`Escalation` — DRY, no cycle: `mycelium-interp` depends only on core+numerics).
  Adds: a cancellation **tree** (`CancelTree`) where cancelling a node cascades to every descendant
  (parent→child, never child→parent — RT7), so a cancelled colony propagates to all its children
  (never-silent, G2); `run_supervised`, which runs a task set on the OS-thread pool, collects **every**
  child's explicit `TaskOutcome` (no dropped/silent variant — RT4/I1), and on the first failure cancels
  the remaining siblings (cooperative observation, never-silent); and `supervise_with_restart`, a live
  bounded-cascade restart policy that is **EXPLAIN-able** — each decision is a reified `SupervisionRecord`
  (no black boxes, ADR-006), escalating explicitly when the rate or total-cascade bound is hit, never an
  unbounded storm. Propagation/collection are `Empirical` (property-tested); the restart bound is `Exact`
  (inherited from `Supervisor`). (M-713; RFC-0008 RT4·RT7·§4.7)
- **Guarantee matrix extended** with six E12-1 rows (scheduler RT2 / backpressure / liveness; deadlock
  sweep; supervision propagation / restart bound), asserted in tests — no tag upgraded without a checked
  basis (VR-5). Reserved-vocabulary guard still green (no `fuse`/`reclaim`/`tier`/`mesh`/… in op names).
- **Honest deferrals (VR-5/G2 — flagged, not guessed):** **M-710** (runtime vocabulary execution —
  `fuse`/`reclaim`/`tier` elaboration) stays **blocked on M-667** (the L1 surface constructs, still
  `needs-design`): there is no L1 surface to elaborate yet. **M-712** (memory management & reclamation)
  stays **blocked on RFC-0027 reaching `Accepted`**, which RFC-0027 itself reserves for maintainer
  sign-off — not advanced unilaterally. E12-1 remains in progress; RFC-0008 stays `Accepted` (the
  machinery is implemented Rust-first; the spec is not silently moved). Verified with `cargo fmt`,
  `cargo clippy -p mycelium-std-runtime --all-targets -D warnings`, and `cargo test -p
  mycelium-std-runtime` (40 tests green). The full `just check` passes except `cargo audit`, which
  cannot fetch the RustSec advisory database in the offline build sandbox (a network limitation, not
  an advisory finding). (E12-1; M-709/M-711/M-713; M-710/M-712 deferred)
### Changed (2026-06-23: RFC-0029 → Accepted — native-AOT design gate cleared, E15-1 honestly re-scoped)

- **RFC-0029 (AOT Optimization, Codegen Maturity, and JIT) → Accepted** (was Draft; append-only — Draft row preserved). The seven §5 open questions are **resolved against the live `crates/mycelium-mlir/`** (checked 2026-06-23), not from the stub's assumptions. Substantive finding that re-scopes E15-1: **E6-1 is `done`**, so the JIT (M-340 `jit.rs`), BitNet packed-ternary accel (M-360 `bitnet.rs` — I2_S/TL1/TL2), real `arith`/`func`→LLVM **MLIR-dialect** lowering (M-601 `dialect/native.rs`), and the **three-way native differential** (M-602 `tests/threeway_differential.rs`) **already landed**; ADR-019 (libMLIR toolchain) is **Enacted**; and **ADR-009 already sanctions interpreter/JIT for dynamic VSA** (no superseding ADR needed to "lift a deferral"). New normative §7 sanctions: inlining/CSE/DCE as **EXPLAIN-able, never-silent** transforms reified as a transform-log (M-726 — **the one genuinely-new mechanism**, no `src/passes/` exists yet); JIT as an **explicit, never-silently-selected** first-class mode (M-727); the BitNet **explicit capability flag** + never-silent graceful degradation (M-728); and the **mutant-witnessed `interp ≡ AOT ≡ JIT`** durability gate (M-729). Honest tags throughout: `Empirical` for the existing differentially-checked paths, `Declared` for the as-yet-unbuilt optimization passes (VR-5 — no overclaim). RFC index + Doc-Index updated. (RFC-0029; E15-1)

### Added (2026-06-23: E17-1 docs tranche — language reference + tutorial, generated stdlib API docs, ADR-023 stability; M-738 release act BLOCKED)

- **M-735 — language reference + tutorial (`docs/reference/`).** A full-surface **language
  reference** (`language-reference.md`) covering lexical structure, nodules/phyla, the four
  representation types + ADTs + substrates, the guarantee-strength lattice, functions & effects,
  every expression form, pattern matching, the swap system, generics & traits, ambient paradigms,
  maturation/`thaw`, `wild`/FFI, the full keyword set, and the L0–L3 layer model — grounded in
  `mycelium.ebnf`, the conformance corpus, and `crates/mycelium-l1/src/token.rs`, with honest VR-5
  notes where surface type-checks-but-doesn't-run (generics/traits → M-673; effects checker-only →
  M-677). A **tutorial** (`tutorial.md`) builds a complete program whose full source is committed as
  the **parser-verified** conformance fixture `accept/20-tutorial-classifier.myc` (parsed by
  `mycelium-l1` `tests/conformance.rs` — examples are CI-checked, never invented). Plus a section
  index (`README.md`). Guarantee: `Declared`. (E17-1; M-735)
- **M-736 — generated per-module stdlib API docs (`mycelium-doc`).** Wired `lib/std/` into the
  `myc-doc` apiref build (`BuildInput::conventional`), so every self-hosted stdlib `.myc` nodule is
  projected into the API reference; added **per-`fn` source-comment extraction** (`apiref::preceding_doc`)
  so a function's preceding `//` block becomes its documented summary (traces to source, never
  invented — an undocumented `fn` stays an explicit flagged gap, G2). The whole source is captured as
  a *checked example* (type-checked). Today `std.result` is covered (the only self-hosting module;
  `map`/`and_then`/`fold` documented, `is_ok`/`is_err`/`unwrap_or` flagged undocumented); coverage
  grows as **E13-1** ports modules. `myc-doc lint` (part of `just check`) green:
  checked-examples 6→7, documented api statements 35→38. New committed reference page
  `docs/reference/stdlib-api.md`. Guarantee: `Declared`. (E17-1; M-736)
- **M-737 — ADR-023 stability & API-compatibility guarantees `Draft → Proposed → Accepted`.** All
  six §5 open questions resolved (append-only): **§3.1** four-layer stability scope (surface syntax,
  Core-IR/cert/interp, LSP wire, Rust crate API) with explicit carve-outs (codegen internals,
  reserved-not-active keywords); **§3.2** dual-version model — the full-language 1.0.0 is a
  *release-event* (`v1.0.0` tag + CHANGELOG + ADR-022 gate record), **not** a crate/workspace version
  (ADR-018 upheld), labelled distinctly from `core 1.0.0`; **§3.3/§3.3.1** release-based never-silent
  deprecation (≥ one minor, removal at 2.0.0) + no surface `@unstable` at 1.0; **§3.4** MIT-only legal
  gate. All §3 claims `Declared` (policy warrants no `Proven` — VR-5). Not Enacted (that is M-738 at
  the tag). (E17-1; M-737; ADR-023)
- **MIT-only license fix (ADR-023 §3.4 gate).** A repo-wide sweep of first-party *shipped* `.myc`
  headers found **six** non-MIT violations, all corrected to **`MIT`**: `lib/std/result.myc` and the
  five `examples/**` programs (`examples/repr-tour/{ambient,swaps,traits,iter}.myc`,
  `examples/hello-phylum/hello.myc`). The only remaining non-MIT `@license` strings are deliberate
  `crates/mycelium-proj/tests/fixtures/` test inputs (Apache + a deliberately-invalid SPDX id) that
  the `mycelium-proj` tests *assert* (non-inheritance + bad-id error) — left as-is by design.
- **M-738 — full-language 1.0.0 release act: BLOCKED (no tag cut).** The terminal release act is
  **not** performed — its external gate deps are unmet: **E13-1** (self-hosting stdlib) and **E18-1**
  (full-language readiness) are both `needs-design`, and **ADR-022** is `Accepted` (not `Enacted`)
  with sub-gate rows A2/A3/A4 still open. Per house rule #3 / G2, **`v1.0.0` is not cut**, ADR-021/
  ADR-022 stay `Accepted`, and the changelog stays `[Unreleased]` — recorded explicitly rather than
  forced prematurely. M-738 fires only when E13-1 + E18-1 + every ADR-022 row close. (E17-1; M-738)
### Changed (2026-06-23: ADR-022 Q4 — T6 native AOT un-gated from 1.0.0 → 1.1)

- **ADR-022 Q4 resolved** (maintainer): **T6 (native AOT maturity / optimization passes / JIT / BitNet accel — epic E15-1) is un-gated from `1.0.0` and rolled to `1.1`** as a QoL/perf enhancement, patched in after release. `lang 1.0.0` ships on the **interpreter (trusted base) + the existing direct-LLVM kernel subset** — optimized native codegen is performance, not correctness. Removed T6 from the ADR-022 §5 gate + §3 scope; updated DN-25 (graph + waves), the `aot10` kickoff row (→ `1.1`/post-1.0.0), and E15-1's DoD scope note. Net: nothing perf-related sits between `lib10` (T4) and the release tag; `aot10` now runs post-1.0.0 alongside `boot10`.
### Changed (2026-06-23: ADR-022 track T1 status refreshed — core/kernel gate-met, tag-ready; kickoff c10)

- **ADR-022 §4/§5 track-T1 status refreshed: `A1·A2·A3·A4·A5 ✅ · B1·B2 ✅` — GATE-MET / TAG-READY.** The three rows that read "⏳ open" at the 2026-06-23 supersession (A2 Medium ledger, A3 WS8 durability, A4 supply-chain) were in fact closed by the original ADR-021 kernel-gate wave (2026-06-21), and the tooling remains present: **A2** — the Medium-findings ledger is 25/25 Fixed, 0 deferred (`docs/reviews/2026-06-14-deep-review/06-medium-findings-ledger.md`; M-653/#306); **A3** — `cargo-mutants` 0 un-triaged survivors (`.cargo/mutants.toml`) + LCG→`proptest` migration + `cargo-fuzz` targets/smoke CI (`fuzz/`; M-654/#313); **A4** — `cargo deny`/`cargo audit` wired **non-silently** into `just check` (`deny.toml`, `scripts/checks/`; M-652/#303). **Honesty (VR-5): this is a status report moving forward on a checked prior landing, not a fresh gate run** — the maintainer ratified T1 as satisfied (refresh-status scope), no re-verification was performed this session.
- **issues.yaml status reconciled (E10-1 / T1):** M-700 (A2 ledger), M-701 (A3 durability), M-702 (A4 supply-chain) → `done` (closed **by reference** to M-653/M-654/M-652 with explicit `landed_basis`); **E10-1** → `in-progress` (gate-met, tag pending). **M-703** (the release act) → `todo` and **MAINTAINER-RESERVED** (overlaps the pre-existing M-655); its title/body corrected — ADR-021 is **Superseded** so it cannot move Accepted→Enacted, and the inherited kernel-gate enactment now attaches to **ADR-022 track T1** at the tag (append-only, house rule #3).
- **Tag-ready hand-off.** No `core 1.0.0` tag was cut and no decision was enacted this session (maintainer-reserved boundary, M-703/M-655). The substrate is tag-ready; the tag + T1 enactment remain the maintainer's act. (kickoff `c10`; ADR-022; E10-1)
### Added (2026-06-23: M-708 / E11-1 — surface stabilization declaration + M-706/M-707/M-704 honest scoping)

- **Surface Stability Declaration (M-708; `docs/spec/Surface-Stability-Declaration.md`).** A stage-1
  surface audit consolidating DN-14 §3, the checker's own refusal comments, and the RFC residual
  sections: **12 features declared present** (value types, ADTs+patterns, recursion, generics
  [checked + monomorphized], traits+coherence, effects, `wild`/FFI [gated], phyla, grading stage-1a,
  `colony`/`hypha`, static HOF, operators) each with a source/test ref, and the **deferred set**
  enumerated with a never-silent refusal + a forward issue ref for each (dynamic HOF → M-704;
  angle-bracket operators → M-745; effect→budget wiring → M-677; `consume`/`grow`/inherent-`impl` →
  M-664; R1/R2 runtime vocab → M-667/M-668; VSA/Substrate + `wild` execution → RFC follow-ups). The
  audit found **no silent-incorrect surface form** — every refusal is an explicit
  `CheckError`/`Residual`/parse error (G2). Advisory, no normative move, no tag upgrade (VR-5).
- **RFC-0030 partial decision (M-706; stays Draft).** Integrated the M-705 operator grammar into
  `mycelium.ebnf`; **proposed** the RFC-0006 **Q8** resolution = ratify `wild { … }` as the
  unsafe-class spelling (gated by `@std-sys` + `!{ffi}`; ratifies the implemented M-661 status quo;
  `unsafe`/`audited` alternatives declined); **corrected** the stub's **Q3** mischaracterization
  (RFC-0006 Q3 is the LR-6 guarantee-grading mechanism — discharged by RFC-0018; the representation
  question is RFC-0012's — so there is no open Q3 here). Draft → Proposed remains **gated on M-707 +
  M-745** — a complete ratified L3 grammar cannot be honestly claimed until the RFC-0020 carve-outs
  and the angle-bracket operators land (VR-5 / house rule #3).
- **Honest deferrals recorded (M-704, M-707).** **M-704** (dynamic HOF): the RFC-0024 §5 residuals
  (closures, dynamic fn-flow, partial application, generic-fn-as-value) are catalogued in the
  stability declaration §3.2 — each a never-silent `Residual` in `mono.rs` today; the issue stays
  open for the implementation (full Reynolds defunctionalization under the §5 STOP-and-flag KC-3
  guard). **M-707** (RFC-0020 enactment): the §4.2/§4.5/R20-Q1…Q5 carve-outs are explicitly
  **re-deferred** (forward ref = M-707; RFC-0020 stays "Accepted (scoped)", no status change —
  append-only); the polymorphic-instantiation/operator-sugar interaction is confirmed independent
  (sugar desugars to `App` before inference). Doc-Index / rfcs README updated; no code change.

### Added (2026-06-23: M-705 / E11-1 — operator syntax: infix sugar desugaring to word functions)

- **Operator syntax — symbolic infix/prefix sugar (RFC-0025 → Proposed; M-705).** Mycelium's
  surface gains optional symbolic operators that desugar **at parse time** to the canonical word
  functions: `a + b` → `add(a, b)`, `a * b + c` → `add(mul(a, b), c)`, `-a` → `neg(a)`, `!a` →
  `not(a)`. The desugaring is **frontend-only** — a pure syntactic rewrite producing the same `App`
  AST as the word call — so **`mycelium-core` is untouched and there is no new L0/L1 node (KC-3)**.
  The word form stays valid everywhere the sugar is (the sugar is **additive** — words are
  canonical). Lexer (`crates/mycelium-l1/src/lexer.rs`) gains the operator tokens
  (`Minus`/`Slash`/`Percent`/`Caret`/`Amp`/`AmpAmp`/`PipePipe`/`EqEq`/`BangEq`; `Plus`/`Star`/
  `Pipe`/`Bang` become context-dual — bound/glob/pattern/effect *and* operator); the parser gains a
  precedence-climbing layer (`parse_binexpr`/`parse_unary`, with `infix_op`/`op_call`). **Precedence
  & associativity follow Rust's table** (RFC-0025 §4.1; the implementation language, cited
  explicitly): unary (tightest) → `* / %` → `+ -` → `&` → `^` → `|` → `== !=` → `&&` → `||`; all
  binary operators left-associative, prefix right-associative. **EXPLAIN (resolves RFC-0025 Q5):**
  the desugared `App` node *is* the audit record — no separate `DesugarRecord` (ADR-006, no black
  boxes). **Honesty (G2/VR-5):** the desugaring is purely syntactic; `add`/`sub`/`mul`/`xor` (and
  unary `neg`/`not`) resolve to kernel prims **today** and are pinned end-to-end across all three
  execution paths (L1-eval ≡ L0-interp ≡ AOT) by new `tests/differential.rs` entries (**Empirical**);
  the other targets (`div`/`rem`/`band`/`bor`/`eq`/`ne`/`and`/`or`) parse + desugar but surface an
  **explicit** "unknown prim" refusal downstream (never silent) pending their stdlib/kernel defs.
  Grammar `docs/spec/grammar/mycelium.ebnf` extended (`op_expr` … `unary_expr`); conformance
  fixture `accept/20-operator-syntax.myc` added. **Deferred → M-745:** the angle-bracket operators
  `< <= > >= << >>` (their `<`/`>` collide with the type-argument `<…>` grammar — RFC-0025 §4.3).
  **RFC-0025 → Proposed** (no tag upgraded — VR-5; Proposed → Accepted awaits maintainer
  ratification, house rule #3). (M-705; RFC-0025; E11-1)
### Added (2026-06-22: M-662 — `phylum` construct + cross-nodule model (E7-1))
- **`mycelium-l1` now parses + type-checks the `phylum` construct and the cross-nodule model** (RFC-0006; DN-06 §6; RFC-0019 §4.5). A source file may open with an optional **`phylum <path>`** header grouping **multiple `nodule` blocks** (`program ::= phylum_header? nodule_block+`); a header-less single nodule is a **phylum-of-one** — backward-compatible (`check_phylum(of_one) ≡ check_nodule`). Top-level `fn`/`type`/`trait` take an optional **`pub`** marker (default **private-to-nodule**; `pub` exports phylum-wide), and **`use`** imports a name across nodules — **specific** (`use a.b.X`) or **glob** (`use a.b.*`, the `pub` surface under a prefix). Two **distinct phylum-wide views** are kept separate: a pub-only **import registry** (`use` resolution) and a pub-blind **coherence view** (the orphan rule). **Never-silent (G2):** a `use` of an **absent** name ("no such name") vs a **private** name ("exists but is not `pub`") are *distinct* explicit `CheckError`s; a **duplicate** explicit import refuses; resolution precedence is local-decl > explicit-`use` > glob (deterministic documented shadowing), and a **referenced glob-vs-glob** collision is an explicit ambiguity (latent until referenced) — never a silent winner. The **RFC-0019 §4.5 orphan rule is generalized phylum-wide** (landing the cross-nodule enforcement M-659 staged): an `impl` is legal iff its trait OR `for`-type head is declared in **some nodule of the phylum** (pub-blind), else a never-silent orphan refusal. The surface printer (`expand_phylum_to_source`) re-emits the header + `pub` + both `use` forms (round-trip stable; a header-less phylum never sprouts a `phylum` line). **No new L0 node** (KC-3) — phylum is a parse/check container; multi-nodule type-checking is real, cross-nodule execution stays staged where needed, and ambient resolution stays per-nodule. **Guarantee `Declared`** (coherence is Declared-with-argument per RFC-0019 §4.5 — not upgraded, VR-5). Grammar oracle updated (`mycelium.ebnf` phylum/`pub`/`use`-glob; `conformance/accept/19-phylum-cross-nodule`; the former reserved-not-active `reject/10` repurposed to the phylum-no-nodule parse refusal; the conformance gate now oracles via `parse_phylum`). RFC-0006 / RFC-0019 §4.5 / DN-06 carry append-only "implemented Rust-first, pending ratification" notes; **DN-14 §3 row 10 → `present`**. **247 `mycelium-l1` tests green** incl. the phylum suite (cross-`use` accept; absent/private/duplicate/glob-ambiguity refusals + non-vacuous controls; own-decl-shadows-glob; phylum-wide orphan accept + both-outside reject + a direct `checkty` orphan-arm unit test; round-trip; phylum-of-one ≡ `check_nodule`) + the conformance gate. The decisions were maintainer-confirmed (single-file v0 · `phylum <path>` header · `pub` + wildcard `use` · qualified per-phylum registry · orphan-at-registration). (M-662; E7-1)

### Added (2026-06-22: DN-21 — Unsafe-Code Hardening Survey + the M-678 hardening epic)
- **`docs/notes/DN-21-Unsafe-Code-Hardening-Survey.md`** — a planning capture (advisory, DN-17 posture): a grounded, read-only audit of every `unsafe` block in the workspace (exactly **6**, all dynamic-linking FFI in `crates/mycelium-mlir` — `jit.rs` dlopen/dlsym/dlclose/transmute-call, `bitnet.rs`, `specialize.rs`; **29 crates** are `#![forbid(unsafe_code)]`, the interpreter trusted base unsafe-free), confirming each carries an adequate ADR-014 `// SAFETY:` justification (3 thin, none wrong). Identifies the `BitnetDotKernel` `*mut c_void` co-location dangling-pointer risk as the top target and scopes a behaviour-preserving, **zero-new-dependency** hardening epic — **M-678** → M-679 (SAFETY-comment + `debug_assert` hardening) · M-680 (forbid-pin the trusted base + the 11 zero-unsafe mlir submodules) · M-681 (`just safety-check` SAFETY-adjacency gate) · M-682 (in-house `Sym<'lib,T>` lifetime-binding newtype, maintainer-chosen over `libloading`) · M-683 (document the `audit_wild` `.myc` vs `clippy` `.rs` scope split) — enacting ADR-014's named follow-ons. §7 names the irreducible unsafe floor honestly (calling JIT'd fn-ptrs / `dlopen` ctor / the ABI claim); the SIMD stays in JIT-compiled LLVM IR by design (FR-C3). No code changed. (DN-21; M-678)

### Added (2026-06-22: M-661 — `wild`/FFI floor typechecking in the `@std-sys` context (E7-1))
- **`mycelium-l1` now type-checks the audited `wild`/FFI floor, gated to an explicit `@std-sys` nodule** (RFC-0016 §8-Q6; LR-9/S6; ADR-014). `@std-sys` is a **nodule-header attribute** (`nodule std.sys.fs @std-sys`), **not** a naming convention — lexed as one **atomic** `Tok::AtStdSys` (the `-` can't lex as `@`+ident; a narrow whole-word match, so `@std`/`@ Exact`/`@std-system` are unaffected), parsed into `Nodule.std_sys`, and threaded through ambient resolution + the resolved twin down to the checker. `checkty.rs` `Cx::check_wild` admits a `wild { body }` block **iff** (a) it is inside a `@std-sys` nodule **and** (b) its enclosing `fn` declares the **`ffi` effect** — `wild` is the `ffi` effect *source*, fed into the M-660 coverage pass — so a `wild` anywhere else is a **hard `CheckError`** (never a silent escape — G2; the issue's "lint warning" wording is **amended to a hard refusal**, cleaner LR-9). The `wild` **body is the trusted/opaque FFI escape — NOT recursively type-checked** (it conforms to the expected type by ascription; *audited, not verified* — VR-5/ADR-014), so a synthesis position refuses with "ascribe the `wild` block's result type". The gate flows into **impl-method bodies** too (an impl in a non-`@std-sys` nodule may not contain `wild`). **Execution stays staged** (no FFI host in v0): `elab.rs` lowers `wild` → an explicit `Residual` and `eval.rs` refuses with a staged message — never a fabricated value (G2), consistent with the M-657/659/660 staging. **Guarantee `Declared`** (a structural + audited *context* gate, not a theorem — VR-5). The `myc-sec` `// SAFETY:`-presence audit (`audit_wild`, ADR-014) is **orthogonal + unchanged**. **No `unsafe` added** (`#![forbid(unsafe_code)]` holds). RFC-0016 §8-Q6 + RFC-0006 LR-9 carry append-only "implemented Rust-first, pending ratification" notes (supersede nothing); **DN-14 §3 row 9 → `conditionally present (audited, std-sys context; type-checks + gates; execution staged)`**. 217 `mycelium-l1` tests green incl. 10 `wild` cases (context gate, opaque body, ffi-coverage refusal, synthesis-demands-ascription, ascribed-let accept, impl-method gating, staged elaboration, attribute-not-convention) + lexer/parser unit tests + conformance `accept/18-wild-std-sys.myc`. (M-661; E7-1)

### Added (2026-06-22: M-660 — effect annotations on fn signatures + coverage check (E7-1))
- **`mycelium-l1` now parses + checks effect annotations on fn signatures** (RFC-0014 §3.4/§4.5; maintainer-pinned surface). A fn (or trait method) may carry an **optional** `!{eff1, eff2}` after its return type — `fn f() -> T !{io}` (Koka-style `!`; effect names are plain identifiers = the kernel kinds `retry|alloc|io|cascade|time` + user `Named`; **absent ⇒ pure** per RFC-0014 I5; a duplicate effect name is a never-silent **parse** refusal). AST `FnSig.effects` + `Tok::Bang`. The **effect-coverage** pass enforces **declared ⊇ performed** — where *performed* = the union of every callee's declared effects (a top-level fn **or** an unqualified trait method), checked over **fn bodies and impl-method bodies** so an effect can never be hidden from a caller (the core RFC-0014 invariant). **Under-declaration is an explicit `CheckError`** (G2/I3) naming the effect + callee; **over-declaration is allowed** (a declaration is a contract — I5); an impl method's effects must equal the trait method's. **Guarantee `Declared`**. **No new L0 node** (KC-3) — effects are checker metadata; they do not lower. **Deferred (flagged):** the `mycelium-interp::budget` runtime wiring + per-effect budget syntax (`retry(<=3)`) → **M-677**; `wild`-sourced effects → M-661. RFC-0014 §3.4 pins the surface (append-only, still `Enacted`); **DN-14 §3 row 8 → `present`**. 203 `mycelium-l1` tests green incl. a monotonicity property sweep, trait/impl effect-conformance, and a soundness regression for trait-method/impl-body coverage; `accept/16-effect-annotations.myc` / `reject/17-duplicate-effect.myc`. (M-660; E7-1)

### Added (2026-06-22: M-659 — stage-1 trait/impl checker + coherence (E7-1))
- **`mycelium-l1` now type-checks `trait`/`impl` declarations with coherence, and resolves bounded-generic + trait-method calls** (RFC-0019 §4.1/4.4/4.5, RFC-0007 §12). The checker builds trait/instance registries and enforces — all as explicit **never-silent** `CheckError`s (G2): **global uniqueness** (≤1 instance per `(trait, type-head)`), the **single-nodule orphan rule** (cross-nodule enforcement staged → M-662), exact **method-set** conformance (a missing or extra method is refused), and **bounded-call satisfiability** (`T: Cmp` requires a resolvable instance; an unqualified trait-method call resolves via a concrete instance or an in-scope bound, ambiguity/undetermined are explicit errors). The parser gains `impl Trait<…> for T { … }`, bounded fn type-params `<T: Cmp + …>` (a bound on a `type`/`trait` param is an explicit refusal), and `Tok::Plus`; the AST gains `Item::Impl` + `TypeParam`/`TraitRef`. The single-parameter **self-bound sugar** `T: Cmp ≡ T: Cmp<T>` is supported (multi-param/ambiguous bounds are an explicit error, never a guess). **Guarantee: `Declared`** — coherence is Declared-with-argument per RFC-0019 §4.5, not machine-checked (VR-5). **Dictionary-passing L0 lowering is STAGED** as an explicit `Residual` → **M-673** (identical to the M-657 generics staging; no new kernel node — KC-3). 187 `mycelium-l1` tests green incl. a coherence property sweep + `accept/14-trait-impl.myc` / `reject/15-trait-param-bound.myc`. RFC-0007 §12 + RFC-0019 carry append-only "implemented Rust-first, pending ratification" notes; **DN-14 §3 row 7 → `partial`** (type-checks; elaboration staged). (M-659; E7-1)

### Added (2026-06-22: track-a — issue↔PR↔sub-issue relationship + date manifest; `gh-issues-sync.py` extraction automation)
- **`tools/github/` now carries a grounded issue/PR relationship + landing-date manifest plus the automation to regenerate it.** `gh-issues-sync.py` gains a `--relationships` mode that derives the issue↔PR map + dates from the two in-repo sources of truth — dated `CHANGELOG.md` headers and `git log origin/main` squash subjects (`(#NNN)`) — cross-checked against the live merged-PR list, and enriches `issues.yaml` **additively**: 88 `landed_pr`/`landed_date` (on `status:done` only), 8 weaker `evidence_pr`/`evidence_date` (not-done), 35 `epic:` edges, each with a `landed_basis` citing its evidence. **Honesty (G2/VR-5):** a strong `landed_*` is asserted only for done issues (referenced-in-a-PR ≠ completed — verified 0 violations); every date/PR is `Empirical`/`Declared` with a cited basis; no field is null-guessed; existing fields are never rewritten. New `pr-index.json` (issue↔PR from the 39 merged PRs) lets the offline cross-check reproduce without a token. A `gh`-CLI-independent REST + GraphQL(Projects-v2) client (stdlib `urllib`, env token, **opt-in `--use-api`**, every mutation `--dry-run`-gated, never-silent, no token in any log) is wired + self-test-covered for the maintainer's token-scoped sync — live Projects-v2 population is honestly token-gated, not faked. Sub-issue links **E7-1 ← M-656, M-658** synced live (MCP). Full `idmap.tsv` reconciliation tracked as **M-675**. (track-a)

### Changed (2026-06-22: M-674 — evaluator runs on the deep worker stack; depth budget is the ceiling)
- **The L1 evaluator (`Evaluator::call`) now runs on the deep, lazily-committed worker stack**
  (`mycelium_stack::with_deep_stack`), so the **explicit recursion-depth budget** — not the caller's
  thread stack — is always what bounds a pathological input. `DEFAULT_DEPTH = 64` is unchanged (no
  behavior change), but raising it via `with_depth(N)` for deep runtime computation (recursive folds
  over large dense/VSA structures — the dense-embeddings/HDC axis) is now **host-stack-safe**: the
  budget refuses cleanly with `DepthExceeded` well before any physical limit (estimated ~65k–130k
  levels on the 256 MiB worker in debug). `SwapEngine` is tightened to `+ Send + Sync` (engines are
  `Copy` unit structs — no call-site change) so the scoped-thread closure is `Send`. Clean-refusal
  regression test added. Extends the uniform banked-guard-4 discipline (M-674) to the evaluator; the
  budget is the portable primitive for the self-hosted frontend, the worker stack the transitional
  Rust-host adapter. (M-674; RFC-0007 §4.6)

### Changed (2026-06-22: M-658 — RFC-0007 §12 trait surface + `impl` reserved)
- **RFC-0007 §12 (append-only) pins the stage-1 trait / bounded-generics surface** `mycelium-l1` v1
  must check (single-parameter `trait`/`impl Trait for T` declarations + **coherence** = orphan rule +
  global uniqueness, per RFC-0019), and **reserves `impl` as a lexer keyword** (`token.rs` `keyword()`
  → `Tok::Impl`) — never a silent identifier (G2). Reject-corpus fixture
  `reject/14-impl-reserved-ident.myc` (self-policed by the conformance table). Elaboration is **staged
  identically to §11.3**: dictionary-passing *types-check* in the checker (M-659); the L0 lowering of an
  instantiated dictionary is a never-silent `Residual` until monomorphization (M-673, generics + traits
  together). Multi-parameter traits / associated types / Repr-polymorphism stay deferred (RFC-0019
  §10). DN-14 §3 row 7 captured (no flip — only the M-659 checker flips it). No v0 calculus change; no
  new kernel node (KC-3). (RFC-0007 §12; RFC-0019; DN-14 §3 row 7; M-658, E7-1)

### Added (2026-06-22: M-657 — stage-1 unbounded generics checker + depth-safe recursion)
- **Stage-1 unbounded parametric generics — checker (`mycelium-l1`).** `type List<A> = …`, generic
  functions (`fn first_or<A>(xs: List<A>, d: A) -> A`), and **call-site instantiation by
  unification** now type-check (RFC-0007 §11): a type parameter is an abstract `Ty::Var`,
  constructor/match field types are instantiated by substitution, and the result type is inferred
  from the arguments. Honest refusals, never a guess (G2/VR-5): wrong type-argument **arity**, an
  **undetermined** type parameter, and a **representation-specific op on a type parameter** (the
  RFC-0019 §4.6 Repr-polymorphism restriction — `Var` is representation-opaque, so a paradigm-specific
  prim/`swap` cannot apply to it; S1 enforced at the checker). **L0 elaboration of a generic
  *instantiation* is staged** behind an explicit never-silent `Residual` ("monomorphization staged");
  a monomorphic program elaborates unchanged. DN-14 §3 row 6 → **partial (type-checks; elaboration
  staged)**, not `present` (a stdlib nodule that instantiates a generic does not yet self-host to L0).
  RFC-0007 §11.2–§11.4 corrected to the as-implemented split (M-656's "uniform elaboration" wording
  superseded). No new kernel node (KC-3). (RFC-0007 §11; RFC-0019; DN-14 §3 row 6; M-657, E7-1)
- **Depth-safe checker/elaborator recursion (`mycelium-l1`).** The checker no longer relies on the
  caller's thread-stack size to bound its recursion (a fragile coupling — a wider `Ty` had reduced the
  implicit margin). It now carries an **explicit reified budget** (`MAX_CHECK_DEPTH = 4096`, the
  "banked guard 4" discipline — a clean `CheckError` past it, never a crash) and runs on a **deep,
  lazily-committed worker stack** (`crate::deep`, 256 MiB) so deep-but-valid input never overflows the
  caller's stack. The budget is **measured-safe**: the worker stack physically supports ~24,600 levels
  in debug (empirically; ~10.9 KiB/frame), so 4096 is a ~6× margin and 16× above the parser's 256-deep
  surface cap. The explicit budget is the **portable primitive** (it carries to the future
  Mycelium-native self-hosted frontend's clocked bounded-computation model — RFC-0007 §4.6); the
  worker stack is a transitional Rust-only adapter. Elaboration runs on the same deep worker.

### Changed (2026-06-22: M-656 — RFC-0007 §11 discharges the §4.4 generics deferral)
- **RFC-0007 §11 (append-only) discharges the §4.4 "polymorphism deliberately out of v0" deferral**,
  routing it to its destination RFC — **RFC-0019 (Accepted)** — and pinning the minimally-sufficient
  **stage-1 generics surface** that `crates/mycelium-l1` v1 must check: (a) **unbounded** parametric
  generics (`type List<A>`, `fn head<A>`, `fn map<A,B>`), type parameters as abstract variables
  (M-657); (b) **bounded** generics + traits via RFC-0019 dictionary-passing (M-658/M-659). The
  §4.4 "instantiating a generic is a deferred error" sentence is **superseded** by §11.3 — the
  refusal becomes a checked pass, **never a guess** (VR-5/G2). The never-silent-swap obligation
  (S1/W8) is restated at the polymorphic level: instantiation never inserts a `Swap`; a
  Repr-polymorphic body that would need one is an explicit `UnresolvedReprPolymorphism`. **No v0
  calculus content changed**; the amendment leans on RFC-0019's **Declared-with-argument** results
  and does not upgrade them (§11.5). DN-14 §3 row 6 gate captured. This is the **spec gate** that
  unblocks the M-657 checker/elaborator implementation. (RFC-0007 §11; RFC-0019; DN-14 §3 row 6; M-656, E7-1)

### Changed (2026-06-21: RFC-0022 + RFC-0023 ratified Draft → Accepted)
- **RFC-0022 (Web-Tooling Phylum) and RFC-0023 (ADK Phylum) ratified to Accepted** by the maintainer
  after the Phase-2 deep-research discharge (RP-10 / RP-9; `dfr` session). RFC-0023's §3 concept-map was
  **repaired at ratification** (new **§3.7** mapping ADK 2.0's graph **Workflow Runtime**, operating
  `mode`s, code-Router, and `RunConfig.max_llm_calls`→`TaskOutcome::BudgetExhausted`; §3 pinned to
  `adk-python` **v2.3.0**), closing the one open completeness item. Ratified design decisions:
  `web.server` + `adk.runner` run on **`mycelium-mlir::runtime`** (no external executor); `ToolError`
  keeps `BadArgs|OutOfDomain|Refused|Upstream` with budget on `TaskOutcome::BudgetExhausted`;
  Session/State is snapshot-v0 with concurrent merge deferred to `fuse`; IDNA pinned at build
  (nontransitional, fail-closed); HTTP/2-3 + TLS + the cross-peer smuggling model are named v1
  non-goals; LLM-leverage stays **no-verdict**. **Accepted = design agreed; Enacted gated on the
  `mycelium-web` / `mycelium-adk` builds + E7-1/E7-2.** Append-only; status trails preserved in each
  RFC's Status cell. Empirical/Declared, never Proven (VR-5). (RP-9/RP-10; M-670/M-671.)

### Added (2026-06-21: M-666 — RFC-0008 R1 `hypha`/`colony` real-concurrency L1 constructs)
- **RFC-0008 R1 `hypha`/`colony` L1 surface constructs** (`mycelium-l1`, `mycelium-mlir`):
  `colony { hypha compute(x), hypha compute(y) }` now parses, type-checks, and lowers to
  **real concurrency** via the M-357 runtime (`run_colony` spawns hyphae as concurrent Tokio Tasks,
  `run_interleaved` provides the sequential reference). RT7 invariant enforced: a `colony` block
  cannot exit before all `hypha` children complete. **RT2 differential validated** — real-concurrency
  output ≡ sequential reference for deterministic computations; determinism is **Empirical** (not
  Proven). Divergence, failure, and empty-colony are explicit `ColonyError` variants (never-silent,
  G2). `colony` and `hypha` are now **active keywords** in the grammar (previously
  reserved-not-active after M-665). Implemented **Rust-first; RFC-0008 pending ratification
  (Accepted, not yet Enacted)** — the trusted base stays sequential (RFC-0008 §4.2/§4.7; no L0
  node added). LSP cross-crate fix: `colony` and `hypha` added to `mycelium-lsp` keyword
  completions (and the `colony-block` snippet); removed from the `reserved_not_active` ban-list.
  (RFC-0008 §4.5/§4.7; M-666, E7-2)

### Added (2026-06-21: M-672 — docsite lang-ref page + honesty fix for reserved runtime keywords)
- **Language-reference autogen page** (`scripts/docsite.sh`, `just docs-site`): the generated
  lang-ref landing page now includes EBNF productions, the three reserved-word tables, the 25-entry
  stdlib spec table, and design-doc links. Closes M-672.
- **CRITICAL honesty fix** (`scripts/docsite.sh`): the "Ratified, not yet lexed / lex as ordinary
  identifiers today" table previously listed all 10 DN-03 §4 runtime terms (`hypha`, `fuse`, `mesh`,
  `graft`, `cyst`, `xloc`, `forage`, `backbone`, `tier`, `reclaim`) as lexing as ordinary identifiers
  — **this was false** after M-665 (landed on `origin/main`, 2026-06-21) reserved all 10 in
  `keyword()`. The 10 runtime words now appear in the **"Reserved-not-active"** table (alongside
  `phylum`/`colony`), with description "reserved keyword — produces a ParseError; not yet active as a
  construct (RFC-0008 §4.5)". The "not yet lexed / lex as identifier" category now contains only
  `impl`, `consume`, `grow` — verified against `crates/mycelium-l1/src/token.rs` `keyword()`
  post-M-665 (G2: never-silent; honesty rule / VR-5).
### Added (2026-06-21: RFC-0023 — ADK-port phylum Draft + research/13-adk-phylum-RECORD.md + RP-9)
- **RFC-0023 — Agent Development Kit Phylum** (`docs/rfcs/RFC-0023-Agent-Development-Kit-Phylum.md`):
  **Draft** design for an `adk` phylum — a Mycelium port of Google's Agent Development Kit, with
  **honesty-as-differentiator**: every tool/model output tagged Declared/Empirical, never Proven;
  every tool call a never-silent `Result<Out, ToolError>`. Nodules: `adk.agent` (the ADK Agent
  construct mapped to `colony`/`hypha`), `adk.tool` (typed `fn -> Result`, never-silent dispatch),
  `adk.session` (content-addressed value state, ADR-003), `adk.runner` (orchestration), `adk.model`
  (LLM harness reuse — `GrokLlmReport`, no second model-call path). v1 Rust-first as `mycelium-adk`.
  **Gated**: `Tool<In,Out>`/`Agent` generic surface blocked on E7-1 (M-657 generics, M-659 traits);
  `.myc` `colony { hypha … }` agent surface blocked on E7-2 (M-666); model transport reuses M-670.
  Not yet built; tracked as M-671.
- **`research/13-adk-phylum-RECORD.md`** (renamed from `research/12-adk-phylum-RECORD.md`; heading
  updated; RP-9 / all other internal references preserved): the Phase-1 evidence base for RFC-0023
  (fractured-methodology pass; four Opus sub-reasoners over one cross-context packet; findings
  Empirical/Declared, never Proven).
- **RP-9** (`docs/notes/research-prompts.md`): the follow-up deep-research pass that gates RFC-0023
  past Draft — must verify ADK→Mycelium concept-map, honesty-as-differentiator claim, tool-dispatch
  never-silent contract, session/runner model + LLM-harness reuse; Open, 2026-06-21.
### Added (2026-06-21: RFC-0022 — Web-Tooling Phylum Draft + research/12-web-phylum-RECORD.md + RP-10)
- **RFC-0022 — Web-Tooling Phylum** (`docs/rfcs/RFC-0022-Web-Tooling-Phylum.md`): **Draft** design for
  a `web` phylum (HTTP client/server/routing/JSON) — nodules `web.http` (never-silent HTTP/1.1
  parser: every malformed input is a located `ParseError`, never a sentinel), `web.json` (thin
  convenience over `std.io`'s one canonical JSON codec — no new codec, DRY/KC-3), `web.route`
  (inspectable route table + EXPLAIN-able dispatch), `web.server` (server as a `colony` of one
  request-handling `hypha` per connection — the RFC-0008 RT model faithfully instantiated),
  `web.client` (`get`/`post`/`request`). v1 Rust-first as `mycelium-web` (RFC-0016 §4.6
  precedent). **Gated**: typed `Json<T>` handler surface blocked on E7-1 (M-657 generics, M-659
  traits); `.myc` `colony { hypha … }` server surface blocked on E7-2 (M-666). Not yet built;
  tracked as M-670.
- **`research/12-web-phylum-RECORD.md`** (`Empirical/Declared` — design-informing prior-art +
  corpus-grounding, never `Proven`): the Phase-1 evidence base for RFC-0022 (T12.1.x–T12.4.x +
  §6 Honest-Uncertainty Register).
- **RP-10** (`docs/notes/research-prompts.md`): the follow-up deep-research pass that gates
  RFC-0022 past Draft — must verify HTTP/1.1 edge-case never-silent contract, JSON delegation,
  server-as-colony RT2 differential (Empirical, not Proven), and route EXPLAIN-ability; Open,
  2026-06-21.

### Added (2026-06-21: M-665 — L1 reserves the 10 DN-03 runtime terms, never-silent (G2))
- **Runtime-vocabulary lexer reservation** (`crates/mycelium-l1/src/{token,parse,lib}.rs`): all 10
  DN-03 §4 runtime terms (`hypha`, `fuse`, `mesh`, `graft`, `cyst`, `xloc`, `forage`, `backbone`,
  `tier`, `reclaim`) are now `Tok` variants in `keyword()` — reserved-not-active (like
  `phylum`/`colony`). Each produces an explicit `ParseError` ("reserved for the runtime model
  (RFC-0008), not yet active") at both item and expression position — **never** a silent identifier
  accept or panic (G2). No grammar production consumes them yet (lexed-reserved, not active). Adds a
  `reject/12-runtime-vocab-reserved-not-active.myc` conformance fixture + bidirectional table↔corpus
  integrity test; EBNF + grammar README updated. Closes E7-2 row M-665.
### Added (2026-06-21: M-669 — mycelium-lsp baseline completions, dogfooding DX)
- **LSP completion provider** (`crates/mycelium-lsp/src/completions.rs`): 36 **active** keyword
  completions (from `mycelium_l1::token::keyword()`) + 5 grammar-grounded snippets (`nodule-header`,
  `fn-def`, `type-adt`, `match-expr`, `swap-expr` — the swap snippet pins both `to:` and `policy:`).
  `wire.rs` advertises `completionProvider` and serves `textDocument/completion`. Honest (G2/VR-5):
  scope is **Declared** (lexical/scaffolding only, no semantic resolution); reserved-not-active
  (`phylum`/`colony`) and not-yet-lexed runtime words are excluded — never offered as usable. 83 tests.
### Added (2026-06-21: DN-20 — Tiered testing & change-scoping: faster `just check`, durable release gate)
- **Three change-scoped test tiers** (`scripts/checks/test.sh`, driven by `MYC_TEST_TIER`):
  `just test-fast` (Tier 0, pre-commit — change-scoped crates' unit/regression tests only,
  sub-second on a single-crate change), `just check` (Tier 1, **unchanged as the CI entrypoint**;
  `just ci` = `just check`) — change-scoped crates **+ their reverse-dependents**, all targets +
  proptest at LOW cases (`PROPTEST_CASES=8`) + every always-on non-test gate, and `just check-full`
  (Tier 2, release/durability — **full workspace**, proptest at HIGH cases (`PROPTEST_CASES=256`),
  `cargo-mutants` + a `cargo-fuzz` smoke; the M-654/ADR-021 Gate A3 durability surface).
- **Change-scoping** (`scripts/checks/changed-crates.sh`, new): maps the working diff to workspace
  crates via `cargo metadata` (longest manifest-dir prefix) and expands to their reverse-dependency
  closure, emitting `-p <crate>` args. **Conservative `--workspace` fallback** on any shared/root
  file change (root `Cargo.toml`/`Cargo.lock`/`.cargo/`/`justfile`/`scripts/`/…), detection failure,
  or missing base ref — it may *over*-test but never *under*-tests. Offline + deterministic +
  skip-graceful.
- **cargo-nextest** runner with a `cargo test` parity fallback (so local↔CI parity holds when
  nextest is absent); `just setup` installs nextest best-effort. nextest skips doctests, so the
  `check`/`full` tiers add a scoped `cargo test --doc` pass.
- **Proptest case-tiering FLAG fix.** `mycelium-numerics/tests/properties.rs` `cfg()` hardcoded
  `cases: 20_000` and silently ignored `PROPTEST_CASES`; it now reads the env var explicitly (low
  default 8). Verified: the same property runs in 0.00 s at 4 cases vs 0.82 s at 20 000 — the knob is
  now genuinely honored. The two mutant-witness configs route through the same `witness_cfg()`.
- **Honesty (VR-5):** no property/bound test is dropped — only its *case count* is tiered (low every
  commit, full on release). Coverage is focused, never removed; KC-3 small-kernel + local↔CI parity
  preserved (the change is in check tooling, not the kernel). Records the methodology + the swarm
  branch-hygiene pre-flight pattern (ties to mitigations #5/#7) in **DN-20** (Accepted) + CLAUDE.md.

### Added (2026-06-21: M-654 — Gate A3 WS8 durability: mutants + proptest + fuzz)
- **cargo-mutants green on the trusted base.** `mycelium-core`, `-cert`, `-interp`, `-numerics`
  report **0 un-triaged survivors**: every surviving mutant is either killed by a new witness test
  or documented as a justified *equivalent* in a single workspace-root `.cargo/mutants.toml` (16
  inline-justified entries — domain-constrained ±1 arithmetic, de Bruijn index bijections, total/
  unreachable defensive arms). Honesty (G2/VR-5): cert's 3 genuinely-equivalent survivors are
  excluded; its other 3 are *killed* by witness tests, not hidden. 14 new `mycelium-core` witness
  tests close the `lower.rs`/`content.rs` survivors (assert the child line not the header; ≥2 binders
  for distinct canonical names; absolute substrate depth).
- **proptest migration.** The hand-rolled fixed-seed LCG suites in `mycelium-numerics` and the
  `mycelium-vsa` seed-based property tests migrated to `proptest` with shrinking + `PROPTEST_*` seed
  rotation, preserving the exact bound-math.
- **cargo-fuzz targets.** A standalone `fuzz/` workspace with three targets (L1 lexer+parser, the
  M-210 checker, core `Value` deserialization) + a `workflow_dispatch`-only smoke CI
  (`.github/workflows/fuzz.yml`); nightly smoke green (0 crashes). `just mutants` / `just fuzz`
  recipes added; cargo-mutants/cargo-fuzz runtime output git-ignored.
- Closes **ADR-021 Gate A3** (DN-19 GAP-4) — the last open 1.0.0 kernel/core gate row. With A1/A2/
  A4/A5/B1/B2 already met, the kernel/core is 1.0.0-ready pending the maintainer's tag (M-655).

### Changed (2026-06-21: M-647/M-648/M-650/M-381/M-646 — editorial enactment sweep)
- **RFC-0016 (Core Library & Standard Library) → Enacted.** All 25 `mycelium-std-*` crates landed
  Rust-first (M-501–M-534, M-540, M-541): the 23-module Tier-A/Tier-B guarantee matrices are asserted
  in tests, never-silent G2 holds across all modules, per-op `EXPLAIN` obligation met. Self-hosting
  migration half (M-502, Phase-5-C) remains open per KC-2 gate ruling.
- **RFC-0017 (Maturation Scope & De-maturation) → Enacted.** `thaw`/scope-`matured` gate implemented
  and tested in `crates/mycelium-l1/`: scope-granularity elaboration via header, `matured fn`
  retirement enforced, `thaw` de-maturation marker present; DN-08's five §3 questions all closed.
- **RFC-0021 (Semantic-Level Projection Framework) → Enacted (framework).** M-380 `LlmCanonical`
  renderer landed in `crates/mycelium-lsp/src/project.rs`, total over all 11 L1 node kinds, P1–P6
  invariants unit-tested. LLM-leverage claim stays `Declared`/open in the isolated non-blocking track
  (RP-1 / M-381) — no leverage result asserted (VR-5).
- **RFC-0018 STAYS Accepted (not Enacted).** Stage-1 *static* graded judgment is not yet implemented
  (only stage-0 dynamic checks exist; DN-14 lists it as gate-fail 5 for self-hosting). Honesty/VR-5:
  acceptance does not upgrade to enacted without the implementation.
- **RFC-0019 STAYS Accepted.** Traits/generics deferred (LR-2); no implementation.
- **DN-04 (Optional Structured Diagnostics) → Resolved.** RFC-0013 enacted (M-345): §4 design landed
  in `crates/mycelium-lsp/src/diagnostics`; all Q1–Q5 decisions absorbed.
- **DN-05 (AOT Recursion Execution Strategy) → Resolved.** M-347 (trampoline) + M-349 (dynamic
  `DepthBudget`) both enacted; `EvalError::DepthLimit` explicit and `EXPLAIN`-able, G2 holds.
- **DN-10 (Remaining L1 Gaps) → Resolved.** R7-Q4 (M-390): prim signature table → content-addressed
  prim declarations; R7-Q3 (M-391): mutual-recursion surface elaboration in `mycelium-l1::elab`.
- **DN-12 (RFC-0020 Ratification-Readiness) → Resolved.** RFC-0020 ratified `Accepted (scoped)` per
  DN-12 recommendation; §4.2/§4.5 carve-outs recorded; readiness-capture purpose complete.
- **DN-11 (Next-Wave Plan) → Resolved.** §5 appended: Phase-5 completion summary (all 25 std crates,
  RFC-0016/0017/0021 Enacted), remaining gate items (M-649 DEFERRED post-1.0, M-502 not-yet), and
  Phase-6 roadmap (Stage-1 generics/traits, effect annotations, native codegen, self-hosting
  expansion, open research gaps).
- **M-649 DEFERRED (post-1.0).** Self-hosting Stage-2 scoped post-1.0 per ADR-021 §5. Gate status:
  5 present / 5 absent. Absent: generics, trait interfaces, effect annotations, wild/FFI, static
  guarantee index. DN-14 deferral note appended. M-649 stays open.
- **M-381 / M-646 closed.** Gate B2 headline met (DN-09 §10 determinate retention ratio — proceed).
  Arm-4 LlmCanonical→L1 bridge landed (M-381); arms 3/5 live runs backlogged per ADR-021 §5
  (need local GBNF model; do not gate 1.0.0).
- Doc-Index, RFC README, issues.yaml, per-doc changelog footers updated; `just check` green.

### Fixed (2026-06-21: M-653 — ADR-021 Gate A2, Medium-findings ledger)
- **Gate A2 closed — every WS2–WS6 Medium from the 2026-06-14 deep review is resolved.** A committed
  ledger (`docs/reviews/2026-06-14-deep-review/06-medium-findings-ledger.md`) records each of the 25
  finding-ids as **Fixed** (0 deferred), each with the mutant-witness regression test that fails
  without the fix. The re-audit found the large majority were already remediated in prior waves but
  never recorded resolved (so A2 read open); M-653 **verified** each fix *and its guarding test*,
  then closed the two genuine gaps — `A6-03` (a `serde_roundtrip` test pinning every
  bound-kind/basis/layout wire spelling) and the L1 reject-corpus
  `reject_expected_table_has_no_orphaned_entries` bidirectional-integrity test (the only net code
  delta). Honesty rule held: fixes only tighten a bound or turn a silent acceptance into an explicit
  refusal — no guarantee tag upgraded (VR-5).
### Changed (2026-06-21: RFC-0006 r4→r5 Doc-Index reconciliation + issue #67 close-out)
- **RFC-0006 Doc-Index row advanced r4 → r5** (`docs/Doc-Index.md`). The row previously reflected
  the r4 ratification (2026-06-15, KC-2-gated concrete syntax). Now reflects the r5 state
  (2026-06-18): Q1 discharged by DN-09 (KC-2 verdict = proceed; L3 = text syntax + projections
  co-equally, M-380/FR-S5); Q3 discharged by RFC-0018 (stage-1 grading accepted); Q6 (literal
  spelling) now committable. Residual non-KC-2-gated item: **Q8 `unsafe`-class L3 spelling**
  (mechanism committed via LR-9; only theming/keyword remains, tracked separately — not a
  RFC-0006 gate). Dependency DAG updated from `RFC-0006 (r4)` to `RFC-0006 (r5)`.
- **RFC-0006 body reconciled (append-only).** Two forward-looking passages superseded by r5:
  (1) §3 L3 bullet ("KC-2-gated…") — clarifying note appended citing DN-09 verdict and M-380;
  (2) §4.4 sequencing ("Now (this RFC, Draft)…KC-2 runs…") — status note appended recording
  steps 1 and 3 complete, steps 2/4 in progress (RFC-0001 r3/r4, RFC-0007…0021 Accepted). No
  ratified normative text was rewritten (VR-5; append-only rule observed).
- **Closes issue #67** (RFC-0006 "deliberate + ratify" tracking issue). RFC-0006 is Accepted at
  r5; all KC-2-gated items discharged; residual Q8 spelling is a separately-tracked design detail.

### Changed (2026-06-21: PM-sync hardening + branching discipline — tooling/process)
- **GitHub PM sync handles its flagged cases gracefully** (`tools/github/`). `labels.json` gains
  `area:spec`, `status:todo`, `type:task`; the project `Area` field gains a `spec` option;
  `conventions.json` gains a `scaffold` → `type:infra` mapping and ~18 recurring scope→area aliases
  (and routes `stdlib`/`std` → `area:stdlib`, `spec` → `area:spec` now that those areas exist).
  `label-aliases.json` gains a **`retire`** list — stock GitHub labels (`duplicate`, `help wanted`,
  `invalid`, `question`, `wontfix`) are deleted **only when unused**, else FLAGGED (G2: never a
  silent drop). PR scopes that are area-less *by design* (doc-ref `adr-*/dn-*/rfc-*`, task-ids,
  wave markers) are now surfaced as `~` info, not `!` flags. `--validate` is warning-clean;
  `--self-test` green (new coverage for `retire` + intentional-scope classification).
- **Branching/merge discipline strengthened** (CLAUDE.md). Explicit *working-branch → PR →
  squash-to-`main`* flow (`main` is never written except by a PR squash-merge); **pull the squashed
  `main` down into the branch before PR-ing** (and propagate down through swarm levels) so the diff
  is clean and the squash-merge is conflict-free; new swarm-failure mitigation **#7** (branch-ref
  drift → silent partial octopus merge: merge the ref the child *reports*, then *count* the landed
  files). `.claude/agent-context.md` + `.claude/kickoff.md` refreshed to match.
- **ADR-021 — Proposed → Accepted (maintainer-ratified).** The 1.0.0 kernel/core release-gate
  *criteria* are agreed; the §7 open questions (A2 Medium ledger, A3/A4 durability + `cargo deny`,
  A5 KC-4 budget, B1 RFC-0003 reconciliation) are now the tracked gate-completion work. Moves to
  `Enacted` only at the tagged release. (ADR README + Doc-Index updated.)
- **Stdlib ratification finalized (24/25).** `docs/spec/stdlib/runtime.md` ratified **Draft →
  Accepted** (v0 R1 surface; preconditions met — RFC-0016/0008 Accepted, M-521 landed, ADR-020
  Enacted, DN-16 re-audit clean); the other 23 implemented specs were already Accepted (DN-07).
  Further `runtime` constructs activate construct-by-construct at the Phase-7 gate (ADR-020).
- **`mycelium-std-sys` spec written + ratified** — the 25th crate's missing `docs/spec/stdlib/sys.md`
  (the one DN-16 NEEDS-WORK item) authored to match the landed crate (honest `Declared` wrappers)
  and ratified, completing 25/25 stdlib spec ratification.
- **arm-3 / arm-5 live runs BACKLOGGED** (maintainer decision) — nice-to-have, do not gate 1.0.0
  (ADR-021 §5); modules already landed (PR #290). M-381 headline (retention ratio) stays DONE.
- **DN-19 (Road to 1.0.0) added** — the prioritized, dependency-ordered remaining-gaps roadmap
  (the open ADR-021 gate rows are the critical path; everything else is post-1.0 / 1.x).
- **ADR-021 Gate A5 (KC-4 cert-overhead budget) ratified + closed.** Maintainer budget:
  per-swap certificate-check **≤ 5 µs absolute AND ≤ 2× the swap cost** (for swaps ≥ the check).
  Re-measured (`xtask kc4`): bijective 1.30×/1.27× (~1.7 µs), bounded 0.12× (~2 µs), observational
  8 ns → **within budget, ~2.5× headroom**. Long-term target: nanosecond range (post-1.0).
  Recorded in phase-2.md §6.7.
- **ADR-021 Gate B1 confirmed + closed** — RFC-0003 (r4), RFC-0006 (r5; KC-2 discharges Q1),
  RFC-0007 (r4) all Accepted; concrete-syntax ratification is KC-2-gated and KC-2 = proceed.
  Remaining open 1.0.0 gate rows: **A2** (Medium ledger), **A3** (WS8 durability), **A4**
  (`cargo deny` in `just check`) — tracked in DN-19.

### Added (2026-06-21: M-381 — arm-3 + arm-5 ablation arms, swarm-built)
- **arm 5 (embedded-DSL baseline, RR-3) is now RUNNABLE.** The model writes a small
  Python embedded DSL; the harness evaluates it in a **restricted sandbox** (blocks
  `import`/dunder/`open`/`eval`/`exec`; `__builtins__` cleared) to `.myc`, then scores
  with the **same `myc-check`** as arms 1/2/4. Malformed/hostile snippet → `None` →
  not-clean, never a false PASS (G2); the sandbox is best-effort restricted eval
  (Declared). New module `grok/arm5_embedded_dsl.py` (21 offline checks → self-test T17).
- **arm 3 (grammar-constrained decoding) implemented + offline-tested, runtime-blocked.**
  New module `grok/arm3_constrained.py`: a GBNF grammar for the gold `.myc` surface +
  an optional local `ConstrainedBackend` that **SKIPs honestly** when no local model is
  configured (the xAI REST surface exposes no grammar param — activation is the M-331
  llama.cpp path: install `llama_cpp` + set `MYC_ARM3_MODEL`). 17 offline checks →
  self-test T16. Never fabricated (VR-5). Harness self-test now **19/19**.
  Built by a Sonnet swarm (Opus-orchestrated) on `claude/orch-0000-llm-ablation-arms-35`.
### Added (2026-06-21: ADR-021 — 1.0.0 release-readiness gate, Proposed)
- **ADR-021 drafted (Proposed) — the explicit 1.0.0 kernel/core release gate.** CHANGELOG has
  long said versioning begins "when the kernel stabilizes" but no gate defined "stabilizes"
  (deep-review remediation roadmap §1). ADR-021 formalizes it: **Gate A** (honesty-integrity +
  durability — zero open High findings ✅, mutants/fuzz, `just check` incl. `cargo deny`, KC-4
  budget) and **Gate B** (decision/external — RFC-0003 reconciliation; **KC-2 verdict now MET**
  via DN-09 §10's determinate retention ratio). Surface language scoped to a tracked `1.x`;
  Phase-3+ maturation (native codegen, JIT, projections, self-hosting) explicitly out of scope.
  Grounded in remediation roadmap §5, ADR-018, Foundation §6. **Proposed only — it defines the
  criteria for maintainer ratification; it asserts no release** (house rule 3). Registered in
  `docs/adr/README.md` + `docs/Doc-Index.md`. M-381 annotated NON-BLOCKING (headline retention
  ratio recorded; arms 3/5 are research follow-ups that do not gate Phase-3 closure or 1.0.0).

### Added (2026-06-20: M-381 — rigorous arm-4 bridge; retention ratio now DETERMINATE)
- **`LlmCanonical→L1` bridge landed (`tools/llm-harness/grok/llm_canonical_to_l1.py`),
  DN-09 §9.4 option (b).** Converts the ablation's arm-4 `LlmCanonical` S-expression output to
  `.myc` surface and scores it with the **same authoritative `myc-check`** (parse+typecheck) as
  arms 1/2 — putting arm 4 on the same quality bar and replacing the prior 0 % scoring artifact
  with a non-zero, type-checked denominator. The converter is **Empirical** (heuristic rewrite;
  the typecheck stays `myc-check`'s); unconvertible output → `None`, scored not-clean, never a
  false PASS (G2). Self-test **17/17** (T15 added). Verified end-to-end: bridged programs
  typecheck clean against the real `myc-check`.
- **M-381 retention ratio is now DETERMINATE [Empirical].** Re-ran the ablation on two models
  (seeds [11,23,42], 8 tasks ⇒ 24 samples/arm): `grok-build-0.1` (run `20260620T230403Z`) arm2
  91.7 % / arm4 16.7 % → **retention 5.50 (550 %)**; `grok-4.3` (run `20260620T234224Z`) arm2
  91.7 % / arm4 41.7 % → **retention 2.20 (220 %)**. Both **≥ ~70 %** — the RFC-0021 §4.7
  promote-`LlmCanonical`-to-primary trigger does **not** fire; the grammar-primed novel surface
  out-performs the familiar-skin projection. Residual biases (signature-supply vs
  bridge-rejection) disclosed in DN-09 §10.2; non-falsification robust to them. Leverage claim
  stays **Declared/open** (arms 3/5 still blocked — only 3 of 5 arms). **KC-2 verdict
  unchanged: proceed.** Spend $0.088 this run (~$0.17 cumulative); raw reports gitignored. Recorded
  in DN-09 §10 (append-only) and M-381 status. Follow-ups (independent branch): arm 3 GBNF
  backend (M-331/llama.cpp), arm 5 embedded-DSL (RR-3, contingency), `xai_sdk` 1.17.0 batch-API
  reconciliation.

### Changed (2026-06-20: ADR-012 — Layered Lexicon status Proposed → Accepted)
- **ADR-012 status flipped to Accepted.** All five §7 flags were resolved by prior downstream
  decisions; the file status was simply never updated. §7.1 applied at review time (tier-label
  de-confliction); §7.2 resolved by RFC-0007 r2 (recursion-only kernel; bounded iteration
  reserved); §7.3 resolved by RFC-0008 (Accepted 2026-06-16) + Research Pass-4; §7.4 resolved
  by ADR-013 (Accepted 2026-06-10, `spore` is the deployable unit); §7.5/§7.6 resolved by DN-03
  (Resolved 2026-06-10, one name per term — flat). Resolution record added as ADR-012 §8.
  `docs/adr/README.md` and `docs/Doc-Index.md` updated to reflect Accepted status.
  (Editorial — no normative content changed.)

### Added (2026-06-20: Wave 4 — M-381 ablation results recorded)
- **M-381 ablation run `20260620T195352Z` — results recorded (DN-09 §9):** Three-arm live
  ablation (grok-build-0.1, seeds [11,23,42], 8 tasks × 3 seeds). Results [Empirical]: arm1
  (bare) 0/24 = 0%; arm2 (grammar-primer) 24/24 = **100%**; arm4 (LlmCanonical) 0/24 = 0%
  (scoring artifact — `myc-check` cannot parse S-expressions by design). Retention ratio:
  **INDETERMINATE** — arm4 denominator = 0 (arm4 pass@1 reflects scorer limitation, not model
  failure). Gold set: g04-widen-swap PARTIAL_PASS (self-corrected), g05-narrow-swap PASS (first
  attempt) — 4/8 total PASSes. Total spend $0.0373 this run, ≈$0.072 cumulative. KC-2 verdict
  unchanged: **proceed** (DN-09 §3). To unlock a determinate retention ratio: add
  LlmCanonical-native scoring (Python-side parse-only or RFC-0021 §4.1 L1-bridge).
  Arm3 (grammar-constrained) and arm5 (embedded-DSL) remain blocked (no GBNF, no RR-3).
  M-381 status: in-progress (non-blocking research track).

### Changed (2026-06-20: Wave 3 — ADR-020 Enacted, M-381 Arm 4 ablation run)
- **ADR-020 — Enacted** (2026-06-20): `runtime`/`colony` phylum-placement decision is now
  **Enacted** — the M-521 v0 R1 implementation landed on `main` in Wave 2
  (`crates/mycelium-std-runtime`; 21 tests, 16-row guarantee matrix, `#![forbid(unsafe_code)]`).
  Status: Accepted → Enacted. ADR-020 is outside ongoing maintenance; future runtime constructs
  (Phase-7 gate) will reference it but not modify it. Append-only.
- **ADR README** updated: status-set now shows `Proposed → Accepted → Enacted → Superseded`;
  ADR-020 table row and summary text corrected (was stale "Proposed").

### Added (2026-06-20: Wave 2 — M-620 native spore deploy seam, M-521 runtime phylum impl, M-381 Arm 4 unblocked)
- **M-620 — DONE (native deploy/germination seam, FLAG Q2 resolved):** `crates/mycelium-std-spore/src/deploy.rs`
  implements `germinate(spore, target) -> Result<DeployResult, DeployError>` — the content-addressed
  native deploy pipeline. `DeployVerification` carries `content_hash_canonical` (Exact) and
  `no_opaque_lowering` (Declared, VR-4). All 4 error paths are explicit, never silent (G2);
  `explain_deploy` is always deterministic (Exact, SC-3). 44 tests pass; guarantee matrix extended
  from 11 → 15 rows. ADR-013 FLAG Q2 ("Phase-6-gated") retired.
- **M-521 — runtime phylum implemented (ADR-020 Accepted):** `crates/mycelium-std-runtime` v0 R1
  surface fully implemented: `Scope<(),E>::join_all` (Empirical, FIFO sweep Exact, panic-caught G2);
  `Colony<T,E>::scope()` factory; `Task` with `run(self)` (Declared purity); `TaskCtx::cancel`;
  bounded `Network::channel(capacity)` returning `(Sender<V>, Receiver<V>)` with real
  `Arc<Mutex<VecDeque>>` backend; `TrySend<V>` / `TryRecv<V>` fail-closed (`ZeroCapacity` on
  capacity=0 — G2); all reserved vocabulary absent (RFC-0008 §4.5 Glossary ⟂); `#![forbid(unsafe_code)]`.
  21 tests; 16-row guarantee matrix; clippy -D warnings clean.
- **M-381 Arm 4 — unblocked (LlmCanonical parser + harness wiring):** `crates/mycelium-lsp/src/llm_canonical_parser.rs`
  — `parse_llm_canonical(source) -> Result<String, ParseError>`, depth-limited recursive descent
  (DEPTH_LIMIT=64; banked guard #4), 4 error variants, 14 tests. `tools/llm-harness/grok/llm_canonical_arm4.py`
  — `arm4_run` with graceful-skip when Rust binary absent (`status: "skip"`, reason reported,
  never silent). `ablation.py`: Arm 4 flipped to `runnable=True`. Python selftest 16/16 green.
  Guarantee tag: `Empirical` (heuristic parser — not upgraded without checked basis, VR-5).
- **ADR-020 — Accepted** (2026-06-20; moved from Proposed): unblocks M-521 implementation.

### Added (2026-06-20: Wave 1 — Phase 5/6 close: M-601 done, ADR-020 Proposed, M-602 verified)
- **ADR-020 — `runtime`/`colony` Phylum Placement (Proposed, M-521):** resolves RFC-0016
  §8-Q4 (the deferred phylum-placement question). Decision: **Option C hybrid** — a dedicated
  `runtime` phylum (`crates/mycelium-std-runtime`) with a thin `std.runtime` re-export facade
  inside `std`; construct-by-construct activation at the Phase-7 gate. v0 R1 surface:
  `Colony<T,E>`/`Scope<T,E>`, `Task`, `TaskCtx`, `Poll`, `SweepOrder`, `Deadlock`,
  `Network`, `Sender<V>`, `Receiver<V>`, `TrySend<V>`, `TryRecv` — with per-op guarantee
  matrix (`Exact`/`Empirical`/`Declared`; `Empirical` on `Scope`/`Network` grounded in the
  RT2 sequentialization + Kahn-determinism differentials; `Declared` on the `Task` purity
  contract). All RFC-0008 §4.5 reserved vocabulary (`hypha`/`fuse`/`xloc`/`cyst`/`graft`/
  `forage`/`backbone`/`mesh`/`tier`/`reclaim`) stays Glossary ⟂. `runtime` v0 is
  `wild`-free. Awaiting maintainer ratification (Proposed → Accepted).
- **M-601 — DONE (honest scope):** native MLIR→LLVM codegen for the bit/trit element-wise
  fragment (core.id, bit.not/and/or/xor, trit.neg) via `mlir-opt-18 | mlir-translate-18`
  behind the `mlir-dialect` feature (`crates/mycelium-mlir/src/dialect/native.rs`); every
  stage dumpable (VR-4); data/closure/recursion explicitly return `DialectError::Unsupported`
  (never silent, VR-5). The full calculus runs end-to-end across interpreter + direct-LLVM +
  MLIR; full-MLIR data/closure lowering is an honest open follow-up (not gated on M-601).
- **M-602 — verified:** three-way interp↔MLIR↔direct-LLVM differential harness in
  `crates/mycelium-mlir/tests/threeway_differential.rs`; 148 tests pass (graceful skip when
  `mlir-opt-18` absent); E1 speedup test skeleton in place (pending `scripts/setup-mlir.sh`
  toolchain installation). MLIR path correctly refuses out-of-fragment nodes while
  interp↔direct-LLVM equivalence holds across the full calculus corpus.
- **M-521 — in-progress (design initiated):** ADR-020 Proposed unblocks M-521-impl. Labels
  updated from `needs-design` → `in-progress`.

### Added (2026-06-20: KC-2 / M-002 close — Grok/xAI harness verification + honest run record)
- **DN-09 §7 (append-only):** records the 2026-06-20 Grok/xAI live run attempt. Harness
  self-test **14/14** (Empirical/plumbing — 14 checks: T0–T12 + T2b verified). Live run **blocked** by
  `HTTP 403 permission-denied` (xAI account has no credits; endpoint reachable; $0 spent).
  Retention ratio (T3.6 / M-381): **INDETERMINATE** (ablation did not run). Schema finding:
  Grok harness output format differs from `mycelium-bench` ingestion schema. Standing
  KC-2 verdict (2026-06-18, **proceed**) is unchanged — the 403 is a billing constraint,
  not a language-learnability result.
- **M-002 status → done** (issues.yaml): the KC-2 verdict is genuinely recorded with its basis
  in DN-09; Grok follow-up attempt documented in DN-09 §7, blocked on billing.
- **M-381 status:** remains in-progress; updated body notes INDETERMINATE retention ratio,
  blocked Grok arm, and schema mismatch requiring resolution before a successful re-run.
- **models.toml reconciled** against operator-provided xAI API docs (docs.x.ai/developers/models
  as of 2026-06-20): removed three undocumented model IDs (`grok-4.20-0309-non-reasoning`,
  `grok-4.20-0309-reasoning`, `grok-4.20-multi-agent-0309`); retained the two confirmed public
  models (`grok-build-0.1`, `grok-4.3`). Self-test T0 updated to expect ≥2 models (not exactly 5);
  self-test is now **15/15** (T13 for `from_api_discovery` added — see dynamic discovery entry).

### Added (2026-06-20: LLM harness — dynamic model discovery via GET /v1/models)
- **`grok/client.py` — `discover_models()`**: queries `GET /v1/models` via stdlib `urllib` (no
  third-party deps); raises `DiscoverModelsError` on HTTP error/parse failure, `ApiKeyMissingError`
  if no key. Never logs the key value. Returns raw model-dict list for conversion.
- **`grok/models.py` — `from_api_discovery()`**: converts the raw API response to `ModelSpec`
  objects. HONESTY (VR-5): all resulting values are **Declared** (API-asserted or conservative
  defaults). Skips entries without a parseable pricing block (warning to stderr, G2). Batch prices
  = sync prices (no invented discount). Raises `ModelConfigError` if nothing usable remains.
  Conservative Declared defaults: `rpm=60`, `tpm=2,000,000`, `context=131,072` when the API omits
  them.
- **`grok/cli.py` — `--discover-models` flag**: when present, queries `GET /v1/models` first;
  falls back to `models.toml` with a warning on `DiscoverModelsError`; falls back with an error
  log on `ApiKeyMissingError`. The `--list-models` output labels its source
  (`GET /v1/models (N discovered, Declared)` vs the file path). Never-silent: a key present but
  returning an auth error logs the failure and falls back; a bad rubric still stops the run (G2).
- **`run.sh` — `--discover-models` passthrough**: forwards the flag to all `grok.cli` invocations
  (list-models, smoke, main run). Consistent with the `--` passthrough design.
- **T13 self-test (`grok/selftest.py`)**: offline, deterministic test of `from_api_discovery()`:
  valid entry → correct `ModelSpec` + Declared defaults; missing pricing skipped; negative pricing
  skipped; duplicate id skipped; missing context_length → 131,072 default; all-bad input → G2
  `ModelConfigError`. Self-test is now **15/15** (was 14 before T13).

### Added (2026-06-20: LLM harness — one-command runner + packaging fix)
- **`tools/llm-harness/run.sh`** — a one-command runner: resolves uv (else system python3),
  `uv sync` (+`xai_sdk` only for `--batch`), runs the offline self-test and **aborts if it
  fails** (never spend on a broken harness — G2), lists the cheapest-first models, then does the
  **capped** live/batch run *only when a key is present* (otherwise it stops gracefully after the
  free checks and prints how to set the key). Knobs: `--check-only`, `--smoke`, `--max-usd`,
  `--models`, `--no-ablation`, `--batch`, `--discover-models`, and `--` passthrough to `grok.cli`.
  shellcheck-clean.
- **Packaging fix:** added the missing `[build-system]` to `tools/llm-harness/pyproject.toml`, so
  `uv sync` builds + installs the project (and the `grok-harness` console script / `--extra batch`)
  instead of skipping the entry point with a warning. Verified `uv sync` clean and `uv run
  grok-harness --self-test` → 15/15. The harness stays **lockless by design** (`.venv`/`uv.lock`
  gitignored; live + self-test are pure stdlib — no third-party runtime deps).
- Audited the harness for real-run pitfalls: the live client is stdlib `urllib` (no missing deps),
  all modules import, the no-key path fails gracefully (clear message, not a traceback), the
  reports dir is created on demand, and live requests carry a 120 s timeout (no infinite hang).

### Added (2026-06-20: LLM harness — USD spend gate (`--max-usd`, default $10))
The Grok/xAI harness (`tools/llm-harness/grok/`) now **gates** total xAI spend (the operator's ≤ $10
requirement), instead of merely *estimating* cost. A new `budget.BudgetGuard` (one instance per run,
shared across all models so the cap is the *total* spend) gates each unit of work — a task and the
ablation block (live) or the batch (batch mode): a unit whose **conservative** cost estimate would push
cumulative **actual** spend past the cap is **refused before it is sent** (`BudgetExceeded`), the run
stops, and whatever completed is emitted as a partial, honestly-flagged report (G2 — never a silent
launch of more work). Honest scope (VR-5): this is a **best-effort** gate, **not a formal upper bound** —
the per-call token figure is a heuristic (`_estimate_tokens` ≈ chars//4 + headroom) and live completions
are unbounded (no `max_tokens`), so a single in-flight request can overrun; the gate biases high and
refuses *new* work early. Actual billed cost is recorded as the run proceeds (the reported spend is the
real one), and a non-finite `--max-usd` (nan/inf) is rejected at parse time so the gate cannot be
silently disabled. `--max-usd` (default `10.0`) is wired through `RunConfig`; the offline `--self-test`
gains a deterministic budget-gate check (now **15/15**, no key/network). Documented in the harness
README ("Live xAI (Grok) runs").

### Added (2026-06-20: Phase 8 — toolchain API ergonomics (M-644))
Purely **additive** ergonomics on the four toolchain library crates (owns only their `src/lib.rs`) —
every change ADDS a symbol / trait impl / attribute; **no in-workspace caller changes** (verified by a
clean `cargo build --workspace`). Completes Phase 8. *Compatibility nuance:* `#[non_exhaustive]` is
additive to the API surface (a variant is never removed), but for an **external** crate that
exhaustively `match`es one of these enums it is a source change requiring a `_` arm — not silently
breaking, and no such matcher exists in-workspace.
- **`#[non_exhaustive]`** on the growth-prone public enums `FmtError` (fmt), `FixTier` (lint), and
  `Severity` (diag) — a future variant is no longer a breaking change *for downstream users that
  already carry a `_` arm*. Verified no in-workspace external exhaustive `match` exists (the cross-crate
  `Severity` matches are on *other* `Severity` types — lsp's, sec's — not `mycelium_diag::Severity`).
- **`Default`** for `Formatted` (derived) and `Fix` (tier `Suggest` — the **never-auto-applied** tier,
  so a defaulted fix can never silently rewrite code, G2); **`From<String> for Formatted`** (a trivial
  lift; `format_source` remains the path that computes `changed`/`notes`).
- **Fluent `with_*` builders**: `LintFinding::with_fix`, `LintReport::with_finding/with_files`,
  `Finding::with_route`, `Report::with_finding/with_files_checked`.
- **`check_source_default(file, src, out)`** — a new-named convenience (Rust has no overloading;
  renaming `check_source` would be breaking) that derives the builtin `ClassRegistry` + baseline policy
  the way `check_sources` does and delegates to the existing 5-arg `check_source`.
- Honest caveat: the `cargo public-api` baselines (`docs/spec/api/*.txt`) are not regenerated here (the
  tool is absent; `api.sh` skips gracefully) — the changes are additions-only, refreshed when the tool
  is available. Verified: `cargo test -p mycelium-fmt -p mycelium-lint -p mycelium-check -p
  mycelium-diag` green; clippy `-D warnings` (`-A unsafe_code`) + full `just check` green.

### Changed (2026-06-20: PM-manifest phase-status reconciliation — Phase 0/3/8)
Reconciled the `tools/github/issues.yaml` board to the verified completion state (the labels had
lagged reality — `docs/planning/phase-0.md §2` already recorded the work done). Audited each item
against on-disk artifacts / child-task status before flipping (honesty rule — no marking-done
without a checked basis):
- **Phase 0** → `status:done` on the 8 completed tasks (M-001 LH/KC-1 probe, M-010 schemas,
  M-011 SPECIFICATION.md, M-012 binary↔ternary spec, M-020 surface fragment, M-090 docs-CI,
  M-091 Rust workspace, M-092 Python tooling — all artifacts present on disk). The one residual,
  **M-002 (KC-2 LLM-leverage verdict)**, was marked **`status:in-progress`** at the time of this
  reconciliation: the DN-09 verdict (*proceed*) was recorded, but the SC-5b baseline + the rigorous
  T3.6 ablation were gated on a **live model run** (tracked as M-381) — surfaced honestly, not
  overclaimed. *M-002 was subsequently closed in this same PR (see "KC-2 / M-002 close" entry
  above) after the Grok/xAI run attempt was documented honestly in DN-09 §7 and the KC-2 verdict
  deemed fully recorded.*
- **Phase 3** → `status:done` on the completed epics E3-2/E3-3/E3-4/E3-6/E3-7 (every child task
  already `status:done`); **Phase 8** → `status:done` on epic E8-1 (children M-383/384/385 done).
- **README** phase-status line updated: Phases 0–5 and 7 complete; Phase 6's VR-4 exit gate met
  (M-601/M-620 follow-ons in progress); Phase 8 complete bar the deferred M-644.

### Added (2026-06-20: E-BENCH — honest benchmarking & evaluation harness (E3-9 / M-645))
A new crate **`crates/mycelium-bench/`** (lib + `bench` bin) that measures **what this language
actually buys us — and where we lose**. Times whole v0-calculus programs across every execution
backend (interpreter trusted-base, AOT env-machine, JIT, direct-LLVM, MLIR-dialect behind
`mlir-dialect`) over a shared 14-program corpus, and classifies each (backend, case) vs the
interpreter as a speed **WIN/LOSS**/neutral, a **capability LOSS** (with reason), a correctness LOSS,
a runtime error, or a graceful **Skip**.
- **Honest differential.** Compares the **observable** (`repr`+`payload`+`guarantee` / content-identity,
  provenance excluded per RFC-0001 §4.6 — matching the M-210 `ObservationalEquiv` checker). This fix
  turned 10 false-positive "correctness losses" (compiled backends stamp `Provenance::Root` where the
  interpreter records a `Derived` chain) into honest speed verdicts — a clean differential (0
  correctness losses).
- **Surfaces losses, not just wins (G2).** The committed sample reports 1 win / 1 neutral / 26
  speed-losses / 14 capability-losses / 0 correctness-losses — e.g. `rec-mutual`/`aot-env` ×1.11 (win);
  `bit-not`/`direct-llvm` a **process-spawn-bound** loss (per-invocation time dominated by spawning a
  fresh native process, not compute — M-602/E1, surfaced not buried); `rec-mutual` a capability-loss
  (mutual recursion unsupported in Increment-3, RFC-0004 §11.6).
- **Ingests the LLM-harness report** (M-330/M-331; KC-2/SC-5b), labelling synthetic as synthetic.
- **Honest tags (VR-5):** every measured number is **Empirical** (trial count + spread); a capability
  loss / skip / error is **Declared**; no verdict is Proven/Exact; **no pre-written performance
  target**. Debug builds refused for perf numbers; spawn/warmup caveats stated.
- **Zero new external deps** (only workspace `serde`/`serde_json` + the internal crates it measures,
  read-only). Verified: `cargo test -p mycelium-bench` 29 lib + 4 integration green; workspace clippy
  `-D warnings` + build green; full `just check` green.

### Added (2026-06-20: Phase 6 — native acceleration + deploy + VR-4 exit gate (M-360, M-610/620/630))
Completed the Phase-6 native path on the keystone (E6-1) — the **VR-4 no-opaque-lowering cross-backend
gate is met (the Phase-6 exit)**. All in `crates/mycelium-mlir/` (+ DN-18); interpreter stays the
trusted base, native is the differential-tested perf-path.
- **M-360 (Phase 3) — measured BitNet SIMD throughput.** The I2_S/TL1/TL2 SIMD unpack kernels already
  existed + were differential-tested; added the missing **measurement**: an `#[ignore]`-by-default
  release harness timing scalar vs SIMD, correctness re-checked vs the oracle. **Empirical, no
  pre-written target** — illustrative: I2_S ×1.09, TL1 ×1.65, TL2 ×4.72 over scalar.
- **M-610 — packed-ternary on the native backend.** `bitnet::KernelLayout` reifies the kernel's
  inspectable `Meta.physical` (scheme + *measured* bits/element — 2.0 for 2-bit, ~1.667 for TL2 — +
  EXPLAIN, never hidden, DN-01). The M-251 E3 wrong-layout differential is carried onto the **actual
  compiled kernel** (real `clang`): a mislabeled layout misreads → caught (NFR-7, non-vacuous).
- **M-620 — deployable native artifact (primitive + design note).** `deploy::NativeArtifact`: a
  content-addressed deploy descriptor whose **identity *is* the program's `ContentHash`**, derived
  *internally* from `Node::content_hash()` (ADR-003) — never a supplied input, so `id()` can never name
  a different program than its IR/attestation embody (a forged identity is *unrepresentable* — G2 in
  its strongest form). Carries the dumpable IR + the VR-4 attestation into the deployed unit; failures
  keep a *structured* signal (an absent toolchain → `DeployError::ToolchainMissing` so a caller skips,
  mirroring `AotError`; an un-lowerable program → `DeployError::NotDeployable`) — never fragile codegen,
  never brittle string-matching. **Honest scope:** the full Spore wiring (the `spore`←`mlir` dep + the
  wire-schema `native` component) is **ADR-level and impl-pending**, designed in **DN-18 (Draft)** —
  `status:in-progress`, not overclaimed.
- **M-630 (Phase-6 EXIT) — VR-4 cross-backend gate.** `vr4::cross_backend_gate` makes the
  no-opaque-lowering obligation **mechanical + enumerable** over all **6 backends** (interpreter, AOT
  env-machine, direct-LLVM, MLIR skeleton, real MLIR `arith`/`func`→LLVM, JIT/SIMD) — each yields an
  inspectable textual lowering stage tagged at honest strength (Declared for trusted-base/skips,
  Empirical for differential-validated dumps; **never Proven**); an absent feature/tool is an explicit
  `Skipped(reason)`, never a fabricated dump. The test drove the real `mlir-opt-18 | mlir-translate-18`
  pipeline to genuine LLVM IR. **No opaque pass on any backend → Phase-6 exit gate met.**
- Verified: `cargo test -p mycelium-mlir` 133 (default) / 140 (`mlir-dialect`) green; `mycelium-spore` &
  `mycelium-proj` 32 green; workspace clippy `-D warnings` + fmt clean; full `just check` green.

### Added (2026-06-20: Phase 3 — Grok-pluggable LLM co-authoring harness (M-330/331; M-381 wired))
Extended `tools/llm-harness/` (M-330/331, already `done`) with a pluggable, batchable, cost-aware Grok/xAI
backend so the language-leverage experiment is runnable from a WSL host (or any box with a key).
- **Three backends, never-silent.** Live **OpenAI-compatible REST** (`https://api.x.ai/v1`, key from
  `XAI_API_KEY`/`GROK_API_KEY`), native **`xai_sdk` batch** (`Client().batch.create(...)` for lower
  cost), and the pre-existing local **llama-server** (RTX-5080/WSL) — selectable per run. A missing key
  / bad config / unexpected SDK shape is an explicit error (G2), never a guess.
- **Configurable, ordered multi-model runs** (`models.toml`): the 5 Grok models run **cheapest→priciest**
  (`grok-build-0.1` $1.00/$2.00 → `grok-4.3` → the `grok-4.20-0309-*` variants $1.25/$2.50), with
  per-model RPM/TPM-aware pacing + backoff. Captures the KC-2/SC-5b quality measures (syntactic validity,
  type-check pass via `myc-check`, edit-to-fix iterations) **plus** tokens, latency, and computed USD;
  emits a per-model JSON + a cross-model comparison markdown.
- **M-381 ablation wired** (`research/11` T11.7 retention-ratio) — runnable mode; arms 3/4/5 are reported
  **blocked on M-380** (never fabricated), the verdict INDETERMINATE until they land.
- **Honesty floor.** No API key exists in CI, so the **live quality/cost/retention numbers are a
  USER-EXECUTED step** (Declared / pending-run). Plumbing is **Empirical** — an offline `--self-test`
  (mocked client) is green 13/13 (model ordering, RPM/TPM pacing, batch-vs-live cost accounting, scoring,
  report emission); the one committed sample report is stamped `SYNTHETIC (self-test)`. Batch prices are
  `Declared` placeholders (default to sync) pending the published batch-pricing doc. Live `--mode live`
  vs `--mode batch` documented for WSL.

### Changed (2026-06-20: Phase 5 — L1 DRY (E4-1) + stdlib error unification (E5-1))
Two behaviour-preserving Phase-5 refactors (existing tests stay green; no guarantee/semantic change, VR-5).
- **E4-1 — L1 front-end DRY + ergonomics (M-640/641/642).** Parser: `comma_separated`/`expect_keyword`
  fold 8 hand-rolled separated-list loops + 12 keyword-opener diagnostics onto one path each (context-
  bearing messages left byte-identical). Passes: a shared `pub(crate) walk_expr` pre-order traversal +
  a `SpecializeRow` trait make the Maranget row-specialization one generic across the bare matrix and
  the arm-tagged decision rows (`decision.rs` 398→190 code lines). Additive API ergonomics (ctors,
  `Hash`, `#[non_exhaustive]` on `Literal`, getters) — **`Default for TypeRef` deliberately omitted**
  (no honest zero value → would mask bugs, G2). All 7 l1 dependents still build + test.
- **E5-1 — stdlib error unification (M-535/536).** New `mycelium_std_core::error_scaffold`: a `StdError`
  marker super-trait + blanket impl + an opt-in `impl_std_error!` declarative macro (plain / generic /
  `source`-delegating) factoring out **only** the mechanical `impl std::error::Error` boilerplate — never
  a Display message, variant, derive, or guarantee tag (DN-17 §5 non-coupling). **24 error types across
  12 std crates** converted, behaviour-preserving (zero Display/variant/derive lines removed).
  **Honest scope:** 5 crates (content, dense, swap, sys, ternary) are **deliberately not** converted —
  they don't depend on `std.core` and `std-sys` sits *below* the std layer; adding the dep for a 1-line
  marker would be a layering inversion (G2/VR-5, KC-3) — left hand-rolled. **Honest LOC:** net **+345**
  (the documented+tested scaffold dominates at this eligible scale; the win accrues as more complex error
  types adopt it) — the DN-17 "net LOC down" expectation is corrected, not claimed.

### Added (2026-06-20: Phase 8 — release mechanics + CLI DRY (M-383, M-384, M-643))
- **ADR-018 (Accepted) — versioning policy.** Per-crate `0.x` SemVer across the workspace,
  **source-only** distribution (no crates.io publish in the design phase), and the `CHANGELOG`
  `[Unreleased]` → release-cut mapping. Grounded in ADR-007 + the squash-only linear-history
  discipline + RFC-0017 §4.1. (ADR index now also lists ADR-019.)
- **release-plz dry-run (M-384).** `release-plz.toml` (per-crate; `publish`/`git_release`/`git_tag`/
  `release_pr` all disabled — defense-in-depth) + `.github/workflows/release.yml` — **`workflow_dispatch`
  only**, advisory (`continue-on-error`), read-only token, runs `release-pr --dry-run` (no push/PR/tag/
  publish). Consistent with the repo's "no automatic remote CI" policy.
- **CLI DRY — `mycelium-cli-common` (M-643).** A dependency-free (`std`-only, KC-3) crate folding the
  three duplicated idioms out of the four toolchain CLIs: `read_source` (stdin-or-file, never-silent
  G2 — tool-tag is a parameter so each bin's stderr stays byte-identical), `walk_myc` (the `.myc`
  recursive sorted walk), and `Args` (the value-flag arg-loop). `mycfmt`/`myc-check`/`myc-lint`/
  `myc-sec` refactored onto it, **behaviour-preserving** (runtime-verified identical stderr/exit codes),
  net −58 LOC. Tags: each helper **Exact** (thin total `std` wrappers). Resolves the scaffold placed by
  the keystone wave.

### Added (2026-06-20: Phase 8 — one-command setup, gitleaks redaction, PM-manifest hardening)
Toolchain/PM infrastructure: a single idempotent, parameterized install command plus secret-scanning,
and a more forgiving-but-honest manifest validator.
- **One-command install (`scripts/install.sh`, `just setup`).** Idempotent + parameterized (component
  flags/`MYC_INSTALL_COMPONENTS` env: rust · python/uv · check-tools · pre-commit hooks; `--mlir`
  opt-in, delegates to `setup-mlir.sh`; `--help`). Probes before acting (a double run is a no-op);
  reads the MSRV from the committed pin (never bumps it); never-silent G2 (optional tool missing →
  clear skip + continue; a required failure ends `exit 1` with a summary); **no `curl|bash`**.
- **gitleaks secret-scanning, redaction-first.** A `gitleaks` pre-commit hook in `--redact` mode
  (pre-commit manages the binary — no manual install) + `.gitleaks.toml` (default ruleset + a
  **path-only** allowlist for lockfiles/`target/`/binaries; **`.example` templates stay scanned**) +
  `just secrets-scan`/`gitleaks`. `.env.example` ships generic safe placeholders (incl.
  `XAI_API_KEY`/`GROK_API_KEY`) and is tracked; real `.env`/`*.key`/`*.pem` are git-ignored.
- **PM-manifest validator — severity refined (per maintainer).** A label/milestone *used by an issue
  but absent from the manifest* is now a **non-fatal, actionable WARNING** (never-silent), not a hard
  abort; `reconcile_labels` **auto-creates** such a label with a default colour + a loud log so the
  warning never becomes a sync break. Defined-but-unused entries are **info-level**. Only genuine
  corruption (malformed JSON/YAML, dangling `doc_refs`) still fails (G2). New **`--debug`** mode
  (`gh-issues-sync.py` + `manifest-check.py`, threaded through) prints the full traceback + detail for
  investigating this class of issue. Defined the previously-missing `status:in-progress` label.
- Verified: `--validate` green (used→warning exits 0; corruption still fails); `--self-test` green;
  `just --list` + pre-commit config + `.gitleaks.toml` + `install.sh -n` all parse; full `just check` green.

### Added (2026-06-20: Wave-1 — native MLIR→LLVM path keystone (M-348/M-603, M-601, M-602); epic E6-1)
Closed Phase-4's last open item (M-348) and delivered the Phase-6 E6-1 keystone: a real, non-fragile
`mlir-dialect` lowering path with libMLIR now durably provisionable on Linux. The interpreter stays the
trusted base; MLIR is a differential-tested perf-path increment, never a black box.
- **Durable libMLIR provisioning (M-348/M-603).** `scripts/setup-mlir.sh` — version-matched to the
  installed LLVM (major derived from `llc`/`clang`, never hard-coded — no silent version bump),
  `apt-get install libmlir-$M-dev mlir-$M-tools`, idempotent, skip-gracefully (`exit 0`), no
  `curl|bash` — exposed as the dedicated **`just setup-mlir`** recipe (kept out of the default `just
  setup` so it never apt-installs/sudo-prompts for the OFF-by-default feature). **ADR-019 (Accepted)** records libMLIR as the build dependency
  of the `mlir-dialect` feature; **DN-15 §9** appends the unblock (provisionable on Linux). Default
  build/test stay green WITHOUT libMLIR.
- **Real MLIR-dialect lowering (M-601) — honest scope.** `mycelium-mlir`'s `dialect::native` lowers the
  **bit/trit element-wise fragment** (`core.id`, `bit.{not,and,or,xor}`, `trit.neg`) to a genuine
  `arith`/`func` MLIR module → `mlir-opt-18 --convert-func-to-llvm --convert-arith-to-llvm
  --reconcile-unrealized-casts | mlir-translate-18 --mlir-to-llvmir` → real LLVM IR → native; every
  stage dumpable (VR-4). Behind the **OFF-by-default `mlir-dialect`** Cargo feature; the toolchain is
  probed at runtime and skips gracefully when absent (`DialectError::ToolchainMissing`). **Honest
  boundary (G2/VR-5):** data/closure/recursion are NOT given a second, divergent MLIR codegen — they
  remain on the proven direct-LLVM backend (`llvm.rs`, M-373/378/379) + env-machine, and the MLIR path
  **explicitly refuses** them (`DialectError::Unsupported`), routing there rather than shipping fragile
  codegen. Tag: **Empirical** (real compiled artifact; correctness evidenced by the differential — not
  Proven). So the v0 calculus runs end-to-end on the compiled paths, but the *MLIR-dialect* increment
  itself is element-wise; trit-carry/data/closure/recursion stay on direct-LLVM/interp.
- **Three-way differential + measured E1 (M-602).** `interp ≡ direct-LLVM ≡ MLIR-dialect` over the
  element-wise corpus through the shared M-210 checker (non-vacuous + discriminating; out-of-fragment
  nodes asserted explicitly refused by MLIR while `interp ≡ direct-LLVM` still holds). The E1 verdict
  moves "not established" → **measured** (release-gated, refuses a debug build, no pre-written target;
  honest caption — the trivial kernel is process-spawn-bound, so this is an AOT-path number, not a
  blanket speedup claim).
- Green: `cargo test -p mycelium-mlir` (default) and `--features mlir-dialect` both pass; fmt + clippy
  (both feature configs, `-A unsafe_code` per ADR-014) clean; full `just check` green. Also scaffolds
  the empty `mycelium-cli-common` crate (workspace infra the Phase-8 release-mechanics work populates).

### Changed (2026-06-20: api-index generator — module-aware symbol attribution, PR #259 review)
Hardened `tools/docgen/code_index.py` so a short name defined in several modules (e.g. `compile` in
both `mycelium_mlir::llvm` and `::dialect::native`) resolves to the **right file** instead of the
first alphabetically: each candidate definition is tagged with its file's module path and matched
against the symbol's (crate-prefix-stripped, trailing-type-segment-stripped) module — **exact** match
first, then a **longest-ancestor** fallback for inline `mod`s. Genuinely-undecidable cases (root
re-exports; a method whose `impl` lives in a different file) are attributed best-effort **and flagged**
(`ambiguous: …`), never silently mis-attributed (G2; source stays ground truth). Also fixed a
duplicate-`src`-dir bug that made every symbol in a dash-named crate look doubly-defined (3416→3424
flagged is now honest, was inflating to 6013). New offline `--self-test` attribution cases (12) gate
it via `scripts/checks/doc-index.sh`. Resolves the Copilot PR #259 api-index review notes.

### Added (2026-06-20: M-397 — `gh-issues-sync` bounded-concurrency, rate-negotiated, batched execution)
Sped up the live reconcile by overlapping the many small, independent `gh` mutations per run while
staying inside GitHub's secondary-rate limits — never-silent (G2), fault-tolerant, and a clean N=1
fallback. Builds on the existing M-382 per-call retry/backoff/120s-timeout in `_run_gh`/`gh()` (used
**as-is**, unchanged).
- **Bounded thread pool over the existing synchronous `gh()`** (subprocess/IO-bound ⇒ threads, **not**
  an asyncio rewrite). New `--concurrency N` (default **6**, conservative for secondary limits);
  **`N=1` reproduces today's exact sequential behaviour** (the debugging / `--verbose` fallback — each
  task runs inline, in submission order, with no executor). `--dry-run` stays sequential for a stable,
  ordered preview and mutates nothing.
- **Batched dispatch with `as_completed` aggregation** — the independent per-item loops now dispatch as
  batches: label create-or-update, milestone create, **issue create (pass 1)**, **per-issue updates
  (pass 2)**, PR backfill, and the noncompliant-label **migration relabels**. The cross-batch
  dependency order is preserved (labels+milestones before issue creation; create-pass-1 fully
  aggregates before the update pass). Only items that are idempotent + target disjoint resources are
  parallelized within a batch.
- **Well-negotiated rate limits** — (a) bounded concurrency; (b) a shared `RateGate` (a
  `threading.Event` + lock): on a `403`/`429`/`secondary rate limit`/`abuse`/`Retry-After` stderr the
  whole pool **PAUSES** for the advised window (parsed by the pure, testable
  `should_pause_for_rate_limit`), then resumes — one post-pause retry, never a continued burst; (c) an
  optional start-of-run `gh api rate_limit` probe that reduces `N` when the remaining core budget is
  low (`negotiate_concurrency`, never-silent; `--no-rate-probe` opts out). A **primary** rate-limit is
  deliberately NOT absorbed into a short pause (it resets hourly — `_gh_fail` surfaces it honestly).
- **Never-silent, fault-tolerant, non-interleaving output** — each task returns `(item, ok, err)`;
  `run_batch` collects results in **submission order** (deterministic summary + failure FLAGs),
  aggregates per batch (`aggregate_results`), prints `>> <batch>: N ok, M failed`, and keeps every
  failure as a re-runnable FLAG — one failure (incl. a stray `SystemExit`) never aborts the batch.
  **All printing is guarded by a process-wide lock** (`safe_print`/`_safe_stderr`) so concurrent /
  `--verbose` output never interleaves.
- **Pure helpers + `--self-test`** — `should_pause_for_rate_limit`, `parse_rate_remaining`,
  `negotiate_concurrency`, `aggregate_results`, `_issue_update_args`, `partition_issue_work`, and
  `run_batch` (N=1 sequential + concurrent fault-capture) are all covered offline. Verified:
  `python3 tools/github/gh-issues-sync.py --self-test` green; ruff lint + `ruff format --check` clean;
  `just check` green. No live/networked `gh` was run — GitHub's exact concurrency/secondary-limit
  thresholds are **Declared** (documented, not Proven) and tuned conservatively (default N=6, pause +
  one retry); see `tools/github/RECONCILE.md`.

### Fixed (2026-06-20: M-382 — `gh-issues-sync` EOF-aware retry/backoff + fault-tolerant migration + `--verbose`)
Hardened `tools/github/gh-issues-sync.py` against the M-382 hang. Root cause was three-fold:
(a) the network-error classifier in `_gh_fail` omitted `EOF`, so the symptom `gh: Post "…":
unexpected EOF` fell through to the non-retryable generic branch; (b) there was **no** retry/backoff
anywhere, so one transient blip on a paginated `gh api` read aborted the whole sync; (c) the
noncompliant-label migration ran each per-issue `gh issue edit` with `check=True`, so a single failing
edit `sys.exit`ed and aborted the entire label migration mid-run (looked like a hang; the stale label
was never deleted).
- **EOF-aware transient classifier** — new pure `_is_transient_network(stderr)` recognizes
  `unexpected eof` / `eof` / `connection reset` / `reset by peer` / `tls handshake` / `i/o timeout`
  **plus** the pre-existing DNS/dial/timeout/unreachable/refused set; used in **both** the `_gh_fail`
  network branch and the new retry (covered by `--self-test`).
- **Bounded retry + exponential backoff in `_run_gh`** (1/2/4/8s, 4 retries) — centralizes the fix for
  **every** `gh` call (pagination, edits, graphql): a never-silent `~ transient network error … retry
  k/MAX in Ds: …` line per retry (G2); a non-transient failure or success returns immediately; after the
  last retry the final `(rc,out,err)` is returned so `gh()`/`_gh_fail` still produce the classified error.
- **Fault-tolerant migration loop** — each per-issue relabel now runs non-aborting; a failure prints
  `! could not relabel #<n> … — left as-is; re-run to retry` and **continues** the batch. **Open issues
  are processed before closed** (a closed-issue failure never blocks the rest). The stale label is
  deleted **only** if every relabel succeeded; otherwise a never-silent `! '<old>' still on N issue(s) —
  not deleted; re-run` FLAG skips the delete (G2 — never break the label↔issue link silently).
- **`--verbose`** — echoes `→ gh <args>` to stderr before each invocation so a hang is pinpointable to
  the exact call. `--dry-run` / `--self-test` preserved; `--self-test` extended to cover the new pure
  logic. Verified: `python3 tools/github/gh-issues-sync.py --self-test` green; ruff lint + format clean.
  No live/networked `gh` was run.

### Added (2026-06-20: `doc-status` currency gate + normalize statuses to the `Enacted` lattice)
The follow-up flagged by the `Enacted`-lattice change (#236): a status sweep + a gate that enforces it.
- **Status normalization (append-only — forward only, no status invented):** the compound spelling
  `Accepted — Enacted` → the canonical standalone **`Enacted`** wherever it denotes a doc's status —
  the `Status` headers of **RFC-0012/0013/0014** (and the r3-specific `Accepted — r3 ENACTED` →
  **`Enacted (r3)`** for **RFC-0011**), each with an append-only changelog-footer note. **RFC-0015**
  promoted `Accepted → Enacted` (its §4 design was ratified Accepted and is landed in
  `mycelium-lsp/src/baseline.rs`, M-362 — a stepped-through forward move). Live body citations of those
  RFCs in `docs/spec/stdlib/diag.md` + `recover.md` normalized too (dated changelog-footer history left
  verbatim).
- **Stale `docs/rfcs/README.md` rows fixed to the authoritative headers** (re-verified per RFC): 0008,
  0010 → **Accepted**; 0011 → **Enacted (r3)**; 0012/0013/0014/0015 → **Enacted**; 0018, 0019 →
  **Accepted**; 0020 → **Accepted (scoped)**; 0021 → **Accepted (framework)**. Status-set prose + the
  template Status line updated to the new lattice.
- **New `doc-status` gate** (`scripts/doc_status_check.py` + `scripts/checks/doc-status.sh` +
  `just doc-status`, wired into `just check`, skip-graceful if python3/PyYAML absent): three never-silent
  passes — (1) **lattice** validation of every decision doc's `Status` header
  (`{Draft, Proposed, Preliminary, Accepted, Enacted, Superseded, Resolved}`; flags the legacy compound),
  (2) **nav-README ↔ authoritative-header cross-check** (the exact drift that left the 8 stale rows),
  (3) **Declared stale-phrase invariants** from `tools/doc-status-invariants.yaml` (e.g. once the stdlib
  is Accepted-or-later, no nav README may say "pending ratification"). Honest posture: a **Declared**
  line/regex heuristic — source is ground truth; the pass-3 rules are maintainer-Declared, never inferred
  (documented in the script header + `scripts/README.md`). Verified: `just check` green; a negative test
  confirms the cross-check catches a re-introduced stale row.

### Changed (2026-06-20: add `Enacted` as a first-class decision status)
Extended the append-only status lattice in `CLAUDE.md` (house rule #3) + `CONTRIBUTING.md` from
`Draft/Proposed → Accepted → Superseded` to **`Draft/Proposed → Accepted → Enacted → Superseded`**
(notes still `→ Resolved`). **`Enacted`** = an Accepted decision now fully implemented/landed —
complete and stable, outside ongoing maintenance + future-dev integration — and it **must step
through `Accepted`** first (no skipping). The canonical spelling is the standalone `Enacted` (earlier
docs used the compound `Accepted — Enacted`). Normalizing existing statuses to the new spelling, the
stale `docs/rfcs/README.md` rows, and a `doc-status` currency gate that enforces the lattice +
nav-doc↔header consistency are a follow-up.

### Changed (2026-06-20: docs/README refresh — ratification status + native-path honesty)
Refreshed the prose READMEs to the current, verified state: the top-level `README.md` and
`docs/spec/stdlib/README.md` now report the **23 Rust-first stdlib specs as `Accepted`** (2026-06-20,
DN-07, on a checked basis — `runtime` + `self-hosting-readiness` stay `Draft`), replacing the stale
*"implemented (Rust-first), pending ratification" / never-silently-Accepted* wording. Corrected the
native-path claims (VR-5): the **direct-LLVM native path** (M-373/M-378/M-379), **JIT** (M-340), and **hot-inject** (M-341) are
built; the real `ternary`→arith/vector→LLVM **MLIR-dialect lowering (M-601)** is **unblocked**
(libMLIR now provisionable on Linux, M-348) **and in progress** — not yet complete — so it moves to the
"in progress" bucket. Also added a squash-only / `/land` note to the README contributing section. Docs
only; append-only (each spec's own Status line was already moved by the DN-07 pass).

### Changed (2026-06-20: squash-only-into-main + curated-commit merge policy)
Baked the merge discipline into `CLAUDE.md` (Commits & PRs + the Autonomous PR workflow): every PR lands on
`main` as a **single squash commit** for a linear, bisectable history (the internal swarm leaf→epic→orch
merges stay octopus/`--no-ff` to preserve lineage; only the `main` landing squashes), and the squash commit
is **curated** — a clear subject + body describing the net change, never the auto-concatenated WIP/fixup/merge
trail. Process/operating-guide change only; codifies existing practice. (Future: fold into `.claude/skills/` +
tooling.)

### Changed (2026-06-20: stdlib spec ratification pass — 23 specs → Accepted, DN-07)
Maintainer ratification (DN-07, append-only) of the implemented Rust-first standard-library specs, after the
DN-16 readiness survey + an independent review:
- **Ratified → `Accepted` (23 specs):** `cmp`, `collections`, `content`, `core`, `dense`, `diag`, `error`,
  `fmt`, `fs`, `io`, `iter`, `math`, `numerics`, `rand`, `recover`, `select`, `spore` (library/manifest half),
  `swap`, `ternary`, `testing`, `text`, `time`, `vsa`. Each carries an asserted §4.5 guarantee matrix; the open
  §7/§8 questions are design/scope calls, not contract violations.
- **Honesty preserved (VR-5) — no guarantee tag upgraded without a checked basis:** the `dense`/`math`/`vsa`/
  `numerics` `Proven` rows were verified/aligned by M-377 (dense elementwise `Proven` finalized via the ADR-010
  IEEE backward-error bound with the finiteness side-condition guarded, M-512 delivered; accumulation rows held
  at `Empirical`; math all `Declared`; vsa cells mirror the cited, test-guarded RFC-0003 §4 matrix).
- **Two pre-ratification fixes:** `cmp` trait naming (`MycEq`/`MycOrd`/`MycPartialOrd`) documented in spec §3
  (a Rust namespace-collision parity note, not honesty); `spore` §7-Q1 ring placement reconciled to **Ring 1 /
  Tier A** via a maintainer-authorized **RFC-0016 §4.2 erratum** (the §4.2 parenthetical was the outlier vs
  §4.3; corrigendum, not a decision reversal).
- **Deferred (stay `Draft`):** `runtime` (Phase-7 / RFC-0008 constructs, no crate) and `self-hosting-readiness`
  (M-502 gate; verdict "not yet established"). Incomplete Draft RFCs/ADRs/DNs (DN-04/05/14/15/17, ADR-012) were
  **not** ratified — they are open or superseded, and ratifying them would bless incomplete work (grounding).
Append-only — no spec rewritten; status headers + per-spec changelog footers + Doc-Index rows moved forward.
See DN-16 (Resolved) for the readiness survey and DN-07 for the ratification posture.

### Changed (2026-06-19: bake the autonomous PR/merge workflow into CLAUDE.md)
Institutionalized the review-before-merge discipline the native waves (M-378/M-379) converged on, as a
new **"Autonomous PR workflow"** section in `CLAUDE.md`: the merge gate is the agent's (self-review with
the `/pr-review` lens → handle every CI/bot review comment, defer honestly rather than ship fragile
output → full `just check` green → merge up), with **pull-down-before-merge-up** (keep tips current, never
integrate across divergent history) and **branch children from a *pushed* tip** (avoid the M-379 Stage-2
worktree-divergence). Process/operating-guide change only — no corpus or code behavior changed.

### Added (2026-06-19: M-379 — direct-LLVM native tail-recursion (Fix) + Binary branch, Increment-3 of M-373)
Wave-1 native continuation, built as a **swarm** (orchestrator + a Sonnet leaf in an isolated worktree,
deconflicted by pulling the parent down). The direct-LLVM backend (`crates/mycelium-mlir/src/llvm.rs`)
now compiles **stack-robust tail-recursion**, discharging the DN-05 #1 requirement for the native path.
`FixGroup`, non-tail recursion, and recursive heap data stay explicit `UnsupportedNode`. Guarantee tag
stays **`Declared`** (hand-written IR + the empirical M-302 differential, not Proven — VR-5); every
out-of-scope case is an explicit refusal (G2); every IR stage dumpable (no opaque pass — RFC-0004 §6/VR-4).
- **Design-first (append-only).** **RFC-0004 §11.6 (r5)** sanctions Increment-3 and discharges the
  §11.3-banked DN-05 #1 requirement; **DN-15 §8** records the realized design; the §5 table row 3 →
  partial (tail landed). No Accepted text rewritten (house rule #3).
- **Binary branch primitive.** The two duplicated `Match` handlers are factored into one `lower_match`
  (behaviour-preserving), extended with a `Binary{8}`-lane scrutinee + `Lit` arms (pack the lane,
  `switch i64` on packed literals) — the base-case conditional terminating recursion needs.
- **Tail-recursion as a loop (DN-05 #1).** `App(Fix{λn. Match n {…}}, init)` lowers to an **iterative
  LLVM `phi` loop** — a tail self-call is a back-edge that updates the loop variable, so the host C
  stack is **O(1) by construction** (no unbounded C stack; never SIGSEGV). A depth counter is checked
  each iteration against an **`AutoDepthBudget`-resolved ceiling** (`budget.rs`, M-349 — reused from the
  AOT env-machine); exceeding it raises a **graceful `AotError::DepthLimit`** via a distinct read-back
  sentinel (`#`). The diverging no-base-case exit traps with `@abort`+`ret` (never raw `unreachable`, G2).
- **Still refused (`UnsupportedNode`, G2):** non-tail recursion, `FixGroup`, non-`λ.Match` `Fix` bodies,
  recursion over non-`Binary{8}`, and a bare-`Fix` program result.
- **Differential (the gate, NFR-7).** New `crates/mycelium-mlir/tests/recursion_differential.rs`: a
  Lit-match-encoded countdown + a one-step recursion value-checked interp ≡ native, a non-terminating
  program reaching a graceful `DepthLimit` (parity with the interpreter's `FuelExhausted` — both
  explicit, non-silent), and three refusal tests. 114 `mycelium-mlir` tests green; clippy `-D warnings`
  clean; api baseline + agent index updated for the new `AotError::DepthLimit` variant.

### Added (2026-06-19: M-378 — direct-LLVM native closures (App/Lam) + heap, Increment-2 of M-373)
Wave-1 native continuation. The direct-LLVM backend (`crates/mycelium-mlir/src/llvm.rs`) now compiles
`App`/`Lam` to **native closures** via closure-conversion, extending Increment-1's non-recursive
`Construct`/`Match`. `Fix`/`FixGroup` stay explicit `UnsupportedNode` (Increment-3, separate session).
Guarantee tag stays **`Declared`** (hand-written IR + the empirical M-302 differential, not Proven —
VR-5); every refusal is explicit (G2) and every IR stage dumpable (no opaque pass — RFC-0004 §6/VR-4).
- **Design-first (append-only).** **RFC-0004 §11.5 (r4)** scopes the Increment-2 sanction under the
  §2 revisit clause; **DN-15 §7** records the realized design (ABI, no-GC strategy, free-var analysis);
  the §5 increment table marks row 2 landed. No Accepted text rewritten (house rule #3).
- **Closure conversion.** A lexical free-variable analysis over the public ANF (`closure_free_vars`;
  no `mycelium-core` change) computes each `Lam`'s captured set → a heap closure record
  `[fn_ptr | capture_0 … capture_k]`; `App` loads `fn_ptr` from slot 0 and emits the indirect
  `call i64 %fp(i8* %env, i64 %arg)`.
- **Narrow value ABI (DN-15 §7.1).** Closures carry/return `Binary{8}` values packed as one `i64`.
  Other widths, `Ternary`, datums across the boundary, currying (closure-as-argument/result), a
  non-closure `App` head, and a closure-valued program result are all explicit `UnsupportedNode`.
- **No-GC strategy: bump arena (DN-15 §7.2).** One `@malloc`d block, records bump-allocated through a
  single `@myc_arena_alloc` seam (never-silent over-capacity `@abort`), `@free`d before normal
  completion. Heap is required (closures escape their creating frame); allocation is statically
  bounded (no recursion). The seam is where **Increment-3** attaches a `DepthBudget`-resolved ceiling
  (DN-05 #1; `budget.rs`, M-349) — designed in, not retrofitted. Closure-free programs emit
  byte-for-byte the same module as before (no arena, no extra declares).
- **Differential (the gate, NFR-7).** Extended the M-302/M-210 three-way differential with a
  7-program `closure_corpus` (identity, single/two captures, body-local const, result-feeds-an-op, a
  nested capturing lambda) + a body-discriminating mutant-witness + an updated refusal-parity test
  (`Fix`/currying/bare-`Lam` still refused). The JIT path refuses closures explicitly (AOT-only).
  11 `mycelium-mlir` differential tests green; `docs/api-index/` regenerated.

### Changed (2026-06-19: M-377 — DN-16 stdlib honesty cleanups, grounded + maintainer-ratified)
Acted on DN-16's actionable cross-cutting divergences (DN-16 → **Resolved**), each grounded in code
first (VR-5). No spec moved to Accepted — per-spec ratification stays the maintainer's call.
- **`Proven` rows verified.** `math` already tags all approx ops `Declared` (no `Proven`; no change).
  `vsa` `Proven` cells *mirror* the RFC-0003 §4 kernel matrix (cited, test-guarded; honest by
  construction; no change). **`dense` Q1 finalized:** elementwise float `add`/`sub`/`scale`/`hadamard`
  → `Proven` (ADR-010 per-element IEEE backward-error bound; finiteness side-condition guarded by
  `DenseError::NonFinite`; M-512 delivered), and the `dense.md` §4 **accumulation rows (`sum`/`dot`)
  aligned down to `Empirical`** to match the landed crate (`nγ_n` is a distinct unchecked theorem —
  VR-5-safe downgrade; the spec table had over-claimed).
- **fmt/io `from_json` framing resolved scope-distinct (both tags kept).** `fmt`-`Exact` = decode
  determinism; `io`-`Empirical` = round-trip fidelity (proptest, no theorem). Different properties of
  the same call; neither over-claims. Cross-referenced in both crate guarantee matrices + both specs.
- **swap §3 pinned** to the landed `mycelium-std-swap` surface (`check_swap` → `Result<GuaranteeStrength,
  CheckError>`, the re-exported `mycelium_cert::check` (M-210), no `build`, richer `CheckError`); §7-Q4 resolved.
- Remaining minor (deferred): the `cmp` `MycEq`/`MycOrd` naming-vs-spec doc gap (not honesty-relevant).

### Changed (2026-06-19: M-376 housekeeping wave — behaviour-preserving DRY/footprint, no-op)
Executed DN-17 §4 P1+P2 (P3 skipped per YAGNI). Pure refactor — no source logic, no guarantee tags,
no public API changed; `Cargo.lock` unchanged; `cargo test --workspace` + full `just check` green; net ~−11 LOC.
- **P1 — workspace dependency dedup.** Added `[workspace.dependencies]` (`serde`/`serde_json`/`blake3`)
  to the root `Cargo.toml` and converted **15 crate manifests** to `*.workspace = true` (single-point
  version control; standard Rust convention). Versioned `xtask`'s 5 path-deps (`version = "0.0.0"`),
  clearing the `cargo deny` `wildcard` advisory.
- **P2 — MLIR test-helper consolidation.** Factored 7 byte-for-byte-identical differential helpers
  (`byte`/`tern`/`i64_value`/`observable`/shared consts) into `crates/mycelium-mlir/tests/common/mod.rs`
  across 6 test files; divergent helpers (different LCG seeds, `int_to_trits` variant) kept local so no
  test behaviour changes. All 24 `mycelium-mlir` tests green (incl. the M-302 `data_corpus` + the
  `match_default_arm` regression).
- **P3 — deferred (YAGNI).** A shared `assert_is_std_error` helper would need a new dev-dep test-util
  crate to save ~8 trivial one-liners across 3 crates — over-engineering. The error-**type** unification
  stays deferred to post-stdlib-ratification (DN-17 §4/§5). **M-376 → done.**

### Added (2026-06-19: DN-17 codebase housekeeping / DRY survey — planning capture, M-376)
- **`docs/notes/DN-17`** (new, Draft) — a grounded read-only DRY/duplication survey of the ~43-crate
  Rust workspace + a priority-ordered, risk-tagged plan for a future **behaviour-preserving**
  housekeeping wave (**M-376**, `status:needs-design`). P1: hoist repeated external deps
  (serde/serde_json/blake3) to `[workspace.dependencies]` + version the 5 xtask path-deps (silences
  the cargo-deny wildcard advisory). P2: consolidate the duplicated MLIR differential test helpers.
  P3: a shared `assert_is_std_error` test helper — the error-**type** unification is **deferred**
  (YAGNI; error shapes still evolve per spec). Intentional duplication (M-540 ambient docstrings,
  per-module guarantee matrices) is explicitly **not** a target. Standard Rust conventions now;
  Mycelium-native conventions deferred until self-hosting (DN-14). No code changed — survey only.

### Fixed (2026-06-19: Wave-5 — PR #213 review fixes, all honesty/never-silent)
Addressing the Copilot review on PR #213 (8 comments, 4 issues — all legitimate):
- **`crates/mycelium-mlir/src/llvm.rs`** — `Rhs::Match` now **lowers the `default` arm** when present
  (it was silently routed to `abort()` — a real interp↔native divergence whenever the default is taken;
  NFR-7/G2). `abort()` is kept only when `default` is `None` (WF7 checker-proven coverage). Fixed at both
  the top-level and nested (`lower_anf_block`) Match sites. Added an explicit **binder/field arity check**
  at both sites (mirrors the interpreter's `DataMalformed`; never a silent truncation).
- **`crates/mycelium-mlir/tests/native_differential.rs`** — new regression test
  `match_default_arm_is_taken_and_observationally_equivalent` (scrutinee tag misses the explicit arms ⇒
  default taken ⇒ interp value vs native; would have caught the bug). Closes the exhaustive-corpus gap.
- **`crates/mycelium-std-sys/src/rand.rs`** — rand tests now **skip gracefully** on
  `EntropyError::Unavailable` (repo idiom); dropped the probabilistic successive-differ content assertion
  (a quality check belongs in a statistical audit, not a unit test; VR-5).
- **RFC-0004 §11.2 (r3) + DN-15 §4.1** — corrected to the **landed stack-`alloca`** representation
  (the increment uses `alloca [N+1 x i64]`, not heap `@malloc`/OOM); non-recursive/bounded ⇒ static
  allocation depth ⇒ no OOM path. Grounding consistency (house rule #4); r1/r2/§1–§10 untouched.

### Added (2026-06-19: Wave-5 landed — M-373 native direct-LLVM increment, M-374 DN-16 survey, M-375 rand real entropy)
**Wave-5 octopus-merged (2026-06-19)** into `claude/orch-wave5-mlir-unblock-s0li6o`. Three streams,
disjoint directories ⇒ conflict-free; 6 files, 1661 insertions; full workspace tests green; no new
public API surface (api-index unchanged); `issues.yaml` validated, no duplicate ids.

- **M-373 — direct-LLVM native data-fragment increment** (the unblocked half of the decomposed M-348).
  - **DN-15** (new, Draft) — honest split of M-348 into the **libMLIR-gated** dialect lowering
    (`dialect.rs` skeleton; stays **blocked**, VR-5) vs the **direct-LLVM-advanceable** `llvm.rs`
    extension (RFC-0004 §2 revisit clause); a per-increment table grading each "needs libMLIR? /
    tractable now? / risk".
  - **RFC-0004 r3** (append-only) — new **§11** scopes the direct-LLVM data-fragment increment under
    the §2 revisit clause; **no r1/r2 Accepted text rewritten** (house rule #3).
  - **`crates/mycelium-mlir/src/llvm.rs`** — natively compiles non-recursive `Construct` (stack
    `alloca [N+1 x i64]`, tag at slot 0) + `Match` (`switch i64` with an explicit `@abort()` default —
    never silent UB, G2; `phi i32` merge). `App`/`Lam`/`Fix`/`FixGroup` split into separate explicit
    `UnsupportedNode` refusals — recursion/closures stay honestly deferred (VR-5). Guarantee `Declared`.
  - **M-302 differential** extended: a 5-program `data_corpus` (interp↔native observational equivalence
    via the shared M-210 checker) + a test asserting the closure/recursion refusal still holds. 104 tests pass.
- **M-374 — `docs/notes/DN-16`** (new, Draft). Per-spec stdlib ratification-readiness survey (all 25
  specs vs their crates): 1 ready, 17 ready-with-flags, 2 ready-scoped, 3 divergent, 2 not-implemented.
  Advisory (DN-07 posture) — **no spec moved to Accepted**; divergences flagged for the maintainer.
- **M-375 — `crates/mycelium-std-sys/src/rand.rs`**. `fill_bytes` now reads real OS entropy from
  `/dev/urandom` (pure-std `std::fs` + `read_exact`; **no new dep**, `#![forbid(unsafe_code)]` preserved);
  all failure paths are explicit `EntropyError` (never panic/zero-fill, G2). Tag stays **`Declared`**
  (real kernel CSPRNG, but no documented quality trials — VR-5). 18 std-sys tests green. Resolves FLAG-RAND-IMPL.

**Orchestrator integrating edits:**
- `tools/github/issues.yaml` — M-373/M-374/M-375 → `status:done` (honest DONE notes); M-348 decomposition
  recorded, its libMLIR-gated half **unchanged (`status:blocked`)**.
- `docs/Doc-Index.md` — registered DN-15 + DN-16 in §1.
- `CHANGELOG.md` — this entry.
- `scripts/install-tools.sh` + `scripts/README.md` — snapshot-cached cloud Setup-script toolchain provisioning.

### Launched (2026-06-19: Wave-5 — decompose the M-348 libMLIR wall; land a real native increment with zero libMLIR)
**Wave-5 launched** on `claude/orch-wave5-mlir-unblock-s0li6o` (Opus orchestrator / Sonnet swarm below).
The wave's thesis: M-348 ("provision libMLIR to unblock the native MLIR→LLVM path", status:blocked) was
treated as a single external wall but is actually TWO parts — (a) genuinely libMLIR-gated (the real
`ternary`→arith/vector→LLVM dialect lowering; `dialect.rs` is a textual skeleton) which STAYS blocked
honestly (VR-5), and (b) advanceable NOW with zero libMLIR (the direct-LLVM-IR backend `llvm.rs`, sanctioned
by the RFC-0004 §2 "lighter direct-LLVM backend" revisit clause). Step 0 decomposed M-348, minting:
- **M-373** (in-progress) — direct-LLVM native data-fragment increment (the unblocked half of M-348);
  Epic 1 spine: Leaf 1A (DN-15 decomposition design + append-only RFC-0004 revision), Leaf 1B (extend
  `llvm.rs` to natively compile a non-recursive Construct/Match sub-fragment; App/Lam/Fix/FixGroup stay
  honest `UnsupportedNode`; extend the M-302 differential).
- **M-374** (in-progress) — stdlib spec ratification-readiness survey (DN-16, docs-only; surfaces
  recommendations, ratifies nothing).
- **M-375** (in-progress) — std-sys rand real OS entropy (resolve FLAG-RAND-IMPL; tag stays honest).

M-348's body records the decomposition; its blocked half (the libMLIR dialect layer) is unchanged.

### Added (2026-06-19: Wave-4B landed — M-541 std-sys FFI floor, M-502 DN-14 self-hosting gate, M-540 RFC-0012 ergonomics annotations)
**Wave-4B octopus-merged (2026-06-19).** Two parallel epics (M541A + M502A) conflict-free merged into
`claude/orch-wave4-docs-sys-4ifdo4`; 30 files, 989 insertions; all tests green (15 new std-sys tests);
`docs/api-index/` regenerated to include new std-sys public surface.

- **M-541 — `crates/mycelium-std-sys/`** (new crate). Minimal audited FFI/syscall floor (RFC-0016
  §8-Q6 / LR-9): four modules, all `[Declared]` (unaudited libm/syscall floor; honest per VR-5):
  - `math` — 14 transcendental wrappers (sin/cos/tan/asin/acos/atan/atan2/exp/exp2/ln/log2/log10/sqrt/cbrt)
  - `rand` — `fill_bytes(buf) → Result<(), EntropyError>`; v0 uses DefaultHasher+SystemTime (non-cryptographic stand-in; explicitly Declared; getrandom replacement deferred — FLAG-RAND-IMPL)
  - `fs` — `read`/`write`/`exists`/`create_dir_all`/`remove_file` over `std::fs`
  - `time` — `wall_nanos()`/`mono_nanos()`/`sleep_nanos()` over `std::time`
  - 15/15 tests pass. `cargo clippy -D warnings` clean. `#![forbid(unsafe_code)]`.

- **M-502 — `docs/notes/DN-14-Self-Hosting-Gate.md`** (new, Draft). Honest self-hosting readiness
  assessment grounded in actual source (`crates/mycelium-l1/src/checkty.rs`, `ast.rs`).
  Verdict: **5/11 features present, 5 gate-fails, 1 partially missing** — self-hosting not established.
  Gate-fails: generics (monomorphic-only), trait interfaces (v0 deferred), effect annotations (no AST
  syntax), `wild`/FFI blocks (LR-9 denied by default), static guarantee index (stage-0 only).
  No feature stamped present on intent alone (VR-5).

- **M-540 — RFC-0012 ambient-representation `//!` docstring annotations** across all 23
  `crates/mycelium-std-*/src/lib.rs` (292 insertions). Each block states the RFC-0012 ambient contract
  (representation implicit at call site, always reified/queryable/EXPLAIN-able), carries
  RFC-0012/DN-07/M-540 traceability, and adds a crate-specific grounded sentence.
  Annotation-only pass — no new `pub` items, no new `#[test]` blocks, `cargo check --workspace` clean.

**Orchestrator integrating edits:**
- `Cargo.toml` — added `"crates/mycelium-std-sys"` to workspace `members`
- `docs/Doc-Index.md` — registered DN-14 in §1 corpus table
- `docs/api-index/` — regenerated (new std-sys public surface; 2561 items located, 3412 flagged)
- `CLAUDE.md` — added "Swarm failure-mode mitigations" section (lessons from Wave-4)

### Added (2026-06-19: Wave-4A landed — E3-8 agent documentation index, M-392..395)
**Wave-4A octopus-merged (2026-06-19).** Epic E3-8 "Agent-facing documentation index" merged into
`claude/orch-wave4-docs-sys-4ifdo4`; 5 commits above base; all Rust tests green; doc-index drift gate ok.

- **M-392 — `tools/docgen/code_index.py`** (pure Python stdlib). JOINs the committed
  `docs/spec/api/*.txt` public-API snapshots against the source tree to build a navigational index.
  2561 items located; 3412 items flagged (re-exports, macro-generated, cfg-gated — G2, never silently
  dropped). `--self-test` verifies determinism + completeness. Honesty tag: `Empirical/Declared`
  (line/regex heuristic; source is ground truth).
  - `docs/api-index/index.json` — machine-readable symbol table (symbol, kind, crate, file:line, summary).
  - `docs/api-index/INDEX.md` — grep-friendly table grouped by crate; human/agent context lookup.
- **M-393 — `just docs` + `just docs-index`** + `scripts/checks/doc-index.sh` drift gate wired into
  `scripts/checks/all.sh` and `just check`. Mirrors the `just api` committed-snapshot pattern.
  `just docs` (rustdoc HTML, target/doc, NOT committed); `just docs-index` (regenerate committed index).
- **M-394 — `tools/github/doc_refs_check.py`** (requires PyYAML). Validates `doc_refs:` list entries in
  `issues.yaml`: `api:<crate>::<path>` → index.json, `corpus:<DOC>[#<anchor>]` → Doc-Index.md,
  `src:<path>[:<line>]` → file on disk. Never-silent on dangling refs. Extended `manifest-check.py`.
  Backfilled `doc_refs:` on all 52 open issues in `issues.yaml` (Wave-4B issues pre-wired).
- **M-395 — `CLAUDE.md` updated** with "Auto-generated docs & the agent index" section +
  `docs/api-index/` added to orchestrator-owned collision-surface list. `dev-workflow` skill updated
  (run `just docs-index` after any public-API change). `changelog` skill updated (index parity note).
  New `.claude/skills/doc-index/SKILL.md` — `/doc-index` skill for regeneration, query, and doc_refs validation.
- **`docs/Doc-Index.md` §7** added registering `docs/api-index/`.
- **`tools/llm-harness/coauthor.py`** ruff format + unused-import fixes (pre-existing FLAG-1 resolved
  in orchestrator integration pass).
- **E3-8, M-392..395** → `status:done` in `issues.yaml`.

Append-only.

### Changed (2026-06-19: wave-4 step-0 reconciliation + wave-4A/4B launch record)
**Issues reconciliation (step-0).** Three tasks implemented and tested but mislabelled
`status:needs-design` corrected to `status:done`:
- **M-350** (Resonator-network factorization) — evidence: `crates/mycelium-vsa/src/resonator.rs`
  (893 lines); `tests/resonator_oracle.rs` + `tests/resonator_profile.rs` pass; convergence regime
  documented; `StopReason::{BudgetExhausted,Oscillating,Stalled}` explicit; `ResonatorProfile`
  carries bounds; never silent (G2 / VR-5).
- **E3-5** (epic: resonator factorization) — M-350 complete.
- **E3-1** (epic: semantic-level projections) — M-380 LlmCanonical enacted; M-381 is a
  non-blocking P3 research track, not a gate on the epic.

**Wave-4A tasks added to issues.yaml** — E3-8 (agent-facing documentation index epic) +
M-392/M-393/M-394/M-395 (toolchain tasks; fill a gap in the M-300–M-400 block):
- **E3-8** — Agent-facing documentation index & auto-doc workflow (note: the Wave-4A plan called
  this E3-6; E3-6 was already taken by the BitNet acceleration epic, so assigned E3-8)
- **M-392** — Agent code-index generator (`tools/docgen/code_index.py`, `docs/api-index/`)
- **M-393** — `just docs` + `just docs-index` + staleness drift gate
- **M-394** — Issue/epic ↔ index cross-references (`doc_refs:` grammar + backfill)
- **M-395** — Workflow integration (CLAUDE.md + `.claude/skills/`)

**Wave-4A swarm launch.** Sonnet Swarm (all agents claude-sonnet-4-6). Orchestrator on
`claude/orch-wave4-docs-sys-4ifdo4`. Epic E3-8 on branch `claude/epic/E36A0-agent-doc-index`;
leaf M-392 on `claude/leaf/E36A0-M392A-code-index-gen`. M-393/394/395 are epic-direct (no
separate leaf). Wave-4A must land before Wave-4B fan-out so Wave-4B agents can read the index
and their issues carry `doc_refs`.

**Wave-4B swarm launch** (to fan out after Wave-4A merges):
- **Epic A (M-541 std-sys)** — `claude/epic/M541A-std-sys-ffi-floor`; owns new crate
  `crates/mycelium-std-sys/` (audited FFI floor per RFC-0016 §8-Q6 + DN-07).
- **Epic B (M-502 + M-540)** — `claude/epic/M502A-self-host-ergonomics`; owns
  `docs/notes/DN-14-Self-Hosting-Gate.md` (new, M-502) + `//!` ambient-representation
  docstring additions across 23 std crates (M-540, annotation-only).

No code merged yet; epics will octopus-merge into this branch after reporting. Append-only.

### Changed (2026-06-19: issues reconciliation + wave-3 swarm launch record)
**Issues reconciliation.** `tools/github/issues.yaml` brought current with the implemented state —
43 Phase-3–5 tasks updated to `status:done` (confirmed against crate presence in the workspace),
M-360 and M-330 set `status:in-progress` (work in-flight on epic branches), M-380 body
disambiguated (KC-2 gate met; RFC-0021 Accepted; LlmCanonical prototype enacted). No creative
state invented; every status backed by a crate or ratified decision (VR-5, G2).

**Wave-3 launch (planning record — code lands via octopus-merge after epics report).**
A **Sonnet Swarm** (all agents on claude-sonnet-4-6) with two independent epics on disjoint
directory surfaces; neither has been merged here yet:

- **Epic E360 (M-360 — TL1/TL2 SIMD vectorized kernels, branch
  `claude/epic/E360-m360-tl1-tl2-simd`).** Extends `crates/mycelium-mlir/src/simd.rs` with TL1
  and TL2 LLVM-IR vector decode kernels; scalar oracle differential and E1-harness section-5
  extension. Guarantee: speedup `Empirical` (as measured by E1).
- **Epic E330 (M-330 — AI co-authoring loop harness, branch `claude/epic/E330-m330-coauthor-loop`,
  commit `caaeed3`).** Completed. Delivers `tools/llm-harness/coauthor.py` and
  `crates/mycelium-lsp/examples/check.rs`; pending octopus-merge into this branch.

**Octopus-merge landed (2026-06-19).** Both epics merged conflict-free into this branch; 135 test
suites green (0 failures); `clippy -D warnings` clean. Statuses updated to `status:done`.

- **M-360 (E360) — TL1 + TL2 hand-vectorized SIMD dot kernels (`crates/mycelium-mlir/`, `xtask/`).**
  - `simd.rs`: `emit_bitnet_dot_simd_tl1_ir()` + `compile_bitnet_dot_simd_tl1()` — TL1 decode uses
    `select(code == 2, −1, code)` (no modulo, SIMD-efficient); `emit_bitnet_dot_simd_tl2_ir()` +
    `compile_bitnet_dot_simd_tl2()` — TL2 unpacks the 3-trit→5-bit LUT bitstream at fixed period-8
    offsets. Four unit tests (IR inspectability + correctness corpus for each packing).
  - `lib.rs`: all 6 public symbols exported (2 emitters + 2 compilers for TL1/TL2, plus the
    existing I2_S pair).
  - `tests/simd_differential.rs`: two new integration tests routing TL1 and TL2 through the shared
    M-210 checker (`tl1_simd_and_scalar_agree_…`, `tl2_simd_and_scalar_agree_…`).
  - `xtask/src/e1.rs`: `simd_section()` extended to compile, correctness-gate, and benchmark all
    three kernels (I2_S, TL1, TL2) against their scalar counterparts. Guarantee: speedup `Empirical`
    (as-measured by E1 — never `Declared`). No FLAGs.
- **M-330 (E330) — AI co-authoring loop harness (`tools/llm-harness/`, `crates/mycelium-lsp/`).**
  - `tools/llm-harness/coauthor.py` (1212 lines, pure stdlib, Termux-portable): MockGenerator
    cycles 7 canned programs (4 valid, 3 fixable), self-corrects by default; LlmGenerator shells
    to `llama-cli`/HTTP server (SKIP when absent, G2); Checker shells `cargo run -p mycelium-lsp
    --example check` (KC-3, no Python kernel deps); VR-5 enforcement scans generated source for
    forbidden tags (`Proven`/`Exact`); G11 dual projection writes `reports/coauthor-<ts>.json` +
    `.txt`; statuses `PASS`/`mock-PASS`/`PARTIAL_PASS`/`SKIP`/`FAIL` all explicit (G2).
  - `crates/mycelium-lsp/examples/check.rs` (26 lines): stdin→publishDiagnostics shim; reads
    Mycelium source, calls `publish_for_source`, writes JSON-RPC to stdout. Zero new Rust deps.
  - `crates/mycelium-lsp/Cargo.toml`: `[[example]] name = "check"` entry added.
  - Verified: `--mock` exit 0; 4 `mock-PASS` + 3 `PARTIAL_PASS` (self-correction demonstrated).
    Guarantee: `Declared` (design phase; mock mode only; real LLM is API-gated SKIP). No FLAGs.

Append-only.

### Added (2026-06-19: M-391 — surface mutual-recursion elaboration confirmed; RP-6 resolved)
The R7-Q3 *surface* half — how a group of ≥2 mutually-recursive top-level functions is written — is
decided and pinned. **RP-6 verdict (DN-13): nodule-wide mutual visibility, no new syntax** — every
top-level `fn` in a `nodule` is mutually visible, and the elaborator auto-groups each call-graph SCC of
≥2 into the RFC-0001 r5 `FixGroup`. The lowering already existed (M-343); the inferred grouping is
materialized as that concrete, content-addressed L0 node (no black box).
- **M-391 (#199).** Confirms + pins the surface front-end (already realized by M-343's nodule-wide
  scoping + Tarjan→`FixGroup` path): the M-210 three-way differential
  (`crates/mycelium-l1/tests/differential.rs`) gains two further surface-written mutual-recursion
  shapes — a repr-returning pair and a multi-field-constructor pair — each agreeing across
  L1-eval ≡ elaborate→L0-interp ≡ AOT through the shared checker; an **identity** assertion pins the
  deterministic `FixGroup` lowering (ADR-003); a **never-silent** regression pins that a reference to an
  undefined function stays an explicit checker error, never a silent phantom group member (G2). No
  change to `mycelium-core` / `mycelium-interp` / the totality checker, and **no public-API/baseline
  change** (KC-3; DN-10 §2.5).
- **RP-6 record (DN-13).** New decision note records the verdict + rationale (least append-only surface
  commitment; KISS; Rust/Unison/ML-module consistency). `docs/spec/grammar/mycelium.ebnf` gains a
  scoping comment (**no production change**); RFC-0007 §8 R7-Q3 (surface half) resolved (append-only);
  `docs/notes/research-prompts.md` RP-6 → Resolved; DN-10 §2.6 annotated.

Verified: `cargo test -p mycelium-l1` (all suites green, incl. the new differential / identity /
never-silent tests); `clippy --all-targets -D warnings -A unsafe_code` clean; `cargo fmt --check` clean.
Append-only.

### Changed (2026-06-19: documentation currency + `fmt`→`io` JSON delegation — Sonnet swarm wave)
The navigational/overview docs are brought current with the implemented state and guarded by a new
programmatic check, and the converged `fmt`→`io` canonical-JSON seam is wired. Landed as a **Sonnet
review/polish swarm** — two leaves in isolated worktrees, integrated by the orchestrator (who owns the
shared files). The swarm earned its keep: the review leaf caught two real delegation bugs the
first-pass code missed.

- **Docs currency (M-371, #202).** `README.md`, `docs/Doc-Index.md`, and
  `docs/Mycelium_Project_Foundation.md` move off the pre-code *"design phase"* framing to the honest
  current state — **42 crates** (+ `xtask`), the reference interpreter + certified swaps + the 23-crate
  Rust-first stdlib; Phases **0–3, 5, 7 complete**, **4, 6, 8 in progress**. README gains an accurate
  repository-structure tree (`crates`/`tools`/`experiments`/`proofs`/`scripts`/`examples`/`justfile`/…),
  a *"The Rust workspace"* + *"Build & checks"* section, decisions/reading-order through
  RFC-0021/ADR-017/DN-12, and honest *Status & open items* (KC-1 **SAFE**; KC-2 **proceed** per DN-09;
  self-hosting **M-502 not established**; specs *"implemented (Rust-first), pending ratification"* — not
  silently `Accepted`). Doc-Index: header/framing refreshed, the §2 DAG extended through RFC-0021, the
  §5 "next steps" replaced by the live phase ladder, and three stale rows corrected by the review leaf
  (**RFC-0008 Draft→Accepted**, Foundation **r3→r4**, an **ADR-014** row). Foundation: an append-only
  **r4** revision note + **Phases 4–8** added to the §6 roadmap + §10 refreshed. The stray empty tracked
  file `oom` is removed.
- **`doc-currency` gate (M-371).** A new skip-graceful `just check` check
  (`scripts/doc_currency.py` + `scripts/checks/doc-currency.sh`, wired into `scripts/checks/all.sh` +
  the `justfile`): asserts the README structure-tree ↔ filesystem, the Doc-Index ↔ `docs/` RFC/ADR/DN
  coverage, and cited crate-count currency (opt-in `<!-- doc-currency:crate-count -->` marker).
  Complements — does **not** duplicate — the existing `links` and `myc-doc` checks, so the navigational
  docs cannot silently drift again.
- **`fmt`→`io` canonical-JSON delegation (M-372, #203).** The converged *"one canonical JSON
  projection"* seam (fmt.md §7-Q1 / io.md §7-Q1 / `docs/spec/stdlib/README.md §5` / RFC-0016 §8-Q1) is
  **ratified and wired**: `std.fmt`'s `to_json`/`from_json` now **delegate** to
  `mycelium_std_io::{to_json, from_json}` instead of a duplicated `serde_json` codec + error
  classification — one JSON, two entry points, the round-trip established **once**, in `std.io`. The
  review leaf caught two real delegation bugs (the alphabetical-map-key error priority for an unknown
  `repr.kind`; `std.io`'s domain-heuristic vs `fmt`'s grammar classification for a missing field) —
  both fixed, with `fmt`'s public API and guarantee tags **unchanged** (`ToJsonError::NonFinite{index}`
  is still the typed non-finite refusal). The honest **tag-framing residual** — `std.io` tags
  `from_json` `Empirical` (proptest corpus), `std.fmt` tags it `Exact` (deterministic decode, no
  accuracy semantics) — is **FLAGGED for the maintainer, not silently changed** (VR-5). The specs read
  *"implemented (Rust-first), pending ratification."*

Verified: `mycelium-std-fmt` **32/32** + `mycelium-std-io` **70/70**; `clippy --all-targets` clean;
`doc-currency` / `links` / `markdown` green. Append-only; issue mapping recorded in
`tools/github/idmap.tsv` (#202/#203, real db-ids).

### Changed (2026-06-19: Tier-A completion fast-follows — cross-crate seam reconciliations)
The cross-module FLAGs the Tier-A wave left for the maintainer are discharged (each a real change —
code + tests + guarantee-matrix/baseline kept green):
- **A1 — `std.testing` `FailRecord` → `Diag` (testing↔diag seam, spec §7-Q2).** `FailRecord` now
  **delegates** to the canonical record via `FailRecord::to_diag()` → `mycelium_diag::Diag` (the
  description becomes the message; the op context + reproducing seed + trial ride along as EXPLAIN
  notes, G11). It keeps the testing-specific seed/trial reproduction metadata a generic `Diag` does
  not model.
- **A2 — `std.error` recover stub → real `Outcome` (error.md §7-Q1).** `std.error` drops its abstract
  stub `RecoverOutcome` enum + `recover` fn and **re-exports** the concrete
  `mycelium_std_recover::{Outcome, Resolution, RecoverOutcome, handle_classified}` — it is the bridge
  *target*, not the home of the recovery algebra (KC-3). Contract holds verbatim: `Recovered |
  Propagated`, no drop (I1), tag inherited from the policy (I2/VR-5).
- **A3 — `DECLARED_FLOAT_EPS` → `std.numerics` (math ε-ownership, NFR-N2 / math.md §7-Q2).** The
  `Declared` libm-floor ε is now homed in `std.numerics` (the ε-carrier module) and **re-exported**
  by `std.math` — stated in exactly one place. Its honest `Proven` upgrade stays the kernel's /
  M-541's to supply (VR-5).
- **A4 — `std.spore` `regrow` → `Approx` (spore.md §7-Q4).** `RegrowthResult` now carries the
  manifest's full certificate `Bound` and projects to `std.numerics::Approx<Factorization>` via
  `as_approx()` — strength **derived** from the bound's basis (`Approx::attach`, never upgraded —
  VR-5), held at the `Empirical` ceiling (FR-C2). Carries `Factorization` (the VSA decode result),
  not `Value` (that mapping is `std.vsa`'s).

**Decision — B2 (kernel f64 finiteness):** keep the **status quo** — `Value::new` stays permissive and
each projection/op refuses a non-finite `f64` explicitly (never-silent at the point of use), rather
than rejecting at construction. Revisit alongside a future binary codec. (A formal ADR can ratify
this; recorded here as the maintainer's decision.)

**B1 (`EffectBudget::Io`/`Named`) — enacted.** `mycelium-interp::budget::EffectBudget` gains
`Ops(u64)` (→ `EffectKind::Io`) and `Named(String, u64)` (→ `EffectKind::Named`) — one budget variant
per effect kind — so `std.recover`'s `cleanup_then_propagate` budgets an `Io`/`Named` cleanup effect
**directly**, removing the leaf's Retry/Attempts proxy. Additive kernel change (no new L0 node, KC-3);
`EffectBudget` is no longer `Copy` (the `Named` variant carries its name); property-tested both sides;
`mycelium-interp` API baseline refreshed.

Public-API baselines refreshed for the five touched crates; full `just check` green.

### Added (2026-06-18: Phase-5 Tier-A completion — Rust-first numerics/diag/recover/spore, M-510/512/520/522)
The Tier-A differentiator surface is completed: the four remaining spec'd-but-uncoded Tier-A Ring-1 `std`
modules land as Rust-first crates, built as a **swarm of four sonnet agents** fanned in with an **octopus
merge** (the orchestrator scaffolds first, owns every shared file, and reconciles after merge — the pattern
in `CLAUDE.md`). Each ships its **RFC-0016 §4.5 guarantee matrix as checked data** (asserted in tests) and
sits at the **honestly-supportable strength (VR-5)** — downgraded, never upgraded without a checked basis.
- **`mycelium-diag`** (new kernel crate) + **`mycelium-std-diag`** (M-510, #151) — the structured
  failure-legibility substrate. A **maintainer-resolved FLAG** (scaffold decision #1): the canonical
  RFC-0013 record types (`Diag`/`Severity`/`Locus`/`Trace`/`Code`) are **extracted into a small
  `mycelium-diag` kernel crate** — a deliberate, bounded trusted-base growth so the record has one owner
  below the std layer — and `mycelium-std-diag` re-exports + wraps it (KC-3). Dual human/JSON projection
  (G11, round-trip-checked), content-addressed presentation-invariant identity (ADR-003); all matrix rows
  `Exact`; `present` returns the error **unchanged** — the I1 structural proof (presentation never gates
  propagation).
- **`mycelium-std-numerics`** (M-512, #153) — the honest ε/δ carrier. `Approx<T>` is a thin
  `Meta`-attached `{Bound, strength}` view (no new numeric type, no kernel change, KC-3); `combine`/`map`
  take the **meet** of input strengths and propagate bounds with **outward (directed) rounding**; `Proven`
  is reachable **only** via a sealed `ProvenThm` witness (FR-N3, type-level; `compile_fail`-doctested);
  refuse-without-a-rule (`Err(NoRule)`, never a fabricated bound). ε constants cited from
  `mycelium-numerics`, restated none (NFR-N2).
- **`mycelium-std-recover`** (M-520, #156, **Rust-first half**) — the declarative recovery bridge. The
  `Outcome`/`Resolution` sums have **no `Dropped` variant** (I1); the closed v0 action set, a
  content-addressed `RecoveryPolicy`/`PolicyRef`, declared + budgeted effects via `mycelium_interp::budget`
  (graceful `EffectBudgetExhausted`), and the never-silent `handle_classified` driver carry a
  `mycelium_diag::Diag`. The recovered tag is honest (`Ok`→floor, `fallback`→`Declared`, `retry`→inherited;
  never laundered up — I2/VR-5), **fixing the P5-B exact-tag bug**. The self-hosting half stays Batch P5-C
  (M-502-gated).
- **`mycelium-std-spore`** (M-522, #163, library/manifest half) — the content-addressed deployable +
  reconstruction-manifest library over the `mycelium-spore` packager + `std.content` + `std.vsa` (KC-3 — no
  new hash, no new trusted code). Identity is the canonical content hash and metadata-invariant (ADR-003); a
  hash mismatch is an explicit `Err` (C1/G2); probabilistic regrowth is held structurally at the
  **`Empirical` ceiling** (FR-C2/VR-5, never `Proven`). Full native deploy is Phase-6-gated (M-620).

The four spec `Status` lines move to **"Implemented (Rust-first), pending ratification"** (append-only;
never silently `Accepted`). Fast-follow reconciliations are FLAGGED, not silently made: `std.testing`'s
placeholder `FailRecord` → `mycelium_diag::Diag`; `std.error`'s abstract `recover` stub → the concrete
`mycelium_std_recover::Outcome`; `EffectBudget::Io`/`Named` variants for `mycelium-interp`; the
`DECLARED_FLOAT_EPS` migration into `std.numerics`; the `regrow` → `Approx<Value>` wrapper. Full
`cargo build`/`clippy --all-targets -D warnings`/`test --workspace` green (1883 tests).

### Added (2026-06-18: Phase-5 Batch P5-B — Rust-first Ring-2 stdlib commons, M-511/514/524/525/526/527/528/529/531/532/533/534)
The second Phase-5 standard-library wave lands: twelve **Ring-2 / Tier-B** `std` crates — the general
library written **to the RFC-0016 §4.1 contract over Ring 0/1** — built as a **swarm of twelve sonnet
agents** fanned in with a single **octopus merge** (the orchestrator owns every shared file; the pattern
is recorded in `CLAUDE.md`). Each crate lives **above the kernel** (KC-3) and ships its **RFC-0016 §4.5
guarantee matrix encoded as checked data** (asserted in tests, never prose-only). Tags sit at the
**honestly-supportable strength (VR-5)** — downgraded, never upgraded without a checked basis.
- **`mycelium-std-error`** (M-527, #168) — `Option`/`Result`/error combinators; pure combinators `Exact`,
  the `unwrap_or` family `Declared` (the substituted default is asserted, not proven), the `recover`
  bridge inherits its policy's tag and never launders upward. The concrete `recover` surface is **owned by
  `std.recover` (M-520)** and stubbed abstractly here (I1 propagation floor).
- **`mycelium-std-cmp`** (M-532, #172) — ordering/equality + **non-representation** conversion; lossless
  widening (incl. the ratified `BF16→F32`) total/`Exact`, lossy narrowing **fallible** (`Result`, never a
  silent truncation/wrap). A *representation* change stays `std.swap`'s (no op double-owned).
- **`mycelium-std-iter`** (M-526, #167) — fold/iterator combinators; totality **inherited** from the
  kernel's RFC-0007 §4.8 total fold (KC-3, not re-proved); short-circuit ops are done-flag folds; the
  transducer fusion law is an `Empirical` property test, never `Proven`.
- **`mycelium-std-collections`** (M-511, #152) — value-semantic `Seq`/`Map`/`Set` with property-checked
  insertion-order invariants (**no silent reorder**); non-identity map hashing kept distinct from
  `std.content`'s canonical identity (ADR-003).
- **`mycelium-std-math`** (M-525, #166) — exact integer/rational/rounding ops `Exact`; **all 14
  transcendentals downgraded to `Declared`** (the libm compute floor is unaudited `wild`/FFI — VR-5); the
  ε magnitudes + upgrade path are owned by `std.numerics` (M-512) / the `std-sys` audit (M-541).
- **`mycelium-std-text`** (M-524, #165) — UTF-8 string ops; `parse`/`from_utf8` return `Result` (never a
  sentinel or silent U+FFFD), lossy transcodes are an explicit `Lossy<T>` value. All `Exact`; the honesty
  load is the fallibility column.
- **`mycelium-std-fmt`** (M-533, #173) — dual human/machine projection (G11); deterministic `Exact`
  projections + an `Exact`-fallible `from_json` with a checked round-trip. The canonical JSON is **owned by
  `std.io`/serialize (M-514)**; `fmt.to_json` delegation is FLAGGED for wiring (no duplicate JSON created).
- **`mycelium-std-testing`** (M-534, #174) — property/golden/differential harness; a `Skipped`/
  `Undetermined` verdict is **reported, never a silent pass**, a `Fail` is a structured record (the
  `std.diag`/M-510 substrate, stubbed structurally), and the mechanism never inflates a subject's tag.
- **`mycelium-std-io`** (M-514, #155) — IO + serialization; affine `Source`/`Sink` enforce
  single-consumption (LR-8); `serialize`/`to_json` `Exact`, `deserialize`/`from_json` `Empirical`
  (proptest round-trip, no theorem). Owns the canonical JSON; the OS-I/O floor is deferred to `std-sys`
  (M-541), so the surface ships over a fully-testable in-memory substrate.
- **`mycelium-std-fs`** (M-528, #169) — filesystem over a substrate; every path/permission failure is an
  explicit `Result` (I1), effects declared (C6), all `Exact`. The real syscall floor is deferred to
  `std-sys` (M-541); ships over a testable `InMemoryFs`.
- **`mycelium-std-time`** (M-529, #170) — monotonic vs wall a **typed** distinction; pure duration
  arithmetic `Exact` with overflow→`Err` (never wrap); clock reads carry a **declared** effect (RT3) at
  the type level. The OS clock floor is deferred to `std-sys` (M-541).
- **`mycelium-std-rand`** (M-531, #171) — seeded PRNG (xoshiro256++) reproducible + `Exact`; samplers
  `Declared`/`Empirical`, **never `Proven`**; platform entropy is a **declared** effect (RT3) over an
  injectable source, the OS entropy floor deferred to `std-sys` (M-541).

The 12 per-module specs under `docs/spec/stdlib/` move **Draft → "implemented (Rust-first), pending
ratification"** (*not* `Accepted`). Cross-module FLAGs raised by the swarm — the `std-sys` `wild`/FFI floor
(M-541; `math`/`io`/`fs`/`time`/`rand`), the `recover` bridge (M-520; `error`/`testing`), the `std.diag`
record substrate (M-510; `testing`), the `fmt`→`io` canonical-JSON delegation, and the early-termination
fold primitive (RFC-0007 §4.8; `iter`) — are recorded for the maintainer's ratification pass, not silently
decided. 722 tests across the 12 crates; workspace `fmt`/`clippy -D warnings`/`test` green. Ratification
and the M-502-gated Mycelium-lang migration half remain downstream.

### Added (2026-06-18: Phase-5 Batch P5-A — Rust-first stdlib enactment, M-513/515/516/517/518/519/523)
The first Phase-5 standard-library code lands: seven `std` capability crates, built as a
**swarm of six sonnet agents** fanned in with a single **octopus merge** (the development pattern is
now recorded in `CLAUDE.md`). Each crate lives **above the kernel** (KC-3) as a consumer of one
landed capability crate, and each ships its **RFC-0016 §4.5 guarantee matrix encoded as checked
data** (asserted in tests, never prose-only).
- **`mycelium-std-core`** (M-515, Ring 0) — the prelude: re-exports the `mycelium-core` value model
  (`Value`/`Repr`/`Meta`, `CoreValue`/`Datum`, the `GuaranteeStrength` lattice, `Bound`/`BoundBasis`,
  `ContentHash`) + a thin §4.8 query surface (`repr_of`/`meta_of`/`guarantee_of`/`bound_of`/
  `provenance_of`) that reports absence with `Option`, never silently. All 9 matrix rows `Exact`/total.
- **`mycelium-std-ternary`** (M-517, Ring 1) — exact balanced-ternary arithmetic + `Bit`/`Trit` +
  the inspectable I2_S/TL1/TL2 packing codecs; 18 ops `Exact`, range/overflow is fallibility not a
  weakened tag.
- **`mycelium-std-swap`** (M-516, Ring 1) — certified representation change over `mycelium-cert`; a
  swap is never silent and returns its certificate; bin↔tern `Exact`-within-range, dense↔vsa
  `Empirical` (δ), `f32→bf16` `Proven` (inherited `ProvenThm`).
- **`mycelium-std-dense`** (M-518, Ring 1) — typed dim-tracked tensors over `mycelium-dense`;
  elementwise `Proven` tags **inherited** from the kernel's checked Higham basis, accumulation ops
  conservatively **downgraded to `Empirical`** pending M-512 (VR-5).
- **`mycelium-std-select`** (M-519, Ring 1) — selection DSL where **every selection emits a valid,
  inspectable EXPLAIN record** (one mechanism; SC-3/G2); deterministic, overrides tested.
- **`mycelium-std-vsa`** (M-513, Ring 1) — hypervector surface over `mycelium-vsa` with **per-model
  honest tags** (MAP-I bind/unbind `Exact`; HRR/FHRR unbind/bundle `Empirical`; resonator
  `reconstruct_factors` never `Proven`); approximate unbind returns `(item, confidence)` via cleanup.
- **`mycelium-std-content`** (M-523, Ring 1) — content-addressing / hash primitives over
  `mycelium-core`; identical content collides, metadata is not identity, malformed refs are explicit
  errors.
- *Verification:* workspace `cargo fmt`/`clippy -D warnings -A unsafe_code` (the CI gate's exemption)/
  `test` all green — 230 stdlib tests added. *Honest scope (VR-5):* these are the **Rust-first**
  implementations (RFC-0016 §4.6); the per-module specs move to **"implemented (Rust-first), pending
  ratification"** — not `Accepted`. The Mycelium-lang migration half (gated on M-502) and the
  per-module FLAGs each crate carries (lexicon/DN-02·06, error-variant gaps, M-512/M-540 follow-ons)
  stay open. Advances **FR/NFR** via RFC-0016's contract; **G2/VR-5/SC-3/KC-3** are enforced per op.

### Decided (2026-06-18: RFC-0021 framework ratified; LLM validation split into an isolated non-blocking track)
The fourth Track-B RFC is ratified at framework scope, with the empirical leverage gate isolated.
- **RFC-0021 → Accepted (framework).** Normative: the projection model (§3), invariants **P1–P6**
  (§4.3), the `Projection` interface + registry (§4.1/§4.4), the dual human/machine architecture, the
  `LlmCanonical` **design** (§4.6), and the §4.7 **supersession mechanism**. The **G11
  ergonomics/feasibility** gate is discharged — *demonstrated* in code (`mycelium-lsp::project`).
- **The LLM-leverage validation is split out** into an isolated, **non-blocking** track — **RP-1 /
  new task M-381** (the T3.6 five-arm retention-ratio ablation; turnkey protocol in `research/11`
  T11.7). It **gates nothing**; its only coupling to the accepted framework is a future
  **supersession** (RFC-0021 §4.7) — if the ablation falsifies (retention < ~70%), a new RFC promotes
  `LlmCanonical` to primary. *Honesty (VR-5):* the ratification asserts **no** leverage result; the
  claim stays `Declared`/open until the run. This restructures (supersedes, for prompt 2) the §9
  "both prompts gate ratification" framing, append-only.
- **Ripples:** RFC-0021 §9 ratification note + §4.7 track note + Meta-changelog appended; RP-1 →
  "Open — isolated non-blocking track (M-381)"; `issues.yaml` mints **M-381**; Doc-Index status →
  Accepted (framework). With this, all four Track-B RFCs (0018/0019/0020/0021) are Accepted at their
  honestly-supportable scope; the only outstanding wave item is the **non-blocking** M-381 run.

### Added (2026-06-18: LlmCanonical projection prototype — RFC-0021 ergonomics gate demonstrated, M-380)
- **`crates/mycelium-lsp/src/project.rs`** — a v0 `LlmCanonical` projection (RFC-0021 §4.6; FR-S5): an
  s-expression renderer over the Core IR, **total over all 11 L1 node kinds** (compiler-enforced
  exhaustive `match`) and **deterministic**, living above the kernel (KC-3) in the dual-intelligibility
  surface crate. It **preserves the honesty overlay by construction** — **P3** (`Swap` rendered
  explicitly, never elided) and **P2** (every `Const`'s guarantee tag rendered; bound presence
  surfaced) — each checked by a unit test (4 tests, all green).
- This **demonstrates** RFC-0021's §9 ergonomics/feasibility gate (RP-4 sub-q 1) in code, upgrading
  `research/11` T11.4 from a grounded assessment to demonstrated evidence for L1. RFC-0021 §9
  addendum + `research/11` changelog appended.
- *Honest scope (VR-5):* the rules are Rust source, not the declared-rule-table form (RFC-0021 §4.2);
  the projection is read-only (no `RoundTrip` parse-back); the *measured authoring cost as L2 grows*,
  the *human-usability* study, and — wholly untouched — the **LLM-leverage** gate remain open. **No
  leverage result is asserted.** (At this step RFC-0021 was still Draft; its framework was ratified
  later in this same wave — see the Decided entry above — and only the non-blocking leverage run,
  M-381, stays open.)

### Decided (2026-06-18: RFC-0018 / RFC-0019 / RFC-0020 ratified — Track B RFCs → Accepted)
With the gating research discharged (RP-2/RP-3) and readiness assessed (DN-12), the maintainer ratified
the three non-empirical Track-B RFCs append-only. (RFC-0021 was ratified separately, later in this same
wave, at framework scope — see the Decided entry above; only its empirical leverage gate — non-blocking
M-381 — stays open.)
- **RFC-0018 → Accepted.** R18-Q1 = **Design A** (data-lineage / data-provenance integrity; `G-Match/A`
  normative — the guarantee system tracks data provenance, not control-flow secrecy); R18-Q4 =
  certificate reference at the type level, validity at elaboration/runtime (KC-3). R18-Q2/R7-Q2 closed.
  **Supersedes RFC-0007 §4.3**'s stage-1 deferral and **discharges RFC-0006 §8 Q3**. *Honesty (VR-5):*
  the noninterference result stays **Declared-with-argument** (not machine-checked) — acceptance is of
  the design, and does not upgrade that tag.
- **RFC-0019 → Accepted.** Coherence = **orphan rule + global uniqueness + reject-overlap** (hash-stable
  resolution); the **Repr-polymorphism restriction set** (§4.6 — "no paradigm-specific `Op` on a
  Repr-abstract argument", locally checkable, S1-preserving) is normative; **newtype waivers rejected
  in v1** (need roles); **multi-param + associated types deferred to v2**; dictionary-passing, kernel
  budget unchanged (KC-3). Soundness stays **Declared-with-argument**.
- **RFC-0020 → Accepted (scoped).** The §4.1/§4.3/§4.4/§4.6/§4.7/§4.8/§4.9 core is Accepted (no research
  gate); §4.2/§4.5/R20-Q1…Q5 carved out as deferred (the RFC-0006 r5 pattern), each unblocking via the
  now-Accepted RFC-0018/0019 and the enacted RFC-0001 r5 `FixGroup`.
- **Ripples:** RFC-0006 §8 Q3 discharged; RFC-0007 §4.3 stage-1 deferral superseded + R7-Q2 resolved
  (append-only changelog notes on both); RP-2 and RP-3 → **Resolved**; Doc-Index statuses → Accepted.

### Added (2026-06-18: RFC-0020 readiness + RFC-0021 design grounding — Track B completes its analyzable scope)
Finishes the next wave's Track B for the two remaining RFCs, honestly bounded by what analysis can settle.
- **`docs/notes/DN-12-RFC-0020-Ratification-Readiness.md`** — RFC-0020 (L2 surface) carries **no
  research gate**; its deferred items depend on sibling RFCs, two of which had their research gates
  discharged this pass (R20-Q1 → RFC-0019/RP-3; R20-Q2 → RFC-0018/RP-2) and one on the enacted
  RFC-0001 r5 `FixGroup` (R20-Q4). Recommends a **scoped** ratification (the §4.1/§4.3/§4.4/§4.6/§4.7/
  §4.8/§4.9 core) with a carve-out — the RFC-0006 r5 precedent. RFC-0020 Meta-changelog appended.
- **`research/11-semantic-projection-framework-RECORD.md`** — grounds RFC-0021's *non-empirical*
  design (T11.1–T11.5): the projection model + P1–P6 as established Unison/MPS prior art with a
  locally-checkable honesty overlay; **one** dual-rendering architecture (RP-4 sub-q 3 answered);
  authoring assessed **feasible** at single-engineer scale (grounded in the existing
  `mycelium-lsp::feedback` node-walk); the Unison human-usability posture (RP-4 sub-q 1). **States
  plainly that the LLM-leverage gate (the T3.6 retention-ratio ablation, RP-1; RP-4 sub-q 2) is
  irreducibly empirical and is NOT discharged** (T11.6) — supplies a turnkey five-arm protocol over the
  existing `experiments/` harness (T11.7), **a protocol, not a result**. Per VR-5 / DN-09 §4 no leverage
  is asserted. RFC-0021 §9 marked **partially advanced**; Meta-changelog appended.
- **RP-1 → Open (irreducibly empirical; protocol ready)** and **RP-4 → partially addressed** in
  `research-prompts.md`; Doc-Index gains DN-12 and updates the RFC-0020/0021 rows. At this step RFC-0020
  and RFC-0021 were **both Draft** (RFC-0020 awaiting the maintainer's scoped-ratification decision;
  RFC-0021's framework ratification-ready behind the carve-out); **both were ratified later in this same
  wave** — RFC-0020 → Accepted (scoped), RFC-0021 → Accepted (framework); see the Decided entries above.
  Only the non-blocking empirical run (M-381) remains.

### Added (2026-06-18: RP-3 discharged — traits coherence + Repr-polymorphism soundness, RFC-0019)
Continues Track B with the trait layer's two flagged-novel research prompts.
- **`research/10-traits-coherence-repr-polymorphism-RECORD.md`** — discharges RP-3 / RFC-0019 §9
  R1+R2. Findings T10.1–T10.9: why content-addressing makes coherence a *correctness* property
  (T10.1); the coherence mechanism — Rust-style orphan rule + global uniqueness + reject-overlap, the
  only one consistent with ADR-003 (T10.2) — with a **total/deterministic/hash-stable resolution
  theorem + sketch** (T10.3); the Q-coherence verdict — reject newtype waivers in v1 (safe admission
  needs a *roles* mechanism, Weirich et al.) (T10.4); the **Repr-polymorphism restriction set** ("no
  paradigm-specific `Op` on a Repr-abstract argument; passthrough / trait-interface / lexical-`Swap`
  only"), shown **locally checkable** (T10.5) and **S1-preserving** with a theorem + sketch (T10.6),
  grounded as the dual of GHC levity polymorphism (T10.7); recommend-defer for multi-param/associated
  types (T10.8); and confirmation that dictionary-passing keeps the kernel node budget unchanged
  (T10.9).
- *Honest scope (VR-5):* both soundness results are tagged **Declared-with-argument** (theorems with
  sketches), **not** `Proven` — mechanization is the future `Proven` basis. The design decisions are
  recommended, not ratified.
- **RFC-0019 §9 R1/R2 marked DISCHARGED**, Meta-changelog appended; **RP-3 → research-discharged** in
  `research-prompts.md`; Doc-Index updated. **RFC-0019 was Draft at this step** — the research gate
  closed here; ratification followed later in this same wave (RFC-0019 → Accepted; see the Decided
  entry above). No normative rule changed.

### Added (2026-06-18: RP-2 discharged — stage-1 grading noninterference, RFC-0018)
Begins the next wave's Track B (RFC ratification research) with the flagged-novel grading RFC.
- **`research/09-stage-1-grading-noninterference-RECORD.md`** — discharges RP-2 / RFC-0018 §11 (the
  pre-ratification noninterference obligation). Findings T9.1–T9.8: the lattice as a Biba integrity
  lattice with certified-`Swap` endorsement (T9.1); the two candidate properties and which one VR-5
  demands — *data-provenance*, not full IFC secrecy (T9.2); the Design-A/B distinguishing
  counterexample, the `pick` program (T9.3); the **data-provenance noninterference theorem + proof
  sketch**, reduced to the existing `meet`/`propagate` implementation and the RFC-0002 certificate
  checker so the novelty is isolated to `G-Swap` (T9.4–T9.5); the purity precondition bounding
  Design A's sufficiency (T9.6); R7-Q2 closure (T9.7); and the R18-Q4 recommendation (T9.8).
- *Honest scope (VR-5):* the soundness result is tagged **Declared-with-argument** (a stated theorem
  with a proof sketch), **not** `Proven` — mechanization (Lean/LiquidHaskell) is the basis for a
  future `Proven` upgrade, named not claimed. The empirical/decision items it cannot settle (the
  R18-Q1 maintainer choice) are recommended (Design A), not ratified.
- **RFC-0018 §11 marked DISCHARGED**, Meta-changelog appended; **RP-2 status → research-discharged**
  in `research-prompts.md`; Doc-Index RFC-0018 row updated. **RFC-0018 was Draft at this step** — of
  its three §10 ratification gates, the research gate (2) closed here; gates 1 (R18-Q1) and 3 (R18-Q4)
  were decided later in this same wave (RFC-0018 → Accepted; see the Decided entry above). No normative
  rule changed.

### Added (2026-06-18: next-wave plan + R7-Q4 enactment — content-addressed prim table)
The post-#194 wave is sorted into a plan, and its leading item is begun.
- **DN-11 — Next-Wave Plan (Draft / Resolved-as-capture).** Indexes the KC-2-unblocked work into
  three dependency-ordered tracks with gates — A: DN-10 L1 completion; B: RFC-0018…0021 ratification
  (each held Draft until its RP-1…RP-4 spike); C: Phase-5 stdlib (gated on M-501/M-502) — names the
  leading item (Track A → R7-Q4), and records the stdlib spec-vs-code status nuance honestly.
- **`tools/github/issues.yaml`:** mints **M-390** (R7-Q4 prim declarations) and **M-391** (R7-Q3
  surface mutual recursion) under Phase 4, with `depends_on` per DN-10 §2.5/§3.5.
- **M-390 (R7-Q4) — prim table `Π` as content-addressed declarations.** New
  `mycelium-core::prim` (`PrimTable`/`PrimDecl`/`PrimSig`/`PrimRef`) mirroring the data registry `Σ`
  (RFC-0001 §4.3 r3; ADR-003): each kernel prim is keyed by the content hash of its *signature +
  intrinsic guarantee* (`g_f`, RFC-0001 §4.7), name kept as metadata. The LSP feedback facade gains a
  sixth artifact kind — a **prim declaration surfaced at every `Op` site** (EXPLAIN over prims, DN-10
  §3.2 step 4; an unrecognized prim is surfaced, never silent). Drift-guards pin `Π_new == Π_old`
  (DN-10 §3.4) against the L1 surface table and the interpreter's intrinsic.
  - *Honest scope (VR-5):* every v0 prim is `intrinsic = Exact` (stored as data). Deferred, flagged:
    `Node::Op` carrying a `PrimRef` content hash in the term, and the **RP-7** BoundBasis-with-citation
    schema for non-`Exact` prims — neither is faked here.
  - *Verified:* `cargo test --workspace` + `clippy -D warnings` green; the **M-210** three-way
    differential (L1-eval ≡ elaborate→L0-interp ≡ AOT) preserved; new unit/equivalence tests in
    `mycelium-core`, `mycelium-l1`, `mycelium-interp`, `mycelium-lsp`. Advances no `FR/NFR/VR/SC`
    upgrade — a uniformity/inspectability gain (G2/SC-3), KC-3 (no L0 node-grammar change).

### Added (2026-06-18: KC-2-unblocked surface/type-system designs — Wave 2 of the maturation pass)
With the KC-2 gate cleared (DN-09), the deferred L-layer designs are drafted in dependency order.
The deep-novel ones land as **Draft** with a pre-ratification **research prompt** (VR-5 — only the
KC-2 verdict, RFC-0017, and the L3 commit are ratified this pass; these are grounded direction):
- **RFC-0018 — Stage-1 Static Guarantee Grading (Draft).** The graded judgment RFC-0006 Q3 /
  RFC-0007 §4.3 deferred: the guarantee lattice as a graded coeffect modality, `Swap`+certificate the
  sole endorsement point. Surfaces the **implicit-flows decision** (R18-Q1) as a required maintainer
  choice + a noninterference proof obligation (research prompt RP-2). Flagged-novel.
- **RFC-0019 — Traits & Parametric Polymorphism / LR-2 (Draft).** Dictionary-passing elaboration to
  existing L1 nodes (kernel budget unchanged), coherence under content-addressed identity;
  Repr-polymorphism (LR-5) + guarantee-indexed methods (LR-6) flagged-novel (RP-3).
- **RFC-0020 — The L2 Surface Term Language (Draft).** The programmer-facing surface as
  elaboration-defined (no independent semantics): inference, content-addressed modules, pattern sugar
  → Maranget-flat L1 `Match`, derived forms; usability-first (DN-09 §3.2).
- **RFC-0021 — Semantic-Level Projection Framework (Draft).** M-380/FR-C1/G11: projections as
  inspectable views over content-addressed defs; the LLM-facing canonical projection (FR-S5) as the
  lever to lift surface leverage. Gated on the G11-ergonomics + T3.6 prompts (RP-1, RP-4).
- **DN-10 — Remaining L1 Gaps.** Planning capture of R7-Q3 (mutual-recursion surface elaboration) and
  R7-Q4 (prim table → content-addressed declarations), each purely additive, with spike prompts.
- **research-prompts.md — Standing Research Prompts.** Consolidated index RP-1…RP-7 for variant passes.
- **Ripple (corpus consistency):** the KC-2 status flips (phase-3 §5 verdict row, M-002 → done,
  E3-1/M-380 → design-active; Foundation §6 P0.2; SPECIFICATION §10.2; self-hosting-readiness
  capability #3 → ready) and the maturation cross-ref notes (RFC-0004 §4, RFC-0008, RFC-0014 §4.7).
- **Indices:** Doc-Index + RFCs README updated for RFC-0017…0021, DN-10, research-prompts.

### Decided (2026-06-18: KC-2 verdict + maturation-scope ratification — language-maturation pass)
- **DN-09 — the KC-2 verdict = proceed.** The maintainer recorded a verdict on the M-002 LLM-leverage
  run (local Qwen2.5-Coder, 10-task gold set, seed 42): measured leverage is *weak-but-recoverable*
  (best arm 7B+examples: 40% first-attempt → **70% eventual**, **+30pp edit-to-fix** via the G10
  semantic-feedback loop), so KC-2's "irrecoverable collapse even with feedback" kill criterion
  (Foundation §2.4) is **not triggered**. This **closes the standing KC-2 gate** the corpus parked on.
  Honest scope (VR-5): single seed / 10 tasks / local ≤7B; the rigorous T3.6 retention-ratio ablation
  was **not** run and stays a tracked research follow-up.
- **RFC-0006 → r5: concrete L3 surface committed.** The verdict discharges §8 Q1 (the one deliberate
  deferral): the L3 strategy is **committed text syntax** (the v0 grammar `docs/spec/grammar/mycelium.ebnf`
  becomes the real surface, refined append-only) **+ a co-equal structured-projection layer** (M-380,
  FR-S5). Q6 literal spelling is now committable. Q3 (stage-1 grading) and the Q8 `unsafe` *spelling*
  stay open on their own merits (never KC-2-gated). The embedded-DSL fallback (RR-3) is retained unspent.
- **RFC-0017 (new, Accepted) — Maturation Scope & De-maturation; ratifies DN-08 (→ Resolved).** Lifts
  `matured` from **per-definition** to **scope** granularity: a `nodule`/`phylum` is matured via its
  **header** (`// @matured: true`), a program/package via its **`mycelium-proj.toml` manifest**;
  **`matured fn` is retired** (per-definition maturation no longer expressible — maintainer decision).
  Reserves **`thaw`** (Surface, conventional-clearest — `germinate` taken by ADR-013; DN-02 three-test gate in §5) as the in-source
  **de-maturation** marker (`thaw fn …` keeps one definition interpreted inside a matured scope —
  never-silent, `EXPLAIN`-able, weakens no advertised honesty tag). **Supersedes RFC-0007 §4.5
  *granularity*** append-only — the `matured ⟹ total` gate + totality classifier are **unchanged**,
  §4.2 merely quantifies them over the matured scope. Reifies a per-scope maturation record (the M-311
  certificate's roll-up, §4.4).

### Changed (2026-06-18: corpus ripple for the KC-2 verdict + RFC-0017)
- **Grammar (`mycelium.ebnf`):** `fn_item` drops the `matured?` prefix and gains an optional
  `thaw` prefix; `matured` is reframed as a header/manifest key (reserved word, no term
  production); the surface is marked the **committed L3 text surface** (DN-09). **Conformance corpus:**
  `accept/08-matured-fn.myc` → `accept/08-maturation-and-thaw.myc` (header maturation + a
  `thaw fn`); new `reject/11-matured-fn-retired.myc` (the old `matured fn` form now rejects).
- **RFC-0007 §4.5 + changelog**, **Glossary §2.10 + new §2.10.1 `thaw`**, **grammar README**,
  **DN-03 changelog** (`thaw` reservation pointer), **Example-Programs-Reference #8**, and
  **Doc-Index** updated for the scope-level maturation + the new docs. (RFC-0004/0008/0014 cross-refs,
  the Nodule-Header `@matured` key, and the planning/Foundation/SPEC/self-hosting KC-2-status flips
  land alongside.)

### Changed (2026-06-18: PR #193 reproducibility + supply-chain hardening)
- **llama.cpp traceability:** the container build records the exact llama.cpp commit that produced
  the binaries to `/opt/llama-cpp.commit` (and an image `LABEL`), so results trace to precise code
  even though `LLAMA_CPP_REF` stays `master` (overridable). Addresses the "moving ref" review note
  without risking the working Blackwell build.
- **Pinned, integrity-checked tooling:** the image pins the Rust toolchain (`RUST_TOOLCHAIN=1.92.0`,
  the MSRV — rustup verifies each component's SHA-256) and installs **uv** at a pinned version from
  **PyPI/TLS** (`UV_VERSION=0.11.21`) via pip instead of `curl|sh` of a moving installer. Both are
  `--build-arg`-overridable.
- **Termux bootstrap:** the Claude Code install now downloads the installer to a file and runs it
  (inspectable/loggable) rather than a blind `curl … | bash`; the upstream installer is a moving
  target with no published checksum to pin, so this is the best available hardening.

### Fixed (2026-06-18: matrix prefetch ran the whole harness suite, OOM-skipped models)
- The KC-2 matrix prefetched models with `harness.py --ensure-model`, which fetches **then runs the
  full LLM-validation suite** (V-01…). On the desktop that suite got OOM-`Killed`, so the prefetch
  exited non-zero and the matrix wrongly **skipped that model's combos** (even though the `.gguf` had
  downloaded). Added `harness.py --ensure-only` (fetch the model, exit 0, do NOT run the suite) and
  switched `run-kc2-matrix.sh` to it. Verified: the new flag short-circuits before the suite.

### Fixed (2026-06-18: container ran `bash <binary>` instead of the command)
- `experiments/docker/Dockerfile` uses `CMD ["bash"]` instead of `ENTRYPOINT ["bash"]`. The image
  built fine, but `ENTRYPOINT ["bash"]` made `podman run IMAGE nvidia-smi` → `bash nvidia-smi` (and
  `… bash run-kc2-matrix.sh` → `bash bash …`), i.e. bash interpreting a binary as a script →
  "cannot execute binary file". The GPU "not visible" warning was a false alarm and the matrix never
  ran. `CMD` keeps the bare-`run` shell default while `run IMAGE <cmd>` now execs `<cmd>` directly
  (also unbreaks the README's `compose run kc2 uv run …` examples). Last-instruction change → rebuild
  reuses the CUDA compile layer (no recompile).

### Added (2026-06-18: build checkpointing — fast link preflight + ccache)
- `experiments/docker/Dockerfile` now **verifies the executable↔shared-lib link in seconds** (its own
  cached layer) *before* the ~10-min CUDA compile: a tiny exe links against a `.so` with undefined
  symbols (the exact shape of the llama.cpp link), so a broken `--allow-shlib-undefined` fails fast
  instead of after the whole build. (Mechanism validated locally: fails without the flag, passes with.)
- **ccache cache mount** (`RUN --mount=type=cache,target=/ccache`) + compiler launchers persist
  compiled objects across builds — a rebuild (even after editing the build recipe, which invalidates
  the layer) reuses the CUDA compile and only re-links, instead of recompiling from scratch. The cache
  survives a failed `RUN`, so a link error never costs the compile twice.
- `experiments/docker/run.sh` passes `--layers` to `podman build` (explicit intermediate-layer
  caching; Docker caches by default), since Podman was observed not reusing the cache.

### Fixed (2026-06-18: CUDA executable link in the container build)
- `experiments/docker/Dockerfile` passes `-DCMAKE_EXE_LINKER_FLAGS=-Wl,--allow-shlib-undefined` to the
  llama.cpp CUDA build. `libcuda.so.1` (the CUDA *driver*) is absent at build time and host-injected at
  runtime (CDI), so the `llama-cli`/`llama-server` link failed on undefined `cu*` driver symbols
  (`cuMemCreate`, …) after all 437 objects compiled. This matches upstream llama.cpp's own
  `.devops/cuda.Dockerfile`. Live-found on an RTX 5080 WSL2 box.

### Fixed (2026-06-18: Podman container build on WSL2)
- `experiments/docker/Dockerfile` now uses the **fully-qualified** base image
  `docker.io/nvidia/cuda:12.8.1-devel-ubuntu24.04`. Podman refuses unqualified short-names (Docker
  auto-prepends `docker.io/`), which broke `run.sh` at `FROM` on an RTX 5080 WSL2 box; one line
  unblocks both engines. Live-validated on that box: `gpu-setup.sh` (toolkit + CDI WSL auto-detect +
  in-container `nvidia-smi`) succeeds. Added an **Ubuntu WSL quickstart** to `docker/README.md` (and
  a note that the WSL `libnvidia-sandboxutils.so.1` warning is benign).

### Added (2026-06-18: Podman/WSL2 GPU path + 1.5B mobile cap)
- `experiments/docker/run.sh` is now **engine-agnostic and compose-free** — prefers **Podman**
  (rootless; outputs land owned by the user), falls back to Docker, with the correct per-engine GPU
  wiring (Podman CDI `--device nvidia.com/gpu=all --security-opt=label=disable`; Docker `--gpus all`).
- `experiments/docker/gpu-setup.sh` — one-time WSL2/Linux GPU preflight: verifies the host GPU,
  ensures the NVIDIA Container Toolkit, configures access (CDI generate for Podman / runtime configure
  for Docker), and verifies a container can see the GPU. Commands **vetted against NVIDIA's
  container-toolkit + CUDA-on-WSL docs** (cited in `docker/README.md` *Sources*).
- **1.5B mobile cap**: `run-kc2-matrix.sh` skips models larger than 1.5B unless `KC2_ALLOW_LARGE=1`
  (the desktop `run.sh` sets it). Phones stay at 0.5B/1.5B; desktop adds 7B+.
- `docker/README.md` rewritten for the Podman-first / WSL2 reality; the Docker Compose file is kept
  as a Docker-only convenience (its `gpus:` key is Docker-specific). Still best-effort/unverified on
  GPU here (no GPU/engine in the sandbox); scripts syntax-checked, cap logic unit-tested.

### Added (2026-06-18: one-command containerized GPU run)
- `experiments/docker/run.sh` — single fire-and-forget command (run from the repo root) that builds
  the CUDA image, verifies GPU visibility, and runs the full model × primer matrix ({0.5B, 1.5B, 7B}
  × {minimal, examples}) with `--serve` auto-offloading to the GPU. Outputs land on the host under
  `experiments/results/<model>-<primer>/`, ready to commit. Models/seeds/budget overridable by env.
  Warns + falls back to CPU if no GPU is visible. **Best-effort / unverified on GPU** in the
  design-phase sandbox (no GPU/Docker here) — syntax-checked; the GPU build + offload are verified by
  the operator via the built-in `nvidia-smi` step. README updated to feature the one-command path.

### Added (2026-06-18: DN-08 — maturation granularity capture)
- New design note **DN-08** (`docs/notes/`, Draft) capturing the maintainer intent that `matured`
  apply at module (`nodule`) / library (`phylum`) / program scope — coarse-grained, at a stable point
  — with per-`fn` maturation *atypical*, and selective *de*-maturation (shifting one subcomponent back
  to interpreted) the rare fine-grained operation. Advisory; RFC-0007's Accepted per-definition gate
  is untouched (append-only). Registered in `Doc-Index.md`; grounded in RFC-0007 §4.5, Glossary §2.10,
  DN-06. Also notes the harness safety property that only myc-check-validated Mycelium (never Rust, the
  implementation language) is ever fed to the model or to `myc-check`.

### Added (2026-06-18: KC-2 grounded primer, A/B variants, per-task token budgets, run matrix)
- **Grounded, leak-free Mycelium primer.** Rewrote `PRIMER_MYCELIUM` from the actual L1 grammar
  (lexer keywords, literal forms, exhaustive `match`, `for`-folds over recursive types, `let`,
  `swap`'s mandatory `to:`/`policy:`). Every embedded example is validated against `myc-check`.
  **Fixed an answer leak**: the old primer contained kc2-04's body (`add(<00+->, <0+0->)`) and
  kc2-07's `Sign`/`match` verbatim — it now uses only non-answer values, verified by a leak check.
- **Primer A/B variants** in `experiments/primers/`: `mycelium-minimal.txt` (syntax only) and
  `mycelium-examples.txt` (+ two complete, valid, *non-answer* worked programs to anchor a weak
  model on a language it was never trained on). Select with `--primer-mycelium FILE`.
- **Per-task token budgets**: `Task.max_new_tokens` sizes each generation to the task's complexity
  (96–144 tokens here) instead of a flat cap — faster on a phone CPU without truncating. `--n-predict`
  now defaults to *auto* (per-task) and, when given, is a hard override. Backends take an optional
  per-call budget (no effect on the tested `StaticGenerator` path).
- **`run-kc2-matrix.sh`**: runs {0.5B, 1.5B} × {minimal, examples} in sequence, unattended, robustly
  prefetching each model and writing `results/<model>-<primer>/` per combo.

### Fixed (2026-06-18: KC-2 extraction + primer defects surfaced by the first 0.5B run)
- **`extract_source` leaked the fence info string.** A model that fenced its code as ` ```source `
  left the literal word `source` as line 1, breaking *every* parse at 1:1 ("found Ident(\"source\")").
  Now the fence info string (any lone single-word tag, or a bare fence) is always dropped — a real
  program's first line is multi-token (`nodule <name>`, `fn …`). Verified against the on-device output.
- **The Mycelium primer showed `#` comments** — which Mycelium does not have (`unexpected character
  '#'`) — so a weak model parroted them into invalid programs. Also a *fairness* bug: `#` is valid in
  the Python baseline arm, so it penalised only the Mycelium arm. The primer now states there are no
  comments and emphasises the required `nodule <name>` header as prose, not an inline `#` annotation.
- Context: in the first complete 0.5B run (10/10 first-attempt invalid, sc5b=0.0) the failures were
  largely mechanical — e.g. kc2-04's model body was byte-identical to the reference and passes once
  the dropped `nodule bench` header is restored. These two fixes + edit-to-fix iterations (max-iters
  ≥ 2) should recover much of that; honest measurement still pending a re-run.

### Added (2026-06-18: containerised desktop-GPU runner + cross-platform fixes)
- **`experiments/docker/`** — a CUDA image (Dockerfile + docker-compose) that runs the whole KC-2
  pipeline on a desktop GPU (e.g. RTX 5080 / Blackwell `sm_120`) without touching the host
  toolchain: Python+uv, Rust (`myc-check`), and a CUDA-built `llama-server` in one image. The repo
  is bind-mounted so **all reports/logs/JSONL land on the host** for git; a named volume persists the
  model cache. Same `--serve` command; GPU auto-detected/offloaded. CPU fallback via
  `--build-arg LLAMA_CUDA=OFF`. Best-effort/untested in the GPU-less design sandbox (compose config
  validated; verify on host with `nvidia-smi`).
- **Model prefetch is now robust** — one `--ensure-model` invocation auto-resumes a dropped/slow
  download with capped backoff and **stall detection** (keeps resuming via HTTP Range as long as
  bytes arrive; gives up only after several no-progress attempts, always keeping the `.part`). New
  `--download-retries N` (default 8; **`0` = keep retrying until complete**) — ideal to prefetch a
  model ahead of time on a flaky phone link. Verified end-to-end against a local HTTP server (fresh,
  resume, and unlimited modes).
- **Windows correctness**: the Mycelium checker finds `myc-check.exe` on Windows (was POSIX-only, so
  the arm silently skipped); `reclaim_memory` guards the glibc `ctypes.CDLL(None)`/`malloc_trim`
  behind `os.name == "posix"` (it raises on Windows). RAM/ctx auto-sizing already degrade gracefully
  without `/proc`. The pipeline itself is stdlib + pathlib, so it runs on Windows (PowerShell:
  `$env:PYTHONPATH="."; python -m …`) and natively in WSL2.

### Added (2026-06-18: KC-2 `--model-id` shortcut)
- `--model-id ID` selects a cached registry model by name (e.g. `--model-id qwen2.5-coder-0.5b`)
  instead of typing a `.gguf` path. Registry-agnostic — resolves the id as a filename prefix in the
  cache dir; if it isn't fetched yet, errors with the exact `--ensure-model` command (never-silent).
  Mutually exclusive with `--model`.

### Added (2026-06-18: faster 0.5B coder model for KC-2 sweeps)
- Registered **`qwen2.5-coder-0.5b`** (Qwen2.5-Coder-0.5B-Instruct-GGUF, Apache-2.0) in the
  llm-harness model registry — ~2–3× quicker decode than the 1.5B on a phone CPU, where generation
  time dominates an unattended sweep. Fetch with
  `python tools/llm-harness/harness.py --ensure-model --model-id qwen2.5-coder-0.5b`.
- The KC-2 experiment now resolves a model by preference (`--model` → cached 0.5B coder → cached 1.5B
  coder → any `.gguf`), so fetching the 0.5B makes it the default automatically without breaking an
  existing 1.5B setup. The validation harness's own `DEFAULT_MODEL_ID` (1.5B) is unchanged — its
  structured-output gate wants the stronger model.

### Fixed (2026-06-17: KC-2 on-device timeout — durable runs + lighter, refreshing budget)
- **Root cause** of the on-device crash: the server backend used a fixed **180 s** read timeout
  (`__main__` never forwarded `--timeout` to it) while the phone decodes at ~0.3–0.7 tok/s, so a
  256-token generation always outran it and the whole suite aborted with **no report written**
  (14 min of work lost; the server log showed a healthy server still generating when the client
  gave up).
- **Durability**: every attempt now streams to `<run>.attempts.jsonl` (flushed per line) and the
  `index.json` is rewritten after every run — an OOM-kill or outer timeout loses nothing. A backend
  error mid-arm is **caught**, the arm is recorded as `partial` (honest rates over the tasks actually
  attempted), the run is flagged `interrupted`, and the sequence stops cleanly — never a lost report.
- **Timeout is per-generation and refreshes every attempt** (no cumulative suite timeout): `--timeout`
  is now forwarded to the server backend and defaults to **600 s**; the backend raises a clear,
  actionable error on a read timeout instead of an opaque `[Errno 110]`.
- **Lighter defaults / faster decode**: `--max-iters` default **3 → 2** (first try + one edit-to-fix),
  `--n-predict` default **256 → 128** (the task solutions are short), plus a `stop` sequence and
  `cache_prompt` on the server request. New `--limit N` runs only the first N tasks. README documents
  pointing `--model` at a lighter model (e.g. qwen2.5-coder-0.5b) for a real speedup.

### Added (2026-06-17: KC-2 gentle pre-run RAM reclaim)
- **`reclaim_memory()`** runs before context sizing on every run (opt out with `--no-reclaim`), so
  freed RAM is available to the model + KV cache (and reflected in `auto_ctx_size`). Non-destructive
  — `gc.collect()` + `malloc_trim(0)` (return freed heap to the OS) + `sync`, plus `drop_caches`
  **only if root** (skipped, never-silent, on an unrooted phone). It **never kills processes**;
  reaping orphan servers (the bigger lever) stays the explicit `--stop-server`. Logs a before→after
  delta; honest that the unrooted gain is modest (the kernel already counts reclaimable cache).

### Added (2026-06-17: KC-2 server teardown — auto, opt-out, and a standalone reaper)
- **Auto-teardown**: `--serve` already stops the server it launched after all reports/logs are
  written (the `try/finally`); **`--keep-server`** opts out (leave it up for a follow-up `--server`).
- **Orphan reaper**: `--stop-server` (optionally `--port N`) reaps running `llama-server` processes —
  for the orphan a manual `llama-server … &` leaves when it loses the port race — and exits.
  Standalone `tools/llm-harness/llama-server-stop.sh` does the same with no Python.
- Matching is by **executable name** (`argv[0]` basename `== llama-server`), excluding self — an
  early version used a cmdline substring (`pgrep -f llama-server`) that matched the teardown
  script's own path and killed the shell. New `find_server_pids` / `stop_external_servers`.

### Added (2026-06-17: KC-2 unattended pipeline — managed server, metrics/logs, suite runner)
- **Auto-managed llama.cpp server** (`mycelium_experiments/kc2/server.py`, `--serve`): loads the
  model ONCE, drives `/completion` (clean one-shot — no interactive REPL), **reuses a healthy server
  or picks a free port** (the manual `llama-server … &` hits "couldn't bind … port 8080" when an old
  server lingers), waits for `/health`, and tears down only what it launched. Never-silent on missing
  binary / early exit / not-ready.
- **Sequential, instrumented runner** (`mycelium_experiments/kc2/runner.py`): runs a *suite* of
  configs (e.g. `--seeds 42,123,7`) back-to-back, unattended, writing per run a `<utc>-<name>.json` +
  `.summary.txt` under `--results-dir` (default `experiments/results/`), plus a combined
  `index.json` and a suite `.log`.
- **Richer metrics**: `run_arm` gained an optional `on_attempt` observer; reports now carry
  **per-attempt records** (generated source, checker verdict, generation wall-time) and a `timing`
  block — well beyond the bare outcome rates.
- Decisions for this increment: *richer in-fragment tasks* (deferred) and *prove the pipeline first*
  (this) — the surface fragment can't express http-client/parser, so "realistic" stays in-fragment.
  Honesty unchanged (G2 SKIP-with-reason; VR-5 measured rates only, verdict maintainer-written).

### Fixed (2026-06-17: build-agnostic one-shot — EOF stdin + echoed-prompt strip)
- On-device the `b0-unknown` Termux `llama-cli` **ignored `-no-cnv`/`--no-display-prompt`** and still
  entered its interactive REPL (slash-command prompt), so a real run hung until Ctrl+C and echoed the
  prompt into stdout. Build-agnostic hardening (it can't depend on flag support):
  - **`stdin=subprocess.DEVNULL`** for the llama-cli subprocess in both `_call_llama_cli` (harness)
    and `cli_backend` (KC-2): a REPL that ignores the flags now hits EOF and **exits after the first
    response** instead of waiting on the terminal — no hang, no Ctrl+C.
  - **Echoed-prompt strip** in `LlamaGenerator`: if the verbatim prompt appears in stdout (a build
    that ignored `--no-display-prompt`), keep only what follows it before parsing.
  - For a guaranteed-clean path, prefer the **server backend** (`--server URL`, `/completion`) — no
    REPL, no conversation mode — documented as the recommended on-device route.

### Fixed (2026-06-17: KC-2 skip-reason rendering — line-aligned + concise summary)
- A skipped-arm reason that embedded a multi-line cargo error rendered badly: the checker's
  byte-tail (`detail[-1500:]`) cut **mid-line** (garbage like `y: cc help`), and the executive
  summary dumped the whole linker block into a one-screen overview. Fixes: `MyceliumChecker` now
  keeps the last **whole lines** (≤25, with a truncation marker) so the first line stays a concise
  reason; `render_summary` shows only that first line and points to the JSON `skipped` field for the
  full detail. Surfaced by the first real on-device KC-2 artifact (the run still SKIPped — the
  toolchain wasn't fixed in that shell — so it carried no model data, but the honesty chain behaved).

### Added (2026-06-17: capture the Termux/Android Claude Code bootstrap in-repo)
- **`tools/termux/cc-termux-bootstrap.sh` + `tools/termux/README.md`** — the proot-Ubuntu
  Claude Code setup used to develop Mycelium on a phone, version-controlled so it survives a
  toolchain reinstall (an ad-hoc copy was lost to `pkg install --reinstall clang`). It provisions a
  glibc Ubuntu via `proot-distro` (the official `claude` binary won't run native on Termux),
  installs Claude Code inside it, and installs a thin Termux launcher (`claude`/`work`/`sd`/`update`/
  `doctor`/`shell`).
- **Footgun fixed (root cause of the earlier build saga):** the launcher defaulted to `cc`, which
  overwrote `$PREFIX/bin/cc → clang` and broke every native build. It now defaults to **`claude`**
  and **refuses** compiler/toolchain names (`cc`/`clang`/`gcc`/…); use a shell alias for muscle memory.
- **Idempotent + secret-safe:** safe to re-run (reuse container, guard user creation, install Claude
  only if missing). No secrets in the script/repo — Claude auth stays interactive in `~/.claude`
  inside the container. Sudo is passwordless **by design**: the phone is unrooted (no Termux-side
  root, never used) and proot root is *emulated*, so a sudo password would guard nothing (anyone
  with Termux access can read the rootfs directly) — documented as the honest choice, not a gap.

### Fixed (2026-06-17: real-mode hang — force one-shot llama-cli, configurable timeout)
- **Real-mode runs hung until they timed out** because recent `llama-cli`, given `--prompt`, enters
  its **interactive conversation REPL**: it generated a correct answer, then waited at a `>` prompt
  forever (subprocess `TimeoutExpired`), and echoed the whole prompt into stdout. Confirmed on-device
  once `myc-check` built. Fix: both the harness `_call_llama_cli` and the KC-2 `cli_backend` now pass
  **`-no-cnv`** (generate once, exit at EOS) **+ `--no-display-prompt`** (stdout = completion only).
  Removable via `--llama-arg` / `--llama-extra-arg` if a build rejects them; `--server` mode was
  always clean.
- **Per-generation timeout is now configurable and more generous** (`--timeout`, default 300s, up
  from a hard-coded 120s) — CPU phones run ~1–2 tok/s, so the old default was too tight. README
  updated.

### Added (2026-06-17: Termux/ARM64 myc-check build prerequisites documented)
- **`experiments/README.md` documents the Termux (Android/ARM64) Rust build failure modes** for
  `myc-check`, found by an on-device build and the never-silent cargo-error surfacing. The actual
  blocker on the test device was a **personal `cc`/`clang` wrapper shadowing the compiler** in
  `$PREFIX/bin` (`note: Unknown command '…/symbols.o'. Try: cc help`); rustc links every build script
  via `cc`, so all failed — and `$PREFIX/bin/clang` was the wrapper too, so pointing at it didn't
  help. Fix: use the **versioned** clang (`CC=$PREFIX/bin/clang-NN`, the matching
  `CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER`, or `~/.cargo/config.toml` `linker = "clang-NN"`), or
  un-shadow it (rename the wrapper, `pkg install --reinstall clang`); never name a personal script
  `cc`/`clang`/`gcc`. The note also keeps the missing-library case (`libandroid-spawn`/`-lXXX`).
  Use the Termux-packaged rust, not rustup. No code change.

### Fixed (2026-06-17: KC-2 Mycelium arm — wrong cargo package + swallowed build error)
- **The KC-2 Mycelium arm always SKIPped because the checker built the wrong crate.**
  `MyceliumChecker._discover` ran `cargo build -p mycelium-l1 --bin myc-check`, but the `myc-check`
  binary lives in the **`mycelium-check`** crate — cargo exited 101 ("no bin target named
  myc-check in mycelium-l1"), which the harness honestly reported as a SKIP (never a false pass).
  Fixed the package name; the arm now builds + runs (the full experiment test suite goes from
  partially-skipped to all-pass).
- **Never-silent gap closed:** a failed `cargo build` now surfaces cargo's actual stderr in the
  SKIP reason (tail, truncated) instead of a bare `exit status 101`, so a *real* compile failure on
  a new platform (e.g. aarch64/Termux) is actionable. `experiments/README.md` build command fixed
  to `-p mycelium-check`.

### Added (2026-06-17: swap budget, SD-card overflow, desktop GPU auto-offload)
- **Optional swap budget (`--use-swap`):** auto context sizing can count ~half of free swap toward
  the memory budget, letting the context grow when RAM is tight — with an explicit speed/thrash
  caveat (swap is still *off by default*; the OOM killer targets RSS).
- **GPU enumeration + auto-offload:** `detect_gpu()` (NVIDIA via `nvidia-smi`, AMD `rocm-smi`,
  Apple Metal) + `auto_gpu_layers()` pick `-ngl` automatically on a desktop with a GPU build —
  full offload when detected VRAM holds the model, else CPU. `--cpu-only` forces CPU; `--n-gpu-layers`
  sets it explicitly. A phone's CPU-only `llama.cpp` reports no GPU, so it's a no-op there.
- **External-storage (SD) reporting:** `detect_external_storage()` surfaces roomy shared/SD volumes
  so they can host the model cache (`--model-dir`) or back a swapfile (root) — informational; never
  auto-mounted/auto-swapon'd.
- **`--doctor`** gains **GPU** and **External storage** sections, shows the context a run would pick
  *with* `--use-swap`, and the same flags (`--use-swap`/`--cpu-only`/`--n-gpu-layers`) exist on the
  KC-2 entry point. All choices are logged with their inputs (EXPLAIN/G2); honest fallbacks when a
  resource is unknown.

### Added (2026-06-17: auto memory enumeration + auto context sizing)
- **The context size is now auto-tuned from the device's available memory** instead of a fixed
  default — the harness and the KC-2 backend enumerate RAM/swap (`/proc/meminfo`, with a POSIX
  `sysconf` fallback) and pick `min(workload need, what available RAM safely holds with headroom)`.
  New `detect_memory()` + `auto_ctx_size()` in both `tools/llm-harness/harness.py` and
  `experiments/.../kc2/llm.py`; `--ctx-size` now defaults to **auto** (pass an explicit `N` to
  override). Swap is detected and reported but **not** counted toward the budget (KV/compute thrash
  and still trip the OOM killer if paged) — an honest, conservative input.
- **No black box (EXPLAIN/G2):** the chosen context is logged with every input (available RAM,
  model size, reserve, the per-token KV assumption, the workload need). `--doctor` gains a
  **"Memory + auto context size"** section showing detected RAM/swap and the context a run would
  pick, and recommends the `qwen2.5-0.5b-instruct` tier when headroom is thin. Memory unknown ⇒ a
  conservative default, never a guess.

### Fixed (2026-06-17: real-mode OOM — cap the llama.cpp context / KV cache)
- **On-device real-mode runs were SIGKILLed (`[Process completed (signal 9)]`) at model load.**
  Cause: with no `-c`, llama.cpp allocates a KV cache for the model's *full trained context*
  (Qwen2.5 = 32k), which — on top of the weights — trips the Android low-memory killer on a phone.
  The harness's prompts are tiny, so that window was never needed.
- Fix: both the validation harness (`tools/llm-harness/harness.py`) and the KC-2 backend
  (`experiments/.../kc2/llm.py`) now pass `--ctx-size`/`-c` with a small default (**2048**),
  tunable via `--ctx-size`. Added a `--llama-arg` / `--llama-extra-arg` passthrough so
  conversation-mode/prompt-echo flags (`-no-cnv`, `--no-display-prompt`) can be supplied per build
  without editing code. `experiments/README.md` gains a signal-9 troubleshooting note (lower
  `--ctx-size`, or use the `qwen2.5-0.5b-instruct` tier). Because SIGKILL can't be caught, this is
  prevention: keeping the run alive is what lets it reach the report-writing step.

### Added (2026-06-17: KC-2 run — executive-summary assessment of the results)
- **A KC-2 run now emits a descriptive executive summary alongside the raw rates** — new
  `experiments/mycelium_experiments/kc2/summary.py` (`assess` + `render_summary`). Per arm it
  reports first-attempt vs eventual pass, a coarse rating (strong/moderate/weak), the edit-to-fix
  (G10) leverage gain, and which tasks never passed / parsed-but-failed-first; with both arms it
  reports the comparison gap. One assessment, two projections (G11): a structured `assessment`
  block in the JSON + a human `*.summary.txt` companion (and printed to the console).
- **Honesty:** the summary *characterises*, it does not decide. The `decision` field and the
  rendered footer state the KC-2 verdict stays maintainer-written (VR-5); caveats flag the coarse
  small-n signal and the primer/model/seed dependence so the reader doesn't over-read.

### Added (2026-06-17: KC-2 experiment runnable against a local llama.cpp model)
- **The KC-2 LLM-leverage experiment (M-002) can now be *run*, not just structured.** The only
  documented blocker was "needs LLM API access"; local llama.cpp removes it. New pieces, all pure
  stdlib, all never-silent (G2) and verdict-free (VR-5):
  - `experiments/mycelium_experiments/kc2/llm.py` — a `LlamaGenerator` (implements the harness
    `Generator` protocol) over a `llama`/`llama-cli` subprocess **or** a llama.cpp HTTP server, with
    per-arm **primers** (generator configuration — generic syntax cheatsheets, no task answers),
    prompt assembly with edit-to-fix feedback, and best-effort source extraction (fences/prose).
  - `experiments/mycelium_experiments/kc2/__main__.py` — `python -m mycelium_experiments.kc2`:
    runs the requested arms, writes a JSON report. An unavailable `myc-check` **SKIPs** the Mycelium
    arm with an explicit reason (never a fake 0%); the baseline arm **executes generated Python** so
    it is **off by default** (opt in with `--allow-untrusted-baseline`, inside a sandbox). A missing
    binary/model aborts with an actionable message, never a silent empty generation.
  - The KC-2 **verdict is still maintainer-written** — these scripts emit measured rates only (VR-5).
  - `experiments/README.md` — the end-to-end run order (doctor → validations → unit tests → the
    real KC-2 run), with the `myc-check` build, the baseline-sandbox caveat, and the primer note.
  - Grounding: M-002 (#3), SC-5b, G10; the existing KC-2 harness/checkers/tasks unchanged.

### Changed (2026-06-17: LLM-harness — readiness verdict in `--doctor`)
- **`--doctor` now ends with a bottom-line READY / NOT READY verdict** for real-mode validations
  (it needs both a llama.cpp CLI and a local model), naming the exact next command or the one fix
  per miss — so the dense report has a single line to read. A NOT-READY state is honest that
  real-mode would **SKIP**, not fail (G2).

### Changed (2026-06-17: LLM-harness — first-class `llama` alias + clearer doctor)
- **The harness now treats `llama` as a first-class CLI alias, not just `llama-cli`.** The Termux
  `llama-cpp` package installs the CLI as plain **`llama`** (confirmed on-device: `which llama-cli`
  is empty, `which llama` resolves to `$PREFIX/bin/llama`). Discovery already matched `llama` via
  `_LLAMA_BIN_NAMES` since the off-PATH work; this pass makes the rest of the surface honest:
  - `--doctor`'s section header is now **`llama.cpp (llama-cli / llama)`** and prints the resolved
    **alias** alongside the path, so it's clear *which* name was found.
  - The off-PATH **glob fallback** in `_resolve_llama_cli` now also matches `llama` (not only
    `llama-cli`), so a hand-built `llama` is found too.
  - Real-mode/install warnings now say **"llama.cpp CLI (llama-cli / llama)"** instead of bare
    `llama-cli`, removing the impression the harness only wants `llama-cli`.
  - Package builds self-report `version: 0 (unknown)` (no embedded git metadata); the doctor now
    de-duplicates a leading `version:` so it no longer renders `version: version: …`.
  - A **KNOWN FOLLOW-UP** is documented in `_call_llama_cli`: recent builds default to interactive
    *conversation* mode and may echo the prompt, which would distort the one-shot completions V-01/V-02
    parse. The `-no-cnv` / `--no-display-prompt` fixes are noted but **not** added blindly (flag
    availability varies by build); to be validated against the target binary, or use `--server` mode.
  - Grounding: G2 never-silent (a missing CLI still SKIPs honestly). README Termux step clarified.

### Changed (2026-06-17: LLM-harness — package/release installs, not Python packages)
- **Bootstrap now installs runtime tools from the OS package manager / official releases instead of
  fragile language-package builds.** The Termux failure that kept recurring was `--doctor` trying to
  `uv tool install huggingface_hub[cli]`, which builds the native **`hf-xet`** dependency from source
  (no aarch64 wheel) and fails. Fixes:
  - **llama.cpp** is now installed from the **system package manager** — Termux `pkg install llama-cpp`
    (repo-signed, prebuilt; binaries land on `$PREFIX/bin`), `brew install llama.cpp` — with a
    detector for `pkg`/`apt-get`/`dnf`/`pacman`/`zypper`/`brew`. Where no package exists, the harness
    prints the vetted from-source / pinned-release steps and SKIPs honestly rather than guessing.
    `--doctor` runs this (with consent) when `llama-cli` is missing.
  - **The hf CLI is now OPTIONAL and never auto-installed.** The built-in **stdlib downloader** is the
    default model-fetch path and now sends `Authorization: Bearer $HF_TOKEN` to `huggingface.co`, so
    **gated repos work without the CLI**. `--install-hf-cli`/`--setup-hf` remain as explicit opt-ins
    (with a warning that the `hf-xet` build may fail on aarch64).
  - **Checksum gate added:** `--model-sha256 HEX` (or a pinned registry value) is **verified** before a
    download is promoted; a mismatch is a loud failure (the `*.part` is kept). No fabricated checksums
    are stored (honesty rule); absent a pinned value, integrity still rests on the GGUF magic + complete
    transfer. New supply-chain helpers: `_detect_system_pkg`, `install_system_package`, `sha256_file`,
    `verify_sha256`, `install_llama_cpp`. README updated (Termux Step 1/2, download section, `--doctor`).
  - Grounding: CONTRIBUTING.md supply-chain rule (no `curl|bash`, no unpinned fetch), G2 never-silent.

### Changed (2026-06-17: LLM-harness — `--doctor` is now self-healing)
- **`--doctor` diagnoses *and* heals by default** instead of only reporting fixes. When a required
  package is missing it now installs it — the **hf CLI** via `uv`/`pipx`/`pip` and the **Claude Code
  CLI** via `npm install -g @anthropic-ai/claude-code` (never `curl|bash`, per the CONTRIBUTING
  supply-chain rule) — **links** an installed-but-unlinked `claude` (`cli.js`) onto `PATH`, **fixes
  `PATH`** (healing implies `--fix-path`, persisting to the shell rc), and offers to **download the
  default model** if absent. Every mutation **prompts for consent unless `--yes`**; a non-interactive
  run without `--yes` declines safely (never-silent, G2). A wrong-arch/corrupt `claude` (an
  `Exec format error`) is reported with the reinstall command rather than auto-"fixed" (arch can't be
  patched). New **`--check-only`** flag restores the prior read-only report (no installs, no `PATH`
  writes). On Termux, the npm install first points npm's global prefix at `$PREFIX` so the `claude`
  link lands on the existing `PATH`. README Troubleshooting section updated.

### Added (2026-06-17: LLM-harness — robust binary discovery, PATH self-healing, `--doctor`)
- **`tools/llm-harness/harness.py` now resolves tools that are installed but off-`PATH`** — the
  real-world Termux failure (`pip --user` → `~/.local/bin`, hand-built `llama.cpp` →
  `~/llama.cpp/build/bin`, npm CLIs unlinked). Discovery searches `PATH` first, then the dirs
  installers/builds actually use, for **llama.cpp** (`~/llama.cpp/build/bin`, `$PREFIX/bin`,
  `$MYCELIUM_LLAMA_DIR`, shallow globs), the **hf CLI** (interpreter scripts dir, `~/.local/bin`,
  pipx/uv venvs, `$PREFIX/bin`; plus a `python -m huggingface_hub…` fallback when the package is
  importable but no console script is linked), and the **Claude Code CLI** (npm global bin via
  `npm config get prefix`, nvm/bun/volta/pnpm dirs, `$PREFIX/bin`). A found-off-`PATH` binary is
  **self-healed** into the current run's `PATH` (so child processes see it) with the exact
  `export PATH=…` surfaced; **`--fix-path`** persists that line to the shell rc (idempotent;
  prompts unless `--yes`).
- **New `--doctor`** subcommand: prints platform/PATH, installers, and the resolved state of
  llama.cpp, the hf CLI (+ auth), the Claude Code CLI, and the model cache — with where it looked
  and the precise fix for each miss. The thing to run on a phone and paste back. New flags:
  `--doctor`, `--fix-path`, `--claude-cli PATH`. hf-CLI handling refactored to an argv *prefix*
  (supports the `-m` fallback) and the Termux `pip` install no longer forces `--user` (which is
  the off-`PATH` trap there). README: new Troubleshooting section.
- **Cached-model reuse + present-model fast path.** Real mode now reuses a model already in the
  cache **without** `--ensure-model` (the post-download walk-away property), and `--ensure-model`
  **skips hf-CLI setup entirely when the model is already present** (no nagging for a tool it
  won't use). `--doctor` reports an **installed-but-unlinked** Claude Code CLI (npm package found
  but no `claude` on `PATH`) with the exact symlink/relink fix. Exit is now deterministic with an
  explicit stdout/stderr flush (guards against a spurious "Aborted" at Termux interpreter teardown).

### Added (2026-06-17: LLM-validation harness — Hugging Face CLI integration)
- **`tools/llm-harness/harness.py` gains Hugging Face CLI support** for model acquisition. On
  `--ensure-model` it now **detects** the `hf` CLI (or legacy `huggingface-cli`), uses it as the
  **preferred** download path (resumable, auth-aware, gated-repo-capable), and **falls back** to the
  built-in stdlib downloader when it's absent — nothing breaks either way. New flags: `--setup-hf`
  (detect → install → check/prompt auth, then exit), `--install-hf-cli`, `--no-hf-cli`, `--hf-cli PATH`,
  `--hf-token TOKEN`, `-y`/`--yes`. **Auth** is checked (`hf auth whoami`) and, if missing, prompts an
  interactive `hf auth login` (or accepts `--hf-token`/`$HF_TOKEN`) — **non-fatal**, since the default
  registry is public. Honesty/supply-chain (CONTRIBUTING.md): install uses the published
  `huggingface_hub[cli]` package via **uv/pipx/pip — never `curl … | bash`** (the upstream one-liner is
  printed as a reviewed manual fallback only); an hf-CLI download is held to the **same GGUF-magic
  verification** as the stdlib path (G2 never-silent). Detection also searches `~/.local/bin` and
  `$PREFIX/bin` so a **Termux / `pip --user`-installed-but-unlinked** `hf` is still found and used,
  with the exact `export PATH=…` fix surfaced. On **Termux** the install-guidance is tailored
  (`pkg` ≡ `apt`: `pkg install python`/`pipx`/`uv` to get an installer first). README updated
  (hf-CLI section + a Termux-PATH/`pkg`≡`apt` FLAG for `hf`/`claude` "command not found").

### Changed (2026-06-17: RFC-0016 ratified — Draft → Accepted, the standard-library keystone)
- **RFC-0016 (Core Library & Standard Library) moves `Draft → Accepted`** by maintainer ratification
  (DN-07 ratification pass; M-501, #149). The §4.1 per-op contract (C1–C6), the §4.2 ring layering, the
  §4.3/§4.4 Tier-A/Tier-B taxonomy (**full 23-module v0 scope**), the §4.5 guarantee-matrix obligation, and the
  §4.6 Rust-first → Mycelium-lang migration order are ratified. **§8 dispositions** (recorded append-only in
  RFC-0016 §8 + changelog): **Q1** full taxonomy / five-candidate floor first / `diag`·`recover` lead, **Q2**
  phylum `std` + crate-mirrored names + one `core`↔`error` error-value name, **Q5** two-level differential
  bar, **Q6** `std-sys` phylum split, and the `BF16→F32` placement (→ `cmp`/`convert`) are **resolved**;
  **Q3** ergonomics-vs-contract accepts the RFC-0012 ambient *direction* with a scheduled per-ring pass
  (**M-540**) and **Q4** `runtime` placement defers to the Phase-7 gate — both deferred-with-direction.
- **DN-07 moves `Draft → Resolved`** (its job — framing the ratification pass — is complete).
- **The concrete L3 *authoring* surface stays KC-2-gated** (A2 ruling; RFC-0006 §10) — the deciding
  experiment M-002 (#3) is unrun (needs LLM API). So the M-502 self-hosting verdict honestly stays
  *not-yet*, the Mycelium-lang migration half of M-510…M-520 waits, and the Rust-first specs/impls proceed.
- Status synced across `docs/Doc-Index.md`, the stdlib spec index (`docs/spec/stdlib/README.md` §4/§5),
  and `self-hosting-readiness.md`.

### Added (2026-06-17: RFC-0016 §7 grounding discharged + the LLM-validation harness scaffold)
- **Research Record 08** (`research/08-honest-stdlib-prior-art-RECORD.md`) — discharges the RFC-0016 §7
  pre-ratification grounding obligation: the cross-language stdlib module-set comparison (Rust/Python/Go/
  OCaml–Haskell → T8.1–T8.4) and the "honest stdlib" prior art (refinement-typed/verified/effect-tracked
  standard libraries → T8.5–T8.7), grounding the Tier-B taxonomy, the ring layering + `std-sys` split, and
  the §4.1 honesty contract; flags the 4-point honest-degradation lattice as Mycelium's novel,
  precedent-free contribution. Tagged Empirical/Declared, never Proven (VR-5).
- **A portable LLM-validation harness** under `tools/llm-harness/` (Workstream B; de-risks the backlogged
  M-330 #97/#127 + M-002 #3) — targets llama.cpp (GGUF) and runs under Termux on Android, with a `--mock`
  dry-run mode (no model; skips gracefully, exercises the plumbing) and a real mode (shells to a local
  `llama-cli`/server). Emits a structured JSON + human report and a timestamped log per run (dual projection
  G11), every validation PASS/SKIP/FAIL explicit and tagged (RFC-0013 I1); a model-absent tool is an
  explicit SKIP, never a false pass. Validations: deterministic-seed round-trip, JSON-projection
  conformance, the guarantee-tag honesty gate (model-derived ⇒ Empirical/Declared, never Proven), and a
  latency/token report. Lives above the kernel (KC-3).

### Added (2026-06-17: standard-library second design wave — the remaining 13 module specs, integrated)
- **Thirteen per-module standard-library design specs** under `docs/spec/stdlib/`, completing the RFC-0016
  taxonomy (all 23 modules now `Draft`). Each is authored to the uniform template and the §4.1 contract,
  shipping its load-bearing **guarantee matrix** (ops × {tag · fallibility · declared effects · EXPLAIN-able})
  and explicit **C1–C6 conformance**: Tier-A **`numerics`** (M-512, #153 — certificate consumer above the
  ADR-010 kernels; tags never upgraded past basis; homes the `Approx<T>` carrier `math`/`dense` deferred),
  **`vsa`** (M-513, #154 — per-`(model,op)` tags read from the RFC-0003 §4 matrix; reconstruction held at the
  FR-C2 probabilistic-only ceiling), **`diag`** (M-510, #151 — the self-hosted structured-diagnostics record;
  presentation never gates propagation, I1), **`recover`** (M-520, #156 — the reified `Outcome`/recovery-policy
  subsystem; every error recovered or re-propagated, never dropped; elaborates to L0 `Match`, no new kernel
  node), **`runtime`** (M-521, #162 — the RFC-0008 concurrency lexicon as reserved vocabulary, Phase-7-gated,
  no premature surface), **`spore`** (M-522, #163 — content-addressed deployable + reconstruction manifest;
  deterministic hash; full native deploy Phase-6-gated on M-620); Tier-B **`collections`** (M-511, #152 —
  value-semantic, no silent reorder), **`text`** (M-524, #165 — `parse → Result`, lossy encoding explicit),
  **`io`/`serialize`** (M-514, #155 — checked round-trip, serialization-is-projection, one canonical JSON),
  **`fs`** (M-528, #169 — every path/permission failure explicit; audited `wild` floor), **`time`** (M-529,
  #170 — monotonic/wall/logical a typed distinction; reads are declared effects), **`rand`** (M-531, #171 —
  entropy a declared effect, seeded vs entropy generators distinct), **`testing`** (M-534, #174 — property/
  golden/differential harness; a skipped check is reported, never a silent pass). Honest throughout (VR-5):
  no `Proven` without a checked basis, no fabricated crate API / bound / schema — genuine unknowns FLAGGED.
- **A common failure-legibility rule, recorded once and consumed everywhere.** A Mycelium program *may*
  legitimately fail/refuse for a specific error case, but every failure is a structured **RFC-0013** record
  with a clear trace + actionable debug info, and is recovered or re-propagated — **never silently swallowed**
  (I1). Discharged in `diag`, consumed by `recover` (policy), `testing` (a `Fail` is a `diag` record), and
  every module's `Err` rows.
- **Cross-module reconciliation extended (stdlib README §5).** The second wave *resolved* two prior deferrals
  — the numerics carrier (`Approx<T>` = a `Meta`-attached bound, closing `math`/`dense`) and the recovery
  bridge (`recover` now owns the concrete `Outcome`/`PolicyRef`) — and *converged* the JSON projection (`fmt`
  delegates to `serialize`) from both sides. New seams recorded for the consolidated `wild`/`std-sys` floor
  (`fs`/`rand`/`math`, §8-Q6), the shared RT3 declared-nondeterminism rule (`time`/`rand`), the reserved
  `runtime` Phase-7 phylum (§8-Q4), the deployable-spore boundary, and the reused differential bar (§8-Q5).
  No two specs conflict on an owned surface; open items are the known §8 questions, not silent decisions.
  Design-first; no code; no kernel change (KC-3).

### Added (2026-06-17: standard-library first design wave — 11 module specs + the M-502 gate, integrated)
- **Eleven per-module standard-library design specs** under `docs/spec/stdlib/`, each authored to the
  uniform template and the RFC-0016 §4.1 contract, each shipping its load-bearing **guarantee matrix**
  (ops × {tag · fallibility · declared effects · EXPLAIN-able}) and explicit **C1–C6 conformance**:
  Tier-A differentiators **`core`** (M-515 — Ring-0 honest value model, re-export-only), **`swap`** (M-516
  — certificate-carrying representation change over the one M-210 checker), **`ternary`** (M-517 —
  balanced-ternary algebra Exact + inspectable I2_S/TL1/TL2 packing), **`dense`** (M-518 — typed
  `Dense{dim,dtype}`, ε via ADR-010, Proven only where checked else downgraded), **`select`** (M-519 — the
  total non-learned policy + mandatory EXPLAIN), **`content`** (M-523 — content-addressing identity, ADR-003);
  Tier-B commons **`iter`** (M-526 — totality-preserving folds, the one lazy combinator named, not silent),
  **`math`** (M-525 — domain errors explicit, transcendentals carry their ε tag), **`error`** (M-527 —
  errors-as-values with the structural I1 never-silent floor), **`cmp`** (M-532 — the convert-vs-swap
  boundary; lossy narrowing explicit), **`fmt`** (M-533 — dual human/machine projection, display ≠ identity).
  Honest throughout (VR-5): no `Proven` tag without a checked basis, no fabricated crate API / bound /
  schema — genuine unknowns are FLAGGED, not invented.
- **Cross-module reconciliation (stdlib README §5).** The independently-authored specs are deconflicted: the
  **swap ↔ convert** boundary and the **numerics-ε ownership** (dense/math → M-512) are *consistent* and
  resolved in-wave; the recurring **naming** (§8-Q2) and **ergonomics-vs-contract** (§8-Q3) items are
  corroborated from eleven angles as signal for RFC-0016's ratification pass; `fmt↔serialize`, the
  `error↔recover` bridge, and `iter`'s early-termination question are FLAGGED to their owning tasks. No two
  specs conflict on an owned surface. Design-first; no code; no kernel change (KC-3).

### Added (2026-06-17: standard-library per-module spec scaffold — Phase-5 design wave orchestration)
- **`docs/spec/stdlib/` — the per-module standard-library spec directory** (Living index + uniform
  `_TEMPLATE.md`), decomposing **RFC-0016 (Draft)** into one design spec per module. The index restates the
  load-bearing **§4.1 per-op contract** (C1–C6) and the **guarantee-matrix** obligation (RFC-0016 §4.5 —
  ops × {tag · fallibility · declared effects · EXPLAIN-able}) as the shared spine every module spec traces
  to, and keys each spec to its Phase-5 task (M-510…M-534). The template enforces **single-template
  conformance** (the §4.1 doc quality-bar lint) so the specs stay uniform + reviewable. First wave marked
  `design landing`: Tier-A differentiators `core`/`swap`/`ternary`/`dense`/`select`/`content` + Tier-B pure
  commons `iter`/`math`/`error`/`cmp`/`fmt`; the remainder `anticipated` for later waves. Design-first — no
  code, no kernel change (KC-3); ratification per module is the maintainer's append-only decision.
- **`docs/spec/stdlib/self-hosting-readiness.md` (M-502, #150)** — the **self-hosting readiness gate** as a
  *checkable verdict*: an eight-row capability checklist (data+matching · functions/closures/recursion ·
  concrete L3 surface · a running term-language prototype · surface guarantee tags · surface effects ·
  ambient repr · organization/packaging) assessed against the landed corpus, composed into an honest
  **not-yet-established** verdict — the *substrate* is ready (RFC-0011/RFC-0001 r4 data/recursion/closures,
  the lattice + effect model, DN-06 packaging), the *surface* to author + run a module is not (concrete L3
  syntax KC-2-gated; M-320 #92 open). Records what the gate blocks (the Mycelium-lang migration half of
  M-510…M-520) vs what proceeds regardless (RFC-0016 ratification, the per-module specs, the Rust-first
  implementations). Never pre-declared (VR-5).
- **`docs/Doc-Index.md`** — indexes the new `docs/spec/stdlib/` directory.

### Added (2026-06-17: M-363 documentation BUILD pipeline + the §4.1 doc quality-bar lint — Phase 9 Wave B)
- **`crates/mycelium-doc/` — the M-363 doc BUILD pipeline** (≈3.5k LoC, tested), enacting the ratified
  `docs/spec/Narrative-Authoring-Pipeline.md`. A **content-addressed doc-IR** (`ir.rs`, reusing the
  kernel's BLAKE3 `ContentHash` shape — ADR-003) into which the corpus (RFCs/ADRs/notes/specs, via a
  dependency-free CommonMark-subset parser, `corpus.rs`), the JSON schemas, and the **M-359 nodule-
  header metadata** (`apiref.rs`) are **projected, never authored** — an item that cannot be grounded
  is an explicit `undocumented` node, **never invented** (the prose analogue of G2). Many renderers
  (`emit/`): a semantic-HTML site, a **Typst** projection (→ PDF; compile skips gracefully when
  `typst` is absent — never a half-build), and a machine **JSON/JSONL** view — all *views of one IR*
  (G11/ADR-003). **EPUB is an honest deferral** (spec §8.5), recorded, not half-built.
- **The §4.1 doc quality-bar lint is now ACTIVE** (`mycelium_doc::doc_lint`): the eight checks
  (single-template-conformance · navigability · progressive-disclosure · **checked-examples** ·
  no-dead-xref · **dual-projection-parity** · no-hallucinated-prose · legibility-accessibility) run over
  the doc-IR. Checked inline examples **actually type-check** via the trusted L1 checker (the same
  `parse → check_nodule` pipeline `myc-check` uses); legibility is honestly **partially-dormant**
  (structure checked; colour-contrast/typography need a rendering engine). `mycelium_lint::doc_lint_status()`
  flips **dormant → active**, sourcing the canonical check-name set from `mycelium-doc` (DRY).
- **`scripts/checks/myc-doc.sh` (+ wired into `scripts/checks/all.sh`)** — a gated step that fails on any
  error-severity §4.1 finding. Green-and-real over the live corpus: 98 documents / 2632 content-addressed
  nodes, 6 examples type-checking, internal xrefs resolving, HTML/JSON parity across all nodes. Skips
  gracefully when `cargo` is absent. KC-3: above the kernel; **no kernel change; no new third-party
  dependency** (this adds the in-repo `mycelium-doc` crate; blake3/serde/serde_json were already vetted).

### Changed (2026-06-17: harden the GitHub PM sync engine — graceful gh failures + least-privilege auth automation)
- **`tools/github/gh-issues-sync.py` — no raw tracebacks (G2).** Every `gh` failure now exits with an
  **explicit, classified remediation** (re-auth / missing-scope / rate-limit / network), replacing the
  unguarded `proc.check_returncode()` that surfaced a `CalledProcessError` traceback on a `gh api` 401
  inside `reconcile_prs`/`reconcile_project`; a top-level guard in `__main__` catches anything else.
  Both the direct run and the `--all` wrapper path now fail gracefully.
- **Least-privilege gh-auth automation (new).** Preflight computes the **minimal** classic-OAuth scope
  set from the *arg'd* operation set (offline ops → none; repo writes → `public_repo` when the target
  is public, else `repo`; `--project` → `read:project` when read-only/dry-run, else `project`),
  compares it to the active token, and — only for a genuinely-absent needed scope — prints an **EXPLAIN**
  (ops → scopes → command) and, **with explicit consent**, runs `gh auth refresh/login -s <exact set>`
  (changing scopes is a state mutation: opt-in, never silent — G2). An **over-granted** token gets a
  non-blocking advisory; the classic-scope **granularity floor** is documented (a fine-grained PAT is
  the path to tighter per-resource perms, trusted to fail loudly). Implemented **once** in the engine;
  both wrappers (`gh-sync-all.sh`/`.ps1`) route through it via `--all` and forward a **`--no-auth-fix`**
  CI escape hatch. Pure scope logic is `--self-test`-covered.
- **`tools/github/conventions.json`** — added the ratified `examples → toolchain` scope alias (clears
  PR #145's flagged `examples` scope; verified via `derive_pr_labels` + `--self-test`/`--validate`).
### Added (2026-06-17: the full standard-library roadmap — RFC-0016 (Draft) + Phase-5 decomposition)
- **`docs/rfcs/RFC-0016-Core-Library-and-Standard-Library.md` (Draft)** — the **Core Library RFC** the
  M-346 stdlib epic anchors and M-501 names. It fixes (1) the **per-op contract** every stdlib operation
  must meet — **C1** never-silent (G2), **C2** honest per-op guarantee tag on the `Exact ⊐ Proven ⊐
  Empirical ⊐ Declared` lattice (VR-5), **C3** no black boxes / EXPLAIN (SC-3/G11), **C4** content-addressed
  value-semantics (ADR-003), **C5** above the small kernel (KC-3), **C6** declared/bounded effects
  (RFC-0014) — verified per module by a **checked guarantee matrix** (the RFC-0003 §4 template), not prose;
  (2) the **module taxonomy** split into **Tier-A differentiator** modules (each the library form of an
  Accepted RFC/ADR — `swap`/`numerics`/`vsa`/`ternary`/`dense`/`select`/`diag`/`recover`/`runtime`/`spore`/
  `content`) and **Tier-B common** modules (`collections`/`text`/`math`/`iter`/`error`/`io`/`fs`/`serialize`/
  `time`/`rand`/`cmp`+`convert`/`fmt`/`testing` — table-stakes, held to the *same* contract); (3) the
  **ring layering** (Ring 0 kernel-adjacent re-exports · Ring 1 capability surfaces · Ring 2 general
  library) that keeps KC-3 honest; and (4) the **Rust-first → Mycelium-lang migration** (dogfooding; gated
  by the M-502 readiness verdict, `diag`+`recover` the first targets per the charter). **Six §8 questions
  FLAGGED** (v0 module set/priority, naming, ergonomics-vs-contract tension A, `runtime` Phase-7 sequencing,
  the migration differential bar, the `wild`/FFI floor) and a §7 `research/` prior-art obligation recorded —
  both clear before ratification (G2: an ungrounded module is FLAGGED, never invented). No code; ratification
  is the maintainer's append-only decision (M-501). No kernel change (KC-3).
- **`docs/planning/phase-5.md`** — the Phase-5 working plan (mirroring `phase-2.md`/`phase-3.md`): the
  keystone + gate (M-501/M-502), the Tier-A/Tier-B task tables, the batch/sequencing plan (Ring 0/1 →
  Ring 2 commons → self-hosting; `runtime` Phase-7-gated), and the six carried §8 FLAGs. Anticipated, not
  ratified.
- **`tools/github/issues.yaml`** — **18 new stdlib module tasks** (`M-515…M-534`, append-only) decomposing
  RFC-0016's taxonomy, on top of the 8 keystone/seed Phase-5 tasks (Phase-5 count 8 → **26**). Each grounded
  in its corpus RFC/ADR, `status:needs-design`/`P3`; numbers minted at the Phase-5 gate (the M-364…368
  staging precedent). `--validate` (129 issues) + `--self-test` + `scripts/checks/all.sh` green. RFC index
  (`docs/rfcs/README.md`) + `docs/Doc-Index.md` updated (RFC-0015 backfilled, RFC-0016 added);
  `tools/github/MILESTONES.md` summary + Meta changelog updated. Docs + manifests only — no crate/kernel
  change (KC-3).

### Added (2026-06-17: PM phase-allocation reconcile — Phase-2 M-2xx back-fill + M-351; Phase 5 & 6 task sets)
- **`tools/github/issues.yaml` — the 19 absent task-ids recovered (append-only).** `gh-issues-sync.py
  --validate` flagged that **19 task-ids in `idmap.tsv` had no entry in `issues.yaml`** (the manifest was
  the incomplete side): the **18 Phase-2 `M-2xx`** epic decompositions (`M-201…M-260`, #48–#65) and
  **`M-351`** (#114). All are now written back into the manifest — **grounded entirely in the cited
  corpus** (`docs/planning/phase-2.md` §2/§6 for every M-2xx title/priority/dependency/delivery detail;
  `CHANGELOG` "Decided (Phase 4 — M-351 …)" + RFC-0012 §8/§9 for M-351), reconstructed and never invented
  (the planning analogue of never-silent, **G2**). All carry `status:done` (Phase-2 exit gate met
  2026-06-12; M-351 decided 2026-06-16) — a label, **not** a state change (the reconciler never infers
  OPEN/CLOSED from a `status:*` label). **Honesty FLAG:** the PM-task brief called M-351 a "Phase-3
  toolchain task", but the corpus + `idmap` place it in **Phase 4** (the M-344 ambient follow-up,
  RFC-0012 R12-Q1/Q2); it is filed where the corpus grounds it, with the discrepancy recorded in a
  section comment rather than silently followed.
- **Phases 5 & 6 are no longer empty.** `--validate` also flagged the `phase:5`/`phase:6` labels **and**
  the "Phase 5 — Self-Hosting & Core Library" / "Phase 6 — Native Acceleration & Deployment" milestones as
  *defined but unused*. Both phases are now decomposed into **grounded, design-first** task sets (all
  `status:needs-design`, `priority:P3`, scoped to what the roadmap implies — not over-invented):
  **Phase 5** (`M-501` Core Library RFC keystone, `M-502` self-hosting readiness gate, the five M-346
  candidate stdlib modules `M-510…M-514`, and `M-520` self-host the RFC-0013/0014 diagnostics+recovery)
  decomposes the **M-346** stdlib epic + the `milestones.json` Phase-5 charter; **Phase 6** (`M-601`
  native MLIR→LLVM full-calculus codegen, `M-602` native NFR-7 differential + E1 speedup, `M-610`
  BitNet/native-ternary acceleration, `M-620` deployable Spore units, `M-630` production hardening +
  the cross-backend VR-4 gate) traces to the Phase-6 charter + RFC-0004 §2 / ADR-009 / ADR-013 / M-348.
  Numbers are **minted on the next `gh-sync-all.sh` run** at each gate (the established M-364…M-368
  staging precedent; the MCP cannot create milestones/colored labels) — none fabricated here.
- **Verification.** `gh-issues-sync.py --validate` (111 issues at this point — 129 after the stdlib decomposition recorded in the entry above; phase 5/6 + idmap-drift notes **resolved**;
  only the reserved-and-intentionally-unused `good-first-issue`/`type:bug`/`type:chore` label notes remain,
  an honest residual) and `--self-test` both pass; `bash scripts/checks/all.sh` prints **ALL CHECKS
  PASSED**. **Manifests-only** change — no crate, no kernel, no `gh-issues-sync.py` engine touched (KC-3).
  The GitHub board reconcile (creating the Phase-5/6 issues + appending their `idmap.tsv` rows) is the
  maintainer's follow-up `gh`-capable step (unavailable in-session).

### Changed (2026-06-17: ratified `scope → area:*` aliases for the board reconciler — clears recurring PR FLAGs)
- **`tools/github/conventions.json` — `scope_to_area.aliases`** populated (was `{}`). The reconciler's
  `--prs` path maps a Conventional-Commit `type(scope): subject` title's `scope` to an `area:*` label
  only on an **exact** area match, else it FLAGs (G2 — never invents). Recurring repo scopes were
  surveyed from `origin/main` history and mapped to the canonical **WS-\*** areas: subsystem/crate
  scopes (`l1/grammar/surface → language`; `core/interp → core-ir` per WS-B "Core IR & reference
  interpreter"; `mlir/jit/runtime → execution`; `numerics/dense/bitnet/simd → numerics`;
  `select → selection`; `swap(s) → swap`; `vsa → vsa`; `lsp/fmt/lint/check/sec/spore/pack/build/xtask/
  tooling/diagnostics → toolchain` per WS-H which lists LSP), the **verified-numerics** family
  (`verification/cert/proofs → numerics` per WS-F "Verified numerics & checker"), and project
  infrastructure (`github/planning/tracker/skills/ci/changelog/workspace/proj/spec/review/kc2/phase-2/
  phase-3/notes/devlog/glossary/schemas/research/experiments/claude → project`).
- **Alias values are the BARE area name** (the engine prepends `area:` in `derive_pr_labels`), now noted
  in the file's `_policy`; the bare form is verified through the real engine function (`--self-test` +
  `--validate` both pass; the mapping was exercised directly, not just schema-checked).
- **Deliberately left UNMAPPED → still FLAGGED** (a decision, not a guess; deferred to a later pass):
  doc-reference scopes (`rfc-*/adr-*/dn-*`, `e1`, `l0`) and task-id scopes (`m-*`). Multi-scope comma
  titles map each recognized part. No new label, no taxonomy change — `area:*` set is unchanged (DRY).

### Added (2026-06-17: the toolchain gate's richer end-to-end conformance fixture — Phase 9 Wave D; M-369)
- **`examples/repr-tour/`** — a richer, multi-nodule canonical phylum (`mycelium-proj.toml` + four
  `.myc` nodules) authored to **pass all four M-361 gates**, so `just check` now proves the tools on
  **representative L1 programs**, not just the minimal `hello-phylum` toy. It tours: a **guarantee-
  annotated swap** (the LR-6 honesty index across the `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` lattice —
  `swaps.myc`), a **trait** + a **matured fn** (`matured ⟹ total`, RFC-0007 §4.5 — `traits.myc`), a
  **`for` fold** over a linearly-recursive value (RFC-0007 §4.8 — `iter.myc`), and **ambient
  representation** (RFC-0012: nodule-scope `default paradigm`, paradigm-less `{N}`, a `with paradigm`
  override whose inner swap stays explicit & never-silent — `ambient.myc`). Every nodule was
  canonicalized with `mycfmt --write` before commit.
- **`scripts/checks/myc-spore.sh` (+ `just myc-spore`)** — a **non-gating** packaging smoke that runs
  `spore build` (M-368, the 5th M-361 tool) over each real root and prints the **deterministic
  content-addressed digest** (blake3; metadata is not identity, ADR-003) as an honest receipt. It is
  added to `scripts/checks/all.sh` for dogfooding visibility but **always exits 0** — packaging is a
  build artifact, not a correctness property; a builder that cannot complete `skip`s with the reason
  (never a silent pass; G2/VR-5) rather than turning the suite red. The four pass/fail gates still own
  correctness.
- **Honest findings kept OUT of the gated examples** (each an open deferral, not a forced-green gate;
  G2/VR-5): the L1 `spore(…)` **expression** is deferred in the type-checker (E2-5/M-260), so it cannot
  pass `myc-check` and is exercised only via the non-gating `spore build` packaging path; and `mycfmt`
  v0 **refuses interior comments** (the §10.2 comment-preserving deferral, a Wave-C item), so the new
  nodules carry their prose in the structured `@summary` header rather than inline. **No kernel change,
  no new dependency** (KC-3). New fixtures live under `examples/` (real, gated, green roots) — NOT under
  `tests/fixtures/`/`reject/`, which stay must-fail and ungated (locked decision #3).
- **`M-369` filed** in `tools/github/issues.yaml` (append-only; no GitHub issue number minted yet —
  resolved at the next `gh-sync-all.sh` board reconcile, which needs a `gh`-capable run with project
  scope, unavailable in-session).

### Added (2026-06-17: the M-361 toolchain is wired into the CI-parity gate — Phase 9 Wave A; epic #132 done)
- **`examples/hello-phylum/`** — a minimal canonical phylum (one `mycelium-proj.toml` + two `.myc`
  nodules) authored to **pass all four M-361 gates**, so the suite runs **green-and-real**, not all-skips.
  Wave D later expands this into the full end-to-end conformance fixture.
- **Four new check scripts** — `scripts/checks/{myc-fmt,myc-check,myc-sec,myc-lint}.sh` — run the folded
  tools over the real project roots (dirs with a `mycelium-proj.toml`, discovered via the new
  `myc_roots` helper in `scripts/lib.sh`). They **exclude any `tests/fixtures/` path** (the
  intentionally-bad must-fail corpus incl. `bad-header.myc` / the `reject/` programs — running the tools
  there would erroneously turn the gate red; locked decision #3), `have cargo`-skip gracefully, and map a
  real finding to a **suite failure** (like `lint`/`test`).
- **Wired into the one source of truth:** the four are appended to `scripts/checks/all.sh` (after `test`),
  given `just` recipes (`myc-fmt`/`myc-check`/`myc-sec`/`myc-lint`), and added as `.pre-commit-config.yaml`
  local hooks (`files: \.myc$|mycelium-proj\.toml$`, `pass_filenames: false`) — so local == pre-commit == CI.
  `just check` now exercises `mycfmt --check`, `myc-check --project`, `myc-sec` (wild-audit), and
  `myc-lint --project`.
- **Honest scope per gate:** `myc-sec` runs the **wild-block audit** with `--no-secrets --no-supply-chain`
  (secrets + supply-chain keep their own dedicated `secrets`/`deny` gates; coverage is preserved at the
  suite level, FULL for the family myc-sec owns — skip ≠ pass, G2/VR-5); `myc-lint --fix` applies nothing
  in v0; `myc-check` stops at name-visibility (M-365 cross-phylum depth deferred); the §4.1 doc-quality
  lint stays dormant until the M-363 doc build (Wave B). **No kernel change, no new dependency** (KC-3).
- **M-361 (#132) closed `status:done`** in `tools/github/issues.yaml` — the epic's gate has landed.

### Added (2026-06-17: one idempotent, manifest-driven reconciler for the ENTIRE GitHub project state)
- **`tools/github/gh-issues-sync.py` is now the single cross-platform engine** for the whole project
  state — labels + milestones + issues **+ PRs + the Project v2 board** — pure Python + `gh` (no new
  dependency, KC-3; no bash, no jq). `--all` is the **full maintenance suite**: preflight → validate →
  labels → milestones → issues → PRs → project. Every level is **create-if-absent + update-to-match +
  `--dry-run` + never-silent (G2) + offline `--self-test`**; in-sync ⇒ zero writes, nothing duplicated.
- **`--prs` (new):** backfills **every** PR (`state=all`) — derives `type:*` (and `area:*` only on an
  exact scope match, else **FLAG**, never invented) from the **Conventional-Commit title** (fallback: the
  PR's commit messages), infers a milestone from referenced task-ids (unambiguous-only, else FLAG), and
  reconciles **add-only** (a human's labels are never stripped). New manifest **`conventions.json`** holds
  the `type(scope)` → label / milestone grammar (the maintainer's stated CC mapping + repo `spec/research/
  design` friends + an empty, declared scope→area alias table).
- **`--project` (new):** reconciles the **Mycelium** Project v2 board via `gh api graphql` —
  find-or-create, custom fields + single-select options, items added, and **Status/Phase/Area/Priority set
  from each item's labels** (idempotent). Views + built-in workflows are settings-only → **recorded in the
  new machine manifest `project.json` and FLAGGED as manual steps** (never silently skipped). The stale
  `project-v2-spec.md` is refreshed to phases 0–8 + the live `area:*` set and now points at `project.json`.
  This **replaces the manual "hand to Grok" step.** (Live GraphQL path is `--dry-run`/`--self-test`-checked;
  **Declared**, not yet Proven, until run on a `project`-scoped machine.)
- **Auto preflight + `--validate` (new):** a sanity check proceeds when `gh` auth/scopes are good and only
  prints the `gh auth refresh -s project` remediation when the **`project`** scope is **genuinely missing**
  (a good token is never asked to refresh; the board is skipped, never the whole run). `--validate` checks
  the manifests are **accurate to the codebase** (conventions/project/labels parity, idmap↔issues, changelog
  hygiene) and gates `--all`.
- **`tools/github/git-signing-sync.py` (new):** a portable (Linux + Windows), pure-Python **commit-signing**
  reconciler. **Default = read-only sanity check**; **`--setup`/`--init`** prompts for **name/email/comment**,
  **reuses** an existing key and **generates only when absent or when `--new-key` forces a rotation** — an
  existing key is **never replaced without `--new-key`**, git config is set on-drift, an existing SSH-signing
  setup is left untouched. `termux-setup.sh` now delegates its GPG + package steps to it (idempotent install;
  gated generation). New thin wrappers `git-signing-setup.{sh,ps1}`; `gh-sync-all.{sh,ps1}` route through the
  unified engine. `.github/ISSUE_TEMPLATE` labels + `PULL_REQUEST_TEMPLATE` aligned to the CC grammar.

### Added (2026-06-17: `myc-lint` — lint + auto-fix, folded — M-366; the M-361 suite is complete)
- **`crates/mycelium-lint`** — the `myc-lint` lint+fix tool (lib + CLI), enacting the M-366 contract
  (Accepted → enacted). Surfaces the M-141 invariant lints + the header lints as **actionable, reified,
  opt-in** fixes with a **suggest / apply / scaffold** boundary. A control-flow change (`implicit-swap` →
  an explicit `swap`; the RFC-0015 §9 advisory → an RFC-0014 recovery handler via `recovery_scaffold`,
  bounded `retry(<=3)`) is a **scaffold**, never auto-applied (A2/I1/I5; tested). **First-impl confirmation
  (§8.1):** no lint has a behaviour-preserving auto-fix that isn't already `mycfmt`'s header
  canonicalization, so **`--fix` applies nothing** in v0 and says so — no silent rewrite (G2). The **§4.1
  doc quality-bar lint is dormant-but-defined** (`DOC_QUALITY_CHECKS` names the 8 checks; awaits the M-363
  doc IR; does not block the gate). Honest deferrals: the §9 lint needs L1 effect declarations (v0 ships
  the scaffold generator, not the triggering lint); Core-IR lints run over the elaborable fragment (a
  non-elaborable definition is skipped, not silently passed). **No new dependency** (KC-3). CLI: `myc-lint
  [--project <dir>] [--fix] [--explain] [<file|->...]`.
- **The M-361 "full-fat toolchain" suite is now code:** all five children folded — **M-364** `mycfmt`,
  **M-368** `spore`, **M-365** `myc-check`, **M-367** `myc-sec`, **M-366** `myc-lint` — each above the
  kernel (KC-3), no new dependency, every contract Accepted/enacted.

### Added (2026-06-17: `myc-sec` — security checks as tooling, folded — M-367)
- **`crates/mycelium-sec`** — the `myc-sec` security tool (lib + CLI), enacting the M-367 contract
  (Accepted → enacted). v0's library core is the Mycelium-specific **`wild`-block audit** (`audit_wild` —
  a lexical recogniser over `.myc`, like the M-141 header lints): it inventories every `wild` block
  (LR-9/S6 — the denied-by-default unsafe escape hatch) and flags any without an adjacent **ADR-014
  `// SAFETY:`** justification (`wild-unjustified`, **medium** — fails only under `--strict`). Tested:
  justified passes, a `wild` in prose/an identifier is no false positive, a blank line breaks the
  justification block. The **skip ≠ pass** crux is enacted: the CLI orchestrates the existing
  `scripts/checks/{secrets,deny}.sh` gates and classifies each **ok / REDUCED / FAIL** (an absent scanner
  or a `skip` is *reduced coverage*, printed in a `FULL`/`REDUCED` coverage receipt — an OK with reduced
  coverage is **not** a clean bill; G2/VR-5). Every finding cites *why*; severity is a fixed declared map.
  **No new dependency** (std-only lib; the bin shells via `std::process`; KC-3). CLI: `myc-sec [--project
  <dir>] [--strict] [--explain] [--no-secrets] [--no-supply-chain]`.

### Added (2026-06-17: `myc-check` — the correctness driver, folded — M-365)
- **`crates/mycelium-check`** — the project-aware correctness/type-check driver (lib + `myc-check` CLI),
  enacting the M-365 contract (Accepted → enacted). The prototype **grew up in place**: the single-file
  **oracle** mode (the M-002/KC-2 harness contract — exit 2/3, `--expect-main`, `ok`/`parse-error:`/
  `check-error:`) is preserved verbatim, and a **`--project`/`--config` mode** added that walks the whole
  project, **aggregates** every refusal deterministically (all files), routes **check** refusals through
  the **M-362 baseline** at the umbrella `NotValidated` class (`Medium`/`stream`; additive-only — never
  suppressed, A1), and exits **2 parse / 3 check / 5 resolution / 0 clean** (CI-usable). Honest: the flat
  `CheckError` is **not** split into a finer class it cannot structurally distinguish (VR-5); a project
  with no `.myc` sources is an explicit exit-5 error, never a silent empty pass (G2). The trusted M-210
  checker (`check_nodule`) is unchanged — this is the driver above it (KC-3); **no new dependency**.
- The prototype `crates/mycelium-l1/src/bin/myc-check.rs` is **removed** (superseded; its oracle behavior
  ported into the driver — nothing references the old bin but a prose doc-comment).

### Added (2026-06-17: `spore` — packaging & publishing, folded — M-368)
- **`crates/mycelium-spore`** — the `spore` packager (lib + CLI), enacting the M-368 contract (Accepted →
  enacted; ADR-013). Builds a **content-addressed spore** from a `mycelium-proj.toml`: **identity is the
  DAG** (project kind + germination surface + source files by raw-byte BLAKE3 + dependency hash edges) and
  **metadata is excluded** (ADR-003) — a `version`/`authors` change leaves the spore id unchanged, a code
  or dep-hash change moves it (both tested). Never-silent publish inputs (G2): a phylum with no surface, no
  `.myc` sources, a **hashless dependency**, or an `[spore].include` naming a non-export is an explicit
  error (exit 3) — **no partial artifact**. `EXPLAIN`/`spore explain` prints the identity receipt + the
  not-identity metadata. CLI: `spore build` (`-o <out>`) / `spore explain` / `--config`. **No new
  dependency** (workspace-pinned `blake3` + `mycelium-core::ContentHash`; KC-3). v0: single project,
  hash-pinned deps, named-provisional descriptor encoding (R2 wire-schema/signing/germination deferred).
- **`crates/mycelium-proj`** — the manifest reader now **interprets `[surface]`/`[dependencies]`/`[spore]`**
  (typed, closed key sets; a non-inline-table dependency or unknown key is an explicit error — G2). `spore`
  is the first consumer of these accepted-but-uninterpreted M-359 tables. `Surface`/`Dependency`/
  `SporeConfig` exported.

### Added (2026-06-17: `mycfmt` — the canonical formatter, folded — M-364)
- **`crates/mycelium-fmt`** — the `mycfmt` formatter (lib + CLI), enacting the M-364 contract; the
  `Mycfmt-Formatter-Contract` moves **Accepted → enacted**. Formatting is an **identity-preserving
  projection** (RFC-0001 §4.6/§4.8; ADR-003): the body is re-printed from the **raw parse** (so
  `default paradigm`/`with paradigm` are preserved, not expanded — formatting ≠ expand-ambient), the
  DN-06 marker + M-359 `// @key:` header are re-emitted canonically, and a **runtime C1 guard**
  re-parses the output and refuses (never emits) anything that would change the surface AST or header.
  **C2 idempotence** + the corpus identity property are tested over `docs/spec/grammar/conformance/`
  (the whole `accept/` set formats in-scope; every `reject/` is refused). Never-silent (G2): parse
  (exit 2) / header (exit 3) / out-of-scope (exit 4 — incl. interior comments and the **hard-pin**
  `[toolchain].format` mismatch) refusals leave the file untouched; `--write` is atomic. CLI:
  stdout (default) · `--check` · `--write` · `--explain` (prints the identity receipt) · `--config`.
- **`crates/mycelium-proj`** — the manifest reader now **interprets `[toolchain]`** (`format`/`lints`;
  closed key set, unknown key = explicit error) — `mycfmt` is the first consumer of the
  accepted-but-uninterpreted M-359 table. `Toolchain` exported. No new dependency (KC-3).

### Changed (2026-06-17: M-364/365/366/367/368 open questions ratified — append-only)
- The maintainer ratified one open question per child contract (folded in append-only; all five stay
  **Proposed**, ready to fold):
  - **M-364** — `[toolchain].format` is a **hard pin** (refuse on version mismatch, exit 4; never format
    with rules the project didn't ask for — G2).
  - **M-365** — warnings **print but do not fail** the build by default; `--deny-warnings` is the opt-in
    CI gate.
  - **M-366** — `safe`-edit set is **conservative** (expressions/control flow → scaffold only; header
    canonicalization is the primary safe-edit); the §4.1 doc lint ships **dormant-but-defined** and does
    **not** block the gate. Held at Proposed a little longer — the safe-edit boundary + doc-lint dormancy
    get final confirmation at the first implementation pass.
  - **M-367** — a `wild` block is justified by the **ADR-014 `// SAFETY:` comment convention** for v0
    (no new structured attribute).
  - **M-368** — v0 may ship a **named-provisional on-disk encoding** (superseded append-only when the
    RFC-0008 R2 wire-schema lands).
  All other open questions across the five contracts remain deferred to the next wave / first
  implementation pass.

### Changed (2026-06-16: M-363 §8 build stack ratified — pipeline design Accepted)
- **`docs/spec/Narrative-Authoring-Pipeline.md` moves Proposed → Accepted** (append-only): the maintainer
  **ratified the §8 build stack** — a custom in-repo **doc-IR generator + Typst** (PDF/EPUB) + a static HTML
  renderer (§8.1a); **Typst** PDF engine (§8.2); **v0 single-version** (§8.3). §8.4 stands at recommendation
  (rustdoc JSON adapter); §8.5 (hosting) deferred. The §8 gate is lifted; the §8 options are retained
  verbatim for the record. This **unblocks M-366's §4.1 doc quality-bar lint** (now specifiable against the
  stack). **Building M-363 remains a separate, not-yet-scheduled task** — ratifying the design is not
  scheduling the build.

### Added (2026-06-16: M-361 child contracts — design, M-365/M-366/M-367/M-368)
- **Four design-first contracts for the remaining M-361 children** (each **Proposed**; present before
  folding; **no code, no new dependency**, all above the kernel — KC-3):
  - **`docs/spec/Myc-Check-Driver-Contract.md`** (M-365) — the project-aware correctness driver: deterministic
    project resolution (manifest `[surface]` + `[dependencies]` + M-359 header inheritance), whole-`phylum`
    **diagnostic aggregation** routed via the **M-362 auto-baseline** (additive-only A1, EXPLAIN-able),
    **honest per-op tags preserved** (VR-5 — never upgraded), CI exit semantics (non-zero on any error;
    opt-in `--deny-warnings`); the trusted M-210 checker unchanged.
  - **`docs/spec/Lint-and-Autofix-Contract.md`** (M-366) — lint+fix under one rule (**no silent rewrite**,
    G2): the M-141 lints + RFC-0013 diagnostics + the RFC-0015 §9 "only logged — add a handler?" advisory as
    **actionable, reified, opt-in** fixes with a bright **suggest / apply / scaffold** boundary (a
    control-flow change — an explicit `swap`, an RFC-0014 recovery handler — is a **scaffold**, never
    auto-applied; A2/I1/I5). Hosts the M-363 **§4.1 doc quality-bar lint** (8 checks), now unblocked by the
    §8 ratification (dormant-but-defined until the doc-IR generator lands).
  - **`docs/spec/Security-Checks-Contract.md`** (M-367) — security as tooling over `scripts/checks/{secrets,
    deny}.sh` (gitleaks · cargo-deny/audit) plus a new in-repo **`wild`-block audit** (LR-9/S6/DN-02 §5 —
    inventory every denied-by-default unsafe block + require an ADR-014 `// SAFETY:` justification). Honesty
    crux: every finding **cites why**, a fixed declared severity map, and a missing scanner is **reduced
    coverage, never a silent pass** (an OK with `REDUCED` coverage is not a clean bill — G2/VR-5).
  - **`docs/spec/Spore-Build-and-Publish-Contract.md`** (M-368) — `mycelium-proj.toml` → `spore` (ADR-013):
    the build pipeline, the **identity-vs-metadata** split (ADR-003 — same code+deps ⇒ same spore hash
    regardless of version/authors), **hash-authoritative dependency resolution** (a hashless/disagreeing dep
    is an explicit error), never-silent publish inputs (**no partial artifact**, G2), an `EXPLAIN` identity
    receipt; honest v0 scope (single-project, hash-pinned — the wire-schema/signing/germination contract
    deferred to RFC-0008 R2 per ADR-013 §4). First consumer of the M-359 `[surface]`/`[dependencies]`/
    `[spore]` tables.

### Added (2026-06-16: `mycfmt` formatter contract — design, M-364)
- **`docs/spec/Mycfmt-Formatter-Contract.md`** (**Proposed**) — the M-364 formatter contract, design-first
  (present before folding). Pins `mycfmt` (the standalone canonical formatter — M-142 grows up) as an
  **identity-preserving projection** (RFC-0001 §4.6/§4.8; ADR-003 — formatting never changes a definition's
  content-addressed identity) with three **checked** invariants: **C1** identity-preservation (the
  load-bearing one — an `EXPLAIN` *identity receipt* shows the content hash unchanged, and a run that
  cannot is a refusal, not a write), **C2** idempotence (byte-for-byte fixed point), **C3**
  header-preservation (the DN-06 `// nodule:` marker + the M-359 `// @key:` structured header, re-emitted
  canonically; a malformed header is an explicit error, never a silent drop — G2/VR-5). Defines the
  never-silent error model (parse/header/out-of-scope exits; **no partial or garbled rewrite**, G2), the
  hand-rolled CLI + exit codes (**no new dependency**), `[toolchain].format` reading (the M-359 table's
  first consumer), and the honest v0 **round-trip-safe scope boundary** — `mycfmt` formats only the fragment
  where `parse ∘ print ∘ parse` is the identity (checked on `grammar/conformance/accept/`) and **refuses**
  the rest (exit 4) rather than risk identity. Architecture: a new above-the-kernel `mycelium-fmt` crate
  over already-landed M-142/M-358/M-359 primitives (KC-3). No `mycfmt` code lands until the contract is
  acknowledged.

### Changed (2026-06-16: M-361 children created + wired — PM)
- **M-364…M-368 created on GitHub and wired as sub-issues of M-361 (#132)** via the staged
  `tools/github/issues.yaml` (gated `gh-sync-all.sh` run): **M-364** #136, **M-365** #137, **M-366** #138,
  **M-367** #139, **M-368** #140. `tools/github/idmap.tsv` appended (task → number → REST db-id). The
  Phase-8 milestone + `phase:8` label are assigned. No code (bookkeeping).

### Changed (2026-06-16: M-361 Phase-8 toolchain epic decomposed — staged, PM)
- **M-361 decomposed into five per-tool children** (the epic body's named tools), staged in
  `tools/github/issues.yaml` as sub-issues of M-361: **M-364** (`mycfmt` formatter — M-142 grows up),
  **M-365** (correctness/type-check driver — `myc-check` grows up), **M-366** (lint + auto-fix, incl. the
  RFC-0015 baseline "class only logged" lint + the M-363 §4.1 doc quality-bar lint), **M-367** (security
  checks as tooling — secrets/supply-chain/`wild`-audit), **M-368** (packaging/publishing:
  `mycelium-proj.toml` → spore, ADR-013). `manifest-check.py` passes (78 issues); MILESTONES + idmap note
  the gated-sync creation at the Phase-8 gate (the established staging → `gh-sync-all.sh` flow). No code.

### Added (2026-06-16: narrative & automated-authoring pipeline — design, M-363)
- **`docs/spec/Narrative-Authoring-Pipeline.md`** (**Proposed**) — the M-363 pipeline design (design-first;
  **ratify before building**): a **one content-addressed doc IR → many renderers** architecture
  (HTML/PDF/EPUB + machine JSON, so all formats share identity — ADR-003/G11, no drift); four projection
  generators (apiref/manual/book/blog) with their corpus sources; one reviewed template (the human gate
  for the fully-automated outputs); and the **§4.1 quality bar as a checkable 8-point lint** — single
  template, navigability, progressive disclosure (RFC-0013 levels), **checked examples** (a stale example
  fails the build — never-silent for docs, G2), no dead xrefs, dual-projection parity, **no hallucinated
  prose / undocumented-is-flagged**, legibility/accessibility. Placed in the M-361 toolchain (KC-3). The
  build stack + format/versioning choices are **flagged for ratification (§8); no pipeline code lands until
  ratified.**
- **`research/07-narrative-authoring-pipeline-RECORD.md`** — prior art (rustdoc/docs.rs, mdBook, Sphinx/MyST,
  Antora, literate programming, Pandoc/Typst, spec-generated manuals) traced as **T7.1–T7.7**, grounding the
  design (the no-drift, checked-examples, one-IR-many-renderers decisions).

### Added (2026-06-16: RFC-0015 automatic baseline diagnostics & recovery, M-362)
- **RFC-0015 ratified `Draft → Accepted`** and enacted. Prior art (DynEL, Rust `tracing`/`log`, Erlang/OTP,
  Python `logging`, structured-logging) traced into **`research/06-automatic-baseline-diagnostics-RECORD.md`**
  (findings **T6.1–T6.5**, discharging the §7 grounding obligation); the four §8 questions **resolved**.
- **`crates/mycelium-lsp/src/baseline.rs`** — the automation layer *over* RFC-0013 (presentation) +
  RFC-0014 (recovery), honest by construction (the §4.1 boundary A1–A4):
  - **`derive_baseline` / `derive_baseline_for`** — auto-derive a zero-config baseline `DiagnosticPolicy`
    from the error-class registry via a **total, inspectable closed `class → (level, route)` table**
    (`baseline_for_class`), optionally scoped per-definition by its **declared effect** classes. The result
    is presentation-only — structurally incapable of changing control flow (A1/I1) — content-addressed,
    and tagged `baseline`.
  - **`explain_baseline`** — the `EXPLAIN`: every class with its derived level/route + **rationale** (A3;
    "what baseline applied here, and why?").
  - **`recovery_profile`** + **`RecoveryProfile`** (`strict` / `resilient`) — the **closed, opt-in,
    bounded** recovery set (A2): `strict` propagates everything; `resilient` applies bounded `retry(≤3)`
    (`RESILIENT_MAX_ATTEMPTS`) to the **explicitly-supplied** classes only (RFC-0014 I4/I5). Recovery is
    **never** auto-applied — it is produced only on explicit request.
- **Honesty boundary, as tests:** A1 (a baseline can never suppress an error — `present` returns it
  unchanged), A2 (recovery bounded + opt-in), A3 (content-addressed + EXPLAIN-able), A4 (derivation is a
  total, deterministic function of the registry — every class covered). No new error mechanism; no kernel
  change (KC-3). `scripts/checks/all.sh` green.

### Added (2026-06-16: structured nodule header + project manifest, M-359)
- **`crates/mycelium-proj`** — the project-metadata layer (KC-3, above the kernel) enacting the
  *Nodule-Header-and-Project-Manifest* spec (**Accepted** 2026-06-16; the three §7 format choices ratified
  by the maintainer: header sigil `// @key: value`; the v0 key set extended with `repository`/`keywords`/
  `deprecated`; `@updated` author-maintained):
  - **`header`** — the structured nodule header parser: the `// @key: value` lines (closed 9-key v0 set)
    over the `// nodule:` marker (reuses M-358's `parse_nodule_header`). An **unknown** key, a
    **duplicate** key, or a **malformed** value (non-SPDX `@license`, non-ISO `@since`/`@updated`,
    ill-formed `@version`, non-URL `@repository`) is an **explicit** error, never silently ignored or
    guessed (G2 / VR-5 — checked, never fabricated).
  - **`manifest`** — `mycelium-proj.toml`, read by a **minimal, no-new-dependency TOML-subset** reader
    (the workspace keeps its deps few/vetted; adding a full TOML crate would be an ADR). It is honestly a
    subset — strings/arrays/inline-tables/booleans, single-line values — and an out-of-subset construct is
    an explicit error (G2). The closed `[project]` table is typed + validated; optional tables are accepted
    but not yet interpreted (M-361).
  - **`resolve`** — top-down inheritance (`in-file > manifest`) with **per-field provenance** and an
    **`EXPLAIN`**, so a field's effective value *and source* are never ambient (G2). A local value
    overrides the manifest (an allowed override, not a conflict; spec §4).
- **`mycelium-lsp::lint_structured_header`** (M-141) surfaces a malformed header as a `Diagnostic`.
- **Schemas** `docs/spec/schemas/{nodule-header,mycelium-proj}.schema.json` + valid/invalid examples
  (the SPDX-membership and calendar-date-range checks live in code, recorded in each schema's
  `x-mycelium.$comment` per the schemas-README rule). End-to-end conformance fixtures in
  `crates/mycelium-proj/tests/`.
- **Honesty/identity:** metadata is **not** identity — nothing here perturbs a content hash (ADR-003).
  No kernel change (KC-3). `scripts/checks/all.sh` green (incl. the JSON-schema gate).

### Changed (2026-06-16: DN-06 lexicon migration — static keyword `colony` → `nodule`, M-358)
- **The L1 surface keyword `colony` is now `nodule`** (DN-06, Resolved 2026-06-16) — a pure, mechanical
  rename across the lexer/token/parser/AST/checker/elaborator (`crates/mycelium-l1`), the LSP toolchain
  surface (`crates/mycelium-lsp`), the normative grammar oracle (`docs/spec/grammar/mycelium.ebnf` +
  README), and the **full accept/reject conformance corpus** (the `01-minimal-*`/`01-no-*-header` fixtures
  renamed accordingly). **No semantic change**: content-addressed identity is computed over elaborated L0,
  never the surface keyword or a Rust type name (ADR-003), so every definition's content hash is unchanged.
- **`phylum` and `colony` are now reserved-not-active keywords.** `phylum` (the library-scale grouping
  above nodules) and `colony` (reassigned to the RFC-0008 §4.7 **dynamic** runtime grouping of `hypha`)
  lex as keywords — so they can never be silent identifiers — but no L1 construct consumes them yet, so
  neither opens a program (new `conformance/reject/10-reserved-not-active.myc`; G2).
- **The `// nodule:` header marker (DN-06 §6) is wired in.** New `mycelium_l1::parse_nodule_header`
  recognises the first-non-blank-line marker (`// nodule: <dotted.name>` or bare `// nodule`); a near-miss
  *named* marker (empty/ill-formed name) is an **explicit** error, never silently dropped (G2). The M-141
  linter surfaces a malformed marker (`lint_nodule_header`) and the M-142 surface formatter preserves a
  valid one across a canonical re-print. The structured `// @key:` header + `mycelium-proj.toml` manifest
  layer on top of this (M-359).
- **Honesty/grounding:** DN-02 §2's `colony = module` line stays superseded by DN-06 (append-only); the
  Glossary, Lexicon-Reference, grammar README, and DN-06 changelog are updated to record execution.
  `scripts/checks/all.sh` green (incl. the conformance gate).

### Added (2026-06-16: typed SPSC channels — the RT2 communicating fragment, M-357 follow-on)
- **`crates/mycelium-mlir/src/channel.rs`** — the Kahn-deterministic *communicating* half of the RFC-0008
  RT2 fragment (§4.3), extending the landed fork/join runtime. **Typed single-producer/single-consumer
  channels**: `Network::channel` returns an affine `Sender`/`Receiver` pair (neither `Clone` — SPSC by
  construction, RT1) over a buffer of **explicit, finite** capacity (`NonZeroUsize` — no unbounded silent
  buffer, RT7's spirit on queues). **Demand-signalled backpressure**: `try_send` on a full buffer returns
  `Full(v)` handing the value back (never dropped); the producer yields and is re-polled as the consumer
  drains. **Explicit close**: dropping the `Sender` lets the `Receiver` drain then see `Closed`
  (end-of-stream, never a hang); a send to a hung-up receiver is `Disconnected(v)` (G2, never a silent
  drop). A new **`Scope::run_dataflow(order, progress)`** (in `runtime.rs`) schedules communicating tasks
  and surfaces a stalled network as an explicit **`Deadlock { parked }`** — never a silent hang (the
  cooperative scheduler cannot block). Determinism is verified by a **Kahn-determinism differential**: the
  same network under two distinct fair schedules (`SweepOrder::Ascending`/`Descending`) yields identical
  outcomes + transcripts (T4.1) — tagged **`Empirical`** (the differential is the evidence) with Kahn T4.1
  cited, **not** `Proven` (no mechanized proof in-repo; VR-5). Deferred (honest boundary): multi-source
  `select`/`merge` (RT3), session/protocol typing beyond the §4.3 hook, zero-capacity rendezvous,
  `xloc`/`mesh` (R2). No kernel change (KC-3); no `unsafe`. RFC-0008 §4.6 staging note + Meta-changelog
  updated (append-only). `just check` green.

### Fixed (2026-06-16: PM manifest drift — labels.json out of sync with issues.yaml)
- **`tools/github/labels.json`** was missing three labels that `issues.yaml` already uses —
  **`type:design`** (12 issues), **`priority:P3`** (11 issues), and **`area:language`** (1). Because
  `gh issue create --label <name>` errors on a label the bootstrap never created, this silently stalled
  issue creation: the five staged Phase-7/8 issues (**M-358/359/361/362/363**) were not created on the
  prior run. Added the three labels (matching the existing color/description style) so a sync run creates
  them first, then the issues that reference them.

### Added (2026-06-16: one-command PM gap-closer + manifest preflight)
- **`tools/github/gh-sync-all.sh`** — a single **idempotent** command that reconciles the repo with the
  manifests in one pass: a preflight, then `gh-bootstrap-local.sh` (labels + milestones), then
  `gh-issues-sync.py` (create absent issues + assign milestones + append `idmap.tsv`). Safe to rerun any
  time `issues.yaml`/`labels.json`/`milestones.json` gains entries; nothing is duplicated. Supports
  `--dry-run` (preview issue creation, no repo writes).
- **`tools/github/manifest-check.py`** — the preflight: every label/milestone `issues.yaml` references
  must be **defined** in `labels.json`/`milestones.json`, else an explicit fail-fast error (the
  never-silent rule, G2 — a missing label can no longer silently leave issues uncreated). Reverse drift
  (a defined-but-unused manifest entry) is an advisory note only.
- Docs updated to make `gh-sync-all.sh` the canonical re-sync entrypoint: `MILESTONES.md`,
  `mcp-bootstrap.md`, `termux-bootstrap.md`. The two component scripts stay single-purpose.

### Added (2026-06-16: mobile/Termux GitHub bootstrap — phone-autonomous PM)
- **`tools/github/termux-setup.sh`** + **`tools/github/gh-issues-sync.py`** + **`termux-bootstrap.md`**.
  A single, ordered, **idempotent** path to run the *whole* GitHub project-management bootstrap from an
  Android phone (Termux) with nothing pre-configured: installs packages from the package manager (no
  `curl | bash`), sets the git identity, generates a passphrase-protected **GPG signing key** and uploads
  only the **public** key, authenticates `gh` (browser/device OAuth or a supplied token, held by `gh` —
  never committed), then chains `gh-bootstrap-local.sh` (labels + milestones) into the new
  `gh-issues-sync.py`. The Python helper is the **gh-driven local analogue of `mcp-bootstrap.md` Steps
  1–2** — it closes the one gap that previously needed a model+MCP session (issue *creation*): snapshot
  issues by title, create only the absent ones with labels, assign milestones by title, and **append**
  (never rewrite) new `task_id → number → db_id` rows to `idmap.tsv`. Honesty-aligned: never-silent (every
  step announced; conflicts/missing milestones are explicit), no black boxes, no secrets in the repo
  (private GPG key stays on-device; token in `gh` config; credential helper, not token-in-URL). Scope
  boundary matches `gh-bootstrap-local.sh`: dependency/sub-issue linking (Step 4) still needs an
  MCP/GraphQL pass. `shellcheck`/`ruff` clean.

### Added (2026-06-16: narrative capture + automated-authoring intent, initial capture)
- **`docs/notes/Narrative-Capture-and-Authoring.md` (Living)** + the seeded **`docs/devlog/`** append-only
  narrative layer. Captures the maintainer's intent to record enough development narrative — decisions,
  **struggles, problems solved, the how and why** — to enable **partially-to-fully automated** authoring
  of project **blog** posts, a **language book**, and a **reference manual**, distributed **free** in
  digital formats. Notes that the honesty rule already makes the corpus a grounded, cited, append-only
  narrative (~80% of the raw material); the one gap (the struggle / problem-solving *how*) is filled by a
  lightweight `docs/devlog/` (first entry: `2026-06-16-rfc0008-integration-wave.md`, a worked example).
  All three outputs are **synthesis from the cited corpus** under the same discipline as the language —
  grounded/cited (no hallucination), projection-not-parallel-truth (no drift — ADR-003),
  human-in-the-loop, append-only provenance. Full pipeline design + tooling is a fresh session, tracked
  **M-363** (Phase 8). Registered in `Doc-Index.md`.
- **Added (future-planning):** a fourth output — **fully-automated documentation + API reference** (the
  most automatable: pure projection from code + schemas + the M-359 nodule-header metadata; rustdoc-first,
  Mycelium-lang doc-comments later; shipped free + served live/LSP-hover) — and a **format quality bar**
  (note §4.1): "clean · presentable · legible · intelligible · digestible" made a **checkable** contract
  (one consistent template; index→detail navigation; progressive-disclosure graded depth reusing
  RFC-0013's levels; checked inline examples; dual human/machine projection — G11; legibility/accessibility
  by construction; **undocumented is flagged, never invented** — the doc analogue of never-silent G2).

### Added (RFC-0015 — 2026-06-16: Automatic Baseline Diagnostics & Recovery, Draft)
- **RFC-0015 (Draft, Proposed)** captures the DynEL **automated-baseline** design point the maintainer
  added to the roadmap: an **automation layer over RFC-0013/0014** that auto-derives a zero-config
  **baseline** diagnostic/logging policy from the language's structured mapping (registry + routes +
  declared effects), **auto-applies** it (wrapping for logging/QoL), and offers a ladder of *light*
  overrides → *fully manual*. The load-bearing **honesty boundary** is fixed up front: automatic =
  **additive presentation/logging only** (safe because RFC-0013 never changes control flow — I1);
  **automatic recovery is opt-in, declared, bounded** (no implicit control-flow change — RFC-0014
  I3/I4/I5); the baseline is a **reified, `EXPLAIN`-able** policy (no black box — SC-3); the derivation is
  a **total, inspectable** function of the mapping, not learned (VR-5/RFC-0005). Tooling-layer; no kernel
  change (KC-3). Forward-pointed from RFC-0013 §9 + RFC-0014 §9; registered in `Doc-Index.md`; tracked
  **M-362**. **No code** — design point only.

### Changed (DN-06 — 2026-06-16: static-organization & dynamic-grouping lexicon — `phylum` / `nodule` / `colony`)
- **DN-06 ratified** (maintainer-directed), introducing on-brand terms for static organization and
  deconflicting a real collision: **`phylum`** (content-addressed **library-scale** unit) and
  **`nodule`** (the **basic** static unit, replacing the generic "module") for static organization, and
  **`colony`** reassigned to the **dynamic** runtime grouping of active `hypha` (RFC-0008 §4.7). The
  reassignment **supersedes DN-02 §2's `colony` = module** line (append-only — DN-02's changelog records
  it; `phylum`/`nodule` had no prior use, so only `colony` collided). Justified by the DN-02 three-test
  gate: `colony` on a *living, supervised grouping of tasks* is a higher-fidelity T-map than on a static
  file, and `nodule` beats the generic "module" for the static unit.
- **Supplement (DN-06 §6 resolved):** a `nodule` is declared by a **header comment**
  (`// nodule: <name>`, or bare `// nodule`) on the first non-blank line — **not** in the filename/path
  (paths stay conventional; no `nodule` bloat). RFCs/docs use `nodule` for "module" going forward. A
  **dedicated `docs/Glossary.md`** is created — a summarized **Index** over a detailed **Glossary**
  (the fungal lexicon + honesty/architecture concepts), each entry citing its normative source, maintained
  separately from the RFCs (registered in `Doc-Index.md`). The header-comment convention folds into M-358.
- **Proposed — structured nodule header + `mycelium-proj.toml` manifest (`docs/spec/Nodule-Header-and-Project-Manifest.md`).**
  At the maintainer's preference for a *structured* header carrying useful metadata (license, authors,
  first/last dates, version) on a nodule/phylum **root**, with **subnodules inheriting** top-down: a
  closed-key in-file header (`// @key: value`), a `mycelium-proj.toml` manifest (the pyproject/Cargo analogue,
  scoped for Mycelium), and explicit `EXPLAIN`-able inheritance (in-file → nodule-root → `mycelium-proj.toml`).
  Honesty-aligned: **metadata is not identity** (the content hash stays canonical — ADR-003), no ambient
  metadata (unknown keys/conflicts are explicit errors — G2), declared-only license/version (VR-5),
  tooling-layer (KC-3). **Proposed** — the format choices (§7) are flagged for sign-off; no code lands
  until ratified. Records the long-term **full-fat toolchain** as the new anticipated **Phase 8** (epic
  **M-361**); the schema's enactment is **M-359**.
- **Adopted going forward:** the RFC-0008 §4.7 structured scope is realized as `mycelium-mlir::runtime`'s
  **`Colony`** (alias of the structured `Scope`). The **surface keyword migration** `colony` → `nodule`
  (the L1 lexer/parser/AST/checker — ~226 refs — plus the grammar EBNF + LR(1) oracle + the 23-file
  conformance corpus) is a pure rename + two reserved additions (`phylum`/`colony`), tracked as **M-358**
  and staged (the grammar contract moves in one auditable change). Until executed, `colony` is the
  deprecated spelling of `nodule`. RFC-0006 + RFC-0008 carry append-only forward-references; `phylum`
  and `colony` are reserved-not-active until their constructs land.

### Changed (RFC-0008 — 2026-06-16: Runtime & Concurrency Execution Model ratified `Draft → Accepted`)
- **RFC-0008 ratified `Draft → Accepted`** (maintainer): the seven runtime invariants **RT1–RT7** and
  the §4 model are now **normative** (the Runtime-tier grounding ADR-012 §7.3 required). Ratification
  opens the runtime track in staged slices: the **budget-unification slice** (RFC-0014 §4.8 — M-353,
  below) and the **route → observability-sink** binding (RFC-0013 §8 — M-354) needed no RT1–RT7
  commitment and proceed first; the **concurrency/supervision** track (RFC-0014 single-task boundary
  lifted — per-task budgets, cancellation, cross-task propagation, `reclaim` bounded cascades; RT4/RT7 —
  M-355/M-356) is the §4.7 revision, presented frozen-spec before folding. The §4.5 runtime vocabulary
  stays **reserved, not active syntax** until the implementation RFCs land.

### Added (RFC-0008 R1 — 2026-06-16: M-357 v0 / deterministic fork/join executor + RT2 differential)
- **M-357 (v0 slice) — the RT2 deterministic fork/join runtime over the §4.7 primitives.** The
  maintainer-chosen minimal scope (fork/join + the differential; typed channels deferred to the next
  slice): `crates/mycelium-mlir/src/runtime.rs` — a structured-concurrency `Scope` (RT7: every child is
  **joined**, none orphaned) over cooperative `Task`s, each carrying its **own** `Budgets` ledger and the
  shared `CancelToken` (M-356 C1/C2). Two strategies — `run_sequential` (the reference) and a
  deterministic `run_interleaved` round-robin — that the RT2 guarantee makes observationally equal over
  **pure** tasks (RT1). The scheduler lives **outside** the kernel (RT2; the trusted evaluator stays
  sequential — KC-3).
- **Verified** (module tests): the **RT2 sequentialization differential** — `run_interleaved` ≡
  `run_sequential` over a counter corpus (with an interleave trace proving the schedules genuinely
  differ) **and over the real env-machine** (tasks running `run_core_with_effects` on `bit.not` L0
  programs; each scheduled outcome equals the standalone `run_core` evaluation — no new meaning,
  NFR-7/KC-3); **RT7** scope-cancellation (cancelling the scope → every pending child resolves to an
  explicit additive `Cancelled`, all joined, none leaked); and **C1** per-task budget isolation (one
  task overrunning its `alloc` budget never exhausts a sibling's). `just check` green. The next R1 slice
  is typed SPSC **channels** (the Kahn-deterministic communicating half). **M-357 (#122)**.

### Added (RFC-0008 §4.7 — 2026-06-16: M-356 / concurrency composition primitives, single-task boundary lifted)
- **M-356 — RFC-0014's single-task boundary lifted onto RFC-0008 (§4.7 added; §8 concurrency deferral
  resolved).** A **frozen-spec** (presented before folding): RFC-0008 **§4.7** specifies four
  compositions, each additive over the explicit error (I1) and declared + bounded (I3/I4) — **(C1)**
  per-task budgets (each task instances its own M-353 ledger; an overrun is an *in-that-task*
  `EvalError::EffectBudget`, never global); **(C2)** cooperative, **additive** cancellation observed at
  budget-check points (an explicit `Cancelled`, never preemptive; scope-tree propagation, RT7);
  **(C3)** cross-task failure propagation via an explicit `TaskOutcome` with **no silent/dropped
  variant** (I1 across the task boundary, RT4); **(C4)** `reclaim` **bounded-cascade** supervision
  bounded on **both** a total `cascade` effect budget (M-353) **and** a windowed max-restart-intensity
  over a **logical clock** (Erlang/OTP, Research Record 05 T5.3; wall-clock deferred to R8-Q3) —
  exceeding either an explicit escalation, never a storm.
- **Enacted** as **scheduler-independent** primitives in `mycelium_interp::supervise`
  (`CancelToken` / `TaskOutcome` / `RestartIntensity` / `Supervisor` / `Escalation`) — **no L0 node**,
  the trusted base stays sequential (RT2; KC-3) — verified there and composed with the recovery driver
  in `crates/mycelium-lsp/tests/recover.rs` (cancellation is explicit + additive; a task failure
  propagates explicitly; a supervised restart storm is bounded on both axes; a per-task budget overrun
  is an in-that-task refusal). The actual **task scheduler/executor and the RT2 sequentialization
  differential are explicitly *not* here** — they are RFC-0008 R1 (**M-357**), built on these
  primitives. `just check` green. Advances G2, VR-5, SC-3. RFC-0014 §8 concurrency deferral **resolved**.
  **M-356 (#121)**.

### Added (Phase 4 — 2026-06-16: M-354 / RFC-0013 §8 diagnostic routes ↔ RFC-0008 observability sinks)
- **M-354 — the diagnostic `route` set closed and bound to RFC-0008 sinks (RFC-0013 §8 resolved).** A
  **closed v0 route vocabulary** — `stream` / `audit` / `log` / `null` / `mesh` — in
  `crates/mycelium-lsp/src/diagnostics/sink.rs`, each bound to an `rfc0008.*` observability sink with an
  **honest delivery guarantee** on the lattice (RT5): `stream` (in-process synchronous), `audit`
  (durable), and `log` (best-effort) are `Declared`; **`null` honestly reports *not delivered*** (never
  a "fire and forget" claimed reliable); **`mesh` is probabilistic**, carrying a declared
  `ProbabilityBound` δ (upgraded to Empirical/Proven only with a checked convergence basis — VR-5/T4.2).
  Route resolution is **checked** against the closed set (the §4.5 X1 "looked up, never evaluated"
  discipline applied to routes — an out-of-set route is an explicit `UnknownRoute`, never a silent
  misroute) and lives **outside** `present` (`DiagnosticRecord::sink` is the dispatch point), so routing
  — or a failed resolution — **never gates propagation** (I1). A typed `Rule::route_to(Route)` setter is
  the checked path; the free-form `route(String)` remains the on-the-wire projection. Tooling layer only;
  **no kernel logging dependency** (KC-3).
- **Verified** by `crates/mycelium-lsp/tests/diagnostics.rs`: **never-silent across every closed route**
  (I1 re-run per route — the error still propagates, each route resolves to its sink), **honest sink
  guarantees** (no sink over-claims `Declared`; the null sink does not deliver; the mesh sink carries a
  well-formed δ — RT5/VR-5), and an **explicit unknown route** (an out-of-set route string surfaces
  `UnknownRoute` without gating propagation). `just check` green. Completes the RFC-0013 §8
  route-targets/observability deferral; advances NFR-2/SC-5b. **M-354 (#119)**.

### Added (Phase 4 — 2026-06-16: M-353 / RFC-0014 §4.8 effect-budget unification, enacted)
- **M-353 — effect budgets unified with the runtime's fuel/depth clocks (RFC-0014 §4.8 completed).** The
  recovery `Budgets` ledger — previously a tooling-only reified mechanism — is **lifted into
  `mycelium-interp`** (`mycelium_interp::budget`: `EffectKind`/`EffectBudget`/`EffectBudgetExhausted`/
  `Budgets`), the **shared budget-resolution surface** both the AOT env-machine (`mycelium-mlir`) and the
  recovery driver (`mycelium-lsp`) depend on — placed to avoid a crate cycle and to sit where the fuel
  clock already lives (**no kernel change** — KC-3, **no** new L0 node, **no** kernel hook). An effect
  overrun now routes through **`mycelium_interp::EvalError::EffectBudget`** — the effect sibling of
  `FuelExhausted` (time) / `DepthLimit` (space) on the **one runtime refusal channel** (the ratified §8
  disposition: *separate named budgets, one enforcement mechanism*): a budgeted effect overruns
  **gracefully at runtime exactly as a runaway recursion does**, never a hang/OOM (I4). The env-machine
  threads the same ledger (`run_core_with_effects`) and charges a declared **`alloc`** budget per
  control-stack frame — the **opt-in** sibling of the DN-05 depth ceiling (same per-frame-bytes basis);
  an absent budget (the default) leaves behaviour identical (I5). `recover::effect` re-exports the moved
  types (RFC-0014's enacted API is unchanged) and keeps the *checker* half (`check_effects`/
  `UndeclaredEffect` — I3) in the tooling layer.
- **Verified:** the **bounded-overrun-is-explicit test extended to the runtime path** (`mycelium-mlir`:
  `a_declared_alloc_effect_budget_overruns_gracefully_at_runtime` → `EvalError::EffectBudget`, and
  `an_absent_alloc_budget_leaves_runtime_behaviour_unchanged`), plus a **meaning-preserving three-way
  differential** where it touches L0 (`mycelium-l1`:
  `the_effect_ledger_is_meaning_preserving_on_the_recovery_match` — threading an ample ledger is
  observable-transparent on the recovery `Match`; NFR-7). `just check` green. Completes the RFC-0014 §4.8
  deferral; advances G2, VR-5, SC-3. **M-353 (#118)**.

### Added (Phase 4 — 2026-06-16: M-352 / RFC-0014 declarative recovery & bounded effects, accepted + enacted)
- **RFC-0014 ratified `Draft → Accepted`** (maintainer; all §8 dispositions normative) and **M-352
  enacted** as a **separable, tooling-layer** subsystem in `crates/mycelium-lsp/src/recover` (**no kernel
  change** — KC-3, zero new L0 nodes; no Python, ADR-007). Three pillars: **errors-as-propagating-values**
  (`Outcome` over a `StructuredError` whose class is registry-resolved — shares RFC-0013's registry, X1);
  **explicit declarative recovery** — the never-silent `handle` applies a reified
  `on <ErrorClass> => <action>` policy (RFC-0005 pattern; content-addressed `PolicyRef`; closed action set
  `fallback`/`retry`/`escalate`/`cleanup_then_propagate`) and yields a `Resolution` that is **always**
  *recovered* or *re-propagated* — there is no "dropped" variant (I1 enforced by the type); and
  **declared, bounded effects** (`EffectKind` set, per-kind `EffectBudget`, the `Budgets` ledger whose
  overrun is a graceful `EffectBudgetExhausted` — I4, and a compositional `check_effects` no-undeclared
  -effect check — I3). A substituted fallback is honestly `Declared`, never upgraded (I2/VR-5).
- **Verified** by `crates/mycelium-lsp/tests/recover.rs` (RFC-0014 §5): the central **never-silent
  recovery invariant** (every action leaves the error recovered or propagated, never dropped — I1), the
  **bounded-overrun-is-explicit** test (`EffectBudgetExhausted`, never a hang/OOM — I4), the **opt-in
  default-scope** test (an undeclared effect can't run — I5), the **no-undeclared-effect** test (I3), the
  **honest-guarantee** test (I2/VR-5), and the shared-registry / no-`eval` discipline (X1). The
  **L0-`Match`-over-error-sums lowering target** — "recovery adds no new kernel node" — is differentially
  verified in `mycelium-l1` (`recovery_match_over_a_result_sum_agrees_three_ways`: L1-eval ≡ L0-interp ≡
  AOT; NFR-7). **Out of v0 scope (honest boundary):** wiring the `Budgets` ledger into the AOT
  env-machine's runtime budget resolver is the RFC-0008 integration (§4.8). `just check` green. Advances
  SC-3, G2, VR-5, NFR-2/SC-5b. RFC-0014 status → **Accepted — Enacted**; **M-352 (#116)** closed.

### Added (Phase 4 — 2026-06-16: M-345 / RFC-0013 structured diagnostics, enacted)
- **M-345 — RFC-0013 structured diagnostics & reified error policy: enacted** in
  `crates/mycelium-lsp/src/diagnostics` (tooling layer; **no kernel change**, KC-3; no Python, ADR-007).
  Four parts: the **error-class registry** (names looked up, **never `eval`-ed** — §4.5 X1; v0 classes
  from the existing lint codes + `SwapError` family + `NotValidated`); the **content-addressed
  diagnostic record** with a BLAKE3 `content_id` and a **dual human + JSON projection** that round-trips
  (G11, §4.3), graded `minimal`/`medium`/`detailed` **levels** with an **allowlisted** detailed tier
  (§4.5 X2), and the never-silent **`present`** renderer that returns the explicit error **unchanged**
  alongside the presentation (§4.1 I1); the reified **`on <ErrorClass> => {message, tags, level,
  route}` policy** (RFC-0005 pattern; content-addressed `PolicyRef`; presentation/routing only — I4),
  with a `PolicyFile` projection that re-validates classes through the registry (file-as-projection,
  §4.7); and the **representation-crossing audit view** (§4.6; routed from RFC-0012 R12-Q2) — every
  `swap` + from/to repr + honesty bound **read off the certificate and never upgraded** (VR-5),
  location-independent (I5).
- **Verified** by `crates/mycelium-lsp/tests/diagnostics.rs` (RFC-0013 §5): the central **never-silent
  invariant** (a battery of policies — routed / message-override / minimal-level / unrelated — all leave
  the error propagating; I1/I2/I4), round-trip projection (I3), registry / no-`eval` (X1, incl.
  whole-file rejection on an unknown class), the detailed-tier allowlist (X2, a secret-bearing field
  never reaches the record or its rendering), and the audit view (I5/VR-5, incl. an underivable crossing
  reporting `unknown`, never `Exact`). `just check` green. Advances NFR-2 / SC-5b and the M-330 AI
  co-author loop. RFC-0013 status → **Accepted — Enacted**.

### Changed (Phase 4 — 2026-06-16: ratifications, RFC-0014 design decisions, M-343 totality completion)
- **RFC-0013 — Structured Diagnostics & Reified Error Policy: `Draft (Proposed) → Accepted`** (maintainer
  sign-off). No design content changed on acceptance; the §4 invariants I1–I5 and the §4.5 exclusions
  X1–X3 are now normative. Unblocks the **M-345** Rust tooling-layer build (`mycelium-lsp`/`xtask`; no
  kernel change). Verified by the central never-silent invariant test (I1/I2/I4) + round-trip / registry /
  allowlist / audit-view tests.
- **RFC-0014 — remaining §8 questions given proposed v0 dispositions** (maintainer sign-off pending; RFC
  stays Draft, no code yet): effect inference = *manual-declare + compositional-check* (caller must
  declare a superset of callee effects — `UndeclaredEffect` otherwise — but the checker never infers an
  undeclared effect); recovery-action set = the *closed* v0 set
  `fallback`/`retry`/`escalate`/`cleanup_then_propagate` (each never-silent + bounded; user actions a §9
  future inheriting I1/I3/I4); concurrency = *deferred to RFC-0008* with a single-task v0 boundary fixed
  now (per-evaluation budgets, no cross-task cascade — deferral is safe); handler composition = *lexical
  innermost-first* (unmatched re-propagates, never drops), handler effects declared + budgeted like any
  code, cascades bounded by `cascade(max_depth)`. With the §7 prior-art tracing done, RFC-0014 is **ready
  for a Draft→Accepted decision**.
- **RFC-0014 — three §8 design questions resolved** (maintainer; RFC stays Draft): effect mechanism =
  **declared annotations, coarse set** (capabilities/effect-rows additive futures only); **no
  kernel-visible hook** — effect-budget enforcement is entirely runtime/checker, **zero new L0 nodes**
  (KC-3); **separate named budgets over one enforcement mechanism** — each effect kind keeps its own
  `EXPLAIN`-able budget, all resolved/enforced by the existing DN-05 plumbing that already clocks `Fix`/
  `FixGroup` fuel and the M-347 depth ceiling (composed alongside, not collapsed). No code until Accepted.
- **RFC-0014 prior art traced into `research/`** — new **Research Record 05** (T5.1–T5.6) grounds
  Result/`?`, algebraic effects (Koka/Eff/OCaml 5), **Erlang/OTP bounded supervision** (verified:
  max-restart-intensity, defaults 1/5s), structured-concurrency cancellation, capabilities, and Mycelium's
  own fuel/depth/DN-05 budget idiom — discharging the §7/§8 grounding obligation (honest deltas + novelty
  flags recorded). RFC-0014 §7/§8 + status line updated to reflect the resolutions and the tracing.
- **M-343 — mutual-descent totality classification (R7-Q3 loose end closed).** The `FixGroup` elaboration +
  three-way differential had landed, but the structural totality checker still classified *every* mutual
  group `Partial`. Extends `crates/mycelium-l1::totality` from self-descent to **mutual structural descent**
  over a call-graph SCC: a group is `Total` iff a per-member designated argument position descends on every
  inter-member call (one well-founded measure; bounded position-assignment search). Sound — only adds
  justified `Total` verdicts; gates `matured`, never meaning (G2; runtime stays fuel-clocked). RFC-0007 §4.5
  revised (append-only); ping/pong now `Total`, a non-productive cycle stays `Partial`.

### Added (Phase 4 — RFC-0014: declarative error recovery & bounded effects, drafted)
- **RFC-0014 — Declarative Error Recovery & Bounded Effects (Draft (Proposed)).** Designs the isolated
  recovery subsystem RFC-0013 §8/§9 deferred (the DN04-Q1 recovery half) — a way for errors to **bubble**
  and **trigger functionality** (fallback, retry, cleanup, escalation), as a **separable** subsystem with
  a bounded blast radius. Three pillars: **errors-as-propagating-values** (the RFC-0001 substrate, G2);
  **explicit declarative recovery** (an explicit handling site that elaborates to L0 `Match` — **KC-3, no
  new kernel node** — plus a reified RFC-0005-pattern `on <ErrorClass> => <action>` recovery policy); and
  **declared, bounded effects** (effects named on signatures so there are no unknown side effects; every
  unbounded effect carries an explicit budget and overruns *gracefully* as `EffectBudgetExhausted` — the
  direct generalisation of the `Fix`/`FixGroup` fuel clock, the M-347 depth ceiling, and DN-05 budgets).
- **Records the maintainer's governing discipline:** effects and even cascades are allowed **when
  explicitly declared and implemented** so they stay *known and bounded* — the enemy is
  *unintended/unknown/unbounded* effects (memory explosion, runaway cascade, spooky action), not effects
  per se; default tightly scoped, broader opt-in by explicit declaration; recovery is **additive over**
  the explicit error (never silent — G2; never fabricates or upgrades a guarantee — VR-5). Isolation:
  budget enforcement lives with RFC-0004/0008/DN-05, **not** the kernel; clean **RFC-0013 split**
  (presentation vs. recovery; shared registry/pattern; RFC-0014 does not weaken RFC-0013's I1).
- Prior art (Result/`?`, algebraic effects, **Erlang/OTP bounded supervision**, structured-concurrency
  cancellation, capabilities, Mycelium's own budget idiom) recorded as **design inspiration not yet traced
  to `research/`** (a pre-ratification task). Many design choices (effect mechanism, budget vocabulary, any
  kernel hook) are **explicit open questions** — no code lands with the draft; ratification + a tracking
  milestone are the maintainer's. RFC index + RFC-0013 §8/§9 cross-refs updated. Advances SC-3, G2, VR-5,
  NFR-2/SC-5b.

### Added (Phase 4 — M-345: RFC-0013 structured diagnostics & reified error policy, drafted from DN-04)
- **RFC-0013 — Structured Diagnostics & Reified Error-Handling Policy (Draft (Proposed)).** Turns the
  DynEL-inspired DN-04 direction into a ratifiable, **tooling-layer** design with **no kernel change**
  (KC-3) and **no Python** (ADR-007 Rust-first; DynEL is reference-only). Imports three contracts —
  **graded context levels** (verbosity over EXPLAIN / `FeedbackSummary` / `NotValidatedReason`), **dual
  human + JSON projection** of one content-addressed diagnostic (G11), and a **reified per-definition
  error-handling policy** `on <ErrorClass> => {message, tags, level, route}` in the RFC-0005/ADR-006
  pattern — and **normatively excludes** three anti-patterns (config-string `eval` → registry lookup;
  wholesale env/locals dump → an allowlisted detailed tier; `logger.catch` swallowing → additive over a
  still-propagating error). Governing invariant: **a diagnostic is additive presentation over an
  explicit error, never a substitute** (G2 never-silent).
- **DN04-Q1 resolved → presentation/routing only for v0.** A policy shapes message/tags/level/route; the
  explicit error/`Option`/refusal **still propagates** unchanged. **Declarative recovery is deferred** to
  a separate future RFC, with the maintainer's constraints recorded (RFC-0013 §8/§9): an **isolated,
  separable** subsystem (SoC, bounded blast radius) with **explicit, declared, bounded** effect
  semantics (errors-as-values / reified effect handlers — errors propagate/bubble and can *trigger*
  functionality; effects and cascades are allowed *when explicitly declared/implemented* so they stay
  known and bounded — the enemy is *unintended/unknown/unbounded* effects, not effects per se), always
  **additive over** the explicit error. DN04-Q2 = free-form
  string tags (v0); DN04-Q3 = file is a projection of the canonical declaration; DN04-Q5 = standalone RFC
  now (stdlib graduation, M-346, a future option).
- **Carries the representation-crossing audit view** routed here from RFC-0012 R12-Q2 / M-351: a
  location-independent view enumerating every `swap` with its honesty bound (Exact/Proven/Empirical/
  Declared, never upgraded — VR-5) and selection policy. Advances NFR-2 / SC-5b (semantic feedback) +
  the AI co-author loop (M-330). DN-04 status updated (now feeds RFC-0013); RFC index updated. No code
  lands with the draft — ratification is the maintainer's append-only decision.

### Added (Phase 4 — M-343: mutual recursion in the L0 calculus; RFC-0001 r5, R7-Q3 resolved)
- **`FixGroup` — one new L0 node for mutual recursion** (RFC-0001 r5; the n-way generalisation of
  `Fix`). `FixGroup{defs, body}` binds a strongly-connected call group simultaneously (each definition
  and the continuation see all the group's names), so two functions can call each other. The
  elaborator (`mycelium-l1::elab`) now decomposes the reachable call graph into SCCs (Tarjan,
  callee-first) and lowers a self-recursive singleton to `Fix` and a group of ≥2 to a `FixGroup`;
  **mutual recursion is no longer an `ElabError::Residual`** — a structurally v0 program no longer
  residualises on recursion at all (only a dynamic `@ guarantee` index does). The node carries **no
  captured environment** and unfolds by substitution under the **same fuel clock** as `Fix` (a *focus*
  member-name unfold + a *continuation* unfold; the group binds all member names so substitution
  shadows them) — a non-productive group is an explicit budget exhaustion, never a hang.
- **Enacted across the trusted base and the AOT path, in lockstep:** `mycelium-core` (node +
  `is_aot_lowerable` + content-addressing + the canonical/core/ANF formatters + `Rhs::FixGroup`),
  `mycelium-interp` (the two-case unfold + capture-avoiding `subst`), `mycelium-mlir::aot` (the
  env-machine `FixGroup` suspension + unfold; the native-LLVM subset refuses it with `UnsupportedNode`
  like the rest of the data/recursion fragment, VR-5), and the dialect/LSP walkers.
- **Verified by the three-way M-210 differential** (L1-eval ≡ elaborate→L0-interp ≡ AOT) extended with
  mutually-recursive programs — ping/pong, even/odd over a Bool result, a constructive group that
  builds data on the way back, and a three-function cycle — plus a `FixGroup`-lowering witness. Resolves
  **R7-Q3** (the cycle *identity* was fixed in RFC-0001 r4; the matching *node* lands now). Full
  `cargo test` green. (NFR-7, VR-5, SC-3, LR-1; KC-3 — the kernel grows by exactly one deliberate,
  ratified node.)

### Decided (Phase 4 — M-351: RFC-0012 R12-Q1 & R12-Q2 resolved; no new ambient code)
- **R12-Q1 (per-use size) → no new sugar.** A paradigm-less **ascription** `e : {N}` already states an
  explicit size at the use site with the paradigm from the central `default` (now tested:
  `mycelium-l1/tests/ambient.rs::a_paradigm_less_ascription_states_the_per_use_size`), so a context-free
  bare decimal is sizable without a surrounding annotation and elaborates identically to longhand (I2).
  **Sizes stay explicit** (no ambient default width); a `u8`/`f64` literal suffix was **rejected**
  (imports signed/dtype affordances the kernel does not provide — v0 `Binary` is unsigned, no `iN`,
  `f64` is a Dense dtype not a width — a false-affordance footgun that also fails to generalize across
  the four paradigms). A paradigm-agnostic `:N` shorthand stays a possible future sugar iff terseness
  earns it (KISS/YAGNI).
- **R12-Q2 (paradigm-boundary swaps) → crossings stay at swap sites.** No default swap policy. **Swap
  sites** vs **`with paradigm` block edges** were weighed against the language's intention (fluid,
  paradigm-agnostic traversal): swap sites win — a `swap` is a free, first-class *anywhere* crossing and
  `with paradigm` stays pure tag-scoping (SoC), so safety stays total (explicit `swap`/G2,
  `MissingConversion`, ADR-016) while traversal stays maximally easy. Block edges would add only
  *auditability*, and only by constraining where crossings may live (forbidding mid-body swaps) — so the
  *boundary-audit* idea is **routed to observability tooling (M-345 → DN-04 / RFC-0008)** as a
  location-independent "every representation crossing + its honesty bound" view, where lossy conversions
  live. The enforced block-edge boundary is recorded as an optional future discipline (RFC-0012 §9, not
  adopted); the RFC-0005 decision-table form stays gated on RFC-0005 policy-objects in `mycelium-l1`.
  **M-351 (#114) closes with no new ambient surface.**

### Added (Phase 4 — M-344: enact RFC-0012 ambient representation; surface-only, never a black box)
- **`mycelium-l1::ambient` — the ambient resolution pass (RFC-0012 §4.3/§4.4 enacted).** A *declared,
  scoped, paradigm-only* default (`default paradigm P`) plus block-scope overrides
  (`with paradigm P { … }`) and a paradigm-less repr `{N}` / `{N, scalar}` / `{model, dim, sparsity}`,
  to offset honesty's verbosity (tension **A**) **without** a black box. Realized as a **surface→surface
  "expand to longhand" pass**: `resolve(Colony) → Colony` fills omitted paradigm tags, strips
  `with paradigm` blocks, and tags bare decimals, then the **unchanged** `check → elaborate` pipeline
  runs — so the two normative invariants hold *by construction*: **(I1)** the ambient inserts no `Swap`
  (it only fills tags/encodings — conversions stay author-written), and **(I2)** resolution is
  observationally the identity (`elaborate(p) = elaborate(resolve(p))`, identical content hash;
  RFC-0001 §4.6). The feature is **opt-in**: a program with no ambient resolves to itself unchanged.
- **Bare-decimal width-from-context (RFC-0012 §4.3; the maintainer-chosen v0 scope).** The checker is
  now **bidirectional**: a bare decimal under an ambient adopts the paradigm's encoding and takes its
  **width from the checked context** (an ascription, a parameter/return/field type, or a concrete
  sibling operand of a width-preserving prim). Where the width is **not** determined, it is an explicit
  **`UnresolvedWidth`** refusal — *never a built-in default width*. `Binary` unsigned and `Ternary`
  balanced encodings are range-checked (an overflow is an explicit refusal, never a silent wrap).
- **Three never-silent refusals (no black box; G2).** `UnresolvedAmbient` (a `{…}` with no enclosing
  ambient — no implicit global fallback), `ParadigmShapeMismatch` (a shape that does not fit the ambient
  paradigm — never coerced), and `MissingConversion` (a cross-paradigm value edge — the checker’s
  cross-paradigm mismatch is sharpened to name from/to + point at writing an explicit `swap`).
  Bare decimals under `Dense`/`VSA` (no bare-decimal encoding) and a duplicate colony `default` are
  refused too.
- **"Expand ambient" projection (M-142/LSP; RFC-0012 §5).** `mycelium-l1::expand_to_source` +
  `mycelium-lsp::expand_ambient` render a document's fully-resolved **longhand twin** on demand (the
  elided default is never *hidden*, only *elided*); a parse/check failure is reported, never a partial
  render. Provenance for "where did this paradigm come from?" is recorded at the **surface/resolution
  layer** (`ResolutionNote` via `resolve_report`) rather than as a new core `Provenance` variant — that
  would change a frozen data-contract schema for metadata that is not hashed (KC-3; see the RFC-0012
  changelog).
- **The RFC-0012 §4.6 meaning-preservation differential (NFR-7; `tests/ambient.rs`).** A corpus of
  `(ambient program, explicit longhand twin)` pairs asserts **identical elaborated content hash** (I2)
  and identical observed value where runnable; the never-silent refusals are each tested as explicit
  errors. **Grammar + conformance**: `mycelium.ebnf` gains `default paradigm` / `with paradigm` / the
  paradigm-less repr, with a new accept fixture (`12-ambient-representation.myc`) and reject fixture
  (`09-default-missing-paradigm.myc`). **Kernel untouched** (KC-3 — L0's frozen node set is unchanged;
  this is RFC-0006 surface sugar that elaborates away). RFC-0012 (Accepted) → **Enacted**; R12-Q3/Q4
  resolved, R12-Q1/Q2 partially (v0 enacted, extensions deferred).

### Added (Phase 4 — M-349: dynamic depth budget for the AOT env-machine; DN-05 §2.4 / DN05-Q5 enacted)
- **`mycelium-mlir::budget` — a `DepthBudget` trait that resolves the env-machine's control-stack
  ceiling *dynamically*, with an `EXPLAIN`-able basis (DN05-Q5 resolved).** With the M-347 trampoline
  the control stack is on the **heap**, so the ceiling is honestly a policy over **memory**: the default
  `AutoDepthBudget` reads detected headroom — `MemAvailable` (`/proc/meminfo`) capped by a finite
  `RLIMIT_AS` (`/proc/self/limits`) — via **pure-`std` `/proc`** (Linux), spends 70 % ÷ a conservative
  1 KiB/frame estimate, and clamps to `[10 000, 2 000 000]`. **Zero `unsafe`** (no FFI, no SP-reading —
  ADR-014's "minimal unsafe" satisfied with *none*); non-Linux or any read/parse failure falls back to
  the conservative static default (the prior `200 000`), **never a guess**. The resolved ceiling **and
  its derivation** are an inspectable `DepthResolution`/`DepthBasis` (`Display`; `aot::default_depth_budget`;
  printed by `xtask recursion-probe`) — no black box (G2); the limit itself stays an explicit
  `EvalError::DepthLimit` (never an abort/hang). `run`/`run_core`/`run_core_with_fuel` now resolve it
  dynamically; `run_core_with_budget` keeps the explicit override. **Measured** on this host: `MemAvailable`
  ≈ 15.99 GB → raw ≈ 10.9M, clamped to the 2 000 000 ceiling (vs the old fixed 200 000); a constrained
  host *tightens* below the fallback (unit-tested: 256 MiB ⇒ ≈ 183k). A **property test** bounds the
  derivation (`[floor, ceil]`, monotone in headroom) for all inputs incl. saturation. Per-frame cost is a
  `Declared` over-count (VR-5), not `Proven`. Trusted interpreter unchanged; three-way differential holds
  (NFR-7). DN-05 §2.4 / **DN05-Q5 → Resolved**; native-path *stack* detection (DN05-Q4 / M-348) reuses the
  trait.

### Changed (Phase 4 — M-347: AOT env-machine made stack-robust via a trampoline; DN-05 #2 enacted)
- **`mycelium-mlir::aot` rewritten as a trampoline over an explicit heap control stack
  (`eval_machine`).** Object-level recursion now lives on the **heap**, so the env-machine uses **O(1)
  host stack** — matching the reference interpreter. Deep recursion is bounded by **two explicit,
  graceful budgets**: `fuel` (Fix unfolds; time → `EvalError::FuelExhausted`) and a **control-stack
  depth ceiling** (space → new `EvalError::DepthLimit { limit }`) — **never a host-stack abort, never a
  hang** (G2). `run_core_with_budget(fuel, max_depth)` exposes both; `run`/`run_core`/`run_core_with_fuel`
  unchanged. **Empirically confirmed** (`xtask recursion-probe`, re-run): the env-machine is graceful at
  every fuel to 5 000 000 (`FuelExhausted` ≤200k, `DepthLimit{200000}` ≥250k) — the pre-fix ~600-unfold
  abort (DN-05 §1.1) is **gone**. The three-way differential (L1≡L0≡AOT) is unchanged (NFR-7 holds).
  **RFC-0004 §2** now banks the matching **normative requirement** for the native MLIR→LLVM path
  (stack-robustness designed in, not retrofitted — DN-05 #1; libMLIR provisioning is M-348). The
  *dynamic* depth budget (derive `max_depth` from headroom) stays the deferred policy (DN-05 §2.4 /
  DN05-Q5); the fixed 200 000 default is conservative + configurable.

### Added (Phase 4 — DN-05 + recursion-probe: AOT recursion stack-robustness strategy, M-347/M-348)
- **DN-05 (Draft) — AOT recursion execution strategy, empirically grounded.** Investigates making the
  M-342 env-machine recursion stack-robust *without bloat*. New `xtask recursion-probe` **measures**
  (not presumes) the limitation: the AOT env-machine aborts (host-stack overflow) at **~600**
  `Fix`-unfolds, while the reference interpreter is graceful at fuel 5 000 000 in **O(1)** host stack
  (a tiny-AST `spin`, abort depth found by binary-searching fuel in subprocesses; re-runnable). Records
  the maintainer-set priority: **(1)** bank native MLIR→LLVM stack-robustness as a design requirement
  (libMLIR-gated; provisioning is near-term via desktop/WSL — **M-348** #110); **(2)** an explicit
  control stack / **trampoline** in the env-machine (near-term buildable; turns the abort into an
  explicit budget/limit — makes never-silent **total** for the AOT path); **(3)** **tail-call
  detection** — cautious, optional, on top of #2, only if it earns its keep (KC-3/KISS/YAGNI). Plus
  **§2.4: the limit must be *dynamic*** — detect stack/heap headroom + per-frame cost at runtime and
  derive the safe depth (the ~14 KB/unfold cost varies by build/platform, so a static constant is the
  wrong knob), behind a small `DepthBudget` trait with a conservative static fallback, `EXPLAIN`-able
  basis, and an explicit error (never an abort/hang/black box). The trusted interpreter stays the base
  for deep recursion until #2 lands; the M-210 differential must still hold (NFR-7). Tracked M-347
  (#109, P1) + M-348 (#110). Design-first — no fix lands with the note.

### Added (Phase 4 — M-342: AOT path extended to the data + recursion fragment; RFC-0011 §4.4 Q5 closed)
- **The AOT `aot::run` env-machine now covers the full v0 calculus (M-342).** `mycelium-core::lower`
  gains ANF for the r3/r4 nodes — `Construct`/`App` (flat) and `Lam`/`Fix`/`Match` (with **nested ANF
  blocks** evaluated lazily, a single program-wide temp counter keeping temps globally unique) — and
  `mycelium-mlir::aot` becomes a big-step **environment machine** with closures (capturing their env),
  call-by-value `App`, fuel-clocked `Fix` unfolding, `Construct`→`Datum`, and arm-selecting `Match`.
  `run_core` returns a `CoreValue` (repr **or** datum); `run` keeps the repr-`Value` signature.
- **The three-way differential now spans the full calculus.** `mycelium-l1`'s data/recursion corpus
  (data, nested matches, self-recursion, `for`-folds) is checked **L1-eval ≡ L0-interp ≡ AOT** on the
  L0 `CoreValue`, with the shared **M-210** checker validating each repr-result pair (NFR-7). Closes
  RFC-0011 §4.4 **Q5**; `Node::is_aot_lowerable` is now total over the v0 node set.
- **Honest scope (VR-5).** The *native* direct-LLVM backend stays the **bit/trit subset** — the data +
  recursion nodes are an explicit `UnsupportedNode` refusal there (data/closure native codegen is the
  deferred MLIR→LLVM work). The env-machine uses the **host call stack** for object recursion (the
  fuel clock bounds *productive work* — a non-productive recursion is an explicit `FuelExhausted`,
  never a hang — but depth beyond the host stack aborts); the trusted base for deep recursion remains
  the O(1)-stack interpreter. A follow-on, **M-347** (#109), tracks making the env-machine recursion
  stack-robust / more efficient.

### Changed (Phase 4 — RFC-0012 RATIFIED: Draft → Accepted)
- **RFC-0012 ratified (Draft → Accepted, 2026-06-16; append-only).** The ambient-representation
  design (§4) is now the normative surface contract: the two invariants (I1 the ambient emits no
  `Swap`; I2 resolution is observationally the identity) and the never-silent override /
  `MissingConversion` rule are in force. The kernel is unaffected (KC-3 — RFC-0001's frozen node set
  is untouched). **No code lands with acceptance** — the elaborator/checker wiring is the gated
  follow-on **M-344** (#106): the resolution pass, the never-silent refusals, M-142/LSP "expand
  ambient" rendering, and the §4.6 meaning-preservation differential. RFC README + Doc-Index updated.

### Added (Phase 4 — roadmap: Mycelium core library / stdlib, M-346)
- **M-346 (#108) — core-library / stdlib roadmap anchor.** Records the maintainer's goal of a solid
  core-library feature set for usability, to be decomposed once the surface language is self-hosting
  (dogfooding; free of other *languages*). Inherits the non-negotiable principles (never-silent G2,
  honest per-op guarantee tags, no black boxes/EXPLAIN, small kernel KC-3 — stdlib lives *above* it,
  content-addressed ADR-003; Rust-first ADR-007 now → Mycelium-lang eventually). Seeds: `diagnostics`
  (DN-04), collections, numerics helpers, VSA/encoding utils, I/O + wire-form serialization. No code;
  draft a Core Library RFC near self-hosting and present before folding.

### Added (Phase 4 — DN-04 Draft: optional structured diagnostics, DynEL-inspired, M-345)
- **DN-04 (Draft) — evaluate DynEL's (`gitlab:albedo_black/DynEL`) feature set as *opt-in* structured
  diagnostics** (`docs/notes/DN-04-…`). Source read (maintainer-supplied zip). **Governing
  constraint:** diagnostics are *additive presentation* over Mycelium's explicit, reasoned errors —
  **never a substitute** for a never-silent error/`Option`/`CheckVerdict::NotValidated` (G2). Imports
  the *contracts* — graded context levels (minimal/medium/detailed), human + machine-readable (JSON)
  output as two **projections** of one content-addressed diagnostic (G11/M-380), and a **reified
  per-definition error-handling policy** `{exceptions, custom_message, tags}` (the RFC-0005 pattern;
  ADR-006) — and explicitly **excludes** DynEL's three anti-patterns: `eval`-on-config (code
  execution), full `os.environ` dump at the detailed level (secret leakage), and `logger.catch`
  exception-swallowing (a never-silent violation). **Rust-first (ADR-007): no Python added** — DynEL
  is reference-only; the feature is a Rust tooling-layer renderer (kernel untouched, KC-3), **eventually
  self-hosted in Mycelium-lang** (dogfooding; free of other *languages*). Tracked as M-345 (#107);
  Doc-Index + `idmap.tsv` / `issues.yaml` updated.

### Added (Phase 4 — RFC-0012 Draft: ambient representation & scoped overrides, M-344)
- **RFC-0012 (Draft) — a surface-only, declared, scoped, *paradigm-only* representation default +
  scoped override/conversion blocks** (`docs/rfcs/RFC-0012-…`), to offset honesty's verbosity (tension
  A) while refusing black boxes. The honest core is two **normative invariants**: **(I1)** the ambient
  emits no `Swap` (it fills an *omitted paradigm* + bare-literal encoding only — conversions stay
  author-written, WF1/WF2); **(I2)** resolution is observationally the identity — a program with the
  ambient and its longhand twin elaborate to *identical* L0 ⟹ identical content hash (RFC-0001 §4.6),
  defended by a meaning-preservation differential (NFR-7/M-210). Forbids the two black-box failure modes
  (repr-inference-from-usage; silent conversion insertion); cross-paradigm edges stay explicit `swap`s
  and a missing one is an explicit `MissingConversion` refusal (G2). The **trusted kernel is untouched**
  (KC-3) — L0's frozen node set does not change; this is RFC-0006 surface/term-layer sugar that
  elaborates away. Cross-module: exported signatures are concrete L0 reprs (ADR-016 boundary), so the
  ambient never leaks across modules. Per maintainer direction (2026-06-16): **paradigm-only**
  granularity, **full v0 scope** (defaults + overrides). **No code, no RFC-0001 change** — Draft is the
  present-before-fold step; ratification + wiring are the maintainer's append-only decision. RFC README +
  Doc-Index updated; issue M-344 (#106) added to `idmap.tsv` / `issues.yaml`.

### Changed (Phase 4 — ADR-016 + ADR-017 RATIFIED: Proposed → Accepted)
- **ADR-016 + ADR-017 ratified (Proposed → Accepted, 2026-06-16; append-only).** Maintainer gate
  cleared — no change to either decision. ADR-016 fixes the interpreted↔compiled ABI (dispatch by
  content hash; the RFC-0001 §4.8 wire form as the canonical value boundary); ADR-017 fixes
  hot-inject (hash-keyed dispatch + content-addressed dynamic linking, immutable-by-construction).
  ADR README + Doc-Index status updated to Accepted; the RFC-0004 §10 OQ-1/OQ-2 pointers stand.

### Added (Phase 4 — M-341: the in-process hot-inject prototype on the M-340 JIT)
- **`mycelium-mlir` gains the `inject` module — ADR-017's named first build step (ADR-016 call ABI).**
  An `Image` holds a `ContentHash → entry` dispatch table over the M-340 `dlopen` JIT:
  - **a call resolves to a compiled entry if present, else interprets** the registered definition
    (the RFC-0004 §9.1 continuum); a hash with neither is an explicit `InjectError::DispatchMiss`,
    never a silent guess (G2/SC-3) — and `resolve` makes the dispatch decision `EXPLAIN`-able;
  - **`inject` loads a content-addressed unit and registers a new `hash → entry`**, never mutating a
    live entry (publish-once; an edit is a new hash under a new entry — the atomicity hazard
    dissolves, ADR-017 decision 4);
  - **`recompile_closure`** computes the changed dependency-closure by hash reachability over the
    dependency graph — the recompile set, with no AST/file diff (decision 3).
  **Verified (NFR-7):** the injected-compiled path is observationally equivalent to the reference
  interpreter through the shared **M-210** TV checker (`ObservationalEquiv`); the safety argument is
  exercised under test — an in-flight call to the old hash finishes on old code while a new caller
  dispatches to the new hash (`tests/inject_hotswap.rs`). **Honest scope (VR-5):** in-process proof
  only; a unit is a *closed* bit/trit-subset program and the call boundary is the call ABI restricted
  to nullary units — the args-carrying value ABI (RFC-0001 §4.8 wire form) and cross-process / native
  units (RFC-0004 §2 / §10 OQ-3) stay deferred. New issues M-341 (#103), M-342 (#104, AOT-fragment
  extension), M-343 (#105, mutual-recursion elaboration) created + added to `idmap.tsv` / `issues.yaml`.

### Added (Phase 4 — ADR-016 + ADR-017 Proposed: the interpreted↔compiled ABI + hot-inject)
- **ADR-016 (Proposed) — the interpreted↔compiled ABI (RFC-0004 §10 OQ-1).** Dispatch a compiled
  stable component by its **content hash** (versioning is free, staleness structurally impossible —
  ADR-003: a change is a new hash, so an old compiled entry can never be applied to a changed
  definition); cross `CoreValue`s in the **self-describing wire form** (RFC-0001 §4.8) as the canonical
  value ABI, with a zero-copy fast-path as a *later, validated* optimization (robust/portable first).
  Honesty crosses the boundary (`Meta`/guarantee travel with the value — WF5). The boundary is
  toolchain, not kernel (KC-3); codegen deferred (MLIR→LLVM, RFC-0004 §2).
- **ADR-017 (Proposed) — hot-inject recompiled definitions (RFC-0004 §10 OQ-2).** A hash-keyed
  dispatch table (ADR-016) + content-addressed dynamic linking (the M-340 `dlopen` JIT is the seed):
  inject = load a content-addressed unit + register `hash → entry`, **never** mutate running code. The
  classic atomicity hazard **dissolves** because definitions are immutable — a change is a *new hash
  under a new entry*, so in-flight calls finish on old code and new callers dispatch to new code; the
  recompile set is **exactly the changed dependency-closure** by hash reachability (no AST diff). A
  working in-process prototype on M-340 is the recommended first build step once ratified; native
  codegen deferred. RFC-0004 §10 OQ-1/OQ-2 now point at the ADRs; ADR README + Doc-Index updated.

### Added (Phase 4 — RFC-0004 §9.2/§9.3 reference impl: build-target profiles in mycelium-build)
- **`mycelium-build` gains the `target` module — the build-target profiles (RFC-0004 r2 §9.2/§9.3),
  orthogonal to the §4 stable-component gate.** `BuildProfile` = `Interpret` (no targets, dev default)
  / `Slim(Target)` (one) / `Selective(set)` (a chosen subset) / `Fat` (all supported) — fat is
  first-class but optional; `targets()` resolves each to a concrete `(os, arch)` set. Slim/selective/fat
  share **one** artifact shape, a content-addressed per-target `VariantTable` (§9.3), with **never-silent
  runtime dispatch** (`select(host)` → the host's variant or an explicit `DispatchMiss` the caller
  resolves by interpreter fallback or refusal — never a wrong-target variant, G2/SC-3). **Honest scope
  (VR-5):** `realizable_targets` admits only the **host** today — a non-host `--slim`/`--target`/`--fat`
  is an explicit `BuildError::CrossTargetDeferred` (cross-target codegen awaits the MLIR→LLVM backend,
  RFC-0004 §2), never a host-only build mislabeled as fat. This is the build-orchestration layer that is
  *ready* for that backend, not the backend. (RFC-0004 §9; 15 build-crate tests)

### Added (Phase 3/4 — M-310 real LSP document sync, on the now-complete text→Node→L0 pipeline)
- **`mycelium-lsp` gains real document sync (`sync` module + `serve` wiring).** With the surface→L0
  pipeline complete (RFC-0011 r3 / RFC-0001 r4), the LSP server now handles
  `textDocument/didOpen`/`didChange`/`didClose` (full sync — `TextDocumentSyncKind.Full`, advertised
  in `initialize`), re-analyzing the whole document through **parse → check** on each edit and pushing
  `textDocument/publishDiagnostics` (cleared on a clean edit / close). **Honest spans (VR-5):** a
  *parse* diagnostic carries a **real** `line:col` range (the lexer's `Pos`); a *check* diagnostic is
  located at its `fn <name>` declaration with the function name in `data.breadcrumb` (the checker
  tracks the failing function, not yet the failing sub-expression span — flagged, never fabricated).
  `mycelium-lsp` now depends on `mycelium-l1` for the text→`Node` path (no cycle). Closes the M-310
  residual that the RFC-0011 enactment unblocked; phase-3 M-310 row → Done. 515 workspace tests pass.

### Changed (Phase 4 — RFC-0001 r4 ENACTED: Lam/App/Fix in L0; full L1-in-Core-IR)
- **Functions + general recursion are folded into the trusted Core IR (RFC-0001 r4), completing
  L1-in-Core-IR and retiring RFC-0007 §4.6's `Residual` for self-recursion entirely.** A
  self-recursive, data-building, matching program now elaborates to a closed L0 term and runs on the
  trusted reference interpreter + the M-210 differential.
  - **RFC-0001 r3 → r4** (append-only; **supersedes the r3 §4.5 grammar**): §4.5 gains `Lam` + `App` +
    `Fix` (RFC-0007 §4.1; **R7-Q1 resolved — a `Fix` node**); §4.2 gains the **function value model**
    (maintainer-confirmed: the v0 surface is first-order, so `Lam`/`App`/`Fix` are **closed** —
    application is capture-free substitution, **no environment-capturing closure value**, honoring
    §4.7; capturing closures + partial application are a named later revision); §4.6's **cycle-ordering
    is finished** (**R7-Q3 for identity** — a mutually-recursive declaration group now content-addresses
    canonically + name-independently). RFC-0007 §4.6 `Residual` retired except mutual recursion +
    dynamic guarantee indices; the `matured` totality gate (RFC-0007 §4.5) restated unchanged (the
    interpreter clocks every `Fix` — a mis-classification gates packaging, never meaning).
  - **Code:** `mycelium-core` (the three nodes + content-addressing + the canonical
    `canonical_cycle_order`); `mycelium-interp` (small-step β-reduction CBV; `Fix` unfolds by
    substitution under the fuel clock → non-productive recursion is an explicit `FuelExhausted`, never
    a hang; applying a non-function / a bare-function result are explicit refusals);
    `mycelium-l1::elab` (each reachable self-recursive function → `let f = Fix(f, λparams. body)`,
    calls → curried `App`, non-recursive calls still inline; `for` → a synthesized self-recursive
    `Fix` fold; **mutual recursion** → explicit `Residual`, deferred R7-Q3); `mycelium-lsp` walks.
  - **Verified (NFR-7):** the M-210 differential extends to the recursive + `for` fragment (L1-eval ≡
    elaborate→L0-interp on the `CoreValue` observable), with a mutual-recursion-refuses witness. 509
    workspace tests pass; clippy clean; `cargo fmt` applied. (RFC-0001 r4 / RFC-0007 §4.6/§8 Meta)

### Changed (Phase 3 — exit gate RE-ASSERTED MET; both residuals closed)
- **`docs/planning/phase-3.md` moves `Living draft → exit-gate met`.** With residuals **R1** (M-310
  text→`Node` path) and **R2** (RFC-0006/0007 ratified) both closed by the RFC-0011 r3 enactment, the §6
  gate's three conditions are satisfied: native execution path (met+measured), matured toolchain (the
  parser→checker→elaborate→L0 pipeline exists; the `didOpen`/`didChange` wiring is an ordinary M-310
  task, not gate-blocking), and L1 surface (RFC-0011 r3 enacted, RFC-0001 → r3). Claimed at the strength
  the checked runs establish (VR-5): 497 workspace tests + the M-210 data-fragment differential. Phase-3
  build tasks (M-310 sync, M-350/M-360 locals) continue past the gate; the standing core-language
  continuation is **RFC-0001 r4** (`Lam/App/Fix` into L0). Append-only (supersedes the "no exit gate
  claimed" line). (phase-3.md §6.1)

### Changed (Phase 3 — RFC-0004 r2: interpreted↔compiled continuum + build-target profiles; additive)
- **RFC-0004 gains §9 (the interpreted↔compiled continuum + build-target profiles) and §10 (open
  questions) — additive, changing no r1 decision (append-only).** Records the maintainer's execution
  direction (2026-06-15): **interpret freely during development (zero build step, the reference
  interpreter is the meaning), compile what is ready, never be forced into a heavyweight build, never
  recompile what has not changed.** §9 makes explicit that execution is a *per-definition continuum*
  (not interpreted-vs-compiled), that mixed interpreted + compiled stable components coexist in one run
  (same L0 `CoreValue` semantics, §3 checker guarantees agreement), and that **incremental compilation
  is "for free" from content-addressing** (ADR-003 — a definition's hash is its identity, so a compiled
  artifact is never stale; M-311/M-312 already realize the cache). The **build-target profiles** are
  normative and flexible: `interpret` (default), `build --slim <os>-<arch>` (one target), `build
  --target <list>` (a chosen subset), `build --fat` (all supported targets, universal) — **fat
  multi-target is first-class but optional, supported from the start**, the slim/selective/fat artifacts
  share one format (a content-addressed per-`(os,arch,cpu-features)` variant table), and runtime variant
  dispatch is **never-silent** (an unmatched host falls back to the interpreter or refuses explicitly,
  never runs a wrong-target variant — the M-360 SIMD feature-dispatch generalized). Cross-target rides
  §2's MLIR→LLVM path and stays **host-only until that backend lands** (honest deferral). §10 flags the
  genuinely-new, undesigned items: the interpreted↔compiled **ABI** (OQ-1), **hot-inject** of recompiled
  definitions into a running image (OQ-2; the M-340 `dlopen` JIT is the seed), the **fat-artifact
  packaging format** (OQ-3), and target-set-as-RFC-0005-policy (OQ-4). (RFC-0004 r2 Meta)

### Changed (Phase 3 — RFC-0011 r3 ENACTED: data + flat `Match` in L0; RFC-0001 → r3; M-320/M-310)
- **The L1 data-and-matching core is now folded into the frozen Core IR and implemented in lockstep
  (RFC-0011 r3, enacting the named RFC-0001 revision).** `Construct` + the flat `Match` are L0 Core IR
  nodes, so a non-recursive program that builds/matches data reaches the trusted reference interpreter
  and the M-210 differential — closing the text→`Node` gap that blocked **M-310** document sync
  (gate residual **R1 closed**) and dead-ended **M-320**'s decision-tree compiler.
  - **RFC-0001 r2 → r3** (append-only; **supersedes the r2 §4.5 grammar**): §4.5 gains `Construct` +
    flat `Match` + `Alt` and **WF6/WF7/WF8**; §4.6 gains the content-addressed **data registry Σ**
    (`CtorRef = #T#i`, Unison self-recursive placeholder hashing; mutual recursion implemented but
    deferred to r4 per R7-Q3); §4.2 gains the **data value `Datum`** + the runtime sum **`CoreValue`**;
    §4.7 gains the **datum guarantee-summary** addendum. RFC-0011 → **Accepted, r3 ENACTED**; RFC-0007
    §4.6's `Residual` is **narrowed** (retired for data/matching; `App`/`Fix`/`for` stay `Residual`, r4).
  - **The one genuinely-open value-model choice (maintainer-confirmed):** `Datum` is a **sibling** type —
    `Value<R>` is unchanged, *not* refactored into a `Repr | Data` sum — and carries a **meet-summary
    guarantee with no `Bound`** (bounds stay on the leaf representation values; an addendum to §4.7). The
    smaller, isolated change honors KC-3/KISS/YAGNI (data values arise only as `Construct`/`Match`
    results, never as `Const` literals in r3).
  - **Code:** `mycelium-core` (the registry, `Datum`/`CoreValue`, the nodes, content-addressing +
    canonical dump; AOT stays repr-only via `Node::is_aot_lowerable`, RFC-0011 §4.4 Q5);
    `mycelium-interp` (small-step `Construct`/`Match` + `eval_core`; `Construct` = `meet(fields)`;
    `Match` meet is identity for `Exact` scrutinees and an **explicit refusal** for a non-`Exact` data
    scrutinee — never a fabricated bound); `mycelium-l1::elab` (the M-320 Maranget tree lowers nested
    patterns to nested flat L0 `Match`, binding all constructor fields; `if` → `Bool` match).
  - **Verified (NFR-7):** the M-210 differential extends to the data fragment — **L1-eval ≡
    elaborate→L0-interp** on the `CoreValue` observable (`L1Value::to_core` bridges name-keyed →
    `#T#i`), with a mutant-witness; the M-310/M-320 phase-3 rows and §6.1 exit-gate verdict updated
    (R1 + R2 closed). 497 workspace tests pass; clippy clean; `cargo fmt` applied.
  - **Honesty/scope (VR-5):** `Lam/App/Fix` remain the named **r4** revision (full L1-in-Core-IR,
    R7-Q1/Q3); the AOT path and mutual-recursion cycle-ordering are explicit, flagged deferrals — not
    silent gaps. (RFC-0001 r3 / RFC-0011 / RFC-0007 §4.6 Meta)

### Changed (Phase 3 — RFC-0006 & RFC-0007 ratified, Draft → Accepted r4; maintainer sign-off)
- **RFC-0006 (surface/term-layering) and RFC-0007 (L1 kernel calculus) are now Accepted (r4), with a
  scoped §10 carve-out.** A completion-review found **no missing normative content** in the
  KC-2-independent scope — both are mature, and the v0 L1 calculus is prototype-realized in
  `crates/mycelium-l1` and exercised by the M-320 usefulness + decision-tree work — and the maintainer
  signed off on the carve-out. **Ratified:** RFC-0006 §3 layering / §4.1 invariants S1–S6 / §4.2
  capability targets LR-1…LR-9 / §4.3 grammar discipline / §8 positions Q2·Q4·Q5·Q7 (now realized by
  RFC-0007 §4.1–4.7 and the ratified **RFC-0011** staged-r3 `Match`-into-L0 decision), and RFC-0007
  §4.1–4.8 (the v0 calculus, stage-0 dynamic guarantee check). **Stays gated/deferred (NOT ratified):**
  concrete L3 surface syntax (KC-2/M-002-external), stage-1 static grading (RFC-0006 Q3 implicit-flows
  decision / R7-Q2), R7-Q1·Q3 → RFC-0001 r4, R7-Q4, and traits/LR-2. No design content changed on
  acceptance; each RFC's status line + §10 carry the carve-out so "Accepted" is never read as ratifying
  the gated parts (VR-5). RFC README index + Doc-Index status updated. This unblocks the core-language
  step (the RFC-0011 r3 enactment + M-320 L0 wiring). (RFC-0006 r4 / RFC-0007 r4 Meta)

### Changed (Phase 3 — true bitnet.cpp 1.67-b/w TL2 layout closes A5-08, M-360; E3-6; RFC-0004 §5)
- **`mycelium-mlir::pack` now realizes `TL2` as the true bitnet.cpp layout (1.67 b/w).** The prior
  `TL2` was a placeholder that packed identically to the `FiveTritPerByte` base-3 reference (5
  trits/byte ⇒ 1.6 b/w), while the selector cost model priced TL2 at the published **1.67 b/w** — the
  A5-08 discrepancy. `TL2` is now the real layout: **3 trits → a 5-bit LUT-index** (`c = d₀+3·d₁+9·d₂
  ∈ [0,27)`), bit-packed as a contiguous 5-bit-field stream ⇒ `5/3 ≈ 1.67` b/w — *less* dense than the
  1.6-b/w base-3 reference on purpose (the 5-bit index is directly LUT-addressable, bitnet's fast-decode
  trade). The two schemes are now genuinely distinct densities; a new shared `needed_bytes(scheme,
  count)` bound model (`⌈5·⌈count/3⌉/8⌉` for TL2) replaces the per-byte assumption. The native TL2
  **dot kernel** (`mycelium-mlir::bitnet`) decodes the bitstream inline (`digit = (code / 3ᵖ) mod 3`)
  with a **branch-free bounds-clamped 2-byte window** — the second byte index is clamped to the last
  valid byte (computed from `n`), so the final group's read never goes out of bounds even when its
  5-bit field fits in one byte (spilled bits masked off by `& 31`). Oracle-checked across widths
  (`jit_dot_matches_reference_all_schemes`); the bound is a refusal test; new `pack` property tests pin
  the 1.67 b/w density and the TL2≠`FiveTritPerByte` distinctness. The selector cost model now **matches**
  the codec — **A5-08 resolved** (the notes in `pack.rs` and `mycelium-select` updated from "stand-in /
  inert discrepancy" to "resolved"). `cargo xtask e1` §3 times the true TL2 kernel (≈1.25× vs scalar —
  honestly *slower per-element* than I2_S, the bitstream decode being more work; as-measured).
  **Honesty/scope (VR-5):** realizes the bitnet.cpp TL2 *density + 5-bit-LUT-index semantics*; the exact
  upstream byte/bit ordering is not claimed byte-identical (needs the source to verify) — the codec is
  self-consistent (round-trip identity) and oracle-checked. (phase-3.md §2 / §9.8 / Meta)

### Added (Phase 3 — BitNet hand-vectorized SIMD kernel, M-360; E3-6; FR-C3 / G3; RFC-0004 §5/§8)
- **`mycelium-mlir::simd` — a hand-vectorized (8-wide) I2_S packed-ternary dot kernel.** The scalar
  BitNet kernels decode one trit per loop step; this emits `i64 @myc_bitnet_dot_simd(ptr %w, ptr %x,
  i64 %n)` that unpacks + multiply-accumulates **8 trits per iteration** with LLVM vector types:
  broadcast the two packed bytes across 8 lanes (`shufflevector` mask `<0,0,0,0,1,1,1,1>`), bring each
  lane's 2-bit code to bit 0 (`lshr` by the constant vector `<0,2,4,6,0,2,4,6>`), `& 3` → code, `− 1`
  → signed weight, `mul <8 x i32>` with the contiguous activations, widen + accumulate into an
  `<8 x i64>` phi, then horizontally reduce (`@llvm.vector.reduce.add.v8i64`) with a **scalar epilogue**
  for the `n mod 8` tail. Every vector op is visible in the emitted IR (no opaque pass — FR-C3 /
  RFC-0004 §6); the vector loads carry explicit `align 1`/`align 4`. It reuses `BitnetDotKernel`'s
  bounds-checked `call` (a `pub(crate) from_loaded` ctor — DRY; same C signature + I2_S density model),
  so a short buffer is still an explicit refusal, never an OOB read. **The vector unpack is
  correctness-critical, so it is differential-checked against the scalar kernel as the oracle** —
  `tests/simd_differential.rs` runs a corpus bracketing the 8-lane width and the tail
  (n ∈ {0,1,7,8,9,15,16,17,31,33,64,255,256,257,1000}) and validates each scalar↔SIMD pair **through
  the single shared M-210 checker** (`ObservationalEquiv`/`Exact`), with a mismatched-buffer
  discrimination test (guard 7) so a green pass is not vacuous. `cargo xtask e1` **§5** times SIMD vs
  scalar over the same runtime buffer (indicative ≈1.2× — honest: clang already auto-vectorizes the
  scalar `-O2` loop, so the hand-vectorized gain is real-but-modest; as-measured, no target
  pre-written). **Scope/honesty (VR-5/G3):** **I2_S only** this increment (TL1/TL2 vectorized unpacks,
  plus the true 1.67-b/w bitnet.cpp **TL2 layout** that closes A5-08, are next); no parity with bitnet.cpp's
  AVX2/AVX512 LUT kernels is claimed; same exact dot product, no guarantee upgraded; the scalar kernels
  stay the oracle. (phase-3.md §2 / §9.8 / Meta)

### Added (Phase 3 — RFC-0011 the keystone: L0 `Match` / L1-in-Core-IR, ratified-decision; M-320/M-310)
- **`docs/rfcs/RFC-0011-L0-Match-and-L1-in-Core-IR.md` (Accepted — decision; enactment sequenced) — the named RFC-0001 revision.**
  The L0 Core IR is frozen at five nodes (`Const/Var/Let/Op/Swap`); RFC-0007 designed five L1 nodes but
  stopped short of putting them *into* L0 (its §4.6 elaboration covers only the evaluation-complete
  fragment, the rest is an explicit `Residual`). RFC-0006 §4.4 step 2 and RFC-0007 §9 name the missing
  step — "add the L1 node set to the Core IR" — and **this is that proposal.** It is the keystone for two
  stalled half-tasks: **M-320** (emit Maranget decision-tree leaves as real L0 nodes — blocked because L0
  has no matching node) and **M-310** (document sync — blocked because there is no text→`Node` path for
  matching/data). The RFC recommends a **staged** revision — **RFC-0001 r3** = the data-and-matching core
  (`Construct` + flat `Match` + a content-addressed data registry, with new kernel WF6/WF7/WF8 lifting
  RFC-0007's W6/W7/W8), staged ahead of an **r4** that adds `Lam/App/Fix` — so the five-node kernel grows
  in two auditable steps (KC-3). It recommends the **flat `Match`** as the kernel node (the M-320 Maranget
  tree stays the *untrusted, inspectable* compilation artifact above the kernel, per RFC-0007 §6), and
  records the two alternatives a maintainer might prefer (a low-level `Switch`/`Leaf` kernel form; the
  one-shot five-node fold). **Ratified 2026-06-15 (decision only; enactment sequenced).** The maintainer
  chose the staged path; RFC-0011 is **Accepted as the decision**, but because it depends on RFC-0007 and
  the maintainer directed that **RFC-0006 + RFC-0007 be completed and ratified first**, the §4.7 enactment
  — the RFC-0001 r2 → r3 text-fold, the RFC-0007 §4.6 narrowing, and the M-320 elaborator wiring — is
  **deferred** to land together as the core-lang step, in order: *exit-gate assembly → M-360 SIMD →
  ratify RFC-0006/0007 → enact r3 + wire*. **Frozen-L0 not flipped (VR-5):** RFC-0001 stays r2/frozen and
  the prototype keeps returning `Residual` until that step. Registered in the RFC README index and the
  Doc-Index. (phase-3.md §9.9 keystone)

### Added (Phase 3 — JIT runtime specialization, M-340; E3-4; ADR-009/ADR-014; RFC-0004 §5/§8)
- **`mycelium-mlir::specialize` — a weight-specialized ternary dot kernel (the classic JIT win).**
  The generic BitNet dot kernel (M-360) reads its weight buffer as a runtime pointer and re-unpacks it
  every call. In the inference setting the **weights are fixed at runtime** and only the activations
  vary, so `emit_specialized_dot_ir(weights)` bakes the (runtime-known) weight vector into the kernel
  `i64 @myc_bitnet_dot_spec(ptr %x)` as constants. The optimiser then **drops the unpack entirely**
  (no packed-byte load / shift / mask / `code−1`), **elides every zero-weight lane** (a `0` weight's
  activation load + multiply vanish from the emitted IR — the model's sparsity becomes inspectable,
  FR-C3), and **strength-reduces ±1 to a single `add`/`sub`**. The only runtime argument is the
  activation pointer; weights and length are compiled in. `compile_specialized_dot` JIT-compiles it
  (`clang -shared -O2`) via the M-340 dynamic loader; `SpecializedDotKernel::call` takes **no weight
  argument** (running it against weights it was not built for is unrepresentable — never a silent
  stale-weights run) and **bounds-checks** the activation buffer (a short buffer is an explicit
  `AotError`, never an OOB read). `nonzero()` exposes the surviving-lane count for EXPLAIN/inspection.
  **Validated (NFR-7):** `tests/specialize_differential.rs` runs the specialized and generic kernels
  over the same activations and validates them as observationally equivalent **through the single
  shared M-210 checker** (`ObservationalEquiv`, `Certificate::exact()` ⇒ `Validated{Exact}`), plus a
  negated-weights discrimination test that the checker must reject (guard 7, so a pass is meaningful).
  **Honest speedup (E1 §4 / VR-5):** `cargo xtask e1` §4 times specialized-vs-generic over the same
  runtime activation buffer (both runtime pointers, no constant folding) after an oracle cross-check;
  indicative single run (n=4096, ~66 % dense) ≈ **10.7× as measured** — reported as-measured, no
  target pre-written, sparsity/machine-dependent. **Honesty/scope:** same exact dot product, no
  guarantee upgraded (both `Exact`); the weights are runtime data baked at JIT time, activations stay
  runtime pointers, so the compute is real. (phase-3.md §2 / §9.10 / Meta)

### Added (Phase 3 — L1 Maranget decision-tree compiler, M-320; E3-3; RFC-0007 §3/§4.4)
- **`mycelium-l1::decision` — the codegen half of the Maranget pipeline.** Compiles a checked
  nested-pattern `match` into a flat decision `Tree` of `switch`/`leaf` nodes over **occurrences**
  (paths into the scrutinee) — Maranget 2008's "good decision trees": a left-to-right column heuristic
  (rotate the first non-wildcard column to the front), constructor/literal specialization, and a
  `default` branch **exactly** when a column's signature is incomplete (a data type missing
  constructors) or its domain is open (`Binary`/`Ternary`, never enumerated). This is RFC-0007 §3's
  "patterns compiled away by the elaborator", as the analysis-level IR. **Verified, not asserted:** a
  test-only tree evaluator (`eval_tree` over concrete `Pat` values) is checked to agree with a
  reference matcher on every `Nat` value up to a depth (a wrong column choice / specialization would
  diverge), plus first-match-on-overlap and the literal-needs-a-default shape. **Wired into the
  checker:** `checkty::infer_match`, after exhaustiveness passes, compiles the match and confirms the
  tree is `has_reachable_fail`-free — an exhaustive match must compile to total coverage, so the
  usefulness analysis (Maranget 2007) and the tree compiler must agree (defense in depth; an internal
  disagreement is an explicit error, never silent). **Honesty/scope (VR-5):** the tree's leaves are
  **not yet emitted as L0 Core IR** — L0 has no `Match` node, and adding one is the planned RFC-0001
  revision (RFC-0007 §4.6); the compilation algorithm is real and checked, and the L0 emission is the
  remaining step. No guarantee is touched; RFC-0006/0007 ratification stays the maintainer's
  append-only decision. (phase-3.md §2 / §9.9 / Meta)

### Added (Phase 3 — LSP wire protocol, M-310; E3-3; FR-S5 / SC-5)
- **`mycelium-lsp::wire` wraps the feedback facade in the LSP transport.** The byte-level JSON-RPC 2.0
  codec — `read_message`/`write_message` with `Content-Length` header framing (a clean inter-message
  EOF returns `None`; a truncated body / missing or invalid `Content-Length` / non-JSON body is an
  explicit `io::Error`, never a silent partial read) — plus the `Diagnostic` → LSP-`Diagnostic` mapping
  (spec `DiagnosticSeverity`: Error→1, Warning→2), the `textDocument/publishDiagnostics` notification
  builder, and a minimal `serve` lifecycle loop (`initialize` → capabilities + `serverInfo`,
  `shutdown` → null result, `exit` → stop; any other **request** → JSON-RPC `MethodNotFound` -32601,
  never silence; unknown notifications ignored). New dependency: the workspace-pinned `serde_json`.
  **Honesty/scope (VR-5):** not a document-syncing server — the facade analyzes Core IR `Node`s, not
  source text, so the server advertises `TextDocumentSyncKind.None` and the diagnostic `range` is a
  **zero placeholder** with the navigable location carried in `data.breadcrumb`; real source spans and
  `didOpen`/`didChange` sync arrive with the L1 surface (M-320), and the wire layer carries them
  without a protocol change. Seven tests (framing round-trip incl. back-to-back, clean-EOF,
  truncated-body refusal, severity mapping, `publishDiagnostics` shape, the scripted-client lifecycle,
  the unknown-request refusal). (phase-3.md §2 / §9.7 / Meta)

### Added (Phase 3 — BitNet TL1/TL2 packed-ternary kernels, M-360; E3-6; RFC-0004 §5/§8)
- **`mycelium-mlir::bitnet` now covers all three bitnet packings.** The I2_S-only dot kernel
  generalised to `emit_bitnet_dot_ir_for(scheme)`: **TL1** inverts the rot=2 code LUT
  (`d01 = (code+1) mod 3`, signed weight `d01−1`) and **TL2** decodes the base-3 5-trits/byte packing
  (`digit = (byte / 3ᵖ) mod 3` with the `3ᵖ ∈ {1,3,9,27,81}` divisor chosen by an inline select-chain),
  each a scalar loop with the scheme-specific unpack inlined and **inspectable** in the emitted LLVM IR
  (no opaque pass — RFC-0004 §6 / FR-C3). `BitnetDotKernel` carries its `PackScheme`, so the
  weight-buffer bounds check tracks the packing density (`n.div_ceil(4)` for I2_S/TL1, `/5` for TL2) —
  a short buffer stays an explicit `AotError`, never an OOB read. A non-bitnet `PackScheme` (Unpacked /
  TwoBitPerTrit / FiveTritPerByte) is the new explicit `AotError::UnsupportedScheme` refusal, never a
  silent misdecode. Each kernel is **differential-checked** against the packing-independent oracle
  `ternary_dot_ref` over the same `pack_trits` packing (`jit_dot_matches_reference_all_schemes`, n up to
  1000; the JIT actually compiled+ran here, matching all three). The **E1 §3** harness
  (`cargo xtask e1`) now times **all three** packings in-process over runtime data, each against a
  hand-written scalar baseline doing the identical per-scheme unpack (measured here: JIT beats scalar
  1.69× I2_S / 1.31× TL1 / 1.15× TL2 — whatever was measured, no pre-written claim, VR-5). The
  **A5-08** cross-reference notes (`mycelium-mlir::pack`, `mycelium-select`) are refined: the scalar
  TL2 kernel decodes the **1.6-b/w placeholder codec**, so it does *not* resolve the published
  1.67-b/w TL2 discrepancy (still inert for selection) — aligning to bitnet.cpp's true TL2 layout is
  now explicitly tied to the **real-layout / SIMD** increment, not the scalar kernel. **Honesty/scope:**
  scalar loops only — no parity with bitnet.cpp's hand-tuned **SIMD** is claimed (the next M-360
  increment); no guarantee is upgraded (VR-5/G3). (phase-3.md §2 / §9.8 / Meta)

### Added (Phase 3 — board sync: Phase-2 issues closed, Phase-3 M-3xx bootstrapped)
- **Tracker hygiene only.** Closed the completed Phase-2 epics (E2-1…E2-7, #28–34) and tasks
  (M-230…M-260, #58–65) as *completed* with grounding comments (CHANGELOG Batch G/H; Phase-2 exit gate
  met 2026-06-12). Created the Phase-3 M-3xx build tasks (#86–#98) from `tools/github/issues.yaml`,
  linked as sub-issues under E3-1…E3-7, closed the six shipped ones (M-301/302/303/311/312/370). Updated
  `tools/github/idmap.tsv` (M-301→#86 … M-380→#98) and `docs/planning/phase-3.md` §2/§8/Meta. No code or
  corpus-normative change.

### Added (Phase 3 — decode `enum_budget` default ratified, M-350; ADR-015; RFC-0010 §8)
- **`docs/adr/ADR-015-decode-enum-budget-default.md`** (Accepted): ratifies the RFC-0010 decode-selector
  default **`DEFAULT_ENUM_BUDGET = 4096`** (= `MAPI_RESONATOR_PROFILE.max_capacity`), the
  *guarantee-maximal* arm — every in-regime request is also enumerable, so the brute-force `Exact` arm
  dominates the whole validated envelope (never take `Empirical` when `Exact` is cheaply available) —
  over the *cost-optimal* ≈128. Grounded in the already-measured `∏k ≈ 100–128` cost-parity crossover
  (`d`-independent; ≈ 19× / ≤ ≈ 157 ms latency tax at the regime edge `∏k=4096`; cited from the
  `decode_method_enum_budget_crossover` instrument, **not re-run**). Tagged a `Declared` policy stance;
  neither value upgrades any guarantee (VR-5) — the budget moves only *which arm runs*, never *what tag
  it earns*. The cheap resonator-arm identifiability precheck (RFC-0010 §8) is recorded as the deferred
  re-open trigger (YAGNI). Standalone decision record — **no code, kernel, or test change**. Registered
  in `docs/Doc-Index.md` and the ADR index; RFC-0010 §8's `enum_budget` open question marked **resolved**
  (append-only footer).

### Fixed (Phase 3 — resonator premature-abort, M-350; RFC-0009 §3/§6)
- **Resonator no longer aborts a still-converging tuple as an oscillation.** The §3 loop decided
  oscillation on *any* recurrence of the decoded index tuple `ι`, so a tuple that had gone **stationary
  on `ι` while its per-slot confidence was still climbing** toward `τ_lock` (e.g. F=3,k=16, Hebbian,
  d=4096: the correct tuple at iter 2 with slot similarities `[1.0, 0.998, 0.72↗]`) recurred in the
  history at distance 1 and was mislabelled `Oscillating{period:1}` — a recoverable instance refused.
  The fix splits the two cases the discrete `ι` alone conflated: a **genuine limit cycle** (a *distinct*
  earlier tuple recurs ⇒ `period ≥ 2`) still refuses as `Oscillating`; a **stationary tuple** keeps
  iterating while the lock bottleneck (min per-slot similarity) is still rising and only refuses, with
  the new explicit `StopReason::Stalled` / `VsaError::ResonatorStalled` verdict, once that climb
  plateaus below `τ_lock` for `STALL_PATIENCE` sweeps (genuine stuck fixed point — **never-silent
  preserved**). Net effect: F=3,k=16 went **1/300 → 0/300** on the seed that exhibited the abort; the
  canonical 1000-trial gate stays **0/1000 ⇒ δ=0.02** (the gate's worst corner was already 0/1000, so
  the conservative ceiling is **unchanged** — no unmotivated tightening, VR-5). Tag stays **`Empirical`,
  MAP-I only, never `Proven`**; only a clean `Converged` clearing `τ_lock` + confidence + margin yields
  factors. The prior `stall_below_lock_*` unit test was updated (not deleted) to assert the new `Stalled`
  verdict; a regression test pins the exact previously-aborting instance to `Converged`. (phase-3.md §2 / Meta)

### Added (Phase 3 — resonator-network factorization prototype, M-350; RFC-0009 §10.2)
- **`mycelium-vsa::resonator`** — the RFC-0009 §3 factorization loop over any `VsaModel`
  (MAP-I-first), recovering the unknown factors of a bind product `s = x₁ ⊛ … ⊛ x_F`. Parallel /
  Jacobi **snapshot** update (§8.1 P6); softmax-superposition or arg-max cleanup (§9 Q2); uniform /
  seeded init (§9 Q1); convergence **and** oscillation decided on the **discrete top-atom index tuple
  `ι`** (§8.1 P3), bounded by the iteration budget. Deterministic via an in-crate LCG (no `rand`).
- **Never-silent honesty made structural (RFC-0009 §5.4/§6).** `factorize` returns a `Factorization`
  **only** on a clean `Converged` verdict that clears `τ_lock` + per-slot confidence + margin;
  `BudgetExhausted`, `Oscillating`, below-confidence, and below-margin are explicit `VsaError`s
  carrying the inspectable `ResonatorTrace` ("converged ≠ correct").
- **`ResonatorProfile` + `MAPI_RESONATOR_PROFILE`** — the `{F, ∏kᵢ, d}` regime gate
  (`check` → `OutsideEmpiricalProfile`; `bound` → `EmpiricalFit`), distinct from the bundle
  `EmpiricalProfile` (§5.2/§9 Q4). First regime `F≤2, k≤8, ∏k≤64, d≥4096`.
- **Trial-validated δ, oracle-measured.** `tests/resonator_oracle.rs` asserts **exact-tuple recovery**
  against a brute-force oracle (+ an exhaustive-argmax identifiability check);
  `tests/resonator_profile.rs` runs exactly `trials` (1000) at the worst point, scoring exact recovery
  (not self-reported convergence — §8.1 P5): **measured 0/1000 ⇒ δ=0.01** conservative ceiling, the
  test that *earns* the `Empirical` tag (VR-5).
- **Value-level decode.** `mycelium-vsa::reconstruct_factors` mirrors `reconstruct_role`: reads the r4
  `Resonator` manifest params, gates on the profile, runs the loop. Tag is **`Empirical`, MAP-I only,
  never `Proven`** (schema-enforced); sparse/HRR/FHRR deferred (§9 Q6). Additive `CleanupMemory`
  `atoms()`/`dim()` accessors; four resonator `VsaError` variants. **Nothing new in the kernel** beyond
  the r4 additive manifest metadata fields. (phase-3.md §2 / Meta)

### Added (Phase 3 — RFC-0010 follow-ups: enum_budget crossover + Value-level wiring, M-350)
- **`enum_budget` crossover measured (RFC-0010 §8).** A wall-clock instrument
  (`tests/decode_select.rs::decode_method_enum_budget_crossover`, `#[ignore]`d) times brute force vs the
  resonator per decode across `{F, k, d}`: the **cost-parity crossover is `∏k ≈ 100–128`** (d-independent
  — both scale with `d`); brute force is cheaper only for `∏k ≲ 64` and costs **≈19×** the resonator at
  the regime edge `∏k=4096` (≈76 ms vs ≈4 ms, d=4096). So `DEFAULT_ENUM_BUDGET = max_capacity` (4096) is
  **guarantee-maximal** (always `Exact` in-regime, bounded ≤ ≈157 ms at d=8192), *not* latency-minimal
  (≈128) — recorded as-measured (VR-5); the default value is a guarantee-vs-latency policy call, exposed
  per call and surfaced in the EXPLAIN cost lines. `DEFAULT_ENUM_BUDGET`'s doc carries the trade.
- **Value-level auto-selected decode** — `mycelium-vsa::reconstruct_factors_selected` routes a
  `Resonator` manifest through the RFC-0010 selector (instead of always running the resonator),
  returning a `DecodeSelection` with the **tag read off the chosen arm**. Unlike `reconstruct_factors`,
  it does **not** pre-gate on the resonator profile — a brute-forceable instance *outside* the resonator
  regime (e.g. `F=4, k=8`, ∏=4096, which the plain decode refuses) is recovered **exactly** by brute
  force (RFC-0010 §4.4). Shared manifest→`ResonatorParams` reading refactored into a helper (DRY). Four
  new `recon` tests (brute-Exact, resonator-Empirical, the F=4 capability gain, non-resonator rejection).
  (phase-3.md §2 / Meta)

### Added (Phase 3 — RFC-0010 decode-methodology selector prototype, M-350)
- **`mycelium-vsa::decode_select`** — the RFC-0010 decode-methodology selector, reusing the **one**
  RFC-0005 selection mechanism as a **third site** (no parallel selector). `reconstruct_factors_auto`
  routes a factorization request among `{ BruteForceExact, Resonator, Refuse }` by an ordered decision
  table over **exact** facts (`F`, `∏kᵢ`, `d`, `ResonatorProfile` membership), runs the chosen arm, and
  returns the recovered factors with the **guarantee tag read off the arm** — brute-force enumeration is
  **`Exact`** (identifiability-checked against ties), the resonator is **`Empirical`**, else an explicit
  `VsaError::DecodeRefused`. Every selection emits the mandatory EXPLAIN (`explain_decode_method` is the
  pure, no-execution form). `DecodeMethodPolicy` is content-addressed (`enum_budget` is part of its
  identity).
- **Honesty floor enforced (RFC-0010 §4.5).** A forced `BruteForceExact` beyond `enum_budget`, a forced
  `BruteForceExact` on a non-identifiable instance (`VsaError::NonIdentifiable`), and a forced
  `Resonator` out of regime all still **refuse** — a first-class override cannot escape the floor or
  upgrade a tag (VR-5). The `mycelium-core::recon` `≤Empirical` ceiling is untouched.
- **Mechanism extended additively** (`mycelium-select`, core-only): an abstract `DecodeMethod`
  candidate, the `DecodeFacts` queryable facts, the `CapacityAtMost`/`FactorsAtMost`/`InResonatorRegime`
  predicates, and the `select_decode_method` adapter. `mycelium-vsa` now depends on `mycelium-select`
  (acyclic — `mycelium-select` is `mycelium-core`-only).
- **Honest finding recorded.** With `DEFAULT_ENUM_BUDGET = MAPI_RESONATOR_PROFILE.max_capacity` (4096),
  *every* in-regime request is also enumerable, so the brute-force `Exact` arm dominates the **entire**
  validated regime (never take `Empirical` when `Exact` is cheaply available) — the resonator arm
  becomes load-bearing only at a tighter budget (latency) or once the validated capacity grows beyond
  the enumeration budget. The `enum_budget` wall-clock crossover stays the RFC-0010 §8 open question.
  (phase-3.md §2 / Meta)

### Added (Phase 3 — RFC-0010 decode-methodology selection design, M-350 needs-design)
- **`docs/rfcs/RFC-0010-Decode-Methodology-Selection.md`** (Draft): the design artifact for choosing a
  **decode methodology** as a **third site of the one RFC-0005 selection mechanism** (no parallel
  selector — DRY/SoC). A content-addressed, `EXPLAIN`-mandatory decision table over **exact** metadata
  (`F`, `∏kᵢ`, `d`, model, `ResonatorProfile` membership) routes among
  `{ BruteForceExact (Exact), Resonator{Hebbian} (Empirical), Refuse }`, with the **guarantee tag read
  off the chosen arm** (VR-5) and out-of-regime / non-identifiable inputs an explicit refusal
  (never-silent — G2). Records the §10.3 finding that the **cleanup-variant axis collapses to one
  winner (Hebbian)** inside the validated envelope, so cleanup-selection is **deferred** (YAGNI) with a
  concrete re-open trigger. **No code; nothing in the kernel.** Registered in the Doc-Index + RFC index;
  design gated on ratification. (phase-3.md §2 / Meta)

### Changed (Phase 3 — resonator operational-capacity wall breached, §10.3 cleanup ablation, M-350)
- **`MAPI_RESONATOR_PROFILE` widened `F≤3, k≤8, ∏k≤512` → `F≤3, k≤16, ∏k≤4096, d≥4096`** by fixing the
  cleanup dynamics, **not** by loosening the honesty contract. The original softmax cleanup fed the
  *real-valued* superposition straight into the next bind, so crosstalk compounded through the
  elementwise product of `F−1` noisy real vectors — the prototype collapsed as `∏k → d`. The §10.3
  ablation (`tests/resonator_profile.rs::resonator_cleanup_ablation`, `#[ignore]`d) measured four
  cleanups at the wall; the **Hebbian bipolar** projection `sign(Σⱼ simⱼ·cⱼ)` (Frady et al. 2020) keeps
  the explain-away on the `±1` alphabet, so the MAP-I unbind stays *exact*. Measured at F=3,k=16
  (∏=4096): **softmax 300/300 fail → Hebbian 0/300** at d=4096; the canonical 1000-trial gate now
  validates the F=3/k=16/d=4096 worst corner at **0/1000 ⇒ δ=0.02** conservative ceiling. New
  `Cleanup::Hebbian` (the validated default) + `Cleanup::SoftmaxSign`; `ResonatorParams::mapi_default`
  and the unspecified-manifest decode path adopt Hebbian (the kernel `CleanupShape` is unchanged —
  Hebbian lives only in `mycelium-vsa`).
- **Honest boundary recorded.** `SoftmaxSign` does **not** breach the wall (sign of a sharp softmax ≈ a
  noisy arg-max); `ArgMax` only partially (brittle at the tight d=4096 corner). F=3,k=32 (∏=32768) is
  left **outside** the validated envelope: 0.085 at d=8192 (not tight), 0.005 only at d≥16384 — recorded
  as boundary data, not claimed. F=3,k=16 added to the brute-force oracle. Tag stays **`Empirical`,
  MAP-I only, never `Proven`**. (phase-3.md §2 / Meta)

### Changed (Phase 3 — resonator validated regime widened + operational-capacity map, M-350)
- **`MAPI_RESONATOR_PROFILE` widened `F≤2, ∏k≤64` → `F≤3, k≤8, ∏k≤512, d≥4096`** with a **measured**
  δ. A staged capacity sweep (`tests/resonator_profile.rs::resonator_capacity_sweep`, `#[ignore]`d)
  mapped the operational edge: F=2/k=8 = **0/300**; F=3/k=8 (∏=512) = **6/1000 = 0.006** at d=4096
  (→ **0.001** at d=8192) ⇒ **δ=0.02** conservative ceiling at the worst corner (gate re-measured
  4/1000 on a fresh seed). The canonical gate now validates the F=3/k=8/d=4096 worst point.
- **Operational-capacity wall recorded (honest boundary data).** The prototype's softmax resonator
  (β=6, budget 50) collapses as `∏k → d`: **F=3/k=16 (∏=4096) ≈ 100% failure even at d=8192/β=10**,
  and k=32 is hopeless. So `k≤8` is the validated edge for F=3 at these knobs — a far smaller
  operational capacity than the literature's tuned resonators, reported as-measured not as-hoped
  (VR-5). Tightening (β, d) helps the in-regime k=8 corner but does **not** breach the wall; that is
  left to a future increment (better cleanup/normalisation). F=3 added to the brute-force oracle.
  Tag stays **`Empirical`, MAP-I only, never `Proven`**. (phase-3.md §2 / Meta)

### Added (Phase 3 — RFC-0009 resonator-network factorization design, M-350 needs-design)
- **`docs/rfcs/RFC-0009-Resonator-Network-Factorization.md`** (Draft): the *needs-design* deliverable
  for M-350 — fixes the convergence regime and the honest guarantee **before** any factorization code
  is built (RR-5/G4). Specifies the iterative resonator update over the existing `VsaModel`
  bind/unbind/cleanup (Frady et al. 2020); a **probabilistic-only** contract (basis capped at
  `Empirical`/`Declared`, **never** `Proven`; the `mycelium-core::recon` `Resonator` schema already
  enforces this ceiling, FR-C2), with the operational regime `{F, kᵢ, d}` as a checked
  `EmpiricalProfile` side-condition; never-silent termination (bounded budget;
  `BudgetExhausted`/`Oscillating` are explicit verdicts, never a wrapped result); full
  reification/`EXPLAIN`; and the open design questions. Prior art (`embeddenator-retrieval`/`-vsa`)
  flagged to mine, not copy. **No code; nothing in the kernel.** Registered in the Doc-Index;
  prototype gated on ratification. (phase-3.md §2 / Meta)
- **RFC-0009 Draft revision — prior-art mining (M-350).** Read the reference implementations
  (`embeddenator-vsa::resonator`, `embeddenator-retrieval::core::resonator`) and folded the findings
  back into the contract while keeping status **Draft** and the honesty contract intact. New **§8.1**
  documents seven concrete pitfalls (unseeded init; an unbacked "self-inverse" on the *lossy*
  sparse-ternary bind; no oscillation detection + a wrong cosine-to-previous convergence test; no
  regime/`δ`; a wrong fixed point returned as an answer with no correctness test; in-place Gauss-Seidel
  rather than parallel update; silent zero-fill fabrication). **§9 open questions resolved as
  recommendations** (uniform seeded init; softmax default, `β = 1/temperature` trial-fit; discrete
  index-tuple convergence + bounded-window cycle detection; oracle-measured `δ` over a `{F, ∏kᵢ, d}`
  `ResonatorProfile`; confidence **+ margin** refusal via `CleanupMemory`; MAP-I-first, sparse/HRR/FHRR
  `Declared` not `Empirical`). Tightened §3/§5/§6 accordingly ("converged ≠ correct"; only a clean
  `Converged` verdict yields factors). Records the maintainer caveat that `embeddenator` is
  acknowledged-experimental / not-yet-working — mined for problem-discovery only, with no evidential
  weight for any guarantee or convergence regime (VR-5). Still **no code; nothing in the kernel.**
  (phase-3.md §2 / Meta)
- **RFC-0009 ratified — Draft → Accepted (M-350).** Maintainer ratifies the contract; status
  `Accepted` (append-only). Authorises the §10.2 prototype (next: the `mycelium-vsa::resonator` MAP-I
  loop + `ResonatorProfile` + brute-force oracle + Value-level `reconstruct_factors()` decode). The
  decode-side manifest params (`cleanup`/`init`/`τ_lock`/`β`/`seed`) land as additive `DecodeSpec`
  metadata fields via the append-only **RFC-0003 r4** revision — additive metadata only, no kernel
  logic/guarantee change, ≤`Empirical` ceiling preserved (RFC-0003 §2; KC-3). (phase-3.md §2 / Meta)

### Added (Phase 3 — L1 nested patterns + Maranget usefulness, M-320)
- **`mycelium-l1::usefulness`** — Maranget's usefulness algorithm `U(P, q)` over a typed pattern
  matrix (Maranget 2007), witness-returning. L1 `match` now supports **nested** constructor/literal
  patterns, with coverage *checked* (W7): **exhaustiveness** (a `_` must not be useful — the witness
  names a concrete missing case, e.g. `S(Z)`, reported verbatim) and **redundancy** (an arm covered by
  the earlier rows is unreachable, subsuming the M-320 duplicate-literal check).
- **Checker + evaluator + totality** lifted from flat to nested: a recursive, type-directed
  `check_pattern` (binders typed by field type, linearity enforced); a unified `infer_match` (data +
  `Binary`/`Ternary`); a recursive `try_match` in the evaluator; and structural-descent smallness
  seeded from **nested** sub-binders (so `S(S(m)) → m` descends and admits `matured`).
- **Scope/honesty:** RFC-0007 is **Draft** and the prototype non-normative; this is the analysis half.
  The Maranget *decision-tree compilation to the flat kernel `Match`* (Maranget 2008; RFC-0007 §3) is
  the elaborator/L0 path and lands with full L1-in-Core-IR. Coverage stays checked, no guarantee
  touched. (phase-3.md §2 / §9.9 / Meta)

### Added (Phase 3 — BitNet packed-ternary acceleration, M-360 first increment; closes the open E1 compute-throughput item)
- **`mycelium-mlir::bitnet`** — the canonical BitNet **ternary multiply-accumulate**
  (`y = Σ digit(wᵢ)·xᵢ`, ternary weights · integer activations) emitted as **inspectable** LLVM IR
  (`i64 @myc_bitnet_dot(ptr %w, ptr %x, i64 %n)`: load the packed I2_S byte, extract the 2-bit code,
  signed weight `code−1`, multiply-add — one transparent op per loop-body step, FR-C3 "metadata, not
  hidden lowering"). JIT-compiled (`clang -shared -O2`) and called **in-process over runtime-pointer
  buffers** via the M-340 dynamic loader (refactored into a reusable `dlopen_path`/`Lib::sym`).
  Differential-checked against the Rust oracle (`ternary_dot_ref`) over several widths; bounds-checked
  so a short buffer is an explicit `AotError`, never an out-of-bounds read.
- **`cargo xtask e1` §3 now measures genuine packed-ternary compute throughput.** Because the kernel's
  weight/activation buffers are runtime arguments (not baked-in constants), the optimiser cannot fold
  the computation — so §3 times real unpack-compute over `n = 4096` elements against a hand-written
  Rust scalar baseline doing the identical I2_S work. This resolves §2's constant-fold/spawn caveat
  that had blocked the compute-throughput verdict. **Scope/honesty:** I2_S + scalar only — no
  bitnet.cpp SIMD parity claimed, TL1/TL2 are the next increments; the E1 number is measured, not
  pre-written (VR-5 / G3). (phase-3.md §2 / §9.8 / Meta)

### Added (Phase 3 — native trit carry arithmetic `add/sub/mul`, M-301 done)
- **`mycelium-mlir` now lowers balanced-ternary carry arithmetic over `Ternary{m}`.** `trit.add` is a
  fixed-width **ripple-carry** (LSB→MSB; balanced digit `x srem 3 − 1` and carry `x sdiv 3 − 1` with
  `x = aᵢ+bᵢ+carry+4 ≥ 1`, so the LLVM `srem`/`sdiv` are euclidean), `trit.sub = add(a, neg b)`, and
  `trit.mul` is **shifted accumulation** in a 2m-trit buffer (each `b` digit scales `a` via `i32 mul`,
  the digit being ±1/0). Each mirrors `mycelium-core::ternary` digit-for-digit.
- **Fixed-width overflow is detected at runtime and never wraps silently (SC-3/G2).** A non-zero final
  carry (add/sub) or non-zero product high trit (mul) sets an `i1` flag carried through an extended
  **read-back protocol**: the AOT artifact prints a `'!'` sentinel line and the JIT kernel — now
  `i32 @myc_kernel(ptr)` — returns a non-zero status, both surfaced as an explicit `AotError::Overflow`
  matching the interpreter's `EvalError::Overflow`. The M-302 (native) and M-340 (JIT) differential
  corpora gain in-range add/sub/mul + a nested `(5+4)−4`, plus an overflow-parity test. **Completes
  M-301** (last open slice). (phase-3.md §2 / §9.1 / Meta)

### Added (Phase 3 — native-ternary forward-compat map, M-370)
- **`docs/notes/Native-Ternary-Forward-Compat.md`** (Living note): documents the **ternary
  value-semantics contract** and the forward map from today's emulated-on-binary packing to a future
  3-state hardware backend, with the `ternary` dialect (`mycelium-mlir::dialect`) as the **stub
  target** and the R7 portability guarantee (what a native backend must keep invariant — values, the
  selection mechanism, the honesty rule, interpreter-as-reference). Documentation + stub only; **no
  3-state backend built** (ADR-005 / VR-5). Registered in the Doc-Index. Completes E3-7 at the
  documentation level.

### Added (Phase 3 — in-process JIT, M-340; first intentional unsafe under ADR-014)
- **`mycelium-mlir::jit`** — an in-process JIT: emits the kernel as `void @myc_kernel(ptr)`, compiles
  it to a shared object (`clang -shared`), and calls it **in-process** via `dlopen`/`dlsym` (the
  first intentional `unsafe` FFI under ADR-014 — justified `// SAFETY:` comments +
  `#[cfg_attr(not(debug_assertions), allow(unsafe_code))]`, **no new dependency**). Reuses the same
  `lower_program` + element encode/decode as the AOT path, so it agrees with the interpreter through
  the shared M-210 `ObservationalEquiv` checker (`tests/jit_differential.rs`, NFR-7). Removes the
  process-spawn overhead of the M-303 AOT path; skips gracefully when `clang` is absent. **Honest
  E1:** the closed kernel constant-folds, so a calibrated compute-throughput verdict still needs
  runtime-input kernels (M-360) — not pre-written (VR-5). (phase-3.md §2 / Meta)

### Added (Phase 3 — native AOT trit slice `trit.neg`, M-301)
- **`mycelium-mlir::llvm` is now kind-aware** (a `Lane` carries `Binary{w}` *or* `Ternary{m}`): the
  direct-LLVM backend lowers **`trit.neg`** over `Ternary{m}` end-to-end (digit-wise `0 - x` — exact,
  no carry), printing ternary output as `'-'`/`'0'`/`'+'` via a branch-free `select` chain (still one
  op per element) and reading it back into a `Ternary{m}` value. The parse shape is derived from the
  actual lowering (`lower_program` is the single source of truth for `emit_llvm_ir` + `result_shape`).
  The M-302 differential corpus gains two trit-`neg` programs (compiled + checked). `trit.add/sub/mul`
  (balanced-ternary carry arithmetic) and `bit.*`/`trit.*` on the wrong lane kind are explicit
  refusals (G2). (phase-3.md §2 / Meta)

### Changed (decision — ADR-014: `unsafe` policy relaxed from `forbid` to permitted-but-warned)
- **`unsafe_code` is now `"warn"` workspace-wide (was `"forbid"`).** `unsafe` is permitted when
  explicit and justified: it **warns** in `cargo build`/`cargo test` (the caution incentive) and
  still compiles/runs, the `just check` lint gate exempts only this lint (`scripts/checks/lint.sh`
  now runs `clippy -- -D warnings -A unsafe_code`, every *other* warning still a hard error), and a
  site silences the dev warning **for production release** with
  `#[cfg_attr(not(debug_assertions), allow(unsafe_code))]` + a mandatory `// SAFETY:` comment.
  Recorded as **ADR-014** (append-only; amends the M-091 lint policy). Enables in-process JIT/FFI
  (M-340) via raw `extern "C"` `dlopen`/`dlsym` with no new dependency. The trusted-base crates stay
  unsafe-free. CONTRIBUTING + the ADR index updated.

### Added (Phase 3 — LSP maturation: structured feedback summary, M-310)
- **`mycelium-lsp::FeedbackSummary`** (`Feedback::summary()`): a structured roll-up of an analysis —
  per-artifact-kind counts, the Error/Warning breakdown, the worst severity, and `is_clean()` — the
  at-a-glance health signal an AI co-author's feedback loop (SC-5b/E3-2) or an IDE status line
  consumes without re-walking the channels. Adds `Diagnostic::path()` (the `at` breadcrumb as a
  navigable `Vec<&str>`). Two tests incl. a worst-severity mutant-witness. (phase-3.md §9.7)

### Added (Phase 3 — content-addressed build cache, M-312)
- **`mycelium-build::cache`** — `BuildCache` caches `BuildCertificate`s by **build-request** content
  address: the key folds the component's identity hash with every decision input (spec ratification,
  the three obligations, the `promote` flag), so an unchanged request is a `Hit` reusing the prior
  certificate and any change in verification state is a `Miss` that re-decides — never a stale hit
  (G2). Three tests incl. the weakened-obligation `Aot → Interpreted` miss (mutant-witnessed).
  (phase-3.md §9.6)

### Added (Phase 3 — build-system stable-component gate, M-311)
- **`mycelium-build`** (new crate, outside the trusted kernel — KC-3): makes the RFC-0004 §4
  stable/experimental gate executable. `check_eligibility` runs the automatic §4 checks (spec
  ratified + obligations discharged) with specific blocking reasons; `decide(component, promote)`
  routes to **AOT only for an eligible, explicitly promoted** component (promotion is deliberate,
  §4) and refuses promotion of an ineligible one (never a silent AOT). Emits a content-addressed
  `BuildCertificate` (`cert_ref`, BLAKE3) with private fields and a re-validating `Deserialize`
  (`deny_unknown_fields`) so a forged `Aot` certificate is rejected on deserialize. Seven tests incl.
  forged-AOT + unknown-field rejection. (phase-3.md §9.5)

### Added (Phase 3 — L1 literal-pattern `match`, M-320)
- **`mycelium-l1`**: `match` now covers `Binary{n}`/`Ternary{m}` scrutinees with **literal patterns**,
  not just data types (the explicitly-deferred v0 gap). `checkty::infer_literal_match` enforces
  repr+width-matching literal arms, rejects duplicate literals, and **requires** a `_`/binder default
  (the 2ⁿ/3ᵐ domain is never enumerated — W7 coverage is never assumed); `eval::eval_literal_match`
  fires an arm on `repr + payload` equality. Elaboration is unchanged (the `Match` family already
  lowers to `Residual`). Five tests incl. three mutant-witnessed refusals. RFC-0007 ratification is
  presented, not flipped — that stays the maintainer's append-only decision (concrete syntax remains
  KC-2-gated). (phase-3.md §9.4)

### Added (Phase 3 — E1 native-path measurement, M-303)
- **`cargo xtask e1` §2** now measures the native AOT path against the interpreter (M-303): one-time
  AOT compile cost, warm native per-invocation (process spawn + run), and interpreter per-eval, for a
  bit-subset program. The E1 verdict moves from "no native path (stub)" to **native path established
  and measured** — the *compute-throughput* verdict ("reaches hand-packed perf") stays honestly NOT
  established, now with a precise reason: the standalone tiny-kernel artifact is process-spawn-bound
  and constant-folds, so it needs in-process execution (JIT/FFI — M-340 / deferred libMLIR). Adds the
  `compile` / `CompiledArtifact::run` compile-once/run-many split to `mycelium-mlir::llvm` (with
  `compile_and_run` as the wrapper). **Batch J (M-301→M-302→M-303) complete at the task level.**
  (phase-3.md §9.3)

### Added (Phase 3 — interp↔native differential, M-302)
- **`mycelium-mlir/tests/native_differential.rs`** — extends the M-151 differential to the *compiled*
  path: a bit-subset corpus runs under the reference interpreter and `compile_and_run`, asserting
  observable `(repr, payload, guarantee)` equality **and** validation through the single shared M-210
  `ObservationalEquiv` checker (NFR-7/VR-4/RR-12). A discrimination test confirms the differential is
  non-vacuous (two different programs → `NotValidated`). Skips gracefully when `llc`/`clang` are
  absent. (phase-3.md §9.2)

### Added (Phase 3 — native execution path, M-301 bit-subset slice)
- **`mycelium-mlir::llvm`** — a **direct-LLVM-IR AOT backend** that genuinely compiles the kernel
  **bit subset** (`core.id`, `bit.not/and/or/xor` over `Binary{w}`) to native code. `emit_llvm_ir`
  renders textual LLVM IR (one SSA op per output bit — no opaque pass, RFC-0004 §6); `compile_and_run`
  drives `llc` + `clang` to a real executable, runs it, and reads the result back as an `Exact`
  `Binary{w}` value. This is the first *compiled* execution path (RFC-0004 §2's direct-LLVM fallback;
  libMLIR absent, LLVM 18 present — the MLIR dialect lowering stays deferred, RR-N1). Everything
  outside the subset is an explicit `AotError` refusal (never silent); `llc`/`clang` absence is a
  skippable `ToolchainMissing`. Tests cover emit shape/determinism, four mutant-witnessed refusals, a
  width-mismatch refusal, and a toolchain-gated native↔interpreter roundtrip. (phase-3.md §9.1)

### Added (Phase-3 planning — scoping cut)
- **`docs/planning/phase-3.md`** (Living draft): scopes the Phase-3 epics #35–#41 (`E3-1…E3-7`) into
  `M-3xx` build tasks. Records the batch/parallelization plan with the **native execution path as the
  keystone** (it unblocks E1 + JIT/BitNet/native-ternary), the Phase-2→3 KC-1…KC-4 re-run, a
  **proposed** exit gate scoped to the buildable/local deliverables (exploratory + KC-2-gated epics
  tracked as honest out-of-gate stretch), and the risk register. **No exit gate claimed.** New risk
  **RR-N1**: the env has LLVM 18 but **no libMLIR**, so the realized first native step is a
  **direct-LLVM-IR AOT backend** (the RFC-0004 §2 fallback) with the MLIR dialect path deferred — a
  sequencing decision flagged for maintainer ratification, not silently adopted. KC-2 (LLM API) and
  the MLIR path (libMLIR) are named as the two external blockers.
- **`tools/github/issues.yaml`**: the Phase-3 epics decomposed into `M-301…M-380` child tasks
  (issue numbers pending bootstrap). Companion-doc references in `phase-0/1/2.md` updated
  (`phase-3.md` is no longer "forthcoming").

### Fixed (deep-review remediation — Medium/Low/Nit tail; all findings now closed)
- The remaining **Medium/Low/Nit** findings across every workstream are resolved (one commit per
  area), completing the review's Gate-A list — **0 findings now open**:
  - **core/cert (WS2):** recon manifest schema↔Rust reconciled (A6-06), `swap-certificate` requires
    bijective `params` (A6-09), `MalformedSparsity` variant (A6-08), basis-rank rule (A1-04),
    SC-3 helper asserts strength (A1-05), kernel `unreachable!`→`debug_assert` (C1-05).
  - **vsa (WS3):** MAP-I/MAP-B bind/unbind enforce the ±1 alphabet (A3-04), `EmptyCodebook` variant
    (A3-07), BSC on-expectation `Proven` documented (A3-06), tie-break/HRR/SC-2 notes (A3-08/09/10),
    `Bundle.hs` header reconciled (A3-05).
  - **l1/interp (WS4):** reject corpus pins per-file expected error reasons (A4), `Wf`-path +
    fuel-at-depth tests (A4-04), documented depth ceiling (A4-03).
  - **select/mlir/dense (WS5):** non-ternary layout refusal (A5-02), `unpack_trits` returns a
    `Result` not a silent truncation (A5-03), non-vacuous dense sweep (A5-05), pinned op-eps
    constants (A5-07), comment fixes (A5-06/A5-08).
  - **lsp/kc2/xtask (WS6):** never-silent unsupported-swap-pair diagnostic (A6-05), `exec` gated
    behind `allow_untrusted` (A6-10/B2-04), kc4 bijective-dec precheck (A6-11).
  - **numerics (WS1 deferred):** kernel-type fields are `pub(crate)` with accessors + a validating
    `Certificate` deserialize, making the outward-rounding/range invariants structural (A2-05);
    composed `Proven` basis preserves the input theorems' provenance (A2-09).

### Added (developer tooling — supply-chain gate)
- **`just deny`** (`scripts/checks/deny.sh`, in `just check`): runs `cargo deny check` + `cargo
  audit` when present (skip-if-missing), with a root `deny.toml` (advisories/licenses/sources).
  `.github/dependabot.yml` added (github-actions + cargo + pip, weekly; PRs only — no auto-CI).
  `[profile.release] overflow-checks = true`. gitleaks/cargo-deny/cargo-audit added to
  `install-tools.sh`. `npx markdownlint-cli2` pinned. Editorial: docs say "ruff format
  (Black-compatible)"; codespell + markdownlint clean repo-wide.

### Fixed (deep-review remediation — Wave 1)
- **WS3 — VSA certified-capacity side-conditions (finding A3-03/C1-02 H6 — the last Wave-1 High;
  advances M-I2/VR-5, SC-2).** `MapI::bundle_values_certified` issued a `Proven` `CapacityBound`
  after checking only the dimension instantiation (`dim ≥ requiredDim`), but the cited
  Clarkson/Thomas theorem also assumes **bipolar (±1) atoms** and **distinct items** — so a `Proven`
  tag could be obtained for a bundle of duplicates or non-bipolar vectors. The certified path now
  checks both before issuing the bound (`check_bipolar` → `NonAlphabetComponent`; `first_duplicate`
  → new `VsaError::DuplicateBundleItems`), and the margin `μ` plus the checked side-condition are
  **recorded in the bound's basis citation** so EXPLAIN/serialization expose exactly what the
  `Proven` tag rests on. Regression test refuses non-bipolar and duplicate inputs and still certifies
  distinct bipolar ones (mutant-witness A3-03); an existing capacity test that built identical
  undersized atoms was corrected to use per-item seeds so it still isolates the dimension condition.
- **WS6 — KC-2 baseline oracle fidelity (findings A6-01 H10, A6-04; M-002 well-posedness).** The
  Python baseline DSL read `Bin` as **unsigned** while the kernel/spec use **two's-complement**, so
  the benchmark's two arms computed different answers for the same prompt — e.g. `kc2-05`
  `swap(0b1011_0010 → 6-trit)` gave the baseline `+178` vs the kernel/spec `−78` (`0-00+0`),
  invisible because the oracle checked only result *shape*. `baseline.Bin.to_int` and the `Tern→Bin`
  swap are now two's-complement (`B_n = [−2^(n−1), 2^(n−1)−1]`), matching `binary.rs` — the worked
  example now yields `−78` in both arms. Added an `expect_value` field to `Task` (the independently
  computed integer) and a well-posedness test asserting each reference baseline's `to_int()` matches
  it, so a value-wrong reference or a future convention drift is caught (A6-04). Scoring stays
  shape-only (SC-5b symmetry). Remaining WS6 (tracked): A6-05 (LSP unsupported-swap diagnostic),
  A6-10/B2-04 (`exec` `allow_untrusted` guard), A6-11 (xtask kc4 precheck).
- **WS5 — `mycelium-select` content-addressing integrity (finding A5-01/B2-02 H9; advances
  RFC-0005 §3).** `SelectionPolicy::new` (and, via it, `Deserialize`) now rejects a rule predicate
  carrying a **non-finite `f64` literal** (`Predicate::literals_finite`, recursing through
  `All`/`Any`/`Not`), with a new `PolicyError::BadPredicateLiteral`. `NaN` and `±∞` both serialize
  to JSON `null`, so two materially different policies (`eps ≤ NaN`, never-matches, vs `eps ≤ ∞`,
  always-matches) would otherwise hash to the **same** `policy_ref` — collapsing the audit anchor
  recorded in `Meta.policy_used`. Regression test asserts all three non-finite forms are refused
  (and nesting is checked), citing A5-01 as its mutant-witness.
- **WS4 — `mycelium-l1` soundness + parser hardening (findings A4-01 H7, A4-02 H8; advances S5/G2,
  RFC-0007 §4.5).**
  - **Totality soundness (H7):** the structural totality checker classified a non-terminating
    function as `Total` and admitted it as `matured`, because a `Match` arm binder reusing an outer
    "smaller-than" variable's name was never dropped — stale smallness leaked into the arm body and a
    non-decreasing recursive call looked structural (`f(n,p)=match n{Z=>Z,S(m)=>match p{Z=>Z,S(m)=>
    f(m,p)}}` diverges yet was accepted). `descend_walk` now drops every binder a pattern introduces
    (recursively) for the arm body and restores it after, re-adding only the genuinely-smaller
    constructor sub-binders — mirroring the existing `Let`/`For` discipline.
  - **Parser DoS (H8):** the recursive-descent parser had no depth guard, so crafted deeply-nested
    input overflowed the host stack and aborted `myc-check` (the M-002 oracle) instead of returning
    an error. `parse_expr` is now depth-guarded (`MAX_EXPR_DEPTH = 256`), returning an explicit
    `ParseError`; bounding the parser bounds the AST depth, protecting the downstream
    typechecker/totality/elaborator passes transitively.
  - Regression tests for both (the divergent witness is `Partial` + `matured` refused; 2000-deep
    input returns `Err`, not a crash), each citing its finding ID as a mutant-witness.
  - Remaining WS4 (tracked): A4-03 (charge eval depth per call-frame), A4-04 (`Wf`-error-path test),
    and switching the reject corpus from `is_err()` to per-file expected-error-substring assertions.
- **WS2 — `mycelium-core` contract integrity (findings A6-02, B2-03, A1-01, A1-02, A1-03;
  advances M-I1…M-I4, the schema contract).** The JSON schema is now enforced on the Rust side too,
  closing the tampered-manifest vector:
  - `#[serde(deny_unknown_fields)]` on `ValueWire`/`MetaWire`/`ReconWire`, so an unknown wire field
    is **rejected**, not silently dropped — `additionalProperties: false` is now a real contract on
    both sides (A6-02). (`Bound` uses `#[serde(flatten)]`, which serde cannot combine with
    `deny_unknown_fields`; its integrity is enforced by `well_formed` below instead.)
  - `Bound::well_formed` now also checks **finiteness** (an infinite ε/crosstalk is a vacuous bound,
    A1-02) and the **basis constraints** — an `EmpiricalFit` must rest on `trials ≥ 1` with a named
    method, a `ProvenThm` must name its citation — so an evidence-free `Empirical` tag (`trials: 0`)
    is refused on deserialize (A6-02/B2-03). Fixed the stale `MetaWire` doc claiming `reconstruction`
    is "not carried" (A1-01/A6-07).
  - New unit tests (`bound.rs`) and wire-tamper regression tests (`serde_roundtrip.rs`), each citing
    its finding ID as a mutant-witness (A1-03).
  - Remaining WS2 (tracked, not yet done): A6-03 (broaden the emit-then-validate schema pinning to
    one example per enum/basis/layout), A6-06 (recon schema↔Rust conditional reconciliation), A6-08
    (sparsity `WfError` variant), A6-09 (cert `params` schema drift), A1-04/A1-05 (nits).
- **WS1 — `mycelium-numerics` honesty hardening (findings A2-01, A2-02, A2-03, A2-04, A2-06,
  A2-07, A2-08; advances VR-3/VR-5, SC-2).** A `Proven`/`Empirical` ε or δ that travels in a
  `Bound` is now a *true* upper bound under floating point, closing the headline honesty hole where
  `compose_error_bound` emitted `ProvenThm` on round-to-nearest f64 that could fall below the real
  bound:
  - New private `round` module: directed (outward) rounding (`add_up`/`mul_up`) via the Knuth/Møller
    two-sum and an FMA, rounding a bound-increasing result up **only when IEEE actually rounded
    down** — so an exact composition (e.g. `Exact ⊕ Exact`) stays exactly `0` and is not silently
    inflated to "approximate".
  - Every ε/δ composition rounds outward: `ErrorBound::{add,scale,mul}`, `AffineForm::radius`, the
    `mul` second-order remainder, `ProbBound::union`, and `ApRhlJudgment::seq`. Each `AffineForm`
    op also folds the magnitude of its own center/coefficient round-off into a reserved
    `ROUNDOFF_SYM`, so `radius` is a sound enclosure under f64 (A2-01).
  - The tier-i checker's tolerance is now **relative** (a few ULPs of the re-derivation) instead of
    an absolute `1e-12` that was vacuous for tiny bounds — a claim of `eps = 0` against a re-derived
    `~5e-13` is now correctly **rejected** (A2-02).
  - `AffineForm::uncertain` returns `Option`, refusing a non-finite center / non-finite or negative
    radius instead of silently collapsing infinite uncertainty to an exact form (A2-03, house rule
    2); `compose_error_bound` re-validates the composed magnitude and refuses an overflow to
    non-finite rather than emitting a fabricated `inf` bound (A2-04); `AffineForm::mul`
    `debug_assert`s its fresh-symbol precondition (A2-06).
  - Property tests strengthened to assert with **zero slack** over both deviation signs (A2-07) and
    new regression/refusal tests added, each citing the finding ID as its mutant-witness (A2-08).
  - Deferred within WS1 (tracked, not yet done): A2-05 (make the kernel-type fields private — a
    cross-crate API change, kept separate from this rounding fix) and A2-09 (composed-`Proven`
    citation provenance, Nit). The outward-rounding guarantee holds for all current call paths,
    which construct these types via `new`/`exact`/the composition methods.

### Changed (deep-review remediation — Wave 1)
- **Dev tooling — banked review lessons into the skills.** `dev-workflow/SKILL.md` gains a "Banked
  guards" section and `_shared/review-rubric.md` a "Recurring defect patterns (grep-first)" list, so
  the honesty-rule seams the review exposed (outward-rounded f64 bounds, fail-closed bound
  constructors, `deny_unknown_fields` + schema re-validation, depth-guarded recursive descent,
  ambiguous-encoding hashing, shadowing-aware analyses, mutant-witness tests) are caught while
  authoring and during review, not only in audit. Each guard cites the finding that motivated it.
- **RFC-0003 → Accepted (r3): §4.1 erratum** reconciling the §4 guarantee-tag table with its own
  "Net" line, resolving review findings **A3-01 / A3-02 (H4/H5)**. On a checked algebraic basis:
  `permute` is `Exact` for every model (the table's "Proven" conflated the permutation *operation* —
  an exactly-invertible coordinate shift — with sequence-decoding error growth, which belongs to the
  `bundle`/`unbind` path), and the HRR/FHRR bind/unbind cell splits into bind `Exact` (exact algebraic
  convolution / complex product) and unbind `Empirical` (the lossy approximate inverse — the residual
  weak link, unchanged). Append-only: the r2 table cells are preserved, §4.1 is authoritative. **No
  code tag changes** — `mycelium-vsa::matrix.rs` / `tests/matrix.rs` already followed the Net line;
  the non-citable "issue #61" rationale in the code comment is replaced by the §4.1 citation.

### Added (developer tooling — code enumeration / mapping)
- **`just map`** (advisory; `scripts/map.sh`): generates a crate-to-crate dependency graph
  (`cargo depgraph` → Graphviz, `cargo tree` fallback), per-crate module/item structure
  (`cargo modules`), and rustdoc including private items, under `target/map/` + `target/doc/`. Not
  part of `just check`. Function-level call graphs in Rust are partial (trait dispatch / generics) —
  use rust-analyzer's call hierarchy or `cargo-call-stack` for those.
- **`just api` / `just api-baseline`** (`scripts/checks/api.sh`, `scripts/api-baseline.sh`): a
  public-API **surface gate** wired into `just check`. It diffs each crate's surface against a
  committed snapshot (`docs/spec/api/<crate>.txt`) and fails on an unreviewed change — a guardrail
  for KC-3 and the A2-05 private-fields work. All tools are optional and **skip gracefully** when
  absent (installer adds them best-effort); snapshots are bootstrapped with `just api-baseline`.

### Added (advisory review artifact)
- **Deep review (2026-06):** `docs/reviews/2026-06-14-deep-review/` — a four-stage advisory
  review (correctness + test-quality, security audit, quality/style vs the house rules, and a
  QC/PE improvement roadmap) of the Phase-1/Phase-2 code at HEAD `e2d627e`. Report-only, gates
  nothing, changed no code. Verdict: strong, honesty-disciplined codebase (0 Critical); 11
  distinct High findings clustered at the honesty-tag/contract seams (numerics `Proven`-on-
  unrounded-f64, VSA matrix/capacity over-tagging vs RFC-0003 §4, a totality-checker soundness
  hole, an unbounded-recursion parser crash, a selection `PolicyRef` collision, and
  schema↔Rust contract leaks). Not registered in `docs/Doc-Index.md` (advisory, non-normative).

### Added (Phase-2 Batch H — schedule-staged packing selector + E3 wrong-layout differential)
- **M-250 (`mycelium-select` + `mycelium-core::Meta::with_physical`):** the **schedule-staged
  packing selector** (RFC-0004 §5; DN-01 Resolved; RFC-0005 §4). `bitnet_packing_policy` builds the
  fixed bitnet.cpp candidate set (`I2_S`/`TL1`/`TL2`) with an `Always → Cheapest` rule over the
  bits/element cost model; `select_layout`/`record_packing_layout` reuse the **one** E2-6 selection
  mechanism (`select_packing`) — adding only the `PackScheme → PhysicalLayout::TritPacked` record
  mapping — and emit the mandatory EXPLAIN. The exhaustive cheapest is `TL2` (1.67 b/w)
  deterministically; a first-class override forces `I2_S`/`TL1`; out-of-range overrides are explicit
  errors. The chosen layout is recorded on `Meta.physical` via the new `Meta::with_physical`, a
  **lossless** record builder (**M-I5**: touches only `physical`, leaving guarantee/bound/value
  untouched). Determinism + override + M-I5 losslessness are tested (`tests/packing.rs`).
- **M-251 (`mycelium-mlir::pack` + `run_with_layout` + `tests/wrong_layout.rs`):** the **E3
  wrong-layout soundness differential** (RFC-0004 §8; NFR-7; RR-12). A substrate byte-layout codec
  (`pack_trits`/`unpack_trits`/`relayout_trits`) gives each scheme a bijective trit↔byte encoding —
  the three bitnet schemes are mutually distinct, so reading a buffer under the wrong scheme
  misreads it (decoding is total, never a panic). `run_with_layout` extends the M-151 interp↔AOT
  differential to the packing stage: a **correctly-labeled** layout (packed-as == tag) is the
  identity and **validates** through the M-210 `ObservationalEquiv` checker; a **mislabeled** layout
  (packed-as ≠ tag) misreads the buffer and the same checker reports an explicit
  `NotValidated{ Diverged }` — the circuit-breaker fires (the layout record the M-250 selector chose
  is trusted *only because a wrong one is caught*). The true scheme used is the one M-250 actually
  selects, tying the soundness check to the selector it guards.
- **E1 perf-harness stub (`cargo xtask e1`):** times the substrate packing codec's pack/unpack
  round-trip per scheme — the build-phase confirmation that staging is cheap to materialize (the
  calibrated kernel benchmark awaits the native libMLIR/LLVM path; ADR-009). Honest framing: it
  reports numbers, the E1 verdict stays **not established** (VR-5; deferred to the native path).
- Phase-2 status: epic **E2-7 complete at the task level** → **all five Phase-2 exit-gate build
  conditions met** (numerics, full swap + shared checker, selection + EXPLAIN, Dense + VSA breadth,
  packing + reconstruction). KC-1…KC-4 re-run at the gate (phase-2.md §5): KC-1 confirmed (build,
  no regression), KC-3 holds (the packing codec landed in `mycelium-mlir`, not the trusted kernel;
  core gained only the tiny `with_physical` record), KC-4 unchanged (the layout check is the
  existing ~10 ns observational instance). KC-2 (LLM-survives-the-surface) and the RFC-0006
  ratification remain open but are **out of the Phase-2 exit-gate scope** (external/maintainer).

### Added (Phase-2 Batch G — Dense surface, VSA breadth, Dense↔VSA swaps, reconstruction manifest)
- **M-230 (`mycelium-dense`, new crate):** the typed dim-tracked `Dense{dim, dtype}` operational
  surface (RFC-0001 §4.1) — `DenseSpace` binds dim+dtype in the type; `add`/`sub`/`scale` are
  `Proven` with per-element relative ε (Higham Thm 2.2, side-conditions checked per element;
  BF16 carries the two-rounding composition `2⁻⁸ + 2⁻²³`); `neg` is `Exact`; `dot`/`similarity`
  are `f64` measurement helpers. Off-grid payloads, overflow, subnormal results, and approximate
  sources are typed explicit errors; a 20k-pair sweep per dtype exercises the bound (SC-2).
- **M-240/M-241/M-242 (`mycelium-vsa`):** the **full RFC-0003 §4 model breadth** — MAP-B
  (sign-rounded bundle), BSC (XOR bind, majority bundle, centered Hamming similarity), HRR
  (circular convolution; correlation unbind), FHRR (phasor phase algebra; explicit
  degenerate-bundle refusal), and SBC (one-hot-per-block sparse codes with the T1.3 placement:
  declared `Sparse{max_active}` class in the `Repr`, observed `SparsityObs` in `Meta`). The §4
  guarantee matrix is encoded as the single source-of-truth table (`RFC0003_MATRIX`) asserted
  model-by-model in tests; **HRR/FHRR unbind stays the pinned `Empirical` weak link** (T1.2).
  New honesty pattern: a declared **`EmpiricalProfile`** (regime + δ + trial count) backs every
  `Empirical` Value-level op and is exercised by exactly its declared trials in
  `tests/empirical_profiles.rs`; outside-profile calls are explicit refusals. **RR-13 enforced:**
  MAP-B bundle nesting beyond depth 1 is the explicit `NestedBundleUnsupported` error.
- **M-231 (`mycelium-cert::dense_vsa`):** Dense↔VSA swaps (RFC-0002 §5) — bipolar `Dense{n,F32}`
  vectors encode as MAP-I superpositions over a deterministic versioned codebook (a genuine
  bipolar bundle, so the T0.2 capacity theorem applies); decode is provenance-gated signed
  correlation. The δ certificate's basis is derived, never asserted: `ProvenThm` iff
  `vsa_dim ≥ requiredDim(n, δ)` (the M-131 checked instantiation), `EmpiricalFit` iff the
  10⁴-trial profile covers the instance, an explicit `InsufficientCapacity` type error elsewhere.
  The **M-210 checker's δ-side lands** (the recorded `Incomplete` placeholder retired):
  `ProbabilityBound` certificates discharge by tier-i union-bound claim-vs-certificate plus
  deterministic re-derivation equality. `CertifiedSwapEngine` + the SC-3 global test cover the
  new rows (SC-2 satisfied for the new swaps).
- **M-260 (`mycelium-core::recon` + `mycelium-vsa::recon`):** the **reconstruction manifest**
  (RFC-0003 §6; `reconstruction-manifest.schema.json`, the ratified name) — `ReconInfo` with a
  validating constructor/deserializer (compositional ⇒ recipe; resonator ⇒ probabilistic-only,
  FR-C2), carried in the ratified `Meta.reconstruction` field (`with_reconstruction`); the
  submodule-side `reconstruct_role` executes the manifest with the threshold made explicit.
  Acceptance: the compositional path **recovers a novel combination** never stored in any
  codebook (the §6 exit criterion), wire-round-tripped end to end.
- Phase-2 status: epics **E2-1, E2-2, E2-5 complete at the task level**; the Phase-2 exit gate
  now waits only on Batch H (M-250 packing selector → M-251 E3 wrong-layout differential).

### Changed (RFC-0007 r3 — `for` spelling adopted)
- **RFC-0007 §4.8 → r3**: the bounded-iteration spelling `for x in xs, acc = init => body`
  moves from *provisional* to **adopted** (maintainer decision, 2026-06-10) — committed now
  rather than held pending a KC-2 ablation run. The kc2-09/kc2-10 benchmark tasks remain as
  measurements of the choice, not its gate; like all v0 surface syntax it stays under RFC-0006
  §1's global KC-2 gate, and revisiting it later is an explicit recorded decision (append-only).
  Wording updated in DN-03 §2, Lexicon Reference, Example-Programs note, `mycelium.ebnf`, the
  prototype doc-comments, and the KC-2 tasks docstring.

### Added (DN-03 — lexicon amendment; resolves ADR-012 §7.5/§7.6)
- **DN-03** (Resolved): amends DN-02 (append-only) through the three-test gate — **adopt**
  `consume` and `grow` (Surface), **decline** `embody` (inherent methods keep the conventional
  `impl`), **reserve** `for` (the RFC-0007 §4.8 bounded-iteration keyword). Ratifies the
  **one name per term** (flat) — **rejecting ADR-012 §7.6's canonical+alias scheme** as needless
  surface area (the "content-addressing makes a second spelling free" benefit is speculative; two
  labels per concept to keep in sync is a real cost now). Ratifies the single Runtime names
  against RFC-0008 §4.5's grounded meanings: `hypha`, **`fuse`** (RT6 is genuine merge —
  `anastomose`/`weave` dropped), `xloc`, **`cyst`** (encystment = the dormant resumable form;
  `cyst(…)` constructor-style like `spore`), **`graft`** (resolves the `myco` collision with the
  language family name), **`mesh`**, `forage`, **`backbone`** (was `rhizomorph`), **`tier`** (was
  `dimorph` — the canonical behavior is interpreted↔native tiering), `reclaim`. `reclaim` scope
  clarified (runtime units, never memory). Runtime vocabulary stays reserved-not-active. Lexicon
  Reference, Example-Programs note, and RFC-0008 §3/§4.2/§4.4/§4.5 updated to the single names;
  Doc-Index gains the DN-03 row.

### Added (ADR-013 — `spore` is the deployable unit; resolves ADR-012 §7.4)
- **ADR-013** (Accepted, maintainer deliberation 2026-06-10): `spore` = the
  **content-addressed deployable unit** — a hash-identified DAG of code (ADR-003 definitions,
  ship-by-hash per T4.3), values (with `Meta` intact), the RFC-0003 §6 **reconstruction
  manifest** as one digest-referenced component, and artifact metadata. The narrow ratified
  sense is the **degenerate case**: `spore(v)` constructs the single-value spore (the manifest
  for `v`); the schema name `reconstruction-manifest` is unchanged. Grounded in T4.3/T4.4
  (Nix/OCI/Wasm/Unison convergence on content-addressed artifact DAGs).
- **RFC-0003 → Accepted (r2)**: §6 scope note only — manifest contents, schema, and guarantees
  unchanged. **RFC-0008 R8-Q5** resolved at the scope level (schema/signing/germination contract
  remain the R2 implementation stage's obligation). Lexicon-Reference `spore` flag resolved;
  ADR index gains 012/013 rows.

### Changed (RFC-0007 r2 — bounded iteration; resolves ADR-012 §7.2)
- **RFC-0007 §4.8 (new, r2)**: bounded iteration as **elaboration-defined sugar** over
  structural recursion — no new kernel node. Normative content = the desugaring to a synthesized
  self-recursive helper over *linearly recursive* (nil/cons-shaped) data, classified `Total` by
  the existing §4.5 checker with zero extension (bounded **by construction**: values are finite
  and acyclic). Provisional spelling A — `for x in xs, acc = init => body` — ships in the
  non-normative prototype grammar (`for` reserved, recorded in DN-03); named-args `fold` is the
  planned L2 library form; the ratified spelling is **KC-2-evidence-gated** (T3.6).
  `while`/`loop`/`break`/`continue`/`return` stay excluded and **unreserved**, with *teaching
  diagnostics* where they already error (parse-level juxtaposition + check-level unknown name).
- **Prototype** (`crates/mycelium-l1`): `for` through the whole pipeline — lexer/parser
  (+ teaching diagnostics), T-For with explicit linear-shape refusals, totality (a `for` adds no
  recursion), an **iterative** spine-walk evaluator (long folds cost fuel, never host stack),
  elaboration `Residual` (Fix is outside the evaluation-complete fragment); EBNF + conformance
  corpus (`accept/11`, `reject/08`). **KC-2**: tasks kc2-09 (`for`) / kc2-10 (explicit
  recursion) form the runnable iteration-spelling ablation pair. 44 crate tests green.

### Added (RFC-0008 + Research Pass 4 — the Runtime tier, grounded)
- **Research Record 04** (`research/04-runtime-concurrency-RECORD.md`; findings **T4.1–T4.6**):
  the fourth research pass, grounding the Runtime tier ADR-012 §7.3 flagged as aspirational —
  concurrency units & structured lifetimes (Erlang isolation, nurseries, Kahn/LVars determinism,
  CakeML clocked-semantics extension), state merge & meshes (CRDT convergence, session types,
  epidemic protocols), mobility & placement (Unison ship-by-hash, the Legion
  placement-is-never-semantics separation, Reactive-Streams backpressure, work-stealing bounds
  with side-conditions), durability (CRIU's exception catalogue vs durable-execution's
  determinism requirement; Nix/OCI/Wasm content-addressed artifacts), failure & supervision
  (OTP, FLP, φ-accrual, Waldo et al.), and mode switching (verified deoptimization, CoreJIT).
  Primary-source verified with per-target uncertainty registers; three explicit novelty flags
  (no found precedent: determinism-gated checkpointability; learned-placement-as-inspectable-
  policy; per-value guarantee tags across a distribution boundary).
- **RFC-0008 — Runtime & Concurrency Execution Model** (Draft): the runtime model the Runtime
  vocabulary presupposed, built on Pass 4. **RT1–RT7 runtime invariants** extend S1–S6 to
  concurrency/distribution: values move & state is never shared (RT1); the deterministic
  fragment is the default with *sequential reference semantics* — NFR-7 extends to concurrency
  via the M-210 checker (RT2); nondeterminism is reified as RFC-0005 policies — placement
  becomes the **third site** of the one selection mechanism (RT3); partial failure is explicit,
  distribution transparency forbidden (RT4); runtime guarantees (delivery/convergence/failure
  suspicion) are tagged on the same lattice with `ProbabilityBound`s (RT5); fusion is lawful
  semilattice merge — payload joins, guarantee meets (RT6); runtime lifetimes are structured —
  *a leaked task is not expressible*, extending LR-9 (RT7). RFC-0004's per-node model is
  extended, not changed; the Runtime vocabulary is grounded (§4.5 operational-meaning table)
  but stays **reserved, not active syntax**, pending DN-03 + implementation RFCs. The `spore`
  scope reconciliation (ADR-012 §7.4) and name ratification are deliberately left to the
  RFC-0003 revision and DN-03 respectively. Indexes updated (`docs/rfcs/README.md`,
  `docs/Doc-Index.md`, Lexicon-Reference status notes).

### Added (L1 execution: evaluator, elaboration, three-way differential)
- **L1 fuel-guarded evaluator** (`crates/mycelium-l1/src/eval.rs`; RFC-0007 §4.6): a big-step
  environment machine mirroring M-110's contract — CakeML-style clocked semantics (explicit
  `FuelExhausted`, never a hang; T3.4), dispatching through the *same* trusted prim registry and
  certified binary↔ternary swap engine as the L0 paths (NFR-7). Runs the full checked surface
  (data values, flat `match`, recursion); the stage-0 **dynamic guarantee-index check**
  (RFC-0007 §4.3): asserting `@ g` stronger than a value's tag is an explicit
  `GuaranteeTooWeak` — an annotation may only weaken, never upgrade (VR-5). A separate explicit
  recursion-**depth guard** (`DepthExceeded`) keeps deep recursion an error, never a host stack
  overflow. Checker-unreachable states are explicit `Stuck` errors, never panics (S5/G2).
- **Elaboration to L0 on the evaluation-complete fragment** (`crates/mycelium-l1/src/elab.rs`;
  RFC-0007 §4.6): acyclic calls inline (CBV order preserved via `Let` bindings); bodies must
  reduce to `Const/Var/Let/Op/Swap` residue; recursion (`Fix`), `match`/`if`, data construction,
  and dynamic guarantee indices are explicit **`Residual` refusals — never a partial artifact**.
  Includes the shared surface→kernel bridge (literals, repr resolution) and the documented v0
  **policy-name reference** stand-in (deterministic, domain-separated; honest about deferring
  RFC-0005 name→policy-object binding) shared by both execution paths.
- **The RFC-0007 §4.6 differential** (`crates/mycelium-l1/tests/differential.rs`; NFR-7): on a
  10-program fragment corpus, **L1-eval ↔ elaborate→L0-interp ↔ AOT** agree on the observable
  (`repr + payload + guarantee`), with every agreeing pair validated through the **M-210 shared
  TV checker** (`ObservationalEquiv`) and a control asserting the checker rejects a genuinely
  divergent pair. Outside-the-fragment behavior is pinned too: elaboration refuses (`Residual`)
  while L1-eval runs — including a `Total`-classified structural recursion that terminates and a
  `Partial` one that exhausts fuel explicitly. 31 crate tests; `just check` green.

### Added (KC-2 harness)
- **KC-2 LLM-leverage harness** (M-002 structural deliverable; Foundation §6 P0.2; SC-5b; G10):
  `experiments/mycelium_experiments/kc2/` — the **fixed 8-task benchmark** (minimal Mycelium
  surface fragment vs a **Python-embedded DSL baseline**, both arms carrying checked reference
  solutions that prove the benchmark well-posed), the `myc-check` CLI oracle
  (`crates/mycelium-l1/src/bin/myc-check.rs`: parse / typecheck / task-signature conformance with
  distinct exit codes — no AI in the judging loop, S6), and the generate→check→feedback harness
  measuring **syntactic validity**, **first-attempt type-check pass rate** (the SC-5b number),
  and **edit-to-fix iterations**. *Running* the experiment remains blocked on LLM API access
  (the documented M-002 external blocker); the report hard-codes
  `verdict: not established` — never pre-written (VR-5). Baseline-arm execution is in-process
  `exec` and documented as requiring a disposable sandbox for untrusted model output. 8 pytest
  tests; `just check` green.

### Added (L1 static analysis + lexicon integration)
- **L1 typechecker + structural totality checker** (`crates/mycelium-l1`, RFC-0007 §4.4/§4.5):
  the v0 monomorphic typechecker over the data registry (declarations-as-registry), exhaustiveness
  checked (W7, never assumed), representation-typed literals, generics/`spore`/`wild` as explicit
  refusals; a Foetus-style structural-descent totality classifier whose verdict gates `matured`
  (mutual recursion stays Partial — R7-Q3). 8 tests; clippy clean.
- **Lexicon integration & architect review** (ADR-012 §7; `Lexicon-Reference.md`,
  `Example-Programs-Reference.md`, `Doc-Index.md`): verified the maintainer's three new lexicon
  documents against the corpus and integrated them. **Applied:** de-conflicted the lexicon
  "L1/L2/L3" tier labels (which collided with RFC-0006's language layers L0–L3) → renamed
  **Surface / Runtime / Formal**; fixed example bracket typos; added grounding notes. **Flagged for
  the maintainer (ADR-012 §7):** the Runtime tier (`hyph`/`anas`/`xloc`/…) is an *aspirational,
  ungrounded* concurrency/distribution model needing a Runtime RFC (RFC-0008) + research Pass-4 and
  reconciliation with RFC-0004; imperative `loop`/`while` contradicts the functional core
  (RFC-0007 §6); `spore` scope drifted from RFC-0003's reconstruction manifest; new Surface terms
  (`consume`/`embody`/`grow`) need a DN-02 amendment through the three-test gate (`embody` weakest);
  several short forms (`sclrt`/`cmn`/`anas`/`myco`) recommended for refinement; example
  bound-kind/partiality corrections. No contradictions found with ADR-010/011, the guarantee
  lattice, or content-addressing.

### Changed (RFC-0006 language-layer requirements)
- **RFC-0006 → r3 (Draft): two foundational language requirements** (maintainer direction;
  grounded in T3.5). **S6 self-sufficiency / AI-independence** — Mycelium is a complete software-
  engineering language whose parser/checker/elaborator/interpreter/AOT path are ordinary
  deterministic software runnable with **no AI/LLM in the loop**; models are an optional
  co-authoring convenience, never a runtime/compile-time/semantic dependency (remove every model
  and the language still builds, checks, runs, and reproduces bit-for-bit). This bounds KC-2: it
  can only choose the L3 surface, never make the language *need* a model. **LR-9 memory safety by
  construction** — Rust-grade safety *outcomes* without the borrow checker: value semantics
  removes use-after-free/data-races/double-free from the model, the language exposes no manual
  alloc/free (automatic deterministic reclamation — Perceus + region inference), the sole leak
  vector (external resources) is closed by the affine `Resource` kind, and any unsafe op is
  denied-by-default + lexically marked — *in safe Mycelium a memory leak is not expressible*. New
  open question Q8 (reclamation mechanism, cycle handling, `unsafe` spelling).

### Added
- **L1 grammar infrastructure + parser prototype** (`docs/spec/grammar/`, `scripts/checks/grammar.sh`,
  `crates/mycelium-l1`; RFC-0006 §4.3; **non-normative until RFC-0006 ratifies**): the WebAssembly-spec
  pattern (T3.1-B) made real. **`docs/spec/grammar/mycelium.ebnf`** — the normative v0 surface grammar
  in W3C notation (not ISO 14977), over the ratified DN-02 vocabulary (`colony`, `use`, `type`,
  `trait`, `fn`, `matured`, `let`/`in`, `if`, `match`, `swap`, `wild`, `spore`, `Substrate{…}`, the
  `T @ Strength` honesty index, representation-typed literals). **A conformance corpus** of 10
  `accept/` + 7 `reject/` `.myc` programs, each with an explanatory header — the corpus is the ground
  truth, not any single parser. **`grammar.sh`** (wired into `just check`/CI) structurally validates
  the artifacts; **`mycelium-l1`** is the real parser gate — a hand-written, dependency-free lexer +
  recursive-descent parser producing an inspectable AST, with `tests/conformance.rs` asserting every
  `accept/` parses and every `reject/` fails with an **explicit `ParseError` (never a panic, never a
  silent accept** — S5/G2). The lexer disambiguates the one tricky token (`<` opening a ternary
  literal vs a type-arg list) by lookahead; a malformed ternary literal is an explicit error. First
  increment of the L1 track (RFC-0006 §3) — typechecker, Maranget match compiler, structural totality
  checker, and L0 elaboration land next.
- **DN-02 (Resolved) — Fungal Lexicon & Reserved-Word Set** (`docs/notes/DN-02-Fungal-Lexicon-and-Reserved-Words.md`;
  feeds RFC-0006 §4.3): the surface vocabulary of Mycelium-the-language, drafted then **ratified by
  the maintainer** the same day. Codifies the **naming law** as a three-test gate (T-map fidelity /
  T-illuminate teaching-value / T-learn dual-readability) — *theme where the fungal metaphor is
  accurate and illuminating; keep conventional where a borrowed term is clearer to learn and read*.
  Ratified themed set: `colony` = module, `network` = the content-addressed dependency web,
  `substrate` = the affine external-resource kind, `spore` = reconstruction manifest (schema stays
  `reconstruction-manifest`), `matured` = promoted stable/AOT component, `wild` = the
  denied-by-default unsafe block. Ratified conventional: `let`, `fn`, `type`, `trait`, `match`,
  `if`, `swap` (a native corpus term), `use`, the guarantee tags; guarantee annotation `T @ Exact`.
  Literals universal-until-elaboration (no cross-family defaulting). Language name = **Mycelium**
  (shared). Status **Resolved** — the set is now frozen into the grammar artifacts.
- **Research Pass 3 — language-layer targets T3.1–T3.6** (`research/03-language-layer-RECORD.md`;
  grounds RFC-0006 Q1–Q6): four parallel primary-source deep-dives. Headlines: every surveyed
  kernel (GHC Core, Lean, Coq, Unison) keeps ~10–16 expression nodes with **data declarations in
  a registry/environment layer** and Unison gives the cycle-hashing recipe (T3.1); the guarantee
  lattice is formally an **integrity lattice** — silent upgrade = IFC's *endorsement*, gated here
  by a checked certificate — and graded coeffects (Granule-style) subsume flat labels, with
  refinements reserved for certificate side-conditions (T3.2); GHC levity polymorphism's two
  restrictions + monomorphization give the LR-5 restriction set (T3.3); divergence-only effect
  tracking (Koka's `div`, degenerate) + Lean's `partial`-opaque split + CakeML clocked semantics
  settle Q4/LR-4 (T3.4); ownership/borrowing confirmed **not applicable** to value semantics
  (Hylo/Swift), linearity deferred to a reserved affine `Resource` hook (T3.5); and the measured
  LLM evidence (MultiPL-E/T, MTOB, SynCode, grammar-aligned-decoding distortion) yields a
  five-condition KC-2 design with an explicit falsification threshold (T3.6). Honest-uncertainty
  register included; two pieces flagged **novel with no found precedent** (grading + runtime
  certificates; totality gating AOT promotion). **RFC-0006 revised to r2 (still Draft)**: §8
  positions per question, new Q7; §4.2 postures updated.
- **RFC-0006 (Draft) — Surface Language, Grammar & Term-Language Layering**
  (`docs/rfcs/RFC-0006-Surface-Language-and-Term-Layering.md`; SPEC §10.2's deferred "later RFC"):
  the deliberation artifact that nails down the language architecture *before* implementation
  accretes a de-facto one. Fixes now: the **L0–L3 layering** (Core IR → kernel calculus → surface
  term language → KC-2-gated projection layer; only L0/L1 trusted — KC-3), the **syntactic honesty
  invariants S1–S5** (never-silent swap stays lexically visible through every layer; guarantee
  tags are part of every binding's observable interface; content-addressed identity; inspectable
  elaboration; explicit partiality), the **capability targets LR-1…LR-8** ("Rust-class and beyond"
  made checkable: ADTs, coherent traits, content-addressed modules, totality-postured recursion,
  plus the beyond-Rust core — Repr polymorphism and guarantee-indexed types; ownership/borrowing
  flagged as likely-not-applicable to a value-semantics substrate), and the **grammar/spec
  discipline** (EBNF + machine-readable grammar artifacts + conformance corpus, mirroring the
  schema pattern). Defers exactly one thing, deliberately: the concrete L3 syntax, which the
  corpus already gates on the KC-2 experiment (M-002; RR-3). Status **Draft** — ratification is a
  maintainer decision. Indexed in `docs/rfcs/README.md`, `docs/Doc-Index.md`, SPEC §10.2.
- **Selection-policy language + mandatory EXPLAIN + site wiring** (`mycelium-select` — a new
  crate — plus the `mycelium-lsp` EXPLAIN channel, **M-220/M-221/M-222**, Phase 2; RFC-0005;
  ADR-006; SC-5): realizes RFC-0005 §2's decision verbatim. **M-220:** `SelectionPolicy` — an
  ordered decision table (`Predicate` over queryable `Meta`: dtype, guarantee, ε bounds, sparsity —
  *exact* metadata, never sampled estimates) over a finite `Candidate` set (`Repr` | `PackScheme`),
  with an explicit `CostModel` (cost = weight × storage **bits**, a real declared unit) and a
  mandatory default arm — total and terminating *by construction* (validated constructor; wire
  forms re-validated on deserialize); deterministic (first-match precedence; `Cheapest` ties break
  to lowest index); **content-addressed** (`policy_ref()` = hash of the canonical serialization —
  RFC-0005 §3); first-class deterministic overrides. **M-221:** every selection emits a
  serializable `Explanation` `{policy ref, inputs considered, cost of every candidate, matched
  rule, chosen, override state}`; `explain(policy, inputs)` is total and deterministic; the
  `mycelium-lsp` facade surfaces it as the fifth artifact kind (`analyze_with(node, &PolicyRegistry)`
  re-derives the trace at each resolvable swap site and raises a `policy-divergence` warning when
  the node's target disagrees with the policy's choice — surfaced, never silent). **M-222:** one
  mechanism, two sites — `select_swap_target`/`select_packing` are thin adapters over the single
  `select` (a wrong-kind candidate at a site is an explicit refusal); the wiring test drives an
  auto-selected target through the real interpreter + `CertifiedSwapEngine` and the result records
  `Meta.policy_used = PolicyRef` (the packing site is consumed by E2-7/M-250). 15 new tests across
  policy semantics, EXPLAIN, LSP surfacing, and the swap-site wiring.
- **KC-4 cert-overhead measurement + SC-3 global exit** (`xtask kc4` +
  `mycelium-cert/tests/sc3.rs`, **M-212**, Phase 2; Foundation KC-4; SC-3; RFC-0002 §2):
  `cargo run --release -p xtask -- kc4` times every implemented swap kind and its M-210
  certificate check (no bench dependency; refuses debug builds — their numbers would be dishonest
  to record). **Measured 2026-06-10** (containerized runner, indicative): bijective check ≈1.6–1.7 µs
  (~1.3× its ~1.3 µs swap — it re-derives the swap), bounded `Dense{768}` check ≈2.0 µs (~0.13× its
  ~16 µs swap), observational pair ≈10 ns. Honest verdict: per-swap checking costs the same order
  as the swap itself — the KC-4 downgrade path is **not triggered on this evidence**; a *ratified*
  numeric budget remains a pending maintainer decision (recorded in `phase-2.md` §6.7, not
  pre-written as "within budget"). The SC-3 global test pins the whole surface: every implemented
  legal-pair row emits a certificate that validates through the one checker, and every
  rejected/unimplemented row is an explicit error — never silent, anywhere.
- **First Bounded/lossy swap — Dense `F32 → BF16`** (`mycelium-cert::dense`, **M-211**, Phase 2;
  RFC-0002 §3/§5; ADR-010 §1): establishes the split regime (ADR-002) alongside the bijective
  binary↔ternary class. `dense_f32_to_bf16` rounds to-nearest-even and emits a
  `SwapCertificate::Bounded` carrying the proven per-element relative rounding bound
  `{Rel, u = 2^−8}` with a `ProvenThm` basis — the strength is *derived from how the bound was
  obtained, never asserted* (RFC-0002 §3), and the theorem's side-conditions are **checked per
  element**: finite, exactly an `f32`, zero-or-normal, no overflow on rounding; each violation is
  a typed explicit `SwapError` (`NonFinite`/`NotAnF32`/`SubnormalUnsupported`/`RoundOverflow`),
  never a silent coercion. Approximate sources are refused (`ApproximateSource`) until the E2-1
  composition rule exists — refusal, never fabrication. The certificate **validates through the
  M-210 shared checker**, a tampered conversion is caught (tier-i rejection), and a new
  `CertifiedSwapEngine` serves the complete certified surface (bijective + bounded + identity),
  explicit `UnsupportedSwap` for everything else. 11 tests incl. a 20k-sweep soundness property
  for the `2^−8` bound and ties-to-even spot checks.
- **Single shared translation-validation certificate checker** (`mycelium-cert::check`, **M-210**,
  Phase 2; RFC-0002 §2; RFC-0004 §3; T1.1): one `check(A, B, R, claimed, evidence)` answering "does
  artifact B refine reference A under relation R within the claimed `{ε,δ,strength}`?" — build once,
  use twice. Three `RefinementRelation` instances: **Bijection** (the M-120 binary↔ternary cert —
  lemma reference + `legal_pair` side-condition checked, then structural *re-derivation equality*
  against B), **BoundedSimilarity** (lossy swaps — the measured A↔B deviation and the claim are both
  re-validated through the E2-4 `mycelium-numerics` tier-i checker; a claim tighter than its
  certificate, a certificate tighter than the measured instance, or a strength upgrade past the
  basis (VR-5) is rejected), and **ObservationalEquiv** (interp↔AOT over the NFR-7 observable —
  the **M-151 differential is folded in** as an instance and now validates every corpus pair
  through this checker). TV incompleteness is an explicit `NotValidated{reason, fallback}` with the
  `UseReference` fallback path — **never a silent pass** (RFC-0002 §2). `mycelium-numerics` now
  exports `basis_strength` (the M-I2…M-I4 basis→strength mapping) for certificate consumers.
  16 checker tests cover all three instances and every refusal path.
- **Interpreter composes approximate inputs honestly** (`mycelium-interp::prims`, **M-204**, Phase 2;
  RFC-0001 §4.7; ADR-010): retires the Phase-1 blanket `ApproxCompositionUnsupported` refusal for
  composable inputs. `exact_result` → `compose_result`: exact-over-exact stays `Exact`/`bound=None`
  (M-I1); over an approximate input it composes per a per-prim `ApproxRule` — `core.id` passes the
  bound through verbatim (citation preserved), `trit.add`/`sub`/`neg` carry the sound affine ε
  composition via `mycelium_numerics::compose_error_bound` (strength `meet`s to the weakest input,
  basis re-derived so M-I2…M-I4 hold), and `bit.*` / `trit.mul` still refuse (no defined ε rule —
  honest, never a fabricated bound). Five new golden tests cover additive ε composition (Proven⊕Proven
  → Proven, ε sums), negation (ε preserved), `core.id` passthrough, meet-down to Declared, and the
  explicit `trit.mul` refusal; the Phase-1 `bit.not` refusal test still holds. **Closes the documented
  Phase-1 honesty gap** (the interpreter previously could not compose approximate inputs).
- **Verified-numerics foundation — two bound kernels + shared certificate + tier-i checker**
  (`mycelium-numerics`, **M-201/M-202/M-203**, Phase 2; ADR-010; RFC-0001 §4.7; SPEC §10.7): a new
  crate realizing ADR-010's two-kernels-one-certificate decision, deliberately *outside*
  `mycelium-core` (KC-3/SoC — the trusted kernel stays small; numerics is a certificate consumer).
  **`error`** composes ε through **affine arithmetic** — `AffineForm` (`x₀ + Σxᵢ·εᵢ`) with *exact*
  linear ops (correlated noise symbols cancel) and a sound `mul` (second-order remainder onto a fresh
  symbol), and the scalar `ErrorBound{eps,norm}` projection (`add`/`sub`/`neg`/`scale`/`mul`).
  **`prob`** composes δ through the **union bound** (`min(1,Σδ)`) and the apRHL `[SEQ]` rule
  (`ApRhlJudgment` — ε adds as the `e^ε` factors multiply, δ adds, both saturating). They meet at the
  shared **`Certificate{eps,delta,strength}`** (`strength` by `meet`), with a **tier-i Rust checker**
  (`check_error_claim`/`check_union_claim`) that re-derives a composition and **rejects any claim
  tighter than the re-derivation** — never a silent pass (RFC-0002 §2) — and the one sanctioned
  cross-kernel rule `accuracy_to_probability` (ADR-010 §4). The three normative properties
  (**Soundness, Monotonicity, Determinism**; RFC-0001 §4.7) are property-tested over 20k-trial inline
  loops (Phase-1 house style — no `proptest`/`rand` dep); 17 tests green, clippy `-D warnings` clean.
- **Phase-2 plan + epic decomposition** (`docs/planning/phase-2.md`; **Phase 2**; Foundation §6;
  SPEC §10.7–§10.10): decomposed the seven Phase-2 epics (#28–#34) into 18 issue-coupled `M-2xx`
  build tasks (#48–#65), created as sub-issues of their epics and joined into `tools/github/idmap.tsv`.
  The plan mirrors `phase-1.md`: readiness table, batch/parallelization structure, the critical path
  (the ADR-010 ε/δ numerics kernels as keystone — they gate every honest approximation downstream),
  and an honest Phase-1→2 re-run of the kill criteria (KC-1 confirmed/no-regression; KC-2
  open/blocked on external LLM access; KC-3 holding — numerics + selection land as their own crates
  to keep the kernel auditable; KC-4 first-measurable when the shared checker lands). Planning
  artifact only — cites the corpus, introduces no requirements.
- **MLIR→LLVM AOT path — ternary-dialect skeleton + runnable AOT artifact** (`mycelium-mlir`,
  **M-150**, Phase 1; RFC-0004 §2/§6; ADR-007; T1.5): `dialect::emit` renders the lowered A-normal
  form as a textual `ternary`-dialect MLIR-style module (one op per binding, all attributes inline —
  the no-opaque-pass anchor), and `aot::run` is the **runnable artifact for the subset** — an
  independent big-step env-machine that executes the lowered ANF directly. Native libMLIR/LLVM
  codegen is **deferred** (Phase 3 matures it; honestly scoped as a textual skeleton + execution
  model, not a compiler).
- **Interp↔AOT differential** (`mycelium-mlir` tests, **M-151**, Phase 1; NFR-7; VR-4; RR-12): a
  harness runs a kernel corpus under both the M-110 reference interpreter (small-step substitution)
  and the M-150 AOT artifact (big-step env-machine over the lowered ANF) and asserts **observable
  equivalence** (repr + payload + guarantee); divergence fails CI. The two paths differ in IR shape
  and evaluation strategy, sharing only the trusted primitive/swap semantics — so the differential
  catches lowering/scheduling/ordering divergence (the cheap baseline preceding per-artifact
  translation validation in Phase 2). A control test confirms the harness discriminates.
- **LSP feedback facade** (`mycelium-lsp::feedback`, **M-140**, Phase 1; FR-S5; Foundation §5.8;
  SC-5): `analyze(node)` exposes the **four** semantic-feedback artifact kinds over one surface —
  (1) typecheck/invariant **diagnostics** (linter), (2) **swap certificates** for statically-
  resolvable swap sites, (3) per-value **bound/guarantee annotations**, (4) **lowering-stage dumps**.
  A failed/unsupported swap is surfaced on the diagnostics channel, never silent. Verified by a
  **scripted-client** integration test driving all four channels (incl. a Proven bound, an
  out-of-range swap, and invariant violations).
- **Canonical formatter** (`mycelium-core::lower::format` + `mycelium-lsp::fmt`, **M-142**, Phase 1;
  RFC-0001 §4.8; ADR-003): a canonical textual normal form that **α-normalizes binder names**
  (`v0, v1, …`), so definitions differing only in names render to identical text and share one
  `content_hash` — reformatting is a projection that never changes content-addressed identity (tested:
  renamed defs format identically and hash equally; formatting leaves identity untouched; free
  variables keep their names).
- **Invariant linter** (`mycelium-lsp::lint`, **M-141**, Phase 1; SC-3; G2; FR-M3; VR-5): static,
  inspectable lints over a Core IR program, emitted as `Diagnostic`s for authoring tools — `implicit-swap`
  (an `Op` mixing paradigms implies a conversion that must be an explicit `Swap`), `unverified-bound`
  (a `Declared` value must always be surfaced, never silently trusted), `placeholder-policy` (a swap
  citing a stub rather than a real `PolicyRef`), and `free-variable` (an open term). Each lint has a
  positive and a negative test. Introduces the toolchain crate `mycelium-lsp` (FR-S5), kept out of
  the auditable kernel (KC-3 — depends on core/interp/cert, nothing depends on it).
- **Inspectable lowering — ≥2 dumpable/diffable stages** (`mycelium-core::lower`, **M-112**, Phase 1;
  RFC-0004 §5/§6; SC-4; WF5): a backend-agnostic lowering pipeline. `stages(node)` returns **`core`**
  (the canonical Core IR tree dump) → **`substrate`** (an A-normal form flattening nested
  `Op`/`Swap`/`Let` to a linear binding list — the pre-codegen shape backends consume), each binding
  whose result repr is statically known (`Const`, `Swap` target) annotated with its **scheduled
  `PhysicalLayout`** (the default schedule, `I2_S` for ternary; RFC-0004 §5 / DN-01). Dumps are
  canonical (deterministic — structurally identical programs render identically, SC-4) and `Meta`
  guarantee tags survive lowering (WF5). `Op`-result layout is left explicitly unannotated (no
  operator typing yet — the omission is honest, not silent; G2).
- **Cleanup / item memory** (`mycelium-vsa::cleanup`, **M-132**, Phase 1; FR-S4; RFC-0003 §3): a
  labelled associative memory (`CleanupMemory`) that snaps a noisy query — an *approximate* `unbind`
  result or a `bundle` decode — to the nearest stored atom by similarity, returning a `Match { label,
  index, confidence, margin }`. The confidence (match cosine) and margin (gap to the runner-up) make
  approximate unbind *usable* and *inspectable* (the retrieval decision is reported, never a hidden
  nearest-neighbour pick; G2). Tested incl. the role⊗filler record-decode use case (bundle two bound
  pairs, unbind by a role, clean up to the right filler).
- **MAP-I bundle capacity bound — `Proven` via checked instantiation** (`mycelium-vsa::capacity`,
  **M-131**, Phase 1; RFC-0003 §5; ADR-010; SC-2; KC-1): `required_dim(m, δ) = ⌈(2/μ²)·ln(m/δ)⌉`
  (μ=0.1) and `proven_capacity_bound` / `MapI::bundle_values_certified`, which attach a **`Proven`**
  `CapacityBound` (basis `ProvenThm`, citing Clarkson-Ubaru-Yang 2023 / Thomas-Dasgupta-Rosing 2021)
  **iff** the checked side-condition `dim ≥ required_dim` holds — exactly the M-001 axiomatized-
  theorem + checked-instantiation pattern (the formula is cited, not re-proven). An undersized
  dimension returns an explicit `InsufficientCapacity` error rather than an unbacked `Proven` tag
  (M-I2/VR-5). `required_dim` reproduces the four M-001 probe settings (1141/1843/2164/2764).
  **Acceptance — ≥10⁴-trial empirical validation (SC-2):** over 10,000 independent trials at
  `dim ≥ required_dim(3, 1e-2)`, the measured nearest-neighbour retrieval-failure rate stays `≤ δ`.
- **VSA submodule — `VsaModel` trait + MAP-I** (`mycelium-vsa`, **M-130**, Phase 1; RFC-0003 §3–§4;
  ADR-008; T2.6): a composition-style `VsaModel` trait (`bind`/`unbind` + self-inverse flag,
  `bundle`, `permute`/`unpermute`, `similarity`, and the honest per-op intrinsic guarantee) and its
  first model **MAP-I** — `bind`/`unbind` are self-inverse and **`Exact`** (elementwise product),
  `permute` is **`Exact`** (cyclic shift), `bundle` is elementwise superposition. Value-level
  adapters for the Exact ops carry honest `Derived` provenance. **Dependency-gated** (ADR-008): the
  crate depends on `mycelium-core` but the kernel does not depend on it — VSA values stay
  type-checkable in the kernel without pulling in this algebra (KC-3). Tests: bind/unbind round-trip
  exactly, permute is invertible/cyclic, a bundle is far more similar to its members than to a
  stranger, dim-mismatch/empty-bundle are explicit errors. The `bundle` **`Proven`** capacity bound
  (M-I2: a *value*-level Proven bound needs a checked basis) is deferred to **M-131** — not stamped
  here (VR-5).
- **Binary↔ternary certified swap** (`mycelium-cert` + `mycelium-core::binary`, **M-120**, Phase 1;
  RFC-0002 §3/§4): `enc`/`dec` per `docs/spec/swaps/binary-ternary.md` over a legal `(n, m)` pair,
  emitting a `SwapCertificate::Bijective` (`LosslessWithinRange`) that references the once-per-pair
  round-trip lemma (`lemma_ref`) bound by concrete `params`. `enc` is total on `B_n`; `dec` is the
  **partial** inverse — a value outside the binary range is an explicit `SwapError::OutOfRange`
  (P4), an illegal pair is a **type error** (`IllegalPair`, RFC-0002 §5), never a `Declared` gamble.
  Within range the result is `Exact`/`bound = None` (P3, M-I1) and records `policy_used` + `Derived`
  provenance. A `BinaryTernarySwapEngine` plugs the swap into the M-110 interpreter. **Acceptance —
  `dec(enc x) = Some x` exhaustively over all 256 bytes** (8↔6, SC-1); serializer output pinned to a
  committed `swap-certificate` example validated against the schema in CI (SC-3). Adds a
  two's-complement codec `mycelium-core::binary` (exhaustively round-trip-tested).
- **Binary↔ternary round-trip proof** (`proofs/binary-ternary-roundtrip/`, **M-121**, Phase 1;
  VR-1/SC-1): the SMT-LIB2 injectivity obligation for the 8↔6 pair — **discharged by Z3 4.16.0
  (`unsat`)**: no two distinct 6-trit vectors collide ⟹ the value map is a bijection onto
  `[−364, 364] ⊇ B_8` ⟹ `dec(enc b) = b` (P1/P2). Wired into `scripts/checks/proofs.sh`
  (skip-graceful without z3); the lemma identity matches `mycelium_cert::roundtrip_lemma_ref()`. P3/P4
  are additionally decided by the M-120 exhaustive Rust corpus. (The fixed `8↔6` instance; a
  width-generic proof is future work — each legal pair gets its own discharged lemma.)
- **Balanced-ternary arithmetic** (`mycelium-core::ternary` + `mycelium-interp`, **M-111**, Phase 1;
  FR-M2): the single home for the balanced-ternary integer codec (`int ↔ trits`, MSB-first, the
  §3.1 digit-extraction algorithm) and fixed-width digit-wise arithmetic — `neg` (digit-wise sign
  flip = value negation), ripple-carry `add`/`sub`, and shifted-add `mul`. Out-of-range results are
  an explicit `None`/`EvalError::Overflow`, **never** a silent wrap (SC-3). The interpreter gains
  `trit.neg/add/sub/mul` primitives over it. **Acceptance — property-tested vs an `i64` oracle by
  exhaustion** over all operand pairs at widths `m ≤ 4` (and the codec round-trip/neg at `m ≤ 5`):
  in range the digit-wise result equals the encoded integer result, out of range it overflows.
  Grounded in `docs/spec/swaps/binary-ternary.md` §1/§3.1; reused by the M-120 swap.
- **Reference interpreter** (`mycelium-interp`, **M-110**, Phase 1): the trusted, executable
  **small-step operational semantics** for the Core IR, closing SPEC §10.3 (RFC-0004 §2; ADR-009;
  NFR-7). Call-by-value substitution over closed `Node`s with the rules E-Let-Bind/Step,
  E-Op-Arg/Apply, E-Swap-Arg/Apply (documented in the crate). An extensible **primitive registry**
  (`PrimRegistry`) ships the exact elementwise built-ins (`core.id`, `bit.not/and/or/xor`,
  `trit.neg`); a **`SwapEngine`** hook ships the trivial same-`Repr` `IdentitySwapEngine`. Results
  thread metadata honestly — guarantee by `meet` (RFC-0001 §4.7), provenance `Derived{op, inputs}`
  over content hashes (§4.6), `policy_used` on swaps. **Never silent**: free variables, unknown/
  ill-typed prims, unsupported cross-paradigm swaps, approximate-input composition (no bound kernel
  yet — ADR-010/E2-4), and fuel exhaustion are all explicit `EvalError`s. 20-case golden corpus.
  Adds `mycelium_core::operation_hash` (provenance op identity for prims). Scope boundary:
  balanced-ternary arithmetic + oracle property tests are **M-111**; the certified binary↔ternary
  swap + proof are **M-120/M-121**.
- **Guarantee `meet`-composition** (`mycelium-core::guarantee`, **M-102**, Phase 1):
  `GuaranteeStrength::meet` (the weakest-wins greatest-lower-bound) plus `propagate`/`meet_all` for
  the RFC-0001 §4.7 rule `guarantee(result) = meet(inputs…, g_f)`, and `TOP`/`ALL` constants. The
  meet-semilattice laws — commutativity, associativity, idempotence, identity `Exact`, `Declared`
  absorbing — are verified by **exhaustion** over all 4×4(×4) tuples (complete for the finite
  lattice, not sampled). Honesty can only degrade, never spuriously upgrade (VR-3/VR-5).
- **Content-addressing** (`mycelium-core::content`, **M-103**, Phase 1): `Node::content_hash` /
  `Value::content_hash` — a BLAKE3 hash over an injective, domain-separated, length-prefixed
  encoding of the *identity-bearing* content: the α-normalized structure (bound vars as de Bruijn
  indices, binder names dropped), types-with-`Repr`, constant literals, operator names, and swap
  target+policy. Dynamic `Meta` (provenance, bounds, sparsity, `policy_used`) is excluded. Adds a
  separable `hash ↔ name` table (`Names`) for names-as-metadata, `ScalarKind::tag`, and
  `ContentHash::from_parts`/`algo`/`digest`. Acceptance met: identical defs collide; trivial (α)
  renames don't change identity; a paradigm/precision/literal/operator change does (RFC-0001 §4.6;
  ADR-003).
- **Core IR (de)serialization** (`mycelium-core`, **M-104**, Phase 1): `serde`
  `Serialize`/`Deserialize` for `Value`/`Meta`/`Repr`/`Bound`/`Provenance`/… emitting *exactly* the
  ratified JSON data contracts (`kind`/`class`/`layout` tags; `VSA`/`BF16`/`TL1`/`TL2` renames;
  `payload` as `{bits|trits|scalars|hypervector}` with MSB-first bit/trit strings; `bound` modelled
  by presence; flat `kind`+`basis` `Bound`). `Deserialize` routes `Value`/`Meta` through their
  checked constructors, so M-I1…M-I4 and payload↔repr mismatches are rejected on the wire — never
  silently accepted. Faithful round-trip (`deserialize(serialize(v)) == v` incl. `Meta`) is tested
  over a corpus spanning all four paradigms × every guarantee/bound/basis/layout; serializer output
  is pinned to three new committed `value` examples (ternary/dense/vsa) that `scripts/checks/schema.sh`
  validates against `value.schema.json` in CI (RFC-0001 §4.8).
- **Core IR data structures** (`mycelium-core`, **M-101**, Phase 1): Rust types mirroring the
  ratified schemas — `Repr`/`ScalarKind`/`SparsityClass`, the `GuaranteeStrength` lattice,
  `Bound`/`BoundBasis`/`BoundKind`/`NormKind` (ADR-011: `basis` universal), `Meta` (with
  `Provenance`, `SparsityObs`, `PhysicalLayout`/`PackScheme`), `Value`/`Payload`, `ContentHash`,
  and the `Node` grammar (closes the core of SPEC §10.2; RFC-0001 §4.5). The honesty invariants
  **M-I1…M-I4** and payload↔repr/repr well-formedness are enforced **by construction**
  (`Meta::new`, `Value::new` → `WfError`). 17 unit tests; `fmt`/`clippy -D warnings`/`test` green on
  MSRV 1.92.
- **Minimal surface-syntax fragment** (`experiments/surface-fragment/`, **M-020**): a throwaway,
  experiment-only concrete syntax (EBNF + desugaring to the Core IR nodes + 3 reference programs:
  swap round-trip, VSA `bundle`, and a no-implicit-conversion type-error) to feed the KC-2
  experiment. **Not** a committed surface — gated on KC-2 (hence under `experiments/`, not
  `docs/spec/`). Linked from `SPECIFICATION.md` §10.1.
- **Binary↔ternary encoding spec** (`docs/spec/swaps/binary-ternary.md`, **M-012**): precise
  `enc`/`dec` for the canonical `8↔6` width — balanced-ternary digit semantics, the legality
  condition `B_n ⊆ T_m`, `LosslessWithinRange` with an `Option`-typed (never-silent) inverse, the
  four M-121 correctness obligations, and a worked round-trip + out-of-range example (RFC-0002
  §4/§5; T2.1). Linked from `SPECIFICATION.md` §6/§10.4.
- **Python tooling skeleton** (`experiments/`, **M-092**): a UV-managed project targeting
  **Python 3.13** (ADR-007) with a `dev` group (pytest, pytest-cov, ruff, black), a trivial
  importable module + passing smoke test, and a committed `uv.lock`. `scripts/checks/test.sh` runs
  it via `uv run --frozen pytest` under the pinned interpreter, so it joins the `just check`/CI
  suite (skip-graceful when uv is absent).
- **Rust workspace skeleton** (**M-091**): a 6-crate Cargo workspace (`mycelium-core`,
  `mycelium-interp`, `mycelium-vsa`, `mycelium-mlir` stub, `mycelium-cert` stub, `xtask`) with
  **MSRV pinned to 1.92** via `rust-toolchain.toml` + `rust-version` (ADR-007), workspace lints
  (`unsafe_code = forbid`, clippy warn), and a smoke test per crate. `cargo fmt --check`,
  `clippy -D warnings`, and `cargo test` are all green on 1.92. Adds `scripts/checks/test.sh` +
  `just test`, wired into the `just check`/CI suite (skip-graceful when a toolchain is absent), so
  test parity now holds local↔CI. Fixes a malformed `Cargo.lock` line in `.gitignore`.
- **M-001 probe scaffold** (`proofs/lh-bundle/`): the Liquid-Haskell MAP-I `bundle`
  capacity-refinement module + cabal project + writeup, encoding the axiomatized-theorem +
  checked-instantiation strategy with ≥3 concrete `(d,m,δ)` settings (RFC-0003 §5; T0.2). **Not yet
  discharged** — no GHC/LH/Z3 in this environment — so KC-1 stays `passed (literature)`; the
  derivation table is the independently-checkable artifact. Establishes `proofs/<name>/` as the
  home for machine-checkable proofs (resolves OQ-2).
- **`SPECIFICATION.md` skeleton** (`docs/spec/SPECIFICATION.md`, **M-011**): the consolidation index
  over the corpus — §1–§9 reconciled to RFC-0001 (r2)/RFC-0002…0005/ADR-010/011/DN-01 and pointed at
  the ratified `docs/spec/schemas/` contracts; §10 enumerates the open build items, each linked to a
  live issue (no floating TODOs). Status `consolidating-draft → ratified-skeleton`.
- **ADR-011 — `BoundBasis` is a property of every `Bound`** (`docs/adr/ADR-011-...md`, Accepted):
  formally supersedes the implicit RFC-0001 r1 §4.3 decision that scoped `basis` to `CapacityBound`
  only, so every approximate value (ε, δ, crosstalk, capacity) honestly records how its bound was
  obtained (VR-5, G5). Resolves OQ-3.
- **Core data-contract schemas** (`docs/spec/schemas/`, **M-010**): the 10 ratified JSON Schemas
  (draft 2020-12) — `repr`, `value`, `meta`, `guarantee`, `bound`, `provenance`,
  `physical-layout`, `swap-certificate`, `policy`, `reconstruction-manifest` — each a faithful
  projection of its source RFC/ADR section, plus ≥1 valid and ≥1 invalid example per schema (the
  invalids exercise the honesty-load-bearing invariants M-I1/M-I4). `just schema` validates the
  set in CI. The OQ-3/OQ-4/OQ-5 clarifications surfaced here are now resolved (see below /
  `docs/spec/schemas/README.md`).
- **Phase-0 working plan** (`docs/planning/phase-0.md`): the first issue-coupled expansion of
  Foundation §6, mapping the nine Phase-0 tasks (M-001/002/010/011/012/020/090/091/092) to their
  GitHub issues, the critical path, honest KC-1/KC-2 gate status, the proposed canonical
  data-contract schema set, and the author-then-ratify reframing for M-010/M-011 (the
  `docs/spec/` artifacts they ratify do not exist yet).
- Initial **design baseline**: project charter (`docs/Mycelium_Project_Foundation.md`, r3),
  document index (`docs/Doc-Index.md`), five RFCs (RFC-0001…0005, all Accepted),
  ADR-010 (Accepted), design note DN-01 (Resolved), and two research records
  (`research/01`, `research/02`).
- Repository scaffolding: `README.md`, `LICENSE` (MIT), `CONTRIBUTING.md`,
  `.gitignore` (Rust + Python), and index/process READMEs for `docs/adr/` and `docs/rfcs/`.
- **GitHub PM bootstrap** (`tools/github/`): `issues.yaml` / `labels.json` / `milestones.json`,
  the `mcp-bootstrap.md` runner + `gh-bootstrap-local.sh`, the `project-v2-spec.md` board spec,
  and the `idmap.tsv` task→issue map.
- **Agent tooling**: `CLAUDE.md` and `.claude/skills/` (`pr-review`, `security-review`,
  `dev-workflow`, `docs-review`, `changelog`) operationalizing the `CONTRIBUTING.md` house rules.
- **Local check tooling** with local↔CI parity: `justfile` + `scripts/checks/*` (markdownlint,
  offline link/cross-reference, json-schema, codespell, shellcheck, secret scan, fmt/lint),
  `.pre-commit-config.yaml`, and a manual-dispatch **advisory** GitHub Actions workflow.

### Changed
- **Proofs wired into the check suite** (`scripts/checks/proofs.sh` + `just proofs`): runs the
  LiquidHaskell `bundle` probe (`LC_ALL=C.UTF-8 cabal build`, a green build ⟺ LH `SAFE`),
  skip-graceful when GHC/cabal/z3 are absent. Added to `just check`/`just ci`; the manual-dispatch CI
  workflow now sets up GHC 9.8.2 + cabal + z3 (with a cabal/dist-newstyle cache) so the proof
  verifies on a manual run. (Whole suite remains `workflow_dispatch`-only.)
- **KC-1 confirmed (build)** (**M-001**): the Liquid-Haskell MAP-I `bundle` capacity refinement
  (`proofs/lh-bundle/`) type-checks **`SAFE` (16 constraints)** and Z3 discharged all four `(d,m,δ)`
  instantiations (GHC 9.8.2 · LiquidHaskell 0.9.8.2 · Z3 4.8.12), ratifying the axiomatized-theorem +
  checked-instantiation strategy (RFC-0003 §5; ADR-010). KC-1 moves `passed (literature) → confirmed
  (build)` in the Foundation §2.4 and Doc-Index §3/§4. (The Clarkson/Thomas theorem remains cited,
  not re-proven — by design.) Haskell build output (`dist-newstyle/`, `.liquid/`) gitignored;
  codespell skips them.
- **Docs/parity CI hardened** (`.github/workflows/checks.yml`, **M-090**): the manual-dispatch
  advisory workflow now sets up **uv** (so the `experiments/` Python 3.13 tests actually run) and
  **Rust** (pinned via `rust-toolchain.toml`, so fmt/clippy/test run), and adds an advisory
  **Codecov** upload of the experiments coverage. Markdown-lint + offline link-check + schema
  validation already covered `docs/**` and the schemas via `just ci`; the PR template was already
  wired. Posture unchanged: `workflow_dispatch` only, non-blocking (no auto-triggers — CLAUDE.md).
- **RFC-0001 → r2** (status stays Accepted): §4.3 `Bound` grammar revised per **ADR-011** —
  `BoundBasis` factored out to a required companion of *every* `Bound` (was: `CapacityBound` only),
  and `NormKind` enumerated `L1|L2|Linf|Rel` as an extensible registry (resolves OQ-4). The r1 §4.3
  grammar is formally superseded; indexes (`Doc-Index.md`, `docs/rfcs/README.md`,
  `docs/adr/README.md`) and the `bound` schema updated to match.

### Changed (baseline-review consistency pass)
- ADR-001 promoted to firmly **Accepted**; the "no statistical approximation vs
  fully-disclosed approximation" definitional question recorded as **settled**
  (fully-disclosed), consistent with the KC-1 pass and the guarantee lattice.
- Foundation §5.2 core-model sketch marked **superseded by RFC-0001** (packing is
  now schedule-staged, not in the type; guarantee lattice is the four-point form).
- Foundation §5.6 updated: **MLIR→LLVM** recorded as the committed AOT path
  (ADR-007 / RFC-0004), not a candidate.
- Foundation §6 Phase 0 annotated with post-research status (largely complete;
  remaining: the Liquid-Haskell `bundle` probe and the KC-2 LLM-leverage experiment).
- `README.md` decisions table: fixed a placeholder reference for the
  "no implicit conversion" rule (grounded in RFC-0001 §3.3 / FR-M3).
- `docs/Doc-Index.md`: the two research rows now point to the in-repo records.

### Fixed
- Markdown hygiene surfaced by the new check tooling: normalized emphasis to the corpus
  asterisk style and added a missing trailing newline (`README.md`,
  `research/01-prior-art-survey-RECORD.md`, `docs/notes/DN-01-Packing-Placement-Tradeoffs.md`).
- Copilot PR-review findings (PR #1, #42) addressed: corrected the binary↔ternary swap's partial
  right-inverse in RFC-0002 §4 (`dec y = Some x ⟹ enc x = y`; the prior `enc y = …` was a type
  error since `enc : Bin_n → Tern_m`); resolved a P0.3 status contradiction in the Foundation
  Meta section (P0.3 is already resolved per §6); corrected stale references in `tools/github/`
  (`gh-bootstrap.sh`, `docs/planning/*`, `project-v2-spec.md`); `gh-bootstrap-local.sh` now
  honors each milestone's `state` instead of hardcoding `open`.
- Tooling self-lint: `scripts/*` made shellcheck/ruff/markdownlint-clean (cd-failure guards,
  if/then/else over `A && B || C`, split imports, fenced-block spacing).

### Security
- `.gitleaks.toml`: removed an allowlist **regex** (`AKIA[0-9A-Z]{16}`) that exempted the AWS
  access-key-ID *pattern* from scanning — it would have suppressed detection of a real leaked
  key. The path allowlist is retained; pattern-level allowlisting is documented as forbidden.

### Open
- One confirming build: the Liquid-Haskell `bundle` capacity-refinement probe (RFC-0003 §5).
- One existential question: **KC-2 / LLM leverage** (the E4 experiment) — not yet settled.
- Decomposed task/issue set and phase planning documents — *forthcoming* (`docs/planning/`).
