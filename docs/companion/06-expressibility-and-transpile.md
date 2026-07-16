# Expressibility and the path to one-shot transpile readiness

## Doctrine (never silent)

| Rule | Meaning |
|---|---|
| **M-991** | Transpiler is a **gap profiler**, not a bulk porter |
| **DN-119** | Goal is L3 *native answers* to problems, not cloning Rust syntax |
| **DN-110/111** | Taxonomy: Native Equivalent · Idiomatic Remapping · Approximation · Interop Bridge |
| **checked_fraction** | Live-oracle `myc check` accept rate — the honest headline |
| **expressible_fraction** | Text emit without hard gap — necessary but not sufficient |

VR-5: do not report "ready" from expressible alone. G2: unmapped constructs gap
explicitly; never fabricate prims (see M-1037 conversion work).

## What "100% expressibility" means here

Not "every Rust token parses as Mycelium." It means: for every construct class
in the port surface, there is a **named native strategy** (landed or Accepted
with a tracked build issue), and the residual is either:

- an honest Approximation/Bridge with EXPLAIN, or
- a filed open work item — never a silent empty emit.

## Current leverage stack (post G1–G3, 2026-07-16)

Landed or partial (re-check Doc-Index / issues):

| Lever | Role |
|---|---|
| M-1090 WU-3 | `write!`/`format!` → Show / `bytes_concat` |
| M-1084 | Import emit qualifier (phylum-mode resolution) |
| M-1037 | Conversion identity / never-fabricate |
| M-1106 | `valid_ident` / D4 mangler |
| M-1086 | derive lowering rows |
| `lib/std/io.myc` | scaffold (host effects residual) |

Still binding (typical residual):

| Residual | Why it blocks one-shot |
|---|---|
| M-1006 re-measure | Honest `checked_fraction` after emit waves |
| M-1084 net-positive union measure | Import lever DoD |
| M-740 / M-993 | L1 / semcore `.myc` self-host for oracle on compiler sources |
| M-875 / DN-100 | Expand-first macros (design-gated) |
| Host `io` effects | Cross std-sys-host / runtime |

## One-shot readiness checklist (working definition)

- [ ] Thematic DN clusters A–C **built** for port surface (not only Accepted)
- [ ] Pilot phylum `checked_fraction` moved off 0% under phylum-mode basis (DN-124)
- [ ] No unknown-prim fabrication on conversion/method desugar
- [ ] Import resolution net-positive on union measure
- [ ] Self-host oracle accepts emitted stdlib pilots (`myc check` green)
- [ ] Companion + CURRENT-STATE document residual honestly

## Wave pattern (autonomous iteration)

```text
dev work (composer leaves, disjoint ownership)
  → PR → dev
  → canary / checks green
  → PR → integration
  → (batch) curated squash → main when maintainer-ready
  → sync-down main → integration → dev
  → next wave on fresh tip
```

See DN-97 / skills `/forward` · `/sync-down` · `/wave`.

## See also

- `docs/planning/gap-analysis-2026-07-16/` — per-crate residual
- [05 — Thematic decision map](05-thematic-decision-map.md)
