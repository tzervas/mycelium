# Kickoff `prm` — Runtime Prim Wiring: `reclaim:supervised` + `fuse_join:data` (M-817, closes M-710)

> Stowed kickoff, UID **`prm`**. Read `CLAUDE.md`, `.claude/kickoffs/README.md`, and
> **`.claude/kickoffs/_doc-maintenance.md`** (anti-drift) first.

## Metadata
| Field | Value |
|---|---|
| **UID** | prm |
| **Head/working branch** | `claude/head/prm-runtime-prims` (off `dev`) |
| **Status** | ready (the r4v residual — surface + elaboration landed; runtime prims unwired) |
| **Swarm mode** | serial (runtime registry + std-runtime dispatch) |
| **Depends on** | DN-58 (Accepted), r4v (M-667 landed — fuse/reclaim/@tier surface + elaboration), `mycelium-std-runtime` supervision (M-713, landed) |

## Scope
Close the **r4v execution residual** so M-710 goes from partial → done (and DN-58 can move toward
Enacted). r4v's elaborator lowers `fuse(a,b)` on **data** types to `Op{prim:"fuse_join:data"}` and
`reclaim(policy){body}` to `Op{prim:"reclaim:supervised"}`, but neither prim is registered in the runtime,
so they currently **refuse never-silently** (honest residual). This wave **wires the two prims**:
- **`fuse_join:data`** — resolve the value's user `Fuse` instance and dispatch to its `join` fn (the
  repr-type fuses — `fuse_join:binary/ternary/dense/bytes/seq` — already execute via the `bit.and`-style
  semilattice-meet path; this is the user-`Data` case only).
- **`reclaim:supervised`** — dispatch to `mycelium-std-runtime::supervision` (`run_supervised` /
  `supervise_with_restart`), threading a `SupervisionRecord` for the EXPLAIN trail (RFC-0027 §9 fields);
  never-silent on reclamation/restart (G2).

## Grounding (doc_refs)
- `corpus:DN-58#§A` (fuse provenance/meet) · `corpus:DN-58#§B` (reclaim → supervision dispatch) ·
  `corpus:RFC-0008` RT6/RT7 · `corpus:RFC-0027#§9` (the EXPLAIN/SupervisionRecord schema).
- `src:crates/mycelium-l1/src/elab.rs` (the `fuse_join:*` / `reclaim:supervised` prim-emission sites) ·
  the `PrimRegistry::with_builtins()` registration point · `src:crates/mycelium-std-runtime/src/supervision.rs`
  (`run_supervised`, `supervise_with_restart`, `SupervisionRecord`, `CancelTree`).

## Approach (serial, inline)
Register `fuse_join:data` and `reclaim:supervised` in `PrimRegistry::with_builtins()` (or the appropriate
runtime registry). `fuse_join:data`: look up the operand type's `Fuse` instance, call its `join`, compose
`Meta` by `meet` + a `Derived{op:"fuse_join"}` provenance node (RFC-0027 §10.6). `reclaim:supervised`:
construct the supervision policy from the `policy` arg, run `body` under `run_supervised`/
`supervise_with_restart`, record the `SupervisionRecord`. Add a **three-way differential** for a data-type
`fuse` and for a supervised `reclaim` scope (L1-eval ≡ L0-interp ≡ AOT) — these earn `Empirical`.

## Definition of Done
- [ ] `fuse` on a user `Fuse`-instance data type **runs three-way** (Empirical); `reclaim(policy){…}`
  **dispatches to supervision** with the EXPLAIN record (never-silent restart/reclaim, G2). **No new L0
  node (KC-3)** — both are runtime prims over existing nodes.
- [ ] `just check` green; honest tags (Empirical post-differential; fuse semilattice laws still
  `Empirical`-via-property-test unless a mechanized basis lands — VR-5).
- [ ] **Doc maintenance (anti-drift):** `issues.yaml` **M-817 → done, M-710 → done** (execution end-to-end
  complete); **DN-58 → propose `Enacted`** (surface + execution now landed) — step through, maintainer nod;
  RFC-0008 §4.6 R1 note "executes"; `CHANGELOG.md` entry; `docs/api-index/` if API changed.

## Landing
`/wave-land` → `main` after green + `/pr-review` self-review + curated squash; backprop.
