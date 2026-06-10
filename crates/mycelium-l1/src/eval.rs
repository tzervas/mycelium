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
use mycelium_core::{GuaranteeStrength, Value};
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
    fn eval(
        &self,
        fuel: &mut u64,
        depth: u32,
        site: &str,
        scope: &mut Vec<(String, L1Value)>,
        e: &Expr,
    ) -> Result<L1Value, L1Error> {
        *fuel = fuel.checked_sub(1).ok_or(L1Error::FuelExhausted)?;
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
        let L1Value::Data { ty, ctor, fields } = sv else {
            return Err(L1Error::Stuck {
                site: site.to_owned(),
                why: "match scrutinee is not a data value".to_owned(),
            });
        };
        for arm in arms {
            match &arm.pattern {
                Pattern::Wildcard => return self.eval(fuel, depth, site, scope, &arm.body),
                Pattern::Ident(n) => {
                    // A nullary-constructor alternative of the scrutinee's type, or a binder
                    // default — same resolution the typechecker performed (W7).
                    let is_ctor = self
                        .env
                        .types
                        .get(&ty)
                        .is_some_and(|d| d.ctors.iter().any(|c| c.name == *n));
                    if is_ctor {
                        if *n == ctor {
                            return self.eval(fuel, depth, site, scope, &arm.body);
                        }
                    } else {
                        scope.push((
                            n.clone(),
                            L1Value::Data {
                                ty: ty.clone(),
                                ctor: ctor.clone(),
                                fields: fields.clone(),
                            },
                        ));
                        let r = self.eval(fuel, depth, site, scope, &arm.body);
                        scope.pop();
                        return r;
                    }
                }
                Pattern::Ctor(n, subs) => {
                    if *n != ctor {
                        continue;
                    }
                    let mut pushed = 0;
                    for (sub, fv) in subs.iter().zip(&fields) {
                        match sub {
                            Pattern::Ident(b) if self.env.ctor(b).is_none() => {
                                scope.push((b.clone(), fv.clone()));
                                pushed += 1;
                            }
                            Pattern::Wildcard | Pattern::Ident(_) => {}
                            Pattern::Ctor(..) | Pattern::Lit(_) => {
                                return Err(L1Error::Unsupported {
                                    site: site.to_owned(),
                                    what:
                                        "nested/literal patterns are refused in v0 (W7 flat match)"
                                            .to_owned(),
                                })
                            }
                        }
                    }
                    let r = self.eval(fuel, depth, site, scope, &arm.body);
                    for _ in 0..pushed {
                        scope.pop();
                    }
                    return r;
                }
                Pattern::Lit(_) => {
                    return Err(L1Error::Unsupported {
                        site: site.to_owned(),
                        what: "literal patterns are deferred in v0".to_owned(),
                    })
                }
            }
        }
        Err(L1Error::Stuck {
            site: site.to_owned(),
            why: format!("no arm matched constructor `{ctor}` of `{ty}` (W7 coverage)"),
        })
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
    use crate::checkty::check_colony;
    use crate::parse;
    use mycelium_core::Payload;

    fn env(src: &str) -> Env {
        check_colony(&parse(src).expect("parses")).expect("checks")
    }

    fn run(src: &str) -> Result<L1Value, L1Error> {
        let env = env(src);
        Evaluator::new(&env).call("main", vec![])
    }

    #[test]
    fn literals_lets_and_prims_evaluate() {
        let v = run("colony d\nfn main() -> Binary{8} = let a = 0b1010_1010 in not(a)")
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
            "colony d\ntype Sign = Neg | Zero | Pos\nfn label(s: Sign) -> Ternary{1} =\n  match s { Neg => <->, Zero => <0>, _ => <+> }\nfn main() -> Ternary{1} = label(Zero)",
        )
        .expect("evaluates");
        let L1Value::Repr(v) = v else { panic!("repr") };
        assert_eq!(
            v.payload(),
            &Payload::Trits(vec![mycelium_core::Trit::Zero])
        );
    }

    #[test]
    fn structural_recursion_terminates_within_fuel() {
        // `drop_` is classified Total (structural descent) — and indeed terminates.
        let v = run(
            "colony d\ntype Nat = Z | S(Nat)\nfn drop_(n: Nat) -> Nat = match n { Z => Z, S(m) => drop_(m) }\nfn main() -> Nat = drop_(S(S(Z)))",
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
            "colony d\ntype Nat = Z | S(Nat)\nfn spin(n: Nat) -> Nat = spin(n)\nfn main() -> Nat = spin(Z)",
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
            "colony d\ntype Nat = Z | S(Nat)\nfn spin(n: Nat) -> Nat = spin(n)\nfn main() -> Nat = spin(Z)",
        );
        let err = Evaluator::new(&env).call("main", vec![]).unwrap_err();
        assert!(
            matches!(err, L1Error::DepthExceeded { .. }),
            "expected DepthExceeded, got {err:?}"
        );
    }

    #[test]
    fn a_for_fold_evaluates_head_to_tail() {
        // checksum(More(0b1111_0000, More(0b0000_1111, End))) = 0b1111_1111 (xor-fold).
        let v = run(
            "colony d\ntype Bytes = End | More(Binary{8}, Bytes)\n\
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
            "colony d\ntype Bytes = End | More(Binary{8}, Bytes)\n\
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
            "colony d\ntype Bytes = End | More(Binary{8}, Bytes)\n\
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
            "colony d\nfn main() -> Ternary{6} @ Proven = swap(0b0000_0010, to: Ternary{6}, policy: rt)",
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
        let env = env("colony d\nfn main() -> Binary{8} = 0b0000_0000");
        let ev = Evaluator::new(&env);
        let err = ev
            .assert_guarantee("t", &L1Value::Repr(declared), Strength::Exact)
            .unwrap_err();
        assert!(matches!(err, L1Error::GuaranteeTooWeak { .. }), "{err:?}");
    }

    #[test]
    fn wild_and_unknown_names_are_explicit_refusals() {
        // `wild` never reaches evaluation through the checker; drive the evaluator directly on an
        // unchecked colony to confirm the refusal is the evaluator's own, not just the checker's.
        let colony =
            parse("colony d\nfn main() -> Binary{8} = wild { foreign(0b0000_0001) }").unwrap();
        let env = Env {
            types: std::collections::BTreeMap::new(),
            fns: colony
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
}
