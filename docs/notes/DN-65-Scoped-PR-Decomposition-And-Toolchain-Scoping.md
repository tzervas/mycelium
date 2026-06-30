# Design Note DN-65 — Scoped-PR Decomposition & Per-PR Toolchain Scoping (workspace prep + workflow)

| Field | Value |
|---|---|
| **Note** | DN-65 |
| **Status** | **Accepted** (2026-06-29 — maintainer-directed workflow policy; process decision, mirrors the workflow-DN precedent DN-20). Enacted-as-convention: CLAUDE.md carries the enforceable distillation, CONTRIBUTING.md the human-facing form, the skills the in-loop integration. The supporting *automation* (a scoped `just` profile) is tracked separately as **M-848** and is not a precondition for the policy. |
| **Decides** | How large work is **decomposed into scoped pull requests**, how each PR is **reviewed by an agent**, and how each agent's **toolchain is scoped and pre-installed** (workspace prep) so work lands as logical, manageable, review-sized units regardless of total delta. |
| **Feeds** | CLAUDE.md (Commits & PRs · Swarm · Local checks); CONTRIBUTING.md (Branches, commits, PRs); the skills `/dev-workflow`, `/pr-review`, `/land`, `/wave-land`, `/kickoff`; the `just setup` tool-install recipe; DN-20 (tiered testing / change-scoping — the test-tier twin of this PR-scoping rule) |
| **Date** | June 29, 2026 |

> **Posture (transparency rule / G2).** This is a **process decision**, not a language/kernel
> design change — it enacts no code and touches no guarantee tag. It is `Accepted` as a workflow
> convention (the maintainer directed it 2026-06-29). The quantitative target below is a **soft
> rule of thumb**, never a hard gate; the binding rule is *logical, closely-scoped PRs*, and the
> size figure only guides where to split. Append-only: this note may gain dated sections; it must
> not be rewritten.

---

## §1 Problem

A swarm (or a single long session) can produce a very large delta — the DN-64 wave landed ~7,350
lines in one PR (#770), and a future wave could be 50k+. A monolithic PR of that size is hard to
review honestly, hard to integrate, and couples unrelated changes so a problem in one part blocks
all the rest. The work *should* be done at whatever scale it takes; what reaches **review and
landing** should be **split into logical, closely-scoped units**.

A second, recurring friction is **toolchain drift**: an agent starts work without the specific
tools its change needs (a `cargo-nextest`, a `ruff`, a `markdownlint`, a Lean toolchain), or off a
stale base, and hits avoidable failures mid-flight. The fix is to **scope and pre-install the
toolchain, and sync off the tip, before work begins** — workspace prep, captured up front.

## §2 Decision

### §2.1 Scoped-PR decomposition

- **Land work as logical, closely-scoped PRs**, grouped by what they touch (a subsystem, a
  crate/phylum, a doc cluster, one feature). Each PR is **isolated and self-contained** — its diff
  is closely related to the work it names, so it is easy to integrate and easy to review.
- **Soft size target ≈ 1,000–2,000 lines of delta per PR** (rule of thumb, not a hard cap). Do
  **not** fragment into dozens of trivial PRs — cohesion wins over a line count. A change that is
  one logical unit but 2,500 lines stays one PR; a 5,000-line change spanning four independent
  subsystems becomes ~four PRs.
- **Total delta is unbounded; PR size is bounded.** A wave may be 50k lines across many agents — it
  still lands as a sequence/fan of scoped PRs, each cohesive. (This PR — the DN-64 enforcement
  work — was itself split into PR-A *RFC-0038 model* and PR-B *this policy* to demonstrate the rule.)
- **Decompose by the existing collision/ownership boundaries** (the swarm file-ownership rule):
  disjoint directories / crates / doc clusters are natural PR seams; shared files (CHANGELOG,
  Doc-Index, issues.yaml) are reconciled per-PR by the integrating parent.
- **Order matters when PRs share files.** Land sequentially and **pull the freshly-merged base down
  before the next PR** (the pull-down-before-merge-up rule), so each PR's diff is against current
  `dev`/`main` and shared-file edits append cleanly rather than conflict.

### §2.2 Per-PR agent review

Each scoped PR gets its **own `/pr-review` pass** — runnable by an agent, using the shared rubric
(`.claude/skills/_shared/review-rubric.md`) and the repo toolchain. Smaller, cohesive diffs make
the review tractable and the rubric's per-finding `file:line → what → why → fix` precise. Surface
every Critical/High; fix what is clear, flag what is ambiguous/architectural (`AskUserQuestion`),
land green. (The review may itself be one agent or a small fan-out by dimension for a large diff.)

### §2.3 Workspace prep — sync off the tip, then scope + pre-install the toolchain

Before an agent starts a scoped unit:

1. **Sync off the latest base/tip.** `git fetch origin`; branch from / fast-forward to the current
   `dev` (or the relevant head) so every agent shares the same head and their tips match. Never
   start from a stale base (the stale-base / branch-ref-drift mitigations).
2. **Scope the toolchain to the work, and pre-install it.** Determine which components the change
   needs **beyond the default**, and run the exact setup up front, so there are no mid-flight
   toolchain surprises. The change-kind → tools mapping:

   | Change kind | Tools to ensure (beyond defaults) | Setup |
   |---|---|---|
   | Rust crate (`crates/**`) | `cargo fmt`, `clippy`, `cargo-nextest` (test), `cargo-public-api` (api gate), `cargo-mutants`/`cargo-fuzz` (durability) | `just setup` |
   | Python (`experiments/**`, `tools/**`) | `uv`, `ruff`, `pytest` (+ `numpy`/`torch` per group) | `uv sync --group <g>` |
   | Docs (`docs/**`, `*.md`) | `markdownlint`(-cli2), `doc_refs_check.py`, `links.sh`/`structured.sh` | `just setup` / `scripts/checks/*` |
   | Proof artifacts (`proofs/**`) | `z3` (SMT), GHC + cabal + LiquidHaskell (LH), `elan`+Lean (Lean) | per-`proofs/*/README.md` |

   The standing entrypoint is **`just setup`** (best-effort install of the check tools). The scoped
   refinement — "for *this* PR's kind, these specific components, with these args" — is the
   automation tracked as **M-848** (`just setup-scoped <profile>` / a per-kind probe): it emits the
   precise commands to set up an agent's environment exactly for the work it owns. Until M-848
   lands, the mapping above is the manual checklist.
3. **Run the change-scoped gates** (DN-20 tiers) — `just check` routes the same recipes locally and
   in CI; in a repo-scoped remote session use the sanctioned out-of-band gates with `--no-verify`.

## §3 How it composes with the swarm (CLAUDE.md)

This is the **PR-landing twin** of two existing rules: DN-20 *change-scoping* (which bounds the
*test surface* per commit) and the swarm *file-ownership partition* (which bounds the *collision
surface* per agent). DN-65 bounds the *review/landing surface* per PR. Together: an agent owns a
disjoint scope, syncs off the tip with its scoped toolchain pre-installed, does the work at whatever
size it takes, and the parent lands it as one (or a few) logical, ~1–2k-line, agent-reviewed PRs —
pulling the base down between PRs that share files.

## §4 Definition of Done

- CLAUDE.md (Commits & PRs) carries the enforceable distillation: decompose into logical ~1–2k-line
  scoped PRs · per-PR `/pr-review` · sync-off-tip + scoped-toolchain workspace prep.
- CONTRIBUTING.md (Branches, commits, PRs) carries the human-facing form.
- `/dev-workflow` (workspace prep + decomposition), `/land` (scoped PRs + pull-down), and the
  `/kickoff` README (per-agent workspace prep) reference the policy so it engages in-loop;
  `/pr-review` needs no change — it already reviews per-diff, which *is* per-scoped-PR under this policy.
- The change-kind → toolchain mapping is recorded (above); the scoped-setup automation is tracked
  as M-848.

## Changelog

| Date | Change |
|---|---|
| 2026-06-29 | Accepted — scoped-PR decomposition (logical, ~1–2k-line units), per-PR agent `/pr-review`, and workspace prep (sync-off-tip + scoped toolchain pre-install). Maintainer-directed. Automation tracked as M-848. |
