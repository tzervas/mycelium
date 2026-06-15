//! **BitNet-class packed-ternary acceleration** — a runtime-data compute kernel (M-360; E3-6;
//! FR-C3 / NFR-4 / G3; RFC-0004 §5/§8; ADR-009/ADR-014; phase-3.md §2 / E1).
//!
//! The canonical BitNet primitive is the **ternary multiply-accumulate**: a dot product `y = Σ wᵢ·xᵢ`
//! where the weights `wᵢ` are balanced ternary `{−1,0,+1}` and the activations `xᵢ` are integers.
//! Because the weights are ternary the per-element multiply collapses to add / subtract / skip — the
//! "pack-store-load-**unpack-compute**" loop RFC-0004 §5 reuses from bitnet.cpp. This module emits
//! that loop as **textual LLVM IR** (inspectable, no opaque pass — FR-C3 "metadata, not hidden
//! lowering"; RFC-0004 §6), JIT-compiles it (`clang -shared`), and calls it **in-process** (the
//! M-340 dynamic loader) over buffers passed as **runtime pointers**.
//!
//! **Why this closes the open E1 question.** The earlier native/JIT kernels (M-301/M-303/M-340) bake
//! their inputs in as constants, so `clang` constant-folds the whole computation — the measured time
//! is call/spawn overhead, not compute (honestly captioned, never claimed as throughput; VR-5). Here
//! the weight and activation buffers are *function arguments*: the optimiser cannot fold them, so the
//! per-call time is **genuine packed-ternary compute** over `n` elements. That is the runtime-input
//! kernel E1 (`cargo xtask e1`) needs to finally report a compute-throughput number.
//!
//! **Scope / honesty (first increment).** One scheme — **I2_S** (the RFC-0004 §5 default: 2-bit,
//! 4 trits/byte, lossless multiply-add) — and a scalar (non-SIMD) loop. It is differential-checked
//! against [`ternary_dot_ref`] (the obvious Rust oracle) so the IR is verified, not asserted. What is
//! **not** claimed: parity with bitnet.cpp's hand-tuned SIMD kernels, or the other packings
//! (TL1/TL2). Those are the next M-360 increments; the E1 verdict reports the measured number and
//! states the comparison baseline explicitly (no pre-written perf claim, VR-5/G3).

use std::ffi::c_void;
use std::fmt::Write as _;

use mycelium_core::ternary::digit;
use mycelium_core::{PackScheme, Trit};

use crate::jit::{dlopen_path, Lib};
use crate::llvm::{path, run_tool, unique_tmp_dir, AotError, TmpDir};
use crate::pack::pack_trits;

/// The packing this kernel decodes inline. **I2_S** is the RFC-0004 §5 default (2-bit, 4 trits/byte,
/// `rot = 0` so a code `c ∈ {0,1,2}` is the base-3 digit and the signed weight is `c − 1`).
pub const KERNEL_SCHEME: PackScheme = PackScheme::I2S;

/// The reference (oracle) ternary dot product `Σ digit(wᵢ)·xᵢ` over `i64`, the exact semantics the
/// JIT kernel must reproduce. `digit` is the balanced-ternary signed value (`mycelium_core::ternary`).
/// Operates on the **unpacked** trits + activations; the kernel decodes the I2_S packing of the same
/// weights at runtime, so a match proves the in-IR unpack is correct.
#[must_use]
pub fn ternary_dot_ref(weights: &[Trit], activations: &[i32]) -> i64 {
    weights
        .iter()
        .zip(activations)
        .map(|(&w, &x)| digit(w) * i64::from(x))
        .sum()
}

/// Emit the textual LLVM IR for the I2_S packed-ternary dot kernel
/// `i64 @myc_bitnet_dot(ptr %w, ptr %x, i64 %n)`: it loops `i ∈ [0, n)`, loads the packed weight byte
/// `w[i>>2]`, extracts the 2-bit code at lane `i&3`, forms the signed weight `code − 1`, loads the
/// activation `x[i]`, and accumulates `weight·x` into an `i64`. Deterministic; one transparent op per
/// step of the loop body (no opaque pass — RFC-0004 §6).
#[must_use]
pub fn emit_bitnet_dot_ir() -> String {
    // A fixed kernel (no per-program lowering), written out directly so every load/shift/mul is
    // visible. SSA names are stable, so the emission is byte-for-byte deterministic.
    let mut ir = String::from("; mycelium BitNet packed-ternary dot kernel (I2_S; M-360)\n");
    ir.push_str("define i64 @myc_bitnet_dot(ptr %w, ptr %x, i64 %n) {\n");
    ir.push_str("entry:\n  br label %loop\n");
    // loop header: carry the index and the running accumulator as phis.
    ir.push_str("loop:\n");
    ir.push_str("  %i = phi i64 [ 0, %entry ], [ %inext, %body ]\n");
    ir.push_str("  %acc = phi i64 [ 0, %entry ], [ %accnext, %body ]\n");
    ir.push_str("  %done = icmp sge i64 %i, %n\n");
    ir.push_str("  br i1 %done, label %exit, label %body\n");
    // body: unpack one I2_S trit and multiply-accumulate.
    ir.push_str("body:\n");
    let _ = write!(
        ir,
        concat!(
            "  %bi = lshr i64 %i, 2\n",                    // byte index = i / 4
            "  %wp = getelementptr i8, ptr %w, i64 %bi\n", // &w[bi]
            "  %byte = load i8, ptr %wp\n",
            "  %byte32 = zext i8 %byte to i32\n",
            "  %lane = and i64 %i, 3\n", // lane = i % 4
            "  %lane32 = trunc i64 %lane to i32\n",
            "  %sh = shl i32 %lane32, 1\n", // shift = lane * 2
            "  %shifted = lshr i32 %byte32, %sh\n",
            "  %code = and i32 %shifted, 3\n", // 2-bit code ∈ {0,1,2}
            "  %digit = sub i32 %code, 1\n",   // signed weight ∈ {-1,0,1} (I2_S rot=0)
            "  %digit64 = sext i32 %digit to i64\n",
            "  %xp = getelementptr i32, ptr %x, i64 %i\n", // &x[i]
            "  %xi = load i32, ptr %xp\n",
            "  %xi64 = sext i32 %xi to i64\n",
            "  %prod = mul i64 %digit64, %xi64\n",
            "  %accnext = add i64 %acc, %prod\n",
            "  %inext = add i64 %i, 1\n",
            "  br label %loop\n",
        )
    );
    // exit: %acc is the loop phi, which dominates here — return it.
    ir.push_str("exit:\n  ret i64 %acc\n}\n");
    ir
}

/// A compiled, in-process BitNet dot kernel: the `.so` (in a per-artifact temp dir, cleaned on drop),
/// the dynamic-library handle (kept loaded for the kernel's lifetime), and the resolved entry point.
/// **Compile once, call many** — the natural shape for the E1 throughput measurement.
pub struct BitnetDotKernel {
    _dir: TmpDir,
    _lib: Lib,
    fptr: *mut c_void,
}

impl BitnetDotKernel {
    /// Run the kernel over `packed_weights` (I2_S-packed) and `activations`, summing the first `n`
    /// ternary products. The lengths are checked against `n` (≥ `n.div_ceil(4)` weight bytes, ≥ `n`
    /// activations) so the native loads are always in bounds — a short buffer is an explicit
    /// [`AotError`], never an out-of-bounds read.
    pub fn call(
        &self,
        packed_weights: &[u8],
        activations: &[i32],
        n: usize,
    ) -> Result<i64, AotError> {
        let need_bytes = n.div_ceil(4); // I2_S packs 4 trits/byte
        if packed_weights.len() < need_bytes {
            return Err(AotError::Run(format!(
                "packed weights too short: need {need_bytes} bytes for {n} trits, got {}",
                packed_weights.len()
            )));
        }
        if activations.len() < n {
            return Err(AotError::Run(format!(
                "activations too short: need {n}, got {}",
                activations.len()
            )));
        }
        let n_i64 = i64::try_from(n).map_err(|_| AotError::Run(format!("n too large: {n}")))?;
        // SAFETY: `fptr` is the address `dlsym` returned for the `i64 myc_bitnet_dot(ptr,ptr,i64)` we
        // emitted and compiled, so the `extern "C"` type matches. The bounds checks above guarantee
        // the kernel reads only `w[0..ceil(n/4)]` and `x[0..n]`, both in-bounds for the slices. The
        // library stays loaded for the call (`_lib`).
        #[cfg_attr(not(debug_assertions), allow(unsafe_code))]
        let sum = unsafe {
            let kernel: extern "C" fn(*const u8, *const i32, i64) -> i64 =
                std::mem::transmute(self.fptr);
            kernel(packed_weights.as_ptr(), activations.as_ptr(), n_i64)
        };
        Ok(sum)
    }
}

/// Compile the BitNet dot kernel to a shared object and load it in-process. Returns
/// [`AotError::ToolchainMissing`] when `clang` is absent so callers can skip (the house idiom).
pub fn compile_bitnet_dot() -> Result<BitnetDotKernel, AotError> {
    let ir = emit_bitnet_dot_ir();
    let dir = unique_tmp_dir()?;
    let ll = dir.join("bitnet.ll");
    let so = dir.join("bitnet.so");
    let guard = TmpDir(dir);

    std::fs::write(&ll, ir.as_bytes()).map_err(|e| AotError::Run(format!("write IR: {e}")))?;
    // -O2 so the optimiser does real codegen over the runtime-pointer loop (the point of E1).
    run_tool(
        "clang",
        &[
            "-shared",
            "-fPIC",
            "-O2",
            "-x",
            "ir",
            path(&ll)?,
            "-o",
            path(&so)?,
        ],
    )?;

    let lib = dlopen_path(&so)?;
    let fptr = lib.sym("myc_bitnet_dot")?;
    Ok(BitnetDotKernel {
        _dir: guard,
        _lib: lib,
        fptr,
    })
}

/// Convenience: pack `weights` under [`KERNEL_SCHEME`], compile the kernel, and run the dot product
/// against `activations` once. The wrapper the differential test checks against [`ternary_dot_ref`].
pub fn jit_ternary_dot(weights: &[Trit], activations: &[i32]) -> Result<i64, AotError> {
    let packed = pack_trits(weights, KERNEL_SCHEME);
    compile_bitnet_dot()?.call(&packed, activations, weights.len().min(activations.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic ternary/activation test data (small LCGs) — fixed, not a statistical sample.
    fn weights(n: usize) -> Vec<Trit> {
        let mut s = 0x1234_5678_u64;
        (0..n)
            .map(|_| {
                s = s.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                match (s >> 33) % 3 {
                    0 => Trit::Neg,
                    1 => Trit::Zero,
                    _ => Trit::Pos,
                }
            })
            .collect()
    }
    fn activations(n: usize) -> Vec<i32> {
        let mut s = 0x9E37_79B9_u64;
        (0..n)
            .map(|_| {
                s = s.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                // small signed activations so the i64 accumulator never overflows in tests
                (((s >> 40) % 201) as i64 - 100) as i32
            })
            .collect()
    }

    #[test]
    fn ir_is_inspectable_and_deterministic() {
        let ir = emit_bitnet_dot_ir();
        // Inspectable: the unpack-compute loop is visible (no opaque pass — FR-C3 / RFC-0004 §6).
        assert!(ir.contains("define i64 @myc_bitnet_dot(ptr %w, ptr %x, i64 %n)"));
        assert!(ir.contains("load i8")); // loads packed weight bytes from the runtime pointer
        assert!(ir.contains("and i32 %shifted, 3")); // extracts the 2-bit I2_S code
        assert!(ir.contains("sub i32 %code, 1")); // forms the signed ternary weight
        assert!(ir.contains("mul i64") && ir.contains("add i64")); // multiply-accumulate
        assert_eq!(emit_bitnet_dot_ir(), emit_bitnet_dot_ir());
    }

    #[test]
    fn ref_matches_hand_computed() {
        // [-1, 0, +1] · [7, 9, 4] = -7 + 0 + 4 = -3. Pins the oracle itself.
        let w = vec![Trit::Neg, Trit::Zero, Trit::Pos];
        let x = vec![7, 9, 4];
        assert_eq!(ternary_dot_ref(&w, &x), -3);
    }

    #[test]
    fn jit_dot_matches_reference() {
        // Mutant-witness: a wrong shift/mask (e.g. extracting the wrong lane) or a `code` instead of
        // `code-1` weight would diverge from the oracle on this mixed data.
        for n in [1usize, 4, 5, 7, 64, 256, 1000] {
            let w = weights(n);
            let x = activations(n);
            match jit_ternary_dot(&w, &x) {
                Ok(got) => assert_eq!(got, ternary_dot_ref(&w, &x), "dot mismatch at n={n}"),
                Err(AotError::ToolchainMissing(_)) => return, // environment skip
                Err(e) => panic!("unexpected BitNet JIT error at n={n}: {e}"),
            }
        }
    }

    #[test]
    fn compile_once_call_many_is_consistent() {
        // The compile-once/run-many shape (used by the E1 harness): the same kernel instance over
        // different buffers must each match the oracle.
        let kernel = match compile_bitnet_dot() {
            Ok(k) => k,
            Err(AotError::ToolchainMissing(_)) => return,
            Err(e) => panic!("compile failed: {e}"),
        };
        for n in [16usize, 100, 333] {
            let w = weights(n);
            let x = activations(n);
            let packed = pack_trits(&w, KERNEL_SCHEME);
            assert_eq!(
                kernel.call(&packed, &x, n).unwrap(),
                ternary_dot_ref(&w, &x),
                "compiled kernel diverged at n={n}"
            );
        }
    }

    #[test]
    fn short_buffers_are_explicit_errors() {
        // Mutant-witness: dropping the bounds checks would let the kernel read out of bounds; a short
        // buffer must be an explicit refusal, never an OOB load.
        let kernel = match compile_bitnet_dot() {
            Ok(k) => k,
            Err(AotError::ToolchainMissing(_)) => return,
            Err(e) => panic!("compile failed: {e}"),
        };
        let packed = pack_trits(&weights(8), KERNEL_SCHEME); // 2 bytes
        assert!(matches!(
            kernel.call(&packed, &[1, 2, 3], 8),
            Err(AotError::Run(_))
        )); // too few acts
        assert!(matches!(
            kernel.call(&[0u8], &activations(8), 8),
            Err(AotError::Run(_))
        )); // too few bytes
    }
}
