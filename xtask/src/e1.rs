//! `cargo xtask e1` — the **E1 perf harness** (M-250 codec stub + M-303 native-path measurement;
//! RFC-0004 §5/§8; DN-01 E1; ADR-009).
//!
//! E1 (RFC-0004 §8) asks whether the *schedule-staged* packing path reaches hand-packed performance
//! for the small fixed scheme set — expected easy per T1.4. The full **compute-throughput** answer
//! needs *in-process* native execution (JIT/FFI; the libMLIR backend or M-340 JIT, deferred —
//! ADR-009); a standalone tiny-kernel artifact compiled here is process-spawn-bound (and constant
//! inputs constant-fold), so this harness does **not** pronounce "reaches hand-packed perf" (VR-5).
//!
//! What it honestly measures, in two sections:
//! 1. **Codec cost** — `pack`/`unpack` round-trip throughput per scheme over the `mycelium_mlir::pack`
//!    codec the E3 differential (M-251) exercises. The build-phase confirmation that staging is
//!    cheap to *materialize*.
//! 2. **Native AOT path** (M-303) — now that the direct-LLVM backend exists (`mycelium_mlir::compile`),
//!    the one-time **AOT compile cost** (emit IR → `llc` → `clang`), the warm **per-invocation** cost
//!    (process spawn + run — spawn-dominated for the trivial kernel, captioned as such), and the
//!    reference **interpreter** per-eval cost, for a bit-subset program. Real numbers, honestly
//!    bounded; skips when `llc`/`clang` are absent.
//!
//! No benchmarking dependency (house style): a warmup pass, then the minimum mean over several
//! batches. Run with `--release` (`cargo run --release -p xtask -- e1`); a debug build is refused.

use std::hint::black_box;
use std::time::Instant;

use mycelium_core::{Meta, Node, PackScheme, Payload, Provenance, Repr, Trit, Value};
use mycelium_interp::{IdentitySwapEngine, Interpreter, PrimRegistry};
use mycelium_mlir::pack::{pack_trits, unpack_trits};
use mycelium_mlir::{compile, AotError};

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

    println!("== E1 §1: staged-packing codec round-trip (pack+unpack) over {DIM} trits ==");
    println!("  (substrate codec cost — confirms staging is cheap to materialize)\n");
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

    native_section();

    println!(
        "\nE1 verdict: native AOT path **established and measured** (M-303) — was: no native path. \
         A calibrated *compute-throughput* verdict (\"reaches hand-packed perf\") remains NOT \
         established: the standalone tiny-kernel artifact is process-spawn-bound and constant-folds, \
         so it needs in-process execution (JIT/FFI — M-340 / the deferred libMLIR backend). Honest \
         per VR-5 — numbers reported, no perf claim pre-written."
    );
}

/// E1 §2 (M-303): time the native AOT path — one-time compile vs warm per-invocation — against the
/// reference interpreter, on a representative bit-subset program (`not(a xor b)` over 8 bits). Skips
/// gracefully when `llc`/`clang` are absent.
fn native_section() {
    println!("\n== E1 §2: native AOT path vs interpreter (bit subset, M-303) ==");

    let prog = not_a_xor_b();

    // One-time AOT compile (emit IR -> llc -> clang).
    let t = Instant::now();
    let artifact = match compile(&prog) {
        Ok(a) => a,
        Err(AotError::ToolchainMissing(tool)) => {
            println!("  skip: native toolchain absent ({tool}) — install llc/clang to measure.");
            return;
        }
        Err(e) => {
            eprintln!("  native compile failed: {e}");
            return;
        }
    };
    #[allow(clippy::cast_precision_loss)]
    let compile_ns = t.elapsed().as_nanos() as f64;

    // Warm native per-invocation: process spawn + run + read-back. Fewer iters — each is a spawn.
    let native_ns = bench(40, || {
        black_box(artifact.run().expect("artifact run"));
    });

    // Reference interpreter, in-process per-eval.
    let interp = Interpreter::new(PrimRegistry::with_builtins(), Box::new(IdentitySwapEngine));
    let interp_ns = bench(20_000, || {
        black_box(interp.eval(black_box(&prog)).expect("interp eval"));
    });

    println!("  AOT compile (emit+llc+clang), one-time : {compile_ns:>12.0} ns");
    println!(
        "  native per-invocation (spawn+run, warm) : {native_ns:>12.0} ns  [process-spawn-bound]"
    );
    println!("  interpreter per-eval (in-process)       : {interp_ns:>12.0} ns");
    println!(
        "  note: the per-invocation figure is dominated by process spawn for this trivial kernel, \
         not kernel compute — see the verdict."
    );
}

/// `not(a xor b)` over two 8-bit constants — a representative straight-line bit-subset program.
fn not_a_xor_b() -> Node {
    let a = byte([true, false, true, true, false, false, true, false]);
    let b = byte([false, false, true, false, true, false, true, true]);
    Node::Op {
        prim: "bit.not".into(),
        args: vec![Node::Op {
            prim: "bit.xor".into(),
            args: vec![Node::Const(a), Node::Const(b)],
        }],
    }
}

fn byte(bits: [bool; 8]) -> Value {
    Value::new(
        Repr::Binary { width: 8 },
        Payload::Bits(bits.to_vec()),
        Meta::exact(Provenance::Root),
    )
    .expect("valid byte")
}
