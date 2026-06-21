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
| **Phase plan** | `docs/planning/phase-5.md` (current); no phase-6 doc yet |
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

### The 1.0.0 gate (ADR-021 — Accepted; see `docs/notes/DN-19-Road-to-1.0.0.md`)

1.0.0 = the **kernel/core** once ADR-021's gate rows close. Status:
- **Closed:** A1 (zero open High) · A5 (KC-4 cert-overhead budget ratified ≤5 µs + ≤2× guardrail) ·
  B1 (RFC-0003/0006/0007 Accepted) · B2 (KC-2 verdict recorded — determinate retention ratio).
- **Open (the remaining gap = the next wave):** **A2** (M-653, Medium-findings ledger) · **A3**
  (M-654, WS8 durability — mutants/proptest/fuzz) · **A4** (M-652, `cargo deny`/`audit` in `just check`).
- Then **M-655**: cut 1.0.0 (ADR-021 `Accepted → Enacted` at the tagged release).
- **Out of scope for 1.0.0 (post-1.0/1.x):** surface language, self-hosting (M-502/M-649), native
  codegen, JIT, projections, RP-8 perf, arms 3/5 (ADR-021 §5).

### Corpus status

| Layer | Status |
|-------|--------|
| RFC-0001…0010 | Accepted |
| RFC-0011…0015 | **Enacted** (`crates/mycelium-lsp/`) |
| RFC-0016…0021 | Accepted (**ADR-021** = the 1.0.0 gate, **Accepted 2026-06-21**) |
| ADR-010…021 | Accepted (ADR-020 **Enacted**; ADR-021 Accepted) |
| stdlib specs | **25/25 Accepted** (DN-07 23 + runtime + sys on 2026-06-21); only `self-hosting-readiness` Draft |
| DNs | DN-01…03,06…10,12,13,16 + **DN-19** Resolved/captured; DN-04,05,14,15,17,18 Draft |

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
| **M-652** | A4 — `cargo deny`/`audit` into `just check` | open (1.0.0 gate) |
| **M-653** | A2 — Medium-findings ledger (close/defer each) | open (1.0.0 gate) |
| **M-654** | A3 — WS8 durability (cargo-mutants + proptest + fuzz) | open (1.0.0 gate) |
| **M-655** | Cut 1.0.0 — ADR-021 → Enacted at the tagged release | open (after A2/A3/A4) |
| M-647 | RFC-0020 L2 surface: scoped ratification (§4.2/§4.5 carve-out) | open |
| M-648 | Editorial sweep: landed RFCs → Enacted; Draft DN → Resolved | open |
| M-651 | Harness→bench schema bridge (Grok report ingestion) | open |
| M-649 | Self-hosting Stage-2 — **post-1.0** (M-502 gate) | open |
| M-381 | LLM-leverage ablation — headline DONE; arms 3/5 backlogged (non-blocking) | in-progress |

> **Component memory files:** see `.claude/memory/` — compact per-component orientation
> (value model, swaps/certificates, VSA, numerics/dense, selection/EXPLAIN, language/execution,
> toolchain, stdlib, honesty model, experiments/LLM). Load the relevant one before deep work.

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
