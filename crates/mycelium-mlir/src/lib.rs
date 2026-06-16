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
//! - [`budget::DepthBudget`] — the **dynamic depth budget** for the env-machine's control stack
//!   (DN-05 §2.4 / DN05-Q5): with the M-347 trampoline the control stack is on the heap, so the depth
//!   ceiling is a *policy over memory headroom*, derived from detected `MemAvailable`/`RLIMIT_AS`
//!   (zero-`unsafe`, pure-`std` `/proc`) with a conservative static fallback and an `EXPLAIN`-able
//!   basis — never a magic constant, never an abort ([`aot::default_depth_budget`]).
//! - [`inject::Image`] — the **in-process hot-inject** prototype (M-341; ADR-016/017): a hash-keyed
//!   `ContentHash → entry` dispatch table over the M-340 JIT. A call resolves to a compiled entry if
//!   present, else interprets (RFC-0004 §9 continuum); injection loads a content-addressed unit and
//!   registers a *new* `hash → entry`, never mutating a live entry (immutability dissolves the
//!   atomicity hazard); the recompile set is the changed dependency-closure by hash reachability
//!   ([`inject::recompile_closure`]). The injected-compiled path is M-210-checked against the
//!   interpreter (NFR-7).

pub mod aot;
pub mod bitnet;
pub mod budget;
pub mod channel;
pub mod dialect;
pub mod inject;
pub mod jit;
pub mod llvm;
pub mod pack;
pub mod runtime;
pub mod simd;
pub mod specialize;

pub use aot::{
    default_depth_budget, run, run_core, run_core_with_effects, run_core_with_fuel, run_with_layout,
};
pub use bitnet::{
    compile_bitnet_dot, compile_bitnet_dot_for, emit_bitnet_dot_ir, emit_bitnet_dot_ir_for,
    jit_ternary_dot, jit_ternary_dot_for, ternary_dot_ref, BitnetDotKernel,
};
pub use budget::{
    AutoDepthBudget, DepthBasis, DepthBudget, DepthResolution, MemSource, StaticDepthBudget,
    StaticReason, STATIC_FALLBACK_DEPTH,
};
pub use channel::{Network, Receiver, Sender, TryRecv, TrySend};
pub use dialect::emit;
pub use inject::{recompile_closure, Image, InjectError, Resolution};
pub use jit::{compile_so, jit_run, JitArtifact};
pub use llvm::{compile, compile_and_run, emit_llvm_ir, AotError, CompiledArtifact};
pub use pack::{pack_trits, relayout_trits, unpack_trits};
pub use runtime::{Colony, Deadlock, Poll, Scope, SweepOrder, Task, TaskCtx};
pub use simd::{compile_bitnet_dot_simd, emit_bitnet_dot_simd_ir};
pub use specialize::{
    compile_specialized_dot, emit_specialized_dot_ir, jit_specialized_dot, SpecializedDotKernel,
};
