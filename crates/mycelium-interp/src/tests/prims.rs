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
