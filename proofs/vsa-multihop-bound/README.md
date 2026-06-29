# VSA Multi-Hop Bound — Proof Artifact Directory (M-832 / OQ-F)

| Field | Value |
|---|---|
| **Task** | M-832 / OQ-F (multi-hop compositional `Proven` bounds) |
| **Status** | **Declared** — candidate theorem; proof obligations pending discharge |
| **Related** | OQ-A (M-827, graded-soundness machine-checked proof); M-001 (single-hop LH probe) |
| **Grounding** | RFC-0003 §5; ADR-010; T0.2 (Clarkson-Ubaru-Yang 2023; Thomas-Dasgupta-Rosing 2021) |

> **Guarantee: Declared throughout.**
> This directory holds candidate theorems and emitted proof obligations — NOT proofs.
> The `Proven` tag requires a proof assistant to discharge the obligations AND
> the underlying theorem to be formally established or cited (VR-5 / ADR-032).

---

## What this directory is

The single-hop MAP-I bundle capacity bound (`required_dim(m, delta) = ceil(200 * ln(m/delta))`,
Clarkson-Ubaru-Yang 2023 Thm 6) is **Proven** via the checked-instantiation pattern:
axiomatize the theorem, have Z3 discharge the concrete arithmetic
(see `proofs/lh-bundle/` — M-001, discharged 2026-06-09).

**OQ-F** asks: does this bound extend to multi-hop VSA compositions
(bind-chains, bundle-of-binds, nested unbind)?  The answer is unknown — it is the
core open question this directory addresses.

This directory holds the **bridge artifacts** that the GPU experiment
(`experiments/mycelium_experiments/vsa_bounds/`) discovers and emits:
- A **candidate theorem** (Declared): a closed-form `required_dim_multihop(F, k, h, delta)`
  expressed via an effective-m model.
- **SMT-LIB 2 (`.smt2`)** obligations: the concrete arithmetic a solver discharges.
- **Liquid Haskell (`.hs`)** skeletons: axiomatize the candidate theorem and ask LH/Z3 to
  discharge the concrete instantiation — mirroring `proofs/lh-bundle/src/Bundle.hs` exactly.

---

## Candidate theorem (Declared)

**Effective-m model (FLAG — open modeling choice):**

The experiment tests three parametric hypotheses for the effective bundle size in
multi-hop compositions:

| Model | m\_eff (bind\_chain / nested\_unbind) | m\_eff (bundle\_of\_binds) | Character |
|---|---|---|---|
| A\_exponential | `F * k^h` | `h * k` | Conservative worst-case per hop |
| B\_linear | `F * k * h` | `h * k` | Linear growth per hop |
| C\_sqrt | `ceil(F * k * sqrt(h))` | `ceil(sqrt(h) * k)` | Sub-linear (optimistic) |

The **candidate theorem** (for the best empirically validated model) is:

```
For a multi-hop VSA composition (bind_chain / bundle_of_binds / nested_unbind)
with parameters (F, k, h, delta):

  required_dim_multihop(F, k, h, delta) = ceil((2 / mu^2) * ln(m_eff(F, k, h) / delta))
                                        = ceil(200 * ln(m_eff / delta))   [mu = 0.1]

If d >= required_dim_multihop(F, k, h, delta), then the membership-decode failure
probability is at most delta.
```

**Status: Declared.** This is a hypothesis awaiting:
1. Empirical validation that it upper-bounds measured failure rates (done by the experiment).
2. A proof assistant discharging the concrete arithmetic instantiation (the `.smt2`/`.hs` files).
3. Formal establishment of the underlying theorem connecting `m_eff` to the multi-hop
   failure probability (OQ-A / OQ-F — the key open question).

---

## Proof strategy (mirrors M-001 / lh-bundle)

The strategy is the **axiomatize + checked-instantiation** pattern from ADR-010:

1. **Axiomatize** the candidate capacity theorem: `assume candidateCapacityThm` in LH
   (the theorem's truth is asserted, not proven by the type system).
2. **Discharge the concrete arithmetic**: for each swept in-regime point, assert
   `d >= requiredDimMultihop(m_eff)` and have Z3 confirm it — purely integer arithmetic.
3. **The open question** is step 1: the axiom must be formally proven (Lean 4 / LH / Agda)
   or cited to a published theorem that covers the multi-hop case.

This is the **same pattern** as the single-hop M-001 probe:
- `proofs/lh-bundle/src/Bundle.hs` axiomatizes `capacityThm` (Clarkson/Thomas, cited)
  and has Z3 discharge `d >= requiredDim(m)` for concrete `(m, d)` pairs.
- The files here do the same for `candidateCapacityThm` (multi-hop, pending citation).

---

## Directory layout

```
proofs/vsa-multihop-bound/
  README.md                          -- this file (Declared scaffold)
  cabal.project                      -- Cabal project for LH probes (stub)
  vsa-multihop-bound.cabal           -- Cabal package manifest (stub)
  src/
    MultihopBound_bindchain_A_exponential.hs   -- LH skeleton: bind_chain, Model A
    MultihopBound_bundleofbinds_A_exponential.hs
    MultihopBound_nestedunbind_A_exponential.hs
  *.smt2                             -- SMT-LIB 2 obligations (populated by --proof run)
  PROOF-SUMMARY.md                   -- discovery run summary (populated by --proof run)
```

The `.smt2` and `.hs` files in `src/` are **populated by the GPU experiment**
(`python -m mycelium_experiments.vsa_bounds --proof --emit-obligations`).
The stub files here are the initial scaffolding; the experiment overwrites them.

---

## How to run

### Step 1: Run the experiment to discover candidate bounds and emit obligations

```bash
cd experiments
# Quick CPU profile (for testing):
python -m mycelium_experiments.vsa_bounds --proof --quick

# Full GPU sweep (for real results):
python -m mycelium_experiments.vsa_bounds --proof --sweep multihop
```

Results land in `experiments/results/` and are copied to `proofs/vsa-multihop-bound/`.

### Step 2: Discharge the SMT-LIB obligations

```bash
# From repo root:
z3 -smt2 proofs/vsa-multihop-bound/<run-id>-multihop-bind_chain-A_exponential.smt2
# Expected output: sat  (all probes pass)
```

If `sat`: the concrete arithmetic is machine-confirmed for the swept points.

### Step 3: Discharge the Liquid Haskell skeleton

```bash
# Requires: GHC 9.8.2 + cabal + LiquidHaskell 0.9.8.2 + Z3 (see proofs/lh-bundle/README.md)
export LC_ALL=C.UTF-8
cd proofs/vsa-multihop-bound
cabal build
# Expected: LIQUID: SAFE (N constraints checked)
```

If SAFE: Z3 confirms the arithmetic. This is the same confirmation as M-001.

### Step 4: Establish the underlying theorem (OQ-A / OQ-F)

The `assume candidateCapacityThm` axiom in the LH skeleton is the **key open question**
(OQ-F). It requires either:
- A formal proof in Lean 4 / LH / Agda connecting `m_eff` to the multi-hop
  confusion probability (OQ-A, M-827).
- A citation to a published theorem that covers the multi-hop case.

Only after steps 2, 3, AND 4 does the `Proven` tag become warranted
for the checked subset (maintainer's prerogative, VR-5).

---

## What is NOT claimed here

- The files in this directory are NOT proofs. They are **proof obligations** (stubs).
- The candidate theorem is **Declared** — a hypothesis, not a theorem.
- No `Proven` tag is assigned anywhere in this directory or the experiment code.
- The effective-m model is an **open modeling choice** — the experiment tests three
  hypotheses but cannot determine which (if any) is theoretically correct.

---

## Changelog

- **2026-06-29:** Scaffold created (M-832 OQ-F extension). Candidate theorem framed,
  LH skeleton structure defined, SMT-LIB strategy documented. Status: Declared.
  Proof obligations emitted by `experiments/mycelium_experiments/vsa_bounds/`
  `proof_obligation.py` (run with `--proof` to populate).
