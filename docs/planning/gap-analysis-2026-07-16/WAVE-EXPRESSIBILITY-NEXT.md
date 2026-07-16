# Wave expressibility-next — one-shot transpile readiness worklist (2026-07-16)

| Field | Value |
|---|---|
| **Status** | Planning + one leaf shipped (`std.io` phylum export) |
| **Base** | `origin/dev` @ post-companion merge (`8b35c2df` area) |
| **Honesty** | Tracker rows `Declared`; vet numbers `Empirical` when cited |
| **Doctrine** | `docs/companion/06-expressibility-and-transpile.md` |

## Executive read

One-shot readiness means every construct class on the port surface has a **named native strategy**
(Accepted + tracked build, or honest Approximation/Bridge with EXPLAIN). **`checked_fraction`**
(live `myc check`) is the headline — not `expressible_fraction` alone (VR-5).

**Post G1–G3 on `dev`:** M-1090 WU-3 (`write!`/`format!` lowering) landed; `lib/std/io.myc`
scaffold exists; M-1037 **partial** identity rows + never-fabricate pins landed in
`mycelium-transpile`; M-1084 symtab gained `self::`/`super::`/cross-phylum resolution (net-close
**still open**). Binding residuals: **Import net-positive measure**, **M-1037 DoD closure**,
**M-1006 re-measure**, **M-740** oracle on compiler sources, **M-875** expand-first (design-gated),
**host `io` effect** reconciliation (FLAG, orch-owned).

## Issue tracker truth (verify-first, 2026-07-16)

| M-id | `issues.yaml` status | Code truth vs tracker |
|---|---|---|
| **M-1090** | `todo` | WU-1/WU-2/WU-3 **landed** on `dev` (PR #1630); issue scoped to **re-measure + OQ-1/float residuals** |
| **M-1084** | `in-progress` | Symtab M-1084 extensions landed; **−2 phylum regression + Import count rise** not net-closed |
| **M-1037** | `todo` | **Partial:** `prim_map` identity (`clone`/`to_owned`/accessors), `.ne` compose, never-fabricate tests; **not** full DoD (`into`/`to_string`/`to_vec`, corpus `checked_fraction` rise) |
| **M-1006** | `in-progress` | Ladder active; post-WU-3/phylum re-measure **FLAG** (L4 in WAVE-G3-NEXT) |
| **M-740** | `in-progress` | M-739 plan landed; staged `.myc` port ongoing — blocks compiler-source oracle |
| **M-875** | `needs-design` | Expand-first pre-pass **not built**; nightly/`cargo expand` env FLAG from issue body |
| **M-1086** | see tracker | Derive Eq/Ord/Hash rows — parallel transpile lever after Import/conversion batch |

## Ranked next waves (disjoint ownership)

### Wave E1 — transpile Import net-close (sequential, single crate)

| Rank | Leaf | Branch pattern | Owns | M-id | DoD sketch |
|---:|---|---|---|---|---|
| 1 | Import net-close | `claude/leaf/M1084-import-net-close` | `crates/mycelium-transpile/src/symtab.rs` + minimal emit/batch wiring | M-1084 | Trace/fix −2 clean regression; net-positive phylum `checked_fraction`; re-verify std-fs/std-io deltas |
| 2 | Conversion residual | `claude/leaf/M1037-conversion-identity` | `crates/mycelium-transpile/src/{emit,prim_map}.rs` + tests | M-1037 | Map or gap `into`/`to_string`/`to_vec` without fabrication; record pilot/corpus measure |
| 3 | Vet re-measure | `claude/leaf/M1006-remeasure-post-wu3` | `docs/planning/` or `experiments/results/` + scripts only | M-1006, M-1090 | `just transpile-vet` on pilots + phylum corpus; **Empirical** table; FLAG orch for `issues.yaml` close-out |

**Collision:** E1 leaves **must not** run in parallel (same `mycelium-transpile` tree). Order: 1 → 2 → 3.

### Wave E2 — std phylum surface (parallel with E1-L1)

| Rank | Leaf | Branch pattern | Owns | Notes |
|---:|---|---|---|---|
| 1 | **`std.io` phylum export** | `claude/leaf/express-residual-stdio-export` | `lib/std/mycelium-proj.toml` only | **Shipped this wave** — `io.myc` on `dev`; export unblocks phylum-mode `use std.io.*` |
| 2 | `io` differential witness | `claude/leaf/G4a-std-io-differential` | `crates/mycelium-l1/tests/std_io.rs` (new) | Optional; mirrors `std_fmt.rs` |
| 3 | Host `io` / sys-host effects | — | **FLAG orch** | RFC-0014 / DN-107; crosses `std-sys-host`, runtime |

### Wave E3 — derive + macro levers (after E1)

| Rank | Item | Owns | M-id | Gate |
|---:|---|---|---|---|
| 1 | Derive Eq/Ord/Hash/Clone emit rows | `mycelium-transpile` + `lib/std/derive_prelude.myc` as needed | M-1086 | After E1-L2 or parallel if disjoint files only |
| 2 | Expand-first macro pre-pass | `mycelium-transpile` design + tool hook | M-875 | **needs-design** — DN-34 §8.5 |

### Wave E4 — self-host oracle (separate epic)

| Rank | Item | Owns | M-id |
|---:|---|---|---|
| 1 | L1 frontend `.myc` stages | `lib/compiler/*.myc` | M-740 |
| 2 | Hand-vetted `lib/` graduation | per-nodule | M-993 |

### Wave E5 — thematic DN build (companion §05)

DN clusters A–C **built** for port surface (not only Accepted). Scoped PRs per `/forward`.

## Top 5 next items (action order)

1. **M-1084** — Import symtab **net-close** (E1 blocker before conversion re-run).
2. **M-1037** — Close conversion DoD (`into`/`to_string`/`to_vec` + corpus measure).
3. **M-1006** — Phylum + pilot **re-measure** after E1-L1+L2.
4. **M-1086** — Derive lowering rows (DeriveAttr-class gaps).
5. **M-740** — Semcore/compiler `.myc` port (oracle for emitted compiler sources).

**Deferred / orch-owned:** M-875 expand-first (design), host `io` effect FLAG, M-1090 `done` flip after re-measure.

## This leaf delivery (E2-L1)

- **Branch:** `claude/leaf/express-residual-stdio-export`
- **Change:** Add `std.io` to `[surface].exports` in `lib/std/mycelium-proj.toml`.
- **Verify:** `./target/debug/myc-check lib/std/io.myc` → `ok` (`Empirical`).
- **FLAG:** CHANGELOG / `issues.yaml` / Doc-Index — orch-owned at integration close-out.

## See also

- `WAVE-G3-NEXT.md`, `SYNTHESIS.md`, `SYNTHESIS-G2.md`, `M1037-pilot-remeasure.md`
