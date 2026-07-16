# Gap report — `mycelium-std-fmt`

| Field | Value |
|---|---|
| **Crate** | `mycelium-std-fmt` |
| **Scope group** | stdlib |
| **Agent model** | grok-composer-2.5-fast |
| **Tip** | origin/dev @ cd71de69 |
| **Honesty** | `Empirical` unless cited otherwise |

## 1. Role in the system
Human/machine projections over canonical `Value` form; JSON delegates to std-io serialize (M-514/M-533, `src/lib.rs:46-49`).

## 2. Public surface snapshot
- `display`/`debug`/`display_bounded`, JSON seam; `GUARANTEE_MATRIX`.
- Port: `lib/std/fmt.myc`.

## 3. Completeness vs Rust-implementation bar
| Gap | Severity | Evidence | Tracked? | Notes |
|---|---|---|---|---|
| DN-127 WU-3 not built | high | DN-136 §1 table | M-1090 | WU-1/WU-2 landed |

## 4. Transpile-to-Mycelium readiness
| Gap | Class | Severity | Evidence | Tracked? | Native answer |
|---|---|---|---|---|---|
| `write!`/`format!` lowering | Impl | block | DN-136 B4; DN-34 §8.22 | M-1090 | Show render fold |
| `Display` trait impl emission | Derive | high | `emit/show.rs` | M-1090/M-1086 | Largest Impl bucket |

## 5. Tests / witnesses
Truncation never-silent tests; JSON delegation tests.

## 6. Delta vs prior assessment
DN-136: **only WU-3 residual** for fmt transpile path. CURRENT-STATE lists M-1090 as next staging lever.

## 7. Recommended next actions (ranked)
1. Implement M-1090 WU-3 transpiler lowering.
2. Re-vet fmt-related impls after Phase-2 emit hooks.

## 8. FLAGs
None.