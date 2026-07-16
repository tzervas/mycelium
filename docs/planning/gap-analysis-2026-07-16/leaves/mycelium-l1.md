# Gap report — `mycelium-l1`

| Field | Value |
|---|---|
| **Crate** | `mycelium-l1` |
| **Scope group** | frontend |
| **Agent model** | grok-composer-2.5-fast |
| **Tip** | origin/dev @ cd71de69 |
| **Honesty** | `Empirical` unless cited otherwise |

## 1. Role in the system
Surface lexer/parser, typechecker, totality, fuel-guarded evaluator, elaborator to Core IR (`README.md`). Feeds the `myc check` oracle used by transpile `--vet`.

## 2. Public surface snapshot
- `parse`, `check_nodule`/`check_phylum`, `totality`, `eval`, `elab`, `mono`, `usefulness` (`src/lib.rs`).
- Staged constructs → `ElabError::Residual` (`src/ast.rs:815+`, `lib.rs:13`).

## 3. Completeness vs Rust-implementation bar
| Gap | Severity | Evidence | Tracked? | Notes |
|---|---|---|---|---|
| Eval-complete fragment ⊂ full surface | high | `elab` refuses outside fragment | M-740 | Self-host absorbs shape |
| Multi-arg lambda / partial application | med | `ast.rs:904` Residual | DN-99 enb | ADR-045 window |
| Ambient pass depth budgets | low | `ambient.rs:83,320` | M-674 | TODO item 2 |
| Cross-nodule AOT parity gaps | med | CURRENT-STATE M-1024 | follow-ups | Interp witness stronger |

## 4. Transpile-to-Mycelium readiness
| Gap | Class | Severity | Evidence | Tracked? | Native answer |
|---|---|---|---|---|---|
| Transpiled L1 files fail file-gated check | Import | block | transpile-vet `eval.rs`: 0% checked, 11.9% expressible | M-740 | Need `.myc` compiler lib |
| Rust generics/effects ≫ ported semcore | Other | high | CURRENT-STATE L1 complete in Rust | M-740 todo | M-739 plan landed |

## 5. Tests / witnesses
`tests/differential.rs` three-way (NFR-7); extensive `src/tests/`.

## 6. Delta vs prior assessment
Prior: generics/HOF/effects **RUN** in Rust (CURRENT-STATE). **Now:** transpile readiness lags — M-740 still `todo`; transpiled L1 sources do not pass `myc check`.

## 7. Recommended next actions (ranked)
1. Execute M-740 staged port with differential gates (M-739).
2. Prioritize `lib/compiler/*.myc` increments blocking vet.
3. Close DN-99 Residual-gated surface items.

## 8. FLAGs
- M-740 promotion cadence is maintainer/orch tracker only (`issues.yaml`).