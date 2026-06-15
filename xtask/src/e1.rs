//! `cargo xtask e1` — the **E1 perf harness** (M-250 codec stub + M-303 native-path measurement;
//! RFC-0004 §5/§8; DN-01 E1; ADR-009).
//!
//! E1 (RFC-0004 §8) asks whether the *schedule-staged* packing path reaches hand-packed performance
//! for the small fixed scheme set — expected easy per T1.4. The full **compute-throughput** answer
//! needs *in-process* native execution (JIT/FFI; the libMLIR backend or M-340 JIT, deferred —
//! ADR-009); a standalone tiny-kernel artifact compiled here is process-spawn-bound (and constant
//! inputs constant-fold), so this harness does **not** pronounce "reaches hand-packed perf" (VR-5).
//!
//! What it honestly measures, in three sections:
//! 1. **Codec cost** — `pack`/`unpack` round-trip throughput per scheme over the `mycelium_mlir::pack`
//!    codec the E3 differential (M-251) exercises. The build-phase confirmation that staging is
//!    cheap to *materialize*.
//! 2. **Native AOT path** (M-303) — now that the direct-LLVM backend exists (`mycelium_mlir::compile`),
//!    the one-time **AOT compile cost** (emit IR → `llc` → `clang`), the warm **per-invocation** cost
//!    (process spawn + run — spawn-dominated for the trivial kernel, captioned as such), and the
//!    reference **interpreter** per-eval cost, for a bit-subset program. Real numbers, honestly
//!    bounded; skips when `llc`/`clang` are absent.
//! 3. **Packed-ternary compute over runtime data** (M-360) — the BitNet I2_S dot kernel
//!    (`mycelium_mlir::compile_bitnet_dot`) run **in-process** over weight/activation buffers passed
//!    as *runtime pointers*. Because the inputs are not baked-in constants the optimiser cannot fold
//!    the computation away, so this is the first section that times **genuine unpack-compute** (vs §2's
//!    spawn/fold overhead) — the runtime-input kernel the compute-throughput verdict needed. Reported
//!    against the in-Rust scalar oracle; skips when `clang` is absent.
//!
//! No benchmarking dependency (house style): a warmup pass, then the minimum mean over several
//! batches. Run with `--release` (`cargo run --release -p xtask -- e1`); a debug build is refused.

use std::hint::black_box;
use std::time::Instant;

use mycelium_core::{Meta, Node, PackScheme, Payload, Provenance, Repr, Trit, Value};
use mycelium_interp::{IdentitySwapEngine, Interpreter, PrimRegistry};
use mycelium_mlir::pack::{pack_trits, unpack_trits};
use mycelium_mlir::{compile, compile_bitnet_dot, ternary_dot_ref, AotError};

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

    let measured_compute = bitnet_section(&buf, DIM);

    if measured_compute {
        println!(
            "\nE1 verdict: packed-ternary **compute throughput is now measured over runtime data** \
             (M-360, I2_S) — the constant-fold/spawn caveat that blocked §2 is gone: the BitNet dot \
             kernel takes its weight/activation buffers as runtime pointers and runs in-process, so \
             §3 times genuine unpack-compute, not call overhead. Reported against a hand-written Rust \
             scalar baseline doing the identical I2_S unpack-compute. Still open (honest, VR-5/G3): \
             parity with bitnet.cpp's hand-tuned \
             SIMD kernels, and the TL1/TL2 packings — the next M-360 increments. No perf claim is \
             pre-written; the number above is whatever was measured."
        );
    } else {
        println!(
            "\nE1 verdict: native AOT path **established and measured** (M-303). The §3 \
             compute-throughput measurement (M-360) was skipped (no `clang`); install the toolchain \
             to measure packed-ternary compute over runtime data. Honest per VR-5 — no perf claim \
             pre-written."
        );
    }
}

/// E1 §3 (M-360): time the **packed-ternary dot kernel over runtime data** in-process against the
/// Rust scalar oracle. Because the buffers are runtime pointers the kernel cannot constant-fold, so
/// this measures genuine unpack-compute — the number §2 could not honestly report. Returns whether a
/// measurement was taken (false ⇒ `clang` absent, skipped). Skips gracefully.
fn bitnet_section(weights: &[Trit], dim: usize) -> bool {
    println!("\n== E1 §3: packed-ternary dot kernel over runtime data (I2_S, M-360) ==");

    let kernel = match compile_bitnet_dot() {
        Ok(k) => k,
        Err(AotError::ToolchainMissing(tool)) => {
            println!("  skip: native toolchain absent ({tool}) — install clang to measure.");
            return false;
        }
        Err(e) => {
            eprintln!("  BitNet kernel compile failed: {e}");
            return false;
        }
    };

    // Runtime buffers: I2_S-packed ternary weights + a deterministic int activation vector. Passed as
    // pointers, so neither the kernel nor the oracle below can be constant-folded away.
    let packed = pack_trits(weights, PackScheme::I2S);
    let acts = activations(dim);

    // Correctness gate before timing: the JIT kernel must agree with the semantic oracle (on the
    // unpacked weights) *and* with the fair scalar baseline (on the same packed buffer).
    let jit_sum = kernel.call(&packed, &acts, dim).expect("kernel runs");
    let oracle_sum = ternary_dot_ref(weights, &acts);
    let baseline_sum = scalar_packed_dot(&packed, &acts, dim);
    assert_eq!(
        jit_sum, oracle_sum,
        "E1 §3: JIT kernel disagrees with the semantic oracle — refusing to time a wrong kernel"
    );
    assert_eq!(
        jit_sum, baseline_sum,
        "E1 §3: scalar baseline disagrees with the oracle"
    );

    // Apples-to-apples: both the JIT and the baseline do the **full I2_S unpack-compute** over the
    // same packed runtime buffer, so the ratio reflects compiled-kernel vs hand-written scalar Rust
    // on identical work (the E1 "reaches hand-packed perf" question), not an unpack-cost asymmetry.
    let iters = 5_000u32;
    let jit_ns = bench(iters, || {
        black_box(
            kernel
                .call(black_box(&packed), black_box(&acts), dim)
                .expect("kernel"),
        );
    });
    let base_ns = bench(iters, || {
        black_box(scalar_packed_dot(black_box(&packed), black_box(&acts), dim));
    });

    #[allow(clippy::cast_precision_loss)]
    let (jit_per, base_per) = (jit_ns / dim as f64, base_ns / dim as f64);
    println!(
        "  JIT packed-ternary dot (in-process)     : {jit_ns:>12.0} ns   {jit_per:>6.3} ns/elem"
    );
    println!(
        "  Rust scalar baseline (same unpack-work) : {base_ns:>12.0} ns   {base_per:>6.3} ns/elem"
    );
    if jit_ns > 0.0 {
        println!(
            "  ratio (baseline / jit)                  : {:>12.2}x",
            base_ns / jit_ns
        );
    }
    println!(
        "  note: both do the full I2_S unpack-compute over the same runtime buffer (no constant \
         folding) — genuine compute throughput, the number §2 could not isolate."
    );
    true
}

/// A hand-written scalar Rust baseline doing the **same** I2_S unpack-compute as the JIT kernel over a
/// packed weight buffer — the apples-to-apples comparison point for E1 §3. Decodes each 2-bit code,
/// forms the signed weight `code − 1`, and accumulates `weight·activation`.
fn scalar_packed_dot(packed: &[u8], activations: &[i32], n: usize) -> i64 {
    let mut acc: i64 = 0;
    for i in 0..n {
        let byte = packed[i >> 2];
        let code = i64::from((byte >> ((i & 3) * 2)) & 0b11);
        acc += (code - 1) * i64::from(activations[i]);
    }
    acc
}

/// A deterministic int activation vector of `n` elements (LCG, small signed range so the i64
/// accumulator never overflows).
fn activations(n: usize) -> Vec<i32> {
    let mut s = 0x9E37_79B9_7F4A_7C15_u64;
    (0..n)
        .map(|_| {
            s = s
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            {
                (((s >> 40) % 201) as i64 - 100) as i32
            }
        })
        .collect()
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
