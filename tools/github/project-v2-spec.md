# GitHub Project (v2) spec — Mycelium

GitHub **Projects v2** custom fields, options, and views are created via the **GraphQL API** (the REST API and most of `gh project` cannot create single-select options or views). This is the part to hand to **Grok** (or run with `gh api graphql`). Below is the desired end state + the GraphQL skeleton.

## Project
- **Name:** `Mycelium`
- **Owner:** user `tzervas`
- **Linked repo:** `tzervas/mycelium`
- **Description:** "Design → build tracking for the Mycelium language."

## Custom fields
| Field | Type | Options |
|---|---|---|
| **Status** | single-select | `Todo`, `In Progress`, `In Review`, `Blocked`, `Done` |
| **Phase** | single-select | `Phase 0`, `Phase 1`, `Phase 2`, `Phase 3` |
| **Area** | single-select | `core-ir`, `swap`, `vsa`, `execution`, `numerics`, `selection`, `toolchain`, `project` |
| **Priority** | single-select | `P0`, `P1`, `P2` |
| **Estimate** | single-select | `S`, `M`, `L` |
| **Iteration** | iteration (optional) | 2-week cadence, start when Phase 1 opens |

(GitHub provides a built-in `Status` field on new projects; you may reuse/rename it rather than creating a second one.)

## Views
1. **Board** — layout: Board, group by **Status**. The day-to-day Kanban.
2. **By Phase** — layout: Table, group by **Phase**, sort by **Priority** then Estimate.
3. **Roadmap** — layout: Roadmap, by **Iteration** (or **Phase** until iterations start).

## Automation (Project workflows; set in Project settings or via API)
- **Auto-add**: items from `tzervas/mycelium` with any `phase:*` label.
- **Item added → Status = Todo.**
- **PR merged / issue closed → Status = Done.**
- **Label `status:blocked` added → Status = Blocked.**

## GraphQL skeleton (for Grok / `gh api graphql`)

```graphql
# 1) Find owner id
query { user(login:"tzervas"){ id } }

# 2) Create the project
mutation($ownerId:ID!){
  createProjectV2(input:{ownerId:$ownerId, title:"Mycelium"}){ projectV2 { id number } }
}

# 3) Create a single-select field with options (repeat per field)
mutation($projectId:ID!){
  createProjectV2Field(input:{
    projectId:$projectId, dataType:SINGLE_SELECT, name:"Phase",
    singleSelectOptions:[
      {name:"Phase 0", color:GREEN, description:""},
      {name:"Phase 1", color:BLUE,  description:""},
      {name:"Phase 2", color:PURPLE,description:""},
      {name:"Phase 3", color:RED,   description:""}
    ]
  }){ projectV2Field { ... on ProjectV2SingleSelectField { id options { id name } } } }
}

# 4) Add an issue to the project (gh can also do this: `gh project item-add`)
mutation($projectId:ID!, $contentId:ID!){
  addProjectV2ItemById(input:{projectId:$projectId, contentId:$contentId}){ item { id } }
}

# 5) Set a single-select field value on an item
mutation($projectId:ID!, $itemId:ID!, $fieldId:ID!, $optionId:String!){
  updateProjectV2ItemFieldValue(input:{
    projectId:$projectId, itemId:$itemId, fieldId:$fieldId,
    value:{ singleSelectOptionId:$optionId }
  }){ projectV2Item { id } }
}
```

Map each issue's labels (`phase:*`, `area:*`, `priority:*`) to the corresponding field option ids from step 3. The runbook (`setup-runbook.md`) gives the full ordered procedure.
