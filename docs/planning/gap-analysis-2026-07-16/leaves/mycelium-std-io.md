# Gap report — `mycelium-std-io`

| Field | Value |
|---|---|
| **Crate** | `mycelium-std-io` |
| **Scope group** | stdlib |
| **Agent model** | grok-composer-2.5-fast |
| **Tip** | origin/dev @ cd71de69 |
| **Honesty** | `Empirical` unless cited otherwise |

## 1. Role in the system
Affine IO + canonical serialize/JSON (`M-514`, `src/lib.rs`). Single-consumption LR-8 (`lib.rs:35-39`).

## 2. Public surface snapshot
- `serialize` wire + JSON; `io::Source`/`Sink` read/write.
- **No `lib/std/io.myc`** in tree (listing 2026-07-16).

## 3. Completeness vs Rust-implementation bar
| Gap | Severity | Evidence | Tracked? | Notes |
|---|---|---|---|---|
| JSON canonical Q1 sign-off | low | `lib.rs:10` FLAG | open | fmt delegates here |

## 4. Transpile-to-Mycelium readiness
| Gap | Class | Severity | Evidence | Tracked? | Native answer |
|---|---|---|---|---|---|
| Missing `io.myc` hand port | Import | block | `lib/std/` listing; DN-136 phylum ok false | M-993/M-740 | Host bridge needed |
| `io` host effect for ported code | Import | high | RFC-0014 | M-1084 | Import net-close |

## 5. Tests / witnesses
Round-trip proptest (`Empirical`); located `SerError`/`IoError`.

## 6. Delta vs prior assessment
DN-136 lists std-io among **phylum ok: false** set. Blocks fmt/json port closure at scale.

## 7. Recommended next actions (ranked)
1. Scaffold `lib/std/io.myc` with serialize bridge.
2. Land M-1084 Import lowering for host `io`.

## 8. FLAGs
- Host effect semantics cross `mycelium-std-sys-host` / runtime — orch-owned.
