# Kickoff `e7l` — E7 Language Completeness (`mycelium-l1`)

> Stowed kickoff, UID **`e7l`**. A parent session for the L1 language-completeness task set.
> Read `.claude/agent-context.md` + `CLAUDE.md` first (house rules win); this file adds the specifics.

## Head branch (your locked base)
**`claude/head/e7-language`** — branch every sub-task off it, merge every sub-task back into it; it is
the single integration point for this task set and a **protected, persistent base** (survives pruning).
Never touch other heads or `main` directly. `main` is PR-only.

## Mission
Drive **E7-1** (L1 Stage-1 language completeness) + **E7-2** (runtime constructs) + **M-649**
(self-hosting Stage-2) to done. Dependency-ordered:

| # | Issue(s) | What |
|---|---|---|
| 0 | pull-down | `git fetch origin main` → merge — **M-666 (`hypha`/`colony`) is your foundation**, already on `main`. |
| 1 | M-656 → M-657 | generics: spec → impl |
| 2 | M-658 → M-659 | traits + `impl`: spec → impl |
| 3 | M-660 | effect annotations |
| 4 | M-661 | `wild` / FFI floor (audited; std-sys) |
| 5 | M-662 | `phylum` + cross-nodule |
| 6 | M-663 | RFC-0018 static guarantee grading — **stays `Declared`** until a checked basis (VR-5) |
| 7 | M-664 | `consume`/`grow`/`impl` surface keywords |
| 8 | M-667 → M-668 | E7-2: `fuse`/`reclaim`/`tier` constructs → R2 design |
| 9 | M-649 | self-host the first stdlib nodule in `.myc` (needs E7-1; M-502 ✅) |

## Ownership
- **You own:** `crates/mycelium-l1/**`, `docs/spec/grammar/**`, and (M-649) exactly one new `.myc`
  stdlib nodule.
- **Read-only / FLAG up** (the head owner reconciles once per merge, never a leaf): `tools/github/issues.yaml`,
  `CHANGELOG.md`, `docs/Doc-Index.md`, `docs/api-index/`, workspace `Cargo.toml`.

## Swarm method — scoped to **HIGH collision → SERIALIZE the L1 files**
`token.rs`/`parse.rs`/`checkty.rs`/`elab.rs` are the collision surface — **never two leaves editing
them in parallel** (mitigation #7). Pattern: **Opus orchestrator** + **Opus** for each spec/design
step + **Sonnet** leaves for bounded impl slices, but the **L1-touching impl tasks land one at a time
in dependency order**, each pulling the head down first. Spec/doc tasks (M-656/M-658/M-660/M-663 text)
may run parallel on disjoint doc sections; the impl tasks (M-657/M-659/M-661/M-662/M-664/M-667)
serialize. Size: small, serial — *not* a wide fan-out.

## Merge / branch method
Sub-branch per task off the head → land into the head via `--no-ff` (or a leaf PR), **pull-down before
each merge-up**. When the whole chain is green on `claude/head/e7-language`, **head → `main` via PR is
the FINAL step** (a separate integration; do not PR to `main` mid-chain unless coordinated).

## Honesty / done
Every bound at its honest strength; RFC-0018 grading `Declared` until checked; never-silent
`Result`/`Option`; specs → **"implemented Rust-first, pending ratification"**, never silently
`Accepted`; a property test per bound; flag architecturally-significant choices (cf. the M-666
concurrency precedent) rather than guess. **Done** = the full E7-1+E7-2+M-649 chain green on the head,
every issue body + status updated, ready for final integration to `main`.
