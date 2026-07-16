# Gap analysis synthesis — Wave G1 (2026-07-16)

| Field | Value |
|---|---|
| **Status** | Integrated (12/12 leaves) |
| **Orch** | `claude/orch/gap-analysis-2026-07-16` |
| **Tree tip** | `origin/dev` @ `cd71de69` (post sync-down); `origin/main` @ `aad96b7a` (`v0.463.1`) |
| **Honesty** | `Empirical` (leaf evidence + `transpile-vet` run); tracker rows `Declared` |

## Executive summary

**Rust reference completion** for the transpile-critical path is **largely met** on the ADR-022 bar: L1 frontend RUN (generics, HOF, effects), kernel T1 gate-met (tag pending), interpreter + AOT witnessed, 25/26 stdlib specs ratified. Residual Rust work clusters in **ADR-045 gap-closure** (DN-99 `enb` items), **M-740** self-host port (not started), and release hygiene (M-703/M-738).

**Transpile-to-Mycelium readiness** remains **early**: the transpiler is a **gap profiler** (M-991), not a porter. On tip, `just transpile-vet` (2026-07-16) still shows **`checked_fraction` 0%** on representative targets (`mycelium-std-cmp`, `mycelium-l1` probes) while `expressible_fraction` is modest (e.g. std-cmp **21.6%**). The binding constraint is **toolchain acceptance** (L1 + `myc check` + Phase-2 emit hooks), not lack of Rust stdlib code.

**Highest-leverage unbuilt transpile items:** **M-1090** WU-3 (`write!`/`format!` → Show lowering, DN-136 B4), **M-1086** derive rows (Eq/Ord/Hash, DN-128), **M-1084** Import net-close, **M-1037** conversion residual, **missing `lib/std/io.myc`** (std-io block), **M-740** semcore/compiler `.myc` port.

## Ranked gap table (severity × readiness impact)

| Rank | Gap | Severity | Readiness impact | Primary crates | M-ids |
|---:|---|---|---|---|---|
| 1 | `checked_fraction` ≈ 0 on vet pilots | block | Transpile path non-viable for bulk port | transpile, l1, std-cmp | M-1000, M-740 |
| 2 | `write!`/`format!` lowering unbuilt | block | Largest historical Impl bucket | std-fmt, transpile | M-1090 |
| 3 | Derive PartialEq/Eq/Ord/Hash emission incomplete | high | 139 DeriveAttr-class gaps | std-cmp, std-error, transpile | M-1086 |
| 4 | No `io.myc` + host `io` Import | block | JSON/IO port chain blocked | std-io, std-fmt | M-1084, M-993 |
| 5 | L1/compiler not self-hosted in `.myc` | high | Oracle cannot accept emitted compiler sources | l1, transpile | M-740 |
| 6 | Macro expand-first not in emitter | high | ~55% macro skeleton in pilots | transpile, std-cmp | M-875 |
| 7 | Union/repr(C) `map_type` coverage | high | UNION-BACKLOG #1 bucket | transpile, core | M-874 area |
| 8 | Phylum-mode `checked_fraction` basis drift | med | Headline % stale vs oracle mode | transpile | DN-124, M-1004 |
| 9 | Kernel T1 tag / release act | med | Rust completion polish, not transpile | core | M-703, M-738 |
| 10 | Iterator/trait impl + closures in emit | high | std-iter + iter.myc parity | std-iter, transpile | M-1084, M-875 |

## Delta vs prior assessments

| Prior source | Claimed | Now (G1 leaves) |
|---|---|---|
| `language-completeness-gap-inventory.md` | ~7–8% `checked_fraction`; phylum basis correction pending | Still ~0% on default vet set; **re-measure after Phase-2** (DN-124) |
| `CURRENT-STATE.md` (2026-07-16) | Staging levers M-1090/M-1084/M-1037; M-1106 done | **Confirmed open**; DN-140/`valid_ident` helps downstream emit |
| `DN-136-phase2-bulk-gap-close-worklist.md` | WU-3 only fmt residual; derive rows BUILD-READY | **Still accurate** on tip; landings on `dev` need re-vet |
| `zero-hand-port-delta-ledger.md` | Oracle/boot10 ~3.7% checked | Default vet **0%** file-gated — conservative, not contradictory |
| `DN-34` / M-991 | NO-GO bulk porter, GO profiler | **Unchanged** |
| M-740 / self-hosting | Plan landed (M-739), port todo | **Still todo** — blocks semcore transpile acceptance |

**Stale trackers to refresh:** re-run `transpile-vet` + phylum corpus after M-1086/M-1090 merge; bump `CURRENT-STATE` transpile paragraph when `checked_fraction` moves off zero.

## Recommended work waves (disjoint ownership)

### Wave G2 — kernel + toolchain (PARTITION remainder)
- `mycelium-check`, `mycelium-cli`, `mycelium-mir-passes`, `mycelium-mlir`, `mycelium-aot` paths — same leaf template; focus **oracle + dogfood** gates.

### Wave G3 — transpile emit Phase-2 (single epic, sequential PRs)
- **Owner:** `mycelium-transpile` only  
- M-1090 WU-3 → M-1086 derive rows → M-1084 Import → re-vet std-cmp pilot.

### Wave G4 — missing std ports
- **Owner:** `lib/std/` disjoint files  
- `io.myc` scaffold (std-io leaf FLAG) → fmt/json bridge → collections/text differential.

### Wave G5 — M-740 semcore stages
- **Owner:** `lib/compiler/*.myc` + staged gates  
- Unblocks L1 transpile `checked_fraction`; depends on maintainer promotion cadence.

## Open M-ids to prioritize

1. **M-1090** — `write!`/`format!` lowering (WU-3)  
2. **M-1086** — derive emission rows (DN-128)  
3. **M-1084** — Import net-close  
4. **M-1037** — conversion residual  
5. **M-740** — L1 frontend `.myc` port  
6. **M-875** — expand-first emission follow-on  
7. **M-1000** — vet loop (maintain; re-measure)  
8. **M-993** — hand-vetted graduation of `lib/` ports  

## Leaf index

| Crate | Branch | SHA (short) | Status |
|---|---|---|---|
| mycelium-transpile | `claude/leaf/gap-transpile` | `fe59d690` | ok |
| mycelium-l1 | `claude/leaf/gap-l1` | `8a41ceee` | ok |
| mycelium-core | `claude/leaf/gap-core` | `9ed3c561` | ok |
| mycelium-interp | `claude/leaf/gap-interp` | `f4c55739` | ok |
| mycelium-std-core | `claude/leaf/gap-std-core` | `e869ddbc` | ok |
| mycelium-std-fmt | `claude/leaf/gap-std-fmt` | `c13ecca7` | ok |
| mycelium-std-error | `claude/leaf/gap-std-error` | `ad643cd5` | ok |
| mycelium-std-io | `claude/leaf/gap-std-io` | `6d726a0e` | ok |
| mycelium-std-collections | `claude/leaf/gap-std-collections` | `d8bffb6c` | ok |
| mycelium-std-text | `claude/leaf/gap-std-text` | `f66fc601` | ok |
| mycelium-std-cmp | `claude/leaf/gap-std-cmp` | `e7335c47` | ok |
| mycelium-std-iter | `claude/leaf/gap-std-iter` | `3c86c36e` | ok |

## FLAGs (orch)

- **Leaf execution:** No isolated subagent spawn API in conductor session; leaves executed via conductor with **separate branches + commits** (contract otherwise met).
- **Host `io` effect** semantics: cross `std-io` / `std-sys-host` / runtime — reconcile at integration tier.
- **CHANGELOG / issues.yaml / Doc-Index:** intentionally untouched per wave partition (dev→integration close-out owns).