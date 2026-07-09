# Design Note DN-97 — Unified Branch / Merge / Propagation Pattern (the simplest batch-up · multi-squash · plain-merge-down loop for highly-parallel AI-accelerated development)

| Field | Value |
|---|---|
| **Note** | DN-97 |
| **Status** | **Accepted** (2026-07-09; **ratified by the maintainer in-session** as **Rank 1** — same-content trunks, lightweight `main` via a `just package-release` artifact, the content-filter dropped) — establishes the **one unified** branching/merge/propagation pattern. **Enacts nothing** and **moves no other doc's status** (house rule #3, append-only). It **consumes** DN-95 (down-propagation) and DN-96 (forward/up-flow) as inputs and **reconciles** them under the maintainer's ratified decisions + three steers (§0), differing from each where those differ (§10). It **supersedes** the standalone recommendations of DN-95/DN-96 where they conflict with §0 (Rank 1 over DN-95's ephemeral-RC; the content-filter dropped) — append-only: DN-95/DN-96 are **not** edited, only referenced. All tags `Declared` unless a cited source or a verified working-tree fact holds them higher (VR-5). |
| **Decides** | *Proposes, for ratification:* the **simplest** end-to-end pattern that delivers the maintainer's objective — pace · efficiency · ease-of-maintenance · token-thrift · **simplicity above all** — for **20–30 disjoint work-sets in flight, landed in batches of 1–6**, each **squash-landing separately into `main`**, then a **cheap, conflict-free, no-force down-propagation** to two persistent lower trunks, after which in-flight work-sets **adapt** and continue. **Headline recommendation (§4, the token-thrift answer):** keep the three trunks **same-content** — tiers differ by **rigor, not content** — so the down-merge is a **plain `git merge --no-ff`, trivially conflict-free, zero merge-driver, zero filter machinery**; achieve the "lightweight production `main`" as a **release/packaging artifact** (`git archive` + `export-ignore`) that strips dev tooling **at build time**, not as divergent tracked content. The **tracked-content per-trunk filter** the brief sketched is fully worked out and **proven no-force** (§5) but is **explicitly ranked second** — it is the single biggest source of process + token overhead and is recommended **only if** production truly needs the `main` *branch itself* filtered (VR-5 / house rule #4). |
| **Consumes** | **DN-95** (the squash-divergence `Proven`-within-model funk theorem §3; `scripts/sync-heads.sh`); **DN-96** (the spec→API→component→code context-windowing pipeline, precursor doc/index branches, bite-sizing, the `/forward` skill). |
| **Feeds** | `CLAUDE.md` §Commits & PRs · §Autonomous PR workflow item 6 · §Concurrent-PR development · §Wave-N; `.claude/kickoffs/README.md`; `scripts/sync-heads.sh` (→ `/sync-down`); a proposed `just package-release` recipe (+ `.gitattributes export-ignore`) and, only if the tracked-filter is chosen, a `.tier/*` filter + `scripts/checks/tier-guard.sh`. |
| **Date** | July 9, 2026 |
| **Task** | DN-mergepattern-synthesis — unify DN-95 (down) + DN-96 (up) into one ratifiable, simplicity-ranked pattern. |

> **Posture (transparency rule / VR-5 / G2 / house rule #4).** This note **works up a decision**; it
> does not take it (house rule #3). The **crux findings in §5 are `Empirical`** — verified in a clean
> git sandbox 2026-07-09 with the exact commands shown. In-repo state (§7 SHAs) is `Empirical`. The
> pattern recommendation is `Declared`-with-argument, and it is **ranked by simplicity/token-cost
> explicitly** (§4) per the maintainer's steer. **No sycophancy:** the note does **not** preserve the
> content-filter merely because it was sketched — it says plainly that the **same-content + release-artifact**
> approach is simpler, cheaper in tokens, and has **no residual sharp edge**, whereas the tracked-content
> filter buys a literally-lightweight `main` *branch* at a real, enumerated machinery cost and one
> genuine sharp edge (a content-divergent shared file, §5.6/§9). The lower tiers' *graph* diverges
> either way (DN-95 §3, `Proven`-within-model, not repealed by any merge discipline) — an accepted cost
> (decision #5). |

---

## §0 The governing inputs (ratified decisions + three steers — these win over any sibling-DN wording)

Six ratified decisions (2026-07-09) and three maintainer steers frame this synthesis.

**Ratified decisions:**

1. **Three PERSISTENT trunks** — `dev`, `integration`, `main`; **no ephemeral RC branch** (overrides
   DN-95's primary recommendation; §10).
2. **Working branches are ephemeral, auto-pruned** on consumption by their target trunk.
3. **Up-flow is non-squash (`--no-ff`)** — working→`dev`, `dev`→`integration` — bite-sized (§2.3).
4. **Landing on `main` is squash-only** — clean/linear/bisectable; under the batch engine, **one curated
   squash per disjoint work-set** (several per batch), not one monolith (§3.2).
5. **Down-flow: merge `main` DOWN** into `integration` then `dev` (`--no-ff`, **no force**, minimal
   deconfliction). Lower-tier **graph divergence accepted** as the price of persistent trunks + no-force
   (DN-95 §3). **Force is prohibited — full stop** (no `--force`/`--force-with-lease`/`+refs`, anywhere).
6. **A lightweight production `main`** — no dev-only tooling/crap; only the tests production needs;
   `integration` intermediate; `dev` full. The brief sketched this as a persistent `.gitignore`-like
   per-trunk *tracked-content* filter updated via chore-PRs. **Steer 2 reopens the mechanism** (below).

**Three maintainer steers (governing):**

- **Steer 1 — optimize the objective, don't implement the sketch verbatim.** If a **better** way
  supports rapid AI-accelerated development on many tightly-scoped tasks, recommend **that**. Optimize
  **pace + efficiency + ease-of-maintenance + token-thrift + simplicity**; *"whatever I can do to
  simplify it is the goal"* and *"I don't wanna be wasting a bunch of tokens on an overly complex
  process."* The winner is the **simplest** pattern that hits throughput + clean-history + no-force —
  not the most elaborate.
- **Steer 2 — weigh the content-filter against simplicity (critical).** The tracked-content filter is
  the biggest source of process + token overhead. **Evaluate honestly whether it earns its keep.** If
  its machinery is too complex for the simplicity + token goal, **recommend the simpler alternative and
  say so plainly** — e.g. **same-content trunks** (trivially conflict-free down-merge, no driver) + a
  **build/release artifact** that strips dev tooling at release time. *Do not preserve the content-filter
  just because it was asked for if a simpler mechanism gives a lightweight production output with far
  less process.* (This note obeys this in §4.)
- **Steer 3 — PR size cap.** **Hard maximum 4,000 LOC total churn per PR** (additions + removals +
  inline modifications), so a human can review it later. **Soft target stays `≈1–2k` LOC** (DN-65).
  **Auto-generated bulk** (api-index / tero-index / lib-index / rendered docs) is **excluded from both**
  the soft target and the 4k cap — it rides the precursor branches (DN-96 §2.3). Encode both in the DN
  and the `/forward` size check (§2.3, §8).

The maintainer's confirmed unit is **LOC** (resolves DN-96 `FLAG-units-1`). Overriding priority:
**the simplest, most robust, most reliable, most token-light process — "I don't want to fight this."**

---

## §1 The unified pattern in one picture (recommended, same-content variant)

```
        ephemeral, auto-pruned                         persistent, protected — differ by RIGOR, not content
   ┌───────────────────────────┐          ┌───────────────────────────────────────────────┐
   ws/<a> ─┐                    │   --no-ff│  dev          (messy OK; change-scoped gate)    │
   ws/<b> ─┼─ /forward (spec→API│  staging │   ▲   --no-ff  ▲  plain merge-down (NO veto)    │
   ws/<c> ─┘  →components→code) ─┼─────────▶│   │           │                                │
     …    (disjoint by seam-map)│          │  integration  (full just check; batch polish)   │
   20–30 in flight, ≤4k LOC each│          │   │   --no-ff  ▲  plain merge-down (NO veto)    │
                                │  squash  │   ▼           │                                 │
   each work-set ──────────────┼──────────┼──▶ main  (squash-only; S_a S_b … linear)         │
     (1–6 per batch)           │  per set │        └── release: `git archive` + export-ignore │
   └───────────────────────────┘          │            ⇒ LIGHTWEIGHT production artifact      │
   forward/up = DN-96                      └───────────────────────────────────────────────┘
                                            down = DN-95 topology, PLAIN merge (crux dissolved)
```

**Tiers differ by *rigor*, not *content*.** `dev` = messy working tier (change-scoped gate). `integration`
= full `just check` + once-per-batch shared-file polish. `main` = squashed release history. **All three
carry identical tracked content**, so the down-merge is a **plain `git merge --no-ff origin/main` —
trivially conflict-free, no merge-driver, no keep-list, no filter files.** "Lightweight production" is a
**packaging output** (§4.1), not a divergent branch.

**The loop (one turn = one batch):**

1. **Develop** N disjoint work-sets in parallel, each on an ephemeral `ws/<topic>` off `dev`, driven by
   **`/forward`** (DN-96): spec → public API → private API → **component seam-map** → code. Stage-4's
   touched-file set **is** the ownership seam → work-sets are **disjoint by construction** (§2). Each is
   `≈1–2k` LOC soft, **`≤4k` LOC hard** (§2.3).
2. **Stage up** each ready work-set into `dev` (`--no-ff`); a batch promotes `dev`→`integration`
   (`--no-ff`) for the tighter gate + the **once-per-batch** shared-file reconciliation (DN-96 precursor).
3. **Land the batch** — each disjoint work-set **squash-PRs into `main` from its own branch** (§3.2):
   `main` gains **1–6 clean squashes**, linear + bisectable; each landed branch **auto-prunes**.
4. **Down-propagate** `main` → `integration` → `dev` — one **`/sync-down`** call, **plain `--no-ff`
   merge**, conflict-free, **no force** (§3.3). *(No veto in the recommended variant; §5 details why the
   tracked-filter fallback would need one.)*
5. **Adapt** — every still-in-flight work-set merges the fresh `dev` **down into itself** (`--no-ff`, no
   force; disjoint ⇒ clean) and resumes up (§3.4).

Steps 1–3 are DN-96; 4–5 are DN-95's topology. The seam between them: *work lands up; `main` flows down;
the next (and in-flight) work-sets start from the freshly-cleaned base* (resolves DN-96 `FLAG-synthesis-1`).

---

## §2 Concurrent disjoint work-sets — disjoint *by construction* (consuming DN-96)

### §2.1 The seam map makes 20–30 sets non-colliding

- **Seam map is a design output, not a merge-time discovery.** DN-96 Stage-4 derives the touched
  file/crate/dir set from the fixed public+private API **before code is written**; disjoint Stage-4 sets
  merge in parallel **conflict-free by construction** (DN-65 disjoint-ownership, lifted up-front).
  Overlap is visible *before a line is written* and resolved by ownership — the shared path **rises to
  the parent trunk**, never a merge conflict.
- **Shared surface never travels on a code work-set.** CHANGELOG, `Doc-Index`, `docs/*-index/`,
  `issues.yaml`, workspace manifests are **integration-owned**; a work-set **FLAGs** its entry up and the
  integrating step applies it once (DN-96 §2.3) — which is *why* 20–30 sets coexist.
- **`dev` is the shared base + early-collision detector**; `integration` re-gates the assembled batch
  before any squash touches `main`.

### §2.2 Why disjointness also makes the down-prop trivial

Because a landed work-set was **both** merged into `dev` (staging) **and** squashed into `main`, `main`'s
squash `S_i` carries content the lower tiers **already hold** — so the down-merge three-way sees "both
sides made the same change" → **auto-resolves clean** (verified, §5-S). With **same-content trunks**
there are **no deletions to propagate**, so the down-merge has *nothing* to conflict on: it is a plain
merge, every time.

### §2.3 Change-sizing — soft `≈1–2k` LOC, **hard 4,000-LOC cap** (steer 3)

Bind working→`dev` and `dev`→`integration` to a **soft `≈1–2k`-LOC** target (DN-65) and an **absolute
`4,000`-LOC hard cap** on total churn (additions + removals + inline modifications) so a human can review
later. **Auto-generated bulk is excluded from both** — it rides the precursor branch (DN-96 §2.3), never
the source PR. Cohesion still wins (an atomic unit whose minimal honest form exceeds the target stays one
work-set — DN-96 §5 trade-off 3 — but should still respect the 4k human-reviewability cap; if it
genuinely cannot, that is a flagged exception, never silent). `integration → main` **is not** bite-bound
(a curated release squash may be large — DN-96 §2.4), but each *per-work-set* squash is a `≤4k` unit. The
`/forward` size check (§8) warns at the soft target and **hard-stops at 4k** (source-only count).

---

## §3 The batch engine — exact command sequences

All agent git work in an **isolated worktree** (`/worktree-guard --leaf`, mitigation #11); `commit` and
`push` are **separate** commands (mitigation #12); every push is **plain** (no force).

### §3.1 Start a work-set (forward)

```
git fetch origin
git switch -c ws/<topic> origin/dev          # off the working tier, never main (mitigation #13)
# /forward CHANGE=<id> TARGET=dev  → spec → public API → private API → component seam-map → code
git push -u origin ws/<topic>                # push so sub-agents branch from a pushed tip
```

### §3.2 Stage up, then batch-land — each disjoint work-set squashes into `main` from its own branch

A GitHub squash-PR squashes a *whole branch diff*; to give `main` **one clean squash per disjoint
work-set**, the squash source must be **exactly that set's disjoint diff** — i.e. `ws/<i>`. So
`dev`/`integration` are **shared staging/polish trunks** (co-test + shared-file reconcile) and **the
canonical `main`-landing head is the work-set's own branch** (carve-out §8.2).

```
# (a) STAGE for co-integration (--no-ff; branch is NOT consumed — it lands on main later):
gh pr create --base dev --head ws/<topic> --title "stage(<i>): <subject>"     # /pr-land TARGET=dev
# (b) promote the batch dev -> integration (--no-ff), tighter gate, reconcile shared files ONCE:
gh pr create --base integration --head dev --title "stage(batch-<n>): assemble + reconcile"

# (c) LAND each disjoint work-set into main, squash-only, one PR per set (1–6 per batch):
git fetch origin main
git switch ws/<topic>
git merge --no-ff origin/main            # adapt onto current main (no force); disjoint => clean
git push origin ws/<topic>
gh pr create --base main --head ws/<topic> --title "feat(<i>): <curated net-change subject>"
#     >>> squash-merge the PR (curated subject+body; never the WIP trail) => main gains S_i <<<
git push origin --delete ws/<topic>      # decision #2: ephemeral branch auto-prunes on landing
git branch -D ws/<topic>
#     Repeat for the next set (pull main down first — a no-op ff over unrelated paths for disjoint sets).

# (d) shared-surface CLOSE-OUT lands as its own squash (disjoint from all code sets — CHANGELOG/indices
#     /issue close-out), from closeout/<batch> off integration:
gh pr create --base main --head closeout/<batch> --title "docs(batch-<n>): changelog + indices + close-out"
```

`main` after the batch: `… S_a S_b S_c S_docs` — a linear run of curated squashes (decision #4).

### §3.3 Down-propagate — `main` → `integration` → `dev` (the `/sync-down` skill), **plain merge, no force**

Run after **every** batch. **Recommended (same-content) variant — no veto, no keep-list:**

```
git switch integration                       # isolated worktree; landed via the sanctioned sync (§8)
git fetch origin main
git merge --no-ff origin/main -m "chore(sync-down): main -> integration"
git push origin HEAD:integration             # plain push

git switch dev
git fetch origin integration
git merge --no-ff origin/integration -m "chore(sync-down): integration -> dev"
git push origin HEAD:dev
```

That is the **entire** down-prop — a plain merge per tier. It is conflict-free because trunks are
same-content and the squashes carry already-staged content (§2.2). *(The tracked-filter fallback adds a
veto step here; §5.4.)*

### §3.4 In-flight adapt — a still-open work-set re-bases onto the fresh `dev`, no force

```
git switch ws/<other>                    # its own worktree
git fetch origin dev
git merge --no-ff origin/dev             # pull the new base down; NO force; disjoint => conflict-free
git push origin ws/<other>               # resume working up
```

A batch of 6 landing while 14 adapt: each of the 14 runs these three commands once; disjoint ⇒
conflict-free except a genuine shared-surface touch (already routed to the parent, §2 — rare, small,
never a force). See S6 (§6).

---

## §4 The content-filter, weighed against simplicity (steer 2) — ranked recommendation

Two mechanisms deliver a "lightweight production `main`." Ranked by the maintainer's objective
(simplicity · token-thrift · robustness):

### §4.1 **RANK 1 (recommended) — same-content trunks + a release/packaging artifact**

**Mechanism.** All three trunks track **identical content**. Production lightness is a **build step**:
mark dev-only paths `export-ignore` in `.gitattributes`, and ship via `git archive` (or a
`just package-release` recipe) which **omits** those paths from the released tarball/spore:

```
# .gitattributes
tools/**        export-ignore
xtask/**         export-ignore
**/tests/dev-only/**  export-ignore
# release:
git archive --format=tar.gz -o dist/mycelium-<ver>.tgz main    # honors export-ignore ⇒ lightweight
```

**What it costs:** one `export-ignore` list + one packaging recipe. **Zero** merge machinery.

**What it delivers against the objective:**

- **Down-prop is trivially clean** — `git merge --no-ff origin/main`, no driver, no veto, no keep-list,
  no filtered landing projection, no `tier-guard`. The **entire §5 crux dissolves.** (Token-thrift: the
  down-prop skill is ~4 git commands; nothing to reason about per file.)
- **No residual sharp edge** — there is **no** content-divergent-file class (§9), because no file
  diverges across tiers.
- **Ease of maintenance** — the exclusion list is one file, edited normally; no per-trunk chore-PR
  choreography, no "exclude from birth" discipline (§5.5).
- **Clean squashed `main`** ✓ · **no-force clean down-prop** ✓ · **lightweight production output** ✓
  (as an artifact) · **parallel batch throughput** ✓.

**The one thing it does not do:** make the `main` *branch's git tree* literally lack the tooling — the
tooling is still tracked on `main`, just **excluded from what ships**. For virtually every "lightweight
production" goal (a smaller deployed spore, a clean install, no dev tools in the shipped artifact) this
is exactly right and far simpler. This is the recommendation.

### §4.2 **RANK 2 (fallback) — tracked-content per-trunk filter + tier-veto down-merge**

**Choose this *only if* production genuinely requires the `main` *branch itself* to be filtered** (e.g. a
consumer clones `main` directly and must not receive tooling, and a packaged artifact is unacceptable).
It is **proven no-force** (§5) but is **strictly more complex and more token-heavy**, and it carries one
genuine sharp edge (§5.6/§9). Its full machinery — `.tier/<trunk>.{exclude,keep}` files, chore-PR update
path, filtered landing-branch projection, the tier-veto in every down-merge, `tier-guard.sh`, and the
"exclude-from-birth" discipline — is specified in §5 so the maintainer can see exactly what Rank 2 costs
before choosing it.

### §4.3 Explicit ranking (the maintainer's objective function)

| Criterion (steer 1/2) | Rank 1 same-content + artifact | Rank 2 tracked-filter + veto |
|---|---|---|
| **Simplicity** | **Highest** — plain merges; 1 ignore-list + 1 recipe | Lowest — filter files, veto, projection, guard |
| **Token-thrift** | **Highest** — down-prop is ~4 commands, no per-file reasoning | Low — every down-prop + every landing reasons over the filter |
| **Robustness / no-force** | ✓ (plain merge can't lose data) | ✓ (veto, proven §5) but more moving parts to get right |
| **Ease of maintenance** | **Highest** — one list, edited normally | Low — per-trunk chore-PRs, exclude-from-birth rule |
| **Residual sharp edge** | **None** | One (content-divergent shared file, §9) |
| **Lightweight `main` *branch* itself** | No (lightweight *artifact*) | Yes (branch tree filtered) |

**Recommendation: adopt Rank 1.** It hits every stated goal (throughput, clean squashed `main`, no-force
clean down-prop, lightweight production) at the **lowest** process + token cost, with **no** sharp edge.
Adopt Rank 2 only if the literal-filtered-`main`-branch requirement is real — in which case §5 is the
proven, no-force way to do it.

---

## §5 If Rank 2 is chosen — the tracked-filter crux, solved and proven no-force (`Empirical`, 2026-07-09)

*(This entire section applies **only** to the Rank-2 fallback. Under the recommended Rank-1 it is moot —
there is no filter, so no crux.)*

### §5.1 The collision Rank 2 creates

`main` is a filtered subset; a naive `git merge origin/main` **down** into `dev` (which has the tooling)
tries to **delete `dev`'s extras** — data loss, and it must be avoided **without force**. A three-way
merge deletes a path **iff** it is in the **merge-base** and deleted on `main` and unmodified on `dev`.

### §5.2 What was tested (sandbox, verbatim)

A throwaway repo: shared `src.txt`, dev-tool `tool.sh`, dev-only `dev-tests/`; `main` filtered (removed
both, changed `src.txt`), `dev` full (kept both, same `src.txt` change):

| Test | Down-merge mechanism | Outcome for the tier extras |
|---|---|---|
| **A** | `git merge --no-ff origin/main` (plain) | **DELETED** — `tool.sh` + `dev-tests/` gone (data loss). |
| **B** | `.gitattributes tool.sh merge=ours` + `git config merge.ours.driver true`, then plain merge | **DELETED anyway** — the `ours` driver fires only on a *content* conflict; a delete-vs-unmodified is a clean deletion, driver **never consulted**. |
| **C** | `git merge --no-ff --no-commit origin/main` → `git checkout HEAD -- <keep paths>` → `git commit` | **PRESERVED** — both kept, `src.txt` took `main`'s change, normal **2-parent** merge, `main` an **ancestor**, **no force**. |

**Load-bearing honest finding (Test B):** the "obvious" fix — `.gitattributes merge=ours` — **does not
work.** It intercepts content merges, not tree-level deletions; reaching for it here ships **silent data
loss**. (This alone is a strong argument for Rank 1.)

### §5.3 The tier-veto down-merge (the Rank-2 mechanism, Test C)

```
git merge --no-ff --no-commit origin/<upper>
git checkout HEAD --pathspec-from-file=.tier/<lower>.keep   # veto: restore OUR tier-specific paths
git commit -m "chore(sync-down): <upper> -> <lower> (tier-veto)"
```

`HEAD` during `--no-commit` is the pre-merge lower tip (ours); `git checkout HEAD -- <paths>` overrides
the merge's staged result for exactly the keep-list — restoring our version whether the merge staged a
**deletion** (tooling) or a **content change** (a divergent file). Plain merge + checkout = normal commit
= **plain push, no force**. Uniform, idempotent, never-silent (the keep-list is committed + reviewable;
`tier-guard.sh` re-asserts survival).

### §5.4 Why the veto is usually a no-op — and where it truly bites

Steady-state `merge-base(dev, main)` is always the last down-merged `main` tip `M_n` (an ancestor of both
`dev` and linear `main`). Since `main` is filtered **from the birth of every tooling path**, `M_n` has no
tooling → next down-merge: **base absent · ours present · theirs absent → kept** (added-on-our-side-only).
**The plain merge is already clean; the veto is a no-op** (sandbox RESULT2 confirmed: second release,
tooling not in base, clean). The veto earns its keep only at: **(1) migration** (first filtered `main`
removes tooling that *was* in the ancestor — §7), **(2) a filter change that newly-excludes an
already-present `main` path** (avoid — §5.5), **(3) accidental content-divergence**.

### §5.5 Producing `main` filtered + the `.tier/*` filter (Rank-2 only)

Two layers: **routing** (a dev-tooling work-set is its own set that never targets `main`; put tooling in
its own glob-discovered dirs/crates so tier differences are **present-vs-absent**, never divergent
manifests) + the **`.tier/<trunk>.{exclude,keep}`** files (per trunk, updated **only via chore-PR into
that trunk**, decision #6). A product landing whose diff includes an entangled excluded path is filtered
via a landing-branch projection (`git rm --cached` the `.tier/main.exclude` paths before the squash-PR).
**Discipline that keeps the veto a no-op: exclude a path from `main` from its first appearance — never
add then remove.**

### §5.6 Rank-2 residual sharp edge

A file that must carry **different content** on two tiers is the one case Rank 2 does not make free: a
`main` change to it either **overwrites** the lower tier's content or (if keep-listed) **drops `main`'s
change** — a manual, flagged reconcile. Mitigation: structure every difference as **present-vs-absent**
(§5.5) so the divergent-content set is `≈empty`. **Rank 1 has no such edge at all** — another reason it
wins.

---

## §6 Adversarial stress-test (VR-5 / house rule #4)

- **S1 — a batch of 6 disjoint squashes lands; 14 must adapt.** `main` gains `S_1..S_6`; `/sync-down`
  (plain merge, Rank 1) brings them to `integration` then `dev` clean; each of the 14 runs §3.4. Disjoint
  ⇒ 14 conflict-free merges; a genuine shared-surface set was already routed to the parent (§2). No
  force. **Survives.**
- **S2 — down-merge (the crux).** Rank 1: plain merge, nothing to delete → clean. Rank 2: veto preserves
  extras (§5.2-C); steady-state no-op (§5.4). **Survives, no force** — verified.
- **S3 — a dev-only file vs a product change.** Rank 1: no divergence exists → nothing to resolve. Rank
  2: present-vs-absent → veto keeps it; genuinely content-divergent → the §5.6/§9 sharp edge (Rank-2
  only). **Survives (Rank 1 with no edge).**
- **S4 — two sets race on the shared surface** (CHANGELOG / an index). Neither carries it
  (integration-owned, §2); the close-out squash carries the reconciled version once. **Survives.**
- **S5 — a filter update mid-cycle** (Rank 2 only). The veto keep-list restores a newly-`git rm`ed path;
  the "exclude-from-birth" rule (§5.5) makes it rare and loud, never silent. Rank 1: **not applicable**
  (no filter). **Survives.**
- **S6 — a down-merge right after a squash** (constant cadence). `S_i` carries content the lower tiers
  already staged → three-way "both sides same change" auto-merges clean. The **double representation**
  (dev holds it as `--no-ff` lineage *and* receives it as squash) is DN-95's graph funk — content-clean,
  graph-divergent, **accepted** (decision #5). **Survives; the accepted cost is restated, not finessed.**
- **S7 — what tempts a force?** Only a wish to rewrite the diverged lower-tier graph — **declined by
  accepted policy** (decision #5), not by a force. Migration (§7) is merge-only. **No force anywhere.**

**Honest verdict:** both ranks survive every sequence with **no force**; **Rank 1 additionally survives
with no filter machinery and no sharp edge**, which is why it is recommended.

---

## §7 Migration from today's diverged state (no force)

Verified 2026-07-09 (`git rev-parse origin/{main,integration,dev}`): `main 707c0559` · `integration
06a1c980` · `dev 3aa0980d`. DN-95 §3 (`Proven`-within-model): existing divergence **cannot be removed by
any merge**, and the reset is **ruled out** (decision #1/#5) — so migration **accepts** the graph
divergence and only establishes the pattern forward, **merge-only, no force**:

- **Rank 1 (recommended):** add `.gitattributes export-ignore` for the dev-only paths + a
  `just package-release` recipe. **That is the whole migration** — no branch surgery, no first-filtered
  release, no veto. Trunks stay same-content; the down-prop is already the plain merge of §3.3. Done.
- **Rank 2 (if chosen):** land the `.tier/*` files + `/sync-down` veto + `tier-guard.sh`; at the next
  batch produce filtered `main` landings (§5.5); the **first** `/sync-down` is the one moment tooling is
  in the base — the **veto restores it** (§5.4-1, sandbox-verified, no force). Thereafter clean forever.

No step uses `--force`/`--force-with-lease`/`+refs` or a delete-and-recreate.

---

## §8 Operability — the skills that make the loop near-fully automated

- **`/forward` (up — DN-96 §4, consumed).** Drives one work-set as context-light stages, then `/pr-land`
  up. This note adds its **size check** — warn at `≈1–2k` LOC (soft), **hard-stop at 4,000** LOC
  source-only churn (steer 3; auto-gen bulk excluded, rides the precursor) — and, **only under Rank 2**,
  the filtered-landing projection (§5.5). Under Rank 1 `/forward` needs no filter awareness at all.
- **`/sync-down` (down — NEW).** The single-invocation down-propagator. **Rank 1:** a plain per-tier
  `git merge --no-ff` (§3.3) + plain push — extends `scripts/sync-heads.sh` almost unchanged (it already
  does `--no-ff` `main`→head + FLAGs conflicts). **Rank 2:** add the `--no-commit` + `checkout
  --pathspec-from-file=.tier/<lower>.keep` veto + a `tier-guard.sh` survival assertion. Idempotent;
  conflict-loud; **never a force**.
- **Release packaging (Rank 1) — `just package-release`** = `git archive` honoring `.gitattributes
  export-ignore` → the lightweight production artifact.

Whole cycle: `/forward` (×N parallel) → `/pr-land` up → per-set squash-PR → **`/sync-down`** → in-flight
sets `git merge --no-ff origin/dev`. The model **invokes** the loop; it does not hold it in context.

---

## §9 The residual sharp edge (honest)

**Under the recommended Rank 1 there is no residual sharp edge** — same-content trunks mean no
deletion/divergence to reconcile; the down-merge cannot lose data. The only accepted cost is the lower
tiers' **graph** divergence (DN-95 §3), which is cosmetic (`main` ships and stays clean) and which the
maintainer has accepted (decision #5).

**Under the Rank-2 fallback**, the one genuine sharp edge is a **content-divergent shared file** (§5.6):
a single file that must exist on two tiers with different bodies needs a manual, flagged reconcile on
down-merge. It is mitigated by structuring tier differences as present-vs-absent, but it is real — and it
is a further reason Rank 1 is preferred.

---

## §10 Reconciliation with DN-95 and DN-96 (append-only; referenced, not edited)

- **DN-96 (up) — consumed whole.** Spec-first pipeline, precursor branches, bite-sizing, `/forward`
  adopted unchanged; this note adds the **4k hard cap** (steer 3) and resolves DN-96's FLAGs:
  `FLAG-units-1` → **LOC**; `FLAG-synthesis-1` → the down half is **`/sync-down`** (plain merge under
  Rank 1), and the index regenerates **once up** and rides `main` **down** (never re-generated on the
  lowers).
- **DN-95 (down) — topology consumed, recommendation *not* adopted.** Decisions #1/#5 rule out DN-95's
  primary recommendation (ephemeral `rc/<cycle>` + boundary-**reset** of `dev`). DN-97 adopts DN-95's §7
  **Baseline A** (persistent trunks + `--no-ff` merge-down, accepted graph divergence) and **simplifies**
  it: under Rank 1 the down-merge is a plain merge with **no** veto (the content-filter that would have
  needed one is replaced by the release artifact). DN-95's `Proven`-within-model funk theorem is
  **accepted**, not repealed — this note buys clean *content* and clean *operation*, not a clean lower
  *graph*.
- **On ratification** DN-97 **supersedes the standalone recommendations** of DN-95 (§6 ephemeral-RC +
  reset) and any DN-96 wording conflicting with §0 — dated, append-only; DN-95/DN-96 left intact.

---

## §11 User stories

- **As the maintainer,** I want the **simplest** possible process that lets me run 20–30 disjoint
  work-sets, land them in batches of 1–6 as separate clean squashes on `main`, and down-propagate to my
  persistent lower trunks with a plain merge and **no force** — so I get throughput without fighting the
  process or burning tokens on merge machinery.
- **As the maintainer,** I want lightweight production **without** divergent tracked content — a
  packaging step that strips dev tooling at release — so my trunks stay same-content and my down-merges
  stay trivially clean.
- **As a work-set author,** I want my Stage-4 seam map to make my `≤4k`-LOC branch disjoint from the
  other 19 in flight, so it squashes into `main` without colliding and I adapt to a landed batch with one
  `git merge --no-ff origin/dev`.
- **As an integrator,** I want `/sync-down` to be a plain, idempotent, never-silent one-shot with no
  per-file reasoning, so the constant down-prop cadence is automatic.
- **As a reviewer,** I want each work-set on `main` as its own `≤4k`-LOC curated squash, so history stays
  bisectable and humanly reviewable.

---

## §12 Definition of Done

**Draft.** **Accepted** when the maintainer (a) picks **Rank 1 (recommended) or Rank 2** for lightweight
production, and (b) ratifies the batch engine (§1/§3) — batched multi-squash landing (§8.1/§8.2), plain
merge-down (or the Rank-2 veto), and the 4k hard cap. **Resolved** when, after acceptance:

1. `CLAUDE.md` (§Commits & PRs, §Autonomous PR workflow item 6, §Concurrent-PR development, §Wave-N) +
   `.claude/kickoffs/README.md` are updated, superseding the merge-down / one-squash / size wording
   (append-only, dated) — maintainer/integrator-owned (FLAG-1);
2. `/sync-down` is built (extending `scripts/sync-heads.sh`) — plain-merge under Rank 1; under Rank 2 a
   veto + keep-list plus `tier-guard.sh` (FLAG-4); `/forward` gains the soft/hard size check and the
   per-set squash note (FLAG-5); under Rank 1, `just package-release` plus `.gitattributes export-ignore`
   land (FLAG-7);
3. the migration (§7) is executed merge-only (no force) and recorded.

A maintainer amendment supersedes by dated section — never a rewrite (append-only).

---

## FLAGs (up to the integrating parent — not edited from this leaf)

| FLAG | What it is | Who |
|---|---|---|
| **FLAG-1** | **`CLAUDE.md` + `.claude/kickoffs/README.md` policy edit** after ratification (append-only, dated). | Maintainer / Integrator |
| **FLAG-2** | **`docs/Doc-Index.md`** — register the DN-97 row. | Integrator |
| **FLAG-3** | **`CHANGELOG.md`** entry for DN-97 (Added — Draft synthesis note). | Integrator |
| **FLAG-4** | **`/sync-down` skill + `scripts/checks/tier-guard.sh`** (guard only needed under Rank 2), extending `scripts/sync-heads.sh`. | Integrator (on acceptance) |
| **FLAG-5** | **`/forward` skill note** — soft `≈1–2k` + hard 4k size check; per-work-set squash + `/sync-down` handoff; Rank-2 filtered-landing only if chosen. | Integrator (on acceptance) |
| **FLAG-6** | **DN slot verification.** DN-97 verified free at authoring (2026-07-09 — mitigation #1); integrator re-verifies at merge. | Integrator |
| **FLAG-7** | **Rank-1 packaging** — `just package-release` (`git archive`) + `.gitattributes export-ignore` list for dev-only paths (only if Rank 1 is adopted). | Integrator (on acceptance) |

---

## Meta — changelog

- **2026-07-09 — Created (Draft; DN-mergepattern-synthesis).** Unifies **DN-95** (down) and **DN-96**
  (up) into **one** ratifiable pattern under the maintainer's six decisions + three steers (§0): three
  **persistent** trunks (no ephemeral RC — overrides DN-95's primary rec), **ephemeral auto-pruned**
  work-sets, **non-squash `--no-ff`** staging, **squash-only** `main` as **N curated squashes per batch**
  (one per disjoint work-set + close-out), **no-force merge-down**, designed **around the real workflow**
  (20–30 disjoint sets, batches of 1–6, per-set squash, cheap down-prop, in-flight adapt) with **exact
  git command sequences** (§3). **Obeys steer 2 (weigh the content-filter against simplicity) and ranks
  the recommendation explicitly (§4):** **Rank 1 (recommended)** keeps trunks **same-content** — down-merge
  is a **plain `git merge --no-ff`, no driver/veto/filter** (the crux dissolves) — and delivers
  "lightweight production `main`" as a **`git archive` + `export-ignore` release artifact** (one ignore
  list + one recipe; **no residual sharp edge**, highest token-thrift + simplicity); **Rank 2 (fallback,
  only if the `main` *branch* must literally be filtered)** is the tracked-content per-trunk filter, fully
  worked and **proven no-force** (§5, `Empirical`: plain filtered down-merge **deletes** tooling (A);
  **`.gitattributes merge=ours` does NOT prevent it** (B); the **tier-veto** `git merge --no-commit` →
  `git checkout HEAD --pathspec-from-file=.tier/<lower>.keep` → commit **preserves it, no force** (C);
  no-op in steady state (§5.4)), but strictly more complex/token-heavy with one sharp edge (§5.6/§9).
  Encodes **steer 3** — soft `≈1–2k` + **hard 4,000-LOC** cap (auto-gen bulk excluded, rides the
  precursor) in §2.3 + the `/forward` check. Includes the **adversarial stress-test** (§6, incl. "6 land,
  14 adapt"), a **merge-only migration** (§7: `main 707c0559`/`integration 06a1c980`/`dev 3aa0980d`; Rank
  1 = just `export-ignore` + a recipe), **operable skills** (§8: `/forward` up + `/sync-down` down +
  `just package-release`), and reconciliation with DN-95/DN-96 (§10; resolves DN-96 `FLAG-units-1`→LOC
  and `FLAG-synthesis-1`). **Enacts nothing; supersedes the standalone DN-95/DN-96 recommendations only
  on ratification** (append-only; DN-95/DN-96 referenced, not edited). Crux + in-repo state `Empirical`;
  recommendation `Declared`-with-argument, simplicity-ranked. **Maintainer ratification required to move
  Draft to Accepted (house rule #3).** (VR-5; G2.)
