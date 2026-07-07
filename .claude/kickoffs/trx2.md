# Kickoff `trx2` — Transpiler-accelerated porting + auto-doc-gen Mycelium doc ports

> **Base: `dev`.** Branch off the current `dev` tip, work in disjoint isolated worktrees, merge to
> `dev`, then promote `dev → integration → main` per the tiered workflow. Fire in a **fresh session**
> via `/kickoff trx2` (clean context budget).

## Mission

Stand up a **transpiler-accelerated porting loop** and **auto-doc-gen Mycelium doc ports** to
accelerate the self-hosting effort — turning hand-porting (brutal at `checkty`/`elab`/`eval`/`mono`
scale) into **transpile → `myc check` + differential vet → fix**. Three work-streams:

1. **Transpiler up-to-snuff** (harden `mycelium-transpile`).
2. **Mass `.myc` boilerplate** — rip through the targeted Rust to produce draft `.myc` *starting
   points* to work from (vetted, honestly `Declared`-until-checked).
3. **Auto-doc-gen Mycelium doc ports** — tooling that emits dogfooded Mycelium versions of the docs
   in their own directory, supporting the rest of the work.

## Grounding — this pulls forward planned work (read before scoping)

This is **not** greenfield, and must not duplicate/conflict with the roadmap:

- **`trx` / M-873** (archived) built the Rust→Mycelium transpiler PoC: best-effort `.myc` + a
  never-silent gap report; results in **DN-34 §8** (≈**12.4%** grand-union coverage, `Empirical`).
  Its output is graded **`Declared`** (heuristic `syn`→text, never validated by a Mycelium
  parser/typechecker). `crates/mycelium-transpile/`.
- **M-991** (minted 2026-07-06, `boot10`): "evaluate `mycelium-transpile` as a heavy-stage
  accelerator" — this kickoff **executes and extends** M-991.
- **`rwr`** (Phase-II, planned, `.claude/kickoffs/rwr.md`) owns the *full* progressive Mycelium
  rewrite incl. a **transpiler hardening ladder + port-wave manifests (D5/differential/D6)**, but is
  **gated on the public flip (`flp`)**. **`trx2` deliberately pulls an early, boot10-supporting slice
  of that ladder forward** — the maintainer's re-sequencing decision (2026-07-06). Keep `trx2` scoped
  to *supporting the port* (boilerplate + doc-gen tooling); the mass 1.0.0 rewrite waves stay `rwr`.
  Per **FLAG-V2** (rwr), compiler self-hosting stays `boot10`'s lane, an aspiration for `rwr`, not a
  lane — `trx2` accelerates `boot10`, it does not replace it.
- **`boot10` / M-740 / DN-26** is the self-hosting capstone `trx2` feeds. The remaining port surface
  is the **semcore heavy core** (`checkty`/`elab`/`eval`/`mono`/`fuse` → `M-993`, blocked) and the
  Stage-6 bootstrap (`M-742`). **`M-994`** (the whole-program L0-differential feasibility, a
  maintainer decision) still gates *vetting* the heavy-core port — `trx2` can generate the boilerplate
  regardless, but a full differential waits on M-994.
  *[Correction, 2026-07-06 launch — this bullet was authored before the same-day kernel-perf wave:
  **M-994 is RESOLVED/`done`** (both kernel walls down, M-986/M-987) and **M-993 is unblocked →
  `todo`**; found stale by the PR #1205 review. The paragraph above is kept as written (the
  stowed-time context), with this note as the current status.]*

## Work-streams (epics — propose ids at launch, do not pre-mint; mitigation #1)

### E-A — Transpiler up-to-snuff (`crates/mycelium-transpile/**`)
Harden the PoC from ~12.4% toward useful coverage on the port surface. Close the highest-value
`gap::GapReport` gaps (prioritize constructs the semcore/stdlib ports need), and **wire the
vet loop**: transpile a crate → run `myc check` on each emitted `.myc` → classify each failure back
into the gap report (so the transpiler's own accuracy is measured against the *real toolchain*, not
just "text emitted"). **DoD:** measurably higher `myc check`-clean emission on 2–3 representative
crates; the transpile→vet→classify loop scripted and documented; every narrowing `Declared`/flagged
(VR-5). Learning here feeds back into the transpiler (the point of M-991).

### E-B — Mass `.myc` boilerplate from the Rust (a NEW dedicated staging dir)
Run the hardened transpiler across the **targeted** Rust to produce draft `.myc` scaffolds in a
dedicated directory (e.g. `lib/compiler/draft/` or `gen/myc-drafts/` — decide at launch), each vetted
by `myc check` and, where a Rust oracle + differential exists, checked against it. **The output is a
STARTING POINT, not a finished port** — honestly `Declared` until a differential upgrades it to
`Empirical`; the `/myc-dogfood` gate (M-989) is the per-file witness. **DoD:** draft `.myc` scaffolds
for the named targets, each `myc check`-clean or gap-flagged (never a silent broken emit, G2), in the
staging dir, with a manifest mapping each draft → its Rust source + vet status.

### E-C — Auto-doc-gen Mycelium doc ports (`crates/mycelium-doc/**` · `tools/docgen/**` · new dir)
Extend the doc-gen tooling (`mycelium-doc` / `tools/docgen/code_index.py`) to emit **dogfooded
Mycelium versions of the docs in their own directory**. **⚑ SCOPE DECISION (confirm at launch):** what
"Mycelium ports of the docs" means is under-specified — options: (a) docs re-expressed as literate
`.myc` modules; (b) a doc-generator *authored in Mycelium* (dogfooding the toolchain); (c)
auto-generated reference docs *derived from* the `.myc` ports (the api-index analogue for `lib/`).
**DoD:** an auto-doc-gen path producing the doc ports into a named directory, deterministic +
regenerable (drift-gated like `docs/api-index/`), honestly labeled (`Empirical/Declared` heuristic;
source is ground truth).

## ⚑ Scope decisions to confirm at launch (maintainer)
1. **Rip-through breadth (E-B):** which Rust — the port surface only (semcore heavy core + remaining
   stdlib crates), or the whole `crates/mycelium-*` workspace? (Whole-workspace is enormous; default
   to the port surface unless told otherwise.)
2. **Staging-dir location + fate (E-B):** where the drafts live, and whether they graduate in place
   into `lib/compiler/`/`lib/std/` or stay a scratch staging tree.
3. **"Mycelium doc ports" semantics (E-C):** the (a)/(b)/(c) above.
4. **Sequencing vs `rwr`:** confirm pulling this transpiler slice forward is intended, and how `trx2`'s
   outputs reconcile with `rwr`'s M-947…M-957 port-wave manifests when Phase-II starts.

## House rules (every agent)
- Branch **off `dev`**; one **isolated worktree** per concurrent agent (mitigation #11); disjoint
  trees (E-A/E-B/E-C own different dirs) → collision-free by construction.
- **Transpiler output is `Declared` until a Mycelium parser/typechecker + differential vets it** —
  never present emitted `.myc` as a finished port; the gap report and `myc check` are the witnesses
  (G2/VR-5). No silent broken emits.
- Scoped PRs (DN-65, ~1–2k-LOC soft target), each `/pr-review`'d; leaf → `dev` → `integration` →
  `main` (squash only to `main`).
- **Doc-maintenance is part of the DoD** (`_doc-maintenance.md`): leave `issues.yaml`, `CHANGELOG`,
  `Doc-Index`, grammar, `docs/api-index/` current.
- The integrator owns the shared collision surface (workspace `Cargo.toml`, `CHANGELOG`, `Doc-Index`,
  `issues.yaml`+`idmap`, `docs/api-index/`); leaves FLAG up, never edit these.

## First steps
1. Sync off the latest `dev` tip; `just setup` (Rust) + `uv sync` (Python doc-gen).
2. Confirm the four scope decisions above with the maintainer.
3. E-A first (the transpiler must be up-to-snuff before E-B's mass run is worth doing); E-C parallel.

## Launch record (2026-07-06 — appended at kickoff)

Fired via `/kickoff trx2` on head branch `claude/trx2-kickoff-rxyzd3` (== the `dev` tip `7dde593`,
post kernel-perf wave). The interactive scope-confirm failed mid-stream (tool error) and the
maintainer directed "continue" — the four decisions below are **orchestrator-resolved on the
kickoff's own recorded defaults**, graded `Declared`, and reversible by the maintainer:

1. **E-B breadth = port surface only** — the kickoff's stated default. Concretely: the semcore
   heavy core (`checkty` 7356 · `mono` 3219 · `elab` 2294 · `eval` 2263 · `fuse` 292 LOC) + the
   12 stdlib crates with no `.myc` twin (conformance, content, dense, fs, io, numerics, rand,
   runtime, sys, sys-host, time, vsa).
2. **Staging dir = `gen/myc-drafts/`** (scratch tree; drafts graduate into `lib/` only when
   hand-vetted during M-993) — keeps `Declared` drafts out of the `/myc-dogfood`-gated `lib/` tree.
3. **E-C = interpretation (c)** — auto-generated reference docs derived from the `.myc` sources
   (`docs/lib-index/`, the `docs/api-index/` analogue for `lib/`); (c) is the only reading
   satisfying this kickoff's own DoD ("deterministic + regenerable, drift-gated"). (b) — a
   generator authored in Mycelium — is noted as a candidate follow-up dogfooding wave.
4. **Sequencing vs `rwr` = confirmed as scoped** — per the maintainer's dated re-sequencing
   decision recorded above (2026-07-06); trx2's gap/vet/manifest outputs become inputs to `rwr`'s
   M-947…M-957 port-wave manifests at Phase-II.

**Minted** (slots verified free, mitigation #1): epics **E32-1** (E-A) · **E33-1** (E-B) ·
**E34-1** (E-C); tasks **M-1000/M-1001** (vet loop · gap-class closure), **M-1002/M-1003**
(staging+manifest · rip-through), **M-1004/M-1005** (lib-index extraction · drift gate).
M-991 → `in-progress` (discharged by E32-1's DN-34 results record). Swarm mode: maintainer
directive at launch — Opus/Sonnet implementation agents, orchestrator coordinates.

**Maintainer confirmation (2026-07-06, same day — amends resolution 1).** The maintainer
confirmed decisions 2–4 as resolved above, and **decided** (no longer orchestrator-`Declared`)
E-B breadth as: **port surface first** (this wave, unchanged), **then — once refined — expand to
the rest of the Rust corpus of the Mycelium language codebase in controlled phases**: automated
porting + patching to close gaps, folding each phase's lessons back into the transpiler until it
is a highly polished solution. Minted **M-1006** (under E33-1) as the phased-ladder umbrella;
per-phase target sets are minted per wave (mitigation #1), and the ladder's outputs reconcile
with `rwr`'s M-947…M-957 port-wave manifests at Phase-II (trx2 phases become their inputs).

## Session-2 continuation (2026-07-07) — self-hosting decisions landed, M-1012 in flight

The wave shifted from transpiler/tero into the **compiler self-hosting** core after the maintainer
made the governing decisions. **This section is the durable handoff — read it first.**

### Landed on `dev` this session (PRs #1237–#1247, all merged)
- **M-1006 §8.11** (PR #1242) — transpiler shared-ref erasure `&T→T`; +2 expressible, checked flat.
  Lesson: emission is near its ceiling on the fixed corpus; the next lever is **cross-nodule
  project-mode vetting** (so referents resolve) or E18-1, NOT more emission arms.
- **tero M-1016** (PR #1241) — query engine + mandatory provenance (uncited answer ⇒ typed Refusal;
  EXPLAIN traces; Empirical latency ~630µs/query over 5141 rows).
- **Decision records:** dossier (PR #1240); **DN-26 §10 Option A** + **ADR-042** (PR #1244);
  **ADR-043** (PR #1247); **DN-88** (PR #1243); **DN-89** (PR #1245).

### The migration lifecycle — now recorded, honor it end-to-end
**ADR-042** (freeze new Rust; rewrite EVERYTHING incl. kernel + toolchain + **codegen backend** to
`.myc`; zero foreign first-party langs by the DN-88 decomposition gate; `wild` only at the
irreducible OS/HW ABI seam; DN-39 boundary unchanged — only the impl *language* changes) → **M-989
myc-dogfood dual witness** (the "proven" gate: Rust differential + native `myc check`/`mycfmt`/
`myc-lint`) → **ADR-043** (retire-when-proven → archive Rust to a protected legacy branch, never
lost → remove from active tree → incremental housekeep → pure-Mycelium) → **DN-88** (decompose to
component repos + GHCR spores + managerial re-export phyla) → **DN-89** (native AI/ML corpus:
transpile+patch / FFI-bind / clean-room-reverse-engineer as a *convergence path* to full-native;
ports leverage+improve, honesty-tagged). `mycelium-tero` is the sanctioned last-Rust PoC
(M-1015→M-1018) then rewrites to `.myc` (M-1019). Open design gates: ADR-042 §6 kernel
bootstrap/trust DN + the native-codegen-backend DN (both `needs-design`, maintainer to design).

### ✅ STEP 1 DONE — M-1012 merged (PR #1246, `3e92db8f`)
First heavy-core increment: elab.rs's pure L0 lowering helpers ported into `lib/compiler/semcore.myc`
under **Option A** (in-language mirror ADTs + `scalar_kind`/`sparsity_class`/`lit_value`/`type_repr`/
`field_spec`/`ty_to_repr`/`ty_to_field_ty_ref`/`policy_name` preimage). **Wild-free.** Honest deferrals
(FLAG-semcore-25/27/29). All `/pr-review` (aef3c202) fixes landed incl. the **non-vacuity convention**
for all 8 `.myc` comparators (the template for M-1013). Verified on both witnesses: `stage5_elab` 11/11,
the full `mycelium-l1` suite green, and native `myc check lib/compiler/semcore.myc` ok. This sets the
M-1013 pattern; the fresh session starts at STEP 2.

### ⇒ FRESH-SESSION STEP 2 — INCORPORATE MARSHALLING (maintainer directive, 2026-07-07)
The M-1012 differential compares two `.myc`-side mirror values with hand-written `.myc` structural
equality (bounded by the new mandatory non-vacuity discipline). The maintainer directs incorporating
**harness marshalling** as the self-hosting differential method: **decode the `.myc` mirror output
into the real `mycelium_core` type (Rust, harness-side) and compare with Rust's trusted derived `==`**
(or compare `mycelium_core::Value` content-hashes) — eliminating the hand-written-`.myc`-eq trust
surface. Tasks: (a) retrofit `compiler_stage5_elab.rs` to marshalling; (b) make marshalling the
differential method for **all** M-1013 increments; (c) update DN-26 §10.1 from "documented option" to
"adopted method." Do this EARLY — it is the trust foundation for the ~11 M-1013 increments.

### ⇒ FRESH-SESSION STEP 3 — M-1013 heavy-core increments 8→14
Per dossier §6.3 (~11–13 scoped PRs, sequential where they share `semcore.myc` regions; each verifies
BOTH witnesses — marshalling differential + native `myc check`; leaf→dev PR each with a `/pr-review`):
8 registration/resolution/`Env`; 9 checker `Cx` (~3.2k, sub-split 2–3 PRs; folds in the deferred
`infer` core = M-1011's `infer_type` residual, FLAG-semcore-20); 10 elab core (consumes M-1012's L0
mirror); 11 mono core (+ FLAG-semcore-17 `mangle_ty_in_ty`/`item_key`); **12 eval — may need the AOT
leg, NOT the interpreter** (M-986 TCO / M-987 ~n³, dossier §7 FLAG-4); 13 fuse; 14 whole-program L0
differential + mutants. Also worth doing as the compiler phylum matures: wire `lib/compiler/` as a
`mycelium-proj.toml` project root so the standard `myc-check`/`myc-fmt` gates cover it first-class
(not just myc-dogfood's per-file walk).

### ⇒ FRESH-SESSION STEP 4 — dev→integration promotion (batch reconciliation)
Delegate to an Opus integrator in an isolated worktree. Batch on dev: dossier, M-1006/§8.11, DN-88,
M-1016, ADR-042/DN-26§10, DN-89, ADR-043, M-1012. Reconcile: status flips (**M-1007→done** [DoD says
DONE, PR #1231], **M-1011** partial-done note [infer_type = increment 9], **M-1012→done**,
**M-1016→done**, M-1006 stays in-progress); **mint** E40-1/M-1020 (decomposition guide, gated),
E41-1/M-1021 (AI/ML, gated), an ADR-042 policy-tracker, + follow-on DN placeholders (kernel
bootstrap-trust, native-codegen-backend); **Doc-Index rows for DN-88/DN-89/ADR-042/ADR-043 (ALL
currently MISSING)** + adr/README rows for ADR-042/043; CHANGELOG for all; **api-index regen**
(mycelium-tero's new public surface); tero/lib/doc index regen; **fix pre-existing `CHANGELOG.md:28`
MD012 + `DN-88.md:33` MD018**; full `just check` green. Flow: reconcile→dev PR, then dev→integration PR.
`integration→main` squash stays the terminal MAINTAINER checkpoint — do NOT auto-squash to main.

### ⇒ PARALLELIZE ACROSS SESSIONS (maintainer directive) — disk-isolate disjoint work
**Disjoint work that does not depend on each other should be kicked off in SEPARATE parallel
sessions, not just parallel agents in one session** — each session gets its own environment/disk, so
they cannot collide on the shared ~41G ceiling (the failure that broke builds this session). This is
the **Wave-N multi-session workflow** (CLAUDE.md §Wave-N): partition by disjoint file ownership,
one protected head branch each, cross-session continuity via `issues.yaml` `depends_on` + body notes
(never by touching another session's files); each head self-integrates, then a final integration
octopus-merges the heads and squash-PRs to `main`. Suggested partition of the remaining work:
- **Session A — semcore self-hosting (critical path, SEQUENTIAL, one session):** marshalling (STEP 2)
  → M-1013 increments 8→14. These share `lib/compiler/semcore.myc` + the differential harness and
  build on each other, so they are NOT disjoint — keep them in one session.
- **Session B — tero productionization (DISJOINT: `crates/mycelium-tero/`):** M-1017 (MCP + HTTP
  fronts + skills) → M-1018 (VSA Layer-2 + the Empirical eval gate). Independent of semcore.
- **Session C — transpiler ladder next phase (DISJOINT: `crates/mycelium-transpile/` + `gen/myc-drafts/`):**
  M-1006's project-mode/cross-nodule vetting lever (per §8.11 — the real way to move `checked_fraction`).
- **Integration session:** after A/B/C land on their heads, one integrator runs the dev→integration
  promotion (STEP 4) reconciling the shared files once.
Within each session, still use isolated worktrees per concurrent agent — but keep the *count* modest
and prune `target/` dirs, since one env's disk is the bound.

### Ops lessons (this session — bake into the fresh run)
- **DISK:** the shared root fs has a **~41G effective ceiling**; per-worktree cargo `target/` (1–3G
  each) + the main `target/` (~15G) fill it fast, breaking BOTH git (`index.lock` ENOSPC) and cargo.
  **Prune completed-agent worktrees + their `target/` dirs periodically** (`git worktree prune`;
  `rm -rf .claude/worktrees/*/target`; clear the main `target/` between waves). Consider one shared
  `CARGO_TARGET_DIR`. Multiple sessions share the volume — a sibling can refill it.
- **Shared-tree contamination (mitigation #11):** background agents (even `isolation:"worktree"`)
  sometimes run git in the shared main tree. Keep the orchestrator's main tree a **clean pointer**
  (a fresh branch off `origin/dev`, never a leaf branch); `git reset --hard` / re-checkout if
  contaminated. `dev` may be checked out in a sibling session's worktree (can't check it out twice).
- **Incomplete agent reports:** an agent that spawns a background verify and hits its turn boundary
  reports mid-action ("waiting for the monitor"). **Always verify the actual pushed/committed state**
  (mitigation #7/#9) — recover uncommitted work from the worktree, commit + push it yourself.

### Completion driver (already armed)
Routine `trig_01EXyj3Q` "myc-port-drive" (cron `30 */3 * * *`, fresh session per fire, push
notifications) — executes the **M-741** ratification IFF it independently verifies all 3 DoD criteria
with checked `Empirical` evidence (else flags on issue #444); never auto-declares 1.0.0. M-741 is
maintainer-pre-authorized (verification-gated). The fresh session's swarm work runs alongside it.

## Session-C UPDATE (2026-07-07) — M-1006 ladder to `main` + kernel prim-gap closure wave

Session C (transpiler ladder, `crates/mycelium-transpile/` + `gen/myc-drafts/` + DN-34) ran to
completion **and pivoted into a kernel prim-gap closure** once the maintainer **unfroze the kernel**.
Full durable record: **DN-34 §8.12–§8.16** (on `main`/`dev`). This section is the current status + the
remaining-task list for a fresh continuation.

### ✅ Landed (all merged)
- **M-1006 transpiler ladder → `main`.** §8.12 positional named-field emission + `myc check --phylum`
  cross-nodule mode (hotfix to all three tiers); §8.13 field-projection/struct-literal desugaring;
  **§8.14 String/str/&str → `Bytes`** (RFC-0033 §3.2) — the **largest single-lever gain of the ladder**
  (`checked_fraction` 4.61% → **5.79%**, +9). Landed to `main` as the sole scoped `integration→main`
  delta (PR #1267) after the big release squash; also on `dev`+`integration`.
- **§8.15 prim-gap audit correction** (house rule #4): my §8.13/§8.14 wrongly recorded the `Binary{N}`
  bitwise ops as *missing* — they **exist** as `bit.and/or/xor/not` (surface `and/or/xor/not`), and
  `==`/`<`→`Binary{1}` is ratified design (RFC-0032 D1). Live prim count **Π = 59** (DN-56/DN-76 "38"
  stale). **rotate IS expressible** (`or(shl_u,shr_u)`) — §8.13 Lever-2's "impossible" was wrong.
- **Kernel prim-gap closure wave (kernel UNFROZEN — §8.14 correction; Π 59 → 66):**
  **CU-1** `bit.mul` unsigned multiply (#1273) · **CU-2** `flt.is_nan/is_finite/is_infinite`
  (ADR-040 §2.5 mandate, #1274) · **CU-6** `bit.popcount/clz/ctz` (#1275) · **CU-4** `ne/gt/cmp_s/le_s/
  ge_s` + the CU-6 `std.math` surface `bmul/bpopcount/bclz/bctz` (#1291). Each carries never-silent
  semantics with property and three-way (L1/L0/AOT) tests; the Π table, `checkty` surface, and
  `prim_table` were updated in lockstep.
- **Agent guidance:** DN-34 §8.16 wave record (#1293); **tero guidance** added to CLAUDE.md +
  `.claude/agent-context.md` + `.claude/memory/README.md` (#1294).

### ⇒ REMAINING TASKS (the updated kickoff worklist — ruled *implement-now* unless noted)
Decision rulings already made (all the project-optimal option — performant · memory-safe · KC-3 ·
never-silent); the approach for each is in **DN-34 §8.16**. Prim-add pattern is in the working notes
`…/scratchpad/trx2-session-c-notes.md` (codec → prim → registry → Π table + 2 count asserts → checkty
family/sig/kernel_name → prim_table → enablement three-way).

1. **CU-3 — float↔int never-silent conversions.** Target-width prims (the `bit.width_cast`/DN-41
   model): `flt→bin` (refuse NaN/±inf/out-of-range) + checked-exact `bin→flt` (err `|n|>2^53`); the
   **lossy** `bin→flt` rounding is a **reified swap** carrying its bound (ADR-040 §2.4/§5, NOT a prim).
2. **CU-5 — executable `wrapping` construct.** M-791 landed the meta/mode axis (`mycelium-core/src/
   meta.rs` + `src/tests/wrapping.rs`) but no runtime path. Wire the named construct → modular
   (never-refusing, `Declared`-tagged) eval over `bin.add/sub/mul`. RFC-0034 §10; no new `wrapping_*`
   prims. (Grep `wrapping` in l1 `elab`/`checkty` for the construct's parse/elaborate handling first.)
3. **CU-7 — arbitrary-width ternary.** `mycelium_core::ternary::BigTernary` (M-756) exists but is
   unsurfaced (runnable ternary is fixed-width `trit.*`, ~40-trit cap). Surface the growable arithmetic
   path RFC-0033 §4.2.2 mandates (ADR-029 Accepted); coordinate the growable-`Repr::Ternary` payload
   with the E20-1 content-address settlement.
4. **Transpiler operator/comparator emission** (`crates/mycelium-transpile/`). Emit `and/or/xor` (not
   the dead `band/bor`) for `&`/`|`/`^`, and the CU-4 comparators, when operands are known `Binary{N}`
   — the **operand-type inference** §8.13's D3 named (thread param/`self` widths through `emit.rs`).
   Then regenerate `gen/myc-drafts/` + measure the `checked_fraction` lift.
5. **Spec-doc sync (doc-currency follow-up).** Document the new `std.cmp` surface (`ne/gt/cmp_s/le_s/
   ge_s`) in `docs/spec/stdlib/cmp.md` and the `std.math` surface (`bmul/bpopcount/bclz/bctz`) in
   `docs/spec/stdlib/math.md`; add a top-level `CHANGELOG.md` entry for the prim-gap wave (integrator).
6. **Deferred to design work (ruling: defer — no half-measures):** **CU-6 rotate/reverse**
   (`std.math` FLAG-math-3 — needs a `bit.rotl`/`bit.rotr` prim or width-reflection; the naive
   `or(shl_u,shr_u)` mis-handles `n=0`, a full-width `shr` refuses); **CU-8 atomics** (`fetch_add` — a
   memory-model RFC, DN-32 §7/RFC-0027 §12); **CU-9 Dense dtype/quant** (RFC-0033 §4.3.2 — rides the
   E20-1 content-address rehash, ADR-030; the maintainer's `vsa_checks` branch has the desktop
   durability numbers to ground it). Mint tracked issues; do not scope partial stubs.

## Session-A continuation (2026-07-07) — M-1013 increment-8 registration family COMPLETE on `dev`

Session A (semcore self-hosting critical path — `lib/compiler/semcore.myc` + the `mycelium-l1`
differential harness) continued STEP 2 → STEP 3. **This is the durable Session-A handoff — read it
first, then the RESUME POINT below.** Working branch this session: `claude/trx2-inc-ladder`
(a clean pointer off `dev`; the main tree, never a leaf branch).

### ✅ Landed on `dev` this session (all leaf→dev PRs, `--no-ff`, both witnesses green, `/pr-review`'d)
- **STEP 2 marshalling** was landed in the PRIOR Session-A window (PR #1253) and promoted to `main`
  in that window's release (#1265). Marshalling is the ADOPTED Stage-5 differential method
  (DN-26 §10.2): decode the `.myc` port's `L1Value` mirror into the real `mycelium_core` type and
  compare against the live Rust oracle. **This is the trust foundation for every increment below.**
- **inc-8 PR-2b — full `collect_tuple_arities` walk (PR #1272, merge `bc4d59f4`).** All legs (type-decl
  ctor fields, fn/trait/impl signatures, `match` patterns, fn-body expressions). **Closed
  FLAG-semcore-30** (the PR-2 trimmed-walk deferral). Un-collapsed the `Item` mirror to the full
  8-variant vocab. Bundled a **justfile de-confliction fix** (removed duplicate `tero-index`/
  `tero-index-gen` recipes a prior sync-down auto-merge had introduced, which made `just` abort).
- **inc-8 `register_traits` (PR #1292, merge `53f46ab1`).** Two-pass trait registry (dup type-param/
  name/method refusals + per-method `check_sig_resolves` + forward-ref-tolerant bound validation).
  Added the **surface `FnSig` decoder** family (distinct from the elab harness's kernel-`KFS` decoder).
  13 live-differential fixtures.
- **inc-8 `register_instances` (PR #1295, merge `88efee0e`).** Full 8-check impl/instance registration +
  coherence (unknown-trait, concrete resolve, arity, `type_head`, phylum-wide pub-blind orphan rule,
  global uniqueness, exact method-set match). Added `CoherenceView`/`InstanceInfo` mirrors (reusing the
  existing `Ty` marshalling). 13 live fixtures (incl. dup-method). **Resolved FLAG-semcore-33** by
  widening `checkty::CoherenceView.types` to `pub(crate)` (the white-box in-crate-test pattern, same as
  PR-1's `resolve_ctors` widening), upgrading the Data-orphan arm from port-only to a full live
  differential.

**Net:** the **trait/instance registration family is COMPLETE** on `dev` — `register_types` (prior
window) + `register_traits` + `register_instances` + the full `collect_tuple_arities` pre-pass, every
piece live-differential-witnessed against the real `checkty` oracle. No open FLAGs in this slice.
Current `dev` tip at handoff: `88efee0e` (moves under concurrent sibling sessions — a snapshot).

### ⇒ RESUME POINT (next Session-A) — `resolve_imports`, then increments 9→14
Next increment-8 piece is **`resolve_imports`** (`checkty.rs::resolve_imports` 1423-1532) — the M-662
per-nodule `use`-import resolution. **It is notably heavier than the register-family and carries a
design decision — scope it verify-first (mitigation #14) before spawning a leaf:**
- Needs an **`Exports`** mirror (`declared: name→is_pub`, plus `types`/`fns`/`traits` maps) and a
  **`NoduleImports`** mirror (`types`/`fns`/`traits` by simple name + an `ambiguous` set).
- Needs the `Item` mirror **un-collapsed further** to carry **`ItUse(UsePath{path, glob})`** (PR-2b
  collapsed `Use` into `ItOther`); keep `collect_tuple_arities_item`'s `ItUse => acc` arm faithful
  (Use is tuple-free).
- Needs **path-string helpers**: join segments with `.`, `direct_child(prefix, qual)` (single trailing
  segment), `split_last_seg(path)`. String work in the port (has `bytes_concat`; needs split/segment
  logic).
- The logic: globs first (lowest precedence) → explicit `use`s (shadow globs); dup-explicit refusal;
  glob-vs-glob → `ambiguous` (never a silent winner); unknown-name vs exists-but-private distinct
  refusals. All never-silent (G2).
- **⚑ DESIGN DECISION (flag, don't guess):** the oracle's `NoduleImports` carries full `DataInfo`/
  `FnDecl`/`TraitInfo` payloads (via `insert_export`), but the *resolution logic* is purely name-based
  (globs/precedence/dup/ambiguity). Option (a) marshal `NoduleImports` fully (needs a surface
  `decode_fn_decl` — not yet built); option (b) project to `(simple→qual bindings, ambiguous set)` and
  compare that — captures ALL resolution behavior without the payload-copy marshalling. **Recommend (b)**
  (the payloads are the source `Exports`' own already-tested decls; the increment's contract is the
  binding/precedence/ambiguity behavior). Confirm at scope time.

Then increments **9** (checker `Cx` ~3.2k — MUST sub-split 2-3 PRs; folds M-1011 `infer_type` residual,
FLAG-semcore-20) → **10** (elab core, consumes M-1012 L0 mirror) → **11** (mono, +FLAG-semcore-17) →
**12** (eval — may need the AOT leg, dossier §7 FLAG-4) → **13** (fuse) → **14** (whole-program
differential + mutants). Pattern for every increment: verify-first vs the oracle → Opus leaf in an
isolated worktree (2 files: `semcore.myc` + `compiler_stage5_*.rs`) → orchestrator re-verifies BOTH
witnesses → `/pr-review` agent → leaf→dev `--no-ff` merge.

### ⇒ HELD: dev→integration→main promotion of the register-family slice (maintainer decision, 2026-07-07)
The completed register-family slice is a **coherent, promotable unit**, but the maintainer directed
**HALT before promoting** — do NOT run the round-trip this session. When resumed (maintainer or next
session), promote it the **curated scoped-slice** way (DN-65 + last window's #1260/#1265 precedent),
because `integration`/`main` carry concurrent sibling work and a full-`dev` promotion would drag
unrelated in-flight changes:
- Branch off `integration`; `git checkout origin/dev --` the semcore slice files (`semcore.myc` + the
  `compiler_stage5_*.rs` test files + `marshal_support.rs` + the `checkty.rs` `pub(crate)` widenings +
  `justfile` de-confliction); confirm each differs from `integration` only by the slice.
- Hand-author the close-out on the shared release surface: **CHANGELOG** (one consolidated block for
  PR-2b + register_traits + register_instances + FLAG-30/33), **DN-26** stays coherent Draft, and the
  **issues.yaml E18-1 status flips** (see below — but the maintainer's PM sync script owns `issues.yaml`;
  coordinate, don't collide).
- Verify on the integration base (stricter tier): full `just check` + both witnesses + fmt/clippy/
  doc_refs. dev→integration `--no-ff`; **`integration→main` squash is HELD as the terminal maintainer
  checkpoint** (maintainer's explicit choice this session — do NOT auto-squash to main).

### Status deltas for the maintainer's PM sync script (I did NOT edit `issues.yaml` — it's script-owned)
- **M-1013** (semcore heavy-core increments) — `in-progress`. Done: increment-8 PR-1 (`resolve_ctors`),
  PR-2 (`register_types`), PR-2b (full `collect_tuple_arities`, FLAG-30 closed), `register_traits`,
  `register_instances` (FLAG-33 resolved). Remaining: `resolve_imports` (+ any `Env` assembly) then
  increments 9→14.
- FLAGs: **FLAG-semcore-30 CLOSED** (PR-2b), **FLAG-semcore-33 RESOLVED** (register_instances PR).
- **Fast-follow (non-gating):** an oracle gap `/pr-review` surfaced — `checkty::collect_tuple_arities_expr`
  treats `Expr::Lit` as a leaf, so a `TupleLit` nested inside a `Literal::List` (`[(1,2)]`) is walked by
  neither the Rust oracle nor the port. Degrades safely (unregistered `Tuple$N` → explicit
  `resolve_tuple` Err). Worth a follow-up against `checkty.rs` (the port is faithfully inheriting it).

### Ops carried forward (Session-A specifics)
- Both witnesses before every land: `cargo test -p mycelium-l1` (the `compiler_stage5_register`/`_elab`
  differentials) + `just myc-dogfood --strict` (`myc check semcore.myc`). clippy `--no-deps` (mycelium-mlir
  unsafe warnings are pre-existing/exempt, ADR-014). `--no-verify` sanctioned (external hooks unreachable);
  gates run out-of-band. Commit msg via `-F <file>` (heredocs/backticks trip the branch-guard parser);
  split `git commit`/`git push` (no `&&`). Force-push PROHIBITED; leaf→dev `--no-ff`; only `main` squashes.
- The GitHub-created merge commits + inherited sibling-session commits on `dev` show as **Unverified**
  (`noreply@github.com`) — NOT ours to rewrite (would need a forbidden force-push to protected `dev`).
  Our own commits carry `noreply@anthropic.com`; the stop-hook flags are expected and inert.
