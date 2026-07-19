# Decomposition alignment assessment — 2026-07-18

| Field | Value |
|---|---|
| **Status** | Assessment record (`Empirical`/`Exact` per row; this doc itself makes no normative decisions) |
| **Scope** | The 2026-07-17 monorepo→component decomposition: 45 Rust component repos, `mycelium-cli-myc`, the `mycelium-lang` umbrella, and the monorepo-side program record — evaluated against the maintainer's three uploaded reference docs (design-steer handoff `H`, citations companion `C`, kickoff `K`, all dated 2026-07-17; **not committed in-repo**) |
| **Out of scope** | Content of the 46 `*-myc` twin repos (existence/pins noted only); any fix or remediation (report-only); relitigating which 2026-07-17 program is authoritative (maintainer question) |
| **Method** | Blob-SHA content parity vs archive `aad96b7a` (fetched + identity-chain verified) · per-repo preflight · clause-matrix evaluation · independent adversarial re-verification of every P0/P1 finding (all survived; corrections folded in) |
| **Evidence** | `SCORECARD.md`, `parity/` (summary + non-identical rows + generator scripts) in this directory |
| **Honesty** | Every finding carries an evidence class: `Exact` = mechanically decided over git objects/bytes, reproducible; `Empirical` = observed via API/CI/network at assessment time; `Declared` = asserted by a document, not independently checked |

## 0. Executive verdict

1. **Superset (the headline question): PASS with one named exception.** `tzervas/mycelium`
   contains everything in the 45 Rust component repos — 0 files missing, 0 unexplained new files,
   0 source mutations; the only non-monorepo content across all 46 repos is per-repo scaffolding
   (seed README, `ci.yml`) and **10 generated files in `mycelium-cli-myc`** (`lib/lib.myc`,
   `lib/myc.myc`, `experiments/vet/*`) which exist **only** in that component repo (`Exact`, §2).
   The monorepo itself was not modified by the decomposition (H3 clean; refs verified).
2. **Alignment with the uploaded steer handoff: NOT ALIGNED.** The decomposition executed under
   the *in-repo* `PROGRAM-SELFHOST-DECOMPOSE-2026-07-17.md` (G→A→D→T→R) and matches that
   program's draft map almost exactly — but the uploaded handoff's Phase-3 gates (Phase-0 audit
   ledger, Phase-1 ratification, AX-core DoD) were all unmet and are uncaptured in-repo, and the
   §6 structural requirements (filter-repo history, CROSS-REF.md, per-repo docs, git-pinned deps,
   version train, identity-invariant suite, replicated CI, front-repo shape) are unmet across the
   board (§3, P0/P1).
3. **The repos are non-functional as shipped.** Every crate manifest carries orphaned monorepo
   workspace inheritance, so `cargo` fails at manifest parse in every repo — container repos have
   no root manifest at all; all cross-repo deps are dangling monorepo-relative path deps; each
   repo's single CI run failed (`Exact` + `Empirical`, F5/F11). The seed READMEs **declare** the
   dep breakage as a FLAG (never-silent-compliant), but nothing builds.
4. **Completion claims outrun their basis (VR-5).** "Phase G→A→D→T→R seed complete" appears in
   the program header with no D/T/R status-log rows and no verification records; PR #1703 claims
   "93 repos" (91 umbrella pins / 92 mirror pins; 93 matches no derivable count); the program's
   own A3 CHANGELOG step is half-done (F3).

Net for the follow-up session: the split is a faithful, honest, well-pinned **snapshot** — nothing
was lost and nothing silent — but it is a *paper decomposition*: ungated per the steer docs,
structurally incomplete per H§6, and inert as build units. §6 lists the remediation worklist.

## 1. Baseline identity chain (`Exact`)

`git ls-remote` + fetch transcripts in `parity/baseline-identity-chain.txt`:
`refs/heads/main` = `refs/heads/archive-main-pre-component-transpile-2026-07-17` =
tag `archive/main-pre-component-transpile-2026-07-17^{}` (annotated object `9e477cf6…`) =
`components.lock` header `archive_main` = every seed README/commit-message SHA =
**`aad96b7a425710db5e91094d4fc2ca21a129e41a`** — fetched locally and used as the parity baseline.
Working branch at assessment time == `origin/dev` tip `64431967`.

## 2. Content parity — superset verdict (`Exact`)

Every file in every component repo classified by blob SHA against the archive tree (both
directions; generator `parity/parity.py`; per-repo counts in `SCORECARD.md`; non-identical rows in
`parity/DETAILS.tsv`):

| Class | Count | Meaning |
|---|---|---|
| IDENT_EXPECTED | 768 | byte-identical at the mapped path |
| IDENT_ELSEWHERE | 42 | byte-identical elsewhere (all: the monorepo MIT `LICENSE` blob) |
| MODIFIED | 45 | mapped path exists, blob differs — **all 45 are the seed README stub** replacing the crate/root README; zero source files modified |
| NEW_SCAFFOLD | 50 | seed scaffolding (`ci.yml` ×47, umbrella README + `components.lock`, one README) |
| NEW_UNEXPLAINED | 10 | **all in `mycelium-cli-myc`** — generated `.myc` + vet artifacts, absent from the monorepo at archive and dev tips |
| MISSING | 0 | no baseline file under any mapped source prefix was dropped |

Adversarial re-verification independently re-swept all 46 repos in both directions and confirmed
the exception list is exactly the seed READMEs + `ci.yml`s + the 10 `cli-myc` files. One
precision: in the 40 rooted repos whose crate had a README at the archive, the seed stub
**replaced** the original crate README (original retained only in the monorepo — superset
unaffected). Asset routing (H§6.1): the archive's `lib/` (32 files), `editors/` (12), `fuzz/`
(23), `.devcontainer/` (2) were routed to **no** component repo (zero blob/path matches; the
`*-myc` twins are uninspectable here — flagged in §7).

## 3. Findings register

Every P0/P1 row survived an independent adversarial refutation pass; corrections are folded in.
`H§…` = uploaded steer handoff clause; `K` = kickoff; severity: P0 gate/directive breach ·
P1 structural absence vs H§6 · P2 declared/sanctioned deviation or monorepo-side item · P3 informational.

### P0

- **F1 — Phase-3 gates skipped** (H Phase 0/1/2 gates; K stop points) `Exact`/`Empirical`.
  Phase-0 `AUDIT-LEDGER` absent anywhere (incl. `git log --all`); Phase-1 captures absent (no Swap
  Ergonomics DN, `docs/notes/` tops out at DN-140, no RFC-0013/0034 amendments, retention spec
  only a Draft DESIGN-04 subsection; DESIGN-01..04 all Draft); AX-core absent at the archive tree
  (no `swap_check`/`policy_resolve` emitters in `crates/mycelium-cert/src/mode.rs`, no
  `cert: Option<ContentHash>` on `Meta` in `crates/mycelium-core/src/meta.rs`, no envelope fields
  in `mycelium-diag`); no recorded maintainer topology review — the draft map's §9 FLAGs
  (K1/K2/S1/U1/H1…) explicitly "block unsupervised mass `gh repo create`" yet repos were created
  on Draft defaults. Timeline: D1 map merged 11:27; pins generated 15:40:02Z; closeout PR #1703
  opened 15:46:37Z, merged **3 s later** — the G→A→D/T/R tail ran in ~41 minutes.
  **Context (binding for fair reading):** the steer docs are not committed in-repo; the executed
  program's *own* gates (G verified → A verified) were satisfied per its status log. The finding
  is: *the steer's gates were unmet and never captured in-repo* — not that the executor broke the
  program it was following. Even the steer's §9 verbatim program puts "audit + clean up Grok-era
  work" first; that audit has no in-repo trace either.
- **F2 — ADR-003 identity invariant not demonstrated** (H§6.2-3) `Exact` absence.
  No golden content-hash/spore-identity suite **exists** for this purpose anywhere (the M-110
  `mycelium-interp/tests/golden.rs` corpus is unrelated); nothing was built pre-split or asserted
  post-split, and components cannot run any suite (F5). Mitigating: sampled extraction is
  byte-identical (§2) — necessary but not sufficient, and unasserted/unrecorded.
- **F3 — completion claims without basis** (H§0 N2/VR-5) `Exact`.
  Program header claims "Phase G→A→D→T→R seed complete"; the status log ends at Phase A (D/T/R
  recorded only in a commit message + `artifacts/`). PR #1703 claims "93" repos; umbrella lock
  pins 91, monorepo mirror 92 (adds a `mycelium-lang` self-pin — the only diff); 93 matches no
  derivable count. CHANGELOG has no archive-SHA row (program's own A3 requires it) and no D/T/R row.

### P1

- **F4 — history not preserved** (H§6.2-1) `Exact`. All 47 repos = exactly 1 clean-slice seed
  commit (author `mycelium-decompose <agents@mycelium.dev>`); sampled remotes carry a single ref.
  Contradicts H§6.2-1 (filter-repo, true history) **and** the draft map's own FLAG-H1 default
  ("filter-repo history"); the in-repo program permitted clean-slice as an option, but no choice
  record exists. Partially declared ("mechanical copy from archive" in every README).
- **F5 — non-buildable dependency wiring** (H§6.2-2) `Exact` + `Empirical`. Every crate manifest
  uses now-orphaned `*.workspace = true` inheritance with **no `[workspace]` root in any repo**
  (containers have no root manifest at all) — `cargo metadata` fails at manifest parse
  ("failed to find a workspace root") before path resolution is even reached; all cross-repo deps
  remain monorepo-relative `path = "../…"`; zero Cargo `git` pins (SHA pins live only in the
  non-Cargo `components.lock`); no version train (no version, no tag, anywhere). Declared as a
  FLAG in every seed README (never-silent-compliant; structurally unmet).
- **F6 — `CROSS-REF.md` absent in all 46** (H§6.1) `Exact`. No cross-reference doc under any
  name; the requirement was dropped at the map stage (draft map §4.4 seed list never included it).
- **F7 — no per-repo docs; no `mycelium-docs`** (H§6.1) `Exact`/`Empirical`. No docs dir or spec
  slice in any component (the only extra `.md`s are two byte-identical crate fixtures + cli-myc's
  generated REMAP.md). No `mycelium-docs` repo exists (repo listing shows only the `mycelium-doc`
  tool crate + its twin). Attribution: `mycelium-docs` absence is the draft map's recorded
  FLAG-D1 disposition ("keep docs in monorepo umbrella until R") — a map-vs-steer conflict; the
  per-component doc-slices half has no counter-decision anywhere and is an unmitigated gap.
- **F8 — front repo unrealized** (H§6.1, H§6.2-4) `Exact`. Executed U1: new `mycelium-lang` =
  4 files (README, LICENSE, `components.lock`, `ci.yml`). Present from H§6.1's front-repo list:
  branding README + rev-only component pins. Absent: meta-crate (no Cargo manifest), workspace,
  version train, content-hash half of "rev + content hash" pins, `packages/`, `examples/`,
  release packaging, umbrella integration build (its CI only prints the lock and greps pin
  format; README admits "CI should verify pins resolve (future)"). `tzervas/mycelium` remains the
  full monorepo (per the map's own U1 "until cutover" — and this is exactly why the superset
  verdict holds; completing H§6.1's thin-front conversion would conflict with the maintainer's
  monorepo-contains-everything requirement as stated today — surfaced as a tension to resolve,
  not silently picked).
- **F9 — topology diverges from PARTITION.md** (H§6.1 "seeded from `PARTITION.md` scope groups")
  `Exact`. Executed: 4 seam containers + 27 per-crate std + 13 tooling + l1/transpile/bench —
  e.g. `mycelium-cert` (PARTITION: kernel) and `mycelium-diag` (PARTITION: toolchain) landed in
  the `mycelium-runtime` container; `stack`/`workstack` (PARTITION: runtime) landed in
  `mycelium-core`. Attribution: the executor matched the draft map §3.1–§3.3 **exactly** (46
  repos as its §8 predicts); the deviation is map-vs-PARTITION, decided in the Draft map with
  FLAGged, never-ratified defaults.
- **F10 — asset routing unexecuted** (H§6.1) `Exact`. 69 baseline blobs under `lib/`, `editors/`,
  `fuzz/`, `.devcontainer/` → zero routed to any inspectable component (limit: `*-myc` twins,
  incl. `mycelium-compiler-myc`, the mapped home of `lib/compiler/**`, uninspectable — §7).
- **F11 — CI replication reduced to a stub** (H§6.2-4) `Exact` + `Empirical`. One identical
  test-only workflow in all 45 Rust repos (checkout + rust 1.96.1 + `cargo test`); no
  fmt/clippy/fuzz/release — short of the archive monorepo's 4 workflows and of the draft map's
  own §6 template (fmt + clippy included there). Each repo's single seed-push run: **failure**
  (structural — F5). Correction from verification: the archive monorepo `checks.yml` is itself
  push-triggered advisory (the "manual-dispatch-only" text in CLAUDE.md is stale vs the workflow
  file), so the divergence is *coverage and gating*, not push-vs-dispatch.

### P2

- **F12 — FLAGs resolved without escalation** (K escalate-don't-guess) `Exact`. FLAG-K1/K2/S1/
  U1/H1/D1/C1 all resolved unilaterally on Draft defaults; zero decomposition-era entries in
  `EXPRESS-ORACLE-BLOCKERS-2026-07-16.md`; the map's own text gates repo creation on those FLAGs.
- **F13 — G-8 baseline claim inconsistent (monorepo-side)** `Exact`. `PolicySlot { transitions:
  Vec<PolicySetRecord>, trace: Vec<Explanation> }` (`crates/mycelium-std-runtime/src/policy_mech.rs:125`,
  append-only by design per DN-78 §3 B-2, uncapped) contradicts the handoff §3 G-8 baseline
  "none exist". Pre-existing monorepo content, not introduced by the decomposition; needs
  cap-or-amend review under the retention program.
- **F14 — `mycelium-cli-myc` superset exception + provenance drift** `Exact`. Its 10 generated
  files exist only in that repo (not in the monorepo at archive or dev tip) — the one place the
  maintainer's "OG contains everything" requirement does not hold; its seed README/commit cite
  dev commit `3277b996`, not the archive SHA.
- **F15 — seeds lag the dev wave** `Exact`. Components snapshot `main@aad96b7a`; `dev` was ~509
  files / +76k insertions ahead at assessment time. Consistent with the program's
  archive-from-main choice; material context for any fix session.

### P3

- **F16 — lock drift**: umbrella lock 91 pins; monorepo mirror 92 (adds the `mycelium-lang`
  self-pin); PR #1703 says "93". One-line reconcile + a corrected count.
- **F17 — W-1 canon not swept**: 199 files with `Binary{8}` literals replicated into 20 component
  repos — the pre-existing de facto 8-bit canon (H§2 corrective unexecuted); not a decomposition
  regression.
- **F19 — mechanical gate state** `Empirical` (2026-07-18): repo-wide `scripts/checks/markdown.sh`
  is red with exactly one pre-existing error —
  `docs/planning/gap-analysis-2026-07-16/DESIGN-02-TAGS-META-AND-CONTAINMENT.md:13` MD027
  (Grok-era; a G-10-class mechanical item, left for the follow-up session). This assessment's own
  files pass clean; branch-guard and the secrets fallback scan pass (gitleaks unavailable in this
  session — noted, not silent).
- **F18 — clean checks (positive)**: steer vocabulary G-6 zero hits (`policy: default/auto/_`);
  H4 do-not-lift respected; no secret-shaped content in new blobs; LICENSE = monorepo MIT blob in
  all 46; decision-doc corpus (`docs/adr|notes|rfcs`) unchanged archive→dev (H1/H2 clean); H3
  no-rewrite clean (`main`/`dev`/`integration`/archive refs all present and consistent).

## 4. The two competing 2026-07-17 programs

| Aspect | In-repo `PROGRAM-SELFHOST-DECOMPOSE` (executed) | Uploaded steer handoff (reference for this assessment) |
|---|---|---|
| Committed in-repo | Yes — README-designated "active implement program" | No — zero grep/log hits |
| Decomposition gate | Phase G verified → Phase A verified | Phase-0 ledger resolved · Phase-1 ratified · AX-core DoD · CI green |
| Gate status at execution | Satisfied per its own status log | All unmet (F1) |
| Topology source | `COMPONENT-REPO-MAP-DRAFT.md` (Draft, FLAG defaults) | `PARTITION.md` scope groups (§6.1) |
| History | "history-preserving **or** clean-slice" | filter-repo, history carried (§6.2-1) |
| Docs | FLAG-D1: stay in monorepo until R | per-component slices + `mycelium-docs` (§6.1) |
| Front repo | U1: new `mycelium-lang`; monorepo stays active trunk | `mycelium` itself becomes the front (§6.1) |

The executor followed the in-repo program and its Draft map faithfully (verified repo-by-repo).
The misalignment this report documents is therefore **program-level**: the steer handoff's
decomposition spec and gates were never landed in-repo, and the in-repo program proceeded on
Draft, unratified defaults with completion claims its own log does not back (F1/F3/F12). Which
program governs is the maintainer's call; under the uploaded reference docs, Phase 3 has not
validly begun.

## 5. Per-repo scorecard

See `SCORECARD.md` (47 rows): seed count, three-way pin match (local HEAD == `origin/main` ==
`components.lock`, plus live remote tip — all green), parity class counts, path-dep count,
workspace-root/CROSS-REF/docs presence (all absent), CI file.

## 6. Recommended remediation worklist (for the follow-up session)

Ordered; each item cites the finding it closes. This is a recommendation, not a decision.

1. **Decide the program conflict** (F1/§4): either land the steer handoff in-repo (ratify its
   Phase-3 spec; supersede/amend `PROGRAM-SELFHOST-DECOMPOSE` — append-only) or amend the steer.
   Everything below depends on which topology + gate set governs.
2. **Retroactive gate work if the split stands** (F1/F2): run the Phase-0 audit ledger; build the
   ADR-003 golden identity suite in the monorepo and assert it against the seeds; record results.
3. **Fix the record** (F3/F16): program status-log D/T/R rows with honest verification state;
   CHANGELOG rows (archive SHA per A3; D/T/R); reconcile 91/92/"93"; add the missing umbrella
   self-pin decision.
4. **Make repos buildable or mark them frozen** (F5/F11): per repo, add a `[workspace]` root
   carrying the inherited keys + convert cross-repo deps to git-pinned deps (rev from
   `components.lock`), or explicitly label the seeds "inert snapshot — do not build" in README +
   CI (never-silent either way). Then real CI (fmt + clippy + test at minimum, per the map's own
   template) and a green run per repo.
5. **Structural completion per whichever spec wins** (F4/F6/F7/F8/F9/F10): history strategy
   (re-extract with filter-repo, or record clean-slice as the chosen, ratified deviation);
   CROSS-REF.md per repo (the path-dep edge list in `parity/DETAILS.tsv` is the seed data);
   docs routing decision; front-repo shape (`mycelium` vs `mycelium-lang` — resolve FLAG-U1 with
   the superset requirement in view); asset routing for `lib/`, `editors/`, `fuzz/`.
6. **Monorepo-side follow-ups** (F13/F17): G-8 cap-or-amend on `PolicySlot`; W-1 sweep remains
   open program work.

## 7. Limitations (never-silent)

1. The 45 uncloned `*-myc` twin repos and `mycelium-compiler-myc` are pinned in `components.lock`
   but outside this session's repo scope — existence, tips, and content unverified here (their
   pins are `Declared` from this session's viewpoint). If `lib/` slices were routed anywhere, it
   would be `mycelium-compiler-myc` (F10 limit).
2. `mycelium-docs` non-existence is `Empirical` (repo listing + failed resolution), not proven.
3. CI-failure evidence sampled on 2 repos (`Empirical`); the structural cause (F5) is `Exact` and
   applies to all 45.
4. Merge/actor identities on PRs #1701–#1703 show the maintainer's account; whether merges were
   agent-automated under that identity is not distinguishable from here (`Declared`).

## Changelog

| When | Note |
|---|---|
| 2026-07-18 | Assessment run (baseline `aad96b7a`, dev tip `64431967`); all P0/P1 findings adversarially re-verified. Report-only — no repo content changed by this assessment. |
