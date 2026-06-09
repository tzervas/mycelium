{-@ LIQUID "--reflection" @-}
{-@ LIQUID "--ple"        @-}

-- | M-001 — MAP-I @bundle@ capacity-refinement probe (RFC-0003 §5).
--
--   DRAFT / SCAFFOLD: this module is NOT yet type-checked. The authoring environment has no
--   GHC / LiquidHaskell / Z3 (see README "Honesty status"). The integer inequality each
--   verification condition reduces to is tabulated in the README and is the authoritative,
--   independently-checkable artifact; running LiquidHaskell is the remaining step.
--
--   Strategy (the thing this probe confirms): AXIOMATIZE the cited capacity theorem's statement,
--   and have Z3 discharge only the concrete ARITHMETIC INSTANTIATION — never re-prove the
--   concentration inequality. (T0.2: Clarkson-Ubaru-Yang 2023 Thm 6; Thomas-Dasgupta-Rosing
--   2021 Thm 2/7. ADR-010 cited-theorem-with-checked-instantiation pattern.)
module Bundle where

-- | @requiredDim m@ is the axiomatized right-hand side of the capacity theorem:
--   @ceil( (2/mu^2) * ln(m/delta) )@, with the illustrative margin @mu = 0.1@ (so @2/mu^2 = 200@)
--   and the per-setting @delta@ from the README table. The /formula/ is cited (T0.2), not proven
--   here; only its concrete values enter the checked arithmetic below.
{-@ reflect requiredDim @-}
requiredDim :: Int -> Int
requiredDim m
  | m <= 3    = 1141 -- m = 3,   delta = 1e-2
  | m <= 10   = 1843 -- m = 10,  delta = 1e-3
  | m <= 50   = 2164 -- m = 50,  delta = 1e-3
  | otherwise = 2764 -- m = 100, delta = 1e-4

-- | A bundle of @m@ items is well-capacitied at dimension @d@ when @d@ meets the axiomatized
--   requirement. This integer inequality is exactly what Z3 discharges.
{-@ type WellCapacitied M = {d:Int | d >= requiredDim M} @-}

-- | The axiomatized capacity guarantee (T0.2): a well-capacitied bundle decodes with failure
--   probability at most its target delta. @assume@ = the cited theorem is taken as given;
--   LiquidHaskell does not re-prove it.
{-@ assume capacityThm :: m:Int -> WellCapacitied m -> {b:Bool | b} @-}
capacityThm :: Int -> Int -> Bool
capacityThm _ _ = True

-- Checked instantiations. Each definition type-checks iff Z3 proves @d >= requiredDim m@ for the
-- concrete arguments; together they are the >= 3 settings #2 requires.
{-@ probe1 :: {b:Bool | b} @-}
probe1 :: Bool
probe1 = capacityThm 3 10000 -- 10000 >= 1141

{-@ probe2 :: {b:Bool | b} @-}
probe2 :: Bool
probe2 = capacityThm 10 10000 -- 10000 >= 1843

{-@ probe3 :: {b:Bool | b} @-}
probe3 :: Bool
probe3 = capacityThm 50 10000 -- 10000 >= 2164

{-@ probe4 :: {b:Bool | b} @-}
probe4 :: Bool
probe4 = capacityThm 100 10000 -- 10000 >= 2764
