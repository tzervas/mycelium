//! M-212 — **SC-3 global**: every *implemented* row of the RFC-0002 §5 legal-pair table emits a
//! certificate and validates through the one M-210 checker; every unimplemented pair is an
//! explicit error through the engine — the swap surface is never silent, anywhere.
//!
//! Implemented rows today: Binary↔Ternary in range (`LosslessWithinRange`, bijective; M-120),
//! Binary↔Ternary out of range / illegal pair (explicit rejection), Dense `F32→BF16` (Bounded ε;
//! M-211). The remaining rows (Dense↔VSA, VSA↔VSA — M-231/M-242) do not exist yet and are
//! asserted to *fail explicitly*, which is exactly what SC-3 requires of an unimplemented swap.

use mycelium_cert::{
    binary_to_ternary, check, dense_f32_to_bf16, ternary_to_binary, CertifiedSwapEngine,
    CheckVerdict, Evidence, RefinementRelation, SwapError, BF16_REL_EPS,
};
use mycelium_core::{
    binary, ContentHash, GuaranteeStrength, Meta, Payload, Provenance, Repr, ScalarKind,
    SparsityClass, Value,
};
use mycelium_interp::{EvalError, SwapEngine};
use mycelium_numerics::Certificate;

fn policy() -> ContentHash {
    ContentHash::parse("blake3:po1icy_Ref00").unwrap()
}

fn bin_of(value: i64, width: u32) -> Value {
    Value::new(
        Repr::Binary { width },
        Payload::Bits(binary::int_to_bits(value, width).unwrap()),
        Meta::exact(Provenance::Root),
    )
    .unwrap()
}

fn dense_f32(xs: Vec<f64>) -> Value {
    Value::new(
        Repr::Dense {
            dim: u32::try_from(xs.len()).unwrap(),
            dtype: ScalarKind::F32,
        },
        Payload::Scalars(xs),
        Meta::exact(Provenance::Root),
    )
    .unwrap()
}

fn assert_validated(v: CheckVerdict, what: &str) {
    assert!(
        matches!(v, CheckVerdict::Validated { .. }),
        "{what}: must validate, got {v:?}"
    );
}

/// **SC-3 global, positive half:** every implemented legal-pair row emits a certificate *and*
/// that certificate validates through the single shared checker — across several `(n, m)` pairs
/// and a Dense vector with mixed magnitudes/signs/zero.
#[test]
fn every_implemented_swap_emits_and_validates_a_certificate() {
    // Bijective rows: a few legal (n, m) pairs, both directions.
    for &(n, m) in &[(8, 6), (4, 3), (10, 7), (16, 11)] {
        let a = bin_of(if n >= 8 { -100 } else { -5 }, n);
        let (b, cert) = binary_to_ternary(&a, m, &policy()).expect("enc must emit a certificate");
        assert_validated(
            check(
                &a,
                &b,
                RefinementRelation::Bijection,
                Certificate::exact(),
                &Evidence::Swap(&cert),
            ),
            "bijective enc",
        );
        let (back, dec_cert) =
            ternary_to_binary(&b, n, &policy()).expect("dec must emit a certificate");
        assert_validated(
            check(
                &b,
                &back,
                RefinementRelation::Bijection,
                Certificate::exact(),
                &Evidence::Swap(&dec_cert),
            ),
            "bijective dec",
        );
    }
    // Bounded row: Dense F32 → BF16.
    let a = dense_f32(vec![1.5, -0.625, 0.0, f64::from(2.5e10_f32), -3.0]);
    let (b, cert) = dense_f32_to_bf16(&a, &policy()).expect("bounded swap must emit a certificate");
    let claimed = Certificate::new(BF16_REL_EPS, 0.0, GuaranteeStrength::Proven).unwrap();
    assert_validated(
        check(
            &a,
            &b,
            RefinementRelation::BoundedSimilarity,
            claimed,
            &Evidence::Swap(&cert),
        ),
        "bounded F32→BF16",
    );
}

/// **SC-3 global, negative half:** the rejected/unimplemented rows of the table are *explicit*
/// errors — out-of-range decode, illegal pair, and every pair with no certified swap yet.
#[test]
fn every_unimplemented_or_illegal_pair_is_explicit() {
    // Out of range: all-+ 6 trits (364) does not fit in 8 bits — rejected, never wrapped.
    let big = Value::new(
        Repr::Ternary { trits: 6 },
        Payload::Trits(vec![mycelium_core::Trit::Pos; 6]),
        Meta::exact(Provenance::Root),
    )
    .unwrap();
    assert_eq!(
        ternary_to_binary(&big, 8, &policy()),
        Err(SwapError::OutOfRange)
    );
    // Illegal pair: a type error, not a Declared gamble (RFC-0002 §5).
    assert_eq!(
        binary_to_ternary(&bin_of(0, 8), 4, &policy()),
        Err(SwapError::IllegalPair { width: 8, trits: 4 })
    );

    // Unimplemented rows through the complete engine: all explicit `UnsupportedSwap`.
    let engine = CertifiedSwapEngine;
    let dense = dense_f32(vec![1.0, 2.0]);
    let vsa_target = Repr::Vsa {
        model: "MAP-I".to_owned(),
        dim: 64,
        sparsity: SparsityClass::Dense,
    };
    let unsupported = [
        (dense.clone(), vsa_target),                // Dense ↔ VSA: M-231
        (dense.clone(), Repr::Binary { width: 8 }), // no cross-paradigm rule
        (
            bin_of(1, 8),
            Repr::Dense {
                dim: 2,
                dtype: ScalarKind::F32,
            },
        ),
    ];
    for (src, target) in unsupported {
        let r = engine.swap(&src, &target, &policy());
        assert!(
            matches!(r, Err(EvalError::UnsupportedSwap { .. })),
            "{:?} → {target:?} must be an explicit UnsupportedSwap, got {r:?}",
            src.repr()
        );
    }
}
