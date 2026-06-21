//! The **L1 fuel-guarded evaluator** (RFC-0007 §4.6): a big-step environment machine mirroring
//! the M-110 reference interpreter's contract — explicit errors only, a step budget instead of a
//! termination assumption (CakeML-style clocked semantics, T3.4), and the *same* trusted
//! primitive registry and swap engine the L0 interpreter and the AOT path dispatch through, so
//! "two execution paths" can never mean "two semantics" (NFR-7).
//!
//! Programs **inside** the evaluation-complete fragment also elaborate to L0
//! ([`crate::elab::elaborate`]) and must agree with this evaluator on the observable
//! (`repr + payload + guarantee`) — the §4.6 differential obligation, validated through the M-210
//! shared checker (`tests/differential.rs`). Programs **outside** the fragment (recursion, match,
//! data values, dynamic guarantee indices) run *only* here.
//!
//! Honesty:
//! - exhausting the step budget is an explicit [`L1Error::FuelExhausted`], never a hang — and
//!   "checked total" means precisely "terminates for every sufficiently large fuel" (§4.5);
//! - a guarantee index `@ g` is checked **dynamically against `Meta`** (stage 0, RFC-0007 §4.3):
//!   asserting `@ g` on a value whose tag is weaker than `g` is an explicit
//!   [`L1Error::GuaranteeTooWeak`] — the assertion never upgrades the tag (VR-5), and a passing
//!   check leaves the value's own (possibly stronger) tag untouched;
//! - states the typechecker proves unreachable still fail as explicit [`L1Error::Stuck`] errors,
//!   never panics or defaults (S5/G2).

use mycelium_cert::BinaryTernarySwapEngine;
use mycelium_core::{CoreValue, DataRegistry, Datum, GuaranteeStrength, Value};
use mycelium_interp::{EvalError as KernelError, PrimRegistry, SwapEngine};

use crate::ast::{Expr, Literal, Pattern, Strength};
use crate::checkty::{prim_kernel_name, Env};
use crate::elab::{lit_value, policy_name_ref, type_repr, ElabError};

/// An L1 runtime value: an L0 representation value, or a constructed datum. Data values are
/// immutable and acyclic by construction — a `Construct` value can only contain values that
/// existed before it (RFC-0007 §4.7).
#[derive(Debug, Clone, PartialEq)]
pub enum L1Value {
    /// An L0 value (`repr + payload + Meta`).
    Repr(Value),
    /// A saturated constructor application (W6).
    Data {
        /// The data type's name (v0 keys the registry by name; RFC-0007 §4.2).
        ty: String,
        /// The constructor's name.
        ctor: String,
        /// The constructor's field values, in declaration order.
        fields: Vec<L1Value>,
    },
}

impl L1Value {
    /// The underlying L0 value, if this is a representation value.
    #[must_use]
    pub fn as_repr(&self) -> Option<&Value> {
        match self {
            L1Value::Repr(v) => Some(v),
            L1Value::Data { .. } => None,
        }
    }

    /// Project this L1 value onto the L0 [`CoreValue`] domain, resolving each constructor's
    /// name-keyed identity (`ty`/`ctor`) to its content-addressed `#T#i` [`mycelium_core::CtorRef`]
    /// through `registry` — the **same** registry the elaborator built (RFC-0011 §4.3). This is the
    /// bridge that makes the M-210 differential meaningful on the data fragment: an L1-eval result
    /// and an elaborate→L0-interp result become comparable *as the same L0 value* (NFR-7). The data
    /// guarantee is the meet-summary [`Datum::new`] computes from the fields, identical on both
    /// paths. Returns `None` if a constructor is not in the registry (outside the r3 fragment).
    #[must_use]
    pub fn to_core(&self, env: &crate::checkty::Env, registry: &DataRegistry) -> Option<CoreValue> {
        match self {
            L1Value::Repr(v) => Some(CoreValue::Repr(v.clone())),
            L1Value::Data { ty, ctor, fields } => {
                let decl = env.types.get(ty)?;
                let index = decl.ctors.iter().position(|c| c.name == *ctor)?;
                let ctor_ref = registry.ctor_ref(ty, u32::try_from(index).ok()?)?;
                let core_fields = fields
                    .iter()
                    .map(|f| f.to_core(env, registry))
                    .collect::<Option<Vec<_>>>()?;
                Some(CoreValue::Data(Datum::new(ctor_ref, core_fields)))
            }
        }
    }
}

/// Why L1 evaluation could not produce a value — always explicit (S5/G2).
#[derive(Debug, Clone, PartialEq)]
pub enum L1Error {
    /// The step budget ran out — the non-termination guard (RFC-0007 §4.5/§4.6).
    FuelExhausted,
    /// The recursion-depth budget ran out. This is a **host-resource guard**, distinct from the
    /// semantic clock: the big-step machine recurses on the host stack, so unbounded call depth
    /// must refuse explicitly rather than overflow it. Raise with [`Evaluator::with_depth`].
    DepthExceeded {
        /// The configured depth budget.
        limit: u32,
    },
    /// The trusted kernel (prim registry / swap engine) refused — the refusal is forwarded
    /// verbatim, never softened.
    Kernel(KernelError),
    /// A dynamic guarantee-index check failed: the asserted `@ g` is *stronger* than the value's
    /// actual tag — an assertion may only weaken, never upgrade (VR-5; RFC-0007 §4.3).
    GuaranteeTooWeak {
        /// The function in which the assertion appears.
        site: String,
        /// The asserted strength.
        asserted: Strength,
        /// The value's actual (weaker) strength.
        actual: GuaranteeStrength,
    },
    /// A construct the v0 evaluator does not support (`wild`, `spore`, bare-integer/list
    /// literals…) — refused with its reason, mirroring the typechecker's refusals.
    Unsupported {
        /// The function in which the construct appears.
        site: String,
        /// What was refused, and why.
        what: String,
    },
    /// An evaluation state the typechecker proves unreachable (unknown name, non-exhaustive
    /// match, arity mismatch…). Reported explicitly rather than panicking, so a checker bug can
    /// never become silent misbehavior.
    Stuck {
        /// The function in which evaluation got stuck.
        site: String,
        /// What went wrong.
        why: String,
    },
}

impl core::fmt::Display for L1Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            L1Error::FuelExhausted => write!(f, "evaluation exceeded its step budget"),
            L1Error::DepthExceeded { limit } => write!(
                f,
                "evaluation exceeded its recursion-depth budget ({limit}) — a host-stack guard, \
                 explicit by design"
            ),
            L1Error::Kernel(e) => write!(f, "kernel refusal: {e}"),
            L1Error::GuaranteeTooWeak {
                site,
                asserted,
                actual,
            } => write!(
                f,
                "in `{site}`: asserted `@ {asserted:?}` but the value's tag is {actual:?} — an \
                 annotation may only weaken (VR-5)"
            ),
            L1Error::Unsupported { site, what } => write!(f, "in `{site}`: {what}"),
            L1Error::Stuck { site, why } => write!(
                f,
                "in `{site}`: stuck — {why} (the typechecker should have refused this program)"
            ),
        }
    }
}

impl std::error::Error for L1Error {}

impl From<KernelError> for L1Error {
    fn from(e: KernelError) -> Self {
        L1Error::Kernel(e)
    }
}

/// The surface strength keyword's kernel lattice point.
#[must_use]
pub fn strength_of(s: Strength) -> GuaranteeStrength {
    match s {
        Strength::Exact => GuaranteeStrength::Exact,
        Strength::Proven => GuaranteeStrength::Proven,
        Strength::Empirical => GuaranteeStrength::Empirical,
        Strength::Declared => GuaranteeStrength::Declared,
    }
}

/// Default step budget — mirrors the reference interpreter's (M-110).
const DEFAULT_FUEL: u64 = 1_000_000;

/// Default recursion-depth budget — conservative enough for an unoptimized (debug) build on a
/// 2 MiB test-thread stack (debug frames are large); deep but terminating programs can raise it
/// via [`Evaluator::with_depth`] when the host stack allows (e.g. a dedicated thread with a
/// larger stack).
const DEFAULT_DEPTH: u32 = 64;

/// The tunable **budgets** of an [`Evaluator`] — the step (`fuel`) and recursion-depth guards — as
/// a single options struct, an alternative to threading the fluent [`Evaluator::with_fuel`] /
/// [`Evaluator::with_depth`] chain. Applied via [`Evaluator::with_opts`]; the fluent setters stay.
///
/// Only the `Copy` budget knobs live here: the *engines* (`PrimRegistry`, `Box<dyn SwapEngine>`)
/// are not part of `EvaluatorOpts` — they are not `Clone`/`Default` and stay set through
/// [`Evaluator::with_engines`], so this struct is a plain, copyable, defaultable bundle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvaluatorOpts {
    /// The step budget (as [`Evaluator::with_fuel`]). [`Default`] is `DEFAULT_FUEL`.
    pub fuel: u64,
    /// The recursion-depth (host-stack) budget (as [`Evaluator::with_depth`]). [`Default`] is
    /// `DEFAULT_DEPTH`.
    pub depth: u32,
}

/// The defaults mirror [`Evaluator::new`] exactly — `DEFAULT_FUEL` / `DEFAULT_DEPTH` — so
/// `Evaluator::new(env).with_opts(EvaluatorOpts::default())` is a no-op (the budgets are unchanged).
impl Default for EvaluatorOpts {
    fn default() -> Self {
        EvaluatorOpts {
            fuel: DEFAULT_FUEL,
            depth: DEFAULT_DEPTH,
        }
    }
}

impl EvaluatorOpts {
    /// Set the step budget (builder-style), leaving `depth` untouched.
    #[must_use]
    pub fn fuel(mut self, fuel: u64) -> Self {
        self.fuel = fuel;
        self
    }

    /// Set the recursion-depth budget (builder-style), leaving `fuel` untouched.
    #[must_use]
    pub fn depth(mut self, depth: u32) -> Self {
        self.depth = depth;
        self
    }
}

/// The L1 evaluator over a checked [`Env`]. Construction wires the same trusted engines the
/// L0 paths use: the built-in prim registry and the certified binary↔ternary swap engine
/// (M-120/M-210) — override with [`Evaluator::with_engines`] for tests or extensions.
pub struct Evaluator<'e> {
    env: &'e Env,
    prims: PrimRegistry,
    swap: Box<dyn SwapEngine>,
    fuel: u64,
    depth: u32,
}

impl<'e> Evaluator<'e> {
    /// An evaluator over `env` with the trusted default engines and the default budgets.
    #[must_use]
    pub fn new(env: &'e Env) -> Self {
        Evaluator {
            env,
            prims: PrimRegistry::with_builtins(),
            swap: Box::new(BinaryTernarySwapEngine),
            fuel: DEFAULT_FUEL,
            depth: DEFAULT_DEPTH,
        }
    }

    /// Replace the prim registry and swap engine.
    #[must_use]
    pub fn with_engines(mut self, prims: PrimRegistry, swap: Box<dyn SwapEngine>) -> Self {
        self.prims = prims;
        self.swap = swap;
        self
    }

    /// Override the step budget.
    #[must_use]
    pub fn with_fuel(mut self, fuel: u64) -> Self {
        self.fuel = fuel;
        self
    }

    /// Override the recursion-depth budget (the host-stack guard).
    #[must_use]
    pub fn with_depth(mut self, depth: u32) -> Self {
        self.depth = depth;
        self
    }

    /// Apply a budget [`EvaluatorOpts`] in one call — equivalent to
    /// `self.with_fuel(opts.fuel).with_depth(opts.depth)`. Additive convenience; the engines are
    /// untouched (configure those with [`Evaluator::with_engines`]).
    #[must_use]
    pub fn with_opts(self, opts: EvaluatorOpts) -> Self {
        self.with_fuel(opts.fuel).with_depth(opts.depth)
    }

    /// Call function `name` with `args`, big-step, under the configured budgets. The result
    /// honors the signature's dynamic guarantee index, if any (RFC-0007 §4.3).
    pub fn call(&self, name: &str, args: Vec<L1Value>) -> Result<L1Value, L1Error> {
        let mut fuel = self.fuel;
        self.invoke(&mut fuel, self.depth, name, args)
    }

    /// One function invocation: bind parameters, evaluate the body, check the return index.
    fn invoke(
        &self,
        fuel: &mut u64,
        depth: u32,
        name: &str,
        args: Vec<L1Value>,
    ) -> Result<L1Value, L1Error> {
        let Some(fd) = self.env.fns.get(name) else {
            return Err(L1Error::Stuck {
                site: name.to_owned(),
                why: format!("unknown function `{name}`"),
            });
        };
        if fd.sig.value_params.len() != args.len() {
            return Err(L1Error::Stuck {
                site: name.to_owned(),
                why: format!(
                    "`{name}` takes {} argument(s), got {}",
                    fd.sig.value_params.len(),
                    args.len()
                ),
            });
        }
        let mut scope: Vec<(String, L1Value)> = fd
            .sig
            .value_params
            .iter()
            .map(|p| p.name.clone())
            .zip(args)
            .collect();
        let result = self.eval(fuel, depth, name, &mut scope, &fd.body)?;
        if let Some(g) = fd.sig.ret.guarantee {
            self.assert_guarantee(name, &result, g)?;
        }
        Ok(result)
    }

    /// Big-step evaluation of `e` under `scope`. Every node costs one unit of fuel, so an
    /// unproductive recursion is an explicit [`L1Error::FuelExhausted`], never a hang.
    ///
    /// **Depth is charged per AST node, not per call frame (A4-03).** `eval` recurses on the host
    /// stack for *every* sub-expression — an operand of an `App`, the bound of a `Let`, an `if`
    /// branch — not only at a function `invoke`. The depth budget is a *host-stack* guard (see
    /// [`L1Error::DepthExceeded`]), so it must count exactly the recursion that consumes host
    /// stack: a deeply **nested expression** (e.g. `not(not(… not(x) …))`) overflows the stack just
    /// as a deep call chain does, and charging only at `invoke` would leave it unguarded. The
    /// honest consequence is that [`DEFAULT_DEPTH`] = 64 is a *nesting* ceiling, not a call-depth
    /// ceiling: an expression whose AST is more than ~64 nodes deep along any single path is
    /// refused with an explicit [`L1Error::DepthExceeded`] even if it makes no recursive call.
    /// This is a deliberate over-approximation in favor of the termination/no-crash guarantee
    /// (S5/G2) — raise the budget via [`Evaluator::with_depth`] on a larger host stack when a
    /// legitimately deep but terminating expression needs it. (`for`-folds walk their spine
    /// iteratively and so are *not* subject to this ceiling per element — see [`Self::eval_for`].)
    fn eval(
        &self,
        fuel: &mut u64,
        depth: u32,
        site: &str,
        scope: &mut Vec<(String, L1Value)>,
        e: &Expr,
    ) -> Result<L1Value, L1Error> {
        *fuel = fuel.checked_sub(1).ok_or(L1Error::FuelExhausted)?;
        // Per-node (not per-call-frame) on purpose: this counts host-stack recursion, which a wide
        // *and* a deep AST both incur. See the method doc for why the per-node charge is the safe
        // choice and what the resulting 64-node nesting ceiling means (A4-03).
        let depth = depth
            .checked_sub(1)
            .ok_or(L1Error::DepthExceeded { limit: self.depth })?;
        match e {
            Expr::Lit(l @ (Literal::Bin(_) | Literal::Trit(_))) => Ok(L1Value::Repr(
                lit_value(site, l).map_err(|e| unsupported(site, &e))?,
            )),
            Expr::Lit(_) => Err(L1Error::Unsupported {
                site: site.to_owned(),
                what: "bare-integer and list literals have no v0 value form (Q6)".to_owned(),
            }),

            Expr::Path(p) => {
                if p.0.len() == 1 {
                    let name = &p.0[0];
                    if let Some((_, v)) = scope.iter().rev().find(|(n, _)| n == name) {
                        return Ok(v.clone());
                    }
                    if let Some((d, i)) = self.env.ctor(name) {
                        if d.ctors[i].fields.is_empty() {
                            return Ok(L1Value::Data {
                                ty: d.name.clone(),
                                ctor: name.clone(),
                                fields: vec![],
                            });
                        }
                    }
                }
                Err(L1Error::Stuck {
                    site: site.to_owned(),
                    why: format!("unresolved name `{}`", p.0.join(".")),
                })
            }

            Expr::Let {
                name,
                ty,
                bound,
                body,
            } => {
                let bv = self.eval(fuel, depth, site, scope, bound)?;
                if let Some(g) = ty.as_ref().and_then(|t| t.guarantee) {
                    self.assert_guarantee(site, &bv, g)?;
                }
                scope.push((name.clone(), bv));
                let r = self.eval(fuel, depth, site, scope, body);
                scope.pop();
                r
            }

            Expr::If { cond, conseq, alt } => {
                let c = self.eval(fuel, depth, site, scope, cond)?;
                match c {
                    L1Value::Data { ref ctor, .. } if ctor == "True" => {
                        self.eval(fuel, depth, site, scope, conseq)
                    }
                    L1Value::Data { ref ctor, .. } if ctor == "False" => {
                        self.eval(fuel, depth, site, scope, alt)
                    }
                    other => Err(L1Error::Stuck {
                        site: site.to_owned(),
                        why: format!("if-condition evaluated to a non-Bool: {other:?}"),
                    }),
                }
            }

            Expr::Match { scrutinee, arms } => {
                self.eval_match(fuel, depth, site, scope, scrutinee, arms)
            }

            Expr::For {
                x,
                xs,
                acc,
                init,
                body,
            } => self.eval_for(fuel, depth, site, scope, x, xs, acc, init, body),

            Expr::Swap {
                value,
                target,
                policy,
            } => {
                let v = self.eval(fuel, depth, site, scope, value)?;
                let Some(src) = v.as_repr() else {
                    return Err(L1Error::Stuck {
                        site: site.to_owned(),
                        why: "swap source is not a representation value".to_owned(),
                    });
                };
                let repr = type_repr(site, target).map_err(|e| unsupported(site, &e))?;
                let out = self.swap.swap(src, &repr, &policy_name_ref(policy))?;
                let out = L1Value::Repr(out);
                if let Some(g) = target.guarantee {
                    self.assert_guarantee(site, &out, g)?;
                }
                Ok(out)
            }

            Expr::WithParadigm { .. } => Err(L1Error::Unsupported {
                site: site.to_owned(),
                what: "internal: a `with paradigm` block reached the evaluator — the ambient \
                       resolution pass strips it (RFC-0012 §4.4)"
                    .to_owned(),
            }),
            Expr::Wild(_) => Err(L1Error::Unsupported {
                site: site.to_owned(),
                what: "`wild` is denied by default (LR-9): no host FFI capability exists in v0"
                    .to_owned(),
            }),
            Expr::Spore(_) => Err(L1Error::Unsupported {
                site: site.to_owned(),
                what: "`spore` is deferred to the reconstruction-manifest work (E2-5/M-260)"
                    .to_owned(),
            }),

            Expr::Ascribe(inner, t) => {
                let v = self.eval(fuel, depth, site, scope, inner)?;
                if let Some(g) = t.guarantee {
                    self.assert_guarantee(site, &v, g)?;
                }
                Ok(v)
            }

            Expr::App { head, args } => self.eval_app(fuel, depth, site, scope, head, args),
        }
    }

    /// Bounded iteration (RFC-0007 §4.8): walk the linearly recursive spine head-to-tail,
    /// folding the accumulator through the body. The walk is **iterative** — a `for` over a long
    /// list costs fuel per element (each body evaluation is clocked) but never host stack, so it
    /// cannot trip the depth guard the way the equivalent hand-written recursion would.
    #[allow(clippy::too_many_arguments)] // the machine threads its budgets + the form's five parts
    fn eval_for(
        &self,
        fuel: &mut u64,
        depth: u32,
        site: &str,
        scope: &mut Vec<(String, L1Value)>,
        x: &str,
        xs: &Expr,
        acc: &str,
        init: &Expr,
        body: &Expr,
    ) -> Result<L1Value, L1Error> {
        let mut spine = self.eval(fuel, depth, site, scope, xs)?;
        let mut accv = self.eval(fuel, depth, site, scope, init)?;
        loop {
            let L1Value::Data { ty, ctor, fields } = spine else {
                return Err(L1Error::Stuck {
                    site: site.to_owned(),
                    why: "`for` spine is not a data value".to_owned(),
                });
            };
            if fields.is_empty() {
                return Ok(accv); // a nil — the spine ends, the fold is the accumulator
            }
            // A cons: exactly one spine field (type == ty) and one element field (checked).
            let Some(d) = self.env.types.get(&ty) else {
                return Err(L1Error::Stuck {
                    site: site.to_owned(),
                    why: format!("`for` over unregistered type `{ty}`"),
                });
            };
            let Some(ci) = d.ctors.iter().position(|c| c.name == ctor) else {
                return Err(L1Error::Stuck {
                    site: site.to_owned(),
                    why: format!("`for` met unknown constructor `{ctor}` of `{ty}`"),
                });
            };
            let mut elem = None;
            let mut rest = None;
            for (f, v) in d.ctors[ci].fields.iter().zip(fields) {
                if matches!(f, crate::checkty::Ty::Data(n) if *n == ty) {
                    rest = Some(v);
                } else {
                    elem = Some(v);
                }
            }
            let (Some(elem), Some(rest)) = (elem, rest) else {
                return Err(L1Error::Stuck {
                    site: site.to_owned(),
                    why: format!(
                        "`{ctor}` is not nil/cons-shaped — the checker should have refused"
                    ),
                });
            };
            // Each element's body evaluation is clocked like any other expression.
            *fuel = fuel.checked_sub(1).ok_or(L1Error::FuelExhausted)?;
            scope.push((x.to_owned(), elem));
            scope.push((acc.to_owned(), accv));
            let next = self.eval(fuel, depth, site, scope, body);
            scope.pop();
            scope.pop();
            accv = next?;
            spine = rest;
        }
    }

    /// The W7 flat-match machine (split out of [`Self::eval`] to keep the recursion frame small —
    /// the depth guard's budget is host stack, so frame size is part of the contract).
    fn eval_match(
        &self,
        fuel: &mut u64,
        depth: u32,
        site: &str,
        scope: &mut Vec<(String, L1Value)>,
        scrutinee: &Expr,
        arms: &[crate::ast::Arm],
    ) -> Result<L1Value, L1Error> {
        let sv = self.eval(fuel, depth, site, scope, scrutinee)?;
        // The checker has verified exhaustiveness, redundancy, types, and arity (W7), so the first
        // arm whose (possibly nested) pattern matches fires. The trailing `Stuck` is unreachable for
        // checked programs but kept as the honest never-silent fallback (G2).
        for arm in arms {
            let mut binds: Vec<(String, L1Value)> = Vec::new();
            if self.try_match(site, &arm.pattern, &sv, &mut binds)? {
                let mark = scope.len();
                scope.extend(binds);
                let r = self.eval(fuel, depth, site, scope, &arm.body);
                scope.truncate(mark);
                return r;
            }
        }
        Err(L1Error::Stuck {
            site: site.to_owned(),
            why: "no arm matched the scrutinee (W7 — the checker requires coverage)".to_owned(),
        })
    }

    /// Try to match `val` against `pat`, accumulating the pattern's binders into `binds`
    /// (left-to-right, recursively for nested patterns). Returns whether it matched; on a partial
    /// nested failure the caller discards `binds`, so no rollback is needed. The
    /// constructor/literal/binder resolution mirrors the typechecker's `check_pattern` exactly, so a
    /// checked program never gets stuck (RFC-0007 §4.7).
    fn try_match(
        &self,
        site: &str,
        pat: &Pattern,
        val: &L1Value,
        binds: &mut Vec<(String, L1Value)>,
    ) -> Result<bool, L1Error> {
        match pat {
            Pattern::Wildcard => Ok(true),
            // A bare name is a nullary-constructor alternative iff it names one of the value's data
            // type's constructors; otherwise it binds the whole value.
            Pattern::Ident(n) => match val {
                L1Value::Data { ty, ctor, .. }
                    if self
                        .env
                        .types
                        .get(ty)
                        .is_some_and(|d| d.ctors.iter().any(|c| c.name == *n)) =>
                {
                    Ok(ctor == n)
                }
                _ => {
                    binds.push((n.clone(), val.clone()));
                    Ok(true)
                }
            },
            Pattern::Ctor(n, subs) => match val {
                L1Value::Data { ctor, fields, .. } => {
                    if ctor != n {
                        return Ok(false);
                    }
                    for (sub, fv) in subs.iter().zip(fields) {
                        if !self.try_match(site, sub, fv, binds)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }
                L1Value::Repr(_) => Ok(false),
            },
            Pattern::Lit(lit) => match val {
                L1Value::Repr(v) => {
                    let lv = crate::elab::lit_value(site, lit).map_err(|e| L1Error::Stuck {
                        site: site.to_owned(),
                        why: format!("malformed literal pattern: {e}"),
                    })?;
                    Ok(lv.repr() == v.repr() && lv.payload() == v.payload())
                }
                L1Value::Data { .. } => Ok(false),
            },
        }
    }

    /// First-order application: user functions, saturated constructors (W6), and prims — split
    /// out of [`Self::eval`] for the same frame-size reason as [`Self::eval_match`].
    fn eval_app(
        &self,
        fuel: &mut u64,
        depth: u32,
        site: &str,
        scope: &mut Vec<(String, L1Value)>,
        head: &Expr,
        args: &[Expr],
    ) -> Result<L1Value, L1Error> {
        let Expr::Path(p) = head else {
            return Err(L1Error::Stuck {
                site: site.to_owned(),
                why: "v0 application head must be a name (first-order)".to_owned(),
            });
        };
        if p.0.len() != 1 {
            return Err(L1Error::Stuck {
                site: site.to_owned(),
                why: format!("dotted call `{}`", p.0.join(".")),
            });
        }
        let name = &p.0[0];
        // CBV: arguments evaluate left-to-right before any application.
        let mut argv = Vec::with_capacity(args.len());
        for a in args {
            argv.push(self.eval(fuel, depth, site, scope, a)?);
        }
        if self.env.fns.contains_key(name) {
            return self.invoke(fuel, depth, name, argv);
        }
        if let Some((d, i)) = self.env.ctor(name) {
            if d.ctors[i].fields.len() != argv.len() {
                return Err(L1Error::Stuck {
                    site: site.to_owned(),
                    why: format!("unsaturated constructor `{name}` (W6)"),
                });
            }
            return Ok(L1Value::Data {
                ty: d.name.clone(),
                ctor: name.clone(),
                fields: argv,
            });
        }
        if let Some(kernel) = prim_kernel_name(name) {
            let vals: Vec<&Value> = argv
                .iter()
                .map(|v| {
                    v.as_repr().ok_or_else(|| L1Error::Stuck {
                        site: site.to_owned(),
                        why: format!("prim `{name}` applied to a data value"),
                    })
                })
                .collect::<Result<_, _>>()?;
            let f = self
                .prims
                .get(kernel)
                .ok_or_else(|| L1Error::Kernel(KernelError::UnknownPrim(kernel.to_owned())))?;
            return Ok(L1Value::Repr(f(kernel, &vals)?));
        }
        Err(L1Error::Stuck {
            site: site.to_owned(),
            why: format!("unknown function/constructor/prim `{name}`"),
        })
    }

    /// The stage-0 dynamic guarantee check (RFC-0007 §4.3): the value's actual tag must be **at
    /// least as strong** as the asserted index — an annotation may only weaken (VR-5). The check
    /// never modifies the value: a passing assertion leaves the (possibly stronger) tag in place,
    /// and a failing one is an explicit error, never a downgrade-and-continue.
    fn assert_guarantee(&self, site: &str, v: &L1Value, asserted: Strength) -> Result<(), L1Error> {
        match v {
            L1Value::Repr(value) => {
                let actual = value.meta().guarantee();
                if actual.rank() > strength_of(asserted).rank() {
                    return Err(L1Error::GuaranteeTooWeak {
                        site: site.to_owned(),
                        asserted,
                        actual,
                    });
                }
                Ok(())
            }
            L1Value::Data { .. } => Err(L1Error::Unsupported {
                site: site.to_owned(),
                what: "a guarantee index on a data-typed value has no Meta to check in v0"
                    .to_owned(),
            }),
        }
    }
}

/// Forward a bridge refusal (shared with elaboration) as an explicit evaluator refusal.
fn unsupported(site: &str, e: &ElabError) -> L1Error {
    L1Error::Unsupported {
        site: site.to_owned(),
        what: e.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checkty::check_nodule;
    use crate::parse;
    use mycelium_core::Payload;

    fn env(src: &str) -> Env {
        check_nodule(&parse(src).expect("parses")).expect("checks")
    }

    fn run(src: &str) -> Result<L1Value, L1Error> {
        let env = env(src);
        Evaluator::new(&env).call("main", vec![])
    }

    #[test]
    fn literals_lets_and_prims_evaluate() {
        let v = run("nodule d\nfn main() -> Binary{8} = let a = 0b1010_1010 in not(a)")
            .expect("evaluates");
        let L1Value::Repr(v) = v else { panic!("repr") };
        assert_eq!(
            v.payload(),
            &Payload::Bits(vec![false, true, false, true, false, true, false, true])
        );
        assert_eq!(v.meta().guarantee(), GuaranteeStrength::Exact);
    }

    #[test]
    fn data_match_and_if_evaluate() {
        let v = run(
            "nodule d\ntype Sign = Neg | Zero | Pos\nfn label(s: Sign) -> Ternary{1} =\n  match s { Neg => <->, Zero => <0>, _ => <+> }\nfn main() -> Ternary{1} = label(Zero)",
        )
        .expect("evaluates");
        let L1Value::Repr(v) = v else { panic!("repr") };
        assert_eq!(
            v.payload(),
            &Payload::Trits(vec![mycelium_core::Trit::Zero])
        );
    }

    // --- nested patterns (Maranget) ----------------------------------------------------------

    const NAT: &str = "nodule d\ntype Nat = Z | S(Nat)\n";

    #[test]
    fn nested_pattern_match_evaluates() {
        // pred2 uses depth-2 nested patterns (S(Z), S(S(m))) and is exhaustive (Z | S(Z) | S(S(_))).
        // Mutant-witness: a flat-only matcher could not bind `m` under two constructors; pred2 of
        // S(S(S(Z))) must peel two S's to yield S(Z).
        let src = format!(
            "{NAT}fn pred2(n: Nat) -> Nat = match n {{ Z => Z, S(Z) => Z, S(S(m)) => m }}\n\
             fn main() -> Nat = pred2(S(S(S(Z))))"
        );
        assert_eq!(
            run(&src).expect("evaluates"),
            L1Value::Data {
                ty: "Nat".into(),
                ctor: "S".into(),
                fields: vec![L1Value::Data {
                    ty: "Nat".into(),
                    ctor: "Z".into(),
                    fields: vec![]
                }]
            }
        );
    }

    #[test]
    fn nested_match_falls_through_to_the_right_arm() {
        // S(Z) selects the middle arm (not S(S(m))) — the nested matcher discriminates by depth.
        let src = format!(
            "{NAT}fn pred2(n: Nat) -> Nat = match n {{ Z => Z, S(Z) => S(Z), S(S(m)) => m }}\n\
             fn main() -> Nat = pred2(S(Z))"
        );
        assert_eq!(
            run(&src).expect("evaluates"),
            L1Value::Data {
                ty: "Nat".into(),
                ctor: "S".into(),
                fields: vec![L1Value::Data {
                    ty: "Nat".into(),
                    ctor: "Z".into(),
                    fields: vec![]
                }]
            }
        );
    }

    // --- M-320: literal-pattern match over Binary/Ternary scrutinees -------------------------

    const CLASSIFY: &str = "nodule d\nfn classify(b: Binary{4}) -> Ternary{1} = \
        match b { 0b0000 => <0>, 0b1111 => <+>, _ => <-> }\n\
        fn main() -> Ternary{1} = classify(0b1111)";

    #[test]
    fn literal_match_over_binary_selects_the_matching_arm() {
        // Mutant-witness: if eval_literal_match compared the wrong payload (or always took the
        // first arm), classify(0b1111) would not yield <+>.
        let v = run(CLASSIFY).expect("evaluates");
        let L1Value::Repr(v) = v else { panic!("repr") };
        assert_eq!(v.payload(), &Payload::Trits(vec![mycelium_core::Trit::Pos]));
    }

    #[test]
    fn literal_match_falls_through_to_the_default() {
        // Mutant-witness: if a non-matching literal arm fired anyway, classify(0b0101) would not
        // reach the `_` default <->.
        let src = CLASSIFY.replace("classify(0b1111)", "classify(0b0101)");
        let L1Value::Repr(v) = run(&src).expect("evaluates") else {
            panic!("repr")
        };
        assert_eq!(v.payload(), &Payload::Trits(vec![mycelium_core::Trit::Neg]));
    }

    #[test]
    fn literal_match_without_a_default_is_non_exhaustive() {
        // Mutant-witness: dropping the mandatory-default check would let a literal match silently
        // assume coverage of the 2^4 domain (W7 violation).
        let src = "nodule d\nfn classify(b: Binary{4}) -> Ternary{1} = \
            match b { 0b0000 => <0>, 0b1111 => <+> }\nfn main() -> Ternary{1} = classify(0b1111)";
        let err = check_nodule(&parse(src).expect("parses")).expect_err("must reject");
        assert!(
            err.message.contains("non-exhaustive"),
            "got: {}",
            err.message
        );
    }

    #[test]
    fn duplicate_literal_pattern_is_rejected() {
        // Mutant-witness: a duplicate literal arm is a redundant (unreachable) arm — the Maranget
        // usefulness check must reject it, never silently accept it (W7). `0b0000` and `0b00_00` are
        // the same literal (the `_` separator is canonicalized away), so the second is unreachable.
        let src = "nodule d\nfn classify(b: Binary{4}) -> Ternary{1} = \
            match b { 0b0000 => <0>, 0b00_00 => <+>, _ => <-> }\nfn main() -> Ternary{1} = classify(0b0000)";
        let err = check_nodule(&parse(src).expect("parses")).expect_err("must reject");
        assert!(err.message.contains("unreachable"), "got: {}", err.message);
    }

    #[test]
    fn literal_pattern_width_must_match_the_scrutinee() {
        // Mutant-witness: dropping the width check would let a 2-bit literal match a Binary{4}
        // scrutinee — a payload-length mismatch that could never fire (or panic downstream).
        let src = "nodule d\nfn classify(b: Binary{4}) -> Ternary{1} = \
            match b { 0b00 => <0>, _ => <-> }\nfn main() -> Ternary{1} = classify(0b0000)";
        let err = check_nodule(&parse(src).expect("parses")).expect_err("must reject");
        assert!(
            err.message.contains("literal pattern has type"),
            "got: {}",
            err.message
        );
    }

    #[test]
    fn structural_recursion_terminates_within_fuel() {
        // `drop_` is classified Total (structural descent) — and indeed terminates.
        let v = run(
            "nodule d\ntype Nat = Z | S(Nat)\nfn drop_(n: Nat) -> Nat = match n { Z => Z, S(m) => drop_(m) }\nfn main() -> Nat = drop_(S(S(Z)))",
        )
        .expect("terminates");
        assert_eq!(
            v,
            L1Value::Data {
                ty: "Nat".into(),
                ctor: "Z".into(),
                fields: vec![]
            }
        );
    }

    #[test]
    fn an_unproductive_recursion_is_an_explicit_fuel_exhaustion() {
        // With the clock tighter than the depth guard, the *semantic* budget trips first.
        let env = env(
            "nodule d\ntype Nat = Z | S(Nat)\nfn spin(n: Nat) -> Nat = spin(n)\nfn main() -> Nat = spin(Z)",
        );
        let err = Evaluator::new(&env)
            .with_fuel(50)
            .call("main", vec![])
            .unwrap_err();
        assert_eq!(err, L1Error::FuelExhausted);
    }

    #[test]
    fn deep_recursion_trips_the_host_stack_guard_explicitly() {
        // With ample fuel, the depth guard refuses explicitly — never a host stack overflow.
        let env = env(
            "nodule d\ntype Nat = Z | S(Nat)\nfn spin(n: Nat) -> Nat = spin(n)\nfn main() -> Nat = spin(Z)",
        );
        let err = Evaluator::new(&env).call("main", vec![]).unwrap_err();
        assert!(
            matches!(err, L1Error::DepthExceeded { .. }),
            "expected DepthExceeded, got {err:?}"
        );
    }

    #[test]
    fn deeply_nested_expression_trips_the_depth_guard_without_any_recursive_call() {
        // A4-03 mutant-witness: depth is charged per AST node, so a *wide-but-shallow* program —
        // here a deep but call-free `not(not(… not(0b…) …))` nest, which makes no recursive
        // function call at all — still hits the host-stack guard explicitly once its nesting
        // exceeds DEFAULT_DEPTH (64). This pins the documented per-node (not per-call-frame)
        // contract: a refactor charging depth only at `invoke` would let this nest recurse on the
        // host stack unguarded, flipping this assertion (the depth guard would never trip) and so
        // turning the test red — the regression we want to catch.
        let mut expr = "0b0000_0001".to_owned();
        for _ in 0..200 {
            expr = format!("not({expr})");
        }
        let deep_env = env(&format!("nodule d\nfn main() -> Binary{{8}} = {expr}"));
        let err = Evaluator::new(&deep_env).call("main", vec![]).unwrap_err();
        assert!(
            matches!(
                err,
                L1Error::DepthExceeded {
                    limit: DEFAULT_DEPTH
                }
            ),
            "expected DepthExceeded(limit=64) from a call-free 200-deep nest, got {err:?}"
        );

        // And the same nest within the depth budget evaluates fine (200 nested `not`s exceed 64;
        // a small nest does not) — confirming the guard refuses *only* genuine over-nesting.
        let shallow = env("nodule d\nfn main() -> Binary{8} = not(not(0b0000_0001))");
        Evaluator::new(&shallow)
            .call("main", vec![])
            .expect("a shallow nest is well within the depth budget");
    }

    #[test]
    fn a_for_fold_evaluates_head_to_tail() {
        // checksum(More(0b1111_0000, More(0b0000_1111, End))) = 0b1111_1111 (xor-fold).
        let v = run(
            "nodule d\ntype Bytes = End | More(Binary{8}, Bytes)\n\
             fn checksum(bs: Bytes) -> Binary{8} =\n    for b in bs, acc = 0b0000_0000 => xor(acc, b)\n\
             fn main() -> Binary{8} = checksum(More(0b1111_0000, More(0b0000_1111, End)))",
        )
        .expect("evaluates");
        let L1Value::Repr(v) = v else { panic!("repr") };
        assert_eq!(v.payload(), &Payload::Bits(vec![true; 8]));
    }

    #[test]
    fn a_for_fold_over_nil_is_the_initial_accumulator() {
        let v = run(
            "nodule d\ntype Bytes = End | More(Binary{8}, Bytes)\n\
             fn checksum(bs: Bytes) -> Binary{8} =\n    for b in bs, acc = 0b1010_1010 => xor(acc, b)\n\
             fn main() -> Binary{8} = checksum(End)",
        )
        .expect("evaluates");
        let L1Value::Repr(v) = v else { panic!("repr") };
        assert_eq!(
            v.payload(),
            &Payload::Bits(vec![true, false, true, false, true, false, true, false])
        );
    }

    #[test]
    fn a_long_for_fold_costs_fuel_not_host_stack() {
        // 200 elements would blow the depth guard (64) as hand-written recursion; the `for`
        // spine walk is iterative and must not (RFC-0007 §4.8). The list value is built
        // programmatically — a 200-deep nested *expression* would itself be depth-guarded.
        let env = env(
            "nodule d\ntype Bytes = End | More(Binary{8}, Bytes)\n\
             fn checksum(bs: Bytes) -> Binary{8} =\n    for b in bs, acc = 0b0000_0000 => xor(acc, b)",
        );
        let byte = || {
            L1Value::Repr(
                Value::new(
                    mycelium_core::Repr::Binary { width: 8 },
                    Payload::Bits(vec![false, false, false, false, false, false, false, true]),
                    mycelium_core::Meta::exact(mycelium_core::Provenance::Root),
                )
                .unwrap(),
            )
        };
        let mut list = L1Value::Data {
            ty: "Bytes".into(),
            ctor: "End".into(),
            fields: vec![],
        };
        for _ in 0..200 {
            list = L1Value::Data {
                ty: "Bytes".into(),
                ctor: "More".into(),
                fields: vec![byte(), list],
            };
        }
        let v = Evaluator::new(&env)
            .call("checksum", vec![list])
            .expect("evaluates");
        let L1Value::Repr(v) = v else { panic!("repr") };
        // 200 xors of 0b0000_0001 → even count → all zeros.
        assert_eq!(v.payload(), &Payload::Bits(vec![false; 8]));
    }

    #[test]
    fn the_certified_swap_runs_and_a_weakening_assertion_passes() {
        // The in-range binary→ternary swap is Exact; asserting `@ Proven` weakens — allowed.
        let v = run(
            "nodule d\nfn main() -> Ternary{6} @ Proven = swap(0b0000_0010, to: Ternary{6}, policy: rt)",
        )
        .expect("evaluates");
        let L1Value::Repr(v) = v else { panic!("repr") };
        assert_eq!(v.repr(), &mycelium_core::Repr::Ternary { trits: 6 });
    }

    #[test]
    fn asserting_stronger_than_actual_is_an_explicit_error() {
        // A Declared-bound value asserted `@ Exact` must refuse (VR-5: never upgrade).
        let declared = Value::new(
            mycelium_core::Repr::Binary { width: 2 },
            Payload::Bits(vec![true, false]),
            mycelium_core::Meta::new(
                mycelium_core::Provenance::Root,
                GuaranteeStrength::Declared,
                Some(mycelium_core::Bound {
                    kind: mycelium_core::BoundKind::Error {
                        eps: 0.1,
                        norm: mycelium_core::NormKind::Linf,
                    },
                    basis: mycelium_core::BoundBasis::UserDeclared,
                }),
                None,
                None,
                None,
            )
            .expect("well-formed meta"),
        )
        .expect("well-formed value");
        let env = env("nodule d\nfn main() -> Binary{8} = 0b0000_0000");
        let ev = Evaluator::new(&env);
        let err = ev
            .assert_guarantee("t", &L1Value::Repr(declared), Strength::Exact)
            .unwrap_err();
        assert!(matches!(err, L1Error::GuaranteeTooWeak { .. }), "{err:?}");
    }

    #[test]
    fn wild_and_unknown_names_are_explicit_refusals() {
        // `wild` never reaches evaluation through the checker; drive the evaluator directly on an
        // unchecked nodule to confirm the refusal is the evaluator's own, not just the checker's.
        let nodule =
            parse("nodule d\nfn main() -> Binary{8} = wild { foreign(0b0000_0001) }").unwrap();
        let env = Env {
            types: std::collections::BTreeMap::new(),
            fns: nodule
                .items
                .iter()
                .filter_map(|i| match i {
                    crate::ast::Item::Fn(f) => Some((f.sig.name.clone(), f.clone())),
                    _ => None,
                })
                .collect(),
            totality: std::collections::BTreeMap::new(),
        };
        let err = Evaluator::new(&env).call("main", vec![]).unwrap_err();
        assert!(matches!(err, L1Error::Unsupported { .. }), "{err:?}");
    }

    // --- M-642 additive ergonomics: EvaluatorOpts / with_opts -----------------------------------

    #[test]
    fn evaluator_opts_default_matches_new_budgets() {
        // `with_opts(default)` is a no-op: same observable result as plain `new` on a program that
        // runs well inside both budgets.
        let e = env("nodule d\nfn main() -> Binary{8} = not(0b0000_0000)");
        let baseline = Evaluator::new(&e).call("main", vec![]).expect("evaluates");
        let via_opts = Evaluator::new(&e)
            .with_opts(EvaluatorOpts::default())
            .call("main", vec![])
            .expect("evaluates");
        assert_eq!(baseline, via_opts);
    }

    #[test]
    fn evaluator_opts_apply_the_fuel_budget() {
        // A starvation-level fuel budget supplied via `with_opts` must take effect — proving the
        // opts struct is actually applied (each node costs one unit; 1 unit cannot finish `not(_)`).
        let e = env("nodule d\nfn main() -> Binary{8} = not(0b0000_0000)");
        let err = Evaluator::new(&e)
            .with_opts(EvaluatorOpts::default().fuel(1))
            .call("main", vec![])
            .unwrap_err();
        assert!(matches!(err, L1Error::FuelExhausted), "{err:?}");
    }

    #[test]
    fn evaluator_opts_builder_sets_both_fields() {
        let o = EvaluatorOpts::default().fuel(42).depth(7);
        assert_eq!(o.fuel, 42);
        assert_eq!(o.depth, 7);
        // `with_opts` is exactly the `with_fuel`+`with_depth` chain (same observable behavior under a
        // generous budget — both evaluate the program), checked here via the no-op-on-success path.
        let e = env("nodule d\nfn main() -> Binary{8} = not(0b1111_0000)");
        let chained = Evaluator::new(&e)
            .with_fuel(1_000)
            .with_depth(64)
            .call("main", vec![])
            .expect("evaluates");
        let opted = Evaluator::new(&e)
            .with_opts(EvaluatorOpts::default().fuel(1_000).depth(64))
            .call("main", vec![])
            .expect("evaluates");
        assert_eq!(chained, opted);
    }
}
