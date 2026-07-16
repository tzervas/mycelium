# Express oracle-blocker close (2026-07-16)

**Branch tip base:** post-companion `origin/dev`.
**Honesty:** pilot numbers `Empirical` via `just transpile-vet` + real `myc-check`.

## Changes (mycelium-transpile)

1. **Non-prelude foreign trait-impls** whole-gapped (no `unknown trait` file poison).
2. **Default → Init** rewrite for hand-written `impl Default`.
3. **Widen** free-fn emit with `width_cast` (de-dup signed/unsigned same Binary width).
4. **D4 collision mangling** for self-methods when bare name already taken.
5. **rotate_left/right** → `or(shl_u, shr_u)` composition (surface prims).
6. **saturating_*** methods gapped (no silent clamp prim).
7. Comparison lit-zero rewrite scaffolding (partial — std-time `is_negative` residual).

## Pilot remeasure (file-gated)

| Target | Before (M1006 pilot) | After this leaf |
|--------|---------------------:|----------------:|
| std-cmp | checked **0.0%** / expr 21.6% | checked **12.6%** / expr 12.6% · **file Clean** |
| std-rand | checked **0.0%** / expr 17.6% | checked **17.6%** / expr 17.6% · **file Clean** |
| std-time | checked **0.0%** / expr 45.9% | checked **0.0%** / expr 45.9% · residual `is_negative` bare `0` lit |

**First genuine movement of default-pilot `checked_fraction` off zero** on cmp/rand.

## Residual (next leaves)

- std-time: signed field compare to zero (Duration.nanos) still emits bare `0`.
- eval.rs: unknown type `Strength` (ast enum not co-emitted).
- Full M-1006 17-target ladder; M-1084 net-close measure; M-740; M-875 design.

