# Changelog

All notable changes to this project are recorded here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Dates are ISO-8601.

This project is in **design + Rust-first implementation**; entries cover both the documentation
corpus and the landing kernel/stdlib code. Semantic versioning will begin when the kernel stabilizes.

> Older entries are archived on periodic sweeps; this file holds `[Unreleased]` + a rolling recent
> window. Full history: archived — see the `archive` git branch (was `docs/archive/changelog/`
> in-tree; extracted 2026-07-09, clean-snapshot prep).

## [Unreleased]

### docs(pm): PROGRAM-SELFHOST-DECOMPOSE active implement program (2026-07-17)

L0 program under `docs/planning/gap-analysis-2026-07-16/`: gap-close → archive main → component
repos → `*-myc` transpile → umbrella re-export (phases G→A→D→T→R; honest gates). Design pack 01–04
stays design review. No product code.

### docs(brand): undouble README logo (2026-07-17)

README brand block used badge + hero of the same mark; keep a single centered badge.

### docs(brand+process): discussion template + brand badge (2026-07-17)

Add `docs/templates/DISCUSSION.md` and GitHub Discussion form
`.github/DISCUSSION_TEMPLATE/design.yml`. Land brand kit under `docs/assets/brand/`
(primary focus `mycelium-primary.jpg` from image-27; badge/hero/social derivatives + variants).
Wire badge + hero into README.

### docs(design): DESIGN-04 language/runtime ledger retention (2026-07-17)

Draft pack 04: inventory and retention model for **Mycelium-internal** perpetual surfaces
(certificates, EXPLAIN gen storage, Meta, first-fault rings). App/ops log offload explicitly out
of scope. Complements DESIGN-01–03.

### docs(design): keep design pack at three docs only (2026-07-17)

Remove leftover `AGENT-F` annex; first-fault site catalog lives entirely in
`DESIGN-03`. Maintainer-facing surface remains DESIGN-01 · 02 · 03 only (Draft).

### docs(design): integrate diagnostics emitters + policy streamline into design pack (2026-07-17)

Design-only deepen of the three-doc pack: policy streamline as ★ primary (01); isolation EXPLAIN
as first-fault instance + grade/meet/seal surfaces (02); Localize-1 site catalog + Rank-1
diagnostics spine (03); Agent F annex. DN-141 body successor stays **Draft**. No product code.

### docs(design): three-doc pack — swaps · tags/containment · diagnostics/UX (2026-07-17)

Design-only. Replaces sprawled council agents A–F + former Draft DN-141 file with **three**
distilled packages under `docs/planning/gap-analysis-2026-07-16/`: `DESIGN-01` (swaps + policy
streamline), `DESIGN-02` (tags/Meta + honesty-poison containment), `DESIGN-03` (AX ranks +
first-fault diagnostic emitters + broader UX). Mermaid diagrams; Draft only (not Accepted).
Implement waves remain paused for design quality (VR-5).

### docs(pm): L0 vs spawned-agent model policy (2026-07-16)

`maint-guide.md` + `PROGRAM-HANDOFF-ONESHOT.md`: **L0** parent session = `grok-4.5`; **all**
spawned agents (L1 all · L2 · PR reviews · explore/plan/security) = `grok-composer-2.5-fast` —
never spawn on `grok-4.5`. L1 floor is **all**, not majority.

### docs(pm): lock standing model floor (`grok-composer-2.5-fast`) (2026-07-16)

`maint-guide.md` Role lattice + `PROGRAM-HANDOFF-ONESHOT.md`: PR reviews (all) · L2 (all) · L1
(majority; prefer all) = `grok-composer-2.5-fast`; record actual model if runtime cannot offer it.
Handoff tips include **C3 #1667** on `dev` `ca8bc623`.

### fix(transpile): M-1084 Import full-path use emit (#1659) (2026-07-16)

ONESHOT Epic B1. Product on `dev` via PR **#1659** (`ec10a0ef`); this entry is the PM/tracker
close-out (CHANGELOG + issues.yaml + handoff done table).

- **Root cause (Empirical):** kernel `resolve_imports` keys exports as **full**
  `nodule.path` + item (`l1.checkty.Width`, `std.fs.error.FsErr`). Live `myc check --phylum`
  accepts full paths and refuses crate-root-stripped short forms (`use checkty.Width` /
  `use error.FsErr` → CheckError). PR #1635's same-crate strip inverted that basis.
- **`SymbolTable::use_emit_qualifier`** is identity on the resolved sibling `nodule_path` for
  same-crate and same-batch cross-crate hits (never fabricate; no silent short-form collapse).
- **Measured:** std-fs Import residual **5** (honest externals/gapped siblings); std-io residual
  **14** with **13** resolved full-path use leaves; synthetic full-path Clean vs strip CheckError.
- **Tracker residual (VR-5):** emit form fixed; **full 768-item phylum `checked_fraction`
  re-baseline is still pending under M-1006** — M-1084 stays **in-progress** until that remeasure
  (not a silent full net-close claim).

### docs(pm): ONESHOT B1 close-out — #1657–#1659 (2026-07-16)

Tracker honesty for the first ONESHOT product/design batch (no one-shot claim):

- **#1659** — M-1084 full-path use emit (product; see entry above).
- **#1658** — ONESHOT handoff + M-1006 post-A5 baseline remeasure
  (`PROGRAM-HANDOFF-ONESHOT.md`, `M1006-baseline-oneshot-2026-07-16.md`,
  `experiments/results/m1006-baseline-oneshot/`).
- **#1657** — M-875 expand-first design draft
  (`M875-expand-first-design-DRAFT.md`); status stays **needs-design** (no implement until
  Accepted).

### docs(pm): ONESHOT handoff + M-1006 post-A5 baseline remeasure (2026-07-16)

PR **#1658** → `dev`.

- **`PROGRAM-HANDOFF-ONESHOT.md`** — active one-shot prep program (DoD, Epic B/C/D, mycelium-only).
- **`M1006-baseline-oneshot-2026-07-16.md`** + `experiments/results/m1006-baseline-oneshot/` — Empirical
  default-5 union **checked 19.5%** (46/236, all Clean) post A4/A5; std-fs/io expansion; ranked residual
  classes. **No** one-shot claim (VR-5).

### docs(m-875): expand-first design draft (#1657) (2026-07-16)

PR **#1657** → `dev`. Design-only close-out for M-875 (ONESHOT B5).

- **`M875-expand-first-design-DRAFT.md`** — Draft under gap-analysis planning path (defers
  mechanism ratification to Accepted DN-100; fills pipeline integration, env, sequencing vs
  M-1084/M-1037, implement-issue DoD).
- Tracker: M-875 stays **needs-design** — **no implement** until Accepted (house rule #3).

### docs(pm): maint-guide OS + ORACLE-R1 A1–A5 program handoff (2026-07-16)

PM / docs close-out for the residual oracle wave (code already on the working tier via
PRs **#1647–#1651**, promote **#1652**). Does **not** claim one-shot transpile readiness or
full M-1006 ladder completion (VR-5).

- **`maint-guide.md`** (repo root) — standing maintenance OS: Phase 0–3, L0→L1→L2 role lattice,
  handoff packet template, trunk policy, acceptance criteria.
- **`docs/planning/gap-analysis-2026-07-16/PROGRAM-HANDOFF.md`** — live L0↔L1 packet (tips,
  Empirical pilot table, open queue, release gate **HOLD** on Epic R / SemVer).
- **`WAVE-L0-ORCHESTRATION-2026-07-16.md`** + gap-analysis **README** — Epic A complete map;
  Epic B (M-1084 → M-1037 → M-1086) next serial transpile path.
- **ORACLE-R1 residuals (product already landed; recorded here for tracker honesty):**
  - **A1** #1647 — lit-zero / signed field compare (`is_negative` / bare-`0` poison).
  - **A2** #1648 — Strength lattice co-emit (`unknown type Strength`).
  - **A3** #1649 — M-1006 default 5-target pilot remeasure post A1+A2
    (`M1006-remeasure-post-A1A2-2026-07-16.md`, `experiments/results/m1006-remeasure-oracler1-a3/`).
  - **A4** #1650 — `DEFAULT_FUEL`/`DEFAULT_DEPTH` co-emit → eval myc-check Clean; expr ~21.4%.
  - **A5** #1651 — wide Show + call-arg BinLit → **std-time checked 0% → 45.9% Clean**.
  - **#1652** — promote working tier → staging tier (lineage merge; same tree as tip).
- **Empirical pilot numbers (default M-1001 five-target set, not full ladder):** post-A1/A2
  union checked **8.5%** / expressible **18.6%**; post-A5 std-time **45.9%** Clean; eval Clean.
  Wide Show remains **Declared** opaque `"<Binary{N}>"` (not Exact Debug).
- Tracker: M-1006 stays **in-progress** with new doc_refs; M-1090 stays **todo** until the
  DoD re-measure on the 30-body bucket (WU-3 emit already landed #1630).

### docs(companion): thematic historian + airlocks + decision clusters (2026-07-16)

Maintained **non-normative** supplement under `docs/companion/` driven by external analysis
pack + audio critique (guarantee-lattice friction / memory lifecycle / thematic DN grouping):

- Guarantee **airlock** patterns for weakest-wins contamination (cleanroom metaphor).
- Memory L1–L3 as one lifecycle (colony bridge; L3 safety net for RC elision).
- Three trust axes (lattice · cert depth · DN-126 strictness) + thematic DN clusters A–F.
- Mermaid diagrams; archive/refresh policy; sources under `docs/companion/_sources/`.
- Wired from README, guide reading-order, Doc-Index. Normative RFC/ADR/DN bodies untouched.

### feat(transpile): M-1090 WU-3 `write!`/`format!` → Show/`bytes_concat` (DN-127) (2026-07-16)

PR #1630 → `dev` (`74171eca`). Composer leaf `claude/leaf/M1090-wu3-write-format-lowering`.

- **`emit/macros/write_format.rs`** — format-string parse + `bytes_concat` fold of literal
  `Bytes` fragments and `render(arg)` (`Show` dispatch); `write!` drops the sink (pure value).
- Alt C first (pure literals / `{{`/`}}`); then Show-gated interpolations; float / `:?` / missing
  Show → honest `MacroInvocation` gap (G2; OQ-1 float residual held).
- Property tests T-1/T-2/T-3 in `src/tests/write_format.rs`; `cargo test -p mycelium-transpile`
  241 green. Lowering **`Declared`** until M-1006/DN-124 re-measure + live-oracle corpus witness.
- **Tracker residual:** M-1090 stays `todo` until re-measure DoD item; WU-1/WU-2 were already
  landed (fmt.myc + prelude seed).

### docs(gap): fractal PARTITION gap analysis G1+G2 — 56 crates (2026-07-16)

PRs #1629 (G1) + #1631 (G2) → `dev`. Orch plan / composer leaves under
`docs/planning/gap-analysis-2026-07-16/` (PARTITION, WAVE plans, 56 `leaves/*.md`, SYNTHESIS +
SYNTHESIS-G2).

- **Rust ADR-022 bar:** largely met on transpile-critical + G2 remainder; residuals are named
  increments (ADR-045, M-740, release hygiene).
- **Transpile readiness:** still early — default `checked_fraction` ≈ 0 on vet pilots; binding
  constraint is toolchain acceptance + Phase-2 emit (M-1084 Import, M-1037 conversion, M-740
  self-host), not missing Rust stdlib code.
- Shared `CHANGELOG`/`issues.yaml`/`Doc-Index` deliberately deferred to this integration-tier
  close-out (swarm partition).

## [0.463.1] - 2026-07-16

### Added — first SemVer cut from Conventional-Commit history

First **history-derived** monorepo version (was placeholder `0.0.0` for the whole life of the tree).

- **Version:** `0.463.1` — `Empirical` over `origin/dev` subjects (chronological stack: each `feat` →
  +0.1.0 minor under `major_version_zero`, each `fix`/`perf` → +patch; merge/wip skipped). Main's
  squash-only `release:` subjects alone would only justify `0.19.0`; dev/int granularity is the
  chosen basis until component-repo decomposition.
- **Wiring:** `[workspace.package] version = "0.463.1"`; all workspace members use
  `version.workspace = true`. `.cz.toml` tracks the same version for future `cz bump` (still
  `publish = false` / no crates.io — ADR-018).
- **Intent:** a fully featured transpile-to-Mycelium-ready Rust base versioned honestly before the
  tree is split into component repos (per-crate independent bumps can resume then).

### feat(transpile): DN-140 valid_ident + length-prefix mangler (M-1106) (2026-07-16)

Composer leaf `claude/leaf/L-EMIT-m1106-valid-ident` (`fd7a984b`).

- **`valid_ident`** in `reserved.rs` — total, idempotent map: identity on legal non-reserved
  idents, `_kw` reserved escape, `_u{HEX}_` illegal-char escape; `sanitize_nodule_path` /
  `guard_ident` call-throughs (DN-140 §4/§7).
- **D4 mangler** — parts run through `valid_ident` + length-prefix boundary (injective by
  construction); generic `Foo[T]` ctor emission no longer hard-parse-fails as the M-1103 gap class.
- Non-identity rewrites emit `// Declared: renamed … (DN-140)` (G2). Mechanism stays **`Declared`**
  until a fuller differential upgrades it (VR-5).
- Tests: `src/tests/valid_ident.rs` + corpus/golden updates; `cargo test -p mycelium-transpile`
  231 passed / 1 ignored.

### chore(maint): Wave-0 hygiene — M-1085 gates + Commitizen 0.x + PR Tracking / epic release-gates (2026-07-15)

Haiku-swarm Wave-0 under the maint-guide framework (orch plans; leaves execute).

- **fix(gates) M-1085** — re-cite `tools/grammar/sugar.yaml` `token.rs:LINE` citations; make
  sugar_index drift-guard demo line-number agnostic; regenerate `docs/sugar-index/`; ruff-format
  `gen/myc-drafts/manifest_gen.py`. Public-api baseline for `mycelium-transpile` already green
  (verify-first residual 0). Flipped `status:todo` → `done`.
- **feat(gates) M-1107** — Commitizen Conventional Commits + `major_version_zero` 0.x.x SemVer
  policy (`.cz.toml`, commit-msg hook, `just cz-check` / `just cz-bump-dry`, `scripts/checks/cz-check.sh`).
  Aligns with ADR-018 / ADR-038 (no auto-1.0.0; dry-run bumps only until publish).
- **docs(contributing)** — required PR **Tracking** footer; GitHub auto-close only on merge to
  `main` (`Closes` vs `Refs`); **epic release-gate** pattern (terminal promote-to-main + release child).
- **tracking** — minted M-1108…M-1118 release-gate todos for all open epics (E9-1, E10-1, E17-1,
  E18-1, E20-1, E22-1, E23-1, E28-1, E33-1, E39-1, E40-1).

### chore(tracking): integration-tier close-out — D4-regression fix + DN-138 increment-1/2 + DN-140 ratified, mint 4 M-ids, flip M-1102 (2026-07-13)

Reconciles `tools/github/issues.yaml`/`docs/Doc-Index.md` against the batch that landed on `dev`
since the last reconciliation (PRs #1589/#1590/#1591/#1592; `dev@2d469efc`), plus one stale status
this pass found while verifying (mitigation #14 — every flip/filing below is checked against the
actual landed code + its PR/commit, never a blind trust of the tracker).

- **fix(transpile): D4 generic self-type mangling hard-parse-fail regression** (PR #1590, merge
  `8f9cddaf`, fix `6c31887a`) — a **regression-and-recovery of M-1101**/DN-131 (PR #1553): once
  DN-131 taught `emit_impl` to accept an impl-level generic parameter, a receiver-less associated
  fn on a generic self type (`impl<T> Foo<T> { fn new(..) }`) mangled to the INVALID identifier
  `Foo[T]__method` — a hard `myc check` parse failure, poisoning the whole containing file under
  the vet loop's file-gated all-or-nothing `checked_fraction` (the dip DN-140's own header cites:
  std-time 6.34%→5.35%). Fixed by gapping the receiver-less-assoc-fn-on-generic-self-type case
  per method (`self_ty_is_generic_application`), never by base-name-stripping (which would
  reintroduce the exact `impl Foo<A>`/`impl Foo<B>` collision D4 exists to prevent). Confirmed live
  on `mycelium-std-time`: parse-error → check-error (an unrelated, already-documented residual).
  Filed **M-1103** (`status: done`, `depends_on: [M-1101]`); DN-34 §8 gains an append-only
  regression-and-recovery changelog line ("§8.23 added").
- **DN-138 (Accepted) — primitive/std derive-instance availability, increment 1 (WU-1/WU-2/WU-3)**
  — landed with no prior `issues.yaml` row (PR #1589, merge `2cd9b773`, DeriveAttr gaps 67→50). A
  conditional `PRELUDE_INSTANCE_SEEDS` spine (parallel to the landed `PRELUDE_TRAIT_SEEDS`) plus
  `field_derive_kind` (replacing a boolean `field_derive_eligible`) routes the five field-gating
  derive rows over primitive/std-type fields. The sig-pin soundness differential needed **two**
  hardening rounds before it genuinely caught a seed naming a non-existent instance — a
  `thread_local` suppression flag proved a no-op across `mycelium_stack::with_deep_stack`'s spawned
  OS-thread boundary; the fix builds the oracle via direct `register_nodule_decls`+
  `register_instances` calls instead (no thread spawn to cross). Filed **M-1104** (`status: done`).
- **DN-138 WU-4 — Vec-recursive derive instances + narrow-width unblock, increment 2** — landed
  (PR #1591, merge `2d469efc`, DeriveAttr gaps 50→48). A conditional `Vec[T]` prelude-type seed
  (`Vec[A] = Nil | Cons(A, Vec[A])`), a new `bin_to_bytes` kernel prim (`bit.to_bytes`) for
  `derive(Hash)`'s scalar route, and `FieldDeriveKind::VecOf` routing (depth-1 `Vec[T]` fields via
  per-element-type auxiliary plain fns, since `Vec`'s coherence head admits only one instance per
  file). Two review findings fixed pre-land: CRITICAL — `PhylumEnv::link()` no longer silently
  replaces a nodule's own hand-declared `Vec` (collision-checked merge, never a silent replace);
  HIGH — narrow `ScalarBinary` widths >64 now honestly gap instead of a runtime-`Overflow`-risking
  `width_cast` narrowing. Filed **M-1105** (`status: done`, `depends_on: [M-1104]`), closing the
  DN-138 §8 worklist.
- **DN-140 (Accepted) — unified valid-identifier emission contract** — docs-only (PR #1592, merge
  `24bd9820`), passed a 3-round strict DN-review gate (a maintainer-input length-prefix redesign of
  the D4 type↔method boundary encoding in round 2). Subsumes DN-139 (its `word→word_kw` rule
  becomes the reserved-word branch); adds a delimited variable-width `_u{HEX}_` per-illegal-character
  Unicode-scalar escape for the class M-1103 above fixed by gapping. Filed **M-1106**
  (`status: todo`, `depends_on: [M-1103]`) for the build (the note itself builds nothing). **Note:**
  DN-139 lives only on an unmerged branch (`claude/leaf/phase2-next-waves-scoping@ee33e4dc`, not
  reachable from `dev`) — its Superseded-by-DN-140 flip is left for whoever lands that branch
  (append-only; not applied here since this pass cannot edit a doc that isn't on `dev`).

**Flipped 1 stale `status:todo` row to `done`** (mitigation #14 — verified against the code before
flipping): **M-1102** (DN-137 native unit-type build) — the previous reconciliation pass filed this
as `todo` (2026-07-13 02:21) but the build landed *after* filing, at 2026-07-13 07:56 (PR #1588,
`fb0bb22f`, unit-type gap 30→0) — a same-day filing/landing race this pass closes.

**`docs/Doc-Index.md`**: registered DN-138 and DN-140 rows (DN-139 has no row — it is not reachable
from `dev`); updated the DN-137 row's landed-basis parenthetical (`todo`→`done`, PR #1588 cited).
`python3 tools/github/doc_refs_check.py` passes clean.

**`docs/api-index/` + `docs/tero-index/` regenerated** (`just docs-index`, `just tero-index-gen` →
`just tero-index` verified current) — both had drifted (line-number staleness) from the code landed
in PRs #1588, #1589, and #1591 since the last regen; a pre-existing gap this pass closes, not
caused by this pass's own (docs/tracking-only) edits.

### chore(tracking): DN-136 Phase-2 wave-1 close-out — file 11 unfiled M-ids, flip 4 stale statuses, DN-135/DN-136 append-only corrections (2026-07-13)

Reconciles `tools/github/issues.yaml` against the DN-136 Phase-1 interfaces build + Phase-2
first-wave leaves that landed on `dev` since the last reconciliation (PRs #1544/#1546–#1555;
`dev@646607d1`) — the recurring drift this project tracks under mitigation #14, and the pattern
this batch's own tracking already found once mid-session (an earlier `tracking-truth-reconcile`
PR #1543 covered up through M-1089's mid-flight state). Every flip/filing below is verified against
the actual landed code + its PR/commit before being recorded (never a blind trust of the tracker).

**Filed 11 M-ids with no prior `issues.yaml` row**, per `docs/planning/DN-136-phase2-bulk-gap-close-worklist.md`
§6 FLAG-3's reserved-id mapping (verified each slot free before minting — mitigation #1; M-1091 was
the highest filed id at session start):

- **M-1092** (DN-135 Result/Option-combinator match-inline lowering, PR #1547) — `done`.
- **M-1093** (DN-134 struct-variant construction + collision-safe `struct_layouts`, PR #1548) —
  `done`; this landing's `struct_layouts` population also closed M-1089's own residual as a side
  effect.
- **M-1094** (DN-133 qualified-associated-fn call emission, PR #1546) — `done`.
- **M-1095** (DN-136 P1-a emit-hook table-dispatch seam, PR #1551) — `done`,
  `depends_on: [M-1092, M-1093, M-1094]`.
- **M-1096** (DN-136 P1-c `map.rs` type-map `TABLE`, PR #1550) — `done`.
- **M-1097/M-1098/M-1099** (DN-136 Phase-2 L1/L2/L3 — `derive(PartialEq/Eq)`, `derive(PartialOrd/Ord)`,
  `derive(Hash)` rows, all PR #1555) — `done`.
- **M-1100** (DN-136 Phase-2 L4 — conversion-method `.clone()`/`.to_owned()` → identity, routed into
  `prim_map::TABLE`, PR #1552 + the same-PR critical fix `1f4e7aa3` narrowing the receiver gate to
  builtins only) — `done`. Note: the leaf's working branch was misnamed
  `claude/leaf/m1101-conversion-prim-map` (off-by-one against this reserved id); the landed commit
  content is unambiguously M-1100/L4, not a duplicate of M-1101.
- **M-1101** (DN-136 Phase-2 L5 / M-1088's own residual — bounded inherent-impl type-param
  transpiler emission, PR #1553) — `done`, `depends_on: [M-1088]`.
- **M-1102** (DN-137 native unit-type build: prelude `type Unit = Unit;` + one `type_map::TABLE`
  row) — `todo` (DN-137 Accepted 2026-07-13; not yet built).

**Flipped 4 stale `status:todo` rows to `done`** (mitigation #14 — each verified against the code,
not the tracker):

- **M-1086** (DN-128 std-derive lowering library) — the Debug/Default/Clone/Copy rows landed via
  PR #1544 (after this issue's own partial-landing note was written, a same-day ordering gap) and
  the PartialEq/PartialOrd/Hash rows via M-1097/M-1098/M-1099 (PR #1555); all live-oracle-witnessed
  (205/205 `mycelium-transpile` tests green). The DoD's own re-measure item has not been re-run
  post-wave — flagged as an open measurement follow-up, not a build gap.
- **M-1088** (DN-131 bounded-generics transpiler residual) — closed by M-1101/PR #1553
  (`bounded_impl_type_params`, verified at `emit.rs:508`).
- **M-1089** (DN-132 P1 struct-variant patterns) — its producer-side residual (`struct_layouts` not
  walking `Item::Enum` variants) closed as a side effect of M-1093/PR #1548 (verified at
  `transpile.rs:394`/`:426`).
- **M-1091** (DN-129 Init/Fault prelude traits) — its `derive Default` → `Init` cross-note gate
  closed by the same M-1086/PR #1544 landing.

**`docs/Doc-Index.md`**: registered DN-133/DN-134/DN-135/DN-136/DN-137 rows (all previously
unregistered despite DN-133–135 being cited by already-filed issues — a pre-existing gap this pass
also closed) and a Planning-docs row for `docs/planning/DN-136-phase2-bulk-gap-close-worklist.md`.
`python3 tools/github/doc_refs_check.py` now passes clean (was failing with 12 dangling refs before
the Doc-Index rows + two `doc_refs` grammar corrections — `corpus:M-1037` is not a doc ref type,
issue ids aren't citable that way; the worklist doc's `#l1`–`#l5` anchors don't resolve to real
headings, so those became a plain `corpus:DN-136-phase2-bulk-gap-close-worklist` ref instead).

**`docs/notes/DN-135-...md`** — append-only scope-correction addendum (house rule #3/#4): DN-135
§Decides/§3's original "chains nest" claim is DISCONFIRMED by M-1092's landed build (a nested
inlined `match` scrutinee fails `myc check`'s constructor type-parameter inference); each combinator
in a chain is judged independently on its own receiver instead. Original prose left untouched.

**`docs/notes/DN-136-...md`** — append-only numeric-grounding correction: the ratification commit's
own claim ("the live count at `c044452d` is 89 `Case` literals, three independent counts agree") was
itself a miscount — re-counted directly against `git show c044452d:.../tests/emit.rs` (both
`grep -c` and an independent Python regex count over the exact `cases()` span): the correct figure
is **88**, not 89.

**`crates/mycelium-transpile/src/emit.rs`**: fixed the unrecognized-derive fallback message
(`lower_struct_derives`), stale since PR #1555 landed Eq/Ord/Hash without updating it — it still
claimed "Eq/Ord/Hash/PartialEq/PartialOrd are a separate, unbuilt increment", though
PartialEq/PartialOrd/Hash have been recognized since that same PR (only bare `Eq`/`Ord` are
deliberately unrecognized, by design — see `emit/derives/mod.rs::TABLE`'s doc). Updated the golden
differential fixture (`tests/fixtures/emit_hook_golden.json`) to match; the byte-identical
differential (`emit_hook_refactor_byte_identical_differential`) stays green.

**`crates/mycelium-transpile/src/emit/derives/ord.rs`** (cosmetic, low-risk): folded the two
separate field-scan loops (Float-check-all-fields-first, then eligibility-check-all-fields) into
one combined per-field loop, mirroring `eq.rs`'s single-pass shape — consistent
first-offending-field reporting when a struct has both a Float field and a separately-ineligible
field. No test pinned the prior two-loop ordering; confirmed the byte-identical differential and
all 205 `mycelium-transpile` tests stay green.

### chore(tracking): tracking-truth reconciliation — M-1077/1079/1080/1081/1088/1089/1090/1091 verified against `dev` (2026-07-12)

`issues.yaml` had drifted from `dev`'s actual code (mitigation #14, recurring this session): the
DN-126–DN-132 build issues below all landed on `dev` between the batch ratification above and this
reconciliation, but stayed `status:todo`. Every candidate re-verified against `dev`'s code + its
landing PR before any flip (never a blind trust of the tracker — VR-5). **M-1089 landed mid-session**
(PR #1535, `f109b0d5`, pulled down via `git merge --no-ff origin/dev` partway through this pass) — its
consumer-side `Pat::Struct` arm is done, but the producer-side enum-struct-variant `StructLayout`
population is not, so it stays `todo` scoped to that residual (see below).

- **M-1079** (DN-124 vet-harness phylum visibility) — **done**. All three units landed (PR #1521,
  `eb6bc0e2`): `mycelium-check`'s per-nodule `PhylumReport` verdicts, `mycelium-transpile`'s
  phylum-mode dual-report (`checked_fraction_phylum`/`delta_basis`), and
  `gen/myc-drafts/regenerate.sh`'s semcore-as-one-phylum batching. `cargo test -p mycelium-check`
  (13/13) + `-p mycelium-transpile --lib` (98/98, re-verified in an isolated worktree at
  `eb6bc0e2` itself — a first pass of this note misquoted 113/113, the count read later in the
  session after M-1080/M-1081/M-1089's tests had also landed on top; corrected per a PR review
  finding, VR-5) green.
- **M-1080** (DN-122 external-trait-impls MVP) — **done** (mechanism landed, honest 0-corpus-delta).
  WU-A/WU-B landed (PR #1522, `b97b008e`) with T-A1–T-A3/T-B1–T-B2 green. The Phase-0 re-measure
  (under M-1079's landed phylum-mode basis, `06b4d7a7`) measures OQ-7's single-param-vs-two-type
  split at **0% on the witnessed wave-1 corpus** — a measured, not fabricated, leverage figure
  (VR-5).
- **M-1081** (DN-125 `&mut self`/`&mut T` value-threading) — **done**. The lowering landed
  (PR #1527, `6cae69eb`); a re-review found and closed a silent-corruption aliasing hole — `let y =
  other;` where `other` is itself a threaded `&mut` binding moved the live reference under
  name-based matching, so a later `*y = ..;` silently mutated the wrong binding while still
  `myc check`-clean — fixed in PR #1530 (`ae4007bf`) by refusing that shape never-silently
  (`Category::Other`). `cargo test -p mycelium-transpile --lib -- tests::mut_thread` 14/14 green.
- **M-1077** (DN-126 two-mode typing) — **partial, stays `todo`**. The demotion switch +
  mechanical-strictification classifier landed (PR #1531, `cd84768a`, 11/11 tests green) — DoD items
  1–2 of 3. The py2rust end-to-end fixture (item 3) is genuinely unbuilt and not near-term (py2rust
  integration is explicitly deferred pending Rust-native maturity).
- **M-1088** (DN-131 impl-slot bounds) — **partial, stays `todo`**. The L1/kernel side (parse,
  desugar, check+mono, witnesses) landed (PR #1529, `6822a78c`, 34/34 + 3/3 tests green); the
  transpiler emission item is not built — `emit.rs` still refuses any bounded type parameter.
- **M-1089** (DN-132 P1 struct-variant patterns) — **partial, stays `todo`**. The consumer-side
  `Pat::Struct` arm in `map_pattern_inner` landed (PR #1535, `f109b0d5`, 22/22 tests green); the
  producer-side `transpile.rs::struct_layouts` still only walks `Item::Struct`, not `Item::Enum`
  struct-variants, so the issue's own headline target — `Self::NotFound { path, .. }` — still gaps.
- **M-1090** (DN-127 native formatting) — **partial, stays `todo`**. WU-1 (`lib/std/fmt.myc`'s
  `to_dec`/`digit_byte`, `myc-check`-clean, 19/19 differential tests) and WU-2 (`Show` prelude seed)
  landed (PR #1526); WU-3 (the transpiler `write!`/`format!` lowering rule) is not built.
- **M-1091** (DN-129 Init/Fault prelude traits) — **partial, stays `todo`**. The seeds, the shared
  `seed_prelude_trait` helper, visibility tests, and OQ-2 (resolved as a bare `Fault[T] {}` marker)
  landed (PR #1526, 18/18 tests green); the `derive Default` → `Init` cross-note gate is blocked on
  **M-1086** (DN-128 std-derive lowering), which remains unbuilt.

`docs/Doc-Index.md`'s DN-122/124/125/126/127/129/131 rows corrected (their "unbuilt, tracked as
M-xxxx" parentheticals were stale); `tools/github/idmap.tsv` gained the two rows the pending
`claude/leaf/pm-sync-m1079-m1080` idmap branch (`f3e02474`) had queued (M-1079→#1519,
M-1080→#1520) — folded into this reconciliation rather than landed separately. M-1084 (Import
net-close) and M-1089 (DN-132 struct-variant patterns) are genuinely in-flight under other leaves
and untouched here; M-1086 (DN-128 std-derive) is genuinely unbuilt and untouched. All DN statuses
(DN-123, DN-125–DN-132) confirmed **Accepted** in both `docs/Doc-Index.md` and their own doc
headers; DN-133 confirmed still **Draft** (untouched, in-flight citation-patch elsewhere).

### docs(dn): ratify DN-126–DN-132 — language-completeness planning batch (Accepted, 2026-07-12)

Batch-ratifies **seven Draft DNs** to **Accepted** under explicit maintainer delegation ("ratify based
on objective reasoning and the project's needs/intents, keep to core principles, report results";
mirrors the DN-115/117/118/122/123/124/125 precedent). Every mechanism/guarantee tag stays `Declared`
(unbuilt) — **Accepted, not Enacted** (house rule #3) for all seven.

- **DN-126** (two-mode typing, M-1077) — loose mode = the existing bidirectional checker in a
  non-refusing posture over the unchanged repr-dynamic evaluator; strict mode = the same checker with
  demotion off (compilation gate unconditional, as today). Ratifies the **three-axis verdict**
  (type-strictness is a genuinely new axis, distinct from ADR-032/RFC-0034 cert-depth and RFC-0018
  guarantee-grade), the **runnable-floor boundary** (name/arity/parse/FFI stay hard in loose mode),
  and mechanical strictification's **principality invariant** (writes down only a *principal* inferred
  type — sound-by-conservatism, VR-5). Zero kernel growth. `doc_refs: corpus:DN-126` added to the
  already-filed **M-1077**.
- **DN-127** (native formatting) — `Display`/`write!`/`format!` → a pure `render: T → Bytes`; a
  `Show` prelude trait for dispatch. Int→decimal is derivable **in std from landed prims**
  (`div_u`/`rem_u`/`bytes_concat`/`width_cast`) — **no new kernel primitive** (KC-3). Float render
  stays an honest residual (OQ-1). Note the merge-order dependency on DN-125 (`&mut Formatter` param) —
  DN-125 is already Accepted and landed on this same base. Minted **M-1090** (`depends_on: [M-1081]`);
  the prior design issue **M-1082** gets an append-only close-out note, `status: superseded-by-dn`.
- **DN-128** (std-derive lowering library) — per-derive `lower` rules as structural folds (DN-54).
  `Clone` = a value-semantics identity no-op (drop as satisfied, don't generate); a derived total
  `Eq` over a `Float` field is **refused** (NaN/ADR-040). OQ-1 (field reflection in a `lower` RHS)
  stays honestly open; Alt C (compiler-internal field-walk) recommended because it survives either
  answer. Minted **M-1086**.
- **DN-129** (Default/Error) — `Init` prelude trait (method **not** `default` — a taken keyword);
  `Error` = errors-as-values + a `Fault: Show` marker; `source()`/`dyn Error` boxing deliberately not
  ported (ADR-033's escape, not the default). Zero kernel growth (DN-55). Minted **M-1091**
  (`depends_on: []`); the prior design issue **M-1083** gets an append-only close-out note,
  `status: superseded-by-dn`.
- **DN-130** (generic trait-instance impls, `impl[T] Trait for Foo[T]`) — a parametric instance head
  monomorphized as a family (M-673 α-substitution); coherence keyed on the constructor head, reusing
  DN-122's home-qualified `CoherenceView`. Scoped to single-parameter, structurally-covering,
  non-overlapping heads; out-of-scope shapes refused never-silently. **Real landing-order dependency
  on M-1080** (honestly stated — not a landed reuse, VR-5). Minted **M-1087**
  (`depends_on: [M-1080]`); M-1080's own body gets an append-only note recording the second dependency.
- **DN-131** (bounds on non-fn sites, impl-slot bounds) — the impl-slot bound rides DN-103's
  desugar-prepend plus the already-landed `check_bounds` + dictionary-free mono — **zero new
  discharge code**. Declines `type`/`trait` decl-head bounds and `where`-clauses per RFC-0019 §4.2's
  own design intent (YAGNI, not convenience). Minted **M-1088**.
- **DN-132** (L3 pattern-surface cluster) — **ratifies P1 (struct-variant patterns)** as the
  buildable mechanism (variant-aware `StructLayout` + a `Pat::Struct` arm, reusing Maranget usefulness
  unchanged, KC-3). **P2/P3 (range/`@`-binding via the `when`-guard idiom) are explicitly
  PREREQUISITE-GATED on M-833/DN-79 landing** — recorded as a documented conditional, **not**
  "already served" (the note's own load-bearing self-correction: `Arm` has no guard field today and
  the transpiler refuses every guard). The B2/C2 dedicated-grammar decline is Accepted regardless of
  that timeline. Status recorded as **Accepted — P1 only** (taking FLAG-6's split as offered). Minted
  **M-1089** for exactly the P1 transpiler build (`depends_on: []`); the self-hosted P1 half and the
  P2/P3 idiom emission stay FLAGged follow-ups, not minted here (avoid filing a blocked issue that
  implies near-term actionability it does not have).

Reconciled `docs/Doc-Index.md` (seven new rows, all Accepted 2026-07-12) and `tools/github/issues.yaml`
(six fresh M-ids **M-1086..M-1091**, plus `doc_refs`/append-only notes on **M-1077**, **M-1080**,
**M-1082**, **M-1083**). Regenerated `docs/api-index/` and `docs/tero-index/` where doc_refs changed.
`doc_refs_check.py`, markdownlint, `structured.sh`, `links.sh`, `secrets.sh` green.

### chore(gates): flag pre-existing `just check-canary` drift on `dev` from PR #1521 (2026-07-12)

Running `just check-canary` (leaf→`dev` gate) during the gap-close-2 integration close-out surfaced
three pre-existing failures on `dev`'s own tip, unrelated to this close-out's docs/`issues.yaml`-only
diff (verified: `git diff` of the affected files against this PR's branch is empty) — left by the
earlier landing of PR #1521 (the DN-124 phylum-mode harness + wave-2 symtab): a stale
`mycelium-transpile` public-API baseline (`api` gate), an unformatted
`gen/myc-drafts/manifest_gen.py` (`format` gate), and the pre-existing `sugar-index` self-test
citation drift. Flagged plainly (never-silent, G2) rather than silently bundled into this
docs-scoped PR; tracked as **M-1085**.

### docs(planning): language-completeness gap inventory — the drive-hard worklist (Draft, 2026-07-12)

`docs/planning/language-completeness-gap-inventory.md` (Draft, living register) synthesizes and
re-derives the current, prioritized **language**-completeness gap inventory for full native
expressibility of Rust (Python carry-forward flagged), grounded against `origin/dev` `fa53dc46` plus
the committed draft corpus and the DN-124 phylum-mode baseline. It re-derives the gap-class
distribution against the current tree (mitigation #14) and corrects nine stale figures/rankings in
the prior analyses — the "Other/type-coverage" class shrank from a claimed 40% to ~23–24%,
external-trait impls moved from "needs-design" to **DN-122-Accepted**, `?`/transcendentals are
landed/superseded, and **`&mut self`/`&mut T` is un-owned** (not DN-118, which scopes only
closure-capture mutation) — the single largest unsolved language residual. It surfaces four
new/under-weighted gaps (DeriveAttr now ~11–12% of gap mass; the missing Display/int→string kernel
prim blocking 26/30 `&mut Formatter` bodies; `ModuleDecl` as its own class; no native `Default`/
`Error` prelude trait), classifies every gap by the ratified DN-111 native-translation taxonomy, and
splits the 17-row residual into build-now (a ratified design exists) vs design-first (needs a Draft
DN) sets. Recommends, does not ratify (house rule #3); every figure `Empirical`/`Declared` at its
basis (VR-5).

### docs(dn-125): ratify native `&mut self`/`&mut T` in-place-mutation note — value-threading (Accepted, 2026-07-12)

**DN-125** (`docs/notes/DN-125-Native-In-Place-Mutation-Through-A-Reference-Value-Threading.md`)
moves **Draft to Accepted** under explicit maintainer delegation to the orchestrator ("ratify based
on objective reasoning and the project's needs/intents, keep to core principles, report results").
Scopes Mycelium's native answer to the problem Rust `&mut self`/`&mut T` solves — the dominant
DN-34 §8.22 `Impl`-class gap — separating a settled question from an open one (mitigation #14): the
runtime **mechanism** is **ANSWERED-BY-DESIGN** (value-threading — take the receiver/argument by
value, return the mutated value, rebind at the call site — zero-copy via the already-ratified DN-33
static uniqueness analysis, DN-35 §5 `rc==1` in-place reuse, and DN-120 identity coherence), while
the transpiler **application** (mechanically lowering a `&mut self` method plus rewriting its call
sites) is genuinely open and un-owned before this note (`&mut self` hard-gaps at `emit.rs:559`,
`&mut T` at `map.rs:344`). Ratifies **Rank 1 — Alt A (value-threading)** over a kernel `&mut`/place
type (Alt B, rejected — reintroduces the aliased-mutation borrow-checking Mycelium deliberately
excludes) and an interior-mutability cell (Alt C, retained only as a narrow Interop-Bridge fallback).
Adversarial stress-test **HELD** for the non-aliased, value-returning shape (re-entrancy is *more*
robust than `&mut`; identity coherence closed by DN-120), **NARROWED** to two never-silent FLAG
boundaries: unprovable-unique/aliased receivers (routed to a borrowck precondition or a DN-33
Mycelium-side proof) and interior-`&mut`-returning methods (`get_mut`/`iter_mut`/`IndexMut`, routed
to Approximation/Interop-Bridge). **Accepted, not Enacted** (house rule #3) — the lowering stays
`Declared`/unbuilt, correct-with-a-copy when built, zero-copy only as DN-33/DN-35 §5 land. Minted
**M-1081** (transpiler value-threading lowering build; `depends_on: [M-1079]`).

### docs(dn-123): ratify records/named-fields surface-lever note — sugar-over-positional (Accepted, 2026-07-12)

**DN-123** (`docs/notes/DN-123-Records-Named-Fields-Surface-Lever.md`) moves **Draft to Accepted**
under the same maintainer delegation. Ratifies **Option A** — a mechanically-lowering sugar over the
existing positional `Ctor`/`Data` machinery plus the transpiler's field-name↔index map
(`StructLayout`), per DN-106's ratified General Principle 2 (gap-closure default = sugar, not a
kernel primitive) and General Principle 1 (surface-sugar transparency) — with **Option C** (an inert
`field_names` type-registry metadata field) as a YAGNI-gated faithfulness upgrade and **Option B**
(a first-class named-field kernel variant) recorded rejected (DN-106 fork B, KC-3). Accepts the
verify-first correction: records already emit positionally and check-clean whenever field *types*
map — the genuine residual is **faithfulness** (the `NamedFieldDrop` sub-gap) plus the **self-hosted
`.myc` surface** (DN-119 L3-G1), not a new expressibility gap. §7's OQ-1 (canonicalize literal field
order to declaration order; names off the content-address hash) and OQ-3 (functional-update spread
affine treatment) are accepted as build preconditions; OQ-2 (cross-phylum name metadata) as a
coordination dependency on DN-113/M-1060. **Accepted, not Enacted** (house rule #3) — the sugar
stays `Declared`/unbuilt. **M-1078** (already minted 2026-07-11) tracks the residual build; this
ratification clears its DoD precondition (DN-123 ratified).

### docs(dn-124): ratify vet-harness phylum-visibility and measurement-basis note (Accepted, 2026-07-12)

**DN-124** (`docs/notes/DN-124-Vet-Harness-Phylum-Visibility-And-Measurement-Basis.md`) moves
**Draft to Accepted** under explicit maintainer delegation to the orchestrator ("ratify based on
objective reasoning and the project's needs/intents, keep to core principles, report results").
Ratifies **P-A** — sound partial per-nodule verdicts on `PhylumReport` via a driver-level
import-closure sub-phylum re-check that reuses the kernel `check_phylum` unchanged (zero kernel
growth, KC-3), then switches the vet path to `myc check --phylum <dir>` — and **M-A** — phylum-mode
is the demonstrably-correct measurement basis (a real build's semantics; proven in-tree by
`phylum_cross_nodule_reference_resolves`), with the one-time `checked_fraction` jump on the switch
dual-reported and labeled a **basis correction, not lever progress**, then re-baselined with
`Δ_basis` attributed (the historical oracle §8 series is annotated, never rewritten per house
rule #3). The **import-closure invariant** is the load-bearing false-clean guard, adversarially
verified against four attack scenarios (§6). **Accepted, not Enacted** — every mechanism/verdict tag stays
`Declared` until Units 1–3 (§5.3) land and are differential-witnessed. Minted **M-1079** (harness
build: `PhylumReport` partial verdicts, `--phylum` vet wiring, `regenerate.sh` semcore batching;
`depends_on: [M-1060]`).

### docs(dn-122): ratify external-trait-impls MVP note (Accepted, 2026-07-12)

**DN-122** (`docs/notes/DN-122-External-Trait-Impls-Across-The-Home-Boundary.md`) moves **Draft to
Accepted** under the same maintainer delegation. Ratifies the **§13 build-ready MVP**: the
single-parameter, param-only-signature foreign-trait-impl class (prelude-scoped first) via a
**transpiler rule-swap (WU-A) plus target-trait availability (WU-B)**, riding the now-landed
M-1060/M-1036 checker substrate with zero new kernel or checker work. Soundness argument: the MVP
admits exactly the complement of the landed M-1060 cross-phylum bare-name-collapse guard, so it
cannot reopen that collapse by construction (the verified `{carrier}×{position}` surface). Resolves
**OQ-6 (target-trait availability) as prelude-seed** over std-phylum-declare — KISS/YAGNI plus
least-soundness-surface grounds, since the prelude-scoped closure is a single uniform home with no
cross-phylum import, diamond, or separate-compilation tension for v1; std-phylum-declare is
deferred to the M-1076/WU-C cross-phylum follow-up. **OQ-7 (single-param vs two-type split of the
114-gap/12.4% class) stays an open, honest residual** — unmeasured until the WU-A Phase-0
re-measure runs under DN-124's phylum-mode vet basis; the leverage tag stays `Declared`. Unsupported
shapes (two-type/`Self`-needing traits, incl. the canonical `Widen` witness; concrete-type-in-sig)
are refused never-silently, tracked as M-1076/M-876 (WU-C, out of v1 scope). **Accepted, not
Enacted** (house rule #3). Minted **M-1080** (MVP build: WU-A emit `use <trait-home>.<Trait>` for
single-param param-only traits, plus WU-B prelude-seed target-trait availability;
`depends_on: [M-1060, M-1079]`).

### fix(l1): M-1060 cross-phylum type-identity soundness closure — 4 fix cycles (2026-07-11)

Adversarial-verification follow-through on the M-1060/DN-113 v1 cross-phylum landing (PR #1503):
four fix cycles closing the cross-phylum bare-name type-identity collapse class — the M-1036
ctor-seal pattern one level up, across the phylum boundary. Each cycle found and closed during the
leaf's own self-verification, not by an external reviewer (mitigation #14).

- **PR #1506 (`9a03f988`, CRITICAL)** — re-homes every constructor field's `Ty::Data` identity
  through `qualify_ty_cross_phylum` against the dependency's own linked `Env::types` (the same
  helper/oracle the `resolved_fn_sigs` loop already used — one re-homing path, not two). Without this,
  a field naming a dependency-internal nodule collided with a same-named consumer nodule and the
  bare-name fallback saw no home mismatch, silently accepting a foreign representation as the
  consumer's own. Also closes a related MED: a foreign trait's method signature naming a concrete
  type is not yet re-homed at impl-check time — closed with a narrow never-silent refusal
  (`register_instances`), not a silent re-resolution.
- **PR #1508 (`a282104a`, holes A/A2/B)** — `check_trait_method_call` resolving a foreign trait's
  method sig against the CONSUMER's own registry when invoked through a generic bound (HOLE A return
  position, HOLE A2 value-param position); `check_app`/`check_app_generic_fn` falling back to
  `resolve_ty` against the caller's own registry when a callee's `resolved_fn_sigs` entry is absent
  (HOLE B). Fixed via two new `NoduleImports` marker sets (`cross_phylum_traits`/`cross_phylum_fns`)
  plus a shared `fn_sig_names_a_concrete_type` core, refusing never-silently only when the callee is
  genuinely cross-phylum and its un-re-homed signature names a concrete type beyond its own generic
  params.
- **PR #1511 (`343276e2`, the 4th/final site)** — `check_path`'s fn-as-first-class-VALUE synthesis
  (`let f = foreignFn`, a HOF argument) re-resolved the surface signature fresh against the caller's
  own registry, bypassing both the re-homed baked entry and the `cross_phylum_fns` marker the three
  call-site guards already closed. Mirrors `check_app`'s value-position pattern (DRY).

An exhaustive carrier×position enumeration (ctor field / call / generic-call / trait-method-call /
value-ref) confirms these four sites are the complete reachable set for this collapse class — see
**DN-113 §7.1** (v1-limitation disclosure, added this integration close-out). The general fix (full
re-homing of every foreign trait/fn signature `TypeRef`, replacing the four conservative refusals
with full acceptance) is deferred, never-silently, tracked as **M-1076** (new, this close-out).
`cargo test -p mycelium-l1`: 1312 tests, 0 failures after the final cycle; `cargo fmt`/`clippy -D
warnings` clean throughout. Guarantee: `Empirical` (checked by the regression corpora in
`crates/mycelium-l1/tests/cross_phylum.rs`; no discharged theorem backs the refusal predicates).
`tools/github/issues.yaml`: **M-1060 → `done`** (this close-out; see its own `landed_basis`).

### fix(docsite): real light/dark toggle + lang-ref callout readability (2026-07-11)

`mycelium-doc`'s `theme.rs` already had a correct, tested light/dark toggle; the reported "dark-only,
no working switch" and washed-out callout boxes were in `scripts/docsite.sh`'s three hand-rolled
pages (the docsite landing page, `lang-ref/index.html`, and the api-index HTML wrapper) — what
`publish-docs.yml` actually deploys. Root cause: those pages carried only an automatic
`prefers-color-scheme` media query (no manual toggle) and hard-coded light-mint/peach callout
backgrounds that the dark override never touched, while body text flipped to near-white —
washed-out near-white-on-unchanged-light-box. Fix: a shared themed CSS/JS set (mirrors
`mycelium_doc::theme`'s design) with custom properties, both `prefers-color-scheme` queries,
`:root[data-theme]` overrides, and a real toggle button persisting to `localStorage` (no external
deps), applied to all three hand-rolled pages, plus a never-silent self-check in `docsite.sh`
asserting the toggle/overrides are present and no hard-coded mint/peach hex leaks back in. Verified:
`cargo fmt`/`clippy -D warnings`/`test -p mycelium-doc` green (137 unit + 3 integration tests);
rebuilt the site and drove real headless Chrome via CDP to confirm both themes render with correct
contrast and the manual toggle overrides the OS preference.

### fix(ci): publish rustdoc index (fix `/rustdoc/` 404) + bump Node to 24 (2026-07-11)

Root cause of the live-site `/rustdoc/index.html` 404 (verified locally): `cargo doc --workspace
--no-deps` never emits a root `target/doc/index.html` for a multi-crate workspace, only
per-crate `index.html`s; `scripts/docsite.sh` symlinked `target/doc` straight into the site's
`rustdoc/` dir with nothing at its root. Fix: `scripts/docsite.sh` now writes a small meta-refresh
landing page at `rustdoc/index.html` redirecting to `mycelium_core/index.html` (the Ring-0
kernel-adjacent crate the wiki's own API-Reference already cites), falling back to any other built
crate dir if absent, skipping with an explicit message (never a silent 404) if nothing doc'd at all.
Also bumps Node 22 → 24 (maintainer-directed) in `.github/workflows/checks.yml`'s `setup-node` step
and the devcontainer's baked Node — the `Node ≥ 20` compatibility *floors* in `markdown.sh`/
`install-tools.sh` are left untouched (those are minimums, not pins). Verified locally
(`cargo doc --workspace --no-deps` clean; `target/docsite/rustdoc/index.html` resolves through the
existing symlink post-fix; `docsite.sh` re-run twice confirms idempotency); the live Pages 404 itself
clears only on the next `publish-docs.yml` manual dispatch against `main`.

### chore(issues): file M-1073/1074/1075 — security-scanning + hashing-accel + Rust-baseline backlog (2026-07-11)

Three backlog issues filed in `tools/github/issues.yaml` (maintainer directive, gap-close-run), each
with explicit user stories + a Definition of Done (house rule #6): **M-1073** (value-semantics-aware
security scanning + exploit hardening for Mycelium programs, filed forward under the existing E22-1
Security Scanning Toolkit epic); **M-1074** (extensible CPU+GPU acceleration for identity
hashing/crypto, standalone — no live epic fits, same pattern as the adjacent M-832/M-1014
desktop-held tracks); **M-1075** (standard Rust-kernel security scans — `cargo-audit`/`cargo-deny`/
unsafe-audit — to establish and maintain a hardened baseline, near-term hygiene not backlog). FLAG:
no live epic covers ongoing Rust-kernel security hygiene (E22-1 scopes Mycelium *programs*, not
kernel supply-chain/unsafe hygiene; M-678 is `done` and narrower) — filed M-1074/M-1075 standalone
rather than force-fitting or minting a speculative epic. `issues.yaml` validated (583 issues at filing
time, no dupes); `doc_refs` check green.

### feat(l1): DN-113 v1 cross-phylum import/resolution subsystem — M-1060 Phase 1 (2026-07-11)

Implements the core check-time mechanism DN-113 ratifies: a `::` phylum-boundary `use dep::a.b.Item`
reference (new `Tok::ColonColon`), the additive `Phyla`/`ResolvedPhylum` dependency-set type, and
`check_phylum_with_deps` layered over the existing `Exports`/`resolve_imports`/`PhylumEnv::link`
machinery (one added phylum-qualifier key dimension, no second linker — DRY, DN-113 §7/§9.6).
Extends DN-112 Rank 1 home-qualified type identity across the phylum boundary via
`qualify_cross_phylum`/`qualify_ty_cross_phylum`: a foreign type's `DataInfo::home` and a
dependency's already-baked pub-fn signatures (`resolved_fn_sigs`) are re-homed at merge time, so a
same-named local type can never satisfy a foreign dependency's type — required for soundness (the
same baked-signature mechanism that closed the M-1036 ctor-seal exploit intra-phylum), not optional
hardening. Four residual cross-phylum type-identity-collapse holes this landing's own adversarial
audit surfaced were closed in follow-up fix cycles (see the "M-1060 cross-phylum type-identity
soundness closure" entry above). `tools/github/issues.yaml`: **M-1060** minted → now `done` (see its
`landed_basis`). Guarantee: `Empirical` (checked by the landed regression corpus); `Declared` for the
deferred v1 scope items (separate compilation, re-export, glob cross-phylum `use`, version ranges).

### feat(transpile): P4/P5 signed-int + `usize`/`isize`/`char` numeric-type-idiom emit (2026-07-11)

DN-99 §8 ENB-6 / M-1029 / ADR-028: `map_type` now maps `i8`/`i16`/`i32`/`i64`/`i128`, `isize`/
`usize`, and `char` to their ratified `Binary{N}` idiom instead of gapping unconditionally
(ADR-028: `Binary` is sign-free — a signed integer denotes the same `Binary{N}` value/content-address
as its unsigned counterpart; signedness lives entirely in which op is applied). `map.rs`: `i8..i128`
→ matching-width `Binary{N}`; `isize`/`usize` → `Binary{64}` (a canonicalized, FLAGged
platform-width default); `char` → `Binary{32}` (codepoint idiom). `emit.rs`: `MappedSig` gains
`signed_param_names` (recorded off the original `syn::Type` before `map_type` erases signedness); a
signed param's `TypeEnv` entry carries an internal never-emitted marker that only the signed-width
helpers understand, so every other `TypeEnv` consumer stays opaque to it by construction (a
signed-source widen cast still gaps honestly instead of silently zero-extending).
`Expr::Binary`/`Expr::Unary` gain signed-gated arms routing to `add_s`/`sub_s`/`mul_s`/`neg_s`/
`lt_s`/`eq` (landed ops only, no new kernel primitives) — mirrors the existing unsigned operand-gate
pattern. Verify-first (mitigation #14): confirmed `i8..i128`/`isize`/`usize`/`char` were unconditional
`GapReason` refusals before this change, and confirmed empirically against a real
`target/debug/myc-check` run that the new signed prims resolve as bare-call prims with no import.
`cargo fmt`/`clippy -D warnings`/`cargo test -p mycelium-transpile`: 78 lib tests +
`guard_hole_census` + doctests, 0 failed.

### docs(dn): DN-123 records/named-fields surface lever (P2) — design + ranked recommendation (2026-07-11)

- **DN-123 (Draft)** — `docs/notes/DN-123-Records-Named-Fields-Surface-Lever.md`. Works DN-121's P2
  lever (records / named-field surface, Struct 80/10%) forward to a ranked recommendation; enacts and
  ratifies nothing (house rule #3, append-only — the maintainer ratifies).
- **Verify-first correction (mitigation #14).** Records are **already substantially supported at the
  `checked_fraction` level**: the transpiler carries a field-name↔index map (`StructLayout`), already
  desugars Rust struct literals / field-projection / struct-update to positional `Data`, and emits
  named-field structs positionally with names **recorded** as a never-silent `NamedFieldDrop` sub-gap
  (`crates/mycelium-transpile/src/emit.rs`). The blocker is an unmapped field TYPE, not named-fieldness;
  the genuine residual is **faithfulness** (dropped names) plus the **self-hosted `.myc` surface**
  (the DN-119 L3-G1 struct-pattern grammar gap).
- **Ranked recommendation.** Option A — a **mechanically-lowering sugar** over positional `Ctor`/`Data`
  plus the name↔index map (per DN-106's ratified GP1/GP2 — the gap-closure default is the sugar, not a
  kernel primitive); Option C (sugar plus inert `field_names` metadata) as a YAGNI-gated faithfulness
  upgrade; Option B (first-class named-field kernel variant) **rejected** (KC-3, value-semantic
  positional design). Identity stays positional/structural (ADR-003) — names never in the hash.
- **Adversarial open questions** OQ-1..5: content-addressed identity vs field ORDER (canonicalize-to-
  declaration-order) and NAMES (off the hash); cross-phylum name metadata (DN-112/DN-113/M-1060);
  functional-update spread affine treatment (M-919); struct-pattern exhaustiveness plus DN-104 seal.
- Also appends the DN-123 `docs/Doc-Index.md` row. `Empirical` where read against dev tip `46006994`;
  `Declared` for the proposed design (VR-5). FLAGged for integration close-out: DN-99 register Struct
  rows, DN-119 exclusion-row annotation, M-876 `doc_refs` → DN-123 (integration/orchestrator-owned).

### docs(notes): file DN-119/120/121/122 (Draft) + DN-32/33 → DN-35 §5 forward cross-refs (2026-07-11)

Four Draft Design Notes filed for maintainer ratification, part of the ongoing gap-close-run wave
(design-phase; docs are the product):

- **DN-119** — L3 Comprehensive Surface Expressibility: Scoping, Reframe, and Phased Plan. Scopes the
  "implement L3 comprehensively" directive: the L3 grammar is substantially complete already
  (register-lag corrections to DN-99); isolates the genuine ~7-class grammar residual from the bulk of
  the gap mass that is misattributed to L3 (actually kernel/runtime/transpiler work) and from the
  deliberate-exclusion set (`&mut`, unbounded `loop`, shared mutability, silent casts) that must not
  get L3 grammar. Recommends reframing "full native capability" and a lane-tagged phased plan.
- **DN-120** — Content-Addressed Identity vs. Temporary-Copy Mutation: Solved-by-Design (Verdict). A
  verdict note (no new mechanism — mitigation #14) closing ADR-003's disclosed residual: DN-35 §5 is
  already the content-address-coherence answer (rc==1 reuse gate, weak-intern evict-or-copy). Corrects
  the landed/open boundary against the tree — the rc==1 detection gate is landed and `Exact`
  (`mycelium-std-runtime/src/rc.rs`); the reuse-write itself stays `Declared`, tracked as DN-35's own
  E12 Increment 3. Distinguishes this from the unrelated DN-109 §6.1/D7 `&mut`-aliasing problem
  (DN-118's lane).
- **DN-121** — The Type-Vocabulary Lever: Scoping the Dominant `checked_fraction` Class. Scopes the
  ~40% type-coverage gap class; corrects "missing kernel type-vocabulary" to "type-reference closure
  on the existing kernel" — most of the class closes via std ADTs + idiom, not a new kernel `Ty`
  variant. Phased, leverage-ranked build plan; flags the outstanding Phase-0 re-measure.
- **DN-122** — External-Trait Impls Across the Home Boundary. Recommends a foreign-trait-import
  mechanism (closure-extended coherence on DN-112 home-qualified identity, reusing DN-113's
  cross-phylum `use`) for the M-876 external-trait-impl gap (DN-121's top lever, ~15%/119 gaps); zero
  L0/kernel/runtime change. Explicitly downstream of DN-112/DN-113 (both Accepted-not-Enacted).

All four are **Draft**, pending maintainer ratification (house rule #3, append-only); none edits
`crates/**`, `lib/**`, or `issues.yaml`. Added forward cross-reference notes (append-only, no
normative text changed) at **DN-32 §2.2** and **DN-33 §6** pointing to **DN-35 §5** (the
content-address-coherence answer) and **DN-120** (the verdict). `Doc-Index.md` rows added for all
four notes.

### fix(l1): M-1036 ctor-seal capability-gate closure — nodule-qualified type identity (DN-112 Rank 1) (2026-07-11)

`claude/leaf/m1036-ctor-seal` (5 cycles of self-verification; 4 soundness holes found and closed
during the leaf's own audit, both final audit arguments proven sound before landing — mitigation #14
(never rubber-stamped)). Closes the M-1027/DN-104 `priv` constructor-seal bypass DN-112 Rank 1
designed: types/constructors were resolved by **bare name, re-resolved in the calling nodule's own
scope**, so a foreign nodule could impersonate a sealed type by declaring its own same-named local
(unsealed) type. `Ty::Data` now carries a **nodule-qualified identity** — `DataInfo::home` (the
declaring nodule's dot-joined path, or `PRELUDE_HOME` for the single reserved builtin/prelude home,
`checkty.rs`), stamped once at first registration and never re-derived per import — with
`qualify_type_name`/`nodule_home` (`checkty.rs`) doing the qualification at `resolve_ty`.

- **The differential is flipped, not just extended:** `crates/mycelium-l1/tests/ctor_seal.rs`'s
  `known_gap_a_same_named_local_shadow_type_bypasses_the_seal` is renamed
  `a_same_named_local_shadow_type_no_longer_bypasses_the_seal` and now asserts the refusal (was
  asserting the exploit's `Ok`). 26 tests in `ctor_seal.rs`, all green.
- **Two CRITICAL soundness holes found and closed during the leaf's own adversarial audit** (not by
  an external reviewer — self-verification per mitigation #14): (1) a generic-callee variant that
  reopened the same-nodule-shadow bypass through an unrelated type parameter; (2) a
  shadow-plus-legitimate-cross-nodule-reach pattern match that let the checker bind a pattern's
  field type to the WRONG (locally-shadowed) `DataInfo` — fixed by
  `lookup_data_home_checked` (`checkty.rs`), a **conservative refuse-on-home-mismatch** variant of
  `lookup_data` used at the pattern-normalization call sites (`normalize_pattern`, shared by
  `Cx::check_pattern` at check-time and the elaborator).
- **Four-site closure of the enumerated residual set:** `crate::mono` (`emit_data`/ctor emission),
  the `resolve_ty` round-trip, `crate::fuse`, and the check-phase/`crate::elab` sites are each
  audited against the same home-check discipline; `crate::decision`'s `compile_rows` and
  `crate::usefulness`'s `signature` are audited and confirmed **not exploitable** for a
  wrong-value silent accept via a "validated-by-construction" argument (every `Pat::Ctor` reaching
  these fns was already home-checked by `normalize_pattern` upstream) rather than routed through
  the checked variant, documented in-line (`decision.rs`, `usefulness.rs`).
- **Honest, disclosed residuals (not silently closed — recorded as follow-up issues below):** (a)
  the shipped fix is the **conservative (b) closure** — refuse-on-home-mismatch — not the full (a)
  **correct-home resolution** (per-import-provenance-scoped lookup against the foreign type's own
  `DataInfo`), which needs a materially larger `crate::mono` change (per-import registries, not one
  merged bare-keyed map) and so is sound but over-refuses a legitimate shadow+cross-nodule-reach
  program; (b) the pre-existing M-919/DN-71 affine-tracker gap (a `Handle`-typed local referenced
  and destructured twice independently) is confirmed, by an equivalent hand-written non-sugar `fn`
  fixture, to be a pre-existing limitation, not a regression introduced here; (c) a LOW-severity doc
  imprecision in `decision.rs`'s `compile_rows` audit comment (its Point 2 argument is less precise
  than Point 1's induction, which is the load-bearing sound argument) was flagged by a verify pass
  and is left as a follow-up doc-polish item rather than blocking the landing.
- **Verified:** `cargo test -p mycelium-l1` — 565 lib tests + 26 `ctor_seal.rs` tests, all green
  (post pull-down merge with `dev`'s facility Stage 3/DN-117 affine work, confirming both mechanisms
  coexist — see the merge commit); `cargo fmt` / `clippy -D warnings -p mycelium-l1` clean.
- **`tools/github/issues.yaml`:** M-1036 → `done`. Three follow-up issues filed for the disclosed
  residuals above (**M-1070**, **M-1071**, **M-1072** — see their entries). DN-112 stays `Accepted`
  (design ratification; the implementation landing does not itself flip a design note past
  `Accepted` without a dedicated Enacted review — house rule #3).

### chore(docs): DN-118 P1 closure-emit — changelog + Doc-Index close-out (landed via PR #1500) (2026-07-11)

`docs/Doc-Index.md` gains a **DN-118** row (was missing a Doc-Index entry despite landing on `dev`
via PR #1500, `claude/leaf/dn118-p1-closure-emit`). **DN-118 — Closure-to-Value-Semantics
Transpiler Enabler and Native-Conformance Contract** ratifies **Option A**: the Rust→Mycelium
transpiler's closure gap is a **closure-EMIT gap**, not a defunctionalization gap (defunctionalization
of env-capturing closures is already implemented in the language, RFC-0024 §4A/M-704, `done`), so
`crates/mycelium-transpile` emits the Mycelium `lambda` surface and lets `mono.rs`'s whole-program
`ClosureSpecialization` resolve captures — no transpiler-side defunctionalizer is built (KC-3/DRY).
A closure that syntactically mutates a captured binding in place (FnMut/`&mut`-style — `syn` carries
no borrowck facts) is conservatively FLAGGED (`Category::Closure`), never auto-emitted (DN-109
D5/D7 ratchet); Phase 1 is single-parameter-closures-only (a verify-first narrowing of the note's
own original plan). P1 (the emit pass, `crates/mycelium-transpile/src/{emit,visit,gap}.rs` +
`src/tests/emit.rs`) lands with this note; P2 (a future RFC-0018-framework `@value_closures`
native-conformance contract) and P3 (borrowck-backed capture-mutation checking) are scoped, not
built. `Empirical` for claims checked against the real toolchain (myc-check-verified emission);
`Declared` for the general semantic-faithfulness claim and the unbuilt P2/P3 scope (VR-5). Status:
**Accepted**, explicitly NOT Enacted (house rule #3).

### feat(l1): M-1054 native metaprogramming facility Stage 1b + Stage 2 — check-phase accept, `Elab::app` dispatch, def-site resolution (DN-116/DN-115, M-1069) (2026-07-11)

Two further staged increments of the DN-110 §5-A Rank-1 facility, on top of the landed Stage 0+1
(above). **M-1054 stays `status:in-progress`** — Stage 3 (affine-over-substituted-`Expr`, DN-117)
continues building separately and is **not** part of this release.

- **Stage 1b (PR #1434, `claude/leaf/m1054-stage1b-check-accept`) — end-to-end reachability.**
  `Cx::check_sugar_call` (`checkty.rs`) starts **accepting** a well-formed value-parametric sugar
  call instead of unconditionally refusing it, typed via **Option B** — a monomorphic rule's RHS
  result type is fixed at *definition* (`infer_expr_rule_rhs_type`, shared with the existing
  def-time RHS validator, DRY); `Elab::app` gains a new last-resort §5.2 dispatch branch to the
  existing, unmodified Stage 1 expansion machinery. Two never-silent gates bound the accepted
  fragment: a Stage 2 (OQ-H1) free-identifier gate, and a Stage 3 (OQ-H4) affine gate refusing any
  value parameter that *is or structurally contains* `Substrate` — an adversarial-verify finding
  during this leaf's own review caught a **composite/nested**-affine false-accept (a `Data` ctor
  wrapping `Substrate`) the original top-level-only check missed; fixed by a recursive structural
  walk (`ty_structurally_contains_substrate`), regression-pinned. Filed as design note **DN-116**
  (originally DN-114 — renumbered at this integration close-out; see below).
- **Stage 2 (PR #1435, `claude/leaf/m1054-stage2-dn-scoping`) — def-site resolution, single-nodule
  fragment (DN-115, `status:done`, M-1069).** Finds single-nodule def-site resolution /
  referential transparency for a sugar rule's free RHS identifiers already holds **by construction**
  since Stage 1b (Pass-1 elaboration against the def-site env) — Stage 2's real work is *proving it
  non-vacuously on the real elaborator* plus closing two narrow check-phase gate-correctness gaps:
  G1 (the VSA/float-prim RHS dispatch sets were over-refused as free identifiers) and G2 (a nullary
  `lower`-rule referenced in value position was over-accepted). The cross-nodule/phylum boundary
  (Stage 4) stays refused (DN-113/M-1060's job). Design-reasoner-ratified (maintainer-delegated,
  orchestrator-selected on the merits, 2026-07-11); implementation tracked as **M-1069**.
- **DN-114 ID collision, resolved (M-1054's own PM close-out, PR #1436, then this integration
  close-out).** The Stage-1b note was filed as `DN-114`, colliding with the unrelated, separately
  numbered `docs/notes/DN-114-Validated-Narrative-Generation.md` (E40-1, kept as DN-114 per
  maintainer directive). Renumbered to **DN-116** at this close-out (file moved; docs-only inbound
  refs — DN-115, DN-117 — repointed in the same pass). `crates/mycelium-l1/` source comments
  still citing "DN-114" are a deliberately deferred residual (a concurrent Stage-3 agent is actively
  editing that crate) — flagged, not silent.
- **Verified:** `cargo test -p mycelium-l1` green at each leaf (535 passed, 1 ignored, 0 failed at
  Stage 1b); `clippy -D warnings` / `fmt` clean; dual-oracle capture-safety + non-vacuity controls
  for every gate (each independently verified to genuinely fail when its mechanism is disabled).
  Tags stay at their checked strength — Stage 1b/2 reachability `Empirical` on the real elaborator
  path; Stage 3 (full affine re-check) and cross-nodule resolution stay `Declared` (VR-5).

### fix(l1): M-1054 native metaprogramming facility Stage 3 — affine soundness over the substituted `Expr`, plus a critical pattern-ctor false-accept fix (DN-117 Accepted) (2026-07-11)

Lands Stage 3 of the DN-110 §5-A Rank-1 facility (`claude/leaf/m1054-stage3-affine`, pulled down
against `dev` per mitigation #6). **M-1054 stays `status:in-progress`** — Stage 4 (cross-nodule
resolution, DN-113/M-1060) is not part of this release.

- **Stage 3 mechanism (`checkty.rs`) — accept-linear, refuse-duplicated.** Replaces the Stage 1b/2
  wholesale refuse-all-affine gate with a precise check: the M-919 affine `Tracker` now walks the
  **substituted `Expr`** (each type-checked argument spliced at every RHS occurrence of its value
  param), at check time, inside `Cx::check_sugar_call` — a check-time-only artifact; `Elab::app`'s
  dispatch and `elab.rs::sugar_expand` are untouched. The prior conservative structural
  over-approximations (`rhs_first_affine_binding`/`expr_is_structurally_affine`) are replaced by
  the real per-argument linear-use walk; `ty_structurally_contains_substrate` is demoted from
  decision to trigger. A dropped affine value param or RHS-local affine binding is **ACCEPT**
  (runtime-backstopped, M-904), correcting the earlier defensive over-refusal.
- **CRITICAL fix, same leaf — pattern/ctor-name-collision false-accept.** Adversarial review found
  `Cx::stage3_substitute_pattern` left a match-arm `Pattern::Ident` binder **unrenamed** whenever
  its spelling coincided with an unrelated registered nullary constructor, on the theory this was
  sound. It was not: when the identifier was genuinely a binder, leaving it unrenamed skipped this
  same walk's own capture-avoidance discipline, letting a spliced argument's free variable of the
  same spelling be captured by the pattern binder instead of the caller's value — hiding a real
  double-consume from the affine tracker. Confirmed reproducible both ways (false `Ok`-accept
  pre-fix, correct `Err`-refuse post-fix, via a `git stash`-revert of just the fix). Fixed:
  `stage3_substitute_pattern`/`stage3_substitute_arm` now return `Result<_, CheckError>`, refusing
  the whole sugar call with a never-silent diagnostic (G2) in the ambiguous case rather than
  guessing — a conservative false-REFUSE, not the unsound false-ACCEPT it replaces. The full close
  (scrutinee-type-directed disambiguation) stays open (FLAG-pattern-ctor-collision). Two new tests
  in `crates/mycelium-l1/src/tests/affine_stage3.rs` (the exploit, now refused; a non-collision
  control, still accepted — no over-refusal), on top of the leaf's original 12-test §5 corpus.
- **DN-117 — Accepted** (`docs/notes/DN-117-M1054-Stage3-Affine-Over-Substituted-Expr.md`,
  delegated ratification, mirroring the DN-115/Stage-2 precedent — ratifies the §1–§7 design
  decisions, **NOT `Enacted`**, house rule #3) with a same-day append-only Errata (Ratification
  point 11) recording the pattern-ctor false-accept finding + fix above.
- **DN-114 mycelium-l1 residual, resolved.** The prior Stage-1b/2 entry above deferred repointing
  `crates/mycelium-l1/` source/test comments citing "DN-114" (meaning the renumbered DN-116
  Stage-1b note) while this Stage-3 leaf was actively editing that crate. Closed in this landing's
  own `dev` pull-down merge: all 22 occurrences across `elab.rs`, `checkty.rs`,
  `src/tests/checkty.rs`, `src/tests/defsite_resolution_stage2.rs`,
  `src/tests/reachability_stage1b.rs`, and `src/tests/facility_stage1_hygiene.rs` repointed to
  **DN-116**, verified by an empty `grep -rn DN-114 crates/mycelium-l1/`.
- **Verified:** `cargo test -p mycelium-l1` — 559 lib tests (0 failed, 1 ignored) plus every
  integration target green, after this landing's `dev` pull-down merge (no regression);
  `clippy -D warnings` / `fmt` clean. Tags stay at their checked strength — the double-consume
  upper bound over the real substituted `Expr` moves to `Empirical` (checked by the §5/§7 +
  errata corpus); no `Proven` claim anywhere (VR-5). The pattern-ctor disambiguation residual and
  the pre-existing prior-handle-alias gap (inherited from M-919, not a Stage-3 regression) stay
  honestly open, not silently closed.

### docs(docs-access): fast-follows — real research/CONTRIBUTING PDF ingestion, asset automation, live dark theme, presentable READMEs (2026-07-11)

Four follow-on PRs completing the documentation-access initiative (above) into a shippable,
themed docsite with real generated assets.

- **Spec-manifest fix (PR #1488, `fix/docgen-research-spec-manifests`)** — the NotebookLM
  research/spec manifests were emitting placeholder text instead of the real PDFs; fixed so every
  cluster renders its actual source content.
- **Research + CONTRIBUTING ingestion (PR #1490, `claude/fix-docgen-research-contributing-ingestion`)**
  — `build --manifest` now ingests all 7 `research/` PDFs plus `CONTRIBUTING.md` into the NotebookLM
  research-PDF cluster and the curated book, closing the prior partial-ingestion gap.
- **Docs asset automation (PR #1489, `feat(docs): docs asset automation`)** — `just docs-assets`
  (`scripts/docs-assets.sh`) drives the full capture → optimize → replace-in-place → prune workflow
  via Playwright (`scripts/docs-assets/capture.mjs`): builds `target/docsite/`, serves it locally,
  captures the committed screenshot set in both themes, optimizes PNGs (`oxipng`/`pngquant`,
  skip-graceful), and prunes any `docs/assets/*` file no longer referenced. `scripts/checks/docs-assets.sh`
  (wired into `just check`) is the lightweight, browser-free drift gate — referenced-but-missing or
  present-but-orphaned, the same discipline as the `api-index`/`tero-index` gates.
- **Real light/dark docsite theme (PR #1491, `feat(docgen): real light/dark docsite theme`)** — every
  page now honours the reader's OS `prefers-color-scheme` by default, with a persisted `data-theme`
  toggle overriding it in both directions; `crate::theme::READING_CSS` (`crates/mycelium-doc/src/theme.rs`)
  carries a real `@media (prefers-color-scheme: dark)` rule for the corpus/book pages (asserted by
  `the_emitted_css_ships_a_real_prefers_color_scheme_dark_rule`), and `scripts/docsite.sh`'s own
  hand-rolled pages carry a matching `DOCSITE_DARK_CSS`. The `-dark` screenshots are now genuine
  renders (`page.emulateMedia`), not a capture-time stylesheet override — `docs/guide/docsite-preview.md`'s
  prior "dark not real" disclaimer no longer applies.
- **Presentable READMEs (PR #1492, `docs(readme): compelling, honest README + guide showcase`)** —
  a rewritten root `README.md` (verified `.myc` examples, docsite screenshots) and crate
  cross-links to the docsite preview, with a corrected crate count.
- **Advances:** documentation access + publishing. **Verified:** `scripts/checks/docs-assets.sh`
  green (no drift); `myc-doc lint` green on the real corpus; markdown gate clean; the real
  light/dark theme asserted by a dedicated `cargo test -p mycelium-doc` case.

### docs(docs-access): documentation-access & generation initiative — Claude Project prompt · themed+highlighted site + Pages/release publishing · NotebookLM export · validated narrative generation (DN-114) (2026-07-11)

A cohesive documentation-access layer so the corpus is explorable, learnable, and shippable — built by
**extending the existing ratified `myc-doc` pipeline**, not introducing a foreign generator (which would
supersede `docs/spec/Narrative-Authoring-Pipeline.md` and fight the non-YAML pipe-table headers).

- **Claude.ai Project system prompt** (`docs/claude-project-system-prompt.md`, `Declared` authoring aid) —
  a copy-pasteable prompt grounding a Claude Web chat assistant in the charter, the guarantee lattice
  (`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`, VR-5), never-silent/append-only/anti-sycophancy house rules,
  the fungal lexicon with honest keyword-status, the ADR/RFC/DN taxonomy + grounding labels, and a
  navigation map. Modeled on `.claude/agent-context.md`.
- **`mycelium-doc` renderer overhaul** (C1/C2/C4) — a comfortable reading theme (`src/theme.rs`: a
  reviewed design-token system whose palette is the guarantee lattice made visual, serif reading column,
  light+dark, scroll-safe tables, a short-label collapsible **semantic+logical sidebar tree** (Topics
  from the book-manifest spine, then a by-type appendix) + ToC + search), **lexer-span syntax highlighting**
  (`src/highlight.rs` via `mycelium-l1` — **no new dep**; `Empirical/Declared` lexical, never-silent
  plain-text fallback), inline-markdown rendering (bold/em/inline-code/links in prose, headings, and
  table cells; code blocks untouched), a **differential re-emit cache** (`src/cache.rs`, id-hash keyed),
  a `build --manifest` scoped emission, and a **Typst PDF print-legibility pass** (code ~0.82× body in a
  hairline box, light-only; every emitted `.typ` compiles clean — verified on the full corpus). Dual-projection
  parity + all 8 §4.1 `doc_lint` checks stay green.
- **Publishing** (C3) — `.github/workflows/publish-docs.yml` now deploys the full themed docsite
  (corpus, agent index, and rustdoc) to GitHub Pages (manual-dispatch), and `scripts/dist/package-docs.sh` ships a
  portable rendered-docs bundle **alongside** the release artifact (`just package-release`). `.gitattributes`
  intentionally unchanged: GitHub-Linguist `.myc` highlighting stays hard-gated (adoption bar), and the
  repo's own rule against a dishonest `.myc`→Rust mapping is honored — site highlighting deploys via Pages.
- **NotebookLM export** (B) — 7 language-focused, book-manifest-shaped clusters (`tools/docgen/notebooklm/`),
  a shared resolver, `scripts/docs/export-notebooklm-pdfs.sh` + `just notebooklm-pdfs` (ratified Typst path,
  never-silent Markdown fallback), each cluster under NotebookLM's 500k-word/source cap.
- **Validated narrative generation** — **DN-114 (Draft)** designs a sentence-level **faithfulness oracle**
  that refines Narrative-Authoring-Pipeline §4/§4.1/§6, so `gen-book`/`gen-manual` (M-363) can *generate*
  the interpretive read while never hallucinating; the `tools/llm-harness/narrate/` harness + `/gen-book`
  skill implement it (constrained-from-facts generation, resolvable `doc_refs`, an adversarial
  claim-grounding gate, a `validated_fraction`, commit-only-validated with never-silent drops, idempotent
  content-addressed caching). Output is always `Empirical`/`Declared`; the shipped checker is honestly a
  deterministic lexical stand-in (§7). Epic **E40-1** + **M-1061..M-1068** decompose the follow-on wave.
- **Advances:** documentation access + the M-363 pipeline. **Verified:** `cargo test -p mycelium-doc`
  (129 unit + 3 integration) green + `myc-doc lint` green on the real corpus (335 docs / 14,885 nodes, all 8 checks); the
  `narrate` harness 46 tests green, demo `validated_fraction = 1.0`, negative-control drops an injected
  hallucination; `doc_refs_check` OK; markdown gate clean; the themed site screenshotted (light+dark) in
  headless Chromium. Tags at supportable strength (renderer heuristics `Empirical/Declared`; DN-114 +
  generated prose `Declared` pending ratification/review — VR-5).

### chore(docgen): verify + automate diff-friendly index regen (2026-07-11)

PR #1421. Maintainer directive: audit the five doc-gen generators (api-index, tero-index,
sugar-index, lib-index, editor grammars) for diff-friendly (update-in-place, not full-file-rewrite)
output. **Audit result: all 5 already diff-friendly, no fixes needed** — stable sort keys, pretty and
stable JSON, no volatile timestamp/run-id/host-path field in any generator or committed artifact
(grepped `datetime|timestamp|generated[_-]?at|utcnow|SystemTime::now|chrono::`, zero hits). Verified
empirically per generator: a tiny real source edit regenerates a small, localized diff, then reverts
cleanly (`git status --short` clean after every probe-and-revert) — no generator or committed
artifact was actually changed by this PR. **Adds `just docs-regen-all`** — the single entrypoint
running all five `*-gen` recipes (`docs-index`, `tero-index-gen`, `sugar-index-gen`,
`lib-index-gen`, `grammar-gen`) in one command; verified zero-drift from a clean tree and all five
drift gates green afterward. Used by this session's own index regeneration (see the
`chore(regen)` entry).

### chore(integration): gap-close-run batch — dev to integration close-out (2026-07-11)

Integration-tier close-out for the whole gap-close-run session (mitigation #14 throughout — every
status flip below is checked against the merged code/PRs, never rubber-stamped). Reconciles the
CHANGELOG (this entry and the four below), `docs/Doc-Index.md`, `tools/github/issues.yaml`, the
committed indices, and the DN-113 doc-status header — the shared collision surface every leaf in
this batch correctly FLAGGED up rather than touched directly.

- **`docs/notes/DN-113-Cross-Phylum-Import-Resolution-Subsystem.md` Status header converted to the
  recognized `| **Status** | ... |` table-row form** (was a blockquote `> **Status:** ...`, which
  `scripts/doc_status_check.py`'s `STATUS_ROW` regex does not match — a genuine gate gap, not a
  content change). Content is preserved verbatim, only the container changes (a `| Field | Value |`
  table matching DN-112's own style). `python3 scripts/doc_status_check.py` now exits 0.
- **`tools/github/issues.yaml` reconciled (append-only, mitigation #2, validated + deduped — 567
  issues, zero duplicate ids after every edit below).** M-1054 gains a Stage-0+Stage-1-landed
  addendum (status stays `in-progress` — the facility is real, checked progress but not yet wired
  into ordinary elaboration; see the M-1054 entry below); M-1051 gains an Increment-3-landed
  addendum (status stays `in-progress` — Increment-2 CLI/LSP surfacing is the sole remaining gap);
  M-1024 gains a CLI-linker-reconciliation addendum (status stays `in-progress` — AOT parity is
  still the tracked residual); **M-1056 status corrected `todo → done`** (the `/native-translate`
  skill is authored and was applied in production to classify all 47 sugar-index rows, a stronger
  validation than the DoD's literal minimum — see the skill entry below); M-1058 gains a
  native-strategy-population addendum (status stays `done`, superseding its own "remains deferred"
  note).
- **Draft DNs surfaced for maintainer ratification: none outstanding from this batch.** DN-112 and
  DN-113 are already `Accepted` (delegated ratification, 2026-07-10/11 — see the entry below);
  DN-110 and DN-111 are already `Accepted` (prior sessions). No new Draft design note was authored
  in this close-out.
- **Periodic docs refreshed:** `docs/CURRENT-STATE.md` gains a bullet recording the facility
  Stage 0+1 landing, the `reveal` Increment-3 landing, the M-1024 CLI reconciliation, and the
  native-strategy population (superseding its own "population is bounded follow-up" line); append-
  only — no decision status moves to `Enacted` by this refresh (house rule #3), the facility stays
  `Accepted`-not-`Enacted` throughout.
- **Gate:** `just check-canary` (every always-on gate + Tier-0 change-scoped tests) — the base-crate
  (`mycelium-l1`) heavy Tier-1/durability sweep (full reverse-dep tests, proptest HIGH, mutants/fuzz,
  VSA/GPU) is desktop-held per the cloud-session policy, not re-run here; the batch's own 526
  `mycelium-l1` lib tests are green (verified per-PR, see each entry below).

### feat(l1): M-1054 native metaprogramming facility Stage 0 + Stage 1 — recognition + production capture-avoidance (2026-07-11)

Two staged increments of the DN-110 §5-A Rank-1 facility (generalizing the landed `lower`/`derive`
term-level lowering framework to expression position) landed this session. **Neither closes the
epic — M-1054 stays `status:in-progress`** (see the issues.yaml addendum above); this is real,
checked progress toward its DoD, not a completion claim.

- **Stage 0 (PR #1426, `claude/leaf/m1054-stage0-facility-skeleton`) — call-site recognition + a
  value-parametric matcher skeleton, provably inert.** `ast.rs` gains `LowerDecl::value_params` (the
  rule's VALUE parameters, distinct from the existing type-parameter `params`). `checkty.rs` gains
  `Cx::check_sugar_call`, tried only as `check_app`'s last resort after every existing resolution
  path has missed: arity/type-matches a call against a registered `lower` rule's `value_params`
  (the same `resolve_ty` plus bidirectional `check` shape as an ordinary fn call, DRY), then refuses
  — unconditionally, every path a named refusal, adversarially verified to be structurally incapable
  of emitting an expansion. `elab.rs` gains `elaborate_lower_rule_with_args`, extending
  `elaborate_lower_rule` (which becomes a thin empty-args wrapper — every existing caller, including
  the external `tests/lower_derive.rs` harness, is untouched).
- **Stage 1 (PR #1429, `claude/leaf/m1054-stage1-facility-hygiene`, review fast-follow `26344f99`)
  — the first facility stage that actually emits an expansion.** `elab.rs`'s
  `elaborate_lower_rule_with_args` gains a two-pass hygienic expansion: pass 1 ordinary `Elab::expr`
  elaboration of the RHS with value params bound to their bare surface spelling; pass 2
  `sugar_expand` (a direct port of the M-1055 E1 prototype's `Expander::go`) freshens every
  RHS-introduced binder via `Elab::fresh` under a process-wide site-qualified namespace
  (`SUGAR_EXPANSION_SITE`, OQ-H5) and substitutes value-parameter placeholders with their matched
  argument `Node`s. New `src/tests/facility_stage1_hygiene.rs` (7 fixtures post-review, each checked
  3 ways — `alpha_eq` vs. an independent oracle, `Interpreter::eval` agreement, and a
  disable-freshening negative control proving the corpus is non-vacuous). Human maintainer plus Grok
  adversarial review posted 8 comments; all verified and replied to, actionable ones landed in
  `26344f99` (widened `sugar_expand`/`sugar_expand_alt` to `pub(crate)` and directly unit-tested on
  hand-built `Lam`/`Fix`/`FixGroup`/`Alt::Ctor` nodes — the full pipeline only ever reaches `Let`,
  a real coverage gap now closed; two new fixtures for shadowed-value-parameter and
  later-fresh-name collision).
- **Verdict: PASS — production capture-avoidance (A)+(B) on the real
  `elaborate_lower_rule_with_args` path moves `Declared → Empirical`, for that path specifically**
  (E1's own `Empirical` stays scoped to the test-only prototype it covered; (C) def-site resolution
  and (D) affine-on-expanded-L0 stay `Declared`, Stage 2/3, untouched). `cargo test -p mycelium-l1
  --lib`: 526 passed, 0 failed, 1 ignored (was 519 pre-Stage-1). `clippy -D warnings` / `fmt` clean.
- **Stage 1b residual, honestly flagged (not a full DoD close):** `checkty::Cx::check_sugar_call` is
  UNCHANGED by Stage 1 — it still refuses every recognized call site; **the facility is not yet
  wired into ordinary program-elaboration dispatch, so no ordinary `.myc` program can invoke it
  yet.** Also: `elaborate_lower_rule`'s synthetic-single-function-`Env` mechanism cannot elaborate a
  `lower` rule whose RHS is a lambda IIFE (pre-existing, independent of Stage 1); free RHS
  identifiers are left as bare `Var`s (Stage 2's OQ-H1 def-site-resolution job); `SUGAR_EXPANSION_SITE`
  is a process-wide monotonic counter, not bit-reproducible across separate process runs (does not
  affect alpha-equivalence correctness). DN-110 stays `Accepted`, NOT `Enacted`; no tag upgraded
  past its checked basis (VR-5).

### feat(reveal)/test(l1): M-1051 Increment-3 certified round-trip + M-1055 E3 PASS (2026-07-11)

PR #1423 (`claude/leaf/m1051-inc3-e3-reveal-roundtrip`, review fast-follow `2ae0cd63`). Builds
`crate::reveal::certified_roundtrip` (DN-38 §5's `delaborate . lower = id` obligation, gated by
`certified` mode) and the M-1055 E3 go/no-go experiment
(`crates/mycelium-l1/src/tests/reveal_roundtrip_e3.rs`).

- **`certified_roundtrip` always computes the L0-term-level identity witness** (`reelaborate` +
  `alpha_eq`), and, as a secondary best-effort convenience, attempts a genuine surface round-trip
  (real reparse → real re-check → real re-elaborate → `alpha_eq`) whenever `render_surface` flags
  the text `reparseable` — reporting an explicit `SurfaceOutcome`, never trusting the flag alone
  (house rule #2, never-silent).
- **Verify-first finding, review-hardened (empirical):** `reparseable = true` is necessary but not
  sufficient for real parser acceptance. Confirmed non-reparseable in practice: `Node::Lam` (no
  param-type annotation in the render), every `Node::Op` (the `#op[..]` marker; kernel prim names
  are always `.`/`:`-namespaced), and — added during adversarial review, closing an over-generalized
  initial claim — `App`/`Match` (real re-check refuses even `%`/`#`-marker-free terms). The
  genuinely surface-round-trippable fragment is narrow: `Const`/`Var`/`Let`(-of-those) only.
- **OQ-H3 disposed: option (1)/(a) — L0-term-level identity witness, never a surface re-parse for
  `%`-names**; option (2) (display-renaming) stays deferred.
- **M-1055's E3 experiment: 6 tests, all green** — composes `certified_roundtrip` over the E1
  non-vacuity recipe (independently-spelled oracle, `Interpreter::eval` differential, a
  disable-freshening mutation self-check). With E1 (PASS, prior session) and E3 (PASS, here) both
  built and run, **M-1055's own E1+E3 Definition of Done is SATISFIED** — M-1055 status
  `in-progress → done` (already applied in a prior close-out pass; recorded here for the CHANGELOG
  record of this PR).
- **Docs (append-only):** `DN-110-8.2-hygiene-deepdive.md` gains an "E3 Result" addendum;
  `DN-110-Native-Metaprogramming-And-Sugar-Lowering-Facility.md` §8.4 gains a narrowly-scoped
  addendum — the reveal spine moves `Declared → Empirical` only for the L0-term-level claim on the
  reparseable fragment, not a blanket upgrade. DN-110 stays `Accepted`, not `Enacted`.
- M-1051 stays `status:in-progress` — Increment-2 (CLI/LSP surfacing of `desugar`/`expand`) is the
  sole remaining gap in its 3-increment DoD.

### fix(cli): M-1024 linker reconciliation — collapse duplicate linkers onto canonical `PhylumEnv::link` (2026-07-11)

PR #1427 (`claude/leaf/m1024-linker-reconciliation`). Collapses `mycelium-cli`'s own duplicate
flatten-by-name linker onto the canonical `checkty::PhylumEnv::link` (M-1024/DN-101 §5-A,
differential-tested by `crates/mycelium-l1/tests/phylum_exec.rs`) — and fixes a real user-facing
bug along the way.

- **The bug (fixed):** `myc check` type-checked each `.myc` source in isolation (`check_nodule` per
  file — a phylum-of-one), so a project with a genuine cross-nodule `use` always refused under
  `myc check`, even though the identical project ran fine under `myc run` (which already assembled
  every source into one `Phylum` via `check_phylum`). Verified before-vs-after against the committed
  `tests/fixtures/run-multi-nodule/` fixture.
- **The DRY consolidation:** a new shared seam, `assemble_and_check_phylum(sources, project_dir)`
  in `mycelium-cli`, is now the one path both `check_project` (`myc check`) and `run_multi_nodule`
  (`myc run`/`myc build`) call — diverging only after assembly. Retired the CLI's own duplicate
  linker (`merge_phylum_env` / `merge_map` / `fmt_pair_key`, ~155 LOC); `run_multi_nodule` now calls
  `phylum_env.link()`, the canonical `mycelium-l1` linker, directly. Net `crates/mycelium-cli/src/lib.rs`:
  -65 LOC. `checkty.rs` confirmed untouched (parallel-safe with the concurrent M-1054 facility build
  touching that file).
- **Intentional behavioral delta (per DN-101 §5-A v0 flat-namespace semantics):** `link` refuses ANY
  cross-nodule simple-name duplicate, even byte-identical — stricter than the retired linker's
  identical-bytes tolerance. A correctness upgrade, not a regression.
- **`CheckReport` honesty shift:** since `check_phylum` is all-or-nothing (one `CheckError` for the
  whole phylum, never a per-nodule verdict), `CheckReport.checked`/`failures` now follow the same
  all-or-nothing honesty as `mycelium-check`'s `PhylumReport` (VR-5) instead of implying a per-nodule
  split it cannot supply.
- `cargo test -p mycelium-cli`: 36 passed (2 new). `cargo test -p mycelium-check`: 9 passed,
  unchanged. `cargo test -p mycelium-l1 --test phylum_exec`: 6 passed, unchanged (the `link`
  differential itself, untouched by this CLI-side change). `clippy -D warnings` / `fmt` clean.
- FLAGGED follow-up (not done here, out of this leaf's declared scope): `mycelium-check`'s
  `check_project`/`check_sources` (phylum-of-one) still exist alongside `check_phylum_dir` (already
  phylum-aware); a smaller in-crate follow-up could point `myc-check --project` at
  `check_phylum_dir` instead.

### feat(skills,sugar-index): `/native-translate` classification skill (M-1056) + native-strategy population (M-1058 follow-up) (2026-07-11)

PR #1422. Operationalizes the ratified DN-111 native-equivalence taxonomy in two parts.

- **`.claude/skills/native-translate/SKILL.md` (M-1056, status `todo → done`).** Implements DN-110
  §9's six-step decision procedure: verify-first against the DN-99 register/corpus (mitigation #14);
  identify the underlying PROBLEM, not Rust's mechanism (DN-109 F1); classify via the DN-111
  `{exact?}×{native?}` generator into `NativeEquivalent | IdiomaticRemapping | Approximation |
  InteropBridge`; apply the DN-109 §3.2 auto-fire ratchet; a sugar-specific `lower`-rule case;
  produce the §9 Step-6 register-style artifact. Documents the per-category VR-5 tag ceiling, the
  (construct, context)-pair plus time-indexed caveat (DN-111 §8.1/§8.4), and the `wild`-fold seam
  (DN-111 §8.3).
- **Self-check basis:** rather than a standalone synthetic self-check against the DN-110 §1 J1–J5
  table, the skill was applied in the SAME PR to classify all 47 `tools/grammar/sugar.yaml` rows —
  covering the J1 exemplar (`lower`/`derive` → `NativeEquivalent`, matching DN-110 §1's J1 row) and
  the J2 exemplar (`object`/`via`/`impl` → `IdiomaticRemapping`, matching DN-110 §1's J2 row)
  directly, a materially stronger validation than the DoD's literal minimum.
- **Sugar-index `native_strategy` column populated (M-1058 follow-up, status stays `done`,
  supersedes its own "remains deferred" note).** 35 `NativeEquivalent`, 3 `IdiomaticRemapping`
  (the `object`/`via`/`impl` inherent-desugar family), 1 `Approximation` (`lambda`'s deferred
  capture semantics), 8 honestly `unclassified # needs-review` (Gap rows with no landed L1
  construct, plus `grow`'s deliberate non-functional reservation) — marked, never fabricated
  (VR-5). `sugar_index.py` regenerated to render the column; `docs/sugar-index/INDEX.md`/`index.json`
  committed.
- Verification: `python3 tools/grammar/sugar_index.py --self-test` green; `scripts/checks/sugar-index.sh`
  green; `scripts/checks/markdown.sh` green (484 docs, 0 errors); `ruff check`/`ruff format --check`
  clean on `sugar_index.py`; full pre-commit suite green on both commits.

### docs: DN-112 + DN-113 ratified (delegated) + DN-110/M-1054 gate-met correction (2026-07-10)

Integration-tier close-out for a maintainer-delegated ratification pass ("ratify the options best fit
objectively speaking for this project" — the maintainer delegated the option choice; the orchestrator
selected the options below on the merits stated in each note's own objective-function analysis). Both
are **design ratifications only, NOT Enacted** (house rule #3) — no code lands with this entry.

- **Accepted (design, delegated ratification, pending Enactment): DN-112 nodule-qualified type
  identity (Rank 1 + builtin-invariant test).** `docs/notes/DN-112-Nodule-Qualified-Type-Identity.md`
  Status **Draft → Accepted**. Ratifies **Rank 1** — a nodule-qualified name carried in the existing
  `Ty::Data` `String` slot, stamped at `resolve_ty` from `DataInfo.home` (the declaring nodule),
  collision-free by construction for `PartialEq`/`subst_ty`/mangling, leaving ~79 destructure sites
  untouched (KC-3-minimal). The `type_head` impl-coherence twin gap is confirmed **IN-SCOPE** (an
  automatic consequence of Rank 1). A new DoD **condition** is added: the implementation must ship a
  dedicated regression test pinning the **builtin/prelude uniform-home invariant**
  (`Bool`/`Option`/`Result`/`Tuple$N` stay under one reserved home across every nodule) — the direct
  answer to the note's own sharpest adversarial finding. Guarantee posture: `Empirical` once the
  general fix lands and is witnessed (the flipped `ctor_seal.rs` differential + the collision property
  test + the builtin-invariant test), `Declared` on a narrower point-patch (VR-5). Unblocks **M-1036**
  — `tools/github/issues.yaml` gains a mechanism note.
- **Accepted (design, delegated ratification, pending Enactment): DN-113 cross-phylum
  import/resolution subsystem (v1 design).** `docs/notes/DN-113-Cross-Phylum-Import-Resolution-Subsystem.md`
  Status **Draft → Accepted**. Ratifies the recommended v1 design: the `::` phylum-boundary `use` head
  (`use dep::nod.sym`, Rank 1); loading via a verified source tree plus a generated content-pinned lock
  (B2+B4), with the separate-compilation interface blob (B3) explicitly deferred; the OQ-H1 cross-phylum
  def-site-ref granularity accepted at **phylum level** — `(phylum_hash, qualified_name)`, with
  per-function content-addressing deferred, honestly; the diamond/version-conflict policy (OQ-CP-1) set
  to **strict, no silent SemVer-coalescing** — two differently-pinned phyla are two different phyla, a
  crossing value is a never-silent type mismatch, with an explicit-opt-in coalescing policy deferred to
  a possible v2 (never a silent default, per house rule #2). The v1/deferred boundary (§8) is accepted
  as specified. Mints **M-1060** (the v1 implementation issue, `depends_on: [M-1024]`, relating to
  M-1054 and M-982).
- **DN-110/M-1054 gate-met correction + FLAG-A architecture ratification (append-only, mitigation
  #14).** `docs/notes/DN-110-Native-Metaprogramming-And-Sugar-Lowering-Facility.md` gains a second §8.4
  addendum recording **FLAG-A** — the facility's two-phase build architecture (an L1 check-phase
  resolving a value-parametric sugar call's def-site and running the affine tracker on the substituted
  `Expr`; an L0 elab-phase extending `elaborate_lower_rule` to be value-parametric, freshening
  `%`-namespace binders off the GLOBAL `Elab::fresh` counter) — the corpus's first citable record of
  this previously-unrecorded decision. Corrects M-1054's stale "blocked until M-1055 PASS" body text:
  **E1 and E3 have both PASSED** (E3 landed via PR #1423, `crates/mycelium-l1/src/tests/
  reveal_roundtrip_e3.rs`), satisfying M-1055's own E1+E3 Definition of Done — **M-1054 is unblocked**
  (`status:todo → status:in-progress`; M-1055 `status:in-progress → status:done`). The facility builds
  as a staged serial pipeline: Stage 0 (a provably-inert, always-refusing recognition+matcher skeleton
  that cannot emit an expansion) is the sanctioned entry point; hygiene lands in Stage 1. DN-110 stays
  `Accepted`, NOT `Enacted`; no guarantee tag is upgraded past its checked basis by this correction
  (VR-5) — it corrects status and records architecture, it does not certify anything as built.
- **`docs/Doc-Index.md` reconciled.** DN-112 and DN-113 rows added, status **Accepted**.
- **`tools/github/issues.yaml` reconciled (append-only, mitigation #2, validated + deduped).** M-1036
  gains a DN-112-Rank-1 mechanism note plus a `corpus:DN-112` `doc_refs` entry; **M-1060** minted
  (next free id after M-1059); M-1054 gains the gate-met + FLAG-A note and its `status` label corrected
  `todo → in-progress`; M-1055 gains an E3 close-out addendum and its `status` label corrected
  `in-progress → done` (landed basis: PR #1413 E1 + PR #1423 E3).

### test(l1): M-1055 E2 + E5 hygiene experiments — close-out (2026-07-10)

Integration-tier close-out reconciling the FLAGs from PR #1415 (`claude/leaf/m1055-e2-e5-hygiene`,
merged to `dev` 2026-07-10), a small follow-on to the M-1051/M-1055-E1 close-out below. **This
close-out handles only the E2/E5 increment** — no other shared-file drift.

- **E2 (def-site resolution) and E5 (affine soundness on the expanded L0): both PASS, but
  additive/non-gating.** A test-only pair of sibling harnesses (`hygiene_defsite_resolution.rs`,
  `hygiene_affine_expanded.rs`) extends the E1 capture-avoidance corpus per
  `DN-110-8.2-hygiene-deepdive.md` §7. **Critical honesty note (a Medium finding surfaced during the
  PR's own adversarial review pass): E2 and E5 are ADDITIVE, non-gating exploration — NOT progress
  against M-1055's formal Definition of Done.** That DoD is **E1 + E3**, the Rank-1 go/no-go the
  deep-dive commissions (deep-dive §9); **E3 still needs `reveal` Increment-3 (M-1051) and remains
  unbuilt**, so the DoD stays unsatisfied regardless of this PR. `DN-110` and
  `DN-110-8.2-hygiene-deepdive.md` stay `Accepted`, **not** `Enacted` (house rule #3) — no guarantee
  tag is upgraded past its checked basis (VR-5).
  - **E2 moves clause (C) def-site resolution `Declared → Empirical` for the in-scope, same-nodule
    case only** — the cross-phylum question **OQ-H1 stays open**, unexercised by this PR.
  - **E5 moves clause (D) affine soundness `Declared → Empirical` for the duplication upper-bound
    property only**, via the real, landed M-919 static pass (`crate::affine::Tracker`) run on a
    fully-substituted expanded surface program — the drop/lower-bound property is explicitly **NOT
    validated** (that is an M-904 **runtime** concern the static pass does not enforce, per
    `tests/affine.rs:304-312`'s own doc comment plus DN-71 §8 FLAG-4) — correcting an initial
    task-brief assumption that a drop would statically REJECT.
  - `cargo test -p mycelium-l1` green (503 passed, 0 failed; 7/7 hygiene tests including E1/E2/E5).
- **`tools/github/issues.yaml` reconciled (append-only, mitigation #2).** M-1055's `landed_basis`
  gains a dated addendum recording the E2/E5 PASS verdicts and their honest scope (above); the issue
  stays `status:in-progress`, **not** `done`. M-1054 (the implementation epic) gains three further
  dated design notes — OQ-H1 confirmed still open, a new E2 verbatim-splice composition-gap note, and
  an OQ-H4 sharpening from E5's upper-bound-only finding — appended to its 2026-07-10 design-notes
  block, the same append-only mechanism the E1 close-out used. **No new tracking issue was minted**
  for these residuals: each is already carried as a dated M-1054 design note (the mechanism the PR's
  own "FLAGs for the integrating parent" note points at — "for whoever picks up the follow-on
  design/implementation work"), and no distinct tracking need beyond that was identified: OQ-H1 was
  already an M-1054/DN-110-8.2 §10 open question, and the drop-lower-bound gap is already tracked as
  M-904. Validated + checked for duplicate ids after edit.
- **`docs/tero-index/` regenerated** (`just tero-index-gen`) — the `issues.yaml`/`CHANGELOG.md`
  addenda above did change indexed content (new/moved `M-1055`/`M-1054` rows, a new changelog-entry
  row); `scripts/checks/tero-index.sh` confirmed stale before the regen and `docs/tero-index/ is
  current` after. The two new test files are test-only source with no `docs/Doc-Index.md` row
  (confirmed by inspection, not assumed).

### feat(l1)/test(l1): M-1051 reveal Increment-1 + M-1055 E1 capture-avoidance PASS — integration close-out (2026-07-10)

Integration-tier close-out reconciling the FLAGs from PR #1412 (M-1051 `reveal` Increment-1) and PR
1413 (M-1055 E1 capture-avoidance experiment) — both already merged to `dev` this session.

- **M-1051 — `reveal` Increment-1 landed; Increments 2 and 3 remain (`status:in-progress`, NOT
  `done`).** New `crates/mycelium-l1/src/reveal.rs` ships the desugar-on-demand core: `reveal_l0`
  (`Exact`, a direct wrap of `elab::elaborate`); `alpha_eq` (`Empirical`, structural alpha-
  equivalence, unit-tested against hand-built adversarial pairs including Fix/FixGroup/Match binder
  forms); `render_surface` (`Empirical`, a total pretty-printer with a never-silent
  `RenderError::Unrenderable` refusal for a `Dense`/`Vsa` `Const` payload or a non-finite `Float` —
  no black boxes, house rule #2); `reelaborate` (`Empirical`, an honestly-scoped closedness re-
  derivation, explicitly NOT a full surface re-parse pipeline). `cargo test -p mycelium-l1` green
  (21/21 new `reveal` tests, 0 regressions). Increment-2 (CLI/LSP surfacing) and Increment-3 (the
  `certified`-mode round-trip check, DN-38 §5/§8.3) are unbuilt.
- **M-1055 — E1 (capture-avoidance) verdict: PASS; E3 deliberately not built
  (`status:in-progress`, NOT `done`).** A test-only harness
  (`crates/mycelium-l1/src/tests/hygiene_expr_sugar.rs`) validates DN-110-8.2-hygiene-deepdive §4
  (A) and (B) — `%`-namespace freshening of RHS binders plus capture-safe verbatim substitution —
  over 4 core fixtures, each checked by `alpha_eq`-to-oracle AND independent
  `mycelium_interp::Interpreter::eval` observational equality, plus a hand-built unhygienic
  "captured" counter-expansion per fixture (proving the harness can observe a real capture bug), plus
  a live-verified negative control (freshening disabled, both checks failed as expected, reverted
  before commit). This moves capture-avoidance for (A) and (B) **only** from `Declared` to
  `Empirical`; E2 (def-site resolution) and E5 (affine-on-expanded-L0) stay `Declared`, unbuilt.
  **E3 (`reveal` round-trip fidelity — the deep-dive's own designated gating experiment for this
  issue's E1+E3 DoD) is deliberately not built**, not merely deferred-and-pending: both the deep-dive
  §7 header and `reveal.rs`'s own honest-scoping callout record that
  `reelaborate(reveal_l0(x))` is definitionally `x.clone()` at v0 (`reveal` v0 has no lossy step to
  invert), so an E3 harness against v0 `reveal` would be vacuous — the same trap PR #1412's
  adversarial review caught and fixed in `reveal` itself. E3 becomes meaningfully non-vacuous only
  once M-1051 Increment-3 lands; until then, **E1 is the sole empirical half of M-1055's own E1+E3
  go/no-go**, and M-1054 stays design-gated.
- **M-1054 (the native-facility implementation epic) gains three design notes from the E1
  language-fit review**, appended to its issue body (append-only, house rule #3) so they survive to
  inform the eventual build: (1) **OQ-H5** — the facility must use the elaborator's GLOBAL
  `Elab::fresh` counter and site-qualified `%sugar#<rule>@<siteN>%tmp` names
  (DN-110-8.2-hygiene-deepdive §4(A)), NOT E1's prototype per-call `%0` counter reset, or nested/
  repeated expansions will silently collide; (2) **OQ-H1** — def-site resolution of a sugar RHS's
  free identifiers to content-addressed L0 references (DN-110-8.2-hygiene-deepdive §4(C)) is unbuilt
  by E1 and must be built by the facility, cross-phylum residual unresolved; (3) a longer-term,
  separately-decided note that alpha-canonical `Node` hashing (canonicalizing binders to de Bruijn
  before `content_hash`) would make content-addressing alpha-aware and give free expansion dedup
  (the deep-dive §6 "true half" finding) — out of this epic's scope, touches the ADR-003 identity
  model repo-wide.
- **Honest scope note (VR-5).** DN-110 and `DN-110-8.2-hygiene-deepdive.md` stay `Accepted` (design
  ratification only) — this close-out records issue-level status; it does not move either note
  toward `Enacted`. None of M-1051 / M-1055 / M-1054 is marked `done`; each carries an honest
  landed-basis note scoped to what actually landed. `tools/github/issues.yaml` validated + checked
  for duplicate ids after edit (mitigation #2).

### docs(notes): maintainer ratification — DN-100 macro-expand-first pre-pass; DN-102 `?` try-operator desugar (2026-07-10)

Accepted (design, pending Enactment): **DN-100** and **DN-102**, per the maintainer's "ratify the
above" naming both. Design/mechanism decisions ratified; house rule #3 — Accepted is NOT Enacted, no
code lands via either ratification.

- **DN-100 — Status Draft -> Accepted.** Ratifies §5 Rank 1 (`cargo expand` as an opt-in,
  off-by-default, never-gating profiling mode for the transpiler's macro-expand pre-pass) plus Rank 2
  (a small dependency-free std-macro shim) as complementary. The §3 honest ROI framing (macro expansion
  raises `expressible_fraction`; its `checked_fraction` effect stays `Declared`/uncertain on this
  corpus) is carried forward unchanged (VR-5) — not overclaimed by ratification. Unblocks **M-1032**
  (the toolchain DN it required is now ratified).
- **DN-102 — Status Draft -> Accepted.** Ratifies SP.5 Rank 1: the landed v0 `?` desugar (§2 fork
  resolution, §3 error-type unification, §5 `let`-RHS position restriction — differential-witnessed,
  M-1025/PR #1363) plus the second research pass's resolutions — FLAG-try-2 (implicit `From`-widening
  is a deliberate exclusion, mapped to the native `map_err` combinator, no error-conversion-trait
  subsystem) and FLAG-try-1 (general-position `?` reframed as an independent CPS-lift follow-up,
  decoupled from the never-type per DN-107 §6-a). Confirms the DN-99 §8 ENB-2 FLAG-try-2 RESOLUTION
  addendum's wording (a dated confirmation addendum added to DN-99 §8 in this same change).
- **Issues reconciled:** **M-1049** (DN-102 second research pass) -> `done`, landed basis the DN-102
  ratification above. New follow-up **M-1059** filed for the general-`?` CPS-lift increment (decoupled
  from M-1030 per DN-107 §6-a, cross-ref DN-102 SP.3/FLAG-try-1) — the residual FLAG-SP-2 routing.
  (M-1054 was already taken by the DN-110 epic; mitigation #1 — verified against the actual tree
  before minting, not the earlier free-slot check against a stale checkout.)
  **M-1032** doc_refs/body note updated to reflect DN-100 as ratified (no code lands here; M-1032
  itself stays `todo`, now unblocked on its required toolchain DN).
- **Doc-Index rows** for DN-100 and DN-102 updated Draft -> Accepted with the ratified basis.

### chore(integration): dev -> integration close-out — DN-111 ratification + gap-close-run batch reconciliation (2026-07-10)

Integration-tier close-out for the gap-close-run batch (dn-102 second pass, blocked-op protocol,
trunk-PR-only rule, tero-trim, M-1041-A `ExprVisitor`, M-1042 nodule-path, M-1044 remap-manifest,
DN-110 ratification, sugar-index M-1058 — nine PRs landed on `dev` this session) plus the held DN-111
ratification. Reconciles the batch's shared-file FLAGs **once**, per the concurrent-PR development
pattern's integration tier.

- **DN-111 — Status Draft → Accepted (maintainer ratification, 2026-07-10; design ratification only,
  NOT Enacted).** Ratifies the **canonical Rust→Mycelium native-translation taxonomy**: **Native
  Equivalent** (was Adaptation — renamed, corrects a Vinay-and-Darbelnet false-friend collision),
  **Idiomatic Remapping** (was Solution — renamed, corrects a genus/species overload with DN-109 F1),
  **Approximation** (kept — already the correct PL term), **Interop Bridge** (kept, qualified). Net:
  2 renamed, 2 kept; DN-110's original handles (Adaptation/Solution/Approximation/Bridge) retained as
  **permanent aliases**, never deprecated. Ratifies the §4.1 two-question generator, the §5 per-category
  decision procedure and honesty-posture ceilings, and the §6 orthogonal-2-D-grid reconciliation with
  DN-109's decidability axis (Mechanical/Heuristic/Judgment). Accepts the §8.1 adversarial finding —
  classification is of a **(construct, context) pair**, time-indexed under the unfrozen kernel — as an
  honest standing limitation. **Ratifies TERMINOLOGY + TAXONOMY only** (VR-5): no classification against
  it is upgraded past `Declared` by this ratification; ships no code, enacts nothing. DN-110 §2 gains the
  append-only pointer resolving its provisional-labels carve-out (DN-110's own Ratification text
  unchanged, append-only per house rule #3).
- **Follow-up close-outs applied at this pass:** **M-1057** (DN-111's own tracking issue) → `done`.
  **M-1056** (`/native-translate` skill) gains `doc_refs: corpus:DN-111` and a terminology-only update
  noting the skill should emit the canonical enum once implemented. **M-1058** (sugar-index) status
  corrected `in-progress` → `done` (verified merged on `dev`, PR #1403, before this flip — mitigation
  #14); its deferred `native_strategy` column's placeholder comment in `tools/grammar/sugar.yaml`
  updated to cite the now-ratified DN-111 enum (was "DN-109's taxonomy is still settling"); populating
  the column's 45 rows stays deferred, bounded follow-up (not fabricated here, VR-5).
- **Batch fast-follow progress notes recorded (issues.yaml):** **M-1041** (DRY `ExprVisitor`/fold —
  Scope-A landed, PR #1398: the pilot abstraction over `mycelium-transpile`'s `emit.rs`/`map.rs`;
  status `in-progress` — the `mycelium-l1` walker half + `prim_map.rs`/`Pat` dispatch remain, honestly
  not claimed done). **M-1049** (DN-102 second research pass — PR #1395: FLAG-try-2 resolved via the
  native `map_err(e, conv)?` idiom, no error-conversion-trait subsystem; recommends Rank 1) — status
  `in-progress`: the research pass is complete and DN-102 re-submitted/surfaced for ratification, but
  **DN-102's own Status stays Draft** (the maintainer has not yet ratified it — surfaced as an open item
  below, not decided here per house rule #3). M-1042 (nodule-path) and M-1044 (remap-manifest) were
  already correctly closed `done` by their own leaf PRs (#1399/#1400) — no change needed here.
- **`docs/Doc-Index.md`** gains a new **DN-111** row (Accepted); the **DN-110** row's summary updated to
  reflect the resolved taxonomy carve-out; the **DN-102** row's summary appended with the second-pass
  addendum (status stays Draft); a new **`docs/sugar-index/`** entry under §6 Agent tooling index; the
  §5 zero-hand-port bullet updated with M-1041/M-1042/M-1044 landing status.
- **`docs/spec/api/mycelium-transpile.txt` (public-API baseline) regenerated first** (`just
  api-baseline`) — the committed baseline predated M-1044's `remap` module, so `docs/api-index/`
  (which sources from `docs/spec/api/*.txt`, not raw source) could not see it without a baseline
  catch-up first; the M-1041-A `visit` module is `pub(crate)`, correctly not part of the public
  surface. `scripts/checks/api.sh` re-run **green (57/57 crates)** after the catch-up — every added
  symbol traces to a landed, merged commit, nothing unreviewed. **`docs/api-index/`** then
  regenerated (`just docs-index`) for the newly-visible public surface: `mycelium_transpile::remap`
  (M-1044, `RemapManifest`/`RemapOperation`/`IdiomClass`/etc., `BatchSummary::remap` +
  `batch::summarize`).
- **`docs/tero-index/`** regenerated (`just tero-index-gen`) after the Doc-Index/DN-status/issues edits
  above (the standing lesson: these edits need a tero-index regen or the drift gate flags).
- Validated: `python3 -c "import yaml; yaml.safe_load(open('tools/github/issues.yaml'))"` clean, **no
  duplicate ids** (565 issues checked); `python3 tools/github/doc_refs_check.py` clean;
  `scripts/checks/markdown.sh` green on all touched/new `.md` (483 docs, 0 errors).
- **Surfaced for the maintainer (not ratified here, held for the orchestrator to escalate):** **DN-100**
  (transpiler cargo-expand pre-pass) and **DN-102** (`?` try-operator desugar, second pass now complete)
  both remain **Draft**, awaiting a ratification pass.

### feat(transpile): zero-hand-port force-multiplier fast-follows — ExprVisitor pilot, nodule-path fix, remap manifest (M-1041/M-1042/M-1044, 2026-07-10)

Three `mycelium-transpile` PRs from the zero-hand-port delta-ledger's filed force-multiplier issues
(behavior-preserving where noted, tests green throughout):

- **M-1041 Scope-A — the DRY `ExprVisitor`/`TypeVisitor` pilot** (PR #1398). Replaces 5 independently
  hand-rolled `match`-over-`{Expr,Type}`-variant dispatch functions across `emit.rs` (3) and `map.rs`
  (2) with 2 canonical dispatchers (new `crates/mycelium-transpile/src/visit.rs`: `walk_expr`/
  `walk_type`) over visitor traits with an associated `Output` type and a required `fallback` method —
  a new `Expr`/`Type` shape now touches one trait method plus whichever visitor(s) care, never a
  silently-drifting nth hand-written match. Behavior-preserving: byte-identical `cargo test -p
  mycelium-transpile` (65/65) before/after. Scope-A only — `prim_map.rs`'s 9 more `Expr::` matches, the
  `Pat`/`map_pattern` dispatch, and the `mycelium-l1` walker half of this issue's DoD are flagged
  follow-on targets, not done here.
- **M-1042 (residual) — path-qualified nodule names for nested modules** (PR #1399). Verify-first
  (mitigation #14) found the issue's central DoD clause (flat-emit last-writer-wins collision) already
  landed by `e0085ec0`; the genuine residual was `derive_nodule_path` deriving a nodule header from the
  crate directory name alone, so two nested `mod.rs` files in the same crate emitted the SAME header
  despite writing to distinct output paths. Fixed by anchoring on the file's last `src` path component
  plus its dotted intra-crate module path (`crates/mycelium-std-cmp/src/foo/mod.rs` → `std.cmp.foo`).
  Data-driven fixture-table tests + a same-crate-sibling-non-collision property test added.
- **M-1044 — the remap manifest** (PR #1400), implementing DN-109 §5.2 per its ratified §7-c/§7-e
  forks: folds `remap: RemapManifest` into the existing `BatchSummary` (no standalone `remap.json`);
  `REMAP.md` renders as a pure, round-trip-tested projection. v0 scope (ratified Mechanical-only
  auto-fire): every transpiled file gets one pure `Keep` entry; `idiom_choices` ships honestly empty (no
  per-item EXPLAIN instrumentation exists yet to populate it from — not fabricated). Guarantee tag stays
  `Declared` throughout (VR-5 — the manifest records decisions, it does not certify them). 11 new tests
  (`src/tests/remap.rs`) plus 2 folded into `src/tests/batch.rs`; `cargo test -p mycelium-transpile`
  green at 75 (was 66).

All three: `cargo fmt`/`clippy -D warnings` clean, change-scoped per the leaf/working-tier gate policy.
Shared files (`CHANGELOG`/`Doc-Index`/`api-index`) were left to the integrating parent per each PR's own
FLAG — reconciled above.

### docs(agents): blocked-op protocol (mitigation #15) + trunk-branches-are-PR-only hardening (2026-07-10)

Two agent-persona house-rule additions, both purely additive/append-only (house rule #3), landed as
separate PRs the same session:

- **Mitigation #15 — the blocked-op protocol** (PR #1396). A standing response for every spawned agent
  that hits a `PreToolUse` hook block or a permission wall, instead of retry-looping, circumventing the
  guard, or fabricating success: (1) recognize the block as a policy/permission boundary, not a code
  failure; (2) never retry-loop or circumvent maliciously, never fabricate completion; (3) try the
  sanctioned alternative first (a PR instead of a raw protected-tier push; `--no-verify` plus
  out-of-band gates for an external-hook 403; split `commit`/`push` for the branch-guard string-match
  false-positive); (4) no alternative → `SendMessage(to: "main")` with the precise ask, keep doing other
  non-blocked work, flag it in the final report. Added as new `CLAUDE.md` mitigation #15 plus a
  persona-voiced paragraph in each of `.claude/agents/{myc-leaf,myc-porter,integrator,design-reasoner,
  pr-reviewer,security-reviewer}.md`.
- **Trunk-branches-are-PR-only, made unmissable** (PR #1397). A prominent, affirmative callout added to
  every agent persona file and to `CLAUDE.md`'s "Commits & PRs" section header: `main`/`integration`/
  `dev` never take a direct commit/merge/push, only a GitHub PR — cross-referencing the branch-guard hook
  and mitigation #10. Workers that drive trunk landings (`myc-leaf`/`myc-porter`/`integrator`) get the
  most explicit placement; the read-only reviewers get a concise one-liner that they never merge to a
  trunk at all.

`scripts/checks/markdown.sh` green (0 errors, 478/483 tracked docs at landing time); `scripts/checks/
secrets.sh` and `scripts/checks/branch-guard.sh` both clean.

### docs(notes): maintainer ratification — DN-110 native-metaprogramming facility + §8.2 hygiene mechanism (2026-07-10)

Records the maintainer's ratification of **DN-110** (Mycelium's native facility for the role Rust fills
with macros — generative-lowering + expression-position sugar rules) and its companion **DN-110-8.2
hygiene deep-dive**, both held as Draft PRs (append-only; house rule #3 — no existing decision text
rewritten, each note gains a dated "Ratification" section plus a Status-line update).

- **DN-110 — Status Draft → Accepted (design, pending Enactment — see below).** §1's five-job role
  decomposition confirmed (J1 derive-gen / J2 delegation already native+landed; J4 compile-time
  computation native via DN-55 static specialization; J5 foreign-syntax DSL a Bridge/flagged exclusion;
  **J3 expression-position sugar the single real gap**). §6's **Rank 1** accepted as the recommended
  mechanism: generalize the landed `lower`/`derive` term-level lowering framework (DN-54/M-812) to
  expression position, with `reveal`/M-1051 as the mandatory transparency spine. §8.2's hygiene/scoping
  model accepted **as the basis**, citing the companion deep-dive's `%`-namespace-partition mechanism.
- **Accepted, NOT Enacted (VR-5, house rule #3, honesty-critical).** This ratifies the *design* only —
  no code has landed for J3, `reveal`/M-1051 has not shipped, and the E1/E3 hygiene experiments have not
  run. **The facility's guarantees stay `Declared`** throughout; no guarantee tag is upgraded past its
  basis. `Enacted` requires full implementation + the E1/E3 (+E2/E4/E5) experiments passing +
  `reveal`/M-1051 landing — not before.
- **§2 taxonomy carve-out.** The Adaptation/Solution/Approximation/Bridge labels are ratified only as
  **provisional, intuitive handles**, not canonical terminology. The canonical Rust→Mycelium
  translation taxonomy is **deferred to a forthcoming companion DN-111 (Draft, not yet authored)**.
- **DN-110-8.2-hygiene-deepdive.md — Status Draft → "Accepted as the basis for DN-110 §8.2".** The §4
  hygiene model (def-site resolution plus `%`-namespace freshening reusing the landed `Elab::fresh`
  gensym plus partition-safe substitution plus affine/type checking on the expanded L0, reusing the
  landed M-919 affine tracker) and the §7 experiment plan are accepted; **E1 + E3 commissioned** as the Rank-1
  go/no-go for J3. The §9 tractability verdict (keep DN-110 Rank-1; §8.2 downgraded from "sharpest open
  question" to "mechanism identified, two bounded residuals to prototype") is recorded. §10's residual
  open questions (OQ-H1…OQ-H6) are **not** dispositioned by this pass — flagged, not guessed — and
  carry into the follow-up issues below. The note stands as a **companion** to DN-110 (cross-referenced
  from DN-110 §8.2), not folded into its text.
- **§8.1 (J5 permanent exclusion vs. later Bridge) and DN-110's remaining §11 open questions stay open**,
  not dispositioned by this ratification.

`docs/Doc-Index.md` gains rows for **DN-110** (→ Accepted) and the **DN-110-8.2 companion note** (→
Accepted as the basis for DN-110 §8.2). `tools/github/issues.yaml` gains **M-1054…M-1057** (all
`status:todo`): a native-facility implementation epic (`depends_on: [M-812, M-1051]`, noting the §8.2
hygiene model is its own sub-design carrying OQ-H1…OQ-H6), an E1+E3 hygiene-experiment prototype issue
(`depends_on: [M-1051]`, noting OQ-H1 as the expression analogue of DN-54 §10 OQ-D), a
`/native-translate` methodology-skill issue, and a DN-111 taxonomy-companion authoring task; plus a
`corpus:DN-110` `doc_refs` cross-ref added to **M-875** and **M-1032** (redirect, not supersede — both
stay scoped to the transpiler-side expand-first question, already answered by DN-100). Validated
(`python3 -c "import yaml; yaml.safe_load(...)"` clean, no duplicate ids; `doc_refs_check.py` clean).
`scripts/checks/markdown.sh` green on all touched `.md`.

### docs(notes): maintainer ratification batch — DN-101 through DN-109 (2026-07-11)

Records the maintainer's ratification decisions on all nine ENB/design-reasoner notes from the
2026-07-10 batch (append-only; house rule #3 — no existing decision text rewritten, each note gains a
dated "Ratification / Maintainer decision" section plus a Status-line update where the maintainer
ratified).

- **DN-101 (Cross-Nodule Runtime Link) — Status Draft → Accepted.** The v0 flat-namespace collision
  policy (§5 option A) is confirmed as the shipped baseline; the long-term choice between §5 option
  (B) qualified per-frame scoping and option (C) name-mangled qualified keys is explicitly left open
  for a dedicated planning pass (API design first, then progressive/iterative implementation). Filed
  **M-1048**.
- **DN-102 (`?` Try-Operator Desugar) — NOT ratified; stays Draft.** The maintainer requires a second
  research pass grounded in the DN-99 gap-review data and project intent before re-staging for
  ratification. **FLAG-try-2** (no `From`-error widening) is cross-routed into DN-99's §8 `enb`
  backlog as an ENB-2 sub-gap addendum. Filed **M-1049**.
- **DN-103 (Impl-Level Generic-Parameter Slot) — Status Draft → Accepted.** §3 Fork 1 ("parse-time
  flatten vs AST slot") is ratified as **option (B)** — the AST slot + Phase-0 desugar-prepend design,
  matching the note's own recommendation.
- **DN-104 (Per-Constructor Visibility Seal) — Status stays Draft** (the shipped M-1027 binary seal is
  not re-ratified as the final design). The maintainer directs pursuing §3's fuller **option (B)**
  (`pub(path)` scoped visibility) as the long-term successor to the binary `priv` seal. `M-1036`
  (nodule-qualified type identity) updated with this forward-direction progress note; filed **M-1050**
  (`depends_on: [M-1036]`) as the design+implementation follow-up.
- **DN-105 (`match` on a `Bytes` Scrutinee) — Status Draft → Accepted, ratified as drafted.** The §3
  literal-only-equality fork and the §4 per-surface-form redundancy-key choice both confirmed.
- **DN-106 (Statement-Sequencing + Record-Update Triage) — Status Draft → Accepted.** §3's fork
  confirmed (no new L1 grammar); fork (B) reframed as an in-scope, mechanically-lowering, reversibly-
  expandable **surface sugar**. Establishes two general, project-wide principles: **surface-sugar
  transparency** (a sugar may be carried when it drives toward the native target, hides nothing via
  on-demand expand/EXPLAIN, and is purely mnemonic) and the **gap-closure default** (map an excluded
  construct's underlying problem to Mycelium's native solution — auto-emit where mechanical, flag
  **with the suggested native idiom** where judgment is needed; bare refusal is the last resort).
  Cross-linked to DN-109's L4 idiom framework and the zero-hand-port delta ledger. Filed **M-1051**
  (desugar/expand-on-demand tooling).
- **DN-107 (Host-Effect Real-Syscall Registry + Never-Type) — Status Draft → Accepted.** Rank 1
  accepted for both sub-gaps (real `wild:` prims via H-i; divergence-as-effect N-C, no bottom `Ty`).
  The §6 fork resolved to **6-a**: general-`?` is independent of the never-type. Corrects **M-1025**'s
  FLAG-try-1 progress note (the "gated on the never-type" framing lived there, not in DN-102's own
  text) to decouple the CPS lift from M-1030, tracked via M-1049.
- **DN-108 (Numerics Transcendentals) — Status Draft → Accepted.** Rank 1 accepted (op-level bound in
  the ADR-010 provenance certificate, v0 tag `Declared`). OQ-3 (vendored vs host libm) accepted with
  recommendation (vendored/version-locked); OQ-4 resolved to land-now (M-1028). Filed **M-1053**
  (companion ADR + the OQ-3 implementation decision).
- **DN-109 (L4 Idiom-Optimal Transpilation + L5 Structural Remapping) — Status Draft (advisory) →
  Accepted.** The Mechanical-only v0 auto-fire set, the remap-manifest schema, and structure-
  preserving 1:1 all accepted; forks §7-a (L2 surface) and §7-c (extend the existing gap-report
  artifact) accepted per the note's own leans. **Fork §7-b/F1 REFRAMED** (not left open): the `&mut`-
  aliasing (D7) and unbounded-iterator (D8) cases are the deliberate-exclusion set, with native
  Mycelium solutions (functional update, bounded `for`) — a borrowck/rust-analyzer frontend is an
  **optional precision aid**, not a porting blocker; the default path is flag-with-suggested-native-
  idiom (`suggested_idiom`, M-1045). **Fork §7-d is FLAGGED**, genuinely unresolved (not silently
  assumed). Adds a new pipeline-wide transparency/revealability requirement, cross-linked to DN-106's
  desugar/expand capability (M-1051). Filed **M-1052**.

`docs/Doc-Index.md` status-column cells updated for DN-101/DN-103/DN-105/DN-106/DN-107/DN-108/DN-109
(→ Accepted); DN-102/DN-104 cells stay Draft, matching their own dispositions above.
`tools/github/issues.yaml` gains **M-1048…M-1053** (all `status:todo`) plus progress notes on
**M-1025** and **M-1036**; validated (`python3 -c "import yaml; yaml.safe_load(...)"` clean, no
duplicate ids; `doc_refs_check.py` clean). `scripts/checks/markdown.sh` green on all touched `.md`.

### chore(extract): trim vendored tero dev-source; wire to published tero (2026-07-10)

`crates/mycelium-tero` (DN-87/E39-1, M-1015…M-1018) was extracted verbatim into its own published
repo, `tzervas/tero-rs` (private, renamed `tero`), which now ships binaries + a container image at
`v0.1.3`. This trim removes the in-tree crate mycelium was still recompiling on every `tero-index`
regeneration and rewires the gate to consume the published binary instead — a build-speed win with
no behavior change, verified `Empirical`.

- **Removed:** `crates/mycelium-tero/` (workspace member + all source); the `tokio`/`axum`
  workspace dependencies added at M-1017 solely for its HTTP front (no other consumer); its
  `xtask/deps-strata.toml` stratum/tier entries.
- **Added:** `scripts/fetch-tero-index.sh` — resolves a checksum-verified, cached `tero-index`
  binary from the pinned `tero-rs` release (`tools/tero-rs/PROVENANCE.md` + `SHA256SUMS.txt`),
  fetching via an authenticated `gh` (the repo is private) on a cache miss. `scripts/checks/tero-index.sh`
  and `just tero-index-gen` now run this binary instead of `cargo run -p mycelium-tero`; the gate
  stays skip-graceful (no cached-or-fetchable binary ⇒ skip, never a false-red), while a freshly-downloaded asset that fails its pinned SHA256 hard-fails the gate (a supply-chain tampering signal, never a skip) and an unsupported host platform takes the skip path (the pin ships a `linux-x86_64` asset).
- **Kept as-is:** `packages/tero-mcp-lite/` and the `.mcp.json` `tero` MCP registration — this
  vendored, zero-dependency Python server (no compile step) is an intentional *consumed* snapshot,
  not development source; `.mcp.json` already ran it, unaffected by this trim.
- **Verified before removal (Empirical, not just claimed):** a three-way differential —
  `cargo run -p mycelium-tero --bin tero-index` (in-tree, pre-removal) vs. the downloaded/
  checksum-verified `tero-index-v0.1.3-linux-x86_64` binary vs. the already-committed
  `docs/tero-index/` — all three byte-identical (`index.json` + `INDEX.md`). Post-removal:
  `cargo build --workspace` clean, `scripts/checks/deps-acyclic.sh` green, `python3
  tools/github/doc_refs_check.py` clean (one dangling `M-1015` `src:` ref to the removed crate
  repointed to `tools/tero-rs/PROVENANCE.md`), `docs/api-index/` + `docs/tero-index/` regenerated.
- **FLAGGED for the maintainer (not executed here):** the private-repo prerequisite. Fetching a
  fresh (uncached) `tero-index` binary now requires an authenticated `gh` CLI with read access to
  `tzervas/tero-rs` — a new prerequisite the prior self-contained `cargo run -p mycelium-tero` did
  not have. Options: make `tero-rs` public, vendor the binary (checked into a release asset /
  `git-lfs`), or accept the `gh`-auth prerequisite as the standing cost (current default — the gate
  degrades to a graceful skip, never a false-red, when it's unmet). This pattern (verify via
  differential → remove recompiled dev-source → consume a pinned published artifact) generalizes to
  the other tools extracted per the workspace-tools publish program.

### docs(planning): zero-hand-port delta ledger + DN-109 idiom/structural design (2026-07-10)

Lands the capstone planning deliverable for the zero-hand-port program: a five-layer analysis
swarm (L1 hand-expressibility ceiling, L2 engines, L3 transpiler, L4/L5 idiom + structural design,
plus a DX/QoL track) synthesized into one living register.

- **`docs/planning/zero-hand-port-delta-ledger.md`** (Status: Draft, living — updated in place as
  phases close). Frames the program around two numbers: the ~85% hand-expressibility ceiling (L1)
  vs the ~7.8% measured auto-transpile floor (`checked_fraction`, L3) — the delta between them is
  the roadmap. Finds ~75-80% of the ~812 measured gap instances are downstream language/kernel
  surface, not transpiler defects (empirically proven: faithful new transpiler rules moved
  `checked_fraction` by 0 across several ladder phases; only kernel/language surface closures
  moved it). Records the visitor-DRY meta-gap (a new `Expr` variant costs ~13 hand-edits) as the
  force-multiplier to build first, and the flat-emit stem-collision data-loss bug as an L5
  prerequisite fix. Names 6 strategic forks for the maintainer (F1, acquiring a borrowck frontend,
  is pivotal). Supporting per-layer inventories committed as appendices under
  `docs/planning/zero-hand-port/` (`delta-L2-engines.md`, `delta-L3-transpiler.md`,
  `delta-DX-qol.md`, `delta-L4L5-idiom-structural-DRAFT.md`).
- **DN-109 — Layer-4 Idiom-Optimal Transpilation and Layer-5 Structural Remapping**
  (`docs/notes/DN-109-Idiom-Optimal-Transpilation-And-Structural-Remapping.md`, **Draft, pending
  ratification** — house rule #3, not self-ratified). Recommends a three-bucket idiom-decision
  framework (Mechanical auto-emit / Heuristic-plus-flag / Judgment-flag-never-guess) bound by a
  conjunctive no-silent-upgrade ratchet (semantics-preserving, no inferred `swap` per S1, no
  guarantee-tag upgrade per VR-5, EXPLAIN-recorded), plus a structure-preserving-1:1 default and a
  **mandatory remap manifest** (`remap.json`/`REMAP.md`) as the transpiler's structural + idiom
  provenance artifact of record. Verify-first finding (mitigation #14): the transpiler's batch mode
  discards directory structure today and last-writer-wins on stem collision — a real data-loss bug,
  not just a missing feature.
- **Seven new tracked issues filed** (`tools/github/issues.yaml`, epic E32-1, all `status:todo`):
  **M-1041** (DRY `ExprVisitor`/fold trait — the L2 meta-gap and DX-D1 finding converged
  independently on the same gap), **M-1042** (structured, source-tree-mirroring transpiler output
  with path-qualified nodule names, fixing the flat-emit collision), **M-1043** (per-item
  `// src: file:line` provenance breadcrumbs in emitted `.myc`), **M-1044** (the remap manifest,
  gated on DN-109's ratification), **M-1045** (actionable `suggested_idiom` on gap diagnostics),
  **M-1046** (closest-to-clean investment ranking in the vet report), **M-1047** (transpiler DX
  polish: `mycfmt` post-pass, dry-run/summary mode, minimal arg parsing — combined). One prior DX
  finding (the tree-sitter `priv` grammar gap) had already independently landed same-day as M-1039
  and is recorded as closed rather than re-filed (verified against the codebase before filing,
  mitigation #14).

`docs/Doc-Index.md` updated with the DN-109 row and a zero-hand-port program pointer under
Build Status. `python3 -c "import yaml; yaml.safe_load(...)"` clean, no duplicate ids;
`tools/github/doc_refs_check.py` clean; `scripts/checks/markdown.sh` green on all touched/new
`.md` files.

### fix(grammar,docs): terminal-review fast-follows — tree-sitter `priv` structural gap + wrong path citation (M-1039/M-1040) (2026-07-10)

Two small findings from the terminal review of the enb-wave batch, tracked and closed same-day.

- **tree-sitter editor grammar missing the `priv` constructor-seal keyword.**
  `tools/grammar/generate.py`'s hand-maintained `STRUCTURAL_KEYWORDS` set was never updated for `priv`
  after M-1027/DN-104 landed the per-constructor visibility seal (`pub type T = priv Mk(..)`), so the
  generator auto-mis-classified `priv` as reserved-inactive and the tree-sitter `constructor` rule had
  no `priv` prefix — the editor grammar could not parse a construct the L1 compiler accepts. Fixed by
  adding `priv` to `STRUCTURAL_KEYWORDS` and `optional('priv')` to the `constructor` rule template,
  then regenerating (`python3 tools/grammar/generate.py`); the committed `grammar.js` delta is the
  mechanical regen output. `scripts/checks/drift.sh` / `grammar.sh` green.
- **Wrong path citation in the self-hosting port ledger.** `docs/planning/self-hosting-port-ledger.md`
  cited `crates/mycelium-l1/src/phylum_exec.rs` for `PhylumEnv::link` — that path is the *test* file
  (`crates/mycelium-l1/tests/phylum_exec.rs`); the impl is `crates/mycelium-l1/src/checkty.rs:1086`
  (verified by grep). Corrected the citation.

Tracked as **M-1039** (grammar gap) and **M-1040** (doc-hygiene), both filed `status:done` with a
landed basis at PR #1385.

### feat(scripts): worktree `target/` cache pruning — `disk-watchdog` + `worktree-target-sweep` (2026-07-10)

Implements the maintainer's storage-management ask for the multi-worktree swarm setup. Measured:
sccache cross-worktree hit rate is ~0 here (each worktree builds its own `target/`, so cache sharing
doesn't help) — the real reclaimable win is pruning stale build output.

- `scripts/disk-watchdog.sh` — read-only usage monitor; lists the top reclaimable worktree `target/`
  dirs by size once a warn/crit threshold is crossed. Never deletes.
- `scripts/worktree-target-sweep.sh` — dry-run-by-default sweeper. A worktree's `target/` is
  reclaimable only when not locked, its branch is merged into the mainline (default `dev`) or the
  worktree dir is gone, `target/` has been idle >= `--stale-min` (default 240) minutes, and no
  `rustc`/`cargo` process is building under that path. `--apply` to delete; `--incremental-only`
  reaps just stale `incremental/` cruft in kept worktrees.
- `just reclaim` (watchdog + sweeper dry-run) / `just reclaim-apply` (sweeper `--apply`).

`scripts/checks/shell.sh` (shellcheck) 73/73 clean incl. both new scripts.

### feat(tools): diff-based, rate-limit-frugal GitHub issue sync + orphan reconciliation (2026-07-10)

Adds `tools/github/sync_issues.py`: a diff-based sync between `tools/github/issues.yaml` and GitHub
Issues — one bulk read, then plan/apply only the drifted or missing deltas (create + edit changed
fields), with a classified orphan-reconciliation mode (`--reconcile-orphans`: closed-id-dup /
open-id-dup / adoptable / non-task classes). Adds `just issues-sync` (dry-run plan) / `just
issues-sync-apply` (apply, capped `--max-writes 25`), `.claude/skills/issues-sync/SKILL.md`, and
`tools/github/README.md`. Desktop/periodic op — needs `gh` authenticated to the repo owner; not part
of the per-commit gate. `--self-test` 25/25 checks pass.

### docs(dn-108): numerics transcendentals accuracy-bounds design note (M-1028/ENB-5, DN-99 #42) (2026-07-10)

Adds Draft design-reasoner note **DN-108** (`docs/notes/DN-108-Numerics-Transcendentals-Accuracy-Bounds.md`)
working DN-99 register row #42 forward: recommends transcendental/irrational-result functions
(`sqrt`/`exp`/`ln`/`sin`/`cos`/`pow`) surface as `flt.*` interpreter prims returning a plain
`Repr::Float` whose accuracy bound rides the existing `Bound`/`Approx` certificate (ADR-010) — no new
numeric type, no new kernel node (KC-3). The v0 tag is explicitly `Declared` (not `Empirical` — no
measured trial corpus yet, VR-5); domain errors and approximate-input composition both refuse
never-silently (G2). Enacts nothing; ratifies nothing (house rule #3) — status stays Draft. `doc_refs:
corpus:DN-108` wired onto **M-1028**.

### docs(dn-107): host-effect real-syscall registry + never-type divergence design note (M-1030/ENB-7) (2026-07-10)

Adds Draft design-reasoner note **DN-107** (`docs/notes/DN-107-Host-Effect-And-Never-Type.md`) working
the M-1030 (ENB-7) decision forward: separates the two ENB-7 sub-gaps (host-effect real syscalls #79;
never-type `-> !` divergence #88), proposes granting real `wild:read/write/get_env/exit` prims over
the existing RFC-0028 `wild`/RFC-0014 effect surface, and recommends divergence-as-effect (no bottom
`Ty`, Rank 1) over a nominal `Never`+`absurd` (Rank 2 reserve) or a true bottom subtype (Rank 3,
rejected for v0). Enacts nothing; ratifies nothing (house rule #3) — status stays Draft. `doc_refs:
corpus:DN-107` wired onto **M-1030**.

### chore(integration): enb batch close-out — api-gate tool-flag fix + baseline catch-up (M-1038) (2026-07-10)

Integration-tier close-out for the enb batch (M-1024/1025/1026/1027/1035/1033). The bounded canary
gate found the batch content itself clean (fmt/clippy/tests/drift/indices/markdown/links/secrets/
deny/licenses all green); the only red was the `api` gate, which decomposed into two things resolved
here (filed as **M-1038**):

- **Tool-flag drift (environmental, pre-existing — fails identically on `main`):** `cargo-public-api`
  0.52.0 removed the `--toolchain` flag that `scripts/checks/api.sh` and `scripts/api-baseline.sh`
  both passed. Fixed to the rustup selector form (`cargo +"$api_toolchain" public-api …`), verified
  empirically to build and emit a non-empty surface before any mass regeneration.
- **Baseline catch-up (genuine landed surface, not batch leakage):** with the gate unblocked, the
  diff — unregenerated since 2026-07-06 — covered both the enb batch's stated deltas (`mycelium-l1`:
  `Expr::Try`/`Expr::Wrapping`/`Ctor::sealed`/`InherentImplDecl::params`/`PhylumEnv::link`/
  `Tok::Priv`/`Tok::Question`/`Tok::Wrapping`; `mycelium-transpile`: the `emit_expr`/
  `emit_block_as_expr` param thread, new `gap::Category` variants, `prim_map`,
  `batch::output_rel_path`) and prior work that landed 2026-07-07..08 while this same broken gate
  silently masked the drift (`mycelium-core::binary` CU-1/CU-3/CU-5/CU-6 bit-manipulation/float-conv/
  wrapping prims; `mycelium-interp::prims::eval_wrapping`; `mycelium-check`'s M-1006 `--phylum`
  cross-nodule mode). Every added symbol traced to a landed, merged commit (`git log -S`) — nothing
  unreviewed, no accidental `pub`, no unrelated-crate leak (the transparency review this batch's DoD
  requires). Baselines regenerated (`just api-baseline`) for the 5 affected crates
  (`mycelium-check`/`-core`/`-interp`/`-l1`/`-transpile`); `scripts/checks/api.sh` re-run GREEN
  (57/57 crates checked/skip-N/A, zero FAIL).

`docs/api-index/` and `docs/tero-index/` regenerated to reflect the new public surface (`just
docs-index`, `just tero-index-gen`); `doc_refs_check.py` clean. Per-batch issue status close-out was
already correctly applied by the batch's own per-issue PRs (M-1024/M-1025 honestly `in-progress` with
their stated residuals; M-1026/M-1027/M-1035 `done` with residuals noted; M-1033 `todo` per DN-106) —
verified against the codebase, not rubber-stamped (mitigation #14); no status changed here. DN-101
through DN-106 remain `Draft` (maintainer-ratification-pending), already indexed in `Doc-Index.md`.

### docs(dn-106): triage ENB-10 statement-sequencing + record-update — both closed at L1 (M-1033) (2026-07-10)

Adds **DN-106 — Statement-Sequencing (`let _`) + Record-Update / Mutation Split: a Triage (ENB-10)**
(Draft) plus four pinning regression witnesses in `crates/mycelium-l1/tests/enablement.rs` — **no
AST/grammar/code change**. Per mitigation #14 (verify a stale issue's claim against the codebase before
implementing), the M-1033 triage finds that **both** ENB-10 sub-gaps' language side is **already closed at
L1**, three-way witnessed (L1-eval ≡ elaborate→L0-interp ≡ trampoline-AOT):

- **Part 1 — statement-sequencing `let _ = e in body`** is an ordinary `let` whose binder is the
  identifier `_` (grammatical `ebnf:291`/`ebnf:447`, parsed, checked, evaluated, elaborated) and is
  moreover the established affine drop/use-once surface (DN-71/M-903). Two pins lock the plain and the
  ascribed `let _: T = e in body` discard forms.
- **Part 2 — functional field-update** is the already-expressible destructure-and-reconstruct
  `match base { Ctor(f0, …) => Ctor(f0, …, NEW, …) }` (Mycelium has positional constructors, no
  named-field record literal and no field-projection, by design — KC-3). One pin locks the reconstruct
  form value-correct; one never-silent pin locks that a Rust-style `{ ..base, field: v }` record-update
  literal has **no** Mycelium surface and is an explicit parse refusal (G2), never a silent mis-parse.

The real residual is entirely **transpiler-lane** (`crates/mycelium-transpile`: the `let _` emit and the
mutation→functional rewrite), confirming DN-99 register row #89's own `tr`/`low` tags and correcting the
M-1033 issue body's over-scoped "grammar-`enb`, HIGH collision, touches `crates/mycelium-l1/**`" framing
(mitigation #14). **M-1033's L1/semcore residual is NIL**; the issue is re-scoped to its transpiler-lane
residual. DN-106 enacts nothing and moves no other doc's status (append-only, house rule #3); tags are
`Empirical` where three-way-witnessed, `Declared` for the unratified Part-2 fork resolution (VR-5).
Reviewed per `/pr-review` (PR #1373); the pins keep the closure from silently regressing. (DN-106 Draft;
DN-99 #89 aligned; M-1033 re-scoped; VR-5 / G2 / house rules #3/#4.)

### feat(transpile): flip DN-99 #72 string-literal `match` gap→emit on the M-1035 enabler; gap fabricated conversion no-ops (2026-07-10)

The first **enabler-driven transpiler win**: now that **M-1035 / ENB-12** landed the L1
match-on-`Bytes` enabler (DN-99 register row #72), the Rust→Mycelium transpiler flips #72 from
*gapped* to *emitted* — `match s { "yes" => true, _ => false }` lowers to the faithful, `myc
check`-clean `match s { "yes" => True, _ => False }` (`&str` → `Bytes`, `"yes"` verbatim,
`true`/`false` → `True`/`False`). Scope: `crates/mycelium-transpile` only (PR #1372 → dev). Emissions
stay `Declared` (no guarantee tag upgraded — VR-5).

- **Emit only WITH a wildcard/default arm; else gap, never-silently (G2/VR-5).** `Bytes` is an OPEN
  value domain, so M-1035's W7 coverage rejects a non-exhaustive `Bytes` match (`non-exhaustive match
  on Bytes: missing _`). The `Expr::Match` guard emits a string-literal match **only** when it carries
  an unguarded irrefutable default (wildcard `_` or a bare binding); a defaultless string-literal
  match still gaps with the precise open-domain reason — never a check-failing non-exhaustive surface.
  Pinned by `string_literal_pattern_emits_with_l1_enabler` (positive + the defaultless-gaps negative).
- **Co-fix: gap Rust ownership/identity-conversion no-op methods, never fabricate a prim (G2/VR-5).**
  `.to_owned()` / `.clone()` / `.to_string()` / `.into()` / `.as_ref()` / … have no Mycelium
  free-function or prim referent (value semantics — ADR-003), so the old bare-call desugar
  (`recv.to_owned()` → `to_owned(recv)`) **fabricated** a call to a non-existent prim (`myc check`:
  `unknown function/constructor/prim to_owned`). These are now gapped explicitly instead of
  fake-emitted. This un-poisons real files under the vet loop's file-gated `checked_fraction` (the
  `checkty::vsa_kernel_model_id` string-`match` whose arm bodies are `"MAP-I".to_owned()` now gaps
  cleanly rather than dragging a fabricated `to_owned` into an emission). Pinned by
  `conversion_noop_method_gaps_never_fabricates_unknown_prim`.
- **Measured effect (`Empirical`).** `checked_fraction` on the 24-target port-surface corpus rises
  **6.193% → 6.740% (+0.547pp)** — but the load-bearing change is *correctness*: no more fabricated
  prims (an honest gap is the right outcome for an unmapped conversion — VR-5/G2). The corpus win now
  awaits the conversion-method mapping (`ToOwned`/`Clone`/`ToString`/`Into` → identity-or-real-surface),
  filed as the next `checked_fraction` lever.
- **How verified (change-scoped).** `cargo test -p mycelium-transpile` green (62 tests, incl. the two
  new pins) · `cargo fmt --check` clean · `cargo clippy -p mycelium-transpile --all-targets -D
  warnings` clean. The `Bytes`-match surface's `myc check`-cleanliness rides M-1035's own three-way
  differential (`crates/mycelium-l1/tests/enablement.rs`), unchanged here.

### feat(l1): admit a `Bytes` scrutinee in `match` — M-1035 (ENB-12) (2026-07-10)

Closes **DN-99 register row #72** (string-literal match pattern): the L1 checker now admits a `match`
whose scrutinee is a `Bytes` value, with **byte-string-literal** arms and a **required default arm** —
the ENB-12 enabler that unblocks every string-dispatch (`match s { "get" => …, "post" => …, _ => … }`)
port target. Under the whole-project unfrozen posture (ADR-045). Design recorded in **Draft DN-105**
(the maintainer ratifies — house rule #3, not self-ratified).

- **One-clause enabler, not a new pattern subsystem (KC-3/DRY).** The single categorical block was
  `check_match`'s scrutinee-type gate, which admitted only `Data`/`Binary`/`Ternary`; lifting it to
  admit `Ty::Bytes` is the whole language change. Everything downstream already handled an open-domain
  literal column generically — `normalize_pattern` already types `0x…`/`"…"` literal patterns as
  `Bytes`, `usefulness::signature()` already returns `None` for `Bytes` (an OPEN domain), the decision
  compiler already marks a non-`Data` column incomplete, and the evaluator's `try_match` already
  compares `Repr::Bytes`/`Payload::Bytes` by content. No new L0 node, no new pattern/AST node, no new
  checking pass.
- **Byte-content equality; both surface spellings.** A `"…"` text literal (`Literal::Str`) and a `0x…`
  hex literal (`Literal::Bytes`) lower to the SAME `Repr::Bytes` value, so `"foo"` and `0x666f6f`
  denote equal values and match interchangeably (`elab.rs::lit_key_to_value` decodes the `by:` and `s:`
  literal-pattern keys to `Repr::Bytes`, using the established `Meta::exact(Provenance::Root)` literal
  form — no guarantee tag upgraded, VR-5).
- **`Bytes` is OPEN ⇒ a default arm is REQUIRED, never-silently (G2).** A literal column never
  completes the domain, so a `Bytes` match without a wildcard/default is a static **W7 non-exhaustive
  refusal** (witness `_`), exactly as for `Binary`/`Ternary`. An ill-typed literal arm (a `Binary`
  literal against a `Bytes` scrutinee) is likewise a static refusal, never a silent coercion.
- **`.myc` mirror (DN-26 dual).** Pattern-typing + coverage leaves were already at parity (generic over
  open types). The `Bytes` repr value is lifted out of the `LOpaque` collapse into a dedicated
  `LReprBytes(Bytes)` carrier so `lval_try_match`'s `PLit` arm does a real byte-content match against a
  text (`Str`) byte-string literal — the eval-match mirror for #72. All three `LVal` consumers
  (`PIdent`/`PCtor`/`PLit`) handle the new variant exhaustively; no AST/pattern node changed, so the
  fingerprint/classify walkers are unaffected. The `0x…`-hex value synthesis in `.myc` stays deferred
  (FLAG-semcore-25, an honest `Err`); other reprs stay `LOpaque` (FLAG-semcore-35).
- **Honest residuals (VR-5/G2).** The native-LLVM textual-IR backend cannot lower a `Bytes`-scrutinee
  match (its `Binary8`-specialized switch) — it **refuses explicitly** (`AotError`), and the
  trampoline AOT handles the case; the cross-surface-form redundancy key stays per-form (`by:`/`s:`), a
  conservative under-report of a redundancy *lint*, never a wrong-arm miscompile (DN-105 §4); the trx
  transpiler's #72 pin (`string_literal_pattern_gaps_with_l1_enabler_reason`) can now flip *gapped* →
  *emitted* as a **separate transpile-crate follow-up** (outside the `mycelium-l1` lane).
- **How verified (change-scoped).** `crates/mycelium-l1/tests/enablement.rs` — a three-way differential
  (L1-eval ≡ elaborate→L0-interp ≡ trampoline-AOT): text-literal hit, fall-through-to-default,
  hex/text cross-spelling hit (`"foo"` hits a `0x666f6f` arm), plus static rejects of the
  non-exhaustive and ill-typed cases and the explicit native-LLVM refusal. `.myc` eval-match
  differential vs the real `Evaluator::try_match` oracle (`src/tests/compiler_stage5_evalmatch.rs`):
  text-literal hit/miss, empty-string edge, binder-captures-`Bytes`, `Bytes`-vs-`Ctor` false, and the
  honest `0x…`-hex deferral probe. `cargo test -p mycelium-l1` green · `cargo fmt --check` clean ·
  `cargo clippy -p mycelium-l1 --all-targets -D warnings` clean · `myc check lib/compiler/semcore.myc`
  clean (dogfood parity) · `markdown.sh` + `doc_refs_check.py` clean.

### feat(l1): per-constructor visibility seal — `priv` ctor — M-1027 (ENB-4) (2026-07-10)

Closes the **sealed-constructor visibility** surface gap (DN-99 register row #37 / §A3, ENB-4; and the
row #69 positional field-visibility sub-case): a `priv` marker on the `constructor` production so
`pub type T = priv Mk(..)` exports the type **NAME** (usable cross-nodule in signatures, `use`, and
**pattern position**) but **withholds the constructor from cross-nodule CONSTRUCTION via an imported
name** — a never-silent refusal for a well-behaved caller going through `use`. Under the whole-project
unfrozen posture (ADR-045). Design recorded in **Draft DN-104** (the maintainer ratifies the corrected,
narrower scope below — house rule #3, not self-ratified).

**CRITICAL, found + independently reproduced at integration review (house rule #4 — no claim upgraded
past a checked basis):** this mechanism is **NOT an enforced security/capability boundary**. Mycelium
resolves types/ctors by bare name in the *caller's own scope* (own local decls shadow imports — the
pre-existing RFC-0006 §4.3 / M-662 precedence rule), so a foreign nodule that declares its **own
same-named local (unsealed) type**, without ever importing the real sealed one, bypasses the seal
entirely — `check_phylum` accepts it. This falsifies the "unforgeable capability-gate"/FR-N3 framing
the design originally claimed; **M-1023's `Approx::proven` port must not treat this as a real security
boundary** until the real fix — nodule-qualified type identity — lands (tracked as **M-1036**). Pinned
as a differential witness (`ctor_seal.rs::known_gap_a_same_named_local_shadow_type_bypasses_the_seal`),
not silently absent from coverage. As landed, `priv` is an **opt-in API-discipline nudge** for
well-behaved `use`-based callers, not a capability guarantee.

- **Surface + AST boolean + one export-table predicate, no new kernel node (KC-3/DRY).** `Tok::Priv`
  is a reserved keyword (never a silent identifier — G2), parsed as an optional prefix in `parse_ctor`
  and threaded into the AST (`Ctor.sealed: bool`, not flattened — faithful for printing / round-trip).
  A `pub type`'s `priv` constructor names are recorded per-type in the phylum export table
  (`Exports.sealed_ctors`) and folded, on import, into a per-nodule **withheld set**
  (`NoduleImports.sealed`) — the exact twin of the existing `ambiguous` glob-collision machinery,
  reusing the already-Enacted M-662/M-1024 cross-nodule export/resolution layer. The nodule's **own**
  constructor names are subtracted (a home construction is never wrongly refused).
- **The DN-53 §B.6 Q1 fork resolved to the BINARY seal.** Not a full Rust-style `pub(path)` scoped-
  visibility grammar (KISS/YAGNI); `priv` is forward-compatible as the `nodule`-scoped point of a
  future lattice. Confronted on its merits in DN-104 §3 (no sycophancy — house rule #4).
- **Never-silent boundary for the import path (G2/VR-5).** Three refusals, each naming the fix:
  constructing a sealed ctor from a **foreign nodule via an imported name** (the withheld-construction
  `CheckError`, at both the nullary-value and saturated-`App` sites, before the arity check so the seal
  diagnostic wins); a redundant `priv` on a **non-`pub`** type (refused at registration); and `priv`
  inside an **`object`** body (refused at parse in the Rust frontend; the `.myc` mirror does not yet
  match — a residual, DN-104 §6). The type NAME and pattern-matching are unaffected — only
  *construction-via-import* is withheld. No guarantee tag upgraded past its checked basis (`Declared` →
  `Empirical` for the import-path refusal only; explicitly **not** upgraded to a capability/security
  claim — see the CRITICAL note above).
- **`.myc` mirror (DN-26 dual — surface + AST + fingerprint parity).** The `Priv` token
  (`token`/`lex`.myc), `Ctor` carrying `sealed` (`ast`/`parse`/`semcore`/`ambient`.myc), `parse_ctor`
  reading `priv`, and the structural-fingerprint walker hashing the seal (**tag 110**, both Rust and
  `.myc` sides — folded only when sealed, so an unsealed ctor hashes byte-identically). The **cross-
  nodule *enforcement* mirror rides the checkty `.myc` port (M-741)** — this increment mirrors the
  `.myc` surface/AST/fingerprint, not the enforcement layer; a `priv` in a `.myc` `object` body is
  accepted-then-unused (Rust refuses at parse). Honest residual, not a silent omission (DN-104 §6).
- **How verified (change-scoped):** `crates/mycelium-l1/tests/ctor_seal.rs` — 11 differential
  witnesses (home-construct OK ×2, foreign-construct-via-import REFUSED + the unsealed control proving
  the seal is non-vacuous, cross-nodule type-use-in-signature + pattern-match OK, redundant-seal +
  object-body refusals, per-ctor subset seal, surface round-trip, **+ the pinned known-gap witness for
  the same-name-shadow bypass**). `cargo test -p mycelium-l1 -p mycelium-fmt -p mycelium-lsp` green;
  `cargo clippy -p mycelium-l1 --all-targets -D warnings` clean; native `myc check` clean over the
  touched `.myc` (dogfood parity, all 9 self-hosted nodules) — the `.myc` surface is additionally
  witnessed by the Rust differential.

### feat(l1): impl-level generic-parameter slot — `impl[T] Foo[T]` — M-1026 (ENB-3) (2026-07-10)

Closes the **inherent-impl** scope of the impl-level-generics surface gap (DN-99 register row #63 / §A2,
ENB-3): a generic inherent block `impl[T] Foo[T] { fn … }` now parses, checks, and monomorphizes, so a Rust
`impl<T> Foo<T> { … }` ports faithfully instead of flattening to free functions. Under the whole-project
unfrozen posture (ADR-045). Design recorded in **Draft DN-103** (the maintainer ratifies the v0 scoping —
house rule #3, not self-ratified).

- **Surface + slot, no new kernel node (KC-3/DRY).** A `[T, …]` type-parameter slot is parsed
  **immediately after the `impl` keyword** via the existing unbounded-name parser (`parse_type_params_opt`);
  it is unambiguous (no `base_type` begins with `[`, so a leading `[` after `impl` is always the slot) and
  backward-compatible (`impl Foo[T] { … }` still parses `[T]` as the head's type argument). The slot is
  threaded into the AST (`InherentImplDecl.params`), kept faithful through ambient/print/walk.
- **Phase-0 desugar reuses the existing monomorphizer.** At the desugar that already lifts inherent methods
  to free functions, each method gains the impl's params **prepended** to its own `fn` type-parameters — so
  the method becomes an ordinary generic free function and monomorphization runs the **existing** fn-generics
  path with **zero new mono code** (VR-5: no guarantee-tag upgraded past its basis). For the plain M-664
  block (empty slot) this is the identity — methods lift verbatim, every existing program unchanged.
- **Never-silent boundary (G2/VR-5).** Three refusals, each naming the fix: a non-empty slot on a **trait
  instance** (`impl[T] Trait for Foo[T]`) is deferred (generic trait-instance coherence, RFC-0019 §4.5), a
  `: bound` inside the slot is refused (bounds live only on `fn` type-params, RFC-0019 §4.1), and a duplicate
  name shared by the impl slot and a method is caught by the existing duplicate-type-parameter check.
- **`.myc` mirror (DN-26 dual).** The 3-field `IID` slot lands in all four self-hosted mirrors
  (`ast`/`ambient`/`parse`/`semcore`.myc) plus the stage-5 encoder and the stage-3 classify-test ctor; the
  self-hosted parser produces the slot. The desugar-prepend stays Rust-only (pre-desugared before the mirror
  runs). `classify_expr` is unaffected (a struct-field extension, not a new node/variant; item-classify
  wildcards the variant).
- **How verified (change-scoped):** `crates/mycelium-l1/tests/check.rs` — a differential
  (`generic_inherent_method_monomorphizes_across_two_type_args`: two type args yield exactly two `unbox$…`
  specializations and a fully closed monomorphized env), a backward-compat identity witness, and three
  never-silent reject witnesses (trait-instance, bound-in-slot, duplicate-param). `cargo test -p
  mycelium-l1 -p mycelium-fmt -p mycelium-lsp` green; dogfood `myc check` strict on the touched `.myc`.

### feat(l1): `?` try-operator grammar sugar + type-directed `match` desugar — M-1025 (ENB-2) first increment (2026-07-10)

Closes the **`let`-binder-RHS scope** of the `?` try-operator surface gap (DN-99 register rows #60
and #52, ENB-2) — the dominant Rust error-propagation shape (`let x = f()?;`) now ports without hand-writing
a nested `match`. Under the whole-project unfrozen posture (ADR-045). **M-1025 stays `in-progress`**
(first landable increment; general-position CPS lift flagged below). Design recorded in **Draft DN-102**
(the maintainer ratifies the v0 scoping and the CPS-lift follow-up — house rule #3, not self-ratified).

- **Surface + lowering, no new kernel node (KC-3).** `Tok::Question` (lexer) parses as a `parse_app`
  postfix `e?` wrapping the operand in `Expr::Try` (`crates/mycelium-l1/src`); the **checker** desugars
  `let x = e? in body` — type-directed on `e`'s checked `Result`/`Option` type — to the existing `match`
  bind (`Ok(x) => body, Err($try_err) => Err($try_err)` resp. `Some(x) => body, None => None`), with the
  continuation `body` **inside** the binding arm. `Try` never survives checking; elab/eval keep only a
  defensive never-silent residual.
- **The design fork, confronted not guessed (DN-102 §2).** Rust's `?` desugars via an early `return` and
  the never-type `!`; Mycelium has neither (no `return` form; `-> !` deferred, DN-99 #88). The naive
  local desugar is doubly ill-typed here (`A` vs `Result[A, E]` do not unify). Putting the continuation
  in the binding arm makes both arms `typeof(body)`, so the propagation arm unifies with **no early
  return and no never-type** — the sound form.
- **Never-silent boundary (G2/VR-5).** The error-type unification rule (DN-102 §3) falls out of the
  `Err($f) => Err($f)` arm — exact-match, **no** `From`-error widening in v0. Every unsupported position
  is a refusal, never a mis-desugar: a `?` outside a `let`-binder RHS, a repeated `e??`, an ascribed
  `?`-binder, and a `?` on a non-`Result`/`Option` operand each raise a `CheckError` naming the fix.
- **`.myc` mirror (DN-26 dual).** `Try(Expr)` is **represented + traversed** across all five self-hosted
  encoders (`ast`/`semcore`/`parse`/`ambient`/`totality`.myc) and every walker, but the self-hosted
  lexer/parser does **not** yet PRODUCE it — the `Wrapping` represented-not-produced precedent (FLAG-try-3).
- **How verified (change-scoped):** `crates/mycelium-l1/tests/try_operator.rs` — 6 behavioural witnesses
  (`Result` Ok/Err, `Option` Some/None, chained `?`, and a chained short-circuit) each pinned against a
  hand-`match` twin on **both** the L1-eval leg and the three-way L0 leg (monomorphize -> elaborate ->
  L0-interp vs the L1 `to_core` projection), plus 4 never-silent reject witnesses. `cargo test -p
  mycelium-l1 -p mycelium-fmt -p mycelium-lsp` green; `cargo fmt --check` and `cargo clippy -p
  mycelium-l1 --all-targets -D warnings` clean; `myc check` on the touched `.myc` mirrors green.
- **Residual (flagged, VR-5/G2):** FLAG-try-1 — the general `?` position (a CPS lift of the enclosing
  expression) is deferred, gated on the never-type `-> !` / an early-return form (M-1030 / DN-99 #88);
  FLAG-try-2 — no `From`-error widening (exact error-channel match only); FLAG-try-3 — the `.myc`
  frontend represents-but-does-not-produce `?` (the `.myc` elab/eval mirror follows the port cadence,
  DN-26).

### feat(l1): cross-nodule runtime execution via `PhylumEnv::link` — M-1024 (ENB-1) first increment (2026-07-10)

Closes the **runtime-execution** half of cross-nodule symbols (DN-99 register row #41, ENB-1) — the
runtime dual of the landed check-time resolution (`checkty::resolve_imports`, M-662). Under the
whole-project unfrozen posture (ADR-045). **M-1024 stays `in-progress`** (first landable increment;
residual flagged below). Design recorded in **Draft DN-101** (renumbered from DN-100 at integration —
that number was taken by M-1032's macro-expand DN, which landed on `dev` first; G2, never-silent).

- **`PhylumEnv::link()`** (`crates/mycelium-l1/src/checkty.rs`) folds every nodule's **checked**
  declarations into **one** linked `Env` the existing `elab`/`mono`/`eval` pipeline consumes unchanged
  (**KC-3** — no new L0 node; **DRY** — reuses the check-time registry/coherence). Each name is merged
  from its **home** nodule's checked (authoritative, ambient-resolved) decl, never a less-resolved
  imported copy — so the linked `Env` is strictly more correct than running a consumer's per-nodule
  `Env` directly (a phylum-of-one included).
- **Verify-first (mitigation #14):** **direct** cross-nodule execution *already worked* (undocumented —
  a consumer's checked `Env` retains its imported `pub` decls with bodies); the stale `check_phylum`
  doc comment is corrected. The real gap closed here is **transitive** — a `pub` fn whose body calls
  its home nodule's **private** helper, previously `Stuck "unknown function"` from the per-nodule `Env`.
- **Never-silent boundary (G2/VR-5):** the v0 flat phylum namespace is one declaration per simple name;
  a cross-nodule name collision is an explicit `CheckError`, **never a silent winner**. Qualified
  per-nodule scoping that would *disambiguate* a collision (rather than refuse it) is the flagged
  **M-982** residual (needs the ratifying DN). No guarantee/emission tag upgraded.
- **How verified (change-scoped):** `crates/mycelium-l1/tests/phylum_exec.rs` — 6 differential
  witnesses (direct + transitive vs an inlined single-nodule oracle on the **L1-eval** and
  **L0-elaborate** legs; the never-silent collision refusal; the without-link `Stuck` control asserting
  the specific variant; the positive direct-without-link control isolating the "already worked"
  finding; phylum-of-one backward-compat). `cargo test -p mycelium-l1` green;
  `cargo fmt --check` + `cargo clippy -p mycelium-l1 --all-targets -D warnings` clean; markdown gate
  clean.
- **Residual (flagged, VR-5/G2):** AOT/MLIR parity leg = M-1024 follow-up (interpreter + L0 legs only
  here — do **not** read as AOT-complete); `.myc` runtime parity = **N/A** (the self-hosted
  `lib/compiler/*.myc` frontend has no evaluator — gated on M-986/M-987); collision disambiguation →
  M-982. DN-99 register rows #72/#85 also corrected `tr-only` → `language-enabler` (mitigation #14;
  DN-34 §8.21; #72 → M-1035, #85 gated on the ENB-1 symbol table).

### fix(l1): add the missing `Wrapping(_)` arm to the `compiler_stage3_ast` test-driver's `classify_expr` (2026-07-10)

Fixes a RED `dev`: 17 failing `ast_myc_*` tests in `crates/mycelium-l1/tests/compiler_stage3_ast.rs`.
The #1355 `Wrapping` Expr-variant port (M-1013/M-791) correctly added `| Wrapping(Expr)` to
`lib/compiler/ast.myc::Expr` (between `Spore` and `Consume`, mirroring `ast.rs::Expr`'s declaration
order) — but this test file's embedded `.myc` driver prelude (`driver_prelude()`'s `classify_expr`
helper, appended to every `program_with_prelude` fixture) still had an 18-arm match, so the
self-hosted checker's exhaustiveness gate (`myc check`: "non-exhaustive match on Expr: missing
Wrapping(_)") failed on every test that pulls in the shared prelude — not just the one test that
directly exercises `Expr` classification. Root cause was a completeness gap in the test harness, not
in `ast.myc` itself (`myc-check` on `ast.myc` is green both before and after this change). Fix: added
`Wrapping(_) => 0b00000000000000000000000000010010` (18) between the existing `Spore` (7) and
`Consume` (8) arms, positioned to mirror the oracle's declaration order for readability but given a
**new, unused** code (18, the next free value) rather than renumbering `Consume..TupleLit` — since
`ast_myc_classifies_every_expr_variant` hard-codes expected classification codes 0–17 for the other 18
variants and does not (yet) assert a value for `Wrapping` itself; renumbering would have silently
changed those tests' semantics for no reason (VR-5 — preserve what's already checked). Verified:
`cargo test -p mycelium-l1 --test compiler_stage3_ast` 26/26 green (was 9 passed/17 failed);
`cargo test -p mycelium-l1` full suite green, no regressions; `myc check lib/compiler/ast.myc` clean;
`cargo fmt --check` and `cargo clippy -p mycelium-l1 --all-targets -D warnings` clean. (M-1013/M-791;
VR-5/G2.)

### feat(transpile): trx increment — reclassify DN-99 #72 as an L1-enabler gap, DN-100 macro pre-pass DN, #1349 FLAG fixes (2026-07-10)

A transpiler-track increment toward the zero-hand-port north star (kickoff `trx`, lane
`crates/mycelium-transpile` + DN-100/DN-34 only). **Emission stays `Declared` (VR-5); the transpiler
classifies more honestly, never upgrades a guarantee tag.** Verify-first (mitigation #14) profiling of
DN-99 Track B's "trivial B1" literal-pattern closures against the real `myc check` oracle (whole
`crates/` corpus, 337 files) showed the B1 ranking wrong for a `checked_fraction` win, so the honest
outcome is a **0.00pp** metric change (checked_fraction 3.8% = 191/5061, expressible 11.3% = 573/5061,
both unchanged from the #1349 baseline):

- **DN-99 #72 (string-literal `match` pattern) reclassified transpiler-only → LANGUAGE-ENABLER.** The
  L1 checker categorically rejects a `match` on a `Bytes` scrutinee (`match scrutinee must be a data,
  Binary, or Ternary type, got Bytes` — verified against the oracle), so emitting the faithful surface
  produces parse-clean but check-*failing* `.myc` (a regression). `emit.rs` now gaps it **never-silently
  (G2)** with a precise reason naming the L1 enabler and citing the exact diagnostic — a transparency
  improvement over the prior generic message, **still gapped, zero metric change** — pinned by
  `string_literal_pattern_gaps_with_l1_enabler_reason`.
- **DN-99 #85 (byte-literal pattern) correctly NOT landed.** It is genuinely transpiler-only and
  myc-check-clean in isolation, but its sole corpus occurrence (`is_ident_byte`) co-locates an
  unknown-prim method call the desugar emits blindly (a pre-existing VR-5/G2 defect needing a symbol
  table), so landing it would regress the file-gated metric. Shipping a regression contradicts the
  raise-`checked_fraction` mandate.
- **DN-100 (Draft)** added — the toolchain DN for **M-1032 / ENB-9** macro expand-first pre-pass:
  whole-corpus macro profile (82 invocations, 18 files, ~93% custom `macro_rules!`), honest ROI (an
  expressibility lever with an uncertain `checked_fraction` effect), 4 alternatives, a ranked
  recommendation (opt-in `cargo expand` and a std-macro shim), and a DoD. Enacts nothing; **not
  self-ratified** (maintainer ratifies).
- **#1349 residual FLAG fixes:** `manifest_gen.py` semcore denominator now excludes `ModuleDecl` as well
  as `TestItem` (matching `gap.rs::Category::excluded_from_denominator`, M-1006 Phase-2, so the semcore
  single-file path and the stdlib batch path compute the identical denominator); `vet.rs` denominator
  prose corrected to "non-excluded"; DN-34 **§8.19** count 267→264 (its own arithmetic 5323−5059=264),
  **§8.20** "~25 files"→112 clobbered across 25 stem-groups, and new **§8.21** records the profiling
  finding.

**Two FLAGs filed for the language lane:** **FLAG-L1-match-Bytes** → new issue **M-1035** (ENB-12,
E28-1, area:language P2 — L1 `match` on a `Bytes` scrutinee, unblocks #72 and every string-`match` port
target); **FLAG-tr-unknown-prim** → subsumed by **M-1024** (ENB-1 cross-nodule symbol resolution, in
flight — a known-prim symbol table closes it), cross-referenced in its body (no duplicate issue,
mitigation #14). Change-scoped green: `cargo fmt --check`, `cargo clippy -p mycelium-transpile -D
warnings`, `cargo test -p mycelium-transpile`, `scripts/checks/markdown.sh`, `doc_refs_check.py`.
(kickoff `trx`; PR #1359; DN-34 §8.21 / DN-99 #72,#85 / DN-100; VR-5 / G2 / house rule #4.)

### docs(adr-045): Bucket-B descriptive freeze-doc sweep — correct stale "frozen" claims to the unfreeze (2026-07-10)

Lands the **Bucket-B** follow-on FLAGged by the ADR-045 ratification entry below: a purely
**descriptive** (append-only) correction of prose that still described the kernel/L0/L1/lexicon as
FROZEN, now pointing at the ratified **whole-project unfreeze** (**ADR-045**, Accepted 2026-07-10).
No Accepted/Enacted decision body is rewritten and no status is changed — only framing text and new
forward-reference pointers are added. Touches: **`docs/CURRENT-STATE.md`** (the "Kernel FROZEN"
status paragraph, the `frz`-kickoff historical note, and the kernel-freeze-condition-#3 open-decision
note — all now read "unfrozen for the ADR-045 gap-closure window; end-state (ADR-042) unchanged;
re-freeze bounded by the DN-99 worklist"); **`README.md`** (a new direction-note paragraph under the
ADR-042/ADR-043 blurb recording the temporary unfreeze); the **25** `mycelium-std-*` crates' DN-66
"Stability" doc-comment headers (comment-only additions — zero code
changed, verified by diff and `cargo fmt --check`), each gaining a paragraph noting the DN-66 freeze
is lifted for the window per ADR-045 while the "spec amendment + changelog entry, not a silent edit"
discipline (G2) holds; **`.claude/memory/lang-lexicon-syntax.md`** and
**`.claude/memory/language-execution.md`** (the L0-layer-cake "frozen" references — the unrelated
`hash-frozen`/migration-verdict senses of "frozen" elsewhere in these files are deliberately
untouched, out of scope); and an append-only **RFC-0001** footnote (following the existing
RFC-0034/ADR-032 footnote precedent) recording the L0-floor freeze lift without touching the RFC's
r0–r5 decision history. Verified: `git diff` confirms every crate-header change is a `//!` line
addition only (no code); `cargo fmt --check` clean on all 25 touched crates. (ADR-045 Bucket-B;
VR-5/G2/house rule #3.)

### docs(adr-045): ratify the whole-project unfreeze for early gap-closure (Draft → Accepted) (2026-07-10)

Ratifies **ADR-045 — Whole-Project Unfreeze (L0 Core IR → L1 kernel → L2/L3 grammar → stdlib lexicon) for
Early Gap-Closure** (Draft → **Accepted**, maintainer-directed 2026-07-10). Broadens the original
kernel+lexicon Draft to the **whole project** and resolves its three scoping questions: **OQ-1** L0 Core IR
(RFC-0001) **in scope** (overriding the Draft's keep-L0-frozen lean — recorded per house rule #4, not
erased); **OQ-2** L2/L3 surface grammar **in scope**; **OQ-3** the window is bounded by the **DN-99 residual
worklist** (the 4 open + 12 partial register rows, tracked as the M-1024…M-1034 `enb` backlog under E28-1)
exhausted, a DN-56/DN-76-successor scorecard re-scored green, and a follow-up maintainer-ratified ADR
reinstating the DN-39-only diff policy — re-score owner the maintainer. Landed-basis: maintainer-ratified
whole-project unfreeze, 2026-07-10. Applies the append-only cascade (Bucket-A/E): dated "Amended by ADR-045
(Accepted)" status pointers on **ADR-042 §2.1(a)**, **DN-56 §9/§6**, **DN-66 §2** (decision bodies
unchanged), and records RFC-0001's L0-floor lift; the ADR-042 §2.1(b) END-STATE (zero foreign first-party
languages, kernel included) plus the DN-39 TCB-admission boundary plus the never-silent/honesty discipline
are all **retained**. **`Accepted → Enacted` is withheld** — the §2.4 re-freeze conditions are not met (the
window is open). The Bucket-B descriptive sweep (`CURRENT-STATE.md`, `README.md`, ≈25 std-crate "DN-66
freeze" headers, `.claude/memory/*`, an RFC-0001 footer) is FLAGged as a coordinated follow-on PR. (ADR-045;
amends ADR-042 / DN-56 / DN-66 for the window; RFC-0001 L0-floor lift recorded; VR-5 / G2 / house rules #3/#4/#5.)

### docs(dn-99): surface-gap closure register + plan for the spw / RFC-0031 stdlib-port wave (2026-07-10)

Adds **DN-99 — Surface-Gap Closure Register and Plan** (Draft), the single closure register for the
`spw` stdlib-port wave: 92 enumerated surface gaps, each with a status
(open/partial/already-closed/transpiler-only/idiom), an `Empirical` evidence cite
(`file:line` / `M-id` read against dev tip `6d906b76`), a layer, a closure approach, a DoD, a DN-flag,
a tracking ref, a semcore-lane collision class, a size, and a priority. Foregrounds the already-closed
set (§3) so landed work is not re-opened — the Float correction (ADR-040 Enacted 2026-07-02; `flt.*`
prims registered) refutes the stale ".myc has no float surface" flag (mitigation #14). Ranks a two-track
closure plan (§4) toward the zero-hand-port north star: language `enb` closures (grammar/kernel/runtime)
and transpiler closures, under the whole-project unfrozen posture (ADR-045). Attests completeness
honestly (§5): one targeted spot-verify round; residual uncertainty `Declared`, never claimed `Proven`.
The §8 `enb` backlog is filed as **M-1024…M-1034** under epic E28-1 — each cross-references M-1023's
Wave-0 consolidation, and those touching `mycelium-l1` note "cloud-semcore-lane, coordinate M-1013"
(unfrozen-actionable, not deferred-by-freeze). Enacts nothing; moves no other doc's status. Authored
READ + DN only. (DN-99; RFC-0031; E28-1; VR-5 / G2 / house rule #3.)

### fix(semcore): complete the `Wrapping` Expr-variant port across the self-hosted `.myc` frontend (2026-07-10)

Closes a pre-existing transparency gap in the self-hosted compiler. `ast.rs::Expr` has 19 variants
including `Wrapping(Box<Expr>)` (the M-791 wrapping-arithmetic opt-in), but the five `.myc` compiler
nodules (`ast`/`ambient`/`totality`/`parse`/`semcore`.myc) each declared an 18-variant `Expr` with
`Wrapping` absent — while `semcore.myc` claimed to mirror `ast.myc::Expr` "verbatim, field-for-field."
That claim was false and the omission carried no FLAG (surfaced under house rule #4). This change adds
`| Wrapping(Expr)` between `Spore` and `Consume` (matching the Rust variant order) in all five `Expr`
decls, and a `Wrapping` arm to every exhaustive Expr-walker — so the type now genuinely mirrors the
reference AST and every traversal handles the variant (wildcard-free; a future variant is still a
`myc check` error, G2). Most walkers recurse into the single child exactly like `Consume` (`resolve_expr`,
`collect_calls_expr`, `descend_walk`, `collect_tuple_arities_expr`, `fvw`); `print_expr` renders the
block surface `wrapping { <expr> }`; the `walk_expr` structural fingerprint uses a NEW unused tag
(`0x6E`, identical in `parse.myc` and `ambient.myc`) so no existing Stage-4 fingerprint hash shifts.
**One non-mechanical arm — `grade`:** `wrapping { … }` attests **`Declared`** as a leaf (RFC-0034 §10 —
the enclosed modular ops are the developer's explicit opt-in, never upgraded past that basis), matching
`Spore`/`Wild` and the Rust `Expr::Wrapping(_) => Declared` — it does NOT inherit the body's grade the
way the grade-transparent `Consume` move does. The `compiler_stage5_freevars` differential gains a
`Wrapping` fixture (`free_vars(wrapping { x }) == ["x"]`) pinning the `fvw` arm against the live
`mono::free_vars` oracle. **Never-silent remaining work (FLAG-semcore-37 / FLAG-parse-12 / FLAG-ast-9):**
the self-hosted lexer/parser does not yet *produce* a `Wrapping` node (no `wrapping` keyword in
token/lex, no parse rule) — the AST is deliberately ahead of the surface; parsing `wrapping { e }` is
deferred. The five-way `Expr` (and keyword/token) duplication this touch exercised is flagged for a
future DRY decision. Verified: `myc-dogfood --strict` green (9/9 nodules), `cargo test -p mycelium-l1
--lib` green (471 passed), `clippy -D warnings` + `cargo fmt` clean, `markdown` gate clean. Graded
`Empirical`. (M-1013 / M-791; E18-1; RFC-0034 §10; VR-5/G2.)

### feat(transpile): M-1006 Phase-2 — path-qualified batch output (whole-corpus-completeness) (2026-07-10)

Lands the whole-corpus-completeness follow-on noted in DN-34 §8.19 (kickoff `trx2`, E33-1). The
transpiler's directory/batch mode named each output by **file stem only**, so a whole-`crates/` run had
every crate's `lib.rs`/`mod.rs`/`error.rs` overwrite the previous one — last-writer-wins, loudly warned
but **lossy** (~25 of 337 files clobbered), which blocked keeping every emission in an automated
multi-crate translation wave. Outputs are now **path-qualified** by mirroring the source tree under the
out-dir (a file's path relative to the batch root becomes its output path — `mycelium-core/src/lib.myc` vs
`mycelium-std/src/lib.myc`), injective by construction so the collision cannot occur. The `crates/` run now
writes all **337** `.myc` files with **zero** collision warnings. Zero committed churn: the 17
`gen/myc-drafts/` targets are flat single-crate `src/` dirs, so their mirrored path reduces to the bare
stem — byte-identical to the prior flat naming. Files: the `mycelium-transpile` CLI bin plus a pure,
unit-tested `batch::output_rel_path`; DN-34 §8.20. Emission-plumbing only — no transpilation
logic, metric, or guarantee tag moves (emission stays `Declared`). Verified: change-scoped `cargo fmt` /
`clippy -D warnings` / `test -p mycelium-transpile` green (63 tests, +5 for `output_rel_path`).

### feat(transpile): M-1006 Phase-2 opens — whole-corpus profile + honest file-linkage taxonomy (2026-07-10)

Opens the M-1006 ladder's Phase-2 (expand the Rust→Mycelium rip-through beyond the 17-target port
surface — kickoff `trx2`, E33-1). Ran the transpiler's first **whole-corpus** profile
(`mycelium-transpile crates/`: 337 files, 0 parse failures, 5,472 items, 573 emitted) and used it to
correct a gap-taxonomy category error surfaced by verify-first profiling of the opaque `Other` bucket
(37.8% of all gaps). Two sub-populations there are **not translatable library surface**: bodyless
`mod foo;` declarations (file-linkage — the module tree is implicit in Mycelium's nodule-per-file
layout, so the sibling file transpiles as its own nodule) and crate/file-level inner attributes
(`#![…]`, which are not `syn::Item`s). Added two honest `Category` variants, **`ModuleDecl`** and
**`InnerAttr`**, and excluded `ModuleDecl` from the expressible-fraction denominator on the identical
rationale `TestItem` already carries (recorded, never dropped — G2; but not counted against coverage).
An inline `mod foo { … }` (a dropped real body) stays a counted `Other` gap — VR-5: the denominator
only ever shrinks by genuinely-non-surface items. **Metric effect (never-silent):** whole-corpus `Other`
2,245 → 1,924; expressible 10.8% → 11.3%. On the committed `gen/myc-drafts/` 17-target manifest,
union expressible 13.4% → 14.1% and `checked_fraction` 7.8% → 8.2% — the numerator (59 myc-check-clean
items) is **unchanged**; the whole move is the tighter, more-correct denominator, no emission added or
upgraded. Files: `crates/mycelium-transpile/src/{gap,transpile}.rs` plus a data-driven `taxonomy` test
module; `gen/myc-drafts/` manifest regenerated (deterministic); DN-34 §8.19 records the baseline and the
Phase-2 residual worklist (`Import` 1,085 = the cross-nodule symbol-table prerequisite, the largest
remaining automation lever; a path-qualified batch-output layout as the whole-corpus-completeness
follow-on). Verified: change-scoped `cargo fmt`/`clippy -D warnings`/`test -p mycelium-transpile` green
(58 tests). Emission `Declared`; vet `Empirical`; DN-34 stays `Draft`.

### build(checks): auto-reflow the MD004 soft-wrap `+`/`*`-at-line-start pitfall (`just md-fix`) (2026-07-10)

The recurring MD004 false-positive — prose that soft-wraps so a `+`/`*` operator lands at line start,
which `markdownlint` reads as a list item and fails the `markdown` gate (CLAUDE.md §"Markdown
authoring") — now has a safe automated fix. `scripts/checks/md_wrap_fix.py` (invoked via **`just
md-fix`** and a `repo: local` pre-commit hook) is **findings-driven**: it asks `markdownlint` for the
exact MD004 lines and reflows only those, lifting the flagged marker off line-start (behaviour-neutral
for prose — the wrapped and reflowed forms render identically). It is safe by construction: a green doc
is a no-op (verified across all 459 tracked docs), so a legitimately-`+`-listed doc such as DN-15 is
untouched; and it **reports, never rewrites,** any finding that resembles a real list (the previous
line is a list intro or item, or the next line is another item at the same marker). `markdownlint
--fix` is deliberately NOT used — it normalizes marker *style*, so under the repo's `consistent` MD004
it rewrites genuine `-` lists to match the prose-wrap marker (verified: a real `-` list became a `+`
list), i.e. a green gate over corrupted content. The `markdown` gate's failure message now points at
`just md-fix`, and the CLAUDE.md pitfall note documents the tool. Verified: the fixer reflows a planted
`+` wrap and re-greens the doc, leaves a colon-introduced `+` list for manual review, and is a no-op on
the whole corpus; `ruff check`/`ruff format` clean. (Tooling/DX; VR-5/G2.)

### E18-1 semcore self-hosting — Stage-5 continues: M-1013 checkty PR-3 (`subst_type_param_in_typeref`) (2026-07-10)

The M-993 staged port of `lib/compiler/semcore.myc` advances one increment: `checkty.rs`'s
**`subst_type_param_in_typeref`** — the DN-54 §10 Model-A rule-instantiation substitution (M-973) that
replaces a rule's type parameter with a concrete `TypeRef` throughout a `TypeRef`, differential-
witnessed via the DN-26 §10.2 harness-marshalling method (the real Rust oracle vs. the `.myc` port,
decoded and compared with Rust's own derived `==`) and green under native `myc check`. A bare nullary
`Named(param, [])` occurrence becomes the concrete type's base, keeping the occurrence's own guarantee
where written and otherwise inheriting the concrete's — the `tr.guarantee.or(concrete.guarantee)`
first-Some-wins merge, ported as the `strength_or` helper; every structural form (`Named` arguments,
`Seq` element, `Fn` arrow, `Tuple` elements) recurses; the repr / `Substrate` / `Bytes` / `Float` /
VSA / dense / `Ambient` atoms carry no nested type-name and clone verbatim. The port is a complete,
wildcard-free 12-arm `BaseType` match (a future `BaseType` variant is a `myc check` error, never a
silent pass-through; G2), and like the Rust oracle it takes no recursion budget (the `subst_ty`
structural-substitution precedent). No new FLAG: `BaseType` is already a field-for-field mirror, the
Rust `if name == param && args.is_empty()` guard is expressed one level at a time (the single-level
pattern convention), and the argument-list recursion reuses the direct-recursive-map idiom
(FLAG-semcore-5, no generic `map`). The Rust oracle was widened `pub(crate)` (zero logic change) for the
differential. The gate extends `crates/mycelium-l1/src/tests/compiler_stage5_tyref.rs` (now 11 tests)
with `subst_type_param_in_typeref_cases` (19 cases: every `BaseType` arm; the four guarantee-merge
corners of bare/bare, occurrence-tagged, concrete-tagged, both-tagged; the guard's negative corners of
nullary-miss and applied-`name == param`; and nested-guarantee preservation) plus a
`subst_marshal_discriminates` non-vacuity twin. This is the first Stage-5 differential to marshal a
guarantee-bearing `TypeRef` on the INPUT side (the shared `encode_typeref` discards the slot), so it
adds a guarantee-threading `enc_tr` encoder; the output guarantee is already checked by the file's
`decode_typeref`. Verified: `cargo test -p mycelium-l1 --lib` green (469 → 471 passed), `clippy -p
mycelium-l1 --tests -D warnings` clean, `cargo fmt` clean, `myc-dogfood --strict` green (all 9
self-hosted nodules). Graded `Empirical` (differential agreement); DN-26 stays **Draft** (→ Resolved
with M-741). Unlocks the DN-54 `subst_type_param_in_{sig,expr,impl}` family as the next rungs.
(M-993/M-1013 checkty PR-3; E18-1; DN-26; DN-54; VR-5/G2.)

### E18-1 semcore self-hosting — Stage-5 continues: M-1013 checkty PR-2 (two pure checkty classifiers) (2026-07-10)

The M-993 staged port of `lib/compiler/semcore.myc` advances one increment: two PURE `checkty.rs`
classifiers, each differential-witnessed via the DN-26 §10.2 harness-marshalling method (the real Rust
oracle vs. the `.myc` port, decoded and compared with Rust's own derived `==`) and green under native
`myc check`. **`paradigm_name`** (checkty.rs 7175-7197) — the swap-paradigm name of a representation
type (`Binary`/`Ternary`/`Dense`/`VSA`, `None` for every other `Ty`) — ports as a flat 11-arm match
with every arm enumerated explicitly (no wildcard, so a future `Ty` variant must force a paradigm
decision rather than fall silently to `None`; G2). **`cons_list_ctors`** (checkty.rs 3592-3624), the
two-constructor linked-list recognizer (one nullary "nil"; one binary "cons" whose second field is the
recursive `Data(name, ..)` self-reference at the type's own arity), ports as a threaded fold
(`cons_list_scan`) that reads the type registry through the already-established FLAG-semcore-4
`Vec[DataInfo]` assoc-list (`types_lookup` standing in for `BTreeMap`). Neither adds a NEW
surface-inexpressibility: `paradigm_name` uses the standard `&'static str`→`Bytes` idiom, and
`cons_list_ctors` reuses the existing assoc-list registry convention. Both Rust oracles were widened
`pub(crate)` (zero logic change) for the differential. The new gate
(`crates/mycelium-l1/src/tests/compiler_stage5_classify.rs`, 5 tests) builds its `DataInfo` fixtures by
extracting them from a parsed + checked `env.types` (never hand-built, so a marshalling bug cannot hide
behind a hand-typed mismatch); it covers all 11 `paradigm_name` arms plus 11 `cons_list_ctors` shapes
(match, arity-parametric match, wrong-second-field, different-data-second-field, three-ctor, one-ctor,
missing name, non-`Data`, the one-field and three-plus-field scan paths, and the no-nullary
`(None, Some)` / `(None, None)` finals), with a non-vacuity `*_discriminates` twin per direction. A
three-lens adversarial review (faithfulness, coverage, honesty) found no divergence from the oracle.
Verified: `cargo test -p mycelium-l1 --lib` green (464 → 469 passed), `clippy -p mycelium-l1 --tests -D
warnings` clean, `cargo fmt` clean, `myc-dogfood --strict` green (all 9 self-hosted nodules). Graded
`Empirical` (differential agreement); DN-26 stays **Draft** (→ Resolved with M-741). (M-993/M-1013
checkty PR-2; E18-1; DN-26; VR-5/G2.)

### spw Wave-0 stdlib-port pilot — std.numerics / std.time / std.content self-hosted to `lib/std/*.myc` (2026-07-09)

The `spw` Wave-0 pilot validates the parallel `.myc` dogfooding-port loop end-to-end: three unported
stdlib crates ported to self-hosted `lib/std/{numerics,time,content}.myc`, each with a
`crates/mycelium-std-conformance/tests/std_<mod>.rs` **three-way differential** (L1-eval ≡ L0-interp ≡
AOT, TV-checked) **plus** a live Rust-oracle comparison — the agreement earns **`Empirical`**, never a
stronger tag (VR-5). Each was independently, adversarially re-verified (accept, not forced-green; tags
and gaps confirmed honest). Landed (M-1020 / M-1021 / M-1022):

- **`std.numerics`** (`lib/std/numerics.myc`, 225 lines; 63 differential cases): the honesty-crux
  STRENGTH surface — the Guarantee/BoundBasis strength-lattice (rank/meet/meet_all/basis_strength),
  the `Approx[A]` carrier, and the `NumErr`/`CheckErr` variant sets. The dominant float-valued ε/δ
  magnitude surface stays Rust (no scalar-Float VALUE in the `.myc` runtime yet — FLAGged to `enb`);
  the sealed FR-N3 `ProvenThm` witness (`Approx::proven`) was **omitted rather than ported ungated**,
  refusing to fabricate a `Proven`-strength escape hatch (VR-5).
- **`std.time`** (`lib/std/time.myc`, 388 lines; 29 cases): the full value-semantic surface — the four
  instant/duration value types, the complete comparison surface (signed `lt_s`, uncapped), the
  deterministic `ManualClock`, the declared-effect wrappers, and the 11-row guarantee matrix. Signed
  128-bit duration/instant **arithmetic** is blocked by the kernel's `TC_MAX_WIDTH=64` two's-complement
  cap (FLAGged to `enb`); only the comparison half is portable today.
- **`std.content`** (`lib/std/content.myc`, 521 lines; 47 cases): the content-addressing surface —
  `digest_eq` (via M-912 `bytes_eq`), the `ContentRef`/`RefKind` accessors, the hand-rolled recursive
  `parse_ref` / `content_ref_from_str` byte scanners, the 7-row guarantee matrix, and the
  `NameRegistry` read/write surface (an assoc-list redesign, never silently substituted).
  `hash_of_value` / `hash_of_def` stay kernel-bound (the structural-hash normalizer, RFC-0031 D1 —
  FLAGged to `enb`).

**STEP-0 transpiler finding (honest, `Empirical`, disconfirming):** re-running the CURRENT transpiler
on all three targets showed **ZERO checked-% delta** vs the committed manifest — numerics 7.4%, content
14.3%, time 18.9%, all unchanged. The two emitter features that landed since the manifest base (DN-51
narrow-cast→truncate; D3 operand-type inference through paren/reference wrappers) are real but
**orthogonal** to these modules' gap classes, so net transpiler-assist to the shipped nodules is ~0%
(only already-clean draft enums/types graduated verbatim; the rest is hand-ported). This is the M-991
"scaffold-not-porter" verdict holding, recorded plainly (VR-5/G2).

Enabler blockers surfaced by the ports are FLAGged to the `enb` epic (E28-1), not forced or silently
dropped (G2): no scalar-Float VALUE in the runtime (the dominant numerics blocker — already tracked as
Gap A / M-895 / M-896 / ADR-040); the `TC_MAX_WIDTH=64` signed-arithmetic cap (the dominant time
blocker); no top-level `const` item; no slice/array (`&[T]` / `[T]`) type; no unit/`()` return; no
sealed/private-visibility primitive (blocks the FR-N3 capability-gate); and no `bytes.find` /
`split_once` prim — plus the still-open cross-nodule `use`-import gap (sidestepped here by the
local-mirror convention). Consolidated as **M-1023** under `enb`. **Retirement (ADR-043): NOT
triggered** — all three are honest partial ports (the Rust oracle crates are not fully replaced), so no
Rust crate is retired. Verified: `myc-check` clean on all three nodules; `cargo test -p
mycelium-std-conformance --test std_numerics --test std_time --test std_content` green; `cargo fmt` and
`clippy -D warnings` clean. (M-1020 / M-1021 / M-1022; E33-1; ADR-042 / ADR-043; RFC-0031 D5/D6;
VR-5/G2.)

### chore: clean-snapshot prep — archives extracted to the `archive` branch, indices regenerated (2026-07-09)

Prep a lean trunk ahead of a promotion: historical archives moved out of the day-to-day tree but
preserved for reference, not deleted. Created the persistent **`archive` git branch** at the
pre-removal `dev` tip (the full corpus lives there byte-for-byte, permanently), then removed
`docs/archive/` (46 files) and `.claude/kickoffs/archive/` (33 files) from this branch — exactly
those two trees. Every dangling reference a repo-wide grep found (`CHANGELOG.md`,
`docs/CURRENT-STATE.md`, `docs/Doc-Index.md`, `docs/adr/README.md`, the
`docs/reviews/2026-06-14-deep-review/06-medium-findings-ledger.md` pointer stub,
`.claude/skills/changelog/SKILL.md`, `.claude/kickoffs/{dfb,README}.md`, `.claude/agent-context.md`)
now says "now on the `archive` branch" instead of pointing at a dead in-tree path; historical
narrative is preserved with an added pointer, not rewritten. `scripts/checks/links.sh` additionally
caught and fixed 5 relative hyperlinks to the removed directory in `.claude/kickoffs/README.md` that
the initial path grep missed. Dropped 8 dangling `src:.claude/kickoffs/archive/*.md` `doc_refs` entries
in `tools/github/issues.yaml` (a mechanical repair — no `doc_refs` grammar prefix can address a
branch; each was one line in a multi-entry list). Regenerated all three committed indices on the
lean tree (`just docs-index`, `just tero-index-gen`, `just lib-index-gen`); all three drift gates
green. Backfilled 8 missing Design-Note rows in `docs/Doc-Index.md` (DN-90…DN-97 — discovered
missing during this pass) and refreshed the stale 2026-07-01 corpus-count digest in
`docs/CURRENT-STATE.md` (27/39/72 → **33 ADRs / 41 RFCs / 97 design notes**). Verified:
`doc_refs_check.py` OK; `markdown.sh` 0 errors (450 docs); `links.sh`/`structured.sh`/`secrets.sh`
(gitleaks) clean.

### DN-97 ratified (Rank 1) — the unified branch/merge/propagation workflow + `/forward` + `/sync-down` skills (2026-07-09)

The maintainer ratified **DN-97 → Accepted** in-session as **Rank 1**: three **same-content**
persistent trunks (`dev`/`integration`/`main`, differing by rigor not tracked content) so **every**
merge — up and down — is a plain, conflict-free `--no-ff` with zero merge-driver machinery; a
lightweight production `main` is achieved as a **`just package-release`** artifact (`git archive` +
`.gitattributes export-ignore`), never divergent tracked content. DN-97 reconciles its two inputs —
**DN-95** (down-propagation; proves, within the git-DAG model, that squash-only + no-force-push +
persistent accumulation tiers makes lower-tier graph divergence mathematically unavoidable) and
**DN-96** (the forward/up-flow spec-first staged pipeline — spec/DN → public API → private API →
component seam-map → code — as a context-windowing mechanism, with bite-sized changes and precursor
doc/index branches for auto-generated bulk) — under the maintainer's ratified decisions, superseding
their standalone recommendations only where they conflict (append-only: DN-95/DN-96 stay **Draft**,
referenced not edited). Operationalized as two new skills plus a recipe: **`/forward`** (the staged
pipeline + the ≈1–2k-LOC soft / 4,000-LOC hard change-sizing cap + the precursor-branch mechanic),
**`/sync-down`** (`main`→`integration`→`dev` propagation as a plain no-force `--no-ff` merge per
tier, landed via a PR per tier so the repo's enforced branch-guard stays respected — never a raw
push, mitigation number 10), and **`just package-release [version] [ref]`** (skip-graceful, never-silent about
what `.gitattributes export-ignore` excludes). Both skills wired into `CLAUDE.md` §Commits & PRs and
§Skills. Verified: markdownlint 0 errors (529 docs at landing time), shellcheck clean (70 scripts),
gitleaks clean, branch-guard ok, `just --list` parses the new recipe, `package-release.sh` produces a
filtered tarball end-to-end against a local ref. (DN-95/DN-96/DN-97; VR-5/G2.)

### E18-1 semcore self-hosting — Stage-5 continues: M-1013 STEP 4/5/6 + the first eval.rs fragment (2026-07-09)

The M-993 staged port of `lib/compiler/semcore.myc` advances four more increments, each with a
live-oracle differential in an in-crate `src/tests/` module and green `myc-dogfood --strict` (all 9
self-hosted nodules). **STEP 4** ports `checkty::resolve_imports` (cross-nodule use-import
resolution, M-662) plus six helpers — the resolution half of the increment-8 wave whose registration
half (`register_types`/`traits`/`instances`) landed as STEP 3; adds an `ItUse(UsePath)` `Item`
variant so imports are read directly. **STEP 5** ports `register_nodule_decls` (deferred at STEP 4
on an over-broad fuse.rs-entanglement read that turned out narrower than assumed: only
`check_fuse_laws`/F-A2 touches the evaluator) plus the `checkty::prelude` Bool-registry seed and the
Fuse-trait vocabulary slice (F-A1: `TRAIT_NAME`/`prelude()` — pure data-shaping, no evaluator
dependency; F-A2 stays deferred). **STEP 6** ports mono.rs's remaining pure Ty/TypeRef round-trip
helpers (`mangle_ty_in_ty`, `item_key`, `closure_field_ty`, `closure_param_ref`,
`ty_to_source_ref`/`ty_to_ref`/`ty_to_ref_tagged`) plus the monomorphization work-item dedup key
(mirrored as `WorkItem` — `Item` is already taken at this nodule's ast level), resolving the
previously-deferred FLAG-semcore-17. **eval PR-1** triages `eval.rs` (increment 12, "the hardest
wave") for a bounded witnessable slice: the CEK work-stack machine itself is irreducibly
Env/PrimRegistry/SwapEngine/fuel/depth-entangled, but `Evaluator::try_match`'s
Wildcard/Ident/Ctor/Tuple leg is separately callable and ports as `lval_try_match` with a new `LVal`
mirror; `Pattern::Lit` is out of scope (needs the trusted kernel `Value`'s equality) and refuses
cleanly rather than guessing (G2). Each increment widens its target function to `pub(crate)`
(zero logic change, the established STEP-3 precedent) so the differential harness can call the real
oracle directly; `lib/compiler/README.md`'s stale Stage-5 wave-progress note is flagged, not
silently left wrong (mitigation #14). Every surface-inexpressible deviation carries a
FLAG-semcore-* note. Verified per-increment: `cargo test -p mycelium-l1 --lib` green throughout (437
→ 464 passed), `clippy -p mycelium-l1 --tests -D warnings` clean, `cargo fmt` clean,
`myc-dogfood --strict` clean. Does not close the "eval.rs deferred" line — the CEK machine's bulk
remains unported. (M-993/M-1013 STEP 4/5/6 + eval PR-1; E18-1; DN-26; VR-5/G2.)

### Kernel: `bit.truncate` prim (DN-51) + transpiler narrow-cast now emits it (2026-07-09)

Adds the explicit, total, lossy `Binary` narrow that DN-51 §2 D3/§6 names as `width_cast`'s
counterpart: `bit.truncate` unconditionally drops the high `N-M` bits and never refuses (unlike
`width_cast`'s checked narrow) — a maintainer-authorized DN-39 post-freeze promotion (Π 68 → 69).
Registered in the content-addressed prim table with an honestly-`Declared` intrinsic tag (DN-51 §4:
"own honest lossy tag — never `Exact`"; neither `Proven`'s checked-theorem nor `Empirical`'s
trial-fit basis applies to a deterministic bit-drop, VR-5); the L1 surface maps to it through the
same width-witness ABI as `width_cast`. Tested with interp-level properties (identity,
keeps-low-bits-mod-2^M, never-`Exact`, round-trip after widen, composition/absorption), an L1
three-way differential (L1-eval == L0-interp == AOT, including the direct
width_cast-refuses/truncate-succeeds contrast), and a Π surface-consistency guard. The
`mycelium-transpile` `Expr::Cast` narrow arm (`Binary{N} as Binary{M}`, `M < N`) now emits
`truncate(operand, <M-bit witness>)` instead of the `FLAG-truncate-not-emittable` gap, matching Rust
`as` narrowing's wrap semantics exactly; a live `myc-check` oracle test proves the emission genuinely
type-checks (narrow/widen/identity/narrow-to-`Binary{1}` cases). Verified: `cargo fmt`/`clippy -D
warnings`/`test` clean across `mycelium-core`/`mycelium-interp`/`mycelium-l1`/`mycelium-transpile`
(the AOT leg ran for real; the `myc-check` binary was built locally so the live oracle ran rather
than skipped); the two prim-inventory count assertions bumped 68 → 69. (DN-51; VR-5/G2.)

### Lint: `unsafe_code = "allow"` so documented unsafe never erroneously alarms — local, remote, or ad-hoc (2026-07-09)

Follow-on to the entry below, per maintainer direction ("applied for both local and remote checks so
you stop getting alarmed erroneously about intentional unsafe"). Set `unsafe_code = "allow"` in
`[workspace.lints.rust]`, making `clippy::undocumented_unsafe_blocks` the single unsafe check: a
documented, intentional unsafe block now passes **everywhere** — local `just check`, remote CI (`just
ci`, same suite), and even ad-hoc `cargo clippy -D warnings` with no exemption flag — while an
undocumented block (no `// SAFETY:` comment) is still caught. This removes the erroneous-alarm class:
previously `unsafe_code = "warn"` fired on *every* unsafe site under any `-D warnings` run lacking the
`-A unsafe_code` exemption, flagging intentional unsafe. What still guards unsafe is unchanged: the
trusted-base crates keep their **source-level** `#![forbid(unsafe_code)]` (which overrides this
workspace `allow`), `scripts/checks/scan.sh` enforces confinement + the per-site intentionality
marker, and `undocumented_unsafe_blocks` enforces the `// SAFETY:` comment. Verified: ad-hoc
`clippy -D warnings` (no `-A`) on `mycelium-mlir` now passes (0 errors, was 8); a synthetic
undocumented block is still caught; trusted-base `forbid` unaffected. No `.rs` changed. (ADR-014;
DN-21; G2.)

### Lint: enforce the mandatory `// SAFETY:` comment via clippy (ADR-014 refinement, 2026-07-09)

Maintainer directive: catch *undocumented* unsafe while allowing documented, intentional unsafe. Added
`clippy::undocumented_unsafe_blocks = "warn"` to `[workspace.lints.clippy]` — with the gate's
`-D warnings` this denies any `unsafe` block lacking a `// SAFETY:` comment, while the existing
`-A unsafe_code` exemption keeps intentional, documented unsafe passing. This turns ADR-014's "mandatory
`// SAFETY:` comment" from a convention into a checked requirement. Verified: all existing unsafe
(confined to `mycelium-mlir/src/jit.rs`, DN-21/u78 — 8 documented blocks) passes; a synthetic
undocumented block is caught. No `.rs` changed. (ADR-014; DN-21; G2.)

### trx2 Lane C — CU-3/5/7 kernel prims + transpiler operand-gated emission and forward-mapped prim table (2026-07-08)

Two scoped leaf→`dev` PRs close §8.16's in-progress worklist (DN-34 §8.17). Kernel (PR #1300, Π
66 to 68): CU-3 adds two never-silent Binary/Float conversion prims — `bin.to_flt` (checked-exact,
refuses `|n| > 2^53`) and `flt.to_bin` (width-witness shape, refuses NaN/inf/negative/fractional/
out-of-width), unsigned-magnitude (ADR-028), `Empirical` (ADR-040 §2.6), with full three-way
(L1/L0/AOT, AOT leg ran) tests; the lossy `bin to flt` rounding stays a reified swap (FLAG-cu3-lossy-
swap). CU-5 wires the `wrapping` eval-mode dispatch over `bin.add`/`sub`/`mul` (RFC-0034 §10; no new
prims), runtime half only — no `wrapping { }` parser surface yet (FLAG-cu5-surface-syntax). CU-7 is a
verify-first correction (mitigation #14): the assumed "40-trit cap on `trit.*`" was inaccurate —
`ternary::add`/`mul` are already arbitrary-width (digit-serial over `&[Trit]`), pinned by a width-80
three-way test; the growable value form stays gated on E20-1 (FLAG-cu7-e20-1-gate). Transpiler (PR
no. 1299, `checked_fraction` 5.79 to 7.76 percent, +15 items): `&`/`|` now emit `and`/`or` and `!=`/`>`
compose from the `eq`/`lt` prims (a house-rule-#4 correction — `ne`/`gt` are non-`pub` functions, not
prims) when both operands resolve to a known `Binary{N}` via a new type environment (a review-found
HIGH bug where the gate mis-fired on shadowed/pattern-bound names was fixed by env invalidation); a
new `prim_map.rs` forward-maps the known kernel surface with `flt_is_*` wired and `wrapping_*`
PENDING-BACKEND (mapped, never emitted); and a stale `f64` to `Float` `map_type` fix unblocked
`std-sys`'s libm wrappers. Emission stays `Declared`; vet figures `Empirical`. (trx2 E32-1/E33-1;
DN-34 §8.17; VR-5/G2.)

### M-1006 transpiler-hardening ladder — String→`Bytes` lands the largest `checked_fraction` gain (2026-07-07)

The Rust→Mycelium transpiler's `transpile → myc check → fix` gap-profiling ladder (kickoff `trx2`
E-B, epic E33-1) advances through three increments now promoted to the staging tier. **§8.12**:
positional named-field emission (`type Foo = Foo(T1, T2)`, names dropped as a `NamedFieldDrop`
sub-gap) gated by a greatest-fixpoint resolvability set so a record only emits when every referenced
type resolves in-file; plus the `myc check --phylum` cross-nodule probe. **§8.13**: `self.<field>`
projection desugars to a `match` on the struct's positional constructor, and struct literals to the
positional ctor call. **§8.14**: Rust `String`/`str`/`&str` map to Mycelium `Bytes` (grounded to
RFC-0033 §3.2 — `Repr::Bytes` is the ratified never-silent UTF-8 text repr; verify-first
`myc check`-confirmed), the **largest single-lever gain of the ladder** — union `checked_fraction`
4.61% to **5.79%** (35 to 44 items myc-check-clean) and `expressible_fraction` 8.29% to **11.45%**
over the 17-target draft corpus (`gen/myc-drafts/`, regenerated deterministically on this tier). Two
further levers (operator emission, inherent-impl dup-name rename) were verify-first-probed and
**deliberately not built** — measured zero corpus yield (house rule #4). The residual ceiling is
re-grounded to a frozen-kernel decision: RFC-0033 §4.1.2 mandates `Binary{N}` bitwise ops but the
Enacted/frozen kernel (DN-56, M-969) has no `bin.band`/`bin.bor`/`bin.bxor`, so completing them is a
DN-39 post-freeze promotion (flagged, not implemented). Emission `Declared`; vet verdict `Empirical`.
DN-34 stays **Draft** (append-only §8.12 to §8.14).

### E18-1 semcore self-hosting — Stage-5 increments (M-1007 → M-1013 PR-2) (2026-07-07)

The self-hosted L1 frontend (`lib/compiler/semcore.myc`) advances through the M-993 staged-port
ladder. Each increment carries a live-oracle Rust differential in an in-crate `src/tests/` module (no
logic `.rs` changed beyond `pub(crate)` widenings that expose the real functions as test oracles):
M-1007 type algebra; M-1008 `unify`/`resolve_ty` + synthetic-tuple helpers; M-1009 the mono
name-mangling family; M-1010 `free_vars`/pattern-binder analysis; M-1011 `lit_ty_of`/`literal_key`/
`normalize_pattern` (its `infer_type` residual deferred to the heavy core); M-1012 the L0
`Value`/`Repr`/`FieldSpec` mirror ADTs plus the pure elab lowering helpers under DN-26 §10 Option A;
then the M-1013 heavy-core opening — **STEP 2** adopts **harness marshalling** as the Stage-5
differential method (decode the port's mirror output into the real `mycelium_core` type and compare
with Rust's trusted derived `==`, retiring the hand-written `.myc` `_eq` comparators; DN-26 §10.2);
**PR-1** ports `resolve_ctors`/`first_duplicate`; **PR-2** ports `register_types` (trimmed tuple
pre-pass, with the fn-body/pattern/sig legs deferred behind the never-silent FLAG-semcore-30). Both
witnesses stay green: the Rust marshalling differential (`cargo test -p mycelium-l1`) and the native
toolchain (`just myc-dogfood --strict` → `myc check lib/compiler/semcore.myc`). Every
surface-inexpressible deviation carries a `FLAG-semcore-*` note (G2/VR-5). DN-26 stays **Draft**
(→ Resolved with M-741). (E18-1; M-1007/M-1008/M-1009/M-1010/M-1011/M-1012/M-1013.)

### mycelium-tero (DN-87 / E39-1) — transparent memory substrate & agent knowledge API (M-1015…M-1018, 2026-07-07)

The project corpus as a generated, provenance-carrying encoding, served platform-agnostically
(`crates/mycelium-tero`) — promoted as a self-contained crate.

- **M-1015 — Layer-1 deterministic corpus index.** A drift-gated `docs/tero-index/{INDEX.md,index.json}`
  (5119 rows on this promoted tier) over docs/research/issues/changelog/skills, generalizing the
  api-/lib-index pattern; an `Empirical/Declared` heuristic, source is ground truth (G2/VR-5).
- **M-1016 — query engine + mandatory provenance.** `QueryEngine` over the Layer-1 report; every
  answer carries a resolvable citation by construction (an uncited query is a typed refusal);
  EXPLAIN-able retrieval; Empirical latency ~630µs/query.
- **M-1017 — API fronts (MCP + HTTP, token-scoped) + skills.** One framework-agnostic core behind an
  `axum`/`tokio` HTTP front and a stdio MCP front (byte-identical answers, a 3-way parity
  differential); token-scoped auth (read-only default, refuse-to-start without tokens); four
  cross-platform skills (`.claude/skills/tero-*`). **ADR-044** adopts tokio+axum (the workspace's
  first async runtime, scoped to this tools-tier crate; no TLS pulled).
- **M-1018 — VSA Layer-2 + Empirical eval gate.** Layer-1 rows encoded as MAP-I hypervectors
  (role-filler bind/bundle, cleanup decode, provenance by construction) plus a committed eval
  harness. **Gate verdict: CLOSED** — Layer-2 does not beat Layer-1 (correctness@1 0.375 vs 0.625;
  ~26× slower latency), provenance 1.0; the system serves Layer-1 and the improved-on-RAG claim
  stays aspiration (DN-87 §6.1). Nothing was tuned to force a pass.
- Supply-chain: `tokio`/`axum` hoisted to `[workspace.dependencies]`, `THIRD-PARTY-LICENSES`
  regenerated, `mycelium-tero` registered in `deps-strata` (stratum 7 / tier `tools`). Follow-on
  **M-1020** minted: native HTTPS/TLS for the HTTP front.

*Note: the three entries above are the staging tier's already-curated/condensed summaries of the same
promoted work the wave continues to describe in granular form below (M-1013 STEP 2 through the DN-87
entries, and M-1006 phase-1); preserved here per the append-only union rule (never drop either side)
even though they narrate overlapping content — a follow-up editorial pass may consolidate.*

### M-1013 STEP 2 — semcore Stage-5 differential retrofit to harness marshalling (2026-07-07)

The Stage-5 self-hosting differential (`compiler_stage5_elab.rs`) switches from comparing the `.myc`
port's mirror output against the Rust oracle by hand-written `.myc` structural-equality comparators
(the FLAG-semcore-28 `_eq` family, now deleted from `semcore.myc`) to HARNESS MARSHALLING: a
never-silent Rust `decode_*` family rebuilds the real `mycelium_core` type from the port's `L1Value`
output and compares it with Rust's own trusted derived `==`. This removes the hand-mirrored
comparators from the trust path -- the comparator is now `mycelium_core`'s own
`#[derive(PartialEq)]`, the very thing the mirror was hand-restating. The M-1012 non-vacuity
discipline migrates from the comparator to the decoder (`marshal_discriminates`: mirror values
distinct in one dimension must decode to unequal Rust values). Derived `==` is the primary comparison
for this Bits/Trits/Bytes subset; content-hash is the recorded switch for the first float-bearing
increment. Both witnesses green: `stage5_elab` 11/11 with the full `mycelium-l1` suite, and native
`myc check lib/compiler/semcore.myc` ok. DN-26 §10.2 records marshalling as the adopted method
(append-only) -- the trust foundation for the remaining M-1013 increments. No logic `*.rs` changed.
(M-1013; E18-1; VR-5/G2.)

### M-1008/M-1009/M-1010 + M-1011(partial) — semcore pure-leaf port wave (M-993 ladder) (2026-07-07)

Four pure/leaf increments of the M-993 semcore SCC port land in `lib/compiler/semcore.myc`
(1721 to 2541 lines), each a faithful arm-by-arm port with a live-oracle differential vs the real
Rust function (verdicts computed at test time, discrimination probes, never hand-derived): unify /
resolve_ty / tuple helpers (M-1008), the mono mangle family (M-1009), free_vars / pattern_binders
(M-1010), and lit_ty_of / literal_key / normalize_pattern (M-1011). 47 stage5 differential tests
green; native `myc check lib/compiler/semcore.myc` ok; no logic *.rs touched (only new in-crate
test modules + an append-only DN-26 note). FLAG-semcore-13..21 document each surface-inexpressible
deviation (dead-arm collapses, value-threaded accumulators, ordered assoc-lists), review-verified
behavior-preserving. Honest deferrals: `infer_type` (a wrapper over the un-ported inference SCC) to
M-1013 -- M-1011 stays `todo` on that residual; `mangle_ty_in_ty`/`item_key` (private, no oracle) to
a later increment. M-1008/M-1009/M-1010 close; M-993 advances, not closed.

### M-1015 — docs/tero-index/: the whole-corpus Layer-1 citation index (DN-87 / E39-1) (2026-07-07)

mycelium-tero Layer 1 lands: the api-index/lib-index deterministic-index pattern generalized over
the WHOLE corpus, emitted to `docs/tero-index/{INDEX.md,index.json}` (5140 rows; grep-friendly
table plus machine index; never-silent `flagged` section — 6 entries, reference notes genuinely
lacking a Status row; uniform `Empirical/Declared` tag, "source is ground truth"). Families: docs
(RFC/ADR/DN/spec/guide/planning) with status, guarantee tag, and section anchors; research records;
issues.yaml (+idmap) with id/title/status/epic/depends_on/doc_refs/gh-number (526 rows, grep
cross-checked); CHANGELOG entries by date/id (83, cross-checked); skills name+description (16,
cross-checked); api-index and lib-index referenced as siblings, never duplicated. Reuses
`mycelium_doc::corpus::ingest` (no parallel markdown heuristic, no pub-surface widening); issues.yaml
read by a purpose-built YAML-subset reader (serde_yaml is neither in-workspace nor maintained —
no new external dependency). Deterministic (two-run byte-identical, incl. the self-referential
fixpoint fix: the index excludes its own output) and drift-gated (`scripts/checks/tero-index.sh`
via `just tero-index`; regenerate via `just tero-index-gen`); 27 tests. Integration note: the
`all.sh` COMPONENT_ID 5-bit space is exhausted, so `tero-index` deliberately shares `doc-index`'s
id 23 (same committed-index-drift family; the digest names the exact gate) — a 32nd unique id
needs a byte-scheme redesign, recorded as a deliberate open decision.

### mycelium-tero — DN-87 named; kickoff `mem` FIRED; the scaffold lands (2026-07-07)

The maintainer named the DN-87 program **`mycelium-tero`** — a quiet homage to **Atsushi Tero**,
for his contribution to science and engineering (the name is sugar; the system is code). This
entry lands the naming across the record (DN-87 title/Naming field, kickoff `mem` + kickoffs
index → FIRED, E39-1 → `in-progress`) and the swarm pre-flight scaffold: `crates/mycelium-tero`
(workspace-registered, buildable, tested stub carrying the DN-87 §6 binding invariants in its
crate docs). Wave 1 = the M-1015 lane (corpus ingestion + the deterministic Layer-1 index), then
M-1016/M-1017/M-1018 parallel per the kickoff's disjoint-lane table.

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

- **Seven kickoffs archived** (moved to `.claude/kickoffs/archive/`; that in-tree dir was itself
  further extracted to the persistent `archive` git branch 2026-07-09, clean-snapshot prep — see
  the entry near the top of this file): the six completed — `acy`
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

Entries 2026-06-27 and earlier (verbatim) are archived — see the `archive` git branch (was
`docs/archive/changelog/CHANGELOG-2026-06.md` in-tree; extracted 2026-07-09, clean-snapshot prep).
