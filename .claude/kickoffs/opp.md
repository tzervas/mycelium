# Kickoff `opp` — Phase-I opportunistic Mycelium ports (ready-now; non-gating)

> **UID:** `opp` · **Basis:** **ADR-038** (Proposed) §2.2/§2.5 + the umbrella roadmap
> `docs/planning/road-to-1.0.0-and-mycelium-rewrite.md` **§6** · RFC-0031 §5 **D5/D6** (the per-op
> stability bar + oracle retention) · `docs/spec/stdlib/self-hosting-readiness.md` §0 (~19/26
> expressible, `Empirical`) · DN-34 §8 (transpiler ≈12.4% union coverage, `Empirical` — accelerant,
> not gate) · the `std_result.rs`/`std_option.rs` harness precedent
> (`crates/mycelium-l1/tests/`) · `docs/planning/self-hosting-port-ledger.md`.
> **Planned by:** Fable (ADR-038 §2.7); **implemented by:** Sonnet/Haiku per the PM table.
> **References the doc-maintenance contract** (`_doc-maintenance.md`) in its DoD.
> **Posture:** welcome, honest, and **never the release gate** (ADR-038 §2.2). Mass porting of the
> corpus waits for `grm` (H2a); these ~9 crates are small enough to absorb grammar follow-ons if
> those land after (`Declared` risk, accepted — roadmap §6).

## Goal

Port the ~9 pure/structural `mycelium-std-*` crates that the surface can express **today** — no H1
enabler needed — as `.myc` phyla/nodules under the RFC-0031 D5 method: **verify-the-surface first**,
pre-port polish (clean the ambiguous Rust — ADR-038 §2.5), transpiler-assisted where coverage
genuinely helps (progressive hardening: **start smallest**), hand-finished, then the
`std_result.rs`-pattern harness (nodule loaded verbatim via `include_str!`) with **three-way
execution (L1-eval ≡ L0-interp ≡ AOT where forms close) + differential vs the Rust reference
oracle** (D6: the Rust crate is retained, not retired — retirement is post-1.0, M-867).

Candidate set (per the Phase-I plan; **each is verified against the surface before its port task
starts — a candidate that needs an enabler is FLAGged back to `enb`, not forced**), ordered
smallest-first by non-test LOC (`Empirical`, 2026-07-01):

`diag` (531) → `core` (623) → `select` (1105) → `swap` (1118) → `recover` (1380) → `error` (1606)
→ `testing` (1671) → `ternary` (1924) → `spore` (2613)

*(The landed 8 nodules — cmp/option/result/iter/collections/text/fmt/math — are disjoint from this
set. The roadmap's candidate sketch also names `convert`; it is NOT in this wave's set — FLAG it to
the port ledger as a next-wave candidate rather than growing this scope.)*

## Scope

**In:** the 9 crate-port tasks + the shared harness-fixture generalization + the ledger close-out.
Each port = pre-port polish (Rust-side clarity commits, no behavior change) → transpile-assist →
hand-finish → harness + differential → honest per-op tags carried over (VR-5 — a tag never
upgrades in translation).

**Out:** retiring any Rust crate (D6 — the oracle stays; M-867 is post-1.0); porting anything that
needs an H1 enabler (that's `enb`'s exit criteria, then a later wave); transpiler *feature work*
beyond what a port genuinely exercises (transpiler hardening has its own doctrine, ADR-038 §2.5 —
FLAG demand data to E18-1 instead); any mass-port scaling past this set (gated by `grm`).

## Swarm method + model tiering (ADR-038 §2.7)

**Parallel-leaf Sonnet/Haiku swarm** — the textbook disjoint-ownership case: each leaf owns one
crate's port artifacts (`lib/std/<name>.myc` + `crates/mycelium-l1/tests/std_<name>.rs` + Rust-side
polish commits confined to `crates/mycelium-std-<name>/`). Shared files (the harness fixture
M-925, workspace wiring, `CHANGELOG`/ledger/indices) are **orchestrator-owned**; leaves FLAG.
Model per the table: Sonnet sets the pattern on the first two ports and takes the semantics-heavy
crates; Haiku takes the structural mid-size ports once the pattern is fixed. **Progressive
hardening:** land `diag` + `core` first (sequentially — they calibrate the pattern + the
transpiler's real assist rate), then fan out the rest in parallel. One isolated worktree per leaf
(mitigation #11); commit/push split (#12); scoped PRs to `dev` via `/pr-land`, sequential where
they share `issues.yaml` rows (mitigation #6).

## PM decomposition — bite-sized tasks

Proposed M-ids **M-925…M-935** (next-free after `grm`'s block; re-verify at minting —
mitigation #1). None minted by this doc. Every port task's DoD includes the same five rows (stated once here,
cited as "**D5 rows**" below): (1) surface-check recorded before porting (`Empirical` — or the task
STOPS and FLAGs); (2) pre-port polish committed separately Rust-side, behavior-neutral (tests
prove it); (3) `.myc` nodule + `include_str!` harness, three-way where forms close; (4)
differential vs the Rust oracle green (D5 bar; signature frozen); (5) per-op tags carried at the
same strength, transpiler-assist fraction recorded honestly in the port ledger.

| M-id (proposed) | Task | User story | Definition of Done | Model | depends_on |
|---|---|---|---|---|---|
| M-925 | **Generalize the port harness** — extract the `std_result.rs`/`std_option.rs` pattern (verbatim `include_str!` load, three-way execution, Rust-reference differential) into a reusable, data-driven fixture so each new port is *cases, not bespoke logic* (house test-layout rule) | As a port leaf, I want a drop-in harness fixture, so that every port ships the same three-way + differential rigor without re-deriving it | Fixture landed; `std_result`/`std_option` migrated onto it (proof it generalizes); a new-port checklist (the D5 rows) embedded as the fixture's doc; `cargo test -p mycelium-l1` green | Sonnet | — |
| M-926 | **Port `std-diag`** (531 LOC — smallest; pattern-setter #1): structural diagnostics | As a stdlib author, I want `diag` self-hosted, so that diagnostics are expressed in the language they diagnose | D5 rows; measured transpiler-assist rate recorded (`Empirical` — calibrates the wave) | Sonnet | M-925 |
| M-927 | **Port `std-core`** (623 LOC; pattern-setter #2 — note: the *stdlib* `std-core` crate, not the kernel `mycelium-core`, which stays Rust per RFC-0031 D1) | As a stdlib author, I want the stdlib's own core nodule self-hosted, so that the dogfooding story starts at its root | D5 rows; any D1-boundary item found inside it FLAGged, never ported past | Sonnet | M-926 (pattern calibrated) |
| M-928 | **Port `std-select`** (1105 LOC): selection/decision surface | As a certified-mode user, I want `select`'s reified selections expressed in `.myc`, so that EXPLAIN-able choice logic is itself inspectable language code | D5 rows; EXPLAIN behavior differentially identical | Haiku | M-927 |
| M-929 | **Port `std-swap`** (1118 LOC): the never-silent representation-change surface | As a language user, I want `swap` self-hosted with its never-silent guarantees intact, so that the language's signature feature is written in the language | D5 rows; never-silent swap semantics covered by reject-case conformance (no silent path survives translation — G2) | Haiku | M-927 |
| M-930 | **Port `std-recover`** (1380 LOC): recovery/fallback surface | As a stdlib author, I want `recover` self-hosted, so that error-recovery combinators are dogfooded | D5 rows | Haiku | M-927 |
| M-931 | **Port `std-error`** (1606 LOC): error taxonomy + construction | As a stdlib author, I want `error` self-hosted, so that the error model the stdlib throws is defined in-language | D5 rows; error-formatting parity in the differential | Sonnet | M-927 |
| M-932 | **Port `std-testing`** (1671 LOC): the structural testing surface | As a contributor, I want `testing` self-hosted, so that `.myc` code can be tested by `.myc` infrastructure | D5 rows; the ported harness can express at least one existing conformance case (self-application smoke) | Sonnet | M-927 |
| M-933 | **Port `std-ternary`** (1924 LOC): balanced-ternary value surface | As a language user, I want `ternary` self-hosted, so that a paradigm pillar is expressed in the language that claims it | D5 rows; numeric edge cases (carries, bounds) in the differential corpus; tags at kernel strength, never above (VR-5) | Sonnet | M-927 |
| M-934 | **Port `std-spore`** (2613 LOC — largest; last by progressive hardening): artifact/packaging surface | As a package author, I want `spore` self-hosted, so that the deployable-artifact model is dogfooded end-to-end | D5 rows; content-address behavior differentially identical (no identity drift — the hashes must match the oracle's) | Sonnet | M-928…M-933 (wave tail) |
| M-935 | **Ledger + readiness close-out** — update `self-hosting-port-ledger.md` + the dogfooding assessment with measured `Empirical` rates (assist %, LOC, defects found by the differential); record which candidates failed the surface check (if any) and why | As the maintainer, I want the wave's real costs and outcomes on the record, so that the Phase-II mass-port plan is grounded in data, not the PoC's estimates | Ledger rows for all 9 (landed or FLAGged-blocked, none silent); assessment doc updated; readiness §0 crate counts refreshed; issue statuses current | Haiku | all ports above |

## Definition of Done (kickoff)

- Each of the 9 crates either ported (D5 rows all green) or **explicitly FLAGged blocked** with the
  named missing enabler routed to `enb`/the ledger — never forced, never silently dropped (G2).
- Rust oracles retained (D6); no crate retirement; no release-gating claim made anywhere ("welcome,
  not the gate" — ADR-038 §2.2).
- Transpiler-assist fractions measured + recorded per crate (`Empirical` — feeds the ADR-038 §2.5
  ROI doctrine and the E18-1 demand data).
- Doc-maintenance per `_doc-maintenance.md`; ports' issues minted at kickoff (slots re-verified);
  `CHANGELOG`/ledger/indices reconciled at integration tier.

## Prerequisites

1. **`acy` (H0) landed** (all Phase-I work lands under the cycle gate). **`enb` is NOT a
   prerequisite** — this set was chosen precisely because it needs no H1 enabler (each task
   re-verifies that claim first and FLAGs instead of waiting).
2. **ADR-038 ratification** for the wave's framing; the D5/D6 method itself is already ratified
   (RFC-0031), so the pattern-setters (M-925/M-926) are sound to start on maintainer go-ahead.
3. Transpiler availability as an *assist* (`crates/mycelium-transpile`, landed — `trx`); no
   transpiler feature work is owed to this wave (accelerant, not gate).
