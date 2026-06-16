//! **Elaboration to the L0 Core IR** (RFC-0007 §4.6, **retired by RFC-0001 r4**). The
//! evaluation-complete fragment is now the **whole v0 calculus**: representation ops (L0), data +
//! matching (r3, `Construct`/flat `Match`), and **functions + recursion** (r4, `Lam`/`App`/`Fix`).
//! So a self-recursive, data-building, matching program elaborates to a closed L0 term. The only
//! explicit [`ElabError::Residual`]s left are **mutual recursion** (deferred, R7-Q3) and a **dynamic
//! guarantee index** `@ g` (RFC-0007 §4.3, stage 0) — never a partial artifact; those run on the L1
//! fuel-guarded evaluator ([`crate::eval`]) instead.
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
//! # How recursion lowers (RFC-0001 r4)
//! Each reachable **self-recursive** function is bound once as `let f = Fix(f, λparams. body)`
//! (callee-first), and a call to it becomes a curried `App` on its `Fix` variable; every **other**
//! call still inlines (the non-recursive call graph is acyclic). `for` desugars to a synthesized
//! self-recursive `Fix` fold over the linear spine (RFC-0007 §4.8). **Mutual recursion** (a cycle
//! through ≥2 distinct functions) is an explicit `Residual` (deferred, R7-Q3).

use std::collections::{BTreeMap, BTreeSet};

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
        Literal::AmbientInt(_, _) => residual(
            site,
            "internal: an unresolved ambient bare decimal reached elaboration — the checker \
             resolves its width before the L0 bridge runs (RFC-0012 §4.3)",
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
        BaseType::Ambient(_) => residual(
            site,
            "internal: an unresolved paradigm-less repr `{…}` reached elaboration — the ambient \
             resolution pass fills it first (RFC-0012 §4.3)",
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
/// As of RFC-0001 r4 the evaluation-complete fragment is the **whole v0 calculus**: data + matching
/// (r3) and now **functions + recursion** (`Lam`/`App`/`Fix`). Each reachable **self-recursive**
/// function is bound once as `let f = Fix(f, λparams. body)` (callee-first), and a call to it
/// elaborates to a curried `App`; every other call still inlines (the non-recursive call graph is
/// acyclic). **Mutual recursion** is an explicit `Residual` (deferred — R7-Q3). Still `Residual`: a
/// dynamic guarantee index `@ g` (RFC-0007 §4.3, stage 0). On success the result is a closed L0 term
/// whose evaluation must agree with the L1 evaluator (NFR-7; the M-210 differential).
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
    // The reachable self-recursive functions, callee-first; mutual recursion is refused here.
    let rec_order = recursive_order(env, entry)?;
    let mut el = Elab {
        env,
        registry,
        fresh: 0,
        rec: BTreeMap::new(),
    };
    // Assign each recursive function a kernel Fix variable (in scope for every recursive body and
    // the entry body), then elaborate each Fix(λparams. body) and the entry, and wrap callee-first.
    for f in &rec_order {
        let kf = el.fresh(f);
        el.rec.insert(f.clone(), kf);
    }
    let mut fixes: Vec<(String, Node)> = Vec::with_capacity(rec_order.len());
    for f in &rec_order {
        let kf = el.rec[f].clone();
        let fix = el.elab_recursive_fn(f, &kf)?;
        fixes.push((kf, fix));
    }
    let mut stack = vec![entry.to_owned()];
    let entry_body = el.expr(&mut stack, &[], &fd.body)?;
    // `fixes` is callee-first; fold in reverse so the first (callee) Let ends up outermost.
    let node = fixes
        .into_iter()
        .rev()
        .fold(entry_body, |acc, (kf, fix)| Node::Let {
            id: kf,
            bound: Box::new(fix),
            body: Box::new(acc),
        });
    Ok(node)
}

/// The reachable **self-recursive** functions from `entry`, ordered **callee-first** (a recursive
/// function that calls another recursive function is bound *inside* it). **Mutual recursion** — a
/// call cycle through two or more *distinct* functions — is an explicit [`ElabError::Residual`]
/// (deferred, R7-Q3); only direct self-recursion is enacted in v0. A function is "reachable" if the
/// entry calls it (transitively); "self-recursive" if its own body calls it.
fn recursive_order(env: &Env, entry: &str) -> Result<Vec<String>, ElabError> {
    // BFS the reachable functions.
    let mut reachable: BTreeSet<String> = BTreeSet::new();
    let mut frontier = vec![entry.to_owned()];
    while let Some(f) = frontier.pop() {
        if !reachable.insert(f.clone()) {
            continue;
        }
        if let Some(fd) = env.fns.get(&f) {
            for callee in calls_in_fn(&fd.body) {
                if env.fns.contains_key(&callee) {
                    frontier.push(callee);
                }
            }
        }
    }
    // Self-recursive = a function whose body calls itself.
    let rec: BTreeSet<String> = reachable
        .iter()
        .filter(|f| calls_in_fn(&env.fns[*f].body).contains(*f))
        .cloned()
        .collect();
    // Refuse mutual recursion: any cycle among *distinct* reachable functions. Detect via DFS over
    // the call graph with self-loops ignored; a back-edge to a node on the current path is mutual.
    detect_mutual_recursion(env, &reachable)?;
    // Order rec functions callee-first over their (acyclic, since no mutual recursion) inter-calls.
    let mut order: Vec<String> = Vec::new();
    let mut visited: BTreeSet<String> = BTreeSet::new();
    fn visit(
        env: &Env,
        f: &str,
        rec: &BTreeSet<String>,
        visited: &mut BTreeSet<String>,
        order: &mut Vec<String>,
    ) {
        if !visited.insert(f.to_owned()) {
            return;
        }
        for callee in calls_in_fn(&env.fns[f].body) {
            if callee != f && rec.contains(&callee) {
                visit(env, &callee, rec, visited, order);
            }
        }
        if rec.contains(f) {
            order.push(f.to_owned());
        }
    }
    for f in &rec {
        visit(env, f, &rec, &mut visited, &mut order);
    }
    Ok(order)
}

/// Refuse a mutual-recursion cycle (≥2 distinct functions) with an explicit `Residual` (R7-Q3,
/// deferred). Self-loops (direct recursion) are allowed and ignored here.
fn detect_mutual_recursion(env: &Env, reachable: &BTreeSet<String>) -> Result<(), ElabError> {
    let mut on_path: BTreeSet<String> = BTreeSet::new();
    let mut done: BTreeSet<String> = BTreeSet::new();
    fn dfs(
        env: &Env,
        f: &str,
        on_path: &mut BTreeSet<String>,
        done: &mut BTreeSet<String>,
    ) -> Result<(), ElabError> {
        on_path.insert(f.to_owned());
        for callee in calls_in_fn(&env.fns[f].body) {
            if callee == f || !env.fns.contains_key(&callee) {
                continue; // self-loop is direct recursion (fine); non-fns handled elsewhere
            }
            if on_path.contains(&callee) {
                return residual(
                    f,
                    format!(
                        "`{f}` and `{callee}` are mutually recursive — mutual recursion is deferred \
                         to a later step (R7-Q3); only self-recursion elaborates in v0"
                    ),
                );
            }
            if !done.contains(&callee) {
                dfs(env, &callee, on_path, done)?;
            }
        }
        on_path.remove(f);
        done.insert(f.to_owned());
        Ok(())
    }
    for f in reachable {
        if !done.contains(f) {
            dfs(env, f, &mut on_path, &mut done)?;
        }
    }
    Ok(())
}

/// The set of function/constructor/prim names a body calls (single-segment heads + bare paths). A
/// superset filter — the caller intersects with `env.fns` to get function calls.
fn calls_in_fn(body: &Expr) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    collect_calls(body, &mut out);
    out
}

fn collect_calls(e: &Expr, out: &mut BTreeSet<String>) {
    match e {
        Expr::Path(p) => {
            if p.0.len() == 1 {
                out.insert(p.0[0].clone());
            }
        }
        Expr::App { head, args } => {
            collect_calls(head, out);
            for a in args {
                collect_calls(a, out);
            }
        }
        Expr::Let { bound, body, .. } => {
            collect_calls(bound, out);
            collect_calls(body, out);
        }
        Expr::If { cond, conseq, alt } => {
            collect_calls(cond, out);
            collect_calls(conseq, out);
            collect_calls(alt, out);
        }
        Expr::Match { scrutinee, arms } => {
            collect_calls(scrutinee, out);
            for arm in arms {
                collect_calls(&arm.body, out);
            }
        }
        Expr::For { xs, init, body, .. } => {
            collect_calls(xs, out);
            collect_calls(init, out);
            collect_calls(body, out);
        }
        Expr::Swap { value, .. } => collect_calls(value, out),
        Expr::WithParadigm { body, .. } => collect_calls(body, out),
        Expr::Wild(inner) | Expr::Spore(inner) | Expr::Ascribe(inner, _) => {
            collect_calls(inner, out);
        }
        Expr::Lit(_) => {}
    }
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

/// The elaboration context: the checked environment, the data registry `Σ`, a fresh-name counter
/// (for inlining + match/lambda binders), and the **recursion scope** — the reachable self-recursive
/// functions mapped to their kernel `Fix` variables (RFC-0001 r4). A call to a name in `rec`
/// elaborates to an `App` chain on its `Fix` var; every other function call still **inlines**.
struct Elab<'e> {
    env: &'e Env,
    registry: DataRegistry,
    fresh: u32,
    rec: BTreeMap<String, String>,
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
                    // A bare reference to a recursive function is its Fix variable (a nullary
                    // recursive function `loop()` reached this way unfolds when forced — RFC-0001 r4).
                    if let Some(kf) = self.rec.get(name) {
                        return Ok(Node::Var(kf.clone()));
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
            Expr::For {
                x,
                xs,
                acc,
                init,
                body,
            } => self.elab_for(stack, scope, x, xs, acc, init, body),
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
            Expr::WithParadigm { .. } => residual(
                site,
                "internal: a `with paradigm` block reached elaboration — the ambient resolution \
                 pass strips it (RFC-0012 §4.4)",
            ),
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

        // A call to a recursive function is a curried `App` on its `Fix` variable (RFC-0001 r4) —
        // never inlined (that would loop). Arguments evaluate left-to-right (CBV).
        if let Some(kf) = self.rec.get(name).cloned() {
            let mut node = Node::Var(kf);
            for a in args {
                let karg = self.expr(stack, scope, a)?;
                node = Node::App {
                    func: Box::new(node),
                    arg: Box::new(karg),
                };
            }
            return Ok(node);
        }

        if let Some(fd) = self.env.fns.get(name) {
            // A non-recursive call inlines; a cycle here would be mutual recursion (refused up front
            // in `recursive_order`) — keep an explicit guard as defense in depth, never a silent loop.
            if stack.iter().any(|f| f == name) {
                return residual(
                    site,
                    format!(
                        "`{name}` is in a call cycle that did not resolve to a self-recursive `Fix` \
                         — mutual recursion is deferred (R7-Q3)"
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

    /// Elaborate a reachable **self-recursive** function `fname` to `Fix(kf, λparams. body)` — the
    /// closed form r4 uses for recursion (RFC-0007 §4.1; the v0 surface is first-order, so the body
    /// is closed except for its params, `kf`, and the other recursive functions in scope). Params
    /// are curried (`λp1. … λpn.`).
    fn elab_recursive_fn(&mut self, fname: &str, kf: &str) -> Result<Node, ElabError> {
        let fd = self.env.fns[fname].clone();
        if let Some(g) = fd.sig.ret.guarantee {
            return residual(
                fname,
                format!("`{fname}` asserts `@ {g:?}` on its result — checked dynamically in v0, no L0 form"),
            );
        }
        let mut scope: Vec<Binding> = Vec::new();
        let mut param_kvars: Vec<String> = Vec::new();
        for p in &fd.sig.value_params {
            if let Some(g) = p.ty.guarantee {
                return residual(
                    fname,
                    format!(
                        "`{fname}` parameter `{}` asserts `@ {g:?}` — checked dynamically in v0",
                        p.name
                    ),
                );
            }
            let kp = self.fresh(&p.name);
            let pty = resolve_ty(fname, &self.env.types, &p.ty)
                .map(|(t, _)| t)
                .map_err(|e| ElabError::Residual {
                    site: fname.to_owned(),
                    what: format!("could not resolve `{fname}`'s parameter type: {e}"),
                })?;
            scope.push((p.name.clone(), kp.clone(), pty));
            param_kvars.push(kp);
        }
        let mut stack = vec![fname.to_owned()];
        let body = self.expr(&mut stack, &scope, &fd.body)?;
        // Curry: λp1. λp2. … body (p1 outermost).
        let lam = param_kvars
            .into_iter()
            .rev()
            .fold(body, |acc, kp| Node::Lam {
                param: kp,
                body: Box::new(acc),
            });
        Ok(Node::Fix {
            name: kf.to_owned(),
            body: Box::new(lam),
        })
    }

    /// Elaborate `for x in xs, acc = init => body` to its synthesized self-recursive fold (RFC-0007
    /// §4.8), as an inline `Fix` over the linearly-recursive spine type:
    ///
    /// ```text
    /// App(App(Fix(fold, λs. λa. Match s {
    ///            Nil          => a,
    ///            Cons(x,rest) => App(App(fold, rest), body[acc↦a]) }),
    ///         xs), init)
    /// ```
    ///
    /// The nil/cons shape was already validated by the checker (`linear_elem_ty`); here we just read
    /// off the element/spine field positions from the registry.
    #[allow(clippy::too_many_arguments)]
    fn elab_for(
        &mut self,
        stack: &mut Vec<String>,
        scope: &[Binding],
        x: &str,
        xs: &Expr,
        acc: &str,
        init: &Expr,
        body: &Expr,
    ) -> Result<Node, ElabError> {
        let site = stack.last().expect("non-empty").clone();
        let sty = infer_type(self.env, &mut Self::ty_scope(scope), xs).map_err(|e| {
            ElabError::Residual {
                site: site.clone(),
                what: format!("could not infer the `for` spine type: {e}"),
            }
        })?;
        let Ty::Data(tname) = &sty else {
            return residual(&site, format!("`for` spine is not a data type: {sty}"));
        };
        let d = self
            .env
            .types
            .get(tname)
            .ok_or_else(|| ElabError::Residual {
                site: site.clone(),
                what: format!("unknown type `{tname}`"),
            })?
            .clone();
        // Find the nil constructor (no fields) and the cons constructor (one spine field of type
        // `tname` + one element field).
        let mut nil_name: Option<String> = None;
        let mut cons: Option<(String, usize, usize, Ty)> = None; // (name, elem_idx, spine_idx, elem_ty)
        for c in &d.ctors {
            if c.fields.is_empty() {
                nil_name = Some(c.name.clone());
                continue;
            }
            let Some(spine_idx) = c
                .fields
                .iter()
                .position(|f| matches!(f, Ty::Data(n) if n == tname))
            else {
                return residual(
                    &site,
                    format!("`for` constructor `{}` has no spine field", c.name),
                );
            };
            let Some(elem_idx) = (0..c.fields.len()).find(|&i| i != spine_idx) else {
                return residual(
                    &site,
                    format!("`for` constructor `{}` has no element field", c.name),
                );
            };
            cons = Some((
                c.name.clone(),
                elem_idx,
                spine_idx,
                c.fields[elem_idx].clone(),
            ));
        }
        let (Some(nil_name), Some((cons_name, _elem_idx, spine_idx, elem_ty))) = (nil_name, cons)
        else {
            return residual(
                &site,
                format!("`for` needs a nil + cons shape on `{tname}`"),
            );
        };
        let aty = infer_type(self.env, &mut Self::ty_scope(scope), init).map_err(|e| {
            ElabError::Residual {
                site: site.clone(),
                what: format!("could not infer the `for` accumulator type: {e}"),
            }
        })?;
        let xs_node = self.expr(stack, scope, xs)?;
        let init_node = self.expr(stack, scope, init)?;

        // Fresh kernel vars for the synthesized fold.
        let fold = self.fresh("fold");
        let s_kv = self.fresh("s");
        let a_kv = self.fresh("acc");
        let elem_kv = self.fresh(x);
        let spine_kv = self.fresh("rest");
        let cons_arity = d
            .ctors
            .iter()
            .find(|c| c.name == cons_name)
            .expect("cons ctor present")
            .fields
            .len();
        let binders: Vec<String> = (0..cons_arity)
            .map(|i| {
                if i == spine_idx {
                    spine_kv.clone()
                } else {
                    elem_kv.clone()
                }
            })
            .collect();

        // The loop body, with `x` ↦ the element binder and `acc` ↦ the accumulator parameter.
        let mut body_scope = scope.to_vec();
        body_scope.push((x.to_owned(), elem_kv.clone(), elem_ty));
        body_scope.push((acc.to_owned(), a_kv.clone(), aty));
        let body_node = self.expr(stack, &body_scope, body)?;

        let nil_ref = self
            .ctor_ref(&nil_name)
            .ok_or_else(|| ElabError::Residual {
                site: site.clone(),
                what: format!("`{nil_name}` is outside the r3 data registry"),
            })?;
        let cons_ref = self
            .ctor_ref(&cons_name)
            .ok_or_else(|| ElabError::Residual {
                site: site.clone(),
                what: format!("`{cons_name}` is outside the r3 data registry"),
            })?;
        // Cons arm body: App(App(fold, rest), body[acc↦a]).
        let rec_call = Node::App {
            func: Box::new(Node::App {
                func: Box::new(Node::Var(fold.clone())),
                arg: Box::new(Node::Var(spine_kv)),
            }),
            arg: Box::new(body_node),
        };
        let match_node = Node::Match {
            scrutinee: Box::new(Node::Var(s_kv.clone())),
            alts: vec![
                Alt::Ctor {
                    ctor: nil_ref,
                    binders: vec![],
                    body: Node::Var(a_kv.clone()),
                },
                Alt::Ctor {
                    ctor: cons_ref,
                    binders,
                    body: rec_call,
                },
            ],
            default: None,
        };
        let fix = Node::Fix {
            name: fold,
            body: Box::new(Node::Lam {
                param: s_kv,
                body: Box::new(Node::Lam {
                    param: a_kv,
                    body: Box::new(match_node),
                }),
            }),
        };
        // App(App(fix, xs), init) — walk the spine head-to-tail from the initial accumulator.
        Ok(Node::App {
            func: Box::new(Node::App {
                func: Box::new(fix),
                arg: Box::new(xs_node),
            }),
            arg: Box::new(init_node),
        })
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
    fn self_recursion_now_elaborates_to_fix_and_runs() {
        // r4: a self-recursive function elaborates to a Fix and runs on the interpreter.
        // drop_(S(S(Z))) ⟶ Z.
        let env = env("colony d\ntype Nat = Z | S(Nat)\n\
             fn drop_(n: Nat) -> Nat = match n { Z => Z, S(m) => drop_(m) }\n\
             fn main() -> Nat = drop_(S(S(Z)))");
        let node = elaborate(&env, "main").expect("self-recursion elaborates in r4");
        let v = mycelium_interp::Interpreter::default()
            .eval_core(&node)
            .expect("terminates");
        assert_eq!(v.as_data().expect("data").fields().len(), 0, "Z");
    }

    #[test]
    fn an_unproductive_recursion_elaborates_then_exhausts_fuel() {
        // A non-terminating recursion still elaborates (it is in the fragment now) but the fuel clock
        // makes its evaluation an explicit refusal, never a hang (RFC-0007 §4.5).
        let env = env("colony d\nfn spin(x: Binary{8}) -> Binary{8} = spin(x)\n\
             fn main() -> Binary{8} = spin(0b0000_0001)");
        let node = elaborate(&env, "main").expect("recursion elaborates in r4");
        let err = mycelium_interp::Interpreter::default()
            .with_fuel(500)
            .eval(&node)
            .unwrap_err();
        assert_eq!(err, mycelium_interp::EvalError::FuelExhausted);
    }

    #[test]
    fn mutual_recursion_is_an_explicit_residual() {
        // R7-Q3: mutual recursion is deferred — an explicit Residual, never a silent loop.
        let env = env("colony d\ntype Nat = Z | S(Nat)\n\
             fn ping(n: Nat) -> Nat = match n { Z => Z, S(m) => pong(m) }\n\
             fn pong(n: Nat) -> Nat = match n { Z => Z, S(m) => ping(m) }\n\
             fn main() -> Nat = ping(S(Z))");
        let err = elaborate(&env, "main").unwrap_err();
        let ElabError::Residual { what, .. } = &err else {
            panic!("expected Residual, got {err:?}");
        };
        assert!(what.contains("mutual"), "got: {what}");
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
    fn a_for_fold_now_elaborates_to_a_fix_fold_and_runs() {
        // r4: `for` desugars to a synthesized self-recursive Fix fold and runs. A 2-element xor-fold
        // of 0b1111_0000 and 0b0000_1111 from 0 is 0b1111_1111.
        let env = env("colony d\ntype Bytes = End | More(Binary{8}, Bytes)\n\
             fn checksum(bs: Bytes) -> Binary{8} = for b in bs, acc = 0b0000_0000 => xor(acc, b)\n\
             fn main() -> Binary{8} = checksum(More(0b1111_0000, More(0b0000_1111, End)))");
        let node = elaborate(&env, "main").expect("`for` elaborates in r4");
        let v = mycelium_interp::Interpreter::default()
            .eval(&node)
            .expect("runs");
        assert_eq!(v.payload(), &Payload::Bits(vec![true; 8]));
    }

    #[test]
    fn a_for_fold_over_nil_is_the_initial_accumulator() {
        let env = env("colony d\ntype Bytes = End | More(Binary{8}, Bytes)\n\
             fn checksum(bs: Bytes) -> Binary{8} = for b in bs, acc = 0b1010_1010 => xor(acc, b)\n\
             fn main() -> Binary{8} = checksum(End)");
        let node = elaborate(&env, "main").expect("`for` elaborates in r4");
        let v = mycelium_interp::Interpreter::default()
            .eval(&node)
            .expect("runs");
        assert_eq!(
            v.payload(),
            &Payload::Bits(vec![true, false, true, false, true, false, true, false])
        );
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
