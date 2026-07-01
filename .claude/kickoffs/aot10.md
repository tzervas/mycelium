# Kickoff `aot10` -- Native AOT maturity, optimization, and acceleration (E15-1 / E25-1)

> Read `CLAUDE.md` (house rules win) + `.claude/kickoffs/README.md` + `RFC-0029` + `DN-15` +
> `ADR-019` + `ADR-034` first. The interpreter stays the trusted base throughout; the native path
> is the performance path. Every optimization pass must be EXPLAIN-able and never-silent
> (ADR-006/G2).

## Metadata

| Field | Value |
|---|---|
| **UID** | `aot10` |
| **Head branch** | `claude/head/aot10` |
| **Status** | LIVE/PARTIAL -- E15-1 (waveN2 M-725..729) landed done; owns the E25-1 remainder now |
| **Swarm mode** | Sonnet |
| **Depends on** | E15-1 (done); RFC-0029 (Accepted); ADR-034 (Accepted, 2026-06-30) |

## RESCOPE (2026-07-01, post-AOT audit)

This kickoff was originally written against the old E15-1/M-725..729 frame. **That frame is now
CLOSED**: M-725, M-726, M-727, M-728 flipped to `status:done` in this resync (stale labels against
already-landed, CHANGELOG-confirmed work); M-729 stays `status:ready` pending a maintainer call on
whether M-858's broader unified differential retroactively satisfies its narrower DoD (see this
resync's PR for the FLAG -- not decided here).

**ADR-034 (Accepted, 2026-06-30, maintainer-ratified) re-gated native AOT as a HARD `lang 1.0.0`
gate row (track T6)** -- it reverses ADR-022 §8 Q4 (which had un-gated T6 to `1.1`) and expands
E15-1's scope to full-language native-codegen coverage, carried forward by the umbrella epic
**E25-1**. `aot10` now owns the **E25-1 remainder** -- the still-open children after the
`M-850..M-861` coverage/perf wave landed:

- **M-856b** -- MLIR-dialect coverage: Dense/VSA through the dialect path (libMLIR-gated). Split
  out of M-856 (done) as an honest partial-landing division -- Construct/Match/Swap landed through
  the dialect leg, Dense/VSA did not. `status:ready`.
- **M-860** -- Parallel AOT codegen: per-function independent lowering, byte-identical emit.
  `status:ready`; depends_on M-859 (done, PR #845).
- **M-862** -- Parallel pure-fragment interpreter/env-machine eval + differential
  (post-tag-cautious). `status:needs-design`; depends_on M-860, M-861 (done, PR #843). **FLAG**:
  this issue's own body recommends landing LAST/cautiously, possibly AFTER the lang 1.0.0 tag --
  pre/post-1.0 timing is a maintainer call, not decided here or in the PM resync.
- **M-863** -- AOT ratification act: ADR-034 + RFC-0029 -> Enacted, DN-15 -> Resolved, status
  flips. `status:ready`; depends_on M-858 (done), M-862 (open). This is the CLOSING act -- run it
  last, after M-856b/M-860/M-862 land.

**Owned directory:** `crates/mycelium-mlir/` (the AOT/JIT backend crate) -- unchanged.
**Read-only for leaves:** `tools/github/issues.yaml`, `CHANGELOG.md`, `docs/Doc-Index.md`,
`docs/api-index/`, workspace `Cargo.toml`, `mycelium-core`/`mycelium-l0`/`mycelium-l1`
(the trusted base; no kernel changes).

## Epic and issue IDs (current)

- **E25-1** (epic, `status:in-progress`) -- Native AOT full-language coverage, parallelism, and
  1.0.0 gating (ADR-034). Children `M-850..M-861` done; remainder below.
  - **M-856b** -- Dense/VSA through the MLIR-dialect path (libMLIR-gated) `[type:feature]`
  - **M-860** -- Parallel AOT codegen, byte-identical emit `[type:feature]`
  - **M-862** -- Parallel pure-fragment eval + differential (post-tag-cautious; pre/post-1.0
    FLAGged) `[type:feature]`
  - **M-863** -- AOT ratification act (ADR-034/RFC-0029/DN-15 closing record) `[type:design]`

(Historical, closed: E15-1's original children M-725/726/727/728/729 -- four done this resync,
M-729 flagged not flipped; E25-1's M-850..M-861 -- all done, landed on `dev` across PRs #815-#851.)

## Grounding

- **ADR-034** (`docs/adr/ADR-034-Full-Language-1.0.0-Gate-T6-AOT-Re-Gating.md`, Accepted
  2026-06-30) -- the re-gating decision that makes this kickoff a hard `lang 1.0.0` gate, not a
  `1.1` QoL item. `Accepted -> Enacted` happens with ADR-022 at the `lang 1.0.0` tag (M-863's job).
- RFC-0029 (Accepted) -- AOT optimization/codegen-maturity/JIT design; advances to Enacted once
  M-863 runs (post-M-862).
- DN-15 -- honest decomposition of the native-path work; `Draft -> Resolved` is M-863's job, not
  pre-judged here.
- ADR-019 (libMLIR toolchain, Enacted) -- provisioning; M-856b is libMLIR-gated (skip-graceful
  where the toolchain is absent, per the M-856/M-857 precedent -- never a faked pass).
- `crates/mycelium-mlir/` -- current backend state (refreshed 2026-07-01 in E25-1's body: direct-LLVM
  covers non-tail Fix/FixGroup, closures, certified Swap, Dense, VSA; MLIR-dialect covers trit.mul +
  Construct/Match/Swap but still refuses Dense/VSA -- M-856b's scope).

## Swarm / parallelization pattern

- **M-856b** and **M-860** are independent (different files: `dialect/native.rs` vs the codegen
  emit path) and may run as parallel Sonnet leaves.
- **M-862** depends on M-860 (parallel codegen) and M-861 (done) -- start after M-860 lands.
- **M-863** is the closing, single-owner ratification act -- runs last, after M-856b/M-860/M-862.

Collision surface stays `crates/mycelium-mlir/` only.

## Definition of Done

- [ ] M-856b: Dense/VSA lower through the MLIR-dialect path; three-way differential green where
  libMLIR is provisioned (skip-graceful + recorded where not); cargo-mutants catches a
  dialect-lowering mutation on Dense/VSA specifically.
- [ ] M-860: `parallel_emit == sequential_emit` byte-equal (`Exact`); cargo-mutants catches a
  join-order mutation.
- [ ] M-862: `parallel_eval == sequential_eval == interp` over the pure fragment; the sequential
  interpreter stays the reference; **maintainer decides pre/post-1.0-tag timing at that PR**.
- [ ] M-863: RFC-0029 -> Enacted; DN-15 -> Resolved; ADR-034 -> Enacted with ADR-022 at the tag;
  status flips recorded append-only.
- [ ] `just check` green on `crates/mycelium-mlir/` with all new features enabled.
- [ ] E25-1 issue status -> `done`; CHANGELOG entry (append-only; honest guarantee tags).

## Landing

Wave-land via `/wave-land` on the `claude/head/aot10` head. PR into `integration` (full
`just check` + honesty review) then squash-PR into `main`. Orchestrator reconciles
`CHANGELOG.md`, `docs/Doc-Index.md`, `docs/api-index/`, `tools/github/issues.yaml` after
the octopus merge of leaves. Run `just docs-index` to regenerate `docs/api-index/` after
any public API change in `mycelium-mlir`.
