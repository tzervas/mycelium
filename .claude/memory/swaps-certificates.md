# Swaps & Certificates — agent memory

Orientation aid (Declared/Empirical). Source + RFC-0002 + ADR-010/011 are ground truth.

## What it is

The **certified, never-silent representation swap** — the language's central operation.
Every swap yields a value in the target paradigm AND an inspectable `SwapCertificate`
describing what the conversion cost (RFC-0002 §2; SC-3). A swap that cannot state a bound
is an explicit `SwapError`, **never a silent coercion** (G2).

## Where it lives

- **Crate:** `crates/mycelium-cert/`
- `src/lib.rs` — `SwapCertificate`, `SwapError`, `BinaryTernarySwapEngine`,
  `CertifiedSwapEngine`, `binary_to_ternary`, `ternary_to_binary`, `legal_pair`
- `src/check.rs` — the single shared translation-validation checker: `check`, `check_core`,
  `RefinementRelation`, `CheckVerdict`, `Evidence`, `Fallback`, `NotValidatedReason`
- `src/dense.rs` — Dense F32→BF16 bounded swap (`dense_f32_to_bf16`, `BF16_REL_EPS`,
  `BF16_MIN_NORMAL`)
- `src/dense_vsa.rs` — Dense↔VSA probabilistic swap (`dense_to_vsa`, `vsa_to_dense`,
  `DENSE_VSA_EMP_DELTA`, `DENSE_VSA_MODEL`)

## Key types & operations

**`SwapCertificate`** (enum, `crates/mycelium-cert/src/lib.rs:51`):
- `Bijective { src, target, policy_used, lemma_ref, params: BinTernParams }` — for
  binary↔ternary in range. `lemma_ref` is the content hash of the once-per-kind round-trip
  lemma (M-121); `params` binds `{width, trits}`. Cacheable by content hash (RFC-0002 §3).
- `Bounded { src, target, policy_used, bound: Bound }` — for lossy swaps. Carries
  `Bound{kind, basis}` where `basis` is always present (ADR-011: `BoundBasis` is universal).
  `strength` is **derived from `basis`**, never asserted (M-I2/M-I3/M-I4).

**`SwapError`** (enum, `crates/mycelium-cert/src/lib.rs:79`): every failure path is explicit:
`WrongSource`, `IllegalPair`, `OutOfRange`, `NonFinite{index}`, `NotAnF32{index}`,
`SubnormalUnsupported{index}`, `RoundOverflow{index}`, `ApproximateSource`,
`InsufficientCapacity{components,dim,required}`, `NotBipolar{index}`, `NotDenseVsaEncoding`,
`AmbiguousDecode{index}`, `Wf(WfError)`. None are ever silent.

**`binary_to_ternary(src, trits_width, policy)`** (`src/lib.rs:267`): total on `B_n` for a
legal pair. Returns `(Value, SwapCertificate::Bijective)`. Meta carries `GuaranteeStrength::Exact`,
no bound, `Provenance::Derived`, `policy_used`. Errors: `WrongSource`, `IllegalPair`.

**`ternary_to_binary(src, binary_width, policy)`** (`src/lib.rs:311`): partial inverse.
Returns `(Value, SwapCertificate::Bijective)` or `SwapError::OutOfRange` when the ternary
value lies outside the binary range (P4 — never a silent wrap or truncation).

**`legal_pair(width, trits)`** (`src/lib.rs:231`): tests `B_n ⊆ T_m ⟺ 2^(n-1) ≤ (3^m−1)/2`.
Uses `i128` to avoid overflow. An illegal pair is a **type error** (RFC-0002 §5), not a gamble.

**`CertifiedSwapEngine`** (`src/lib.rs:393`): the complete certified swap surface (M-212).
Routes binary↔ternary, Dense F32→BF16, Dense↔VSA, and identity. Everything else is an
explicit `EvalError::UnsupportedSwap`.

## The certificate checker — `check(...)` (M-210)

`check(a, b, relation, claimed, evidence) -> CheckVerdict` (`crates/mycelium-cert/src/check.rs:158`).
Does artifact `B` refine reference `A` under `R` within `{ε, δ, strength}`?
Shared by representation swaps (RFC-0002) AND interpreter↔AOT equivalence (RFC-0004 §3).

**`RefinementRelation`** (`src/check.rs:45`):
- `Bijection` — binary↔ternary. Discharges by **structural re-derivation equality**: the
  lemma ref and `(n,m)` side-condition are checked, then the swap is re-derived from `A` and
  compared payload-for-payload with `B`. Claim must be `{0, 0, Exact}`.
- `BoundedSimilarity` — lossy swaps. The certificate's ε must cover the *measured* deviation
  of this instance, AND the claim must not be tighter than the certificate. Both re-validated
  through the `mycelium-numerics` tier-i checker (`src/check.rs:423`). VR-5 enforced at `src/check.rs:378`.
- `ObservationalEquiv` — interpreter↔AOT. Structural equality of `(repr, payload, guarantee)`.
  Extended to `CoreValue`/`Datum` via `check_core` (`src/check.rs:293`).

**`CheckVerdict`** (`src/check.rs:109`): `Validated{strength}` or `NotValidated{reason, fallback}`.
**Never a third state; never a silent pass.** TV incompleteness → explicit `NotValidated::Incomplete`
with `Fallback::UseReference` (keep `A`; run the trusted interpreter — ADR-007).

**`NotValidatedReason`** (`src/check.rs:78`): `Diverged{detail}` (genuine counterexample),
`CertificateMismatch{detail}` (binding failure or too-tight claim), `ClaimTooTight{recomputed,claimed}`,
`Incomplete{detail}` (checker cannot decide — not a counterexample).

## The split regime (ADR-002)

| Pair | Regime | Certificate | Strength |
|---|---|---|---|
| Binary ↔ Ternary (in range) | `LosslessWithinRange`, Exact | `Bijective` | `Exact` |
| Binary ↔ Ternary (out of range) | rejected, `OutOfRange` | — | — |
| Dense F32 → BF16 | Bounded (ε) | `Bounded`, `Error{eps=2^-8, Rel}` | `Proven` |
| Dense ↔ VSA | Bounded/probabilistic (δ) | `Bounded`, `Probability{delta}` | `Proven` or `Empirical` |
| Pair with no statable bound | type error | — | — |

## Key invariants (honesty)

- **Never silent (SC-3 / G2):** every out-of-range or unstateable-bound case is an explicit
  `SwapError`. No implicit coercion, wrap, or silent approximation.
- **Strength is derived, never asserted:** the `GuaranteeStrength` tag comes from `basis`
  (M-I2/M-I3/M-I4; ADR-011). `Proven` requires a `ProvenThm` basis with checked side-conditions.
- **VR-5 — claim never upgrades past evidence:** the checker rejects a claim whose `strength`
  is stronger than the certificate's checked `basis_strength` (`src/check.rs:378`).
- **TV completeness is stated:** every incomplete case is `NotValidated::Incomplete`, never a
  silent pass (RFC-0002 §2). Incompleteness is not a counterexample.
- **Approximate sources refused:** `ApproximateSource` whenever source `guarantee != Exact` and
  no composition rule exists for its bound (E2-1 open rule).
- **Provenance-gated decode:** `vsa_to_dense` checks the source was produced by
  `swap.dense_vsa.enc.v1` (op hash) before attaching the δ (`src/dense_vsa.rs:223`).

## Cert-overhead budget (KC-4 / ADR-021 A5)

Ratified 2026-06-21 (ADR-021 §3 A5): ≤ **5 µs** absolute AND ≤ **2×** the swap cost.
Measured: bijective ~1.7 µs / 1.3×; bounded F32→BF16 ~2 µs / 0.12×; observational ~8 ns.
All pass with ~2.5× headroom. Long-term target: nanosecond range (not a 1.0.0 blocker; RP-8).

## Read more

- `docs/rfcs/RFC-0002-Swap-Certificate-and-Split-Regime.md` — normative (all sections)
- `docs/Mycelium_Project_Foundation.md` §ADR-002 — split regime decision
- `docs/adr/ADR-010-Verified-Numerics-Foundation.md` — bound kernels
- `docs/adr/ADR-011-BoundBasis-Is-Universal.md` — BoundBasis on every Bound
- `docs/adr/ADR-021-1.0.0-Release-Readiness-Gate.md` §A5 — KC-4 budget
- `crates/mycelium-cert/tests/` — integration tests

## Gotchas

- `legal_pair` uses `i128` (not `i64`) to avoid overflow for the binary side.
  An illegal pair is a **type error** (RFC-0002 §5), not a gamble to tag `Declared`.
- The checker is **per-instance**, not a whole-engine proof. Bijective certs re-derive and
  compare; the lemma is cited, not re-proved.
- `vsa_to_dense` requires `Provenance::Derived{op == hash("swap.dense_vsa.enc.v1")}` —
  provenance-gated. A type-correct but differently-produced VSA value → `NotDenseVsaEncoding`.
- Dense↔VSA is `ProvenThm` when `vsa_dim ≥ requiredDim(n, δ)` (Clarkson-Ubaru-Yang 2023);
  falls back to `EmpiricalFit` (n ≤ 16, vsa_dim ≥ 32n, δ ≥ 0.05, 10 000 trials);
  refuses with `InsufficientCapacity` otherwise.
- `ApproximateSource` is a refusal, not a recoverable path. The E2-1 composition rule for
  combining an input bound with the swap's ε is explicitly open.
- `BinaryTernarySwapEngine` handles only binary↔ternary and identity; `CertifiedSwapEngine`
  delegates unrecognized pairs to it, yielding `EvalError::UnsupportedSwap`.
