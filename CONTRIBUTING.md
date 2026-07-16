# Contributing to Mycelium

Mycelium is in the **design phase**: the corpus in `docs/` is the product right now, and the discipline below is what keeps it auditable as it grows into an implementation. Please read `README.md` and `docs/Doc-Index.md` first.

---

## How the project is organized

- **`docs/Mycelium_Project_Foundation.md`** — the charter (vision, scope, requirements `FR/NFR/VR`, success `SC-*` and kill `KC-*` criteria, the ADR log 001–009, roadmap, risk register). The living source of truth for *what* and *why*.
- **`docs/rfcs/`** — RFCs: detailed, normative designs of specific subsystems. See `docs/rfcs/README.md` for the index + process.
- **`docs/adr/`** — Architecture Decision Records. See `docs/adr/README.md` for the index + process. (ADR-001…009 live in the Foundation §8; ADR-010+ are standalone files here.)
- **`docs/notes/`** — design notes / tradeoff studies (e.g., DN-01) that *feed* a decision without being normative themselves.
- **`research/`** — the evidence base: records of the two research passes, with source lists.

---

## The decision process (append-only)

Decisions are **never silently edited**. Each ADR/RFC carries a status that only moves forward:

```
Draft / Proposed / Preliminary  →  Accepted  →  Enacted  →  Superseded
                                       │
                                  (Resolved, for design notes)
```

- **`Enacted`** = an Accepted decision that is now **fully implemented/landed** — complete and stable, outside ongoing maintenance and future-development integration. It **must step through `Accepted`** first: a decision can never jump straight to `Enacted`. (Earlier docs used the compound `Accepted — Enacted`; the canonical spelling is the standalone `Enacted`.)
- To change an accepted decision, **supersede** it: write a new ADR/RFC (or revision) and link the old one forward. Don't rewrite history.
- Every normative claim **cites its grounding** using the established labels: survey labels (`G1–G11`, tensions `A–E`, recommendations `R1–R8`) and research labels (`T0.x / T1.x / T2.x`). If a claim can't be grounded, mark it as an open question, not a fact.
- Documents are revised in place only for *editorial* fixes and status transitions; the changelog at the bottom of each doc (and the top-level `CHANGELOG.md`) records what moved.

### When to write what
- **ADR** — a single architectural decision and its consequences (template in `docs/adr/README.md`).
- **RFC** — a full subsystem design that multiple decisions plug into (template in `docs/rfcs/README.md`).
- **Design note (DN)** — explores a tradeoff and *recommends*; it does not decide. When its recommendation is adopted, mark it `Resolved` and fold the result into the relevant RFC/ADR.

### Definition of Done + user stories (every decision and work item)
- **Definition of Done.** Every ADR/RFC/DN **and** every epic/issue carries an explicit, checkable **Definition of Done** — the explicit "what must be true to call this complete." A decision's gate (e.g. ADR-022 §5) or an issue's acceptance criteria *are* its DoD; never leave "done" implicit (this is a corollary of the transparency rule — you cannot claim something done without first stating what done means).
- **User stories.** Every epic and issue carries explicit **user stories** — `As a <role>, I want <capability>, so that <benefit>` — capturing realistic use cases and the concrete problems it must resolve, so work is grounded in real usage rather than abstraction. Use real roles: language user, library/phylum author, stdlib author, compiler engineer, tool author, AI co-author agent, maintainer, operator, downstream app developer.

### Licensing — MIT only
- The entire project is **MIT licensed**: no Apache-2.0, no dual-license, on any first-party artifact (root `LICENSE`, every crate's `license`, example/reference manifests). The `mycelium-proj` SPDX *parser* still recognizes other SPDX identifiers — accepting an identifier is parser correctness, not a project-license claim. The `deny.toml` **third-party dependency** license allow-list is a separate policy (ADR-022 §8 Q2), not a first-party license.

---

## The transparency rule (non-negotiable) *(reframed from "the honesty rule" by ADR-032; mechanism unchanged)*

This is the project's reason to exist; it applies to docs and, later, code.

- Accuracy/guarantee claims use the lattice **`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`**, assigned **per model and per operation** — never in aggregate (`VR-5`).
- A bound may be tagged **`Proven`** *only* if it cites a theorem whose side-conditions are checked. Otherwise it is **`Empirical`** (validated by trials) or **`Declared`** (user-asserted; always flagged in tooling).
- New results may *upgrade* a tag; absence of a proof keeps it weaker. Downgrading to keep a claim accurate is always acceptable; upgrading without a checked basis is not.
- **No black boxes.** Any feature that could introduce opaque behavior — especially "intelligent" automatic representation selection — must be reified, inspectable, and explainable (`EXPLAIN`). If you find yourself hiding a conversion or an approximation, that's a bug.

---

## Engineering principles (house style)

SRP · OCP · LSP · ISP · DIP · DRY · KISS · YAGNI · Law of Demeter · Separation of Concerns. **Composition over inheritance.** Keep the kernel small enough for one domain expert to audit (`KC-3`) — a large kernel is itself a black box.

---

## Development environment (implementation phase)

> No code has landed yet; this is the agreed toolchain so contributors set up consistently.

- **Rust** — the kernel + reference interpreter. **MSRV 1.96.1** (pinned; ADR-041). Format with `cargo fmt`; lint with `cargo clippy`. The AOT path uses **MLIR → LLVM** (confined to the performance path; the interpreter stays the trusted base).
  - **`unsafe` is permitted-but-warned, not forbidden (ADR-014).** It warns in `cargo build`/`cargo test` (the caution incentive) and the `just check` gate exempts only that lint (`-A unsafe_code`) so intentional, justified unsafe (FFI/JIT) passes — but every `unsafe` block must carry a `// SAFETY:` justification, and silence the dev warning *for release only* with `#[cfg_attr(not(debug_assertions), allow(unsafe_code))]`. Keep the trusted-base crates unsafe-free.
- **Python 3.13 / 3.14** — tooling, experiments, and the LLM-leverage harness. Managed with **UV**. Tests with **pytest**; coverage to **codecov**. Style: **PEP 8**, formatted with **`ruff format`** (Black-compatible) and linted with **ruff** — the actual tooling wired into `justfile`/`scripts/`/CI.
- **Devcontainers** are preferred for any environment not fully covered by UV, to keep setups reproducible.
- **VSA submodule** reuses the `balanced-ternary` crate and ports `torchhd`'s operation set as a reference.

### CI / quality gates (intended)
GitHub Actions running `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, and `uv run pytest` with coverage; Trunk for meta-linting. A change that adds an approximate operation must ship its bound + guarantee tag and a property test that exercises the bound (`SC-2`).

---

## Workflow

- **Kanban**, with tasks that are **dependency-ordered and priority-tagged**, grouped into related, non-blocking units of work, and phased so development flows in a logical, efficient order. (The decomposed task/issue set and phase plans live under `docs/planning/` — *forthcoming*.)
- We use a lightweight **persona loop**: a *Project Manager* assigns and sequences tasks and coordinates with an *Evaluator* for QA; the *Evaluator* gives critical feedback and sends failed work back for revision. Treat review as a first-class step, not an afterthought.
- **Kill criteria (`KC-1…KC-4`) are re-checked at every phase gate.** A gate that doesn't check them is hiding risk.

---

## Branches, commits, PRs

- Branch from `main`; keep branches focused on one task/issue.
- **Conventional Commits are enforced** via Commitizen (config: `.cz.toml`; check: `just cz-check`;
  pre-commit `commit-msg` hook). Subjects stay imperative and scoped (e.g.
  `docs(rfc-0003): tighten capacity-bound wording`) and reference the issue/task. Agent intermediate
  commits may use the allowed `wip` / `WIP` prefix (see `allowed_prefixes` in `.cz.toml`). In
  repo-scoped sessions that use `git commit --no-verify`, run the equivalent out-of-band
  (`just cz-check`, or `cz check --rev-range HEAD~1..HEAD`) before push — the bypass is only for
  unreachable external hooks, never a license to skip the message lint (see `CLAUDE.md` §Local
  checks).
- A PR should state which `FR/NFR/VR/SC` it advances (or which ADR/RFC it implements), and how it was verified. Editorial-only PRs say so.
- **No force pushes.** `git push --force` / `--force-with-lease` (and `+refs` push specs) are prohibited
  on every branch — and never on the protected `main`/`integration`/`dev`/`claude/head/*`. Correct a
  diverged or misaligned branch by bringing history *together*, never by rewriting published history: a
  rejected non-fast-forward push is a never-silent cue to reconcile, not to overwrite. For an
  **already-pushed** branch, **merge `main` into it** (pull-down), resolve, then a *plain* push — a merge
  only adds a commit, so the push fast-forwards the remote and no force is needed. **Do not rebase a
  pushed branch** (rebasing rewrites its published commits, so the plain push would be rejected and only a
  force could land it) — rebase is only for a **local-only, never-pushed** branch before its first push.
  When local work is in the way, the mechanism is **`git stash` → reconcile (merge `main` in) →
  `git stash pop` → deconflict** → plain push — it keeps your work and resolves divergence, where
  a force would silently discard the other side. If a published branch's own commits already landed on
  `main` (so it can never fast-forward), abandon it and branch a **fresh** one off current `main`,
  re-applying only the unlanded work. See `CLAUDE.md` §Commits & PRs for the agent-facing form of this
  rule.
- **Scoped PRs and workspace prep (DN-65).** Land work as **logical, closely-scoped PRs** — group by
  what they touch and aim for a soft **~1–2k-line delta** per PR (a rule of thumb for *where to
  split*, not a hard limit; keep a cohesive change together rather than fragmenting it). Do the work
  at whatever scale it takes — a large effort still **lands as a sequence/fan of small, reviewable
  PRs**, each closely related to its change, so it is easy to review and integrate. Before starting a
  unit, **update off the latest tip** and **install the toolchain your change needs** (Rust →
  `just setup`; Python → `uv sync`; docs → the markdown/`doc_refs` checks; proofs → `z3`/LH/Lean) so
  you are not surprised mid-flight. When PRs share files, land them in order and pull the merged base
  down before the next. Full policy plus the change-kind→toolchain map: `docs/notes/DN-65-…md`.
- **Concurrent, tier-scoped, agent-reviewed development (the optimal pattern).** Work flows
  `feature/leaf → dev → integration → main` — in this pattern each hop is PR-reviewed (tightening
  the default where branches below `dev` merge freely; see `CLAUDE.md` §Commits & PRs). **Testing tightens up the tiers:** a **leaf
  runs only the change-scoped checks for what it touches** (deep on those components, not the whole
  workspace), `dev` is the working tier, and **`integration` is where the gates tighten and the polish
  concentrates** — APIs regenerated, docs finalized, issues/epics closed out (so leaves FLAG those up
  rather than editing `CHANGELOG`/indices themselves). Each concurrent agent works in its **own isolated
  `git worktree`**; each PR gets a dedicated **`/pr-review` agent** that posts findings as PR comments,
  patches them, replies with the resolution, updates the PR description, and merges up the tree — with
  the **merge to `main` the terminal (maintainer) checkpoint**. Operationalized as the **`/wave`**,
  **`/pr-land`**, and **`/worktree-guard`** skills (`.claude/skills/`). Full agent-facing form:
  `CLAUDE.md` §Concurrent-PR development and §Autonomous PR workflow.

### PR Tracking footer (required on every PR)

Every PR description ends with a **Tracking** block so the issue graph, epic lineage, and close
intent stay explicit (never-silent — G2). Resolve `M-####` / `E#-#` to GitHub issue numbers via
`tools/github/idmap.tsv` — **never invent** `#N` values.

```markdown
## Tracking
- Task: M-#### (and/or E#-#)
- Epic: E#-# (if any)
- Refs: #N [, #M …]     # GitHub issue numbers from tools/github/idmap.tsv
- Closes: #N            # ONLY when PR base is main AND DoD fully met
```

- **Task / Epic** carry the project ids (`M-####`, `E#-#`) used in `tools/github/issues.yaml`.
- **Refs** lists the GitHub issue numbers the PR advances (looked up from `idmap.tsv`). Use
  `Refs` or `Part of` on every non-`main` PR.
- **Closes** is reserved for completed issues on a **main-bound** PR whose Definition of Done is
  fully met (see auto-close rules below). Omit the line entirely when it does not apply — do not
  put a placeholder `Closes:`.

### GitHub auto-close rules

GitHub's closing keywords (`Closes` / `Fixes` / `Resolves` plus `#N`) fire **only on merge into the
default branch `main`**. They do nothing useful on other bases and confuse readers who expect a
close that never happens.

| PR hop | Closing keywords | Use instead |
|---|---|---|
| leaf → `dev` | **Never** `Closes` / `Fixes` / `Resolves` | `Refs: #N` or `Part of #N` |
| `dev` → `integration` | **Never** `Closes` / `Fixes` / `Resolves` | `Refs: #N` or `Part of #N` |
| `integration` → `main` (or any release PR into `main`) | `Closes #N` for each completed issue | — |

A leaf or staging-tier PR that puts `Closes #N` in its body will **not** close the issue on merge
(the base is not `main`) and will mislead reviewers. Keep close intent on the main-bound PR only,
and only when that issue's DoD is fully met.

### Epic release-gate pattern

Each open epic carries a **terminal child** issue of the form:

`E#-# — release-gate: promote to main + cut release`

- The epic **stays open** until that gate lands on `main` and is closed (via `Closes #N` on the
  main-bound PR — never earlier).
- **Gate Definition of Done:** every other child of the epic is `done`; the epic's work is
  promoted to `main`; and the release notes / version act follow **ADR-018** (per-crate `0.x`
  SemVer; no accidental `1.0.0`) and **ADR-038** (public release is a sub-`1.0.0` usability cut;
  `1.0.0` is a separate, manual gate act — never auto-bumped). Release-gate issues themselves are
  minted by the orchestrator when an epic opens; leaves do not invent them.

---

## Provenance

Everything in `docs/` traces to the research recorded in `research/`.
