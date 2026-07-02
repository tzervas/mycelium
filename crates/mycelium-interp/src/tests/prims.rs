//! Mutant-witness tests for prims.rs survivors (M-654 Gate A3).
use crate::prims::*;
use crate::EvalError;
use mycelium_core::{Meta, Payload, Provenance, Repr, Value};

fn byte(bits: [bool; 8]) -> Value {
    Value::new(
        Repr::Binary { width: 8 },
        Payload::Bits(bits.to_vec()),
        Meta::exact(Provenance::Root),
    )
    .unwrap()
}

// ---- prims.rs:61 — PrimRegistry::empty → Default::default() ----
// JUSTIFIED: PrimRegistry derives Default (BTreeMap::new()), and `empty()` also constructs
// BTreeMap::new(). The two are semantically identical — both produce an empty registry with no
// registered prims. This mutant is genuinely equivalent and is excluded via mutants.toml.

// ---- prims.rs:169 — expect_arity → Ok(()) ----
// Mutant: expect_arity always succeeds, even with wrong arity — arity errors are never raised.
// Kill: invoking a prim with wrong arity must return a PrimType error, not succeed silently.
#[test]
fn expect_arity_rejects_wrong_arity() {
    // Mutant-witness: prims.rs:169 replace expect_arity → Ok(()).
    // bit.not requires exactly 1 arg; providing 0 or 2 must be a PrimType error.
    // Test via the PrimRegistry public API.
    let reg = PrimRegistry::with_builtins();
    let f = reg.get("bit.not").expect("bit.not registered");
    let b = byte([true; 8]);
    // Zero args → PrimType.
    assert!(
        matches!(f("bit.not", &[]), Err(EvalError::PrimType { .. })),
        "bit.not with 0 args must be PrimType"
    );
    // Two args → PrimType.
    assert!(
        matches!(f("bit.not", &[&b, &b]), Err(EvalError::PrimType { .. })),
        "bit.not with 2 args must be PrimType"
    );
    // One arg → Ok (correct arity).
    assert!(
        f("bit.not", &[&b]).is_ok(),
        "bit.not with 1 arg must succeed"
    );
}

// ---- prims.rs:240 — prim_bit_and: & → | or ^ ----
// Mutant A (& → |): AND is replaced by OR — (1&0)=0 but (1|0)=1.
// Mutant B (& → ^): AND is replaced by XOR — (1&1)=1 but (1^1)=0.
// Kill: test a case where AND, OR, and XOR all differ (e.g. a=1,b=0 and a=1,b=1).
#[test]
fn bit_and_is_conjunction_not_disjunction_or_xor() {
    // Mutant-witness: prims.rs:240 & → | or ^.
    let reg = PrimRegistry::with_builtins();
    let f = reg.get("bit.and").expect("bit.and registered");

    // Operands: a = [true; 8], b = [false; 8].
    // AND: all false. OR: all true. XOR: all true. AND ≠ OR,XOR.
    let a = byte([true; 8]);
    let b_zeros = byte([false; 8]);
    let result = f("bit.and", &[&a, &b_zeros]).expect("bit.and evaluates");
    assert_eq!(
        result.payload(),
        &Payload::Bits(vec![false; 8]),
        "bit.and([1;8], [0;8]) must be [0;8] (AND), not [1;8] (OR/XOR)"
    );

    // Operands: a = [true; 8], b = [true; 8].
    // AND: all true. OR: all true. XOR: all false. AND ≠ XOR here.
    let b_ones = byte([true; 8]);
    let result2 = f("bit.and", &[&a, &b_ones]).expect("bit.and evaluates");
    assert_eq!(
        result2.payload(),
        &Payload::Bits(vec![true; 8]),
        "bit.and([1;8], [1;8]) must be [1;8] (AND/OR), distinguishing from XOR ([0;8])"
    );
}

// ---- prims.rs:243 — prim_bit_or: | → & or ^ ----
// Mutant A (| → &): OR is replaced by AND — (1|0)=1 but (1&0)=0.
// Mutant B (| → ^): OR is replaced by XOR — (1|1)=1 but (1^1)=0.
// Kill: test case where OR, AND, XOR all differ.
#[test]
fn bit_or_is_disjunction_not_conjunction_or_xor() {
    // Mutant-witness: prims.rs:243 | → & or ^.
    let reg = PrimRegistry::with_builtins();
    let f = reg.get("bit.or").expect("bit.or registered");

    // Operands: a = [true; 8], b = [false; 8].
    // OR: all true. AND: all false. XOR: all true. OR ≠ AND.
    let a = byte([true; 8]);
    let b_zeros = byte([false; 8]);
    let result = f("bit.or", &[&a, &b_zeros]).expect("bit.or evaluates");
    assert_eq!(
        result.payload(),
        &Payload::Bits(vec![true; 8]),
        "bit.or([1;8], [0;8]) must be [1;8] (OR), not [0;8] (AND)"
    );

    // Operands: a = [true; 8], b = [true; 8].
    // OR: all true. AND: all true. XOR: all false. OR ≠ XOR here.
    let b_ones = byte([true; 8]);
    let result2 = f("bit.or", &[&a, &b_ones]).expect("bit.or evaluates");
    assert_eq!(
        result2.payload(),
        &Payload::Bits(vec![true; 8]),
        "bit.or([1;8], [1;8]) must be [1;8] (OR/AND), distinguishing from XOR ([0;8])"
    );

    // Mixed: a=[T,F,T,F,T,F,T,F], b=[F,F,F,F,F,F,F,F].
    // OR=[T,F,T,F,T,F,T,F], AND=[F;8], XOR=[T,F,T,F,T,F,T,F] — OR and XOR agree here.
    // But the two tests above already distinguish OR from both AND and XOR.
}

// ---- RFC-0032 D1 (M-747): comparison/equality prims ----

/// MSB-first bit vector from a string (e.g. `"1010_0000"`, underscores ignored).
fn bits(s: &str) -> Vec<bool> {
    s.chars().filter(|c| *c != '_').map(|c| c == '1').collect()
}

/// A `Binary{8}` value from an MSB-first bit string (e.g. `"1010_0000"`, underscores ignored).
fn b8(s: &str) -> Value {
    let v = bits(s);
    assert_eq!(v.len(), 8, "b8 expects 8 bits");
    let mut a = [false; 8];
    a.copy_from_slice(&v);
    byte(a)
}

#[test]
fn cmp_eq_is_structural_equality_returning_binary1() {
    let reg = PrimRegistry::with_builtins();
    let f = reg.get("cmp.eq").expect("cmp.eq registered");
    let a = b8("1010_0000");
    let same = b8("1010_0000");
    let diff = b8("1010_0001");
    // Equal ⇒ Binary{1} = 0b1; the repr collapses from Binary{8} to Binary{1}.
    let r = f("cmp.eq", &[&a, &same]).expect("cmp.eq evaluates");
    assert_eq!(r.repr(), &Repr::Binary { width: 1 });
    assert_eq!(r.payload(), &Payload::Bits(vec![true]));
    // Unequal ⇒ 0b0 (never a silent 0b1).
    let r = f("cmp.eq", &[&a, &diff]).expect("cmp.eq evaluates");
    assert_eq!(r.payload(), &Payload::Bits(vec![false]));
}

#[test]
fn cmp_lt_is_unsigned_magnitude_strict() {
    let reg = PrimRegistry::with_builtins();
    let f = reg.get("cmp.lt").expect("cmp.lt registered");
    let lo = b8("1000_0000"); // 128
    let hi = b8("1010_0000"); // 160
                              // 128 < 160 ⇒ true.
    assert_eq!(
        f("cmp.lt", &[&lo, &hi]).expect("lt").payload(),
        &Payload::Bits(vec![true])
    );
    // Strict: not less when equal, and not less when greater.
    assert_eq!(
        f("cmp.lt", &[&hi, &hi]).expect("lt").payload(),
        &Payload::Bits(vec![false])
    );
    assert_eq!(
        f("cmp.lt", &[&hi, &lo]).expect("lt").payload(),
        &Payload::Bits(vec![false])
    );
}

#[test]
fn cmp_width_mismatch_is_never_silent() {
    // A `Binary{8}` vs `Binary{1}` comparison is an explicit PrimType error — never a silent
    // false (G2). (Same-paradigm, mismatched width.)
    let reg = PrimRegistry::with_builtins();
    let f = reg.get("cmp.eq").expect("cmp.eq registered");
    let wide = b8("0000_0000");
    let narrow = Value::new(
        Repr::Binary { width: 1 },
        Payload::Bits(vec![false]),
        Meta::exact(Provenance::Root),
    )
    .unwrap();
    assert!(
        matches!(
            f("cmp.eq", &[&wide, &narrow]),
            Err(EvalError::PrimType { .. })
        ),
        "mismatched-width eq must be PrimType, never a silent false"
    );
}

// ---- RFC-0032 D2 (M-748): never-silent binary arithmetic ----

#[test]
fn bit_add_in_range_and_overflow_never_silent() {
    let reg = PrimRegistry::with_builtins();
    let f = reg.get("bit.add").expect("bit.add registered");
    // 1 + 2 = 3, carries propagate MSB-first correctly.
    let r = f("bit.add", &[&b8("0000_0001"), &b8("0000_0010")]).expect("add");
    assert_eq!(r.payload(), &Payload::Bits(bits("0000_0011")));
    // 0b0000_1111 (15) + 0b0000_0001 (1) = 0b0001_0000 (16) — carry chain across the nibble.
    let r = f("bit.add", &[&b8("0000_1111"), &b8("0000_0001")]).expect("add");
    assert_eq!(r.payload(), &Payload::Bits(bits("0001_0000")));
    // 255 + 1 overflows Binary{8}: explicit Overflow, never a silent wrap to 0.
    assert!(
        matches!(
            f("bit.add", &[&b8("1111_1111"), &b8("0000_0001")]),
            Err(EvalError::Overflow { .. })
        ),
        "add overflow must be explicit, never a silent wrap"
    );
}

#[test]
fn bit_sub_in_range_and_underflow_never_silent() {
    let reg = PrimRegistry::with_builtins();
    let f = reg.get("bit.sub").expect("bit.sub registered");
    // 5 - 2 = 3, borrow chain correct.
    let r = f("bit.sub", &[&b8("0000_0101"), &b8("0000_0010")]).expect("sub");
    assert_eq!(r.payload(), &Payload::Bits(bits("0000_0011")));
    // 16 - 1 = 15 — borrow across the nibble.
    let r = f("bit.sub", &[&b8("0001_0000"), &b8("0000_0001")]).expect("sub");
    assert_eq!(r.payload(), &Payload::Bits(bits("0000_1111")));
    // 0 - 1 underflows (no unsigned negative): explicit Overflow, never a silent wrap to 255.
    assert!(
        matches!(
            f("bit.sub", &[&b8("0000_0000"), &b8("0000_0001")]),
            Err(EvalError::Overflow { .. })
        ),
        "sub underflow must be explicit, never a silent wrap"
    );
}

// ---- RFC-0033 §4.1.2/§4.1.3 (M-887, `enb` Gap B): never-silent two's-complement multiply ----

/// A `Binary{width}` value of all-`false` bits, then patched via `set` — used to build wide
/// (> 8-bit) operands the `b8` helper can't express.
fn wide_binary(width: usize, ones_at_msb_first: &[usize]) -> Value {
    let mut bits = vec![false; width];
    for &i in ones_at_msb_first {
        bits[i] = true;
    }
    Value::new(
        Repr::Binary {
            width: width as u32,
        },
        Payload::Bits(bits),
        Meta::exact(Provenance::Root),
    )
    .unwrap()
}

#[test]
fn bin_mul_in_range_positive_and_negative() {
    let reg = PrimRegistry::with_builtins();
    let f = reg.get("bin.mul").expect("bin.mul registered");
    // 3 * 4 = 12 (0b0000_0011 * 0b0000_0100 = 0b0000_1100).
    let r = f("bin.mul", &[&b8("0000_0011"), &b8("0000_0100")]).expect("mul");
    assert_eq!(r.payload(), &Payload::Bits(bits("0000_1100")));
    assert_eq!(r.repr(), &Repr::Binary { width: 8 });
    // -3 * 4 = -12: -3 is 0b1111_1101, -12 is 0b1111_0100.
    let r = f("bin.mul", &[&b8("1111_1101"), &b8("0000_0100")]).expect("mul");
    assert_eq!(r.payload(), &Payload::Bits(bits("1111_0100")));
    // -3 * -4 = 12.
    let r = f("bin.mul", &[&b8("1111_1101"), &b8("1111_1100")]).expect("mul");
    assert_eq!(r.payload(), &Payload::Bits(bits("0000_1100")));
}

/// The classic two's-complement overflow edge: `i8::MIN * -1 = 128`, out of `B_8 = [-128, 127]` —
/// an explicit `Overflow`, never a silent wrap back to `-128`.
#[test]
fn bin_mul_min_times_neg_one_overflows() {
    let reg = PrimRegistry::with_builtins();
    let f = reg.get("bin.mul").expect("bin.mul registered");
    let min = b8("1000_0000"); // -128
    let neg_one = b8("1111_1111"); // -1
    assert!(
        matches!(
            f("bin.mul", &[&min, &neg_one]),
            Err(EvalError::Overflow { .. })
        ),
        "i8::MIN * -1 must be an explicit overflow, never a silent wrap"
    );
}

#[test]
fn bin_mul_overflow_never_silent() {
    let reg = PrimRegistry::with_builtins();
    let f = reg.get("bin.mul").expect("bin.mul registered");
    // 127 * 2 = 254, out of B_8 ([-128, 127]).
    assert!(
        matches!(
            f("bin.mul", &[&b8("0111_1111"), &b8("0000_0010")]),
            Err(EvalError::Overflow { .. })
        ),
        "mul overflow must be explicit, never a silent wrap"
    );
}

#[test]
fn bin_mul_width_mismatch_is_never_silent() {
    let reg = PrimRegistry::with_builtins();
    let f = reg.get("bin.mul").expect("bin.mul registered");
    let wide = b8("0000_0001");
    let narrow = Value::new(
        Repr::Binary { width: 1 },
        Payload::Bits(vec![false]),
        Meta::exact(Provenance::Root),
    )
    .unwrap();
    assert!(
        matches!(
            f("bin.mul", &[&wide, &narrow]),
            Err(EvalError::PrimType { .. })
        ),
        "mismatched-width mul must be PrimType, never a silent coercion"
    );
}

/// A width beyond the current `bin.mul` cap (`mycelium_core::binary::MUL_MAX_WIDTH`) is an explicit
/// `PrimType` refusal — distinct from an in-range-width `Overflow` — never a silently-truncated
/// native-int computation (M-887 scope boundary; FLAGged for the Gap-B follow-ons).
#[test]
fn bin_mul_over_cap_width_is_never_silent() {
    let reg = PrimRegistry::with_builtins();
    let f = reg.get("bin.mul").expect("bin.mul registered");
    let width = mycelium_core::binary::MUL_MAX_WIDTH + 1;
    let a = wide_binary(width, &[]);
    let b = wide_binary(width, &[]);
    assert!(
        matches!(f("bin.mul", &[&a, &b]), Err(EvalError::PrimType { .. })),
        "an over-cap width must be an explicit PrimType refusal, never a silent truncation"
    );
}

/// **Property test (the overflow bound):** for every in-range pair at a small width, `bin.mul`'s
/// result agrees with an `i64` oracle; every out-of-range pair is an explicit `Overflow`. Mirrors
/// `mycelium_core::binary`'s own `mul_matches_integer_oracle` at the codec layer, one level up
/// through the prim's dispatch + never-silent-error mapping.
#[test]
fn bin_mul_matches_integer_oracle_at_width6() {
    let reg = PrimRegistry::with_builtins();
    let f = reg.get("bin.mul").expect("bin.mul registered");
    let n: u32 = 6;
    let lo = -(1i64 << (n - 1));
    let hi = (1i64 << (n - 1)) - 1;
    for x in lo..=hi {
        for y in lo..=hi {
            let av = mycelium_core::binary::int_to_bits(x, n).unwrap();
            let bv = mycelium_core::binary::int_to_bits(y, n).unwrap();
            let a = Value::new(
                Repr::Binary { width: n },
                Payload::Bits(av),
                Meta::exact(Provenance::Root),
            )
            .unwrap();
            let b = Value::new(
                Repr::Binary { width: n },
                Payload::Bits(bv),
                Meta::exact(Provenance::Root),
            )
            .unwrap();
            let expected = i128::from(x) * i128::from(y);
            let got = f("bin.mul", &[&a, &b]);
            if expected >= i128::from(lo) && expected <= i128::from(hi) {
                let want_bits = mycelium_core::binary::int_to_bits(expected as i64, n).unwrap();
                assert_eq!(
                    got.expect("in-range mul must succeed").payload(),
                    &Payload::Bits(want_bits),
                    "mul {x}*{y} at n={n}"
                );
            } else {
                assert!(
                    matches!(got, Err(EvalError::Overflow { .. })),
                    "mul {x}*{y} at n={n} should overflow, got {got:?}"
                );
            }
        }
    }
}

// ---- RFC-0033 §4.1.2/§4.1.3 (M-888, `enb` Gap B): never-silent unsigned division/remainder ----

/// A `Binary{n}` value from a non-negative `u64`, built via `mycelium_core::binary::uint_to_bits`.
fn u_bin(value: u64, n: u32) -> Value {
    let bits = mycelium_core::binary::uint_to_bits(value, n).expect("in range");
    Value::new(
        Repr::Binary { width: n },
        Payload::Bits(bits),
        Meta::exact(Provenance::Root),
    )
    .unwrap()
}

#[test]
fn bin_div_and_rem_worked_examples() {
    let reg = PrimRegistry::with_builtins();
    let div = reg.get("bin.div").expect("bin.div registered");
    let rem = reg.get("bin.rem").expect("bin.rem registered");
    // 7 / 2 = 3 remainder 1.
    let a = u_bin(7, 8);
    let b = u_bin(2, 8);
    let q = div("bin.div", &[&a, &b]).expect("7 / 2");
    let r = rem("bin.rem", &[&a, &b]).expect("7 % 2");
    assert_eq!(
        q.payload(),
        &Payload::Bits(mycelium_core::binary::uint_to_bits(3, 8).unwrap())
    );
    assert_eq!(
        r.payload(),
        &Payload::Bits(mycelium_core::binary::uint_to_bits(1, 8).unwrap())
    );
}

#[test]
fn bin_div_by_zero_is_never_silent() {
    let reg = PrimRegistry::with_builtins();
    let div = reg.get("bin.div").expect("bin.div registered");
    let rem = reg.get("bin.rem").expect("bin.rem registered");
    let a = u_bin(7, 8);
    let zero = u_bin(0, 8);
    assert!(
        matches!(
            div("bin.div", &[&a, &zero]),
            Err(EvalError::PrimType { .. })
        ),
        "division by zero must be an explicit PrimType refusal, never a panic or silent value"
    );
    assert!(
        matches!(
            rem("bin.rem", &[&a, &zero]),
            Err(EvalError::PrimType { .. })
        ),
        "remainder by zero must be an explicit PrimType refusal, never a panic or silent value"
    );
}

#[test]
fn bin_div_rem_width_mismatch_is_never_silent() {
    let reg = PrimRegistry::with_builtins();
    let div = reg.get("bin.div").expect("bin.div registered");
    let wide = u_bin(1, 8);
    let narrow = u_bin(1, 1);
    assert!(
        matches!(
            div("bin.div", &[&wide, &narrow]),
            Err(EvalError::PrimType { .. })
        ),
        "mismatched-width div must be PrimType, never a silent coercion"
    );
}

/// A width beyond the current `bin.div`/`bin.rem` cap (`mycelium_core::binary::DIV_MAX_WIDTH`) is
/// an explicit `PrimType` refusal — never a silently-truncated native-int computation (M-888 scope
/// boundary, mirroring `bin.mul`'s `MUL_MAX_WIDTH` refusal).
#[test]
fn bin_div_over_cap_width_is_never_silent() {
    let reg = PrimRegistry::with_builtins();
    let div = reg.get("bin.div").expect("bin.div registered");
    let width = mycelium_core::binary::DIV_MAX_WIDTH + 1;
    let a = wide_binary(width, &[]);
    let b = wide_binary(width, &[]);
    assert!(
        matches!(div("bin.div", &[&a, &b]), Err(EvalError::PrimType { .. })),
        "an over-cap width must be an explicit PrimType refusal, never a silent truncation"
    );
}

/// **Property test (the Euclidean identity):** for every pair at a small width with a nonzero
/// divisor, `bin.div`/`bin.rem` satisfy `a == (a/b)*b + (a%b)` bit-exactly, with `remainder <
/// divisor`; every zero-divisor pair is an explicit `PrimType` refusal, never a panic. Mirrors
/// `mycelium_core::binary`'s own `div_rem_matches_euclidean_identity_oracle` at the codec layer,
/// one level up through the prim's dispatch + never-silent-error mapping.
#[test]
fn bin_div_rem_satisfy_euclidean_identity_at_width6() {
    let reg = PrimRegistry::with_builtins();
    let div = reg.get("bin.div").expect("bin.div registered");
    let rem = reg.get("bin.rem").expect("bin.rem registered");
    let n: u32 = 6;
    let hi: u64 = (1u64 << n) - 1;
    for x in 0..=hi {
        for y in 0..=hi {
            let a = u_bin(x, n);
            let b = u_bin(y, n);
            let got_q = div("bin.div", &[&a, &b]);
            let got_r = rem("bin.rem", &[&a, &b]);
            if y == 0 {
                assert!(
                    matches!(got_q, Err(EvalError::PrimType { .. })),
                    "div by zero at x={x} must refuse, got {got_q:?}"
                );
                assert!(
                    matches!(got_r, Err(EvalError::PrimType { .. })),
                    "rem by zero at x={x} must refuse, got {got_r:?}"
                );
            } else {
                let q_val = got_q.expect("in-range div must succeed");
                let r_val = got_r.expect("in-range rem must succeed");
                let Payload::Bits(q_bits) = q_val.payload() else {
                    panic!("bin.div must return Payload::Bits")
                };
                let Payload::Bits(r_bits) = r_val.payload() else {
                    panic!("bin.rem must return Payload::Bits")
                };
                let qv = mycelium_core::binary::bits_to_uint(q_bits);
                let rv = mycelium_core::binary::bits_to_uint(r_bits);
                assert_eq!(qv, x / y, "quotient {x}/{y} at n={n}");
                assert_eq!(rv, x % y, "remainder {x}/{y} at n={n}");
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

// ---- RFC-0033 §4.1.2/§4.1.3 (M-889, `enb` Gap B): never-silent logical shift ----

#[test]
fn bin_shl_and_shr_worked_examples() {
    let reg = PrimRegistry::with_builtins();
    let shl = reg.get("bin.shl").expect("bin.shl registered");
    let shr = reg.get("bin.shr").expect("bin.shr registered");
    // 1 << 3 = 8, 8 >> 3 = 1.
    let one = u_bin(1, 8);
    let three = u_bin(3, 8);
    let r = shl("bin.shl", &[&one, &three]).expect("1 << 3");
    assert_eq!(
        r.payload(),
        &Payload::Bits(mycelium_core::binary::uint_to_bits(8, 8).unwrap())
    );
    let eight = u_bin(8, 8);
    let r = shr("bin.shr", &[&eight, &three]).expect("8 >> 3");
    assert_eq!(
        r.payload(),
        &Payload::Bits(mycelium_core::binary::uint_to_bits(1, 8).unwrap())
    );
    // Logical (zero-filling) right shift: 0x80 >> 4 = 0x08, never sign-extended.
    let hi_bit = u_bin(0b1000_0000, 8);
    let four = u_bin(4, 8);
    let r = shr("bin.shr", &[&hi_bit, &four]).expect("0x80 >> 4");
    assert_eq!(
        r.payload(),
        &Payload::Bits(mycelium_core::binary::uint_to_bits(0b0000_1000, 8).unwrap())
    );
}

#[test]
fn bin_shift_by_zero_is_identity() {
    let reg = PrimRegistry::with_builtins();
    let shl = reg.get("bin.shl").expect("bin.shl registered");
    let shr = reg.get("bin.shr").expect("bin.shr registered");
    let a = u_bin(0b1010_1010, 8);
    let zero = u_bin(0, 8);
    let r = shl("bin.shl", &[&a, &zero]).expect("shl by 0");
    assert_eq!(r.payload(), a.payload());
    let r = shr("bin.shr", &[&a, &zero]).expect("shr by 0");
    assert_eq!(r.payload(), a.payload());
}

/// A shift amount `>= width` is an explicit `PrimType` refusal — never UB, a silently wrapped
/// shift amount, or a silently-zeroed result.
#[test]
fn bin_shift_amount_at_or_above_width_is_never_silent() {
    let reg = PrimRegistry::with_builtins();
    let shl = reg.get("bin.shl").expect("bin.shl registered");
    let shr = reg.get("bin.shr").expect("bin.shr registered");
    let a = u_bin(1, 8);
    let width = u_bin(8, 8);
    assert!(
        matches!(
            shl("bin.shl", &[&a, &width]),
            Err(EvalError::PrimType { .. })
        ),
        "shift-amount == width must be an explicit PrimType refusal, never UB/wrap"
    );
    assert!(
        matches!(
            shr("bin.shr", &[&a, &width]),
            Err(EvalError::PrimType { .. })
        ),
        "shift-amount == width must be an explicit PrimType refusal, never UB/wrap"
    );
    let above = u_bin(255, 8);
    assert!(matches!(
        shl("bin.shl", &[&a, &above]),
        Err(EvalError::PrimType { .. })
    ));
    assert!(matches!(
        shr("bin.shr", &[&a, &above]),
        Err(EvalError::PrimType { .. })
    ));
}

#[test]
fn bin_shift_width_mismatch_is_never_silent() {
    let reg = PrimRegistry::with_builtins();
    let shl = reg.get("bin.shl").expect("bin.shl registered");
    let wide = u_bin(1, 8);
    let narrow = u_bin(1, 1);
    assert!(
        matches!(
            shl("bin.shl", &[&wide, &narrow]),
            Err(EvalError::PrimType { .. })
        ),
        "mismatched-width shift must be PrimType, never a silent coercion"
    );
}

/// A width beyond the current `bin.shl`/`bin.shr` cap (`mycelium_core::binary::SHIFT_MAX_WIDTH`)
/// is an explicit `PrimType` refusal — never a silently-truncated native-int computation (M-889
/// scope boundary, mirroring `bin.mul`/`bin.div`'s width-cap refusals).
#[test]
fn bin_shift_over_cap_width_is_never_silent() {
    let reg = PrimRegistry::with_builtins();
    let shl = reg.get("bin.shl").expect("bin.shl registered");
    let width = mycelium_core::binary::SHIFT_MAX_WIDTH + 1;
    let a = wide_binary(width, &[]);
    let b = wide_binary(width, &[]);
    assert!(
        matches!(shl("bin.shl", &[&a, &b]), Err(EvalError::PrimType { .. })),
        "an over-cap width must be an explicit PrimType refusal, never a silent truncation"
    );
}

/// **Property test (the shift-amount bound):** for every value/shift-amount pair at a small width,
/// `bin.shl`/`bin.shr` agree with a native `u64` shift for in-range amounts and refuse explicitly
/// for `k >= n`. Mirrors `mycelium_core::binary`'s own `shift_matches_native_oracle` at the codec
/// layer, one level up through the prim's dispatch + never-silent-error mapping.
#[test]
fn bin_shift_matches_native_oracle_at_width6() {
    let reg = PrimRegistry::with_builtins();
    let shl = reg.get("bin.shl").expect("bin.shl registered");
    let shr = reg.get("bin.shr").expect("bin.shr registered");
    let n: u32 = 6;
    let hi: u64 = (1u64 << n) - 1;
    for v in 0..=hi {
        for k in 0..=hi {
            let a = u_bin(v, n);
            let kb = u_bin(k, n);
            let got_shl = shl("bin.shl", &[&a, &kb]);
            let got_shr = shr("bin.shr", &[&a, &kb]);
            if k >= u64::from(n) {
                assert!(
                    matches!(got_shl, Err(EvalError::PrimType { .. })),
                    "shl {v}<<{k} at n={n} should refuse, got {got_shl:?}"
                );
                assert!(
                    matches!(got_shr, Err(EvalError::PrimType { .. })),
                    "shr {v}>>{k} at n={n} should refuse, got {got_shr:?}"
                );
            } else {
                let mask = (1u64 << n) - 1;
                let expected_shl = (v << k) & mask;
                let expected_shr = v >> k;
                let shl_val = got_shl.expect("in-range shl must succeed");
                let shr_val = got_shr.expect("in-range shr must succeed");
                let Payload::Bits(shl_bits) = shl_val.payload() else {
                    panic!("bin.shl must return Payload::Bits")
                };
                let Payload::Bits(shr_bits) = shr_val.payload() else {
                    panic!("bin.shr must return Payload::Bits")
                };
                assert_eq!(
                    mycelium_core::binary::bits_to_uint(shl_bits),
                    expected_shl,
                    "shl {v}<<{k} at n={n}"
                );
                assert_eq!(
                    mycelium_core::binary::bits_to_uint(shr_bits),
                    expected_shr,
                    "shr {v}>>{k} at n={n}"
                );
            }
        }
    }
}

// ── M-890 (`enb` Gap C): the dense elementwise prim group ───────────────────────────────────────
//
// `dense.add`/`dense.sub`/`dense.neg`/`dense.scale` — the first tensor-valued prims. The kernel
// (`mycelium-dense`) constructs the result `Value` with its honest per-op tag; the wrapper carries
// it through unchanged (VR-5). These tests pin: (1) the Π-table intrinsic ↔ kernel `op_guarantee`
// consistency (the cross-crate guard `mycelium-core` cannot host), (2) accept-path payloads +
// carried tags/bounds, (3) the never-silent reject surface (shape/dtype mismatch, overflow,
// approximate sources, malformed scale factors), and (4) the per-element relative-error bound
// against an exact f64 oracle (the cheap property the `Proven` tag discloses).

use mycelium_core::{Bound, BoundBasis, BoundKind, NormKind, PrimTable, ScalarKind};
use mycelium_dense::{DenseOp, DenseSpace};

/// A `Dense{n, F32}` value from on-grid elements (test fixture).
fn dense_f32(xs: Vec<f64>) -> Value {
    let n = u32::try_from(xs.len()).expect("test dims are small");
    DenseSpace::new(n, ScalarKind::F32)
        .expect("F32 is a supported dtype")
        .value(xs)
        .expect("fixture elements are finite and on-grid")
}

/// The Π-table intrinsic must equal the kernel's per-op tag — the VR-5 "carried, never upgraded"
/// contract, guarded here because `mycelium-core` (Π) cannot depend on `mycelium-dense` (kernel).
#[test]
fn dense_prim_table_intrinsics_match_the_kernel_op_guarantees() {
    let table = PrimTable::builtins();
    for (name, op) in [
        ("dense.add", DenseOp::Add),
        ("dense.sub", DenseOp::Sub),
        ("dense.neg", DenseOp::Neg),
        ("dense.scale", DenseOp::Scale),
        ("dense.dot", DenseOp::Dot),
        ("dense.similarity", DenseOp::Similarity),
    ] {
        assert_eq!(
            table.intrinsic(name),
            Some(DenseSpace::op_guarantee(op)),
            "{name}: the Π intrinsic must be carried verbatim from DenseSpace::op_guarantee"
        );
    }
}

#[test]
fn dense_add_carries_the_kernel_proven_tag_and_bound() {
    let reg = PrimRegistry::with_builtins();
    let f = reg.get("dense.add").expect("dense.add registered");
    let a = dense_f32(vec![1.5, 2.5]);
    let b = dense_f32(vec![0.25, -1.0]);
    let y = f("dense.add", &[&a, &b]).expect("in-range add");
    assert_eq!(y.payload(), &Payload::Scalars(vec![1.75, 1.5]));
    // The tag is the KERNEL's, carried unchanged: Proven + the per-element relative ε under a
    // ProvenThm basis (never re-derived by compose_result, whose intrinsic is Exact).
    assert_eq!(
        y.meta().guarantee(),
        mycelium_core::GuaranteeStrength::Proven
    );
    let space = DenseSpace::new(2, ScalarKind::F32).unwrap();
    match y.meta().bound() {
        Some(Bound {
            kind: BoundKind::Error { eps, norm },
            basis: BoundBasis::ProvenThm { .. },
        }) => {
            assert_eq!(
                *eps,
                space.op_rel_eps(),
                "ε must be the kernel's op_rel_eps"
            );
            assert_eq!(*norm, NormKind::Rel);
        }
        other => panic!("expected the kernel's ProvenThm Error bound, got {other:?}"),
    }
    // Provenance is the kernel's Derived{op: hash("dense.add"), inputs}.
    match y.meta().provenance() {
        Provenance::Derived { op, inputs } => {
            assert_eq!(op, &mycelium_core::operation_hash("dense.add"));
            assert_eq!(inputs, &vec![a.content_hash(), b.content_hash()]);
        }
        other => panic!("expected Derived provenance, got {other:?}"),
    }
}

#[test]
fn dense_sub_and_neg_accept_paths() {
    let reg = PrimRegistry::with_builtins();
    let sub = reg.get("dense.sub").expect("dense.sub registered");
    let neg = reg.get("dense.neg").expect("dense.neg registered");
    let a = dense_f32(vec![1.5, 2.5]);
    let b = dense_f32(vec![0.5, -1.0]);
    let d = sub("dense.sub", &[&a, &b]).expect("in-range sub");
    assert_eq!(d.payload(), &Payload::Scalars(vec![1.0, 3.5]));
    assert_eq!(
        d.meta().guarantee(),
        mycelium_core::GuaranteeStrength::Proven
    );
    // neg is Exact (the grids are symmetric — never rounds) with no bound.
    let n = neg("dense.neg", &[&a]).expect("neg is total over on-grid inputs");
    assert_eq!(n.payload(), &Payload::Scalars(vec![-1.5, -2.5]));
    assert_eq!(
        n.meta().guarantee(),
        mycelium_core::GuaranteeStrength::Exact
    );
    assert!(n.meta().bound().is_none(), "Exact results carry no bound");
}

#[test]
fn dense_scale_takes_a_dense1_factor() {
    let reg = PrimRegistry::with_builtins();
    let f = reg.get("dense.scale").expect("dense.scale registered");
    let a = dense_f32(vec![1.5, -2.0]);
    let c = dense_f32(vec![2.0]); // the pre-Gap-A scalar form: Dense{1, same dtype}
    let y = f("dense.scale", &[&a, &c]).expect("on-grid scale");
    assert_eq!(y.payload(), &Payload::Scalars(vec![3.0, -4.0]));
    assert_eq!(
        y.meta().guarantee(),
        mycelium_core::GuaranteeStrength::Proven
    );
}

#[test]
fn dense_shape_mismatch_is_never_silent() {
    let reg = PrimRegistry::with_builtins();
    let f = reg.get("dense.add").expect("dense.add registered");
    let a = dense_f32(vec![1.0, 2.0]);
    let b3 = dense_f32(vec![1.0, 2.0, 3.0]);
    // Dim mismatch → explicit PrimType naming expected/got — never a broadcast (G2).
    let err = f("dense.add", &[&a, &b3]).expect_err("dim mismatch must refuse");
    match err {
        EvalError::PrimType { prim, why } => {
            assert_eq!(prim, "dense.add");
            assert!(
                why.contains("dimension mismatch"),
                "the refusal must name the shape mismatch: {why}"
            );
        }
        other => panic!("expected PrimType, got {other:?}"),
    }
    // Dtype mismatch → explicit PrimType, never a re-round.
    let bf = DenseSpace::new(2, ScalarKind::Bf16)
        .unwrap()
        .value(vec![1.5, -2.0])
        .unwrap();
    assert!(
        matches!(f("dense.add", &[&a, &bf]), Err(EvalError::PrimType { .. })),
        "dtype mismatch must be an explicit refusal"
    );
    // A non-Dense operand → explicit PrimType.
    let bits = byte([true; 8]);
    assert!(
        matches!(
            f("dense.add", &[&bits, &a]),
            Err(EvalError::PrimType { .. })
        ),
        "a non-Dense first operand must be an explicit refusal"
    );
    assert!(
        matches!(
            f("dense.add", &[&a, &bits]),
            Err(EvalError::PrimType { .. })
        ),
        "a non-Dense second operand must be an explicit refusal"
    );
    // Wrong arity → explicit PrimType.
    assert!(matches!(
        f("dense.add", &[&a]),
        Err(EvalError::PrimType { .. })
    ));
}

#[test]
fn dense_overflow_and_approx_sources_refuse_explicitly() {
    let reg = PrimRegistry::with_builtins();
    let f = reg.get("dense.add").expect("dense.add registered");
    // Overflow: f32::MAX + f32::MAX exceeds the dtype's finite range → EvalError::Overflow.
    let max = dense_f32(vec![f64::from(f32::MAX)]);
    assert!(
        matches!(
            f("dense.add", &[&max, &max]),
            Err(EvalError::Overflow { .. })
        ),
        "an out-of-range result must be an explicit Overflow, never ±Inf"
    );
    // An approximate source has no defined composition rule (M-204/M-211) →
    // ApproxCompositionUnsupported — carried from the kernel's ApproximateSource refusal.
    let a = dense_f32(vec![1.0, 2.0]);
    let approx = f("dense.add", &[&a, &dense_f32(vec![0.5, 0.5])]).expect("a Proven value");
    assert!(
        matches!(
            f("dense.add", &[&a, &approx]),
            Err(EvalError::ApproxCompositionUnsupported { .. })
        ),
        "an approximate (Proven) source must refuse — no composition rule yet"
    );
}

#[test]
fn dense_scale_factor_contract_is_never_silent() {
    let reg = PrimRegistry::with_builtins();
    let f = reg.get("dense.scale").expect("dense.scale registered");
    let a = dense_f32(vec![1.5, -2.0]);
    // A non-Dense{1} factor (wrong dim) → explicit PrimType.
    let c2 = dense_f32(vec![2.0, 2.0]);
    assert!(
        matches!(
            f("dense.scale", &[&a, &c2]),
            Err(EvalError::PrimType { .. })
        ),
        "a Dense{{2}} factor must refuse — the scalar form is Dense{{1}}"
    );
    // A non-Dense factor → explicit PrimType.
    let bits = byte([false; 8]);
    assert!(matches!(
        f("dense.scale", &[&a, &bits]),
        Err(EvalError::PrimType { .. })
    ));
    // A factor of the wrong dtype → explicit PrimType (never a silent re-round).
    let cbf = DenseSpace::new(1, ScalarKind::Bf16)
        .unwrap()
        .value(vec![2.0])
        .unwrap();
    assert!(matches!(
        f("dense.scale", &[&a, &cbf]),
        Err(EvalError::PrimType { .. })
    ));
    // An approximate factor → ApproxCompositionUnsupported (no defined composition rule).
    let one = dense_f32(vec![1.0]);
    let approx_c = reg.get("dense.add").unwrap()("dense.add", &[&one, &one]).expect("Proven");
    assert!(matches!(
        f("dense.scale", &[&a, &approx_c]),
        Err(EvalError::ApproxCompositionUnsupported { .. })
    ));
}

/// **Property test (the disclosed bound):** over an on-grid corpus, each `dense.add`/`dense.sub`
/// result element differs from the exact `f64` oracle by at most the disclosed per-element
/// relative ε (`op_rel_eps` — the very bound the carried `Proven` tag claims), and
/// `dense.neg` is an exact involution (`neg(neg(x)) == x`, the `Exact` claim). Loop-corpus style,
/// mirroring `bin_mul_matches_integer_oracle_at_width6`.
#[test]
fn dense_elementwise_results_respect_the_disclosed_relative_bound() {
    let reg = PrimRegistry::with_builtins();
    let add = reg.get("dense.add").unwrap();
    let sub = reg.get("dense.sub").unwrap();
    let neg = reg.get("dense.neg").unwrap();
    let space = DenseSpace::new(4, ScalarKind::F32).unwrap();
    let eps = space.op_rel_eps();
    // On-grid f32 corpus spanning magnitudes and signs (all exactly representable in f32).
    let corpus: [[f64; 4]; 4] = [
        [1.5, -0.625, 1024.0, -3.25],
        [0.25, 7.5, -0.03125, 2048.0],
        [-1.0, 0.5, 100.5, -0.75],
        [3.0, -12.25, 0.125, 640.0],
    ];
    for xs in &corpus {
        for ys in &corpus {
            let a = dense_f32(xs.to_vec());
            let b = dense_f32(ys.to_vec());
            for (prim, f, oracle) in [
                ("dense.add", add, (|x, y| x + y) as fn(f64, f64) -> f64),
                ("dense.sub", sub, (|x, y| x - y) as fn(f64, f64) -> f64),
            ] {
                let y = f(prim, &[&a, &b]).expect("corpus results are in range");
                let Payload::Scalars(out) = y.payload() else {
                    panic!("{prim} must return Payload::Scalars")
                };
                for (i, (&got, (&x, &yv))) in out.iter().zip(xs.iter().zip(ys)).enumerate() {
                    let exact = oracle(x, yv);
                    // |got − exact| ≤ ε·|exact|: the per-element relative bound the Proven tag
                    // discloses (exact == 0 ⇒ got must be exactly 0 — no absolute slack).
                    assert!(
                        (got - exact).abs() <= eps * exact.abs(),
                        "{prim} element {i}: |{got} − {exact}| exceeds ε·|exact| (ε = {eps})"
                    );
                }
            }
            // Exact involution: neg(neg(a)) == a, payload-identical (the Exact claim).
            let n1 = neg("dense.neg", &[&a]).expect("neg is total");
            let n2 = neg("dense.neg", &[&n1]).expect("neg is total");
            assert_eq!(
                n2.payload(),
                a.payload(),
                "dense.neg must be an exact involution"
            );
        }
    }
}

// ── M-891 (`enb` Gap C): the dense measurement pair `dense.dot`/`dense.similarity` ──────────────
//
// The kernel constructs the `Dense{1, F64}` measurement value with its honest per-op tag —
// `Proven` with the **binary64 accumulation bound** (absolute/`Linf`, `dot_abs_eps`/
// `similarity_abs_eps` — deliberately NOT the dtype's per-element `op_rel_eps`; see the module
// note in `prims.rs`) — and the wrapper carries it through unchanged (VR-5). These tests pin:
// (1) the Π ↔ kernel tag consistency (folded into the M-890 guard above), (2) the accept-path
// payload + the carried tag/bound/provenance, (3) the EXPLAIN inspectability of the disclosed ε +
// its ProvenThm citation off the result value itself, (4) the never-silent reject surface, and
// (5) the disclosed-bound property over analytically-known dots, including the cancellation case
// a per-element relative claim would fail.

/// M-891 accept path: `dense.dot` returns the f64 measurement as `Dense{1, F64}` with the
/// kernel's `Proven` accumulation bound — and that bound is **EXPLAIN-able**: guarantee, ε,
/// norm, and the ProvenThm citation are all inspectable off the value's `Meta` (G2/SC-3).
#[test]
fn dense_dot_carries_the_inspectable_accumulation_bound() {
    let reg = PrimRegistry::with_builtins();
    let f = reg.get("dense.dot").expect("dense.dot registered");
    let a = dense_f32(vec![1.5, 2.0, -0.5]);
    let b = dense_f32(vec![2.0, 0.25, 4.0]);
    let y = f("dense.dot", &[&a, &b]).expect("in-range dot");
    // 3.0 + 0.5 − 2.0 = 1.5 (every product and partial sum exact in f64).
    assert_eq!(
        y.repr(),
        &Repr::Dense {
            dim: 1,
            dtype: ScalarKind::F64
        },
        "the measurement result form is Dense{{1, F64}}"
    );
    assert_eq!(y.payload(), &Payload::Scalars(vec![1.5]));
    assert_eq!(
        y.meta().guarantee(),
        mycelium_core::GuaranteeStrength::Proven
    );
    let space = DenseSpace::new(3, ScalarKind::F32).unwrap();
    match y.meta().bound() {
        Some(Bound {
            kind: BoundKind::Error { eps, norm },
            basis: BoundBasis::ProvenThm { citation },
        }) => {
            // ε is the kernel's disclosed absolute accumulation bound over the computed
            // abs-product sum (3.0 + 0.5 + 2.0) — NOT op_rel_eps (the dtype ε never enters).
            assert_eq!(*eps, space.dot_abs_eps(3.0 + 0.5 + 2.0));
            assert_eq!(*norm, NormKind::Linf);
            assert!(
                citation.contains("Higham"),
                "the EXPLAIN-able citation must name its theorem basis: {citation}"
            );
        }
        other => panic!("expected the kernel's ProvenThm Linf bound, got {other:?}"),
    }
    match y.meta().provenance() {
        Provenance::Derived { op, inputs } => {
            assert_eq!(op, &mycelium_core::operation_hash("dense.dot"));
            assert_eq!(inputs, &vec![a.content_hash(), b.content_hash()]);
        }
        other => panic!("expected Derived provenance, got {other:?}"),
    }
}

#[test]
fn dense_similarity_accept_paths_and_zero_convention() {
    let reg = PrimRegistry::with_builtins();
    let f = reg.get("dense.similarity").expect("registered");
    let a = dense_f32(vec![1.0, 0.0]);
    let b = dense_f32(vec![0.0, 1.0]);
    let space = DenseSpace::new(2, ScalarKind::F32).unwrap();
    // Orthogonal → exactly 0 (products are 0 each).
    let y = f("dense.similarity", &[&a, &b]).expect("similarity is total over on-grid inputs");
    assert_eq!(
        y.repr(),
        &Repr::Dense {
            dim: 1,
            dtype: ScalarKind::F64
        }
    );
    assert_eq!(y.payload(), &Payload::Scalars(vec![0.0]));
    assert_eq!(
        y.meta().guarantee(),
        mycelium_core::GuaranteeStrength::Proven
    );
    match y.meta().bound() {
        Some(Bound {
            kind: BoundKind::Error { eps, norm },
            basis: BoundBasis::ProvenThm { .. },
        }) => {
            assert_eq!(*eps, space.similarity_abs_eps());
            assert_eq!(*norm, NormKind::Linf);
        }
        other => panic!("expected the kernel's ProvenThm Linf bound, got {other:?}"),
    }
    // Self-similarity is 1 within the disclosed ε.
    let s = f("dense.similarity", &[&a, &a]).expect("self-similarity");
    let Payload::Scalars(sim) = s.payload() else {
        panic!("similarity must return scalars")
    };
    assert!((sim[0] - 1.0).abs() <= space.similarity_abs_eps());
    // The zero-norm convention (documented in the citation): exactly 0, never silent.
    let z = dense_f32(vec![0.0, 0.0]);
    let zc = f("dense.similarity", &[&a, &z]).expect("zero-norm convention");
    assert_eq!(zc.payload(), &Payload::Scalars(vec![0.0]));
}

#[test]
fn dense_measurement_reject_surface_is_never_silent() {
    let reg = PrimRegistry::with_builtins();
    for prim in ["dense.dot", "dense.similarity"] {
        let f = reg.get(prim).expect("registered");
        let a = dense_f32(vec![1.0, 2.0]);
        // Dim mismatch → explicit PrimType naming the offense — never a broadcast (G2).
        let b3 = dense_f32(vec![1.0, 2.0, 3.0]);
        let err = f(prim, &[&a, &b3]).expect_err("dim mismatch must refuse");
        match err {
            EvalError::PrimType { prim: p, why } => {
                assert_eq!(p, prim);
                assert!(why.contains("dimension mismatch"), "{prim}: {why}");
            }
            other => panic!("{prim}: expected PrimType, got {other:?}"),
        }
        // Dtype mismatch → explicit PrimType, never a re-round.
        let bf = DenseSpace::new(2, ScalarKind::Bf16)
            .unwrap()
            .value(vec![1.5, -2.0])
            .unwrap();
        assert!(matches!(
            f(prim, &[&a, &bf]),
            Err(EvalError::PrimType { .. })
        ));
        // A non-Dense operand (either side) → explicit PrimType.
        let bits = byte([true; 8]);
        assert!(matches!(
            f(prim, &[&bits, &a]),
            Err(EvalError::PrimType { .. })
        ));
        assert!(matches!(
            f(prim, &[&a, &bits]),
            Err(EvalError::PrimType { .. })
        ));
        // Wrong arity → explicit PrimType.
        assert!(matches!(f(prim, &[&a]), Err(EvalError::PrimType { .. })));
        // An approximate source → ApproxCompositionUnsupported (no composition rule yet).
        let approx = reg.get("dense.add").unwrap()("dense.add", &[&a, &dense_f32(vec![0.5, 0.5])])
            .expect("a Proven value");
        assert!(matches!(
            f(prim, &[&a, &approx]),
            Err(EvalError::ApproxCompositionUnsupported { .. })
        ));
    }
}

/// **Property test (the disclosed bound):** over cases whose *true* real-arithmetic dot is known
/// analytically, the computed payload differs from the truth by at most the ε **the value's own
/// bound discloses** — including the catastrophic-cancellation case (`fl(2⁶⁰ + 1) = 2⁶⁰`, so the
/// computed dot is 0 against a true 1) where a per-element relative claim (`op_rel_eps`) would be
/// flat-out false. The absolute accumulation bound must (and does) cover it.
#[test]
fn dense_dot_respects_its_own_disclosed_bound() {
    let reg = PrimRegistry::with_builtins();
    let f = reg.get("dense.dot").expect("registered");
    let two30 = f64::from(2f32.powi(30));
    let cases: [(&[f64], &[f64], f64); 4] = [
        (&[1.5, 2.0, -0.5], &[2.0, 0.25, 4.0], 1.5),
        (&[1.0, 2.0, 3.0, 4.0], &[4.0, 3.0, 2.0, 1.0], 20.0),
        (&[0.0, 0.0], &[1.0, -1.0], 0.0),
        (&[two30, 1.0, -two30], &[two30, 1.0, two30], 1.0),
    ];
    for (xs, ys, exact) in cases {
        let a = dense_f32(xs.to_vec());
        let b = dense_f32(ys.to_vec());
        let y = f("dense.dot", &[&a, &b]).expect("in-range dot");
        let Payload::Scalars(out) = y.payload() else {
            panic!("dense.dot must return Payload::Scalars")
        };
        let Some(Bound {
            kind: BoundKind::Error { eps, .. },
            ..
        }) = y.meta().bound()
        else {
            panic!("dense.dot must carry its Error bound")
        };
        assert!(
            (out[0] - exact).abs() <= *eps,
            "|{} − {exact}| exceeds the value's own disclosed ε = {eps}",
            out[0]
        );
    }
}

// ---- ADR-040 §2.5 (M-898, `enb` Gap A): the scalar-float arithmetic group ----------------------
//
// The reference-case corpus below is the **evidence** behind the `EmpiricalFit` basis every
// `flt.*` result carries (ADR-040 §2.6): every expected value is **hand-derived from IEEE-754
// binary64 RNE semantics** (exact-arithmetic rows, ties-to-even at the 2^53 boundary,
// overflow/underflow edges, signed zeros, the specials algebra, canonical-NaN identity) and
// written as an independent literal/constant — never recomputed with the op under test. The
// corpus row count is pinned to `FLT_CONFORMANCE_TRIALS`, so the trials the basis *records*
// equal the trials actually *run* (VR-5 — evidence never drifts from the claim).

use mycelium_core::{FloatWidth, GuaranteeStrength, CANONICAL_NAN_BITS};

/// An `Exact` `Float{F64}` value (the M-897 float-literal form — the ops' normal input).
fn fv(x: f64) -> Value {
    Value::new(
        Repr::Float {
            width: FloatWidth::F64,
        },
        Payload::Float(x),
        Meta::exact(Provenance::Root),
    )
    .expect("a Float payload matches a Float repr")
}

/// The canonical quiet NaN (the single NaN identity — ADR-040 §2.3).
fn cnan() -> f64 {
    f64::from_bits(CANONICAL_NAN_BITS)
}

/// One reference row: op, operands, and the hand-derived expected bit pattern.
struct FltCase {
    op: &'static str,
    args: Vec<f64>,
    expected: f64,
    why: &'static str,
}

/// The M-898 IEEE-754 binary64 RNE reference corpus (see the section note). Exactly
/// [`FLT_CONFORMANCE_TRIALS`] rows — asserted by `flt_reference_case_corpus`.
fn flt_reference_cases() -> Vec<FltCase> {
    let c = |op, args: &[f64], expected, why| FltCase {
        op,
        args: args.to_vec(),
        expected,
        why,
    };
    vec![
        // flt.add — exact-arithmetic rows (all operands and results on the dyadic grid).
        c("flt.add", &[1.5, 2.25], 3.75, "exact dyadic sum"),
        c("flt.add", &[0.5, 0.25], 0.75, "exact dyadic sum"),
        c(
            "flt.add",
            &[-1.5, 1.5],
            0.0,
            "IEEE 6.3: x + (−x) is +0 under RNE",
        ),
        c(
            "flt.add",
            &[-0.0, -0.0],
            -0.0,
            "IEEE 6.3: (−0) + (−0) is −0",
        ),
        c(
            "flt.add",
            &[-0.0, 0.0],
            0.0,
            "IEEE 6.3: opposite-signed zeros sum to +0 under RNE",
        ),
        // Ties-to-even at the 2^53 representability edge (spacing 2): 2^53 + 1 is the midpoint
        // of {2^53, 2^53 + 2} → the even mantissa (2^53) wins; (2^53 + 2) + 1 is the midpoint of
        // {2^53 + 2, 2^53 + 4} → the even mantissa (2^53 + 4) wins.
        c(
            "flt.add",
            &[9_007_199_254_740_992.0, 1.0],
            9_007_199_254_740_992.0,
            "RNE tie at 2^53: midpoint rounds to the even mantissa (down)",
        ),
        c(
            "flt.add",
            &[9_007_199_254_740_994.0, 1.0],
            9_007_199_254_740_996.0,
            "RNE tie at 2^53 + 3: midpoint rounds to the even mantissa (up)",
        ),
        c(
            "flt.add",
            &[f64::MAX, f64::MAX],
            f64::INFINITY,
            "overflow → +inf, in-band (ratified FLAG-2)",
        ),
        c(
            "flt.add",
            &[-f64::MAX, -f64::MAX],
            f64::NEG_INFINITY,
            "overflow → −inf, in-band",
        ),
        c("flt.add", &[f64::INFINITY, 1.0], f64::INFINITY, "inf + finite = inf"),
        c(
            "flt.add",
            &[f64::INFINITY, f64::NEG_INFINITY],
            f64::from_bits(CANONICAL_NAN_BITS),
            "inf + (−inf) is invalid → NaN (canonical)",
        ),
        c(
            "flt.add",
            &[f64::from_bits(CANONICAL_NAN_BITS), 1.0],
            f64::from_bits(CANONICAL_NAN_BITS),
            "NaN propagates (canonical)",
        ),
        // flt.sub.
        c("flt.sub", &[3.75, 1.5], 2.25, "exact dyadic difference"),
        c("flt.sub", &[1.0, 1.0], 0.0, "x − x is +0 under RNE"),
        c("flt.sub", &[0.0, 0.0], 0.0, "(+0) − (+0) is +0 under RNE"),
        c(
            "flt.sub",
            &[-0.0, 0.0],
            -0.0,
            "(−0) − (+0) is (−0) + (−0) = −0",
        ),
        c(
            "flt.sub",
            &[f64::INFINITY, f64::INFINITY],
            f64::from_bits(CANONICAL_NAN_BITS),
            "inf − inf is invalid → NaN (canonical)",
        ),
        c(
            "flt.sub",
            &[1.0, f64::INFINITY],
            f64::NEG_INFINITY,
            "finite − inf = −inf",
        ),
        c(
            "flt.sub",
            &[f64::MAX, -f64::MAX],
            f64::INFINITY,
            "overflow → +inf, in-band",
        ),
        // flt.mul.
        c("flt.mul", &[1.5, 2.0], 3.0, "exact dyadic product"),
        c("flt.mul", &[-1.5, 2.0], -3.0, "exact dyadic product, sign rule"),
        c("flt.mul", &[0.5, 0.5], 0.25, "exact dyadic product"),
        c(
            "flt.mul",
            &[f64::MAX, 2.0],
            f64::INFINITY,
            "overflow → +inf, in-band",
        ),
        c(
            "flt.mul",
            &[0.0, f64::INFINITY],
            f64::from_bits(CANONICAL_NAN_BITS),
            "0 × inf is invalid → NaN (canonical)",
        ),
        c("flt.mul", &[-1.0, 0.0], -0.0, "IEEE sign rule: (−1) × (+0) = −0"),
        c(
            "flt.mul",
            &[f64::INFINITY, -2.0],
            f64::NEG_INFINITY,
            "inf × negative = −inf",
        ),
        // Underflow at the subnormal floor (spacing 2^-1074): (2^-1074) × 0.5 = 2^-1075 is the
        // midpoint of {0, 2^-1074} → the even candidate (0) wins under RNE.
        c(
            "flt.mul",
            &[5e-324, 0.5],
            0.0,
            "RNE tie at the subnormal floor: midpoint rounds to the even candidate 0",
        ),
        // flt.div.
        c("flt.div", &[3.0, 2.0], 1.5, "exact dyadic quotient"),
        c(
            "flt.div",
            &[1.0, 0.0],
            f64::INFINITY,
            "div-by-zero → +inf, in-band (never a trap — ratified FLAG-2)",
        ),
        c(
            "flt.div",
            &[-1.0, 0.0],
            f64::NEG_INFINITY,
            "div-by-zero, sign rule → −inf",
        ),
        c(
            "flt.div",
            &[1.0, -0.0],
            f64::NEG_INFINITY,
            "div by −0, sign rule → −inf (−0 is observably distinct — ADR-040 §2.3)",
        ),
        c(
            "flt.div",
            &[0.0, 0.0],
            f64::from_bits(CANONICAL_NAN_BITS),
            "0/0 is invalid → NaN (canonical)",
        ),
        c(
            "flt.div",
            &[f64::INFINITY, f64::INFINITY],
            f64::from_bits(CANONICAL_NAN_BITS),
            "inf/inf is invalid → NaN (canonical)",
        ),
        c("flt.div", &[1.0, f64::INFINITY], 0.0, "finite/inf = +0"),
        c("flt.div", &[-1.0, f64::INFINITY], -0.0, "finite/inf, sign rule = −0"),
        // flt.neg — sign-bit flip (exact; never rounds).
        c("flt.neg", &[1.5], -1.5, "sign flip"),
        c("flt.neg", &[0.0], -0.0, "neg(+0) = −0 (bit-distinct — ADR-040 §2.3)"),
        c("flt.neg", &[-0.0], 0.0, "neg(−0) = +0"),
        c("flt.neg", &[f64::INFINITY], f64::NEG_INFINITY, "neg(inf) = −inf"),
        c(
            "flt.neg",
            &[f64::from_bits(CANONICAL_NAN_BITS)],
            f64::from_bits(CANONICAL_NAN_BITS),
            "neg(NaN) re-canonicalizes: NaN sign/payload bits are not observable (§2.3)",
        ),
    ]
}

/// **The conformance corpus (the `EmpiricalFit` evidence):** every row's delivered bit pattern
/// equals its hand-derived IEEE-754 RNE reference **bit-for-bit** (a payload `==` would pass
/// `-0.0 == 0.0` and fail NaN — bits do neither), and the row count equals the
/// `FLT_CONFORMANCE_TRIALS` the basis records.
#[test]
fn flt_reference_case_corpus() {
    let reg = PrimRegistry::with_builtins();
    let cases = flt_reference_cases();
    assert_eq!(
        cases.len() as u64,
        FLT_CONFORMANCE_TRIALS,
        "the recorded trials must equal the trials actually run (VR-5)"
    );
    for case in &cases {
        let f = reg.get(case.op).expect("flt prim registered");
        let args: Vec<Value> = case.args.iter().copied().map(fv).collect();
        let argrefs: Vec<&Value> = args.iter().collect();
        let y = f(case.op, &argrefs).unwrap_or_else(|e| {
            panic!("{}({:?}) must be total, got {e:?}", case.op, case.args)
        });
        let Payload::Float(x) = y.payload() else {
            panic!("{}: result payload must be Float", case.op)
        };
        assert_eq!(
            x.to_bits(),
            case.expected.to_bits(),
            "{}({:?}): got {x:?}, want {:?} — {}",
            case.op,
            case.args,
            case.expected,
            case.why
        );
        assert_eq!(
            y.repr(),
            &Repr::Float {
                width: FloatWidth::F64
            },
            "{}: result repr must be Float{{F64}}",
            case.op
        );
    }
}

/// A value corpus for the property sweeps: finite grid points (exact + inexact decimals),
/// signed zeros, subnormals, the finite extremes, both infinities, and the canonical NaN.
fn flt_value_corpus() -> Vec<f64> {
    vec![
        0.0,
        -0.0,
        1.0,
        -1.0,
        1.5,
        -2.5,
        0.1,
        0.2,
        1.0 / 3.0,
        1e10,
        -1e-300,
        5e-324,
        f64::MAX,
        -f64::MAX,
        f64::MIN_POSITIVE,
        9_007_199_254_740_992.0,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::from_bits(CANONICAL_NAN_BITS),
    ]
}

/// **Property (commutativity, bit-exact):** `flt.add`/`flt.mul` are commutative bit-for-bit over
/// the whole corpus — including specials and NaN, because every NaN result is canonical (one NaN,
/// one bit pattern; ADR-040 §2.3 is what makes float commutativity *bit*-exact, not just IEEE-==).
#[test]
fn flt_add_mul_commute_bitwise_on_the_corpus() {
    let reg = PrimRegistry::with_builtins();
    for op in ["flt.add", "flt.mul"] {
        let f = reg.get(op).expect("registered");
        for &a in &flt_value_corpus() {
            for &b in &flt_value_corpus() {
                let (va, vb) = (fv(a), fv(b));
                let xy = f(op, &[&va, &vb]).expect("total");
                let yx = f(op, &[&vb, &va]).expect("total");
                let (Payload::Float(p), Payload::Float(q)) = (xy.payload(), yx.payload()) else {
                    panic!("{op}: float results expected")
                };
                assert_eq!(
                    p.to_bits(),
                    q.to_bits(),
                    "{op}({a:?}, {b:?}) must commute bit-exactly"
                );
            }
        }
    }
}

/// **Property (additive identity):** `x + 0.0` is IEEE-equal to `x` for every non-NaN `x`, and
/// bit-identical for every `x` except `−0.0` (where IEEE itself defines `−0 + (+0) = +0` under
/// RNE — the documented identity-vs-equality seam, ADR-040 FLAG-4).
#[test]
fn flt_add_zero_is_the_identity_modulo_ieee() {
    let reg = PrimRegistry::with_builtins();
    let f = reg.get("flt.add").expect("registered");
    let zero = fv(0.0);
    for &x in &flt_value_corpus() {
        let vx = fv(x);
        let y = f("flt.add", &[&vx, &zero]).expect("total");
        let Payload::Float(out) = y.payload() else {
            panic!("float result expected")
        };
        if x.is_nan() {
            assert_eq!(out.to_bits(), CANONICAL_NAN_BITS, "NaN + 0 is canonical NaN");
        } else {
            assert_eq!(*out, x, "x + 0.0 must be IEEE-equal to x (x = {x:?})");
            if x.to_bits() != (-0.0f64).to_bits() {
                assert_eq!(out.to_bits(), x.to_bits(), "bit-identity for x ≠ −0.0");
            }
        }
    }
}

/// **Property (involution):** `flt.neg ∘ flt.neg` is a bit-identity over the whole corpus — the
/// signed zeros round-trip (`+0 → −0 → +0`), the infinities round-trip, and NaN re-canonicalizes
/// to itself.
#[test]
fn flt_neg_neg_is_a_bit_identity() {
    let reg = PrimRegistry::with_builtins();
    let f = reg.get("flt.neg").expect("registered");
    for &x in &flt_value_corpus() {
        let vx = fv(x);
        let once = f("flt.neg", &[&vx]).expect("total");
        let twice = f("flt.neg", &[&once]).expect("total");
        let Payload::Float(out) = twice.payload() else {
            panic!("float result expected")
        };
        assert_eq!(
            out.to_bits(),
            x.to_bits(),
            "neg(neg({x:?})) must be a bit-identity"
        );
    }
}

/// **Property (one NaN, one address — ADR-040 §2.3):** every NaN any `flt.*` op produces over the
/// corpus carries exactly the canonical bits — no constructor path yields a non-canonical NaN.
#[test]
fn flt_nan_results_are_always_canonical() {
    let reg = PrimRegistry::with_builtins();
    for op in ["flt.add", "flt.sub", "flt.mul", "flt.div"] {
        let f = reg.get(op).expect("registered");
        for &a in &flt_value_corpus() {
            for &b in &flt_value_corpus() {
                let (va, vb) = (fv(a), fv(b));
                let y = f(op, &[&va, &vb]).expect("total");
                let Payload::Float(out) = y.payload() else {
                    panic!("float result expected")
                };
                if out.is_nan() {
                    assert_eq!(
                        out.to_bits(),
                        CANONICAL_NAN_BITS,
                        "{op}({a:?}, {b:?}): NaN must be canonical"
                    );
                }
            }
        }
    }
}

/// **The ADR-040 §2.6 tag contract, inspectable off the value (EXPLAIN — G2/SC-3):** every
/// `flt.*` result over `Exact` inputs is `Empirical` with the zero-deviation-vs-spec bound
/// (`eps = 0`, `Linf`) on the `EmpiricalFit{FLT_CONFORMANCE_TRIALS, …}` basis, with `Derived`
/// provenance — and the Π table's intrinsic agrees with what the wrapper delivers (the DN-10
/// §3.4 table↔kernel consistency, float form).
#[test]
fn flt_results_carry_the_adr040_empirical_tag_and_bound() {
    let reg = PrimRegistry::with_builtins();
    let table = PrimTable::builtins();
    let one = fv(1.0);
    let half = fv(0.5);
    for op in ["flt.add", "flt.sub", "flt.mul", "flt.div", "flt.neg"] {
        let f = reg.get(op).expect("registered");
        let args: Vec<&Value> = if op == "flt.neg" {
            vec![&half]
        } else {
            vec![&one, &half]
        };
        let y = f(op, &args).expect("total");
        assert_eq!(
            y.meta().guarantee(),
            GuaranteeStrength::Empirical,
            "{op}: the per-op tag is the ratified ADR-040 §2.6 Empirical (VR-5)"
        );
        assert_eq!(
            table.intrinsic(op),
            Some(GuaranteeStrength::Empirical),
            "{op}: Π intrinsic must agree with the delivered tag (DN-10 §3.4)"
        );
        match y.meta().bound() {
            Some(Bound {
                kind: BoundKind::Error { eps, norm },
                basis: BoundBasis::EmpiricalFit { trials, method },
            }) => {
                assert_eq!(*eps, 0.0, "{op}: zero deviation vs the RNE spec");
                assert_eq!(*norm, NormKind::Linf);
                assert_eq!(
                    *trials, FLT_CONFORMANCE_TRIALS,
                    "{op}: the basis records the corpus actually run"
                );
                assert!(!method.trim().is_empty());
            }
            other => panic!("{op}: expected the EmpiricalFit zero-deviation bound, got {other:?}"),
        }
        assert!(
            matches!(y.meta().provenance(), Provenance::Derived { .. }),
            "{op}: provenance must be Derived"
        );
    }
}

/// **Composition:** a `flt.*` result (Empirical, zero-deviation) is a legal input to the next
/// `flt.*` op — chained float arithmetic composes, and the chained result keeps the same honest
/// tag/bound form. An input carrying a *genuine* approximation bound (`eps > 0`) is an explicit
/// [`EvalError::ApproxCompositionUnsupported`] — no defined float ε-rule yet, refused, never
/// fabricated (G2/VR-5).
#[test]
fn flt_chaining_composes_and_true_approximations_refuse() {
    let reg = PrimRegistry::with_builtins();
    let add = reg.get("flt.add").expect("registered");
    let mul = reg.get("flt.mul").expect("registered");
    // Chain: (1.5 × 2.0) + 0.25 = 3.25 — the intermediate is Empirical and composes.
    let prod = mul("flt.mul", &[&fv(1.5), &fv(2.0)]).expect("total");
    assert_eq!(prod.meta().guarantee(), GuaranteeStrength::Empirical);
    let sum = add("flt.add", &[&prod, &fv(0.25)]).expect("chained flt ops must compose");
    let Payload::Float(out) = sum.payload() else {
        panic!("float result expected")
    };
    assert_eq!(out.to_bits(), 3.25f64.to_bits());
    assert_eq!(sum.meta().guarantee(), GuaranteeStrength::Empirical);
    // A genuinely-approximate Float input (eps > 0) has no defined propagation rule — refuse.
    let approx = Value::new(
        Repr::Float {
            width: FloatWidth::F64,
        },
        Payload::Float(1.0),
        Meta::new(
            Provenance::Root,
            GuaranteeStrength::Empirical,
            Some(Bound {
                kind: BoundKind::Error {
                    eps: 1e-3,
                    norm: NormKind::Rel,
                },
                basis: BoundBasis::EmpiricalFit {
                    trials: 10,
                    method: "a synthetic approximate source".to_owned(),
                },
            }),
            None,
            None,
            None,
        )
        .expect("well-formed meta"),
    )
    .expect("well-formed value");
    assert!(
        matches!(
            add("flt.add", &[&approx, &fv(1.0)]),
            Err(EvalError::ApproxCompositionUnsupported { .. })
        ),
        "a true approximation must refuse explicitly, never a fabricated bound"
    );
}

/// **Never-silent type/arity discipline:** a non-`Float` operand and a wrong arity are explicit
/// [`EvalError::PrimType`] refusals — never a coercion (G2).
#[test]
fn flt_type_and_arity_refusals_are_never_silent() {
    let reg = PrimRegistry::with_builtins();
    let add = reg.get("flt.add").expect("registered");
    let neg = reg.get("flt.neg").expect("registered");
    let b = byte([false; 8]);
    let x = fv(1.0);
    assert!(
        matches!(add("flt.add", &[&b, &x]), Err(EvalError::PrimType { .. })),
        "a Binary operand must refuse"
    );
    assert!(
        matches!(add("flt.add", &[&x]), Err(EvalError::PrimType { .. })),
        "arity 1 for flt.add must refuse"
    );
    assert!(
        matches!(neg("flt.neg", &[&x, &x]), Err(EvalError::PrimType { .. })),
        "arity 2 for flt.neg must refuse"
    );
}
