//! Parallel evaluation of **provably-pure** Core IR fragments (M-862; RFC-0008 §4.2; ADR-034;
//! DN-25). This is a **perf path**, validated against the sequential reference by a differential —
//! never a second semantics: [`Interpreter::eval_core`]/[`Interpreter::step`] stay the trusted,
//! small-step meaning of a program (RFC-0008 §4.2, mirroring the AOT path's relationship to the
//! interpreter, KC-3 — concurrency adds *scheduling*, never new *meaning*).
//!
//! # What gets parallelized (EXPLAIN-able, never silent — G2)
//! Only the **independent field lists** of [`Node::Construct`]/[`Node::Op`] — each field/argument is
//! a closed subterm with no data dependency on its siblings (substitution has already closed every
//! `Var` before evaluation reaches here; RFC-0001 §4.5) — are fanned out onto rayon's bounded global
//! thread pool. Everything else ([`Node::Let`], [`Node::Match`] arm *selection*, [`Node::App`]
//! β-reduction, [`Node::Fix`]/[`Node::FixGroup`] unfolding) stays exactly the sequential reduction
//! order, because those steps are either inherently ordered (a `Let`'s body needs the bound value) or
//! would otherwise force *speculative* execution of a branch that may never be taken (a `Match` arm) —
//! and a not-taken arm can still be *ill-formed* (e.g. `NonExhaustiveMatch` deeper inside), so running
//! it "for free" in parallel could surface an error that a sequential run never would. `Node::App`'s
//! function/argument positions ARE evaluated concurrently (`rayon::join`) since both are already
//! independent, closed subterms under call-by-value — mirroring `Construct`/`Op`.
//!
//! # The purity gate ([`is_pure`])
//! [`is_pure`] is a **whole-fragment, structural** (syntactic) predicate, deliberately conservative
//! and all-or-nothing: if *any* subterm is not provably effect-free, the *whole* fragment falls back
//! to the ordinary sequential [`Interpreter::eval_core`] — never a partial/mixed evaluation order.
//! Grounding for what counts as "provably pure":
//! - Every built-in [`crate::prims::PrimFn`] is documented as "a pure function from argument values
//!   to a result value" ([`crate::prims`] module docs) — **except** the reserved `wild:`-namespaced
//!   host-capability escape hatch (RFC-0028 §4.3), the *only* place a `Node::Op` can reach an
//!   arbitrary, potentially-effectful host operation. `is_pure` therefore excludes any `Op` whose
//!   `prim` starts with `"wild:"`.
//! - [`Node::Swap`] delegates to a runtime-supplied `Box<dyn SwapEngine>` (crate::swap) whose
//!   concrete behaviour cannot be statically inspected from a `Node` alone (a custom engine could do
//!   anything). `is_pure` conservatively treats **every** `Swap` node as an opacity/parallelism
//!   boundary — never assumed pure, even though the shipped [`crate::swap::IdentitySwapEngine`]
//!   happens to be.
//! - `Const`/`Var`/`Lam` are trivially pure (no reduction).
//!
//! # Tag: Empirical (differential-checked)
//! The equivalence `eval_core_parallel(e) == eval_core(e)` for `e` in the pure fragment is checked by
//! a corpus differential (`src/tests/parallel.rs`), not proven — tagged **Empirical** per the
//! transparency lattice (never upgraded to `Proven` without a checked side-condition, VR-5).

use std::sync::atomic::{AtomicU64, Ordering};

use mycelium_core::{CoreValue, GuaranteeStrength, Node};
use rayon::prelude::*;

use crate::{as_const, collect_values, guarantee_of_value, node_to_core_value, select_arm, subst};
use crate::{EvalError, Interpreter};

/// Whether `node` is a **provably pure** (effect-free) Core IR fragment — the structural,
/// conservative, EXPLAIN-able gate that governs whether [`Interpreter::eval_parallel`]/
/// [`Interpreter::eval_core_parallel`] may reorder/parallelize its evaluation. See the module docs
/// for the grounding of each case. All-or-nothing over the whole subtree: a single impure/opaque leaf
/// (a `wild:` op, or any `Swap`) makes the **entire** fragment ineligible, never just the leaf.
#[must_use]
pub fn is_pure(node: &Node) -> bool {
    match node {
        Node::Const(_) | Node::Var(_) | Node::Lam { .. } => true,
        // The reserved host-capability escape hatch (RFC-0028 §4.3) is the one channel a `Node::Op`
        // can reach an arbitrary, potentially-effectful implementation through; every other prim is
        // documented pure (`crate::prims` module docs).
        Node::Op { prim, args } => !prim.starts_with("wild:") && args.iter().all(is_pure),
        // A `Box<dyn SwapEngine>` is an opaque, runtime-supplied implementation (crate::swap) — its
        // purity cannot be verified from the `Node` alone, so it is conservatively never pure.
        Node::Swap { .. } => false,
        Node::Let { bound, body, .. } => is_pure(bound) && is_pure(body),
        Node::Construct { args, .. } => args.iter().all(is_pure),
        Node::Match {
            scrutinee,
            alts,
            default,
        } => {
            is_pure(scrutinee)
                && alts.iter().all(|a| match a {
                    mycelium_core::Alt::Ctor { body, .. } => is_pure(body),
                    mycelium_core::Alt::Lit { body, .. } => is_pure(body),
                })
                && default.as_deref().is_none_or(is_pure)
        }
        Node::App { func, arg } => is_pure(func) && is_pure(arg),
        Node::Fix { body, .. } => is_pure(body),
        Node::FixGroup { defs, body } => defs.iter().all(|(_, d)| is_pure(d)) && is_pure(body),
    }
}

/// Tick the shared fuel counter (an [`AtomicU64`] so concurrent branches can share one budget),
/// returning [`EvalError::FuelExhausted`] on underflow — never silent, and never a per-branch budget
/// that could let two threads jointly overrun the declared total (RFC-0007 §4.5 CakeML clock).
fn tick(fuel: &AtomicU64) -> Result<(), EvalError> {
    fuel.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |f| f.checked_sub(1))
        .map(|_| ())
        .map_err(|_| EvalError::FuelExhausted)
}

impl Interpreter {
    /// Evaluate `node` to a [`CoreValue`] exactly like [`Interpreter::eval_core`], except that when
    /// the **whole** fragment is [`is_pure`], independent `Construct`/`Op` argument lists (and an
    /// `App`'s function/argument positions) are evaluated **in parallel** on rayon's bounded global
    /// thread pool instead of the strictly left-to-right small-step order. Observable behaviour is
    /// identical to the sequential reference (RT2-preserving) — the *only* difference is which CPU
    /// evaluates which independent subterm, never a difference in the result. An impure fragment
    /// (reaches a `wild:` op or any `Swap`) falls back to plain [`Interpreter::eval_core`] in full —
    /// never a partial/mixed order (never-silent selection, G2).
    ///
    /// **Empirical, differential-checked** (M-862): `eval_core_parallel(e) == eval_core(e)` for `e`
    /// in the pure fragment is checked over a corpus in `src/tests/parallel.rs`, not proven.
    pub fn eval_core_parallel(&self, node: &Node) -> Result<CoreValue, EvalError> {
        if !is_pure(node) {
            return self.eval_core(node);
        }
        let fuel = AtomicU64::new(self.fuel);
        let normal = self.eval_value_parallel(node, &fuel)?;
        node_to_core_value(&normal)
    }

    /// Evaluate `node` to a representation [`crate::Value`], mirroring [`Interpreter::eval`] — see
    /// [`Interpreter::eval_core_parallel`] for the parallel-evaluation contract.
    pub fn eval_parallel(&self, node: &Node) -> Result<crate::Value, EvalError> {
        match self.eval_core_parallel(node)? {
            CoreValue::Repr(v) => Ok(v),
            CoreValue::Data(_) => Err(EvalError::DataResult),
        }
    }

    /// The big-step, purity-gated evaluator: reduces `node` fully to a normal-form [`Node`] (a
    /// `Const` or a saturated `Construct` of values). Only ever called on a subtree that is already
    /// known [`is_pure`] (checked once at the [`Interpreter::eval_core_parallel`] entry point, and
    /// preserved by construction since every `is_pure` case requires all its children to be pure
    /// too) — so no case here re-checks purity, and the `Swap` arm below (unreachable from a pure
    /// caller) is still implemented faithfully rather than `unreachable!()`, matching the crate's
    /// no-panics discipline (G2).
    fn eval_value_parallel(&self, node: &Node, fuel: &AtomicU64) -> Result<Node, EvalError> {
        match node {
            Node::Const(_) | Node::Lam { .. } => Ok(node.clone()),
            Node::Var(x) => Err(EvalError::FreeVariable(x.clone())),

            Node::Let { id, bound, body } => {
                let b = self.eval_value_parallel(bound, fuel)?;
                tick(fuel)?;
                let next = subst(body, id, &b);
                self.eval_value_parallel(&next, fuel)
            }

            Node::Op { prim, args } => {
                let evaluated: Vec<Node> = args
                    .par_iter()
                    .map(|a| self.eval_value_parallel(a, fuel))
                    .collect::<Result<_, _>>()?;
                let values = collect_values(&evaluated)?;
                let f = self
                    .prims
                    .get(prim)
                    .ok_or_else(|| EvalError::UnknownPrim(prim.clone()))?;
                let result = f(prim, &values)?;
                tick(fuel)?;
                Ok(Node::Const(result))
            }

            // Unreachable from a pure caller (`is_pure` excludes every `Swap`); implemented
            // faithfully anyway rather than panicking, per the crate's never-silent discipline.
            Node::Swap {
                src,
                target,
                policy,
            } => {
                let s = self.eval_value_parallel(src, fuel)?;
                let v = as_const(&s)?;
                let result = self.swap.swap(v, target, policy)?;
                tick(fuel)?;
                Ok(Node::Const(result))
            }

            Node::Construct { ctor, args } => {
                let evaluated: Vec<Node> = args
                    .par_iter()
                    .map(|a| self.eval_value_parallel(a, fuel))
                    .collect::<Result<_, _>>()?;
                Ok(Node::Construct {
                    ctor: ctor.clone(),
                    args: evaluated,
                })
            }

            Node::Match {
                scrutinee,
                alts,
                default,
            } => {
                // The scrutinee determines which single arm runs — arms are never spec­ulatively
                // evaluated in parallel (a not-taken arm may be ill-formed/erroring; see module docs).
                let s = self.eval_value_parallel(scrutinee, fuel)?;
                let g = guarantee_of_value(&s)?;
                if g != GuaranteeStrength::Exact {
                    return Err(EvalError::GuaranteeMeetUnsupported { scrutinee: g });
                }
                let body = select_arm(&s, alts, default.as_deref())?;
                tick(fuel)?;
                self.eval_value_parallel(&body, fuel)
            }

            Node::App { func, arg } => {
                // Both positions are independent, already-closed subterms under call-by-value.
                let (f, a) = rayon::join(
                    || self.eval_value_parallel(func, fuel),
                    || self.eval_value_parallel(arg, fuel),
                );
                let (f, a) = (f?, a?);
                match f {
                    Node::Lam { param, body } => {
                        tick(fuel)?;
                        let next = subst(&body, &param, &a);
                        self.eval_value_parallel(&next, fuel)
                    }
                    _ => Err(EvalError::ApplyNonFunction),
                }
            }

            Node::Fix { name, body } => {
                tick(fuel)?;
                let unfolded = subst(body, name, node);
                self.eval_value_parallel(&unfolded, fuel)
            }

            Node::FixGroup { defs, body } => {
                // Mirrors `Interpreter::step`'s `FixGroup` unfold exactly (focus vs continuation).
                let target: Node = match body.as_ref() {
                    Node::Var(v) => defs
                        .iter()
                        .find(|(name, _)| name == v)
                        .map_or_else(|| (**body).clone(), |(_, d)| (**d).clone()),
                    _ => (**body).clone(),
                };
                let unfolded = defs.iter().fold(target, |acc, (name, _)| {
                    let focus = Node::FixGroup {
                        defs: defs.clone(),
                        body: Box::new(Node::Var(name.clone())),
                    };
                    subst(&acc, name, &focus)
                });
                tick(fuel)?;
                self.eval_value_parallel(&unfolded, fuel)
            }
        }
    }
}
