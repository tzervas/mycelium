# Gap report — `mycelium-std-collections`

| Field | Value |
|---|---|
| **Crate** | `mycelium-std-collections` |
| **Scope group** | stdlib |
| **Agent model** | grok-composer-2.5-fast |
| **Tip** | origin/dev @ cd71de69 |
| **Honesty** | `Empirical` unless cited otherwise |

## 1. Role in the system
Vec/map-style collections over `Value` (`src/lib.rs` ~225 lines). Port: `lib/std/collections.myc`.

## 2. Public surface snapshot
- Six `src/*.rs` modules; bounded collection ops per ADR-035 stdlib bar.

## 3. Completeness vs Rust-implementation bar
| Gap | Severity | Evidence | Tracked? | Notes |
|---|---|---|---|---|
| Capacity/bounded generic policies | med | spec + E13-1 bar | E13-1 done | Narrow bar |

## 4. Transpile-to-Mycelium readiness
| Gap | Class | Severity | Evidence | Tracked? | Native answer |
|---|---|---|---|---|---|
| Generic collection impl emission | Impl | high | DN-34 Impl bucket | M-1086 | Derive+impl |
| Macro-generated impls | Other | high | UNION-BACKLOG | M-875 | expand-first |

## 5. Tests / witnesses
Collection laws; bounded op tests.

## 6. Delta vs prior assessment
`collections.myc` exists; `checked_fraction` file-gated until full file clean.

## 7. Recommended next actions (ranked)
1. M-875 expand-first for macro impls.
2. Differential `collections.myc` vs Rust crate.

## 8. FLAGs
None.
