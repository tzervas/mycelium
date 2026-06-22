//! In-process **JIT** execution (M-340; E3-4; ADR-009; ADR-014; phase-3.md §2 Batch L).
//!
//! Compiles the bit/trit subset to a shared object (`clang -shared`) and calls it **in-process** via
//! `dlopen`/`dlsym` — removing the process-spawn overhead of the AOT path (M-303). It reuses the
//! *same* lowering (`crate::llvm::lower_program`) and the *same* element encoding/decoding
//! (`emit_char_code`/`decode_result`) as the AOT path, so the JIT is a genuine fourth execution path
//! that must agree with the reference interpreter on the observable (`repr + payload + guarantee`,
//! NFR-7/RR-12) — checked in `tests/jit_differential.rs`.
//!
//! **Intentional unsafe (ADR-014).** This is the first `unsafe` in the workspace: the dynamic-linker
//! FFI (`dlopen`/`dlsym`/`dlclose`, resolved via libc — no new dependency) and the one fn-pointer
//! `transmute`. Each site carries a `// SAFETY:` justification and
//! `#[cfg_attr(not(debug_assertions), allow(unsafe_code))]` (warns in dev/test as the caution
//! incentive, silent in release).
//!
//! **Honesty / E1.** The kernel is *closed* (constants baked in), so `clang` constant-folds it — the
//! in-process per-call time measures call overhead, not kernel compute. A calibrated
//! compute-throughput verdict still needs kernels over *runtime data* (M-360, real packed-ternary
//! kernels). This module establishes the JIT *path* + NFR-7 equivalence, **not** the E1 throughput
//! number (VR-5 — not pre-written).

use std::ffi::{c_void, CString};
use std::fmt::Write as _;
use std::os::raw::{c_char, c_int};
use std::path::PathBuf;

use mycelium_core::{Node, Value};

use crate::llvm::{
    decode_result, emit_char_code, lower_program, path, run_tool, unique_tmp_dir, AotError,
    LaneKind, TmpDir,
};

extern "C" {
    fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> c_int;
}

const RTLD_NOW: c_int = 2;

/// Emit the JIT kernel as `i32 @myc_kernel(ptr %out)`: it writes each result element's ASCII char
/// into `out[i]` (one op per element — same transparent rendering as the AOT path) and **returns the
/// overflow status** (0 = ok, 1 = balanced-ternary overflow). The non-`void` return is the in-process
/// half of the read-back protocol: on overflow the kernel returns 1 *without* writing `out`, mirroring
/// the AOT sentinel line and the interpreter's `EvalError::Overflow`. Deterministic.
fn emit_kernel_fn(node: &Node) -> Result<(String, LaneKind, usize), AotError> {
    let lowered = lower_program(node)?;
    if !lowered.funcs.is_empty() {
        // Closures (Increment-2) emit top-level functions + a bump arena that this single-kernel JIT
        // emitter does not assemble — refuse explicitly rather than emit a kernel that calls
        // undefined symbols (G2). The AOT `compile_and_run` path supports closures.
        return Err(AotError::UnsupportedNode(
            "closures (Increment-2) are not supported on the JIT path; use the AOT \
             compile_and_run path"
                .to_owned(),
        ));
    }
    let kind = lowered.result.kind;
    let width = lowered.result.vals.len();
    let vals = lowered.result.vals;
    let overflow = lowered.overflow;
    let mut ssa = lowered.ssa;

    let mut ir = String::from("; mycelium direct-LLVM JIT kernel (M-340)\n");
    ir.push_str("define i32 @myc_kernel(ptr %out) {\nentry:\n");
    ir.push_str(&lowered.body);

    let emit_stores_and_ok = |ir: &mut String, ssa: &mut crate::llvm::Ssa| {
        for (i, v) in vals.iter().enumerate() {
            let c = emit_char_code(kind, v, ssa, ir);
            let t = ssa.fresh();
            let _ = writeln!(ir, "  {t} = trunc i32 {c} to i8");
            let p = ssa.fresh();
            let _ = writeln!(ir, "  {p} = getelementptr i8, ptr %out, i64 {i}");
            let _ = writeln!(ir, "  store i8 {t}, ptr {p}");
        }
        ir.push_str("  ret i32 0\n");
    };

    match overflow {
        None => emit_stores_and_ok(&mut ir, &mut ssa),
        // Branch on the runtime overflow flag: return 1 (no stores) on overflow, else write + 0.
        Some(ovf) => {
            let _ = writeln!(ir, "  br i1 {ovf}, label %ovf, label %ok");
            ir.push_str("ovf:\n  ret i32 1\nok:\n");
            emit_stores_and_ok(&mut ir, &mut ssa);
        }
    }
    ir.push_str("}\n");
    Ok((ir, kind, width))
}

/// A JIT-compiled kernel: the `.so` on disk (in a per-artifact temp dir, cleaned on drop) + the
/// result shape. Produced by [`compile_so`]; call any number of times in-process with
/// [`JitArtifact::call`].
pub struct JitArtifact {
    _dir: TmpDir,
    so: PathBuf,
    kind: LaneKind,
    width: usize,
}

impl JitArtifact {
    /// Call the kernel in-process (`dlopen` → `dlsym` → call) and read the result back as an `Exact`
    /// `Value`. Returns an explicit [`AotError`] on any FFI failure — never a silent/garbage result.
    pub fn call(&self) -> Result<Value, AotError> {
        let lib = dlopen_path(&self.so)?; // dlclose on drop
        let fptr = lib.sym("myc_kernel")?;
        // `sym` returns an explicit error on a null result, so `fptr` is non-null here; the
        // debug-assert makes that precondition machine-checked in dev/test builds (DN-21 §6 M-679).
        debug_assert!(
            !fptr.is_null(),
            "lib.sym must return a non-null address or Err"
        );

        let mut buf = vec![0u8; self.width];
        // SAFETY: `fptr` is the address `dlsym` returned for the `i32 myc_kernel(ptr)` we just
        // emitted and compiled, so the `extern "C" fn(*mut u8) -> i32` type matches; `buf` is exactly
        // `self.width` bytes and the kernel writes one byte per result element (`self.width` total)
        // only on the ok path, so the write is in-bounds. The library stays loaded for the call
        // (`lib`).
        #[cfg_attr(not(debug_assertions), allow(unsafe_code))]
        let status = unsafe {
            let kernel: extern "C" fn(*mut u8) -> i32 = std::mem::transmute(fptr);
            kernel(buf.as_mut_ptr())
        };
        // Read-back protocol: a non-zero status means the in-process kernel overflowed the m-trit
        // range — an explicit error, never a silently-wrapped (and unwritten) buffer.
        if status != 0 {
            return Err(AotError::Overflow(format!(
                "fixed-width result out of {}-trit range",
                self.width
            )));
        }
        decode_result(self.kind, self.width, buf.iter().map(|&b| b as char))
    }
}

/// A loaded shared library that `dlclose`s itself on drop. `pub(crate)` so other in-crate JIT kernels
/// (e.g. the M-360 BitNet dot kernel) reuse the same dynamic-loader rather than re-rolling the FFI.
pub(crate) struct Lib(*mut c_void);
impl Lib {
    /// Resolve `symbol` in this library to a raw function/data address (an explicit error if absent).
    pub(crate) fn sym(&self, symbol: &str) -> Result<*mut c_void, AotError> {
        let name = CString::new(symbol)
            .map_err(|e| AotError::Run(format!("symbol name has interior NUL: {e}")))?;
        lookup_sym(self.0, &name)
    }
}
impl Drop for Lib {
    fn drop(&mut self) {
        // SAFETY: `self.0` is a handle returned by `dlopen` and not closed elsewhere; closing it
        // once on drop is the matching `dlclose`. **Dangling-pointer obligation (DN-21 §4):** any
        // address `dlsym` derived from this handle (a JIT'd fn-pointer) is unloaded by this
        // `dlclose`, so no derived pointer may be called after the owning `Lib` is dropped. Today
        // every call site keeps the `Lib` live for the whole call (the fn-ptr is resolved and
        // invoked while a `&Lib` / co-located `_lib` field is in scope), so no derived pointer
        // outlives the handle — M-682 lifts that co-location convention into a compiler-checked
        // lifetime.
        #[cfg_attr(not(debug_assertions), allow(unsafe_code))]
        unsafe {
            dlclose(self.0);
        }
    }
}

/// `dlopen` a shared object by path, returning a [`Lib`] that closes it on drop. The reusable loader
/// shared by [`JitArtifact`] and the M-360 BitNet kernel.
pub(crate) fn dlopen_path(so: &std::path::Path) -> Result<Lib, AotError> {
    let cpath = CString::new(path(so)?)
        .map_err(|e| AotError::Run(format!("so path has interior NUL: {e}")))?;
    Ok(Lib(open_lib(&cpath)?))
}

fn open_lib(so: &CString) -> Result<*mut c_void, AotError> {
    // SAFETY: `so` is a valid NUL-terminated path (a `CString`) to the `.so` just written;
    // `RTLD_NOW` resolves symbols eagerly so a bad library fails here rather than at call time.
    // **Platform assumption (DN-21 §3):** `RTLD_NOW = 2` is hard-coded for the glibc/musl Linux ABI
    // (no `libc` constant is pulled in — an intentional ADR-014 zero-dependency choice); the JIT is
    // Linux-only today. **Global-constructor safety:** the JIT IR emits no `@llvm.global_ctors`
    // (`Empirical` — verified by reading `emit_kernel_fn`/`emit_*_ir`), so `dlopen` runs no foreign
    // constructor code on load.
    #[cfg_attr(not(debug_assertions), allow(unsafe_code))]
    let handle = unsafe { dlopen(so.as_ptr(), RTLD_NOW) };
    if handle.is_null() {
        Err(AotError::Run(
            "dlopen failed for the JIT shared object".to_owned(),
        ))
    } else {
        Ok(handle)
    }
}

fn lookup_sym(handle: *mut c_void, sym: &CString) -> Result<*mut c_void, AotError> {
    // The handle reached here only via `open_lib`, which returns `Err` on a null `dlopen` result,
    // so `handle` is non-null by construction; assert it in dev/test (DN-21 §6 M-679).
    debug_assert!(
        !handle.is_null(),
        "lookup_sym requires a live (non-null) dlopen handle"
    );
    // SAFETY: `handle` is a live `dlopen` handle — it originates from `open_lib`, which returns the
    // handle only after checking it is non-null, and the only callers (`Lib::sym`) hold it in a live
    // `Lib`, so the library is still loaded at this point; `sym` is a valid NUL-terminated C string.
    #[cfg_attr(not(debug_assertions), allow(unsafe_code))]
    let ptr = unsafe { dlsym(handle, sym.as_ptr()) };
    if ptr.is_null() {
        Err(AotError::Run(
            "dlsym could not find `myc_kernel`".to_owned(),
        ))
    } else {
        Ok(ptr)
    }
}

/// Compile the bit/trit-subset program to a shared object without calling it. Returns
/// [`AotError::ToolchainMissing`] when `clang` is absent so callers can skip; out-of-subset
/// constructs are the same explicit refusals as [`crate::emit_llvm_ir`].
pub fn compile_so(node: &Node) -> Result<JitArtifact, AotError> {
    let (ir, kind, width) = emit_kernel_fn(node)?;
    let dir = unique_tmp_dir()?;
    let ll = dir.join("jit.ll");
    let so = dir.join("jit.so");
    let guard = TmpDir(dir);

    std::fs::write(&ll, ir.as_bytes()).map_err(|e| AotError::Run(format!("write IR: {e}")))?;
    run_tool(
        "clang",
        &["-shared", "-fPIC", "-x", "ir", path(&ll)?, "-o", path(&so)?],
    )?;

    Ok(JitArtifact {
        _dir: guard,
        so,
        kind,
        width,
    })
}

/// Compile the program to a shared object and call it once, in-process. The convenience wrapper over
/// [`compile_so`] + [`JitArtifact::call`]; the JIT execution path checked against the interpreter
/// (NFR-7).
pub fn jit_run(node: &Node) -> Result<Value, AotError> {
    compile_so(node)?.call()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mycelium_core::{Meta, Payload, Provenance, Repr, Trit};

    fn binary(bits: Vec<bool>) -> Value {
        let width = bits.len() as u32;
        Value::new(
            Repr::Binary { width },
            Payload::Bits(bits),
            Meta::exact(Provenance::Root),
        )
        .unwrap()
    }

    #[test]
    fn jit_kernel_emits_a_function_writing_to_out() {
        let prog = Node::Op {
            prim: "bit.not".into(),
            args: vec![Node::Const(binary(vec![true, false]))],
        };
        let (ir, _, width) = emit_kernel_fn(&prog).unwrap();
        assert!(ir.contains("define i32 @myc_kernel(ptr %out)"));
        assert!(ir.contains("store i8")); // writes results into the out buffer
        assert!(ir.contains("ret i32 0")); // ok status (no overflow path for a bit op)
        assert_eq!(width, 2);
    }

    #[test]
    fn jit_bit_not_matches_interpreter() {
        // Mutant-witness: a wrong store offset / fn signature would read back a different payload.
        let prog = Node::Op {
            prim: "bit.not".into(),
            args: vec![Node::Const(binary(vec![true, false, true, true]))],
        };
        match jit_run(&prog) {
            Ok(v) => {
                assert_eq!(v.payload(), &Payload::Bits(vec![false, true, false, false]));
                assert_eq!(v.repr(), &Repr::Binary { width: 4 });
            }
            Err(AotError::ToolchainMissing(_)) => { /* environment skip */ }
            Err(e) => panic!("unexpected JIT error: {e}"),
        }
    }

    #[test]
    fn jit_trit_neg_matches_interpreter() {
        let prog = Node::Op {
            prim: "trit.neg".into(),
            args: vec![Node::Const(
                Value::new(
                    Repr::Ternary { trits: 3 },
                    Payload::Trits(vec![Trit::Pos, Trit::Zero, Trit::Neg]),
                    Meta::exact(Provenance::Root),
                )
                .unwrap(),
            )],
        };
        match jit_run(&prog) {
            Ok(v) => assert_eq!(
                v.payload(),
                &Payload::Trits(vec![Trit::Neg, Trit::Zero, Trit::Pos])
            ),
            Err(AotError::ToolchainMissing(_)) => {}
            Err(e) => panic!("unexpected JIT error: {e}"),
        }
    }

    fn tern(trits: Vec<Trit>) -> Value {
        let m = trits.len() as u32;
        Value::new(
            Repr::Ternary { trits: m },
            Payload::Trits(trits),
            Meta::exact(Provenance::Root),
        )
        .unwrap()
    }

    #[test]
    fn jit_trit_add_matches_oracle() {
        // 5 + 4 = 9 in 3 trits: [+,-,-] + [0,+,+] = [+,0,0] — the in-process ripple-carry path.
        let prog = Node::Op {
            prim: "trit.add".into(),
            args: vec![
                Node::Const(tern(vec![Trit::Pos, Trit::Neg, Trit::Neg])),
                Node::Const(tern(vec![Trit::Zero, Trit::Pos, Trit::Pos])),
            ],
        };
        match jit_run(&prog) {
            Ok(v) => assert_eq!(
                v.payload(),
                &Payload::Trits(vec![Trit::Pos, Trit::Zero, Trit::Zero])
            ),
            Err(AotError::ToolchainMissing(_)) => {}
            Err(e) => panic!("unexpected JIT error: {e}"),
        }
    }

    #[test]
    fn jit_trit_overflow_is_explicit() {
        // 4 + 4 = 8 in 2 trits overflows: the kernel returns the non-zero status, surfaced as an
        // explicit Overflow — never a silently-wrapped (unwritten) buffer. Mutant-witness: a `void`
        // kernel (no status) could not signal this in-process.
        let prog = Node::Op {
            prim: "trit.add".into(),
            args: vec![
                Node::Const(tern(vec![Trit::Pos, Trit::Pos])),
                Node::Const(tern(vec![Trit::Pos, Trit::Pos])),
            ],
        };
        match jit_run(&prog) {
            Ok(v) => panic!("overflow must not produce a value, got {:?}", v.payload()),
            Err(AotError::Overflow(_)) => { /* expected */ }
            Err(AotError::ToolchainMissing(_)) => {}
            Err(e) => panic!("unexpected JIT error: {e}"),
        }
    }
}
