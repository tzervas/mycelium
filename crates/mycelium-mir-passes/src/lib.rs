//! `mycelium-mir-passes` — the RC-annotated IR and reference-counting lowering passes (MEM-4).
//!
//! Implements the **MEM-4** leg of the DN-32 three-layer memory model: the static
//! uniqueness-analysis / Perceus-style reference-counting passes that DN-33 ratified (status
//! **Accepted**, §8.1). This crate is **optimisation-only and OUTSIDE the trusted Core IR** (KC-3 /
//! DN-33 §8.1 Q2): it *consumes* `mycelium_core::Node` read-only and produces a **separate**
//! RC-annotated IR ([`rc_ir::RcNode`]); the audited kernel (`mycelium-core`) does not grow, and a
//! bug here is a missed optimisation, never unsafety — the runtime `RcCell` probe
//! (`mycelium-std-runtime::rc`) remains the sound fallback (DN-33 §2).
//!
//! # What is built (MEM-4·B0 — the RC-emission pipeline foundation)
//!
//! The investigation recorded in DN-33 §6.1 found MEM-4 had *no input to operate on*: nothing
//! emitted RC ops, so there was nothing to elide. This crate supplies that foundation:
//!
//! - [`rc_ir`] — the **RC-annotated IR** `RcNode` (a mirror of the Core IR first-order fragment plus
//!   `Dup`/`Drop` wrappers and a per-binding own/borrow [`rc_ir::Mode`]).
//! - [`emit`] — the **naive (fully-owned) RC-emission** lowering `Node → RcNode`: a binding used `k`
//!   times gets `k-1` `Dup`s and each use consumes one; an unused binding gets one `Drop`. Recursion
//!   (`Fix`/`FixGroup`) is refused explicitly (G2 — never-silent).
//! - [`balance`] — the **structural balance invariant** (`1 + dups == uses + drops` per owned
//!   binding), verified independently over the emitted IR: the structural-invariant half of the
//!   ratified Q3 soundness strategy (DN-33 §8.1).
//!
//! # What is next (MEM-4 Increment 1 — not in this crate yet)
//!
//! Non-escaping **borrow elision** (mark non-consuming reads `Borrowed`, eliding their `Dup`/`Drop`)
//! plus the **differential** half of Q3 (run with/without elision, compare reclamation traces). The
//! [`rc_ir::Mode::Borrowed`] variant and the borrowed clause in [`balance`] are already in place for
//! it.
//!
//! # Guarantee posture (VR-5)
//!
//! The emission's **balance property** is `Exact` (by construction, independently checked). No
//! performance claim is made — B0 deliberately emits the *most* RC ops; Increment 1 removes the
//! redundant ones, and any `dup`/`drop`-reduction figure stays `Declared` until measured on a
//! corpus (DN-33 §8.1 Q5; the *count* is `Exact`, read off the IR).
//!
//! Design: `docs/notes/DN-33-Layer1-Static-Uniqueness-Analysis.md`; task E12 / MEM-4.
#![forbid(unsafe_code)]

pub mod balance;
pub mod emit;
pub mod rc_ir;

#[cfg(test)]
mod tests;
