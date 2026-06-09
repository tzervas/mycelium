# M-001 — Liquid-Haskell `bundle` capacity-refinement probe

| Field | Value |
|---|---|
| **Task** | M-001 ([#2](https://github.com/tzervas/mycelium/issues/2)) · P0 · verification |
| **Status** | **Scaffolded — NOT yet discharged.** The arithmetic is tabulated and independently checkable below; running LiquidHaskell is the remaining step. |
| **Grounding** | RFC-0003 §5; ADR-010; T0.2 (Clarkson-Ubaru-Yang 2023; Thomas-Dasgupta-Rosing 2021); KC-1 |
| **Confirms** | the *axiomatized-theorem + checked-instantiation* strategy for honest VSA bounds |

## What this probe is

The single **confirming build** the research left open (Doc-Index §4; RFC-0003 §5). It encodes the
MAP-I `bundle` capacity bound as a refinement type and asks Z3 to discharge the **arithmetic
instantiation** for several concrete settings. Success ratifies the strategy that lets Mycelium tag
`bundle` as `Proven` honestly: **axiomatize the cited theorem's statement, and have the checker
discharge only the concrete arithmetic** — not re-prove the concentration inequality.

The refinement (RFC-0003 §5):

```
bundle :: {v | activeCount v ≤ s}
        → {d | d ≥ ⌈(2/μ²)·ln(m/δ)⌉}
        → {r | failProb r ≤ δ}
```

- The **theorem** (`d ≥ requiredDim(m,δ) ⟹ failProb ≤ δ`) is **axiomatized** — its soundness is
  cited (T0.2), *not* proven by the type system. In `src/Bundle.hs` this is the `assume capacityThm`.
- The **checked part** is the integer inequality `d ≥ requiredDim(m,δ)` for each concrete instance.
  That is what Z3 discharges. In `src/Bundle.hs` these are `probe1 … probe4`.

## ⚠️ Honesty status (VR-5)

This environment has **no GHC / LiquidHaskell / Z3**, so the module has **not been type-checked and
Z3 has not run**. Therefore:

- **KC-1 remains `passed (literature)`** — it is **not** upgraded to `confirmed (build)`. That
  upgrade is gated on an actual green LH run (the acceptance criterion of #2).
- `src/Bundle.hs` is a **DRAFT**: idiomatic and intended to be correct, but unverified. Treat the
  table below — not the Haskell — as the authoritative, independently-checkable artifact.

## Derivation table (independently checkable)

With illustrative normalized margin **μ = 0.1** (so `2/μ² = 200`), `requiredDim = ⌈200·ln(m/δ)⌉`.
μ is a model parameter; the *formula* is the axiomatized Clarkson/Thomas statement (T0.2). All four
settings use dimension **d = 10000** (a standard MAP-I width):

| probe | items m | target δ | ln(m/δ) | requiredDim = ⌈200·ln(m/δ)⌉ | d = 10000 ≥ requiredDim? |
|---|---|---|---|---|---|
| 1 | 3 | 1e-2 | 5.7038 | **1141** | ✓ |
| 2 | 10 | 1e-3 | 9.2103 | **1843** | ✓ |
| 3 | 50 | 1e-3 | 10.8198 | **2164** | ✓ |
| 4 | 100 | 1e-4 | 13.8155 | **2764** | ✓ |

(Reproduce: `python3 -c "import math; print([math.ceil(200*math.log(m/d)) for m,d in [(3,1e-2),(10,1e-3),(50,1e-3),(100,1e-4)]])"`.)

≥3 concrete `(d,k,s,m,δ)` settings discharge, satisfying #2's acceptance once the LH run is green.

## How to run (once the toolchain is available)

```sh
# Requires: ghcup (GHC + cabal), the liquidhaskell plugin, and z3 on PATH.
z3 --version
cabal build            # runs the LiquidHaskell GHC plugin over src/Bundle.hs
# Green build  ⟺  Z3 discharged probe1..probe4  ⟺  KC-1 → confirmed (build).
```

## On success — what to update (do NOT pre-write)

1. Flip this file's **Status** to `confirmed (build)` with the LH/Z3 version + date.
2. KC-1: `passed (literature) → confirmed (build)` in `docs/Mycelium_Project_Foundation.md` §2.4
   and `docs/Doc-Index.md` §3.
3. Close #2 with the run log; note it ratifies ADR-010's cited-theorem strategy.

## Layout decision (resolves OQ-2)

Formal/machine-checkable proofs live under **`proofs/<name>/`** (this is the first). The M-121
binary↔ternary round-trip proof (#19) will follow the same convention. Recorded so phase-1.md and
`SPECIFICATION.md` §10 can point here.
