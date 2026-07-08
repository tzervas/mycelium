# GitHub Project (v2) spec — Mycelium

GitHub **Projects v2** custom fields, options, and items are reconciled over the **GraphQL API**
(the REST API and most of `gh project` cannot create single-select options or set item fields).
This file is the **human-readable companion**; the **machine source of truth** the tooling
consumes is [`project.json`](project.json). The reconciler is `gh-issues-sync.py --project`
(driven by `gh api graphql`) — it **replaces the old "hand it to Grok" step**.

> Reconcile it:  `python tools/github/gh-issues-sync.py --project --dry-run`  (preview), then drop
> `--dry-run` to apply. `--project` is also part of `--all` (the full maintenance suite) when the
> `gh` token carries the `project` scope.

## Project
- **Name:** `Mycelium`
- **Owner:** user `tzervas`
- **Linked repo:** `tzervas/mycelium`
- **Description:** "Design → build tracking for the Mycelium language."

## Custom fields (machine source: `project.json` → `fields`)
| Field | Type | Options |
|---|---|---|
| **Status** | single-select | `Todo`, `In Progress`, `In Review`, `Blocked`, `Done` |
| **Phase** | single-select | `Phase 0` … `Phase 8` (the live milestone set) |
| **Area** | single-select | `core-ir`, `swap`, `vsa`, `execution`, `numerics`, `selection`, `toolchain`, `project`, `language`, `stdlib`, `release`, `spec` |
| **Priority** | single-select | `P0`, `P1`, `P2`, `P3` |
| **Estimate** | single-select (optional) | `S`, `M`, `L` |

The **Area** options mirror the `area:*` labels in `labels.json` and the **Phase** options mirror
the `phase:*` labels / `milestones.json` — `--validate` fails if they drift out of parity. GitHub
seeds a built-in **Status** field on new projects; the reconciler **reuses it by name** and adds
the missing `In Review` / `Blocked` options rather than creating a second Status.

## Label → field mapping (machine source: `project.json` → `field_label_map`)
Each item's labels set its single-select fields, idempotently (a value is written only when it
drifts):
- `phase:N` → **Phase** = `Phase N`
- `area:X` → **Area** = `X`. **Multi-area (M-676):** an item may legitimately carry **several**
  `area:*` labels; the single-select Area field then **anchors to the PRIMARY area** — the value
  earliest in the field's declared option order (the **Area** row above; an alphabetical tie-break
  for any value outside that order) — and records the full span as an informational note. An item
  carrying at least one area label is therefore **never** left `Area: … — not set` (G2); this
  mirrors the primary-task milestone anchor (#353), maintainer-ratified 2026-06-23. Switch Area to
  a multi-select field if a board needs every area surfaced. Fields **without** a `multi` policy
  (Phase/Priority) still flag a genuine two-value conflict as unset — two of those is a real
  conflict, not legitimate multi-membership.
- `priority:PN` → **Priority** = `PN`
- `status:blocked` → **Status** = `Blocked`; `status:done` → **Status** = `Done`
  (`status:needs-design` has no Status option — it is **reported as unmapped, never guessed**, and
  the *item-added → Todo* workflow default stands).

## Views (settings-only — recorded intent, flagged manual)
View layout/grouping/sorting is **not reliably API-writable**, so the reconciler **records the
intent in `project.json` and FLAGS each as a manual step** (never silently skipped):
1. **Board** — layout: Board, group by **Status**. The day-to-day Kanban.
2. **By Phase** — layout: Table, group by **Phase**, sort by **Priority** then **Estimate**.
3. **Roadmap** — layout: Roadmap, by **Iteration** (or **Phase** until iterations start).

## Automation (settings-only — recorded intent, flagged manual)
The built-in Project **workflows** are not writable through the public GraphQL API today, so each
is recorded in `project.json` → `automation` and **FLAGGED as a manual step** by the run:
- **Auto-add**: items from `tzervas/mycelium` carrying any `phase:*` label.
- **Item added → Status = Todo.**
- **Issue closed / PR merged → Status = Done.**
- **Label `status:blocked` added → Status = Blocked.**

If/when GitHub exposes a workflow-mutation API, flip the entry's `api_writable` to `true` and the
reconciler will manage it instead of flagging it.

## Auth (preflight, never a blanket refresh)
Projects v2 mutations need the **`project`** scope on `gh`'s token. The reconciler **preflights**:
if `gh` is authenticated **and** the scope is present it proceeds silently; it only prints the
one-time remediation **`gh auth refresh -s project`** when the scope is **genuinely missing** (a
good token is never asked to refresh; a fine-grained token whose scopes can't be read is trusted
to fail loudly at the call site). `gh` holds the credential — **no token is read or stored** here.

## GraphQL skeleton (what the engine issues; also runnable by hand)

```graphql
# 1) Owner id            query { user(login:"tzervas"){ id } }
# 2) Find-or-create      query { user(login:"tzervas"){ projectsV2(first:100){ nodes{ id number title } } } }
#                        mutation { createProjectV2(input:{ownerId:$id, title:"Mycelium"}){ projectV2 { id number } } }
# 3) Create a field      mutation { createProjectV2Field(input:{projectId:$pid, dataType:SINGLE_SELECT,
#                          name:"Phase", singleSelectOptions:[{name:"Phase 0", color:GREEN, description:""} …]}){ … } }
# 4) Add an item         mutation { addProjectV2ItemById(input:{projectId:$pid, contentId:$nodeId}){ item { id } } }
# 5) Set a field value   mutation { updateProjectV2ItemFieldValue(input:{projectId:$pid, itemId:$iid,
#                          fieldId:$fid, value:{ singleSelectOptionId:$oid }}){ projectV2Item { id } } }
```

> **Validation caveat (honesty rule).** The pure mapping/diff logic (`label_to_field_values`,
> `plan_field_reconcile`) is covered by `--self-test`; the **live GraphQL path must be validated
> with `--dry-run`** on a machine with `gh` + the `project` scope before a live run. Until then its
> live behaviour is **Declared**, not Proven. Adding an option to an **existing** field is **flagged
> for the UI** in v0 (the only API path replaces the option set and is not safely idempotent without
> live validation) — creating a wholly-absent field with all its options is done over the API.
