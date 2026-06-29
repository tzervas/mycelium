{-@ LIQUID "--reflection" @-}
{-@ LIQUID "--ple"        @-}

-- | M-832 / OQ-F — Multi-hop VSA capacity-refinement probe (bind_chain, A_exponential).
--
--   STATUS: Declared — STUB awaiting experiment output.
--
--   This file is a PLACEHOLDER.  Populate by running:
--     cd experiments
--     python -m mycelium_experiments.vsa_bounds --proof --emit-obligations
--
--   The experiment will overwrite this file with concrete probe instances derived
--   from the multihop sweep data.
--
--   Strategy (mirrors proofs/lh-bundle/src/Bundle.hs):
--     - AXIOMATIZE the candidate capacity theorem (assume candidateCapacityThm).
--     - Have Z3 discharge the concrete arithmetic: d >= requiredDimMultihop(m_eff).
--     - m_eff = F * k^h  (Model A_exponential, bind_chain composition).
--
--   Guarantee: Declared (stub; no probes populated yet).
module MultihopBound_bindchain_Aexponential where

-- | @requiredDimMultihop m@ — axiomatized lookup table (stub; populated by experiment).
--   Formula: ceil(200 * ln(m_eff / delta)), with m_eff = F * k^h.
{-@ reflect requiredDimMultihop @-}
requiredDimMultihop :: Int -> Int
requiredDimMultihop m
  | m <= 1    = 0      -- degenerate stub
  | otherwise = 9999999  -- sentinel: experiment will fill concrete values

-- | Well-capacitied refinement type.
{-@ type WellCapacitied M = {d:Int | d >= requiredDimMultihop M} @-}

-- | Axiomatized candidate multi-hop capacity theorem (Declared — pending proof/citation).
--   NOTE: This is the key open question (OQ-F / OQ-A).  A formal theorem connecting
--   m_eff = F * k^h to the multi-hop failure probability must be established.
{-@ assume candidateCapacityThm :: m:Int -> WellCapacitied m -> {b:Bool | b} @-}
candidateCapacityThm :: Int -> Int -> Bool
candidateCapacityThm _ _ = True

-- Probes (populated by experiment run):
-- probe1 = candidateCapacityThm <m_eff> <d>  -- F=... k=... h=...

-- STATUS: Declared (stub). Run `python -m mycelium_experiments.vsa_bounds --proof` to populate.
