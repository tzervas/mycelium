//! The swap extension point for the interpreter (M-110).
//!
//! A [`Node::Swap`](mycelium_core::Node::Swap) is the *only* representation-changing node
//! (RFC-0001 §4.5 WF1); evaluating one is delegated to a [`SwapEngine`]. The real, certificate-
//! emitting binary↔ternary swap is **M-120** (`docs/spec/swaps/binary-ternary.md`); this module
//! defines the interface plus the trivial [`IdentitySwapEngine`] (a same-`Repr` swap, which is
//! exactly lossless) so the interpreter can evaluate swap nodes today. An unsupported swap is an
//! explicit error — **never** a silent coercion (SC-3; G2).

use mycelium_core::{ContentHash, Meta, Provenance, Repr, Value};

use crate::EvalError;

/// Evaluates a `Swap` node. Implementations must be *never silent*: an out-of-domain or unsupported
/// conversion returns an [`EvalError`] (the interpreter surfaces it), it does not coerce.
pub trait SwapEngine {
    /// Convert `src` to `target` under `policy`, returning the converted [`Value`] or an error. The
    /// result's [`Meta`] must record `policy_used` (ADR-006) and an honest guarantee/bound.
    fn swap(&self, src: &Value, target: &Repr, policy: &ContentHash) -> Result<Value, EvalError>;
}

/// The trivial swap engine: a swap whose `target` equals the source `Repr` is the identity — exactly
/// lossless, `guarantee` preserved. Any *cross-paradigm* swap returns
/// [`EvalError::UnsupportedSwap`], deferring to the certified M-120 engine. This keeps M-110 honest:
/// it executes the swaps it can prove trivially exact and refuses the rest rather than guessing.
#[derive(Debug, Clone, Copy, Default)]
pub struct IdentitySwapEngine;

impl SwapEngine for IdentitySwapEngine {
    fn swap(&self, src: &Value, target: &Repr, policy: &ContentHash) -> Result<Value, EvalError> {
        if src.repr() != target {
            return Err(EvalError::UnsupportedSwap {
                from: src.repr().clone(),
                to: target.clone(),
            });
        }
        // Same representation → identity. The value is unchanged; metadata records that it was
        // produced by a swap (policy_used set, ADR-006) and keeps the source's guarantee/bound.
        let src_meta = src.meta();
        let meta = Meta::new(
            Provenance::Derived {
                op: mycelium_core::operation_hash("swap.identity"),
                inputs: vec![src.content_hash()],
            },
            src_meta.guarantee(),
            src_meta.bound().cloned(),
            src_meta.sparsity(),
            src_meta.physical(),
            Some(policy.clone()),
        )
        .map_err(EvalError::Wf)?;
        Value::new(src.repr().clone(), src.payload().clone(), meta).map_err(EvalError::Wf)
    }
}
