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

---

## Continuation handoff (2026-06-22) — POST-COMPACTION RESUME POINT

> Read this to resume `e7l` after a context compaction. **Swarm mode: Sonnet swarm + Opus
> orchestrator** — you (Opus) orchestrate; spawn **Sonnet** leaves for ALL impl work (eases Opus
> availability/529). L1 work is **serial-on-L1** → decompose into **sequential micro-tasks**, each a
> small Sonnet leaf that pushes every commit. `git fetch origin` first.

### Branch state
- **`main`** = `5313964` (advanced — DFR landed; pull down into the head before the eventual head→main, mitigation #6).
- **`claude/head/e7-language`** (protected head) = `3917a32` — has **generics (M-656/M-657, #346)** + **Ty::App refactor (M-673, #348)**, both Copilot-reviewed.
- **`claude/keen-hypatia-bdmtt4`** (WORK branch) = `fb2ad99` — = head tip + **non-parametric traits (M-658/M-659) merged** (208 tests, `just check` green). **NOT yet landed on the head / not yet PR'd.**

### LANDED on the head
1. **Generics** (RFC-0007 §4.9; `checkty::monomorphize` — generic ADTs+fns incl. recursive/mutual, reuses Fix/FixGroup, no new kernel node KC-3; binder capture; opt-in `MYCELIUM_MONO_INSTANCE_CAP`; Declared; never-silent).
2. **`Ty::App` refactor** — abstract generic types structural `Ty::App(name, Box<Vec<Ty>>)` (not mangled strings); structural subst/unify/mention; abstract `App`/`Var` confined to the checking phase, monomorphized to concrete `Ty::Data(mangled)` before elab/eval.

### ON THE WORK BRANCH (merged, NOT landed): non-parametric traits
`Ty::Arrow(Box<Ty>,Box<Ty>)` added (grounded in RFC-0007 §4.3 kernel grammar). `Env.traits`/`Env.impls`. `trait T { fn m(x: ConcreteTy)->… }` (0-param) + `impl T for C` + bound `fn f<X:T>(x:X)`; coherence (global-uniqueness + missing/extra/mismatched-method + unknown-trait + missing-instance — all never-silent); **literal runtime dictionary-passing** dispatch (maintainer-chosen: curried-`Lam` runtime-dictionary realization; eval-path = per-instance monomorphic-dispatch oracle; L0 path = literal runtime dictionary; three-way differential proves agreement; no new kernel node). Tag Declared.

### ACTIVE TASK — extend traits to PARAMETRIC (maintainer chose this) via a Sonnet micro-task swarm
Target: `trait Cmp<A> { fn cmp(a:A,b:A)->Binary{2} }` (trait TYPE PARAM in method sigs) + `impl Cmp<Binary{8}> for Binary{8}` + bound `<X:Cmp> ⇒ impl Cmp<X> for X`. ADDITIVE (0-param traits + all 208 tests stay green). Reuses Ty::Var/Ty::App/Ty::Arrow/dictionary/monomorphize.
**Micro-tasks (serial-on-L1, each a small Sonnet leaf branched from the prior's pushed tip, push each):**
- **MT1** `parse.rs`+`ast.rs`: impl header trait-args (`impl Ident type_args? for type_ref {…}`) + `ImplDecl.trait_args: Vec<TypeRef>` + parse test.
- **MT2** `checkty.rs` Pass 1c+1d: resolve trait method sigs with the trait param in `tyvars` (→`Ty::Var`) + dict shell generic over the param; impl substitutes `A↦C`, checks methods vs substituted sigs, require `C==for_ty` (else refuse).
- **MT3** `checkty.rs` `check_generic_call`+`monomorphize`: parametric bound resolution (`X:T ⇒ impls[(T, mangle(X))]`, missing→explicit error) + generic-dictionary threading.
- **MT4** tests: `accept/17-parametric-trait.myc` + `check.rs` parametric cases + `differential.rs` three-way.
Every leaf: HARD PRECONDITION (grep `traits:`/`Ty::Arrow` in checkty.rs; else `git reset --hard origin/claude/keen-hypatia-bdmtt4`); never-silent (G2); Declared (VR-5, no `Proven`); no new kernel node (KC-3); no `proptest` (bounded `#[test]`); `cargo clippy -p mycelium-l1 --all-targets -D warnings` (ignore pre-existing `mycelium-mlir` unsafe noise).

### AFTER parametric traits are green on the work branch (orchestrator/Opus does these)
1. **Write RFC-0007 §4.10 spec** (mirror §4.9): parametric trait surface + literal-runtime-dictionary model + honest deferrals (multi-param traits, assoc types, supertraits, multi-bound `+`/no `Tok::Plus`, `impl T<C> for D` with C≠D). + ebnf `impl_item`/`bound` in `docs/spec/grammar/mycelium.ebnf`. RFC-0007 stays Accepted; slice Declared, "Rust-first pending ratification."
2. **Reconcile** (orchestrator-owned): `issues.yaml` M-658/M-659 → done (honest notes); `CHANGELOG.md`; RFC-0019 "implemented Rust-first (M-658/M-659), Declared, pending ratification" note; regenerate `docs/api-index/` (`just docs-index` — new `Env.traits`/`impls` + `Ty::Arrow`).
3. **`just check` green** (`export PATH="$HOME/.local/bin:$HOME/.cargo/bin:/opt/node22/bin:$PATH"`). Any L1 line-shift re-stales api-index → regen + re-run.
4. **Self-review** (/pr-review) → **PR work→head (`--no-ff`)** → Copilot auto-reviews (sourcery = no-op skip; address comments, reply once, `merge_pull_request` method=merge) → **pull head down** into the work branch.

### REMAINING E7 chain (→ "full lexicon" = DFB's unblock), dependency-ordered
M-660 effects → M-661 `wild`/FFI → M-662 `phylum`/cross-nodule → M-663 RFC-0018 grading (Declared) → M-664 `consume`/`grow`/`impl` → E7-2 M-667 `fuse`/`reclaim`/`tier` → M-668 R2 design → M-649 self-host. Each: spec-then-impl, serial-on-L1, Sonnet micro-task swarm, honest tags, land on head via PR.

### Operational lessons (this session)
- **Worktree-leaf drift:** an `isolation:worktree` leaf often (a) leaves the MAIN worktree on the leaf's branch — `git checkout claude/keen-hypatia-bdmtt4` after; (b) starts from a STALE base (~main, not the work tip) — hence the hard precondition. Merge the ref the leaf REPORTS; verify merge-base; prune stale worktrees/branches after (mitigation #5/#7).
- **api-index churn:** any L1 line-shift (incl. `cargo fmt` collapsing a multi-line `format!()`) re-stales `docs/api-index/` → `just docs-index` + re-run `just check`.
- **Toolchain installed** (`scripts/install-tools.sh`): `just`/pre-commit/yamllint/codespell/shellcheck/cargo-deny/audit; markdownlint via `npx`; cargo-nextest absent → `cargo test` fallback. Export PATH per Bash call.
