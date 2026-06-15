//! **Hand-vectorized (SIMD) packed-ternary dot kernel** (M-360; E3-6; FR-C3 / NFR-4 / G3;
//! RFC-0004 §5/§8; ADR-009/ADR-014; phase-3.md §2 / E1).
//!
//! The [`crate::bitnet`] scalar kernels decode one trit per loop step. This module emits a
//! **hand-vectorized** I2_S dot kernel — `i64 @myc_bitnet_dot_simd(ptr %w, ptr %x, i64 %n)` — that
//! unpacks and multiply-accumulates **8 trits per iteration** using LLVM vector types
//! (`<8 x i32>`/`<8 x i64>`), with a scalar epilogue for the `n mod 8` tail. The vector **unpack** is
//! the correctness-critical part — a wrong shuffle mask or shift vector silently misreads weights —
//! so it is written transparently (every vector op visible in the textual IR, no opaque pass —
//! RFC-0004 §6 / FR-C3) and **differential-checked against the scalar kernel as the oracle**
//! (`tests/simd_differential.rs`, and `jit_simd_matches_scalar` here) over a corpus that brackets the
//! vector width and the tail (n ∈ {0,1,7,8,9,15,16,17,…}).
//!
//! **The vectorized I2_S unpack.** For 8 consecutive trits (= 2 packed bytes, 4 trits/byte):
//! broadcast `[byte0,byte1]` to the 8 lanes (`shufflevector` mask `<0,0,0,0,1,1,1,1>`), shift each
//! lane to bring its 2-bit code to bit 0 (`lshr` by the constant vector `<0,2,4,6,0,2,4,6>`), mask
//! `& 3` to the code, `− 1` to the signed weight, multiply by the contiguously-loaded `<8 x i32>`
//! activations, widen, and accumulate into an `<8 x i64>` phi; the loop tail horizontally reduces it
//! (`@llvm.vector.reduce.add.v8i64`) and a scalar loop finishes the remainder.
//!
//! **Scope / honesty (VR-5/G3).** Same exact dot product as the scalar kernel — no guarantee
//! upgraded; the reduction is exact i64 integer arithmetic. The speedup over the scalar JIT kernel is
//! whatever `cargo xtask e1` §5 measures over runtime data; no target is pre-written. **I2_S only**
//! for this increment (the default scheme); the TL1/TL2 vectorized unpacks and the *true 1.67-b/w
//! TL2 layout* (A5-08) are the next M-360 steps. The scalar kernels stay the oracle.

use mycelium_core::PackScheme;

use crate::bitnet::BitnetDotKernel;
use crate::jit::dlopen_path;
use crate::llvm::{path, run_tool, unique_tmp_dir, AotError, TmpDir};

/// The SIMD kernel's symbol — distinct from the scalar [`crate::bitnet`] kernel so both can be loaded
/// at once (e.g. for the E1 §5 differential timing).
const SIMD_SYM: &str = "myc_bitnet_dot_simd";

/// Emit the textual LLVM IR for the **hand-vectorized I2_S** packed-ternary dot kernel
/// `i64 @myc_bitnet_dot_simd(ptr %w, ptr %x, i64 %n)`: an 8-wide vector body (8 trits/iteration) plus
/// a scalar tail for `n mod 8`. Deterministic; every vector op is visible (no opaque pass). The
/// vector loads carry explicit `align 1`/`align 4` so an arbitrary (sub-vector-aligned) buffer offset
/// is a legal unaligned load, never UB.
#[must_use]
pub fn emit_bitnet_dot_simd_ir() -> String {
    // Written out directly (no per-program lowering) so every shuffle/shift/mask is inspectable and
    // the emission is byte-for-byte deterministic.
    String::from(concat!(
        "; mycelium SIMD (8-wide) BitNet I2_S packed-ternary dot kernel (M-360)\n",
        "declare i64 @llvm.vector.reduce.add.v8i64(<8 x i64>)\n",
        "define i64 @myc_bitnet_dot_simd(ptr %w, ptr %x, i64 %n) {\n",
        "entry:\n",
        "  %vn = and i64 %n, -8\n", // full 8-lane iterations cover [0, vn)
        "  br label %vloop\n",
        // vector loop: carry the index and the <8 x i64> accumulator.
        "vloop:\n",
        "  %i = phi i64 [ 0, %entry ], [ %inext, %vbody ]\n",
        "  %vacc = phi <8 x i64> [ zeroinitializer, %entry ], [ %vaccnext, %vbody ]\n",
        "  %vdone = icmp sge i64 %i, %vn\n",
        "  br i1 %vdone, label %tail, label %vbody\n",
        "vbody:\n",
        "  %bi = lshr i64 %i, 2\n", // first weight byte = i/4 (4 trits/byte)
        "  %wp = getelementptr i8, ptr %w, i64 %bi\n",
        "  %b2 = load <2 x i8>, ptr %wp, align 1\n", // [byte0, byte1] — 8 trits
        "  %b2_32 = zext <2 x i8> %b2 to <2 x i32>\n",
        // broadcast byte0 to lanes 0-3, byte1 to lanes 4-7
        "  %bc = shufflevector <2 x i32> %b2_32, <2 x i32> poison, <8 x i32> <i32 0, i32 0, i32 0, i32 0, i32 1, i32 1, i32 1, i32 1>\n",
        // shift each lane so its 2-bit code lands in bits[0:1]
        "  %sh = lshr <8 x i32> %bc, <i32 0, i32 2, i32 4, i32 6, i32 0, i32 2, i32 4, i32 6>\n",
        "  %code = and <8 x i32> %sh, <i32 3, i32 3, i32 3, i32 3, i32 3, i32 3, i32 3, i32 3>\n",
        "  %wt = sub <8 x i32> %code, <i32 1, i32 1, i32 1, i32 1, i32 1, i32 1, i32 1, i32 1>\n", // signed weight = code − 1
        "  %xp = getelementptr i32, ptr %x, i64 %i\n",
        "  %xv = load <8 x i32>, ptr %xp, align 4\n", // 8 contiguous activations
        "  %prod = mul <8 x i32> %wt, %xv\n",
        "  %prod64 = sext <8 x i32> %prod to <8 x i64>\n",
        "  %vaccnext = add <8 x i64> %vacc, %prod64\n",
        "  %inext = add i64 %i, 8\n",
        "  br label %vloop\n",
        // horizontally reduce the vector accumulator, then finish the tail scalar-wise.
        "tail:\n",
        "  %hsum = call i64 @llvm.vector.reduce.add.v8i64(<8 x i64> %vacc)\n",
        "  br label %sloop\n",
        "sloop:\n",
        "  %j = phi i64 [ %vn, %tail ], [ %jnext, %sbody ]\n",
        "  %sacc = phi i64 [ %hsum, %tail ], [ %saccnext, %sbody ]\n",
        "  %sdone = icmp sge i64 %j, %n\n",
        "  br i1 %sdone, label %exit, label %sbody\n",
        "sbody:\n",
        "  %sbi = lshr i64 %j, 2\n",
        "  %swp = getelementptr i8, ptr %w, i64 %sbi\n",
        "  %sbyte = load i8, ptr %swp\n",
        "  %sbyte32 = zext i8 %sbyte to i32\n",
        "  %slane = and i64 %j, 3\n",
        "  %slane32 = trunc i64 %slane to i32\n",
        "  %ssh = shl i32 %slane32, 1\n",
        "  %sshifted = lshr i32 %sbyte32, %ssh\n",
        "  %scode = and i32 %sshifted, 3\n",
        "  %sdigit = sub i32 %scode, 1\n",
        "  %sdigit64 = sext i32 %sdigit to i64\n",
        "  %sxp = getelementptr i32, ptr %x, i64 %j\n",
        "  %sxi = load i32, ptr %sxp\n",
        "  %sxi64 = sext i32 %sxi to i64\n",
        "  %sprod = mul i64 %sdigit64, %sxi64\n",
        "  %saccnext = add i64 %sacc, %sprod\n",
        "  %jnext = add i64 %j, 1\n",
        "  br label %sloop\n",
        "exit:\n",
        "  ret i64 %sacc\n",
        "}\n",
    ))
}

/// Compile the hand-vectorized I2_S BitNet dot kernel to a shared object and load it in-process,
/// returning a [`BitnetDotKernel`] (I2_S scheme) so the SIMD path reuses the scalar kernel's
/// bounds-checked `call`. Returns [`AotError::ToolchainMissing`] when `clang` is absent (the house
/// skip idiom). Same C signature + I2_S bounds model as the scalar kernel — only the body differs.
pub fn compile_bitnet_dot_simd() -> Result<BitnetDotKernel, AotError> {
    let ir = emit_bitnet_dot_simd_ir();
    let dir = unique_tmp_dir()?;
    let ll = dir.join("bitnet_simd.ll");
    let so = dir.join("bitnet_simd.so");
    let guard = TmpDir(dir);

    std::fs::write(&ll, ir.as_bytes()).map_err(|e| AotError::Run(format!("write IR: {e}")))?;
    // -O2 so the backend lowers the vector IR to real SIMD instructions (the point of E1 §5).
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
    let fptr = lib.sym(SIMD_SYM)?;
    Ok(BitnetDotKernel::from_loaded(
        guard,
        lib,
        fptr,
        PackScheme::I2S,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitnet::ternary_dot_ref;
    use crate::pack::pack_trits;
    use mycelium_core::Trit;

    fn weights(n: usize) -> Vec<Trit> {
        let mut s = 0x7777_3333_u64;
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
        let mut s = 0xC0FF_EE42_u64;
        (0..n)
            .map(|_| {
                s = s.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                (((s >> 40) % 201) as i64 - 100) as i32
            })
            .collect()
    }

    #[test]
    fn ir_is_vectorized_inspectable_and_deterministic() {
        let ir = emit_bitnet_dot_simd_ir();
        assert!(ir.contains("define i64 @myc_bitnet_dot_simd(ptr %w, ptr %x, i64 %n)"));
        // The vector unpack is visible (no opaque pass — FR-C3 / RFC-0004 §6).
        assert!(ir.contains("shufflevector")); // the byte broadcast
        assert!(ir.contains("lshr <8 x i32>")); // the per-lane code shift
        assert!(ir.contains("mul <8 x i32>")); // the vector multiply
        assert!(ir.contains("@llvm.vector.reduce.add.v8i64")); // horizontal reduction
        assert_eq!(emit_bitnet_dot_simd_ir(), emit_bitnet_dot_simd_ir()); // deterministic
    }

    #[test]
    fn jit_simd_matches_scalar_oracle_across_the_width_boundary() {
        // Mutant-witness: a wrong shuffle mask / shift vector, or a missing scalar tail, diverges from
        // the oracle precisely at the n values that straddle the 8-lane width and the tail.
        let kernel = match compile_bitnet_dot_simd() {
            Ok(k) => k,
            Err(AotError::ToolchainMissing(_)) => return, // environment skip
            Err(e) => panic!("SIMD compile failed: {e}"),
        };
        for n in [0usize, 1, 2, 7, 8, 9, 15, 16, 17, 31, 64, 100, 257, 1000] {
            let w = weights(n);
            let x = activations(n);
            let packed = pack_trits(&w, PackScheme::I2S);
            // n=0 needs a non-empty buffer only conceptually; pack of [] is [], call with n=0 is 0.
            let got = kernel.call(&packed, &x, n).expect("SIMD kernel runs");
            assert_eq!(got, ternary_dot_ref(&w, &x), "SIMD dot mismatch at n={n}");
        }
    }

    #[test]
    fn simd_short_buffers_are_explicit_errors() {
        // The reused bounds check (I2_S: n.div_ceil(4) bytes, n activations) still refuses a short
        // buffer — the vector loads must never read past the buffer.
        let kernel = match compile_bitnet_dot_simd() {
            Ok(k) => k,
            Err(AotError::ToolchainMissing(_)) => return,
            Err(e) => panic!("SIMD compile failed: {e}"),
        };
        let packed = pack_trits(&weights(16), PackScheme::I2S); // 4 bytes
        assert!(matches!(
            kernel.call(&packed, &[1, 2, 3], 16),
            Err(AotError::Run(_))
        ));
        assert!(matches!(
            kernel.call(&[0u8], &activations(16), 16),
            Err(AotError::Run(_))
        ));
    }
}
