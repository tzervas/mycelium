# docs/archive/ — verbatim historical archive

**Purpose:** a verbatim archive of accreted historical/iterative doc clusters (and one superseded
ADR), moved here byte-for-byte (`git mv`, house rule #3 — append-only, never rewritten) so the
front-facing docs stay lean. The lean front lives at the repo root, `docs/Doc-Index.md`, and the
normative corpus (`docs/spec/`, `docs/adr/`, `docs/rfcs/`, `docs/notes/`, `docs/guide/`,
`docs/reference/`, `docs/wiki/`). Nothing here was edited for content; only two mechanical
depth-corrections were applied where a move broke a moved file's own outbound relative links (see
below) and the pointer-stubs needed to keep inbound references resolving.

| original path | archive path | date | why |
|---|---|---|---|
| `docs/devlog/` (15 files) | `docs/archive/devlog/` | 2026-07 | historical-iterative — dated append-only narrative devlog entries (design/build sessions), superseded as day-to-day reference by the RFC/ADR/DN corpus |
| `docs/handoffs/` (15 files, incl. `research/` subdir) | `docs/archive/handoffs/` | 2026-07 | historical-iterative — session/wave handoff context notes and research-lane synthesis docs, consumed at the time; not living reference material |
| `docs/reviews/2026-06-14-deep-review/` (7 files) | `docs/archive/reviews/2026-06-14-deep-review/` | 2026-07 | historical-iterative — a point-in-time four-stage deep-review snapshot (correctness/security/quality/roadmap + the M-653 Medium-findings ledger), closed out and referenced by citation only |
| `docs/verification/` (5 `M-xxx-verified.md` files) | `docs/archive/verification/` | 2026-07 | historical-iterative — per-milestone verification write-ups for landed work, cited by ID from spec/guide docs rather than browsed directly |
| `docs/adr/ADR-021-1.0.0-Release-Readiness-Gate.md` | `docs/archive/adr/ADR-021-1.0.0-Release-Readiness-Gate.md` | 2026-07 | superseded — ADR-021 is Superseded by ADR-022 (2026-06-23); its kernel Gate A/B criteria live on as ADR-022 track T1, so ADR-021 itself is historical record, not a live decision doc |

## Pointer-stubs

A pointer-stub (a one-paragraph redirect, at the old path) was created only where a mechanical
reference-integrity check actually flagged the old path as referenced:

- `docs/reviews/2026-06-14-deep-review/06-medium-findings-ledger.md` — flagged by
  `tools/github/doc_refs_check.py` (M-653's `src:` refs). The stub also satisfies the directory-level
  `src:docs/reviews/2026-06-14-deep-review` ref (existence-only check).

No other old path was flagged by `scripts/checks/links.sh` or `doc_refs_check.py`, so no other stub
was created — the remaining mentions of the old paths across the corpus (`CHANGELOG.md`,
`tools/github/issues.yaml`, various RFC/ADR/DN prose) are historical prose citations in backtick
code spans, not live markdown links or `doc_refs:` entries, and are left as an accurate record of
what was true when written.

## Mechanical link-depth correction

`docs/archive/devlog/2026-06-17-phase9-wave-b-doc-build.md` carried two outbound relative links
(`../notes/Narrative-Capture-and-Authoring.md`, `../spec/Narrative-Authoring-Pipeline.md`) that
resolved correctly from the file's original location (`docs/devlog/`) but broke once the file moved
one directory deeper to `docs/archive/devlog/` — the link targets themselves were never moved. Both
were corrected to `../../notes/...` / `../../spec/...` so they still resolve to the same,
unmoved target. This is a mechanical path-depth fix required by the relocation itself, not a
content rewrite (see the Phase-1 kickoff commit for the full rationale).
