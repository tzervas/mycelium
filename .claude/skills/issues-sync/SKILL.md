---
name: issues-sync
description: >-
  Reconcile the repo's GitHub issues to the committed source of truth (tools/github/issues.yaml
  + idmap.tsv) the rate-limit-frugal, diff-based way: one bulk snapshot, only-the-deltas writes,
  dry-run by default. Hits the API only for issues that are missing (create) or drifted (update),
  skipping everything already in sync. A periodic DESKTOP operation (real API) — not for the
  cloud/web session.
when_to_use: >-
  Use periodically on a real host (desktop, gh-authenticated) after issues.yaml changes — to bring
  GitHub issues back in line with the manifest without a per-issue API fan-out. Run the dry-run to
  see the plan, then --apply the deltas. Not for labels/milestones/PRs/Project-board (that is the
  full engine, gh-sync-all.sh) and not for a repo-scoped cloud session (gh is unauthenticated there).
allowed-tools: Bash(python3:*), Bash(gh:*), Bash(just:*), Read, Grep
---

# /issues-sync — diff-based, rate-limit-frugal GitHub issue sync

Reconciles GitHub issues to `tools/github/issues.yaml` (source of truth) with **one bulk read and
only-the-deltas writes**. Dry-run by default; `--apply` required to mutate. Backed by
`tools/github/sync_issues.py` (see `tools/github/README.md` for the full model).

**This is a desktop / periodic operation.** It needs `gh` authenticated to the repo owner, so it
does **not** run in a repo-scoped cloud/web session. In the cloud, stop at the dry-run (offline
against the cached snapshot) and hand the `--apply` to a desktop run.

## The loop

```bash
# 1. Refresh the bulk snapshot and print the plan (ZERO mutations).
#    ~6 HTTP pages for 544 issues — NOT one call per issue.
python3 tools/github/sync_issues.py --refresh          # or: just issues-sync

# 2. Read the plan: N in-sync (skipped), M to update (which fields), K to create, J orphans.
#    Orphans (on GitHub, absent from issues.yaml) are REPORTED, never auto-deleted — add --verbose
#    to list them. Investigate an unexpected orphan/create before applying.

# 3. Apply ONLY the deltas, capped for safety:
python3 tools/github/sync_issues.py --apply --max-writes 25    # or: just issues-sync-apply

# 4. Confirm idempotence: a re-run should show everything in-sync (zero writes).
python3 tools/github/sync_issues.py --refresh
```

Add `--update-bodies` only when you intend to push issue bodies (off by default — GitHub bodies
accrue enactment notes a manifest would clobber). Add `--verbose` to list orphans.

## Reconcile orphans (so they stop being orphaned)

The plan may report **orphans** (GitHub issues not resolvable to an `issues.yaml` entry). Don't just
report them — **classify and reconcile** them (dry-run first; never deletes):

```bash
# 5. Classified plan: each orphan → non-task (allowlist) / superseded (dup) / adoptable / uncertain.
python3 tools/github/sync_issues.py --reconcile-orphans

# 6. Act per class (deliberate — class-2 comments + class-3 adoptions are persistent):
python3 tools/github/sync_issues.py --reconcile-orphans --apply --max-writes 25
```

- **non-task** → recorded in `pr-overrides.json` `orphans.allowlist` (an issue already in the engine's
  `overrides` block, like `#67`, is accounted automatically — no action).
- **superseded** → an idempotent link comment to the canonical issue (marker-guarded) + recorded in
  `orphans.superseded`. Never reopened, never deleted.
- **adoptable** → reverse-imported into `issues.yaml` as the next free `M-id` with best-effort fields
  and `_adopt_flags` for a human to complete; idmap row appended. **Eyeball each adoption.**
- **uncertain** → NO auto-action; a human decides (e.g. an OPEN issue that mirrors a tracked entry).

After a reconcile, a re-run reports **0 orphans** except any left **uncertain** (by design). See the
three-class table in `tools/github/README.md`.

## Safeguards (never-silent, G2)

- **Dry-run by default** — no `--apply`, no writes. The plan is printed in full.
- **Rate-limit gate** — `gh api rate_limit` is checked before any write; the run **stops** (loudly)
  if the remaining core budget is below the floor. `--max-writes N` is an independent hard cap.
- **Cache** — the snapshot is cached to a gitignored `.gh-snapshot.json`; repeated dry-runs cost
  zero API calls. A mutating `--apply` invalidates the cache so the next run re-grounds.
- **Append-only idmap** — new issues' numbers are appended to `idmap.tsv`, never rewritten.

## Verify the logic offline

```bash
python3 tools/github/sync_issues.py --self-test    # pure diff logic, no network
```

## Relationship to the full engine

`sync_issues.py` is the focused issue-only path. For a full reconcile (labels + milestones + issues +
PRs + Project v2 board) use the engine: `bash tools/github/gh-sync-all.sh --dry-run` then without
`--dry-run`. Both are diff-based; this skill is the frequent, issue-only, dry-run-first pass.
