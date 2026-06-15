//! `cargo xtask e1` — the **E1 staged-packing perf-harness stub** (M-250; RFC-0004 §5/§8; DN-01
//! E1).
//!
//! E1 (RFC-0004 §8) asks whether the *schedule-staged* packing path reaches hand-packed performance
//! for the small fixed scheme set — expected easy per T1.4 (≈5 schemes, materially easier than
//! Halide's exponential search). The *full* answer needs the native libMLIR/LLVM backend (deferred;
//! ADR-009), so a calibrated kernel benchmark is **not** buildable here.
//!
//! What this stub honestly measures: the **substrate codec cost** — `pack`/`unpack` round-trip
//! throughput per scheme over the `mycelium_mlir::pack` codec the E3 differential (M-251) exercises.
//! It is the build-phase confirmation that staging is cheap to *materialize*, recorded as the E1
//! placeholder until the native path lands. It is a stub, not the E1 verdict (VR-5 — never
//! pre-written): it reports numbers, it does not pronounce "reaches hand-packed perf".
//!
//! No benchmarking dependency (house style): a warmup pass, then the minimum mean over several
//! batches. Run with `--release` (`cargo run --release -p xtask -- e1`); a debug build is refused.

use std::hint::black_box;
use std::time::Instant;

use mycelium_core::{PackScheme, Trit};
use mycelium_mlir::pack::{pack_trits, unpack_trits};

const BATCHES: usize = 5;

/// The fixed bitnet.cpp scheme set, plus the two reference packings, in the codec.
const SCHEMES: [(&str, PackScheme); 5] = [
    ("I2_S (2.0 b/w)", PackScheme::I2S),
    ("TL1  (2.0 b/w)", PackScheme::Tl1),
    ("TL2  (1.67 b/w)", PackScheme::Tl2),
    ("TwoBitPerTrit", PackScheme::TwoBitPerTrit),
    ("FiveTritPerByte", PackScheme::FiveTritPerByte),
];

/// A deterministic ternary buffer of `n` trits (LCG over `{Neg, Zero, Pos}`).
fn trits(n: usize) -> Vec<Trit> {
    let mut state = 0x5EED_1234_u64;
    (0..n)
        .map(|_| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            match (state >> 33) % 3 {
                0 => Trit::Neg,
                1 => Trit::Zero,
                _ => Trit::Pos,
            }
        })
        .collect()
}

/// Mean ns/call of the fastest batch of `iters` calls (after one warmup batch).
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
        #[allow(clippy::cast_precision_loss)]
        let per_call = t.elapsed().as_nanos() as f64 / f64::from(iters);
        best = best.min(per_call);
    }
    best
}

pub fn run() {
    if cfg!(debug_assertions) {
        eprintln!(
            "xtask e1: refusing to measure a debug build — run with `--release` \
             (`cargo run --release -p xtask -- e1`)."
        );
        std::process::exit(2);
    }

    const DIM: usize = 4096;
    let buf = trits(DIM);
    let iters = 2_000u32;

    println!("E1 staged-packing codec round-trip (pack+unpack) over {DIM} trits — STUB:");
    println!("  (substrate codec cost only; the native-backend kernel benchmark is deferred — ADR-009)\n");
    for (name, scheme) in SCHEMES {
        let ns = bench(iters, || {
            let bytes = pack_trits(black_box(&buf), scheme);
            let back = unpack_trits(black_box(&bytes), scheme, DIM).expect("round-trip unpack");
            black_box(back);
        });
        #[allow(clippy::cast_precision_loss)]
        let per_trit = ns / DIM as f64;
        println!("  {name:<18} round-trip {ns:>10.0} ns   {per_trit:>7.3} ns/trit");
    }
    println!(
        "\nE1 verdict: NOT established (stub). This confirms staging is cheap to materialize; \
         \"reaches hand-packed perf\" awaits the native path."
    );
}
