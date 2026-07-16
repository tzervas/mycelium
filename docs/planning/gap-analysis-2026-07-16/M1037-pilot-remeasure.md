# M-1037 pilot re-measure (Empirical, advisory)

**Date:** 2026-07-16
**Command:** `just transpile-vet crates/mycelium-std-cmp/src`
**Basis:** single-crate oracle mode (not phylum); post M-1037 leaf on `origin/dev` tip.

| Metric | Value |
|--------|------:|
| Items | 111 |
| expressible_fraction | 21.6% (24 emitted) |
| checked_fraction | 0.0% (0/111 file-gated clean) |

**Interpretation (VR-5):** This pilot does **not** by itself prove a corpus-wide `checked_fraction`
rise — std-cmp remains file-gated blocked on pre-existing check errors unrelated to conversion-method
mapping. M-1037's lever is enabling more **emitted** arm bodies (identity accessors + composed `.ne`)
to avoid fabricated-prim poisoning when those shapes appear in port targets. Full M-1006 /
M-1090 re-measure after Import + conversion batch is still **FLAG** residual for the integrating
parent (L4 leaf in WAVE-G3-NEXT).
