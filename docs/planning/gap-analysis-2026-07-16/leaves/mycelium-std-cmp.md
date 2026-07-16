# Gap report — `mycelium-std-cmp`

| Field | Value |
|---|---|
| **Crate** | `mycelium-std-cmp` |
| **Scope group** | stdlib |
| **Agent model** | grok-composer-2.5-fast |
| **Tip** | origin/dev @ cd71de69 |
| **Honesty** | `Empirical` unless cited otherwise |

## 1. Role in the system
Ordering/equality (`Ord3`, derives). Large surface (`src/lib.rs` ~1604 lines). Transpile pilot crate. Port: `lib/std/cmp.myc`.

## 2. Public surface snapshot
- `Ord3`/eq APIs; derive linkage per DN-128 (refuse ineligible fields, e.g. Float/NaN ADR-040).

## 3. Completeness vs Rust-implementation bar
| Gap | Severity | Evidence | Tracked? | Notes |
|---|---|---|---|---|
| Float total order policy | med | DN-128 derive gate | DN-128 | Refuse-whole-impl pattern |

## 4. Transpile-to-Mycelium readiness
| Gap | Class | Severity | Evidence | Tracked? | Native answer |
|---|---|---|---|---|---|
| 0% checked / 21.6% expressible on vet | Other | block | transpile-vet 2026-07-16 (111 items) | M-1000 | File-gated |
| Derive PartialEq/Eq/Ord emission | Derive | high | DN-136 B1 | M-1086 | 139 DeriveAttr corpus |

## 5. Tests / witnesses
Ord3 laws; derive conformance tests.

## 6. Delta vs prior assessment
Historical transpile pilot + union backlog leader. Phase-2 derive rows on dev should move metric — **re-vet required**.

## 7. Recommended next actions (ranked)
1. Re-run `transpile-vet` on `mycelium-std-cmp` after M-1086.
2. Land DN-128 Eq/Ord derive emit rows.

## 8. FLAGs
None.