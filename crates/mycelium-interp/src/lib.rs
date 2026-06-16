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
//! # Algebraic data (r3 — RFC-0001 §4.5 / RFC-0011)
//! Two more node families evaluate here, mirroring the L1 evaluator's `try_match` so L1-eval and
//! L0-interp agree (NFR-7):
//!
//! ```text
//!  (E-Con-Arg)    argᵢ ⟶ argᵢ'      (args 0..i are values, i leftmost non-value)
//!                 ─────────────────────────────────────────────────────
//!                 Construct{c, [..,argᵢ,..]} ⟶ Construct{c, [..,argᵢ',..]}
//!
//!  (E-Con-Value)  every arg is a value  ⇒  Construct{c, [v…]} is a NORMAL FORM (a data value)
//!
//!  (E-Match-Scrut) s ⟶ s'
//!                  ─────────────────────────────────────────────
//!                  Match{s, alts, d} ⟶ Match{s', alts, d}
//!
//!  (E-Match-Sel)  s is a value, first-matching alt/default selects body, binders ↦ fields
//!                 ─────────────────────────────────────────────  (scrutinee guarantee Exact)
//!                 Match{s, alts, d} ⟶ body[binders ↦ fields]
//! ```
//!
//! # Functions & recursion (r4 — RFC-0001 r4 / RFC-0007 §4.1)
//! Three more nodes complete L1-in-Core-IR, retiring the elaboration `Residual` entirely. The v0
//! surface is first-order, so an elaborated `Lam` is **closed** (no captured environment) and
//! application is capture-free substitution — the existing `subst` carries it:
//!
//! ```text
//!  (E-Lam)        Lam{x, e} is a NORMAL FORM (a function value)
//!
//!  (E-App-Fun)    f ⟶ f'                  (E-App-Arg)  f value, a ⟶ a'
//!                 ─────────────────────────             ────────────────────────────
//!                 App{f, a} ⟶ App{f', a}               App{f, a} ⟶ App{f, a'}
//!
//!  (E-App-Beta)   ─────────────────────────────────────────────  (a is a value)
//!                 App{Lam{x, e}, a} ⟶ e[x ↦ a]
//!
//!  (E-Fix)        ─────────────────────────────────────────────  (under the fuel clock)
//!                 Fix{f, e} ⟶ e[f ↦ Fix{f, e}]
//! ```
//!
//! `Fix` unfolds by substitution every step, so a non-productive recursion is an explicit
//! [`EvalError::FuelExhausted`], never a hang (RFC-0007 §4.5, CakeML clock); the totality checker
//! gates `matured` (packaging), never meaning. Applying a non-function is an explicit
//! [`EvalError::ApplyNonFunction`]; a program that evaluates to a bare function is
//! [`EvalError::FunctionResult`] (a v0 entry returns a repr/data value, not a function).
//!
//! A `Construct` whose arguments are all values is itself a value (a data value, GHC-Core style); at
//! the `eval` boundary it reads off as a [`mycelium_core::Datum`]. `Match` selects the first
//! matching alternative (constructor arm on `CtorRef` identity; literal arm on `repr+payload`
//! equality), binds its fields left-to-right, and defaults on no match (the checker proves coverage,
//! WF7; a genuine no-match is an explicit [`EvalError::NonExhaustiveMatch`]). **Guarantee meet
//! (RFC-0011 §4.6):** a `Match` result is met with the scrutinee's guarantee — for the *reachable r3
//! fragment* the scrutinee is `Exact`, so the meet is the identity; a **non-`Exact` data scrutinee**
//! is the explicit r3 boundary [`EvalError::GuaranteeMeetUnsupported`] (degrading a precise
//! per-value bound by a composite *summary* would force fabricating a bound — refused, never
//! silent). `Construct` itself takes the meet of its fields' guarantees (in the [`mycelium_core::Datum`]
//! summary).
//!
//! # What is *not* here (by scope)
//! Balanced-ternary **arithmetic** with an integer oracle is **M-111**; the certified binary↔ternary
//! **swap** is **M-120** (this crate ships only the trivial identity swap,
//! [`swap::IdentitySwapEngine`]); the full term language (abstraction/recursion/modules) is a later
//! RFC. Composing an *approximate* input is refused until the ADR-010 bound kernels land (Phase 2 /
//! E2-4).

pub mod prims;
pub mod swap;

use mycelium_core::{Alt, CoreValue, Datum, GuaranteeStrength, Node, Repr, Value, WfError};

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
    /// A `Match` reduced with no alternative matching and no `default` (RFC-0011 §4.3 WF7). The
    /// checker proves coverage above the kernel, so this is unreachable for checked programs — kept
    /// as the explicit never-silent fallback (G2), never a panic or a silent default.
    NonExhaustiveMatch,
    /// A `Construct`/`Match` node was malformed against the data fragment (an arity mismatch the
    /// checker should have caught, a non-saturated constructor — WF6/WF7). Explicit, never a guess.
    DataMalformed {
        /// What was malformed, and why.
        why: String,
    },
    /// The r3 boundary (RFC-0011 §4.6): a `Match` on a **non-`Exact` data scrutinee` would have to
    /// fold the scrutinee's composite *summary* guarantee into the result. Realising that without
    /// fabricating a bound is deferred (the reachable r3 fragment is `Exact`). Refused explicitly,
    /// never silently dropped.
    GuaranteeMeetUnsupported {
        /// The scrutinee's (non-`Exact`) summary guarantee.
        scrutinee: GuaranteeStrength,
    },
    /// [`Interpreter::eval`] was asked for a representation [`Value`] but the program evaluated to a
    /// **data value** ([`mycelium_core::Datum`]). Use [`Interpreter::eval_core`] for the data
    /// fragment. Explicit, so a repr-only caller never silently mishandles a datum.
    DataResult,
    /// An `App` whose function position reduced to a **non-function** value (a `Const`/`Construct`,
    /// not a `Lam`) — RFC-0001 r4. The checker proves applications are well-typed, so this is
    /// unreachable for checked programs; kept as the explicit never-silent fallback (G2).
    ApplyNonFunction,
    /// The program evaluated to a **function value** (a bare `Lam`) — RFC-0001 r4. A v0 entry returns
    /// a representation or data value, never a function (the first-order surface has no function-typed
    /// results), so this is an explicit refusal rather than a silent or partial observable.
    FunctionResult,
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
            EvalError::NonExhaustiveMatch => write!(
                f,
                "match had no matching alternative and no default (WF7 — the checker requires \
                 coverage)"
            ),
            EvalError::DataMalformed { why } => write!(f, "malformed data node: {why}"),
            EvalError::GuaranteeMeetUnsupported { scrutinee } => write!(
                f,
                "match on a non-Exact data scrutinee ({scrutinee:?}): the guarantee-meet through \
                 Match is deferred in r3 (RFC-0011 §4.6) — the reachable fragment is Exact"
            ),
            EvalError::DataResult => write!(
                f,
                "the program evaluated to a data value; use eval_core for the data fragment"
            ),
            EvalError::ApplyNonFunction => write!(
                f,
                "applied a non-function value (the checker should have refused this application)"
            ),
            EvalError::FunctionResult => write!(
                f,
                "the program evaluated to a function value (a v0 entry returns a repr/data value)"
            ),
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

            Node::Construct { ctor, args } => {
                // (E-Con-Arg): reduce the leftmost non-value argument, if any.
                for (i, arg) in args.iter().enumerate() {
                    if let Step::Next(arg2) = self.step(arg)? {
                        let mut next = args.clone();
                        next[i] = *arg2;
                        return Ok(Step::Next(Box::new(Node::Construct {
                            ctor: ctor.clone(),
                            args: next,
                        })));
                    }
                }
                // (E-Con-Value): all arguments are values → this Construct is a normal form.
                Ok(Step::Value)
            }

            Node::Match {
                scrutinee,
                alts,
                default,
            } => match self.step(scrutinee)? {
                // (E-Match-Scrut): reduce the scrutinee to a value first.
                Step::Next(s2) => Ok(Step::Next(Box::new(Node::Match {
                    scrutinee: s2,
                    alts: alts.clone(),
                    default: default.clone(),
                }))),
                // (E-Match-Sel): the scrutinee is a value → select the arm and meet the guarantee.
                Step::Value => {
                    // The Match result is met with the scrutinee's guarantee (RFC-0011 §4.6). For the
                    // reachable r3 fragment the scrutinee is Exact, so the meet is the identity; a
                    // non-Exact data scrutinee is the explicit r3 boundary (never a fabricated bound).
                    let g = guarantee_of_value(scrutinee)?;
                    if g != GuaranteeStrength::Exact {
                        return Err(EvalError::GuaranteeMeetUnsupported { scrutinee: g });
                    }
                    let body = select_arm(scrutinee, alts, default.as_deref())?;
                    Ok(Step::Next(Box::new(body)))
                }
            },

            // (E-Lam): a lambda abstraction is a normal form (a function value).
            Node::Lam { .. } => Ok(Step::Value),

            Node::App { func, arg } => match self.step(func)? {
                // (E-App-Fun): reduce the function position to a value first.
                Step::Next(f2) => Ok(Step::Next(Box::new(Node::App {
                    func: f2,
                    arg: arg.clone(),
                }))),
                Step::Value => match self.step(arg)? {
                    // (E-App-Arg): then reduce the argument (call-by-value).
                    Step::Next(a2) => Ok(Step::Next(Box::new(Node::App {
                        func: func.clone(),
                        arg: a2,
                    }))),
                    // (E-App-Beta): both are values → β-reduce. The function must be a Lam (the
                    // checker proves this); applying any other value is an explicit refusal.
                    Step::Value => match func.as_ref() {
                        Node::Lam { param, body } => {
                            Ok(Step::Next(Box::new(subst(body, param, arg))))
                        }
                        _ => Err(EvalError::ApplyNonFunction),
                    },
                },
            },

            // (E-Fix): unfold by substitution under the fuel clock — Fix(f, e) ⟶ e[f ↦ Fix(f, e)].
            // Always a redex; a non-productive recursion exhausts fuel explicitly, never hangs
            // (RFC-0007 §4.5, CakeML clock).
            Node::Fix { name, body } => {
                let unfolded = subst(body, name, node);
                Ok(Step::Next(Box::new(unfolded)))
            }
        }
    }

    /// Evaluate `node` to a **representation** value by iterating [`step`](Self::step) to a normal
    /// form. Returns the resulting [`Value`], or an [`EvalError`] (including
    /// [`EvalError::FuelExhausted`] if the budget is exceeded, or [`EvalError::DataResult`] if the
    /// program evaluates to a data value — use [`eval_core`](Self::eval_core) for the data fragment).
    pub fn eval(&self, node: &Node) -> Result<Value, EvalError> {
        match self.eval_core(node)? {
            CoreValue::Repr(v) => Ok(v),
            CoreValue::Data(_) => Err(EvalError::DataResult),
        }
    }

    /// Evaluate `node` to a [`CoreValue`] — a representation value **or** a data value (the r3 data
    /// fragment, RFC-0011). Iterates [`step`](Self::step) to a normal form and reads it off: a
    /// `Const` is a representation value; a saturated `Construct` of values is a [`Datum`] (its
    /// meet-summary guarantee computed from its fields). This is the path the M-210 differential
    /// runs for matching/data, against the L1 evaluator (NFR-7).
    pub fn eval_core(&self, node: &Node) -> Result<CoreValue, EvalError> {
        let mut current = node.clone();
        let mut fuel = self.fuel;
        loop {
            match self.step(&current)? {
                Step::Value => return node_to_core_value(&current),
                Step::Next(next) => {
                    fuel = fuel.checked_sub(1).ok_or(EvalError::FuelExhausted)?;
                    current = *next;
                }
            }
        }
    }
}

/// Read a normal-form node off as a [`CoreValue`]: a `Const` is a representation value; a saturated
/// `Construct` of values is a [`Datum`] (the meet-summary computed by [`Datum::new`]). Any other
/// node is not a normal form — an explicit error, never a silent default.
fn node_to_core_value(node: &Node) -> Result<CoreValue, EvalError> {
    match node {
        Node::Const(v) => Ok(CoreValue::Repr(v.clone())),
        Node::Construct { ctor, args } => {
            let fields = args
                .iter()
                .map(node_to_core_value)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(CoreValue::Data(Datum::new(ctor.clone(), fields)))
        }
        Node::Var(x) => Err(EvalError::FreeVariable(x.clone())),
        // A bare Lam normal form is a function value — not an observable v0 result (RFC-0001 r4).
        Node::Lam { .. } => Err(EvalError::FunctionResult),
        _ => Err(EvalError::DataMalformed {
            why: "evaluation ended on a non-normal-form node".to_owned(),
        }),
    }
}

/// The guarantee of a **value** node (a `Const` or a saturated `Construct`): the representation
/// value's own `Meta.guarantee`, or the `meet`-summary of a data value's fields (RFC-0011 §4.6).
fn guarantee_of_value(node: &Node) -> Result<GuaranteeStrength, EvalError> {
    match node {
        Node::Const(v) => Ok(v.meta().guarantee()),
        Node::Construct { args, .. } => {
            let mut g = GuaranteeStrength::Exact;
            for a in args {
                g = g.meet(guarantee_of_value(a)?);
            }
            Ok(g)
        }
        Node::Var(x) => Err(EvalError::FreeVariable(x.clone())),
        _ => Err(EvalError::DataMalformed {
            why: "match scrutinee did not reduce to a value".to_owned(),
        }),
    }
}

/// Select the first-matching `Match` alternative (or the default) and return its binder-substituted
/// body (RFC-0011 §4.6; mirrors the L1 evaluator's `try_match`). A constructor arm matches a
/// `Construct` of the same [`CtorRef`](mycelium_core::CtorRef), binding its fields left-to-right; a
/// literal arm matches a `Const` equal on `repr+payload`. No match + no default is an explicit
/// [`EvalError::NonExhaustiveMatch`].
fn select_arm(scrutinee: &Node, alts: &[Alt], default: Option<&Node>) -> Result<Node, EvalError> {
    for alt in alts {
        match alt {
            Alt::Ctor {
                ctor,
                binders,
                body,
            } => {
                if let Node::Construct { ctor: c2, args } = scrutinee {
                    if c2 == ctor {
                        if binders.len() != args.len() {
                            return Err(EvalError::DataMalformed {
                                why: format!(
                                    "constructor arm binds {} of {} field(s) (WF6/WF7)",
                                    binders.len(),
                                    args.len()
                                ),
                            });
                        }
                        // Bind fields left-to-right; the args are closed values, so substitution is
                        // capture-free (the same property the Const substitution relies on).
                        let mut b = body.clone();
                        for (binder, arg) in binders.iter().zip(args) {
                            b = subst(&b, binder, arg);
                        }
                        return Ok(b);
                    }
                }
            }
            Alt::Lit { value, body } => {
                if let Node::Const(v) = scrutinee {
                    if v.repr() == value.repr() && v.payload() == value.payload() {
                        return Ok(body.clone());
                    }
                }
            }
        }
    }
    match default {
        Some(d) => Ok(d.clone()),
        None => Err(EvalError::NonExhaustiveMatch),
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
        Node::Construct { ctor, args } => Node::Construct {
            ctor: ctor.clone(),
            args: args.iter().map(|a| subst(a, var, value)).collect(),
        },
        Node::Match {
            scrutinee,
            alts,
            default,
        } => Node::Match {
            scrutinee: Box::new(subst(scrutinee, var, value)),
            alts: alts
                .iter()
                .map(|alt| match alt {
                    Alt::Ctor {
                        ctor,
                        binders,
                        body,
                    } => Alt::Ctor {
                        ctor: ctor.clone(),
                        binders: binders.clone(),
                        // Shadowing: an arm binder that re-binds `var` blocks substitution in its body.
                        body: if binders.iter().any(|b| b == var) {
                            body.clone()
                        } else {
                            subst(body, var, value)
                        },
                    },
                    Alt::Lit { value: lit, body } => Alt::Lit {
                        value: lit.clone(),
                        body: subst(body, var, value),
                    },
                })
                .collect(),
            default: default.as_ref().map(|d| Box::new(subst(d, var, value))),
        },
        // r4: a Lam/Fix binder shadows `var` in its body; App substitutes into both positions.
        Node::Lam { param, body } => Node::Lam {
            param: param.clone(),
            body: if param == var {
                body.clone()
            } else {
                Box::new(subst(body, var, value))
            },
        },
        Node::App { func, arg } => Node::App {
            func: Box::new(subst(func, var, value)),
            arg: Box::new(subst(arg, var, value)),
        },
        Node::Fix { name, body } => Node::Fix {
            name: name.clone(),
            body: if name == var {
                body.clone()
            } else {
                Box::new(subst(body, var, value))
            },
        },
    }
}

#[cfg(test)]
mod data_tests {
    //! The r3 data fragment at the L0 boundary (RFC-0011): `Construct`/`Match` evaluation, the
    //! meet-summary guarantee, and the never-silent refusals. These pin the L0 semantics
    //! independently of the L1 elaborator (the L1↔L0 agreement is the M-210 differential).
    use super::*;
    use mycelium_core::{
        Bound, BoundBasis, BoundKind, CtorRef, CtorSpec, DataRegistry, DeclSpec, FieldSpec, Meta,
        NormKind, Payload, Provenance, Repr, Value,
    };
    use std::collections::BTreeMap;

    /// `type Nat = Z | S(Nat)` plus `type Box = Mk(Binary{8})`.
    fn registry() -> DataRegistry {
        let mut m = BTreeMap::new();
        m.insert(
            "Nat".to_owned(),
            DeclSpec {
                ctors: vec![
                    CtorSpec { fields: vec![] },
                    CtorSpec {
                        fields: vec![FieldSpec::Data("Nat".to_owned())],
                    },
                ],
            },
        );
        m.insert(
            "Box".to_owned(),
            DeclSpec {
                ctors: vec![CtorSpec {
                    fields: vec![FieldSpec::Repr(Repr::Binary { width: 8 })],
                }],
            },
        );
        DataRegistry::build(&m).unwrap()
    }

    fn z(reg: &DataRegistry) -> Node {
        Node::Construct {
            ctor: reg.ctor_ref("Nat", 0).unwrap(),
            args: vec![],
        }
    }
    fn s(reg: &DataRegistry, inner: Node) -> Node {
        Node::Construct {
            ctor: reg.ctor_ref("Nat", 1).unwrap(),
            args: vec![inner],
        }
    }
    fn byte(g: GuaranteeStrength) -> Value {
        let meta = if g == GuaranteeStrength::Exact {
            Meta::exact(Provenance::Root)
        } else {
            Meta::new(
                Provenance::Root,
                g,
                Some(Bound {
                    kind: BoundKind::Error {
                        eps: 0.1,
                        norm: NormKind::Linf,
                    },
                    basis: BoundBasis::EmpiricalFit {
                        trials: 1,
                        method: "m".into(),
                    },
                }),
                None,
                None,
                None,
            )
            .unwrap()
        };
        Value::new(
            Repr::Binary { width: 8 },
            Payload::Bits(vec![false; 8]),
            meta,
        )
        .unwrap()
    }

    fn datum(reg: &DataRegistry, ty: &str, i: u32, fields: Vec<CoreValue>) -> CoreValue {
        CoreValue::Data(Datum::new(reg.ctor_ref(ty, i).unwrap(), fields))
    }

    #[test]
    fn construct_evaluates_to_a_datum() {
        let reg = registry();
        let interp = Interpreter::default();
        // S(S(Z)) ⟶ the data value S(S(Z)).
        let node = s(&reg, s(&reg, z(&reg)));
        let v = interp.eval_core(&node).expect("evaluates");
        assert_eq!(
            v,
            datum(
                &reg,
                "Nat",
                1,
                vec![datum(&reg, "Nat", 1, vec![datum(&reg, "Nat", 0, vec![])])]
            )
        );
    }

    #[test]
    fn eval_on_a_data_result_is_an_explicit_refusal() {
        // The repr-only `eval` must refuse a data result explicitly (never silently mishandle it).
        let reg = registry();
        let err = Interpreter::default().eval(&z(&reg)).unwrap_err();
        assert_eq!(err, EvalError::DataResult);
    }

    #[test]
    fn match_selects_the_constructor_arm_and_binds_fields() {
        // match S(Z) { Z => Z, S(m) => m } ⟶ Z   (the S arm binds m = Z).
        let reg = registry();
        let node = Node::Match {
            scrutinee: Box::new(s(&reg, z(&reg))),
            alts: vec![
                Alt::Ctor {
                    ctor: reg.ctor_ref("Nat", 0).unwrap(),
                    binders: vec![],
                    body: z(&reg),
                },
                Alt::Ctor {
                    ctor: reg.ctor_ref("Nat", 1).unwrap(),
                    binders: vec!["m".to_owned()],
                    body: Node::Var("m".to_owned()),
                },
            ],
            default: None,
        };
        let v = Interpreter::default().eval_core(&node).expect("evaluates");
        assert_eq!(v, datum(&reg, "Nat", 0, vec![]));
    }

    #[test]
    fn match_picks_the_first_matching_arm_not_a_later_one() {
        // Mutant-witness: matching Z must take the Z arm, not the S arm or the default.
        let reg = registry();
        let node = Node::Match {
            scrutinee: Box::new(z(&reg)),
            alts: vec![Alt::Ctor {
                ctor: reg.ctor_ref("Nat", 0).unwrap(),
                binders: vec![],
                body: s(&reg, z(&reg)), // Z arm yields S(Z) so we can tell which arm fired
            }],
            default: Some(Box::new(z(&reg))),
        };
        let v = Interpreter::default().eval_core(&node).expect("evaluates");
        assert_eq!(
            v,
            datum(&reg, "Nat", 1, vec![datum(&reg, "Nat", 0, vec![])])
        );
    }

    #[test]
    fn literal_arm_matches_on_repr_and_payload() {
        // match Mk(0b1111_1111) { Mk(b) => match b { 0b1111_1111 => Z, _ => S(Z) } }
        let reg = registry();
        let all_ones = Value::new(
            Repr::Binary { width: 8 },
            Payload::Bits(vec![true; 8]),
            Meta::exact(Provenance::Root),
        )
        .unwrap();
        let inner_match = Node::Match {
            scrutinee: Box::new(Node::Var("b".to_owned())),
            alts: vec![Alt::Lit {
                value: all_ones.clone(),
                body: z(&reg),
            }],
            default: Some(Box::new(s(&reg, z(&reg)))),
        };
        let node = Node::Match {
            scrutinee: Box::new(Node::Construct {
                ctor: reg.ctor_ref("Box", 0).unwrap(),
                args: vec![Node::Const(all_ones)],
            }),
            alts: vec![Alt::Ctor {
                ctor: reg.ctor_ref("Box", 0).unwrap(),
                binders: vec!["b".to_owned()],
                body: inner_match,
            }],
            default: None,
        };
        let v = Interpreter::default().eval_core(&node).expect("evaluates");
        assert_eq!(v, datum(&reg, "Nat", 0, vec![])); // the 0b1111_1111 literal arm fired → Z
    }

    #[test]
    fn no_match_and_no_default_is_an_explicit_non_exhaustive_error() {
        // Mutant-witness: a Match with a non-covering alt set and no default must refuse, not hang
        // or default silently (WF7 is the checker's job, but the kernel never silently assumes it).
        let reg = registry();
        let node = Node::Match {
            scrutinee: Box::new(s(&reg, z(&reg))),
            alts: vec![Alt::Ctor {
                ctor: reg.ctor_ref("Nat", 0).unwrap(), // only Z covered; scrutinee is S(Z)
                binders: vec![],
                body: z(&reg),
            }],
            default: None,
        };
        assert_eq!(
            Interpreter::default().eval_core(&node).unwrap_err(),
            EvalError::NonExhaustiveMatch
        );
    }

    #[test]
    fn construct_summary_guarantee_is_the_meet_of_fields() {
        // Mk(Empirical byte) → an Empirical data value (honesty degrades — RFC-0011 §4.6).
        let reg = registry();
        let node = Node::Construct {
            ctor: reg.ctor_ref("Box", 0).unwrap(),
            args: vec![Node::Const(byte(GuaranteeStrength::Empirical))],
        };
        let v = Interpreter::default().eval_core(&node).expect("evaluates");
        assert_eq!(v.guarantee(), GuaranteeStrength::Empirical);
    }

    #[test]
    fn matching_a_non_exact_data_scrutinee_is_the_explicit_r3_boundary() {
        // match Mk(Empirical) { Mk(b) => b } — the scrutinee's summary is Empirical, so the
        // guarantee-meet through Match is the explicit r3 deferral (never a fabricated bound).
        let reg = registry();
        let node = Node::Match {
            scrutinee: Box::new(Node::Construct {
                ctor: reg.ctor_ref("Box", 0).unwrap(),
                args: vec![Node::Const(byte(GuaranteeStrength::Empirical))],
            }),
            alts: vec![Alt::Ctor {
                ctor: reg.ctor_ref("Box", 0).unwrap(),
                binders: vec!["b".to_owned()],
                body: Node::Var("b".to_owned()),
            }],
            default: None,
        };
        assert_eq!(
            Interpreter::default().eval_core(&node).unwrap_err(),
            EvalError::GuaranteeMeetUnsupported {
                scrutinee: GuaranteeStrength::Empirical
            }
        );
    }

    #[test]
    fn an_aot_lowerable_check_excludes_data_nodes() {
        let reg = registry();
        assert!(!z(&reg).is_aot_lowerable());
        assert!(Node::Const(byte(GuaranteeStrength::Exact)).is_aot_lowerable());
    }

    fn _unused(_: CtorRef) {}
}

#[cfg(test)]
mod r4_tests {
    //! r4 functions + recursion at the L0 boundary (RFC-0001 r4): β-reduction, Fix unfolding under
    //! the fuel clock, and the never-silent refusals. Pins the L0 semantics independently of the
    //! elaborator (the L1↔L0 agreement is the M-210 differential).
    use super::*;
    use mycelium_core::{
        CtorSpec, DataRegistry, DeclSpec, FieldSpec, Meta, Payload, Provenance, Repr, Value,
    };
    use std::collections::BTreeMap;

    fn nat() -> DataRegistry {
        let mut m = BTreeMap::new();
        m.insert(
            "Nat".to_owned(),
            DeclSpec {
                ctors: vec![
                    CtorSpec { fields: vec![] },
                    CtorSpec {
                        fields: vec![FieldSpec::Data("Nat".to_owned())],
                    },
                ],
            },
        );
        DataRegistry::build(&m).unwrap()
    }
    fn z(r: &DataRegistry) -> Node {
        Node::Construct {
            ctor: r.ctor_ref("Nat", 0).unwrap(),
            args: vec![],
        }
    }
    fn s(r: &DataRegistry, n: Node) -> Node {
        Node::Construct {
            ctor: r.ctor_ref("Nat", 1).unwrap(),
            args: vec![n],
        }
    }
    fn byte(bits: [bool; 8]) -> Node {
        Node::Const(
            Value::new(
                Repr::Binary { width: 8 },
                Payload::Bits(bits.to_vec()),
                Meta::exact(Provenance::Root),
            )
            .unwrap(),
        )
    }

    #[test]
    fn beta_reduction_applies_a_closed_lambda() {
        // (λx. not(x)) 0b0000_1111  ⟶  not(0b0000_1111) = 0b1111_0000
        let lam = Node::Lam {
            param: "x".into(),
            body: Box::new(Node::Op {
                prim: "bit.not".into(),
                args: vec![Node::Var("x".into())],
            }),
        };
        let app = Node::App {
            func: Box::new(lam),
            arg: Box::new(byte([false, false, false, false, true, true, true, true])),
        };
        let v = Interpreter::default().eval(&app).expect("runs");
        assert_eq!(
            v.payload(),
            &Payload::Bits(vec![true, true, true, true, false, false, false, false])
        );
    }

    #[test]
    fn curried_application_reduces_left_to_right() {
        // (λx. λy. xor(x, y)) a b
        let lam = Node::Lam {
            param: "x".into(),
            body: Box::new(Node::Lam {
                param: "y".into(),
                body: Box::new(Node::Op {
                    prim: "bit.xor".into(),
                    args: vec![Node::Var("x".into()), Node::Var("y".into())],
                }),
            }),
        };
        let app = Node::App {
            func: Box::new(Node::App {
                func: Box::new(lam),
                arg: Box::new(byte([true, true, true, true, false, false, false, false])),
            }),
            arg: Box::new(byte([false, false, false, false, true, true, true, true])),
        };
        let v = Interpreter::default().eval(&app).expect("runs");
        assert_eq!(v.payload(), &Payload::Bits(vec![true; 8])); // xor of disjoint halves = all ones
    }

    /// `drop_ = Fix(f, λn. match n { Z => Z, S(m) => f m })` — structural recursion to Z.
    fn drop_(r: &DataRegistry) -> Node {
        Node::Fix {
            name: "f".into(),
            body: Box::new(Node::Lam {
                param: "n".into(),
                body: Box::new(Node::Match {
                    scrutinee: Box::new(Node::Var("n".into())),
                    alts: vec![
                        Alt::Ctor {
                            ctor: r.ctor_ref("Nat", 0).unwrap(),
                            binders: vec![],
                            body: z(r),
                        },
                        Alt::Ctor {
                            ctor: r.ctor_ref("Nat", 1).unwrap(),
                            binders: vec!["m".into()],
                            body: Node::App {
                                func: Box::new(Node::Var("f".into())),
                                arg: Box::new(Node::Var("m".into())),
                            },
                        },
                    ],
                    default: None,
                }),
            }),
        }
    }

    #[test]
    fn fix_drives_structural_recursion_to_a_value() {
        // drop_(S(S(S(Z)))) ⟶ Z
        let r = nat();
        let app = Node::App {
            func: Box::new(drop_(&r)),
            arg: Box::new(s(&r, s(&r, s(&r, z(&r))))),
        };
        let v = Interpreter::default().eval_core(&app).expect("terminates");
        assert_eq!(v.as_data().expect("data").fields().len(), 0, "Z");
    }

    #[test]
    fn an_unproductive_fix_exhausts_fuel_explicitly() {
        // Fix(f, f) loops; the fuel clock makes it an explicit refusal, never a hang.
        let spin = Node::Fix {
            name: "f".into(),
            body: Box::new(Node::Var("f".into())),
        };
        let err = Interpreter::default()
            .with_fuel(100)
            .eval_core(&spin)
            .unwrap_err();
        assert_eq!(err, EvalError::FuelExhausted);
    }

    #[test]
    fn applying_a_non_function_is_an_explicit_refusal() {
        // (0b…)(0b…) — applying a representation value is a type error the checker would catch.
        let app = Node::App {
            func: Box::new(byte([false; 8])),
            arg: Box::new(byte([true; 8])),
        };
        assert_eq!(
            Interpreter::default().eval_core(&app).unwrap_err(),
            EvalError::ApplyNonFunction
        );
    }

    #[test]
    fn a_function_result_is_an_explicit_refusal() {
        // A program that evaluates to a bare lambda is not an observable v0 result.
        let lam = Node::Lam {
            param: "x".into(),
            body: Box::new(Node::Var("x".into())),
        };
        assert_eq!(
            Interpreter::default().eval_core(&lam).unwrap_err(),
            EvalError::FunctionResult
        );
    }

    #[test]
    fn lam_app_fix_are_not_aot_lowerable() {
        let r = nat();
        assert!(!drop_(&r).is_aot_lowerable());
    }
}
