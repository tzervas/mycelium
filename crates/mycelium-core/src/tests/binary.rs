//! White-box tests for [`crate::binary`]. Extracted from the logic file as-touched by M-887 (test
//! layout rule, M-797) — the pre-existing codec tests plus the new [`crate::binary::mul`] coverage.

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
