# Verified Numerics & Dense Embeddings — agent memory

Orientation aid (Declared/Empirical). Source + ADR-010/011 + RFC-0001 §4.7 are ground truth.

## What it is

Two **separate bound kernels** (different monoids — ADR-010/T0.1c settled negative result)
meeting at one **shared certificate** `{ε, δ, strength}`:

- **`ErrorBound` kernel (ε):** affine arithmetic — sound, compositional, correlation-aware.
- **`ProbBound` kernel (δ):** union bound + apRHL sequencing.

Plus the **Dense paradigm operational surface**: typed elementwise ops with honest per-op
guarantee tags. Not a unified approximation algebra — two distinct composition paths.

## Where it lives

**Numerics crate:** `crates/mycelium-numerics/`
- `src/lib.rs` — crate root + re-exports
- `src/cert.rs` — `Certificate`, `ErrorOp`, `CheckOutcome`, `check_error_claim`,
  `check_union_claim`, `compose_error_bound`, `basis_strength`, `recompute_error`, `error_norm`
- `src/error.rs` — `AffineForm`, `ErrorBound`, `NoiseSym`, `ROUNDOFF_SYM`
- `src/prob.rs` — `ProbBound`, `ApRhlJudgment`

**Dense crate:** `crates/mycelium-dense/`
- `src/lib.rs` — `DenseSpace`, `DenseOp`, `DenseError`, `F32_OP_REL_EPS`, `BF16_OP_REL_EPS`,
  `DENSE_MIN_NORMAL`

## Key types & operations

### ErrorBound (ε) — `crates/mycelium-numerics/src/error.rs`

**`AffineForm`** (`src/error.rs:46`): `x₀ + Σ xᵢ·εᵢ` over noise symbols `εᵢ ∈ [−1,+1]`.
- Linear ops (`add`/`sub`/`neg`/`scale`) are exact on shared noise symbols — correlated
  uncertainty cancels. Each op folds its own f64 round-off into `ROUNDOFF_SYM` (`u64::MAX`,
  `src/error.rs:40`) so `radius()` stays a sound enclosure (A2-01/WS1).
- `mul` is nonlinear; second-order remainder goes onto a fresh noise symbol (`src/error.rs:186`).
- `radius()` (`src/error.rs:94`): `Σ|xᵢ|` outward-rounded — the sound ε.

**`ErrorBound{eps, norm}`** (`src/error.rs:221`): the scalar projection that travels in
`BoundKind::Error`. Private fields — only constructible via `new`/`exact`/compositions (A2-05).
- `new(eps, norm) -> Option<Self>`: rejects negative or non-finite eps (never silent).
- `add`, `sub`, `scale`, `mul`: all outward-rounded, all `None` on norm mismatch.
- `eps(x+y) = eps(x)+eps(y)` (outward); `eps(cx) = |c|·eps(x)` (outward);
  `eps(xy) ≤ |x₀|·eps(y) + |y₀|·eps(x) + eps(x)·eps(y)`.

### ProbBound (δ) — `crates/mycelium-numerics/src/prob.rs`

**`ProbBound{delta}`** (`src/prob.rs:16`): δ ∈ [0,1]; private field (A2-05).
- `union(bounds) -> ProbBound`: `min(1, Σδᵢ)` outward-rounded (`src/prob.rs:44`). Identity = `certain()`.
- `or(&self, other) -> ProbBound`: binary union.

**`ApRhlJudgment{eps, delta}`** (`src/prob.rs:69`): relational `⟨ε,δ⟩` judgment.
- `seq(&self, next) -> ApRhlJudgment`: `[SEQ]` rule — ε adds, δ adds (clamped to 1) (`src/prob.rs:98`).

### Shared certificate — `crates/mycelium-numerics/src/cert.rs`

**`Certificate{eps, delta, strength}`** (`src/cert.rs:159`): both kernels reduce to this.
- `strength` composes by **`meet`** (weakest wins; `GuaranteeStrength::TOP = Exact`).
- `new(eps, delta, strength) -> Option<Self>`: validates ranges (finite, eps ≥ 0, delta ∈ [0,1]).
- `exact() -> Self`: `{0, 0, Exact}` — the bijective cert constant.
- `from_error(ErrorBound, strength) -> Self`: lift ε, δ side = 0.
- `from_prob(ProbBound, strength) -> Self`: lift δ, ε side = 0.

**`BoundBasis`** (in `mycelium-core`; ADR-011): universal companion of every `Bound`.
- `ProvenThm{citation}` → `Proven`
- `EmpiricalFit{trials, method}` → `Empirical`
- `UserDeclared` → `Declared`
`basis_strength(basis) -> GuaranteeStrength` (`src/cert.rs:255`): derives strength from basis.
Certificate consumers (M-210 checker) call this — never accept an asserted strength.

**Tier-i Rust checker** (ADR-010 "Trusted base"):
- `check_error_claim(inputs, op, claimed) -> CheckOutcome` (`src/cert.rs:109`):
  re-derives `recomputed`; valid iff `claimed.eps + slack ≥ recomputed.eps`. Slack = few ULPs
  of the re-derived magnitude (`CHECK_REL_TOL = 8·f64::EPSILON`, `src/cert.rs:30`).
- `check_union_claim(inputs, claimed) -> CheckOutcome` (`src/cert.rs:129`):
  valid iff `claimed.delta + slack ≥ min(1, Σδᵢ)`.
- `CheckOutcome`: `Valid`, `Rejected{recomputed, claimed}`, `Malformed`. Never a silent pass.

**`compose_error_bound(inputs, op) -> Option<ComposedBound>`** (`src/cert.rs:322`):
Entry point for the interpreter (M-204). Composes `Error` bounds via the kernel; strength is
`meet` of inputs; basis matches that strength. Returns `None` — refuses, never fabricates —
on non-Error kind, norm mismatch, wrong arity, empty, or overflow to non-finite.

**Cross-kernel inference** (one sanctioned, `src/cert.rs:148`):
`accuracy_to_probability(acc: ErrorBound, tau: f64, acc_delta: f64) -> Option<ProbBound>`:
"failure prob = `acc_delta` if `acc.eps ≤ tau`, else 1.0". No other mixing exists (ADR-010 §4).

### Dense ops — `crates/mycelium-dense/src/lib.rs`

**`DenseSpace{dim, dtype}`** (`src/lib.rs:194`): typed `Dense{dim, dtype}` space.
Only `F32` and `BF16` supported in v1 (`F16`/`F64` → `DenseError::UnsupportedDtype`).

**Elementwise ops — honest per-op guarantee tags:**
- **`neg_value`** → **`Exact`** (`src/lib.rs:393`): grids are symmetric; negation never rounds.
- **`add_values`** → **`Proven`** (`src/lib.rs:364`): per-element relative ε = `F32_OP_REL_EPS`
  (`2^-24`) or `BF16_OP_REL_EPS` (`2^-8 + 2^-23`); basis `ProvenThm` citing Higham 2002 Thm 2.2.
  Side-conditions checked per element: exact on-grid inputs, finite zero-or-normal result, no overflow.
- **`sub_values`** → **`Proven`**, same bound as `add` (`src/lib.rs:368`).
- **`scale_value`** → **`Proven`**, same bound; scale factor must be finite and on-grid (`src/lib.rs:413`).

**Why `add`/`sub`/`scale` are Proven:** the cited theorem (Higham 2002, Thm 2.2) gives a closed
relative-error bound for a single IEEE round-to-nearest operation — its side-conditions (finite,
normal, no overflow, exact on-grid inputs) are checked per element in `scalars_of` and
`round_result`. The computation IS the instantiation; the theorem is accepted axiomatically
(RFC-0002 §7; ADR-010 tier-i).

**Measurement helpers** (no `Meta` to tag — bare `f64`):
- **`dot`** (`src/lib.rs:429`): inner product in f64. No guarantee tag.
- **`similarity`** (`src/lib.rs:437`): cosine similarity in `[-1, 1]`. No guarantee tag.

**Why dot/similarity carry no tag:** they return bare `f64`, not `Value` — there is no `Meta`
slot. The nγ_n floating-point accumulation bound (the theorem that would bound `n`-term sums)
is not yet in the corpus. These are measurement utilities, not verified ops.

**`DenseError`** (`src/lib.rs:70`): typed, never silent. Key variants: `DimMismatch`,
`DtypeMismatch`, `UnsupportedDtype`, `NonFinite{index}`, `NotOnGrid{index}`, `ScalarOffGrid`,
`SubnormalUnsupported{index}`, `Overflow{index}`, `ApproximateSource`, `Wf(WfError)`.

**Constants:**
- `F32_OP_REL_EPS = 2^-24` (`src/lib.rs:34`) — single-rounding unit roundoff
- `BF16_OP_REL_EPS = 2^-8 + 2^-23` (`src/lib.rs:39`) — two-rounding composition bound
- `DENSE_MIN_NORMAL = f32::MIN_POSITIVE as f64 = 2^-126` (`src/lib.rs:43`) — subnormal floor

## Proven vs Empirical — the honest distinction

| Op | Tag | Basis | Why |
|---|---|---|---|
| `neg` | `Exact` | none | grid symmetry; zero rounding |
| `add`/`sub`/`scale` | `Proven` | `ProvenThm{Higham 2002 Thm 2.2}` | single-rounding bound; side-conditions checked per element |
| Dense F32→BF16 swap | `Proven` | `ProvenThm{Higham 2002 Thm 2.2}` | same theorem, `u = 2^-8` |
| Dense↔VSA (proven path) | `Proven` | `ProvenThm{Clarkson-Ubaru-Yang 2023}` | capacity side-condition checked |
| Dense↔VSA (empirical path) | `Empirical` | `EmpiricalFit{10000 trials}` | theorem side-condition fails; profile validated |
| composed ε (`Proven⊕Proven`) | `Proven` | `ProvenThm{ADR-010 §1 affine-arith}` | `compose_error_bound`; side-condition: all inputs Proven |
| composed ε (any `Empirical`) | `Empirical` | `EmpiricalFit{min trials}` | weakest wins via `meet` |
| `dot`/`similarity` | none | n/a | bare `f64`; nγ_n accumulation theorem not in corpus |

**The honesty rule:** strength is always *derived* from `basis` via `basis_strength()`, never
asserted. `Proven` is only valid when the cited theorem's side-conditions are checked at the
call site. Downgrade to `Empirical`/`Declared` rather than assert (VR-5).

## Key invariants (honesty)

- **Strength from basis (M-I2/M-I3/M-I4):** `Proven ⟹ ProvenThm`; `Empirical ⟹ EmpiricalFit`;
  `Declared ⟹ UserDeclared`. Enforced at construction; `basis_strength` is the derivation fn.
- **Never stronger than evidence (VR-5):** `compose_error_bound` takes the `meet`; the tier-i
  checker rejects a claim tighter than re-derivation. Outward rounding everywhere (A2-01).
- **Never silent (G2):** invalid eps/delta → `None` from constructors; norm mismatch → `None`.
  Overflow of composition to non-finite → refused by `ErrorBound::new` in `compose_error_bound`.
- **Approximate sources refused in Dense:** `scalars_of` checks `guarantee == Exact`; otherwise
  returns `DenseError::ApproximateSource` (the E2-1 composition rule is open — M-204/M-211).
- **Subnormal refused:** `round_result` returns `SubnormalUnsupported` for subnormal results —
  the cited theorem's side-condition covers only normal-and-zero (same scope as M-211).
- **BoundBasis is universal (ADR-011):** every `Bound` carries a `basis` — no bound can float
  without disclosing how it was obtained.

## Read more

- `docs/adr/ADR-010-Verified-Numerics-Foundation.md` — two kernels, trusted base, cross-kernel
- `docs/adr/ADR-011-BoundBasis-Is-Universal.md` — BoundBasis on every Bound variant
- `docs/rfcs/RFC-0001-Core-IR-and-Metadata-Schema.md` §4.7 — bound composition in the IR
- `crates/mycelium-numerics/tests/properties.rs` — Soundness/Monotonicity/Determinism tests
- `crates/mycelium-dense/tests/` — Dense op property tests
- `docs/rfcs/RFC-0002-Swap-Certificate-and-Split-Regime.md` §3/§5 — how certs consume kernels

## Gotchas

- `ErrorBound` and `ProbBound` **do not mix** (ADR-010/T0.1c). The only sanctioned bridge is
  `accuracy_to_probability`. Never add ε and δ directly.
- `CHECK_REL_TOL = 8·f64::EPSILON` is **relative**, not absolute. A claim of `eps=0` against
  a re-derived `5e-13` is correctly rejected (the old absolute `1e-12` tolerance was vacuous
  for tiny bounds — A2-02 fix).
- `compose_error_bound` returns `None` on **any** non-Error input bound. It does not compose
  `Probability` bounds. `ProbBound::union` is the separate path.
- `ROUNDOFF_SYM = u64::MAX` is reserved for the accumulated f64 round-off of the affine ops
  themselves. Never pass `u64::MAX` as a user noise symbol to `AffineForm::uncertain`.
- `AffineForm::mul` needs a `fresh` symbol not already in either operand. The caller is
  responsible for minting it (debug_assert guards this, not production check).
- `F16`/`F64` dtypes: `DenseSpace::new` refuses them (`UnsupportedDtype`). Honest scope: `F64`
  ops cannot be re-derived against an exact reference in `f64`; `F16` awaits a use case.
- `dot`/`similarity` accept approximate sources (they call `scalars_of` which requires `Exact`).
  They will return `DenseError::ApproximateSource` for approximate inputs — same guard.
