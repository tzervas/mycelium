# Gap report — `mycelium-transpile`

| Field | Value |
|---|---|
| **Crate** | `mycelium-transpile` |
| **Scope group** | transpile |
| **Agent model** | grok-composer-2.5-fast |
| **Tip** | origin/dev @ cd71de69 (sync-down post v0.463.1; main @ aad96b7a) |
| **Honesty** | `Empirical` unless cited otherwise |

## 1. Role in the system
Rust→Mycelium spike (`syn` AST walk) producing `.myc` + never-silent `.gap.json`, with optional `--vet` running the real `myc check` oracle (`crates/mycelium-transpile/README.md`). M-991/DN-34 §8.7 verdict: **gap-profiling instrument**, not a bulk porter.

## 2. Public surface snapshot
- Modules: `transpile`, `batch`, `emit`, `map`, `gap`, `reserved`, `vet` (`src/lib.rs`).
- `Category` / `GapReason` taxonomy (`src/gap.rs:17+`); recursion guard `guarded` (`gap.rs:358+`).
- Metrics: `expressible_fraction` (`Declared`) vs `checked_fraction` (`Empirical`, `src/vet.rs`).

## 3. Completeness vs Rust-implementation bar
| Gap | Severity | Evidence | Tracked? | Notes |
|---|---|---|---|---|
| Macro expand-first pre-pass | high | `fixtures/UNION-BACKLOG.md`; M-875 | M-875 | DN-100 design done; emission open |
| Union/repr(C) in `map_type` | high | `gap.rs`; DN-34 §8.5 | M-874 area | Historical 36% demand bucket |
| Derive/impl emission completeness | high | `src/emit/derives/*`; DN-136 | M-1086, M-1090 | Phase-2 on dev |
| Workstack depth on AST walks | med | `gap.rs:312+` | wired | Budget integrated |

## 4. Transpile-to-Mycelium readiness
| Gap | Class | Severity | Evidence | Tracked? | Native answer |
|---|---|---|---|---|---|
| `checked_fraction` ≪ expressible on vet targets | Other | block | `transpile-vet.sh` 2026-07-16: 0% checked on std-cmp/L1 probes | M-1000 | File-gated oracle |
| Default vet set not phylum corpus | Import | high | `README.md:57-60` | M-1004/M-1005 | lib-index separate |
| Ident/reserved collisions | Other | med | `reserved.rs` (M-1001) | M-1001 done | DN-140 helps downstream |

## 5. Tests / witnesses
In-crate `src/tests/` per house layout; vet outcomes recorded in `vet.json` (never silent `ToolUnavailable`).

## 6. Delta vs prior assessment
`language-completeness-gap-inventory.md` / CURRENT-STATE: ~12.4% expressible union vs ~3.7% boot10 `checked_fraction`. **Now:** representative `just transpile-vet` still reports **0% checked** on std-cmp — lever is toolchain acceptance + Phase-2 emit hooks, not emission alone.

## 7. Recommended next actions (ranked)
1. Land M-1090 WU-3 (`write!`/`format!` lowering per DN-136 B4).
2. Close M-1084 Import net and M-1037 conversion residuals.
3. Re-measure phylum-mode `checked_fraction` (DN-124) after derive rows land.

## 8. FLAGs
- Union backlog re-ranking is orch-owned (`fixtures/UNION-BACKLOG.md`).