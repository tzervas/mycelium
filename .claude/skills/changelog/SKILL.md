---
name: changelog
description: >-
  Maintain Mycelium's changelogs. Keeps the top-level CHANGELOG.md (Keep a Changelog
  1.1.0, ISO dates, design-phase framing) and each doc's changelog footer in sync with
  a change, and enforces the append-only status discipline (Draft→Accepted→Superseded,
  Resolved for notes). Use when finishing a change, moving a doc's status, or cutting a release.
when_to_use: >-
  Use when staged/committed changes need a changelog entry, when an ADR/RFC/DN status
  transitions, before opening a PR, or when preparing a release section.
argument-hint: "[summarize-staged | add <Added|Changed|Deprecated|Removed|Fixed|Security|Open> <entry> | release <version>]"
allowed-tools: Bash(git diff:*), Bash(git status:*), Bash(git log:*), Read, Grep, Glob, Edit
---

# changelog

Keep the record honest and append-only.

## Top-level `CHANGELOG.md`
- Format: **[Keep a Changelog](https://keepachangelog.com/en/1.1.0/)**, ISO-8601 dates. The repo
  is in the **design phase** — "changes" are to the docs corpus; versioning starts when the
  kernel does. Preserve that framing.
- Add entries under **`## [Unreleased]`** in the right group. Standard groups: **Added,
  Changed, Deprecated, Removed, Fixed, Security**. This repo also uses a custom **`### Open`**
  section for tracked-but-unresolved items (e.g. the LH probe, KC-2) — keep it.
- Entry style: terse, past-tense-free imperative-ish bullets that name the doc/path and the
  grounding where relevant (mirror existing entries, e.g. "Foundation §5.6 updated: MLIR→LLVM
  recorded as the committed AOT path (ADR-007 / RFC-0004)."). Group related edits.
- To draft from staged work: read `git diff --staged` (and `git status`), map each change to a
  group, and propose bullets. Don't invent changes that aren't in the diff.

## Per-doc changelog footers & status
- Each RFC/ADR/DN carries its own changelog footer and a **status**. When content changes,
  append to that footer and move status only **forward**: `Draft/Proposed/Preliminary →
  Accepted → Superseded` (design notes: `→ Resolved`).
- **Never rewrite an Accepted decision in place** — supersede it with a new revision/file and
  link the old one forward. Editorial fixes + status transitions are the only in-place edits;
  record them.
- Keep `Proven | Empirical | Declared` tags honest per VR-5 (per model/op, never aggregate);
  a changelog note that *upgrades* a tag must point to the checked basis that earned it.

## Releases (later)
- When versioning starts: move `[Unreleased]` items into a dated `## [X.Y.Z] - YYYY-MM-DD`
  section, add comparison links, and reset `[Unreleased]`.

## Output
Propose the exact diff (which file, which section, which bullets). Confirm cross-references in
new bullets resolve. Note if a status transition implies a supersede that hasn't been written.
