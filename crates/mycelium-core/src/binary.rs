//! Two's-complement binary integer semantics (M-120 support; extended by M-887 with fixed-width
//! multiply, M-888 with fixed-width unsigned division/remainder).
//!
//! An `n`-bit value is read **most-significant-first** as a two's-complement integer with range
//! `B_n = [−2^(n-1), 2^(n-1) − 1]` (`docs/spec/swaps/binary-ternary.md` §2). This is the binary-side
//! codec the binary↔ternary swap (M-120) uses; the balanced-ternary side lives in
//! [`crate::ternary`]. Values use `i64`, exact for every `n ≤ 64`. [`mul`] (M-887) reuses this same
//! `n ≤ 64` cap (via an `i128` intermediate product) and gives the never-silent fixed-width multiply
//! contract `mycelium-interp`'s `bin.mul` prim (RFC-0033 §4.1.2/§4.1.3; ADR-028) delegates to —
//! mirroring how [`crate::ternary::mul`] is the kernel side of the `trit.mul` prim.
//!
//! [`div_rem`] (M-888) reads its operands under the **unsigned** bitvector interpretation instead
//! ([`bits_to_uint`]/[`uint_to_bits`], not [`bits_to_int`]/[`int_to_bits`]) — RFC-0033 §4.1.1's
//! `Repr::Binary` stored value has no signedness, and §4.1.2 requires division to be a **distinct
//! named op per signedness** (unlike `add`/`sub`/`mul`/`neg`, which MAY be shared across the
//! signed/unsigned reading). This lands the unsigned reading first, under the `bin.div`/`bin.rem`
//! names the M-888 task specifies; the signed (two's-complement) `div`/`rem` ride M-767 under their
//! own distinct name (FLAGged in the `mycelium-interp` prim doc comment — `bin.*` was M-887's
//! namespace for the *signed* two's-complement multiply, so an *unsigned* op sharing it is a naming
//! tension worth a maintainer look, even though it is what the M-888 task text names).

/// The signed two's-complement value of an MSB-first bit string. The empty string is `0`.
#[must_use]
pub fn bits_to_int(bits: &[bool]) -> i64 {
    if bits.is_empty() {
        return 0;
    }
    // Unsigned magnitude, then subtract 2^n if the sign bit (the MSB) is set.
    let n = bits.len();
    let mut unsigned: i128 = 0;
    for &b in bits {
        unsigned = (unsigned << 1) | i128::from(b);
    }
    if bits[0] {
        unsigned -= 1i128 << n;
    }
    unsigned as i64
}

/// The `n`-bit two's-complement representation of `value`, MSB-first — or `None` if `value` is
/// outside `B_n` (explicit out-of-range, never a silent wrap; §2/§4 P4).
#[must_use]
pub fn int_to_bits(value: i64, n: u32) -> Option<Vec<bool>> {
    if n == 0 {
        return if value == 0 { Some(Vec::new()) } else { None };
    }
    let n = n as usize;
    let lo = -(1i128 << (n - 1));
    let hi = (1i128 << (n - 1)) - 1;
    let v = i128::from(value);
    if v < lo || v > hi {
        return None;
    }
    // Reduce mod 2^n into the unsigned range, then read bits MSB-first.
    let modulus = 1i128 << n;
    let u = v.rem_euclid(modulus);
    let mut bits = vec![false; n];
    for (i, slot) in bits.iter_mut().enumerate() {
        // bit for position (n-1-i) counting from the MSB → shift (n-1-i).
        *slot = (u >> (n - 1 - i)) & 1 == 1;
    }
    Some(bits)
}

/// The **unsigned** magnitude of an MSB-first bit string, exact for `n ≤ 64`. Unlike
/// [`bits_to_int`] (which reads the two's-complement/**signed** value), this reads the same bits as
/// an unsigned bitvector — RFC-0033 §4.1.1: `Repr::Binary`'s stored value has no signedness;
/// signedness is a property of the *op*, not the *value*. The empty string is `0`.
#[must_use]
pub fn bits_to_uint(bits: &[bool]) -> u64 {
    let mut v: u64 = 0;
    for &b in bits {
        v = (v << 1) | u64::from(b);
    }
    v
}

/// The `n`-bit **unsigned** representation of `value`, MSB-first — `None` if `value` does not fit
/// the unsigned range `[0, 2^n − 1]` (n ≤ 64) — explicit out-of-range, never a silent
/// wrap/truncation. The unsigned counterpart to [`int_to_bits`].
#[must_use]
pub fn uint_to_bits(value: u64, n: u32) -> Option<Vec<bool>> {
    if n == 0 {
        return if value == 0 { Some(Vec::new()) } else { None };
    }
    if n < 64 && value >= (1u64 << n) {
        return None;
    }
    let n = n as usize;
    let mut bits = vec![false; n];
    for (i, slot) in bits.iter_mut().enumerate() {
        *slot = (value >> (n - 1 - i)) & 1 == 1;
    }
    Some(bits)
}

/// The current [`mul`] operand-width cap (`n ≤ 64`) — exact via an `i128` intermediate product, the
/// same cap [`bits_to_int`]/[`int_to_bits`] already declare. Public so callers (the `bin.mul` prim)
/// can distinguish an over-cap *width* refusal from an in-range-width arithmetic *overflow* refusal
/// without duplicating the constant (G2 — the two refusal reasons stay honestly distinct at the
/// caller's `EvalError` layer, even though this function collapses both to `None`).
pub const MUL_MAX_WIDTH: usize = 64;

/// Two's-complement fixed-width multiply of two equal-width `n`-bit two's-complement integers
/// (MSB-first), for `n ≤ `[`MUL_MAX_WIDTH`]. `None` when `a.len() != b.len()`, `a.len() >
/// MUL_MAX_WIDTH`, or the exact product does not fit `B_n = [−2^(n-1), 2^(n-1) − 1]` — never-silent
/// fixed-width overflow (RFC-0033 §4.1.2/§4.1.3; ADR-028 — the shared, signedness-agnostic bit
/// pattern, read here under the two's-complement/signed interpretation), the same contract
/// [`crate::ternary::mul`] gives the balanced-ternary side.
///
/// **Implementation.** Both operands round-trip through [`bits_to_int`] into `i64` (exact for `n ≤
/// 64`), widen to `i128` for the multiply (`|a|,|b| ≤ 2^63 ⇒ |a·b| ≤ 2^126 « i128::MAX` — the
/// product itself never overflows `i128`), then the exact product is range-checked against `B_n`
/// before narrowing back through [`int_to_bits`]. This is exact, not an approximation: every step
/// up to the final range check is a lossless widening.
#[must_use]
pub fn mul(a: &[bool], b: &[bool]) -> Option<Vec<bool>> {
    if a.len() != b.len() || a.len() > MUL_MAX_WIDTH {
        return None;
    }
    let n = a.len() as u32;
    if n == 0 {
        return Some(Vec::new()); // B_0 = {0}; 0 * 0 = 0, trivially in range.
    }
    let av = i128::from(bits_to_int(a));
    let bv = i128::from(bits_to_int(b));
    let product = av * bv; // never overflows i128 — see the doc comment above.
    let lo = -(1i128 << (n - 1));
    let hi = (1i128 << (n - 1)) - 1;
    if product < lo || product > hi {
        return None; // the exact product does not fit B_n — never-silent overflow.
    }
    // Safe narrow: the range check above puts `product` inside B_n ⊆ i64's range (n ≤ 64).
    int_to_bits(product as i64, n)
}

/// The current [`div_rem`] operand-width cap (`n ≤ 64`) — exact via a `u64` unsigned magnitude, the
/// same cap [`bits_to_uint`]/[`uint_to_bits`] already declare. Public so callers (the `bin.div`/
/// `bin.rem` prims) can distinguish an over-cap *width* refusal from a *div-by-zero* refusal without
/// duplicating the constant (G2 — the two refusal reasons stay honestly distinct at the caller's
/// `EvalError` layer, even though this function collapses both to `None`).
pub const DIV_MAX_WIDTH: usize = 64;

/// Unsigned fixed-width division and remainder of two equal-width `n`-bit bitvectors (MSB-first),
/// for `n ≤ `[`DIV_MAX_WIDTH`]. Returns `(quotient, remainder)` such that the Euclidean identity
/// holds bit-exactly: `a == quotient * b + remainder` with `0 ≤ remainder < b` (unsigned integer
/// division — `quotient = a / b`, `remainder = a % b` — is Euclidean division for unsigned
/// operands, unlike the signed case, where truncating and Euclidean division diverge on negative
/// operands; RFC-0033 §4.1.2/§4.1.3).
///
/// `None` when `a.len() != b.len()`, `a.len() > DIV_MAX_WIDTH`, or `b` is the all-zero bitvector —
/// an explicit, never-silent div-by-zero refusal (G2), never a panic or a fabricated result.
/// Division never overflows for unsigned fixed-width operands (`quotient`/`remainder` are always
/// `< 2^n` when they exist, since `a < 2^n`), so width mismatch/cap and div-by-zero are the only
/// refusal reasons — there is no separate "overflow" case, unlike [`mul`].
#[must_use]
pub fn div_rem(a: &[bool], b: &[bool]) -> Option<(Vec<bool>, Vec<bool>)> {
    if a.len() != b.len() || a.len() > DIV_MAX_WIDTH {
        return None;
    }
    let n = a.len() as u32;
    if n == 0 {
        // B_0's only representable value is 0 (the zero-width bitvector) — 0 / 0 is div-by-zero,
        // not a special case to be silently defined away.
        return None;
    }
    let av = bits_to_uint(a);
    let bv = bits_to_uint(b);
    if bv == 0 {
        return None; // explicit div-by-zero refusal — never silent (G2).
    }
    let q = av / bv;
    let r = av % bv;
    // Safe narrow: q <= av < 2^n and r < bv <= av < 2^n, so both fit n bits.
    Some((uint_to_bits(q, n)?, uint_to_bits(r, n)?))
}
