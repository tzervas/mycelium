# Kickoff `e7l` — E7 Language Completeness (`mycelium-l1`)

> Stowed kickoff, UID **`e7l`**. A parent session for the L1 language-completeness task set.
> Read `.claude/agent-context.md` + `CLAUDE.md` first (house rules win); this file adds the specifics.

## ⚡ RESUME HERE (updated 2026-06-22 — read this first)

**Branch fresh off `main`.** `main` tip is `dfb7af5` — carries the full e7l first tranche + M-660 + M-674
depth-safety. On resume: `git fetch origin main` → branch a fresh working branch off `origin/main`
(`git checkout -b claude/<desc> origin/main`). `main` is PR-only; your working branch squash-PRs to `main`
per logical unit.

**Continue the chain at M-661 — M-660 (effect annotations) LANDED (`dfb7af5`).** Direction (maintainer,
FIRM): **complete the FULL lexicon first** (M-661→M-664, then E7-2 M-667/M-668) **before any dogfooding** —
a complete language surface is what unlocks whole-project self-hosting + the example phylum. Do NOT shortcut
to a partial-language self-host.

**Done so far (ALL LANDED to `main`):**
- ✅ **M-656 / M-657-checker / M-658** — generics spec + checker (elab staged → M-673); trait spec + `impl` reserved.
- ✅ **M-659 — stage-1 trait/impl CHECKER + coherence** (`4b53bde`): `Item::Impl`, bounded `<T: Cmp>`,
  `Tok::Plus`, trait/instance registries, coherence (global uniqueness + single-nodule orphan), method-set
  conformance, bounded-call + unqualified trait-method resolution — all never-silent `CheckError` (G2),
  guarantee `Declared`; dictionary-passing L0 lowering STAGED → **M-673** (traits type-check but do NOT
  yet RUN). (Copilot caught + we fixed a real `require_instance` over-acceptance soundness bug.)
- ✅ **track-a PM tooling** (`fb92479`, #353): `gh-issues-sync.py --relationships` (issue↔PR↔date manifest,
  status-aware landed/evidence), opt-in `--use-api` REST+GraphQL client, multi-phase milestone anchor.
  Follow-ups filed: **M-675** (idmap full reconcile), **M-676** (multi-area project field — SECONDARY),
  **M-677** (effect→interp budget wiring).
- ✅ **M-674 depth-safety** (`mycelium-stack`, explicit budgets on all L1 passes, kernel `#![forbid(unsafe_code)]`).
- ✅ **M-660 — effect annotations** (`dfb7af5`): surface `fn … -> T !{eff1, eff2}` (Koka-style `!`; effect
  names = kernel kinds `retry|alloc|io|cascade|time` + user `Named`; absent ⇒ pure; duplicate = never-silent
  parse refusal). AST `FnSig.effects: Vec<String>`, `Tok::Bang`. Checker `check_effect_coverage`: declared ⊇
  performed (over fn bodies AND impl-method bodies). Under-declaration = explicit `CheckError`. **No new L0
  node (KC-3)** — effects are checker-only metadata, do NOT lower/run. Guarantee `Declared`. DN-14 §3 row 8
  → `present`. RFC-0014 §3.4 surface pinned (still Enacted).

**Next (full lexicon, in order):** **M-661** (`wild`/FFI floor — `wild { … }` inside a fn declaring `!{ffi}`;
`wild` is the `ffi` effect SOURCE for M-660's checker; `myc-sec` gate flags unapproved `wild`) → M-662
(`phylum`/cross-nodule + cross-nodule orphan enforcement M-659 deferred) → M-663 (RFC-0018 grading, stays
`Declared`) → M-664 (`consume`/`grow`/`impl` surface keywords) → **E7-2** M-667/M-668. **THEN dogfooding:**
M-673 (elaboration — monomorphization + trait dictionaries; makes generics/traits RUN) → M-649 (self-host
first `.myc` nodule) → example phylum.

> **Lesson recorded:** the original brief named a protected head `claude/head/e7-language`; in practice
> a single working branch off `main`, squash-PR'd per tranche, worked cleanly (no separate head
> needed). The L1 collision-serialization (token/parse/checkty/elab one editor at a time) held; a
> *disjoint* file (e.g. `eval.rs`) can run as a parallel leaf alongside the serial chain.

## Mission
Drive **E7-1** (L1 Stage-1 language completeness) + **E7-2** (runtime constructs) + **M-649**
(self-hosting Stage-2) to done. Dependency-ordered:

| # | Issue(s) | What |
|---|---|---|
| 0 | pull-down | ✅ done — `main` (tip `dfb7af5`) carries the M-666 foundation + full e7l tranche. |
| 1 | M-656 → M-657 | ✅ generics: spec done; **checker** done (elab staged → M-673). |
| 2 | M-658 → M-659 | ✅ M-658 (spec + `impl` reserved) done; M-659 trait checker done. |
| 3 | M-660 | ✅ effect annotations (checker-only, no L0 node; DN-14 row 8 → present). |
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
