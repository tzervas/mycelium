# Research Record 02 — Targeted Research Findings T0/T1/T2 (Pass 2)

> **What this file is.** A durable record of the second research pass, which resolved the enumerated open questions from Pass 1. The *full narrative* findings report (≈7–8k words, with inline citations) was delivered as a conversation artifact; this record preserves scope, results, the decisions applied, and the source base. Ask the maintainer (or re-run the assistant) to drop the full narrative in here for a self-contained copy.

## Scope
Resolve the make-or-break and design-blocking questions the survey left open, in three tiers: **T0** existential/feasibility, **T1** mechanism-selection, **T2** design-detail + coverage-gap closure.

## Results by target

### T0 — Existential
- **T0.2 / KC-1 — PASSED.** Proven, *non-asymptotic* VSA bundling bounds exist, so core VSA can carry honest `Proven` tags. Anchors: **Clarkson, Ubaru & Yang (2023)** capacity analysis (arXiv:2301.10352) and **Thomas, Dasgupta & Rosing (2021)** theoretical foundations (JAIR; arXiv:2010.07426). This was the project's central risk; it cleared.
- **T0.1 — two bound kernels, one certificate.** A single algebra cannot unify ε-magnitude and δ-probability composition (settled negative). Decision → **ADR-010**: `ErrorBound` (affine arithmetic; Daisy/FloVer) + `ProbBound` (union-bound; apRHL couplings) meeting at a shared `{ε,δ,strength}` certificate; certificate-checker-in-Rust trusted base.

### T1 — Mechanism selection
- **T1.1 — translation validation.** Per-instance certificate checking (CompCert/Valex, seL4 SMT-TV, Crellvm, CakeML) is the mature pattern → ONE refinement/equivalence checker shared by swaps (RFC-0002) and interpreter-vs-compiled equivalence (RFC-0004).
- **T1.2 — per-model × per-operation guarantee matrix.** bind/unbind self-inverse Exact for MAP/BSC; permute Exact everywhere; bundle Proven for MAP-I/sparse-Bloom, on-expectation for BSC, Empirical for HRR/FHRR; **HRR/FHRR unbind is the residual Empirical weak link** → RFC-0003 §4.
- **T1.3 — sparsity as static refinement.** Declared sparsity class is feasible as a Liquid-Haskell-style refinement; capacity bounds = axiomatized cited theorem + checked arithmetic instantiation → RFC-0001 §4.4, RFC-0003 §5. (Closest prior art: Heim / Yi & Achour, OOPSLA 2023 — as analysis, not types.)
- **T1.4 — schedule-staged packing is tractable.** The small fixed packing set (≈5 schemes) avoids Halide's hard general scheduling problem → DN-01 Resolved, RFC-0004 §5. Packings reuse bitnet.cpp: I2_S (lossless default), TL1 (ARM), TL2 (x86/low-mem).
- **T1.5 — MLIR backbone → LLVM**, Rust interpreter as reference semantics/trusted base → ADR-007, RFC-0004 §2.

### T2 — Design detail & coverage-gap closure
- **T2.1 — binary↔ternary is `LosslessWithinRange`.** Total bijection impossible at fixed widths (2ⁿ = 3ᵐ only trivially); `Option`-typed inverse, Exact-within-range, never silent; round-trip SMT-dischargeable → RFC-0002 §4. Encodings: IOTA TIP-5; Jones 2-bits-per-trit family.
- **T2.2 — reconstruction manifest.** Distinguish indexed retrieval from true compositional reconstruction; manifest = model+dim, content-addressed codebooks, compositional recipe, decoding procedure, bound certificate; resonator factorization (Frady et al., Neural Computation 2020) is lossy-bounded, best-effort, Phase-3 → RFC-0003 §6.
- **T2.3 — total cost-based selection policy + mandatory EXPLAIN**; avoids the DB cardinality-estimation black box because Mycelium's statistics are exact metadata, not sampled estimates → RFC-0005.
- **T2.4 — array/representation languages** (Futhark, Dex) — coverage gap closed; informs schedule-staging.
- **T2.5 — neurosymbolic synthesis IRs** — coverage gap closed.
- **T2.6 — Rust VSA/ternary ecosystem is immature** (no torchhd analogue) → build the VSA submodule; reuse the `balanced-ternary` crate; port torchhd's op set as reference → RR-14, ADR-007.

## One confirming build still open
The Liquid-Haskell `bundle` capacity-refinement probe (RFC-0003 §5): encode a MAP-I capacity refinement instantiating the T0.2 theorems and confirm Z3 discharges the arithmetic — this ratifies the axiomatized-theorem + checked-instantiation strategy end to end. (The one existential question *not* fully settled is KC-2 / LLM leverage — the E4 surface experiment.)

## Key sources
(Representative; full inline citations are in the conversation artifact.)
- Clarkson, Ubaru, Yang (2023) — VSA capacity analysis (arXiv:2301.10352).
- Thomas, Dasgupta, Rosing (2021) — Theoretical foundations of HDC, JAIR (arXiv:2010.07426).
- Frady, Kleyko, Sommer — VSA capacity (Neural Computation).
- Becker et al. — FloVer (verified roundoff-bound checker); Darulova et al. — Daisy; VCFloat2.
- Barthe et al. — apRHL / approximate couplings; span liftings; probabilistic relational reasoning.
- Kaminski, Katoen et al. — weakest pre-expectation calculus.
- Leroy et al. — CompCert; Klein et al. — seL4 (translation validation); Crellvm; CakeML.
- Vazou et al. — Liquid Haskell; Yi & Achour — Heim (OOPSLA 2023, sparse VSA analysis).
- Henriksen et al. — Futhark; Paszke et al. — Dex.
- Ma/Wang et al. — BitNet b1.58 (arXiv:2402.17764); bitnet.cpp (arXiv:2502.11880).
- Frady, Kent, Olshausen, Sommer — Resonator networks (Neural Computation 32(12), 2020).
- CockroachDB / YugabyteDB cost-based-optimizer documentation.
- `balanced-ternary` Rust crate; Heddes et al. — torchhd.
- Knuth — *TAOCP* Vol. 2; Douglas W. Jones — binary-coded ternary.
