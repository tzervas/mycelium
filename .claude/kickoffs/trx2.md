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
