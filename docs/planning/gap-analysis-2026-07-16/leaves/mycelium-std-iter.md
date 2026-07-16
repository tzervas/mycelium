# Gap report — `mycelium-std-iter`

| Field | Value |
|---|---|
| **Crate** | `mycelium-std-iter` |
| **Scope group** | stdlib |
| **Agent model** | grok-composer-2.5-fast |
| **Tip** | origin/dev @ cd71de69 |
| **Honesty** | `Empirical` unless cited otherwise |

## 1. Role in the system
Iterator/HOF surface (RFC-0031 alignment). `src/lib.rs` ~1131 lines. Port: `lib/std/iter.myc`.

## 2. Public surface snapshot
- Seven `src/*.rs` modules; adapter APIs.

## 3. Completeness vs Rust-implementation bar
| Gap | Severity | Evidence | Tracked? | Notes |
|---|---|---|---|---|
| Capturing closures in L1 | low | M-704 done | M-704 | Rust RUN |

## 4. Transpile-to-Mycelium readiness
| Gap | Class | Severity | Evidence | Tracked? | Native answer |
|---|---|---|---|---|---|
| `Iterator` trait impl + assoc types | Impl | high | transpile trait emission | M-1084/M-1086 | Import+Impl |
| Closure/fn-pointer gaps | Other | high | UNION-BACKLOG | M-875 | expand-first |

## 5. Tests / witnesses
Adapter tests; HOF differential where wired.

## 6. Delta vs prior assessment
L1 HOF complete in Rust; transpile iter crate remains Impl/Import heavy. `iter.myc` hand port exists.

## 7. Recommended next actions (ranked)
1. Trait impl emission for `Iterator`.
2. Align `iter.myc` API with Rust for differential.

## 8. FLAGs
None.
