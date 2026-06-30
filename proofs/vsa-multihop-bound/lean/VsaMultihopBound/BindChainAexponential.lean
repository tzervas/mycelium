-- | M-832 / OQ-F — Multi-hop VSA capacity-refinement probe (bind_chain, A_exponential).
--
--   STATUS: Declared — STUB awaiting experiment output (VR-5 / ADR-032).
--
--   This file is a PLACEHOLDER.  Populate by running:
--     cd experiments
--     python -m mycelium_experiments.vsa_bounds --demo --numpy-only --no-plots
--   or:
--     python -m mycelium_experiments.vsa_bounds --proof --emit-obligations
--
--   The experiment will emit a populated Lean 4 file at:
--     proofs/vsa-multihop-bound/lean/<run-id>-multihop-bind_chain-A_exponential.lean
--
--   Strategy (mirrors proofs/lh-bundle/src/Bundle.hs):
--     - AXIOMATIZE the candidate capacity theorem (``axiom candidateCapacityThm``).
--     - Have Lean 4 / `native_decide` discharge the concrete arithmetic:
--       d >= requiredDimMultihop m_eff for each swept in-regime point.
--     - m_eff = F * k^h  (Model A_exponential, bind_chain composition).
--
--   The axiom `candidateCapacityThm` is the open question (OQ-F / OQ-A / M-827):
--   a formal theorem connecting m_eff = F * k^h to the multi-hop failure probability
--   must be established (or a published theorem cited) before `Proven` is warranted.
--
--   `native_decide` is kernel-checked (sound) and discharges ONLY the concrete integer
--   inequality d >= requiredDimMultihop m_eff.  It relies on the axiom above for meaning.
--
--   Guarantee: Declared (stub; no probes populated yet).

namespace VsaMultihopBound.BindChain_AExponential

/-- `requiredDimMultihop` is the axiomatized lookup table (stub; populated by experiment).
    Formula: ceil(200 * ln(m_eff / delta)), with m_eff = F * k^h (Model A_exponential).
    Guarantee: Declared — the formula is cited (T0.2, extended to multi-hop);
    the extension to multi-hop is the open OQ-F hypothesis. -/
def requiredDimMultihop (m : Nat) : Nat :=
  if m ≤ 1 then 0      -- degenerate stub
  else 9999999          -- sentinel: experiment will fill concrete values

/-- The axiomatized candidate multi-hop capacity guarantee (Declared — open question OQ-F/OQ-A):
    a composition at dimension d ≥ requiredDimMultihop m decodes with failure probability ≤ delta.
    ``axiom`` = the multi-hop generalisation of Clarkson/Thomas (T0.2) is ASSUMED, not proven.
    This is the key open question (OQ-A / M-827).  A formal proof must be established
    before `Proven` is warranted.
    Guarantee: Declared. -/
axiom candidateCapacityThm (m d : Nat) (h : d ≥ requiredDimMultihop m) : True

-- Probes (populated by experiment run):
-- example : <d> ≥ requiredDimMultihop <m_eff> := by native_decide
--   -- bind_chain F=... k=... h=...

-- STATUS: Declared (stub). Run the experiment with --proof to populate.

end VsaMultihopBound.BindChain_AExponential
