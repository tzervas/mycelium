{-@ LIQUID "--reflection" @-}
{-@ LIQUID "--ple"        @-}

-- | M-832 / OQ-F — Multi-hop VSA capacity-refinement probe (bind_chain, A_exponential).
--
--   STATUS: Declared — PENDING proof-assistant discharge (VR-5 / ADR-032).
--
--   Strategy (mirrors proofs/lh-bundle/src/Bundle.hs):
--     - AXIOMATIZE the candidate multi-hop capacity theorem's statement.
--     - Have Z3 discharge only the concrete ARITHMETIC INSTANTIATION — never
--       re-prove the concentration inequality.
--
--   Candidate theorem (Declared):
--     For composition 'bind_chain', model 'A_exponential':
--       m_eff(F, k, h) = F * k^h
--       required_dim_multihop(m_eff, delta) = ceil((2/mu^2) * ln(m_eff/delta))
--     Then d >= required_dim_multihop implies failure probability <= delta.
--
--   Empirically validated regime: compositions=['bind_chain', 'bundle_of_binds', 'nested_unbind']; F=[2]; k=[4]; h=[1, 2]; d in [2048, 8192]
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
module MultihopBound_bindchain_Aexponential where

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
  | m <= 8  = 1199  -- m_eff=8, delta=0.02, req_dim=1199
  | m <= 32  = 1476  -- m_eff=32, delta=0.02, req_dim=1476
  | otherwise  = 1476  -- conservative: largest in table

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

{-@ probe1 :: {b:Bool | b} @-}
probe1 :: Bool
probe1 = candidateCapacityThm 8 2048  -- bind_chain F=2 k=4 h=1: 2048 >= 1199 (requiredDimMultihop 8)

{-@ probe2 :: {b:Bool | b} @-}
probe2 :: Bool
probe2 = candidateCapacityThm 8 4096  -- bind_chain F=2 k=4 h=1: 4096 >= 1199 (requiredDimMultihop 8)

{-@ probe3 :: {b:Bool | b} @-}
probe3 :: Bool
probe3 = candidateCapacityThm 8 8192  -- bind_chain F=2 k=4 h=1: 8192 >= 1199 (requiredDimMultihop 8)

{-@ probe4 :: {b:Bool | b} @-}
probe4 :: Bool
probe4 = candidateCapacityThm 32 2048  -- bind_chain F=2 k=4 h=2: 2048 >= 1476 (requiredDimMultihop 32)

{-@ probe5 :: {b:Bool | b} @-}
probe5 :: Bool
probe5 = candidateCapacityThm 32 4096  -- bind_chain F=2 k=4 h=2: 4096 >= 1476 (requiredDimMultihop 32)

{-@ probe6 :: {b:Bool | b} @-}
probe6 :: Bool
probe6 = candidateCapacityThm 32 8192  -- bind_chain F=2 k=4 h=2: 8192 >= 1476 (requiredDimMultihop 32)

-- End of auto-generated probe file.
-- STATUS: Declared — run `cabal build` to discharge (expects LIQUID: SAFE).
-- Guarantee: Declared until SAFE is confirmed AND the axiom is formally established.
