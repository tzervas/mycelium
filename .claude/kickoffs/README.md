# Kickoffs — tiered `dev → integration → main` workflow

Development runs on a **three-tier branch model** with a **stringency gradient** — messy below,
polished on top — plus **stowed kickoffs** (one per isolated-tree work package) so multiple
**Sonnet swarms** run in parallel across **disjoint crates/directories**, collision-free by
construction (CLAUDE.md §Swarm).

> **The top-level of this directory holds only *current* kickoffs.** Completed kickoffs are moved to
> [`archive/`](archive/) once their tranche has landed on `main` and been validated against the
> codebase (the audit that produced this list, 2026-06-28). See **§Completed (archived)** below.

## The tiers (each PR-gated; stringency rises with the tier)

```
feature/leaf  ──PR──▶  dev  ──PR──▶  integration  ──squash-PR──▶  main
 (isolated tree)      (messy OK)      (full gate)                (polished · released)
```

| Branch | Tier | Bar to land here (via PR) | Merge style |
|---|---|---|---|
| **`main`** | release | the **full** `just check` + `/pr-review` + a **curated squash** — the clean, bisectable released history | **squash only** (from `integration`) |
| **`integration`** | staging | the **full** `just check` green + honesty / grounding / append-only review; shared files reconciled once | `--no-ff` from `dev` (lineage preserved) |
| **`dev`** | working | **compiles + change-scoped tests pass** — messy is fine: WIP, exploration, octopus/swarm merges | octopus / `--no-ff` from feature/leaf |
| **`feature` / `leaf`** | work | the swarm's own `/dev-workflow` discipline | branched **off `dev`** |

- **Persistent + PR-gated:** `main`, `integration`, `dev` — **no direct push, PR only**. Everything
  below `dev` is ephemeral and merges freely (no PR needed).
- **Down-propagation after a release is a `--no-ff` *merge*, never a force-push** (CLAUDE.md
  mitigation #6). Because `integration → main` **squashes**, `main` diverges from the tier branches;
  the squash is brought back down by *merging* `main` into `integration` and `dev` (content becomes
  identical, `main` an ancestor of both — the tip SHAs differ by design; that is correct, not drift).
  A fast-forward is *not* possible after a squash. Force-pushes to protected branches are prohibited.
- **Doc-maintenance is part of every kickoff's DoD** — see [`_doc-maintenance.md`](_doc-maintenance.md)
  (anti-drift): each kickoff leaves `issues.yaml`, specs, `CHANGELOG`, grammar, and `docs/api-index/`
  current, so the next sequential kickoff inherits truth, not drift.

## Current kickoffs

Fire each in a **fresh session** via `/kickoff <uid>` (clean context budget). Each owns a **disjoint
tree**, branches **off `dev`**, merges into `dev`, then promotes `dev → integration → main`.

### The full-language 1.0.0 tracks (ADR-022 §5 · DN-25)

| UID | Track | Owns | Status / remaining |
|---|---|---|---|
| **`c10`** | T1 — core/kernel 1.0.0 sub-gate (E10-1) | `crates/mycelium-core/**` · kernel T1 scope | **gate-met / tag-ready**; only **M-703** (cut the tag) remains — **maintainer-reserved** |
| **`s10`** | T2 — surface-language completeness & grammar (E11-1) | `crates/mycelium-l1/**` · `docs/spec/grammar/**` | **in progress**; remaining **M-707** (RFC-0020 L2 carve-outs) → **M-706** (RFC-0030 grammar ratification); **M-704** dynamic HOF tracked under `hof` |
| **`r10`** | T3 — runtime & concurrency execution maturity (E12-1) | `crates/mycelium-std-runtime/**` · `crates/mycelium-mlir/src/runtime.rs` | **in progress**; M-709/711/713 + M-710 landed; remaining **M-712** (L1 `reclaim`→runtime reclamation; RFC-0027 Accepted — gate now the L1 surface) |
| **`lib10`** | T4 — standard library in Mycelium (E13-1) | `lib/std/**` · `crates/mycelium-std-*/**` | **in progress (long pole)**; M-715/716/717/718 landed; remaining **M-719** (API-freeze + Rust-crate retirement; post-1.0 acceptable per RFC-0031) |
| **`rel10`** | T8 — documentation, stability & 1.0.0 release (E17-1) | `docs/**` · `CHANGELOG.md` · stability/release scope | **in progress**; M-735/736/737 landed; remaining **M-738** (release act — gated on the other tracks; cuts the tag) |
| **`boot10`** | T9 — self-hosting capstone (E18-1) | `lib/std/**` · `crates/mycelium-l1/**` · self-hosting | **blocked** on E11-1 (`s10`) + E13-1 (`lib10`); M-739…M-742 `needs-design` |

*(T5 FFI = `ffi10` and T7 toolchain = `tool10` are **complete → archived**. T6 native AOT = `aot10` is
**post-1.0 / 1.1**, below.)*

### Kernel-enablement

| UID | Scope | Owns | Status / remaining |
|---|---|---|---|
| **`kpr`** | E19-1 — value reprs + prims that unblock E13-1 (RFC-0032) | `crates/mycelium-interp/src/prims.rs` · `crates/mycelium-core/**` (coord `c10`) | **in progress**; M-746/747/748/749/750/751 landed; remaining **M-752** (Tier-2 enablement smoke ports — now unblocked) |

### Surface follow-ons (serialize on `crates/mycelium-l1/src/{parse,checkty,elab}.rs`)

| UID | Scope | Owns | Status / remaining |
|---|---|---|---|
| **`srf`** | E7-2 lexicon tail | `crates/mycelium-l1/**` · `.claude/memory/lang-lexicon-syntax.md` | **in progress**; M-665/666/667 landed; remaining **M-664** (`consume` parse/check/elab + inherent `impl T { … }`) ∥ **M-668** (R2 planning doc: xloc/mesh/cyst/graft/forage/backbone) |
| **`hof`** | R3 — closures / dynamic HOF (M-704, now `ready`) | `crates/mycelium-l1/**` (incl. `mono.rs`) | **design-ready** (RFC-0024 Accepted §4A full Reynolds); impl pending — `Expr::Lambda` still lowers to `Residual` |
| **`lwd`** | DN-54 — `lower`/`derive` **elaboration** + KC-3 guard (**M-812-cont**) | `crates/mycelium-l1/src/{elab,checkty}.rs` | **ready**; `low` landed surface+structural; this lands RHS→L0 elaboration + the KC-3 kernel-growth guard + acyclicity + §7 harness → DN-54 Enacted |
| **`strm`** | DN-57 follow-on — mandatory `;` + flatten/stream tooling (**E24-1**, M-818–821 minted) | `crates/mycelium-l1/src/parse.rs` · `crates/mycelium-fmt/**` · CLI · corpus | **ready**; optional `;` landed; remaining: **M-818** mandatory-`;` (serial-on-L1) → **M-819** `mycfmt --flatten` ∥ **M-820** `myc --stream` (disjoint) → **M-821** corpus migration → DN-57 Enacted |

### PM tooling & post-1.0

| UID | Scope | Status / remaining |
|---|---|---|
| **`tul`** | GitHub PM tooling | M-675 (`idmap.tsv` reconcile) **done**; only **M-676** (Projects-v2 Area field) remains — deferrable/secondary (P3) |
| **`aot10`** | T6 — native AOT maturity (E15-1) | **POST-1.0 / 1.1** — ADR-022 §8 Q4 un-gated it as QoL/perf, *not* a 1.0.0 blocker; RFC-0029 Accepted, M-725…729 `ready` |
| **`dfb`** | **the dogfooding boundary** — `crates/mycelium-web` + `crates/mycelium-adk` (NEW) | ⏸ **SHELVED** behind the L1-surface-completeness wave (HOF/`hof` · comment-preserving `mycfmt` · operators). Research gate (`dfr`) discharged; resume once the surface is complete + ergonomic |

## Completed (archived → [`archive/`](archive/))

Validated against the codebase 2026-06-28; each tranche landed on `main`. Epic continuations (where
any) are owned by the still-current kickoff noted.

| UID | Landed | Continuation owner |
|---|---|---|
| **`hrd`** | DN-40 A1/A2/A3 input-validation closure (parser depth-guard + typed dep-hash); RFC-0028/DN-40 reconciled | — |
| **`ops`** | M-745 comparison/shift operators (`< > << >>`); RFC-0025 → Enacted; RFC-0037 §6 FLAG-E | — |
| **`prm`** | M-817 `fuse`/`reclaim` execute three-way; DN-58 §A/§B → Enacted; closes M-710 | — |
| **`r4v`** | M-667 `fuse`/`reclaim`/`@tier` L1 surface (DN-58) | runtime exec → `r10`/done |
| **`obj`** | M-811 `object`/`via` surface → desugar (DN-53) | — |
| **`low`** | M-812 `lower`/`derive` surface + structural checks (DN-54); **M-812-cont** (RHS elaboration + KC-3 guard) is a separate tracked `todo` | M-812-cont (issue) |
| **`run`** | M-673 monomorphization + dictionary-free trait resolution | — |
| **`std`** | M-649 first self-hosted `.myc` nodule (`lib/std/result.myc`) | — |
| **`lex`** | M-663 RFC-0018 stage-1a static guarantee grading → Enacted | — |
| **`e7l` · `e7lb` · `e7lc`** | E7-1/E7-2 L1-surface chain M-656→M-663/667 (generics · traits · effects · `wild`/FFI · phylum) | `srf` (M-664/M-668) |
| **`u78`** | M-678–683 DN-21 unsafe-code hardening (all `unsafe` confined to `jit.rs`) | — |
| **`tool10`** | E16-1 toolchain, IDE & package distribution (M-730–734) | — |
| **`ffi10`** | E14-1 FFI & system interface — `wild`/`@std-sys` execution + syscall floor (M-720–724) | — |
| **`dfr`** | RP-10/RP-9 web/ADK research gate discharged; RFC-0022/0023 → Accepted (#344) | `dfb` (builds, shelved) |
| **`rsm`** | cross-cutting Session-2 — W1 (M-753/718/717) + W2 docs-currency + W3 capture (DN-45–50, M-800–807 stubs) all landed | M-719 close → `lib10` |

## Parallelization & sequencing guide

Collision-free-by-construction means **one kickoff owns one disjoint directory**. The only contended
file is `crates/mycelium-l1/src/{parse,checkty,elab,mono}.rs` (the L1 frontend) — everything that edits
it **must serialize**; everything else is **parallel by construction**.

### The one serial lane — `crates/mycelium-l1/src/{parse,checkty,elab,mono}.rs`

These all surgery the same L1 frontend files, so **exactly one is in flight at a time** (land + promote
green before the next — "do L1 surgery inline, never delegate to parallel leaves", CLAUDE.md #8/#10).
Recommended order (cheapest-unblocks-most first; any order is valid as long as it's serial):

```
srf(M-664) → s10(M-707 → M-706) → hof(M-704) → lwd(M-812-cont) → strm(M-818) → r10(M-712)
```

- **`srf` M-664** (`consume`/inherent-`impl`) and **`s10` M-707/M-706** (RFC-0020/RFC-0030) are the
  surface-completeness core. **`hof` M-704** (closures) touches `mono.rs`. **`lwd`** (M-812-cont) touches
  `elab.rs`/`checkty.rs`. **`strm` M-818** (mandatory `;`) touches `parse.rs`. **`r10` M-712** (reclaim
  elaboration) touches `elab.rs`. `srf` M-668 (R2 planning, docs-only) is **not** on this lane — see
  parallel set.

### Parallel by construction — disjoint trees (run concurrently, with each other AND the serial lane)

| Kickoff | Disjoint tree it owns | Why no collision |
|---|---|---|
| **`lib10`** | `lib/std/**` (`.myc`) | Mycelium-source, not Rust; consumes the L1 surface read-only |
| **`kpr`** (M-752) | `crates/mycelium-interp/src/prims.rs` + `crates/mycelium-l1/tests/**` (smoke ports) | prims + test files; coordinates with `c10` on `crates/mycelium-core/**` (flag-don't-guess) |
| **`rel10`** | `docs/**`, `crates/mycelium-doc/**`, the release notes | docs/release; cites code read-only |
| **`aot10`** | `crates/mycelium-mlir/**` | native-codegen; **POST-1.0/1.1**, run after the release |
| **`tul`** (M-676) | `tools/github/**` | PM tooling only |
| **`srf` M-668** | `docs/notes/**` (R2 planning DN) | docs-only leaf; parallel with its own M-664 |
| **`strm` M-819/M-820** | `crates/mycelium-fmt/**` · `crates/mycelium-cli/**` | fmt/CLI; parallel **after** M-818 lands |

### Sequenced by dependency (cannot start until a gate clears) — *not* parallelizable yet

- **`boot10`** (E18-1 self-hosting) — blocked on **E11-1 (`s10`)** + **E13-1 (`lib10`)** landing.
- **`c10` M-703** (cut core tag) — gated on **E19-1 (`kpr` M-752)** + maintainer (reserved).
- **`rel10` M-738** (release act) — gated on every 1.0.0 track green; runs **last**.
- **`dfb`** (dogfooding) — **shelved** behind L1-surface completeness (`srf`/`s10`/`hof` + ergonomic
  `mycfmt`); the dogfood boundary, not pre-dogfood work.

### The integrator's shared-file rule

Kickoffs treat these **read-only** and **FLAG up**; the integrator reconciles them once at
`dev → integration`: workspace `Cargo.toml`, `CHANGELOG.md`, `docs/Doc-Index.md`,
`tools/github/issues.yaml` + `idmap.tsv`, `docs/api-index/` (regenerated, never hand-merged). Cross-work
continuity rides `issues.yaml` `depends_on` + body notes — never by touching another tree's files.

### One-line scheduler

Run **one** L1-lane kickoff at a time (`srf`→`s10`→`hof`→`lwd`→`strm`→`r10`); run **`lib10` + `rel10` +
`kpr` + `tul`** fully in parallel alongside it; hold **`boot10`/`c10`-tag/`rel10`-M738/`dfb`** until
their gates clear; `aot10` is post-1.0.

## Coverage — the current set IS comprehensive for pre-dogfooding

A reverse-coverage audit (2026-06-28) confirmed every **open pre-dogfooding** task maps to a current
kickoff above. The following open items are **deliberately *not* kickoffs** (excluded from the
pre-dogfood set), so the next session doesn't chase them:

- **Done, just status-lag:** **E7-5 / M-692** (operator syntax) — satisfied by the landed M-745 (`ops`,
  RFC-0025 Enacted); flipped to `done`. **M-724** (FFI safety verify) — E14-1 is `done`; label-lag only.
- **Routed into an existing kickoff:** **M-677** (declared-effects → interp budget) → `r10`.
- **Design-pending (kickoff only *after* the RFC is ratified):** **E20-1** (collections / RFC-0033,
  `proposed`), **E21-1** (tunable-cert / RFC-0034, `proposed`), **E22-1** (security-scan / RFC-0035).
- **Post-1.0 / release-engineering (Phase 8):** **E9-1** editor highlighting + **M-697** · **M-743**
  MIT-licensing audit · **M-744** issue-dedup. Plus `aot10` (T6 native AOT).
- **Housekeeping / ongoing hardening (not feature kickoffs):** **M-674** (explicit-budget robustness) ·
  **M-797** (inline-test retrofit) · **M-816** (stale-branch pruning).

## Reserved (maintainer-only; excluded from every kickoff)
**M-655 / M-703** (cut the 1.0.0 tag) · **M-381 / M-646** (LLM local runs).
