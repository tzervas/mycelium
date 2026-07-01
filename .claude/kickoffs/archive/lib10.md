# Kickoff `lib10` — Standard library in Mycelium (E13-1)

> Read `CLAUDE.md` (house rules win) + `.claude/kickoffs/README.md` + `RFC-0031` + `RFC-0016`
> first. This kickoff is **the defining full-language-1.0.0 criterion**: the stdlib and core
> libs beyond the bare Rust kernel are written fully in Mycelium (`.myc`), stable, and fully
> usable.

## Metadata

| Field | Value |
|---|---|
| **UID** | `lib10` |
| **Head branch** | `claude/head/lib10` |
| **Status** | ready |
| **Swarm mode** | Sonnet |
| **Depends on** | E11-1 (type system completeness — traits, HOF, operator syntax must land before ports can compile); `run` ✅ (M-673); `std` ✅ (M-649, `lib/std/result.myc`) |

## Scope

Port the Mycelium stdlib and core libs from Rust-first (`mycelium-std-*`) to fully self-hosted
`.myc` nodules in `lib/std/`. The Rust kernel (`mycelium-core`, `mycelium-l0`, `mycelium-l1`,
`mycelium-swap`) stays Rust — that boundary is defined by RFC-0031. Today only `lib/std/result.myc`
self-hosts; this wave delivers the rest of the Tier-A differentiator modules and the core Tier-B
modules (M-715…M-719), differential-tested and stable.

**Owned directory:** `lib/std/` (new `.myc` nodules + differential test files).
**Read-only for leaves:** `tools/github/issues.yaml`, `CHANGELOG.md`, `docs/Doc-Index.md`,
`docs/api-index/`, workspace `Cargo.toml`, `docs/rfcs/`, `crates/mycelium-std-*`.

## Epic and issue IDs

- **E13-1** — Standard library in Mycelium (self-hosted stdlib + core libs)
  - **M-714** — stdlib composition + phylum layout (RFC-0031 authoring) `[type:design]`
  - **M-715** — core/prelude in Mycelium: Option/Result/cmp/Ord/Eq + iter traits as `.myc` `[type:feature]`
  - **M-716** — collections in Mycelium: Vec/Map/Set as `.myc` with honest bounds `[type:feature]`
  - **M-717** — text + fmt in Mycelium: string handling + formatting as `.myc` `[type:feature]`
  - **M-718** — math + numerics surface in Mycelium: math ops + epsilon/delta surface as `.myc` `[type:feature]`
  - **M-719** — stdlib conformance + stability: differential tests + stable documented API for every std module `[type:verification]`

## Grounding

- RFC-0031 (`docs/rfcs/RFC-0031-Self-Hosted-Standard-Library-Composition.md`) — must reach
  **Accepted** before M-715…M-719 implementation begins; M-714 is its authoring task.
- RFC-0016 (Enacted) — scope/contract/taxonomy; the per-op contract (C1–C6) applies to every
  `.myc` port.
- `lib/std/result.myc` (M-649) — the composition prototype; all ports follow its patterns.
- `docs/spec/stdlib/*.md` — 25 module specs; the design basis for each port.
- RFC-0024 (HOF, implemented Rust-first, pending ratification) — unblocks combinators
  (`map`/`and_then`/`fold`) in M-715.
- RFC-0019 (traits/polymorphism) — unblocks `iter`/`cmp`/`Ord`/`Eq` in M-715.
- G2/VR-5 — never-silent, honest guarantee tags required on every exported op.

## Swarm / parallelization pattern

**Serial gate then parallel leaves.** M-714 (RFC-0031 design) must complete and the RFC must
reach Accepted before any implementation leaf starts. Once Accepted:

- **M-715 (core/prelude)** must land before M-716 (collections) and M-718 (math) because
  they depend on `Option`/`Result`/`Ord`/`Eq`/`iter`.
- **M-716, M-717, M-718** are mutually independent and may run as parallel Sonnet leaves
  (disjoint nodule families: `lib/std/collections/`, `lib/std/text/`, `lib/std/math/`).
- **M-719 (conformance + stability)** is the integration gate: runs after all ports land.

Collision surface: only `lib/std/` directory. Each leaf owns a disjoint subdirectory.
Orchestrator owns: `lib/std/README.md` (if it exists), `CHANGELOG.md`, `docs/Doc-Index.md`,
`docs/api-index/`, `tools/github/issues.yaml`.

## Sequencing and dependencies

```
E11-1 (type system — traits/HOF/operator syntax) — prerequisite
  ↓
M-714 (RFC-0031: design + Accepted) — GATE
  ↓
M-715 (core/prelude: Option/Result/cmp/Ord/Eq/iter)
  ↓  ↓  ↓
M-716  M-717  M-718   (parallel: collections / text+fmt / math+numerics)
  ↓  ↓  ↓
M-719 (conformance + stability: differential tests, API freeze)
```

Language surface readiness preconditions per module:
- **M-715 core/prelude** — requires RFC-0024 HOF (for combinators) + RFC-0019 traits (for
  `Ord`/`Eq`/`iter`) to be implemented Rust-first at minimum.
- **M-716 collections** — requires M-715 (depends on `Option`/`iter`).
- **M-717 text+fmt** — requires M-715 (depends on `Result`/`Option`).
- **M-718 math+numerics** — requires RFC operator-syntax surface (DN-23 / future RFC-0025)
  if math ops use infix; otherwise M-715 sufficient.

## Definition of Done

- [ ] RFC-0031 reaches **Accepted** (M-714 complete).
- [ ] `lib/std/core.myc` (or equivalent path) — Option/Result/Ord/Eq in `.myc`, differential
  tests pass (`.myc` eval ≡ Rust `mycelium-std-core` reference), honest guarantee tags,
  never-silent G2, EXPLAIN-able selections.
- [ ] `lib/std/iter.myc` — the iter traits as `.myc`, differential-tested.
- [ ] `lib/std/collections.myc` (Vec/Map/Set) — honest bounds, differential-tested.
- [ ] `lib/std/text.myc` + `lib/std/fmt.myc` — string handling + formatting, differential-tested.
- [ ] `lib/std/math.myc` + `lib/std/numerics.myc` — math ops + epsilon/delta surface, honest
  tags, differential-tested.
- [ ] M-719: every ported module has a stable, documented API; the corresponding
  `mycelium-std-*` Rust crate is deprecated or retired per RFC-0031's stability bar.
- [ ] `just check` green on the full `lib/std/` tree.
- [ ] E13-1 issue status → `done`; CHANGELOG entry (append-only, "implemented Mycelium-lang,
  pending ratification" framing where RFC-0031 is not yet Enacted).

## Landing

Wave-land via `/wave-land` on the `claude/head/lib10` head. PR into `integration` (full
`just check` + honesty review) then squash-PR into `main`. Orchestrator reconciles
`CHANGELOG.md`, `docs/Doc-Index.md`, `docs/api-index/`, `tools/github/issues.yaml` after
the octopus merge of leaves.
