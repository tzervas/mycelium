//! Two's-complement binary integer semantics (M-120 support; extended by M-887 with fixed-width
//! multiply, M-888 with fixed-width unsigned division/remainder, M-889 with fixed-width logical
//! shifts).
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
//!
//! [`shl`]/[`shr`] (M-889) round out the signedness-split `shift` op set (RFC-0033 §4.1.2) with the
//! **logical** (unsigned) reading — same unsigned bitvector codec as [`div_rem`]. Shift-amount `>=`
//! width is an explicit never-silent refusal (never UB, wrap, or a silently-zeroed result); the
//! **arithmetic** (sign-extending) right shift is the distinct signed op M-767 lands later.
//!
//! [`add`]/[`sub`]/[`neg`] (M-766, `enb` Gap B) complete the **shared** two's-complement arithmetic
//! set RFC-0033 §4.1.2 names (`add`/`sub`/`mul`/`neg` — identical bit pattern across the signed/
//! unsigned reading, so they MAY be a single named op each; ADR-028). They read operands through the
//! same signed [`bits_to_int`]/[`int_to_bits`] codec [`mul`] already uses, and refuse an out-of-`B_n`
//! result explicitly — never a silent wrap. **Inventory note (verified against the registry before
//! landing these, per the M-766 task):** the pre-existing `bit.add`/`bit.sub` (kpr/E19-1, RFC-0032
//! D2) are a **different, unsigned-committed** family — their overflow criterion is unsigned
//! carry/borrow-out, which *under-refuses* relative to the signed range `B_n` (e.g. at `Binary{4}`,
//! `5 + 3 = 8` is unsigned-in-range `[0,15]` but signed-out-of-range `B_4 = [-8,7]`), so they cannot
//! stand in for the two's-complement `add`/`sub` this RFC names. [`mul`] (M-887) already established
//! the `bin.*`-namespaced, signed-committed reading this module follows; [`add`]/[`sub`] complete
//! that pair and [`neg`] is the genuinely-missing fourth member (there is no existing unsigned
//! "negate" to reconcile against — negation is inherently a signed concept).
//!
//! [`div_signed`]/[`rem_signed`]/[`shr_signed`]/[`cmp_signed`] (M-767, `enb` Gap B) complete the
//! **signedness-split** op set RFC-0033 §4.1.2 requires as *distinct named ops* — the signed
//! counterparts to the unsigned [`div_rem`] and the logical [`shr`], plus the two's-complement
//! ordering (ADR-028: signedness is a property of the *operation*, not the stored bitvector; its
//! Consequences section pins the SMT-LIB alignment — split `bvsdiv`/`bvudiv`, `bvslt`/`bvult`,
//! `bvashr`/`bvlshr`). **Rounding convention (grounding, VR-5):** RFC-0033 §4.1.2/§4.1.3 requires
//! the signedness *split* and never-silent overflow but does not literally pin the signed-division
//! rounding; the implemented convention is **truncation toward zero** with the remainder's sign
//! following the dividend (`a == q·b + r`, `|r| < |b|`) — the ADR-028-cited SMT-LIB `bvsdiv`/
//! `bvsrem` semantics (also Rust's `/`/`%`). That choice is `Declared` against the RFC text alone
//! and grounded by the ADR-028 SMT-LIB citation — FLAGged in the M-767 report for ratification,
//! never silently chosen. The single signed-division **overflow** case (`B_n`'s minimum ÷ −1: the
//! true quotient `+2^(n-1)` exceeds `B_n`'s maximum) is an explicit refusal, never an
//! SMT-LIB-style wrap — §4.1.3's never-silent overflow rule outranks the wrap SMT-LIB defines
//! there.

use core::cmp::Ordering;

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

/// The current [`shl`]/[`shr`] operand-width cap (`n ≤ 64`) — exact via a `u64`/`u128` unsigned
/// intermediate, the same cap [`bits_to_uint`]/[`uint_to_bits`] already declare. Public so callers
/// (the `bin.shl`/`bin.shr` prims) can distinguish an over-cap *width* refusal from an
/// out-of-range *shift-amount* refusal without duplicating the constant (G2 — the two refusal
/// reasons stay honestly distinct at the caller's `EvalError` layer, mirroring [`MUL_MAX_WIDTH`]/
/// [`DIV_MAX_WIDTH`]).
pub const SHIFT_MAX_WIDTH: usize = 64;

/// **Logical** (unsigned) fixed-width left shift of an `n`-bit bitvector (MSB-first) by a
/// **shift-amount operand of the same width and shape** — `shift` is itself read as an unsigned
/// `n`-bit bitvector (via [`bits_to_uint`]), for `n ≤ `[`SHIFT_MAX_WIDTH`]. Bits shifted past the
/// MSB are dropped (never wrapped/rotated) and zero bits are shifted in at the LSB — the unsigned/
/// shared reading (RFC-0033 §4.1.2's `shift` op set; the **arithmetic** (sign-extending) right
/// shift is a *distinct* signed op deferred to M-767, per §4.1.2's signedness-split requirement).
///
/// `None` when `a.len() != shift.len()`, `a.len() > SHIFT_MAX_WIDTH`, or the shift amount is `>=`
/// the width `n` — an explicit, never-silent out-of-range-shift-amount refusal (G2), never UB, a
/// silent wrap-around/modulo of the shift amount, or a silently-zeroed result. (At `n == 0` the
/// only representable shift amount is `0`, and `0 >= 0`, so `n == 0` always refuses — mirroring
/// [`div_rem`]'s `n == 0` div-by-zero refusal.)
#[must_use]
pub fn shl(a: &[bool], shift: &[bool]) -> Option<Vec<bool>> {
    if a.len() != shift.len() || a.len() > SHIFT_MAX_WIDTH {
        return None;
    }
    let n = a.len() as u32;
    if n == 0 {
        return None;
    }
    let k = bits_to_uint(shift);
    if k >= u64::from(n) {
        return None; // shift-amount >= width — explicit, never-silent refusal (G2).
    }
    // Widen to u128 before shifting so bits shifted past bit 63 (at n == 64) don't overflow/panic;
    // `av < 2^n` and `k < n <= 64`, so `av << k < 2^(2n) <= 2^128`, always fits u128.
    let av = u128::from(bits_to_uint(a));
    let modulus = 1u128 << n; // n <= 64, so this is exact in u128.
    let result = ((av << k) % modulus) as u64; // < modulus <= 2^64, exact narrow.
    uint_to_bits(result, n)
}

/// **Logical** (unsigned, zero-filling) fixed-width right shift — the counterpart to [`shl`], same
/// operand shape (`shift` is an unsigned `n`-bit bitvector) and the same cap/refusal contract:
/// `None` on a width mismatch, an over-cap width, or a shift amount `>= n` (never UB/wrap/silent,
/// including at `n == 0`). Bits shifted past the LSB are dropped; zero bits are shifted in at the
/// MSB. (The **arithmetic**/sign-extending right shift is the distinct signed op deferred to
/// M-767.)
#[must_use]
pub fn shr(a: &[bool], shift: &[bool]) -> Option<Vec<bool>> {
    if a.len() != shift.len() || a.len() > SHIFT_MAX_WIDTH {
        return None;
    }
    let n = a.len() as u32;
    if n == 0 {
        return None;
    }
    let k = bits_to_uint(shift);
    if k >= u64::from(n) {
        return None; // shift-amount >= width — explicit, never-silent refusal (G2).
    }
    let av = bits_to_uint(a);
    let result = av >> k; // k < n <= 64, safe unsigned/logical right shift.
    uint_to_bits(result, n)
}

/// The current [`add`]/[`sub`]/[`neg`] operand-width cap (`n ≤ 64`) — exact via the `i64` two's-
/// complement codec ([`bits_to_int`]/[`int_to_bits`]), the same cap those functions already declare.
/// Shared by all three two's-complement arithmetic ops, mirroring how [`DIV_MAX_WIDTH`] is shared by
/// [`div_rem`]'s quotient/remainder pair and [`SHIFT_MAX_WIDTH`] by [`shl`]/[`shr`].
pub const TC_MAX_WIDTH: usize = 64;

/// Two's-complement fixed-width add of two equal-width `n`-bit two's-complement integers (MSB-first),
/// for `n ≤ `[`TC_MAX_WIDTH`]. `None` when `a.len() != b.len()`, `a.len() > TC_MAX_WIDTH`, or the
/// exact sum does not fit `B_n = [−2^(n-1), 2^(n-1) − 1]` — never-silent fixed-width overflow
/// (RFC-0033 §4.1.2/§4.1.3), mirroring [`mul`]'s contract. `a`/`b` are widened to `i128` before
/// summing so the addition itself never overflows the intermediate (`|a|,|b| ≤ 2^63` ⇒ `|a+b| ≤
/// 2^64 « i128::MAX`); only the final range check against `B_n` can refuse.
#[must_use]
pub fn add(a: &[bool], b: &[bool]) -> Option<Vec<bool>> {
    if a.len() != b.len() || a.len() > TC_MAX_WIDTH {
        return None;
    }
    let n = a.len() as u32;
    if n == 0 {
        return Some(Vec::new()); // B_0 = {0}; 0 + 0 = 0, trivially in range.
    }
    let av = i128::from(bits_to_int(a));
    let bv = i128::from(bits_to_int(b));
    let sum = av + bv; // never overflows i128 — see the doc comment above.
    let lo = -(1i128 << (n - 1));
    let hi = (1i128 << (n - 1)) - 1;
    if sum < lo || sum > hi {
        return None; // the exact sum does not fit B_n — never-silent overflow.
    }
    // Safe narrow: the range check above puts `sum` inside B_n ⊆ i64's range (n ≤ 64).
    int_to_bits(sum as i64, n)
}

/// Two's-complement fixed-width subtract (`a − b`) of two equal-width `n`-bit two's-complement
/// integers (MSB-first), for `n ≤ `[`TC_MAX_WIDTH`]. Same never-silent contract as [`add`]: `None` on
/// a width mismatch/over-cap width, or when the exact difference does not fit `B_n`.
#[must_use]
pub fn sub(a: &[bool], b: &[bool]) -> Option<Vec<bool>> {
    if a.len() != b.len() || a.len() > TC_MAX_WIDTH {
        return None;
    }
    let n = a.len() as u32;
    if n == 0 {
        return Some(Vec::new()); // B_0 = {0}; 0 - 0 = 0, trivially in range.
    }
    let av = i128::from(bits_to_int(a));
    let bv = i128::from(bits_to_int(b));
    let diff = av - bv; // never overflows i128 — |av|,|bv| <= 2^63.
    let lo = -(1i128 << (n - 1));
    let hi = (1i128 << (n - 1)) - 1;
    if diff < lo || diff > hi {
        return None; // the exact difference does not fit B_n — never-silent overflow.
    }
    int_to_bits(diff as i64, n)
}

/// Two's-complement fixed-width negate (`−a`) of an `n`-bit two's-complement integer (MSB-first), for
/// `n ≤ `[`TC_MAX_WIDTH`]. `None` when `a.len() > TC_MAX_WIDTH`, or when the exact negation does not
/// fit `B_n` — the classic two's-complement negate-overflow edge: `B_n`'s minimum value `−2^(n-1)`
/// has no positive counterpart in `B_n` (its magnitude `2^(n-1)` exceeds the maximum `2^(n-1) − 1`),
/// so negating it is an explicit refusal, never a silent wrap back to itself. The genuinely-missing
/// member of the shared two's-complement set (§4.1.2) — `add`/`sub`/`mul` were already landed (`kpr`'s
/// unsigned `bit.add`/`bit.sub`, reconciled as a *different* family in the module doc comment above,
/// and M-887's signed [`mul`]); there is no pre-existing "negate" to reconcile against.
#[must_use]
pub fn neg(a: &[bool]) -> Option<Vec<bool>> {
    if a.len() > TC_MAX_WIDTH {
        return None;
    }
    let n = a.len() as u32;
    if n == 0 {
        return Some(Vec::new()); // B_0 = {0}; -0 = 0, trivially in range.
    }
    let av = i128::from(bits_to_int(a));
    let negated = -av; // never overflows i128 — |av| <= 2^63.
    let lo = -(1i128 << (n - 1));
    let hi = (1i128 << (n - 1)) - 1;
    if negated < lo || negated > hi {
        return None; // the exact negation does not fit B_n — never-silent overflow (the MIN case).
    }
    int_to_bits(negated as i64, n)
}

/// Validate the shared signed-division operand contract and decode: equal widths, `n ≤
/// `[`DIV_MAX_WIDTH`]` (the same cap as the unsigned [`div_rem`] — both pairs ride the `i64`/`u64`
/// codecs' exactness bound), `n > 0`, and a nonzero divisor. Returns the two's-complement operand
/// values and the width; `None` on any violated precondition (the caller's `EvalError` layer keeps
/// the refusal reasons honestly distinct by re-checking the cheap ones — G2, mirroring how
/// [`DIV_MAX_WIDTH`]'s doc comment splits the unsigned pair's reasons).
fn signed_div_operands(a: &[bool], b: &[bool]) -> Option<(i64, i64, u32)> {
    if a.len() != b.len() || a.len() > DIV_MAX_WIDTH {
        return None;
    }
    let n = a.len() as u32;
    if n == 0 {
        // B_0's only representable value is 0 — 0 / 0 is div-by-zero, exactly as in `div_rem`.
        return None;
    }
    let bv = bits_to_int(b);
    if bv == 0 {
        return None; // explicit div-by-zero refusal — never silent (G2).
    }
    Some((bits_to_int(a), bv, n))
}

/// **Signed** (two's-complement) fixed-width division of two equal-width `n`-bit two's-complement
/// integers (MSB-first), for `n ≤ `[`DIV_MAX_WIDTH`]` — the distinct-named signed counterpart to
/// the unsigned [`div_rem`] quotient (RFC-0033 §4.1.2/§4.1.3; ADR-028; M-767). The quotient is
/// **truncated toward zero** (SMT-LIB `bvsdiv`; Rust `/` — see the module doc's rounding-convention
/// note): `-7 / 2 = -3`, not the floored `-4`.
///
/// `None` when `a.len() != b.len()`, `a.len() > DIV_MAX_WIDTH`, `b` is zero (explicit div-by-zero,
/// never a panic — G2), **or the quotient overflows `B_n`** — unlike unsigned division, signed
/// fixed-width division has exactly one overflow case: `B_n`'s minimum `−2^(n-1)` divided by `−1`
/// has true quotient `+2^(n-1)`, which exceeds `B_n`'s maximum `2^(n-1) − 1`. That case is an
/// explicit, never-silent refusal (RFC-0033 §4.1.3), never a wrap back to the minimum.
#[must_use]
pub fn div_signed(a: &[bool], b: &[bool]) -> Option<Vec<bool>> {
    let (av, bv, n) = signed_div_operands(a, b)?;
    // Truncated division in i128 — |av| ≤ 2^63 and |bv| ≥ 1, so the quotient never overflows the
    // intermediate; only the final B_n range check (inside `int_to_bits`) can refuse, and the only
    // pair it refuses is (min, −1) → +2^(n-1) (see the doc comment above).
    let q = i128::from(av) / i128::from(bv);
    let q64 = i64::try_from(q).ok()?; // out of i64 only for (i64::MIN, -1) at n = 64.
    int_to_bits(q64, n)
}

/// **Signed** (two's-complement) fixed-width remainder — the companion to [`div_signed`], with the
/// remainder's **sign following the dividend** (SMT-LIB `bvsrem`; Rust `%`): together they satisfy
/// the truncated-division identity `a == div_signed(a,b)·b + rem_signed(a,b)` with `|r| < |b|`
/// wherever the quotient exists (property-tested).
///
/// `None` on the same operand preconditions as [`div_signed`] (width mismatch, over-cap width,
/// div-by-zero). Unlike the quotient, the remainder **never overflows**: `|r| < |b| ≤ 2^(n-1)`
/// puts every remainder inside `B_n`, so `rem_signed(min, −1) = Some(0)` succeeds even though
/// `div_signed(min, −1)` refuses — the exact result fits, and refusing an in-range exact result
/// would be an over-refusal §4.1.3 does not ask for (the divergence from Rust's `checked_rem`,
/// which refuses on the hardware's paired-instruction behavior, is deliberate and documented).
#[must_use]
pub fn rem_signed(a: &[bool], b: &[bool]) -> Option<Vec<bool>> {
    let (av, bv, n) = signed_div_operands(a, b)?;
    // Truncated remainder in i128 (sign follows the dividend). Always in B_n: |r| < |bv| ≤ 2^(n-1)
    // bounds r to [−2^(n-1)+1, 2^(n-1)−1] ⊂ B_n, so the narrowing below cannot refuse.
    let r = i128::from(av) % i128::from(bv);
    int_to_bits(r as i64, n)
}

/// **Arithmetic** (sign-extending) fixed-width right shift — the distinct-named signed counterpart
/// to the logical [`shr`] (RFC-0033 §4.1.2's signedness-split `shift` op set; ADR-028's SMT-LIB
/// `bvashr`/`bvlshr` split; M-767). Same operand shape and refusal contract as [`shr`]: the shift
/// amount is itself an unsigned `n`-bit bitvector (read via [`bits_to_uint`]), and `None` on a
/// width mismatch, an over-cap width (`n > `[`SHIFT_MAX_WIDTH`]`), or a shift amount `>= n`
/// (explicit, never-silent — never UB, a wrapped shift amount, or an implicit "fill with the sign
/// bit for any amount" extension; at `n == 0` every amount refuses, as in [`shr`]).
///
/// Copies of the **sign bit** (the MSB) are shifted in where [`shr`] shifts in zeros, so the
/// result equals `⌊a / 2^k⌋` under the two's-complement reading (floor, i.e. toward −∞ — the
/// standard arithmetic-shift semantics; e.g. `-1 >> k = -1` for every in-range `k`). The result
/// always fits `B_n` (a magnitude never grows under an arithmetic right shift), so shifting never
/// overflows — the refusal reasons are exactly the operand preconditions above.
#[must_use]
pub fn shr_signed(a: &[bool], shift: &[bool]) -> Option<Vec<bool>> {
    if a.len() != shift.len() || a.len() > SHIFT_MAX_WIDTH {
        return None;
    }
    let n = a.len() as u32;
    if n == 0 {
        return None;
    }
    let k = bits_to_uint(shift);
    if k >= u64::from(n) {
        return None; // shift-amount >= width — explicit, never-silent refusal (G2).
    }
    let av = bits_to_int(a);
    let result = av >> k; // k < n <= 64 ⇒ k <= 63: a defined, arithmetic (sign-extending) shift.
    int_to_bits(result, n)
}

/// **Signed** (two's-complement) total order over two equal-width bitvectors — the distinct-named
/// signed counterpart to the D1 comparison's unsigned-magnitude order on `Binary` (RFC-0033
/// §4.1.2: ordering differs by signedness, so it MUST be a distinct named op; ADR-028's
/// `bvslt`/`bvult` split; M-767). `None` on a width mismatch (the caller refuses explicitly —
/// never a silent ordering); equal-width operands always order (`Some`), the zero-width bitvector
/// comparing `Equal` (`B_0 = {0}`).
///
/// **Width-unbounded and purely structural** (no `DIV_MAX_WIDTH`-style cap): the two's-complement
/// order is the unsigned lexicographic order with the sign bit's polarity flipped — compare
/// `(¬a[0], a[1..])` against `(¬b[0], b[1..])` — so no integer decode (and hence no width cap)
/// is involved, exactly as the unsigned D1 order compares the raw bits. Agreement with
/// [`bits_to_int`]'s value order is property-tested on the decodable domain (`n ≤ 64`).
#[must_use]
pub fn cmp_signed(a: &[bool], b: &[bool]) -> Option<Ordering> {
    if a.len() != b.len() {
        return None;
    }
    if a.is_empty() {
        return Some(Ordering::Equal);
    }
    // Flip the sign bit (negatives sort below non-negatives), then MSB-first lexicographic.
    Some((!a[0], &a[1..]).cmp(&(!b[0], &b[1..])))
}
