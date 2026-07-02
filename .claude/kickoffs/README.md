# Kickoffs — tiered `dev → integration → main` workflow

Development runs on a **three-tier branch model** with a **stringency gradient** — messy below,
polished on top — plus **stowed kickoffs** (one per isolated-tree work package) so multiple
**Sonnet swarms** run in parallel across **disjoint crates/directories**, collision-free by
construction (CLAUDE.md §Swarm).

> **The top-level of this directory holds only *current* kickoffs.** Completed kickoffs are moved to
> [`archive/`](archive/) once their tranche has landed on `main` and been validated against the
> codebase (the audit that produced this list, 2026-06-28). See **§Completed (archived)** below.
>
> **Recent landings (2026-06-29 → 06-30, on `main`).** The serial-language closeout — tuple type +
> `f(x)(y)` (M-826), or-patterns (M-823), partial application (M-822), DN-54 §10 (M-824), backbone
> (M-825) — **resolved the `s10`/`hof` flagged residuals** (or-patterns, tuple-gated partial app).
> The **DN-64 §7** design wave also landed: **RFC-0038 Inject-Mode Security Axis → `Accepted`**
> (design ratified, mechanism unbuilt → claims stay `Declared` until Enacted), **DN-65** scoped-PR /
> toolchain-scoping workflow policy, `research/26`+`27`, and the VSA proof-discovery experiment
> (M-827…M-849). These are design-phase landings, **not** 1.0.0-track kickoffs — the track table
> below is unchanged by them (only the `s10`/`hof` residual notes are refreshed).
>
> **Workspace prep + scoped PRs (DN-65).** At kickoff, every agent **syncs off the latest tip**
> (same head, tips match) and **pre-installs the toolchain its work-package needs** (Rust →
> `just setup`; Python → `uv sync`; docs → markdown/`doc_refs`; proofs → `z3`/LH/Lean) before
> working. Land each work-package as **logical, closely-scoped PRs** (~1–2k-LOC soft target,
> individually `/pr-review`'d), not one monolith — see `docs/notes/DN-65-…md`.
>
> **Post-AOT PM resync (2026-07-01).** `aot10` (T6, native AOT) is **NO LONGER post-1.0/1.1** — ADR-034
> (Accepted 2026-06-30, maintainer-ratified) re-gates it as a **hard `lang 1.0.0` gate row**, reversing
> ADR-022 §8 Q4. It is moved into the 1.0.0-track table below. Epics **E7-1**, **E7-2**, and **E21-1**
> are also confirmed **closed** (`status:done`) in this resync — all of their children landed with
> checked evidence; see `tools/github/issues.yaml`.
>
> **Kickoff prose-sync + three closures (2026-07-01, same day).** `aot10` (E15-1/E25-1 — the M-863
> ratification act lands M-856b/M-860/M-862/M-863, all `done`), `kpr` (E19-1 — M-752 lands `done`), and
> `lib10` (E13-1 — M-714…M-719 all `done`) are now **✅ DONE → archived** (moved to `archive/` unchanged,
> per the archival convention above; their table rows below stay, marked done, mirroring `s10`/`r10`).
> **ADR-035** (Accepted, maintainer-ratified) narrows T4's Definition of Done to the DN-66 stable-API
> freeze + core-lib self-host slice (M-714…M-718 + M-719's freeze half); the full RFC-0031 §5 D6
> Rust-crate-retirement half is re-scoped **post-1.0** as new issue **M-867** (`status:todo`, P3), not a
> 1.0.0 blocker. **ADR-036** (Accepted, maintainer-ratified) makes the `lang 1.0.0` **tag** and the
> project's **public release** two distinct milestones: the tag needs only the already-met core-lib
> self-host slice; E18-1's remaining children (M-739…M-742, `boot10`) become the **comprehensive
> dogfooding** track — real, within-1.0.0, tracked work, but it gates the *public release* (and keeps
> the repo private until validated), not the tag, and runs in parallel rather than serially blocking.
> Net effect per `issues.yaml`'s own M-738 `landed_basis`: **every engineering gate row for the tag is
> now closed** — M-738 (`rel10`) stays `status:blocked`, but purely on the maintainer's own tag-cut act.

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
| **`s10`** | T2 — surface-language completeness & grammar (E11-1) | `crates/mycelium-l1/**` · `docs/spec/grammar/**` | ✅ **DONE → archived** (2026-06-29): M-704 (dynamic HOF) · M-705/M-708 (ops/stabilization, prior) · **M-706** (RFC-0030 grammar gaps) · **M-707** (RFC-0020 §10 carve-out enactment) all landed. **E11-1 surface-language complete.** Residuals **RESOLVED in the serial closeout** (2026-06-29, #767): or-patterns (R20-Q3 → **M-823**), partial application + first-class tuple type incl. `f(x)(y)` (**M-822**/**M-826**), list bidirectional inference (R20-Q5 → **M-823**); the `for`-body→spine two-pass feedback remains a flagged open item (never-silent) |
| **`r10`** | T3 — runtime & concurrency execution maturity (E12-1) | `crates/mycelium-std-runtime/**` · `crates/mycelium-mlir/src/runtime.rs` | ✅ **DONE → archived** (2026-06-29): M-709/710/711/712/713 + **M-677** (declared-effect → interp budget ledger + per-effect `retry(<=3)` surface syntax wired; overrun → explicit `EffectBudgetExhausted`; KC-3, no new L0 node) all landed. **E12-1 runtime maturity complete.** |
| **`lib10`** | T4 — standard library in Mycelium (E13-1) | `lib/std/**` · `crates/mycelium-std-*/**` | ✅ **DONE → archived** (2026-07-01): M-714…M-719 all landed. **ADR-035** (Accepted) narrows T4's bar to the DN-66 stable-API freeze + core-lib self-host slice (M-714…M-718 + M-719's freeze half) — met; the full D6 Rust-crate-retirement half is re-scoped **post-1.0** as new issue **M-867** (`status:todo`, P3). **E13-1 complete.** |
| **`rel10`** | T8 — documentation, stability & 1.0.0 release (E17-1) | `docs/**` · `CHANGELOG.md` · stability/release scope | **in progress**; M-735/736/737 landed; remaining **M-738** (release act) is `status:blocked` **purely on the maintainer's tag-cut act** (2026-07-01) — every engineering gate row it waited on (E13-1, E15-1/E25-1, ADR-023) is now closed, and E18-1 is no longer a blocker at all (ADR-036, below) |
| **`boot10`** | T9 — self-hosting capstone (E18-1) | `lib/std/**` · `crates/mycelium-l1/**` · self-hosting | **unblocked** (2026-07-01) — E11-1 (`s10`) and E13-1 (`lib10`) have both landed; M-739…M-742 stay `needs-design`. **ADR-036** (Accepted, maintainer-ratified) reframes this remaining scope as the **comprehensive-dogfooding** track: real, within-1.0.0, tracked work, but it gates the project's separate *public-release* milestone, not the `lang 1.0.0` tag (the tag's only self-hosting bar — the core-lib slice — is already met via E13-1) |
| **`aot10`** | T6 — native AOT full-language coverage, parallelism & 1.0.0 gating (E15-1/E25-1) | `crates/mycelium-mlir/**` | ✅ **DONE → archived** (2026-07-01): the **M-863 ratification act** lands the remainder — **M-856b** (dialect Dense/VSA), **M-860** (parallel codegen), **M-862** (parallel pure-fragment eval), **M-863** itself — all `done`; E15-1 + E25-1 both close. RFC-0029 → **Enacted**; DN-15 → **Resolved**; **ADR-034 stays Accepted** (its own `Accepted → Enacted` step is coupled to the `lang 1.0.0` tag act, M-738 — not yet run, per house rule #3) |

*(T2 = `s10`, T3 = `r10`, T4 = `lib10`, T5 = `ffi10`, T6 = `aot10`, T7 = `tool10` are all **complete →
archived**.)*

### Phase-I function-first kickoffs (ADR-038 Accepted, 2026-07-01 · umbrella roadmap)

Authored 2026-07-01 by the planning tier (ADR-038 §2.7 — Fable plans, Sonnet/Haiku implement; every
task PM-prepped with user stories + DoD **before** any implementation agent). Decomposition source:
`docs/planning/road-to-1.0.0-and-mycelium-rewrite.md`. All M-ids in these kickoffs are **proposed,
not minted** — verify free slots at each kickoff (mitigation #1). `acy` **lands first**; `enb` is
the usability critical path; `grm` is decision-gated; `opp` is non-gating and parallel.

| UID | Scope | Owns | Status / remaining |
|---|---|---|---|
| **`acy`** | **H0** — acyclic-deps enforcement + workspace hygiene (roadmap §2) — **LANDS FIRST** | dep-structure check (xtask/`deny.toml`) · `cert`/`select` fixture refactors · runtime-ABI seam crate · workspace `publish = false` sweep · M-866 `mono.rs` recursion bound | 📋 planned — 11 tasks (M-877…M-886 proposed + existing M-866) |
| **`enb`** | **H1** — below-grammar enabler closure: B→C→A(float route-ii)→E prim lane + D-lite/`myc run`/strings/`hash.*` (roadmap §3; the Phase-I critical path) | `mycelium-interp/src/prims.rs` + `mycelium-l1` frontend (**the serial lane**) · `mycelium-cli` (`myc run`) · float ADR + DN-39 dossier | 📋 planned — 30 tasks (M-887…M-914 proposed + RFC-0033-named M-766/M-767); after `acy`; maintainer gates inside (float ADR · consume model · M-828 D-lite split) |
| **`grm`** | **H2a** — grammar-stability gate before mass porting (roadmap §5): RFC-0037 follow-ons · DN-54 completion · tuple decision · ADR-033 FLAG-1 | `mycelium-l1` frontend (serial) · extension-checker · `mycelium.ebnf`/editor grammars/api-index regeneration | 📋 planned — 10 tasks (M-915…M-924 proposed); **ratification-gated** (three maintainer decisions; dossiers first) |
| **`opp`** | Opportunistic `.myc` ports — 9 ready-now pure/structural crates, smallest-first (roadmap §6; **non-gating**) | `lib/std/{diag,core,select,swap,recover,error,testing,ternary,spore}.myc` (new) · `mycelium-l1/tests/std_*.rs` · per-crate pre-port Rust polish | 📋 planned — 11 tasks (M-925…M-935 proposed); needs only `acy`; runs parallel with `enb` |

### Phase-II kickoffs (post-public — ADR-038 §2.3/§2.8 · roadmap §7)

Authored 2026-07-01 by the planning tier (same discipline). `flp` is the **Phase-I→II boundary
event** — its prep may start once the Phase-I DoD is in sight, but **the flip act itself is
strictly last** and waits on the maintainer's usability ratification (ADR-036 §2.4 as refined by
ADR-038). `rwr` is deliberately **higher-altitude** (progressive, per-wave minting, decision-gated
— ADR-038 §2.3); its per-crate issues are minted per wave, not by the kickoff. All M-ids proposed,
not minted (mitigation #1).

| UID | Scope | Owns | Status / remaining |
|---|---|---|---|
| **`flp`** | **The PUBLIC FLIP: decompose + publish** — author the owed binding decomposition ADR (ADR-039 proposed number; DN-27 §4), then lock+archive the monorepo, per-phylum-group repo scaffolds (~8–12, maintainer ⟐), per-repo CI+GHCR (ADR-037 rails), issue/docs porting, the one-act flip at a `0.x` | the decomposition ADR · repo scaffolds + history carry · reusable CI workflows · `repo:` issue axis · the flip runbook + act | 📋 planned — 11 tasks (M-936…M-946 proposed); everything private until the act; **the flip is maintainer-gated** (usability ratification · `0.x` · FLAG-V1) |
| **`rwr`** | **Phase-II progressive Mycelium rewrite** (post-public; the public semver climbs `0.x → 1.0.0`; `1.0.0` = fully rewritten where appropriate + 100% operational) | port-wave manifests + waves (D5/differential/D6) · transpiler hardening ladder · toolchain-port scoping · V-wave remainder (RFC-0033 M-760…M-784 incl. the single M-780 rehash at its tripwire) · the `1.0.0` terminal dossier | 📋 planned — 11 wave-level tasks (M-947…M-957 proposed); **gated on `flp` + `grm`** (mass porting) and `enb` (V-wave audit); compiler self-hosting stays an **aspiration**, not a lane (FLAG-V2) |

### Kernel-enablement

| UID | Scope | Owns | Status / remaining |
|---|---|---|---|
| **`kpr`** | E19-1 — value reprs + prims that unblock E13-1 (RFC-0032) | `crates/mycelium-interp/src/prims.rs` · `crates/mycelium-core/**` (coord `c10`) | ✅ **DONE → archived** (2026-07-01): M-746…M-752 all landed (RFC-0032 Accepted; every enabler + the M-752 Tier-2 enablement smoke ports). **E19-1 complete** — unblocks `c10`'s M-703 dependency. |

### Surface follow-ons (`crates/mycelium-l1/src/{parse,checkty,elab,mono}.rs`) — ✅ ALL COMPLETE

All surface follow-on kickoffs have landed and are **archived** (2026-06-29): **`srf`** (E7-2 — M-664
`consume` + inherent `impl`; M-668 R2 planning DN-63), **`hof`** (R3 dynamic HOF / closures, M-704),
**`lwd`** (DN-54 `derive`→L0 elaboration + sound KC-3 guard, M-812-cont), and **`strm`** (E24-1 —
M-818/M-821 mandatory `;`, M-819 `mycfmt --flatten`, M-820 `myc --stream`; **DN-57 → Enacted**). See
**§Completed (archived)** for each. **The `crates/mycelium-l1` frontend serial lane is now clear** —
the next serial-on-L1 work (`boot10` self-hosting) is **unblocked** (2026-07-01): `lib10`/E13-1 has
landed (see the resync note above). Residual status (updated
2026-06-29/30): `hof` partial-application + first-class tuple type incl. `f(x)(y)` — **RESOLVED**
(M-822/M-826); `s10` or-patterns (R20-Q3) + list inference (R20-Q5) — **RESOLVED** (M-823), only the
`for`-body→spine two-pass remains a flagged open item; `lwd` `derive`-site consumption model —
DN-54 §10 design-pass landed (M-824; DN-54 stays Accepted); `srf`/E7-2 **R2 construct activation**
(`forage`/`backbone` et al.) — now **directed** by DN-64 §7 OQ-B (M-828), future `needs-design`.

### PM tooling & post-1.0

| UID | Scope | Status / remaining |
|---|---|---|
| **`tul`** | GitHub PM tooling | M-675 (`idmap.tsv` reconcile) **done**; only **M-676** (Projects-v2 Area field) remains — deferrable/secondary (P3) |
| **`dfb`** | **the dogfooding boundary** — `crates/mycelium-web` + `crates/mycelium-adk` (NEW) | ⏸ **UNSHELVABLE — awaiting maintainer decision to resume or re-shelve** (2026-07-01): the shelving condition (L1-surface-completeness wave — `s10`/`hof`/`lwd`/`strm`) is now **SATISFIED/archived**; the technical blocker is gone but M-670/M-671 stay `status:blocked` pending an explicit maintainer call. Research gate (`dfr`) discharged |

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
| **`s10`** | E11-1 surface-language completeness — M-704 (HOF) · M-706 (RFC-0030 grammar gaps) · M-707 (RFC-0020 §10 carve-out enactment) landed (joins prior M-705/M-708). Residuals flagged: or-patterns (R20-Q3), R20-Q5, partial-app (tuple-gated) | — (residuals in RFC-0020 §9 / RFC-0024 §4A.8) |
| **`hof`** | M-704 dynamic HOF — closures/capture/dynamic-fn-flow + capturing `map` run three-way; KC-3 (no new L0 node) | ✅ residual RESOLVED — tuple type + `f(x)(y)` (M-826) + partial application (M-822) landed (serial closeout #767) |
| **`lwd`** | M-812-cont DN-54 `derive`→L0 elaboration + §4.1/§4.2 checks + sound §6 KC-3 guard + §7 harness | `derive`-site consumption model → maintainer ratification (DN-54 stays Accepted) |
| **`srf`** | E7-2 lexicon tail — M-664 (`consume` + inherent `impl`) + M-668 (R2 planning DN-63) landed 2026-06-29 | E7-2 **R2 construct activation** (per-construct xloc/mesh/cyst/graft/forage/backbone impl RFCs, DN-63) — future, `needs-design` |
| **`strm`** | E24-1 DN-57 enactment — M-818/M-821 (mandatory `;` + corpus migration), M-819 (`mycfmt --flatten`), M-820 (`myc --stream`) landed; **DN-57 → Enacted** (#762) | — |
| **`r10`** | E12-1 runtime maturity — M-709/710/711/712/713 + M-677 (effect→budget ledger + `retry(<=3)` surface syntax, KC-3) landed 2026-06-29 | — |
| **`aot10`** | E15-1/E25-1 native AOT — M-863 ratification act closes the remainder (M-856b dialect Dense/VSA, M-860 parallel codegen, M-862 parallel pure-fragment eval, M-863 itself); RFC-0029 → Enacted, DN-15 → Resolved (ADR-034 stays Accepted pending the `lang 1.0.0` tag, M-738) landed 2026-07-01 | `rel10` (M-738 gate now closed on this front) |
| **`kpr`** | E19-1 kernel self-hosting-enablement surface — RFC-0032 Accepted + M-746…M-752 (comparison/binary-arith prims, `Repr::Seq`/`Repr::Bytes`, width-generics reassigned to `s10` as M-753, Tier-2 enablement smoke ports) landed 2026-07-01 | `c10` (M-703's E19-1 dependency now met) |
| **`lib10`** | E13-1 stdlib in Mycelium — M-714…M-719 landed; ADR-035 narrows T4's bar to the DN-66 stable-API freeze + core-lib self-host slice (met); full D6 Rust-crate retirement re-scoped post-1.0 as M-867 landed 2026-07-01 | `rel10` (M-738 gate now closed on this front); `boot10` (E13-1 precondition met) |

## Parallelization & sequencing guide

Collision-free-by-construction means **one kickoff owns one disjoint directory**. The only contended
file is `crates/mycelium-l1/src/{parse,checkty,elab,mono}.rs` (the L1 frontend) — everything that edits
it **must serialize**; everything else is **parallel by construction**.

### The one serial lane — `crates/mycelium-l1/src/{parse,checkty,elab,mono}.rs`

These all surgery the same L1 frontend files, so **exactly one is in flight at a time** (land + promote
green before the next — "do L1 surgery inline, never delegate to parallel leaves", CLAUDE.md #8/#10).

> ✅ **The L1 serial lane is COMPLETE (landed 2026-06-29 on `claude/sequential-kickoff-workflow-qbdigb`,
> PR #750):** `srf(M-664) → s10(M-707 → M-706) → hof(M-704) → lwd(M-812-cont) → strm(M-818) → r10(M-712)`
> all landed (run by a serial Opus swarm: one isolated-worktree leaf per task, sync-first off the
> pushed tip, octopus-merged + orchestrator-reconciled). **Flagged residuals carried forward** (all
> never-silent, none blocking): these residuals have since **landed** (serial closeout 2026-06-29,
> #767) — `hof` partial application + tuple type incl. `f(x)(y)` (M-822/M-826), `s10` or-patterns
> (R20-Q3) + list inference (R20-Q5) (M-823); `lwd` `derive`-site got the DN-54 §10 design-pass
> (M-824). Only the `for`-body→spine two-pass feedback remains a flagged open item.
>
> ✅ **The follow-on serial L1 wave is also COMPLETE (landed 2026-06-29, PRs #755→#757 squash +
> #758/#759 backprop):** `r10`'s **M-677** (effect→budget ledger + `retry(<=3)` surface syntax — the
> one genuinely serial-on-L1 task) landed, and — disjoint/parallel by construction — `strm`'s
> **M-819** (`mycfmt --flatten`) and **M-820** (`myc --stream`) plus `srf`'s **M-668** (R2 planning
> DN-63) landed alongside. **DN-57 → Enacted** (#762). **The `parse`/`checkty`/`elab`/`mono` serial
> lane now has NO queued tasks** — every ticketed task touching the L1 frontend is done. The next
> serial-on-L1 work is **`boot10`** (self-hosting; owns `mycelium-l1/**`) — **now unblocked**
> (2026-07-01): `lib10`/E13-1 (the long pole) has landed and archived, alongside `kpr`/E19-1 and
> `aot10`/E15-1+E25-1. Per **ADR-036**, `boot10`'s remaining scope (M-739…M-742) is the
> **comprehensive-dogfooding** track — real work, but it gates the project's separate public-release
> milestone, not the `lang 1.0.0` tag. **The path forward is `rel10` + `tul` (+ `boot10` if desired)**,
> not another serial L1 kickoff. Still-open L1-adjacent items are **non-lane**: `M-674`
> (ongoing explicit-budget hardening). (`kpr`/M-752's smoke ports, which touched
> `mycelium-l1/tests/**` not the frontend `src`, are done and archived.)

Recommended order for a *fresh* serial lane (cheapest-unblocks-most first; any order is valid as long
as it's serial). The completed lane above is kept for the method/record.

### Parallel by construction — disjoint trees (run concurrently, with each other AND the serial lane)

| Kickoff | Disjoint tree it owns | Why no collision |
|---|---|---|
| **`rel10`** | `docs/**`, `crates/mycelium-doc/**`, the release notes | docs/release; cites code read-only |
| **`tul`** (M-676) | `tools/github/**` | PM tooling only |
| **`boot10`** (M-739…M-742) | `lib/compiler/**` (new) + `mycelium-l1/**` (read/differential only) | now unblocked (E11-1/E13-1 both landed); ADR-036's comprehensive-dogfooding track — real work, non-gating on the tag |

*(`lib10`, `kpr`, and `aot10` — formerly listed here — landed 2026-07-01 and are now archived; see
§Completed above.)*

### Sequenced by dependency (cannot start until a gate clears) — *not* parallelizable yet

- **`c10` M-703** (cut core tag) — its engineering deps (M-700/701/702, E19-1) are all `done`; gated on
  **maintainer** only (reserved).
- **`rel10` M-738** (release act) — every engineering gate row is now closed (E13-1 done via `lib10`;
  E15-1/E25-1 done via `aot10`; E18-1 non-gating per ADR-036; ADR-023/M-737 done); `status:blocked`
  **purely on the maintainer's tag-cut act** (2026-07-01); runs **last**.
- **`dfb`** (dogfooding) — its shelving condition (L1-surface completeness) is now satisfied
  (2026-07-01); **unshelvable — awaiting maintainer decision to resume or re-shelve**, not
  pre-dogfood work either way.

(`boot10` is no longer listed as gated — see the Parallel table above; it is unblocked but its scope is
the comprehensive-dogfooding track, not a `lang 1.0.0` tag blocker.)

### The integrator's shared-file rule

Kickoffs treat these **read-only** and **FLAG up**; the integrator reconciles them once at
`dev → integration`: workspace `Cargo.toml`, `CHANGELOG.md`, `docs/Doc-Index.md`,
`tools/github/issues.yaml` + `idmap.tsv`, `docs/api-index/` (regenerated, never hand-merged). Cross-work
continuity rides `issues.yaml` `depends_on` + body notes — never by touching another tree's files.

### One-line scheduler

✅ **The L1 serial lane (`srf`→`s10`→`hof`→`lwd`→`strm`→`r10`) is COMPLETE** — no L1-lane kickoff
remains. ✅ **`lib10`, `kpr`, and `aot10` are also COMPLETE (2026-07-01, archived)** — every 1.0.0
engineering track is now green. **What's left:** `rel10`'s M-738 (the release act) is `status:blocked`
purely on the **maintainer's tag-cut act**; `c10`'s M-703 is likewise maintainer-reserved only; `tul`
has one deferrable secondary item (M-676); `boot10` is unblocked and may run (ADR-036's
comprehensive-dogfooding track, non-gating on the tag); `dfb` awaits a maintainer resume/re-shelve
call.

## Coverage — the current set IS comprehensive for pre-dogfooding

A reverse-coverage audit (2026-06-28) confirmed every **open pre-dogfooding** task maps to a current
kickoff above. The following open items are **deliberately *not* kickoffs** (excluded from the
pre-dogfood set), so the next session doesn't chase them:

- **Done, just status-lag:** **E7-5 / M-692** (operator syntax) — satisfied by the landed M-745 (`ops`,
  RFC-0025 Enacted); flipped to `done`. **M-724** (FFI safety verify) — E14-1 is `done`; label-lag
  flipped 2026-07-01 (PR #499). **E7-1** and **E7-2** (L1 stage-1 completeness / RFC-0008 runtime
  vocabulary) — all children done; both epics flipped `done` 2026-07-01.
- **Routed into an existing kickoff:** **M-677** (declared-effects → interp budget) → `r10`.
- **Design-pending (kickoff only *after* the RFC is ratified):** **E20-1** (collections / RFC-0033,
  `proposed`), **E22-1** (security-scan / RFC-0035). **E21-1** (tunable-cert / RFC-0034) is **no
  longer design-pending** — RFC-0034 → "Enacted — with code" (2026-06-24) and all 11 children landed;
  E21-1 flipped `done` 2026-07-01.
- **Post-1.0 / release-engineering (Phase 8):** **E9-1** editor highlighting + **M-697** · **M-743**
  MIT-licensing audit · **M-744** issue-dedup.
- **Housekeeping / ongoing hardening (not feature kickoffs):** **M-674** (explicit-budget robustness) ·
  **M-797** (inline-test retrofit) · **M-816** (stale-branch pruning).

## Reserved (maintainer-only; excluded from every kickoff)
**M-655 / M-703** (cut the 1.0.0 tag) · **M-381 / M-646** (LLM local runs).
