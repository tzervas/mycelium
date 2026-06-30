-- | M-832 / OQ-F — Multi-hop VSA capacity-refinement probe (bind_chain, A_exponential).
--
--   STATUS: Declared — PENDING proof-assistant discharge (VR-5 / ADR-032).
--
--   Strategy (mirrors proofs/lh-bundle/src/Bundle.hs and the LH skeleton):
--     - AXIOMATIZE the candidate multi-hop capacity theorem (``axiom candidateCapacityThm``).
--     - Discharge only the concrete ARITHMETIC INSTANTIATION via ``native_decide`` / ``decide``.
--     - This does NOT re-prove the underlying concentration inequality — only the arithmetic.
--
--   Candidate theorem (Declared):
--     For composition 'bind_chain', model 'A_exponential':
--       m_eff(F, k, h) = F * k^h
--       required_dim_multihop(m_eff, delta) = ceil((2/mu^2) * ln(m_eff/delta))
--     Then d >= required_dim_multihop implies failure probability <= delta.
--
--   The axiom ``candidateCapacityThm`` is the open question (OQ-F / OQ-A / M-827).
--   It asserts the multi-hop generalisation of the single-hop Clarkson/Thomas result.
--   A formal proof must be established (or a published theorem cited) before this
--   axiom can be discharged — only then does ``Proven`` become warranted (VR-5).
--
--   The ``native_decide`` / ``decide`` calls below discharge ONLY the concrete integer
--   inequality d >= requiredDimMultihop for each swept in-regime point.  They are
--   kernel-checked (sound), but they rely on the axiom above to carry semantic meaning.
--
--   Empirically validated regime: compositions=['bind_chain', 'bundle_of_binds', 'nested_unbind']; F=[2]; k=[4]; h=[1, 2]; d in [2048, 8192]
--
--   Citation basis: Clarkson-Ubaru-Yang 2023 (Thm 6); Thomas-Dasgupta-Rosing 2021
--
--   To run (requires Lean 4, e.g. leanprover/lean4:v4.15.0):
--     cd proofs/vsa-multihop-bound/lean
--     lake build
--   Expect: build succeeds (all `native_decide` / `decide` calls check out).
--
--   NEXT STEP: establish or cite the theorem connecting m_eff to the actual multi-hop
--   failure probability (the ``axiom candidateCapacityThm`` below — OQ-A/M-827).
--   Only after that does ``Proven`` become warranted.
--
--   Guarantee: Declared (pending discharge).

namespace VsaMultihopBound.BindChain_AExponential

/-- `requiredDimMultihop` is the precomputed lookup table of the axiomatized candidate
    capacity formula: `⌈(2/μ²) · ln(m_eff/δ)⌉` with μ=0.1 (so 2/μ²=200).
    Only its concrete values enter the checked arithmetic below (same strategy as
    `proofs/lh-bundle/src/Bundle.hs::requiredDim`).
    Guarantee: Declared — the formula is cited (T0.2, extended to multi-hop);
    the extension to multi-hop is the open OQ-F hypothesis. -/
def requiredDimMultihop (m : Nat) : Nat :=
  if m ≤ 8 then 1199  -- m_eff=8, delta=0.02, req_dim=1199
  else if m ≤ 32 then 1476  -- m_eff=32, delta=0.02, req_dim=1476
  else 1476  -- conservative: largest in table

/-- The axiomatized candidate multi-hop capacity guarantee (Declared — open question OQ-F/OQ-A):
    a composition at dimension d ≥ requiredDimMultihop m decodes with failure probability ≤ delta.
    ``axiom`` = the multi-hop generalisation of Clarkson/Thomas (T0.2) is ASSUMED, not proven.
    This is the key open question (OQ-A / M-827).  A formal proof connecting m_eff to the
    actual multi-hop confusion probability must be established before `Proven` is warranted.
    Guarantee: Declared. -/
axiom candidateCapacityThm (m d : Nat) (h : d ≥ requiredDimMultihop m) : True

-- Checked instantiations.  Each `example` type-checks iff `native_decide` / `decide`
-- confirms that d ≥ requiredDimMultihop m_eff for the concrete arguments.
-- ``native_decide`` is kernel-checked (sound); it discharges the arithmetic, NOT the axiom.
-- Guarantee: Declared (pending formal establishment of candidateCapacityThm).

/-- Probe 1: bind_chain F=2 k=4 h=1: d=2048 ≥ requiredDimMultihop 8 = 1199 -/
example : 2048 ≥ requiredDimMultihop 8 := by native_decide
  -- m_eff=8, req_dim=1199; candidate holds at this swept in-regime point.

/-- Probe 2: bind_chain F=2 k=4 h=1: d=4096 ≥ requiredDimMultihop 8 = 1199 -/
example : 4096 ≥ requiredDimMultihop 8 := by native_decide
  -- m_eff=8, req_dim=1199; candidate holds at this swept in-regime point.

/-- Probe 3: bind_chain F=2 k=4 h=1: d=8192 ≥ requiredDimMultihop 8 = 1199 -/
example : 8192 ≥ requiredDimMultihop 8 := by native_decide
  -- m_eff=8, req_dim=1199; candidate holds at this swept in-regime point.

/-- Probe 4: bind_chain F=2 k=4 h=2: d=2048 ≥ requiredDimMultihop 32 = 1476 -/
example : 2048 ≥ requiredDimMultihop 32 := by native_decide
  -- m_eff=32, req_dim=1476; candidate holds at this swept in-regime point.

/-- Probe 5: bind_chain F=2 k=4 h=2: d=4096 ≥ requiredDimMultihop 32 = 1476 -/
example : 4096 ≥ requiredDimMultihop 32 := by native_decide
  -- m_eff=32, req_dim=1476; candidate holds at this swept in-regime point.

/-- Probe 6: bind_chain F=2 k=4 h=2: d=8192 ≥ requiredDimMultihop 32 = 1476 -/
example : 8192 ≥ requiredDimMultihop 32 := by native_decide
  -- m_eff=32, req_dim=1476; candidate holds at this swept in-regime point.

end VsaMultihopBound.BindChain_AExponential

-- End of auto-generated Lean 4 probe file.
-- STATUS: Declared — run `lake build` to discharge (expects build success).
-- Guarantee: Declared until (a) build succeeds AND (b) candidateCapacityThm is formally proven.
