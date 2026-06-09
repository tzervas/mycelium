//! `mycelium-mlir` — the MLIR→LLVM AOT path: a **ternary-dialect skeleton** (M-150; RFC-0004 §2/§6;
//! ADR-007; T1.5).
//!
//! **Scope / honesty.** A real backend binds libMLIR (C++) and emits LLVM IR → native; that is not
//! buildable in this Rust-only environment, and is **deferred** (Phase 3 matures it; ADR-009). What
//! lands here is the honest skeleton:
//!
//! - [`dialect::emit`] — a **textual** ternary-dialect rendering of the lowered IR
//!   (`mycelium-core::lower` A-normal form): one dialect op per binding, every value/attr visible.
//!   This is the *per-stage-dumpable, no-opaque-pass* anchor (RFC-0004 §6) — text, not native code.
//! - [`aot::run`] — the **runnable artifact for the subset**: an independent **big-step env-machine**
//!   that *executes the lowered ANF directly* (sequential binding evaluation), as opposed to the
//!   reference interpreter's small-step substitution (M-110). It models the compiled path's
//!   semantics so the interp↔AOT differential (M-151) is a genuine *two-path* check (NFR-7): the
//!   paths differ in IR shape and evaluation strategy, sharing only the trusted primitive/swap
//!   semantics, so the differential catches lowering/scheduling/ordering divergence.

pub mod aot;
pub mod dialect;

pub use aot::run;
pub use dialect::emit;
