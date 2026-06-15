//! Balanced-ternary integer semantics and arithmetic (M-111; FR-M2).
//!
//! A [`Trit`] is a digit in `{−1, 0, +1}`. An `m`-trit balanced-ternary number with digits written
//! **most-significant-first** `⟨t₀ … t_{m-1}⟩` denotes the integer
//! `value(t) = Σⱼ digit(tⱼ)·3^(m-1-j)` (`docs/spec/swaps/binary-ternary.md` §1). This module is the
//! single home for the codec (`int ↔ trits`) and the digit-wise arithmetic; it is reused by the
//! reference interpreter's `trit.*` primitives (M-111) and by the binary↔ternary swap (M-120).
//!
//! Two identities the spec calls out (§1) hold by construction here and are oracle-tested:
//! **negation = digit-wise sign flip** ([`neg`]) and the symmetric range `[−(3^m−1)/2, (3^m−1)/2]`
//! ([`max_magnitude`]). Arithmetic is **fixed-width**: a result outside the range is an explicit
//! `None`/overflow — never a silent wrap (SC-3; G2).
//!
//! Integer values use `i64`; this is exact for every width up to `m = 40` (`(3^40−1)/2 < i64::MAX`),
//! far beyond the small widths exercised here. Larger widths are out of scope until a bignum need
//! appears.

use crate::value::Trit;

/// The signed value of a single trit.
#[must_use]
pub fn digit(t: Trit) -> i64 {
    match t {
        Trit::Neg => -1,
        Trit::Zero => 0,
        Trit::Pos => 1,
    }
}

fn from_digit(d: i64) -> Trit {
    // C1-05: every caller normalizes into the balanced-ternary digit domain `{−1, 0, +1}` before
    // reaching here — `int_to_trits` folds the `r == 2` carry to `−1`, and `add`'s `(s+1).rem_euclid(3) − 1`
    // is provably in `[−1, +1]`. So `_ => Zero` is never taken on a well-formed call; the
    // `debug_assert!` documents and (in debug builds) checks that domain invariant without a
    // release-build panic in the trusted kernel. A stray out-of-domain digit maps to `Zero`
    // (the additive identity) rather than wrapping silently — still sound, never undefined.
    match d {
        -1 => Trit::Neg,
        1 => Trit::Pos,
        0 => Trit::Zero,
        _ => {
            debug_assert!(false, "balanced-ternary digit out of range: {d}");
            Trit::Zero
        }
    }
}

/// The maximum representable magnitude in `m` trits: `(3^m − 1) / 2`. The range is the symmetric
/// `[−max, +max]`. Returns `None` if `3^m` would overflow `i64` (`m ≥ 41`).
#[must_use]
pub fn max_magnitude(m: u32) -> Option<i64> {
    let mut pow: i64 = 1;
    for _ in 0..m {
        pow = pow.checked_mul(3)?;
    }
    Some((pow - 1) / 2)
}

/// The integer denoted by an MSB-first trit string (`value(t)`, §1). The empty string is `0`.
#[must_use]
pub fn trits_to_int(trits: &[Trit]) -> i64 {
    // Horner from the most-significant digit: v = v·3 + dⱼ.
    trits.iter().fold(0i64, |acc, &t| acc * 3 + digit(t))
}

/// The unique `m`-trit balanced representation of `value`, MSB-first — or `None` if `value` lies
/// outside the `m`-trit range (an explicit out-of-range result, never a silent truncation; §3.1).
#[must_use]
pub fn int_to_trits(value: i64, m: u32) -> Option<Vec<Trit>> {
    let mut v = value;
    let mut lsb_first = Vec::with_capacity(m as usize);
    for _ in 0..m {
        // Balanced remainder in {−1, 0, +1}: take r ∈ {0,1,2} then fold 2 ≡ −1 (carry up).
        let mut r = v.rem_euclid(3);
        v = v.div_euclid(3);
        if r == 2 {
            r = -1;
            v += 1; // borrow: 2 ≡ −1 (mod 3)
        }
        lsb_first.push(from_digit(r));
    }
    if v != 0 {
        return None; // value did not fit in m trits — out of range
    }
    lsb_first.reverse(); // to MSB-first
    Some(lsb_first)
}

/// Digit-wise negation: `value(neg t) = −value(t)` exactly (balanced ternary is sign-symmetric, §1).
/// Width-preserving and always in range.
#[must_use]
pub fn neg(trits: &[Trit]) -> Vec<Trit> {
    trits
        .iter()
        .map(|&t| match t {
            Trit::Neg => Trit::Pos,
            Trit::Zero => Trit::Zero,
            Trit::Pos => Trit::Neg,
        })
        .collect()
}

/// Ripple-carry add over two equal-length MSB-first trit strings, fixed-width. Returns `None` on
/// overflow (a non-zero final carry), i.e. when the true sum leaves the `m`-trit range — explicit,
/// never a silent wrap.
#[must_use]
pub fn add(a: &[Trit], b: &[Trit]) -> Option<Vec<Trit>> {
    if a.len() != b.len() {
        return None;
    }
    let m = a.len();
    let mut out = vec![Trit::Zero; m];
    let mut carry: i64 = 0;
    // Process least-significant first (the tail of an MSB-first string).
    for i in (0..m).rev() {
        let s = digit(a[i]) + digit(b[i]) + carry;
        // Normalize s ∈ [−3,3] into a balanced digit + carry.
        let d = (s + 1).rem_euclid(3) - 1;
        carry = (s + 1).div_euclid(3);
        out[i] = from_digit(d);
    }
    if carry != 0 {
        return None; // out of m-trit range
    }
    Some(out)
}

/// Fixed-width subtraction `a − b` = `add(a, neg(b))`.
#[must_use]
pub fn sub(a: &[Trit], b: &[Trit]) -> Option<Vec<Trit>> {
    if a.len() != b.len() {
        return None;
    }
    add(a, &neg(b))
}

/// Fixed-width multiplication. Computes the full product by shifted accumulation (independent of
/// machine integer multiply) in a `2m`-trit buffer, then returns the low `m` trits iff the high
/// trits are all zero — otherwise `None` (overflow, explicit).
#[must_use]
pub fn mul(a: &[Trit], b: &[Trit]) -> Option<Vec<Trit>> {
    if a.len() != b.len() {
        return None;
    }
    let m = a.len();
    if m == 0 {
        return Some(Vec::new());
    }
    let wide = 2 * m;
    let mut acc = vec![Trit::Zero; wide];
    // For each digit of b (power k, counting from the LSB), add ±(a << k) into the accumulator.
    for (k, &bk) in b.iter().rev().enumerate() {
        let factor = digit(bk);
        if factor == 0 {
            continue;
        }
        // a, possibly negated, placed at positions [k, k+m) of an LSB-first buffer.
        let a_signed: Vec<Trit> = if factor < 0 { neg(a) } else { a.to_vec() };
        let mut partial_lsb = vec![Trit::Zero; wide];
        for (j, &t) in a_signed.iter().rev().enumerate() {
            partial_lsb[k + j] = t;
        }
        // Add partial (LSB-first) into acc (LSB-first) — reuse the MSB-first adder via reversal.
        let mut acc_msb: Vec<Trit> = acc.iter().rev().copied().collect();
        let partial_msb: Vec<Trit> = partial_lsb.iter().rev().copied().collect();
        // The 2m-wide sum cannot overflow 2m trits for m-trit operands, so add() is total here.
        acc_msb = add(&acc_msb, &partial_msb)?;
        acc = acc_msb.iter().rev().copied().collect();
    }
    // acc is LSB-first, width 2m. The product fits in m trits iff positions [m, 2m) are all zero.
    if acc[m..].iter().any(|&t| t != Trit::Zero) {
        return None; // overflow
    }
    let low_msb: Vec<Trit> = acc[..m].iter().rev().copied().collect();
    Some(low_msb)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Walk every integer representable in `m` trits, paired with its codec encoding.
    fn each_in_range(m: u32, mut f: impl FnMut(i64, Vec<Trit>)) {
        let max = max_magnitude(m).unwrap();
        for v in -max..=max {
            f(v, int_to_trits(v, m).expect("in range"));
        }
    }

    #[test]
    fn worked_example_matches_spec() {
        // binary-ternary.md §5: −78 in 6 trits is ⟨0,−1,0,0,+1,0⟩.
        let t = int_to_trits(-78, 6).unwrap();
        assert_eq!(
            t,
            vec![
                Trit::Zero,
                Trit::Neg,
                Trit::Zero,
                Trit::Zero,
                Trit::Pos,
                Trit::Zero
            ]
        );
        assert_eq!(trits_to_int(&t), -78);
    }

    #[test]
    fn range_is_symmetric() {
        assert_eq!(max_magnitude(1), Some(1));
        assert_eq!(max_magnitude(6), Some(364)); // (3^6−1)/2
        assert_eq!(int_to_trits(365, 6), None); // just past the max → out of range
        assert_eq!(int_to_trits(-365, 6), None);
    }

    #[test]
    fn codec_round_trips_exhaustively() {
        for m in 1..=5 {
            each_in_range(m, |v, t| {
                assert_eq!(t.len(), m as usize);
                assert_eq!(trits_to_int(&t), v, "round-trip at m={m}");
            });
        }
    }

    #[test]
    fn neg_is_value_negation() {
        for m in 1..=5 {
            each_in_range(m, |v, t| {
                assert_eq!(trits_to_int(&neg(&t)), -v, "neg at m={m}");
            });
        }
    }

    /// **Oracle property test (add):** the digit-wise ripple-carry adder agrees with the `i64`
    /// oracle for *every* pair at small widths — in range it equals the encoded sum, out of range
    /// it is `None`.
    #[test]
    fn add_matches_integer_oracle() {
        for m in 1..=4 {
            let max = max_magnitude(m).unwrap();
            for x in -max..=max {
                for y in -max..=max {
                    let a = int_to_trits(x, m).unwrap();
                    let b = int_to_trits(y, m).unwrap();
                    let got = add(&a, &b);
                    let expected = x + y;
                    if expected.abs() <= max {
                        assert_eq!(got, int_to_trits(expected, m), "add {x}+{y} at m={m}");
                    } else {
                        assert_eq!(got, None, "add {x}+{y} should overflow at m={m}");
                    }
                }
            }
        }
    }

    #[test]
    fn sub_matches_integer_oracle() {
        for m in 1..=4 {
            let max = max_magnitude(m).unwrap();
            for x in -max..=max {
                for y in -max..=max {
                    let a = int_to_trits(x, m).unwrap();
                    let b = int_to_trits(y, m).unwrap();
                    let got = sub(&a, &b);
                    let expected = x - y;
                    if expected.abs() <= max {
                        assert_eq!(got, int_to_trits(expected, m), "sub {x}-{y} at m={m}");
                    } else {
                        assert_eq!(got, None, "sub {x}-{y} should overflow at m={m}");
                    }
                }
            }
        }
    }

    /// **Oracle property test (mul):** the shifted-add multiplier agrees with the `i64` oracle for
    /// every pair at small widths.
    #[test]
    fn mul_matches_integer_oracle() {
        for m in 1..=4 {
            let max = max_magnitude(m).unwrap();
            for x in -max..=max {
                for y in -max..=max {
                    let a = int_to_trits(x, m).unwrap();
                    let b = int_to_trits(y, m).unwrap();
                    let got = mul(&a, &b);
                    let expected = x * y;
                    if expected.abs() <= max {
                        assert_eq!(got, int_to_trits(expected, m), "mul {x}*{y} at m={m}");
                    } else {
                        assert_eq!(got, None, "mul {x}*{y} should overflow at m={m}");
                    }
                }
            }
        }
    }

    #[test]
    fn unequal_widths_are_rejected() {
        let a = int_to_trits(1, 2).unwrap();
        let b = int_to_trits(1, 3).unwrap();
        assert_eq!(add(&a, &b), None);
        assert_eq!(sub(&a, &b), None);
        assert_eq!(mul(&a, &b), None);
    }
}
