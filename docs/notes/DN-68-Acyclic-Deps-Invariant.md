# Design Note DN-68 — The Acyclic-Deps Invariant (M-885)

| Field | Value |
|---|---|
| **Note** | DN-68 |
| **Status** | **Draft** (2026-07-02, authored by the M-885 leaf) |
| **Decides** | Nothing new — this note **documents** the structural dependency invariant already encoded by `xtask/src/deps/` + `xtask/deps-strata.toml` (M-877/M-878/M-879), so the gate is a written policy with a stated rationale and change-procedure, not a mystery a contributor has to reverse-engineer from a violation message. |
| **Feeds** | `xtask/deps-strata.toml` `[meta].basis_ref` and both `[[named_rules]]` `basis_ref` fields (currently placeholder text — "pending authorship" / "pending" — that the orchestrator updates to point here at integration, per this leaf's FLAG below); `cargo run -p xtask -- deps`'s violation output, which cites this invariant. |
| **Depends on** | `docs/adr/ADR-038-Pragmatic-Dogfooding-Function-First-Release-Strategy.md` §2.1 (the North Star: "Rust where appropriate, Mycelium everywhere else" — the trusted base, KC-3, is one of the reasons Rust stays); `docs/planning/road-to-1.0.0-and-mycelium-rewrite.md` §2 (H0 — Foundations: acyclic-deps hardening, the roadmap section this whole M-877…M-885 wave implements); PR #864 (`refactor(sched): relocate work-stealing Scheduler to mycelium-sched`) — the precedent for how an upward-layer anomaly is fixed by extraction, not by rewriting the rule; `docs/Mycelium_Project_Foundation.md` KC-3 (small auditable kernel). |
| **Date** | 2026-07-02 |
| **Task** | M-885, branch `acy/m885-invariant-dn` |

> **Posture (transparency rule / VR-5).** This note describes machinery that already exists and is
> already exercised by tests (`xtask/src/tests/deps.rs`); it ratifies nothing new and changes no
> code. Every claim below is tagged at the strength the source material actually supports: `Exact`
> where the checker performs a real, deterministic computation (Tarjan SCC, a `>` comparison over a
> frozen map); `Empirical` where a value was derived once from a point-in-time measurement (the
> `[strata]` map); `Declared` where a value is an asserted architectural judgment call, not derived
> from the graph (the `[tiers]` map). Nothing here is upgraded past what `xtask/src/deps/` and
> `xtask/deps-strata.toml` actually do — this note describes the code, it does not invent policy the
> code doesn't already enforce.

---

## 1. The invariant

The Mycelium workspace's 52 `mycelium-*` crates plus `xtask` (53 crates total) must form a
**strictly-downward acyclic dependency graph**: no dependency edge — **normal or dev** — may point
from a crate to another crate at the same or a higher layer, under either of the two views defined
below. Two specific consequences of this general shape are asserted as named, checked rules
(§3): (a) **`mycelium-interp` must never depend, in any dependency kind, on any `mycelium-std-*`
crate** — this is the KC-3 trusted-base boundary (`docs/Mycelium_Project_Foundation.md` KC-3;
ADR-038 §2.1's North Star names the trusted base as one of the reasons Rust — and a small, auditable
dependency shape — stays load-bearing); and (b) **no crate may depend on a crate in a strictly higher
architectural tier** (`core` < `std` < `tools`).

This is a *structural* invariant, not merely a stylistic preference: `cargo` itself rejects a
`[dependencies]` cycle (it cannot resolve one), but **`cargo` never rejects a `[dev-dependencies]`
cycle** — a dev-dep cycle compiles and tests run fine, so nothing in the ordinary toolchain would
ever surface one. Before this wave, three such cycles existed, centered on `mycelium-cert`
(`select →[dev] cert → vsa → select`; `cert →[dev] proj → l1 → cert`; `cert →[dev] spore → proj → l1
→ cert` — `docs/planning/road-to-1.0.0-and-mycelium-rewrite.md` §2, `Empirical`, cargo-metadata dig
2026-07-01), plus an upward-tier normal-dep anomaly (`mycelium-mlir → mycelium-std-runtime`, `core`
tier depending on `std` tier). The gate this note documents exists specifically to make both classes
of defect — silent dev-cycles and silent tier-order violations — loud instead (G2: never-silent).

## 2. Two independent views over one graph

`xtask/deps-strata.toml` encodes the same 53-crate normal-dependency graph two ways, at two
different guarantee strengths (VR-5: each tagged honestly, neither upgraded past its basis):

### 2.1 `[strata]` — the fine per-crate layer (`Empirical`)

One integer per crate: the **longest normal-only dependency chain from a sink** (a leaf crate with
no in-workspace normal deps is stratum 0; every other crate's stratum is `1 + max(stratum(dep) for
dep in its normal deps)`). It was derived **once**, inclusively, from `cargo metadata
--format-version 1` at `dev@388a865` (2026-07-02 per the file's `[meta]`), and is compiled into the
checker as a **frozen snapshot** (`include_str!`), not re-derived on every run.

Because the numbers were derived inclusively over the graph as it stood at derivation time, this map
is tautologically consistent with that snapshot's normal-dep graph — checking it against itself
reports zero violations. Its value is forward-looking: it is a **regression gate**. A normal edge
added *later* that violates the frozen ordering (a low-stratum crate gaining a dependency on a
high-stratum one) is caught, even though the map cannot retroactively see an architecture problem
that already existed at derivation time — a limitation the coarse `[tiers]` view (§2.2) exists
precisely to cover. `mycelium-core` sits at stratum 0 with the most dependents (41, per the
roadmap's `Empirical` dig); `mycelium-interp` sits at stratum 2.

### 2.2 `[tiers]` — the coarse architectural bucket (`Declared`)

Every crate is also assigned to exactly one of three ordered buckets — `core < std < tools`
(`tier_order` in the TOML) — independent of its numeric stratum. This is a **manually-asserted**
judgment about intended architecture, not a value derived from the graph: `core` is the language
kernel + runtime substrate (must stay independent of the standard library — the KC-3 trusted-base
boundary); `std` is the `mycelium-std-*` library layer built on top of `core`; `tools` is
user-facing CLI/tooling/automation built on top of both.

This is exactly why `[tiers]` catches what `[strata]` structurally cannot: at HEAD (documentation
time, before H0-3's fix lands), `mycelium-mlir` (tier `core`) has a normal dependency on
`mycelium-std-runtime` (tier `std`) — an upward-tier anomaly that `[strata]`'s point-in-time
derivation silently absorbed (mlir's own stratum simply grew large enough to accommodate the edge),
but which the `no-upward-tier-edges` rule (§3) flags, because a crate's tier bucket is a fixed
architectural assertion that does not move just because a bad edge exists.

**Never-silent (G2):** a crate absent from either map is not skipped — it is itself reported as a
violation ("crate not in stratum/tier map"), by both `check_normal_downward` and `check_tier_order`
(`xtask/src/deps/checks.rs`).

## 3. The two named rules

Beyond the two structural checks above (normal-edge downward-ordering per `[strata]`, and
combined-graph acyclicity per §4), `xtask/deps-strata.toml`'s `[[named_rules]]` table encodes two
specific, inspectable cross-boundary rules (never a hard-coded magic string in the Rust source — G2;
adding a *third* rule of a kind the checker already understands means editing this TOML, not
`checks.rs`'s control flow):

- **`no-interp-std-dep`** (`kind = "forbidden-target-prefix"`, `source = "mycelium-interp"`,
  `forbidden_target_prefix = "mycelium-std-"`). Forbids **any** dependency edge — normal, dev, *or*
  build — from `mycelium-interp` to a crate whose name starts with `mycelium-std-`. This is the
  KC-3 trusted-base boundary made mechanical: the interpreter is the small, auditable kernel; it
  must never pull in the standard library, in any capacity, including tests. It also forecloses a
  latent risk the roadmap names explicitly: an `interp ↔ cert` cycle, should certified-mode
  execution logic ever land partly inside `interp` while `mycelium-cert` still reaches back through
  the std layer.

- **`no-upward-tier-edges`** (`kind = "tier-order"`). Forbids any normal, dev, or build edge from a
  crate in an earlier `tier_order` bucket UP to a crate in a later one (i.e. `core → std`, `core →
  tools`, or `std → tools` are violations; `tools → std`, `tools → core`, or `std → core` — the
  allowed downward direction — are fine).
  The known HEAD violation this rule reports (`mycelium-mlir → mycelium-std-runtime`) is
  **expected**, not a bug in the check — it is fixed by extracting the runtime-ABI surface `mlir`
  needs into a lower crate, the same shape as the precedent below, not by loosening this rule.

## 4. Where the check lives, and how it runs

The gate is `cargo run -p xtask -- deps` (`xtask/src/deps/mod.rs::run`), implemented as a `cargo
metadata --format-version 1` analysis rather than `cargo-deny`, so it stays self-contained with no
runtime dependency beyond `cargo` itself. It:

1. Builds an in-workspace `Graph` (`xtask/src/deps/graph.rs`) from the resolver's own graph —
   `Exact`, a lossless reduction, not a heuristic — dropping every edge that touches a crate outside
   the workspace (an external crate cannot participate in an intra-workspace layering violation or
   cycle).
2. Runs four checks (`xtask/src/deps/checks.rs::run_all`), in order:
   - `check_normal_downward` — every **normal** edge's source stratum must be strictly greater than
     its target stratum, per the frozen `[strata]` map (`Exact` comparison over an `Empirical` map).
   - `check_acyclic_including_dev` — **Tarjan's strongly-connected-components** algorithm over the
     combined normal+dev+build graph. This is deliberately an SCC decomposition, not a single
     DFS-back-edge pass: a back-edge pass only reports the cycles a particular traversal order
     happens to hit and can under-report, where Tarjan's SCC finds every crate that participates in
     *any* cycle, deterministically and completely (`Exact`). A component with more than one member
     is a cycle; members that belong to several historically-named cycles can collapse into one
     reported component when they share crates — every contributing edge is still listed, so a
     reader can see each path, but the report is the true, complete answer, not a bug.
   - `check_named_rules` — evaluates every `[[named_rules]]` entry (§3) by its declared `kind`; an
     unrecognized `kind` is itself reported as a violation, never silently skipped.
3. Prints every violation with its rule id, the offending edge (with dependency kind), and the
   rule's `basis_ref` — **never** a bare pass/fail exit code (G2) — then exits non-zero iff any
   violation fired.

**Not yet wired into `just check`** at the time of this note — that is tracked separately as M-880
(the justfile is orchestrator-owned; the `deps` subcommand itself, and this documentation, are
independent of that wiring). Stated accurately: the subcommand exists and is runnable directly
today; its inclusion in the default check loop is pending, not yet landed.

## 5. The change-procedure — why this stays a policy, not a rubber stamp

**A stratum or tier assignment changes only by review** — a dedicated PR that edits
`xtask/deps-strata.toml` with a stated rationale for the change, reviewed on its own terms. It is
**never** acceptable to edit the ban-list (the `[strata]`/`[tiers]` maps, or a `[[named_rules]]`
entry) inside the *same* PR that introduces the edge the current rules would reject. That pattern
turns a structural gate into a rubber stamp: the check would pass not because the dependency graph
stayed correct, but because the check's own definition of "correct" was widened in the same breath
that broke it.

The correct move when a genuine upward or cross-boundary dependency is needed is **extraction**, not
loosening: pull the shared surface into a new (or existing) lower-tier crate, so the dependency
becomes downward again. **PR #864** (`refactor(sched): relocate work-stealing Scheduler to
mycelium-sched`) is the concrete precedent — the `mycelium-std-runtime → mycelium-interp` cycle was
resolved not by permitting the cycle, but by extracting the scheduler into a new `mycelium-sched`
crate depending only on `mycelium-core`, so both `mycelium-interp` and `mycelium-std-runtime` could
depend on it downward instead. `mycelium-std-runtime` kept a re-export at the original path so
existing consumers compiled unchanged. The `mlir → std-runtime` fix tracked as roadmap item H0-3
(`docs/planning/road-to-1.0.0-and-mycelium-rewrite.md` §2) is the same shape: extract the
runtime-ABI surface `mlir` actually needs into a lower crate, not add an exception to
`no-upward-tier-edges`.

A **re-derivation** of the `[strata]` map (as opposed to a single crate's assignment) is a larger
event — it means re-running the derivation over the current graph and replacing the frozen snapshot
wholesale — and should be treated as its own reviewed change with a fresh `derived_from` timestamp
in `[meta]`, not folded into an unrelated PR.

## 6. Open questions (flagged, not guessed)

- **`[strata]` staleness.** Because `[strata]` is a frozen, point-in-time snapshot rather than
  re-derived live, it can only regress-test against edges added *after* derivation; it says nothing
  about whether the derivation itself should be periodically refreshed as the graph evolves (e.g.
  after several extraction-style fixes change the true longest-chain depths). No re-derivation
  cadence is defined here — left to the maintainer.
- **Citation wiring.** At authorship time, `xtask/deps-strata.toml`'s `[meta].basis_ref` and both
  `[[named_rules]]` `basis_ref` fields still read "DN: acyclic-deps invariant (M-885, pending
  authorship / pending)" rather than pointing at this file. Updating those three strings is a change
  to `xtask/deps-strata.toml`, which this leaf was scoped to leave untouched (a sibling leaf owns
  that file concurrently) — flagged for the integrating parent (§ Report).
- **`just check` wiring timing.** This note describes the checker as it exists; it does not assert
  M-880 has landed. Whether `just check` gates on `deps` by the time this note is indexed is a fact
  the integrating parent should verify before citing this note as describing a *fully wired* gate.

## Changelog

- 2026-07-02 — Draft authored (M-885 leaf, `acy/m885-invariant-dn`). Documents the existing
  `xtask/src/deps/` + `xtask/deps-strata.toml` machinery (M-877/M-878/M-879); ratifies nothing new.
