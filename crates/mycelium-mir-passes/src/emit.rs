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

use mycelium_core::{Alt, Node, VarId};

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

// ── MEM-4 Increment 1 — non-escaping borrow elision ──────────────────────────
//
// In the immutable value model, a reader primitive (`Op`/`Swap`) *reads* its operands and produces
// a fresh result, retaining nothing — so an operand position is a **borrow** (non-consuming read),
// not a move. A `let` binding whose every use is such a borrow is **fully borrowable**: it needs no
// `Dup` (one reference serves all reads) and is reclaimed by a single `DropAfter` once its reads are
// done. This is strictly fewer RC ops than the naive owned emission (`k-1` `Dup`s → `0`), and it is
// **semantics-preserving** (the same value is reclaimed, exactly once) — verified by the differential
// harness in [`crate::eval`].
//
// Increment 1 is intraprocedural and conservative: it elides only **fully-borrowable `let`
// bindings**. A binding with ANY escaping use (the binding flows to the result, into a `Construct`,
// or to an `App`/`Match`) stays fully **owned** (the naive emission). `Lam` parameters stay `Owned`
// (interprocedural borrowing — `Mode::Borrowed` at a call boundary — is a later increment).

use std::collections::HashSet;

use crate::rc_ir::RcNode as N;

/// Lower a Core IR [`Node`] with MEM-4 Increment 1 **borrow elision** applied.
///
/// Identical to [`emit_owned`] except that every **fully-borrowable** `let` binding (every use is a
/// reader-primitive read — [`is_fully_borrowable`]) is emitted with its uses as
/// [`RcNode::Borrow`](crate::rc_ir::RcNode::Borrow), **no** `Dup`, and a single
/// [`RcNode::DropAfter`](crate::rc_ir::RcNode::DropAfter) reclaiming it after its reads.
///
/// Returns [`EmitError::UnsupportedNode`] for `Fix`/`FixGroup` (G2 — never-silent on recursion).
///
/// Guarantee: the elision is **semantics-preserving** — `Empirical` (the differential harness in
/// [`crate::eval`] checks that, for a corpus of terms, the multiset of reclaimed values is identical
/// to the owned emission's, with no use-after-free), backed by the structural `DropAfter`-after-reads
/// argument. The `dup`-count reduction is `Exact` (read off the IR); the *performance* benefit of
/// that reduction stays `Declared` until measured (DN-33 §8.1 Q5).
pub fn emit_elided(node: &Node) -> Result<RcNode, EmitError> {
    // reuse = false: borrow elision only (Increment 1).
    emit_ann(node, &Ann::new(false))
}

/// Lower a Core IR [`Node`] with MEM-4 Increment 1 (**borrow elision**) **and** Increment 2
/// (**`rc == 1` reuse annotation**) applied.
///
/// A superset of [`emit_elided`]: in addition to borrow-eliding fully-borrowable bindings, a `let`
/// binding that is a **sole-owned single move** ([`is_sole_owned_move`] — used exactly once, in a
/// move position) has that move emitted as [`RcNode::MoveUnique`](crate::rc_ir::RcNode::MoveUnique),
/// recording that the runtime `UniqueOwner` branch is statically guaranteed (FBIP-reuse-eligible).
///
/// Returns [`EmitError::UnsupportedNode`] for `Fix`/`FixGroup` (G2).
///
/// Guarantee: the reuse annotation is **semantics-preserving** and its soundness is
/// **machine-verified** — [`crate::eval`] errors ([`crate::eval::RcError::UnsoundUnique`]) if any
/// `MoveUnique` is reached at a reference count other than 1. Tag `Empirical` (differential + the
/// verifying evaluator), not `Proven`.
pub fn emit_reuse(node: &Node) -> Result<RcNode, EmitError> {
    // reuse = true: borrow elision + the rc==1 reuse annotation (Increment 2).
    emit_ann(node, &Ann::new(true))
}

/// Annotation context threaded through emission: the in-scope `borrowed` and `unique` variable sets
/// and whether the `rc == 1` reuse annotation (Increment 2) is enabled.
#[derive(Clone)]
struct Ann {
    borrowed: HashSet<VarId>,
    unique: HashSet<VarId>,
    reuse: bool,
}

impl Ann {
    fn new(reuse: bool) -> Self {
        Ann {
            borrowed: HashSet::new(),
            unique: HashSet::new(),
            reuse,
        }
    }

    fn with_borrowed(&self, id: &VarId) -> Ann {
        let mut a = self.clone();
        a.borrowed.insert(id.clone());
        a
    }

    fn with_unique(&self, id: &VarId) -> Ann {
        let mut a = self.clone();
        a.unique.insert(id.clone());
        a
    }
}

/// Emit `node` under the annotation context: every use of a `borrowed` variable becomes a
/// non-consuming [`RcNode::Borrow`]; every use of a `unique` variable becomes a sole-owner
/// [`RcNode::MoveUnique`]; fully-borrowable `let`s are borrow-elided; sole-owned single-move `let`s
/// are reuse-annotated (when `reuse` is enabled).
fn emit_ann(node: &Node, ann: &Ann) -> Result<RcNode, EmitError> {
    match node {
        Node::Const(v) => Ok(N::Const(v.clone())),
        Node::Var(x) => Ok(if ann.borrowed.contains(x) {
            N::Borrow(x.clone())
        } else if ann.unique.contains(x) {
            N::MoveUnique(x.clone())
        } else {
            N::Var(x.clone())
        }),
        Node::Let { id, bound, body } => {
            let rc_bound = emit_ann(bound, ann)?;
            if is_fully_borrowable(id, body) {
                // Borrow-elide: read `id` without consuming, reclaim with a single DropAfter.
                let rc_body = emit_ann(body, &ann.with_borrowed(id))?;
                Ok(N::Let {
                    id: id.clone(),
                    bound: Box::new(rc_bound),
                    body: Box::new(N::drop_after(id, rc_body)),
                })
            } else if ann.reuse && is_sole_owned_move(id, body) {
                // Reuse-annotate (Increment 2): `id` is used exactly once, in a move position, so its
                // reference count is statically 1 at that consume → emit it as MoveUnique. k == 1 ⇒
                // no Dup, no Drop (the single move reclaims it).
                let rc_body = emit_ann(body, &ann.with_unique(id))?;
                Ok(N::Let {
                    id: id.clone(),
                    bound: Box::new(rc_bound),
                    body: Box::new(balance_binder(id, 1, rc_body)),
                })
            } else {
                // Owned (naive) emission for this binding.
                let k = count_occurrences(id, body);
                let rc_body = emit_ann(body, ann)?;
                Ok(N::Let {
                    id: id.clone(),
                    bound: Box::new(rc_bound),
                    body: Box::new(balance_binder(id, k, rc_body)),
                })
            }
        }
        Node::Op { prim, args } => Ok(N::Op {
            prim: prim.clone(),
            args: emit_args_a(args, ann)?,
        }),
        Node::Swap {
            src,
            target,
            policy,
        } => Ok(N::Swap {
            src: Box::new(emit_ann(src, ann)?),
            target: target.clone(),
            policy: policy.clone(),
        }),
        Node::Construct { ctor, args } => Ok(N::Construct {
            ctor: ctor.clone(),
            args: emit_args_a(args, ann)?,
        }),
        Node::Match {
            scrutinee,
            alts,
            default,
        } => {
            let rc_scrutinee = Box::new(emit_ann(scrutinee, ann)?);
            let mut rc_alts = Vec::with_capacity(alts.len());
            for alt in alts {
                rc_alts.push(emit_alt_a(alt, ann)?);
            }
            let rc_default = match default {
                Some(d) => Some(Box::new(emit_ann(d, ann)?)),
                None => None,
            };
            Ok(N::Match {
                scrutinee: rc_scrutinee,
                alts: rc_alts,
                default: rc_default,
            })
        }
        Node::Lam { param, body } => {
            // Lam params stay Owned (interprocedural borrowing is a later increment).
            let k = count_occurrences(param, body);
            let rc_body = emit_ann(body, ann)?;
            Ok(N::Lam {
                param: param.clone(),
                mode: Mode::Owned,
                body: Box::new(balance_binder(param, k, rc_body)),
            })
        }
        Node::App { func, arg } => Ok(N::App {
            func: Box::new(emit_ann(func, ann)?),
            arg: Box::new(emit_ann(arg, ann)?),
        }),
        Node::Fix { .. } => Err(EmitError::UnsupportedNode("Fix")),
        Node::FixGroup { .. } => Err(EmitError::UnsupportedNode("FixGroup")),
    }
}

fn emit_args_a(args: &[Node], ann: &Ann) -> Result<Vec<RcNode>, EmitError> {
    args.iter().map(|a| emit_ann(a, ann)).collect()
}

fn emit_alt_a(alt: &Alt, ann: &Ann) -> Result<RcAlt, EmitError> {
    match alt {
        Alt::Ctor {
            ctor,
            binders,
            body,
        } => {
            let rc_body = emit_ann(body, ann)?;
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
            body: emit_ann(body, ann)?,
        }),
    }
}

/// Whether `var`'s binding is a **sole-owned single move**: used **exactly once**, and that use is a
/// **move** (not a borrow position). At such a use the reference count is statically 1, so the
/// runtime `UniqueOwner` branch is guaranteed — the `rc == 1` reuse annotation (Increment 2) applies.
///
/// Guarantee: `Exact` — a deterministic structural test (conservative: only the unambiguous
/// single-move case; multi-move last-consume is a later refinement).
#[must_use]
pub fn is_sole_owned_move(var: &VarId, body: &Node) -> bool {
    count_occurrences(var, body) == 1 && borrow_occurrences(var, body) == 0
}

/// Whether `var`'s binding is **fully borrowable** over `body`: it is used at least once and **every**
/// use is in a reader-primitive (borrow) position (`Op` argument or `Swap` source). Such a binding
/// never escapes (it does not flow to the result, into a `Construct`, or to an `App`/`Match`), so it
/// can be read without consuming and reclaimed once at the end.
///
/// Guarantee: `Exact` — a deterministic structural test (conservative: any escaping use makes it
/// `false`, keeping the binding owned — never wrongly elided).
#[must_use]
pub fn is_fully_borrowable(var: &VarId, body: &Node) -> bool {
    let total = count_occurrences(var, body);
    total >= 1 && borrow_occurrences(var, body) == total
}

/// Count occurrences of `var` in **borrow positions** (direct `Op` argument / `Swap` source),
/// respecting shadowing. A bare `Var`, a `Construct`/`App`/`Match`/tail occurrence is **not** a
/// borrow position (those are moves/escapes).
#[must_use]
pub fn borrow_occurrences(var: &VarId, node: &Node) -> usize {
    match node {
        Node::Const(_) | Node::Var(_) => 0,
        Node::Let { id, bound, body } => {
            borrow_occurrences(var, bound)
                + if id == var {
                    0
                } else {
                    borrow_occurrences(var, body)
                }
        }
        // Op args and Swap src ARE borrow positions: a direct `Var(var)` child counts; a deeper
        // child is recursed (its own immediate parent decides).
        Node::Op { args, .. } => args.iter().map(|a| arg_borrow(var, a)).sum(),
        Node::Swap { src, .. } => arg_borrow(var, src),
        // Construct args are MOVES (stored into the data value): a direct `Var(var)` is NOT a borrow;
        // only deeper reader positions count → recurse with `borrow_occurrences` (not `arg_borrow`).
        Node::Construct { args, .. } => args.iter().map(|a| borrow_occurrences(var, a)).sum(),
        Node::Match {
            scrutinee,
            alts,
            default,
        } => {
            // The scrutinee is a move (deconstructed), not a borrow → recurse for deeper readers.
            let mut n = borrow_occurrences(var, scrutinee);
            for alt in alts {
                n += match alt {
                    Alt::Ctor { binders, body, .. } => {
                        if binders.iter().any(|b| b == var) {
                            0
                        } else {
                            borrow_occurrences(var, body)
                        }
                    }
                    Alt::Lit { body, .. } => borrow_occurrences(var, body),
                };
            }
            n + default.as_deref().map_or(0, |d| borrow_occurrences(var, d))
        }
        Node::Lam { param, body } => {
            if param == var {
                0
            } else {
                borrow_occurrences(var, body)
            }
        }
        Node::App { func, arg } => borrow_occurrences(var, func) + borrow_occurrences(var, arg),
        Node::Fix { name, body } => {
            if name == var {
                0
            } else {
                borrow_occurrences(var, body)
            }
        }
        Node::FixGroup { defs, body } => {
            if defs.iter().any(|(name, _)| name == var) {
                0
            } else {
                defs.iter()
                    .map(|(_, d)| borrow_occurrences(var, d))
                    .sum::<usize>()
                    + borrow_occurrences(var, body)
            }
        }
    }
}

/// One reader-primitive argument: a direct `Var(var)` is a borrow position (count 1); anything else
/// recurses (its own structure decides where `var`'s borrows are).
fn arg_borrow(var: &VarId, arg: &Node) -> usize {
    match arg {
        Node::Var(x) if x == var => 1,
        other => borrow_occurrences(var, other),
    }
}
