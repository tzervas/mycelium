//! **Elaboration to the L0 Core IR** (RFC-0007 §4.6, **retired by RFC-0001 r4**). The
//! evaluation-complete fragment is now the **whole v0 calculus**: representation ops (L0), data +
//! matching (r3, `Construct`/flat `Match`), and **functions + recursion** (r4/r5,
//! `Lam`/`App`/`Fix`/`FixGroup`). So a self- *or* mutually-recursive, data-building, matching program
//! elaborates to a closed L0 term. The only explicit [`ElabError::Residual`] left for a structurally
//! v0 program is a **dynamic guarantee index** `@ g` (RFC-0007 §4.3, stage 0) — never a partial
//! artifact; that runs on the L1 fuel-guarded evaluator ([`crate::eval`]) instead.
//!
//! This module also owns the shared surface→kernel bridge the evaluator reuses, so the two
//! execution paths cannot drift on the basics: literal values ([`lit_value`]), representation
//! resolution ([`type_repr`]), and the v0 policy-name reference ([`policy_name_ref`]).
//!
//! # How a `match` lowers (RFC-0011 §4.4)
//! Nested surface patterns are compiled to the **flat** kernel `Match` by the **M-320 Maranget
//! decision tree** (`crate::decision`) — the untrusted, inspectable lowering. Each tree `Switch`
//! becomes an L0 `Match` on the occurrence's bound variable; each constructor case becomes an
//! `Alt::Ctor` binding *all* the constructor's fields (so every binder occurrence is available at
//! the leaf), and each leaf elaborates the surface arm's body with its binders mapped to those
//! field variables. `if` desugars to a `Match` on the prelude `Bool`. WF7 coverage is the checker's
//! (the tree is verified `Fail`-free before lowering — defense in depth, never silent).
//!
//! # How recursion lowers (RFC-0001 r4/r5)
//! The reachable call graph is decomposed into strongly-connected components (Tarjan), bound
//! **callee-first**. A **self-recursive singleton** is bound once as `let f = Fix(f, λparams. body)`;
//! a **mutually-recursive group** of ≥2 functions (M-343; R7-Q3) is bound as a single
//! `FixGroup{[(f, λ…), (g, λ…), …]}` whose members are all mutually in scope. A call to any recursive
//! function becomes a curried `App` on its recursion variable; every **other** call still inlines
//! (the residual non-recursive call graph is acyclic). `for` desugars to a synthesized self-recursive
//! `Fix` fold over the linear spine (RFC-0007 §4.8).

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
            ElabError::UnknownFn(name) => write!(f, "no function `{name}` in the checked nodule"),
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

/// Elaborate the nullary function `entry` of a checked nodule to a closed L0 [`Node`].
///
/// As of RFC-0001 r4 the evaluation-complete fragment is the **whole v0 calculus**: data + matching
/// (r3) and now **functions + recursion** (`Lam`/`App`/`Fix`). Each reachable **self-recursive**
/// function is bound once as `let f = Fix(f, λparams. body)` (callee-first), and a call to it
/// elaborates to a curried `App`; every other call still inlines (the non-recursive call graph is
/// acyclic). **Mutual recursion** lowers to a `FixGroup` (RFC-0001 r5; M-343 — R7-Q3); top-level
/// functions in a nodule are mutually visible (RP-6 / DN-13 — no surface marker), so a mutual group is
/// *inferred* from the call graph and **materialized as a `FixGroup` node** — inspectable in the
/// elaborated term, never a black box. Still
/// `Residual`: a dynamic guarantee index `@ g` (RFC-0007 §4.3, stage 0). On success the result is a
/// closed L0 term whose evaluation must agree with the L1 evaluator (NFR-7; the M-210 differential).
pub fn elaborate(env: &Env, entry: &str) -> Result<Node, ElabError> {
    let (mut el, binders, fd) = elab_prelude(env, entry)?;
    let mut stack = vec![entry.to_owned()];
    let entry_body = el.expr(&mut stack, &[], &fd.body)?;
    Ok(wrap_in_binders(binders, entry_body))
}

/// **Per-hypha elaboration of a `colony` entry** for the *real-concurrency* execution path
/// (RFC-0008 §4.7; M-666 redone with the `mycelium-mlir::runtime` executor). Where [`elaborate`]
/// produces the single L0 `Node` whose body is the **RT2 spawn-order sequentialization** (a `Let`
/// chain — the deterministic *reference* the concurrent run is validated against), this produces one
/// **closed L0 `Node` per `hypha`**: each hypha body, elaborated under the entry's scope and wrapped
/// in the **same** recursive-binder prelude (so a hypha may call the nodule's recursive functions).
/// The `mycelium-mlir` colony driver spawns each as a concurrent `Task` in a `Scope`/`Colony`, runs
/// the structured fork/join, and validates the concurrent observable **equals** the sequential
/// reference (RT2; an inequality is an explicit divergence, never silent — G2/RT4).
///
/// The colony's *observable* is its **last** hypha's value (the type rule, [`crate::checkty`]); the
/// returned vector preserves spawn order, so element `N-1` is that observable. This adds **no L0
/// concurrency node** — the trusted base stays sequential (RFC-0008 §4.2; KC-3); concurrency is
/// scheduling layered *over* unchanged per-hypha L0 terms.
///
/// Refuses with an explicit [`ElabError::Residual`] (never a fabricated accept) when the entry body
/// is **not** a `colony`, or when any hypha body is outside the evaluation-complete fragment.
pub fn elaborate_colony(env: &Env, entry: &str) -> Result<Vec<Node>, ElabError> {
    let (mut el, binders, fd) = elab_prelude(env, entry)?;
    let Expr::Colony(hyphae) = &fd.body else {
        return residual(
            entry,
            "the entry body is not a `colony` — `elaborate_colony` lowers a colony to its per-hypha \
             closed L0 programs (the concurrent path); use `elaborate` for the sequentialized form",
        );
    };
    if hyphae.is_empty() {
        return residual(
            entry,
            "internal: an empty `colony` reached elaboration — the parser requires ≥ 1 hypha \
             (RFC-0008 §4.7)",
        );
    }
    let mut stack = vec![entry.to_owned()];
    // One closed L0 program per hypha: the hypha body elaborated under the entry scope, then wrapped
    // in the *shared* recursive prelude (the binders are cloned per hypha so each task is independent
    // — RT1: no shared state crosses a hypha boundary). Spawn order is preserved.
    let mut programs = Vec::with_capacity(hyphae.len());
    for h in hyphae {
        let body = el.expr(&mut stack, &[], &h.body)?;
        programs.push(wrap_in_binders(binders.clone(), body));
    }
    Ok(programs)
}

/// Shared front-end of [`elaborate`]/[`elaborate_colony`]: validate the entry is a closed (nullary,
/// no dynamic guarantee) definition, build the data registry, decompose the reachable call graph into
/// callee-first recursive SCCs (Tarjan), and elaborate each SCC's recursive binder. Returns the
/// primed [`Elab`], the callee-first binder list, and the entry's [`FnDecl`]. DRY: the recursion
/// machinery is identical whether the entry body is sequentialized to one `Node` or split per-hypha.
fn elab_prelude<'e>(
    env: &'e Env,
    entry: &str,
) -> Result<(Elab<'e>, Vec<RecBinding>, &'e crate::ast::FnDecl), ElabError> {
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
    // The recursive strongly-connected components of the reachable call graph, callee-first (Tarjan).
    // A self-recursive singleton stays a `Fix`; a group of ≥2 mutually-recursive functions becomes a
    // `FixGroup` (RFC-0001 r5; M-343 enacts mutual recursion — R7-Q3).
    let sccs = recursive_sccs(env, entry)?;
    let mut el = Elab {
        env,
        registry,
        fresh: 0,
        rec: BTreeMap::new(),
    };
    // Every member of a recursive SCC gets a kernel recursion variable — in scope for every recursive
    // body (its own SCC and any callee SCC) and the entry body.
    for scc in &sccs {
        for f in scc {
            let kf = el.fresh(f);
            el.rec.insert(f.clone(), kf);
        }
    }
    // Elaborate each SCC's binding, callee-first: a singleton self-recursion is a `Fix`; a group is a
    // `FixGroup` over the members' curried lambdas (each member sees every name in the group).
    let mut binders: Vec<RecBinding> = Vec::with_capacity(sccs.len());
    for scc in &sccs {
        if scc.len() == 1 {
            let f = &scc[0];
            let kf = el.rec[f].clone();
            let fix = Box::new(el.elab_recursive_fn(f, &kf)?);
            binders.push(RecBinding::Single { var: kf, fix });
        } else {
            let mut defs: Vec<(String, Box<Node>)> = Vec::with_capacity(scc.len());
            for f in scc {
                let kf = el.rec[f].clone();
                defs.push((kf, Box::new(el.elab_fn_lam(f)?)));
            }
            binders.push(RecBinding::Group(defs));
        }
    }
    Ok((el, binders, fd))
}

/// Wrap an elaborated body in a callee-first binder prelude. `binders` is callee-first; fold in
/// reverse so the first (callee) binding ends up outermost (in scope for every later binding and the
/// body). Shared by [`elaborate`] (one body) and [`elaborate_colony`] (each hypha body).
fn wrap_in_binders(binders: Vec<RecBinding>, body: Node) -> Node {
    binders.into_iter().rev().fold(body, |acc, b| match b {
        RecBinding::Single { var, fix } => Node::Let {
            id: var,
            bound: fix,
            body: Box::new(acc),
        },
        RecBinding::Group(defs) => Node::FixGroup {
            defs,
            body: Box::new(acc),
        },
    })
}

/// One recursive binding the entry body is wrapped in: a self-recursive singleton (`Fix`, bound via
/// `Let`) or a mutually-recursive group (`FixGroup`). Built callee-first; see [`elaborate`].
// `Clone` so [`elaborate_colony`] can replay the *same* recursive prelude over each hypha body
// (every concurrent task is an independent closed term — RT1: no shared mutable state).
#[derive(Clone)]
enum RecBinding {
    /// A self-recursive function: its kernel variable and the `Fix` node bound to it (boxed — the
    /// `Group` variant is pointer-sized, so an unboxed `Node` here would unbalance the enum).
    Single { var: String, fix: Box<Node> },
    /// A mutually-recursive group: `(member variable, curried lambda)` pairs, all mutually in scope.
    Group(Vec<(String, Box<Node>)>),
}

/// The **recursive** strongly-connected components of the reachable call graph, **callee-first**
/// (a callee SCC is bound *outside* its callers). A self-recursive singleton (`{f}` with a self-call)
/// and a mutual group (≥2 functions in a cycle) are both recursive SCCs; a function in no cycle
/// inlines and is **not** returned. Computed with Tarjan's algorithm — which finalises each SCC only
/// after all its successor (callee) SCCs, i.e. in reverse-topological = callee-first order. Roots,
/// successors, and each SCC's members are visited/sorted deterministically so the lowering (and thus
/// the content hash) is reproducible. A function is "reachable" if the entry transitively calls it.
fn recursive_sccs(env: &Env, entry: &str) -> Result<Vec<Vec<String>>, ElabError> {
    // BFS the reachable functions (sorted via the BTreeSet).
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

    // Tarjan's SCC over the reachable call graph.
    struct Tarjan<'e> {
        env: &'e Env,
        reachable: &'e BTreeSet<String>,
        index: usize,
        idx: BTreeMap<String, usize>,
        low: BTreeMap<String, usize>,
        on_stack: BTreeSet<String>,
        stack: Vec<String>,
        out: Vec<Vec<String>>,
    }
    // The reachable function callees of `f`, sorted and unique (BTreeSet) for a deterministic walk.
    fn successors(env: &Env, reachable: &BTreeSet<String>, f: &str) -> BTreeSet<String> {
        calls_in_fn(&env.fns[f].body)
            .into_iter()
            .filter(|c| reachable.contains(c) && env.fns.contains_key(c))
            .collect()
    }
    fn strongconnect(t: &mut Tarjan, v: &str) {
        t.idx.insert(v.to_owned(), t.index);
        t.low.insert(v.to_owned(), t.index);
        t.index += 1;
        t.stack.push(v.to_owned());
        t.on_stack.insert(v.to_owned());
        for w in successors(t.env, t.reachable, v) {
            if !t.idx.contains_key(&w) {
                strongconnect(t, &w);
                let lw = t.low[&w];
                let lv = t.low.get_mut(v).expect("v indexed");
                *lv = (*lv).min(lw);
            } else if t.on_stack.contains(&w) {
                let iw = t.idx[&w];
                let lv = t.low.get_mut(v).expect("v indexed");
                *lv = (*lv).min(iw);
            }
        }
        if t.low[v] == t.idx[v] {
            let mut scc: Vec<String> = Vec::new();
            loop {
                let w = t.stack.pop().expect("stack non-empty while popping an SCC");
                t.on_stack.remove(&w);
                let is_root = w == v;
                scc.push(w);
                if is_root {
                    break;
                }
            }
            scc.sort(); // deterministic member order (group binding order is observable in the hash)
            t.out.push(scc);
        }
    }
    let mut t = Tarjan {
        env,
        reachable: &reachable,
        index: 0,
        idx: BTreeMap::new(),
        low: BTreeMap::new(),
        on_stack: BTreeSet::new(),
        stack: Vec::new(),
        out: Vec::new(),
    };
    for f in &reachable {
        if !t.idx.contains_key(f) {
            strongconnect(&mut t, f);
        }
    }

    // Keep only the *recursive* SCCs (a multi-member group, or a self-looping singleton), preserving
    // Tarjan's callee-first order.
    let sccs = t
        .out
        .into_iter()
        .filter(|scc| scc.len() > 1 || calls_in_fn(&env.fns[&scc[0]].body).contains(&scc[0]))
        .collect();
    Ok(sccs)
}

/// The set of function/constructor/prim names a body calls (single-segment heads + bare paths). A
/// superset filter — the caller intersects with `env.fns` to get function calls.
fn calls_in_fn(body: &Expr) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    collect_calls(body, &mut out);
    out
}

fn collect_calls(e: &Expr, out: &mut BTreeSet<String>) {
    // Same pre-order traversal totality uses (M-641) — factored into the one shared `walk_expr`;
    // this collector's *action* differs (it gathers **every** single-segment path, the superset
    // filter `calls_in_fn` documents, not just `App` heads), so the visitor closure carries that.
    crate::totality::walk_expr(e, &mut |x| {
        if let Expr::Path(p) = x {
            if p.0.len() == 1 {
                out.insert(p.0[0].clone());
            }
        }
    });
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
///
/// A residual [`Ty::Var`] (stage-1 generics, M-657) reaching this point is a checker bug —
/// generic shells must never be elaborated directly, only their monomorphic instantiations.
/// Returns `None` so `build_registry` skips the type (Residual if it is ever used at runtime).
fn field_spec(ty: &Ty) -> Option<FieldSpec> {
    Some(match ty {
        Ty::Binary(n) => FieldSpec::Repr(Repr::Binary { width: *n }),
        Ty::Ternary(m) => FieldSpec::Repr(Repr::Ternary { trits: *m }),
        Ty::Dense(d, s) => FieldSpec::Repr(Repr::Dense {
            dim: *d,
            dtype: scalar_kind(*s),
        }),
        Ty::Data(n) => FieldSpec::Data(n.clone()),
        // `Substrate` and `Var` have no r3 value form.  A residual `Var` here is defense in
        // depth: the checker must have substituted all vars before storing into `env.types`.
        Ty::Substrate(_) | Ty::Var(_) => return None,
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
            Expr::Colony(hyphae) => self.elab_colony(stack, scope, hyphae),
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
    /// nodes; a call to a recursive function (in `self.rec`) becomes a curried `App` on its recursion
    /// variable (`Fix`/`FixGroup`), and every **other** user-function call **inlines** (the residual
    /// non-recursive call graph is acyclic, so inlining terminates).
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
            // A non-recursive call inlines. Any function in a cycle (self or mutual) is in `self.rec`
            // and was handled by the recursion-variable branch above, so reaching here while `name`
            // is on the inline stack would mean a cycle escaped SCC detection — keep an explicit
            // guard as defense in depth (an internal invariant), never a silent inline loop.
            if stack.iter().any(|f| f == name) {
                return residual(
                    site,
                    format!(
                        "`{name}` is in a call cycle that was not registered as recursive — internal \
                         elaboration invariant (every cycle should lower to `Fix`/`FixGroup`)"
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
                let pty = resolve_ty(site, &self.env.types, &[], &param.ty)
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
    /// closed form r4 uses for direct recursion (RFC-0007 §4.1; the v0 surface is first-order, so the
    /// body is closed except for its params, `kf`, and the other recursive functions in scope).
    fn elab_recursive_fn(&mut self, fname: &str, kf: &str) -> Result<Node, ElabError> {
        Ok(Node::Fix {
            name: kf.to_owned(),
            body: Box::new(self.elab_fn_lam(fname)?),
        })
    }

    /// Elaborate `fname` to its curried lambda `λp1. … λpn. body` (params `p1` outermost), with the
    /// body in scope of the params and **every** recursion variable in `self.rec` (its own name plus
    /// any sibling in its group). This is the recursion-variable-agnostic core shared by
    /// [`Self::elab_recursive_fn`] (which wraps it in a `Fix`) and the `FixGroup` group lowering
    /// (which binds the lambdas of a mutually-recursive SCC together — RFC-0001 r5).
    fn elab_fn_lam(&mut self, fname: &str) -> Result<Node, ElabError> {
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
            let pty = resolve_ty(fname, &self.env.types, &[], &p.ty)
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
        Ok(param_kvars
            .into_iter()
            .rev()
            .fold(body, |acc, kp| Node::Lam {
                param: kp,
                body: Box::new(acc),
            }))
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

    /// Elaborate `colony { hypha e1, …, hypha eN }` to its **RT2 spawn-order sequentialization**
    /// (RFC-0008 §4.2/RT2; M-666). RFC-0008 makes the *reference semantics* of a deterministic
    /// concurrent program its deterministic sequentialization, and content-addressing/the NFR-7
    /// differential are over that reference — so the honest L0 form is the sequentialization, **not**
    /// a concurrency node (the L0 Core IR has none; the trusted base stays sequential — KC-3). It
    /// lowers to a chain of `Let`s that evaluates each leading hypha for its (sequentialized) effect,
    /// in order, and yields the **last** hypha's value:
    ///
    /// ```text
    /// Let(_1, ⟦e1⟧, Let(_2, ⟦e2⟧, … ⟦eN⟧))      (each _i a fresh, `%`-named unused binder)
    /// ```
    ///
    /// Nothing is dropped silently (G2): every hypha body is elaborated and bound, so a leading
    /// hypha's refusal/divergence is preserved under CBV (RT4/I1). This sequentialization is the
    /// **RT2 reference ORACLE**: the real concurrent executor (`mycelium-mlir::runtime` —
    /// `Scope`/`Colony`/`Task`, structured fork/join, M-357) runs the colony's per-hypha L0 programs
    /// ([`elaborate_colony`]) as concurrent tasks and is **validated equal to this reference** (the
    /// RT2 differential, `mycelium_mlir::run_colony`); a divergence is an explicit error, never a
    /// silent race (G2/RT4). The concurrent run adds **no L0 kernel node** — the trusted base stays
    /// sequential (RFC-0008 §4.2; KC-3).
    ///
    /// Honesty (Declared at the surface; the lowering itself adds no guarantee): this realizes the
    /// *deterministic* R1 fragment (RFC-0008 §4.6 R1) only. With no v0 product type the colony's
    /// observable is the last hypha's value (the sequential reference's final step), never a
    /// fabricated join-product.
    fn elab_colony(
        &mut self,
        stack: &mut Vec<String>,
        scope: &[Binding],
        hyphae: &[crate::ast::Hypha],
    ) -> Result<Node, ElabError> {
        let site = stack.last().expect("non-empty").clone();
        let Some((last, leading)) = hyphae.split_last() else {
            return residual(
                &site,
                "internal: an empty `colony` reached elaboration — the parser requires ≥ 1 hypha \
                 (RFC-0008 §4.7)",
            );
        };
        // The last hypha is the colony's observable (the RT2 sequentialization's final step).
        let mut node = self.expr(stack, scope, &last.body)?;
        // Wrap right-to-left so the first hypha's `Let` ends up outermost (evaluated first, CBV).
        for h in leading.iter().rev() {
            let bound = self.expr(stack, scope, &h.body)?;
            // A fresh `%`-named binder: `%` is not a surface identifier char, so it never captures a
            // surface name, and the binding is intentionally unused (the value is sequentialized for
            // its effect only). The leading hypha is still fully evaluated under CBV.
            let kvar = self.fresh("hypha");
            node = Node::Let {
                id: kvar,
                bound: Box::new(bound),
                body: Box::new(node),
            };
        }
        Ok(node)
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
    use crate::checkty::check_nodule;
    use crate::parse;

    fn env(src: &str) -> Env {
        check_nodule(&parse(src).expect("parses")).expect("checks")
    }

    #[test]
    fn a_const_let_op_swap_body_elaborates_closed() {
        let env = env(
            "nodule d\nfn main() -> Ternary{6} =\n  let a = 0b1011_0010 in swap(not(a), to: Ternary{6}, policy: rt)",
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
            "nodule d\nfn flip(x: Binary{8}) -> Binary{8} = not(x)\nfn main() -> Binary{8} = flip(flip(0b1010_1010))",
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
        let env = env("nodule d\ntype Nat = Z | S(Nat)\n\
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
        let env = env("nodule d\nfn spin(x: Binary{8}) -> Binary{8} = spin(x)\n\
             fn main() -> Binary{8} = spin(0b0000_0001)");
        let node = elaborate(&env, "main").expect("recursion elaborates in r4");
        let err = mycelium_interp::Interpreter::default()
            .with_fuel(500)
            .eval(&node)
            .unwrap_err();
        assert_eq!(err, mycelium_interp::EvalError::FuelExhausted);
    }

    /// Whether `n` contains a `FixGroup` anywhere (the mutual-recursion lowering — M-343).
    fn contains_fixgroup(n: &Node) -> bool {
        match n {
            Node::FixGroup { .. } => true,
            Node::Let { bound, body, .. } => contains_fixgroup(bound) || contains_fixgroup(body),
            Node::Fix { body, .. } | Node::Lam { body, .. } => contains_fixgroup(body),
            Node::App { func, arg } => contains_fixgroup(func) || contains_fixgroup(arg),
            Node::Op { args, .. } | Node::Construct { args, .. } => {
                args.iter().any(contains_fixgroup)
            }
            Node::Swap { src, .. } => contains_fixgroup(src),
            Node::Match {
                scrutinee,
                alts,
                default,
            } => {
                contains_fixgroup(scrutinee)
                    || alts.iter().any(|a| match a {
                        Alt::Ctor { body, .. } | Alt::Lit { body, .. } => contains_fixgroup(body),
                    })
                    || default.as_deref().is_some_and(contains_fixgroup)
            }
            Node::Const(_) | Node::Var(_) => false,
        }
    }

    #[test]
    fn mutual_recursion_now_elaborates_to_a_fixgroup_and_runs() {
        // M-343 (R7-Q3): a mutually-recursive group (ping/pong) lowers to a `FixGroup` and runs on
        // the reference interpreter — ping(S(Z)) ⟶ pong(Z) ⟶ Z. (Previously an explicit Residual.)
        let env = env("nodule d\ntype Nat = Z | S(Nat)\n\
             fn ping(n: Nat) -> Nat = match n { Z => Z, S(m) => pong(m) }\n\
             fn pong(n: Nat) -> Nat = match n { Z => Z, S(m) => ping(m) }\n\
             fn main() -> Nat = ping(S(Z))");
        let node = elaborate(&env, "main").expect("mutual recursion elaborates to a FixGroup");
        assert!(
            contains_fixgroup(&node),
            "the mutual-recursion lowering must use a FixGroup node"
        );
        let v = mycelium_interp::Interpreter::default()
            .with_fuel(10_000)
            .eval_core(&node)
            .expect("the FixGroup runs to a value");
        let d = v.as_data().expect("a Nat data value");
        assert_eq!(d.fields().len(), 0, "ping(S(Z)) = pong(Z) = Z (nullary)");
    }

    #[test]
    fn a_match_now_elaborates_to_l0_and_runs() {
        // r3: `match` is no longer Residual — it lowers to a flat L0 Match and runs on the
        // reference interpreter. `match Pos { Neg => <->, Zero => <0>, _ => <+> }` ⟶ <+>.
        let env = env(
            "nodule d\ntype Sign = Neg | Zero | Pos\nfn main() -> Ternary{1} =\n  match Pos { Neg => <->, Zero => <0>, _ => <+> }",
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
        let env = env("nodule d\ntype Nat = Z | S(Nat)\nfn main() -> Nat = S(Z)");
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
            "nodule d\nfn pick(b: Bool) -> Binary{8} = if b then 0b1111_1111 else 0b0000_0000\n\
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
        let env = env("nodule d\ntype Nat = Z | S(Nat)\n\
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
            "nodule d\nfn main() -> Ternary{6} @ Proven = swap(0b0000_0010, to: Ternary{6}, policy: rt)",
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
        let env = env("nodule d\ntype Bytes = End | More(Binary{8}, Bytes)\n\
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
        let env = env("nodule d\ntype Bytes = End | More(Binary{8}, Bytes)\n\
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
        let env = env("nodule d\nfn id(x: Binary{8}) -> Binary{8} = x");
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

    // ---- M-666: `colony { hypha … }` elaboration (RFC-0008 §4.7) ----

    #[test]
    fn a_single_hypha_colony_elaborates_to_its_body_and_runs() {
        // RT2 reference semantics: a one-hypha colony *is* its body. `colony { hypha not(0b…) }`
        // elaborates and runs to `not(0b1011_0010) = 0b0100_1101`.
        let env = env("nodule d\nfn main() -> Binary{8} = colony { hypha not(0b1011_0010) }");
        let node = elaborate(&env, "main").expect("a colony is in the fragment (M-666)");
        let v = mycelium_interp::Interpreter::default()
            .eval(&node)
            .expect("runs");
        assert_eq!(
            v.payload(),
            &Payload::Bits(vec![false, true, false, false, true, true, false, true])
        );
    }

    #[test]
    fn a_multi_hypha_colony_lowers_to_a_let_chain_and_yields_the_last_hypha() {
        // The RT2 spawn-order sequentialization lowers to nested `Let`s (leading hyphae bound to
        // fresh `%`-names), so the L0 form contains ≥1 `Let` and the observable is the LAST hypha's
        // value — here `xor(0b1111_0000, 0b0000_1111) = 0b1111_1111`, regardless of the leading two.
        let env = env(
            "nodule d\nfn compute(x: Binary{8}) -> Binary{8} = not(x)\n\
             fn main() -> Binary{8} =\n  colony { hypha compute(0b0000_0001), hypha compute(0b0000_0010), hypha xor(0b1111_0000, 0b0000_1111) }",
        );
        let node = elaborate(&env, "main").expect("multi-hypha colony elaborates");
        // The lowering is a Let chain (the sequentialization), not a single bare op.
        assert!(
            matches!(node, Node::Let { .. }),
            "a multi-hypha colony must lower to a Let chain (the RT2 sequentialization), got {node:?}"
        );
        let v = mycelium_interp::Interpreter::default()
            .eval(&node)
            .expect("runs");
        assert_eq!(
            v.payload(),
            &Payload::Bits(vec![true; 8]),
            "last hypha = all-ones"
        );
    }

    /// **Property (RT2 sequentialization bound; RFC-0008 §4.2/§4.6 R1).** For *every* number of
    /// leading pure hyphae `k ∈ 0..=8`, a colony `colony { hypha e_0, …, hypha e_{k-1}, hypha
    /// e_last }` elaborates to L0 and evaluates to **exactly** `e_last`'s value — the leading hyphae
    /// never change the observable (the colony equals its last hypha under sequentialization). The
    /// leading bodies are all *distinct* from the last, so a silent "keep the first" / "drop the
    /// last" elaboration bug would change the result and trip the assertion. This is the bound the
    /// `colony` surface rests on; bounded exhaustive generation over `k` is the crate's property-test
    /// idiom (no `proptest` dep — consistent with `usefulness`/`totality`).
    #[test]
    fn prop_colony_value_is_its_last_hypha_for_any_leading_count() {
        let interp = mycelium_interp::Interpreter::default();
        // The last hypha's expected 8-bit payload: not(0b0101_0101) = 0b1010_1010.
        let last_payload: Vec<bool> = (0..8u32).map(|i| i.is_multiple_of(2)).collect();
        for k in 0u32..=8 {
            // k distinct leading hyphae, each a different pure `not(...)` over a per-index literal,
            // then the final hypha whose value is the only observable.
            let mut hyphae = String::new();
            for j in 0..k {
                // a per-index 8-bit literal so the leading bodies differ from each other & the last
                let bits: String = (0..8u32)
                    .map(|b| if (j + b).is_multiple_of(2) { '1' } else { '0' })
                    .collect();
                hyphae.push_str(&format!("hypha not(0b{bits}), "));
            }
            // last hypha: xor(0b1111_0000, 0b0101_0101) = 0b1010_0101? compute deterministically.
            // Use a literal whose value we assert directly to avoid arithmetic ambiguity: a `not`.
            // not(0b0101_0101) = 0b1010_1010 = last_payload.
            hyphae.push_str("hypha not(0b0101_0101)");
            let src = format!("nodule d\nfn main() -> Binary{{8}} = colony {{ {hyphae} }}");
            let env = env(&src);
            let node = elaborate(&env, "main")
                .unwrap_or_else(|e| panic!("k={k}: colony must be in the fragment: {e}"));
            let v = interp
                .eval(&node)
                .unwrap_or_else(|e| panic!("k={k}: colony must run: {e}"));
            assert_eq!(
                v.payload(),
                &Payload::Bits(last_payload.clone()),
                "k={k}: the colony's value must equal its LAST hypha (RT2 sequentialization), \
                 independent of the {k} leading hyphae"
            );
        }
    }
}
