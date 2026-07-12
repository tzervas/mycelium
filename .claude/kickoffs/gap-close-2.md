# Kickoff: gap-close-2

> Instigator for the wave AFTER `gap-close-run` (which released 2026-07-12, main `97de2ac9`: M-1060
> cross-phylum SOUND + facility Stages 0–3 + closure-emit + numerics + DN-119…123 + security backlog +
> live docsite). This wave: **re-measure honestly, close the `checked_fraction` levers, complete the
> language, then the transpiler in earnest — toward self-host.** Reloads clean CLAUDE.md + tool policies.

## First actions on kickoff
1. **Read the durable state** (`/root/.claude/projects/-root-git-isolated-mycelium/memory/`), in order:
   - `session-program-state.md` — LIVE handoff (the ⭐-ranked blocks; the newest = the `gap-close-run`
     release). **Verify trunk tips with `gh api repos/tzervas/mycelium/git/ref/heads/{main,integration,dev}`
     — real GitHub, not local; the local checkout lags because landings go through the GitHub PR API.**
   - `kernel-unfrozen-zero-handport.md` — the design foundation (all-layers-writable, sugar-as-macros,
     map-the-PROBLEM-to-a-native-SOLUTION, mechanical-lowering-sugar as the default gap-closer).
   - `loose-typing-dx-mode-direction.md` — the DX/gradual-typing + Python-via-py2rust direction
     (**Rust-native FIRST**; Python is later, gated on py2rust maturing).
   - `security-scanning-hardening-direction.md` — backlog: value-semantics-aware scanning (M-1073) +
     extensible CPU(SHA-NI/AES-NI/AVX)+GPU(5080) hashing/crypto accel (M-1074); Rust baseline done (M-1075).
   - `sanctioned-paths-blocked-ops.md`, `delegate-nested-swarms-for-efficiency.md`,
     `workspace-tools-publish-contract.md`, `local-swarm-rag-docs-vision.md`,
     `agents-prepare-orchestrator-merges.md`.
2. **Verify the follow-up issues exist** (`M-1076`/`1077`/`1078` filed in `gap-close-run`) and grep
   `issues.yaml` for the current lever set before assigning IDs.

## The program (drive order)

**0. RE-MEASURE FIRST — the single highest-value diagnostic.** The last honest `checked_fraction` was
   **~7.8%, measured PRE the kernel/facility/cross-phylum waves** — certainly higher now, but **unmeasured**.
   Run the **Phase-0 `checked_fraction` re-measure** across the delta-ledger target set (`/transpile-vet`;
   the 17-target split). This re-baselines the whole program and tells you which gap classes actually
   dominate NOW — do not pick levers by the stale register (mit #14: the register lags the code). VR-5:
   report the honest number; do not invent a percentage.

**A. Close the `checked_fraction` levers** (measurement-ranked after step 0; current best-guess order):
   1. **External-trait impls — P1** (DN-122, **now UNBLOCKED by M-1060 landing**). The recorded top
      leverage (~15%, "poisons whole files"). Start with the prelude-scoped-coherence MVP (DN-122 OQ-1).
   2. **M-1076 — general foreign-trait/fn signature re-homing** (DN-113 §7 / DN-122). Retires the four
      narrow never-silent refusals M-1060 v1 uses (re-home foreign sigs at merge like ctor fields already
      are) — turns conservative refusals into full acceptance. Adversarially verify (the M-1060 lesson:
      this class hid a soundness hole at every step — enumerate carrier×position).
   3. **Records/named-fields — DN-123** (SMALLER than the register implies; residual = faithfulness
      [dropped names / `NamedFieldDrop`] + the self-hosted `.myc` struct-pattern surface, DN-119 L3-G1).
   4. **Bounded generics — P3** (needs design; design-reasoner Draft DN first).
   5. **Transcendentals — P6** (DN-108 Accepted, impl-open, XL) · **Never-type — P7** (DN-107, lowest).

**B. Complete the LANGUAGE, THEN the transpiler in earnest** (maintainer sequencing — core language
   fixing first, so semantics are complete + sound before the bulk transpiler work): kernel
   type-vocabulary residual (DN-121), cross-nodule runtime exec, the **DN-118 P2 value-safety contract**,
   L3 grammar residual (pattern-surface, impl-generics, general-`?`), ergonomics. **THEN** build the full
   transpiler pipeline (symbol-table + macro pass + comprehensive-emit — lever H) that leverages a complete
   language to "augment the transpilation" (map onto a complete sound value-semantics model, not around
   holes). The closure-EMIT pass already ran (DN-118 P1) — that was a targeted exception, not the bulk work.

**C. Self-host endgame.** Full self-host → dogfood the frontend through the transpiler → rip through the
   stdlib **nodule-by-nodule, phylum-by-phylum, differential-witnessed** (`/myc-drafts` graduation, the
   M-993 pattern; `/myc-dogfood` as the second witness). Long tail.

**D. Distinct tracks (queue after A/B underway; scope when ready):**
   - **Runtime & distributed concurrency** — the fungal runtime tier (hypha/colony/fuse/mesh/reclaim,
     mostly ratified names not-yet-lexed). Basis RFC-0008/0009, DN-61/63, ADR-020, DN-58/59/67.
   - **DX / QoL — M-1077**: loose/gradual typing mode (infer+hint on the fast interpreted path; strict
     typing gates COMPILE) + the lexicon-alias layer. Design-reasoner first.
   - **Python transpilation** — via the **Python→Rust→Mycelium** composition (maintainer's py2rust/
     py-rust-bridge tooling). **Explicitly AFTER Rust-native is solid + py2rust matured** — not a near-term
     lever. Its own future DN.
   - **Security + accel backlog** — M-1073 (value-semantics-aware Mycelium scanning), M-1074 (extensible
     capability-detected CPU+GPU hashing/crypto, portable fallback; first backends 14700K + 5080).

**E. Ops / hygiene (interleave):**
   - **Steady-state branch prune** (memory branch-policy): reach core-three + backup/release only; file
     the unmerged Draft DNs, land/archive the flagged branches, prune the docs-notebooklm-pdfs artifact.
   - **Docs archive + stale-vocabulary audit** (compressed `zip`, incrementally updatable, to sanitize
     tero) — audit the corpus against the ratified fungal lexicon; archive superseded material.
   - **Desktop-held heavy verification** (don't run in cloud): durability tier / mutants / fuzz, VSA/GPU,
     proof discharge (z3/LH/Lean) — collect into a desktop PR (`scripts/vsa-desktop-checks.sh`).
   - **Fleet Wave-1/2 remainder** ([[workspace-tools-publish-contract]]) — publish as GitHub-Release
     package artifacts, not containers.

## Operating pattern (carry forward from gap-close-run — it earned its keep)
- **Verify-first before building; adversarial-verify before landing.** In `gap-close-run` this caught a
  real soundness issue at EVERY semcore step — M-1060 alone took **4 fix cycles** to close one collapse
  class. For any kernel/type-identity/cross-boundary work, run the enumerate→attack→fix→re-verify loop
  until an *independent* pass confirms convergence. Prefer the general fix over whack-a-mole where feasible.
- **Cost-optimized swarms, MAX VELOCITY, saturate disjoint parallelism** — Haiku mechanical / Sonnet
  judgment / Opus deep-design+verify. Agents may spawn sub-swarms. **Set `isolation:"worktree"` as the
  Agent PARAMETER on every edit/commit agent** (not just in the prompt) — else its commit hits the
  branch-guard false-block (the `gap-close-run` integrator lesson).
- **Honest VR-5 tags, never-silent (G2), append-only, no sycophancy, no force-push.** Design-reasoners
  draft, the maintainer ratifies.
- **Protected-trunk merges are PR-only** and the **auto-mode classifier gates them** — the orchestrator's
  in-context authorization carries most; a subagent protected-merge pings the orchestrator. `main`
  squash/`integration`/`dev` merges need the maintainer's explicit "go" (they gave `protected trunk merge
  via PR` for the `gap-close-run` release). Land via `/land` / `/pr-land` / `/sync-down`.
- **Tier-scoped gates** (DN-20): `just check-canary` for leaf→dev / dev→integration (a base-crate
  `mycelium-l1` touch balloons `just check` into a multi-hour reverse-dep sweep — desktop-held). Doc-only
  reconciliations gate on the doc gates.
- **Persist durable state to memory before compaction** (mit #8/#9); update `session-program-state.md` as
  work lands so a fresh `/kickoff gap-close-2` resumes cleanly.
