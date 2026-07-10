# `tools/github/` — GitHub project-management sync

The repo's GitHub issues are a *projection* of a committed, in-repo source of truth. This directory
holds the source-of-truth manifests and the tooling that reconciles GitHub to them.

## Source-of-truth model

| File | Role |
|---|---|
| `issues.yaml` | **The** issue corpus (544 tasks). Each entry: `id` (`M-###`/`E##-#`), `title`, `body`, `labels` (incl. `phase:*`/`type:*`/`area:*`/`priority:*`/`status:*`), `milestone`, `depends_on`, `doc_refs`, and enactment metadata (`landed_pr`, `landed_date`, …). |
| `idmap.tsv` | The `task_id → issue_number → issue_db_id` map (TAB-separated, append-only). Resolves a task to its GitHub issue so a **title rename updates** the issue instead of duplicating it. |
| `labels.json` / `milestones.json` / `project.json` / `conventions.json` | Label colours, milestones, the Project v2 board, and the Conventional-Commit → label grammar. |

**Status is a label, not GitHub open/closed.** Every entry carries exactly one `status:*` label
(`status:todo|in-progress|blocked|needs-design|done`). GitHub OPEN/CLOSED state is **never** inferred
from it — an issue's state changes only if an entry declares an explicit `state: open|closed`
(none currently do). This is deliberate: a `done` task's issue may stay open for discussion.

## Two tools, one diff-based model

Both reconcile GitHub to the manifests with a **single bulk read + only-the-deltas writes** — never
a per-issue fan-out. They share the diff semantics (idmap-number-first matching; labels converge as a
full set; explicit-`state:`-only state changes; normalized body compare).

### `sync_issues.py` — the frugal, cached, issue-only path (periodic use)

The focused tool for "keep the issues in sync" on a real host (desktop) periodically. **Dry-run by
default** — it never writes without `--apply`.

```bash
# 1. Preview the plan (ZERO mutations; uses the cached snapshot if fresh):
python tools/github/sync_issues.py

# 2. Force a fresh bulk snapshot first (one `gh --paginate` read, ~6 HTTP pages for 544 issues):
python tools/github/sync_issues.py --refresh

# 3. Apply ONLY the deltas (create missing, edit drifted fields), capped for safety:
python tools/github/sync_issues.py --apply --max-writes 25

# Offline unit check of the pure diff logic (no network):
python tools/github/sync_issues.py --self-test
```

How it stays rate-limit-frugal:

- **One bulk snapshot**, cached to `.gh-snapshot.json` (gitignored). Repeated dry-runs and offline
  diff development re-fetch **nothing**; `--refresh` re-grounds it, `--max-cache-age` bounds staleness.
- **Local field-by-field diff** classifies every entry **in-sync** (skip — zero API calls),
  **needs-update** (only the drifted fields), **missing** (create), or **orphan** (on GitHub, absent
  from `issues.yaml` — **reported, never auto-deleted**).
- **Apply touches only deltas.** In-sync issues cost nothing. A tally reports writes made vs. the
  naive per-issue baseline (≈544).
- **Safeguards:** dry-run default, `--apply` required to write, `gh api rate_limit` checked before
  any write (stops below a floor, never-silent), `--max-writes N` hard cap, cache invalidated after a
  mutating run so the next run re-grounds.

Honest tags: the diff is **Exact** over the cached snapshot; the *snapshot* is **Empirical** (only as
fresh as the last fetch); orphan handling is **Declared** (report, never delete).

#### Orphan reconciliation (`--reconcile-orphans`)

An **orphan** is a GitHub issue not resolvable to any `issues.yaml` entry. Reporting is not enough —
`--reconcile-orphans` **classifies** each orphan and (under `--apply`) makes it stop being one.
Dry-run by default; **never deletes**; each class has a distinct, never-silent action:

```bash
python tools/github/sync_issues.py --reconcile-orphans            # classified plan (no writes)
python tools/github/sync_issues.py --reconcile-orphans --apply    # act per class (deliberate)
```

| Class | What it is | Reconcile action |
|---|---|---|
| **1 · non-task** | a **closed** issue with no duplicate signal — a tracked RFC/discussion issue (e.g. `#67` RFC-0006) | record it in `pr-overrides.json` → `orphans.allowlist` so it is *accounted for* and no longer flagged. (An issue already in the engine's `overrides` block — like `#67` — is accounted automatically.) |
| **2 · superseded** | a **closed** issue whose title's task-id (`M-###`) maps via `idmap.tsv` to a **different**, canonical issue — a duplicate | confirm it is closed (never reopen); post an **idempotent** link comment to the canonical issue (marker-guarded, never re-posted); record it in `orphans.superseded`. **Never deletes.** |
| **3 · adoptable** | an **open** issue with no duplicate signal — a genuine issue opened directly on GitHub | **reverse-import** into the source of truth: mint the next free `M-id` (collision-checked), append a best-effort `issues.yaml` entry (title/body/labels/state mapped; uninferable phase/type/area/priority surfaced as `_adopt_flags`), append the `idmap.tsv` row. **FLAGged for human review.** |
| **· uncertain** | a conflicting signal — an **open** task-id duplicate, or a strong title match to a tracked entry | **no auto-action**; flagged for a human to decide (G2/VR-5). |

The reconcile ledger lives in the **same `pr-overrides.json`** the engine already uses — under a new
top-level `orphans` block that the engine ignores (its loader reads only `overrides`), so extending
the file is side-effect-free there. Class-2 comments and class-3 adoptions are **persistent GitHub /
source mutations** — run `--apply` deliberately. After a reconcile, a re-run reports **0 orphans**
except any left in the **uncertain** bucket (which by design require a human decision).

### `gh-issues-sync.py` — the full cross-platform reconcile engine

The superset: labels + milestones + **issues** + PRs + the Project v2 board, plus manifest validation
and least-privilege auth preflight. Driven by `gh-sync-all.sh` (Linux/macOS) / `gh-sync-all.ps1`
(Windows). Its issue level **already** does a bulk `snapshot_issues()` + field diff — it is **not** a
per-issue fan-out. Use it for a full-suite reconcile; use `sync_issues.py` for the frequent,
issue-only, dry-run-first pass. See `RECONCILE.md` for the engine's full contract.

`sync_issues.py` adds, over the engine's issue level: a **persisted snapshot cache**, a
**dry-run-first default**, a **`--max-writes` cap**, **classified orphan reconciliation**
(`--reconcile-orphans`), and an **API-call tally**.

## Deprecated / historical

- **`mcp-bootstrap.md`** — the original model-executed bootstrap over the GitHub **MCP** server. It
  touches the API **per issue** (`list_issues`/`issue_write` each). Superseded for routine sync by the
  two diff-based tools above; retained for the initial-provisioning narrative and the sub-issue /
  dependency-linking pass that still needs MCP/GraphQL.
- **`gh-bootstrap-local.sh`** — labels + milestones only, via `gh`. Still valid; the engine's `--all`
  supersedes it cross-platform.

## Validation

```bash
python3 -c "import yaml; yaml.safe_load(open('tools/github/issues.yaml')); print('OK')"  # parses
python3 tools/github/doc_refs_check.py                                                    # doc_refs grammar
python3 tools/github/sync_issues.py --self-test                                           # diff logic
```
