# Gap report — `mycelium-std-core`

| Field | Value |
|---|---|
| **Crate** | `mycelium-std-core` |
| **Scope group** | stdlib |
| **Agent model** | grok-composer-2.5-fast |
| **Tip** | origin/dev @ cd71de69 |
| **Honesty** | `Empirical` unless cited otherwise |

## 1. Role in the system
Ring-0 prelude re-exporting kernel value model + read-only queries (`src/lib.rs`, `docs/spec/stdlib/core.md`).

## 2. Public surface snapshot
- Re-exports `Value`, `Repr`, `Meta`, guarantee lattice; query fns.
- Hand port: `lib/std/core.myc` (present).

## 3. Completeness vs Rust-implementation bar
| Gap | Severity | Evidence | Tracked? | Notes |
|---|---|---|---|---|
| ADR-045 unfreeze amendments | med | `lib.rs:46-49` | ADR-045 | Spec+changelog required |
| Ambient repr pass M-540 | low | `lib.rs:31` | M-540 | Scheduled |

## 4. Transpile-to-Mycelium readiness
| Gap | Class | Severity | Evidence | Tracked? | Native answer |
|---|---|---|---|---|---|
| Thin Rust vs hand `core.myc` alignment | Other | med | `lib/std/core.myc` | M-993 | Hand-vetted path |
| `map_type` for kernel re-exports | Import | med | transpile `map.rs` | M-874 | Type mapping |

## 5. Tests / witnesses
Guarantee matrix table tests; RFC-0016 conformance.

## 6. Delta vs prior assessment
DN-66 freeze lifted by ADR-045. Port exists; not in default transpile-vet set.

## 7. Recommended next actions (ranked)
1. Keep `core.myc` synced with kernel re-exports.
2. Use as Ring-0 hub for other `lib/std` ports.

## 8. FLAGs
None.
