# Changelog

All notable changes to this project are recorded here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Dates are ISO-8601.

This project is in **design + Rust-first implementation**; entries cover both the documentation
corpus and the landing kernel/stdlib code. Semantic versioning will begin when the kernel stabilizes.

> Older entries are archived on periodic sweeps; this file holds `[Unreleased]` + a rolling recent
> window. Full history: `docs/archive/changelog/`.

## [Unreleased]

### DN-87 — the transparent memory substrate & agent knowledge API captured; E39-1 + M-1015…M-1019 minted (2026-07-07)

Captures (Proposed) the maintainer's vision near-verbatim: the corpus (methodologies, decisions,
intents, language docs, tracker state) converted into a **generated, transparent,
provenance-carrying encoding** that *supplements* the human-friendly format — search/access
**improved upon RAG**, with that claim explicitly `Empirical`-gated behind M-1018's graded eval
harness — exposed as a **secure, platform-agnostic API** (MCP + HTTP, token-scoped) plus an
optimized skill set usable by Claude Code, Grok, and any other platform. Hybrid substrate:
the deterministic structured-index pattern generalized corpus-wide (Layer 1, the floor) + a **VSA
semantic layer** on `mycelium-vsa` with `EXPLAIN`-able resonator retrieval (Layer 2, the
improved-on-RAG bet). v0 = Rust core + Python ingestion; the **Mycelium-lang package** is a
phase-gated dogfood milestone (M-1019 ⟂ M-993). Mandatory provenance — an uncited answer is a
refusal (G2); deterministic, drift-gated regeneration; MIT-only by house rule, with the
maintainer's contribute-to-society intent recorded. Epic **E39-1** + **M-1015…M-1019** minted
(E35–E38 left to DN-86's proposal); kickoff **`mem`** stowed — **the build wave (the fractal-swarm
/ concurrent-PR shape) fires when the maintainer names the project**. Four design forks
orchestrator-resolved (`Declared`, maintainer-overridable) after the interactive confirm failed —
the session's standing pattern.

### DN-85 — the multi-language transpilation program + single-language full-stack goal (2026-07-06)

Records (Proposed, maintainer direction) the generalization of DN-34's Rust-only transpiler into a
**multi-source-language** program whose flagship goal is a **single-language Mycelium full stack** —
collapse a polyglot ecosystem's application, native extensions, and compute kernels into one
language, toolchain, and guarantee model. Sequencing: Rust (in flight, trx2) then **Python-first
(pure Python, gated on sound type inference)** then C/C++/Fortran/Cython/CUDA as demand arises.
Interim strategy: transpile the coverable layer and **FFI-bind** the native backend (e.g.
PyTorch/TensorFlow C++/CUDA) until its transpiler lands. **Open-source constraint** with the honest
provenance ladder — transpile vs. binding vs. reverse-engineered Mycelium-native reimplementation; a
bound or reverse-engineered artifact is **never** tagged a faithful port (G2/VR-5).
`Declared`/aspirational; not `1.0.0`-gating (ADR-036). DN-34 gains a forward-pointer; Doc-Index
registers the note. Its **architecture companion is DN-86** (front-end abstraction + method).

### DN-86 — multi-language transpiler front-ends architecture (2026-07-06)

New design note (Draft), the **architecture companion to DN-85** (which holds the strategy/vision) —
authored concurrently and reconciled to a distinct number. The front-end-abstraction shape for
extending the transpiler to ingest **Python** (then TypeScript, Java) alongside Rust. Grounds the
refactor in the current `mycelium-transpile` split: the `vet.rs` myc-check loop and `.myc` emitter
are **source-agnostic**, and `gap.rs`'s *structure* is reused wholesale though its taxonomy carries
Rust-source-shaped categories (`Trait`/`Impl`/`MacroDef`/`DeriveAttr`/…) a multi-language front-end
must **generalize** (the honest correction to a flat "backend already language-neutral" claim); only
`transpile.rs`/`emit.rs`/`map.rs` are `syn`-coupled. Transfers the trx2 M-1006 ladder as the method.
Per house rule #4 states the boundaries: Python dynamic typing (§4.1); the **C/CUDA library-core
boundary** (§4.2 — numpy/scipy/pytorch cores are compiled C/C++/CUDA, *not* Python source, so
transpilation yields the Python layer only and the compute is a separate Mycelium-native/FFI track);
follow-on work bounded by Mycelium surface coverage (§4.3). Records the self-hosted-`.myc` toolchain
switch (§5) as a `boot10`/DN-26 dependency. Ids proposed (E35–E38), not minted; decides nothing
normatively.

### M-1006 phase-1 — transpiler hardening against the DN-34 §8.9 gap worklist (2026-07-06)

First phase of the M-1006 whole-corpus rip-through ladder (kickoff `trx2` E-B, epic E33-1), run as
two disjoint-file leaves over the same 17 wave-1 targets and octopus-merged. Three grammar-grounded
transpiler fixes, each never-silent (`crates/mycelium-transpile`): concrete generic type-applications
now map to `type_args` (`Head<A,…>` → `Head[A,…]`, recursive, never-partial — a whole gap sub-class
closed); string/float/array expression literal arms (`StrLit`/`FloatLit`/`ListLit` — non-finite
floats and un-escapable control chars refuse rather than emit garbage); and sharpened `MultiStmtBody`
diagnostics. Measured with the real `myc check` oracle (`Empirical`): union `expressible_fraction`
6.06% → 6.19% (`std-io::read_all` unblocked via a nested `Result[Vec[Binary{8}], IoError]`),
`checked_fraction` flat at 3.69%, `GenericBound` gaps 59 → 46. DN-34 §8.10 records the before/after
plus the M-1006-DoD residual enumeration: the dominant remaining classes (type-coverage scalars,
named-field structs/variants, imports, bounded generics, Rust built-in derives) are language-surface
design (E18-1), not transpiler defects — the current-corpus transpiler-fixable surface is
near-exhausted (stopping point recorded, G2). Emission `Declared`; drafts stay in `gen/myc-drafts/`,
never imported by `lib/`. `docs/notes/DN-34` §8.10; `gen/myc-drafts/` regenerated.

### Transpiler usage operationalized as skills — `/transpile-vet` + `/myc-drafts` (2026-07-06)

The trx2 wave-1 process is captured as parameterized skills (the `/wave`/`/pr-land` precedent) so
future transpiler usage is lightweight — no re-orchestration: **`/transpile-vet`** (run the
M-1000/M-1001 loop; read `checked_fraction` vs `expressible_fraction` without conflating them; the
binding honesty rules and DN-34 append-only recording discipline; wave-1 calibration numbers) and
**`/myc-drafts`** (deterministic corpus regeneration; manifest-first triage before porting; the
`lib/` graduation checklist with its differential witness; the 5-step M-1006 ladder-phase recipe
with the rwr Phase-II reconciliation guard). Registered in CLAUDE.md §Skills + the agent context;
M-1006's body cites them as its per-phase recipe.

### M-1002/M-1003 — gen/myc-drafts/: the vetted draft corpus over the full boot10 port surface (2026-07-06)

E33-1 wave-1 rip-through: `gen/myc-drafts/` staging tree (README honesty contract — everything
`Declared`, never imported by `lib/`, never dogfood-gated; drafts graduate only via hand-vetted
M-993 work), a shellcheck-clean `regenerate.sh` driver + `manifest_gen.py` aggregator (`just
myc-drafts-regen`), and the run itself over all 17 port-surface targets (5 semcore files + 12
unported stdlib crates): **union checked_fraction 3.7%** (759 non-test items / 46 emitted / 28
check-clean), 51/56 emitted files myc-check-clean, zero hard transpile failures, zero silent
holes (G2). Confirms M-991's NO-GO-as-bulk / GO-as-profiling verdict at full-surface scale;
eval 2.4% + std-time 8.1% independently reproduce E-A's §8.8 samples (cross-validation).
Determinism verified byte-identical (manifest + full-tree sha256 across independent runs).
DN-34 §8.9 appended: per-target table + the ranked 812-gap residual worklist (Other/type-coverage
322, Impl 119, Import 117, Struct 80, GenericBound 59) — the M-1006 ladder's phase-1 input.
Kickoff `trx2` E-B (epic E33-1; wave 1 of the maintainer's two-stage breadth plan).

### M-1000/M-1001 — the transpile → myc-check vet loop + top gap-class closure; M-991 assessed (2026-07-06)

The transpiler (M-873 PoC) now vets its own output against the real toolchain: `--vet` runs
`myc check` per emitted file and reports **`checked_fraction`** (file-gated, honestly-conservative:
a failing file credits 0) alongside the old `expressible_fraction` — exposing that the prior
"coverage" numbers over-counted emissions that poison the checker (all targets started at
**0% checked**). M-1001 closed the two universal check-poisons flag-don't-guess (unresolved
`use` → `Category::Import` gaps; Mycelium reserved-word collisions → `Category::ReservedWord`
gaps, drift-guarded against the l1 lexer table), lifting eval.rs to 2.4% and std-time to 8.1%
checked. **M-991 verdict (DN-34 §8.7–§8.8, append-only): NO-GO as an automated bulk transpiler
for the semcore port (the residue is language-surface design work, not boilerplate), GO as a
never-silent gap-profiling instrument** — the vet loop turns the port into a ranked, checked
worklist. Advisory `just transpile-vet` wired (on-demand, not a gate). Emission stays `Declared`;
vet verdicts are `Empirical`. Kickoff `trx2` E-A (epic E32-1).

### M-1004/M-1005 — docs/lib-index/: the api-index analogue for the self-hosted `.myc` tree (2026-07-06)

Added `crates/mycelium-doc/src/lib_index.rs` (`myc-doc lib-index`) and the committed
`docs/lib-index/{INDEX.md,index.json}`: 3313 items (26 nodules — 17 `std`, 9 `compiler`; 373
types, 1201 constructors, 1713 fns; 0 flagged) extracted from every `lib/std/` + `lib/compiler/`
`.myc` file, grouped by phylum/nodule, `Empirical/Declared` heuristic (source is ground truth).
Reuses `apiref.rs`'s nodule/fn extraction rather than a parallel heuristic (DRY); building it
surfaced and fixed four pre-existing bugs shared with the corpus doc-IR (`=>` return-arrow
truncation, a stray trailing `;` on every nodule name, a section-divider comment misattributed as
an item's doc summary, and multi-line-signature truncation). Drift-gated
(`scripts/checks/lib-index.sh` via `just lib-index`, wired into `just check`; regenerate via
`just lib-index-gen`), proven by a deliberate-drift test (corrupted a committed field → gate
failed with `diff -r` + exit 2 → reverted → gate passed). Determinism verified byte-identical
across independent runs. Kickoff `trx2` E-C (epic E34-1), launched same day with epics
E32-1/E33-1 (transpiler vet loop + mass `.myc` drafts, in flight) and the M-1006 phased
rip-through ladder (maintainer-decided breadth amendment).

Closes the maintainer-flagged **performance inversion**: post-M-995/996, the AOT env-machine was
still ~4.5× *slower* than the L1 interpreter same-profile (release, apples-to-apples — the honest
baseline the old cross-profile numbers had understated). Profiling (callgrind; ~55–60% of
instructions in malloc/free/memcpy) showed the planned env fix alone wasn't dominant, so the fix
landed as **four measured steps** in `aot.rs`:

1. **Env representation** — mutable top segment + `Rc`-frozen parent frames (O(1)-amortized capture
   at closure creation; innermost-wins chain lookup; iterative `EnvFrame::drop`). Alone: 0.22×→0.27×.
2. **Prepared code mirror** — the lowered ANF mirrored **once** into `Rc`-shared blocks; the machine
   had been deep-cloning `Lam`/`Fix` bodies and match-arm subtrees *per execution*. →0.72×.
3. **Interned atoms** — `Rc<Atom>` keys prepared once; re-binding is a refcount bump. →parity.
4. **`AotVal::Repr(Rc<Value>)`** — variable references are refcount bumps; the value enum shrinks to
   pointer size (the old "intentionally inlined" trade-off superseded-by-measurement, noted at the
   type). →**1.5–1.7× ahead** on snoc (n=100/200/400), 1.2–1.4× on a 50k tail loop.

Both machines fit clean ~n² (M-995 fixed the curve; M-999 removed a ~7× constant). The **ordering
witness** is committed (`tests/aot_vs_interp_bench.rs`, `#[ignore]`d comparative benchmark — rerun
with `--release --ignored --nocapture`; single-trial `Empirical`, ~2–5% jitter). **Zero expectation
edits**: `mycelium-mlir` 382/0 (439/0 with `mlir-dialect`), `mycelium-l1` 991/0, no new `unsafe`.
**Review correction (PR #1194 HIGH, owned):** the bench's first placement (in `mycelium-mlir`, via a
new `l1` dev-dep) closed a **real** `{l1, mlir}` dev-dependency cycle that `cargo xtask deps`
rejects (DN-68 — the initial "deps-acyclic green" claim was a faulty verification, a shell pipe-rc
bug, not a gate bug); fixed by **moving the bench to `crates/mycelium-l1/tests/`** (the pre-existing
dev-edge direction) and removing the reverse edge — re-verified `xtask deps` exit 0, no violations.
**The honest ladder, recorded:** ~1.5× is the realistic band for a trampolined ANF-machine;
"far faster" belongs to the direct-LLVM native path, whose v0 coverage is already wide (RFC-0029:
data, native swap, widened closures, `Fix` loop rewrite, Dense/VSA) — growing that coverage is the
big lever. FLAG: `llvm.rs`'s stale header (says closures/recursion "deferred", contradicted by
M-850/M-851 in the same file) → docs sweep.

### M-996 — AOT env-machine TCO: tail frames elided, observably (maintainer-authorized) (2026-07-06)

Completes the cross-machine convergence of the M-994 arc: the AOT env-machine
(`aot.rs::run_core`) now elides tail frames, closing the §5.1 family-parity gap fix (a) opened
(the same program at the same budget succeeded interpreted but refused `DepthLimit` on the AOT
env-machine — the full-calculus AOT leg that Stage-6's three-way and the M-993 "(c) fallback" run on).

- **Machine-appropriate shape (not a copy of the interpreter's):** in the ANF env-machine,
  tail-transparency is an *intrinsic O(1) property of the continuation* — a `Resume(Cont)` whose
  block is complete and whose `result()` is exactly the bound name is a pure passthrough (settle
  binds, then passes the same value up unconditionally). So the "peek" is that test at push time and
  the "commit" is **not pushing** the frame (eagerly dropping the caller's saved env — the drain
  analog). No transparent frame ever enters the stack. `ApplyThen` (the `Fix` unfold) has real
  post-work and is never elided. No Substrate-like affine values exist in the AOT fragment (stated,
  not cargo-culted). Depth accounting per §4.0: elided calls never take a depth guard (a tail call
  *at* the ceiling succeeds; a guard-leak pin proves net-zero).
- **Observable, per the no-black-boxes rule:** a `TcoTrace { total_elided }` counter threaded through
  the machine (the interpreter's `TcoTrace` analog), asserted in the deep-loop test
  (`count(10_000)` @ depth 64 → `Ok(0)`, ≥10,000 elisions). A **user-facing** EXPLAIN surface for
  AOT traces does not exist yet — minted as **M-998**, not silently skipped.
- **Behavior shifts — exactly the two authorized (maintainer, 2026-07-06):** deep-tail
  `DepthLimit → Ok(value)`, divergent-tail `DepthLimit → FuelExhausted` (convergence with the
  interpreter's long-standing behavior; the graceful-ceiling property stays pinned via the **non-tail**
  witness, which doubles as the no-over-elision guard). Everything terminating is byte-identical:
  267 `mycelium-mlir` lib tests + all differentials green; reverse-dependents untouched and green
  (`mycelium-l1` 991/0 incl. the canonical `DepthLimit{4096}` non-tail pin; `std-conformance` 293/0).
  **Parity witness:** `countdown(10_000)` now agrees L0-interp ≡ AOT env-machine (same value +
  guarantee) — with the L1 pin of the same shape, all three machines agree. An explicit combined
  L1↔AOT deep-tail case in `depth_metric_parity.rs` is a flagged follow-on (M-996 note).
- **Corollary, recorded not hidden:** with a *declared* `alloc` effect budget, elided tail frames
  charge no alloc bytes — the §4.0 principle (no frame ⇒ no control-stack memory) applied to the
  alloc sibling; a deep tail loop that would have overrun a declared ledger via its `Resume` frames
  may now complete (the `Fix` `ApplyThen` frames still charge; the existing alloc-overrun pin passes
  unmodified).
- Measured (debug, `Empirical`): `count(500)` @ depth 1000 `DepthLimit{1000}` → `Ok(0)`; ~36% less
  frame churn on a 30k-iteration loop (1.13s → 0.72s).

### M-995 — AOT env-machine value structural-sharing (the M-987 perf win on the AOT path) (2026-07-06)

Carries the M-994 (b) win to the **AOT** (`mycelium-mlir`), since it enhances runtime performance
drastically. The AOT env-machine (`aot.rs::run_core`) had the *same* per-reference O(nodes) deep-copy
(`AotVal::Core` clone on every `lookup` + `Match`-arm bind) — measured **~n³ (fitted 2.98)**.

- **Not a literal port:** the interpreter's `Arc`-on-`L1Value::Data.fields` can't apply — the AOT's
  fields live in the **frozen** `mycelium_core::Datum` (DN-56). The freeze-respecting fix is an
  **AOT-local `AotVal::Data(Rc<AotDatum>)`** cons cell with `Rc`-shared sub-trees, so a
  reference/env-clone **and** a destructure field-bind are O(1); the frozen `Datum` is untouched, a
  `CoreValue` is materialised only at `to_core` (iterative), and `AotDatum::Drop` is iterative
  (deep-spine SIGABRT-safe).
- **Measured (release, `Empirical`):** exponent 2.98 → ~2.3–2.5, **13×/21×/35×** at n=100/200/400
  (38.6s → 1.11s at n=400). Honest caveat: a *less-clean* win than the interpreter's clean n³→n²
  (14–64×) — the residual is the env-machine's HashMap-environment cloning per match/app (a future
  env-rep change could recover a cleaner n²). Still a large, genuine win.
- **Behavior-preserving:** the full `-p mycelium-mlir` suite is green (263 lib + integration incl. the
  three-way `mlir-dialect` differentials), results **byte-identical** (`ObservationalEquiv` +
  M-210 `Validated{Exact}`); the `aot_frame_size` pin holds; **zero new `unsafe`**. `mycelium-mlir` is
  **not** in the DN-56 freeze scope (the AOT is the RFC-0041 perf path; the frozen `mycelium_core` type
  is untouched) — lands via the §6 behavior-preserving channel + normal review.
- **The (a) TCO analog is NOT here (decision-gated — M-996):** the AOT env-machine has *no* TCO, but
  adding it is a behavior-changing new feature (a divergent tail loop moves `DepthLimit` →
  `FuelExhausted`, breaking a pinned graceful-error test) — a maintainer decision, not a
  behavior-preserving landing. The native LLVM path already has real O(1) TCO for the canonical
  tail-`Fix` shape.

### M-994 fix (b) — O(1) `Data` clone via `Arc` structural sharing (M-987 ~n³→~n²; M-994 resolved) (2026-07-06)

The *cost* half of M-994, completing the decision. The confirmed root of the ~n³ L1-eval cost was
that `eval_path` deep-copies an O(nodes) value on **every** variable reference (`L1Value::clone` for
`Data` rebuilt the whole spine). Since `Data` is immutable + acyclic by construction, wrapping its
`fields` in `Arc<Vec<L1Value>>` makes a clone a refcount bump — O(1).

- **`Arc`, not `Rc`** (honest deviation): `L1Value` must be `Send + Sync` (the evaluator holds values
  behind `Mutex`), so `Rc` fails to compile. Atomic refcounting is marginally costlier but still O(1);
  the measured win confirms it's negligible. The hand-written iterative `Clone` (~60 LOC) is deleted
  (now derived); `Drop` reworked to stay iterative for a *uniquely-owned* deep spine (`Arc::get_mut`),
  while shared subtrees drop O(1) — the 200k-deep `guard_hole_census` no-SIGABRT test still passes.
- **Measured (debug, `Empirical`), before (dev) → after:** n=100 0.393s→0.028s (14×), n=200
  2.965s→0.100s (30×), n=400 23.94s→0.375s (64×). **Fitted complexity p: 2.96 (~n³) → 1.86–2.01
  (~n²)** — one factor of n removed, speedup growing with n; the 1252-token case went from ~12 min
  (extrapolated) to ~4.0s.
- **Behavior-preserving (the §6 landing basis):** the **M-210 differential (32/32) is green and
  UNCHANGED** — no fingerprint/error edited; all `compiler_stage*` + conformance + lib tests green.
  Landed through the RFC-0041 §6 within-freeze hardening channel (identical values/errors/order).
- Folded in the PR #1189 LOW DRY nit (the `LetPop` Substrate-escape check → shared
  `substrate_escapes_into`).

**M-987 → done; M-994 → done.** With (a) (depth) + (b) (cost) both landed, the DN-26 §9 flag-2
**interpreted-first Stage-6 gate is now practical** at compiler scale; (c) AOT remains the fallback
for inputs beyond their reach. Unblocks the eval side of the semcore heavy-core port (M-993).

### M-994 fix (a) — widen L1 evaluator TCO through tail-transparent frames (M-986 closed) (2026-07-06)

Resolves the *depth* half of the M-994 decision (maintainer-approved: land (a) then (b), keep AOT as
the fallback). The L1 evaluator's TCO precondition ("no pending post-work") was too narrow — it
treated a `MatchPop`/`LetPop` frame above the caller's `InvokePost` as pending work, so a tail call
inside a `match` arm or `let` body was never elided, and since every terminating loop needs a `match`,
**no in-language loop could exceed the 4096 depth budget** (M-986).

- **The fix** (`crates/mycelium-l1/src/eval.rs::enter_call`, ~47 LOC): peek *through* any run of
  `MatchPop`/`LetPop` — observationally transparent to the value (they only restore scope) — so a tail
  call under them is still in tail position; on commit, drain them executing each one's scope cleanup
  eagerly (incl. the M-904 `LetPop` Substrate release for a non-escaping handle — never a silent leak).
  The non-tail path is byte-for-byte unchanged (peek-then-commit).
- **An append-only RFC-0041 §4.6 amendment** completing that section's ratified TCO intent (Decides
  item 5) — not new kernel surface. Maintainer-signed-off via the §6 within-freeze channel: it shifts
  the runs-vs-refuses frontier (so not purely §Posture-I2-behavior-preserving), but there is no L0
  oracle for these deep loops to diverge from (L0 has no TCO), and the **M-210 differential +
  `compiler_stage*` fingerprint parity are unchanged** — value-preserving for terminating programs.
- **Tests:** the two M-986 known-gap pins flipped to assert the closed behavior (a 10,000-iteration
  `match` loop now returns `Ok`, `total_elided ≥ 10000`; a 150-item nodule that refused at `depth=512`
  now passes), plus a **non-tail self-call still refuses `DepthExceeded{4096}`** guard (proving no
  over-elision). `compiler_stage3` 7/7; lib 367; differential 32/32 unchanged. **M-986 → done.**
- **Still open — M-987 (~n³ cost), fix (b) next:** (a) unlocks depth but an 800-item parse now runs
  yet is ~n³ *slow* (demonstrated live). Fix (b) — `Rc`-share `L1Value::Data` (O(1) clone; the
  confirmed root of the cubic) — is the affordability half; it lands behavior-preserving through the
  §6 channel. (a)+(b) together make the DN-26 §9 flag-2 interpreted-first Stage-6 gate practical.

### M-740 Stage 5 (increment 1) — partial self-hosted `compiler.semcore` (2026-07-06)

`boot10` (E18-1) wave 5, per DN-26 §7.3/§9: the **first, deliberately partial** increment of the
semantic-core nodule `compiler.semcore` (`lib/compiler/semcore.myc`). The full semcore is a 9-file
strongly-connected component (~16.7k Rust lines); this increment lands only the **tractable sub-core
that depends on checkty's *types* but not its logic or the evaluator**, and defers the heavy
entangled core — honestly, not silently.

- **In this increment:** the `Ty`/`Width`/`DataInfo`/`CtorInfo`/`Pat` type vocabulary (data
  declarations only), the Maranget **`usefulness`** (exhaustiveness/redundancy) + **`decision`**
  (decision-tree) pipeline, the static **`affine`** use-once tracker, and **`grade`** (guarantee
  grading). Flat-namespace prefixing (`Ty-`/`Wd-`/`Mp-`/`Hd-`) per the FLAG-ast-5/FLAG-parse-2
  discipline. Native `myc check` reports `ok`.
- **A true live-oracle differential** (`crates/mycelium-l1/src/tests/compiler_stage5_semcore.rs`,
  17/17, `Empirical`): because `usefulness`/`decision`/`affine`/`grade` are `pub(crate)`, the gate is
  an **in-crate** unit module (CLAUDE.md test-layout: white-box `use crate::…`) that calls the
  **live Rust oracle** on the same small synthetic inputs the `.myc` encodes and compares — not
  hand-derived expectations. The harness was perturbation-verified (a corrupted expectation fails
  loudly). This closed the first-cut hand-derived gap (FLAG-semcore-10). Sole residual
  **FLAG-semcore-10-b:** grade's exact `Strength` is recovered by probing the four-level lattice
  (`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`) through the live `check_guarantees`, whose finer
  internals are private even in-crate — surfaced, not hidden. **No logic module under
  `crates/mycelium-l1/src/` was modified and no visibility was changed** (in-crate access needs
  neither).
- **Deferred, feasibility-gated on M-986/M-987 (recorded as an open question, not narrowed):** the
  heavy entangled core `checkty`/`elab`/`eval`/`mono` + `fuse` (which *runs* the evaluator), the
  whole-program **L0-output differential**, and the `cargo-mutants` witness. Running a self-hosted
  checker/elaborator inside the L1 evaluator over a whole program almost certainly cannot complete
  under the current kernel (M-986: no in-language loop exceeds the 4096 depth budget; M-987: ~n³
  eval cost). Minted: **M-993** (heavy-core port), **M-994** (the L0-differential feasibility
  question). The lift is a maintainer decision (widen kernel TCO vs. reduce eval cost vs. lean on
  the AOT leg), surfaced in DN-26.

### M-740 Stage 4 — self-hosted SCC leaf nodules (substrate · totality · ambient) (2026-07-06)

`boot10` (E18-1) wave 4, per the DN-26 §7.3 stage map: Stage 4 lands the three semantic-core
**dependency leaves** as sibling nodules — `compiler.substrate` (`lib/compiler/substrate.myc`),
`compiler.totality` (`totality.myc`), `compiler.ambient` (`ambient.myc`) — the `.myc` port of
`crates/mycelium-l1/src/substrate.rs`, `totality.rs`, `ambient.rs`. Each depends only on `ast`
(already ported) or nothing, so none pulls in the entangled core.

- **Native-toolchain dogfood (new this wave):** the real `myc check` binary (`mycelium-check`)
  reports `ok` on all three nodules — and on the five previously-landed ones (`token`/`lex`/
  `nodule`/`ast`/`parse`) — so the self-hosted frontend is now vetted by the *actual toolchain*, a
  second independent witness alongside the Rust differential (this is the entry point that a
  `/myc-dogfood` gate will make repeatable; per-file today, project-level pending the M-982
  cross-nodule-execution lift). `mycfmt` parses all eight but reports them non-canonical, and
  *refuses* two (`lex.myc`/`parse.myc`) on the M-690 formatter limitation (trailing comment on a
  nested match arm) — filed as a toolchain-enhancement follow-up.
- **`compiler.substrate`** (DN-71 Model S): the deterministic surface of the affine handle —
  `SubstrateProvenance`, a threaded-`id` `acquire`, `explain`, `ReleaseEvent`, `SubstrateError`,
  and a value-threaded consume-once. **FLAG-substrate-1 (honest limit, not faked):** the Rust
  `Arc<AtomicBool>` consume-flag is shared across every clone of an identity — the runtime
  cross-alias use-once backstop; a pure-value port has no shared interior mutability, so
  `try_consume` here enforces use-once only along a single threaded value. A hand-written `itoa`
  fills the still-absent decimal-format prim (ast.myc FLAG-ast-7). Gate 5/5.
- **`compiler.totality`** (RFC-0007 Foetus structural-termination checker + the shared `walk_expr`
  traversal): `classify_all` Total/Partial over synthetic `FnDecl` sets, 6/6. FLAG-totality-1
  `BTreeMap`/`BTreeSet`→sorted assoc-list (documented deterministic-order precondition), -2 the
  `&mut impl FnMut` walks specialized to concrete accumulations (no HOF in the port), -3 the
  threaded 4096 `depth` budget replacing `mycelium_stack::with_deep_stack`, -4 `Pattern::Or`'s
  `panic!` invariant lowered to a dead `Ok` fallback (no panic prim). Split-match one-deep applied
  even to a `Bool` inside `Ok` — the usefulness checker rejects the combined pair (a real,
  minted-not-muted behaviour).
- **`compiler.ambient`** (RFC-0012 ambient-representation resolution + canonical pretty-printer):
  `resolve`/`resolve_report`/`expand_to_source`/`expand_phylum_to_source`, `MAX_AMBIENT_DEPTH`=4096,
  the two mirror traversals over the widest AST enums. Gate 4/4: byte-for-byte `expand_to_source`
  parity + an AST fingerprint on accepts, 5-way error-kind parity on refusals.
  **FLAG-ambient-6 (differential scope, flagged not silent):** the gate covers 8 hand-built
  synthetic nodules + 4 refusal fixtures and **zero raw conformance-corpus files** — a *structural*
  limit, not the M-987 cost wall: `compiler.ambient` operates on an already-parsed `Nodule` and
  cannot reach `compiler.parse` (cross-nodule execution staged, M-982), so a source file can't be
  fed without an AST-serializer bridge (deferred). FLAG-ambient-2 (`resolve_report` `notes` honestly
  empty), -3 (error message text not rendered; 5-way classification compared), -4 (no
  `with_deep_stack` analogue).
- **Honesty (VR-5, flagged in-file):** every differential is graded `Empirical`; all narrowings
  carry in-file `FLAG-<nodule>-N` comments; nothing under `crates/mycelium-l1/src/` was modified
  (the Rust frontend stays the trusted oracle until M-741).

### M-740 Stage 3 — self-hosted AST vocabulary + parser (2026-07-05)

`boot10` (E18-1) wave 3, per the DN-26 §7.3 stage map: Stage 3 lands `compiler.ast`
(`lib/compiler/ast.myc`) and `compiler.parse` (`lib/compiler/parse.myc`) — the `.myc` port of
`crates/mycelium-l1/src/ast.rs` and `parse.rs`, with the corpus-wide parser differential.

- **`compiler.ast`:** the full surface-AST vocabulary (36 types / 102 constructors, the small
  helper impls). FLAG-ast-1..8 in-file: `String`→`Bytes`, `u32`→`Binary{32}`, lossless
  `i64`→`Binary{64}`; recursive types need no `Box` (two-pass shell-then-resolve registration,
  verified before authoring); keyword renames reuse token.myc spellings; **FLAG-ast-5 is a new
  collision class** — the per-nodule constructor namespace is flat, so bare variant names reused
  across *different* enums collide even when none is a keyword (per-type prefixes, the
  `collections.myc` precedent); `BTreeMap`→ordered assoc-list; `WidthRef` `Display` and
  `#[non_exhaustive]` not ported (flagged, never silent). Gate `compiler_stage3_ast.rs` 26/26:
  parse+`check_nodule`, a 103-row ported-constructor inventory (`Declared`/audited), and a
  per-variant L1-eval construct-and-classify exercise.
- **`compiler.parse`:** all ~91 `parse.rs` functions accounted for; **both `parse` and
  `parse_phylum` ported end-to-end** (source text → AST, self-contained token+lexer+AST copy per
  M-982, FLAG-parse-1). Every match destructures exactly one constructor level (the M-980
  discipline — zero checker panics across the ~4,400-line nodule); `MAX_EXPR_DEPTH`=4096
  preserved; the source-length-bounded loop discipline is per the RFC-0041 §7 W7 amendment-11
  TCO acceptance criterion (see the PR #1166 review cycle below for the list-building-loop
  re-shape that made it hold).
- **The gate** (`crates/mycelium-l1/tests/compiler_stage3.rs`, 4/4 green, `Empirical`):
  classification parity vs the live Rust oracle over the **full conformance corpus on both legs**
  (accept 27/27 — 26 via `parse`, the phylum-headed file via `parse_phylum` with `parse` refusing
  it on both sides; reject 30/30; zero divergences), a preorder per-constructor-tag AST
  fingerprint (tag table 1–109, `rotl(7)`-XOR mix, node count, `Bytes`-length/`u32` leaf mixing;
  hand-locked Rust mirror walk) on every accepted leg, and a 6-file real-stdlib subset leg —
  171s wall via the args-in/verdict-out one-eval harness, ~8× cheaper than the per-driver shape
  (retrofit of the Stage-1/2 gates minted as **M-983**; the full lib-tree sweep, post-M-981
  economics, as **M-984**).
- **New finding (FLAG-parse-2):** the lexer-keyword-ctor × AST-ctor flat-namespace collision
  (31 `T`-prefix names) surfaces whenever two frontend stages share one nodule — recorded
  append-only in DN-26 for the Stage-5 semcore packaging.
- **Honest narrowings (VR-5, flagged in-file):** L1-eval leg only — no three-way at this scale
  (M-981); error message/position fidelity not compared (classification only, FLAG-parse-8);
  eval fuel sized to 200M for the lib leg (evaluator default 1M — flagged as a maintainer call,
  not decided). The Stage-1 lexer narrowings carry over verbatim with the lexer copy.
- **Review cycle (PR #1166):** the `/pr-review` pass caught a real HIGH — the list-building
  loops were `Cons`-after-return (not direct-tail; reproduced at 5,000 items via
  `DepthExceeded{4096}`). Fixed by converting all 27 source-length-bounded loops to
  accumulator + `rev_acc` direct-tail shape (fingerprint parity re-verified, zero divergences).
  The fix surfaced three kernel-side findings, minted not muted: **M-986** — the evaluator's
  TCO elides only bare-body self-calls, so tail calls inside `match`/`let` are never elided
  and *no* in-language loop can exceed the 4096 depth budget today (the source shape is the
  ready form; its depth benefit is dormant until the kernel widens tail position — pinned by
  loud known-gap tests); **M-987** — L1-eval cost ~n³ in token count (0.6 s / 26 s / 133 s at
  200 / 752 / 1,252 tokens, debug); **M-988** — mono re-inference rejects generic bare `Nil`
  the checker accepted (55 explicit ascriptions as the workaround). Also fixed: the stale
  107-entry tag-table comments (→ 109) and the stale line count. The Stage-1 lexer's own
  non-tail twin is **M-985** (pre-existing, never claimed, now flagged). Post-patch gate:
  `compiler_stage3` 6/6 green.

### M-740 Stage 2 — self-hosted nodule-header recogniser (2026-07-05)

`boot10` (E18-1) continues per the DN-26 §7.3 stage map: Stage 2 lands the `compiler.nodule_header`
nodule (`lib/compiler/nodule.myc`), the full `.myc` port of `crates/mycelium-l1/src/nodule.rs`
(the DN-06 §6 first-non-blank-line `// nodule[: name]` marker recogniser).

- **The port:** `parse_nodule_header` (blank-line skipping, 1-based line tracking), the
  bare/named-marker recogniser, never-silent ill-formed-name errors (empty name, empty segment,
  non-identifier segment — G2), and the `dotted`/`canonical` accessors. Every
  source-length-bounded recursion is direct-tail (the RFC-0041 §7 W7 amendment-11 TCO acceptance
  criterion for M-740).
- **The gate** (`crates/mycelium-l1/tests/compiler_stage2.rs`, 3/3 green, `Empirical`): one
  three-way run (L1-eval ≡ L0-interp ≡ AOT — feasible at this stage's small scale, unlike Stage 1's
  lexer, M-981) plus a 26-case synthetic edge battery transcribed from the oracle's own unit tests
  plus the header-parse differential against the live Rust oracle over every `.myc` file in the
  conformance corpus (accept and reject) and `lib/std/` plus `lib/compiler/` — 66+ files, comparing
  the 4-way classification code, the joined dotted name plus `canonical` spelling (named case), and
  the 1-based error line (error case).
- **One real dogfooding finding (FLAG-nodule-5):** DN-26 §7.3's nodule name `compiler.nodule` is
  unspellable — `nodule` is a reserved word, so the surface declaration `nodule compiler.nodule;`
  cannot parse (the FLAG-token-3 keyword-collision class at the nodule-NAME level). The stage ships
  as `compiler.nodule_header`; DN-26 carries the append-only correction note (status stays Draft).
- **Honest narrowings (flagged in-file, VR-5):** ASCII-only trim vs Rust's Unicode `str::trim`
  (FLAG-nodule-2, the FLAG-lex-4 analog); static error messages with line fidelity kept
  (FLAG-nodule-3); the per-file sweep runs the L1-eval leg only (M-981, as in Stage 1).

### Kickoff-corpus reconciliation (2026-07-05)

Post-Phase-I doc maintenance on the kickoff corpus (`.claude/kickoffs/`) plus `docs/CURRENT-STATE.md`
— documentation only, no code changes.

- **Seven kickoffs archived** (moved to `.claude/kickoffs/archive/`): the six completed — `acy`
  (H0, commits 6636f56/ba0b800, E27-1), `enb` (H1) and `opp` (both PR #1020, 2026-07-02), `grm`
  (H2a) and `frz` (H2 — the kernel freeze, both PR #1051, 2026-07-02), and `trx` (transpiler PoC,
  landed 2026-07-01) — each with a prepended completion header whose task ranges were verified
  `status:done` against `issues.yaml`; plus `rcp`, the superseded, never-executed predecessor
  umbrella plan (replaced 2026-07-01 by ADR-038's function-first decomposition into
  acy/enb/grm/opp/frz).
- **`.claude/kickoffs/README.md` refreshed to 2026-07-05 truth:** a "Landings 2026-07-02 → 07-05"
  masthead note (incl. the RFC-0041 W0–W7 promotion — RFC-0041 → Enacted, DN-84 → Resolved,
  M-978/M-979 closed, the M-969/M-959 status lags corrected), the Phase-I table collapsed to its
  archive pointers, six new Completed rows and the `rcp` pointer, and the scheduler plus the
  dependency-sequencing section re-pointed (stale gates dropped; the maintainer's reserved queue
  listed).
- **`boot10` is the next engineering kickoff** (maintainer decision, 2026-07-05): M-740's M-978
  gate cleared by the promotion (M-739 still gates it); two straggler leaf branches recorded for
  rescue-first; TCO's direct-tail-only scope recorded as an M-740 acceptance criterion (RFC-0041
  §7 W7 amendment #11); M-970 (FLAG-970 formatter bug, P3) rides the first wave as a disjoint
  `mycelium-fmt` leaf.
- **`dfb` RE-SHELVED** (maintainer decision, 2026-07-05) until after `boot10` and the public flip —
  dated notes appended to the M-670/M-671 bodies in `issues.yaml` (statuses stay `blocked`, no
  label changes); `tul` chain updated in place (M-675 done 2026-07-01, only M-676 remains, P3).
- **`docs/CURRENT-STATE.md` stale claims fixed:** kernel FROZEN 2026-07-02 (DN-56 → Enacted on the
  DN-76 4/4 green scorecard plus KC-3 review; it previously said 1/5 conditions met), RFC-0041 →
  Enacted 2026-07-05 (recursion-depth safety landed, the `myc run` SIGABRT closed), DN-84 →
  Resolved.

### RFC-0041 promotion to `main` — recursion-depth safety Enacted with this landing (2026-07-05: M-978 · M-979 · M-959 · M-969 status reconcile)

The maintainer approved the full promotion (2026-07-05); the reconciled W0–W7 wave moves
`dev → integration → main` by this landing (RFC-0041's §9 Enacted-claimability condition is met the
moment this entry reaches `main`). Status moves, all append-only with a checked basis:

- **RFC-0041 `Accepted → Enacted`** — every §9 DoD line literally met or honestly re-scoped by the
  recorded §7/§9 amendments; the flagship `myc run` SIGABRT refuses `DepthLimit{4096}`; §5.1
  error-parity green; one deterministic budget on every path; `#![forbid(unsafe_code)]` intact.
  The W7 follow-ons (W3b bare-`Repr`, `count_occurrences` O(N²) work-step bound, single-variant
  unification, AOT per-frame precision, `content_hash` O(depth²), coarse-worker sites, the
  geiger-baseline `--update` regeneration — the committed baseline is a disclosed W0 placeholder)
  stay tracked, not silent.
- **DN-84 `Draft → Resolved`** — designs (B)/(C)/(D) all delivered via RFC-0041.
- **M-979 and M-978 → done** — (D) the work-stack conversion and (B) grow-on-demand plus the unified
  budget respectively (M-978 was subsumed as RFC-0041 W1/W2/W7 rather than a separate RFC).
- **Bookkeeping-lag corrections (G2):** **M-969 → done** (the kernel freeze was *executed*
  2026-07-02 — commit b211cca, PR #1050→#1051 — but its issue stayed `blocked`) and **M-959 → done**
  (DN-80 Accepted plus the `reject_ledger.rs` regression guard green plus DN-76 §5A.1 condition-1
  GREEN predate the freeze; the `todo` was a lag).
- **M-740** (the `.myc` frontend port, self-hosting capstone): its RFC-0041 blocker (M-978) is
  cleared by this promotion; **M-739** (the DN-26 bootstrap plan, `needs-design`) still gates it.

### RFC-0041 W7 — Enacted-closure wave: the §9 DoD open items closed or honestly re-scoped (2026-07-03: M-979)

Closes the maintainer-held open items from the post-implementation assessment. Determinations were made by
the maintainer on a **Fable plan/QC assessment** of all twelve open items; the wave ran as four disjoint
isolated-worktree leaves (per-leaf reviewed; the `mir-passes` leaf independently adversarially
memory-safety-reviewed — no Critical/High). Held at `dev`.

- **`--unbounded` implemented (Rust-first).** `myc run --unbounded` lifts the recursion budget via the new
  additive `Interpreter::with_depth(u32::MAX)` with a never-silent stderr banner; the corpus/conformance
  runner refuses `--unbounded` (test-guarded, exit 64). `myc build --unbounded` is interface-parity only
  (frontend l1 ceilings are not CLI-tunable yet — tracked follow-on).
- **`mir-passes` recursion guarded (guard-and-refuse).** `eval(&RcNode)`, `emit_elided`/`emit_reuse` charge
  the shared `RecursionBudget` on every RcNode edge and refuse never-silently with `DepthExceeded`; the
  public infallible counters are deep-stack-wrapped. No input SIGABRTs any `mycelium-mir-passes` pass. The
  `count_occurrences` O(N²) work-step bound stays a documented DoS-only residual deferred to W2.
- **Process-arena coverage closed for untrusted-reachable paths.** A coverage audit
  (`docs/notes/W7-arena-coverage-audit.md`) found `ProcessArena` had zero consumers; the two
  untrusted-reachable allocation-proportional passes (LSP `llm_canonical`, `fmt` render family via new
  `FmtError::OutOfBudget`) now charge it and refuse with `OutOfBudget`. Unreachable/non-proportional passes
  are explicitly exempt with the audit as the `Empirical` basis.
- **Frozen-core hardening (test-only, no logic change).** A `Value`/`Repr` construction-gate census upgrades
  "a deeply-nested `Value` is unbuildable" to `Empirical`, and a Box-owned spine tripwire fails if `Rc`/`Arc`
  appears on the frozen `Node`/`Datum` spine. **Correction (VR-5):** §4.5's "`Value`/`Repr` … unbuildable"
  overclaims for a bare `Repr` (constructible by a direct variant literal, no gate) — scoped to `Value`;
  bare-`Repr` iterative destruction folds into the coordinated W3b.
- **`with_depth` parity check** verifies the `DepthExceeded{u32}`↔`DepthLimit{usize}` family mapping at
  arbitrary small budgets (ceilings {1,2,8,100}), not just the floor.

**Amendments (append-only, RFC-0041 §7/§9):** no-alloc-in-Drop scoped to "no abort except genuine OOM during
deep unwind" (#5); the §4.5 class scoped to constructible types with `Value`/`Repr` → W3b (#6); the arena to
"every allocation-proportional path reachable from untrusted input" (#8); the AOT per-frame metric ruled a
precision follow-on under the §5.1 family-parity contract (#3). TCO's direct-tail-only scope becomes an
explicit M-740 acceptance criterion (#11); the W6 wide-tuple "document" resolution upheld (#12). With W7,
every §9 DoD line is literally met or honestly re-scoped with a checked basis — **whole-RFC `Enacted` is
claimable once W7 lands on `main`** (RFC stays `Accepted` until then). `#![forbid(unsafe_code)]` intact.

### RFC-0041 W6 — data-spine iteration: the wide-tuple asymmetry documented (RFC-0041 Phase-4 COMPLETE) (2026-07-03: M-979)

The final wave, and an **assess-then-act** one (the RFC §4.7 explicitly permits "convert **or** document
the wide-tuple asymmetry"). **Decision: document — conversion not warranted** (evidence-based, VR-5):
- The `usefulness::useful` / `decision::compile_rows` pattern-matrix passes recurse ~N-deep on tuple/ctor
  **arity** (data-shaped width), and a 4095-field product type is surface-reachable and false-refuses at
  the 4096 floor. **But** on the production 256 MiB deep-stack worker that refusal is a **clean never-silent
  `DepthExceeded{4096}`, not a SIGABRT** — the W1 budget guard already meets the "no input SIGABRTs" DoD.
  The residual is a *precision* defect (a shallow-but-wide pattern refused as if deeply nested), not a
  safety one, and a 4095-field product type is pathological (unlike the realistic list literals the twin
  `check_list` handles — converted in W1). A byte-identical iterative rewrite of the trusted *branching*
  Maranget passes is high-risk for ~zero real benefit (KISS/YAGNI/KC-3), and §7 gates the twin's conversion
  on "if profiling demands" — which it does not.
- **Change is docs + tests only** (no logic change; differential + conformance byte-identical): grounded
  `§4.7 (W6)` notes on `useful`/`compile` marking the measured boundary, the safety property, and the exact
  conversion seam (charge `charge_steps` per column, like `check_list`) should the maintainer overrule;
  boundary witness tests pinning the never-silent clean refusal. **Flagged for maintainer:** overrule →
  convert only if 4095-arity is deemed adversarially realistic (§5 untrusted-input lens).

**RFC-0041 Phase-4 — all seven implementation waves (W0–W6) have landed** on the working tier: the flagship
`myc run` SIGABRT (RR-29 §0.1) is closed, the §5.1 cross-path error-parity gate is green, the frozen-core
value types + all three execution machines (L0 interp · L1 eval · AOT) refuse deep input never-silently on
one shared budget, and the host stack grows on demand. `#![forbid(unsafe_code)]` holds across every landed
crate (the only `unsafe` is the audited upstream `stacker`/`psm`). **The core recursion-safety contract
holds end-to-end.** RFC-0041 stays **Accepted** with per-wave `Enacted` scopes; **whole-RFC `Enacted` is
NOT yet claimable** (VR-5): a post-implementation assessment surfaced genuine **§9 DoD open items** — the
DoD-required **`--unbounded` mode** was decided (DN-84 §9.3) but never scheduled/implemented; the
`mir-passes` `eval(&RcNode)` recursion hole (a §4.7-listed crate) is still unguarded, so DoD §9 item 1
isn't literally met there; and an AOT per-frame-vs-source-call metric reconciliation is owed (§4.0/§4.4).
These, plus the flagged deviations (RFC §5.1 amendment, §7 status, M-979 issue), **await maintainer
determinations**; M-740 self-hosting unblocks once they're resolved.

### RFC-0041 W4 — L0 reference-interpreter budgeted work-stack; the flagship `myc run` SIGABRT closed (2026-07-03: M-979)

Closes RR-29 §0.1 — the remotely-reachable flagship bug: a crafted deep-but-fuel-cheap `.myc` value
**SIGABRT-ed `myc run`** (the trusted L0 interpreter, reachable via a hostile spore). It now **refuses
cleanly with `EvalError::DepthLimit{4096}`**. `#![forbid(unsafe_code)]` intact.

- **Budgeted the substitution machinery** (`mycelium-interp`): `step`/`subst`/`node_to_core_value`/
  `guarantee_of_value`/`select_arm` thread the shared `RecursionBudget`, charging one `DepthGuard` per
  structural descent (siblings don't accumulate). Per §4.1 the L0 machine **stays a substitution
  machine** — only budget + guard threaded in (`subst` became fallible); `eval_core` runs on the
  growable deep stack (`ensure_sufficient_stack`).
- **Constructed `EvalError::DepthLimit`** (defined-but-never-built until now) via `From<BudgetError>`
  (canonical `DepthExceeded{u32}` → `DepthLimit{usize}` at the same threshold). `myc run` is fixed with
  no CLI change; W3's iterative `Node::clone` composes so the front-door deep clone no longer aborts.
- **`parallel::is_pure` made iterative** (explicit work-stack, O(1) host stack) — closes its own W4
  census hole; the four W4-tagged census tests are un-ignored (deep value/subst → `DepthLimit`;
  `is_pure`/`plan_parallel` complete without aborting).
- **§5.1 error-parity gate GREEN** (un-ignored). **Evidence-driven finding (VR-5):** the RFC's original
  "one statically-deep source refused identically by all three paths" premise proved *empirically
  unachievable* — the parser cap now equals the eval floor (a deep *source* is refused at parse), the
  AOT trampoline is data-spine-immune, and L0 substitution is `O(N²)` on runtime recursion. The gate was
  rewritten to assert the parity that **actually holds** — every path refuses over-budget with the
  canonical variant *family* at the shared 4096 floor, each exercised with a bounded-time input (L1 on
  `spin` → `DepthExceeded{4096}`; L0 on a deep value → `DepthLimit{4096}`; AOT on `spin` at the explicit
  floor → `DepthLimit{4096}`). **Residual flagged:** literal single-*variant* unification is partial (L1
  `DepthExceeded` vs L0/AOT `DepthLimit`); full convergence would change the interp/AOT error enums
  (trusted-base observable) — a maintainer-decision follow-up. Independently adversarially reviewed.
  RFC-0041 stays **Accepted**; W4 `Enacted`. **Only W6 remains.**

### RFC-0041 W3+W5 — frozen-core iterative destruction, L1-eval CEK machine, TCO, eval raise (2026-07-03: M-979)

The coordinated frozen-core/reference-machine pair (maintainer-approved past the checkpoint). Kills the
deep-recursion `SIGABRT` on the kernel value types and the L1 evaluator, and raises eval's depth budget
to the workspace default. `#![forbid(unsafe_code)]` holds throughout — safe `Box`/`Vec` + `mem::{replace,
take}` only.

- **W3 — frozen-core iterative destruction (`mycelium-core`, via the DN-56 §6 within-freeze channel).**
  `Node`/`Datum`/`CoreValue` gain manual **iterative** `Drop`/`Clone`/`PartialEq` and iterative
  `Canon::node`/`content_hash` (a single shared heterogeneous `Datum↔CoreValue` worklist; `mem::replace`
  take-loops), so a deep spine no longer overflows on destruction/clone/hash — including on the refusal
  path and the caller stack. **Within-freeze bar met:** Clone/PartialEq/hash are **bit-identical** to the
  derived forms (witnessed against a recursive reference oracle across all variants and binder scopes);
  M-210 3-way differential green; mutation-witnessed (100k-deep construct/destruct/clone/unwind, incl. an
  alternating `Datum↔CoreValue↔Value` chain); only recursion→iteration transforms, no new
  type/variant/field. The **doc-IR `mycelium-doc::ir::Node`** member gets the same iterative `Drop` (a
  tooling crate — no freeze channel; the W1 `mem::forget` test workarounds are removed).
- **W5 — L1 evaluator → CEK machine (`mycelium-l1`).** The 7-fn recursive eval SCC is now one explicit
  `Vec<Frame>` work-stack (O(1) host stack), reifying the interleaved post-child work (scope push/pop,
  `release_if_abandoned`/`ReleaseEvent`, guarantee asserts) as continuation frames; error paths unwind
  the work-stack running each frame's cleanup (never-silent, G2). `L1Value` gains iterative `Drop`/`Clone`
  (a deep `Cons` value no longer SIGABRTs — witnessed at 200k). **TCO** is applied **only** under the
  no-pending-post-work precondition (no `sig.ret.guarantee` index and no Substrate-typed param — so a
  Substrate release / return-guarantee assert is never silently skipped; both mandatory witness tests
  pass, and only *direct* tail calls are elided — a safe under-approximation), with a bounded EXPLAIN
  ring buffer of elided calls. **`DEFAULT_DEPTH` raised 64 → 4096** on the shared budget.
- **Honest deviations / residuals (VR-5/G2 — flagged for maintainer, not silenced):**
  1. **Zero-alloc-in-`Drop` not achieved.** The RFC (§4.5) wanted an alloc-free iterative `Drop`; that
     needs either a new next-pointer field (barred by the within-freeze bar) or `unsafe` pointer-reversal
     (barred by `forbid(unsafe_code)`), so the iterative Drops use an empty-start `Vec` worklist. This
     trades a *certain* multi-MB stack-overflow abort for a small alloc that fails only under genuine
     OOM-during-deep-unwind — a net safety gain, but the OOM-unwind edge remains. **Maintainer call:**
     accept the tradeoff, or relax a constraint (a contained `unsafe` leaf, or a DN-39 review to add a
     next-pointer field).
  2. **`Value`/`Repr` deliberately not converted (deferred to a coordinated W3b).** Deep `Seq` values are
     *construction-gated* (`Value::new`/`check_well_formed`/serde all recurse on `Seq.elem`), so a deep
     `Value` cannot be built — its recursive `Drop` is unreachable, and converting only its
     Drop/Clone/eq/hash would be un-mutation-witnessable on the most identity-critical frozen type. W3b
     makes the *construction* path iterative together with destruction.
  3. Pre-existing `content_hash` `O(depth²)` for deeply-nested-*binder* terms (de Bruijn linear scan);
     the L1-eval per-node-vs-source-call metric residual (§4.0, W5-documented); `PartialEq` stays derived
     where no deep-compare path exists. All tracked.
- **§5.1 error-parity gate stays `#[ignore]`** — the L1-eval path now refuses with `DepthExceeded{4096}`,
  but the cross-path gate needs **W4** (the L0 interp still has no budget). RFC-0041 stays **Accepted**;
  W3+W5 `Enacted` for their scope.

### RFC-0041 W3½ — AOT env-machine extraction onto the shared budget (2026-07-03: M-979)

Fourth RFC-0041 wave and the **last before the frozen-core checkpoint** — a **behavior-preserving**
refactor: the AOT `Vec<Frame>` env-machine (`mycelium-mlir`) now charges the shared
`mycelium_workstack::RecursionBudget` and grows via `ensure_sufficient_stack`, instead of its own
ad-hoc `stack.len() >= max_depth` ceiling. The reference oracle is **unmoved**.

- Both frame-push sites (`enter_apply` for App/Fix/FixGroup, and the `Match` continuation) now call
  `RecursionBudget::try_enter`, holding the `DepthGuard` for the frame's lifetime (so
  `budget.current_depth() == stack.len()` at every enter). `BudgetError::DepthExceeded{u32}` maps to
  the **unchanged** `EvalError::DepthLimit{usize}` at the same limit — byte-for-byte the prior
  threshold, so `recursion_differential.rs` and the three-way `differential.rs` stay green with **zero**
  expected-value edits (that *is* the behavior-preserving gate).
- The machine entry is wrapped in `ensure_sufficient_stack`; the DN-05 floor/headroom split is
  preserved (§4.4 — the differential runs at the floor, the dynamic `[10k, 2M]` stays as headroom).
- Resolves the W2 residual: an in-crate `size_of::<Frame>() <= MAX_FRAME_BYTES` pin (Frame ≈ 336 B with
  the added `DepthGuard`, under the 384 baseline it set).
- **Honest flag (VR-5):** the AOT charges **per live control-stack frame** (App *and* Match
  continuations), not purely per §4.0 source-call boundary — this is **identical to pre-W3½ behavior**
  (why the differentials are unmoved); the per-frame-vs-source-call reconciliation is W5's job. W3½
  `Enacted` (AOT extraction scope); RFC stays **Accepted**.

### RFC-0041 W2 — host-stack grow, startup assertion, frame baseline (2026-07-03: M-979)

Third RFC-0041 wave — the host-stack **grow** infrastructure and the never-silent no-grow refusal.
Introduces the workspace's first *new* unsafe **dependency** while authoring **no** unsafe (the
stack-switching unsafe stays contained upstream in `stacker`/`psm`, called via their safe API — ADR-014).

- **`mycelium-stack` fine-grained grow** (still `#![forbid(unsafe_code)]`): `stacker = "=0.1.24"`
  (exact-pinned; pulls `psm 0.1.31`), an `ensure_sufficient_stack`/`grow` wrapper (stride-1, rustc's
  pattern), a **runtime** growth-availability probe (`remaining_stack`, not a cargo feature — §4.3), and
  `growable_ceiling_honors_floor` — which **refuses to start with an explicit error** on a no-grow target
  (wasm; `psm` is a silent no-op there) whenever the fixed ceiling would fall below `floor × frame`,
  never a silent SIGABRT below the floor (§4.3, G2).
- **`mycelium-workstack`**: `ensure_sufficient_stack`'s body now routes through the runtime-gated grow
  layered on the deep-worker base (signature unchanged; non-regressing — a bare top-level grow would
  reintroduce the deep-input SIGABRT); a `check_startup` gate wiring `assert_mem_ceiling_honors_floor`
  with `MAX_FRAME_BYTES = 384` (the §4.2 determinism invariant).
- **Frame-size CI baseline** (§4.2, the ADR-041 lesson): pins `size_of` of the machine value structs
  (`CoreValue`/`Node`/`L1Value`, all 240 B) at or below `MAX_FRAME_BYTES` so a toolchain frame-size bump
  fails CI, not production (in `mycelium-l1/tests/`; the private AOT `Frame` = 328 B, which sets the
  baseline, is a tracked residual pin for `mycelium-mlir`).
- **Supply chain**: `THIRD-PARTY-LICENSES.md` regenerated (stacker/psm plus their transitive tree; both
  MIT/Apache-2.0, first-party stays MIT); `mycelium-workstack` added to the unsafe-per-use audit; the
  cargo-geiger baseline remains a documented placeholder (tool absent in-env) — noted in `about.toml`.
- **Honest scope (VR-5/G2):** W2 lands the grow *infrastructure* and the never-silent no-grow *refusal*.
  The per-recursion-point stride-1 grow (replacing the coarse worker) is consumer-side wiring that lands
  as the evaluators convert (W3½/W4/W5); the W1 infallible-pass memory-DoS bound still awaits
  arena-charging. Both tracked, not silenced. RFC-0041 stays **Accepted**; W2 `Enacted` for the
  grow/startup scope.

### RFC-0041 W1 — budget crate + frontend guard wiring (2026-07-03: M-979)

Second wave of RFC-0041 — the first **behavior-changing** one. Introduces the shared budget core and
closes the frontend-tool + checker guard holes so deep input **refuses never-silently** (or renders on
a grown stack) instead of SIGABRT-ing. Ran as scaffold-first + a disjoint 6-leaf swarm.

- **New leaf crate `mycelium-workstack`** (`#![forbid(unsafe_code)]`, downward-only DN-68) — the
  canonical home of the never-silent **`RecursionBudget`**: a depth ceiling on the §4.0 metric
  (default 4096), a memory ceiling, a work-step (CPU) ceiling, a process-wide **`ProcessArena`** (an
  atomic byte counter so concurrent passes can't sum past a per-process ceiling), the canonical
  over-budget surface **`BudgetError::{DepthExceeded{limit:u32}, OutOfBudget{…}}`**, a thin
  `ensure_sufficient_stack` guard helper (W1 delegates to the 256 MiB `with_deep_stack` worker; W2
  swaps in fine-grained `stacker`), and the `assert_mem_ceiling_honors_floor` §4.2 invariant (checked
  fn; wired at startup in W2). Consumer-side charging (the leaf never depends on `interp`/`core`/`l1`
  — the §4.1 deps-cycle fix). 18 in-crate tests incl. isolation + mutant-witness; added to
  `cargo-mutants` scope.
- **Frontend guard holes closed** (§4.7) — each pass now wraps its outermost entry in
  `ensure_sufficient_stack` and/or charges the budget: **`mycelium-l1`** checker
  (`usefulness`/`decision` now return `Result<_, BudgetError>`, `grade` maps to `CheckError`) +
  **`check_list` routed through iteration** (the data-vs-control fix — a work-step per element, O(1)
  control depth, byte-identical checking for concrete types) + **parser `MAX_EXPR_DEPTH` 256 → 4096**
  (verified safe on the deep-stack worker; eval's 64 held to W5); **`mycelium-fmt`** render family;
  **`mycelium-lsp`** `render_node` (the editor-buffer priority surface); **`mycelium-transpile`** emit
  (new `GapReason::RecursionBudget`); **`mycelium-doc`** `Node::walk`; **`mycelium-mir-passes`**
  `emit_owned` (new `EmitError::DepthExceeded`) + `count_occurrences` (grown; the O(N²) re-walk flagged
  as a W2 residual). The 14 frontend census tests are **un-ignored + passing**.
- **Scoping correction (never-silent, G2):** two W0-census holes touch the trusted base — `write_canon`
  (frozen `mycelium-core`) and `is_pure`/`plan_parallel` (`mycelium-interp`) — so they were **deferred
  off W1** (re-tagged W3 / W4) to land with the maintainer checkpoint, not in the frontend wave.
- **Honest scope of the infallible-pass fix (VR-5/G2):** passes with infallible signatures
  (`fmt`/`lsp`/`doc` render, `count_occurrences`) are *grown onto the 256 MiB worker* — this **raises
  the overflow threshold, it is not yet a hard never-silent refusal** (input past ~256 MiB still
  aborts). Their memory-ceiling `OutOfBudget` refusal lands in **W2** (fine-grained grow + the real
  ceiling). The *fallible* passes (`l1` checker, `transpile`, `mir` `emit_owned`) already refuse
  never-silently at 4096 this wave. Also: W1's coarse `ensure_sufficient_stack` spawns a 256 MiB
  worker thread per top-level call (a transitional cost + concurrency memory-pressure vector) — W2's
  in-place `stacker` removes the spawn.
- **Residual guard holes surfaced by the swarm** (tracked, not silently closed): the recursive-`Drop`
  bomb on deep fixtures (the W3 class — `mycelium-doc::ir::Node` is a **new** member found this wave);
  `mycelium-mir-passes` `eval(&RcNode)` / `emit_elided` / `emit_reuse` and the `count_occurrences`
  O(N²) re-walk (W2); `syn`'s own unbudgeted parser recursion (third-party, dev-tool only).
- RFC-0041 stays **Accepted**; W1 is `Enacted` for the frontend scope. Verified: full `just check` green
  (differential + census; the §5.1 error-parity gate stays `#[ignore="W5"]`).

### RFC-0041 W0 — recursion-depth safety net (gates + metric + census; 2026-07-03: M-979)

First implementation wave of the Accepted **RFC-0041** (recursion-depth safety) — a pure **safety
net**: no behavior change, no frozen-core edits. Establishes the measurement + regression
infrastructure the consequential waves (W1–W6) land against.

- **§4.0 depth metric** — a pure source-call-boundary depth function + property/mutant-witness tests
  (`crates/mycelium-l1/tests/depth_metric_parity.rs`): one unit per user-`App`/`Fix` boundary (n-ary
  `f(a,b,c)` is depth 1), data-spine charged by element. Passing.
- **§5.1 error-parity differential** — a cross-path (L1-eval · L0-interp · AOT) over-budget
  differential asserting all three refuse with the **canonical** variant at the same metric
  threshold. Tagged **`#[ignore = "W5"]`** — it fails today (paths diverge; the L0 interp has no
  budget), green when W4 constructs the interp budget and W5 aligns eval. Canonical over-budget
  variant decided: **`DepthExceeded { limit: u32 }`** on the §4.0 metric (→
  `mycelium-workstack::RecursionBudget`, W1); the interp/AOT env-machine `EvalError::DepthLimit{usize}`
  reconciles to it in W4/W3½.
- **Guard-hole census** — one `#[ignore = "Wn"]`-tagged real-repro test per RR-29 guard hole across
  eight crates (`tests/guard_hole_census.rs`), each tagged with the wave that closes it (W1 frontend ·
  W3 value-drop · W4 L0-interp · W5 eval); infallible-signature holes documented honestly (VR-5),
  not faked.
- **Depth-structured fuzz** — `fuzz_depth_{parse,check,interp}`; the interp target **empirically
  reproduces** the known L0-interp `SIGABRT` (RR-29 §0.1 — a `Node::clone` stack overflow), the
  regression net the fix waves close.
- **Durability scope** — `mycelium-l1` + `mycelium-mlir` added to `just mutants` (the depth guards
  were unmutated — RR-29 §4); `mycelium-stack` added to the unsafe-per-use audit-A; a cargo-geiger
  baseline scaffold (the real baseline + `stacker`/`psm` exact-pinning deferred to W2 when they land).
- Also unblocks two pre-existing gate failures inherited from the dep-refresh wave: regenerated
  `THIRD-PARTY-LICENSES.md` (dep-version drift) and a `.codespellrc` skip for the generated
  `package-lock.json` integrity-hash false-positive. RFC-0041 stays **Accepted** (each wave moves
  `→ Enacted` only when the full cross-path differential goes green).

### DN-84 — dynamic host-stack + unified deterministic depth budget (2026-07-03: M-978 · M-979)

New Draft design note **DN-84** capturing the direction to make the recursive frontend crash-proof
(no host-stack `SIGABRT`) with essentially-unbounded, cleanly-handled nesting — while preserving
never-silent (G2), determinism, KC-3, and self-hosting portability. Maintainer decisions recorded
(§11 + correction): **design (D) — the explicit heap work-stack — is solved *now***, before the
M-740 `.myc` port absorbs the shape; one **global** deterministic budget (default 4096, headroom to
tens-of-thousands) + coarse-entry host-stack management as supporting infrastructure; an opt-in,
non-deterministic, corpus-excluded `--unbounded` REPL mode. Mandated method: **research → plan →
adversarial review → implement**, with secure-by-design periodic adversarial passes. Motivated by
the ADR-041 near-miss (a toolchain frame-size change turned a 256-deep guard into a `SIGABRT`).
Issues: **M-978** (direction decided → `todo`) · **M-979** (solve-now track, `in-progress`);
**M-740** now depends on the settled design. Decides nothing normatively; status **Draft**.

### Toolchain + dependency freshness: MSRV → 1.96.1, workspace deps refreshed (2026-07-03: ADR-041)

Maintainer-authorized toolchain hygiene pass. No kernel semantics change; the interpreter stays the
trusted base (ADR-007 strategy unchanged — only the pinned version moves).

- **MSRV 1.92 → 1.96.1** (**ADR-041**, Accepted 2026-07-03; amends ADR-007's pin clause only —
  append-only, charter text preserved, house rule #3). Pins moved in lockstep: `rust-toolchain.toml`
  (`channel`), `Cargo.toml` (`rust-version`), `CLAUDE.md`, `CONTRIBUTING.md`. Verified green on
  `rustc 1.96.1`: `cargo build`/`clippy -D warnings`/`fmt`/`test --workspace` — **4265 tests pass**.
- **New-toolchain lint fixes** (clippy 1.96): `unnecessary_sort_by` → `sort_by_key`
  (`mycelium-lsp`); `manual_checked_ops` scoped-`#[allow]` on two division-oracle tests
  (`mycelium-core`, `mycelium-interp`) — the plain `x / y` is the trusted oracle in a `y != 0` branch
  and must stay plain.
- **Parser deep-stack fix (G2 regression, never-silent).** `parse`/`parse_phylum` now run on the
  managed deep stack (`mycelium_stack::with_deep_stack`, as `eval`/`ambient` already did), so the
  explicit `MAX_EXPR_DEPTH=256` budget — not the host stack — is the binding nesting limit,
  independent of per-toolchain frame sizes. Witness: 1.96.1's larger parser frames overflowed the
  2 MB test stack at the guard boundary on the `type_args` path, turning an explicit refusal back into
  a SIGABRT; the four DN-40 deep-nesting guard tests are green again (A4-02 / DN-40).
- **Dependency refresh** (latest semver-compatible via `cargo update`; two pre-1.0 tooling bumps in
  `xtask` verified non-breaking): `cargo_metadata` 0.18 → 0.23, `toml` 0.8 → 1.x (+ transitive
  `thiserror` 2, `winnow` 1). Shipped/kernel crates untouched beyond the lockfile.
- **Security (separate PR):** the VS Code extension's `@vscode/vsce` 2.32 → 3.9.2 clears Dependabot
  #1/#2 (`markdown-it`/`linkify-it` quadratic-complexity advisories); `npm audit` clean.

### Human-readable `.myc` formatting + `Vec` list literal (2026-07-03: M-976 · M-977)

Post-freeze presentation + surface-ergonomics work — the kernel is untouched (both are tooling /
frontend lowerings). All behavior-neutrality is `Empirical` (C1/C2 + AST-identity + the differential
suites), never `Proven`.

- **Shape-Dispatched Readable `mycfmt`** (M-976, `crates/mycelium-fmt`, DN-82 §7). A whitespace-only
  Readable style that kills the deepening `Cons` pyramid and the mirror-image closing-paren wall:
  R1 flat-spine for right-nested same-head chains, R2 rustfmt-block for wide-flat calls, R3 one indent
  per real nesting for genuine trees, R4/R4c binding layout. A **house-style knob** — `LayoutCfg`
  with `SpineInner::{InlineWhenFits (default), AlwaysExpand}` and `mycfmt --readable --expand-spine`
  (compact vs expanded, both behavior-neutral). **Default width retuned 88 to 100** (rustfmt's
  `max_width`, the value the Mycelium Rust kernel itself uses — grounded, not Black's Python 88).
  `lib/std` re-rendered.
- **`Vec` list literal** (M-977, **RFC-0040**, `crates/mycelium-l1`). Type-directed elaboration: a
  `[e1, …, en]` literal against a cons-list-shaped `Vec[T]` desugars to the `Cons` chain (and is
  re-checked as it); `Seq{T, N}` and non-list ADTs are untouched/refused. **No grammar/parser/L0
  change** — a frontend lowering onto the frozen kernel (freeze-safe, DN-56 §6), behavior-neutral by
  AST identity. `lib/std`'s static tables (`matrix()` etc.) now read as each-item-closed `[…]` with a
  single terminal `;` — no closer run, no pyramid. Resolves DN-82 FLAG-976-1; the variadic
  `all_of`/`concat` fold (FLAG-976-2) stays a deferred future RFC.

### Kernel freeze declared (2026-07-02: M-969 — the closing act of Phase-I)

**The Mycelium kernel is declared frozen** (`core 1.0.0`-class). This is the deliberate closing act
of Phase-I: a `Declared` decision resting on `Empirical` evidence (VR-5 — *not* a claim of a
theorem-proven-complete kernel), gated on all five DN-56 §5 conditions being checked green.

- **The gate (DN-56 §5, all five green):** census / never-silent floor (W5) · reject-ledger (DN-80 +
  the M-959 regression guard, 9/9) · primitive set closed (Π = 38 prims; ADR-033 FLAG-1 dispositioned
  IN via DN-74; `vsa.*` + Gap-E landed) · lowering surface closed (RFC-0037 Enacted; the DN-54
  `lower`/`derive` extension surface checked; DN-71/DN-73/DN-74 resolved; the DN-54 §10 attachment
  model enacted, Model A / M-973; grammar baseline M-924) · KC-3 completeness review **passed** (run
  via the DN-39 machinery, 2026-07-02).
- **Independently scored:** the four previously-open conditions were re-verified against `integration`
  by an independent assessment (guarding against completion bias — house rule #4) and recorded in
  **DN-76 §5A: 4 of 4 green**. DN-56 advanced `Accepted → Enacted` (append-only, stepping through
  Accepted — house rule #3).
- **Post-freeze diff policy:** the frozen kernel (the `mycelium-core` trusted base + the L1 ten-node
  calculus + the ratified Π) changes **only** via a **DN-39 default-DENY promotion**; any other kernel
  change is a `core 2.0.0` event. Every future language feature is a frontend lowering over the frozen
  kernel — a black box is *unexpressible* by construction.
- **What it does not claim (VR-5):** not a proof of bug-freedom or census-completeness — the
  census/KC-3 verdicts are `Empirical` (no gap *found*, not *proven* absent). It is a checked,
  auditable declaration that the kernel is a stable fixed base a public release can stand on.

Basis: DN-56 §9 + Changelog; DN-76 §5A. Held for the maintainer, unchanged by the freeze: the public
flip, tag cuts, and the DN-83 stability-window decision.

### Added / Changed / Fixed (2026-07-02: `grm`/`frz` — lowering-surface close-out, mycfmt readable, transparency fixes)

The `grm` (grammar/lowering) and `frz` (kernel-freeze) lanes' Phase-I H2 kernel work. This closes
the freeze's "lowering surface" condition; the kernel-freeze declaration (M-969) remains the
strictly-last maintainer/orchestrator act. All guarantee claims stay at their checked strength
(VR-5). Basis: PRs #1038, #1040, #1042/#1045 (reject-ledger reconciles), #1043, #1044, #1046, #1047.

- **`mycfmt --readable` human-multiline style** (`crates/mycelium-fmt`, M-974/#1038). A new
  `Style::{Compact, Readable}` + `format_source_readable`/`format_source_styled` render large
  segments across lines (breaks after commas) at an 88-col target. **Presentation-only and proven
  behavior-neutral** — 14/17 `lib/std` nodules reformatted with green `include_str!` round-trips.
  **DN-82** scopes the readable canonical to `lib/std` only (not a global flip). Guarantee: the
  behavior-neutrality is **Empirical** (differential + three-way).
- **`Fuse` prelude and semilattice-law checker** (`crates/mycelium-l1/src/fuse.rs`, M-965/#1040).
  A built-in `Fuse` trait plus a definition-time checker that refuses a `join` violating
  idempotence / commutativity / associativity over a finite enumerable domain, with a concrete
  counterexample (never-silent, G2). **Empirical** (exhaustive over the domain); a non-enumerable
  domain is *skipped*, never silently assumed lawful (VR-5).
- **`via` delegation — deterministic, `EXPLAIN`-able ordering** (`crates/mycelium-l1`, M-966/#1044).
  Two `via` clauses claiming one trait are refused never-silently naming both candidate field
  indices; `Env::via_provenance` records the chosen delegate. Also fixed a latent bug: parametric
  `via` delegation never type-checked (the abstract method signature was not argument-substituted).
- **Per-instantiation guarantee-tags through monomorphization** (`crates/mycelium-l1/src/mono.rs`,
  M-967/#1046, executes M-844). **Fixes a silent-loss VR-5 bug:** mono re-emitted specialized
  signatures via `ty_to_ref` → `TypeRef::unguaranteed`, silently dropping each source `@ g` tag on
  every monomorphized param/return/`Let`/`Ascribe`. Now every reconstruction site threads the
  original declaration's guarantee — no tag lost, merged, or upgraded across instantiation.
- **LSP semantic-token classification completed** (`crates/mycelium-lsp`, M-975/#1043). `classify()`
  is now **exhaustive over `Tok`** (a future unclassified token fails to compile, not silently
  drops); string/float/bytes literals and the `Seq`/`Bytes`/`Float` + M-915 short repr keywords
  classify correctly.
- **DN-54 §10 attachment model — Accepted (Model A) and enacted** (DN-81, M-973). Sibling-item
  injection, wired through the M-919 affine tracker (`derive_site_double_consume` red-then-green).
- **Grammar stability close-out** (M-924/#1047). The ebnf gains the first-class function-type
  `A => B` production (matching `parse_type_ref_guarded`) with a positive conformance fixture;
  grammar artifacts re-verified in sync (44/44 zero-ERROR parse). **DN-83** *proposes* an
  RFC/ADR-gated surface-grammar stability window (status Proposed — maintainer decision pending).
- **Reject-ledger kept exhaustive** (DN-80, M-959 guard). Re-audited through the wave — parse
  corpus 30 fixtures, check-level 217 sites across 41 families (fixture 31, family-8 lower/derive,
  family-40 `Fuse` law, family-6 `via`) — and the regression guard **caught a real compile break**
  (an M-965×M-973 semantically-conflicting merge that left `Env{}` missing a field) plus every
  unledgered reject. All `Empirical` (mechanical inventory).

> Also landed ad-hoc during the wave (traceable via the DN provenance above; formal `issues.yaml`
> registration is a lightweight follow-up): M-972/M-972b (DN-81 dossier + DN-81 correction),
> M-973 (attachment enact), M-974 (mycfmt readable), M-975 (LSP classify).

### Added (2026-07-02: M-697 — language identity and full-surface syntax highlighting for `.myc`)

Brought the editor grammars current with the landed corpus and packaged Mycelium for
identification and highlighting across editors and forges. The keyword set stays **lexer-derived**
(the `just drift-check` gate is unchanged); the grammars grew from a reserved-word scaffold to the
full landed surface. All outward-facing publishing is **staged, not fired** — the artifacts plus a
ready-to-file runbook are in-repo; the maintainer fires the external submissions.
Basis: PRs #1034, #1035, #1036, #1037, #1039.

- **Editor grammars v2** (`tools/grammar/`, PR #1034). `generate.py` now emits full-surface
  tmLanguage + a **structural** tree-sitter grammar covering strings and the minimal escape set
  (M-910), floats (ADR-040/M-897), `0b`/`0t`/`0x` literals, the RFC-0025/M-745 operator set, the
  M-915 short repr aliases (`bin`/`tern`/`emb`/`hvec`), ambient reprs, tuples, generics `[…]`,
  guarantee annotations `T @ Strength`, function types `A => B`, effects `!{…}`, and every landed
  declaration/expression form. The retired `<+0->` compact-ternary pattern is removed (RFC-0037 D4);
  the retired `->` renders `invalid.deprecated`. Bucket correction: `Float` and the short aliases
  bucket as `type`. Guarantee: keyword sets mechanical; structural productions **Empirical** —
  verified by parsing the full conformance accept corpus (25) plus `lib/std` (18) with zero ERROR
  nodes, not proven equivalent to the EBNF (which stays the accept/reject oracle); two Declared
  permissive deviations documented in-file.
- **VS Code / Cursor extension** (`editors/vscode/`, PR #1035) — `tzervas.mycelium-language`, language
  id `mycelium`, `scopeName: source.mycelium`, extension `.myc`; committed `.vsix`, a
  `language-configuration.json`, and `vscode-tmgrammar-test` scope tests (2/2 green). Packaging is
  verified; live in-editor rendering is Empirical-not-UI-tested (no GUI editor in the build env).
- **Publishable tree-sitter package** (`tools/grammar/tree-sitter-mycelium/`, PR #1036) — committed
  generated `src/`, a `tree-sitter.json`, and `test/corpus/` (12/12 `tree-sitter test` green;
  42/42 parse sweep). This is the asset Linguist and Neovim/Zed/Helix/Emacs consume.
- **Distribution runbook, Rouge lexer, and `.gitattributes`** (PR #1039). `tools/grammar/DISTRIBUTION.md`
  is the ready-to-file runbook (Open VSX as the chosen Azure-free registry; the MS Marketplace as
  optional; the `github-linguist/linguist` `languages.yml` entry with the collision check — `.myc`
  is **FREE**, verified 2026-07-02; per-editor tree-sitter setup). A tested Rouge lexer draft
  (`tools/grammar/rouge/`, exercised against `rouge` 5.0.0 over 48 real `.myc` files) stages the
  GitLab path. The root `.gitattributes` classifies generated/vendored/binary artifacts and,
  per the maintainer's honesty constraint, **does not** map `.myc` to any existing language —
  "Mycelium" in the GitHub bar comes only from the gated Linguist submission.
- **Drift-gate the downstream copy** (PR #1037 + the integration wiring). `generate.py` emits the
  extension's `syntaxes/` tmLanguage as a drift-checked downstream copy (a missing/stale copy fails
  the gate — G2); `.codespellrc` allowlists `rouge`/`notin`; `generate.py` gains its exec bit.

### Fixed (2026-07-02: M-971 — DN-68 acyclic-deps regression, 12 to 0 violations; dev to integration close-out)

The Phase-I H1 wave (below) regressed the DN-68 acyclic-deps invariant to 12 violations; this fix
(PR #1015) resolves all of them by structural extraction, mirroring the M-881/882 fixture-refactor
and M-883/884 rt-abi/sched seam precedents, with no strata/tier whitelist.

- **New `mycelium-std-conformance` crate** (tier `std`, stratum 0). Relocated the 11 oracle-backed
  `lib/std/*.myc` port-differential tests and their shared `tests/harness/` out of the core-tier
  `mycelium-l1`, dropping 9 `mycelium-std-*` dev-deps from `mycelium-l1` and dissolving the
  `{l1, proj, spore, std-spore, std-testing}` dev-cycle that ran through the removed edges. The
  relocated tests still run from their new home unchanged.
- **New `mycelium-vsa-decode` crate** (RFC-0010 decode-methodology selection seam). Extracted
  `decode_select` and `reconstruct_factors_selected` up out of `mycelium-vsa` (the only VSA code
  depending on `mycelium-select`), breaking the `{interp, select, vsa}` dev-cycle and the
  `interp to vsa` upward-stratum violation structurally: `mycelium-vsa` now depends only on
  `mycelium-core` (stratum re-derived 2 to 1). No external crate consumed the moved surface;
  consumers (`cert`/`mlir`/`std-vsa`/`std-spore`) are untouched.
- `xtask/deps-strata.toml` updated: both new crates registered in `[strata]`/`[tiers]`, the
  `mycelium-vsa` re-derivation recorded, `[meta].derived_from` updated.
- Verified: `cargo run -p xtask -- deps` reports **0 violations** (was 12); full workspace test,
  fmt, clippy, and build all clean; `api` gate green (regenerated `mycelium-vsa.txt` plus baselines
  for the two new crates).
- **Integration close-out** — `docs/api-index/` regenerated to cover the two new crates and the
  `mycelium-vsa` surface shrink; `docs/Doc-Index.md` and `tools/github/issues.yaml` (M-971 to
  `done`) updated per the concurrent-dev pattern (leaves FLAG close-out items, the integrating
  parent applies them once). Basis: PR #1015.

### Added (2026-07-02: Phase-I H1 wave — enb enablers, opp ports, grm/frz dossiers; integration close-out)

The Phase-I H1 wave landed the below-grammar functional-usability enablers ADR-038 §2.6 named, a
first tranche of self-hosted stdlib ports, and the design dossiers that disposition the remaining
kernel-freeze questions. This entry is the `dev → integration` whole-batch close-out (api-index +
grammar regenerated, statuses transitioned append-only, issues closed).

- **Integer prim surface completed** (kickoff `enb`, E28-1, Gap B — RFC-0033 §4.1.2/§4.1.3). New
  never-silent two's-complement prims in `crates/mycelium-interp/src/prims.rs`: `bin.mul` (M-887,
  overflow → explicit error, no wrap-by-default), `bin.div`/`bin.rem` (M-888, explicit div-by-zero
  error), `bin.shl`/`bin.shr` (M-889, explicit out-of-range shift-amount), plus the signed op set
  M-766 (neg and overflow-detect) and M-767 (signed div/rem/shift variants). Property tests on
  every bound; conformance accept and reject.
- **Dense and VSA prims** (E28-1, Gap C/D). Dense elementwise (M-890) and dot/similarity (M-891)
  over `crates/mycelium-dense`; VSA bind (M-892), certified bundle (M-893), and cleanup/reconstruct/
  required_dim (M-894) over `crates/mycelium-vsa`, each surfaced through the interpreter with
  three-way differential and conformance green.
- **Scalar-float value form landed and ADR-040 Enacted** (E28-1, Gap A). Route-(ii) `Repr::Float`
  binary64 (M-896), float literal lex/parse (M-897), IEEE arithmetic (M-898), comparison (M-899),
  and the certified-mode gate (M-900). Round-to-nearest-even only, canonical quiet-NaN, bit-distinct
  signed zeros, in-band specials with never-silent conversion boundaries. **ADR-040** stepped
  Accepted → **Enacted**; companion promotion dossier **DN-69** (PROMOTE — the first candidate to
  clear the DN-39 four-clause bar).
- **`Substrate`/`consume` affine construct executes at the L1-eval level** (E28-1, Gap E; DN-71
  Model S). Substrate v0 opaque affine handle (M-902), static use-once affine tracker with a
  never-silent runtime backstop (M-903), and identity-move `consume` lowering with a v0 drop posture
  (M-904) — all in `crates/mycelium-l1/src/eval.rs`, no new L0 node. Cross-checked against the `grm`
  DN-54 dossier: same model, not forked.
- **R2-lite runtime surface (D-lite D1)** (E28-1; DN-70). `forage` activated as the `@forage(policy)`
  hypha placement annotation with a mandatory-EXPLAIN placement trail (M-906); `backbone` verified as
  a landed decision, not an executing construct (M-907, DN-70 §4/FLAG-D3). Mesh/xloc/cyst long-lead
  research track started (M-913, **Research Record 28**).
- **`myc run` and surface literals** (E28-1). `myc run` single- and multi-nodule execution with
  manifest-driven linking (M-908/M-909, `crates/mycelium-cli`); string literals (M-910, grammar +
  lexer/parser) and a `mycelium-fmt` string-literal fix (M-911); `hash.blake3` and `bytes.eq` prims
  (M-912). H1 capstone demo and readiness re-verify (M-914).
- **Nine self-hosted stdlib nodule ports** (kickoff `opp`, E29-1). A differential port harness
  (M-925) plus `std.core`/`diag`/`error`/`recover`/`select`/`swaps`/`ternary`/`testing`/`spores`
  ported to `lib/std/*.myc` (M-926…M-934), Rust-ref ≡ `.myc` differential green, and added to the
  `[surface].exports` freeze list. Measured transpiler-assist % per nodule recorded in the
  self-hosting port ledger (M-935); the D1-kernel-boundary halves (re-export and `hash`-mint) stay
  Rust per KC-3.

### Changed (2026-07-02: Phase-I H1 wave — decision dispositions and status transitions)

- **Design dossiers authored and accepted** (kickoffs `grm`/`frz`), all under the maintainer's
  2026-07-02 delegation of these decisions to the wave orchestrator (`Declared` as relayed), recorded
  append-only at the integration-reconcile promotion gate: **DN-73** tuple-type ratification (M-920 →
  Accepted, Option A), **DN-74** ADR-033 FLAG-1 `FieldSpec::Fn` soundness (M-922 → Accepted, Option A
  — dispositions the soundness question without stepping ADR-033), **DN-75** DN-54 completion audit
  (M-917 → Resolved, audit stands), **DN-76** kernel-freeze four-condition scorecard (M-958 →
  Accepted as the M-969 gate instrument; the kernel is **not** frozen — 0/4 green, M-969 stays gated),
  **DN-77** inject-mode build-scope (M-960 → Accepted, Option B), **DN-78** the M-828 R2 remainder
  buildable-vs-research split and memory-model confirmation (M-962…M-964 → Accepted), and **DN-79**
  `when`-guard clause semantics and guarantee propagation (M-968 → Accepted, impl held).
- **Inject-mode Phase-I subset built** (frz, M-961; DN-77 §4). The confirmed buildable slice of
  RFC-0038 landed Rust-first (`crates/mycelium-mlir/src/{inject_gate,inject_cert,inject}.rs`); the
  matching RFC-0038 claims (§4.2/§5.1/§6.2/§7.1/§7.3/§8.4/§8.6/§8.5) flipped `Declared → Enacted` for
  exactly that slice, with everything else (all §9 R&D, the `module`/`call` grains) held `Declared`.
  RFC-0038 as a whole stays **Accepted** (its §13 Implementation DoD is not fully met).
- **Integer-prim signedness naming convention** (DN-72). Integer-prim surface names carry an explicit
  `_u`/`_s` signedness suffix (never-silent about signedness); **DN-72** Accepted and enacted in the
  same change.
- **RFC-0033 progress note** — the Gap-A/Gap-B enablers above (binary prims, signed ops, float value
  form) landed; the design and the post-1.0 V1–V5 deferral are unchanged, and no content-address
  identity is spent (single-rehash-deferred-to-first-value-persistence stands, §7).
- **Integration close-out** — `docs/api-index/` and the derived grammar artifacts regenerated;
  `docs/Doc-Index.md` + `docs/adr/README.md` register ADR-040 and DN-69…DN-79 and Research Record 28;
  every landed M-id flipped to `done` with a `landed_basis`; the held `grm`/`frz` L1-impl issues
  (M-915/916/919/921/923/924, M-959/965/966/967/969) left blocked/todo — never silently closed.

### Added (2026-07-02: kickoff `acy` — Phase-I H0 acyclic-deps hardening, integration close-out)

- **Structural acyclic-deps gate landed and wired into `just check`** (M-877…M-880). A new
  `xtask deps` subcommand (`xtask/src/deps/`) analyzes `cargo metadata --format-version 1` over the
  full workspace and enforces, per edge (normal, dev, *and* build — cargo itself never rejects a
  dev-dep cycle): (a) every normal edge respects the frozen per-crate `[strata]` ordering
  (`xtask/deps-strata.toml`, `Empirical`); (b) the combined normal+dev+build graph is acyclic
  (Tarjan SCC, `Exact`); and (c) two named cross-boundary rules — `no-interp-std-dep`
  (`mycelium-interp` may never depend on any `mycelium-std-*` crate, in any dependency kind — the
  KC-3 trusted-base boundary) and `no-upward-tier-edges` (no crate may depend on a crate in a
  strictly higher `core < std < tools` architectural tier). Every violation prints the offending
  edge, its dependency kind, and the rule's citation (never a bare pass/fail exit code — G2). Wired
  into `just check` with a graceful, never-silent skip when the tool is absent
  (`scripts/checks/deps-acyclic.sh`).
- **All three known dev-dep cycles broken** (M-881, M-882): `mycelium-select →[dev] mycelium-cert →
  mycelium-vsa → mycelium-select` (corrected from the kickoff doc's `mycelium-std-select` typo — the
  actual crate is `mycelium-select`) and the two `mycelium-cert →[dev] {mycelium-proj,mycelium-spore}
  → mycelium-l1 → mycelium-cert` cycles are gone; each crate's dev-only cross-crate imports were
  replaced by local, fixture-driven tests with the same assertions and guarantee-tag strength
  preserved (VR-5). All three cycles were a single 7-crate strongly-connected component; the gate now
  reports 0 violations at HEAD.
- **The `mlir → std-runtime` upward-tier anomaly extracted, not loosened** (M-883, M-884). A new
  crate, **`mycelium-rt-abi`** (tier `core`, confirmed name — see DN-68), holds the reclamation and
  supervision surface `mycelium-mlir` actually needs; `mycelium-mlir` now depends on
  `mycelium-rt-abi` instead of `mycelium-std-runtime`, and `mycelium-std-runtime` re-exports the
  same modules at their original paths (no consumer-visible API break). Same shape as the PR #864
  `mycelium-sched` precedent: extraction over rule-loosening.
- **`mono.rs` recursion-safety bug fixed** (M-866, a real M-674 follow-up, done early and
  independently of the acyclic-deps sequencing). `free_vars`/`pattern_binders` in
  `crates/mycelium-l1/src/mono.rs` now carry an explicit `MAX_WALK_DEPTH`-style depth budget
  matching `totality.rs`'s discipline, returning `ElabError::DepthExceeded` instead of a silent
  host-stack overflow on a pathologically-nested specialized body (G2); a just-past-budget
  regression test asserts the explicit error.
- **`publish = false` sweep verified complete** (M-886) — all 54/54 workspace members resolve
  `publish = false`, versions stay `0.0.0` per ADR-038 §2.2; this was already satisfied at kickoff
  start and is recorded here as a verified, not newly-applied, fact.
- **DN-68 — The Acyclic-Deps Invariant authored** (M-885, `Draft`): the strata/tiers data model,
  the two named rules, where the gate lives, and the change-procedure (a stratum/tier reassignment is
  its own reviewed PR, never folded into the PR that needed the exception). Indexed in
  `docs/Doc-Index.md`.
- **DN-66 §6 currency note appended** — the `mlir → std-runtime` load-bearing basis §4.c cited is
  void post-extraction; §4.c's original text is unchanged (append-only, house rule #3).
- `docs/api-index/` regenerated to cover the new `mycelium-rt-abi` crate and the relocated
  reclamation/supervision modules.
- Basis: PRs #935 and #936 (kickoff `acy`, Phase-I H0); `cargo run -p xtask -- deps` reports 0
  violations at HEAD.

### Changed (2026-07-01: ADR-038 ratified — Accepted; FLAG-V1/V2 resolved)

- **ADR-038 ratified by the maintainer ("ratify 38") — status `Proposed → Accepted`.** The
  function-first release strategy now **binds**: the public release is gated on **functional
  usability** and is **version-independent** — it happens at a sub-`1.0.0` (**`0.x`**) semver on
  reaching functional usability, well before `1.0.0`; **`1.0.0` = fully dogfooded / self-hosted /
  rewritten into Mycelium *where appropriate* + 100% operational**.
  - **FLAG-V1 resolved** — the `lang 1.0.0` label collision with ADR-022's functional-completeness
    milestone dissolves: since the public release is a sub-`1.0.0` (`0.x`) semver, it is never
    labeled `1.0.0`, so no ADR-022 relabel is needed.
  - **FLAG-V2 resolved** — `1.0.0` requires the project to be fully dogfooded/self-hosted/rewritten
    into Mycelium *where appropriate*; compiler self-hosting rides §2.3's demonstrably-better
    stability/performance condition (part of "where appropriate," not a hard gate).
  - The **ADR-036 §2.4 refinement** (release gate: functional usability, version-independent, not
    Rust-replacement) and the **RFC-0031 §5 D1 supersession** (compiler-forever-Rust permanence
    lifted; D1's boundary itself remains operative through Phase I) are now **in force**.
  - Cross-references synced corpus-wide: `docs/adr/ADR-036-Dogfooding-and-Public-Release-Strategy.md`
    §2.4 + changelog row, `docs/rfcs/RFC-0031-Self-Hosted-Standard-Library-Composition.md` §5 D1 scope
    note + changelog row, `docs/adr/README.md` (status paragraph + ADR-036/ADR-038 table rows),
    `docs/Doc-Index.md` (ADR-038 row), and the Phase-I/II kickoffs (`.claude/kickoffs/{flp,rwr,enb,
    grm,opp,acy,rcp,README}.md`) + `docs/CURRENT-STATE.md` — every "Proposed / binds-on-ratification /
    FLAG open" reference updated to Accepted / in-force / resolved.

### Changed (2026-07-01: ADR-038 refinement — versioning axis + execution doctrine, still Proposed)

- **ADR-038 refined (same session, pre-ratification; held `Proposed`).** Two maintainer-directed
  refinements folded into the still-Proposed strategy ADR (with an append-only changelog row in the
  ADR — the authoring trail recorded, not silently overwritten):
  - **Public release decoupled from the version number (§2.8, new).** The public flip is gated on
    **functional usability alone** and happens at **whatever semantic version fits — a `0.x`, well
    before `1.0.0`** ("v1 as a publicity gate is arbitrary"). The **public semver tracks the
    Mycelium-rewrite progress**, climbing `0.x → 1.0.0` in the open, with **`1.0.0` ≡ "fully
    rewritten into Mycelium (where appropriate) and 100% operational"** (Phase II's terminal).
    **For now the version stays `0.0.0`; the concrete semver scheme is deferred until actually ready
    to publish.** Two ambiguities FLAGged for the maintainer, not guessed: **FLAG-V1** (the
    `lang 1.0.0` label collision with ADR-022's functional-completeness milestone) and **FLAG-V2**
    (whether `1.0.0` requires compiler self-hosting).
  - **Execution doctrine (§2.7) refined.** **Fable-class models are reserved *solely* for planning
    and complex design (they do not implement); implementation and all lighter work run on
    Opus/Sonnet/Haiku scoped to the intensity and complexity of the task.** (Recorded as strategy;
    the CLAUDE.md swarm-mode-table wording update is a small follow-up FLAGged for the maintainer.)
  - Propagated the append-only pointer on **ADR-036 §2.4** (release gate now also version-independent)
    and revised the umbrella roadmap `road-to-1.0.0-and-mycelium-rewrite.md` (phase map, exit
    criteria, §7, §8 FLAG table — added FLAG-V1/V2 + the deferred semver-scheme row) and the ADR-038
    rows in `docs/adr/README.md` + `docs/Doc-Index.md`. ADR-038 stays **Proposed** at the maintainer's
    instruction ("once adapted I'll say it's ratify-ready").

### Added (2026-07-01: ADR-038 Proposed — function-first release strategy + the road-to-1.0.0 umbrella roadmap)

- **ADR-038 — Pragmatic Dogfooding: the Function-First Release Strategy** (**Proposed**, awaiting
  maintainer ratification — authored from the maintainer's 2026-07-01 session directives, not
  self-ratified). Records the North Star **"Rust where appropriate, Mycelium everywhere else"**
  (pragmatic dogfooding, not zero-Rust dogmatism); **Phase I → `lang 1.0.0` and the public release
  gated on functional usability** (repo private, crates `0.0.0`, `publish = false` until the flip);
  **Phase II → post-public progressive Mycelium rewrite** with compiler self-hosting a deferred,
  doubly-conditional aspiration (only if stability/perf-proven; only after transpiler polish); the
  transpiler doctrine (progressive hardening, pre-port polish, manifest transcoding only where
  ROI-positive — accelerant, never gate); float route (ii) (scalar-float `Repr` via a future float
  ADR and a DN-39 promotion review; single rehash coordinated with the deferred ADR-030/031 doors,
  deferred to first value-persistence); and the planning-tier/implementation-tier execution doctrine
  with mandatory PM prep (user stories and DoD before any implementation agent).
- **Umbrella roadmap revised:** `docs/planning/rust-reference-completion-and-acyclic-deps.md` →
  **`docs/planning/road-to-1.0.0-and-mycelium-rewrite.md`** (git mv; pointer stub left at the old
  path; `.claude/kickoffs/rcp.md` and `docs/CURRENT-STATE.md` references updated). Re-sequenced
  function-first per ADR-038: **H0** acyclic-deps enforcement plus workspace publish-hygiene and
  M-866; **H1** the below-grammar usability enablers (order B→C→A(route-ii)→E→D-lite, plus
  `myc run`/string-literal/`hash.*` — from the readiness-§0 verification); **H2** the Rust-reference
  closeout lanes (l1 semantics, value/AOT tail, runtime maturity, toolchain/UX, inject-mode, kernel
  freeze); **H2a** the grammar-stability gate before mass porting (RFC-0037 follow-ons, DN-54
  completion, tuple decision, ADR-033 FLAG-1); opportunistic `.myc` ports non-gating; **Phase II**
  cleanly separated. Nothing from the prior plan dropped (mapping in its meta-changelog).
- **Append-only notes (bodies preserved, changelog rows added):** **ADR-036** — §2.4's release gate
  refined by ADR-038 to functional usability (binding upon ratification; §2.1–§2.3 unchanged);
  **RFC-0031** — §5 D1's "compiler stays Rust forever" permanence superseded (ADR-036 §2.2's
  toolchain-wide dogfooding scope, made explicit by ADR-038; D1 boundary stays operative through
  Phase I); **RFC-0033** — M-766/M-767 plus the float-`Repr` work pulled forward from the deferred
  V-wave into Phase I, §7 single-rehash dogfood-gate discipline honored unchanged.
- Indexed in `docs/adr/README.md` (also adding the previously-missing **ADR-037** index row —
  index-coverage gap closed) and `docs/Doc-Index.md`. No `issues.yaml` entries minted (task minting
  happens at execution kickoff per the roadmap).

### Added (2026-07-01: M-873 follow-on — transpiler hardening: width_cast emission, batch mode, 8-twin union backlog)

- **Faithful `width_cast` conversion emission (DN-41).** `mycelium-transpile` now emits unsigned
  `Binary` widening `impl` bodies as the **real** `width_cast(self, <Binary{M} witness>)` prim (witness
  = a synthesized all-zero `BinLit` of `M` bits; grammar/RFC-0020-confirmed width-from-content; DN-41 §3
  makes the witness bits unused). Raised **std-cmp 3.6%→12.6%** (10 conversion `impl`s became genuine
  emissions). Honestly still gapped: **signed** widening (ADR-028 sign-free `Binary` — a real semantic
  gap), `bool`-`Self` widening (no witness), and all **narrowing** (DN-41 fallible/`Result`, no single-
  `= expr` form). The principle: emit a body **iff** it maps to a *confirmed real* surface, else gap it.
- **Directory/batch CLI mode** — `mycelium-transpile <crate-src-dir> <out>` transpiles a whole crate's
  `src/` (skips tests), emitting per-file `.myc`/`.gap.json` + combined `summary.json`/`union.gap.json`.
- **Union surface-feature backlog across 6 core-lib crates** (`fixtures/UNION-BACKLOG.md`,
  `union-backlog.json`): grand union **43/346 ≈ 12.4%** expressible (`Empirical`). Re-ranked demand data
  — **unsupported *types* #1 (36%: `String`/`text`, `usize`/`isize`, `char`, closures, and signed ints —
  an ADR-028 sign-free consequence)**, macros #2 (22%), trait-bounded generics #3 (12%). Recorded in
  DN-34 §8.5.
- **Grounded self-hosting finding (DN-34 §8.6):** `std.option`/`std.result` have **no Rust source**
  (authored directly in Mycelium — M-715/M-649); excluded from the corpus, never substituted (VR-5/G2).
- **Honest artifact parity fix:** regenerated the single-file `std-cmp` fixtures (they were stale after
  `width_cast` landed — now 14 emitted / 20 `width_cast` lines, matching the batch output + the code).
- 16/16 tests green (fmt/clippy clean). **Flagged for integration:** the `cargo-public-api` baseline
  still can't be generated in-env (tool absent) — deferred, not fabricated.

### Added (2026-07-01: M-873 — Rust→Mycelium transpiler PoC + prioritized surface-feature backlog)

- **`crates/mycelium-transpile` (new, PoC — kickoff `trx`, DN-34 §8).** A `syn`-based Rust→Mycelium
  transpiler spike: it reads one Rust crate's AST and emits (a) a best-effort `.myc` for the
  expressible fraction and (b) a **never-silent, structured gap report**
  (`{file, line, rust_construct, reason, category}` JSON) for everything it cannot faithfully express.
  Built on an **exhaustive dispatch** whose fallback arm always records a gap — *not* an allowlist
  (the seed `py2rust` analyzer was an allowlist with a silent pass-through; DN-34 §8.1 corrects the
  seed posture with measured specifics). New deps (`syn`/`quote`/`serde`) are scoped to this crate
  only (KC-3, not the kernel). 7/7 tests green (`fmt`/`clippy -D warnings` clean); fixtures
  (`fixtures/std-cmp.{myc,gap.json}`) checked in as evidence.
- **First `Empirical` transpiler data (converts DN-34 §6-Q6 + assessment §5a from `Declared`).** Run
  on `mycelium-std-cmp` and diffed against `lib/std/cmp.myc`: **4 of 111 non-test top-level items
  expressible ≈ 3.6%** against the current surface *without* macro expansion (a lower bound); the
  dominant blocker is **macro-generated code (~55% of gaps)**, so the highest-leverage next step is
  transpiler-side **macro expansion**. Measured PoC cost **~0.85–0.95M tokens** — at/below the low end
  of the `Declared` "first spike ~1–3M" estimate.
- **Prioritized surface-feature backlog (first-class output, DN-34 §8.3).** The union of gaps, ranked
  by measured demand on `std-cmp` — macros → conversion/`as`-cast op bodies → trait definitions →
  trait-bounded generics → payload-carrying enum variants → derive attrs → named-field structs — as
  the real, demand-grounded input to E18-1's `needs-design` work.
- **Transparency (G2/VR-5).** The emitted `.myc` is tagged `Declared`/unvalidated (no Mycelium
  parser/checker confirms it); the diff extraction is a `Declared` heuristic. A review pass
  reclassified 12 numeric-widening `impl` blocks that had a fabricated `from(self)` body from
  *emitted* to *gapped* — the emitter now flags any body it cannot faithfully lower rather than
  inventing one (DN-34 §8.2). Assessment doc + self-hosting port ledger updated with the measured
  rate; DN-34 stays **Draft** (a spike, not the gated full phase).

### Added (2026-07-01: M-872 — remote registry name@version immutability + dogfooding effort/usage assessment)

- **Remote spore publish now enforces `name@version` immutability (M-872).** `publish_remote` gained a
  best-effort pre-check (list-tags → pull → compare `spore_id`): republishing a **different** spore under
  an existing `name@version` is refused as `RemoteError::Conflict` (exit 6), an identical re-publish is
  idempotent, and a first publish proceeds. Parity with the local store's `Conflict` semantics
  (ADR-003/M-732). Grounded, never-silent `oras` error classification (verified against `registry:2`
  **and** GHCR): a missing repo/tag (`name unknown`/`not found`) maps to `NotFound` so a first publish
  proceeds, while an auth failure stays `Transport` — a missing credential is never read as "nothing
  published" (G2). **Honest ceiling (Declared, VR-5):** OCI tags are server-side mutable, so this is a
  *client-side* guard, not a proven server invariant. 59 spore tests (2 new) + live-verified.
- **`docs/planning/dogfooding-effort-and-usage-assessment.md`** — a `Declared` forecast sizing the
  comprehensive-dogfooding track (replace all Rust with Mycelium): footprint (51 crates, 126.5k non-test
  LOC, 287 modules), a productivity baseline from a measured agent sample, a tiered per-crate token model
  (~45M-token floor for the LOC port; realistic all-in ~70–120M once the language-capability build +
  differential validation are included), and cheapest-capability-first sequencing. States plainly what it
  cannot measure (the weekly usage meter). Linked from the self-hosting port ledger.

### Added (2026-07-01: ADR-037 / M-871 — remote spore registry: GHCR/OCI dense-map distribution + live dogfood)

- **`crates/mycelium-spore` gains a remote/networked backend** (`mycelium_spore::remote`) siblings the
  M-732 local file store, so spores are installable without crates.io, hosted in the **GitHub Packages
  container registry (GHCR)**. Fixed by **ADR-037** (Accepted then Enacted, same day) and grounded in the
  release strategy (ADR-036): host phylum/nodule/spore in the GitHub Packages registry to prove out the
  registry design (DN-28) and implementation, no crates.io, repo private until dogfooded.
- **DN-28 dense-map over OCI (ADR-037 §2).** A published spore is one OCI 1.1 artifact
  (`artifactType application/vnd.mycelium.spore.v1`) at `ghcr.io/<owner>/<phylum>:<version>`: each source
  object becomes one OCI blob (title `<blake3-hex>.myco`), **deduped by digest** across versions; the
  dense-map DAG (`spore_id`, kind, surface, object references, dependency edges) becomes the OCI config
  blob; `name@version` becomes the OCI tag. The dense-map codec is a hand-rolled, injective,
  length-prefixed encoding with a strict never-silent parser (mirroring `content_address`) — **no new
  runtime dependency** (KC-3).
- **Fetch-and-verify on resolve (DN-28 §3; G2).** Every fetched object's bytes must BLAKE3 to its declared
  content address, and the reconstructed source set must recompute — via the single canonical
  `content_address` (never re-implemented) — to the recorded `spore_id`. A missing object, an
  extra/undescribed blob, a byte mismatch, or a `spore_id` mismatch is an explicit `Integrity` refusal;
  `resolve -o <dir>` materializes the verified tree plus the `mycelium-densemap`.
- **CLI routes by explicit `--registry` scheme (never guessed):** a bare path keeps the local store;
  `oci://<host>[/path]` or `ghcr://<owner>` selects the remote backend. `oras` is the v0 wire-transport
  driver behind the `OciTransport` trait (a pure-Rust client is append-only future work); `oras` absent is
  an explicit `ToolMissing` error, never a silent skip. Exact-version or `latest` selection; a SemVer
  range stays `Unsupported` (ADR-018 deferred), never mis-resolved.
- **Live dogfood verified (Empirical).** Round-trips green against a local `registry:2`
  (`just spore-oci-selftest`, 57 unit tests incl. proptest round-trip/injectivity/adversarial) **and the
  live GitHub Packages registry** — the example phyla `hello` and `std` published to
  `ghcr.io/tzervas/{hello,std}` and resolved back with byte-identical, hash-verified `spore_id`s
  (`just spore-ghcr-dogfood <owner>` + `scripts/dist/`). DN-28 gains an append-only forward pointer to
  ADR-037 (its status unchanged).
- **Disclosed v0 gap (never-silent, G2):** remote publish does not yet enforce `name@version` immutability
  the way the local store does (OCI tags are mutable; a best-effort client-side pre-check is tracked as
  **M-872**). Stated in ADR-037, the contract spec §10, and the `RemoteError::Conflict` doc-comment.

### Added (2026-07-01: M-870 — third-party attributions + NOTICE generation)

- **`THIRD-PARTY-LICENSES.md`** added at the repo root: every third-party Rust crate in the
  workspace dependency graph (53 `(crate, version)` entries across 51 unique crate names),
  generated via [`cargo-about`](https://github.com/EmbarkStudios/cargo-about) from `Cargo.lock`,
  with the actual license text for each of the 25 unique license-text groups (deduped by identical
  text, referenced by SPDX id — MIT ×22, Apache-2.0, BSD-2-Clause, Unicode-3.0). Config committed
  for reproducible regeneration: `about.toml` (accepted-license allow-list mirroring `deny.toml`)
  and a custom Markdown `about.hbs` template (the stock template emits HTML). Closes the
  notice-preservation gap `M-743`'s first-party MIT audit didn't cover — MIT/BSD/Apache-2.0 all
  require the license text to travel with a shipped artifact.
- **`just licenses`** (alias `just third-party-licenses`) regenerates the file;
  **`scripts/checks/licenses.sh`** is a drift gate (`scripts/checks/all.sh` component 28, part of
  `just check`/`just check-full`) that skip-gracefully passes when `cargo-about` isn't installed
  and otherwise fails on staleness or an unresolved license (never a silent gap — G2).
- **NVIDIA disclosure (`experiments/README.md`):** the optional Python `gpu` dependency-group
  (`uv sync --group gpu`, used by the M-832 `vsa_bounds` sweep) pulls NVIDIA-proprietary CUDA
  runtime packages transitively through `torch`, under NVIDIA's own EULA — not OSI-approved.
  Documented as opt-in only, experiments-only, and never part of a distributed Mycelium artifact —
  out of `THIRD-PARTY-LICENSES.md`'s Rust-only scope but disclosed all the same (never-silent, G2).

### Added (2026-07-01: M-363 follow-up — myc-doc BOOK output + a Podman/Docker docs container)

- **`crates/mycelium-doc` gains a BOOK renderer** (`mycelium_doc::book`) — the M-363 spec's output
  (b), "the full language book", closing the one named output that wasn't yet built (HTML/Typst/JSON
  landed 2026-06-17; book did not). A curated, linear, chaptered reading order over the *same*
  content-addressed doc-IR (Getting Started → Language Guide → Language Reference → Standard Library
  → Concepts → Toolchain → Contributing → Appendices), driven by a small committed manifest
  (`docs/book-manifest.json`): explicit `sources` for hand-curated order, drift-proof `globs` for the
  Standard Library/RFC/ADR/DN appendix chapters (a new file under a globbed directory appears in the
  next build, no manifest edit needed — the same discipline as `tools/docgen/code_index.py`). Each
  page carries prev/next navigation + a chapter breadcrumb; a hand-rolled `book/search-index.json` +
  vanilla-JS `search.js` gives client-side search with **no new dependency** (reuses `serde`/
  `serde_json`, already vetted — KC-3).
- **Composition, not re-authorship:** the book renders a *scoped* `DocModel` through the existing
  `emit::html::render` and re-wraps the extracted `<article>` (byte-identical `data-cid`s) in a
  book-specific shell — it does not re-derive page content. The one non-`.md` source
  (`docs/spec/grammar/mycelium.ebnf`) is synthesized as a single verbatim, unchecked `Example` node
  (the exact file bytes, never invented prose); `CONTRIBUTING.md` (outside `docs/`) rides through the
  normal ingest+resolve pipeline via a new `BuildInput::extra_md_files` field, so its cross-references
  resolve like any other corpus doc. `BuildInput::conventional`'s default is unchanged — the existing
  `myc-doc build`/`lint` commands and their output are untouched by this addition (verified: `myc-doc
  lint` still reports the same 8/8 green checks over the unchanged 289-document corpus).
- **Never-silent by construction:** a manifest chapter that resolves to zero pages, a source path
  that matches no ingested document, or a page double-booked into two chapters is a build error
  (`BookError`), never a silently-dropped chapter or a dead ToC link (G2). Verified against the real
  corpus: `myc-doc book` produces 185 pages across 11 chapters with **zero** dead ToC/prev-next links
  and zero unresolved cross-references in the rendered book pages.
- **New CLI + justfile:** `myc-doc book [--repo-root .] [--out target/doc]`; `just docs-book`
  (advisory, not part of `just check`, same posture as `docs-site`).
- **`docs/Containerfile`** (Podman-first, docker-fallback): a two-stage build — a pinned Rust 1.92
  builder (matching `rust-toolchain.toml`/ADR-007) runs `myc-doc build` + `myc-doc book` +
  `cargo doc --workspace --no-deps`, then a minimal `python:3.13-slim` stage serves the assembled
  static site via `python3 -m http.server` (Python is first-class here). A small landing page links
  Book / Corpus / Rustdoc / **Agent code index** (`docs/api-index/`) — the container serves AI agents
  and the maintainer alike. `docs/gen-rustdoc-index.py` fills the one real gap found while verifying
  this live: `cargo doc --workspace` emits no top-level index, so a small script lists exactly the
  crate directories that were actually generated (never a hardcoded, driftable list). New
  `scripts/docs-container.sh` (+ `just docs-container-build` / `docs-container-run`): prefers
  `podman`, falls back to `docker`, errors clearly if neither is installed. Verified non-vacuously on
  this box: built with `podman build`, ran with `podman run -p 8080:8000`, and `curl`-checked every
  section (`/`, `/book/index.html`, `/book/search-index.json`, `/corpus/index.html`,
  `/rustdoc/index.html`, `/api-index/INDEX.md`) returns 200 with the expected content.
- **Test layout, as-touched (CLAUDE.md M-797 discipline):** `build.rs` (extended for
  `extra_md_files`) had its inline tests extracted to `src/tests/build.rs`; the new `book.rs` starts
  clean in `src/tests/book.rs` from day one — this crate's other pre-existing inline-test modules are
  untouched (the accepted lazy-retrofit posture; not this change's scope).
### Added (2026-07-01: M-865 — harness-level parallel AOT/JIT dispatch extending M-862's pure-arg batch)

- **`mycelium-mlir::concurrent`** (`compile_and_run_concurrent`, `jit_run_concurrent`,
  `plan_concurrent`/`ConcurrentPlan`) extends M-862's interpreter-side top-level pure-argument batch
  (a pure, ≥2-argument top-level `Op` — narrowed from `Op`/`Construct` to `Op`-only, see below) to the
  **direct-LLVM AOT** and **in-process JIT** execution paths, dispatched at the **Rust harness level**
  through the *same* `mycelium_sched::scheduler::Scheduler::run_indexed` entry point M-860's
  `emit_llvm_ir_many` already uses — no new scheduler surface, no LLVM-IR-level concurrency
  primitive. Each batch argument is submitted as its own job and evaluated by the exact trusted
  sequential runner for that path (`compile_and_run_with_swap_mode` / `jit_run`); results are
  recomposed by invoking that **same** runner once more on a tiny reconstructed `prim(consts…)` node
  — so prim-application semantics is never hand-reimplemented, only *scheduled* differently.
- **Honest scope narrowing (never-silent, G2):** a `Construct`-headed batch is explicitly *out* of
  this dispatcher's scope — the direct-LLVM whole-program contract requires a top-level result to
  reduce to a representation `Lane` (`lower_program_with_swap_mode`'s `into_lane` check), which a bare
  `Construct` cannot produce standalone, so there is no per-argument compile entry point to recompose
  through for that head. Documented in the module's own docs, not silently dropped.
- **New differential (`tests/concurrent_threeway_differential.rs`, M-858-style):**
  interp-sequential ≡ interp-parallel (M-862) ≡ AOT-parallel ≡ JIT-parallel over the `Op`-headed
  batch corpus, each pair validated through the shared M-210 `ObservationalEquiv` checker, with a
  `ran_aot`/`ran_jit` toolchain non-vacuity guard **and** a plan-level non-vacuity guard
  (`ConcurrentPlan::OpBatch` genuinely selected — never a silent fall-through to sequential). A
  **mutant witness** (`mutant_witness_catches_a_wrong_index_compose_aot`/`_jit`) demonstrates the
  differential actually *catches* a deliberately-broken concurrent dispatch (a wrong-index recompose
  over a non-commutative `trit.sub`), not merely asserts agreement. Verified non-vacuously on this
  box (libMLIR-18 + `llc`/`clang` present) with and without `--features mlir-dialect`.
- **M-865's original title over-claimed "AOT-runtime concurrency + async execution parity with the
  interpreter" — rescoped honestly, same day.** The language has **no executable concurrency
  surface** today (`hypha` ratified-not-lexed, `async` unimplemented, every execution path still runs
  sequentially), so that framing was vacuous. M-865's *actual*, landed scope is the harness-level
  extension above; real hypha/colony/async parity is carved out to a new post-1.0.0-tag issue,
  **M-869**, gated on the language growing a spawn/hypha surface. RFC-0008 §Meta and DN-61 §Meta
  carry matching append-only notes (RFC-0008 status unchanged: Accepted; DN-61 status unchanged).
  Guarantee tag: **Empirical** (differential-checked), never `Proven` (VR-5).

### Changed (2026-07-01: M-864 — persistent bounded work-stealing pool for the Scheduler, nested-safe submission)

- **`Scheduler::run_indexed` dispatches onto a persistent, process-wide pool (`mycelium_sched::pool`),
  not fresh OS threads per call.** The pool is created once, lazily, sized to
  `available_parallelism()`, and reused for the life of the process — including across **nested**
  `run_indexed` calls (a job that itself calls `run_indexed` again). A caller blocked on its own
  batch's completion **helps** drain the shared queue (`Pool::help_while`, the Cilk/TBB/Rayon
  work-helping pattern) rather than parking. The batch's lanes are **populated up front and never
  bare-block** (the queue is unbounded — no backpressure), so `help_while` is the *only* wait on any
  batch's critical path: the structural reason a **fixed**-size pool never deadlocks under arbitrarily
  deep nesting. Tag: **Empirical** — validated by **forced-low-worker-count** nested stress tests
  (`P ∈ {1,2,3,4}`, incl. the `[15,15,6]` shape) that *hang on the pre-fix code and pass on this one*,
  under a wall-clock timeout, plus global-pool stress + a Linux thread-count regression witness; not a
  mechanized proof (VR-5). This removes the resource concern M-860/M-862 both had to work around by
  capping their own parallelism to a single, non-nested, top-level batch.
- **Sound-on-arrival via an adversarial deadlock review (same day):** the *first* cut of this pool
  kept M-861's `capacity` backpressure, whose feeder bare-blocked *before* help-stealing — a real
  nested-submission deadlock at `width > capacity + P` (reproduced), plus a panicking job that hung
  the join and killed a pool worker. Both fixed at the root before landing: the **backpressure/
  `capacity` bound is removed** (it was the deadlock cause and a non-normative impl detail per DN-61
  §A.2 — the pool queue is now unbounded, memory bounded by the batch's job count), and the join is
  **panic-safe** (`std::panic::catch_unwind` per job keeps the persistent worker alive; the first job
  panic re-raises at the join, `thread::scope`-style; an RAII drop-guard decrements the batch
  countdown on every unwind path). The now-false `SCHEDULER_BACKPRESSURE_STRENGTH` (`Exact`) constant
  and its `mycelium-std-runtime` re-export are **removed** rather than left as a stale claim (VR-5);
  `Scheduler::capacity` / `with_workers(_, capacity)` remain for source compatibility but no longer
  bound anything (documented, never-silent).
- **Honest limit (never-silent, VR-5): bounded *progress*, not bounded *stack*.** `help_while` pops
  the shared queue indiscriminately, so under **deep-AND-wide** low-`P` nesting a single OS thread can
  stack help-steal frames from many sibling batches (~`O(w^(d-1))`) → a **stack overflow, not a
  hang**. So nested `run_indexed` is deadlock-free / panic-safe / deterministic at any depth but only
  **stack-safe for moderate depth×width**. The boundary was *measured* (DN-67 §3.4 table: e.g. at
  forced `P=1`, depth 5 completes at every tested width but depth 6 overflows at width 4), and a
  characterizing test (`[4,4,4,4]`, well inside the safe region) documents it. Current consumers
  (M-860/M-862) submit a single non-nested batch, so they are trivially safe. The `O(depth)`-stack
  leapfrogging fix is the tracked follow-up **M-868**.
- **Breaking API change, ratified: `run_indexed` now requires `F: Send + 'static` / `T: Send +
  'static`** (previously just `Send`, borrowing freely via the old `std::thread::scope`). A persistent
  pool's worker threads outlive any single call, so a job can no longer safely borrow from the
  caller's stack frame. Ratified in new **`docs/notes/DN-67-Persistent-Work-Stealing-Pool.md`**
  (`Draft`), which also carries the full caller-by-caller audit and the deadlock-freedom argument.
- **Every current caller adjusted** (none needed `unsafe`; the crate stays
  `#![forbid(unsafe_code)]`): `mycelium-mlir`'s M-860 `emit_llvm_ir_many_with_swap_mode` now clones
  each `Node` per job instead of borrowing it (determinism unaffected — the content-hash sort still
  runs over the original nodes first); `mycelium-interp`'s M-862 `eval_top_batch` now clones the
  `Interpreter` once per batch behind an `Arc` and shares an `Arc<AtomicU64>` fuel counter — made cheap
  by giving `Interpreter` `#[derive(Clone)]` (its `swap` field moves from `Box<dyn SwapEngine>` to
  `Arc<dyn SwapEngine>`, and `SwapEngine`'s bound widens from `Sync` to `Send + Sync`). Two callers not
  named in the M-864 issue's own body — found only by building the whole workspace, since
  `mycelium-mlir` transitively depends on `mycelium-std-runtime` — needed the same treatment:
  `dataflow::run_dataflow_scheduled` (M-711) now takes ownership of each still-pending task via
  `mem::replace` with a transient placeholder for the duration of a sweep's parallel poll, restoring it
  afterward; `supervision::run_supervised` (M-713) now clones its `CancelToken` per job (an
  `Arc<AtomicBool>`-backed handle, so every clone still shares the same cancellation flag).
- **`mycelium-std-runtime`'s inline tests extracted (M-797, as-touched):** `dataflow.rs` and
  `supervision.rs`'s former inline `#[cfg(test)] mod tests` blocks move to `src/tests/dataflow.rs` /
  `src/tests/supervision.rs`. The dataflow ownership-restore test is **strengthened** to assert the
  exact total step count (so a wrong-slot restore that strands a task is caught even when it doesn't
  deadlock), and the supervision cancel-token test is **honestly downgraded** to assert only the
  deterministic shared-flag propagation (the cross-sibling-observation claim was scheduling-dependent;
  `external_cancel_propagates_to_all_tasks` already covers per-job-clone flag-sharing deterministically).
- M-860's byte-identical parallel-emit test and M-862's parallel-eval differential/determinism suites
  are unmodified and re-verified green; `mycelium-sched` gains nested-recursion + **forced-low-`P`
  deadlock** + **panic-safety** stress tests (30/30 total); `mycelium-std-runtime` stays 98/98 green.

### Added (2026-07-01: ADR-036 — dogfooding and public-release strategy ratified)

- **ADR-036 — Dogfooding and Public-Release Strategy (`Accepted`, maintainer-ratified).** Fixes the
  `lang 1.0.0` **tag** and the project's **public release** as two distinct milestones. The tag is cut
  on the **Rust reference implementation**; self-hosting gates it only at the existing **core-lib
  self-host slice** (ADR-022 §8 Q1 — unchanged, explicitly preserved). **Comprehensive dogfooding**
  (progressively rewriting the whole toolchain/stdlib/kernel *in* Mycelium, beside the Rust originals —
  E18-1's full scope beyond the core-lib slice) is a first-class **within-1.0.0**, non-tag-gating,
  **parallel** track. Each Mycelium reimplementation is **Rust≡Mycelium differential-validated**
  (extending the interp≡AOT≡JIT discipline, RFC-0029 §7.5/M-210) and **replaces** its Rust counterpart
  only once tested, benched, validated, and it satisfies the maintainer. The repository **stays
  private** until dogfooding is complete and validated — the **public release** happens only then,
  refining the trigger condition **DN-27** (Draft, untouched) deferred to "a future ADR." This is an
  **additive** decision, not a §5/§8 Q1 criteria amendment (contrast ADR-024/034/035): ADR-022 §8 Q1
  and §10 each carry an append-only "see ADR-036" pointer, their own resolution/vision text unchanged.
- **Cross-reference application (ADR-036):** E18-1's issue body (`tools/github/issues.yaml`) carries an
  append-only, non-status-changing note framing it as the dogfooding-capstone track, roadmapped by
  `docs/planning/self-hosting-port-ledger.md` (which itself gets a header note to the same effect). No
  epic/issue status is flipped by this act.

### Added (2026-07-01: Phase-2 ratification — ADR-035 T4 scope amendment + RFC-0033/ADR-025..028 ratification)

- **ADR-035 — Full-Language 1.0.0 Gate (Track T4) Scope Amendment (`Accepted`, maintainer-ratified).**
  Narrows ADR-022 track T4's `lang 1.0.0` Definition of Done to the documented **stable-API freeze**
  (**DN-66**) + the **core-lib self-host slice** (M-714…M-718) — full RFC-0031 §5 D6 Rust-crate
  retirement for all 26 `mycelium-std-*` crates is **deferred to the post-1.0 long-term arc** (ADR-022
  §10), mirroring how §8 Q1 already narrowed T9 and how **ADR-024** narrowed T1. Grounded in DN-66's
  per-crate finding that zero crates clear the D6 trigger today (six same-named `.myc` nodules are
  structurally disjoint prototypes, not ports; `mycelium-std-runtime` is load-bearing —
  `crates/mycelium-mlir` depends on it directly). ADR-022 §5 T4 row + §8 Q1 carry append-only "narrowed
  by ADR-035" pointers (their normative text is not rewritten); RFC-0031 §5 D6 carries an append-only
  scope note (the D5/D6 mechanism itself is unchanged). **DN-66** itself moves `Draft → Accepted` by
  this ratifying act.
- **Issue status flips (ADR-035):** `M-719` (`in-progress` → `done` — its stable-API-freeze half now
  closes T4's narrowed 1.0.0 bar; the D6-retirement half is spun out to a new post-1.0 backlog item);
  `E13-1` epic (`in-progress` → `done` — all named children done under the narrowed scope). `E18-1`'s
  body carries a clarifying, non-status-changing note (its own remaining children, M-739…M-742, are
  unaffected and stay open). A new post-1.0 backlog issue, **`M-867`**, is minted (`status:todo`) to
  carry the full per-op D5/D6 retirement work forward.
- **RFC-0033 (Value-Model Collections & Precision): `Proposed` → `Accepted`** (maintainer-ratified).
  The value-model collections (`Seq`/`Bytes`) + the four paradigms' precision/width semantics
  (§1–§8) are ratified. **ADR-025, ADR-026, ADR-027, ADR-028 flip `Proposed` → `Accepted`** in the same
  act (ADR-029/030/031 were already Accepted, 2026-06-24, PR #536). **The V1–V5 kernel implementation
  (M-760…M-784 — the content-address one-way doors + swap/guarantee reconciliation) is deferred to
  post-1.0** — the design is ratified now; the value-model growth beyond the already-landed V0
  `BigTernary` (M-754…M-757, `done`) proceeds as a post-1.0 wave. No V-numbered implementation task
  (M-758…M-784) is flipped by this act.
- **Issue status flip (RFC-0033):** `M-785` (`in-progress` → `done` — its own Definition of Done,
  "RFC-0033 + ADR-025…031 reach Accepted," is now met). `E20-1` epic label moves `proposed` →
  `in-progress` (the design half is done; the epic itself stays open pending the deferred post-1.0
  implementation, not flipped to `done`).
- Stale cross-references to ADR-028's prior `Proposed` status updated append-only where they were
  cited as grounding (`docs/Doc-Index.md`, `docs/notes/DN-41-Width-Cast-Prim.md`,
  `docs/notes/DN-51-Binary-Width-Arithmetic-Promotion-and-Narrowing.md`) — each records the status at
  authoring time and notes the later transition; no finding or decision in those notes is revised.

### Changed (2026-07-01: M-863 — AOT ratification act: RFC-0029 → Enacted, DN-15 → Resolved, E15-1/E25-1/E19-1 status flips)

- **RFC-0029 (AOT Optimization, Codegen Maturity, and JIT): `Accepted` → `Enacted`.** With E25-1's
  remaining children — M-856b (Dense/VSA through the MLIR-dialect path), M-860 (parallel per-function
  AOT codegen), and M-862 (parallel pure-fragment interpreter eval) — landed this wave, every E15-1
  (M-725…M-729) and E25-1 (M-850…M-862) child is `done` with a checked three-way differential
  (M-858's unified mutant-witnessed harness, PR #851, 0-missed). The path this RFC sanctions is
  complete and stable — the condition its own Posture note reserved for `Enacted` (house rule #3:
  stepped through `Accepted` first). The interpreter stays the trusted-base reference throughout
  (ADR-007/NFR-7); this RFC governs only the native performance layer.
- **DN-15 (Native-Path Direct-LLVM Decomposition): `Draft` → `Resolved`.** The §10 status question
  this note's own prior resync flagged for the maintainer is now settled: M-856/M-856b/M-857/M-858
  closed the last open Increment-4 (MLIR-dialect) catch-up, so both halves the note decomposed —
  the direct-LLVM-advanceable half (§3) and the libMLIR-gated half (§2/§4.4) — are landed for the
  full ADR-034 coverage scope, each checked-differential. A §10 resolution paragraph is appended
  (append-only; no prior section rewritten).
- **DN-25 (Road to Full-Language 1.0.0): T6 row refreshed (advisory map, no status move).** All 15
  E25-1 + all 5 E15-1 children now show `done` in the §2 T6 row and §3 inventory.
- **ADR-034: DoD checkboxes updated, Status deliberately left `Accepted` (FLAG).** Every §5
  Definition-of-Done item except the terminal one is now met (E15-1/E25-1 coverage, RFC-0029
  Enacted, DN-15 Resolved, the ADR-022 pointers, `M-738 depends_on E15-1`). ADR-034's own Status
  field and its final DoD bullet both couple `Accepted → Enacted` to the `lang 1.0.0` tag act
  (M-738), which has not run (M-738 stays `status:blocked` on E13-1/E18-1). Per house rule #3/VR-5
  this is **not** flipped to Enacted here — flagged for the maintainer, not guessed past the
  checked tag-coupling basis.
- **Issue status flips:** `M-729` (`ready` → `done` — M-858's unified harness is the closing
  extension of its own differential-durability DoD, resolving a body/label inconsistency the prior
  close-out had only flagged); `E15-1`, `E25-1` epics (`in-progress` → `done` — all children
  verified done); `E19-1` epic (`in-progress` → `done` — a stale-label resync; M-746…M-752/M-798
  were already all `done`). `M-863` (this act) itself flips `ready` → `done`.
- Not flipped: **ADR-034's Status** (stays `Accepted`, tag-coupled to M-738 — see above).
- **Nav-index reconcile (dev→integration gate).** The M-863 header flip left one stale nav row —
  `docs/rfcs/README.md` still showed RFC-0029 as `Accepted`; synced it to `Enacted` (matching the
  authoritative header, forward-only). `docs/api-index/` regenerated for line-number drift. Both
  were caught by the full `just check` (`doc-status`/`doc-index` gates) at the integration promotion,
  not by the change-scoped leaf checks — the integration tier doing exactly its reconciliation job.

### Added (2026-07-01: Wave-1 — native-scheduler parallelism, dialect Dense/VSA, stdlib freeze, governance gates)

- **`mycelium-sched` foundational crate + Scheduler relocation (PR #864, previously untracked).**
  The M-861 work-stealing OS-thread `Scheduler` moved out of `mycelium-std-runtime` into a new
  crate, `crates/mycelium-sched` (deps: `mycelium-core` only, for `GuaranteeStrength`), landing it
  **below** `mycelium-interp` and breaking the `interp`↔`std-runtime` dependency cycle that blocked
  M-862's native-scheduler rewire. `mycelium-std-runtime` re-exports the same path, so downstream
  call sites (bench included) are unaffected. Pure structural refactor, no behavior change (DN-61
  §A.2: scheduler internal strategy is non-normative — only RT2 determinism is). This landed with
  no changelog entry at the time; recorded here.
- **M-856b — MLIR-dialect coverage for Dense/VSA (libMLIR-gated).** The dialect leg
  (`crates/mycelium-mlir/src/dialect/native.rs`) now lowers Dense/VSA element-wise ops, extending
  the three-way differential (interp == direct-LLVM == dialect) over Dense and all four
  1.0.0-mandatory VSA models (MAP-I/BSC/HRR/FHRR), matching direct-LLVM's existing
  `dense_codegen.rs`/`vsa_codegen.rs` fragment. Skip-graceful (`DialectError::ToolchainMissing`)
  where `mlir-opt`/`mlir-translate` are absent — never a faked pass. Tag: **Empirical** where
  libMLIR is provisioned.
- **M-860 — parallel per-function AOT codegen via the native scheduler.** Per-function/per-nodule
  lowering now dispatches through `mycelium_sched::scheduler::Scheduler::run_indexed`, joined by a
  stable content-hash sort so the parallel emission is byte-identical to the sequential emit (no
  new nondeterminism, emission order pinned). Reworked from an initial rayon-based prototype onto
  the native scheduler before landing — **no rayon** dependency added. Tag: **Exact** (byte-equal
  by construction, checked via the join-order differential).
- **M-862 — parallel pure-fragment interpreter evaluation via the native scheduler.** Independent,
  provably-pure Core IR fragments (a top-level `Op`/`Construct`'s direct argument batch) now
  evaluate in parallel through the same `Scheduler::run_indexed`, gated by the existing purity
  check and bounded to the outermost independent batch (no nested `run_indexed`); the choice is
  reified in an EXPLAIN-able `ParallelPlan` (`SequentialImpure` / `SequentialNoBatch` /
  `TopLevelBatch`, never a silent fallback). Differential-verified against the trusted sequential
  interpreter (25x determinism, purity-gate, fuel-parity, impure-fallback tests). Enabled by the
  Scheduler relocation above (PR #864) — **no rayon**. Tag: **Empirical**.
- **M-743 — MIT-only first-party license audit gate.** `scripts/checks/license-first-party.sh`
  added and wired into `scripts/checks/all.sh`, enforcing ADR-022 §7's MIT-only first-party policy
  as a standing green check rather than a one-time sweep.
- **M-674 — explicit recursion budgets on the totality and ambient passes.** Both passes now carry
  an explicit, reified depth budget (mirroring the checker/elaborator/parser/evaluator discipline
  M-674 already established) and refuse cleanly with a never-silent `*DepthExceeded` on
  exhaustion, rather than relying transitively on the parser's bound. The sibling `mono.rs`
  `free_vars`/pattern-binders recursion remains unbounded — an explicitly open follow-up, out of
  this issue's totality/ambient scope (flagged, not silently dropped — G2).
- **M-719 — stdlib stable-API freeze (DN-66).**
  `docs/notes/DN-66-Stdlib-Stable-API-Freeze-And-Rust-Crate-Retirement-Status.md` freezes the
  current public-API baseline for all 26 `mycelium-std-*` crates as a dated, grounded snapshot and
  assesses the RFC-0031 §5 D6 retirement trigger — finding no crate yet clears it (the 5
  same-named `.myc` prototypes are disjoint subsets, not full ports). Additive stability
  doc-comments added to all 26 crates; no crate retired, no `#[deprecated]` applied — retirement
  remains a separate, unmet precondition (DN-66 §4). Partial closure of M-719 (not fully done —
  the per-op audit precondition for retirement is still open).
  - Adversarial-review fixes on the above: `eval_core_parallel`'s top-level batch now defers to the
    trusted sequential `eval_core` on any argument error, closing a fuel-starvation divergence
    under concurrent scheduling; the four dialect-differential `assert!(ran, …)` sites
    (`dense_differential.rs`/`vsa_differential.rs`, value and measurement ops) are gated on
    `MlirTools::is_available()`, restoring the documented skip-graceful contract on a box without
    libMLIR; `license-first-party.sh`'s three license-line lookups get a trailing `|| true` so a
    missing license prints its finding instead of aborting silently under `set -e` (G2); and
    `mono.rs::finish` now maps a totality depth-budget trip to `ElabError::DepthExceeded` (not
    `::Residual`), so the M-674 refusal is reported honestly rather than mistaken for a semantic
    verdict.

### Documentation (2026-07-01: README/docs decomposition — leaner landing pages, topic-split guides, accuracy pass)

- **Root README decomposed (551 → 107 lines)** into a lean, navigable landing page plus nine linked
  topic docs under `docs/guide/` (why-and-design, guarantees-and-verification, workspace-map,
  comparisons, repository-structure, status-and-roadmap, decisions-and-reading-order, glossary,
  contributing-and-provenance), each with a ToC and back-nav. No content lost — relocated and cross-linked.
- **Tooling/experiment READMEs decomposed:** `tools/llm-harness/README.md` (570 → 167) split into
  MODEL-ACQUISITION / TERMUX-SETUP / GROK-HARNESS; `experiments/README.md` (263 → 84) with a new
  KC2-RUNBOOK; accuracy fixes (harness module map, KC-2 satisfied-kill-criterion status, and the
  `scripts/README.md` gate table corrected from 11 to the real 26 rows).
- **50 crate READMEs** given a uniform shape and `docs/api-index/INDEX.md#<crate>` nav, with grounded
  accuracy fixes (mycelium-mlir E25 surface, mir-passes fn names, bench module paths, std-collections `foldable()`).
- **docs/ reference + spec + wiki** nav footers, ToCs, and AOT-status accuracy; synced two stale
  decision-status references to the authoritative RFC bodies (RFC-0025 and RFC-0030 are **Enacted**,
  matched to their Status fields — no upgrade past basis).
- **Tooling:** `scripts/doc_currency.py` now reads the repository-structure tree from its new home
  `docs/guide/repository-structure.md` (README fallback retained); Node upgraded 18 to 22 so the
  `markdownlint-cli2` gate runs (was graceful-skip) — all 432 docs lint clean.

Verified: `markdown.sh` (432 docs, 0 errors), `links.sh`, `doc_refs_check.py`, and `doc_currency.py` all green.

### Added (2026-06-30: E25-1 staging-tier close-out — dynamic-VSA JIT, dialect Construct/Match/Swap, unified mutant-witnessed differential)

- **M-855 — JIT for dynamic VSA/HDC workloads (PR #848, RFC-0039 §6).** Runtime-specialized JIT
  execution for the four 1.0.0-mandatory VSA models (MAP-I, BSC, HRR, FHRR) covering
  `bind`/`unbind`/`bundle`/`permute`/`similarity` (`crates/mycelium-mlir/src/vsa_jit.rs`), reusing
  M-854's refusal surface and read-back verbatim (DRY). Differential against the interpreter is
  9/9 non-vacuous; `cargo-mutants` is 0-missed on local hardware (one BSC-majority boundary gap
  closed, two equivalent mutants justified inline). This is a partial landing, stated honestly:
  cleanup/resonator loops are explicitly **deferred**, not covered by this JIT path or its
  differential. Tag: **Empirical**.
- **M-856 — MLIR-dialect catch-up for Construct/Match and Swap (PR #850, libMLIR-gated).** The
  dialect leg (`crates/mycelium-mlir/src/dialect/native.rs`) now lowers `Construct`/`Match` (data)
  and `Swap` (binary↔ternary transcode) through the real `arith`/`func`/`cf` dialect path; the
  three-way differential interp == direct-LLVM == dialect is verified non-vacuous against a
  provisioned libMLIR. **Honest partial-landing split (G2 — not silently dropped):** Dense/VSA
  through the dialect is out of scope for this landing and is carried forward as a new issue,
  **M-856b**. Tag: Empirical where libMLIR is provisioned; skip-graceful
  (`DialectError::ToolchainMissing`) where absent.
- **M-858 — unified mutant-witnessed three-way differential (PR #851).** A single differential
  entrypoint (`tests/unified_threeway_differential.rs`) now covers interp / direct-LLVM /
  MLIR-dialect (plus JIT for the in-subset fragment) over element-wise/arithmetic, data
  (Construct/Match), and certified-swap corpora, including overflow parity and an honest boundary
  check that closures/recursion are actually verified refused by the dialect leg (not just
  claimed). Two coverage gaps in M-856's new dialect surface were found and closed with real
  witnesses — a swap-boundary case (`swap(8 → binary4)`, the first value past the target width)
  and a same-kind different-width `Match` case exercising the arm-shape `||` guard — closing **5
  dialect mutant survivors, 0-missed**. This is the checked basis that earns the native codegen
  claim its **Empirical** tag, not Declared (VR-5); it subsumes M-856's own dialect-witness
  obligation. Dense/VSA are not yet part of the dialect leg of this differential (deferred to
  M-856b).
- **Fixed — bench Swap capability-loss classification was stale (PR #849, M-852 follow-up).** A
  `mycelium-bench` test (`recursion_and_swap_are_capability_losses_and_data_is_never_silent`) hard-
  coded `Swap` as an always-a-capability-loss fragment on both compiled backends, a fact that
  predated M-852's native Swap codegen. Corrected: legal-pair `Swap` now folds into the same
  never-silent (value / capability-loss / skip, measured not pre-asserted — VR-5) obligation
  `Fragment::Data` already carries, since M-852's shared `lower_program` path lowers a legal
  binary↔ternary round trip to a `Value` on both compiled backends whose repr/payload/guarantee
  match the interpreter's (only the dynamic `Meta::provenance` differs, `Root` vs `Derived`, per
  RFC-0001 §4.6). Illegal-pair/unsupported swaps remain explicit capability losses; recursion
  (`Fix`/`FixGroup`) is unaffected and stays always-a-loss on both compiled backends.
- **E25-1 (epic) progress refresh.** 3 more children landed `done` this wave (M-855/856/858,
  bringing the total to 9 of the now-15 tracked children, after M-856's honest split adds
  M-856b); still open: M-856b (new), M-860 (parallel codegen), M-862/863 (post-tag-cautious
  perf-eval + ratification). `issues.yaml` records each landed child's `landed_pr`/`landed_date`/
  `landed_basis` plus an append-only "DONE" note to its body. Flagged, not silently corrected: the
  manifest shows M-859/M-861 still `status:ready` despite carrying `landed_pr` merges (PR #845,
  #843) from before this close-out — left for the next resync/maintainer rather than unilaterally
  flipped by this agent.

### Added (2026-06-30: E25-1 native-AOT full-coverage increments land — recursion, closures, Swap, Dense, VSA, trit.mul dialect)

- **M-850 — direct-LLVM full recursion (heap trampoline, PR #818).** Non-tail `Fix` + mutual
  recursion (`FixGroup`) now lower via a heap-allocated control-stack trampoline
  (`crates/mycelium-mlir/src/trampoline.rs`), bounded by the same `AutoDepthBudget` the env-machine
  uses (M-349, reused not reinvented) — deep recursion that previously refused now runs to a
  graceful `DepthLimit`, never a C-stack overflow (DN-05 #1, G2). Removes the `FixGroup` refusal
  (`llvm.rs:585`) and the DN-15 §8.5 Match-in-pre-tail limitation. `cargo-mutants` catches a
  trampoline-frame mutation (0 missed) on a checked basis, so the tag upgrades **Declared to
  Empirical** (VR-5) — not Proven; the differential is interp == direct-LLVM, not a formal proof.
- **M-851 — direct-LLVM closure-ABI widening (PR #821).** Closures over any repr/width, curried
  application, and closure-valued intermediate results now lower natively via
  **specialize-at-application inlining** — a `Lam` builds a suspended closure value and an `App`
  inlines its body at the concrete argument shape — removing the narrow packed-`i64` `Binary{8}`
  ABI (M-378) and its heap arena. This is an honest correction to the issue's original "uniform
  pointer-boxed lane" sketch: the realized mechanism is inlining, an architectural choice surfaced
  to and accepted by the maintainer, not a runtime box/unbox pair. Closure-valued *program results*
  and cross-boundary datum/`Fix` captures stay explicit never-silent `UnsupportedNode` (runtime
  dispatch deferred). `cargo-mutants` 8/0 missed → **Empirical**.
- **M-852 — direct-LLVM Swap native codegen (PR #823).** The `Swap` node — the only `Repr`-changing
  node (WF1) — now lowers natively for the certified binary↔ternary class
  (`crates/mycelium-mlir/src/swap_codegen.rs`): value-preserving enc/dec transcode in dumpable IR.
  The maintainer-ratified design resolves the issue's open FLAG with a **two-mode `SwapCertMode`**:
  **`Recheck`** (default) independently re-checks the certificate at compile time over
  `mycelium-core`; **`ReuseInterp`** (opt-in) carries the interpreter-computed certificate forward —
  both modes EXPLAIN-recorded (mode + source, no opaque choice). Never-silent refusals: Dense/VSA,
  illegal pair, over-`i64`-width (both modes), swap-in-recursion. Two real silent-miscompile bugs
  were caught and fixed *before* landing: an over-width `1<<64` overflow (`cargo-mutants`-caught)
  and an in-bound illegal-pair encode-quotient discard (review-caught). Tag: **Empirical**
  (Proven correctly not claimed — VR-5; cert-equivalence checked via the M-210 checker, not a
  formal proof of the lowering itself).
- **M-853 — native Dense lowering (PR #824, RFC-0039 §5.1).** Element-wise Dense ops
  (add/sub/neg/scale/dot/similarity) over the **un-quantized F32/BF16** fragment now lower natively
  (`crates/mycelium-mlir/src/dense_codegen.rs`), per RFC-0039's OQ-2 scoping. Three-way differential
  through the M-210 checker is bit-exact (the dialect leg honestly refuses Dense); `cargo-mutants`
  catches 67/70 viable mutants. Tag: codegen claim **Empirical**; the read-back carries the
  reference's own per-op tag (**Proven** for add/sub/scale, **Exact** for neg) — the native path
  preserves these only because it introduces no new approximation (VR-5). Quantized Dense
  (ADR-030 int/fp8/TF32) stays an explicit never-silent refusal, gated on **E20-1** landing
  `QuantDesc`.
- **M-854 — native VSA lowering (PR #825, RFC-0039 §5.2).** `bind`/`unbind`/`bundle`/`permute`/
  `similarity` over the four 1.0.0-mandatory models — **MAP-I, BSC, HRR, FHRR** — now lower
  natively (`crates/mycelium-mlir/src/vsa_codegen.rs`), mirroring `mycelium-vsa` digit-for-digit.
  Three-way differential through the M-210 checker is bit-exact (dialect leg honestly refuses VSA).
  **Honest per-op tags carried exactly per RFC-0039 §5.2 — no upgrade past the checked basis
  (VR-5):**
  - **Proven is preserved ONLY for the single-op MAP-I `bundle` capacity bound** (the checked
    instantiation `capacity::proven_capacity_bound`, replayed in `capacity.rs`, basis
    `proofs/lh-bundle`/M-001). The multi-hop compositional work (M-832) is in-progress research
    emitting undischarged proof obligations and **never** stamps Proven — codegen does not borrow
    from it.
  - **HRR/FHRR `bundle` is Empirical within a measured capacity profile** — the reference's
    documented `EmpiricalProfile` coverage window (odd `m ≤ 5`, `d ≥ 1024`, single-factor,
    codebook ≤ 16) — and **refuses `OutsideEmpiricalProfile`** beyond it; never a silent
    Empirical-anyway.
  - **SBC/MAP-B (niche models, OQ-3) and quantized/block-sparse/complex-carrier VSA (ADR-031,
    OQ-4) stay explicit never-silent refusals**, gated on E20-1 landing the carrier `Repr` fields.
  - `cargo-mutants` is **0-missed via toolchain-independent emission/read-back witnesses + 4
    inline-justified equivalents** — this property is **cross-environment** (does not depend on a
    local `mlir-18-tools`/libMLIR install present in the witnessing environment), distinct from
    M-857's libMLIR-gated witnesses below.
  - **Heavy / large-hypervector-dimension VSA workloads and the full mutant-durability pass beyond
    the toolchain-independent witness set are GPU-deferred (maintainer-run)** — the landed claim
    covers the CPU small-dimension envelope only; large-dim profile extension is a follow-up.
  - RFC-0039 stays **Accepted** (implemented Rust-first, pending ratification) — this landing does
    **not** move it to Enacted (house rule #3: Enacted requires the full E25-1 path complete +
    stable, not one increment).
- **M-857 — `trit.mul` through the real MLIR-dialect path (PR #820, libMLIR-gated).**
  Balanced-ternary `trit.mul` (shifted-accumulate, 2m-trit buffer) now lowers through the real
  `arith`/`func`/`cf` dialect path (`crates/mycelium-mlir/src/dialect/native.rs`), sharing
  `emit_trit_add_step` with the direct-LLVM `emit_trit_mul` (DRY). The
  `DialectError::Unsupported("trit.mul")` refusal is removed; the dialect boundary moves to
  closures/recursion/data/`Swap`/Dense/VSA (each an explicit, test-pinned refusal — **M-856**
  is the tracked dialect catch-up for those fragments). **This witness is libMLIR-gated** (unlike
  M-854's toolchain-independent witnesses): the three-way differential ran non-vacuous against a
  provisioned libMLIR 18.1.3, `cargo-mutants` 9-caught/0-missed in that environment. Tag:
  **Empirical** where libMLIR is provisioned; skip-graceful (`DialectError::ToolchainMissing`)
  where absent — never a faked pass (G2/VR-5).
- **E25-1 (epic) status moves `todo` → `in-progress`.** 6 of 14 children landed `done` this wave
  (M-850/851/852/853/854/857); the remaining 8 (M-855 dynamic-VSA JIT, M-856 dialect catch-up,
  M-858 unified mutant-witnessed three-way differential, M-859 bench scaling, M-860 parallel
  codegen, M-861 scheduler work-stealing, M-862/863 perf-eval + ratification) are unstarted this
  wave. `issues.yaml` records each landed child's `landed_pr`/`landed_date`/`landed_basis` plus an
  honest "DONE" append to its body (append-only — house rule #3); none of RFC-0029 → Enacted,
  DN-15 → Resolved, or ADR-034 → Enacted are claimed by this partial landing.
- **Methodology resolution — scoped-toolchain setup (M-848).** A sibling toolchain effort is
  reported (this wave) to be making `just setup`/the scoped-toolchain tooling idempotent and
  auto-installing, so that differential tests run non-vacuous (genuinely exercising the toolchain
  path, not silently skipping it) without changing test semantics — superseding the previously
  proposed separate non-vacuity guard with the simpler fix at the toolchain layer. `M-848` is
  recorded `in-progress` in `issues.yaml` with a marker noting no corresponding branch/commit was
  found in `origin` as of this resync (flagged for maintainer confirmation), pending its own
  landing report.

### Added (2026-06-30: RFC-0039 — Native Dense & VSA Codegen, Accepted)

- **RFC-0039 (Accepted, maintainer-ratified 2026-06-30)** is the design vehicle (ADR-034 §6) for
  native codegen of `Repr::Dense` and `Repr::Vsa` plus the dynamic-VSA JIT — the gap RFC-0029 §3
  explicitly excludes. It decides design only (ratifies the design; asserts no implementation;
  M-853/M-854/M-855 stay design-gated on it; → Enacted only when the path is complete + stable). The
  four open questions were resolved at ratification: **OQ-1** — the §6 cross-reference IS the vehicle
  for the ADR-009 dynamic-VSA JIT deferral lift, no separate ADR-009 amendment; **OQ-2** — native Dense
  scopes to the F32/BF16 un-quantized fragment first, the full ADR-030 int/fp8/TF32 quant/accumulator/
  packing set widens as E20-1 lands `QuantDesc`; **OQ-3** — the standard models MAP-I/BSC/HRR/FHRR are
  1.0.0-native-mandatory, the niche SBC/MAP-B extend post-mandate; **OQ-4 (both)** — native codegen
  covers the un-quantized/real fragment now AND commits to widening to quantized-Dense (ADR-030) plus
  element-space/block-sparse/complex VSA (ADR-031), gated only on **E20-1** landing those `Repr` fields
  (the enabling dependency for the full-coverage half), refusing the unbuilt variants never-silently in
  the interim. The native path preserves the RFC-0003 §4.1 per-op tags only where the checked basis
  holds (VR-5 — single-op MAP-I bundle Proven via `proofs/lh-bundle`/`capacity.rs`; the multi-hop M-832
  work stays in-progress research, never Proven). Carried by a M-210-checked, mutant-witnessed,
  interpreter-referenced honesty contract. Advances E25-1 / ADR-034 track T6.

### Fixed (2026-06-30: branch-guard PreToolUse hook — worktree resolution)

- **The branch-guard PreToolUse hook now resolves the branch from the command's worktree, not the main
  checkout.** `scripts/hooks/claude-git-branch-guard.sh` keyed the protected-branch decision off
  `CLAUDE_PROJECT_DIR` (the main checkout), so it false-positived an **isolated worktree agent**
  committing to its own leaf branch whenever the main checkout sat on a protected branch (`dev`) — the
  worktree variant of mitigation #12. It now reads the payload's `cwd` (the directory the git command
  runs in) and judges THAT worktree's `HEAD`, with `CLAUDE_PROJECT_DIR` only a fail-safe. The guard
  stays fully armed: a real commit/merge/push on a protected branch — in any worktree — and any
  force-push still block. Verified with five cases (leaf-commit ALLOW · dev-commit BLOCK · force-push
  BLOCK · push-to-dev BLOCK · non-git ALLOW).

### Added (2026-06-30: concurrent-PR pattern operationalized as parameterized skills — `/wave`, `/pr-land`, `/worktree-guard`)

- **Three new parameterized skills** capture the concurrent-PR development pattern as enforceable,
  reusable agent tooling (the `/branch-guard` shape) so the discipline holds by construction and
  need not be re-explained per wave.
  - **`/wave`** (`skills/wave/SKILL.md`) — umbrella for §Concurrent-PR development: partition by
    file ownership → one isolated `git worktree` per agent → change-scoped leaf checks and own-issue
    updates → per-PR review and merge via `/pr-land` → integration-tier close-out; `main` stays the
    terminal maintainer checkpoint. Parameterized by `ITEMS`/`MODE`/`BASE`.
  - **`/pr-land`** (`skills/pr-land/SKILL.md`) — per-PR agent-review loop: an isolated Sonnet
    `/pr-review` agent posts findings as PR comments → patches → replies → updates the description
    → merges the PR **up the tree** (onto the working or staging tier; stops before `main` — that
    is `/land`). Parameterized by `PR`/`BASE`/`MODEL`.
  - **`/worktree-guard`** and **`scripts/checks/worktree-guard.sh`** + the **`just worktree-guard`**
    recipe (alias `just wg`) — the isolated-worktree safeguard (CLAUDE.md mitigation #11), idempotent
    and parameterized: `--leaf` asserts a concurrent agent is in an isolated worktree; `--orchestrator`
    (default) resolves and checks the **main** worktree (the first `git worktree list` entry), so it is
    correct even when invoked from a linked worktree. Shellcheck-clean; `--quiet` mode for hook/CI use;
    never-silent (G2). Wired into the justfile mirroring `just branch-guard`.
  - **CLAUDE.md**: skills list, mitigation #11 enforcement note, and §Concurrent-PR development
    "operationalized as skills" pointer updated. **CONTRIBUTING.md**: concurrent-PR bullet updated
    to name the three skills.

### Changed (2026-06-30: ADR-034 — native AOT re-gated INTO `lang 1.0.0`)

- **ADR-034 — Full-Language 1.0.0 Gate (Track T6) Re-Gating: `Accepted`** (maintainer-ratified,
  append-only). **Reverses ADR-022 §8 Q4** (which un-gated native AOT to `1.1`): epic **E15-1** (native
  AOT) is **re-gated INTO `lang 1.0.0` as a hard gate row**, scope expanded to **full-language
  native-codegen coverage** — closures, non-tail and mutual recursion, `trit.mul`, `Swap`, Dense, VSA,
  and JIT for dynamic VSA/HDC — delivered "through the lowers" over scoped PRs. ADR-022 §3/§5 T6/§8 Q4
  carry append-only "re-gated by ADR-034" pointers (the Q4 resolution text is preserved, not rewritten —
  house rule #3). `M-738` (the `lang 1.0.0` release act) now `depends_on E15-1`.
- **Program registered (umbrella epic `E25-1`, issues `M-850`…`M-863`).** The native-AOT
  full-coverage increments (recursion trampoline, closure-ABI widening, `Swap`/Dense/VSA codegen,
  dynamic-VSA JIT, dialect catch-up, unified mutant-witnessed three-way differential), the perf +
  parallelism extension (bench single+multicore scaling + regression gates, parallel per-function
  codegen, scheduler work-stealing, parallel pure-fragment eval), and the ratification act are
  registered in `tools/github/issues.yaml` with user stories + Definition of Done. **RFC-0039** (Native
  Dense & VSA Codegen) is the proposed design vehicle for the Dense/VSA increments.
- **Honest posture (VR-5/G2).** The native AOT is implemented and landed on `main` (waveN2) and **builds and tests
  pass** at the bit/trit and bounded-data subset (verified 2026-06-30; the `mlir-dialect` leg
  skips gracefully where libMLIR is absent, ADR-019). Full coverage is **not yet met** and is **not**
  claimed — each E25-1/E15-1 leaf stays `Declared` until it lands with a checked three-way differential
  (interp ≡ AOT ≡ JIT), mutant-witnessed. The interpreter remains the trusted-base reference; the AOT
  path stays outside the kernel (KC-3). RFC-0029 moves `Accepted → Enacted` only at completion (M-863).

### Changed (2026-06-29: RFC-0038 ratified — `Proposed → Accepted`)

- **RFC-0038 — Inject-Mode Security Axis: `Proposed → Accepted`** (maintainer approved, append-only).
  The full inject-mode security + trust model is ratified — `loose`/`inoculated` modes, `InjectCert` =
  spore signature, enforcement granularity (§8.4), scope resolution + deviation manifest (§8.5),
  defaults by project kind (§8.6), interpreted opt-in signing + `BadSignature` (§8.7), and the colony
  trust topology (§8.8). **Acceptance ratifies the *design*, not an implementation:** the mechanism is
  unbuilt, so every mechanism claim stays `Declared` (VR-5) until **Enacted** Rust-first (§13
  Implementation DoD). Open R&D (§K.2/§L/§M; §8.8 controller protocol/blacklist — M-849) carries
  forward, not closed by acceptance.

### Changed (2026-06-29: RFC-0038 §8.8 — colony trust topology + #772 review fixes, M-849)

- **RFC-0038 colony trust topology (`Proposed`; maintainer direction).** Adds §8.8: a mesh
  distributes trust in one of two **configurable topologies** — **controller mode** (one or more
  controller colonies / a redundant, regionally-partitioned **controller stack** distributing the
  `TrustRoot`, for enterprise-scale central management of tens of thousands of colonies) vs
  **masterless mode** (each colony self-manages trust against its own internal store, §7.2), plus
  **node invalidation/blacklist** (permanent or temporary, config-driven, node-level trust
  revocation — never-silent). Framed by the **no-black-box-by-construction** inspectability thesis
  (`reveal`/`EXPLAIN`/provenance) that makes self-developing AI meshes auditable. Controller
  protocol / blacklist semantics / topology transition are open infrastructure R&D (extend RFC-0008
  mesh). Folds in the **#772 review fixes**: §13 Design+Implementation DoD now enumerate §8.4–§8.8 +
  `BadSignature`/granularity/deviation/blacklist conformance; §M hierarchy notation reconciled to the
  normative §8.5; §5.1 `BadSignature` dual-path clarified; `(configurable)` dropped from the library
  default row; §8.4 two-knobs wording. All `Declared`; enacts nothing. RFC-0038 now carries the full
  enforcement + trust model and is ready for maintainer approval.

### Added (2026-06-29: DN-65 — scoped-PR decomposition & per-PR toolchain scoping workflow policy, M-848)

- **DN-65 — scoped-PR decomposition & workspace prep (`Accepted` workflow policy; maintainer-directed).**
  Large work is **done at any scale but lands as logical, closely-scoped PRs** (soft ~1–2k-LOC-delta
  rule of thumb — cohesion over a line count; a 50k-line wave lands as a fan/sequence of small,
  individually `/pr-review`'d PRs). Before working a unit: **sync off the latest tip** and
  **pre-install the toolchain the change-kind needs** (the DN-65 §2.3 change-kind→tool map: Rust →
  `just setup`; Python → `uv sync`; docs → markdownlint/`doc_refs`; proofs → `z3`/LH/Lean) — workspace
  prep, so nothing surprises mid-flight. The PR-landing twin of DN-20 (change-scoping) and the swarm
  file-ownership partition. Distilled into CLAUDE.md (Commits & PRs), CONTRIBUTING.md, and the skills
  `/dev-workflow`/`/land`/`/kickoff`; the scoped-setup automation (`just setup-scoped`) is tracked as
  **M-848**.

### Changed (2026-06-29: RFC-0038 §8.4–§8.7 — enforcement granularity, scope resolution, and the deviation manifest, M-847)

- **RFC-0038 enforcement-granularity model (`Proposed`; maintainer direction).** Adds an
  **enforcement-granularity** axis orthogonal to the `loose`/`inoculated` mode: `whole`
  (application/spore signature checked once at compile/load — the **application default**, NOT
  per-call), `module` (per-phylum/nodule), and `call` (per-dispatch — the opt-in trusted-computing
  extreme). A **scope-resolution hierarchy** (`global ⊃ project ⊃ colony ⊃ module ⊃ nodule ⊃
  function ⊃ line`) sets the posture once and **auto-decorates everything beneath**, with **granular
  override** (open up or lock down a specific site) and a never-silent **default-plus-deviations
  manifest** (G2 — the declared default plus an enumerated list of the sites that differ). **Defaults
  scale to project kind/maturity** (scripts/interpreted/early → `loose`; library → `inoculated`/
  `module`; application → `inoculated`/`whole`; trusted-computing → `inoculated`/`call` opt-in). The
  interpreted path defaults `loose` but supports **opt-in per-inject signing** (dev private key signs,
  `TrustRoot` public key verifies; `InjectError::BadSignature` added for a wrong/untrusted signer
  alongside `UnsignedCode`). Gives §M/OQ-M its shape (residual R&D narrowed to the config surface);
  advances M-836/M-838/M-840. All `Declared`; enacts nothing.

### Added (2026-06-29: VSA proof-discovery — all three effective-`m` models + both Lean 4 and Liquid Haskell, M-832)

- **All three effective-`m` models, comparatively (M-832 / OQ-F).** The `--proof` mode now discovers
  and emits obligations for **all three** candidate models (`A_exponential` / `B_linear` / `C_sqrt`)
  across all three compositions in one run, with a **comparative ranking per composition** in
  `PROOF-SUMMARY.md` (tightest valid upper bound; refuted models listed explicitly, never silently
  dropped — G2). The maintainer reads the comparison rather than pre-choosing a model.
- **Both proof assistants — Lean 4 and Liquid Haskell.** Alongside the SMT-LIB (refutation pattern) and
  Liquid-Haskell skeletons, a new **`emit_lean()`** emits Lean 4 probes (`axiom candidateCapacityThm` +
  per-point `native_decide` arithmetic instantiation), with a `proofs/vsa-multihop-bound/lean/` scaffold
  (`lean-toolchain` pinned to `leanprover/lean4:v4.15.0`, `lakefile.toml`, a representative module). The
  Lean path also feeds the OQ-A/M-827 mechanization (research/26 recommends Lean 4). VR-5: both
  assistants **axiomatize** the candidate theorem and discharge only the arithmetic — neither stamps
  `Proven`. A committed **`EXAMPLE-*`** obligation set (from a real CPU `--demo` run; 6 in-regime probes,
  3 refuted cases honestly reported) makes the output concrete without running anything.

### Added (2026-06-29: DN-64 §7 maintainer dispositions — RFC-0038 inject-mode security axis, research/26+27 R&D records, VSA-bounds GPU experiment, M-827…M-846)

- **DN-64 §7 — maintainer dispositions on all 20 open questions (OQ-A…OQ-T).** Each OQ recorded at
  the strength the maintainer set it to, none upgraded past its basis (VR-5); OQ-H's R&D disposition
  was supplied after the initial 19. Ratifies the production hot-inject mode rename `sealed` to
  **`inoculated`** (`loose` retained for local-dev); routes the hot-inject cluster (OQ-K…OQ-Q) to
  RFC-0038; mints tracking issues M-827…M-846. Append-only; DN-64 stays `Draft`.
- **RFC-0038 — Inject-Mode Security Axis (`Proposed`; enacts no code).** A hot-inject security axis
  **orthogonal** to the fast/certified cert axis (RFC-0034 §8): `loose` (unsigned injection permitted,
  every injected call G2-tagged) vs `inoculated` (a valid `InjectCert` required, never-silent
  `InjectError::UnsignedCode` refusal, gating the interpreter-fallback path too). The `InjectCert`
  **is** the spore's signature component (ADR-013 §2 comp. 4) — `myc-prepare` signs a spore that is
  both deployable unit and inject gate, fusing the gate with the VR-4 no-opaque-lowering attestation
  (DN-18/M-630; ADR-006 for EXPLAIN-ability). A colony verifies the cert valid/trusted/unexpired/unsuperseded against its **own**
  `TrustRoot`; signing authority is project-scoped and graded by scope-of-work. Key-management detail,
  replay/expiry, and inject-mode scoping (§K.2/§L/§M) are named open R&D. References RFC-0034/ADR-013/
  ADR-017 without changing them (append-only).
- **Research Records 26 + 27 — DN-64 R&D planning.** `research/26` (type system — graded-soundness
  proof path, E2-1 bound composition, three-layer memory ergonomics, substrate/hypha reclamation,
  per-instantiation grades) and `research/27` (ergonomics — `forage`/`backbone` activation plus
  mechanized EXPLAIN-able policy capture, guard clauses, short-keyword scope, annotation-burden
  wrappers, composite aggregation, proposal-time naming gate, record-literal shadowing). All proposals
  `Declared`; external mechanisms `Empirical` at source.
- **VSA compositional-`Proven`-bounds GPU experiment (M-832, OQ-F).** A runnable harness at
  `experiments/mycelium_experiments/vsa_bounds/` — numpy reference path (always runs) plus an optional
  torch/CUDA accelerator, never-silent backend selection — that reimplements `capacity.rs::required_dim`
  at exact parity and sweeps `single` (bundle-capacity, the `Proven` anchor) and `multihop`
  (bind-chain / bundle-of-binds / nested-unbind) failure rates across `{model, F, k, d, h, δ}` to map
  where a closed-form bound still tracks the measured rate. VR-5: it measures rates only — the
  "this subset admits `Proven` bounds" verdict stays the maintainer's, from `SUMMARY.md` plus plots.
  **Extended toward the mechanical proof (the OQ-A bridge):** a `--proof` mode discovers candidate
  closed-form multi-hop bounds (`candidate_bound.py` — effective-`m` models, fit plus never-silent
  regime validation) and emits checkable proof obligations (`proof_obligation.py` — SMT-LIB and
  Liquid-Haskell skeletons mirroring `proofs/lh-bundle/` and `capacity.rs`'s checked-instantiation),
  scaffolded under `proofs/vsa-multihop-bound/`. It proposes a theorem and emits the obligation a
  prover must discharge — it never stamps `Proven`. 29 CI tests green (no torch required); `uv sync
  --group gpu` enables the GPU path.

### Added (2026-06-29: serial-lane closeout — M-822 partial application, M-826 tuple type, M-823 or-patterns + R20-Q5, M-824 DN-54 design-pass, M-825 backbone)

- **Multi-argument partial application via currying (M-822; RFC-0024 §4A.8).** A multi-param `lambda`
  or named fn used as a value curries into nested single-param closures, reusing the M-704 Reynolds
  defunctionalization machinery; `f(x)` yields a partially-applied closure. The "tuple-gated" premise
  proved unnecessary — currying needs no tuple type. KC-3 preserved (no new L0 node); three-way
  differential agreement (`Empirical`).
- **v0 tuple/product type + `f(x)(y)` chained application (M-826).** A first-class tuple type usable
  wherever any type appears: tuple literals `(a, b)`, tuple types `(T, U)`, tuple patterns and `let`/
  `match` destructuring, nested tuples, and multi-value return. Desugars to a synthetic single-ctor
  `Tuple$N` `Construct` (KC-3 — `mycelium-core` untouched). The first-order application restriction is
  lifted so inline `f(x)(y)` works (routes through the §4A.5 apply dispatcher). Verified **three-way**
  (L1-eval ≡ L0-interp ≡ AOT) — the differential caught and forced the fix of a desugar-completeness bug
  (tuples must desugar through mono even for non-generic programs). Flagged for a later maintainer call
  (non-blocking): positional projection (`t.0`) is destructure-only; unit `()` is arity-≥2-only.
- **Or-patterns + list bidirectional inference (M-823; RFC-0020 §9).** Match arms accept or-patterns
  `A | B => e`, desugared at the checker to one arm per alternative (KC-3 — no new L0 node) with a
  never-silent binding-consistency check (alternatives must bind the same names at the same types) and
  union exhaustiveness. List literals get bidirectional element-type inference from context (R20-Q5);
  the `for`-body→spine two-pass feedback remains a flagged open item (RFC-0020 §9, never-silent).
  Three-way differential for both.
- **DN-54 §10 — derive-site attachment design-pass (M-824, `Draft` addendum).** Enumerates two
  attachment models (sibling-item injection vs derived-impl registry) with an honest tradeoff and a
  recommendation (Model A); DN-54 stays `Accepted` (design only, not implemented). Surfaces five open
  questions for the implementing RFC.
- **`backbone` = runtime-dynamic promoted (M-825).** Records the maintainer decision in RFC-0008 §4.5
  (append-only) and resolves DN-63 FLAG-15; the future `backbone` implementation RFC proceeds on the
  promoted-dynamic model. `Declared`; RFC-0008 status unchanged.
- **Integration:** `mycelium-fmt`/`mycelium-check` render the new tuple and or-pattern AST variants;
  `mycelium.ebnf` gains the tuple type/literal/pattern and or-pattern productions. KC-3 held across the
  whole wave (`mycelium-core` untouched). Full `just check` at the dev landing.

### Added (2026-06-29: DN-64 — language-design synthesis exploration note, research aside)

- **DN-64 — Mycelium Language Design: Synthesis Exploration Note (`Draft`, advisory).** A research
  synthesis (commissioned aside) over five parallel corpus sweeps — surface ergonomics, Mycelium-unique
  types/constructs, unique application capabilities, the hot-inject security model, and conventions —
  produced by a small Haiku/Sonnet research swarm. Maps each unique construct (never-silent repr swap,
  the guarantee lattice as a type-level property, provenance/`Meta`, `substrate`, bounded effects) to a
  traditional paradigm and frames it as an extension; sketches small apps only Mycelium makes natural;
  and proposes a signed/cert-gated **hot-inject** model with `loose`/`sealed` modes as a new RFC-0034
  axis orthogonal to the fast/certified swap-cert axis. **Proposes nothing normatively** — every claim
  `Declared`, with 5 recommendations and 8 open questions surfaced for maintainer ratification (VR-5/G2).

### Changed (2026-06-29: DN-57 → Enacted — delimiter semantics surface complete)

- **DN-57 advances `Accepted → Enacted` (append-only; house rule 3).** The delimiter-semantics
  surface is now fully implemented and green on `main`: **M-818** (the mandatory `;` component
  terminator, the nodule-header terminator, `mycfmt`/`expand_to_source` `;`-emission, and the corpus
  migration), **M-819** (`mycfmt --flatten` single-line stream form), and **M-820** (`myc --stream`
  token-driven streaming
  parse). The §2/§5 streaming, comment-safety, and never-silent ergonomics claims that were `Declared`
  are now `Empirical` for the implemented surface (M-820's comment-safe splitter with explicit
  lex/parse/eof/empty errors; M-819's AST-equal round-trip); claims about not-yet-built variants (true
  incremental I/O, the on-the-wire encoding) stay `Declared`. The terminator still adds **no AST node**,
  so Enactment introduces no kernel growth (KC-3). Recorded in DN-57 §6; no `§1–§5` content rewritten.

### Added (2026-06-29: next-wave — M-819 `mycfmt --flatten`, M-820 `myc --stream`, M-677 effect budgets, M-668 DN-63 R2 planning)

- **`mycfmt --flatten` — single-line human↔stream form (M-819; DN-57 §2).** A new formatter mode
  emits a whole nodule on one line (`nodule d; item1; item2;`), the unambiguous stream form that the
  mandatory `;` (M-818) makes well-defined. Renders from the AST via the existing canonical machinery
  (a layout-policy switch, not a parallel formatter); comments and `// @key:` structured-header
  metadata are stripped, recorded explicitly in the result notes (G2, never silent). Round-trip
  `parse(flatten(src)) == parse(canonical(src))` is `Empirical` (corpus plus conformance over the full
  accept set). The `--flatten --write` combination is refused explicitly.
- **`myc --stream` — streaming-parse CLI entry (M-820; DN-57 §2).** Consumes `;`-terminated
  components from stdin or a file. The splitter is token-driven: it lexes via
  `mycelium_l1::lexer::lex`, segments the token stream at `Tok::Nodule` header tokens, and checks each
  segment ends with the `Tok::Semi` terminator, so a `nodule` or `;` inside a comment or string literal
  can never mis-split (comment-safe by construction, `Empirical`). Never-silent on malformed input —
  explicit located `myc-stream-lex` / `-parse` / `-eof` / `-empty` diagnostics, and a failed component
  does not abort the good ones (G2). v0 buffers the whole input (`Declared`); true per-component
  incremental I/O awaits a resumable parser entry in `mycelium-l1` (flagged follow-up).
- **Declared effects now consume a runtime budget ledger, plus per-effect budget syntax (M-677;
  RFC-0014 §3.4/§4.5 I4).** A fn's declared effects are threaded through evaluation into the
  `mycelium-interp` budget ledger (M-353): the ledger is primed per invocation with the fn's ceiling,
  one unit is consumed per budgeted effect, and an overrun yields the explicit `EffectBudgetExhausted`
  (`L1Error::EffectBudget`), never a hang or OOM (G2). Surface syntax extends the effect annotation
  with an optional bound — `!{retry(<=3), alloc(<=64KiB)}` — parsed as `eff(<=N)` with an optional
  binary-size suffix (`KiB`/`MiB`/`GiB`, folded to a byte count); a zero budget and a duplicate effect
  name are explicit refusals. **KC-3 preserved: no new L0 node** — the budget is a
  `FnSig.effect_budgets` field threaded as metadata, with `mycelium-core` untouched. Budget
  monotonicity and the under/at/over-budget paths are tested, and the M-210 three-way differential
  agrees (`Empirical`, v0 per-call model).
- **Integration fix (M-677 with M-819): `mycfmt` now renders the budget bounds.** The formatter
  emitted only the effect names (`!{retry, alloc}`), which would have silently dropped the `(<=N)`
  bounds and broken round-trip for budgeted fns; it now renders `name(<=N)` (raw byte count,
  AST-equal), with a regression test that compares against the original parsed AST.
- **DN-63 — RFC-0008 R2 distribution-vocabulary planning (M-668, `Draft`).** New design note
  decomposing the six R2 constructs (`xloc`, `mesh`, `cyst`, `graft`, `forage`, `backbone`) into
  per-construct implementation-RFC tracks with dependency ordering
  (`forage` then `backbone` then `mesh` then `graft` then `xloc` then `cyst`), per-construct
  typing/elaboration sketches, and honest guarantee tags (all `Declared` at planning stage; `mesh`
  probabilistic RT5/T4.2, `xloc` fallible RT4). R2 is explicitly gated on R1 completion (M-667).
  Surfaces a maintainer decision — `backbone` declared-vs-promoted (manifest-level vs runtime) — to
  settle before its implementation RFC.
- **Verified:** `cargo build --workspace` clean (the M-677 `FnSig` field is additive, so no fmt/cli
  exhaustive-match break); `cargo test` green for `mycelium-l1`/`-fmt`/`-cli`/`-interp`; clippy
  `-D warnings` clean. Full `just check` run at the dev landing.

### Changed (2026-06-29: strm — M-818 mandatory `;` component terminator (DN-57); closes M-821)

- **`;` is now the MANDATORY component terminator (DN-57 follow-on; M-818).** Required after the
  nodule header and every top-level item, trait signature, `impl`/inherent method, and `object`
  member. A missing terminator is a never-silent `ParseError` naming the component (G2); fully
  whitespace-free source (`nodule d;fn a()=>…;fn b()=>…;`) now parses. **DN-57 §3 settled**
  (append-only): uniform rule — a `}`-closed block still takes the trailing `;` (deliberately not
  Rust's "`}` ends the item"); the terminator adds no AST node. `mycfmt` emits `;` canonically.
  `mycelium.ebnf` updated (`nodule_block ::= nodule_header ';' (item ';')*` — the header carries its
  own mandatory `;`, and a header-only nodule is well-formed).
- **Workspace-wide corpus migration (closes M-821):** every `.myc` source and in-test Mycelium
  program string gained its `;` — 25 accept + 24 reject conformance fixtures (+ new
  `reject/29-missing-semicolon-terminator.myc`), `lib/std/**`, the examples, and ~565 in-test
  strings across `mycelium-l1`/`-fmt`/`-cli`/`-lsp`/`-doc`/`-check`/`-bench` + `experiments/`.
- **Two pre-existing breakages this surfaced + fixed (transparency):** (1) `mycelium-fmt` did not
  build on the base — the M-664 `Expr::Consume` / `Item::InherentImpl` variants left non-exhaustive
  matches in the fmt crate (M-664 was verified with `cargo test -p mycelium-l1` only, not a workspace
  build — lesson: AST-variant changes need a reverse-dependent build); the missing arms were added.
  (2) A stale `ambient.rs` ternary printer (`<…>` → `0t…`, RFC-0037 D4).
- DN-57 recorded **implemented (Rust-first)** append-only — **NOT** flipped to `Enacted` (the
  `mycfmt --flatten` / `myc --stream` tooling, M-819/M-820, remains separate; house rule #3).
- **Verified:** `just check` test gate green (nextest 1639 passed / 0 failed, 5 skipped; pytest 12);
  `cargo test -p mycelium-l1 -p mycelium-fmt -p mycelium-cli -p mycelium-lsp` 39 binaries / 0 failed;
  clippy + format + grammar (25 accept/29 reject) + the `myc-*` gates green. (Pre-existing `just
  check` gaps unrelated to M-818: `api` needs an absent nightly toolchain; `markdown` findings
  predate the branch; `doc-index` regenerated by the orchestrator below.)

### Added (2026-06-29: lwd — M-812-cont DN-54 `lower`/`derive` elaboration + KC-3 kernel-growth guard)

- **`derive` now elaborates to L0; the load-bearing DN-54 safety lands (M-812-cont).** `low` (M-812)
  shipped the `lower`/`derive` surface + structural checks; this lands the parts that only matter once
  `derive` actually elaborates:
  - **RHS elaboration to L0** — `elab::elaborate_lower_rule` reads `Env::lower_rules` and lowers a
    rule RHS through the **same code path** a hand-written nullary fn body takes (`Empirical`, via the
    §7 differential + the M-210 `check_core` validation). No longer a never-silent residual.
  - **§4.1 IL-grammar RHS type-check** (`check_lower_rule_rhs_type`) + **§4.6 purity** (`wild` refused
    in a rule RHS, even in `@std-sys`) + **§4.2 cross-rule acyclicity** (`check_lower_rule_acyclicity`
    — self- and mutual-cycles refused). All `Declared`, all never-silent (G2).
  - **§6 KC-3 kernel-growth guard — genuinely sound (`Proven`-by-construction, narrow checked
    sense).** The elaborator returns the closed `mycelium_core::Node` enum (the frozen L0 grammar), so
    a `lower` rule *cannot* construct a node outside the kernel set; the one surface-growth path (a
    host op) is closed by §4.6. Confirmed non-vacuously by `Node::is_aot_lowerable` over the frozen
    set — not a theatrical guard.
  - **§7 verification harness** (`tests/lower_derive.rs`, 5 tests) **replaces** the two `low`-era no-L0
    residual guard tests.
  - **Honest residual (flagged, never-silent):** DN-54 underdetermines the `derive`-**site**
    consumption/attachment model + parametric instantiation — the nullary/monomorphic elaboration
    landed; the consumption model is left for **maintainer ratification**. DN-54 stays **Accepted**
    with an append-only impl note (NOT `Enacted` — consumption model outstanding; VR-5 / house rule #3).
  - **Latent bug fixed:** lowercase `true`/`false` are not L1 names (the prelude `Bool` ctors are
    `True`/`False`); the new §4.1 type-check correctly rejects `lower X = true` — conformance fixture
    and tests corrected. Verified: `cargo test -p mycelium-l1` 671 passed / 0 failed; fmt+clippy clean.

### Added (2026-06-29: hof — M-704 dynamic higher-order functions / closures, RFC-0024 §4A)

- **Closures, environment capture, and dynamic fn-flow now elaborate + run three-way (M-704).** The
  RFC-0024 §5 residuals are closed via the full Reynolds construction (§4A): a `lambda` becomes a
  tagged closure struct (its captured free variables, deterministic first-occurrence order) and
  `apply` becomes a generated first-order dispatcher (`match` on the closure value). **KC-3 holds —
  no new L0 kernel node**: a closure is an ordinary `L1Value::Data` tag-sum, `apply` an ordinary
  `FnDecl` whose body is a `Match`, both lowered unchanged by the existing elaborator/registry
  (zero `mycelium-core` change). The per-arrow tag-sum + dispatcher are emitted once at
  monomorphization `finish()` time, after the whole-program closure set is known (no open-world
  fallback arm). Closure defunctionalization is `EXPLAIN`-able (`MonoSelections::closure_iter()` /
  `ClosureSpecialization`; house rule #2).
  - **Shapes running three-way (`Empirical`):** captureless lambda, single- and multi-capture,
    closure-capturing-closure, dynamic-fn-out-of-`match`, dynamic-fn-as-data-field, a capturing
    `map` combinator (the consuming proof), and named-fn-as-escaping-value (→ nullary closure ctor,
    §4A.4). The `Expr::Lambda` `Residual` is gone from checkty/mono (elab/eval keep it only as a
    defensive never-silent staging invariant).
  - **Honest residual (flagged, never-silent G2):** multi-argument lambdas / partial application are
    **tuple-gated** (RFC-0024 §4A.8 — v0 has no tuple/product type). A multi-param `lambda` is an
    explicit checker refusal, not a silent accept. Completing it needs a maintainer **tuple-type
    decision** — tracked forward on RFC-0024 §4A.8 / M-704.
  - RFC-0024 stays **Accepted** with an append-only "implemented (Rust-first), §5 residual resolved"
    note (NOT flipped to `Enacted` — full HOF incl. partial application is not yet landed; VR-5 /
    house rule #3). `mycelium.ebnf` lambda production + `.claude/memory/language-execution.md`
    updated.
- **Integration note (transparency):** the M-704 leaf was developed in an isolated worktree that
  branched from a stale base (CLAUDE.md mitigation #5/#7), so it lacked M-664's `Expr::Consume` /
  `Item::InherentImpl`; the orchestrator resolved the eval/grade/mono match-arm conflicts to the
  union and added `Consume` arms to the three new closure-traversal helpers. Verified green
  post-merge (206 lib + 107 check + closures + differential + all targets; fmt + clippy clean).

### Changed (2026-06-29: s10 — RFC-0020 carve-out enactment (M-707) + RFC-0030 grammar completeness (M-706))

- **RFC-0020 L2 carve-outs reconciled (M-707 done; RFC-0020 §10 enactment update, append-only).** The
  carve-outs were deferred *pending* RFC-0018/RFC-0019/RFC-0001-r5 — all since landed — so most are now
  **enacted**, the rest explicitly re-deferred (honest, grounded in `tests/check.rs::rfc0020_*`):
  - **§4.2 polymorphic instantiation — ENACTED:** a generic's type arguments are inferred from the
    call-site argument types (M-657 unification + M-673 monomorphization); an undetermined
    instantiation stays a never-silent error (G2). **R20-Q1 RESOLVED** (dictionary-free static
    monomorphization interface).
  - **R20-Q2 (grade inference) — RESOLVED:** a separate `grade.rs` pass (M-663); grades stay
    `Declared` where unproven (VR-5).
  - **R20-Q4 (mutual recursion) — ENACTED:** elaborates to `FixGroup` (M-343/M-391); the
    `MutualRecursionDeferred` refusal is retired.
  - **§4.5 derived forms (`derive`) — PARTIAL:** surface + structural checks landed (M-812/DN-54);
    RHS-elaboration + KC-3 guard deferred to **M-812-cont**.
  - **R20-Q3 (or-patterns) + R20-Q5 (list-literal/`for` bidirectional inference) — RE-DEFERRED**
    (RFC-0020 §9; no silent accept, G2). RFC-0020 stays **Accepted (scoped)** — full `Enacted` awaits
    §4.5 completion + an or-pattern decision.
- **RFC-0030 grammar completeness (M-706 done; RFC-0030 already Enacted).** The dependency gate
  (M-705/M-745/M-707) cleared; an EBNF audit against the landed parser closed three genuine gaps the
  grammar omitted but the parser accepts (never-silent defect, G2): the top-level **`impl_item`** (trait
  and inherent forms) was entirely missing from `item`; **`consume_expr`** was missing from `expr`; and
  **`lambda_expr`** was defined but unreferenced in `expr`. `just drift-check` + conformance green.
- **Verified:** `cargo test -p mycelium-l1` green; `just drift-check` green. Advances **E11-1**
  (surface-language completeness). `issues.yaml` (M-706, M-707 → done) reconciled.

### Added (2026-06-28: srf — M-664 `consume` expression + inherent `impl T { … }` blocks)

- **`consume <expr>` is now an ACTIVE surface expression (DN-03 §1 / LR-8; M-664 done).** It acquires
  and takes exclusive ownership of an affine `Substrate` value. The type rule is **checked** — the
  operand must have a `Substrate{tag}` type, and any other operand type (or a mismatched result
  context) is a never-silent `CheckError` (G2). Execution and single-use *affinity* are honestly
  **staged** (`Declared`): `Substrate` has no v0 value forms / no L0 representation lowering (it is an
  external-resource kind, not a repr type), so `consume` elaborates to a never-silent `Residual` —
  exactly like every other `Substrate` site (VR-5: the type discipline is checked, the runtime
  behavior deferred, no over-claim). v0 has no value-level affine-usage tracker (only pattern-binder
  linearity), so single-use is asserted by the construct, not yet enforced — recorded in `grade.rs`.
- **Inherent `impl T { fn … }` method blocks are now ACTIVE (DN-03 §1; M-664 done).** Distinct from a
  trait instance (`impl Trait for T`), an inherent block groups ordinary explicitly-typed functions
  with a type. It desugars at check time (Phase 0, alongside `object`) to its methods lifted verbatim
  as top-level `Item::Fn`s — the same model the `object` inherent-`fn` lowering uses, so all existing
  registration / checking / monomorphization / elaboration apply unchanged (**KC-3 — zero kernel
  growth**). A name collision with another top-level fn is caught by the existing duplicate-fn check
  (never silent, G2). The `for_ty` is organizational metadata in v0 (no qualified `T::m` call syntax
  yet). The top-level `impl` parser now disambiguates trait-instance vs inherent on the `for`/`{`
  follower; any other follower is an explicit parse error.
- **AST:** `Expr::Consume(Box<Expr>)` and `Item::InherentImpl(InherentImplDecl)` added; handled across
  `parse`/`checkty`/`elab`/`eval`/`mono`/`grade`/`totality`/`ambient` (compiler-enforced exhaustiveness).
- **Conformance:** accept fixture `25-consume-and-inherent-impl.myc` added; reject fixture renamed
  `18-consume-reserved-not-active.myc` → `18-consume-not-an-item.myc` (with `consume` now active, the
  remaining reject is *item-position* use — an expression is not a top-level item). `grammar/mycelium.ebnf`
  gains the previously-missing top-level `impl_item` (trait + inherent forms) and `consume_expr`.
- **Verified:** `cargo test -p mycelium-l1` green (203 lib + 105 check + conformance + all targets);
  `cargo fmt` + `cargo clippy -p mycelium-l1 -D warnings` clean. Advances **E7-1** (FR — surface
  completeness). `.claude/memory/lang-lexicon-syntax.md` + `issues.yaml` (M-664 → done) reconciled.

### Added (2026-06-28: prm wave — `fuse` (data) + `reclaim` EXECUTE three-way; DN-58 → Enacted, M-710/M-817 done)

- **`fuse` and `reclaim` now RUN end-to-end three-way (L1-eval ≡ L0-interp ≡ AOT, `Empirical`) — the
  r4v execution residual is closed (M-817, closes M-710; DN-58 §A/§B → Enacted).**
  - **`fuse` (repr):** the `Binary` `Fuse` semilattice meet executes via a new registered
    `fuse_join:binary` prim (`mycelium-interp` + the `mycelium-core` `PrimTable`) — bitwise-AND, the
    boolean-lattice greatest-lower-bound, carrying the canonical `Derived{op:"fuse_join"}` provenance
    (DN-58 §A.5 / RFC-0027 §10.6). The L1 evaluator, L0 interpreter, and AOT env-machine all dispatch
    the same prim. Non-`Binary` repr meets stay an honest never-silent residual (no committed meet —
    DN-58 §A.6 F-A3).
  - **`fuse` (data):** a user `Data` type with a `Fuse` instance — `fuse(a, b)` desugars at
    **monomorphization** (`mycelium-l1/src/mono.rs`) to the resolved `Fuse::join` call (the coherent
    instance recorded as an EXPLAIN selection — no black box), an ordinary inlined call that runs
    three-way (DN-58 §A.5). This is the user-merge case the `prm` kickoff targeted.
  - **`reclaim`:** the trusted base lowers `reclaim(policy) { body }` to its **sequential reference**
    (`Let{_ = policy, body}` — runs three-way, no new L0 node). The **real** RT7 supervision —
    bounded restart cascade + `SupervisionRecord` EXPLAIN trail — is a new runtime-tier driver
    `mycelium_mlir::run_reclaim` (+ `ReclaimRun`/`ReclaimError`, fed by the new
    `mycelium_l1::elaborate_reclaim`'s lazy policy/body nodes), dispatching to
    `mycelium-std-runtime::supervise_with_restart` and validated equal to the sequential reference on
    success — the same layering the `colony` executor (M-666) uses over unchanged per-task L0 terms.
  - **Mechanism note (transparency / VR-5):** this refines the M-817 brief's "register two prims"
    sketch. The trusted base (`mycelium-interp`/`-l1`) cannot depend on `mycelium-std-runtime` (cycle)
    and a bare `PrimFn` can resolve neither a user `join` nor a lazy supervised body, while DN-58 §A.5
    already specified a `join` *call* — so the data-fuse is an elaboration desugar and the reclaim
    supervision is a runtime-tier driver (only the `Binary` repr meet is a pure prim). Flagged to and
    approved by the maintainer before landing. **KC-3: no new L0 node.**
  - **Tests:** four three-way differential tests (`fuse_repr`, `fuse_data`,
    `reclaim_sequential_reference`, `reclaim_real_supervision_driver` — incl. bounded escalation with
    the EXPLAIN trace) + a `generic_corpus` data-fuse case; `PrimTable` parity updated. `just check`
    green. Honestly deferred (VR-5): non-`Binary` meets (F-A3), the policy-value → restart-bounds
    mapping (F-B2), and restart-recovers-a-transient-failure (needs effectful bodies).

### Changed (2026-06-28: `hrd` — DN-40 A1/A2/A3 doc-drift closure; code already landed, docs reconciled)

- **Doc reconciliation, no code change.** The DN-40 **A1** (CRITICAL parser type-subgrammar DoS),
  **A2** (HIGH pattern-subgrammar DoS), and **A3** (HIGH dep-hash parse-don't-validate) fixes that
  RFC-0028 §4.4 (signed off 2026-06-28) and the 2026-06-28 ratification batch below describe as
  **COMMISSIONED / "active gaps in the current codebase"** were found to have **already landed on
  `dev` 2026-06-26** (`4456bd3`; A3 `e7e705f`/`3f55eaa`) — recorded in §Security (2026-06-26: DN-40
  input-validation hardening) further down. Re-verified green this session: the `mycelium-l1`
  crash-refused depth regressions (`tests/check.rs::deeply_nested_{type_arrow,type_args,ctor_pattern}_is_refused_not_a_crash`
  and `parse::deep_operator_nesting_is_refused_not_crashed`; shared `MAX_EXPR_DEPTH = 256` budget) and
  the `mycelium-proj` typed-`ContentHash` manifest tests. Reconciled the lagging docs **append-only**
  (house rule #3): RFC-0028 status row + §4.4 status-note + §4.4.4 closure note, and the DN-40 status
  closure note — the historical commissioning entries are preserved as the as-signed-off record.
  **Tags:** the recursion bound is `Proven`-by-construction; the `256` limit value stays `Declared`
  (VR-5). `issues.yaml` needed no change — E14-1 and M-722 are already `status:done` and landed
  *after* A1/A2/A3, so the must-fix-before-E14-1 sequencing was met. `cargo-fuzz` is not installed in
  this environment, so the `fuzz_l1_parse` smoke skipped gracefully (local↔CI skip-on-absent policy);
  its no-panic invariant is exercised by the crash-refused tests above.

### Added (2026-06-28: ops kickoff — M-745 angle/shift operators wired in `mycelium-l1`)

- **M-745 done: the comparison and shift operators `<` `>` `<<` `>>` are now wired (RFC-0025 §4.1;
  RFC-0030 §4.3 gate met).** Frontend-only sugar desugaring to canonical word functions — **no
  L0/L1 kernel change (KC-3)**. The original type-arg disambiguation that made M-745 "needs-design"
  was dissolved upstream by RFC-0037 D1 (type arguments moved `<…>` → `[…]`), so `<`/`>` are
  operator-only and need no contextual lexing.
  - **Lexer** (`crates/mycelium-l1/src/lexer.rs`): `<<`/`>>` lex whole as `Tok::Shl`/`Tok::Shr`
    (`lex_langle`/`lex_rangle`); `<`/`>` stay `Tok::LAngle`/`Tok::RAngle`. No nested-generic `>>`
    hazard now that type args use `[…]`.
  - **Parser** (`parse.rs::infix_op`): `<`/`>` → `lt`/`gt` at bp 25 (§4.1 **Tier 8**, between `bor`
    and `eq`); `<<`/`>>` → `shl`/`shr` at bp 55 (**Tier 4**, between `add` and `band`). Precedence
    follows the **ratified §4.1 table (= Rust)**: shift tighter than the bitwise ops, comparison
    looser than them — **not** RFC-0037 §6's illustrative sketch, which inverted shift vs bitwise
    (flagged inconsistent — RFC-0025 changelog **FLAG-E**; the EBNF here is precedence-correct).
  - **Grammar** (`docs/spec/grammar/mycelium.ebnf`): `cmp_expr` (Tier 8) + `shift_expr` (Tier 4)
    productions added; the §4.3 deferral note retired. `just grammar-gen`/drift: operators are not
    keyword-derived, so the editor grammars are unchanged (drift green).
  - **Tests:** `src/tests/parse.rs` (desugar equivalence, the new-tier precedence, left-assoc for
    `<<`/`<`); `src/tests/lexer.rs` (`<<`/`>>` whole-token lexing); `accept/20-operator-syntax.myc`
    parse-oracle cases. **`cargo test -p mycelium-l1` green.**
  - `<=`/`>=` have **no glyph** (retired by RFC-0037 D1); word forms `lte`/`gte` are ordinary calls.
    The new word targets (`lt`/`gt`/`shl`/`shr`/`lte`/`gte`) parse + desugar but surface an explicit
    "unknown function/prim" refusal downstream until their prims land (M-809) — never silent (G2).
  - **RFC-0025 Accepted → ENACTED** (maintainer ratified in-session, 2026-06-28): with the wiring
    landed + green, the maintainer made the Accepted → Enacted move the RFC reserved for them ("do
    NOT self-Enact"; house rule #3 — stepped through Accepted, not skipped). Enacted covers the
    surface wiring + desugaring; word targets lacking a prim still refuse explicitly until M-809
    (G2). Docs reconciled: `issues.yaml` M-745 → done; RFC-0025 status + changelog (Enacted) +
    RFC-0030 §4.3 append-only notes; `.claude/memory/lang-lexicon-syntax.md` operator table.

### Added (2026-06-28: r4v + ADR-033 FLAG-1 integration wave — fuse/reclaim/tier L1 surface ACTIVE; ADR-033 full-sig encoding landed)

- **r4v wave (M-667 done; M-710 in-progress/partial): `fuse`, `reclaim`, `@tier` are now ACTIVE
  constructs in `crates/mycelium-l1` — no longer reserved-not-active (DN-58 §A/§B/§C).** Parse:
  `parse_fuse_expr` (`fuse(a, b)` — lawful binary merge over the `Fuse` semilattice, RFC-0008 RT6);
  `parse_reclaim_expr` (`reclaim(policy) { body }` — supervised scope, RFC-0008 RT7); `@tier(compiled
  | interpreted)` attribute path on `fn` items (per-definition execution-mode hint, RFC-0004
  `ExecutionMode`, NFR-7 non-semantic). AST: `Expr::Fuse { left, right }`, `Expr::Reclaim { policy,
  body }`, `FnDecl.tier: Option<TierMode>`. Checker: homogeneity + `Fuse`-instance check for `fuse`;
  policy-expression check for `reclaim`; attribute validation for `@tier` — **all never-silent (G2)**.
  Conformance: `accept/24-fuse-reclaim-tier.myc` added; `reject/12` updated (`mesh`/`graft`/`cyst`/
  `xloc`/`forage`/`backbone` remain reserved-not-active). `mycelium-fmt` gains `fuse`/`reclaim`/`@tier`
  display arms. **`cargo test -p mycelium-l1` 201 green.**

  Execution status (honest, VR-5): **`fuse` repr-type execution is Empirical** — three-way
  differential (`tests/differential.rs`) runs green for repr-typed operands. **RESIDUAL — never-silent
  (G2):** `reclaim` elab dispatches to a Residual stub (the `run_supervised` hook into
  `mycelium-std-runtime` is not yet wired); data-type `fuse` prim registration in the runtime
  registry is not yet wired. Follow-on: **M-817** (wire reclaim:supervised + fuse_join:data prims
  into the runtime registry). **M-710 remains in-progress/partial** (the end-to-end execution
  verification closes M-710).

- **ADR-033 FLAG-1 Path A — full function signature in `FieldSpec::Fn` dispatch hash (M-810,
  `mycelium-core`). Empirical via distinct-hash property test + no-match differential.**
  `FieldSpec::Fn { arity }` → `FieldSpec::Fn { sig: FnSig }` with a `FieldTyRef` per param + return
  type. `encode_decl` gains a full-signature encoding arm (`FIELD_FN` / `FN_SIG_*` / `FTR_*` tags —
  injective over typed structure). A `Fn { arity: 2, sig: (Binary{8},Binary{8})→Binary{8} }` field
  hashes **distinctly** from a `Fn { arity: 2, sig: (Binary{16},Binary{16})→Binary{16} }` field —
  closing the FLAG-1 type-confusion hole at the kernel level (silent G2 violation: two same-arity
  but different-type fn fields previously collided on content identity). `FieldTy::Fn` resolved
  analogue matches. `cargo test -p mycelium-core` 233+11 green. **Soundness tag: `Empirical`** (the
  injectivity property is trial-tested via distinct-hash property test + no-match differential; it
  is **not `Proven`** — unmechanized VR-5). FLAG-1 → **resolved (implemented)**. ADR-033 → propose
  **`Enacted`** (pending maintainer's final nod): the full-sig encoding landed and verified; the
  KC-3 growth is deliberate and bounded.

### Added (2026-06-28: obj+low integration wave — `object`/`via` + `lower`/`derive` surface, Rust-first)

- **DN-53 object-composition surface (M-811) — implemented Rust-first, pending ratification.** `object`
  and `via` are now **active** keywords in `crates/mycelium-l1`: `object Name[params] { Ctor(…); via N :
  Trait; impl …; fn … }` parses at item position and **desugars in the checker** to `type` + `impl`
  (+ generated `via`-forwarding impls) + `fn` — the honest non-OOP model (no `class`/mutable-self/
  inheritance/implicit-dynamic-dispatch). **Zero kernel growth (KC-3); `reveal`-able (DN-38 §5).** Phase-0
  structural desugar + Phase-0b `via` forwarding-impl generation (never-silent: unknown trait /
  out-of-range field index → `CheckError`, G2); full ambient resolution + surface re-render. Three-way
  Empirical differential (`observe(object) == observe(lower(object))`, `tests/object_desugar.rs`).
  Item-level `pub fn` inside the object body deferred (conservative). DN-53 → *Implemented (Rust-first),
  pending ratification*.
- **DN-54 user-extensible generative-lowering surface (M-812) — implemented Rust-first PARTIAL; the
  load-bearing KC-3 safety deferred (M-812-cont).** `lower` and `derive` are now **active** keywords:
  `lower Name[params]? = <rhs>` defines a generative-lowering rule, `derive Name for T` applies one
  (settling the `grow → derive` reconciliation, DN-38 §8.1 — `grow` now emits a teaching diagnostic
  pointing at `derive`). **Landed:** parse + AST (`Item::Lower`/`Item::Derive`) + the checker's
  **structural** validations — rule-name uniqueness, param-name uniqueness, `derive` name-resolution —
  all **never-silent** (G2); rule registered in `Env::lower_rules`. **Deferred (M-812-cont; held
  `Declared`, VR-5):** (1) **RHS elaboration to L0** — `crate::elab` does not yet read `Env::lower_rules`,
  so a `derive` currently emits **no L0 term** (an honest never-silent residual, *not* a fabricated
  accept — pinned by two integration guard tests); (2) the **§4.1 IL-grammar RHS type-check**; (3) the
  **§6 KC-3 kernel-growth guard**; (4) §4.2 cross-rule acyclicity; (5) the §7 verification harness.
  Guards (2)/(3) are meaningful only once (1) lands. DN-54 → *Implemented (Rust-first, surface +
  structural checks), KC-3 + IL-grammar + RHS-elaboration pending — pending ratification* (**NOT
  Enacted**).
- **Shared-surface reconciliation (integration).** `mycelium.ebnf` gains `object_item`/`lower_item`/
  `derive_item` productions; editor grammars regenerated (`just grammar-gen` — `derive`/`via` now lexed;
  drift gate green); `docs/api-index/` regenerated for the new AST/API; Glossary §2.10.2 + DN-02/DN-03
  lexicon flips (`object`/`via`/`lower`/`derive` → active); `issues.yaml` M-811 done, M-812 partial with
  honest note, **M-812-cont** added (todo) tracking the deferred safety. Conformance fixtures renumbered
  at integration to deconflict (object `accept/22`→`23`, `reject/26`→`28`; DN-54 keeps `accept/22-lower-derive`
  plus `reject/26`/`27`). `mycelium-l1` builds + tests green with **both** features present.

### Added (2026-06-28: maintainer ratification — 8 design vehicles → Accepted, decisions recorded)
- **Maintainer ratification batch (2026-06-28, in-session):** 8 design-vehicle drafts from the 2026-06-27 batch are now **Accepted**, with maintainer decisions incorporated (append-only; house rule #3; VR-5):
  - **RFC-0024** → **Accepted**: (a) currying for multi-arg arrows IN SCOPE for M-704; (b) still-generic-fn-as-arg IN SCOPE for M-704 (no longer deferred). §5 updated. → Enacted once M-704 lands.
  - **ADR-033** → **FLAG-1 RESOLVED (Path A selected)**: full function signature (params+return) encoded in `FieldSpec::Fn` dispatch hash; FLAG-1 moves to resolved-pending-implementation. → Enacted once full-sig encoding lands (sub-task M-810). Soundness tag stays `Declared` (VR-5).
  - **RFC-0036** → **Accepted**: single frozen L0 kernel (Option A); 9/10 nodes irreducibly primitive; `FixGroup` FLAG-B still open (derivability check before freeze); zero new VSA/HDC primitives. → Enacted once FLAG-B resolved + freeze mechanism implemented.
  - **RFC-0028** §4.4 → **signed off**: host-encoding validation bridge accepted; DN-40 A1 (CRITICAL), A2 (HIGH), A3 (HIGH) fixes COMMISSIONED for implementation (must land before E14-1).
  - **RFC-0025 + RFC-0030** → operator residue ratified; **M-745 wiring (lt/gt/shl/shr/lte/gte) COMMISSIONED** (M-809 grammar-supersession epic). RFC-0025 stays Accepted (Enacted after impl); RFC-0030 stays Enacted with commissioning note.
  - **DN-59** → **Accepted**: G3 reclamation strategy accepted (7 axes); **DN-62 (fuel-model research note) COMMISSIONED** (being drafted in parallel; FLAG-1 drop-latency question to be addressed there).
  - **DN-60** → **Accepted**: G6 effect-system Phase-2 direction (D1/D2/D3) accepted; **new RFC-0014 revision COMMISSIONED** (being drafted in parallel).
  - **DN-61** → **Part A (R1 scheduler normativity) Accepted**; **Part B (R2 distributed agenda) stays Draft** — open research agenda (R8-Q3/Q4, RFC-0027 OQ-2, xloc, fuse-merge). Split explicit in status field.

### Older entries

See `docs/archive/changelog/CHANGELOG-2026-06.md` for entries 2026-06-27 and earlier (verbatim).
