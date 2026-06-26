# RFC-0002 — Swap Certificate & Split Regime

| Field | Value |
|---|---|
| **RFC** | 0002 |
| **Status** | **Accepted** (solidified from the research pass) |
| **Type** | Foundational / normative |
| **Date** | June 08, 2026 |
| **Depends on** | RFC-0001 (`Swap` node, `Meta.{guarantee,bound,policy_used}`, guarantee lattice, content-addressing); ADR-002 (split regime); ADR-010 (bound kernels & certificate); VR-4; Research Findings **T1.1**, **T2.1** |
| **Coupled with** | RFC-0004 (shares the single certificate checker), RFC-0005 (`PolicyRef`) |

## 1. Scope
Defines the `SwapCertificate`, the legal `(R_src → R_target)` pairs and their verification regime, and the per-swap (not once-for-all) validation mechanism. Turns RFC-0001's `Swap` node into a verifiable operation.

## 2. The single certificate & checker (T1.1)
Translation validation — proving each *instance* correct — is the mature, proven pattern (CompCert/Valex; seL4 SMT-based TV; Crellvm credible/witness compilation; SMT equivalence checking). It is strictly right for evolving, bounded/probabilistic lowerings.

**Decision (shared with RFC-0004):** ONE refinement/equivalence-certificate format and checker, taking `(A, B, R, claimed-bound, certificate)` and answering "does artifact B refine reference A under relation R within `{ε, δ, strength}`?" Both a representation swap *and* interpreter-vs-compiled equivalence (RFC-0004) are instances:
- bijective swap → R is a bijection, bound `{0, 0, Exact}`;
- lossy swap → R is approximate similarity-preservation, bound from ADR-010 + T0.2;
- interpreter-vs-compiled → R is observational (or bounded) equivalence.
Exact cases discharge by SMT/symbolic equality; approximate cases by the ADR-010 bound kernels. The checker is a Rust certificate *consumer* (ADR-010 trusted-base tiers). TV may *fail to validate a correct* translation (incompleteness) → an explicit fallback path is required, never a silent pass.

## 3. Certificate content
- **Bijective swaps:** prove the swap *function* bijective **once per swap-kind**; each use emits a lightweight certificate referencing that lemma + concrete parameters, **cacheable by content hash** (RFC-0001 §4.6). No per-value proofs.
- **Bounded/lossy swaps:** certificate carries `Bound` (ε and/or δ) + `BoundBasis` (`ProvenThm | EmpiricalFit | UserDeclared`) + `PolicyRef`. The `strength` tag is **derived from how the bound was obtained**, never asserted: `Proven` only if it cites a theorem whose side-conditions (dimension, sparsity class, model) are checked here.

## 4. Binary↔Ternary bijection semantics (T2.1) — normative
Balanced ternary: digits {−1,0,+1}; negation = digit-wise sign flip; truncation = rounding (Knuth). **Cardinality mismatch is central:** n bits = 2ⁿ values; m trits = 3ᵐ values over the symmetric range [−(3ᵐ−1)/2, +(3ᵐ−1)/2] (8 bits = 256; 6 trits = 729, range −364..+364).

- A **total bijection** requires 2ⁿ = 3ᵐ, which holds only trivially → **never for fixed nonzero widths**.
- The real regime is **`LosslessWithinRange`**: an injection of the smaller domain into the larger codomain, with the inverse defined only on the image.

**Specification:** define `enc : Bin_n → Tern_m` and `dec : Tern_m → Option Bin_n`; prove (a) `dec (enc x) = Some x` ∀x (left-inverse/injectivity) and (b) `dec y = Some x ⟹ enc x = y` (partial right-inverse on the image). This **round-trip property is SMT-dischargeable** for fixed widths and provable by `decide`/computation in Coq. Tag **Exact within range**; out-of-range inputs are an explicit `Option`/error, **never silent** (consistent with no-hidden-behavior). Canonical encodings to adopt/learn from: IOTA TIP-5 byte↔trit (6-trit groups/byte); the Douglas W. Jones 2-bits-per-trit family.

This is the only genuinely bijective/provable swap class — exactly as the split regime (ADR-002) assumes.

## 5. Legal-pair table (normative skeleton; bounds per ADR-010 / RFC-0003)
| `R_src → R_target` | Regime | Bound basis |
|---|---|---|
| Binary ↔ Ternary (in range) | `LosslessWithinRange`, Exact | proof of `enc`/`dec` round-trip |
| Binary ↔ Ternary (out of range) | rejected / explicit error | — (never silent) |
| Dense `F32` → `BF16` | Bounded (ε) | rounding-error theory (ADR-010 ErrorBound) |
| Dense ↔ VSA | Bounded/probabilistic (ε, δ) | VSA capacity results (RFC-0003, T0.2) |
| VSA model ↔ VSA model | case-by-case | per-pair derivation (RFC-0003 matrix) |
| pair with no statable bound | **type error** | — (not a `Declared` gamble) |

## 6. Interfaces
Consumes RFC-0001 `Swap`, `Meta.{guarantee,bound,policy_used}`, the lattice, content-addressing. Uses **ADR-010** bound kernels + certificate. **Shares the §2 checker with RFC-0004.** `PolicyRef` from **RFC-0005**.

## 7. Residual
Mechanically checking the *derivations* of lossy-swap bounds end-to-end (vs. axiomatizing cited theorems) is future work; the accepted path axiomatizes theorem statements (tag `Proven`, proof = citation) and checks only the arithmetic instantiation (per ADR-010).

> **Footnote — tunable certification (RFC-0034 / ADR-032, 2026-06-24; append-only).** The per-swap certificate emission/checking mandated here applies **at the active certification mode (`certified`)**; the `fast` (default) and `balanced` relaxations are governed by **RFC-0034**, and the swap stays **never silent** in every mode (G2 — mode-tagged + `EXPLAIN`-able). The certificate machinery is **unchanged**. See **ADR-032**, which supersedes the *unconditional* reading.
