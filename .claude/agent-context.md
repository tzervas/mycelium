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
| **Cited memory (tero)** | `docs/tero-index/INDEX.md` — the queryable corpus index (DN-87/E39-1); prefer **`/tero-query`** for cross-cutting answers-with-provenance, grep the INDEX.md as the offline fallback. Use it to ground a claim in one hop instead of re-reading the corpus. The `tero-mcp-lite` server (`packages/tero-mcp-lite/`) is registered via the repo-root `.mcp.json`, so the `mcp__tero__*` tools are usable directly in-session |
| **Portable MCP packages** | `packages/` — `tero-mcp-lite/` (the registered `tero` MCP server) + `GROK-HANDOFF.md` + `BACKLOG.md` |
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

## Current state

Current state → `docs/CURRENT-STATE.md` (the dense pointer index; refreshed at integration-tier
close-out). Full dated history of this section (through 2026-06-29) →
`docs/archive/agent-context/log-2026-06.md`.

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
- `/transpile-vet` — transpile Rust → vet with `myc check` → read `checked_fraction` (gap-profiling instrument, not a bulk porter)
- `/myc-drafts` — the `gen/myc-drafts/` corpus: triage before porting, graduate drafts into `lib/`, run M-1006 ladder phases
- **`/tero-query`** — the **transparent memory API** (DN-87/E39-1): cited, provenance-carrying answers about
  decisions/issues/docs/changelog over MCP or HTTP. **Leverage it for memory + context** — an uncited query
  returns a typed refusal, never a silent empty. Companions: `/tero-cite` (provenance only), `/tero-explain`
  (why-these-sources trace), `/tero-refresh` (reload the served index after `just tero-index`). Prefer it
  over grepping the corpus by hand when you want the answer **with** its citation; offline fallback is
  grepping the committed `docs/tero-index/INDEX.md`. The `mcp__tero__*` tools are already live in-session
  via the repo-root `.mcp.json` (`packages/tero-mcp-lite/`) — no manual server start needed.
