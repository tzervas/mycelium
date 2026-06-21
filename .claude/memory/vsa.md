# VSA Memory — Vector Symbolic Architectures / Hyperdimensional Computing

**Normative source:** RFC-0003 (Accepted r4) `docs/rfcs/RFC-0003-VSA-Submodule-Boundary.md`
**Resonator:** RFC-0009 (Accepted) `docs/rfcs/RFC-0009-Resonator-Network-Factorization.md`

---

## What it is

Hyperdimensional computing over high-dimensional random vectors (hypervectors). Five models share
one algebra (bind/unbind/bundle/permute/similarity) with honest, per-op guarantee tags that differ
per model. Binding associatively encodes relationships; bundling superimposes sets; permuting
encodes roles/order. Clean-up (nearest-neighbor lookup) recovers stored atoms from noisy sums.

---

## Where it lives

- **Crate:** `crates/mycelium-vsa/` (RFC-0003 §2, ADR-008: optional submodule — kernel does NOT
  depend on it; programs mentioning `Repr::Vsa` type-check without the algebra: KC-3)
- `src/lib.rs` — `VsaModel` trait, `VsaOp` enum, `VsaError` variants, `EmpiricalProfile` struct
- `src/matrix.rs` — `RFC0003_MATRIX` const + `matrix_tag()` (single source of truth for §4 tags)
- `src/capacity.rs` — `proven_capacity_bound()`, `required_dim()` (MAP-I bundle Proven check)
- `src/mapi.rs` — `MapI` (bipolar ±1, elementwise product bind, integer superposition bundle)
- `src/mapb.rs` — `MapB` (sign-rounded bundle)
- `src/bsc.rs` — `Bsc` (XOR bind, majority bundle)
- `src/hrr.rs` / `src/fhrr.rs` — `Hrr`, `Fhrr` (circular convolution / complex multiply)
- `src/sbc.rs` — `Sbc` (sparse block codes)
- `src/cleanup.rs` — `CleanupMemory`, `Match { confidence, margin }`
- `src/resonator.rs` — `factorize()`, `ResonatorParams`, `ResonatorTrace`, `MAPI_RESONATOR_PROFILE`
- `src/decode_select.rs` — `reconstruct_factors_auto()`, `DecodeSelection`, `explain_decode_method()`
- `src/recon.rs` — `reconstruct_factors()`, `reconstruct_factors_selected()`, `reconstruct_role()`
- **Proof:** `proofs/lh-bundle/src/Bundle.hs` — Liquid Haskell MAP-I bundle capacity (Proven basis)

---

## Key types and operations

### `VsaModel` trait (`crates/mycelium-vsa/src/lib.rs:322`)

- `model_id()`, `self_inverse()`, `intrinsic_guarantee(op: VsaOp) -> GuaranteeStrength`
- `bind(&[f64], &[f64]) -> Result<Vec<f64>, VsaError>`
- `unbind(&[f64], &[f64]) -> Result<Vec<f64>, VsaError>`
- `bundle(&[&[f64]]) -> Result<Vec<f64>, VsaError>`
- `permute(&[f64], shift: i64) -> Result<Vec<f64>, VsaError>`
- `similarity(&[f64], &[f64]) -> f64` (cosine)

### `VsaOp` enum: `Bind | Unbind | Bundle | Permute`

### §4 guarantee matrix — `RFC0003_MATRIX` (`crates/mycelium-vsa/src/matrix.rs:34`)

| Model | Bind | Unbind | Bundle | Permute |
|-------|------|--------|--------|---------|
| MAP-I | Exact | Exact (self-inverse) | Proven | Exact |
| MAP-B | Exact | Exact (self-inverse) | Proven (membership-only, no deep nesting) | Exact |
| BSC | Exact | Exact (self-inverse) | Proven (on-expectation only, weaker) | Exact |
| HRR | Exact | Empirical (not self-inverse, lossy) | Empirical | Exact |
| FHRR | Exact | Empirical (not self-inverse, lossy) | Empirical | Exact |
| SBC | Proven | Proven | Proven (Bloom/Counting-Bloom) | Exact |

**§4.1 erratum (r3):** permute is Exact for ALL models. HRR/FHRR bind is Exact (exact algebraic
op); unbind is Empirical (approximate inverse needing cleanup). Source:
`crates/mycelium-vsa/src/matrix.rs:19-24`.

---

## Key invariants (honesty)

1. **`permute` is always Exact** — bijection (cyclic shift), zero approximation (RFC-0003 §4.1).
2. **HRR/FHRR unbind is Empirical** — approximate inverse, not self-inverse; "the residual weak
   link" (RR-13; RFC-0003 §4 "Net"). Single-factor at most Empirical; multi-factor needs resonator.
3. **MAP-I bundle Proven only when side-conditions satisfied:**
   `dim >= requiredDim(m, delta) = ceil((2/mu^2) * ln(m/delta))` with `mu=0.1`.
   Basis: Clarkson-Ubaru-Yang 2023 Thm 6; Thomas-Dasgupta-Rosing 2021.
   Checked in Rust: `capacity::proven_capacity_bound()` returns `None` when dim insufficient.
   LH proof: `proofs/lh-bundle/src/Bundle.hs` — Z3 discharges arithmetic instantiation only
   (theorem axiomatized from literature; `src/capacity.rs:12-15`).
4. **MAP-B bundle deep nesting refused** — reliability decays `1/2 + 1/2^r` at depth r; refused
   explicitly with `VsaError::NestedBundleUnsupported` (`src/lib.rs:121`). Never silent.
5. **BSC bundle: matrix Proven, value-level Empirical** — matrix records the literature's
   on-expectation tag (Heim / Yi & Achour); value path uses `BSC_BUNDLE_PROFILE` (Empirical,
   delta from trials). NOT a contradiction. Read `src/matrix.rs:44-50` (A3-06/C1-04).
6. **SBC Proven** — Bloom/Counting-Bloom (Clarkson Thms 22-23); first rigorous set-intersection.
7. **Never stamp Proven without checked basis** — `InsufficientCapacity` and `DuplicateBundleItems`
   errors in `MapI::bundle_values_certified()` (`src/mapi.rs:135`). VR-5, M-I2.

---

## Capacity bounds

- **MAP-I formula:** `requiredDim(m, delta) = ceil(200 * ln(m/delta))` (`mu=0.1`)
  - m=3, delta=1e-2 -> 1141; m=10, delta=1e-3 -> 1843; m=100, delta=1e-4 -> 2764
  - Source: `src/capacity.rs:26-41`; tests confirm at `:72-79`
- **Binding arity** blows up capacity super-exponentially — deep compositions cannot honestly
  carry tight Proven bounds (RFC-0003 §4 "Net").
- **`EmpiricalProfile`** (`src/lib.rs:361`) — trial-validated regime struct:
  `check(items, dim)` returns explicit `OutsideEmpiricalProfile` when outside validated range.

---

## Resonator network factorization (RFC-0009)

**Use case:** recover F unknown factors of `s = x1*...*xF` when all factors come from known
codebooks but are all unknown simultaneously (brute-force enumeration intractable).

**Algorithm** (`src/resonator.rs`): parallel/Jacobi update — update all slots against a SNAPSHOT
of previous estimates (not in-place/Gauss-Seidel). Cleanup via Hebbian bipolar
(`sign(sum_j sim_j * c_j)`) keeps estimates on +-1 alphabet so MAP-I unbind stays exact.

**Honest guarantee:** Empirical only (MAP-I/BSC with exact bind); Declared for HRR/FHRR/sparse
(lossy bind). Never Proven. Schema in `mycelium-core::recon` enforces <=Empirical ceiling
(`ReconInfo::new` / A6 checks). RFC-0009 §5.

**Validated envelope** (`MAPI_RESONATOR_PROFILE` in `src/resonator.rs`):
`F<=3, k<=16, prod_k<=4096, d>=4096`, delta=0.02 (Hebbian cleanup). Measured vs brute-force
oracle (exact-tuple recovery rate), NOT self-reported convergence rate. RFC-0009 §5.3.

**Stop reasons** (`StopReason`): `Converged` (iota stable AND every slot >= tau_lock) |
`BudgetExhausted` | `Oscillating` (limit cycle period>=2, distinct earlier tuple recurs) |
`Stalled` (stationary iota, confidence plateau below tau_lock). Only `Converged` clearing
tau_lock + confidence + margin yields factors. Everything else is an explicit error (G2).

**M-350 premature-abort fix:** stationary iota that is still sharpening is NOT a cycle. A
stationary iota (== previous sweep) keeps iterating while the lock bottleneck (min per-slot
similarity) is still rising; refuses only when climb plateaus. Source: RFC-0009 §3 + §8.1 P3.

**Manifest parameters** (RFC-0003 §6.1 r4, additive fields on `DecodeSpec`):
`cleanup` (Softmax/ArgMax), `beta` (>0), `tau_lock` (in [0,1]), `init`, `seed (u64)`.
Out-of-range -> explicit `MalformedReconstruction`, never silently accepted (G2).

---

## When to prefer MAP-I or BSC for Proven work

Use MAP-I or BSC when you need a checkable Proven capacity bound (self-inverse bind, Exact unbind,
Proven bundle iff side-conditions satisfied). Avoid HRR/FHRR for Proven work — unbind is
inherently Empirical (RR-13). For decode: brute force (Exact) when `prod_k <= 4096`; resonator
(Empirical) when in regime; explicit `Refuse` otherwise (`decode_select.rs`).

---

## Gotchas

- **Non-bipolar components** -> bind/unbind_values refuse with `NonAlphabetComponent` (A3-04;
  `src/mapi.rs:189-194`). The Exact self-inverse identity only holds on +-1.
- **DuplicateBundleItems** — Clarkson/Thomas assumes distinct atoms; duplicates inflate the
  apparent capacity, making a Proven tag unbacked. Refused (A3-03; `src/lib.rs:91-96`).
- **VSA is NEVER in the kernel** — ADR-008 boundary. Kernel only carries `Repr::Vsa`,
  `Payload::Hypervector`, ModelId registry hook. All algebra is in the submodule.
- **Resonator trace boxed** in every `VsaError::Resonator*` variant to keep the error enum small
  (`src/lib.rs:154-195`).
- **Reconstruction manifest** (RFC-0003 §6) distinguishes indexed retrieval (codebook + similarity)
  from true compositional reconstruction (role schema + algebraic inverse). NOT the same thing.
- **BSC bundle matrix tag** says Proven but the value-level path is Empirical. Not a bug; see
  invariant 5 above and `src/matrix.rs:44-50`.

---

## Read more

- `docs/rfcs/RFC-0003-VSA-Submodule-Boundary.md` — §4 matrix, §4.1 erratum, §5 sparsity, §6 manifest
- `docs/rfcs/RFC-0009-Resonator-Network-Factorization.md` — §3 algorithm, §5 guarantee, §6 never-silent
- `proofs/lh-bundle/src/Bundle.hs` — Liquid Haskell MAP-I capacity proof (Proven basis)
- `crates/mycelium-vsa/src/matrix.rs` — `RFC0003_MATRIX` const (authoritative per-op tags)
- `crates/mycelium-vsa/src/capacity.rs` — `proven_capacity_bound()` (checked instantiation)
- `crates/mycelium-vsa/src/mapi.rs` — `MapI` value-level adapters with honest Meta
