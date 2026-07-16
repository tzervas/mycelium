# Gap report — `mycelium-lsp`

| Field | Value |
|---|---|
| **Crate** | `mycelium-lsp` |
| **Scope group** | toolchain |
| **Agent model** | grok-composer-2.5-fast |
| **Tip** | origin/dev @ 708bbc14 |
| **Honesty** | `Empirical` unless cited otherwise |

## 1. Role in the system
LSP + diagnostic policy + baseline registry (M-362). Part of Wave G2 PARTITION (toolchain group). Rust reference surface is primary; transpile impact is mostly indirect unless noted.

## 2. Public surface snapshot
- Crate root `crates/mycelium-lsp/src/` (~7279 LOC Rust, repo count 2026-07-16).
- Key entry: `src/lib.rs` module graph and re-exports.
- Guarantee tags: per module docs (`Exact`/`Proven`/`Empirical`/`Declared`) where operations emit `Meta`.

## 3. Completeness vs Rust-implementation bar
| Gap | Severity (block/high/med/low) | Evidence (file:line / test) | Tracked? (M-id / none) | Notes |
|---|---|---|---|---|
| Residual domain work | low | `no TODO/FIXME in src grep (mycelium-lsp)` | DN-99 / enb | ADR-045 bounded diffs only |
| Release / spec hygiene | low | G1 SYNTHESIS | M-703 area | Not transpile-blocking |

## 4. Transpile-to-Mycelium readiness
How this crate affects / is affected by `mycelium-transpile` + `lib/*.myc` port:
| Gap | Class (if transpile: Import/Derive/Impl/Other/…) | Severity | Evidence | Tracked? | Native answer (DN-111) if known |
|---|---|---|---|---|---|
| Oracle path (`myc check`) | Other | high | G1: checked_fraction ≈0 | M-1000 | Must accept emitted lib |
| Dogfood on own sources | Impl | med | Self-host later | M-740 | Deferred |

## 5. Tests / witnesses
- Crate tests under `src/tests/` or `tests/` per house layout; run via `cargo test -p mycelium-lsp`.
- Property/differential where documented in crate `lib.rs` (e.g. numerics properties, mir-passes RC differential).

## 6. Delta vs prior assessment
| Prior | Now on tip |
|---|---|
| `language-completeness-gap-inventory.md` | Transpile path still early; this crate's Rust bar largely met for toolchain |
| G1 SYNTHESIS (2026-07-16) | G2 extends PARTITION; no contradiction — defers bulk port to M-740/M-993 |

## 7. Recommended next actions (ranked)
1. Keep changes scoped under ADR-045 / DN-99 when touching public API.
2. Run `myc check` dogfood when editing toolchain-facing types.
3. Do not treat this crate as bulk transpile pilot until std-cmp vet moves off 0% checked.

## 8. FLAGs
None (orch integrates shared-file items at dev→integration).
