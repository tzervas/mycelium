# Gap report — `mycelium-dense`

| Field | Value |
|---|---|
| **Crate** | `mycelium-dense` |
| **Scope group** | kernel |
| **Agent model** | grok-composer-2.5-fast |
| **Tip** | origin/dev @ 708bbc14 |
| **Honesty** | `Empirical` unless cited otherwise |

## 1. Role in the system
Dense paradigm ops with per-op guarantee tags (M-230; F32/BF16 scope). Part of Wave G2 PARTITION (kernel group). Rust reference surface is primary; transpile impact is mostly indirect unless noted.

## 2. Public surface snapshot
- Crate root `crates/mycelium-dense/src/` (~1008 LOC Rust, repo count 2026-07-16).
- Key entry: `src/lib.rs` module graph and re-exports.
- Guarantee tags: per module docs (`Exact`/`Proven`/`Empirical`/`Declared`) where operations emit `Meta`.

## 3. Completeness vs Rust-implementation bar
| Gap | Severity (block/high/med/low) | Evidence (file:line / test) | Tracked? (M-id / none) | Notes |
|---|---|---|---|---|
| Residual domain work | med | `no TODO/FIXME in src grep (mycelium-dense)` | DN-99 / enb | F16/F64 dense + approximate-source composition (M-204) |
| Release / spec hygiene | low | G1 SYNTHESIS | M-703 area | Not transpile-blocking |

## 4. Transpile-to-Mycelium readiness
How this crate affects / is affected by `mycelium-transpile` + `lib/*.myc` port:
| Gap | Class (if transpile: Import/Derive/Impl/Other/…) | Severity | Evidence | Tracked? | Native answer (DN-111) if known |
|---|---|---|---|---|---|
| Near-term `.myc` port | Other | low | DN-34; M-991 profiler not porter | M-740 | Rust trusted base |
| Transpile dependency | Import | med | Indirect via std/lib only | M-1084 | N/A for this crate directly |

## 5. Tests / witnesses
- Crate tests under `src/tests/` or `tests/` per house layout; run via `cargo test -p mycelium-dense`.
- Property/differential where documented in crate `lib.rs` (e.g. numerics properties, mir-passes RC differential).

## 6. Delta vs prior assessment
| Prior | Now on tip |
|---|---|
| `language-completeness-gap-inventory.md` | Transpile path still early; this crate's Rust bar largely met for kernel |
| G1 SYNTHESIS (2026-07-16) | G2 extends PARTITION; no contradiction — defers bulk port to M-740/M-993 |

## 7. Recommended next actions (ranked)
1. Keep changes scoped under ADR-045 / DN-99 when touching public API.
2. Maintain acyclic deps (DN-68); no upward tier edges.
3. Do not treat this crate as bulk transpile pilot until std-cmp vet moves off 0% checked.

## 8. FLAGs
None (orch integrates shared-file items at dev→integration).
