# E12 — Memory-Model Build Plan (DN-32 / RFC-0027 implementation)

**Status:** Living plan (2026-06-25). The design is decided end-to-end (RFC-0027 **Accepted**; DN-32
**Accepted**); this is the implementation roadmap, decomposed into **tightly-scoped Sonnet-swarm waves** with
the orchestrator (parent) owning shared files + integration. Usage-efficient: leaves are small + disjoint.

## Goal
Implement the **three-layer hybrid memory architecture** (DN-32) into the runtime, and clear the standing CI
blockers — systematically, never-silent (G2), honest tags (VR-5), small auditable kernel (KC-3).

## Current state (2026-06-25)
- Design: RFC-0027 Accepted (RC mechanism; the EXPLAIN record §9; OQ-1/OQ-4 resolved by DN-32; OQ-3 mitigated).
  DN-32 = affine-primary (L1) → optimized RC for explicit sharing (L2) → regions/batched scope reclamation (L3).
- Runtime tier: `crates/mycelium-std-runtime` (colony · scheduler · task · supervision · dataflow · network),
  `crates/mycelium-mlir/src/runtime.rs` (R1 v0 fork/join executor). Value model: `crates/mycelium-core` (Meta,
  content-addressing). **No reclamation / RC / region structures exist yet — greenfield.**
- Standing CI reds (pre-existing): `api` (stale baselines std-sys/l1/lsp/std-sys-host/spore + missing cli),
  `myc-fmt` (lib/std/{cmp,option}.myc not canonical), `deny` (network-403, environmental — unfixable here),
  markdown/spell/python-fmt (corpus-wide, low priority).

## Blocker track (parallel, mechanical — clears the green-CI blockers)
- **BLK-1 — api baselines:** regenerate the drifted committed baselines (std-sys, l1, lsp, std-sys-host, spore,
  + add cli) so the `api` gate is green. Pure regeneration; no code change.
- **BLK-2 — myc-fmt:** `mycfmt --write` the non-canonical `lib/std/*.myc` so the `myc-fmt` gate is green.
- *(BLK-3 markdown/spell/python-fmt — deferred, low value; `deny` is environmental.)*

## Memory-model waves (DN-32 Phase 1) — tightly-scoped, dependency-ordered
- **MEM-1 (foundation) — the reclamation EXPLAIN/audit record (RFC-0027 §9).** A `ReclamationRecord`
  `{ scope_id, sweep_epoch, trigger ∈ {RcZero, ScopeExit, ChannelClose}, value_meta_hash, channel_id? }`,
  emit-once + never-silent (G2), extending the RFC-0005 EXPLAIN contract. Self-contained data + contract +
  tests. *No dependency on the hard static analysis.* **First.**
- **MEM-2 — explicit RC + `rc==1` reuse probe (L2 core).** A refcount cell (non-atomic intra-hypha), the
  `rc→0`=free / `rc==1`=reuse / `rc>1`=share decision, emitting a MEM-1 record on reclaim. Depends on MEM-1.
- **MEM-3 — region/scope batched reclamation (L3).** Scope = region; batched reclaim at scope-exit over the
  RT7 scope tree; weak sibling coupling (OQ-1 resolved). Depends on MEM-1/2.
- **MEM-4 (hard, design-first) — Layer-1 static uniqueness analysis** (affine-primary / Perceus-style RC
  elision lowering) **+ the cross-hypha RC-vs-affine reconciliation sub-question** (DN-32 §7 / RFC-0027 §12).
  Design-heavy; sequenced through a **research-backed design note (DN-33)** before any implementation
  (KC-3 tension per DN-32 §6b). The runtime `RcCell` probe is the sound fallback, so MEM-4 is purely
  *additive* (it only elides provably-redundant RC ops — a bug downgrades to a missed optimization, never
  unsafety). Recommended decomposition (from the DN-33 research dossier): (1) non-escaping borrow elision
  [smallest, testable: refcount-invariant static check] → (2) `rc==1` reuse annotation → (3) full FIP static
  guarantee [Phase 3, deferred]. Cross-hypha: **Option A** (sole-move-only, affine channel = Pony-iso /
  Rust-`Box`; `RcCell` stays `!Send`) recommended for R1; **Option B** (shared-crosses-atomic-RC = `Arc`)
  deferred to R2 (`xloc`/`mesh`).

## Sequencing
- **Wave 1:** BLK (blocker removal) ∥ MEM-1 (reclamation record) — disjoint, Sonnet, octopus-merge. ✅ done.
- **Wave 2:** MEM-2 (RC core — `RcZero` live trigger). ✅ done.
- **Wave 3:** MEM-3 (regions — `ScopeExit` live trigger + canonical `ScopeNodeId`/`RegionEpoch`). ✅ done.
- **Wave 4:** ChannelClose live trigger + canonical `ChannelNodeId` (the last MEM-1 placeholder) ∥ live-executor
  scope/region wiring (`with_region`/`RegionScope`) + end-to-end L1/L2/L3 composition test. ✅ done — **Phase-1
  three-layer model feature-complete at the runtime tier** (all three triggers live; all ID placeholders
  canonicalized; scope-exit fires from a live scope).
- **DN-33 (MEM-4 design):** ✅ authored, then **ratified Draft → Accepted (2026-06-25, §8.1)** —
  additive lowering pass; cross-hypha **Option A** for R1 (`RcCell` stays `!Send`); **separate
  RC-annotated IR** (Core IR `node.rs` stays pristine); **differential + structural-invariant**
  soundness (`Empirical`). Design unblocked.
- **MEM-4 build (forward epic, per the §8.1 resolutions) — sequenced:**
  - **MEM-4·B0 — RC-emission pipeline foundation:** ✅ **done (2026-06-25).** New
    `crates/mycelium-mir-passes/` with the **RC-annotated IR** (`RcNode` — mirrors the first-order
    `Node` fragment + `Dup`/`Drop` + own/borrow `Mode`), the **naive (fully-owned) RC-emission**
    lowering `Node → RcNode` (shadowing-aware; `Fix`/`FixGroup` refused explicitly — G2), and the
    **structural balance invariant** (`1 + dups == uses + drops`, independently checked over the
    emitted IR — mutation-tested). `mycelium-core/src/node.rs` untouched (KC-3 / Q2). 21 tests green.
    *(The reference RC-evaluator / differential half of Q3 moves to Increment 1, where there are two
    emissions to compare.)*
  - **MEM-4·1 — Increment 1 (non-escaping borrow elision):** the escape analysis marking non-escaping
    uses *borrowed* (eliding their `dup`/`drop`), + the **Q3 differential harness** (emit with/without
    elision → identical results AND identical reclamation records). Subsumes `substrate` uniqueness
    (Q4). Behind the Q5 gate (measured `dup`/`drop`-reduction; count `Exact`, perf `Declared`).
  - **Later — Increment 2** (`rc==1` reuse annotation) **/ Increment 3** (full FIP static guarantee,
    Phase 3). The runtime `RcCell` probe stays the sound fallback throughout.

## Swarm discipline (per CLAUDE.md)
Sonnet leaves, disjoint dirs, `cargo fmt`/`clippy -D warnings -A unsafe_code`/`test -p <crate>` green, in-crate
`src/tests/` (M-797 as-touched), honest per-op tags, EXPLAIN-able / never-silent. The orchestrator owns shared
files (workspace manifest, CHANGELOG, issues.yaml, docs/api-index, this plan), runs the full `just check`, and
lands curated squashes. New runtime claims land **`Declared`** until measured (DN-32 §6a — perf is a goal).
