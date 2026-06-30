---
name: wave
description: >-
  Run a concurrent wave of tightly-scoped work the safe, fast way: partition into disjoint items, spawn
  ONE isolated git worktree per agent, each leaf runs only change-scoped checks and updates only its
  own issue, each result is reviewed and merged up the tree via /pr-land, and the integration tier does
  the whole-batch polish/close-out. The umbrella for the concurrent-PR development pattern — maximum
  velocity with maximum accuracy, parameterized by the work-items list.
when_to_use: >-
  Use when an orchestrator decomposes work into several independent, tightly-scoped items (a crate /
  directory / doc-cluster / kickoff each) to run in parallel. It ties together worktree isolation,
  tier-scoped testing, per-PR agent review, and integration-tier reconciliation so a large parallel
  wave stays collision-free, honest, and fast.
allowed-tools: Bash(scripts/checks/worktree-guard.sh:*), Bash(scripts/checks/branch-guard.sh:*), Bash(git worktree:*), Bash(git fetch:*), Bash(git rev-parse:*)
---

# wave

The standing pattern for highly-concurrent, tightly-scoped work (CLAUDE.md §Concurrent-PR development).
It maximizes velocity **and** accuracy while minimizing token/context churn. The orchestrator drives
it; the leaves stay tightly scoped and isolated; review and reconciliation happen at the right tiers.

## Parameters

- `ITEMS` — the disjoint work items (one crate / directory / doc-cluster / issue each). Partition by
  **file ownership**, not just by task, so the merges are conflict-free by construction.
- `MODE` — the swarm model assignment (CLAUDE.md §Swarm modes; default Sonnet). The orchestrator
  passes the resolved model explicitly to every spawn.
- `BASE` — the tier each item's PR lands on (default `dev`).

## The loop

1. **Pre-flight (mandatory).** `git fetch`; confirm the orchestrator's main worktree is a clean pointer
   on the intended base (`/worktree-guard`, `/branch-guard --expect <base>`); mint any new issue IDs
   only after grepping `issues.yaml` for the free slot (mitigation #1); push the base before spawning
   worktree children (mitigation #5).
2. **Partition + isolate.** One **isolated `git worktree` per concurrent agent** —
   `isolation:"worktree"` on every spawn (mitigation #11; **never** the shared main tree). Each leaf
   runs `/worktree-guard --leaf` before its first git write. Each agent edits **only its disjoint
   tree** and **its own issue entry**; everything shared is read-only / FLAG-up.
3. **Change-scoped at the leaf.** Each leaf runs **only the checks for what it touches + its direct
   blast radius** — `cargo fmt`/`clippy -D warnings`/`test -p <crate>` + the targeted
   differential/conformance — **not** the full-workspace `just check`. It records in its own issue
   exactly what it did; commits in small batches and pushes (durability #5/#9; separate commit/push,
   no protected names in messages — mitigation #12). It **reports back** branch + SHA + FLAGs; it does
   not finalize `CHANGELOG`/indices or close out issues (those are the integration tier's).
4. **Per-PR review, merge up the tree.** For each item's PR, run **/pr-land** (an isolated Sonnet
   `/pr-review` agent posts findings as PR comments → patches → replies → updates the description →
   merges into `BASE` once clean + green). When PRs share a file, land them **sequentially**, pulling
   the freshly-merged base down before the next (mitigation #6).
5. **Integration-tier close-out.** Once the wave's items are on `dev`, the **integration** promotion is
   where the gates **tighten** and the polish concentrates: the full `just check`/durability tier, the
   **final wiring-in**, **APIs regenerated** (`docs/api-index/`, baselines), **all documentation
   finalized** (`CHANGELOG`, `Doc-Index`, spec cross-refs), and **issues + epics closed out** (`→
   done`) — the one place the whole-batch reconciliation happens (the orchestrator owns it).
6. **`main` is the terminal checkpoint.** `integration → main` is a curated **squash** PR held for the
   maintainer (`/land` / `/wave-land`) — never an agent auto-merge.

## Why it works

Disjoint ownership + isolated worktrees ⇒ the merges are conflict-free by construction. Change-scoped
leaf testing + integration-tier polish ⇒ fast feedback below, full rigor where it counts. Per-PR agent
review ⇒ every change is audited against the house rules before it lands, honestly (house rule #4).
Transparency/append-only/never-silent (G2/VR-5) survive the parallelism: specs move to "implemented,
pending ratification", never silently to Accepted/Enacted; tags stay at their supportable strength.

Composes with **/pr-land** (per-item review+merge), **/worktree-guard** + **/branch-guard** (the
safeguards), **/kickoff** (a wave item may be a stowed kickoff), and **/dev-workflow** (the leaf's
implementation discipline). Full rationale: CLAUDE.md §Concurrent-PR development, §Swarm development,
§Autonomous PR workflow, and mitigations #11/#12.
