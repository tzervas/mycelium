# Kickoff `std` — first self-hosted generic stdlib nodule (`std.result`)

> Unblocked by **M-673** (monomorphization landed). Read `.claude/agent-context.md` + `CLAUDE.md`
> (house rules win) + `.claude/kickoffs/README.md` + **`run.md` §What M-673 delivered** (the L1
> base changes this kickoff builds on) first.

## Mission — M-649 (#284)

Author **`std.result`** — the first self-hosted generic stdlib nodule — in `.myc` L1 syntax.
Scope: `Result<A,E>` type + `is_ok`, `is_err`, `unwrap_or`. Single Sonnet leaf. Docs-only + `.myc`
file; no `crates/mycelium-l1/` Rust edits.

## Acceptance criteria

1. **Nodule file** at `lib/std/result.myc` (or equivalent standard path; pick consistently with
   any existing stdlib skeleton; if none exists, establish the path and FLAG it).
2. **`myc-check` exits 0** on the nodule in isolation.
3. **Differential tests** for each exported fn (`is_ok`, `is_err`, `unwrap_or`): Mycelium-lang
   value on the monomorphized env **≡** Rust reference implementation, verified via the M-210
   three-way checker. One test per fn, one test per interesting edge case (e.g. `unwrap_or` on
   `Err`).
4. **M-501 contract** met throughout: never-silent G2 (no silent `unwrap`/fallible path);
   honest per-op guarantee tags (`Declared` for the type-level contract, `Empirical` for
   differential agreement); `EXPLAIN`-able selections.
5. **HOF gap flagged** (see below) — must appear as a FLAG comment in the nodule and in this
   kickoff's closing note, not silently omitted.
6. **DN-14 Status → Resolved** (append-only) — record remaining gate-fails honestly:
   wild/FFI execution staged; refinement stage-1b/2 future; HOF deferred (see below).

## FLAG — honest HOF gap (v0 scope boundary)

v0 has **no surface function type** and application is **first-order only**. Higher-order
combinators (`map`, `and_then`, `fold`) **cannot self-host yet** — they require function types
as values, which are not yet in the L1 surface. This is a `Declared` limitation, not a bug.
**Do not implement stubs that pretend to be HOF.** Record the gap explicitly in the nodule header
and in the DN-14 Resolved note.

## Swarm layout — single Sonnet leaf

**One Sonnet leaf** owns this entire kickoff (`.myc` file + tests + DN-14 Resolved note).
No fractal fan-out needed — the changeset is tightly scoped and has no collision surface with
`srf` (which edits `crates/mycelium-l1/` Rust; this leaf edits only `lib/std/result.myc` +
test files).

**Orchestrator owns** (FLAG up, do not edit): `tools/github/issues.yaml`, `CHANGELOG.md`,
`docs/Doc-Index.md`, `docs/api-index/`, workspace `Cargo.toml`.

## M-673 base requirements (carry-forward from `run.md`)

Branch from the **post-M-673 `dev` tip**. The L1 base includes:
- `crates/mycelium-l1/src/mono.rs` (present; do not delete or move).
- `Env { …, impls: BTreeMap::new() }` required in any test `Env` literals.
- `Ty::Data(String, Vec<Ty>)` — two-field variant.
- `Residual` sites in `elab.rs` are kept as defensive invariants.

## Dependency on `srf`

`std.result` uses only `Result<A,E>` — a generic ADT definable today with the post-M-673 checker.
It does **not** require `consume`/`grow`/`impl` (M-664) or `fuse`/`reclaim`/`tier` (M-667).
**This kickoff may land before or after `srf`** — the `.myc` nodule is fully disjoint from the
`crates/mycelium-l1/` Rust files `srf` edits. No serialization needed.

## Done

M-649 landed on `main`; M-649 issue body + status → `done`; DN-14 → **Resolved** (append-only,
noting: wild/FFI execution staged, refinement stage-1b/2 future, HOF deferred); changelog entry
("implemented Rust-first, pending ratification" framing; honest tags).
