# Design Note DN-95 — Merge & Down-Propagation Pattern (clean-base tiers vs. the squash-divergence funk)

| Field | Value |
|---|---|
| **Note** | DN-95 |
| **Status** | **Draft** (2026-07-09) — a decision-work-up for the maintainer to ratify. **Enacts nothing** and **moves no other doc's status**: it models the current `dev → integration → main` squash-and-merge-down pattern, enumerates candidate replacements, and recommends one — but the binding move is the maintainer's ratification (house rule #3, append-only). All tags `Declared` unless a cited source or a verified working-tree fact holds them higher (VR-5). |
| **Feeds** | The **§Commits & PRs** / **§Autonomous PR workflow item 6** / **§Wave-N** branching policy in `CLAUDE.md`; mitigations **#6** (no force-push) and **#13** (stale-base); `scripts/sync-heads.sh`; `.claude/kickoffs/README.md` (the tier table + down-propagation note). |
| **Date** | July 9, 2026 |
| **Decides** | *Proposes, for ratification (nothing enacted):* **the optimal branching + merge + down-propagation pattern going forward.** The recommendation: **stop persistently accumulating unsquashed lineage on the lower tiers.** Make the staging tier an **ephemeral release-candidate cut fresh off `main` each cycle** and **reset the working tier `dev` to `main` at each quiescent release boundary** via a **sanctioned, bounded branch delete-and-recreate** (an admin history reset carved out of mitigation #6 — *not* a routine force-push), so the lowers always lay cleanly on `main`'s linear squashed history. The alternatives — the current `--no-ff` merge-down (baseline), an in-place force-reset (forbidden), and full trunk-based (drop the persistent lowers entirely) — are tabled in §5/§7 with tradeoffs. **The one policy question the maintainer must rule on is stated in §9.** |
| **Task** | DN-mergepattern-eval design work-up |

> **Posture (transparency rule / VR-5 / G2).** This note **works up a decision**; it does not take
> it — the binding move is the maintainer's ratification and the `CLAUDE.md` policy edit that follows
> (house rule #3, append-only). In-repo state assertions (§2 SHAs, ahead/behind counts, content
> diffs) are `Empirical` — verified against `origin/{main,integration,dev}` on 2026-07-09 and cited
> with the command that produced them. The central finding (§3) — that squash-only **and**
> no-force-push **and** persistent accumulation tiers makes graph divergence *mathematically
> unavoidable* — is `Proven`-within-its-model: it is a property of the git DAG, side-conditions
> stated. The recommendation itself (§6) is `Declared`-with-argument: it follows from that finding,
> but "the ephemeral-RC and boundary-reset cost is worth paying" is a judgement, and its real downside
> (a sanctioned history reset; lineage moves to ephemeral branches) is surfaced in §6/§8, not buried.
> No sycophancy: §7 says plainly that if the maintainer forbids *any* history reset, the honest answer
> is that the funk is unavoidable and the baseline is correct (house rule #4).

---

## §1 Purpose — the maintainer's problem, stated precisely

The project runs tiered branches **`dev → integration → main`**, **squash-only into `main`**, with
`main`/`integration`/`dev` **protected, PR-only**, and **force-push absolutely prohibited on every
branch** (`CLAUDE.md` mitigation #6). The moment a PR squashes onto `main`, `main`'s history
**diverges** from `integration`/`dev` (which still carry the pre-squash WIP commits). Today the
divergence is patched by a **`--no-ff` merge of `main` back down** into the lowers
(`scripts/sync-heads.sh`; `.claude/kickoffs/README.md`), which makes `main` an ancestor again but
leaves the lowers carrying an ever-growing, graph-divergent history — this session observed
`integration` **2076 commits ahead of `main` by graph** while **content-diff is 3 files / 131 lines**
(§2). The maintainer wants the opposite end-state: after work has squashed **up** into `main`,
propagate `main` back **down** so the lowers **lay cleanly on top of `main`'s linear history** — a
clean base to cut the next release from — *"the most conflict-free, easiest way to deal with this …
robust, easy to de-conflict, and easy to back/down-propagate after `main` has received a new
merge,"* with *"no weird funk going forward."*

The hard constraint this note must not hand-wave: **"lay the lowers cleanly on `main`" normally means
reset/rebase them onto `main` — which rewrites their published history = a force-push, which
mitigation #6 forbids full-stop.** §3 proves the tension is real and unavoidable; §6 gives the
minimum-cost way through it; §9 names the single policy decision that unblocks it.

---

## §2 The current pattern, modelled precisely (`Empirical`, 2026-07-09)

Verified state of the three protected branches (`git rev-parse --short`, `git rev-list --count`,
`git diff --shortstat`, `git merge-base`):

| Fact | Value | Command |
|---|---|---|
| `main` tip | `707c0559` (single-parent squash) | `git rev-list --parents -n1 origin/main` → `707c0559 7b344b8f` |
| `integration` tip | `06a1c980` | `git rev-parse origin/integration` |
| `dev` tip | `e9932234` | `git rev-parse origin/dev` |
| `integration` ahead of `main` (graph) | **2076 commits** | `git rev-list --count origin/main..origin/integration` |
| `main` ahead of `integration` (graph) | **1 commit** (the newest squash `707c0559`, not yet down-propagated) | `git rev-list --count origin/integration..origin/main` |
| `integration` vs `main` **content** diff | **3 files, 131 insertions, 18 deletions** | `git diff --shortstat origin/main origin/integration` |
| `integration` ahead of `dev` (graph) | **1859 commits** | `git rev-list --count origin/dev..origin/integration` |
| `dev` ahead of `integration` (graph) | **18 commits** | `git rev-list --count origin/integration..origin/dev` |
| `integration` vs `dev` **content** diff | **27 files, 2802 insertions, 1248 deletions** | `git diff --shortstat origin/integration origin/dev` |
| `merge-base(main, integration)` | `7b344b8f` (= `main`'s *previous* squash tip, i.e. the parent of `707c0559`) | `git merge-base origin/main origin/integration` |

Read that last row together with the "2076 ahead / 3-file content diff" row: **`integration` already
contains the *previous* release squash `7b344b8f` as an ancestor** (proof the merge-down has run
before), is nearly content-identical to `main`, yet its *graph* is 2076 commits divergent. That is
the "funk" — precisely.

### §2.1 Why a fast-forward is impossible after a squash (the mechanism)

An `integration → main` PR lands as **one new squash commit `S`** whose only parent is `main`'s prior
tip. The WIP/promoted commits `c1…cn` that *composed* that PR keep living on `integration` (and their
ancestors on `dev`) with their **original SHAs**. Squashing does not rewrite `c1…cn`; it **replaces
them, in `main`'s ancestry, with the single synthetic `S`.** Consequently, immediately post-squash:

- `integration`'s tip is **not** an ancestor of `S` (`S`'s ancestry contains none of `c1…cn`), and
- `S` is **not** a descendant of `integration`'s tip.

They have **diverged**: `main` gained `S`; `integration` still holds `c1…cn`. Therefore neither
`git merge --ff-only origin/main` (into `integration`) **nor** any fast-forward of `integration`
onto `main` is possible — even though the *content* is nearly identical. Fast-forward requires an
ancestor relationship the squash destroyed. (This is the same mechanism `CLAUDE.md` §6 already names
for working branches: *"A fast-forward is not possible after a squash."*)

### §2.2 What the `--no-ff` merge-down actually produces, and why it accumulates

To restore the ancestor relation without a force-push, the current fix merges `main` **down**:
`git merge --no-ff origin/main` into `integration` (and `dev`). This creates a **merge commit `M`**
with two parents — `integration`'s old tip and `S` — after which `S` (hence `main`) is again an
ancestor of `integration`, and content converges. **But `integration` now permanently carries the
*union* of: the unsquashed `c1…cn`, the merge commit `M`, and the squash `S`** — i.e. the same net
change is represented **twice** in the graph (once as WIP, once as the squash). Every release repeats
this. After *n* releases the lowers carry *n* copies of pre-squash WIP plus *n* squashes plus *n*
merge commits, while content stays close to `main`. The growth is **monotonic and unbounded** —
exactly the 2076 / 1859-commit balloon in §2. Nothing in the current pattern ever *removes* the
redundant pre-squash lineage, because removing it from a published protected branch would be a history
rewrite = a force-push (mitigation #6). **The accumulation is not a bug in the merge-down; it is the
merge-down working as designed.** The maintainer's complaint is therefore a request to change the
*design*, not to fix a defect.

---

## §3 The core finding — the tension is real and (given the constraints) mathematically unavoidable

> **Claim (`Proven`-within-its-model).** With **(i)** squash-only landing on `main`, **(ii)**
> no-force-push on the lower tiers, and **(iii)** lower tiers that *persistently accumulate the
> commits which get squashed*, graph divergence of the lowers from `main` is **unavoidable and
> unbounded**. You cannot simultaneously have all three of {squash-only, no-force-push, persistent
> accumulation tiers} **and** a clean linear base on the lowers.

*Proof sketch.* A branch `X` can `merge --ff-only origin/main` **iff `X` is an ancestor of `main`**
(`X` holds no commit `main` lacks). A tier whose *job* is to assemble work destined for `main`
necessarily holds, at release time, the very commits `c1…cn` that get squashed into `S`. By §2.1,
after the squash `X`'s `c1…cn` are absent from `S`'s ancestry, so `X` is **not** an ancestor of
`main`, so ff is impossible. The *only* ways to make `X` an ancestor of `main` again are: **(a)** add
a merge commit (the `--no-ff` merge-down, giving accumulation, §2.2), or **(b)** move `X`'s tip *to*
`S` by `reset`/`rebase` plus push, which **rewrites `X`'s published history = a force-push**
(forbidden by (ii)). There is no third mechanism in git. QED.

Two corollaries fix the whole design space:

- **C1 — a *persistent, work-accumulating* tier can never stay clean by fast-forward.** `integration`
  (which *is* the assembled release candidate) is structurally case-(a)/(b): it will diverge (funk)
  or require a rewrite (force-push). No merge discipline escapes this.
- **C2 — the ff-only "clean pointer" trick (`CLAUDE.md` §6) works *only* for branches that were bare
  pointers at release time.** A working branch that "never carried the squashed commits stays an
  ancestor of the new tip, so `--ff-only` always succeeds" (§6, verbatim). That is exactly case-(a)
  *avoided by construction*: the branch held **no** `ci`, so it was never displaced by `S`. The trick
  scales to a whole tier **only if that tier never accumulates the released commits** — i.e. only if
  the accumulation moves off it onto ephemeral branches.

**So the design lever is singular:** *do the lower tiers persistently accumulate the commits that get
squashed, or not?* If **yes** then divergence (baseline, §5-A) unless you force-push (forbidden). If
**no** then the lowers can stay clean by ff, but the accumulation/lineage must live on **ephemeral**
branches that are discarded at the squash. Everything below is a consequence of that one choice.

---

## §4 Requirements the pattern must satisfy (the maintainer's criteria, made testable)

| # | Criterion | Testable form |
|---|---|---|
| **K1** | Conflict-free / easy to de-conflict | Down-propagation and next-release cut introduce **no** manual conflict resolution in the common case; a genuine conflict is a normal merge (never a force). |
| **K2** | Robust | Fails **loudly** (never silently) on divergence; recovery never needs a force-push. |
| **K3** | Clean base for the next merge to `main` | After down-propagation, the lowers' graph base **is** `main`'s linear squashed history — no accumulated pre-squash funk. |
| **K4** | Easy down-propagation after a `main` merge | The propagate-down step is a short, scriptable, idempotent sequence. |
| **K5** | Respects mitigation #6 | **No** `--force` / `--force-with-lease` / `+refs` on any branch — or, if a bounded history reset is required, it is **explicitly flagged as a maintainer policy relaxation**, never smuggled in as "just a reset." |

---

## §5 Candidate patterns

Each: mechanics · down-propagation after a `main` merge · force-push / policy implication.

### A — Baseline: persistent tiers plus `--no-ff` merge-down *(status quo)*

- **Mechanics.** As §2. `dev`/`integration` persist and accumulate; after each squash, merge `main`
  down with `--no-ff` (`scripts/sync-heads.sh`).
- **Down-propagation.** `git merge --no-ff origin/main` into each lower; push. Always works, no force.
- **Force/policy.** **None** — fully compliant.
- **Verdict.** Satisfies K1/K2/K4/K5 but **fails K3** — it *is* the funk. Honest, robust, ugly.
  The correct answer *iff* the maintainer forbids any history reset (see §7).

### B — Reset/recreate the lowers from `main` after each release

- **B1 in-place force-reset:** `git branch -f integration origin/main` (or `git reset --hard`) plus a
  **force-push**. Clean base, trivial. **Violates K5 (mitigation #6) outright.** Listed only to reject
  it explicitly — *not* recommended, *not* silently assumed available.
- **B2 sanctioned delete-and-recreate:** at a **quiescent** boundary (no un-drained WIP on the tier),
  a maintainer/admin **deletes** the protected branch and **recreates** it from `main`
  (`git push origin --delete integration`, then `git branch integration origin/main`, then
  `git push origin integration`), briefly lifting branch protection. Clean base.
- **Down-propagation.** Not a *merge* — a *recreate*; the new branch **is** `main`'s tip, clean by
  construction.
- **Force/policy.** B2 is **not a `--force`/`+refs` push** (it is a delete plus fresh create), but it
  **is a rewrite of a protected branch's published history**, so it is morally in the same family and
  **must be a maintainer-authorized, bounded exception** (§9). Requires a quiescent boundary: any
  in-flight work targeting the tier must be squashed up or parked on an ephemeral branch **first**,
  or it is lost.

### C — Ephemeral release-candidate cut fresh off `main` per cycle

- **Mechanics.** `integration` **stops being a long-lived branch.** Each release cycle cut a fresh
  `rc/<cycle>` off `main`, promote `dev`'s ready work into it, run the full gate, squash to `main`,
  then **delete** `rc/<cycle>`. Next cycle's RC is born off the *new* `main`.
- **Down-propagation.** **None needed for the staging tier** — it is born clean off `main` and dies
  at the squash; there is nothing to propagate *into*. Only `dev` remains to handle.
- **Force/policy.** **No force-push, no history rewrite.** Changes the tier *model* (integration is no
  longer a persistent protected branch) — a policy change, but a benign one.

### D — Trunk-based: drop the persistent lowers entirely

- **Mechanics.** Abolish long-lived `dev`/`integration`. Feature/`wave` branches cut off `main`, PR
  (squash) to `main`, deleted after merge. `main` is the only persistent branch. A per-PR (or a
  short-lived per-batch `rc/*`) gate stands in for the integration tier.
- **Down-propagation.** **Non-existent by construction** — every new branch is cut fresh off the
  latest `main`; there are no lowers to diverge. Fully eliminates the problem.
- **Force/policy.** No force-push ever. But it **drops the two-stage staging buffer** the current
  process (and the Wave-N head-branch model, the swarm octopus-into-`dev` pattern) is built around —
  the largest process change on offer.

### E — Fast-forward-only "clean-pointer" tiers *(the §6 trick, generalized — and its limit)*

- **Idea.** Keep `dev`/`integration` persistent but forbid them from ever *carrying* the squashed
  commits: all WIP/octopus merges live on ephemeral `wave/*`/`leaf/*` branches; the persistent tiers
  advance **only** by `merge --ff-only origin/main`. Then, per C2, ff always succeeds — clean base,
  no force.
- **The limit (from §3-C1).** This works for a tier **only while that tier holds no released commit**.
  The *staging* tier's whole purpose is to hold the assembled RC that gets squashed, so it **cannot**
  be ff-only; E for `integration` **collapses into C** (the RC must be ephemeral). E is viable for a
  tier that is a *pure pointer* (e.g. a `dev` that never itself accumulates release commits — those
  live on `wave/*`), but a `dev` that squash-absorbs feature branches between releases *also* gains
  commits `main` lacks and **also** loses ff at the boundary (C1 again), so that `dev` needs a
  boundary reset (B2) too. **E is not a standalone escape; it is the ff-only *discipline* that,
  applied to genuinely-pointer branches, makes C/B2 conflict-free.**

**No pattern escapes §3.** C, D, and B2 all achieve K3 by the *same* underlying move — *don't let the
persistent branches accumulate the squashed lineage* (C/D remove the accumulation structurally; B2
clears it at each boundary). A/B1 are the two ends that fail (A fails K3; B1 fails K5).

### §5.1 Comparison table

| Pattern | K1 conflict-free | K2 robust (loud) | K3 clean base | K4 easy down-prop | K5 no-force / policy | Net |
|---|---|---|---|---|---|---|
| **A** merge-down (status quo) | Yes | Yes | **No** (funk) | Yes (`--no-ff`) | **Compliant** | Honest but funky |
| **B1** force-reset | Yes | Weak (silent overwrite) | Yes | Yes | **Violates #6** | **Rejected** |
| **B2** sanctioned recreate | Yes | Yes (quiescence guard) | **Yes** | Yes (recreate) | **Needs §9 grant** | Recommended path (dev) |
| **C** ephemeral RC | Yes | Yes (ff/delete) | **Yes** (staging) | **N/A — born clean** | **Compliant** | Recommended (staging); zero-policy partial win |
| **D** trunk-based | Yes | Yes | **Yes** | **N/A** | **Compliant** | Strictly cleaner; drops the tiers |
| **E** ff-only pointer | Yes | Yes (loud ff-fail) | Yes*, if no accumulation | Yes (`--ff-only`) | Compliant | Discipline, not a standalone tier for staging |

*E's K3 holds only for a genuinely-pointer branch (§5-E limit).

---

## §6 Recommendation — **C plus boundary-reset `dev` (B2), governed by the ff-only discipline (E)**

**Primary recommendation, for ratification:** keep the *concept* of tiers the maintainer wants, but
**stop persistently accumulating unsquashed lineage on them.** Concretely:

1. **`main`** — unchanged: persistent, protected, **squash-only**, linear/bisectable. This is already
   correct and is the history that matters for releases.
2. **Staging tier becomes ephemeral `rc/<cycle>` (Pattern C).** Replace the persistent `integration`
   with a release-candidate branch **cut fresh off `main`** each cycle, gated in full, squashed to
   `main`, then **deleted.** It is born clean and dies at the squash — **zero down-propagation, zero
   accumulation, no force-push.** (If the maintainer prefers to keep the *name* `integration` as a
   protected branch, keep it — but **recreate it from `main` each cycle via B2** rather than merging
   `main` down into it. Same clean-base effect; see §9.)
3. **Working tier `dev`** — kept persistent (the swarm/kickoff model wants a stable working base), but
   **reset to `main` at each quiescent release boundary** via the **sanctioned B2 recreate**, so it
   re-bases clean every cycle. Between resets it accumulates WIP normally (transient — cleared at the
   next boundary). All heavy/octopus swarm work stays on **ephemeral `wave/*`/`leaf/*`** branches
   (they may be as messy/`--no-ff`/octopus as desired — they are deleted at squash, so their lineage
   never lands on a persistent branch).
4. **Below `dev`** — unchanged: ephemeral leaves in isolated worktrees (mitigation #11), octopus/
   `--no-ff` freely, squashed away.

### §6.1 Concrete mechanics (exact command sequences)

**Land a release-candidate to `main` (squash-only, unchanged):**
```
# on rc/<cycle> (or integration), full `just check` green + /pr-review clean:
gh pr create --base main --head rc/<cycle> --title "release: <curated subject>"
# then squash-merge the PR (curated subject+body; never the WIP trail) -> main tip = S
```

**Propagate down plus re-clean the tiers after `S` lands (the boundary reset — B2):**
```
git fetch origin main
# (a) staging tier: nothing to propagate — delete the spent RC; next cycle cuts a fresh one off main
git push origin --delete rc/<cycle>            # ephemeral: gone; its net change lives in S
# (b) working tier dev: sanctioned recreate onto the clean base (quiescent boundary only — see 6.2)
#     [maintainer/admin, branch protection briefly lifted for dev]
git push origin --delete dev
git branch dev origin/main
git push origin dev                            # dev now == S: clean linear base, main an ancestor
```

**Start the next cycle's work (always off the freshly-cleaned base):**
```
git fetch origin
git switch -c wave/<topic> origin/dev          # dev == main tip -> clean base by construction
# ... octopus/leaf work on ephemeral branches, PR each up to dev (squash), assemble rc/<next> off main
git switch -c rc/<next> origin/main
```

**De-conflict (the only conflict surface — an in-flight branch based on the old base):**
```
# an ephemeral wave/leaf branch cut before the reset just merges the new base in — NEVER a force:
git fetch origin main
git switch wave/<topic>
git merge origin/main            # resolve normally; a merge only ADDS a commit
git push                         # plain push fast-forwards the remote branch — no force needed
```

### §6.2 The quiescence precondition (make the reset safe)

The B2 reset of `dev` is safe **iff** at the boundary `dev` holds **no un-drained work** — i.e. every
commit on `dev` that is not on `main` has either (i) been squashed into `S`, or (ii) been preserved on
a live ephemeral branch. Enforce with a **loud pre-reset check** (never-silent, G2), e.g.:
```
git fetch origin
# commits on dev not in main:
git rev-list origin/main..origin/dev
# for each, assert it is reachable from some live wave/*/leaf/* branch OR is content-captured in S;
# if any is orphaned -> ABORT the reset, print the orphans, drain them first.
```
Wire this into the recreate step so a reset can never silently discard work (the durability twin of
mitigation #9). *(FLAG-4: propose adding `scripts/checks/boundary-reset-guard.sh` for this — an
idempotent pure-read guard alongside `base-guard.sh`.)*

### §6.3 Migration from today's actual state (`dev e9932234`, `integration 06a1c980`, `main 707c0559`)

The existing 2076-commit divergence **cannot be removed by any merge** (§3) — a merge only *adds*.
Reaching the clean-base end-state is a **one-time sanctioned transition**:

1. **Drain the deltas up.** The residual content lives in the small diffs of §2 — `integration` vs
   `main` (3 files / 131 lines) and `integration` vs `dev` (27 files / 2802 lines). Land them onto
   `main` through the normal squash-PR path (RC off `main`, gate, squash), so `main` holds everything
   worth keeping. Park anything not release-ready on a live `wave/*` branch.
2. **Confirm quiescence** (§6.2): `git rev-list origin/main..origin/dev` and
   `git rev-list origin/main..origin/integration` contain only drained/captured commits.
3. **One-time B2 recreate** (maintainer/admin, protection briefly lifted): delete plus recreate `dev`
   and `integration` from `main`. From here the §6.1 discipline keeps them clean forever.

**Honest caveat:** step 3 is the sanctioned history reset — the migration itself **forces the §9
policy decision**; there is no force-push-free, reset-free path from *today's already-accumulated*
divergence to a clean base (a merge-down would only add more funk). This is stated plainly, not
finessed (VR-5).

### §6.4 The seam to the forward-development flow (where this DN plugs in)

This DN owns **merge topology + down-propagation only**; the *forward* methodology (API-first design,
per-change sizing ≈1–2k LOC, auto-gen-doc precursor branches — a sibling design note) **consumes** this
pattern as an input. The coupling is a **single, named seam**:

> **A change lands *up*; `main` then flows *down*, giving the next change a clean base.**

Concretely, one turn of the loop is: *(forward-dev owns)* a bite-sized unit is designed and built on an
ephemeral `wave/*`/`leaf/*` branch → *(this DN owns)* it PRs **up** (leaf → `dev`, squash) and, when a
release cut is due, `rc/<cycle>` → `main` (squash) → **the boundary propagate-down (§6.1) re-cleans
`dev`** → *(forward-dev owns)* the **next** unit branches off the freshly-cleaned `dev` (== `main`) and
repeats. The two flows touch **only** at that handoff: this DN guarantees the **base** the forward flow
starts each unit from is `main`'s clean linear tip; the forward flow decides **what** that unit is and
how big. Neither needs to know the other's internals.

**Why this pattern is exactly what a small-bite-sized-PR forward flow wants** (K-criteria, made
forward-facing):

- **Every upward PR is small and independent by construction.** Because all real work lives on
  ephemeral branches and lands via squash, a ≈1–2k-LOC change is one leaf branch → one squash commit
  on `dev`/`main`. The pattern imposes **no** lower bound on PR size and **no** cross-PR entanglement
  on the persistent tiers — the DN-65 scoped-PR decomposition drops straight in.
- **Each landed PR leaves a clean base for the next.** After a squash, `dev` is re-cleaned to `main`'s
  tip (§6.1), so the *next* unit's diff is exactly its own change against the release base — no stale
  base (mitigation #13), no 2076-commit funk to diff against, no accidental in-flight-wave revert.
  This is the precise property an auto-gen-doc precursor branch and an API-first "design-then-fill"
  cadence rely on: the precursor and the fill both branch from a known-clean `main`-equal base.
- **Down-propagation never blocks the forward cadence.** The propagate-down is a delete-of-ephemeral
  (RC) plus a loud `--ff-only`/sanctioned-recreate (`dev`) — seconds, scriptable, idempotent (K4). It
  runs *between* units at the release boundary, not inside a unit, so it never stalls an in-flight PR.

**Non-goals (explicitly out of scope, owned by the sibling forward-dev DN):** API-first design method,
per-change LOC sizing, auto-gen-doc precursor-branch mechanics. This DN only *names the seam* and
guarantees the clean base on the near side of it.

### §6.5 Failure modes it prevents

- **Unbounded graph balloon** (§2) — removed at the source: persistent branches never accumulate
  cross-release lineage (K3).
- **The force-push temptation** (mitigation #6) — removed: in steady state there is *nothing to
  force* (the lowers never diverge, so no one ever wants to rewrite them); the only reset is the
  bounded, authorized boundary recreate, explicitly carved out (K5).
- **Silent stale-base / divergence** (mitigations #7/#13) — the ff-only discipline (E) turns any
  accidental divergence into a **loud `--ff-only` failure**, and §6.2's guard turns an unsafe reset
  into a **loud abort**, never a silent drop.

---

## §7 Honest trade-offs (no sycophancy — house rule #4)

- **The recommendation is not free: it requires a sanctioned history reset.** C's ephemeral RC needs
  no rewrite, but resetting the persistent `dev` (and, if kept-by-name, `integration`) *does*. That is
  a **real relaxation of the spirit of mitigation #6** — even though B2 is a delete-plus-recreate
  rather than a `--force` push, it rewrites a protected branch's published tip. If the maintainer's
  rule is "**no rewrite of a protected branch's published history, ever, by any mechanism**," then §6
  is **not available**, and the **honest recommendation collapses to Baseline A** (§5): the funk is
  *inherent* (§3) and the correct posture is to **stop treating `git log --graph` on the lowers as a
  defect** — `main` stays clean and bisectable (the history that ships), the lowers stay
  content-correct, and the graph noise is cosmetic. That is a legitimate, fully-compliant choice; it
  simply does not deliver the clean-base end-state the maintainer asked for. **You cannot have both**
  (§3) — this is the trade the maintainer must make, and pretending otherwise would be the exact
  ungrounded-affirmation defect house rule #4 forbids.
- **Lineage moves off the persistent branches.** Under C/D and the boundary reset, the messy/octopus
  swarm lineage lives on **ephemeral** `wave/*`/`leaf/*` branches and is **discarded at the squash**.
  The permanent record of a change becomes **the squash commit on `main` plus the PR (its diff and
  resolved review threads)** — not an unsquashed WIP trail on `dev`. This is a genuine loss *if* you
  value permanent, browsable pre-squash lineage on a protected branch. The judgement in §6 is that
  this loss is small — the PR plus curated squash already preserve the *net* change and the review
  history, and the unsquashed WIP on a protected branch is exactly the "funk" being removed — but it
  **is** a loss, stated as one.
- **Process change cost.** C changes what `integration` *is* (persistent to ephemeral RC, or
  recreated each cycle); D is a larger change (drop the persistent lowers). Both ripple into the
  Wave-N head-branch model, `scripts/sync-heads.sh`, and the kickoff docs. Non-trivial, but bounded
  and one-time.
- **D (trunk-based) is *strictly cleaner* than the recommendation** for the stated goal — it deletes
  the problem rather than managing it — and is a fair choice if the maintainer is willing to drop the
  two persistent lower tiers. It is **not** the primary recommendation only because the maintainer's
  framing ("the project uses tiered branches … going forward") signals a desire to *keep* the tier
  concept. If that preference is soft, **D should be reconsidered on its merits** — it is the
  theoretical optimum here.

---

## §8 Adversarial stress-test of the recommendation

- **S1 — someone commits directly to `dev` (bypassing the ephemeral-branch route).** `dev` gains a
  commit `main` lacks, so the next `--ff-only origin/main` **fails loudly** (K2). *Guard:* `dev` is
  protected/PR-only (already), so the direct commit is blocked upstream; if it happens anyway, recovery
  is squash-it-up-then-recreate — **never a force** (mirrors `CLAUDE.md` §6 "diverged branch ->
  re-branch from `main`"). **Survives.**
- **S2 — an in-flight ephemeral PR was based on the pre-reset `dev`.** Its base (old `dev` == old
  `main`) is an **ancestor** of the new `main`, so it replays cleanly; if it touched the same files as
  the release, resolve by **merging `main` in** (§6.1 de-conflict) — a plain push, no force.
  **Survives.**
- **S3 — two releases in quick succession.** Each: squash, delete RC, recreate `dev`. No accumulation
  between them; the balloon can never re-form. **Survives.**
- **S4 — the boundary reset would orphan un-drained work.** §6.2's guard **aborts loudly** and prints
  the orphans; the reset is blocked until they are drained. No silent loss (K2, durability). **Survives.**
- **S5 — what tempts a force-push here?** Only the desire to *rewrite* a diverged persistent tip. The
  pattern **removes the divergence** (steady-state lowers are always ancestors of `main`), so the
  temptation never arises; the sole reset is the *authorized* boundary recreate. The one residual
  force-temptation — the **one-time migration** from today's 2076-commit divergence — is met by the
  **sanctioned** B2 recreate (§6.3), explicitly flagged as the policy decision (§9), not smuggled as a
  routine push. **Survives, with the caveat named.**

The recommendation survives every sequence I could construct **provided** the §9 policy relaxation is
granted; without it, §7's honest fallback (Baseline A) is the answer.

---

## §9 The single policy decision the maintainer must make

> **Ratify (or reject) a bounded, sanctioned "release-boundary recreate" of the lower tiers —
> a maintainer/admin branch delete-and-recreate from `main` at a quiescent boundary, explicitly
> carved out of mitigation #6 as *not* a routine force-push — in exchange for lowers that always lay
> cleanly on `main`.**

- **Rule it IN** then adopt §6 (ephemeral RC plus boundary-reset `dev`, ff-only discipline). Clean
  base, no routine force-push, the funk gone; cost = one authorized reset per release plus lineage on
  ephemeral branches (§7).
- **Rule it OUT** (no rewrite of a protected branch by *any* mechanism) then the funk is **inherent**
  (§3); adopt Baseline A honestly and **stop treating the lowers' graph noise as a defect** (`main`
  is what ships and it stays clean). Optionally soften by making `integration` an **ephemeral RC
  anyway** (Pattern C needs *no* reset — it is delete-of-an-ephemeral-branch, always allowed), which
  removes the funk from the *staging* tier for free and leaves only `dev` accumulating.
- **Middle option** then **Pattern C alone** (ephemeral RC, keep `dev` as-is under Baseline-A
  merge-down): removes the staging-tier funk with **zero policy relaxation**, at the cost of `dev`
  still ballooning. This is the **lowest-risk partial win** if the maintainer wants improvement
  without any §9 grant.

This note recommends **rule-it-IN plus §6**, with **Pattern C alone** as the no-policy-change fallback
and **Baseline A** as the honest do-nothing floor.

---

## §10 Never-silent guarantees carried by the recommendation (G2)

- Every down-propagation is either a **delete-of-ephemeral** (RC) or a **loud `--ff-only`** (pointer
  tiers) — a divergence surfaces as a **failed ff**, never a silent papered-over merge.
- The boundary reset is **gated by §6.2's quiescence guard** — an unsafe reset **aborts and prints the
  orphaned commits**, never drops work silently (durability, mitigation #9).
- The sanctioned reset is **authorized and logged** (a maintainer action with protection briefly
  lifted), not a routine push — the exception is explicit and auditable, per the §9 grant.

---

## §11 User stories

- **As the maintainer,** I want the lower tiers to lay cleanly on `main`'s linear history after each
  release, so the next release is cut from a clean base with no accumulated graph funk — and I want to
  be told plainly that achieving this costs one authorized boundary reset per release (not hidden
  behind a "just a merge" euphemism).
- **As a swarm/kickoff agent,** I want to branch my work off a `dev` that is a clean pointer at `main`,
  so my leaf's diff is exactly my change against the release base — never an accidental revert of an
  in-flight wave (mitigation #13) and never a 2076-commit-divergent base.
- **As a reviewer,** I want the permanent record of a change to be its curated squash on `main` plus
  its PR (diff plus resolved threads), so history stays bisectable and legible — not an unsquashed WIP
  trail duplicated on a protected branch.

---

## Definition of Done

This DN is **Draft**. It is **Accepted** when the maintainer rules on §9 (adopt §6, Pattern-C-only,
or Baseline-A-honestly). It is **Resolved** when, after acceptance:

1. the ratified pattern is written into `CLAUDE.md` (§Commits & PRs, §Autonomous PR workflow item 6,
   §Wave-N) and `.claude/kickoffs/README.md`'s tier table plus down-propagation note, **superseding**
   the current `--no-ff` merge-down wording (append-only; a dated amendment, never a rewrite) —
   integrator/maintainer-owned (FLAG-1); **and**
2. if §6 is adopted: `scripts/sync-heads.sh` is updated (or replaced) for the ephemeral-RC plus
   boundary-reset flow, and the §6.2 quiescence guard (`scripts/checks/boundary-reset-guard.sh`) is
   added (FLAG-4); **and**
3. the one-time migration (§6.3) is executed at a quiescent boundary and recorded.

A maintainer amendment to the direction supersedes by dated section — never a rewrite (append-only).

---

## FLAGs (up to the integrating parent — not edited from this leaf)

| FLAG | What it is | Who |
|---|---|---|
| **FLAG-1** | **`CLAUDE.md` plus `.claude/kickoffs/README.md` policy edit.** After ratification, supersede the `--no-ff` merge-down wording with the chosen pattern (append-only, dated). Maintainer/integrator-owned — not touched from this branch. | Maintainer / Integrator |
| **FLAG-2** | **`docs/Doc-Index.md`** — register the DN-95 row. Integration-tier owned — not touched from this leaf. | Integrator |
| **FLAG-3** | **`CHANGELOG.md`** entry for DN-95 (Added — Draft note). Integration-tier owned — not touched from this leaf. | Integrator |
| **FLAG-4** | **New guard script** `scripts/checks/boundary-reset-guard.sh` (§6.2) — propose only; author on acceptance of §6. | Integrator (on acceptance) |
| **FLAG-5** | **DN slot verification.** DN-95 verified free at authoring (no `docs/notes/DN-95-*`, no `DN-95` in `issues.yaml`/`CHANGELOG.md`/`docs/`, 2026-07-09 — mitigation #1); integrator re-verifies at merge before registering the Doc-Index row. | Integrator |

---

## Meta — changelog

- **2026-07-09 — Created (Draft; DN-mergepattern-eval work-up).** Models the current
  `dev → integration → main` squash-and-merge-down pattern (§2, `Empirical`: `integration` 2076
  commits ahead of `main` by graph vs. a 3-file/131-line content diff — the observed "funk"), proves
  (§3, `Proven`-within-its-model) that squash-only **and** no-force-push **and** persistent
  accumulation tiers makes graph divergence **mathematically unavoidable**, and enumerates five
  candidate patterns (§5: baseline merge-down, force-reset (rejected — mitigation #6), ephemeral RC,
  trunk-based, ff-only clean-pointer) with a comparison table (§5.1). **Recommends (§6):** ephemeral
  release-candidate for the staging tier plus boundary-reset `dev` from `main` via a **sanctioned,
  bounded delete-and-recreate** (an admin history reset carved out of mitigation #6, *not* a routine
  force-push), governed by the ff-only discipline — with exact command sequences, a quiescence guard
  (§6.2), a migration path from today's diverged state (§6.3), and an adversarial stress-test (§8).
  **The single policy decision** (§9): ratify or reject the boundary-reset carve-out — rule-it-IN
  gives §6; rule-it-OUT gives Baseline A honestly (the funk is inherent; `main` is what ships and
  stays clean); **Pattern-C-only** is the zero-policy-change partial win. Trade-offs surfaced without
  sycophancy (§7): the recommendation costs a sanctioned reset per release and moves lineage onto
  ephemeral branches; trunk-based (D) is *strictly cleaner* if the persistent lowers can be dropped.
  Enacts nothing; moves no other doc's status; recommendation tags `Declared`-with-argument, the DAG
  finding `Proven`-within-model, in-repo state `Empirical`. **Ratification by the maintainer required
  to move Draft to Accepted (house rule #3, append-only).** `CHANGELOG.md` / `Doc-Index.md` /
  `issues.yaml` / `CLAUDE.md` edits are integration/maintainer-tier owned (FLAGs 1–5).
  (Append-only; VR-5; G2.)
