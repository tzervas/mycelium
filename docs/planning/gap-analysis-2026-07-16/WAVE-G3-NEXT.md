# Wave G3-next — post-promotion emit + std scaffold (2026-07-16)

**Orch:** plan only · **Exec:** `grok-composer-2.5-fast` · **TG:** `@mycelium`
**Base after promote:** `origin/integration` (includes `dev` @ M-1090 WU-3 + close-out)

## Why this wave

G1+G2 + M-1090 WU-3 are on the trunk path. Binding residual for transpile readiness:

| Priority | Item | Status | Ownership |
|---:|---|---|---|
| 1 | **M-1084** Import net-close (symtab resolve > reclassify) | in-progress | `mycelium-transpile` |
| 2 | **M-1037** conversion-op identity / real-surface map | todo | `mycelium-transpile` |
| 3 | **M-1006** re-measure after WU-3 | in-progress | transpile-vet / scripts |
| 4 | **G4-a** missing `lib/std/io.myc` scaffold | untracked gap | `lib/std/` only |
| later | M-740 L1 `.myc` port | in-progress | `lib/compiler/` — separate epic |

## Collision discipline

- **M-1084 then M-1037 sequential** (both own `crates/mycelium-transpile/**`) — not parallel.
- **G4-a `io.myc`** parallel with M-1084 (disjoint `lib/std/io.myc`).
- **M-1006 re-measure** after M-1084 (or after both transpile leaves) so the number is post-Import.

## Leaf contracts

### L1 — M-1084 Import net-close
- Branch: `claude/leaf/M1084-import-net-close`
- Scope: `crates/mycelium-transpile/src/symtab.rs` + emit/batch wiring only as needed
- DoD: trace -2 clean regression; net-positive phylum-mode `checked_fraction` or honest documented root cause; std-fs/std-io re-verify
- Depends: M-1079 done
- Tests: change-scoped transpile + harness re-measure if present

### L2 — M-1037 conversion identity (after L1 merges to working tip)
- Branch: `claude/leaf/M1037-conversion-identity`
- Scope: transpile emit conversion methods (`to_owned`/`clone`/`into`/… → identity or real surface)
- Never fabricate unknown prims (G2)
- Tests: pin never-fabricates + corpus-oriented unit tests

### L3 — G4-a `lib/std/io.myc` scaffold (parallel with L1)
- Branch: `claude/leaf/G4a-std-io-myc-scaffold`
- Scope: **only** `lib/std/io.myc` (+ tests if in-crate pattern; FLAG proj.toml if required)
- Honest minimal nodule: types/effects stubs matching std-io leaf FLAG; `myc check` clean or explicit residual
- No full host effect design (FLAG to orch / DN)

### L4 — M-1006 / transpile-vet re-measure (after L1+L2)
- Branch: `claude/leaf/M1006-remeasure-post-wu3`
- Run `just transpile-vet` (or project recipe) on pilot targets; record Empirical results under
  `docs/planning/` or `experiments/results/` per repo convention; FLAG issues.yaml M-1090 if numbers support partial close

## Conductor
1. Wait for promote-to-integration merge green.
2. Branch work off **pushed** `origin/dev` or `origin/integration` tip (same content post-promote).
3. Spawn L1 + L3 in parallel; after L1 lands, spawn L2; then L4.
4. Each leaf PR → `dev`; orch reconciles shared files once; promote batch → integration.
5. TG `[mycelium · g3]` batch status; no TUI wait.
