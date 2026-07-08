# Spec — `std.math` (numeric functions over the honest numerics)

| Field | Value |
|---|---|
| **Status** | **Accepted** (2026-06-20, maintainer-ratified per DN-07 — guarantee matrix asserted in tests; open §7/§8 questions are design/scope calls, not contract violations; was *Implemented (Rust-first) — pending ratification* 2026-06-18, Draft/needs-design 2026-06-17) — the Rust-first code landed as `mycelium-std-math` (M-525, #166, Batch P5-B; guarantee matrix asserted in tests). The Mycelium-lang migration (M-502-gated) remains. |
| **Module / Ring** | `std.math` · Ring 2 (RFC-0016 §4.2) · Tier B (RFC-0016 §4.4) |
| **Tracks** | `M-525` (#166) — the Phase-5 task this spec delivers |
| **Scope** | Numeric functions over Mycelium's value model: exact integer/rational ops, comparison/aggregation, and the transcendental/approximate family (`sqrt`/`exp`/`log`/`pow`/`sin`/…). Where precision matters, results route through the verified numerics (`mycelium-numerics`, ADR-010) and **carry that crate's bound + tag**. |
| **Boundary** | Out of scope: the ε/δ **bound kernels and certificate machinery** themselves — those are `std.numerics` (M-512), the library form of ADR-010; `math` is a *consumer* of that surface, not its home. A lossy *numeric-type* conversion (e.g. `i64 → f32`) is `std.cmp`/`convert` (M-532), not a `math` op. A *representation* change is `std.swap` (M-516). VSA/dense numeric ops are `std.vsa` (M-513) / `std.dense` (M-518). |
| **Depends on** | ADR-010 (the verified-numerics foundation — the ε/δ bound kernels + shared `{ε,δ,strength}` certificate the approximate ops route through); RFC-0016 §4.1 (the contract); RFC-0001 (the value model — `Value`/`Repr`/`Meta`, the guarantee lattice §4.3, `Bound`/`BoundBasis`). |
| **Grounds on** | `mycelium-numerics` (landed: M-201 `::error` ε affine kernel, M-202 `::prob` δ union kernel, M-203 `::cert` shared `{ε,δ,strength}` certificate + tier-i checker — all Done 2026-06-09, `docs/planning/phase-2.md`); ADR-011 (every `Bound` carries a `BoundBasis`); `std.core` (M-515) `Option`/`Result`/error values. KC-3: above the kernel — no new trusted code. |

---

## 1. Summary

`std.math` is the ordinary numeric-function surface — `abs`, `min`/`max`, `pow`, `sqrt`, `exp`, `log`,
the trigonometrics, rounding — held to the §4.1 contract. Its **honesty crux** is C2 in its purest form: an
op's guarantee tag is determined by what is *established*, never pre-claimed. Exact integer/rational ops tag
`Exact`; every transcendental/approximate result **carries the ε bound and `BoundBasis` produced by the
`mycelium-numerics` ε kernel (ADR-010)** and tags at that bound's *established* strength — `Proven` only with a
cited theorem whose side-conditions are checked, otherwise honestly `Empirical` or `Declared` (VR-5). Its second
crux is C1/G2: every **domain restriction** (`sqrt` of a negative, `log` of zero, division by zero) is an
explicit `Result::Err`/refusal — **never** a NaN/Inf-by-silence. Ring 2, Tier B; it adds no trusted code (KC-3),
consuming the landed `mycelium-numerics` crate and `std.core`.

## 2. Scope & module boundary

- **In scope:** exact ops over integers/rationals (`abs`, `neg`, `add`/`sub`/`mul`, integer `min`/`max`,
  `gcd`/`lcm`, `signum`); checked division and remainder (fallible on a zero divisor); the
  transcendental/approximate family (`sqrt`, `cbrt`, `pow`/`exp`/`log`/`logb`, `sin`/`cos`/`tan` and inverses,
  `hypot`); rounding/quantization (`floor`/`ceil`/`round`/`trunc`) presented as the explicit, tagged operation
  they are; small numeric constants (`pi`, `e`) carried with their representation's tag.
- **Out of scope (and who owns it):**
  - The ε/δ **bound kernels + certificate** themselves — `std.numerics` (M-512), the ADR-010 library surface
    `math` *consumes*. `math` does not define a bound; it threads through the one `numerics` returns.
  - **Numeric-type conversion** (widen/narrow between scalar kinds, lossy or not) — `std.cmp`/`convert`
    (M-532); a lossy narrowing is *their* explicit fallible op, not a silent `math` cast.
  - **Representation change** (`f32 ⇄ bf16`, dense ⇄ packed) — `std.swap` (M-516), which emits a swap
    certificate. `math` never performs a silent repr swap.
  - VSA/HDC and dense-tensor numeric ops — `std.vsa` (M-513) / `std.dense` (M-518).
- **Ring & layering:** Ring 2 (RFC-0016 §4.2). `math` is **new library code written to the contract over
  Ring 0/1**: exact ops are thin total functions over the value model; approximate ops *wrap* the Ring-1
  `numerics` surface (the ε kernel + certificate). It is a certificate **consumer**, never a producer of new
  trusted bound code (KC-3).

## 3. Exported-op surface (design sketch)

A design sketch — enough to fix the surface and feed the guarantee matrix, not a committed grammar. Value-semantic,
immutable-by-default. Fallible ops return `Result`; an approximate op returns its result **with the attached
`Bound`/tag** (in `Meta`, per RFC-0001 §4.3 / the `bound.schema.json` projection), never a bare scalar.

```
// illustrative signatures (not a committed surface)

// --- exact, total ---
abs(x: Int)            -> Int            // Exact, total
signum(x: Int)         -> Int            // Exact, total
min(a: Int, b: Int)    -> Int            // Exact, total (ties: a stable, documented rule — never silent loss)
max(a: Int, b: Int)    -> Int            // Exact, total
gcd(a: Int, b: Int)    -> Int            // Exact, total

// --- exact, but domain-restricted -> Result (C1: explicit error, never sentinel) ---
checked_div(a: Int, b: Int) -> Result<Int,  MathErr>   // Err(DivByZero) when b == 0
checked_rem(a: Int, b: Int) -> Result<Int,  MathErr>   // Err(DivByZero) when b == 0
ratio(a: Int, b: Int)       -> Result<Rat,  MathErr>   // Err(DivByZero) when b == 0

// --- approximate: route through numerics, carry the bound + tag (C2) ---
//     `Approx<T>` = a value plus its attached { Bound (eps, norm, basis), strength } from ADR-010.
sqrt(x: Real) -> Result<Approx<Real>, MathErr>   // Err(NegativeDomain) when x < 0
log (x: Real) -> Result<Approx<Real>, MathErr>   // Err(NonPositiveDomain) when x <= 0
logb(b: Real, x: Real) -> Result<Approx<Real>, MathErr> // Err(NonPositiveDomain | BadBase)
pow (x: Real, y: Real) -> Result<Approx<Real>, MathErr>  // Err on 0^neg, neg^non-integer, overflow
exp (x: Real) -> Result<Approx<Real>, MathErr>   // Err(Overflow) past the representable range
sin (x: Real) -> Approx<Real>                    // total in domain; carries eps (range-reduction error)
cos (x: Real) -> Approx<Real>
tan (x: Real) -> Result<Approx<Real>, MathErr>   // Err(PoleDomain) at the odd multiples of pi/2
asin(x: Real) -> Result<Approx<Real>, MathErr>   // Err(OutOfDomain) when |x| > 1

// --- rounding/quantization: the operation is explicit and tagged, not a silent re-round (C1/C3) ---
round(x: Real, mode: RoundMode) -> Approx<Int>   // mode is REQUIRED & reified; EXPLAIN-able

enum MathErr { DivByZero, NegativeDomain, NonPositiveDomain, BadBase, PoleDomain, OutOfDomain, Overflow }
```

> **Note (design choice, FLAGGED §7 Q1):** the sketch shows `Approx<T>` as the carrier that pairs a value with its
> `Bound`+strength. Whether that is a distinct type, a `Meta`-attached bound on the plain value (RFC-0001 §4.3), or
> a `numerics`-owned wrapper is M-512's call, not settled here — `math` only commits to *carrying* the bound, never
> dropping it.

### 3.1. Width-generic binary arithmetic/bit-manipulation primitives (self-hosted Mycelium-lang, `lib/std/math.myc`)

A *distinct layer* from the numeric function surface above: `lib/std/math.myc` (RFC-0031 §5 D4 Tier-1) exports **width-generic, total** binary arithmetic and bit-manipulation operations over `Binary{N}`, one definition per op, monomorphized to the call-site width (M-753 / DN-42). Unsigned addition/subtraction and bit operations are `Exact` over the whole `Binary{N}` domain; binary multiply is `Exact` on in-range products (overflow is explicit refusal). Bit-count operations (`popcount`/`clz`/`ctz`) are `Exact`. Differential agreement (L1-eval ≡ L0-interp ≡ AOT) is `Empirical` (trials, `std_math.rs`). **Never-silent (G2):** overflow/underflow are explicit refusals, width-mismatch is refused.

```
// Binary addition/subtraction (unsigned, never-silent overflow/underflow)
fn badd{N}(a: Binary{N}, b: Binary{N}) => Binary{N}       // Err(Overflow) on carry-out
fn bsub{N}(a: Binary{N}, b: Binary{N}) => Binary{N}       // Err(Overflow) on borrow

// Binary bitwise logic (total, Exact)
fn band{N}(a: Binary{N}, b: Binary{N}) => Binary{N}       // bitwise AND
fn bor{N}(a: Binary{N}, b: Binary{N}) => Binary{N}        // bitwise OR
fn bxor{N}(a: Binary{N}, b: Binary{N}) => Binary{N}       // bitwise XOR
fn bnot{N}(a: Binary{N}) => Binary{N}                     // bitwise NOT (one's complement)

// Binary multiply with overflow refusal (CU-1: RFC-0033 §4.1.2)
fn bmul{N}(a: Binary{N}, b: Binary{N}) => Binary{N}       // Err(Overflow) on product > U_N (never modular wrap)

// Bit-count / manipulation operations (total, Exact)
fn bpopcount{N}(a: Binary{N}) => Binary{N}                // population count (set-bit count)
fn bclz{N}(a: Binary{N}) => Binary{N}                     // count leading zeros (N for all-zero)
fn bctz{N}(a: Binary{N}) => Binary{N}                     // count trailing zeros (N for all-zero)
```

## 4. Guarantee matrix (the load-bearing deliverable — RFC-0016 §4.5)

Rows = exported ops. To be encoded as a checked table (the RFC-0003 §4 / `bound.schema.json` template) and asserted
in tests once code lands — never prose only. **Tag legend:** `Exact` = exact result, no accuracy semantics;
`Bounded-ε` = approximate, carries an `ErrorBound{eps, norm, basis}` from the ADR-010 ε kernel and tags at the
`basis`'s *established* `strength` (`Proven`-with-checked-side-conditions / `Empirical` / `Declared` — **resolved
per build, never pre-claimed**, see below).

| Op | Guarantee tag | Fallibility (explicit error set) | Declared effects | EXPLAIN-able? |
|---|---|---|---|---|
| `abs` | `Exact` | total | none | n/a |
| `neg` | `Exact` | total | none | n/a |
| `signum` | `Exact` | total | none | n/a |
| `min` / `max` (Int) | `Exact` | total (tie rule documented, never silent loss) | none | n/a |
| `gcd` / `lcm` | `Exact` | total | none | n/a |
| `checked_div` | `Exact` | `Err(DivByZero)` | none | yes (the refusal record) |
| `checked_rem` | `Exact` | `Err(DivByZero)` | none | yes (the refusal record) |
| `ratio` (→ `Rat`) | `Exact` | `Err(DivByZero)` | none | yes (the refusal record) |
| `floor`/`ceil`/`trunc` | `Exact` | total | none | n/a |
| `round` (with `RoundMode`) | `Exact` (the *rounding* is exact under the named mode) | total in domain | none | yes (mode is reified) |
| `sqrt` | `Bounded-ε` (basis-resolved strength) | `Err(NegativeDomain)` | none | yes (bound + basis cert) |
| `cbrt` | `Bounded-ε` | total in domain | none | yes (bound + basis cert) |
| `exp` | `Bounded-ε` | `Err(Overflow)` | none | yes (bound + basis cert) |
| `log` | `Bounded-ε` | `Err(NonPositiveDomain)` | none | yes (bound + basis cert) |
| `logb` | `Bounded-ε` | `Err(NonPositiveDomain \| BadBase)` | none | yes (bound + basis cert) |
| `pow` | `Bounded-ε` | `Err(DivByZero \| OutOfDomain \| Overflow)` | none | yes (bound + basis cert) |
| `hypot` | `Bounded-ε` | `Err(Overflow)` | none | yes (bound + basis cert) |
| `sin` / `cos` | `Bounded-ε` | total in domain | none | yes (bound + basis cert) |
| `tan` | `Bounded-ε` | `Err(PoleDomain)` | none | yes (bound + basis cert) |
| `asin`/`acos` | `Bounded-ε` | `Err(OutOfDomain)` when `\|x\| > 1` | none | yes (bound + basis cert) |
| `atan`/`atan2` | `Bounded-ε` | total in domain (`atan2`: `Err(Undefined)` at `(0,0)`) | none | yes (bound + basis cert) |
| `bmul{N}` (unsigned binary multiply) | `Exact` | `Err(Overflow)` when product exceeds `U_N` — never a modular wrap | none | yes (the refusal record) |
| `bpopcount{N}` (population count) | `Exact` | total (`Binary{N}` bit count) | none | n/a |
| `bclz{N}` (count leading zeros) | `Exact` | total (`Binary{N}`, N for all-zero) | none | n/a |
| `bctz{N}` (count trailing zeros) | `Exact` | total (`Binary{N}`, N for all-zero) | none | n/a |

**Tag justification (VR-5 — downgrade rather than overclaim):**
- **`Exact` rows** carry no accuracy semantics — exact integer/rational arithmetic, or a comparison/aggregation that
  selects an existing value. `min`/`max` document a stable tie rule so a tie is never a *silent* loss of which input
  won (C1). `round` is `Exact` in the sense that the rounding is the exact image under the *named, reified* `RoundMode`
  — the mode is the inspectable artifact (C3), not a hidden default.
- **`Bounded-ε` rows** are approximate; their result is **only as strong as the bound the `mycelium-numerics` ε kernel
  establishes (M-201 `::error`; ADR-010 §1, affine arithmetic).** The carried `Bound` is `ErrorBound{eps, norm,
  basis}` and the tag is the `basis`'s `strength`:
  - `Proven` **only** when the bound's `BoundBasis = ProvenThm{citation}` and that theorem's side-conditions
    (representation/`dtype`, input range, range-reduction regime) are *checked* at the call (ADR-011; `bound.schema.json`;
    the §4.1-C2 rule). This spec **does not pre-claim `Proven`** for any transcendental — it asserts only that the op
    *carries whatever strength `numerics` certifies*.
  - `Empirical` when the basis is `EmpiricalFit{trials, method}` (a measured error profile).
  - `Declared` when the basis is `UserDeclared` — always surfaced with the "declared, unverified" marker (M-I4/VR-5).
  The exact ε magnitudes, norms, and which transcendentals reach `Proven` are **owned by ADR-010 / M-512**, not
  fabricated here (see §7 Q2).
- **No `NaN`/`Inf`-by-silence anywhere** (C1/G2): every row whose mathematical domain is restricted returns an explicit
  `Err(...)` instead of an IEEE special value. The explicit error *is* the never-silent guarantee.

## 5. §4.1 contract conformance (C1–C6)

- **C1 — never-silent (G2).** Every domain restriction is an explicit `Result::Err` from the `MathErr` set —
  `sqrt(-1) → Err(NegativeDomain)`, `log(0) → Err(NonPositiveDomain)`, `checked_div(_, 0) → Err(DivByZero)`,
  `tan` at a pole → `Err(PoleDomain)`. No op returns `NaN`, `+Inf`, a clamp, or a sentinel for an out-of-domain input.
  An overflow past the representable range is `Err(Overflow)`, not a saturating `Inf`.
- **C2 — honest per-op tag (VR-5).** This is the module's crux. Exact ops tag `Exact`; approximate ops tag at the
  *established* strength of the `Bound` the ADR-010 ε kernel returns, never above it. `Proven` is gated on a
  checked-side-condition cited theorem (per `bound.schema.json`'s `BoundBasis`); absent that, the op honestly tags
  `Empirical`/`Declared`. The spec deliberately **does not assert `Proven` for any transcendental** — it asserts the
  *routing* and the *downgrade discipline*.
- **C3 — no black boxes / EXPLAIN (SC-3/G11).** Each approximate result carries its `{eps, norm, basis, strength}`
  certificate (M-203 `::cert`), inspectable and EXPLAIN-able — *why* this bound, by which basis. Rounding carries its
  reified `RoundMode`; a domain refusal carries a diagnostic record naming the violated restriction. No opaque heuristic
  sets a user-visible numeric outcome.
- **C4 — content-addressed, value-semantic (ADR-003 / RFC-0001).** Ops are pure functions of their inputs (no hidden
  state, no ambient rounding mode); results are immutable values. The carried `Bound` is `Meta`, and metadata is **not**
  identity (ADR-003) — two values equal up to their bound are the same value.
- **C5 — above the small kernel (KC-3).** `math` adds no trusted code: exact ops are total functions over the value
  model; approximate ops consume the *already-trusted* `mycelium-numerics` ε kernel + tier-i checker (M-201/203). No
  `wild`/FFI is asserted at the `math` layer (the transcendentals are computed through the trusted numerics surface,
  not a libm shim — but the compute floor is FLAGGED §7 Q3).
- **C6 — declared, bounded effects (RFC-0014).** Every `math` op is **pure** — `effects: none` across the matrix. No
  IO, no clock, no ambient randomness, no global rounding-mode read. (A future fixed-budget iterative refinement, if
  any, would declare an `alloc`/iteration budget — FLAGGED §7 Q4.)

## 6. Grounding

- The approximate-op routing + tag discipline: **ADR-010** (Accepted) — the two bound kernels (ε affine / δ union)
  meeting at one `{ε, δ, strength}` certificate, `strength` composing by meet (weakest-wins), exactly RFC-0001's
  lattice. `math` consumes the **ε (`ErrorBound`) kernel** for transcendental round-off.
- The landed surface `math` grounds on: **`mycelium-numerics`** — M-201 `::error` (ε affine), M-203 `::cert`
  (shared certificate + tier-i checker), Done 2026-06-09 (`docs/planning/phase-2.md`). M-204 retired the
  *refusal* default for additive arithmetic — composed approximate results carry `Proven`/`Empirical`, not a blanket
  `Declared` (RFC-0001 §4.7).
- The `Bound`/`BoundBasis` shape and the `Proven`-needs-checked-side-conditions rule: **ADR-011** (every `Bound`
  carries a `BoundBasis`) and `docs/spec/schemas/bound.schema.json` (the `ErrorBound{eps, norm, basis}` +
  `BoundBasis ∈ {ProvenThm, EmpiricalFit, UserDeclared}` projection).
- The per-op contract C1–C6, the tier/ring placement, the guarantee-matrix obligation: **RFC-0016** §4.1 / §4.2 /
  §4.4 (the `math` row: "rounding/approximation carries its tag (C2); domain errors explicit") / §4.5.
- The honesty lattice + value model: **RFC-0001** (the `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` lattice §4.3,
  `Value`/`Repr`/`Meta`); **VR-5** (honest tags), **G2** (never-silent), **G11** (dual projection), **KC-3**
  (small kernel), **ADR-003** (metadata is not identity).

## 7. Open questions (FLAGGED — resolve before ratification)

- **(Q1) The `Approx<T>` carrier shape.** Does an approximate result return a distinct `Approx<Real>` type, a plain
  value with a `Meta`-attached `Bound` (RFC-0001 §4.3), or a `numerics`-owned wrapper? This is **M-512's** design
  call; `math` commits only to *never dropping the bound*. — *Disposition: defer to M-512; thread whatever it returns.*
- **(Q2) Which transcendentals reach `Proven`, and the exact ε magnitudes/norms.** The matrix tags the approximate
  family `Bounded-ε` at *basis-resolved* strength; **which** ops have a checked-side-condition cited theorem (so
  legitimately tag `Proven`) versus settle for `Empirical`, and their concrete ε/`NormKind`, are **owned by ADR-010 /
  M-512** and are **not fabricated here**. — *Disposition: the matrix's strength column is filled per-op when M-512
  lands; this spec fixes the discipline, not the numbers. Ties to RFC-0016 §8-Q3 (ergonomics vs the contract).*
- **(Q3) The transcendental compute floor — `wild`/FFI?** Are transcendentals evaluated through a pure trusted
  routine inside `mycelium-numerics`, or do they bottom out in a platform libm via `wild` (ADR-014)? If the latter,
  `math` (or `numerics`) inherits a `wild` block to inventory (LR-9), and the C5 "no `wild`" claim above narrows.
  — *Disposition: FLAGGED; ties to RFC-0016 §8-Q6 (the `wild`/FFI floor / `std-sys` split).*
- **(Q4) Iterative-refinement effect budget.** If any approximate op uses bounded iteration to tighten its ε, that
  iteration count is a declared budget (C6/RFC-0014). The matrix currently lists all ops `effects: none`; revisit if an
  op needs a refinement loop. — *Disposition: FLAGGED; keep pure unless M-512 forces a budgeted op.*
- **(Q5) `min`/`max`/`round` tie & mode policy surface.** The tie rule (`min`/`max`) and the `RoundMode` set
  (`HalfEven`/`HalfUp`/`TowardZero`/…) must be reified and EXPLAIN-able (C3), not hard-coded defaults. Whether the mode
  is a required argument (as sketched) or a reified ambient policy (the RFC-0012 lesson) is a §8-Q3 ergonomics call.
  — *Disposition: FLAGGED; default to required-explicit until the ergonomics pass.*

## Meta — changelog

- **2026-06-17 — Draft (needs-design).** Stands up the `std.math` (M-525, #166) module spec under RFC-0016 (Draft):
  Ring 2 / Tier B numeric functions over the honest numerics. Fixes the scope + boundary (consumes `std.numerics`/
  ADR-010; defers numeric-type conversion to `cmp`/`convert`, repr change to `swap`), the exported-op surface sketch
  (exact integer/rational ops; checked division; the transcendental/approximate family; reified rounding), and — the
  load-bearing deliverable — the per-op **guarantee matrix**: exact ops tag `Exact`; every transcendental carries the
  ADR-010 ε kernel's `ErrorBound`+`BoundBasis` and tags at the bound's *established* strength, **never pre-claiming
  `Proven`** (VR-5 downgrade discipline); every domain restriction (`sqrt` of negative, `log`/`logb` of non-positive,
  `tan` at a pole, division by zero, overflow) is an explicit `MathErr`, **never NaN/Inf-by-silence** (C1/G2). §4.1
  conformance (C1–C6) stated concretely; grounding traces to ADR-010/011, `mycelium-numerics` (M-201/203, Done),
  `bound.schema.json`, RFC-0016 §4.1/§4.5, RFC-0001, VR-5/G2/KC-3. Five questions FLAGGED (the `Approx` carrier shape,
  which transcendentals reach `Proven` + the exact ε numbers, the `wild`/FFI compute floor, an iterative-refinement
  budget, the tie/round-mode policy surface) — the numbers and `Proven` reachability are M-512's to fill, never
  invented here. No code; no kernel change (KC-3). Append-only.
- **2026-06-19 — §7-Q2 (ε-ownership) RESOLVED.** The `Declared` libm-floor ε (`DECLARED_FLOAT_EPS`)
  is homed in `std.numerics` (the ε-carrier module, M-512) and re-exported by `std.math` — stated in
  exactly one place (NFR-N2). The honest `Proven` magnitude stays the kernel's (ADR-010) / M-541's to
  supply (VR-5). Append-only.

- **2026-06-20 — Accepted (maintainer ratification, DN-07).** The maintainer ratified this Rust-first spec: the §4.5 guarantee matrix is asserted in tests, never-silent fallibility and honest per-op tags hold, and the open §7/§8 questions are design/scope calls, not contract violations. No guarantee tag was upgraded without a checked basis (VR-5). The Proven/Empirical rows were verified/aligned by M-377 (M-512 delivered) — dense elementwise Proven via the ADR-010 IEEE backward-error bound (finiteness side-condition guarded), accumulation rows held at Empirical; nothing upgraded on faith. Status moves *Implemented (Rust-first) — pending ratification → Accepted*. Append-only; no kernel change (KC-3).

- **2026-06-27 — Disambiguation: `lib/std/math.myc` is a *distinct* self-hosted primitive surface, not this numeric spec (M-718, rsm S2).** This spec owns the **numeric** function surface (`abs`/`sqrt`/`exp`/`log`/`sin`/… over Int/Rat/Real, with the ε-bound transcendental family). The newly-landed `lib/std/math.myc` (v0.1.0) is a *separate, narrower* artifact: a **self-hosted width-generic arithmetic/logic primitive surface** — `badd`/`bsub`/`band`/`bor`/`bxor`/`bnot` over `Binary{N}` and `tadd`/`tsub`/`tmul`/`tneg` over `Ternary{M}` — wrapping the kernel bit/trit prims (RFC-0032 §5 D2; consumed via RFC-0031 §5 D4 Tier-1). It is **width-generic** (one definition, monomorphized to the call-site width; M-753 / DN-42) and executes three-way (`crates/mycelium-l1/tests/std_math.rs`; agreement `Empirical`). It is **not** `std.math`: no transcendentals, no ε/δ, and (flagged in the nodule, VR-5) **no division and no binary multiply** — the kernel surfaces no such prim, so a self-hosted one is deferred rather than faked. Overflow/underflow/out-of-range is an explicit never-silent `Overflow` refusal (G2), never a modular wrap. This entry only **records the disambiguation**; it does not change this spec's surface, status, or the M-502-gated numeric migration. Append-only; no kernel change (KC-3).

- **2026-07-08 — Self-hosted binary multiply (`bmul{N}`) and bit-count surface now documented (CU-1/CU-6).** The `bmul{N}` surface (unsigned width-preserving multiply with never-silent `Overflow` refusal when product exceeds `U_N`) and the bit-count helpers (`bpopcount{N}`, `bclz{N}`, `bctz{N}`, all total and `Exact`) close arithmetic/logic gaps in the self-hosted primitive surface. The earlier 2026-06-27 note "**no division and no binary multiply**" is superseded: `bmul` is now available (kernel `bit.mul` prim landed, RFC-0033 §4.1.2); binary division remains deferred (kernel prims exist but the self-hosted width-generic surface awaits a future increment). Added to §3.1 surface-sketch and guarantee matrix (§4); differential agreement (L1-eval ≡ L0-interp ≡ AOT) is `Empirical` (trials, `lib/std/math.myc:81-95` + `crates/mycelium-l1/src/checkty.rs:7260-7281` + `crates/mycelium-interp/src/prims.rs:144-150`). This spec's status and scope remain unchanged. Append-only; no kernel change (KC-3).
