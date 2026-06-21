//! Two's-complement binary integer semantics (M-120 support).
//!
//! An `n`-bit value is read **most-significant-first** as a two's-complement integer with range
//! `B_n = [−2^(n-1), 2^(n-1) − 1]` (`docs/spec/swaps/binary-ternary.md` §2). This is the binary-side
//! codec the binary↔ternary swap (M-120) uses; the balanced-ternary side lives in
//! [`crate::ternary`]. Values use `i64`, exact for every `n ≤ 64`.

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

#[cfg(test)]
mod tests {
    use super::*;

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

    // Mutant-witness (binary.rs:31:25): `value == 0` → `value != 0`
    // The mutation changes the n=0 guard so that any non-zero value returns Some(Vec::new())
    // (the zero-width representation of any integer!) instead of None. The existing
    // round_trips_exhaustively_at_n8 test only covers n=8, missing this guard entirely.
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
}
