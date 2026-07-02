# Kickoff `flp` — the PUBLIC FLIP (monorepo, first) + the later decomposition (the Phase-I→II boundary)

> **UID:** `flp` · **Basis:** **ADR-038** (Accepted, 2026-07-01) §2.2/§2.4/§2.8 + the umbrella roadmap
> `docs/planning/road-to-1.0.0-and-mycelium-rewrite.md` §1/§7 · **ADR-036 §2.4 as refined by ADR-038**
> (the flip trigger: functional usability, maintainer-ratified) · **ADR-037** (Enacted — the GHCR/OCI
> spore-registry rails, live-dogfooded) + DN-28 · **DN-27** (Draft — §4 owes the binding decomposition
> ADR this kickoff authors; §5's open questions are its decision surface) · **ADR-018** (per-crate
> SemVer kept; its source-only/no-publish half is superseded by the ADR authored here) · RFC-0031 §5
> **D7** (one spore per phylum — the re-export form).
> **Planned by:** Fable (ADR-038 §2.7); **implemented by:** Opus/Sonnet/Haiku per the PM table.
> **References the doc-maintenance contract** (`_doc-maintenance.md`) in its DoD.
>
> **Sequencing — TWO STAGES, in this order (maintainer direction, 2026-07-01):**
> **Stage 1 — the public flip, of the MONOREPO.** Mycelium is **currently a monorepo and goes public
> as one**, at a `0.x`, when the maintainer ratifies the usability gate. The flip act (**M-938**) is
> **STRICTLY LAST in Phase I** and touches nothing structural — it makes the existing monorepo (and
> its GHCR packages) public. No decomposition, no lock, no archive happens here.
> **Stage 2 — the decomposition, LATER, post-public.** The split into per-phylum-group component
> repos happens **after** the public flip, as a distinct post-public phase, executed against the
> **binding decomposition ADR (ADR-039)** — which is **authored and pushed to the remote for the
> maintainer to review and make the decomposition decision point** (granularity/topology are the
> maintainer's call on the pushed ADR, never guessed — G2). The monorepo is **locked then archived
> only at the end of Stage 2** (once the components are live), as the permanent provenance root —
> never deleted.

## Goal

Cross the Phase-I→II boundary in two ordered stages: **(1)** take the monorepo **public at a `0.x`
in one gated act** once it is fully functional + usable (ADR-036 §2.4 as refined); **(2)** *later,
in the open*, author the owed binding decomposition ADR (**ADR-039** — pushed to the remote as the
maintainer's decision point), then decompose the now-public monorepo into **per-phylum-group repos**
with per-repo CI + GHCR on the ADR-037 rails, ported issues + docs, and spore-first re-exports —
**locking then archiving the monorepo last** as the provenance root.

**Privacy posture (audit early, hold through Stage 1):** nothing is public before M-938 — including
the already-live dogfood packages `ghcr.io/tzervas/*` (ADR-037's `hello`/`std` round-trip
artifacts), whose visibility is **audited, not assumed** (M-937). After M-938 the monorepo is public
by design; Stage-2 component repos are stood up in the open.

## Scope

**In:** Stage 1 — the visibility audit + the monorepo public-flip act; Stage 2 — the decomposition
ADR (pushed for review), repo scaffolds, history carry, CI/GHCR, issue/docs porting, re-export
wiring, the lock-then-archive of the monorepo, and post-decomposition verification. **Out:** any
Phase-II *rewrite* work (that is `rwr`); the kernel freeze (that is `frz`); the semver *scheme*
design before flip-time (deferred per ADR-038 §2.8 — the `0.x` is chosen at the act); any language
or kernel change (this kickoff is pure distribution/topology — DN-27 §3); crates.io (excluded by
ADR-018/ADR-037 — GHCR is the registry).

## ⚠ Maintainer decisions this kickoff carries (FLAG, never guess — G2/VR-5)

| Decision | Prepared by | Gates | Ref |
|---|---|---|---|
| **"Fully functional + usable" ratification** (the Stage-1 flip trigger) | Phase-I kickoffs (`acy`/`enb`/`grm`/`opp`/`frz`) | M-938 | ADR-038 §5 · ADR-036 §2.4 as refined |
| **The public `0.x` version** (chosen at the flip act, not before) | — | M-938 | ADR-038 §2.8 |
| FLAG-V1 — the `lang 1.0.0` label reconciliation — **RESOLVED 2026-07-01** (public release is sub-`1.0.0`/`0.x`; no label collision, no ADR-022 relabel needed) | — | M-938 close-out | ADR-038 §2.8 |
| **Decomposition granularity + topology** — per-phylum-group (~8–12 repos, recommended) vs per-crate vs other; the crate→repo table — **decided by the maintainer on the pushed ADR-039** (the decision point) | M-936 | Stage 2 (M-939 onward) | DN-27 §5 · ADR-039 ⟐ |
| **ADR-039 ratification** (topology · naming · lock-then-archive · versioning · re-export · history · CI · issue-porting) | M-936 | every Stage-2 task | DN-27 §4 |

## The strawman crate→repo table (`Declared` — M-936 carries it into ADR-039 for the maintainer's ⟐)

Granularity recommendation: **per-phylum-group, NOT per-crate** — 52 crate-repos would multiply CI,
issue, and release surface ~6× for no consumer benefit; ~9 cohesive repos keep each dependency
surface small (DN-27 user stories) while staying navigable. Naming rule: **`mycelium-<phylum>`**.
This is a *strawman for the maintainer to accept/amend on the pushed ADR* — it decides nothing.

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
| *(monorepo)* | **locked → GitHub-archived** read-only provenance root; never deleted (end of Stage 2) |

## Swarm method + model tiering (ADR-038 §2.7)

**Stage 1 is a single supervised maintainer session** — the visibility audit (M-937) runs early and
independent; the flip act (M-938) is never fanned out, never automated past its checklist. **Stage 2
is a small Hybrid-tiered swarm, serial through the ADR gate:** M-936 (the ADR) is the single gate
everything in Stage 2 waits on. After ADR-039 is **Accepted** (the maintainer's decision on the
pushed ADR), the scaffold/CI/porting tasks fan out in parallel (each new repo is its own disjoint
tree; `issues.yaml`, the shared reusable workflows, and all indices are orchestrator-owned — leaves
FLAG). One isolated worktree per leaf (mitigation #11); commit/push split (#12); scoped PRs via
`/pr-land`.

## PM decomposition — bite-sized tasks

Proposed M-ids **M-936…M-946** — next-free after the Phase-I kickoffs' proposed blocks (highest
minted today is M-876; `acy`/`enb`/`grm`/`opp` propose M-877…M-935); **re-verify each slot at
minting** (mitigation #1). None minted by this doc.

### Stage 1 — the public flip (of the monorepo; Phase-I→II boundary)

| M-id (proposed) | Task | User story | Definition of Done | Model | depends_on |
|---|---|---|---|---|---|
| M-937 | **GHCR dogfood-package visibility audit** — enumerate every `ghcr.io/tzervas/*` mycelium package (the ADR-037 `hello`/`std` dogfoods and any since); assert each is **private**; wire a re-runnable check so no pre-flip package goes public silently | As the maintainer, I want the pre-flip privacy posture verified, not assumed, so that nothing leaks before the deliberate public act | Visibility of every package recorded (`Empirical`); any public package FLAGged immediately; the check re-runnable + documented | Haiku | — (early; independent) |
| M-938 | **THE PUBLIC FLIP (of the monorepo; strictly last in Phase I; one act):** the maintainer ratifies the usability gate + picks the `0.x` + records FLAG-V1's disposition (already resolved 2026-07-01 — public sub-`1.0.0`, no collision); make the **monorepo repository public** and its **GHCR packages public**; set the version to the chosen `0.x`; flip `publish` on the publishable spores. **No decomposition, no lock, no archive here** — the monorepo stays the active, now-public repo | As the maintainer, I want going public to be one deliberate, gated, reversible-until-executed act on the monorepo I already have, so that "public" is a decision with a checklist, never a drift — and decomposition is a separate later choice | Executed ONLY after the maintainer's usability ratification (recorded); the monorepo + its GHCR public; version at the chosen `0.x`; the act logged append-only in the CHANGELOG; Phase II declared open | Opus | Phase-I DoD (`acy`/`enb`/`grm`/`opp`/`frz`) + **maintainer gates** (usability ratification · `0.x` · FLAG-V1 resolved) |

### Stage 2 — the decomposition (LATER, post-public; gated on ADR-039)

| M-id (proposed) | Task | User story | Definition of Done | Model | depends_on |
|---|---|---|---|---|---|
| M-936 | **Author the binding decomposition ADR** (next-free number — ADR-039 at planning time, re-verify) and **push it to the remote for the maintainer to review + make the decomposition decision point**: fixes (1) topology — DN-27's two-tier component + re-export repos, re-exports **lazy** (a consumer pulls one phylum-group, never the world); (2) **granularity** — per-phylum-group ~8–12 repos recommended, NOT per-crate, carrying the crate→repo table (maintainer ⟐ on the pushed ADR); (3) the **`mycelium-<phylum>` repo-naming rule**; (4) **lock-first-then-archive the monorepo** — protected read-only archive branch, monorepo = provenance root, GitHub-archive never delete; (5) versioning — **keep ADR-018's per-crate SemVer; supersede its source-only/no-publish half** with per-repo GHCR spore delivery (ADR-037 rails); (6) re-export form — **spore-first** (D7); (7) git history — **filtered carry + a committed monorepo→repo SHA map**; (8) per-repo CI+GHCR — **build once as reusable workflows**; (9) multi-repo issue porting — a **`repo:` label axis** + `gh-issues-sync.py --repo` fan-out (closes DN-27 §5's tooling question) | As the maintainer, I want the decomposition's every mechanic fixed in one ratifiable ADR I review on the remote, so that the split is an executed decision, not an improvisation | ADR drafted **Proposed** (not self-ratified), **pushed to the remote**; all nine points enumerated with options + a recommendation wherever a ⟐ stands; DN-27 gains its forward pointer (§4's owed-ADR row satisfied at Accepted); indexed (Doc-Index, adr/README); every FLAG listed | Opus | — (draft may start once Stage 1 is in sight; ratifying is the maintainer's decision on the pushed ADR) |
| M-939 | **Per-phylum repo scaffolds** — for each ratified crate→repo row: repo created (public, since the monorepo is already public post-M-938); `mycelium-proj.toml` + nodule layout for Mycelium-language phyla, workspace-subset `Cargo.toml` for Rust crates; MIT LICENSE; README with the provenance pointer to the monorepo | As a downstream developer, I want each repo a small, self-contained dependency surface, so that I depend on one phylum-group, not the monorepo | Every table row scaffolded; each builds green standalone; naming rule applied | Sonnet | M-936 **Accepted** (granularity ⟐ decided) |
| M-940 | **Git-history filtered carry + SHA map** — per-repo filtered history (`git-filter-repo` or equiv.); a committed monorepo-SHA→repo-SHA map; spot-verified content identity across the carry | As a maintainer/auditor, I want each repo's history real and traceable, so that provenance survives decomposition (never a clean-slate erasure) | History carried per ADR-039's rule; SHA map committed per repo; spot-check recorded (`Empirical`) | Sonnet | M-939 |
| M-941 | **Per-repo CI + GHCR publish as reusable workflows** — build ONCE (shared workflow definitions), each repo calls them; parity with `just check`; the manual-dispatch/advisory posture carried over (no auto-triggers without an explicit decision); spore publish per ADR-037 | As a contributor, I want N repos to share one CI definition, so that gates stay in lockstep instead of drifting per-repo | Reusable workflows landed; every repo wired as a thin caller; one green run per repo recorded; no `on: push`/`on: pull_request` auto-triggers added | Sonnet | M-939 |
| M-942 | **Multi-repo issue porting** — add the `repo:` label axis to `issues.yaml`; extend `gh-issues-sync.py` with `--repo` fan-out; port scoped open issues to their target repos; cross-repo/orchestration issues stay with the root | As the PM system, I want issues to live where the code lives, so that per-repo contributors see their scope without the monorepo's backlog | Label axis + `--repo` fan-out landed (idempotent, reconcile contract preserved); scoped issues ported; `doc_refs`/idmap validity maintained | Sonnet | M-936 **Accepted** |
| M-943 | **Scoped docs porting** — per-repo doc subsets (the specs/api-index slices each repo owns) + cross-repo linkage; the corpus stays canonical in its own repo per the ADR's topology (no forked normative docs — pointers, not copies) | As a model/AI co-author, I want each repo navigable with its own docs and the corpus canonical in one place, so that retrieval never hits divergent copies | Doc subsets ported per the topology; every cross-repo link resolves; exactly one canonical home per normative doc (G2 — no silent forks) | Sonnet | M-939 |
| M-944 | **Spore-first re-export wiring** — the umbrella/re-export repo presents the D7 one-spore-per-phylum surface; resolve pulls per-repo spores from GHCR; publish→resolve→fetch-and-verify round-trip green per repo (ADR-037 §2 discipline) | As a library consumer, I want one coherent phylum interface backed by verified spores, so that I import a phylum, not a pile of components (DN-27 user story) | Re-export surface landed spore-first; round-trip verified per repo (`Empirical`); hash-verified `spore_id`s recorded | Sonnet | M-939, M-941 |
| M-945 | **Lock-then-archive the monorepo (Stage-2 close; after the components are live):** branch-protection lock (read-only), THEN the GitHub-archive act; provenance-root record (README banner + Doc-Index note + the SHA-map pointer); explicit no-delete invariant. Executed only once the component repos are verified live | As a future consumer, I want the monorepo preserved read-only as the provenance root, so that every split repo traces to its origin (ADR-003 identity preserved) | Ordering (lock → archive) explicit + executed; monorepo archived read-only, never deleted; provenance record landed | Sonnet | M-939…M-944 (components live) |
| M-946 | **Post-decomposition verification + close-out** — every component repo public + MIT + provenance-pointered; GHCR resolve from a clean environment; issues live per-repo; the roadmap/CURRENT-STATE/ledger updated | As the Phase-II effort, I want the boundary event verified with evidence, so that `rwr` continues from a checked state, not an assumed one | Verification matrix recorded (`Empirical` per repo); defects FLAGged, none silent; docs/indices current per `_doc-maintenance.md` | Haiku | M-945 |

## Definition of Done (kickoff)

- **Stage 1:** nothing public before M-938 (M-937's audit + check held throughout); the monorepo +
  its GHCR made public in one maintainer-gated act at a `0.x`; Phase II declared open; the act logged
  append-only.
- **Stage 2:** ADR-039 authored, **pushed to the remote for maintainer review**, ratified, and
  **executed as written** — topology, granularity, naming, lock-then-archive, versioning
  supersession, re-export form, history carry, CI, issue porting all per the ADR (deviations FLAGged,
  never improvised); component repos scaffolded with real history, green shared CI, ported
  issues/docs, and verified spore round-trips; the monorepo locked then archived last, never deleted;
  post-decomposition verification recorded (`Empirical`).
- Doc-maintenance per `_doc-maintenance.md`; every FLAG raised, none guessed.

## Prerequisites

1. **Stage 1:** the **Phase-I DoD** met and the maintainer's **"fully functional + usable"
   ratification** (ADR-038 §5) — the flip never front-runs the gate; the Phase-I kickoffs
   (`acy`/`enb`/`grm`/`opp`/`frz`) landed (the flip inherits the whole Phase-I gate, the maintainer's
   checklist authoritative).
2. **Stage 2:** the monorepo public (M-938 done), **ADR-038 Accepted** (the strategy ADR-039
   executes), and **ADR-039 Accepted** (the maintainer's decision on the pushed ADR) — for every
   Stage-2 task after M-936.
