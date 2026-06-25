//! Reference RC-evaluator + differential harness — MEM-4 / DN-33 §8.1 Q3 (differential half).
//!
//! An **abstract reference-counting machine** over the RC-annotated IR's straight-line fragment
//! (`Const/Let/Op/Swap/Var/Borrow/Dup/Drop/DropAfter`). It does **not** compute values — it tracks
//! *references* and *reclamations* — and so serves as the executable semantics against which the
//! borrow elision is checked: [`differential`] runs a term's owned emission and its borrow-elided
//! emission and asserts they reclaim **the same multiset of values** (semantics-preserving) with
//! **no use-after-free**.
//!
//! # Accounting model (closed program)
//!
//! Each allocation (`Const`, and the fresh result of an `Op`/`Swap`) gets a distinct [`AllocId`]
//! with reference count 1. `Dup` increments; `Drop`/`DropAfter` and a consuming `Var` **move**
//! decrement (reclaiming at zero, logged in order); a `Borrow` is a non-consuming **read** that
//! asserts the value is still live. Reclamation order/identity is deterministic (allocation order is
//! fixed by the term skeleton, which is identical for the owned and elided emissions), so the two
//! reclamation logs are directly comparable.
//!
//! # Honesty (VR-5)
//!
//! This is an **abstract** machine (references + reclamation, not data): the consuming/reader
//! distinction is modelled, but operand *content* is not, so it checks the RC discipline, not value
//! correctness. Control-flow nodes (`App/Match/Construct/Lam`) and recursion are **out of the
//! straight-line fragment** and return an explicit [`RcError::UnsupportedNode`] (G2 — never-silent);
//! the differential corpus is straight-line. The elision's semantics-preservation is therefore
//! `Empirical` (differential trials over a corpus), not `Proven`.

use std::collections::HashMap;

use mycelium_core::VarId;

use crate::rc_ir::RcNode;

/// A distinct allocation identity (assigned in evaluation order).
pub type AllocId = u64;

/// An RC-discipline violation detected by the evaluator (never-silent, G2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RcError {
    /// A variable was used with no binding in scope.
    UnboundVar(VarId),
    /// A value was read (`Borrow`) or moved after it had already been reclaimed (rc was 0).
    UseAfterFree(VarId),
    /// A reference count was decremented below zero (over-release / double free).
    DoubleFree(VarId),
    /// A node outside the straight-line fragment (e.g. `App`/`Match`/`Construct`/`Lam`).
    UnsupportedNode(&'static str),
}

impl std::fmt::Display for RcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RcError::UnboundVar(v) => write!(f, "unbound variable `{v}`"),
            RcError::UseAfterFree(v) => write!(f, "use-after-free: `{v}` read/moved after reclaim"),
            RcError::DoubleFree(v) => {
                write!(f, "double free: reference count of `{v}` went negative")
            }
            RcError::UnsupportedNode(k) => {
                write!(
                    f,
                    "RC-evaluator does not support `{k}` (straight-line fragment only)"
                )
            }
        }
    }
}

impl std::error::Error for RcError {}

/// The outcome of evaluating a term: its result allocation and the reclamation log (in order).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalReport {
    /// The allocation yielded as the term's result (it escapes — not necessarily reclaimed).
    pub result: AllocId,
    /// The allocations reclaimed (reference count reached zero), in reclamation order.
    pub reclaimed: Vec<AllocId>,
}

impl EvalReport {
    /// The reclamation log as a sorted multiset — the comparison key for [`differential`]
    /// (order-independent: elision may change *when* a value is reclaimed, not *whether*).
    #[must_use]
    pub fn reclaimed_sorted(&self) -> Vec<AllocId> {
        let mut v = self.reclaimed.clone();
        v.sort_unstable();
        v
    }
}

struct Machine {
    next: AllocId,
    rc: HashMap<AllocId, i64>,
    reclaimed: Vec<AllocId>,
}

impl Machine {
    fn new() -> Self {
        Machine {
            next: 0,
            rc: HashMap::new(),
            reclaimed: Vec::new(),
        }
    }

    fn alloc(&mut self) -> AllocId {
        let id = self.next;
        self.next += 1;
        self.rc.insert(id, 1);
        id
    }

    fn dup(&mut self, a: AllocId) {
        *self.rc.entry(a).or_insert(0) += 1;
    }

    /// Decrement; reclaim at zero, error below zero.
    fn dec(&mut self, a: AllocId, var: &VarId) -> Result<(), RcError> {
        let r = self.rc.entry(a).or_insert(0);
        *r -= 1;
        if *r == 0 {
            self.reclaimed.push(a);
            Ok(())
        } else if *r < 0 {
            Err(RcError::DoubleFree(var.clone()))
        } else {
            Ok(())
        }
    }

    fn assert_live(&self, a: AllocId, var: &VarId) -> Result<(), RcError> {
        if self.rc.get(&a).copied().unwrap_or(0) > 0 {
            Ok(())
        } else {
            Err(RcError::UseAfterFree(var.clone()))
        }
    }
}

/// Evaluate an [`RcNode`] in the abstract RC machine, returning its reclamation report.
///
/// Errors (never-silent) on an unbound variable, a use-after-free, a double-free, or a node outside
/// the straight-line fragment.
pub fn eval(node: &RcNode) -> Result<EvalReport, RcError> {
    let mut m = Machine::new();
    let env = HashMap::new();
    let result = go(node, &env, &mut m)?;
    Ok(EvalReport {
        result,
        reclaimed: m.reclaimed,
    })
}

fn lookup(env: &HashMap<VarId, AllocId>, x: &VarId) -> Result<AllocId, RcError> {
    env.get(x)
        .copied()
        .ok_or_else(|| RcError::UnboundVar(x.clone()))
}

fn go(node: &RcNode, env: &HashMap<VarId, AllocId>, m: &mut Machine) -> Result<AllocId, RcError> {
    match node {
        RcNode::Const(_) => Ok(m.alloc()),
        RcNode::Var(x) => {
            // Move: consume one reference of x.
            let a = lookup(env, x)?;
            m.dec(a, x)?;
            Ok(a)
        }
        RcNode::Borrow(x) => {
            // Read: the value must be live; reference count unchanged.
            let a = lookup(env, x)?;
            m.assert_live(a, x)?;
            Ok(a)
        }
        RcNode::Dup { var, body } => {
            let a = lookup(env, var)?;
            m.dup(a);
            go(body, env, m)
        }
        RcNode::Drop { var, body } => {
            let a = lookup(env, var)?;
            m.dec(a, var)?;
            go(body, env, m)
        }
        RcNode::DropAfter { var, body } => {
            // Evaluate the body (its reads of `var` happen here), THEN reclaim `var`.
            let r = go(body, env, m)?;
            let a = lookup(env, var)?;
            m.dec(a, var)?;
            Ok(r)
        }
        RcNode::Let { id, bound, body } => {
            let a = go(bound, env, m)?;
            let mut e2 = env.clone();
            e2.insert(id.clone(), a);
            go(body, &e2, m)
        }
        RcNode::Op { args, .. } => {
            for arg in args {
                go(arg, env, m)?;
            }
            Ok(m.alloc()) // the primitive produces a fresh result
        }
        RcNode::Swap { src, .. } => {
            go(src, env, m)?;
            Ok(m.alloc())
        }
        RcNode::Construct { .. } => Err(RcError::UnsupportedNode("Construct")),
        RcNode::Match { .. } => Err(RcError::UnsupportedNode("Match")),
        RcNode::Lam { .. } => Err(RcError::UnsupportedNode("Lam")),
        RcNode::App { .. } => Err(RcError::UnsupportedNode("App")),
    }
}

/// The verdict of a differential run on one term (DN-33 §8.1 Q3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Differential {
    /// Whether the owned and elided emissions reclaim the **same multiset** of values.
    pub same_reclamations: bool,
    /// `Dup` count of the owned emission.
    pub owned_dups: usize,
    /// `Dup` count of the elided emission.
    pub elided_dups: usize,
}

impl Differential {
    /// The elision is **semantics-preserving** iff the reclamation multisets match.
    #[must_use]
    pub fn is_semantics_preserving(&self) -> bool {
        self.same_reclamations
    }

    /// `Dup`s removed by elision (≥ 0; the optimisation's effect — `Exact`, read off the IR).
    #[must_use]
    pub fn dups_removed(&self) -> usize {
        self.owned_dups.saturating_sub(self.elided_dups)
    }
}

/// Count `Dup` nodes anywhere in an [`RcNode`].
#[must_use]
pub fn count_dups(node: &RcNode) -> usize {
    match node {
        RcNode::Const(_) | RcNode::Var(_) | RcNode::Borrow(_) => 0,
        RcNode::Dup { body, .. } => 1 + count_dups(body),
        RcNode::Drop { body, .. } | RcNode::DropAfter { body, .. } => count_dups(body),
        RcNode::Let { bound, body, .. } => count_dups(bound) + count_dups(body),
        RcNode::Op { args, .. } | RcNode::Construct { args, .. } => {
            args.iter().map(count_dups).sum()
        }
        RcNode::Swap { src, .. } => count_dups(src),
        RcNode::Match {
            scrutinee,
            alts,
            default,
        } => {
            count_dups(scrutinee)
                + alts
                    .iter()
                    .map(|a| match a {
                        crate::rc_ir::RcAlt::Ctor { body, .. }
                        | crate::rc_ir::RcAlt::Lit { body, .. } => count_dups(body),
                    })
                    .sum::<usize>()
                + default.as_deref().map_or(0, count_dups)
        }
        RcNode::Lam { body, .. } => count_dups(body),
        RcNode::App { func, arg } => count_dups(func) + count_dups(arg),
    }
}

/// Run the differential check: evaluate the owned and elided emissions of the **same** Core IR term
/// and compare. Returns the [`Differential`] verdict, or an [`RcError`] if either emission
/// use-after-frees / double-frees (which would itself be a soundness failure of the elision).
///
/// The two emissions are supplied as already-lowered [`RcNode`]s so this function stays independent
/// of `crate::emit` (the caller pairs `emit_owned(t)` with `emit_elided(t)`).
pub fn differential(owned: &RcNode, elided: &RcNode) -> Result<Differential, RcError> {
    let owned_report = eval(owned)?;
    let elided_report = eval(elided)?;
    Ok(Differential {
        same_reclamations: owned_report.reclaimed_sorted() == elided_report.reclaimed_sorted(),
        owned_dups: count_dups(owned),
        elided_dups: count_dups(elided),
    })
}
