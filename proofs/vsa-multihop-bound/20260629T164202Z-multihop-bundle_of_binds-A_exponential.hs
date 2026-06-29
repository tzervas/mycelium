{-@ LIQUID "--reflection" @-}
{-@ LIQUID "--ple"        @-}

-- | M-832 / OQ-F — Multi-hop VSA capacity-refinement probe (bundle_of_binds, A_exponential).
--
--   STATUS: Declared — PENDING proof-assistant discharge (VR-5 / ADR-032).
--
--   Strategy (mirrors proofs/lh-bundle/src/Bundle.hs):
--     - AXIOMATIZE the candidate multi-hop capacity theorem's statement.
--     - Have Z3 discharge only the concrete ARITHMETIC INSTANTIATION — never
--       re-prove the concentration inequality.
--
--   Candidate theorem (Declared):
--     For composition 'bundle_of_binds', model 'A_exponential':
--       m_eff(F, k, h) = h * k
--       required_dim_multihop(m_eff, delta) = ceil((2/mu^2) * ln(m_eff/delta))
--     Then d >= required_dim_multihop implies failure probability <= delta.
--
--   Empirically validated regime: no in-regime points found
--
--   Citation basis: Clarkson-Ubaru-Yang 2023 (Thm 6); Thomas-Dasgupta-Rosing 2021
--
--   To run (once GHC + LiquidHaskell + Z3 are available):
--     cd proofs/vsa-multihop-bound
--     cabal build   -- expects: LIQUID: SAFE (N constraints checked)
--   If SAFE: the arithmetic instantiation is machine-confirmed for the swept points.
--
--   NEXT STEP: establish the theorem connecting m_eff to the actual multi-hop confusion
--   probability (the ``assume candidateCapacityThm`` below — axiom awaiting formal proof
--   or citation to a published theorem).  Only after that does Proven become warranted.
--
--   Guarantee: Declared (pending discharge).
module MultihopBound_bundleofbinds_Aexponential where

-- | @requiredDimMultihop m@ is the axiomatized right-hand side of the candidate capacity
--   theorem: @ceil( (2/mu^2) * ln(m_eff/delta) )@ with mu = 0.1 (so 2/mu^2 = 200).
--   The formula is cited (T0.2, extended to multi-hop via the m_eff model); only its
--   concrete values enter the checked arithmetic below.
--
--   This is a LOOKUP TABLE of precomputed values for the swept (m_eff, delta) pairs —
--   exactly the same strategy as proofs/lh-bundle/src/Bundle.hs::requiredDim.
{-@ reflect requiredDimMultihop @-}
requiredDimMultihop :: Int -> Int
requiredDimMultihop m
  | otherwise = 9999999  -- No in-regime points: conservative sentinel

-- | A bundle is well-capacitied at dimension d when d meets the candidate threshold.
{{-@ type WellCapacitied M = {{d:Int | d >= requiredDimMultihop M}} @-}}

-- | The axiomatized candidate multi-hop capacity guarantee:
--   a well-capacitied composition decodes with failure probability at most delta.
--   ``assume`` = the cited theorem (T0.2, extended) is taken as given;
--   LiquidHaskell does not re-prove it — only the concrete arithmetic is discharged.
--
--   NOTE: This is the key OPEN QUESTION (OQ-F / OQ-A).  The axiom below is a
--   HYPOTHESIS — a formal theorem connecting m_eff to the multi-hop failure probability
--   must be established before this can be treated as anything other than Declared.
{{-@ assume candidateCapacityThm :: m:Int -> WellCapacitied m -> {{b:Bool | b}} @-}}
candidateCapacityThm :: Int -> Int -> Bool
candidateCapacityThm _ _ = True

-- Checked instantiations.  Each definition type-checks iff Z3 proves
-- @d >= requiredDimMultihop m_eff@ for the concrete arguments.
-- Together they are the concrete arithmetic the theorem requires.
-- Guarantee: Declared (pending LH/Z3 discharge).

-- NOTE: No in-regime non-refuted points — no probes generated.
-- End of auto-generated probe file.
-- STATUS: Declared — run `cabal build` to discharge (expects LIQUID: SAFE).
-- Guarantee: Declared until SAFE is confirmed AND the axiom is formally established.
