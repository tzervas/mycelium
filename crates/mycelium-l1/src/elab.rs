//! **Elaboration to the L0 Core IR** on the *evaluation-complete fragment* (RFC-0007 §4.6):
//! definitions whose call graph is acyclic and whose bodies, after inlining, contain only
//! `Const/Var/Let/Op/Swap` residue. Elaborating anything outside the fragment is an explicit
//! [`ElabError::Residual`] — **never a partial artifact** (RFC-0007 §4.6); those programs run on
//! the L1 fuel-guarded evaluator ([`crate::eval`]) instead.
//!
//! This module also owns the shared surface→kernel bridge the evaluator reuses, so the two
//! execution paths cannot drift on the basics: literal values ([`lit_value`]), representation
//! resolution ([`type_repr`]), and the v0 policy-name reference ([`policy_name_ref`]).
//!
//! v0 scope, all refusals explicit:
//! - `Match`/`if`, `Construct` (data values), and recursion (`Fix`) have no L0 node — `Residual`;
//! - a guarantee index `@ g` is checked *dynamically* in v0 (RFC-0007 §4.3, stage 0) and has no
//!   L0 form — `Residual`;
//! - inlining is the only normalization performed; the fragment's simply-typed, `Fix`-free terms
//!   need nothing more for `Const/Var/Let/Op/Swap` bodies (RFC-0007 §4.6).

use mycelium_core::{
    operation_hash, Meta, Node, Payload, PolicyRef, Provenance, Repr, ScalarKind, Trit, Value,
};

use crate::ast::{BaseType, Expr, Literal, Path, Scalar, TypeRef};
use crate::checkty::{prim_kernel_name, Env};

/// Why a definition could not be elaborated to L0 — always explicit, never a partial artifact
/// (RFC-0007 §4.6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElabError {
    /// The body (or something it calls) is outside the evaluation-complete fragment; the program
    /// still *runs* — on the L1 fuel-guarded evaluator (RFC-0007 §4.6).
    Residual {
        /// The definition being elaborated when the refusal arose.
        site: String,
        /// Which construct fell outside the fragment, and why.
        what: String,
    },
    /// The requested entry definition does not exist in the checked environment.
    UnknownFn(String),
}

impl core::fmt::Display for ElabError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ElabError::Residual { site, what } => write!(
                f,
                "`{site}` is outside the evaluation-complete fragment (RFC-0007 §4.6): {what} — \
                 run it on the L1 evaluator"
            ),
            ElabError::UnknownFn(name) => write!(f, "no function `{name}` in the checked colony"),
        }
    }
}

impl std::error::Error for ElabError {}

fn residual<T>(site: &str, what: impl Into<String>) -> Result<T, ElabError> {
    Err(ElabError::Residual {
        site: site.to_owned(),
        what: what.into(),
    })
}

/// Build the L0 [`Value`] of a representation literal (Q6: a literal *is* its representation —
/// width = digit count). Bare integers and lists have no representation family and are refused
/// (the typechecker already refuses them; this refusal keeps the bridge honest on its own).
pub fn lit_value(site: &str, l: &Literal) -> Result<Value, ElabError> {
    match l {
        Literal::Bin(s) => {
            let bits: Vec<bool> = s
                .chars()
                .filter(|c| *c == '0' || *c == '1')
                .map(|c| c == '1')
                .collect();
            let width = u32::try_from(bits.len()).expect("digit count fits u32");
            Value::new(
                Repr::Binary { width },
                Payload::Bits(bits),
                Meta::exact(Provenance::Root),
            )
            .map_or_else(
                |e| residual(site, format!("malformed binary literal: {e}")),
                Ok,
            )
        }
        Literal::Trit(s) => {
            let trits: Vec<Trit> = s
                .chars()
                .map(|c| match c {
                    '+' => Ok(Trit::Pos),
                    '0' => Ok(Trit::Zero),
                    '-' => Ok(Trit::Neg),
                    other => Err(other),
                })
                .collect::<Result<_, _>>()
                .map_or_else(
                    |c| residual(site, format!("non-trit char {c:?} in ternary literal")),
                    Ok,
                )?;
            let width = u32::try_from(trits.len()).expect("trit count fits u32");
            Value::new(
                Repr::Ternary { trits: width },
                Payload::Trits(trits),
                Meta::exact(Provenance::Root),
            )
            .map_or_else(
                |e| residual(site, format!("malformed ternary literal: {e}")),
                Ok,
            )
        }
        Literal::Int(_) => residual(
            site,
            "a bare integer literal has no representation family (Q6)",
        ),
        Literal::List(_) => residual(site, "list literals are deferred in v0"),
    }
}

/// Resolve a surface [`TypeRef`] to a kernel [`Repr`] (swap targets). Only representation types
/// resolve; named/data, VSA, and `Substrate` types are explicit refusals.
pub fn type_repr(site: &str, t: &TypeRef) -> Result<Repr, ElabError> {
    match &t.base {
        BaseType::Binary(n) => Ok(Repr::Binary { width: *n }),
        BaseType::Ternary(m) => Ok(Repr::Ternary { trits: *m }),
        BaseType::Dense(d, s) => Ok(Repr::Dense {
            dim: *d,
            dtype: match s {
                Scalar::F16 => ScalarKind::F16,
                Scalar::Bf16 => ScalarKind::Bf16,
                Scalar::F32 => ScalarKind::F32,
                Scalar::F64 => ScalarKind::F64,
            },
        }),
        BaseType::Vsa { .. } => residual(site, "VSA types are deferred in the L1 v0 prototype"),
        BaseType::Substrate(tag) => residual(
            site,
            format!("Substrate{{{tag}}} is not a representation type"),
        ),
        BaseType::Named(name, _) => residual(
            site,
            format!("`{name}` is not a representation type — no kernel Repr"),
        ),
    }
}

/// The v0 **policy-name reference**: a deterministic, domain-separated content address derived
/// from the surface policy *name* (`policy: roundtrip`).
///
/// Honesty note (Declared): RFC-0005 policy *objects* are content-addressed over their canonical
/// serialization (`mycelium-select::SelectionPolicy::policy_ref`); binding surface names to
/// registered policy objects is later integration work. Until it lands, this name-derived address
/// keeps `Meta.policy_used` answerable and — because the evaluator and the elaborator share this
/// one function — keeps every execution path's swaps on the *same* `PolicyRef`, so the NFR-7
/// differential is meaningful. Domain-separated (`policy-name.v0:`) so it can never collide with
/// a structural or operation hash.
#[must_use]
pub fn policy_name_ref(policy: &Path) -> PolicyRef {
    operation_hash(&format!("policy-name.v0:{}", policy.0.join(".")))
}

/// Elaborate the nullary function `entry` of a checked colony to a closed L0 [`Node`].
///
/// The fragment check is by construction: recursion (a call cycle through `entry`), `Match`/`if`,
/// data construction, and dynamic guarantee indices all return [`ElabError::Residual`]. On
/// success the resulting node is a closed `Const/Var/Let/Op/Swap` term whose evaluation must
/// agree with the L1 evaluator and the AOT path on the observable (NFR-7; RFC-0007 §4.6).
pub fn elaborate(env: &Env, entry: &str) -> Result<Node, ElabError> {
    let Some(fd) = env.fns.get(entry) else {
        return Err(ElabError::UnknownFn(entry.to_owned()));
    };
    if !fd.sig.value_params.is_empty() {
        return residual(
            entry,
            "the entry has value parameters — v0 elaborates closed (nullary) entries; \
             apply it from a nullary definition",
        );
    }
    let mut el = Elab { env, fresh: 0 };
    let mut stack = vec![entry.to_owned()];
    if let Some(g) = fd.sig.ret.guarantee {
        return residual(
            entry,
            format!(
                "the return guarantee index `@ {g:?}` is checked dynamically in v0 \
                 (RFC-0007 §4.3) — no L0 form"
            ),
        );
    }
    el.expr(&mut stack, &[], &fd.body)
}

/// The elaboration context: the checked environment plus a fresh-name counter for inlining.
struct Elab<'e> {
    env: &'e Env,
    fresh: u32,
}

impl Elab<'_> {
    /// A fresh kernel variable for surface name `base`. `%` is not an identifier character in the
    /// surface lexer, so fresh names can never capture or collide with surface binders.
    fn fresh(&mut self, base: &str) -> String {
        let n = self.fresh;
        self.fresh += 1;
        format!("{base}%{n}")
    }

    /// Elaborate `e` under `scope` (surface name → kernel variable). `stack` is the call path —
    /// the cycle (recursion) detector and the error site.
    fn expr(
        &mut self,
        stack: &mut Vec<String>,
        scope: &[(String, String)],
        e: &Expr,
    ) -> Result<Node, ElabError> {
        let site = stack.last().expect("stack starts with the entry").clone();
        let site = site.as_str();
        match e {
            Expr::Lit(l) => Ok(Node::Const(lit_value(site, l)?)),
            Expr::Path(p) => {
                if p.0.len() == 1 {
                    let name = &p.0[0];
                    if let Some((_, kvar)) = scope.iter().rev().find(|(s, _)| s == name) {
                        return Ok(Node::Var(kvar.clone()));
                    }
                    if self.env.ctor(name).is_some() {
                        return residual(
                            site,
                            format!(
                                "`{name}` constructs a data value — `Construct` has no L0 node"
                            ),
                        );
                    }
                }
                residual(site, format!("unresolved name `{}`", p.0.join(".")))
            }
            Expr::Let {
                name,
                ty,
                bound,
                body,
            } => {
                if let Some(g) = ty.as_ref().and_then(|t| t.guarantee) {
                    return residual(
                        site,
                        format!("the guarantee index `@ {g:?}` is checked dynamically in v0 — no L0 form"),
                    );
                }
                let kbound = self.expr(stack, scope, bound)?;
                let kvar = self.fresh(name);
                let mut inner = scope.to_vec();
                inner.push((name.clone(), kvar.clone()));
                let kbody = self.expr(stack, &inner, body)?;
                Ok(Node::Let {
                    id: kvar,
                    bound: Box::new(kbound),
                    body: Box::new(kbody),
                })
            }
            Expr::If { .. } => residual(site, "`if` elaborates to `Match`, which has no L0 node"),
            Expr::Match { .. } => residual(site, "`Match` has no L0 node"),
            Expr::Swap {
                value,
                target,
                policy,
            } => {
                if let Some(g) = target.guarantee {
                    return residual(
                        site,
                        format!("the guarantee index `@ {g:?}` is checked dynamically in v0 — no L0 form"),
                    );
                }
                let src = self.expr(stack, scope, value)?;
                Ok(Node::Swap {
                    src: Box::new(src),
                    target: type_repr(site, target)?,
                    policy: policy_name_ref(policy),
                })
            }
            Expr::Wild(_) => residual(site, "`wild` is denied by default (LR-9)"),
            Expr::Spore(_) => residual(site, "`spore` is deferred (E2-5/M-260)"),
            Expr::Ascribe(inner, t) => {
                if let Some(g) = t.guarantee {
                    return residual(
                        site,
                        format!("the guarantee index `@ {g:?}` is checked dynamically in v0 — no L0 form"),
                    );
                }
                // The type part is static and already checked — elaboration is transparent.
                self.expr(stack, scope, inner)
            }
            Expr::App { head, args } => self.app(stack, scope, head, args),
        }
    }

    /// Elaborate an application: prims become `Op` nodes; user-function calls **inline** (the
    /// fragment's call graph is acyclic, so inlining terminates); constructors are `Residual`.
    fn app(
        &mut self,
        stack: &mut Vec<String>,
        scope: &[(String, String)],
        head: &Expr,
        args: &[Expr],
    ) -> Result<Node, ElabError> {
        let site = stack.last().expect("non-empty").clone();
        let site = site.as_str();
        let Expr::Path(p) = head else {
            return residual(site, "v0 application head must be a name (first-order)");
        };
        if p.0.len() != 1 {
            return residual(site, format!("dotted call `{}`", p.0.join(".")));
        }
        let name = &p.0[0];

        if let Some(fd) = self.env.fns.get(name) {
            // Recursion = a cycle through the call stack → Fix → outside the fragment.
            if stack.iter().any(|f| f == name) {
                return residual(
                    site,
                    format!(
                        "`{name}` is recursive (Fix) — outside the evaluation-complete fragment"
                    ),
                );
            }
            if let Some(g) = fd.sig.ret.guarantee {
                return residual(
                    site,
                    format!(
                        "`{name}` asserts `@ {g:?}` on its result — checked dynamically in v0, no L0 form"
                    ),
                );
            }
            // Inline: Let-bind each argument left-to-right (preserving CBV evaluation order),
            // then elaborate the callee body with its parameters mapped to the fresh binders.
            // The callee sees *only* its parameters (top-level functions close over nothing).
            let mut bindings = Vec::new();
            for (param, arg) in fd.sig.value_params.iter().zip(args) {
                if let Some(g) = param.ty.guarantee {
                    return residual(
                        site,
                        format!(
                            "`{name}` parameter `{}` asserts `@ {g:?}` — checked dynamically in v0, no L0 form",
                            param.name
                        ),
                    );
                }
                let karg = self.expr(stack, scope, arg)?;
                bindings.push((param.name.clone(), self.fresh(&param.name), karg));
            }
            let callee_scope: Vec<(String, String)> = bindings
                .iter()
                .map(|(s, k, _)| (s.clone(), k.clone()))
                .collect();
            stack.push(name.clone());
            let body = self.expr(stack, &callee_scope, &fd.body)?;
            stack.pop();
            // Wrap right-to-left so the leftmost argument's Let is outermost (evaluated first).
            let node = bindings
                .into_iter()
                .rev()
                .fold(body, |acc, (_, kvar, karg)| Node::Let {
                    id: kvar,
                    bound: Box::new(karg),
                    body: Box::new(acc),
                });
            return Ok(node);
        }

        if self.env.ctor(name).is_some() {
            return residual(
                site,
                format!("`{name}` constructs a data value — `Construct` has no L0 node"),
            );
        }

        if let Some(kernel) = prim_kernel_name(name) {
            let mut kargs = Vec::new();
            for a in args {
                kargs.push(self.expr(stack, scope, a)?);
            }
            return Ok(Node::Op {
                prim: kernel.to_owned(),
                args: kargs,
            });
        }

        residual(site, format!("unknown function/constructor/prim `{name}`"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checkty::check_colony;
    use crate::parse;

    fn env(src: &str) -> Env {
        check_colony(&parse(src).expect("parses")).expect("checks")
    }

    #[test]
    fn a_const_let_op_swap_body_elaborates_closed() {
        let env = env(
            "colony d\nfn main() -> Ternary{6} =\n  let a = 0b1011_0010 in swap(not(a), to: Ternary{6}, policy: rt)",
        );
        let node = elaborate(&env, "main").expect("in the fragment");
        // Closed: the interpreter must not hit a free variable.
        let interp = mycelium_interp::Interpreter::default();
        // The identity engine refuses the cross-paradigm swap, but the term itself is closed and
        // well-formed — getting an UnsupportedSwap (not FreeVariable) proves closure.
        let err = interp.eval(&node).unwrap_err();
        assert!(matches!(
            err,
            mycelium_interp::EvalError::UnsupportedSwap { .. }
        ));
    }

    #[test]
    fn a_call_is_inlined_acyclically() {
        let env = env(
            "colony d\nfn flip(x: Binary{8}) -> Binary{8} = not(x)\nfn main() -> Binary{8} = flip(flip(0b1010_1010))",
        );
        let node = elaborate(&env, "main").expect("acyclic calls inline");
        let v = mycelium_interp::Interpreter::default()
            .eval(&node)
            .expect("runs");
        // not(not(x)) == x
        assert_eq!(
            v.payload(),
            &Payload::Bits(vec![true, false, true, false, true, false, true, false])
        );
    }

    #[test]
    fn recursion_is_an_explicit_residual() {
        let env = env(
            "colony d\nfn spin(x: Binary{8}) -> Binary{8} = spin(x)\nfn main() -> Binary{8} = spin(0b0000_0001)",
        );
        let err = elaborate(&env, "main").unwrap_err();
        let ElabError::Residual { what, .. } = &err else {
            panic!("expected Residual, got {err:?}");
        };
        assert!(what.contains("recursive"), "got: {what}");
    }

    #[test]
    fn match_is_an_explicit_residual() {
        let env = env(
            "colony d\ntype Sign = Neg | Zero | Pos\nfn main() -> Ternary{1} =\n  match Pos { Neg => <->, Zero => <0>, _ => <+> }",
        );
        let err = elaborate(&env, "main").unwrap_err();
        assert!(matches!(err, ElabError::Residual { .. }), "got {err:?}");
    }

    #[test]
    fn a_guarantee_index_is_an_explicit_residual() {
        let env = env(
            "colony d\nfn main() -> Ternary{6} @ Proven = swap(0b0000_0010, to: Ternary{6}, policy: rt)",
        );
        let err = elaborate(&env, "main").unwrap_err();
        let ElabError::Residual { what, .. } = &err else {
            panic!("expected Residual, got {err:?}");
        };
        assert!(what.contains("guarantee index"), "got: {what}");
    }

    #[test]
    fn the_entry_must_be_nullary() {
        let env = env("colony d\nfn id(x: Binary{8}) -> Binary{8} = x");
        let err = elaborate(&env, "id").unwrap_err();
        assert!(matches!(err, ElabError::Residual { .. }));
    }

    #[test]
    fn the_policy_name_ref_is_deterministic_and_name_sensitive() {
        let a = policy_name_ref(&Path(vec!["rt".into()]));
        let b = policy_name_ref(&Path(vec!["rt".into()]));
        let c = policy_name_ref(&Path(vec!["other".into()]));
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
