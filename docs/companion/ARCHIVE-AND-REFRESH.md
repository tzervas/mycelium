# Corpus archive and front-matter refresh (policy + 2026-07-16 pass)

## Goals

1. **Archive** historical / superseded planning snapshots to the protected
   `archive` branch (never delete history — ADR-043 discipline applied to docs).
2. **Prune** the default reading surface so humans land in companion + guide +
   CURRENT-STATE, not a numeric DN flood.
3. **Keep** every Accepted/Enacted RFC/ADR/DN in-tree as the permanent record
   (append-only); **curate** access via thematic maps.

## What stays front-facing

| Path | Role |
| --- | --- |
| `docs/companion/**` | Thematic historian (this supplement) |
| `docs/guide/**` | Human guide set |
| `docs/wiki/**` | Short wiki mirrors |
| `docs/CURRENT-STATE.md` | Fast truth pointer |
| `docs/Doc-Index.md` | Status oracle |
| `docs/rfcs/**`, `docs/adr/**`, `docs/notes/**` | Normative history (complete set) |
| `docs/spec/**` | Specs |

## What is archive-candidate (planning noise / superseded snapshots)

Move **copies** to `archive` branch under `docs/archive/2026-07-16-companion-pass/`
when not required for live gates; leave a stub pointer in-tree if a path is linked
from issues. Candidates (verify no live `doc_refs` break before removal):

- Older one-shot planning dumps superseded by gap-analysis-2026-07-16
- Zero-hand-port intermediate drafts already superseded by DN-109…111 + inventory
- Duplicate "reading order" experiments once companion is the map

**This pass (landed with companion):**

- [x] Companion authored + sources committed under `_sources/`
- [x] Doc-Index + guide + README pointers
- [ ] Bulk move of superseded planning files (run after `doc_refs` audit — M-style
      chore; FLAG for integrator if issues.yaml points at paths)

## Archive branch procedure

```bash
git fetch origin archive
git checkout -B archive/docs-2026-07-16 origin/archive   # or orphan if needed
# copy trees from a worktree of main/dev at the freeze SHA
git commit -m "docs(archive): snapshot pre-companion corpus slice YYYY-MM-DD"
git push origin archive/docs-2026-07-16
# open PR into archive branch per repo policy, or push if archive allows
```

In-tree stub example:

```markdown
> **Archived:** full text lives on branch `archive` path
> `docs/archive/2026-07-16-companion-pass/<file>`. Companion map:
> `docs/companion/05-thematic-decision-map.md`.
```

## Ordering optimization (front matter only)

| Was | Becomes |
|---|---|
| Guide "read all RFCs in number order" | Guide → companion thematic map → selective RFCs |
| DN-119…140 as changelog | Cluster A–F in companion §05 |
| Memory L3 as last bullet | Lifecycle chapter with colony bridge |

Numeric DN/RFC **files are not renamed** (append-only / link stability).

## Honesty

Archive moves do not change Accepted status. Companion claims remain weaker than
normative docs unless cited.
