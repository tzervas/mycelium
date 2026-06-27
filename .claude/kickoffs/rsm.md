# Kickoff `rsm` — Resume development · documentation currency · future-workstream capture

> **Created** 2026-06-26, immediately after the **security-posture pass** (PR #639 — DN-44 + reliable
> `cargo deny` + advisory `just scan` hardening) landed on `main` and synced down the tiers. This
> kickoff is the **durable record** of the maintainer's forward direction (so it survives any context
> reset) **plus** the executable next wave. Fire in a fresh session with `/kickoff rsm`.

---

## ▶ Session-2 continuation (updated 2026-06-27 — READ THIS FIRST on resume)

Session-1 landed a consolidated increment to `main` (and synced down the tiers) — **do not redo it.**

**DONE in Session-1 (on `main`):**
- **W1 · M-753 width-generics — DONE** (DN-42 Option A, v1 = free functions). Width is a const-generic
  param bound at monomorphization: `Ty::Binary(Width{Lit,Var})`/`Ty::Ternary(...)`; `resolve_ty`
  maps `Binary{N}`→`Width::Var`; `unify` binds `N` **same-paradigm-only** (cross-paradigm + width
  mismatch are explicit refusals, never a swap); mono pins `N` per call (undetermined → `Residual`,
  never a default) + fragments specializations (`id_bits$Binary8/$Binary16`). Surface syntax =
  **positional-by-use** (maintainer-chosen). 11 three-way tests (`tests/width_generic.rs`) + 3
  white-box mono tests + full `mycelium-l1` suite green; clippy clean. **Unblocks M-718.** Instance
  coherence (DN-42 §7 Q5) deferred past v1.
- **W3 · capture — DONE.** F1–F7 as `Draft` stubs, registered in Doc-Index + issues.yaml (epic
  **E23-1**, **M-800–M-807**): DN-45 OSV-of-`.myc`, DN-46 honest-insecurity-disclosure gate, DN-47
  projection (vs RFC-0021), DN-48 L4/`reveal`, RFC-0036 kernel/primitives consolidation, DN-49
  post-critical quality passes, **DN-50 parsable-vs-runnable gap (F7)**. Capture-only; nothing decided.
- **Branch-protection guard — DONE** (maintainer-requested). 3 layers: `.claude/settings.json`
  PreToolUse(Bash) hook → `scripts/hooks/claude-git-branch-guard.sh`; git pre-commit/pre-push →
  `scripts/checks/branch-guard.sh`; `/branch-guard` skill + `just branch-guard`; CLAUDE.md
  **mitigation #10**. Protected branches (`main`/`integration`/`dev`/`claude/head/*`) are now
  **hard-blocked** from direct commit/merge/push (PR-only). Idempotent + parameterized.
- **W2 (partial) — DONE.** CLAUDE.md operating procedures **#8** (persist-before-compaction) / **#9**
  (commit+push-frequently) / **#10** (branch-guard); issues.yaml currency (M-753 done, M-718 ready).

**REMAINING (Session-2 scope):**
1. **W1 dev wave** (M-718 now unblocked): **M-718** width-generic `std.math`/`std.cmp` in `lib/std`
   + generalize `map_get<N>`/`set_contains<N>` off the `Binary{8}` interim → **M-719** conformance over
   the generic surface → **close M-717** (UTF-8 validity layer over the byte prims) → **re-flag M-715**
   (recursive-HOF / RFC-0024 defunctionalization — keep deferred, do NOT re-attempt this wave).
2. **W2 docs-currency sweep**: refresh `.claude/agent-context.md` + `.claude/memory/*` (esp.
   `language-execution.md` for width-generics); `just docs-index` regen (M-753 changed the public
   `Ty`); idmap reconcile; planning docs.
3. **Land** Session-2 `dev → integration → main`.

**Execution discipline for Session-2 (hard lessons from Session-1 — see CLAUDE.md #8/#9/#10):**
- **DO NOT delegate L1 surgery to leaf agents.** Session-1's leaves delegated to sub-agents that
  **orphaned** on TaskStop and raced the main tree for hours (work recovered, but very expensive). Do
  L1 checker/mono work **inline**, or via **one monitored worktree leaf** — never fan-out for
  tightly-coupled checkty/mono/parse/elab edits.
- **Stale cargo cache** during concurrent worktree builds yields false errors — confirm with md5
  stability + a cache-bust (`touch` a `src` file) before trusting `cargo check`.
- The **branch guard is active** — stay on your working branch; protected branches are PR-only.
- M-718 is `.myc` + tests (largely disjoint from L1 Rust) — lower collision risk than M-753.

---

## What this kickoff is (and isn't)

A **cross-cutting coordination kickoff**, not a single isolated tree. It sequences three concerns —
**(W1)** the immediate dev wave, **(W2)** a documentation-currency sweep, **(W3)** capture of the
larger future workstreams as DN/RFC/research stubs — plus **operating procedures** the maintainer
asked be standing policy. W1/W2 are **executable now**; W3 is **capture-only** (design/research later,
each its own decision — do **not** implement W3 items in this wave; that would over-reach and violate
append-only by pre-deciding open questions).

Branch model: per `.claude/kickoffs/README.md` — work branches **off `dev`**, promotes `dev →
integration → main`. **Sonnet swarm** default. Persistent tiers (`main`/`integration`/`dev`) are
PR-gated; `main` squash-only.

---

## Operating procedures (maintainer instruction — standing policy for every agent in this wave)

1. **Persist before you can't.** When an agent approaches context compaction, it **writes its state to
   disc** — a scratch/memory file (working notes, decisions, where it is, what's next) — so a compaction
   never silently loses the thread. (Reinforces CLAUDE.md mitigation #6; **promote this to CLAUDE.md in
   W2** so it binds beyond this wave.)
2. **Commit + push, frequently, to a working branch.** Every agent commits in small batches and
   **pushes to its working branch** (CLAUDE.md mitigation #5 `wip(batch M/N)` cadence) — so if an agent
   is lost on compaction, its work is recoverable from the branch, not gone. No agent holds hours of
   uncommitted work.
3. **Flag, don't guess** (G2/VR-5) — unchanged.

---

## W1 — Immediate dev wave: width-generics → generic math → conformance

The original "next phase", now greenlit (DN-41 + DN-42 **ratified**). Serial surgery on the L1
typechecker/monomorphizer — wants a fresh session's full context budget.

| Task | Scope | Depends |
|---|---|---|
| **M-753 width-generics** | const-generic-over-representation-width **free functions** (`fn f<N>(x: Binary{N})`), per **DN-42 Option A** (width as a const-generic param bound at monomorphization; v1 = free-fns, instance coherence deferred). Surgery in `crates/mycelium-l1/src/checkty.rs` + `mono.rs`. (`s10`/`kpr` territory — coordinate.) | DN-42 ✅, M-747/748/751 |
| **M-718 generic math/numerics** | the generic comparison/math surface + **generic-key collection lookup** (`Map`/`Set` over a width-generic `K`, replacing the `Binary{8}`-monomorphic interim from wave-n1), unblocked by M-753. `lib/std/**` + `crates/mycelium-l1/tests/`. | M-753 |
| **M-719 conformance** | conformance suite over the new generic surface — three-way (L1-eval ≡ L0-interp ≡ AOT) + never-silent refusals. | M-718 |

**Ownership:** `crates/mycelium-l1/**` (serial-on-L1), `lib/std/**`, `crates/mycelium-l1/tests/**`.
Coordinates with the `s10` (E11-1 surface) and `kpr` (E19-1 prims) heads — FLAG cross-head edits up.
**DoD:** width-generic free fns monomorphize three-way; generic math/cmp + generic-key lookup land with
honest tags + never-silent bounds; M-719 conformance green; `just check` green.

**Known flagged gaps to close as part of / alongside W1** (each currently a documented FLAG, not a
fabrication): the **recursive-HOF defunctionalization gap** (RFC-0024 — `iter` combinators
`map`/`filter`/`fold` type-check but don't monomorphize; M-715), and the **UTF-8 validity layer**
(overlong / surrogate / `> U+10FFFF` rejection for M-717 — a flagged increment, never faked).

---

## W2 — Documentation currency sweep ("bring everything up to current state + annotate the projected near-future")

A lot of state is **behind** the landed code. This wave makes the docs/tracking reflect **current
reality** and **annotates what is projected/planned** — explicitly *not* the full "massive dockery"
overhaul (partly shelved because active work keeps changing things), just **current-state + a
`projected/planned near-future` annotation** so the corpus is honest about where it is vs where it's going.

- **Update docs to current landed state** — fold in the recent waves (wave-n1, waveN2, the security
  posture), DN-41/42/43/44, M-716/717/718…, RFC statuses. Where a doc describes a not-yet-landed state,
  **annotate it `PROJECTED / PLANNED (near-future)`** rather than rewriting it as done (never claim
  ahead of basis — VR-5).
- **Regenerate the API index** — `just docs-index`, commit the delta (orchestrator-owned;
  `docs/api-index/`). Likely stale after the recent crate/test moves.
- **Reconcile task tracking** — `tools/github/issues.yaml` statuses + `idmap.tsv` ↔ live GitHub issues;
  refresh the `.claude/memory/*` orientation files where a component changed materially (they're living
  aids — keep honest + currency-accurate).
- **Promote the operating procedures** (above) into CLAUDE.md so they bind beyond this kickoff.
- **Refresh this kickoff index** + the planning docs (`docs/planning/phase-*.md`) to current state.

**Ownership:** `docs/**`, `.claude/memory/**`, `.claude/kickoffs/**`, `tools/github/**`,
`docs/api-index/` (integrator-regenerated, never hand-merged). **DoD:** docs reflect current state;
projected-state annotated, never silently claimed; API index regenerated + committed; issues/idmap
current; doc gates green (`doc-status`, `markdown`, `links`, `doc_refs_check.py`).

---

## W3 — Future-workstream capture (DN / RFC / research stubs — record now, design/decide later)

The maintainer's larger vision, captured verbatim-in-intent so it is **never lost**. Each becomes its
own DN/RFC/research item; **none is decided here** (append-only — these are open explorations, not
ratified directions). Recommended: spin these up as **`Draft` DN/RFC stubs** in W2 (so they're tracked
in `Doc-Index` + `issues.yaml`), then research/design them in later waves.

**F1 — OSV scanning *of Mycelium programs* (post-1.0 / dogfood).** Extend language/toolchain support so
`osv-scanner` (and the in-env supply-chain posture, DN-44) can scan **actual Mycelium implementations**
and give accurate security guidance on real `.myc` code. Sequencing the maintainer set: **high-level**
scanning becomes possible once the language is **fully implemented + usable** (≈ full 1.0, syntax +
low-level functionality at a decent point, even while dogfooding back into Mycelium); **low-level**
scanning lands once the language is **fully self-hosted** (written in itself). **Plan only now** — can't
be done accurately/fully until the language is there. (Complements RFC-0035, the Mycelium-native toolkit.)

**F2 — Operationalize the honest-insecurity-disclosure corollary (DN-44 §1.1).** A **standard disclosure
block** (disclaimer + reasoning/justification + program-author guidance) for every intentionally-unhardened
or unpatchable-at-language-level surface, and a **gate** asserting every `wild`/FFI/intentional-escape
surface carries one. Preference order is firm: **fix it in the language/toolchain first**; the
program-level workaround + disclosure is the fallback. → a DN (operationalization) + a gate task.

**F3 — Projection (research + DN/RFC).** Explore *what projection is* in software engineering /
programming languages and *how it works*; whether **Mycelium has it** (note: **RFC-0021 "Semantic-Level
Projections"** already exists — establish the relationship: is that the same notion, a subset, or
adjacent?). If absent: do we want it, how would we implement it, what would it comprise, the drawbacks
and considerations, and the **2nd / 3rd / 4th-and-beyond-order effects**. If present: can we **prove** it,
and make it **more economic / performant / safe / secure**? → research record + a DN; possibly an RFC.

**F4 — L3 → L4, and the "reveal" lowering (research + DN/RFC).** Consider whether an **L4** layer is
appropriate — what it would look like, what it would mean, and how layers **reveal** (the maintainer's
chosen vernacular for lowering to the next-lower level) downward. Open question: can **L4 reveal straight
to L0**? Rationale to explore: lowering is mechanized/schematized/highly repeatable, and L4/L3/L2/L1 are
**all lowered to L0** for both interpretation-time and compile-time — so a direct L4→L0 reveal may be
coherent. → DN/RFC + research.

**F5 — Kernel & primitives consolidation; possible multi-kernel split (RFC + research).** The kernel
stays **minimal**, but the maintainer's model is: **all primitives live in the kernel**, and the kernel
is the **most heavily tested / benchmarked / chaos-engineered** artifact in the project — every condition
and error mapped and handled — so it can be **frozen, locked, and pinned as the 1.0 kernel**, rarely if
ever touched again. Small *because targeted*: only primitives, and only doing in primitives what should
be done in primitives; everything else builds on top. Open structural question: **one kernel, or several**
— e.g. a `mycelium` kernel + a binary/ternary kernel + an embedding kernel + a VSA/HDC kernel? Best
approach **TBD** → an RFC + research (ties to KC-3 small-kernel, DN-39 kernel-promotion-review, the
substrate split RFC-0001/0003). This also frames the **"what must be a primitive vs built on top"**
boundary review.

**F6 — Post-critical-work quality passes (capture; sequence after the language is fully usable).** Once
the critical 1.0 work is done and the language is fully working/usable, a deliberate set of passes:
- **Testing refactor FIRST** — re-organize the suite to be **parameterized / fixture-driven / dynamic**,
  an **easy entry point** to wire in multiple test kinds + configurations (incl. **chaos testing**), and
  to surface **all the metrics / results** we want. Tie to: once the trusted core is nailed down enough to
  have **mapped every acceptable/rejecting variant & invariant**, decide **what actually needs testing vs
  not**, then reorganize/clean optimally (test-layout rules: CLAUDE.md §Test-layout; M-797 inline-test
  retrofit).
- **Language-level DRY / efficiency pass** — every place we unnecessarily repeat or redo, or aren't as
  simple/efficient/effective as our constraints allow — **while retaining tunable certification + explicit
  swap**, and making *those* easier + more organic to use, and everything more performant / secure /
  memory-safe / ergonomic.
- **Public/private + no-black-boxes** — resolve the public-vs-private testing-visibility conflict (white-box
  in-crate `src/tests/`, CLAUDE.md §Test-layout) across the board.
- **L3 review + the L4 question (F4)** as part of the same lowering-architecture pass.

---

## Swarm / ownership / sync discipline (standard)

- Fire `/kickoff rsm` in a fresh session; branch off `dev`. Sonnet swarm default.
- W1 (Rust L1 surgery) and W2 (docs/tracking) are **largely disjoint** — W1 owns `crates/mycelium-l1/**`,
  `lib/std/**`, and tests; W2 owns `docs/**`, `.claude/**`, `tools/github/**`, `docs/api-index/`. They
  **can run in parallel** save for the shared release-surface files (CHANGELOG, Doc-Index, issues.yaml,
  api-index) which the **integrator reconciles once** (CLAUDE.md §Swarm file-ownership).
- W3 is **capture-only**: stubs land in W2's docs territory; **no W3 implementation** in this wave.
- Sync discipline unchanged: free child merges → PR into `dev` → `integration` → squash-PR to `main` →
  propagate squashed `main` **down** (mitigation #6). No force pushes (CLAUDE.md).

## Definition of Done (this kickoff phase)

1. **W1**: M-753 → M-718 → M-719 landed; width-generic free-fns + generic math/cmp + generic-key lookup
   three-way green; flagged gaps either closed or re-flagged honestly. `just check` green.
2. **W2**: docs current; projected-state annotated (never silently claimed); `just docs-index` committed;
   issues/idmap/memory current; operating procedures promoted to CLAUDE.md; doc gates green.
3. **W3**: F1–F6 captured as `Draft` DN/RFC/research stubs (registered in `Doc-Index` + `issues.yaml`),
   each with the maintainer's framing preserved and marked **open / not-yet-decided** — nothing
   pre-decided.
4. Transparency/append-only survive throughout (VR-5/G2); the security posture (DN-44) is unweakened.
