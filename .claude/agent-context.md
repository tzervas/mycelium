# Mycelium — Agent Session Context

> **Read this first. It replaces re-reading CLAUDE.md from scratch.**
> Authoritative rules: `CLAUDE.md` (wins on any conflict), `CONTRIBUTING.md`.
> This file is a compact orientation brief; it is not normative.

---

## What this project is

Mycelium is a unified value-semantics substrate (binary/ternary/dense/VSA) with **certified,
never-silent** representation swaps and **honest, per-operation guarantees**. It is in active
**design + Rust-first implementation** phase. The corpus in `docs/` is the primary product;
code lands incrementally per the phase plan.

---

## Non-negotiable rules (abbreviated)

1. **Honesty lattice**: `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`. Tag every bound/guarantee
   per-model/per-op. `Proven` requires a theorem with *checked* side-conditions. Downgrade freely;
   never upgrade without a checked basis (VR-5).
2. **Never silent** (G2): swaps/conversions are never silent; out-of-range is `Option`/error.
3. **Append-only decisions**: `Draft → Accepted → Enacted → Superseded`. Never rewrite Accepted
   content — supersede with a new doc. Notes: `→ Resolved`.
4. **Ground every claim**: normative statements cite `G1–G11 / A–E / R1–R8 / T0.x–T2.x`.
5. **Squash-only to main, always via PR** — every change lands on `main` as a single curated
   squash commit through a GitHub PR.

---

## Key file pointers

| What | Where |
|------|-------|
| **Symbol index** (navigational aid) | `docs/api-index/INDEX.md` — grep-friendly; `index.json` for machine |
| **Doc index** | `docs/Doc-Index.md` — canonical map of all spec/RFC/ADR/DN status |
| **Task tracking** | `tools/github/issues.yaml` — M-xxx IDs + status + doc_refs |
| **ID map** | `tools/github/idmap.tsv` — M-xxx → GitHub issue number/db-id |
| **Changelog** | `CHANGELOG.md` (append-only, most recent at top) |
| **Phase plan** | `docs/planning/phase-5.md`; Phase-6 roadmap in **`DN-11 §5`**; active wave → `.claude/kickoff.md` |
| **RFC index** | `docs/rfcs/README.md` |
| **ADR index** | `docs/adr/README.md` |
| **DN index** | `docs/Doc-Index.md` §Design Notes |
| **Grammar** | `docs/spec/grammar/mycelium.ebnf` + `crates/mycelium-l1/src/` |
| **Stdlib specs** | `docs/spec/stdlib/*.md` |
| **API baseline** | `docs/spec/api/` |
| **Check** | `just check` (= CI; skip gracefully if tool absent) |

### `doc_refs:` grammar (in issues.yaml)

```
api:<crate>::<path>      symbol in docs/api-index/index.json
corpus:<DOC>[#<anchor>]  doc/section in docs/Doc-Index.md
src:<path>[:<line>]      source file (repo-relative)
```

Validate: `python3 tools/github/doc_refs_check.py`

---

## Current state (2026-06-21)

### Recently landed (most recent first)

- **1.0.0 kernel/core gate CLOSED** — **M-654 (#313)** Gate A3: cargo-mutants **0 un-triaged survivors**
  on the trusted base (`core`/`cert`/`interp`/`numerics`; equivalents justified in one workspace-root
  `.cargo/mutants.toml`), LCG suites → **proptest** (pinned `cases:1`), **cargo-fuzz** targets + smoke CI.
  With A1/A2/A4/A5/B1/B2 already met, **every ADR-021 row is green**.
- **Gap-closure epics filed (#312)** — **E7-1** / **E7-2** + 13 issues M-656…M-668; cross-referenced in
  DN-14 §3 / DN-11 §5 + the lexicon memory.
- **Editorial enactment sweep (#310)** — RFC-0016/0017/0021 → **Enacted**; DN-04/05/10/11/12 → Resolved;
  **M-649 deferred** (post-1.0) — now back in scope (E7-1 unblocks it; M-502 ✅).
- **Lexicon/syntax/grammar memory (#311)** — `.claude/memory/lang-lexicon-syntax.md` + a CLAUDE.md
  fungal-lexicon quick-ref (phylum/nodule/spore/hypha/colony — never "crate"/"module").
- Also: A4 non-skip gate (M-652 #303), A2 Medium-findings ledger (M-653 #306), scope→milestone +
  per-PR/issue override manifest (#304/#309), bench Grok ingestion (M-651 #308), llm-harness
  one-command `run.sh --all` (#307).

### The 1.0.0 gate — **CLOSED** (ADR-021 Accepted; the maintainer enacts at the tag)

Every row green: **A1 · A2 · A3 · A4 · A5 · B1 · B2.** The kernel/core is **1.0.0-ready**.
- **M-655** (cut 1.0.0) is **unblocked** and **maintainer-reserved**: move ADR-021 `Accepted → Enacted`
  (append-only) + tag the kernel/core set (per-crate SemVer, ADR-018) + roll CHANGELOG `[1.0.0]`.
- **Post-1.0 work = the active wave** (see `.claude/kickoff.md`): E7-1 + E7-2 (language/runtime
  completeness), dogfooding (web/ADK phyla, doc-site, LSP completions), and **M-649** (self-hosting).
- **Maintainer-reserved (excluded from the wave):** M-655 (tag) and M-381/M-646 (LLM local runs).

### Corpus status

| Layer | Status |
|-------|--------|
| RFC-0001…0010 | Accepted |
| RFC-0011…0015 | **Enacted** (`crates/mycelium-lsp/`) |
| RFC-0016 / 0017 / 0021 | **Enacted** (stdlib · maturation · projection framework) |
| RFC-0018 / 0019 | Accepted (stage-1 grading / traits — **not yet implemented**; E7-1 closes them) |
| RFC-0020 | Accepted (scoped) |
| ADR-010…021 | Accepted (ADR-020 **Enacted**; **ADR-021** gate met, awaiting `Enacted` at the 1.0.0 tag) |
| stdlib specs | **25/25 ratified**; only `self-hosting-readiness` Draft |
| DNs | DN-01…03,06…13,16,19 Resolved; **DN-14** Draft (self-hosting gate — E7-1); DN-15,17,18 Draft |

### Implementation state

- **Rust-first stdlib**: 25 `mycelium-std-*` crates, all with guarantee matrices, **all specs ratified**
  (DN-16 2026-06-21 re-audit: 24/25 clean + `sys` spec written → 25/25; no honesty-tag violations).
- **Native codegen**: M-601…M-630 — MLIR→LLVM, BitNet, deployable Spore (libMLIR-gated).
- **Runtime phylum**: ADR-020 Enacted — `crates/mycelium-std-runtime` v0 R1.
- **Toolchain**: `myc-check`/`mycfmt`/`myc-lint`/`myc-sec`/`myc-doc`/`spore`/`bench`/`lsp`. `just docs-site`
  builds a local browsable docsite (corpus + api-index + rustdoc → `target/docsite/`).
- **KC-2 / M-381**: **RESOLVED** — verdict = proceed; the **rigorous arm-4 LlmCanonical→L1 bridge**
  (DN-09 §9.4 option b) made the **retention ratio DETERMINATE** (grok-build-0.1 5.50×, grok-4.3 2.20×;
  both ≥70% → §4.7 trigger does not fire; DN-09 §10). arm-3/arm-5 modules landed; live runs **backlogged**.

### Open items (issues.yaml)

| ID | Title | Status |
|----|-------|--------|
| **E7-1** | L1 Stage-1 language completeness — generics→traits→effects→FFI→phylum→grading (M-656…M-664) | needs-design (active wave) |
| **E7-2** | RFC-0008 runtime vocabulary — M-665 (lexer) → M-666/667 constructs → M-668 R2 | needs-design (M-665 in flight) |
| **Dogfooding** | web phylum (planned RFC-0022) · ADK port (planned RFC-0023) · doc-site build · LSP completions | research / active |
| **M-649** | self-host the first stdlib nodule in Mycelium-lang | needs-design (after E7-1; M-502 ✅) |
| M-655 | Cut 1.0.0 tag — ADR-021 → Enacted | **maintainer-reserved** |
| M-381 / M-646 | LLM-leverage ablation arms 3/5 — local runs | **maintainer-reserved** |

> **Component memory files:** see `.claude/memory/` — compact per-component orientation
> (value model, swaps/certificates, VSA, numerics/dense, selection/EXPLAIN, language/execution,
> **lexicon/syntax/grammar**, toolchain, stdlib, honesty model, experiments/LLM). Load the relevant
> one before deep work.

---

## Swarm dev quick-reference

Branch naming (kebab-case, Base36 IDs):

```
Orchestrator   claude/orch-0000-<kebab>
Epic           claude/epic/<EPIC>-<kebab>
Leaf           claude/leaf/<EPIC>-<LEAF>-<kebab>
```

Collision-free invariants: each agent owns a **disjoint directory**; shared files
(`CHANGELOG.md`, `docs/Doc-Index.md`, `tools/github/issues.yaml`, `docs/api-index/`) are
**orchestrator-owned** — leaf/epic agents treat them read-only and FLAG needed changes up.

Merge flow: **bottom-up octopus** (leaf → epic → orch) then **squash PR → main**.
Always PR into main; never push main directly.

Before assigning a new M-xxx or E-xxx ID, verify the slot is free:
`grep "id: M-640" tools/github/issues.yaml` (adapt as needed).

---

## Skills (`.claude/skills/`)

- `/dev-workflow` — implementation discipline
- `/pr-review` — honesty/grounding/append-only diff review
- `/changelog` — keep CHANGELOG.md + per-doc footers in sync
- `/land` — self-review + green check → curated squash PR → main
- `/docs-review` — cross-refs, notation, grounding labels
- `/security-review` — secrets, supply-chain, shell/CI safety
- `/doc-index` — regenerate and query `docs/api-index/`; check `doc_refs` grammar validity
- `/deep-research` — fan-out multi-source research + adversarial verification (the research **follow-up** phase)
