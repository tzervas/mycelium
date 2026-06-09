# M-001 — Liquid-Haskell `bundle` capacity-refinement probe

| Field | Value |
|---|---|
| **Task** | M-001 ([#2](https://github.com/tzervas/mycelium/issues/2)) · P0 · verification |
| **Status** | ✅ **Discharged — confirmed (build), 2026-06-09.** LiquidHaskell reports `SAFE (16 constraints checked)`; Z3 discharged all four probes. Toolchain: GHC 9.8.2 · LiquidHaskell 0.9.8.2 / liquid-fixpoint 0.9.6.3.1 · Z3 4.8.12. |
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

## Result & honesty status (VR-5)

`cabal build` with the LiquidHaskell plugin reports **`LIQUID: SAFE (16 constraints checked)`**
(exit 0): the module type-checks and **Z3 discharged** the verification conditions for all four
`probe` instantiations. Toolchain: GHC 9.8.2, LiquidHaskell 0.9.8.2, liquid-fixpoint 0.9.6.3.1,
Z3 4.8.12 (run under `LC_ALL=C.UTF-8`).

- **KC-1 → `confirmed (build)`** (Foundation §2.4; Doc-Index §3). The confirming build is complete.
- **What is confirmed:** the *strategy* — Z3 discharges the **arithmetic instantiation** of an
  **axiomatized** capacity theorem (`assume capacityThm`, T0.2). The Clarkson/Thomas theorem itself
  remains cited, not re-proven — exactly the ADR-010 / RFC-0002 §7 pattern. This is precisely what
  the probe was meant to ratify, not a from-scratch proof of the concentration inequality.

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

All four (≥3 required) `(d,m,δ)` settings discharge — #2's acceptance is met.

## How to reproduce

```sh
# Requires: GHC 9.8.2 + cabal (ghcup), the liquidhaskell 0.9.8.2 plugin, and z3 on PATH.
# LiquidHaskell writes a UTF-8 HTML report, so a UTF-8 locale is required:
export LC_ALL=C.UTF-8
cabal build            # runs the LiquidHaskell GHC plugin over src/Bundle.hs
#   ⇒ LIQUID: SAFE (16 constraints checked)
# Green build  ⟺  Z3 discharged probe1..probe4  ⟺  KC-1 confirmed (build).
```

## Done on confirmation (2026-06-09)

1. ✅ This file's **Status** → `confirmed (build)` with LH/Z3 versions.
2. ✅ KC-1 `passed (literature) → confirmed (build)` in `docs/Mycelium_Project_Foundation.md` §2.4
   and `docs/Doc-Index.md` §3.
3. ✅ #2 closed with the run log; ratifies ADR-010's cited-theorem + checked-instantiation strategy.

## Layout decision (resolves OQ-2)

Formal/machine-checkable proofs live under **`proofs/<name>/`** (this is the first). The M-121
binary↔ternary round-trip proof (#19) will follow the same convention. Recorded so phase-1.md and
`SPECIFICATION.md` §10 can point here.
