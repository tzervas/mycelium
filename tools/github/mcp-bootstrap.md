# Mycelium GitHub bootstrap — MCP runner (model-executed)

This is the **machine-runnable** counterpart to `gh-bootstrap-local.sh`. A model (Claude Code or
Grok) with the **GitHub MCP server** attached executes the steps below by calling the MCP
tools directly. It fills the gaps a model **can** fill over MCP; the gaps it **cannot**
(label colors/descriptions, milestone *creation*) are handled by `gh-bootstrap-local.sh`.

> Why split? The GitHub MCP server exposes issue + sub-issue write tools but **no
> label-management or milestone-creation tool**. So labels-with-colors and milestones are
> created locally with `gh`; everything else is done here over MCP.

**No MCP session? Run it from a phone.** `termux-bootstrap.md` + `termux-setup.sh` drive the
whole bootstrap (packages, git identity, GPG signing key, `gh` auth, then
`gh-bootstrap-local.sh` + `gh-issues-sync.py`) from Termux. `gh-issues-sync.py` is the
gh-driven local analogue of Steps 1–2 below (idempotent create + milestone + `idmap.tsv`);
Step 4 (dependency/sub-issue linking) still needs an MCP/GraphQL pass.

## Tool / capability matrix

| Task | Tool | MCP can do it? |
|---|---|---|
| Create / update issue (title, body, labels, milestone-number, assignees, state) | `issue_write` | ✅ |
| List issues (idempotency check by title) | `list_issues` | ✅ |
| Add / remove / reorder sub-issues (epic children) | `sub_issue_write` | ✅ |
| Read a label | `get_label` | ✅ (read only) |
| Create / recolor a label | — | ❌ → `gh-bootstrap-local.sh` |
| Create a milestone | — | ❌ → `gh-bootstrap-local.sh` |

## Verified MCP tool schemas

Verified in-session against the GitHub MCP server (`mcp__github__*`). Tool names below omit
the harness prefix; call them as your client exposes them.

### `issue_write`

```jsonc
{
  "method": "create" | "update",     // required
  "owner": "tzervas", "repo": "mycelium",  // required
  "title": "string",                 // create: required
  "body": "string",
  "labels": ["phase:0", "type:spec"],// label NAMES (strings)
  "milestone": 1,                    // milestone NUMBER (must already exist)
  "assignees": ["login"],
  "state": "open" | "closed",
  "issue_number": 5                  // update: required
}
```

Note: passing a `labels` name that does not yet exist makes GitHub auto-create it in the
default color. `gh-bootstrap-local.sh` repaints those to the spec colors/descriptions.

### `list_issues`

```jsonc
{ "owner": "tzervas", "repo": "mycelium",
  "state": "OPEN" | "CLOSED", "labels": ["phase:0"],
  "perPage": 100, "after": "<endCursor>" }   // paginate via pageInfo.endCursor
```

### `sub_issue_write`

```jsonc
{ "method": "add" | "remove" | "reprioritize",  // required
  "owner": "tzervas", "repo": "mycelium",        // required
  "issue_number": 28,        // PARENT issue number
  "sub_issue_id": 4617129312,// CHILD issue DATABASE id (NOT its number) — see idmap.tsv col 3
  "after_id": 0, "before_id": 0, "replace_parent": false }
```

### `get_label`

```jsonc
{ "owner": "tzervas", "repo": "mycelium", "name": "phase:0" }
```

## Inputs
- `issues.yaml` — source of truth (id, title, body, labels, milestone, depends_on).
- `idmap.tsv` — `task_id <TAB> issue_number <TAB> issue_db_id` (already created; refresh if you re-run).
- `/tmp/mycelium-msmap.tsv` — `milestone_number <TAB> milestone_title`, produced by
  `gh-bootstrap-local.sh`. Required for Step 2.

## Idempotency rule (applies to every step)
Before any `create`, fetch existing issues once with `list_issues` (paginate to cover all)
and build a `title -> number` set. **Create only titles that are absent.** Re-running must
not duplicate. For `update`, setting a field to the value it already has is a safe no-op.

---

## Step 1 — Issues (idempotent create)
For each entry in `issues.yaml.issues`:
1. If its `title` already exists (from the `list_issues` snapshot) → skip create, keep the
   existing number.
2. Else `issue_write{ method:"create", owner, repo, title, body, labels }`.
3. Record `id -> {number, db_id}` (the create response returns both `url` (→ number) and `id`).

Write/refresh `idmap.tsv` with all three columns.

## Step 2 — Assign milestones (requires `gh-bootstrap-local.sh` to have run first)
MCP cannot create milestones, so this step depends on the milestone map.
1. Load `/tmp/mycelium-msmap.tsv` → `title -> number`.
2. For each issue in `issues.yaml`, resolve its `milestone` (a title) to a number via the map.
3. `issue_write{ method:"update", owner, repo, issue_number:<n>, milestone:<num> }`.
   Idempotent: re-setting the same milestone is a no-op.

## Step 3 — Labels safety net (usually a no-op)
Step 1 already applies labels at create time. If you need to reconcile, `issue_write update`
with the full `labels` array for that issue. Colors/descriptions are **not** settable here —
run `gh-bootstrap-local.sh` for those.

## Step 4 — Dependencies / sub-issues (the Grok pass)
Use `idmap.tsv` (col 3 = database id) + each issue's `depends_on` in `issues.yaml`:
- **Epic children (Phase 2/3):** once an epic is decomposed, attach each child with
  `sub_issue_write{ method:"add", issue_number:<parent number>, sub_issue_id:<child db_id> }`.
- **`depends_on` ("blocked by"):** prefer GitHub's issue-dependencies API
  (not exposed as an MCP tool here — use GraphQL/REST in the Grok pass); if unavailable,
  fall back to a task-list checklist in the blocking issue. Resolve every `M-xxx`/`E*` id to
  its number/db-id via `idmap.tsv`.

## Field-mapping reference (labels → Project fields, for the board pass)
`phase:* → Phase`, `area:* → Area`, `priority:* → Priority`, `status:blocked → Status:Blocked`.
Status defaults to `Todo` on add. See `project-v2-spec.md`.
