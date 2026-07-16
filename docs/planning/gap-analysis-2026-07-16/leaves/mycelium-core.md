# Gap report — `mycelium-core`

| Field | Value |
|---|---|
| **Crate** | `mycelium-core` |
| **Scope group** | kernel |
| **Agent model** | grok-composer-2.5-fast |
| **Tip** | origin/dev @ cd71de69 |
| **Honesty** | `Empirical` unless cited otherwise |

## 1. Role in the system
Core IR, `Value`/`Repr`/`Meta`, guarantee lattice, content identity. ADR-022 T1 criteria gate-met; tag M-703 maintainer-reserved (CURRENT-STATE).

## 2. Public surface snapshot
- Entry `src/lib.rs` (~114 lines); 17 module files.
- No `TODO`/`Residual` hits in `src/` (repo grep 2026-07-16).

## 3. Completeness vs Rust-implementation bar
| Gap | Severity | Evidence | Tracked? | Notes |
|---|---|---|---|---|
| T1 1.0.0 tag not cut | med | CURRENT-STATE T1 | M-703/M-655 | Release act |
| ADR-045 unfreeze window | med | ADR-045; DN-99 | enb wave | Bounded diffs OK |
| L1 nodes for full Residual retirement | low | cross-ref interp `lib.rs:63` | L1/interp | Joint |

## 4. Transpile-to-Mycelium readiness
| Gap | Class | Severity | Evidence | Tracked? | Native answer |
|---|---|---|---|---|---|
| Kernel types in transpiler `map_type` | Import | high | `transpile/src/map.rs` | M-874 | Union bucket |
| No near-term kernel `.myc` port | Other | low | DN-34 | M-740+ | Rust stays trusted |

## 5. Tests / witnesses
Property tests on value/repr/guarantee; `cargo-public-api` baselines.

## 6. Delta vs prior assessment
Kernel freeze (DN-56) superseded by **ADR-045** for gap closure. T1 implementation bar largely met; transpile blocked downstream in L1/stdlib emission acceptance.

## 7. Recommended next actions (ranked)
1. Keep kernel diffs scoped to DN-39/DN-99 during unfreeze.
2. Regenerate `docs/api-index/` on public API changes.
3. Do not gate transpile on kernel port.

## 8. FLAGs
None.
