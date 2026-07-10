# Zero-Hand-Port Delta Ledger

| Field | Value |
|---|---|
| **Status** | **Draft — living planning document** (2026-07-10), in the register style of DN-99. Updated in place as phases close and numbers are re-measured; append rather than overwrite when a figure changes materially (VR-5 — keep the prior figure visible with a date). Recommends, does **not** ratify (house rule #3). |
| **Grounded in** | A five-layer analysis swarm run 2026-07-10: **L1** hand-expressibility ceiling (embedded in §1 below; no separate file), **L2** engines (`delta-L2-engines.md`), **L3** transpiler (`delta-L3-transpiler.md`), **L4/L5** idiom + structural design (`delta-L4L5-idiom-structural-DRAFT.md`, landing as **DN-109**), plus a **DX/QoL** track (`delta-DX-qol.md`). All under `docs/planning/zero-hand-port/`. |
| **Guarantee** | Every strategic claim below is tagged `Declared` or `Empirical` per its basis (VR-5); see §8 for the honesty caveats binding the whole ledger. |

> The measured remaining delta between today and 100% clean, idiomatic, Mycelium-optimal
> transpilation of Rust. Grounded in the five-layer analysis swarm above. This doc **recommends**,
> it does not ratify. To be landed as a planning doc; the L4/L5 design lands as **DN-109**.

## 1. The two numbers that frame everything

- **Hand-expressibility ceiling (L1): ~85%** — a human can already write an idiomatic Mycelium
  form for ~55 Rust constructs (EXPRESSIBLE) plus ~14 PARTIAL; only **3 genuine language build-gaps**
  remain (transcendental floats #42/M-1028, never-type #88/M-1030, async #56 — out of scope).
- **Measured auto-transpile floor (L3): `checked_fraction` ~7.8%** — 59/760 items, 17-target corpus,
  **file-gated** (one poison item zeros a whole file, a harsh lower bound). Pre-enb-wave.

**The delta between 85% and 7.8% IS the roadmap.** Hand-idiom-exists does not imply
transpiler-auto-emits-check-clean. The gap is: downstream language/kernel surface the transpiler
can target, a few transpiler-only rule classes, file-gating harshness, and an irreducible
semantic-judgment residual.

## 2. Composition of the delta (L3, empirically proven)

Of the ~812 measured gap instances: **~75-80% are DOWNSTREAM (language/kernel surface), ~20-25%
transpiler-only.** Ladder phases adding faithful *transpiler* rules moved `checked_fraction` by
**0**; every real gain came from kernel/language surface. **No fabrication remains (G2)** — the
transpiler is already honest; it is *waiting on the language.* Top gap classes: type-coverage 322
(40%), external-trait impls 119 (15%), imports/`use` 117 (14%), struct/record 80 (10%),
generic-bound 59 (7%), macro 64 (8%).

**Confirms the north star with a sharp edge: language/kernel surface-closure is the dominant lever,
not transpiler rules.**

## 3. Engine reality (L2)

- **Correctness base is COMPLETE:** the interpreter and the AOT *env-machine* (`aot::run`) are
  total over the v0 calculus. Transpile-correctness does NOT wait on any engine.
- **Two laggards, neither blocking transpile-correctness:** (a) **native-LLVM codegen** (perf
  path, refuses data/recursion nodes; M-373/M-601); (b) **semcore `.myc` mirror** (self-hosting,
  AST *shape* 100% mirrored, *semantic logic* partial; M-741). Both are parallel tracks.
- **Visitor-DRY meta-gap (CONFIRMED, UNTRACKED):** a new `Expr` variant touches **~13 sites** (8
  exhaustive walkers plus fmt plus ~5 mirror encoders); no trait `ExprVisitor`/fold exists. This
  taxes *every* future language-surface closure, a **force-multiplier to build first.**

## 4. Idiom + structural design (L4/L5)

- **Flat-emit data-loss bug (verify-first find):** the transpiler emits FLAT `<stem>.myc`,
  discards directory structure, last-writer-wins on stem collision. **No structural mapping and no
  provenance today** — L5 starts *worse than* structure-preserving 1:1. Fix first.
- **L4 idiom framework — three buckets by who can soundly decide:**
  - **Mechanical** (auto-emit v0): int-to-`Binary{N}`, Option/Result/`?`-to-match,
    struct/enum-to-data-decl, `&T`-erasure (landed), `unsafe`-to-`wild`.
  - **Heuristic** (rule plus EXPLAIN flag): Dense-selection, bounded-iterator-to-`for`.
  - **Judgment** (flag, never guess): `&mut`-to-value-semantics, Ternary/VSA selection,
    `as`-to-`swap`, `unwrap` retarget, closures.
  - **Conjunctive ratchet:** an auto-transform fires ONLY if it (1) preserves semantics under
    value-semantics, (2) inserts no `swap` (S1), (3) upgrades no guarantee tag (VR-5), (4) is
    EXPLAIN-recorded.
- **L5 structural remapping — structure-preserving 1:1 plus MANDATORY remap manifest.**
  Transpiler gains: a path model (`mod` tree to nodule dotted path, fixing the collision bug), a
  nodule-planner stage, a manifest emitter. Provenance artifact = committed item-granular
  **`remap.json` (plus rendered `REMAP.md`)**: each source-Rust-location maps to a target-nodule
  with `operation` (Keep/Consolidate/Split/Relocate/CrateToPhylum), `rationale`, `safety`,
  `api_surface_changed`, `identity_neutral`, plus an `idiom_choices` EXPLAIN trail. Mandatory from
  v0 even for pure 1:1. Satisfies "mapped, documented, explained (how/where/why)."

## 4.5. DX / QoL track (the "close out all QoL/DX gaps" directive)

Grounded inventory of the transpile-port-verify loop's developer-experience debt. **12 gaps: 0
blockers, 6 "slows-the-program", 6 "polish"** — the *blockers* are all downstream language surface
(owned by §2), so DX debt slows porting, it doesn't stop it. **Confirmed-GOOD (do not re-open):**
emit output IS deterministic; never-silent IS satisfied; the `// nodule:` header and `@summary` ARE
emitted; the checked/expressible split IS surfaced. (Refines §4: the flat-emit data-loss is
mitigated at the workflow layer — `regenerate.sh` uses per-crate out-dirs — but the CLI itself
still last-writer-wins within a run; fix in the tool, not just the driver.)

**Top DX closures, ranked (all NEW/untracked except LSP; re-mint free M-ids at filing — the
agent's proposed M-1023.. collide with landed ids, mitigation #1):**

1. **D1 — visitor/fold trait for `emit.rs` (force-multiplier, L).** SAME gap as §3's visitor-DRY
   meta-gap — *two independent analyses converge here.* The M-1006 ladder folds lessons into
   `emit.rs` every phase, so this tax is paid on the critical path repeatedly. **Build first.**
2. **D2/D3 — structured output mirror plus path-qualified names (M).** The flat-emit fix in the
   CLI (the maintainer's named gap) plus the L5 path-model foundation. Converges with §4.
3. **D3b — per-item `// src: file:line` breadcrumbs (S).** Biggest single hand-porter accelerator;
   the span data already exists in `Gap`.
4. **D6 — actionable `suggested_idiom` on each gap (M).** Turns never-silent into genuinely
   helpful.
5. **D8 — "closest-to-clean" investment ranking (S).** Tells the porter which file/item to fix
   first (vet is file-gated all-or-nothing; the diagnostic is already captured).
6. **D4 — `mycfmt` post-pass on emissions (S).** Canonical, readable drafts.
7. **D9 — LSP references/rename/code-action (L, partially tracked: E2-5 Full-LSP epic).**
8. **D5 — dry-run/summary mode (S).** Fast gap-profiling without writing artifacts.

**DX sequencing:** (1) **D1 first** (restructure `emit.rs` onto the visitor — D3b/D4/D2 all edit
`emit.rs` and should ride the cleaner surface); (2) emit-ergonomics cluster on top
(D3b plus D4 plus D2/D3, serial, one crate); (3) reporting cluster in parallel (D6 plus D8 plus
D5, disjoint files `gap.rs`/`vet.rs`/`batch.rs`); (4) toolchain polish last (D9 LSP and D10
grammar-drift — `grammar.js` has NO visibility node at all; `priv` is one symptom of that broader
drift). This DX sequencing IS Phase-1 plus Phase-4 of §6 — the force-multiplier (D1) and the
structured-output/provenance cluster (D2/D3/D3b plus the remap manifest) are the same front the
roadmap already leads with.

## 5. The ceiling fork (the honest limit of *mechanical* zero-hand-port)

`syn` alone cannot prove non-aliasing (`&mut`) or termination (unbounded `loop`/lazy iterators).
Without a **semantic/borrowck frontend** (rustc / rust-analyzer), those cases stay *permanent
human-judgment flags*. **So "100% mechanical" likely requires acquiring a borrowck frontend;
otherwise a bounded human-judgment residual is architectural, not a bug.** This is the pivotal
maintainer decision.

## 6. Sequenced roadmap (blast-radius-ranked)

**Phase 0 — re-baseline (do immediately, once builds settle):** fresh `checked_fraction`
post-enb-wave (the wave landed kernel surface; per the ladder pattern the floor is likely
materially greater than 7.8%). Measure, don't assume.

**Phase 0 partial re-measure (`Empirical`, 2026-07-10, `just transpile-vet` default 5-target set —
NOT the full 17-target corpus this §'s 7.8% figure covers, so not yet directly comparable; recorded
per the append-rather-than-overwrite discipline above):** `crates/mycelium-l1/src/eval.rs`
`checked_fraction` **7.1%** (3/42); `crates/mycelium-l1/src/fuse.rs` **0.0%** (0/10, hard parse
error — `[ParseError=1]`); `crates/mycelium-std-time/src` **18.9%** (7/37); `crates/mycelium-std-rand/src`
**0.0%** checked / 17.6% expressible (6/34 emitted, 0 clean — `[CheckError=1]`);
`crates/mycelium-std-cmp/src` **0.0%** checked / 12.6% expressible (14/111 emitted, 0 clean —
`[CheckError=1]`). Union over these 5 targets: **10/234 ≈ 4.3%** checked_fraction, 30/234 ≈ 12.8%
expressible_fraction. Directionally consistent with the ledger's own finding (§2: transpiler rules
alone move `checked_fraction` by ~0, kernel/language surface is the lever) — `std-time`'s 18.9% is
the highest of the five, plausibly reflecting the enb-wave's `?`/generic-slot/visibility-seal
closures reaching a stdlib crate that exercises them, though this is not isolated/attributed here.
**The full 17-target corpus re-measure this Phase's DoD calls for is still outstanding** — this is a
bounded default-set sample, not the complete Phase-0 close-out.

**Phase 1 — force-multiplier infrastructure (build first):**

- DRY `ExprVisitor` plus mirror-parity generator (kills the ~13-site tax) — *file it, untracked.*
- Transpiler path-model plus remap-manifest plus fix the flat-emit collision (L5 foundation and a
  real bug).

**Phase 2 — language/kernel surface closure (the dominant 75-80%), in blast-radius order:**

1. Cross-nodule symbol table / project-mode (L1 #1 plus L3's one big transpiler-only class,
   Import 117) — unblocks `mod`/`use`/qualified paths. M-1024 in-progress.
2. Record/struct surface (Struct 80 plus type-coverage subset).
3. External-trait impls (119).
4. Bounded-generic coverage (59).
5. Signed/platform-width (M-1029).
6. Transcendental floats (M-1028), format-string mini-language (M-1034), never-type (M-1030, if
   the `?`-coupling fork keeps it needed — DN-107 argues general-`?` does NOT need it).

**Phase 3 — transpiler-only:** expand-first macro pass (M-1032/M-875, 64 gaps).

**Phase 4 — idiom engine:** implement the Mechanical auto-fire bucket plus the EXPLAIN/flag
pipeline for Heuristic/Judgment (DN-109).

**Phase 5 — parallel tracks (never block transpile-correctness):** native-LLVM data/recursion
codegen; semcore mirror port (M-741).

**Phase 6 — rip-through:** module-to-nodule ports with the remap manifest documenting every
restructuring; re-measure `checked_fraction` each phase; graduate drafts differential-witnessed
(M-993).

## 7. Strategic forks for the maintainer

- **F1 (pivotal): borrowck frontend?** Acquire rustc/RA semantic analysis (leading to mechanical
  `&mut`/termination handling, a higher ceiling) vs accept a permanent human-judgment residual
  (KISS, M-991 frame).
- **F2:** restructure at L2 (surface) vs Core IR.
- **F3:** remap manifest as a new artifact vs extend existing `summary.json`/`union.gap.json`.
- **F4:** does non-1:1 restructuring belong in the transpiler at all, or a separate human-driven
  refactor pass (KISS / the M-991 gap-profiler boundary)?
- **F5:** confirm the v0 auto-fire set (Mechanical-only) or promote specific Heuristic rules.
- **F6:** DN-107's finding — decouple general-`?` (CPS lift) from the never-type (M-1030)?

## 8. Honesty caveats (VR-5)

- `checked_fraction` 7.8% is pre-enb-wave and file-gated; the Phase-0 re-measure is required
  before committing numeric targets.
- Two analysis agents read a stale main-checkout working tree (thought DN-99..106 absent); this
  ledger is grounded on dev's actual latest (DN-101..108 landed; next free DN-109).
- The 85% hand-expressibility figure is an *idiom-exists* ceiling, not an *auto-transpile*
  guarantee; §1's gap is the whole point.
- Frequency counts (812 gaps, class splits) are DN-34/DN-99 Empirical basis, not independently
  re-counted here.

## Changelog

- 2026-07-10 — initial Draft, synthesized from the five-layer analysis swarm; filed alongside
  DN-109 and the new tracked issues M-1041..M-1047.
