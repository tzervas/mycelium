---
name: docs-review
description: >-
  Documentation review for the Mycelium corpus. Checks that cross-references resolve,
  notation is normalized, grounding labels are valid, ADR/RFC/DN status transitions are
  legal and append-only, and the changelog reflects the change. Most of this repo is
  normative docs, so this is the primary review for a docs-only diff.
when_to_use: >-
  Use when reviewing changes to docs/, research/, READMEs, RFCs/ADRs/notes, or any
  markdown; or when asked to check documentation correctness/consistency before merge.
argument-hint: "[PR#|<base>...<head>|<commit>|<path>] [--tier T0|T1|T2] [--all]"
allowed-tools: Bash(git diff:*), Bash(git show:*), Bash(git log:*), Bash(just links:*), Bash(git grep:*), Read, Grep, Glob
---

# docs-review

Documentation review, repo-grounded. Shared triage/severity/report contract:
[review-rubric](../_shared/review-rubric.md) — read it first, pick a tier, emit the report.
A docs-only diff is still reviewed properly (correctness, grounding, consistency) even though
its security tier is light.

## Checklist
**References & structure**
- Every relative link / cross-reference / `@import` resolves (run/trust `just links` — the
  offline checker). A moved or renamed file must leave **no** dangling reference (prose,
  backticked paths, and tables included — the link checker won't catch backticked paths, so
  grep them: `git grep -n "old/path"`).
- Section/ID references are real: `RFC-000x §y`, `FR/NFR/VR/SC/KC` ids, ADR numbers, survey/
  research labels. A reference to a non-existent section/id → High.
- Doc index (`docs/Doc-Index.md`) and the dependency DAG stay consistent with what exists.
  Files marked *forthcoming* (e.g. `docs/planning/`) are fine as long as they're labeled so.

**Notation & house style**
- Guarantee lattice renders as `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` (the `⊐` form, not
  `⊃`); implication `⟹`; unicode superscripts (`µ²`). Flag notation drift (Medium).
- Markdown matches `.markdownlint.jsonc` (run/trust `just md`). Emphasis style consistent.

**Grounding & honesty (CONTRIBUTING)**
- Every normative claim cites its basis (`G1–G11`, tensions `A–E`, `R1–R8`, `T0.x–T2.x`) or is
  explicitly an open question. An ungrounded assertion stated as fact → High.
- Guarantee/bound language stays honest per VR-5 (per-model/op, never aggregate); a `Proven`
  claim names the theorem + checked side-conditions. Hallucinated citations → Critical.

**Decision discipline (append-only)**
- Status moves forward only (`Draft/Proposed/Preliminary → Accepted → Superseded`; notes
  `→ Resolved`). An Accepted decision edited in place (beyond editorial/status) → Critical;
  it must be **superseded** with a forward link instead.
- The doc's own changelog footer and the top-level `CHANGELOG.md` record the change (defer the
  edit to the `changelog` skill). A status transition with no changelog entry → Medium.

**Hallucination pass**
- Verify each new factual claim against its cited source file/section. Numbers, dates, names,
  and IDs must be real. Cross-check that the diff's stated intent matches what it actually changes.

## Report
Emit the rubric §4 format, advisory posture: `file:line → what → why → fix`, then summary,
analysis, and prioritized recommendations (always in `--all` mode).
