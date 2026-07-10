# Kickoff `spw` — Stdlib Port Wave: parallel `.myc` dogfooding ports (E33-1 / M-867 breadth)

## Metadata

| Field | Value |
|---|---|
| **UID** | spw |
| **Working base** | branch off `dev` (`claude/orch-0000-stdlib-port-wave`); leaves in isolated worktrees |
| **Status** | ready (Wave-0 pilot) |
| **Swarm mode** | Opus (set at session start; honor for every spawned agent) |
| **Runs on** | the maintainer's **desktop** (WSL / many-core / GPU) — the CLOUD session owns the disjoint semcore serial lane, so the two never collide |
| **Depends on** | `trx2`/E33-1 draft corpus (`gen/myc-drafts/`, DONE) · the stdlib conformance harness (`crates/mycelium-std-conformance/tests/harness/mod.rs`, DONE) |

---

## Governing decisions (ADR-042 / ADR-043, both Accepted 2026-07-07)

- **ADR-042 (Rust-Base Freeze + Mycelium-First Expansion):** new Rust surface is **frozen now**; new
  functionality is authored in `.myc`; the **end-state** is the entire first-party project — **kernel
  included** — rewritten to `.myc` (zero foreign first-party languages by the DN-88 decomposition gate).
  This wave is therefore the **central thrust**, not peripheral breadth; the semcore/kernel rewrite (the
  cloud's serial lane) is the deepest, last step.
- **ADR-043 (Rust Retirement & Legacy Archival):** a Rust crate is **retired once its `.myc` port is
  FULLY VALIDATED** (retire-when-proven), the full legacy preserved on the **`archive` branch**, with
  per-crate housekeeping following removal. This supersedes the ADR-035/M-867 "post-1.0" framing with a
  **per-crate, as-proven** gate — a module can retire as soon as its differential clears.

---

## Mechanism (maintainer-ratified — do not deviate)

**The transpiler is a SCAFFOLD + gap-profiler, NOT a bulk porter** (M-991 verdict, DN-34 §8.7–§8.9,
`Empirical`): union `checked_fraction` is **~0–8%**. So you **HAND-PORT the disjoint majority** (express
Rust structs as ADTs, impls as functions, imports by hand — the things the transpiler structurally
can't emit) and merely **graduate the myc-check-clean fraction** the draft already produced. Every
ported module is upgraded from `Declared` to `Empirical` **only** by a differential against its Rust
oracle (VR-5). Never claim a module is ported without that differential.

## Scope — the PARALLEL stream (disjoint by construction)

Port the unported stdlib crates to `lib/std/<mod>.myc`, each with its `crates/mycelium-std-conformance/
tests/std_<mod>.rs` differential. These are **disjoint** (own `.myc` file + own test file + own crate
oracle), so N leaves run collision-free. **Ownership boundary (hard):** a leaf touches ONLY its
`lib/std/<mod>.myc` + `crates/mycelium-std-conformance/tests/std_<mod>.rs`. It treats
`crates/mycelium-std-conformance/tests/harness/mod.rs` and `Cargo.toml` as **read-only** (FLAG up).

**DO NOT touch `lib/compiler/**` or `crates/mycelium-l1/**`** — the semcore SCC self-hosting is the
CLOUD session's SERIAL lane (one file, `lib/compiler/semcore.myc`). Cross-session continuity rides
`issues.yaml` + branches, never by editing the other session's tree (CLAUDE.md Wave-N workflow).

## Wave-0 pilot targets (pure, disjoint, mature harness)

Ranked from `gen/myc-drafts/MANIFEST.md` by purity + `checked%`: **`std-numerics`** · **`std-time`** ·
**`std-content`**. (Host-FFI modules — `std-fs`/`std-io`/`std-sys` — come later; their differentials
need `wild`/`@std-sys` side-effect handling.) One isolated worktree per target.

## The per-leaf runbook (the D5 five rows — `crates/mycelium-std-conformance/tests/harness/mod.rs`)

1. **Triage before porting** (`/myc-drafts`, never port cold): read the target's `MANIFEST.md` row +
   `gen/myc-drafts/stdlib/<mod>/union.gap.json`. Split myc-check-clean draft items (fix-up) from
   gapped items (hand-port / design). **Surface-check:** confirm the residual is expressible in `.myc`
   today; if a real blocker needs a below-grammar enabler, **STOP and FLAG to `enb`** — never force (G2).
2. **Pre-port Rust polish** (if any) committed separately, behavior-neutral (existing Rust tests still pass).
3. **`.myc` nodule + `include_str!` harness** — load the ported nodule verbatim; drive `assert_three_way`
   / `assert_cases` (L1-eval ≡ L0-interp ≡ AOT, TV-checked).
4. **Differential vs the retained Rust crate oracle** green (the leaf wires its own oracle call from the
   crate's public API — see harness §Usage row 4). This is what earns `Empirical`.
5. **Honesty tags** carried, never upgraded in translation (`Empirical` for the differential agreement;
   `Declared`/`Exact` per the op's own basis).

## Gates + storage discipline (per leaf)

- **Change-scoped only:** `cargo run -p mycelium-check --bin myc-check -- lib/std/<mod>.myc` +
  `cargo test -p mycelium-std-conformance <its test>` + `cargo fmt` + `clippy -p mycelium-std-conformance
  --tests -D warnings`. **Never** `cargo build --workspace` in a leaf (OOM guard).
- **sccache** shared compile cache on; **cap concurrency** to fit disk; **clean the worktree `target/`**
  on leaf completion; watch `df` between rounds.
- **Heavy durability is OFF the leaf loop** — it runs ONCE on the desktop via
  **`scripts/wave-desktop-checks.sh`** (full `just check` + `check-full` + the self-hosting differentials
  + the VSA/GPU bundle), results committed back. Cloud/leaf sessions run only the light tiers.

## Swarm flow

Per-PR review loop (`/pr-land`): each leaf branch → `/pr-review` → patch → merge into the orchestrator
branch (`--no-ff`, lineage preserved). Orchestrator octopus-merges the leaves, reconciles the shared
files it owns (`harness/mod.rs` is read-only to leaves; `Cargo.toml`, `CHANGELOG`, `Doc-Index`,
`issues.yaml`, indices are integrator-owned — reconciled once at `dev → integration`), runs
`wave-desktop-checks.sh`, then promotes `dev → integration → main` (squash only into `main`).

## Definition of Done (Wave-0 pilot)

- The pilot validates the loop end-to-end: ≥1 stdlib module ported with a **green three-way differential**
  + native `myc check`, landed as a scoped PR, with the swarm mechanics (isolated worktrees, sccache,
  disk) confirmed under parallel load.
- Each ported module: three-way differential green (`Empirical`); no `lib/` edit from a corpus run;
  no silent gap (every deviation FLAGged).
- **Retire-when-proven (ADR-043):** when a module's port is FULLY validated, the **leaf FLAGs it for
  retirement** — the **orchestrator/integrator** performs the ADR-043 removal of the retired Rust crate
  (archive the legacy to the **`archive` branch**, drop the workspace `Cargo.toml` member + reverse-dep
  refs, per-crate housekeeping). A leaf never removes a crate itself (that touches the shared workspace
  manifest). Retirement is **not** a Wave-0 pilot gate — the pilot proves the *port* loop; the first
  retirements can follow once the maintainer confirms a pilot module is fully validated.
- Retro captures the quirk/edge-case taxonomy (Rust + Mycelium) + automation lessons → feeds Wave-1
  (wider fan-out + the stream-B transpiler gap-class / language-feature track, run separately).

## Grounding

`gen/myc-drafts/MANIFEST.md` (per-target `checked%` + gap categories) · DN-34 §8.7–§8.9 (M-991 verdict) ·
`crates/mycelium-std-conformance/tests/harness/mod.rs` (the D5 harness) · `/myc-drafts` + `/transpile-vet`
+ `/wave` + `/pr-land` skills · **ADR-042** (Rust-base freeze + Mycelium-first, kernel-included end-state)
· **ADR-043** (retire-when-proven + `archive` branch) · ADR-035/M-867 (superseded by ADR-043's per-crate
gate) · CLAUDE.md §Concurrent-PR development + §Wave-N workflow + mitigations #6/#11/#14.
