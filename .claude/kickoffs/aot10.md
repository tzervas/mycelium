# Kickoff `aot10` -- Native AOT maturity, optimization, and acceleration (E15-1)

> Read `CLAUDE.md` (house rules win) + `.claude/kickoffs/README.md` + `RFC-0029` + `DN-15` +
> `ADR-019` first. The interpreter stays the trusted base throughout; the native path is the
> performance path. Every optimization pass must be EXPLAIN-able and never-silent (ADR-006/G2).

## Metadata

| Field | Value |
|---|---|
| **UID** | `aot10` |
| **Head branch** | `claude/head/aot10` |
| **Status** | ready |
| **Swarm mode** | Sonnet |
| **Depends on** | E6-1 (native-path wave -- M-725 subsumes its remaining tasks); RFC-0029 reaching Accepted before M-726...M-729 implementation begins |

## Scope

Mature the native AOT path from the current state (direct-LLVM kernel subset + `mlir-dialect`
feature skeleton, per `crates/mycelium-mlir/src/lib.rs`) into a real optimizing backend:
full libMLIR lowering, EXPLAIN-able optimization passes (inlining/CSE/DCE), JIT for dynamic
VSA/HDC workloads (ADR-009 deferred to enacted), BitNet packed-ternary acceleration (FR-C3),
and a mutant-witnessed three-way differential (interp vs AOT vs JIT).

**Owned directory:** `crates/mycelium-mlir/` (the AOT/JIT backend crate).
**Read-only for leaves:** `tools/github/issues.yaml`, `CHANGELOG.md`, `docs/Doc-Index.md`,
`docs/api-index/`, workspace `Cargo.toml`, `mycelium-core`/`mycelium-l0`/`mycelium-l1`
(the trusted base; no kernel changes).

## Epic and issue IDs

- **E15-1** -- Native AOT maturity, optimization, and acceleration
  - **M-725** -- full libMLIR integration: ternary-dialect to MLIR to LLVM lowering, subsuming E6-1 remaining `[type:feature]`
  - **M-726** -- optimization passes (RFC-0029): inlining/CSE/DCE as EXPLAIN-able, never-silent, differential-checked transforms `[type:feature]`
  - **M-727** -- JIT for dynamic / VSA workloads (ADR-009 deferred to enacted) `[type:feature]`
  - **M-728** -- BitNet-class packed-ternary acceleration (FR-C3) `[type:feature]`
  - **M-729** -- codegen differential durability: interp == AOT == JIT three-way agreement, mutant-witnessed `[type:verification]`

## Grounding

- RFC-0029 (`docs/rfcs/RFC-0029-AOT-Optimization-Codegen-Maturity-and-JIT.md`) -- must reach
  **Accepted** before M-726...M-729 implementation begins.
- RFC-0004 (Execution Model -- ss2 revisit clause sanctions advancing the native path; ss6
  inspectability applies to every optimization pass).
- DN-15 (Draft, 2026-06-19) -- honest decomposition of M-348; ground truth for which native-path
  work is libMLIR-gated vs direct-LLVM-advanceable.
- ADR-019 (libMLIR toolchain) -- provisioning decision; **check current binding status before
  starting M-725** (DN-15 ss1 records it was blocked at M-373's time).
- ADR-009 (JIT deferred) -- the source of the deferral M-727 is designed to lift.
- ADR-006 (no black boxes) + G2/VR-5 -- every optimization pass is EXPLAIN-able and never-silent.
- `crates/mycelium-mlir/src/lib.rs` -- current backend state (checked 2026-06-23).

## Swarm / parallelization pattern

**Serial gate then staged parallel.** RFC-0029 must reach Accepted before implementation.
Then:

- **M-725 (full libMLIR integration)** -- must land first; it provides the complete lowering
  infrastructure all other leaves depend on.
- **M-726 (optimization passes)** and **M-727 (JIT)** are independent once M-725 lands; may
  run as parallel Sonnet leaves (M-726 adds pass pipeline; M-727 adds JIT execution mode;
  both stay within `crates/mycelium-mlir/`).
- **M-728 (BitNet acceleration)** -- depends on M-725 (needs the lowering path to hook into).
  May run in parallel with M-727 if M-725 has landed.
- **M-729 (differential durability)** -- integration gate; runs after M-725...M-728 are done.

Collision surface: `crates/mycelium-mlir/` only. Leaves must partition by module/file within
that crate (e.g. M-726 owns `crates/mycelium-mlir/src/passes/`, M-727 owns
`crates/mycelium-mlir/src/jit.rs`, M-728 owns `crates/mycelium-mlir/src/accel/`).

## Sequencing and dependencies

```
RFC-0029 Accepted -- GATE
  |
M-725 (full libMLIR integration -- subsumes E6-1 remaining)
  |           |
M-726       M-727   (parallel: optimization passes / JIT)
  |
M-728 (BitNet packed-ternary acceleration)
  |
M-729 (three-way differential durability: interp == AOT == JIT)
```

**Pre-flight check for M-725 (mandatory before branching):** Confirm ADR-019 / libMLIR
binding status in the current environment. DN-15 ss1 records that real ternary-dialect lowering
was libMLIR-gated at M-373. If still gated, M-725 must either stage a direct-LLVM-advanceable
increment first (per DN-15 ss2) or FLAG the block before spawning leaves.

## Definition of Done

- [ ] RFC-0029 reaches **Accepted** (design gate cleared).
- [ ] M-725: `crates/mycelium-mlir/` provides complete ternary-dialect to `arith`/`func` to
  LLVM dialect lowering; the `mlir-dialect` skeleton is replaced by a real binding; E6-1
  remaining tasks are subsumed and their issues closed.
- [ ] M-726: at minimum inlining, CSE, and DCE passes are implemented; each pass is reified as
  an EXPLAIN-able transform (no black-box heuristics); differential tests confirm the optimized
  output is semantically equivalent to unoptimized (interp == AOT with/without passes, `Empirical`).
- [ ] M-727: JIT is available for dynamic / VSA workloads; it is never silently selected (must
  be an explicit mode -- consistent with the never-silent G2 rule); correctness obligation:
  JIT result == interpreter result (`Empirical`).
- [ ] M-728: BitNet packed-ternary acceleration is behind an explicit capability flag; the
  reference ternary path always produces the same result (`Empirical` differential); graceful
  degradation on hardware without the acceleration is never-silent.
- [ ] M-729: the three-way differential (interp == AOT == JIT) is mutant-witnessed
  (`cargo-mutants` / `cargo-fuzz` smoke confirms the test suite catches codegen divergence).
- [ ] `just check` green on `crates/mycelium-mlir/` with all new features enabled.
- [ ] E15-1 issue status -> `done`; CHANGELOG entry (append-only; honest guarantee tags;
  "implemented (native-path), pending ratification" framing where RFC-0029 is not yet Enacted).

## Landing

Wave-land via `/wave-land` on the `claude/head/aot10` head. PR into `integration` (full
`just check` + honesty review) then squash-PR into `main`. Orchestrator reconciles
`CHANGELOG.md`, `docs/Doc-Index.md`, `docs/api-index/`, `tools/github/issues.yaml` after
the octopus merge of leaves. Run `just docs-index` to regenerate `docs/api-index/` after
any public API change in `mycelium-mlir`.
