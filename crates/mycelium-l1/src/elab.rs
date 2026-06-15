//! **Elaboration to the L0 Core IR** (RFC-0007 §4.6, **narrowed by RFC-0011 r3**). The
//! evaluation-complete fragment now reaches **algebraic data and matching**: `Construct` and the
//! flat `Match` are L0 nodes (RFC-0001 r3), so a program that builds/matches data — but does not
//! recurse (`Fix`) or call an unknown lambda (`App`) — elaborates to a closed L0 term. Anything
//! still outside the fragment is an explicit [`ElabError::Residual`] — **never a partial artifact**;
//! those programs run on the L1 fuel-guarded evaluator ([`crate::eval`]) instead.
//!
//! This module also owns the shared surface→kernel bridge the evaluator reuses, so the two
//! execution paths cannot drift on the basics: literal values ([`lit_value`]), representation
//! resolution ([`type_repr`]), and the v0 policy-name reference ([`policy_name_ref`]).
//!
//! # How a `match` lowers (RFC-0011 §4.4)
//! Nested surface patterns are compiled to the **flat** kernel `Match` by the **M-320 Maranget
//! decision tree** ([`crate::decision`]) — the untrusted, inspectable lowering. Each tree `Switch`
//! becomes an L0 `Match` on the occurrence's bound variable; each constructor case becomes an
//! `Alt::Ctor` binding *all* the constructor's fields (so every binder occurrence is available at
//! the leaf), and each leaf elaborates the surface arm's body with its binders mapped to those
//! field variables. `if` desugars to a `Match` on the prelude `Bool`. WF7 coverage is the checker's
//! (the tree is verified `Fail`-free before lowering — defense in depth, never silent).
//!
//! v0 scope, all refusals still explicit:
//! - recursion (`Fix`) and unknown application (`App`) have no L0 node yet (r4) — `Residual`;
//! - `for` desugars to a structural `Fix` — `Residual` (RFC-0007 §4.8);
//! - a guarantee index `@ g` is checked *dynamically* in v0 (RFC-0007 §4.3, stage 0) — `Residual`.

use std::collections::BTreeMap;

use mycelium_core::{
    operation_hash, Alt, CtorRef, CtorSpec, DataRegistry, DeclSpec, FieldSpec, Meta, Node, Payload,
    PolicyRef, Provenance, Repr, ScalarKind, Trit, Value,
};

use crate::ast::{Arm, BaseType, Expr, Literal, Path, Scalar, TypeRef};
use crate::checkty::{infer_type, normalize_pattern, prim_kernel_name, resolve_ty, Env, Ty};
use crate::decision::{self, Head, Tree};

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

/// A surface name's elaboration binding: `(surface name, kernel variable, v0 type)`. The type lets
/// the elaborator re-infer a `match` scrutinee's type (to lower its patterns) without a second
/// inference pass over the whole body.
type Binding = (String, String, Ty);

/// Elaborate the nullary function `entry` of a checked colony to a closed L0 [`Node`].
///
/// The fragment narrows as RFC-0011 r3 lands: **data construction and matching now elaborate** (to
/// `Construct`/`Match` L0 nodes). Still outside the fragment, all explicit [`ElabError::Residual`]:
/// recursion (`Fix`), unknown application (`App`), `for` (a structural `Fix`), and dynamic guarantee
/// indices. On success the result is a closed L0 term whose evaluation must agree with the L1
/// evaluator on the observable (NFR-7; RFC-0011 §4.4 — the M-210 differential).
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
    if let Some(g) = fd.sig.ret.guarantee {
        return residual(
            entry,
            format!(
                "the return guarantee index `@ {g:?}` is checked dynamically in v0 \
                 (RFC-0007 §4.3) — no L0 form"
            ),
        );
    }
    let registry = build_registry(env)?;
    let mut el = Elab {
        env,
        registry,
        fresh: 0,
    };
    let mut stack = vec![entry.to_owned()];
    el.expr(&mut stack, &[], &fd.body)
}

/// Build the content-addressed data registry `Σ` (RFC-0001 §4.3 r3) from the checked environment's
/// type declarations, so the elaborator can resolve constructor names to `#T#i` [`CtorRef`]s. A type
/// carrying a field outside the r3 data fragment (e.g. a `Substrate` field) is skipped; if a
/// *reachable* type references it, the registry build fails and the program is honestly `Residual`.
///
/// Public so a differential / a consumer can rebuild the **same** registry the elaborator used
/// (it is a pure, content-addressed function of `env.types`) — e.g. to map an L1 evaluator's
/// name-keyed data value onto the elaborated L0 value's `#T#i` identity (NFR-7).
pub fn build_registry(env: &Env) -> Result<DataRegistry, ElabError> {
    let mut specs: BTreeMap<String, DeclSpec> = BTreeMap::new();
    'types: for (name, d) in &env.types {
        let mut ctors = Vec::with_capacity(d.ctors.len());
        for c in &d.ctors {
            let mut fields = Vec::with_capacity(c.fields.len());
            for f in &c.fields {
                match field_spec(f) {
                    Some(fs) => fields.push(fs),
                    None => continue 'types, // a non-r3 field — skip this type (Residual if used)
                }
            }
            ctors.push(CtorSpec { fields });
        }
        specs.insert(name.clone(), DeclSpec { ctors });
    }
    DataRegistry::build(&specs).map_err(|e| ElabError::Residual {
        site: "<data registry>".to_owned(),
        what: format!("a reachable data type is outside the r3 fragment: {e}"),
    })
}

/// Convert a v0 field type to a registry [`FieldSpec`]; `None` for a type with no r3 value form.
fn field_spec(ty: &Ty) -> Option<FieldSpec> {
    Some(match ty {
        Ty::Binary(n) => FieldSpec::Repr(Repr::Binary { width: *n }),
        Ty::Ternary(m) => FieldSpec::Repr(Repr::Ternary { trits: *m }),
        Ty::Dense(d, s) => FieldSpec::Repr(Repr::Dense {
            dim: *d,
            dtype: scalar_kind(*s),
        }),
        Ty::Data(n) => FieldSpec::Data(n.clone()),
        Ty::Substrate(_) => return None,
    })
}

/// The `Scalar` → kernel `ScalarKind` mapping (shared with [`type_repr`]).
fn scalar_kind(s: Scalar) -> ScalarKind {
    match s {
        Scalar::F16 => ScalarKind::F16,
        Scalar::Bf16 => ScalarKind::Bf16,
        Scalar::F32 => ScalarKind::F32,
        Scalar::F64 => ScalarKind::F64,
    }
}

/// The elaboration context: the checked environment, the data registry `Σ`, and a fresh-name
/// counter for inlining and match-binder introduction.
struct Elab<'e> {
    env: &'e Env,
    registry: DataRegistry,
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

    /// The `#T#i` [`CtorRef`] for constructor `name`, resolved through the same `Env::ctor` lookup
    /// the checker uses (so the elaborator and the L1 evaluator agree on constructor identity).
    fn ctor_ref(&self, name: &str) -> Option<CtorRef> {
        let (d, i) = self.env.ctor(name)?;
        self.registry.ctor_ref(&d.name, u32::try_from(i).ok()?)
    }

    /// The surface→type view of `scope`, for re-inferring a scrutinee/bound type.
    fn ty_scope(scope: &[Binding]) -> Vec<(String, Ty)> {
        scope
            .iter()
            .map(|(s, _, t)| (s.clone(), t.clone()))
            .collect()
    }

    /// Elaborate `e` under `scope` (surface name → kernel variable + type). `stack` is the call
    /// path — the cycle (recursion) detector and the error site.
    fn expr(
        &mut self,
        stack: &mut Vec<String>,
        scope: &[Binding],
        e: &Expr,
    ) -> Result<Node, ElabError> {
        let site = stack.last().expect("stack starts with the entry").clone();
        let site = site.as_str();
        match e {
            Expr::Lit(l) => Ok(Node::Const(lit_value(site, l)?)),
            Expr::Path(p) => {
                if p.0.len() == 1 {
                    let name = &p.0[0];
                    if let Some((_, kvar, _)) = scope.iter().rev().find(|(s, _, _)| s == name) {
                        return Ok(Node::Var(kvar.clone()));
                    }
                    // A bare nullary constructor (Z, Nil, True, …) is a saturated Construct.
                    if self.env.ctor(name).is_some() {
                        let ctor = self.ctor_ref(name).ok_or_else(|| ElabError::Residual {
                            site: site.to_owned(),
                            what: format!("`{name}` is outside the r3 data registry"),
                        })?;
                        return Ok(Node::Construct { ctor, args: vec![] });
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
                // The bound's type (re-inferred) goes into scope so a later `match` on this binding
                // can lower its patterns.
                let bty = infer_type(self.env, &mut Self::ty_scope(scope), bound).map_err(|e| {
                    ElabError::Residual {
                        site: site.to_owned(),
                        what: format!("could not re-infer `let {name}`'s type: {e}"),
                    }
                })?;
                let kvar = self.fresh(name);
                let mut inner = scope.to_vec();
                inner.push((name.clone(), kvar.clone(), bty));
                let kbody = self.expr(stack, &inner, body)?;
                Ok(Node::Let {
                    id: kvar,
                    bound: Box::new(kbound),
                    body: Box::new(kbody),
                })
            }
            Expr::If { cond, conseq, alt } => self.elab_if(stack, scope, cond, conseq, alt),
            Expr::Match { scrutinee, arms } => self.elab_match(stack, scope, scrutinee, arms),
            Expr::For { .. } => residual(
                site,
                "`for` elaborates to a structurally recursive fold (Fix) — outside the \
                 evaluation-complete fragment (RFC-0007 §4.8)",
            ),
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

    /// `if c then t else e` desugars to a flat `Match` on the prelude `Bool` (RFC-0007 §4.4; the
    /// constructors `True`/`False` come from the same registry the surface checks against).
    fn elab_if(
        &mut self,
        stack: &mut Vec<String>,
        scope: &[Binding],
        cond: &Expr,
        conseq: &Expr,
        alt: &Expr,
    ) -> Result<Node, ElabError> {
        let site = stack.last().expect("non-empty").clone();
        let cond_node = self.expr(stack, scope, cond)?;
        let true_ref = self.bool_ctor(&site, "True")?;
        let false_ref = self.bool_ctor(&site, "False")?;
        let conseq_node = self.expr(stack, scope, conseq)?;
        let alt_node = self.expr(stack, scope, alt)?;
        let cond_var = self.fresh("cond");
        let m = Node::Match {
            scrutinee: Box::new(Node::Var(cond_var.clone())),
            alts: vec![
                Alt::Ctor {
                    ctor: true_ref,
                    binders: vec![],
                    body: conseq_node,
                },
                Alt::Ctor {
                    ctor: false_ref,
                    binders: vec![],
                    body: alt_node,
                },
            ],
            default: None,
        };
        Ok(Node::Let {
            id: cond_var,
            bound: Box::new(cond_node),
            body: Box::new(m),
        })
    }

    fn bool_ctor(&self, site: &str, name: &str) -> Result<CtorRef, ElabError> {
        self.ctor_ref(name).ok_or_else(|| ElabError::Residual {
            site: site.to_owned(),
            what: format!("the prelude `Bool` constructor `{name}` is missing from the registry"),
        })
    }

    /// Lower a surface `match` to the flat L0 `Match` via the M-320 Maranget decision tree
    /// (RFC-0011 §4.4). Re-infers the scrutinee type, normalises each arm pattern (collecting binder
    /// occurrences), compiles the verified-`Fail`-free decision tree, and threads it into nested L0
    /// `Match` nodes — binding the scrutinee once in an enclosing `Let`.
    fn elab_match(
        &mut self,
        stack: &mut Vec<String>,
        scope: &[Binding],
        scrutinee: &Expr,
        arms: &[Arm],
    ) -> Result<Node, ElabError> {
        let site = stack.last().expect("non-empty").clone();
        // 1. Re-infer the scrutinee type (the checker validated it; this recomputes it for lowering).
        let sty = infer_type(self.env, &mut Self::ty_scope(scope), scrutinee).map_err(|e| {
            ElabError::Residual {
                site: site.clone(),
                what: format!("could not re-infer the match scrutinee's type: {e}"),
            }
        })?;
        // 2. Elaborate the scrutinee and bind it once (a Match tests sub-values of one value).
        let scrut_node = self.expr(stack, scope, scrutinee)?;
        let scrut_var = self.fresh("scrut");
        // 3. Normalise every arm's pattern → the coverage matrix + per-arm binder occurrences.
        let mut matrix: Vec<Vec<crate::usefulness::Pat>> = Vec::with_capacity(arms.len());
        let mut arm_binders: Vec<Vec<(String, Ty, Vec<usize>)>> = Vec::with_capacity(arms.len());
        for arm in arms {
            let mut binds = Vec::new();
            let pat =
                normalize_pattern(&self.env.types, &site, &arm.pattern, &sty, &[], &mut binds)
                    .map_err(|e| ElabError::Residual {
                        site: site.clone(),
                        what: format!("could not normalise a match pattern: {e}"),
                    })?;
            matrix.push(vec![pat]);
            arm_binders.push(binds);
        }
        // 4. Compile (and re-verify Fail-free) the Maranget decision tree — the untrusted lowering.
        let arm_ix: Vec<usize> = (0..arms.len()).collect();
        let occ_root = [Vec::<usize>::new()];
        let tree = decision::compile(&self.env.types, &matrix, &arm_ix, &occ_root, &[sty]);
        if decision::has_reachable_fail(&tree) {
            return residual(
                &site,
                "the match compiled to a decision tree with a reachable Fail (usefulness and the \
                 Maranget compiler disagree) — refusing to emit an unsound L0 Match",
            );
        }
        // 5. Lower the tree to nested L0 Match nodes; the root occurrence is the bound scrutinee.
        let mut occ_map: BTreeMap<Vec<usize>, String> = BTreeMap::new();
        occ_map.insert(Vec::new(), scrut_var.clone());
        let body = self.lower_tree(stack, scope, &tree, &occ_map, arms, &arm_binders)?;
        Ok(Node::Let {
            id: scrut_var,
            bound: Box::new(scrut_node),
            body: Box::new(body),
        })
    }

    /// Lower a Maranget [`Tree`] into nested L0 `Match` nodes. `occ_map` maps each already-bound
    /// occurrence (a path into the scrutinee) to its kernel variable; a `Switch` matches on the
    /// occurrence's variable, a constructor case binds *all* its fields (extending `occ_map`), and a
    /// leaf elaborates the surface arm body with its binders resolved through `occ_map`.
    fn lower_tree(
        &mut self,
        stack: &mut Vec<String>,
        scope: &[Binding],
        tree: &Tree,
        occ_map: &BTreeMap<Vec<usize>, String>,
        arms: &[Arm],
        arm_binders: &[Vec<(String, Ty, Vec<usize>)>],
    ) -> Result<Node, ElabError> {
        let site = stack.last().expect("non-empty").clone();
        match tree {
            Tree::Leaf(i) => {
                // Bind the arm's pattern binders to the kernel variables at their occurrences, then
                // elaborate the arm body in that extended scope.
                let mut arm_scope = scope.to_vec();
                for (name, ty, occ) in &arm_binders[*i] {
                    let kvar = occ_map.get(occ).ok_or_else(|| ElabError::Residual {
                        site: site.clone(),
                        what: format!(
                            "internal: binder `{name}` at occurrence {occ:?} was not bound by the \
                             decision tree"
                        ),
                    })?;
                    arm_scope.push((name.clone(), kvar.clone(), ty.clone()));
                }
                self.expr(stack, &arm_scope, &arms[*i].body)
            }
            Tree::Fail => residual(
                &site,
                "internal: the decision tree reached a Fail (a checked-exhaustive match must not)",
            ),
            Tree::Switch {
                occurrence,
                cases,
                default,
            } => {
                let scrut_kvar =
                    occ_map
                        .get(occurrence)
                        .cloned()
                        .ok_or_else(|| ElabError::Residual {
                            site: site.clone(),
                            what: format!(
                                "internal: switch occurrence {occurrence:?} is not bound"
                            ),
                        })?;
                let mut alts = Vec::with_capacity(cases.len());
                for (head, subtree) in cases {
                    match head {
                        Head::Ctor(name, arity) => {
                            let ctor = self.ctor_ref(name).ok_or_else(|| ElabError::Residual {
                                site: site.clone(),
                                what: format!("`{name}` is outside the r3 data registry"),
                            })?;
                            // Bind ALL fields (not just the discriminated ones) so every binder
                            // occurrence below is available at the leaf.
                            let binders: Vec<String> =
                                (0..*arity).map(|_| self.fresh(name)).collect();
                            let mut child_map = occ_map.clone();
                            for (j, b) in binders.iter().enumerate() {
                                let mut child = occurrence.clone();
                                child.push(j);
                                child_map.insert(child, b.clone());
                            }
                            let body = self.lower_tree(
                                stack,
                                scope,
                                subtree,
                                &child_map,
                                arms,
                                arm_binders,
                            )?;
                            alts.push(Alt::Ctor {
                                ctor,
                                binders,
                                body,
                            });
                        }
                        Head::Lit(key) => {
                            let value = lit_key_to_value(&site, key)?;
                            let body =
                                self.lower_tree(stack, scope, subtree, occ_map, arms, arm_binders)?;
                            alts.push(Alt::Lit { value, body });
                        }
                    }
                }
                let default_node = match default {
                    Some(d) => Some(Box::new(self.lower_tree(
                        stack,
                        scope,
                        d,
                        occ_map,
                        arms,
                        arm_binders,
                    )?)),
                    None => None,
                };
                Ok(Node::Match {
                    scrutinee: Box::new(Node::Var(scrut_kvar)),
                    alts,
                    default: default_node,
                })
            }
        }
    }

    /// Elaborate an application: prims become `Op` nodes; saturated constructors become `Construct`
    /// nodes; user-function calls **inline** (the fragment's call graph is acyclic, so inlining
    /// terminates); recursion is an explicit `Residual` (Fix, r4).
    fn app(
        &mut self,
        stack: &mut Vec<String>,
        scope: &[Binding],
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
                let pty = resolve_ty(site, &self.env.types, &param.ty)
                    .map(|(t, _)| t)
                    .map_err(|e| ElabError::Residual {
                        site: site.to_owned(),
                        what: format!("could not resolve `{name}`'s parameter type: {e}"),
                    })?;
                bindings.push((param.name.clone(), self.fresh(&param.name), karg, pty));
            }
            let callee_scope: Vec<Binding> = bindings
                .iter()
                .map(|(s, k, _, t)| (s.clone(), k.clone(), t.clone()))
                .collect();
            stack.push(name.clone());
            let body = self.expr(stack, &callee_scope, &fd.body)?;
            stack.pop();
            // Wrap right-to-left so the leftmost argument's Let is outermost (evaluated first).
            let node = bindings
                .into_iter()
                .rev()
                .fold(body, |acc, (_, kvar, karg, _)| Node::Let {
                    id: kvar,
                    bound: Box::new(karg),
                    body: Box::new(acc),
                });
            return Ok(node);
        }

        // A saturated constructor application builds a data value (W6 saturation is already checked).
        if self.env.ctor(name).is_some() {
            let ctor = self.ctor_ref(name).ok_or_else(|| ElabError::Residual {
                site: site.to_owned(),
                what: format!("`{name}` is outside the r3 data registry"),
            })?;
            let mut kargs = Vec::with_capacity(args.len());
            for a in args {
                kargs.push(self.expr(stack, scope, a)?);
            }
            return Ok(Node::Construct { ctor, args: kargs });
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

/// Reconstruct the L0 [`Value`] of a literal-pattern key (`b:1010` / `t:+0-`) produced by the
/// checker's `literal_key` (the `_` separators already normalised away). The width is the digit
/// count (Q6: a literal *is* its representation).
fn lit_key_to_value(site: &str, key: &str) -> Result<Value, ElabError> {
    if let Some(bits) = key.strip_prefix("b:") {
        let bits: Vec<bool> = bits.chars().map(|c| c == '1').collect();
        let width = u32::try_from(bits.len()).expect("digit count fits u32");
        Value::new(
            Repr::Binary { width },
            Payload::Bits(bits),
            Meta::exact(Provenance::Root),
        )
        .map_or_else(
            |e| residual(site, format!("malformed binary literal key: {e}")),
            Ok,
        )
    } else if let Some(trits) = key.strip_prefix("t:") {
        let trits: Vec<Trit> = trits
            .chars()
            .map(|c| match c {
                '+' => Ok(Trit::Pos),
                '0' => Ok(Trit::Zero),
                '-' => Ok(Trit::Neg),
                other => Err(other),
            })
            .collect::<Result<_, _>>()
            .map_or_else(
                |c| residual(site, format!("non-trit char {c:?} in ternary literal key")),
                Ok,
            )?;
        let width = u32::try_from(trits.len()).expect("trit count fits u32");
        Value::new(
            Repr::Ternary { trits: width },
            Payload::Trits(trits),
            Meta::exact(Provenance::Root),
        )
        .map_or_else(
            |e| residual(site, format!("malformed ternary literal key: {e}")),
            Ok,
        )
    } else {
        residual(site, format!("unrecognised literal key `{key}`"))
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
    fn a_match_now_elaborates_to_l0_and_runs() {
        // r3: `match` is no longer Residual — it lowers to a flat L0 Match and runs on the
        // reference interpreter. `match Pos { Neg => <->, Zero => <0>, _ => <+> }` ⟶ <+>.
        let env = env(
            "colony d\ntype Sign = Neg | Zero | Pos\nfn main() -> Ternary{1} =\n  match Pos { Neg => <->, Zero => <0>, _ => <+> }",
        );
        let node = elaborate(&env, "main").expect("match elaborates in r3");
        let v = mycelium_interp::Interpreter::default()
            .eval(&node)
            .expect("runs");
        assert_eq!(v.payload(), &Payload::Trits(vec![Trit::Pos]));
    }

    #[test]
    fn a_data_value_now_elaborates_to_construct() {
        // A program returning a data value elaborates to Construct (via eval_core).
        let env = env("colony d\ntype Nat = Z | S(Nat)\nfn main() -> Nat = S(Z)");
        let node = elaborate(&env, "main").expect("Construct elaborates in r3");
        let v = mycelium_interp::Interpreter::default()
            .eval_core(&node)
            .expect("runs");
        let d = v.as_data().expect("a data value");
        assert_eq!(d.fields().len(), 1, "S(Z) has one field");
    }

    #[test]
    fn an_if_desugars_to_a_bool_match() {
        // `if` lowers to a Match on the prelude Bool — exercises the True/False registry path.
        let env = env(
            "colony d\nfn pick(b: Bool) -> Binary{8} = if b then 0b1111_1111 else 0b0000_0000\n\
             fn main() -> Binary{8} = pick(True)",
        );
        let node = elaborate(&env, "main").expect("if elaborates in r3");
        let v = mycelium_interp::Interpreter::default()
            .eval(&node)
            .expect("runs");
        assert_eq!(v.payload(), &Payload::Bits(vec![true; 8]));
    }

    #[test]
    fn a_nested_pattern_match_elaborates_and_runs() {
        // pred2 uses depth-2 nested patterns; the Maranget tree lowers them to nested flat L0 Matches.
        // pred2(S(S(S(Z)))) ⟶ S(Z).
        let env = env("colony d\ntype Nat = Z | S(Nat)\n\
             fn pred2(n: Nat) -> Nat = match n { Z => Z, S(Z) => Z, S(S(m)) => m }\n\
             fn main() -> Nat = pred2(S(S(S(Z))))");
        let node = elaborate(&env, "main").expect("nested match elaborates in r3");
        let v = mycelium_interp::Interpreter::default()
            .eval_core(&node)
            .expect("runs");
        let d = v.as_data().expect("a data value");
        assert_eq!(d.fields().len(), 1, "S(Z)");
        assert_eq!(
            d.fields()[0].as_data().expect("inner Z").fields().len(),
            0,
            "the inner value is Z"
        );
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
    fn a_for_fold_is_an_explicit_residual() {
        // `for` desugars to Fix — outside the evaluation-complete fragment (RFC-0007 §4.8).
        let env = env("colony d\ntype Bytes = End | More(Binary{8}, Bytes)\n\
             fn main() -> Binary{8} = for b in End, acc = 0b0000_0000 => xor(acc, b)");
        let err = elaborate(&env, "main").unwrap_err();
        let ElabError::Residual { what, .. } = &err else {
            panic!("expected Residual, got {err:?}");
        };
        assert!(what.contains("for"), "got: {what}");
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
