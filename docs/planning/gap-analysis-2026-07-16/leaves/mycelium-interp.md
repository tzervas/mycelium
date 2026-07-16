# Gap report — `mycelium-interp`

| Field | Value |
|---|---|
| **Crate** | `mycelium-interp` |
| **Scope group** | runtime |
| **Agent model** | grok-composer-2.5-fast |
| **Tip** | origin/dev @ cd71de69 |
| **Honesty** | `Empirical` unless cited otherwise |

## 1. Role in the system
Trusted L0 Core IR reference interpreter; ground truth for L1 differential and `myc run` (CURRENT-STATE).

## 2. Public surface snapshot
- Large `src/lib.rs` (1725 lines) documenting supported nodes/prims.
- Shared prim/swap engines with L1 `eval`.

## 3. Completeness vs Rust-implementation bar
| Gap | Severity | Evidence | Tracked? | Notes |
|---|---|---|---|---|
| Remaining L1-in-Core-IR nodes | med | `lib.rs:63` | L1/interp | Residual retirement |
| Perf vs AOT | low | ADR-022 T6 done | — | Not correctness gate |

## 4. Transpile-to-Mycelium readiness
| Gap | Class | Severity | Evidence | Tracked? | Native answer |
|---|---|---|---|---|---|
| Transpiler does not replace interpreter | Other | low | M-991/DN-34 | — | Rust trusted base |
| Host IO primitives for ported std | Import | med | std-io bridge | M-514 | When `io.myc` exists |

## 5. Tests / witnesses
Crate tests; cross-crate differential via `mycelium-l1`.

## 6. Delta vs prior assessment
Interpreter role unchanged. Transpile path validates via elaboration semantics, not raw Rust→`.myc` execution.

## 7. Recommended next actions (ranked)
1. Add Core IR nodes only with three-way witness.
2. Stabilize API during ADR-045 window.

## 8. FLAGs
None.