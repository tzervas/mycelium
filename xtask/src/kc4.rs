//! `cargo xtask kc4` — the **KC-4 per-swap certificate-check overhead measurement** (M-212;
//! Foundation KC-4; RFC-0002 §2).
//!
//! Times, for every implemented swap kind, (a) the swap itself (which always emits its
//! certificate — SC-3) and (b) the M-210 `check` validation of that certificate, and reports the
//! per-call cost and the check/swap overhead ratio. The interp↔AOT observational instance is
//! measured too (it shares the same checker).
//!
//! **Honesty (VR-5).** This prints *measured numbers from this run on this machine* — the KC-4
//! verdict is whatever a run establishes, never pre-written. Run with `--release`
//! (`cargo run --release -p xtask -- kc4`); a debug build is refused (its numbers would be
//! systematically inflated and dishonest to record).
//!
//! No benchmarking dependency (house style): a warmup pass, then the **minimum mean over several
//! batches** of wall-clock time per call — minimum-of-batches is the standard cheap estimator for
//! the noise floor of a short deterministic operation.

use std::hint::black_box;
use std::time::Instant;

use mycelium_cert::{
    binary_to_ternary, check, dense_f32_to_bf16, ternary_to_binary, CheckVerdict, Evidence,
    RefinementRelation, BF16_REL_EPS,
};
use mycelium_core::{
    binary, ternary, ContentHash, GuaranteeStrength, Meta, Payload, Provenance, Repr, ScalarKind,
    Value,
};
use mycelium_numerics::Certificate;

const BATCHES: usize = 5;

fn policy() -> ContentHash {
    ContentHash::parse("blake3:po1icy_Ref00").unwrap()
}

/// Mean ns/call of the *fastest* batch of `iters` calls (after one warmup batch).
fn bench(iters: u32, mut f: impl FnMut()) -> f64 {
    for _ in 0..iters {
        f();
    }
    let mut best = f64::INFINITY;
    for _ in 0..BATCHES {
        let t = Instant::now();
        for _ in 0..iters {
            f();
        }
        #[allow(clippy::cast_precision_loss)] // ns totals here are far below 2^52
        let per_call = t.elapsed().as_nanos() as f64 / f64::from(iters);
        best = best.min(per_call);
    }
    best
}

fn report(kind: &str, swap_ns: f64, check_ns: f64) {
    println!(
        "  {kind:<34} swap {swap_ns:>9.0} ns   check {check_ns:>9.0} ns   check/swap {:>6.2}×",
        check_ns / swap_ns
    );
}

fn assert_validated(v: &CheckVerdict, what: &str) {
    assert!(
        matches!(v, CheckVerdict::Validated { .. }),
        "{what}: the measured configuration must validate, got {v:?}"
    );
}

fn byte_of(v: i64) -> Value {
    Value::new(
        Repr::Binary { width: 8 },
        Payload::Bits(binary::int_to_bits(v, 8).unwrap()),
        Meta::exact(Provenance::Root),
    )
    .unwrap()
}

fn tern_of(v: i64, trits: u32) -> Value {
    Value::new(
        Repr::Ternary { trits },
        Payload::Trits(ternary::int_to_trits(v, trits).unwrap()),
        Meta::exact(Provenance::Root),
    )
    .unwrap()
}

/// A deterministic dense F32 vector (LCG; values exact f32, normal range).
fn dense_f32(dim: u32) -> Value {
    let mut state = 0xE23_0212_u64;
    let xs: Vec<f64> = (0..dim)
        .map(|_| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            // Mantissa in [1, 2), modest exponents — exact f32 by construction.
            #[allow(clippy::cast_possible_truncation)]
            let x = ((((state >> 40) as f64) / f64::from(1u32 << 24) + 1.0)
                * 2.0_f64.powi(i32::try_from((state >> 16) % 21).unwrap() - 10))
                as f32;
            f64::from(x)
        })
        .collect();
    Value::new(
        Repr::Dense {
            dim,
            dtype: ScalarKind::F32,
        },
        Payload::Scalars(xs),
        Meta::exact(Provenance::Root),
    )
    .unwrap()
}

pub fn run() {
    if cfg!(debug_assertions) {
        eprintln!(
            "kc4: refusing to measure a debug build — run `cargo run --release -p xtask -- kc4` \
             (debug numbers would be dishonest to record)"
        );
        std::process::exit(2);
    }
    let pol = policy();
    println!("KC-4 per-swap certificate-check overhead (M-212) — measured this run, this machine:");

    // ---- bijective enc: Binary{8} → Ternary{6} ----
    let a = byte_of(-78);
    let (b, cert) = binary_to_ternary(&a, 6, &pol).unwrap();
    assert_validated(
        &check(
            &a,
            &b,
            RefinementRelation::Bijection,
            Certificate::exact(),
            &Evidence::Swap(&cert),
        ),
        "bijective enc",
    );
    let swap_ns = bench(100_000, || {
        black_box(binary_to_ternary(black_box(&a), 6, &pol).unwrap());
    });
    let check_ns = bench(100_000, || {
        black_box(check(
            black_box(&a),
            black_box(&b),
            RefinementRelation::Bijection,
            Certificate::exact(),
            &Evidence::Swap(black_box(&cert)),
        ));
    });
    report("bijective enc  Binary{8}→Ternary{6}", swap_ns, check_ns);

    // ---- bijective dec: Ternary{6} → Binary{8} ----
    let a = tern_of(101, 6);
    let (b, cert) = ternary_to_binary(&a, 8, &pol).unwrap();
    assert_validated(
        &check(
            &a,
            &b,
            RefinementRelation::Bijection,
            Certificate::exact(),
            &Evidence::Swap(&cert),
        ),
        "bijective dec",
    );
    let swap_ns = bench(100_000, || {
        black_box(ternary_to_binary(black_box(&a), 8, &pol).unwrap());
    });
    let check_ns = bench(100_000, || {
        black_box(check(
            black_box(&a),
            black_box(&b),
            RefinementRelation::Bijection,
            Certificate::exact(),
            &Evidence::Swap(black_box(&cert)),
        ));
    });
    report("bijective dec  Ternary{6}→Binary{8}", swap_ns, check_ns);

    // ---- bounded: Dense{768, F32} → Dense{768, BF16} ----
    let a = dense_f32(768);
    let (b, cert) = dense_f32_to_bf16(&a, &pol).unwrap();
    let claimed = Certificate::new(BF16_REL_EPS, 0.0, GuaranteeStrength::Proven).unwrap();
    assert_validated(
        &check(
            &a,
            &b,
            RefinementRelation::BoundedSimilarity,
            claimed,
            &Evidence::Swap(&cert),
        ),
        "bounded F32→BF16",
    );
    let swap_ns = bench(10_000, || {
        black_box(dense_f32_to_bf16(black_box(&a), &pol).unwrap());
    });
    let check_ns = bench(10_000, || {
        black_box(check(
            black_box(&a),
            black_box(&b),
            RefinementRelation::BoundedSimilarity,
            claimed,
            &Evidence::Swap(black_box(&cert)),
        ));
    });
    report("bounded  Dense{768} F32→BF16", swap_ns, check_ns);

    // ---- observational: interp vs AOT result of a swap program ----
    // The two execution paths produce the values; the checker validates the pair. Reported
    // against the bijective swap cost as the reference unit of work.
    let x = byte_of(42);
    let (y, _) = binary_to_ternary(&x, 6, &pol).unwrap();
    let (y2, _) = binary_to_ternary(&x, 6, &pol).unwrap();
    let check_ns = bench(100_000, || {
        black_box(check(
            black_box(&y),
            black_box(&y2),
            RefinementRelation::ObservationalEquiv,
            Certificate::exact(),
            &Evidence::Observational,
        ));
    });
    println!(
        "  {:<34} check {check_ns:>9.0} ns   (structural observable equality)",
        "observational  interp↔AOT pair"
    );

    println!(
        "\nVerdict input for KC-4: record these numbers in docs/planning/phase-2.md §6.7 — the \
         budget itself is unratified (Foundation: \"an agreed budget\"); ratifying one is a \
         maintainer decision."
    );
}
