# Gap report — `mycelium-std-text`

| Field | Value |
|---|---|
| **Crate** | `mycelium-std-text` |
| **Scope group** | stdlib |
| **Agent model** | grok-composer-2.5-fast |
| **Tip** | origin/dev @ cd71de69 |
| **Honesty** | `Empirical` unless cited otherwise |

## 1. Role in the system
Bytes/string/text operations (`src/lib.rs` ~322 lines). Port: `lib/std/text.myc`.

## 2. Public surface snapshot
- Text/bytes APIs; conformance to stdlib text spec.

## 3. Completeness vs Rust-implementation bar
| Gap | Severity | Evidence | Tracked? | Notes |
|---|---|---|---|---|
| Encoding edge coverage | med | spec conformance | — | Where landed, witnessed |

## 4. Transpile-to-Mycelium readiness
| Gap | Class | Severity | Evidence | Tracked? | Native answer |
|---|---|---|---|---|---|
| str/Bytes inherent + trait impls | Impl | high | DN-34 Impl/Union | M-1090/M-1086 | fmt+cmp linkage |
| String constructor macros | Impl | med | transpile emit | M-1090 | Overlaps fmt |

## 5. Tests / witnesses
Codec tests; bounded slice operations.

## 6. Delta vs prior assessment
`text.myc` in `lib/std`; transpile pilot (std-cmp) still 0% checked — shared derive/fmt levers.

## 7. Recommended next actions (ranked)
1. Joint fmt+text transpile lowering tests.
2. Close Ord/Eq derive rows used by text types.

## 8. FLAGs
None.
