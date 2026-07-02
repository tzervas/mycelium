# Kickoff `flp` — the PUBLIC FLIP: decompose + publish (the Phase-I→II boundary event)

> **UID:** `flp` · **Basis:** **ADR-038** (Accepted, 2026-07-01) §2.2/§2.4/§2.8 + the umbrella roadmap
> `docs/planning/road-to-1.0.0-and-mycelium-rewrite.md` §1/§7 · **DN-27** (Draft — §4 owes the
> binding decomposition ADR this kickoff authors; §5's open questions are its decision surface) ·
> **ADR-036 §2.4 as refined by ADR-038** (the flip trigger: functional usability, maintainer-ratified)
> · **ADR-037** (Enacted — the GHCR/OCI spore-registry rails, live-dogfooded) + DN-28 · **ADR-018**
> (per-crate SemVer kept; its source-only/no-publish half is superseded by the ADR authored here) ·
> RFC-0031 §5 **D7** (one spore per phylum — the re-export form).
> **Planned by:** Fable (ADR-038 §2.7); **implemented by:** Opus/Sonnet/Haiku per the PM table.
> **References the doc-maintenance contract** (`_doc-maintenance.md`) in its DoD.
> **Sequencing: fires at the Phase-I→II boundary.** Prep tasks (the ADR, scaffolds, CI, porting)
> may start once the Phase-I DoD is in sight; **the flip act itself (M-945) is STRICTLY LAST** and
> executes only on the maintainer's usability ratification (ADR-038 §5). Everything stays
> **private** until that one act.

## Goal

Execute the Phase-I→II boundary: author the **owed binding decomposition ADR** (DN-27 §4 — the
next-free ADR number, **ADR-039 at planning time; re-verify at authoring**), decompose the monorepo
into **per-phylum-group public repos**, wire **per-repo CI + GHCR** on the ADR-037 rails, port
scoped **issues + docs**, then **go public at a `0.x` in one act** — all repos created private and
flipped together, only when the maintainer ratifies the ADR-036 §2.4-as-refined usability gate.
The monorepo is **locked first, then archived** as the permanent provenance root — never deleted.

**Privacy posture (audit early, hold throughout):** nothing is public before M-945 — including the
already-live dogfood packages `ghcr.io/tzervas/*` (ADR-037's `hello`/`std` round-trip artifacts),
whose visibility is **audited, not assumed** (M-937).

## Scope

**In:** the decomposition ADR + the executable boundary tasks it directs (lock+archive runbook,
repo scaffolds, history carry, CI/GHCR, issue/docs porting, re-export wiring, the flip act, the
post-flip verification). **Out:** any Phase-II rewrite work (that is `rwr`); the semver *scheme*
design before flip-time (deferred per ADR-038 §2.8 — the `0.x` is chosen at the act); any language
or kernel change (this kickoff is pure distribution/topology — DN-27 §3); crates.io (excluded by
ADR-018/ADR-037 — GHCR is the registry).

## ⚠ Maintainer decisions this kickoff carries (FLAG, never guess)

| Decision | Prepared by | Gates | Ref |
|---|---|---|---|
| **Decomposition granularity** — per-phylum-group (~8–12 repos, recommended) vs per-crate vs other; the crate→repo table | M-936 | M-939 onward | DN-27 §5 · ADR-039 ⟐ |
| ADR-039 ratification (topology · naming · lock-then-archive · versioning · re-export · history · CI · issue-porting) | M-936 | every executable task | DN-27 §4 |
| "Fully functional + usable" ratification (the flip trigger) | Phase-I kickoffs | M-945 | ADR-038 §5 · ADR-036 §2.4 as refined |
| The public `0.x` version + the semver scheme (decided at the act, not before) | — | M-945 | ADR-038 §2.8 |
| FLAG-V1 — the `lang 1.0.0` label reconciliation — **RESOLVED 2026-07-01** (public release is sub-`1.0.0`/`0.x`; no label collision, no ADR-022 relabel needed) | — | M-945 close-out | ADR-038 §2.8 |

## The strawman crate→repo table (`Declared` — M-936 carries it into ADR-039; maintainer ⟐)

Granularity recommendation: **per-phylum-group, NOT per-crate** — 52 crate-repos would multiply CI,
issue, and release surface ~6× for no consumer benefit; ~9 cohesive repos keep each dependency
surface small (DN-27 user stories) while staying navigable. Naming rule: **`mycelium-<phylum>`**.

| Repo (proposed) | Carries (from the 52-crate workspace) |
|---|---|
| `mycelium` (umbrella/re-export root) | spore-first re-export surface over the set (RFC-0031 D7) · install docs · examples. **Name collision with the monorepo — resolved in ADR-039** (e.g. monorepo renamed at archive time, or the umbrella takes another name; FLAG) |
| `mycelium-kernel` | core · dense · vsa · numerics · select · cert · stack · sched (+ the `acy` runtime-ABI seam crate) |
| `mycelium-lang` | l1 · interp · mir-passes · diag · check |
| `mycelium-aot` | mlir |
| `mycelium-toolchain` | cli · cli-common · fmt · lint · lsp · doc · build · proj · bench · sec · transpile |
| `mycelium-std` | the 26 `mycelium-std-*` crates + the `lib/std/*.myc` nodules |
| `mycelium-spore` | spore + the registry scripts/tooling (ADR-037 rails) |
| `mycelium-docs` | the normative corpus (`docs/` ADR/RFC/DN/specs · `research/`) |
| *(monorepo)* | **locked → GitHub-archived** read-only provenance root; never deleted |

## Swarm method + model tiering (ADR-038 §2.7)

**Small Hybrid-tiered swarm, serial through the ADR gate.** M-936 (the ADR) and M-937 (the
visibility audit) run first — M-936 is the single gate everything executable waits on. After
ADR-039 is **Accepted**, the scaffold/CI/porting tasks fan out in parallel (each new repo is its
own disjoint tree; `issues.yaml`, the shared reusable workflows, and all indices are
orchestrator-owned — leaves FLAG). **The flip act (M-945) is a single supervised session with the
maintainer** — never fanned out, never automated past its checklist. One isolated worktree per
leaf (mitigation #11); commit/push split (#12); scoped PRs via `/pr-land`.

## PM decomposition — bite-sized tasks

Proposed M-ids **M-936…M-946** — next-free after the Phase-I kickoffs' proposed blocks (highest
minted today is M-876; `acy`/`enb`/`grm`/`opp` propose M-877…M-935); **re-verify each slot at
minting** (mitigation #1). None minted by this doc.

| M-id (proposed) | Task | User story | Definition of Done | Model | depends_on |
|---|---|---|---|---|---|
| M-936 | **Author the binding decomposition ADR** (next-free number — ADR-039 at planning time, re-verify): fixes (1) topology — DN-27's two-tier component + re-export repos, re-exports **lazy** (a consumer pulls one phylum-group, never the world); (2) **granularity** — per-phylum-group ~8–12 repos recommended, NOT per-crate, carrying the crate→repo table (maintainer ⟐); (3) the **`mycelium-<phylum>` repo-naming rule**; (4) **lock-first-then-archive `main`** — protected read-only archive branch, monorepo = provenance root, GitHub-archive never delete; (5) versioning — **keep ADR-018's per-crate SemVer; supersede its source-only/no-publish half** with per-repo GHCR spore delivery (ADR-037 rails); version stays `0.0.0`, scheme deferred to flip (ADR-038 §2.8); (6) re-export form — **spore-first** (D7); (7) git history — **filtered carry + a committed monorepo→repo SHA map**; (8) per-repo CI+GHCR — **build once as reusable workflows**; (9) multi-repo issue porting — a **`repo:` label axis** + `gh-issues-sync.py --repo` fan-out (closes DN-27 §5's tooling question) | As the maintainer, I want the decomposition's every mechanic fixed in one ratifiable ADR, so that the flip is an executed decision, not an improvisation | ADR drafted **Proposed** (not self-ratified); all nine points enumerated with options + a recommendation wherever a ⟐ stands; DN-27 gains its forward pointer (§4's owed-ADR row satisfied at Accepted); indexed (Doc-Index, adr/README); every FLAG listed | Opus | — (draft may start once the Phase-I DoD is in sight; ratifying needs ADR-038 Accepted) |
| M-937 | **GHCR dogfood-package visibility audit** — enumerate every `ghcr.io/tzervas/*` mycelium package (the ADR-037 `hello`/`std` dogfoods and any since); assert each is **private**; wire a re-runnable check so no pre-flip package goes public silently | As the maintainer, I want the pre-flip privacy posture verified, not assumed, so that nothing leaks before the one public act | Visibility of every package recorded (`Empirical`); any public package FLAGged immediately; the check re-runnable + documented | Haiku | — (early; independent) |
| M-938 | **Lock-first-then-archive runbook** for the monorepo: branch-protection lock (read-only `main`), THEN the GitHub-archive act; provenance-root record (README banner + Doc-Index note + the SHA-map pointer); explicit no-delete invariant; dry-run against a scratch repo | As a future consumer, I want the monorepo preserved read-only as the provenance root, so that every split repo traces to its origin (ADR-003 identity preserved) | Runbook committed + dry-run evidence; ordering (lock → archive) explicit; execution deferred to M-945's act | Sonnet | M-936 **Accepted** |
| M-939 | **Per-phylum repo scaffolds** — for each ratified crate→repo row: repo created **private**; `mycelium-proj.toml` + nodule layout for Mycelium-language phyla, workspace-subset `Cargo.toml` for Rust crates; MIT LICENSE; README with the provenance pointer | As a downstream developer, I want each repo a small, self-contained dependency surface, so that I depend on one phylum-group, not the monorepo | Every table row scaffolded private; each builds green standalone; naming rule applied; zero public repos (M-937's check extended to repos) | Sonnet | M-936 **Accepted** (granularity ⟐ decided) |
| M-940 | **Git-history filtered carry + SHA map** — per-repo filtered history (`git-filter-repo` or equiv.); a committed monorepo-SHA→repo-SHA map; spot-verified content identity across the carry | As a maintainer/auditor, I want each repo's history real and traceable, so that provenance survives decomposition (never a clean-slate erasure) | History carried per ADR-039's rule; SHA map committed per repo; spot-check recorded (`Empirical`) | Sonnet | M-939 |
| M-941 | **Per-repo CI + GHCR publish as reusable workflows** — build ONCE (shared workflow definitions), each repo calls them; parity with `just check`; the manual-dispatch/advisory posture carried over (no auto-triggers without an explicit decision); spore publish per ADR-037 | As a contributor, I want N repos to share one CI definition, so that gates stay in lockstep instead of drifting per-repo | Reusable workflows landed; every repo wired as a thin caller; one green run per repo recorded; no `on: push`/`on: pull_request` auto-triggers added | Sonnet | M-939 |
| M-942 | **Multi-repo issue porting** — add the `repo:` label axis to `issues.yaml`; extend `gh-issues-sync.py` with `--repo` fan-out; port scoped open issues to their target repos; cross-repo/orchestration issues stay with the root | As the PM system, I want issues to live where the code lives, so that per-repo contributors see their scope without the monorepo's backlog | Label axis + `--repo` fan-out landed (idempotent, reconcile contract preserved); scoped issues ported; `doc_refs`/idmap validity maintained | Sonnet | M-936 **Accepted** |
| M-943 | **Scoped docs porting** — per-repo doc subsets (the specs/api-index slices each repo owns) + cross-repo linkage; the corpus stays canonical in its own repo per the ADR's topology (no forked normative docs — pointers, not copies) | As a model/AI co-author, I want each repo navigable with its own docs and the corpus canonical in one place, so that retrieval never hits divergent copies | Doc subsets ported per the topology; every cross-repo link resolves; exactly one canonical home per normative doc (G2 — no silent forks) | Sonnet | M-939 |
| M-944 | **Spore-first re-export wiring** — the umbrella/re-export repo presents the D7 one-spore-per-phylum surface; resolve pulls per-repo spores from GHCR; publish→resolve→fetch-and-verify round-trip green per repo (ADR-037 §2 discipline) | As a library consumer, I want one coherent phylum interface backed by verified spores, so that I import a phylum, not a pile of components (DN-27 user story) | Re-export surface landed spore-first; round-trip verified per repo (`Empirical`); hash-verified `spore_id`s recorded | Sonnet | M-939, M-941 |
| M-945 | **THE PUBLIC FLIP (strictly last; one act):** maintainer ratifies the usability gate + picks the `0.x` + records FLAG-V1's disposition (already resolved 2026-07-01 — public sub-`1.0.0`, no collision); execute M-938's runbook (lock, then archive the monorepo); flip **every** repo private→public **together**; set versions to the chosen `0.x`; flip `publish` per ADR-039; make the GHCR packages public | As the maintainer, I want the flip to be one deliberate, gated, reversible-until-executed act, so that "public" is a decision with a checklist, never a drift | Executed ONLY after the maintainer's ratification (recorded); all repos public in the one act; monorepo locked+archived; versions at the chosen `0.x`; the act logged append-only in the CHANGELOG | Opus | M-936…M-944 + **maintainer gates** (usability ratification · `0.x` · FLAG-V1 resolved) |
| M-946 | **Post-flip verification + close-out** — every repo public + MIT + provenance-pointered; GHCR resolve from a clean environment; issues live per-repo; the roadmap/CURRENT-STATE/ledger updated; Phase II declared open | As the Phase-II effort, I want the boundary event verified with evidence, so that `rwr` starts from a checked state, not an assumed one | Verification matrix recorded (`Empirical` per repo); defects FLAGged, none silent; docs/indices current per `_doc-maintenance.md` | Haiku | M-945 |

## Definition of Done (kickoff)

- ADR-039 authored, ratified by the maintainer, and **executed as written** — topology, granularity,
  naming, lock-then-archive, versioning supersession, re-export form, history carry, CI, issue
  porting all per the ADR (deviations FLAGged, never improvised).
- All repos scaffolded **private** with real history, green shared CI, ported issues/docs, and
  verified spore round-trips **before** the act; the flip itself one maintainer-gated act at a
  `0.x`; the monorepo locked then archived, never deleted.
- Nothing public before M-945 (M-937's audit + check held throughout); post-flip verification
  recorded (`Empirical`); doc-maintenance per `_doc-maintenance.md`; every FLAG raised, none guessed.

## Prerequisites

1. **Phase-I DoD in sight** for drafting (M-936/M-937 may start then); **the maintainer's
   "fully functional + usable" ratification** (ADR-038 §5) for the act itself — the flip never
   front-runs the gate.
2. **ADR-038 Accepted** (the strategy ADR-039 executes) and **ADR-039 Accepted** (for every
   executable task after M-936/M-937).
3. The Phase-I kickoffs (`acy`/`enb`/`grm`/`opp`) need not all be archived for *prep* — but the
   act inherits the whole Phase-I gate, whatever its final composition (the maintainer's checklist
   is authoritative, not this doc).
