# Gap report — `mycelium-std-error`

| Field | Value |
|---|---|
| **Crate** | `mycelium-std-error` |
| **Scope group** | stdlib |
| **Agent model** | grok-composer-2.5-fast |
| **Tip** | origin/dev @ cd71de69 |
| **Honesty** | `Empirical` unless cited otherwise |

## 1. Role in the system
`Option`/`Result` combinators and error surface (RFC-0016). Ports: `lib/std/option.myc`, `result.myc`, `error.myc`.

## 2. Public surface snapshot
- `src/lib.rs` ~249 lines; combinator + taxonomy APIs.

## 3. Completeness vs Rust-implementation bar
| Gap | Severity | Evidence | Tracked? | Notes |
|---|---|---|---|---|
| General-position `?` / try desugar | med | M-1025 first increment | M-1025 | CPS lift gated |

## 4. Transpile-to-Mycelium readiness
| Gap | Class | Severity | Evidence | Tracked? | Native answer |
|---|---|---|---|---|---|
| Result/Option derive + impl blocks | Derive | high | DN-136 B1; 139 DeriveAttr gaps | M-1086 | Eq/Hash rows |
| Error trait / module imports | Import | med | transpile gaps | M-1084 | Import net-close |

## 5. Tests / witnesses
Combinator laws; explicit error variants (C1).

## 6. Delta vs prior assessment
M-1025 landed try-operator slice; transpile Derive/Impl buckets still dominate.

## 7. Recommended next actions (ranked)
1. Land derive rows affecting Option/Result.
2. Re-vet error crate src after Phase-2.

## 8. FLAGs
None.
