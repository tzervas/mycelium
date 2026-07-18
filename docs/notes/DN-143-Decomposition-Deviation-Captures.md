# DN-143 — Decomposition deviation captures (topology · history · front repo)

| Field | Value |
|---|---|
| **DN** | 143 |
| **Status** | **Draft** (captured 2026-07-18 by the course-correction program; awaiting maintainer ratification at the post-fix review — H1/H2: no agent-side `Accepted`) |
| **Grounds** | `docs/planning/design-steer-2026-07-17/PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §6 (the approved Phase-3 spec) · `docs/planning/gap-analysis-2026-07-16/COMPONENT-REPO-MAP-DRAFT.md` (the executed map) · `docs/planning/alignment-assessment-2026-07-18/ALIGNMENT-REPORT.md` (findings F4/F8/F9; §4 two-programs table) · maintainer directive 2026-07-18 (course correction; "correcting the alignment of the rust component repos … so they all properly work and compile") |
| **User story** | As the maintainer, I want the three structural deviations between the executed decomposition and the steer handoff §6 captured as explicit, ratifiable decisions, so that the component-repo fix wave can proceed on a recorded basis and my post-fix review ratifies or reverses each in one place. |
| **Definition of Done** | Each deviation below carries a maintainer verdict (ratify / reverse / amend) with a date; the steer handoff §6 is amended or superseded accordingly (append-only); the ratification queue rows CC-B2/CC-B3 in `../planning/course-correction-2026-07-18/PROGRAM.md` are closed. |

All three captures are `Declared` decisions-in-force (course-correction defaults D-1, D-2, D-8),
not ratified positions. Each states the steer clause it deviates from, the evidence, and the
reversal path.

## 1. Topology — keep the executed 46-repo seam layout (deviates from steer §6.1)

**Steer §6.1** seeds component repos from the `PARTITION.md` scope groups (kernel 7 · runtime 5 ·
frontend 1 · aot 2 · stdlib 27 "default: one repo" · toolchain 12 · transpile 1 · bench 1).
**Executed** (per the draft map §3.1–§3.3, verified repo-by-repo by the assessment): five kernel
seam repos (`mycelium-core` {core,stack,workstack} · `mycelium-value`
{dense,numerics,select,vsa,vsa-decode} · `mycelium-runtime` {sched,rt-abi,interp,cert,diag} ·
`mycelium-l1` · `mycelium-codegen` {mir-passes,mlir}), 27 per-phylum std repos, 13 tooling repos,
`mycelium-transpile`, `mycelium-bench` — 46 repos matching the map exactly, including the
map-vs-PARTITION regroupings (cert+diag into the runtime seam; stack+workstack into core).

**Capture:** the seam layout **stands**. Basis: the maintainer's course-correction directive
operates on the existing repos ("correcting … the rust component repos … so they all properly work
and compile"); re-extraction to scope groups would discard the repos being fixed; the seam
grouping is dependency-coherent (the map's FLAG-K1/K2 analysis). **Reversal path:** re-extract per
§6.1 groups from the archive (all content remains in the monorepo; nothing is lost by having
seeded 46).

## 2. History — clean-slice seeds stand (deviates from steer §6.2-1)

**Steer §6.2-1**: `git filter-repo` per component; new lineages carry their true history.
**Executed:** 46 single-commit clean-slice seeds from archive `aad96b7a` (assessment F4; the
draft map's own FLAG-H1 default was filter-repo, so this deviates from both).

**Capture:** the clean-slice seeds **stand for the fix wave**. Basis: the monorepo retains the
complete history (superset verdict; archive tag/branch preserved); compile-ability — the directive's
goal — is orthogonal to seed lineage; history can be layered later without force (merge a
filter-repo lineage into a seed `main` with `--allow-unrelated-histories`, an additive merge).
**Reversal path:** per-repo filter-repo extraction merged in post-review; no force push required.

## 3. Front repo — `mycelium-lang` umbrella retained; `mycelium`-as-front deferred (deviates from steer §6.1)

**Steer §6.1**: the front repo is `mycelium` itself — re-export/orchestration umbrella, version
train, pins (rev + content hash), packages/, examples/, release packaging; the full docs corpus
moves to `mycelium-docs`. **Executed:** option U1 of the draft map — a new `mycelium-lang`
umbrella; `tzervas/mycelium` remains the full monorepo; no `mycelium-docs`.

**Capture:** U1 **stands**; the `mycelium`-thin-front conversion and the `mycelium-docs` split
are **deferred to the maintainer** (ratification queue CC-B3). Basis: the maintainer's standing
requirement that `tzervas/mycelium` contain everything in the component repos is *currently
satisfied precisely because* the thin-front conversion did not happen — executing steer §6.1 as
written would conflict with it. This tension is surfaced, not silently resolved (G2). The course
correction upgrades `mycelium-lang` to carry the steer's front-repo *capabilities* (version train,
rev + content-hash pins, umbrella integration CI) without demoting the monorepo.

## 4. Topology micro-amendment — `mycelium-select` moves from the `mycelium-value` repo to `mycelium-runtime` (amends the executed map §3.1)

**Problem (`Exact`, measured 2026-07-18):** the executed seam layout contains exactly one
repo-level dependency cycle — `mycelium-value ↔ mycelium-runtime` — caused solely by
`mycelium-select → mycelium-interp` (the map's own FLAG-K2 "non-DAG edge") against
`cert/interp → {numerics, vsa, dense}`. Under steer §6.2-2 (path→git-pinned deps), a repo-level
cycle is not implementable: chasing mutually-referencing revs produces duplicate same-name
packages from different git sources, which cargo rejects. The full component dep graph and the
cycle detection are reproducible from the component manifests (course-correction Phase-B
tooling).

**Capture:** `mycelium-select` (the crate, unchanged) relocates to `mycelium-runtime`'s
`crates/`. Result: `mycelium-value` = {dense, numerics, vsa, vsa-decode} depending only on core;
`mycelium-runtime` = {sched, rt-abi, interp, cert, diag, select} depending on core + value —
the repo graph becomes a DAG (re-verified after the move). Every dependent of `select`
(`l1`, `lsp`, `std-select`, …) already depends on the runtime repo, so no new edges appear.
Crate coverage stays 56/56; the monorepo is untouched. **Reversal path:** move the crate back if
the maintainer instead resolves FLAG-K2 by inverting/removing the `select→interp` edge in code.

## Changelog

| When | Note |
|---|---|
| 2026-07-18 | Minted Draft by the course-correction program (A5): three deviation captures with reversal paths; ratification queued (CC-B2/CC-B3). |
| 2026-07-18 | §4 added same-day: the select→runtime micro-amendment (repo-level DAG requirement under git pins; FLAG-K2 realized). |
