//! The runnable AOT artifact (M-150 "runnable artifact for the subset"; M-151 differential target).
//!
//! Executes the **lowered A-normal form** (`mycelium-core::lower`) with a **big-step env-machine**:
//! bindings are evaluated in order into an environment, operands are looked up, primitives and swaps
//! are applied via the shared registries. This is an *independent execution path* from the M-110
//! reference interpreter (small-step substitution over the nested tree), so checking the two for
//! observable equivalence (M-151) is a real NFR-7 two-path test — it stands in for "interpreter vs
//! compiled native" until the libMLIR backend lands.
//!
//! **Full v0 calculus (M-342; RFC-0011 §4.4 Q5 closed).** [`run_core`] covers the whole calculus —
//! `Const/Var/Let/Op/Swap` plus the r3/r4 data + recursion nodes: `Construct` builds a [`Datum`],
//! `Match` selects an arm (binding constructor fields), `Lam` is a closure capturing its environment,
//! `App` applies it call-by-value, and `Fix` unfolds under a fuel clock. The three-way differential
//! (L1-eval ≡ L0-interp ≡ AOT) now spans this whole fragment. (The *native LLVM* backend stays the
//! bit/trit subset and refuses the rest with an explicit `UnsupportedNode` — VR-5; data/closure
//! codegen is the deferred MLIR→LLVM work.)
//!
//! **Honest limitation (VR-5).** Unlike the reference interpreter — which iterates `step` and so uses
//! *O(1)* host stack regardless of object-level recursion depth — this env-machine uses the **host
//! call stack** for object recursion (`Fix` unfolds nest Rust frames). The **fuel clock bounds
//! productive work** (a non-productive recursion is an explicit [`EvalError::FuelExhausted`], never a
//! hang), but recursion *deeper than the host stack allows* aborts rather than returning a graceful
//! error. For the bounded-depth differential corpus this never arises; the **trusted** base for deep
//! recursion remains the interpreter, and a stack-robust native path is the deferred MLIR→LLVM work.

use std::collections::HashMap;

use mycelium_core::lower::{self, Anf, AnfAlt, Atom, Rhs};
use mycelium_core::{
    CoreValue, Datum, GuaranteeStrength, Node, PackScheme, Payload, PhysicalLayout, Repr, Value,
};
use mycelium_interp::{EvalError, PrimRegistry, SwapEngine};

use crate::pack;

/// The default fuel for the env-machine's `Fix` clock — generous; the guard is against a
/// non-productive recursion, surfaced as an explicit [`EvalError::FuelExhausted`], never a hang
/// (mirrors the reference interpreter, RFC-0007 §4.5).
const AOT_FUEL: u64 = 1_000_000;

/// A value in the AOT env-machine: a fully-evaluated [`CoreValue`] (repr value or datum), or a
/// **closure** / **recursive suspension** for the r4 function fragment. Closures capture their
/// defining environment by value (the v0 surface is first-order, so this is a finite capture).
#[derive(Clone)]
enum AotVal {
    /// A representation value or a datum (a normal form).
    Core(CoreValue),
    /// A lambda closure: parameter, body block, and the captured environment.
    Closure { param: String, body: Anf, env: Env },
    /// A `Fix` suspension: unfolds on application under the fuel clock.
    Fix { name: String, body: Anf, env: Env },
}

type Env = HashMap<Atom, AotVal>;

fn lookup(env: &Env, a: &Atom) -> Result<AotVal, EvalError> {
    env.get(a).cloned().ok_or_else(|| match a {
        Atom::Named(x) => EvalError::FreeVariable(x.clone()),
        Atom::Temp(k) => EvalError::FreeVariable(format!("%{k}")),
    })
}

/// Coerce an [`AotVal`] to a [`CoreValue`], or an explicit refusal for a bare function value (a v0
/// entry returns a repr/data value, never a function — mirrors the interpreter's `FunctionResult`).
fn as_core(v: AotVal) -> Result<CoreValue, EvalError> {
    match v {
        AotVal::Core(cv) => Ok(cv),
        AotVal::Closure { .. } | AotVal::Fix { .. } => Err(EvalError::FunctionResult),
    }
}

/// Coerce an [`AotVal`] to a representation [`Value`] (for an `Op`/`Swap` operand): a datum or a
/// function in that position is a type error the checker prevents — explicit, never a guess.
fn as_repr_value(v: AotVal) -> Result<Value, EvalError> {
    match v {
        AotVal::Core(CoreValue::Repr(rv)) => Ok(rv),
        AotVal::Core(CoreValue::Data(_)) => Err(EvalError::DataMalformed {
            why: "a primitive/swap operand reduced to a data value, not a representation value"
                .to_owned(),
        }),
        AotVal::Closure { .. } | AotVal::Fix { .. } => Err(EvalError::DataMalformed {
            why: "a primitive/swap operand reduced to a function value".to_owned(),
        }),
    }
}

/// Run a Core IR program through the AOT path to a [`CoreValue`] (the full v0 calculus — repr, data,
/// and recursion; M-342). Lowers to ANF, then evaluates with a big-step environment machine (an
/// independent path from the M-110 small-step interpreter — the NFR-7 two-path check).
pub fn run_core(
    node: &Node,
    prims: &PrimRegistry,
    swap: &dyn SwapEngine,
) -> Result<CoreValue, EvalError> {
    run_core_with_fuel(node, prims, swap, AOT_FUEL)
}

/// [`run_core`] with an explicit `Fix`-unfold budget. A non-productive recursion exhausts it as an
/// explicit [`EvalError::FuelExhausted`] — never a hang (the never-silent termination guard).
pub fn run_core_with_fuel(
    node: &Node,
    prims: &PrimRegistry,
    swap: &dyn SwapEngine,
    fuel: u64,
) -> Result<CoreValue, EvalError> {
    let anf = lower::lower_to_anf(node);
    let mut fuel = fuel;
    let result = eval_block(&anf, Env::new(), prims, swap, &mut fuel)?;
    as_core(result)
}

/// Run a Core IR program through the AOT path to a representation [`Value`]. Convenience over
/// [`run_core`] for the repr fragment: a data result is the explicit [`EvalError::DataResult`]
/// (mirrors `Interpreter::eval`), never a silent mishandling.
pub fn run(node: &Node, prims: &PrimRegistry, swap: &dyn SwapEngine) -> Result<Value, EvalError> {
    match run_core(node, prims, swap)? {
        CoreValue::Repr(v) => Ok(v),
        CoreValue::Data(_) => Err(EvalError::DataResult),
    }
}

/// Evaluate one ANF block in `env`: bind each RHS in order, then return the result atom's value.
fn eval_block(
    anf: &Anf,
    mut env: Env,
    prims: &PrimRegistry,
    swap: &dyn SwapEngine,
    fuel: &mut u64,
) -> Result<AotVal, EvalError> {
    for b in anf.bindings() {
        let v = eval_rhs(&b.rhs, &env, prims, swap, fuel)?;
        env.insert(b.name.clone(), v);
    }
    lookup(&env, anf.result())
}

fn eval_rhs(
    rhs: &Rhs,
    env: &Env,
    prims: &PrimRegistry,
    swap: &dyn SwapEngine,
    fuel: &mut u64,
) -> Result<AotVal, EvalError> {
    match rhs {
        Rhs::Const(v) => Ok(AotVal::Core(CoreValue::Repr(v.clone()))),
        Rhs::Alias(a) => lookup(env, a),
        Rhs::Op { prim, args } => {
            let vals: Vec<Value> = args
                .iter()
                .map(|a| as_repr_value(lookup(env, a)?))
                .collect::<Result<_, _>>()?;
            let refs: Vec<&Value> = vals.iter().collect();
            let f = prims
                .get(prim)
                .ok_or_else(|| EvalError::UnknownPrim(prim.clone()))?;
            Ok(AotVal::Core(CoreValue::Repr(f(prim, &refs)?)))
        }
        Rhs::Swap {
            src,
            target,
            policy,
        } => {
            let s = as_repr_value(lookup(env, src)?)?;
            Ok(AotVal::Core(CoreValue::Repr(
                swap.swap(&s, target, policy)?,
            )))
        }
        Rhs::Construct { ctor, args } => {
            let fields: Vec<CoreValue> = args
                .iter()
                .map(|a| as_core(lookup(env, a)?))
                .collect::<Result<_, _>>()?;
            Ok(AotVal::Core(CoreValue::Data(Datum::new(
                ctor.clone(),
                fields,
            ))))
        }
        // A lambda captures the current environment (first-order ⇒ finite capture).
        Rhs::Lam { param, body } => Ok(AotVal::Closure {
            param: param.clone(),
            body: body.clone(),
            env: env.clone(),
        }),
        Rhs::Fix { name, body } => Ok(AotVal::Fix {
            name: name.clone(),
            body: body.clone(),
            env: env.clone(),
        }),
        Rhs::App { func, arg } => {
            let f = lookup(env, func)?;
            let a = lookup(env, arg)?;
            apply(f, a, prims, swap, fuel)
        }
        Rhs::Match {
            scrutinee,
            alts,
            default,
        } => {
            let cv = as_core(lookup(env, scrutinee)?)?;
            // r3 boundary (RFC-0011 §4.6): the guarantee-meet through Match is the identity only when
            // the scrutinee is Exact; a non-Exact scrutinee is the explicit deferral (never a
            // fabricated bound) — mirrors the reference interpreter.
            if cv.guarantee() != GuaranteeStrength::Exact {
                return Err(EvalError::GuaranteeMeetUnsupported {
                    scrutinee: cv.guarantee(),
                });
            }
            select_and_eval(&cv, alts, default.as_ref(), env, prims, swap, fuel)
        }
    }
}

/// Select the first-matching arm (or default) of a lowered `Match` and evaluate its block, binding
/// constructor fields left-to-right (mirrors the interpreter's `select_arm`). No match + no default
/// is an explicit [`EvalError::NonExhaustiveMatch`] (the checker proves coverage above the kernel).
fn select_and_eval(
    cv: &CoreValue,
    alts: &[AnfAlt],
    default: Option<&Anf>,
    env: &Env,
    prims: &PrimRegistry,
    swap: &dyn SwapEngine,
    fuel: &mut u64,
) -> Result<AotVal, EvalError> {
    for alt in alts {
        match alt {
            AnfAlt::Ctor {
                ctor,
                binders,
                body,
            } => {
                if let CoreValue::Data(d) = cv {
                    if d.ctor() == ctor {
                        if binders.len() != d.fields().len() {
                            return Err(EvalError::DataMalformed {
                                why: format!(
                                    "constructor arm binds {} of {} field(s) (WF6/WF7)",
                                    binders.len(),
                                    d.fields().len()
                                ),
                            });
                        }
                        let mut arm_env = env.clone();
                        for (binder, field) in binders.iter().zip(d.fields()) {
                            arm_env
                                .insert(Atom::Named(binder.clone()), AotVal::Core(field.clone()));
                        }
                        return eval_block(body, arm_env, prims, swap, fuel);
                    }
                }
            }
            AnfAlt::Lit { value, body } => {
                if let CoreValue::Repr(rv) = cv {
                    if rv.repr() == value.repr() && rv.payload() == value.payload() {
                        return eval_block(body, env.clone(), prims, swap, fuel);
                    }
                }
            }
        }
    }
    match default {
        Some(d) => eval_block(d, env.clone(), prims, swap, fuel),
        None => Err(EvalError::NonExhaustiveMatch),
    }
}

/// Apply a function value to an argument. A closure runs its body in the captured environment with
/// the parameter bound; a `Fix` **unfolds** under the fuel clock — `Fix(f,e)` evaluates `e` with `f`
/// bound to the `Fix` itself, then applies the result (always a closure for the first-order surface).
/// Applying a non-function is an explicit [`EvalError::ApplyNonFunction`].
fn apply(
    f: AotVal,
    arg: AotVal,
    prims: &PrimRegistry,
    swap: &dyn SwapEngine,
    fuel: &mut u64,
) -> Result<AotVal, EvalError> {
    match f {
        AotVal::Closure { param, body, env } => {
            let mut call_env = env;
            call_env.insert(Atom::Named(param), arg);
            eval_block(&body, call_env, prims, swap, fuel)
        }
        AotVal::Fix {
            ref name,
            ref body,
            ref env,
        } => {
            *fuel = fuel.checked_sub(1).ok_or(EvalError::FuelExhausted)?;
            let mut unfold_env = env.clone();
            unfold_env.insert(Atom::Named(name.clone()), f.clone());
            let unfolded = eval_block(body, unfold_env, prims, swap, fuel)?;
            apply(unfolded, arg, prims, swap, fuel)
        }
        AotVal::Core(_) => Err(EvalError::ApplyNonFunction),
    }
}

/// Run a Core IR program through the AOT path **with a schedule-staged packing layout** (M-251;
/// RFC-0004 §5/§8). The result is first computed by [`run`], then — for a ternary result — its
/// trits are materialized into a physical buffer **packed under `packed_as`** and **read back under
/// the recorded tag `read_as`** (the `Meta.physical` claim), and the layout is recorded on the
/// result's `Meta` (M-I5 lossless, [`Value`]'s `with_physical`).
///
/// When the tag is correct (`packed_as == read_as`) the read-back is the identity, so the result is
/// observably equal to the layout-agnostic reference (the interpreter / [`run`]) — and the M-210
/// observational-equivalence check validates. A **mislabeled** tag (`packed_as != read_as`)
/// misreads the buffer, producing a different payload that the same check rejects (NFR-7) — the E3
/// soundness property: the layout record is trusted *only because a wrong one is caught*.
///
/// Non-ternary results carry no trit-packing layout, so they pass through unchanged.
pub fn run_with_layout(
    node: &Node,
    prims: &PrimRegistry,
    swap: &dyn SwapEngine,
    packed_as: PackScheme,
    read_as: PackScheme,
) -> Result<Value, EvalError> {
    let v = run(node, prims, swap)?;
    match (v.repr(), v.payload()) {
        (Repr::Ternary { .. }, Payload::Trits(trits)) => {
            let read = pack::relayout_trits(trits, packed_as, read_as);
            let meta = v
                .meta()
                .clone()
                .with_physical(PhysicalLayout::TritPacked { scheme: read_as });
            Value::new(v.repr().clone(), Payload::Trits(read), meta)
                .map_err(|e| EvalError::Swap(e.to_string()))
        }
        _ => Ok(v),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mycelium_core::{Meta, Payload, Provenance, Repr};
    use mycelium_interp::IdentitySwapEngine;

    fn byte() -> Value {
        Value::new(
            Repr::Binary { width: 8 },
            Payload::Bits(vec![true, false, true, true, false, false, true, false]),
            Meta::exact(Provenance::Root),
        )
        .unwrap()
    }

    #[test]
    fn runs_a_let_op_program() {
        // let a = byte in bit.not(a)
        let node = Node::Let {
            id: "a".into(),
            bound: Box::new(Node::Const(byte())),
            body: Box::new(Node::Op {
                prim: "bit.not".into(),
                args: vec![Node::Var("a".into())],
            }),
        };
        let out = run(&node, &PrimRegistry::with_builtins(), &IdentitySwapEngine).unwrap();
        let expected: Vec<bool> = match byte().payload() {
            Payload::Bits(b) => b.iter().map(|&x| !x).collect(),
            _ => unreachable!(),
        };
        assert_eq!(out.payload(), &Payload::Bits(expected));
    }

    #[test]
    fn free_variable_is_explicit() {
        let node = Node::Var("nope".into());
        assert_eq!(
            run(&node, &PrimRegistry::with_builtins(), &IdentitySwapEngine),
            Err(EvalError::FreeVariable("nope".into()))
        );
    }

    #[test]
    fn applies_a_closure_in_the_env_machine() {
        // (λx. bit.not(x)) byte  — exercises Lam capture + App + closure-body eval (M-342).
        let node = Node::App {
            func: Box::new(Node::Lam {
                param: "x".into(),
                body: Box::new(Node::Op {
                    prim: "bit.not".into(),
                    args: vec![Node::Var("x".into())],
                }),
            }),
            arg: Box::new(Node::Const(byte())),
        };
        let out = run(&node, &PrimRegistry::with_builtins(), &IdentitySwapEngine).unwrap();
        let expected: Vec<bool> = match byte().payload() {
            Payload::Bits(b) => b.iter().map(|&x| !x).collect(),
            _ => unreachable!(),
        };
        assert_eq!(out.payload(), &Payload::Bits(expected));
    }

    #[test]
    fn a_nonproductive_recursion_exhausts_fuel_not_hangs() {
        // (fix f => λx. f x) byte — unfolds forever; the fuel clock makes it an explicit
        // FuelExhausted, never a hang (the never-silent termination guard, mirroring the interpreter).
        let node = Node::App {
            func: Box::new(Node::Fix {
                name: "f".into(),
                body: Box::new(Node::Lam {
                    param: "x".into(),
                    body: Box::new(Node::App {
                        func: Box::new(Node::Var("f".into())),
                        arg: Box::new(Node::Var("x".into())),
                    }),
                }),
            }),
            arg: Box::new(Node::Const(byte())),
        };
        let r = run_core_with_fuel(
            &node,
            &PrimRegistry::with_builtins(),
            &IdentitySwapEngine,
            32,
        );
        assert_eq!(r, Err(EvalError::FuelExhausted));
    }
}
