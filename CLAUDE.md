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
Mycelium is a fast, memory-safe, ergonomic multi-paradigm value-semantics language (binary/ternary/dense/VSA) whose **transparent, never-silent** representation swaps and **per-operation provenance/auditability** are baked in as **optional, tunable** capabilities (`fast` default · `certified` on request — RFC-0034/ADR-032). It is in the
**design phase**: the corpus in `docs/` is the product right now; code lands per the phase plan
(see `tools/github/issues.yaml` and the `M-xxx`/`E*` task ids).

## Lexicon — name things correctly (fungal, on-brand)
Mycelium's libraries/units are **not** "crates"/"modules". Core mapping: **`phylum`** = library /
package (versioned, content-addressed; ≈ crate) · **`nodule`** = the basic static unit / "module"
(opens a program via a `// nodule:` header) · **`spore`** = the deployable/published artifact
(ADR-013) · **`hypha`** = one concurrent execution unit (task) · **`colony`** = a runtime grouping
of hyphae · **`swap`** = the never-silent representation change. (The *Rust kernel* packages named
`mycelium-*` are genuinely Rust crates; *Mycelium-language* units are phyla/nodules — keep that
distinction.) Of these grouping terms `nodule` is an active keyword and `phylum`/`colony` are
reserved-not-active (lexed, no construct yet); most of the runtime tier (`hypha`, `fuse`, `mesh`, …)
are ratified names not yet lexed. Full reference
— reserved words, surface syntax, grammar, conventions: **`.claude/memory/lang-lexicon-syntax.md`**;
canonical definitions: `docs/Glossary.md` + DN-02/03/06.

## Non-negotiable house rules
1. **The transparency rule** *(reframed from "the honesty rule" by ADR-032; mechanism unchanged)*. Every accuracy/guarantee claim is tagged per-model/per-op on the
   lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`. `Proven` is allowed **only** with a theorem
   whose side-conditions are *checked*; otherwise `Empirical` (trials) or `Declared` (asserted,
   always flagged). Downgrade to stay accurate; never upgrade without a checked basis (VR-5).
2. **No black boxes.** Selections/conversions/approximations are reified, inspectable, and
   `EXPLAIN`-able. A swap is **never silent**; out-of-range is an explicit `Option`/error.
3. **Append-only decisions.** ADR/RFC/DN status moves forward only
   (`Draft/Proposed → Accepted → Enacted → Superseded`; notes `→ Resolved`). **`Enacted`** = an
   Accepted decision now **fully implemented/landed** — complete and stable, outside ongoing
   maintenance and future-dev integration; it **must step through `Accepted`** first (never skip
   straight to Enacted). To change an Accepted/Enacted decision, **supersede** it — don't rewrite
   history.
4. **Ground every claim — including agreement.** Normative statements cite their basis (`G1–G11`,
   tensions `A–E`, `R1–R8`, `T0.x–T2.x`) or are marked open questions. No ungrounded "facts".
   **The same rule binds assent: agree only on merit, never to please.** An affirmation *is* a claim —
   tag its strength (checked/`Proven` vs plausible/`Empirical` vs asserted/`Declared`) and surface the
   disconfirming evidence **even when it cuts against the maintainer's stated direction**. **Sycophancy
   is a defect, ranked with an ungrounded claim.** The maintainer's standing preference is explicit:
   *be corrected over being wrongly affirmed — follow the evidence, not the speaker.* Say "I don't
   know" / "this is unproven" plainly; flag confidence; never soften a real disagreement into
   agreement. This is **VR-5 applied to agreement: don't upgrade assent past its basis.**
5. **Small, auditable kernel** (KC-3). SOLID · DRY · KISS · YAGNI · Law of Demeter · SoC;
   composition over inheritance.
6. **User stories + Definition of Done; MIT-only.** Every epic/issue carries explicit **user stories**
   (`As a <role>, I want <X>, so that <Y>` — realistic use cases + the problems to resolve) **and** an
   explicit **Definition of Done**; every ADR/RFC/DN carries a Definition of Done (its gate/criteria —
   a corollary of rule 1: you can't claim "done" without stating what done means). The project
   is **MIT-licensed only** — no Apache/dual-license on first-party artifacts (CONTRIBUTING §Licensing;
   ADR-022 §7).

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
just setup      # best-effort install of the check tools (uv tool / npx / pip / cargo-nextest)
just test-fast  # Tier 0 (pre-commit): change-scoped crates' unit/regression tests — fastest
just check      # Tier 1 (default; = `just ci`): change-scoped tests (LOW proptest cases) + all gates
just check-full # Tier 2 (release/durability): full workspace, HIGH proptest cases, mutants + fuzz
just fmt        # auto-format (rust + python)
just hooks      # install the pre-commit hooks
```

**Three test tiers (DN-20)** — change-scoped + heavy-gated, via cargo-nextest (with a `cargo test`
fallback): `just test-fast` is the sub-second pre-commit loop (scoped crates' unit/regression tests
only); **`just check` stays the default and the CI entrypoint** (`just ci` = `just check`) —
change-scoped crates **+ their reverse-dependents**, all targets + proptest at LOW cases + every
always-on non-test gate; `just check-full` is the release/nightly durability gate (full workspace,
HIGH proptest cases, `cargo-mutants` + `cargo-fuzz` smoke). **Transparency (VR-5):** no property/bound
test is dropped — only its *case count* is tiered (low every commit, full on release); the
change-scoping only ever *widens* to `--workspace` (over-test, never under-test), and `check-full`
always runs everything.

Checks **skip gracefully** when a tool or language isn't present yet (most code doesn't exist
yet). Never hand off a red `just check` without explaining the skip.

**Pre-commit in repo-scoped remote sessions — `--no-verify` is permitted, gates run out-of-band.**
In a Claude-Code-on-the-web / GitHub-Action session whose GitHub access is **scoped to this repo**,
`pre-commit` cannot fetch its *external* hook repos (`pre-commit/pre-commit-hooks`, `gitleaks` — the
scoped proxy 403s them), which aborts the **entire** hook run before any local hook executes, blocking
every `git commit`/`git push`. In that environment **`git commit --no-verify` / `git push --no-verify`
is the sanctioned path** — it is pre-allowed in `.claude/settings.json` (`permissions.allow`), scoped
to exactly the `--no-verify` forms. This is **not** a license to skip checks: before each such commit,
run the equivalent gates **out-of-band** — `cargo fmt` · `cargo clippy -D warnings` · `cargo test`
(or `just check`) · `scripts/checks/branch-guard.sh` · `scripts/checks/secrets.sh`, **plus
`scripts/checks/markdown.sh` whenever the change touches any `.md`** (and `links.sh`/`structured.sh`
for cross-ref/YAML edits) — and the harness-level **PreToolUse branch-guard hook stays armed**
regardless of `--no-verify`, so the protected-branch block still holds (mitigation #10). Local sessions
where pre-commit *can* fetch its hooks keep using the normal verified path. Never use `--no-verify` to
skip a gate that *would* have caught a real failure — only to route around the unreachable-external-repo
abort (G2: the bypass is documented + conditioned, never silent).

**Markdown authoring — the soft-wrap `+`/`-`/`*` pitfall (MD004, learned 2026-06-29).** `markdownlint`
reads any line whose first non-space char is `+`, `-`, `*`, or `N.` as a **list item** — so prose that
soft-wraps such that a continuation line *starts* with one of those (e.g. wrapping `acquire + take`,
`(trait + inherent forms)`, or `fixture + tests` so `+ …` lands at line start) trips **MD004**
(unordered-list-style) and **fails the `markdown` gate**. Prevention: when authoring `.md` prose
(esp. `CHANGELOG.md` entries + RFC/DN notes), **never let a wrap put `+`/`-`/`*`/`N.` at the start of
a continuation line** — reword (`+`→"and"/"plus") or keep the token off line-start. Likewise a blank
line *between two adjacent blockquotes* trips **MD028** — join them with a `>` continuation line. The
`markdown.sh` gate (now in the out-of-band set above) catches both; running it on touched docs before
committing prevents the red gate at PR time.

## Test layout — no tests in logic files (in-crate `src/tests/`)
**Logic files carry no test code.** Every `#[cfg(test)]` unit test lives in a dedicated **in-crate**
test module, not inline in the `.rs` it tests: `#[cfg(test)] mod tests;` in `lib.rs` → `src/tests/`
(a `mod.rs` declaring one submodule per source module, e.g. `src/tests/cert_mode.rs`), each doing
`use crate::…::*` for **white-box access to private items** (precedent: `mycelium-std-recover/src/tests.rs`).
This keeps logic files clean **without** the black-box coverage loss of fully-external `tests/` (which would
force internals `pub`). Integration/behavioural tests that only need the public API still go in `tests/`.
**Complex test logic lives in fixtures + parameterization, not in test bodies** — data-driven cases
(corpus tables, `CertMode::ALL`-style parameterization, the conformance `REJECT_EXPECTED` pattern,
`differential.rs::data_corpus()`), so a test body is *assert over a case*, not bespoke logic.
**Enforced going-forward, retrofit lazily** (maintainer's chosen rollout): new/changed code complies
immediately, and **when you modify a logic file that still has inline tests, extract them as part of that
change** (as-touched — no big-bang refactor; the codebase stays mixed until the lazy sweep completes,
which is accepted). The remaining inline-test retrofit (~185 files) is tracked as **M-797**.

## Remote CI policy
**No automatic remote CI.** `.github/workflows/checks.yml` is **manual-dispatch only**
(`workflow_dispatch`) and **advisory** (non-blocking) — to keep parity with local while avoiding
surprise Actions cost. It runs the same `just ci` (= `just check`, the fast Tier-1 default — DN-20:
change-scoped tests at low proptest cases + all non-test gates; the heavy `just check-full`
durability tier — full workspace, mutants, fuzz — is run deliberately, not in this advisory job). Do
not add `on: push` / `on: pull_request` auto-triggers without an explicit decision.

## Commits & PRs
- Conventional, imperative subjects referencing the issue/task
  (`docs(rfc-0003): tighten capacity-bound wording`, `feat(swap): …`).
- A PR states which `FR/NFR/VR/SC` it advances (or which ADR/RFC it implements) and **how it was
  verified**. Editorial-only PRs say so.
- **Tiered branches (`dev → integration → main`) — `main` is never touched directly.** Day-to-day
  work branches off **`dev`** (the working tier — messy OK: WIP, exploration, octopus/swarm merges;
  only *compiles + change-scoped tests* required), promotes via PR to **`integration`** (the staging
  tier — the full `just check` green + the transparency/append-only review, shared files reconciled once),
  and `integration → main` is the polished, **squash-only release** (PR-gated by `/pr-review`; the
  agent-driven review-with-another-agent pass is the gate — third-party review bots (Copilot,
  Sourcery) are **disabled** in this repo). Each tier is PR-gated and **more stringent than the last**; `main`/`integration`/
  `dev` are persistent + protected (no direct push), everything below `dev` is ephemeral and merges
  freely. `main` advances **only** through the `integration → main` squash-PR — never a direct
  `git push`/merge/commit, even for a one-file fix. Full workflow + the per-isolated-tree kickoff
  index (parallel Sonnet swarms): **`.claude/kickoffs/README.md`**.
- **Squash-only into `main`.** Every PR lands on `main` as a **single squash commit** — a linear,
  bisectable history that keeps downstream development and integration merges smooth. Squashing
  happens **only** at the PR→`main` step; the internal swarm integration merges (leaf→epic→orch)
  stay octopus/`--no-ff` to preserve lineage — **only the final landing on `main` squashes.**
- **Curate the squash commit — housekeeping is part of the merge.** Write a clear, self-contained
  subject + body describing the *net* change; **never** let the auto-concatenated WIP /
  `wip(batch …)` / fixup / merge trail stand as the squash message. The commit left on `main` is the
  permanent record — keep it clean and legible, not cluttered.

## Swarm development — octopus-merge pattern (parallel agents, zero collision)
When a wave decomposes into several **tightly-scoped, independent** tasks (e.g. one stdlib
module / capability crate per task), run them as a **swarm of agents merged with a single
octopus merge**, with **one orchestrator owning every shared file**. This keeps parallel work
collision-free *by construction* rather than by after-the-fact conflict resolution.

**Pre-flight — align + prune + push before launching any agent (mandatory; ties to the
branch-ref-drift and stale-base mitigations #5/#7).** Branch-ref drift and stale-base launches are
the two cheapest swarm failures to *prevent* and the most expensive to *recover from*. Before
spawning the first agent, the orchestrator runs a branch-hygiene pre-flight so every agent launches
from a correct, pushed tip:
1. **Align the working branch with `main`.** `git fetch origin`, then ensure the orchestrator branch
   is on the intended `main` tip (`git rebase origin/main` or branch fresh from it) — never fan out
   from a branch that has silently diverged from `main`.
2. **Prune stale local branches + worktrees.** Delete every local branch that is **not `main` and
   not the current working branch**, and remove stale worktrees, so no agent can branch from or merge
   a leftover ref (`git worktree prune`; `git branch` → delete the others; `git worktree list` to
   confirm). A clean ref namespace is what makes "merge the ref the child reports" unambiguous.
3. **Push the working branch, then launch from the *pushed* tip.** An `isolation:"worktree"` agent
   branches from the branch's **upstream** (`origin/…`), so push the orchestrator/epic branch
   **before** spawning its children. Push first; deconflict never (mitigation #5). Propagate
   freshly-squashed `main` *down* through every level after each landing (mitigation #6).

The discipline:
1. **Partition by file ownership, not just by task.** Each agent owns a **disjoint directory**
   (prefer **one crate per task** — `crates/mycelium-std-<module>` — so the only shared file is
   the workspace `Cargo.toml`). An agent edits **nothing** outside its directory.
2. **Orchestrator owns all common/shared files** — the wave's collision surface: workspace
   `Cargo.toml`, `CHANGELOG.md`, `docs/Doc-Index.md`, `tools/github/issues.yaml` + `idmap.tsv`,
   `docs/planning/phase-*.md`, shared spec indices, per-doc changelog footers, `docs/api-index/`.
   Agents never touch these; the orchestrator reconciles them once, after merge.
3. **Scaffold first, then fan out.** The orchestrator creates each task's skeleton (crate
   manifest with deps pre-filled, stub `lib.rs`), registers it in the workspace, and **commits +
   pushes the scaffold** so every agent branches from a *buildable* base and never needs to edit
   shared wiring.
4. **One agent per task, isolated worktree.** Launch each on its own git worktree branch
   (`isolation: "worktree"`), in parallel. Each follows `/dev-workflow`, ships its
   guarantee tags + tests, runs `cargo fmt`/`clippy -D warnings`/`test -p <crate>` green, commits
   to its worktree branch (does **not** push), and reports its branch + SHA + any FLAGs. An agent
   that hits ambiguity **flags it**, it does not guess (G2/VR-5 apply to agents too).
5. **Octopus-merge back into the working branch.** `git merge --no-ff <b1> <b2> … <bN>`. Disjoint
   directories + a pre-finalized workspace manifest ⇒ the N-way merge is conflict-free. The
   orchestrator then makes the single integrating edits to the shared files (step 2), runs the
   **full** `just check`, fixes integration, and commits + pushes.
6. **Transparency survives the swarm.** The orchestrator reviews each merged crate against the house
   rules before the wave's changelog entry; tags stay at the supportable strength, and a
   spec moves to "implemented (Rust-first), pending ratification", never silently to `Accepted`.

## Fractal Swarm Development System
The **recursive generalization** of the octopus-merge pattern above. That pattern is single-level
(one orchestrator, N leaf agents); the fractal system **nests it**: an **Orchestrator** spawns
**Epic Agents**, each of which spawns **Leaf Agents**. The collision-free invariants — disjoint
work, parent-owned shared files, bottom-up octopus merges, transparency preserved through integration —
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
> `just check`, fix integration, review every merged subtree for transparency/grounding/append-only,
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
> the epic's checks green, review for transparency/grounding, and report your **branch + SHA + FLAGs** up
> to the Orchestrator. Do **not** push to `main`. Flag ambiguity; never guess (G2/VR-5).

**Leaf Agent Role**
> You are a **Leaf Agent** for task `<LEAF>` under epic `<EPIC>` — model **Haiku** (Hybrid) else the
> mode's model (Sonnet default · Haiku · Opus). Branch from your **Epic** branch as
> `claude/leaf/<EPIC>-<LEAF>-<kebab-description>`, in an isolated worktree. Edit **only your disjoint
> directory**; treat all **Epic- and Orchestrator-owned files as read-only** (if you need a change
> there, **FLAG it up**, don't edit it). Follow `/dev-workflow`: small auditable steps,
> per-op guarantee tags, a property test for every bound, never-silent fallibility (`Option`/
> `Result`), `EXPLAIN`-able selections. Run `cargo fmt` / `clippy -D warnings` / `test -p <crate>`
> green. Commit to your leaf branch (do **not** push); report your **branch + SHA + any FLAGs** to
> your Epic Agent.

## Swarm failure-mode mitigations (lessons from Wave-4, 2026-06-19)

These are recurring failure patterns observed in multi-agent waves. Treat them as mandatory
pre-checks and post-checks — not optional hygiene.

### 1. ID namespace collision
**Pattern:** Orchestrator mints a new `M-xxx` / `Exx` ID from the plan without verifying the slot
is free in `issues.yaml`. Results in a collision that must be fixed mid-wave.  
**Mitigation:** Before assigning any new ID, grep `issues.yaml` for the candidate: `grep "id: E3-8"
tools/github/issues.yaml`. If taken, find the next free slot. Do this before spawning epics.

### 2. Union-merge YAML duplication
**Pattern:** `.gitattributes` applies `merge=union` to `issues.yaml` so octopus merges
append both sides of every touched block, creating duplicate YAML keys that are syntactically
valid but semantically wrong.  
**Mitigation:** Immediately after every octopus merge, validate + dedup `issues.yaml`:
`python3 -c "import yaml; yaml.safe_load(open('tools/github/issues.yaml')); print('OK')"`.
If duplicates are present, consolidate manually into single canonical entries before any further
commits. Consider whether `issues.yaml` should remain in the union-merge set (it is
orchestrator-owned and never agent-edited — union merge has no benefit).

### 3. Tool discovery / PATH failures
**Pattern:** Agent invokes `python3 -m ruff` but `ruff` is installed as a standalone binary at
`~/.local/bin/ruff` (via `uv tool install`) and that path is not on `$PATH` for subprocess
invocations.  
**Mitigation:** Always prefer `just fmt` and `just check` — the justfile resolves tool paths.
When invoking raw tools, probe first: `command -v ruff || ~/.local/bin/ruff`. The `just setup`
recipe should verify tool paths and warn if they're not on `$PATH`.

### 4. Interactive git flags in automation
**Pattern:** `git add -p` / `git add -i` / `git rebase -i` launch interactive pagers that block
non-interactive agent contexts.  
**Mitigation:** Never use `-p`, `-i`, or `--interactive` git flags in agent context. Stage with
explicit file paths: `git add <file1> <file2>`. The CLAUDE.md git section already says no `-i`
for rebase; the same applies to `add`.

### 5. Agent progress opacity (appears hung)
**Pattern:** An agent annotating N independent items (e.g. 23 std crates) sequentially emits no
visible signals — looks identical to a stuck agent from the orchestrator's view.  
**Mitigation:** Agents processing N ≥ 5 independent, repetitive items MUST commit in batches
(every 5–7 items) with a `wip(batch M/N): ...` message. Orchestrator can then poll progress
via `git log worktree-agent-<id> --oneline`. Agents may also emit a brief text status line
after each batch.

### 6. Orchestrator context exhaustion
**Pattern:** A single orchestrator session accumulates the full context of Step 0 + Wave-4A
fan-out + monitoring + integration across many large file reads, exhausting the context window
before Wave-4B even starts.  
**Mitigation:** Spawn a read-only `Explore` subagent for any pre-work that requires reading > 3
large files. Use `TaskOutput(block=false)` for progress polls (don't block). Summarize each
phase explicitly (in-context) before starting the next. The orchestrator's context budget is the
scarcest resource in a multi-wave swarm — protect it.

### 7. Branch-ref drift → silent partial octopus merge (lesson, 2026-06-21)
**Pattern:** A worktree agent commits to a branch whose name differs from the one the parent
assumed (e.g. the parent merged `worktree-agent-<id>` but the agent had created and committed to
`claude/leaf/<EPIC>-<LEAF>-…`). `git merge --no-ff <assumed-branch>` then "succeeds" by merging an
**empty/stale** ref, so only a subset of the children's files land — and an octopus merge reports
success regardless. The gap is invisible unless you count the result.  
**Mitigation:** Two cheap, mandatory guards:
1. **Merge the ref the child reports, not an assumed name.** Every agent reports its **exact branch
   ref and SHA**; the parent merges *that* ref. Before merging, confirm the ref carries the expected
   payload: `git ls-tree --name-only <ref> <dir>/` is non-empty (or `git rev-list --count
   main..<ref>` > 0).
2. **Verify the merge landed the whole set.** After the octopus merge, assert the expected file
   count/paths are present (e.g. `ls <dir> | wc -l` equals the number of leaves × files-per-leaf,
   and a manifest/index ↔ actual `diff` is empty). A green merge is **not** evidence the content
   arrived — count it. (This is the merge-time twin of mitigation #2's post-merge YAML dedup.)

### 8. Lost intent on compaction — persist before you can't (standing policy, rsm 2026-06-26)
**Pattern:** A session/agent approaching context compaction silently loses in-flight reasoning,
decisions, and "where am I / what's next" state — the thread breaks mid-task and the next window
can't recover the intent (only the code, if it was committed).
**Mitigation:** Before you can't — when any agent (orchestrator/epic/leaf) nears compaction or a
long handoff — **write working state to disc**: a scratch/memory file with working notes, decisions
made, current position, and the next steps. Branches are the durable artifact for *code* (#5/#9);
this note is the durable artifact for *intent*. This is **standing policy for every agent**, not just
swarm runs (reinforces #6: the orchestrator's context budget is the scarcest resource — protect it).

### 9. Unrecoverable work — commit + push frequently to a working branch (standing policy, rsm 2026-06-26)
**Pattern:** An agent holding hours of uncommitted (or committed-but-unpushed) work loses **all** of
it when orphaned on compaction (the Wave-4 mass-orphan durability lesson: worktree branches are the
durable artifact; an unpushed tip is gone).
**Mitigation:** Every agent commits in **small batches and pushes to its working branch** on the
`wip(batch M/N)` cadence (#5) — no agent holds hours of uncommitted work, and worktree leaves
**push before they complete**. If lost, the work is recoverable from the branch, not gone. This is
**standing policy for every agent**. (#5 is the *visibility* twin of this *durability* rule.)

### 10. Wrong-branch / commit-to-`main` — now ENFORCED, not just convention (standing policy, rsm 2026-06-27)
**Pattern:** an agent (or an orphaned sub-agent) commits/pushes to a protected branch, or writes to
the wrong working branch — the discipline below was documentation, so nothing actually *stopped* it.
**Mitigation — the branch-guard (`/branch-guard`), three layers, idempotent + parameterized:**
(1) a **Claude Code `PreToolUse(Bash)` hook** (`.claude/settings.json` → `scripts/hooks/claude-git-branch-guard.sh`)
blocks an agent's `git commit`/`merge`/`cherry-pick`/`rebase` on, or push to, a protected branch, and
any force-push, **before** the tool runs — the layer that stops agents; (2) **git pre-commit + pre-push
hooks** (`.pre-commit-config.yaml`, `repo: local` → `scripts/checks/branch-guard.sh`) for direct git
use; (3) the **`/branch-guard` skill + `just branch-guard`** checked step the workflows call.
Protected set (`MYC_PROTECTED_BRANCHES`, default `main integration dev claude/head/*`) and the
expected working branch (`CLAUDE_WORKING_BRANCH` / `--expect`) are **parameters**; the checks are pure
reads (idempotent). Landing onto a protected branch is **via GitHub PR**, never local git — so the
block is exactly correct. Never-silent (G2): a blocked op prints the protected/wrong branch + the fix.

## Autonomous PR workflow — review-before-merge, no human gate

The merge gate is the agent's, not a human's. A parent (orchestrator/epic) **merges its children
up the tree itself** once the work is clean — but only after the discipline below. This makes the
swarm self-driving while keeping transparency and history intact. (A maintainer may still override; if
asked to wait, wait.)

1. **Self-review before every merge.** Before merging anything (leaf→epic, epic→orch, orch→`main`,
   or a PR→`main`), run the `/pr-review` lens on the diff yourself: the transparency rule (per-op tags
   never upgraded without a checked basis), append-only decisions, grounding, never-silent G2, and
   a hallucination/consistency pass. Fix what you find or stop and flag it — never merge past a
   Critical/High you can't resolve.
2. **Handle every CI / bot review comment first.** For each review comment (CI failure,
   a human note; third-party review bots are disabled — see the tiered-branch note above): investigate, then **fix if you're confident and it's small**, **defer**
   if the fix would be fragile or large (keep an explicit refusal + a clear message + a spec-§ note,
   never ship fragile/incorrect output to satisfy a comment — G2/VR-5), or **ask** (`AskUserQuestion`)
   if it's ambiguous or architecturally significant. Reply once, frugally; the diff is the record.
3. **Green, then merge.** The full `just check` (local↔CI parity) must be green — fix integration,
   regenerate orchestrator-owned artifacts (`docs/api-index/`, api baselines, `CHANGELOG`,
   `issues.yaml` status), then merge as a **single curated squash** to `main` (the squash-only policy
   above — a clean subject + body for the net change, never the WIP/`wip(batch …)`/fixup/merge trail).
4. **Pull-down before merge-up — keep tips current, never integrate across divergent history.**
   Before merging a child into its parent, **pull the parent down into the child first** (or branch
   the child from the parent's *latest pushed tip*), so the child already contains the parent's
   history and the merge-up is a clean fast-forward / conflict-free. Repeat at **every** level going
   up. If a leaf was spawned from a stale base, the leaf (which owns its code's context) pulls the
   parent down and resolves *there*, then reports back — the orchestrator never resolves a large
   merge blind.
5. **Branch children from a *pushed* tip, not the upstream.** Push the parent branch **before**
   spawning worktree children — an `isolation:"worktree"` leaf branches from the branch's *upstream*
   (`origin/...`), so an un-pushed parent tip leaves the leaf on a stale base and forces an
   after-the-fact deconfliction (lesson: M-379 Stage-2). Push first; deconflict never.
6. **Pull the squashed `main` down before PR-ing into `main`.** Because landing squashes the whole
   PR into one commit, a branch's pre-merge commits **diverge** from `main`'s new history the moment
   any PR lands. So **before opening or squash-merging a PR into `main`, first pull the latest
   (squashed) `main` down into the working branch** (`git fetch origin main` → merge/rebase onto it),
   resolve there, and re-run `just check`. The PR diff then shows **only** this branch's net change
   against current `main`, and the squash-merge stays conflict-free. In a swarm, propagate the
   freshly-squashed `main` **down through every level** (orch → epic → leaf) after each landing so no
   lower branch keeps building on a superseded base — pull-down flows *down*, squash-merge flows *up*.
   **Force pushes are prohibited — full stop.** No `git push --force`, no `--force-with-lease`, no
   `+refs` push spec, on **any** branch (and *absolutely never* on the protected `main`/`integration`/
   `dev`/`claude/head/*`). Misalignment is corrected by bringing history *together*, **never** by
   rewriting published history. For an **already-pushed** branch — the case that matters — **merge `main`
   into it** (pull-down), resolve, then a *plain* push: a merge only ever *adds* a commit, so the push
   fast-forwards the remote branch and no force is needed. **Do not rebase a pushed branch** — rebasing
   rewrites its published commits, so the plain push would be rejected (non-fast-forward) and *only* a
   force (which this rule forbids) could land it; reach for **merge**, not rebase, once a branch is
   published. (A **local-only, never-pushed** branch may be rebased freely before its first push — that
   is reconciliation, not a rewrite of published history.) This is the durable rule: a plain push that is
   rejected (non-fast-forward) is a never-silent signal to *reconcile*, not a problem to overwrite. When
   local work is in the way of pulling history together, the mechanism is **`git stash` → reconcile
   (merge `main` in — rebase only if the branch is still local-only) → `git stash pop` → deconflict** → a
   plain push: it *keeps* your work and resolves the divergence, where a force would have
   silently discarded the other side. Stashing-and-deconflicting is always preferable to a force-push —
   there is no divergence a force fixes that a merge (+ stash-pop) cannot fix without losing history.
   Keep the per-session working branch a *clean pointer at `main`*: do the
   work + reconcile on a per-task/leaf branch, PR **that**, and after the squash lands bring the working
   branch up with `git fetch origin main` → `git merge --ff-only origin/main` (`git stash` first if
   dirty) → a plain `git push`. Because the working branch never carried the squashed commits it stays an
   ancestor of the new tip, so `--ff-only` always succeeds — and *fails loudly* if it ever diverged (the
   never-silent guard) instead of papering over divergence with a force.
   **The one case that used to "justify" a force — a published branch whose own pre-squash commits
   already landed on `main` — is resolved without one:** treat the diverged branch as spent and branch a
   **fresh** one off the current `main`, re-applying only the *unlanded* work (cherry-pick / re-commit);
   or merge current `main` into it and continue forward. A diverged branch is a cue to re-branch from
   `main`, never a license to overwrite history. (Local-only, never-pushed branches may still be rebased
   freely before their first push — that is reconciliation, not a force-push of published history.)

## Wave-N multi-session workflow — protected bases, free children, squash-only `main`

When a wave is too big for one session, split it into **independent parent sessions** by **disjoint
file ownership** — one **protected head branch** each — stowed as kickoffs in `.claude/kickoffs/`
(`README.md` indexes them; each has a short UID, e.g. `e7l`/`dfr`/`dfb`). Invariants:
- **Persistent branches are PR-gated & protected:** `main` and every `claude/head/*` base — no direct
  push/merge, **PR only**. They **persist** when stale working branches are pruned.
- **Only `main` squash-merges.** Heads (and the octopus/`--no-ff` child merges below them) **preserve
  lineage** — squashing happens *only* at the head→`main` PR.
- **Ephemeral child branches merge freely (no PR).** Below a head, working/leaf branches octopus/
  `--no-ff` into each other in whatever pattern is optimal. Flow: free child merges → **PR into the
  head** → (final) **squash-PR into `main`** → **propagate the squashed `main` back down** into the
  other heads/children (`scripts/sync-heads.sh`; pull-down flows down — mitigation #6).
- **Swarm pattern is scoped per collision profile** (in each kickoff): serial-on-shared-files for a
  high-collision crate (`mycelium-l1`), parallel-leaf octopus for disjoint dirs, fractured Opus
  reasoners for research. Size to the work — don't over-fan-out.
- **Cross-session continuity rides the issues** (`issues.yaml` `depends_on` + body notes), never by
  touching another session's files. Heads complete + self-integrate first; a final integration
  octopus-merges the heads, reconciles shared files once, and squash-PRs to `main`.
- Transparency/append-only survive the split (VR-5/G2) exactly as in the single-session swarm.

## Skills (`.claude/skills/`)
Invoke with `/<name>`; they auto-engage when relevant.
- **`/dev-workflow`** — the implementation discipline above, as a working loop.
- **`/pr-review`** — opinionated PR/diff review (transparency rule, grounding, append-only,
  hallucination pass). Adaptive depth (T0/T1/T2) + `--all` exhaustive mode.
- **`/security-review`** — secrets, supply-chain, shell/CI safety; auto-light on docs-only.
- **`/docs-review`** — cross-refs, notation, grounding labels, status/changelog discipline.
- **`/changelog`** — keep `CHANGELOG.md` + per-doc footers in sync, append-only.
- **`/doc-index`** — regenerate and query the agent code index (`docs/api-index/`), check
  `doc_refs` grammar validity.
- **`/land`** — land a reviewed PR on main: self-review + handle CI/bot comments → green `just check` → curated squash-merge (squash-only) → branch/worktree cleanup.

The review skills share one rubric: `.claude/skills/_shared/review-rubric.md` (tiers, severity,
report format). Posture is **advisory** — they recommend, they don't gate.

## Map
- `docs/Mycelium_Project_Foundation.md` — charter (FR/NFR/VR, SC-*/KC-*, ADR-001…009, roadmap).
- `docs/rfcs/`, `docs/adr/`, `docs/notes/` — normative designs, decisions, tradeoff notes.
- `research/` — the evidence base the corpus traces to.
- `tools/github/` — issue/label/milestone bootstrap (`mcp-bootstrap.md`, `gh-bootstrap-local.sh`,
  `issues.yaml`, `idmap.tsv`).
- `justfile`, `.pre-commit-config.yaml`, `scripts/` — the local/CI check tooling.

## Auto-generated docs & the agent index

`docs/api-index/` holds two committed, deterministic artifacts generated by `tools/docgen/code_index.py`:
- `index.json` — machine-readable symbol table (crate, file:line, summary, guarantee_tag)
- `INDEX.md` — grep-friendly table for agent context lookups, grouped by crate

**Transparency:** the index is an `Empirical/Declared` line/regex heuristic — source is ground truth.
Use the index to find where to `Read`, not as an authoritative reference. Re-exports,
macro-generated items, and cfg-gated items appear in the `flagged` section (G2: never silently dropped).

**How to use:** point an agent at `docs/api-index/INDEX.md#<crate>` via a `doc_refs` entry so it
loads targeted context instead of re-reading whole files.

**How to regenerate:** `just docs-index` — the owning parent must run this and commit the delta
after any octopus merge that touched a public API (before pushing).

**Ownership:** `docs/api-index/` is orchestrator-owned (listed above). It is REGENERATED
by the integrating parent — never hand-merged, never union-merged.

**`doc_refs:` grammar** (in `tools/github/issues.yaml`):
- `api:<crate>::<path>` — a symbol in `docs/api-index/index.json`
- `corpus:<DOC>[#<anchor>]` — a doc/section in `docs/Doc-Index.md`
- `src:<path>[:<line>]` — a source file location (relative to repo root)
Validate with: `python3 tools/github/doc_refs_check.py`
