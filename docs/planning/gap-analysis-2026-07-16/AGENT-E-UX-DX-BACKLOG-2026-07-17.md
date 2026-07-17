# Design Agent E — UX/DX backlog (beyond pure swaps · tags · isolation)

| Field | Value |
|-------|--------|
| **Status** | **Draft** (council research — **not** Accepted; does not ratify) |
| **Agent** | E — Broader UX/DX backlog (author / operator / porter daily experience) |
| **Model** | grok-4.5 (high effort research) |
| **Honesty** | Claims are **`Declared`** (corpus-grounded design judgment) unless tagged `Empirical` with a cite |
| **Scope** | Mycelium only; **read-only** — no product code; no DN/ADR status moves |
| **Council** | [DESIGN-COUNCIL-SWAPS-TAGS-2026-07-17.md](./DESIGN-COUNCIL-SWAPS-TAGS-2026-07-17.md) |
| **Complements** | [AGENT-A](./AGENT-A-SWAPS-ERGONOMICS-2026-07-17.md) · [DN-141](../../notes/DN-141-Tagging-Meta-Honesty-Lattice-UX.md) (B) · [AGENT-D](./AGENT-D-HONESTY-POISON-CONTAINMENT-2026-07-17.md) · [AGENT-C](./AGENT-C-AX-STACK-SYNTHESIS-2026-07-17.md) · [delta-DX-qol](../zero-hand-port/delta-DX-qol.md) |

> **Posture (VR-5 / G2 / house rule #3).** This inventories and ranks. It does **not** move any
> RFC/DN/ADR status. Non-negotiables still bind: never-silent (G2), no guarantee upgrade without
> basis (VR-5), prefer **deterministic machinery** over folklore, KISS/YAGNI (rule #5). **Explicit
> non-goals of this note:** re-designing swaps, lattice tags, or honesty-poison isolation machinery —
> those belong to A/B/D/C; this note only records **interactions** with them (§4) and presentation
> that makes containment usable.

---

## 0. Mandate restated + method

**Question:** Beyond swaps + lattice tags + airlocks, what other UX/DX improvements are worth
investigating so daily authoring, operating, porting, and agent-assisted development stay smooth
without violating house rules?

**Method (verify-first):**

1. Read peer council artifacts (A/B/C), ONESHOT transpile pain (`PROGRAM-HANDOFF-ONESHOT.md`,
   M-1006 baselines), and the already-filed DX QoL track (`delta-DX-qol.md` / M-1041..M-1047).
2. Scan diagnostics (DN-04/RFC-0013/DN-22), modes (DN-29/RFC-0034/DN-126), sugar (`reveal` /
   DN-110/M-1051), CLI (`mycelium-cli`), fallibility (DN-102/DN-135), lexicon (DN-02),
   progressive disclosure (tutorial + research T7), agent tooling (tero / indices).
3. Rank by **author-day tax × leverage × house-rule safety**, not by novelty.

**What is already covered (do not double-count as "new" design work):**

| Track | Owner | Status |
|-------|-------|--------|
| Swap ceremony / cert threading / policy catalog | Agent A | Draft inventory |
| Lattice / Meta / mode conflation / airlock surface | DN-141 / Agent B | Draft DN |
| AX-stack synthesis (incl. containment) | Agent C | Draft synthesis |
| Honesty-poison containment (airlock/firewall/quarantine/meet-boundary) | Agent D | Draft inventory |
| Transpiler DX D1–D12 | `delta-DX-qol.md` | Filed M-1041..M-1047 |
| Expressibility residual (language gaps) | language-completeness inventory | Drive-hard worklist — *correctness*, not pure UX |

This inventory **adds** the residual author/operator surface that is neither pure language-gap nor
already owned by A/B/C/DX-QoL — and **re-surfaces** a few DX-QoL rows only where they are daily
author pain, not just contributor-transpiler pain.

---

## 1. Pain inventory (concrete, cited)

Each row is a real friction surface. Severity is design judgment (`Declared`) ordered by daily tax
for **authors / operators / porters / agents**, not only for language implementers.

### 1.1 Diagnostics, errors, LSP EXPLAIN

| # | Pain | Who pays | Basis |
|---|------|----------|-------|
| **E1** | **LSP missing edit-loop providers.** Hover/definition/diagnostics/semantic-tokens/fmt/completions exist, but **no references, rename, code_action, signature_help**; completions are lexical/scaffolding only (no scope/type awareness). Hand-editing `.myc` is under-tooled. | hand-porters, lib authors | `Empirical` — `delta-DX-qol.md` D9; E2-5 "Full LSP" epic undecomposed; `issues.yaml` still asks which providers are 1.0-required |
| **E2** | **`reveal` / expand-on-demand not shipped.** Sugar and `lower`/`derive` desugars are load-bearing for zero-hand-port (DN-110), but the mandatory transparency spine (`reveal`, M-1051) is ratified-not-built. Authors cannot "see what the sugar became" the way `cargo expand` / Racket Macro Stepper do. | everyone who uses sugar or reads emitted drafts | `Empirical` — DN-110 §1.1; M-1051 `todo`; `reveal` not-yet-lexed (M-664 residual) |
| **E3** | **Diagnostic graded presentation is designed, unevenly productized.** RFC-0013 / DN-04: diagnostics additive over explicit errors; dual human/JSON; graded verbosity. DN-22 compact codes (byte + base-36) are Draft. CLI `Report` is structured (`error[code]: …` + `help:`) — good — but there is no single author-facing **EXPLAIN panel** story tying grade · mode · swap cert · sugar expansion · gap reason. | authors, AI co-author loop (M-330) | `Declared` productization gap; `Exact` governing rule (DN-04) |
| **E4** | **Transpile gap reasons are descriptive, not always actionable.** `GapReason` says what/why; rarely the exact hand-idiom. Closest-to-clean ranking missing. | porters | `Empirical` — D6/D8 → M-1045/M-1046 |

### 1.2 Modes & progressive disclosure

| # | Pain | Who pays | Basis |
|---|------|----------|-------|
| **E5** | **Three trust axes look like one dial (presentation).** Grade · cert depth · typing strictness are orthogonal (DN-126 §2; DN-141 P3). Authors still conflate "fast = no tags = loose types." Companion three-axes framing is partial; tooling does not separate the dials in UI. | everyone | `Declared` framing pain + Accepted DN-126 three-axis verdict |
| **E6** | **`fast` / `certified` discoverability is mode-machinery-first, tutorial-second.** RFC-0034/ADR-032 enacted; default is `fast`. Tutorial §5 shows swap with `policy: rt` but does not teach *when* to engage certification or how mode tags appear in diagnostics. | newcomers | `Empirical` — `docs/reference/tutorial.md` structure (tero-index); DN-29 reframe |
| **E7** | **Loose/strict typing (DN-126 / M-1077) is Accepted design, unbuilt.** The progressive-disclosure *engine* for "run early, strictify later" does not exist yet — so progressive disclosure is docs-only today. | Python-port path, explorers | `Declared` until M-1077 |

### 1.3 CLI / project / check surface

| # | Pain | Who pays | Basis |
|---|------|----------|-------|
| **E8** | **`myc` is a thin front door.** Real: `init` / `build` / `check` / `test`(=check) / `run` / `--stream`. Honest note that unit-test runner is deferred. No first-class `myc fmt`, `myc lint`, `myc explain`, `myc transpile`, `myc vet`, `myc reveal`, mode flags. Operators learn N binaries + skills instead of one CLI. | operators, CI authors | `Empirical` — `crates/mycelium-cli/src/bin/myc.rs:1–40`; lib.rs notes on `test` |
| **E9** | **`myc test` is type-check with a disclaimer.** Correct under VR-5 (never pretends a runner exists), but the *name* sets wrong expectations vs Rust/Python. Dogfood loop still lives in `just myc-dogfood` / skill surface. | operators | `Empirical` — `myc.rs` cmd_test note |
| **E10** | **Project surface (`mycelium-proj.toml`) is real but onboarding-light.** M-359/M-368 spore pipeline exists; multi-nodule `run` is M-908/M-909. Phylum/nodule/header mental model is dense for a first program (tutorial starts at nodule-file, not project). | newcomers | `Declared` onboarding gap; `Empirical` manifest machinery exists (`mycelium-proj`) |
| **E11** | **Phylum-mode vs oracle-mode vet is a measurement footgun.** Correct emission fails oracle `myc check` for cross-nodule `use` (DN-124). Authors/porters misread "fails check" as "bad emit." Dual-report / re-baseline not fully productized in daily CLI. | porters, metrics consumers | `Empirical` — DN-124; language-completeness §2 |

### 1.4 Fallibility & sugar

| # | Pain | Who pays | Basis |
|---|------|----------|-------|
| **E12** | **`?` is let-RHS only (v0).** General-position CPS lift deferred (DN-102). Porters still hand-nest `match` for non-let chains. | porters, stdlib authors | `Empirical` — DN-102 position restriction; DN-99 #60 residual |
| **E13** | **Result/Option combinator-over-closure residual (DN-135 Accepted, M-1092 unbuilt).** Constant-function maps (`map(\|()\| …)`, `map_err(\|_\| …)`) still gap — pervasive "Other" class. | porters | `Declared` design; unbuilt |
| **E14** | **Ambient paradigm / sugar expand-all is partial.** `default` ambient paradigm is active (sugar-index); expression-position sugar rules (DN-110 J3) and `reveal` are not. Authors cannot expand-all sugars for teaching or review. | authors, reviewers | `Empirical` sugar-index + DN-110 gap J3 |

### 1.5 Testing, dogfood, provenance

| # | Pain | Who pays | Basis |
|---|------|----------|-------|
| **E15** | **Native `.myc` test runner is absent.** Differential/conformance lives in Rust crates; dogfood is `myc check` + skill. No `#[test]`-class story for Mycelium authors. | lib authors post-stdlib | `Empirical` — CLI honesty note; `mycelium-std-testing` is harness *types*, not a runner UX |
| **E16** | **ONESHOT / M-1006 loop is powerful but operator-heavy.** `checked_fraction` vs `expressible_fraction` split is correct (VR-5). Missing: ranked worklist in default report (M-1046), tree-mirror emit (M-1042), breadcrumbs (M-1043), suggested idiom (M-1045), dry-run (M-1047). Current pilot unions ~19.5% checked — residual ranking is manual docs. | porters, L0 | `Empirical` — PROGRAM-HANDOFF-ONESHOT; delta-DX-qol |
| **E17** | **Dataset / trial lineage for Empirical claims has no author path.** Meta can carry bound+basis (M-I1…M-I4); surface is thin (DN-141 P4). "I claim Empirical with these trials" is not an ergonomic authoring surface (also B's domain — noted as **shared**). | certified authors | DN-141 P4/P5 |

### 1.6 Agent / developer tooling & lexicon

| # | Pain | Who pays | Basis |
|---|------|----------|-------|
| **E18** | **Tero / indices are strong for agents, weak as *author* UX.** `/tero-query`, `docs/tero-index/INDEX.md`, `docs/api-index/`, sugar-index — excellent for agents. No "explain this diagnostic code / this gap class / this sugar" one-hop for humans in the editor. | human authors | `Declared` presentation gap; tero itself `Empirical` Layer-1 |
| **E19** | **Fungal lexicon cognitive load is real but bounded.** DN-02 naming law is sound (theme where illuminating). Residual: phylum/nodule/spore/hypha/colony/swap + reserved runtime tier (`fuse`/`mesh`/…) + guarantee lattice + cert modes = many simultaneous new words for a first week. Teaching diagnostics for reserved-not-active words help (Surface-Stability-Declaration). | newcomers | `Declared` load; `Exact` naming law |
| **E20** | **Progressive disclosure is tutorial-thin vs corpus-deep.** Tutorial is a complete small program (nodule → match → for → swap → honesty). Certified core, modes, Meta, spore publish, VSA, and agent workflows live in a large RFC/DN web. No laddered "tutorial → companion → certified" product path. | newcomers → power users | `Empirical` tutorial exists; research T7 progressive-disclosure law cited but not productized |
| **E21** | **Cross-language port author UX is gap-profiler-first.** Transpiler is honest (GapReason, never fabricate). Author still lacks: path-mirror out-dir, per-item `// src:`, suggested idiom, closest-to-clean ranking, optional `mycfmt` post-pass — all filed, mostly unbuilt. | porters | `Empirical` — D1–D12 / M-1041..1047 |

---

## 2. Ranked backlog (P0–P3)

**Objective function (explicit):**

\[
\text{score} = \text{daily tax} \times \text{breadth of users} \times \text{leverage on ONESHOT/dogfood}
\times \text{house-rule safety} \;/\; \text{KC-3 risk}
\]

Prefer **deterministic machinery + tooling presentation** over new language forms when both close
the same pain (aligns with Agent C / DN-141).

### P0 — ship or unblock soon (high tax, high leverage, low house-rule risk)

| ID | Improvement | Kind | Rationale | Corpus cite | Tracked? |
|----|-------------|------|-----------|-------------|----------|
| **P0.1** | **Transpile author worklist: breadcrumbs + closest-to-clean + suggested_idiom** | tooling (deterministic report) | Biggest single accelerator for hand-port / ONESHOT residual triage without language change | D3b/D6/D8; M-1043/1045/1046 | **yes** |
| **P0.2** | **Phylum-mode dual-report as default vet story** | tooling / measurement | Stops "correct emit fails check" false narrative; force-multiplier for Import class | DN-124; language-completeness §2 row 7 | **yes** (DN-124 units) |
| **P0.3** | **Tree-mirror emit + no stem overwrite** | tooling | Data-loss + navigation pain for multi-file ports | D2/D3; M-1042 | **yes** |
| **P0.4** | **Three-axis presentation kit (docs + CLI/LSP labels)** | docs + tooling presentation | Unblocks E5 without new semantics; required so A/B sugar is not misread | DN-126 §2; DN-141 P3/F; companion 04 gap | **partial** (DN-126 Accepted; presentation unbuilt) |

### P1 — high value; small design or medium build

| ID | Improvement | Kind | Rationale | Corpus cite | Tracked? |
|----|-------------|------|-----------|-------------|----------|
| **P1.1** | **`reveal` / expand-on-demand (M-1051) as first-class DX** | language surface + tooling | House rule #2 load-bearing for sugar; teaches instead of hides; unblocks DN-110 J3 credibility | DN-110; DN-38; M-1051 | **yes** `todo` |
| **P1.2** | **LSP code_action + references + rename (decompose E2-5)** | tooling | Hand-edit tax; code_action is the natural home for "insert swap candidate" (A X8) without auto-insert | D9; E2-5 | **epic only** — **needs decomposition** |
| **P1.3** | **`myc` facade: wire existing tools under one CLI** (`fmt`, `lint`, `check --phylum`, `explain <code>`, optional `transpile`/`vet`) | tooling | Operators currently juggle binaries; keep honesty (test ≠ runner until real) | `myc.rs` surface; DN-22 EXPLAIN | **partial** — CLI exists; subcommands missing |
| **P1.4** | **DN-135 match-inline build (M-1092)** | machinery (transpiler) | Closes pervasive Result/Option constant-map gaps; pure Idiomatic Remapping | DN-135; M-1092 | **yes** design Accepted |
| **P1.5** | **Actionable diagnostic "fix path" field** (generalize M-1045 shape to `myc check` / RFC-0013) | tooling | DynEL/RFC-0013 presentation; AI co-author loop (M-330) wants structured next step | DN-04; DN-22; NFR-2 | **partial** |
| **P1.6** | **Mode discoverability: `myc check --mode` (`fast`/`certified`) + always print active mode tag** | tooling + presentation | Makes RFC-0034 legible in the daily loop; never silent mode (G2) | RFC-0034; DN-29 | **mode machinery exists; CLI flag UX incomplete** |

### P2 — important for 0.x usability / Phase I function-first

| ID | Improvement | Kind | Rationale | Corpus cite | Tracked? |
|----|-------------|------|-----------|-------------|----------|
| **P2.1** | **Native `.myc` test runner (or honest rename of `myc test`)** | tooling + std | Either ship minimal runner or rename to `myc verify` until real — current name is a soft lie of expectation (not of implementation — VR-5 note is present) | CLI note; ADR-038 usability | **open** |
| **P2.2** | **Progressive disclosure ladder (tutorial → companion → certified cookbook)** | docs | One path from hello-nodule to cert modes / Meta / spore without drowning in RFCs | tutorial; research T7; DN-141 P10 dual-write | **open product** |
| **P2.3** | **Lexicon onboarding card** (one page: themed vs conventional + "reserved-not-active" list) | docs | Cuts E19 without renaming anything (DN-02 law stands) | DN-02; Surface-Stability-Declaration | **partial** (lexicon exists, not progressive) |
| **P2.4** | **`?` general-position CPS lift** | language | After v0; reduces fallibility ceremony | DN-102 residual | **yes** deferred |
| **P2.5** | **M-1077 loose/strict typing implement** | language/checker | Progressive run-early path; Python port enabler | DN-126 Accepted | **yes** M-1077 |
| **P2.6** | **Visitor-DRY / emit fold (M-1041)** | contributor tooling | Force-multiplier on every M-1006 phase — not end-user, but slows *all* UX-facing emit fixes | D1 | **yes** |
| **P2.7** | **EXPLAIN panel unification (LSP + CLI)** — one schema: grade why · mode · sugar expand · swap cert · gap | tooling | Productizes E3; generation≠consumption for tags (DN-141 F) | RFC-0013; RFC-0034 §7; DN-141 | **design partial** |

### P3 — polish / later waves

| ID | Improvement | Kind | Rationale | Corpus cite | Tracked? |
|----|-------------|------|-----------|-------------|----------|
| **P3.1** | **DN-22 compact diagnostic codes productized** | tooling | Token-efficient AI channel; Draft only today | DN-22 | Draft |
| **P3.2** | **Transpile dry-run / --help / optional mycfmt** | tooling | M-1047 polish | D4/D5/D12 | **yes** |
| **P3.3** | **Tero-in-editor / "cite this hover"** | agent tooling | Nice for humans; agents already have MCP | DN-87 | open |
| **P3.4** | **Dataset lineage author sugar** (basis-carrying Empirical — shared with DN-141 D) | language + Meta | Certified path; not daily `fast` | DN-141 D/P4 | design Draft |
| **P3.5** | **Signature help + type-aware completions** | tooling | After references/rename | E2-5 remainder | epic |
| **P3.6** | **Expression-position sugar rules (DN-110 J3) + expand-all** | language | After `reveal` spine | DN-110 | Accepted design, unbuilt |
| **P3.7** | **Project wizard improvements** (`myc init` multi-nodule templates, sample spore publish) | tooling | Phase I usability | M-359/M-368 | partial |

---

## 3. Deterministic machinery vs docs vs tooling

| Class | What belongs here | Backlog IDs |
|-------|-------------------|-------------|
| **Deterministic machinery** (rules/tables/desugars; EXPLAIN-able; no folklore) | Phylum-mode credit rules; DN-135 match-inline; `?` CPS; M-1077 demotion switch; structural grade catalog *(owned by B)*; legal-pair matrix *(owned by A)*; sugar lowering + `reveal` | P0.2, P1.1, P1.4, P2.4, P2.5, P3.6; *plus A/B X1–X4* |
| **Tooling** (CLI/LSP/vet/reports; presentation of truth, not new truth) | Worklist reports; tree-mirror emit; `myc` facade; LSP providers; EXPLAIN panels; mode flags; dual-report; compact codes | P0.1, P0.3, P0.4, P1.2, P1.3, P1.5, P1.6, P2.1, P2.7, P3.1–P3.3, P3.5, P3.7 |
| **Docs / progressive disclosure** | Three-axis kit; tutorial ladder; lexicon card; dual-write cleanup | P0.4, P2.2, P2.3, parts of P1.6 |

**Rule of thumb for this track:** if a pain is "authors don't know which dial they turned," prefer
**tooling presentation + docs** before new syntax. If a pain is "authors must rewrite the same
pattern by hand 100×," prefer **deterministic desugar/machinery** (with `reveal`).

---

## 4. Interactions with swap / tag / poison-isolation designs (synergies · conflicts)

### Synergies (build UX *with* A/B/C/D, not around them)

| UX item | Synergy with A/B/C/D |
|---------|---------------------|
| **P0.4 three-axis labels** | Required so authors do not misread cert ambient (A X5) or grade EXPLAIN (B F) as type-loose |
| **P1.1 `reveal`** | Same spine as sugar-over-swap elision and tag-EXPLAIN consumption tiers — one expand model |
| **P1.2 LSP code_action** | Natural carrier for **insert-swap candidate** (A X8) and **insert airlock scaffold** (B E) — candidates only, never auto |
| **P1.5/P2.7 EXPLAIN panel** | Host for grade-why (B), policy expansion (A), mode tag (RFC-0034) in one dual human/JSON view |
| **P1.6 mode CLI flag** | Surfaces cert depth without collapsing into grade (DN-141 reject ambient grade upgrade) |
| **P0.1 suggested_idiom** | Can point at airlock / explicit swap / `map_err` patterns from A/B catalogs once they exist |
| **P2.7 EXPLAIN panel** | Host for isolation EXPLAIN package (Agent D §5: `boundary_kind`, remint basis, meet_refuse) |
| **P1.2 code_action** | Insert **seal / quarantine export** scaffolds (D2/D1) as candidates, never auto |
| **P0.4 three-axis** | Prevents mode firewall (D3) from being misread as grade change |

### Conflicts / footguns to avoid

| Temptation | Why it conflicts | Safer alternative |
|------------|------------------|-------------------|
| Auto-insert swaps from LSP | Violates RFC-0012 I1 / A never-auto | code_action **candidate** + user accept |
| Auto-insert seals that remint without predicate | Laundry / greenwash (Agent D O2; companion 02) | seal scaffold requires pred / cert slot |
| Ambient nodule `@ Exact` to "simplify UX" | Rejected by DN-141 Alt C; Agent D reject ambient laundry | lint profile "public API must write `@ g`" |
| Hide `Declared` in `fast` UI | Lies about floor (VR-5); hides poison | show floor; collapse *detail* via verbosity levels (DN-04) |
| Global Declared UI to "be safe" | Quality kill (Agent D O3) | partition + seal, keep Exact cores |
| `myc test` that swallows type errors into "pass" | G2 | keep refuse; either real runner or rename |
| Collapse three axes into one "strictness" slider | DN-126 / DN-141 | three labeled dials always |

---

## 5. Quick wins vs hard multi-wave items

### Quick wins (days–1 PR class; mostly tooling/docs)

1. **P0.1 partial:** emit `// src: file:line` breadcrumbs (M-1043) + print first diagnostic per file in vet (M-1046 slice).
2. **P0.4 docs:** one companion/cheatsheet page — three axes table (copy DN-126 §2) linked from tutorial §8.
3. **P2.3:** one-page lexicon card (themed vs conventional + reserved-not-active).
4. **P1.6 partial:** print active `CertMode` on every `myc check` summary line (even if flag wiring waits).
5. **P3.2 slice:** transpile `--summary` / dry-run without write (M-1047).
6. **P1.3 slice:** `myc fmt` / `myc lint` as thin wrappers over existing binaries (no new semantics).

### Medium (1–3 waves; design Accepted or filed)

1. **P0.2** phylum-mode dual-report default (DN-124 units).
2. **P0.3** tree-mirror emit (M-1042).
3. **P1.4** DN-135 build (M-1092).
4. **P1.2 first cut:** LSP `code_action` + `references` only (defer rename polish).
5. **P1.5** `suggested_idiom` / help field on check diagnostics.

### Hard multi-wave (design + build + dogfood)

1. **P1.1 + P3.6** `reveal` + expression sugar facility (DN-110 full).
2. **P2.5** M-1077 loose/strict (checker posture + mechanical strictify + residual surfacing).
3. **P2.4** general-position `?` CPS.
4. **P2.1** real `.myc` test runner + differential-in-Mycelium story.
5. **P2.7** unified EXPLAIN panel (schema + LSP + CLI + tero cite).
6. **Full E2-5 LSP** (type-aware completions, signature help, rename hygiene).
7. Anything that **depends on A/B ratification** (swap cert ambient, airlock phylum, basis-carrying `@ Empirical`) — do not implement ahead of council steer.

---

## 6. ONESHOT / transpile pain as *symptoms* (mapping)

| Symptom (Empirical) | Underlying UX/DX class | Backlog |
|---------------------|------------------------|---------|
| Pilot union ~19.5% checked; residual ranked in markdown by hand | Worklist not in tool | P0.1 |
| Import fails under oracle despite good emit | Measurement UX | P0.2 |
| Flat emit / stem collision | Navigation / data loss | P0.3 |
| Derive / Bool / `!=` closed as one-off emit arms | Contributor DX tax (visitor-DRY) | P2.6 |
| Result combinator gaps (`map` unit/wild) | Fallibility ergonomics | P1.4 |
| Macro / format residual needs expand-first | Sugar transparency missing | P1.1, P3.6 |
| Agents re-grep corpus instead of one EXPLAIN | Human/agent diagnostic product | P2.7, P3.3 |
| "Is this fast or certified?" invisible in check output | Mode discoverability | P1.6, P0.4 |

**Important honesty:** raising `checked_fraction` is **mostly language/transpiler correctness**
(language-completeness inventory), not UX polish. UX work **accelerates triage and authoring**; it
does not invent native answers for `&mut self` / Display prim / records. Do not sell DX as a
substitute for expressibility levers (VR-5).

---

## 7. Recommended direction (ranked, not ratified)

### Rank 1 — **Tooling presentation layer over existing truth** (default investment)

Ship P0.1–P0.4 + P1.3 + P1.6 as a **UX wave that touches almost no language semantics**: ranked
vet worklists, phylum-mode dual report, tree-mirror emit, three-axis labels, `myc` facade wrappers,
mode tag always printed. This is pure G11 projection over reified errors/modes — DN-04 governing
constraint satisfied by construction.

### Rank 2 — **Transparency spine for sugar + fallibility**

`reveal` (P1.1) + DN-135 build (P1.4) + LSP code_action candidates (P1.2). Teaches mechanical
lowering; unblocks port idioms; hosts A/B candidate inserts without ambient auto-magic.

### Rank 3 — **Progressive author path**

Docs ladder (P2.2/P2.3) + real test story (P2.1) + M-1077 (P2.5) sequenced after Rank 1 so the
dials authors learn are the ones the tools already show.

### Explicitly deprioritize (for *this* UX track)

- New fungal synonyms for conventional terms (fails DN-02 T-learn).
- Collapsing trust axes into one slider.
- Auto-swap / auto-grade-upgrade "to reduce ceremony."
- Replacing language-gap work with prettier diagnostics alone.

---

## 8. Adversarial stress-test of Rank 1

| Attack | Response |
|--------|----------|
| "Pretty reports without higher checked_fraction is vanity." | Partially fair. Rank 1 does **not** claim metric wins; it claims **triage time** and **fewer false "emit is wrong" reads** (P0.2). Metric wins stay on expressibility waves. |
| "One mega-CLI violates KC-3." | Facade wrappers over existing crates are SoC-preserving if they add no trusted kernel deps; refuse if it pulls logging into L0. |
| "Printing mode tags is noise." | Verbosity levels (DN-04 minimal/medium/detailed) — default one line; detailed expands. Never omit the mode bit entirely (G2). |
| "suggested_idiom will go stale." | Derive from content-addressed catalogs (gap class → idiom) same as DN-22 codes-from-registry; tombstone, don't rewrite (append-only). |
| "This duplicates delta-DX-qol." | P0.1–P0.3 **are** that track — elevated here because they are the **author-facing** face of ONESHOT pain, not only contributor DRY. |

**Verdict:** Rank 1 survives; it must be **sold as acceleration + honesty of measurement**, never as
a substitute for language levers or for A/B's swap/tag machinery.

---

## 9. Open questions for maintainer

1. **CLI product shape:** thin multi-binary forever vs `myc` as the sole front door (fmt/lint/explain/transpile/vet subcommands)?
2. **`myc test` naming:** keep honest-disclaimer, rename to `myc verify`, or prioritize a real `.myc` runner for Phase I usability (ADR-038)?
3. **LSP 1.0 bar:** which providers are release-gating (references/rename/code_action) vs optional (signature_help, type-aware completion)? (Already asked in issues — needs a call.)
4. **Default vet mode:** switch headline metric to phylum-mode immediately (with Δ_basis) or dual-report for one full cycle (DN-124 M-A)?
5. **`reveal` sequencing vs ONESHOT:** implement reveal before expression sugar, or only after M-875 expand-first design Accept?
6. **Progressive disclosure owner:** companion docs only, or a `docs/learn/` ladder with runnable fixtures gated by `myc check`?
7. **How much of Rank 1 may proceed without A/B council ratification?** (Recommendation: all of Rank 1 except code_actions that insert swap/airlock scaffolds — those wait for A/B surface names.)
8. **Agent-E follow-on:** promote any P0/P1 cluster into a Draft DN (e.g. "Toolchain presentation contract") or keep as planning-only under gap-analysis?

---

## 10. Definition of Done (for this inventory artifact)

- [x] Pain inventory beyond swaps/tags/airlocks, corpus-cited.
- [x] Ranked P0–P3 backlog with rationale.
- [x] Machinery vs docs vs tooling classification.
- [x] Interactions with A/B/C (synergies + conflicts).
- [x] Quick wins vs multi-wave split.
- [x] Open questions for maintainer.
- [ ] Maintainer steers → optional promotion to Draft DN / wave slots (out of scope here).

---

## 11. FLAGs (integrating parent / L0)

| Target | FLAG |
|--------|------|
| `CHANGELOG.md` | Note only if this planning file is indexed as a durable council artifact |
| `docs/Doc-Index.md` | Optional row under planning/gap-analysis-2026-07-16 |
| `tools/github/issues.yaml` | **Do not mint** new M-ids here; prefer existing M-1041..1047, M-1051, M-1077, M-1092, E2-5 decomposition |
| Council synthesis | Feed Rank 1–3 into Agent C AX-stack and L0 re-rank after A/B/C/D report |
| CLAUDE.md | No change |

---

## 12. Changelog (this file)

| When | Note |
|------|------|
| 2026-07-17 | Draft backlog (Design Agent E) — ranked UX/DX beyond pure swaps/tags; grounded in ONESHOT, delta-DX-qol, DN-04/22/29/102/110/126/135/141, CLI surface, DN-02 lexicon; cross-linked to Agent D isolation |
| 2026-07-17 | Integrator: canonical filename `AGENT-E-UX-DX-BACKLOG`; poison/isolation synergies and footguns woven into §4 |
