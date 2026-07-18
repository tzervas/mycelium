# Program handoff — Design-pack steer capture, corrective guidance & execution program

| Field | Value |
|---|---|
| **Status** | **Maintainer-STEERED 2026-07-17** — deliberation complete on the four-doc design pack (`docs/planning/gap-analysis-2026-07-16/DESIGN-01..04`). This document is the **capture-for-ratification + execution handoff**. Decisions in §1–§2 are **binding maintainer steers**; no RFC/ADR/DN is Accepted until minted/amended per house rule. This document itself is `Declared`. |
| **Audience** | Implementing agent (**Claude Code**), operating under repo-root `maint-guide.md` + `.claude/kickoffs/` discipline |
| **Companions** | `CITATIONS-DESIGN-STEER-2026-07-17.md` (external evidence — cite as `[C#]`) · `KICKOFF-CLAUDE-CODE-DESIGN-STEER.md` (operating prompt) |
| **Grounds (repo)** | DESIGN-01..04 · RFC-0001/0002/0005/0012/0013/0018/0033/0034 · ADR-003/006/013/029/032 · DN-04/16/29/135 · `docs/spec/stdlib/swap.md` · `docs/planning/gap-analysis-2026-07-16/PARTITION.md` · `maint-guide.md` |
| **Supersedes / relates** | Re-sequences `PROGRAM-HANDOFF-ONESHOT.md` (transpile resume now gated behind Phase 3 decomposition). Four DESIGN docs remain the design source; this doc records their steering-board answers. |
| **Honesty** | Steer register entries = maintainer decisions (binding). Rationale summaries = `Declared`. External claims carry `[C#]` refs. Numeric retention defaults are `Declared` placeholders until the Phase-2 sizing pass upgrades them (`Empirical`). |

---

## 0. Prime directives (unchanged; agent must not violate)

| # | Gate | Source |
|---|---|---|
| N1 | Never-silent (G2) — every refusal/decision observable | corpus-wide |
| N2 | No guarantee upgrade without basis (VR-5) | RFC-0018 §3.2 |
| N3 | No silent / auto `swap` (S1) — no elaboration step may introduce a `Swap` | RFC-0018 §3.3 |
| N4 | Policy identity recorded whenever selection runs | RFC-0005 §3 |
| N5 | Contamination stops at walls — no laundering, no global quality kill | DESIGN-02 |
| N6/**N9** | First-fault localization — where/how/why in one hop, never a tree dig | DESIGN-03 |
| N7 | Generation ≥ middle tier in every mode; consumption dialable | RFC-0034 §7 (normative) |
| N8 | Deterministic tables before sugar; walls before elision | DESIGN-03 |
| H1 | DN/ADR numbers minted only after free-ID check; decision corpus append-only | house rule |
| H2 | Never claim `Accepted` without maintainer ratification date | house rule |
| H3 | Never rewrite git history; fixes are forward commits | house rule |
| H4 | Do-not-lift list respected (e.g. `embeddonator`) | `big_ternary.rs` header |

---

## 1. Steer register — 2026-07-17 (binding)

### 1.1 Pack 01 — Swaps & policy

| Q | Decision | Rationale / grounds | Implementation notes |
|---|---|---|---|
| **P1-Q1** | Spelling is **`policy: ambient`**. Explicit catalog-name spelling (`policy: <name>`) stays legal. `policy: _` and `policy: auto` are **rejected vocabulary**. | "Ambient" is the corpus's ratified word for a *declared, scoped* default resolved by precedence that elaborates to identical L0 + identical content hash (RFC-0012, NFR-7). RFC-0034 §6 already reuses the ambient mechanism for CertMode; policy becomes the third instance, retention (§1.4) the fourth. `default` imports an unexamined-fallback prior that contradicts ADR-006/G2. [C3][C5] | Resolution law: resolve through the RFC-0012 scoped precedence (global ⊐ phylum ⊐ nodule, most-specific-wins) against declared ambient policy + catalog pair-registration; **unresolved = hard error** ("no ambient policy declared for this pair in scope"), never a fallback. Always record resolved `PolicyRef` hash + EXPLAIN origin (`declared@nodule` / `declared@phylum` / `catalog`). Conformance: `expand(policy: ambient) ≡ longhand`, **same content hash** — CI golden per RFC-0012 NFR-7. |
| **P1-Q2** | Keep **explicit `Swapped`** as the authoring model until X15 lands (confirms the standing `swap.md` §7-Q2 disposition). **Close the cert-discard gap ASAP** via diagnostic sink + certificate **handle** — do **not** widen the `SwapEngine` trait. | The trait boundary discards certificates today (`mycelium-cert/src/mode.rs` — "the trait's `swap` discards it"). Widening the trait re-imports the P2 threading ceremony. Handle-plus-store mirrors the existing `Meta.policy_used: ContentHash` pattern. Cert cost lessons: [C6][C7][C8]. | Seams: (a) `impl SwapEngine for ModeGatedSwapEngine` (`mode.rs`) — the `NotValidated` branch is the **first `swap_check` emitter site**; the validated path emits a first-fault-bus `swap_check` crumb + cert handle. (b) Add `cert: Option<ContentHash>` to `Meta` beside `policy_used` (excluded from content hash like all `Meta` — RFC-0001 §4.6). (c) Certificate **bodies** go to a mode-gated content-addressed store (emit modes only), capped per §1.4 P4-Q1. X8 cert-ambient later reduces to surface sugar over this. |
| **P1-Q3** | **Allow `to:` elision**, mechanically gated ×3; ships in AX-sugar (X9), after walls. | Bidirectional expected-type elaboration is sound when unique; explicit-conversion precedent [C1][C2][C3][C34]; RFC-0012 gives the free conformance proof. | Gates: (1) **uniqueness-or-refuse** — exactly one legal pair consistent with the expected type, else hard error listing candidates as never-auto-applied suggestions (X11 posture, rustc `Applicability` analog [C20]); (2) elaboration-hash CI goldens (elided ≡ longhand); (3) regime typing computed from the **resolved pair**, never the spelling — partial pairs type `Result` with or without `to:`. |
| **P1-Q4** | **Dedicated Swap Ergonomics DN** (mint after free-ID check). M-540's per-ring ergonomics pass **consumes** the ratified DN; it is not the vehicle. | Policy streamline is the elevated primary package; it needs a citable ratification home. `Blocked-Decisions-Ratification-Map.md` then routes M-540 at the DN. | DN contents checklist in §4.1. Include the **layer reconcile**: kernel-level `dec` is `Option`-typed (RFC-0002 §4); std surface is `Result`-only (DN-16 pin); the regime table names which layer owns which shape so A5 wording cannot drift. |

### 1.2 Pack 02 — Tags, Meta & containment

| Q | Decision | Rationale / grounds | Implementation notes |
|---|---|---|---|
| **P2-Q1** | Remint bases v0 = **Swap certificate** (existing RFC-0018 §3.3 endorsement) **+ total Exact-decidable predicate**. Trial-basis `Empirical` remint deferred to a later, separately-audited, separately-EXPLAINed channel. | Minimal extension of the existing only-raise rule; every release path is attack surface (declassification discipline [C13]); shipped analog: sanitizer-gated remint (Trusted Types pattern, ◆pointer) + "parse, don't validate" [C9]. | `std.airlock` v0: predicate must be total and Exact-decidable; laundry (empty basis) is refused by construction with `seal_remint` first-fault. |
| **P2-Q2** | **Export-only seal first** (no `Quarantined[T]` carrier in v0) — **plus SPIKE S1**: timeboxed design spike on the carrier as prep for probable future need. | Coarse boundary enforcement delivers the wall at lowest ceremony (JIF/LIO burden evidence [C11][C16][C17]); maintainer judges future need probable → spike now, implement later. | **S1 spike output** (design note appendix, `Declared`): carrier type sketch (`Quarantined[T]` as zero-cost newtype over graded type), interaction with meet rules R4/R5, ceremony cost estimate, migration path from export-only seals, and **adoption trigger criteria** (what observed pain activates implementation). No production code from the spike. |
| **P2-Q3** | **Yes** — in certified colonies, fast-mode spores are admitted **only via explicit airlock**. | Upgrades RFC-0034 §6 cross-mode visibility to *governance*; clearance analog [C11][C12]; spore envelope already carries min-grade + mode (ADR-013 / DESIGN-02). | Colony admission check reads the spore envelope; refusal emits `mode_firewall` first-fault. |
| **P2-Q4** | **Meet free inside the nodule only**; phylum crossing is a boundary decision + EXPLAIN — **plus SPIKE S2**: timeboxed spike on phylum-wide free meet for probable future need. | Smallest defensible blast radius; matches finest tier of the RFC-0012/0034 scoping precedence; keeps RFC-0018 §4.5 (implicit flows — still-open maintainer decision) local. | **S2 spike output** (`Declared`): blast-radius analysis of phylum-wide free meet, interaction with the RFC-0018 §4.5 implicit-flows decision (resolve or explicitly couple), boundary-table delta, and **trigger criteria** (e.g. measured boundary-ceremony cost inside cohesive phyla). No rule change from the spike. |
| **P2-Q5** | **DN-141 rewrite** — pack 02 ratifies as DN-141's body. DN-142 not minted (speculative granularity). | Pack 02 *is* the DN-141 distillation; one source of truth. | Rewrite carries the S1/S2 spike stubs as recorded appendices. |

### 1.3 Pack 03 — Machinery, diagnostics & UX

| Q | Decision | Rationale / grounds | Implementation notes |
|---|---|---|---|
| **P3-Q1** | The first-fault bus **is the RFC-0013 diagnostic record, extended** — Localize-1 schema as an amendment to `Diag`; RFC-0005 EXPLAIN, CLI, and LSP are **consumers** of the one record. Merges ranked options 1+2; **no third system**. | `Diag` already has content hashing + dual human/JSON projection (RFC-0013 I3) and graded levels mapping 1:1 onto lean/normal/audit; missing exactly the envelope fields. GHC/rustc precedent for growing structured diagnostics in place [C21]; LSP mapping [C26]; OTel-later mapping stays clean [C24][C25]. | Fields to add to `Diag` (RFC-0013 amendment, §4.1): `event_id`, `phase`, `site_kind` (13-entry catalog, DESIGN-03 §3.3a), `decision`, `how` (registry code), `grades{in,out}`, `policy_ref`, `cert_mode`, `basis_ref`, `parent_event`/`child_cause`. Existing `minimal/medium/detailed` = the consumption tiers (no new tier system). |
| **P3-Q2** | **Yes, always-on generation in `fast`** — captured as a **clarification, not a new rule**: RFC-0034 §7 already normatively requires signal generation ≥ middle tier in every mode; first-fault records are that signal. | Head/tail-sampling precedent for gen≠consumption [C23]; JFR always-on viability [C29]. | Capture as an RFC-0034 §7 clarifying note (append-only footnote), not an amendment to semantics. |
| **P3-Q3** | Pause tooling that ships **now**: (a) always-print active `CertMode` on `check`/`run`; (b) three-axis labels (grade · mode · typing) wherever diagnostics appear. Lean first-fault one-liner ships as **X15's first deliverable** (needs `site_kind`). Transpile worklist + dual-report follow envelope stabilization (`transpile_gap` is an envelope instance). | (a)/(b) read existing `Meta` fields — zero schema commitment; diagnostics quality is an adoption driver [C22]. | — |
| **P3-Program** | **Interleave narrowly, then gate hard**: X15 + the two P0 surfaces move now; **ONESHOT residual waves stay HOLD until AX-core (X1–X5+X15) DoD**. | Transpile waves generate the Declared floods and gap-class soup; resuming before walls + localization recreates the dig the pause was called to end. | Wave order in §5. |

### 1.4 Pack 04 — Retention

| Q | Decision | Rationale / grounds | Implementation notes |
|---|---|---|---|
| **P4-Q1** | **Dual caps (count + bytes)**, per-surface, mode-scaled. Placeholder defaults (**`Declared`** until Phase-2 sizing): `fast` — first-fault ring 64 records / 256 KiB, cert-handle cap 0; `certified` — 1024 records / 8 MiB hot, cert-handle cap 256, 16 warm epochs. All overridable via `LanguageRetentionPolicy`. | Anchor pattern: JFR maxsize/memorysize + Prometheus time+size dual cap [C29][C30]. The defaults themselves ride the honesty lattice: `Declared` → `Empirical` after sizing. | Sizing pass: today's accumulation is **zero** (nothing retains — §3 finding G-8); size **prospective** per-record footprints statically from struct defs (`SwapCertificate`, extended `Diag`, warm digest) + a synthetic load run. |
| **P4-Q2** | **Bounded default + warning when unset.** Escalate to *required-explicit* only where a colony/spore **declares an audit obligation** — then absence of a policy is a check error. | JFR bounded-default posture; JDK-8 unlimited-default footgun [C29]. | — |
| **P4-Q3** | **Host FFI callback first** (C-ABI; matches C-backend bootstrap + KC-3 no-daemon). Hook signature designed so a later language-effect form wraps the same callback. At-least-once ack; drop counters EXPLAINed. | OTel bounded-queue drop accounting [C27][C28]; effects-lowering complexity deferred (◆pointer: Koka/OCaml). | — |
| **P4-Q4** | **Yes, lossy warm digests** — with **declared lossiness**: the digest states what was dropped and what error semantics survive; prefer declared-bound sketch shapes over unbounded-error ones. RP7 restated: a digest never upgrades a grade or fabricates a checked cert. | DDSketch relative-error guarantee vs t-digest unbounded worst case [C31][C32]. | EXPLAIN-of-drop record: `{retained=digest, export=hash?, dropped=N, loss_semantics}`. |
| **P4-Q5** | **No surface may claim "full in-process audit retained" under `certified`.** Certified = checked, exportable, **verifiable** evidence (content-hashed export digests + inclusion proofs). Append-all survives only as explicit opt-in audit mode that states its own bound. | Certificate Transparency model: verifiability from exported Merkle digests, not from pinning [C33]. | — |

---

## 2. Corrective W-1 — binary width canon: 8 → **64** (32 recognized fallback)

**Maintainer corrective (binding, 2026-07-17):** the de facto 8-bit binary canon is impractical for all but the most constrained embedded targets. Contemporary standard is **64-bit**, with **32-bit** the common fallback. **All widths remain first-class and supported.**

### 2.1 Verified state (repo audit, 2026-07-17)

- **No literal grammar/`Default` exists** — no `binary{8}` in `docs/spec/grammar`, no `impl Default for Repr`. The canon is **de facto**, established by example and export.
- **De facto sites (non-test):** `lib/std/swap.myc` exports only `{8,6}` and `{4,3}` pairs (`bin8_to_tern6` etc.); `matrix_len => Binary{8}` while `bytes_len => Binary{32}` (**inconsistent length canon**); `Repr::Binary { width: 8 }` literals in `mycelium-interp` (`prims.rs`, `lib.rs`), `mycelium-mlir` (`runtime.rs`, `deploy.rs`, `vr4.rs`), `mycelium-std-spore` (`deploy.rs`, `spore_ops.rs`), `mycelium-std-content`, `mycelium-std-select`, `mycelium-lsp` (`llm_canonical_parser.rs`, `fmt.rs`), `mycelium-core/src/node.rs`; plus `docs/lib-index/INDEX.md` (generated).
- **The transpiler already agrees with the corrective:** `emit.rs` maps `u64`/`usize` → width **64** and handles the full `i8..i128` family — the newest layer already treats 64 as canon.
- **Ternary ceiling (the one real constraint):** balanced-ternary core arithmetic is **not** width-capped, and `BigTernary` (E20-1/M-756, landed, Exact, never-overflows) removes any arithmetic ceiling. The `i64` cap lives **only in the conversion utilities** (`int_to_trits`/`trits_to_int`/`max_magnitude` → `None` at m ≥ 41). `binary{64}` full-range needs **41 trits** (3⁴⁰ < 2⁶⁴ ≤ 3⁴¹); `binary{32}` needs **21** (3²⁰ < 2³² ≤ 3²¹), which fits `i64` today.

### 2.2 Normative corrective

1. **Canonical width = `Binary{64}`** wherever a width must be assumed, exemplified, or exported as the primary form; **`Binary{32}`** recognized common fallback. `{8,6}`/`{4,3}` demoted to **embedded profiles + test vectors** (retained, not canon).
2. **Canonical bijection pairs:** `Binary{32} ↔ Ternary{21}` (**available now**) and `Binary{64} ↔ Ternary{41}` (**enablement item E-W1 below**). Both `LosslessWithinRange` per RFC-0002 §4; out-of-range stays explicit `Option`/error; once-per-swap-kind lemma + content-hash-cacheable certs unchanged; fixed-width round-trip remains SMT-dischargeable (RFC-0002 §4).
3. **Length/count canon:** length- and count-typed returns standardize on **`Binary{64}`** (`usize` parity with the transpiler mapping). Fix `matrix_len` (`Binary{8}` → `Binary{64}`) and align `bytes_len`.
4. **E-W1 (enablement):** lift the conversion utilities' `i64` ceiling — route `int_to_trits`/`trits_to_int`/`max_magnitude` through `i128` or the `BigTernary` path for m > 40. **M-758 (`PackedTernary` perf path) remains YAGNI/benchmark-gated** — this corrective does not activate it.
5. **Sweep list:** the §2.1 site list + add `bin32_to_tern21`/`tern21_to_bin32` std exports now and `bin64_to_tern41`/`tern41_to_bin64` behind E-W1 + regenerate `docs/lib-index` + docs/companion examples updated to 64/32 canon.
6. **Capture:** amend `docs/spec/swaps/binary-ternary.md` + `docs/spec/stdlib/swap.md` (append-only) with the canon + pairs; note on RFC-0033 (value model) if width canon is stated there.

---

## 3. Phase 0 — Grok-era audit & cleanup (corrective guidance for the agent)

**Context.** Implement waves during maintainer downtime were executed by `grok-composer-2.5-fast` (L0: `grok-4.5`) in the maintainer's words "in forgetful haste." **Audit before building.** Findings ledger first; fixes as **forward commits** (H3); ambiguities escalate via the `EXPRESS-ORACLE-BLOCKERS` file pattern rather than guessing.

**Output:** `docs/planning/audit-grok-2026-07/AUDIT-LEDGER.md` — one row per finding: `{id, severity, site, gate violated, evidence, fix commit | escalation}`. **Stop point: maintainer reviews the ledger before Phase 1.**

| ID | Check | How to detect | Disposition |
|---|---|---|---|
| **G-1** | No auto-inserted `swap` (S1/N3) | Audit every `Node::Swap` construction site outside written-swap lowering; RFC-0012 elaboration goldens (sugar ≡ longhand hash) still pass | Any violation = P0 fix |
| **G-2** | No grade upgrade without basis (VR-5/N2) | Grep `GuaranteeStrength` writes/strengthens outside `Swap`-cert endorsement; annotation paths weaken-only | P0 fix |
| **G-3** | `NotValidated` never treated as success | Audit all `CheckVerdict` matches; `GatedSwap::validated()` semantics preserved (absence-of-check ≠ pass) | P0 fix |
| **G-4** | No silent fallback in resolution paths | Policy/mode/import resolution failure paths hard-error (never a default substituted) | P0 fix |
| **G-5** | Status discipline (H1/H2) | DN/RFC/ADR `Status` fields vs ratification dates; `Doc-Index.md` + `CHANGELOG.md` consistency; DN free-ID collisions in `docs/notes/` | Fix status text; never backdate |
| **G-6** | Steer vocabulary | Grep `policy: default`, `policy: auto`, `policy: _` in code/docs/tests — must be **zero** post-capture (ambient or explicit name only) | Sweep |
| **G-7** | Width canon (W-1) | Grep **new** `Binary { width: 8 }` / `Binary{8}` introductions beyond the §2.1 sweep list; length-typed returns not `Binary{64}` | Fold into W-1 sweep |
| **G-8** | No unbounded accumulators before caps exist | Grep `Vec<SwapCertificate>`, `Vec<Diag>`, `Vec<Explanation>`, pushes on cert/diag/explain paths. Baseline 2026-07-17: **none exist** (certs discarded at trait boundary; explanations transient; no rings anywhere). Keep it that way until §1.4 caps land with the store | Any new accumulator without a cap = P0 |
| **G-9** | No third diagnostic system | Any new error/diag schema outside `mycelium-diag::Diag`/RFC-0013 (e.g. parallel human+JSON writers) | Migrate to `Diag` |
| **G-10** | Mechanical green | `cargo build`/`test`/`clippy -D warnings`/`fmt --check`; `justfile` targets; mutants config sane; CI workflows (`checks`/`fuzz`/`release`/`publish-docs`) pass; markdownlint/codespell/gitleaks; relative-link check on docs | Fix or ledger |
| **G-11** | Do-not-lift respected (H4) | No code lifted from listed upstreams (`embeddonator` etc.) | P0 escalation |

---

## 4. Phase 1 — Ratifiable capture (P0; after ledger review)

Each item: append-only, changelog row, `Doc-Index.md` updated, free-ID check where a number is minted. **Stop point: maintainer ratifies before Phase 2 implement waves** (X15 emitter *scaffolding* may proceed in parallel branches but does not merge to `dev` pre-ratification).

1. **Swap Ergonomics DN** (new; mint after free-ID check). Contents: P1-Q1 ambient spelling law (resolution precedence, hard-error rule, EXPLAIN origin, hash-conformance CI, rejected vocabulary); P1-Q2 handle+sink architecture + explicit-`Swapped`-until-X15; P1-Q3 elision gates; regime **layer reconcile** (kernel `Option` per RFC-0002 §4 / std `Result` per DN-16); A1 legal-pair matrix = checker materialization of RFC-0002 §5; W-1 width-canon cross-reference.
2. **DN-141 rewrite** from DESIGN-02 + §1.2 steers; S1/S2 spike stubs as appendices (`Declared`, with trigger criteria).
3. **RFC-0013 amendment** — envelope fields on `Diag` (§1.3 P3-Q1 list), site-kind catalog (DESIGN-03 §3.3a) as the `Code`-adjacent registry, first-fault linking rules (first-fault wins; symptoms cite cause), tiers = existing `minimal/medium/detailed`.
4. **RFC-0034 §7 clarifying footnote** — first-fault records are inspectability-signal generation (P3-Q2); append-only.
5. **`LanguageRetentionPolicy` spec** — fields per DESIGN-04 §5.3; resolution rides RFC-0012 scoping (fourth instance); §1.4 P4-Q1 placeholder defaults recorded **`Declared`**; EXPLAIN-of-drop/compact record shape (P4-Q4); export-hook FFI signature (P4-Q3); the P4-Q5 no-full-retention rule.
6. **W-1 capture** — §2.2 amendments to `binary-ternary.md` / `swap.md` (+ RFC-0033 note if applicable); E-W1 work item minted.

---

## 5. Phase 2 — Implementation waves (post-ratification)

| Wave | Work | Notes |
|---|---|---|
| **W-A** (first) | **X15 bus, first emitters:** extend `Diag` per RFC-0013 amendment; `swap_check` emitter at `ModeGatedSwapEngine`'s `SwapEngine` impl (NotValidated branch = refuse event; validated path = crumb + **cert handle**); `policy_resolve` emitter in selection/resolve; `Meta.cert: Option<ContentHash>`; mode-gated content-addressed cert store with §1.4 caps; **CertMode print** on `check`/`run`; **three-axis labels**. | Closes the P1-Q2 gap. Lean first-fault one-liner (`where · site_kind · decision`) is the W-A exit criterion. |
| **W-B** | **X1 policy streamline:** `policy: ambient` surface + catalog resolve-and-record + legal-pair matrix in checker (RFC-0002 §5 rows); elaboration-hash goldens. | |
| **W-C** | **X2–X5:** structural grade catalog + CI overclaim guard; regime→result enforcement (`regime_type_lie`); meet-boundary table; isolation EXPLAIN as envelope instances. | AX-core DoD = X1–X5 + X15 + probes in DESIGN-03 §7 pass. |
| **W-D** | **Spikes S1 + S2** (timeboxed; design-note appendices only; no production code). **Sizing pass** for §1.4 defaults (static struct footprints + synthetic load) → upgrade defaults `Declared`→`Empirical`. **E-W1** conversion-ceiling lift + W-1 sweep. | May run parallel to W-B/W-C. |
| **W-E** | AX-iso (X6 `std.airlock` predicate-basis remint + laundry CI; X7 certified mode×grade firewall + P2-Q3 colony admission). | |
| **Hold** | ONESHOT residual waves — **HOLD until AX-core DoD** (P3-Program). AX-sugar (X8/X9/X10) after gates; X8 gated on failure-materialization via the bus. | |

---

## 6. Phase 3 — Repository decomposition (maintainer-directed; **final stage before transpile work**)

**Gate:** Phase 0 ledger resolved · Phase 1 ratified · **AX-core DoD** (envelope/schemas stabilized — do not split moving schemas across repo boundaries) · CI green.

### 6.1 Target topology

| Repo | Seed / contents |
|---|---|
| **Component repos** | Seeded from `PARTITION.md` scope groups: `kernel` (7 crates) · `runtime` (5) · `frontend` (1: `mycelium-l1`) · `aot` (2) · `stdlib` (27 — maintainer may sub-split; default: one repo) · `toolchain` (12) · `transpile` (1) · `bench` (1). Each carries **its relevant docs** (spec slices, DNs/RFC excerpts it owns) + a **`CROSS-REF.md`**. |
| **`CROSS-REF.md`** (per component) | **Mycelium-internal dependencies only** (external Rust crates stay in Cargo metadata, not cross-ref docs): rows of `{component, repo URL, pinned rev + content hash, interfaces consumed, owning docs}`. |
| **`mycelium-docs`** | Full docs corpus + project-management records + archives: `docs/` (canonical whole-corpus home), `CHANGELOG.md`, `docs/planning/*` incl. gap-analysis archives, `docs/measurements`, `research/`, `experiments/` (archival), audit ledgers. Component repos carry *copies/slices* of the docs they own; this repo is the **complete record**. |
| **`mycelium`** (front) | Re-export/orchestration: workspace umbrella + meta-crate re-exporting components; **version train**; component pins (rev + hash); top-level README/branding; `packages/`; `examples/`; release packaging "as typical." |
| **Asset routing** | `lib/*.myc` → stdlib repo · `editors/` → toolchain · `fuzz/` targets → owning components · `.devcontainer`/CI templates replicated per repo from a shared template. |

### 6.2 Method & invariants

1. **History preserved:** `git filter-repo` per component from the monorepo (H3 — no history rewriting *within* a lineage; the split creates new lineages carrying their true history).
2. **Deps:** workspace `path` deps → `git` deps pinned by rev; **lockstep version train v0** (single version number across all components initially — cut the train from the front repo).
3. **Identity invariant (ADR-003):** decomposition must perturb **no** content hashes / spore identities. Build a **golden hash suite** (representative definitions + spores) before the split; assert identical after. Any delta = P0 stop.
4. **CI:** replicate `checks`/`fuzz`/`release` per repo; front repo runs an **umbrella integration build** of all pins.
5. **Cross-cutting-change protocol:** umbrella-PR pattern — one front-repo PR bumping N pins with linked component PRs; no long-lived divergent pins.
6. **Known cost, mitigated:** multi-repo raises coordination cost during cross-cutting change; that is exactly why the AX-core gate precedes the split and the version train keeps components in lockstep until post-transpile stabilization.

---

## 7. Phase 4 — Transpile readiness on components

- **One component at a time, leaf-first by dependency** (stdlib leaves → kernel-adjacent → runtime → toolchain; `PARTITION.md` order refined by the actual dep graph).
- **Per-component DoD:** dual-report vet as default; `transpile_gap` worklist with `// src:` breadcrumbs + closest-to-clean ordering; zero P0 gap classes; three-axis labels on every report; no tag fabrication (T6 — draft-phylum quarantine for Declared floods).
- **ONESHOT one-shot claim remains HOLD** until per-component DoD + release gate (unchanged from gap-analysis README).

---

## 8. Definition of done (this handoff)

- [x] All 17 steering-board answers captured as binding steers (§1) with grounds
- [x] Two maintainer-added spikes (S1, S2) recorded with outputs + trigger criteria
- [x] Width corrective W-1 verified against code and captured normatively (§2), incl. E-W1 enablement
- [x] Grok-era audit checklist (Phase 0) with detection methods + stop point
- [x] Ratifiable-capture worklist (Phase 1) with vehicles per steer
- [x] Wave order (Phase 2) honoring interleave-then-gate; ONESHOT HOLD restated
- [x] Decomposition program (Phase 3) per maintainer spec: component repos + per-repo docs + `CROSS-REF.md` (Mycelium-internal only) + `mycelium-docs` + front re-export repo; identity invariant + version train
- [x] Transpile ladder (Phase 4) leaf-first on components
- [ ] Maintainer ratifies Phase-1 captures (gate)
- [ ] Agent executes with stop points honored

## 9. Steering log (verbatim record, 2026-07-17)

P1: Q1 `ambient` · Q2 concur (close gaps ASAP) · Q3 concur (mechanical safety) · Q4 concur (dedicated DN).
P2: Q1 concur · Q2 concur **+ spike** (implementation prep; probable future need) · Q3 concur · Q4 concur **+ spike** (phylum-wide; probable future need) · Q5 concur.
P3: Q1–Q3 + program: concur.
P4: Q1–Q5: concur.
Corrective: binary default width 8 impractical beyond constrained embedded; canon = **64**, common fallback **32**; all widths supported.
Program: audit + clean up Grok-era work → **decomposition into component repos before transpile work** (docs travel with components; cross-ref docs Mycelium-internal only; dedicated full-docs/PM repo; front-facing re-export repo) → per-component transpile readiness.

## Changelog

| When | Note |
|---|---|
| 2026-07-17 | Handoff minted from maintainer deliberation over DESIGN-01..04; steers §1, corrective §2, program §3–§7. Companion citations + kickoff prompt issued alongside. |
