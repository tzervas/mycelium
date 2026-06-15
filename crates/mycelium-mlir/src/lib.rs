//! `mycelium-mlir` — the AOT path: a textual **ternary-dialect skeleton** (M-150) plus a
//! **direct-LLVM-IR backend** that genuinely compiles the bit subset to native code (M-301;
//! RFC-0004 §2/§6; ADR-007/009; T1.5; phase-3.md §1).
//!
//! **Scope / honesty.** The ratified AOT path is `MLIR → LLVM` (RFC-0004 §2), but libMLIR (C++) is
//! absent in this environment; the full `ternary`-dialect → `arith`/`vector` → LLVM lowering is
//! therefore **deferred** (Phase 3 matures it once libMLIR is provisioned; ADR-009). What lands here:
//!
//! - [`dialect::emit`] — a **textual** ternary-dialect rendering of the lowered IR
//!   (`mycelium-core::lower` A-normal form): one dialect op per binding, every value/attr visible.
//!   This is the *per-stage-dumpable, no-opaque-pass* anchor (RFC-0004 §6) — text, not native code,
//!   and the dumpable skeleton of the eventual MLIR path.
//! - [`aot::run`] — the **env-machine** runnable model: an independent big-step evaluator over the
//!   lowered ANF (sequential binding evaluation) vs the reference interpreter's small-step
//!   substitution (M-110). A genuine *two-path* check for the interp↔AOT differential (M-151/NFR-7).
//! - [`llvm::compile_and_run`] — the **compiled native artifact** (M-301; RFC-0004 §2's *direct-LLVM
//!   fallback*): for the bit subset it emits textual LLVM IR ([`llvm::emit_llvm_ir`], one SSA op per
//!   output bit), drives `llc` + `clang` to a real executable, runs it, and reads the result back.
//!   This is a *third, compiled* execution path; everything outside the bit subset is an explicit
//!   [`llvm::AotError`] refusal (never silent), with `llc`/`clang` absence reported as a skippable
//!   `ToolchainMissing`. The interp↔native differential (M-302) checks it against the interpreter.

pub mod aot;
pub mod bitnet;
pub mod dialect;
pub mod jit;
pub mod llvm;
pub mod pack;

pub use aot::{run, run_with_layout};
pub use bitnet::{
    compile_bitnet_dot, emit_bitnet_dot_ir, jit_ternary_dot, ternary_dot_ref, BitnetDotKernel,
};
pub use dialect::emit;
pub use jit::{compile_so, jit_run, JitArtifact};
pub use llvm::{compile, compile_and_run, emit_llvm_ir, AotError, CompiledArtifact};
pub use pack::{pack_trits, relayout_trits, unpack_trits};
