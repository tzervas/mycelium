//! `std.math` — Ring-2 / Tier-B numeric functions over the honest numerics (M-525).
//!
//! # Summary
//!
//! `std.math` is the ordinary numeric-function surface — `abs`, `min`/`max`, `pow`, `sqrt`, `exp`,
//! `log`, the trigonometrics, rounding — held to the RFC-0016 §4.1 contract (C1–C6).
//!
//! Its **honesty crux** is two-fold:
//!
//! 1. **C2 (VR-5):** every op's guarantee tag is determined by what is *established*, never
//!    pre-claimed. Exact integer/rational ops tag `Exact`; every approximate result carries an
//!    `ErrorBound{eps, norm, basis}` from the `mycelium-numerics` ε kernel (ADR-010) and tags at
//!    that bound's *established* strength — `Proven` **only** where a theorem's side-conditions are
//!    checked, otherwise honestly `Empirical` or `Declared`.
//!
//! 2. **C1 (G2):** every domain restriction (`sqrt` of negative, `log` of zero, division by zero)
//!    is an explicit `Result::Err` — **never** a NaN, Inf, or sentinel.
//!
//! # Architecture
//!
//! Ring 2, Tier B. Adds no trusted code (KC-3): exact ops are pure, total functions over primitive
//! Rust types; approximate ops carry the `Declared` tag because the transcendental compute floor
//! (libm / `wild` FFI) is **deferred** to the `std-sys` phylum (FLAG — see §FLAG below). The
//! `Approx<f64>` carrier is a thin view: a plain `f64` value with its attached `{Bound, strength}`
//! pair (RFC-0001 §4.3 / the `bound.schema.json` projection), **not** a new numeric type and **not**
//! a kernel change (README §5 / math.md §7-Q1 resolved by `numerics`).
//!
//! # FLAG — transcendental compute floor (§8-Q6 / M-541)
//!
//! The approximate ops (`sqrt`, `cbrt`, `exp`, `log`, `logb`, `pow`, `hypot`, `sin`, `cos`, `tan`,
//! `asin`, `acos`, `atan`, `atan2`) delegate to Rust's `f64` intrinsics, which bottom out in the
//! platform libm. This constitutes an unaudited `wild` / FFI floor (ADR-014). Per spec §5-C5 and
//! §8-Q6 (RESOLVED: the audited `wild` floor splits into a separate `std-sys` phylum, M-541), the
//! transcendental floor is **not** an audited `wild` block here — it is the unresolved M-541 work.
//! As a consequence, all approximate ops carry `Declared` strength (not `Proven` or `Empirical`) in
//! this implementation, because no audited theorem with checked side-conditions yet backs the libm
//! calls (VR-5: downgrade to stay honest, never upgrade without a checked basis).
//!
//! # Guarantee matrix
//!
//! Encoded as data in [`GUARANTEE_MATRIX`] and asserted in the test suite — never prose-only
//! (RFC-0016 §4.5).
//!
//! # Contract conformance (RFC-0016 §4.1 C1–C6)
//!
//! - **C1 never-silent (G2):** every domain restriction is an explicit `Err(MathErr::…)` — no
//!   NaN, no ±Inf, no sentinel, no silent clamp.
//! - **C2 honest per-op tag (VR-5):** exact ops tag `Exact`; approximate ops tag `Declared` (the
//!   honest floor for a `wild`/libm-backed compute, pending M-541 auditing).
//! - **C3 no black boxes / EXPLAIN (SC-3/G11):** approximate results carry their `Bound`
//!   inspectable via [`Approx::explain`]; rounding carries its reified [`RoundMode`]; domain
//!   refusals carry a diagnostic string naming the violated restriction.
//! - **C4 value-semantic:** all ops are pure functions of their inputs; results are immutable.
//! - **C5 above the kernel (KC-3):** no `unsafe`, no FFI — the transcendental libm floor is
//!   reached via Rust's own `f64::sqrt` etc., which is not a new `wild` block introduced *here*;
//!   see the FLAG above for the M-541 disposition.
//! - **C6 declared bounded effects:** every op is pure — `effects: none`. No IO, no clock, no
//!   ambient rounding mode, no global state.
//!
//! Design spec: `docs/spec/stdlib/math.md`; task M-525, issue #166.
#![forbid(unsafe_code)]

pub mod approx;
pub mod exact;
pub mod matrix;

pub use approx::{Approx, ApproxExplain};
pub use exact::RoundMode;
pub use matrix::{assert_matrix_invariants, GuaranteeRow, GUARANTEE_MATRIX};

// ---- MathErr ----------------------------------------------------------------

/// The explicit error set for fallible `std.math` ops (spec §3; C1 / G2).
///
/// Every domain restriction surfaces as one of these variants — **never** as a NaN, ±Inf,
/// sentinel, or silent clamp. The variant name is the EXPLAIN artifact for a refusal (C3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathErr {
    /// Division (or ratio construction) with a zero divisor.
    DivByZero,
    /// Argument is negative where it must be non-negative (e.g. `sqrt(x < 0)`).
    NegativeDomain,
    /// Argument is non-positive where it must be strictly positive (e.g. `log(x ≤ 0)`).
    NonPositiveDomain,
    /// Base argument is invalid for `logb` (e.g. base ≤ 0, base = 1, or base is NaN/Inf).
    BadBase,
    /// Argument is at a pole of the function (e.g. `tan` at an odd multiple of π/2).
    PoleDomain,
    /// Argument is outside the function's domain (e.g. `asin(|x| > 1)`).
    OutOfDomain,
    /// Result magnitude exceeds the representable range of `f64`.
    Overflow,
}

impl core::fmt::Display for MathErr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            MathErr::DivByZero => "DivByZero: divisor is zero",
            MathErr::NegativeDomain => {
                "NegativeDomain: argument must be non-negative (e.g. sqrt requires x >= 0)"
            }
            MathErr::NonPositiveDomain => {
                "NonPositiveDomain: argument must be strictly positive (e.g. log requires x > 0)"
            }
            MathErr::BadBase => {
                "BadBase: logarithm base must be > 0, != 1, and finite (e.g. logb(b, x))"
            }
            MathErr::PoleDomain => {
                "PoleDomain: argument is at a pole of the function (e.g. tan at pi/2 + n*pi)"
            }
            MathErr::OutOfDomain => {
                "OutOfDomain: argument is outside the function's domain (e.g. asin requires |x| <= 1)"
            }
            MathErr::Overflow => {
                "Overflow: result magnitude exceeds the representable range of f64"
            }
        };
        f.write_str(s)
    }
}

impl std::error::Error for MathErr {}
