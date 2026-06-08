# Research Record 02 — Targeted Research Findings T0/T1/T2 (Pass 2)

> **What this file is.** A durable record of the second research pass, which resolved the enumerated open questions from Pass 1.

## Scope
Resolve the make-or-break and design-blocking questions the survey left open, in three tiers: **T0** existential/feasibility, **T1** mechanism-selection, **T2** design-detail + coverage-gap closure.

## Results by target

### T0 — Existential
- **T0.2 / KC-1 — PASSED.** Proven, *non-asymptotic* VSA bundling bounds exist (Clarkson-Ubaru-Yang 2023 and Thomas-Dasgupta-Rosing 2021).
- **T0.1 — two bound kernels, one certificate.** A single algebra cannot unify ε-magnitude and δ-probability composition.

### T1 — Mechanism selection
- **T1.1 — translation validation.** Per-instance certificate checking is the mature pattern.
- **T1.2 — per-model × per-operation guarantee matrix.** Detailed honest tags per model.
- **T1.3 — sparsity as static refinement.** Feasible for declared bounds.
- **T1.4 — schedule-staged packing is tractable.** Small fixed set avoids Halide's hard problem.
- **T1.5 — MLIR backbone → LLVM.**

### T2 — Design detail & coverage-gap closure
- **T2.1 — binary↔ternary is `LosslessWithinRange`.**
- **T2.2 — reconstruction manifest.** Distinguishes indexed retrieval from true compositional reconstruction.
- **T2.3 — total cost-based selection policy + mandatory EXPLAIN.**
- Coverage gaps on array languages, neurosymbolic IRs, verified probabilistic numerics, and Rust VSA ecosystem closed.

## One confirming build still open
The Liquid-Haskell `bundle` capacity-refinement probe (RFC-0003 §5).

## Key sources
Clarkson, Ubaru, Yang (2023); Thomas, Dasgupta, Rosing (2021); Frady et al.; Becker et al. (FloVer); Barthe et al. (apRHL); Leroy et al. (CompCert); Vazou et al. (Liquid Haskell); Henriksen et al. (Futhark); bitnet.cpp papers.
