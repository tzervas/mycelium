# Design Note DN-17 — Codebase Housekeeping & DRY Survey

| Field | Value |
|---|---|
| **Note** | DN-17 |
| **Status** | **Draft** (2026-06-19; M-376) |
| **Feeds** | M-376 (the tracked housekeeping wave this note scopes); house rule #5 (SOLID · DRY · KISS · YAGNI · SoC · Law of Demeter); KC-3 (small auditable kernel) |
| **Date** | June 19, 2026 |
| **Decides** | *Planning capture, advisory (DN-08/DN-07 posture) — not a ratified decision.* Records a grounded read-only duplication survey of the Rust workspace and a priority-ordered, risk-tagged extraction plan for a **future** behaviour-preserving housekeeping wave. No code is changed by this note; nothing is extracted here. |
| **Task** | M-376 — codebase housekeeping / DRY extraction (design-first) |

> **Posture (honesty rule / VR-5 / YAGNI).** Every finding below is grounded in source (file +
> approximate count). The survey is conservative: it flags only **genuine, stable** repetition
> (recurring ≥3× and unlikely to churn), and it explicitly marks duplication that must **not** be
> extracted (intentional design documentation, per-module contracts, well-encapsulated helpers).
> Two standing constraints: **(1)** any extraction must be a behaviour-preserving no-op, verified by
> the existing interp↔native differential (M-302) + the workspace test suite green — that machinery
> is the guard that makes a fearless refactor honest; **(2)** we adopt **standard Rust conventions**
> now (shared crates, `[workspace.dependencies]`, derive/macro where idiomatic) and defer
> *Mycelium-native* conventions until there is real Mycelium code to write them in — self-hosting is
> not yet established (DN-14: 5/11 features gate-fail).

---

## 1. Why this note exists

The codebase has grown to ~43 crates (kernel + reference interpreter + 23-crate Rust-first stdlib +
toolchain). At this scale, repeated boilerplate both inflates the footprint and risks divergence
(the same contract implemented slightly differently in N places). A periodic DRY pass — extracting
genuinely-repeated code into shared helpers/crates aligned with Rust conventions — keeps the kernel
small (KC-3) and the corpus auditable (house rule #5). This note is the **grounded scope** for that
pass so the work is targeted, not speculative; M-376 tracks the execution.

## 2. Findings (grounded survey)

### 2.1 Workspace external-dependency duplication — P1 (now-safe)
No `[workspace.dependencies]` section exists in the root `Cargo.toml`. The same external deps are
repeated across many crate manifests: `serde = { version = "1", features = ["derive"] }` (~11×),
`serde_json = "1"` (~14×), `blake3 = "1"` (~7×). Hoisting these to `[workspace.dependencies]` and
referencing them as `serde.workspace = true` is **standard Rust practice**, removes ~20 LOC of
manifest repetition, and — more importantly — makes version bumps single-point. Zero runtime/semantic
risk. The root `Cargo.toml` is orchestrator-owned (it is the wave collision surface).

### 2.2 `xtask` path-deps lack a version field — P1 (now-safe, advisory cleanup)
`xtask/Cargo.toml` declares 5 path-deps as `mycelium-* = { path = "../crates/..." }` with no
`version`, which `cargo deny` reports as `warning[wildcard]: found 5 wildcard dependencies for crate
'xtask'` (observed in `just check`'s `deny` step; advisory, non-blocking). Adding `version = "0.0.0"`
alongside each `path` silences the lint and matches the convention the rest of the workspace can
adopt. Tiny, zero-risk.

### 2.3 MLIR differential test scaffolding — P2 (low risk, modest win)
The interp↔AOT/native differential harness repeats small corpus/helper builders (`byte()`, `tern()`,
`policy()`, `corpus()`) across `crates/mycelium-mlir/tests/{differential,native_differential,
jit_differential,wrong_layout,inject_hotswap}.rs` — ~80–120 LOC of near-identical helper definitions.
Factor them into a shared in-crate test module (`crates/mycelium-mlir/tests/common/mod.rs`) and
`use` it from each test file. Test-only, APIs stable since M-151→M-302, **zero semantic risk**;
~100 LOC saved. (The Wave-5 `data_corpus` additions in `native_differential.rs` join the same pattern.)

### 2.4 Per-crate error boilerplate — P3 (defer; YAGNI at design phase)
8 dedicated `error.rs` files: the six `mycelium-std-*` (`text`/`io`/`fs`/`iter`/`collections`/`content`)
sum to **1,451 LOC**, plus `mycelium-numerics` (303) and `mycelium-l1` (30) → **1,784 LOC across all
eight** (measured `wc -l`, 2026-06-19); plus error enums inside several `lib.rs` (`mycelium-core`,
`-cert`, `-build`, …). Each hand-rolls `impl Display` + `impl std::error::Error` (+ optional
`source()` chaining) + a near-identical test suite (`*_is_std_error`, `*_display_includes_*`).
Roughly ~600 LOC of `Display`/`Error`/test boilerplate is structurally repeated.

This is the **largest** nominal footprint, but the **riskiest to extract now** (YAGNI / coupling):
error definitions are spec-driven and still evolving per issue scope (RFC-0013 loci, RFC-0016 §4.5
honesty requirement; M-514/M-524 etc.). A premature shared error crate or derive macro would couple
modules that the spec keeps loosely coupled. **Recommended staging:** (a) the *safe partial* step now
is a shared **test-helper** (e.g. `fn assert_is_std_error(e: &dyn std::error::Error)`) to cut the
test duplication without touching the error types; (b) revisit a derive/macro (thiserror-style, or a
minimal in-house macro to keep the dependency floor low) only **after** the stdlib specs ratify and
error shapes stabilise. Do not extract the error *types* during the design phase.

## 3. Explicitly NOT extraction targets (intentional / well-factored)

- **`mycelium-std-*` ambient docstring blocks** (~40 LOC × 23 crates) — intentional per **M-540**
  (each module declares its RFC-0012 ambient contract + honesty crux). Required transparency, not
  duplication to remove.
- **Per-module guarantee matrices** (`guarantee_matrix::MATRIX`) — each crate declares its *own*
  per-op guarantee table (RFC-0016 §4.5). Intentionally per-module, not shared code.
- **`llvm.rs` `Ssa`/`Bbc` counters** — small, self-contained, already encapsulated; not repeated.
- **`scripts/checks/*`** — already share `scripts/lib.sh` (good prior art for the pattern).

## 4. Priority-ordered plan (for M-376; behaviour-preserving)

| Pri | Item | Target | ~LOC | Risk | Sequencing |
|---|---|---|---|---|---|
| **P1** | serde/serde_json/blake3 repetition | `[workspace.dependencies]` + `*.workspace = true` | ~20 | minimal (std Rust) | first; one PR |
| **P1** | xtask path-deps wildcard | add `version = "0.0.0"` to the 5 path-deps | ~5 | minimal | with P1 |
| **P2** | MLIR test corpus helpers | `crates/mycelium-mlir/tests/common/` | ~100 | minimal (test-only) | second; verified by the differential staying green |
| **P3** | error test boilerplate | shared `assert_is_std_error` test helper | ~100 | low | third (types untouched) |
| **P3** | error `Display`/`Error` types | derive/macro **deferred** | ~400 | **YAGNI now** | post-stdlib-ratification |

**Execution discipline (M-376):** survey-first (this note); one PR per item group; each PR a pure
no-op proven by `cargo test --workspace` + the M-302 differential + full `just check` green; honest
per-op guarantee tags unchanged (a refactor never changes a guarantee). Order P1→P2→P3; stop at the
YAGNI line.

## 5. Honest caveat (what is premature)

The error-*type* unification (the biggest LOC line) is deliberately deferred: during the design phase
the std specs are still pending ratification (DN-16) and error shapes change per issue. Extracting
them now would trade visible duplication for invisible coupling — the worse failure under the
house DRY/KISS/YAGNI rule. The footprint win that is **safe today** is the manifest dedup (P1) and the test-helper
consolidation (P2); the rest waits for the specs to stabilise. No item here is a blocker; this is
maintenance, scheduled at a stable point (DN-08 coarse-granularity posture).

---

## Changelog

- **2026-06-19 (Draft; M-376):** Initial planning capture. Read-only DRY/duplication survey of the
  Rust workspace; priority-ordered, risk-tagged extraction plan for a future behaviour-preserving
  housekeeping wave. Grounded in a workspace survey; nothing extracted here.
