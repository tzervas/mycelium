# Kickoff `e7l` — E7 Language Completeness (`mycelium-l1`)

> Stowed kickoff, UID **`e7l`**. A parent session for the L1 language-completeness task set.
> Read `.claude/agent-context.md` + `CLAUDE.md` first (house rules win); this file adds the specifics.

## ⚡ RESUME HERE (updated 2026-06-22 — read this first)

**Branch fresh off `main`.** `main` already carries the M-666 `hypha`/`colony` foundation + the whole
post-1.0 wave (tip was `5313964` "dfr discharge" before this session). **This session's e7l first
tranche + the depth-safety architecture LANDED to `main` via squash PR** (2026-06-22 — see "Done so
far"). So on resume: `git fetch origin main` → branch a fresh working branch off `origin/main`
(`git checkout -b claude/<desc> origin/main`); everything below is already on `main`. `main` is PR-only;
your working branch squash-PRs to `main` per logical unit.

**Continue the chain at M-659** (the next un-done item).

**Done so far (LANDED to `main` this session, 2026-06-22):**
- ✅ **M-656** — RFC-0007 §11 (generics deferral discharged → RFC-0019).
- ✅ **M-657 checker** — unbounded parametric generics type-check (`Ty::Var`, applied `Ty::Data`,
  unification-based call-site instantiation, arity/undetermined/repr-op refusals). **Elaboration of a
  generic instantiation is STAGED** as an explicit `Residual` → **M-673** (monomorphization follow-up).
- ✅ **M-658** — RFC-0007 §12 trait surface + **`impl` reserved** (`Tok::Impl`, reject-corpus
  `14-impl-reserved-ident.myc`). Trait *checker* is M-659 (next).
- ✅ **Depth-safety / limit-point discipline (M-674):** explicit budgets on all 4 L1 passes (parser
  256, checker `MAX_CHECK_DEPTH=4096`, elaborator `MAX_ELAB_DEPTH=4096`, evaluator `DEFAULT_DEPTH=64`
  now host-stack-safe); checker/elaborator/evaluator run on a **deep worker stack** in the new
  **`mycelium-stack`** crate (isolated outside the kernel); kernel is **`#![forbid(unsafe_code)]`**
  (machine-proven). Measured checker ceiling ~24,600 levels (debug). **M-673** (monomorphization) +
  **M-674** (remaining: totality/ambient budgets + cross-crate audit — evaluator item DONE) filed.

**Next un-done: M-659 (traits checker)** — a large atomic unit: AST `Item::Impl` + parser productions
for `impl Trait for T { … }` and bounded type-params `T: Trait` (RFC-0019 §4.1 — do not yet exist),
then trait-declaration + impl-block checking with **coherence** (orphan rule + global uniqueness,
RFC-0019 §4.5), then dictionary-passing **typing** (L0 lowering STAGED like generics → M-673). Do NOT
half-land it (parse-but-skip-in-checker = a silent no-op, G2 violation). Then M-660 → M-661 → M-662 →
M-663 → M-664 → M-667/M-668 → M-649.

> **Lesson recorded:** the original brief named a protected head `claude/head/e7-language`; in practice
> a single working branch off `main`, squash-PR'd per tranche, worked cleanly (no separate head
> needed). The L1 collision-serialization (token/parse/checkty/elab one editor at a time) held; a
> *disjoint* file (e.g. `eval.rs`) can run as a parallel leaf alongside the serial chain.

## Mission
Drive **E7-1** (L1 Stage-1 language completeness) + **E7-2** (runtime constructs) + **M-649**
(self-hosting Stage-2) to done. Dependency-ordered:

| # | Issue(s) | What |
|---|---|---|
| 0 | pull-down | ✅ done — `main` carries the M-666 `hypha`/`colony` foundation + post-1.0 wave. |
| 1 | M-656 → M-657 | ✅ generics: spec done; **checker** done (elab staged → M-673). |
| 2 | M-658 → M-659 | ✅ M-658 (spec + `impl` reserved) done; **M-659 trait checker = NEXT**. |
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
