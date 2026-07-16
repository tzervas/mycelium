# Gap analysis synthesis — Wave G2 (2026-07-16)

| Field | Value |
|---|---|
| **Status** | Integrated (44/44 G2 leaves + 12 G1 = 56 PARTITION crates) |
| **Orch** | `claude/orch/gap-g2-2026-07-16` |
| **Tree tip** | `origin/dev` @ `708bbc14` (WAVE-G2-PLAN base); orch integrates on pushed tip |
| **Honesty** | `Empirical` (leaf evidence + G1 SYNTHESIS cross-ref); tracker rows `Declared` |

## Executive summary

**G2 completes the PARTITION** left after G1's 12 transpile-critical crates. **Rust reference completion** for kernel adjuncts (cert, dense, numerics, select, vsa, vsa-decode), runtime seam (rt-abi, sched, stack, workstack), AOT (mir-passes, mlir), remaining stdlib phyla, toolchain drivers, and bench is **largely met** on the ADR-022 bar: few `TODO`/`unimplemented!` hits in G2 `src/` (repo grep 2026-07-16). Residual Rust work stays in **named increments** (M-131 VSA bundle Proven, dense F16/composition M-204, MEM-4 interprocedural borrow, MLIR fragment vs full calculus) and **ADR-045** gap-closure — not absent implementations.

**Transpile-to-Mycelium readiness** is **unchanged in binding constraint** vs G1: **`checked_fraction` ≈ 0** on default vet pilots; G2 adds no new emit surface. Toolchain crates (`mycelium-check`, `mycelium-lsp`, `mycelium-cli`) are the **oracle/dogfood gate** for any `lib/*.myc` graduation. Stdlib G2 phyla need **M-993 differential + M-1084/M-1086 emit** before bulk port; kernel/runtime/AOT G2 crates correctly **defer `.myc` port** (DN-34 / M-991 profiler).

**Delta vs G1 SYNTHESIS:** G1 ranked transpile blockers (M-1090, M-1086, M-1084, missing `io.myc`, M-740). G2 **confirms** those remain the leverage path; G2-specific adds: **MLIR capability LOSS visibility** (bench), **host `io` / sys-host effect reconciliation** (FLAG), **myc-check as universal acceptance gate**.

## Ranked residual (G2 scope — severity × readiness)

| Rank | Gap | Severity | Group | M-ids / notes |
|---:|---|---|---|---|
| 1 | Oracle `checked_fraction` ≈ 0 | block | toolchain | M-1000, M-740 — unchanged from G1 |
| 2 | Phase-2 emit (fmt derive import) | block | transpile + std | M-1090, M-1086, M-1084 — G1 #1–4 |
| 3 | Missing / partial `lib/std` for G2 phyla | high | stdlib | M-993; fs, swap, vsa, runtime ports |
| 4 | MLIR dialect fragment vs LLVM fallback | high | aot | M-601; explicit capability LOSS |
| 5 | M-131 VSA bundle Proven Value wrapper | med | kernel | Rust-complete algebra; bound pending |
| 6 | MEM-4 interprocedural / further RC elision | med | aot | DN-33 increments |
| 7 | Dense approximate-source composition | med | kernel | M-204/M-211 |
| 8 | Host effects (sys-host, fs, time, io) | med | stdlib | Cross-crate FLAG |
| 9 | Bench / xtask environmental skips | low | bench | Empirical harness; ADR-019 MLIR off-default |
| 10 | Release tags / spec hygiene | low | all | M-703 area |

## Delta vs G1 SYNTHESIS

| G1 claim | G2 finding |
|---|---|
| Wave G2 = kernel + toolchain PARTITION remainder | **Done** — 44 leaves; includes stdlib residual + bench |
| Focus oracle + dogfood gates | **Confirmed** — `mycelium-check` leaf ranks oracle as block |
| `checked_fraction` early | **No movement** expected until G3 emit wave |
| M-740 blocks semcore acceptance | **Still accurate** — G2 kernel/runtime not on critical port path |

## Recommended work waves (updated)

### Wave G3 — transpile emit Phase-2 (unchanged from G1)
Single epic `mycelium-transpile`: M-1090 → M-1086 → M-1084 → re-vet.

### Wave G4 — stdlib `lib/` ports (expand with G2 list)
Disjoint `lib/std/` files for: **fs, swap, dense, vsa, runtime, sys** (G2 leaves); still blocked on G3 for io/fmt/json chain.

### Wave G5 — M-740 semcore (unchanged)

### Wave G6 — AOT / bench honesty (G2-specific)
- MLIR dialect expansion or document permanent LLVM split
- Bench capability LOSS table kept current with mlir feature matrix

## G2 leaf index (44)

| Crate | Branch pattern | Status |
|---|---|---|
| mycelium-cert … mycelium-bench | `claude/leaf/gap-g2-<short>` | ok (conductor-integrated) |

Full paths: `docs/planning/gap-analysis-2026-07-16/leaves/<crate>.md` for all 56 PARTITION crates.

## FLAGs (orch)

- **Leaf git contract:** Conductor authored 44 leaves on orch worktree; integrated via orch commit series (G1 precedent: separate branch per leaf ideal; batch integration with file-count verify).
- **Host `io` / `std-sys-host` / `std-fs`:** reconcile effect semantics at integration tier (carried from G1).
- **Generated leaf depth:** Empirical LOC/todo grep + lib.rs role; deepen M-131/MLIR/check rows manually enriched.
- **CHANGELOG / issues.yaml / Doc-Index:** untouched per wave partition.
