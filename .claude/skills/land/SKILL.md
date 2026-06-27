---
name: land
description: >-
  Land a reviewed PR or branch onto main: self-review with the /pr-review lens,
  handle every CI / bot comment, run just check green, curated squash-merge to
  main (squash-only; internal swarm merges stay octopus/--no-ff), then
  branch + worktree cleanup.
when_to_use: >-
  Use when landing a PR or branch onto main — either as a final single-PR
  landing, or as the last step of a swarm wave (orch→main). Not for
  leaf→epic or epic→orch merges (those stay octopus/--no-ff; /land is only
  for the squash-onto-main step).
allowed-tools: Bash(git:*), Bash(just:*), Bash(gh:*), Read, Grep, Glob
---

# land

The working loop for safely landing work on `main`. This operationalises
`CLAUDE.md §Autonomous PR workflow` and `§Commits & PRs`; those sections win
on any conflict. This skill is **advisory discipline, not a gate-bypass** —
a maintainer override still wins; if asked to wait, wait.

## When to use

- Landing a single PR → `main`.
- The final step of a swarm wave: orchestrator branch → `main` after all epics
  are merged up and `just check` is green.

Do **not** use for leaf → epic or epic → orch merges; those are octopus /
`--no-ff` (lineage-preserving) and governed by the swarm discipline in
`CLAUDE.md §Swarm development`.

## The loop

### 1. Self-review the diff

Run `/pr-review` on the full diff of the branch against `main` before touching
anything else. Check:

- **Honesty rule** — every guarantee/bound carries a per-model/per-op tag
  (`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`). `Proven` is valid only if a
  theorem's side-conditions are *checked*. Flag any upgrade lacking a checked
  basis (VR-5).
- **Append-only decisions** — ADR/RFC/DN status moves forward only
  (`Draft → Accepted → Superseded`); no silent rewrites of Accepted content.
- **Grounding** — normative statements cite `G1–G11 / A–E / R1–R8 / T0.x–T2.x`
  or are marked open questions. Ungrounded assertions → High.
- **Never-silent G2** — swaps/conversions are not silent; out-of-range returns
  `Option`/error, never a quiet value.
- **Hallucination / consistency pass** — verify cross-references, IDs
  (FR/NFR/VR/SC/KC, RFC §), file paths, and API names against actual source.

**Stop on any Critical or High you cannot resolve.** Fix it or flag it up — never
merge past it.

```
git diff origin/main...HEAD   # or: gh pr diff <PR#>
# then invoke /pr-review
```

### 2. Handle every CI / bot review comment

For each review comment (Copilot suggestion, CI failure, human note):

- **Fix** it if you are confident and the change is small.
- **Defer honestly** if the fix would be fragile or large — write an explicit
  refusal with a clear message and a spec-§ note. Never ship fragile or
  incorrect output just to close a comment (G2 / VR-5).
- **Ask** (`AskUserQuestion`) if the comment is ambiguous or architecturally
  significant.

Reply once per comment, frugally. The diff is the permanent record.

### 3. Green — full `just check`

```
just check   # identical to CI (just ci); must be green or skips explained
```

After `just check` passes, regenerate orchestrator-owned artifacts that may
have drifted:

- `just docs-index` → commits `docs/api-index/` delta if any public API changed.
- `just api-baseline` → commits `docs/spec/api/` delta if any API baseline changed.
- Update `CHANGELOG.md` (use `/changelog`) and any `issues.yaml` status fields.

Never hand off a red `just check` without a written explanation of the skip.

### 4. Pull-down before merge-up

Before squashing onto `main`, ensure the branch already contains the latest
`main` tip — so the squash is conflict-free and the diff is exactly the net
change:

```
git fetch origin main
git merge origin/main   # or rebase; resolve any conflict here, in context
just check              # re-verify after the pull-down
```

### 5. Curated squash-merge to `main`

Squash-only into `main`. Internal swarm merges (leaf → epic → orch) stay
octopus / `--no-ff` to preserve lineage; **only the final landing on `main`
squashes**. The landing is a **GitHub PR squash-merge**, not a local `git push`/`merge` to `main` —
the branch-guard (PreToolUse + pre-push hooks; `/branch-guard`) **blocks** any local commit/merge/push
to `main`/`integration`/`dev`/`claude/head/*`, which is exactly correct: protected branches are PR-only.

Write a clear, self-contained squash commit — subject + body describing the
*net* change. Never let the auto-concatenated WIP / `wip(batch …)` / fixup /
merge trail stand as the squash message. The commit left on `main` is the
permanent record.

Via GitHub PR (preferred — triggers CI record):

```
gh pr merge <PR#> --squash --subject "<conventional subject>" --body "<body>"
```

Or locally if there is no open PR:

```
git checkout main
git merge --squash <branch>
git commit -m "<conventional subject>

<body>

Co-Authored-By: ..."
git push origin main
```

Commit subject convention: `<type>(<scope>): <imperative description>` —
reference the issue/task (`M-xxx`/`E*`). State which `FR/NFR/VR/SC` the PR
advances (or which ADR/RFC it implements) and how it was verified; editorial-
only landings say so.

### 6. Post-merge cleanup

Delete the merged branch and any associated worktrees; sync local `main`:

```
git push origin --delete <branch>      # delete remote branch
git branch -d <branch>                 # delete local branch (safe — squash already landed)
git worktree remove <worktree-path>    # if a worktree was used
git worktree prune                     # remove stale worktree metadata
git fetch --prune origin               # sync refs
git log origin/main --oneline -5       # confirm linear squash history
```

Confirm the squash commit appears as the tip of `main` with the curated
subject, not a merge commit and not a WIP trail.
