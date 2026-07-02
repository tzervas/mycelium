//! White-box tests for [`crate::binary`]. Extracted from the logic file as-touched by M-887 (test
//! layout rule, M-797) — the pre-existing codec tests plus the new [`crate::binary::mul`] coverage.
//! M-888 adds the unsigned [`crate::binary::div_rem`] coverage. M-889 adds the logical
//! [`crate::binary::shl`]/[`crate::binary::shr`] coverage.

use crate::binary::*;

#[test]
fn worked_example_byte() {
    // binary-ternary.md §5: 0b1011_0010 (MSB-first) = −78 in two's complement.
    let bits = [true, false, true, true, false, false, true, false];
    assert_eq!(bits_to_int(&bits), -78);
    assert_eq!(int_to_bits(-78, 8), Some(bits.to_vec()));
}

#[test]
fn range_edges() {
    assert_eq!(
        bits_to_int(&[true, false, false, false, false, false, false, false]),
        -128
    );
    assert_eq!(
        bits_to_int(&[false, true, true, true, true, true, true, true]),
        127
    );
    assert_eq!(int_to_bits(127, 8).map(|b| bits_to_int(&b)), Some(127));
    assert_eq!(int_to_bits(-128, 8).map(|b| bits_to_int(&b)), Some(-128));
    assert_eq!(int_to_bits(128, 8), None); // out of range
    assert_eq!(int_to_bits(-129, 8), None);
}

#[test]
fn round_trips_exhaustively_at_n8() {
    for v in -128..=127 {
        let bits = int_to_bits(v, 8).expect("in range");
        assert_eq!(bits.len(), 8);
        assert_eq!(bits_to_int(&bits), v);
    }
}

// Mutant-witness (binary.rs: `value == 0` → `value != 0` in `int_to_bits`'s n=0 guard):
// the mutation would let any non-zero value return `Some(Vec::new())` (the zero-width
// representation of any integer!) instead of `None`. `round_trips_exhaustively_at_n8` only
// covers n=8, missing this guard entirely.
#[test]
fn int_to_bits_n0_rejects_nonzero() {
    // n=0 has a zero-width repr that can only hold the value 0.
    assert_eq!(
        int_to_bits(0, 0),
        Some(Vec::new()),
        "0 in 0 bits is representable"
    );
    assert_eq!(int_to_bits(1, 0), None, "1 cannot be represented in 0 bits");
    assert_eq!(
        int_to_bits(-1, 0),
        None,
        "-1 cannot be represented in 0 bits"
    );
}

// ---- M-887: `mul` — never-silent two's-complement fixed-width multiply --------------------------

#[test]
fn mul_worked_examples() {
    // 3 * 4 = 12, in range at Binary{8}.
    let a = int_to_bits(3, 8).unwrap();
    let b = int_to_bits(4, 8).unwrap();
    assert_eq!(mul(&a, &b), int_to_bits(12, 8));

    // -3 * 4 = -12.
    let a = int_to_bits(-3, 8).unwrap();
    let b = int_to_bits(4, 8).unwrap();
    assert_eq!(mul(&a, &b), int_to_bits(-12, 8));

    // -3 * -4 = 12.
    let a = int_to_bits(-3, 8).unwrap();
    let b = int_to_bits(-4, 8).unwrap();
    assert_eq!(mul(&a, &b), int_to_bits(12, 8));

    // 0 * anything = 0.
    let zero = int_to_bits(0, 8).unwrap();
    let x = int_to_bits(-100, 8).unwrap();
    assert_eq!(mul(&zero, &x), int_to_bits(0, 8));
}

/// The classic two's-complement multiply-overflow edge: `i8::MIN * -1 = 128`, which does not fit
/// `B_8 = [-128, 127]` — an explicit `None`, never a silent wrap back to `-128`.
#[test]
fn mul_min_times_neg_one_overflows() {
    let min = int_to_bits(-128, 8).unwrap();
    let neg_one = int_to_bits(-1, 8).unwrap();
    assert_eq!(mul(&min, &neg_one), None);
}

#[test]
fn mul_overflow_and_in_range_boundary() {
    // 127 * 2 = 254, out of B_8 ([-128, 127]).
    let a = int_to_bits(127, 8).unwrap();
    let b = int_to_bits(2, 8).unwrap();
    assert_eq!(mul(&a, &b), None);

    // -128 * 1 = -128, exactly the low boundary, in range.
    let a = int_to_bits(-128, 8).unwrap();
    let b = int_to_bits(1, 8).unwrap();
    assert_eq!(mul(&a, &b), int_to_bits(-128, 8));

    // 63 * 2 = 126, in range; 64 * 2 = 128, out of range — the boundary is exact, not off-by-one.
    let sixty_three = int_to_bits(63, 8).unwrap();
    let sixty_four = int_to_bits(64, 8).unwrap();
    let two = int_to_bits(2, 8).unwrap();
    assert_eq!(mul(&sixty_three, &two), int_to_bits(126, 8));
    assert_eq!(mul(&sixty_four, &two), None);
}

#[test]
fn mul_rejects_unequal_widths() {
    let a = int_to_bits(1, 4).unwrap();
    let b = int_to_bits(1, 8).unwrap();
    assert_eq!(mul(&a, &b), None);
}

#[test]
fn mul_rejects_over_cap_width() {
    // n = 65 exceeds `MUL_MAX_WIDTH` (64) — an explicit `None`, never a silently-truncated
    // native-int computation.
    let a = vec![false; MUL_MAX_WIDTH + 1];
    let b = vec![false; MUL_MAX_WIDTH + 1];
    assert_eq!(mul(&a, &b), None);
    // At the cap itself (n = 64) the boundary is accepted (0 * 0 = 0 trivially).
    let a64 = vec![false; MUL_MAX_WIDTH];
    let b64 = vec![false; MUL_MAX_WIDTH];
    assert_eq!(mul(&a64, &b64), int_to_bits(0, 64));
}

#[test]
fn mul_n0_is_trivially_zero() {
    assert_eq!(mul(&[], &[]), Some(Vec::new()));
}

/// **Oracle property test (the overflow bound):** the codec-and-i128 multiplier agrees with a
/// direct `i128` oracle for *every* pair at small widths — in range it equals the exact product's
/// encoding, out of range it is `None`. Mirrors `ternary::mul_matches_integer_oracle`.
#[test]
fn mul_matches_integer_oracle() {
    for n in 1u32..=8 {
        let lo = -(1i64 << (n - 1));
        let hi = (1i64 << (n - 1)) - 1;
        for x in lo..=hi {
            for y in lo..=hi {
                let a = int_to_bits(x, n).unwrap();
                let b = int_to_bits(y, n).unwrap();
                let got = mul(&a, &b);
                let expected = i128::from(x) * i128::from(y);
                if expected >= i128::from(lo) && expected <= i128::from(hi) {
                    let expected_i64 = expected as i64;
                    assert_eq!(got, int_to_bits(expected_i64, n), "mul {x}*{y} at n={n}");
                } else {
                    assert_eq!(got, None, "mul {x}*{y} should overflow at n={n}");
                }
            }
        }
    }
}

// ---- M-888: `div_rem` — never-silent unsigned fixed-width division/remainder -------------------

#[test]
fn uint_round_trips_exhaustively_at_n8() {
    for v in 0u64..=255 {
        let bits = uint_to_bits(v, 8).expect("in range");
        assert_eq!(bits.len(), 8);
        assert_eq!(bits_to_uint(&bits), v);
    }
}

#[test]
fn uint_to_bits_rejects_out_of_range() {
    assert_eq!(uint_to_bits(255, 8), Some(vec![true; 8]));
    assert_eq!(uint_to_bits(256, 8), None); // out of range for 8 bits
    assert_eq!(uint_to_bits(0, 0), Some(Vec::new()));
    assert_eq!(uint_to_bits(1, 0), None);
}

#[test]
fn div_rem_worked_examples() {
    // 7 / 2 = 3 remainder 1.
    let a = uint_to_bits(7, 8).unwrap();
    let b = uint_to_bits(2, 8).unwrap();
    let (q, r) = div_rem(&a, &b).expect("7 / 2");
    assert_eq!(bits_to_uint(&q), 3);
    assert_eq!(bits_to_uint(&r), 1);

    // 255 / 1 = 255 remainder 0 (upper boundary at n=8).
    let a = uint_to_bits(255, 8).unwrap();
    let b = uint_to_bits(1, 8).unwrap();
    let (q, r) = div_rem(&a, &b).expect("255 / 1");
    assert_eq!(bits_to_uint(&q), 255);
    assert_eq!(bits_to_uint(&r), 0);

    // 0 / anything nonzero = 0 remainder 0.
    let zero = uint_to_bits(0, 8).unwrap();
    let x = uint_to_bits(17, 8).unwrap();
    let (q, r) = div_rem(&zero, &x).expect("0 / 17");
    assert_eq!(bits_to_uint(&q), 0);
    assert_eq!(bits_to_uint(&r), 0);
}

/// Division by zero is an explicit `None` — never a panic, never a silently-defined value.
#[test]
fn div_rem_by_zero_refuses() {
    let a = uint_to_bits(7, 8).unwrap();
    let zero = uint_to_bits(0, 8).unwrap();
    assert_eq!(div_rem(&a, &zero), None);
    // Even 0 / 0 refuses — it is not special-cased to (0, 0).
    assert_eq!(div_rem(&zero, &zero), None);
}

#[test]
fn div_rem_rejects_unequal_widths() {
    let a = uint_to_bits(1, 4).unwrap();
    let b = uint_to_bits(1, 8).unwrap();
    assert_eq!(div_rem(&a, &b), None);
}

#[test]
fn div_rem_rejects_over_cap_width() {
    let a = vec![false; DIV_MAX_WIDTH + 1];
    let b = vec![false; DIV_MAX_WIDTH + 1];
    assert_eq!(div_rem(&a, &b), None);
    // At the cap itself the boundary is accepted (subject to the operands, not the width).
    let a64 = uint_to_bits(10, 64).unwrap();
    let b64 = uint_to_bits(3, 64).unwrap();
    let (q, r) = div_rem(&a64, &b64).expect("10 / 3 at n=64");
    assert_eq!(bits_to_uint(&q), 3);
    assert_eq!(bits_to_uint(&r), 1);
}

#[test]
fn div_rem_n0_is_div_by_zero() {
    // The zero-width bitvector's only value is 0; 0 / 0 refuses, it is not silently `(0, 0)`.
    assert_eq!(div_rem(&[], &[]), None);
}

/// **Oracle property test (the Euclidean identity):** for every pair at small widths with a
/// nonzero divisor, `a == quotient * b + remainder` holds bit-exactly and `remainder < b`; every
/// zero-divisor pair is an explicit `None`. Mirrors `mul_matches_integer_oracle`.
#[test]
fn div_rem_matches_euclidean_identity_oracle() {
    for n in 1u32..=8 {
        let hi: u64 = (1u64 << n) - 1;
        for x in 0..=hi {
            for y in 0..=hi {
                let a = uint_to_bits(x, n).unwrap();
                let b = uint_to_bits(y, n).unwrap();
                let got = div_rem(&a, &b);
                if y == 0 {
                    assert_eq!(
                        got, None,
                        "div_rem {x}/{y} should refuse (div-by-zero) at n={n}"
                    );
                } else {
                    let (q, r) =
                        got.unwrap_or_else(|| panic!("div_rem {x}/{y} at n={n} must succeed"));
                    let qv = bits_to_uint(&q);
                    let rv = bits_to_uint(&r);
                    assert_eq!(qv, x / y, "quotient {x}/{y} at n={n}");
                    assert_eq!(rv, x % y, "remainder {x}/{y} at n={n}");
                    // Euclidean identity, bit-exact.
                    assert_eq!(
                        qv * y + rv,
                        x,
                        "Euclidean identity {x} == ({x}/{y})*{y} + {x}%{y}"
                    );
                    assert!(rv < y, "remainder must be < divisor");
                }
            }
        }
    }
}

// ---- M-889: `shl`/`shr` — never-silent logical fixed-width shifts ------------------------------

#[test]
fn shl_worked_examples() {
    // 1 << 3 = 8 at Binary{8}.
    let a = uint_to_bits(1, 8).unwrap();
    let k = uint_to_bits(3, 8).unwrap();
    let got = shl(&a, &k).expect("1 << 3");
    assert_eq!(bits_to_uint(&got), 8);

    // Bits shifted past the MSB are dropped (never wrapped): 0b1111_1111 << 1 = 0b1111_1110.
    let a = uint_to_bits(255, 8).unwrap();
    let k = uint_to_bits(1, 8).unwrap();
    let got = shl(&a, &k).expect("255 << 1");
    assert_eq!(bits_to_uint(&got), 254);
}

#[test]
fn shr_worked_examples() {
    // 8 >> 3 = 1 at Binary{8}.
    let a = uint_to_bits(8, 8).unwrap();
    let k = uint_to_bits(3, 8).unwrap();
    let got = shr(&a, &k).expect("8 >> 3");
    assert_eq!(bits_to_uint(&got), 1);

    // Bits shifted past the LSB are dropped, zero-filled at the MSB (logical, not arithmetic):
    // 0b1000_0000 >> 4 = 0b0000_1000, not sign-extended.
    let a = uint_to_bits(0b1000_0000, 8).unwrap();
    let k = uint_to_bits(4, 8).unwrap();
    let got = shr(&a, &k).expect("0x80 >> 4");
    assert_eq!(bits_to_uint(&got), 0b0000_1000);
}

/// Shifting by `0` is the identity, for both directions.
#[test]
fn shift_by_zero_is_identity() {
    for n in 1u32..=8 {
        let hi: u64 = (1u64 << n) - 1;
        for v in 0..=hi {
            let a = uint_to_bits(v, n).unwrap();
            let zero = uint_to_bits(0, n).unwrap();
            assert_eq!(shl(&a, &zero), Some(a.clone()), "shl by 0 at v={v}, n={n}");
            assert_eq!(shr(&a, &zero), Some(a.clone()), "shr by 0 at v={v}, n={n}");
        }
    }
}

/// A shift amount `>= width` is an explicit `None` — never-silent (never UB, a modulo-wrapped
/// shift amount, or a silently-zeroed result).
#[test]
fn shift_amount_at_or_above_width_refuses() {
    let a = uint_to_bits(0b0000_0001, 8).unwrap();
    // Exactly at the width boundary.
    let k8 = uint_to_bits(8, 8).unwrap();
    assert_eq!(shl(&a, &k8), None, "shift by exactly the width must refuse");
    assert_eq!(shr(&a, &k8), None, "shift by exactly the width must refuse");
    // Above the width.
    let k255 = uint_to_bits(255, 8).unwrap();
    assert_eq!(shl(&a, &k255), None, "shift above the width must refuse");
    assert_eq!(shr(&a, &k255), None, "shift above the width must refuse");
    // One below the width boundary is in range (n=8, k=7 is the max valid shift).
    let k7 = uint_to_bits(7, 8).unwrap();
    assert!(shl(&a, &k7).is_some(), "shift by width-1 must succeed");
    assert!(shr(&a, &k7).is_some(), "shift by width-1 must succeed");
}

#[test]
fn shift_rejects_unequal_widths() {
    let a = uint_to_bits(1, 4).unwrap();
    let k = uint_to_bits(1, 8).unwrap();
    assert_eq!(shl(&a, &k), None);
    assert_eq!(shr(&a, &k), None);
}

#[test]
fn shift_rejects_over_cap_width() {
    let a = vec![false; SHIFT_MAX_WIDTH + 1];
    let k = vec![false; SHIFT_MAX_WIDTH + 1];
    assert_eq!(shl(&a, &k), None);
    assert_eq!(shr(&a, &k), None);
    // At the cap itself the boundary is accepted.
    let a64 = uint_to_bits(1, 64).unwrap();
    let k64 = uint_to_bits(63, 64).unwrap();
    assert_eq!(shl(&a64, &k64).map(|b| bits_to_uint(&b)), Some(1u64 << 63));
    let a64b = uint_to_bits(u64::MAX, 64).unwrap();
    assert_eq!(shr(&a64b, &k64).map(|b| bits_to_uint(&b)), Some(1u64));
}

/// The zero-width bitvector's only representable shift amount is `0`, and `0 >= width(0)`, so
/// `n == 0` always refuses — it is not special-cased to a trivial identity.
#[test]
fn shift_n0_always_refuses() {
    assert_eq!(shl(&[], &[]), None);
    assert_eq!(shr(&[], &[]), None);
}

/// **Oracle property test (the shift-amount bound):** for every value/shift-amount pair at a small
/// width, `shl`/`shr` agree with a native `u64` shift for in-range amounts and refuse explicitly
/// for `k >= n`. Mirrors `div_rem_matches_euclidean_identity_oracle`.
#[test]
fn shift_matches_native_oracle() {
    for n in 1u32..=8 {
        let hi: u64 = (1u64 << n) - 1;
        for v in 0..=hi {
            for k in 0..=hi {
                let a = uint_to_bits(v, n).unwrap();
                let kb = uint_to_bits(k, n).unwrap();
                let got_shl = shl(&a, &kb);
                let got_shr = shr(&a, &kb);
                if k >= u64::from(n) {
                    assert_eq!(got_shl, None, "shl {v}<<{k} at n={n} should refuse");
                    assert_eq!(got_shr, None, "shr {v}>>{k} at n={n} should refuse");
                } else {
                    let mask = if n == 64 { u64::MAX } else { (1u64 << n) - 1 };
                    let expected_shl = (v << k) & mask;
                    let expected_shr = v >> k;
                    assert_eq!(
                        got_shl.map(|b| bits_to_uint(&b)),
                        Some(expected_shl),
                        "shl {v}<<{k} at n={n}"
                    );
                    assert_eq!(
                        got_shr.map(|b| bits_to_uint(&b)),
                        Some(expected_shr),
                        "shr {v}>>{k} at n={n}"
                    );
                }
            }
        }
    }
}
