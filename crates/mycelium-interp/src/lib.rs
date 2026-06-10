//! `mycelium-interp` — the **reference interpreter**: the trusted, executable small-step semantics
//! for the Core IR (M-110; RFC-0004 §2; ADR-009; NFR-7). It is the *meaning* of a program — the AOT
//! path (M-150/M-151) is differential-tested against it, never the other way round.
//!
//! # Small-step operational semantics (closes SPEC §10.3)
//!
//! Programs are **closed** Core IR [`Node`]s (RFC-0001 §4.5). The values (normal forms) are the
//! constants, `Const(v)`. Evaluation is **call-by-value** and proceeds by substitution; we write
//! `e ⟶ e'` for one step and `e[x ↦ v]` for capture-avoiding substitution (trivial here: the only
//! substituends are closed `Const` values, so there are no free variables to capture).
//!
//! ```text
//!  (E-Let-Step)   bound ⟶ bound'
//!                 ───────────────────────────────────────────────
//!                 Let{x, bound, body} ⟶ Let{x, bound', body}
//!
//!  (E-Let-Bind)   ───────────────────────────────────────────────         (bound is a value)
//!                 Let{x, Const(v), body} ⟶ body[x ↦ Const(v)]
//!
//!  (E-Op-Arg)     argᵢ ⟶ argᵢ'         (args 0..i are values, i leftmost non-value)
//!                 ───────────────────────────────────────────────
//!                 Op{p, [..,argᵢ,..]} ⟶ Op{p, [..,argᵢ',..]}
//!
//!  (E-Op-Apply)   all args are Const(vⱼ)        δ(p, [vⱼ]) = Const(r)
//!                 ───────────────────────────────────────────────
//!                 Op{p, [Const(vⱼ)]} ⟶ Const(r)
//!
//!  (E-Swap-Arg)   src ⟶ src'
//!                 ───────────────────────────────────────────────
//!                 Swap{src, t, π} ⟶ Swap{src', t, π}
//!
//!  (E-Swap-Apply) ───────────────────────────────────────────────       σ(v, t, π) = Const(r)
//!                 Swap{Const(v), t, π} ⟶ Const(r)
//! ```
//!
//! `δ` is the primitive-operator semantics ([`prims`]); `σ` is the swap semantics ([`swap`]). Both
//! thread metadata **honestly**: an `Op`/`Swap` result's guarantee is the `meet` of its inputs and
//! the operation's own intrinsic strength (RFC-0001 §4.7, via `GuaranteeStrength::propagate`), and
//! its provenance is `Derived{ op, inputs }` over content hashes (RFC-0001 §4.6). A `Var` that is
//! free (an open term) is **stuck** — an explicit [`EvalError::FreeVariable`], not a silent default.
//!
//! # What is *not* here (by scope)
//! Balanced-ternary **arithmetic** with an integer oracle is **M-111**; the certified binary↔ternary
//! **swap** is **M-120** (this crate ships only the trivial identity swap,
//! [`swap::IdentitySwapEngine`]); the full term language (abstraction/recursion/modules) is a later
//! RFC. Composing an *approximate* input is refused until the ADR-010 bound kernels land (Phase 2 /
//! E2-4).

pub mod prims;
pub mod swap;

use mycelium_core::{Node, Repr, Value, WfError};

pub use prims::PrimRegistry;
pub use swap::{IdentitySwapEngine, SwapEngine};

/// The result of one small-step attempt on a node.
#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    /// The node is already a value (`Const`) — no redex.
    Value,
    /// The node reduced by one step to this successor. Boxed because a [`Node`] embeds a whole
    /// [`Value`] and would otherwise dwarf the `Value` variant.
    Next(Box<Node>),
}

/// Why evaluation could not proceed (always explicit — the interpreter is never silent; SC-3/G2).
#[derive(Debug, Clone, PartialEq)]
pub enum EvalError {
    /// A free variable was encountered (the program is not closed).
    FreeVariable(String),
    /// No primitive is registered under this name.
    UnknownPrim(String),
    /// A primitive was applied to the wrong arity/paradigm/width.
    PrimType {
        /// The primitive name.
        prim: String,
        /// A human-readable explanation.
        why: String,
    },
    /// A primitive would have to compose an approximate input for which it has **no defined
    /// ε-propagation rule** (the logical `bit.*` ops; `trit.mul` pending the Dense magnitudes; or an
    /// input carrying a non-`Error` bound). The additive arithmetic *does* compose now via the
    /// verified-numerics kernel (M-204; ADR-010); this is refused rather than fabricating a bound.
    ApproxCompositionUnsupported {
        /// The primitive name.
        prim: String,
    },
    /// The swap engine does not support this `(from → to)` conversion (the certified cross-paradigm
    /// swap is M-120).
    UnsupportedSwap {
        /// Source representation.
        from: Repr,
        /// Target representation.
        to: Repr,
    },
    /// A fixed-width arithmetic result fell outside the representable range — explicit, never a
    /// silent wrap (SC-3; balanced-ternary range, `binary-ternary.md` §1).
    Overflow {
        /// The primitive name.
        prim: String,
    },
    /// Evaluation exceeded its step budget (a non-termination guard).
    FuelExhausted,
    /// A swap engine reported a failure (e.g. an illegal pair or an out-of-range conversion). The
    /// message comes from the engine; it is always explicit, never a silent coercion.
    Swap(String),
    /// A constructed result violated a Core IR well-formedness invariant (RFC-0001 §4.3/§4.5).
    Wf(WfError),
}

impl core::fmt::Display for EvalError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            EvalError::FreeVariable(x) => write!(f, "free variable: {x}"),
            EvalError::UnknownPrim(p) => write!(f, "unknown primitive: {p}"),
            EvalError::PrimType { prim, why } => write!(f, "type error in {prim}: {why}"),
            EvalError::ApproxCompositionUnsupported { prim } => write!(
                f,
                "{prim}: no defined ε-propagation rule for an approximate input (ADR-010/M-204)"
            ),
            EvalError::UnsupportedSwap { from, to } => {
                write!(
                    f,
                    "unsupported swap: {from:?} → {to:?} (certified swap is M-120)"
                )
            }
            EvalError::Overflow { prim } => {
                write!(
                    f,
                    "{prim}: fixed-width arithmetic overflow (result out of range)"
                )
            }
            EvalError::FuelExhausted => write!(f, "evaluation exceeded its step budget"),
            EvalError::Swap(msg) => write!(f, "swap failed: {msg}"),
            EvalError::Wf(e) => write!(f, "well-formedness violation: {e}"),
        }
    }
}

impl std::error::Error for EvalError {}

/// Default step budget — generous for the non-recursive core language (it always terminates), a
/// guard against pathological inputs.
const DEFAULT_FUEL: u64 = 1_000_000;

/// The reference interpreter: a primitive registry + a swap engine. [`Interpreter::default`] wires
/// the exact built-in prims and the identity swap engine.
pub struct Interpreter {
    prims: PrimRegistry,
    swap: Box<dyn SwapEngine>,
    fuel: u64,
}

impl Default for Interpreter {
    fn default() -> Self {
        Interpreter {
            prims: PrimRegistry::with_builtins(),
            swap: Box::new(IdentitySwapEngine),
            fuel: DEFAULT_FUEL,
        }
    }
}

impl Interpreter {
    /// Build an interpreter with a custom prim registry and swap engine (e.g. M-120's certified
    /// swap, or M-111's arithmetic prims).
    #[must_use]
    pub fn new(prims: PrimRegistry, swap: Box<dyn SwapEngine>) -> Self {
        Interpreter {
            prims,
            swap,
            fuel: DEFAULT_FUEL,
        }
    }

    /// Override the step budget.
    #[must_use]
    pub fn with_fuel(mut self, fuel: u64) -> Self {
        self.fuel = fuel;
        self
    }

    /// The registered primitive names (for tooling/EXPLAIN).
    #[must_use]
    pub fn prim_names(&self) -> Vec<&str> {
        self.prims.names()
    }

    /// Perform exactly one small-step reduction on `node` (the `⟶` relation above).
    ///
    /// Returns [`Step::Value`] if `node` is already a `Const`, or [`Step::Next`] with the reduced
    /// term. Errors are explicit (free variable, unknown/ill-typed prim, unsupported swap).
    pub fn step(&self, node: &Node) -> Result<Step, EvalError> {
        match node {
            Node::Const(_) => Ok(Step::Value),

            Node::Var(x) => Err(EvalError::FreeVariable(x.clone())),

            Node::Let { id, bound, body } => match self.step(bound)? {
                // (E-Let-Bind): bound is a value → substitute it into the body.
                Step::Value => Ok(Step::Next(Box::new(subst(body, id, bound)))),
                // (E-Let-Step): reduce the bound expression first (call-by-value).
                Step::Next(bound2) => Ok(Step::Next(Box::new(Node::Let {
                    id: id.clone(),
                    bound: bound2,
                    body: body.clone(),
                }))),
            },

            Node::Op { prim, args } => {
                // (E-Op-Arg): reduce the leftmost non-value argument, if any.
                for (i, arg) in args.iter().enumerate() {
                    if let Step::Next(arg2) = self.step(arg)? {
                        let mut next = args.clone();
                        next[i] = *arg2;
                        return Ok(Step::Next(Box::new(Node::Op {
                            prim: prim.clone(),
                            args: next,
                        })));
                    }
                }
                // (E-Op-Apply): all arguments are values → apply δ.
                let values = collect_values(args)?;
                let f = self
                    .prims
                    .get(prim)
                    .ok_or_else(|| EvalError::UnknownPrim(prim.clone()))?;
                let result = f(prim, &values)?;
                Ok(Step::Next(Box::new(Node::Const(result))))
            }

            Node::Swap {
                src,
                target,
                policy,
            } => match self.step(src)? {
                // (E-Swap-Apply): source is a value → apply σ.
                Step::Value => {
                    let v = as_const(src)?;
                    let result = self.swap.swap(v, target, policy)?;
                    Ok(Step::Next(Box::new(Node::Const(result))))
                }
                // (E-Swap-Arg): reduce the source first.
                Step::Next(src2) => Ok(Step::Next(Box::new(Node::Swap {
                    src: src2,
                    target: target.clone(),
                    policy: policy.clone(),
                }))),
            },
        }
    }

    /// Evaluate `node` to a value by iterating [`step`](Self::step) to a normal form. Returns the
    /// resulting [`Value`], or an [`EvalError`] (including [`EvalError::FuelExhausted`] if the step
    /// budget is exceeded).
    pub fn eval(&self, node: &Node) -> Result<Value, EvalError> {
        let mut current = node.clone();
        let mut fuel = self.fuel;
        loop {
            match self.step(&current)? {
                Step::Value => return Ok(as_const(&current)?.clone()),
                Step::Next(next) => {
                    fuel = fuel.checked_sub(1).ok_or(EvalError::FuelExhausted)?;
                    current = *next;
                }
            }
        }
    }
}

/// Extract the `Value` from a `Const` node (an internal invariant when [`Step::Value`] was reported).
fn as_const(node: &Node) -> Result<&Value, EvalError> {
    match node {
        Node::Const(v) => Ok(v),
        Node::Var(x) => Err(EvalError::FreeVariable(x.clone())),
        // Unreachable when called after a `Step::Value`; treated as "stuck" defensively.
        _ => Err(EvalError::FreeVariable(
            "<non-value normal form>".to_owned(),
        )),
    }
}

/// Collect a list of nodes known to be values into their `Value`s.
fn collect_values(args: &[Node]) -> Result<Vec<&Value>, EvalError> {
    args.iter().map(as_const).collect()
}

/// Capture-avoiding substitution `node[var ↦ value]`. Substituends are closed `Const` values, so no
/// renaming is ever needed; substitution stops under a binder that shadows `var`.
fn subst(node: &Node, var: &str, value: &Node) -> Node {
    match node {
        Node::Const(_) => node.clone(),
        Node::Var(x) => {
            if x == var {
                value.clone()
            } else {
                node.clone()
            }
        }
        Node::Let { id, bound, body } => Node::Let {
            id: id.clone(),
            bound: Box::new(subst(bound, var, value)),
            // Shadowing: a re-binding of `var` blocks substitution in the body.
            body: if id == var {
                body.clone()
            } else {
                Box::new(subst(body, var, value))
            },
        },
        Node::Op { prim, args } => Node::Op {
            prim: prim.clone(),
            args: args.iter().map(|a| subst(a, var, value)).collect(),
        },
        Node::Swap {
            src,
            target,
            policy,
        } => Node::Swap {
            src: Box::new(subst(src, var, value)),
            target: target.clone(),
            policy: policy.clone(),
        },
    }
}
