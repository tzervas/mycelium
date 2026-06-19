# CLAUDE.md — Mycelium

> Language is in so much an encoding, code itself in an abstract.
>
> To engage in such is endearing, as complexity and elegance is love,
>
> And beauty poured forth from the mind and soul.

Operating guide for Claude Code (and other agents) working in this repo. Authoritative human
docs: @README.md, @CONTRIBUTING.md, @docs/Doc-Index.md. This file is the short, enforceable
distillation; `CONTRIBUTING.md` wins on any conflict.

## What this repo is
Mycelium is a unified value-semantics substrate (binary/ternary/dense/VSA) with **certified,
never-silent** representation swaps and **honest, per-operation guarantees**. It is in the
**design phase**: the corpus in `docs/` is the product right now; code lands per the phase plan
(see `tools/github/issues.yaml` and the `M-xxx`/`E*` task ids).

## Non-negotiable house rules
1. **The honesty rule.** Every accuracy/guarantee claim is tagged per-model/per-op on the
   lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`. `Proven` is allowed **only** with a theorem
   whose side-conditions are *checked*; otherwise `Empirical` (trials) or `Declared` (asserted,
   always flagged). Downgrade to stay honest; never upgrade without a checked basis (VR-5).
2. **No black boxes.** Selections/conversions/approximations are reified, inspectable, and
   `EXPLAIN`-able. A swap is **never silent**; out-of-range is an explicit `Option`/error.
3. **Append-only decisions.** ADR/RFC/DN status moves forward only
   (`Draft/Proposed → Accepted → Superseded`; notes `→ Resolved`). To change an Accepted
   decision, **supersede** it — don't rewrite history.
4. **Ground every claim.** Normative statements cite their basis (`G1–G11`, tensions `A–E`,
   `R1–R8`, `T0.x–T2.x`) or are marked open questions. No ungrounded "facts".
5. **Small, auditable kernel** (KC-3). SOLID · DRY · KISS · YAGNI · Law of Demeter · SoC;
   composition over inheritance.

## Toolchain
- **Rust** kernel + reference interpreter — MSRV **1.92** pinned; `cargo fmt`, `cargo clippy
  -D warnings`, `cargo test`. MLIR→LLVM is the perf-path AOT; the interpreter is the trusted base.
- **Python 3.13/3.14** via **UV** — `pytest` + codecov, **ruff** + **`ruff format`** (Black-compatible).
- Don't silently bump committed version pins (MSRV, Python) — that's a decision (ADR), not a
  build detail, even if your local toolchain is newer.

## Local checks — run before every commit (local↔CI parity)
One source of truth (`just`); pre-commit and CI route through the same recipes.

```
just            # list recipes
just setup      # best-effort install of the check tools (uv tool / npx / pip)
just check      # run the FULL suite — identical to what CI runs (`just ci`)
just fmt        # auto-format (rust + python)
just hooks      # install the pre-commit hooks
```

Checks **skip gracefully** when a tool or language isn't present yet (most code doesn't exist
yet). Never hand off a red `just check` without explaining the skip.

## Remote CI policy
**No automatic remote CI.** `.github/workflows/checks.yml` is **manual-dispatch only**
(`workflow_dispatch`) and **advisory** (non-blocking) — to keep parity with local while avoiding
surprise Actions cost. It runs the same `just ci`. Do not add `on: push` / `on: pull_request`
auto-triggers without an explicit decision.

## Commits & PRs
- Conventional, imperative subjects referencing the issue/task
  (`docs(rfc-0003): tighten capacity-bound wording`, `feat(swap): …`).
- A PR states which `FR/NFR/VR/SC` it advances (or which ADR/RFC it implements) and **how it was
  verified**. Editorial-only PRs say so.
- Branch from `main`, one task per branch.

## Swarm development — octopus-merge pattern (parallel agents, zero collision)
When a wave decomposes into several **tightly-scoped, independent** tasks (e.g. one stdlib
module / capability crate per task), run them as a **swarm of agents merged with a single
octopus merge**, with **one orchestrator owning every shared file**. This keeps parallel work
collision-free *by construction* rather than by after-the-fact conflict resolution.

The discipline:
1. **Partition by file ownership, not just by task.** Each agent owns a **disjoint directory**
   (prefer **one crate per task** — `crates/mycelium-std-<module>` — so the only shared file is
   the workspace `Cargo.toml`). An agent edits **nothing** outside its directory.
2. **Orchestrator owns all common/shared files** — the wave's collision surface: workspace
   `Cargo.toml`, `CHANGELOG.md`, `docs/Doc-Index.md`, `tools/github/issues.yaml` + `idmap.tsv`,
   `docs/planning/phase-*.md`, shared spec indices, per-doc changelog footers. Agents never touch
   these; the orchestrator reconciles them once, after merge.
3. **Scaffold first, then fan out.** The orchestrator creates each task's skeleton (crate
   manifest with deps pre-filled, stub `lib.rs`), registers it in the workspace, and **commits +
   pushes the scaffold** so every agent branches from a *buildable* base and never needs to edit
   shared wiring.
4. **One agent per task, isolated worktree.** Launch each on its own git worktree branch
   (`isolation: "worktree"`), in parallel. Each follows `/dev-workflow`, ships its honest
   guarantee tags + tests, runs `cargo fmt`/`clippy -D warnings`/`test -p <crate>` green, commits
   to its worktree branch (does **not** push), and reports its branch + SHA + any FLAGs. An agent
   that hits ambiguity **flags it**, it does not guess (G2/VR-5 apply to agents too).
5. **Octopus-merge back into the working branch.** `git merge --no-ff <b1> <b2> … <bN>`. Disjoint
   directories + a pre-finalized workspace manifest ⇒ the N-way merge is conflict-free. The
   orchestrator then makes the single integrating edits to the shared files (step 2), runs the
   **full** `just check`, fixes integration, and commits + pushes.
6. **Honesty survives the swarm.** The orchestrator reviews each merged crate against the house
   rules before the wave's changelog entry; tags stay at the honestly-supportable strength, and a
   spec moves to "implemented (Rust-first), pending ratification", never silently to `Accepted`.

## Fractal Swarm Development System
The **recursive generalization** of the octopus-merge pattern above. That pattern is single-level
(one orchestrator, N leaf agents); the fractal system **nests it**: an **Orchestrator** spawns
**Epic Agents**, each of which spawns **Leaf Agents**. The collision-free invariants — disjoint
work, parent-owned shared files, bottom-up octopus merges, honesty preserved through integration —
hold **at every level**. Everything in the section above still applies; this section adds the
**model modes, branch naming, multi-level ownership, merge flow, and reusable role prompts**.

### Swarm modes (model assignment)
The active mode fixes which model **every** spawned agent uses. Set it at session start with
`use <X> swarm`; the orchestrator must honor it for all agents it (transitively) spawns, passing
the resolved model **explicitly** to each spawn — never substituting silently.

| Mode | Orchestrator | Epic Agents | Leaf Agents |
|---|---|---|---|
| **Sonnet Swarm** *(default — used when none is specified)* | Sonnet | Sonnet | Sonnet |
| **Haiku Swarm** | Haiku | Haiku | Haiku |
| **Opus Swarm** | Opus | Opus | Opus |
| **Hybrid Swarm** | Opus | Sonnet | Haiku |

- No mode named ⇒ **Sonnet Swarm**. `use hybrid swarm` ⇒ Orchestrator **Opus**, Epic **Sonnet**,
  Leaf **Haiku**. `use opus swarm` / `use haiku swarm` ⇒ that model for the whole hierarchy.

### Branch naming convention
Exact formats — kebab-case descriptions; `<EPIC>`/`<LEAF>` are compact **Base36** IDs (alphabet
`0-9A-Z`, e.g. `MS1A0`, `MS42F`):

```
Orchestrator   claude/orch-0000-<kebab-description>
Epic Agent     claude/epic/<EPIC>-<kebab-description>          e.g. claude/epic/MS1A0-vsa-models
Leaf Agent     claude/leaf/<EPIC>-<LEAF>-<kebab-description>   e.g. claude/leaf/MS1A0-MS42F-hrr-unbind
```

A **Leaf branch must embed its parent Epic ID** as the first ID segment, so lineage
(`Epic → Leaf`) is readable from the branch name alone.

### Core file-ownership rule
Ownership **rises to the nearest shared parent**:
- Any file/module modified by **more than one agent at the same level** is owned by their shared
  parent.
- Multiple **Leaf** agents touching the same file ⇒ owned by their **Epic** (same epic) or the
  **Orchestrator** (across epics).
- A child treats every **parent-owned file as read-only** — if it needs a change there, it
  **FLAGs it up**, it does not edit it.
- **Only the owning parent** performs integration on a shared file.

This is the fractal form of "the orchestrator owns the collision surface": at each level the parent
owns whatever its children share, so every merge below it is conflict-free *by construction*.

### Hierarchy & merge flow

```
main
 └─ Orchestrator   claude/orch-0000-…        (branches from main)
     └─ Epic       claude/epic/<EPIC>-…      (branches from the Orchestrator branch)
         └─ Leaf   claude/leaf/<EPIC>-<LEAF>-… (branches from its Epic branch)
```

Merges are **bottom-up octopus merges**: **Leaf → Epic → Orchestrator → main**. Each parent
octopus-merges its children's branches, makes the single integrating edit to the files **it** owns,
runs the level-appropriate checks green (`just check` at the top), reviews each merged subtree
against the house rules, then reports/merges up.

### Role prompt blocks
Reusable, self-contained briefs. Each agent resolves its model from the **active swarm mode**,
follows the **branch convention**, and obeys the **file-ownership rule**.

**Orchestrator Role**
> You are the **Orchestrator** of a `<MODE>` swarm — model **Opus** (Hybrid) else the mode's model
> (Sonnet default · Haiku · Opus). Branch from `main` as `claude/orch-0000-<kebab-description>`.
> Decompose the wave into independent **epics**; mint a Base36 `<EPIC>` id for each and spawn one
> **Epic Agent** per epic — at the model the active mode assigns (Hybrid ⇒ Sonnet) — each branching
> from **your** branch. You **own every file shared across epics** (workspace manifests,
> `CHANGELOG.md`, `docs/Doc-Index.md`, `tools/github/*`, `docs/planning/phase-*.md`, shared
> indices); epics treat these **read-only**. After epics report, **octopus-merge** their branches
> into your branch, make the single integrating edits to your owned files, run the **full**
> `just check`, fix integration, review every merged subtree for honesty/grounding/append-only,
> update the changelog (append-only; specs → "implemented, pending ratification", never silently
> `Accepted`), commit, and merge up to `main`. Pass the resolved model explicitly to each spawn.

**Epic Agent Role**
> You are an **Epic Agent** owning epic `<EPIC>` — model **Sonnet** (Hybrid) else the mode's model
> (Sonnet default · Haiku · Opus). Branch from the Orchestrator branch as
> `claude/epic/<EPIC>-<kebab-description>`. Decompose your epic into independent **leaf tasks**;
> mint a Base36 `<LEAF>` id for each and spawn one **Leaf Agent** per task — at the model the active
> mode assigns (Hybrid ⇒ Haiku) — each branching from **your** branch as
> `claude/leaf/<EPIC>-<LEAF>-…`. You **own every file shared across your leaves**; leaves treat
> these **read-only**, and you treat **Orchestrator-owned files read-only** (FLAG up, don't edit).
> After leaves report, **octopus-merge** them into your epic branch, integrate your owned files, run
> the epic's checks green, review for honesty/grounding, and report your **branch + SHA + FLAGs** up
> to the Orchestrator. Do **not** push to `main`. Flag ambiguity; never guess (G2/VR-5).

**Leaf Agent Role**
> You are a **Leaf Agent** for task `<LEAF>` under epic `<EPIC>` — model **Haiku** (Hybrid) else the
> mode's model (Sonnet default · Haiku · Opus). Branch from your **Epic** branch as
> `claude/leaf/<EPIC>-<LEAF>-<kebab-description>`, in an isolated worktree. Edit **only your disjoint
> directory**; treat all **Epic- and Orchestrator-owned files as read-only** (if you need a change
> there, **FLAG it up**, don't edit it). Follow `/dev-workflow`: small auditable steps, honest
> per-op guarantee tags, a property test for every bound, never-silent fallibility (`Option`/
> `Result`), `EXPLAIN`-able selections. Run `cargo fmt` / `clippy -D warnings` / `test -p <crate>`
> green. Commit to your leaf branch (do **not** push); report your **branch + SHA + any FLAGs** to
> your Epic Agent.

## Skills (`.claude/skills/`)
Invoke with `/<name>`; they auto-engage when relevant.
- **`/dev-workflow`** — the implementation discipline above, as a working loop.
- **`/pr-review`** — opinionated PR/diff review (honesty rule, grounding, append-only,
  hallucination pass). Adaptive depth (T0/T1/T2) + `--all` exhaustive mode.
- **`/security-review`** — secrets, supply-chain, shell/CI safety; auto-light on docs-only.
- **`/docs-review`** — cross-refs, notation, grounding labels, status/changelog discipline.
- **`/changelog`** — keep `CHANGELOG.md` + per-doc footers in sync, append-only.

The review skills share one rubric: `.claude/skills/_shared/review-rubric.md` (tiers, severity,
report format). Posture is **advisory** — they recommend, they don't gate.

## Map
- `docs/Mycelium_Project_Foundation.md` — charter (FR/NFR/VR, SC-*/KC-*, ADR-001…009, roadmap).
- `docs/rfcs/`, `docs/adr/`, `docs/notes/` — normative designs, decisions, tradeoff notes.
- `research/` — the evidence base the corpus traces to.
- `tools/github/` — issue/label/milestone bootstrap (`mcp-bootstrap.md`, `gh-bootstrap-local.sh`,
  `issues.yaml`, `idmap.tsv`).
- `justfile`, `.pre-commit-config.yaml`, `scripts/` — the local/CI check tooling.
