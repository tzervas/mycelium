# Kickoff `tul` — GitHub project-management tooling reconcile (`tools/github`)

> Read `.claude/agent-context.md` + `CLAUDE.md` (house rules win) + `.claude/kickoffs/README.md`
> first. **Fully disjoint from the L1 work** (`run`/`srf`) and from `dfr` — owns only `tools/github/`
> (+ `tools/github/*.py`), so **fire it in parallel** in its own session. Docs/tooling tier (low
> fragility); still runs the full `just check` before landing.

## ⚡ RESUME HERE

> **Status update (2026-07-05):** **M-675 is `done` (landed 2026-07-01)** — the idmap reconcile
> below is complete. **Only M-676 remains** (Projects-v2 Area field; P3, deferrable/secondary).

**Branch off `dev`.** Promote `dev → integration → main` per the tiered workflow.

**M-675 (#354) — reconcile `tools/github/idmap.tsv` against live GitHub — ✅ done (2026-07-01).** `idmap.tsv`
(task-id → issue-number → issue-db-id) is missing every Phase-5/6/7/8 issue (E7-x, M-5xx/6xx); the
track-a manifest appended the e7l rows but a full reconciliation needs each issue's live `issue_db_id`
(the sub-issue-link key) **fetched from GitHub** (the GitHub MCP `list_issues`/`issue_read`, or a
token-scoped `gh-issues-sync.py --use-api`).
- enumerate the live issues; map every `M-xxx`/`E*` id → number + db-id; rewrite `idmap.tsv`
  **append-only**;
- **Honesty (G2/VR-5): db-ids are *fetched, never guessed*** — the current gap is explicitly noted in
  `idmap.tsv`, keep that discipline (a number-only row beats a guessed db-id).
- **Acceptance:** `idmap.tsv` covers every non-epic id in `issues.yaml`; `manifest-check.py`
  cross-validates id↔number; the sub-issue/dependency/Projects-v2 sync dry-run resolves all db-ids
  with no "missing db-id" FLAG.

## Chain (dependency-ordered)
| # | Issue(s) | What | Status |
|---|---|---|---|
| 1 | **M-675** (#354) | reconcile `idmap.tsv` ↔ live GitHub (numbers + db-ids), append-only | ✅ **done (2026-07-01)** |
| 2 | **M-676** (#357) | gh-issues-sync Projects-v2 **Area** single-select under multi-value `area:*` labels — pick a deterministic, honest rule (anchor to the primary `area:*`, span recorded) so a multi-area item never silently "not set"; `--self-test` covers it | **▶ the only remaining item** (P3, deferrable) |

## Ownership / method
- **Owns:** `tools/github/**` (`idmap.tsv`, `pr-index.json`, the sync scripts + `*.py`). **Read-only /
  FLAG up:** `tools/github/issues.yaml` content semantics (statuses are reconciled by the L1/owning
  parents — `tul` maps ids↔issues, it does **not** re-status issues), `CHANGELOG.md`, `docs/Doc-Index.md`.
- Needs **GitHub read access** (the MCP `list_issues`/`issue_read`, or a maintainer token for
  `--use-api`); if neither is available in-session, land the deterministic-rule + number-only rows and
  **FLAG the db-id fetch** for a token-scoped follow-up (don't guess db-ids).
- **Done** = `idmap.tsv` reconciled (M-675) + the multi-area rule landed (M-676) on `main`; both issue
  bodies + statuses updated.

## Related (NOT in `tul` — L1-collision, runs in the `srf`/`run` serial track)
**M-677** (#358) — wire declared effects → `mycelium-interp::budget` ledger + per-effect budget syntax
(`!{retry(<=3)}`). Touches `crates/mycelium-l1` `elab.rs` + `mycelium-interp`, so it serializes with
the L1 work, not here.
