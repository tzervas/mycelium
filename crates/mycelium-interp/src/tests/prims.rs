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
