//! Naive (fully-owned) RC-emission lowering `Node → RcNode` — MEM-4·B0 / DN-33 §6.
//!
//! This is the **foundation** the investigation (DN-33 §6.1) found missing: a lowering that *emits*
//! the reference-counting operations, so that MEM-4's elision (Increment 1) has something to elide.
//! It is **naive / fully-owned**: every binding owns its reference, so a binding used `k` times gets
//! `k-1` `Dup`s (one reference per use) and each use consumes one; a binding used `0` times gets one
//! `Drop`. No borrowing yet — that is Increment 1.
//!
//! # Scope — the first-order fragment (G2: never-silent on the rest)
//!
//! Total over `Const/Var/Let/Op/Swap/Construct/Match/Lam/App`. Recursion (`Fix`/`FixGroup`) is
//! **out of scope** for this increment (RC of recursive bindings is harder — DN-33 §6) and returns
//! an explicit [`EmitError::UnsupportedNode`] rather than being silently mis-emitted.
//!
//! # The emission rule (per owned binding)
//!
//! For a binding of `x` with `k` consuming uses in its scope body:
//! - `k == 0` → prepend one `Drop x` (the bound value is reclaimed immediately; never leaked).
//! - `k >= 1` → prepend `k-1` `Dup x` (so there are `k` references; the `k` uses consume them).
//!
//! The bound value starts with one reference (produced by evaluating `bound`), so the net is
//! `1 + (k-1) == k` references created and `k` consumed → balance zero (verified independently by
//! [`crate::balance`]).
//!
//! Guarantee: the emission is `Exact` **for the balance property** — by construction every owned
//! binding is reference-balanced (proven independently by the balance check, and mutation-tested).
//! No performance claim is made (B0 emits the *most* RC ops; Increment 1 removes the redundant
//! ones).

use mycelium_core::{Alt, Node};

use crate::rc_ir::{Mode, RcAlt, RcNode};

/// Why RC-emission could not lower a node.
///
/// Exhaustive and never-silent (G2): a node outside the supported fragment is an explicit error,
/// not a silent pass-through or a wrong emission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmitError {
    /// The node is outside the first-order fragment this increment supports (e.g. `Fix`/`FixGroup`).
    /// Carries the node kind for diagnostics.
    UnsupportedNode(&'static str),
}

impl std::fmt::Display for EmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmitError::UnsupportedNode(kind) => write!(
                f,
                "RC-emission does not yet support `{kind}` (recursion is a later MEM-4 increment — \
                 DN-33 §6); refusing rather than mis-emitting (G2)"
            ),
        }
    }
}

impl std::error::Error for EmitError {}

/// Lower a Core IR [`Node`] to the naive fully-owned [`RcNode`] (MEM-4·B0).
///
/// Returns [`EmitError::UnsupportedNode`] for `Fix`/`FixGroup` (G2 — never-silent on recursion).
///
/// Guarantee: `Exact` for the balance property (every owned binding is reference-balanced by
/// construction). See [`crate::balance::check_balance`] for the independent verification.
pub fn emit_owned(node: &Node) -> Result<RcNode, EmitError> {
    match node {
        Node::Const(v) => Ok(RcNode::Const(v.clone())),
        Node::Var(x) => Ok(RcNode::Var(x.clone())),
        Node::Let { id, bound, body } => {
            let rc_bound = emit_owned(bound)?;
            let k = count_occurrences(id, body);
            let rc_body = emit_owned(body)?;
            Ok(RcNode::Let {
                id: id.clone(),
                bound: Box::new(rc_bound),
                body: Box::new(balance_binder(id, k, rc_body)),
            })
        }
        Node::Op { prim, args } => Ok(RcNode::Op {
            prim: prim.clone(),
            args: emit_args(args)?,
        }),
        Node::Swap {
            src,
            target,
            policy,
        } => Ok(RcNode::Swap {
            src: Box::new(emit_owned(src)?),
            target: target.clone(),
            policy: policy.clone(),
        }),
        Node::Construct { ctor, args } => Ok(RcNode::Construct {
            ctor: ctor.clone(),
            args: emit_args(args)?,
        }),
        Node::Match {
            scrutinee,
            alts,
            default,
        } => {
            let rc_scrutinee = Box::new(emit_owned(scrutinee)?);
            let mut rc_alts = Vec::with_capacity(alts.len());
            for alt in alts {
                rc_alts.push(emit_alt(alt)?);
            }
            let rc_default = match default {
                Some(d) => Some(Box::new(emit_owned(d)?)),
                None => None,
            };
            Ok(RcNode::Match {
                scrutinee: rc_scrutinee,
                alts: rc_alts,
                default: rc_default,
            })
        }
        Node::Lam { param, body } => {
            let k = count_occurrences(param, body);
            let rc_body = emit_owned(body)?;
            Ok(RcNode::Lam {
                param: param.clone(),
                mode: Mode::Owned,
                body: Box::new(balance_binder(param, k, rc_body)),
            })
        }
        Node::App { func, arg } => Ok(RcNode::App {
            func: Box::new(emit_owned(func)?),
            arg: Box::new(emit_owned(arg)?),
        }),
        Node::Fix { .. } => Err(EmitError::UnsupportedNode("Fix")),
        Node::FixGroup { .. } => Err(EmitError::UnsupportedNode("FixGroup")),
    }
}

/// Emit each argument of an `Op`/`Construct`, short-circuiting on the first error.
fn emit_args(args: &[Node]) -> Result<Vec<RcNode>, EmitError> {
    args.iter().map(emit_owned).collect()
}

/// Emit one match alternative, balancing each of its (owned) field binders against its uses.
fn emit_alt(alt: &Alt) -> Result<RcAlt, EmitError> {
    match alt {
        Alt::Ctor {
            ctor,
            binders,
            body,
        } => {
            let rc_body = emit_owned(body)?;
            // Each binder is an owned binding scoped to this arm body; balance each against its
            // occurrence count. Nesting order is irrelevant to balance (each binder is independent).
            let wrapped = binders.iter().fold(rc_body, |acc, b| {
                let k = count_occurrences(b, body);
                balance_binder(b, k, acc)
            });
            Ok(RcAlt::Ctor {
                ctor: ctor.clone(),
                binders: binders.clone(),
                body: wrapped,
            })
        }
        Alt::Lit { value, body } => Ok(RcAlt::Lit {
            value: value.clone(),
            body: emit_owned(body)?,
        }),
    }
}

/// Place the owned-binding RC annotations at the top of `body`:
/// `k == 0` → one `Drop`; `k >= 1` → `k-1` `Dup`s.
fn balance_binder(var: &mycelium_core::VarId, k: usize, body: RcNode) -> RcNode {
    if k == 0 {
        RcNode::drop_one(var, body)
    } else {
        RcNode::dup_n(var, k - 1, body)
    }
}

/// Count the **free** consuming occurrences of `var` in `node`, respecting shadowing.
///
/// An inner binder that re-binds `var` shadows it: occurrences under that inner scope do **not**
/// count for the outer `var` (rubric A4-01 — analysis across binders must respect shadowing).
/// Total over the whole `Node` grammar (including `Fix`/`FixGroup`) so it is correct even where
/// emission later refuses — counting is never the thing that silently goes wrong.
///
/// Guarantee: `Exact` — a deterministic structural count.
#[must_use]
pub fn count_occurrences(var: &mycelium_core::VarId, node: &Node) -> usize {
    match node {
        Node::Const(_) => 0,
        Node::Var(x) => usize::from(x == var),
        Node::Let { id, bound, body } => {
            let in_bound = count_occurrences(var, bound);
            // `id` shadows `var` inside `body` only if they are the same name.
            let in_body = if id == var {
                0
            } else {
                count_occurrences(var, body)
            };
            in_bound + in_body
        }
        Node::Op { args, .. } | Node::Construct { args, .. } => {
            args.iter().map(|a| count_occurrences(var, a)).sum()
        }
        Node::Swap { src, .. } => count_occurrences(var, src),
        Node::Match {
            scrutinee,
            alts,
            default,
        } => {
            let mut n = count_occurrences(var, scrutinee);
            for alt in alts {
                n += match alt {
                    Alt::Ctor { binders, body, .. } => {
                        if binders.iter().any(|b| b == var) {
                            0 // shadowed by a field binder
                        } else {
                            count_occurrences(var, body)
                        }
                    }
                    Alt::Lit { body, .. } => count_occurrences(var, body),
                };
            }
            n + default.as_deref().map_or(0, |d| count_occurrences(var, d))
        }
        Node::Lam { param, body } => {
            if param == var {
                0
            } else {
                count_occurrences(var, body)
            }
        }
        Node::App { func, arg } => count_occurrences(var, func) + count_occurrences(var, arg),
        Node::Fix { name, body } => {
            if name == var {
                0
            } else {
                count_occurrences(var, body)
            }
        }
        Node::FixGroup { defs, body } => {
            if defs.iter().any(|(name, _)| name == var) {
                0 // the group binds all its names everywhere in defs + body
            } else {
                defs.iter()
                    .map(|(_, d)| count_occurrences(var, d))
                    .sum::<usize>()
                    + count_occurrences(var, body)
            }
        }
    }
}
