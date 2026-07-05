# Kickoff `trx` — Rust→Mycelium transpiler PoC (against the *existing* surface + gap-report)

> **✅ LANDED (2026-07-01) — `dev`.** PoC (PR #911) + hardening follow-on (width_cast emission, batch
> mode, 8-twin union backlog). `crates/mycelium-transpile`; M-873 → done; results in **DN-34 §8**
> (§8.5 union: 12.4% expressible across 6 crates; §8.6 option/result are self-hosted, no Rust source).
> Follow-on demand data (E18-1 `needs-design`): unsupported **types** #1 (36%), macros #2, bounded
> generics #3. Deferred (tooling-blocked in-env): macro-expansion (needs nightly+cargo-expand) and the
> `cargo-public-api` baseline. Subsequent work → the Rust-first-everything / no-circular-deps plan.
>
> **UID:** `trx` · **Issue:** M-873 (epic E18-1 self-hosting capstone) · **Basis:** DN-34 (transpiler
> strategy, Draft) · DN-26 (bootstrap plan) · `docs/planning/dogfooding-effort-and-usage-assessment.md`
> (§5a/§5b — the cost model + the crates→transpiler→self-hosted path) · `docs/planning/self-hosting-port-ledger.md`.
> **References the doc-maintenance contract** (`_doc-maintenance.md`) as part of its Definition of Done.

## Goal (maintainer, 2026-07-01)

Get a Rust→Mycelium transpiler **working against the language surface that already exists** — do **not**
wait for full surface maturity. Emit what the current surface can express; **flag** (never guess, G2) what
it can't. Use it to **partially implement** the 26 Rust `mycelium-std-*` crates, and produce — as a
first-class output — the **prioritized surface-feature backlog** (the union of gap-reports) that grounds
E18-1's `needs-design` work with real demand data. "Depends on how we implement it" → the load-bearing
implementation choice is **make the gap-report first-class alongside the `.myc` output.**

## Why this is cheap (the one design lever): diff against known-good

The core-lib slice already has **hand-written `.myc` twins** (M-714–719 `done`): `lib/std/{cmp, option,
result, iter, collections, text, fmt, math}.myc`. So point the transpiler at those crates' **Rust source**
and **diff its output against the existing `.myc`** → correctness is a cheap **diff, not an
execution/differential run**. That removes the most expensive part of a PoC.

- **PoC target: `crates/mycelium-std-cmp` (623 non-test LOC)** — small, and it has a `.myc` twin
  (`lib/std/cmp.myc`). (`std-core` is an equally-good fallback.)
- **Seed from the maintainer's `py2rust` + `py-rust-bridge`** (DN-34) — adapt their AST-walk +
  never-silent compatibility-analyzer architecture; do **not** build the Rust parser greenfield (use
  `syn` for the AST if a local parser isn't reused).

## Scope (hard — one crate, one direction)

**In:** a new `crates/mycelium-transpile` (or `tools/transpile/`) that reads one Rust crate's source →
emits (a) a best-effort `.myc` for expressible constructs + (b) a **structured, never-silent gap report**
(`{file, line, rust_construct, reason_unmappable}`). A construct-mapping table (Rust → Mycelium surface)
covering what `std-cmp` needs. A **diff harness** comparing output to `lib/std/cmp.myc`.

**Out (this PoC):** the other 25 crates; execution/differential-run validation (the diff is the check);
kernel/toolchain/AOT tiers; any new *language* feature (those are flagged, not built — they become the
backlog). Never emit a construct the surface can't express — **flag it** (G2/VR-5).

## Efficiency playbook (the maintainer is usage-constrained — honor this)

- **Model tiering (Hybrid swarm):** Opus orchestrator for design + gap-judgment; **Sonnet** leaf for the
  construct-mapping + emitter; Haiku only for mechanical mapping-table rows. **~1–2 agents — NOT a
  Workflow fan-out** (over-orchestration costs more than it saves at PoC scale).
- **Explore agents for read-heavy scoping** (the `py2rust` architecture, `std-cmp`'s Rust↔`.myc` pair) →
  return conclusions only; protect the orchestrator context (mitigation #6).
- **Change-scoped checks:** `just test-fast` / `cargo test -p mycelium-transpile` during iteration; full
  `just check` only at the end (DN-20).
- **Structured outputs (schemas):** the gap-report + mapping table are schema-validated JSON, not prose.
- **One focused warm session** (5-min prompt-cache TTL); avoid cold restarts + file re-reads (the harness
  tracks file state). Follow `/dev-workflow`; work in an **isolated worktree**.

## Opening moves (turnkey)

1. **Workspace prep:** `git fetch origin dev`; branch a leaf off current `dev` in an **isolated worktree**
   (`claude/leaf/E18-<LEAF>-transpile-poc`); `just setup` (Rust toolchain; add `syn`/`quote` if used —
   that dep is scoped to the new transpiler crate only, KC-3, not the kernel).
2. **Explore** (read-only): the `py2rust`/`py-rust-bridge` transpiler architecture (maintainer to point at
   the repo/path if not vendored) + `crates/mycelium-std-cmp/src/*.rs` ↔ `lib/std/cmp.myc` (the
   ground-truth pair). Return: the construct inventory `std-cmp` uses + the surface features `cmp.myc`
   demonstrates are available.
3. **Build** (Sonnet leaf): the minimal AST-walk + construct-mapping for `std-cmp`'s constructs + the
   gap-report emitter. Run it on `std-cmp`; **diff output vs `lib/std/cmp.myc`**; iterate until the
   expressible fraction matches and every gap is flagged (no silent drops).
4. **Report:** the diff result (what matched), the gap-report (what the surface lacks), and the measured
   **token cost** — this converts the assessment's `Declared` rates to **`Empirical`** (record it in the
   assessment doc + the port ledger).

## Definition of Done

- `mycelium-transpile` transpiles `std-cmp`'s Rust → `.myc`, emitting the expressible fraction + a
  never-silent gap report; `cargo fmt`/`clippy -D warnings`/`test -p mycelium-transpile` green.
- The diff vs `lib/std/cmp.myc` is characterized (matched / refined / flagged), never a silent mismatch.
- The gap report is captured as the **seed of the surface-feature backlog** (a DN entry or issues under
  E18-1), grounding the `needs-design` work with real data.
- **Doc-maintenance (`_doc-maintenance.md`):** update the assessment doc + port ledger with the **measured
  `Empirical` rate**; M-873 → `done`/`in-progress` per what lands; CHANGELOG entry; `doc_refs` valid.
- Land as a scoped PR to `dev` via `/pr-land` (leaf→dev review loop). Tier state at kickoff: `main`
  `2299838`, `integration` `60b0d36`, `dev` `9d02b2e` (all reconciled post-E25/E26).

## Prerequisite the maintainer supplies at session start

The path/repo for **`py2rust` + `py-rust-bridge`** (the architectural seed, DN-34) — if not vendored into
this repo, point the Explore agent at it. Everything else is in-repo and ready.
