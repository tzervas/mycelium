# Design Note DN-20 — Tiered Testing & Change-Scoping

| Field | Value |
|---|---|
| **Note** | DN-20 |
| **Status** | **Accepted** (maintainer-ratified 2026-06-21) |
| **Feeds** | `just check` / `just ci` (the CI entrypoint, unchanged); `just test-fast` (new pre-commit tier); `just check-full` (new release/durability tier); M-654 / ADR-021 Gate A3 (the WS8 durability gate — mutants + proptest + fuzz — now reachable via `just check-full`); the swarm pre-flight pattern in CLAUDE.md (branch-hygiene, ties to mitigations #5/#7) |
| **Date** | June 21, 2026 |
| **Decides** | The local↔CI test discipline: **three change-scoped, heavy-gated test tiers** (`just test-fast` · `just check` · `just check-full`) run through **cargo-nextest** (with a `cargo test` parity fallback), with **proptest case-tiering** (low every commit, high on release) and a **conservative change-scoping** that may over-test but never under-tests. The everyday loop gets fast; the release gate stays fully durable. No property/bound test is removed — only its *case count* is tiered (VR-5). |
| **Task** | Testing-efficiency + swarm pre-flight methodology (maintainer-ratified this session) |

> **Posture (honesty rule / VR-5).** The load-bearing invariant of this note is that **no
> property/bound test is ever dropped or weakened** — the property tests are the *empirical basis*
> for the per-op guarantee tags (`Empirical`/`Proven` rows), so tiering their **case count** (low on
> every commit, full on release) keeps every bound exercised every commit and at full statistical
> power on release. Coverage is **focused, never removed**. The change-scoping is conservative *by
> construction*: it can only ever *widen* a run to the full workspace, never narrow it past what the
> diff touches, and `just check-full` always runs everything — so a fast-tier scoping decision can
> never cause an under-test that a release would miss. Grounded in source (`scripts/checks/test.sh`,
> `scripts/checks/changed-crates.sh`, `crates/mycelium-numerics/tests/properties.rs`) and the
> CLAUDE.md house rules (KC-3 small kernel, local↔CI parity, G2 never-silent).

---

## 1. Problem

`just check` ran the **full** workspace test suite (`cargo test --workspace --all-features`) plus
every non-test gate on **every** invocation — pre-commit, mid-task, and CI alike. As the workspace
grew to ~45 crates with proptest suites and integration tests, the everyday edit→check loop became
slow enough to discourage running it, which is exactly the failure mode the local↔CI-parity
discipline exists to prevent. The durability gates (cargo-mutants, cargo-fuzz) were deliberately
*outside* `just check` (M-654), so there was no single "run the heavy gate before release" entrypoint
either.

The ask: **make `just check` much faster while remaining just as effective**, and give the heavy
durability work a first-class home — without ever trading away the honest, per-op guarantee basis.

## 2. Decision — three tiers

| Recipe | Tier | Scope | Tests | Proptest cases | Heavy gates | When |
|---|---|---|---|---|---|---|
| `just test-fast` | **0** | change-scoped crates only | unit + regression/witness (`--lib`) | — (none) | no | every commit (fastest) |
| `just check` | **1** | change-scoped crates **+ reverse-deps** | unit + regression/witness + integration + proptest + doctests | **LOW** (`PROPTEST_CASES=8`) | no (skips mutants + fuzz) | default; **local↔CI parity** |
| `just check-full` | **2** | **full workspace** | all of the above | **HIGH** (`PROPTEST_CASES=256`) | **mutants + fuzz smoke** | release / nightly / durability |

- **`just check` stays the CI entrypoint.** `just ci` is still `just check`, and
  `.github/workflows/checks.yml` still runs `just ci` (manual-dispatch, advisory — unchanged). Tier 1
  additionally runs the always-on non-test gates that already existed in `scripts/checks/` (fmt,
  clippy, markdown, doc-status, doc_refs, gh-issues sync `--validate`, deny/audit, …) exactly as
  before — only the *test* step is now scoped + case-tiered.
- **Tier 0 (`test-fast`)** is the pre-commit fast path: only the crates the diff touched, only their
  unit/regression tests (no integration, no proptest, no doctests, no mutation/fuzz). On a
  single-crate change this is sub-second.
- **Tier 2 (`check-full`)** is the durability gate (the M-654 WS8 work): the full workspace at high
  proptest cases, then `just mutants` (cargo-mutants over the trusted base) and a `cargo-fuzz` smoke
  (`scripts/checks/fuzz-smoke.sh`, one target, 60 s, skip-graceful when nightly/cargo-fuzz absent).
  Slow by design — run deliberately.

## 3. Change-scoping (`scripts/checks/changed-crates.sh`)

The fast/default tiers compute the workspace crates affected by the working diff and emit
`-p <crate>` selection args (or a conservative full-run / empty-selection sentinel):

1. **Diff** the working tree (committed `merge-base..HEAD` ∪ staged ∪ unstaged ∪ untracked) against a
   base ref (`$MYC_BASE_REF` → `origin/main` → `main`).
2. **Map** each changed path to its workspace crate by **longest manifest-directory prefix**, read
   from `cargo metadata --no-deps` (the resolver's own data — an *exact* mapping, not a heuristic).
3. **Expand** that touched set to its **reverse-dependency closure** (every workspace crate that
   transitively depends on a touched crate), BFS over the intra-workspace edges from a full
   `cargo metadata` resolve graph. A change to a dependency therefore re-runs its dependents.
4. **Emit** the sorted `-p` args.

**Conservative fallback to `--workspace`** (the never-under-test safety) fires when **any** of: a
shared/root file changed (root `Cargo.toml`, `Cargo.lock`, `.cargo/`, `rust-toolchain.toml`,
`deny.toml`, `.gitattributes`, `justfile`, `scripts/`, `.github/`), change-detection cannot run (no
`git`/`cargo`/`jq`), no base ref exists, or `cargo metadata` fails. When the diff maps to **no** crate
(docs-only, or a clean tree) the script emits a distinct `--no-changes` sentinel and the test step is
skipped for that tier (the non-test doc gates in `just check` still run).

> **Safety contract.** The fast tier may **over**-test (e.g. a shared-file touch widens to the whole
> workspace) but must never silently **under**-test. The only `Declared` element is the shared-file
> trigger list — a maintainer-chosen set that can only ever *widen* to `--workspace`, so it cannot
> cause an under-test. Everything else (path→crate mapping, reverse-dep closure) is an exact graph
> computation. And `just check-full` runs the full workspace regardless, so a release catches
> anything a fast-tier scoping ever excluded.

The script is **offline + deterministic** (only `git`, `cargo metadata --offline`, `jq`; no network,
no clock), prints narration to stderr and the selection to stdout (clean for `$(...)` capture), and
is **skip-graceful** (exits 0 on every fallback).

> **Underscore-normalization gotcha (recorded so it is not silently re-broken).** cargo's
> `resolve.nodes[].deps[].name` are the *normalized* crate names (hyphens → underscores, e.g.
> `mycelium_core`), whereas `packages[].name` is the canonical manifest name (`mycelium-core`). The
> reverse-dep edge construction maps the normalized dep name back to the canonical name before
> recording an edge; without this the intra-workspace join silently misses **every** edge, collapsing
> the closure to the empty set — a silent under-test. Verified: a `mycelium-numerics` change expands
> to 24 crates, a `mycelium-core` change to 44.

## 4. cargo-nextest (with a `cargo test` fallback)

The runner is **`cargo nextest run`** when `cargo-nextest` is installed (faster, parallel
test execution), else **`cargo test`** — so local↔CI parity holds whether or not nextest is present.
`just setup` (via `scripts/install-tools.sh`) installs nextest best-effort (`cargo install --locked
cargo-nextest`, idempotent, skip-graceful); when it can't (offline / install failure) the suite
falls back to `cargo test` and is otherwise identical. nextest is therefore a pure speed-up, never a
gate.

**nextest does not execute doctests.** To avoid silently dropping doctest coverage, the `check`/`full`
tiers add an explicit `cargo test --doc` pass (cheap), scoped to the same crate selection. The fast
tier intentionally omits doctests (and integration/proptest) for speed.

## 5. Proptest case-tiering

Proptest suites default low and honor `PROPTEST_CASES`; the tiers set it per the table in §2 (Tier 0
runs no proptest; Tier 1 low = 8; Tier 2 high = 256). The VSA suites already default low (the outer
`cases` is the *batch* count, default 1, distinct from the inner `TRIALS` seed loop — unchanged, they
remain the honest empirical basis for the SC-2 capacity bounds).

**Numerics correction (the one substantive FLAG).** `mycelium-numerics/tests/properties.rs` *claimed*
to honor `PROPTEST_CASES` but its `cfg()` used a `ProptestConfig { cases: 20_000, .. }` struct literal
— which **hardcodes** the count and silently ignores the env var (proptest only auto-reads
`PROPTEST_CASES` for a `ProptestConfig::default()` whose `cases` is left untouched). It is now read
**explicitly** (parse `PROPTEST_CASES`, low default `DEFAULT_CASES = 8`), so the count is genuinely
tiered. Verified by timing: the same property runs in 0.00 s at 4 cases and 0.82 s at 20 000 cases —
i.e. the env var now drives the count (previously both would have taken the hardcoded 20 000-case
time). The two mutant-witness guard configs (`cases: 1000`) were routed through the same
`witness_cfg()` helper for consistency.

> This correction **strengthens** honesty: the bound is still exercised every commit (low cases) and
> at full statistical power on release (high cases). No bound lost coverage — the env knob simply now
> works as documented.

## 6. Honesty guardrail (load-bearing)

The property tests are the empirical basis for the per-op guarantee tags on the
`Exact ⊐ Proven ⊐ Empirical ⊐ Declared` lattice (house rule 1 / VR-5). This note tiers their **case
count**, never their **existence**:

- **Every bound is exercised on every commit** (Tier 1, low cases) and **at full statistical power on
  release** (Tier 2, high cases).
- **Regression/witness tests** (the mutant-witnesses that pin each guard) run whenever their crate —
  or a crate it depends on — changes (Tier 0/1 via the reverse-dep closure), and unconditionally at
  release (Tier 2).
- **Coverage is focused, never removed.** No `cargo-mutants` survivor is hidden, no proptest is
  deleted, no guarantee tag is upgraded — the tiers only change *how many cases* and *which crates*
  run *now*, with the full set guaranteed at release.

This keeps the **small, auditable kernel** discipline (KC-3): the change is in the *check tooling*
(`justfile`, `scripts/checks/`), not in the kernel or its guarantees, and it preserves **local↔CI
parity** — `just check` remains the single recipe CI runs, now scoped + case-tiered identically
locally and remotely.

## 7. Consequences

- **Faster everyday loop.** `test-fast` is sub-second on a single-crate change; `check` scopes to the
  touched crates + their dependents at low proptest cases.
- **Durability has a home.** `check-full` is the one entrypoint that runs the full workspace at high
  cases + mutants + fuzz (the M-654 / ADR-021 Gate A3 durability surface).
- **No honesty regression.** The guarantee basis is intact; the only test-behavior *change* is a bug
  *fix* (numerics `PROPTEST_CASES` now honored).
- **Parity preserved.** nextest-or-`cargo test`, scoped-or-`--workspace`, the result set on a release
  (`check-full`) is the historical full suite.

---

## Changelog

- **2026-06-21** — Created (Accepted, maintainer-ratified this session). Records the three-tier
  change-scoped testing methodology (`test-fast` / `check` / `check-full`), cargo-nextest with the
  `cargo test` parity fallback, proptest case-tiering, the conservative change-scoping with its
  never-under-test safety, and the honesty guardrail (VR-5 — case count tiered, coverage never
  removed). Fixes the `mycelium-numerics` proptest `PROPTEST_CASES` hardcode FLAG.
