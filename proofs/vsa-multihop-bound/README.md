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
(see `proofs/lh-bundle/` -- M-001, discharged 2026-06-09).

**OQ-F** asks: does this bound extend to multi-hop VSA compositions
(bind-chains, bundle-of-binds, nested unbind)?  The answer is unknown -- it is the
core open question this directory addresses.

This directory holds the **bridge artifacts** that the GPU experiment
(`experiments/mycelium_experiments/vsa_bounds/`) discovers and emits:

- A **candidate theorem** (Declared): a closed-form `required_dim_multihop(F, k, h, delta)`
  expressed via an effective-m model, validated comparatively across all three models.
- **SMT-LIB 2 (`.smt2`)** obligations: the concrete arithmetic a solver discharges.
- **Liquid Haskell (`.hs`)** skeletons: axiomatize the candidate theorem and ask LH/Z3 to
  discharge the concrete instantiation -- mirroring `proofs/lh-bundle/src/Bundle.hs` exactly.
- **Lean 4 (`.lean`)** skeletons: mirror the LH structure with `axiom candidateCapacityThm`
  and `native_decide` probes -- the path to the OQ-A/M-827 Lean 4 mechanization.

---

## Candidate theorem (Declared)

**Effective-m model (FLAG -- open modeling choice):**

The experiment tests three parametric hypotheses for the effective bundle size in
multi-hop compositions, all tested comparatively per composition in one run:

| Model | m\_eff (bind\_chain / nested\_unbind) | m\_eff (bundle\_of\_binds) | Character |
|---|---|---|---|
| A\_exponential | `F * k^h` | `h * k` | Conservative worst-case per hop |
| B\_linear | `F * k * h` | `h * k` | Linear growth per hop |
| C\_sqrt | `ceil(F * k * sqrt(h))` | `ceil(sqrt(h) * k)` | Sub-linear (optimistic) |

The **candidate theorem** (for a given model) is:

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
2. A proof assistant discharging the concrete arithmetic instantiation (the `.smt2`/`.hs`/`.lean`).
3. Formal establishment of the underlying theorem connecting `m_eff` to the multi-hop
   failure probability (OQ-A / OQ-F -- the key open question).

The PROOF-SUMMARY.md (emitted by the experiment) includes a **comparative ranking per
composition**: for each composition, which model is the tightest valid upper bound (not
refuted, most in-regime points, highest safety margin). Refuted models are always listed
-- never silently dropped (G2 / never-silent).

---

## Proof strategy (mirrors M-001 / lh-bundle)

The strategy is the **axiomatize + checked-instantiation** pattern from ADR-010:

1. **Axiomatize** the candidate capacity theorem:
   - LH: `assume candidateCapacityThm` in the Haskell skeleton.
   - Lean 4: `axiom candidateCapacityThm` in the Lean skeleton.
   The theorem's truth is asserted, not proven by the type system.
2. **Discharge the concrete arithmetic** via the **refutation pattern** (SMT) or
   **`native_decide`** (Lean 4) / Z3 (LH): for each swept in-regime point, confirm
   `d >= requiredDimMultihop(m_eff)` for the concrete arguments.
   This mirrors `proofs/binary-ternary-roundtrip/roundtrip_8x6.smt2`.
3. **The open question** is step 1: the axiom must be formally proven (Lean 4 / LH / Agda)
   or cited to a published theorem that covers the multi-hop case (OQ-A / M-827).

Both assistants (LH and Lean 4) follow the same pattern -- axiomatize, then discharge
arithmetic -- and both rely on the same open axiom. Neither proof of the arithmetic
substitutes for the formal proof of the axiom.

---

## Directory layout

```
proofs/vsa-multihop-bound/
  README.md                            -- this file (Declared scaffold)
  .gitignore                           -- ignores timestamped run-outputs below
  cabal.project                        -- Cabal project for LH probes (Liquid Haskell)
  vsa-multihop-bound.cabal             -- Cabal package manifest (LH)
  src/
    MultihopBound_bindchain_Aexponential.hs   -- representative LH skeleton (bind_chain, Model A)
  lean/
    lean-toolchain                     -- Lean 4 toolchain pin (leanprover/lean4:v4.15.0)
    lakefile.toml                      -- Lake build manifest for the Lean 4 probes
    VsaMultihopBound.lean              -- top-level module (imports all sub-modules)
    VsaMultihopBound/
      BindChainAexponential.lean       -- representative Lean 4 skeleton (bind_chain, Model A)
  EXAMPLE-bind_chain-A_exponential.smt2    -- force-added CPU-run example (git add -f)
  EXAMPLE-bind_chain-A_exponential.hs      -- force-added CPU-run example (git add -f)
  EXAMPLE-bind_chain-A_exponential.lean    -- force-added CPU-run example (git add -f)
  EXAMPLE-PROOF-SUMMARY.md                -- force-added CPU-run example (git add -f)
  <run-id>-multihop-<comp>-<model>.smt2   -- emitted SMT-LIB obligation (gitignored)
  <run-id>-multihop-<comp>-<model>.hs     -- emitted LH probe (gitignored)
  <run-id>-multihop-<comp>-<model>.lean   -- emitted Lean 4 probe (gitignored)
  <run-id>-PROOF-SUMMARY.md               -- discovery-run summary (gitignored)
```

The committed scaffold is `README.md`, the Cabal stubs, the `src/` LH skeleton, and the
`lean/` Lean 4 skeleton. The `EXAMPLE-*` artifacts (force-added via `git add -f`) are
representative populated obligations from a real CPU run, committed so a reviewer can
inspect them without running anything. The timestamped `<run-id>-...` obligations are
**run-outputs** (gitignored, regenerated per sweep).

---

## How to run

### Step 1: Run the experiment to discover candidate bounds and emit obligations

```bash
cd experiments

# Demo / CPU profile (designed to produce in-regime obligations without GPU):
python -m mycelium_experiments.vsa_bounds --demo --numpy-only --no-plots

# Quick CPU profile (smaller d, fewer trials -- may not produce in-regime points):
python -m mycelium_experiments.vsa_bounds --proof --quick --numpy-only

# Full GPU sweep (for real results):
python -m mycelium_experiments.vsa_bounds --proof --sweep multihop
```

The `--demo` profile uses `d=[1024,2048,4096,8192]`, `F=[2]`, `k=[4]`, `h=[1,2]`,
`trials=50` -- designed so that at `d=8192`, Models A/B/C are all in-regime for `h=1`
(m\_eff=8, required\_dim(8,0.02)~=1382 << 8192). Guarantee: Empirical.

Results land in `experiments/results/` and the obligations are copied to
`proofs/vsa-multihop-bound/` (gitignored run-outputs). The `--demo` run produces
obligations for **all three models x all three compositions** simultaneously, plus a
PROOF-SUMMARY.md with comparative ranking per composition.

### Step 2: Discharge the SMT-LIB obligations

```bash
# From repo root:
z3 -smt2 proofs/vsa-multihop-bound/EXAMPLE-bind_chain-A_exponential.smt2
# Expected output per probe: unsat  (refutation pattern -- negation is unsatisfiable)
```

Each probe uses the **refutation pattern**: it asserts `(not (>= d req_dim))` and expects
`unsat`. `unsat` means no counterexample exists -- the candidate holds for that swept point.
`sat` means a refuting instance was found and the candidate must be investigated.

If all probes return `unsat`: the concrete arithmetic is machine-confirmed for the swept points.

### Step 3: Discharge the Liquid Haskell skeleton

```bash
# Requires: GHC 9.8.2 + cabal + LiquidHaskell 0.9.8.2 + Z3 (see proofs/lh-bundle/README.md)
export LC_ALL=C.UTF-8
cd proofs/vsa-multihop-bound
cabal build
# Expected: LIQUID: SAFE (N constraints checked)
```

If SAFE: Z3 confirms the arithmetic. This is the same confirmation as M-001.

### Step 4: Discharge the Lean 4 skeleton

```bash
# Requires: Lean 4 v4.15.0 via elan
# Install elan: https://github.com/leanprover/elan
elan install leanprover/lean4:v4.15.0
cd proofs/vsa-multihop-bound/lean
lake build
# Expected: build succeeds (all native_decide probes kernel-checked)
```

`native_decide` is kernel-checked (sound): build success means Lean's kernel verified
`d >= requiredDimMultihop m_eff` for each concrete probe. This serves the OQ-A/M-827
Lean 4 mechanization path (see `research/26-dn64-typesystem-rnd-RECORD.md`).

### Step 5: Establish the underlying theorem (OQ-A / OQ-F)

The `assume candidateCapacityThm` axiom (LH) and `axiom candidateCapacityThm` (Lean 4)
are the **key open question** (OQ-F). They require either:

- A formal proof in Lean 4 / LH / Agda connecting `m_eff` to the multi-hop
  confusion probability (OQ-A, M-827).
- A citation to a published theorem that covers the multi-hop case.

Only after steps 2, 3, 4, AND 5 does the `Proven` tag become warranted
for the checked subset (maintainer's prerogative, VR-5).

---

## EXAMPLE artifacts

The `EXAMPLE-*` files are populated obligations from a CPU-feasible demo run, committed
via `git add -f` so a reviewer can inspect them without running anything:

- `EXAMPLE-bind_chain-A_exponential.smt2` -- SMT-LIB 2 obligation (bind\_chain, Model A)
- `EXAMPLE-bind_chain-A_exponential.hs` -- Liquid Haskell skeleton (bind\_chain, Model A)
- `EXAMPLE-bind_chain-A_exponential.lean` -- Lean 4 skeleton (bind\_chain, Model A)
- `EXAMPLE-PROOF-SUMMARY.md` -- comparative PROOF-SUMMARY from the demo run

These are illustrative CPU-run examples; a GPU run with `--proof --sweep multihop`
regenerates richer (timestamped, gitignored) versions with more in-regime points.

---

## What is NOT claimed here

- The files in this directory are NOT proofs. They are **proof obligations** (stubs).
- The candidate theorem is **Declared** -- a hypothesis, not a theorem.
- No `Proven` tag is assigned anywhere in this directory or the experiment code.
- The effective-m model is an **open modeling choice** -- the experiment tests all three
  hypotheses (A/B/C) comparatively but cannot determine which (if any) is theoretically
  correct.
- Both proof assistants (LH and Lean 4) axiomatize the candidate theorem; neither proves
  it. `native_decide` (Lean) and Z3 (LH/SMT) discharge ONLY the concrete arithmetic.

---

## Changelog

- **2026-06-29 (v2):** Extended with Lean 4 scaffold (`lean/` subdir: `lakefile.toml`,
  `lean-toolchain`, `VsaMultihopBound/BindChainAexponential.lean`). Updated
  `emit_obligations()` to emit all three models x all three compositions x three formats
  (SMT2, LH, Lean). Added comparative ranking per composition to PROOF-SUMMARY.md.
  Added `--demo` profile for CPU-feasible in-regime obligation generation.
  Committed EXAMPLE artifacts. Status: Declared.
- **2026-06-29 (v1):** Scaffold created (M-832 OQ-F extension). Candidate theorem framed,
  LH skeleton structure defined, SMT-LIB strategy documented. Status: Declared.
  Proof obligations emitted by `experiments/mycelium_experiments/vsa_bounds/`
  `proof_obligation.py` (run with `--proof` to populate).
