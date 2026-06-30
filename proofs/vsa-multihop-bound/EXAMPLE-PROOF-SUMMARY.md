# VSA Multi-Hop Bound — PROOF-SUMMARY

Run: `20260629T174620Z` | Backend: `numpy-cpu`

> **Guarantee: Declared**  This summary presents candidate theorems and emitted proof obligations.  It is NOT a proof.  The `Proven` verdict requires a proof assistant (LH/Z3, SMT solver, or Lean 4) to discharge the obligations AND the underlying theorem to be formally established or cited.  See VR-5 / ADR-032.

## Best candidate

- Effective-m model: `A_exponential`
- Status: REFUTED at 3 points — see details below
- Description: `m_eff = F * k^h (bind_chain/nested_unbind) or h*k (bundle_of_binds); exponential worst-case per hop.  Same as sweeps.py::_naive_extrapolated_m.`
- Goodness of fit: REFUTED at 3/24 points — candidate is NOT a universal upper bound over the swept regime.  See refuted_points for details.
- Empirically validated regime: compositions=['bind_chain', 'bundle_of_binds', 'nested_unbind']; F=[2]; k=[4]; h=[1, 2]; d in [2048, 8192]

## Candidate theorem (Declared)

Let `m_eff(F, k, h)` be as defined by the best candidate model.  Then:

```
required_dim_multihop(F, k, h, delta)
  = ceil((2 / mu^2) * ln(m_eff(F, k, h) / delta))
  = ceil(200 * ln(m_eff(F, k, h) / delta))   [mu = 0.1]
```

For a multi-hop composition with parameters (F, k, h, delta), if `d >= required_dim_multihop(...)`, then the membership-decode failure probability is at most `delta`.

> **Status: Declared.**  This is a hypothesis — not a proven theorem.  The single-hop basis is the Clarkson-Ubaru-Yang 2023 Thm 6 / Thomas-Dasgupta-Rosing 2021 result.  Extending it to multi-hop requires a new theorem or reduction (OQ-F / OQ-A).

## Comparative ranking per composition

> **Guarantee: Empirical + Declared.**  Rankings are based on empirical sweep data.  A non-refuted model is Empirical evidence of validity; it is NOT Proven.  Refuted models are listed explicitly (never-silent, G2).

### bind_chain

| Rank | Model | Valid UB? | In-regime | Refuted | Min margin |
|---|---|---|---|---|---|
| 1 | `A_exponential` | YES (not refuted) | 6 | 0 | 0.0200 |
| 2 | `B_linear` | YES (not refuted) | 6 | 0 | 0.0200 |
| 3 | `C_sqrt` | YES (not refuted) | 6 | 0 | 0.0200 |

**Tightest valid upper bound:** `A_exponential` (6 in-regime points, min margin 0.0200

### bundle_of_binds

| Rank | Model | Valid UB? | In-regime | Refuted | Min margin |
|---|---|---|---|---|---|
| 1 | `A_exponential` | YES (not refuted) | 6 | 0 | 0.0200 |
| 2 | `B_linear` | YES (not refuted) | 6 | 0 | 0.0200 |
| 3 | `C_sqrt` | YES (not refuted) | 6 | 0 | 0.0200 |

**Tightest valid upper bound:** `A_exponential` (6 in-regime points, min margin 0.0200

### nested_unbind

| Rank | Model | Valid UB? | In-regime | Refuted | Min margin |
|---|---|---|---|---|---|
| 1 | `A_exponential` | REFUTED (3 pts) | 6 | 3 | 0.0200 |
| 2 | `B_linear` | REFUTED (3 pts) | 6 | 3 | 0.0200 |
| 3 | `C_sqrt` | REFUTED (3 pts) | 6 | 3 | 0.0200 |

**No valid upper bound found** for this composition in the swept regime.
All models are refuted or have no in-regime points.  This is NOT a proof of invalidity — only a signal that the sweep did not find a valid regime; try larger d or different parameters.

## All candidate models (aggregate across all compositions)

| Model | Status | In-regime | Refuted | Min margin |
|---|---|---|---|---|
| `A_exponential` | REFUTED (3) | 18/24 | 3 | 0.0200 |
| `B_linear` | REFUTED (3) | 18/24 | 3 | 0.0200 |
| `C_sqrt` | REFUTED (3) | 18/24 | 3 | 0.0200 |

## Emitted proof obligations

The following files are the checkable artifacts.  Each follows the `axiomatize + checked-instantiation` pattern from `proofs/lh-bundle/` and `proofs/binary-ternary-roundtrip/`.

### SMT-LIB 2 obligations (`.smt2`)

Run with: `z3 -smt2 <file>` — expect `unsat` for every probe (refutation pattern: asserts NOT property; `unsat` = no counterexample found = candidate holds for all swept in-regime points).

- `20260629T174620Z-multihop-bind_chain-A_exponential.smt2` — bind_chain_A_exponential_smt2
- `20260629T174620Z-multihop-bind_chain-B_linear.smt2` — bind_chain_B_linear_smt2
- `20260629T174620Z-multihop-bind_chain-C_sqrt.smt2` — bind_chain_C_sqrt_smt2
- `20260629T174620Z-multihop-bundle_of_binds-A_exponential.smt2` — bundle_of_binds_A_exponential_smt2
- `20260629T174620Z-multihop-bundle_of_binds-B_linear.smt2` — bundle_of_binds_B_linear_smt2
- `20260629T174620Z-multihop-bundle_of_binds-C_sqrt.smt2` — bundle_of_binds_C_sqrt_smt2
- `20260629T174620Z-multihop-nested_unbind-A_exponential.smt2` — nested_unbind_A_exponential_smt2
- `20260629T174620Z-multihop-nested_unbind-B_linear.smt2` — nested_unbind_B_linear_smt2
- `20260629T174620Z-multihop-nested_unbind-C_sqrt.smt2` — nested_unbind_C_sqrt_smt2

### Liquid Haskell skeletons (`.hs`)

Run with: `cd proofs/vsa-multihop-bound && cabal build` — expect `LIQUID: SAFE (N constraints checked)`.

- `20260629T174620Z-multihop-bind_chain-A_exponential.hs` — bind_chain_A_exponential_lh
- `20260629T174620Z-multihop-bind_chain-B_linear.hs` — bind_chain_B_linear_lh
- `20260629T174620Z-multihop-bind_chain-C_sqrt.hs` — bind_chain_C_sqrt_lh
- `20260629T174620Z-multihop-bundle_of_binds-A_exponential.hs` — bundle_of_binds_A_exponential_lh
- `20260629T174620Z-multihop-bundle_of_binds-B_linear.hs` — bundle_of_binds_B_linear_lh
- `20260629T174620Z-multihop-bundle_of_binds-C_sqrt.hs` — bundle_of_binds_C_sqrt_lh
- `20260629T174620Z-multihop-nested_unbind-A_exponential.hs` — nested_unbind_A_exponential_lh
- `20260629T174620Z-multihop-nested_unbind-B_linear.hs` — nested_unbind_B_linear_lh
- `20260629T174620Z-multihop-nested_unbind-C_sqrt.hs` — nested_unbind_C_sqrt_lh

### Lean 4 skeletons (`.lean`)

Run with: `cd proofs/vsa-multihop-bound/lean && lake build` — expect build success (all `native_decide` / `decide` probes check out).

- `20260629T174620Z-multihop-bind_chain-A_exponential.lean` — bind_chain_A_exponential_lean
- `20260629T174620Z-multihop-bind_chain-B_linear.lean` — bind_chain_B_linear_lean
- `20260629T174620Z-multihop-bind_chain-C_sqrt.lean` — bind_chain_C_sqrt_lean
- `20260629T174620Z-multihop-bundle_of_binds-A_exponential.lean` — bundle_of_binds_A_exponential_lean
- `20260629T174620Z-multihop-bundle_of_binds-B_linear.lean` — bundle_of_binds_B_linear_lean
- `20260629T174620Z-multihop-bundle_of_binds-C_sqrt.lean` — bundle_of_binds_C_sqrt_lean
- `20260629T174620Z-multihop-nested_unbind-A_exponential.lean` — nested_unbind_A_exponential_lean
- `20260629T174620Z-multihop-nested_unbind-B_linear.lean` — nested_unbind_B_linear_lean
- `20260629T174620Z-multihop-nested_unbind-C_sqrt.lean` — nested_unbind_C_sqrt_lean

## Next steps for the maintainer

1. **Run the SMT-LIB obligations:** `z3 -smt2 <file>.smt2` for each emitted file.
   Uses the **refutation pattern**: each probe asserts `(not (>= d req_dim))` and expects `unsat` — meaning no counterexample exists and the candidate holds.
   - If `unsat`: the concrete arithmetic instantiation is machine-confirmed for the swept points (no refuting counterexample found).
   - If `sat`: a specific point refutes the candidate — investigate.
2. **Run the LH skeleton:** `cabal build` in `proofs/vsa-multihop-bound/`.
   - If `LIQUID: SAFE`: Z3 confirms the arithmetic. This is the same confirmation as the single-hop M-001 probe.
3. **Run the Lean 4 skeleton:** `lake build` in `proofs/vsa-multihop-bound/lean/`.
   - Build success means `native_decide` confirmed the arithmetic instantiation is kernel-checked (sound).  Serves the OQ-A / M-827 Lean 4 mechanization path.
4. **Establish the underlying theorem** (OQ-A / OQ-F): the `assume candidateCapacityThm` axiom (LH) and `axiom candidateCapacityThm` (Lean 4) are the key open questions.  They require either a formal proof or a citation to a published theorem that covers the multi-hop case.
5. **Stamp Proven** (maintainer prerogative, VR-5): only after BOTH (a) solver confirms SAFE/unsat AND (b) the axiom is formally established does the `Proven` tag become warranted for the checked subset.

> FLAG (open modeling choice): The effective-m model is the key hypothesis.  The best empirical model is `A_exponential` but this may not be the correct theoretical model.  The maintainer must confirm which model to pursue for the formal proof.

---

*Generated by `experiments/mycelium_experiments/vsa_bounds/proof_obligation.py` (M-832, OQ-F).  Guarantee: Declared throughout.*
