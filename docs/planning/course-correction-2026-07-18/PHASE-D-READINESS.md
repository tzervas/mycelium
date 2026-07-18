# Phase D — Transpile readiness measurement (2026-07-18)

| Field | Value |
|---|---|
| **Status** | `Declared` program artifact; every fraction/count below is `Empirical` (measured against the real `myc-check` oracle, this commit). Draft status — no agent-side `Accepted`. |
| **Owner** | Phase D worker (this leaf). Owns this file exclusively; does **not** edit `PROGRAM.md`/`CHANGELOG.md`/`Doc-Index.md`/`issues.yaml` (FLAGGED up — see §6). |
| **Basis** | `crates/mycelium-transpile` `--vet` loop (`/transpile-vet`) over the established M-1006 ladder pilot set (M-1001 default-5 + std-fs/std-io expansion), run against the real `myc-check` oracle built from this branch's tip. |
| **Program tie-in** | `../PROGRAM.md` Phase D ("Transpile readiness: closable gap classes iterated, M-1006 remeasure, per-component DoD per steer Phase 4"); steer `../design-steer-2026-07-17/PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §7 (Phase 4 DoD). |
| **Prior art** | `../gap-analysis-2026-07-16/M1006-remeasure-post-G-alpha-2026-07-17.md`, `.../M1006-remeasure-post-G-beta-2026-07-17.md` (the last two ladder remeasures — G-α/G-β transpiler advances). |

## 0. Verify-first correction (mitigation #14) — the task's own baseline framing

The kickoff for this leaf stated "the all-7 baseline was `checked_fraction` 28.7% `Empirical`
**BEFORE** the G-α/G-β advances landed." That is **not what the committed record shows**, and this
section corrects it plainly (VR-5 — surface disconfirming evidence, even against the framing that
launched the task):

- `M1006-remeasure-post-G-beta-2026-07-17.md` (tip `4acd3a20`, **after** both G-α *and* G-β landed)
  already reports all-7 `checked_fraction` **28.7% (98/342)** — i.e. 28.7% is the **post**-advance
  number, not a pre-advance baseline.
- The G-α-only combined measure (before G-β) was **25.7% (88/342)**; the true pre-G-α baseline
  (`M1006-remeasure-post-C3C4-2026-07-16.md` era) was materially different again.

So "the current number may be materially higher [than a pre-advance 28.7%]" was not a supportable
expectation going in — the committed 28.7% already *is* the current, post-advance number as of
2026-07-17. This section's job was to re-verify it against the actual tip, not assume it was stale
in a particular direction — and Phase C's AX-core waves (W-A..W-D) landed **substantial** changes to
`crates/mycelium-l1/src/checkty.rs` (+115/−… lines — the same file the DN-80 reject-ledger pin
tracks) plus new `ambient_policy.rs`/`grade_catalog.rs`/`legal_pair.rs`/`meet_boundary.rs`/
`regime.rs` modules, all landing *between* the post-G-β measurement and this leaf's start. Since
`myc-check` (the oracle `--vet` uses) is built from `mycelium-check` → `mycelium-l1`, **the oracle
itself could have shifted** even with `crates/mycelium-transpile` and the 7 target source files
completely unchanged. That is the genuine, well-grounded reason a fresh remeasure was warranted here
— not the "pre/post G-α/G-β" framing in the kickoff.

**Confirmed by diff before remeasuring:** `git diff --stat 4acd3a20..HEAD` shows `crates/mycelium-transpile/`
unchanged (one `cargo fmt` commit on a test file only) and all 7 pilot-target source files
(`crates/mycelium-l1/src/eval.rs`, `fuse.rs`, `crates/mycelium-std-{time,rand,cmp,fs,io}/src/*`)
byte-identical since `4acd3a20`; only `crates/mycelium-l1/src/{checkty,ambient_policy,grade_catalog,
legal_pair,meet_boundary,regime,ast,parse,lib}.rs` (the AX-core W-B/W-C/W-D checker surface) changed.

## 1. Fresh remeasure — tip `1f1b0a84` (before this leaf's own fix)

Same 7-target pilot set, same commands, as `M1006-remeasure-post-G-beta-2026-07-17.md`:

```
T="$CARGO_TARGET_DIR/debug/mycelium-transpile"; MYC_CHECK_CMD="$CARGO_TARGET_DIR/debug/myc-check"
$T --vet crates/mycelium-l1/src/eval.rs      out/eval
$T --vet crates/mycelium-l1/src/fuse.rs      out/fuse
$T --vet crates/mycelium-std-time/src        out/time
$T --vet crates/mycelium-std-rand/src        out/rand
$T --vet crates/mycelium-std-cmp/src         out/cmp
$T --vet crates/mycelium-std-fs/src          out/fs
$T --vet crates/mycelium-std-io/src          out/io
```

### Default M-1001 five-target set

| Target | non-test | checked | `checked_fraction` | expressible | File class |
|--------|---------:|--------:|--------------------:|------------:|------------|
| `eval.rs` | 42 | 9 | **21.4%** | 21.4% | Clean |
| `fuse.rs` | 12 | 0 | **0.0%** | 0.0% | Clean (zero-emit profile) |
| `std-time` | 37 | 17 | **45.9%** | 45.9% | Clean |
| `std-rand` | 34 | 6 | **17.6%** | 17.6% | Clean |
| `std-cmp` | 111 | 14 | **12.6%** | 12.6% | Clean |

**Union default-5: 46/236 → `checked_fraction` 19.5%** — byte-identical to every prior default-5
measurement back through post-C3C4 (2026-07-16).

### Expansion (std-fs / std-io)

| Target | non-test | checked | `checked_fraction` | expressible | File class |
|--------|---------:|--------:|--------------------:|------------:|------------|
| `std-fs` | 47 | 28 | **59.6%** | 59.6% | Clean×7 |
| `std-io` | 59 | 24 | **40.7%** | 40.7% | Clean×5 |

**Union all-7: 98/342 → `checked_fraction` 28.7%; `expressible_fraction` 28.7%** (checked ==
expressible: zero `CheckError` files, same as the post-G-β result). **Δ vs post-G-β `4acd3a20`:
0.0pp everywhere.** This is a genuine, verified flat result, not an assumption — the oracle's
AX-core additions (§0) turned out **not** to move this pilot set's pass/fail status for any item.

**Gap tally (union, 514 total records — identical count AND identical per-category breakdown to
post-G-β):**

| Count | Category | Count | Category |
|------:|----------|------:|----------|
| 85 | DeriveSatisfied (advisory, non-blocking) | 26 | MultiStmtBody |
| 68 | Other | 15 | TestItem (denominator-excluded) |
| 64 | MacroInvocation | 13 | GenericBound |
| 62 | DeriveAttr (advisory, non-blocking) | 10 | Struct / ModuleDecl (5 each) |
| 55 | Impl | 7 | Trait |
| 44 | NamedFieldDrop (advisory, non-blocking) | 5 | InnerAttr, MacroDef |
| 37 | Import (non-emission heat, not oracle first-poison) | 4/3/1 | PayloadVariant/AssocConst/ReservedWord |

**G1 gate re-check (steer `PROGRAM-SELFHOST-DECOMPOSE-2026-07-17.md` criterion):** default pilot +
std-fs/io **file Clean** — **PASS**, unchanged from post-G-β. Zero unknown first-poison classes
remain closable without a design gate on this pilot set.

## 2. Gap class closed — G-γ discard-statement / `let _` lowering

**Class:** `MultiStmtBody` — "function body has a semicolon-terminated (value-discarding) statement
expression before the tail" (a bare `g(x);` mid-body) and "`let` binding uses an unsupported
pattern" for `Pat::Wild` (`let _ = g(x);`). Both were a **blanket transpiler wall**, not a real
language-surface gap: the Mycelium grammar already accepts `let _ = <value> in <rest>` — `_` is an
ordinary `Ident` (`Ident ::= (Letter|'_') (Letter|[0-9]|'_')*`), `let_expr` takes any `Ident`, and
the checker has no unused-binding diagnostic. **Verified against the real `myc-check` oracle before
writing the fix** (two hand-built fixtures, both `ok`), matching the precedent style of
`d0384cc1`/`e3a16c42`/`f8c72895`.

**Change:** `crates/mycelium-transpile/src/emit.rs`, `emit_block_as_expr_inner` (the `Stmt::Expr(e,
_)` arm in the non-tail "lets" loop) now emits `let _ = <emit_expr(e)> in <rest>` instead of
refusing outright; `emit_local_binding` gained a `Pat::Wild(_) => "_".to_string()` arm alongside the
existing `Pat::Ident` case. Neither change fabricates anything: the discarded expression is still
routed through the ordinary `emit_expr`, so anything it can't faithfully lower still gaps exactly as
before (G2/VR-5) — this is a pure emission-**completeness** fix, not a new emission path. Faithfully
mirrors Rust's own semantics: a discarded value (even a fallible `Result`) is exactly as silently
dropped in the lowered `.myc` as it already was in the source Rust — the fix translates existing
source behavior, it does not introduce new silence.

**Differential witness:**
- `crates/mycelium-transpile/src/tests/emit.rs::discard_statement_and_let_wild_check_clean_live` —
  three fixtures (bare discard, `let _ =`, repeated `_`-shadowing) run through the real `myc-check`
  binary via `MycChecker`/`find_myc_check`; all assert `VetClass::Clean`. **`Empirical`** (this
  toolchain, this commit).
- `discard_statement_lowers_to_let_underscore_binding` /
  `let_wild_binding_lowers_to_let_underscore` — text-level pins of the emitted shape.
- `multi_stmt_body_reason_names_the_statement_kind` — updated (the discard case moved out of this
  test into the two new ones above; the remaining nested-item/macro-invocation cases are untouched
  and still gap as `MultiStmtBody`).
- `emit_hook_refactor_byte_identical_differential` — the DN-136 P1-a golden-snapshot differential.
  One case (`closure_purely_local_mutation_not_misclassified_as_closure_gap`) changed as an
  **intentional, reviewed** behavior change (exactly the case that doc's own comment sanctions
  re-snapshotting for): its `acc += x;` statement now reaches `emit_expr` and is refused there as an
  unsupported compound-assign operator (`Category::Other`) instead of hitting the old blanket
  `MultiStmtBody` wall — a **deeper, more precise** diagnostic for the same still-gapped outcome.
  Golden snapshot regenerated via the file's own sanctioned `--ignored` generator and diffed against
  the pre-fix committed snapshot: **exactly this one entry changed**, confirmed by `diff` before
  committing (no other case moved).
- `cargo test -p mycelium-transpile`: **277 passed, 1 ignored, 0 failed** (278 total, +3 new tests).
  `cargo fmt --check` / `cargo clippy --all-targets -D warnings` both clean.

**Effect on the 7-target pilot headline — reported honestly, not oversold:**

Re-running the identical 7-target vet loop **after** the fix: **byte-identical to §1** — 514 total
gap records (same count), 98/342 checked (28.7%, **0.0pp delta**). Diffing the gap JSON item-by-item
(file, line, item) confirms **zero items moved from gapped→checked-clean** on this specific pilot;
**5 items' gap *category* changed** (MultiStmtBody→Other) as the discard/`let _` statement inside
those bodies now reaches a **different**, deeper blocking construct in the *same* function (an
early-return `if` with no `else`, an assignment on a by-value builder's `self.<field>`, an
unresolved `for`-loop tuple pattern) — i.e. the fix is real and does traverse further into these
bodies, it just doesn't happen to be the *last* wall in any of these 7 files' currently-gapped
functions. This is the same "residual advancement, not lever-progress on this pilot" pattern G-α
Rank 1–2 established (`M1006-remeasure-post-G-alpha-2026-07-17.md` §"What G-α Rank 1+2 moved") —
recorded here rather than silently omitted because the headline didn't move (G2/VR-5).

**One item's gap record is materially sharper as a corpus-scale finding, not just a diagnostic
reword:** the fix moved the `if`-without-`else` (early-return) construct from being **hidden behind**
the old blanket MultiStmtBody wall to being **directly visible** as its own `Other`-category gap
(**12 occurrences** across the 7-target pilot after the fix, up from being indistinguishable inside
26 generic MultiStmtBody records before it). That is new, useful gap-profiling signal for a future
leaf (§4).

## 3. Residual gap classes — design-gated vs. closable-but-deferred

Per the mission's stopping rule ("stop when remaining classes are design-gated... ledger, never
force"), every other class with meaningful count was evaluated and is recorded here rather than
attempted in this bounded pass:

| Class (count, all-7) | Verdict | Why |
|---|---|---|
| **MacroInvocation + MacroDef (64+5=69)** | **Design-gated — M-875** | Largest single wall (std-cmp's macro-heavy body). Confirmed already design-gated by the post-G-β ranking; re-confirmed here (`grep` for M-875 status in `issues.yaml` — still open, not Accepted). Not attempted. |
| **`.contains()` (Other, string/byte substring search)** | **Design-gated** | `crates/mycelium-transpile/src/emit.rs`'s own comment: "no verified bare-call kernel prim mapping in this pipeline" — closing this needs a **new kernel primitive** (`mycelium-l1`/checkty surface), i.e. l1 semantics, explicitly out of this leaf's scope per the mission's own >l1-semantics ledger rule. Not attempted. |
| **`Option::ok_or` (method-call "no proven-emitted free-fn referent")** | **Design-gated — needs a stdlib addition, not a transpiler fix** | `lib/std/option.myc` has `is_some`/`is_none`/`unwrap_or`/`map`/`and_then`/`fold`/`or_else`/`flatten` but **no `ok_or`** (confirmed by reading the file) — the Mycelium *surface* this construct would need does not yet exist, so per this leaf's own "surface exists, transpiler doesn't emit it" scoping rule it does **not** qualify as transpiler-mechanical. Adding `fn ok_or[A, E](o: Option[A], err: E) => Result[A, E] = match o { Some(x) => Ok(x), None => Err(err) };` is a small, principled stdlib addition (mirrors the file's existing composition style exactly) but belongs to Phase E (`*-myc` delivery / stdlib graduation), not Phase D (transpiler-only). Flagged, not implemented here. |
| **Trait-impl of non-prelude traits — `Display`/`From`/`Error`/`Drop`/user traits (Impl, 55 total, 14+10+7+…)** | **Design-gated** | "no ambient trait definition and synthetic [impl] surface" — needs a language-level ambient-trait mechanism, not a per-call emitter fix. Cited in post-G-β ranking as Rank 4; unchanged. |
| **`DeriveSatisfied` (85) / `DeriveAttr` (62) / `NamedFieldDrop` (44)** | **Not gap-blocking — informational only** | Confirmed via `crates/mycelium-transpile/src/gap.rs`: `DeriveSatisfied` is explicitly `is_non_gap_advisory`; `DeriveAttr`/`NamedFieldDrop` coexist with successful emission of the same item (a dropped attribute / a positional-not-named-field record) — closing these would not move `checked_fraction`, they are provenance records, not refusals. No action needed. |
| **`if` without `else` (early-return desugar) — 12 occurrences, newly visible per §2** | **Closable-but-deferred (not design-gated, but out of this pass's budget)** | The Mycelium surface (`if_expr`) exists but *requires both arms* — lowering Rust's early-return idiom (`if cond { return x; } rest`) needs a real control-flow restructuring (the remaining statements become the implicit `else` arm, a small CPS-style transform), not a 3-line fix. Judged too large/risky for this bounded pass (estimated >100 LOC touching the body-emission core, moderate regression risk); ledgered for a dedicated future leaf, not attempted here. |
| **Tuple-`let` destructuring (`let (a, b) = e;`) — 5 occurrences (`unsupported pattern`; 4 in `std-rand` incl. one bundled inside an `Impl` sub-issue, 1 in `std-io`)** | **Closable-but-deferred** | `pattern` grammar supports tuple patterns only in match `arm`, not `let_expr` — closing this needs a synthetic-name + `match`-wrap desugar (moderate complexity, low count). Deferred, same reasoning as above. |
| **GenericBound (13), Struct/ModuleDecl (10), Trait (7), InnerAttr (5), PayloadVariant (4), AssocConst (3), ReservedWord (1)** | **Not evaluated in depth this pass** | Individually small counts; per the mission's "highest count × lowest risk first" ordering these rank below the classes above. No verdict beyond "not attempted" — flagged, not silently dropped. |
| **Import (37, non-emission heat)** | **Do not re-open** | Already-closed first-poison per G-α Rank 2; remaining records are legitimate cross-phylum-boundary co-include refusals, not a current oracle blocker. Unchanged from post-G-β ranking. |

## 4. Per-component readiness — steer §7 Phase-4 DoD (verbatim criteria)

> dual-report vet as default; `transpile_gap` worklist with `// src:` breadcrumbs + closest-to-clean
> ordering; zero P0 gap classes; three-axis labels on every report; no tag fabrication (T6 —
> draft-phylum quarantine for Declared floods)

| DoD item | This leaf's reading (stated, not assumed) | Status |
|---|---|---|
| Dual-report vet as default | `--vet --phylum` run for every crate target (fs/io/time/rand/cmp); eval/fuse are single-file l1 sources (`phylum: null` in `vet.json` is the correct, honest null — no phylum dimension to report, not a missing report) | **PASS** (all applicable targets) |
| `transpile_gap` worklist, `// src:` breadcrumbs, closest-to-clean ordering | Produced below (§5), ranked by `checked_fraction` descending | **PASS** (produced this leaf) |
| Zero P0 gap classes | Read as: no gap class represents an *unsound emission* (a fabricated body that silently passes, or a checker-poisoning bare name) — the measurable proxy is **zero `CheckError` files**. All 7 targets: **Clean** (0 `CheckError`), confirmed §1. No fabrication class found in this pass's review (§3 — every closable class was verified against the real oracle or the checker's own kernel-prim surface before any claim). | **PASS** on this pilot set (not a whole-corpus claim — see caveat below) |
| Three-axis labels (grade · mode · typing) on every report | This is an AX-core `Diag`-surface criterion (Phase C W-A: CertMode print + three-axis labels), not a `transpile-vet` gap-report field — `transpile_gap`/`vet.json` records category/file/line/reason, a different (also non-silent) axis set fit for its own purpose. **N/A to transpile-vet directly**; the underlying AX-core criterion is independently satisfied by the already-landed Phase C W-A work. Recorded here rather than silently marked pass. | **N/A (inherited-pass via Phase C)** |
| No tag fabrication (T6 — draft-phylum quarantine) | Confirmed: `gen/myc-drafts/` is never imported by `lib/` (`grep -rl myc-drafts lib/` finds only two **provenance comments**, `lib/std/numerics.myc`/`lib/std/io.myc`, citing the draft's path as history — not a live import); `scripts/checks/myc-dogfood.sh` never references `gen/myc-drafts`. Every emitted `.myc` in this leaf's output carries the standard "Declared, unvalidated" header; every fraction in this doc is tagged `Empirical` with its run basis. | **PASS** |

**Caveat on "zero P0 gap classes" (never-silent, not a broader claim):** this verdict is scoped to
the **7-target M-1006 ladder pilot set** measured here, not the whole Rust corpus (the Phase-2
whole-corpus profile — 337 files, 5472 items — was not re-run in this bounded pass; see §6 FLAG).

### Per-target readiness rows

| Target | `checked_fraction` | File class | P0 (CheckError) | Residual gap classes (this target) |
|---|---:|---|---|---|
| `std-fs` | **59.6%** | Clean×7 | 0 — PASS | Impl (trait-impls), NamedFieldDrop (advisory), builder by-value-`self` field assignment, if/else (2) |
| `std-time` | **45.9%** | Clean | 0 — PASS | `ok_or` (Option surface gap, 8), NamedFieldDrop, if/else (3) |
| `std-io` | **40.7%** | Clean×5 | 0 — PASS | `.contains()` (design-gated, 3), `map_err` chain-receiver miss, tuple-`let` (1), if/else (2) |
| `eval.rs` | **21.4%** | Clean | 0 — PASS | Import (11, cross-nodule `std::collections` — out-of-batch, not closable in-file), Impl (11, trait-impls), GenericBound (5) |
| `std-rand` | **17.6%** | Clean | 0 — PASS | tuple-`let` destructure (4, incl. one bundled `Impl` sub-issue), if/else (5), array-repeat `[x; N]` (no Mycelium equivalent) |
| `std-cmp` | **12.6%** | Clean | 0 — PASS | MacroInvocation wall (design-gated M-875) — the dominant blocker for this target |
| `fuse.rs` | **0.0%** | Clean (zero-emit) | 0 — PASS | Every top-level item gaps (tuple-pattern `for` loop, no emission at all yet) — furthest from clean |

## 5. `transpile_gap` worklist — closest-to-clean ordering, `// src:` breadcrumbs

Ranked by `checked_fraction` descending (per steer wording — "closest-to-clean" read as *fraction*
already expressed, not raw remaining-item count; a `Declared` judgment call, noted as such):

1. **`std-fs`** (59.6%, 28/47) — `// src:crates/mycelium-std-fs/src/{metadata.rs,options.rs}` are
   **already 100%** file-local (5/5, 3/3); next-closest: `// src:crates/mycelium-std-fs/src/guarantee_matrix.rs`
   (5/6, 83%) → `// src:crates/mycelium-std-fs/src/{substrate.rs,error.rs,path.rs}` (4 remaining
   each) → `// src:crates/mycelium-std-fs/src/lib.rs` (6 remaining, furthest in this crate).
2. **`std-time`** (45.9%, 17/37) — `// src:crates/mycelium-std-time/src/lib.rs` single-file;
   residual concentrated in `ok_or` (§3) and the `if`-without-`else` early-return idiom (§2/§3).
3. **`std-io`** (40.7%, 24/59) — `// src:crates/mycelium-std-io/src/{lib.rs,guarantee_matrix.rs}`
   closest (1 remaining each, 75–83%) → `// src:crates/mycelium-std-io/src/io.rs` (6 remaining,
   54%) → `// src:crates/mycelium-std-io/src/error.rs` (12 remaining, 37%) →
   `// src:crates/mycelium-std-io/src/serialize.rs` (15 remaining, 12% — furthest; `.contains()` +
   `map_err`-chain + `check_json_representable`'s tuple/if residuals concentrate here).
4. **`eval.rs`** (21.4%, 9/42) — `// src:crates/mycelium-l1/src/eval.rs`.
5. **`std-rand`** (17.6%, 6/34) — `// src:crates/mycelium-std-rand/src/lib.rs`.
6. **`std-cmp`** (12.6%, 14/111) — `// src:crates/mycelium-std-cmp/src/lib.rs`; MacroInvocation
   (M-875) is the dominant, design-gated blocker — not transpiler-closable until M-875 is Accepted.
7. **`fuse.rs`** (0.0%, 0/12) — `// src:crates/mycelium-l1/src/fuse.rs`; furthest from clean, zero
   emission at all on this pilot (unchanged since the earliest M-1006 measurements).

## 6. FLAGs (orch/program-owned — not edited here)

| Item | FLAG |
|---|---|
| **`PROGRAM.md` Phase D row + phase log** | Needs a new phase-log entry recording this leaf's remeasure + close (state: pending → in-progress/partial); this leaf does not edit `PROGRAM.md` (parent-owned). |
| **`CHANGELOG.md` / `Doc-Index.md`** | This leaf's `crates/mycelium-transpile` change (discard-statement/`let _` lowering) and this new doc are unlogged in the shared changelog/index — integrating parent applies once. |
| **`issues.yaml` M-1006** | `doc_refs` should gain this file's path; `body` could record the G-γ close (discard/`let _` lowering) alongside the G-α/G-β entries already there. Not edited here (shared file). |
| **`Option::ok_or` stdlib addition** | Flagged in §3 as a small, well-scoped Phase-E (not Phase-D) stdlib addition — `lib/std/option.myc` — not implemented in this transpiler-only pass. |
| **Whole-corpus (Phase-2, 337-file) remeasure** | Not re-run in this bounded pass — the "zero P0" / DoD verdicts above are scoped to the 7-target ladder pilot, not the full corpus. A future leaf should re-run the Phase-2 whole-corpus profile to confirm the discard/`let _` fix's effect at corpus scale (this leaf's synthetic-fixture + unit-test evidence proves the mechanism; it does not by itself bound the corpus-wide `checked_fraction` delta). |
| **`if`-without-`else` early-return desugar + tuple-`let` destructure** | Ledgered closable-but-deferred (§3) — real transpiler features, moderate complexity/risk, out of this pass's time-box. Good next-leaf candidates (12 and 4 occurrences respectively on this pilot alone). |
| **DN-80 reject-ledger pin** | Confirmed unaffected — this leaf touched only `crates/mycelium-transpile/src/{emit.rs,tests/emit.rs,tests/fixtures/emit_hook_golden.json}`, none of the pinned files (`checkty.rs`/`grade.rs`/`fuse.rs` in `mycelium-l1`). No pin update needed; verified via `git status --short`. |

## 7. Honesty ladder (per §1 house rules)

- Every fraction in §1/§2/§4/§5: **`Empirical`** — measured against the real, locally-built
  `myc-check` oracle on this branch's tip, `MYC_CHECK_CMD` discipline per `/transpile-vet`.
- The discard-statement/`let _` lowering itself: **`Empirical`** — the emitted `let _ = .. in ..`
  surface is proven `myc-check`-clean by a live-oracle unit test
  (`discard_statement_and_let_wild_check_clean_live`), not merely asserted.
- Every emitted `.myc` file (from this leaf's own `--vet` runs or the transpiler generally) stays
  **`Declared`** until a differential upgrades it — none of this leaf's work claims a port, only a
  measurement + an emitter-completeness fix.
- Ranking judgments (§3 design-gated vs. closable-but-deferred; §5 "closest-to-clean" ordering):
  **`Declared`** — reasoned from Empirical evidence (gap categories, counts, kernel-prim-mapping
  citations) but the *prioritization itself* is a judgment call, stated as such.
- No `Proven`/`Exact` claims anywhere in this document.
