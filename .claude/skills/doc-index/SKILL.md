---
name: doc-index
description: >-
  Regenerate and query the Mycelium agent code index (docs/api-index/), and
  validate doc_refs entries in issues.yaml. Use when you need to regenerate the
  committed agent index, query it for symbol locations, or check doc_refs validity.
when_to_use: >-
  Use when regenerating docs/api-index/ after a public-API change, when querying
  for a symbol's location in source, when validating doc_refs: entries in
  issues.yaml, or when checking whether the committed index is current.
allowed-tools: Bash(python3:*), Bash(just:*), Read, Grep, Glob
---

# /doc-index — Regenerate & Query the Agent Code Index

## Regenerate

```bash
just docs-index          # regenerate docs/api-index/ and commit the result
bash scripts/checks/doc-index.sh   # drift gate: PASS if current, FAIL if stale
```

The drift gate is also wired into `just check` (after `api`). It runs skip-graceful
when python3 is absent.

## Query

- `docs/api-index/INDEX.md` — grep-friendly table grouped by crate (for agent context)
- `docs/api-index/index.json` — machine-structured: `symbol`, `file`, `line`, `summary`

Example lookup: `grep "binary::bits_to_int" docs/api-index/INDEX.md`

## doc_refs grammar

In `tools/github/issues.yaml`, a `doc_refs:` list entry may be:
- `api:<crate>::<path>` — a symbol in `docs/api-index/index.json` (checked against `items`)
- `corpus:<DOC>[#<anchor>]` — a doc/section in `docs/Doc-Index.md`; the anchor is a heading
  slug heuristic (Empirical/Declared — may miss atypical headings)
- `src:<path>[:<line>]` — a source file location relative to repo root; if `:<line>` given,
  line must exist in the file

Validate all refs:

```bash
python3 tools/github/doc_refs_check.py --issues-yaml tools/github/issues.yaml
```

The validator also runs as part of `python3 tools/github/manifest-check.py`.

## Honesty

The index is `Empirical/Declared` — a line/regex heuristic. Source is always ground truth.
Items the heuristic can't locate appear in the `flagged` section of `index.json` and
`INDEX.md` (G2: never silently dropped). This includes re-exports (`pub use`),
macro-generated items, and cfg-gated items.

**Never claim the index is exact.** Use it to find where to `Read`, then verify in source.

## Ownership

`docs/api-index/` is **orchestrator-owned** — it is REGENERATED (never hand-merged) by the
integrating parent after any octopus merge that touched a public API, before pushing.
