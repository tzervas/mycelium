//! The **v0 monomorphic typechecker** (RFC-0007 §4.4) plus the program environment it checks
//! against: the data-type registry (declarations are registry entries, never term nodes —
//! RFC-0007 §4.2) and the function table. Every refusal is an explicit [`CheckError`] — generics,
//! `spore`, value-level integers without context, and `wild` blocks (denied by default, LR-9) are
//! *refused with a reason*, never guessed at.

use std::cell::Cell;
use std::collections::BTreeMap;

use crate::ambient::AmbientError;
use crate::ast::{
    BaseType, Expr, FnDecl, FnSig, Hypha, ImplDecl, Item, Literal, Nodule, Paradigm, Path, Pattern,
    Scalar, Strength, TraitRef, TypeDecl, TypeRef,
};

/// The checker's **explicit expression-nesting budget** (the "banked guard 4" discipline; A4-02).
/// Type-checking recurses on the expression AST; rather than rely on the host call stack to bound
/// that recursion (a resource that varies by thread and by IR frame size — never a semantic limit),
/// the checker carries this reified budget and refuses past it with a clean [`CheckError`], exactly as
/// the parser ([`crate::parse::MAX_EXPR_DEPTH`]) and the evaluator (`eval::DEFAULT_DEPTH`) do for their
/// recursions. It is set comfortably **above** the parser's surface-nesting cap (so it never trips for
/// parser-produced ASTs — it is the defense-in-depth ceiling for an AST handed straight to the checker
/// via the API), and the recursion runs on a deep worker stack ([`mycelium_stack`]) so this budget — not a
/// host-stack overflow — is always what bounds a pathological input.
///
/// **Grounding (measured, not guessed).** The 256 MiB worker stack physically supports **~24,600**
/// levels of `check` recursion in a debug build (empirically: 24,589 survives, 24,765 aborts;
/// ~10.9 KiB/frame — release frames are smaller, so the ceiling is *higher* there). This budget
/// (`4096`) is therefore a **~6× safety margin** below the measured physical ceiling, and **16×**
/// above the parser's 256-deep surface cap — so a real (parsed) program is never within ~16× of it,
/// and even a synthetic AST refuses cleanly with ~6× stack headroom to spare. Raising it is safe up to
/// roughly a third of the physical ceiling; widen the worker stack first if more is ever wanted.
///
/// **Self-hosting:** this explicit budget is the portable primitive (it carries over to the
/// Mycelium-native frontend's clocked bounded-computation model); the worker stack is the transitional
/// Rust-only adapter (`mycelium_stack`).
pub const MAX_CHECK_DEPTH: u32 = 4096;

/// RAII depth accounting for the checker's recursive [`Cx::check`] (paired with [`MAX_CHECK_DEPTH`]):
/// increments the live nesting depth on entry and decrements it on drop, so the budget is honoured on
/// **every** exit path (early `return`, `?`, or fall-through) — never a counter that leaks on error.
struct DepthGuard<'a>(&'a Cell<u32>);

impl Drop for DepthGuard<'_> {
    fn drop(&mut self) {
        self.0.set(self.0.get().saturating_sub(1));
    }
}

/// A checked type. Stage-0 is monomorphic; stage-1 (RFC-0007 §11) adds **type parameters as
/// abstract variables** ([`Ty::Var`]) and **applied data types** ([`Ty::Data`] with arguments).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    /// `Binary{n}`.
    Binary(u32),
    /// `Ternary{m}`.
    Ternary(u32),
    /// `Dense{d, s}`.
    Dense(u32, Scalar),
    /// A registered data type applied to type arguments — `Data("List", [Binary(8)])` is
    /// `List<Binary{8}>`; an empty argument vector is a monomorphic/nullary type (`Data("Bool", [])`).
    /// Content addressing of declarations: RFC-0007 §4.2 (parameterized declarations are one registry
    /// entry); the prototype keys by name since v0 is single-nodule.
    Data(String, Vec<Ty>),
    /// `Substrate{tag}` — the affine external-resource kind (LR-8). No value forms exist in v0.
    Substrate(String),
    /// An **abstract type parameter** (a skolem variable) — in scope only while checking a generic
    /// declaration's constructors or a generic function's body (RFC-0007 §11.2). Two `Var`s are equal
    /// iff their names match; that structural equality is the engine of parametric checking. A
    /// `Var`-typed value is **representation-opaque**: no representation-specific `Op` may apply to it
    /// (this is the unbounded-case form of RFC-0019 §4.6's Repr-polymorphism restriction — it falls
    /// out of the abstract-variable discipline, restating S1 at the polymorphic level).
    Var(String),
}

impl core::fmt::Display for Ty {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Ty::Binary(n) => write!(f, "Binary{{{n}}}"),
            Ty::Ternary(m) => write!(f, "Ternary{{{m}}}"),
            Ty::Dense(d, s) => write!(f, "Dense{{{d}, {s:?}}}"),
            Ty::Data(n, args) if args.is_empty() => write!(f, "{n}"),
            Ty::Data(n, args) => {
                write!(f, "{n}<")?;
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{a}")?;
                }
                write!(f, ">")
            }
            Ty::Substrate(t) => write!(f, "Substrate{{{t}}}"),
            Ty::Var(v) => write!(f, "{v}"),
        }
    }
}

/// An explicit check failure (never a silent pass or a guess — S5/G2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckError {
    /// Which function (or item) the error is in, when known.
    pub site: String,
    /// What went wrong.
    pub message: String,
}

impl CheckError {
    fn new(site: &str, message: impl Into<String>) -> Self {
        CheckError {
            site: site.to_owned(),
            message: message.into(),
        }
    }

    /// Public, ergonomic constructor: a check failure at `site` with `message`. Mirrors the
    /// crate-internal `CheckError::new` (which stays private) so external callers — the
    /// toolchain crates that surface L1 diagnostics — can build a [`CheckError`] without reaching
    /// through the struct fields (Law of Demeter). Additive; no existing signature changes.
    #[must_use]
    pub fn at(site: impl Into<String>, message: impl Into<String>) -> Self {
        CheckError {
            site: site.into(),
            message: message.into(),
        }
    }
}

impl core::fmt::Display for CheckError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "check error in `{}`: {}", self.site, self.message)
    }
}

impl std::error::Error for CheckError {}

/// A resolution-pass refusal flows through the unified [`CheckError`] (the toolchain's one
/// surface→checked-program error), preserving the never-silent site + message (RFC-0012 §4.3/§4.4).
impl From<AmbientError> for CheckError {
    fn from(e: AmbientError) -> Self {
        let site = match &e {
            AmbientError::UnresolvedAmbient { site }
            | AmbientError::ParadigmShapeMismatch { site, .. }
            | AmbientError::BareDecimalNoEncoding { site, .. } => site.clone(),
            AmbientError::MultipleDefaults { .. } => "<nodule>".to_owned(),
        };
        CheckError {
            site,
            message: e.to_string(),
        }
    }
}

/// One constructor of a registered data type.
#[derive(Debug, Clone, PartialEq)]
pub struct CtorInfo {
    /// Constructor name.
    pub name: String,
    /// Field types.
    pub fields: Vec<Ty>,
}

/// A registered data type. **Stage-1 (RFC-0007 §11):** `params` are the type parameters (empty for a
/// monomorphic type); a constructor's [`CtorInfo::fields`] may reference them as [`Ty::Var`]. The
/// fields are stored *abstractly* (over the parameters) — [`subst_ty`] instantiates them at concrete
/// type arguments when a value is constructed or matched.
#[derive(Debug, Clone, PartialEq)]
pub struct DataInfo {
    /// Type name.
    pub name: String,
    /// Type parameters, in declaration order (empty for a monomorphic type). `List<A>` ⇒ `["A"]`.
    pub params: Vec<String>,
    /// Constructors, in declaration order (the index is the `#type#i` of RFC-0007 §4.2). Field types
    /// are abstract over `params` (may contain [`Ty::Var`]).
    pub ctors: Vec<CtorInfo>,
}

/// A registered **trait** (RFC-0019 §4.2; LR-2). Stage-1 is single-parameter, but the structure
/// carries `params: Vec<String>` uniformly (the trait's type-variable names, in scope as `Ty::Var`
/// while its method signatures are checked). The method `sigs` are stored as their surface
/// [`FnSig`]s; an `impl`'s methods are checked against the trait's sigs with `params ↦ trait_args`.
#[derive(Debug, Clone, PartialEq)]
pub struct TraitInfo {
    /// Trait name.
    pub name: String,
    /// Type-parameter names (single-parameter in stage-1; abstract over the method sigs).
    pub params: Vec<String>,
    /// The required method signatures, in declaration order.
    pub sigs: Vec<FnSig>,
}

/// A registered **instance** `impl Trait<args> for T` (RFC-0019 §4.5). Keyed in [`Env::instances`]
/// by `(trait_name, type_head(for_ty))`; the full `for_ty`/`trait_args` are kept for `EXPLAIN` and
/// the staged dictionary lowering (M-673).
#[derive(Debug, Clone, PartialEq)]
pub struct InstanceInfo {
    /// The trait this instances.
    pub trait_name: String,
    /// The (concrete) trait arguments.
    pub trait_args: Vec<Ty>,
    /// The (concrete) type the instance is for.
    pub for_ty: Ty,
    /// The provided method names (a non-silent record of what the dictionary supplies).
    pub methods: Vec<String>,
}

/// The **coherence key** of a type (RFC-0019 §4.5): the head a `(trait, type-head)` instance key is
/// computed from. Width/shape is intentionally **erased** — stage-1 keys per *head*, conservatively
/// rejecting two instances on the same head even at different widths (a documented, deferrable
/// refinement; the alternative, width-granular keying, needs the role/variance machinery deferred to
/// v2). A bare type **variable** is not a legal instance head in stage-1 (an `impl … for T` over an
/// abstract `T` would be a blanket instance — refused explicitly, never silently), so `Ty::Var`
/// yields `None`.
#[must_use]
pub fn type_head(ty: &Ty) -> Option<String> {
    Some(match ty {
        Ty::Binary(_) => "Binary".to_owned(),
        Ty::Ternary(_) => "Ternary".to_owned(),
        Ty::Dense(_, _) => "Dense".to_owned(),
        Ty::Substrate(t) => format!("Substrate:{t}"),
        Ty::Data(n, _) => format!("Data:{n}"),
        Ty::Var(_) => return None,
    })
}

/// Substitute type arguments for the abstract parameters in a stage-1 type (RFC-0007 §11.2): replace
/// each [`Ty::Var`] by its binding in `s`, recursing into [`Ty::Data`] arguments. A `Var` with no
/// binding is left as-is (it is a parameter still in scope — e.g. while checking a generic body). This
/// is total and never inserts a `Swap` (S1): it only renames/instantiates type structure.
pub(crate) fn subst_ty(ty: &Ty, s: &BTreeMap<String, Ty>) -> Ty {
    match ty {
        Ty::Var(v) => s.get(v).cloned().unwrap_or_else(|| ty.clone()),
        Ty::Data(n, args) => Ty::Data(n.clone(), args.iter().map(|a| subst_ty(a, s)).collect()),
        Ty::Binary(_) | Ty::Ternary(_) | Ty::Dense(_, _) | Ty::Substrate(_) => ty.clone(),
    }
}

/// Build the parameter→argument substitution for a data type's constructor fields (RFC-0007 §11.2):
/// pairs each declared parameter name with the corresponding concrete type argument. A mismatched
/// length yields a partial map (the caller has already arity-checked, or is in a position where the
/// extra/missing entries simply do not substitute — never a panic).
pub(crate) fn param_subst(params: &[String], args: &[Ty]) -> BTreeMap<String, Ty> {
    params.iter().cloned().zip(args.iter().cloned()).collect()
}

/// Does `ty` mention any abstract type parameter ([`Ty::Var`])? Used to decide whether a (partially
/// substituted) declared type is concrete enough to drive a bidirectional check, or must let the
/// argument synthesize its own type so the parameter can be inferred (RFC-0007 §11.3).
pub(crate) fn has_var(ty: &Ty) -> bool {
    match ty {
        Ty::Var(_) => true,
        Ty::Data(_, args) => args.iter().any(has_var),
        Ty::Binary(_) | Ty::Ternary(_) | Ty::Dense(_, _) | Ty::Substrate(_) => false,
    }
}

/// One-sided **unification** of a declared type (which may contain [`Ty::Var`] parameters) against a
/// concrete `actual` type, accumulating the parameter substitution `s` (RFC-0007 §11.3). Used to
/// **infer** a generic function's type arguments from its call-site argument types, and a constructor's
/// from its field arguments. A parameter is bound at most once; a second, conflicting binding is an
/// explicit mismatch (never a silent re-coercion — G2/VR-5). No `Swap` is ever inserted: a
/// representation-level disagreement (`Binary{8}` vs `Binary{16}`) is an explicit error, not a
/// conversion (S1).
pub(crate) fn unify(
    site: &str,
    decl: &Ty,
    actual: &Ty,
    s: &mut BTreeMap<String, Ty>,
) -> Result<(), CheckError> {
    match (decl, actual) {
        (Ty::Var(v), _) => match s.get(v) {
            Some(bound) if bound != actual => Err(CheckError::new(
                site,
                format!(
                    "type parameter `{v}` would have to be both {bound} and {actual} — \
                     an ambiguous instantiation, not a guess (RFC-0007 §11.3)"
                ),
            )),
            _ => {
                s.insert(v.clone(), actual.clone());
                Ok(())
            }
        },
        // A parameter appearing on the concrete side (nested generic call inside a generic body):
        // treat as equality (both must already agree).
        (_, Ty::Var(_)) if decl == actual => Ok(()),
        (Ty::Data(n1, a1), Ty::Data(n2, a2)) if n1 == n2 && a1.len() == a2.len() => {
            for (d, a) in a1.iter().zip(a2) {
                unify(site, d, a, s)?;
            }
            Ok(())
        }
        _ if decl == actual => Ok(()),
        _ => Err(CheckError::new(
            site,
            format!("cannot match {decl} against {actual} (RFC-0007 §11.3 — never a silent swap)"),
        )),
    }
}

/// The checked program environment: registry + function table. Built by [`check_nodule`]; the
/// evaluator and elaborator consume it (so nothing runs unchecked).
#[derive(Debug, Clone)]
pub struct Env {
    /// Data registry, keyed by type name.
    pub types: BTreeMap<String, DataInfo>,
    /// Function table, keyed by name.
    pub fns: BTreeMap<String, FnDecl>,
    /// Per-function totality classification (RFC-0007 §4.5), filled by the totality checker.
    pub totality: BTreeMap<String, crate::totality::Totality>,
    /// Trait registry (RFC-0019 §4.2; LR-2), keyed by trait name. A trait is a **registry entry**,
    /// never a kernel node (KC-3). Stored for `EXPLAIN` and for the staged dictionary lowering
    /// (M-673).
    pub traits: BTreeMap<String, TraitInfo>,
    /// Instance registry (RFC-0019 §4.5; the **coherence** key), keyed by `(trait_name, type_head)`.
    /// Head-granular keying is the stage-1 coherence discipline — at most one instance per
    /// `(trait, type-head)` (global uniqueness). Stored for instance resolution + the staged
    /// dictionary lowering (M-673).
    pub instances: BTreeMap<(String, String), InstanceInfo>,
}

impl Env {
    /// Find the data type owning constructor `ctor`, with its index — `None` if no type has it.
    #[must_use]
    pub fn ctor(&self, ctor: &str) -> Option<(&DataInfo, usize)> {
        self.types
            .values()
            .find_map(|d| d.ctors.iter().position(|c| c.name == ctor).map(|i| (d, i)))
    }

    /// The registered data type named `name`, if any. Additive read-only accessor over the public
    /// [`types`](Env::types) map — a Law-of-Demeter-friendly alternative to `env.types.get(name)`.
    #[must_use]
    pub fn type_info(&self, name: &str) -> Option<&DataInfo> {
        self.types.get(name)
    }

    /// The function declaration named `name`, if any. Additive read-only accessor over the public
    /// [`fns`](Env::fns) map.
    #[must_use]
    pub fn fn_decl(&self, name: &str) -> Option<&FnDecl> {
        self.fns.get(name)
    }

    /// The totality verdict for function `name`, if it has been classified. Additive read-only
    /// accessor over the public [`totality`](Env::totality) map. `Totality` is `Copy`, so this
    /// returns it by value (never silently fabricating a verdict for an unclassified name — `None`).
    #[must_use]
    pub fn fn_totality(&self, name: &str) -> Option<crate::totality::Totality> {
        self.totality.get(name).copied()
    }

    /// The registered trait named `name`, if any (RFC-0019 §4.2). Additive read-only accessor over
    /// the public [`traits`](Env::traits) map.
    #[must_use]
    pub fn trait_info(&self, name: &str) -> Option<&TraitInfo> {
        self.traits.get(name)
    }

    /// The registered instance for `(trait_name, head)`, if any (RFC-0019 §4.5). `head` is a
    /// [`type_head`]. Additive read-only accessor over the public [`instances`](Env::instances) map.
    #[must_use]
    pub fn instance(&self, trait_name: &str, head: &str) -> Option<&InstanceInfo> {
        self.instances
            .get(&(trait_name.to_owned(), head.to_owned()))
    }
}

/// The builtin prelude: `type Bool = False | True` (`if` scrutinizes it; RFC-0007 keeps `if` as
/// elaboration-level sugar over `Match` on this registry entry).
fn prelude() -> DataInfo {
    DataInfo {
        name: "Bool".to_owned(),
        params: vec![],
        ctors: vec![
            CtorInfo {
                name: "False".to_owned(),
                fields: vec![],
            },
            CtorInfo {
                name: "True".to_owned(),
                fields: vec![],
            },
        ],
    }
}

/// Resolve a surface [`TypeRef`] to a checked [`Ty`], with the type parameters `tyvars` in scope
/// (RFC-0007 §11.2): a `Named(name, [])` whose `name` is a type parameter resolves to [`Ty::Var`];
/// any other `Named` is a data type whose **arity is checked** against its declaration (`List<A>`
/// applied to the wrong number of arguments is an explicit error, never a guess). VSA types stay a
/// deferred refusal. The guarantee index is *allowed* and returned alongside (checked dynamically at
/// stage 0 — RFC-0007 §4.3). `tyvars` is `&[]` in a monomorphic context.
pub(crate) fn resolve_ty(
    site: &str,
    types: &BTreeMap<String, DataInfo>,
    tyvars: &[String],
    t: &TypeRef,
) -> Result<(Ty, Option<Strength>), CheckError> {
    let base = match &t.base {
        BaseType::Binary(n) => Ty::Binary(*n),
        BaseType::Ternary(m) => Ty::Ternary(*m),
        BaseType::Dense(d, s) => Ty::Dense(*d, *s),
        BaseType::Substrate(tag) => Ty::Substrate(tag.clone()),
        BaseType::Vsa { .. } => {
            return Err(CheckError::new(
                site,
                "VSA types are deferred in the L1 v0 prototype (no value forms yet)",
            ))
        }
        BaseType::Named(name, args) => {
            // A bare name that is a type parameter in scope is an abstract type variable (§11.2).
            if args.is_empty() && tyvars.iter().any(|v| v == name) {
                Ty::Var(name.clone())
            } else {
                let Some(decl) = types.get(name) else {
                    return Err(CheckError::new(site, format!("unknown type `{name}`")));
                };
                // Arity is checked — never a guess (§11.3). A type parameter cannot be applied.
                if args.len() != decl.params.len() {
                    return Err(CheckError::new(
                        site,
                        format!(
                            "`{name}` takes {} type argument(s), got {} (RFC-0007 §11.3)",
                            decl.params.len(),
                            args.len()
                        ),
                    ));
                }
                let mut resolved = Vec::with_capacity(args.len());
                for a in args {
                    resolved.push(resolve_ty(site, types, tyvars, a)?.0);
                }
                Ty::Data(name.clone(), resolved)
            }
        }
        BaseType::Ambient(_) => {
            // The resolution pass ([`crate::ambient`]) fills every paradigm-less repr before the
            // checker runs; a residual one is an internal invariant break, refused explicitly (never
            // a silent guess) — defense in depth (RFC-0012 §4.3).
            return Err(CheckError::new(
                site,
                "internal: an unresolved paradigm-less repr `{…}` reached the checker — the \
                 ambient resolution pass should have filled it (RFC-0012 §4.3)",
            ));
        }
    };
    Ok((base, t.guarantee))
}

/// Check a whole nodule: build the registry (prelude + declarations), then type every function
/// body against its signature, classify totality. No maturation gate is applied (the scope is
/// treated as non-matured). Returns the checked [`Env`].
///
/// As of M-344 (RFC-0012) the input is first run through the **ambient resolution pass**
/// ([`crate::ambient::resolve`]) — paradigm-less reprs are filled, `with paradigm` blocks stripped,
/// bare decimals tagged — so the checker only ever sees fully-explicit (longhand) forms. A program
/// using no ambient is unchanged (resolution is identity).
pub fn check_nodule(nodule: &Nodule) -> Result<Env, CheckError> {
    check_and_resolve(nodule).map(|(env, _)| env)
}

/// Like [`check_nodule`] but with an explicit `matured_scope` flag (RFC-0017 §4.2): when `true`,
/// every reachable definition whose `thaw == false` must be `Total` (the existing totality
/// classifier, unchanged) — a non-total non-thaw definition is an explicit `CheckError`. Definitions
/// marked `thaw` are exempt from the gate (RFC-0017 §4.3). When `matured_scope` is `false` this
/// is identical to [`check_nodule`].
pub fn check_nodule_matured(nodule: &Nodule, matured_scope: bool) -> Result<Env, CheckError> {
    check_and_resolve_matured(nodule, matured_scope).map(|(env, _)| env)
}

fn check_and_resolve_matured(
    nodule: &Nodule,
    matured_scope: bool,
) -> Result<(Env, Nodule), CheckError> {
    // Run the recursive pass on a deep worker stack so deep-but-valid input never overflows the
    // *caller's* thread stack — the explicit [`MAX_CHECK_DEPTH`] budget, not the host stack, bounds a
    // pathological input (banked guard 4; the worker stack is the transitional Rust-only adapter —
    // see [`mycelium_stack`]). Borrows are fine: the worker is a scoped thread.
    mycelium_stack::with_deep_stack(|| check_and_resolve_matured_inner(nodule, matured_scope))
}

fn check_and_resolve_matured_inner(
    nodule: &Nodule,
    matured_scope: bool,
) -> Result<(Env, Nodule), CheckError> {
    let resolved = crate::ambient::resolve(nodule)?;
    let env = check_resolved_matured(&resolved, matured_scope)?;
    let mut items = Vec::with_capacity(resolved.items.len());
    for item in &resolved.items {
        match item {
            Item::Fn(fd) => {
                let resolved_fd = env
                    .fns
                    .get(&fd.sig.name)
                    .cloned()
                    .unwrap_or_else(|| fd.clone());
                items.push(Item::Fn(resolved_fd));
            }
            other => items.push(other.clone()),
        }
    }
    let twin = Nodule {
        path: resolved.path.clone(),
        items,
    };
    Ok((env, twin))
}

/// Like [`check_nodule`], but also returns the **fully-resolved longhand twin** of the program
/// (paradigm tags filled *and* bare-decimal widths resolved from context) — the source the M-142/LSP
/// "expand ambient" projection renders (RFC-0012 §5). The returned [`Nodule`] elaborates to the
/// identical L0 (and content hash) as the original (I2; RFC-0012 §4.3).
pub fn check_and_resolve(nodule: &Nodule) -> Result<(Env, Nodule), CheckError> {
    check_and_resolve_matured(nodule, false)
}

/// The core checker, run on an already ambient-resolved nodule, with an explicit maturation flag.
/// When `matured_scope` is true, every fn with `thaw == false` must be `Total` (RFC-0017 §4.2).
fn check_resolved_matured(nodule: &Nodule, matured_scope: bool) -> Result<Env, CheckError> {
    let mut types = BTreeMap::new();
    let p = prelude();
    types.insert(p.name.clone(), p);

    // Pass 1: register data declarations (so they can reference each other). Stage-1 (RFC-0007 §11):
    // a **generic** declaration `type List<A> = …` registers its type parameters; its constructor
    // fields are resolved abstractly (with the parameters in scope), so they may contain `Ty::Var`.
    for item in &nodule.items {
        if let Item::Type(td) = item {
            if let Some(dup) = first_duplicate(&td.params) {
                return Err(CheckError::new(
                    &td.name,
                    format!("duplicate type parameter `{dup}` in `{}`", td.name),
                ));
            }
            if types.contains_key(&td.name) {
                return Err(CheckError::new(&td.name, "duplicate type declaration"));
            }
            // Insert a shell first so recursive field references resolve.
            types.insert(
                td.name.clone(),
                DataInfo {
                    name: td.name.clone(),
                    params: td.params.clone(),
                    ctors: vec![],
                },
            );
        }
    }
    for item in &nodule.items {
        if let Item::Type(td) = item {
            let ctors = resolve_ctors(&types, td)?;
            types.get_mut(&td.name).expect("registered above").ctors = ctors;
        }
    }

    // Pass 1b: register trait declarations (RFC-0019 §4.2; LR-2). A trait is a registry entry, never
    // a kernel node (KC-3). Its method signatures must resolve with the trait's params in scope as
    // abstract type-variables; duplicate trait names and duplicate method names are explicit errors.
    let traits = register_traits(&types, nodule)?;

    // Pass 2: collect functions (signatures must resolve). Stage-1: a **generic** function
    // `fn head<A>(…)` is registered as-is; its body is checked in Pass 3 with `A` an abstract
    // variable, and call sites instantiate it (RFC-0007 §11.2/§11.3).
    let mut fns: BTreeMap<String, FnDecl> = BTreeMap::new();
    for item in &nodule.items {
        match item {
            Item::Fn(fd) => {
                if let Some(dup) = first_duplicate(&fd.sig.param_names()) {
                    return Err(CheckError::new(
                        &fd.sig.name,
                        format!("duplicate type parameter `{dup}` in `{}`", fd.sig.name),
                    ));
                }
                if fns.insert(fd.sig.name.clone(), fd.clone()).is_some() {
                    return Err(CheckError::new(&fd.sig.name, "duplicate function"));
                }
            }
            // `default` is consumed by the resolution pass; it never reaches `check_resolved`.
            // Traits were registered in Pass 1b; impls are handled in Pass 2b (below).
            Item::Default(_) | Item::Trait(_) | Item::Impl(_) | Item::Use(_) | Item::Type(_) => {}
        }
    }

    // Pass 2b: register trait **instances** (RFC-0019 §4.5). Coherence (global uniqueness + the
    // orphan rule) is enforced as each instance is registered; ALL instances are registered before
    // any method body is checked (Pass 3b), so a method body may rely on an instance declared by a
    // *later* `impl`. This pass resolves heads + checks coherence; it does not yet check bodies.
    let instances = register_instances(&types, &traits, nodule)?;

    // Pass 3: type every body **against** its declared return type (bidirectional, RFC-0012 §4.3)
    // and resolve any ambient bare-decimal widths from context — rewriting each body so the
    // downstream evaluator/elaborator see only concrete literals.
    let mut resolved_fns: BTreeMap<String, FnDecl> = BTreeMap::new();
    for fd in fns.values() {
        // Stage-1: the function's type parameters are in scope while resolving its signature and
        // checking its body — each resolves to a `Ty::Var` (RFC-0007 §11.2). Bounds (RFC-0019 §4.1)
        // are validated and carried so an unqualified trait-method call in the body can type through
        // a bound (the dictionary it stands for is staged — RFC-0007 §12.3 / M-673).
        let (body, _ret) = check_fn_body(&types, &fns, &traits, &instances, fd)?;
        resolved_fns.insert(
            fd.sig.name.clone(),
            FnDecl {
                thaw: fd.thaw,
                sig: fd.sig.clone(),
                body,
            },
        );
    }
    let fns = resolved_fns;

    // Pass 3b: check each `impl` method body against its **expected** signature (the trait sig with
    // the trait's params substituted by this impl's trait_args — RFC-0019 §4.5). The instance set is
    // fully registered (Pass 2b), so a method may use any instance. Bodies are checked but not
    // re-stored (the elaborator stages the dictionary lowering — M-673), only validated.
    for item in &nodule.items {
        if let Item::Impl(id) = item {
            check_impl_methods(&types, &fns, &traits, &instances, id)?;
        }
    }

    // Pass 4: totality classification + the scope-quantified matured gate (RFC-0017 §4.2).
    // When `matured_scope` is true, every fn with `thaw == false` must be `Total`; a non-total
    // non-thaw fn is an explicit error (RFC-0007 §4.5 / RFC-0017 §4.2). A `thaw` fn is exempt.
    let totality = crate::totality::classify_all(&fns);
    if matured_scope {
        for fd in fns.values() {
            if !fd.thaw && totality[&fd.sig.name] != crate::totality::Totality::Total {
                return Err(CheckError::new(
                    &fd.sig.name,
                    format!(
                        "`{}` is in a matured scope and must be total (RFC-0007 §4.5 / \
                         RFC-0017 §4.2) — mark it `thaw fn` to exempt it, or make it total",
                        fd.sig.name
                    ),
                ));
            }
        }
    }

    Ok(Env {
        types,
        fns,
        totality,
        traits,
        instances,
    })
}

fn resolve_ctors(
    types: &BTreeMap<String, DataInfo>,
    td: &TypeDecl,
) -> Result<Vec<CtorInfo>, CheckError> {
    let mut ctors = Vec::new();
    for c in &td.ctors {
        if ctors.iter().any(|x: &CtorInfo| x.name == c.name) {
            return Err(CheckError::new(
                &td.name,
                format!("duplicate constructor `{}`", c.name),
            ));
        }
        let mut fields = Vec::new();
        for f in &c.fields {
            // The declaration's type parameters are in scope, so a field may be an abstract
            // `Ty::Var` (`Cons(A, List<A>)` ⇒ fields `[Var("A"), Data("List", [Var("A")])]`).
            let (ty, _) = resolve_ty(&td.name, types, &td.params, f)?;
            fields.push(ty);
        }
        ctors.push(CtorInfo {
            name: c.name.clone(),
            fields,
        });
    }
    Ok(ctors)
}

/// The first value that appears more than once in `xs` (left to right), if any. Used to reject
/// duplicate type-parameter names — an explicit error, never a silently-shadowed binding (G2).
fn first_duplicate(xs: &[String]) -> Option<&String> {
    let mut seen = std::collections::BTreeSet::new();
    xs.iter().find(|x| !seen.insert((*x).clone()))
}

/// **Trait pass** (RFC-0019 §4.2; LR-2 — guarantee: `Declared`, a structural registry check, not a
/// theorem). Register each `trait Tr<params> { fn … }` as a [`TraitInfo`]: reject a duplicate trait
/// name; reject duplicate type-parameter names and duplicate method names; and check that **every**
/// method signature *resolves* with the trait's params in scope as abstract type-variables (its
/// value-param types and return type via [`resolve_ty`]). A trait is a registry entry, never a
/// kernel node (KC-3). Every refusal is an explicit [`CheckError`] (G2).
fn register_traits(
    types: &BTreeMap<String, DataInfo>,
    nodule: &Nodule,
) -> Result<BTreeMap<String, TraitInfo>, CheckError> {
    let mut traits: BTreeMap<String, TraitInfo> = BTreeMap::new();
    for item in &nodule.items {
        let Item::Trait(td) = item else { continue };
        let site = &td.name;
        if let Some(dup) = first_duplicate(&td.params) {
            return Err(CheckError::new(
                site,
                format!("duplicate type parameter `{dup}` in trait `{}`", td.name),
            ));
        }
        if traits.contains_key(&td.name) {
            return Err(CheckError::new(site, "duplicate trait declaration"));
        }
        // Method names must be distinct within the trait (a duplicated requirement is ambiguous).
        let mut seen_methods: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
        for s in &td.sigs {
            if !seen_methods.insert(s.name.as_str()) {
                return Err(CheckError::new(
                    site,
                    format!("duplicate method `{}` in trait `{}`", s.name, td.name),
                ));
            }
            // Each method sig must resolve with the trait's params (and the method's own params) in
            // scope as type-variables (RFC-0007 §11.2). A method may carry its own type-params too;
            // bounds on them are validated against the complete trait registry in the pass below (G2).
            let mut tyvars = td.params.clone();
            tyvars.extend(s.param_names());
            check_sig_resolves(types, site, &tyvars, s)?;
        }
        traits.insert(
            td.name.clone(),
            TraitInfo {
                name: td.name.clone(),
                params: td.params.clone(),
                sigs: td.sigs.clone(),
            },
        );
    }
    // The registry is now complete: validate that every trait-method type-parameter BOUND names a
    // KNOWN trait. This is a second pass precisely so a bound may forward-reference a later-declared
    // trait. An unknown bound (`fn f<T: Nope>(…)`) is an explicit error here, never silently
    // registered (G2) — otherwise the ill-formed requirement would surface only at an unrelated
    // later check, or never (if no impl exercises it).
    for tr in traits.values() {
        for s in &tr.sigs {
            for tp in &s.params {
                for b in &tp.bounds {
                    if !traits.contains_key(&b.name) {
                        return Err(CheckError::new(
                            &tr.name,
                            format!(
                                "trait `{}` method `{}`: unknown trait `{}` in the bound `{}: {}` \
                                 (RFC-0019 §4.1 — a method's type-parameter bound must name a \
                                 declared trait)",
                                tr.name, s.name, b.name, tp.name, b.name
                            ),
                        ));
                    }
                }
            }
        }
    }
    Ok(traits)
}

/// Confirm a [`FnSig`]'s value-parameter types and return type all resolve under `tyvars` (the
/// abstract type-variables in scope). Shared by the trait pass (method requirements) and the
/// bounded-fn body checker (signature validation). Does not check the body — only the signature.
fn check_sig_resolves(
    types: &BTreeMap<String, DataInfo>,
    site: &str,
    tyvars: &[String],
    sig: &FnSig,
) -> Result<(), CheckError> {
    for p in &sig.value_params {
        resolve_ty(site, types, tyvars, &p.ty)?;
    }
    resolve_ty(site, types, tyvars, &sig.ret)?;
    Ok(())
}

/// **Impl pass — registration + coherence** (RFC-0019 §4.5; LR-2 — guarantee: `Declared`, the
/// coherence argument is Declared-with-argument per RFC-0019, not machine-checked). For each
/// `impl Trait<args> for T`:
/// - resolve `for_ty` and each `trait_args` to a **concrete** [`Ty`] (no type-variables in scope);
/// - the trait must exist and the argument **arity** must match the trait's params (else explicit);
/// - **global uniqueness:** key `(trait, type_head(for_ty))` must be free, else an overlapping-
///   instance / coherence refusal naming the pair (RFC-0019 §4.5; ADR-003);
/// - **orphan rule (single-nodule stage-1):** the trait is declared here, OR `for_ty` is a `Data`
///   declared here, OR `for_ty` is a primitive repr type (`Binary`/`Ternary`/`Dense`/`Substrate`);
///   otherwise an explicit orphan refusal (cross-nodule enforcement is staged with the phylum work,
///   M-662).
///
/// Method *bodies* are not checked here (that is [`check_impl_methods`], after the whole instance
/// set is known); method *presence* (exact-match against the trait's sigs) IS checked here.
fn register_instances(
    types: &BTreeMap<String, DataInfo>,
    traits: &BTreeMap<String, TraitInfo>,
    nodule: &Nodule,
) -> Result<BTreeMap<(String, String), InstanceInfo>, CheckError> {
    // Which traits / data types are declared in *this* nodule (the orphan-rule locality test).
    let local_traits: std::collections::BTreeSet<&str> = nodule
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Trait(td) => Some(td.name.as_str()),
            _ => None,
        })
        .collect();
    let local_types: std::collections::BTreeSet<&str> = nodule
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Type(td) => Some(td.name.as_str()),
            _ => None,
        })
        .collect();

    let mut instances: BTreeMap<(String, String), InstanceInfo> = BTreeMap::new();
    for item in &nodule.items {
        let Item::Impl(id) = item else { continue };
        let site = &id.trait_name;
        // The trait must exist.
        let Some(tr) = traits.get(&id.trait_name) else {
            return Err(CheckError::new(
                site,
                format!(
                    "`impl` for unknown trait `{}` (RFC-0019 §4.5)",
                    id.trait_name
                ),
            ));
        };
        // Resolve the `for` type and the trait arguments concretely (no type-variables in scope —
        // an instance head is a concrete type in stage-1).
        let (for_ty, _) = resolve_ty(site, types, &[], &id.for_ty)?;
        let mut trait_args = Vec::with_capacity(id.trait_args.len());
        for a in &id.trait_args {
            trait_args.push(resolve_ty(site, types, &[], a)?.0);
        }
        // Arity: the trait's params count must equal the written trait-argument count.
        if trait_args.len() != tr.params.len() {
            return Err(CheckError::new(
                site,
                format!(
                    "trait `{}` takes {} type argument(s), but this `impl` supplies {} \
                     (RFC-0019 §4.5)",
                    tr.name,
                    tr.params.len(),
                    trait_args.len()
                ),
            ));
        }
        // Coherence key — the type head (width/shape erased; stage-1 keys per head).
        let Some(head) = type_head(&for_ty) else {
            return Err(CheckError::new(
                site,
                format!(
                    "an `impl … for {for_ty}` over a bare type variable is not a legal instance \
                     head in stage-1 (no blanket instances — RFC-0019 §4.5); the `for` type must \
                     be concrete"
                ),
            ));
        };
        // Orphan rule (single-nodule stage-1; RFC-0019 §4.5): legal iff the trait is local, OR the
        // `for` type is a local data type, OR a primitive repr type.
        let trait_local = local_traits.contains(id.trait_name.as_str());
        let type_local = match &for_ty {
            Ty::Data(n, _) => local_types.contains(n.as_str()),
            // Primitive repr types are "owned here" for stage-1 single-nodule (RFC-0019 §4.5).
            Ty::Binary(_) | Ty::Ternary(_) | Ty::Dense(_, _) | Ty::Substrate(_) => true,
            Ty::Var(_) => false,
        };
        if !trait_local && !type_local {
            return Err(CheckError::new(
                site,
                format!(
                    "orphan instance: `impl {} for {for_ty}` is in neither the trait's nodule nor \
                     `{for_ty}`'s nodule (RFC-0019 §4.5 orphan rule); cross-nodule instances are \
                     staged with the phylum work (M-662)",
                    id.trait_name
                ),
            ));
        }
        // Global uniqueness — at most one instance per `(trait, head)` (RFC-0019 §4.5; ADR-003). A
        // duplicate (even at a different width on the same head — the documented stage-1 over-rejection)
        // is an explicit coherence refusal, never a silent shadow (G2).
        let key = (id.trait_name.clone(), head.clone());
        if instances.contains_key(&key) {
            return Err(CheckError::new(
                site,
                format!(
                    "overlapping instance — coherence/global-uniqueness violation (RFC-0019 §4.5): \
                     a second `impl {} for` a `{head}` type. Stage-1 keys per (trait, type-head), so \
                     two instances on the same head (even at different widths) conflict.",
                    id.trait_name
                ),
            ));
        }
        // Method presence: the impl's method set must EXACTLY match the trait's required sigs.
        check_impl_method_set(tr, id)?;
        instances.insert(
            key,
            InstanceInfo {
                trait_name: id.trait_name.clone(),
                trait_args,
                for_ty,
                methods: id.methods.iter().map(|m| m.sig.name.clone()).collect(),
            },
        );
    }
    Ok(instances)
}

/// The impl's method **set** must match the trait's requirement set **exactly** (RFC-0019 §4.5):
/// a missing method and an extra method are both explicit refusals (never silently filled or
/// dropped — G2). Signature/body agreement is [`check_impl_methods`]; this is only presence/names.
fn check_impl_method_set(tr: &TraitInfo, id: &ImplDecl) -> Result<(), CheckError> {
    let site = &id.trait_name;
    let required: std::collections::BTreeSet<&str> =
        tr.sigs.iter().map(|s| s.name.as_str()).collect();
    let provided: std::collections::BTreeSet<&str> =
        id.methods.iter().map(|m| m.sig.name.as_str()).collect();
    for need in &required {
        if !provided.contains(need) {
            return Err(CheckError::new(
                site,
                format!("impl of `{}` is missing method `{need}`", tr.name),
            ));
        }
    }
    for have in &provided {
        if !required.contains(have) {
            return Err(CheckError::new(
                site,
                format!(
                    "impl of `{}` has method `{have}` not in trait `{}`",
                    tr.name, tr.name
                ),
            ));
        }
    }
    // A method must not be provided twice (ambiguous dictionary slot).
    if let Some(dup) = first_duplicate(
        &id.methods
            .iter()
            .map(|m| m.sig.name.clone())
            .collect::<Vec<_>>(),
    ) {
        return Err(CheckError::new(
            site,
            format!(
                "impl of `{}` provides method `{dup}` more than once",
                tr.name
            ),
        ));
    }
    Ok(())
}

/// Check one `impl`'s method bodies against their **expected** signatures (RFC-0019 §4.5; guarantee:
/// `Declared` — a structural check). The expected signature of method `m` is the trait's sig for `m`
/// with the trait's params substituted by this impl's `trait_args`; the method body is checked the
/// normal fn-body way against those substituted value-param/return types. A signature mismatch
/// (wrong param types or arity, wrong return) is an explicit refusal, never silently accepted.
fn check_impl_methods(
    types: &BTreeMap<String, DataInfo>,
    fns: &BTreeMap<String, FnDecl>,
    traits: &BTreeMap<String, TraitInfo>,
    instances: &BTreeMap<(String, String), InstanceInfo>,
    id: &ImplDecl,
) -> Result<(), CheckError> {
    let tr = traits
        .get(&id.trait_name)
        .expect("instance registration checked the trait exists");
    // The trait-arg substitution `trait.params ↦ impl.trait_args` (resolved concretely).
    let (for_ty, _) = resolve_ty(&id.trait_name, types, &[], &id.for_ty)?;
    let mut trait_args = Vec::with_capacity(id.trait_args.len());
    for a in &id.trait_args {
        trait_args.push(resolve_ty(&id.trait_name, types, &[], a)?.0);
    }
    let s = param_subst(&tr.params, &trait_args);
    for method in &id.methods {
        let req = tr
            .sigs
            .iter()
            .find(|x| x.name == method.sig.name)
            .expect("method-set match checked at registration");
        let site = &method.sig.name;
        // Arity must match the requirement.
        if method.sig.value_params.len() != req.value_params.len() {
            return Err(CheckError::new(
                site,
                format!(
                    "impl method `{}` of `{}` takes {} parameter(s), but the trait requires {}",
                    method.sig.name,
                    tr.name,
                    method.sig.value_params.len(),
                    req.value_params.len()
                ),
            ));
        }
        // Each declared parameter type must equal the (substituted) required type; likewise the
        // return type. The method's own value-param types are resolved concretely (the impl is over a
        // concrete `for_ty`, so the method carries no abstract type-variables in stage-1).
        for (mp, rp) in method.sig.value_params.iter().zip(&req.value_params) {
            let (got, _) = resolve_ty(site, types, &[], &mp.ty)?;
            let want = subst_ty(&resolve_ty(site, types, &tr.params, &rp.ty)?.0, &s);
            if got != want {
                return Err(CheckError::new(
                    site,
                    format!(
                        "impl method `{}` parameter `{}`: {}",
                        method.sig.name,
                        mp.name,
                        edge_mismatch("type", &want, &got)
                    ),
                ));
            }
        }
        let (got_ret, _) = resolve_ty(site, types, &[], &method.sig.ret)?;
        let want_ret = subst_ty(&resolve_ty(site, types, &tr.params, &req.ret)?.0, &s);
        if got_ret != want_ret {
            return Err(CheckError::new(
                site,
                format!(
                    "impl method `{}` return: {}",
                    method.sig.name,
                    edge_mismatch("type", &want_ret, &got_ret)
                ),
            ));
        }
        // The body is checked the normal fn-body way (against the method's own — now validated —
        // signature). `for_ty` is concrete, so the body has no abstract type-variables; the full
        // trait/instance context is available so the body may itself call trait methods.
        check_fn_body(types, fns, traits, instances, method)?;
    }
    let _ = for_ty; // resolved above for the arg substitution; head reuse is at registration.
    Ok(())
}

/// Check a function (or impl method) body against its declared signature (RFC-0007 §11; RFC-0019
/// §4.1). Validates the type-parameter bounds, builds the `tyvars`/`bounds` scopes, resolves the
/// value-param + return types, and runs the bidirectional [`Cx::check`]. Returns the **resolved**
/// body (ambient bare-decimals filled) and the resolved return type. Shared by Pass 3 (top-level
/// fns) and [`check_impl_methods`] (impl methods) — DRY.
fn check_fn_body(
    types: &BTreeMap<String, DataInfo>,
    fns: &BTreeMap<String, FnDecl>,
    traits: &BTreeMap<String, TraitInfo>,
    instances: &BTreeMap<(String, String), InstanceInfo>,
    fd: &FnDecl,
) -> Result<(Expr, Ty), CheckError> {
    let site = &fd.sig.name;
    let tyvars = fd.sig.param_names();
    // Validate every bound names a real trait with the right argument arity (RFC-0019 §4.1). The
    // bound's trait-args may reference the fn's own type-variables (`T: Cmp<T>`), so resolve them
    // with `tyvars` in scope. A bound on an unknown trait / wrong arity is an explicit refusal.
    let bounds = check_bounds(types, traits, site, &tyvars, &fd.sig.params)?;
    let mut scope: Vec<(String, Ty)> = Vec::new();
    for p in &fd.sig.value_params {
        let (ty, _) = resolve_ty(site, types, &tyvars, &p.ty)?;
        scope.push((p.name.clone(), ty));
    }
    let (ret, _) = resolve_ty(site, types, &tyvars, &fd.sig.ret)?;
    let cx = Cx {
        site,
        types,
        fns,
        traits,
        instances,
        tyvars: &tyvars,
        bounds: &bounds,
        depth: Cell::new(0),
    };
    let (got, body) = cx.check(&mut scope, &fd.body, Some(&ret))?;
    if got != ret {
        return Err(CheckError::new(site, edge_mismatch("body", &ret, &got)));
    }
    Ok((body, ret))
}

/// Validate a function/method's type-parameter **bounds** (RFC-0019 §4.1): each bound must name a
/// registered trait with the correct type-argument arity, and the bound's trait-args must resolve
/// under `tyvars`. Returns the `(param, bounds)` pairs (only the bounded params) for the checking
/// context. Every refusal is explicit (G2). The dictionary the bound stands for is staged to
/// elaboration (RFC-0007 §12.3 / M-673) — the checker only validates satisfiability ("typing").
fn check_bounds(
    types: &BTreeMap<String, DataInfo>,
    traits: &BTreeMap<String, TraitInfo>,
    site: &str,
    tyvars: &[String],
    params: &[crate::ast::TypeParam],
) -> Result<Vec<(String, Vec<TraitRef>)>, CheckError> {
    let mut bounds: Vec<(String, Vec<TraitRef>)> = Vec::new();
    for p in params {
        for b in &p.bounds {
            let Some(tr) = traits.get(&b.name) else {
                return Err(CheckError::new(
                    site,
                    format!(
                        "bound `{}: {}` names unknown trait `{}` (RFC-0019 §4.1)",
                        p.name, b.name, b.name
                    ),
                ));
            };
            // Arity: written args must match the trait's params — **except** the canonical
            // elided-self form `T: Cmp` on a single-parameter trait, which is sugar for `T: Cmp<T>`
            // (Rust/Haskell `T: Cmp` ⇒ `Cmp T`). That elision is *only* valid for a single-param
            // trait with zero written args; any other count mismatch is an explicit refusal (G2).
            let elided_self = b.args.is_empty() && tr.params.len() == 1;
            if !elided_self && b.args.len() != tr.params.len() {
                return Err(CheckError::new(
                    site,
                    format!(
                        "bound `{}: {}<…>` supplies {} type argument(s), but trait `{}` takes {} \
                         (write `{}: {}` for the single-parameter self-bound, or supply all args)",
                        p.name,
                        b.name,
                        b.args.len(),
                        b.name,
                        tr.params.len(),
                        p.name,
                        b.name
                    ),
                ));
            }
            for a in &b.args {
                resolve_ty(site, types, tyvars, a)?;
            }
        }
        if !p.bounds.is_empty() {
            bounds.push((p.name.clone(), p.bounds.clone()));
        }
    }
    Ok(bounds)
}

/// The checking context for one function body.
struct Cx<'a> {
    site: &'a str,
    types: &'a BTreeMap<String, DataInfo>,
    fns: &'a BTreeMap<String, FnDecl>,
    /// Trait registry (RFC-0019 §4.2) — for resolving bounded-generic-call satisfiability and
    /// unqualified trait-method calls (`Tr::m`). Empty in re-inference (`infer_type`).
    traits: &'a BTreeMap<String, TraitInfo>,
    /// Instance registry (RFC-0019 §4.5), keyed by `(trait, type-head)` — the coherence map a
    /// bounded call / trait-method call resolves against. Empty in re-inference.
    instances: &'a BTreeMap<(String, String), InstanceInfo>,
    /// The type parameters in scope for this body (RFC-0007 §11.2) — empty for a monomorphic
    /// function. A bare `Named` type that matches one of these resolves to [`Ty::Var`].
    tyvars: &'a [String],
    /// The **bounds** on the type parameters in scope (`T: Cmp` ⇒ `("T", TraitRef{Cmp})`), so an
    /// unqualified trait-method call inside a bounded body can be typed through a bound (the
    /// dictionary it stands for is staged to elaboration — RFC-0007 §12.3 / M-673). Parallel to
    /// `tyvars`; empty for an unbounded/monomorphic body.
    bounds: &'a [(String, Vec<TraitRef>)],
    /// Live expression-nesting depth for the explicit [`MAX_CHECK_DEPTH`] budget (interior
    /// mutability so [`Self::check`] stays `&self`). Reset per body; accounted by [`DepthGuard`].
    depth: Cell<u32>,
}

impl Cx<'_> {
    fn err<T>(&self, msg: impl Into<String>) -> Result<T, CheckError> {
        Err(CheckError::new(self.site, msg))
    }

    /// Enter one level of `check` recursion against the explicit [`MAX_CHECK_DEPTH`] budget
    /// (banked guard 4): charge a level, refuse with a clean [`CheckError`] past the budget (never a
    /// host-stack overflow), and return a [`DepthGuard`] that releases the level on **any** exit path.
    fn enter(&self) -> Result<DepthGuard<'_>, CheckError> {
        let d = self.depth.get() + 1;
        if d > MAX_CHECK_DEPTH {
            return self.err(format!(
                "expression nesting exceeds the checker depth budget ({MAX_CHECK_DEPTH}) — an \
                 explicit budget (banked guard 4), refused cleanly rather than overflowing the \
                 host stack (RFC-0007 §4.6 clocked-recursion discipline)"
            ));
        }
        self.depth.set(d);
        Ok(DepthGuard(&self.depth))
    }

    fn ctor(&self, name: &str) -> Option<(&DataInfo, usize)> {
        self.types
            .values()
            .find_map(|d| d.ctors.iter().position(|c| c.name == name).map(|i| (d, i)))
    }

    /// Infer the type of `e` under `scope` (a lexical stack; shadowing = later wins). A thin wrapper
    /// over the bidirectional [`Self::check`] with no expected type — used where only the type is
    /// wanted and `e` carries no ambient bare-decimal needing context (e.g. the elaborator's
    /// re-inference over already-resolved terms).
    fn infer(&self, scope: &mut Vec<(String, Ty)>, e: &Expr) -> Result<Ty, CheckError> {
        self.check(scope, e, None).map(|(ty, _)| ty)
    }

    /// **Bidirectional check** (RFC-0012 §4.3): type `e` under `scope`, optionally *against* an
    /// `expected` type, and return the type together with a **resolved** expression — any ambient
    /// bare decimal ([`Literal::AmbientInt`]) whose width the context determines is rewritten to a
    /// concrete `Binary`/`Ternary` literal, so the evaluator and elaborator see only explicit forms.
    /// Where the width is *not* determined, the refusal is an explicit `UnresolvedWidth` (never a
    /// built-in default). A cross-paradigm edge surfaces as a `MissingConversion` (never silent).
    fn check(
        &self,
        scope: &mut Vec<(String, Ty)>,
        e: &Expr,
        expected: Option<&Ty>,
    ) -> Result<(Ty, Expr), CheckError> {
        // Charge one nesting level against the explicit depth budget; released on every exit path.
        // This is what bounds checker recursion — not the host stack (RFC-0007 §4.6; A4-02).
        let _depth = self.enter()?;
        match e {
            Expr::Lit(Literal::AmbientInt(p, v)) => {
                let lit = self.resolve_ambient_int(*p, *v, expected)?;
                let ty = lit_ty_of(self.site, &lit)?;
                Ok((ty, Expr::Lit(lit)))
            }
            Expr::Lit(l) => Ok((self.lit_ty(l)?, Expr::Lit(l.clone()))),
            Expr::Path(p) => self.check_path(scope, p, e, expected),
            // The heavy, allocation-bearing arms are separate methods so this dispatch frame stays
            // small — a deep but call-light nest (e.g. `not(not(…))`) must fit the host stack the
            // parser's depth bound allows, in debug builds too (A4-02).
            Expr::Let {
                name,
                ty,
                bound,
                body,
            } => self.check_let(scope, name, ty.as_ref(), bound, body, expected),
            Expr::If { cond, conseq, alt } => self.check_if(scope, cond, conseq, alt, expected),
            Expr::Match { scrutinee, arms } => self.check_match(scope, scrutinee, arms, expected),
            Expr::For {
                x,
                xs,
                acc,
                init,
                body,
            } => self.check_for(scope, x, xs, acc, init, body, expected),
            Expr::Swap {
                value,
                target,
                policy,
            } => self.check_swap(scope, value, target, policy),
            Expr::Wild(_) => self.err(
                "`wild` is denied by default (LR-9): no host FFI capability exists in v0, so a \
                 wild block cannot be checked or run — this refusal is the design, not a gap",
            ),
            Expr::Spore(_) => {
                self.err("`spore` is deferred to the reconstruction-manifest work (E2-5/M-260)")
            }
            Expr::Colony(hyphae) => self.check_colony(scope, hyphae, expected),
            Expr::WithParadigm { .. } => self.err(
                "internal: a `with paradigm` block reached the checker — the ambient resolution \
                 pass should have stripped it (RFC-0012 §4.4)",
            ),
            Expr::Ascribe(inner, t) => self.check_ascribe(scope, inner, t),
            Expr::App { head, args } => self.check_app(scope, head, args, expected),
        }
    }

    fn check_path(
        &self,
        scope: &[(String, Ty)],
        p: &Path,
        e: &Expr,
        expected: Option<&Ty>,
    ) -> Result<(Ty, Expr), CheckError> {
        if p.0.len() != 1 {
            return self.err(format!(
                "dotted path `{}` does not resolve in v0 (single-nodule)",
                p.0.join(".")
            ));
        }
        let name = &p.0[0];
        if let Some((_, ty)) = scope.iter().rev().find(|(n, _)| n == name) {
            return Ok((ty.clone(), e.clone()));
        }
        if let Some((d, i)) = self.ctor(name) {
            if d.ctors[i].fields.is_empty() {
                // Nullary constructor as a value. A **generic** type has no fields to infer its type
                // arguments from, so they must come from `expected` (bidirectional) — an absent or
                // mismatched context is an explicit "ascribe it" error, never a guess (§11.3).
                let targs = if d.params.is_empty() {
                    vec![]
                } else {
                    match expected {
                        Some(Ty::Data(en, eargs))
                            if en == &d.name && eargs.len() == d.params.len() =>
                        {
                            eargs.clone()
                        }
                        _ => {
                            return self.err(format!(
                                "constructor `{name}` of generic `{}<…>` needs its type \
                                 argument(s) from context — ascribe the value (RFC-0007 §11.3, \
                                 never a guess)",
                                d.name
                            ))
                        }
                    }
                };
                return Ok((Ty::Data(d.name.clone(), targs), e.clone()));
            }
            return self.err(format!(
                "constructor `{name}` takes {} field(s) — apply it (W6 saturation)",
                d.ctors[i].fields.len()
            ));
        }
        self.err(teach_unknown(name, &format!("unknown name `{name}`")))
    }

    fn check_let(
        &self,
        scope: &mut Vec<(String, Ty)>,
        name: &str,
        ty: Option<&TypeRef>,
        bound: &Expr,
        body: &Expr,
        expected: Option<&Ty>,
    ) -> Result<(Ty, Expr), CheckError> {
        let want = match ty {
            Some(t) => Some(resolve_ty(self.site, self.types, self.tyvars, t)?.0),
            None => None,
        };
        let (bty, bound2) = self.check(scope, bound, want.as_ref())?;
        if let Some(w) = &want {
            if w != &bty {
                return self.err(format!("let `{name}`: {}", edge_mismatch("bound", w, &bty)));
            }
        }
        scope.push((name.to_owned(), bty));
        let r = self.check(scope, body, expected);
        scope.pop();
        let (rty, body2) = r?;
        Ok((
            rty,
            Expr::Let {
                name: name.to_owned(),
                ty: ty.cloned(),
                bound: Box::new(bound2),
                body: Box::new(body2),
            },
        ))
    }

    fn check_if(
        &self,
        scope: &mut Vec<(String, Ty)>,
        cond: &Expr,
        conseq: &Expr,
        alt: &Expr,
        expected: Option<&Ty>,
    ) -> Result<(Ty, Expr), CheckError> {
        let bool_ty = Ty::Data("Bool".to_owned(), vec![]);
        let (c, cond2) = self.check(scope, cond, Some(&bool_ty))?;
        if c != bool_ty {
            return self.err(format!("if-condition must be Bool, got {c}"));
        }
        let (t, conseq2) = self.check(scope, conseq, expected)?;
        // The else-branch may borrow the then-branch's type as its expected (so a bare decimal in
        // one branch can take the other's width).
        let (f, alt2) = self.check(scope, alt, expected.or(Some(&t)))?;
        if t != f {
            return self.err(format!(
                "if-branches disagree: {}",
                edge_mismatch("else", &t, &f)
            ));
        }
        Ok((
            t,
            Expr::If {
                cond: Box::new(cond2),
                conseq: Box::new(conseq2),
                alt: Box::new(alt2),
            },
        ))
    }

    fn check_swap(
        &self,
        scope: &mut Vec<(String, Ty)>,
        value: &Expr,
        target: &TypeRef,
        policy: &Path,
    ) -> Result<(Ty, Expr), CheckError> {
        // The source repr is unconstrained by the target, so a bare-decimal source has no context
        // here — it must be ascribed (else an explicit UnresolvedWidth).
        let (vty, value2) = self.check(scope, value, None)?;
        if !matches!(vty, Ty::Binary(_) | Ty::Ternary(_) | Ty::Dense(_, _)) {
            return self.err(format!(
                "swap source must be a representation type, got {vty}"
            ));
        }
        let (tty, _) = resolve_ty(self.site, self.types, self.tyvars, target)?;
        if !matches!(tty, Ty::Binary(_) | Ty::Ternary(_) | Ty::Dense(_, _)) {
            return self.err(format!(
                "swap target must be a representation type, got {tty}"
            ));
        }
        Ok((
            tty,
            Expr::Swap {
                value: Box::new(value2),
                target: target.clone(),
                policy: policy.clone(),
            },
        ))
    }

    /// Type a `colony { hypha e1, …, hypha eN }` block (RFC-0008 §4.7; M-666). Every `hypha` body
    /// is type-checked under the **current** scope (RT1: hyphae share no state — each closes over
    /// the lexical environment by value, never over a mutable cell), and the colony's result type is
    /// the **last** hypha's type — the **RT2 spawn-order sequentialization**'s final observable
    /// (RFC-0008 §4.6 R1: the deterministic fragment's reference semantics is the sequentialization).
    /// The `expected` type, if any, applies to that last hypha (the colony's value); the earlier
    /// hyphae are checked with no expected. A colony must hold **≥ 1** hypha (defense-in-depth — the
    /// parser already requires it); an empty colony is an explicit refusal, never a silent unit.
    ///
    /// Honesty (Declared): this typing is the conservative v0 surface for RFC-0008 §4.7. With no
    /// product/tuple type in the v0 calculus, the colony cannot yet yield *all* hyphae's joined
    /// results as one heterogeneous value — that is later work (a join-result product; RFC-0008
    /// RT6/§4.7). The last-hypha-as-observable rule keeps the type honest and matches the RT2
    /// sequential reference; it never invents a product type or silently discards a type mismatch.
    fn check_colony(
        &self,
        scope: &mut Vec<(String, Ty)>,
        hyphae: &[Hypha],
        expected: Option<&Ty>,
    ) -> Result<(Ty, Expr), CheckError> {
        let Some((last, leading)) = hyphae.split_last() else {
            return self.err(
                "a `colony` must contain at least one `hypha` (RFC-0008 §4.7 — a colony is a \
                 grouping of *active* hyphae); an empty `colony { }` has no observable",
            );
        };
        let mut checked: Vec<Hypha> = Vec::with_capacity(hyphae.len());
        // Leading hyphae: each is its own computation with no expected type. RT1 — each is checked
        // under the same lexical scope (closed over by value), never mutating it.
        for h in leading {
            let (_ty, body2) = self.check(scope, &h.body, None)?;
            checked.push(Hypha { body: body2 });
        }
        // The final hypha carries the colony's observable (the RT2 sequentialization's last step), so
        // the `expected` type applies to it.
        let (rty, last_body2) = self.check(scope, &last.body, expected)?;
        checked.push(Hypha { body: last_body2 });
        Ok((rty, Expr::Colony(checked)))
    }

    fn check_ascribe(
        &self,
        scope: &mut Vec<(String, Ty)>,
        inner: &Expr,
        t: &TypeRef,
    ) -> Result<(Ty, Expr), CheckError> {
        let (want, _) = resolve_ty(self.site, self.types, self.tyvars, t)?;
        let (ity, inner2) = self.check(scope, inner, Some(&want))?;
        if ity != want {
            return self.err(format!(
                "ascription: {}",
                edge_mismatch("expression", &want, &ity)
            ));
        }
        Ok((want, Expr::Ascribe(Box::new(inner2), t.clone())))
    }

    fn check_app(
        &self,
        scope: &mut Vec<(String, Ty)>,
        head: &Expr,
        args: &[Expr],
        expected: Option<&Ty>,
    ) -> Result<(Ty, Expr), CheckError> {
        let Expr::Path(p) = head else {
            return self.err("v0 application head must be a name (first-order; RFC-0007 §4.4)");
        };
        if p.0.len() != 1 {
            return self.err(format!(
                "dotted call `{}` does not resolve in v0",
                p.0.join(".")
            ));
        }
        let name = &p.0[0];

        // User function: each argument is checked **against** its declared parameter type, so a
        // bare-decimal argument takes the parameter's width.
        if let Some(fd) = self.fns.get(name) {
            if fd.sig.value_params.len() != args.len() {
                return self.err(format!(
                    "`{name}` takes {} argument(s), got {}",
                    fd.sig.value_params.len(),
                    args.len()
                ));
            }
            // Monomorphic callee — unchanged v0 path (exact bidirectional checking + error messages).
            if fd.sig.params.is_empty() {
                let mut rebuilt = Vec::with_capacity(args.len());
                for (pm, a) in fd.sig.value_params.iter().zip(args) {
                    let (want, _) = resolve_ty(self.site, self.types, &[], &pm.ty)?;
                    let (got, a2) = self.check(scope, a, Some(&want))?;
                    if want != got {
                        return self.err(format!(
                            "`{name}` parameter `{}`: {}",
                            pm.name,
                            edge_mismatch("argument", &want, &got)
                        ));
                    }
                    rebuilt.push(a2);
                }
                let (ret, _) = resolve_ty(self.site, self.types, &[], &fd.sig.ret)?;
                return Ok((ret, app_node(head, rebuilt)));
            }
            // Generic callee — extracted to a separate (non-inlined) method so `check_app`'s frame
            // stays small on the common monomorphic/prim path (the A4-02 host-stack-depth bound).
            return self.check_app_generic_fn(scope, head, name, fd, args);
        }

        // Constructor (W6 saturation).
        if let Some((d, i)) = self.ctor(name) {
            let arity = d.ctors[i].fields.len();
            let dname = d.name.clone();
            let params = d.params.clone();
            if arity != args.len() {
                return self.err(format!(
                    "constructor `{name}` takes {arity} field(s), got {} (W6 saturation)",
                    args.len()
                ));
            }
            // Monomorphic data type — the original inline path (small frame, exact v0 errors).
            if params.is_empty() {
                let fields = d.ctors[i].fields.clone();
                let mut rebuilt = Vec::with_capacity(args.len());
                for (want, a) in fields.iter().zip(args) {
                    let (got, a2) = self.check(scope, a, Some(want))?;
                    if want != &got {
                        return self.err(format!(
                            "constructor `{name}` field: {}",
                            edge_mismatch("argument", want, &got)
                        ));
                    }
                    rebuilt.push(a2);
                }
                return Ok((Ty::Data(dname, vec![]), app_node(head, rebuilt)));
            }
            // Generic data type — extracted (frame-size; A4-02).
            let fields = d.ctors[i].fields.clone();
            return self
                .check_app_generic_ctor(scope, head, name, dname, params, fields, args, expected);
        }

        // Unqualified **trait-method** call (RFC-0019 §4.4; RFC-0007 §12.1): if `name` is not a
        // fn/ctor (checked above) but is a method of exactly one registered trait, resolve it
        // through a bound in scope or a concrete instance — extracted (frame-size; A4-02).
        if self.is_trait_method(name) {
            return self.check_trait_method_call(scope, head, name, args, expected);
        }

        // Builtin prim: width-polymorphic and width-preserving, so the result's expected width (or
        // a concrete operand's width) anchors any bare-decimal operand (RFC-0012 §4.3). Inlined
        // (not a separate method) to keep the per-nesting-level host-stack frame count at the
        // pre-M-344 depth — the parser bounds AST nesting, and the checker must fit that bound
        // without overflowing (A4-02).
        let Some(fam) = prim_family(name) else {
            return self.err(teach_unknown(
                name,
                &format!("unknown function/constructor/prim `{name}`"),
            ));
        };
        // First, type the operands that are *not* bare decimals; they anchor the width.
        let mut typed: Vec<Option<(Ty, Expr)>> = vec![None; args.len()];
        let mut anchor: Option<u32> = expected.and_then(|t| fam.width_of(t));
        for (i, a) in args.iter().enumerate() {
            if matches!(a, Expr::Lit(Literal::AmbientInt(_, _))) {
                continue;
            }
            let (t, a2) = self.check(scope, a, None)?;
            if anchor.is_none() {
                anchor = fam.width_of(&t);
            }
            typed[i] = Some((t, a2));
        }
        // Then resolve each bare-decimal operand against the anchor.
        let mut arg_tys = Vec::with_capacity(args.len());
        let mut rebuilt = Vec::with_capacity(args.len());
        for (i, a) in args.iter().enumerate() {
            let (t, a2) = match typed[i].take() {
                Some(x) => x,
                None => {
                    let w = anchor.ok_or_else(|| {
                        CheckError::new(
                            self.site,
                            format!(
                                "UnresolvedWidth: a bare decimal operand of `{name}` has no width \
                                 here — no concrete operand or expected type pins it. Ascribe it, \
                                 or write it explicitly (RFC-0012 §4.3, never a default width)"
                            ),
                        )
                    })?;
                    self.check(scope, a, Some(&fam.ty(w)))?
                }
            };
            arg_tys.push(t);
            rebuilt.push(a2);
        }
        match prim_sig(name, &arg_tys) {
            Some(ret) => Ok((ret, app_node(head, rebuilt))),
            None => self.err(format!(
                "`{name}` does not accept argument types {arg_tys:?} (T-Op; RFC-0007 §4.4)"
            )),
        }
    }

    /// Check a call to a **generic** function (RFC-0007 §11.3): resolve the callee's signature with
    /// the *callee's* type parameters as abstract variables, **unify** the declared parameter types
    /// against the actual argument types to infer the type arguments, and substitute the solution
    /// into the return type. An unsolved type parameter is an explicit error — never a guessed
    /// default (G2/VR-5). Extracted (`#[inline(never)]`) so [`Self::check_app`]'s frame stays within
    /// the A4-02 host-stack-depth bound on the common monomorphic/prim path.
    #[inline(never)]
    fn check_app_generic_fn(
        &self,
        scope: &mut Vec<(String, Ty)>,
        head: &Expr,
        name: &str,
        fd: &FnDecl,
        args: &[Expr],
    ) -> Result<(Ty, Expr), CheckError> {
        let callee_vars = fd.sig.param_names();
        let mut subst: BTreeMap<String, Ty> = BTreeMap::new();
        let mut rebuilt = Vec::with_capacity(args.len());
        for (pm, a) in fd.sig.value_params.iter().zip(args) {
            let want = resolve_ty(self.site, self.types, &callee_vars, &pm.ty)?.0;
            let want_now = subst_ty(&want, &subst);
            // A fully-concrete (post-substitution) expected type drives the argument's check (so a
            // bare decimal takes the width); a still-abstract one lets the argument synthesize.
            let exp = if has_var(&want_now) {
                None
            } else {
                Some(&want_now)
            };
            let (got, a2) = self.check(scope, a, exp)?;
            unify(self.site, &want_now, &got, &mut subst).map_err(|e| {
                CheckError::new(
                    self.site,
                    format!("`{name}` argument `{}`: {}", pm.name, e.message),
                )
            })?;
            rebuilt.push(a2);
        }
        for v in &callee_vars {
            if !subst.contains_key(v) {
                return self.err(format!(
                    "`{name}` is generic over `{v}`, but this call does not determine it — \
                     ascribe an argument or the result (RFC-0007 §11.3, never a guessed default)"
                ));
            }
        }
        // Bounded-generic-call satisfiability (RFC-0019 §4.1/§4.5; RFC-0007 §12.1). Now `T = C` is
        // solved, for each bound `T: Tr` on a type-parameter require an instance `(Tr, head(C))` to
        // exist — else an explicit "no instance" refusal. The dictionary VALUE is NOT constructed
        // here (staged to elaboration — RFC-0007 §12.3 / M-673); the checker only validates the bound
        // is *satisfiable* ("dictionary typing"), never a silent skip (G2).
        for p in &fd.sig.params {
            for b in &p.bounds {
                let concrete = subst
                    .get(&p.name)
                    .cloned()
                    .unwrap_or(Ty::Var(p.name.clone()));
                self.require_instance(
                    &b.name,
                    &concrete,
                    &format!("required by `{name}`'s bound `{}: {}`", p.name, b.name),
                )?;
            }
        }
        let ret = subst_ty(
            &resolve_ty(self.site, self.types, &callee_vars, &fd.sig.ret)?.0,
            &subst,
        );
        Ok((ret, app_node(head, rebuilt)))
    }

    /// Require that an instance `(trait_name, type_head(concrete))` exists, or refuse explicitly
    /// (RFC-0019 §4.5; G2). Used for bounded-generic-call satisfiability and concrete trait-method
    /// resolution. If `concrete` is still abstract (a bare `Ty::Var` — the call is itself inside a
    /// bounded generic whose bound already guarantees the instance at the eventual concrete type),
    /// the requirement is **discharged by the bound in scope**: it is satisfied iff that same
    /// `(var: trait)` bound is present in `self.bounds` (else an explicit "no instance/bound" error).
    /// The dictionary value is staged to elaboration (M-673) — this is satisfiability only.
    fn require_instance(
        &self,
        trait_name: &str,
        concrete: &Ty,
        because: &str,
    ) -> Result<(), CheckError> {
        match type_head(concrete) {
            Some(head) => match self.instances.get(&(trait_name.to_owned(), head)) {
                // Head-erasure is the COHERENCE key (≤1 instance per head); RESOLUTION must still
                // match the FULL concrete type — a `Binary{8}` instance does not satisfy a `Binary{4}`
                // call. Head-erasure over-REJECTS duplicates; it must never over-ACCEPT a different
                // type (G2: never silently reuse a mismatched instance).
                Some(info) if info.for_ty == *concrete => Ok(()),
                Some(info) => self.err(format!(
                    "no instance `{trait_name}` for `{concrete}` ({because}) — the `{trait_name}` \
                     instance on this type head is declared for `{}`, not `{concrete}` (RFC-0019 §4.5)",
                    info.for_ty
                )),
                None => self.err(format!(
                    "no instance `{trait_name}` for `{concrete}` ({because}) — declare \
                     `impl {trait_name}<…> for {concrete} {{ … }}` (RFC-0019 §4.5, never assumed)"
                )),
            },
            // `concrete` is a type-variable in scope: discharge via a matching bound (the dictionary
            // is threaded by the eventual caller — RFC-0007 §12.3 / M-673).
            None => {
                let Ty::Var(v) = concrete else {
                    return self.err(format!(
                        "no instance `{trait_name}` for `{concrete}` ({because})"
                    ));
                };
                let satisfied = self
                    .bounds
                    .iter()
                    .any(|(pv, bs)| pv == v && bs.iter().any(|b| b.name == trait_name));
                if satisfied {
                    Ok(())
                } else {
                    self.err(format!(
                        "no instance/bound provides `{trait_name}` for type variable `{v}` \
                         ({because}) — add the bound `{v}: {trait_name}` (RFC-0019 §4.1)"
                    ))
                }
            }
        }
    }

    /// Is `name` a method of some registered trait? (The call-resolution path uses this to decide
    /// whether to try trait-method resolution — fn/ctor names were already dispatched, so a name
    /// reaching here that matches a trait method is an unqualified trait-method call.)
    fn is_trait_method(&self, name: &str) -> bool {
        self.traits
            .values()
            .any(|tr| tr.sigs.iter().any(|s| s.name == name))
    }

    /// Resolve and type an **unqualified trait-method call** `m(args)` (RFC-0019 §4.4; RFC-0007
    /// §12.1; guarantee: `Declared`). The method must belong to **exactly one** trait (ambiguity
    /// across traits is an explicit error — never a guess). The trait's single type-parameter is
    /// determined by **unifying** the trait method's signature against the actual argument types;
    /// then an instance must exist — either a concrete `(Tr, head(C))` or a `T: Tr` **bound in
    /// scope** (dictionary staged to elaboration — M-673). The call types at the (substituted)
    /// method return type. Extracted (`#[inline(never)]`) for the frame-size reason as the other
    /// generic paths (A4-02).
    #[inline(never)]
    fn check_trait_method_call(
        &self,
        scope: &mut Vec<(String, Ty)>,
        head: &Expr,
        name: &str,
        args: &[Expr],
        expected: Option<&Ty>,
    ) -> Result<(Ty, Expr), CheckError> {
        // 1. The trait(s) that declare a method named `name`. Exactly one ⇒ resolve; >1 ⇒ ambiguous.
        let owners: Vec<&TraitInfo> = self
            .traits
            .values()
            .filter(|tr| tr.sigs.iter().any(|s| s.name == name))
            .collect();
        let tr = match owners.as_slice() {
            [one] => *one,
            [] => unreachable!("is_trait_method gated this call"),
            many => {
                let names: Vec<&str> = many.iter().map(|t| t.name.as_str()).collect();
                return self.err(format!(
                    "ambiguous trait-method call `{name}` — declared by multiple traits ({}); \
                     stage-1 has no qualified-call syntax, so this is an explicit refusal, never a \
                     guess (RFC-0019 §4.4)",
                    names.join(", ")
                ));
            }
        };
        if tr.params.len() != 1 {
            return self.err(format!(
                "trait-method resolution for `{name}` needs a single-parameter trait in stage-1 \
                 (trait `{}` has {} parameters — multi-parameter traits are v2, RFC-0019 §10)",
                tr.name,
                tr.params.len()
            ));
        }
        let sig = tr
            .sigs
            .iter()
            .find(|s| s.name == name)
            .expect("owner has the method");
        if sig.value_params.len() != args.len() {
            return self.err(format!(
                "trait method `{}::{name}` takes {} argument(s), got {}",
                tr.name,
                sig.value_params.len(),
                args.len()
            ));
        }
        // 2. Unify the method's (abstract-over-the-trait-param) value-param types against the actual
        //    argument types to solve the trait parameter — never a guess (RFC-0007 §11.3).
        let tparam = &tr.params[0];
        let trait_vars = std::slice::from_ref(tparam);
        let mut subst: BTreeMap<String, Ty> = BTreeMap::new();
        // Seed from `expected` against the (abstract) return type, so a nullary-ish return can pin
        // the parameter even when an argument is a bare decimal.
        if let Some(exp) = expected {
            let ret_abs = resolve_ty(self.site, self.types, trait_vars, &sig.ret)?.0;
            let _ = unify(self.site, &ret_abs, exp, &mut subst);
        }
        let mut rebuilt = Vec::with_capacity(args.len());
        for (pm, a) in sig.value_params.iter().zip(args) {
            let want = resolve_ty(self.site, self.types, trait_vars, &pm.ty)?.0;
            let want_now = subst_ty(&want, &subst);
            let exp = if has_var(&want_now) {
                None
            } else {
                Some(&want_now)
            };
            let (got, a2) = self.check(scope, a, exp)?;
            unify(self.site, &want_now, &got, &mut subst).map_err(|e| {
                CheckError::new(
                    self.site,
                    format!(
                        "trait method `{}::{name}` argument `{}`: {}",
                        tr.name, pm.name, e.message
                    ),
                )
            })?;
            rebuilt.push(a2);
        }
        // 3. The trait parameter must be determined; then an instance (concrete or via a scope bound)
        //    must provide the trait — else an explicit "no instance/bound" refusal (never a guess).
        let Some(receiver) = subst.get(tparam).cloned() else {
            return self.err(format!(
                "trait-method call `{name}` does not determine trait `{}`'s type parameter `{tparam}` \
                 from its arguments — ascribe an argument or the result (RFC-0019 §4.4, never a guess)",
                tr.name
            ));
        };
        self.require_instance(
            &tr.name,
            &receiver,
            &format!(
                "no instance/bound provides `{}::{name}` for these arguments",
                tr.name
            ),
        )?;
        let ret = subst_ty(
            &resolve_ty(self.site, self.types, trait_vars, &sig.ret)?.0,
            &subst,
        );
        Ok((ret, app_node(head, rebuilt)))
    }

    /// Check a saturated application of a **generic** data constructor (RFC-0007 §11.2/§11.3). The
    /// constructor's declared fields are abstract over the type's parameters; the type arguments are
    /// taken from `expected` when it pins this data type (bidirectional), otherwise **inferred** from
    /// the field arguments via [`unify`]. An undetermined parameter is an explicit "ascribe it" error
    /// — never a guess. Extracted (`#[inline(never)]`) for the same frame-size reason as
    /// [`Self::check_app_generic_fn`].
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn check_app_generic_ctor(
        &self,
        scope: &mut Vec<(String, Ty)>,
        head: &Expr,
        name: &str,
        dname: String,
        params: Vec<String>,
        fields: Vec<Ty>,
        args: &[Expr],
        expected: Option<&Ty>,
    ) -> Result<(Ty, Expr), CheckError> {
        let mut subst: BTreeMap<String, Ty> = BTreeMap::new();
        if let Some(Ty::Data(en, eargs)) = expected {
            if *en == dname && eargs.len() == params.len() {
                for (p, ea) in params.iter().zip(eargs) {
                    subst.insert(p.clone(), ea.clone());
                }
            }
        }
        let mut rebuilt = Vec::with_capacity(args.len());
        for (want, a) in fields.iter().zip(args) {
            let want_now = subst_ty(want, &subst);
            let exp = if has_var(&want_now) {
                None
            } else {
                Some(&want_now)
            };
            let (got, a2) = self.check(scope, a, exp)?;
            unify(self.site, &want_now, &got, &mut subst).map_err(|e| {
                CheckError::new(
                    self.site,
                    format!("constructor `{name}` field: {}", e.message),
                )
            })?;
            rebuilt.push(a2);
        }
        let mut targs = Vec::with_capacity(params.len());
        for p in &params {
            match subst.get(p) {
                Some(t) => targs.push(t.clone()),
                None => {
                    return self.err(format!(
                        "constructor `{name}` does not determine type parameter `{p}` of \
                         `{dname}<…>` — ascribe the value (RFC-0007 §11.3, never a guess)"
                    ))
                }
            }
        }
        Ok((Ty::Data(dname, targs), app_node(head, rebuilt)))
    }

    /// T-For (RFC-0007 §4.8): `xs` must be a *linearly recursive* data type (nil/cons shape);
    /// `init : A`; `body : A` under `x : E, acc : A`; the whole expression is `A`. Every shape
    /// violation is an explicit refusal — general catamorphisms are an L2 concern.
    #[allow(clippy::too_many_arguments)]
    fn check_for(
        &self,
        scope: &mut Vec<(String, Ty)>,
        x: &str,
        xs: &Expr,
        acc: &str,
        init: &Expr,
        body: &Expr,
        expected: Option<&Ty>,
    ) -> Result<(Ty, Expr), CheckError> {
        let (sty, xs2) = self.check(scope, xs, None)?;
        let Ty::Data(tname, targs) = &sty else {
            return self.err(format!(
                "`for` iterates a linearly recursive data value, got {sty} (RFC-0007 §4.8)"
            ));
        };
        // For a generic linear type (`List<Binary{8}>`) the element type is the declared element
        // (`Var("A")`) with the scrutinee's type arguments substituted in (RFC-0007 §11.2).
        let elem = linear_elem_ty(self.site, self.types, tname, targs)?;
        // The accumulator type is the whole expression's type, so the `for`'s expected anchors `init`.
        let (aty, init2) = self.check(scope, init, expected)?;
        scope.push((x.to_owned(), elem));
        scope.push((acc.to_owned(), aty.clone()));
        let r = self.check(scope, body, Some(&aty));
        scope.pop();
        scope.pop();
        let (bty, body2) = r?;
        if bty != aty {
            return self.err(format!(
                "`for` body must yield the accumulator type {}",
                edge_mismatch("body", &aty, &bty)
            ));
        }
        Ok((
            aty,
            Expr::For {
                x: x.to_owned(),
                xs: Box::new(xs2),
                acc: acc.to_owned(),
                init: Box::new(init2),
                body: Box::new(body2),
            },
        ))
    }

    /// Type a `match` over a data, `Binary`, or `Ternary` scrutinee with **nested** patterns
    /// (RFC-0007 §4.4/§4.7). Each arm's pattern is checked against the scrutinee type (binders enter
    /// scope at their field types), the arm bodies must agree, and coverage is decided by the
    /// **Maranget usefulness** algorithm ([`crate::usefulness`]): the match must be **exhaustive**
    /// (a `_` is not useful — otherwise the witness names a missing case) and every arm must be
    /// **reachable** (an arm covered by the earlier ones is a redundancy error). This unifies the data
    /// match and the M-320 literal match: a `Binary`/`Ternary` value domain is never enumerated, so a
    /// literal match still needs a `_`/binder default (W7 — coverage is checked, never assumed).
    /// Bare-decimal literal patterns take their width from the scrutinee/field type (RFC-0012 §4.3).
    fn check_match(
        &self,
        scope: &mut Vec<(String, Ty)>,
        scrutinee: &Expr,
        arms: &[crate::ast::Arm],
        expected: Option<&Ty>,
    ) -> Result<(Ty, Expr), CheckError> {
        let (sty, scrut2) = self.check(scope, scrutinee, None)?;
        if !matches!(sty, Ty::Data(_, _) | Ty::Binary(_) | Ty::Ternary(_)) {
            return self.err(format!(
                "match scrutinee must be a data, Binary, or Ternary type, got {sty}"
            ));
        }
        if arms.is_empty() {
            return self.err("a match needs at least one arm");
        }
        let col = [sty.clone()];
        let mut rows: Vec<Vec<crate::usefulness::Pat>> = Vec::new();
        let mut result: Option<Ty> = None;
        let mut arms2: Vec<crate::ast::Arm> = Vec::with_capacity(arms.len());
        for arm in arms {
            // Resolve any ambient bare-decimal literal patterns against the scrutinee/field types
            // first, so the matrix, the evaluator, and the elaborator all see concrete literals.
            let pattern = self.resolve_pattern_lits(&arm.pattern, &sty)?;
            // Type the (possibly nested) pattern against the scrutinee type, collecting its binders.
            let mut binds: Vec<(String, Ty, Vec<usize>)> = Vec::new();
            let pat = self.check_pattern(&pattern, &sty, &mut binds)?;
            self.check_linear(&binds)?;
            // Redundancy (W7): an arm covered by the earlier rows is unreachable.
            if crate::usefulness::useful(self.types, &rows, std::slice::from_ref(&pat), &col)
                .is_none()
            {
                return self.err(
                    "this arm is unreachable — earlier arms already cover it (W7: every arm must \
                     be reachable)",
                );
            }
            // Type the body with the pattern's binders in scope.
            let depth = scope.len();
            for (name, ty, _occ) in &binds {
                scope.push((name.clone(), ty.clone()));
            }
            let body_expected = expected.or(result.as_ref());
            let r = self.check(scope, &arm.body, body_expected);
            scope.truncate(depth);
            let (bty, body2) = r?;
            match &result {
                None => result = Some(bty),
                Some(r) if *r != bty => {
                    return self.err(format!(
                        "match arms disagree: {}",
                        edge_mismatch("arm", r, &bty)
                    ))
                }
                Some(_) => {}
            }
            rows.push(vec![pat]);
            arms2.push(crate::ast::Arm {
                pattern,
                body: body2,
            });
        }
        // Exhaustiveness (W7): a wildcard must not be useful — else its witness is a missing case.
        if let Some(witness) =
            crate::usefulness::useful(self.types, &rows, &[crate::usefulness::Pat::Wild], &col)
        {
            return self.err(format!(
                "non-exhaustive match on {sty}: missing {} (W7 — coverage is checked, never assumed)",
                crate::usefulness::render(&witness[0])
            ));
        }
        // Codegen half (M-320; RFC-0007 §3, "compiled away by the elaborator"): compile the checked
        // match to a Maranget decision tree and confirm it is **Fail-free** — an exhaustive match must
        // compile to total coverage, so the usefulness analysis and the tree compiler must agree
        // (defense in depth; never silent if they don't). The tree is the flat-tests form; emitting
        // its leaves as L0 kernel nodes awaits the RFC-0001 revision (RFC-0007 §4.6).
        let arm_ix: Vec<usize> = (0..rows.len()).collect();
        let occ = [Vec::<usize>::new()];
        let tree = crate::decision::compile(self.types, &rows, &arm_ix, &occ, &col);
        if crate::decision::has_reachable_fail(&tree) {
            return self.err(
                "internal: an exhaustive match compiled to a decision tree with a reachable Fail \
                 (the usefulness check and the Maranget compiler disagree — report this)",
            );
        }
        let rty =
            result.ok_or_else(|| CheckError::new(self.site, "a match needs at least one arm"))?;
        Ok((
            rty,
            Expr::Match {
                scrutinee: Box::new(scrut2),
                arms: arms2,
            },
        ))
    }

    /// Resolve any ambient bare-decimal (`AmbientInt`) literal **patterns** in `pat` to concrete
    /// literals, taking each one's width from `expected` — the scrutinee type at the root, and each
    /// constructor field's type as it recurses. A literal pattern under a non-repr/cross-paradigm
    /// position is left unchanged so [`normalize_pattern`] raises the precise W7 error.
    fn resolve_pattern_lits(&self, pat: &Pattern, expected: &Ty) -> Result<Pattern, CheckError> {
        Ok(match pat {
            Pattern::Lit(Literal::AmbientInt(p, v)) => {
                Pattern::Lit(self.resolve_ambient_int(*p, *v, Some(expected))?)
            }
            Pattern::Ctor(name, subs) => {
                // Recurse with each sub-pattern's field type, when the expected type is the owning
                // data type and the constructor/arity line up; otherwise leave `subs` for the
                // normalizer to diagnose.
                let field_tys = match expected {
                    // The declared field types are abstract over the type's parameters; substitute
                    // the scrutinee's type arguments so a generic field recurses at its concrete
                    // type (RFC-0007 §11.2).
                    Ty::Data(tn, targs) => self.types.get(tn).and_then(|d| {
                        d.ctors
                            .iter()
                            .find(|c| c.name == *name)
                            .filter(|c| c.fields.len() == subs.len())
                            .map(|c| {
                                let s = param_subst(&d.params, targs);
                                c.fields.iter().map(|f| subst_ty(f, &s)).collect::<Vec<_>>()
                            })
                    }),
                    _ => None,
                };
                let mut out = Vec::with_capacity(subs.len());
                for (i, s) in subs.iter().enumerate() {
                    match &field_tys {
                        Some(fts) => out.push(self.resolve_pattern_lits(s, &fts[i])?),
                        None => out.push(s.clone()),
                    }
                }
                Pattern::Ctor(name.clone(), out)
            }
            Pattern::Wildcard | Pattern::Lit(_) | Pattern::Ident(_) => pat.clone(),
        })
    }

    /// Resolve a bare decimal (`AmbientInt`) to a concrete `Binary`/`Ternary` literal at the width
    /// the `expected` type pins (RFC-0012 §4.3). Never a built-in default: an absent or
    /// cross-paradigm context is an explicit refusal.
    fn resolve_ambient_int(
        &self,
        p: Paradigm,
        v: i64,
        expected: Option<&Ty>,
    ) -> Result<Literal, CheckError> {
        match (p, expected) {
            (Paradigm::Binary, Some(Ty::Binary(w))) => encode_binary(self.site, v, *w),
            (Paradigm::Ternary, Some(Ty::Ternary(w))) => encode_balanced_ternary(self.site, v, *w),
            (_, Some(other)) => self.err(format!(
                "a bare `{p}` decimal cannot fill a {other} context — {} (RFC-0012 §4.3)",
                match paradigm_name(other) {
                    Some(o) => format!("`{p}` and `{o}` are different paradigms; write an explicit \
                                        `swap` or a tagged literal"),
                    None => "a bare decimal only resolves in a Binary/Ternary context".to_owned(),
                }
            )),
            (_, None) => self.err(format!(
                "UnresolvedWidth: a bare `{p}` decimal `{v}` has no width here — its width must come \
                 from the checked context (an ascription, a parameter/return type, or another \
                 operand). Ascribe it or write it explicitly (RFC-0012 §4.3; never a default width)"
            )),
        }
    }

    /// Type-check `pat` against `expected`, accumulating its binders (`name: ty @ occurrence`) into
    /// `binds`, and return the normalized [`crate::usefulness::Pat`] for the coverage matrix.
    /// Delegates to the free [`normalize_pattern`] (shared with the elaborator), starting at the root
    /// occurrence `[]`.
    fn check_pattern(
        &self,
        pat: &Pattern,
        expected: &Ty,
        binds: &mut Vec<(String, Ty, Vec<usize>)>,
    ) -> Result<crate::usefulness::Pat, CheckError> {
        normalize_pattern(self.types, self.site, pat, expected, &[], binds)
    }

    /// A pattern must bind each name at most once (linearity) — a repeated binder is ambiguous, so it
    /// is an explicit error rather than a silent last-wins.
    fn check_linear(&self, binds: &[(String, Ty, Vec<usize>)]) -> Result<(), CheckError> {
        for (i, (n, _, _)) in binds.iter().enumerate() {
            if binds[..i].iter().any(|(m, _, _)| m == n) {
                return self.err(format!(
                    "pattern binds `{n}` more than once (bindings must be linear)"
                ));
            }
        }
        Ok(())
    }

    /// Literal typing (Q6): a literal *is* its representation — a binary literal's width is its
    /// digit count, a ternary literal's trit count its width. Bare integers and lists need
    /// context v0 does not yet give them → explicit refusal, never a cross-family default.
    fn lit_ty(&self, l: &Literal) -> Result<Ty, CheckError> {
        lit_ty_of(self.site, l)
    }
}

/// The literal-typing rule (Q6), as a free function so the elaborator can reuse it without a
/// checking context. A literal *is* its representation (width = digit count); bare integers and
/// lists are explicit refusals.
pub(crate) fn lit_ty_of(site: &str, l: &Literal) -> Result<Ty, CheckError> {
    match l {
        Literal::Bin(s) => {
            let n = s.chars().filter(|c| *c == '0' || *c == '1').count();
            if n == 0 {
                return Err(CheckError::new(site, "empty binary literal"));
            }
            Ok(Ty::Binary(u32::try_from(n).expect("digit count fits u32")))
        }
        Literal::Trit(s) => {
            if s.is_empty() {
                return Err(CheckError::new(site, "empty ternary literal"));
            }
            Ok(Ty::Ternary(
                u32::try_from(s.len()).expect("trit count fits u32"),
            ))
        }
        Literal::Int(_) => Err(CheckError::new(
            site,
            "a bare integer literal has no representation family (no cross-family defaulting, \
             Q6) — write a binary/ternary literal, declare a `default paradigm` (RFC-0012), or \
             ascribe a Dense element",
        )),
        Literal::AmbientInt(_, _) => Err(CheckError::new(
            site,
            "internal: an unresolved ambient bare decimal reached `lit_ty_of` — the checker \
             resolves its width from context first (RFC-0012 §4.3)",
        )),
        Literal::List(_) => Err(CheckError::new(
            site,
            "list literals are deferred in v0 (Dense construction)",
        )),
    }
}

/// Normalize a surface [`Pattern`] against its `expected` type into a [`crate::usefulness::Pat`]
/// (the coverage-matrix shape), collecting its binders as `(name, type, occurrence)` — the
/// **occurrence** is the path of field indices from the scrutinee root to the binder's position.
/// Shared by the checker (`Cx::check_pattern`, occurrence `[]`) and the **elaborator** (which needs
/// the matrix + the binder occurrences to lower a `match` to flat L0 `Match`/binders — RFC-0011
/// §4.4). Nested constructor/literal patterns recurse (RFC-0007 §4.7); a binder is a wildcard for
/// coverage (it refines nothing), a nullary constructor an empty `Ctor`.
pub(crate) fn normalize_pattern(
    types: &BTreeMap<String, DataInfo>,
    site: &str,
    pat: &Pattern,
    expected: &Ty,
    occ: &[usize],
    binds: &mut Vec<(String, Ty, Vec<usize>)>,
) -> Result<crate::usefulness::Pat, CheckError> {
    use crate::usefulness::Pat;
    match pat {
        Pattern::Wildcard => Ok(Pat::Wild),
        Pattern::Ident(n) => {
            // A bare name is a nullary-constructor alternative iff it names one of the expected
            // data type's constructors; otherwise it binds the whole position (at this occurrence).
            if let Ty::Data(tn, _) = expected {
                let d = types.get(tn).expect("registered data type");
                if let Some(c) = d.ctors.iter().find(|c| c.name == *n) {
                    if !c.fields.is_empty() {
                        return Err(CheckError::new(
                            site,
                            format!(
                                "constructor pattern `{n}` must bind its {} field(s) (W7)",
                                c.fields.len()
                            ),
                        ));
                    }
                    return Ok(Pat::Ctor(n.clone(), vec![]));
                }
            }
            binds.push((n.clone(), expected.clone(), occ.to_vec()));
            Ok(Pat::Wild)
        }
        Pattern::Ctor(n, subs) => {
            let Ty::Data(tn, targs) = expected else {
                return Err(CheckError::new(
                    site,
                    format!(
                        "constructor pattern `{n}` on a {expected} scrutinee — match a literal or `_`"
                    ),
                ));
            };
            let d = types.get(tn).expect("registered data type").clone();
            let Some(c) = d.ctors.iter().find(|c| c.name == *n) else {
                return Err(CheckError::new(
                    site,
                    format!("`{n}` is not a constructor of {tn}"),
                ));
            };
            if subs.len() != c.fields.len() {
                return Err(CheckError::new(
                    site,
                    format!(
                        "pattern `{n}` binds {} of {} field(s) (W7: exactly the arity)",
                        subs.len(),
                        c.fields.len()
                    ),
                ));
            }
            // The constructor's field types are abstract over the type's parameters; instantiate them
            // at the scrutinee's type arguments so each binder gets its concrete type (RFC-0007 §11.2).
            let s = param_subst(&d.params, targs);
            let mut out = Vec::with_capacity(subs.len());
            for (i, (sub, fty)) in subs.iter().zip(&c.fields).enumerate() {
                let mut child = occ.to_vec();
                child.push(i);
                let fty = subst_ty(fty, &s);
                out.push(normalize_pattern(types, site, sub, &fty, &child, binds)?);
            }
            Ok(Pat::Ctor(n.clone(), out))
        }
        Pattern::Lit(lit) => {
            let lty = lit_ty_of(site, lit)?;
            if lty != *expected {
                return Err(CheckError::new(
                    site,
                    format!(
                        "literal pattern has type {lty} but the scrutinee is {expected} \
                         (W7: a literal arm must match the scrutinee's repr and width)"
                    ),
                ));
            }
            Ok(Pat::Lit(literal_key(lit)))
        }
    }
}

/// Re-infer an expression's type against a checked [`Env`] (the elaborator needs the scrutinee type
/// to lower a `match`, and a `let`-bound's type to track its scope — RFC-0011). The program is
/// already checked, so this recomputes a type the checker validated; it does not re-litigate errors.
pub(crate) fn infer_type(
    env: &Env,
    scope: &mut Vec<(String, Ty)>,
    e: &Expr,
) -> Result<Ty, CheckError> {
    let cx = Cx {
        site: "<elaborate>",
        types: &env.types,
        fns: &env.fns,
        // The trait/instance registries are available for re-inference (a monomorphic body may
        // call a trait method resolved through a concrete instance — RFC-0019 §4.5).
        traits: &env.traits,
        instances: &env.instances,
        // Re-inference runs over already-checked, monomorphic terms (a generic *instantiation* is
        // refused at elaboration before re-inference — RFC-0007 §11.3 staging), so no type
        // parameters / bounds are in scope here.
        tyvars: &[],
        bounds: &[],
        depth: Cell::new(0),
    };
    cx.infer(scope, e)
}

/// A canonical key for de-duplicating literal patterns (M-320): normalize away `_` separators so
/// `0b1010` and `0b10_10` collide as the *same* literal. Only `Bin`/`Trit` reach here (the caller
/// type-checks the literal first, which rejects `Int`/`List`).
fn literal_key(lit: &Literal) -> String {
    match lit {
        Literal::Bin(s) => format!(
            "b:{}",
            s.chars()
                .filter(|c| *c == '0' || *c == '1')
                .collect::<String>()
        ),
        Literal::Trit(s) => format!("t:{s}"),
        Literal::Int(i) => format!("i:{i}"),
        Literal::AmbientInt(p, i) => format!("amb:{p}:{i}"),
        Literal::List(_) => "list".to_owned(),
    }
}

/// The teaching diagnostic for imperative control-flow words used as names (RFC-0007 §4.8):
/// the error was happening anyway (unknown name) — make it teach instead of confuse.
fn teach_unknown(name: &str, base: &str) -> String {
    if matches!(name, "while" | "loop" | "break" | "continue" | "return") {
        format!(
            "{base} — `{name}` is not a Mycelium form; iterate by recursion or \
             `for x in xs, acc = init => body` (RFC-0007 §4.8)"
        )
    } else {
        base.to_owned()
    }
}

/// The v0 linear-recursion shape check (RFC-0007 §4.8): every constructor of `tname` is either
/// a **nil** (no fields) or a **cons** (exactly one spine field of type `tname` + exactly one
/// element field), with one element type across all cons constructors. Returns the element
/// type; anything else is an explicit refusal.
fn linear_elem_ty(
    site: &str,
    types: &BTreeMap<String, DataInfo>,
    tname: &str,
    targs: &[Ty],
) -> Result<Ty, CheckError> {
    let d = types
        .get(tname)
        .ok_or_else(|| CheckError::new(site, format!("unknown type `{tname}`")))?;
    // The declared element type is abstract over the type's parameters; instantiate it at the
    // scrutinee's type arguments (RFC-0007 §11.2) so `for` over a `List<Binary{8}>` binds `Binary{8}`.
    let s = param_subst(&d.params, targs);
    let mut elem: Option<Ty> = None;
    let mut has_cons = false;
    for c in &d.ctors {
        if c.fields.is_empty() {
            continue; // a nil — ends the spine
        }
        let (spine, rest): (Vec<&Ty>, Vec<&Ty>) = c
            .fields
            .iter()
            .partition(|f| matches!(f, Ty::Data(n, _) if n == tname));
        if spine.len() != 1 || rest.len() != 1 {
            return Err(CheckError::new(
                site,
                format!(
                    "`for` needs a linearly recursive type: constructor `{}` of `{tname}` must \
                     have exactly one `{tname}` field and one element field (general \
                     catamorphisms are an L2 concern — RFC-0007 §4.8)",
                    c.name
                ),
            ));
        }
        has_cons = true;
        let elem_ty = subst_ty(rest[0], &s);
        match &elem {
            None => elem = Some(elem_ty),
            Some(e) if *e == elem_ty => {}
            Some(e) => {
                return Err(CheckError::new(
                    site,
                    format!(
                        "`for` needs one element type across `{tname}`'s constructors: \
                         {e} vs {elem_ty}"
                    ),
                ))
            }
        }
    }
    if !has_cons {
        return Err(CheckError::new(
            site,
            format!("`{tname}` has no recursive constructor — nothing for `for` to iterate"),
        ));
    }
    Ok(elem.expect("has_cons implies an element type"))
}

/// Rebuild an [`Expr::App`] node from its head and (resolved) arguments.
fn app_node(head: &Expr, args: Vec<Expr>) -> Expr {
    Expr::App {
        head: Box::new(head.clone()),
        args,
    }
}

/// The paradigm name of a representation type (for the never-silent cross-paradigm framing).
fn paradigm_name(t: &Ty) -> Option<&'static str> {
    match t {
        Ty::Binary(_) => Some("Binary"),
        Ty::Ternary(_) => Some("Ternary"),
        Ty::Dense(_, _) => Some("Dense"),
        Ty::Data(_, _) | Ty::Substrate(_) | Ty::Var(_) => None,
    }
}

/// Frame a type-edge mismatch (RFC-0012 §4.4): a **cross-paradigm** edge (`want` and `got` are
/// representation types of *different* paradigms) becomes an explicit `MissingConversion` pointing
/// at writing a `swap` — the never-silent guarantee. A same-paradigm mismatch (e.g. two `Binary`
/// widths) keeps the plain wording.
fn edge_mismatch(edge: &str, want: &Ty, got: &Ty) -> String {
    match (paradigm_name(want), paradigm_name(got)) {
        (Some(w), Some(g)) if w != g => format!(
            "MissingConversion: {edge} is {got} but {want} is required — a cross-paradigm edge needs \
             an explicit `swap(…, to: {want}, policy: …)` (RFC-0012 §4.4; never silently converted)"
        ),
        _ => format!("{edge} has type {got}, expected {want}"),
    }
}

/// A bare-decimal-bearing prim family — every v0 prim is width-polymorphic and width-preserving, so
/// the result width (or a concrete operand's) anchors a bare-decimal operand (RFC-0012 §4.3).
#[derive(Clone, Copy)]
enum PrimFam {
    Binary,
    Ternary,
}

impl PrimFam {
    /// The width of `t` if it is this family's representation type, else `None`.
    fn width_of(self, t: &Ty) -> Option<u32> {
        match (self, t) {
            (PrimFam::Binary, Ty::Binary(w)) | (PrimFam::Ternary, Ty::Ternary(w)) => Some(*w),
            _ => None,
        }
    }

    /// This family's type at width `w`.
    fn ty(self, w: u32) -> Ty {
        match self {
            PrimFam::Binary => Ty::Binary(w),
            PrimFam::Ternary => Ty::Ternary(w),
        }
    }
}

/// The family of a builtin prim, or `None` if `name` is not a known prim.
fn prim_family(name: &str) -> Option<PrimFam> {
    Some(match name {
        "not" | "xor" => PrimFam::Binary,
        "add" | "sub" | "mul" | "neg" => PrimFam::Ternary,
        _ => return None,
    })
}

/// Encode a non-negative decimal `v` as an **unsigned** `Binary{width}` literal (MSB-first), or an
/// explicit refusal if it does not fit (RFC-0012 §4.3 — never a silent wrap/truncation).
fn encode_binary(site: &str, v: i64, width: u32) -> Result<Literal, CheckError> {
    if width == 0 {
        return Err(CheckError::new(
            site,
            "cannot encode a decimal at Binary{0} (zero width)",
        ));
    }
    let vu = u128::try_from(v).map_err(|_| {
        CheckError::new(
            site,
            format!("negative decimal `{v}` has no unsigned `Binary` encoding"),
        )
    })?;
    if width < 128 && (vu >> width) != 0 {
        return Err(CheckError::new(
            site,
            format!("decimal `{v}` does not fit Binary{{{width}}} (unsigned range 0..2^{width})"),
        ));
    }
    let mut s = String::with_capacity(width as usize);
    for i in (0..width).rev() {
        let bit = if i < 128 { (vu >> i) & 1 } else { 0 };
        s.push(if bit == 1 { '1' } else { '0' });
    }
    Ok(Literal::Bin(s))
}

/// Encode a decimal `v` as a **balanced-ternary** `Ternary{width}` literal (MSB-first, digits in
/// `{-,0,+}`), or an explicit refusal if it does not fit the symmetric range (RFC-0012 §4.3).
fn encode_balanced_ternary(site: &str, v: i64, width: u32) -> Result<Literal, CheckError> {
    if width == 0 {
        return Err(CheckError::new(
            site,
            "cannot encode a decimal at Ternary{0} (zero width)",
        ));
    }
    let mut n = i128::from(v);
    let mut digits: Vec<i8> = Vec::new(); // LSB-first, each in {-1, 0, 1}
    while n != 0 {
        let mut r = (n % 3) as i8;
        n /= 3;
        if r == 2 {
            r = -1;
            n += 1;
        } else if r == -2 {
            r = 1;
            n -= 1;
        }
        digits.push(r);
    }
    if digits.len() > width as usize {
        return Err(CheckError::new(
            site,
            format!(
                "decimal `{v}` does not fit Ternary{{{width}}} (balanced range ±(3^{width}-1)/2)"
            ),
        ));
    }
    digits.resize(width as usize, 0);
    let s: String = digits
        .iter()
        .rev()
        .map(|d| match d {
            -1 => '-',
            0 => '0',
            1 => '+',
            _ => unreachable!("a balanced trit is in {{-1, 0, 1}}"),
        })
        .collect();
    Ok(Literal::Trit(s))
}

/// The builtin prim signature table `Π` (RFC-0007 §4.4 T-Op), width-polymorphic. Surface names
/// map onto the trusted interpreter's registry (`bit.*`/`trit.*`).
#[must_use]
pub fn prim_sig(name: &str, args: &[Ty]) -> Option<Ty> {
    match (name, args) {
        ("not", [Ty::Binary(n)]) => Some(Ty::Binary(*n)),
        ("xor", [Ty::Binary(a), Ty::Binary(b)]) if a == b => Some(Ty::Binary(*a)),
        ("add" | "sub" | "mul", [Ty::Ternary(a), Ty::Ternary(b)]) if a == b => {
            Some(Ty::Ternary(*a))
        }
        ("neg", [Ty::Ternary(m)]) => Some(Ty::Ternary(*m)),
        _ => None,
    }
}

/// The surface→kernel prim-name mapping (the `Op` node's `prim` — RFC-0007 §4.1).
#[must_use]
pub fn prim_kernel_name(name: &str) -> Option<&'static str> {
    Some(match name {
        "not" => "bit.not",
        "xor" => "bit.xor",
        "add" => "trit.add",
        "sub" => "trit.sub",
        "mul" => "trit.mul",
        "neg" => "trit.neg",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn env(src: &str) -> Env {
        check_nodule(&parse(src).expect("parses")).expect("checks")
    }

    fn check_err(src: &str) -> CheckError {
        check_nodule(&parse(src).expect("parses")).expect_err("must fail to check")
    }

    // ---- M-666: `colony { hypha … }` type rule (RFC-0008 §4.7) ----

    #[test]
    fn a_colony_types_as_its_last_hypha() {
        // The colony's result type is the LAST hypha's (the RT2 sequentialization's observable). Here
        // the body must match the fn's `Binary{8}` return — the leading hyphae may be any type.
        let e = env(
            "nodule d\nfn compute(x: Binary{8}) -> Binary{8} = not(x)\n\
             fn run() -> Binary{8} = colony { hypha compute(0b0000_0001), hypha compute(0b0000_0010) }",
        );
        assert!(e.fn_decl("run").is_some());
    }

    #[test]
    fn a_colony_whose_last_hypha_mistypes_is_an_explicit_error() {
        // The last hypha carries the colony's type, so a `Ternary` last hypha under a `Binary{8}`
        // return is a never-silent body mismatch (the bidirectional check catches it).
        let err = check_err(
            "nodule d\nfn run() -> Binary{8} = colony { hypha not(0b0000_0001), hypha <00+0> }",
        );
        assert!(
            err.message.contains("body") || err.message.contains("expected"),
            "a mistyped last hypha must be an explicit edge mismatch, got: {}",
            err.message
        );
    }

    #[test]
    fn a_leading_hypha_that_does_not_type_check_is_still_an_error() {
        // RT4/I1: a leading hypha's refusal is never silently dropped — an ill-typed leading hypha
        // (an unknown name) fails the whole colony check.
        let err = check_err(
            "nodule d\nfn run() -> Binary{8} = colony { hypha nope(0b0), hypha not(0b0000_0001) }",
        );
        assert!(
            err.message.contains("nope") || err.message.contains("unknown"),
            "an ill-typed leading hypha must surface its error, got: {}",
            err.message
        );
    }

    #[test]
    fn check_error_at_is_a_public_alias() {
        // `::at` builds the same value the private `new` does (the canonical site+message struct).
        assert_eq!(
            CheckError::at("main", "boom"),
            CheckError::new("main", "boom"),
        );
    }

    #[test]
    fn env_getters_mirror_the_public_maps() {
        // A program with a data type and two functions, one recursive (so totality is filled).
        let e = env("nodule d\ntype Nat = Z | S(Nat)\n\
             fn count(n: Nat) -> Nat = match n { Z => Z, S(m) => S(count(m)) }\n\
             fn main() -> Nat = count(S(Z))");
        // type_info ⇔ types.get
        assert_eq!(e.type_info("Nat"), e.types.get("Nat"));
        assert!(e.type_info("Nat").is_some());
        assert!(e.type_info("Nope").is_none());
        // fn_decl ⇔ fns.get
        assert_eq!(e.fn_decl("count"), e.fns.get("count"));
        assert!(e.fn_decl("count").is_some());
        assert!(e.fn_decl("absent").is_none());
        // fn_totality ⇔ totality.get (copied)
        assert_eq!(e.fn_totality("count"), e.totality.get("count").copied());
        assert!(e.fn_totality("count").is_some());
        assert!(e.fn_totality("absent").is_none());
    }
}

#[cfg(test)]
mod depth_budget_tests {
    use super::*;
    use crate::ast::{BaseType, Expr, FnDecl, FnSig, Item, Literal, Nodule, Path, TypeRef};

    /// A `not(not(… not(0b0) …))` nest `depth` deep — built directly (the parser caps surface nesting
    /// at `MAX_EXPR_DEPTH`, so a direct AST is the way to exercise the *checker's* own budget).
    pub(crate) fn deep_not(depth: usize) -> Expr {
        let mut e = Expr::Lit(Literal::Bin("0".to_string()));
        for _ in 0..depth {
            e = Expr::App {
                head: Box::new(Expr::Path(Path(vec!["not".to_string()]))),
                args: vec![e],
            };
        }
        e
    }

    pub(crate) fn nodule_with_body(body: Expr) -> Nodule {
        Nodule {
            path: Path(vec!["d".to_string()]),
            items: vec![Item::Fn(FnDecl {
                thaw: false,
                sig: FnSig {
                    name: "main".to_string(),
                    params: vec![],
                    value_params: vec![],
                    ret: TypeRef {
                        base: BaseType::Binary(1),
                        guarantee: None,
                    },
                },
                body,
            })],
        }
    }

    #[test]
    fn the_depth_budget_trips_cleanly_and_just_under_it_succeeds() {
        // Just under the budget: the checker completes — the deep worker stack ([`mycelium_stack`])
        // absorbs `MAX_CHECK_DEPTH` levels with large margin (measured physical ceiling ≫ budget).
        let ok = check_nodule(&nodule_with_body(deep_not((MAX_CHECK_DEPTH - 5) as usize)));
        assert!(ok.is_ok(), "just under the budget should check ok: {ok:?}");
        // Past the budget: a clean, explicit refusal — never a host-stack overflow (banked guard 4).
        let err = check_nodule(&nodule_with_body(deep_not((MAX_CHECK_DEPTH + 50) as usize)))
            .expect_err("past the budget must refuse");
        assert!(
            err.message.contains("depth budget"),
            "expected the explicit depth-budget refusal, got: {}",
            err.message
        );
    }
}
