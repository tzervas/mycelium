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

## Current state (2026-06-20)

### Corpus status

| Layer | Status |
|-------|--------|
| RFC-0001…0010 | Accepted (ratified; implementations vary — see §Enacted below) |
| RFC-0011…0015 | **Enacted** (implemented in `crates/mycelium-lsp/`) |
| RFC-0016…0021 | Accepted (ratified; enactment staged by M-5xx/M-6xx) |
| ADR-010…020 | Accepted (ADR-020 **Enacted** — M-521 runtime phylum v0) |
| DN-01…03, 06…09, 13, 16 | **Resolved** |
| DN-04, 05, 10, 11, 14, 15, 17, 18 | **Draft** (decisions/captures pending) |

### Implementation state

- **Rust-first stdlib**: M-501…M-534 done — 19 stdlib crates + 4 Tier-A completions
- **Native codegen**: M-601…M-630 done — MLIR→LLVM, BitNet, deployable Spore
- **Runtime phylum**: M-521 Enacted — `crates/mycelium-runtime` v0 R1
- **Toolchain**: M-361…M-385 done — mycfmt, myc-check, myc-lint, myc-sec
- **L1 parser + grammar**: working (`myc-check` exit-2 on invalid, exit-0 on valid)
- **Self-hosting** (≥1 stdlib module in Mycelium-lang): **not yet done** (Phase 5 gate)
- **M-381 research** (retention-ratio ablation): in-progress, non-blocking — arm2 100%
  [Empirical], arm4 INDETERMINATE (scorer limitation, not model failure)

### Open items (issues.yaml)

| ID | Title | Status |
|----|-------|--------|
| M-381 | LLM-leverage ablation (T3.6) — arm4 needs LlmCanonical scorer | in-progress |
| Draft DNs | DN-04/05/10/14/15/17/18 — design decisions / analysis captures | draft |
| Self-hosting | First stdlib module in Mycelium-lang (L1/L2 syntax) | not yet issued |
| RFC enactment | Several Accepted RFCs have complete Rust implementations but Enacted not yet flipped | editorial |

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
