# Grok handoff — finish Mycelium's GitHub project management

This is the **second pass** of the PM bootstrap. Claude Code did the first pass over the GitHub
MCP server (which can write issues but cannot manage labels/milestones or build a Project v2
board). Everything still outstanding is collected here for **Grok**, which has full GitHub
GraphQL/REST access.

> Paste the **prompt block** below into Grok, and attach the accompanying archive
> (`mycelium-grok-handoff.tar.gz`). All file references in the prompt are to files in that archive.

---

## Current state (authoritative — as of this handoff)

Repo: **`tzervas/mycelium`**.

- **Issues: done.** 40 issues exist and are **OPEN**, numbered **#2–#41** (issue #1 is an
  earlier PR). The `id → issue-number → issue-db-id` map is `idmap.tsv` (40 rows).
- **Labels: partial.** Each issue already carries its correct label **names**
  (`phase:* type:* area:* priority:*`, plus `status:needs-design` on #35/#39). But those labels
  were **auto-created by GitHub in default gray with no descriptions**, and the **5 labels not
  used by any issue do not exist yet** (`type:docs`, `type:chore`, `type:bug`, `status:blocked`,
  `good-first-issue`). The full, correct spec for all **26** labels is `labels.json`.
- **Milestones: not created.** None of the 4 milestones exist yet, and **no issue has a
  milestone assigned**. Spec: `milestones.json`; per-issue milestone titles: `issues.yaml`.
- **Project v2: not created.** Spec: `project-v2-spec.md`.
- **Dependencies: not wired.** Per-issue `depends_on` (task-ids) live in `issues.yaml`; resolve
  them to numbers/db-ids via `idmap.tsv`.

Everything below is **idempotent**: re-running must not duplicate (match labels by name,
milestones by title, issues by number, project items by content id).

---

## Prompt block (paste into Grok)

```text
You are completing the GitHub project-management setup for the repo tzervas/mycelium using the
GitHub GraphQL + REST APIs. I've attached an archive; all files referenced below are in it.

Current state (do not recreate what exists):
- 40 issues already exist, OPEN, numbered #2–#41. Map: idmap.tsv (task-id, issue-number, db-id).
- Each issue already has its label NAMES, but the labels are default-gray with no descriptions,
  and 5 labels are missing entirely. Full spec: labels.json (26 labels).
- No milestones exist and no issue has a milestone. Spec: milestones.json; per-issue milestone
  title: issues.yaml.
- No Project v2 exists. Spec: project-v2-spec.md.
- Dependencies are not wired. Per-issue depends_on (task-ids) are in issues.yaml.

Do all of the following, idempotently (match labels by name, milestones by title, issues by
number, project items by content id; skip/patch instead of duplicating):

1. LABELS — reconcile every label in labels.json to its exact name+color+description. Create the
   5 missing ones and repaint the 21 default-gray ones (POST/PATCH /repos/tzervas/mycelium/labels).

2. MILESTONES — create the 4 milestones from milestones.json (title, state, description) if absent
   (POST /repos/tzervas/mycelium/milestones); build a title→number map.

3. ASSIGN MILESTONES — for each issue in issues.yaml, resolve its `milestone` title to a number
   and set it on the issue by number from idmap.tsv (PATCH the issue). Re-setting is a no-op.

4. PROJECT v2 — create a user-owned Project v2 titled "Mycelium" for owner tzervas, linked to the
   repo, exactly per project-v2-spec.md: single-select fields Status (Todo/In Progress/In Review/
   Blocked/Done — reuse built-in Status if present), Phase (Phase 0..3), Area (core-ir/swap/vsa/
   execution/numerics/selection/toolchain/project), Priority (P0/P1/P2), Estimate (S/M/L), plus an
   optional 2-week Iteration field. Use the field colors/options from the spec.

5. ADD ITEMS + FIELDS — add all 40 issues (idmap.tsv) to the project. For each item set Phase /
   Area / Priority from the issue's labels (phase:*→Phase, area:*→Area, priority:*→Priority), set
   Status = Todo, and set Estimate from the planning docs (phase-0.md … phase-3.md) if a size is
   given, else leave blank.

6. VIEWS — create three views: Board (group by Status), By Phase (table, group by Phase, sort by
   Priority), Roadmap (by Iteration, or by Phase until iterations start).

7. AUTOMATION — auto-add repo issues labeled phase:*; item-added → Status Todo; issue closed /
   PR merged → Status Done; label status:blocked → Status Blocked.

8. DEPENDENCIES / SUB-ISSUES — using each issue's depends_on in issues.yaml + idmap.tsv:
   set "blocked by" links via GitHub's issue-dependencies API (REST: the issues
   dependencies endpoints, or GraphQL); if dependencies aren't enabled on the repo, fall back to a
   task-list checklist in the blocking issue. The Phase 2/3 epics (E2-*, E3-*) are NOT decomposed
   yet — when they later are, attach children as sub-issues with GraphQL addSubIssue using the
   child's db-id (idmap.tsv col 3), not its number.

Report back: the project number + URL; the created field ids and option ids; how many items were
added; milestones created + how many issues got each; labels created vs repainted; and every
dependency link you could not create, with the reason.
```

---

## Inputs (archive manifest)

| File | Use |
|---|---|
| `labels.json` | 26 labels — names, colors, descriptions (step 1) |
| `milestones.json` | 4 milestones — title, state, description (steps 2–3) |
| `issues.yaml` | per-issue title, labels, **milestone title**, **depends_on** (steps 3, 5, 8) |
| `idmap.tsv` | `task-id <TAB> issue-number <TAB> issue-db-id` (steps 3, 5, 8) |
| `project-v2-spec.md` | fields, options, colors, views, automation, GraphQL skeleton (steps 4–7) |
| `mcp-bootstrap.md` | verified GitHub tool schemas (reference, if Grok uses the MCP surface) |
| `phase-0.md … phase-3.md`, `program-plan.md`, `github-mapping.md` | epic decomposition + Estimate sizing context (steps 5, 8) |

## Acceptance

- All 26 labels match `labels.json` (color + description); 4 milestones exist; every issue has the
  milestone its `issues.yaml` entry names.
- A "Mycelium" Project v2 exists with the spec'd fields/options/views/automation; all 40 issues are
  items with Phase/Area/Priority/Status set.
- `depends_on` is reflected as "blocked by" links (or task-list fallback), resolved via `idmap.tsv`.
- Re-running changes nothing (idempotent).
