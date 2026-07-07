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
for all 8 `.myc` comparators (the template for M-1013). Verified on both witnesses: `stage5_elab` 11/11
+ full `mycelium-l1` suite green + native `myc check lib/compiler/semcore.myc` ok. This sets the M-1013
pattern; the fresh session starts at STEP 2.

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
