# Kickoffs — tiered `dev → integration → main` workflow

Post-M-662, development runs on a **three-tier branch model** with a **stringency gradient** — messy
below, polished on top — plus **stowed kickoffs** (one per isolated-tree work package) so multiple
**Sonnet swarms** run in parallel across **disjoint crates/directories**, collision-free by
construction (CLAUDE.md §Swarm).

## The tiers (each PR-gated; stringency rises with the tier)

```
feature/leaf  ──PR──▶  dev  ──PR──▶  integration  ──squash-PR──▶  main
 (isolated tree)      (messy OK)      (full gate)                (polished · released)
```

| Branch | Tier | Bar to land here (via PR) | Merge style |
|---|---|---|---|
| **`main`** | release | the **full** `just check` + `/pr-review` + a Copilot round + a **curated squash** — the clean, bisectable, released history | **squash only** (from `integration`) |
| **`integration`** | staging | the **full** `just check` green + honesty / grounding / append-only review; shared files reconciled once | `--no-ff` from `dev` (lineage preserved) |
| **`dev`** | working | **compiles + change-scoped tests pass** — messy is fine: WIP, exploration, octopus/swarm merges | octopus / `--no-ff` from feature/leaf |
| **`feature` / `leaf`** | work | the swarm's own `/dev-workflow` discipline | branched **off `dev`** |

- **`dev` is where work first lands.** Below `integration` things can be messy (WIP commits,
  exploratory branches, octopus swarm merges); only **compiles + scoped tests** is required.
- **`integration` is the promotion gate.** `dev → integration` requires the **full `just check`**
  green + the honesty review — this is where work is polished and the shared files reconciled once.
- **`main` is the release.** `integration → main` is a **single curated squash** (the squash-only-to-
  `main` policy is unchanged), gated by `/pr-review` + the Copilot round. `main` stays clean.
- **Persistent + PR-gated:** `main`, `integration`, `dev` — **no direct push, PR only** (set branch
  protection in the GitHub UI). Everything below `dev` is ephemeral and merges freely (no PR needed).
- **Fast-forward, not force** (CLAUDE.md mitigation #6): keep a session's *working* pointer clean;
  do work on feature/leaf branches; bring the tier branch up with `merge --ff-only` + a plain push.

## Parallel swarms — one kickoff per isolated tree

Each active kickoff **owns a disjoint directory**, so its **Sonnet swarm** (default mode) runs in its
own session/worktree without touching another's files. Fire each in a fresh session with `/kickoff
<uid>`; each branches **off `dev`**, merges its result **into `dev`**, then `dev → integration →
main` promotes it up.

| UID | Kickoff | Isolated tree (owns) | Swarm method | Depends on |
|---|---|---|---|---|
| **`srf`** | `srf.md` | `crates/mycelium-l1/**` · `.claude/memory/lang-lexicon-syntax.md` | Sonnet · **serial-on-L1** (M-664 → M-667) + parallel docs leaf (M-668) | `run` ✅ (M-673 landed) |
| **`std`** | `std.md` | `lib/std/result.myc` + differential tests | Sonnet · **single leaf** (disjoint from L1 Rust) | `run` ✅ (M-673 landed) |
| **`tul`** | `tul.md` | `tools/github/**` | Sonnet (docs/tooling) | Status: **done**; idmap reconciled, 61 ids mapped (M-675; #496) |
| **`dfb`** | `dfb.md` | `crates/mycelium-web/` · `crates/mycelium-adk/` (NEW) | Sonnet · parallel-leaf | ⏸ **SHELVED** behind E7-3/E7-4/E7-5 (L1 surface completeness) — maintainer 2026-06-23 |
| *(active)* **l1-capstone** | run-kickoff continuation; head `claude/orch-0000-l1-capstone` | `crates/mycelium-l1/**` · `crates/mycelium-fmt/**` · `lib/std/**` | **E7-3** HOF (serial-on-L1) ∥ **E7-4** comment-preserving mycfmt (lexer+fmt, disjoint) → M-649 complete | `run` ✅ (M-673 landed) |
| **`c10`** | `c10.md` | `crates/mycelium-core/**` · kernel T1 scope | Sonnet · serial-on-kernel | Status: **core gate met / tag-ready**; head `claude/head/c10`; governs E10-1 (kernel/core 1.0.0 sub-gate, T1); gate: ADR-022 T1 + DN-25; no task dep (T1 is the preserved ADR-021 kernel track); E10-1; M-700/701/702 done, M-703 tag reserved (#500) |
| **`s10`** | `s10.md` | `crates/mycelium-l1/**` · `docs/spec/grammar/**` · surface-language scope | Sonnet · serial-on-L1 | Status: **in progress**; head `claude/head/s10`; governs E11-1 (surface-language completeness & grammar, T2); gate: ADR-022 T2 + DN-25; deps: E7-1 (M-657/M-659 ✅) + E7-3 (M-685…M-688 ✅); operator syntax + surface stabilization landed (M-705/706/708; #502) |
| **`r10`** | `r10.md` | `crates/mycelium-std-runtime/**` · `crates/mycelium-mlir/src/runtime.rs` · runtime scope | Sonnet · parallel-leaf | Status: **in progress**; head `claude/head/r10`; governs E12-1 (runtime & concurrency execution maturity, T3); gate: ADR-022 T3 + DN-25; deps: E7-2 (M-666 ✅); scheduler/deadlock/supervision landed (M-709/711/713; #501); M-710/M-712 open |
| **`lib10`** | `lib10.md` | `lib/std/**` · `crates/mycelium-std-*/**` · stdlib scope | Sonnet · parallel-leaf | Status: **ready** (**long pole**); head `claude/head/lib10`; governs E13-1 (standard library in Mycelium, T4); gate: ADR-022 T4 + DN-25; deps: E11-1 (`s10`) |
| **`ffi10`** | `ffi10.md` | `crates/mycelium-std-sys/**` · FFI scope | Sonnet · parallel-leaf | Status: **in progress (on dev)**; head `claude/head/ffi10`; governs E14-1 (FFI & system interface, T5); gate: ADR-022 T5 + DN-25; no blocking dep (disjoint from T2/T3); wild/@std-sys execution landed to dev (#499); not yet on main |
| **`aot10`** | `aot10.md` | `crates/mycelium-mlir/**` · native-codegen scope | Sonnet · parallel-leaf | Status: **ready, but `1.1`/post-1.0.0** — T6 un-gated 2026-06-23 (ADR-022 §8 Q4: QoL/perf, **not a 1.0.0 blocker**; runs after the release alongside `boot10`); head `claude/head/aot10`; governs E15-1 (native AOT maturity, T6 → `1.1`); gate: ADR-022 §8 Q4 + DN-25; deps: E6-1 |
| **`tool10`** | `tool10.md` | `tools/**` · editor grammars · package-dist scope | Sonnet · parallel-leaf | Status: **ready**; head `claude/head/tool10`; governs E16-1 (toolchain, IDE & package distribution, T7); gate: ADR-022 T7 + DN-25; deps: E9-1 (`tul`/RFC-0026) |
| **`rel10`** | `rel10.md` | `docs/**` · `CHANGELOG.md` · stability & release scope | Sonnet · serial (docs-heavy) | Status: **in progress**; head `claude/head/rel10`; governs E17-1 (documentation, stability & 1.0.0 release, T8); gate: ADR-022 T8 + DN-25; no code dep (runs in parallel with T1–T7; gates the release tag); language ref + stdlib API docs + ADR-023 landed (M-735/736/737; #493); M-738 release act blocked (gate) |
| **`boot10`** | `boot10.md` | `lib/std/**` · `crates/mycelium-l1/**` · self-hosting scope | Sonnet · serial-on-L1 | Status: **ready** (**long pole**); head `claude/head/boot10`; governs E18-1 (self-hosting capstone, T9); gate: ADR-022 T9 + DN-25; deps: E11-1 (`s10`) + E13-1 (`lib10`) |
| **`kpr`** | `kpr.md` | `crates/mycelium-interp/src/prims.rs` · `crates/mycelium-l1/src/checkty.rs` (`prim_kernel_name`) · `docs/rfcs/RFC-0032-*` · **coordinated:** `crates/mycelium-core/**` (with `c10`) + L1 type system (with `s10`) | Sonnet · serial-on-prims then serial-on-core (`Repr::Seq`→`Repr::Bytes`) | Status: **gate cleared** — RFC-0032 **Accepted** (M-746, §5 D1–D7); enablers M-747…M-750 `todo`; head `claude/head/kpr`; governs **E19-1** (prims + value reprs that **unblock E13-1 Tier-1/Tier-2**); deps: RFC-0031 §5 D4 ✅; D6 (in core 1.0.0) extends ADR-022 T1 — **append-only via supersession, pending mechanism**; coordinates with `c10` (core) + `s10` (M-753 width-generics) |

**Parallelism (collision profile):**
- **`srf` owns `crates/mycelium-l1/` (Rust) → serial-on-L1** (M-664 leaf lands, then M-667 rebases
  and lands; M-668 docs leaf can run in parallel). See `srf.md` §M-673 run-collisions for the mandatory
  rebase checklist.
- **`std` owns only `.myc` + test files** — fully disjoint from `srf`'s Rust edits. `std` and `srf`
  **may run in parallel** (no collision).
- **`tul` ⟂ (the L1 track) are fully disjoint — fire in parallel** (separate sessions).
  `tul` = `tools/github/` only; the L1 track = `crates/mycelium-l1`. (`dfr` — research/docs only — is
  **done**: landed #344, see Completed.)
- **`dfb`** is **SHELVED** (maintainer re-sequencing, 2026-06-23): the dogfooding builds wait behind
  the **L1 surface-completeness wave** — **E7-3** (HOF / RFC-0024), **E7-4** (comment-preserving
  `mycfmt`), **E7-5** (operator syntax / DN-23), run on the `claude/orch-0000-l1-capstone` head.
  Building real apps is what most exercises these surface gaps; resume `dfb` once the surface is
  complete + ergonomic (issues.yaml M-670/M-671 carry the shelve note + `depends_on` E7-3/E7-4).
- **l1-capstone wave** (active, this session): **E7-3** owns `crates/mycelium-l1/` (serial-on-L1:
  M-685 → M-686 → M-687 → M-688); **E7-4** owns `crates/mycelium-l1/src/lexer.rs` + `crates/mycelium-fmt/`
  (M-689 ✅ → M-690 → M-691) and is **disjoint from E7-3** (fully parallel). All agents branch off the
  **`claude/orch-0000-l1-capstone`** head (the common fixed base); the head advances as each leaf merges,
  and the next leaf branches from / pulls down the advanced head. M-649 completes (pseudocode → real
  combinators) on the head once E7-3 lands.
- **`kpr` is the kernel-enablement leg that unblocks `lib10`** (E13-1 Tier-1/Tier-2). It owns
  `crates/mycelium-interp/src/prims.rs` + the `prim_kernel_name` map (largely unowned by other legs)
  and is **design-gated by RFC-0032/M-746** — no implementation leaf fires until that RFC is Accepted.
  Two coordinated overlaps (flag-don't-guess, maintainer sign-off before merge): the value-model reprs
  (M-749/M-750) touch `crates/mycelium-core/**` (**`c10`**'s kernel-T1 scope — the 1.0.0-placement is
  an RFC-0032 decision), and width-generics (M-751) touch the `mycelium-l1` type system (**`s10`**'s
  E11-1 scope — RFC-0032 Q5 decides whether `kpr` owns it or it reassigns to `s10`). `kpr` ⟂ `lib10`'s
  `lib/std/**` (the unblock is *demonstrated* via smoke tests under `crates/mycelium-l1/tests/`; the
  `.myc` consumers land in `lib10`), so cross-leg continuity rides the issues' `depends_on`
  (M-716 ⟸ M-749 · M-717 ⟸ M-750 · M-718 ⟸ M-747/M-748/M-751).

### Cross-track deconfliction — `r10` (runtime) ↔ `rel10` (docs/release)

`r10` is a **code** track and `rel10` the **docs/release** track; their working directories are
**disjoint, so they develop fully in parallel**:

- **`r10` (code)** owns `crates/mycelium-std-runtime/**`, `crates/mycelium-mlir/src/runtime.rs`,
  `crates/mycelium-l1/src/elab.rs` (M-710), plus its own **design docs** — `docs/rfcs/RFC-0008-*`,
  `docs/rfcs/RFC-0027-*`, `docs/spec/stdlib/runtime.md` — and its **per-crate API baseline**
  `docs/spec/api/mycelium-std-runtime.txt`.
- **`rel10` (docs/release)** owns `docs/reference/**`, `crates/mycelium-doc/**`,
  `docs/adr/ADR-023-*`, and the release notes + tag.

They overlap on exactly **five release-surface files** (the same set `rel10` shares with *every*
code track). Each has a single owner so neither head writes a file the other writes — the other
track **FLAGs up**, it does not edit:

| Shared file | Owner | The other track's protocol |
|---|---|---|
| `docs/api-index/` (unified `index.json` + `INDEX.md`) | **integrator** — regenerated once via `just docs-index` on the merged state (CLAUDE.md: never hand-merged) | neither head hand-edits it; per-crate baselines `docs/spec/api/<crate>.txt` stay **disjoint** (each track owns its own crate's file) |
| `CHANGELOG.md` | **`rel10`** (curation is M-738) | `r10` supplies its entry as PR-body / FLAG text; never edits the top-level CHANGELOG on its head |
| `docs/adr/ADR-022-*.md` §5 gate table | **`rel10`** (release-gate steward) | `r10` **FLAGs "T3 → gate-met" up**; its live status rides its own `issues.yaml` rows, not ADR-022 |
| `docs/Doc-Index.md` | **integrator** (reconcile once at `dev → integration`) | each FLAGs its index line; neither edits it on its head |
| `tools/github/issues.yaml` + `idmap.tsv` | **integrator** (reconcile once) | disjoint rows (M-709…713 vs M-735…738); validate + dedup after merge (mitigation #2) |

**Sequencing is a landing-time constraint only — development stays parallel:**

1. **`r10` lands before `rel10` reconciles the gate.** `rel10`'s ADR-022 §5 T3 status flip + the
   M-738 release act must reflect `r10`'s *final* T3 state, so `rel10` integrates the gate **last**
   ("`rel10` gates the release").
2. **`rel10`'s runtime prose cites `r10`'s spec as source-of-truth.** Write the M-735 runtime
   section + M-736 stdlib-runtime API in parallel, then run a content-consistency pass against
   `r10`'s final RFC-0008 / `runtime.md` / `mycelium-std-runtime` at integration — don't block prose
   on code.

**Not an `r10`↔`rel10` concern (noted to avoid false coupling):** `r10`'s **M-712** (reclamation) is
blocked on **RFC-0027 → Accepted**, which `r10` owns and advances itself. `r10`'s **M-710** edits
`crates/mycelium-l1/src/elab.rs`, putting it on the **L1-serialization track with `s10`/`srf`/
`l1-capstone`** — an `r10`↔L1 collision, not an `r10`↔`rel10` one. `rel10`'s **M-736** *reads*
`lib/std/**` (lib10's write territory) read-only — a content dependency on `lib10`, no write
collision.

Cross-work continuity rides the **issues** (`tools/github/issues.yaml` `depends_on` + body notes),
never by touching another tree's files. (`dfb` predates this workflow — ignore its old
`claude/head/*` reference; it now branches off `dev` like everything else.) **M-677** (effect→budget
runtime) is L1-collision and runs inside the `srf` serial track, not as its own parallel wave.

## Completed (archived)
- **`run`** — **M-673 LANDED** on `main` (via `claude/int-docs-mono-wave`, 2026-06-22):
  monomorphization + dictionary-free static trait resolution → generics/traits run to closed L0;
  `mono.rs` added; `Env.impls` field added; `Ty::Data` carries args; three-way differential green;
  DN-14 §3 rows 6+7 → `present`; M-657/M-659 → `done`. Unblocks **`std`** (M-649 — first
  self-hosted `.myc` nodule). See `run.md` for the full M-673 base-change summary.
- **`dfr`** — **RP-10/RP-9 research gate DISCHARGED + RFC-0022/0023 → Accepted, LANDED** on `main`
  (#344, 2026-06-21): four fractured Opus sub-reasoners per RFC verified the Honest-Uncertainty
  Registers against primary specs (RFC 9110/9112 · RFC 8259 · WHATWG-URL; ADK v2.3.0) + landed
  substrate — design-sound, no falsification (`research/12 §8` · `research/13 §6`). Both RFCs **Draft →
  Accepted** (maintainer ratification; **Enacted** still gated on the builds); M-670/M-671 bodies carry
  the cleared gate + the `dfb` build constraints. Unblocks **`dfb`** (now gated on the L1 surface only).
- **`e7l` / `e7lb` / `e7lc`** — the E7-1/E7-2 L1-surface chain **M-656 → M-662 LANDED** on `main`
  (generics · traits · effects · `wild`/FFI · phylum + cross-nodule). Continued by **`lex`**.
- **`lex`** — **M-663 LANDED** on `main` (#375→`dev`, #377→`integration`, #380 release→`main`): RFC-0018
  stage-1a static guarantee grading (`grade.rs` Pass 3d) enacted; RFC-0018 → **Enacted**; DN-14 §3 row
  11 → `present`. Plus a Copilot-caught grade-upgrade soundness fix + the check-tooling packed exit
  codes / failure digest (**DN-22** design capture). Continued by **`run`** (M-673, now landed).
- **`u78`** — **M-678 epic (M-679…M-683) LANDED** on `main` (#378): DN-21 unsafe-code hardening —
  all workspace `unsafe` confined to `jit.rs`, the trusted base `#![forbid(unsafe_code)]`-pinned, and
  the `just safety-check` SAFETY-adjacency gate added.

## Reserved (maintainer-only; excluded from every kickoff)
**M-655** (cut the 1.0.0 tag) · **M-381 / M-646** (LLM local runs).
