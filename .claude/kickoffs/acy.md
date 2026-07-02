# Kickoff `acy` — Phase-I H0: foundation + acyclic-deps hardening (LANDS FIRST)

> **UID:** `acy` · **Basis:** **ADR-038** (Proposed — function-first strategy) + the umbrella roadmap
> `docs/planning/road-to-1.0.0-and-mycelium-rewrite.md` **§2 (H0)** · dep-graph survey (`Empirical`,
> 2026-07-01, spot-verified at HEAD) · PR #864 (the `mycelium-sched` extraction precedent).
> **Planned by:** Fable (ADR-038 §2.7 — planning tier); **implemented by:** Sonnet/Haiku leaves per the
> PM table below. **References the doc-maintenance contract** (`_doc-maintenance.md`) in its DoD.
> **Sequencing: this kickoff lands FIRST** — every later Phase-I kickoff (`enb`, `grm`, `opp`) lands
> *under* the cycle-enforcement this one installs; retrofitting the invariant after H1/H2 churn would
> mean re-auditing all of it (roadmap §2).

## Goal

Make the 52-crate workspace's **acyclic, downward-only dependency structure enforced by construction**:
the structural gate in `just check` goes **red the moment anyone *introduces* a violation** — a cycle
(normal *or* dev), an upward-layer edge, or an `interp → std-*` edge — with per-edge diagnostics,
while staying **green on clean state** (the guard is proven by negative fixtures, not by breaking
HEAD); resolve the three known `cert` dev-dep cycles;
extract the `mlir → std-runtime` ABI seam; complete the ADR-038 §2.2 publication-hygiene sweep
(`publish = false` on all 52 crates); and fix the M-866 `mono.rs` recursion-safety bug early.

**Current state (`Empirical`, cargo-metadata dig 2026-07-01):** normal deps already form a clean
acyclic 8-stratum DAG (`mycelium-core`: 41 dependents, 0 dependencies). The defects: (1) three
dev-dep cycles centered on `mycelium-cert` — `select →[dev] cert → vsa → select` ·
`cert →[dev] proj → l1 → cert` · `cert →[dev] spore → proj → l1 → cert`; (2) the
`mycelium-mlir → mycelium-std-runtime` upward-layer edge; (3) `publish = false` set on only 3/52
crates. The invariant to preserve: **strictly downward deps; `interp` NEVER depends on any `std-*`**.

## Scope

**In:** the structural check (cargo-deny ban-set and/or an xtask over `cargo metadata`, covering
dev-deps — cargo never rejects dev-dep cycles), wired into `just check`; the three cycle fixes;
the runtime-ABI seam extraction; the invariant DN; the `publish = false` sweep; M-866.

**Out:** everything H1+ (no new prims, no language features — that is `enb`); any stratum
*re-design* (the check encodes the graph as it is, minus the defects); version bumps (crates stay
`0.0.0` per ADR-038 §2.2). If a cycle fix seems to require moving types between crates beyond
cert-local test refactoring, **FLAG it, don't improvise** (a cross-crate type move is an owned-file
decision).

## Swarm method + model tiering (ADR-038 §2.7)

Small **Sonnet swarm** (Sonnet orchestrator, Sonnet/Haiku leaves per the table) — 2–4 leaves max;
this is a hygiene wave, not a fan-out. Partition: the check tasks (M-877→M-880) are one **serial**
mini-lane (same xtask/config files); the cycle fixes, the ABI extraction, the sweep, and M-866 are
**parallel by disjoint crates**. Orchestrator owns the workspace `Cargo.toml`, `justfile`,
`deny.toml`, and all shared indices (leaves FLAG, never edit). One isolated worktree per leaf
(mitigation #11); commit/push split (mitigation #12); land as scoped PRs to `dev` via `/pr-land`.

## PM decomposition — bite-sized tasks

Proposed M-ids verified free at planning time (`grep 'id: M-' tools/github/issues.yaml` — highest
minted is M-876); **re-verify each slot at minting** (mitigation #1). None minted by this doc.

| M-id (proposed) | Task | User story | Definition of Done | Model | depends_on |
|---|---|---|---|---|---|
| M-877 | Stratum-map data file + structural check v0: **normal-deps downward-only** (xtask over `cargo metadata` and/or cargo-deny ban-set), per-edge diagnostics | As a contributor, I want an introduced upward dependency to fail `just check` naming the offending edge, so that the acyclic architecture cannot erode silently | Committed stratum map (data, not prose); check fails on a synthetic upward edge in a test fixture and passes at HEAD; violation output names edge + rule (G2); change-scoped tests green | Sonnet | — |
| M-878 | Extend the check to **dev-deps** (cargo never rejects dev-dep cycles) | As a contributor, I want dev-dep cycles caught by the same gate, so that test-only edges can't reintroduce the `cert` cycle class | Check detects all three known cert dev-cycles when run pre-fix (fixture or historical verification); dev/normal edges distinguished in diagnostics | Sonnet | M-877 |
| M-879 | Encode the two named rules: **`interp` never depends on any `std-*`** + no upward-layer edges (covers the latent `interp ↔ cert` risk) | As the maintainer, I want the trusted-base boundary (KC-3) machine-checked, so that certified-mode work can't silently couple `interp` to the stdlib | Rules encoded as named entries in the check config; violation message cites the invariant doc (M-885); negative fixture proves each rule fires | Haiku | M-877 |
| M-880 | Wire the check into `just check` (+ CI parity, graceful skip when tools absent, never-silent skip message) | As an agent, I want the gate in the standard `just check` entrypoint, so that local↔CI parity holds without a separate command to remember | `just check` runs the check; skip path prints why (G2); documented in the justfile recipe list | Haiku | M-877, M-878, M-879 |
| M-881 | Break dev-cycle 1: `select →[dev] cert → vsa → select` — replace `select`'s dev-import of `cert` with local fixtures | As a contributor, I want `select`'s tests self-contained, so that the select/cert/vsa dev-cycle disappears without weakening any test | Cycle gone (check green); test coverage preserved (same assertions, fixture-driven per the house test-layout rule); tags keep their strength (VR-5); `cargo test -p mycelium-std-select` green | Sonnet | M-878 (detection first) |
| M-882 | Break dev-cycles 2+3: `cert →[dev] proj` and `cert →[dev] spore` — convert `cert`'s tests to fixtures (both cycles share the `cert →[dev] X → … → l1 → cert` shape) | As a contributor, I want `cert`'s tests fixture-driven, so that both remaining dev-cycles close in one cohesive change | Both cycles gone (check green); coverage preserved; zero dev-dep cycles at HEAD re-surveyed and recorded (`Empirical`); `cargo test -p mycelium-cert` green | Sonnet | M-878 |
| M-883 | Scaffold the **runtime-ABI seam crate** (working name `mycelium-rt-abi` — FLAG the name for maintainer confirmation) holding the surface `mlir` needs from `std-runtime` (precedent: `mycelium-sched`, PR #864; the seam is visible in mlir's own re-export comment) | As the AOT lane, I want the ABI surface in a lower stratum, so that `mlir` never reaches upward into the stdlib layer | New crate registered in the workspace (orchestrator-owned edit), `publish = false`, `0.0.0`; ABI items moved/re-exported with no behavior change; workspace builds green | Sonnet | — |
| M-884 | Repoint `mlir` (and `std-runtime`) at the ABI crate; **remove the `mlir → std-runtime` edge**; add the edge to the check's ban-set | As the architecture, I want the upward-layer anomaly gone and banned, so that it cannot return | `cargo metadata` shows no `mlir → std-runtime` edge; check bans it explicitly; `cargo test -p mycelium-mlir -p mycelium-std-runtime` green; AOT differential tests unchanged | Sonnet | M-883 |
| M-885 | **Invariant DN**: strata map, the two rules, where the check lives, and how a stratum assignment changes (by review, not by editing the ban-list in the violating PR) | As a future contributor, I want the invariant's rationale and change-procedure written down, so that the check is a policy, not a mystery | DN drafted (Draft status, append-only discipline); cited by the check's error output; indexed in `docs/Doc-Index.md` (integration-tier reconciliation) | Sonnet | M-877 (parallel OK; final text after M-880) |
| M-886 | **`publish = false` sweep** — all 52 crates (workspace-level inheritance where possible; 3/52 today), versions stay `0.0.0` (ADR-038 §2.2) | As the maintainer, I want the publication discipline mechanically true, so that no crate can be published before the Phase-I flip | `grep`-verifiable: every workspace member resolves `publish = false`; recorded (`Empirical`); workspace builds green | Haiku | — |
| M-866 (existing) | Bound the `mono.rs` `free_vars`/`pattern_binders` **unbounded recursion** (real recursion-safety bug — do early, independent of sequencing) | As a language user, I want deeply-nested programs to fail with an explicit bound error instead of a stack overflow, so that the compiler is never-silent under adversarial input | Explicit depth bound with `Result` error (no silent crash); **property test bounding the recursion**; `cargo test -p mycelium-l1` green; issue M-866 → done | Sonnet | — |

## Opening moves (turnkey)

1. **Workspace prep (DN-65):** `git fetch origin dev`; branch off the current `dev` tip in an
   isolated worktree; `just setup` (Rust; add `cargo-deny` if the ban-set mechanism is chosen).
2. **Mint the issues** from the PM table (verify free slots — mitigation #1), with the user
   stories + DoD above copied into each issue body (`doc_refs` per the grammar).
3. Run the serial check-lane (M-877→M-880) first or in parallel with M-866/M-886 (disjoint files);
   cycle fixes (M-881/M-882) and the ABI seam (M-883/M-884) fan out once detection (M-878) exists.
4. Land as scoped PRs to `dev` via `/pr-land`; shared files (workspace `Cargo.toml`, `justfile`,
   `deny.toml`, `CHANGELOG`, `Doc-Index`, `issues.yaml`) reconciled by the orchestrator/integrator.

## Definition of Done (kickoff)

- The structural gate **rejects regressions and passes clean state**: `just check` goes red iff a
  cycle (normal or dev), an upward-layer edge, or an `interp → std-*` edge is *introduced* — proven
  by negative fixtures (the M-877/M-879 DoDs), never by leaving HEAD broken — and is **green at
  HEAD** once M-881/M-882/M-884 land the fixes; diagnostics are per-edge and cite the invariant DN.
- Zero dev-dep cycles at HEAD (re-survey recorded, `Empirical`); `mlir` no longer depends on
  `std-runtime`; `publish = false` workspace-wide; M-866 fixed with a bound property test.
- Change-scoped tests green throughout; doc-maintenance per `_doc-maintenance.md` (CHANGELOG,
  issue statuses, `doc_refs` valid); every FLAG raised, none guessed (G2/VR-5).

## Prerequisites

None beyond a current `dev` tip. **ADR-038 ratification is NOT a prerequisite for H0** — the
acyclic invariant and hygiene sweep are sound under the prior plan too (roadmap §2 absorbed
Workstream A unchanged). The `mycelium-rt-abi` crate *name* is the one maintainer touchpoint (FLAG
at PR time; the extraction itself follows the PR #864 precedent).
