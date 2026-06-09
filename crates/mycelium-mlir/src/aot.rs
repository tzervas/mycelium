//! The runnable AOT artifact (M-150 "runnable artifact for the subset"; M-151 differential target).
//!
//! Executes the **lowered A-normal form** (`mycelium-core::lower`) with a **big-step env-machine**:
//! bindings are evaluated in order into an environment, operands are looked up, primitives and swaps
//! are applied via the shared registries. This is an *independent execution path* from the M-110
//! reference interpreter (small-step substitution over the nested tree), so checking the two for
//! observable equivalence (M-151) is a real NFR-7 two-path test — it stands in for "interpreter vs
//! compiled native" until the libMLIR backend lands.

use std::collections::HashMap;

use mycelium_core::lower::{self, Atom, Rhs};
use mycelium_core::{Node, Value};
use mycelium_interp::{EvalError, PrimRegistry, SwapEngine};

fn lookup(env: &HashMap<Atom, Value>, a: &Atom) -> Result<Value, EvalError> {
    env.get(a).cloned().ok_or_else(|| match a {
        Atom::Named(x) => EvalError::FreeVariable(x.clone()),
        Atom::Temp(k) => EvalError::FreeVariable(format!("%{k}")),
    })
}

/// Run a Core IR program through the AOT path: lower to ANF, then evaluate the bindings sequentially.
/// Returns the result [`Value`], or an explicit [`EvalError`] (never a silent failure).
pub fn run(node: &Node, prims: &PrimRegistry, swap: &dyn SwapEngine) -> Result<Value, EvalError> {
    let anf = lower::lower_to_anf(node);
    let mut env: HashMap<Atom, Value> = HashMap::new();
    for b in anf.bindings() {
        let value = match &b.rhs {
            Rhs::Const(v) => v.clone(),
            Rhs::Alias(a) => lookup(&env, a)?,
            Rhs::Op { prim, args } => {
                let vals: Vec<Value> = args
                    .iter()
                    .map(|a| lookup(&env, a))
                    .collect::<Result<_, _>>()?;
                let refs: Vec<&Value> = vals.iter().collect();
                let f = prims
                    .get(prim)
                    .ok_or_else(|| EvalError::UnknownPrim(prim.clone()))?;
                f(prim, &refs)?
            }
            Rhs::Swap {
                src,
                target,
                policy,
            } => {
                let s = lookup(&env, src)?;
                swap.swap(&s, target, policy)?
            }
        };
        env.insert(b.name.clone(), value);
    }
    lookup(&env, anf.result())
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
}
