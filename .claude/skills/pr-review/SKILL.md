---
name: pr-review
description: >-
  Opinionated pull-request / diff review for the Mycelium repo. Enforces
  CONTRIBUTING.md — the honesty rule (per-op guarantee tags), append-only ADR/RFC
  discipline, grounding citations, and the "which FR/NFR/VR/SC + how verified"
  expectation — and does a hallucination/consistency pass. Auto-scales depth to the
  change (editorial → standard → deep) and can run an exhaustive all-severities pass.
when_to_use: >-
  Use when asked to review a PR, a branch, a commit, or "the diff"; to sanity-check
  changes before opening/merging a PR; or to catch hallucinated/unsupported claims.
argument-hint: "[PR#|<base>...<head>|<commit>] [--tier T0|T1|T2] [--all]"
allowed-tools: Bash(git diff:*), Bash(git show:*), Bash(git log:*), Bash(git status:*), Read, Grep, Glob
---

# pr-review

Opinionated, repo-grounded PR review. Shared triage/severity/report contract:
[review-rubric](../_shared/review-rubric.md) — **read it first**, classify the change into a
tier (or honor `--tier` / `--all`), and emit the standard report.

## 1. Get the change
- PR number → read it via the GitHub MCP `pull_request_read` (metadata + diff + linked issue).
- `<base>...<head>` / branch / commit → `git diff <base>...<head>` or `git show <ref>`.
- Note the change's stated intent (PR body / commit message) so you can check intent‑vs‑diff.

## 2. Triage (see rubric §1)
Score size, change-kind, path-fragility, and honesty-rule surface → choose **T0/T1/T2**.
Report the tier and the signals.

## 3. Review checklist (depth per tier)
**Correctness & intent**
- Does the diff do what its message claims — nothing more (scope creep), nothing less?
- For code: invariants hold; error paths explicit (no silent failure — esp. swaps/conversions
  must never be silent; out-of-range returns `Option`/error, never a quiet value).
- Logic/edge cases; concurrency; resource cleanup. New behavior has tests.

**Mycelium honesty rule** (non-negotiable — CONTRIBUTING)
- Any guarantee/bound carries a per-model/per-op tag `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`.
  A `Proven` tag is valid **only** if it cites a theorem whose side-conditions are *checked*;
  otherwise it must be `Empirical` or `Declared`. Flag any upgrade lacking a checked basis.
- A shipped approximate operation must include its **bound + guarantee tag + a property test
  that exercises the bound** (SC-2). Missing any → High.
- No black boxes: representation selection / conversions must be reified + `EXPLAIN`-able.

**Decision discipline & grounding**
- ADR/RFC changes are **append-only**: status moves forward (Draft→Accepted→Superseded); to
  change an Accepted decision you supersede it, never silently rewrite. Flag in-place edits to
  Accepted normative content.
- Every normative claim cites its grounding (`G1–G11`, tensions `A–E`, `R1–R8`, `T0.x/T1.x/T2.x`)
  or is explicitly marked an open question. Ungrounded assertions → High.

**Hallucination / consistency pass**
- Verify every factual/citation claim in the diff against the actual source file/section/line.
  Cross-references must resolve (run/trust `just links`). Numbers, IDs (FR/NFR/VR/SC/KC, RFC §),
  file paths, and API names must be real. A claim contradicting its source → Critical.

**Hygiene**
- Conventional, imperative commit subject referencing the issue/task. PR states which
  `FR/NFR/VR/SC` it advances (or which ADR/RFC it implements) and **how it was verified**;
  editorial-only PRs say so.
- Engineering house style (SOLID/DRY/KISS/YAGNI/Demeter/SoC, composition over inheritance,
  small auditable kernel). Changelog updated when a doc status moved or a notable change landed
  (defer to the `changelog` skill).

## 4. Report
Emit the rubric §4 format. Posture is **advisory** — recommend, don't gate. Be specific:
`file:line → what → why → fix`. End with summary, analysis, and prioritized recommendations
(always in `--all` mode).
