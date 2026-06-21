# Kickoff `dfr` — Web/ADK Deep-Research Follow-up (the two-phase gate)

> Stowed kickoff, UID **`dfr`**. Read `.claude/agent-context.md` + `CLAUDE.md` first.

## Head branch (your locked base)
**`claude/head/dogfood-research`** — protected, persistent base; all work merges into it; `main` is
PR-only; the head → `main` PR is the final step.

## Mission
Run the **follow-up deep-research pass** that GATES the two dogfooding RFCs: discharge **RP-10** (web,
RFC-0022) and **RP-9** (ADK, RFC-0023) — the Honest-Uncertainty Registers in the RFCs / RECORDs. Move
each RFC **Draft → Accepted *only if* its register discharges**; else keep it **Draft** and record
exactly what remains open (never silently close a gate).

## Ownership
- **You own:** `docs/rfcs/RFC-0022-Web-Tooling-Phylum.md` + `RFC-0023-Agent-Development-Kit-Phylum.md`
  (Status only — append-only), `research/12-web-phylum-RECORD.md` + `research/13-adk-phylum-RECORD.md`
  (append findings), `docs/notes/research-prompts.md` (RP-10/RP-9 → discharged/open).
- **Read-only / FLAG up:** `issues.yaml`, `CHANGELOG.md`, `Doc-Index.md`.

## Swarm method — scoped to **research, zero code collision → FRACTURED Opus reasoners**
Per the maintainer's methodology: **fracture each RP into tightly-scoped Opus max-effort sub-reasoners
sharing one tight cross-context packet** (the RFC + its RECORD + the open gates). Invoke `/deep-research`
per sub-question (fan-out web search → fetch → adversarial verify → cited synthesis). ~4 reasoners per
RFC — e.g. web: {HTTP/never-silent · JSON-codec-reuse vs `std.io` · server-as-`colony` determinism ·
routing/EXPLAIN}; adk: {ADK→Mycelium concept map · honesty-as-differentiator · tool-dispatch never-silent ·
session/runner + LLM-harness reuse}. The orchestrator synthesizes findings into the RECORD + a discharge
assessment. **Keep reasoners properly sized; do not let the orchestrator's context balloon — summarize
each reasoner's result, don't inline raw search dumps.**

## Merge / branch method
Reasoners return findings (docs → no shared-file collision). The orchestrator writes the synthesis +
RFC Status onto `claude/head/dogfood-research`. Head → `main` PR is the final step.

## Continuity — **CRITICAL: this unblocks `dfb`**
When RP-10 / RP-9 **discharge**, update **M-670** (web) / **M-671** (adk) issue **bodies**: state the
gate is cleared + any constraint the Rust-first build must honor. That edit is the signal the `dfb`
build session waits on.

## Honesty / done
Findings **Empirical/Declared**, never `Proven`. An RFC moves past Draft only with the register
discharged; an undischarged gate stays an explicit open item. Append-only status. **Done** = RP-10 +
RP-9 each assessed (discharged, or explicitly-still-open with the open set named) on the head, RFC
Status updated, M-670/M-671 bodies updated, ready for final integration to `main`.
