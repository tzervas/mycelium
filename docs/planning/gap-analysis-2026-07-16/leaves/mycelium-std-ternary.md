# Gap report — `mycelium-std-ternary`

| Field | Value |
|---|---|
| **Crate** | `mycelium-std-ternary` |
| **Scope group** | stdlib |
| **Agent model** | grok-composer-2.5-fast |
| **Tip** | origin/dev @ 708bbc14 |
| **Honesty** | `Empirical` unless cited otherwise |

## 1. Role in the system
Ternary paradigm std surface. Part of Wave G2 PARTITION (stdlib group). Rust reference surface is primary; transpile impact is mostly indirect unless noted.

## 2. Public surface snapshot
- Crate root `crates/mycelium-std-ternary/src/` (~1933 LOC Rust, repo count 2026-07-16).
- Key entry: `src/lib.rs` module graph and re-exports.
- Guarantee tags: per module docs (`Exact`/`Proven`/`Empirical`/`Declared`) where operations emit `Meta`.

## 3. Completeness vs Rust-implementation bar
| Gap | Severity (block/high/med/low) | Evidence (file:line / test) | Tracked? (M-id / none) | Notes |
|---|---|---|---|---|
| Residual domain work | high | `no TODO/FIXME in src grep (mycelium-std-ternary)` | DN-99 / enb | Spec ratification vs lib/*.myc graduation |
| Release / spec hygiene | low | G1 SYNTHESIS | M-703 area | Not transpile-blocking |

## 4. Transpile-to-Mycelium readiness
How this crate affects / is affected by `mycelium-transpile` + `lib/*.myc` port:
| Gap | Class (if transpile: Import/Derive/Impl/Other/…) | Severity | Evidence | Tracked? | Native answer (DN-111) if known |
|---|---|---|---|---|---|
| `lib/std/*.myc` parity | Import/Impl | high | zero-hand-port ledger | M-993 | Per-phylum differential |
| Emitter acceptance | Derive/Impl | high | DN-136 Phase-2 | M-1084/M-1086 | Blocks checked_fraction |

## 5. Tests / witnesses
- Crate tests under `src/tests/` or `tests/` per house layout; run via `cargo test -p mycelium-std-ternary`.
- Property/differential where documented in crate `lib.rs` (e.g. numerics properties, mir-passes RC differential).

## 6. Delta vs prior assessment
| Prior | Now on tip |
|---|---|
| `language-completeness-gap-inventory.md` | Transpile path still early; this crate's Rust bar largely met for stdlib |
| G1 SYNTHESIS (2026-07-16) | G2 extends PARTITION; no contradiction — defers bulk port to M-740/M-993 |

## 7. Recommended next actions (ranked)
1. Keep changes scoped under ADR-045 / DN-99 when touching public API.
2. Graduate matching `lib/std` phylum with differential witness (M-993).
3. Do not treat this crate as bulk transpile pilot until std-cmp vet moves off 0% checked.

## 8. FLAGs
None (orch integrates shared-file items at dev→integration).
