# Program — transpile readiness → archive → component repos → `*-myc` → re-export

| Field | Value |
|---|---|
| **Status** | **Active implement program** (L0 program) |
| **Honesty** | Gates `Declared` until measured; pilot numbers `Empirical` |
| **Framework** | `maint-guide.md` · L0 = hard planning · agents = `grok-composer-2.5-fast` |
| **Base** | monorepo `tzervas/mycelium` trunks `dev`/`integration`/`main` |
| **Ground** | DN-88 · DN-27 · DN-34 · ADR-022 · ADR-036/038 · ADR-042 · ADR-045 |
| **Prior implement** | `PROGRAM-HANDOFF-ONESHOT.md` (history — subsumed for spawn briefs) |
| **Design pack** | DESIGN-01…04 stay **design review** (Draft); do not block Phase G residual close |

## End-state (maintainer directive)

1. **Close transpilation gaps** enough for hands-off whole-repo / whole-component transpile
   (gap-profiler honesty: no fabrication).
2. **Archive `main` at pre-transpilation state** (recoverable snapshot — never lose Rust monorepo
   tip).
3. **Decompose** monorepo into **component repos** (DN-88 / ADR-022 component + phylum re-export
   horizon).
4. **Transpile each component** into a paired **`*-myc` component repo** (Mycelium sources + dual
   witness path).
5. **Umbrella re-export repo** that composes them into a clean, usable **self-hosted Mycelium**
   presentation.

```mermaid
flowchart LR
  G[Gap-close monorepo] --> A[Archive main pre-xpile]
  A --> D[Decompose component repos]
  D --> T["Transpile → *-myc repos"]
  T --> R[Re-export umbrella repo]
```

**Phase order is strict:** **G → A → D → T → R**. Do not open D until A is verified; do not open T
until D has a component map; do not claim self-host presentation until R’s CI pins component SHAs.

## Phase G — Gap-close (NOW)

### Gate (honest DoD — not “100% lines green”)

| # | Criterion |
|---|---|
| G1 | Default pilot set + std-fs/io: **file Clean** or residual **EXPLAIN + ranked**, no unknown first-poison classes we can close without design gates |
| G2 | M-1006 ladder: ≥ one full Empirical remeasure table on tip after residual waves |
| G3 | Top residual classes closed or **design-gated** (M-875 expand-first stays design until Accepted) |
| G4 | Tip-bound remote CI green (or host-witness + API bind) on `dev`/`integration` |
| G5 | No VR-5 lie: never claim one-shot ready with fabricated prims |

### Residual waves (serial on transpile where needed)

| Wave | Focus | Notes |
|---|---|---|
| **G-α** | `Result` / residual after Source-Sink; Import non-type tails | Fractal L2 leaves |
| **G-β** | Next first-poison classes from remeasure | Rank heat |
| **G-γ** | Broader M-1006 target set | Measure + close |
| **G-δ** | Design-gated only (M-875, host io effects) | No implement until Accepted |

## Phase A — Archive main (after G gate)

| Step | Action |
|---|---|
| A1 | Tag / branch: e.g. `archive/main-pre-component-transpile-YYYY-MM-DD` from `origin/main` tip |
| A2 | Push archive ref (never force-delete later without maintainer) |
| A3 | Document SHA in this program file + CHANGELOG |

**Does not** rewrite `main` history. Snapshot only.

## Phase D — Decompose component repos

Ground: DN-88 · DN-27 · ADR-022 · ADR-036/038 dogfooding · ADR-045 window.

| Step | Action |
|---|---|
| D1 | Component map: crates/phyla ownership → repo names (`mycelium-core`, `mycelium-std-io`, …) |
| D2 | Extract history-preserving or clean-slice (maintainer: prefer `git subtree` / filter-repo plan) |
| D3 | Create `tzervas/<component>` repos; CI minimal; MIT |
| D4 | Monorepo becomes thin or freeze pointer until re-export |

**Autonomy:** L0 plans map; L1/L2 extract; no delete of monorepo content until archive (A) verified.

## Phase T — Transpile to `*-myc`

| Step | Action |
|---|---|
| T1 | For each component: `transpile-vet` → hand-close residuals → land `.myc` under `lib/` / component layout |
| T2 | Dual witness: Rust differential + `myc check` where applicable |
| T3 | Repo name convention: `<component>-myc` or `mycelium-<x>-myc` (decide in D1 map) |
| T4 | Emissions stay Empirical/Declared until differential upgrades |

## Phase R — Re-export umbrella

| Step | Action |
|---|---|
| R1 | New repo e.g. `mycelium` or `mycelium-lang` presenting phyla surfaces |
| R2 | Manifest / re-exports of component `*-myc` deps (spore or git deps as ratified) |
| R3 | README + docs path for “use Mycelium self-hosted” |
| R4 | CI: check umbrella builds against pinned component SHAs |

## Honesty / non-goals

- **Not** claiming “all gaps closed” until G1–G5 green.
- **Not** force-pushing `main`.
- **Not** deleting monorepo history.
- App-level log offload remains **out of language** (DESIGN-04).
- Design pack 01–04 remains **Draft design review** — mechanisms stay `Declared` until ratify +
  differential; Phase G residual close does not wait on pack acceptance except G-δ design gates.

## Status log

| When | Note |
|---|---|
| 2026-07-17 | Program opened; Phase G active; README points here as active implement program |
| 2026-07-17 | **D1 draft map** (prep only — Phase D still gated on G→A): [`COMPONENT-REPO-MAP-DRAFT.md`](./COMPONENT-REPO-MAP-DRAFT.md). Declared operational map; does not claim DN-88 §3 met; no repos created |
